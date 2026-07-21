# Provider-reference refresh ledger — 2026-07-21, second pass

This ledger records the exact source pins, complete commit and changed-path inventories, behavior decisions, local implementation, and validation obligations for the second refresh performed on 2026-07-21. It continues [`docs/upstream-refresh-2026-07-21.md`](upstream-refresh-2026-07-21.md); provider-reference history is evidence, not content-merged application history. The ignored `inspiration/` checkouts are not part of the candidate tree.

## Immutable boundary

- Pre-refresh Enhanced commit: `367c3d3f9e6fb08efcbe4fbceb1eeb1c5e6c2a33`
- Pre-refresh Enhanced tree: `e5500ed2451efcce06d83c914743a82dc9753f38`
- Isolated branch: `refresh/upstream-20260721-r2`
- Fetch-pin commit: `75d3291c1ff9c3fe2ca8a5b0f55ac59f7b8c5e25`
- Fetch-pin tree: `d7cdcfe5ee3fa9dd28d3bb732bd85d08cd85decb`
- Check timestamp: `2026-07-21T11:37:07Z`
- Grok Build remained at commit `a881e6703f46b01d8c7d4a5437683546df30449d`, tree `e3a013ffc66a6dd77ec1c35dfe261741bcf84928`.
- Because the Grok commit and tree did not advance, the existing acknowledgement remains exact and this pass must not create a duplicate zero-tree-delta marker.

## Advanced source pins

| Source | Previous reviewed commit / tree | Audited commit / tree | Range | Changed paths | Ancestor |
| --- | --- | --- | ---: | ---: | --- |
| OpenAI Codex | `c0cd337766ff27a75623c5baba199389f94f2ab3` / `b476cd40265f1ec474169eedfb7dabc2924c4d7c` | `b9800de4867e500a92add3cde795cf4790306d0f` / `beacdfb5b2d0b413b5b255ac61b96b9aeb79e947` | 22 commits | 131 | yes |
| OpenCode | `849c2598abc7d2b40261e74b5826bc74ffc78308` / `1484a4c6b34f52f4d77678d615709777a4962f28` | `cb562b2c6289c2eee707078f9ab644cbe1d3d8a9` / `4a6c7f446d849918b1a4259e2fcf6d0dba581dcb` | 2 commits | 1 | yes |
| Kimi Code | `c2d7bebd04106473bb4dbab2903756aa3f14a880` / `6372ff7ef2b4e61f6116178c9af2256d7c4e995c` | `a3699dd6aa7b41efd3129a117007d195282379fd` / `5e219da00781cafc0caf475c2bd708cfe09179f3` | 13 commits | 522 | yes |
| CodexBar | `f8636cb37eb0f96d261604ee94e6481496aadfeb` / `60064e5429e48694b9395f856fa3c6d5af051b4c` | `cc8da27cec92029a6435bfee4a703a719290234e` / `e41036396d949aed3c579a52af74b1b8bab780f6` | 5 commits | 21 | yes |

The other eight tracked sources were fetched and remained byte-identical at their recorded reviewed revisions: Grok Build, OpenCode Codex auth, Warp themes, Kimi CLI, Z.AI Python SDK, Z.AI coding plugins, GLM-5, and the Z.AI usage-browser reference.

## Behavior inventory

The outcomes below classify observable behavior separately from source architecture. `adopt` changes the Enhanced candidate; `already equivalent` cites an existing local contract; and `not applicable` is limited to replacement-application, reference-only, release-route, or managed-proxy surfaces outside the preserved Grok application. There are zero temporary deferrals and zero unclassified behaviors.

| Evidence ID | Source behavior | Outcome and local contract | Focused evidence |
| --- | --- | --- | --- |
| `CDX-B980-EXACT-ROUTE` | Select a route-aware HTTP transport from the complete final request URL, reuse transports by resolved proxy route, and apply that selection consistently to backend requests. | **adopt.** Enhanced adds a bounded Codex-only route pool. Inference, OAuth, catalog, usage, hosted search, and image generation resolve their exact request URL before preparing dynamic credentials; clients are reused by route, not by base URL. xAI, Kimi, Z.AI, Custom, telemetry, and update transports are unchanged. | `codex_pool_resolves_the_complete_request_url`; `codex_pool_reuses_a_client_when_distinct_urls_select_the_same_route`; sampler, shell, and tools request suites. |
| `CDX-B980-REDIRECT-EQ` | Re-resolve PAC/system-proxy routes on redirects. | **already equivalent through a stronger boundary.** Every credential-bearing Enhanced Codex client refuses all redirects, so there is no second hop on which credentials or a stale route can be replayed. Exact first-hop selection is adopted under `CDX-B980-EXACT-ROUTE`; upstream redirect replay is intentionally not imported. | Sampler `dedicated_client_never_reaches_redirect_sink`; usage, web-search, image, catalog, and OAuth redirect-isolation tests. |
| `CDX-B980-HTTP-EQ` | Separate HTTP execution from request logging and keep backend request execution bound to its selected client. | **already equivalent.** Enhanced diagnostics omit URLs, proxy values, bodies, and credential state. Streaming requests now carry the exact route-selected client from preparation through `execute`, while ordinary `RequestBuilder::send` retains its owning client. | Sampler client diagnostics/auth suite and provider HTTP redaction tests. |
| `CDX-B980-CAINFO-NA` | Honor `CARGO_HTTP_CAINFO` inside Codex managed network-proxy environments. | **not applicable.** Enhanced does not import Codex managed permission-profile or network-proxy architecture. Ordinary OS PAC/WPAD/static proxy and environment fallback remain supported without changing Grok permissions. | Source diff and existing provider-scoped route policy. |
| `CDX-B980-APP-NA` | Codex code mode, external-session migration and analytics, app-server thread settings, cloud-environment discovery, sandbox process details, plugin startup/remote routing, spelling, and Wine tests. | **not applicable.** These are Codex replacement-app, sandbox, plugin, analytics, or test surfaces and do not change the ChatGPT Codex auth, catalog, Responses, compaction, usage, or hosted-tool contracts exposed through Grok Build Enhanced. | Complete commit/path inventories below. |
| `CDX-B980-RELEASE-NA` | Alpha/hotfix channels, R2 artifact publication, daemon updater proxying, and an alternate OpenAI installer source. | **not applicable.** Enhanced release checks and downloads are intentionally fork-owned and may not fall back to OpenAI release metadata, installers, npm packages, or artifact buckets. | Release ownership policy in `AGENTS.md` and release contracts. |
| `OC-CB56-APP-NA` | Make OpenCode project rows draggable and regenerate application artifacts. | **not applicable.** OpenCode remains an interoperability reference, not Enhanced's desktop/web project UI. No provider auth or wire contract changed. | Complete commit/path inventories below. |
| `KIMI-A369-VACUOUS` | Drop an assistant turn containing no sendable text, tool calls, signed/encrypted reasoning, or other content so replay cannot permanently wedge a session. | **adopt.** Chat Completions and Messages projections now omit only wholly vacuous assistant turns. Empty reasoning/thinking remains byte-for-byte present when a sibling text or tool call is sendable. Durable `ConversationItem` history is never rewritten. | `conversation_to_chat_messages_omits_output_free_assistant_turn`; `conversation_to_chat_messages_keeps_empty_reasoning_beside_tool_call`; `messages_request_omits_output_free_assistant_turn`; `messages_request_keeps_empty_thinking_beside_assistant_text`; durable-item regression. |
| `KIMI-A369-CANCEL-CACHE-EQ` | Keep user cancellation non-retryable and attach stable prompt-cache affinity in the rebuilt Kimi model wire. | **already equivalent.** `CancellationToken` and `AttemptOutcome::Cancelled` bypass retry; Kimi and Codex already send stable provider-supported cache keys. Generic Custom endpoints intentionally do not receive unsupported OpenAI-only cache fields. | Sampler cancellation/retry contracts and Kimi/Codex cache-affinity tests. |
| `KIMI-A369-EFFORT-EQ` | Expose concrete thinking-effort selections over ACP. | **already equivalent.** Enhanced model metadata and session-scoped effort state already expose and preserve concrete catalog-supported levels without crossing provider/session boundaries. | ACP model-state and Kimi effort mapping tests. |
| `KIMI-A369-HIDDEN-EQ` | Hide synthetic system-trigger messages during resumed replay as well as live display. | **already equivalent.** `PromptOrigin::hide_user_echo_from_scrollback` is the shared live/resume policy for synthetic goal/task prompts. | Prompt-origin and resumed scrollback tests. |
| `KIMI-A369-LONG-SESSION-EQ` | Keep long sessions responsive and bound resumed-history rendering. | **already equivalent.** Enhanced uses lazy/bounded transcript measurement and evicts off-screen render-cache entries; it does not need Kimi's replacement transcript-RPC truncation mechanism. | TUI layout/cache tests and preserved session-history contracts. |
| `KIMI-A369-MCP-EQ` | Reconnect a dropped MCP server and retry a tool call after transport failure. | **already equivalent.** `xai-grok-mcp` already reconnects/retries HTTP and ACP transport failures while preserving the exposed tool set. | MCP reconnect and tool-refresh tests. |
| `KIMI-A369-APP-NA` | Kimi replacement model-engine architecture, syntax colors, VS Code rendering, telemetry schema, custom agent files, and KAP server session/files/tools APIs. | **not applicable.** These belong to Kimi's replacement application/agent engine and do not alter Enhanced's Kimi auth, catalog, inference, usage, hosted search/fetch, retry, or logout adapter contracts. Ordinary upstream telemetry policy is not reshaped. | Complete commit/path inventories below. |
| `CB-CC8D-ZAI-EQ` | Centralize percentage clamping, data-drive pace capability, and stabilize environment-sensitive CodexBar tests. | **already equivalent / research unchanged.** The changes preserve existing provider pace behavior and do not touch CodexBar's Z.AI provider files or Z.AI usage schema. No runtime Z.AI provider, login, credential, or product claim is authorized by this research reference. | Exact CodexBar diff and unchanged Z.AI research conclusions. |

## Provider isolation and legal review

- Codex route selection occurs before each dynamic credential snapshot is prepared. The resulting route-selected client executes that same request; redirects remain refused for OAuth, catalog, inference, usage, hosted search, and image generation.
- The pool is bounded to 16 resolved routes and reuses reqwest connection pools only when exact URL resolution selects the same route. Existing SHA-256 destination cache keys, 60-second success TTL, 5-second unavailable TTL, serialized platform lookup, and redacted route/error formatting remain intact.
- No xAI, Kimi Code, Z.AI Coding Plan, Custom, telemetry, or updater client was migrated to the Codex route pool. Provider credentials, endpoints, catalogs, usage, hosted tools, retries, and logout state remain explicit and isolated.
- The route-pool behavior was independently implemented against the audited Apache-2.0 Codex contract; the existing root and crate-local Codex notices already cover `xai-grok-provider-http` and every adapted Codex integration path. The Kimi wire filter is an independently implemented compatibility behavior over existing generic projections; no Kimi source was copied. No additional license text is required.

## Validation gate

Candidate validation completed before committing reviewed provenance:

- `cargo fmt --all` and `git diff --check`;
- 20 provider HTTP, 310 sampling-types, and 298 sampler library tests;
- 7 Codex OAuth, 21 usage, and 25 catalog tests;
- 11 Codex hosted-search and 21 image-generation tests;
- `CARGO_INCREMENTAL=0 cargo check -p xai-grok-pager-bin`;
- 70 fork-script and 20 release/install-script unit tests; and
- the static fork contracts covering branding, provider isolation, Codex search, Warp locks, updater routes, generated workspace discipline, workflow pins, and tracked-secret hygiene.

Strict manifest ownership is intentionally run only after this ledger and the reviewed pins are committed, so the checker authenticates the complete candidate tree rather than an uncommitted declaration. Credential-gated live calls are not required for this source classification and must not expose authenticated payloads.

## Outcome reconciliation

Across 42 advanced source commits:

- **Adopt:** 6 commits whose primary behavior is exact-URL Codex routing or Kimi vacuous-assistant filtering
- **Already equivalent:** 11 commits whose primary observable behavior was already satisfied
- **Not applicable:** 25 commits confined to replacement applications, reference-only architecture, tests, or upstream release routes
- **Temporarily deferred:** 0
- **Unclassified:** 0

A commit can cite more than one evidence ID when it contains both an applicable behavior and replacement-architecture changes. The behavior table, rather than raw commit count, is authoritative for those split cases.

## Complete commit disposition inventory

### OpenAI Codex: c0cd337..b9800de

| # | Commit | Subject | Evidence |
| ---: | --- | --- | --- |
| 1 | `99efeef6506cd7f6512404d0ad8755a87ff5a011` | Add buffered code-mode exec yields (#34441) | `CDX-B980-APP-NA` |
| 2 | `9078e3237165a5378cfbf7637223222774a22a30` | Add a route-aware HTTP client pool (#34447) | `CDX-B980-EXACT-ROUTE` |
| 3 | `3bc49e1721ef0453d695950dc67bd2f0616c4883` | Make external session detection limits configurable (#34449) | `CDX-B980-APP-NA` |
| 4 | `a30aee8d906c2ec4dfa07c794504d4ee7099f98a` | Attribute external agent imports by provider (#34451) | `CDX-B980-APP-NA` |
| 5 | `9970cd706fc4f25bbb97b42f4b68d993dabe91e2` | Support alpha hotfix release versions (#34463) | `CDX-B980-RELEASE-NA` |
| 6 | `1836ae0612052137d0cabaff7807ff8314cee940` | Preserve thread settings for goal-first and forked threads (#34469) | `CDX-B980-APP-NA` |
| 7 | `9e5625d9cf636f4ef35aac7ca321a8be96f561ec` | Separate HTTP execution from request logging (#34476) | `CDX-B980-HTTP-EQ` |
| 8 | `c04452a240a13dbd0ede548fd1842d433d18419e` | Honor `CARGO_HTTP_CAINFO` in managed proxy environments (#34478) | `CDX-B980-CAINFO-NA` |
| 9 | `841e47b8fb113a201b68e0f1f5790ba22836a241` | Re-resolve system proxy routes across redirects (#34479) | `CDX-B980-EXACT-ROUTE`, `CDX-B980-REDIRECT-EQ` |
| 10 | `b8c2d29cc23b41fa7c7f5f5483e92fc71099635a` | Add route-aware redirect test coverage (#34481) | `CDX-B980-EXACT-ROUTE`, `CDX-B980-REDIRECT-EQ` |
| 11 | `d5998e74522245ce6e7b746dab6c5f0ed428a8d1` | Expand route-aware proxy redirect coverage (#34483) | `CDX-B980-EXACT-ROUTE`, `CDX-B980-REDIRECT-EQ` |
| 12 | `dc21b46aea67947c3ab1b574cabcd79ff7084c73` | Route backend requests through the HTTP client factory (#34490) | `CDX-B980-EXACT-ROUTE` |
| 13 | `0e15c31d91bb4b1dc714957e4697bba40318eb5d` | Route cloud environment discovery through the HTTP client pool (#34491) | `CDX-B980-APP-NA` |
| 14 | `99eb575649888646df3bb13f01bb78f115f894ab` | Honor system proxy settings in the daemon updater (#34495) | `CDX-B980-RELEASE-NA` |
| 15 | `4f1992732c832fe125608980a03ec2b66710c4e4` | Preserve custom arg0 for sandboxed exec-server processes (#34497) | `CDX-B980-APP-NA` |
| 16 | `cc875d61ce4be88a05371e185fb2f5530220315c` | Mirror Rust release artifacts to Cloudflare R2 (#34505) | `CDX-B980-RELEASE-NA` |
| 17 | `94bb6a09a66cbf6a70b854cfc9276eefc2c621ed` | Respect system proxies during plugin startup sync (#34506) | `CDX-B980-APP-NA` |
| 18 | `a148e0b50a5cfa642512dca77c367bde5194f882` | Publish release metadata to R2 channels (#34508) | `CDX-B980-RELEASE-NA` |
| 19 | `d937bfac84786b453ddb2d3fdb2712c1eca830ea` | Honor system proxy settings for remote plugins (#34509) | `CDX-B980-APP-NA` |
| 20 | `765675a122c9d1d4add7ce53d48c2d416fae39e9` | Add an optional releases.openai.com installer source (#34514) | `CDX-B980-RELEASE-NA` |
| 21 | `7982aa27ff7b5438121af6ec0e229b6b67ae810f` | Allow `numer` in codespell checks (#34516) | `CDX-B980-APP-NA` |
| 22 | `b9800de4867e500a92add3cde795cf4790306d0f` | Pass empty inherited FDs in the Wine PTY test (#34517) | `CDX-B980-APP-NA` |

### OpenCode: 849c259..cb562b2

| # | Commit | Subject | Evidence |
| ---: | --- | --- | --- |
| 1 | `2d2339d37d86e45df5e8f53dee881bcaa2bff553` | feat(app): draggable project rows (#38055) | `OC-CB56-APP-NA` |
| 2 | `cb562b2c6289c2eee707078f9ab644cbe1d3d8a9` | chore: generate | `OC-CB56-APP-NA` |

### Kimi Code: c2d7beb..a3699dd

| # | Commit | Subject | Evidence |
| ---: | --- | --- | --- |
| 1 | `71bcfba54a6836f4b6d4e26babde67576b293a64` | fix(agent-core): drop vacuous assistant messages that permanently wedge sessions (#1968) | `KIMI-A369-VACUOUS` |
| 2 | `115b0968cefede7fac1494c6f0154ea5545a89da` | fix(tui): hide system-trigger messages in resume replay (#1990) | `KIMI-A369-HIDDEN-EQ` |
| 3 | `73eb5f89e06fb15d42c7585a147eb1c5caef0725` | fix(tui): remove red coloring from code syntax highlighting (#1995) | `KIMI-A369-APP-NA` |
| 4 | `6dd4fd33688b37904d5302436fc2daaf09d66c7d` | refactor(agent-core-v2): rebuild the model wire layer on the kosong architecture (#1970) | `KIMI-A369-CANCEL-CACHE-EQ`, `KIMI-A369-APP-NA` |
| 5 | `beeb964393c8f9a38c2b1e2273e4415fc434b16d` | fix(vscode): reduce webview streaming re-render churn (#1994) | `KIMI-A369-APP-NA` |
| 6 | `a8f1ca3f1016a3e84986f297367e833bc731ac39` | feat(acp): support selecting thinking effort levels (#1992) | `KIMI-A369-EFFORT-EQ` |
| 7 | `e070a580f081e6b0fcc2374a510b6d68ee1746e6` | feat(telemetry): emit turn_id and agent_id on turn and tool events (#1675) | `KIMI-A369-APP-NA` |
| 8 | `ce0e3ceb04223bdaad8e8931bad46eff561055b6` | feat: support custom agent files (#1735) | `KIMI-A369-APP-NA` |
| 9 | `74da87a457c2964694a844dd22a4925f5113b167` | feat(kap-server): broadcast agent.created/agent.disposed session events (#1997) | `KIMI-A369-APP-NA` |
| 10 | `e45832398d0d9cad98dbad1cbf1e5b103a20aace` | fix(tui): keep long sessions responsive and bound resumed history (#1976) | `KIMI-A369-LONG-SESSION-EQ` |
| 11 | `92576e4d850ada51a24e72fe76a83cc512df922a` | fix(agent-core-v2): reconnect dropped MCP servers on tool call failure (#1991) | `KIMI-A369-MCP-EQ` |
| 12 | `d67a2003abf2d8d802dcf24f806e0a811724b83e` | feat(kap-server): add GET /api/v1/fs:content endpoint for host files (#2012) | `KIMI-A369-APP-NA` |
| 13 | `a3699dd6aa7b41efd3129a117007d195282379fd` | feat(kap-server): expose per-tool active state in GET /api/v1/tools (#2005) | `KIMI-A369-APP-NA` |

### CodexBar: f8636cb..cc8da27

| # | Commit | Subject | Evidence |
| ---: | --- | --- | --- |
| 1 | `4d9fbd50c77d235b62a8da8927f45bf3b789efac` | refactor: centralize usage percent clamping | `CB-CC8D-ZAI-EQ` |
| 2 | `2af39f514e8f3d294da15f9e913e53ebc1e4afe0` | refactor: data-drive provider pace capability | `CB-CC8D-ZAI-EQ` |
| 3 | `65a2f86d369106fa0d136399144e0bf5b2e1e8f4` | test: stabilize environment-sensitive coverage | `CB-CC8D-ZAI-EQ` |
| 4 | `4a95a0f8c5a8e71e5594812d99bd35ce4464d52b` | Merge pull request #2363 from steipete/codex/refactor-percent-clamp | `CB-CC8D-ZAI-EQ` |
| 5 | `cc8da27cec92029a6435bfee4a703a719290234e` | Merge pull request #2364 from steipete/codex/refactor-pace-capability | `CB-CC8D-ZAI-EQ` |

## Complete changed-path inventory

These are the exact `git diff --name-status <previous-reviewed> <audited>` records. Rename/copy rows retain Git's source and destination columns. All 675 records are present exactly once.

<details><summary>OpenAI Codex: 131 changed paths</summary>

```text
M	.codespellignore
A	.github/scripts/publish_r2_release.py
M	.github/workflows/python-runtime-build.yml
M	.github/workflows/python-runtime-release.yml
A	.github/workflows/r2-release.yml
M	.github/workflows/rust-release.yml
M	bazel/rules/testing/wine/src/lib_tests.rs
M	codex-rs/Cargo.lock
M	codex-rs/analytics/src/analytics_client_tests.rs
M	codex-rs/analytics/src/events.rs
M	codex-rs/analytics/src/facts.rs
M	codex-rs/analytics/src/reducer.rs
M	codex-rs/app-server-daemon/Cargo.toml
M	codex-rs/app-server-daemon/src/lib.rs
M	codex-rs/app-server-daemon/src/update_loop.rs
M	codex-rs/app-server-daemon/src/update_loop_tests.rs
M	codex-rs/app-server-protocol/schema/json/ClientRequest.json
M	codex-rs/app-server-protocol/schema/json/codex_app_server_protocol.schemas.json
M	codex-rs/app-server-protocol/schema/json/codex_app_server_protocol.v2.schemas.json
M	codex-rs/app-server-protocol/schema/json/v2/ExternalAgentConfigDetectParams.json
M	codex-rs/app-server-protocol/schema/json/v2/ExternalAgentConfigImportParams.json
M	codex-rs/app-server-protocol/schema/typescript/v2/ExternalAgentConfigDetectParams.ts
M	codex-rs/app-server-protocol/schema/typescript/v2/ExternalAgentConfigImportParams.ts
M	codex-rs/app-server-protocol/src/protocol/v2/config.rs
M	codex-rs/app-server-protocol/src/protocol/v2/tests.rs
M	codex-rs/app-server/README.md
M	codex-rs/app-server/src/bin/exec_server.rs
M	codex-rs/app-server/src/config_manager.rs
M	codex-rs/app-server/src/external_agent_migration/processor.rs
M	codex-rs/app-server/src/lib.rs
M	codex-rs/app-server/src/request_processors/account_processor.rs
M	codex-rs/app-server/src/request_processors/account_processor/rate_limit_resets.rs
M	codex-rs/app-server/src/request_processors/plugins.rs
M	codex-rs/app-server/src/request_processors/thread_goal_processor.rs
M	codex-rs/app-server/tests/suite/v2/external_agent_config.rs
M	codex-rs/app-server/tests/suite/v2/thread_resume.rs
M	codex-rs/arg0/src/lib.rs
M	codex-rs/backend-client/Cargo.toml
M	codex-rs/backend-client/src/client.rs
M	codex-rs/backend-client/src/client/rate_limit_resets.rs
M	codex-rs/backend-client/src/client/rate_limit_resets_tests.rs
A	codex-rs/backend-client/src/client_request_tests.rs
M	codex-rs/cli/Cargo.toml
M	codex-rs/cli/src/main.rs
M	codex-rs/cloud-config/Cargo.toml
M	codex-rs/cloud-config/src/backend.rs
M	codex-rs/cloud-config/src/bundle_loader.rs
M	codex-rs/cloud-tasks-client/Cargo.toml
M	codex-rs/cloud-tasks-client/src/http.rs
M	codex-rs/cloud-tasks/Cargo.toml
M	codex-rs/cloud-tasks/src/env_detect.rs
A	codex-rs/cloud-tasks/src/env_detect_tests.rs
M	codex-rs/cloud-tasks/src/lib.rs
M	codex-rs/cloud-tasks/src/util.rs
M	codex-rs/code-mode-protocol/src/description.rs
M	codex-rs/core-plugins/Cargo.toml
M	codex-rs/core-plugins/src/discoverable_tests.rs
A	codex-rs/core-plugins/src/http_client_selector.rs
M	codex-rs/core-plugins/src/lib.rs
M	codex-rs/core-plugins/src/manager.rs
M	codex-rs/core-plugins/src/manager_tests.rs
M	codex-rs/core-plugins/src/remote.rs
M	codex-rs/core-plugins/src/remote/remote_installed_plugin_sync.rs
M	codex-rs/core-plugins/src/remote/share.rs
M	codex-rs/core-plugins/src/remote/share/checkout.rs
M	codex-rs/core-plugins/src/remote/share/tests.rs
M	codex-rs/core-plugins/src/remote_bundle.rs
M	codex-rs/core-plugins/src/remote_legacy.rs
M	codex-rs/core-plugins/src/remote_tests.rs
M	codex-rs/core-plugins/src/startup_sync.rs
A	codex-rs/core-plugins/src/startup_sync/http_client.rs
M	codex-rs/core-plugins/src/startup_sync_tests.rs
M	codex-rs/core-plugins/src/test_support.rs
M	codex-rs/core/config.schema.json
M	codex-rs/core/src/codex_thread.rs
M	codex-rs/core/src/config/config_tests.rs
M	codex-rs/core/src/config/mod.rs
M	codex-rs/core/src/plugins/discoverable_tests.rs
M	codex-rs/core/src/session/handlers.rs
M	codex-rs/core/src/session/mod.rs
M	codex-rs/core/src/session/session.rs
M	codex-rs/core/src/session/tests.rs
M	codex-rs/core/src/tools/code_mode/execute_spec.rs
M	codex-rs/core/src/tools/code_mode/mod.rs
M	codex-rs/core/src/tools/spec_plan.rs
M	codex-rs/core/src/tools/spec_plan_tests.rs
M	codex-rs/core/tests/suite/fork_thread.rs
M	codex-rs/core/tests/suite/mod.rs
M	codex-rs/deny.toml
A	codex-rs/exec-server/src/arg0_exec_helper.rs
M	codex-rs/exec-server/src/lib.rs
M	codex-rs/exec-server/src/process_sandbox.rs
M	codex-rs/exec-server/src/process_sandbox_tests.rs
M	codex-rs/exec-server/testing/exec_server.rs
M	codex-rs/exec-server/tests/common/mod.rs
M	codex-rs/exec-server/tests/exec_process.rs
M	codex-rs/external-agent-migration/src/detect/mod.rs
M	codex-rs/external-agent-migration/src/detect/sessions/cla.rs
M	codex-rs/external-agent-migration/src/detect/sessions/common.rs
M	codex-rs/external-agent-migration/src/detect/sessions/cur.rs
M	codex-rs/external-agent-migration/src/detect/sessions/cur_tests.rs
M	codex-rs/external-agent-migration/src/detect/sessions/mod.rs
M	codex-rs/external-agent-migration/src/lib.rs
M	codex-rs/external-agent-migration/src/migration_source.rs
M	codex-rs/external-agent-migration/src/model.rs
M	codex-rs/external-agent-migration/src/service.rs
M	codex-rs/features/src/lib.rs
M	codex-rs/http-client/src/default_client.rs
M	codex-rs/http-client/src/lib.rs
M	codex-rs/http-client/src/outbound_proxy.rs
A	codex-rs/http-client/src/outbound_proxy_redirect_coverage_tests.rs
M	codex-rs/http-client/src/outbound_proxy_tests.rs
A	codex-rs/http-client/src/route_aware_client_pool.rs
A	codex-rs/http-client/src/route_aware_client_pool_tests.rs
A	codex-rs/http-client/src/route_aware_redirect.rs
A	codex-rs/http-client/src/route_aware_redirect_integration_tests.rs
A	codex-rs/http-client/src/route_aware_redirect_tests.rs
M	codex-rs/login/src/auth/default_client.rs
M	codex-rs/login/src/outbound_proxy.rs
M	codex-rs/memories/write/src/guard.rs
M	codex-rs/network-proxy/src/certs.rs
M	codex-rs/protocol/src/protocol.rs
M	codex-rs/tui/src/app_server_session.rs
M	codex-rs/tui/src/external_agent_config_migration_flow.rs
M	scripts/install/install.ps1
M	scripts/install/install.sh
M	scripts/install/test_install_sh.py
M	sdk/python/_runtime_setup.py
A	sdk/python/release_version.py
M	sdk/python/scripts/update_sdk_artifacts.py
M	sdk/python/tests/test_artifact_workflow_and_binaries.py
```

</details>

<details><summary>OpenCode: 1 changed paths</summary>

```text
M	packages/app/src/pages/home.tsx
```

</details>

<details><summary>Kimi Code: 522 changed paths</summary>

```text
M	.agents/skills/agent-core-dev/telemetry.md
A	.changeset/acp-thinking-effort-levels.md
A	.changeset/agent-file-subagents.md
A	.changeset/agent-lifecycle-events.md
A	.changeset/custom-agent-files.md
A	.changeset/fix-abort-not-retryable.md
A	.changeset/fix-openai-prompt-cache-key.md
A	.changeset/fix-vacuous-assistant-wedge.md
A	.changeset/global-tool-gating.md
A	.changeset/goal-replay-leak.md
A	.changeset/host-fs-content-endpoint.md
A	.changeset/kosong-layered-wire-architecture.md
A	.changeset/long-session-tui-performance.md
A	.changeset/mcp-tool-call-reconnect.md
A	.changeset/model-resolution-inspection.md
A	.changeset/pi-tui-frame-line-reuse.md
A	.changeset/system-md-override.md
A	.changeset/tool-pattern-warnings.md
A	.changeset/tools-list-active-flag.md
A	.changeset/tui-code-highlight-no-red.md
M	AGENTS.md
M	apps/kimi-code/src/cli/commands.ts
M	apps/kimi-code/src/cli/options.ts
M	apps/kimi-code/src/cli/v2/run-v2-print.ts
M	apps/kimi-code/src/main.ts
M	apps/kimi-code/src/tui/commands/undo.ts
M	apps/kimi-code/src/tui/components/chrome/gutter-container.ts
M	apps/kimi-code/src/tui/components/media/code-highlight.ts
M	apps/kimi-code/src/tui/components/messages/step-summary.ts
M	apps/kimi-code/src/tui/components/messages/user-message.ts
M	apps/kimi-code/src/tui/controllers/session-event-handler.ts
M	apps/kimi-code/src/tui/controllers/session-replay.ts
M	apps/kimi-code/src/tui/controllers/streaming-ui.ts
M	apps/kimi-code/src/tui/controllers/subagent-event-handler.ts
M	apps/kimi-code/src/tui/kimi-tui.ts
A	apps/kimi-code/src/tui/theme/highlight-theme.ts
M	apps/kimi-code/src/tui/theme/pi-tui-theme.ts
M	apps/kimi-code/src/tui/utils/message-replay.ts
M	apps/kimi-code/src/tui/utils/transcript-window.ts
M	apps/kimi-code/test/cli/main.test.ts
M	apps/kimi-code/test/cli/options.test.ts
M	apps/kimi-code/test/cli/run-prompt.test.ts
M	apps/kimi-code/test/cli/run-shell.test.ts
M	apps/kimi-code/test/cli/v2-run-print.test.ts
M	apps/kimi-code/test/tui/activity-pane.test.ts
M	apps/kimi-code/test/tui/components/media/code-highlight.test.ts
A	apps/kimi-code/test/tui/components/messages/step-summary.test.ts
M	apps/kimi-code/test/tui/kimi-tui-message-flow.test.ts
M	apps/kimi-code/test/tui/kimi-tui-startup.test.ts
M	apps/kimi-code/test/tui/message-replay.test.ts
M	apps/kimi-code/test/tui/signal-handlers.test.ts
A	apps/kimi-code/test/tui/tui-frame.bench.ts
M	apps/kimi-inspect/src/App.tsx
A	apps/kimi-inspect/src/components/ModelCatalogView.tsx
A	apps/kimi-inspect/src/components/NavRail.tsx
M	apps/kimi-inspect/src/components/Sidebar.tsx
M	apps/kimi-inspect/src/panels.ts
M	apps/kimi-inspect/src/ui.tsx
M	apps/kimi-web/src/composables/messagesToTurns.ts
M	apps/vscode/src/handlers/file.handler.ts
M	apps/vscode/webview-ui/src/components/ChatArea.tsx
M	apps/vscode/webview-ui/src/components/ChatMessage.tsx
M	apps/vscode/webview-ui/src/components/CompactionCard.tsx
M	docs/en/configuration/config-files.md
M	docs/en/customization/agents.md
M	docs/en/reference/kimi-command.md
M	docs/zh/configuration/config-files.md
M	docs/zh/customization/agents.md
M	docs/zh/reference/kimi-command.md
M	packages/acp-adapter/src/config-options.ts
M	packages/acp-adapter/src/model-catalog.ts
M	packages/acp-adapter/src/server.ts
M	packages/acp-adapter/src/session.ts
M	packages/acp-adapter/test/_helpers/harness-stubs.ts
M	packages/acp-adapter/test/config-options.test.ts
M	packages/acp-adapter/test/model-catalog.test.ts
M	packages/acp-adapter/test/set-session-config-option.test.ts
M	packages/agent-core-v2/AGENTS.md
M	packages/agent-core-v2/package.json
M	packages/agent-core-v2/scripts/check-domain-layers.mjs
A	packages/agent-core-v2/src/_base/utils/fileMeta.ts
M	packages/agent-core-v2/src/_base/utils/render-prompt.ts
M	packages/agent-core-v2/src/agent/blob/agentBlobService.ts
M	packages/agent-core-v2/src/agent/blob/agentBlobServiceImpl.ts
M	packages/agent-core-v2/src/agent/contextInjector/contextInjector.ts
M	packages/agent-core-v2/src/agent/contextMemory/compactionHandoff.ts
M	packages/agent-core-v2/src/agent/contextMemory/contextMemoryService.ts
M	packages/agent-core-v2/src/agent/contextMemory/contextOps.ts
M	packages/agent-core-v2/src/agent/contextMemory/contextTranscript.ts
M	packages/agent-core-v2/src/agent/contextMemory/loopEventFold.ts
M	packages/agent-core-v2/src/agent/contextMemory/toolResultRender.ts
M	packages/agent-core-v2/src/agent/contextMemory/types.ts
A	packages/agent-core-v2/src/agent/contextMemory/vacuousContent.ts
M	packages/agent-core-v2/src/agent/contextProjector/contextProjector.ts
M	packages/agent-core-v2/src/agent/contextProjector/contextProjectorService.ts
M	packages/agent-core-v2/src/agent/contextSize/contextSize.ts
M	packages/agent-core-v2/src/agent/contextSize/contextSizeService.ts
M	packages/agent-core-v2/src/agent/externalHooks/types.ts
M	packages/agent-core-v2/src/agent/fullCompaction/compaction-instruction.md
M	packages/agent-core-v2/src/agent/fullCompaction/fullCompactionService.ts
M	packages/agent-core-v2/src/agent/fullCompaction/strategy.ts
M	packages/agent-core-v2/src/agent/goal/injection/goal-active-reminder.md
M	packages/agent-core-v2/src/agent/goal/injection/goal-blocked-reminder.md
M	packages/agent-core-v2/src/agent/goal/injection/goal-paused-reminder.md
M	packages/agent-core-v2/src/agent/goal/injection/goalInjection.ts
M	packages/agent-core-v2/src/agent/llmRequester/llmRequestOps.ts
M	packages/agent-core-v2/src/agent/llmRequester/llmRequester.ts
M	packages/agent-core-v2/src/agent/llmRequester/llmRequesterService.ts
M	packages/agent-core-v2/src/agent/loop/loop.ts
M	packages/agent-core-v2/src/agent/loop/loopService.ts
M	packages/agent-core-v2/src/agent/loop/stepRequest.ts
M	packages/agent-core-v2/src/agent/loop/turnEvents.ts
M	packages/agent-core-v2/src/agent/loop/turnOps.ts
M	packages/agent-core-v2/src/agent/mcp/client-http.ts
M	packages/agent-core-v2/src/agent/mcp/client-shared.ts
M	packages/agent-core-v2/src/agent/mcp/client-sse.ts
M	packages/agent-core-v2/src/agent/mcp/client-stdio.ts
M	packages/agent-core-v2/src/agent/mcp/connection-manager.ts
M	packages/agent-core-v2/src/agent/mcp/mcp.ts
M	packages/agent-core-v2/src/agent/mcp/mcpService.ts
M	packages/agent-core-v2/src/agent/mcp/output.ts
M	packages/agent-core-v2/src/agent/mcp/tools/mcp.ts
M	packages/agent-core-v2/src/agent/mcp/types.ts
M	packages/agent-core-v2/src/agent/media/image-compress.ts
M	packages/agent-core-v2/src/agent/media/mediaToolsRegistrar.ts
M	packages/agent-core-v2/src/agent/media/registerMediaTools.ts
M	packages/agent-core-v2/src/agent/media/tools/read-media.md
M	packages/agent-core-v2/src/agent/media/tools/read-media.ts
M	packages/agent-core-v2/src/agent/permissionGate/permissionGateService.ts
M	packages/agent-core-v2/src/agent/plan/profile/plan.ts
M	packages/agent-core-v2/src/agent/plan/tools/enter-plan-mode.ts
M	packages/agent-core-v2/src/agent/plan/tools/exit-plan-mode.ts
D	packages/agent-core-v2/src/agent/profile/configSection.ts
M	packages/agent-core-v2/src/agent/profile/errors.ts
M	packages/agent-core-v2/src/agent/profile/profile.ts
M	packages/agent-core-v2/src/agent/profile/profileOps.ts
M	packages/agent-core-v2/src/agent/profile/profileService.ts
D	packages/agent-core-v2/src/agent/profile/thinking.ts
M	packages/agent-core-v2/src/agent/prompt/promptService.ts
M	packages/agent-core-v2/src/agent/rpc/core-api.ts
M	packages/agent-core-v2/src/agent/rpc/prompt-metadata.ts
M	packages/agent-core-v2/src/agent/rpc/rpcService.ts
M	packages/agent-core-v2/src/agent/skill/skillService.ts
M	packages/agent-core-v2/src/agent/stepRetry/stepRetryService.ts
M	packages/agent-core-v2/src/agent/swarm/tools/agent-swarm.ts
M	packages/agent-core-v2/src/agent/task/taskService.ts
M	packages/agent-core-v2/src/agent/toolDedupe/toolDedupe.ts
M	packages/agent-core-v2/src/agent/toolDedupe/toolDedupeService.ts
M	packages/agent-core-v2/src/agent/toolExecutor/toolExecutor.ts
M	packages/agent-core-v2/src/agent/toolExecutor/toolExecutorService.ts
M	packages/agent-core-v2/src/agent/toolExecutor/toolHooks.ts
A	packages/agent-core-v2/src/agent/toolPolicy/configSection.ts
A	packages/agent-core-v2/src/agent/toolPolicy/evaluate.ts
A	packages/agent-core-v2/src/agent/toolPolicy/toolPolicy.ts
A	packages/agent-core-v2/src/agent/toolPolicy/toolPolicyService.ts
M	packages/agent-core-v2/src/agent/toolRegistry/toolRegistry.ts
M	packages/agent-core-v2/src/agent/toolRegistry/toolRegistryService.ts
M	packages/agent-core-v2/src/agent/toolResultTruncation/toolResultTruncationService.ts
M	packages/agent-core-v2/src/agent/toolSelect/toolSelectService.ts
M	packages/agent-core-v2/src/agent/usage/usage.ts
M	packages/agent-core-v2/src/agent/usage/usageOps.ts
M	packages/agent-core-v2/src/agent/usage/usageService.ts
A	packages/agent-core-v2/src/app/agentFileCatalog/agentCatalogRuntimeOptions.ts
A	packages/agent-core-v2/src/app/agentFileCatalog/agentFile.ts
A	packages/agent-core-v2/src/app/agentFileCatalog/agentFileDiscovery.ts
A	packages/agent-core-v2/src/app/agentFileCatalog/agentProfileFromFile.ts
A	packages/agent-core-v2/src/app/agentFileCatalog/agentProfileSource.ts
A	packages/agent-core-v2/src/app/agentFileCatalog/agentRoots.ts
A	packages/agent-core-v2/src/app/agentFileCatalog/configSection.ts
A	packages/agent-core-v2/src/app/agentFileCatalog/paths.ts
A	packages/agent-core-v2/src/app/agentFileCatalog/systemFile.ts
A	packages/agent-core-v2/src/app/agentFileCatalog/types.ts
A	packages/agent-core-v2/src/app/agentFileCatalog/userFileAgentSource.ts
M	packages/agent-core-v2/src/app/agentProfileCatalog/agentProfileCatalog.ts
M	packages/agent-core-v2/src/app/agentProfileCatalog/profile-shared.ts
M	packages/agent-core-v2/src/app/agentProfileCatalog/system.md
M	packages/agent-core-v2/src/app/auth/auth.ts
M	packages/agent-core-v2/src/app/auth/authService.ts
M	packages/agent-core-v2/src/app/auth/configSection.ts
M	packages/agent-core-v2/src/app/auth/webSearch/webSearchService.ts
M	packages/agent-core-v2/src/app/authLegacy/authLegacyService.ts
A	packages/agent-core-v2/src/app/config/sectionDiff.ts
D	packages/agent-core-v2/src/app/llmProtocol/catalog.ts
D	packages/agent-core-v2/src/app/llmProtocol/finishReason.ts
D	packages/agent-core-v2/src/app/llmProtocol/kimiOptions.ts
D	packages/agent-core-v2/src/app/llmProtocol/provider.ts
D	packages/agent-core-v2/src/app/llmProtocol/providers/capability-registry.ts
D	packages/agent-core-v2/src/app/llmProtocol/providers/kimi.ts
D	packages/agent-core-v2/src/app/llmProtocol/providers/providers.ts
D	packages/agent-core-v2/src/app/llmProtocol/request.ts
D	packages/agent-core-v2/src/app/llmProtocol/thinkingEffort.ts
D	packages/agent-core-v2/src/app/llmProtocol/tool.ts
D	packages/agent-core-v2/src/app/model/completionBudget.ts
D	packages/agent-core-v2/src/app/model/model.ts
D	packages/agent-core-v2/src/app/model/modelAuth.ts
D	packages/agent-core-v2/src/app/model/modelImpl.ts
D	packages/agent-core-v2/src/app/model/modelInstance.ts
D	packages/agent-core-v2/src/app/model/modelOverrides.ts
D	packages/agent-core-v2/src/app/model/modelResolver.ts
D	packages/agent-core-v2/src/app/model/modelResolverService.ts
D	packages/agent-core-v2/src/app/model/modelService.ts
D	packages/agent-core-v2/src/app/model/thinking.ts
D	packages/agent-core-v2/src/app/modelCatalog/modelCatalog.ts
D	packages/agent-core-v2/src/app/modelCatalog/modelCatalogService.ts
D	packages/agent-core-v2/src/app/platform/configSection.ts
D	packages/agent-core-v2/src/app/platform/platform.ts
D	packages/agent-core-v2/src/app/platform/platformService.ts
M	packages/agent-core-v2/src/app/plugin/pluginService.ts
D	packages/agent-core-v2/src/app/protocol/protocol.ts
D	packages/agent-core-v2/src/app/protocol/protocolAdapterRegistry.ts
M	packages/agent-core-v2/src/app/sessionLegacy/sessionLegacyService.ts
M	packages/agent-core-v2/src/app/sessionLifecycle/sessionLifecycle.ts
M	packages/agent-core-v2/src/app/sessionLifecycle/sessionLifecycleService.ts
M	packages/agent-core-v2/src/app/telemetry/agentTelemetryContext.ts
M	packages/agent-core-v2/src/app/telemetry/agentTelemetryContextService.ts
M	packages/agent-core-v2/src/app/telemetry/events.ts
M	packages/agent-core-v2/src/app/telemetry/telemetry.ts
M	packages/agent-core-v2/src/app/telemetry/telemetryService.ts
M	packages/agent-core-v2/src/app/web/webService.ts
M	packages/agent-core-v2/src/errors.ts
M	packages/agent-core-v2/src/index.ts
R072	packages/agent-core-v2/src/app/llmProtocol/capability.ts	packages/agent-core-v2/src/kosong/contract/capability.ts
R075	packages/agent-core-v2/src/app/llmProtocol/errors.ts	packages/agent-core-v2/src/kosong/contract/errors.ts
R092	packages/agent-core-v2/src/app/llmProtocol/generate.ts	packages/agent-core-v2/src/kosong/contract/generate.ts
A	packages/agent-core-v2/src/kosong/contract/inspection.ts
R088	packages/agent-core-v2/src/app/llmProtocol/message.ts	packages/agent-core-v2/src/kosong/contract/message.ts
R081	packages/agent-core-v2/src/app/llmProtocol/messageHelpers.ts	packages/agent-core-v2/src/kosong/contract/messageHelpers.ts
A	packages/agent-core-v2/src/kosong/contract/provider.ts
R077	packages/agent-core-v2/src/app/llmProtocol/requestTrace.ts	packages/agent-core-v2/src/kosong/contract/requestTrace.ts
R081	packages/agent-core-v2/src/_base/utils/tokens.ts	packages/agent-core-v2/src/kosong/contract/tokens.ts
A	packages/agent-core-v2/src/kosong/contract/tool.ts
R074	packages/agent-core-v2/src/app/llmProtocol/usage.ts	packages/agent-core-v2/src/kosong/contract/usage.ts
A	packages/agent-core-v2/src/kosong/model/catalog.ts
A	packages/agent-core-v2/src/kosong/model/catalogService.ts
A	packages/agent-core-v2/src/kosong/model/completionBudget.ts
R069	packages/agent-core-v2/src/app/model/configSection.ts	packages/agent-core-v2/src/kosong/model/configSection.ts
A	packages/agent-core-v2/src/kosong/model/discovery.ts
R051	packages/agent-core-v2/src/app/modelCatalog/configSection.ts	packages/agent-core-v2/src/kosong/model/discoveryConfigSection.ts
A	packages/agent-core-v2/src/kosong/model/discoveryService.ts
R083	packages/agent-core-v2/src/app/model/envOverlay.ts	packages/agent-core-v2/src/kosong/model/envOverlay.ts
R056	packages/agent-core-v2/src/app/modelCatalog/errors.ts	packages/agent-core-v2/src/kosong/model/errors.ts
R069	packages/agent-core-v2/src/app/model/hostRequestHeaders.ts	packages/agent-core-v2/src/kosong/model/hostRequestHeaders.ts
A	packages/agent-core-v2/src/kosong/model/inspection.ts
A	packages/agent-core-v2/src/kosong/model/model.ts
A	packages/agent-core-v2/src/kosong/model/model.types.ts
A	packages/agent-core-v2/src/kosong/model/modelAuth.ts
A	packages/agent-core-v2/src/kosong/model/modelRequester.ts
A	packages/agent-core-v2/src/kosong/model/modelRequesterImpl.ts
A	packages/agent-core-v2/src/kosong/model/modelService.ts
A	packages/agent-core-v2/src/kosong/model/thinking.ts
R079	packages/agent-core-v2/src/app/protocol/errors.ts	packages/agent-core-v2/src/kosong/protocol/errors.ts
A	packages/agent-core-v2/src/kosong/protocol/protocol.ts
A	packages/agent-core-v2/src/kosong/protocol/protocolBase.ts
A	packages/agent-core-v2/src/kosong/protocol/protocolTrait.ts
R091	packages/agent-core-v2/src/app/llmProtocol/providers/anthropic-profile.ts	packages/agent-core-v2/src/kosong/provider/bases/anthropic/anthropic-profile.ts
A	packages/agent-core-v2/src/kosong/provider/bases/anthropic/anthropic.contrib.ts
R075	packages/agent-core-v2/src/app/llmProtocol/providers/anthropic.ts	packages/agent-core-v2/src/kosong/provider/bases/anthropic/anthropic.ts
A	packages/agent-core-v2/src/kosong/provider/bases/anthropic/anthropicHooks.ts
A	packages/agent-core-v2/src/kosong/provider/bases/anthropic/index.ts
A	packages/agent-core-v2/src/kosong/provider/bases/google-genai/google-genai.contrib.ts
R084	packages/agent-core-v2/src/app/llmProtocol/providers/google-genai.ts	packages/agent-core-v2/src/kosong/provider/bases/google-genai/google-genai.ts
A	packages/agent-core-v2/src/kosong/provider/bases/google-genai/index.ts
R071	packages/agent-core-v2/src/app/llmProtocol/providers/merge-user-messages.ts	packages/agent-core-v2/src/kosong/provider/bases/merge-user-messages.ts
R085	packages/agent-core-v2/src/app/llmProtocol/providers/chat-completions-stream.ts	packages/agent-core-v2/src/kosong/provider/bases/openai/chat-completions-stream.ts
A	packages/agent-core-v2/src/kosong/provider/bases/openai/index.ts
R069	packages/agent-core-v2/src/app/llmProtocol/providers/openai-common.ts	packages/agent-core-v2/src/kosong/provider/bases/openai/openai-common.ts
A	packages/agent-core-v2/src/kosong/provider/bases/openai/openai-legacy.contrib.ts
R055	packages/agent-core-v2/src/app/llmProtocol/providers/openai-legacy.ts	packages/agent-core-v2/src/kosong/provider/bases/openai/openai-legacy.ts
A	packages/agent-core-v2/src/kosong/provider/bases/openai/openai-responses.contrib.ts
R087	packages/agent-core-v2/src/app/llmProtocol/providers/openai-responses.ts	packages/agent-core-v2/src/kosong/provider/bases/openai/openai-responses.ts
A	packages/agent-core-v2/src/kosong/provider/bases/openai/openaiHooks.ts
R073	packages/agent-core-v2/src/app/llmProtocol/providers/request-auth.ts	packages/agent-core-v2/src/kosong/provider/bases/request-auth.ts
R083	packages/agent-core-v2/src/app/llmProtocol/providers/tool-call-id.ts	packages/agent-core-v2/src/kosong/provider/bases/tool-call-id.ts
R072	packages/agent-core-v2/src/app/provider/configSection.ts	packages/agent-core-v2/src/kosong/provider/configSection.ts
A	packages/agent-core-v2/src/kosong/provider/protocolAdapterRegistry.ts
R075	packages/agent-core-v2/src/app/provider/provider.ts	packages/agent-core-v2/src/kosong/provider/provider.ts
A	packages/agent-core-v2/src/kosong/provider/providerDefinition.ts
R056	packages/agent-core-v2/src/app/provider/providerService.ts	packages/agent-core-v2/src/kosong/provider/providerService.ts
R087	packages/agent-core-v2/src/app/llmProtocol/providers/kimi-files.ts	packages/agent-core-v2/src/kosong/provider/providers/kimi/kimi-files.ts
R097	packages/agent-core-v2/src/app/llmProtocol/providers/kimi-schema.ts	packages/agent-core-v2/src/kosong/provider/providers/kimi/kimi-schema.ts
A	packages/agent-core-v2/src/kosong/provider/providers/kimi/kimi.contrib.ts
A	packages/agent-core-v2/src/kosong/provider/providers/standard.contrib.ts
M	packages/agent-core-v2/src/os/backends/node-local/tools/bash.md
M	packages/agent-core-v2/src/os/backends/node-local/tools/bash.ts
M	packages/agent-core-v2/src/os/backends/node-local/tools/read.md
M	packages/agent-core-v2/src/session/agentLifecycle/agentLifecycleService.ts
M	packages/agent-core-v2/src/session/agentLifecycle/profile/profiles.ts
M	packages/agent-core-v2/src/session/cron/sessionCronService.ts
M	packages/agent-core-v2/src/session/cron/sessionCronServiceImpl.ts
M	packages/agent-core-v2/src/session/cron/tools/cron-create.ts
M	packages/agent-core-v2/src/session/cron/tools/cron-delete.ts
A	packages/agent-core-v2/src/session/sessionAgentProfileCatalog/explicitFileAgentSource.ts
A	packages/agent-core-v2/src/session/sessionAgentProfileCatalog/extraFileAgentSource.ts
A	packages/agent-core-v2/src/session/sessionAgentProfileCatalog/projectFileAgentSource.ts
A	packages/agent-core-v2/src/session/sessionAgentProfileCatalog/sessionAgentProfileCatalog.ts
A	packages/agent-core-v2/src/session/sessionAgentProfileCatalog/sessionAgentProfileCatalogService.ts
M	packages/agent-core-v2/src/session/sessionFs/fsService.ts
A	packages/agent-core-v2/src/session/sessionToolPolicy/sessionToolPolicy.ts
A	packages/agent-core-v2/src/session/sessionToolPolicy/sessionToolPolicyService.ts
M	packages/agent-core-v2/src/session/subagent/mirrorAgentRun.ts
M	packages/agent-core-v2/src/session/subagent/runAgentTurn.ts
M	packages/agent-core-v2/src/session/subagent/subagent.ts
M	packages/agent-core-v2/src/session/subagent/subagentService.ts
M	packages/agent-core-v2/src/session/subagent/tools/agent.ts
M	packages/agent-core-v2/src/session/subagent/tools/subagent-task.ts
M	packages/agent-core-v2/src/session/swarm/agentRunBatch.ts
M	packages/agent-core-v2/src/session/swarm/sessionSwarm.ts
M	packages/agent-core-v2/src/session/swarm/sessionSwarmService.ts
M	packages/agent-core-v2/src/session/todo/sessionTodoService.ts
M	packages/agent-core-v2/src/tool/toolContract.ts
M	packages/agent-core-v2/src/wire/model.ts
M	packages/agent-core-v2/src/wire/wireService.ts
M	packages/agent-core-v2/test/_base/utils/tokens.test.ts
M	packages/agent-core-v2/test/agent/blob/agentBlobService.test.ts
M	packages/agent-core-v2/test/agent/contextMemory/context.test.ts
M	packages/agent-core-v2/test/agent/contextMemory/contextTranscript.test.ts
M	packages/agent-core-v2/test/agent/contextMemory/loopEventFold.test.ts
M	packages/agent-core-v2/test/agent/contextMemory/splice-replay.test.ts
M	packages/agent-core-v2/test/agent/contextProjector/contextProjector.bench.ts
M	packages/agent-core-v2/test/agent/contextProjector/projector-tool-exchanges.test.ts
M	packages/agent-core-v2/test/agent/contextSize/contextSize.test.ts
M	packages/agent-core-v2/test/agent/fullCompaction/fullCompaction.test.ts
M	packages/agent-core-v2/test/agent/fullCompaction/strategy.test.ts
M	packages/agent-core-v2/test/agent/goal/goal.test.ts
M	packages/agent-core-v2/test/agent/goal/injection/goalInjection.test.ts
M	packages/agent-core-v2/test/agent/goal/tools/goal-tools.test.ts
M	packages/agent-core-v2/test/agent/llmRequester/llmRequester.test.ts
M	packages/agent-core-v2/test/agent/llmRequester/llmRequesterService.test.ts
M	packages/agent-core-v2/test/agent/loop/loop.test.ts
M	packages/agent-core-v2/test/agent/loop/stubs.ts
M	packages/agent-core-v2/test/agent/mcp/connection-manager.test.ts
M	packages/agent-core-v2/test/agent/mcp/mcp.test.ts
M	packages/agent-core-v2/test/agent/mcp/output.test.ts
M	packages/agent-core-v2/test/agent/mcp/stubs.ts
M	packages/agent-core-v2/test/agent/media/tools/read-media.test.ts
M	packages/agent-core-v2/test/agent/permissionGate/permissionGate.test.ts
M	packages/agent-core-v2/test/agent/permissionPolicy/permissionPolicyService.test.ts
M	packages/agent-core-v2/test/agent/permissionPolicy/policies/default-tool-approve.test.ts
M	packages/agent-core-v2/test/agent/permissionPolicy/policies/exit-plan-mode-review-ask.test.ts
M	packages/agent-core-v2/test/agent/permissionPolicy/policies/goal-start-review-ask.test.ts
M	packages/agent-core-v2/test/agent/permissionPolicy/policies/plan-mode-guard-deny.test.ts
M	packages/agent-core-v2/test/agent/plan/plan.test.ts
M	packages/agent-core-v2/test/agent/plan/tools/exit-plan-mode.test.ts
M	packages/agent-core-v2/test/agent/plan/tools/plan-tools-telemetry.test.ts
M	packages/agent-core-v2/test/agent/profile/binding.test.ts
M	packages/agent-core-v2/test/agent/profile/config-state.test.ts
M	packages/agent-core-v2/test/agent/profile/profileOps.test.ts
M	packages/agent-core-v2/test/agent/profile/thinking.test.ts
M	packages/agent-core-v2/test/agent/rpc/setPermission.test.ts
M	packages/agent-core-v2/test/agent/rpc/undoHistory.test.ts
M	packages/agent-core-v2/test/agent/shellCommand/shellCommand.test.ts
M	packages/agent-core-v2/test/agent/skill/skill.test.ts
M	packages/agent-core-v2/test/agent/stepRetry/stepRetry.test.ts
M	packages/agent-core-v2/test/agent/swarm/swarm.test.ts
M	packages/agent-core-v2/test/agent/task/idle-notification-repro.test.ts
M	packages/agent-core-v2/test/agent/task/rpc-events.test.ts
M	packages/agent-core-v2/test/agent/toolDedupe/toolDedupe.test.ts
M	packages/agent-core-v2/test/agent/toolExecutor/toolExecutor.test.ts
A	packages/agent-core-v2/test/agent/toolPolicy/evaluate.test.ts
M	packages/agent-core-v2/test/agent/toolResultTruncation/toolResultTruncation.test.ts
M	packages/agent-core-v2/test/agent/toolSelect/toolSelectService.test.ts
A	packages/agent-core-v2/test/app/agentFileCatalog/agentFile.test.ts
A	packages/agent-core-v2/test/app/agentFileCatalog/agentFileDiscovery.test.ts
A	packages/agent-core-v2/test/app/agentFileCatalog/agentRoots.test.ts
A	packages/agent-core-v2/test/app/agentFileCatalog/systemFile.test.ts
A	packages/agent-core-v2/test/app/agentProfileCatalog/profile-shared.test.ts
M	packages/agent-core-v2/test/app/auth/auth.test.ts
M	packages/agent-core-v2/test/app/config/config.test.ts
M	packages/agent-core-v2/test/app/externalHooksRunner/externalHooksRunner.test.ts
M	packages/agent-core-v2/test/app/externalHooksRunner/integration.test.ts
M	packages/agent-core-v2/test/app/llmProtocol/errors.test.ts
D	packages/agent-core-v2/test/app/llmProtocol/providers/anthropic-context-management.test.ts
D	packages/agent-core-v2/test/app/llmProtocol/providers/anthropic-max-tokens.test.ts
D	packages/agent-core-v2/test/app/llmProtocol/providers/empty-thinking-roundtrip.test.ts
D	packages/agent-core-v2/test/app/llmProtocol/providers/openai-legacy.test.ts
D	packages/agent-core-v2/test/app/llmProtocol/providers/provider-errors.test.ts
D	packages/agent-core-v2/test/app/llmProtocol/select-tools.test.ts
D	packages/agent-core-v2/test/app/llmProtocol/structured-output.test.ts
M	packages/agent-core-v2/test/app/messageLegacy/messageLegacy.test.ts
D	packages/agent-core-v2/test/app/model/completionBudget.test.ts
M	packages/agent-core-v2/test/app/model/model.test.ts
D	packages/agent-core-v2/test/app/model/modelImpl.test.ts
D	packages/agent-core-v2/test/app/model/modelResolver-runtime.test.ts
D	packages/agent-core-v2/test/app/model/modelResolver.test.ts
D	packages/agent-core-v2/test/app/modelCatalog/modelCatalog.test.ts
M	packages/agent-core-v2/test/app/plugin/pluginService.test.ts
M	packages/agent-core-v2/test/app/protocol/errors.test.ts
D	packages/agent-core-v2/test/app/protocol/protocolAdapterRegistry.test.ts
M	packages/agent-core-v2/test/app/provider/provider.test.ts
M	packages/agent-core-v2/test/app/provider/stubs.ts
M	packages/agent-core-v2/test/app/sessionLegacy/sessionLegacy.test.ts
M	packages/agent-core-v2/test/app/sessionLifecycle/sessionLifecycle.test.ts
M	packages/agent-core-v2/test/app/skillCatalog/skill-tool-manager.test.ts
M	packages/agent-core-v2/test/app/telemetry/agentTelemetryContext.test.ts
M	packages/agent-core-v2/test/app/telemetry/events.test.ts
M	packages/agent-core-v2/test/app/telemetry/telemetryService.test.ts
M	packages/agent-core-v2/test/app/web/web-fetch-service.test.ts
M	packages/agent-core-v2/test/harness/agent.ts
M	packages/agent-core-v2/test/harness/scripted-generate.ts
M	packages/agent-core-v2/test/harness/snapshots.ts
M	packages/agent-core-v2/test/index.test.ts
A	packages/agent-core-v2/test/kosong/contract/errors.test.ts
A	packages/agent-core-v2/test/kosong/contract/generate.test.ts
A	packages/agent-core-v2/test/kosong/contract/usage-tokens.test.ts
A	packages/agent-core-v2/test/kosong/model/catalog.test.ts
A	packages/agent-core-v2/test/kosong/model/completionBudget.test.ts
A	packages/agent-core-v2/test/kosong/model/discovery.test.ts
A	packages/agent-core-v2/test/kosong/model/envOverlay.test.ts
A	packages/agent-core-v2/test/kosong/model/modelAuth.test.ts
A	packages/agent-core-v2/test/kosong/model/modelRequester.test.ts
A	packages/agent-core-v2/test/kosong/model/modelService.test.ts
A	packages/agent-core-v2/test/kosong/model/thinking.test.ts
A	packages/agent-core-v2/test/kosong/protocol/errors.test.ts
A	packages/agent-core-v2/test/kosong/protocol/protocol.test.ts
A	packages/agent-core-v2/test/kosong/protocol/protocolBase.test.ts
A	packages/agent-core-v2/test/kosong/protocol/protocolTrait.test.ts
A	packages/agent-core-v2/test/kosong/provider/composition.test.ts
A	packages/agent-core-v2/test/kosong/provider/errors.test.ts
A	packages/agent-core-v2/test/kosong/provider/kimi.test.ts
A	packages/agent-core-v2/test/kosong/provider/openaiHooks.test.ts
A	packages/agent-core-v2/test/kosong/provider/providerService.test.ts
A	packages/agent-core-v2/test/kosong/stubs.ts
A	packages/agent-core-v2/test/lint/vendor-name-gates.test.ts
M	packages/agent-core-v2/test/os/backends/node-local/tools/bash.test.ts
M	packages/agent-core-v2/test/session/agentLifecycle/agentLifecycle.test.ts
M	packages/agent-core-v2/test/session/cron/cron-tools.test.ts
A	packages/agent-core-v2/test/session/sessionAgentProfileCatalog/sessionAgentProfileCatalog.test.ts
M	packages/agent-core-v2/test/session/sessionSkillCatalog/skillCatalog.test.ts
M	packages/agent-core-v2/test/session/swarm/sessionSwarm.test.ts
M	packages/agent-core-v2/test/tool/tool.test.ts
M	packages/agent-core/src/agent/context/index.ts
M	packages/agent-core/src/agent/context/projector.ts
M	packages/agent-core/src/agent/records/index.ts
A	packages/agent-core/src/agent/replay/turns.ts
M	packages/agent-core/src/index.ts
M	packages/agent-core/src/rpc/core-api.ts
M	packages/agent-core/src/rpc/core-impl.ts
M	packages/agent-core/src/services/message/message.ts
M	packages/agent-core/src/session/index.ts
M	packages/agent-core/src/session/provider-manager.ts
M	packages/agent-core/src/session/subagent-host.ts
M	packages/agent-core/test/agent/context/projector.test.ts
M	packages/agent-core/test/agent/records/index.test.ts
M	packages/agent-core/test/agent/resume.test.ts
M	packages/agent-core/test/harness/model-alias-session.test.ts
M	packages/agent-core/test/harness/runtime-provider.test.ts
M	packages/agent-core/test/harness/skill-session.test.ts
M	packages/agent-core/test/session/subagent-host.test.ts
A	packages/kap-server/src/lib/httpRange.ts
M	packages/kap-server/src/protocol/events-zod.ts
M	packages/kap-server/src/protocol/rest-modelCatalog.ts
M	packages/kap-server/src/protocol/rest-prompt.ts
M	packages/kap-server/src/protocol/tool.ts
M	packages/kap-server/src/routes/fs.ts
M	packages/kap-server/src/routes/modelCatalog.ts
M	packages/kap-server/src/routes/prompts.ts
M	packages/kap-server/src/routes/tools.ts
M	packages/kap-server/src/routes/workspaceFs.ts
M	packages/kap-server/src/services/modelCatalog/modelCatalogRefreshScheduler.ts
M	packages/kap-server/src/services/transcript/coreBinding.ts
M	packages/kap-server/src/start.ts
M	packages/kap-server/src/transport/ws/v1/events.ts
M	packages/kap-server/src/transport/ws/v1/sessionEventBroadcaster.ts
M	packages/kap-server/test/__snapshots__/apiSurface.snapshot.test.ts.snap
M	packages/kap-server/test/fs.test.ts
M	packages/kap-server/test/messages.test.ts
M	packages/kap-server/test/modelCatalog.test.ts
M	packages/kap-server/test/modelCatalogRefreshScheduler.test.ts
M	packages/kap-server/test/prompts.test.ts
M	packages/kap-server/test/rpc.test.ts
M	packages/kap-server/test/services/transcript.test.ts
M	packages/kap-server/test/sessionEventBroadcaster.test.ts
M	packages/kap-server/test/tasks.test.ts
M	packages/kap-server/test/tools.test.ts
M	packages/kap-server/test/transcript.test.ts
M	packages/kap-server/test/workspaceFs.test.ts
M	packages/klient/AGENTS.md
A	packages/klient/examples/kimi-select-tools.ts
A	packages/klient/examples/model-requester-boundary.ts
M	packages/klient/examples/smoke.ts
M	packages/klient/package.json
M	packages/klient/src/contract/agent/rpc.ts
M	packages/klient/src/contract/global/catalog.ts
M	packages/klient/src/contract/global/events.ts
M	packages/klient/src/contract/global/models.ts
A	packages/klient/src/contract/global/providerDiscovery.ts
M	packages/klient/src/contract/global/providers.ts
M	packages/klient/src/contract/index.ts
M	packages/klient/src/core/facade/agent.ts
M	packages/klient/src/core/facade/global.ts
M	packages/klient/src/index.ts
A	packages/klient/src/transports/args.ts
M	packages/klient/src/transports/ipc/channel.ts
M	packages/klient/src/transports/memory/serviceRegistry.ts
M	packages/klient/test/contract-parity.ts
A	packages/klient/test/e2e/invalid-input-matrix.test.ts
M	packages/kosong/src/errors.ts
M	packages/kosong/src/index.ts
M	packages/kosong/src/providers/anthropic.ts
M	packages/kosong/src/providers/openai-common.ts
M	packages/kosong/src/providers/openai-legacy.ts
M	packages/kosong/src/providers/openai-responses.ts
M	packages/kosong/test/anthropic-errors.test.ts
M	packages/kosong/test/errors.test.ts
M	packages/kosong/test/openai-common-errors.test.ts
M	packages/kosong/test/openai-legacy.test.ts
M	packages/kosong/test/openai-responses.test.ts
M	packages/node-sdk/src/index.ts
M	packages/node-sdk/src/rpc.ts
M	packages/node-sdk/src/types.ts
M	packages/pi-tui/AGENTS.md
M	packages/pi-tui/src/tui.ts
M	packages/pi-tui/test/tui-render.test.ts
M	packages/protocol/src/__tests__/rest-prompt.test.ts
M	packages/protocol/src/__tests__/tool.test.ts
M	packages/protocol/src/events.ts
M	packages/protocol/src/rest/prompt.ts
M	packages/protocol/src/tool.ts
M	packages/transcript/src/store/transcriptStore.ts
M	packages/transcript/src/wire/schema.ts
M	packages/transcript/test/store.test.ts
M	pnpm-lock.yaml
```

</details>

<details><summary>CodexBar: 21 changed paths</summary>

```text
M	Sources/CodexBar/MenuCardView+ModelHelpers.swift
M	Sources/CodexBarCore/Providers/Abacus/AbacusUsageSnapshot.swift
M	Sources/CodexBarCore/Providers/Alibaba/AlibabaCodingPlanProviderDescriptor.swift
M	Sources/CodexBarCore/Providers/Alibaba/AlibabaTokenPlanProviderDescriptor.swift
M	Sources/CodexBarCore/Providers/Codebuff/CodebuffUsageSnapshot.swift
M	Sources/CodexBarCore/Providers/CommandCode/CommandCodeUsageSnapshot.swift
M	Sources/CodexBarCore/Providers/Copilot/CopilotProviderDescriptor.swift
M	Sources/CodexBarCore/Providers/Cursor/CursorProviderDescriptor.swift
M	Sources/CodexBarCore/Providers/Cursor/CursorStatusProbe.swift
M	Sources/CodexBarCore/Providers/Doubao/DoubaoProviderDescriptor.swift
M	Sources/CodexBarCore/Providers/ElevenLabs/ElevenLabsUsageFetcher.swift
M	Sources/CodexBarCore/Providers/Grok/GrokProviderDescriptor.swift
M	Sources/CodexBarCore/Providers/OpenAI/OpenAIAPICreditBalanceFetcher.swift
M	Sources/CodexBarCore/Providers/OpenCodeGo/OpenCodeGoProviderDescriptor.swift
M	Sources/CodexBarCore/Providers/ProviderDescriptor.swift
M	Sources/CodexBarCore/UsageFetcher.swift
A	Sources/CodexBarCore/UsagePercent.swift
M	Tests/CodexBarTests/BoundedChildProcessProofTests.swift
A	Tests/CodexBarTests/ProviderPaceCapabilityTests.swift
A	Tests/CodexBarTests/UsagePercentTests.swift
M	Tests/CodexBarTests/ZenMuxProviderTests.swift
```

</details>
