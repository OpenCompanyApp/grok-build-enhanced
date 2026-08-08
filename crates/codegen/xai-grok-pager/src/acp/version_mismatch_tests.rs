use super::{is_version_mismatch_banner, version_mismatch_banner};
use crate::glyphs::sanitize_toast_message;

fn expected_banner(client: &str, leader: &str) -> String {
    sanitize_toast_message(&format!(
        "⚠ Version mismatch: client {client}, leader {leader} — restart grok to match"
    ))
    .into_owned()
}

#[test]
fn formats_versions_and_ignores_wire_message() {
    assert_eq!(
        version_mismatch_banner(
            r#"{"clientVersion":"0.1.157","leaderVersion":"0.1.150","message":"ignore"}"#
        ),
        Some(expected_banner("0.1.157", "0.1.150"))
    );
}

#[test]
fn rejects_unusable_payloads() {
    for params in [
        "{}",
        r#"{"clientVersion":"0.1.157"}"#,
        r#"{"leaderVersion":"0.1.150"}"#,
        r#"{"clientVersion":"","leaderVersion":"0.1.150"}"#,
        r#"{"clientVersion":"\n\t","leaderVersion":"0.1.150"}"#,
        "null",
        "[]",
    ] {
        assert_eq!(version_mismatch_banner(params), None, "{params}");
    }
}

#[test]
fn scrubs_control_chars_and_keeps_marker() {
    let text = version_mismatch_banner(
        r#"{"clientVersion":"0.1.157\n\u0007x","leaderVersion":"0.1.150\r\n"}"#,
    )
    .expect("banner");
    assert!(!text.chars().any(char::is_control));
    assert!(is_version_mismatch_banner(&text));
}
