pub mod command;
pub mod http;

use std::time::Duration;

use serde::Deserialize;

use crate::config::HookSpec;
use crate::event::HookEventEnvelope;
use crate::result::{HookDecision, HttpInfo, StopHookOutcome};

pub use crate::event::GateKind;

/// Context passed to any hook runner for environment setup.
pub struct RunContext<'a> {
    pub session_id: &'a str,
    pub workspace_root: &'a str,
    pub process_scope: Option<xai_grok_tools::util::ProcessScope>,
}

/// Result of running a single hook (any handler type).
#[derive(Debug)]
pub enum HookRunnerResult {
    Allow {
        updated_input: Option<serde_json::Value>,
    },
    Deny {
        reason: String,
        hook_name: String,
    },
    Stop(StopHookOutcome),
    /// Hook ran successfully (observe-only, or no gate signal).
    Success,
    /// Hook failed — caller should fail open.
    Failed(String),
}

/// JSON emitted by a `PreToolUse` gate hook. Every field is optional: `decision`
/// carries the allow/deny verdict, `reason` its message, and
/// `hookSpecificOutput.updatedInput` an optional rewrite of the tool input.
#[derive(Debug, Deserialize)]
pub(crate) struct GateHookJson {
    #[serde(default)]
    pub decision: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default, rename = "hookSpecificOutput")]
    pub hook_specific_output: Option<GateHookSpecificOutputJson>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GateHookSpecificOutputJson {
    #[serde(default, rename = "updatedInput")]
    pub updated_input: Option<serde_json::Value>,
}

impl GateHookJson {
    /// True only when the payload carries a `decision` or `hookSpecificOutput`;
    /// a stray JSON log line is not a gate document and must not read as allow.
    fn is_gate_document(&self) -> bool {
        self.decision.is_some() || self.hook_specific_output.is_some()
    }

    fn updated_input(&self, hook_name: &str) -> Option<serde_json::Value> {
        let value = self.hook_specific_output.as_ref()?.updated_input.as_ref()?;
        if value.is_object() {
            return Some(value.clone());
        }
        tracing::warn!(hook_name, "ignoring non-object `updatedInput` from hook");
        None
    }
}

/// Interpret a [`GateHookJson`] as a [`HookDecision`]. An unknown decision value
/// is an error so typos surface instead of failing open.
///
/// `fallback_reason` supplies the deny message when the JSON carries none
/// (command hooks pass the first stderr line — the hook's feedback channel;
/// HTTP hooks have no stderr and pass `None`).
pub(crate) fn gate_json_to_decision(
    json: &GateHookJson,
    hook_name: &str,
    fallback_reason: Option<&str>,
) -> Result<HookDecision, String> {
    match json.decision.as_deref() {
        Some("deny") => Ok(HookDecision::Deny {
            reason: json
                .reason
                .clone()
                .filter(|r| !r.trim().is_empty())
                .or_else(|| fallback_reason.map(str::to_string))
                .unwrap_or_else(|| format!("denied by hook '{hook_name}'")),
            hook_name: hook_name.to_string(),
        }),
        Some("allow") | None => Ok(HookDecision::Allow),
        Some(other) => Err(format!(
            "unknown decision value '{other}' from hook '{hook_name}'"
        )),
    }
}

/// JSON vocabulary shared by command and HTTP stop gates.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct StopHookJson {
    #[serde(default)]
    pub decision: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default, rename = "continue")]
    pub continue_: Option<bool>,
    #[serde(default, rename = "stopReason")]
    pub stop_reason: Option<String>,
    #[serde(default, rename = "hookSpecificOutput")]
    pub hook_specific_output: Option<StopHookSpecificOutputJson>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct StopHookSpecificOutputJson {
    #[serde(default, rename = "additionalContext")]
    pub additional_context: Option<String>,
}

pub(crate) fn stop_json_to_outcome(
    json: StopHookJson,
    hook_name: &str,
) -> Result<StopHookOutcome, String> {
    let block_reason = match json.decision.as_deref() {
        Some("block") => Some(
            json.reason
                .filter(|reason| !reason.trim().is_empty())
                .unwrap_or_else(|| format!("Blocked by stop hook '{hook_name}'")),
        ),
        Some("approve") | None => None,
        Some(other) => {
            return Err(format!(
                "unknown decision value '{other}' from hook '{hook_name}'"
            ));
        }
    };
    Ok(StopHookOutcome {
        block_reason,
        additional_context: json
            .hook_specific_output
            .and_then(|output| output.additional_context)
            .filter(|context| !context.trim().is_empty()),
        force_stop: (json.continue_ == Some(false)).then_some(crate::result::StopOverride {
            reason: json.stop_reason.filter(|reason| !reason.trim().is_empty()),
        }),
    })
}

/// Bundle returned by each runner: result, duration, and optional HTTP metadata.
pub type HookRunOutput = (HookRunnerResult, Duration, Option<HttpInfo>);

pub async fn run_hook(
    spec: &HookSpec,
    envelope: &HookEventEnvelope,
    ctx: &RunContext<'_>,
    mode: GateKind,
) -> HookRunOutput {
    match spec.handler_type {
        crate::config::HandlerType::Command => {
            let (result, elapsed) = command::run_command_hook(spec, envelope, ctx, mode).await;
            (result, elapsed, None)
        }
        crate::config::HandlerType::Http => http::run_http_hook(spec, envelope, ctx, mode).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stop_json_supports_block_context_and_force_stop() {
        let json: StopHookJson = serde_json::from_str(
            r#"{"decision":"block","reason":"tests","continue":false,"stopReason":"forced","hookSpecificOutput":{"additionalContext":"lint"}}"#,
        )
        .unwrap();
        let outcome = stop_json_to_outcome(json, "stop").unwrap();
        assert_eq!(outcome.block_reason.as_deref(), Some("tests"));
        assert_eq!(outcome.additional_context.as_deref(), Some("lint"));
        assert_eq!(
            outcome.force_stop.and_then(|force| force.reason).as_deref(),
            Some("forced")
        );
    }

    #[test]
    fn malformed_stop_decisions_are_errors() {
        let json: StopHookJson = serde_json::from_str(r#"{"decision":"deny"}"#).unwrap();
        assert!(stop_json_to_outcome(json, "stop").is_err());
    }
}
