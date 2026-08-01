use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use tokio::io::AsyncReadExt;
use xai_grok_sampling_types::{
    CredentialBinding, CredentialSourceId, ProviderId, Result, SamplingError,
};

pub(crate) const MAX_VIDEO_BYTES: u64 = 100 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct CredentialKey {
    provider: ProviderId,
    source: CredentialSourceId,
    record_id: Option<String>,
    generation: u64,
}

impl From<&CredentialBinding> for CredentialKey {
    fn from(binding: &CredentialBinding) -> Self {
        Self {
            provider: binding.provider,
            source: binding.source,
            record_id: binding.record_id.clone(),
            generation: binding.generation,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct FileIdentity {
    canonical_path: PathBuf,
    len: u64,
    modified: Option<SystemTime>,
}

#[derive(Clone, Debug)]
pub(crate) struct PreparedVideo {
    pub(crate) identity: FileIdentity,
    pub(crate) bytes: Vec<u8>,
    pub(crate) file_name: String,
    pub(crate) mime_type: &'static str,
}

#[derive(Clone, Default)]
pub(crate) struct UploadCache {
    inner: Arc<Mutex<HashMap<(CredentialKey, FileIdentity), String>>>,
}

impl UploadCache {
    pub(crate) fn get(
        &self,
        binding: &CredentialBinding,
        identity: &FileIdentity,
    ) -> Option<String> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&(CredentialKey::from(binding), identity.clone()))
            .cloned()
    }

    pub(crate) fn insert(
        &self,
        binding: &CredentialBinding,
        identity: FileIdentity,
        remote_id: String,
    ) {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert((CredentialKey::from(binding), identity), remote_id);
    }
}

pub(crate) fn accepted_mime(path: &Path, declared: &str) -> Result<&'static str> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase);
    let expected = match extension.as_deref() {
        Some("mp4") => "video/mp4",
        Some("mpeg" | "mpg") => "video/mpeg",
        Some("mov" | "qt") => "video/quicktime",
        Some("webm") => "video/webm",
        Some("mkv") => "video/x-matroska",
        Some("avi") => "video/x-msvideo",
        Some("flv") => "video/x-flv",
        Some("3gp" | "3gpp") => "video/3gpp",
        _ => {
            return Err(SamplingError::InvalidConfiguration(
                "unsupported Kimi video format",
            ));
        }
    };
    if declared.trim().eq_ignore_ascii_case(expected) {
        Ok(expected)
    } else {
        Err(SamplingError::InvalidConfiguration(
            "Kimi video MIME type does not match its file extension",
        ))
    }
}

pub(crate) async fn prepare(path: &str, declared_mime: &str) -> Result<PreparedVideo> {
    if path.starts_with("ms://") {
        return Err(SamplingError::InvalidConfiguration(
            "persisted Kimi video upload identifiers are not accepted",
        ));
    }
    let canonical_path = tokio::fs::canonicalize(path).await.map_err(|_| {
        SamplingError::InvalidConfiguration("Kimi video file is missing or unreadable")
    })?;
    let mime_type = accepted_mime(&canonical_path, declared_mime)?;
    let metadata = tokio::fs::metadata(&canonical_path).await.map_err(|_| {
        SamplingError::InvalidConfiguration("Kimi video file is missing or unreadable")
    })?;
    if !metadata.is_file() {
        return Err(SamplingError::InvalidConfiguration(
            "Kimi video input must be a regular file",
        ));
    }
    if metadata.len() > MAX_VIDEO_BYTES {
        return Err(SamplingError::InvalidConfiguration(
            "Kimi video file exceeds the 100 MiB limit",
        ));
    }
    let identity = FileIdentity {
        canonical_path: canonical_path.clone(),
        len: metadata.len(),
        modified: metadata.modified().ok(),
    };
    let file = tokio::fs::File::open(&canonical_path).await.map_err(|_| {
        SamplingError::InvalidConfiguration("Kimi video file is missing or unreadable")
    })?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_VIDEO_BYTES + 1)
        .read_to_end(&mut bytes)
        .await
        .map_err(|_| {
            SamplingError::InvalidConfiguration("Kimi video file is missing or unreadable")
        })?;
    if bytes.len() as u64 > MAX_VIDEO_BYTES {
        return Err(SamplingError::InvalidConfiguration(
            "Kimi video file exceeds the 100 MiB limit",
        ));
    }
    let final_metadata = tokio::fs::metadata(&canonical_path).await.map_err(|_| {
        SamplingError::InvalidConfiguration("Kimi video file changed while being prepared")
    })?;
    if final_metadata.len() != identity.len
        || final_metadata.modified().ok() != identity.modified
        || bytes.len() as u64 != identity.len
    {
        return Err(SamplingError::InvalidConfiguration(
            "Kimi video file changed while being prepared",
        ));
    }
    let file_name = canonical_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("video")
        .to_owned();
    Ok(PreparedVideo {
        identity,
        bytes,
        file_name,
        mime_type,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_declared_supported_video_formats() {
        for (name, mime) in [
            ("a.mp4", "video/mp4"),
            ("a.mpeg", "video/mpeg"),
            ("a.mov", "video/quicktime"),
            ("a.webm", "video/webm"),
            ("a.mkv", "video/x-matroska"),
            ("a.avi", "video/x-msvideo"),
            ("a.flv", "video/x-flv"),
            ("a.3gp", "video/3gpp"),
        ] {
            assert_eq!(accepted_mime(Path::new(name), mime).unwrap(), mime);
        }
        assert!(accepted_mime(Path::new("a.txt"), "video/mp4").is_err());
        assert!(accepted_mime(Path::new("a.mp4"), "video/webm").is_err());
    }

    #[tokio::test]
    async fn missing_and_oversized_files_fail_before_reading_payload_bytes() {
        let missing =
            std::env::temp_dir().join(format!("grok-kimi-missing-{}.mp4", uuid::Uuid::new_v4()));
        let error = prepare(missing.to_str().unwrap(), "video/mp4")
            .await
            .unwrap_err();
        assert!(error.to_string().contains("missing or unreadable"));

        let oversized =
            std::env::temp_dir().join(format!("grok-kimi-oversized-{}.mp4", uuid::Uuid::new_v4()));
        let file = std::fs::File::create(&oversized).unwrap();
        file.set_len(MAX_VIDEO_BYTES + 1).unwrap();
        drop(file);
        let error = prepare(oversized.to_str().unwrap(), "video/mp4")
            .await
            .unwrap_err();
        assert!(error.to_string().contains("100 MiB"));
        std::fs::remove_file(oversized).unwrap();
    }
}
