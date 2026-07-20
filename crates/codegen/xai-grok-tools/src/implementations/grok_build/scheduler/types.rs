use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

#[derive(thiserror::Error, Debug)]
pub enum SchedulerError {
    #[error("invalid interval: {0}")]
    InvalidInterval(String),

    #[error("maximum of {0} scheduled tasks reached")]
    TaskLimitReached(usize),

    #[error("no scheduled task with id {0}; call scheduler_list to see active task ids")]
    TaskNotFound(String),
}

/// A single scheduled recurring or one-shot task.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledTask {
    pub id: String,
    pub interval_secs: u64,
    pub prompt: String,
    pub recurring: bool,
    pub durable: bool,
    /// Compatibility mode: execute fires in the main conversation instead of
    /// spawning a background loop subagent.
    pub foreground: bool,
    pub created_at: DateTime<Utc>,
    pub last_fired_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    /// Most recent child in this task's resumable loop chain.
    pub last_subagent_id: Option<String>,
    /// Number of iterations in the current chain, including its fresh root.
    pub iterations_since_fresh: u32,
    /// A prompt patch keeps the prior child as an overlap guard, but forces the
    /// next valid fire to start a fresh chain.
    pub chain_reset_pending: bool,
}

/// Start a fresh child transcript after this many iterations in one chain.
pub const LOOP_FRESH_CHAIN_EVERY: u32 = 10;

/// Scheduler-owned child completions must remain bounded in coordinator state
/// and in parent-facing completion projections.
pub const LOOP_COMPLETION_OUTPUT_CAP: usize = 4_000;

fn default_recurring() -> bool {
    true
}

/// Deliberate persistence migration for scheduler records written before
/// background loop subagents existed. Those records did not contain
/// `foreground`; preserving their legacy main-conversation behavior avoids
/// silently changing an already-persisted task on upgrade. Newly-created tasks
/// explicitly serialize `foreground: false` and therefore run in background.
impl<'de> Deserialize<'de> for ScheduledTask {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct PersistedScheduledTask {
            id: String,
            interval_secs: u64,
            prompt: String,
            #[serde(default = "default_recurring")]
            recurring: bool,
            durable: bool,
            #[serde(default)]
            foreground: Option<bool>,
            created_at: DateTime<Utc>,
            last_fired_at: Option<DateTime<Utc>>,
            expires_at: Option<DateTime<Utc>>,
            #[serde(default)]
            last_subagent_id: Option<String>,
            #[serde(default)]
            iterations_since_fresh: u32,
            #[serde(default)]
            chain_reset_pending: bool,
        }

        let persisted = PersistedScheduledTask::deserialize(deserializer)?;
        Ok(Self {
            id: persisted.id,
            interval_secs: persisted.interval_secs,
            prompt: persisted.prompt,
            recurring: persisted.recurring,
            durable: persisted.durable,
            // Missing means the task predates background loops, not that the
            // user selected the new default.
            foreground: persisted.foreground.unwrap_or(true),
            created_at: persisted.created_at,
            last_fired_at: persisted.last_fired_at,
            expires_at: persisted.expires_at,
            last_subagent_id: persisted.last_subagent_id,
            iterations_since_fresh: persisted.iterations_since_fresh,
            chain_reset_pending: persisted.chain_reset_pending,
        })
    }
}

impl ScheduledTask {
    pub fn new(interval_secs: u64, prompt: String, recurring: bool, durable: bool) -> Self {
        Self::with_fire_immediately(interval_secs, prompt, recurring, durable, false)
    }

    pub fn with_fire_immediately(
        interval_secs: u64,
        prompt: String,
        recurring: bool,
        durable: bool,
        fire_immediately: bool,
    ) -> Self {
        let now = Utc::now();
        // When fire_immediately is true, anchor created_at in the past so that
        // next_fire_at() = created_at + interval = now, firing on the first tick.
        let created_at = if fire_immediately {
            now - chrono::Duration::seconds(interval_secs as i64)
        } else {
            now
        };
        Self {
            id: uuid::Uuid::now_v7().to_string().replace('-', "")[..12].to_string(),
            interval_secs,
            prompt,
            recurring,
            durable,
            foreground: false,
            created_at,
            last_fired_at: None,
            expires_at: if recurring {
                Some(now + chrono::Duration::days(7))
            } else {
                None
            },
            last_subagent_id: None,
            iterations_since_fresh: 0,
            chain_reset_pending: false,
        }
    }

    /// Next fire time, computed from `last_fired_at` (or `created_at` if never fired).
    pub fn next_fire_at(&self) -> DateTime<Utc> {
        let anchor = self.last_fired_at.unwrap_or(self.created_at);
        anchor + chrono::Duration::seconds(self.interval_secs as i64)
    }

    /// Whether this task has expired (recurring tasks only).
    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        self.expires_at.is_some_and(|exp| now >= exp)
    }

    /// Whether this task was missed (one-shot: fire time already passed, never fired).
    pub fn is_missed(&self, now: DateTime<Utc>) -> bool {
        !self.recurring && self.last_fired_at.is_none() && self.next_fire_at() < now
    }
}

/// Persisted state for the scheduler, stored via Resources + ResourcesPersistence.
/// Only durable tasks are serialized; non-durable tasks are filtered out before save.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchedulerState {
    pub tasks: Vec<ScheduledTask>,
}

crate::register_resource!("grok_build", "Scheduler", SchedulerState);

/// Handle for tools to communicate with the SchedulerActor.
/// Ephemeral -- not serialized, not persisted. Inserted via `resources.insert()`.
#[derive(Clone)]
pub struct SchedulerHandle(pub mpsc::UnboundedSender<SchedulerCommand>);

pub enum SchedulerCommand {
    Create {
        task: ScheduledTask,
        reply: oneshot::Sender<Result<ScheduledTask, SchedulerError>>,
    },
    Update {
        id: String,
        prompt: Option<String>,
        interval_secs: Option<u64>,
        reply: oneshot::Sender<Result<ScheduledTask, SchedulerError>>,
    },
    Delete {
        id: String,
        reply: oneshot::Sender<bool>,
    },
    List {
        reply: oneshot::Sender<Vec<ScheduledTask>>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_recurring_task_has_7_day_expiry() {
        let task = ScheduledTask::new(300, "check deploy".into(), true, false);
        assert!(task.expires_at.is_some());
        let expiry = task.expires_at.unwrap();
        let diff = expiry - task.created_at;
        assert_eq!(diff.num_days(), 7);
    }

    #[test]
    fn new_one_shot_task_has_no_expiry() {
        let task = ScheduledTask::new(300, "check deploy".into(), false, false);
        assert!(task.expires_at.is_none());
    }

    #[test]
    fn next_fire_at_uses_created_at_when_never_fired() {
        let task = ScheduledTask::new(300, "test".into(), true, false);
        let expected = task.created_at + chrono::Duration::seconds(300);
        assert_eq!(task.next_fire_at(), expected);
    }

    #[test]
    fn next_fire_at_uses_last_fired_at_when_present() {
        let mut task = ScheduledTask::new(300, "test".into(), true, false);
        let fired = Utc::now();
        task.last_fired_at = Some(fired);
        let expected = fired + chrono::Duration::seconds(300);
        assert_eq!(task.next_fire_at(), expected);
    }

    #[test]
    fn is_expired_returns_true_when_past_expiry() {
        let mut task = ScheduledTask::new(300, "test".into(), true, false);
        task.expires_at = Some(Utc::now() - chrono::Duration::hours(1));
        assert!(task.is_expired(Utc::now()));
    }

    #[test]
    fn is_expired_returns_false_when_before_expiry() {
        let task = ScheduledTask::new(300, "test".into(), true, false);
        assert!(!task.is_expired(Utc::now()));
    }

    #[test]
    fn is_expired_returns_false_for_one_shot() {
        let task = ScheduledTask::new(300, "test".into(), false, false);
        assert!(!task.is_expired(Utc::now()));
    }

    #[test]
    fn legacy_state_without_recurring_field_deserializes_as_recurring() {
        let json = r#"{"id":"abc123","intervalSecs":300,"prompt":"check",
                       "durable":true,"createdAt":"2026-01-01T00:00:00Z",
                       "lastFiredAt":null,"expiresAt":null}"#;
        let task: ScheduledTask = serde_json::from_str(json).unwrap();
        assert!(task.recurring);
        assert!(
            task.foreground,
            "pre-background persisted tasks deliberately retain foreground behavior"
        );
    }

    #[test]
    fn persisted_background_task_remains_background() {
        let json = r#"{"id":"abc123","intervalSecs":300,"prompt":"check",
                       "recurring":true,"durable":true,"foreground":false,
                       "createdAt":"2026-01-01T00:00:00Z",
                       "lastFiredAt":null,"expiresAt":null}"#;
        let task: ScheduledTask = serde_json::from_str(json).unwrap();
        assert!(!task.foreground);
    }

    #[test]
    fn new_recurring_task_defaults_to_background() {
        let task = ScheduledTask::new(300, "check".into(), true, false);
        assert!(!task.foreground);
        let json = serde_json::to_value(&task).unwrap();
        assert_eq!(json["foreground"], false);
    }

    #[test]
    fn legacy_one_shot_state_still_deserializes() {
        let json = r#"{"id":"abc123","intervalSecs":300,"prompt":"check",
                       "recurring":false,"durable":true,
                       "createdAt":"2026-01-01T00:00:00Z",
                       "lastFiredAt":null,"expiresAt":null}"#;
        let task: ScheduledTask = serde_json::from_str(json).unwrap();
        assert!(!task.recurring);
    }

    #[test]
    fn is_missed_returns_true_for_unfired_one_shot_past_due() {
        let mut task = ScheduledTask::new(1, "test".into(), false, false);
        task.created_at = Utc::now() - chrono::Duration::seconds(10);
        assert!(task.is_missed(Utc::now()));
    }

    #[test]
    fn is_missed_returns_false_for_recurring() {
        let mut task = ScheduledTask::new(1, "test".into(), true, false);
        task.created_at = Utc::now() - chrono::Duration::seconds(10);
        assert!(!task.is_missed(Utc::now()));
    }

    #[test]
    fn is_missed_returns_false_if_already_fired() {
        let mut task = ScheduledTask::new(1, "test".into(), false, false);
        task.created_at = Utc::now() - chrono::Duration::seconds(10);
        task.last_fired_at = Some(Utc::now());
        assert!(!task.is_missed(Utc::now()));
    }

    #[test]
    fn task_id_is_12_chars() {
        let task = ScheduledTask::new(300, "test".into(), true, false);
        assert_eq!(task.id.len(), 12);
    }

    #[test]
    fn scheduler_state_default_is_empty() {
        let state = SchedulerState::default();
        assert!(state.tasks.is_empty());
    }
}
