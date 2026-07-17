mod openai_codex;
mod responses;

use crate::types::output::WebSearchReference;

pub(super) use openai_codex::OpenAiCodexBackend;
pub(super) use responses::ResponsesBackend;

const MAX_SEARCH_QUERY_BYTES: usize = 16 * 1024;
const MAX_SEARCH_COMMAND_BYTES: usize = 64 * 1024;
const MAX_SEARCH_COMMAND_DEPTH: usize = 16;
const MAX_SEARCH_COMMAND_NODES: usize = 2_048;
const MAX_ALLOWED_DOMAINS: usize = 20;
const MAX_ALLOWED_DOMAIN_BYTES: usize = 253;
const MAX_ALLOWED_DOMAINS_BYTES: usize = 4 * 1024;
const ALLOWED_COMMAND_KEYS: &[&str] = &[
    "search_query",
    "image_query",
    "open",
    "click",
    "find",
    "screenshot",
    "finance",
    "weather",
    "sports",
    "time",
    "response_length",
];

/// Backend-neutral search result used by the public client wrapper.
pub(super) struct BackendSearchResult {
    pub(super) content: String,
    pub(super) citation_pairs: Vec<(String, String)>,
    pub(super) references: Vec<WebSearchReference>,
}

impl BackendSearchResult {
    pub(super) fn citations(&self) -> Vec<String> {
        self.citation_pairs
            .iter()
            .map(|(_, url)| url.clone())
            .collect()
    }
}

pub(super) fn execution_error(message: impl Into<String>) -> xai_tool_runtime::ToolError {
    xai_tool_runtime::ToolError::execution(
        xai_tool_protocol::ToolId::new("web_search").expect("valid tool ID"),
        message.into(),
    )
}

pub(super) fn validate_search_query(query: &str) -> Result<(), xai_tool_runtime::ToolError> {
    if query.trim().is_empty() {
        return Err(execution_error("A non-empty web search query is required"));
    }
    if query.len() > MAX_SEARCH_QUERY_BYTES {
        return Err(execution_error("Web search query exceeded the size limit"));
    }
    Ok(())
}

pub(super) fn validate_allowed_domains(
    domains: Option<Vec<String>>,
) -> Result<Option<Vec<String>>, xai_tool_runtime::ToolError> {
    let Some(domains) = domains else {
        return Ok(None);
    };
    if domains.len() > MAX_ALLOWED_DOMAINS {
        return Err(execution_error(
            "Web search domain filters exceeded the count limit",
        ));
    }

    let mut normalized = Vec::with_capacity(domains.len());
    let mut seen = std::collections::HashSet::with_capacity(domains.len());
    let mut total_bytes = 0usize;
    for domain in domains {
        let domain = domain.trim();
        if domain.is_empty()
            || domain.len() > MAX_ALLOWED_DOMAIN_BYTES
            || domain.chars().any(char::is_control)
        {
            return Err(execution_error("Web search domain filter is invalid"));
        }
        let host = url::Host::parse(domain)
            .map_err(|_| execution_error("Web search domain filter is invalid"))?
            .to_string()
            .to_ascii_lowercase();
        total_bytes = total_bytes.saturating_add(host.len());
        if total_bytes > MAX_ALLOWED_DOMAINS_BYTES {
            return Err(execution_error(
                "Web search domain filters exceeded the size limit",
            ));
        }
        if seen.insert(host.clone()) {
            normalized.push(host);
        }
    }

    Ok((!normalized.is_empty()).then_some(normalized))
}

pub(super) fn validate_search_commands(
    commands: &serde_json::Value,
) -> Result<(), xai_tool_runtime::ToolError> {
    let object = commands
        .as_object()
        .ok_or_else(|| execution_error("Web search commands must be an object"))?;
    if object.is_empty()
        || object.len() > ALLOWED_COMMAND_KEYS.len()
        || object
            .keys()
            .any(|key| !ALLOWED_COMMAND_KEYS.contains(&key.as_str()))
    {
        return Err(execution_error("Web search commands are invalid"));
    }

    let mut visited = 0usize;
    validate_command_value(commands, 0, &mut visited)?;
    let encoded = serde_json::to_vec(commands)
        .map_err(|_| execution_error("Web search commands could not be encoded"))?;
    if encoded.len() > MAX_SEARCH_COMMAND_BYTES {
        return Err(execution_error(
            "Web search commands exceeded the size limit",
        ));
    }
    Ok(())
}

fn validate_command_value(
    value: &serde_json::Value,
    depth: usize,
    visited: &mut usize,
) -> Result<(), xai_tool_runtime::ToolError> {
    if depth > MAX_SEARCH_COMMAND_DEPTH || *visited >= MAX_SEARCH_COMMAND_NODES {
        return Err(execution_error(
            "Web search commands exceeded the structure limit",
        ));
    }
    *visited += 1;
    match value {
        serde_json::Value::Array(values) => {
            for value in values {
                validate_command_value(value, depth + 1, visited)?;
            }
        }
        serde_json::Value::Object(values) => {
            for value in values.values() {
                validate_command_value(value, depth + 1, visited)?;
            }
        }
        serde_json::Value::String(value) if value.len() > MAX_SEARCH_QUERY_BYTES => {
            return Err(execution_error(
                "Web search command text exceeded the size limit",
            ));
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_filters_are_bounded_normalized_hosts() {
        let domains = validate_allowed_domains(Some(vec![
            " Example.COM ".to_owned(),
            "example.com".to_owned(),
        ]))
        .unwrap();
        assert_eq!(domains, Some(vec!["example.com".to_owned()]));

        for invalid in [
            "https://example.com",
            "user@example.com",
            "example.com/path",
            "example.com:443",
        ] {
            assert!(validate_allowed_domains(Some(vec![invalid.to_owned()])).is_err());
        }
    }

    #[test]
    fn queries_are_nonempty_and_bounded() {
        assert!(validate_search_query("release notes").is_ok());
        assert!(validate_search_query("   ").is_err());
        assert!(validate_search_query(&"x".repeat(MAX_SEARCH_QUERY_BYTES + 1)).is_err());
    }

    #[test]
    fn commands_reject_unknown_keys_and_excessive_text() {
        assert!(validate_search_commands(&serde_json::json!({"open": []})).is_ok());
        assert!(validate_search_commands(&serde_json::json!({"unknown": []})).is_err());
        assert!(
            validate_search_commands(&serde_json::json!({
                "search_query": [{"q": "x".repeat(MAX_SEARCH_QUERY_BYTES + 1)}]
            }))
            .is_err()
        );
    }
}
