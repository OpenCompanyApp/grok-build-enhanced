//! Per-user-turn coordination for native web tools.
//!
//! Keys are one-way in-memory fingerprints of the effective tool and canonical
//! arguments. The ledger never stores or logs URLs, queries, hosts, provider
//! payloads, auth state, or raw argument JSON.

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::Mutex;
use xai_grok_tools::types::output::{ToolRunResult, WebToolErrorCode, WebToolFailure};
use xai_tool_runtime::{ToolError, ToolErrorKind};

pub(crate) type WebAttemptFingerprint = [u8; 32];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct WebAttemptStats {
    pub(crate) dedup_replays: u64,
    pub(crate) network_retries: u64,
    pub(crate) repeated_failures: u64,
}

#[derive(Clone)]
struct CachedWebError {
    kind: ToolErrorKind,
    failure: WebToolFailure,
}

impl CachedWebError {
    fn from_error(error: &ToolError) -> Self {
        let failure = error
            .details
            .as_ref()
            .and_then(|details| serde_json::from_value::<WebToolFailure>(details.clone()).ok())
            .unwrap_or_else(|| {
                WebToolFailure::for_code(match error.kind {
                    ToolErrorKind::Unauthorized => WebToolErrorCode::AuthenticationRequired,
                    ToolErrorKind::Timeout => WebToolErrorCode::Timeout,
                    ToolErrorKind::RateLimited => WebToolErrorCode::RateLimited,
                    ToolErrorKind::ServiceUnavailable | ToolErrorKind::NetworkError => {
                        WebToolErrorCode::ProviderUnavailable
                    }
                    _ => WebToolErrorCode::ProviderUnavailable,
                })
            });
        Self {
            kind: error.kind,
            failure,
        }
    }

    fn to_error(&self) -> ToolError {
        ToolError::new(self.kind, self.failure.prompt_text()).with_details(
            serde_json::to_value(&self.failure).expect("web failure envelope is JSON-serializable"),
        )
    }
}

#[derive(Clone)]
enum CachedWebOutcome {
    Tool {
        result: ToolRunResult,
        failure: Option<WebToolFailure>,
    },
    Error(CachedWebError),
}

impl CachedWebOutcome {
    fn from_result(result: &Result<ToolRunResult, ToolError>) -> Self {
        match result {
            Ok(result) => Self::Tool {
                failure: result.output.web_failure(),
                result: result.clone(),
            },
            Err(error) => Self::Error(CachedWebError::from_error(error)),
        }
    }

    fn failure(&self) -> Option<&WebToolFailure> {
        match self {
            Self::Tool { failure, .. } => failure.as_ref(),
            Self::Error(error) => Some(&error.failure),
        }
    }

    fn to_result(&self) -> Result<ToolRunResult, ToolError> {
        match self {
            Self::Tool { result, .. } => Ok(result.clone()),
            Self::Error(error) => Err(error.to_error()),
        }
    }
}

#[derive(Default)]
struct EntryState {
    network_attempts: u8,
    in_flight: bool,
    outcome: Option<CachedWebOutcome>,
}

#[derive(Default)]
struct Entry {
    state: tokio::sync::Mutex<EntryState>,
    completed: tokio::sync::Notify,
}

#[derive(Default)]
pub(crate) struct WebAttemptLedger {
    entries: Mutex<HashMap<WebAttemptFingerprint, Arc<Entry>>>,
    dedup_replays: AtomicU64,
    network_retries: AtomicU64,
    repeated_failures: AtomicU64,
}

impl WebAttemptLedger {
    pub(crate) fn begin_turn(self: &Arc<Self>) -> WebAttemptTurnGuard {
        self.clear();
        self.dedup_replays.store(0, Ordering::Relaxed);
        self.network_retries.store(0, Ordering::Relaxed);
        self.repeated_failures.store(0, Ordering::Relaxed);
        WebAttemptTurnGuard(Arc::clone(self))
    }

    pub(crate) fn fingerprint(
        effective_tool_name: &str,
        arguments: &serde_json::Value,
    ) -> WebAttemptFingerprint {
        fn canonicalize(value: &serde_json::Value) -> serde_json::Value {
            match value {
                serde_json::Value::Array(values) => {
                    serde_json::Value::Array(values.iter().map(canonicalize).collect())
                }
                serde_json::Value::Object(values) => {
                    let sorted = values
                        .iter()
                        .filter(|(key, _)| {
                            key.as_str()
                                != xai_grok_tools::implementations::web_search::CODEX_WEB_SEARCH_CONTEXT_FIELD
                        })
                        .map(|(key, value)| (key.clone(), canonicalize(value)))
                        .collect::<std::collections::BTreeMap<_, _>>();
                    serde_json::Value::Object(sorted.into_iter().collect())
                }
                scalar => scalar.clone(),
            }
        }

        let mut hasher = blake3::Hasher::new_derive_key("grok-build web attempt ledger v1");
        hasher.update(effective_tool_name.as_bytes());
        hasher.update(&[0]);
        let canonical = canonicalize(arguments);
        let encoded = serde_json::to_vec(&canonical).expect("JSON values are serializable");
        hasher.update(&encoded);
        *hasher.finalize().as_bytes()
    }

    pub(crate) async fn run<F, Fut>(
        &self,
        key: WebAttemptFingerprint,
        run: F,
    ) -> Result<ToolRunResult, ToolError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<ToolRunResult, ToolError>>,
    {
        let entry = self
            .entries
            .lock()
            .entry(key)
            .or_insert_with(|| Arc::new(Entry::default()))
            .clone();
        let mut run = Some(run);
        let mut waited_for_in_flight = false;

        loop {
            let notified = entry.completed.notified();
            let mut state = entry.state.lock().await;
            if state.in_flight {
                drop(state);
                notified.await;
                waited_for_in_flight = true;
                continue;
            }

            if waited_for_in_flight {
                let outcome = state
                    .outcome
                    .as_ref()
                    .expect("completed web attempt must retain a sanitized outcome")
                    .clone();
                drop(state);
                self.dedup_replays.fetch_add(1, Ordering::Relaxed);
                return outcome.to_result();
            }

            if let Some(outcome) = state.outcome.as_ref() {
                match outcome.failure() {
                    None
                    | Some(WebToolFailure {
                        retryable: false, ..
                    }) => {
                        let outcome = outcome.clone();
                        drop(state);
                        self.dedup_replays.fetch_add(1, Ordering::Relaxed);
                        return outcome.to_result();
                    }
                    Some(WebToolFailure {
                        retryable: true, ..
                    }) if state.network_attempts >= 2 => {
                        drop(state);
                        self.repeated_failures.fetch_add(1, Ordering::Relaxed);
                        return Err(repeated_failure_error());
                    }
                    Some(WebToolFailure {
                        retryable: true, ..
                    }) => {
                        self.network_retries.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }

            state.network_attempts = state.network_attempts.saturating_add(1);
            state.in_flight = true;
            drop(state);

            let execute = run
                .take()
                .expect("one caller can start at most one web network attempt");
            let result = execute().await;
            let cached = CachedWebOutcome::from_result(&result);
            let mut state = entry.state.lock().await;
            state.in_flight = false;
            state.outcome = Some(cached);
            drop(state);
            entry.completed.notify_waiters();
            return result;
        }
    }

    pub(crate) fn stats(&self) -> WebAttemptStats {
        WebAttemptStats {
            dedup_replays: self.dedup_replays.load(Ordering::Relaxed),
            network_retries: self.network_retries.load(Ordering::Relaxed),
            repeated_failures: self.repeated_failures.load(Ordering::Relaxed),
        }
    }

    fn clear(&self) {
        self.entries.lock().clear();
    }
}

pub(crate) struct WebAttemptTurnGuard(Arc<WebAttemptLedger>);

impl Drop for WebAttemptTurnGuard {
    fn drop(&mut self) {
        let stats = self.0.stats();
        tracing::debug!(
            dedup_replays = stats.dedup_replays,
            network_retries = stats.network_retries,
            repeated_failures = stats.repeated_failures,
            "native web attempt ledger completed"
        );
        self.0.clear();
    }
}

fn repeated_failure_error() -> ToolError {
    let failure = WebToolFailure::for_code(WebToolErrorCode::RepeatedFailure);
    ToolError::new(ToolErrorKind::Execution, failure.prompt_text()).with_details(
        serde_json::to_value(&failure).expect("web failure envelope is JSON-serializable"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use xai_grok_tools::types::output::{TextOutput, ToolOutput};

    fn success(text: &str) -> ToolRunResult {
        ToolRunResult {
            output: ToolOutput::Text(TextOutput::from(text)),
            prompt_text: text.to_owned(),
            effective_tool_name: None,
            external_content: None,
        }
    }

    fn failure(code: WebToolErrorCode) -> ToolError {
        let failure = WebToolFailure::for_code(code);
        ToolError::new(ToolErrorKind::Execution, failure.prompt_text())
            .with_details(serde_json::to_value(failure).unwrap())
    }

    #[test]
    fn fingerprints_ignore_object_order_and_shell_context() {
        let left = serde_json::json!({
            "q": "current release",
            "filters": {"b": 2, "a": 1},
            "_grok_codex_context": {"max_output_tokens": 999999}
        });
        let right = serde_json::json!({
            "filters": {"a": 1, "b": 2},
            "q": "current release"
        });
        assert_eq!(
            WebAttemptLedger::fingerprint("web_search", &left),
            WebAttemptLedger::fingerprint("web_search", &right)
        );
        assert_ne!(
            WebAttemptLedger::fingerprint("web_search", &right),
            WebAttemptLedger::fingerprint("web_fetch", &right)
        );
    }

    #[tokio::test]
    async fn concurrent_identical_calls_share_one_result() {
        let ledger = Arc::new(WebAttemptLedger::default());
        let key = WebAttemptLedger::fingerprint("web_search", &serde_json::json!({"q": "x"}));
        let calls = Arc::new(AtomicUsize::new(0));
        let first = {
            let ledger = Arc::clone(&ledger);
            let calls = Arc::clone(&calls);
            tokio::spawn(async move {
                ledger
                    .run(key, || async move {
                        calls.fetch_add(1, Ordering::SeqCst);
                        tokio::task::yield_now().await;
                        Ok(success("shared"))
                    })
                    .await
            })
        };
        let second = {
            let ledger = Arc::clone(&ledger);
            let calls = Arc::clone(&calls);
            tokio::spawn(async move {
                ledger
                    .run(key, || async move {
                        calls.fetch_add(1, Ordering::SeqCst);
                        Ok(success("unexpected"))
                    })
                    .await
            })
        };

        assert_eq!(first.await.unwrap().unwrap().prompt_text, "shared");
        assert_eq!(second.await.unwrap().unwrap().prompt_text, "shared");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(ledger.stats().dedup_replays, 1);
    }

    #[tokio::test]
    async fn retryable_failure_runs_twice_then_becomes_repeated_failure() {
        let ledger = WebAttemptLedger::default();
        let key = WebAttemptLedger::fingerprint("web_search", &serde_json::json!({"q": "x"}));
        let calls = AtomicUsize::new(0);

        for _ in 0..2 {
            let result = ledger
                .run(key, || async {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Err(failure(WebToolErrorCode::Timeout))
                })
                .await;
            assert!(result.is_err());
        }
        let repeated = ledger
            .run(key, || async {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(success("must not run"))
            })
            .await
            .unwrap_err();

        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(
            repeated
                .details
                .and_then(|details| details.get("code").cloned()),
            Some(serde_json::json!("repeated_failure"))
        );
        assert_eq!(ledger.stats().network_retries, 1);
        assert_eq!(ledger.stats().repeated_failures, 1);
    }

    #[tokio::test]
    async fn non_retryable_failure_is_replayed_without_network() {
        let ledger = WebAttemptLedger::default();
        let key = WebAttemptLedger::fingerprint(
            "web_fetch",
            &serde_json::json!({"url": "https://example.invalid"}),
        );
        let calls = AtomicUsize::new(0);
        for _ in 0..2 {
            let result = ledger
                .run(key, || async {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Err(failure(WebToolErrorCode::SsrfBlocked))
                })
                .await;
            assert!(result.is_err());
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(ledger.stats().dedup_replays, 1);
    }
}
