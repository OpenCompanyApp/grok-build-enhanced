use std::fmt;
use std::sync::Arc;

use reqwest::header::{AUTHORIZATION, HeaderName, HeaderValue};
use xai_grok_sampler::{RequestAuth, RequestAuthError};
use xai_grok_sampling_types::{ApiBackend, CredentialBinding};

use super::{KimiCodeCredentialStore, KimiCodeCredentials};

#[derive(Clone)]
struct KimiCodeAuthSnapshot {
    api_key: String,
    binding: CredentialBinding,
}

#[derive(Clone)]
enum KimiCodeCredentialResolver {
    Stored(KimiCodeCredentialStore),
    CapturedEnvironment(KimiCodeAuthSnapshot),
}

impl KimiCodeCredentialResolver {
    fn new(
        store: KimiCodeCredentialStore,
        credentials: KimiCodeCredentials,
        expected: &CredentialBinding,
    ) -> Self {
        if expected.same_record(&super::process_environment_binding()) {
            Self::CapturedEnvironment(KimiCodeAuthSnapshot {
                api_key: credentials.api_key().to_owned(),
                binding: expected.clone(),
            })
        } else {
            Self::Stored(store)
        }
    }

    fn resolve(&self) -> Result<Option<KimiCodeAuthSnapshot>, RequestAuthError> {
        match self {
            Self::Stored(store) => store
                .load()
                .map(|credentials| {
                    credentials.map(|credentials| KimiCodeAuthSnapshot {
                        api_key: credentials.api_key().to_owned(),
                        binding: credentials.credential_binding(),
                    })
                })
                .map_err(|_| RequestAuthError::CredentialsUnavailable),
            Self::CapturedEnvironment(snapshot) => Ok(Some(snapshot.clone())),
        }
    }
}

fn resolve_pinned_snapshot(
    resolver: &KimiCodeCredentialResolver,
    expected: &CredentialBinding,
) -> Result<Option<KimiCodeAuthSnapshot>, RequestAuthError> {
    let Some(snapshot) = resolver.resolve()? else {
        return Ok(None);
    };
    Ok((snapshot.binding.same_record(expected)
        && snapshot.binding.generation >= expected.generation)
        .then_some(snapshot))
}

#[derive(Clone)]
pub struct KimiCodeSamplerRequestAuth {
    resolver: KimiCodeCredentialResolver,
    backend: ApiBackend,
    expected: CredentialBinding,
}

impl KimiCodeSamplerRequestAuth {
    pub fn new(
        store: KimiCodeCredentialStore,
        credentials: KimiCodeCredentials,
        backend: ApiBackend,
        expected: CredentialBinding,
    ) -> Self {
        Self {
            resolver: KimiCodeCredentialResolver::new(store, credentials, &expected),
            backend,
            expected,
        }
    }
}

impl fmt::Debug for KimiCodeSamplerRequestAuth {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("KimiCodeSamplerRequestAuth")
            .finish_non_exhaustive()
    }
}

impl RequestAuth for KimiCodeSamplerRequestAuth {
    fn apply(
        &self,
        headers: &mut reqwest::header::HeaderMap,
    ) -> Result<CredentialBinding, RequestAuthError> {
        let snapshot = resolve_pinned_snapshot(&self.resolver, &self.expected)?
            .ok_or(RequestAuthError::CredentialsUnavailable)?;
        headers.remove(AUTHORIZATION);
        headers.remove(HeaderName::from_static("x-api-key"));
        let mut value = match self.backend {
            ApiBackend::Messages => HeaderValue::from_str(&snapshot.api_key),
            ApiBackend::ChatCompletions => {
                HeaderValue::from_str(&format!("Bearer {}", snapshot.api_key))
            }
            ApiBackend::Responses => return Err(RequestAuthError::CredentialsUnavailable),
        }
        .map_err(|_| RequestAuthError::CredentialsUnavailable)?;
        value.set_sensitive(true);
        match self.backend {
            ApiBackend::Messages => {
                headers.insert(HeaderName::from_static("x-api-key"), value);
            }
            ApiBackend::ChatCompletions => {
                headers.insert(AUTHORIZATION, value);
            }
            ApiBackend::Responses => unreachable!(),
        }
        Ok(snapshot.binding)
    }

    fn recover_unauthorized(
        &self,
        _rejected: CredentialBinding,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
        // Kimi Code API keys are static. A 401 must surface and require an
        // explicit login/key replacement rather than replaying the same key.
        Box::pin(std::future::ready(false))
    }
}

pub fn shared_sampler_request_auth(
    store: KimiCodeCredentialStore,
    credentials: KimiCodeCredentials,
    backend: ApiBackend,
    expected: CredentialBinding,
) -> xai_grok_sampler::SharedRequestAuth {
    Arc::new(KimiCodeSamplerRequestAuth::new(
        store,
        credentials,
        backend,
        expected,
    ))
}

#[derive(Clone)]
pub struct KimiCodeToolAuthProvider {
    resolver: KimiCodeCredentialResolver,
    expected: CredentialBinding,
}

impl KimiCodeToolAuthProvider {
    pub fn new(
        store: KimiCodeCredentialStore,
        credentials: KimiCodeCredentials,
        expected: CredentialBinding,
    ) -> Self {
        Self {
            resolver: KimiCodeCredentialResolver::new(store, credentials, &expected),
            expected,
        }
    }
}

impl fmt::Debug for KimiCodeToolAuthProvider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("KimiCodeToolAuthProvider")
            .finish_non_exhaustive()
    }
}

impl xai_grok_tools::types::ApiKeyProvider for KimiCodeToolAuthProvider {
    fn current_api_key(&self) -> Option<String> {
        // Kimi credentials are available only through the provider-marked
        // request-auth contract below. Generic xAI tool clients must never be
        // able to obtain this key as an unscoped bearer.
        None
    }

    fn request_auth_provider_id(&self) -> Option<&str> {
        Some(xai_grok_tools::types::KIMI_CODE_PROVIDER_ID)
    }

    fn current_request_auth_async(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Option<xai_grok_tools::types::RequestAuth>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            let snapshot = resolve_pinned_snapshot(&self.resolver, &self.expected)
                .ok()
                .flatten()?;
            Some(xai_grok_tools::types::RequestAuth::for_provider_snapshot(
                xai_grok_tools::types::KIMI_CODE_PROVIDER_ID,
                xai_grok_tools::types::RequestCredentialSnapshot::new(
                    snapshot
                        .binding
                        .record_id
                        .unwrap_or_else(|| "kimi-code".to_owned()),
                    snapshot.binding.generation.max(1),
                ),
                [(
                    "authorization".to_owned(),
                    format!("Bearer {}", snapshot.api_key),
                )],
            ))
        })
    }
}

pub fn shared_tool_auth_provider(
    store: KimiCodeCredentialStore,
    credentials: KimiCodeCredentials,
    expected: CredentialBinding,
) -> xai_grok_tools::types::SharedApiKeyProvider {
    Arc::new(KimiCodeToolAuthProvider::new(store, credentials, expected))
}

#[cfg(test)]
mod tests {
    use super::*;
    use xai_grok_sampler::RequestAuth as _;
    use xai_grok_sampling_types::ProviderId;

    #[tokio::test]
    async fn messages_uses_sensitive_x_api_key_without_bearer() {
        let directory = tempfile::tempdir().unwrap();
        let store = KimiCodeCredentialStore::from_auth_path(directory.path().join("auth.json"));
        let credentials = KimiCodeCredentials::new("sentinel-kimi-key").unwrap();
        let expected = credentials.credential_binding();
        store.save(credentials.clone()).await.unwrap();
        let auth =
            KimiCodeSamplerRequestAuth::new(store, credentials, ApiBackend::Messages, expected);
        let mut headers = reqwest::header::HeaderMap::new();
        let binding = auth.apply(&mut headers).unwrap();
        assert_eq!(binding.provider, ProviderId::KimiCode);
        assert!(headers["x-api-key"].is_sensitive());
        assert!(headers.get(AUTHORIZATION).is_none());
    }

    #[tokio::test]
    async fn captured_environment_auth_does_not_adopt_a_later_stored_record() {
        let directory = tempfile::tempdir().unwrap();
        let store = KimiCodeCredentialStore::from_auth_path(directory.path().join("auth.json"));
        let environment = KimiCodeCredentials::new("sentinel-environment-key").unwrap();
        let expected = super::super::process_environment_binding();
        let auth = KimiCodeSamplerRequestAuth::new(
            store.clone(),
            environment,
            ApiBackend::ChatCompletions,
            expected.clone(),
        );
        store
            .save(KimiCodeCredentials::new("sentinel-stored-key").unwrap())
            .await
            .unwrap();

        let mut headers = reqwest::header::HeaderMap::new();
        let binding = auth.apply(&mut headers).unwrap();

        assert_eq!(binding, expected);
        assert_eq!(
            headers[AUTHORIZATION].to_str().unwrap(),
            "Bearer sentinel-environment-key"
        );
    }

    #[test]
    fn restored_environment_binding_from_another_process_fails_closed() {
        let directory = tempfile::tempdir().unwrap();
        let store = KimiCodeCredentialStore::from_auth_path(directory.path().join("auth.json"));
        let environment = KimiCodeCredentials::new("sentinel-environment-key").unwrap();
        let expected = CredentialBinding {
            provider: ProviderId::KimiCode,
            source: xai_grok_sampling_types::CredentialSourceId::KimiCodeApiKey,
            record_id: Some("env-process:previous-process".to_owned()),
            generation: 1,
        };
        let auth = KimiCodeSamplerRequestAuth::new(
            store,
            environment,
            ApiBackend::ChatCompletions,
            expected,
        );
        let mut headers = reqwest::header::HeaderMap::new();

        let result = auth.apply(&mut headers);

        assert!(matches!(
            result,
            Err(RequestAuthError::CredentialsUnavailable)
        ));
        assert!(headers.get(AUTHORIZATION).is_none());
    }

    #[test]
    fn stored_auth_storage_error_does_not_fall_through_to_another_source() {
        let directory = tempfile::tempdir().unwrap();
        let auth_path = directory.path().join("auth-path-is-a-directory");
        std::fs::create_dir(&auth_path).unwrap();
        let store = KimiCodeCredentialStore::from_auth_path(auth_path);
        let credentials = KimiCodeCredentials::new("sentinel-stored-key").unwrap();
        let expected = credentials.credential_binding();
        let auth = KimiCodeSamplerRequestAuth::new(
            store,
            credentials,
            ApiBackend::ChatCompletions,
            expected,
        );
        let mut headers = reqwest::header::HeaderMap::new();

        assert!(auth.apply(&mut headers).is_err());
        assert!(headers.get(AUTHORIZATION).is_none());
    }

    #[tokio::test]
    async fn tool_auth_does_not_expose_the_kimi_key_as_a_generic_bearer() {
        use xai_grok_tools::types::ApiKeyProvider as _;

        let directory = tempfile::tempdir().unwrap();
        let store = KimiCodeCredentialStore::from_auth_path(directory.path().join("auth.json"));
        let credentials = KimiCodeCredentials::new("sentinel-kimi-key").unwrap();
        let expected = credentials.credential_binding();
        store.save(credentials.clone()).await.unwrap();
        let auth = KimiCodeToolAuthProvider::new(store, credentials, expected);

        assert!(auth.current_api_key().is_none());
        assert_eq!(
            auth.request_auth_provider_id(),
            Some(xai_grok_tools::types::KIMI_CODE_PROVIDER_ID)
        );
        assert!(auth.current_request_auth_async().await.is_some());
    }

    #[tokio::test]
    async fn request_auth_rejects_an_account_switch_after_session_binding() {
        let directory = tempfile::tempdir().unwrap();
        let store = KimiCodeCredentialStore::from_auth_path(directory.path().join("auth.json"));
        let first = KimiCodeCredentials::new("sentinel-first-key").unwrap();
        let expected = first.credential_binding();
        store.save(first.clone()).await.unwrap();
        let auth = KimiCodeSamplerRequestAuth::new(
            store.clone(),
            first,
            ApiBackend::ChatCompletions,
            expected,
        );
        store
            .save(KimiCodeCredentials::new("sentinel-second-key").unwrap())
            .await
            .unwrap();

        let mut headers = reqwest::header::HeaderMap::new();
        assert!(auth.apply(&mut headers).is_err());
        assert!(headers.get(AUTHORIZATION).is_none());
    }
}
