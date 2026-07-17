use chrono::{DateTime, Duration, Utc};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;

use super::is_xai_oauth2_issuer;

pub(crate) const TOKEN_TTL: Duration = Duration::days(30);
const DEFAULT_EARLY_INVALIDATION_SECS: u64 = 300; // 5 minutes

/// Legacy auth.json scope key. Fallback for old devbox auth files.
pub(super) const LEGACY_SCOPE: &str = "https://accounts.x.ai/sign-in";

/// auth.json scope key for plain API key auth (desktop login, `grok login --api-key`).
pub const API_KEY_SCOPE: &str = "xai::api_key";

const BLOCKED_REASON_NO_LOGS: &str = "BLOCKED_REASON_NO_LOGS";
const BLOCKED_REASON_NO_LOGS_MODERATED: &str = "BLOCKED_REASON_NO_LOGS_MODERATED";

/// Token provenance (debugging/auth.json only -- no code branches on this).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthMode {
    /// Deprecated. Kept for deserializing old auth.json files.
    #[serde(alias = "grok")]
    WebLogin,
    /// OIDC or OAuth2 interactive login via customer IdP
    #[serde(alias = "oidc")]
    Oidc,
    /// External auth provider binary
    External,
    /// Plain API key (e.g. from grok-desktop login or `grok login --api-key`)
    ApiKey,
}

impl AuthMode {
    /// Whether this auth mode can access `supported_in_api: false` models.
    pub fn is_session_auth(&self) -> bool {
        match self {
            Self::WebLogin | Self::Oidc => true,
            Self::External | Self::ApiKey => false,
        }
    }
}

/// Wire value of `principal_type` for team OAuth principals (capitalized by
/// the auth service). Single source for every comparison site.
pub(crate) const TEAM_PRINCIPAL_TYPE: &str = "Team";

#[derive(Clone, Serialize, Deserialize)]
pub struct GrokAuth {
    pub key: String,
    pub auth_mode: AuthMode,
    pub create_time: DateTime<Utc>,
    pub user_id: String,
    pub email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_image_asset_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_blocked_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub team_blocked_reasons: Vec<String>,
    #[serde(default)]
    pub coding_data_retention_opt_out: bool,

    /// Deprecated. Kept for deserializing existing auth.json files.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_grok_code_access: Option<bool>,

    /// Refresh token (OIDC/OAuth2 or external provider).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,

    /// Server-provided expiration (from OIDC `expires_in`).
    /// When present, takes precedence over the hardcoded `TOKEN_TTL`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,

    /// OIDC issuer URL that issued this token (needed for refresh via discovery).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oidc_issuer: Option<String>,

    /// OIDC client_id used to obtain this token (needed for refresh).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oidc_client_id: Option<String>,
}

impl std::fmt::Debug for GrokAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GrokAuth")
            .field("auth_mode", &self.auth_mode)
            .field("has_key", &!self.key.is_empty())
            .field("has_user_id", &!self.user_id.is_empty())
            .field("has_email", &self.email.is_some())
            .field("expires_at", &self.expires_at)
            .field("has_refresh_token", &self.refresh_token.is_some())
            .finish_non_exhaustive()
    }
}

impl GrokAuth {
    /// Seconds since this credential was minted. Negative when the local
    /// clock stepped back past `create_time` (NTP correction, VM restore, or
    /// a sibling machine's clock via an adopted auth.json) — `create_time`
    /// is always stamped from the minting machine's local clock.
    pub(crate) fn mint_age_seconds(&self) -> i64 {
        Utc::now()
            .signed_duration_since(self.create_time)
            .num_seconds()
    }

    /// `true` when the token comes from a first-party xAI account
    /// (OIDC login against https://auth.x.ai or the local-dev equivalent).
    pub fn is_xai_auth(&self) -> bool {
        match self.auth_mode {
            AuthMode::Oidc => self
                .oidc_issuer
                .as_deref()
                .is_some_and(is_xai_oauth2_issuer),
            AuthMode::External | AuthMode::ApiKey | AuthMode::WebLogin => false,
        }
    }

    /// `true` when this auth can access grok.com managed MCP connectors.
    pub fn is_managed_mcp_eligible(&self) -> bool {
        self.is_xai_auth() || self.auth_mode == AuthMode::WebLogin
    }

    pub fn is_team_principal(&self) -> bool {
        self.principal_type.as_deref() == Some(TEAM_PRINCIPAL_TYPE) && self.team_id.is_some()
    }

    /// `true` when the team has Zero Data Retention (ZDR) enabled.
    pub fn is_zdr_team(&self) -> bool {
        self.team_blocked_reasons
            .iter()
            .any(|r| r == BLOCKED_REASON_NO_LOGS || r == BLOCKED_REASON_NO_LOGS_MODERATED)
    }

    /// `true` when the team has ZDR or the user opted out of coding data
    /// retention. Use this for trace-upload and research-data gates.
    /// Product analytics (`telemetry_enabled`) and user-facing sync
    /// features should use `is_zdr_team()` directly.
    pub fn is_data_collection_disabled(&self) -> bool {
        self.is_zdr_team() || self.coding_data_retention_opt_out
    }

    /// Carry `/user`-derived fields from a previous auth so refresh rebuilds don't drop them.
    pub(crate) fn carry_user_profile_from(&mut self, prev: &GrokAuth) {
        self.user_id = prev.user_id.clone();
        self.email = prev.email.clone();
        self.principal_type = prev.principal_type.clone();
        self.principal_id = prev.principal_id.clone();
        self.team_id = prev.team_id.clone();
        self.team_name = prev.team_name.clone();
        self.team_role = prev.team_role.clone();
        self.organization_id = prev.organization_id.clone();
        self.organization_name = prev.organization_name.clone();
        self.organization_role = prev.organization_role.clone();
        self.user_blocked_reason = prev.user_blocked_reason.clone();
        self.team_blocked_reasons = prev.team_blocked_reasons.clone();
        self.coding_data_retention_opt_out = prev.coding_data_retention_opt_out;
    }
}

impl Default for GrokAuth {
    fn default() -> Self {
        Self {
            key: String::new(),
            auth_mode: AuthMode::Oidc,
            create_time: Utc::now(),
            user_id: String::new(),
            email: None,
            first_name: None,
            last_name: None,
            profile_image_asset_id: None,
            principal_type: None,
            principal_id: None,
            team_id: None,
            team_name: None,
            team_role: None,
            organization_id: None,
            organization_name: None,
            organization_role: None,
            user_blocked_reason: None,
            team_blocked_reasons: vec![],
            coding_data_retention_opt_out: false,
            has_grok_code_access: None,
            refresh_token: None,
            expires_at: None,
            oidc_issuer: None,
            oidc_client_id: None,
        }
    }
}

#[cfg(test)]
impl GrokAuth {
    /// Returns a `GrokAuth` with sensible defaults for tests. Override fields
    /// with struct update syntax:
    /// ```ignore
    /// GrokAuth { key: "my-key".into(), ..GrokAuth::test_default() }
    /// ```
    pub fn test_default() -> Self {
        Self {
            key: "test-key".into(),
            user_id: "test-user".into(),
            ..Default::default()
        }
    }
}

/// Provider-aware contents of `auth.json`.
///
/// Historically every scope was assumed to contain a [`GrokAuth`].  That
/// assumption makes adding another first-class provider unsafe: serde would
/// either reject the whole file or a later xAI write would discard fields it
/// did not understand.  This wrapper keeps the existing xAI-facing map API,
/// while retaining provider records and unknown future scopes losslessly at
/// the JSON-value level across read/modify/write cycles.
#[derive(Clone, Default)]
pub struct AuthStore {
    records: BTreeMap<String, AuthRecord>,
}

#[derive(Clone)]
enum AuthRecord {
    Grok(GrokAuth),
    OpenAiCodex(super::codex::CodexCredentials),
    Unknown(serde_json::Value),
}

impl AuthStore {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Insert an xAI credential. Kept source-compatible with the former
    /// `BTreeMap<String, GrokAuth>` alias.
    pub(crate) fn insert(&mut self, scope: String, auth: GrokAuth) -> Option<GrokAuth> {
        match self.records.insert(scope, AuthRecord::Grok(auth)) {
            Some(AuthRecord::Grok(previous)) => Some(previous),
            Some(AuthRecord::OpenAiCodex(_) | AuthRecord::Unknown(_)) | None => None,
        }
    }

    pub(crate) fn insert_codex(
        &mut self,
        credentials: super::codex::CodexCredentials,
    ) -> Option<super::codex::CodexCredentials> {
        match self.records.insert(
            super::codex::OPENAI_CODEX_SCOPE.to_string(),
            AuthRecord::OpenAiCodex(credentials),
        ) {
            Some(AuthRecord::OpenAiCodex(previous)) => Some(previous),
            Some(AuthRecord::Grok(_) | AuthRecord::Unknown(_)) | None => None,
        }
    }

    /// Return an xAI credential. Provider records deliberately do not appear
    /// through this compatibility view.
    pub(crate) fn get(&self, scope: impl AsRef<str>) -> Option<&GrokAuth> {
        match self.records.get(scope.as_ref()) {
            Some(AuthRecord::Grok(auth)) => Some(auth),
            Some(AuthRecord::OpenAiCodex(_) | AuthRecord::Unknown(_)) | None => None,
        }
    }

    pub(crate) fn get_mut(&mut self, scope: impl AsRef<str>) -> Option<&mut GrokAuth> {
        match self.records.get_mut(scope.as_ref()) {
            Some(AuthRecord::Grok(auth)) => Some(auth),
            Some(AuthRecord::OpenAiCodex(_) | AuthRecord::Unknown(_)) | None => None,
        }
    }

    pub(crate) fn get_codex(&self) -> Option<&super::codex::CodexCredentials> {
        match self.records.get(super::codex::OPENAI_CODEX_SCOPE) {
            Some(AuthRecord::OpenAiCodex(credentials)) => Some(credentials),
            Some(AuthRecord::Grok(_) | AuthRecord::Unknown(_)) | None => None,
        }
    }

    /// Remove any record at `scope`. The return value remains the old xAI
    /// return type because existing callers only use removal for xAI scopes.
    pub(crate) fn remove(&mut self, scope: impl AsRef<str>) -> Option<GrokAuth> {
        match self.records.remove(scope.as_ref()) {
            Some(AuthRecord::Grok(auth)) => Some(auth),
            Some(AuthRecord::OpenAiCodex(_) | AuthRecord::Unknown(_)) | None => None,
        }
    }

    pub(crate) fn remove_codex(&mut self) -> Option<super::codex::CodexCredentials> {
        match self.records.remove(super::codex::OPENAI_CODEX_SCOPE) {
            Some(AuthRecord::OpenAiCodex(credentials)) => Some(credentials),
            Some(AuthRecord::Grok(_) | AuthRecord::Unknown(_)) | None => None,
        }
    }

    pub(crate) fn contains_key(&self, scope: impl AsRef<str>) -> bool {
        self.records.contains_key(scope.as_ref())
    }

    pub(crate) fn keys(&self) -> impl Iterator<Item = &String> {
        self.records.keys()
    }

    /// Iterate only xAI credentials. This preserves the semantics expected by
    /// policy code such as managed-team discovery.
    pub(crate) fn values(&self) -> impl Iterator<Item = &GrokAuth> {
        self.records.values().filter_map(|record| match record {
            AuthRecord::Grok(auth) => Some(auth),
            AuthRecord::OpenAiCodex(_) | AuthRecord::Unknown(_) => None,
        })
    }

    pub(crate) fn len(&self) -> usize {
        self.records.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

impl Serialize for AuthStore {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.records.len()))?;
        for (scope, record) in &self.records {
            match record {
                AuthRecord::Grok(auth) => map.serialize_entry(scope, auth)?,
                AuthRecord::OpenAiCodex(credentials) => {
                    map.serialize_entry(scope, credentials)?;
                }
                AuthRecord::Unknown(value) => map.serialize_entry(scope, value)?,
            }
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for AuthStore {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = BTreeMap::<String, serde_json::Value>::deserialize(deserializer)?;
        let records = raw
            .into_iter()
            .map(|(scope, value)| {
                let record = if scope == super::codex::OPENAI_CODEX_SCOPE {
                    serde_json::from_value(value.clone())
                        .ok()
                        .filter(|credentials: &super::codex::CodexCredentials| {
                            credentials.validate_persisted().is_ok()
                        })
                        .map(AuthRecord::OpenAiCodex)
                        .unwrap_or(AuthRecord::Unknown(value))
                } else {
                    serde_json::from_value(value.clone())
                        .map(AuthRecord::Grok)
                        .unwrap_or(AuthRecord::Unknown(value))
                };
                (scope, record)
            })
            .collect();
        Ok(Self { records })
    }
}

/// User information from the cli-chat-proxy `GET /v1/user` endpoint.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UserInfo {
    pub(crate) user_id: String,
    #[serde(default)]
    pub(super) email: Option<String>,
    #[serde(default)]
    pub(super) first_name: Option<String>,
    #[serde(default)]
    pub(super) last_name: Option<String>,
    #[serde(default)]
    pub(super) profile_image_asset_id: Option<String>,
    #[serde(default)]
    pub(super) principal_type: Option<String>,
    #[serde(default)]
    pub(super) principal_id: Option<String>,
    #[serde(default)]
    pub(super) team_id: Option<String>,
    #[serde(default)]
    pub(super) team_name: Option<String>,
    #[serde(default)]
    pub(super) team_role: Option<String>,
    #[serde(default)]
    pub(super) organization_id: Option<String>,
    #[serde(default)]
    pub(super) organization_name: Option<String>,
    #[serde(default)]
    pub(super) organization_role: Option<String>,
    #[serde(default)]
    pub(super) user_blocked_reason: Option<String>,
    #[serde(default)]
    pub(super) team_blocked_reasons: Option<Vec<String>>,
    #[serde(default)]
    pub(super) coding_data_retention_opt_out: Option<bool>,
    /// Live subscription tier from the backend (only present when
    /// `?include=subscription` is passed to `/user`).
    #[serde(default)]
    pub(crate) subscription_tier: Option<String>,
}

/// Look up auth from the store by scope key.
///
/// Legacy `WebLogin` tokens (from the pre-OIDC `grok login --legacy`
/// flow) are skipped — they are validated via a per-request DB lookup
/// server-side which fails at high volume.  Skipping them here forces
/// affected users to re-authenticate via OIDC on next launch.
pub fn lookup_auth(map: &AuthStore, scope: &str) -> Option<GrokAuth> {
    let auth = map.get(scope).cloned().or_else(|| {
        if scope == LEGACY_SCOPE {
            None
        } else {
            map.get(LEGACY_SCOPE).cloned()
        }
    })?;
    if auth.auth_mode == AuthMode::WebLogin {
        tracing::info!("auth: ignoring legacy WebLogin token — re-authentication required");
        return None;
    }
    Some(auth)
}

/// Early-invalidation buffer. Override with `GROK_AUTH_EARLY_INVALIDATION_SECS`
/// for testing (e.g. `=5` to shrink the buffer to 5 seconds).
pub(super) fn early_invalidation() -> Duration {
    std::env::var("GROK_AUTH_EARLY_INVALIDATION_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(|s| Duration::seconds(s as i64))
        .unwrap_or_else(|| Duration::seconds(DEFAULT_EARLY_INVALIDATION_SECS as i64))
}

pub(crate) fn is_expired(auth: &GrokAuth) -> bool {
    is_expired_with_buffer(auth, early_invalidation())
}

/// Like [`is_expired`] but with an explicit pre-expiry buffer. Pass
/// `Duration::zero()` for actual (hard) expiry — the instant the token would
/// really be rejected on the wire, with no early-invalidation margin.
pub(crate) fn is_expired_with_buffer(auth: &GrokAuth, buffer: Duration) -> bool {
    if let Some(expires_at) = auth.expires_at {
        Utc::now() >= (expires_at - buffer)
    } else {
        let age = Utc::now().signed_duration_since(auth.create_time);
        age >= (TOKEN_TTL - buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_auth(mode: AuthMode) -> GrokAuth {
        GrokAuth {
            key: "k".into(),
            auth_mode: mode,
            create_time: Utc::now(),
            user_id: "u".into(),
            email: None,
            first_name: None,
            last_name: None,
            profile_image_asset_id: None,
            principal_type: None,
            principal_id: None,
            team_id: None,
            team_name: None,
            team_role: None,
            organization_id: None,
            organization_name: None,
            organization_role: None,
            user_blocked_reason: None,
            team_blocked_reasons: vec![],
            coding_data_retention_opt_out: false,
            has_grok_code_access: None,
            refresh_token: None,
            expires_at: None,
            oidc_issuer: None,
            oidc_client_id: None,
        }
    }

    #[test]
    fn lookup_auth_skips_weblogin_on_primary_scope() {
        let mut map = AuthStore::new();
        map.insert("scope".into(), make_auth(AuthMode::WebLogin));
        assert!(lookup_auth(&map, "scope").is_none());
    }

    #[test]
    fn lookup_auth_skips_weblogin_on_legacy_fallback() {
        let mut map = AuthStore::new();
        map.insert(LEGACY_SCOPE.into(), make_auth(AuthMode::WebLogin));
        assert!(lookup_auth(&map, "other-scope").is_none());
    }

    #[test]
    fn lookup_auth_returns_oidc_token() {
        let mut map = AuthStore::new();
        map.insert("scope".into(), make_auth(AuthMode::Oidc));
        assert!(lookup_auth(&map, "scope").is_some());
    }

    #[test]
    fn lookup_auth_returns_api_key_token() {
        let mut map = AuthStore::new();
        map.insert("scope".into(), make_auth(AuthMode::ApiKey));
        assert!(lookup_auth(&map, "scope").is_some());
    }

    #[test]
    fn future_codex_record_is_preserved_as_unknown() {
        let raw = serde_json::json!({
            (super::super::codex::OPENAI_CODEX_SCOPE): {
                "schema_version": 999,
                "provider": "openai_codex",
                "future_secret_container": { "opaque": "sentinel" }
            },
            "future::other": { "shape": [1, 2, 3] }
        });
        let store: AuthStore = serde_json::from_value(raw.clone()).unwrap();
        assert!(store.get_codex().is_none());
        assert!(store.contains_key(super::super::codex::OPENAI_CODEX_SCOPE));
        assert_eq!(serde_json::to_value(store).unwrap(), raw);
    }

    /// subscriptionTier present → deserializes to Some.
    #[test]
    fn user_info_subscription_tier_present() {
        let json = r#"{
            "userId": "u1",
            "subscriptionTier": "SuperGrokPro"
        }"#;
        let info: UserInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.subscription_tier.as_deref(), Some("SuperGrokPro"));
    }

    /// subscriptionTier absent → deserializes to None (backwards compat).
    #[test]
    fn user_info_subscription_tier_absent() {
        let json = r#"{"userId": "u1"}"#;
        let info: UserInfo = serde_json::from_str(json).unwrap();
        assert!(info.subscription_tier.is_none());
    }

    /// subscriptionTier null → deserializes to None.
    #[test]
    fn user_info_subscription_tier_null() {
        let json = r#"{"userId": "u1", "subscriptionTier": null}"#;
        let info: UserInfo = serde_json::from_str(json).unwrap();
        assert!(info.subscription_tier.is_none());
    }

    /// subscriptionTier empty string → deserializes to Some("").
    /// The paywall poller treats this as "no subscription" (line 230:
    /// `Some(tier) if !tier.is_empty()`) and keeps polling.
    #[test]
    fn user_info_subscription_tier_empty_string() {
        let json = r#"{"userId": "u1", "subscriptionTier": ""}"#;
        let info: UserInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.subscription_tier.as_deref(), Some(""));
    }
}
