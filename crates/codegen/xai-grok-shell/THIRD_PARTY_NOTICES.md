# Third-Party Notices

This crate contains modified adaptations of public client behavior from the
[openai/codex](https://github.com/openai/codex) project:

- `src/auth/codex/`, adapted from `codex-rs/login/` for browser PKCE,
  device-code login, token refresh/revocation, account identity, and secure
  storage.
- `src/remote/openai_codex_catalog.rs` and
  `src/agent/models/codex_catalog.rs`, adapted from
  `codex-rs/model-provider/` and `codex-rs/model-provider-info/` for
  authenticated account-scoped model discovery.
- `src/agent/models.rs`, `src/session/provider/openai_codex.rs`, and
  `src/session/acp_session_impl/turn.rs`, adapted from the current Codex v2
  multi-agent effort and provider-session policies so Ultra uses Grok Build's
  native subagent loop while preserving its permission, binding, and nesting
  rules.
- `src/session/acp_session_impl/tool_calls.rs`, adapted from
  `codex-rs/ext/image-generation/src/tool.rs` so generated and edited images
  remain multimodal tool-result history while Grok Build retains its own
  media storage and conversation pipeline.

These files have been extensively modified to use a Grok Build-owned scoped
credential record, preserve Grok Build's existing agent/session/TUI runtime,
and fail closed across provider and account boundaries. They are not verbatim
copies. This is the prominent modification notice required by Apache License
2.0 section 4(b).

Copyright 2025 OpenAI

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

## Moonshot AI / Kimi Code CLI

The Kimi Code provider under `src/auth/kimi_code/` and its provider/session
routing adapt the public managed catalog, usage, protocol-selection, cache
binding, and session-affinity behavior from the
[Moonshot AI Kimi Code CLI](https://github.com/MoonshotAI/kimi-code). The code
has been translated to Rust and substantially modified for Grok Build's
existing authentication store, model manager, ACP runtime, sessions, and
provider-isolation rules. It does not import official Kimi CLI credentials or
send official-client identity headers.

MIT License

Copyright (c) 2026 Moonshot AI

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
