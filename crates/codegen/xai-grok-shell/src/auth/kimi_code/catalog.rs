use std::collections::HashSet;
use std::io::Write;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};
use xai_grok_sampler::AuthScheme;
use xai_grok_sampling_types::{
    ApiBackend, KIMI_CODE_BASE_URL, KIMI_CODE_MODELS_URL, ProviderId, ReasoningEffort,
    ReasoningEffortOption,
};

use super::{KimiCodeAuthError, KimiCodeCredentials};
use crate::agent::config::{ModelEntry, ModelInfo};

const CACHE_SCHEMA_VERSION: u32 = 1;
const MAX_CATALOG_BYTES: usize = 2 * 1024 * 1024;
const CACHE_FILE: &str = "kimi-code-models.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KimiCodeModel {
    pub id: String,
    pub context_length: u64,
    #[serde(default)]
    pub supports_reasoning: bool,
    #[serde(default)]
    pub supports_image_in: bool,
    #[serde(default)]
    pub supports_video_in: bool,
    #[serde(default = "default_true")]
    pub supports_tool_use: bool,
    #[serde(default)]
    pub supports_thinking_type: Option<String>,
    #[serde(default)]
    pub think_efforts: Option<KimiCodeThinkEfforts>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub protocol: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct KimiCodeThinkEfforts {
    #[serde(default)]
    pub support: bool,
    #[serde(default)]
    pub valid_efforts: Vec<String>,
    #[serde(default)]
    pub default_effort: Option<String>,
}

#[derive(Deserialize)]
struct KimiCodeModelsResponse {
    data: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct KimiCodeCatalogCache {
    schema_version: u32,
    credential_id: String,
    fetched_at: DateTime<Utc>,
    models: Vec<KimiCodeModel>,
}

pub async fn fetch_models(
    credentials: &KimiCodeCredentials,
) -> Result<Vec<KimiCodeModel>, KimiCodeAuthError> {
    credentials.validate_persisted()?;
    let client = super::transport::http_client(std::time::Duration::from_secs(30))?;
    let mut authorization = HeaderValue::from_str(&format!("Bearer {}", credentials.api_key()))
        .map_err(|_| KimiCodeAuthError::InvalidCredential)?;
    authorization.set_sensitive(true);
    let response = client
        .get(KIMI_CODE_MODELS_URL)
        .header(AUTHORIZATION, authorization)
        .header(ACCEPT, "application/json")
        .header(
            USER_AGENT,
            format!("grok-agent/{}", xai_grok_version::VERSION),
        )
        .send()
        .await
        .map_err(|_| KimiCodeAuthError::InvalidResponse)?;
    let status = response.status();
    if !status.is_success() {
        return Err(KimiCodeAuthError::Http(status));
    }
    let bytes = super::transport::read_limited_body(response, MAX_CATALOG_BYTES).await?;
    let response: KimiCodeModelsResponse =
        serde_json::from_slice(&bytes).map_err(|_| KimiCodeAuthError::InvalidResponse)?;
    require_usable_models(validate_models(parse_model_values(response.data))?)
}

fn parse_model_values(values: Vec<serde_json::Value>) -> Vec<KimiCodeModel> {
    values.into_iter().filter_map(parse_model_value).collect()
}

fn parse_model_value(value: serde_json::Value) -> Option<KimiCodeModel> {
    let object = value.as_object()?;
    let id = object.get("id")?.as_str()?.to_owned();
    let context_length = object.get("context_length")?.as_u64()?;
    let optional_bool = |field: &str, fallback: bool| {
        object
            .get(field)
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(fallback)
    };
    let optional_string = |field: &str| {
        object
            .get(field)
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
    };
    let think_efforts = object
        .get("think_efforts")
        .and_then(serde_json::Value::as_object)
        .map(|efforts| KimiCodeThinkEfforts {
            support: efforts
                .get("support")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            valid_efforts: efforts
                .get("valid_efforts")
                .and_then(serde_json::Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(serde_json::Value::as_str)
                        .filter(|value| !value.trim().is_empty())
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default(),
            default_effort: efforts
                .get("default_effort")
                .and_then(serde_json::Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .map(str::to_owned),
        });

    Some(KimiCodeModel {
        id,
        context_length,
        supports_reasoning: optional_bool("supports_reasoning", false),
        supports_image_in: optional_bool("supports_image_in", false),
        supports_video_in: optional_bool("supports_video_in", false),
        supports_tool_use: optional_bool("supports_tool_use", true),
        supports_thinking_type: optional_string("supports_thinking_type"),
        think_efforts,
        display_name: optional_string("display_name"),
        protocol: optional_string("protocol"),
    })
}

fn require_usable_models(
    models: Vec<KimiCodeModel>,
) -> Result<Vec<KimiCodeModel>, KimiCodeAuthError> {
    let models: Vec<_> = models
        .into_iter()
        .filter(|model| model.supports_tool_use)
        .collect();
    if models.is_empty() {
        return Err(KimiCodeAuthError::EmptyCatalog);
    }
    Ok(models)
}

fn validate_models(models: Vec<KimiCodeModel>) -> Result<Vec<KimiCodeModel>, KimiCodeAuthError> {
    let mut validated = Vec::with_capacity(models.len());
    let mut model_ids = HashSet::with_capacity(models.len());
    for mut model in models {
        let display_name_is_invalid = model.display_name.as_deref().is_some_and(|name| {
            name.len() > 512 || name.trim().is_empty() || name.chars().any(char::is_control)
        });
        if model.id.trim().is_empty()
            || model.id.len() > 256
            || model.id.chars().any(char::is_control)
            || model.id.chars().any(char::is_whitespace)
            || !model_ids.insert(model.id.clone())
            || model.context_length == 0
            || display_name_is_invalid
        {
            // One malformed optional catalog entry must not hide unrelated
            // entitled models. Invalid essential identity fields are skipped.
            continue;
        }
        if !matches!(
            model.protocol.as_deref(),
            None | Some("kimi") | Some("anthropic")
        ) {
            // Official Kimi treats only `anthropic` specially; unknown values
            // retain the default Chat Completions route.
            model.protocol = None;
        }
        if !matches!(
            model.supports_thinking_type.as_deref(),
            None | Some("only") | Some("no") | Some("both")
        ) {
            // Forward-compatible optional drift falls back to the legacy
            // `supports_reasoning` capability.
            model.supports_thinking_type = None;
        }
        validated.push(model);
    }
    Ok(validated)
}

pub fn cache_path(grok_home: &Path) -> PathBuf {
    grok_home.join("cache").join(CACHE_FILE)
}

pub fn save_cache(
    grok_home: &Path,
    credentials: &KimiCodeCredentials,
    models: &[KimiCodeModel],
) -> Result<(), KimiCodeAuthError> {
    save_cache_for_credential_id(grok_home, &credentials.credential_id, models)
}

fn save_cache_for_credential_id(
    grok_home: &Path,
    credential_id: &str,
    models: &[KimiCodeModel],
) -> Result<(), KimiCodeAuthError> {
    let path = cache_path(grok_home);
    let parent = path.parent().ok_or_else(|| {
        KimiCodeAuthError::Storage(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "invalid Kimi Code cache path",
        ))
    })?;
    std::fs::create_dir_all(parent)?;
    let cache = KimiCodeCatalogCache {
        schema_version: CACHE_SCHEMA_VERSION,
        credential_id: credential_id.to_owned(),
        fetched_at: Utc::now(),
        models: models.to_vec(),
    };
    let bytes =
        serde_json::to_vec_pretty(&cache).map_err(|_| KimiCodeAuthError::InvalidResponse)?;
    if bytes.len() > MAX_CATALOG_BYTES {
        return Err(KimiCodeAuthError::InvalidResponse);
    }
    let temporary = path.with_extension(format!("tmp-{}", uuid::Uuid::new_v4()));
    let write_result = (|| -> std::io::Result<()> {
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
    if let Err(error) = write_result {
        let _ = std::fs::remove_file(&temporary);
        return Err(KimiCodeAuthError::Storage(error));
    }
    if let Err(error) = std::fs::rename(&temporary, &path) {
        let _ = std::fs::remove_file(&temporary);
        return Err(KimiCodeAuthError::Storage(error));
    }
    Ok(())
}

pub fn remove_cache(grok_home: &Path) -> Result<(), KimiCodeAuthError> {
    match std::fs::remove_file(cache_path(grok_home)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(KimiCodeAuthError::Storage(error)),
    }
}

pub fn load_cached_model_entries() -> IndexMap<String, ModelEntry> {
    let grok_home = crate::util::grok_home::grok_home();
    let store = super::KimiCodeCredentialStore::new(&grok_home);
    let credential_id = match store.load() {
        Ok(Some(credentials)) => credentials.credential_id,
        // Environment-only credentials are process-local and deliberately do
        // not own a durable entitlement cache.
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
    if !metadata.file_type().is_file() || metadata.len() > MAX_CATALOG_BYTES as u64 {
        return IndexMap::new();
    }
    let Ok(bytes) = std::fs::read(path) else {
        return IndexMap::new();
    };
    if bytes.len() > MAX_CATALOG_BYTES {
        return IndexMap::new();
    }
    let Ok(cache) = serde_json::from_slice::<KimiCodeCatalogCache>(&bytes) else {
        return IndexMap::new();
    };
    if cache.schema_version != CACHE_SCHEMA_VERSION || cache.credential_id != credential_id {
        return IndexMap::new();
    }
    let Ok(models) = validate_models(cache.models) else {
        return IndexMap::new();
    };
    map_models(models)
}

pub fn map_models(models: Vec<KimiCodeModel>) -> IndexMap<String, ModelEntry> {
    models
        .into_iter()
        .filter_map(map_model)
        .collect::<IndexMap<_, _>>()
}

fn map_model(model: KimiCodeModel) -> Option<(String, ModelEntry)> {
    // Grok Build's preserved agent loop always advertises native tools. Models
    // that explicitly reject tool use cannot safely enter that loop.
    if !model.supports_tool_use {
        return None;
    }
    let context_window = NonZeroU64::new(model.context_length)?;
    let catalog_key = format!("kimi-code/{}", model.id);
    let backend = if model.protocol.as_deref() == Some("anthropic") {
        ApiBackend::Messages
    } else {
        ApiBackend::ChatCompletions
    };
    let auth_scheme = if backend == ApiBackend::Messages {
        AuthScheme::XApiKey
    } else {
        AuthScheme::Bearer
    };
    let (reasoning_effort, reasoning_efforts) = reasoning_options(&model);
    let mut info = ModelInfo::fallback(&model.id);
    info.id = Some(catalog_key.clone());
    info.provider = ProviderId::KimiCode;
    info.base_url = KIMI_CODE_BASE_URL.to_owned();
    info.name = model.display_name.filter(|name| !name.trim().is_empty());
    info.description = Some(match backend {
        ApiBackend::Messages => "Kimi Code membership model (Messages protocol)".to_owned(),
        _ => "Kimi Code membership model (Chat Completions protocol)".to_owned(),
    });
    info.api_backend = backend;
    info.auth_scheme = auth_scheme;
    info.context_window = context_window;
    info.reasoning_effort = reasoning_effort;
    info.supports_reasoning_effort = reasoning_efforts.len() > 1;
    info.reasoning_efforts = reasoning_efforts;
    info.supports_image_input = model.supports_image_in;
    info.supports_backend_search = false;
    info.supported_in_api = true;
    info.user_selectable = true;
    info.hidden = false;
    Some((
        catalog_key,
        ModelEntry {
            info,
            api_key: None,
            env_key: None,
            api_base_url: None,
        },
    ))
}

fn reasoning_options(
    model: &KimiCodeModel,
) -> (Option<ReasoningEffort>, Vec<ReasoningEffortOption>) {
    let thinking_type = model.supports_thinking_type.as_deref();
    if thinking_type == Some("no") || (!model.supports_reasoning && thinking_type.is_none()) {
        return (Some(ReasoningEffort::None), Vec::new());
    }
    let efforts = model
        .think_efforts
        .as_ref()
        .filter(|efforts| efforts.support)
        .map(|efforts| efforts.valid_efforts.as_slice())
        .unwrap_or(&[]);
    let mut values = Vec::new();
    if thinking_type != Some("only") {
        values.push(ReasoningEffort::None);
    }
    for raw in efforts {
        if let Some(effort) = parse_effort(raw)
            && !values.contains(&effort)
        {
            values.push(effort);
        }
    }
    if !values.iter().any(|effort| *effort != ReasoningEffort::None) {
        // Older catalogs expose only `supports_reasoning=true`. Preserve an
        // enabled choice instead of accidentally presenting such models as
        // Off-only when no effort list is available yet.
        values.push(ReasoningEffort::High);
    }
    let advertised_default = model
        .think_efforts
        .as_ref()
        .filter(|efforts| efforts.support)
        .and_then(|efforts| efforts.default_effort.as_deref())
        .and_then(parse_effort)
        .filter(|effort| values.contains(effort));
    let default = advertised_default
        .or_else(|| {
            values
                .contains(&ReasoningEffort::High)
                .then_some(ReasoningEffort::High)
        })
        .or_else(|| {
            values
                .iter()
                .copied()
                .find(|effort| *effort != ReasoningEffort::None)
        })
        .or(Some(ReasoningEffort::None));
    let options = values
        .into_iter()
        .map(|effort| ReasoningEffortOption {
            id: effort.as_str().to_owned(),
            value: effort,
            label: effort_label(effort).to_owned(),
            description: None,
            default: Some(effort) == default,
        })
        .collect();
    (default, options)
}

fn parse_effort(value: &str) -> Option<ReasoningEffort> {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" => Some(ReasoningEffort::None),
        "minimal" => Some(ReasoningEffort::Minimal),
        "low" => Some(ReasoningEffort::Low),
        "medium" => Some(ReasoningEffort::Medium),
        "high" => Some(ReasoningEffort::High),
        "xhigh" => Some(ReasoningEffort::Xhigh),
        "max" => Some(ReasoningEffort::Max),
        "ultra" => Some(ReasoningEffort::Ultra),
        _ => None,
    }
}

fn effort_label(effort: ReasoningEffort) -> &'static str {
    match effort {
        ReasoningEffort::None => "Off",
        ReasoningEffort::Minimal => "Minimal",
        ReasoningEffort::Low => "Low",
        ReasoningEffort::Medium => "Medium",
        ReasoningEffort::High => "High",
        ReasoningEffort::Xhigh => "Extra High",
        ReasoningEffort::Max => "Max",
        ReasoningEffort::Ultra => "Ultra",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_routes_an_unknown_optional_protocol_through_chat_completions() {
        let model = KimiCodeModel {
            id: "future-model".to_owned(),
            context_length: 262_144,
            supports_reasoning: true,
            supports_image_in: false,
            supports_video_in: false,
            supports_tool_use: true,
            supports_thinking_type: Some("both".to_owned()),
            think_efforts: None,
            display_name: Some("Future model".to_owned()),
            protocol: Some("future-protocol".to_owned()),
        };

        let validated = validate_models(vec![model]).unwrap();
        let models = map_models(validated);

        assert_eq!(
            models["kimi-code/future-model"].info.api_backend,
            ApiBackend::ChatCompletions
        );
    }

    #[test]
    fn catalog_keeps_a_valid_model_when_a_sibling_has_an_invalid_identity() {
        let invalid = KimiCodeModel {
            id: "bad model".to_owned(),
            context_length: 262_144,
            supports_reasoning: false,
            supports_image_in: false,
            supports_video_in: false,
            supports_tool_use: true,
            supports_thinking_type: None,
            think_efforts: None,
            display_name: Some("Invalid".to_owned()),
            protocol: None,
        };
        let valid = KimiCodeModel {
            id: "k3".to_owned(),
            context_length: 262_144,
            supports_reasoning: true,
            supports_image_in: false,
            supports_video_in: false,
            supports_tool_use: true,
            supports_thinking_type: None,
            think_efforts: None,
            display_name: Some("K3".to_owned()),
            protocol: Some("kimi".to_owned()),
        };

        let validated = validate_models(vec![invalid, valid]).unwrap();

        assert_eq!(validated.len(), 1);
        assert_eq!(validated[0].id, "k3");
    }

    #[test]
    fn catalog_decode_keeps_a_model_when_optional_fields_drift() {
        let parsed = parse_model_values(vec![serde_json::json!({
            "id": "k3",
            "context_length": 262144,
            "supports_reasoning": "yes",
            "supports_image_in": null,
            "supports_tool_use": {"unexpected": true},
            "think_efforts": {
                "support": true,
                "valid_efforts": ["low", 42, "high"],
                "default_effort": ["high"]
            },
            "display_name": 42,
            "protocol": {"future": true}
        })]);

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].id, "k3");
        assert!(!parsed[0].supports_reasoning);
        assert!(parsed[0].supports_tool_use);
        assert_eq!(
            parsed[0].think_efforts.as_ref().unwrap().valid_efforts,
            ["low", "high"]
        );
    }

    #[test]
    fn legacy_reasoning_capability_includes_an_enabled_effort() {
        let models = map_models(vec![KimiCodeModel {
            id: "legacy-thinking".to_owned(),
            context_length: 262_144,
            supports_reasoning: true,
            supports_image_in: false,
            supports_video_in: false,
            supports_tool_use: true,
            supports_thinking_type: None,
            think_efforts: None,
            display_name: None,
            protocol: None,
        }]);

        let entry = &models["kimi-code/legacy-thinking"];

        assert_eq!(entry.info.reasoning_effort, Some(ReasoningEffort::High));
        assert!(
            entry
                .info
                .reasoning_efforts
                .iter()
                .any(|option| { option.value == ReasoningEffort::High && option.default == true })
        );
    }

    #[test]
    fn catalog_rejects_entitlement_without_a_tool_compatible_model() {
        let result = require_usable_models(vec![KimiCodeModel {
            id: "chat-only".to_owned(),
            context_length: 262_144,
            supports_reasoning: false,
            supports_image_in: false,
            supports_video_in: false,
            supports_tool_use: false,
            supports_thinking_type: Some("no".to_owned()),
            think_efforts: None,
            display_name: Some("Chat only".to_owned()),
            protocol: Some("kimi".to_owned()),
        }]);

        assert!(matches!(result, Err(KimiCodeAuthError::EmptyCatalog)));
    }

    #[test]
    fn catalog_omits_models_that_cannot_run_grok_tools() {
        let models = map_models(vec![KimiCodeModel {
            id: "chat-only".to_owned(),
            context_length: 262_144,
            supports_reasoning: false,
            supports_image_in: false,
            supports_video_in: false,
            supports_tool_use: false,
            supports_thinking_type: Some("no".to_owned()),
            think_efforts: None,
            display_name: Some("Chat only".to_owned()),
            protocol: Some("kimi".to_owned()),
        }]);
        assert!(models.is_empty());
    }

    #[test]
    fn catalog_protocol_is_authoritative_and_models_are_namespaced() {
        let models = map_models(vec![KimiCodeModel {
            id: "kimi-k2".to_owned(),
            context_length: 262_144,
            supports_reasoning: true,
            supports_image_in: true,
            supports_video_in: false,
            supports_tool_use: true,
            supports_thinking_type: Some("both".to_owned()),
            think_efforts: Some(KimiCodeThinkEfforts {
                support: true,
                valid_efforts: vec!["low".to_owned(), "high".to_owned(), "max".to_owned()],
                default_effort: Some("high".to_owned()),
            }),
            display_name: Some("Kimi K2".to_owned()),
            protocol: Some("anthropic".to_owned()),
        }]);
        let entry = &models["kimi-code/kimi-k2"];
        assert_eq!(entry.info.provider, ProviderId::KimiCode);
        assert_eq!(entry.info.api_backend, ApiBackend::Messages);
        assert_eq!(entry.info.auth_scheme, AuthScheme::XApiKey);
        assert!(entry.info.supports_image_input);
        assert_eq!(entry.info.reasoning_effort, Some(ReasoningEffort::High));
    }

    #[cfg(unix)]
    #[test]
    fn saved_catalog_cache_is_owner_only_and_bound_to_its_credential_record() {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempfile::tempdir().unwrap();
        let credentials = KimiCodeCredentials::new("sentinel-kimi-key").unwrap();
        let models = vec![KimiCodeModel {
            id: "k3".to_owned(),
            context_length: 262_144,
            supports_reasoning: true,
            supports_image_in: false,
            supports_video_in: false,
            supports_tool_use: true,
            supports_thinking_type: Some("both".to_owned()),
            think_efforts: None,
            display_name: Some("K3".to_owned()),
            protocol: Some("kimi".to_owned()),
        }];

        save_cache(directory.path(), &credentials, &models).unwrap();

        let mode = std::fs::metadata(cache_path(directory.path()))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
        assert!(
            load_cached_model_entries_for(directory.path(), &credentials.credential_id)
                .contains_key("kimi-code/k3")
        );
        assert!(load_cached_model_entries_for(directory.path(), "different-record").is_empty());
    }

    #[test]
    fn oversized_catalog_cache_is_ignored_before_json_parsing() {
        let directory = tempfile::tempdir().unwrap();
        let path = cache_path(directory.path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, vec![b' '; MAX_CATALOG_BYTES + 1]).unwrap();

        let models = load_cached_model_entries_for(directory.path(), "opaque-record");

        assert!(models.is_empty());
    }
}
