//! Provider-bound session runtime construction.
//!
//! This module is crate-private on purpose: sessions may consume a complete
//! sampler/tool runtime, but callers outside the shell must not rebuild pieces
//! of provider authentication independently.

mod openai_codex;

pub(crate) use openai_codex::{
    BoundProviderRuntime, ProviderBindingError, ProviderModelRoute, bind_provider_runtime,
    pin_provider_candidate_to_active_record,
};
pub(in crate::session) use openai_codex::{inject_codex_multi_agent_policy, resolve_fast_mode};
