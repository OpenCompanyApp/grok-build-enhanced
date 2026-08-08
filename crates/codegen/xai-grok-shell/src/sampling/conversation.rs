//! API-agnostic conversation representation.
//!
//! The canonical types now live in `xai_grok_sampling_types::conversation`.
//! This module re-exports them and adds grok-shell-specific types
//! (`ConversationRequestTrace`) that depend on internal crate types.

use std::collections::HashSet;

// Re-export everything from the standalone crate.
pub use xai_grok_sampling_types::conversation::*;

// ============================================================================
// grok-shell-specific types (depend on internal crate types)
// ============================================================================

/// Tracing context for conversation requests.
///
/// Stays in `xai-grok-shell` because it references
/// `TraceExportConfig` (a shell-internal type) and the
/// `ArtifactTracker` from the upload pipeline. The legacy
/// `stream_via_*` path used `artifact_tracker` to spawn fire-and-
/// forget GCS uploads of the request payload; that path was later removed
/// without re-wiring trace upload through the new sampler. The field
/// is kept so the struct shape stays compatible with persisted snapshots
/// and so trace upload can be re-enabled on the sampler path without a
/// schema change.
#[derive(Debug, Clone)]
pub struct ConversationRequestTrace {
    pub gcs_config: crate::session::repo_changes::TraceExportConfig,
    #[expect(
        dead_code,
        reason = "retained for snapshot compat; wire when sampler path uploads traces"
    )]
    pub(crate) artifact_tracker: Option<crate::upload::manifest::ArtifactTracker>,
}

// `ConversationRequestTrace` satisfies the `TraceContext` trait bounds
// (`Clone + Send + Sync + Debug + 'static`) via the blanket impl, so it can
// be stored in `ConversationRequest.trace` and `ChatCompletionRequest.trace`
// via `Box::new(trace)`.
//
// Tests for conversation types now live in xai-grok-sampling-types crate.

/// Fork-safety filter for copied chat history: drops synthetic user messages,
/// then truncates at the last complete turn so the child never sees a partial
/// one. A turn is complete when the Assistant's tool calls are all answered;
/// Reasoning and BackendToolCall items are transparent to the scan.
///
/// NOTE: keep the "complete turn" definition in sync with
/// `count_complete_turns` in `xai-grok-subagent-resolution/src/context.rs`.
pub(crate) fn fork_filter_chat(items: &mut Vec<ConversationItem>) {
    items.retain(|item| match item {
        ConversationItem::User(user) => user.synthetic_reason.is_none(),
        _ => true,
    });

    let mut last_complete_end = 0;
    let mut index = 0;
    while index < items.len() {
        match &items[index] {
            ConversationItem::System(_) => {
                last_complete_end = index + 1;
                index += 1;
            }
            ConversationItem::Assistant(assistant) => {
                let expected: HashSet<&str> = assistant
                    .tool_calls
                    .iter()
                    .map(|call| call.id.as_ref())
                    .collect();
                let mut found = HashSet::new();
                let mut end = index + 1;
                while end < items.len() {
                    match &items[end] {
                        ConversationItem::ToolResult(result) => {
                            if expected.contains(result.tool_call_id.as_str()) {
                                found.insert(result.tool_call_id.as_str());
                            }
                            end += 1;
                        }
                        ConversationItem::Reasoning(_) | ConversationItem::BackendToolCall(_) => {
                            end += 1;
                        }
                        _ => break,
                    }
                }
                if found == expected {
                    last_complete_end = end;
                    index = end;
                } else {
                    break;
                }
            }
            _ => index += 1,
        }
    }

    items.truncate(last_complete_end);
}
