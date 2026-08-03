use std::collections::HashSet;
use std::io::Write;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};
use xai_grok_sampler::AuthScheme;
use xai_grok_sampling_types::{
    ApiBackend, OPENCODE_GO_BASE_URL, OPENCODE_GO_MAX_CATALOG_BYTES,
    OPENCODE_GO_MAX_CATALOG_MODELS, OPENCODE_GO_MODELS_URL, ProviderId, ReasoningEffort,
    ReasoningEffortOption,
};

use super::OpenCodeGoAuthError;
use crate::agent::config::{ModelEntry, ModelInfo};

const CACHE_SCHEMA_VERSION: u32 = 1;
const CACHE_FILE: &str = "opencode-go-models.json";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenCodeGoMediaSupport {
    pub image: bool,
    pub audio: bool,
    pub video: bool,
    pub pdf: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenCodeGoModel {
    pub id: String,
    pub display_name: String,
    pub backend: ApiBackend,
    pub context_length: u64,
    pub output_limit: u64,
    pub media: OpenCodeGoMediaSupport,
    pub deprecated: bool,
}

#[derive(Clone)]
struct Capability {
    id: &'static str,
    name: &'static str,
    backend: ApiBackend,
    context: u64,
    output: u64,
    image: bool,
    audio: bool,
    video: bool,
    pdf: bool,
    deprecated: bool,
}

macro_rules! cap {
    ($id:literal, $name:literal, $backend:ident, $context:expr, $output:expr, [$($media:ident),*], $deprecated:expr) => {{
        let mut value = Capability {
            id: $id,
            name: $name,
            backend: ApiBackend::$backend,
            context: $context,
            output: $output,
            image: false,
            audio: false,
            video: false,
            pdf: false,
            deprecated: $deprecated,
        };
        $(value.$media = true;)*
        value
    }};
}

fn capabilities() -> Vec<Capability> {
    vec![
        cap!(
            "grok-4.5",
            "Grok 4.5",
            ChatCompletions,
            500_000,
            500_000,
            [image],
            false
        ),
        cap!(
            "gpt-5.6-luna",
            "GPT 5.6 Luna",
            Responses,
            1_050_000,
            128_000,
            [image, pdf],
            false
        ),
        cap!(
            "glm-5.2",
            "GLM-5.2",
            ChatCompletions,
            1_000_000,
            131_072,
            [],
            false
        ),
        cap!(
            "glm-5.1",
            "GLM-5.1",
            ChatCompletions,
            202_752,
            32_768,
            [],
            false
        ),
        cap!("glm-5", "GLM-5", ChatCompletions, 202_752, 32_768, [], true),
        cap!(
            "kimi-k3",
            "Kimi K3",
            ChatCompletions,
            1_048_576,
            131_072,
            [image, video],
            false
        ),
        cap!(
            "kimi-k2.7-code",
            "Kimi K2.7 Code",
            ChatCompletions,
            262_144,
            262_144,
            [image, video],
            false
        ),
        cap!(
            "kimi-k2.6",
            "Kimi K2.6",
            ChatCompletions,
            262_144,
            65_536,
            [image, video],
            false
        ),
        cap!(
            "kimi-k2.5",
            "Kimi K2.5",
            ChatCompletions,
            262_144,
            65_536,
            [image, video],
            true
        ),
        cap!(
            "deepseek-v4-pro",
            "DeepSeek V4 Pro",
            ChatCompletions,
            1_000_000,
            384_000,
            [],
            false
        ),
        cap!(
            "deepseek-v4-flash",
            "DeepSeek V4 Flash",
            ChatCompletions,
            1_000_000,
            384_000,
            [],
            false
        ),
        cap!(
            "mimo-v2.5",
            "MiMo V2.5",
            ChatCompletions,
            1_000_000,
            128_000,
            [image, audio, video],
            false
        ),
        cap!(
            "mimo-v2.5-pro",
            "MiMo V2.5 Pro",
            ChatCompletions,
            1_048_576,
            128_000,
            [],
            false
        ),
        cap!(
            "mimo-v2-pro",
            "MiMo V2 Pro",
            ChatCompletions,
            1_048_576,
            128_000,
            [],
            true
        ),
        cap!(
            "mimo-v2-omni",
            "MiMo V2 Omni",
            ChatCompletions,
            262_144,
            128_000,
            [image, audio, pdf],
            true
        ),
        cap!(
            "minimax-m3",
            "MiniMax M3",
            Messages,
            1_000_000,
            131_072,
            [image, video],
            false
        ),
        cap!(
            "minimax-m2.7",
            "MiniMax M2.7",
            Messages,
            204_800,
            131_072,
            [],
            false
        ),
        cap!(
            "minimax-m2.5",
            "MiniMax M2.5",
            Messages,
            204_800,
            65_536,
            [],
            true
        ),
        cap!(
            "qwen3.7-max",
            "Qwen3.7 Max",
            Messages,
            1_000_000,
            65_536,
            [],
            false
        ),
        cap!(
            "qwen3.7-plus",
            "Qwen3.7 Plus",
            Messages,
            1_000_000,
            65_536,
            [image, video],
            false
        ),
        cap!(
            "qwen3.6-plus",
            "Qwen3.6 Plus",
            Messages,
            1_000_000,
            65_536,
            [image, video],
            false
        ),
        cap!(
            "qwen3.5-plus",
            "Qwen3.5 Plus",
            Messages,
            262_144,
            65_536,
            [image, video],
            true
        ),
        cap!("hy3", "Hy3", ChatCompletions, 256_000, 64_000, [], false),
        cap!(
            "hy3-preview",
            "Hy3 Preview",
            ChatCompletions,
            256_000,
            64_000,
            [],
            true
        ),
    ]
}

pub fn media_support(model_id: &str) -> Option<OpenCodeGoMediaSupport> {
    let id = model_id.strip_prefix("opencode-go/").unwrap_or(model_id);
    capabilities()
        .into_iter()
        .find(|capability| capability.id == id)
        .map(|capability| OpenCodeGoMediaSupport {
            image: capability.image,
            audio: capability.audio,
            video: capability.video,
            pdf: capability.pdf,
        })
}

#[derive(Deserialize)]
struct ModelsResponse {
    data: Vec<ModelIdentity>,
}

#[derive(Deserialize)]
struct ModelIdentity {
    id: String,
}

#[derive(Serialize, Deserialize)]
struct CatalogCache {
    schema_version: u32,
    fetched_at: DateTime<Utc>,
    models: Vec<OpenCodeGoModel>,
}

pub async fn fetch_models() -> Result<Vec<OpenCodeGoModel>, OpenCodeGoAuthError> {
    let client = xai_grok_provider_http::with_extra_root_certificates(reqwest::Client::builder())
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|_| OpenCodeGoAuthError::InvalidResponse)?;
    let response = client
        .get(OPENCODE_GO_MODELS_URL)
        .header(ACCEPT, "application/json")
        .header(
            USER_AGENT,
            format!("grok-agent/{}", xai_grok_version::VERSION),
        )
        .send()
        .await
        .map_err(|_| OpenCodeGoAuthError::InvalidResponse)?;
    if !response.status().is_success() {
        return Err(OpenCodeGoAuthError::Http(response.status()));
    }
    if response
        .content_length()
        .is_some_and(|length| length > OPENCODE_GO_MAX_CATALOG_BYTES as u64)
    {
        return Err(OpenCodeGoAuthError::InvalidResponse);
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|_| OpenCodeGoAuthError::InvalidResponse)?;
    if bytes.len() > OPENCODE_GO_MAX_CATALOG_BYTES {
        return Err(OpenCodeGoAuthError::InvalidResponse);
    }
    let response: ModelsResponse =
        serde_json::from_slice(&bytes).map_err(|_| OpenCodeGoAuthError::InvalidResponse)?;
    if response.data.len() > OPENCODE_GO_MAX_CATALOG_MODELS {
        return Err(OpenCodeGoAuthError::InvalidResponse);
    }
    intersect_catalog(response.data)
}

fn intersect_catalog(
    values: Vec<ModelIdentity>,
) -> Result<Vec<OpenCodeGoModel>, OpenCodeGoAuthError> {
    let mut live = HashSet::new();
    for value in values {
        let valid = !value.id.is_empty()
            && value.id.len() <= 128
            && value
                .id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'));
        if valid {
            live.insert(value.id);
        }
    }
    let models = capabilities()
        .into_iter()
        .filter(|capability| live.contains(capability.id))
        .map(|capability| OpenCodeGoModel {
            id: capability.id.to_owned(),
            display_name: capability.name.to_owned(),
            backend: capability.backend,
            context_length: capability.context,
            output_limit: capability.output,
            media: OpenCodeGoMediaSupport {
                image: capability.image,
                audio: capability.audio,
                video: capability.video,
                pdf: capability.pdf,
            },
            deprecated: capability.deprecated,
        })
        .collect::<Vec<_>>();
    if models.iter().all(|model| model.deprecated) {
        return Err(OpenCodeGoAuthError::EmptyCatalog);
    }
    Ok(models)
}

pub fn cache_path(grok_home: &Path) -> PathBuf {
    grok_home.join("cache").join(CACHE_FILE)
}

pub fn save_cache(grok_home: &Path, models: &[OpenCodeGoModel]) -> Result<(), OpenCodeGoAuthError> {
    let path = cache_path(grok_home);
    let parent = path.parent().ok_or_else(|| {
        OpenCodeGoAuthError::Storage(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "invalid OpenCode Go cache path",
        ))
    })?;
    std::fs::create_dir_all(parent)?;
    let bytes = serde_json::to_vec_pretty(&CatalogCache {
        schema_version: CACHE_SCHEMA_VERSION,
        fetched_at: Utc::now(),
        models: models.to_vec(),
    })
    .map_err(|_| OpenCodeGoAuthError::InvalidResponse)?;
    if bytes.len() > OPENCODE_GO_MAX_CATALOG_BYTES {
        return Err(OpenCodeGoAuthError::InvalidResponse);
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
        file.write_all(&bytes)?;
        file.sync_all()
    })();
    if let Err(error) = result {
        let _ = std::fs::remove_file(&temporary);
        return Err(OpenCodeGoAuthError::Storage(error));
    }
    if let Err(error) = std::fs::rename(&temporary, &path) {
        let _ = std::fs::remove_file(&temporary);
        return Err(OpenCodeGoAuthError::Storage(error));
    }
    Ok(())
}

pub fn load_cached_model_entries() -> IndexMap<String, ModelEntry> {
    let path = cache_path(&crate::util::grok_home::grok_home());
    let Ok(metadata) = std::fs::symlink_metadata(&path) else {
        return IndexMap::new();
    };
    if !metadata.file_type().is_file() || metadata.len() > OPENCODE_GO_MAX_CATALOG_BYTES as u64 {
        return IndexMap::new();
    }
    let Ok(bytes) = std::fs::read(path) else {
        return IndexMap::new();
    };
    let Ok(cache) = serde_json::from_slice::<CatalogCache>(&bytes) else {
        return IndexMap::new();
    };
    if cache.schema_version != CACHE_SCHEMA_VERSION {
        return IndexMap::new();
    }
    map_models(cache.models)
}

pub fn map_models(models: Vec<OpenCodeGoModel>) -> IndexMap<String, ModelEntry> {
    models
        .into_iter()
        .filter_map(map_model)
        .collect::<IndexMap<_, _>>()
}

fn map_model(model: OpenCodeGoModel) -> Option<(String, ModelEntry)> {
    let context_window = NonZeroU64::new(model.context_length)?;
    let catalog_key = format!("opencode-go/{}", model.id);
    let mut info = ModelInfo::fallback(&model.id);
    info.id = Some(catalog_key.clone());
    info.provider = ProviderId::OpenCodeGo;
    info.base_url = OPENCODE_GO_BASE_URL.to_owned();
    info.name = Some(model.display_name);
    info.description = Some(format!(
        "OpenCode Go subscription model ({:?} protocol){}",
        model.backend,
        if model.deprecated { "; deprecated" } else { "" }
    ));
    info.api_backend = model.backend.clone();
    info.auth_scheme = if model.backend == ApiBackend::Messages {
        AuthScheme::XApiKey
    } else {
        AuthScheme::Bearer
    };
    info.context_window = context_window;
    info.max_completion_tokens = u32::try_from(model.output_limit).ok();
    info.reasoning_effort = Some(ReasoningEffort::High);
    info.reasoning_efforts = vec![ReasoningEffortOption {
        id: "high".to_owned(),
        value: ReasoningEffort::High,
        label: "High".to_owned(),
        description: None,
        default: true,
    }];
    info.supports_image_input = model.media.image;
    info.supported_in_api = true;
    info.user_selectable = !model.deprecated;
    info.hidden = model.deprecated;
    Some((
        catalog_key,
        ModelEntry {
            info,
            api_key: None,
            env_key: None,
            auth_provider: None,
            api_base_url: None,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_catalog_is_intersected_with_audited_routes() {
        let models = intersect_catalog(vec![
            ModelIdentity {
                id: "gpt-5.6-luna".to_owned(),
            },
            ModelIdentity {
                id: "future-model".to_owned(),
            },
            ModelIdentity {
                id: "minimax-m2.7".to_owned(),
            },
        ])
        .unwrap();
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].backend, ApiBackend::Responses);
        assert_eq!(models[1].backend, ApiBackend::Messages);
    }

    #[test]
    fn model_names_remain_owned_by_open_code_go() {
        let models = map_models(vec![OpenCodeGoModel {
            id: "grok-4.5".to_owned(),
            display_name: "Grok 4.5".to_owned(),
            backend: ApiBackend::ChatCompletions,
            context_length: 500_000,
            output_limit: 500_000,
            media: OpenCodeGoMediaSupport {
                image: true,
                ..Default::default()
            },
            deprecated: false,
        }]);
        assert_eq!(
            models["opencode-go/grok-4.5"].info.provider,
            ProviderId::OpenCodeGo
        );
    }
}
