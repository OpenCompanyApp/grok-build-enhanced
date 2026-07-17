use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, USER_AGENT};

use super::{
    BackendSearchResult, execution_error, validate_allowed_domains, validate_search_commands,
    validate_search_query,
};
use crate::attribution::{SharedAttributionCallback, ToolConsumer};
use crate::implementations::web_search::types::{
    CodexWebSearchContext, CodexWebSearchMessageRole, CodexWebSearchTurnMetadata,
};
use crate::types::SharedApiKeyProvider;
use crate::types::api_key_provider::{
    AUTH_RECOVERY_EXHAUSTED_DETAILS_KEY, AUTH_RECOVERY_PROVIDER_DETAILS_KEY,
    OPENAI_CODEX_PROVIDER_ID, OpenAiCodexAuthError, require_openai_codex_auth_provider,
    resolve_openai_codex_request_auth,
};
use crate::types::output::WebSearchReference;

const OPENAI_CODEX_BASE_URL: &str = "https://chatgpt.com/backend-api/codex";
const OPENAI_CODEX_ORIGINATOR: &str = "grok_build_codex";
const CODEX_TURN_METADATA_HEADER: &str = "x-codex-turn-metadata";
const MAX_CODEX_SEARCH_RESPONSE_BYTES: usize = 10 * 1024 * 1024;
const MAX_CODEX_SEARCH_INPUT_MESSAGES: usize = 8;
const MAX_CODEX_SEARCH_INPUT_BYTES: usize = 32 * 1024;
const MAX_CODEX_SEARCH_USER_MESSAGE_BYTES: usize = 16 * 1024;
const MAX_CODEX_SEARCH_ASSISTANT_BYTES: usize = 4 * 1024;
const MAX_CODEX_TURN_METADATA_BYTES: usize = 4 * 1024;
const MAX_CODEX_RESULT_REFERENCES: usize = 32;
const MAX_CODEX_RESULT_CITATIONS: usize = 64;
const MAX_CODEX_RESULT_NODES: usize = 4_096;
const MAX_CODEX_RESULT_DEPTH: usize = 16;
const MAX_CODEX_REF_ID_BYTES: usize = 512;
const MAX_CODEX_RESULT_TITLE_CHARS: usize = 512;
const MAX_CODEX_RESULT_URL_BYTES: usize = 4_096;

/// ChatGPT Codex standalone search backend.
///
/// Authentication is intentionally non-optional. The backend only asks the
/// provider for generation-bound `RequestAuth`; it has no static-key field or
/// fallback path.
#[derive(Clone)]
pub(in crate::implementations::web_search) struct OpenAiCodexBackend {
    http: reqwest::Client,
    base_url: String,
    model: String,
    session_id: String,
    request_auth_provider: SharedApiKeyProvider,
    attribution_callback: Option<SharedAttributionCallback>,
}

impl OpenAiCodexBackend {
    pub(in crate::implementations::web_search) fn new(
        base_url: &str,
        model: &str,
        session_id: &str,
        request_auth_provider: SharedApiKeyProvider,
    ) -> Result<Self, xai_tool_runtime::ToolError> {
        let base_url = validate_codex_base_url(base_url)?;
        validate_body_identifier("session", session_id)?;
        validate_body_identifier("model", model)?;
        require_openai_codex_auth_provider(&request_auth_provider).map_err(codex_auth_error)?;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&format!("grok-build-codex/{}", xai_grok_version::VERSION))
                .map_err(|_| execution_error("Invalid Codex web search client version"))?,
        );
        headers.insert(
            HeaderName::from_static("originator"),
            HeaderValue::from_static(OPENAI_CODEX_ORIGINATOR),
        );
        headers.insert(
            HeaderName::from_static("version"),
            HeaderValue::from_static(xai_grok_version::OPENAI_CODEX_COMPATIBILITY_VERSION),
        );

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(60))
            // Never replay bearer/account/FedRAMP headers to a redirect target.
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| execution_error("Codex web search HTTP client could not be built"))?;

        Ok(Self {
            http,
            base_url,
            model: model.to_owned(),
            session_id: session_id.to_owned(),
            request_auth_provider,
            attribution_callback: None,
        })
    }

    pub(in crate::implementations::web_search) fn set_attribution_callback(
        &mut self,
        callback: Option<SharedAttributionCallback>,
    ) {
        self.attribution_callback = callback;
    }

    pub(in crate::implementations::web_search) async fn search(
        &self,
        query: &str,
        allowed_domains: Option<Vec<String>>,
    ) -> Result<BackendSearchResult, xai_tool_runtime::ToolError> {
        validate_search_query(query)?;
        self.run_commands(
            query,
            serde_json::json!({
                "search_query": [{"q": query}],
            }),
            allowed_domains,
            None,
        )
        .await
    }

    pub(in crate::implementations::web_search) async fn run_commands(
        &self,
        input_text: &str,
        commands: serde_json::Value,
        allowed_domains: Option<Vec<String>>,
        context: Option<&CodexWebSearchContext>,
    ) -> Result<BackendSearchResult, xai_tool_runtime::ToolError> {
        validate_search_commands(&commands)?;
        let allowed_domains = validate_allowed_domains(allowed_domains)?;

        let mut settings = serde_json::Map::from_iter([
            ("allowed_callers".to_owned(), serde_json::json!(["direct"])),
            (
                "external_web_access".to_owned(),
                serde_json::Value::Bool(true),
            ),
        ]);
        if let Some(domains) = allowed_domains.filter(|domains| !domains.is_empty()) {
            settings.insert(
                "filters".to_owned(),
                serde_json::json!({"allowed_domains": domains}),
            );
        }

        let mut body = serde_json::Map::from_iter([
            ("id".to_owned(), serde_json::json!(self.session_id)),
            ("model".to_owned(), serde_json::json!(self.model)),
            ("commands".to_owned(), commands),
            ("settings".to_owned(), serde_json::Value::Object(settings)),
            ("max_output_tokens".to_owned(), serde_json::json!(8192)),
        ]);
        let turn_metadata = if let Some(context) = context {
            let (input, metadata) = validate_codex_search_context(context)?;
            body.insert("input".to_owned(), input);
            Some(metadata)
        } else if !input_text.trim().is_empty() {
            if input_text.len() > MAX_CODEX_SEARCH_USER_MESSAGE_BYTES {
                return Err(execution_error(
                    "Codex web search input exceeded the size limit",
                ));
            }
            body.insert(
                "input".to_owned(),
                serde_json::Value::String(input_text.to_owned()),
            );
            None
        } else {
            None
        };

        self.execute(serde_json::Value::Object(body), turn_metadata)
            .await
    }

    async fn execute(
        &self,
        body: serde_json::Value,
        turn_metadata: Option<HeaderValue>,
    ) -> Result<BackendSearchResult, xai_tool_runtime::ToolError> {
        // `base_url` is sealed by the constructor, so this exact suffix is the
        // only Codex search endpoint that can receive provider credentials.
        let url = format!("{}/alpha/search", self.base_url);
        let mut recovered = false;

        loop {
            let auth = resolve_openai_codex_request_auth(&self.request_auth_provider)
                .await
                .map_err(codex_auth_error)?;
            let rejected_snapshot = auth.credential_snapshot().clone();

            let mut request = self.http.post(&url).json(&body);
            if let Some(value) = turn_metadata.as_ref() {
                request = request.header(
                    HeaderName::from_static(CODEX_TURN_METADATA_HEADER),
                    value.clone(),
                );
            }
            let response = auth
                .apply(request)
                .send()
                .await
                .map_err(|_| execution_error("Codex web search request could not be sent"))?;
            let status = response.status();

            if status == reqwest::StatusCode::UNAUTHORIZED {
                self.record_401_attribution();
                if !recovered
                    && self
                        .request_auth_provider
                        .recover_unauthorized_async(Some(rejected_snapshot))
                        .await
                {
                    recovered = true;
                    continue;
                }
                return Err(xai_tool_runtime::ToolError::unauthorized(
                    "ChatGPT Codex rejected web search authentication".to_owned(),
                )
                .with_details(serde_json::json!({
                    "tool_id": "web_search",
                    "status": 401,
                    (AUTH_RECOVERY_PROVIDER_DETAILS_KEY): OPENAI_CODEX_PROVIDER_ID,
                    (AUTH_RECOVERY_EXHAUSTED_DETAILS_KEY): true,
                })));
            }
            if !status.is_success() {
                return Err(execution_error(format!(
                    "Codex web search request failed with HTTP {status}"
                )));
            }

            let bytes = read_response_with_limit(response, MAX_CODEX_SEARCH_RESPONSE_BYTES).await?;
            let response: CodexSearchWireResponse = serde_json::from_slice(&bytes)
                .map_err(|_| execution_error("Codex web search returned an invalid response"))?;
            let projection = codex_result_projection(response.results.as_deref());
            return Ok(BackendSearchResult {
                content: response.output,
                citation_pairs: projection.citation_pairs,
                references: projection.references,
            });
        }
    }

    fn record_401_attribution(&self) {
        if let Some(callback) = self.attribution_callback.as_ref() {
            callback.record_401(ToolConsumer::WebSearch, true);
        }
    }
}

fn codex_auth_error(_error: OpenAiCodexAuthError) -> xai_tool_runtime::ToolError {
    execution_error("Codex web search authentication is unavailable")
}

fn validate_body_identifier(_label: &str, value: &str) -> Result<(), xai_tool_runtime::ToolError> {
    if value.trim().is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
        return Err(execution_error("Codex web search configuration is invalid"));
    }
    Ok(())
}

fn validate_codex_base_url(base_url: &str) -> Result<String, xai_tool_runtime::ToolError> {
    let production = base_url == OPENAI_CODEX_BASE_URL
        || base_url.strip_suffix('/') == Some(OPENAI_CODEX_BASE_URL);

    // Downstream crate tests need a narrow loopback seam for mock servers.
    // Production builds never accept a caller-selected credential destination.
    #[cfg(any(test, feature = "test-support"))]
    let loopback = reqwest::Url::parse(base_url).ok().is_some_and(|url| {
        url.scheme() == "http"
            && matches!(url.path(), "" | "/")
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none()
            && url
                .host_str()
                .is_some_and(|host| matches!(host, "127.0.0.1" | "localhost" | "::1"))
    });
    #[cfg(not(any(test, feature = "test-support")))]
    let loopback = false;

    if production {
        return Ok(OPENAI_CODEX_BASE_URL.to_owned());
    }
    if loopback {
        return Ok(base_url.trim_end_matches('/').to_owned());
    }
    Err(execution_error(
        "Codex web search credentials may only be sent to the ChatGPT Codex endpoint",
    ))
}

async fn read_response_with_limit(
    response: reqwest::Response,
    max_bytes: usize,
) -> Result<Vec<u8>, xai_tool_runtime::ToolError> {
    use futures_util::StreamExt as _;

    if response
        .content_length()
        .is_some_and(|length| length > max_bytes as u64)
    {
        return Err(execution_error(
            "Codex web search response exceeded the size limit",
        ));
    }

    let mut stream = response.bytes_stream();
    let mut body = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk =
            chunk.map_err(|_| execution_error("Codex web search response could not be read"))?;
        if chunk.len() > max_bytes.saturating_sub(body.len()) {
            return Err(execution_error(
                "Codex web search response exceeded the size limit",
            ));
        }
        body.try_reserve_exact(chunk.len())
            .map_err(|_| execution_error("Codex web search response could not be read"))?;
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

fn validate_codex_search_context(
    context: &CodexWebSearchContext,
) -> Result<(serde_json::Value, HeaderValue), xai_tool_runtime::ToolError> {
    if context.input.is_empty() || context.input.len() > MAX_CODEX_SEARCH_INPUT_MESSAGES {
        return Err(execution_error(
            "Codex web search context has an invalid message count",
        ));
    }
    if context.input.first().map(|message| message.role) != Some(CodexWebSearchMessageRole::User)
        || context.input.last().map(|message| message.role) != Some(CodexWebSearchMessageRole::User)
    {
        return Err(execution_error(
            "Codex web search context must start and end with visible user text",
        ));
    }

    let mut total_bytes = 0usize;
    let mut assistant_bytes = 0usize;
    let mut user_messages = 0usize;
    let mut wire_messages = Vec::with_capacity(context.input.len());
    for message in &context.input {
        if message.text.trim().is_empty() {
            return Err(execution_error(
                "Codex web search context contains an empty message",
            ));
        }
        total_bytes = total_bytes.saturating_add(message.text.len());
        let (role, content_type) = match message.role {
            CodexWebSearchMessageRole::User => {
                user_messages += 1;
                if message.text.len() > MAX_CODEX_SEARCH_USER_MESSAGE_BYTES {
                    return Err(execution_error(
                        "Codex web search user context exceeded the size limit",
                    ));
                }
                ("user", "input_text")
            }
            CodexWebSearchMessageRole::Assistant => {
                assistant_bytes = assistant_bytes.saturating_add(message.text.len());
                ("assistant", "output_text")
            }
        };
        wire_messages.push(serde_json::json!({
            "type": "message",
            "role": role,
            "content": [{
                "type": content_type,
                "text": message.text,
            }],
        }));
    }
    if !(1..=2).contains(&user_messages)
        || total_bytes > MAX_CODEX_SEARCH_INPUT_BYTES
        || assistant_bytes > MAX_CODEX_SEARCH_ASSISTANT_BYTES
    {
        return Err(execution_error(
            "Codex web search context exceeded its bounded visible-history window",
        ));
    }

    let metadata = codex_turn_metadata_header(&context.turn_metadata)?;
    Ok((serde_json::Value::Array(wire_messages), metadata))
}

fn codex_turn_metadata_header(
    metadata: &CodexWebSearchTurnMetadata,
) -> Result<HeaderValue, xai_tool_runtime::ToolError> {
    for value in [
        metadata.session_id.as_str(),
        metadata.thread_id.as_str(),
        metadata.turn_id.as_str(),
        metadata.model.as_str(),
    ] {
        if value.trim().is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
            return Err(execution_error("Codex web search turn metadata is invalid"));
        }
    }
    if metadata.reasoning_effort.as_ref().is_some_and(|effort| {
        effort.is_empty()
            || effort.len() > 32
            || !effort
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    }) {
        return Err(execution_error(
            "Codex web search reasoning metadata is invalid",
        ));
    }

    let encoded = ascii_json_string(metadata)?;
    if encoded.len() > MAX_CODEX_TURN_METADATA_BYTES {
        return Err(execution_error(
            "Codex web search turn metadata exceeded the size limit",
        ));
    }
    debug_assert!(encoded.is_ascii());
    let mut header = HeaderValue::from_str(&encoded)
        .map_err(|_| execution_error("Codex web search turn metadata could not be encoded"))?;
    // Correlation identifiers are not credentials, but keeping the complete
    // header sensitive prevents HTTP diagnostics from exposing turn state.
    header.set_sensitive(true);
    Ok(header)
}

fn ascii_json_string<T: serde::Serialize>(
    value: &T,
) -> Result<String, xai_tool_runtime::ToolError> {
    use std::fmt::Write as _;

    let json = serde_json::to_string(value)
        .map_err(|_| execution_error("Codex web search turn metadata could not be encoded"))?;
    let mut ascii = String::with_capacity(json.len());
    for character in json.chars() {
        if character.is_ascii() {
            ascii.push(character);
        } else {
            let mut code_units = [0u16; 2];
            for code_unit in character.encode_utf16(&mut code_units) {
                write!(&mut ascii, "\\u{:04x}", *code_unit).expect("writing to String cannot fail");
            }
        }
    }
    Ok(ascii)
}

#[derive(serde::Deserialize)]
struct CodexSearchWireResponse {
    #[serde(default)]
    output: String,
    #[serde(default)]
    results: Option<Vec<serde_json::Value>>,
}

struct CodexResultProjection {
    citation_pairs: Vec<(String, String)>,
    references: Vec<WebSearchReference>,
}

#[derive(Default)]
struct CodexResultProjectionBuilder {
    citation_pairs: Vec<(String, String)>,
    references: Vec<WebSearchReference>,
    seen_urls: std::collections::HashSet<String>,
    seen_refs: std::collections::HashSet<String>,
    visited_nodes: usize,
}

fn codex_result_projection(results: Option<&[serde_json::Value]>) -> CodexResultProjection {
    let mut builder = CodexResultProjectionBuilder::default();
    for result in results.unwrap_or_default() {
        collect_codex_result_projection(result, 0, &mut builder);
        if builder.visited_nodes >= MAX_CODEX_RESULT_NODES {
            break;
        }
    }
    CodexResultProjection {
        citation_pairs: builder.citation_pairs,
        references: builder.references,
    }
}

fn collect_codex_result_projection(
    value: &serde_json::Value,
    depth: usize,
    builder: &mut CodexResultProjectionBuilder,
) {
    if depth > MAX_CODEX_RESULT_DEPTH || builder.visited_nodes >= MAX_CODEX_RESULT_NODES {
        return;
    }
    builder.visited_nodes += 1;
    match value {
        serde_json::Value::Object(object) => {
            if let Some(url) = object
                .get("url")
                .and_then(serde_json::Value::as_str)
                .and_then(sanitize_codex_result_url)
            {
                let title = object
                    .get("title")
                    .and_then(serde_json::Value::as_str)
                    .map(sanitize_codex_result_title)
                    .unwrap_or_default();
                if builder.citation_pairs.len() < MAX_CODEX_RESULT_CITATIONS
                    && builder.seen_urls.insert(url.clone())
                {
                    builder.citation_pairs.push((title.clone(), url.clone()));
                }
                if builder.references.len() < MAX_CODEX_RESULT_REFERENCES
                    && let Some(ref_id) = object
                        .get("ref_id")
                        .and_then(serde_json::Value::as_str)
                        .and_then(sanitize_codex_ref_id)
                    && builder.seen_refs.insert(ref_id.clone())
                {
                    builder
                        .references
                        .push(WebSearchReference { ref_id, title, url });
                }
            }
            for child in object.values() {
                collect_codex_result_projection(child, depth + 1, builder);
                if builder.visited_nodes >= MAX_CODEX_RESULT_NODES {
                    break;
                }
            }
        }
        serde_json::Value::Array(values) => {
            for child in values {
                collect_codex_result_projection(child, depth + 1, builder);
                if builder.visited_nodes >= MAX_CODEX_RESULT_NODES {
                    break;
                }
            }
        }
        _ => {}
    }
}

fn sanitize_codex_ref_id(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()
        && value.len() <= MAX_CODEX_REF_ID_BYTES
        && value.is_ascii()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':')))
    .then(|| value.to_owned())
}

fn sanitize_codex_result_title(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(MAX_CODEX_RESULT_TITLE_CHARS)
        .collect()
}

fn sanitize_codex_result_url(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value.len() > MAX_CODEX_RESULT_URL_BYTES {
        return None;
    }
    let parsed = reqwest::Url::parse(value).ok()?;
    if !matches!(parsed.scheme(), "http" | "https")
        || parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return None;
    }
    Some(parsed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementations::web_search::types::CodexWebSearchMessage;
    use crate::types::{ApiKeyProvider, RequestAuth, RequestCredentialSnapshot};
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    struct CodexTestProvider {
        auth_calls: AtomicUsize,
        recovery_calls: AtomicUsize,
        recover: bool,
        rejected: Mutex<Option<RequestCredentialSnapshot>>,
    }

    impl CodexTestProvider {
        fn new(recover: bool) -> Self {
            Self {
                auth_calls: AtomicUsize::new(0),
                recovery_calls: AtomicUsize::new(0),
                recover,
                rejected: Mutex::new(None),
            }
        }
    }

    impl ApiKeyProvider for CodexTestProvider {
        fn current_api_key(&self) -> Option<String> {
            None
        }

        fn request_auth_provider_id(&self) -> Option<&str> {
            Some(OPENAI_CODEX_PROVIDER_ID)
        }

        fn current_request_auth_async(
            &self,
        ) -> Pin<Box<dyn Future<Output = Option<RequestAuth>> + Send + '_>> {
            let call = self.auth_calls.fetch_add(1, Ordering::SeqCst);
            let (token, generation) = if call == 0 {
                ("Bearer sentinel-first-token", 7)
            } else {
                ("Bearer sentinel-second-token", 8)
            };
            Box::pin(std::future::ready(Some(
                RequestAuth::for_provider_snapshot(
                    OPENAI_CODEX_PROVIDER_ID,
                    RequestCredentialSnapshot::new("sentinel-credential-identity", generation),
                    [
                        ("authorization".to_owned(), token.to_owned()),
                        (
                            "chatgpt-account-id".to_owned(),
                            "sentinel-account".to_owned(),
                        ),
                        ("x-openai-fedramp".to_owned(), "true".to_owned()),
                    ],
                ),
            )))
        }

        fn recover_unauthorized_async(
            &self,
            rejected: Option<RequestCredentialSnapshot>,
        ) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
            self.recovery_calls.fetch_add(1, Ordering::SeqCst);
            *self.rejected.lock().unwrap() = rejected;
            Box::pin(std::future::ready(self.recover))
        }
    }

    struct StaticKeyProvider;

    impl ApiKeyProvider for StaticKeyProvider {
        fn current_api_key(&self) -> Option<String> {
            Some("sentinel-static-key".to_owned())
        }
    }

    struct MislabelledCodexProvider;

    impl ApiKeyProvider for MislabelledCodexProvider {
        fn current_api_key(&self) -> Option<String> {
            None
        }

        fn request_auth_provider_id(&self) -> Option<&str> {
            Some(OPENAI_CODEX_PROVIDER_ID)
        }

        fn current_request_auth_async(
            &self,
        ) -> Pin<Box<dyn Future<Output = Option<RequestAuth>> + Send + '_>> {
            Box::pin(std::future::ready(Some(
                RequestAuth::for_provider_snapshot(
                    "sentinel-foreign-provider",
                    RequestCredentialSnapshot::new("sentinel-foreign-identity", 4),
                    [
                        (
                            "authorization".to_owned(),
                            "Bearer sentinel-foreign-token".to_owned(),
                        ),
                        (
                            "chatgpt-account-id".to_owned(),
                            "sentinel-foreign-account".to_owned(),
                        ),
                    ],
                ),
            )))
        }
    }

    fn backend(
        base_url: &str,
        provider: SharedApiKeyProvider,
    ) -> Result<OpenAiCodexBackend, xai_tool_runtime::ToolError> {
        OpenAiCodexBackend::new(
            base_url,
            "codex-search-model",
            "sentinel-session-id",
            provider,
        )
    }

    fn request_header_matches(request: &Request, name: &str, expected: &str) -> bool {
        request
            .headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            == Some(expected)
    }

    fn metadata() -> CodexWebSearchTurnMetadata {
        CodexWebSearchTurnMetadata {
            session_id: "sentinel-session-id".to_owned(),
            thread_id: "sentinel-thread-id".to_owned(),
            turn_id: "sentinel-turn-id".to_owned(),
            model: "codex-search-model".to_owned(),
            reasoning_effort: Some("xhigh".to_owned()),
        }
    }

    #[test]
    fn endpoint_is_sealed_and_constructor_rejects_static_key_providers() {
        for rejected in [
            "https://foreign.example/backend-api/codex",
            "http://chatgpt.com/backend-api/codex",
            "https://chatgpt.com:444/backend-api/codex",
            "https://user@chatgpt.com/backend-api/codex",
            "https://chatgpt.com/backend-api/codex/alpha",
            "https://chatgpt.com/backend-api/codex?redirect=foreign",
            "https://chatgpt.com/backend-api/codex#fragment",
        ] {
            let provider: SharedApiKeyProvider = Arc::new(CodexTestProvider::new(false));
            let error = backend(rejected, provider)
                .err()
                .expect("noncanonical Codex endpoint must fail")
                .to_string();
            assert!(error.contains("ChatGPT Codex endpoint"));
            assert!(!error.contains(rejected));
        }

        let provider: SharedApiKeyProvider = Arc::new(CodexTestProvider::new(false));
        let accepted = backend(&format!("{OPENAI_CODEX_BASE_URL}/"), provider)
            .expect("one trailing slash should canonicalize");
        assert_eq!(accepted.base_url, OPENAI_CODEX_BASE_URL);

        let static_provider: SharedApiKeyProvider = Arc::new(StaticKeyProvider);
        let error = backend(OPENAI_CODEX_BASE_URL, static_provider)
            .err()
            .expect("a generic/static key provider must fail construction")
            .to_string();
        assert!(error.contains("authentication is unavailable"));
        assert!(!error.contains("sentinel-static-key"));
    }

    #[tokio::test]
    async fn redirects_are_terminal_and_never_receive_codex_credentials() {
        let target = MockServer::start().await;
        let source = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/alpha/search"))
            .respond_with(
                ResponseTemplate::new(307)
                    .insert_header("location", format!("{}/credential-sink", target.uri())),
            )
            .mount(&source)
            .await;

        let provider: SharedApiKeyProvider = Arc::new(CodexTestProvider::new(false));
        let error = backend(&source.uri(), provider)
            .unwrap()
            .search("redirect contract", None)
            .await
            .err()
            .expect("redirect must be terminal")
            .to_string();
        assert!(error.contains("307"));

        let source_requests = source.received_requests().await.unwrap();
        assert_eq!(source_requests.len(), 1);
        for name in ["authorization", "chatgpt-account-id", "x-openai-fedramp"] {
            assert!(source_requests[0].headers.contains_key(name));
        }
        assert!(target.received_requests().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn chunked_responses_are_bounded_without_content_length() {
        use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0_u8; 4096];
            let _ = stream.read(&mut request).await.unwrap();
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n3\r\nabc\r\n3\r\ndef\r\n0\r\n\r\n",
                )
                .await
                .unwrap();
        });

        let response = reqwest::get(format!("http://{address}/chunked"))
            .await
            .unwrap();
        assert_eq!(response.content_length(), None);
        let error = read_response_with_limit(response, 5)
            .await
            .expect_err("streamed response must obey the byte cap")
            .to_string();
        assert!(error.contains("size limit"));
        server.await.unwrap();
    }

    #[test]
    fn context_and_turn_metadata_are_bounded_sensitive_projections() {
        let context = CodexWebSearchContext {
            input: vec![
                CodexWebSearchMessage::user("previous user"),
                CodexWebSearchMessage::assistant("previous assistant"),
                CodexWebSearchMessage::user("current user"),
            ],
            turn_metadata: metadata(),
        };
        let (wire, header) =
            validate_codex_search_context(&context).expect("bounded visible context should pass");
        assert!(wire.as_array().is_some_and(|messages| messages.len() == 3));
        assert!(header.is_sensitive());
        let rendered = format!("{header:?}");
        for forbidden in [
            "sentinel-session-id",
            "sentinel-thread-id",
            "sentinel-turn-id",
        ] {
            assert!(!rendered.contains(forbidden));
        }

        let oversized = CodexWebSearchContext {
            input: vec![
                CodexWebSearchMessage::user("previous user"),
                CodexWebSearchMessage::assistant("a".repeat(MAX_CODEX_SEARCH_ASSISTANT_BYTES + 1)),
                CodexWebSearchMessage::user("current user"),
            ],
            turn_metadata: metadata(),
        };
        assert!(validate_codex_search_context(&oversized).is_err());

        let too_many_users = CodexWebSearchContext {
            input: vec![
                CodexWebSearchMessage::user("one"),
                CodexWebSearchMessage::user("two"),
                CodexWebSearchMessage::user("three"),
            ],
            turn_metadata: metadata(),
        };
        assert!(validate_codex_search_context(&too_many_users).is_err());
    }

    #[test]
    fn result_projection_is_bounded_and_drops_unsafe_provider_fields() {
        let mut results = (0..80)
            .map(|index| {
                serde_json::json!({
                    "ref_id": format!("turn0search{index}"),
                    "title": format!("Result {index}"),
                    "url": format!("https://example.com/{index}"),
                    "opaque": {"provider_state": "sentinel-provider-state"},
                })
            })
            .collect::<Vec<_>>();
        results.extend([
            serde_json::json!({
                "ref_id": "unsafe",
                "title": "credential URL",
                "url": "https://sentinel-user:sentinel-password@example.com/private",
            }),
            serde_json::json!({
                "ref_id": "control\nid",
                "title": "control ref",
                "url": "https://safe.example/",
            }),
        ]);

        let projection = codex_result_projection(Some(&results));
        assert_eq!(projection.references.len(), MAX_CODEX_RESULT_REFERENCES);
        assert_eq!(projection.citation_pairs.len(), MAX_CODEX_RESULT_CITATIONS);
        let projected = serde_json::to_string(&projection.references).unwrap();
        for forbidden in [
            "sentinel-provider-state",
            "sentinel-user",
            "sentinel-password",
            "control\\nid",
        ] {
            assert!(!projected.contains(forbidden));
        }
    }

    #[tokio::test]
    async fn alpha_search_schema_headers_and_projection_are_backend_scoped() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/alpha/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "output": "Search output",
                "results": [
                    {
                        "ref_id": "turn0search0",
                        "url": "https://example.com/article",
                        "title": "  Example\n article  "
                    },
                    {
                        "ref_id": "unsafe",
                        "url": "https://sentinel-user:sentinel-pass@example.com/private",
                        "opaque_state": "sentinel-provider-state"
                    }
                ]
            })))
            .mount(&server)
            .await;

        let provider: SharedApiKeyProvider = Arc::new(CodexTestProvider::new(false));
        let context = CodexWebSearchContext {
            input: vec![
                CodexWebSearchMessage::user("previous user"),
                CodexWebSearchMessage::assistant("previous assistant"),
                CodexWebSearchMessage::user("current user"),
            ],
            turn_metadata: metadata(),
        };
        let commands = serde_json::json!({
            "search_query": [{"q": "current release", "recency": 7}],
            "image_query": [{"q": "terminal interface"}],
            "open": [{"ref_id": "turn0search0", "lineno": 20}],
            "click": [{"ref_id": "turn0view0", "id": 3}],
            "find": [{"ref_id": "turn0search0", "pattern": "install"}],
            "screenshot": [{"ref_id": "turn0view1", "pageno": 0}],
            "finance": [{"ticker": "BTC", "type": "crypto", "market": ""}],
            "weather": [{
                "location": "Belgium, Brussels",
                "start": "2026-07-17",
                "duration": 3,
            }],
            "sports": [{
                "tool": "sports",
                "fn": "schedule",
                "league": "nba",
                "team": "BOS",
                "num_games": 2,
                "locale": "en-US",
            }],
            "time": [{"utc_offset": "+02:00"}],
            "response_length": "long",
        });
        let result = backend(&server.uri(), provider)
            .unwrap()
            .run_commands(
                "standalone commands",
                commands.clone(),
                Some(vec!["example.com".to_owned()]),
                Some(&context),
            )
            .await
            .expect("Codex standalone search should succeed");
        assert_eq!(result.content, "Search output");
        assert_eq!(result.references.len(), 1);
        assert_eq!(result.references[0].title, "Example article");
        assert_eq!(result.citation_pairs.len(), 1);

        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        let request = &requests[0];
        assert_eq!(request.url.path(), "/alpha/search");
        assert!(request_header_matches(
            request,
            "authorization",
            "Bearer sentinel-first-token"
        ));
        assert!(request_header_matches(
            request,
            "chatgpt-account-id",
            "sentinel-account"
        ));
        assert!(request_header_matches(request, "x-openai-fedramp", "true"));
        assert!(request.headers.contains_key(CODEX_TURN_METADATA_HEADER));
        for forbidden in [
            "x-xai-token-auth",
            "x-grok-conv-id",
            "x-grok-req-id",
            "x-grok-model-override",
            "x-codex-turn-state",
        ] {
            assert!(!request.headers.contains_key(forbidden));
        }

        let body: serde_json::Value = serde_json::from_slice(&request.body).unwrap();
        assert!(
            body.as_object()
                .is_some_and(|object| object.keys().all(|key| matches!(
                    key.as_str(),
                    "id" | "model" | "input" | "commands" | "settings" | "max_output_tokens"
                )))
        );
        assert!(body.get("commands").is_some_and(|value| value == &commands));
        assert!(
            body.get("input")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|messages| messages.len() == 3)
        );
        assert!(
            body.pointer("/settings/allowed_callers/0")
                .and_then(serde_json::Value::as_str)
                == Some("direct")
        );
        assert!(
            body.pointer("/settings/filters/allowed_domains/0")
                .and_then(serde_json::Value::as_str)
                == Some("example.com")
        );
        let body_text = String::from_utf8_lossy(&request.body);
        for forbidden in [
            "sentinel-first-token",
            "sentinel-account",
            "sentinel-credential-identity",
            "x-codex-turn-state",
        ] {
            assert!(!body_text.contains(forbidden));
        }
    }

    #[tokio::test]
    async fn recovery_uses_the_signing_generation_and_replays_only_once() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/alpha/search"))
            .respond_with(
                ResponseTemplate::new(401)
                    .set_body_string("sentinel-provider-body-must-not-surface"),
            )
            .mount(&server)
            .await;

        let provider = Arc::new(CodexTestProvider::new(true));
        let provider_dyn: SharedApiKeyProvider = provider.clone();
        let failure = backend(&server.uri(), provider_dyn)
            .unwrap()
            .search("recovery contract", None)
            .await
            .err()
            .expect("a second 401 must terminate");
        let details = failure.details.as_ref().expect("structured 401 details");
        assert!(
            details.get(AUTH_RECOVERY_PROVIDER_DETAILS_KEY)
                == Some(&serde_json::json!(OPENAI_CODEX_PROVIDER_ID))
        );
        assert!(details.get(AUTH_RECOVERY_EXHAUSTED_DETAILS_KEY) == Some(&serde_json::json!(true)));
        let error = failure.to_string();

        assert_eq!(provider.auth_calls.load(Ordering::SeqCst), 2);
        assert_eq!(provider.recovery_calls.load(Ordering::SeqCst), 1);
        let rejected = provider.rejected.lock().unwrap().clone().unwrap();
        assert!(rejected.generation() == 7);
        assert!(rejected.opaque_id() == "sentinel-credential-identity");
        assert_eq!(server.received_requests().await.unwrap().len(), 2);
        for forbidden in [
            "sentinel-provider-body-must-not-surface",
            "sentinel-first-token",
            "sentinel-second-token",
            "sentinel-account",
            "sentinel-credential-identity",
        ] {
            assert!(!error.contains(forbidden));
        }
    }

    #[tokio::test]
    async fn failures_do_not_reflect_provider_bodies_or_mismatched_auth_material() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/alpha/search"))
            .respond_with(
                ResponseTemplate::new(500).set_body_string("sentinel-token-account-provider-body"),
            )
            .mount(&server)
            .await;
        let provider: SharedApiKeyProvider = Arc::new(CodexTestProvider::new(false));
        let error = backend(&server.uri(), provider)
            .unwrap()
            .search("failure contract", None)
            .await
            .err()
            .expect("500 should fail")
            .to_string();
        assert!(error.contains("500"));
        assert!(!error.contains("sentinel-token-account-provider-body"));

        let untouched = MockServer::start().await;
        let mismatched: SharedApiKeyProvider = Arc::new(MislabelledCodexProvider);
        let error = backend(&untouched.uri(), mismatched)
            .unwrap()
            .search("provider isolation", None)
            .await
            .err()
            .expect("foreign RequestAuth must fail before HTTP")
            .to_string();
        assert!(untouched.received_requests().await.unwrap().is_empty());
        for forbidden in [
            "sentinel-foreign-provider",
            "sentinel-foreign-identity",
            "sentinel-foreign-token",
            "sentinel-foreign-account",
        ] {
            assert!(!error.contains(forbidden));
        }
    }
}
