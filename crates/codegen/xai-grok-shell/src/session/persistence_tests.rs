use super::*;
use crate::session::storage::jsonl::AppendDurability;

struct ActorGuard {
    handle: PersistenceHandle,
    task: tokio::task::JoinHandle<()>,
}

impl ActorGuard {
    async fn stop(self) {
        self.task.abort();
        let _ = self.task.await;
    }
}

fn test_actor(info: Info, storage: Arc<dyn StorageAdapter>) -> ActorGuard {
    test_actor_with_syncs(info, storage, None, None)
}

fn test_actor_with_remote_sync(
    info: Info,
    storage: Arc<dyn StorageAdapter>,
    remote_sync: Option<RemoteSync>,
) -> ActorGuard {
    test_actor_with_syncs(info, storage, remote_sync, None)
}

fn test_actor_with_syncs(
    info: Info,
    storage: Arc<dyn StorageAdapter>,
    remote_sync: Option<RemoteSync>,
    relay_sync: Option<crate::relay::RelaySync>,
) -> ActorGuard {
    let (tx, rx) = mpsc::unbounded_channel();
    let summary_tx = tx.clone();
    let sampling_client = OaiCompatClient::new(xai_grok_sampler::SamplerConfig::default()).unwrap();
    let task = tokio::spawn(
        SessionPersistence {
            info,
            storage,
            pending_notifications: std::collections::VecDeque::new(),
            rx,
            remote_sync,
            relay_sync,
            summary: crate::session::summary::SummaryGenerator::new(
                crate::session::summary::SummaryConfig {
                    sampling_client,
                    model: String::new(),
                    session_id: "durable-update-test".to_string(),
                    persistence_tx: summary_tx,
                },
            ),
            registry_title_sync: None,
            gateway: None,
        }
        .run(),
    );
    ActorGuard {
        handle: PersistenceHandle {
            tx,
            noop: false,
            unobserved_codex_summary: false,
        },
        task,
    }
}

fn notification(info: &Info, text: &str) -> acp::SessionNotification {
    acp::SessionNotification::new(
        info.id.clone(),
        acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk::new(acp::ContentBlock::Text(
            acp::TextContent::new(text),
        ))),
    )
}

fn neutral_update(info: &Info, text: &str) -> SessionUpdate {
    SessionUpdate::Acp(Box::new(notification(info, text)))
}

fn thought_update(info: &Info, text: &str) -> SessionUpdate {
    SessionUpdate::Acp(Box::new(acp::SessionNotification::new(
        info.id.clone(),
        acp::SessionUpdate::AgentThoughtChunk(acp::ContentChunk::new(acp::ContentBlock::Text(
            acp::TextContent::new(text),
        ))),
    )))
}

fn break_summary_writes(dir: &std::path::Path) {
    let summary = dir.join("summary.json");
    std::fs::remove_file(&summary).unwrap();
    std::fs::create_dir(summary).unwrap();
}

async fn recv_observed(
    observed: &mut tokio::sync::mpsc::UnboundedReceiver<acp::SessionNotification>,
) -> acp::SessionNotification {
    tokio::time::timeout(std::time::Duration::from_secs(1), observed.recv())
        .await
        .expect("sync observer timed out")
        .expect("sync observer closed")
}

async fn assert_no_observed(
    observed: &mut tokio::sync::mpsc::UnboundedReceiver<acp::SessionNotification>,
) {
    assert!(
        tokio::time::timeout(std::time::Duration::from_millis(50), observed.recv())
            .await
            .is_err(),
        "sync observer received an unexpected duplicate notification"
    );
}

fn notification_text(notification: &acp::SessionNotification) -> &str {
    let acp::SessionUpdate::AgentMessageChunk(chunk) = &notification.update else {
        panic!("expected an agent message chunk");
    };
    let acp::ContentBlock::Text(text) = &chunk.content else {
        panic!("expected text content");
    };
    &text.text
}

#[tokio::test]
async fn noop_handle_rejects_durable_append_without_dispatch() {
    let info = Info {
        id: acp::SessionId::new("noop-durable-update"),
        cwd: "/test".into(),
    };

    assert!(matches!(
        PersistenceHandle::noop()
            .append_update_durably(neutral_update(&info, "durable"))
            .await,
        Err(DurableAppendError::NotCommitted(error))
            if error.kind() == io::ErrorKind::Unsupported
    ));
}

#[tokio::test]
async fn closed_actor_before_dispatch_reports_not_committed() {
    let info = Info {
        id: acp::SessionId::new("closed-durable-update"),
        cwd: "/test".into(),
    };
    let (tx, rx) = mpsc::unbounded_channel();
    drop(rx);
    let handle = PersistenceHandle {
        tx,
        noop: false,
        unobserved_codex_summary: false,
    };

    assert!(matches!(
        handle
            .append_update_durably(neutral_update(&info, "durable"))
            .await,
        Err(DurableAppendError::NotCommitted(error))
            if error.kind() == io::ErrorKind::BrokenPipe
    ));
}

#[tokio::test]
async fn actor_drop_after_dispatch_reports_acknowledgement_lost() {
    let info = Info {
        id: acp::SessionId::new("lost-durable-ack"),
        cwd: "/test".into(),
    };
    let (tx, mut rx) = mpsc::unbounded_channel();
    let handle = PersistenceHandle {
        tx,
        noop: false,
        unobserved_codex_summary: false,
    };
    let actor = tokio::spawn(async move {
        let Some(PersistenceMsg::AppendUpdateDurablyAndAck { respond_to, .. }) = rx.recv().await
        else {
            panic!("expected durable append message");
        };
        drop(respond_to);
    });

    let result = handle
        .append_update_durably(neutral_update(&info, "durable"))
        .await;
    actor.await.unwrap();

    assert!(matches!(
        result,
        Err(DurableAppendError::AcknowledgementLost(error))
            if error.kind() == io::ErrorKind::BrokenPipe
    ));
}

#[tokio::test]
async fn actor_retries_precommit_failure_without_dropping_newer_update() {
    let dir = tempfile::tempdir().unwrap();
    let info = Info {
        id: acp::SessionId::new("retry-precommit-update"),
        cwd: dir.path().to_string_lossy().into_owned(),
    };
    let storage = Arc::new(JsonlStorageAdapter::with_explicit_session_dir(
        dir.path().to_path_buf(),
    ));
    storage
        .init_session(&info, default_model_id())
        .await
        .unwrap();

    let updates_path = dir.path().join("updates.jsonl");
    std::fs::create_dir(&updates_path).unwrap();
    let actor = test_actor(info.clone(), storage.clone());
    actor
        .handle
        .tx
        .send(PersistenceMsg::Update(neutral_update(&info, "before")))
        .unwrap();
    actor
        .handle
        .tx
        .send(PersistenceMsg::Update(thought_update(&info, "after")))
        .unwrap();
    let (first_ack, first_flush) = tokio::sync::oneshot::channel();
    actor
        .handle
        .tx
        .send(PersistenceMsg::FlushAndAck {
            respond_to: first_ack,
        })
        .unwrap();
    first_flush.await.unwrap();

    std::fs::remove_dir(&updates_path).unwrap();
    let (second_ack, second_flush) = tokio::sync::oneshot::channel();
    actor
        .handle
        .tx
        .send(PersistenceMsg::FlushAndAck {
            respond_to: second_ack,
        })
        .unwrap();
    second_flush.await.unwrap();

    let updates = storage.load_session(&info).await.unwrap().updates;
    let texts = updates
        .iter()
        .filter_map(|update| {
            let SessionUpdate::Acp(notification) = update else {
                return None;
            };
            let (kind, chunk) = match &notification.update {
                acp::SessionUpdate::AgentMessageChunk(chunk) => ("message", chunk),
                acp::SessionUpdate::AgentThoughtChunk(chunk) => ("thought", chunk),
                _ => return None,
            };
            let acp::ContentBlock::Text(text) = &chunk.content else {
                return None;
            };
            Some(format!("{kind}:{}", text.text))
        })
        .collect::<Vec<_>>();
    assert_eq!(texts, ["message:before", "thought:after"]);
    actor.stop().await;
}

#[tokio::test]
async fn failed_pending_drain_retains_fifo_and_skips_durable_target() {
    let info = Info {
        id: acp::SessionId::new("durable-drain-failure"),
        cwd: "/test".into(),
    };
    let attempts = Arc::new(std::sync::Mutex::new(Vec::new()));
    let observed_attempts = attempts.clone();
    let storage =
        JsonlStorageAdapter::with_update_append_probe("/unused".into(), move |durability| {
            observed_attempts.lock().unwrap().push(durability);
            Err(io::Error::other("pending append failed"))
        });
    let (remote_sync, mut remote_observed) = RemoteSync::test_observer();
    let (relay_sync, mut relay_observed) = crate::relay::RelaySync::test_observer();
    let actor = test_actor_with_syncs(
        info.clone(),
        Arc::new(storage),
        Some(remote_sync),
        Some(relay_sync),
    );
    actor
        .handle
        .tx
        .send(PersistenceMsg::Update(neutral_update(&info, "pending")))
        .unwrap();

    for _ in 0..2 {
        assert!(matches!(
            actor
                .handle
                .append_update_durably(neutral_update(&info, "durable"))
                .await,
            Err(DurableAppendError::NotCommitted(error))
                if error.to_string() == "pending append failed"
        ));
    }

    assert!(matches!(
        attempts.lock().unwrap().as_slice(),
        [AppendDurability::Buffered, AppendDurability::Buffered]
    ));
    assert_no_observed(&mut remote_observed).await;
    assert_no_observed(&mut relay_observed).await;
    actor.stop().await;
}

#[tokio::test]
async fn committed_pending_bookkeeping_failure_syncs_once_and_skips_durable_target() {
    let dir = tempfile::tempdir().unwrap();
    let info = Info {
        id: acp::SessionId::new("committed-pending-update"),
        cwd: dir.path().to_string_lossy().into_owned(),
    };
    let attempts = Arc::new(std::sync::Mutex::new(Vec::new()));
    let observed_attempts = attempts.clone();
    let storage = Arc::new(JsonlStorageAdapter::with_update_append_probe(
        dir.path().to_path_buf(),
        move |durability| {
            observed_attempts.lock().unwrap().push(durability);
            Ok(())
        },
    ));
    storage
        .init_session(&info, default_model_id())
        .await
        .unwrap();
    let (remote_sync, mut remote_observed) = RemoteSync::test_observer();
    let (relay_sync, mut relay_observed) = crate::relay::RelaySync::test_observer();
    let actor = test_actor_with_syncs(info.clone(), storage, Some(remote_sync), Some(relay_sync));
    actor
        .handle
        .tx
        .send(PersistenceMsg::Update(neutral_update(&info, "pending")))
        .unwrap();
    break_summary_writes(dir.path());

    let error = actor
        .handle
        .append_update_durably(neutral_update(&info, "durable"))
        .await
        .unwrap_err();
    assert!(matches!(error, DurableAppendError::NotCommitted(_)));
    assert!(
        error
            .to_string()
            .contains("before the durable target was attempted")
    );
    assert!(matches!(
        attempts.lock().unwrap().as_slice(),
        [AppendDurability::Buffered]
    ));

    let remote = recv_observed(&mut remote_observed).await;
    let relay = recv_observed(&mut relay_observed).await;
    assert_eq!(notification_text(&remote), "pending");
    assert_eq!(notification_text(&relay), "pending");
    assert_no_observed(&mut remote_observed).await;
    assert_no_observed(&mut relay_observed).await;
    actor.stop().await;
}

#[tokio::test]
async fn committed_durable_target_bookkeeping_failure_syncs_once() {
    let dir = tempfile::tempdir().unwrap();
    let info = Info {
        id: acp::SessionId::new("committed-durable-update"),
        cwd: dir.path().to_string_lossy().into_owned(),
    };
    let attempts = Arc::new(std::sync::Mutex::new(Vec::new()));
    let observed_attempts = attempts.clone();
    let storage = Arc::new(JsonlStorageAdapter::with_update_append_probe(
        dir.path().to_path_buf(),
        move |durability| {
            observed_attempts.lock().unwrap().push(durability);
            Ok(())
        },
    ));
    storage
        .init_session(&info, default_model_id())
        .await
        .unwrap();
    break_summary_writes(dir.path());
    let (remote_sync, mut remote_observed) = RemoteSync::test_observer();
    let (relay_sync, mut relay_observed) = crate::relay::RelaySync::test_observer();
    let actor = test_actor_with_syncs(info.clone(), storage, Some(remote_sync), Some(relay_sync));

    assert!(matches!(
        actor
            .handle
            .append_update_durably(neutral_update(&info, "durable"))
            .await,
        Err(DurableAppendError::Committed(_))
    ));
    assert!(matches!(
        attempts.lock().unwrap().as_slice(),
        [AppendDurability::Durable]
    ));

    let remote = recv_observed(&mut remote_observed).await;
    let relay = recv_observed(&mut relay_observed).await;
    assert_eq!(notification_text(&remote), "durable");
    assert_eq!(notification_text(&relay), "durable");
    assert_no_observed(&mut remote_observed).await;
    assert_no_observed(&mut relay_observed).await;
    actor.stop().await;
}

#[tokio::test]
async fn durable_ack_drains_pending_update_in_fifo_order() {
    let dir = tempfile::tempdir().unwrap();
    let info = Info {
        id: acp::SessionId::new("durable-update"),
        cwd: dir.path().to_string_lossy().into_owned(),
    };
    let storage = Arc::new(JsonlStorageAdapter::with_explicit_session_dir(
        dir.path().to_path_buf(),
    ));
    storage
        .init_session(&info, default_model_id())
        .await
        .unwrap();
    let actor = test_actor(info.clone(), storage.clone());
    actor
        .handle
        .tx
        .send(PersistenceMsg::Update(neutral_update(&info, "before")))
        .unwrap();
    let (respond_to, response) = tokio::sync::oneshot::channel();
    actor
        .handle
        .tx
        .send(PersistenceMsg::AppendUpdateDurablyAndAck {
            update: neutral_update(&info, "durable"),
            respond_to,
        })
        .unwrap();
    response.await.unwrap().unwrap();
    let summary = storage.load_summary(&info).await.unwrap();
    assert_eq!(summary.num_messages, 2);

    let updates = storage.load_session(&info).await.unwrap().updates;
    let texts = updates
        .iter()
        .filter_map(|update| {
            let SessionUpdate::Acp(notification) = update else {
                return None;
            };
            let acp::SessionUpdate::AgentMessageChunk(chunk) = &notification.update else {
                return None;
            };
            let acp::ContentBlock::Text(text) = &chunk.content else {
                return None;
            };
            Some(text.text.clone())
        })
        .collect::<Vec<_>>();
    assert_eq!(texts, ["before", "durable"]);
    actor.stop().await;
}
