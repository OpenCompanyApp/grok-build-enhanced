use super::*;

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
    let (tx, rx) = mpsc::unbounded_channel();
    let summary_tx = tx.clone();
    let sampling_client = OaiCompatClient::new(xai_grok_sampler::SamplerConfig::default()).unwrap();
    let task = tokio::spawn(
        SessionPersistence {
            info,
            storage,
            pending_notifications: std::collections::VecDeque::new(),
            rx,
            remote_sync: None,
            relay_sync: None,
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
