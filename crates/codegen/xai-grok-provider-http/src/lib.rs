//! Provider-scoped HTTP route construction.
//!
//! This leaf crate keeps operating-system proxy and PAC discovery out of
//! reqwest's workspace-wide feature set. Callers must opt in explicitly at a
//! first-class provider boundary; unrelated xAI, Kimi, Z.AI, Custom, telemetry,
//! update, and generic tool clients retain their existing transport behavior.

pub mod extra_ca;
pub mod outbound_proxy;

pub use extra_ca::{
    ENV_GROK_EXTRA_CA_BUNDLE, MAX_EXTRA_CA_BUNDLE_BYTES, extra_root_ders,
    with_extra_root_certificates, with_extra_root_certificates_blocking,
};

pub use outbound_proxy::{
    BuildRouteAwareHttpClientError, ClientRouteClass, OpenAiCodexClientPool,
    build_openai_codex_client,
};
