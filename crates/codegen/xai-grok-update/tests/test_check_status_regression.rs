//! Hermetic end-to-end regressions for `grok update --check --json`.
//!
//! Enhanced normalizes legacy npm/internal provenance to the fork-owned
//! GitHub Release channel. These tests lock in the user-visible status shape,
//! exact native-asset requirement, and no-fallback behavior without consulting
//! public release metadata or invoking a legacy package-manager CLI.

#![cfg(unix)]

mod common;

use serial_test::serial;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{FakeBinGuard, host_platform, reset_home, set_test_version, test_home};
use xai_grok_update::UpdateConfig;
use xai_grok_update::auto_update::check_update_status_from_api_for_test;

fn make_update_config() -> UpdateConfig {
    UpdateConfig {
        proxy_base_url: "http://test.invalid/v1".to_string(),
        auth_scope: "test".to_string(),
        deployment_key: None,
        alpha_test_key: None,
        channel: "stable".to_string(),
        npm_registry: None,
    }
}

fn host_os_arch() -> (&'static str, &'static str) {
    let os = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        panic!("unsupported test platform");
    };
    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        panic!("unsupported test architecture");
    };
    (os, arch)
}

fn setup_legacy_npm(current_version: &str) -> FakeBinGuard {
    let _ = test_home();
    reset_home();
    set_test_version(current_version);
    // SAFETY: every caller is serial and reset_home clears the variable.
    unsafe { std::env::set_var("GROK_INSTALLER", "npm") };
    FakeBinGuard::install_npm()
}

async fn mount_release(version: &str) -> MockServer {
    let server = MockServer::start().await;
    let asset = format!("grok-{version}-{}", host_platform());
    Mock::given(method("GET"))
        .and(path("/releases"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([{
                "tag_name": format!("v{version}"),
                "draft": false,
                "prerelease": false,
                "assets": [{"name": asset}]
            }])),
        )
        .expect(1)
        .mount(&server)
        .await;
    server
}

async fn check_against(server: &MockServer) -> xai_grok_update::auto_update::UpdateStatus {
    let (os, arch) = host_os_arch();
    check_update_status_from_api_for_test(
        &make_update_config(),
        &format!("{}/releases", server.uri()),
        os,
        arch,
    )
    .await
}

#[tokio::test]
#[serial]
async fn legacy_npm_provenance_uses_fork_release_api_and_never_npm() {
    let npm = setup_legacy_npm("9.9.8");
    let server = mount_release("9.9.9").await;

    let status = check_against(&server).await;
    let json = serde_json::to_value(&status).unwrap();

    assert_eq!(status.current_version, "9.9.8");
    assert_eq!(status.latest_version.as_deref(), Some("9.9.9"));
    assert!(status.update_available);
    assert_eq!(status.installer.as_deref(), Some("gh-release"));
    assert_eq!(status.channel, "stable");
    assert!(status.error.is_none());

    // Lock in the public key names used by `grok update --check --json`.
    for key in [
        "currentVersion",
        "latestVersion",
        "updateAvailable",
        "installer",
        "channel",
        "autoUpdate",
        "error",
    ] {
        assert!(json.get(key).is_some(), "missing JSON key {key}: {json}");
    }
    assert!(npm.args_log().is_empty(), "npm must never be consulted");
}

#[tokio::test]
#[serial]
async fn check_status_never_advertises_a_release_rollback() {
    let npm = setup_legacy_npm("9.9.9");
    let server = mount_release("9.9.8").await;

    let status = check_against(&server).await;

    assert_eq!(status.latest_version.as_deref(), Some("9.9.8"));
    assert!(!status.update_available);
    assert_eq!(status.installer.as_deref(), Some("gh-release"));
    assert!(status.error.is_none());
    assert!(npm.args_log().is_empty(), "npm must never be consulted");
}

#[tokio::test]
#[serial]
async fn fork_release_error_is_visible_without_a_legacy_fallback() {
    let npm = setup_legacy_npm("9.9.8");
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/releases"))
        .respond_with(ResponseTemplate::new(403).set_body_string("forbidden"))
        .expect(1)
        .mount(&server)
        .await;

    let status = check_against(&server).await;
    let json = serde_json::to_value(&status).unwrap();

    assert!(status.latest_version.is_none());
    assert!(!status.update_available);
    assert_eq!(status.installer.as_deref(), Some("gh-release"));
    let error = status
        .error
        .as_deref()
        .expect("release failure must be shown");
    assert!(error.contains("HTTP 403"), "unexpected error: {error}");
    assert!(json["latestVersion"].is_null());
    assert!(
        json["error"]
            .as_str()
            .is_some_and(|value| value.contains("HTTP 403"))
    );
    assert!(npm.args_log().is_empty(), "npm fallback is forbidden");
}
