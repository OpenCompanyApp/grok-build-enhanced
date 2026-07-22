use super::*;
use crate::session::info::Info;
use crate::session::persistence::default_model_id;
use crate::session::storage::{SessionUpdate, StorageAdapter};

fn info() -> Info {
    Info {
        id: acp::SessionId::new("durable-jsonl"),
        cwd: "/test".into(),
    }
}

fn update(info: &Info, text: String) -> SessionUpdate {
    SessionUpdate::Acp(Box::new(acp::SessionNotification::new(
        info.id.clone(),
        acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk::new(acp::ContentBlock::Text(
            acp::TextContent::new(text),
        ))),
    )))
}

fn user_update(info: &Info, text: &str) -> SessionUpdate {
    SessionUpdate::Acp(Box::new(acp::SessionNotification::new(
        info.id.clone(),
        acp::SessionUpdate::UserMessageChunk(acp::ContentChunk::new(acp::ContentBlock::Text(
            acp::TextContent::new(text),
        ))),
    )))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ordinary_and_durable_appends_keep_every_physical_line_parseable() {
    const N: usize = 100;
    let dir = tempfile::tempdir().unwrap();
    let info = info();
    let adapter = JsonlStorageAdapter::with_explicit_session_dir(dir.path().to_path_buf());
    adapter
        .init_session(&info, default_model_id())
        .await
        .unwrap();
    let ordinary = adapter.clone();
    let durable = adapter.clone();
    let info_a = info.clone();
    let info_b = info.clone();
    let ordinary = tokio::spawn(async move {
        for index in 0..N {
            ordinary
                .append_update(&info_a, &update(&info_a, format!("ordinary-{index}")))
                .await
                .unwrap();
        }
    });
    let durable = tokio::spawn(async move {
        for index in 0..N {
            durable
                .append_update_durable_commit_aware(
                    &info_b,
                    &update(&info_b, format!("durable-{index}")),
                )
                .await
                .unwrap();
        }
    });
    ordinary.await.unwrap();
    durable.await.unwrap();

    let bytes = std::fs::read(dir.path().join("updates.jsonl")).unwrap();
    let parsed = bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(serde_json::from_slice::<SessionUpdateEnvelope>)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(parsed.len(), N * 2);
}

#[tokio::test]
async fn append_commit_is_reported_when_bookkeeping_fails() {
    let dir = tempfile::tempdir().unwrap();
    let info = info();
    let adapter = JsonlStorageAdapter::with_explicit_session_dir(dir.path().to_path_buf());
    adapter
        .init_session(&info, default_model_id())
        .await
        .unwrap();
    let summary = dir.path().join("summary.json");
    std::fs::remove_file(&summary).unwrap();
    std::fs::create_dir(&summary).unwrap();

    assert!(matches!(
        adapter
            .append_update_durable_commit_aware(&info, &update(&info, "committed".into()))
            .await,
        Err(crate::session::storage::AppendUpdateError::Committed(_))
    ));
    let bytes = std::fs::read(dir.path().join("updates.jsonl")).unwrap();
    let parsed = bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(serde_json::from_slice::<SessionUpdateEnvelope>)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(parsed.len(), 1);
}

#[tokio::test]
async fn chat_rebuild_restores_compacted_history_and_auto_continue() {
    use crate::extensions::notification::{
        AutoContinueInfo, CompactionCheckpointFile, CompactionCheckpointInfo,
        SessionNotification as XaiNotification, SessionUpdate as XaiUpdate,
    };

    let dir = tempfile::tempdir().unwrap();
    let info = info();
    let adapter = JsonlStorageAdapter::with_explicit_session_dir(dir.path().to_path_buf());
    adapter
        .init_session(&info, default_model_id())
        .await
        .unwrap();
    let checkpoint_id = "019f7bdf-c293-7b92-a5a7-5464711476f2";
    let checkpoint_dir = dir
        .path()
        .join(crate::session::storage::COMPACTION_CHECKPOINTS_DIR);
    std::fs::create_dir_all(&checkpoint_dir).unwrap();
    let checkpoint = CompactionCheckpointFile {
        checkpoint_id: checkpoint_id.to_owned(),
        prompt_index_at_compaction: 4,
        compacted_history: vec![crate::sampling::ConversationItem::user("compacted summary")],
        schema_version: 1,
        created_at: "2026-07-20T00:00:00Z".to_owned(),
        original_user_info: None,
        reread_file_paths: Vec::new(),
    };
    std::fs::write(
        checkpoint_dir.join(format!("{checkpoint_id}.json")),
        serde_json::to_vec(&checkpoint).unwrap(),
    )
    .unwrap();

    let marker = SessionUpdate::Xai(Box::new(XaiNotification {
        session_id: info.id.clone(),
        update: XaiUpdate::CompactionCheckpoint(Box::new(CompactionCheckpointInfo {
            checkpoint_id: checkpoint_id.to_owned(),
            prompt_index_at_compaction: 4,
            checkpoint_file: format!(
                "{}/{checkpoint_id}.json",
                crate::session::storage::COMPACTION_CHECKPOINTS_DIR
            ),
            auto_continue: Some(AutoContinueInfo {
                prompt_text: "continue after compact".to_owned(),
            }),
            schema_version: 1,
            created_at: "2026-07-20T00:00:00Z".to_owned(),
        })),
        meta: None,
    }));
    adapter.append_update(&info, &marker).await.unwrap();
    adapter
        .append_update(&info, &update(&info, "post compact response".into()))
        .await
        .unwrap();

    let count = crate::session::storage::chat_rebuild::rebuild_chat_history(dir.path()).unwrap();
    let rebuilt = std::fs::read_to_string(dir.path().join("chat_history.jsonl")).unwrap();
    let items = rebuilt
        .lines()
        .map(serde_json::from_str::<crate::sampling::ConversationItem>)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(count, 3);
    assert_eq!(
        serde_json::to_value(items).unwrap(),
        serde_json::to_value(vec![
            crate::sampling::ConversationItem::user("compacted summary"),
            crate::sampling::ConversationItem::user("continue after compact"),
            crate::sampling::ConversationItem::assistant("post compact response"),
        ])
        .unwrap()
    );
}

#[tokio::test]
async fn chat_rebuild_discards_rewound_dead_branch() {
    use crate::extensions::notification::{
        SessionNotification as XaiNotification, SessionUpdate as XaiUpdate,
    };

    let dir = tempfile::tempdir().unwrap();
    let info = info();
    let adapter = JsonlStorageAdapter::with_explicit_session_dir(dir.path().to_path_buf());
    adapter
        .init_session(&info, default_model_id())
        .await
        .unwrap();
    for update in [
        user_update(&info, "kept prompt"),
        update(&info, "kept response".into()),
        user_update(&info, "dead prompt"),
        update(&info, "dead response".into()),
    ] {
        adapter.append_update(&info, &update).await.unwrap();
    }
    let rewind = SessionUpdate::Xai(Box::new(XaiNotification {
        session_id: info.id.clone(),
        update: XaiUpdate::RewindMarker {
            target_prompt_index: 1,
            created_at: "2026-07-20T00:00:00Z".to_owned(),
        },
        meta: None,
    }));
    adapter.append_update(&info, &rewind).await.unwrap();
    for update in [
        user_update(&info, "replacement prompt"),
        update(&info, "replacement response".into()),
    ] {
        adapter.append_update(&info, &update).await.unwrap();
    }

    crate::session::storage::chat_rebuild::rebuild_chat_history(dir.path()).unwrap();
    let rebuilt = std::fs::read_to_string(dir.path().join("chat_history.jsonl")).unwrap();
    let items = rebuilt
        .lines()
        .map(serde_json::from_str::<crate::sampling::ConversationItem>)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(
        serde_json::to_value(items).unwrap(),
        serde_json::to_value(vec![
            crate::sampling::ConversationItem::user("kept prompt"),
            crate::sampling::ConversationItem::assistant("kept response"),
            crate::sampling::ConversationItem::user("replacement prompt"),
            crate::sampling::ConversationItem::assistant("replacement response"),
        ])
        .unwrap()
    );
}

#[test]
fn checkpoint_reference_rejects_noncanonical_paths() {
    let dir = tempfile::tempdir().unwrap();
    let error = crate::session::storage::compaction_checkpoint_path(
        dir.path(),
        "019f7bdf-c293-7b92-a5a7-5464711476f2",
        "../../auth.json",
    )
    .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
}

#[test]
fn lock_serializes_tail_heal_and_complete_record() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("updates.jsonl");
    std::fs::write(&path, b"torn").unwrap();
    JsonlStorageAdapter::append_jsonl_line_sync(
        &path,
        b"{\"record\":1}\n".to_vec(),
        AppendDurability::Buffered,
    )
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(path).unwrap(),
        "torn\n{\"record\":1}\n"
    );
}

#[test]
fn directory_barrier_failure_is_retried_even_after_file_exists() {
    let mut attempts = 0;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("updates.jsonl");
    let mut flaky_parent = || {
        attempts += 1;
        if attempts == 1 {
            Err(io::Error::other("directory barrier failed"))
        } else {
            Ok(())
        }
    };

    assert!(
        JsonlStorageAdapter::append_jsonl_line_sync_with(
            &path,
            b"{\"record\":1}\n".to_vec(),
            AppendDurability::Durable,
            std::fs::File::sync_all,
            &mut flaky_parent,
        )
        .is_err()
    );
    JsonlStorageAdapter::append_jsonl_line_sync_with(
        &path,
        b"{\"record\":1}\n".to_vec(),
        AppendDurability::Durable,
        std::fs::File::sync_all,
        &mut flaky_parent,
    )
    .unwrap();
    assert_eq!(attempts, 2);
}

#[test]
fn file_barrier_error_propagates() {
    fn fail(_file: &std::fs::File) -> io::Result<()> {
        Err(io::Error::other("file barrier failed"))
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("updates.jsonl");
    let error = JsonlStorageAdapter::append_jsonl_line_sync_with(
        &path,
        b"{\"record\":1}\n".to_vec(),
        AppendDurability::Durable,
        fail,
        || Ok(()),
    )
    .unwrap_err();
    assert_eq!(error.to_string(), "file barrier failed");
}

#[test]
fn cwd_switch_retry_after_post_append_barrier_failure_is_already_present() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("chat_history.jsonl");
    let item = ConversationItem::working_directory_switch("moved", 3);
    let mut line = serde_json::to_vec(&item).unwrap();
    line.push(b'\n');

    assert!(matches!(
        JsonlStorageAdapter::append_cwd_switch_line_sync_with(
            &path,
            line.clone(),
            3,
            |_| Err(io::Error::other("file barrier failed")),
            || Ok(()),
        ),
        Err(crate::session::storage::AppendCwdSwitchError::Committed {
            acknowledgement: xai_chat_state::StrictAppendAck::Appended,
            ..
        })
    ));
    assert!(matches!(
        JsonlStorageAdapter::append_cwd_switch_line_sync_with(
            &path,
            line,
            3,
            |_| Ok(()),
            || Ok(()),
        )
        .unwrap(),
        xai_chat_state::StrictAppendAck::AlreadyPresent(item)
            if item.text_content() == "moved"
    ));
    assert_eq!(std::fs::read_to_string(path).unwrap().lines().count(), 1);
}

#[tokio::test]
async fn cwd_switch_retry_repairs_bookkeeping_without_duplicate() {
    let dir = tempfile::tempdir().unwrap();
    let info = info();
    let adapter = JsonlStorageAdapter::with_explicit_session_dir(dir.path().to_path_buf());
    adapter
        .init_session(&info, default_model_id())
        .await
        .unwrap();
    let item = ConversationItem::working_directory_switch("moved", 4);

    let summary_path = dir.path().join("summary.json");
    let original_summary = std::fs::read(&summary_path).unwrap();
    std::fs::write(&summary_path, b"invalid summary").unwrap();
    assert!(matches!(
        adapter.append_cwd_switch_commit_aware(&info, &item).await,
        Err(crate::session::storage::AppendCwdSwitchError::Committed {
            acknowledgement: xai_chat_state::StrictAppendAck::Appended,
            ..
        })
    ));
    assert!(matches!(
        adapter
            .append_cwd_switch_commit_aware(
                &info,
                &ConversationItem::working_directory_switch("retry", 4),
            )
            .await,
        Err(crate::session::storage::AppendCwdSwitchError::Committed {
            acknowledgement: xai_chat_state::StrictAppendAck::AlreadyPresent(authoritative),
            ..
        }) if authoritative.text_content() == "moved"
    ));
    std::fs::write(&summary_path, original_summary).unwrap();
    assert_eq!(
        adapter.read_summary_sync(&info).unwrap().num_chat_messages,
        0
    );

    assert!(matches!(
        adapter
            .append_cwd_switch_commit_aware(&info, &item)
            .await
            .unwrap(),
        xai_chat_state::StrictAppendAck::AlreadyPresent(item)
            if item.text_content() == "moved"
    ));
    let summary = adapter.read_summary_sync(&info).unwrap();
    assert_eq!(summary.num_chat_messages, 1);
    assert_eq!(summary.cwd_switch_bookkeeping_generation, 4);

    assert!(matches!(
        adapter
            .append_cwd_switch_commit_aware(&info, &item)
            .await
            .unwrap(),
        xai_chat_state::StrictAppendAck::AlreadyPresent(item)
            if item.text_content() == "moved"
    ));
    let retried = adapter.read_summary_sync(&info).unwrap();
    assert_eq!(retried.num_chat_messages, 1);
    assert_eq!(retried.cwd_switch_bookkeeping_generation, 4);
    assert_eq!(
        std::fs::read_to_string(dir.path().join("chat_history.jsonl"))
            .unwrap()
            .lines()
            .count(),
        1
    );
}

#[tokio::test]
async fn cwd_switch_retained_by_history_replacement_is_not_recounted() {
    let dir = tempfile::tempdir().unwrap();
    let info = info();
    let adapter = JsonlStorageAdapter::with_explicit_session_dir(dir.path().to_path_buf());
    adapter
        .init_session(&info, default_model_id())
        .await
        .unwrap();
    let item = ConversationItem::working_directory_switch("retained", 6);

    adapter
        .replace_chat_history(&info, std::slice::from_ref(&item))
        .await
        .unwrap();
    let replaced = adapter.read_summary_sync(&info).unwrap();
    assert_eq!(replaced.num_chat_messages, 1);
    assert_eq!(replaced.cwd_switch_bookkeeping_generation, 6);

    assert!(matches!(
        adapter
            .append_cwd_switch_commit_aware(
                &info,
                &ConversationItem::working_directory_switch("retry", 6),
            )
            .await
            .unwrap(),
        xai_chat_state::StrictAppendAck::AlreadyPresent(authoritative)
            if authoritative.text_content() == "retained"
    ));
    let summary = adapter.read_summary_sync(&info).unwrap();
    assert_eq!(summary.num_chat_messages, 1);
    assert_eq!(summary.cwd_switch_bookkeeping_generation, 6);
    assert_eq!(
        adapter
            .read_chat_history_sync(adapter.chat_file(&info), CHAT_FORMAT_VERSION)
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn cwd_switch_reappend_after_history_replacement_restores_message_count() {
    let dir = tempfile::tempdir().unwrap();
    let info = info();
    let adapter = JsonlStorageAdapter::with_explicit_session_dir(dir.path().to_path_buf());
    adapter
        .init_session(&info, default_model_id())
        .await
        .unwrap();
    let item = ConversationItem::working_directory_switch("moved", 7);

    assert!(matches!(
        adapter
            .append_cwd_switch_commit_aware(&info, &item)
            .await
            .unwrap(),
        xai_chat_state::StrictAppendAck::Appended
    ));
    adapter.replace_chat_history(&info, &[]).await.unwrap();
    let replaced = adapter.read_summary_sync(&info).unwrap();
    assert_eq!(replaced.cwd_switch_bookkeeping_generation, 7);
    assert_eq!(replaced.num_chat_messages, 0);

    assert!(matches!(
        adapter
            .append_cwd_switch_commit_aware(&info, &item)
            .await
            .unwrap(),
        xai_chat_state::StrictAppendAck::Appended
    ));
    let summary = adapter.read_summary_sync(&info).unwrap();
    assert_eq!(summary.cwd_switch_bookkeeping_generation, 7);
    assert_eq!(summary.num_chat_messages, 1);
    assert_eq!(
        adapter
            .read_chat_history_sync(adapter.chat_file(&info), CHAT_FORMAT_VERSION)
            .unwrap()
            .len(),
        1
    );
}

#[cfg(target_os = "macos")]
#[test]
fn darwin_fullfsync_seam_reports_invalid_descriptor() {
    assert!(super::super::fullfsync_raw(-1).is_err());
}
