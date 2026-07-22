// Per-test-case module for the `pty_e2e` integration test crate.
#[allow(unused_imports)]
use crate::common::*;

/// A saved `[ui] screen_mode = "minimal"` preference makes a later plain
/// launch open in minimal mode. CLI and slash-command switches are tested
/// separately as session-scoped overrides that must not mutate this setting.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn minimal_sticky_screen_mode_persists() {
    let content = ContentController::start().await.expect("start content");
    content.set_response(format!("{} saved-default payload.", turn_sentinel(1)));

    let project = tempfile::tempdir().expect("create project dir");
    std::fs::create_dir_all(project.path().join(".git")).expect("create .git");

    let grok_home = content.home().join(".grok");
    std::fs::create_dir_all(&grok_home).expect("create .grok");
    std::fs::write(
        grok_home.join("config.toml"),
        "[ui]\nscreen_mode = \"minimal\"\n",
    )
    .expect("write saved screen-mode default");

    let binary = pager_binary().expect("resolve pager binary");
    let mut harness = PtyHarness::spawn_with_content_in_dir(
        &binary,
        DEFAULT_ROWS,
        DEFAULT_COLS,
        &content,
        &["--no-leader"],
        Some(project.path()),
    )
    .expect("spawn plain pager");
    harness.set_respond_to_queries(true);

    wait_minimal_ready(&mut harness);
    assert!(
        !harness.contains_text(WELCOME_SCREEN_SENTINEL),
        "saved minimal default must bypass the fullscreen welcome screen\nscreen:\n{}",
        harness.screen_contents()
    );
    assert!(
        !harness.contains_text("panicked"),
        "pager panicked\nscreen:\n{}",
        harness.screen_contents()
    );

    quit_minimal(&mut harness);
}
