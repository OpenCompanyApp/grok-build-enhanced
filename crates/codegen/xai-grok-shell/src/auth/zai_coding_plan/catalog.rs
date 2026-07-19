use std::collections::HashSet;
use std::io::Write;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use reqwest::header::{ACCEPT, ACCEPT_LANGUAGE, AUTHORIZATION, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};
use xai_grok_sampler::AuthScheme;
use xai_grok_sampling_types::{
    ApiBackend, ProviderId, ReasoningEffort, ReasoningEffortOption, ZAI_CODING_PLAN_BASE_URL,
    ZAI_CODING_PLAN_MAX_RESPONSE_BYTES, ZAI_CODING_PLAN_MODELS_URL,
};

use super::{ZaiCodingPlanAuthError, ZaiCodingPlanCredentials};
use crate::agent::config::{ModelEntry, ModelInfo};

const CACHE_SCHEMA_VERSION: u32 = 1;
const CACHE_FILE: &str = "zai-coding-plan-models.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZaiCodingPlanModel {
    pub id: String,
    #[serde(default)]
    pub display_name: Option<String>,
}

#[derive(Deserialize)]
struct ModelsResponse {
    data: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct CatalogCache {
    schema_version: u32,
    credential_id: String,
    fetched_at: DateTime<Utc>,
    models: Vec<ZaiCodingPlanModel>,
}

pub async fn fetch_models(
    credentials: &ZaiCodingPlanCredentials,
) -> Result<Vec<ZaiCodingPlanModel>, ZaiCodingPlanAuthError> {
    credentials.validate_persisted()?;
    let client = super::transport::http_client(std::time::Duration::from_secs(30))?;
    let mut authorization = HeaderValue::from_str(&format!("Bearer {}", credentials.api_key()))
        .map_err(|_| ZaiCodingPlanAuthError::InvalidCredential)?;
    authorization.set_sensitive(true);
    let response = client
        .get(ZAI_CODING_PLAN_MODELS_URL)
        .header(AUTHORIZATION, authorization)
        .header(ACCEPT, "application/json")
        .header(ACCEPT_LANGUAGE, "en-US,en")
        .header(
            USER_AGENT,
            format!("grok-agent/{}", xai_grok_version::VERSION),
        )
        .send()
        .await
        .map_err(|_| ZaiCodingPlanAuthError::InvalidResponse)?;
    let status = response.status();
    let bytes =
        super::transport::read_limited_body(response, ZAI_CODING_PLAN_MAX_RESPONSE_BYTES).await?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| ZaiCodingPlanAuthError::InvalidResponse)?;
    if let Some(code) = super::transport::business_code(&value) {
        return Err(ZaiCodingPlanAuthError::Business(code));
    }
    if !status.is_success() {
        return Err(ZaiCodingPlanAuthError::Http(status));
    }
    let response: ModelsResponse =
        serde_json::from_value(value).map_err(|_| ZaiCodingPlanAuthError::InvalidResponse)?;
    let models = validate_models(response.data.into_iter().filter_map(parse_model).collect());
    if models.is_empty() {
        return Err(ZaiCodingPlanAuthError::EmptyCatalog);
    }
    Ok(models)
}

fn parse_model(value: serde_json::Value) -> Option<ZaiCodingPlanModel> {
    let object = value.as_object()?;
    let id = object.get("id")?.as_str()?.to_owned();
    let display_name = object
        .get("name")
        .or_else(|| object.get("display_name"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned);
    Some(ZaiCodingPlanModel { id, display_name })
}

fn validate_models(models: Vec<ZaiCodingPlanModel>) -> Vec<ZaiCodingPlanModel> {
    let mut seen = HashSet::new();
    models
        .into_iter()
        .filter(|model| {
            !model.id.trim().is_empty()
                && model.id.len() <= 256
                && !model.id.chars().any(char::is_control)
                && !model.id.chars().any(char::is_whitespace)
                && seen.insert(model.id.clone())
                && model.display_name.as_deref().is_none_or(|name| {
                    !name.trim().is_empty()
                        && name.len() <= 512
                        && !name.chars().any(char::is_control)
                })
        })
        .collect()
}

pub fn cache_path(grok_home: &Path) -> PathBuf {
    grok_home.join("cache").join(CACHE_FILE)
}

pub fn save_cache(
    grok_home: &Path,
    credentials: &ZaiCodingPlanCredentials,
    models: &[ZaiCodingPlanModel],
) -> Result<(), ZaiCodingPlanAuthError> {
    let path = cache_path(grok_home);
    let parent = path.parent().ok_or_else(|| {
        ZaiCodingPlanAuthError::Storage(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "invalid Z.AI Coding Plan cache path",
        ))
    })?;
    std::fs::create_dir_all(parent)?;
    let cache = CatalogCache {
        schema_version: CACHE_SCHEMA_VERSION,
        credential_id: credentials.credential_id.clone(),
        fetched_at: Utc::now(),
        models: models.to_vec(),
    };
    let bytes =
        serde_json::to_vec_pretty(&cache).map_err(|_| ZaiCodingPlanAuthError::InvalidResponse)?;
    if bytes.len() > ZAI_CODING_PLAN_MAX_RESPONSE_BYTES {
        return Err(ZaiCodingPlanAuthError::InvalidResponse);
    }
    let temporary = path.with_extension(format!("tmp-{}", uuid::Uuid::new_v4()));
    let result = (|| -> std::io::Result<()> {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temporary)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        }
        file.write_all(&bytes)?;
        file.sync_all()
    })();
    if let Err(error) = result {
        let _ = std::fs::remove_file(&temporary);
        return Err(ZaiCodingPlanAuthError::Storage(error));
    }
    if let Err(error) = std::fs::rename(&temporary, &path) {
        let _ = std::fs::remove_file(&temporary);
        return Err(ZaiCodingPlanAuthError::Storage(error));
    }
    Ok(())
}

pub fn remove_cache(grok_home: &Path) -> Result<(), ZaiCodingPlanAuthError> {
    match std::fs::remove_file(cache_path(grok_home)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(ZaiCodingPlanAuthError::Storage(error)),
    }
}

pub fn load_cached_model_entries() -> IndexMap<String, ModelEntry> {
    let grok_home = crate::util::grok_home::grok_home();
    let store = super::ZaiCodingPlanCredentialStore::new(&grok_home);
    let credential_id = match store.load() {
        Ok(Some(credentials)) => credentials.credential_id,
        _ => return IndexMap::new(),
    };
    load_cached_model_entries_for(&grok_home, &credential_id)
}

fn load_cached_model_entries_for(
    grok_home: &Path,
    credential_id: &str,
) -> IndexMap<String, ModelEntry> {
    let path = cache_path(grok_home);
    let Ok(metadata) = std::fs::symlink_metadata(&path) else {
        return IndexMap::new();
    };
    if !metadata.file_type().is_file() || metadata.len() > ZAI_CODING_PLAN_MAX_RESPONSE_BYTES as u64
    {
        return IndexMap::new();
    }
    let Ok(bytes) = std::fs::read(path) else {
        return IndexMap::new();
    };
    if bytes.len() > ZAI_CODING_PLAN_MAX_RESPONSE_BYTES {
        return IndexMap::new();
    }
    let Ok(cache) = serde_json::from_slice::<CatalogCache>(&bytes) else {
        return IndexMap::new();
    };
    if cache.schema_version != CACHE_SCHEMA_VERSION || cache.credential_id != credential_id {
        return IndexMap::new();
    }
    map_models(validate_models(cache.models))
}

pub fn map_models(models: Vec<ZaiCodingPlanModel>) -> IndexMap<String, ModelEntry> {
    models.into_iter().map(map_model).collect()
}

fn map_model(model: ZaiCodingPlanModel) -> (String, ModelEntry) {
    let catalog_key = format!("zai-coding-plan/{}", model.id);
    let (qualified, context_window, max_completion_tokens, efforts, default_effort) =
        capability_overlay(&model.id);
    let mut info = ModelInfo::fallback(&model.id);
    info.id = Some(catalog_key.clone());
    info.provider = ProviderId::ZaiCodingPlan;
    info.base_url = ZAI_CODING_PLAN_BASE_URL.to_owned();
    info.name = model.display_name.filter(|name| !name.trim().is_empty());
    info.description = Some(if qualified {
        "Z.AI GLM Coding Plan model (Chat Completions)".to_owned()
    } else {
        "Z.AI Coding Plan catalog model (capabilities not yet qualified)".to_owned()
    });
    info.api_backend = ApiBackend::ChatCompletions;
    info.auth_scheme = AuthScheme::Bearer;
    info.context_window = NonZeroU64::new(context_window).unwrap();
    info.max_completion_tokens = max_completion_tokens;
    info.reasoning_effort = default_effort;
    info.supports_reasoning_effort = efforts.len() > 1;
    info.reasoning_efforts = efforts;
    info.supports_image_input = false;
    info.supports_backend_search = false;
    info.supported_in_api = qualified;
    info.user_selectable = qualified;
    info.hidden = false;
    (
        catalog_key,
        ModelEntry {
            info,
            api_key: None,
            env_key: None,
            api_base_url: None,
        },
    )
}

fn capability_overlay(
    id: &str,
) -> (
    bool,
    u64,
    Option<u32>,
    Vec<ReasoningEffortOption>,
    Option<ReasoningEffort>,
) {
    let known = matches!(id, "glm-5.2" | "glm-5-turbo" | "glm-4.7");
    let context = if id == "glm-5.2" { 1_000_000 } else { 200_000 };
    let max_output = matches!(id, "glm-5.2" | "glm-4.7").then_some(131_072);
    if !known {
        return (false, context, None, Vec::new(), None);
    }
    let levels: &[ReasoningEffort] = if id == "glm-5.2" {
        &[
            ReasoningEffort::None,
            ReasoningEffort::High,
            ReasoningEffort::Max,
        ]
    } else {
        &[ReasoningEffort::None, ReasoningEffort::High]
    };
    let default = if id == "glm-5.2" {
        ReasoningEffort::Max
    } else {
        ReasoningEffort::High
    };
    let efforts = levels
        .iter()
        .copied()
        .map(|value| ReasoningEffortOption {
            id: value.as_str().to_owned(),
            value,
            label: match value {
                ReasoningEffort::None => "Off",
                ReasoningEffort::High => "High",
                ReasoningEffort::Max => "Max",
                _ => unreachable!(),
            }
            .to_owned(),
            description: None,
            default: value == default,
        })
        .collect();
    (known, context, max_output, efforts, Some(default))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glm_52_is_text_only_and_uses_one_million_context() {
        let entries = map_models(vec![ZaiCodingPlanModel {
            id: "glm-5.2".to_owned(),
            display_name: None,
        }]);
        let entry = &entries["zai-coding-plan/glm-5.2"];
        assert_eq!(entry.info.context_window.get(), 1_000_000);
        assert_eq!(entry.info.max_completion_tokens, Some(131_072));
        assert!(!entry.info.supports_image_input);
        assert!(entry.info.user_selectable);
    }

    #[test]
    fn authenticated_unknown_model_is_visible_but_not_selectable() {
        let entries = map_models(vec![ZaiCodingPlanModel {
            id: "future-model".to_owned(),
            display_name: None,
        }]);
        let entry = &entries["zai-coding-plan/future-model"];
        assert!(!entry.info.user_selectable);
        assert!(!entry.info.supported_in_api);
    }
}
