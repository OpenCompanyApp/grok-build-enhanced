use std::collections::BTreeMap;

use chrono::{Duration, Utc};
use xai_grok_shell::auth::codex::{
    CODEX_CREDENTIAL_SCHEMA_VERSION, CodexCredentialProvider, CodexCredentialStore,
    CodexCredentials, CodexOAuthClient, SecretString,
};
use xai_grok_shell::auth::{AuthMode, GrokAuth};

fn credentials() -> CodexCredentials {
    let now = Utc::now();
    CodexCredentials {
        schema_version: CODEX_CREDENTIAL_SCHEMA_VERSION,
        provider: CodexCredentialProvider::OpenAiCodex,
        credential_id: uuid::Uuid::new_v4().to_string(),
        access_token: SecretString::new("codex-access-secret").unwrap(),
        refresh_token: SecretString::new("codex-refresh-secret").unwrap(),
        id_token: SecretString::new("codex-id-secret").unwrap(),
        expires_at: now + Duration::hours(1),
        created_at: now,
        updated_at: now,
        last_refresh_at: now,
        revision: 1,
        chatgpt_account_id: SecretString::new("chatgpt-account-secret").unwrap(),
        chatgpt_user_id: None,
        email: None,
        plan_type: Some("plus".to_owned()),
        is_fedramp: false,
        additional_fields: BTreeMap::new(),
    }
}

#[tokio::test]
async fn codex_scope_round_trip_preserves_xai_and_unknown_records() {
    let dir = tempfile::tempdir().unwrap();
    let auth_path = dir.path().join("auth.json");
    let xai = GrokAuth {
        key: "xai-api-secret".to_owned(),
        auth_mode: AuthMode::ApiKey,
        user_id: "xai-user".to_owned(),
        ..Default::default()
    };
    let initial = serde_json::json!({
        "xai::api_key": xai,
        "future::provider": {"opaque": [1, 2, 3]},
    });
    std::fs::write(&auth_path, serde_json::to_vec_pretty(&initial).unwrap()).unwrap();

    let store = CodexCredentialStore::from_auth_path(auth_path.clone());
    let saved = credentials();
    store.save(saved.clone()).await.unwrap();
    let loaded = store.load().unwrap().expect("Codex credential");
    assert_eq!(loaded, saved);
    let debug = format!("{loaded:?}");
    for secret in [
        loaded.access_token.expose_secret(),
        loaded.refresh_token.expose_secret(),
        loaded.id_token.expose_secret(),
        loaded.account_id(),
    ] {
        assert!(!debug.contains(secret));
    }

    let persisted: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&auth_path).unwrap()).unwrap();
    assert_eq!(persisted["xai::api_key"]["key"], "xai-api-secret");
    assert_eq!(
        persisted["future::provider"],
        serde_json::json!({"opaque": [1, 2, 3]})
    );
    assert_eq!(persisted["openai::codex"]["revision"], 1);

    store.remove().await.unwrap().expect("removed credential");
    let after_remove: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&auth_path).unwrap()).unwrap();
    assert!(after_remove.get("openai::codex").is_none());
    assert_eq!(after_remove["xai::api_key"]["key"], "xai-api-secret");
    assert_eq!(
        after_remove["future::provider"],
        serde_json::json!({"opaque": [1, 2, 3]})
    );
}

#[tokio::test]
async fn browser_login_url_has_pkce_state_and_current_codex_contract() {
    let pending = CodexOAuthClient::new()
        .begin_browser_login(None)
        .await
        .expect("loopback login should start");
    let url = pending.authorization_url();
    assert_eq!(url.scheme(), "https");
    assert_eq!(url.host_str(), Some("auth.openai.com"));
    assert_eq!(url.path(), "/oauth/authorize");
    let query: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
    assert_eq!(query.get("response_type").map(String::as_str), Some("code"));
    assert_eq!(
        query.get("code_challenge_method").map(String::as_str),
        Some("S256")
    );
    assert!(
        query
            .get("code_challenge")
            .is_some_and(|value| value.len() >= 43)
    );
    assert!(query.get("state").is_some_and(|value| value.len() >= 43));
    assert_eq!(
        query.get("originator").map(String::as_str),
        Some("grok_build_codex")
    );
    assert!(
        query
            .get("redirect_uri")
            .is_some_and(|value| value.starts_with("http://localhost:"))
    );
}
