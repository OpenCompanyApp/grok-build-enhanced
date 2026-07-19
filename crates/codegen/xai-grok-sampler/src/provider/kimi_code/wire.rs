//! Kimi wire compatibility that is independent of authentication and endpoint
//! policy: tool-call identifiers and proprietary usage placement.

use std::collections::{HashMap, HashSet};

use serde_json::Value;
use xai_grok_sampling_types::{ChatCompletionChunk, ChatCompletionResponse, Result, SamplingError};

const MAX_TOOL_CALL_ID_BYTES: usize = 64;
const EMPTY_TOOL_CALL_ID: &str = "tool_call";

pub(super) fn normalize_chat_history(body: &mut Value, preserved_thinking: bool) -> Result<()> {
    let Some(messages) = body.get_mut("messages") else {
        return Ok(());
    };
    let messages = messages
        .as_array_mut()
        .ok_or(SamplingError::InvalidConfiguration(
            "Kimi Code messages must be a JSON array",
        ))?;

    for message in messages {
        let Some(object) = message.as_object_mut() else {
            return Err(SamplingError::InvalidConfiguration(
                "Kimi Code messages must contain JSON objects",
            ));
        };

        // Grok keeps the generating model on assistant history for local
        // accounting. It is not part of Kimi's Chat Completions contract.
        object.remove("model_id");

        if object.get("role").and_then(Value::as_str) != Some("assistant") {
            continue;
        }

        if preserved_thinking && !object.contains_key("reasoning_content") {
            // `thinking.keep: all` requires every prior assistant turn to carry
            // the proprietary field, including turns that reasoned to an empty
            // string. This mirrors the official Kimi adapter's round trip.
            object.insert("reasoning_content".to_owned(), Value::String(String::new()));
        }

        if object
            .get("tool_calls")
            .and_then(Value::as_array)
            .is_some_and(|calls| !calls.is_empty())
            && object
                .get("content")
                .is_some_and(is_effectively_empty_content)
        {
            // Kimi rejects an empty text content field next to assistant tool
            // calls, while omitting the field is accepted by the API.
            object.remove("content");
        }
    }
    Ok(())
}

fn is_effectively_empty_content(content: &Value) -> bool {
    match content {
        Value::String(text) => text.trim().is_empty(),
        Value::Array(parts) => parts.iter().all(|part| {
            part.get("type").and_then(Value::as_str) == Some("text")
                && part
                    .get("text")
                    .and_then(Value::as_str)
                    .is_some_and(|text| text.trim().is_empty())
        }),
        _ => false,
    }
}

/// Collapse only the adjacent user turns that a strict Messages-compatible
/// endpoint can merge without changing tool-result ordering semantics.
pub(super) fn normalize_messages_history(body: &mut Value) -> Result<()> {
    let Some(messages) = body.get_mut("messages") else {
        return Ok(());
    };
    let messages = messages
        .as_array_mut()
        .ok_or(SamplingError::InvalidConfiguration(
            "Kimi Code messages must be a JSON array",
        ))?;

    let mut normalized = Vec::with_capacity(messages.len());
    for message in std::mem::take(messages) {
        if !message.is_object() {
            return Err(SamplingError::InvalidConfiguration(
                "Kimi Code messages must contain JSON objects",
            ));
        }
        let merge = normalized.last().is_some_and(|last| {
            is_user_message(last)
                && is_user_message(&message)
                && (is_tool_result_only(last) || !is_tool_result_only(&message))
        });
        if merge {
            let last = normalized
                .last_mut()
                .expect("merge decision requires a previous message");
            merge_message_content(last, message)?;
        } else {
            normalized.push(message);
        }
    }
    *messages = normalized;
    Ok(())
}

fn is_user_message(message: &Value) -> bool {
    message.get("role").and_then(Value::as_str) == Some("user")
}

fn is_tool_result_only(message: &Value) -> bool {
    message
        .get("content")
        .and_then(Value::as_array)
        .is_some_and(|content| {
            !content.is_empty()
                && content
                    .iter()
                    .all(|block| block.get("type").and_then(Value::as_str) == Some("tool_result"))
        })
}

fn content_blocks(content: Value) -> Result<Vec<Value>> {
    match content {
        Value::Array(blocks) => Ok(blocks),
        Value::String(text) => Ok(vec![serde_json::json!({
            "type": "text",
            "text": text,
        })]),
        _ => Err(SamplingError::InvalidConfiguration(
            "Kimi Code Messages content must be text or a JSON array",
        )),
    }
}

fn merge_message_content(last: &mut Value, mut next: Value) -> Result<()> {
    let last_object = last
        .as_object_mut()
        .ok_or(SamplingError::InvalidConfiguration(
            "Kimi Code messages must contain JSON objects",
        ))?;
    let last_content = last_object
        .remove("content")
        .ok_or(SamplingError::InvalidConfiguration(
            "Kimi Code Messages user message is missing content",
        ))?;
    let next_content = next
        .as_object_mut()
        .and_then(|object| object.remove("content"))
        .ok_or(SamplingError::InvalidConfiguration(
            "Kimi Code Messages user message is missing content",
        ))?;
    let mut blocks = content_blocks(last_content)?;
    blocks.extend(content_blocks(next_content)?);
    last_object.insert("content".to_owned(), Value::Array(blocks));
    Ok(())
}

pub(super) fn inject_messages_cache_breakpoints(body: &mut Value) -> Result<()> {
    let Some(object) = body.as_object_mut() else {
        return Err(SamplingError::InvalidConfiguration(
            "Kimi Code Messages body must be an object",
        ));
    };

    if let Some(system) = object.get_mut("system") {
        match system {
            Value::String(text) => {
                *system = serde_json::json!([{
                    "type": "text",
                    "text": std::mem::take(text),
                    "cache_control": {"type": "ephemeral"}
                }]);
            }
            Value::Array(blocks) => {
                if let Some(last_block) = blocks.last_mut() {
                    if last_block.get("type").and_then(Value::as_str) != Some("text") {
                        return Err(SamplingError::InvalidConfiguration(
                            "Kimi Code Messages system blocks must be text",
                        ));
                    }
                    let block =
                        last_block
                            .as_object_mut()
                            .ok_or(SamplingError::InvalidConfiguration(
                                "Kimi Code Messages system blocks must be JSON objects",
                            ))?;
                    block.insert(
                        "cache_control".to_owned(),
                        serde_json::json!({"type": "ephemeral"}),
                    );
                }
            }
            _ => {
                return Err(SamplingError::InvalidConfiguration(
                    "Kimi Code Messages system must be text or a JSON array",
                ));
            }
        }
    }

    if let Some(last_block) = object
        .get_mut("messages")
        .and_then(Value::as_array_mut)
        .and_then(|messages| messages.last_mut())
        .and_then(|message| message.get_mut("content"))
        .and_then(Value::as_array_mut)
        .and_then(|content| content.last_mut())
        && is_cacheable_messages_block(last_block)
    {
        let block = last_block
            .as_object_mut()
            .ok_or(SamplingError::InvalidConfiguration(
                "Kimi Code Messages content blocks must be JSON objects",
            ))?;
        block.insert(
            "cache_control".to_owned(),
            serde_json::json!({"type": "ephemeral"}),
        );
    }

    if let Some(last_tool) = object
        .get_mut("tools")
        .and_then(Value::as_array_mut)
        .and_then(|tools| tools.last_mut())
    {
        let tool = last_tool
            .as_object_mut()
            .ok_or(SamplingError::InvalidConfiguration(
                "Kimi Code Messages tools must contain JSON objects",
            ))?;
        tool.insert(
            "cache_control".to_owned(),
            serde_json::json!({"type": "ephemeral"}),
        );
    }
    Ok(())
}

fn is_cacheable_messages_block(block: &Value) -> bool {
    matches!(
        block.get("type").and_then(Value::as_str),
        Some(
            "text"
                | "image"
                | "document"
                | "search_result"
                | "tool_use"
                | "tool_result"
                | "server_tool_use"
                | "web_search_tool_result"
        )
    )
}

pub(super) fn normalize_tool_call_ids(body: &mut Value, backend: &'static str) -> Result<()> {
    let Some(messages) = body.get_mut("messages") else {
        return Ok(());
    };
    let messages = messages
        .as_array_mut()
        .ok_or(SamplingError::InvalidConfiguration(
            "Kimi Code messages must be a JSON array",
        ))?;

    let mut raw_ids = Vec::new();
    let mut seen = HashSet::new();
    for message in messages.iter() {
        match backend {
            "chat_completions" => collect_chat_ids(message, &mut raw_ids, &mut seen),
            "messages" => collect_messages_ids(message, &mut raw_ids, &mut seen),
            _ => {}
        }
    }
    if raw_ids.is_empty() {
        return Ok(());
    }

    let mapping = build_id_mapping(&raw_ids);
    for message in messages {
        match backend {
            "chat_completions" => apply_chat_id_mapping(message, &mapping),
            "messages" => apply_messages_id_mapping(message, &mapping),
            _ => {}
        }
    }
    Ok(())
}

fn collect_id<'a>(id: &'a str, raw_ids: &mut Vec<&'a str>, seen: &mut HashSet<&'a str>) {
    if seen.insert(id) {
        raw_ids.push(id);
    }
}

fn collect_chat_ids<'a>(
    message: &'a Value,
    raw_ids: &mut Vec<&'a str>,
    seen: &mut HashSet<&'a str>,
) {
    if let Some(tool_calls) = message.get("tool_calls").and_then(Value::as_array) {
        for tool_call in tool_calls {
            if let Some(id) = tool_call.get("id").and_then(Value::as_str) {
                collect_id(id, raw_ids, seen);
            }
        }
    }
    if let Some(id) = message.get("tool_call_id").and_then(Value::as_str) {
        collect_id(id, raw_ids, seen);
    }
}

fn collect_messages_ids<'a>(
    message: &'a Value,
    raw_ids: &mut Vec<&'a str>,
    seen: &mut HashSet<&'a str>,
) {
    let Some(content) = message.get("content").and_then(Value::as_array) else {
        return;
    };
    for block in content {
        let key = match block.get("type").and_then(Value::as_str) {
            Some("tool_use") => "id",
            Some("tool_result") => "tool_use_id",
            _ => continue,
        };
        if let Some(id) = block.get(key).and_then(Value::as_str) {
            collect_id(id, raw_ids, seen);
        }
    }
}

fn build_id_mapping(raw_ids: &[&str]) -> HashMap<String, String> {
    let mut mapping = HashMap::with_capacity(raw_ids.len());
    let mut used = HashSet::with_capacity(raw_ids.len());

    // Already-valid identifiers win collisions so provider-generated IDs stay
    // unchanged whenever possible.
    for raw in raw_ids {
        if is_valid_id(raw) {
            mapping.insert((*raw).to_owned(), (*raw).to_owned());
            used.insert((*raw).to_owned());
        }
    }

    for raw in raw_ids {
        if mapping.contains_key(*raw) {
            continue;
        }
        let normalized = sanitize_id(raw);
        let unique = unique_id(&normalized, &used);
        used.insert(unique.clone());
        mapping.insert((*raw).to_owned(), unique);
    }
    mapping
}

fn is_valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= MAX_TOOL_CALL_ID_BYTES
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn sanitize_id(id: &str) -> String {
    let sanitized: String = id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '_' | '-') {
                character
            } else {
                '_'
            }
        })
        .take(MAX_TOOL_CALL_ID_BYTES)
        .collect();
    if sanitized.is_empty() {
        EMPTY_TOOL_CALL_ID.to_owned()
    } else {
        sanitized
    }
}

fn unique_id(base: &str, used: &HashSet<String>) -> String {
    let candidate = truncate_with_suffix(base, "");
    if !used.contains(&candidate) {
        return candidate;
    }
    for index in 2_u64.. {
        let suffix = format!("_{index}");
        let candidate = truncate_with_suffix(base, &suffix);
        if !used.contains(&candidate) {
            return candidate;
        }
    }
    unreachable!("the numeric suffix space is effectively unbounded")
}

fn truncate_with_suffix(base: &str, suffix: &str) -> String {
    let keep = MAX_TOOL_CALL_ID_BYTES.saturating_sub(suffix.len());
    let mut output: String = base.chars().take(keep).collect();
    output.push_str(suffix);
    output
}

fn apply_chat_id_mapping(message: &mut Value, mapping: &HashMap<String, String>) {
    if let Some(tool_calls) = message.get_mut("tool_calls").and_then(Value::as_array_mut) {
        for tool_call in tool_calls {
            replace_id(tool_call, "id", mapping);
        }
    }
    replace_id(message, "tool_call_id", mapping);
}

fn apply_messages_id_mapping(message: &mut Value, mapping: &HashMap<String, String>) {
    let Some(content) = message.get_mut("content").and_then(Value::as_array_mut) else {
        return;
    };
    for block in content {
        match block.get("type").and_then(Value::as_str) {
            Some("tool_use") => replace_id(block, "id", mapping),
            Some("tool_result") => replace_id(block, "tool_use_id", mapping),
            _ => {}
        }
    }
}

fn replace_id(value: &mut Value, key: &str, mapping: &HashMap<String, String>) {
    let Some(raw) = value.get(key).and_then(Value::as_str) else {
        return;
    };
    let Some(normalized) = mapping.get(raw) else {
        return;
    };
    value[key] = Value::String(normalized.clone());
}

pub(crate) fn stream_error(data: &str) -> Option<SamplingError> {
    let value = serde_json::from_str::<Value>(data).ok()?;
    let error = value.get("error")?;
    if !error.is_object() && !error.is_string() {
        return None;
    }
    let error_type = error
        .get("type")
        .or_else(|| error.get("code"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let message = error
        .get("message")
        .and_then(Value::as_str)
        .or_else(|| error.as_str())
        .unwrap_or_default();
    Some(classify_stream_error(error_type, message))
}

pub(crate) fn classify_stream_error(error_type: &str, message: &str) -> SamplingError {
    let classification = format!("{error_type} {message}").to_ascii_lowercase();
    let (status, should_retry) = if classification.contains("api key")
        || classification.contains("authentication")
        || classification.contains("unauthorized")
    {
        (reqwest::StatusCode::UNAUTHORIZED, Some(false))
    } else if classification.contains("usage limit")
        || classification.contains("quota")
        || classification.contains("billing cycle")
        || classification.contains("monthly limit")
        || classification.contains("five-hour limit")
        || classification.contains("5-hour limit")
    {
        (reqwest::StatusCode::TOO_MANY_REQUESTS, Some(false))
    } else if classification.contains("does not have access")
        || classification.contains("permission denied")
        || classification.contains("forbidden")
        || classification.contains("membership")
    {
        (reqwest::StatusCode::FORBIDDEN, Some(false))
    } else if classification.contains("context")
        || classification.contains("token limit")
        || classification.contains("message size")
        || classification.contains("validation")
        || classification.contains("invalid_request")
        || classification.contains("reasoning_content is missing")
    {
        (reqwest::StatusCode::BAD_REQUEST, Some(false))
    } else if classification.contains("overload")
        || classification.contains("too many requests")
        || classification.contains("concurrency limit")
    {
        (
            reqwest::StatusCode::from_u16(529).expect("529 is a valid extension status code"),
            Some(true),
        )
    } else {
        (reqwest::StatusCode::INTERNAL_SERVER_ERROR, None)
    };

    SamplingError::Api {
        status,
        message: super::canonical_error_message(message),
        model_metadata: None,
        retry_after_secs: None,
        should_retry,
    }
}

pub(crate) fn deserialize_chat_response(bytes: &[u8]) -> Result<ChatCompletionResponse> {
    let mut value: Value = serde_json::from_slice(bytes).map_err(SamplingError::Serialization)?;
    normalize_usage(&mut value);
    normalize_response_tool_call_ids(&mut value);
    serde_json::from_value(value).map_err(SamplingError::Serialization)
}

fn normalize_response_tool_call_ids(value: &mut Value) {
    let Some(choices) = value.get_mut("choices").and_then(Value::as_array_mut) else {
        return;
    };
    for tool_call in choices
        .iter_mut()
        .filter_map(|choice| choice.pointer_mut("/message/tool_calls"))
        .filter_map(Value::as_array_mut)
        .flatten()
    {
        let Some(tool_call) = tool_call.as_object_mut() else {
            continue;
        };
        if tool_call
            .get("id")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        {
            tool_call.insert(
                "id".to_owned(),
                Value::String(uuid::Uuid::new_v4().to_string()),
            );
        }
    }
}

pub(crate) fn deserialize_chat_chunk(data: &str) -> Result<ChatCompletionChunk> {
    let mut value: Value = serde_json::from_str(data).map_err(SamplingError::Serialization)?;
    normalize_usage(&mut value);
    serde_json::from_value(value).map_err(SamplingError::Serialization)
}

fn normalize_usage(value: &mut Value) {
    let nested_usage = value.pointer("/choices/0/usage").cloned();
    let Some(object) = value.as_object_mut() else {
        return;
    };
    if object.get("usage").is_none_or(Value::is_null)
        && let Some(nested_usage) = nested_usage.filter(Value::is_object)
    {
        object.insert("usage".to_owned(), nested_usage);
    }

    let Some(usage) = object.get_mut("usage").and_then(Value::as_object_mut) else {
        return;
    };
    let Some(cached_tokens) = usage.get("cached_tokens").cloned().filter(Value::is_number) else {
        return;
    };
    let details = usage
        .entry("prompt_tokens_details")
        .or_insert_with(|| Value::Object(Default::default()));
    if !details.is_object() {
        *details = Value::Object(Default::default());
    }
    details
        .as_object_mut()
        .expect("prompt token details replaced with an object")
        .insert("cached_tokens".to_owned(), cached_tokens);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_history_ids_are_bounded_and_tool_results_keep_the_same_mapping() {
        let colliding_valid = format!("{}X", "a".repeat(63));
        let raw = format!("{}?", "a".repeat(63));
        let mut body = serde_json::json!({
            "messages": [
                {"role": "assistant", "tool_calls": [
                    {"id": colliding_valid, "type": "function", "function": {"name": "one", "arguments": "{}"}},
                    {"id": raw, "type": "function", "function": {"name": "two", "arguments": "{}"}}
                ]},
                {"role": "tool", "tool_call_id": raw, "content": "done"}
            ]
        });

        normalize_tool_call_ids(&mut body, "chat_completions").unwrap();

        let first = body["messages"][0]["tool_calls"][0]["id"].as_str().unwrap();
        let second = body["messages"][0]["tool_calls"][1]["id"].as_str().unwrap();
        let result = body["messages"][1]["tool_call_id"].as_str().unwrap();
        assert_eq!(first, format!("{}X", "a".repeat(63)));
        assert_eq!(second.len(), 64);
        assert_ne!(second, first);
        assert_eq!(result, second);
    }

    #[test]
    fn messages_history_uses_one_safe_id_for_tool_use_and_result_blocks() {
        let mut body = serde_json::json!({
            "messages": [
                {"role": "assistant", "content": [{"type": "tool_use", "id": "call|unsafe", "name": "read", "input": {}}]},
                {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "call|unsafe", "content": "done"}]}
            ]
        });

        normalize_tool_call_ids(&mut body, "messages").unwrap();

        assert_eq!(body["messages"][0]["content"][0]["id"], "call_unsafe");
        assert_eq!(
            body["messages"][1]["content"][0]["tool_use_id"],
            "call_unsafe"
        );
    }

    #[test]
    fn quota_stream_error_is_canonical_and_non_retryable() {
        let error = stream_error(
            r#"{"error":{"type":"usage_limit_reached","message":"monthly usage limit for private account detail"}}"#,
        )
        .expect("error event must be recognized");

        match error {
            SamplingError::Api {
                status,
                message,
                should_retry,
                ..
            } => {
                assert_eq!(status, reqwest::StatusCode::TOO_MANY_REQUESTS);
                assert_eq!(should_retry, Some(false));
                assert_eq!(
                    message,
                    "Kimi Code usage quota is exhausted; check the Kimi Code Console for reset details"
                );
                assert!(!message.contains("private"));
            }
            other => panic!("expected canonical API error, got {other:?}"),
        }
    }

    #[test]
    fn unknown_chat_finish_reason_survives_deserialization() {
        let chunk = deserialize_chat_chunk(
            r#"{"id":"chunk","object":"chat.completion.chunk","created":1,"model":"k3","choices":[{"index":0,"delta":{},"finish_reason":"future_reason"}]}"#,
        )
        .unwrap();

        assert!(matches!(
            chunk.choices[0].finish_reason.as_ref(),
            Some(xai_grok_sampling_types::FinishReason::Unknown(reason))
                if reason == "future_reason"
        ));
    }

    #[test]
    fn nonstream_tool_call_without_an_id_gets_a_generated_id() {
        let response = deserialize_chat_response(
            br#"{"id":"response","object":"chat.completion","created":1,"model":"k3","choices":[{"index":0,"message":{"role":"assistant","content":null,"tool_calls":[{"type":"function","function":{"name":"read_file","arguments":"{}"}}]},"finish_reason":"tool_calls"}]}"#,
        )
        .unwrap();

        let id = &response.choices[0].message.tool_calls[0].id;
        assert!(uuid::Uuid::parse_str(id).is_ok());
    }

    #[test]
    fn proprietary_choice_usage_and_top_level_cache_tokens_are_projected() {
        let chunk = deserialize_chat_chunk(
            r#"{"id":"chunk","object":"chat.completion.chunk","created":1,"model":"k3","choices":[{"index":0,"delta":{},"finish_reason":null,"usage":{"prompt_tokens":20,"completion_tokens":3,"total_tokens":23,"cached_tokens":12}}]}"#,
        )
        .unwrap();

        let usage = chunk.usage.unwrap();
        assert_eq!(usage.prompt_tokens, 20);
        assert_eq!(usage.completion_tokens, 3);
        assert_eq!(usage.prompt_tokens_details.unwrap().cached_tokens, 12);
    }
}
