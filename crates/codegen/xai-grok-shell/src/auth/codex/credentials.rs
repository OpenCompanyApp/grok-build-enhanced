use std::collections::BTreeMap;
use std::fmt;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use super::CodexAuthError;

pub const CODEX_CREDENTIAL_SCHEMA_VERSION: u32 = 1;

/// A serializable secret whose formatting implementations never expose any
/// credential material. Access is deliberately explicit at the request edge.
#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct SecretString(String);

impl SecretString {
    pub fn new(value: impl Into<String>) -> Result<Self, CodexAuthError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(CodexAuthError::InvalidTokenResponse(
                "credential field was empty",
            ));
        }
        Ok(Self(value))
    }

    /// Expose the secret only where an authenticated request is constructed.
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

impl<'de> Deserialize<'de> for SecretString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodexCredentialProvider {
    #[serde(rename = "openai_codex")]
    OpenAiCodex,
}

fn schema_version() -> u32 {
    CODEX_CREDENTIAL_SCHEMA_VERSION
}

fn provider() -> CodexCredentialProvider {
    CodexCredentialProvider::OpenAiCodex
}

fn new_credential_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Credentials for the ChatGPT Codex subscription backend.
///
/// This record intentionally does not reuse [`crate::auth::GrokAuth`]: a
/// ChatGPT access token is not an xAI API key and has different identity,
/// routing, refresh, and account-switching semantics.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodexCredentials {
    #[serde(default = "schema_version")]
    pub schema_version: u32,
    #[serde(default = "provider")]
    pub provider: CodexCredentialProvider,
    /// Opaque local identifier used to bind sessions and cache entries to a
    /// credential generation without persisting account PII. It is random,
    /// never derived from a token or ChatGPT identity.
    #[serde(default = "new_credential_id")]
    pub credential_id: String,
    pub access_token: SecretString,
    pub refresh_token: SecretString,
    pub id_token: SecretString,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_refresh_at: DateTime<Utc>,
    pub revision: u64,
    pub chatgpt_account_id: SecretString,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chatgpt_user_id: Option<SecretString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<SecretString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_type: Option<String>,
    #[serde(default)]
    pub is_fedramp: bool,
    /// Preserve fields written by a newer client during read/modify/write.
    #[serde(flatten)]
    pub additional_fields: BTreeMap<String, serde_json::Value>,
}

impl fmt::Debug for CodexCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CodexCredentials").finish_non_exhaustive()
    }
}

impl CodexCredentials {
    pub(crate) fn validate_persisted(&self) -> Result<(), CodexAuthError> {
        if self.schema_version != CODEX_CREDENTIAL_SCHEMA_VERSION {
            return Err(CodexAuthError::InvalidTokenResponse(
                "unsupported credential schema version",
            ));
        }
        if uuid::Uuid::parse_str(&self.credential_id).is_err() {
            return Err(CodexAuthError::InvalidTokenResponse(
                "credential_id was not a UUID",
            ));
        }
        if self.account_id().trim().is_empty()
            || self.access_token().trim().is_empty()
            || self.refresh_token().trim().is_empty()
            || self.id_token.expose_secret().trim().is_empty()
        {
            return Err(CodexAuthError::InvalidTokenResponse(
                "persisted credential field was empty",
            ));
        }
        Ok(())
    }

    pub fn access_token(&self) -> &str {
        self.access_token.expose_secret()
    }

    pub fn credential_id(&self) -> &str {
        &self.credential_id
    }

    pub fn credential_binding(&self) -> xai_grok_sampling_types::CredentialBinding {
        xai_grok_sampling_types::CredentialBinding {
            provider: xai_grok_sampling_types::ProviderId::OpenAiCodex,
            source: xai_grok_sampling_types::CredentialSourceId::OpenAiCodexSubscription,
            record_id: Some(self.credential_id.clone()),
            generation: self.revision,
        }
    }

    pub fn refresh_token(&self) -> &str {
        self.refresh_token.expose_secret()
    }

    pub fn account_id(&self) -> &str {
        self.chatgpt_account_id.expose_secret()
    }

    pub fn user_id(&self) -> Option<&str> {
        self.chatgpt_user_id
            .as_ref()
            .map(SecretString::expose_secret)
    }

    pub fn needs_refresh_at(&self, now: DateTime<Utc>) -> bool {
        self.expires_at <= now + Duration::minutes(5)
    }

    pub fn same_identity(&self, other: &Self) -> bool {
        self.account_id() == other.account_id()
            && self.user_id() == other.user_id()
            && self.is_fedramp == other.is_fedramp
    }

    pub(crate) fn from_token_response(
        response: TokenResponse,
        previous: Option<&Self>,
        allowed_workspaces: Option<&[String]>,
    ) -> Result<Self, CodexAuthError> {
        let now = Utc::now();
        let TokenResponse {
            id_token,
            access_token,
            refresh_token,
            expires_in,
        } = response;
        let received_access_token = access_token.is_some();
        let access_token = access_token
            .or_else(|| previous.map(|credentials| credentials.access_token.0.clone()))
            .ok_or(CodexAuthError::InvalidTokenResponse(
                "token response omitted access_token",
            ))?;
        let id_token = id_token
            .or_else(|| previous.map(|credentials| credentials.id_token.0.clone()))
            .ok_or(CodexAuthError::InvalidTokenResponse(
                "token response omitted id_token",
            ))?;
        let refresh_token = refresh_token
            .or_else(|| previous.map(|credentials| credentials.refresh_token.0.clone()))
            .ok_or(CodexAuthError::InvalidTokenResponse(
                "token response omitted refresh_token",
            ))?;

        let identity = token_identity(&id_token, &access_token)?;
        if let Some(allowed) = allowed_workspaces
            && !allowed
                .iter()
                .any(|workspace| workspace == &identity.account_id)
        {
            return Err(CodexAuthError::WorkspaceNotAllowed);
        }

        let expires_at = if received_access_token {
            identity.access_expires_at.or_else(|| {
                expires_in
                    .and_then(|seconds| i64::try_from(seconds).ok())
                    .map(|seconds| now + Duration::seconds(seconds))
            })
        } else {
            // A refresh response may rotate only its ID/refresh tokens. Keep
            // the prior bearer's actual expiry in that case: `expires_in`
            // cannot safely extend an access token that was not replaced.
            identity
                .access_expires_at
                .or_else(|| previous.map(|credentials| credentials.expires_at))
        };
        let expires_at = expires_at.ok_or(CodexAuthError::InvalidTokenResponse(
            "access token omitted a usable expiry",
        ))?;

        let mut credentials = Self {
            schema_version: CODEX_CREDENTIAL_SCHEMA_VERSION,
            provider: CodexCredentialProvider::OpenAiCodex,
            credential_id: previous
                .map(|credentials| credentials.credential_id.clone())
                .unwrap_or_else(new_credential_id),
            access_token: SecretString::new(access_token)?,
            refresh_token: SecretString::new(refresh_token)?,
            id_token: SecretString::new(id_token)?,
            expires_at,
            created_at: previous.map_or(now, |credentials| credentials.created_at),
            updated_at: now,
            last_refresh_at: now,
            revision: previous.map_or(1, |credentials| credentials.revision.saturating_add(1)),
            chatgpt_account_id: SecretString::new(identity.account_id)?,
            chatgpt_user_id: identity.user_id.map(SecretString::new).transpose()?,
            email: identity.email.map(SecretString::new).transpose()?,
            plan_type: identity.plan_type,
            is_fedramp: identity.is_fedramp,
            additional_fields: previous
                .map(|credentials| credentials.additional_fields.clone())
                .unwrap_or_default(),
        };

        if let Some(previous) = previous {
            // Claims can be absent from a refresh response's replacement ID
            // token. Preserve previously established optional metadata.
            if credentials.chatgpt_user_id.is_none() {
                credentials.chatgpt_user_id = previous.chatgpt_user_id.clone();
            }
            if credentials.email.is_none() {
                credentials.email = previous.email.clone();
            }
            if credentials.plan_type.is_none() {
                credentials.plan_type.clone_from(&previous.plan_type);
            }
            if !credentials.same_identity(previous) {
                return Err(CodexAuthError::AccountChanged);
            }
        }

        Ok(credentials)
    }
}

#[derive(Clone, Deserialize)]
pub(crate) struct TokenResponse {
    #[serde(default)]
    pub(crate) id_token: Option<String>,
    #[serde(default)]
    pub(crate) access_token: Option<String>,
    #[serde(default)]
    pub(crate) refresh_token: Option<String>,
    #[serde(default)]
    pub(crate) expires_in: Option<u64>,
}

impl fmt::Debug for TokenResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TokenResponse").finish_non_exhaustive()
    }
}

#[derive(Default, Deserialize)]
struct JwtClaims {
    #[serde(default)]
    exp: Option<i64>,
    #[serde(default)]
    email: Option<String>,
    #[serde(rename = "https://api.openai.com/profile", default)]
    profile: Option<ProfileClaims>,
    #[serde(rename = "https://api.openai.com/auth", default)]
    auth: Option<AuthClaims>,
}

#[derive(Default, Deserialize)]
struct ProfileClaims {
    #[serde(default)]
    email: Option<String>,
}

#[derive(Default, Deserialize)]
struct AuthClaims {
    #[serde(default)]
    chatgpt_plan_type: Option<String>,
    #[serde(default)]
    chatgpt_user_id: Option<String>,
    #[serde(default)]
    user_id: Option<String>,
    #[serde(default)]
    chatgpt_account_id: Option<String>,
    #[serde(default)]
    chatgpt_account_is_fedramp: Option<bool>,
}

struct TokenIdentity {
    account_id: String,
    user_id: Option<String>,
    email: Option<String>,
    plan_type: Option<String>,
    is_fedramp: bool,
    access_expires_at: Option<DateTime<Utc>>,
}

fn decode_claims(token: &str) -> Result<JwtClaims, CodexAuthError> {
    let mut parts = token.split('.');
    let (Some(header), Some(payload), Some(signature), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(CodexAuthError::InvalidJwt);
    };
    if header.is_empty() || payload.is_empty() || signature.is_empty() {
        return Err(CodexAuthError::InvalidJwt);
    }
    let payload = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| CodexAuthError::InvalidJwt)?;
    serde_json::from_slice(&payload).map_err(|_| CodexAuthError::InvalidJwt)
}

/// Decode identity metadata from tokens returned by the TLS-protected OAuth
/// exchange. This is intentionally not signature verification and must never
/// be used as the authentication boundary; the backend validates the bearer.
fn token_identity(id_token: &str, access_token: &str) -> Result<TokenIdentity, CodexAuthError> {
    let id = decode_claims(id_token)?;
    let access = decode_claims(access_token)?;
    let id_auth = id.auth.unwrap_or_default();
    let access_auth = access.auth.unwrap_or_default();
    let is_fedramp = match (
        id_auth.chatgpt_account_is_fedramp,
        access_auth.chatgpt_account_is_fedramp,
    ) {
        (Some(from_id), Some(from_access)) if from_id != from_access => {
            return Err(CodexAuthError::AccountChanged);
        }
        (Some(value), _) | (_, Some(value)) => value,
        (None, None) => false,
    };

    if let (Some(from_id), Some(from_access)) = (
        id_auth.chatgpt_account_id.as_deref(),
        access_auth.chatgpt_account_id.as_deref(),
    ) && from_id != from_access
    {
        return Err(CodexAuthError::AccountChanged);
    }
    let account_id = id_auth
        .chatgpt_account_id
        .or(access_auth.chatgpt_account_id)
        .filter(|value| !value.trim().is_empty())
        .ok_or(CodexAuthError::MissingAccountId)?;

    let id_user = id_auth.chatgpt_user_id.or(id_auth.user_id);
    let access_user = access_auth.chatgpt_user_id.or(access_auth.user_id);
    if let (Some(from_id), Some(from_access)) = (id_user.as_deref(), access_user.as_deref())
        && from_id != from_access
    {
        return Err(CodexAuthError::AccountChanged);
    }

    let email = id
        .email
        .or_else(|| id.profile.and_then(|profile| profile.email))
        .or(access.email)
        .filter(|value| !value.trim().is_empty());
    let access_expires_at = access
        .exp
        .and_then(|seconds| DateTime::<Utc>::from_timestamp(seconds, 0));

    Ok(TokenIdentity {
        account_id,
        user_id: id_user.or(access_user),
        email,
        plan_type: id_auth.chatgpt_plan_type.or(access_auth.chatgpt_plan_type),
        is_fedramp,
        access_expires_at,
    })
}

#[cfg(test)]
pub(crate) fn jwt_for_test(claims: serde_json::Value) -> String {
    let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"none"}"#);
    let payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&claims).expect("serialize claims"));
    format!("{header}.{payload}.signature")
}

#[cfg(test)]
pub(crate) fn credentials_for_test(
    account_id: &str,
    refresh_token: &str,
    expires_at: DateTime<Utc>,
) -> CodexCredentials {
    let id_token = jwt_for_test(serde_json::json!({
        "https://api.openai.com/auth": { "chatgpt_account_id": account_id }
    }));
    let access_token = jwt_for_test(serde_json::json!({
        "exp": expires_at.timestamp(),
        "https://api.openai.com/auth": { "chatgpt_account_id": account_id }
    }));
    CodexCredentials::from_token_response(
        TokenResponse {
            id_token: Some(id_token),
            access_token: Some(access_token),
            refresh_token: Some(refresh_token.to_owned()),
            expires_in: None,
        },
        None,
        None,
    )
    .expect("test credentials")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response_for(
        account: &str,
        user: Option<&str>,
        refresh_token: Option<&str>,
        fedramp: bool,
    ) -> TokenResponse {
        let id_token = jwt_for_test(serde_json::json!({
            "email": "person@example.test",
            "https://api.openai.com/auth": {
                "chatgpt_account_id": account,
                "chatgpt_user_id": user,
                "chatgpt_plan_type": "plus",
                "chatgpt_account_is_fedramp": fedramp,
            }
        }));
        let access_token = jwt_for_test(serde_json::json!({
            "exp": (Utc::now() + Duration::hours(1)).timestamp(),
            "https://api.openai.com/auth": {
                "chatgpt_account_id": account,
                "chatgpt_user_id": user,
                "chatgpt_account_is_fedramp": fedramp,
            }
        }));
        TokenResponse {
            id_token: Some(id_token),
            access_token: Some(access_token),
            refresh_token: refresh_token.map(str::to_owned),
            expires_in: Some(3600),
        }
    }

    fn response_with_fedramp_claims(
        id_fedramp: Option<bool>,
        access_fedramp: Option<bool>,
    ) -> TokenResponse {
        let mut id_auth = serde_json::json!({ "chatgpt_account_id": "claim-account" });
        if let Some(value) = id_fedramp {
            id_auth["chatgpt_account_is_fedramp"] = serde_json::Value::Bool(value);
        }
        let mut access_auth = serde_json::json!({ "chatgpt_account_id": "claim-account" });
        if let Some(value) = access_fedramp {
            access_auth["chatgpt_account_is_fedramp"] = serde_json::Value::Bool(value);
        }

        TokenResponse {
            id_token: Some(jwt_for_test(serde_json::json!({
                "https://api.openai.com/auth": id_auth,
            }))),
            access_token: Some(jwt_for_test(serde_json::json!({
                "exp": (Utc::now() + Duration::hours(1)).timestamp(),
                "https://api.openai.com/auth": access_auth,
            }))),
            refresh_token: Some("claim-refresh".to_owned()),
            expires_in: Some(3600),
        }
    }

    #[test]
    fn extracts_account_user_plan_fedramp_and_expiry() {
        let credentials = CodexCredentials::from_token_response(
            response_for("acct-one", Some("user-one"), Some("refresh-one"), true),
            None,
            Some(&["acct-one".to_owned()]),
        )
        .unwrap();

        assert_eq!(credentials.account_id(), "acct-one");
        assert_eq!(credentials.user_id(), Some("user-one"));
        assert_eq!(credentials.plan_type.as_deref(), Some("plus"));
        assert!(credentials.is_fedramp);
        assert!(credentials.expires_at > Utc::now() + Duration::minutes(50));
        assert_eq!(credentials.revision, 1);
        assert!(!credentials.credential_id().is_empty());
    }

    #[test]
    fn equal_fedramp_claims_are_accepted() {
        let enabled = CodexCredentials::from_token_response(
            response_with_fedramp_claims(Some(true), Some(true)),
            None,
            None,
        )
        .unwrap();
        let disabled = CodexCredentials::from_token_response(
            response_with_fedramp_claims(Some(false), Some(false)),
            None,
            None,
        )
        .unwrap();

        assert!(enabled.is_fedramp);
        assert!(!disabled.is_fedramp);
    }

    #[test]
    fn one_present_fedramp_claim_is_authoritative() {
        let from_id = CodexCredentials::from_token_response(
            response_with_fedramp_claims(Some(true), None),
            None,
            None,
        )
        .unwrap();
        let from_access = CodexCredentials::from_token_response(
            response_with_fedramp_claims(None, Some(true)),
            None,
            None,
        )
        .unwrap();
        let explicit_disabled = CodexCredentials::from_token_response(
            response_with_fedramp_claims(Some(false), None),
            None,
            None,
        )
        .unwrap();

        assert!(from_id.is_fedramp);
        assert!(from_access.is_fedramp);
        assert!(!explicit_disabled.is_fedramp);
    }

    #[test]
    fn conflicting_fedramp_claims_fail_closed() {
        for response in [
            response_with_fedramp_claims(Some(true), Some(false)),
            response_with_fedramp_claims(Some(false), Some(true)),
        ] {
            assert!(matches!(
                CodexCredentials::from_token_response(response, None, None),
                Err(CodexAuthError::AccountChanged)
            ));
        }
    }

    #[test]
    fn rejects_mismatched_token_accounts_and_disallowed_workspace() {
        let mut response = response_for("acct-one", None, Some("refresh"), false);
        response.access_token = Some(jwt_for_test(serde_json::json!({
            "exp": (Utc::now() + Duration::hours(1)).timestamp(),
            "https://api.openai.com/auth": { "chatgpt_account_id": "acct-two" }
        })));
        assert!(matches!(
            CodexCredentials::from_token_response(response, None, None),
            Err(CodexAuthError::AccountChanged)
        ));

        assert!(matches!(
            CodexCredentials::from_token_response(
                response_for("acct-one", None, Some("refresh"), false),
                None,
                Some(&["acct-two".to_owned()]),
            ),
            Err(CodexAuthError::WorkspaceNotAllowed)
        ));
    }

    #[test]
    fn refresh_rotation_preserves_omitted_access_identity_expiry_and_opaque_id() {
        let original = CodexCredentials::from_token_response(
            response_for("acct-one", Some("user-one"), Some("refresh-old"), false),
            None,
            None,
        )
        .unwrap();
        let mut rotated_response =
            response_for("acct-one", Some("user-one"), Some("refresh-new"), false);
        // A refresh endpoint may rotate the refresh token while omitting both
        // replacement JWTs. A misleading `expires_in` must not extend the
        // unchanged bearer beyond its original expiry.
        rotated_response.id_token = None;
        rotated_response.access_token = None;
        rotated_response.expires_in = Some(24 * 60 * 60);
        let rotated =
            CodexCredentials::from_token_response(rotated_response, Some(&original), None).unwrap();

        assert_eq!(rotated.access_token(), original.access_token());
        assert_eq!(rotated.refresh_token(), "refresh-new");
        assert_eq!(rotated.expires_at, original.expires_at);
        assert_eq!(rotated.credential_id(), original.credential_id());
        assert_eq!(rotated.revision, original.revision + 1);
        assert!(rotated.same_identity(&original));
    }

    #[test]
    fn refresh_rejects_account_switch() {
        let original = CodexCredentials::from_token_response(
            response_for("acct-one", Some("user-one"), Some("refresh-old"), false),
            None,
            None,
        )
        .unwrap();
        assert!(matches!(
            CodexCredentials::from_token_response(
                response_for("acct-two", Some("user-two"), Some("refresh-new"), false),
                Some(&original),
                None,
            ),
            Err(CodexAuthError::AccountChanged)
        ));
    }

    #[test]
    fn debug_redacts_every_credential_and_identity_value() {
        let response = response_for(
            "sentinel-account",
            Some("sentinel-user"),
            Some("sentinel-refresh"),
            false,
        );
        let response_debug = format!("{response:?}");
        let credentials = CodexCredentials::from_token_response(response, None, None).unwrap();
        let access_token = credentials.access_token().to_owned();
        let id_token = credentials.id_token.expose_secret().to_owned();
        let credential_id = credentials.credential_id().to_owned();
        let credential_debug = format!("{credentials:?}");
        let mut alternate_routing = credentials.clone();
        alternate_routing.is_fedramp = !alternate_routing.is_fedramp;
        let alternate_debug = format!("{alternate_routing:?}");
        let secret_debug = format!("{:?}", SecretString::new("sentinel-secret").unwrap());

        assert_eq!(response_debug, "TokenResponse { .. }");
        assert_eq!(credential_debug, "CodexCredentials { .. }");
        assert_eq!(secret_debug, "<redacted>");
        for rendered in [&response_debug, &credential_debug, &secret_debug] {
            for forbidden in [
                "sentinel-account",
                "sentinel-user",
                "sentinel-refresh",
                "sentinel-secret",
            ] {
                assert!(
                    !rendered.contains(forbidden),
                    "credential debug output exposed private material"
                );
            }
            for forbidden in [&access_token, &id_token, &credential_id] {
                assert!(
                    !rendered.contains(forbidden),
                    "credential debug output exposed credential material"
                );
            }
        }
        assert_eq!(credential_debug, alternate_debug);
        assert!(!credential_debug.to_ascii_lowercase().contains("fedramp"));
    }
}
