//! Keeps `x.ai/task_completed` lines short enough for a client to read, both
//! when this build sends one and when replay reaches one an earlier build
//! wrote. Bounding the output field alone does not bound the line: the
//! wrapper and JSON encoding go on top of it.

use serde_json::Value;
use serde_json::value::RawValue;
use xai_grok_tools::types::TaskSnapshot;

use crate::extensions::notification::{SessionNotification, SessionUpdate};

/// Half the 64 KiB a Python `asyncio` stream reader allows.
pub(crate) const FRAME_MAX_BYTES: usize = 32 * 1024;
pub(crate) const METHOD: &str = "x.ai/task_completed";
const WRAPPER_BYTES: usize = r#"{"jsonrpc":"2.0","method":"_","params":}"#.len() + 1;
const FIELD_MAX_BYTES: usize = 1024;
const COMPACTED_FIELDS: [&str; 4] = ["command", "display_command", "description", "cwd"];

fn body_budget() -> usize {
    FRAME_MAX_BYTES.saturating_sub(WRAPPER_BYTES + METHOD.len())
}

/// A message body proven to fit the frame budget.
pub(crate) struct FittedFrame(Box<RawValue>);

impl FittedFrame {
    pub(crate) fn into_inner(self) -> Box<RawValue> {
        self.0
    }
}

impl std::ops::Deref for FittedFrame {
    type Target = RawValue;

    fn deref(&self) -> &RawValue {
        &self.0
    }
}

/// Build a fitted message, rewriting the notification so persistence matches
/// what was sent. `None` means even the status-only form could not fit.
pub(crate) fn encode(notification: &mut SessionNotification) -> Option<FittedFrame> {
    let budget = body_budget();
    let Some(snapshot) = task_snapshot(notification) else {
        return within(
            serde_json::value::to_raw_value(&*notification).ok()?,
            budget,
        );
    };

    let output = std::mem::take(&mut snapshot.output);
    if let Some(params) = fit_into(notification, &output, budget) {
        return Some(params);
    }

    let snapshot = task_snapshot(notification)?;
    snapshot.output = String::new();
    snapshot.truncated = true;
    compact_hard(snapshot);
    let params = within(
        serde_json::value::to_raw_value(&*notification).ok()?,
        budget,
    );
    if params.is_none() {
        tracing::warn!("task_completed message is too long to send even with no output");
    }
    params
}

fn fit_into(
    notification: &mut SessionNotification,
    output: &str,
    budget: usize,
) -> Option<FittedFrame> {
    let mut room = room_for_output(notification, budget)?;
    if encoded_len(output) > room {
        compact(task_snapshot(notification)?);
        room = room_for_output(notification, budget)?;
    }

    let snapshot = task_snapshot(notification)?;
    let (fitted, cut) = fit_output(output, &snapshot.output_file, room);
    snapshot.truncated |= cut;
    snapshot.output = fitted;
    within(
        serde_json::value::to_raw_value(&*notification).ok()?,
        budget,
    )
}

fn room_for_output(notification: &SessionNotification, budget: usize) -> Option<usize> {
    let rest = serde_json::value::to_raw_value(notification).ok()?;
    Some(budget.saturating_sub(rest.get().len()))
}

fn fit_output(output: &str, output_file: &std::path::Path, room: usize) -> (String, bool) {
    if encoded_len(output) <= room {
        return (output.to_string(), false);
    }
    let footer = format!(
        "\n\n... (output truncated; full output at {})",
        output_file.display()
    );
    let footer_room = encoded_len(&footer);
    if footer_room > room {
        return (String::new(), true);
    }
    let kept = prefix_within_encoded_len(output, room - footer_room);
    (format!("{kept}{footer}"), true)
}

fn compact(snapshot: &mut TaskSnapshot) {
    snapshot.command = prefix_within_encoded_len(&snapshot.command, FIELD_MAX_BYTES).to_string();
    snapshot.display_command = snapshot
        .display_command
        .as_deref()
        .map(|text| prefix_within_encoded_len(text, FIELD_MAX_BYTES).to_string());
    snapshot.description = snapshot
        .description
        .as_deref()
        .map(|text| prefix_within_encoded_len(text, FIELD_MAX_BYTES).to_string());
    snapshot.cwd = prefix_within_encoded_len(&snapshot.cwd, FIELD_MAX_BYTES).to_string();
}

fn compact_hard(snapshot: &mut TaskSnapshot) {
    compact(snapshot);
    let path = snapshot.output_file.display().to_string();
    snapshot.output_file = prefix_within_encoded_len(&path, FIELD_MAX_BYTES).into();
}

fn task_snapshot(notification: &mut SessionNotification) -> Option<&mut TaskSnapshot> {
    let SessionUpdate::TaskCompleted { task_snapshot, .. } = &mut notification.update else {
        return None;
    };
    Some(task_snapshot)
}

fn within(params: Box<RawValue>, budget: usize) -> Option<FittedFrame> {
    (params.get().len() <= budget).then_some(FittedFrame(params))
}

fn encoded_len(text: &str) -> usize {
    text.chars().map(encoded_char_len).sum()
}

fn prefix_within_encoded_len(text: &str, max: usize) -> &str {
    let mut used = 0;
    for (index, character) in text.char_indices() {
        used += encoded_char_len(character);
        if used > max {
            return &text[..index];
        }
    }
    text
}

fn encoded_char_len(character: char) -> usize {
    match character {
        '"' | '\\' | '\n' | '\r' | '\t' | '\u{8}' | '\u{c}' => 2,
        control if (control as u32) < 0x20 => 6,
        other => other.len_utf8(),
    }
}

pub(crate) enum Refit {
    Unchanged,
    Fitted(FittedFrame),
    Unfittable,
}

/// Shrink a recorded task completion that predates the frame bound. Unknown
/// fields are retained by editing the JSON value rather than deserializing it.
pub(crate) fn refit_recorded(params: &RawValue) -> Refit {
    let budget = body_budget();
    if params.get().len() <= budget {
        return Refit::Unchanged;
    }
    let Some(mut record) = serde_json::from_str::<Value>(params.get()).ok() else {
        return Refit::Unchanged;
    };
    if !is_task_completion(&record) {
        return Refit::Unchanged;
    }
    match shrink_record(&mut record, budget) {
        Some(refit) => Refit::Fitted(refit),
        None => Refit::Unfittable,
    }
}

fn is_task_completion(record: &Value) -> bool {
    record
        .get("update")
        .and_then(|update| update.get("sessionUpdate"))
        .and_then(Value::as_str)
        == Some("task_completed")
}

fn shrink_record(record: &mut Value, budget: usize) -> Option<FittedFrame> {
    let snapshot = record.get_mut("update")?.get_mut("task_snapshot")?;
    let output = snapshot.get("output")?.as_str()?.to_owned();
    let output_file = snapshot
        .get("output_file")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();

    snapshot["output"] = Value::String(String::new());
    for field in COMPACTED_FIELDS {
        if let Some(text) = snapshot.get(field).and_then(Value::as_str) {
            snapshot[field] =
                Value::String(prefix_within_encoded_len(text, FIELD_MAX_BYTES).to_owned());
        }
    }
    let room = budget.saturating_sub(serde_json::to_string(&record).ok()?.len());
    let (fitted, cut) = fit_output(&output, std::path::Path::new(&output_file), room);
    let snapshot = record.get_mut("update")?.get_mut("task_snapshot")?;
    snapshot["output"] = Value::String(fitted);
    if cut {
        snapshot["truncated"] = Value::Bool(true);
    }
    if let Some(refit) = within(serde_json::value::to_raw_value(&record).ok()?, budget) {
        return Some(refit);
    }

    let snapshot = record.get_mut("update")?.get_mut("task_snapshot")?;
    snapshot["output"] = Value::String(String::new());
    snapshot["truncated"] = Value::Bool(true);
    if let Some(path) = snapshot.get("output_file").and_then(Value::as_str) {
        snapshot["output_file"] =
            Value::String(prefix_within_encoded_len(path, FIELD_MAX_BYTES).to_owned());
    }
    within(serde_json::value::to_raw_value(&record).ok()?, budget)
}

#[cfg(test)]
#[path = "task_completed_frame_tests.rs"]
mod tests;
