use std::path::PathBuf;
use std::time::SystemTime;

use agent_client_protocol as acp;
use pretty_assertions::assert_eq;
use xai_grok_tools::computer::types::TaskKind;

use super::*;

fn notification(output: &str) -> SessionNotification {
    SessionNotification {
        session_id: acp::SessionId::new("test-session"),
        update: SessionUpdate::TaskCompleted {
            task_snapshot: TaskSnapshot {
                task_id: "bg-1".to_string(),
                command: "grep -r pattern .".to_string(),
                display_command: None,
                cwd: "/workspace".to_string(),
                start_time: SystemTime::now(),
                end_time: Some(SystemTime::now()),
                output: output.to_string(),
                output_file: PathBuf::from("/tmp/bg-1.log"),
                truncated: false,
                output_total_bytes: output.len(),
                exit_code: Some(0),
                signal: None,
                completed: true,
                block_waited: false,
                explicitly_killed: false,
                kill_result_delivered: false,
                kind: TaskKind::Bash,
                owner_session_id: None,
                description: None,
                is_backgrounded: true,
            },
            will_wake: false,
        },
        meta: None,
    }
}

fn frame_len(params: &RawValue) -> usize {
    WRAPPER_BYTES + METHOD.len() + params.get().len()
}

#[test]
fn small_output_is_untouched() {
    let mut notification = notification("all done\n");
    let params = encode(&mut notification).unwrap();

    assert!(frame_len(&params) <= FRAME_MAX_BYTES);
    let snapshot = task_snapshot(&mut notification).unwrap();
    assert_eq!(snapshot.output, "all done\n");
    assert!(!snapshot.truncated);
}

#[test]
fn a_multi_megabyte_log_fits_and_points_at_the_file() {
    let mut notification = notification(&"Z".repeat(2 * 1024 * 1024));
    let params = encode(&mut notification).unwrap();

    assert!(frame_len(&params) <= FRAME_MAX_BYTES);
    let snapshot = task_snapshot(&mut notification).unwrap();
    assert!(snapshot.truncated);
    assert!(snapshot.output.contains("/tmp/bg-1.log"));
    assert_eq!(snapshot.output_total_bytes, 2 * 1024 * 1024);
}

#[test]
fn escaped_output_fits_too() {
    let mut notification = notification(&"\u{7}".repeat(30 * 1024));
    let params = encode(&mut notification).unwrap();

    assert!(frame_len(&params) <= FRAME_MAX_BYTES);
    assert!(!task_snapshot(&mut notification).unwrap().output.is_empty());
}

#[test]
fn oversized_non_output_fields_are_capped_too() {
    let mut notification = notification("");
    {
        let snapshot = task_snapshot(&mut notification).unwrap();
        snapshot.command = "\u{7}".repeat(80 * 1024);
        snapshot.description = Some("d".repeat(80 * 1024));
        snapshot.cwd = format!("/{}", "c".repeat(80 * 1024));
    }
    let params = encode(&mut notification).unwrap();

    assert!(frame_len(&params) <= FRAME_MAX_BYTES);
    assert!(encoded_len(&task_snapshot(&mut notification).unwrap().command) <= FIELD_MAX_BYTES);
}

#[test]
fn a_long_log_path_still_fits() {
    let mut notification = notification(&"Z".repeat(64 * 1024));
    task_snapshot(&mut notification).unwrap().output_file =
        PathBuf::from(format!("/tmp/{}/task.log", "p".repeat(30 * 1024)));
    let params = encode(&mut notification).unwrap();

    assert!(frame_len(&params) <= FRAME_MAX_BYTES);
    assert!(
        task_snapshot(&mut notification)
            .unwrap()
            .output_file
            .display()
            .to_string()
            .ends_with("/task.log")
    );
}

#[test]
fn the_encoded_message_matches_the_persisted_notification() {
    let mut notification = notification(&"Z".repeat(2 * 1024 * 1024));
    let params = encode(&mut notification).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(params.get()).unwrap(),
        serde_json::to_value(&notification).unwrap()
    );
}

#[test]
fn a_message_that_cannot_fit_at_all_is_not_returned() {
    let mut notification = notification("hello");
    task_snapshot(&mut notification).unwrap().task_id = "t".repeat(FRAME_MAX_BYTES);
    assert!(encode(&mut notification).is_none());
}

#[test]
fn an_old_record_with_oversized_fields_is_refit_not_dropped() {
    let mut record = serde_json::to_value(notification(&"Z".repeat(64 * 1024))).unwrap();
    record["update"]["task_snapshot"]["output_file"] =
        serde_json::Value::String(format!("/tmp/{}/task.log", "p".repeat(80 * 1024)));
    record["update"]["task_snapshot"]["command"] = serde_json::Value::String("c".repeat(80 * 1024));
    record["future_field"] = serde_json::json!({"kept": true});
    let raw = serde_json::value::to_raw_value(&record).unwrap();

    match refit_recorded(&raw) {
        Refit::Fitted(fitted) => {
            assert!(fitted.get().len() <= body_budget());
            assert!(fitted.get().contains("future_field"));
        }
        Refit::Unchanged | Refit::Unfittable => panic!("oversized record must be refit"),
    }
}

#[test]
fn the_reservation_matches_the_transport_line() {
    let body = "{}";
    let line = format!(r#"{{"jsonrpc":"2.0","method":"_{METHOD}","params":{body}}}"#) + "\n";
    assert_eq!(line.len() - body.len(), WRAPPER_BYTES + METHOD.len());
    assert!("x.ai/session/update".len() <= METHOD.len());
}

#[test]
fn encoded_len_matches_the_encoder() {
    for text in [
        "plain",
        "quote\" and backslash\\",
        "newline\ntab\treturn\r",
        "control\u{1}\u{7}\u{1f}",
        "unicode: 日本語 🎉",
    ] {
        let written = serde_json::to_string(text).unwrap();
        assert_eq!(
            encoded_len(text) + 2,
            written.len(),
            "mismatch for {text:?}"
        );
    }
}
