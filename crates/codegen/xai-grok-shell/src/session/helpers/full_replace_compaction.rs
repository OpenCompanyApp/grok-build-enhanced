//! grok-build's L5 wiring onto the shared full-replace engine
//! (`xai_grok_compaction::code_compaction`).
//!
//! The shared engine drives the sample → retry → degenerate/failure
//! classification loop via [`sample_full_replace_summary`](xai_grok_compaction::sample_full_replace_summary);
//! this module adapts grok-build's transport and telemetry to its two seams:
//!
//! - [`ShellCompactionSampler`] wraps
//!   [`generate_session_compact`](crate::session::helpers::session_compact::generate_session_compact)
//!   as the shared [`CompactionSampler`]. It also stashes the full
//!   [`CompactOutput`] of the last successful call so the L5 loop can still
//!   record the streaming telemetry (TTFT / stream span / stop reason) that
//!   the shared [`LlmCompactionOutput`] doesn't model.
//! - [`ShellFullReplaceObserver`] collects the per-attempt
//!   [`CompactionAttempt`] rows, rejection counters, and emits the
//!   `CompactionRetryDegraded` event — preserving the pre-migration telemetry.
//!
//! The verbatim → fitted → lossy **input ladder** and auto-compaction
//! suppression stay in L5 (`compaction.rs`), driven by the
//! `context_overflow` / `deterministic` flags on
//! [`FullReplaceError`](xai_grok_compaction::FullReplaceError).

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use agent_client_protocol as acp;
use async_trait::async_trait;
use xai_grok_compaction::{
    CompactionPrompt, CompactionSampleError, CompactionSampler, FullReplaceAttemptOutcome,
    FullReplaceObserver, LlmCompactionOutput,
};
use xai_grok_sampler::SamplerConfig as SamplingConfig;
use xai_grok_sampling_types::{ConversationItem, HostedTool, ToolSpec};
use xai_grok_telemetry::events::{CompactionRetryDegraded, CompactionTrigger};

use xai_chat_state::compaction_utils::{
    CompactionAttempt, MAX_CAPTURED_SUMMARY_CHARS, bound_captured_output,
};

use crate::sampling::Client as OaiCompatClient;
use crate::session::helpers::session_compact::{
    CompactFailure, CompactOutput, build_compaction_chat_history, generate_session_compact,
};

/// Wraps `generate_session_compact` as the shared engine's
/// [`CompactionSampler`] for grok-build's full-replace pass.
///
/// Holds the per-call request context the seam does not carry (tools, client,
/// session, config) and stashes the last successful [`CompactOutput`] so the
/// caller can recover the streaming telemetry not modeled by
/// [`LlmCompactionOutput`].
///
/// The summarization prompt is selected here by `use_short_prompt` (the
/// short-prompt harness uses the short self-summarization prompt; everyone
/// else the structured grok-build prompt), so the shared `CompactionPrompt`
/// the engine passes is ignored — the engine builds the grok-build prompt,
/// which equals what `build_compaction_chat_history(.., false)` appends, and
/// the short-prompt harness needs its own variant the engine can't produce.
struct CompactionSamplingRoute {
    client: OaiCompatClient,
    config: SamplingConfig,
}

pub(crate) struct ShellCompactionSampler {
    use_short_prompt: bool,
    user_context: Option<String>,
    tools: Vec<ToolSpec>,
    hosted_tools: Vec<HostedTool>,
    primary: CompactionSamplingRoute,
    fallback: Option<CompactionSamplingRoute>,
    fallback_active: AtomicBool,
    session_id: acp::SessionId,
    /// Per-chunk idle timeout forwarded to `generate_session_compact`: a stalled
    /// summarizer stream (no model-output chunk for this long) fails instead of
    /// hanging.
    idle_timeout: Duration,
    /// Wall-clock budget (secs) forwarded to `generate_session_compact` as the
    /// reasoning-runaway backstop; `0` disables it.
    wall_clock_budget_secs: u64,
    /// Full output of the most recent successful sample (for L5 telemetry).
    last_success: Mutex<Option<CompactOutput>>,
    /// Model used by the most recent request. This keeps persisted compaction
    /// artifacts accurate when the previous-model route falls back.
    last_attempted_model: Mutex<String>,
}

impl ShellCompactionSampler {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        use_short_prompt: bool,
        user_context: Option<String>,
        tools: Vec<ToolSpec>,
        hosted_tools: Vec<HostedTool>,
        client: OaiCompatClient,
        session_id: acp::SessionId,
        sampling_config: SamplingConfig,
        fallback: Option<(OaiCompatClient, SamplingConfig)>,
        idle_timeout: Duration,
        wall_clock_budget_secs: u64,
    ) -> Self {
        let last_attempted_model = sampling_config.model.clone();
        Self {
            use_short_prompt,
            user_context,
            tools,
            hosted_tools,
            primary: CompactionSamplingRoute {
                client,
                config: sampling_config,
            },
            fallback: fallback.map(|(client, config)| CompactionSamplingRoute { client, config }),
            fallback_active: AtomicBool::new(false),
            session_id,
            idle_timeout,
            wall_clock_budget_secs,
            last_success: Mutex::new(None),
            last_attempted_model: Mutex::new(last_attempted_model),
        }
    }

    /// Take the [`CompactOutput`] of the most recent successful sample, if any.
    pub(crate) fn take_last_success(&self) -> Option<CompactOutput> {
        self.last_success.lock().unwrap().take()
    }

    /// Return the model used by the latest primary or fallback request.
    pub(crate) fn last_attempted_model(&self) -> String {
        self.last_attempted_model.lock().unwrap().clone()
    }

    async fn sample_route(
        &self,
        chat_history: Vec<ConversationItem>,
        route: &CompactionSamplingRoute,
    ) -> Result<CompactOutput, CompactFailure> {
        *self.last_attempted_model.lock().unwrap() = route.config.model.clone();
        generate_session_compact(
            chat_history,
            self.tools.clone(),
            self.hosted_tools.clone(),
            route.client.clone(),
            self.session_id.clone(),
            &route.config,
            self.idle_timeout,
            self.wall_clock_budget_secs,
        )
        .await
    }
}

#[async_trait]
impl CompactionSampler for ShellCompactionSampler {
    type Item = ConversationItem;

    async fn sample_compaction(
        &self,
        turns: &[ConversationItem],
        _prompt: &CompactionPrompt,
        _timeout: Duration,
    ) -> Result<LlmCompactionOutput, CompactionSampleError> {
        // Append the harness-selected summarization prompt as the final user
        // message (compat short vs structured grok-build), ignoring the shared
        // engine's `_prompt` (see the struct doc).
        let chat_history = build_compaction_chat_history(
            turns.to_vec(),
            self.user_context.as_deref(),
            self.use_short_prompt,
        );

        let output = if self.fallback_active.load(Ordering::Relaxed) {
            let fallback = self
                .fallback
                .as_ref()
                .expect("fallback_active requires a fallback route");
            self.sample_route(chat_history, fallback).await
        } else {
            match self.sample_route(chat_history.clone(), &self.primary).await {
                Err(failure)
                    if self.fallback.is_some() && failure.should_retry_with_current_model() =>
                {
                    let fallback = self.fallback.as_ref().unwrap();
                    self.fallback_active.store(true, Ordering::Relaxed);
                    tracing::warn!(
                        previous_model = %self.primary.config.model,
                        current_model = %fallback.config.model,
                        "previous-model Codex compaction failed; retrying with active model"
                    );
                    self.sample_route(chat_history, fallback).await
                }
                result => result,
            }
        };

        match output {
            Ok(output) => {
                let response = output.content.clone();
                *self.last_success.lock().unwrap() = Some(output);
                Ok(LlmCompactionOutput {
                    response,
                    thinking: String::new(),
                })
            }
            Err(failure) => Err(compact_failure_to_sample_error(failure)),
        }
    }
}

/// Map grok-build's [`CompactFailure`] onto the shared engine's
/// [`CompactionSampleError`] so the shared retry loop classifies it the same
/// way the in-shell loop did:
///
/// - `Deterministic` → [`CompactionSampleError::Build`] (whose
///   `is_deterministic()` is `true`); a context-length overflow keeps its
///   message text so the engine's `is_context_length_error` check fires and
///   sets `context_overflow`.
/// - `Transient` → [`CompactionSampleError::Other`] (`is_deterministic()` is
///   `false`), so the engine retries it.
fn compact_failure_to_sample_error(failure: CompactFailure) -> CompactionSampleError {
    let (deterministic, err) = match failure {
        CompactFailure::Deterministic(err)
        | CompactFailure::DeterministicWithCurrentModelFallback(err) => (true, err),
        CompactFailure::Transient(err) => (false, err),
    };
    let message = acp_error_message(&err);
    if deterministic {
        CompactionSampleError::Build(message)
    } else {
        CompactionSampleError::Other(anyhow::anyhow!(message))
    }
}

/// Render the human-readable detail an `acp::Error` carries in its `data`
/// field (where `classify_*` stash `"compact failed: <upstream>"`).
fn acp_error_message(err: &acp::Error) -> String {
    err.data
        .as_ref()
        .and_then(|d| d.as_str())
        .unwrap_or("<no data>")
        .to_string()
}

/// Collected telemetry from a full-replace pass, drained by the L5 loop after
/// the shared engine returns.
pub(crate) struct FullReplaceTelemetry {
    pub attempts: u32,
    pub attempt_details: Vec<CompactionAttempt>,
    pub degenerate_rejections: u32,
    pub transient_rejections: u32,
    pub deterministic_rejections: u32,
    /// Raw text of the last degenerate (rejected) summary, for the artifact.
    pub last_rejected_summary: Option<String>,
}

#[derive(Default)]
struct ObserverState {
    attempts: u32,
    attempt_details: Vec<CompactionAttempt>,
    degenerate_rejections: u32,
    transient_rejections: u32,
    deterministic_rejections: u32,
    last_rejected_summary: Option<String>,
    last_error_msg: Option<String>,
}

/// [`FullReplaceObserver`] that reproduces grok-build's per-attempt telemetry:
/// `CompactionAttempt` rows, rejection counters, the `CompactionRetryDegraded`
/// event, and the warn/error tracing — without the shared engine depending on
/// a telemetry backend.
pub(crate) struct ShellFullReplaceObserver {
    trigger: CompactionTrigger,
    context_window: u64,
    compaction_id: String,
    session_id: String,
    estimated_input_tokens: u64,
    retry_delay_secs: u64,
    state: Mutex<ObserverState>,
}

impl ShellFullReplaceObserver {
    pub(crate) fn new(
        trigger: CompactionTrigger,
        context_window: u64,
        compaction_id: String,
        session_id: String,
        estimated_input_tokens: u64,
        retry_delay_secs: u64,
    ) -> Self {
        Self {
            trigger,
            context_window,
            compaction_id,
            session_id,
            estimated_input_tokens,
            retry_delay_secs,
            state: Mutex::new(ObserverState::default()),
        }
    }

    /// Cumulative number of attempts so far (across all input-ladder stages).
    /// Read mid-loop to label the `input_overflow` retry event.
    pub(crate) fn attempt_count(&self) -> u32 {
        self.state.lock().unwrap().attempts
    }

    /// Whether any attempt so far produced a degenerate summary — lets the L5
    /// loop distinguish degenerate-exhausted from empty-exhausted.
    pub(crate) fn degenerate_seen(&self) -> bool {
        self.state.lock().unwrap().degenerate_rejections > 0
    }

    /// The most recent rendered error/diagnostic detail, for `last_error`.
    pub(crate) fn last_error_message(&self) -> Option<String> {
        self.state.lock().unwrap().last_error_msg.clone()
    }

    /// Drain the collected telemetry. The cumulative attempt count spans all
    /// input-ladder stages because the same observer instance is shared across
    /// every per-stage call.
    pub(crate) fn into_telemetry(self) -> FullReplaceTelemetry {
        let s = self.state.into_inner().unwrap();
        FullReplaceTelemetry {
            attempts: s.attempts,
            attempt_details: s.attempt_details,
            degenerate_rejections: s.degenerate_rejections,
            transient_rejections: s.transient_rejections,
            deterministic_rejections: s.deterministic_rejections,
            last_rejected_summary: s.last_rejected_summary,
        }
    }
}

impl FullReplaceObserver for ShellFullReplaceObserver {
    fn on_attempt(&self, _attempt: u32, outcome: &FullReplaceAttemptOutcome<'_>) {
        let mut s = self.state.lock().unwrap();
        // The shared `attempt` resets per ladder stage; keep a cumulative count
        // so artifact rows match the pre-migration numbering.
        s.attempts += 1;
        let attempt = s.attempts;

        match outcome {
            FullReplaceAttemptOutcome::Success { summary } => {
                s.attempt_details.push(CompactionAttempt {
                    attempt,
                    outcome: "success".to_string(),
                    summary_chars: summary.chars().count() as u64,
                    summary: None,
                    error: None,
                });
            }
            FullReplaceAttemptOutcome::Degenerate {
                summary,
                will_retry,
            } => {
                s.degenerate_rejections += 1;
                let summary_chars = summary.chars().count();
                s.attempt_details.push(CompactionAttempt {
                    attempt,
                    outcome: "degenerate".to_string(),
                    summary_chars: summary_chars as u64,
                    summary: Some(bound_captured_output(summary, MAX_CAPTURED_SUMMARY_CHARS)),
                    error: None,
                });
                s.last_rejected_summary = Some((*summary).to_string());
                s.last_error_msg = Some(format!(
                    "compact failed: degenerate summary \
                     ({summary_chars} chars for ~{} input tokens)",
                    self.estimated_input_tokens
                ));
                if *will_retry {
                    xai_grok_telemetry::session_ctx::log_event(CompactionRetryDegraded {
                        trigger: self.trigger,
                        reason: "degenerate_summary",
                        from_stage: None,
                        to_stage: None,
                        summary_chars: Some(summary_chars as u64),
                        attempt,
                        context_window: self.context_window,
                        compaction_id: self.compaction_id.clone(),
                    });
                    tracing::warn!(
                        session_id = %self.session_id,
                        attempt,
                        summary_chars,
                        estimated_input_tokens = self.estimated_input_tokens,
                        retry_delay_secs = self.retry_delay_secs,
                        "Compaction produced a degenerate summary, retrying in {} seconds...",
                        self.retry_delay_secs
                    );
                } else {
                    tracing::error!(
                        session_id = %self.session_id,
                        attempt,
                        summary_chars,
                        estimated_input_tokens = self.estimated_input_tokens,
                        "Compaction produced only degenerate summaries after max retries"
                    );
                }
            }
            FullReplaceAttemptOutcome::EmptyResponse { .. } => {
                // The shell surfaces an empty response as a transient error
                // (`generate_session_compact` returns `Transient`), so it never
                // reaches the shared `Ok("")` branch; handle defensively.
                s.transient_rejections += 1;
                let msg = "compact failed: model returned empty response".to_string();
                s.attempt_details.push(CompactionAttempt {
                    attempt,
                    outcome: "transient".to_string(),
                    summary_chars: 0,
                    summary: None,
                    error: Some(msg.clone()),
                });
                s.last_error_msg = Some(msg);
            }
            FullReplaceAttemptOutcome::Failure {
                message,
                deterministic,
                context_overflow,
                will_retry,
            } => {
                // A context overflow is recorded as a `deterministic` attempt
                // (matching the pre-migration row) but does NOT count toward
                // `deterministic_rejections` — the L5 ladder steps down on it
                // and tracks its own `input_overflow_rejections`.
                if *deterministic {
                    if !*context_overflow {
                        s.deterministic_rejections += 1;
                        tracing::error!(
                            session_id = %self.session_id,
                            attempt,
                            error = %message,
                            "Compaction failed (deterministic error class, no further retries)"
                        );
                    }
                    s.attempt_details.push(CompactionAttempt {
                        attempt,
                        outcome: "deterministic".to_string(),
                        summary_chars: 0,
                        summary: None,
                        error: Some((*message).to_string()),
                    });
                } else {
                    s.transient_rejections += 1;
                    s.attempt_details.push(CompactionAttempt {
                        attempt,
                        outcome: "transient".to_string(),
                        summary_chars: 0,
                        summary: None,
                        error: Some((*message).to_string()),
                    });
                    if *will_retry {
                        tracing::warn!(
                            session_id = %self.session_id,
                            attempt,
                            retry_delay_secs = self.retry_delay_secs,
                            error = %message,
                            "Compaction attempt {} failed, retrying in {} seconds...",
                            attempt,
                            self.retry_delay_secs
                        );
                    } else {
                        tracing::error!(
                            session_id = %self.session_id,
                            attempt,
                            error = %message,
                            "Compaction failed after max retries"
                        );
                    }
                }
                s.last_error_msg = Some((*message).to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! Scenario: previous-model compaction fails at the inference boundary.
    //! Responsibility: retry once with the active model using real production
    //! sampler wiring and only the HTTP inference service stubbed. Run with
    //! `cargo test -p xai-grok-shell previous_model_failure_uses_active_model_fallback --lib`.

    use super::*;
    use reqwest::header::{AUTHORIZATION, HeaderValue};
    use serde_json::json;
    use std::sync::Arc;
    use xai_grok_test_support::{MockInferenceServer, ScriptedResponse};

    struct TestCodexRequestAuth;

    impl xai_grok_sampler::RequestAuth for TestCodexRequestAuth {
        fn apply(
            &self,
            headers: &mut reqwest::header::HeaderMap,
        ) -> Result<xai_grok_sampling_types::CredentialBinding, xai_grok_sampler::RequestAuthError>
        {
            let mut authorization = HeaderValue::from_static("Bearer test-compaction-token");
            authorization.set_sensitive(true);
            headers.insert(AUTHORIZATION, authorization);
            let mut account = HeaderValue::from_static("test-compaction-account");
            account.set_sensitive(true);
            headers.insert("chatgpt-account-id", account);
            let mut binding = xai_grok_sampling_types::CredentialBinding::openai_codex(Some(
                "test-compaction-record".to_owned(),
            ));
            binding.generation = 1;
            Ok(binding)
        }
    }

    #[tokio::test]
    async fn previous_model_failure_uses_active_model_fallback() {
        let server = MockInferenceServer::start().await.unwrap();
        server.enqueue_response(
            "/v1/responses",
            ScriptedResponse::json(
                400,
                json!({"error": {"type": "invalid_request_error", "message": "unsupported model"}}),
            ),
        );
        server.set_response("Summary produced by the active fallback model.");
        let mut previous_config = xai_grok_sampler::SamplerConfig::openai_codex("gpt-previous");
        previous_config.base_url = server.url();
        previous_config.max_retries = Some(0);
        previous_config.request_auth = Some(Arc::new(TestCodexRequestAuth));
        let active_config = xai_grok_sampler::SamplerConfig {
            model: "gpt-current".to_owned(),
            ..previous_config.clone()
        };
        let previous_client =
            xai_grok_sampler::SamplingClient::new(previous_config.clone()).unwrap();
        let active_client = xai_grok_sampler::SamplingClient::new(active_config.clone()).unwrap();
        let sampler = ShellCompactionSampler::new(
            false,
            None,
            Vec::new(),
            Vec::new(),
            previous_client,
            acp::SessionId::new("compaction-model-fallback-test"),
            previous_config,
            Some((active_client, active_config)),
            Duration::from_secs(5),
            0,
        );
        let prompt = CompactionPrompt {
            system: String::new(),
            user: String::new(),
        };

        let result = sampler
            .sample_compaction(
                &[
                    ConversationItem::system("system"),
                    ConversationItem::user("summarize"),
                ],
                &prompt,
                Duration::ZERO,
            )
            .await
            .expect("active-model fallback succeeds");

        assert_eq!(
            result.response,
            "Summary produced by the active fallback model."
        );
        let models = server
            .request_bodies()
            .into_iter()
            .filter_map(|body| body["model"].as_str().map(str::to_owned))
            .collect::<Vec<_>>();
        assert_eq!(models, vec!["gpt-previous", "gpt-current"]);
        assert!(server.has_responses_request());
    }
}
