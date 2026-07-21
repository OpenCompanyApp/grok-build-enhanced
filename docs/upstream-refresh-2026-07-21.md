# Provider-reference refresh ledger — 2026-07-21

This ledger records the exact source pins, complete commit and changed-path inventories, behavior decisions, local implementation, and validation obligations for the 2026-07-21 refresh. Provider-reference history is evidence, not content-merged application history. The ignored `inspiration/` checkouts are not part of the candidate tree.

## Immutable boundary

- Pre-refresh Enhanced commit: `f255295d2b85b6dcab960d9af6e0d607eafec1ca`
- Pre-refresh Enhanced tree: `0c8ac9f8e9502e548a95baa553ae26988e5fac44`
- Isolated branch: `refresh/upstream-20260721`
- Check timestamp: `2026-07-21T01:40:46Z`
- Grok Build remained at commit `a881e6703f46b01d8c7d4a5437683546df30449d`, tree `e3a013ffc66a6dd77ec1c35dfe261741bcf84928`.
- Because the Grok commit and tree did not advance, the existing acknowledgement remains exact and this refresh must not create a duplicate zero-tree-delta marker.

## Advanced source pins

| Source | Previous reviewed commit / tree | Audited commit / tree | Range | Changed paths | Ancestor |
| --- | --- | --- | ---: | ---: | --- |
| OpenAI Codex | `3e2f79727a4e8ddfc8e3acb838d496b121094b9e` / `2e260a5ad1a15dbc9e66569f80b799241b446c87` | `c0cd337766ff27a75623c5baba199389f94f2ab3` / `b476cd40265f1ec474169eedfb7dabc2924c4d7c` | 56 commits | 393 | yes |
| OpenCode | `67caf894e0843ee370e72839e8265e483233479b` / `a791ac7f3b077fa7a93715afad94f13f6d3faf6b` | `849c2598abc7d2b40261e74b5826bc74ffc78308` / `1484a4c6b34f52f4d77678d615709777a4962f28` | 30 commits | 84 | yes |
| Kimi Code | `df6899553962d1764c9f4c3bec1b63c811cb425e` / `8f20e2e7152633ada3e58ea746678e0822b9a3ea` | `c2d7bebd04106473bb4dbab2903756aa3f14a880` / `6372ff7ef2b4e61f6116178c9af2256d7c4e995c` | 13 commits | 232 | yes |

The other nine tracked references were fetched and remained byte-identical at their recorded reviewed revisions: Grok Build, OpenCode Codex auth, Warp themes, Kimi CLI, Z.AI Python SDK, Z.AI coding plugins, GLM-5, CodexBar, and the Z.AI usage-browser reference.

## Behavior inventory

The outcomes below classify observable behavior separately from source architecture. `adopt` changes the Enhanced candidate; `already equivalent` cites an existing local contract; and `not applicable` is limited to replacement application architecture or provider-reference product surfaces outside the preserved Grok application. There are zero temporary deferrals and zero unclassified provider behaviors.

| Evidence ID | Source behavior | Outcome and local contract | Focused evidence |
| --- | --- | --- | --- |
| `CDX-C0CD-PROXY` | Resolve outbound routes explicitly, including macOS CFNetwork/SystemConfiguration and Windows WinHTTP PAC/WPAD/static proxies, with explicit environment fallback and bounded caching. | **adopt.** Enhanced adds a provider-scoped leaf crate and lazy route-aware clients for Codex inference, OAuth, catalog, usage, hosted search, and image generation/edit. It does not enable reqwest system proxy behavior globally, so xAI, Kimi, Z.AI, Custom, telemetry, and update clients remain unchanged. | `xai-grok-provider-http` route/cache/redaction/platform tests; shell, sampler, and tools client tests. |
| `CDX-C0CD-IMAGE` | A Codex invalid-tool-image HTTP 400 is terminal; do not mutate history and replay the turn. | **already equivalent.** Codex response diagnostics are reduced to a fixed message before generic image-stripping classification, and HTTP 400 is terminal. The request history is untouched. | `codex_invalid_image_bad_request_is_terminal_without_replay`. |
| `CDX-C0CD-CATALOG` | Refresh catalog metadata and preserve omitted `supports_reasoning_summary_parameter` as supported by default. | **already equivalent.** The authenticated dynamic catalog preserves unknown model metadata and defaults the omitted summary-support flag to `true`; no static entitlement list replaces the server catalog. | `sends_exact_scoped_headers_and_parses_rich_catalog`; `missing_reasoning_summary_support_flag_defaults_to_supported`. |
| `CDX-C0CD-COMPACTION` | Optimize remote-compaction history handling without changing the provider wire result. | **already equivalent.** Enhanced already performs provider-bound remote compaction over cloned request history, preserves durable session records, and isolates continuation state by Codex credential/model. | Existing remote-compaction, resume, model-switch, and persistence tests. |
| `CDX-C0CD-MANAGED-PROXY-NA` | Feed Codex managed permission-profile network proxy settings into Codex's internal network-proxy feature. | **not applicable.** Enhanced does not import the Codex permission-profile/network-sandbox architecture. Its preserved Grok permissions remain authoritative; ordinary OS and environment proxy behavior is adopted independently by `CDX-C0CD-PROXY`. | Source diff plus provider-scoped route boundary tests. |
| `CDX-C0CD-APP-NA` | Codex app-server, Codex TUI, SQLite rollout pagination, Guardian, Codex Apps/plugin service, Codex-specific sandbox, audio, visualization, and packaging changes. | **not applicable.** These do not alter the ChatGPT Codex auth, catalog, Responses, compaction, usage, or hosted-tool contracts exposed through the existing Grok application. No Codex replacement app architecture is imported. | Complete commit/path inventories below and provider matrix review. |
| `OC-849C-OVERFLOW` | Recognize additional provider context-window overflow messages. | **adopt.** Enhanced centralizes the wording in `xai-grok-compaction` and delegates sampling classification to it. Explicit rate-limit, throttling, and service-unavailable guards prevent token-throttle messages from being mistaken for context overflow. | Compaction classifier regressions and sampler retry-routing regressions. |
| `OC-849C-ATTACHMENT-WIRE` | Omit whitespace-only text parts when a user message also contains an image. | **adopt.** Chat Completions, Responses, and Messages wire projections filter only blank text adjacent to images. Nonblank bytes, text-only whitespace, and durable conversation items remain unchanged. | Conversation projection and durable-history regressions for all three protocols. |
| `OC-849C-MISTRAL-ID` | Normalize Mistral-family tool-call IDs to exactly nine ASCII alphanumeric characters and keep call/result IDs paired. | **adopt.** The transform is Custom-provider-only, request-local, deterministic, collision-safe, and applied to Mistral, Devstral, Codestral, Pixtral, and Mixtral model families before network I/O. | Sampler normalization, collision, family, and provider-isolation tests. |
| `OC-849C-KIMI-EFFORT` | Select adaptive Kimi thinking effort from the active session/model variant. | **already equivalent.** Enhanced scopes effort to each Kimi sampling configuration/session and emits explicit provider-native thinking metadata, including the highest catalog-supported levels. | Kimi effort mapping, Messages/Chat wire, session model-switch, and cache-affinity tests. |
| `OC-849C-APP-NA` | OpenCode desktop/web UI, console, release, mention editing, terminal theme, session-import presentation, and unrelated provider defaults. | **not applicable.** OpenCode remains an interoperability reference, not a runtime provider or replacement UI. | Complete commit/path inventories below. |
| `KIMI-C2D7-EFFORT` | Scope thinking effort to the current CLI/VS Code session. | **already equivalent.** Enhanced stores effort in provider/session sampling state rather than a global Kimi setting and includes effort in opaque cache affinity without exposing session identity. | Kimi effort and cache-affinity tests; model-switch/session sampling tests. |
| `KIMI-C2D7-ACP-AUTH` | Allow ACP sessions to start from a configured non-OAuth provider credential. | **already equivalent.** Enhanced's ACP path explicitly loads the selected Kimi API-key record and binds it to the Kimi provider; it never requires xAI or Codex OAuth for that session. | ACP provider credential gate, Kimi credential binding, auth-error, and provider-isolation tests. |
| `KIMI-C2D7-APP-NA` | Kimi transcript RPC replacement, VS Code/web/tips/releases, foreground web servers, and agent-core-v2 permission broadcasts. | **not applicable.** These belong to the Kimi replacement application/agent engine and do not change the Kimi provider adapter's auth, catalog, inference, usage, hosted search/fetch, retry, or logout contracts. | Complete commit/path inventories below. |

## Provider isolation and legal review

- Codex system-proxy discovery is initialized lazily and only by Codex-owned clients. Constructors remain nonblocking, redirects remain refused on credential-bearing clients, and HTTP/1 fallback retains its explicit no-pooling policy.
- Proxy URLs, credentials, bypass lists, destination URLs, platform failures, account state, and credential bindings cannot enter route errors or debug output. Destination cache keys are SHA-256 digests; success and unavailable entries use 60-second and 5-second TTLs.
- xAI, Kimi Code, Z.AI Coding Plan, and Custom client construction was not changed to use the new resolver. No provider can fall through to Codex credentials, endpoints, catalog, usage, hosted tools, or logout state.
- The platform resolver was adapted from the exact Apache-2.0 Codex pin. Root and crate-local third-party notices identify the source paths and modification, and the lockfile records the macOS framework crates. The generated root `Cargo.toml` was not edited.

## Validation gate

The audited surfaces passed the following checks on the candidate tree:

- `cargo fmt --all` and `git diff --check`;
- 18 provider HTTP, 129 compaction, 305 sampling-types, 298 sampler, and 2,762 tools library tests;
- 59 Codex auth/usage tests (plus one credential-gated live test left ignored), 25 catalog tests, and focused xAI/Codex/Kimi/Z.AI/Custom isolation checks;
- `CARGO_INCREMENTAL=0 cargo check -p xai-grok-pager-bin`;
- 70 fork-script unit tests, fork contracts, release-pipeline tests, install-script tests, generated-workspace checks, Codex hosted-search contracts, Warp vendor locks, workflow pins, and tracked-secret checks;
- strict committed-tree manifest ownership after the thematic commits described by this ledger.

The macOS CFNetwork/SystemConfiguration path compiled in the binary check. Pure PAC, WinHTTP proxy-list, bypass, caching, single-flight, environment-fallback, and redaction behavior runs cross-platform in the 18-test leaf suite. No Windows Rust target is installed in this environment, so the checked-in Windows FFI module and its pinned-source platform tests could not be compiled here; the module differs from the exact audited Codex source only in its nonsecret user-agent label.

An exploratory whole-crate `xai-grok-shell --lib` run also exposed seven failures in untouched baseline files (`agent/config.rs` and `session/acp_conversion.rs`) before aborting on a stack overflow. Running those filters individually reproduced their existing assertions, while every refresh-owned Codex auth, usage, catalog, and provider-isolation group passed. The unrelated whole-shell result is recorded here rather than hidden or broadened into this provider refresh.

Credential-gated live checks are not required to classify source behavior and must never print authenticated payloads. The installed Enhanced 0.2.6 build had already completed a successful entitled headless Codex Sol web-search smoke test immediately before this refresh; this candidate's new route behavior is validated offline and remains unpublished.

## Outcome reconciliation

Across 99 advanced source commits:

- **Adopt:** 4 commit-level behavior groups
- **Already equivalent:** 6 commit-level behavior groups
- **Not applicable:** 89 commits whose changed behavior is confined to replacement application architecture or unrelated reference-provider surfaces
- **Temporarily deferred:** 0
- **Unclassified:** 0

A commit can cite more than one evidence ID when it contains both a provider contract and a replacement-app presentation change. The behavior table, rather than raw commit count, is authoritative for those split cases.

## Complete commit disposition inventory

### OpenAI Codex: 3e2f797..c0cd337

| # | Commit | Subject | Evidence |
| ---: | --- | --- | --- |
| 1 | `aa982319c2642918182e88bdace1ef9fea9ebc4b` | Speed up TUI Markdown layout (#34216) | `CDX-C0CD-APP-NA` |
| 2 | `74bfbda9b5879ebb7ff90d53c7f70b616a677393` | Keep incremental rendering with visualization context (#34217) | `CDX-C0CD-APP-NA` |
| 3 | `854a82dbfda67b549831c032c9d172c687d1bb30` | Track TUI command completion separately from output (#34218) | `CDX-C0CD-APP-NA` |
| 4 | `d0516cfe4ba03a87df6f88224a3d26e99907e61e` | Avoid buffering replay-irrelevant thread notifications (#34222) | `CDX-C0CD-APP-NA` |
| 5 | `6a54efb76bf53f498ae184d8b3b1f46c23f78ea5` | Cache finalized Markdown history rendering (#34223) | `CDX-C0CD-APP-NA` |
| 6 | `c86b1be3cdbe12307843bcc9e7a44c1904ddcdf1` | Avoid cloning file changes in TUI diff rendering (#34224) | `CDX-C0CD-APP-NA` |
| 7 | `7844386e3de08febd13075eaaaf0e6f9dbe52c58` | Backfill completion items only for the active exec turn (#34226) | `CDX-C0CD-APP-NA` |
| 8 | `5a208c1fc353573fa3838c70a90ea9c59ad8884c` | Persist names for paginated threads (#34229) | `CDX-C0CD-APP-NA` |
| 9 | `a97ae65362e8b03c196a24c961cf7bfaa129db56` | Remeasure dynamic cells in the transcript overlay (#34232) | `CDX-C0CD-APP-NA` |
| 10 | `678157acaa819d5510adfe359abb5d0392cfe461` | Avoid redundant TUI subagent metadata requests (#34234) | `CDX-C0CD-APP-NA` |
| 11 | `bf3c1972b7d045c0a3a48dff91f381070f8f69e1` | Migrate legacy exec policy allow rules (#34271) | `CDX-C0CD-APP-NA` |
| 12 | `2deed3fb9c00c74dac3d177ea700d6fb7a94539d` | Preserve zsh tied PATH exports in shell snapshots (#34293) | `CDX-C0CD-APP-NA` |
| 13 | `86102db5a1a7a49ce08e79f900e0897d94aa3770` | Reject unsupported history modes when loading rollouts (#34344) | `CDX-C0CD-APP-NA` |
| 14 | `221a34102929d9a82f492d3d5e4213cdc76ac441` | Remove unused Rust helpers (#34345) | `CDX-C0CD-APP-NA` |
| 15 | `2244d11a1d9eaa389a5da4c99b746c74240abd8c` | Track inline visualization directives during streaming (#34346) | `CDX-C0CD-APP-NA` |
| 16 | `ada5a79ddf516fffefbc4846a9490f092412c61a` | Avoid cloning deferred TUI lifecycle payloads (#34347) | `CDX-C0CD-APP-NA` |
| 17 | `eceb3eeaf3a68d732596fd8c0e8a6807f9166770` | Cache TUI flex heights across frame passes (#34348) | `CDX-C0CD-APP-NA` |
| 18 | `2661d8577ee17885f63f3a7c95dc96f5b3c67593` | Parallelize TUI bootstrap requests (#34355) | `CDX-C0CD-APP-NA` |
| 19 | `20440a0833c475b899074f1794e543350fccf5b8` | Render streamed command output through preview iterators (#34357) | `CDX-C0CD-APP-NA` |
| 20 | `ef6b597f416ec2a105c50ee65caf84a1daa86983` | Keep streamed command output bounded in the TUI (#34359) | `CDX-C0CD-APP-NA` |
| 21 | `1e20272fa5a4885240bd92bd02f42e56f61f1045` | Avoid cloning thread history for token usage replay (#34361) | `CDX-C0CD-APP-NA` |
| 22 | `f944456d81f32cadd96d037c190d7ce65a956306` | Animate Max and Ultra reasoning effort changes (#34365) | `CDX-C0CD-APP-NA` |
| 23 | `28aacbb9d9e4fd1d8f566ab221834e74fd83875a` | Avoid cloning hyperlink text during TUI rendering (#34366) | `CDX-C0CD-APP-NA` |
| 24 | `b6de5b524cdc07400b0ac049ae70ac8e90285e67` | Use app-server skill metadata directly in the TUI (#34368) | `CDX-C0CD-APP-NA` |
| 25 | `5c18cc0acc3734f0e78e422a7fd94ea4a2be652e` | Clear stale Guardian reviews when turns end (#34371) | `CDX-C0CD-APP-NA` |
| 26 | `9a7e823e5be340e6e54bfac60edc6e1e0fd72d49` | Extend second-based latency histogram buckets (#34375) | `CDX-C0CD-APP-NA` |
| 27 | `7e51abbbd12282a1d7da4e77f72a28f82249b652` | Avoid rendering generated images twice (#34378) | `CDX-C0CD-APP-NA` |
| 28 | `8431dc590a5bba9a1185d5579a5aabfbc469e50b` | Stop retrying turns with invalid tool images (#34380) | `CDX-C0CD-IMAGE` |
| 29 | `6b9a5592a652f107dc7f75720d69c4b22af4d57b` | Avoid cloning Responses WebSocket payloads (#34381) | `CDX-C0CD-APP-NA` |
| 30 | `19b2273d8a54ec28aa0174a95884182aa6e5081e` | Keep paginated thread Git metadata in SQLite (#34382) | `CDX-C0CD-APP-NA` |
| 31 | `b00c9b2e16ccdbf2c7c8d58a590e0fc2ca97573b` | Mark multi-agent v2 as stable (#34383) | `CDX-C0CD-APP-NA` |
| 32 | `692a0fb7e52dccead1d96894273d76eaf5af1b5a` | Update packaged ripgrep to 15.2.0 (#34384) | `CDX-C0CD-APP-NA` |
| 33 | `6f785632b000f7d8e85100506b88b3bab5b8d8a0` | Preserve audio across history and tool outputs (#34385) | `CDX-C0CD-APP-NA` |
| 34 | `2793c826e8f5a04e0619a18e9bbdae91c0435cd5` | Enable memories for paginated threads (#34386) | `CDX-C0CD-APP-NA` |
| 35 | `5a4f5ee64c4e7de22c21f1c38feb3edfb167b7d8` | Refresh bundled model metadata (#34387) | `CDX-C0CD-CATALOG`, `CDX-C0CD-APP-NA` |
| 36 | `6bf4845b60e0abccd0c64690e9c7591e0efb85d8` | Route Codex Apps MCP through plugin service (#34389) | `CDX-C0CD-APP-NA` |
| 37 | `45ac251e178416ff5c3022457ad8d2778c0d4549` | Use copy-on-write storage for history snapshots (#34390) | `CDX-C0CD-APP-NA` |
| 38 | `bd92b056ddd91bd7c2ecfea3d8773f7eb5a879a6` | Ignore inherited ACEs when refreshing Windows write roots (#34392) | `CDX-C0CD-APP-NA` |
| 39 | `e4836f998da166aba456f60d2e74eb79d6e2542b` | Add configurable hook context spill limits (#34393) | `CDX-C0CD-APP-NA` |
| 40 | `8c41ed33ce3e39460e7b13b14c35e0c39bb5980d` | Run compact session-start hooks before turn continuation (#34396) | `CDX-C0CD-APP-NA` |
| 41 | `e52c35b0001ea3e4a1744b99c4250a5b1a09e44d` | Propagate approval rejection reasons (#34400) | `CDX-C0CD-APP-NA` |
| 42 | `ec3140db1297f3acebec7d6916b329cad3b12693` | Update tests for history and hook API changes (#34403) | `CDX-C0CD-APP-NA` |
| 43 | `b7e39aa31608b6eaba4f317538a8f82985a9e854` | Resolve paginated rollout lineages (#34407) | `CDX-C0CD-APP-NA` |
| 44 | `19940967bdb5ac04aec5d08ebd465481f1ac964d` | Support threadless MCP connections without event channels (#34408) | `CDX-C0CD-APP-NA` |
| 45 | `44481a1c4548d1cc0cc3c95aa03b59ec4cba074a` | Limit the Linux `/proc` preflight filesystem view (#34409) | `CDX-C0CD-APP-NA` |
| 46 | `81e89fa5af13012c8313f032a17b11b9a5170d33` | Require absolute paths for test SQLite configuration (#34411) | `CDX-C0CD-APP-NA` |
| 47 | `687f05cb946d10c96f90dd7ce82e11465c6e20a7` | Remove CSV-backed agent jobs (#34413) | `CDX-C0CD-APP-NA` |
| 48 | `cf821e8ec850c6d8380feea0e84859dd8ff54cd0` | Show completed hook warnings in TUI headers (#34416) | `CDX-C0CD-APP-NA` |
| 49 | `60272096bc125ad7bd8ec26508b19d1e0db2874b` | Enrich app/read connector metadata (#34417) | `CDX-C0CD-APP-NA` |
| 50 | `35c2278dd5c49daf8a4e44468038aed9be9e866e` | Support Windows sandboxing in the exec server (#34423) | `CDX-C0CD-APP-NA` |
| 51 | `56c11cf6586c0579e4e3eca14eefb0916b14c78c` | Move shared skill models into `codex-skills` (#34429) | `CDX-C0CD-APP-NA` |
| 52 | `fd3c1dc13d0a0941af406e1bc1f697c9d14110ea` | Optimize remote compaction history handling (#34431) | `CDX-C0CD-COMPACTION` |
| 53 | `2be7d3bcd9d1aec2780f0a71fe79cbb5afd877a1` | Support catalog messages for non-request approval policies (#34434) | `CDX-C0CD-APP-NA` |
| 54 | `c9ef7eff005c3299a5a5f0004c34c6a3eedf2564` | Resolve outbound proxy routes explicitly (#34435) | `CDX-C0CD-PROXY` |
| 55 | `88fac6fe108237a105d3203e3508b0d531054312` | Honor managed permission profiles in network proxy resolution (#34436) | `CDX-C0CD-MANAGED-PROXY-NA` |
| 56 | `c0cd337766ff27a75623c5baba199389f94f2ab3` | Increase the patch approval test timeout (#34438) | `CDX-C0CD-APP-NA` |

### OpenCode: 67caf89..849c259

| # | Commit | Subject | Evidence |
| ---: | --- | --- | --- |
| 1 | `7985c2066a8f38c48a7d8fefbafcbab96ffa3117` | fix(app): show keybind tooltips on prompt input controls (#37824) | `OC-849C-APP-NA` |
| 2 | `ba4b8e21f4ecd70f30b8dc24458f56a4fe84eca5` | feat(app): toggle debug tools from dev badge (#36689) | `OC-849C-APP-NA` |
| 3 | `2a097f3af70542ec596814c99d997ff2c8562682` | fix(llm): expand context overflow patterns (#37840) | `OC-849C-OVERFLOW` |
| 4 | `61653a20a5729ba1068a39b3287b7500934c01e8` | sync | `OC-849C-APP-NA` |
| 5 | `cfa134f042151877c03222f34a7bdba2c770609b` | Merge branch 'dev' of github.com:anomalyco/opencode into dev | `OC-849C-APP-NA` |
| 6 | `cd46f22d513d60b7a9bdca1111d25c50d2398355` | sync | `OC-849C-APP-NA` |
| 7 | `20a3a2138ea3091d1f33e03502779e6f567a3f19` | feat(opencode): use adaptive thinking effort for kimi family on anthr… (#37696) | `OC-849C-KIMI-EFFORT` |
| 8 | `3b719aba48c86842c4db49b85d2555b312ff34c1` | chore: generate | `OC-849C-APP-NA` |
| 9 | `5586f9675e3e947e0bf690979d590d9224f968f4` | fix(app): restore model variant accessibility (#37857) | `OC-849C-APP-NA` |
| 10 | `3fc5af6dd1fd8c667a101409307bd4bc51633923` | feat(app): update settings description line height (#37759) | `OC-849C-APP-NA` |
| 11 | `3ba3954aa03ec92371075f171a4d4c14c7b8c833` | feat(app): review panel improvements (#36716) | `OC-849C-APP-NA` |
| 12 | `9105f359668c4ea7d348a050c3ecde2ea3228358` | chore: generate | `OC-849C-APP-NA` |
| 13 | `a19b52e85bf2630b86157030e2cf7c9fc20ce552` | fix(app): omit empty prompt text parts (#37577) | `OC-849C-ATTACHMENT-WIRE` |
| 14 | `4a81e8392b4c18cbcc0914527bdab8ff94b9a434` | fix(app): gate notification server selection (#37890) | `OC-849C-APP-NA` |
| 15 | `8bec40a816a993a12dd5108133e1afd31d3f9bf1` | fix(app): align Windows upgrade drawer (#37895) | `OC-849C-APP-NA` |
| 16 | `5542415b6424aa9bad8827dc72e8cd0630b2c0ab` | fix(app): shrink Windows drawer close button (#37909) | `OC-849C-APP-NA` |
| 17 | `cc3615b535d05e0301ee83a5f2e207432e58a595` | fix(app): tolerate missing message parents (#37918) | `OC-849C-APP-NA` |
| 18 | `e2531dff0f361c208d1deb48091f816074060296` | chore(app): bump ghostty-web (#37923) | `OC-849C-APP-NA` |
| 19 | `b67fda133a186c7c294c8822f7eda89f36d57aff` | chore: update nix node_modules hashes | `OC-849C-APP-NA` |
| 20 | `43e472bba70728d79ad9a2003067b08104e24f7a` | feat(app): sync embedded terminal theme (#37931) | `OC-849C-APP-NA` |
| 21 | `70535a1bad273b0455139b8b48f21f95da9d6f07` | chore: update nix node_modules hashes | `OC-849C-APP-NA` |
| 22 | `da312b009ca4bef725d8b186d8f9e3e4e0bbd9de` | fix(app): preserve command menu drafts (#37942) | `OC-849C-APP-NA` |
| 23 | `47fc6f299148eb3af8a95e8ee5bf3e59073fec83` | fix(app): preserve mentions during paste (#37940) | `OC-849C-APP-NA` |
| 24 | `4872c48c230728150e8e3406722943450ed58dcb` | fix(app): complete mentions at cursor (#37941) | `OC-849C-APP-NA` |
| 25 | `96ff82a11fc3073a359cd796b199228de70520aa` | fix(console): remove desktop promo overlay | `OC-849C-APP-NA` |
| 26 | `d36a2d8981bad2a456835c65794fc96d980187ff` | sync release versions for v1.18.4 | `OC-849C-APP-NA` |
| 27 | `4cc022481c184cb9d5e14f7e882b6749d2414553` | fix(opencode): display proper session import errors (#36258) | `OC-849C-APP-NA` |
| 28 | `5a8ee27254c61da27a99c78beebe69557f12c8e3` | fix(provider): update muse-spark reasoning default and system prompt (#37562) | `OC-849C-APP-NA` |
| 29 | `3033afba5134c486dc3f8f5ae96c8b653285eeda` | fix(provider): normalize Mistral family tool call IDs (#37982) | `OC-849C-MISTRAL-ID` |
| 30 | `849c2598abc7d2b40261e74b5826bc74ffc78308` | docs: generalize session title model description (#37986) | `OC-849C-APP-NA` |

### Kimi Code: df68995..c2d7beb

| # | Commit | Subject | Evidence |
| ---: | --- | --- | --- |
| 1 | `11c1683a1cd2adab276562419d2d353629063d80` | feat: scope thinking effort to the current session (#1933) | `KIMI-C2D7-EFFORT` |
| 2 | `d71bf9e5a56b5978316e715f7c131c784967d562` | feat(web): add a cache invalidation note to the model switcher (#1940) | `KIMI-C2D7-APP-NA` |
| 3 | `a05228c67122c8233dc87226ce0ca7414780b680` | ci: release packages (#1868) | `KIMI-C2D7-APP-NA` |
| 4 | `5ae60fa6736b63b80bd764ef01d6c0334eb80595` | feat(transcript): add unified transcript layer, drop the /api/v2 RPC surface (#1888) | `KIMI-C2D7-APP-NA` |
| 5 | `f6f4192957ace3f0cceb734a04b3b26b1d2f88be` | fix(agent-core-v2): broadcast permission mode switches to live subagents (#1948) | `KIMI-C2D7-APP-NA` |
| 6 | `e8f2a077d3d3bb7e0c42be370da1591a6fa35f10` | docs(changelog): sync 0.28.0 from apps/kimi-code/CHANGELOG.md (#1944) | `KIMI-C2D7-APP-NA` |
| 7 | `9223a37622f1bff2b0ce58e04dc56d76b865b571` | feat(vscode): scope thinking effort to the current session (#1951) | `KIMI-C2D7-APP-NA` |
| 8 | `bcee3ac54272b9c5611fc3793517cc21a0d0d081` | chore(vscode): release 0.6.4 (#1958) | `KIMI-C2D7-APP-NA` |
| 9 | `dde92fe5abf1f5eb7d246abe8ed9ced7ae4b6640` | feat: support max and exact version targeting for tips banner (#1960) | `KIMI-C2D7-APP-NA` |
| 10 | `ad8cc8525198a08bc1181cee9a15bbb4521cd9bc` | feat(cli): foreground-only web servers, deprecated kimi server kill (#1967) | `KIMI-C2D7-APP-NA` |
| 11 | `c5b6103bb9b0a163d48cbce0034c3fc7dea7c344` | fix(acp): allow configured provider auth (#934) | `KIMI-C2D7-ACP-AUTH` |
| 12 | `efacf0452d46f5dbd67499eabc053869495d5213` | ci: release packages (#1946) | `KIMI-C2D7-APP-NA` |
| 13 | `c2d7bebd04106473bb4dbab2903756aa3f14a880` | docs(changelog): sync 0.28.1 from apps/kimi-code/CHANGELOG.md (#1975) | `KIMI-C2D7-APP-NA` |

## Complete changed-path inventory

These are exact `git diff --name-status <reviewed> <audited>` records. Rename/copy rows retain Git's source and destination columns.

<details><summary>OpenAI Codex: 393 changed paths</summary>

```text
M	MODULE.bazel.lock
M	codex-rs/Cargo.lock
M	codex-rs/Cargo.toml
M	codex-rs/app-server-daemon/src/lib.rs
M	codex-rs/app-server-protocol/schema/json/ApplyPatchApprovalResponse.json
M	codex-rs/app-server-protocol/schema/json/ExecCommandApprovalResponse.json
M	codex-rs/app-server-protocol/schema/json/codex_app_server_protocol.schemas.json
M	codex-rs/app-server-protocol/schema/json/codex_app_server_protocol.v2.schemas.json
M	codex-rs/app-server-protocol/schema/json/v2/AppsReadResponse.json
M	codex-rs/app-server-protocol/schema/json/v2/ConfigRequirementsReadResponse.json
M	codex-rs/app-server-protocol/schema/json/v2/HooksListResponse.json
M	codex-rs/app-server-protocol/schema/typescript/ReviewDecision.ts
M	codex-rs/app-server-protocol/schema/typescript/v2/ConfiguredHookHandler.ts
M	codex-rs/app-server-protocol/schema/typescript/v2/ConnectorMetadata.ts
M	codex-rs/app-server-protocol/schema/typescript/v2/HookMetadata.ts
M	codex-rs/app-server-protocol/src/protocol/item_builders.rs
M	codex-rs/app-server-protocol/src/protocol/thread_history.rs
M	codex-rs/app-server-protocol/src/protocol/v2/apps.rs
M	codex-rs/app-server-protocol/src/protocol/v2/config.rs
M	codex-rs/app-server-protocol/src/protocol/v2/item.rs
M	codex-rs/app-server-protocol/src/protocol/v2/plugin.rs
M	codex-rs/app-server/Cargo.toml
M	codex-rs/app-server/README.md
M	codex-rs/app-server/src/app_info.rs
M	codex-rs/app-server/src/bespoke_event_handling.rs
M	codex-rs/app-server/src/command_exec.rs
M	codex-rs/app-server/src/request_processors.rs
M	codex-rs/app-server/src/request_processors/apps_processor.rs
M	codex-rs/app-server/src/request_processors/apps_processor/installed.rs
M	codex-rs/app-server/src/request_processors/catalog_processor.rs
M	codex-rs/app-server/src/request_processors/config_processor.rs
M	codex-rs/app-server/src/request_processors/process_exec_processor.rs
M	codex-rs/app-server/src/request_processors/thread_lifecycle.rs
M	codex-rs/app-server/src/request_processors/thread_processor.rs
M	codex-rs/app-server/src/request_processors/token_usage_replay.rs
M	codex-rs/app-server/src/request_processors/turn_processor.rs
M	codex-rs/app-server/src/thread_status.rs
M	codex-rs/app-server/tests/common/test_app_server.rs
M	codex-rs/app-server/tests/suite/v2/app_read.rs
M	codex-rs/app-server/tests/suite/v2/config_rpc.rs
M	codex-rs/app-server/tests/suite/v2/dynamic_tools.rs
M	codex-rs/app-server/tests/suite/v2/hooks_list.rs
M	codex-rs/app-server/tests/suite/v2/imagegen_extension.rs
M	codex-rs/app-server/tests/suite/v2/mcp_resource.rs
M	codex-rs/app-server/tests/suite/v2/remote_thread_store.rs
M	codex-rs/app-server/tests/suite/v2/thread_list.rs
M	codex-rs/app-server/tests/suite/v2/thread_read.rs
M	codex-rs/app-server/tests/suite/v2/turn_start.rs
M	codex-rs/chatgpt/src/connectors.rs
M	codex-rs/cli/src/doctor/thread_inventory.rs
M	codex-rs/cli/src/main.rs
M	codex-rs/cli/tests/debug_clear_memories.rs
M	codex-rs/codex-api/src/common.rs
M	codex-rs/codex-api/src/endpoint/responses_websocket.rs
M	codex-rs/codex-mcp/src/connection_manager.rs
M	codex-rs/codex-mcp/src/connection_manager_tests.rs
M	codex-rs/codex-mcp/src/elicitation.rs
M	codex-rs/codex-mcp/src/mcp/mod.rs
M	codex-rs/codex-mcp/src/mcp/mod_tests.rs
M	codex-rs/codex-mcp/src/rmcp_client.rs
M	codex-rs/config/src/config_requirements.rs
M	codex-rs/config/src/config_toml.rs
M	codex-rs/config/src/hook_config.rs
M	codex-rs/config/src/hooks_tests.rs
M	codex-rs/config/src/permissions_toml.rs
M	codex-rs/config/src/thread_config.rs
M	codex-rs/config/src/types.rs
M	codex-rs/connectors/src/metadata_store.rs
M	codex-rs/connectors/src/metadata_store_tests.rs
M	codex-rs/core-plugins/Cargo.toml
M	codex-rs/core-plugins/src/loader.rs
M	codex-rs/core-plugins/src/manager.rs
M	codex-rs/core-plugins/src/manager_tests.rs
M	codex-rs/core-plugins/src/tool_suggest_metadata.rs
M	codex-rs/core-skills/src/config_rules.rs
M	codex-rs/core-skills/src/loader/environment.rs
M	codex-rs/core-skills/src/model.rs
M	codex-rs/core/Cargo.toml
M	codex-rs/core/config.schema.json
M	codex-rs/core/src/audio_preparation.rs
M	codex-rs/core/src/client.rs
M	codex-rs/core/src/codex_delegate.rs
M	codex-rs/core/src/codex_delegate_tests.rs
M	codex-rs/core/src/compact_remote.rs
M	codex-rs/core/src/compact_remote_request.rs
M	codex-rs/core/src/compact_remote_v2.rs
M	codex-rs/core/src/compact_remote_v2_attempt.rs
M	codex-rs/core/src/config/config_loader_tests.rs
M	codex-rs/core/src/config/config_tests.rs
M	codex-rs/core/src/config/edit.rs
M	codex-rs/core/src/config/mod.rs
M	codex-rs/core/src/connectors.rs
D	codex-rs/core/src/context/image_generation_instructions.rs
M	codex-rs/core/src/context/mod.rs
M	codex-rs/core/src/context/world_state/permissions_tests.rs
M	codex-rs/core/src/context_manager/history.rs
M	codex-rs/core/src/context_manager/history_tests.rs
M	codex-rs/core/src/context_manager/mod.rs
M	codex-rs/core/src/context_manager/normalize.rs
M	codex-rs/core/src/exec_policy.rs
M	codex-rs/core/src/exec_policy_tests.rs
M	codex-rs/core/src/guardian/mod.rs
M	codex-rs/core/src/guardian/review.rs
M	codex-rs/core/src/guardian/review_session.rs
M	codex-rs/core/src/guardian/tests.rs
D	codex-rs/core/src/landlock.rs
M	codex-rs/core/src/lib.rs
M	codex-rs/core/src/mcp_tool_call.rs
M	codex-rs/core/src/mcp_tool_call_tests.rs
M	codex-rs/core/src/otel_init.rs
M	codex-rs/core/src/realtime_context.rs
M	codex-rs/core/src/session/config_lock.rs
M	codex-rs/core/src/session/mcp.rs
M	codex-rs/core/src/session/mcp_tests.rs
M	codex-rs/core/src/session/mod.rs
M	codex-rs/core/src/session/session.rs
M	codex-rs/core/src/session/tests.rs
M	codex-rs/core/src/session/turn.rs
M	codex-rs/core/src/shell_snapshot.rs
M	codex-rs/core/src/shell_snapshot_tests.rs
M	codex-rs/core/src/skills.rs
M	codex-rs/core/src/state/service.rs
M	codex-rs/core/src/stream_events_utils.rs
M	codex-rs/core/src/tasks/review.rs
M	codex-rs/core/src/test_support.rs
M	codex-rs/core/src/tools/approvals.rs
M	codex-rs/core/src/tools/approvals_tests.rs
M	codex-rs/core/src/tools/code_mode/mod.rs
M	codex-rs/core/src/tools/events.rs
D	codex-rs/core/src/tools/handlers/agent_jobs.rs
D	codex-rs/core/src/tools/handlers/agent_jobs/report_agent_job_result.rs
D	codex-rs/core/src/tools/handlers/agent_jobs/spawn_agents_on_csv.rs
D	codex-rs/core/src/tools/handlers/agent_jobs_spec.rs
D	codex-rs/core/src/tools/handlers/agent_jobs_spec_tests.rs
D	codex-rs/core/src/tools/handlers/agent_jobs_tests.rs
M	codex-rs/core/src/tools/handlers/extension_tools.rs
M	codex-rs/core/src/tools/handlers/mod.rs
M	codex-rs/core/src/tools/network_approval.rs
M	codex-rs/core/src/tools/network_approval_tests.rs
M	codex-rs/core/src/tools/runtimes/shell/unix_escalation.rs
M	codex-rs/core/src/tools/spec_plan.rs
M	codex-rs/core/src/tools/spec_plan_tests.rs
M	codex-rs/core/src/turn_metadata_tests.rs
M	codex-rs/core/src/unified_exec/process_manager.rs
M	codex-rs/core/src/util.rs
M	codex-rs/core/src/web_search.rs
M	codex-rs/core/src/windows_sandbox.rs
M	codex-rs/core/tests/common/apps_test_server.rs
M	codex-rs/core/tests/common/responses.rs
M	codex-rs/core/tests/common/test_codex.rs
D	codex-rs/core/tests/suite/agent_jobs.rs
M	codex-rs/core/tests/suite/approvals.rs
A	codex-rs/core/tests/suite/audio_truncation.rs
M	codex-rs/core/tests/suite/codex_delegate.rs
M	codex-rs/core/tests/suite/compact_remote.rs
M	codex-rs/core/tests/suite/exec_policy.rs
M	codex-rs/core/tests/suite/hooks.rs
M	codex-rs/core/tests/suite/mcp_auth_elicitation.rs
M	codex-rs/core/tests/suite/mcp_auth_refresh.rs
M	codex-rs/core/tests/suite/mod.rs
M	codex-rs/core/tests/suite/network_approval.rs
M	codex-rs/core/tests/suite/otel.rs
M	codex-rs/core/tests/suite/permissions_messages.rs
M	codex-rs/core/tests/suite/request_permissions.rs
M	codex-rs/core/tests/suite/rmcp_client.rs
M	codex-rs/core/tests/suite/shell_snapshot.rs
M	codex-rs/core/tests/suite/view_image.rs
M	codex-rs/core/tests/suite/workspace_roots.rs
M	codex-rs/exec-server/src/client.rs
M	codex-rs/exec-server/src/environment.rs
M	codex-rs/exec-server/src/local_process.rs
M	codex-rs/exec-server/src/process_sandbox.rs
M	codex-rs/exec-server/src/process_sandbox_tests.rs
M	codex-rs/exec-server/src/resolved_capability.rs
M	codex-rs/exec-server/tests/exec_process.rs
M	codex-rs/exec/src/lib.rs
A	codex-rs/exec/tests/suite/completion_backfill_tests.rs
M	codex-rs/exec/tests/suite/mod.rs
M	codex-rs/exec/tests/suite/sandbox.rs
M	codex-rs/execpolicy/Cargo.toml
M	codex-rs/execpolicy/src/lib.rs
A	codex-rs/execpolicy/src/sandbox_migration.rs
A	codex-rs/execpolicy/src/sandbox_migration_tests.rs
M	codex-rs/ext/extension-api/tests/registry.rs
A	codex-rs/ext/image-generation/src/artifact.rs
M	codex-rs/ext/image-generation/src/lib.rs
M	codex-rs/ext/image-generation/src/tests.rs
M	codex-rs/ext/image-generation/src/tool.rs
M	codex-rs/ext/mcp/tests/hosted_apps_mcp.rs
M	codex-rs/ext/skills/Cargo.toml
M	codex-rs/ext/skills/src/catalog.rs
M	codex-rs/ext/skills/src/provider/executor.rs
M	codex-rs/ext/skills/src/provider/host.rs
M	codex-rs/ext/skills/tests/skills_extension.rs
M	codex-rs/features/src/lib.rs
M	codex-rs/features/src/tests.rs
M	codex-rs/feedback/src/lib.rs
M	codex-rs/hooks/src/declarations.rs
M	codex-rs/hooks/src/engine/command_runner_tests.rs
M	codex-rs/hooks/src/engine/discovery.rs
M	codex-rs/hooks/src/engine/dispatcher.rs
M	codex-rs/hooks/src/engine/mod.rs
M	codex-rs/hooks/src/engine/mod_tests.rs
M	codex-rs/hooks/src/events/common.rs
M	codex-rs/hooks/src/events/compact.rs
M	codex-rs/hooks/src/events/post_tool_use.rs
M	codex-rs/hooks/src/events/pre_tool_use.rs
M	codex-rs/hooks/src/events/session_end_tests.rs
M	codex-rs/hooks/src/events/session_start.rs
M	codex-rs/hooks/src/events/stop.rs
M	codex-rs/hooks/src/events/user_prompt_submit.rs
M	codex-rs/hooks/src/output_spill.rs
M	codex-rs/hooks/src/output_spill_tests.rs
M	codex-rs/http-client/Cargo.toml
M	codex-rs/http-client/src/outbound_proxy.rs
M	codex-rs/http-client/src/outbound_proxy_tests.rs
M	codex-rs/linux-sandbox/src/linux_run_main.rs
M	codex-rs/linux-sandbox/src/linux_run_main_tests.rs
M	codex-rs/mcp-server/src/exec_approval.rs
M	codex-rs/mcp-server/src/patch_approval.rs
M	codex-rs/mcp-server/tests/common/Cargo.toml
M	codex-rs/mcp-server/tests/common/lib.rs
M	codex-rs/memories/write/src/phase2.rs
M	codex-rs/models-manager/models.json
M	codex-rs/models-manager/src/model_info_tests.rs
M	codex-rs/otel/src/metrics/client.rs
M	codex-rs/otel/tests/suite/timing.rs
M	codex-rs/prompts/src/permissions_instructions.rs
M	codex-rs/prompts/src/permissions_instructions_tests.rs
M	codex-rs/protocol/src/approvals.rs
M	codex-rs/protocol/src/items.rs
M	codex-rs/protocol/src/legacy_events.rs
M	codex-rs/protocol/src/models.rs
M	codex-rs/protocol/src/num_format.rs
M	codex-rs/protocol/src/openai_models.rs
M	codex-rs/protocol/src/permissions.rs
M	codex-rs/protocol/src/protocol.rs
M	codex-rs/rollout-trace/src/compaction.rs
M	codex-rs/rollout-trace/src/thread_tests.rs
M	codex-rs/rollout/src/metadata.rs
M	codex-rs/rollout/src/metadata_tests.rs
M	codex-rs/rollout/src/ordinal.rs
M	codex-rs/rollout/src/recorder.rs
M	codex-rs/rollout/src/state_db.rs
M	codex-rs/rollout/src/state_db_tests.rs
M	codex-rs/sandboxing/Cargo.toml
M	codex-rs/sandboxing/src/lib.rs
A	codex-rs/sandboxing/src/spawn.rs
M	codex-rs/skills/Cargo.toml
M	codex-rs/skills/src/lib.rs
A	codex-rs/skills/src/model.rs
A	codex-rs/skills/src/model_tests.rs
A	codex-rs/state/migrations/0041_threads_name.sql
A	codex-rs/state/migrations/0042_drop_agent_jobs.sql
M	codex-rs/state/src/extract.rs
M	codex-rs/state/src/lib.rs
M	codex-rs/state/src/migrations_tests.rs
D	codex-rs/state/src/model/agent_job.rs
M	codex-rs/state/src/model/mod.rs
M	codex-rs/state/src/model/thread_metadata.rs
M	codex-rs/state/src/runtime.rs
D	codex-rs/state/src/runtime/agent_jobs.rs
M	codex-rs/state/src/runtime/backfill.rs
M	codex-rs/state/src/runtime/logs.rs
M	codex-rs/state/src/runtime/memories.rs
M	codex-rs/state/src/runtime/remote_control.rs
M	codex-rs/state/src/runtime/test_support.rs
M	codex-rs/state/src/runtime/threads.rs
M	codex-rs/state/src/sqlite.rs
M	codex-rs/thread-manager-sample/src/main.rs
M	codex-rs/thread-store/src/local/helpers.rs
M	codex-rs/thread-store/src/local/list_threads.rs
M	codex-rs/thread-store/src/local/mod.rs
M	codex-rs/thread-store/src/local/read_thread.rs
A	codex-rs/thread-store/src/local/rollout_lineage.rs
A	codex-rs/thread-store/src/local/rollout_lineage_tests.rs
M	codex-rs/thread-store/src/local/search_threads.rs
M	codex-rs/thread-store/src/local/update_thread_metadata.rs
M	codex-rs/thread-store/src/thread_metadata_sync.rs
M	codex-rs/tui/Cargo.toml
M	codex-rs/tui/src/app.rs
M	codex-rs/tui/src/app/event_dispatch.rs
M	codex-rs/tui/src/app/loaded_threads.rs
M	codex-rs/tui/src/app/safety_buffering.rs
M	codex-rs/tui/src/app/session_lifecycle.rs
M	codex-rs/tui/src/app/tests.rs
M	codex-rs/tui/src/app/tests/safety_buffering.rs
A	codex-rs/tui/src/app/tests/session_lifecycle_requests.rs
M	codex-rs/tui/src/app/thread_events.rs
M	codex-rs/tui/src/app/thread_routing.rs
M	codex-rs/tui/src/app_server_session.rs
M	codex-rs/tui/src/bottom_pane/chat_composer.rs
A	codex-rs/tui/src/bottom_pane/chat_composer_effort_tests.rs
A	codex-rs/tui/src/bottom_pane/effort_ignition.rs
A	codex-rs/tui/src/bottom_pane/effort_ignition_styles.rs
A	codex-rs/tui/src/bottom_pane/effort_ignition_tests.rs
A	codex-rs/tui/src/bottom_pane/effort_status_line.rs
A	codex-rs/tui/src/bottom_pane/effort_status_line_tests.rs
M	codex-rs/tui/src/bottom_pane/hooks_browser_view.rs
M	codex-rs/tui/src/bottom_pane/mcp_server_elicitation.rs
M	codex-rs/tui/src/bottom_pane/mentions_v2/search_catalog.rs
M	codex-rs/tui/src/bottom_pane/mod.rs
A	codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__chat_composer__effort_tests__effort_transition_keeps_the_full_footer_row.snap
A	codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__chat_composer__tests__ultra_accent_upgrades_prompt_glyph.snap
A	codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__effort_ignition__tests__effort_ignition_animation_gallery.snap
A	codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__effort_status_line__tests__effort_status_line_transition_frames.snap
A	codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__hooks_browser_view__tests__hooks_browser_additional_context_limit.snap
M	codex-rs/tui/src/chatwidget.rs
M	codex-rs/tui/src/chatwidget/command_lifecycle.rs
M	codex-rs/tui/src/chatwidget/input_restore.rs
M	codex-rs/tui/src/chatwidget/input_submission.rs
M	codex-rs/tui/src/chatwidget/mcp_startup.rs
M	codex-rs/tui/src/chatwidget/rendering.rs
M	codex-rs/tui/src/chatwidget/session_flow.rs
M	codex-rs/tui/src/chatwidget/settings.rs
M	codex-rs/tui/src/chatwidget/skills.rs
A	codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__guardian_goal_continuation_drops_stale_reviews.snap
A	codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__live_app_server_command_output_delta_active.snap
A	codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__live_app_server_command_output_delta_interrupted.snap
A	codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__live_app_server_command_output_delta_transcript.snap
M	codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__post_tool_use_hook_events_render_snapshot.snap
M	codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__pre_tool_use_hook_events_render_snapshot.snap
M	codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__session_start_hook_events_render_snapshot.snap
M	codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__user_prompt_submit_app_server_hook_notifications_render_snapshot.snap
M	codex-rs/tui/src/chatwidget/status_state.rs
M	codex-rs/tui/src/chatwidget/streaming.rs
M	codex-rs/tui/src/chatwidget/tests.rs
M	codex-rs/tui/src/chatwidget/tests/app_server.rs
M	codex-rs/tui/src/chatwidget/tests/composer_submission.rs
M	codex-rs/tui/src/chatwidget/tests/guardian.rs
M	codex-rs/tui/src/chatwidget/tests/helpers.rs
M	codex-rs/tui/src/chatwidget/tests/history_replay.rs
M	codex-rs/tui/src/chatwidget/tool_lifecycle.rs
M	codex-rs/tui/src/chatwidget/tool_requests.rs
M	codex-rs/tui/src/chatwidget/turn_runtime.rs
M	codex-rs/tui/src/custom_terminal.rs
M	codex-rs/tui/src/debug_config.rs
M	codex-rs/tui/src/diff_render.rs
A	codex-rs/tui/src/exec_cell/live_output.rs
A	codex-rs/tui/src/exec_cell/live_output_tests.rs
M	codex-rs/tui/src/exec_cell/mod.rs
M	codex-rs/tui/src/exec_cell/model.rs
M	codex-rs/tui/src/exec_cell/render.rs
A	codex-rs/tui/src/exec_cell/snapshots/codex_tui__exec_cell__render__tests__truncated_live_output_preview_and_transcript.snap
M	codex-rs/tui/src/history_cell/base.rs
M	codex-rs/tui/src/history_cell/hook_cell.rs
A	codex-rs/tui/src/history_cell/markdown_render_cache.rs
M	codex-rs/tui/src/history_cell/messages.rs
A	codex-rs/tui/src/history_cell/messages_tests.rs
M	codex-rs/tui/src/history_cell/mod.rs
M	codex-rs/tui/src/history_cell/notices.rs
M	codex-rs/tui/src/history_cell/patches.rs
M	codex-rs/tui/src/history_cell/plans.rs
A	codex-rs/tui/src/history_cell/plans_tests.rs
M	codex-rs/tui/src/history_cell/session.rs
M	codex-rs/tui/src/history_cell/tests.rs
M	codex-rs/tui/src/inline_visualization_tests.rs
M	codex-rs/tui/src/live_wrap.rs
M	codex-rs/tui/src/markdown_render.rs
M	codex-rs/tui/src/pager_overlay.rs
M	codex-rs/tui/src/render/highlight.rs
M	codex-rs/tui/src/render/line_utils.rs
M	codex-rs/tui/src/render/renderable.rs
M	codex-rs/tui/src/render/renderable_tests.rs
M	codex-rs/tui/src/skills_helpers.rs
M	codex-rs/tui/src/snapshots/codex_tui__debug_config__tests__debug_config_output_lists_agents_fields.snap
A	codex-rs/tui/src/snapshots/codex_tui__inline_visualization__tests__transcript_overlay_visualization_becomes_available.snap
M	codex-rs/tui/src/startup_hooks_review.rs
A	codex-rs/tui/src/status/snapshots/codex_tui__status__tests__transcript_overlay_status_rate_limit_refresh.snap
M	codex-rs/tui/src/status/tests.rs
M	codex-rs/tui/src/streaming/render.rs
M	codex-rs/tui/src/streaming/render_tests.rs
M	codex-rs/tui/src/style.rs
M	codex-rs/tui/src/terminal_hyperlinks.rs
M	codex-rs/tui/src/terminal_palette.rs
M	codex-rs/tui/src/update_prompt.rs
M	codex-rs/tui/src/wrapping.rs
M	codex-rs/utils/cli/src/config_override.rs
M	codex-rs/utils/output-truncation/src/lib.rs
M	codex-rs/utils/output-truncation/src/truncate_tests.rs
M	codex-rs/utils/pty/src/pipe.rs
M	codex-rs/utils/pty/src/pty.rs
M	codex-rs/utils/pty/src/tests.rs
M	codex-rs/utils/pty/src/windows_tests.rs
M	codex-rs/utils/sandbox-summary/Cargo.toml
D	codex-rs/utils/sandbox-summary/src/config_summary.rs
M	codex-rs/utils/sandbox-summary/src/lib.rs
M	codex-rs/websocket-client/src/dialer.rs
M	codex-rs/websocket-client/src/dialer_tests.rs
M	codex-rs/windows-sandbox-rs/src/acl.rs
M	codex-rs/windows-sandbox-rs/src/bin/setup_main/win.rs
M	codex-rs/windows-sandbox-rs/src/lib.rs
M	scripts/codex_package/rg
```

</details>

<details><summary>OpenCode: 84 changed paths</summary>

```text
M	bun.lock
M	nix/hashes.json
A	packages/app/e2e/regression/prompt-input-v2-command-draft.spec.ts
M	packages/app/e2e/regression/prompt-thinking-level.spec.ts
M	packages/app/package.json
M	packages/app/src/addons/serialize.test.ts
M	packages/app/src/addons/serialize.ts
M	packages/app/src/components/help-button.tsx
M	packages/app/src/components/prompt-input-v2.tsx
M	packages/app/src/components/prompt-input/build-request-parts.ts
M	packages/app/src/components/settings-v2/settings-v2.css
M	packages/app/src/components/terminal.tsx
M	packages/app/src/components/titlebar.tsx
M	packages/app/src/context/notification.tsx
M	packages/app/src/context/server-session.test.ts
M	packages/app/src/context/server-session.ts
M	packages/app/src/pages/layout-new.tsx
M	packages/app/src/pages/layout.tsx
M	packages/app/src/pages/session.tsx
M	packages/app/src/pages/session/file-tabs.tsx
M	packages/app/src/pages/session/session-side-panel.tsx
M	packages/app/src/pages/session/v2/session-file-browser-tab.tsx
M	packages/cli/package.json
M	packages/codemode/package.json
M	packages/console/app/package.json
M	packages/console/app/src/app.tsx
M	packages/console/core/package.json
M	packages/console/function/package.json
M	packages/console/mail/package.json
M	packages/console/support/package.json
M	packages/core/package.json
M	packages/core/src/pty/pty.node.ts
M	packages/desktop/package.json
M	packages/effect-drizzle-sqlite/package.json
M	packages/effect-sqlite-node/package.json
M	packages/enterprise/package.json
M	packages/function/package.json
M	packages/http-recorder/package.json
M	packages/llm/package.json
M	packages/llm/src/provider-error.ts
M	packages/llm/test/provider-error.test.ts
M	packages/opencode/package.json
M	packages/opencode/src/cli/cmd/import.ts
M	packages/opencode/src/provider/transform.ts
M	packages/opencode/src/session/prompt/meta.txt
M	packages/opencode/test/cli/import.test.ts
M	packages/opencode/test/provider/transform.test.ts
M	packages/plugin/package.json
M	packages/sdk/js/package.json
M	packages/server/package.json
M	packages/session-ui/package.json
M	packages/session-ui/src/v2/components/prompt-input/index.tsx
M	packages/session-ui/src/v2/components/prompt-input/interaction.ts
M	packages/session-ui/src/v2/components/prompt-input/machine.test.ts
M	packages/session-ui/src/v2/components/prompt-input/machine.ts
M	packages/session-ui/src/v2/components/prompt-input/store.test.ts
M	packages/session-ui/src/v2/components/prompt-input/store.ts
M	packages/session-ui/src/v2/components/session-review-v2.css
M	packages/slack/package.json
M	packages/stats/app/package.json
M	packages/stats/core/package.json
M	packages/stats/server/package.json
M	packages/tui/package.json
M	packages/ui/package.json
M	packages/web/package.json
M	packages/web/src/content/docs/ar/zen.mdx
M	packages/web/src/content/docs/bs/zen.mdx
M	packages/web/src/content/docs/da/zen.mdx
M	packages/web/src/content/docs/de/zen.mdx
M	packages/web/src/content/docs/es/zen.mdx
M	packages/web/src/content/docs/fr/zen.mdx
M	packages/web/src/content/docs/it/zen.mdx
M	packages/web/src/content/docs/ja/zen.mdx
M	packages/web/src/content/docs/ko/zen.mdx
M	packages/web/src/content/docs/nb/zen.mdx
M	packages/web/src/content/docs/pl/zen.mdx
M	packages/web/src/content/docs/pt-br/zen.mdx
M	packages/web/src/content/docs/ru/zen.mdx
M	packages/web/src/content/docs/th/zen.mdx
M	packages/web/src/content/docs/tr/zen.mdx
M	packages/web/src/content/docs/zen.mdx
M	packages/web/src/content/docs/zh-cn/zen.mdx
M	packages/web/src/content/docs/zh-tw/zen.mdx
M	sdks/vscode/package.json
```

</details>

<details><summary>Kimi Code: 232 changed paths</summary>

```text
D	.changeset/fix-afk-naming.md
D	.changeset/fix-permission-copy-cli-acp.md
D	.changeset/fix-permission-copy-web.md
D	.changeset/fix-permission-mode-copy.md
D	.changeset/fix-yolo-replay-copy.md
D	.changeset/multi-server-default-shared-home.md
D	.changeset/unify-hostfs-stat-semantics.md
M	AGENTS.md
M	apps/kimi-code/CHANGELOG.md
M	apps/kimi-code/package.json
M	apps/kimi-code/src/cli/sub/web/deprecated-server.ts
M	apps/kimi-code/src/cli/sub/web/index.ts
D	apps/kimi-code/src/cli/sub/web/kill.ts
A	apps/kimi-code/src/cli/sub/web/legacy-kill.ts
D	apps/kimi-code/src/cli/sub/web/ps.ts
M	apps/kimi-code/src/cli/sub/web/run.ts
M	apps/kimi-code/src/cli/sub/web/shared.ts
M	apps/kimi-code/src/tui/banner/banner-provider.ts
M	apps/kimi-code/src/tui/commands/config.ts
M	apps/kimi-code/src/tui/commands/provider.ts
M	apps/kimi-code/src/tui/commands/registry.ts
M	apps/kimi-code/src/tui/commands/web.ts
M	apps/kimi-code/src/tui/utils/thinking-config.ts
M	apps/kimi-code/test/cli/web/web.test.ts
M	apps/kimi-code/test/tui/banner/banner-provider.test.ts
M	apps/kimi-code/test/tui/commands/web.test.ts
M	apps/kimi-code/test/tui/kimi-tui-message-flow.test.ts
M	apps/kimi-code/test/tui/utils/thinking-config.test.ts
M	apps/kimi-inspect/package.json
M	apps/kimi-inspect/src/App.tsx
M	apps/kimi-inspect/src/channel/channel.test.ts
M	apps/kimi-inspect/src/channel/channel.ts
M	apps/kimi-inspect/src/channel/channels.ts
M	apps/kimi-inspect/src/channel/client.ts
M	apps/kimi-inspect/src/channel/errors.ts
M	apps/kimi-inspect/src/channel/index.ts
M	apps/kimi-inspect/src/channel/proxyChannel.ts
D	apps/kimi-inspect/src/channel/wsChannel.ts
A	apps/kimi-inspect/src/channel/wsLike.ts
D	apps/kimi-inspect/src/channel/wsSocket.ts
M	apps/kimi-inspect/src/components/ChatView.tsx
M	apps/kimi-inspect/src/components/Inspector.tsx
M	apps/kimi-inspect/src/components/Sidebar.tsx
M	apps/kimi-inspect/src/connection.tsx
D	apps/kimi-inspect/src/live.tsx
M	apps/kimi-inspect/src/panels.ts
A	apps/kimi-inspect/src/transcript/api.ts
A	apps/kimi-inspect/src/transcript/store.ts
A	apps/kimi-inspect/src/transcript/transcript.test.ts
A	apps/kimi-inspect/src/transcript/ws.ts
M	apps/kimi-web/src/api/daemon/agentEventProjector.ts
M	apps/kimi-web/src/api/types.ts
M	apps/kimi-web/src/components/chat/Composer.vue
M	apps/kimi-web/src/components/mobile/MobileSettingsSheet.vue
M	apps/kimi-web/src/composables/client/useModelProviderState.ts
M	apps/kimi-web/src/composables/client/useSideChat.ts
M	apps/kimi-web/src/composables/client/useWorkspaceState.ts
M	apps/kimi-web/src/composables/useKimiWebClient.ts
M	apps/kimi-web/src/i18n/locales/en/status.ts
M	apps/kimi-web/src/i18n/locales/zh/status.ts
M	apps/kimi-web/src/lib/modelThinking.test.ts
M	apps/kimi-web/src/lib/modelThinking.ts
M	apps/kimi-web/src/lib/storage.ts
M	apps/kimi-web/test/side-chat.test.ts
M	apps/kimi-web/test/workspace-state.test.ts
M	apps/vscode/CHANGELOG.md
M	apps/vscode/package.json
M	apps/vscode/shared/bridge.ts
M	apps/vscode/shared/types.ts
M	apps/vscode/src/handlers/config.handler.ts
M	apps/vscode/test/bridge-handler.test.ts
M	apps/vscode/test/settings-store.test.ts
M	apps/vscode/webview-ui/src/components/ThinkingButton.tsx
M	apps/vscode/webview-ui/src/components/inputarea/InputArea.tsx
M	apps/vscode/webview-ui/src/stores/settings.store.ts
M	docs/en/reference/kimi-command.md
M	docs/en/reference/slash-commands.md
M	docs/en/release-notes/changelog.md
M	docs/zh/reference/kimi-command.md
M	docs/zh/reference/slash-commands.md
M	docs/zh/release-notes/changelog.md
M	flake.nix
M	packages/acp-adapter/src/server.ts
M	packages/acp-adapter/test/auth-gate.test.ts
M	packages/agent-core-v2/CHANGELOG.md
M	packages/agent-core-v2/package.json
M	packages/agent-core-v2/src/agent/loop/loopService.ts
M	packages/agent-core-v2/src/agent/loop/turnEvents.ts
M	packages/agent-core-v2/src/agent/questionTools/tools/ask-user.ts
M	packages/agent-core-v2/src/agent/rpc/rpcService.ts
M	packages/agent-core-v2/src/agent/shellCommand/shellCommandService.ts
M	packages/agent-core-v2/src/app/config/configService.ts
A	packages/agent-core-v2/src/app/config/migrations.ts
M	packages/agent-core-v2/src/app/sessionLegacy/sessionLegacyService.ts
M	packages/agent-core-v2/src/session/agentLifecycle/agentLifecycle.ts
M	packages/agent-core-v2/src/session/agentLifecycle/agentLifecycleService.ts
M	packages/agent-core-v2/src/session/question/question.ts
M	packages/agent-core-v2/src/session/question/questionService.ts
M	packages/agent-core-v2/test/agent/loop/loop.test.ts
M	packages/agent-core-v2/test/agent/plan/plan.test.ts
M	packages/agent-core-v2/test/agent/questionTools/tools/ask-user.test.ts
A	packages/agent-core-v2/test/agent/rpc/setPermission.test.ts
M	packages/agent-core-v2/test/agent/shellCommand/shellCommand.test.ts
M	packages/agent-core-v2/test/app/config/config.test.ts
M	packages/agent-core-v2/test/app/gateway/gateway.test.ts
M	packages/agent-core-v2/test/app/sessionExport/sessionExport.test.ts
M	packages/agent-core-v2/test/app/sessionLegacy/sessionLegacy.test.ts
M	packages/agent-core-v2/test/app/sessionLifecycle/sessionLifecycle.test.ts
M	packages/agent-core-v2/test/harness/agent.ts
M	packages/agent-core-v2/test/session/agentLifecycle/agentLifecycle.test.ts
M	packages/agent-core-v2/test/session/question/question.test.ts
M	packages/agent-core-v2/test/session/swarm/sessionSwarm.test.ts
M	packages/agent-core-v2/test/session/todo/sessionTodo.test.ts
M	packages/agent-core-v2/test/session/workspaceCommand/workspaceCommand.test.ts
M	packages/agent-core-v2/test/tool/tool.test.ts
M	packages/agent-core/src/config/index.ts
A	packages/agent-core/src/config/migrations.ts
M	packages/agent-core/src/rpc/core-impl.ts
M	packages/agent-core/test/config/configs.test.ts
M	packages/kap-server/CHANGELOG.md
M	packages/kap-server/package.json
M	packages/kap-server/src/contract.ts
M	packages/kap-server/src/middleware/auth.ts
M	packages/kap-server/src/protocol/events-zod.ts
M	packages/kap-server/src/routes/auth.ts
M	packages/kap-server/src/routes/connections.ts
M	packages/kap-server/src/routes/registerApiV1Routes.ts
M	packages/kap-server/src/routes/tools.ts
A	packages/kap-server/src/routes/transcript.ts
A	packages/kap-server/src/services/transcript/coreBinding.ts
A	packages/kap-server/src/services/transcript/coreEventMap.ts
A	packages/kap-server/src/services/transcript/index.ts
A	packages/kap-server/src/services/transcript/transcriptService.ts
M	packages/kap-server/src/start.ts
M	packages/kap-server/src/transport/channel.ts
M	packages/kap-server/src/transport/channelRegistry.ts
M	packages/kap-server/src/transport/dispatcher.ts
M	packages/kap-server/src/transport/errors.ts
M	packages/kap-server/src/transport/mainAgent.ts
M	packages/kap-server/src/transport/registerDebugRoutes.ts
R071	packages/kap-server/src/transport/registerRpcRoutes.ts	packages/kap-server/src/transport/serviceDispatcherRoutes.ts
M	packages/kap-server/src/transport/ws/connectionRegistry.ts
D	packages/kap-server/src/transport/ws/eventMap.ts
D	packages/kap-server/src/transport/ws/registerWs.ts
M	packages/kap-server/src/transport/ws/v1/events.ts
M	packages/kap-server/src/transport/ws/v1/registerWsV1.ts
M	packages/kap-server/src/transport/ws/v1/sessionEventBroadcaster.ts
M	packages/kap-server/src/transport/ws/v1/wsConnectionV1.ts
D	packages/kap-server/src/transport/ws/wsClient.ts
D	packages/kap-server/src/transport/ws/wsConnection.ts
D	packages/kap-server/src/transport/ws/wsProtocol.ts
M	packages/kap-server/test/__snapshots__/apiSurface.snapshot.test.ts.snap
M	packages/kap-server/test/apiSurface.snapshot.test.ts
M	packages/kap-server/test/connections.test.ts
M	packages/kap-server/test/disableAuth.e2e.test.ts
D	packages/kap-server/test/eventMap.test.ts
M	packages/kap-server/test/rpc.test.ts
A	packages/kap-server/test/services/transcript.test.ts
M	packages/kap-server/test/sessionEventBroadcaster.test.ts
M	packages/kap-server/test/sessions.test.ts
A	packages/kap-server/test/transcript.test.ts
M	packages/kap-server/test/transport-errors.test.ts
D	packages/kap-server/test/ws.test.ts
M	packages/kap-server/test/wsConnectionV1.test.ts
M	packages/kap-server/test/wsHostOrigin.test.ts
M	packages/kap-server/test/wsUpgradeAuth.test.ts
M	packages/klient/AGENTS.md
M	packages/klient/README.md
M	packages/klient/examples/basic.ts
M	packages/klient/examples/context-usage.ts
M	packages/klient/examples/smoke.ts
M	packages/klient/package.json
M	packages/klient/src/contract/agent/events.ts
M	packages/klient/src/core/klient.ts
M	packages/klient/src/index.ts
D	packages/klient/src/transports/http/channel.ts
D	packages/klient/src/transports/http/eventBridge.ts
D	packages/klient/src/transports/http/index.ts
M	packages/klient/src/transports/ipc/codec.ts
D	packages/klient/src/transports/ws/wsSocket.ts
D	packages/klient/test/e2e/dual/01-prompt.test.ts
D	packages/klient/test/e2e/dual/02-approval.test.ts
D	packages/klient/test/e2e/dual/05-workspace.test.ts
D	packages/klient/test/e2e/dual/06-catalog.test.ts
D	packages/klient/test/e2e/dual/07-children.test.ts
D	packages/klient/test/e2e/dual/08-pending.test.ts
M	packages/klient/test/e2e/harness/index.ts
M	packages/klient/test/e2e/legacy/image-file-prompts.test.ts
M	packages/klient/test/e2e/legacy/terminal.test.ts
D	packages/klient/test/e2e/v2/channelRegistry.test.ts
D	packages/klient/test/e2e/v2/helpers/token.ts
D	packages/klient/test/e2e/v2/smoke.test.ts
D	packages/klient/test/e2e/v2/token.test.ts
M	packages/klient/test/helpers/conformance.ts
D	packages/klient/test/helpers/dual.ts
D	packages/klient/test/http-conformance.test.ts
D	packages/klient/test/http.test.ts
D	packages/klient/test/wsSocket.test.ts
M	packages/klient/tsdown.config.ts
M	packages/node-sdk/test/session-event-types.test.ts
M	packages/protocol/src/__tests__/events.test.ts
M	packages/protocol/src/__tests__/snapshot.test.ts
M	packages/protocol/src/events.ts
A	packages/transcript/CHANGELOG.md
A	packages/transcript/package.json
A	packages/transcript/src/granularity/filterOps.ts
A	packages/transcript/src/granularity/grade.ts
A	packages/transcript/src/history/groupTurns.ts
A	packages/transcript/src/index.ts
A	packages/transcript/src/model/attachment.ts
A	packages/transcript/src/model/frame.ts
A	packages/transcript/src/model/ids.ts
A	packages/transcript/src/model/interaction.ts
A	packages/transcript/src/model/item.ts
A	packages/transcript/src/model/meta.ts
A	packages/transcript/src/model/task.ts
A	packages/transcript/src/model/todo.ts
A	packages/transcript/src/model/turn.ts
A	packages/transcript/src/ops/apply.ts
A	packages/transcript/src/ops/operation.ts
A	packages/transcript/src/pagination/paginate.ts
A	packages/transcript/src/store/agentTranscript.ts
A	packages/transcript/src/store/transcriptStore.ts
A	packages/transcript/src/view/registry.ts
A	packages/transcript/src/wire/events.ts
A	packages/transcript/src/wire/schema.ts
A	packages/transcript/test/layers.test.ts
A	packages/transcript/test/store.test.ts
A	packages/transcript/tsconfig.json
A	packages/transcript/tsdown.config.ts
A	packages/transcript/vitest.config.ts
M	pnpm-lock.yaml
```

</details>
