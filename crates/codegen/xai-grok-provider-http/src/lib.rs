//! Provider-scoped HTTP route construction.
//!
//! This leaf crate keeps operating-system proxy and PAC discovery out of
//! reqwest's workspace-wide feature set. Callers must opt in explicitly at a
//! first-class provider boundary; unrelated xAI, Kimi, Z.AI, Custom, telemetry,
//! update, and generic tool clients retain their existing transport behavior.

pub mod outbound_proxy;

pub use outbound_proxy::{
    BuildRouteAwareHttpClientError, ClientRouteClass, build_openai_codex_client,
};
