use reqwest::header::{HeaderMap, HeaderValue};

use super::headers::CODEX_TURN_STATE_HEADER;

/// Sticky-routing state returned by the ChatGPT Codex backend. The value is
/// opaque and credential-adjacent: it is kept only in memory, marked
/// sensitive, and scoped to the request ID shared by one Grok user/tool turn.
struct CodexTurnState {
    request_id: String,
    value: HeaderValue,
}

/// Actor-scoped owner for the opaque ChatGPT sticky-routing value.
///
/// A sampling client is rebuilt for every sampler submission (and may also be
/// rebuilt during transport recovery), while one user turn spans several
/// submissions when tools are called. Keeping the state in this shared owner
/// preserves the provider's per-turn contract without persisting or exposing
/// the value.
#[derive(Clone, Default)]
pub(crate) struct CodexTurnStateStore {
    inner: std::sync::Arc<std::sync::Mutex<Option<CodexTurnState>>>,
}

impl CodexTurnStateStore {
    pub(crate) fn apply(
        &self,
        builder: reqwest::RequestBuilder,
        request_id: &str,
    ) -> reqwest::RequestBuilder {
        if request_id.is_empty() {
            return builder;
        }
        let mut state = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match state.as_ref() {
            Some(turn_state) if turn_state.request_id == request_id => {
                builder.header(CODEX_TURN_STATE_HEADER, turn_state.value.clone())
            }
            Some(_) => {
                // A new Grok prompt gets a fresh Codex turn. Never replay the
                // sticky-routing value across that boundary.
                *state = None;
                builder
            }
            None => builder,
        }
    }

    pub(crate) fn capture(&self, headers: &HeaderMap, request_id: &str) {
        if request_id.is_empty() {
            return;
        }
        let Some(value) = headers.get(CODEX_TURN_STATE_HEADER) else {
            return;
        };
        let mut value = value.clone();
        value.set_sensitive(true);
        let mut state = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match state.as_ref() {
            // Match the official once-per-turn behavior: the first value is
            // authoritative and remains unchanged throughout tool rounds and
            // retries in this request ID.
            Some(turn_state) if turn_state.request_id == request_id => {}
            _ => {
                *state = Some(CodexTurnState {
                    request_id: request_id.to_string(),
                    value,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use reqwest::header::HeaderName;

    use super::*;

    fn build(store: &CodexTurnStateStore, request_id: &str) -> reqwest::Request {
        store
            .apply(
                reqwest::Client::new().post("https://example.test/responses"),
                request_id,
            )
            .build()
            .expect("request should build")
    }

    #[test]
    fn state_survives_clones_only_within_same_request_id() {
        const FIRST_STATE: &str = "opaque-sticky-state";
        const LATER_STATE: &str = "must-not-replace-first-state";

        let store = CodexTurnStateStore::default();
        assert!(
            build(&store, "request-1")
                .headers()
                .get(CODEX_TURN_STATE_HEADER)
                .is_none()
        );

        let mut response_headers = HeaderMap::new();
        response_headers.insert(
            HeaderName::from_static(CODEX_TURN_STATE_HEADER),
            HeaderValue::from_static(FIRST_STATE),
        );
        store.capture(&response_headers, "request-1");

        let cloned = store.clone();
        let mut later_response_headers = HeaderMap::new();
        later_response_headers.insert(
            HeaderName::from_static(CODEX_TURN_STATE_HEADER),
            HeaderValue::from_static(LATER_STATE),
        );
        cloned.capture(&later_response_headers, "request-1");

        let same_turn = build(&cloned, "request-1");
        let replayed = same_turn
            .headers()
            .get(CODEX_TURN_STATE_HEADER)
            .expect("same turn should replay sticky state");
        assert!(replayed.as_bytes() == FIRST_STATE.as_bytes());
        assert!(replayed.is_sensitive());

        assert!(
            build(&cloned, "")
                .headers()
                .get(CODEX_TURN_STATE_HEADER)
                .is_none()
        );
        assert!(
            build(&cloned, "request-1")
                .headers()
                .get(CODEX_TURN_STATE_HEADER)
                .is_some()
        );

        assert!(
            build(&cloned, "request-2")
                .headers()
                .get(CODEX_TURN_STATE_HEADER)
                .is_none()
        );
        assert!(
            build(&store, "request-1")
                .headers()
                .get(CODEX_TURN_STATE_HEADER)
                .is_none()
        );
    }
}
