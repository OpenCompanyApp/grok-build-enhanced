use std::fmt;
use std::sync::Arc;

use reqwest::header::{AUTHORIZATION, HeaderName, HeaderValue};
use xai_grok_sampler::{RequestAuth, RequestAuthError};
use xai_grok_sampling_types::{ApiBackend, CredentialBinding};

use super::{OpenCodeGoCredentialStore, OpenCodeGoCredentials};

#[derive(Clone)]
struct Snapshot {
    api_key: String,
    binding: CredentialBinding,
}

#[derive(Clone)]
enum Resolver {
    Stored(OpenCodeGoCredentialStore),
    Environment(Snapshot),
}

#[derive(Clone)]
pub struct OpenCodeGoRequestAuth {
    resolver: Resolver,
    backend: ApiBackend,
    expected: CredentialBinding,
}

impl OpenCodeGoRequestAuth {
    pub fn new(
        store: OpenCodeGoCredentialStore,
        credentials: OpenCodeGoCredentials,
        backend: ApiBackend,
        expected: CredentialBinding,
    ) -> Self {
        let resolver = if expected.same_record(&super::process_environment_binding()) {
            Resolver::Environment(Snapshot {
                api_key: credentials.api_key().to_owned(),
                binding: expected.clone(),
            })
        } else {
            Resolver::Stored(store)
        };
        Self {
            resolver,
            backend,
            expected,
        }
    }

    fn snapshot(&self) -> Result<Snapshot, RequestAuthError> {
        let snapshot = match &self.resolver {
            Resolver::Stored(store) => store
                .load()
                .map_err(|_| RequestAuthError::CredentialsUnavailable)?
                .map(|credentials| Snapshot {
                    api_key: credentials.api_key().to_owned(),
                    binding: credentials.credential_binding(),
                }),
            Resolver::Environment(snapshot) => Some(snapshot.clone()),
        }
        .ok_or(RequestAuthError::CredentialsUnavailable)?;
        if !snapshot.binding.same_record(&self.expected)
            || snapshot.binding.generation < self.expected.generation
        {
            return Err(RequestAuthError::CredentialsUnavailable);
        }
        Ok(snapshot)
    }
}

impl fmt::Debug for OpenCodeGoRequestAuth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenCodeGoRequestAuth")
            .finish_non_exhaustive()
    }
}

impl RequestAuth for OpenCodeGoRequestAuth {
    fn apply(
        &self,
        headers: &mut reqwest::header::HeaderMap,
    ) -> Result<CredentialBinding, RequestAuthError> {
        let snapshot = self.snapshot()?;
        headers.remove(AUTHORIZATION);
        headers.remove(HeaderName::from_static("x-api-key"));
        let (name, raw) = match self.backend {
            ApiBackend::Messages => (HeaderName::from_static("x-api-key"), snapshot.api_key),
            ApiBackend::ChatCompletions | ApiBackend::Responses => {
                (AUTHORIZATION, format!("Bearer {}", snapshot.api_key))
            }
        };
        let mut value =
            HeaderValue::from_str(&raw).map_err(|_| RequestAuthError::CredentialsUnavailable)?;
        value.set_sensitive(true);
        headers.insert(name, value);
        Ok(snapshot.binding)
    }

    fn recover_unauthorized(
        &self,
        _rejected: CredentialBinding,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + '_>> {
        Box::pin(std::future::ready(false))
    }
}

pub fn shared_request_auth(
    store: OpenCodeGoCredentialStore,
    credentials: OpenCodeGoCredentials,
    backend: ApiBackend,
    expected: CredentialBinding,
) -> xai_grok_sampler::SharedRequestAuth {
    Arc::new(OpenCodeGoRequestAuth::new(
        store,
        credentials,
        backend,
        expected,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use xai_grok_sampler::RequestAuth as _;

    fn auth(backend: ApiBackend) -> OpenCodeGoRequestAuth {
        let credentials = OpenCodeGoCredentials::new("synthetic-go-key").unwrap();
        let binding = super::super::process_environment_binding();
        OpenCodeGoRequestAuth::new(
            OpenCodeGoCredentialStore::from_auth_path(
                std::env::temp_dir().join("unused-opencode-go-auth.json"),
            ),
            credentials,
            backend,
            binding,
        )
    }

    #[test]
    fn openai_routes_use_one_sensitive_bearer() {
        for backend in [ApiBackend::ChatCompletions, ApiBackend::Responses] {
            let mut headers = reqwest::header::HeaderMap::new();
            auth(backend).apply(&mut headers).unwrap();
            assert!(headers[AUTHORIZATION].is_sensitive());
            assert!(headers.get("x-api-key").is_none());
            assert!(!format!("{headers:?}").contains("synthetic-go-key"));
        }
    }

    #[test]
    fn messages_route_uses_one_sensitive_x_api_key() {
        let mut headers = reqwest::header::HeaderMap::new();
        auth(ApiBackend::Messages).apply(&mut headers).unwrap();
        assert!(headers["x-api-key"].is_sensitive());
        assert!(headers.get(AUTHORIZATION).is_none());
        assert!(!format!("{headers:?}").contains("synthetic-go-key"));
    }
}
