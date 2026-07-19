use std::fmt;
use std::sync::Arc;

use reqwest::header::{AUTHORIZATION, HeaderName, HeaderValue};
use xai_grok_sampler::{RequestAuth, RequestAuthError};
use xai_grok_sampling_types::CredentialBinding;

use super::{ZaiCodingPlanCredentialStore, ZaiCodingPlanCredentials};

#[derive(Clone)]
struct ZaiCodingPlanAuthSnapshot {
    api_key: String,
    binding: CredentialBinding,
}

#[derive(Clone)]
enum ZaiCodingPlanCredentialResolver {
    Stored(ZaiCodingPlanCredentialStore),
    CapturedEnvironment(ZaiCodingPlanAuthSnapshot),
}

impl ZaiCodingPlanCredentialResolver {
    fn new(
        store: ZaiCodingPlanCredentialStore,
        credentials: ZaiCodingPlanCredentials,
        expected: &CredentialBinding,
    ) -> Self {
        if expected.same_record(&super::process_environment_binding()) {
            Self::CapturedEnvironment(ZaiCodingPlanAuthSnapshot {
                api_key: credentials.api_key().to_owned(),
                binding: expected.clone(),
            })
        } else {
            Self::Stored(store)
        }
    }

    fn resolve(&self) -> Result<Option<ZaiCodingPlanAuthSnapshot>, RequestAuthError> {
        match self {
            Self::Stored(store) => store
                .load()
                .map(|credentials| {
                    credentials.map(|credentials| ZaiCodingPlanAuthSnapshot {
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
    resolver: &ZaiCodingPlanCredentialResolver,
    expected: &CredentialBinding,
) -> Result<Option<ZaiCodingPlanAuthSnapshot>, RequestAuthError> {
    let Some(snapshot) = resolver.resolve()? else {
        return Ok(None);
    };
    Ok((snapshot.binding.same_record(expected)
        && snapshot.binding.generation >= expected.generation)
        .then_some(snapshot))
}

#[derive(Clone)]
pub struct ZaiCodingPlanSamplerRequestAuth {
    resolver: ZaiCodingPlanCredentialResolver,
    expected: CredentialBinding,
}

impl ZaiCodingPlanSamplerRequestAuth {
    pub fn new(
        store: ZaiCodingPlanCredentialStore,
        credentials: ZaiCodingPlanCredentials,
        expected: CredentialBinding,
    ) -> Self {
        Self {
            resolver: ZaiCodingPlanCredentialResolver::new(store, credentials, &expected),
            expected,
        }
    }
}

impl fmt::Debug for ZaiCodingPlanSamplerRequestAuth {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ZaiCodingPlanSamplerRequestAuth")
            .finish_non_exhaustive()
    }
}

impl RequestAuth for ZaiCodingPlanSamplerRequestAuth {
    fn apply(
        &self,
        headers: &mut reqwest::header::HeaderMap,
    ) -> Result<CredentialBinding, RequestAuthError> {
        let snapshot = resolve_pinned_snapshot(&self.resolver, &self.expected)?
            .ok_or(RequestAuthError::CredentialsUnavailable)?;
        headers.remove(AUTHORIZATION);
        headers.remove(HeaderName::from_static("x-api-key"));
        let mut value = HeaderValue::from_str(&format!("Bearer {}", snapshot.api_key))
            .map_err(|_| RequestAuthError::InvalidAuthorizationHeader)?;
        value.set_sensitive(true);
        headers.insert(AUTHORIZATION, value);
        Ok(snapshot.binding)
    }

    fn recover_unauthorized(
        &self,
        _rejected: CredentialBinding,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
        // Coding Plan API keys are static and have no refresh operation.
        Box::pin(std::future::ready(false))
    }
}

pub fn shared_sampler_request_auth(
    store: ZaiCodingPlanCredentialStore,
    credentials: ZaiCodingPlanCredentials,
    expected: CredentialBinding,
) -> xai_grok_sampler::SharedRequestAuth {
    Arc::new(ZaiCodingPlanSamplerRequestAuth::new(
        store,
        credentials,
        expected,
    ))
}

#[derive(Clone)]
pub struct ZaiCodingPlanToolAuthProvider {
    resolver: ZaiCodingPlanCredentialResolver,
    expected: CredentialBinding,
}

impl ZaiCodingPlanToolAuthProvider {
    pub fn new(
        store: ZaiCodingPlanCredentialStore,
        credentials: ZaiCodingPlanCredentials,
        expected: CredentialBinding,
    ) -> Self {
        Self {
            resolver: ZaiCodingPlanCredentialResolver::new(store, credentials, &expected),
            expected,
        }
    }
}

impl fmt::Debug for ZaiCodingPlanToolAuthProvider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ZaiCodingPlanToolAuthProvider")
            .finish_non_exhaustive()
    }
}

impl xai_grok_tools::types::ApiKeyProvider for ZaiCodingPlanToolAuthProvider {
    fn current_api_key(&self) -> Option<String> {
        None
    }

    fn request_auth_provider_id(&self) -> Option<&str> {
        Some(xai_grok_tools::types::ZAI_CODING_PLAN_PROVIDER_ID)
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
                xai_grok_tools::types::ZAI_CODING_PLAN_PROVIDER_ID,
                xai_grok_tools::types::RequestCredentialSnapshot::new(
                    snapshot
                        .binding
                        .record_id
                        .unwrap_or_else(|| "zai-coding-plan".to_owned()),
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
    store: ZaiCodingPlanCredentialStore,
    credentials: ZaiCodingPlanCredentials,
    expected: CredentialBinding,
) -> xai_grok_tools::types::SharedApiKeyProvider {
    Arc::new(ZaiCodingPlanToolAuthProvider::new(
        store,
        credentials,
        expected,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use xai_grok_sampler::RequestAuth as _;
    use xai_grok_sampling_types::ProviderId;

    #[tokio::test]
    async fn sampler_uses_one_sensitive_bearer() {
        let directory = tempfile::tempdir().unwrap();
        let store =
            ZaiCodingPlanCredentialStore::from_auth_path(directory.path().join("auth.json"));
        let credentials = ZaiCodingPlanCredentials::new("sentinel-zai-key").unwrap();
        let expected = credentials.credential_binding();
        store.save(credentials.clone()).await.unwrap();
        let auth = ZaiCodingPlanSamplerRequestAuth::new(store, credentials, expected);
        let mut headers = reqwest::header::HeaderMap::new();

        let binding = auth.apply(&mut headers).unwrap();

        assert_eq!(binding.provider, ProviderId::ZaiCodingPlan);
        assert!(headers[AUTHORIZATION].is_sensitive());
        assert!(headers.get("x-api-key").is_none());
    }

    #[tokio::test]
    async fn request_auth_rejects_an_account_switch() {
        let directory = tempfile::tempdir().unwrap();
        let store =
            ZaiCodingPlanCredentialStore::from_auth_path(directory.path().join("auth.json"));
        let first = ZaiCodingPlanCredentials::new("sentinel-first-key").unwrap();
        let expected = first.credential_binding();
        store.save(first.clone()).await.unwrap();
        let auth = ZaiCodingPlanSamplerRequestAuth::new(store.clone(), first, expected);
        store
            .save(ZaiCodingPlanCredentials::new("sentinel-second-key").unwrap())
            .await
            .unwrap();
        let mut headers = reqwest::header::HeaderMap::new();

        assert!(auth.apply(&mut headers).is_err());
        assert!(headers.get(AUTHORIZATION).is_none());
    }
}
