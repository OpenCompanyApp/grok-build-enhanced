//! `/feedback` -- send session feedback.

use crate::app::actions::Action;
use crate::slash::command::{CommandExecCtx, CommandResult, SlashCommand};

/// Send session feedback inline or enter feedback mode.
pub struct FeedbackCommand;

impl SlashCommand for FeedbackCommand {
    fn name(&self) -> &str {
        "feedback"
    }

    fn description(&self) -> &str {
        "Send feedback about the current session"
    }

    fn usage(&self) -> &str {
        "/feedback [text]"
    }

    fn takes_args(&self) -> bool {
        true
    }

    fn arg_placeholder(&self) -> Option<&str> {
        Some("[feedback text]")
    }

    fn run(&self, ctx: &mut CommandExecCtx, args: &str) -> CommandResult {
        let trimmed = args.trim();
        // Composer images ride the action, but dispatch attaches them (the
        // command layer never sees the prompt), so they start empty here.
        let result = if ctx.screen_mode.is_minimal() {
            if trimmed.is_empty() {
                CommandResult::Action(Action::OpenFeedbackPane {
                    prefill: None,
                    images: Default::default(),
                })
            } else {
                CommandResult::Action(Action::SendFeedback {
                    text: trimmed.to_string(),
                    images: Default::default(),
                    // Minimal mode never shows the trace-consent card.
                    trace: None,
                })
            }
        } else {
            CommandResult::Action(Action::OpenFeedbackPane {
                prefill: (!trimmed.is_empty()).then(|| trimmed.to_string()),
                images: Default::default(),
            })
        };
        let action = match &result {
            CommandResult::Action(Action::OpenFeedbackPane { prefill, .. }) => {
                if prefill.is_some() {
                    "open_prefill"
                } else {
                    "open_empty"
                }
            }
            CommandResult::Action(Action::SendFeedback { .. }) => "send_immediate",
            _ => "other",
        };
        crate::unified_log::info(
            "feedback.command",
            ctx.session_id.map(|s| s.0.as_ref()),
            Some(serde_json::json!({
                "screen_mode": ctx.screen_mode.meta_label(),
                "arg_chars": trimmed.chars().count(),
                "action": action,
            })),
        );
        result
    }
}
