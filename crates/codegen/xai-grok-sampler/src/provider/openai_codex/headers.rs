use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderName, HeaderValue};

use xai_grok_sampling_types::{
    CredentialBinding, CredentialSourceId, OPENAI_CODEX_COMPATIBILITY_VERSION,
    OPENAI_CODEX_RESPONSES_LITE_HEADER, ProviderId, Result, SamplingError,
};

pub(crate) const CHATGPT_ACCOUNT_ID_HEADER: &str = "chatgpt-account-id";
pub(crate) const OPENAI_FEDRAMP_HEADER: &str = "x-openai-fedramp";
pub(crate) const OPENAI_CODEX_ORIGINATOR: &str = "grok_build_codex";
pub(crate) const CODEX_SESSION_ID_HEADER: &str = "session-id";
pub(crate) const CODEX_THREAD_ID_HEADER: &str = "thread-id";
pub(crate) const CODEX_CLIENT_REQUEST_ID_HEADER: &str = "x-client-request-id";
pub(crate) const CODEX_TURN_STATE_HEADER: &str = "x-codex-turn-state";

fn is_protected_credential_header(name: &HeaderName) -> bool {
    matches!(
        name.as_str(),
        "authorization"
            | "proxy-authorization"
            | "x-api-key"
            | CHATGPT_ACCOUNT_ID_HEADER
            | "openai-organization"
            | "openai-project"
            | "x-openai-actor-authorization"
            | OPENAI_FEDRAMP_HEADER
            | "cookie"
            | "x-xai-token-auth"
            | "x-userid"
            | "x-email"
    )
}

fn is_allowed_credential_header(name: &HeaderName) -> bool {
    matches!(
        name.as_str(),
        "authorization" | CHATGPT_ACCOUNT_ID_HEADER | OPENAI_FEDRAMP_HEADER
    )
}

/// Detect credential-bearing aliases that must never escape a provider auth
/// hook. Header names are case-insensitive already; removing punctuation also
/// catches variants such as `x-api-key`, `x-apikey`, and `x-api-key-backup`.
///
/// This intentionally does not treat request identity/correlation headers as
/// credentials. Codex protocol headers are sealed separately after auth and
/// ordinary transport headers such as `traceparent` remain available.
pub(crate) fn is_credential_header_or_alias(name: &HeaderName) -> bool {
    if is_protected_credential_header(name) {
        return true;
    }

    let compact: String = name
        .as_str()
        .bytes()
        .filter(u8::is_ascii_alphanumeric)
        .map(char::from)
        .collect();

    compact.contains("authorization")
        || compact.contains("authentication")
        || compact.contains("apikey")
        || compact.contains("token")
        || compact.contains("secret")
        || compact.contains("credential")
        || compact.contains("cookie")
        || compact.contains("account")
        || compact.contains("organization")
        || compact.contains("project")
}

pub(crate) fn is_xai_specific_header(name: &HeaderName) -> bool {
    let name = name.as_str();
    name.starts_with("x-grok-")
        || name.starts_with("x-xai-")
        || matches!(name, "x-api-key" | "x-userid" | "x-email")
}

pub(crate) fn is_provider_identity_header(name: &HeaderName) -> bool {
    matches!(
        name.as_str(),
        "originator"
            | "version"
            | OPENAI_CODEX_RESPONSES_LITE_HEADER
            | CODEX_SESSION_ID_HEADER
            | CODEX_THREAD_ID_HEADER
            | CODEX_CLIENT_REQUEST_ID_HEADER
            | CODEX_TURN_STATE_HEADER
    )
}

pub(crate) fn insert_provider_identity(headers: &mut HeaderMap) {
    headers.insert(
        HeaderName::from_static("originator"),
        HeaderValue::from_static(OPENAI_CODEX_ORIGINATOR),
    );
    headers.insert(
        HeaderName::from_static("version"),
        HeaderValue::from_static(OPENAI_CODEX_COMPATIBILITY_VERSION),
    );
}

/// Strip credentials and provider-owned identity after generic header
/// injection, then restore the sealed Codex client identity before RequestAuth
/// receives exclusive ownership of credentials.
pub(crate) fn prepare_for_request_auth(headers: &mut HeaderMap) {
    let forbidden: Vec<HeaderName> = headers
        .keys()
        .filter(|name| {
            is_credential_header_or_alias(name)
                || is_xai_specific_header(name)
                || is_provider_identity_header(name)
        })
        .cloned()
        .collect();
    for name in forbidden {
        headers.remove(name);
    }
    insert_provider_identity(headers);
}

/// Seal provider-owned headers after RequestAuth runs, validate its narrow
/// credential output, and bind the request to the exact credential generation
/// that signed it.
pub(crate) fn seal_after_request_auth(
    headers: &mut HeaderMap,
    credential_binding: Option<&CredentialBinding>,
) -> Result<()> {
    for name in [
        OPENAI_CODEX_RESPONSES_LITE_HEADER,
        CODEX_SESSION_ID_HEADER,
        CODEX_THREAD_ID_HEADER,
        CODEX_CLIENT_REQUEST_ID_HEADER,
        CODEX_TURN_STATE_HEADER,
        "originator",
        "version",
    ] {
        headers.remove(name);
    }
    insert_provider_identity(headers);
    validate_request_auth_headers(headers)?;
    let binding = credential_binding.ok_or(SamplingError::InvalidConfiguration(
        "OpenAI Codex request authentication omitted its credential generation",
    ))?;
    validate_credential_binding(binding)
}

fn validate_request_auth_headers(headers: &mut HeaderMap) -> Result<()> {
    if headers
        .keys()
        .any(|name| is_credential_header_or_alias(name) && !is_allowed_credential_header(name))
    {
        return Err(SamplingError::InvalidConfiguration(
            "OpenAI Codex request authentication emitted an unsupported credential header",
        ));
    }
    if headers.keys().any(is_xai_specific_header) {
        return Err(SamplingError::InvalidConfiguration(
            "OpenAI Codex request authentication emitted an xAI-specific header",
        ));
    }

    let authorization_count = headers.get_all(AUTHORIZATION).iter().count();
    if authorization_count == 0 {
        return Err(SamplingError::Auth(
            "OpenAI Codex authorization is unavailable".to_string(),
        ));
    }
    if authorization_count != 1 {
        return Err(SamplingError::InvalidConfiguration(
            "OpenAI Codex request authentication emitted duplicate credential headers",
        ));
    }
    let has_bearer = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .is_some_and(|token| !token.is_empty());
    if !has_bearer {
        return Err(SamplingError::Auth(
            "OpenAI Codex authorization is unavailable".to_string(),
        ));
    }

    let account_count = headers.get_all(CHATGPT_ACCOUNT_ID_HEADER).iter().count();
    if account_count == 0 {
        return Err(SamplingError::Auth(
            "OpenAI Codex account selection is unavailable".to_string(),
        ));
    }
    if account_count != 1 {
        return Err(SamplingError::InvalidConfiguration(
            "OpenAI Codex request authentication emitted duplicate credential headers",
        ));
    }
    let has_account = headers
        .get(CHATGPT_ACCOUNT_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|account_id| !account_id.trim().is_empty());
    if !has_account {
        return Err(SamplingError::Auth(
            "OpenAI Codex account selection is unavailable".to_string(),
        ));
    }

    let fedramp_values = headers.get_all(OPENAI_FEDRAMP_HEADER);
    let fedramp_count = fedramp_values.iter().count();
    if fedramp_count > 1 {
        return Err(SamplingError::InvalidConfiguration(
            "OpenAI Codex request authentication emitted duplicate credential headers",
        ));
    }
    if fedramp_values
        .iter()
        .next()
        .is_some_and(|value| value.as_bytes() != b"true")
    {
        return Err(SamplingError::InvalidConfiguration(
            "OpenAI Codex FedRAMP authentication state must be exactly true when present",
        ));
    }

    // These values are sensitive even when a RequestAuth implementation did
    // not mark them itself. This prevents future request-debug output from
    // exposing either the bearer token or selected ChatGPT account.
    if let Some(value) = headers.get_mut(AUTHORIZATION) {
        value.set_sensitive(true);
    }
    if let Some(value) = headers.get_mut(CHATGPT_ACCOUNT_ID_HEADER) {
        value.set_sensitive(true);
    }
    if let Some(value) = headers.get_mut(OPENAI_FEDRAMP_HEADER) {
        value.set_sensitive(true);
    }

    Ok(())
}

fn validate_credential_binding(binding: &CredentialBinding) -> Result<()> {
    if binding.provider != ProviderId::OpenAiCodex
        || binding.source != CredentialSourceId::OpenAiCodexSubscription
        || binding
            .record_id
            .as_deref()
            .is_none_or(|record_id| record_id.trim().is_empty())
        || binding.generation == 0
    {
        return Err(SamplingError::InvalidConfiguration(
            "OpenAI Codex request authentication omitted its credential generation",
        ));
    }
    Ok(())
}

pub(crate) fn apply_request_identity(
    builder: reqwest::RequestBuilder,
    session_id: &str,
    conversation_id: &str,
    responses_lite: bool,
) -> Result<reqwest::RequestBuilder> {
    let sensitive_value = |raw: &str| {
        HeaderValue::from_str(raw)
            .map(|mut value| {
                value.set_sensitive(true);
                value
            })
            .map_err(|_| {
                SamplingError::InvalidConfiguration(
                    "OpenAI Codex request identity contains an invalid header value",
                )
            })
    };
    let mut builder = builder;
    if !session_id.is_empty() {
        builder = builder.header(CODEX_SESSION_ID_HEADER, sensitive_value(session_id)?);
    }
    if !conversation_id.is_empty() {
        builder = builder
            .header(CODEX_THREAD_ID_HEADER, sensitive_value(conversation_id)?)
            .header(
                CODEX_CLIENT_REQUEST_ID_HEADER,
                sensitive_value(conversation_id)?,
            );
    }
    if responses_lite {
        builder = builder.header(OPENAI_CODEX_RESPONSES_LITE_HEADER, "true");
    }
    Ok(builder)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_binding() -> CredentialBinding {
        let mut binding = CredentialBinding::openai_codex(Some("credential-record".to_owned()));
        binding.generation = 1;
        binding
    }

    fn valid_auth_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_static("Bearer subscription-token"),
        );
        headers.insert(
            CHATGPT_ACCOUNT_ID_HEADER,
            HeaderValue::from_static("selected-account"),
        );
        headers
    }

    #[test]
    fn credential_aliases_are_rejected_but_transport_headers_survive() {
        for name in [
            "proxy-authorization",
            "cookie",
            "set-cookie",
            "x-api-key-backup",
            "x-apikey",
            "openai-api-key",
            "x-auth-token",
            "x-client-secret",
        ] {
            assert!(is_credential_header_or_alias(&HeaderName::from_static(
                name
            )));
        }
        assert!(!is_credential_header_or_alias(&HeaderName::from_static(
            "traceparent"
        )));
    }

    #[test]
    fn sealed_auth_marks_every_credential_value_sensitive() {
        let mut headers = valid_auth_headers();
        headers.insert(OPENAI_FEDRAMP_HEADER, HeaderValue::from_static("true"));
        seal_after_request_auth(&mut headers, Some(&valid_binding())).unwrap();

        for name in [
            AUTHORIZATION.as_str(),
            CHATGPT_ACCOUNT_ID_HEADER,
            OPENAI_FEDRAMP_HEADER,
        ] {
            assert!(headers[name].is_sensitive(), "{name} must be sensitive");
        }
        assert_eq!(headers["originator"], OPENAI_CODEX_ORIGINATOR);
        assert_eq!(headers["version"], OPENAI_CODEX_COMPATIBILITY_VERSION);
    }

    #[test]
    fn request_identity_is_sensitive_and_provider_owned() {
        let request = apply_request_identity(
            reqwest::Client::new().post("https://example.test/responses"),
            "session-1",
            "thread-1",
            true,
        )
        .unwrap()
        .build()
        .unwrap();

        for name in [
            CODEX_SESSION_ID_HEADER,
            CODEX_THREAD_ID_HEADER,
            CODEX_CLIENT_REQUEST_ID_HEADER,
        ] {
            assert!(request.headers()[name].is_sensitive());
        }
        assert_eq!(
            request.headers()[OPENAI_CODEX_RESPONSES_LITE_HEADER],
            "true"
        );
    }
}
