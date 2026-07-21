# Third-party notices

## openai/codex

Source: https://github.com/openai/codex

Upstream revision: `c0cd337766ff27a75623c5baba199389f94f2ab3`

Upstream paths:

- `codex-rs/http-client/src/outbound_proxy.rs`
- `codex-rs/http-client/src/outbound_proxy/macos.rs`
- `codex-rs/http-client/src/outbound_proxy/windows.rs`

License: Apache License 2.0

Copyright 2025 OpenAI

The proxy resolver in this crate is adapted and extensively modified for Grok
Build Enhanced. It exposes only provider-scoped reqwest client construction,
uses fixed-shape redacted failures, and does not import Codex's HTTP client,
custom-CA, telemetry, WebSocket, or application architecture. The repository
root `LICENSE` contains the Apache License 2.0 text.
