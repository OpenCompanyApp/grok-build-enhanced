//! Genuine turn-completion `Stop`/`SubagentStop` resampling gate.

use super::*;
use xai_grok_hooks::event::{
    self, BackgroundTaskType, StopBackgroundTask, StopSessionCron, sanitize_stop_text,
};
use xai_grok_hooks::{dispatcher, result};

pub const MAX_STOP_HOOK_CONTINUATIONS_PER_TURN: u32 = 8;

const SESSION_END_STOP_BUDGET: std::time::Duration = std::time::Duration::from_secs(5);
const WORK_SNAPSHOT_BUDGET: std::time::Duration = std::time::Duration::from_secs(2);
const STOP_FEEDBACK_TEXT_MAX: usize = 10_000;

fn stop_entry_from_task(task: &xai_grok_tools::types::TaskSnapshot) -> StopBackgroundTask {
    let command_text = sanitize_stop_text(task.display_command.as_deref().unwrap_or(&task.command));
    let (kind, command, description) = match task.kind {
        xai_grok_tools::computer::types::TaskKind::Bash => {
            (BackgroundTaskType::Shell, Some(command_text), None)
        }
        xai_grok_tools::computer::types::TaskKind::Monitor => {
            (BackgroundTaskType::Monitor, None, Some(command_text))
        }
    };
    StopBackgroundTask {
        id: sanitize_stop_text(&task.task_id),
        r#type: kind,
        status: "running".to_string(),
        description,
        command,
        agent_type: None,
    }
}

fn stop_entries_from_background_tasks(
    tasks: &[xai_grok_tools::types::TaskSnapshot],
    session_id: &str,
) -> Vec<StopBackgroundTask> {
    let mut projected: Vec<_> = tasks
        .iter()
        .filter(|task| task.is_outstanding())
        // The terminal backend is shared. Unknown ownership fails closed at
        // this data boundary (omit the descriptor) rather than leaking another
        // session's command into a stop-hook request.
        .filter(|task| task.owner_session_id.as_deref() == Some(session_id))
        .map(stop_entry_from_task)
        .collect();
    projected.sort_by(|left, right| {
        left.r#type
            .cmp(&right.r#type)
            .then_with(|| left.id.cmp(&right.id))
    });
    projected
}

fn stop_entry_from_subagent(
    summary: &xai_grok_tools::implementations::grok_build::task::types::ActiveSubagentSummary,
) -> StopBackgroundTask {
    StopBackgroundTask {
        id: sanitize_stop_text(&summary.subagent_id),
        r#type: BackgroundTaskType::Subagent,
        status: "running".to_string(),
        description: Some(sanitize_stop_text(&summary.description)),
        command: None,
        agent_type: Some(sanitize_stop_text(&summary.subagent_type)),
    }
}

fn stop_cron_from_scheduled(
    task: &xai_grok_tools::implementations::grok_build::scheduler::types::ScheduledTask,
) -> StopSessionCron {
    StopSessionCron {
        id: sanitize_stop_text(&task.id),
        schedule: sanitize_stop_text(
            &xai_grok_tools::implementations::grok_build::scheduler::interval::interval_to_human(
                task.interval_secs,
            ),
        ),
        recurring: task.recurring,
        prompt: sanitize_stop_text(&task.prompt),
    }
}

fn format_stop_feedback(blocks: &[dispatcher::StopBlock], additional_context: &[String]) -> String {
    use std::fmt::Write as _;

    let mut feedback = String::new();
    if !blocks.is_empty() {
        feedback.push_str("Stop hook feedback:\n");
        for block in blocks {
            let _ = writeln!(feedback, "- {}", block.reason);
        }
    }
    for context in additional_context {
        if !feedback.is_empty() {
            feedback.push('\n');
        }
        feedback.push_str(context);
    }
    event::clip_text(&feedback, STOP_FEEDBACK_TEXT_MAX)
}

/// A session-end Stop is observe-only: any deliberate block was parsed
/// correctly, but there is no live turn to resample, so report it as success.
pub(super) fn demote_ignored_blocks(
    results: Vec<result::HookRunResult>,
) -> Vec<result::HookRunResult> {
    use xai_grok_hooks::result::HookRunResult;

    results
        .into_iter()
        .map(|result| match result {
            HookRunResult::Blocked {
                hook_name,
                elapsed,
                http_info,
                ..
            } => HookRunResult::Success {
                hook_name,
                elapsed,
                http_info,
            },
            other => other,
        })
        .collect()
}

impl SessionActor {
    /// Dispatch an observe-only Stop while the top-level session is shutting
    /// down. A hard budget prevents hooks from delaying actor teardown.
    pub(crate) async fn dispatch_session_end_stop(&self, reason: &str) {
        if self.startup_hints.is_subagent || !self.hook_event_active(event::HookEventName::Stop) {
            return;
        }
        let envelope = self.fire_hook(
            event::HookEventName::Stop,
            None,
            event::HookPayload::Stop {
                reason: reason.to_string(),
                stop_hook_active: false,
                last_assistant_message: None,
                background_tasks: None,
                session_crons: None,
            },
        );
        let Some(registry) = self.hook_registry.borrow().clone() else {
            return;
        };
        let ctx = self.hook_run_ctx();
        let dispatch =
            dispatcher::dispatch_stop(&registry, event::HookEventName::Stop, &envelope, &ctx);
        let Ok(mut dispatch) = tokio::time::timeout(SESSION_END_STOP_BUDGET, dispatch).await else {
            tracing::warn!("session-end stop hooks exceeded the shutdown budget; skipping");
            return;
        };
        dispatch.results = demote_ignored_blocks(dispatch.results);
        self.send_hook_execution("stop", None, None, &dispatch.results)
            .await;
        self.emit_hook_executed_telemetry("stop", None, &dispatch.results)
            .await;
    }

    async fn list_active_subagents(
        &self,
    ) -> Vec<xai_grok_tools::implementations::grok_build::task::types::ActiveSubagentSummary> {
        use xai_grok_tools::implementations::grok_build::task::types::{
            SubagentEvent, SubagentListActiveRequest,
        };

        let Some(ref event_tx) = self.tool_context.subagent_event_tx else {
            return Vec::new();
        };
        let (respond_to, response) = tokio::sync::oneshot::channel();
        if event_tx
            .send(SubagentEvent::ListActive(SubagentListActiveRequest {
                parent_session_id: self.session_id_string(),
                respond_to,
            }))
            .is_err()
        {
            return Vec::new();
        }
        response.await.unwrap_or_default()
    }

    async fn collect_stop_gate_work_snapshot(
        &self,
    ) -> (Vec<StopBackgroundTask>, Vec<StopSessionCron>) {
        let bridge = self.tool_bridge_handle();
        let session_id = self.session_id_string();
        let (background, subagents, scheduled) = tokio::join!(
            bridge.list_background_tasks(),
            self.list_active_subagents(),
            bridge.list_scheduled_tasks(),
        );

        let mut tasks = stop_entries_from_background_tasks(&background, &session_id);
        tasks.extend(subagents.iter().map(stop_entry_from_subagent));
        tasks.sort_by(|left, right| {
            left.r#type
                .cmp(&right.r#type)
                .then_with(|| left.id.cmp(&right.id))
        });

        let now = chrono::Utc::now();
        let mut crons: Vec<_> = scheduled
            .iter()
            .filter(|task| !task.is_expired(now))
            .map(stop_cron_from_scheduled)
            .collect();
        crons.sort_by(|left, right| left.id.cmp(&right.id));
        (tasks, crons)
    }

    /// Snapshot active work without exposing internal provider/session objects.
    /// Query stalls fail open to an empty projection rather than blocking turn
    /// completion indefinitely.
    async fn stop_gate_work_snapshot(&self) -> (Vec<StopBackgroundTask>, Vec<StopSessionCron>) {
        match tokio::time::timeout(WORK_SNAPSHOT_BUDGET, self.collect_stop_gate_work_snapshot())
            .await
        {
            Ok(snapshot) => snapshot,
            Err(_) => {
                tracing::warn!("stop-hook active-work snapshot timed out; using an empty snapshot");
                (Vec::new(), Vec::new())
            }
        }
    }

    async fn announce_force_stop(&self, prevent: &dispatcher::StopBlock) {
        self.send_hook_annotation(&format!(
            "\u{26a0} Hook `{}` stopped the agent: {}",
            prevent.hook_name, prevent.reason
        ))
        .await;
    }

    async fn build_stop_payload(&self, stop_hook_active: bool) -> event::HookPayload {
        let last_assistant_message = self
            .chat_state_handle
            .get_last_assistant_text_in_turn()
            .await;
        if self.startup_hints.is_subagent {
            event::HookPayload::SubagentStop {
                phase: event::SubagentStopPhase::Gate,
                subagent_id: self.session_id_string(),
                subagent_type: self.subagent_type_label().unwrap_or_default(),
                stop_hook_active: Some(stop_hook_active),
                last_assistant_message,
            }
        } else {
            let (background_tasks, session_crons) = self.stop_gate_work_snapshot().await;
            event::HookPayload::Stop {
                reason: "end_turn".to_string(),
                stop_hook_active,
                last_assistant_message,
                background_tasks: Some(background_tasks),
                session_crons: Some(session_crons),
            }
        }
    }

    async fn emit_stop_results(
        &self,
        event: event::HookEventName,
        prompt_id: &str,
        results: &[result::HookRunResult],
    ) {
        let event_name = event.to_string();
        self.send_hook_execution(&event_name, None, Some(prompt_id), results)
            .await;
        self.emit_hook_executed_telemetry(&event_name, None, results)
            .await;
    }

    /// Run all file/HTTP and ACP stop gates for one genuine completion.
    /// Failures contribute no signal. Explicit force-stop overrides every
    /// block, and the caller's continuation counter enforces the eight-round
    /// cap before any hook is dispatched.
    pub(super) async fn run_stop_gate(
        &self,
        prompt_id: &str,
        continuations_this_turn: u32,
    ) -> StopGateDecision {
        let event = if self.startup_hints.is_subagent {
            event::HookEventName::SubagentStop
        } else {
            event::HookEventName::Stop
        };
        let has_file_hooks = self
            .hook_registry
            .borrow()
            .as_ref()
            .is_some_and(|registry| registry.has_enabled_hooks_for_canonical(event));
        let has_client_hooks = self.client_hooks.borrow().contains_key(&event);
        if !has_file_hooks && !has_client_hooks {
            return StopGateDecision::AllowStop;
        }
        if continuations_this_turn >= MAX_STOP_HOOK_CONTINUATIONS_PER_TURN {
            tracing::warn!(
                continuations_this_turn,
                "stop-hook continuation limit reached; ending the turn"
            );
            self.send_hook_annotation(&format!(
                "\u{26a0} Stop hooks kept the agent working {MAX_STOP_HOOK_CONTINUATIONS_PER_TURN} times this turn: limit reached, ending the turn"
            ))
            .await;
            return StopGateDecision::AllowStop;
        }

        let payload = self.build_stop_payload(continuations_this_turn > 0).await;
        let envelope = self.make_hook_envelope(event, Some(prompt_id.to_string()), payload);

        let mut dispatch = dispatcher::StopDispatchResult::default();
        let registry = self.hook_registry.borrow().clone();
        if let Some(registry) = registry {
            let ctx = self.hook_run_ctx();
            dispatch = dispatcher::dispatch_stop(&registry, event, &envelope, &ctx).await;
        }

        if let Some(prevent) = dispatch.prevent_continuation.take() {
            self.emit_stop_results(event, prompt_id, &dispatch.results)
                .await;
            // A force-stop skips the client gate because its signals cannot
            // change the decision, but observers still receive the exact same
            // bounded envelope.
            self.notify_client_hooks(&envelope);
            self.announce_force_stop(&prevent).await;
            return StopGateDecision::AllowStop;
        }

        let client = self.run_stop_client_hooks(&envelope).await;
        let mut all_results = std::mem::take(&mut dispatch.results);
        all_results.extend(client.results);
        if !all_results.is_empty() {
            self.emit_stop_results(event, prompt_id, &all_results).await;
        }

        dispatch.blocks.extend(client.blocks);
        dispatch
            .additional_context
            .extend(client.additional_context);
        if let Some(prevent) = client.prevent_continuation {
            self.announce_force_stop(&prevent).await;
            return StopGateDecision::AllowStop;
        }
        if !dispatch.wants_continuation() {
            return StopGateDecision::AllowStop;
        }

        self.announce_keep_working(&dispatch.blocks, &dispatch.additional_context)
            .await;
        StopGateDecision::KeepWorking {
            feedback: format_stop_feedback(&dispatch.blocks, &dispatch.additional_context),
        }
    }

    async fn announce_keep_working(
        &self,
        blocks: &[dispatcher::StopBlock],
        additional_context: &[String],
    ) {
        for block in blocks {
            self.send_hook_annotation(&format!(
                "\u{21a9} Stop blocked by hook `{}`, continuing: {}",
                block.hook_name, block.reason
            ))
            .await;
            xai_grok_telemetry::session_ctx::log_event(xai_grok_telemetry::events::HookBlocked {
                hook_name: block.hook_name.clone(),
            });
        }
        if blocks.is_empty() {
            for context in additional_context {
                self.send_hook_annotation(&format!(
                    "\u{21a9} Stop hook feedback, continuing: {context}"
                ))
                .await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task_snapshot(
        id: &str,
        kind: xai_grok_tools::computer::types::TaskKind,
        owner: Option<&str>,
    ) -> xai_grok_tools::types::TaskSnapshot {
        xai_grok_tools::types::TaskSnapshot {
            task_id: id.into(),
            command: "sandbox-exec tail -f /var/log/syslog".into(),
            display_command: Some("tail -f /var/log/syslog".into()),
            cwd: "/tmp".into(),
            start_time: std::time::SystemTime::UNIX_EPOCH,
            end_time: None,
            output: String::new(),
            output_file: std::path::PathBuf::from("/tmp/out"),
            truncated: false,
            exit_code: None,
            signal: None,
            completed: false,
            kind,
            block_waited: false,
            explicitly_killed: false,
            owner_session_id: owner.map(str::to_string),
        }
    }

    #[test]
    fn background_snapshot_is_session_isolated_and_deterministic() {
        let mut completed = task_snapshot(
            "completed",
            xai_grok_tools::computer::types::TaskKind::Bash,
            Some("session-a"),
        );
        completed.completed = true;
        let entries = stop_entries_from_background_tasks(
            &[
                task_snapshot(
                    "z-monitor",
                    xai_grok_tools::computer::types::TaskKind::Monitor,
                    Some("session-a"),
                ),
                task_snapshot(
                    "other-session",
                    xai_grok_tools::computer::types::TaskKind::Bash,
                    Some("session-b"),
                ),
                task_snapshot(
                    "unknown-owner",
                    xai_grok_tools::computer::types::TaskKind::Bash,
                    None,
                ),
                task_snapshot(
                    "a-shell",
                    xai_grok_tools::computer::types::TaskKind::Bash,
                    Some("session-a"),
                ),
                completed,
            ],
            "session-a",
        );
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.id.as_str())
                .collect::<Vec<_>>(),
            ["a-shell", "z-monitor"]
        );
        assert_eq!(entries[0].r#type, BackgroundTaskType::Shell);
        assert_eq!(entries[1].r#type, BackgroundTaskType::Monitor);
    }

    #[test]
    fn task_projection_prefers_display_command_and_redacts_secrets() {
        let mut task = task_snapshot(
            "task-1",
            xai_grok_tools::computer::types::TaskKind::Bash,
            Some("session-a"),
        );
        task.display_command =
            Some("curl -H 'Authorization: Bearer secret-token' https://x".into());
        let entry = stop_entry_from_task(&task);
        let command = entry.command.expect("shell command projected");
        assert!(!command.contains("secret-token"));
        assert!(command.contains("[redacted]"));
        assert!(!command.contains("sandbox-exec"));
    }

    #[test]
    fn subagent_projection_is_bounded_and_has_no_provider_state() {
        let summary =
            xai_grok_tools::implementations::grok_build::task::types::ActiveSubagentSummary {
                subagent_id: "sub-1".into(),
                subagent_type: "explore".into(),
                description: format!("api_key=secret {}", "🦀".repeat(2_000)),
                elapsed_ms: 5,
            };
        let entry = stop_entry_from_subagent(&summary);
        let description = entry.description.as_deref().unwrap();
        assert!(description.chars().count() <= event::MAX_STOP_ENTRY_TEXT_CHARS);
        assert!(!description.contains("secret"));
        let value = serde_json::to_value(entry).unwrap();
        for forbidden in [
            "credentials",
            "providerState",
            "codexTurnState",
            "accountId",
            "apiKey",
        ] {
            assert!(
                value.get(forbidden).is_none(),
                "leaked forbidden field {forbidden}"
            );
        }
    }

    #[test]
    fn format_stop_feedback_preserves_aggregation_order() {
        let block = |hook_name: &str, reason: &str| dispatcher::StopBlock {
            hook_name: hook_name.into(),
            reason: reason.into(),
        };
        assert_eq!(
            format_stop_feedback(
                &[block("first", "one"), block("second", "two")],
                &["context-a".into(), "context-b".into()],
            ),
            "Stop hook feedback:\n- one\n- two\n\ncontext-a\ncontext-b"
        );
    }

    #[test]
    fn scheduled_task_projection_is_bounded() {
        let task =
            xai_grok_tools::implementations::grok_build::scheduler::types::ScheduledTask::new(
                300,
                "🦀".repeat(2_000),
                true,
                false,
            );
        let cron = stop_cron_from_scheduled(&task);
        assert_eq!(cron.schedule, "every 5 minutes");
        assert!(cron.recurring);
        assert!(cron.prompt.chars().count() <= event::MAX_STOP_ENTRY_TEXT_CHARS);
    }

    #[test]
    fn demote_ignored_blocks_only_changes_deliberate_blocks() {
        use xai_grok_hooks::result::HookRunResult;

        let results = demote_ignored_blocks(vec![
            HookRunResult::Blocked {
                hook_name: "gate".into(),
                detail: "blocked stop: run tests".into(),
                elapsed: std::time::Duration::from_millis(5),
                http_info: None,
            },
            HookRunResult::Failed {
                hook_name: "broken".into(),
                error: "exit code 1".into(),
                elapsed: std::time::Duration::from_millis(3),
                http_info: None,
            },
        ]);
        assert!(
            matches!(&results[0], HookRunResult::Success { hook_name, .. } if hook_name == "gate")
        );
        assert!(matches!(&results[1], HookRunResult::Failed { .. }));
    }
}
