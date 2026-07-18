//! Hermetic production-contract test for the fork-owned release installer.

#![cfg(unix)]

mod common;

use serial_test::serial;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{host_platform, reset_home, set_test_version, small_good_artifact, test_home};
use xai_grok_update::auto_update::install_gh_release_for_test;

#[tokio::test]
#[serial]
async fn pinned_upgrade_downloads_only_the_exact_enhanced_asset() {
    let home = test_home();
    reset_home();
    set_test_version("9.9.8");

    let server = MockServer::start().await;
    let version = "9.9.9";
    let asset = format!("grok-{version}-{}", host_platform());
    let release_path = "/releases";
    let download_path = format!("/download/v{version}/{asset}");
    let binary = small_good_artifact();

    Mock::given(method("GET"))
        .and(path(release_path))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "tag_name": format!("v{version}"),
                "draft": false,
                "prerelease": false,
                "assets": [{"name": asset}]
            }
        ])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("HEAD"))
        .and(path(download_path.as_str()))
        .respond_with(
            ResponseTemplate::new(200).insert_header("content-length", binary.len().to_string()),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(download_path.as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(binary))
        .expect(1)
        .mount(&server)
        .await;

    install_gh_release_for_test(
        Some(version),
        &format!("{}{release_path}", server.uri()),
        &format!("{}/download", server.uri()),
    )
    .await
    .expect("the exact fork asset should install");

    let managed = home.join("bin/grok");
    assert!(managed.exists(), "managed grok link should resolve");
    assert!(
        std::process::Command::new(&managed)
            .arg("--version")
            .status()
            .expect("installed test artifact should execute")
            .success()
    );
    let config = std::fs::read_to_string(home.join("config.toml")).unwrap();
    assert!(config.contains("installer = \"gh-release\""), "{config}");
}
