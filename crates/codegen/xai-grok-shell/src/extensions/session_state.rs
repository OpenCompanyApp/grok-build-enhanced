//! `x.ai/session/state` reads a session's metadata columns; `x.ai/session/import`
//! writes them, with the transcript, to recreate a session on another host.

use std::path::{Path, PathBuf};

use agent_client_protocol as acp;
use serde::Deserialize;
use serde_json::{Value, json};

use super::ExtResult;
use crate::session::persistence::Summary;
use crate::session::storage as st;

/// The summary column, required to load a session.
const SUMMARY_COLUMN: &str = "summary";
/// Compaction checkpoint files keyed by their UUID. They are mirrored with
/// metadata so imported sessions retain compacted context and rewind support.
const COMPACTION_CHECKPOINTS_COLUMN: &str = "compactionCheckpoints";

/// Logical column name to its file under the session directory. Paths come from the
/// storage layer so import and load never disagree about the on-disk layout. `summary`
/// is last so import writes it last, as the commit marker; keep it there.
const COLUMNS: &[(&str, &str)] = &[
    ("plan", st::PLAN_FILE),
    ("planMode", st::PLAN_MODE_FILE),
    ("signals", st::SIGNALS_FILE),
    ("goal", st::GOAL_STATE_FILE),
    ("announcement", st::ANNOUNCEMENT_STATE_FILE),
    (SUMMARY_COLUMN, st::SUMMARY_FILE),
];

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct StateRequest {
    session_id: String,
    cwd: String,
}

/// A session id is a UUID (see acp_agent's new_session); requiring that keeps it safe
/// to join into a filesystem path.
fn validate_session_uuid(session_id: &str) -> Result<(), acp::Error> {
    uuid::Uuid::try_parse(session_id)
        .map(|_| ())
        .map_err(|_| acp::Error::invalid_params().data("sessionId must be a UUID"))
}

/// `x.ai/session/state`: return metadata columns keyed by logical name. Errors when
/// the session isn't found on this host, since it reads a single record whose absence
/// is not an empty result (unlike the collection returned by `x.ai/session/updates`).
pub async fn handle_state(args: &acp::ExtRequest) -> ExtResult {
    let request: StateRequest = super::parse_params(args)?;
    validate_session_uuid(&request.session_id)?;

    let Some(dir) = resolve_session_dir(&request.session_id, &request.cwd) else {
        return Err(acp::Error::invalid_params().data("session not found"));
    };
    let mut state = serde_json::Map::new();
    for (column, rel) in COLUMNS {
        if let Ok(text) = std::fs::read_to_string(dir.join(rel))
            && let Ok(mut value) = serde_json::from_str::<Value>(&text)
        {
            strip_local_provider_metadata(&mut value);
            state.insert((*column).to_string(), value);
        }
    }
    let mut checkpoints = read_compaction_checkpoints(&dir)
        .map_err(|error| acp::Error::internal_error().data(error.to_string()))?;
    if checkpoints
        .as_object()
        .is_some_and(|checkpoints| !checkpoints.is_empty())
    {
        strip_local_provider_metadata(&mut checkpoints);
        state.insert(COMPACTION_CHECKPOINTS_COLUMN.to_string(), checkpoints);
    }
    super::to_raw_response(&state)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportRequest {
    session_id: String,
    cwd: String,
    #[serde(default)]
    state: std::collections::HashMap<String, Value>,
    /// One JSON object per `updates.jsonl` line, not pre-serialized strings.
    #[serde(default)]
    updates: Vec<Value>,
}

/// `x.ai/session/import`: recreate a session on this host from mirrored columns and
/// transcript. A session that already exists locally is left unchanged.
pub async fn handle_import(args: &acp::ExtRequest) -> ExtResult {
    let mut request: ImportRequest = super::parse_params(args)?;
    validate_session_uuid(&request.session_id)?;

    let info = crate::session::info::Info {
        id: acp::SessionId::new(request.session_id.clone()),
        cwd: request.cwd.clone(),
    };
    let dir = crate::session::persistence::session_dir(&info);

    // resolve_session_dir gates on summary.json, so an interrupted import (dir created,
    // summary not yet written) is recreated on retry rather than skipped forever.
    let has_local_session = resolve_session_dir(&request.session_id, &request.cwd).is_some();
    if !has_local_session {
        for value in request.state.values_mut() {
            strip_local_provider_metadata(value);
        }
        let Some(summary_value) = request.state.get_mut(SUMMARY_COLUMN) else {
            return Err(
                acp::Error::invalid_params().data("session/import requires a summary column")
            );
        };
        sanitize_imported_summary(summary_value, &request.session_id, &request.cwd)?;
        // Write the `.cwd` sidecar for hash-based (long-path) dirs so the session stays
        // recoverable by id, not just by (id, cwd).
        crate::util::grok_home::ensure_sessions_cwd_dir(&request.cwd)
            .map_err(|e| acp::Error::internal_error().data(e.to_string()))?;
        write_import(&dir, &request.state, &request.updates)
            .map_err(|e| acp::Error::internal_error().data(e.to_string()))?;
    }
    super::to_raw_response(&json!({ "imported": !has_local_session }))
}

/// Rewrite a mirrored summary's host-specific fields to describe this host.
fn sanitize_summary_for_host(summary: &mut serde_json::Map<String, Value>, id: &str, cwd: &str) {
    strip_local_provider_metadata_map(summary);
    if let Some(info_obj) = summary.get_mut("info").and_then(Value::as_object_mut) {
        info_obj.insert("id".to_string(), Value::String(id.to_string()));
        info_obj.insert("cwd".to_string(), Value::String(cwd.to_string()));
    }
    summary.insert(
        "chat_format_version".to_string(),
        json!(crate::session::persistence::CHAT_FORMAT_VERSION),
    );
    summary.insert("git_remotes".to_string(), json!([]));
    for field in [
        "prompt_display_cwd",
        "source_workspace_dir",
        "git_root_dir",
        "head_commit",
        "head_branch",
        "worktree_label",
        "request_id",
    ] {
        summary.remove(field);
    }
    set_or_remove(
        summary,
        "grok_home",
        crate::session::persistence::grok_home_string(),
    );
    set_or_remove(
        summary,
        "sandbox_profile",
        xai_grok_sandbox::configured_profile_name().map(String::from),
    );
}

/// Validate the imported summary only after local-only provider/account metadata is
/// removed. Re-serializing the typed summary guarantees `credential_binding` persists
/// as `None` and drops unknown provider-local sentinel fields before the commit marker
/// is written.
fn sanitize_imported_summary(value: &mut Value, id: &str, cwd: &str) -> Result<(), acp::Error> {
    {
        let Some(summary) = value.as_object_mut() else {
            return Err(
                acp::Error::invalid_params().data("session/import summary must be an object")
            );
        };
        sanitize_summary_for_host(summary, id, cwd);
    }
    let mut parsed = Summary::deserialize(&*value)
        .map_err(|_| acp::Error::invalid_params().data("summary column is not a valid summary"))?;
    parsed.credential_binding = None;
    *value = serde_json::to_value(parsed)
        .map_err(|e| acp::Error::internal_error().data(e.to_string()))?;
    Ok(())
}

/// Remove local auth-store identity and account metadata recursively. Provider identity
/// remains encoded by `current_model_id`; the destination runtime must select credentials
/// from its own provider-local store and must never honor a mirrored record/account binding.
fn strip_local_provider_metadata(value: &mut Value) {
    if let Value::Object(obj) = value {
        strip_local_provider_metadata_map(obj);
    } else if let Value::Array(items) = value {
        for item in items {
            strip_local_provider_metadata(item);
        }
    }
}

fn strip_local_provider_metadata_map(obj: &mut serde_json::Map<String, Value>) {
    obj.retain(|key, value| {
        if is_local_provider_metadata_key(key) {
            return false;
        }
        strip_local_provider_metadata(value);
        true
    });
}

fn is_local_provider_metadata_key(key: &str) -> bool {
    let compact = key
        .bytes()
        .filter(|byte| byte.is_ascii_alphanumeric())
        .map(|byte| byte.to_ascii_lowercase() as char)
        .collect::<String>();
    let credential_local = compact.contains("credential")
        && ["binding", "record", "generation", "account", "source", "id"]
            .iter()
            .any(|part| compact.contains(part));
    let provider_local = compact.contains("provider")
        && ["binding", "credential", "account", "auth", "record"]
            .iter()
            .any(|part| compact.contains(part));
    let auth_local = compact.contains("auth")
        && ["binding", "credential", "account", "record"]
            .iter()
            .any(|part| compact.contains(part));
    let account_local = compact == "accountid"
        || compact.ends_with("accountid")
        || compact == "accountmetadata"
        || compact.ends_with("accountmetadata");
    credential_local || provider_local || auth_local || account_local
}

fn validate_checkpoint(
    expected_id: &str,
    checkpoint: &crate::extensions::notification::CompactionCheckpointFile,
) -> std::io::Result<()> {
    uuid::Uuid::try_parse(expected_id).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "checkpoint key must be a UUID",
        )
    })?;
    if checkpoint.checkpoint_id != expected_id || checkpoint.schema_version > 1 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "checkpoint metadata does not match its mirrored key",
        ));
    }
    Ok(())
}

fn read_compaction_checkpoints(dir: &Path) -> std::io::Result<Value> {
    let checkpoint_dir = dir.join(st::COMPACTION_CHECKPOINTS_DIR);
    let entries = match std::fs::read_dir(&checkpoint_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Value::Object(serde_json::Map::new()));
        }
        Err(error) => return Err(error),
    };
    let mut paths = entries
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    paths.sort();

    let mut checkpoints = serde_json::Map::new();
    for path in paths {
        if path.extension().and_then(|extension| extension.to_str()) != Some("json")
            || !path.symlink_metadata()?.file_type().is_file()
        {
            continue;
        }
        let Some(id) = path.file_stem().and_then(|name| name.to_str()) else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "checkpoint filename is not valid UTF-8",
            ));
        };
        let checkpoint: crate::extensions::notification::CompactionCheckpointFile =
            serde_json::from_slice(&std::fs::read(&path)?)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        validate_checkpoint(id, &checkpoint)?;
        checkpoints.insert(
            id.to_owned(),
            serde_json::to_value(checkpoint)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?,
        );
    }
    Ok(Value::Object(checkpoints))
}

struct ImportedCheckpoint {
    id: String,
    checkpoint: crate::extensions::notification::CompactionCheckpointFile,
    bytes: Vec<u8>,
}

fn decode_imported_checkpoints(value: Option<&Value>) -> std::io::Result<Vec<ImportedCheckpoint>> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let object = value.as_object().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "compactionCheckpoints must be an object",
        )
    })?;
    object
        .iter()
        .map(|(id, value)| {
            let checkpoint: crate::extensions::notification::CompactionCheckpointFile =
                serde_json::from_value(value.clone())
                    .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
            validate_checkpoint(id, &checkpoint)?;
            let bytes = serde_json::to_vec(&checkpoint)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
            Ok(ImportedCheckpoint {
                id: id.clone(),
                checkpoint,
                bytes,
            })
        })
        .collect()
}

fn validate_imported_checkpoint_references(
    updates: &[Value],
    checkpoints: &[ImportedCheckpoint],
) -> std::io::Result<()> {
    let checkpoints = checkpoints
        .iter()
        .map(|checkpoint| (checkpoint.id.as_str(), &checkpoint.checkpoint))
        .collect::<std::collections::HashMap<_, _>>();

    for update in updates {
        if update.get("method").and_then(Value::as_str) != Some(st::XAI_SESSION_UPDATE_METHOD) {
            continue;
        }
        let envelope: st::SessionUpdateEnvelope = serde_json::from_value(update.clone())
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        let notification: crate::extensions::notification::SessionNotification =
            serde_json::from_value(envelope.params)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        let crate::extensions::notification::SessionUpdate::CompactionCheckpoint(info) =
            notification.update
        else {
            continue;
        };

        st::compaction_checkpoint_path(Path::new(""), &info.checkpoint_id, &info.checkpoint_file)?;
        let checkpoint = checkpoints
            .get(info.checkpoint_id.as_str())
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "compaction checkpoint update has no mirrored checkpoint file",
                )
            })?;
        if checkpoint.prompt_index_at_compaction != info.prompt_index_at_compaction
            || checkpoint.schema_version != info.schema_version
            || checkpoint.created_at != info.created_at
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "compaction checkpoint metadata does not match its update",
            ));
        }
    }
    Ok(())
}

fn set_or_remove(obj: &mut serde_json::Map<String, Value>, key: &str, value: Option<String>) {
    match value {
        Some(v) => {
            obj.insert(key.to_string(), Value::String(v));
        }
        None => {
            obj.remove(key);
        }
    }
}

/// Writes summary.json last, and each file to a temporary name first, so an interrupted
/// import leaves an incomplete session that load treats as absent.
fn write_import(
    dir: &Path,
    state: &std::collections::HashMap<String, Value>,
    updates: &[Value],
) -> std::io::Result<()> {
    // Validate every variable-name artifact before touching an existing partial
    // import. Checkpoint keys are never interpreted as paths.
    let checkpoints = decode_imported_checkpoints(state.get(COMPACTION_CHECKPOINTS_COLUMN))?;
    validate_imported_checkpoint_references(updates, &checkpoints)?;
    std::fs::create_dir_all(dir)?;

    // Clear every file this import owns so a leftover from a failed attempt can't
    // merge with the new snapshot; this import is authoritative.
    let _ = std::fs::remove_file(dir.join(st::CHAT_HISTORY_FILE));
    let _ = std::fs::remove_file(dir.join(st::UPDATES_FILE));
    for (_, rel) in COLUMNS {
        let _ = std::fs::remove_file(dir.join(rel));
    }
    let checkpoint_dir = dir.join(st::COMPACTION_CHECKPOINTS_DIR);
    match std::fs::remove_dir_all(&checkpoint_dir) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }

    if !updates.is_empty() {
        st::write_jsonl_atomic(&dir.join(st::UPDATES_FILE), updates)?;
    }
    if !checkpoints.is_empty() {
        std::fs::create_dir_all(&checkpoint_dir)?;
        for checkpoint in checkpoints {
            st::write_bytes_atomic(
                &checkpoint_dir.join(format!("{}.json", checkpoint.id)),
                &checkpoint.bytes,
            )?;
        }
    }

    for (column, rel) in COLUMNS {
        if let Some(value) = state.get(*column) {
            write_column(dir, rel, value)?;
        }
    }
    Ok(())
}

fn write_column(dir: &Path, rel: &str, value: &Value) -> std::io::Result<()> {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    st::write_bytes_atomic(&path, value.to_string().as_bytes())
}

/// The session's directory, or `None` when it isn't found on this host. Falls back to
/// an id scan when `(id, cwd)` has no summary (subagents use their own cwd); both
/// branches require summary.json so a bare directory doesn't count as present.
fn resolve_session_dir(session_id: &str, cwd: &str) -> Option<PathBuf> {
    let info = crate::session::info::Info {
        id: acp::SessionId::new(session_id.to_string()),
        cwd: cwd.to_string(),
    };
    let dir = crate::session::persistence::session_dir(&info);
    if dir.join(st::SUMMARY_FILE).is_file() {
        return Some(dir);
    }
    crate::session::persistence::find_session_dir_by_id(session_id)
        .filter(|found| found.join(st::SUMMARY_FILE).is_file())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn summary_column_is_the_import_commit_marker() {
        assert_eq!(COLUMNS.last(), Some(&(SUMMARY_COLUMN, st::SUMMARY_FILE)));
    }

    #[test]
    fn sanitize_summary_for_host_rewrites_host_fields() {
        let mut summary = json!({
            "info": { "id": "s1", "cwd": "/remote/host/work" },
            "chat_format_version": 0,
            "prompt_display_cwd": "/remote/host/work",
            "source_workspace_dir": "/remote/host",
            "git_root_dir": "/remote/host/repo",
            "git_remotes": ["origin"],
            "head_commit": "deadbeef",
            "head_branch": "feature",
            "worktree_label": "wt",
            "request_id": "req-1",
        })
        .as_object()
        .unwrap()
        .clone();

        sanitize_summary_for_host(&mut summary, "s-new", "/local/work");

        assert_eq!(summary["info"]["id"], json!("s-new"));
        assert_eq!(summary["info"]["cwd"], json!("/local/work"));
        assert_eq!(
            summary["chat_format_version"],
            json!(crate::session::persistence::CHAT_FORMAT_VERSION)
        );
        assert_eq!(summary["git_remotes"], json!([]));
        for gone in [
            "prompt_display_cwd",
            "source_workspace_dir",
            "git_root_dir",
            "head_commit",
            "head_branch",
            "worktree_label",
            "request_id",
        ] {
            assert!(!summary.contains_key(gone), "{gone} should be dropped");
        }
    }

    #[test]
    fn write_import_writes_columns_updates_and_drops_stale_chat() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        std::fs::write(dir.join("chat_history.jsonl"), b"stale cache").unwrap();
        // A column left by a failed prior import that the new payload omits.
        std::fs::write(dir.join("signals.json"), b"{\"stale\":true}").unwrap();

        let mut state = std::collections::HashMap::new();
        state.insert(
            "summary".to_string(),
            json!({ "info": { "id": "s1", "cwd": "/work" } }),
        );
        state.insert("plan".to_string(), json!({ "items": [] }));
        state.insert("goal".to_string(), json!({ "active": false }));
        let updates = vec![
            json!({ "method": "session/update", "params": { "a": 1 } }),
            json!({ "method": "session/update", "params": { "b": 2 } }),
        ];

        write_import(dir, &state, &updates).unwrap();

        assert!(dir.join("summary.json").exists(), "summary.json written");
        assert_eq!(
            std::fs::read_to_string(dir.join("plan.json")).unwrap(),
            r#"{"items":[]}"#
        );
        assert_eq!(
            std::fs::read_to_string(dir.join("goal/state.json")).unwrap(),
            r#"{"active":false}"#
        );
        assert_eq!(
            std::fs::read_to_string(dir.join("updates.jsonl"))
                .unwrap()
                .lines()
                .count(),
            2
        );
        assert!(
            !dir.join("chat_history.jsonl").exists(),
            "stale chat cache dropped so load rebuilds"
        );
        assert!(
            !dir.join("signals.json").exists(),
            "orphan column from a failed import dropped"
        );
    }

    #[test]
    fn import_round_trips_compaction_checkpoints_by_uuid() {
        let tmp = tempfile::TempDir::new().unwrap();
        let checkpoint_id = "019f7bdf-c293-7b92-a5a7-5464711476f2";
        let checkpoint = crate::extensions::notification::CompactionCheckpointFile {
            checkpoint_id: checkpoint_id.to_owned(),
            prompt_index_at_compaction: 3,
            compacted_history: vec![crate::sampling::ConversationItem::user("summary")],
            schema_version: 1,
            created_at: "2026-07-20T00:00:00Z".to_owned(),
            original_user_info: None,
            reread_file_paths: Vec::new(),
        };
        let mut state = std::collections::HashMap::new();
        state.insert(
            SUMMARY_COLUMN.to_owned(),
            json!({ "info": { "id": "s1", "cwd": "/work" } }),
        );
        state.insert(
            COMPACTION_CHECKPOINTS_COLUMN.to_owned(),
            json!({ (checkpoint_id): checkpoint }),
        );

        write_import(tmp.path(), &state, &[]).unwrap();

        let mirrored = read_compaction_checkpoints(tmp.path()).unwrap();
        assert_eq!(
            mirrored[checkpoint_id]["prompt_index_at_compaction"],
            json!(3)
        );
        assert_eq!(
            mirrored[checkpoint_id]["checkpoint_id"],
            json!(checkpoint_id)
        );
    }

    #[test]
    fn import_rejects_checkpoint_path_keys_before_writing() {
        let tmp = tempfile::TempDir::new().unwrap();
        let checkpoint = crate::extensions::notification::CompactionCheckpointFile {
            checkpoint_id: "../../foreign".to_owned(),
            prompt_index_at_compaction: 0,
            compacted_history: Vec::new(),
            schema_version: 1,
            created_at: "2026-07-20T00:00:00Z".to_owned(),
            original_user_info: None,
            reread_file_paths: Vec::new(),
        };
        let state = std::collections::HashMap::from([(
            COMPACTION_CHECKPOINTS_COLUMN.to_owned(),
            json!({ "../../foreign": checkpoint }),
        )]);

        assert!(write_import(tmp.path(), &state, &[]).is_err());
        assert!(!tmp.path().join(st::COMPACTION_CHECKPOINTS_DIR).exists());
    }

    #[test]
    fn import_rejects_missing_referenced_checkpoint_before_writing() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path().join("session");
        let checkpoint_id = "019f7bdf-c293-7b92-a5a7-5464711476f2";
        let notification = crate::extensions::notification::SessionNotification {
            session_id: acp::SessionId::new("session"),
            update: crate::extensions::notification::SessionUpdate::CompactionCheckpoint(Box::new(
                crate::extensions::notification::CompactionCheckpointInfo {
                    checkpoint_id: checkpoint_id.to_owned(),
                    prompt_index_at_compaction: 3,
                    checkpoint_file: format!(
                        "{}/{checkpoint_id}.json",
                        st::COMPACTION_CHECKPOINTS_DIR
                    ),
                    auto_continue: None,
                    schema_version: 1,
                    created_at: "2026-07-20T00:00:00Z".to_owned(),
                },
            )),
            meta: None,
        };
        let envelope = st::SessionUpdateEnvelope {
            timestamp: 1,
            method: st::XAI_SESSION_UPDATE_METHOD.to_owned(),
            params: serde_json::to_value(notification).unwrap(),
        };
        let updates = vec![serde_json::to_value(envelope).unwrap()];

        assert!(
            write_import(&dir, &std::collections::HashMap::new(), &updates).is_err(),
            "an imported update must not reference a checkpoint omitted from state"
        );
        assert!(
            !dir.exists(),
            "checkpoint validation must finish before import touches the destination"
        );
    }

    #[test]
    fn uuid_validation_rejects_path_components() {
        assert!(validate_session_uuid("019f7bdf-c293-7b92-a5a7-5464711476f2").is_ok());
        assert!(validate_session_uuid("../../foreign-session").is_err());
        assert!(validate_session_uuid("not-a-uuid").is_err());
    }

    #[test]
    fn state_export_drops_local_provider_and_account_sentinels() {
        let mut value = json!({
            "credential_binding": {
                "record_id": "SENTINEL-CREDENTIAL-RECORD",
                "generation": 9
            },
            "providerBinding": { "accountId": "SENTINEL-PROVIDER-ACCOUNT" },
            "nested": { "chatgpt_account_id": "SENTINEL-NESTED-ACCOUNT" },
            "safe": "preserved"
        });

        strip_local_provider_metadata(&mut value);

        assert_eq!(value["safe"], json!("preserved"));
        let encoded = value.to_string();
        assert!(!encoded.contains("SENTINEL"));
        assert!(value.get("credential_binding").is_none());
        assert!(value.get("providerBinding").is_none());
    }

    #[test]
    fn imported_provider_route_persists_no_foreign_binding_or_account_metadata() {
        let tmp = tempfile::TempDir::new().unwrap();
        let id = "019f7bdf-c293-7b92-a5a7-5464711476f2";
        let cwd = tmp.path().join("local-work").to_string_lossy().into_owned();
        let info = crate::session::info::Info {
            id: acp::SessionId::new(id),
            cwd: cwd.clone(),
        };
        let mut summary = serde_json::to_value(
            Summary::new(&info, acp::ModelId::new("openai-codex/gpt-5.4")).unwrap(),
        )
        .unwrap();
        let summary_obj = summary.as_object_mut().unwrap();
        summary_obj.insert(
            "credential_binding".to_string(),
            json!({
                "provider": "kimi_code",
                "source": "kimi_code_api_key",
                "record_id": "SENTINEL-FOREIGN-RECORD",
                "generation": 7
            }),
        );
        summary_obj.insert(
            "providerAccountId".to_string(),
            json!("SENTINEL-FOREIGN-ACCOUNT"),
        );

        sanitize_imported_summary(&mut summary, id, &cwd).unwrap();

        let parsed: Summary = serde_json::from_value(summary.clone()).unwrap();
        assert_eq!(parsed.current_model_id.0.as_ref(), "openai-codex/gpt-5.4");
        assert!(
            parsed.credential_binding.is_none(),
            "an imported provider route may bind only destination-local credentials"
        );
        assert!(!summary.to_string().contains("SENTINEL"));

        let mut state = std::collections::HashMap::new();
        state.insert(SUMMARY_COLUMN.to_string(), summary);
        write_import(tmp.path(), &state, &[]).unwrap();
        let persisted: Summary =
            serde_json::from_slice(&std::fs::read(tmp.path().join(st::SUMMARY_FILE)).unwrap())
                .unwrap();
        assert!(persisted.credential_binding.is_none());
    }
}
