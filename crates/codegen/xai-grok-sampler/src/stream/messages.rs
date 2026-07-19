//! Layer-2 stream transform for the Anthropic Messages API.
//!
//! Consumes a raw `MessageStreamEvent` stream and produces
//! [`SamplingEvent`]s. Pure: no I/O, no shell coupling.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use futures_util::stream::{BoxStream, Stream};

use xai_grok_sampling_types::messages::{self, MessageStreamEvent};
use xai_grok_sampling_types::{
    AssistantItem, ConversationItem, ConversationResponse, ProviderId, ResponseModelMetadata,
    SamplingError, StopReason, TokenUsage, ToolCall, rs,
};

use crate::events::{SamplingChannel, SamplingErrorInfo, SamplingEvent};
use crate::metrics::InferenceLatencyStats;
use crate::types::RequestId;

/// Returns whether a Messages API event reflects real model progress
/// rather than a liveness-only heartbeat (Ping).
pub(crate) fn messages_event_has_meaningful_content(event: &MessageStreamEvent) -> bool {
    match event {
        MessageStreamEvent::Ping | MessageStreamEvent::Unknown => false,
        MessageStreamEvent::MessageStart { .. }
        | MessageStreamEvent::MessageDelta { .. }
        | MessageStreamEvent::MessageStop
        | MessageStreamEvent::ContentBlockStart { .. }
        | MessageStreamEvent::ContentBlockDelta { .. }
        | MessageStreamEvent::ContentBlockStop { .. }
        | MessageStreamEvent::Error { .. } => true,
    }
}

/// Per-block streaming accumulator. The Anthropic Messages API reports
/// content as a sequence of indexed blocks (text / thinking /
/// tool_use), each with start / delta / stop events. We accumulate
/// per-index and finalize each block on `ContentBlockStop`.
struct BlockState {
    block_type: BlockType,
    text_acc: String,
    tool_name: String,
    tool_id: String,
    args_acc: String,
    thinking_acc: String,
    signature: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockType {
    Text,
    ToolUse,
    Thinking,
}

/// Transform a raw Anthropic Messages API stream into a stream of
/// [`SamplingEvent`]s.
///
/// Yields exactly one terminal event ([`SamplingEvent::Completed`] or
/// [`SamplingEvent::Failed`]) per request. Server-side `Error` events use
/// provider-specific classification when one is available.
pub fn stream_messages<'a>(
    raw_stream: BoxStream<'a, Result<MessageStreamEvent, SamplingError>>,
    model_metadata: Option<ResponseModelMetadata>,
    request_id: RequestId,
    idle_timeout: Duration,
) -> impl Stream<Item = SamplingEvent> + Send + 'a {
    stream_messages_for_provider(
        raw_stream,
        model_metadata,
        request_id,
        idle_timeout,
        ProviderId::Xai,
    )
}

pub(crate) fn stream_messages_for_provider<'a>(
    raw_stream: BoxStream<'a, Result<MessageStreamEvent, SamplingError>>,
    model_metadata: Option<ResponseModelMetadata>,
    request_id: RequestId,
    idle_timeout: Duration,
    provider: ProviderId,
) -> impl Stream<Item = SamplingEvent> + Send + 'a {
    async_stream::stream! {
        use messages::{ContentBlock, StreamDelta};

        let stream_start = Instant::now();
        let mut chunk_timestamps: Vec<Instant> = Vec::new();

        yield SamplingEvent::StreamStarted {
            request_id: request_id.clone(),
            timestamp_ms: chrono::Utc::now().timestamp_millis(),
        };

        if let Some(metadata) = model_metadata {
            yield SamplingEvent::ModelMetadata {
                request_id: request_id.clone(),
                metadata,
            };
        }

        // Per-block accumulators keyed by content block index.
        let mut blocks: BTreeMap<u32, BlockState> = BTreeMap::new();

        // Final-message-level accumulators
        let mut final_model: Option<String> = None;
        // Anthropic Messages API `input_tokens` is the uncached portion; cache hits and writes are reported
        // in separate buckets and must be summed for the true total prompt size.
        let mut final_input_tokens: u32 = 0;
        let mut final_cache_read_input_tokens: u32 = 0;
        let mut final_cache_creation_input_tokens: u32 = 0;
        let mut final_output_tokens: u32 = 0;
        let mut final_stop_reason: Option<StopReason> = None;
        let mut final_stop_message: Option<String> = None;

        // Assistant-response accumulators (built up as ContentBlockStop
        // events fire). Reasoning is collected into a synthesized
        // `rs::ReasoningItem` and emitted as a sibling
        // `ConversationItem::Reasoning` before the trailing Assistant.
        let mut assistant_text = String::new();
        let mut assistant_tool_calls: Vec<ToolCall> = Vec::new();
        let mut assistant_reasoning: Vec<rs::ReasoningItem> = Vec::new();

        // Index counters
        let mut chunk_index: u64 = 0;
        let mut message_chunk_count: u64 = 0;
        let mut first_token_emitted = false;
        let mut last_content_chunk_at = Instant::now();

        // Tool-call index counter for per-tool deltas (separate from
        // the block index, which can be interleaved with text/thinking
        // blocks).
        let mut next_tool_index: u32 = 0;
        let mut block_to_tool_index: BTreeMap<u32, u32> = BTreeMap::new();

        let mut stream = raw_stream;
        loop {
            let event_result = match tokio::time::timeout(idle_timeout, stream.next()).await {
                Ok(Some(event_result)) => event_result,
                Ok(None) => break,
                Err(_elapsed) => {
                    let err = SamplingError::IdleTimeout {
                        elapsed_secs: idle_timeout.as_secs(),
                    };
                    yield SamplingEvent::Failed {
                        request_id: request_id.clone(),
                        error: SamplingErrorInfo::from(&err),
                    };
                    return;
                }
            };

            let event = match event_result {
                Ok(event) => event,
                Err(err) => {
                    yield SamplingEvent::Failed {
                        request_id: request_id.clone(),
                        error: SamplingErrorInfo::from(&err),
                    };
                    return;
                }
            };

            let event_has_content = messages_event_has_meaningful_content(&event);

            match event {
                MessageStreamEvent::MessageStart { message } => {
                    final_model = Some(message.model.clone());
                    final_input_tokens = message.usage.input_tokens;
                    final_cache_read_input_tokens = message.usage.cache_read_input_tokens;
                    final_cache_creation_input_tokens = message.usage.cache_creation_input_tokens;
                }

                MessageStreamEvent::ContentBlockStart {
                    index,
                    content_block,
                } => match content_block {
                    ContentBlock::Thinking {
                        thinking,
                        signature,
                    } => {
                        blocks.insert(
                            index,
                            BlockState {
                                block_type: BlockType::Thinking,
                                text_acc: String::new(),
                                tool_name: String::new(),
                                tool_id: String::new(),
                                args_acc: String::new(),
                                thinking_acc: thinking.clone(),
                                signature: signature.clone(),
                            },
                        );
                        if !first_token_emitted {
                            first_token_emitted = true;
                            yield SamplingEvent::FirstToken {
                                request_id: request_id.clone(),
                            };
                        }
                        if !thinking.is_empty() {
                            chunk_index += 1;
                            yield SamplingEvent::ChannelToken {
                                request_id: request_id.clone(),
                                channel: SamplingChannel::Reasoning,
                                text: thinking,
                                chunk_index,
                            };
                        }
                    }
                    ContentBlock::RedactedThinking { data } => {
                        blocks.insert(
                            index,
                            BlockState {
                                block_type: BlockType::Thinking,
                                text_acc: String::new(),
                                tool_name: String::new(),
                                tool_id: String::new(),
                                args_acc: String::new(),
                                thinking_acc: String::new(),
                                signature: Some(data),
                            },
                        );
                        if !first_token_emitted {
                            first_token_emitted = true;
                            yield SamplingEvent::FirstToken {
                                request_id: request_id.clone(),
                            };
                        }
                    }
                    ContentBlock::Text { text, .. } => {
                        blocks.insert(
                            index,
                            BlockState {
                                block_type: BlockType::Text,
                                text_acc: text.clone(),
                                tool_name: String::new(),
                                tool_id: String::new(),
                                args_acc: String::new(),
                                thinking_acc: String::new(),
                                signature: None,
                            },
                        );
                        if !first_token_emitted {
                            first_token_emitted = true;
                            yield SamplingEvent::FirstToken {
                                request_id: request_id.clone(),
                            };
                        }
                        if !text.is_empty() {
                            chunk_timestamps.push(Instant::now());
                            chunk_index += 1;
                            message_chunk_count += 1;
                            yield SamplingEvent::ChannelToken {
                                request_id: request_id.clone(),
                                channel: SamplingChannel::Text,
                                text,
                                chunk_index,
                            };
                        }
                    }
                    ContentBlock::ToolUse {
                        id,
                        name,
                        input: _,
                    } => {
                        let tool_index = next_tool_index;
                        next_tool_index += 1;
                        block_to_tool_index.insert(index, tool_index);

                        blocks.insert(
                            index,
                            BlockState {
                                block_type: BlockType::ToolUse,
                                text_acc: String::new(),
                                tool_name: name.clone(),
                                tool_id: id.clone(),
                                // Anthropic Messages API streams arguments via
                                // InputJsonDelta events; starting from
                                // "{}" then appending fragments would
                                // produce invalid JSON.
                                args_acc: String::new(),
                                thinking_acc: String::new(),
                                signature: None,
                            },
                        );

                        // Emit initial id+name so subscribers can pre-allocate
                        // UI for the tool call before arguments stream in.
                        yield SamplingEvent::ToolCallDelta {
                            request_id: request_id.clone(),
                            tool_index,
                            id: Some(id),
                            name: Some(name),
                            arguments_delta: None,
                        };
                    }
                    _ => {} // Image / ToolResult are not expected in assistant streams.
                },

                MessageStreamEvent::ContentBlockDelta { index, delta } => {
                    if let Some(state) = blocks.get_mut(&index) {
                        match delta {
                            StreamDelta::ThinkingDelta { thinking } => {
                                if !thinking.is_empty() {
                                    state.thinking_acc.push_str(&thinking);
                                    if !first_token_emitted {
                                        first_token_emitted = true;
                                        yield SamplingEvent::FirstToken {
                                            request_id: request_id.clone(),
                                        };
                                    }
                                    chunk_index += 1;
                                    yield SamplingEvent::ChannelToken {
                                        request_id: request_id.clone(),
                                        channel: SamplingChannel::Reasoning,
                                        text: thinking,
                                        chunk_index,
                                    };
                                }
                            }
                            StreamDelta::SignatureDelta { signature } => {
                                state.signature = Some(signature);
                            }
                            StreamDelta::TextDelta { text } => {
                                if !text.is_empty() {
                                    state.text_acc.push_str(&text);
                                    if !first_token_emitted {
                                        first_token_emitted = true;
                                        yield SamplingEvent::FirstToken {
                                            request_id: request_id.clone(),
                                        };
                                    }
                                    chunk_timestamps.push(Instant::now());
                                    chunk_index += 1;
                                    message_chunk_count += 1;
                                    yield SamplingEvent::ChannelToken {
                                        request_id: request_id.clone(),
                                        channel: SamplingChannel::Text,
                                        text,
                                        chunk_index,
                                    };
                                }
                            }
                            StreamDelta::InputJsonDelta { partial_json } => {
                                state.args_acc.push_str(&partial_json);
                                if let Some(&tool_index) = block_to_tool_index.get(&index) {
                                    yield SamplingEvent::ToolCallDelta {
                                        request_id: request_id.clone(),
                                        tool_index,
                                        id: None,
                                        name: None,
                                        arguments_delta: Some(partial_json),
                                    };
                                }
                            }
                            StreamDelta::Unknown => {}
                        }
                    }
                }

                MessageStreamEvent::ContentBlockStop { index } => {
                    if let Some(state) = blocks.remove(&index) {
                        match state.block_type {
                            BlockType::Text => {
                                if !state.text_acc.is_empty() {
                                    if !assistant_text.is_empty() {
                                        assistant_text.push('\n');
                                    }
                                    assistant_text.push_str(&state.text_acc);
                                }
                            }
                            BlockType::Thinking => {
                                // The block's presence is itself significant.
                                // Kimi can emit an empty unsigned thinking block
                                // before a tool call and requires that exact
                                // shape to survive history replay.
                                let summary = if state.thinking_acc.is_empty() {
                                    vec![]
                                } else {
                                    vec![rs::SummaryPart::SummaryText(
                                        rs::SummaryTextContent {
                                            text: state.thinking_acc,
                                        },
                                    )]
                                };
                                assistant_reasoning.push(rs::ReasoningItem {
                                    id: String::new(),
                                    summary,
                                    content: None,
                                    encrypted_content: state.signature,
                                    status: None,
                                });
                            }
                            BlockType::ToolUse => {
                                assistant_tool_calls.push(ToolCall {
                                    id: std::sync::Arc::<str>::from(state.tool_id),
                                    name: state.tool_name,
                                    arguments: std::sync::Arc::<str>::from(state.args_acc),
                                });
                            }
                        }
                    }
                }

                MessageStreamEvent::MessageDelta { delta, usage } => {
                    // Normalize the provider's stop detail to a plain message;
                    // the shell logs it when it surfaces a refusal.
                    if let Some(details) = delta.stop_details {
                        final_stop_message = if provider.is_kimi_code()
                            && details.explanation.is_some()
                        {
                            Some("Kimi Code declined to continue this response".to_owned())
                        } else {
                            details.explanation
                        };
                    }
                    final_stop_reason = delta.stop_reason.map(|sr| match sr {
                        messages::StopReason::EndTurn => StopReason::Stop,
                        messages::StopReason::MaxTokens => StopReason::Length,
                        messages::StopReason::StopSequence => StopReason::Stop,
                        messages::StopReason::ToolUse => StopReason::ToolCalls,
                        // The model declined to continue; whatever streamed is
                        // the complete response, so end the turn cleanly.
                        messages::StopReason::Refusal => StopReason::ContentFilter,
                        messages::StopReason::PauseTurn => {
                            // Anthropic Messages API expects a resend-to-continue; we end the
                            // turn instead, so leave a triage trail.
                            tracing::warn!(
                                wire_stop_reason = "pause_turn",
                                "pause_turn ended the turn like stop (no auto-continue)"
                            );
                            StopReason::Stop
                        }
                        messages::StopReason::ModelContextWindowExceeded => {
                            // Output-side overflow on a successful stream: stays in the
                            // max_tokens truncation class — compact-on-error recovery needs
                            // an Api error carrying model metadata plus a prompt-side
                            // overflow, neither of which exists here.
                            tracing::warn!(
                                wire_stop_reason = "model_context_window_exceeded",
                                "context window hit mid-generation; surfacing as max_tokens truncation"
                            );
                            StopReason::Length
                        }
                        messages::StopReason::Unknown(wire) => {
                            if provider.is_kimi_code() {
                                tracing::warn!(
                                    "unrecognized Kimi Code stop_reason; treating as stop"
                                );
                            } else {
                                tracing::warn!(
                                    wire_stop_reason = %wire,
                                    "unrecognized stop_reason in messages stream; treating as stop"
                                );
                            }
                            StopReason::Stop
                        }
                    });
                    final_output_tokens = usage.output_tokens;
                    // Optional on the delta; preserve message_start values when omitted.
                    if let Some(input) = usage.input_tokens {
                        final_input_tokens = input;
                    }
                    if let Some(cache_read) = usage.cache_read_input_tokens {
                        final_cache_read_input_tokens = cache_read;
                    }
                    if let Some(cache_creation) = usage.cache_creation_input_tokens {
                        final_cache_creation_input_tokens = cache_creation;
                    }
                }

                MessageStreamEvent::MessageStop => {
                    // Final message complete; the loop exits naturally
                    // when the underlying stream ends.
                }

                MessageStreamEvent::Ping | MessageStreamEvent::Unknown => {
                    // Liveness or an unknown extension event carries no
                    // projectable response content.
                }

                MessageStreamEvent::Error { error } => {
                    let err = if provider.is_kimi_code() {
                        crate::provider::kimi_code::messages_stream_error(
                            &error.r#type,
                            &error.message,
                        )
                    } else {
                        SamplingError::Api {
                            status: reqwest::StatusCode::INTERNAL_SERVER_ERROR,
                            message: format!("{}: {}", error.r#type, error.message),
                            model_metadata: None,
                            retry_after_secs: None,
                            should_retry: None,
                        }
                    };
                    yield SamplingEvent::Failed {
                        request_id: request_id.clone(),
                        error: SamplingErrorInfo::from(&err),
                    };
                    return;
                }
            }

            if event_has_content {
                last_content_chunk_at = Instant::now();
            } else if last_content_chunk_at.elapsed() > idle_timeout {
                let err = SamplingError::IdleTimeout {
                    elapsed_secs: idle_timeout.as_secs(),
                };
                yield SamplingEvent::Failed {
                    request_id: request_id.clone(),
                    error: SamplingErrorInfo::from(&err),
                };
                return;
            }
        }

        if final_stop_reason == Some(StopReason::Length) {
            yield SamplingEvent::Failed {
                request_id: request_id.clone(),
                error: SamplingErrorInfo::from(&SamplingError::MaxTokensTruncation),
            };
            return;
        }

        // ── Build the final response ─────────────────────────────────
        let model_id = final_model.unwrap_or_default();
        // Match the OAI Responses convention: prompt_tokens = full prompt, cached_prompt_tokens = cache hits only.
        let total_prompt_tokens = final_input_tokens
            .saturating_add(final_cache_read_input_tokens)
            .saturating_add(final_cache_creation_input_tokens);
        let usage = if total_prompt_tokens > 0 || final_output_tokens > 0 {
            Some(TokenUsage {
                prompt_tokens: total_prompt_tokens,
                completion_tokens: final_output_tokens,
                total_tokens: total_prompt_tokens.saturating_add(final_output_tokens),
                reasoning_tokens: 0,
                cached_prompt_tokens: final_cache_read_input_tokens,
            })
        } else {
            None
        };

        let stop_reason = if !assistant_tool_calls.is_empty() {
            // Completed tool_use blocks win even over Refusal: the calls are
            // real model output the agent loop must resolve.
            Some(StopReason::ToolCalls)
        } else {
            final_stop_reason
        };

        let assistant_item = ConversationItem::Assistant(AssistantItem {
            content: std::sync::Arc::<str>::from(assistant_text),
            tool_calls: assistant_tool_calls,
            model_id: Some(model_id),
            model_fingerprint: None,
            // The Messages API does not echo the applied reasoning effort.
            reasoning_effort: None,
            external_content: None,
        });

        let mut items: Vec<ConversationItem> = assistant_reasoning
            .into_iter()
            .map(ConversationItem::Reasoning)
            .collect();
        items.push(assistant_item);

        let stream_end = Instant::now();
        let metrics =
            InferenceLatencyStats::from_timestamps(stream_start, &chunk_timestamps, stream_end);

        let response = ConversationResponse {
            items,
            stop_reason,
            usage,
            // Anthropic Messages API carries no cost on the wire.
            cost_usd_ticks: None,
            message_chunks_emitted: message_chunk_count,
            doom_loop_signals: Vec::new(),
            stop_message: final_stop_message,
            provider_end_turn: None,
        };

        yield SamplingEvent::Completed {
            request_id: request_id.clone(),
            response: Box::new(response),
            metrics,
        };
    }
}

#[cfg(test)]
#[path = "messages_tests.rs"]
mod tests;
