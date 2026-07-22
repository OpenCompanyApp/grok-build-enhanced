# Upstream refresh parity ledger — 2026-07-21, third pass

This ledger records the exact source pins, complete advanced-source commit and changed-path inventories, observable-behavior decisions, local implementation evidence, and the exhaustive raw-tree classification for the third refresh performed on 2026-07-21. It continues [`docs/upstream-refresh-2026-07-21-r2.md`](upstream-refresh-2026-07-21-r2.md). Provider-reference history is evidence, not content-merged application history, and ignored `inspiration/` checkouts are not part of the candidate tree.

There are **zero temporary deferrals** and **zero unclassified behaviors**. Grok behavior on preserved surfaces was adopted through the existing Enhanced architecture; provider credentials, endpoints, catalogs, usage, retries, hosted tools, and logout state remain explicit and isolated.

## Immutable boundary

- Previous clean unpublished candidate: `72cdffa9f5391e3cb65f3f5d20cb90988ec66b9a`, tree `a107408bdb301cd7fcaef24fd614813c38f5d212`.
- Isolated branch/worktree: `refresh/upstream-20260721-r3` / `../grok-build-enhanced-refresh-20260721-r3`.
- Fetch-pin commit: `067698179a313a05e6f811f3c7b1d7187d4f05b2`, tree `2fcf2b8ad97d0639a496a74b51f3252353fe096e`.
- Check timestamp: `2026-07-21T18:12:13Z`.
- Acknowledgement source ID: `grok-build-upstream`.
- Previous reviewed Grok commit/tree: `a881e6703f46b01d8c7d4a5437683546df30449d` / `e3a013ffc66a6dd77ec1c35dfe261741bcf84928`.
- Pinned Grok commit/tree: `3af4d5d39897855bdcc74f23e690024a5dc05573` / `e595174931be9bfb490aacf149e2c9cc0ca0ebba`.
- Pinned Grok target parents: `a881e6703f46b01d8c7d4a5437683546df30449d`.
- The protected main-worktree `AGENTS.md` SHA-256 at refresh start was `e5467bd0ffd740f690390090a2ff416958e7f3ff94fb849c546d80d217db1639`.

## All 12 immutable source pins

| Source | Previous reviewed commit / tree | Pinned commit / tree | Range | Changed paths | Ancestor |
| --- | --- | --- | ---: | ---: | --- |
| Grok Build | `a881e6703f46b01d8c7d4a5437683546df30449d` / `e3a013ffc66a6dd77ec1c35dfe261741bcf84928` | `3af4d5d39897855bdcc74f23e690024a5dc05573` / `e595174931be9bfb490aacf149e2c9cc0ca0ebba` | 1 commit | 556 name-status records; 557 raw tree paths | yes |
| OpenAI Codex | `b9800de4867e500a92add3cde795cf4790306d0f` / `beacdfb5b2d0b413b5b255ac61b96b9aeb79e947` | `51200321eb7b862a29ffceaba8b19db1934a9b38` / `f776ca65baecd8157602572803d41ec92be9d7ab` | 20 commits | 100 | yes |
| OpenCode | `cb562b2c6289c2eee707078f9ab644cbe1d3d8a9` / `4a6c7f446d849918b1a4259e2fcf6d0dba581dcb` | `0317531906d3f3bb01cf33c16319870cfde9170c` / `e9344f8affc0b7f5f0537cb6c3ac09852d05f53a` | 5 commits | 21 | yes |
| OpenCode Codex auth | `bec2ad69b252ef4ad7dd33b9532ff8b4fdb6d016` / `1da59bae7069563b2817143567b57c78e5758300` | same | 0 | 0 | unchanged |
| Warp themes | `b385044250f1ed3c9379ab34a8fe82f02fdffaa4` / `c678999ab9ef62cfba1d70822b117db727d06f8a` | same | 0 | 0 | unchanged |
| Kimi Code | `a3699dd6aa7b41efd3129a117007d195282379fd` / `5e219da00781cafc0caf475c2bd708cfe09179f3` | `b5efba7abcaf4041f81ec520097a61e6546e8c50` / `26080b7faeed49746ea11bec5a92d0fa23a4e189` | 6 commits | 101 | yes |
| Kimi CLI | `4a550effdfcb29a25a5d325bf935296cc50cd417` / `96cf7e65f10325b717db982effd32b3cec55a331` | same | 0 | 0 | unchanged |
| Z.AI Python SDK | `ca5109c0aa9bf173839be391b4b14aeadf9a9bf9` / `1fa4d15e067bf48dbb6dd69ca3b2e4143d5f242a` | same | 0 | 0 | unchanged |
| Z.AI coding plugins | `0446d0bb0bc537d97d3ab3664c4b8b9c4a0e1254` / `efea84479dc67bc4af7d2c3b59b4aca8f5332899` | same | 0 | 0 | unchanged |
| GLM-5 | `436efa09bc868a6922e307624189e7018406beb9` / `8ac85a6098dc83ebd539a9093442e8192fbf052c` | same | 0 | 0 | unchanged |
| CodexBar | `cc8da27cec92029a6435bfee4a703a719290234e` / `e41036396d949aed3c579a52af74b1b8bab780f6` | same | 0 | 0 | unchanged |
| Z.AI usage browser | `54cd1f33a703c417f2492ee1f21f22b3633a43c4` / `08b00849b96c5883a265f4d4d43e2836d01cdd9d` | same | 0 | 0 | unchanged |

The eight unchanged references were fetched and attested at the exact reviewed commit/tree pairs above. They introduce no new provider contract, runtime provider, login flow, credential surface, theme corpus, or legal obligation in this pass.

## Grok observable-behavior inventory

The rows below inventory behavior separately from source architecture. Every applicable preserved-surface behavior is adopted; implementation difficulty is not used as an exclusion.

| Evidence ID | Upstream behavior | Outcome and local implementation | Focused evidence |
| --- | --- | --- | --- |
| `GB-3AF4-SESSIONS-ACP` | Eager per-frame persistence; relocated/mirrored session discovery, import, resume, ACP replay, durable terminal reconciliation, and distinct refusal completion. | **adopt.** Session storage, relocation, replay, queue adoption, and viewer completion are ported while provider bindings remain session-scoped. A refusal completes exactly once and remains distinguishable as ACP `StopReason::Refusal`, including under the leader. | `xai-chat-state` actor tests; shell JSONL/relocation tests; two ignored refusal ACP scenarios; pager session lifecycle/load/foreign tests. |
| `GB-3AF4-HOOKS` | Stop hooks may return feedback and continue the agent rather than ending the turn; command and HTTP handlers retain typed dispatch and bounded defaults. | **adopt.** Stop-hook feedback re-enters the same session loop, folds lifecycle output into the correct marker, and does not cross provider/session state. Handler dispatch is exhaustive over typed command/HTTP variants, with a 600-second Stop/SubagentStop default rather than an accidental short generic timeout. | Hook dispatcher tests; four ignored stop-hook integration scenarios; stop-gate and turn-completion tests. |
| `GB-3AF4-AUTO-FEEDBACK` | Auto mode reports classifier denials to the model and escalates only after repeated blocks; feedback includes author identity when supplied. | **adopt.** The loop preserves denial feedback, retry/escalation state, and attributed feedback. | Workspace auto-mode tests; shell feedback/goal tests. |
| `GB-3AF4-SCHEDULER-QUEUE` | Wake admission/reservations/suppression, durable scheduler deletion acknowledgement, and queue combination prevent duplicate or overlapping work. | **adopt.** Scheduler persistence revisions, `AutoWakeDeliveredIds`, task completion reminders, and structured `QueueDrain` effects/page-flip notifications are integrated. | Scheduler actor/create/delete tests; prompt-queue tests; wake-suppression and queue-adoption tests. |
| `GB-3AF4-TURN-QUEUE` | Combined queued prompts, page-flip-on-send, markerless wake completions, parked-marker folding, and exact running-prompt adoption. | **adopt.** The pager retains the structured drain result, page-flip entry, combined text, stable prompt IDs, and mismatched-stash rules. | Pager queue/turn/prompt tests; page-flip, edit-merge, endline-wakeup PTYs. |
| `GB-3AF4-WORKFLOWS` | Workflows have durable execution state, tools, watcher updates, dashboard/tasks integration, overlays, and slash controls. | **adopt.** Workflow definitions, runtime, tools, notifications, watcher coalescing, and TUI surfaces are integrated without replacing ACP or permissions. | `xai-workflow` tests; workflow tool/session tests; pager workflow rendering tests. |
| `GB-3AF4-EXTERNAL-EDITOR` | Minimal Ctrl+G and `/edit-prompt` edit the current draft externally without sending it. | **adopt.** Typed editor launch/restore preserves draft state; fullscreen Ctrl+G retains tasks-pane behavior. | External-editor dispatch tests and minimal external-editor PTY. |
| `GB-3AF4-DOCTOR` | Early `grok doctor`, `/doctor`, JSON reports, terminal/tmux/clipboard/keyboard probes, and guarded SSH-wrap fix. | **adopt.** Doctor dispatch occurs before TUI initialization and fix operations remain explicit/confirmed. | Doctor module/unit tests and `doctor_early_dispatch`. |
| `GB-3AF4-USAGE` | Session-first `/usage` shows token counts/cost and may chain provider billing; consumer management remains provider-aware. | **adopt.** Session usage is always rendered, then xAI/Codex/Kimi/Z.AI usage is selected explicitly; Custom returns an empty managed surface and never falls through to xAI. | Pager billing/usage tests; shell usage/billing tests including Custom negative coverage. |
| `GB-3AF4-CUSTOM-AUTH` | Shared `[model_providers.<id>]` settings and rotating command-provided credentials support custom gateways. | **adopt with isolation.** Custom provider settings and credential helpers are explicit; xAI, Codex, Kimi, Z.AI, and Custom auth/catalog/usage/logout paths cannot fall through to each other. | Shell model-provider/config/auth tests and sensitive-header redaction tests. |
| `GB-3AF4-MAX-EFFORT` | Catalog-advertised `max` reasoning effort is distinct and above `xhigh`. | **adopt.** ACP model state and provider adapters preserve exact advertised levels and session selection. | Effort-level pager, ACP, Kimi, and sampling tests. |
| `GB-3AF4-SKILLS` | Markdown under skills is returned in full by `read_file`; builtin skill packaging changes do not truncate skill instructions. | **adopt.** Skill-path detection bypasses ordinary Markdown truncation while preserving tool output limits elsewhere. | `read_file` skill tests and shell skill discovery tests. |
| `GB-3AF4-VOICE` | Voice distinguishes microphone silence/permission failure from no detected speech. | **adopt.** Capture/probe/pipeline events retain the diagnostic distinction through pager notices. | Voice PCM/probe and pager voice dispatch tests. |
| `GB-3AF4-WORKTREE` | Managed worktrees support automatic GC and safe discovery/checkout. | **adopt with stronger boundary.** Auto-GC is ported; symlink escapes outside managed roots are rejected. | Fast-worktree auto-GC/discovery tests and managed-worktree symlink regression. |
| `GB-3AF4-PERMISSIONS` | Execution-risk and shell-access decisions remain explicit across auto mode and workspace operations. | **adopt.** Risk classification and approval remain inside the existing Grok permission model; provider adapters gain no bypass. | Workspace permission/exec-risk/shell-access tests. |
| `GB-3AF4-CONFIG` | Managed text/config writes are transactional, validated, source-aware, and reload safely. | **adopt.** Managed-text format/source/transaction/validator modules and config watcher/reloader behavior are integrated. | Managed-text and config reload/settings-write tests. |
| `GB-3AF4-TERMINAL` | Ctrl+B backgrounds commands, Ctrl+G is mode-sensitive, remote image paste works through wrappers, and terminal/tmux/color/input handling is diagnosed consistently; dirty child exits and forwarded termination signals restore latched terminal modes. | **adopt.** Terminal keyboard, tmux probe, clipboard, mouse, and rendering changes are integrated. Production PTY wrapping now observes complete CSI sequences, tracks DEC private modes and kitty keyboard pushes, restores exact latched modes after EOF or signals, forwards termination to the child, exits as `128 + signal`, and remains byte-transparent on clean exit. | Pager-render terminal tests; 128 wrap-focused pager tests; all eight wrap PTYs; clipboard and keyboard-normalization tests. |
| `GB-3AF4-SCREEN-MODE` | `--minimal`, `--fullscreen`, and slash-command mode switches affect the current session, while the saved `[ui].screen_mode` setting controls future default launches. | **adopt.** Startup no longer persists a CLI override. Relaunch hints retain explicit mode flags for hand-pasted resume commands, and only configuration/settings writes change the saved default. | 37 screen-mode pager tests; minimal CLI non-persistence and saved-default PTYs; updated getting-started, slash-command, and configuration docs. |
| `GB-3AF4-MINIMAL-RECAP` | Minimal mode preserves full reasoning, collapses successful lookups, and recap reuses prior cached context. | **adopt.** Minimal commit/render and recap context paths are updated without changing model/provider wire contracts. | Minimal PTYs; recap/compaction tests. |
| `GB-3AF4-BACKGROUND-DASHBOARD` | Tasks pane/status/dashboard accurately represent running descendants; background loops do not overlap while descendants remain active. | **adopt.** Dashboard viewport leases, reconnect restoration, empty-session filtering, task/workflow rows, and loop descendant gates are integrated. | Dashboard/tasks/loop tests and viewport lease/reconnect regressions. |
| `GB-3AF4-RUNTIME` | Sampler/session runtime carries new retry, continuation, conversation, and status metadata without changing provider identity. | **adopt with provider-aware adaptation.** Generic runtime changes are ported around existing provider-scoped endpoints, headers, retries, tools, and compaction. | Sampling-types/sampler suites and provider-negative tests. |
| `GB-3AF4-TEST-INFRA` | Deterministic inference matching replaces timing-sensitive global FIFO assumptions. | **adopt.** Endpoint-scoped `InferenceExpectation` barriers and agent-turn expectations are restored in shared PTY support. | `xai-grok-test-support` and PTY harness suites. |
| `GB-3AF4-CORE` | Remaining shell, pager, tools, workspace, notification, and UI maintenance supports the behaviors above without a separate user-visible contract. | **adopt.** Applicable maintenance is integrated and compiled as part of the same preserved architecture. | Full touched-crate suites and binary check. |
| `GB-3AF4-BUILD` | Per-crate manifests, lockfile, changelogs, snapshots, and generated schemas support the adopted runtime. | **adopt.** Per-crate manifests and `Cargo.lock` are updated; the generated root manifest is not hand-edited. | Cargo metadata/build plus generated-manifest contract. |
| `GB-3AF4-DOCS` | User guide and changelogs describe the adopted controls and behavior. | **adopt.** Enhanced-compatible docs retain provider/release branding and compatibility surfaces. | User-guide diff and documentation checks. |
| `GB-3AF4-GENERATED-ROOT` | Upstream regenerated root `Cargo.toml`. | **not applicable.** The fork contract forbids hand-editing this generated file; per-crate manifests and lockfile carry the adopted dependency changes. | Root-manifest invariant and generated-manifest contract. |
| `GB-3AF4-SOURCE-REV` | Upstream build provenance stamp. | **not applicable.** Enhanced release provenance is fork-owned and generated by its own release workflow; an upstream source stamp is never copied. | Fork release/provenance contracts. |

Additional upstream bug-fix behavior is covered inside these rows: startup Git state includes unstaged/untracked files; randomized tool parameter names preserve descriptions; empty commands stop and wait for background work; OAuth preview redirects remain correct; `/jump`, timeline rail, stable anchors, and page-flip notices remain usable; and the startup version/doctor parser avoids filesystem/process initialization before early command dispatch. There are no open Grok obligations.

## Provider-reference behavior inventory

| Evidence ID | Reference behavior | Outcome and local contract |
| --- | --- | --- |
| `CDX-5120-MCP-EQ` | Focus MCP catalog/binding modules and preserve strict thread-scoped refresh behavior. | **already equivalent.** Enhanced merges global and session MCP configuration without mutating stored overrides and reconnects through its existing MCP contract. |
| `CDX-5120-GIT-STDIN-EQ` | Detach metadata Git commands from inherited stdin. | **already equivalent.** Centralized Git command construction uses detached stdio / `stdin(Stdio::null())`, including managed-worktree paths. |
| `CDX-5120-EDITOR-EQ` | Restore the TUI around external editing while retaining raw mode. | **already equivalent.** The adopted Grok typed external-editor path suspends/restores the terminal and preserves drafts. |
| `CDX-5120-CUDA-ADOPT` | Highlight `.cu` and `.cuh` as C++. | **adopt.** Enhanced maps CUDA extensions/tokens case-insensitively to the two-face C++ grammar. |
| `CDX-5120-EXTENSIONS-NA` | Codex step-scoped extension contributor data. | **not applicable.** This is Codex extension architecture, not the Enhanced Responses/provider wire or Grok ACP extension surface. |
| `CDX-5120-ROLLOUT-NA` | Codex rollout construction, SQLite materialization, inherited thread paging, deletion/reference protection. | **not applicable.** Enhanced preserves Grok JSONL/session storage rather than importing Codex app persistence. Observable Enhanced resume/fork durability is covered by `GB-3AF4-SESSIONS-ACP`. |
| `CDX-5120-EXEC-NA` | Noise handshake allocation in Codex exec-server. | **not applicable.** Enhanced does not embed Codex exec-server or Noise transport. |
| `CDX-5120-SKILL-SHADOW-NA` | Experimental/shadow Codex lexical and RRF skill selectors. | **not applicable.** They do not change the active ChatGPT Codex provider wire or an existing Grok user-facing selection contract. |
| `CDX-5120-SETTINGS-NA` | Fetch Codex app user settings for commit attribution. | **not applicable.** Enhanced exposes no Codex commit-attribution surface; no auth/catalog/Responses/usage contract changed. |
| `CDX-5120-PLUGIN-NA` | Accept app-server `forceRefetch` plugin-list input without behavior. | **not applicable.** Enhanced does not import the Codex app-server/plugin protocol. |
| `CDX-5120-UNIX-NA` | Compile a Codex TUI suspend helper only on Unix. | **not applicable.** This is a Codex compile gate; Enhanced terminal portability is independently covered. |
| `CDX-5120-CLEANUP-NA` | Remove unused Codex TUI setters/commands. | **not applicable.** No observable provider contract changed. |
| `CDX-5120-TESTS-NA` | Remove obsolete ignored Codex tests. | **not applicable.** No behavior or wire contract changed. |
| `OC-0317-APP-NA` | OpenCode Zen catalog docs, progress-circle contrast, nested slash autocomplete, and generated docs. | **not applicable.** These are OpenCode reference-app/UI/catalog surfaces; Codex auth and interoperability are unchanged. |
| `KIMI-B5EF-ENGINE-NA` | Kimi replacement-engine loop/background env overrides and env-safe persistence. | **not applicable.** Enhanced does not import Kimi's agent engine/config service; Grok loop/background behavior remains on its native controls. |
| `KIMI-B5EF-RELEASE-NA` | Warn when Kimi CLI was installed from a third party. | **not applicable.** Enhanced updates/releases are intentionally fork-owned and never use Kimi distribution routes. |
| `KIMI-B5EF-WEB-NA` | Checkerboard rendering for transparent images in Kimi Web. | **not applicable.** Kimi Web is not an Enhanced surface. |
| `KIMI-B5EF-LOGOUT-EQ` | Managed Kimi logout clears dangling default model/thinking. | **already equivalent under a different auth boundary.** Enhanced Kimi uses API-key auth; logout removes only Kimi credentials/cache and cannot clear or read another provider's state. |
| `KIMI-B5EF-USAGE-EQ` | Managed OAuth account-plan usage endpoint. | **already equivalent for the direct provider.** Enhanced queries authoritative `/coding/v1/usages` for Kimi API-key sessions and renders it through provider-aware `/usage`. |
| `KIMI-B5EF-CATALOG-SPLIT` | Consume context/input limits, declared thinking levels/off semantics, protocol/endpoints, and capability metadata from models.dev; also import arbitrary third-party provider overrides. | **already equivalent for Kimi / not applicable for arbitrary imports.** Authenticated Kimi discovery already honors Kimi context length, thinking type/levels (`low`, `high`, `max`), tool/image/protocol metadata, and never guesses an endpoint. Generic catalog-import/provider-override architecture is outside the explicit adapter scope. |

### Provider isolation and legal review

- xAI, OpenAI Codex, Kimi Code, Z.AI Coding Plan, and Custom selection remains explicit for auth, dynamic credentials, endpoints, model catalogs, compaction, usage, retries, hosted tools, and logout.
- Custom billing has an explicit empty managed surface and a negative test proving it cannot fall through to configured xAI state.
- Codex CUDA highlighting was independently implemented against a behavioral compatibility observation; no Codex source was copied. The Grok source update remains under the repository's existing license and notices. No new external runtime source or license was introduced, so `THIRD-PARTY-NOTICES` and crate-local notices require no new entry.

## Complete commit disposition inventory

### OpenAI Codex: `b9800de4867e500a92add3cde795cf4790306d0f`..`51200321eb7b862a29ffceaba8b19db1934a9b38`

| Commit | Subject | Evidence |
| --- | --- | --- |
| `2d85e6d3a616dc1fac258a5320c7a00a5e5bceb2` | Split MCP connection manager into focused modules (#34522) | `CDX-5120-MCP-EQ` |
| `c44c4de7b410993dacb2e88c7084c9c968bc963a` | Add step-scoped data to extension contributors (#34525) | `CDX-5120-EXTENSIONS-NA` |
| `f69f88f8116f541daddada3a056de5772a891f15` | Centralize compacted rollout item construction (#34533) | `CDX-5120-ROLLOUT-NA` |
| `0c01a18ede9ed48608f8869b18603c45f0116df4` | Detach Git metadata commands from stdin (#34540) | `CDX-5120-GIT-STDIN-EQ` |
| `2d5c25920271743de70e73ac077423dbf04aff6f` | Size Noise handshake buffers to their messages (#34544) | `CDX-5120-EXEC-NA` |
| `6915bac7ba2753277cfb6679b547c03e4fe567ed` | Add reciprocal rank fusion skill selection (#34547) | `CDX-5120-SKILL-SHADOW-NA` |
| `27a9c4d6bd1f6875127a2f0f6c2c10d95b5e5b09` | Test thread-scoped MCP refresh behavior (#34550) | `CDX-5120-MCP-EQ` |
| `b5b8a63fea2b24ffe94e7b3bfb51b4c9d5b5fd20` | Simplify TUI restoration for the external editor (#34551) | `CDX-5120-EDITOR-EQ` |
| `4c2dc5d2858822a9db82091cc17e06d34880df48` | Remove unused RtOptions setters (#34552) | `CDX-5120-CLEANUP-NA` |
| `40a719238cbcd2c5428ff5c3c7687e86b3d47fae` | Remove the unused TUI shutdown app command (#34553) | `CDX-5120-CLEANUP-NA` |
| `268288ad6dc0ee4a80e1bff38354efbb71b13ac6` | Remove obsolete ignored tests (#34558) | `CDX-5120-TESTS-NA` |
| `98c3bbdc798a9d9b301db3a92d7118c5281d5b3e` | Add backend client support for Codex user settings (#34559) | `CDX-5120-SETTINGS-NA` |
| `f6aad1f3631455477a4f129e6b997ab511e38cf5` | Extract MCP binding clients from the connection manager (#34561) | `CDX-5120-MCP-EQ` |
| `175f82147f4989d714d6899261b6c2c391f28b0b` | Record rollout boundaries for materialized turns (#34562) | `CDX-5120-ROLLOUT-NA` |
| `7bb13ab846a36341129bc431bbe277ded748a444` | Page through inherited thread history (#34563) | `CDX-5120-ROLLOUT-NA` |
| `0b175e6439a8608ba7726ee153fd8590619e8f34` | Protect fork history references during rollout cleanup (#34566) | `CDX-5120-ROLLOUT-NA` |
| `726689564aa7ab938f40bfd94479d5b66c48c91b` | Highlight CUDA files as C++ in the TUI (#34570) | `CDX-5120-CUDA-ADOPT` |
| `6ac68be6fef048b579f0a4413ae107409b476b30` | Accept `forceRefetch` in plugin list requests (#34573) | `CDX-5120-PLUGIN-NA` |
| `af71774d2645ec900cccf2a40d186a56c7a42f71` | Gate the TUI suspend restore helper on Unix (#34578) | `CDX-5120-UNIX-NA` |
| `51200321eb7b862a29ffceaba8b19db1934a9b38` | Add routing-card lexical skill selection (#34581) | `CDX-5120-SKILL-SHADOW-NA` |

### OpenCode: `cb562b2c6289c2eee707078f9ab644cbe1d3d8a9`..`0317531906d3f3bb01cf33c16319870cfde9170c`

| Commit | Subject | Evidence |
| --- | --- | --- |
| `abc4b83f58bcfb632d96375f9c607d25fd31a52f` | Zen: gemini 3.6 flash and 3.5 flash lite | `OC-0317-APP-NA` |
| `21c4c93770073e331051ea3ebd706bdc9bdc8b5b` | fix(ui): improve progress circle contrast (#38101) | `OC-0317-APP-NA` |
| `b513fafe8354b85d1ba4609733b7f33c7b46b7d5` | fix(app): support nested slash command autocomplete (#38097) | `OC-0317-APP-NA` |
| `5f241f1cc1fc0c266044b64bf9e860d4e37c9c1f` | chore: generate | `OC-0317-APP-NA` |
| `0317531906d3f3bb01cf33c16319870cfde9170c` | zen: add laguna-2-2.1 | `OC-0317-APP-NA` |

### Kimi Code: `a3699dd6aa7b41efd3129a117007d195282379fd`..`b5efba7abcaf4041f81ec520097a61e6546e8c50`

| Commit | Subject | Evidence |
| --- | --- | --- |
| `37eda4e59aebc8ecafa91be3f43f971ed63963a3` | feat(config): add env overrides for loop control and background task limits (#1993) | `KIMI-B5EF-ENGINE-NA` |
| `576d65038035570bea90b58d5824bcd60ca11258` | feat(cli): add third-party source note to update prompt (#2014) | `KIMI-B5EF-RELEASE-NA` |
| `154e0824880c8573433e4ec7ada083744dbfe9f9` | fix(web): keep transparent images readable over a checkerboard canvas (#2022) | `KIMI-B5EF-WEB-NA` |
| `0a1b5fa00fd90ffd30f6e119b9f071adcf07734b` | fix(agent-core-v2): clear dangling defaultModel and thinking on managed logout (#2026) | `KIMI-B5EF-LOGOUT-EQ` |
| `8e0dcf304966989ba210167fa4d9361729a2f338` | feat(kap-server): add GET /api/v1/oauth/usage for managed account plan usage (#2027) | `KIMI-B5EF-USAGE-EQ` |
| `b5efba7abcaf4041f81ec520097a61e6546e8c50` | fix: consume the model metadata declared by the models.dev catalog (#2015) | `KIMI-B5EF-CATALOG-SPLIT` |

## Complete advanced-reference changed-path inventories

These are exact `git diff --name-status <reviewed> <pin>` records. Each record appears once.

<details><summary>OpenAI Codex: 100 changed paths</summary>

```text
M	codex-rs/Cargo.lock
M	codex-rs/app-server-protocol/schema/json/ClientRequest.json
M	codex-rs/app-server-protocol/schema/json/codex_app_server_protocol.schemas.json
M	codex-rs/app-server-protocol/schema/json/codex_app_server_protocol.v2.schemas.json
M	codex-rs/app-server-protocol/schema/json/v2/PluginListParams.json
M	codex-rs/app-server-protocol/schema/typescript/v2/PluginListParams.ts
M	codex-rs/app-server-protocol/src/protocol/common.rs
M	codex-rs/app-server-protocol/src/protocol/v2/plugin.rs
M	codex-rs/app-server-protocol/src/protocol/v2/tests.rs
M	codex-rs/app-server/README.md
M	codex-rs/app-server/src/mcp_refresh.rs
M	codex-rs/app-server/src/request_processors.rs
M	codex-rs/app-server/src/request_processors/plugins.rs
M	codex-rs/app-server/src/request_processors/thread_delete.rs
M	codex-rs/app-server/tests/suite/v2/external_agent_config.rs
M	codex-rs/app-server/tests/suite/v2/plugin_list.rs
M	codex-rs/app-server/tests/suite/v2/plugin_share.rs
M	codex-rs/app-server/tests/suite/v2/skills_list.rs
M	codex-rs/app-server/tests/suite/v2/thread_delete.rs
M	codex-rs/backend-client/Cargo.toml
M	codex-rs/backend-client/src/client.rs
M	codex-rs/backend-client/src/lib.rs
M	codex-rs/backend-client/src/types.rs
M	codex-rs/cloud-tasks/src/lib.rs
A	codex-rs/codex-mcp/src/binding_clients.rs
M	codex-rs/codex-mcp/src/connection_manager.rs
A	codex-rs/codex-mcp/src/connection_manager/required.rs
A	codex-rs/codex-mcp/src/connection_manager/tool_catalog.rs
M	codex-rs/codex-mcp/src/connection_manager_tests.rs
M	codex-rs/codex-mcp/src/lib.rs
M	codex-rs/core/src/compact.rs
M	codex-rs/core/src/compact_remote.rs
M	codex-rs/core/src/compact_remote_v2.rs
M	codex-rs/core/src/compact_tests.rs
M	codex-rs/core/src/compact_token_budget.rs
M	codex-rs/core/src/guardian/tests.rs
M	codex-rs/core/src/session/mod.rs
M	codex-rs/core/src/session/step_context.rs
M	codex-rs/core/src/session/tests.rs
M	codex-rs/core/src/session/tests/guardian_tests.rs
M	codex-rs/core/src/session/turn.rs
M	codex-rs/core/src/session/world_state.rs
M	codex-rs/core/src/tools/router.rs
M	codex-rs/core/src/tools/router_tests.rs
M	codex-rs/core/tests/suite/codex_delegate.rs
M	codex-rs/core/tests/suite/unified_exec.rs
M	codex-rs/exec-server/src/noise_channel.rs
M	codex-rs/ext/extension-api/examples/enabled_extensions.rs
M	codex-rs/ext/extension-api/examples/enabled_extensions/shared_state_extension.rs
M	codex-rs/ext/extension-api/src/contributors.rs
M	codex-rs/ext/extension-api/src/contributors/context.rs
M	codex-rs/ext/extension-api/src/contributors/world_state.rs
M	codex-rs/ext/extension-api/tests/registry.rs
M	codex-rs/ext/goal/src/extension.rs
M	codex-rs/ext/goal/tests/goal_extension_backend.rs
M	codex-rs/ext/image-generation/src/extension.rs
M	codex-rs/ext/memories/src/extension.rs
M	codex-rs/ext/memories/src/tests.rs
M	codex-rs/ext/skills/src/dynamic_skill_selector.rs
M	codex-rs/ext/skills/src/dynamic_skill_selector/character_ngram_tests.rs
M	codex-rs/ext/skills/src/dynamic_skill_selector/fielded_bm25_tests.rs
M	codex-rs/ext/skills/src/dynamic_skill_selector/multi_query_lexical_tests.rs
A	codex-rs/ext/skills/src/dynamic_skill_selector/routing_card_lexical.rs
A	codex-rs/ext/skills/src/dynamic_skill_selector/routing_card_lexical_tests.rs
A	codex-rs/ext/skills/src/dynamic_skill_selector/rrf_lexical_char.rs
A	codex-rs/ext/skills/src/dynamic_skill_selector/rrf_lexical_char_tests.rs
M	codex-rs/ext/skills/src/dynamic_skill_selector/weighted_lexical_tests.rs
M	codex-rs/ext/skills/src/extension.rs
M	codex-rs/ext/skills/src/shadow_selection_experiment.rs
M	codex-rs/ext/skills/tests/skills_extension.rs
M	codex-rs/ext/web-search/src/extension.rs
M	codex-rs/git-utils/src/info.rs
M	codex-rs/rollout/src/compression.rs
M	codex-rs/rollout/src/compression_tests.rs
M	codex-rs/rollout/src/lib.rs
A	codex-rs/rollout/src/rollout_reference_index.rs
A	codex-rs/rollout/src/rollout_reference_index_tests.rs
A	codex-rs/state/thread_history_migrations/0003_turn_rollout_positions.sql
M	codex-rs/thread-store/src/lib.rs
M	codex-rs/thread-store/src/local/delete_thread.rs
M	codex-rs/thread-store/src/local/mod.rs
M	codex-rs/thread-store/src/local/rollout_lineage.rs
M	codex-rs/thread-store/src/local/thread_history.rs
M	codex-rs/thread-store/src/local/thread_history/read.rs
M	codex-rs/thread-store/src/local/thread_history/read_tests.rs
M	codex-rs/thread-store/src/local/thread_history/search.rs
A	codex-rs/thread-store/src/local/thread_history/segment_paging.rs
M	codex-rs/thread-store/src/local/thread_history_materialization.rs
M	codex-rs/thread-store/src/local/thread_history_materialization_tests.rs
M	codex-rs/thread-store/src/store.rs
M	codex-rs/thread-store/src/types.rs
M	codex-rs/tui/src/app/background_requests.rs
M	codex-rs/tui/src/app/input.rs
M	codex-rs/tui/src/app/pending_interactive_replay.rs
M	codex-rs/tui/src/app_command.rs
M	codex-rs/tui/src/diff_render.rs
M	codex-rs/tui/src/render/highlight.rs
M	codex-rs/tui/src/snapshots/codex_tui__diff_render__tests__cpp_module_extension_highlighting.snap
M	codex-rs/tui/src/tui.rs
M	codex-rs/tui/src/wrapping.rs
```
</details>

<details><summary>OpenCode: 21 changed paths</summary>

```text
M	packages/session-ui/src/v2/components/prompt-input/machine.test.ts
M	packages/session-ui/src/v2/components/prompt-input/machine.ts
M	packages/ui/src/v2/components/progress-circle-v2.css
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
```
</details>

<details><summary>Kimi Code: 101 changed paths</summary>

```text
M	.agents/skills/agent-core-dev/config.md
A	.changeset/catalog-import-broader-and-safer.md
A	.changeset/config-env-overrides.md
A	.changeset/config-env-persist-and-stale-fixes.md
A	.changeset/fix-web-media-alpha-canvas.md
A	.changeset/thinking-levels-from-declared-capabilities.md
A	.changeset/update-third-party-source-note.md
M	README.md
M	README.zh-CN.md
M	apps/kimi-code/src/cli/sub/provider.ts
M	apps/kimi-code/src/cli/update/preflight.ts
M	apps/kimi-code/src/constant/app.ts
M	apps/kimi-code/src/tui/commands/prompts.ts
M	apps/kimi-code/src/tui/commands/provider.ts
M	apps/kimi-code/src/tui/components/dialogs/api-key-input-dialog.ts
M	apps/kimi-code/test/cli/provider.test.ts
M	apps/kimi-code/test/cli/update/preflight.test.ts
M	apps/kimi-code/test/tui/kimi-tui-message-flow.test.ts
M	apps/kimi-web/scripts/check-style.mjs
M	apps/kimi-web/src/components/FilePreview.vue
M	apps/kimi-web/src/components/chat/Markdown.vue
M	apps/kimi-web/src/components/chat/tool-calls/MediaTool.vue
M	apps/kimi-web/src/style.css
M	apps/kimi-web/src/views/DesignSystemView.vue
M	docs/en/configuration/config-files.md
M	docs/en/configuration/env-vars.md
M	docs/en/reference/kimi-command.md
M	docs/zh/configuration/config-files.md
M	docs/zh/configuration/env-vars.md
M	docs/zh/reference/kimi-command.md
M	packages/acp-adapter/src/model-catalog.ts
M	packages/acp-adapter/test/config-options.test.ts
M	packages/acp-adapter/test/model-catalog.test.ts
M	packages/agent-core-v2/src/agent/fullCompaction/fullCompactionService.ts
M	packages/agent-core-v2/src/agent/fullCompaction/strategy.ts
M	packages/agent-core-v2/src/agent/loop/configSection.ts
M	packages/agent-core-v2/src/agent/media/configSection.ts
M	packages/agent-core-v2/src/agent/task/configSection.ts
M	packages/agent-core-v2/src/app/auth/auth.ts
M	packages/agent-core-v2/src/app/auth/authService.ts
M	packages/agent-core-v2/src/app/auth/oauthProtocol.ts
M	packages/agent-core-v2/src/app/config/config.ts
M	packages/agent-core-v2/src/app/config/configService.ts
M	packages/agent-core-v2/src/kosong/contract/capability.ts
M	packages/agent-core-v2/src/kosong/model/catalog.ts
M	packages/agent-core-v2/src/kosong/model/catalogService.ts
M	packages/agent-core-v2/src/kosong/model/inspection.ts
M	packages/agent-core-v2/src/kosong/model/model.ts
M	packages/agent-core-v2/src/kosong/model/modelAuth.ts
M	packages/agent-core-v2/src/kosong/model/thinking.ts
M	packages/agent-core-v2/src/kosong/protocol/protocol.ts
M	packages/agent-core-v2/src/kosong/provider/bases/anthropic/anthropic-profile.ts
M	packages/agent-core-v2/src/kosong/provider/bases/openai/openai-legacy.contrib.ts
M	packages/agent-core-v2/src/kosong/provider/bases/openai/openai-legacy.ts
M	packages/agent-core-v2/src/kosong/provider/bases/openai/openai-responses.contrib.ts
M	packages/agent-core-v2/src/kosong/provider/bases/openai/openai-responses.ts
M	packages/agent-core-v2/src/kosong/provider/providerService.ts
M	packages/agent-core-v2/src/session/subagent/configSection.ts
M	packages/agent-core-v2/test/agent/profile/config-state.test.ts
M	packages/agent-core-v2/test/agent/profile/thinking.test.ts
M	packages/agent-core-v2/test/app/auth/auth.test.ts
M	packages/agent-core-v2/test/app/config/config.test.ts
M	packages/agent-core-v2/test/app/model/model.test.ts
M	packages/agent-core-v2/test/app/provider/provider.test.ts
M	packages/agent-core-v2/test/kosong/model/catalog.test.ts
M	packages/agent-core-v2/test/kosong/provider/composition.test.ts
M	packages/agent-core/src/agent/background/index.ts
M	packages/agent-core/src/agent/compaction/full.ts
M	packages/agent-core/src/agent/config/thinking.ts
M	packages/agent-core/src/agent/context/index.ts
M	packages/agent-core/src/agent/index.ts
M	packages/agent-core/src/agent/turn/index.ts
M	packages/agent-core/src/config/model.ts
M	packages/agent-core/src/config/schema.ts
M	packages/agent-core/src/services/session/sessionService.ts
M	packages/agent-core/src/session/provider-manager.ts
M	packages/agent-core/test/agent/background/manager.test.ts
M	packages/agent-core/test/agent/config/thinking.test.ts
M	packages/agent-core/test/agent/turn.test.ts
M	packages/agent-core/test/config/model-overrides.test.ts
M	packages/agent-core/test/harness/model-alias-session.test.ts
M	packages/agent-core/test/harness/runtime-provider.test.ts
M	packages/agent-core/test/services/model-catalog-service.test.ts
M	packages/kap-server/src/routes/oauth.ts
M	packages/kap-server/test/__snapshots__/apiSurface.snapshot.test.ts.snap
M	packages/kap-server/test/auth.test.ts
M	packages/kap-server/test/modelCatalog.test.ts
A	packages/kap-server/test/oauthUsage.test.ts
M	packages/kosong/src/capability.ts
M	packages/kosong/src/catalog.ts
M	packages/kosong/src/index.ts
M	packages/kosong/src/providers/anthropic-profile.ts
M	packages/kosong/src/providers/openai-legacy.ts
M	packages/kosong/src/providers/openai-responses.ts
M	packages/kosong/test/anthropic.test.ts
M	packages/kosong/test/catalog.test.ts
M	packages/kosong/test/openai-legacy.test.ts
M	packages/kosong/test/openai-responses.test.ts
M	packages/node-sdk/src/catalog.ts
M	packages/node-sdk/src/index.ts
M	packages/node-sdk/test/catalog.test.ts
```
</details>

## Validation gate

Validation completed against the final formatted candidate before acknowledgement:

- `cargo fmt --all` and `git diff --check` passed.
- The full pager library suite passed **7,622 tests**, with 10 intentionally ignored and zero failures. The full shell library suite passed **5,954 tests**, with 14 intentionally ignored and zero failures.
- The settings integration suite passed all **270 tests**. Wrap-focused pager coverage passed all **128 tests**, and screen-mode pager coverage passed all **37 tests**.
- Built-binary ignored coverage passed all five doctor scenarios, all four Stop-hook scenarios, and both refusal ACP scenarios.
- All eight wrap PTYs passed, including clean byte transparency, latched-mode restoration after child death, signal forwarding/restoration, and exit status 143 for SIGTERM. The markerless-wake, page-flip enabled/disabled, parallel/sequential edit-merge, and iTerm readline PTYs passed. Focused minimal PTYs passed for CLI non-persistence, saved defaults, external editing, full thinking, collapsed lookup, and continuation replay.
- `CARGO_INCREMENTAL=0 cargo check -p xai-grok-pager-bin` passed. Cargo emitted only the known non-fatal structural warning that `xai-grok-pager-render/build.rs` is both the build script and the `warp_vendor_build_validation` integration-test target.
- All **72 fork-script guardrail tests** and all **20 release/installer tests** passed. The aggregate static fork checker passed branding, provider isolation, Codex search, Warp locks, updater routes, generated-workspace discipline, workflow pins, and tracked-secret hygiene.
- The committed prospective first-parent passed `fork/scripts/check_manifest.py --strict-coverage --prepare-upstream-acknowledgements`; every baseline-to-candidate path is owned and the ordered acknowledgement chain is exact.

Credential-gated live calls were not repeated for this source-only audit. No authenticated payload, account state, or credential-bearing diagnostic was captured.

## Raw-path outcome summary

- **Adopt:** 555 raw Grok paths.
- **Already equivalent:** 0 raw Grok paths (equivalence is recorded at behavior/reference level where applicable).
- **Not applicable:** 2 raw Grok paths (`Cargo.toml` generated root and `SOURCE_REV` upstream provenance).
- **Temporarily deferred:** 0.
- **Unclassified:** 0.

A raw-path `adopt` row means all applicable observable behavior in that source path is present in the Enhanced candidate, potentially through an adapted implementation that preserves downstream provider, permission, session, release, or UI architecture. It does not claim byte identity.

## Complete 557 raw-path ledger

| # | Raw upstream path | Outcome | Evidence |
| ---: | --- | --- | --- |
| 1 | `M` `Cargo.lock` | adopt | `GB-3AF4-BUILD` |
| 2 | `M` `Cargo.toml` | not applicable | `GB-3AF4-GENERATED-ROOT` |
| 3 | `M` `SOURCE_REV` | not applicable | `GB-3AF4-SOURCE-REV` |
| 4 | `M` `crates/codegen/xai-chat-state/src/actor/mod.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 5 | `M` `crates/codegen/xai-chat-state/src/actor/mutations.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 6 | `M` `crates/codegen/xai-chat-state/src/actor/request_builder.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 7 | `M` `crates/codegen/xai-chat-state/src/actor/tests.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 8 | `M` `crates/codegen/xai-chat-state/src/commands.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 9 | `M` `crates/codegen/xai-chat-state/src/compaction_utils.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 10 | `M` `crates/codegen/xai-chat-state/src/handle.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 11 | `M` `crates/codegen/xai-chat-state/src/lib.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 12 | `M` `crates/codegen/xai-chat-state/src/persistence.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 13 | `M` `crates/codegen/xai-fast-worktree/Cargo.toml` | adopt | `GB-3AF4-BUILD` |
| 14 | `M` `crates/codegen/xai-fast-worktree/src/api.rs` | adopt | `GB-3AF4-WORKTREE` |
| 15 | `A` `crates/codegen/xai-fast-worktree/src/auto_gc.rs` | adopt | `GB-3AF4-WORKTREE` |
| 16 | `M` `crates/codegen/xai-fast-worktree/src/db/mod.rs` | adopt | `GB-3AF4-WORKTREE` |
| 17 | `M` `crates/codegen/xai-fast-worktree/src/discovery.rs` | adopt | `GB-3AF4-WORKTREE` |
| 18 | `M` `crates/codegen/xai-fast-worktree/src/git/checkout.rs` | adopt | `GB-3AF4-WORKTREE` |
| 19 | `M` `crates/codegen/xai-fast-worktree/src/lib.rs` | adopt | `GB-3AF4-WORKTREE` |
| 20 | `M` `crates/codegen/xai-grok-agent/src/builder.rs` | adopt | `GB-3AF4-CORE` |
| 21 | `M` `crates/codegen/xai-grok-agent/src/config.rs` | adopt | `GB-3AF4-CORE` |
| 22 | `M` `crates/codegen/xai-grok-config-types/src/lib.rs` | adopt | `GB-3AF4-CORE` |
| 23 | `M` `crates/codegen/xai-grok-config/src/config_override.rs` | adopt | `GB-3AF4-CONFIG` |
| 24 | `M` `crates/codegen/xai-grok-config/src/lib.rs` | adopt | `GB-3AF4-CORE` |
| 25 | `A` `crates/codegen/xai-grok-config/src/managed_text/format.rs` | adopt | `GB-3AF4-CONFIG` |
| 26 | `A` `crates/codegen/xai-grok-config/src/managed_text/mod.rs` | adopt | `GB-3AF4-CONFIG` |
| 27 | `A` `crates/codegen/xai-grok-config/src/managed_text/source.rs` | adopt | `GB-3AF4-CONFIG` |
| 28 | `A` `crates/codegen/xai-grok-config/src/managed_text/tests.rs` | adopt | `GB-3AF4-CONFIG` |
| 29 | `A` `crates/codegen/xai-grok-config/src/managed_text/transaction.rs` | adopt | `GB-3AF4-CONFIG` |
| 30 | `A` `crates/codegen/xai-grok-config/src/managed_text/validator.rs` | adopt | `GB-3AF4-CONFIG` |
| 31 | `M` `crates/codegen/xai-grok-hooks/src/dispatcher.rs` | adopt | `GB-3AF4-HOOKS` |
| 32 | `M` `crates/codegen/xai-grok-markdown/src/mermaid.rs` | adopt | `GB-3AF4-CORE` |
| 33 | `M` `crates/codegen/xai-grok-mcp/src/servers.rs` | adopt | `GB-3AF4-CORE` |
| 34 | `M` `crates/codegen/xai-grok-memory/src/dream.rs` | adopt | `GB-3AF4-CORE` |
| 35 | `M` `crates/codegen/xai-grok-pager-bin/Cargo.toml` | adopt | `GB-3AF4-BUILD` |
| 36 | `M` `crates/codegen/xai-grok-pager-bin/src/main.rs` | adopt | `GB-3AF4-CORE` |
| 37 | `M` `crates/codegen/xai-grok-pager-minimal/src/live.rs` | adopt | `GB-3AF4-MINIMAL-RECAP` |
| 38 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/content.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 39 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/lib.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 40 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/scenarios/empty_enter_send_now.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 41 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/scenarios/plan_approval_resume.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 42 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/scripted.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 43 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/scroll_matrix/runner.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 44 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/scroll_matrix/session.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 45 | `M` `crates/codegen/xai-grok-pager-pty-harness/tests/scroll_correctness_ptyctl.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 46 | `M` `crates/codegen/xai-grok-pager-pty-harness/tests/scroll_matrix_curated.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 47 | `M` `crates/codegen/xai-grok-pager-render/src/appearance/cache.rs` | adopt | `GB-3AF4-CORE` |
| 48 | `M` `crates/codegen/xai-grok-pager-render/src/clipboard/mod.rs` | adopt | `GB-3AF4-TERMINAL` |
| 49 | `M` `crates/codegen/xai-grok-pager-render/src/glyphs.rs` | adopt | `GB-3AF4-TERMINAL` |
| 50 | `M` `crates/codegen/xai-grok-pager-render/src/terminal/keyboard.rs` | adopt | `GB-3AF4-TERMINAL` |
| 51 | `M` `crates/codegen/xai-grok-pager-render/src/terminal/mod.rs` | adopt | `GB-3AF4-TERMINAL` |
| 52 | `M` `crates/codegen/xai-grok-pager-render/src/terminal/test.rs` | adopt | `GB-3AF4-TERMINAL` |
| 53 | `A` `crates/codegen/xai-grok-pager-render/src/terminal/tmux_probe.rs` | adopt | `GB-3AF4-DOCTOR` |
| 54 | `M` `crates/codegen/xai-grok-pager-render/src/theme/color_support.rs` | adopt | `GB-3AF4-TERMINAL` |
| 55 | `M` `crates/codegen/xai-grok-pager-render/src/util.rs` | adopt | `GB-3AF4-CORE` |
| 56 | `M` `crates/codegen/xai-grok-pager/Cargo.toml` | adopt | `GB-3AF4-BUILD` |
| 57 | `M` `crates/codegen/xai-grok-pager/README.md` | adopt | `GB-3AF4-DOCS` |
| 58 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/03-keyboard-shortcuts.md` | adopt | `GB-3AF4-TERMINAL` |
| 59 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/04-slash-commands.md` | adopt | `GB-3AF4-DOCS` |
| 60 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/05-configuration.md` | adopt | `GB-3AF4-DOCS` |
| 61 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/06-theming.md` | adopt | `GB-3AF4-DOCS` |
| 62 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/08-skills.md` | adopt | `GB-3AF4-DOCS` |
| 63 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/14-headless-mode.md` | adopt | `GB-3AF4-DOCS` |
| 64 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/16-subagents.md` | adopt | `GB-3AF4-BACKGROUND-DASHBOARD` |
| 65 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/20-background-tasks.md` | adopt | `GB-3AF4-BACKGROUND-DASHBOARD` |
| 66 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/21-terminal-support.md` | adopt | `GB-3AF4-TERMINAL` |
| 67 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/22-permissions-and-safety.md` | adopt | `GB-3AF4-PERMISSIONS` |
| 68 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/23-dashboard.md` | adopt | `GB-3AF4-BACKGROUND-DASHBOARD` |
| 69 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/README.md` | adopt | `GB-3AF4-DOCS` |
| 70 | `M` `crates/codegen/xai-grok-pager/src/acp/meta.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 71 | `M` `crates/codegen/xai-grok-pager/src/acp/model_state.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 72 | `M` `crates/codegen/xai-grok-pager/src/acp/spawn.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 73 | `M` `crates/codegen/xai-grok-pager/src/acp/tracker.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 74 | `M` `crates/codegen/xai-grok-pager/src/actions/defaults.rs` | adopt | `GB-3AF4-CORE` |
| 75 | `M` `crates/codegen/xai-grok-pager/src/actions/mod.rs` | adopt | `GB-3AF4-CORE` |
| 76 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/mcp.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 77 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/mod.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 78 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/prompt_origin.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 79 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/queue.rs` | adopt | `GB-3AF4-TURN-QUEUE` |
| 80 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/session_notification.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 81 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/settings.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 82 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/goals.rs` | adopt | `GB-3AF4-AUTO-FEEDBACK` |
| 83 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/mod.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 84 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/plan_mode.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 85 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/queue_and_adoption.rs` | adopt | `GB-3AF4-TURN-QUEUE` |
| 86 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/session_events.rs` | adopt | `GB-3AF4-TURN-QUEUE` |
| 87 | `A` `crates/codegen/xai-grok-pager/src/app/acp_handler/workflow_ingest.rs` | adopt | `GB-3AF4-WORKFLOWS` |
| 88 | `M` `crates/codegen/xai-grok-pager/src/app/actions.rs` | adopt | `GB-3AF4-CORE` |
| 89 | `M` `crates/codegen/xai-grok-pager/src/app/agent.rs` | adopt | `GB-3AF4-CORE` |
| 90 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/input.rs` | adopt | `GB-3AF4-CORE` |
| 91 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/links.rs` | adopt | `GB-3AF4-CORE` |
| 92 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/mod.rs` | adopt | `GB-3AF4-CORE` |
| 93 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/notices.rs` | adopt | `GB-3AF4-CORE` |
| 94 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/panes.rs` | adopt | `GB-3AF4-CORE` |
| 95 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/prompt.rs` | adopt | `GB-3AF4-CORE` |
| 96 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/queue.rs` | adopt | `GB-3AF4-TURN-QUEUE` |
| 97 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/render.rs` | adopt | `GB-3AF4-CORE` |
| 98 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/selection.rs` | adopt | `GB-3AF4-CORE` |
| 99 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/session.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 100 | `A` `crates/codegen/xai-grok-pager/src/app/agent_view/workflows_overlay.rs` | adopt | `GB-3AF4-WORKFLOWS` |
| 101 | `M` `crates/codegen/xai-grok-pager/src/app/app_view.rs` | adopt | `GB-3AF4-CORE` |
| 102 | `M` `crates/codegen/xai-grok-pager/src/app/cli.rs` | adopt | `GB-3AF4-CORE` |
| 103 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/dashboard.rs` | adopt | `GB-3AF4-BACKGROUND-DASHBOARD` |
| 104 | `A` `crates/codegen/xai-grok-pager/src/app/dispatch/external_editor.rs` | adopt | `GB-3AF4-EXTERNAL-EDITOR` |
| 105 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/mod.rs` | adopt | `GB-3AF4-CORE` |
| 106 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/prompt.rs` | adopt | `GB-3AF4-CORE` |
| 107 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/queue.rs` | adopt | `GB-3AF4-TURN-QUEUE` |
| 108 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/router.rs` | adopt | `GB-3AF4-CORE` |
| 109 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/session/foreign.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 110 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 111 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/settings/setters.rs` | adopt | `GB-3AF4-CORE` |
| 112 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/settings/ui.rs` | adopt | `GB-3AF4-CORE` |
| 113 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/status.rs` | adopt | `GB-3AF4-CORE` |
| 114 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/task_result.rs` | adopt | `GB-3AF4-CORE` |
| 115 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/auth.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 116 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/billing.rs` | adopt | `GB-3AF4-USAGE` |
| 117 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/cta_e2e.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 118 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/dashboard.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 119 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/mod.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 120 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/modes.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 121 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/prompt.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 122 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/rewind.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 123 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/router.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 124 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/foreign.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 125 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/fork.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 126 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/lifecycle.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 127 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/load.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 128 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/take_deferred.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 129 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/settings.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 130 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/status.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 131 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/task_result.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 132 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/turn.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 133 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/voice.rs` | adopt | `GB-3AF4-VOICE` |
| 134 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/transcript.rs` | adopt | `GB-3AF4-CORE` |
| 135 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/turn.rs` | adopt | `GB-3AF4-CORE` |
| 136 | `M` `crates/codegen/xai-grok-pager/src/app/effects/helpers.rs` | adopt | `GB-3AF4-CORE` |
| 137 | `M` `crates/codegen/xai-grok-pager/src/app/effects/mod.rs` | adopt | `GB-3AF4-CORE` |
| 138 | `M` `crates/codegen/xai-grok-pager/src/app/effects/tests.rs` | adopt | `GB-3AF4-CORE` |
| 139 | `M` `crates/codegen/xai-grok-pager/src/app/event_loop.rs` | adopt | `GB-3AF4-BACKGROUND-DASHBOARD` |
| 140 | `A` `crates/codegen/xai-grok-pager/src/app/external_editor.rs` | adopt | `GB-3AF4-EXTERNAL-EDITOR` |
| 141 | `M` `crates/codegen/xai-grok-pager/src/app/leader_cluster/scenarios.rs` | adopt | `GB-3AF4-CORE` |
| 142 | `M` `crates/codegen/xai-grok-pager/src/app/mod.rs` | adopt | `GB-3AF4-CORE` |
| 143 | `M` `crates/codegen/xai-grok-pager/src/app/modals.rs` | adopt | `GB-3AF4-CORE` |
| 144 | `M` `crates/codegen/xai-grok-pager/src/app/mouse.rs` | adopt | `GB-3AF4-CORE` |
| 145 | `M` `crates/codegen/xai-grok-pager/src/app/queue_edit.rs` | adopt | `GB-3AF4-TURN-QUEUE` |
| 146 | `M` `crates/codegen/xai-grok-pager/src/app/session_startup.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 147 | `A` `crates/codegen/xai-grok-pager/src/app/snapshots/xai_grok_pager__app__status_blocks__tests__session_usage_block_absent_cost.snap` | adopt | `GB-3AF4-USAGE` |
| 148 | `A` `crates/codegen/xai-grok-pager/src/app/snapshots/xai_grok_pager__app__status_blocks__tests__session_usage_block_full.snap` | adopt | `GB-3AF4-USAGE` |
| 149 | `M` `crates/codegen/xai-grok-pager/src/app/status_blocks.rs` | adopt | `GB-3AF4-USAGE` |
| 150 | `M` `crates/codegen/xai-grok-pager/src/app/subagent.rs` | adopt | `GB-3AF4-BACKGROUND-DASHBOARD` |
| 151 | `M` `crates/codegen/xai-grok-pager/src/app/turn_completion/tests.rs` | adopt | `GB-3AF4-TURN-QUEUE` |
| 152 | `D` `crates/codegen/xai-grok-pager/src/diagnostics.rs` | adopt | `GB-3AF4-DOCTOR` |
| 153 | `A` `crates/codegen/xai-grok-pager/src/diagnostics/doctor_format.rs` | adopt | `GB-3AF4-DOCTOR` |
| 154 | `A` `crates/codegen/xai-grok-pager/src/diagnostics/doctor_format_tests.rs` | adopt | `GB-3AF4-DOCTOR` |
| 155 | `A` `crates/codegen/xai-grok-pager/src/diagnostics/fix.rs` | adopt | `GB-3AF4-DOCTOR` |
| 156 | `A` `crates/codegen/xai-grok-pager/src/diagnostics/fix_tests.rs` | adopt | `GB-3AF4-DOCTOR` |
| 157 | `A` `crates/codegen/xai-grok-pager/src/diagnostics/mod.rs` | adopt | `GB-3AF4-DOCTOR` |
| 158 | `A` `crates/codegen/xai-grok-pager/src/diagnostics/model.rs` | adopt | `GB-3AF4-DOCTOR` |
| 159 | `A` `crates/codegen/xai-grok-pager/src/diagnostics/probes/mod.rs` | adopt | `GB-3AF4-DOCTOR` |
| 160 | `A` `crates/codegen/xai-grok-pager/src/diagnostics/probes/tmux.rs` | adopt | `GB-3AF4-DOCTOR` |
| 161 | `A` `crates/codegen/xai-grok-pager/src/diagnostics/view.rs` | adopt | `GB-3AF4-DOCTOR` |
| 162 | `A` `crates/codegen/xai-grok-pager/src/diagnostics/view_tests.rs` | adopt | `GB-3AF4-DOCTOR` |
| 163 | `M` `crates/codegen/xai-grok-pager/src/docs.rs` | adopt | `GB-3AF4-CORE` |
| 164 | `A` `crates/codegen/xai-grok-pager/src/doctor_cmd/human.rs` | adopt | `GB-3AF4-DOCTOR` |
| 165 | `A` `crates/codegen/xai-grok-pager/src/doctor_cmd/json.rs` | adopt | `GB-3AF4-DOCTOR` |
| 166 | `A` `crates/codegen/xai-grok-pager/src/doctor_cmd/mod.rs` | adopt | `GB-3AF4-DOCTOR` |
| 167 | `A` `crates/codegen/xai-grok-pager/src/doctor_cmd/tests.rs` | adopt | `GB-3AF4-DOCTOR` |
| 168 | `M` `crates/codegen/xai-grok-pager/src/headless.rs` | adopt | `GB-3AF4-CORE` |
| 169 | `M` `crates/codegen/xai-grok-pager/src/input/keyboard_normalizer.rs` | adopt | `GB-3AF4-TERMINAL` |
| 170 | `M` `crates/codegen/xai-grok-pager/src/input/mouse.rs` | adopt | `GB-3AF4-TERMINAL` |
| 171 | `M` `crates/codegen/xai-grok-pager/src/input/mouse/tests.rs` | adopt | `GB-3AF4-TERMINAL` |
| 172 | `M` `crates/codegen/xai-grok-pager/src/lib.rs` | adopt | `GB-3AF4-CORE` |
| 173 | `M` `crates/codegen/xai-grok-pager/src/minimal/api.rs` | adopt | `GB-3AF4-MINIMAL-RECAP` |
| 174 | `M` `crates/codegen/xai-grok-pager/src/plugin_cmd.rs` | adopt | `GB-3AF4-CORE` |
| 175 | `M` `crates/codegen/xai-grok-pager/src/scrollback/block.rs` | adopt | `GB-3AF4-CORE` |
| 176 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/mod.rs` | adopt | `GB-3AF4-CORE` |
| 177 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/session_event.rs` | adopt | `GB-3AF4-TURN-QUEUE` |
| 178 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/edit.rs` | adopt | `GB-3AF4-CORE` |
| 179 | `A` `crates/codegen/xai-grok-pager/src/scrollback/blocks/workflow.rs` | adopt | `GB-3AF4-WORKFLOWS` |
| 180 | `M` `crates/codegen/xai-grok-pager/src/scrollback/state/mod.rs` | adopt | `GB-3AF4-CORE` |
| 181 | `M` `crates/codegen/xai-grok-pager/src/scrollback/wrappers/entry_renderer.rs` | adopt | `GB-3AF4-CORE` |
| 182 | `M` `crates/codegen/xai-grok-pager/src/settings/defs.rs` | adopt | `GB-3AF4-CORE` |
| 183 | `M` `crates/codegen/xai-grok-pager/src/settings/registry.rs` | adopt | `GB-3AF4-CORE` |
| 184 | `M` `crates/codegen/xai-grok-pager/src/slash/acp_command.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 185 | `M` `crates/codegen/xai-grok-pager/src/slash/command.rs` | adopt | `GB-3AF4-CORE` |
| 186 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/always_approve.rs` | adopt | `GB-3AF4-CORE` |
| 187 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/announcements.rs` | adopt | `GB-3AF4-CORE` |
| 188 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/auto.rs` | adopt | `GB-3AF4-CORE` |
| 189 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/cd.rs` | adopt | `GB-3AF4-CORE` |
| 190 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/copy.rs` | adopt | `GB-3AF4-CORE` |
| 191 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/dashboard.rs` | adopt | `GB-3AF4-BACKGROUND-DASHBOARD` |
| 192 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/debug.rs` | adopt | `GB-3AF4-CORE` |
| 193 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/docs.rs` | adopt | `GB-3AF4-CORE` |
| 194 | `A` `crates/codegen/xai-grok-pager/src/slash/commands/doctor.rs` | adopt | `GB-3AF4-DOCTOR` |
| 195 | `A` `crates/codegen/xai-grok-pager/src/slash/commands/edit_prompt.rs` | adopt | `GB-3AF4-EXTERNAL-EDITOR` |
| 196 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/effort.rs` | adopt | `GB-3AF4-MAX-EFFORT` |
| 197 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/effort_levels.rs` | adopt | `GB-3AF4-MAX-EFFORT` |
| 198 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/expand.rs` | adopt | `GB-3AF4-CORE` |
| 199 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/export.rs` | adopt | `GB-3AF4-CORE` |
| 200 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/find.rs` | adopt | `GB-3AF4-CORE` |
| 201 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/fork.rs` | adopt | `GB-3AF4-CORE` |
| 202 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/help.rs` | adopt | `GB-3AF4-CORE` |
| 203 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/history.rs` | adopt | `GB-3AF4-CORE` |
| 204 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/jump.rs` | adopt | `GB-3AF4-CORE` |
| 205 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/loop_cmd.rs` | adopt | `GB-3AF4-BACKGROUND-DASHBOARD` |
| 206 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/mod.rs` | adopt | `GB-3AF4-CORE` |
| 207 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/model.rs` | adopt | `GB-3AF4-CORE` |
| 208 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/multiline.rs` | adopt | `GB-3AF4-CORE` |
| 209 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/plan.rs` | adopt | `GB-3AF4-CORE` |
| 210 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/privacy.rs` | adopt | `GB-3AF4-CORE` |
| 211 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/queue.rs` | adopt | `GB-3AF4-TURN-QUEUE` |
| 212 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/screen_mode_switch.rs` | adopt | `GB-3AF4-CORE` |
| 213 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/settings_cmd.rs` | adopt | `GB-3AF4-CORE` |
| 214 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/tasks.rs` | adopt | `GB-3AF4-CORE` |
| 215 | `D` `crates/codegen/xai-grok-pager/src/slash/commands/terminal_setup.rs` | adopt | `GB-3AF4-TERMINAL` |
| 216 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/theme.rs` | adopt | `GB-3AF4-TERMINAL` |
| 217 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/toggle_mouse_reporting.rs` | adopt | `GB-3AF4-CORE` |
| 218 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/transcript.rs` | adopt | `GB-3AF4-CORE` |
| 219 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/usage.rs` | adopt | `GB-3AF4-USAGE` |
| 220 | `A` `crates/codegen/xai-grok-pager/src/slash/commands/workflows.rs` | adopt | `GB-3AF4-WORKFLOWS` |
| 221 | `M` `crates/codegen/xai-grok-pager/src/slash/mod.rs` | adopt | `GB-3AF4-CORE` |
| 222 | `M` `crates/codegen/xai-grok-pager/src/slash/registry.rs` | adopt | `GB-3AF4-CORE` |
| 223 | `M` `crates/codegen/xai-grok-pager/src/views/agent.rs` | adopt | `GB-3AF4-CORE` |
| 224 | `M` `crates/codegen/xai-grok-pager/src/views/agent_status.rs` | adopt | `GB-3AF4-CORE` |
| 225 | `M` `crates/codegen/xai-grok-pager/src/views/dashboard/layout.rs` | adopt | `GB-3AF4-BACKGROUND-DASHBOARD` |
| 226 | `M` `crates/codegen/xai-grok-pager/src/views/dashboard/peek.rs` | adopt | `GB-3AF4-BACKGROUND-DASHBOARD` |
| 227 | `M` `crates/codegen/xai-grok-pager/src/views/dashboard/render.rs` | adopt | `GB-3AF4-BACKGROUND-DASHBOARD` |
| 228 | `M` `crates/codegen/xai-grok-pager/src/views/dashboard/row.rs` | adopt | `GB-3AF4-BACKGROUND-DASHBOARD` |
| 229 | `M` `crates/codegen/xai-grok-pager/src/views/dashboard/state.rs` | adopt | `GB-3AF4-BACKGROUND-DASHBOARD` |
| 230 | `M` `crates/codegen/xai-grok-pager/src/views/extensions_modal.rs` | adopt | `GB-3AF4-CORE` |
| 231 | `M` `crates/codegen/xai-grok-pager/src/views/goal_detail.rs` | adopt | `GB-3AF4-AUTO-FEEDBACK` |
| 232 | `M` `crates/codegen/xai-grok-pager/src/views/memory_modal.rs` | adopt | `GB-3AF4-CORE` |
| 233 | `M` `crates/codegen/xai-grok-pager/src/views/mod.rs` | adopt | `GB-3AF4-CORE` |
| 234 | `M` `crates/codegen/xai-grok-pager/src/views/modal.rs` | adopt | `GB-3AF4-CORE` |
| 235 | `M` `crates/codegen/xai-grok-pager/src/views/queue_pane.rs` | adopt | `GB-3AF4-TURN-QUEUE` |
| 236 | `M` `crates/codegen/xai-grok-pager/src/views/session_picker.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 237 | `M` `crates/codegen/xai-grok-pager/src/views/settings_modal/state.rs` | adopt | `GB-3AF4-CORE` |
| 238 | `M` `crates/codegen/xai-grok-pager/src/views/settings_modal/tests.rs` | adopt | `GB-3AF4-CORE` |
| 239 | `M` `crates/codegen/xai-grok-pager/src/views/shortcuts_help.rs` | adopt | `GB-3AF4-CORE` |
| 240 | `M` `crates/codegen/xai-grok-pager/src/views/tasks_pane.rs` | adopt | `GB-3AF4-BACKGROUND-DASHBOARD` |
| 241 | `M` `crates/codegen/xai-grok-pager/src/views/turn_status.rs` | adopt | `GB-3AF4-CORE` |
| 242 | `M` `crates/codegen/xai-grok-pager/src/views/welcome/mod.rs` | adopt | `GB-3AF4-CORE` |
| 243 | `A` `crates/codegen/xai-grok-pager/src/views/workflows.rs` | adopt | `GB-3AF4-WORKFLOWS` |
| 244 | `M` `crates/codegen/xai-grok-pager/src/voice/handle.rs` | adopt | `GB-3AF4-VOICE` |
| 245 | `A` `crates/codegen/xai-grok-pager/tests/doctor_early_dispatch.rs` | adopt | `GB-3AF4-DOCTOR` |
| 246 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/auto_wake_cancel_preserves_queued_user_prompt.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 247 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/background_task_reaped_on_quit.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 248 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/basename_path_demo_pty.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 249 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/bash_queued_mid_turn_drains_as_bash.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 250 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/cancel_discards_buffered_interjection.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 251 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/cancel_then_resend_prompt_appears_once.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 252 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/common.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 253 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/ctrlc_after_activity_no_rewind_prompt_once.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 254 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/ctrlc_with_queued_prompt_no_dup.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 255 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/drag_select_autoscroll_full_scrollout_copy_pty.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 256 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/edit_collapsed_oneliner_pty.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 257 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/edit_hl_inplace_refresh_pty.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 258 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/edit_interject_lone_queued_row_keeps_tui_alive.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 259 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/edit_merge_sequential_pty.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 260 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/empty_enter_force_sends_top_queued.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 261 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/empty_enter_sends_top_not_last_of_two.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 262 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/endline_park_two_static_markers.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 263 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/endline_wakeups_are_markerless.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 264 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/interjection_reaches_model_ctrl_l_in_vscode_family.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 265 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/interjection_reaches_model_in_same_turn.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 266 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_commits_thinking_body_to_scrollback.rs` | adopt | `GB-3AF4-MINIMAL-RECAP` |
| 267 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_continue_reprints_transcript.rs` | adopt | `GB-3AF4-MINIMAL-RECAP` |
| 268 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_ctrl_o_send_now_queued_apple_terminal.rs` | adopt | `GB-3AF4-MINIMAL-RECAP` |
| 269 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_double_esc_committed_queued_prompt_single_render.rs` | adopt | `GB-3AF4-MINIMAL-RECAP` |
| 270 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_external_editor_round_trip.rs` | adopt | `GB-3AF4-EXTERNAL-EDITOR` |
| 271 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_flush_left_no_hpad.rs` | adopt | `GB-3AF4-MINIMAL-RECAP` |
| 272 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_lookup_commits_one_line_summary.rs` | adopt | `GB-3AF4-MINIMAL-RECAP` |
| 273 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_queue_indicator_shows_while_running.rs` | adopt | `GB-3AF4-MINIMAL-RECAP` |
| 274 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/mod.rs` | adopt | `GB-3AF4-MINIMAL-RECAP` |
| 275 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/page_flip_on_send_pty.rs` | adopt | `GB-3AF4-TURN-QUEUE` |
| 276 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/queue_and_interjection_lifecycle.rs` | adopt | `GB-3AF4-TURN-QUEUE` |
| 277 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/queued_bash_promotion_renders_output_pty.rs` | adopt | `GB-3AF4-TURN-QUEUE` |
| 278 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/queued_message_renders_once_not_twice.rs` | adopt | `GB-3AF4-TURN-QUEUE` |
| 279 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/read_tool_header_selection_copies_path_only_pty.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 280 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/reasoning_efforts_fallback_menu_matches_builtin.rs` | adopt | `GB-3AF4-MAX-EFFORT` |
| 281 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/reasoning_efforts_from_config_toml_menu.rs` | adopt | `GB-3AF4-MAX-EFFORT` |
| 282 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/removed_queued_prompt_never_sent.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 283 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/reparked_wait_repushes_buried_marker.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 284 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/scroll.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 285 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/send_now_tip_after_mid_turn_queue.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 286 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/show_thinking_blocks_toggle_hides_existing_pty.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 287 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/spinner_reappears_after_wait_resumes.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 288 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/stuck_drag_recovers_on_esc_pty.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 289 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/verb_group_fold_expand_collapse_pty.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 290 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/verb_group_header_drag_copy_pty.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 291 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/verb_group_settings_toggle_pty.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 292 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/verb_group_streaming_fold_pty.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 293 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/verb_group_thinking_fold_pty.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 294 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/verify_bashq_claim2_force_interject.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 295 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/verify_bashq_claim3_edit_keeps_bash.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 296 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/wheel_overscroll_at_bottom_reengages_follow_mid_stream.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 297 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/wheel_scrolls_viewport_during_streaming_turn.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 298 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e_clipboard.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 299 | `M` `crates/codegen/xai-grok-pager/tests/pty_xtversion.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 300 | `M` `crates/codegen/xai-grok-pager/tests/settings_e2e.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 301 | `M` `crates/codegen/xai-grok-plugin-marketplace/src/config.rs` | adopt | `GB-3AF4-CORE` |
| 302 | `M` `crates/codegen/xai-grok-sampler/src/actor/request_task.rs` | adopt | `GB-3AF4-RUNTIME` |
| 303 | `M` `crates/codegen/xai-grok-sampler/src/config.rs` | adopt | `GB-3AF4-RUNTIME` |
| 304 | `M` `crates/codegen/xai-grok-sampler/src/retry.rs` | adopt | `GB-3AF4-RUNTIME` |
| 305 | `M` `crates/codegen/xai-grok-sampler/src/stream/responses.rs` | adopt | `GB-3AF4-RUNTIME` |
| 306 | `M` `crates/codegen/xai-grok-sampling-types/src/conversation.rs` | adopt | `GB-3AF4-RUNTIME` |
| 307 | `M` `crates/codegen/xai-grok-sampling-types/src/error.rs` | adopt | `GB-3AF4-RUNTIME` |
| 308 | `M` `crates/codegen/xai-grok-sampling-types/src/types.rs` | adopt | `GB-3AF4-RUNTIME` |
| 309 | `M` `crates/codegen/xai-grok-sandbox/src/paths.rs` | adopt | `GB-3AF4-PERMISSIONS` |
| 310 | `M` `crates/codegen/xai-grok-sandbox/src/profiles.rs` | adopt | `GB-3AF4-PERMISSIONS` |
| 311 | `M` `crates/codegen/xai-grok-shared/src/clipboard.rs` | adopt | `GB-3AF4-TERMINAL` |
| 312 | `M` `crates/codegen/xai-grok-shared/src/ui_config.rs` | adopt | `GB-3AF4-CORE` |
| 313 | `M` `crates/codegen/xai-grok-shell-base/src/cpu_profile.rs` | adopt | `GB-3AF4-CORE` |
| 314 | `M` `crates/codegen/xai-grok-shell/CHANGELOG.md` | adopt | `GB-3AF4-DOCS` |
| 315 | `M` `crates/codegen/xai-grok-shell/Cargo.toml` | adopt | `GB-3AF4-BUILD` |
| 316 | `M` `crates/codegen/xai-grok-shell/benches/session_list.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 317 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.107.json` | adopt | `GB-3AF4-DOCS` |
| 318 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.107.md` | adopt | `GB-3AF4-DOCS` |
| 319 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.108.json` | adopt | `GB-3AF4-DOCS` |
| 320 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.108.md` | adopt | `GB-3AF4-DOCS` |
| 321 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.109.json` | adopt | `GB-3AF4-DOCS` |
| 322 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.109.md` | adopt | `GB-3AF4-DOCS` |
| 323 | `D` `crates/codegen/xai-grok-shell/skills/best-of-n/SKILL.md` | adopt | `GB-3AF4-SKILLS` |
| 324 | `D` `crates/codegen/xai-grok-shell/skills/check-work/SKILL.md` | adopt | `GB-3AF4-SKILLS` |
| 325 | `D` `crates/codegen/xai-grok-shell/skills/code-review/SKILL.md` | adopt | `GB-3AF4-SKILLS` |
| 326 | `D` `crates/codegen/xai-grok-shell/skills/create-skill/SKILL.md` | adopt | `GB-3AF4-SKILLS` |
| 327 | `D` `crates/codegen/xai-grok-shell/skills/help/SKILL.md` | adopt | `GB-3AF4-SKILLS` |
| 328 | `D` `crates/codegen/xai-grok-shell/skills/imagine/SKILL.md` | adopt | `GB-3AF4-SKILLS` |
| 329 | `M` `crates/codegen/xai-grok-shell/src/agent/app.rs` | adopt | `GB-3AF4-CORE` |
| 330 | `M` `crates/codegen/xai-grok-shell/src/agent/config.rs` | adopt | `GB-3AF4-CORE` |
| 331 | `M` `crates/codegen/xai-grok-shell/src/agent/config_model_override_parse.rs` | adopt | `GB-3AF4-CORE` |
| 332 | `M` `crates/codegen/xai-grok-shell/src/agent/ext_parsers.rs` | adopt | `GB-3AF4-CORE` |
| 333 | `M` `crates/codegen/xai-grok-shell/src/agent/init.rs` | adopt | `GB-3AF4-CORE` |
| 334 | `M` `crates/codegen/xai-grok-shell/src/agent/mod.rs` | adopt | `GB-3AF4-CORE` |
| 335 | `A` `crates/codegen/xai-grok-shell/src/agent/model_providers.rs` | adopt | `GB-3AF4-CUSTOM-AUTH` |
| 336 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/acp_agent.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 337 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs` | adopt | `GB-3AF4-CORE` |
| 338 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/mod.rs` | adopt | `GB-3AF4-CORE` |
| 339 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/subagent_coordinator.rs` | adopt | `GB-3AF4-BACKGROUND-DASHBOARD` |
| 340 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/tests.rs` | adopt | `GB-3AF4-CORE` |
| 341 | `M` `crates/codegen/xai-grok-shell/src/agent/relay.rs` | adopt | `GB-3AF4-CORE` |
| 342 | `M` `crates/codegen/xai-grok-shell/src/agent/session_config.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 343 | `M` `crates/codegen/xai-grok-shell/src/agent/session_registry_client.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 344 | `M` `crates/codegen/xai-grok-shell/src/agent/subagent/coordinator_lifecycle.rs` | adopt | `GB-3AF4-BACKGROUND-DASHBOARD` |
| 345 | `M` `crates/codegen/xai-grok-shell/src/agent/subagent/coordinator_query.rs` | adopt | `GB-3AF4-BACKGROUND-DASHBOARD` |
| 346 | `M` `crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs` | adopt | `GB-3AF4-BACKGROUND-DASHBOARD` |
| 347 | `M` `crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs` | adopt | `GB-3AF4-BACKGROUND-DASHBOARD` |
| 348 | `M` `crates/codegen/xai-grok-shell/src/agent/subagent/tests/mod.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 349 | `M` `crates/codegen/xai-grok-shell/src/agent/subagent/tests/rest.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 350 | `M` `crates/codegen/xai-grok-shell/src/auth/auth_provider.rs` | adopt | `GB-3AF4-CUSTOM-AUTH` |
| 351 | `M` `crates/codegen/xai-grok-shell/src/auth/external_auth.rs` | adopt | `GB-3AF4-CUSTOM-AUTH` |
| 352 | `M` `crates/codegen/xai-grok-shell/src/auth/manager.rs` | adopt | `GB-3AF4-CUSTOM-AUTH` |
| 353 | `M` `crates/codegen/xai-grok-shell/src/auth/refresh/external_refresher.rs` | adopt | `GB-3AF4-CUSTOM-AUTH` |
| 354 | `M` `crates/codegen/xai-grok-shell/src/auth/refresh/mod.rs` | adopt | `GB-3AF4-CUSTOM-AUTH` |
| 355 | `M` `crates/codegen/xai-grok-shell/src/builtin.rs` | adopt | `GB-3AF4-CORE` |
| 356 | `M` `crates/codegen/xai-grok-shell/src/claude_import.rs` | adopt | `GB-3AF4-CORE` |
| 357 | `M` `crates/codegen/xai-grok-shell/src/config/reloader.rs` | adopt | `GB-3AF4-CONFIG` |
| 358 | `M` `crates/codegen/xai-grok-shell/src/config/tests.rs` | adopt | `GB-3AF4-CONFIG` |
| 359 | `M` `crates/codegen/xai-grok-shell/src/config/watcher.rs` | adopt | `GB-3AF4-CONFIG` |
| 360 | `M` `crates/codegen/xai-grok-shell/src/extensions/feedback.rs` | adopt | `GB-3AF4-AUTO-FEEDBACK` |
| 361 | `M` `crates/codegen/xai-grok-shell/src/extensions/marketplace.rs` | adopt | `GB-3AF4-CORE` |
| 362 | `M` `crates/codegen/xai-grok-shell/src/extensions/mod.rs` | adopt | `GB-3AF4-CORE` |
| 363 | `M` `crates/codegen/xai-grok-shell/src/extensions/notification.rs` | adopt | `GB-3AF4-CORE` |
| 364 | `M` `crates/codegen/xai-grok-shell/src/extensions/rewind.rs` | adopt | `GB-3AF4-CORE` |
| 365 | `M` `crates/codegen/xai-grok-shell/src/extensions/session_admin.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 366 | `M` `crates/codegen/xai-grok-shell/src/extensions/skills.rs` | adopt | `GB-3AF4-SKILLS` |
| 367 | `A` `crates/codegen/xai-grok-shell/src/extensions/usage.rs` | adopt | `GB-3AF4-USAGE` |
| 368 | `M` `crates/codegen/xai-grok-shell/src/inspect/mod.rs` | adopt | `GB-3AF4-CORE` |
| 369 | `M` `crates/codegen/xai-grok-shell/src/leader/client.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 370 | `M` `crates/codegen/xai-grok-shell/src/leader/protocol.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 371 | `M` `crates/codegen/xai-grok-shell/src/remote/pull.rs` | adopt | `GB-3AF4-CORE` |
| 372 | `M` `crates/codegen/xai-grok-shell/src/session/acp_conversion.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 373 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 374 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/goal.rs` | adopt | `GB-3AF4-AUTO-FEEDBACK` |
| 375 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/goal_support.rs` | adopt | `GB-3AF4-AUTO-FEEDBACK` |
| 376 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/interjection.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 377 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/laziness.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 378 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/memory_dream.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 379 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/model_switch.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 380 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/notification_drain.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 381 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/prompt_queue.rs` | adopt | `GB-3AF4-SCHEDULER-QUEUE` |
| 382 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/recap.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 383 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/reminders.rs` | adopt | `GB-3AF4-SCHEDULER-QUEUE` |
| 384 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/run_loop.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 385 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 386 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/session_setup.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 387 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/slash_exec.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 388 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/spawn.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 389 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/stop_gate.rs` | adopt | `GB-3AF4-HOOKS` |
| 390 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/tasks_cancel.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 391 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/tool_calls.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 392 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/turn.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 393 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/turn_end.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 394 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/updates.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 395 | `A` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/workflow.rs` | adopt | `GB-3AF4-WORKFLOWS` |
| 396 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/auto_wake_suppression_tests.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 397 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/cancel_running_task_tests.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 398 | `D` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/goal/goal_backoff_tests.rs` | adopt | `GB-3AF4-AUTO-FEEDBACK` |
| 399 | `D` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/goal/goal_classifier_e2e_tests.rs` | adopt | `GB-3AF4-AUTO-FEEDBACK` |
| 400 | `D` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/goal/goal_reminder_subagent_rules_tests.rs` | adopt | `GB-3AF4-SCHEDULER-QUEUE` |
| 401 | `D` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/goal/goal_strategist_e2e_tests.rs` | adopt | `GB-3AF4-AUTO-FEEDBACK` |
| 402 | `D` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/goal/goal_summarizer_e2e_tests.rs` | adopt | `GB-3AF4-AUTO-FEEDBACK` |
| 403 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/idle_resume_tests.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 404 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/inline_auto_compact_flow_tests.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 405 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/memory_config_tests.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 406 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/prompt_queue_actor_tests.rs` | adopt | `GB-3AF4-SCHEDULER-QUEUE` |
| 407 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/recap_display_only_tests.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 408 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/reminder_policy_tests.rs` | adopt | `GB-3AF4-SCHEDULER-QUEUE` |
| 409 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/replay_buffer_send_update_tests.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 410 | `D` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/rewrite_zero_turn_prefix_tests.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 411 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/support.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 412 | `M` `crates/codegen/xai-grok-shell/src/session/agent_rebuild.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 413 | `M` `crates/codegen/xai-grok-shell/src/session/chat_persistence.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 414 | `M` `crates/codegen/xai-grok-shell/src/session/commands.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 415 | `M` `crates/codegen/xai-grok-shell/src/session/compaction.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 416 | `M` `crates/codegen/xai-grok-shell/src/session/feedback_manager.rs` | adopt | `GB-3AF4-AUTO-FEEDBACK` |
| 417 | `M` `crates/codegen/xai-grok-shell/src/session/goal_classifier.rs` | adopt | `GB-3AF4-AUTO-FEEDBACK` |
| 418 | `M` `crates/codegen/xai-grok-shell/src/session/goal_classifier/evidence.rs` | adopt | `GB-3AF4-AUTO-FEEDBACK` |
| 419 | `A` `crates/codegen/xai-grok-shell/src/session/goal_evaluator.rs` | adopt | `GB-3AF4-AUTO-FEEDBACK` |
| 420 | `M` `crates/codegen/xai-grok-shell/src/session/goal_orchestrator.rs` | adopt | `GB-3AF4-AUTO-FEEDBACK` |
| 421 | `M` `crates/codegen/xai-grok-shell/src/session/goal_planner.rs` | adopt | `GB-3AF4-AUTO-FEEDBACK` |
| 422 | `M` `crates/codegen/xai-grok-shell/src/session/goal_role_tools.rs` | adopt | `GB-3AF4-AUTO-FEEDBACK` |
| 423 | `M` `crates/codegen/xai-grok-shell/src/session/goal_strategist.rs` | adopt | `GB-3AF4-AUTO-FEEDBACK` |
| 424 | `M` `crates/codegen/xai-grok-shell/src/session/goal_summarizer.rs` | adopt | `GB-3AF4-AUTO-FEEDBACK` |
| 425 | `M` `crates/codegen/xai-grok-shell/src/session/goal_tracker.rs` | adopt | `GB-3AF4-AUTO-FEEDBACK` |
| 426 | `M` `crates/codegen/xai-grok-shell/src/session/handle.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 427 | `M` `crates/codegen/xai-grok-shell/src/session/helpers/compaction_context.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 428 | `M` `crates/codegen/xai-grok-shell/src/session/helpers/memory_flush.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 429 | `M` `crates/codegen/xai-grok-shell/src/session/helpers/replay.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 430 | `M` `crates/codegen/xai-grok-shell/src/session/helpers/session_compact.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 431 | `M` `crates/codegen/xai-grok-shell/src/session/helpers/session_recap.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 432 | `M` `crates/codegen/xai-grok-shell/src/session/merge.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 433 | `M` `crates/codegen/xai-grok-shell/src/session/mod.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 434 | `M` `crates/codegen/xai-grok-shell/src/session/persistence.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 435 | `M` `crates/codegen/xai-grok-shell/src/session/prompt_queue.rs` | adopt | `GB-3AF4-SCHEDULER-QUEUE` |
| 436 | `M` `crates/codegen/xai-grok-shell/src/session/slash_commands.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 437 | `M` `crates/codegen/xai-grok-shell/src/session/storage/jsonl/durable_tests.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 438 | `M` `crates/codegen/xai-grok-shell/src/session/storage/jsonl/mod.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 439 | `M` `crates/codegen/xai-grok-shell/src/session/storage/jsonl/tests.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 440 | `M` `crates/codegen/xai-grok-shell/src/session/storage/mod.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 441 | `A` `crates/codegen/xai-grok-shell/src/session/storage/relocation/fs.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 442 | `A` `crates/codegen/xai-grok-shell/src/session/storage/relocation/journal.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 443 | `A` `crates/codegen/xai-grok-shell/src/session/storage/relocation/mod.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 444 | `A` `crates/codegen/xai-grok-shell/src/session/storage/relocation/tests.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 445 | `M` `crates/codegen/xai-grok-shell/src/session/storage/search.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 446 | `M` `crates/codegen/xai-grok-shell/src/session/storage/summary_write.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 447 | `M` `crates/codegen/xai-grok-shell/src/session/templates/goal_continuation_directive.md` | adopt | `GB-3AF4-AUTO-FEEDBACK` |
| 448 | `A` `crates/codegen/xai-grok-shell/src/session/templates/goal_continuation_directive_legacy.md` | adopt | `GB-3AF4-AUTO-FEEDBACK` |
| 449 | `M` `crates/codegen/xai-grok-shell/src/session/templates/goal_rules.md` | adopt | `GB-3AF4-AUTO-FEEDBACK` |
| 450 | `A` `crates/codegen/xai-grok-shell/src/session/templates/goal_rules_legacy.md` | adopt | `GB-3AF4-AUTO-FEEDBACK` |
| 451 | `M` `crates/codegen/xai-grok-shell/src/session/unified_list/mod.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 452 | `A` `crates/codegen/xai-grok-shell/src/session/workflow/host_service.rs` | adopt | `GB-3AF4-WORKFLOWS` |
| 453 | `A` `crates/codegen/xai-grok-shell/src/session/workflow/manager.rs` | adopt | `GB-3AF4-WORKFLOWS` |
| 454 | `A` `crates/codegen/xai-grok-shell/src/session/workflow/mod.rs` | adopt | `GB-3AF4-WORKFLOWS` |
| 455 | `A` `crates/codegen/xai-grok-shell/src/session/workflow/notify.rs` | adopt | `GB-3AF4-WORKFLOWS` |
| 456 | `A` `crates/codegen/xai-grok-shell/src/session/workflow/registry.rs` | adopt | `GB-3AF4-WORKFLOWS` |
| 457 | `A` `crates/codegen/xai-grok-shell/src/session/workflow/schema_contract.rs` | adopt | `GB-3AF4-WORKFLOWS` |
| 458 | `A` `crates/codegen/xai-grok-shell/src/session/workflow/store.rs` | adopt | `GB-3AF4-WORKFLOWS` |
| 459 | `A` `crates/codegen/xai-grok-shell/src/session/workflow/tracker.rs` | adopt | `GB-3AF4-WORKFLOWS` |
| 460 | `A` `crates/codegen/xai-grok-shell/src/session/workflows/deep_research.rhai` | adopt | `GB-3AF4-WORKFLOWS` |
| 461 | `M` `crates/codegen/xai-grok-shell/src/test_support/lsp_runtime.rs` | adopt | `GB-3AF4-CORE` |
| 462 | `M` `crates/codegen/xai-grok-shell/src/tools/notification_bridge.rs` | adopt | `GB-3AF4-CORE` |
| 463 | `M` `crates/codegen/xai-grok-shell/src/tools/tool_context.rs` | adopt | `GB-3AF4-CORE` |
| 464 | `M` `crates/codegen/xai-grok-shell/src/trace_classifier/mod.rs` | adopt | `GB-3AF4-CORE` |
| 465 | `D` `crates/codegen/xai-grok-shell/src/upload/config_files.rs` | adopt | `GB-3AF4-CORE` |
| 466 | `M` `crates/codegen/xai-grok-shell/src/upload/gcs.rs` | adopt | `GB-3AF4-CORE` |
| 467 | `M` `crates/codegen/xai-grok-shell/src/upload/manifest.rs` | adopt | `GB-3AF4-CORE` |
| 468 | `M` `crates/codegen/xai-grok-shell/src/upload/mod.rs` | adopt | `GB-3AF4-CORE` |
| 469 | `M` `crates/codegen/xai-grok-shell/src/upload/trace.rs` | adopt | `GB-3AF4-CORE` |
| 470 | `M` `crates/codegen/xai-grok-shell/src/upload/turn.rs` | adopt | `GB-3AF4-CORE` |
| 471 | `M` `crates/codegen/xai-grok-shell/src/util/config/mcp.rs` | adopt | `GB-3AF4-CONFIG` |
| 472 | `M` `crates/codegen/xai-grok-shell/src/util/config/mod.rs` | adopt | `GB-3AF4-CONFIG` |
| 473 | `M` `crates/codegen/xai-grok-shell/src/util/config/settings_writes.rs` | adopt | `GB-3AF4-CONFIG` |
| 474 | `M` `crates/codegen/xai-grok-shell/src/util/config/worktree.rs` | adopt | `GB-3AF4-WORKTREE` |
| 475 | `M` `crates/codegen/xai-grok-shell/src/util/mod.rs` | adopt | `GB-3AF4-CORE` |
| 476 | `A` `crates/codegen/xai-grok-shell/src/util/subprocess.rs` | adopt | `GB-3AF4-CORE` |
| 477 | `A` `crates/codegen/xai-grok-shell/src/util/user_identity.rs` | adopt | `GB-3AF4-CORE` |
| 478 | `M` `crates/codegen/xai-grok-shell/tests/test_doom_loop_recovery.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 479 | `M` `crates/codegen/xai-grok-shell/tests/test_leader_stdio_integration.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 480 | `M` `crates/codegen/xai-grok-subagent-resolution/src/lib.rs` | adopt | `GB-3AF4-BACKGROUND-DASHBOARD` |
| 481 | `M` `crates/codegen/xai-grok-subagent-resolution/src/overrides.rs` | adopt | `GB-3AF4-BACKGROUND-DASHBOARD` |
| 482 | `M` `crates/codegen/xai-grok-test-support/README.md` | adopt | `GB-3AF4-TEST-INFRA` |
| 483 | `M` `crates/codegen/xai-grok-test-support/src/acp_client.rs` | adopt | `GB-3AF4-SESSIONS-ACP` |
| 484 | `M` `crates/codegen/xai-grok-test-support/src/headless.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 485 | `M` `crates/codegen/xai-grok-test-support/src/lib.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 486 | `M` `crates/codegen/xai-grok-test-support/src/sse.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 487 | `M` `crates/codegen/xai-grok-tools-api/src/config_validation.rs` | adopt | `GB-3AF4-CORE` |
| 488 | `M` `crates/codegen/xai-grok-tools-api/src/slash_commands.rs` | adopt | `GB-3AF4-CORE` |
| 489 | `M` `crates/codegen/xai-grok-tools/schema/tool_meta.schema.json` | adopt | `GB-3AF4-CORE` |
| 490 | `M` `crates/codegen/xai-grok-tools/src/bridge.rs` | adopt | `GB-3AF4-CORE` |
| 491 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/mod.rs` | adopt | `GB-3AF4-CORE` |
| 492 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/read_file/mod.rs` | adopt | `GB-3AF4-SKILLS` |
| 493 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/scheduler/actor.rs` | adopt | `GB-3AF4-SCHEDULER-QUEUE` |
| 494 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/scheduler/create.rs` | adopt | `GB-3AF4-SCHEDULER-QUEUE` |
| 495 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/scheduler/delete.rs` | adopt | `GB-3AF4-SCHEDULER-QUEUE` |
| 496 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/scheduler/list.rs` | adopt | `GB-3AF4-SCHEDULER-QUEUE` |
| 497 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/scheduler/types.rs` | adopt | `GB-3AF4-SCHEDULER-QUEUE` |
| 498 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/backend.rs` | adopt | `GB-3AF4-CORE` |
| 499 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs` | adopt | `GB-3AF4-CORE` |
| 500 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/types.rs` | adopt | `GB-3AF4-CORE` |
| 501 | `A` `crates/codegen/xai-grok-tools/src/implementations/grok_build/workflow/mod.rs` | adopt | `GB-3AF4-WORKFLOWS` |
| 502 | `M` `crates/codegen/xai-grok-tools/src/implementations/lsp/types.rs` | adopt | `GB-3AF4-CORE` |
| 503 | `M` `crates/codegen/xai-grok-tools/src/normalization.rs` | adopt | `GB-3AF4-CORE` |
| 504 | `M` `crates/codegen/xai-grok-tools/src/notification/handle.rs` | adopt | `GB-3AF4-CORE` |
| 505 | `M` `crates/codegen/xai-grok-tools/src/notification/handle_tests.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 506 | `M` `crates/codegen/xai-grok-tools/src/notification/types.rs` | adopt | `GB-3AF4-CORE` |
| 507 | `M` `crates/codegen/xai-grok-tools/src/registry/proto_convert.rs` | adopt | `GB-3AF4-CORE` |
| 508 | `M` `crates/codegen/xai-grok-tools/src/registry/types.rs` | adopt | `GB-3AF4-CORE` |
| 509 | `M` `crates/codegen/xai-grok-tools/src/reminders/task_completion.rs` | adopt | `GB-3AF4-SCHEDULER-QUEUE` |
| 510 | `M` `crates/codegen/xai-grok-tools/src/tool_taxonomy.rs` | adopt | `GB-3AF4-CORE` |
| 511 | `M` `crates/codegen/xai-grok-tools/src/types/output.rs` | adopt | `GB-3AF4-CORE` |
| 512 | `M` `crates/codegen/xai-grok-tools/src/types/tool.rs` | adopt | `GB-3AF4-CORE` |
| 513 | `M` `crates/codegen/xai-grok-tools/src/types/tool_io.rs` | adopt | `GB-3AF4-CORE` |
| 514 | `M` `crates/codegen/xai-grok-version/Cargo.toml` | adopt | `GB-3AF4-BUILD` |
| 515 | `M` `crates/codegen/xai-grok-voice/src/audio/capture.rs` | adopt | `GB-3AF4-VOICE` |
| 516 | `M` `crates/codegen/xai-grok-voice/src/audio/capture_linux.rs` | adopt | `GB-3AF4-VOICE` |
| 517 | `M` `crates/codegen/xai-grok-voice/src/audio/mod.rs` | adopt | `GB-3AF4-VOICE` |
| 518 | `M` `crates/codegen/xai-grok-voice/src/event.rs` | adopt | `GB-3AF4-VOICE` |
| 519 | `M` `crates/codegen/xai-grok-voice/src/lib.rs` | adopt | `GB-3AF4-VOICE` |
| 520 | `A` `crates/codegen/xai-grok-voice/src/pcm.rs` | adopt | `GB-3AF4-VOICE` |
| 521 | `M` `crates/codegen/xai-grok-voice/src/pipeline.rs` | adopt | `GB-3AF4-VOICE` |
| 522 | `M` `crates/codegen/xai-grok-voice/src/probe.rs` | adopt | `GB-3AF4-VOICE` |
| 523 | `M` `crates/codegen/xai-grok-workspace/src/capability.rs` | adopt | `GB-3AF4-CORE` |
| 524 | `M` `crates/codegen/xai-grok-workspace/src/folder_trust.rs` | adopt | `GB-3AF4-PERMISSIONS` |
| 525 | `M` `crates/codegen/xai-grok-workspace/src/handle.rs` | adopt | `GB-3AF4-CORE` |
| 526 | `M` `crates/codegen/xai-grok-workspace/src/hub_server.rs` | adopt | `GB-3AF4-CORE` |
| 527 | `M` `crates/codegen/xai-grok-workspace/src/permission/auto_mode.rs` | adopt | `GB-3AF4-PERMISSIONS` |
| 528 | `M` `crates/codegen/xai-grok-workspace/src/permission/bash_command_splitting.rs` | adopt | `GB-3AF4-PERMISSIONS` |
| 529 | `A` `crates/codegen/xai-grok-workspace/src/permission/exec_risk.rs` | adopt | `GB-3AF4-PERMISSIONS` |
| 530 | `M` `crates/codegen/xai-grok-workspace/src/permission/manager.rs` | adopt | `GB-3AF4-PERMISSIONS` |
| 531 | `M` `crates/codegen/xai-grok-workspace/src/permission/mod.rs` | adopt | `GB-3AF4-PERMISSIONS` |
| 532 | `M` `crates/codegen/xai-grok-workspace/src/permission/policy.rs` | adopt | `GB-3AF4-PERMISSIONS` |
| 533 | `M` `crates/codegen/xai-grok-workspace/src/permission/resolution.rs` | adopt | `GB-3AF4-PERMISSIONS` |
| 534 | `M` `crates/codegen/xai-grok-workspace/src/permission/shell_access.rs` | adopt | `GB-3AF4-PERMISSIONS` |
| 535 | `M` `crates/codegen/xai-grok-workspace/src/preview_supervisor.rs` | adopt | `GB-3AF4-CORE` |
| 536 | `M` `crates/codegen/xai-grok-workspace/src/worktree/mod.rs` | adopt | `GB-3AF4-WORKTREE` |
| 537 | `M` `crates/codegen/xai-hunk-tracker/src/actor/file_utils.rs` | adopt | `GB-3AF4-CORE` |
| 538 | `M` `crates/codegen/xai-prompt-queue/Cargo.toml` | adopt | `GB-3AF4-BUILD` |
| 539 | `A` `crates/codegen/xai-prompt-queue/src/combine.rs` | adopt | `GB-3AF4-SCHEDULER-QUEUE` |
| 540 | `M` `crates/codegen/xai-prompt-queue/src/lib.rs` | adopt | `GB-3AF4-SCHEDULER-QUEUE` |
| 541 | `M` `crates/codegen/xai-prompt-queue/src/types.rs` | adopt | `GB-3AF4-SCHEDULER-QUEUE` |
| 542 | `M` `crates/codegen/xai-ratatui-textarea/examples/textarea_demo.rs` | adopt | `GB-3AF4-CORE` |
| 543 | `M` `crates/codegen/xai-ratatui-textarea/src/textarea.rs` | adopt | `GB-3AF4-CORE` |
| 544 | `A` `crates/codegen/xai-workflow/Cargo.toml` | adopt | `GB-3AF4-BUILD` |
| 545 | `A` `crates/codegen/xai-workflow/examples/validate.rs` | adopt | `GB-3AF4-WORKFLOWS` |
| 546 | `A` `crates/codegen/xai-workflow/src/engine.rs` | adopt | `GB-3AF4-WORKFLOWS` |
| 547 | `A` `crates/codegen/xai-workflow/src/host.rs` | adopt | `GB-3AF4-WORKFLOWS` |
| 548 | `A` `crates/codegen/xai-workflow/src/journal.rs` | adopt | `GB-3AF4-WORKFLOWS` |
| 549 | `A` `crates/codegen/xai-workflow/src/lib.rs` | adopt | `GB-3AF4-WORKFLOWS` |
| 550 | `A` `crates/codegen/xai-workflow/src/meta.rs` | adopt | `GB-3AF4-WORKFLOWS` |
| 551 | `A` `crates/codegen/xai-workflow/src/run.rs` | adopt | `GB-3AF4-WORKFLOWS` |
| 552 | `A` `crates/codegen/xai-workflow/src/validate.rs` | adopt | `GB-3AF4-WORKFLOWS` |
| 553 | `M` `crates/common/xai-computer-hub-mcp-adapter/src/bridge.rs` | adopt | `GB-3AF4-CORE` |
| 554 | `M` `crates/common/xai-computer-hub-sdk/src/notification.rs` | adopt | `GB-3AF4-CORE` |
| 555 | `M` `crates/common/xai-tool-protocol/tests/identifier_validation.rs` | adopt | `GB-3AF4-TEST-INFRA` |
| 556 | `M` `crates/common/xai-tool-runtime/src/render.rs` | adopt | `GB-3AF4-CORE` |
| 557 | `M` `crates/common/xai-tool-runtime/tests/error_conversion.rs` | adopt | `GB-3AF4-TEST-INFRA` |

The ledger contains each raw added, modified, or deleted path between the exact reviewed and pinned Grok trees exactly once. No path is temporarily deferred. The final acknowledgement declaration binds this file by SHA-256 only after implementation, provenance, manifest ownership, and validation are committed on the prospective first-parent history.
