use serde::Serialize;
use xai_grok_tools::types::output::{ExternalContentMetadata, WebToolErrorCode};

/// Maximum serialized size for `toolInput` or `toolResult` in bytes (128 KB).
pub const MAX_PAYLOAD_SIZE: usize = 128 * 1024;

/// Hook event types.
///
/// Accepts both PascalCase (`"PreToolUse"`) and snake_case (`"pre_tool_use"`)
/// during deserialization for migration compatibility.
/// Serializes to snake_case for the hook envelope wire format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HookEventName {
    // ── Session lifecycle ───────────────────────────────────────
    SessionStart,
    SessionEnd,
    /// Gates a genuine agent turn completion; also fires observe-only during
    /// top-level session shutdown. Cancellation, refusal, provider error, and
    /// max-turn termination do not enter this gate.
    Stop,
    /// Fires when the turn ends due to an API error. Output and exit code are ignored.
    StopFailure,

    // ── Tool events ─────────────────────────────────────────────
    PreToolUse,
    PostToolUse,
    /// Fires after a tool call fails (throws an error).
    PostToolUseFailure,
    /// Fires when a tool call is denied by the permission system.
    PermissionDenied,

    // ── User / notification events ──────────────────────────────
    /// Fires when the user submits a prompt.
    UserPromptSubmit,
    /// Fires when a notification is sent (e.g., permission prompt, idle).
    Notification,

    // ── Subagent events ─────────────────────────────────────────
    /// Fires when a subagent is spawned.
    SubagentStart,
    /// Fires when a subagent completes.
    SubagentStop,
    /// Alias for SubagentStop (kept for backward compatibility).
    SubagentEnd,

    // ── Compaction events ───────────────────────────────────────
    /// Fires before context compaction.
    PreCompact,
    /// Fires after context compaction completes.
    PostCompact,
}

impl<'de> serde::Deserialize<'de> for HookEventName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            // PascalCase (native) + snake_case + camelCase (third-party compat).
            // Per-operation hook names (beforeShellExecution, afterFileEdit, etc.)
            // map to our generic PreToolUse/PostToolUse — the hook script receives the
            // tool name in JSON input and can filter, or use the `matcher` field.
            "SessionStart" | "session_start" | "sessionStart" => Ok(Self::SessionStart),
            "PreToolUse"
            | "pre_tool_use"
            | "preToolUse"
            | "beforeShellExecution"
            | "beforeMCPExecution"
            | "beforeReadFile" => Ok(Self::PreToolUse),
            "PostToolUse"
            | "post_tool_use"
            | "postToolUse"
            | "afterShellExecution"
            | "afterMCPExecution"
            | "afterFileEdit"
            | "afterAgentResponse"
            | "afterAgentThought" => Ok(Self::PostToolUse),
            "PostToolUseFailure" | "post_tool_use_failure" | "postToolUseFailure" => {
                Ok(Self::PostToolUseFailure)
            }
            "SessionEnd" | "session_end" | "sessionEnd" => Ok(Self::SessionEnd),
            "Stop" | "stop" => Ok(Self::Stop),
            "StopFailure" | "stop_failure" | "stopFailure" => Ok(Self::StopFailure),
            "Notification" | "notification" => Ok(Self::Notification),
            "UserPromptSubmit" | "user_prompt_submit" | "beforeSubmitPrompt" => {
                Ok(Self::UserPromptSubmit)
            }
            "PermissionDenied" | "permission_denied" | "permissionDenied" => {
                Ok(Self::PermissionDenied)
            }
            "SubagentStart" | "subagent_start" | "subagentStart" => Ok(Self::SubagentStart),
            "SubagentStop" | "subagent_stop" | "subagentStop" => Ok(Self::SubagentStop),
            "SubagentEnd" | "subagent_end" | "subagentEnd" => Ok(Self::SubagentEnd),
            "PreCompact" | "pre_compact" | "preCompact" => Ok(Self::PreCompact),
            "PostCompact" | "post_compact" | "postCompact" => Ok(Self::PostCompact),
            other => Err(serde::de::Error::custom(format!(
                "unknown hook event: '{other}'. Expected one of: \
                 SessionStart, PreToolUse, PostToolUse, PostToolUseFailure, \
                 SessionEnd, Stop, StopFailure, Notification, UserPromptSubmit, \
                 PermissionDenied, SubagentStart, SubagentStop, \
                 PreCompact, PostCompact"
            ))),
        }
    }
}

impl std::fmt::Display for HookEventName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SessionStart => write!(f, "session_start"),
            Self::PreToolUse => write!(f, "pre_tool_use"),
            Self::PostToolUse => write!(f, "post_tool_use"),
            Self::PostToolUseFailure => write!(f, "post_tool_use_failure"),
            Self::SessionEnd => write!(f, "session_end"),
            Self::Stop => write!(f, "stop"),
            Self::StopFailure => write!(f, "stop_failure"),
            Self::Notification => write!(f, "notification"),
            Self::UserPromptSubmit => write!(f, "user_prompt_submit"),
            Self::PermissionDenied => write!(f, "permission_denied"),
            Self::SubagentStart => write!(f, "subagent_start"),
            Self::SubagentStop | Self::SubagentEnd => write!(f, "subagent_stop"),
            Self::PreCompact => write!(f, "pre_compact"),
            Self::PostCompact => write!(f, "post_compact"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateKind {
    /// Hook output is recorded but cannot affect execution.
    Observe,
    /// `PreToolUse` allow/deny gate.
    Tool,
    /// Turn-completion `Stop`/`SubagentStop` gate.
    Stop,
}

impl HookEventName {
    /// Collapse alias variants to their canonical form so a registration and the fired
    /// event meet on one key regardless of which spelling each used (`SubagentEnd` is an
    /// alias of `SubagentStop`).
    pub fn canonical(self) -> Self {
        match self {
            Self::SubagentEnd => Self::SubagentStop,
            other => other,
        }
    }

    /// How this event's hook output is interpreted.
    pub fn gate_kind(self) -> GateKind {
        match self.canonical() {
            Self::PreToolUse => GateKind::Tool,
            Self::Stop | Self::SubagentStop => GateKind::Stop,
            Self::SubagentEnd => unreachable!("canonicalized above"),
            _ => GateKind::Observe,
        }
    }

    /// Returns true if this event type uses the tool allow/deny vocabulary.
    pub fn is_blocking(&self) -> bool {
        matches!(self, Self::PreToolUse)
    }

    /// Events that don't support matcher patterns (fire on every occurrence).
    pub fn is_lifecycle(&self) -> bool {
        matches!(
            self,
            Self::SessionStart | Self::SessionEnd | Self::Stop | Self::UserPromptSubmit
        )
    }
}

/// Maximum serialized character count for every string projected into a
/// `Stop`/`SubagentStop` envelope.
pub const MAX_STOP_ENTRY_TEXT_CHARS: usize = 1000;

/// Maximum serialized size of an entire `Stop`/`SubagentStop` envelope,
/// including common metadata and every active-work descriptor.
pub const MAX_STOP_ENVELOPE_BYTES: usize = 64 * 1024;

/// Clip `text` to at most `max` Unicode scalar values. The omission marker is
/// included in the bound, and slicing never splits a UTF-8 codepoint.
pub fn clip_text(text: &str, max: usize) -> String {
    let char_count = text.chars().count();
    if char_count <= max {
        return text.to_string();
    }
    if max == 0 {
        return String::new();
    }

    // The marker's omitted count depends on how many characters the marker
    // itself displaces. Iterate to a fixed point (normally two passes).
    let mut kept = max;
    loop {
        let omitted = char_count.saturating_sub(kept);
        let marker = format!("… [+{omitted} chars]");
        let next_kept = max.saturating_sub(marker.chars().count());
        if next_kept == kept {
            let clipped: String = text.chars().take(kept).collect();
            return format!("{clipped}{marker}");
        }
        kept = next_kept;
    }
}

/// Sanitize model/user-authored text before placing it in a stop-hook payload.
///
/// This is deliberately a projection boundary, not serialization of an
/// internal task/provider object. It strips terminal controls, redacts common
/// credential/header/account assignments and token shapes, then applies the
/// Unicode-safe free-text bound.
pub fn sanitize_stop_text(text: &str) -> String {
    use std::sync::LazyLock;

    static BEARER: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"(?i)\bbearer\s+[A-Za-z0-9._~+/=-]+").expect("valid bearer regex")
    });
    static SECRET_ASSIGNMENT: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(
            r#"(?ix)
            \b(
                authorization|proxy[-_]?authorization|api[-_]?key|
                access[-_]?token|refresh[-_]?token|id[-_]?token|password|passwd|
                client[-_]?secret|secret|cookie|set[-_]?cookie|account[-_]?id|
                codex[-_]?turn[-_]?state
            )\b
            (\s*[:=]\s*)
            (?:"[^"]*"|'[^']*'|[^\s,;\}\]]+)
            "#,
        )
        .expect("valid stop secret-assignment regex")
    });
    static TOKEN_SHAPE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(
            r"(?x)\b(?:sk-[A-Za-z0-9_-]{8,}|xai-[A-Za-z0-9_-]{8,}|gh[pousr]_[A-Za-z0-9_]{8,}|AKIA[0-9A-Z]{16}|eyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,})\b",
        )
        .expect("valid token-shape regex")
    });

    let controls_stripped: String = text
        .chars()
        .filter(|c| !c.is_control() || matches!(c, '\n' | '\t'))
        .collect();
    let redacted = BEARER.replace_all(&controls_stripped, "[redacted]");
    let redacted = SECRET_ASSIGNMENT.replace_all(&redacted, "$1$2[redacted]");
    let redacted = TOKEN_SHAPE.replace_all(&redacted, "[redacted]");
    clip_text(&redacted, MAX_STOP_ENTRY_TEXT_CHARS)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SubagentStopPhase {
    Gate,
    /// Reserved for compatibility; genuine completion currently fires `Gate`.
    Observe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BackgroundTaskType {
    Shell,
    Monitor,
    Subagent,
}

/// Whitelisted, credential-free projection of one in-flight work item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StopBackgroundTask {
    pub id: String,
    pub r#type: BackgroundTaskType,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,
}

/// Whitelisted projection of a session-scoped scheduled wakeup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StopSessionCron {
    pub id: String,
    pub schedule: String,
    pub recurring: bool,
    pub prompt: String,
}

impl StopBackgroundTask {
    fn sanitize(&mut self) {
        self.id = sanitize_stop_text(&self.id);
        self.status = sanitize_stop_text(&self.status);
        self.description = self
            .description
            .take()
            .map(|value| sanitize_stop_text(&value));
        self.command = self.command.take().map(|value| sanitize_stop_text(&value));
        self.agent_type = self
            .agent_type
            .take()
            .map(|value| sanitize_stop_text(&value));
    }
}

impl StopSessionCron {
    fn sanitize(&mut self) {
        self.id = sanitize_stop_text(&self.id);
        self.schedule = sanitize_stop_text(&self.schedule);
        self.prompt = sanitize_stop_text(&self.prompt);
    }
}

/// The normalized event envelope sent to hook commands on stdin as JSON.
///
/// Contains common metadata plus an event-specific payload.
/// All field names use camelCase for the JSON wire format.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HookEventEnvelope {
    pub hook_event_name: HookEventName,
    pub session_id: String,
    pub cwd: String,
    pub workspace_root: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_id: Option<String>,
    #[serde(flatten)]
    pub payload: HookPayload,
}

/// Event-specific payload variants, flattened into the envelope JSON via
/// `#[serde(untagged)]`. Grouped to match `HookEventName`.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum HookPayload {
    // ── Session lifecycle ───────────────────────────────────────
    SessionStart {
        source: String,
        #[serde(rename = "modelId", skip_serializing_if = "Option::is_none")]
        model_id: Option<String>,
        #[serde(rename = "agentType", skip_serializing_if = "Option::is_none")]
        agent_type: Option<String>,
    },
    SessionEnd {
        reason: String,
        #[serde(rename = "turnCount", skip_serializing_if = "Option::is_none")]
        turn_count: Option<u64>,
        #[serde(rename = "toolCallCount", skip_serializing_if = "Option::is_none")]
        tool_call_count: Option<u64>,
    },
    Stop {
        reason: String,
        /// True after a previous stop hook already continued this turn.
        #[serde(rename = "stopHookActive")]
        stop_hook_active: bool,
        #[serde(
            rename = "lastAssistantMessage",
            skip_serializing_if = "Option::is_none"
        )]
        last_assistant_message: Option<String>,
        /// Whitelisted active-work descriptors. `None` means this fire site does
        /// not enumerate work (for example, observe-only session shutdown).
        #[serde(rename = "backgroundTasks", skip_serializing_if = "Option::is_none")]
        background_tasks: Option<Vec<StopBackgroundTask>>,
        #[serde(rename = "sessionCrons", skip_serializing_if = "Option::is_none")]
        session_crons: Option<Vec<StopSessionCron>>,
    },
    StopFailure {
        error: String,
    },

    // ── Tool events ─────────────────────────────────────────────
    PreToolUse {
        /// The tool the model invoked. For the meta-dispatch tools (`use_tool`
        /// and the external MCP-call tool) this is the resolved underlying tool
        /// (`server__tool`), not the dispatcher — matchers key on it directly.
        #[serde(rename = "toolName")]
        tool_name: String,
        #[serde(rename = "toolUseId")]
        tool_use_id: String,
        #[serde(rename = "toolInput")]
        tool_input: serde_json::Value,
        #[serde(rename = "toolInputTruncated")]
        tool_input_truncated: bool,
        #[serde(rename = "permissionMode", skip_serializing_if = "Option::is_none")]
        permission_mode: Option<String>,
        /// The subagent's type when this tool runs inside one (the envelope's `sessionId`
        /// gives its identity); `None` for the top-level session.
        #[serde(rename = "subagentType", skip_serializing_if = "Option::is_none")]
        subagent_type: Option<String>,
    },
    PostToolUse {
        /// Resolved underlying tool for meta-dispatch tools (see `PreToolUse`).
        #[serde(rename = "toolName")]
        tool_name: String,
        #[serde(rename = "toolUseId")]
        tool_use_id: String,
        #[serde(rename = "toolInput")]
        tool_input: serde_json::Value,
        #[serde(rename = "toolResult")]
        tool_result: serde_json::Value,
        #[serde(rename = "toolInputTruncated")]
        tool_input_truncated: bool,
        #[serde(rename = "toolResultTruncated")]
        tool_result_truncated: bool,
        /// Non-secret typed provenance when the model-visible result came from
        /// an untrusted external web source.
        #[serde(rename = "externalContent", skip_serializing_if = "Option::is_none")]
        external_content: Option<ExternalContentMetadata>,
        /// Stable sanitized failure code for native web tools.
        #[serde(rename = "webFailureCode", skip_serializing_if = "Option::is_none")]
        web_failure_code: Option<WebToolErrorCode>,
        #[serde(rename = "durationMs", skip_serializing_if = "Option::is_none")]
        duration_ms: Option<u64>,
        #[serde(rename = "isBackgrounded")]
        is_backgrounded: bool,
        #[serde(rename = "subagentType", skip_serializing_if = "Option::is_none")]
        subagent_type: Option<String>,
    },
    PostToolUseFailure {
        /// Resolved underlying tool for meta-dispatch tools (see `PreToolUse`).
        #[serde(rename = "toolName")]
        tool_name: String,
        #[serde(rename = "toolUseId")]
        tool_use_id: String,
        #[serde(rename = "toolInput")]
        tool_input: serde_json::Value,
        #[serde(rename = "toolInputTruncated")]
        tool_input_truncated: bool,
        error: String,
        #[serde(rename = "subagentType", skip_serializing_if = "Option::is_none")]
        subagent_type: Option<String>,
    },
    PermissionDenied {
        /// Resolved underlying tool for meta-dispatch tools (see `PreToolUse`).
        #[serde(rename = "toolName")]
        tool_name: String,
        #[serde(rename = "toolUseId")]
        tool_use_id: String,
        #[serde(rename = "toolInput")]
        tool_input: serde_json::Value,
        #[serde(rename = "toolInputTruncated")]
        tool_input_truncated: bool,
    },

    // ── User / notification events ──────────────────────────────
    /// Fires when the user submits a prompt.
    UserPromptSubmit {
        #[serde(skip_serializing_if = "Option::is_none")]
        prompt: Option<String>,
    },
    /// Fires on agent notifications (permission prompts, idle, etc.).
    Notification {
        #[serde(rename = "notificationType")]
        notification_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        /// Compat: some callers use `level` instead of `notificationType`.
        #[serde(skip_serializing_if = "Option::is_none")]
        level: Option<String>,
    },

    // ── Subagent events ─────────────────────────────────────────
    /// Fires when a subagent is spawned.
    SubagentStart {
        #[serde(rename = "subagentId")]
        subagent_id: String,
        #[serde(rename = "subagentType")]
        subagent_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
    /// Fires as the genuine subagent turn-completion gate.
    SubagentStop {
        phase: SubagentStopPhase,
        #[serde(rename = "subagentId")]
        subagent_id: String,
        #[serde(rename = "subagentType")]
        subagent_type: String,
        #[serde(rename = "stopHookActive", skip_serializing_if = "Option::is_none")]
        stop_hook_active: Option<bool>,
        #[serde(
            rename = "lastAssistantMessage",
            skip_serializing_if = "Option::is_none"
        )]
        last_assistant_message: Option<String>,
    },

    // ── Compaction events ───────────────────────────────────────
    PreCompact {
        /// "manual" or "auto".
        source: String,
    },
    PostCompact {
        /// "manual" or "auto".
        source: String,
    },
}

impl HookPayload {
    /// Value tested by matcher-aware events. Events with no selector return
    /// `None`, which intentionally means match-all (fail open).
    pub fn match_value(&self) -> Option<&str> {
        let value = match self {
            Self::PreToolUse { tool_name, .. }
            | Self::PostToolUse { tool_name, .. }
            | Self::PostToolUseFailure { tool_name, .. }
            | Self::PermissionDenied { tool_name, .. } => tool_name,
            Self::Notification {
                notification_type, ..
            } => notification_type,
            Self::SubagentStart { subagent_type, .. }
            | Self::SubagentStop { subagent_type, .. } => subagent_type,
            Self::SessionStart { source, .. }
            | Self::PreCompact { source }
            | Self::PostCompact { source } => source,
            Self::SessionEnd { reason, .. } => reason,
            Self::StopFailure { error } => error,
            Self::Stop { .. } | Self::UserPromptSubmit { .. } => return None,
        };
        Some(value.as_str()).filter(|value| !value.is_empty())
    }

    fn sanitize_stop_projection(&mut self) {
        match self {
            Self::Stop {
                reason,
                last_assistant_message,
                background_tasks,
                session_crons,
                ..
            } => {
                *reason = sanitize_stop_text(reason);
                *last_assistant_message = last_assistant_message
                    .take()
                    .map(|value| sanitize_stop_text(&value))
                    .filter(|value| !value.trim().is_empty());
                if let Some(tasks) = background_tasks {
                    for task in tasks.iter_mut() {
                        task.sanitize();
                    }
                    tasks.sort_by(|left, right| {
                        left.r#type
                            .cmp(&right.r#type)
                            .then_with(|| left.id.cmp(&right.id))
                    });
                }
                if let Some(crons) = session_crons {
                    for cron in crons.iter_mut() {
                        cron.sanitize();
                    }
                    crons.sort_by(|left, right| left.id.cmp(&right.id));
                }
            }
            Self::SubagentStop {
                subagent_id,
                subagent_type,
                last_assistant_message,
                ..
            } => {
                *subagent_id = sanitize_stop_text(subagent_id);
                *subagent_type = sanitize_stop_text(subagent_type);
                *last_assistant_message = last_assistant_message
                    .take()
                    .map(|value| sanitize_stop_text(&value))
                    .filter(|value| !value.trim().is_empty());
            }
            _ => {}
        }
    }

    /// Drop the deterministic tail of the active-work projection. Scheduled
    /// wakeups are removed before background work, then assistant text; the
    /// same input therefore always produces the same bounded envelope.
    fn discard_stop_projection_tail(&mut self) -> bool {
        match self {
            Self::Stop {
                last_assistant_message,
                background_tasks,
                session_crons,
                ..
            } => {
                if session_crons
                    .as_mut()
                    .is_some_and(|crons| crons.pop().is_some())
                {
                    return true;
                }
                if background_tasks
                    .as_mut()
                    .is_some_and(|tasks| tasks.pop().is_some())
                {
                    return true;
                }
                last_assistant_message.take().is_some()
            }
            Self::SubagentStop {
                last_assistant_message,
                ..
            } => last_assistant_message.take().is_some(),
            _ => false,
        }
    }
}

impl HookEventEnvelope {
    /// Canonicalize, sanitize, order, and size-bound a stop projection before
    /// any command, HTTP, or ACP client hook can observe it.
    pub fn enforce_stop_projection_bounds(&mut self) {
        self.hook_event_name = self.hook_event_name.canonical();
        if !matches!(
            &self.payload,
            HookPayload::Stop { .. } | HookPayload::SubagentStop { .. }
        ) {
            return;
        }

        self.payload.sanitize_stop_projection();
        self.session_id = clip_text(&self.session_id, MAX_STOP_ENTRY_TEXT_CHARS);
        self.cwd = clip_text(&self.cwd, MAX_STOP_ENTRY_TEXT_CHARS);
        self.workspace_root = clip_text(&self.workspace_root, MAX_STOP_ENTRY_TEXT_CHARS);
        self.timestamp = clip_text(&self.timestamp, MAX_STOP_ENTRY_TEXT_CHARS);
        self.transcript_path = self
            .transcript_path
            .take()
            .map(|value| clip_text(&value, MAX_STOP_ENTRY_TEXT_CHARS));
        self.client_identifier = self
            .client_identifier
            .take()
            .map(|value| clip_text(&value, MAX_STOP_ENTRY_TEXT_CHARS));
        self.prompt_id = self
            .prompt_id
            .take()
            .map(|value| clip_text(&value, MAX_STOP_ENTRY_TEXT_CHARS));

        while serde_json::to_vec(&self).map_or(usize::MAX, |json| json.len())
            > MAX_STOP_ENVELOPE_BYTES
            && self.payload.discard_stop_projection_tail()
        {}

        debug_assert!(
            serde_json::to_vec(&self).is_ok_and(|json| json.len() <= MAX_STOP_ENVELOPE_BYTES),
            "bounded stop envelope exceeded {MAX_STOP_ENVELOPE_BYTES} bytes"
        );
    }
}

/// Truncate a JSON value if its serialized size exceeds `MAX_PAYLOAD_SIZE`.
///
/// Returns `(possibly_truncated_value, was_truncated)`.
pub fn truncate_payload(value: serde_json::Value) -> (serde_json::Value, bool) {
    let serialized = serde_json::to_string(&value).unwrap_or_default();
    if serialized.len() <= MAX_PAYLOAD_SIZE {
        return (value, false);
    }

    // Cut at the largest char boundary <= MAX_PAYLOAD_SIZE so the slice never
    // splits a multibyte codepoint.
    let mut end = MAX_PAYLOAD_SIZE;
    while !serialized.is_char_boundary(end) {
        end -= 1;
    }
    let mut result = serialized[..end].to_string();
    result.push_str(" [truncated]");
    (serde_json::Value::String(result), true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_name_deser_all_variants() {
        let cases: &[(&str, &str, HookEventName)] = &[
            ("SessionStart", "session_start", HookEventName::SessionStart),
            ("PreToolUse", "pre_tool_use", HookEventName::PreToolUse),
            ("PostToolUse", "post_tool_use", HookEventName::PostToolUse),
            (
                "PostToolUseFailure",
                "post_tool_use_failure",
                HookEventName::PostToolUseFailure,
            ),
            ("SessionEnd", "session_end", HookEventName::SessionEnd),
            ("Stop", "stop", HookEventName::Stop),
            ("StopFailure", "stop_failure", HookEventName::StopFailure),
            ("Notification", "notification", HookEventName::Notification),
            (
                "UserPromptSubmit",
                "user_prompt_submit",
                HookEventName::UserPromptSubmit,
            ),
            (
                "PermissionDenied",
                "permission_denied",
                HookEventName::PermissionDenied,
            ),
            (
                "SubagentStart",
                "subagent_start",
                HookEventName::SubagentStart,
            ),
            ("SubagentStop", "subagent_stop", HookEventName::SubagentStop),
            ("SubagentEnd", "subagent_end", HookEventName::SubagentEnd),
            ("PreCompact", "pre_compact", HookEventName::PreCompact),
            ("PostCompact", "post_compact", HookEventName::PostCompact),
        ];

        for (pascal, snake, expected) in cases {
            let from_pascal: HookEventName =
                serde_json::from_str(&format!("\"{pascal}\"")).unwrap();
            assert_eq!(
                from_pascal, *expected,
                "PascalCase deser failed for {pascal}"
            );

            let from_snake: HookEventName = serde_json::from_str(&format!("\"{snake}\"")).unwrap();
            assert_eq!(from_snake, *expected, "snake_case deser failed for {snake}");
        }
    }

    #[test]
    fn event_name_display_all_variants() {
        let cases: &[(HookEventName, &str)] = &[
            (HookEventName::SessionStart, "session_start"),
            (HookEventName::PreToolUse, "pre_tool_use"),
            (HookEventName::PostToolUse, "post_tool_use"),
            (HookEventName::PostToolUseFailure, "post_tool_use_failure"),
            (HookEventName::SessionEnd, "session_end"),
            (HookEventName::Stop, "stop"),
            (HookEventName::StopFailure, "stop_failure"),
            (HookEventName::Notification, "notification"),
            (HookEventName::UserPromptSubmit, "user_prompt_submit"),
            (HookEventName::PermissionDenied, "permission_denied"),
            (HookEventName::SubagentStart, "subagent_start"),
            (HookEventName::SubagentStop, "subagent_stop"),
            (HookEventName::SubagentEnd, "subagent_stop"), // alias collapses
            (HookEventName::PreCompact, "pre_compact"),
            (HookEventName::PostCompact, "post_compact"),
        ];
        for (event, expected) in cases {
            assert_eq!(&event.to_string(), expected, "Display wrong for {event:?}");
        }
    }

    #[test]
    fn event_name_serde_roundtrip() {
        let name = HookEventName::PreToolUse;
        let json = serde_json::to_string(&name).unwrap();
        assert_eq!(json, "\"pre_tool_use\"");
        let parsed: HookEventName = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, name);
    }

    #[test]
    fn event_name_unknown_rejected() {
        let result = serde_json::from_str::<HookEventName>("\"UnknownEvent\"");
        assert!(result.is_err());
    }

    #[test]
    fn event_name_is_blocking() {
        assert!(HookEventName::PreToolUse.is_blocking());
        for event in [
            HookEventName::SessionStart,
            HookEventName::PostToolUse,
            HookEventName::PostToolUseFailure,
            HookEventName::SessionEnd,
            HookEventName::Stop,
            HookEventName::StopFailure,
            HookEventName::Notification,
            HookEventName::UserPromptSubmit,
            HookEventName::PermissionDenied,
            HookEventName::SubagentStart,
            HookEventName::SubagentStop,
            HookEventName::SubagentEnd,
            HookEventName::PreCompact,
            HookEventName::PostCompact,
        ] {
            assert!(!event.is_blocking(), "{event:?} should not be blocking");
        }
    }

    #[test]
    fn event_name_is_lifecycle() {
        let lifecycle = [
            HookEventName::SessionStart,
            HookEventName::SessionEnd,
            HookEventName::Stop,
            HookEventName::UserPromptSubmit,
        ];
        for event in lifecycle {
            assert!(event.is_lifecycle(), "{event:?} should be lifecycle");
        }

        let matchable = [
            HookEventName::PreToolUse,
            HookEventName::PostToolUse,
            HookEventName::PostToolUseFailure,
            HookEventName::PermissionDenied,
            HookEventName::StopFailure,
            HookEventName::Notification,
            HookEventName::SubagentStart,
            HookEventName::SubagentStop,
            HookEventName::SubagentEnd,
            HookEventName::PreCompact,
            HookEventName::PostCompact,
        ];
        for event in matchable {
            assert!(
                !event.is_lifecycle(),
                "{event:?} should support matchers, not be lifecycle"
            );
        }
    }

    #[test]
    fn truncate_small_payload() {
        let value = serde_json::json!({"key": "small"});
        let (result, truncated) = truncate_payload(value.clone());
        assert!(!truncated);
        assert_eq!(result, value);
    }

    #[test]
    fn truncate_large_payload() {
        let big_string = "x".repeat(MAX_PAYLOAD_SIZE + 1000);
        let value = serde_json::Value::String(big_string);
        let (result, truncated) = truncate_payload(value);
        assert!(truncated);
        let s = result.as_str().unwrap();
        assert!(s.ends_with("[truncated]"));
        // Serialized size of the result string value should be <= MAX_PAYLOAD_SIZE + overhead
        assert!(s.len() < MAX_PAYLOAD_SIZE + 100);
    }

    #[test]
    fn truncate_large_payload_cuts_on_char_boundary() {
        // '€' is 3 bytes, so the MAX_PAYLOAD_SIZE-th byte lands mid-codepoint.
        let value = serde_json::Value::String("€".repeat(MAX_PAYLOAD_SIZE));
        let (result, truncated) = truncate_payload(value);
        assert!(truncated);
        assert!(result.as_str().unwrap().ends_with("[truncated]"));
    }

    #[test]
    fn envelope_serializes_camel_case() {
        let envelope = HookEventEnvelope {
            hook_event_name: HookEventName::SessionStart,
            session_id: "test-session".into(),
            cwd: "/tmp".into(),
            workspace_root: "/tmp".into(),
            timestamp: "2025-01-01T00:00:00Z".into(),
            transcript_path: None,
            client_identifier: None,
            prompt_id: None,
            payload: HookPayload::SessionStart {
                source: "new".into(),
                model_id: Some("grok-3".into()),
                agent_type: None,
            },
        };
        let json = serde_json::to_string(&envelope).unwrap();
        assert!(json.contains("hookEventName"));
        assert!(json.contains("sessionId"));
        assert!(json.contains("workspaceRoot"));
        assert!(json.contains("modelId"));
        // Should NOT contain snake_case versions
        assert!(!json.contains("hook_event_name"));
        assert!(!json.contains("session_id"));
    }

    #[test]
    fn post_tool_use_serializes_external_content_metadata() {
        use xai_grok_tools::types::output::ExternalContentSource;

        let payload = HookPayload::PostToolUse {
            tool_name: "web_search".into(),
            tool_use_id: "call-web".into(),
            tool_input: serde_json::json!({"query": "example"}),
            tool_result: serde_json::json!({"type": "web_search"}),
            tool_input_truncated: false,
            tool_result_truncated: false,
            external_content: Some(ExternalContentMetadata::direct(
                ExternalContentSource::WebSearch,
            )),
            web_failure_code: None,
            duration_ms: Some(5),
            is_backgrounded: false,
            subagent_type: None,
        };

        let json = serde_json::to_value(payload).unwrap();
        assert_eq!(json["externalContent"]["sources"][0], "web_search");
        assert_eq!(json["externalContent"]["derived"], false);
    }

    #[test]
    fn stop_text_clipping_is_unicode_safe_and_includes_marker_in_bound() {
        let clipped = clip_text(&"🦀".repeat(2_000), MAX_STOP_ENTRY_TEXT_CHARS);
        assert!(clipped.is_char_boundary(clipped.len()));
        assert_eq!(clipped.chars().count(), MAX_STOP_ENTRY_TEXT_CHARS);
        assert!(clipped.contains("[+"));
        assert!(clipped.ends_with(" chars]"));
    }

    #[test]
    fn stop_text_redacts_credentials_and_controls_before_projection() {
        let text =
            "\u{1b}[31mAuthorization: Bearer top-secret api_key=sk-abcdefghijk account_id=acct-1";
        let sanitized = sanitize_stop_text(text);
        assert!(!sanitized.contains('\u{1b}'));
        assert!(!sanitized.contains("top-secret"));
        assert!(!sanitized.contains("sk-abcdefghijk"));
        assert!(!sanitized.contains("acct-1"));
        assert!(sanitized.matches("[redacted]").count() >= 3);
        assert!(sanitized.chars().count() <= MAX_STOP_ENTRY_TEXT_CHARS);
    }

    #[test]
    fn stop_envelope_projection_is_sorted_secret_free_and_aggregate_bounded() {
        let tasks = (0..200)
            .rev()
            .map(|index| StopBackgroundTask {
                id: format!("task-{index:03}"),
                r#type: if index % 2 == 0 {
                    BackgroundTaskType::Shell
                } else {
                    BackgroundTaskType::Monitor
                },
                status: "running".into(),
                description: Some(format!(
                    "refresh_token=secret-{index} {}",
                    "🦀".repeat(1_500)
                )),
                command: Some(format!(
                    "Authorization: Bearer bearer-{index} {}",
                    "x".repeat(1_500)
                )),
                agent_type: Some("explore".into()),
            })
            .collect();
        let crons = (0..100)
            .rev()
            .map(|index| StopSessionCron {
                id: format!("cron-{index:03}"),
                schedule: "every 1 minute".into(),
                recurring: true,
                prompt: format!("api_key=secret-{index} {}", "p".repeat(1_500)),
            })
            .collect();
        let mut envelope = HookEventEnvelope {
            hook_event_name: HookEventName::Stop,
            session_id: "session-a".into(),
            cwd: "/tmp".into(),
            workspace_root: "/tmp".into(),
            timestamp: "2026-07-20T00:00:00Z".into(),
            transcript_path: None,
            client_identifier: None,
            prompt_id: Some("prompt-1".into()),
            payload: HookPayload::Stop {
                reason: "end_turn".into(),
                stop_hook_active: false,
                last_assistant_message: Some(
                    "codex_turn_state=private Authorization: Bearer model-secret".into(),
                ),
                background_tasks: Some(tasks),
                session_crons: Some(crons),
            },
        };

        envelope.enforce_stop_projection_bounds();
        let bytes = serde_json::to_vec(&envelope).unwrap();
        assert!(
            bytes.len() <= MAX_STOP_ENVELOPE_BYTES,
            "{} bytes",
            bytes.len()
        );
        let wire = String::from_utf8(bytes).unwrap();
        for secret in ["secret-", "model-secret", "private", "bearer-"] {
            assert!(!wire.contains(secret), "projected secret {secret:?}");
        }
        for forbidden_key in [
            "credentials",
            "providerState",
            "codexTurnState",
            "accountId",
            "apiKey",
        ] {
            assert!(!wire.contains(&format!("\"{forbidden_key}\"")));
        }

        let HookPayload::Stop {
            background_tasks: Some(tasks),
            session_crons: Some(crons),
            last_assistant_message,
            ..
        } = &envelope.payload
        else {
            panic!("expected bounded Stop payload");
        };
        assert!(
            crons.is_empty(),
            "deterministic pruning removes crons first"
        );
        assert!(tasks.len() < 200, "aggregate bound must prune task tail");
        assert!(tasks.windows(2).all(|pair| {
            (pair[0].r#type, pair[0].id.as_str()) <= (pair[1].r#type, pair[1].id.as_str())
        }));
        for task in tasks {
            for value in [
                Some(task.id.as_str()),
                Some(task.status.as_str()),
                task.description.as_deref(),
                task.command.as_deref(),
                task.agent_type.as_deref(),
            ]
            .into_iter()
            .flatten()
            {
                assert!(value.chars().count() <= MAX_STOP_ENTRY_TEXT_CHARS);
            }
        }
        assert!(
            last_assistant_message
                .as_deref()
                .is_none_or(|value| value.chars().count() <= MAX_STOP_ENTRY_TEXT_CHARS)
        );
    }
}
