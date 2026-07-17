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
- `src/agent/models.rs` and `src/session/acp_session_impl/turn.rs`, adapted
  from the current Codex v2 multi-agent effort policy so Ultra uses Grok
  Build's native subagent loop while preserving its permission and nesting
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
