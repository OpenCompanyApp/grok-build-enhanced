//! `/usage` -- show provider usage or open the matching management page.

use crate::app::actions::Action;
use crate::slash::command::{AppCtx, ArgItem, CommandExecCtx, CommandResult, SlashCommand};

/// Show coding usage or open the active provider's usage settings.
///
/// `/usage`        -- show current credit usage
/// `/usage show`   -- same as above
/// `/usage manage` -- open provider usage settings in browser
pub struct UsageCommand;

impl SlashCommand for UsageCommand {
    fn name(&self) -> &str {
        "usage"
    }

    /// `/cost` is the minimal-mode name for the same credit-usage summary:
    /// it commits a usage/cost system block rather than opening a
    /// pane, so it's an alias rather than a separate command.
    fn aliases(&self) -> &[&str] {
        &["cost"]
    }

    fn description(&self) -> &str {
        "View usage or open provider usage settings"
    }

    fn usage(&self) -> &str {
        "/usage [show|manage]"
    }

    fn takes_args(&self) -> bool {
        true
    }

    fn arg_placeholder(&self) -> Option<&str> {
        Some("show | manage")
    }

    fn suggest_args(&self, ctx: &AppCtx, _args_query: &str) -> Option<Vec<ArgItem>> {
        let manage_description = if ctx.models.current_model_is_openai_codex() {
            "Open ChatGPT Codex usage settings"
        } else if ctx.models.current_model_is_kimi_code() {
            "Open Kimi Code Console"
        } else {
            "Open billing management page"
        };
        Some(vec![
            ArgItem {
                display: "show".to_string(),
                match_text: "show".to_string(),
                insert_text: "show".to_string(),
                description: "View credit usage".to_string(),
            },
            ArgItem {
                display: "manage".to_string(),
                match_text: "manage".to_string(),
                insert_text: "manage".to_string(),
                description: manage_description.to_string(),
            },
        ])
    }

    fn run(&self, ctx: &mut CommandExecCtx, args: &str) -> CommandResult {
        let arg = args.trim();
        match arg {
            "" | "show" => CommandResult::Action(Action::ShowUsage),
            "manage" if ctx.models.current_model_is_openai_codex() => CommandResult::Action(
                Action::OpenUrl("https://chatgpt.com/codex/settings/usage".to_string()),
            ),
            "manage" if ctx.models.current_model_is_kimi_code() => CommandResult::Action(
                Action::OpenUrl("https://www.kimi.com/code/console".to_string()),
            ),
            "manage" => {
                CommandResult::Action(Action::OpenUrl("https://grok.com/?_s=usage".to_string()))
            }
            _ => CommandResult::Error(format!(
                "Unknown argument: {arg}. Use /usage show or /usage manage"
            )),
        }
    }
}
