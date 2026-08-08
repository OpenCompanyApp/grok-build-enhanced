//! Per-turn dashboard-summary lifecycle on `SessionActor`.

use super::*;

impl SessionActor {
    /// Restart the display-only summary for the successful turn `prompt_id`.
    /// A newer completion or a new prompt invalidates the older generation.
    pub(crate) fn restart_turn_summary(self: &Arc<Self>, prompt_id: String) {
        if !self.turn_summary_enabled || self.startup_hints.is_subagent {
            return;
        }
        if self
            .current_prompt_id
            .lock()
            .expect("current_prompt_id mutex poisoned")
            .is_some()
        {
            return;
        }
        self.abort_turn_summary();
        let generation = self.turn_summary_generation.get().wrapping_add(1);
        self.turn_summary_generation.set(generation);
        let actor = self.clone();
        let task = tokio::task::spawn_local(async move {
            actor.generate_turn_summary(&prompt_id, generation).await;
            if actor.turn_summary_generation.get() == generation {
                *actor.turn_summary_task.borrow_mut() = None;
            }
        });
        *self.turn_summary_task.borrow_mut() = Some(task);
    }

    pub(crate) fn abort_turn_summary(&self) {
        self.turn_summary_generation
            .set(self.turn_summary_generation.get().wrapping_add(1));
        if let Some(task) = self.turn_summary_task.borrow_mut().take() {
            task.abort();
        }
    }

    async fn generate_turn_summary(&self, prompt_id: &str, generation: u64) {
        use crate::session::helpers::turn_summary;

        let conversation = self.chat_state_handle.get_conversation().await;
        let Some(anchor) = turn_summary::last_user_anchor(&conversation) else {
            return;
        };
        let setup = match self.prepare_side_call().await {
            Ok(setup) => setup,
            Err(error) => {
                tracing::warn!(%error, "turn summary: failed to prepare sampling client");
                return;
            }
        };
        let instruction =
            turn_summary::turn_summary_instruction(self.reminder_wrapper_tag(), &anchor);
        let items = crate::session::helpers::session_recap::budget_instruction_items(
            conversation,
            instruction,
            setup.strip_reasoning,
            setup.context_window,
        );
        let request = self
            .side_call_request(
                &setup,
                items,
                format!("turn-summary-{}", uuid::Uuid::new_v4()),
                format!("xai-turn-summary-{}", uuid::Uuid::new_v4()),
            )
            .await;
        let response = match setup.client.conversation_collect(request).await {
            Ok(response) => response,
            Err(error) => {
                tracing::warn!(%error, "turn summary: model call failed");
                return;
            }
        };
        let summary = turn_summary::clean_turn_summary_text(&response.assistant_text());
        if summary.is_empty() || self.turn_summary_generation.get() != generation {
            return;
        }

        let _ = self
            .notifications
            .persistence_tx
            .send(PersistenceMsg::LastTurnSummary(Some((
                summary.clone(),
                prompt_id.to_owned(),
            ))));
        self.send_xai_notification_transient(
            crate::extensions::notification::SessionUpdate::LastTurnSummary {
                summary,
                prompt_id: Some(prompt_id.to_owned()),
            },
        );
    }
}
