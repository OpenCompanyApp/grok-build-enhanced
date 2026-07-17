//! `/fast` -- control authenticated ChatGPT Codex Fast mode.
//!
//! The pager validates the small command grammar locally, then preserves the
//! shell as the source of truth for provider, entitlement, and session state.

use crate::slash::command::{AppCtx, ArgItem, CommandExecCtx, CommandResult, SlashCommand};

/// Control Fast mode for the active authenticated ChatGPT Codex session.
pub struct FastCommand;

impl SlashCommand for FastCommand {
    fn name(&self) -> &str {
        "fast"
    }

    fn description(&self) -> &str {
        "Control authenticated ChatGPT Codex Fast mode"
    }

    fn session_scoped(&self) -> bool {
        true
    }

    fn usage(&self) -> &str {
        "/fast [on|off|status]"
    }

    fn takes_args(&self) -> bool {
        true
    }

    fn args_required(&self) -> bool {
        false
    }

    fn arg_placeholder(&self) -> Option<&str> {
        Some("on | off | status")
    }

    fn suggest_args(&self, _ctx: &AppCtx, _args_query: &str) -> Option<Vec<ArgItem>> {
        Some(vec![
            ArgItem {
                display: "on".to_string(),
                match_text: "on".to_string(),
                insert_text: "on".to_string(),
                description: "Enable Codex Fast mode".to_string(),
            },
            ArgItem {
                display: "off".to_string(),
                match_text: "off".to_string(),
                insert_text: "off".to_string(),
                description: "Use standard Codex speed".to_string(),
            },
            ArgItem {
                display: "status".to_string(),
                match_text: "status".to_string(),
                insert_text: "status".to_string(),
                description: "Show the current Fast mode setting".to_string(),
            },
        ])
    }

    fn run(&self, _ctx: &mut CommandExecCtx, args: &str) -> CommandResult {
        match args.trim() {
            "" => CommandResult::QueueCommand("/fast".to_string()),
            "on" => CommandResult::QueueCommand("/fast on".to_string()),
            "off" => CommandResult::QueueCommand("/fast off".to_string()),
            "status" => CommandResult::QueueCommand("/fast status".to_string()),
            _ => CommandResult::Error("Usage: /fast [on|off|status]".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acp::model_state::ModelState;
    use crate::app::bundle::BundleState;
    use crate::settings::PagerLocalSnapshot;

    static BUNDLE_STATE: BundleState = BundleState {
        has_cache: false,
        version: String::new(),
        personas: Vec::new(),
        roles: Vec::new(),
        agents: Vec::new(),
        skills: Vec::new(),
        persona_details: Vec::new(),
        role_details: Vec::new(),
    };

    fn run(args: &str) -> CommandResult {
        let models = ModelState::default();
        let mut ctx = CommandExecCtx {
            models: &models,
            session_id: None,
            bundle_state: &BUNDLE_STATE,
            screen_mode: crate::app::ScreenMode::Fullscreen,
            pager_state: PagerLocalSnapshot::default(),
        };
        FastCommand.run(&mut ctx, args)
    }

    #[test]
    fn queues_each_exact_shell_command() {
        for (args, expected) in [
            ("", "/fast"),
            ("on", "/fast on"),
            ("off", "/fast off"),
            ("status", "/fast status"),
        ] {
            match run(args) {
                CommandResult::QueueCommand(command) => assert_eq!(command, expected),
                other => panic!("expected QueueCommand for {args}, got {other:?}"),
            }
        }
    }

    #[test]
    fn trims_valid_argument_before_queueing() {
        assert!(matches!(
            run("  status  "),
            CommandResult::QueueCommand(command) if command == "/fast status"
        ));
    }

    #[test]
    fn rejects_invalid_arguments_locally() {
        for args in ["turbo", "on now", "ON"] {
            assert!(matches!(
                run(args),
                CommandResult::Error(message) if message == "Usage: /fast [on|off|status]"
            ));
        }
    }

    #[test]
    fn metadata_and_suggestions_describe_codex_fast_mode() {
        let command = FastCommand;
        assert_eq!(
            command.description(),
            "Control authenticated ChatGPT Codex Fast mode"
        );
        assert_eq!(command.usage(), "/fast [on|off|status]");
        assert!(command.session_scoped());
        assert!(command.takes_args());
        assert!(!command.args_required());

        let models = ModelState::default();
        let app_ctx = AppCtx {
            models: &models,
            cwd: std::path::Path::new("."),
            has_session_announcements: false,
            screen_mode: crate::app::ScreenMode::Fullscreen,
        };
        let items = command
            .suggest_args(&app_ctx, "")
            .expect("Fast mode should advertise argument completions");
        assert_eq!(
            items
                .iter()
                .map(|item| item.insert_text.as_str())
                .collect::<Vec<_>>(),
            ["on", "off", "status"]
        );
    }
}
