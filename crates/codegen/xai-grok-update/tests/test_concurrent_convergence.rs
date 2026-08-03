//! Lock-free managed-install integrity tests.
//!
//! Disk-aware fork-release convergence is covered with an injected resolved
//! version in `test_downgrade_matrix`; exact fork asset installation is covered
//! with WireMock in `test_fork_release_routing`. This suite retains the
//! filesystem probe invariants and exercises the inherited low-level installer
//! concurrently, without reaching production release metadata or a legacy CLI.
//!
//! The same-instant race is accepted as rare; these tests pin the property
//! that makes it harmless — concurrent installs (same or *different* versions)
//! never corrupt the active binary. Before the per-attempt temp-name fix, every
//! `0.1.x` download shared one `grok-0.1.tmp` (`with_extension("tmp")` eats
//! everything after the last dot), so racer A could atomically rename racer
//! B's half-written file into place.

#![cfg(unix)]

mod common;

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use serial_test::serial;

use common::artifact_server::ArtifactServer;
use common::{
    can_exec_shell_scripts, host_platform, make_update_config, reset_home, small_good_artifact,
    test_home,
};
use xai_grok_update::auto_update::install_internal_from_base;
use xai_grok_update::version::installed_on_disk_version;

/// Assert the active `~/.grok/bin/grok` resolves to the expected versioned
/// binary, actually runs, and has exactly the expected content (the content
/// check is what catches a cross-racer temp-file corruption).
fn assert_active_binary(home: &Path, version: &str, platform: &str, expected_content: &[u8]) {
    let link = home.join("bin").join("grok");
    assert!(link.is_symlink(), "grok must be a symlink");
    let resolved = dunce::canonicalize(&link)
        .unwrap_or_else(|e| panic!("active grok symlink does not resolve: {e}"));
    assert_eq!(
        resolved.file_name().unwrap().to_string_lossy(),
        format!("grok-{version}-{platform}"),
        "active grok must be the expected version"
    );
    assert_eq!(
        std::fs::read(&resolved).unwrap(),
        expected_content,
        "active binary content must be exactly the served artifact (no \
         partial/interleaved writes from a racing updater)"
    );
    let ran_ok = std::process::Command::new(&resolved)
        .arg("--version")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    assert!(ran_ok, "active grok must pass the smoke-test");
}

/// Lay down a managed-install layout in the test GROK_HOME:
/// `bin/grok -> ../downloads/grok-<version>-<platform>` (what
/// `install_internal_from_base` produces).
fn fake_managed_install(version: &str) {
    let home = test_home();
    let downloads = home.join("downloads");
    let bin = home.join("bin");
    std::fs::create_dir_all(&downloads).unwrap();
    std::fs::create_dir_all(&bin).unwrap();
    let name = format!("grok-{version}-{}", host_platform());
    std::fs::write(downloads.join(&name), small_good_artifact()).unwrap();
    std::fs::set_permissions(
        downloads.join(&name),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    std::os::unix::fs::symlink(
        std::path::Path::new("../downloads").join(&name),
        bin.join("grok"),
    )
    .unwrap();
}

// Production convergence and exact fork-asset download behavior are covered
// hermetically in `test_downgrade_matrix` and `test_fork_release_routing`.

#[tokio::test]
#[serial]
async fn disk_probe_preserves_prerelease_versions() {
    let _ = test_home();
    reset_home();
    // An alpha install must read back as the full pre-release version —
    // truncating to "0.1.220" would mask the alpha → stable update.
    fake_managed_install("0.1.220-alpha.4");
    assert_eq!(
        installed_on_disk_version().as_deref(),
        Some("0.1.220-alpha.4")
    );
}

#[tokio::test]
#[serial]
async fn disk_probe_rejects_dangling_symlink() {
    // If the symlink survives but its target binary was deleted (manual
    // ~/.grok/downloads cleanup), the probe must report None — otherwise
    // every updater would claim "already up to date" forever while no
    // runnable binary exists, and nothing would ever repair the install.
    let home = test_home();
    reset_home();
    let platform = host_platform();
    fake_managed_install("0.2.7");
    assert_eq!(installed_on_disk_version().as_deref(), Some("0.2.7"));

    std::fs::remove_file(
        home.join("downloads")
            .join(format!("grok-0.2.7-{platform}")),
    )
    .unwrap();

    assert_eq!(
        installed_on_disk_version(),
        None,
        "a dangling symlink must not report an installed version"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Race integrity: the accepted same-instant race must stay harmless. Two (or
// three) installers running concurrently — even for DIFFERENT versions —
// must never leave a corrupt active binary. Pre-fix, all 0.1.x downloads
// shared one `grok-0.1.tmp`, so a concurrent racer could atomically rename a
// half-written file into place.
// ─────────────────────────────────────────────────────────────────────────────

async fn run_concurrent_installs(
    server: &ArtifactServer,
    versions: &[&str],
) -> Vec<anyhow::Result<()>> {
    let base = server.uri();
    let mut tasks = Vec::new();
    for version in versions {
        let base = base.clone();
        let version = version.to_string();
        tasks.push(tokio::spawn(async move {
            let cfg = make_update_config("stable");
            install_internal_from_base(Some(&version), &cfg, &base).await
        }));
    }
    let mut results = Vec::new();
    for t in tasks {
        results.push(t.await.expect("install task must not panic"));
    }
    results
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn concurrent_same_version_installs_leave_valid_active_binary() {
    if !can_exec_shell_scripts() {
        eprintln!("skipping: shell scripts cannot execute in this sandbox");
        return;
    }
    let home = test_home();
    reset_home();
    let platform = host_platform();
    let artifact = small_good_artifact();
    let server = ArtifactServer::start(artifact.clone());
    // Hold responses open so the racers genuinely overlap mid-download.
    server.set_slow(true);

    let results = run_concurrent_installs(&server, &["0.1.181", "0.1.181", "0.1.181"]).await;
    for r in results {
        r.expect("every racing install must succeed (atomic swap, last writer wins)");
    }

    // Lock-free model: concurrent racers may each download (accepted waste);
    // the invariant is integrity, not the count.
    assert!(server.request_count() >= 1);
    assert_active_binary(home, "0.1.181", &platform, &artifact);
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn concurrent_different_version_installs_do_not_corrupt_each_other() {
    if !can_exec_shell_scripts() {
        eprintln!("skipping: shell scripts cannot execute in this sandbox");
        return;
    }
    let home = test_home();
    reset_home();
    let platform = host_platform();
    let artifact = small_good_artifact();
    let server = ArtifactServer::start(artifact.clone());
    server.set_slow(true);

    // Pre-fix, BOTH of these wrote to downloads/grok-0.1.tmp concurrently
    // (with_extension("tmp") truncates at the last dot), so one racer could
    // rename the other's partial file into its own versioned path.
    let results = run_concurrent_installs(&server, &["0.1.181", "0.1.182"]).await;
    for r in results {
        r.expect("both racing installs must succeed");
    }

    // Both versioned binaries must exist with full, uncorrupted content.
    for version in ["0.1.181", "0.1.182"] {
        let path = home
            .join("downloads")
            .join(format!("grok-{version}-{platform}"));
        assert_eq!(
            std::fs::read(&path).unwrap(),
            artifact,
            "binary {version} must contain exactly the served artifact"
        );
    }

    // The active symlink points at whichever racer swapped last; it must
    // resolve and run regardless.
    let resolved = dunce::canonicalize(home.join("bin").join("grok")).unwrap();
    assert_eq!(std::fs::read(&resolved).unwrap(), artifact);
    let name = resolved.file_name().unwrap().to_string_lossy().to_string();
    assert!(
        !name.contains(".tmp"),
        "active grok must never be a temp file: {name}"
    );

    // No stray shared temp file left behind (the pre-fix collision name).
    assert!(
        !home.join("downloads").join("grok-0.1.tmp").exists(),
        "the pre-fix shared temp name must not exist"
    );
}
