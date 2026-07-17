use std::collections::HashMap;

use xai_grok_sampling_types::{Result, rs};

use super::{errors, responses};

/// Stateful compatibility decoder for the deliberately sparse Responses SSE
/// frames emitted by the ChatGPT Codex backend.
///
/// The pinned `async-openai` stream types model the public API's fully
/// populated envelopes. The Codex client protocol instead omits bookkeeping
/// fields such as sequence/output indexes, and its terminal response commonly
/// contains only an id and usage. Keep that tolerance provider-scoped: xAI
/// continues through the strict decoder below.
#[derive(Debug)]
pub(crate) struct CodexSseDecoder {
    fallback_model: String,
    next_sequence_number: u64,
    next_output_index: u32,
    item_output_indexes: HashMap<String, u32>,
    active_output_index: Option<u32>,
    active_item_id: Option<String>,
}

impl CodexSseDecoder {
    pub(crate) fn new(fallback_model: impl Into<String>) -> Self {
        let fallback_model = fallback_model.into();
        Self {
            fallback_model: if fallback_model.is_empty() {
                "openai-codex".to_string()
            } else {
                fallback_model
            },
            next_sequence_number: 0,
            next_output_index: 0,
            item_output_indexes: HashMap::new(),
            active_output_index: None,
            active_item_id: None,
        }
    }

    /// Decode one raw Codex data frame. Unknown non-terminal frames (including
    /// `response.metadata`) are intentionally ignored, matching the official
    /// client's forward-compatible behavior. No provider-controlled value is
    /// included in an error or log message.
    pub(crate) fn decode(&mut self, data: &str) -> Result<Option<rs::ResponseStreamEvent>> {
        let mut value = serde_json::from_str::<serde_json::Value>(data)
            .map_err(|_| errors::invalid_stream_event())?;

        if value
            .get("error")
            .is_some_and(|error| error.is_object() || error.is_string())
        {
            return Err(errors::rejected_stream());
        }

        let kind = value
            .get("type")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(errors::invalid_stream_event)?
            .to_owned();

        if kind == "error" {
            return Err(errors::rejected_stream());
        }
        if kind == "response.failed" {
            return Err(errors::failed_stream());
        }
        if kind == "response.incomplete" {
            return Err(errors::incomplete_stream());
        }

        let supported = matches!(
            kind.as_str(),
            "response.created"
                | "response.in_progress"
                | "response.queued"
                | "response.completed"
                | "response.output_item.added"
                | "response.output_item.done"
                | "response.output_text.delta"
                | "response.output_text.done"
                | "response.function_call_arguments.delta"
                | "response.function_call_arguments.done"
                | "response.reasoning_summary_text.delta"
                | "response.reasoning_summary_text.done"
                | "response.reasoning_text.delta"
                | "response.reasoning_text.done"
                | "response.custom_tool_call_input.delta"
                | "response.custom_tool_call_input.done"
                | "response.web_search_call.in_progress"
                | "response.web_search_call.searching"
                | "response.web_search_call.completed"
                | "response.image_generation_call.in_progress"
                | "response.image_generation_call.generating"
                | "response.image_generation_call.completed"
                | "response.image_generation_call.partial_image"
        );
        if !supported {
            return Ok(None);
        }

        let sequence_number = self.sequence_number(&value);
        let object = value
            .as_object_mut()
            .ok_or_else(errors::invalid_stream_event)?;
        object.insert(
            "sequence_number".to_string(),
            serde_json::Value::from(sequence_number),
        );

        match kind.as_str() {
            "response.created"
            | "response.in_progress"
            | "response.queued"
            | "response.completed" => {
                let status = match kind.as_str() {
                    "response.completed" => "completed",
                    "response.queued" => "queued",
                    _ => "in_progress",
                };
                let response = object
                    .get_mut("response")
                    .ok_or_else(errors::invalid_stream_event)?;
                normalize_codex_response(response, &self.fallback_model, status)?;
            }
            "response.output_item.added" | "response.output_item.done" => {
                let output_index = self.item_output_index(&value);
                let object = value
                    .as_object_mut()
                    .ok_or_else(errors::invalid_stream_event)?;
                object.insert(
                    "output_index".to_string(),
                    serde_json::Value::from(output_index),
                );
                let item = object
                    .get_mut("item")
                    .ok_or_else(errors::invalid_stream_event)?;
                let item_status = if kind == "response.output_item.done" {
                    "completed"
                } else {
                    "in_progress"
                };
                normalize_codex_output_item(item, output_index, item_status)?;
                self.remember_item_aliases(&value, output_index);
                self.active_output_index = Some(output_index);
                self.active_item_id = primary_codex_item_id(&value)
                    .or_else(|| Some(format!("codex-item-{output_index}")));
            }
            "response.output_text.delta" | "response.output_text.done" => {
                let output_index = self.delta_output_index(&value);
                let item_id = self.item_id(&value, output_index);
                let object = value
                    .as_object_mut()
                    .ok_or_else(errors::invalid_stream_event)?;
                insert_u32(object, "output_index", output_index);
                insert_u32_if_missing(object, "content_index", 0);
                insert_string(object, "item_id", item_id);
                if kind == "response.output_text.delta" {
                    insert_string_if_missing(object, "delta", "");
                    insert_null_if_missing(object, "logprobs");
                } else {
                    insert_string_if_missing(object, "text", "");
                    insert_null_if_missing(object, "logprobs");
                }
            }
            "response.function_call_arguments.delta" | "response.function_call_arguments.done" => {
                let output_index = self.delta_output_index(&value);
                let item_id = self.item_id(&value, output_index);
                let object = value
                    .as_object_mut()
                    .ok_or_else(errors::invalid_stream_event)?;
                insert_u32(object, "output_index", output_index);
                insert_string(object, "item_id", item_id);
                if kind == "response.function_call_arguments.delta" {
                    insert_string_if_missing(object, "delta", "");
                } else {
                    insert_string_if_missing(object, "arguments", "");
                }
            }
            "response.reasoning_summary_text.delta" | "response.reasoning_summary_text.done" => {
                let output_index = self.delta_output_index(&value);
                let item_id = self.item_id(&value, output_index);
                let object = value
                    .as_object_mut()
                    .ok_or_else(errors::invalid_stream_event)?;
                insert_u32(object, "output_index", output_index);
                insert_u32_if_missing(object, "summary_index", 0);
                insert_string(object, "item_id", item_id);
                if kind == "response.reasoning_summary_text.delta" {
                    insert_string_if_missing(object, "delta", "");
                } else {
                    insert_string_if_missing(object, "text", "");
                }
            }
            "response.reasoning_text.delta" | "response.reasoning_text.done" => {
                let output_index = self.delta_output_index(&value);
                let item_id = self.item_id(&value, output_index);
                let object = value
                    .as_object_mut()
                    .ok_or_else(errors::invalid_stream_event)?;
                insert_u32(object, "output_index", output_index);
                insert_u32_if_missing(object, "content_index", 0);
                insert_string(object, "item_id", item_id);
                if kind == "response.reasoning_text.delta" {
                    insert_string_if_missing(object, "delta", "");
                } else {
                    insert_string_if_missing(object, "text", "");
                }
            }
            "response.custom_tool_call_input.delta" | "response.custom_tool_call_input.done" => {
                let output_index = self.delta_output_index(&value);
                let item_id = self.item_id(&value, output_index);
                let object = value
                    .as_object_mut()
                    .ok_or_else(errors::invalid_stream_event)?;
                insert_u32(object, "output_index", output_index);
                insert_string(object, "item_id", item_id);
                if kind == "response.custom_tool_call_input.delta" {
                    insert_string_if_missing(object, "delta", "");
                } else {
                    insert_string_if_missing(object, "input", "");
                }
            }
            "response.web_search_call.in_progress"
            | "response.web_search_call.searching"
            | "response.web_search_call.completed"
            | "response.image_generation_call.in_progress"
            | "response.image_generation_call.generating"
            | "response.image_generation_call.completed" => {
                let output_index = self.delta_output_index(&value);
                let item_id = self.item_id(&value, output_index);
                let object = value
                    .as_object_mut()
                    .ok_or_else(errors::invalid_stream_event)?;
                insert_u32(object, "output_index", output_index);
                insert_string(object, "item_id", item_id);
            }
            "response.image_generation_call.partial_image" => {
                let output_index = self.delta_output_index(&value);
                let item_id = self.item_id(&value, output_index);
                let object = value
                    .as_object_mut()
                    .ok_or_else(errors::invalid_stream_event)?;
                insert_u32(object, "output_index", output_index);
                insert_u32_if_missing(object, "partial_image_index", 0);
                insert_string(object, "item_id", item_id);
                insert_string_if_missing(object, "partial_image_b64", "");
            }
            _ => return Ok(None),
        }

        responses::normalize_extended_codex_response_effort(&mut value);
        let mut event = serde_json::from_value::<rs::ResponseStreamEvent>(value)
            .map_err(|_| errors::invalid_stream_event())?;
        crate::client::apply_terminal_event_overrides(&mut event, data);
        Ok(Some(event))
    }

    fn sequence_number(&mut self, value: &serde_json::Value) -> u64 {
        if let Some(sequence_number) = value
            .get("sequence_number")
            .and_then(serde_json::Value::as_u64)
        {
            self.next_sequence_number = self
                .next_sequence_number
                .max(sequence_number.saturating_add(1));
            sequence_number
        } else {
            let sequence_number = self.next_sequence_number;
            self.next_sequence_number = self.next_sequence_number.saturating_add(1);
            sequence_number
        }
    }

    fn item_output_index(&mut self, value: &serde_json::Value) -> u32 {
        let explicit = json_u32(value.get("output_index"));
        let aliases = codex_item_aliases(value);
        let output_index = explicit
            .or_else(|| {
                aliases
                    .iter()
                    .find_map(|alias| self.item_output_indexes.get(alias).copied())
            })
            .unwrap_or_else(|| self.allocate_output_index());
        self.advance_output_index(output_index);
        for alias in aliases {
            self.item_output_indexes.insert(alias, output_index);
        }
        output_index
    }

    fn delta_output_index(&mut self, value: &serde_json::Value) -> u32 {
        let explicit = json_u32(value.get("output_index"));
        let aliases = codex_item_aliases(value);
        let output_index = explicit
            .or_else(|| {
                aliases
                    .iter()
                    .find_map(|alias| self.item_output_indexes.get(alias).copied())
            })
            .or(self.active_output_index)
            .unwrap_or_else(|| self.allocate_output_index());
        self.advance_output_index(output_index);
        for alias in aliases {
            self.item_output_indexes.insert(alias, output_index);
        }
        self.active_output_index = Some(output_index);
        output_index
    }

    fn allocate_output_index(&mut self) -> u32 {
        let output_index = self.next_output_index;
        self.next_output_index = self.next_output_index.saturating_add(1);
        output_index
    }

    fn advance_output_index(&mut self, output_index: u32) {
        self.next_output_index = self.next_output_index.max(output_index.saturating_add(1));
    }

    fn remember_item_aliases(&mut self, value: &serde_json::Value, output_index: u32) {
        for alias in codex_item_aliases(value) {
            self.item_output_indexes.insert(alias, output_index);
        }
    }

    fn item_id(&mut self, value: &serde_json::Value, output_index: u32) -> String {
        let item_id = primary_codex_item_id(value)
            .or_else(|| self.active_item_id.clone())
            .unwrap_or_else(|| format!("codex-item-{output_index}"));
        self.item_output_indexes
            .insert(item_id.clone(), output_index);
        self.active_item_id = Some(item_id.clone());
        item_id
    }
}

fn json_u32(value: Option<&serde_json::Value>) -> Option<u32> {
    value
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
}

fn codex_item_aliases(value: &serde_json::Value) -> Vec<String> {
    [
        value.pointer("/item/id"),
        value.pointer("/item/call_id"),
        value.get("item_id"),
        value.get("call_id"),
    ]
    .into_iter()
    .flatten()
    .filter_map(serde_json::Value::as_str)
    .filter(|value| !value.is_empty())
    .map(str::to_owned)
    .collect()
}

fn primary_codex_item_id(value: &serde_json::Value) -> Option<String> {
    codex_item_aliases(value).into_iter().next()
}

fn insert_u32(object: &mut serde_json::Map<String, serde_json::Value>, key: &str, value: u32) {
    object.insert(key.to_string(), serde_json::Value::from(value));
}

fn insert_u32_if_missing(
    object: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: u32,
) {
    if object.get(key).is_none_or(serde_json::Value::is_null) {
        insert_u32(object, key, value);
    }
}

fn insert_string(
    object: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: String,
) {
    object.insert(key.to_string(), serde_json::Value::String(value));
}

fn insert_string_if_missing(
    object: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: &str,
) {
    if object.get(key).is_none_or(serde_json::Value::is_null) {
        object.insert(
            key.to_string(),
            serde_json::Value::String(value.to_string()),
        );
    }
}

fn insert_null_if_missing(object: &mut serde_json::Map<String, serde_json::Value>, key: &str) {
    if !object.contains_key(key) {
        object.insert(key.to_string(), serde_json::Value::Null);
    }
}

fn normalize_codex_response(
    response: &mut serde_json::Value,
    fallback_model: &str,
    status: &str,
) -> Result<()> {
    let response = response
        .as_object_mut()
        .ok_or_else(errors::invalid_stream_event)?;
    insert_u32_if_missing(response, "created_at", 0);
    insert_string_if_missing(response, "id", "codex-response");
    insert_string_if_missing(response, "model", fallback_model);
    insert_string_if_missing(response, "object", "response");
    insert_string_if_missing(response, "status", status);

    // Never forward arbitrary provider metadata through the compatibility
    // envelope. Preserve only the protocol bit the Grok loop needs, encoded
    // as a string because async-openai models metadata as `Map<String,
    // String>`. Extended reasoning provenance is added separately after this
    // normalization step.
    let end_turn = response
        .remove("end_turn")
        .and_then(|value| value.as_bool());
    response.remove("metadata");
    if let Some(end_turn) = end_turn {
        let mut metadata = serde_json::Map::new();
        metadata.insert(
            responses::CODEX_END_TURN_METADATA_KEY.to_string(),
            serde_json::Value::String(end_turn.to_string()),
        );
        response.insert("metadata".to_string(), serde_json::Value::Object(metadata));
    }

    if response
        .get("output")
        .is_none_or(serde_json::Value::is_null)
    {
        response.insert("output".to_string(), serde_json::Value::Array(Vec::new()));
    }
    let output = response
        .get_mut("output")
        .and_then(serde_json::Value::as_array_mut)
        .ok_or_else(errors::invalid_stream_event)?;
    for (output_index, item) in output.iter_mut().enumerate() {
        let output_index =
            u32::try_from(output_index).map_err(|_| errors::invalid_stream_event())?;
        normalize_codex_output_item(item, output_index, status)?;
    }

    if let Some(usage) = response.get_mut("usage")
        && !usage.is_null()
    {
        normalize_codex_usage(usage)?;
    }
    Ok(())
}

fn normalize_codex_usage(usage: &mut serde_json::Value) -> Result<()> {
    let usage = usage
        .as_object_mut()
        .ok_or_else(errors::invalid_stream_event)?;
    insert_u32_if_missing(usage, "input_tokens", 0);
    insert_u32_if_missing(usage, "output_tokens", 0);
    insert_u32_if_missing(usage, "total_tokens", 0);

    if usage
        .get("input_tokens_details")
        .is_none_or(serde_json::Value::is_null)
    {
        usage.insert(
            "input_tokens_details".to_string(),
            serde_json::json!({"cached_tokens": 0}),
        );
    } else {
        let details = usage
            .get_mut("input_tokens_details")
            .and_then(serde_json::Value::as_object_mut)
            .ok_or_else(errors::invalid_stream_event)?;
        insert_u32_if_missing(details, "cached_tokens", 0);
    }

    if usage
        .get("output_tokens_details")
        .is_none_or(serde_json::Value::is_null)
    {
        usage.insert(
            "output_tokens_details".to_string(),
            serde_json::json!({"reasoning_tokens": 0}),
        );
    } else {
        let details = usage
            .get_mut("output_tokens_details")
            .and_then(serde_json::Value::as_object_mut)
            .ok_or_else(errors::invalid_stream_event)?;
        insert_u32_if_missing(details, "reasoning_tokens", 0);
    }
    Ok(())
}

fn normalize_codex_output_item(
    item: &mut serde_json::Value,
    output_index: u32,
    status: &str,
) -> Result<()> {
    let item = item
        .as_object_mut()
        .ok_or_else(errors::invalid_stream_event)?;
    let kind = item
        .get("type")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(errors::invalid_stream_event)?
        .to_owned();
    let item_id = format!("codex-item-{output_index}");

    match kind.as_str() {
        "message" => {
            insert_string_if_missing(item, "id", &item_id);
            insert_string_if_missing(item, "role", "assistant");
            insert_string_if_missing(item, "status", status);
            if item.get("content").is_none_or(serde_json::Value::is_null) {
                item.insert("content".to_string(), serde_json::Value::Array(Vec::new()));
            }
            let content = item
                .get_mut("content")
                .and_then(serde_json::Value::as_array_mut)
                .ok_or_else(errors::invalid_stream_event)?;
            for part in content {
                if part.get("type").and_then(serde_json::Value::as_str) == Some("output_text") {
                    let part = part
                        .as_object_mut()
                        .ok_or_else(errors::invalid_stream_event)?;
                    if part
                        .get("annotations")
                        .is_none_or(serde_json::Value::is_null)
                    {
                        part.insert(
                            "annotations".to_string(),
                            serde_json::Value::Array(Vec::new()),
                        );
                    }
                    insert_null_if_missing(part, "logprobs");
                    insert_string_if_missing(part, "text", "");
                }
            }
        }
        "reasoning" => {
            insert_string_if_missing(item, "id", &item_id);
            if item.get("summary").is_none_or(serde_json::Value::is_null) {
                item.insert("summary".to_string(), serde_json::Value::Array(Vec::new()));
            }
        }
        "function_call" => {
            insert_string_if_missing(item, "call_id", &format!("codex-call-{output_index}"));
            insert_string_if_missing(item, "name", "");
            insert_string_if_missing(item, "arguments", "");
        }
        "custom_tool_call" => {
            insert_string_if_missing(item, "id", &item_id);
            insert_string_if_missing(item, "call_id", &format!("codex-call-{output_index}"));
            insert_string_if_missing(item, "name", "");
            insert_string_if_missing(item, "input", "");
        }
        "image_generation_call" => {
            insert_string_if_missing(item, "id", &item_id);
            insert_string_if_missing(item, "status", status);
            insert_null_if_missing(item, "result");
        }
        "web_search_call" => {
            insert_string_if_missing(item, "id", &item_id);
            insert_string_if_missing(item, "status", status);
            if item.get("action").is_none_or(serde_json::Value::is_null) {
                item.insert(
                    "action".to_string(),
                    serde_json::json!({"type": "search", "query": "", "sources": null}),
                );
            }
        }
        "file_search_call" => {
            insert_string_if_missing(item, "id", &item_id);
            insert_string_if_missing(item, "status", status);
            if item.get("queries").is_none_or(serde_json::Value::is_null) {
                item.insert("queries".to_string(), serde_json::Value::Array(Vec::new()));
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_malformed_sse_diagnostics_do_not_reflect_payload() {
        let reflected = "codex-access-token account-selected-by-auth-store";
        let data = format!(r#"{{"type":"response.created","response":"{reflected}"}}"#);
        let error = match CodexSseDecoder::new("gpt-5.6").decode(&data) {
            Err(error) => error,
            Ok(_) => panic!("malformed provider stream event was unexpectedly accepted"),
        };
        let rendered = error.to_string();
        if rendered != "serialization error: ChatGPT Codex stream event was invalid" {
            panic!("provider stream parse diagnostics exposed unexpected detail");
        }
        assert!(!rendered.contains(reflected));
    }

    #[test]
    fn codex_sparse_official_wire_fixtures_decode_statefully() {
        fn decode(
            decoder: &mut CodexSseDecoder,
            value: serde_json::Value,
        ) -> rs::ResponseStreamEvent {
            decoder
                .decode(&value.to_string())
                .expect("fixture should decode")
                .expect("fixture should produce an event")
        }

        let mut decoder = CodexSseDecoder::new("gpt-5.6");

        let event = decode(
            &mut decoder,
            serde_json::json!({
                "type": "response.created",
                "response": {"id": "resp_1"}
            }),
        );
        let rs::ResponseStreamEvent::ResponseCreated(created) = event else {
            panic!("expected response.created");
        };
        assert_eq!(created.sequence_number, 0);
        assert_eq!(created.response.id, "resp_1");
        assert_eq!(created.response.model, "gpt-5.6");
        assert_eq!(created.response.status, rs::Status::InProgress);

        let event = decode(
            &mut decoder,
            serde_json::json!({
                "type": "response.output_item.added",
                "item": {"type": "reasoning", "id": "reason_1", "summary": []}
            }),
        );
        let rs::ResponseStreamEvent::ResponseOutputItemAdded(reasoning) = event else {
            panic!("expected reasoning output_item.added");
        };
        assert_eq!(reasoning.sequence_number, 1);
        assert_eq!(reasoning.output_index, 0);

        let event = decode(
            &mut decoder,
            serde_json::json!({
                "type": "response.reasoning_summary_text.delta",
                "delta": "summary ",
                "summary_index": 0
            }),
        );
        let rs::ResponseStreamEvent::ResponseReasoningSummaryTextDelta(summary) = event else {
            panic!("expected reasoning summary delta");
        };
        assert_eq!(summary.output_index, 0);
        assert_eq!(summary.item_id, "reason_1");
        assert_eq!(summary.delta, "summary ");

        let event = decode(
            &mut decoder,
            serde_json::json!({
                "type": "response.reasoning_summary_text.done",
                "text": "summary complete",
                "summary_index": 0
            }),
        );
        let rs::ResponseStreamEvent::ResponseReasoningSummaryTextDone(summary) = event else {
            panic!("expected reasoning summary done");
        };
        assert_eq!(summary.output_index, 0);
        assert_eq!(summary.item_id, "reason_1");

        let event = decode(
            &mut decoder,
            serde_json::json!({
                "type": "response.reasoning_text.delta",
                "delta": "private reasoning",
                "content_index": 0
            }),
        );
        let rs::ResponseStreamEvent::ResponseReasoningTextDelta(reasoning) = event else {
            panic!("expected reasoning text delta");
        };
        assert_eq!(reasoning.output_index, 0);
        assert_eq!(reasoning.item_id, "reason_1");

        let event = decode(
            &mut decoder,
            serde_json::json!({
                "type": "response.output_item.done",
                "item": {
                    "type": "function_call",
                    "call_id": "call_1",
                    "name": "shell",
                    "arguments": "{}"
                }
            }),
        );
        let rs::ResponseStreamEvent::ResponseOutputItemDone(first_call) = event else {
            panic!("expected first function output_item.done");
        };
        assert_eq!(first_call.output_index, 1);

        let event = decode(
            &mut decoder,
            serde_json::json!({
                "type": "response.output_item.done",
                "item": {
                    "type": "function_call",
                    "call_id": "call_2",
                    "name": "read_file",
                    "arguments": "{\"path\":\"README.md\"}"
                }
            }),
        );
        let rs::ResponseStreamEvent::ResponseOutputItemDone(second_call) = event else {
            panic!("expected second function output_item.done");
        };
        assert_eq!(second_call.output_index, 2);

        let event = decode(
            &mut decoder,
            serde_json::json!({
                "type": "response.function_call_arguments.done",
                "item_id": "call_2",
                "arguments": "{\"path\":\"README.md\"}"
            }),
        );
        let rs::ResponseStreamEvent::ResponseFunctionCallArgumentsDone(arguments) = event else {
            panic!("expected function arguments done");
        };
        assert_eq!(arguments.output_index, 2);
        assert_eq!(arguments.item_id, "call_2");

        let event = decode(
            &mut decoder,
            serde_json::json!({
                "type": "response.output_item.added",
                "item": {
                    "type": "message",
                    "role": "assistant",
                    "id": "msg_1",
                    "content": []
                }
            }),
        );
        let rs::ResponseStreamEvent::ResponseOutputItemAdded(message_added) = event else {
            panic!("expected message output_item.added");
        };
        assert_eq!(message_added.output_index, 3);

        let event = decode(
            &mut decoder,
            serde_json::json!({
                "type": "response.output_text.delta",
                "delta": "hello"
            }),
        );
        let rs::ResponseStreamEvent::ResponseOutputTextDelta(text) = event else {
            panic!("expected output text delta");
        };
        assert_eq!(text.output_index, 3);
        assert_eq!(text.item_id, "msg_1");
        assert_eq!(text.content_index, 0);
        assert_eq!(text.logprobs, None);

        let event = decode(
            &mut decoder,
            serde_json::json!({
                "type": "response.output_item.done",
                "item": {
                    "type": "message",
                    "role": "assistant",
                    "id": "msg_1",
                    "content": [{"type": "output_text", "text": "hello"}]
                }
            }),
        );
        let rs::ResponseStreamEvent::ResponseOutputItemDone(message_done) = event else {
            panic!("expected message output_item.done");
        };
        assert_eq!(message_done.output_index, 3);
        let rs::OutputItem::Message(message) = message_done.item else {
            panic!("expected normalized message");
        };
        assert_eq!(message.status, rs::OutputStatus::Completed);
        let rs::OutputMessageContent::OutputText(text) = &message.content[0] else {
            panic!("expected output text content");
        };
        assert!(text.annotations.is_empty());
        assert_eq!(text.logprobs, None);

        let event = decode(
            &mut decoder,
            serde_json::json!({
                "type": "response.completed",
                "response": {
                    "id": "resp_1",
                    "end_turn": false,
                    "metadata": {"provider_secret": "must-not-survive"},
                    "usage": {
                        "input_tokens": 7,
                        "input_tokens_details": null,
                        "output_tokens": 3,
                        "output_tokens_details": null,
                        "total_tokens": 10
                    }
                }
            }),
        );
        let rs::ResponseStreamEvent::ResponseCompleted(completed) = event else {
            panic!("expected sparse response.completed");
        };
        assert_eq!(completed.response.model, "gpt-5.6");
        assert_eq!(completed.response.status, rs::Status::Completed);
        assert!(completed.response.output.is_empty());
        let usage = completed.response.usage.expect("usage");
        assert_eq!(usage.input_tokens_details.cached_tokens, 0);
        assert_eq!(usage.output_tokens_details.reasoning_tokens, 0);
        let metadata = completed.response.metadata.expect("end_turn metadata");
        assert_eq!(
            metadata
                .get(responses::CODEX_END_TURN_METADATA_KEY)
                .map(String::as_str),
            Some("false")
        );
        assert!(!metadata.contains_key("provider_secret"));
    }

    #[test]
    fn codex_decoder_ignores_unknown_nonterminal_metadata() {
        let reflected = "codex-access-token account-selected-by-auth-store";
        let mut decoder = CodexSseDecoder::new("gpt-5.6");
        let data =
            format!(r#"{{"type":"response.metadata","metadata":{{"reflected":"{reflected}"}}}}"#);
        assert!(
            decoder
                .decode(&data)
                .expect("metadata is ignored")
                .is_none()
        );

        let data = format!(r#"{{"type":"response.future_event","value":"{reflected}"}}"#);
        assert!(decoder.decode(&data).expect("unknown is ignored").is_none());
    }

    #[test]
    fn codex_decoder_preserves_both_end_turn_values_without_provider_metadata() {
        for (end_turn, expected) in [(true, "true"), (false, "false")] {
            let mut decoder = CodexSseDecoder::new("gpt-5.6");
            let event = decoder
                .decode(
                    &serde_json::json!({
                        "type": "response.completed",
                        "response": {
                            "id": "resp_1",
                            "end_turn": end_turn,
                            "metadata": {"provider_value": "must-not-survive"}
                        }
                    })
                    .to_string(),
                )
                .expect("terminal fixture should decode")
                .expect("terminal fixture should produce an event");
            let rs::ResponseStreamEvent::ResponseCompleted(completed) = event else {
                panic!("expected response.completed");
            };
            let metadata = completed.response.metadata.expect("private metadata");
            assert_eq!(
                metadata
                    .get(responses::CODEX_END_TURN_METADATA_KEY)
                    .map(String::as_str),
                Some(expected)
            );
            assert!(!metadata.contains_key("provider_value"));
        }
    }

    #[test]
    fn codex_decoder_failures_never_reflect_provider_payload() {
        let reflected = "codex-access-token account-selected-by-auth-store";
        let mut decoder = CodexSseDecoder::new("gpt-5.6");

        for data in [
            format!(r#"{{"type":"response.output_text.delta","delta":{{"value":"{reflected}"}}}}"#),
            format!(
                r#"{{"type":"response.failed","response":{{"error":{{"message":"{reflected}"}}}}}}"#
            ),
            format!(r#"{{"type":"error","message":"{reflected}"}}"#),
            format!(r#"{{"error":{{"message":"{reflected}"}}}}"#),
        ] {
            let error = match decoder.decode(&data) {
                Err(error) => error,
                Ok(_) => panic!("invalid provider stream payload was unexpectedly accepted"),
            };
            let rendered = error.to_string();
            assert!(!rendered.contains(reflected));
        }
    }

    #[test]
    fn extended_effort_metadata_survives_sparse_terminal_decode() {
        for effort in ["max", "ultra"] {
            let data = serde_json::json!({
                "type": "response.completed",
                "response": {
                    "id": "resp_1",
                    "reasoning": {"effort": effort}
                }
            })
            .to_string();
            let event = CodexSseDecoder::new("gpt-5.6")
                .decode(&data)
                .unwrap()
                .unwrap();
            let rs::ResponseStreamEvent::ResponseCompleted(event) = event else {
                panic!("expected response.completed");
            };
            assert_eq!(
                event
                    .response
                    .reasoning
                    .as_ref()
                    .and_then(|reasoning| reasoning.effort.clone()),
                Some(rs::ReasoningEffort::Xhigh)
            );
            assert_eq!(
                event
                    .response
                    .metadata
                    .as_ref()
                    .and_then(|metadata| {
                        metadata.get(
                        xai_grok_sampling_types::OPENAI_CODEX_EXTENDED_REASONING_EFFORT_METADATA_KEY
                    )
                    })
                    .map(String::as_str),
                Some(effort)
            );
        }
    }
}
