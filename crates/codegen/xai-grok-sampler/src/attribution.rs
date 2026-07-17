//! 401 attribution callback hook for the sampling client.
//!
//! Every 401 response site can optionally emit an attribution event without
//! exposing request credentials to telemetry code.
//!
//! `xai-grok-sampler` is intentionally decoupled from `xai-grok-shell`
//! (no shell types, no logging crate, no auth-manager dependency). The
//! caller wires an implementation of [`Auth401AttributionCallback`]
//! into [`crate::SamplerConfig::attribution_callback`]; the sampler
//! invokes the callback at each UNAUTHORIZED arm with endpoint identity only.
//!
//! When the callback is `None` (the default), the 401 sites are silent
//! and return the same `SamplingError::Auth` they would otherwise.

use std::sync::Arc;

/// A logical 401-emitting site inside the sampling client. The string
/// identifier ends up in the consumer field of the attribution event
/// so downstream queries can break down 401s by API path.
///
/// # Scope: sampler endpoints only
///
/// This enum enumerates the six HTTP endpoints owned by
/// `SamplingClient` (chat completions, responses, messages -- each in
/// streaming and non-streaming form). It does *not* cover image
/// generation, video generation, web search, or embedding -- those
/// tools live in `xai-grok-tools`
/// (`crates/codegen/xai-grok-tools/src/implementations/`), have their
/// own HTTP clients that do not flow through `SamplingClient`, and
/// hook into the `xai_grok_tools::ApiKeyProvider` trait rather than
/// this enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SamplingConsumer {
    /// `chat_completion_stream`: OpenAI-compatible streaming OpenAI Chat Completions API.
    ChatCompletionsStream,
    /// `chat_completion`: OpenAI-compatible non-streaming OpenAI Chat Completions API.
    ChatCompletions,
    /// `create_response_stream`: Responses API streaming.
    ResponsesStream,
    /// `create_response`: Responses API non-streaming.
    Responses,
    /// `messages_stream`: Anthropic Messages API streaming.
    MessagesStream,
    /// `messages`: Anthropic Messages API non-streaming.
    Messages,
}

impl SamplingConsumer {
    /// Stable string identifier for this emit site.
    pub fn as_endpoint(self) -> &'static str {
        match self {
            Self::ChatCompletionsStream => "chat_completions_stream",
            Self::ChatCompletions => "chat_completions",
            Self::ResponsesStream => "responses_stream",
            Self::Responses => "responses",
            Self::MessagesStream => "messages_stream",
            Self::Messages => "messages",
        }
    }
}

/// Hook invoked by [`crate::SamplingClient`] at every 401 response site.
///
/// Implementations must be cheap to invoke and must not block. They
/// run inside the request's response-handling path and any latency
/// they add is paid by the user-visible 401 error path.
//
// The `Debug` bound is a structural requirement: [`crate::SamplerConfig`]
// derives `Debug` and carries an `Option<Arc<dyn Auth401AttributionCallback>>`
// field, which only compiles when the trait is `Debug`. Do not remove
// the bound when factoring this trait out -- it will break
// `derive(Debug)` on `SamplerConfig`.
pub trait Auth401AttributionCallback: Send + Sync + std::fmt::Debug {
    /// Record a 401 attribution event for one logical 401 response.
    ///
    /// `sent_bearer` reports only whether the request carried bearer-style
    /// authentication. No credential bytes or derived identifiers cross this
    /// boundary.
    fn record_401(&self, consumer: SamplingConsumer, sent_bearer: bool);
}

/// Shared, cheap-to-clone alias for the attribution callback.
pub type SharedAttributionCallback = Arc<dyn Auth401AttributionCallback>;
