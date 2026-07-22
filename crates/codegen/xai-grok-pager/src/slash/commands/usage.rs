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

    fn takes_args_now(&self, ctx: &AppCtx) -> bool {
        // Non-consumer: bare `/usage` only — Enter should send, not chain for args.
        ctx.billing_surface_visible
    }

    fn suggest_args(&self, ctx: &AppCtx, _args_query: &str) -> Option<Vec<ArgItem>> {
        if !ctx.billing_surface_visible {
            return None;
        }
        let manage_description = if ctx.models.current_model_is_openai_codex() {
            "Open ChatGPT Codex usage settings"
        } else if ctx.models.current_model_is_kimi_code() {
            "Open Kimi Code Console"
        } else if ctx.models.current_model_is_zai_coding_plan() {
            "Open Z.AI Coding Plan usage"
        } else {
            "Open billing management page"
        };
        Some(vec![
            ArgItem {
                display: "show".into(),
                match_text: "show".into(),
                insert_text: "show".into(),
                description: "View usage".into(),
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
        if !ctx.billing_surface_visible {
            return match arg {
                "" => CommandResult::Action(Action::ShowUsage),
                _ => CommandResult::Error(format!("Unknown argument: {arg}. Use /usage")),
            };
        }
        match arg {
            "" | "show" => CommandResult::Action(Action::ShowUsage),
            "manage" if ctx.models.current_model_is_openai_codex() => CommandResult::Action(
                Action::OpenUrl("https://chatgpt.com/codex/settings/usage".to_string()),
            ),
            "manage" if ctx.models.current_model_is_kimi_code() => CommandResult::Action(
                Action::OpenUrl("https://www.kimi.com/code/console".to_string()),
            ),
            "manage" if ctx.models.current_model_is_zai_coding_plan() => {
                CommandResult::Action(Action::OpenUrl(
                    "https://z.ai/manage-apikey/coding-plan/personal/usage".to_string(),
                ))
            }
            "manage" => {
                CommandResult::Action(Action::OpenUrl("https://grok.com/?_s=usage".to_string()))
            }
            _ => CommandResult::Error(format!(
                "Unknown argument: {arg}. Use /usage show or /usage manage"
            )),
        }
    }
}
