# Upstream refresh parity ledger — 2026-07-25

This ledger records the immutable fetch pins, complete advanced-source commit/path inventories, observable-behavior decisions, downstream thread-exhaustion repair, provider compatibility adaptations, and open review obligations for the 2026-07-25 refresh. It continues [`docs/upstream-refresh-2026-07-21-r3.md`](upstream-refresh-2026-07-21-r3.md). Fetched reference history is evidence only; no upstream tree was content-merged or rebased into Enhanced.

The reviewed revisions intentionally remain unchanged. The pinned Grok snapshot has **91 open atomic `adopt` obligations**, so it is not eligible for a zero-tree-delta acknowledgement. The authoritative 99-row inventory is recorded in [`upstream-refresh-2026-07-25-grok-behaviors.md`](upstream-refresh-2026-07-25-grok-behaviors.md). Codex/OpenCode have explicit proof obligations, and Kimi video prompts retain one stable temporary deferral. `latest_fetched` is an independent review queue, not an ancestry claim.

## Immutable boundary

- Pre-refresh candidate: `060cb355811d43854963bc6907e3c7fe0e17289e`, tree `c38bdf1e5117c8b6834b3c1c3b1ae3be15e709f5`.
- Isolated branch/worktree: `refresh/upstreams-20260725-thread-spawn` / `../grok-build-enhanced-refresh-20260725`.
- Fetch timestamp: `2026-07-25T12:08:06Z`.
- Protected primary-checkout `AGENTS.md` SHA-256: `e5467bd0ffd740f690390090a2ff416958e7f3ff94fb849c546d80d217db1639`.
- No merge, rebase, acknowledgement marker, push, tag, release, PR mutation, or Homebrew mutation was performed in this refresh.

## All 12 immutable source pins

| Source | Reviewed commit / tree | Latest fetched commit / tree | Range | Changed records | Ancestor |
| --- | --- | --- | ---: | ---: | --- |
| Grok Build | `3af4d5d39897855bdcc74f23e690024a5dc05573` / `e595174931be9bfb490aacf149e2c9cc0ca0ebba` | `6e386420825bd44ae648c63e7c8cba12fcec9401` / `3db5a3bd92232bb54581fb8701c6ec79ba48293d` | 3 commits | 673 name-status; 676 raw | yes |
| OpenAI Codex | `51200321eb7b862a29ffceaba8b19db1934a9b38` / `f776ca65baecd8157602572803d41ec92be9d7ab` | `4c43465133428898aa84f0bfc02c306ed65fb66a` / `0595b8dcf1e7da753bd10970c3cdac8eb6d64361` | 168 commits | 1,072 name-status; 1,074 raw | yes |
| OpenCode | `0317531906d3f3bb01cf33c16319870cfde9170c` / `e9344f8affc0b7f5f0537cb6c3ac09852d05f53a` | `7534d23551f665e65080809975b4ca5c7d63807b` / `4dad6b62a5b3855bb330726f3d357b1cc21cfa85` | 64 commits | 246 | yes |
| OpenCode Codex auth | `bec2ad69b252ef4ad7dd33b9532ff8b4fdb6d016` / `1da59bae7069563b2817143567b57c78e5758300` | `bec2ad69b252ef4ad7dd33b9532ff8b4fdb6d016` / `1da59bae7069563b2817143567b57c78e5758300` | 0 | 0 | unchanged |
| Warp themes | `b385044250f1ed3c9379ab34a8fe82f02fdffaa4` / `c678999ab9ef62cfba1d70822b117db727d06f8a` | `b385044250f1ed3c9379ab34a8fe82f02fdffaa4` / `c678999ab9ef62cfba1d70822b117db727d06f8a` | 0 | 0 | unchanged |
| Kimi Code | `b5efba7abcaf4041f81ec520097a61e6546e8c50` / `26080b7faeed49746ea11bec5a92d0fa23a4e189` | `c497af60e6cd20aab05e590f98a28fb15dd3491d` / `758fe6fecf803ed812076f5804638adbd24b59af` | 26 commits | 532 name-status; 547 raw | yes |
| Kimi CLI | `4a550effdfcb29a25a5d325bf935296cc50cd417` / `96cf7e65f10325b717db982effd32b3cec55a331` | `4a550effdfcb29a25a5d325bf935296cc50cd417` / `96cf7e65f10325b717db982effd32b3cec55a331` | 0 | 0 | unchanged |
| Z.AI Python SDK | `ca5109c0aa9bf173839be391b4b14aeadf9a9bf9` / `1fa4d15e067bf48dbb6dd69ca3b2e4143d5f242a` | `ca5109c0aa9bf173839be391b4b14aeadf9a9bf9` / `1fa4d15e067bf48dbb6dd69ca3b2e4143d5f242a` | 0 | 0 | unchanged |
| Z.AI coding plugins | `0446d0bb0bc537d97d3ab3664c4b8b9c4a0e1254` / `efea84479dc67bc4af7d2c3b59b4aca8f5332899` | `0446d0bb0bc537d97d3ab3664c4b8b9c4a0e1254` / `efea84479dc67bc4af7d2c3b59b4aca8f5332899` | 0 | 0 | unchanged |
| GLM-5 | `436efa09bc868a6922e307624189e7018406beb9` / `8ac85a6098dc83ebd539a9093442e8192fbf052c` | `436efa09bc868a6922e307624189e7018406beb9` / `8ac85a6098dc83ebd539a9093442e8192fbf052c` | 0 | 0 | unchanged |
| CodexBar | `cc8da27cec92029a6435bfee4a703a719290234e` / `e41036396d949aed3c579a52af74b1b8bab780f6` | `cc8da27cec92029a6435bfee4a703a719290234e` / `e41036396d949aed3c579a52af74b1b8bab780f6` | 0 | 0 | unchanged |
| Z.AI usage browser | `54cd1f33a703c417f2492ee1f21f22b3633a43c4` / `08b00849b96c5883a265f4d4d43e2836d01cdd9d` | `54cd1f33a703c417f2492ee1f21f22b3633a43c4` / `08b00849b96c5883a265f4d4d43e2836d01cdd9d` | 0 | 0 | unchanged |

The eight unchanged references introduce no new runtime provider, login flow, credential surface, theme corpus, research claim, or legal obligation in this pass.

## Thread-resource exhaustion investigation and repair

### Upstream status and root cause

- The pinned Grok target `6e386420825bd44ae648c63e7c8cba12fcec9401` does not repair either reported macOS error-35 path.
- Nucleo `0.5.0` remains pinned to revision `5b74652e482f7c07d827f18c6d21e7540c242c69`; its worker construction calls Rayon `build().expect("creating threadpool failed")`. Current Nucleo upstream still uses the same infallible construction pattern at audit time.
- Grok model prefetch/refresh used Tokio `spawn_blocking`; Tokio may panic when the blocking pool cannot create another worker. Production uses `panic = "abort"`, so `catch_unwind` cannot make either panic recoverable.
- Thread pressure was amplified by eager per-prompt file/history daemons, Nucleo private Rayon pools, parallel filesystem walkers, detached teardown overlap, and eager blocking-pool warmups.

### Downstream behavior

- File matching no longer constructs `Nucleo<T>` or its private Rayon pool. One lazy, fallibly-created daemon owns a synchronous `Matcher`/`MultiPattern`, sequential cancellable walking, query coalescing, deterministic ranking, and depth-one publication.
- File and history daemons exist only while their overlays are active. Failed creation returns no suggestions, logs a fixed-shape warning, and can be retried later; it never aborts the process.
- Startup/runtime model work uses named `std::thread::Builder::spawn` plus a oneshot result instead of Tokio’s infallible blocking-pool growth path. Failure retains current catalog/settings state.
- The shared reqwest blocking client is fallible because its internal runtime thread can itself fail under EAGAIN. Its xAI catalog DNS and Codex route-client DNS use one process-wide, fallibly-created resolver worker instead of Tokio’s infallible `getaddrinfo` blocking pool; resolver creation or lookup failure becomes an ordinary request error.
- macOS/Windows system-proxy discovery uses a named, fallibly-created worker rather than `spawn_blocking`. Failure returns a redacted proxy-resolution error, so optional catalog discovery retains cached/default state instead of aborting.
- File restart/query updates are atomic under one generation, and file/history update channels do not block the UI while sequential scoring or walking is active.
- Git, clipboard, browser-open, upload cleanup, HTTP warmup, persistent-agent startup, and leader-lock sites either warn and skip optional work or return a typed startup error for required work.
- Eager blocking-pool warmups and the extra early-prefetch timeout helper thread were removed. xAI, Codex, Kimi, Z.AI, and Custom provider identity/credentials remain unchanged by this provider-neutral repair.

Focused contract evidence includes synthetic `WouldBlock` construction failures for model, DNS, system-proxy, file, and history workers; synchronous fuzzy scoring without a worker pool; atomic restart/query generation; directory filtering before top-k; deterministic equal-score ordering; lazy/retryable overlay state; and binary compilation. No credential-bearing live call or thread-exhaustion stress process was used.

## Grok behavior and raw-path reconciliation

The authoritative atomic inventory contains **99 observable rows** in [`upstream-refresh-2026-07-25-grok-behaviors.md`](upstream-refresh-2026-07-25-grok-behaviors.md): **91 open `adopt`**, **7 closed `already equivalent`**, and **1 closed `not applicable`**. Those rows are classified separately from architecture and are the behavior-parity result for this pin.

The table below is only a coarse thematic map used by the 676-record raw-tree ledger. Its IDs group paths for ownership and review navigation; they are not atomic behaviors, and a thematic `adopt` can contain a mix of open and locally equivalent behavior. Any mixed cluster stays open conservatively. The atomic inventory, not this path map, controls acknowledgement eligibility.

| Thematic path ID | Upstream path cluster | Disposition | State | Review gate |
| --- | --- | --- | --- | --- |
| `GB-6E38-SESSION-LIFECYCLE` | Session creation failures, relocated/mirrored resume, import, and durable startup state surface errors instead of hanging. | **adopt** | **open** | Port the new lifecycle/error contract and cover disk-full, moved-root, import, and resume paths. |
| `GB-6E38-FORK-REWIND` | Forking after compaction preserves checkpoint lineage so later rewind remains valid. | **adopt** | **open** | Port checkpoint-lineage repair and add compact → fork → rewind coverage. |
| `GB-6E38-ACP-EVENTS` | ACP/session notifications, permission/auth events, request updates, and replay stay ordered and typed. | **adopt** | **open** | Reconcile ACP event additions with provider-neutral replay tests. |
| `GB-6E38-TOOL-MEDIA` | Tool callbacks and media controls include configurable image/video generation exposure and matching slash-command behavior. | **adopt** | **open** | Port tool/media gating without bypassing provider capabilities; add config/env and negative routing tests. |
| `GB-6E38-LEADER-STARTUP` | Leader/client startup, readiness, and remote model-selection handoff remain coherent under delayed initialization. | **adopt** | **open** | Port startup/readiness behavior and run leader reconnect/model-pick PTYs. |
| `GB-6E38-SUBAGENT-MCP` | Plugin subagents inherit the parent session’s connected MCP servers while remaining unable to declare privileged MCP/hooks/modes. | **adopt** | **open** | Implement bounded inheritance and prove parent-session ownership plus privilege negatives. |
| `GB-6E38-AUTH-RECOVERY` | Expired-token auto-compaction/login recovery retries the compact operation and original prompt; session info explains auth mode and management route. | **adopt** | **open** | Port provider-aware recovery and session-info presentation with cross-provider negative tests. |
| `GB-6E38-CUSTOM-GATEWAYS` | Custom gateway/provider configuration and auth lifecycle remain explicit across session rebuild and model selection. | **adopt** | **open** | Reconcile changed custom-provider behavior while preserving explicit credential sources and endpoints. |
| `GB-6E38-VERSION-UPDATES` | Version policy, changelog/build metadata, early startup warnings, and update routing reflect the current Grok behavior. | **adopt** | **open** | Adapt behavior while retaining Enhanced-owned release/update/provenance routes. |
| `GB-6E38-XAI-MODELS` | xAI catalog defaults, model selection, compaction metadata, and effort/tool capabilities follow the new upstream defaults. | **adopt** | **open** | Audit and port xAI-only defaults without changing Codex/Kimi/Custom catalogs. |
| `GB-6E38-PERMISSIONS-AUTO` | Permission focus and auto-classifier failures fall back to a normal prompt rather than silently denying. | **adopt** | **open** | Port fallback/focus behavior and add timeout/error/scrollback permission tests. |
| `GB-6E38-TOOL-OVERRIDES` | Tool overrides, tool availability, slash exposure, and managed tool metadata remain internally consistent. | **adopt** | **open** | Port override resolution and test provider/session/tool-set boundaries. |
| `GB-6E38-LOOP-GUARD` | A turn that repeats the exact same tool call many times terminates instead of looping forever. | **adopt** | **open** | Port the loop guard and test exact-repeat, changing-argument, and provider-neutral behavior. |
| `GB-6E38-BACKGROUND-SHELL` | `!cmd` and background shell work receive the intended one-hour timeout and correct completion/session ownership. | **adopt** | **open** | Port timeout and ownership behavior; add cancellation and unrelated-session negatives. |
| `GB-6E38-QUEUE-EDIT` | Queued-prompt combine/edit mode, newline insertion, and stable versioned row editing. | **already equivalent** | **closed** | Atomic evidence `GB-A572-035`, `GB-A572-039`, and `GB-69F0-016`; local queue edit and prompt-queue contracts are cited in `E3`. |
| `GB-6E38-SCHEDULER` | Scheduler persistence, wake admission, deletion, and durable task state remain coherent across restarts. | **adopt** | **open** | Reconcile scheduler changes and run durability/restart coverage. |
| `GB-6E38-REMINDERS` | Reminder generation and task-completion notices target the correct active session and turn. | **adopt** | **open** | Port reminder semantics and add multi-session delivery tests. |
| `GB-6E38-WORKFLOWS` | Workflow execution, tools, watcher updates, persistence, and TUI state follow current upstream behavior. | **adopt** | **open** | Reconcile workflow runtime/UI paths and run workflow persistence tests. |
| `GB-6E38-MARKETPLACE` | Removing MCP servers, plugins, or hook sources asks for confirmation and marketplace state remains consistent. | **adopt** | **open** | Port confirmation/state behavior and add cancel/confirm tests. |
| `GB-6E38-PROTECTED-HOOKS` | Managed/protected hooks and callback continuation preserve policy ownership and bounded execution. | **adopt** | **open** | Reconcile hook source protection and continuation tests. |
| `GB-6E38-WORKSPACE-ERRORS` | Workspace/session filesystem failures become visible typed errors rather than indefinite startup or operation hangs. | **adopt** | **open** | Port error projection and add disk/full/unavailable workspace tests. |
| `GB-6E38-APP-BUILDER` | App-builder/computer-hub status and tool callbacks project correct progress and completion state. | **adopt** | **open** | Reconcile status callbacks and add state-projection tests. |
| `GB-6E38-PRIVACY` | Privacy/telemetry banner and settings presentation follow the current upstream lifecycle. | **adopt** | **open** | Port the ordinary upstream choice surface without suppressing telemetry details. |
| `GB-6E38-DOCTOR` | `grok doctor fix` actions run from the TUI and startup warnings point to `/doctor`. | **adopt** | **open** | Port in-TUI fix dispatch with explicit confirmation and CLI/TUI tests. |
| `GB-6E38-TUTORIAL` | Getting-started/tutorial and welcome-state transitions reflect current controls and persisted completion. | **adopt** | **open** | Port tutorial/welcome behavior and add persisted-state tests. |
| `GB-6E38-ESC-CANCEL` | A single Esc cancels the running turn except in fullscreen vim scrollback mode. | **adopt** | **open** | Port mode-sensitive cancellation and run fullscreen/minimal/overlay PTYs. |
| `GB-6E38-SHORTCUTS` | Shortcut labels, mode hints, and keyboard help match actual bindings. | **adopt** | **open** | Reconcile labels and snapshot/PTY coverage. |
| `GB-6E38-MINIMAL-BASH` | Minimal-mode Bash chrome and command lifecycle remain concise while preserving cancellation and output. | **adopt** | **open** | Port minimal Bash rendering/lifecycle and run minimal PTYs. |
| `GB-6E38-DASHBOARD-TASKS` | Dashboard task rows, hover/click gaps, background status, and session ownership are accurate. | **adopt** | **open** | Port layout/hit-testing/state behavior and dashboard PTYs. |
| `GB-6E38-CLIPBOARD-CONFIRM` | Clipboard/copy actions and destructive-source removal confirmation provide accurate user feedback. | **adopt** | **open** | Reconcile clipboard confirmation/fallback behavior and terminal-specific tests. |
| `GB-6E38-VOICE-HELPER` | macOS voice capture runs in a temporary helper process to reduce resident memory while preserving diagnostics. | **adopt** | **open** | Port helper-process capture with permission/silence/cancellation tests. |
| `GB-6E38-CONFIG-COMPAT` | Both spellings of the workspace teleport-disable flag load and save correctly. | **adopt** | **open** | Port alias/canonical-write behavior with round-trip tests. |
| `GB-6E38-MCP-TIMEOUTS` | Managed MCP tools can run slow operations without the previous premature timeout. | **adopt** | **open** | Port timeout ownership and test bounded slow-call/cancellation behavior. |
| `GB-6E38-UNDO-REDO` | Prompt input undo/redo behavior remains coherent across ordinary editing. | **already equivalent** | **closed** | Existing Enhanced textarea/input behavior and tests already cover the observable contract. |
| `GB-6E38-NPM-DISTRIBUTION` | Official xAI npm postinstall now installs under `$GROK_HOME/bin`. | **not applicable** | **closed** | Enhanced releases only fork-owned native GitHub assets/Homebrew routes; official xAI npm distribution is outside scope. |

Atomic summary: **91 open `adopt`**, **7 closed `already equivalent`**, **1 closed `not applicable`**, **0 Grok temporary deferrals**, and **0 unclassified observable rows**. Open adoption is intentionally distinct from temporary deferral: these behaviors belong on preserved Grok surfaces and have not been accepted as reviewed. The coarse map above has **32 open**, **2 equivalent**, and **1 not-applicable** thematic clusters and must not be used as the behavior count.

## Provider-reference behavior inventory

### OpenAI Codex

| Evidence ID | Upstream behavior → local behavior | Outcome/state | Action or evidence |
| --- | --- | --- | --- |
| `CDX-4C43-ROUTED-AUTH` | OAuth login/refresh/revoke and ordinary Responses calls receive resolved proxy routing → provider-owned exact-URL client pools already route each endpoint and refuse redirects. | **already equivalent / open proof** | Add refresh, revoke, and normal Responses proxy integration proofs; never fall back to a generic client. |
| `CDX-4C43-ENT26` | Raw plan code `ent26` is displayed as Enterprise → raw credentials/usage remain lossless while the pager maps only the presentation string. | **adopt / closed** | `credit_bar` display test plus raw JWT credential and usage-deserialization tests. |
| `CDX-4C43-CATALOG` | Post-auth provider state is disposed/refetched so newly entitled models appear → Codex-only catalog refresh exists after ACP login, explicit selection, and auth-file changes. | **already equivalent / open proof** | Add login/auth-change → newly entitled model visibility with xAI-state negative coverage. |
| `CDX-4C43-ITEM-IDS` | Stable IDs are assigned to client-created Responses items across resume/compaction → Enhanced always rebuilds full HTTP history, sends no `previous_response_id`, and retains server reasoning/call IDs. | **already equivalent / open proof** | Add tool call/output → persist/resume → compact → next request proof independent of client item IDs. |
| `CDX-4C43-WS-RECOVERY` | WebSocket `previous_response_not_found` retries with a full request → Enhanced has no Codex WebSocket transport or stored continuation ID. | **not applicable / closed** | HTTP full-history architecture has no corresponding continuation state. |
| `CDX-4C43-SERIALIZATION` | Lower-clone serialization preserves identical Responses JSON. | **already equivalent / closed** | No observable wire change. |
| `CDX-4C43-CUSTOM-SEARCH` | Opted-in Custom providers use their own standalone-search endpoint/auth → not a Codex subscription feature. | **not applicable / closed** | Any future Custom search must default off and must never receive Codex credentials. |
| `CDX-4C43-RETRY` | Retry metadata preserves server delay and separates private details → local parser caps Retry-After, honors retry veto, redacts bodies, and separates auth/transport budgets. | **already equivalent / open proof** | Add complete HTTP 429 → bounded actor retry → redacted diagnostics and normal-turn cache-key stability across 401. |
| `CDX-4C43-CODE-MODE` | Responses Lite `code_mode_tool_names` metadata is emitted only with Codex V8/code-mode execution. | **not applicable / closed** | Do not advertise a runtime Enhanced does not embed or disclose external MCP inventory. |
| `CDX-4C43-MCP-REVISION` | MCP calls bind to the captured Codex tool-catalog/client revision. | **not applicable / closed** | Codex app MCP runtime is not imported; Grok MCP/session ownership remains authoritative. |
| `CDX-4C43-CATALOG-MATRIX` | Context, compaction threshold, reasoning, service tiers/Fast, Responses Lite visibility, and `comp_hash` remain unchanged in this range. | **already equivalent / closed** | Existing authenticated catalog and provider-isolation suites remain authoritative. |
| `CDX-4C43-HOSTED-TOOLS` | Hosted web/image clients use route-aware construction → local exact-URL Codex pools, canonical endpoint seals, dynamic auth, redirect refusal, and bounded recovery already apply. | **already equivalent / open proof** | Normal hosted-tool route proof remains part of credential-gated qualification. |

### OpenCode interoperability

| Evidence ID | Behavior | Outcome/state | Evidence/action |
| --- | --- | --- | --- |
| `OC-7534-CACHE-KEY` | SDK-specific camelCase/snake_case prompt-cache options. | **already equivalent / closed** | Enhanced bypasses SDK option naming and emits exact provider-scoped `prompt_cache_key`. |
| `OC-7534-AUTH-REFETCH` | Dispose/refetch provider state after API-key or OAuth authentication. | **already equivalent / open proof** | Local Codex-only refresh exists; close with the post-auth newly-entitled-model test above. |
| `OC-7534-APP` | Remaining OpenCode app/UI/cloud/Zen/catalog/Claude/MiniMax/docs/maintenance changes. | **not applicable / closed** | No Codex subscription auth, Responses, usage, compaction, hosted-tool, or retry contract change. |

### Kimi Code

| Evidence ID | Behavior | Outcome/state | Evidence/action |
| --- | --- | --- | --- |
| `KIMI-C497-CATALOG` | Catalog correctness, declared thinking levels, protocol/capability follow-ups. | **already equivalent / closed** | Authenticated Kimi discovery already preserves exact Kimi metadata and endpoint selection. |
| `KIMI-C497-REASONING` | Echo preserved thinking under the reasoning field actually used by the endpoint. | **adopt / closed** | Response normalization accepts `reasoning_content`, `reasoning_details`, or `reasoning`; the first string wins, the observed key is replayed across cheap clones, and credential-binding changes reset it. |
| `KIMI-VIDEO-001` | Prompt-attached video upload through `/files` with `purpose=video`, then `ms://<id>` prompt content. | **temporarily deferred / open** | See the complete stable deferral below. |
| `KIMI-C497-WEB-CONFIG` | `KIMI_WEB_*` configuration for Kimi app web services. | **not applicable / closed** | Enhanced uses explicit provider-hosted Kimi tools plus SSRF-screened fallback, not Kimi app service configuration. |
| `KIMI-C497-MCP-TIMEOUT` | Global MCP timeout settings in Kimi’s replacement engine. | **not applicable / closed** | Grok MCP ownership remains authoritative; Grok’s own timeout delta is tracked as `GB-6E38-MCP-TIMEOUTS`. |
| `KIMI-C497-SECONDARY` | Configurable secondary model for Kimi subagents. | **not applicable / closed** | Kimi replacement-agent architecture is not imported; Grok model/subagent routing remains explicit. |
| `KIMI-C497-PROVIDER-WRITES` | Kimi server provider write/import endpoints and models.dev registry import. | **not applicable / closed** | No arbitrary provider import or Kimi app-server control plane is exposed. |
| `KIMI-C497-ENGINE` | Permission/workspace/transcript/config/print-lifecycle/user-tool refactors. | **not applicable / closed** | Replacement engine behavior does not alter the direct Kimi provider adapter contract. |
| `KIMI-C497-UI-WEB` | Kimi TUI/Web copy/steering/tip behavior. | **not applicable / closed** | Enhanced preserves Grok TUI/agent-loop behavior. |
| `KIMI-C497-RELEASE-DOCS` | Kimi package releases, generated manifests, and docs/changelog maintenance. | **not applicable / closed** | No Enhanced runtime/provider contract or legal notice changes. |

### Stable temporary deferral `KIMI-VIDEO-001`

- **Pinned source/paths:** Kimi `c497af60e6cd20aab05e590f98a28fb15dd3491d`; commit `4c763f6763acb67a73d133f7450d092e71d63692`; prompt media, file upload, protocol schema, and tests changed by that commit.
- **Owner:** Enhanced Kimi provider-adapter maintainers.
- **Blocker:** shared Grok sampling/content/session schemas do not currently represent prompt-attached video. Adding Kimi-only untyped content would break provider/session/ACP boundaries.
- **User impact:** Kimi sessions cannot yet send a local video attachment through Kimi’s `/files` API as `purpose=video` and replay it as `ms://<id>`. Existing image and hosted web capabilities are unaffected.
- **Target:** next provider-schema milestone or the next upstream refresh, no later than `2026-08-01`.
- **Acceptance criteria:** introduce provider-neutral typed video content; upload only under an exact Kimi credential binding to the canonical Kimi endpoint; use `purpose=video`; project only the returned opaque file ID as `ms://<id>`; persist/resume safely; reject all cross-provider replay.
- **Intended tests:** schema round trip, canonical endpoint/header seal, no redirect, bounded upload, Kimi request projection, session resume, provider-switch stripping, and xAI/Codex/Custom negative credential tests.

## Open proof obligations for advanced provider references

| Stable ID | Source pin | Owner | User impact | Acceptance evidence |
| --- | --- | --- | --- | --- |
| `CDX-PROXY-001` | `4c43465133428898aa84f0bfc02c306ed65fb66a` | Codex adapter | No confirmed defect; route equivalence lacks end-to-end proof. | Refresh, revoke, normal Responses, and hosted-tool requests traverse selected routes without generic fallback or diagnostic leakage. |
| `CDX-CATALOG-001` | `4c43465133428898aa84f0bfc02c306ed65fb66a` | Codex catalog | New entitlement visibility is not proven end-to-end. | Auth change exposes a new Codex model immediately, clears old Codex entitlements first, and leaves xAI state unchanged. |
| `CDX-HISTORY-001` | `4c43465133428898aa84f0bfc02c306ed65fb66a` | Codex session/runtime | Full-history independence from client item IDs is not proven across compaction. | Tool call/output survives persistence, resume, compaction, and next request without `previous_response_id`. |
| `CDX-RETRY-001` | `4c43465133428898aa84f0bfc02c306ed65fb66a` | Codex sampler | No confirmed defect; ordinary-turn retry/cache proof is incomplete. | 429 delay/redaction and 401 prompt-cache stability pass through the complete request actor. |
| `OC-AUTH-001` | `7534d23551f665e65080809975b4ca5c7d63807b` | Codex catalog | Same user impact as `CDX-CATALOG-001`. | Closed by the shared post-auth model-visibility test. |

No production credential-routing or provider-isolation defect was found. These are proof obligations, so Codex/OpenCode reviewed pins do not advance in this pass.

## Complete advanced-source commit disposition inventories

`CDX-4C43-NO-ADAPTER-DELTA` denotes a reviewed Codex app-server/TUI/cloud/MCP/runtime/docs/maintenance commit with no change to the enumerated subscription-provider contract. It is not a claim that Codex application architecture is imported.

### OpenAI Codex: `51200321eb7b862a29ffceaba8b19db1934a9b38`..`4c43465133428898aa84f0bfc02c306ed65fb66a` (168 commits)

| Commit | Subject | Evidence |
| --- | --- | --- |
| `65f8bf68533332628b7fc213eade2a91d18d36ee` | Bind MCP calls to captured catalog revisions (#34588) | `CDX-4C43-MCP-REVISION` |
| `7442f5f9323d116755dfe630e22c931a8aeaa5c7` | Add keyed shell environment policy filters (#34590) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `ee71c4a90f49ce52a8c8801681111e9b5a19d7aa` | Enforce exact values from managed config requirements (#34597) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `87f71e35b86cc4d2da4d81728004adac45a9dd3a` | Skip missing paths in filesystem sandbox entries (#34598) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `6c00dc087e4c01312017389483573500001e9fe9` | Sanitize skill names in injection metrics (#34601) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `2497972808e7a5fc2c4db50a140bbd1559fc1d75` | Allow explicitly permitted loopback proxy targets (#34603) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `25d2dfcad010386610867a4635e0874296b468f1` | Allow naming sessions with `/new` and `/clear` (#34605) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `c7e3838987f56dd180cd6c8e77c012ebd2bb6088` | Add compatibility policies for skill catalog rendering (#34611) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `dfd2d8133c516778f72fea24e3d47c9a5f34e49b` | Detach non-interactive subprocesses from stdin (#34612) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `999a715089a697e8d205e7d7d725ea532664dd95` | Route Windows sandbox proxy traffic by restricting SID (#34613) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `ad020f29aeda36ceac4f7c75fc6b5c7e00ab6f7b` | Initialize missing-path behavior in exec-server sandbox test (#34615) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `d838ea0f64e44e99b87438b2b58c9ccf0bca6289` | Add exec-server network policy callback types (#34620) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `5bfd74d36cb1c019f50c747ad52fd652dc6741a3` | Load paginated model context across rollout lineages (#34621) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `ded4eacdbd4df4e001a1f072ed3e2b27797f2345` | Increase the auto-review model override test timeout (#34622) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `9b33613db62526359686349bf717f93532807849` | Terminate Windows process trees with job objects (#34624) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `bbad09a83b1fd9d7e507d688fcd266a7889d7df6` | Fix Windows TUI navigation key handling (#34625) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `37eef7baccebaeb00e42a88b323054cdbfe418c5` | Scale skill metadata budgets with model context windows (#34626) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `a26f219f6788c951dcb3bf435fab4c6d0f4d2f40` | Harden Windows elevated sandbox startup (#34629) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `adb143a2919ae77eaba55c2150685e489168be38` | Add a policy-aware HTTP client builder (#34630) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `1b77da35cc5d8cd358148a5ae4f6be128850c85b` | Migrate agent identity to the shared HTTP client (#34631) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `0a39ff138e7ed816a756169a36ce09cecd047aa6` | Keep the TUI open when starting a turn fails (#34636) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `81de4f251cfdaf32ecb85e2160ebfc11a562d44b` | Attribute review findings to repository rules (#34637) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `bdd3118c71a29f26b9df3a47f91efea38a0d58bd` | Update Windows process-tree tests for inherited FDs (#34640) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `c5eb33aed12d4977dc38403ecf8b42d89939ea32` | Harden managed proxy setup for sandboxed executions (#34641) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `539c0e1100512bbdaaeafdfa3dafc1c8954eb288` | Migrate login HTTP construction to `HttpClient` (#34643) | `CDX-4C43-ROUTED-AUTH` |
| `690995b7c1e8c8099c684402b5957535bada4e80` | Verify Git plugin SHA checkouts (#34644) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `4a443994bd12f49f2f08b21a2f224d9d42b9e734` | Always assign response item IDs (#34645) | `CDX-4C43-ITEM-IDS` |
| `f899a79c030a1bb7814872fe1551cf3c2772a40e` | Propagate resolved proxy policy through auth routing (#34649) | `CDX-4C43-ROUTED-AUTH` |
| `a26bc337cf61cbca9cd9ac25b25c88c32186eb20` | Require auth managers to receive routing configuration (#34650) | `CDX-4C43-ROUTED-AUTH` |
| `9fce9e13fd649f8c4549079eb6ca5d697e2ce0e4` | Migrate core test support to the shared HTTP client (#34651) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `1823c13771a07cf4ae98a971a06bb1b4ab7eda2d` | Render turn diffs for foreign environment paths (#34654) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `d4fcb2873bf23464cfacd804a31d46529db943b0` | Honor configured proxy routes for auth refreshes (#34655) | `CDX-4C43-ROUTED-AUTH` |
| `cefcffd692a3d070e9341ffa756a04e56d086c19` | Preserve approvals reviewer when forking threads (#34664) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `661339bb0941c055602688a83bcc8f72be21b54d` | Document `PathUri` drive letter canonicalization (#34667) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `4f3852107e5eedeb4cb89b57a6d4a35b49f8a59a` | Expand codex-http-client usage guidance (#34669) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `21db216db05d13713f09189fc44872d22cf47fc4` | Route LM Studio requests through the shared HTTP client (#34678) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `963cda85aa2a4cfb85e52d771d22d9f3069951fa` | Add session headers to realtime conversation starts (#34681) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `6e5a2d6b8d148a5554fdceb6f399ca45bd1c78d9` | Configure Codex Auto Review model metadata (#34687) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `516f1e2aff62293f4fe09f14868082f3852d0930` | Rename the MCP connection manager to `McpConnectionSet` (#34708) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `9fc715c0861c956c894a91890b78dc05b304ba29` | Order unified exec lifecycle events reliably (#34713) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `33d394c69e23e8906e1d0b9a3807f10ea4b7c294` | Skip Git enrichment for prewarm and Guardian turns (#34728) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `667b6bbaf12187619f51f9fec6329755dce72f64` | Publish stable installer aliases to R2 (#34729) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `6278742c418a2b3d3d29fd025dfdc13489a27987` | Preserve skill catalog entries under metadata pressure (#34732) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `84d2b203ed58f8fad25a601ce2f9f6a753ae940a` | Make MCP resource clients follow the latest runtime (#34733) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `fd51e505401b7b2da958edc269e4d7280be86bd5` | Remove step-scoped data from extension contributors (#34734) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `bd9a28a839d3dc4cf1facdf66cd02bb5732189e3` | Drop skill descriptions before omitting catalog entries (#34738) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `f21f98936ca365185bb6f6b97bc029e40aef314f` | Update skills budget tests for extension API changes (#34744) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `2ffe8cd579c27fb3ece5c4725d7fff5477a15cc9` | Match core skill ordering in extension catalogs (#34746) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `65ae4c26e088913176a50d6daeb742d00942caee` | Register the MCP 2026-07-28 feature flag (#34747) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `f69dc49b1579999b9788f9d3779fd180c6df67c7` | Reduce app-server JSON serialization overhead (#34761) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `64dc1c7a01b2aaa03701f08cf88b659c1a9737b3` | Retry websocket requests when the previous response is missing (#34763) | `CDX-4C43-WS-RECOVERY` |
| `e370d23691fbdaa12a2bf017d7e6bc101cca9b5f` | Reduce typed app-server request serialization overhead (#34766) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `ab816f3ca0fea858a4fc012e1bed826050d82961` | Add the git attribution extension (#34769) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `32f4687b8c43fb4062405106e761f85983aa96cc` | Enable exec-server network policy callbacks (#34770) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `9ee63da142121a57e5fe478ecb3aee822d5d003e` | Size unified mention popups to visible results (#34771) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `c00e2e851cd80191319d3a9146cfcff36b7c3b29` | Normalize whitespace-only lines in agent messages (#34772) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `730ec920032ba6bd16c97106616fb53cda8ae96b` | Clamp session headers to narrow terminal widths (#34775) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `80f3c3141e4dd421c861408a87703bad0cb09874` | Include the final agent message in turn completion summaries (#34777) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `ff8d521ba1097c07cec8b1aaa6e0242db9628a7b` | Coalesce wrapped OSC 8 hyperlinks in the TUI terminal (#34778) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `c5779ed6bb2acd868eeaa372e8996e493d9ed556` | Use the live parent history mode when forking agents (#34779) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `cc559bb971e74abfd581d9ebe3911e4564b66b9d` | Upgrade Bazel Rust and LLVM dependencies (#34781) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `44436fd0759249f47da8bfc41e6cae12af5b56ee` | Reject dynamic environments named `local` (#34784) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `fe6aa9d16c88b46df82d2c18fdc887e758871053` | Report skill catalog truncation during rendering (#34785) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `10cc57c95c2c8f1d01c8deaa75efb29b099d9c28` | Simplify app-server integration test setup (#34786) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `66bd101fff6f0e7e05a594ec7bdb78b92f6b66d3` | Avoid unnecessary post-sampling token estimates (#34789) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `2c49493b5b6bfcd3efbd83ae6031492296908c63` | Remove obsolete step store from git attribution tests (#34795) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `5381edb133075143839efb73de4f8695cf6332d0` | Skip syntax highlighting for lines over 4 KiB (#34796) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `f343d1237d8d360e8224997a846acde0b04a17cd` | Suppress omission notices in core-compatible skill catalogs (#34797) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `a59a419afa3492c58eb3d4865bf8ab1fd8c68330` | Use path URIs in shell approval keys (#34806) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `946ed315a484e052051d02b3e0274642da55dcf8` | Centralize SQLite connection configuration (#34808) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `06782eded78ff6e76d5139c2704988da99d2df42` | Fix network access rendering in sandbox prompts (#34811) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `08ae0fc0cef06a1b57d134019e924692e5be9ffe` | Consolidate thread startup around `StartThreadOptions` (#34814) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `4ebd976312b9088b3c845724e08d79c9f77505f4` | Support configurable realtime BEM channel prefixes (#34816) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `88eb3a2b8a925aaec939fadb9bb04a324926ae9d` | Enable git attribution across Codex entry points (#34819) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `7fd7a2f9a26d2d745b102ecf6754f599043c260e` | Run code-mode tests in non-Windows Bazel CI (#34823) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `bbfc3f0152cf332d01547ddfac835409bc8ce485` | Normalize Guardian review cwd reuse keys (#34824) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `12c115d558b9fec378b4445636a327b05e7de8cc` | Reduce cloning when building Responses requests (#34825) | `CDX-4C43-SERIALIZATION` |
| `b5748e6e3cbc3c9831f84aa016486721b4923d1c` | Remove Windows Bazel lint toolchain overrides (#34827) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `88f1cd9664d09b68909a258a061a662c1f099ce6` | Flush analytics before in-process app server shutdown (#34831) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `bd5b55e403e867f957e371840377cd284023f98c` | Track compaction time in turn profiles (#34835) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `d7e8f4c3dccc1647c88330efa73e58030d0b164d` | Preserve user input when MCP startup is interrupted (#34839) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `400ee190c30d5e4a88549c070a2335311f0baa91` | Add persisted thread pinning to the app server (#34840) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `79500d3cc1c7e64e079f501bfd92231bb3d052e9` | Remove first-party type from app metadata (#34844) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `0da13c6c993cbb6de3ce88591b316a40cbd411b1` | Track multi-agent mode in world state (#34845) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `0f9fb40fa9c4cc4b1ed0d595ce3ba70468a0c87a` | Allow custom providers to opt into standalone web search (#34846) | `CDX-4C43-CUSTOM-SEARCH` |
| `9d823343026e600dab694e41865ed60613da31b6` | Use Guardian model limits for review sessions (#34847) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `83ff1c2f809d810482be0bfda197f4f3f7abd697` | Cache remote plugin catalogs by scope (#34849) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `0a0a9b6c8f2153205ee31eea8e5bbaeb9a3ef5f7` | Disable image generation for Free-plan accounts (#34850) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `b72079a2cfe4eea5ca7931776cd6030b36e8728b` | Use batch metadata for plugin app summaries (#34851) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `44d76c6a6dd04fa2efc302b906ac8774267a1272` | Wake sleeping threads for queued agent mail (#34852) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `4e0cee8030c4833baa6331b378101545accc9956` | Wait for local plugin cache refreshes in `plugin/list` (#34877) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `6e0455fdc4114ae5d14a88ec966c090208e71e0c` | Set a default user agent for MCP HTTP requests (#34883) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `4462b9deef211723b781b426f5e5d36a5777115f` | Allow disabling the multi-agent wait tool (#34887) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `39a2438d16514d0d6f88105d17b0f747994af487` | Prefer releases.openai.com in standalone installers (#34910) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `e497325a6a1743cfadeee41a6b5f05ebf7fd0221` | Centralize thread MCP state in `McpRuntime` (#34930) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `48ebbf5334cbb65dd77ef372b4a7da3b0041964c` | Use the API plugin marketplace for Amazon Bedrock (#34931) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `808d3c2702ce8eae007c457aa930e7c3b68dd5f6` | Keep session defaults static during config batch writes (#34940) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `e19e65317a333ce725b18ac6f1e3bc904b74d2a1` | Reuse MCP connections across runtime refreshes (#34952) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `34b935e3e57f5071917fae20471024fee4190c82` | Replace closed MCP connections during reconciliation (#34957) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `7d748d3bbcbd640988813de962455f27c918abdf` | Handle @ in local marketplace paths (#34959) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `9e1f43dc2da77b0b25bb30bf60d6b5db34afc954` | Move MCP connection helpers into the test module (#34962) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `205d37a20f742b0bf8e191622bd07c43f567ea49` | Keep the sleep tool outside code mode (#34969) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `ad65f016ed0c91992fb175fa881a373cc460dd2a` | Honor disabled redirects in route-aware HTTP clients (#34978) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `8d34c0667215f9ae4f8a11678e27752d8a4a120f` | Infer the bundled Claude Code plugin marketplace (#34979) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `ce803c45aed425b08b94d8e3c5fb7db0d2193568` | Record externally completed agent config imports (#34981) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `5c94796dc9e88580fdf0b05ef9ce9d975a86e1a6` | Enforce single-writer ownership for paginated threads (#34986) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `0d4910331db5e5c6ec8e42f94d82119570801f95` | Preserve timestamps when importing external agent sessions (#34989) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `74e9d7efc416b1cb9f3ad10c70a91afbcb6d6a29` | Allow omitting MCP tool prefixes per server (#34991) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `c769a053406985c1fc47281fb81171b7721a3687` | Honor the configured SQLite home across state consumers (#34994) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `265cd2e100ff091bee0e691a7e28b42e4eb56837` | Initialize execution environments with the final HTTP policy (#34995) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `7bafdada8beaad9325ed69218f743f058e3598ab` | Separate Codex error details from retry metadata (#34996) | `CDX-4C43-RETRY` |
| `2c92af09cfc95d4190ffe8f86298ba65eebcf7d4` | Warn when skill catalogs exceed their context budget (#34997) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `62ba648136c7e60b9380c40b60cb553a7d8eb1ab` | Make TUI turn interrupts nonblocking (#35000) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `7d4b417cd1542412c13cdcf4492a12952d7e29bb` | Keep side conversations open when switching threads (#35011) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `5b402270f926cd1f9288c24cddd27410400cabe6` | Expose remote skill icon URLs through app server (#35012) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `b834702b27b3403403f5d1cd8992960a1a6211a5` | Support incremental replay of updated thread items (#35013) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `ceb2ffb793b7b990b435e70b4d71ee86eba823c4` | Align installed app duration metrics with the legacy baseline (#35015) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `5bdbd3ee90d746c3b8a040a53c434262ed07ee74` | Add trusted plugin script attribution (#35016) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `84fa68b429f12af1b4cdbb74c40a6fb1e74a30d3` | Attribute command executions to trusted plugin scripts (#35020) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `fe0d472c4c3a5e1c5163808446e904cb4c20fc54` | Adapt keyboard event reporting to the terminal (#35021) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `1ee8f49175a91c8fbfb9e93310331dfc45fc0dbd` | Route exec-server HTTP through configured proxy policy (#35023) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `091e4a5d7c7a79d07b1254591ffcaac1d231d16b` | Preserve refreshed Apps tools across MCP runtime updates (#35028) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `9fc4e5a7aaf0d4da64e8bef74d2aac02c2d12c79` | Preserve plugin attribution across command approvals (#35029) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `963316583b744e707d653a854b30e90d8dda8a78` | Enforce writer ownership for thread archive and deletion (#35031) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `41775559ca1ed253c2aa68db772a715cb348a9eb` | Expose Browser Use requirements through the app server (#35033) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `d45055ae58ef495610688086c3b490d1721a11b2` | Route environment registry requests through the shared HTTP client (#35034) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `b115de97d710ebb6984a0d91080c6d19d9b7017a` | Preserve Windows sandbox proxy settings in guardian sessions (#35036) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `4123bf61891636772eeed9a2387a6413645270d1` | Track app/read request duration (#35048) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `bb24b67d330946d8f30023cdcc5e9b3d2cdc73f5` | Register the Guardian V2 feature flag (#35049) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `fb4e6ba2f49278c49b33ced725964ace9d0f5c37` | Allow disabling the update_plan tool (#35054) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `94ebae725e5e8f22b5d86773d9223047f57b6118` | Route exec-server WebSockets through configured proxies (#35056) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `09241ae4db0f1eb25dfaedecf28d4b07dbd820a9` | Decouple exec-server HTTP from reqwest types (#35059) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `1d4b58f32d0b238e43adedc0f751812b974dc7bf` | Track deferred tool namespaces in world state (#35063) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `3947f0d0c3e255bade02e241c16cb43d284c0e65` | Avoid duplicating deferred sources in tool search (#35065) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `f47f28cd0d3da65d22711d43d1eb498893f5a735` | Fix Bazel test configuration for platform-specific data (#35067) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `0dfa778dae6a94b2ff2c69176cbaf063a3bf18a1` | Add WebSocket transport to the code-mode host (#35078) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `f61b51ddd924643514b33234816a8a2772b1aec7` | Support remote code-mode hosts in app-server (#35098) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `a28374e0dbb4119659fb68f8c73de48e01838a5e` | Support Agent Plugins manifests (#35105) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `81da9deb065d7adb283816b19b40f89bcc484276` | Allow hosts to customize `wait_for_environment` descriptions (#35106) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `ef2d3edb959a75a90665ade37d554f3ca65fe880` | Prewarm MCP runtime updates in the background (#35144) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `6c729ef1c1dcfbcbe1bd9d0c2dddde24377ae899` | Refresh MCP runtimes when session auth changes (#35146) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `f201c30c52a35f819262865a53df94b6f4ea7a50` | Reconnect MCP servers on explicit refresh (#35151) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `c8957bbf0f79fa29c5e08b8c0b942c12ea3893f2` | Encapsulate MCP refresh coordination (#35164) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `5dd992acd3f5242196ec690bb462e0d2687b485d` | Route extension warnings to app-server threads (#35168) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `5a1c54fc2110b1299c80e949425044dff478da25` | Compact host skill paths under metadata pressure (#35172) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `1a817bb95d942d4ca93f6ed09c97968713ff6d2a` | Wait for reloaded worker completion in the resume test (#35175) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `7c71783135b020e8f4db3fa26dc4319901c260b5` | Expose executor skills through skill tools (#35184) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `634a998d8aaeaf5f535e04d8475b17a62e7043a7` | Preserve output from hooks that exit before reading stdin (#35194) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `5f6a2c3adb159f606b13f4dd6b057d8a3431a1d1` | Make the Apps recovery exposure test deterministic (#35196) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `fe8500c0a00eaddda49b68e6ade818a93b58dfb5` | Enable resource reads for explicit executor skills (#35198) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `3645a4397c4889ea483a3b9a61ad7cf5921aa384` | Refresh MCP runtimes across thread startup (#35204) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `000d2540ad73996f3589ae178bfe447bfd67cef2` | Use current MCP authority for elicitation reviews (#35205) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `a177013eb055f16fcc9cd07308877ca36c88ede1` | Refresh managed MCP requirements for active threads (#35213) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `58b427722857117ac3e702b9eb406d47616022e2` | Refresh MCP config independently across threads (#35216) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `05f000263b2b2528cc9ca2a100270da9c6bf2fed` | Support paginated thread forks (#35220) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `99744cfe04806ebaa1e5d08e3e790070f852472b` | Avoid persisting non-local threads for hook transcripts (#35221) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `544007d006d869b4af03a205f6a94a9f2b022051` | Support the ent26 enterprise plan (#35238) | `CDX-4C43-ENT26` |
| `89a3b89c4c1d1afaaa93b6669c9e4e03247f8a99` | Route MCP auth discovery through runtime HTTP clients (#35239) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `1811b67a84952b8ebbd0605a0388e02a18453eb8` | Support ephemeral forks of paginated threads (#35251) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `32329b289d05eb6a3f8e35c267ceb25ba46716a2` | Expose workspace plugin publish capability (#35254) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `07fd04abb199fa4c3a1530873ec69e938061a615` | Propagate remote plugin IDs to skill metadata (#35261) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `0d2a0aa76ba8cabb468501a04d050834ec2ef80d` | Track remote plugin IDs in skill invocation analytics (#35262) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `a4535884169be8da2f81b8a4debecbd4dc11aa97` | Sign bundled macOS helper binaries (#35264) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `cba0e2701c9e3e67a877a16dbbd7a577d477a630` | Allow disabling the in-process code-mode host fallback (#35266) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `63fe5a6b71d45dfff24a6a1e5da0699e054f145d` | Harden network approval cancellation and concurrency (#35267) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `25b6fc9bbc49bbec12e8d38ceee550fc07cbc60d` | Include code-mode tool names in Responses Lite metadata (#35271) | `CDX-4C43-CODE-MODE` |
| `c3e926e61cb234fc180d8896bdb9655b58c3e735` | Trace remote exec-server connection setup (#35275) | `CDX-4C43-NO-ADAPTER-DELTA` |
| `4c43465133428898aa84f0bfc02c306ed65fb66a` | Skip plugin MCP filtering when no allowlists are configured (#35280) | `CDX-4C43-NO-ADAPTER-DELTA` |

### OpenCode: `0317531906d3f3bb01cf33c16319870cfde9170c`..`7534d23551f665e65080809975b4ca5c7d63807b` (64 commits)

| Commit | Subject | Evidence |
| --- | --- | --- |
| `c40e3e74a7d0009b3f57c12703fb81102d08b98b` | zen: add security check | `OC-7534-APP` |
| `76ced5418f50ee5cfb4c256af358bc0ab063a51b` | chore: generate | `OC-7534-APP` |
| `4438f69aac46806c631866489a26b644488a784e` | zen: add security check | `OC-7534-APP` |
| `130038eb63a94d832b0aea0d3ea9c83511db3d4d` | fix(app): defer unavailable notification state (#38186) | `OC-7534-APP` |
| `c9db6e9a1fe181fad2259689ef4ad9a5e89fbd5b` | fix(app): show running shell command (#38080) | `OC-7534-APP` |
| `0a601cf334b9a83cc2854108a2b860f25e6e7e8e` | fix(docs): correct Kimi K2.7 Code request limits (#38248) | `OC-7534-APP` |
| `50eee1f5a4b8580ef01a152ab21937ac12dc6ccc` | fix(provider): correct MiniMax M3 thinking variants (#38330) | `OC-7534-APP` |
| `411eff73f026d4950c07947c4d983788cb615baa` | feat(go): add Hy3 to Go model lineup (#38349) | `OC-7534-APP` |
| `542ba88602767490772efa423350f57622b68601` | fix(provider): select prompt cache keys by SDK (#38424) | `OC-7534-CACHE-KEY` |
| `fada1a538f4eb11d617229f15f23aaa8cfbd2d2a` | fix(provider): serialize Mistral prompt cache keys (#38448) | `OC-7534-APP` |
| `92cede0541305a99579b0575b79297089d37e6da` | chore: update nix node_modules hashes | `OC-7534-APP` |
| `e45210c6d218e368b1ddbd14fad378f5c1322741` | chore(app): vendor v2 promise client (#38467) | `OC-7534-APP` |
| `84c79c13991ec9df5a80954d324964e7816536d7` | chore: update nix node_modules hashes | `OC-7534-APP` |
| `d03e0c5e547f2bc7ae44e60eb21bfb24dad623fd` | feat(app): add dual-server compatibility (#38462) | `OC-7534-APP` |
| `347510a73b3ed5fa98504dd7122c15ea16c2d340` | chore: generate | `OC-7534-APP` |
| `e59ba24b801b41d7bb0cabe868c496c61e8ad8c6` | feat(app): support current event transport (#38464) | `OC-7534-APP` |
| `62e4641235d7847dadc60da37cca8a023dd54fc1` | chore: generate | `OC-7534-APP` |
| `20589d66d514993652af66932cb3a253f6e2f9fe` | fix(provider): preserve Mistral reasoning history (#38453) | `OC-7534-APP` |
| `743f6410f2e5002723fc5e893039ac49fbfe0de8` | chore: update nix node_modules hashes | `OC-7534-APP` |
| `204f48de8beada708ec0fff9310d556733ae4395` | docs(zen): add Ling 3.0 Flash free model (#38503) | `OC-7534-APP` |
| `37c263e1536f728064dcf78a5284251427b85d10` | feat(app): project current server state (#38459) | `OC-7534-APP` |
| `adba484df45a799d274d056112086f1588c8d961` | chore: generate | `OC-7534-APP` |
| `db88c423355a935d2fd266add07715c772f40ab7` | fix(app): hydrate v1 session progress (#38606) | `OC-7534-APP` |
| `ce9a875181b8ac7507e7eb84245b28ed31d75477` | feat(app): render current session timeline (#38466) | `OC-7534-APP` |
| `090a26a301b00e2bfde513f4c155aeb36430e376` | chore: generate | `OC-7534-APP` |
| `29af2e39ff7e35e24ea6ece72dbdafabbaaaf15d` | feat(app): migrate session interactions (#38461) | `OC-7534-APP` |
| `386afb77e0e1d9d61e1d4cea906f0108776c7c15` | chore: generate | `OC-7534-APP` |
| `589ef16128b0d787e389ee9b2544b53089ef5b0a` | refactor(app): split home view controllers (#38607) | `OC-7534-APP` |
| `5ce89dc2ad1b7f303f850fda4f8c222d9d14903b` | chore: generate | `OC-7534-APP` |
| `d07323ef5900afb88b35db0fa40741890a3f1c10` | feat(app): migrate discovery workflows (#38465) | `OC-7534-APP` |
| `2ea4bb793ec9240251b39706fb5564039023fd79` | chore: generate | `OC-7534-APP` |
| `a48912cbb10f972cd9b9be8a5f3bace296df0f4f` | fix(app): restore directory-scoped session status for v1 servers (#38637) | `OC-7534-APP` |
| `55f4a2691ae9e72a84c821d789f0912353197cbe` | fix(app): preserve paginated timeline order (#38641) | `OC-7534-APP` |
| `3819848cf20a4d46a2a4e7d21fc970795e35cc9b` | feat(app): support current review data (#38460) | `OC-7534-APP` |
| `bce2992729a9e0f1fe6dc3afa40f62004ab7a672` | chore: generate | `OC-7534-APP` |
| `ce7f54d5e7f1f36cc41858560fd6eb29ec96e5ce` | fix(app): make prompt input agent toggle reactive (#38653) | `OC-7534-APP` |
| `57ddfeb756ac87574a2c6623464e7120f185f4fe` | fix(app): classify existing web profiles for layout transition (#38117) | `OC-7534-APP` |
| `c4545ab12fc6fef94be26aa72d7273cb9baed738` | chore: generate | `OC-7534-APP` |
| `91ed2567ef7c613228c4adedc52fbf6e935a5333` | refactor(app): resolve server protocol state (#38648) | `OC-7534-APP` |
| `80a4fe8f39a974327497cc3c774569ee2512b0fc` | fix(app): remove diff rendering from file-specific tabs (#38662) | `OC-7534-APP` |
| `3337495427a7cdfb6eec2b82073bd8730c38ed6e` | chore: generate | `OC-7534-APP` |
| `aaa42fe3bfa89a282c42a8eb3fb4a3665371d0a8` | fix(app): isolate v2 servers from legacy layout (#38649) | `OC-7534-APP` |
| `67a04787bf15762abc305081563cfb14a35cb426` | fix(app): gate config permission auto-accept (#38650) | `OC-7534-APP` |
| `ad78ef5a4c65932b8f592f0150a67185813ee5cd` | feat(app): support current pty transport (#38463) | `OC-7534-APP` |
| `ae4be983cbec7b8275efaab63572e279471694a7` | chore: generate | `OC-7534-APP` |
| `b62806683eead4a47cc89029ea6085b4cb7a06c1` | fix(app): preserve inline file mentions (#38663) | `OC-7534-APP` |
| `9ba82a1b8c67f251adbcf9ae0fe36b4e76a64236` | fix(app): gate legacy server features (#38651) | `OC-7534-APP` |
| `66495a2a22cd0a57efcc4f721e65532f0987b4e8` | chore: generate | `OC-7534-APP` |
| `909db63265971d67d2fe4ba7f9d7b74cc33e2fdc` | fix(app): restore optimistic timeline state (#38693) | `OC-7534-APP` |
| `e63996919b6267d00a5ea224ab03b0f58fbd15d8` | fix(opencode): preserve grep symlink paths (#38581) | `OC-7534-APP` |
| `f51665191af10f1e4e0512af3708e9c2c58ecb8d` | fix(opencode): preserve grep symlink paths (#38581) | `OC-7534-APP` |
| `e62b09e6fce296ac8dde95f18fa5f8cfc17f0592` | zen: opus 5 | `OC-7534-APP` |
| `553b42fcc73ce956f281a55f8e3f21f4967f11dc` | Merge branch 'dev' of github.com:anomalyco/opencode into dev | `OC-7534-APP` |
| `4b19ea2a71a33e65cd7f3ed19964b1bed1722483` | fix(llm): preserve response message phases (#38452) | `OC-7534-APP` |
| `53669cab2b19815f03c73518fbad0790da31b65b` | chore: generate | `OC-7534-APP` |
| `7840562d1b7ec46bc2beb02e2114ce14f7b9a384` | fix(llm): revert response message phases (#38761) | `OC-7534-APP` |
| `2b2aacc93975330f9fd045d4306f698b0c6a8f8f` | fix(provider): generalize Claude adaptive thinking (#38757) | `OC-7534-APP` |
| `a85d8d23aa297b3051e642c28e3fc79b457fc4bc` | sync release versions for v1.18.5 | `OC-7534-APP` |
| `065dc274ec46a9995c04d8908293d3502bcff67e` | fix(core): branch-keyed repository cache with gated reference readiness (#38759) | `OC-7534-APP` |
| `5e2a6257b22c0141a20c281f4c2a641311afe5a5` | chore: generate | `OC-7534-APP` |
| `9e8b2171a5ed52651d98f45cda022bdefa71b724` | fix(app): refresh V1 providers after auth (#38786) | `OC-7534-AUTH-REFETCH` |
| `2b2b69d668ed05836ea6d3fa7f42d416bdb61806` | fix(app): refresh V1 MCP state (#38816) | `OC-7534-APP` |
| `0a6637e17aa79789a86608121f4dee8fad442d4f` | chore(app): vendor v2 client snapshot (#38818) | `OC-7534-APP` |
| `7534d23551f665e65080809975b4ca5c7d63807b` | chore: update nix node_modules hashes | `OC-7534-APP` |

### Kimi Code: `b5efba7abcaf4041f81ec520097a61e6546e8c50`..`c497af60e6cd20aab05e590f98a28fb15dd3491d` (26 commits)

| Commit | Subject | Evidence |
| --- | --- | --- |
| `ec88d352e8f4dc5e8ffd1212f016138458f69893` | fix: five correctness follow-ups to the catalog metadata work (#2030) | `KIMI-C497-CATALOG` |
| `430cd382a838e7a9de50aff7bd42586a4ccd1bd5` | refactor(agent-core-v2): drop pass-through methods from AgentRPCService (#2042) | `KIMI-C497-ENGINE` |
| `ba921ca5315bf31394c218ee70e233875bd740b7` | fix: gate always-thinking inference to OpenAI wires, plus catalog review follow-ups (#2036) | `KIMI-C497-CATALOG` |
| `4c763f6763acb67a73d133f7450d092e71d63692` | feat: send prompt-attached videos directly with the prompt (#1999) | `KIMI-VIDEO-001` |
| `8250e590f3ed5990c233ef5a2c7666468f0bcb05` | docs(cron): drop references to the non-existent `kimi resume` command (#2050) | `KIMI-C497-RELEASE-DOCS` |
| `8bf5bacba9e524c38fb808c0122070037ead25a8` | ci: release packages (#1989) | `KIMI-C497-RELEASE-DOCS` |
| `b32170b0181e61cb5dea68ff90a73ec2db71dffe` | docs(changelog): sync 0.29.0 from apps/kimi-code/CHANGELOG.md (#2054) | `KIMI-C497-RELEASE-DOCS` |
| `c6291c3ad71358c0e18b82c76056561235e321e9` | feat(v2-print): align kimi -p run lifecycle with the default engine (#2017) | `KIMI-C497-ENGINE` |
| `64f053cf46c6d8a50d529d15bc3f2f4fc88cea8f` | feat: agent-core-v2 permission/workspace refactors and transcript durability (#2021) | `KIMI-C497-ENGINE` |
| `e0f2a417691701e9bc73eaf5feebd4b667f5efab` | feat(datasource): add wind, imf, gildata, sec_edgar, and sp_data sources (#2029) | `KIMI-C497-ENGINE` |
| `188c0fcbf7c884d4a86bd4eebd012b0ab7aeb5da` | refactor(agent-core-v2): decouple kosong from config persistence (#2068) | `KIMI-C497-ENGINE` |
| `5240b5c83c876fd6fcbe199b3c7b4f65ef75d215` | feat(agent-core-v2): add generated config and wire-protocol manifests (#2086) | `KIMI-C497-RELEASE-DOCS` |
| `ca38b7ed864ad5fa2b2e3c8b96d8a7b10a734445` | chore: remove superpowers toolbar tip (#2089) | `KIMI-C497-UI-WEB` |
| `527d485d9296fe20f473a4a578d9e6a499c20cd9` | feat: add global default MCP server timeout configs (#2065) | `KIMI-C497-MCP-TIMEOUT` |
| `5fdbdb4a22b86ae6f7ba7c775741689aaaf215f0` | feat: configure web search/fetch services via KIMI_WEB_* env vars (#2096) | `KIMI-C497-WEB-CONFIG` |
| `d751b6796c6e9c4b29356d00d0a84678f24f3cb5` | feat(kap-server): global session work status, transcript subscribe_v2, and plan endpoint (#2094) | `KIMI-C497-ENGINE` |
| `66f611aae99887ad2076aa3482a0df5e415d3511` | fix: echo thinking under the reasoning field the endpoint actually uses (#2104) | `KIMI-C497-REASONING` |
| `7b62ed5b2c2709719f360c01a2f513dee34ae179` | feat: support a configurable secondary model for subagents (#2064) | `KIMI-C497-SECONDARY` |
| `dad11ed44ee16296d2aff4db20a9eae8ebba1c76` | chore: prune non-user-facing changesets (#2134) | `KIMI-C497-RELEASE-DOCS` |
| `f4c3967a417a539372eadab6c809d27b8a14c005` | ci: release packages (#2061) | `KIMI-C497-RELEASE-DOCS` |
| `c2b2c4eb49ad7fdd3463d2e867e2dbaed2c53736` | docs(changelog): sync 0.29.1 from apps/kimi-code/CHANGELOG.md (#2136) | `KIMI-C497-RELEASE-DOCS` |
| `a2401cc1ed26e5758c081e657bcff6a75cb061bb` | feat(kap-server): add provider write endpoints and models.dev/registry import (#2110) | `KIMI-C497-PROVIDER-WRITES` |
| `3615b5da9f33f0d04139459966f2ac58fc872fe3` | refactor(agent-core-v2): drop definition-level capability resolution (#2142) | `KIMI-C497-ENGINE` |
| `0d00a07c02e334ca904077b2ea8c56cf58b44586` | fix(web): preserve selected text when copying over HTTP (#2120) | `KIMI-C497-UI-WEB` |
| `f06eb5c60e0a4e51162d1854dda1db41892b457c` | feat(agent-core): defer registered user tools (#2119) | `KIMI-C497-ENGINE` |
| `c497af60e6cd20aab05e590f98a28fb15dd3491d` | fix(tui): steer user messages into the running turn while a goal is active (#2153) | `KIMI-C497-UI-WEB` |

## Complete advanced-reference changed-path inventories

<details><summary>OpenAI Codex: 1072 name-status records</summary>

```text
M	.bazelrc
M	.github/scripts/build-codex-package-archive.sh
M	.github/scripts/publish_r2_release.py
M	.github/scripts/run-argument-comment-lint-bazel.sh
M	.github/workflows/rust-release.yml
M	BUILD.bazel
M	MODULE.bazel
M	MODULE.bazel.lock
M	README.md
M	codex-rs/Cargo.lock
M	codex-rs/Cargo.toml
M	codex-rs/agent-graph-store/Cargo.toml
M	codex-rs/agent-graph-store/src/local.rs
M	codex-rs/agent-identity/Cargo.toml
M	codex-rs/agent-identity/src/lib.rs
M	codex-rs/analytics/src/analytics_client_tests.rs
M	codex-rs/analytics/src/client.rs
M	codex-rs/analytics/src/client_tests.rs
M	codex-rs/analytics/src/events.rs
M	codex-rs/analytics/src/facts.rs
M	codex-rs/analytics/src/reducer.rs
M	codex-rs/app-server-client/src/lib.rs
M	codex-rs/app-server-client/src/remote.rs
M	codex-rs/app-server-protocol/schema/json/ClientRequest.json
M	codex-rs/app-server-protocol/schema/json/ServerNotification.json
M	codex-rs/app-server-protocol/schema/json/codex_app_server_protocol.schemas.json
M	codex-rs/app-server-protocol/schema/json/codex_app_server_protocol.v2.schemas.json
M	codex-rs/app-server-protocol/schema/json/v2/AccountRateLimitsUpdatedNotification.json
M	codex-rs/app-server-protocol/schema/json/v2/AccountUpdatedNotification.json
M	codex-rs/app-server-protocol/schema/json/v2/AppListUpdatedNotification.json
M	codex-rs/app-server-protocol/schema/json/v2/AppsListResponse.json
M	codex-rs/app-server-protocol/schema/json/v2/AppsReadResponse.json
M	codex-rs/app-server-protocol/schema/json/v2/ConfigBatchWriteParams.json
M	codex-rs/app-server-protocol/schema/json/v2/ConfigRequirementsReadResponse.json
M	codex-rs/app-server-protocol/schema/json/v2/ExternalAgentConfigImportHistoriesReadResponse.json
A	codex-rs/app-server-protocol/schema/json/v2/ExternalAgentConfigImportHistoryRecordParams.json
A	codex-rs/app-server-protocol/schema/json/v2/ExternalAgentConfigImportHistoryRecordResponse.json
M	codex-rs/app-server-protocol/schema/json/v2/ExternalAgentConfigImportParams.json
M	codex-rs/app-server-protocol/schema/json/v2/GetAccountRateLimitsResponse.json
M	codex-rs/app-server-protocol/schema/json/v2/GetAccountResponse.json
M	codex-rs/app-server-protocol/schema/json/v2/ItemCompletedNotification.json
M	codex-rs/app-server-protocol/schema/json/v2/ItemStartedNotification.json
M	codex-rs/app-server-protocol/schema/json/v2/PluginInstalledResponse.json
M	codex-rs/app-server-protocol/schema/json/v2/PluginListResponse.json
M	codex-rs/app-server-protocol/schema/json/v2/PluginReadResponse.json
M	codex-rs/app-server-protocol/schema/json/v2/PluginShareListResponse.json
M	codex-rs/app-server-protocol/schema/json/v2/PluginShareSaveResponse.json
M	codex-rs/app-server-protocol/schema/json/v2/ReviewStartResponse.json
M	codex-rs/app-server-protocol/schema/json/v2/SkillsListResponse.json
M	codex-rs/app-server-protocol/schema/json/v2/ThreadForkResponse.json
M	codex-rs/app-server-protocol/schema/json/v2/ThreadListParams.json
M	codex-rs/app-server-protocol/schema/json/v2/ThreadListResponse.json
M	codex-rs/app-server-protocol/schema/json/v2/ThreadMetadataUpdateParams.json
M	codex-rs/app-server-protocol/schema/json/v2/ThreadMetadataUpdateResponse.json
M	codex-rs/app-server-protocol/schema/json/v2/ThreadReadResponse.json
M	codex-rs/app-server-protocol/schema/json/v2/ThreadResumeResponse.json
M	codex-rs/app-server-protocol/schema/json/v2/ThreadRollbackResponse.json
M	codex-rs/app-server-protocol/schema/json/v2/ThreadStartResponse.json
M	codex-rs/app-server-protocol/schema/json/v2/ThreadStartedNotification.json
M	codex-rs/app-server-protocol/schema/json/v2/ThreadUnarchiveResponse.json
M	codex-rs/app-server-protocol/schema/json/v2/TurnCompletedNotification.json
M	codex-rs/app-server-protocol/schema/json/v2/TurnStartResponse.json
M	codex-rs/app-server-protocol/schema/json/v2/TurnStartedNotification.json
M	codex-rs/app-server-protocol/schema/typescript/ClientRequest.ts
A	codex-rs/app-server-protocol/schema/typescript/PathUri.ts
M	codex-rs/app-server-protocol/schema/typescript/PlanType.ts
M	codex-rs/app-server-protocol/schema/typescript/index.ts
M	codex-rs/app-server-protocol/schema/typescript/v2/AppMetadata.ts
M	codex-rs/app-server-protocol/schema/typescript/v2/AppToolSummary.ts
A	codex-rs/app-server-protocol/schema/typescript/v2/BrowserUseRequirements.ts
M	codex-rs/app-server-protocol/schema/typescript/v2/ConfigBatchWriteParams.ts
M	codex-rs/app-server-protocol/schema/typescript/v2/ConfigRequirements.ts
M	codex-rs/app-server-protocol/schema/typescript/v2/ExternalAgentConfigImportHistory.ts
A	codex-rs/app-server-protocol/schema/typescript/v2/ExternalAgentConfigImportHistoryRecordParams.ts
A	codex-rs/app-server-protocol/schema/typescript/v2/ExternalAgentConfigImportHistoryRecordResponse.ts
M	codex-rs/app-server-protocol/schema/typescript/v2/ExternalAgentConfigImportParams.ts
A	codex-rs/app-server-protocol/schema/typescript/v2/FeedbackRequirements.ts
M	codex-rs/app-server-protocol/schema/typescript/v2/PluginShareContext.ts
M	codex-rs/app-server-protocol/schema/typescript/v2/PluginShareSaveResponse.ts
M	codex-rs/app-server-protocol/schema/typescript/v2/SkillInterface.ts
M	codex-rs/app-server-protocol/schema/typescript/v2/Thread.ts
M	codex-rs/app-server-protocol/schema/typescript/v2/ThreadItem.ts
M	codex-rs/app-server-protocol/schema/typescript/v2/ThreadListParams.ts
M	codex-rs/app-server-protocol/schema/typescript/v2/ThreadMetadataUpdateParams.ts
M	codex-rs/app-server-protocol/schema/typescript/v2/index.ts
M	codex-rs/app-server-protocol/src/export.rs
M	codex-rs/app-server-protocol/src/protocol/common.rs
M	codex-rs/app-server-protocol/src/protocol/item_builders.rs
M	codex-rs/app-server-protocol/src/protocol/thread_history.rs
M	codex-rs/app-server-protocol/src/protocol/v2/apps.rs
M	codex-rs/app-server-protocol/src/protocol/v2/config.rs
M	codex-rs/app-server-protocol/src/protocol/v2/item.rs
M	codex-rs/app-server-protocol/src/protocol/v2/permissions.rs
M	codex-rs/app-server-protocol/src/protocol/v2/plugin.rs
M	codex-rs/app-server-protocol/src/protocol/v2/realtime.rs
M	codex-rs/app-server-protocol/src/protocol/v2/tests.rs
M	codex-rs/app-server-protocol/src/protocol/v2/thread.rs
M	codex-rs/app-server-protocol/src/protocol/v2/thread_data.rs
M	codex-rs/app-server-protocol/tests/schema_fixtures.rs
M	codex-rs/app-server-test-client/src/lib.rs
M	codex-rs/app-server-transport/src/transport/mod.rs
M	codex-rs/app-server-transport/src/transport/remote_control/clients.rs
M	codex-rs/app-server-transport/src/transport/remote_control/enroll.rs
M	codex-rs/app-server-transport/src/transport/remote_control/server_api.rs
M	codex-rs/app-server-transport/src/transport/remote_control/tests.rs
M	codex-rs/app-server-transport/src/transport/remote_control/tests/clients_tests.rs
M	codex-rs/app-server-transport/src/transport/remote_control/tests/pairing_tests.rs
M	codex-rs/app-server-transport/src/transport/remote_control/websocket.rs
M	codex-rs/app-server/Cargo.toml
M	codex-rs/app-server/README.md
M	codex-rs/app-server/src/app_info.rs
M	codex-rs/app-server/src/app_server_tracing.rs
M	codex-rs/app-server/src/bespoke_event_handling.rs
M	codex-rs/app-server/src/bin/exec_server.rs
A	codex-rs/app-server/src/code_mode_host.rs
A	codex-rs/app-server/src/code_mode_host_tests.rs
M	codex-rs/app-server/src/config_manager.rs
M	codex-rs/app-server/src/config_manager_service.rs
M	codex-rs/app-server/src/config_manager_service_tests.rs
M	codex-rs/app-server/src/effective_plugin_change.rs
M	codex-rs/app-server/src/extensions.rs
M	codex-rs/app-server/src/external_agent_migration/processor.rs
M	codex-rs/app-server/src/external_agent_migration/protocol.rs
M	codex-rs/app-server/src/external_agent_migration/session_importer.rs
M	codex-rs/app-server/src/in_process.rs
M	codex-rs/app-server/src/lib.rs
M	codex-rs/app-server/src/main.rs
M	codex-rs/app-server/src/main_tests.rs
M	codex-rs/app-server/src/mcp_refresh.rs
M	codex-rs/app-server/src/message_processor.rs
M	codex-rs/app-server/src/message_processor_tracing_tests.rs
M	codex-rs/app-server/src/request_processors.rs
M	codex-rs/app-server/src/request_processors/account_processor.rs
M	codex-rs/app-server/src/request_processors/apps_processor.rs
M	codex-rs/app-server/src/request_processors/apps_processor/installed.rs
A	codex-rs/app-server/src/request_processors/apps_processor/installed_tests.rs
A	codex-rs/app-server/src/request_processors/apps_processor/read.rs
M	codex-rs/app-server/src/request_processors/catalog_processor.rs
M	codex-rs/app-server/src/request_processors/config_processor.rs
M	codex-rs/app-server/src/request_processors/feedback_doctor_report.rs
M	codex-rs/app-server/src/request_processors/mcp_processor.rs
M	codex-rs/app-server/src/request_processors/plugins.rs
M	codex-rs/app-server/src/request_processors/request_errors.rs
M	codex-rs/app-server/src/request_processors/thread_delete.rs
M	codex-rs/app-server/src/request_processors/thread_lifecycle.rs
M	codex-rs/app-server/src/request_processors/thread_processor.rs
M	codex-rs/app-server/src/request_processors/thread_processor_tests.rs
M	codex-rs/app-server/src/request_processors/thread_resume_redaction.rs
M	codex-rs/app-server/src/request_processors/thread_summary.rs
M	codex-rs/app-server/src/request_processors/turn_processor.rs
M	codex-rs/app-server/src/skills_watcher.rs
M	codex-rs/app-server/src/thread_state.rs
M	codex-rs/app-server/tests/common/config.rs
A	codex-rs/app-server/tests/common/config_tests.rs
M	codex-rs/app-server/tests/common/lib.rs
M	codex-rs/app-server/tests/common/test_app_server.rs
M	codex-rs/app-server/tests/suite/auth.rs
M	codex-rs/app-server/tests/suite/conversation_summary.rs
M	codex-rs/app-server/tests/suite/fuzzy_file_search.rs
M	codex-rs/app-server/tests/suite/logging.rs
M	codex-rs/app-server/tests/suite/v2/account.rs
M	codex-rs/app-server/tests/suite/v2/app_installed.rs
M	codex-rs/app-server/tests/suite/v2/app_list.rs
M	codex-rs/app-server/tests/suite/v2/app_read.rs
M	codex-rs/app-server/tests/suite/v2/client_metadata.rs
A	codex-rs/app-server/tests/suite/v2/code_mode_host.rs
M	codex-rs/app-server/tests/suite/v2/command_exec.rs
M	codex-rs/app-server/tests/suite/v2/compaction.rs
M	codex-rs/app-server/tests/suite/v2/config_rpc.rs
M	codex-rs/app-server/tests/suite/v2/current_time.rs
M	codex-rs/app-server/tests/suite/v2/dynamic_tools.rs
M	codex-rs/app-server/tests/suite/v2/executor_mcp.rs
M	codex-rs/app-server/tests/suite/v2/executor_skills.rs
M	codex-rs/app-server/tests/suite/v2/experimental_api.rs
M	codex-rs/app-server/tests/suite/v2/experimental_feature_list.rs
M	codex-rs/app-server/tests/suite/v2/external_agent_config.rs
A	codex-rs/app-server/tests/suite/v2/git_attribution.rs
M	codex-rs/app-server/tests/suite/v2/hooks_list.rs
M	codex-rs/app-server/tests/suite/v2/imagegen_extension.rs
M	codex-rs/app-server/tests/suite/v2/initialize.rs
M	codex-rs/app-server/tests/suite/v2/marketplace_add.rs
M	codex-rs/app-server/tests/suite/v2/marketplace_remove.rs
M	codex-rs/app-server/tests/suite/v2/marketplace_upgrade.rs
M	codex-rs/app-server/tests/suite/v2/mcp_resource.rs
M	codex-rs/app-server/tests/suite/v2/mcp_server_status.rs
M	codex-rs/app-server/tests/suite/v2/mcp_tool.rs
M	codex-rs/app-server/tests/suite/v2/memory_reset.rs
M	codex-rs/app-server/tests/suite/v2/mod.rs
M	codex-rs/app-server/tests/suite/v2/model_list.rs
M	codex-rs/app-server/tests/suite/v2/model_provider_capabilities_read.rs
M	codex-rs/app-server/tests/suite/v2/output_schema.rs
M	codex-rs/app-server/tests/suite/v2/permission_profile_list.rs
M	codex-rs/app-server/tests/suite/v2/plan_item.rs
M	codex-rs/app-server/tests/suite/v2/plugin_install.rs
M	codex-rs/app-server/tests/suite/v2/plugin_list.rs
M	codex-rs/app-server/tests/suite/v2/plugin_read.rs
M	codex-rs/app-server/tests/suite/v2/plugin_share.rs
M	codex-rs/app-server/tests/suite/v2/plugin_uninstall.rs
M	codex-rs/app-server/tests/suite/v2/rate_limit_reset_credits.rs
M	codex-rs/app-server/tests/suite/v2/rate_limits.rs
M	codex-rs/app-server/tests/suite/v2/realtime_conversation.rs
M	codex-rs/app-server/tests/suite/v2/remote_control.rs
M	codex-rs/app-server/tests/suite/v2/remote_thread_store.rs
M	codex-rs/app-server/tests/suite/v2/request_permissions.rs
M	codex-rs/app-server/tests/suite/v2/request_user_input.rs
M	codex-rs/app-server/tests/suite/v2/request_validation.rs
M	codex-rs/app-server/tests/suite/v2/review.rs
M	codex-rs/app-server/tests/suite/v2/safety_check_downgrade.rs
M	codex-rs/app-server/tests/suite/v2/selected_environment.rs
M	codex-rs/app-server/tests/suite/v2/session_end.rs
M	codex-rs/app-server/tests/suite/v2/skills_list.rs
M	codex-rs/app-server/tests/suite/v2/sleep.rs
M	codex-rs/app-server/tests/suite/v2/thread_archive.rs
M	codex-rs/app-server/tests/suite/v2/thread_delete.rs
M	codex-rs/app-server/tests/suite/v2/thread_fork.rs
M	codex-rs/app-server/tests/suite/v2/thread_inject_items.rs
M	codex-rs/app-server/tests/suite/v2/thread_list.rs
M	codex-rs/app-server/tests/suite/v2/thread_loaded_list.rs
M	codex-rs/app-server/tests/suite/v2/thread_memory_mode_set.rs
M	codex-rs/app-server/tests/suite/v2/thread_metadata_update.rs
M	codex-rs/app-server/tests/suite/v2/thread_read.rs
M	codex-rs/app-server/tests/suite/v2/thread_resume.rs
M	codex-rs/app-server/tests/suite/v2/thread_rollback.rs
M	codex-rs/app-server/tests/suite/v2/thread_settings_update.rs
M	codex-rs/app-server/tests/suite/v2/thread_shell_command.rs
M	codex-rs/app-server/tests/suite/v2/thread_start.rs
M	codex-rs/app-server/tests/suite/v2/thread_status.rs
M	codex-rs/app-server/tests/suite/v2/thread_unarchive.rs
M	codex-rs/app-server/tests/suite/v2/thread_unsubscribe.rs
M	codex-rs/app-server/tests/suite/v2/turn_interrupt.rs
M	codex-rs/app-server/tests/suite/v2/turn_start.rs
M	codex-rs/app-server/tests/suite/v2/turn_start_zsh_fork.rs
M	codex-rs/app-server/tests/suite/v2/turn_steer.rs
M	codex-rs/app-server/tests/suite/v2/web_search.rs
M	codex-rs/apply-patch/src/lib.rs
M	codex-rs/backend-client/src/client.rs
M	codex-rs/chatgpt/src/connectors.rs
M	codex-rs/cli/BUILD.bazel
M	codex-rs/cli/Cargo.toml
M	codex-rs/cli/src/debug_sandbox.rs
M	codex-rs/cli/src/doctor.rs
M	codex-rs/cli/src/doctor/git.rs
M	codex-rs/cli/src/doctor/thread_inventory.rs
M	codex-rs/cli/src/login.rs
M	codex-rs/cli/src/main.rs
M	codex-rs/cli/src/mcp_cmd.rs
M	codex-rs/cli/src/plugin_cmd.rs
M	codex-rs/cli/src/remote_control_cmd.rs
M	codex-rs/cli/src/sandbox_setup.rs
M	codex-rs/cli/src/state_db_recovery.rs
M	codex-rs/cli/tests/debug_clear_memories.rs
M	codex-rs/cli/tests/login.rs
M	codex-rs/cli/tests/mcp_list.rs
M	codex-rs/cli/tests/sandbox_network_proxy.rs
M	codex-rs/cloud-config/src/bundle_loader.rs
M	codex-rs/cloud-config/src/service_tests.rs
M	codex-rs/cloud-tasks/src/lib.rs
M	codex-rs/cloud-tasks/src/util.rs
M	codex-rs/code-mode-host/Cargo.toml
M	codex-rs/code-mode-host/src/host_tests.rs
M	codex-rs/code-mode-host/src/lib.rs
M	codex-rs/code-mode-host/src/main.rs
M	codex-rs/code-mode-host/src/peer.rs
A	codex-rs/code-mode-host/src/transport.rs
A	codex-rs/code-mode-host/src/transport_tests.rs
A	codex-rs/code-mode-host/tests/websocket.rs
M	codex-rs/code-mode-protocol/src/host/codec.rs
M	codex-rs/code-mode-protocol/src/host/codec_tests.rs
M	codex-rs/code-mode-protocol/src/host/mod.rs
M	codex-rs/code-mode/Cargo.toml
M	codex-rs/code-mode/src/lib.rs
M	codex-rs/code-mode/src/remote_session.rs
M	codex-rs/code-mode/src/remote_session/connection.rs
M	codex-rs/code-mode/src/remote_session/connection/driver/delegate_runtime.rs
M	codex-rs/code-mode/src/remote_session/connection/driver_tests.rs
M	codex-rs/code-mode/src/remote_session/connection/reader.rs
A	codex-rs/code-mode/src/remote_session/connection/transport.rs
M	codex-rs/code-mode/src/remote_session_tests.rs
M	codex-rs/codex-api/Cargo.toml
M	codex-rs/codex-api/src/api_bridge.rs
M	codex-rs/codex-api/src/api_bridge_tests.rs
M	codex-rs/codex-api/src/common.rs
M	codex-rs/codex-api/src/endpoint/responses_websocket.rs
M	codex-rs/codex-api/src/lib.rs
M	codex-rs/codex-api/tests/clients.rs
M	codex-rs/codex-backend-openapi-models/src/models/rate_limit_status_payload.rs
M	codex-rs/codex-mcp/Cargo.toml
A	codex-rs/codex-mcp/src/binding.rs
M	codex-rs/codex-mcp/src/binding_clients.rs
A	codex-rs/codex-mcp/src/binding_tests.rs
M	codex-rs/codex-mcp/src/connection_manager.rs
M	codex-rs/codex-mcp/src/connection_manager/required.rs
A	codex-rs/codex-mcp/src/connection_manager/resources.rs
A	codex-rs/codex-mcp/src/connection_manager/startup.rs
M	codex-rs/codex-mcp/src/connection_manager/tool_catalog.rs
M	codex-rs/codex-mcp/src/connection_manager_tests.rs
M	codex-rs/codex-mcp/src/elicitation.rs
M	codex-rs/codex-mcp/src/lib.rs
M	codex-rs/codex-mcp/src/mcp/auth.rs
M	codex-rs/codex-mcp/src/mcp/mod.rs
M	codex-rs/codex-mcp/src/mcp/mod_tests.rs
M	codex-rs/codex-mcp/src/resource_client.rs
M	codex-rs/codex-mcp/src/rmcp_client.rs
M	codex-rs/codex-mcp/src/runtime.rs
M	codex-rs/codex-mcp/src/server.rs
M	codex-rs/codex-mcp/src/tools.rs
M	codex-rs/config/src/config_requirements.rs
M	codex-rs/config/src/config_toml.rs
M	codex-rs/config/src/diagnostics.rs
M	codex-rs/config/src/lib.rs
M	codex-rs/config/src/loader/mod.rs
M	codex-rs/config/src/merge.rs
M	codex-rs/config/src/merge_tests.rs
M	codex-rs/config/src/requirements_layers/layer.rs
M	codex-rs/config/src/requirements_layers/stack.rs
M	codex-rs/config/src/requirements_layers/stack_tests.rs
M	codex-rs/config/src/schema.rs
A	codex-rs/config/src/shell_environment_policy.rs
A	codex-rs/config/src/shell_environment_policy_tests.rs
M	codex-rs/config/src/state.rs
M	codex-rs/config/src/state_tests.rs
M	codex-rs/config/src/thread_config.rs
M	codex-rs/config/src/thread_config/proto/codex.thread_config.v1.proto
M	codex-rs/config/src/thread_config/proto/codex.thread_config.v1.rs
M	codex-rs/config/src/thread_config/remote.rs
M	codex-rs/config/src/tui_keymap.rs
M	codex-rs/config/src/types.rs
M	codex-rs/connectors/src/app_info.rs
M	codex-rs/connectors/src/app_tool_policy_tests.rs
M	codex-rs/connectors/src/lib.rs
M	codex-rs/connectors/src/metadata_store.rs
M	codex-rs/connectors/src/metadata_store_tests.rs
M	codex-rs/core-api/Cargo.toml
M	codex-rs/core-api/src/lib.rs
M	codex-rs/core-plugins/Cargo.toml
A	codex-rs/core-plugins/src/agent_plugin_manifest.rs
A	codex-rs/core-plugins/src/agent_plugin_manifest_tests.rs
M	codex-rs/core-plugins/src/lib.rs
M	codex-rs/core-plugins/src/loader.rs
M	codex-rs/core-plugins/src/loader_tests.rs
M	codex-rs/core-plugins/src/manager.rs
M	codex-rs/core-plugins/src/manager_tests.rs
M	codex-rs/core-plugins/src/manifest.rs
M	codex-rs/core-plugins/src/marketplace_add/source.rs
M	codex-rs/core-plugins/src/provider_tests.rs
M	codex-rs/core-plugins/src/remote.rs
M	codex-rs/core-plugins/src/remote/catalog_cache.rs
A	codex-rs/core-plugins/src/remote/catalog_cache_tests.rs
M	codex-rs/core-plugins/src/remote/share.rs
M	codex-rs/core-plugins/src/remote/share/tests.rs
A	codex-rs/core-plugins/src/remote_plugin_id_resolver.rs
M	codex-rs/core-plugins/src/remote_tests.rs
A	codex-rs/core-plugins/src/script_attribution.rs
A	codex-rs/core-plugins/src/script_attribution_tests.rs
M	codex-rs/core-plugins/src/store.rs
M	codex-rs/core-plugins/src/test_support.rs
M	codex-rs/core-plugins/src/tool_suggest_metadata.rs
M	codex-rs/core-skills/src/injection.rs
M	codex-rs/core-skills/src/injection_tests.rs
M	codex-rs/core-skills/src/invocation_utils_tests.rs
M	codex-rs/core-skills/src/loader.rs
M	codex-rs/core-skills/src/loader/discovery.rs
M	codex-rs/core-skills/src/loader/environment.rs
M	codex-rs/core-skills/src/loader_tests.rs
M	codex-rs/core-skills/src/model.rs
M	codex-rs/core-skills/src/remote.rs
M	codex-rs/core-skills/src/render.rs
M	codex-rs/core-skills/src/root_loader.rs
M	codex-rs/core-skills/src/service.rs
M	codex-rs/core-skills/src/service_tests.rs
M	codex-rs/core-skills/tests/environment_loader.rs
M	codex-rs/core/Cargo.toml
M	codex-rs/core/config.schema.json
M	codex-rs/core/src/agent/agent_resolver.rs
M	codex-rs/core/src/agent/control.rs
M	codex-rs/core/src/agent/control/execution.rs
M	codex-rs/core/src/agent/control/execution_tests.rs
M	codex-rs/core/src/agent/control/legacy.rs
M	codex-rs/core/src/agent/control/residency.rs
M	codex-rs/core/src/agent/control/residency_tests.rs
M	codex-rs/core/src/agent/control/spawn.rs
M	codex-rs/core/src/agent/control_tests.rs
M	codex-rs/core/src/agent/registry.rs
M	codex-rs/core/src/agent/registry_tests.rs
M	codex-rs/core/src/client.rs
M	codex-rs/core/src/client_common_tests.rs
M	codex-rs/core/src/client_tests.rs
M	codex-rs/core/src/codex_delegate.rs
M	codex-rs/core/src/codex_delegate_tests.rs
M	codex-rs/core/src/codex_thread.rs
M	codex-rs/core/src/compact.rs
M	codex-rs/core/src/compact_model_fallback.rs
M	codex-rs/core/src/compact_remote.rs
M	codex-rs/core/src/compact_remote_request.rs
M	codex-rs/core/src/compact_remote_v2.rs
M	codex-rs/core/src/compact_remote_v2_attempt.rs
M	codex-rs/core/src/compact_tests.rs
M	codex-rs/core/src/compact_token_budget.rs
M	codex-rs/core/src/config/config_loader_tests.rs
M	codex-rs/core/src/config/config_tests.rs
M	codex-rs/core/src/config/mod.rs
M	codex-rs/core/src/config/network_proxy_spec.rs
M	codex-rs/core/src/config/permissions.rs
M	codex-rs/core/src/config/permissions_tests.rs
A	codex-rs/core/src/config/requirements.rs
M	codex-rs/core/src/config/schema_tests.rs
M	codex-rs/core/src/connectors.rs
M	codex-rs/core/src/context/mod.rs
M	codex-rs/core/src/context/multi_agent_mode_instructions.rs
M	codex-rs/core/src/context/world_state/environment_render_tests.rs
M	codex-rs/core/src/context/world_state/mod.rs
A	codex-rs/core/src/context/world_state/multi_agent_mode.rs
A	codex-rs/core/src/context/world_state/multi_agent_mode_tests.rs
A	codex-rs/core/src/context/world_state/snapshots/codex_core__context__world_state__multi_agent_mode__tests__snapshots.snap
A	codex-rs/core/src/context/world_state/tools.rs
A	codex-rs/core/src/context/world_state/tools_tests.rs
M	codex-rs/core/src/context_manager/updates.rs
M	codex-rs/core/src/environment_selection.rs
M	codex-rs/core/src/event_mapping.rs
M	codex-rs/core/src/exec.rs
M	codex-rs/core/src/exec_policy_tests.rs
M	codex-rs/core/src/exec_policy_windows_tests.rs
M	codex-rs/core/src/exec_tests.rs
M	codex-rs/core/src/git_info_tests.rs
M	codex-rs/core/src/guardian/mod.rs
M	codex-rs/core/src/guardian/review.rs
M	codex-rs/core/src/guardian/review_session.rs
M	codex-rs/core/src/guardian/tests.rs
M	codex-rs/core/src/lib.rs
M	codex-rs/core/src/mcp.rs
M	codex-rs/core/src/mcp_tool_call.rs
M	codex-rs/core/src/mcp_tool_call_tests.rs
M	codex-rs/core/src/prompt_debug.rs
M	codex-rs/core/src/realtime_context.rs
M	codex-rs/core/src/realtime_context_tests.rs
M	codex-rs/core/src/realtime_conversation.rs
M	codex-rs/core/src/realtime_conversation/bem.rs
M	codex-rs/core/src/realtime_conversation/bem_tests.rs
M	codex-rs/core/src/realtime_conversation_tests.rs
M	codex-rs/core/src/responses_metadata.rs
M	codex-rs/core/src/responses_retry.rs
M	codex-rs/core/src/responses_retry_tests.rs
M	codex-rs/core/src/rollout.rs
M	codex-rs/core/src/safety_tests.rs
M	codex-rs/core/src/sandbox_tags_tests.rs
M	codex-rs/core/src/session/config_lock.rs
M	codex-rs/core/src/session/elicitation_holders_tests.rs
M	codex-rs/core/src/session/handlers.rs
M	codex-rs/core/src/session/mcp.rs
A	codex-rs/core/src/session/mcp_prewarm.rs
A	codex-rs/core/src/session/mcp_refresh.rs
M	codex-rs/core/src/session/mcp_runtime.rs
M	codex-rs/core/src/session/mod.rs
M	codex-rs/core/src/session/session.rs
M	codex-rs/core/src/session/step_context.rs
M	codex-rs/core/src/session/tests.rs
M	codex-rs/core/src/session/tests/guardian_tests.rs
M	codex-rs/core/src/session/turn.rs
M	codex-rs/core/src/session/turn_context.rs
M	codex-rs/core/src/session/turn_tests.rs
M	codex-rs/core/src/session/world_state.rs
M	codex-rs/core/src/session_rollout_init_error.rs
M	codex-rs/core/src/session_startup_prewarm.rs
M	codex-rs/core/src/skills.rs
M	codex-rs/core/src/state/service.rs
M	codex-rs/core/src/state/turn.rs
M	codex-rs/core/src/stream_events_utils_tests.rs
M	codex-rs/core/src/tasks/compact.rs
M	codex-rs/core/src/tasks/mod.rs
M	codex-rs/core/src/tasks/regular.rs
M	codex-rs/core/src/tasks/user_shell.rs
M	codex-rs/core/src/test_support.rs
M	codex-rs/core/src/thread_manager.rs
M	codex-rs/core/src/thread_manager_tests.rs
M	codex-rs/core/src/thread_rollout_truncation_tests.rs
M	codex-rs/core/src/tools/events.rs
M	codex-rs/core/src/tools/handlers/extension_tools.rs
M	codex-rs/core/src/tools/handlers/mcp_resource/list_mcp_resource_templates.rs
M	codex-rs/core/src/tools/handlers/mcp_resource/list_mcp_resources.rs
M	codex-rs/core/src/tools/handlers/mcp_resource/read_mcp_resource.rs
M	codex-rs/core/src/tools/handlers/mod.rs
M	codex-rs/core/src/tools/handlers/multi_agents/close_agent.rs
M	codex-rs/core/src/tools/handlers/multi_agents/wait.rs
M	codex-rs/core/src/tools/handlers/multi_agents_common.rs
M	codex-rs/core/src/tools/handlers/multi_agents_tests.rs
M	codex-rs/core/src/tools/handlers/multi_agents_v2/interrupt_agent.rs
M	codex-rs/core/src/tools/handlers/request_plugin_install.rs
M	codex-rs/core/src/tools/handlers/shell.rs
M	codex-rs/core/src/tools/handlers/sleep.rs
M	codex-rs/core/src/tools/handlers/tool_search.rs
M	codex-rs/core/src/tools/handlers/tool_search_spec.rs
M	codex-rs/core/src/tools/handlers/unified_exec/write_stdin.rs
M	codex-rs/core/src/tools/handlers/wait_for_environment.rs
M	codex-rs/core/src/tools/network_approval.rs
M	codex-rs/core/src/tools/network_approval_tests.rs
M	codex-rs/core/src/tools/orchestrator.rs
M	codex-rs/core/src/tools/registry.rs
M	codex-rs/core/src/tools/router.rs
M	codex-rs/core/src/tools/router_tests.rs
M	codex-rs/core/src/tools/runtimes/apply_patch.rs
M	codex-rs/core/src/tools/runtimes/mod.rs
M	codex-rs/core/src/tools/runtimes/shell.rs
M	codex-rs/core/src/tools/runtimes/shell/unix_escalation.rs
M	codex-rs/core/src/tools/runtimes/shell/unix_escalation_tests.rs
M	codex-rs/core/src/tools/runtimes/shell_tests.rs
M	codex-rs/core/src/tools/runtimes/unified_exec.rs
M	codex-rs/core/src/tools/sandboxing.rs
M	codex-rs/core/src/tools/sandboxing_tests.rs
M	codex-rs/core/src/tools/spec_plan.rs
M	codex-rs/core/src/tools/spec_plan_tests.rs
M	codex-rs/core/src/turn_diff_tracker.rs
M	codex-rs/core/src/turn_diff_tracker_tests.rs
M	codex-rs/core/src/turn_metadata.rs
M	codex-rs/core/src/turn_metadata_tests.rs
M	codex-rs/core/src/turn_timing.rs
M	codex-rs/core/src/turn_timing_tests.rs
M	codex-rs/core/src/unified_exec/async_watcher.rs
M	codex-rs/core/src/unified_exec/async_watcher_tests.rs
M	codex-rs/core/src/unified_exec/mod.rs
M	codex-rs/core/src/unified_exec/mod_tests.rs
M	codex-rs/core/src/unified_exec/process.rs
M	codex-rs/core/src/unified_exec/process_manager.rs
M	codex-rs/core/src/unified_exec/process_manager_tests.rs
M	codex-rs/core/src/unified_exec/process_tests.rs
M	codex-rs/core/src/windows_sandbox.rs
M	codex-rs/core/src/windows_sandbox_tests.rs
M	codex-rs/core/tests/common/Cargo.toml
M	codex-rs/core/tests/common/apps_test_server.rs
M	codex-rs/core/tests/common/context_snapshot.rs
M	codex-rs/core/tests/common/hooks.rs
M	codex-rs/core/tests/common/responses.rs
M	codex-rs/core/tests/common/streaming_sse.rs
M	codex-rs/core/tests/common/test_codex.rs
M	codex-rs/core/tests/responses_headers.rs
M	codex-rs/core/tests/suite/agent_websocket.rs
M	codex-rs/core/tests/suite/agents_md.rs
M	codex-rs/core/tests/suite/apply_patch_cli.rs
M	codex-rs/core/tests/suite/approvals.rs
M	codex-rs/core/tests/suite/audio_truncation.rs
M	codex-rs/core/tests/suite/auto_review.rs
M	codex-rs/core/tests/suite/client.rs
M	codex-rs/core/tests/suite/client_websockets.rs
M	codex-rs/core/tests/suite/code_mode.rs
M	codex-rs/core/tests/suite/compact.rs
M	codex-rs/core/tests/suite/compact_remote.rs
M	codex-rs/core/tests/suite/compact_remote_parity.rs
M	codex-rs/core/tests/suite/extension_sandbox.rs
M	codex-rs/core/tests/suite/fork_thread.rs
A	codex-rs/core/tests/suite/git_enrichment.rs
M	codex-rs/core/tests/suite/guardian_review.rs
M	codex-rs/core/tests/suite/hooks.rs
M	codex-rs/core/tests/suite/image_rollout.rs
M	codex-rs/core/tests/suite/items.rs
M	codex-rs/core/tests/suite/mcp_auth_elicitation.rs
M	codex-rs/core/tests/suite/mcp_auth_refresh.rs
A	codex-rs/core/tests/suite/mcp_startup_refresh_http_proxy.rs
M	codex-rs/core/tests/suite/mcp_tool_cache.rs
M	codex-rs/core/tests/suite/mcp_tool_exposure.rs
M	codex-rs/core/tests/suite/mcp_turn_metadata.rs
M	codex-rs/core/tests/suite/mod.rs
M	codex-rs/core/tests/suite/model_switching.rs
M	codex-rs/core/tests/suite/multi_agent_mode.rs
M	codex-rs/core/tests/suite/multi_agent_resume.rs
M	codex-rs/core/tests/suite/network_approval.rs
M	codex-rs/core/tests/suite/pending_input.rs
M	codex-rs/core/tests/suite/plugins.rs
M	codex-rs/core/tests/suite/prompt_caching.rs
M	codex-rs/core/tests/suite/prompt_debug_tests.rs
M	codex-rs/core/tests/suite/realtime_conversation.rs
M	codex-rs/core/tests/suite/realtime_initial_items.rs
M	codex-rs/core/tests/suite/remote_env.rs
M	codex-rs/core/tests/suite/request_plugin_install.rs
M	codex-rs/core/tests/suite/responses_lite.rs
A	codex-rs/core/tests/suite/responses_system_proxy.rs
M	codex-rs/core/tests/suite/rmcp_client.rs
M	codex-rs/core/tests/suite/rollout_list_find.rs
M	codex-rs/core/tests/suite/search_tool.rs
A	codex-rs/core/tests/suite/skills_extension.rs
A	codex-rs/core/tests/suite/snapshots/all__suite__mcp_tool_exposure__deferred_tools_initial_unchanged_and_removed.snap
A	codex-rs/core/tests/suite/snapshots/all__suite__mcp_tool_exposure__deferred_tools_recover_during_sampling.snap
A	codex-rs/core/tests/suite/snapshots/all__suite__mcp_tool_exposure__deferred_tools_resume_without_duplicate_update.snap
M	codex-rs/core/tests/suite/spawn_agent_description.rs
M	codex-rs/core/tests/suite/sqlite_state.rs
M	codex-rs/core/tests/suite/stream_error_allows_next_turn.rs
M	codex-rs/core/tests/suite/stream_no_completed.rs
M	codex-rs/core/tests/suite/subagent_notifications.rs
M	codex-rs/core/tests/suite/tools.rs
M	codex-rs/core/tests/suite/unified_exec.rs
M	codex-rs/core/tests/suite/unified_exec_process_events.rs
M	codex-rs/core/tests/suite/unified_exec_zsh_fork_approvals.rs
M	codex-rs/core/tests/suite/unstable_features_warning.rs
M	codex-rs/core/tests/suite/view_image.rs
M	codex-rs/core/tests/suite/windows_sandbox.rs
M	codex-rs/deny.toml
M	codex-rs/exec-server-protocol/src/lib.rs
A	codex-rs/exec-server-protocol/src/network_policy.rs
A	codex-rs/exec-server-protocol/src/network_policy_tests.rs
M	codex-rs/exec-server-protocol/src/protocol.rs
M	codex-rs/exec-server/Cargo.toml
M	codex-rs/exec-server/README.md
M	codex-rs/exec-server/src/client.rs
M	codex-rs/exec-server/src/client/http_client.rs
M	codex-rs/exec-server/src/client/http_response_body_stream.rs
R073	codex-rs/exec-server/src/client/reqwest_http_client.rs	codex-rs/exec-server/src/client/route_aware_http_client.rs
M	codex-rs/exec-server/src/client_api.rs
M	codex-rs/exec-server/src/client_recovery.rs
M	codex-rs/exec-server/src/client_recovery_tests.rs
M	codex-rs/exec-server/src/client_transport.rs
M	codex-rs/exec-server/src/client_transport_tests.rs
M	codex-rs/exec-server/src/connection.rs
M	codex-rs/exec-server/src/environment.rs
A	codex-rs/exec-server/src/environment_bootstrap.rs
A	codex-rs/exec-server/src/environment_bootstrap_tests.rs
M	codex-rs/exec-server/src/environment_provider.rs
M	codex-rs/exec-server/src/environment_toml.rs
M	codex-rs/exec-server/src/fs_sandbox.rs
M	codex-rs/exec-server/src/lib.rs
M	codex-rs/exec-server/src/local_process.rs
A	codex-rs/exec-server/src/network_policy_decisions.rs
A	codex-rs/exec-server/src/network_policy_decisions_tests.rs
M	codex-rs/exec-server/src/process_sandbox.rs
M	codex-rs/exec-server/src/process_sandbox_tests.rs
M	codex-rs/exec-server/src/relay.rs
M	codex-rs/exec-server/src/remote.rs
M	codex-rs/exec-server/src/remote/noise_tests.rs
M	codex-rs/exec-server/src/remote_file_system.rs
M	codex-rs/exec-server/src/remote_file_system_path_uri_tests.rs
M	codex-rs/exec-server/src/rpc.rs
A	codex-rs/exec-server/src/rpc_server_requests.rs
A	codex-rs/exec-server/src/rpc_server_requests_tests.rs
M	codex-rs/exec-server/src/server.rs
M	codex-rs/exec-server/src/server/handler.rs
M	codex-rs/exec-server/src/server/handler/tests.rs
M	codex-rs/exec-server/src/server/processor.rs
M	codex-rs/exec-server/src/server/transport.rs
M	codex-rs/exec-server/src/server/transport_tests.rs
M	codex-rs/exec-server/src/trace_context.rs
M	codex-rs/exec-server/testing/BUILD.bazel
M	codex-rs/exec-server/testing/exec_server.rs
M	codex-rs/exec-server/tests/chatgpt_cloudflare_affinity.rs
M	codex-rs/exec-server/tests/common/mod.rs
M	codex-rs/exec-server/tests/deferred_environment.rs
M	codex-rs/exec-server/tests/environment.rs
M	codex-rs/exec-server/tests/exec_process.rs
M	codex-rs/exec-server/tests/file_stream.rs
M	codex-rs/exec-server/tests/file_system/support.rs
M	codex-rs/exec-server/tests/file_system_unix.rs
M	codex-rs/exec-server/tests/health.rs
M	codex-rs/exec-server/tests/http_client.rs
M	codex-rs/exec-server/tests/http_request.rs
A	codex-rs/exec-server/tests/http_request_logging.rs
M	codex-rs/exec-server/tests/relay.rs
M	codex-rs/exec-server/tests/selected_capability_roots.rs
A	codex-rs/exec-server/tests/support/BUILD.bazel
A	codex-rs/exec-server/tests/support/Cargo.toml
A	codex-rs/exec-server/tests/support/lib.rs
M	codex-rs/exec/src/event_processor_with_human_output_tests.rs
M	codex-rs/exec/src/lib.rs
M	codex-rs/exec/src/lib_tests.rs
M	codex-rs/exec/tests/event_processor_with_json_output.rs
M	codex-rs/exec/tests/suite/apply_patch.rs
M	codex-rs/ext/agent/src/lib.rs
M	codex-rs/ext/extension-api/Cargo.toml
M	codex-rs/ext/extension-api/examples/enabled_extensions.rs
M	codex-rs/ext/extension-api/examples/enabled_extensions/shared_state_extension.rs
M	codex-rs/ext/extension-api/src/capabilities/events.rs
M	codex-rs/ext/extension-api/src/capabilities/mod.rs
M	codex-rs/ext/extension-api/src/contributors.rs
M	codex-rs/ext/extension-api/src/contributors/context.rs
M	codex-rs/ext/extension-api/src/contributors/thread_lifecycle.rs
M	codex-rs/ext/extension-api/src/contributors/world_state.rs
M	codex-rs/ext/extension-api/src/lib.rs
M	codex-rs/ext/extension-api/tests/registry.rs
A	codex-rs/ext/git-attribution/BUILD.bazel
A	codex-rs/ext/git-attribution/Cargo.toml
A	codex-rs/ext/git-attribution/src/git_attribution_tests.rs
A	codex-rs/ext/git-attribution/src/lib.rs
A	codex-rs/ext/git-attribution/src/policy.rs
A	codex-rs/ext/git-attribution/src/world_state.rs
M	codex-rs/ext/goal/Cargo.toml
M	codex-rs/ext/goal/src/extension.rs
M	codex-rs/ext/goal/tests/goal_extension_backend.rs
M	codex-rs/ext/image-generation/src/backend.rs
M	codex-rs/ext/image-generation/src/extension.rs
M	codex-rs/ext/mcp/tests/hosted_apps_mcp.rs
M	codex-rs/ext/memories/src/extension.rs
M	codex-rs/ext/memories/src/tests.rs
M	codex-rs/ext/skills/Cargo.toml
M	codex-rs/ext/skills/src/catalog.rs
M	codex-rs/ext/skills/src/extension.rs
M	codex-rs/ext/skills/src/fragments.rs
M	codex-rs/ext/skills/src/lib.rs
M	codex-rs/ext/skills/src/provider.rs
M	codex-rs/ext/skills/src/provider/executor.rs
M	codex-rs/ext/skills/src/provider/host.rs
A	codex-rs/ext/skills/src/provider/host_tests.rs
M	codex-rs/ext/skills/src/provider/orchestrator.rs
M	codex-rs/ext/skills/src/render.rs
A	codex-rs/ext/skills/src/render_tests.rs
M	codex-rs/ext/skills/src/shadow_selection_experiment.rs
M	codex-rs/ext/skills/src/state.rs
M	codex-rs/ext/skills/src/tools/list.rs
M	codex-rs/ext/skills/src/tools/mod.rs
M	codex-rs/ext/skills/src/tools/read.rs
A	codex-rs/ext/skills/src/warnings.rs
M	codex-rs/ext/skills/src/world_state.rs
M	codex-rs/ext/skills/tests/executor_file_system_authority.rs
M	codex-rs/ext/skills/tests/skills_extension.rs
M	codex-rs/ext/web-search/src/extension.rs
M	codex-rs/ext/web-search/src/tool.rs
M	codex-rs/external-agent-migration/src/service_tests/plugins/basics.rs
M	codex-rs/external-agent-migration/src/source_cla.rs
M	codex-rs/features/src/feature_configs.rs
M	codex-rs/features/src/lib.rs
M	codex-rs/features/src/tests.rs
M	codex-rs/feedback/src/lib.rs
M	codex-rs/file-system/src/lib.rs
M	codex-rs/git-utils/src/info.rs
M	codex-rs/git-utils/src/lib.rs
M	codex-rs/hooks/src/engine/command_runner.rs
M	codex-rs/hooks/src/engine/command_runner_tests.rs
M	codex-rs/http-client/README.md
R088	codex-rs/http-client/src/default_client.rs	codex-rs/http-client/src/client.rs
A	codex-rs/http-client/src/client_builder.rs
A	codex-rs/http-client/src/client_builder_tests.rs
M	codex-rs/http-client/src/lib.rs
M	codex-rs/http-client/src/outbound_proxy.rs
M	codex-rs/http-client/src/outbound_proxy_tests.rs
M	codex-rs/http-client/src/route_aware_client_pool.rs
M	codex-rs/http-client/src/route_aware_client_pool_tests.rs
M	codex-rs/http-client/src/transport.rs
A	codex-rs/http-client/src/transport_tests.rs
M	codex-rs/linux-sandbox/src/bwrap.rs
M	codex-rs/linux-sandbox/src/linux_run_main.rs
M	codex-rs/linux-sandbox/src/linux_run_main_tests.rs
M	codex-rs/linux-sandbox/src/proxy_routing.rs
M	codex-rs/linux-sandbox/tests/suite/landlock.rs
M	codex-rs/linux-sandbox/tests/suite/managed_proxy.rs
M	codex-rs/lmstudio/Cargo.toml
M	codex-rs/lmstudio/src/client.rs
M	codex-rs/login/Cargo.toml
M	codex-rs/login/src/auth/agent_identity.rs
M	codex-rs/login/src/auth/auth_headers.rs
M	codex-rs/login/src/auth/auth_tests.rs
M	codex-rs/login/src/auth/bedrock_api_key_tests.rs
M	codex-rs/login/src/auth/default_client.rs
M	codex-rs/login/src/auth/default_client_tests.rs
M	codex-rs/login/src/auth/manager.rs
M	codex-rs/login/src/auth/personal_access_token.rs
M	codex-rs/login/src/auth/revoke.rs
M	codex-rs/login/src/auth_env_telemetry.rs
M	codex-rs/login/src/device_code_auth.rs
M	codex-rs/login/src/lib.rs
M	codex-rs/login/src/outbound_proxy.rs
M	codex-rs/login/src/server.rs
A	codex-rs/login/src/test_support.rs
M	codex-rs/login/src/token_data_tests.rs
M	codex-rs/login/tests/suite/auth_refresh.rs
M	codex-rs/login/tests/suite/device_code_login.rs
M	codex-rs/login/tests/suite/login_server_e2e.rs
M	codex-rs/login/tests/suite/logout.rs
M	codex-rs/mcp-server/Cargo.toml
M	codex-rs/mcp-server/src/codex_tool_runner.rs
M	codex-rs/mcp-server/src/lib.rs
M	codex-rs/mcp-server/src/message_processor.rs
M	codex-rs/mcp-server/tests/suite/codex_tool.rs
M	codex-rs/memories/write/src/runtime.rs
M	codex-rs/memories/write/src/startup_tests.rs
M	codex-rs/model-provider-info/src/lib.rs
M	codex-rs/model-provider-info/src/model_provider_info_tests.rs
M	codex-rs/model-provider/src/amazon_bedrock/error.rs
M	codex-rs/model-provider/src/amazon_bedrock/error_tests.rs
M	codex-rs/model-provider/src/auth.rs
M	codex-rs/model-provider/src/models_endpoint.rs
M	codex-rs/model-provider/src/provider.rs
M	codex-rs/models-manager/models.json
M	codex-rs/models-manager/src/manager_tests.rs
M	codex-rs/network-proxy/Cargo.toml
M	codex-rs/network-proxy/README.md
M	codex-rs/network-proxy/src/config.rs
M	codex-rs/network-proxy/src/connect_policy.rs
M	codex-rs/network-proxy/src/http_proxy.rs
M	codex-rs/network-proxy/src/lib.rs
M	codex-rs/network-proxy/src/mitm.rs
M	codex-rs/network-proxy/src/proxy.rs
M	codex-rs/network-proxy/src/remote_config.rs
M	codex-rs/network-proxy/src/remote_config_tests.rs
M	codex-rs/network-proxy/src/socks5.rs
M	codex-rs/network-proxy/src/state.rs
M	codex-rs/network-proxy/src/upstream.rs
M	codex-rs/network-proxy/src/upstream_tests.rs
A	codex-rs/network-proxy/src/windows_proxy_ingress.rs
A	codex-rs/network-proxy/src/windows_proxy_ingress_tests.rs
A	codex-rs/network-proxy/src/windows_tcp_attribution.rs
A	codex-rs/network-proxy/src/windows_tcp_attribution_tests.rs
A	codex-rs/network-proxy/tests/windows_stable_ingress.rs
M	codex-rs/plugin/src/load_outcome.rs
M	codex-rs/plugin/src/plugin_id.rs
M	codex-rs/prompts/src/permissions_instructions_tests.rs
M	codex-rs/prompts/templates/permissions/sandbox_mode/danger_full_access.md
M	codex-rs/prompts/templates/permissions/sandbox_mode/read_only.md
M	codex-rs/prompts/templates/permissions/sandbox_mode/workspace_write.md
M	codex-rs/prompts/templates/review/rubric.md
M	codex-rs/protocol/src/account.rs
M	codex-rs/protocol/src/approvals.rs
M	codex-rs/protocol/src/auth.rs
M	codex-rs/protocol/src/config_types.rs
M	codex-rs/protocol/src/error.rs
M	codex-rs/protocol/src/error_tests.rs
M	codex-rs/protocol/src/items.rs
M	codex-rs/protocol/src/legacy_events.rs
M	codex-rs/protocol/src/models.rs
M	codex-rs/protocol/src/permissions.rs
M	codex-rs/protocol/src/protocol.rs
M	codex-rs/rmcp-client/Cargo.toml
M	codex-rs/rmcp-client/src/auth_status.rs
M	codex-rs/rmcp-client/src/lib.rs
M	codex-rs/rmcp-client/src/oauth.rs
M	codex-rs/rmcp-client/src/oauth/tests/persistor_tests.rs
M	codex-rs/rmcp-client/src/oauth_http_client.rs
M	codex-rs/rmcp-client/src/perform_oauth_login.rs
M	codex-rs/rmcp-client/src/rmcp_client.rs
M	codex-rs/rmcp-client/src/utils.rs
M	codex-rs/rmcp-client/tests/streamable_http_oauth_startup.rs
M	codex-rs/rmcp-client/tests/streamable_http_test_support.rs
A	codex-rs/rmcp-client/tests/streamable_http_user_agent.rs
M	codex-rs/rollout-trace/src/protocol_event.rs
M	codex-rs/rollout-trace/src/protocol_event_tests.rs
M	codex-rs/rollout/Cargo.toml
M	codex-rs/rollout/src/compression.rs
M	codex-rs/rollout/src/compression_tests.rs
M	codex-rs/rollout/src/config.rs
M	codex-rs/rollout/src/lib.rs
M	codex-rs/rollout/src/list.rs
M	codex-rs/rollout/src/metadata_tests.rs
M	codex-rs/rollout/src/ordinal.rs
M	codex-rs/rollout/src/recorder.rs
M	codex-rs/rollout/src/recorder_tests.rs
M	codex-rs/rollout/src/reverse_jsonl_scanner.rs
M	codex-rs/rollout/src/reverse_jsonl_scanner_tests.rs
M	codex-rs/rollout/src/search.rs
M	codex-rs/rollout/src/state_db.rs
M	codex-rs/rollout/src/state_db_tests.rs
M	codex-rs/rollout/src/tests.rs
M	codex-rs/sandboxing/src/manager.rs
M	codex-rs/sandboxing/src/manager_tests.rs
M	codex-rs/sandboxing/src/policy_transforms.rs
M	codex-rs/sandboxing/src/policy_transforms_tests.rs
M	codex-rs/sandboxing/src/seatbelt_tests.rs
M	codex-rs/sandboxing/src/spawn.rs
M	codex-rs/sandboxing/src/windows.rs
M	codex-rs/shell-command/src/parse_command.rs
M	codex-rs/skills/src/model.rs
A	codex-rs/state/migrations/0043_threads_is_pinned.sql
A	codex-rs/state/migrations/0044_external_agent_config_imports_provider_id.sql
M	codex-rs/state/src/audit.rs
M	codex-rs/state/src/bin/logs_client.rs
M	codex-rs/state/src/extract.rs
M	codex-rs/state/src/lib.rs
M	codex-rs/state/src/log_db.rs
M	codex-rs/state/src/log_db_filter_tests.rs
M	codex-rs/state/src/migrations_tests.rs
M	codex-rs/state/src/model/thread_metadata.rs
M	codex-rs/state/src/runtime.rs
M	codex-rs/state/src/runtime/backfill.rs
M	codex-rs/state/src/runtime/external_agent_config_imports.rs
M	codex-rs/state/src/runtime/external_agent_config_imports_tests.rs
M	codex-rs/state/src/runtime/goals.rs
M	codex-rs/state/src/runtime/logs.rs
M	codex-rs/state/src/runtime/memories.rs
M	codex-rs/state/src/runtime/recovery_tests.rs
M	codex-rs/state/src/runtime/remote_control.rs
M	codex-rs/state/src/runtime/test_support.rs
M	codex-rs/state/src/runtime/threads.rs
M	codex-rs/state/src/sqlite.rs
A	codex-rs/state/thread_history_migrations/0004_thread_items_updated_at_ordinal.sql
M	codex-rs/thread-manager-sample/src/main.rs
M	codex-rs/thread-store/Cargo.toml
M	codex-rs/thread-store/src/in_memory.rs
M	codex-rs/thread-store/src/lib.rs
M	codex-rs/thread-store/src/local/archive_thread.rs
M	codex-rs/thread-store/src/local/create_thread.rs
M	codex-rs/thread-store/src/local/delete_thread.rs
M	codex-rs/thread-store/src/local/helpers.rs
M	codex-rs/thread-store/src/local/list_threads.rs
M	codex-rs/thread-store/src/local/live_writer.rs
M	codex-rs/thread-store/src/local/mod.rs
M	codex-rs/thread-store/src/local/model_context.rs
M	codex-rs/thread-store/src/local/model_context_tests.rs
A	codex-rs/thread-store/src/local/paginated_fork.rs
M	codex-rs/thread-store/src/local/read_thread.rs
M	codex-rs/thread-store/src/local/rollout_lineage.rs
M	codex-rs/thread-store/src/local/rollout_lineage_tests.rs
M	codex-rs/thread-store/src/local/search_threads.rs
M	codex-rs/thread-store/src/local/test_support.rs
M	codex-rs/thread-store/src/local/thread_history.rs
M	codex-rs/thread-store/src/local/thread_history/read.rs
M	codex-rs/thread-store/src/local/thread_history/read_tests.rs
M	codex-rs/thread-store/src/local/thread_history/search.rs
M	codex-rs/thread-store/src/local/thread_history/segment_paging.rs
A	codex-rs/thread-store/src/local/thread_history/turn_lookup.rs
M	codex-rs/thread-store/src/local/thread_history_materialization.rs
M	codex-rs/thread-store/src/local/thread_history_materialization_tests.rs
M	codex-rs/thread-store/src/local/unarchive_thread.rs
M	codex-rs/thread-store/src/local/update_thread_metadata.rs
A	codex-rs/thread-store/src/local/writer_lock.rs
A	codex-rs/thread-store/src/local/writer_lock_tests.rs
M	codex-rs/thread-store/src/store.rs
M	codex-rs/thread-store/src/types.rs
M	codex-rs/tools/Cargo.toml
M	codex-rs/tools/src/lib.rs
M	codex-rs/tools/src/tool_spec.rs
M	codex-rs/tools/src/tool_spec_tests.rs
M	codex-rs/tui/src/additional_dirs.rs
M	codex-rs/tui/src/app/agent_status_feed_tests.rs
M	codex-rs/tui/src/app/config_persistence.rs
M	codex-rs/tui/src/app/event_dispatch.rs
M	codex-rs/tui/src/app/input.rs
M	codex-rs/tui/src/app/loaded_threads.rs
M	codex-rs/tui/src/app/plugin_mentions.rs
M	codex-rs/tui/src/app/session_lifecycle.rs
M	codex-rs/tui/src/app/side.rs
A	codex-rs/tui/src/app/snapshots/codex_tui__app__tests__directive_only_completion_removes_streamed_directive.snap
M	codex-rs/tui/src/app/snapshots/codex_tui__app__tests__required_stream_reflow_during_capped_initial_replay.snap
M	codex-rs/tui/src/app/snapshots/codex_tui__app__tests__required_stream_reflow_during_capped_initial_replay_survives_transcript_overlay.snap
M	codex-rs/tui/src/app/tests.rs
M	codex-rs/tui/src/app/tests/advanced_reasoning_tests.rs
M	codex-rs/tui/src/app/tests/safety_buffering.rs
M	codex-rs/tui/src/app/tests/session_lifecycle_requests.rs
A	codex-rs/tui/src/app/tests/turn_submission.rs
M	codex-rs/tui/src/app/thread_events.rs
M	codex-rs/tui/src/app/thread_routing.rs
M	codex-rs/tui/src/app/thread_session_state.rs
M	codex-rs/tui/src/app_event.rs
M	codex-rs/tui/src/app_info.rs
M	codex-rs/tui/src/app_server_session.rs
M	codex-rs/tui/src/auto_review_denials.rs
M	codex-rs/tui/src/bottom_pane/chat_composer.rs
M	codex-rs/tui/src/bottom_pane/footer.rs
M	codex-rs/tui/src/bottom_pane/mentions_v2/popup.rs
M	codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__chat_composer__tests__default_unified_mention_popup.snap
M	codex-rs/tui/src/bottom_pane/snapshots/codex_tui__bottom_pane__chat_composer__tests__unified_mention_popup_falls_back_from_bound_plugin_on_right.snap
M	codex-rs/tui/src/chatwidget/interrupts.rs
M	codex-rs/tui/src/chatwidget/protocol.rs
M	codex-rs/tui/src/chatwidget/protocol_requests.rs
M	codex-rs/tui/src/chatwidget/replay.rs
M	codex-rs/tui/src/chatwidget/slash_dispatch.rs
A	codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__live_app_server_turn_completion_repairs_dropped_message_deltas.snap
M	codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__side_context_label_preserves_status_line.snap
A	codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__side_context_label_shows_hidden_side.snap
M	codex-rs/tui/src/chatwidget/snapshots/codex_tui__chatwidget__tests__side_context_label_shows_parent_status.snap
M	codex-rs/tui/src/chatwidget/streaming.rs
M	codex-rs/tui/src/chatwidget/tests/app_server.rs
M	codex-rs/tui/src/chatwidget/tests/composer_submission.rs
M	codex-rs/tui/src/chatwidget/tests/exec_flow.rs
M	codex-rs/tui/src/chatwidget/tests/guardian.rs
M	codex-rs/tui/src/chatwidget/tests/helpers.rs
M	codex-rs/tui/src/chatwidget/tests/history_replay.rs
M	codex-rs/tui/src/chatwidget/tests/mcp_startup.rs
M	codex-rs/tui/src/chatwidget/tests/permissions.rs
M	codex-rs/tui/src/chatwidget/tests/popups_and_settings.rs
M	codex-rs/tui/src/chatwidget/tests/side.rs
M	codex-rs/tui/src/chatwidget/tests/slash_commands.rs
M	codex-rs/tui/src/chatwidget/tests/status_and_layout.rs
M	codex-rs/tui/src/chatwidget/transcript.rs
M	codex-rs/tui/src/chatwidget/turn_runtime.rs
M	codex-rs/tui/src/custom_terminal.rs
M	codex-rs/tui/src/debug_config.rs
M	codex-rs/tui/src/history_cell/messages.rs
M	codex-rs/tui/src/history_cell/session.rs
M	codex-rs/tui/src/history_cell/snapshots/codex_tui__history_cell__tests__raw_mode_toggle_transcript.snap
A	codex-rs/tui/src/history_cell/snapshots/codex_tui__history_cell__tests__session_header_clamps_to_narrow_width.snap
A	codex-rs/tui/src/history_cell/snapshots/codex_tui__history_cell__tests__single_line_command_over_highlight_limit_uses_plain_text_fallback.snap
M	codex-rs/tui/src/history_cell/tests.rs
M	codex-rs/tui/src/keymap.rs
M	codex-rs/tui/src/keymap_setup/actions.rs
M	codex-rs/tui/src/lib.rs
M	codex-rs/tui/src/onboarding/auth.rs
A	codex-rs/tui/src/onboarding/snapshots/codex_tui__onboarding__auth__tests__continue_in_browser_narrow_long_url.snap
M	codex-rs/tui/src/permission_compat.rs
M	codex-rs/tui/src/render/highlight.rs
M	codex-rs/tui/src/resume_picker.rs
M	codex-rs/tui/src/session_archive_commands.rs
M	codex-rs/tui/src/session_log.rs
M	codex-rs/tui/src/session_resume.rs
M	codex-rs/tui/src/slash_command.rs
M	codex-rs/tui/src/snapshots/codex_tui__debug_config__tests__debug_config_requirement_sources.snap
M	codex-rs/tui/src/snapshots/codex_tui__inline_visualization__tests__agent_code_blocks_preserve_visualization_directive_literals.snap
M	codex-rs/tui/src/snapshots/codex_tui__inline_visualization__tests__finalized_agent_cell_visualization_link.snap
M	codex-rs/tui/src/snapshots/codex_tui__keymap_setup__tests__keymap_picker_all_tab_search.snap
M	codex-rs/tui/src/snapshots/codex_tui__keymap_setup__tests__keymap_picker_custom.snap
M	codex-rs/tui/src/snapshots/codex_tui__keymap_setup__tests__keymap_picker_fast_mode_enabled.snap
M	codex-rs/tui/src/snapshots/codex_tui__keymap_setup__tests__keymap_picker_first_actions.snap
M	codex-rs/tui/src/snapshots/codex_tui__keymap_setup__tests__keymap_picker_narrow.snap
M	codex-rs/tui/src/snapshots/codex_tui__keymap_setup__tests__keymap_picker_wide.snap
M	codex-rs/tui/src/status/tests.rs
M	codex-rs/tui/src/tui.rs
M	codex-rs/tui/src/tui/event_stream.rs
M	codex-rs/tui/src/tui/keyboard_modes.rs
A	codex-rs/tui/src/tui/windows_console.rs
M	codex-rs/utils/path-uri/src/api_path_string_tests.rs
M	codex-rs/utils/path-uri/src/lib.rs
M	codex-rs/utils/path-uri/src/tests.rs
M	codex-rs/utils/plugins/src/lib.rs
M	codex-rs/utils/plugins/src/plugin_namespace.rs
M	codex-rs/utils/pty/Cargo.toml
M	codex-rs/utils/pty/src/lib.rs
M	codex-rs/utils/pty/src/pipe.rs
A	codex-rs/utils/pty/src/pipe_tests.rs
A	codex-rs/utils/pty/src/win/job.rs
M	codex-rs/utils/pty/src/win/mod.rs
M	codex-rs/utils/pty/src/win/procthreadattr.rs
M	codex-rs/utils/pty/src/win/psuedocon.rs
M	codex-rs/utils/pty/src/windows_tests.rs
M	codex-rs/v8-poc/BUILD.bazel
M	codex-rs/websocket-client/src/dialer.rs
M	codex-rs/websocket-client/src/dialer_tests.rs
M	codex-rs/websocket-client/src/lib.rs
M	codex-rs/windows-sandbox-rs/BUILD.bazel
M	codex-rs/windows-sandbox-rs/src/acl.rs
M	codex-rs/windows-sandbox-rs/src/bin/command_runner/win.rs
M	codex-rs/windows-sandbox-rs/src/bin/setup_main/win.rs
M	codex-rs/windows-sandbox-rs/src/conpty/mod.rs
M	codex-rs/windows-sandbox-rs/src/deny_read_resolver.rs
M	codex-rs/windows-sandbox-rs/src/elevated/ipc_framed.rs
M	codex-rs/windows-sandbox-rs/src/elevated/runner_client.rs
M	codex-rs/windows-sandbox-rs/src/elevated_impl.rs
M	codex-rs/windows-sandbox-rs/src/identity.rs
M	codex-rs/windows-sandbox-rs/src/lib.rs
M	codex-rs/windows-sandbox-rs/src/proc_thread_attr.rs
M	codex-rs/windows-sandbox-rs/src/process.rs
M	codex-rs/windows-sandbox-rs/src/resolved_permissions.rs
M	codex-rs/windows-sandbox-rs/src/setup.rs
M	codex-rs/windows-sandbox-rs/src/token.rs
A	codex-rs/windows-sandbox-rs/src/token_tests.rs
M	codex-rs/windows-sandbox-rs/src/unified_exec/backends/elevated.rs
M	codex-rs/windows-sandbox-rs/src/unified_exec/backends/elevated_tests.rs
M	codex-rs/windows-sandbox-rs/src/unified_exec/backends/legacy.rs
M	codex-rs/windows-sandbox-rs/src/unified_exec/mod.rs
M	codex-rs/windows-sandbox-rs/src/unified_exec/tests.rs
M	codex-rs/windows-sandbox-rs/src/wrapper.rs
M	codex-rs/windows-sandbox-rs/src/wrapper_tests.rs
M	defs.bzl
M	patches/BUILD.bazel
D	patches/aws-lc-sys_memcmp_check.patch
D	patches/aws-lc-sys_windows_msvc_memcmp_probe.patch
D	patches/aws-lc-sys_windows_msvc_prebuilt_nasm.patch
M	patches/llvm_rusty_v8_custom_libcxx.patch
M	patches/ring_windows_msvc_include_dirs.patch
M	patches/rules_rs_build_script_deps_annotation.patch
D	patches/rules_rs_windows_exec_linker.patch
D	patches/rules_rs_windows_gnullvm_exec.patch
A	patches/rules_rust_build_script_tools_transition.patch
D	patches/rules_rust_windows_bootstrap_process_wrapper_linker.patch
D	patches/rules_rust_windows_build_script_runner_paths.patch
D	patches/rules_rust_windows_exec_bin_target.patch
D	patches/rules_rust_windows_exec_msvc_build_script_env.patch
D	patches/rules_rust_windows_exec_rustc_dev_rlib.patch
D	patches/rules_rust_windows_exec_std.patch
D	patches/rules_rust_windows_gnullvm_build_script.patch
M	patches/rules_rust_windows_msvc_direct_link_args.patch
M	patches/rules_rust_windows_process_wrapper_skip_temp_outputs.patch
M	scripts/codex_package/README.md
M	scripts/codex_package/cli.py
M	scripts/codex_package/test_layout.py
M	scripts/codex_package/test_zsh.py
M	scripts/codex_package/zsh.py
M	scripts/install/install.ps1
M	scripts/install/install.sh
M	scripts/install/test_install_sh.py
M	third_party/v8/BUILD.bazel
M	tools/argument-comment-lint/lint_aspect.bzl
```
</details>

<details><summary>OpenCode: 246 name-status records</summary>

```text
M	bun.lock
M	nix/hashes.json
M	package.json
A	packages/app/V1_API_MIGRATION.md
M	packages/app/e2e/performance/timeline-stability/fixture.ts
M	packages/app/e2e/performance/timeline/session-parent-hydration-benchmark.spec.ts
M	packages/app/e2e/regression/cross-server-tab-close.spec.ts
M	packages/app/e2e/regression/remote-session-settings.spec.ts
M	packages/app/e2e/regression/remote-tab-busy.spec.ts
M	packages/app/e2e/regression/review-line-comment.spec.ts
M	packages/app/e2e/regression/review-open-file.spec.ts
M	packages/app/e2e/regression/review-state-persistence.spec.ts
M	packages/app/e2e/regression/review-terminal-stacked.spec.ts
M	packages/app/e2e/regression/session-list-path-loading.spec.ts
M	packages/app/e2e/regression/session-request-docks.spec.ts
M	packages/app/e2e/regression/session-timeline-lifecycle-state.spec.ts
M	packages/app/e2e/regression/session-timeline-transport.spec.ts
M	packages/app/e2e/regression/session-todo-dock-navigation.spec.ts
M	packages/app/e2e/regression/subagent-child-navigation.spec.ts
M	packages/app/e2e/regression/tab-navigate-mousedown.spec.ts
M	packages/app/e2e/regression/terminal-composer-focus.spec.ts
M	packages/app/e2e/regression/terminal-hidden.spec.ts
M	packages/app/e2e/regression/terminal-tab-switch.spec.ts
M	packages/app/e2e/utils/mock-server.ts
M	packages/app/e2e/utils/sse-transport.ts
M	packages/app/package.json
M	packages/app/src/app.tsx
M	packages/app/src/components/command-palette.ts
M	packages/app/src/components/dialog-command-palette-v2.tsx
M	packages/app/src/components/dialog-connect-provider.tsx
M	packages/app/src/components/dialog-custom-provider.tsx
M	packages/app/src/components/dialog-fork.tsx
M	packages/app/src/components/dialog-select-directory-v2.tsx
M	packages/app/src/components/dialog-select-directory.tsx
M	packages/app/src/components/dialog-select-mcp.tsx
M	packages/app/src/components/dialog-select-server.tsx
M	packages/app/src/components/directory-picker-domain.test.ts
M	packages/app/src/components/directory-picker-domain.ts
M	packages/app/src/components/edit-project.ts
M	packages/app/src/components/prompt-input-v2.tsx
M	packages/app/src/components/prompt-input.tsx
M	packages/app/src/components/prompt-input/submit.test.ts
M	packages/app/src/components/prompt-input/submit.ts
M	packages/app/src/components/server/server-row-menu.tsx
M	packages/app/src/components/settings-general.tsx
M	packages/app/src/components/settings-providers.tsx
M	packages/app/src/components/settings-v2/dialog-settings-v2.tsx
M	packages/app/src/components/settings-v2/general.tsx
M	packages/app/src/components/settings-v2/providers.tsx
M	packages/app/src/components/status-popover-body.tsx
M	packages/app/src/components/status-popover-indicator.test.ts
M	packages/app/src/components/status-popover-indicator.ts
M	packages/app/src/components/terminal.tsx
M	packages/app/src/components/titlebar-tab-nav.tsx
M	packages/app/src/components/titlebar.tsx
M	packages/app/src/context/directory-sync.ts
M	packages/app/src/context/file.tsx
M	packages/app/src/context/global-sync/bootstrap.test.ts
M	packages/app/src/context/global-sync/bootstrap.ts
M	packages/app/src/context/global-sync/child-store.ts
M	packages/app/src/context/global-sync/event-reducer.test.ts
M	packages/app/src/context/global-sync/event-reducer.ts
M	packages/app/src/context/global-sync/mcp.test.ts
M	packages/app/src/context/global-sync/mcp.ts
M	packages/app/src/context/global-sync/session-cache.test.ts
M	packages/app/src/context/global-sync/session-cache.ts
M	packages/app/src/context/global-sync/session-load.ts
M	packages/app/src/context/global-sync/types.ts
M	packages/app/src/context/global-sync/utils.test.ts
M	packages/app/src/context/global-sync/utils.ts
M	packages/app/src/context/layout.tsx
M	packages/app/src/context/models.tsx
M	packages/app/src/context/permission.tsx
M	packages/app/src/context/server-sdk.test.ts
M	packages/app/src/context/server-sdk.tsx
A	packages/app/src/context/server-session-v2-reducer.test.ts
A	packages/app/src/context/server-session-v2-reducer.ts
M	packages/app/src/context/server-session.test.ts
M	packages/app/src/context/server-session.ts
M	packages/app/src/context/server-sync.test.ts
M	packages/app/src/context/server-sync.tsx
M	packages/app/src/context/settings.test.ts
M	packages/app/src/context/settings.tsx
M	packages/app/src/context/terminal.tsx
M	packages/app/src/hooks/use-providers.ts
M	packages/app/src/pages/home-session-archive.test.ts
M	packages/app/src/pages/home-session-archive.ts
M	packages/app/src/pages/home.tsx
A	packages/app/src/pages/home/home-controller.ts
A	packages/app/src/pages/home/home-projects-controller.tsx
A	packages/app/src/pages/home/home-projects-view.tsx
A	packages/app/src/pages/home/home-projects.tsx
A	packages/app/src/pages/home/home-scroll-controller.ts
A	packages/app/src/pages/home/home-session-search-controller.ts
A	packages/app/src/pages/home/home-sessions-controller.tsx
A	packages/app/src/pages/home/home-sessions-view.tsx
A	packages/app/src/pages/home/home-sessions.tsx
A	packages/app/src/pages/home/legacy-home.tsx
M	packages/app/src/pages/layout.tsx
M	packages/app/src/pages/layout/project-avatar-state.ts
M	packages/app/src/pages/layout/session-tab-avatar.tsx
M	packages/app/src/pages/session.tsx
M	packages/app/src/pages/session/composer/session-composer-controls.ts
M	packages/app/src/pages/session/composer/session-composer-state.ts
M	packages/app/src/pages/session/composer/session-question-dock.tsx
M	packages/app/src/pages/session/file-tabs.tsx
M	packages/app/src/pages/session/review-tab.tsx
M	packages/app/src/pages/session/session-side-panel.tsx
M	packages/app/src/pages/session/timeline/message-timeline.tsx
M	packages/app/src/pages/session/timeline/projection.ts
A	packages/app/src/pages/session/timeline/rows-current.test.ts
M	packages/app/src/pages/session/timeline/rows.ts
M	packages/app/src/pages/session/use-session-commands.tsx
M	packages/app/src/pages/session/v2/review-diff-kinds.ts
M	packages/app/src/pages/session/v2/review-panel-v2.tsx
M	packages/app/src/pages/session/v2/session-file-browser-tab.tsx
M	packages/app/src/utils/diffs.test.ts
M	packages/app/src/utils/diffs.ts
A	packages/app/src/utils/server-compat.test.ts
A	packages/app/src/utils/server-compat.ts
M	packages/app/src/utils/server-health.test.ts
M	packages/app/src/utils/server-health.ts
A	packages/app/src/utils/server-protocol.test.ts
A	packages/app/src/utils/server-protocol.ts
M	packages/app/src/utils/server.ts
A	packages/app/src/utils/session-message.test.ts
A	packages/app/src/utils/session-message.ts
A	packages/app/src/utils/session.test.ts
A	packages/app/src/utils/session.ts
M	packages/app/src/utils/terminal-websocket-url.test.ts
M	packages/app/src/utils/terminal-websocket-url.ts
M	packages/app/test-browser/command-palette.test.ts
A	packages/app/vendor/opencode-ai-client-1.17.13-v2.tgz
M	packages/cli/package.json
M	packages/codemode/package.json
M	packages/console/app/package.json
M	packages/console/app/src/i18n/ar.ts
M	packages/console/app/src/i18n/br.ts
M	packages/console/app/src/i18n/da.ts
M	packages/console/app/src/i18n/de.ts
M	packages/console/app/src/i18n/en.ts
M	packages/console/app/src/i18n/es.ts
M	packages/console/app/src/i18n/fr.ts
M	packages/console/app/src/i18n/it.ts
M	packages/console/app/src/i18n/ja.ts
M	packages/console/app/src/i18n/ko.ts
M	packages/console/app/src/i18n/no.ts
M	packages/console/app/src/i18n/pl.ts
M	packages/console/app/src/i18n/ru.ts
M	packages/console/app/src/i18n/th.ts
M	packages/console/app/src/i18n/tr.ts
M	packages/console/app/src/i18n/uk.ts
M	packages/console/app/src/i18n/zh.ts
M	packages/console/app/src/i18n/zht.ts
M	packages/console/app/src/routes/go/index.tsx
M	packages/console/app/src/routes/workspace/[id]/go/lite-section.tsx
M	packages/console/app/src/routes/zen/util/handler.ts
A	packages/console/core/migrations/20260721182121_wet_pestilence/migration.sql
A	packages/console/core/migrations/20260721182121_wet_pestilence/snapshot.json
M	packages/console/core/package.json
M	packages/console/core/src/schema/workspace.sql.ts
M	packages/console/function/package.json
M	packages/console/mail/package.json
M	packages/console/support/package.json
M	packages/core/package.json
M	packages/core/src/reference.ts
M	packages/core/src/repository-cache.ts
M	packages/core/src/repository.ts
A	packages/core/test/provider-mistral.test.ts
M	packages/core/test/reference.test.ts
M	packages/core/test/repository-cache.test.ts
M	packages/core/test/repository.test.ts
M	packages/desktop/package.json
M	packages/desktop/src/main/server.ts
M	packages/effect-drizzle-sqlite/package.json
M	packages/effect-sqlite-node/package.json
M	packages/enterprise/package.json
M	packages/function/package.json
M	packages/http-recorder/package.json
M	packages/llm/package.json
M	packages/opencode/package.json
M	packages/opencode/src/provider/transform.ts
M	packages/opencode/src/tool/grep.ts
M	packages/opencode/test/provider/transform.test.ts
M	packages/opencode/test/server/httpapi-reference.test.ts
M	packages/opencode/test/session/llm.test.ts
M	packages/opencode/test/tool/grep.test.ts
M	packages/plugin/package.json
M	packages/sdk/js/package.json
M	packages/server/package.json
M	packages/session-ui/package.json
M	packages/session-ui/src/components/basic-tool.tsx
M	packages/session-ui/src/components/message-file.test.ts
M	packages/session-ui/src/components/message-file.ts
M	packages/session-ui/src/components/message-part.tsx
M	packages/session-ui/src/components/session-diff.ts
M	packages/session-ui/src/components/session-review.tsx
M	packages/session-ui/src/components/session-turn.tsx
M	packages/session-ui/src/components/tool-error-card.tsx
M	packages/session-ui/src/context/data.tsx
M	packages/session-ui/src/v2/components/session-review-file-preview-v2.tsx
M	packages/slack/package.json
M	packages/stats/app/package.json
M	packages/stats/core/package.json
M	packages/stats/server/package.json
M	packages/tui/package.json
M	packages/ui/package.json
M	packages/web/package.json
M	packages/web/src/content/docs/ar/go.mdx
M	packages/web/src/content/docs/ar/zen.mdx
M	packages/web/src/content/docs/bs/go.mdx
M	packages/web/src/content/docs/bs/zen.mdx
M	packages/web/src/content/docs/da/go.mdx
M	packages/web/src/content/docs/da/zen.mdx
M	packages/web/src/content/docs/de/go.mdx
M	packages/web/src/content/docs/de/zen.mdx
M	packages/web/src/content/docs/es/go.mdx
M	packages/web/src/content/docs/es/zen.mdx
M	packages/web/src/content/docs/fr/go.mdx
M	packages/web/src/content/docs/fr/zen.mdx
M	packages/web/src/content/docs/go.mdx
M	packages/web/src/content/docs/it/go.mdx
M	packages/web/src/content/docs/it/zen.mdx
M	packages/web/src/content/docs/ja/go.mdx
M	packages/web/src/content/docs/ja/zen.mdx
M	packages/web/src/content/docs/ko/go.mdx
M	packages/web/src/content/docs/ko/zen.mdx
M	packages/web/src/content/docs/nb/go.mdx
M	packages/web/src/content/docs/nb/zen.mdx
M	packages/web/src/content/docs/pl/go.mdx
M	packages/web/src/content/docs/pl/zen.mdx
M	packages/web/src/content/docs/pt-br/go.mdx
M	packages/web/src/content/docs/pt-br/zen.mdx
M	packages/web/src/content/docs/ru/go.mdx
M	packages/web/src/content/docs/ru/zen.mdx
M	packages/web/src/content/docs/th/go.mdx
M	packages/web/src/content/docs/th/zen.mdx
M	packages/web/src/content/docs/tr/go.mdx
M	packages/web/src/content/docs/tr/zen.mdx
M	packages/web/src/content/docs/zen.mdx
M	packages/web/src/content/docs/zh-cn/go.mdx
M	packages/web/src/content/docs/zh-cn/zen.mdx
M	packages/web/src/content/docs/zh-tw/go.mdx
M	packages/web/src/content/docs/zh-tw/zen.mdx
A	patches/@ai-sdk%2Fmistral@3.0.51.patch
M	sdks/vscode/package.json
```
</details>

<details><summary>Kimi Code: 532 name-status records</summary>

```text
M	.agents/skills/agent-core-dev/SKILL.md
M	.agents/skills/agent-core-dev/config.md
M	.agents/skills/agent-core-dev/design.md
M	.agents/skills/agent-core-dev/domain-boundaries.md
M	.agents/skills/agent-core-dev/edge-exposure.md
M	.agents/skills/agent-core-dev/orient.md
M	.agents/skills/agent-core-dev/permission.md
M	.agents/skills/agent-core-dev/server-align.md
M	.agents/skills/gen-changesets/SKILL.md
D	.changeset/acp-thinking-effort-levels.md
D	.changeset/agent-file-subagents.md
D	.changeset/agent-lifecycle-events.md
D	.changeset/catalog-import-broader-and-safer.md
D	.changeset/config-env-overrides.md
D	.changeset/config-env-persist-and-stale-fixes.md
D	.changeset/custom-agent-files.md
A	.changeset/defer-user-tools.md
D	.changeset/fix-abort-not-retryable.md
D	.changeset/fix-openai-prompt-cache-key.md
D	.changeset/fix-vacuous-assistant-wedge.md
A	.changeset/fix-web-clipboard-paste.md
D	.changeset/fix-web-media-alpha-canvas.md
D	.changeset/global-tool-gating.md
D	.changeset/goal-replay-leak.md
A	.changeset/goal-steer-queued-messages.md
D	.changeset/host-fs-content-endpoint.md
D	.changeset/kosong-layered-wire-architecture.md
D	.changeset/long-session-tui-performance.md
D	.changeset/mcp-tool-call-reconnect.md
D	.changeset/model-resolution-inspection.md
D	.changeset/pi-tui-frame-line-reuse.md
D	.changeset/system-md-override.md
D	.changeset/thinking-levels-from-declared-capabilities.md
D	.changeset/tool-pattern-warnings.md
D	.changeset/tools-list-active-flag.md
D	.changeset/tui-code-highlight-no-red.md
D	.changeset/update-third-party-source-note.md
M	AGENTS.md
M	apps/kimi-code/CHANGELOG.md
M	apps/kimi-code/package.json
M	apps/kimi-code/src/cli/prompt-render.ts
M	apps/kimi-code/src/cli/v2/run-v2-print.ts
M	apps/kimi-code/src/tui/constant/tips.ts
M	apps/kimi-code/src/tui/kimi-tui.ts
M	apps/kimi-code/src/tui/utils/image-placeholder.ts
M	apps/kimi-code/test/cli/run-v2-print.test.ts
M	apps/kimi-code/test/cli/v2-run-print.test.ts
M	apps/kimi-code/test/tui/input/image-placeholder.test.ts
M	apps/kimi-code/test/tui/kimi-tui-message-flow.test.ts
M	apps/kimi-code/test/utils/kimi-datasource-plugin.test.ts
M	apps/kimi-inspect/src/App.tsx
A	apps/kimi-inspect/src/activity/store.test.ts
A	apps/kimi-inspect/src/activity/store.ts
A	apps/kimi-inspect/src/activity/useSessionActivity.ts
A	apps/kimi-inspect/src/activity/ws.ts
A	apps/kimi-inspect/src/audit/audit.test.ts
A	apps/kimi-inspect/src/audit/diff.ts
A	apps/kimi-inspect/src/audit/serialize.ts
A	apps/kimi-inspect/src/audit/trail.ts
A	apps/kimi-inspect/src/audit/truncate.ts
M	apps/kimi-inspect/src/channel/client.ts
A	apps/kimi-inspect/src/components/AppServicesView.tsx
M	apps/kimi-inspect/src/components/ChatView.tsx
M	apps/kimi-inspect/src/components/Inspector.tsx
M	apps/kimi-inspect/src/components/NavRail.tsx
A	apps/kimi-inspect/src/components/ServicePanels.tsx
M	apps/kimi-inspect/src/components/Sidebar.tsx
A	apps/kimi-inspect/src/components/audit/AuditPanel.tsx
A	apps/kimi-inspect/src/components/audit/StateTree.test.tsx
A	apps/kimi-inspect/src/components/audit/StateTree.tsx
A	apps/kimi-inspect/src/components/methodArgs.test.ts
A	apps/kimi-inspect/src/components/methodArgs.ts
M	apps/kimi-inspect/src/panels.ts
M	apps/kimi-inspect/src/transcript/api.ts
M	apps/kimi-inspect/src/transcript/store.ts
M	apps/kimi-inspect/src/transcript/transcript.test.ts
M	apps/kimi-inspect/src/transcript/ws.ts
M	apps/kimi-web/src/api/daemon/mappers.ts
M	apps/kimi-web/src/api/daemon/wire.ts
M	apps/kimi-web/src/api/types.ts
M	apps/kimi-web/src/components/FilePreview.vue
M	apps/kimi-web/src/components/chat/Markdown.vue
M	apps/kimi-web/src/composables/useFilePreview.ts
M	apps/kimi-web/src/i18n/locales/en/filePreview.ts
M	apps/kimi-web/src/i18n/locales/zh/filePreview.ts
M	apps/kimi-web/src/lib/clipboard.ts
M	apps/kimi-web/test/clipboard.test.ts
M	apps/kimi-web/test/event-reducer.test.ts
M	apps/kimi-web/test/turn-logic.test.ts
M	apps/vscode/CHANGELOG.md
M	apps/vscode/package.json
M	docs/en/configuration/config-files.md
M	docs/en/configuration/data-locations.md
M	docs/en/configuration/env-vars.md
M	docs/en/configuration/providers.md
M	docs/en/customization/agents.md
M	docs/en/customization/mcp.md
M	docs/en/customization/plugins.md
M	docs/en/guides/use-cases.md
M	docs/en/reference/kimi-command.md
M	docs/en/reference/tools.md
M	docs/en/release-notes/changelog.md
M	docs/zh/configuration/config-files.md
M	docs/zh/configuration/data-locations.md
M	docs/zh/configuration/env-vars.md
M	docs/zh/configuration/providers.md
M	docs/zh/customization/agents.md
M	docs/zh/customization/mcp.md
M	docs/zh/customization/plugins.md
M	docs/zh/guides/use-cases.md
M	docs/zh/reference/kimi-command.md
M	docs/zh/reference/tools.md
M	docs/zh/release-notes/changelog.md
M	packages/acp-adapter/CHANGELOG.md
M	packages/acp-adapter/package.json
M	packages/agent-core-v2/AGENTS.md
M	packages/agent-core-v2/CHANGELOG.md
M	packages/agent-core-v2/docs/Permission.md
A	packages/agent-core-v2/docs/config-manifest.toml
M	packages/agent-core-v2/docs/service-design.md
A	packages/agent-core-v2/docs/wire-manifest.d.ts
M	packages/agent-core-v2/package.json
M	packages/agent-core-v2/scripts/check-domain-layers.mjs
A	packages/agent-core-v2/scripts/gen-config-manifest.mts
A	packages/agent-core-v2/scripts/gen-wire-manifest.mts
A	packages/agent-core-v2/scripts/lib/jsonSchema.mts
A	packages/agent-core-v2/src/_base/utils/typeEquality.ts
M	packages/agent-core-v2/src/agent/contextMemory/messageProjection.ts
M	packages/agent-core-v2/src/agent/contextMemory/protocolMessage.ts
M	packages/agent-core-v2/src/agent/externalHooks/externalHooksService.ts
M	packages/agent-core-v2/src/agent/fullCompaction/fullCompactionService.ts
M	packages/agent-core-v2/src/agent/goal/goalService.ts
M	packages/agent-core-v2/src/agent/llmRequester/llmRequesterService.ts
M	packages/agent-core-v2/src/agent/mcp/client-http.ts
M	packages/agent-core-v2/src/agent/mcp/client-shared.ts
M	packages/agent-core-v2/src/agent/mcp/client-sse.ts
M	packages/agent-core-v2/src/agent/mcp/client-stdio.ts
M	packages/agent-core-v2/src/agent/mcp/config-schema.ts
A	packages/agent-core-v2/src/agent/mcp/configSection.ts
M	packages/agent-core-v2/src/agent/mcp/connection-manager.ts
M	packages/agent-core-v2/src/agent/mcp/mcpService.ts
A	packages/agent-core-v2/src/agent/media/kimiFileUrl.ts
M	packages/agent-core-v2/src/agent/media/mediaToolsRegistrar.ts
M	packages/agent-core-v2/src/agent/media/registerMediaTools.ts
M	packages/agent-core-v2/src/agent/media/tools/read-media.ts
A	packages/agent-core-v2/src/agent/media/videoResolver.ts
A	packages/agent-core-v2/src/agent/media/videoResolverService.ts
A	packages/agent-core-v2/src/agent/media/videoUpload.ts
M	packages/agent-core-v2/src/agent/permissionGate/permissionGate.ts
M	packages/agent-core-v2/src/agent/permissionGate/permissionGateService.ts
M	packages/agent-core-v2/src/agent/permissionPolicy/permissionPolicy.ts
M	packages/agent-core-v2/src/agent/permissionPolicy/permissionPolicyService.ts
D	packages/agent-core-v2/src/agent/permissionPolicy/policies/agent-swarm-exclusive-deny.ts
M	packages/agent-core-v2/src/agent/permissionPolicy/policies/default-tool-approve.ts
D	packages/agent-core-v2/src/agent/permissionPolicy/policies/deny-all.ts
D	packages/agent-core-v2/src/agent/permissionPolicy/policies/goal-start-review-ask.ts
M	packages/agent-core-v2/src/agent/permissionPolicy/policies/path-utils.ts
D	packages/agent-core-v2/src/agent/permissionPolicy/policies/plan-mode-guard-deny.ts
D	packages/agent-core-v2/src/agent/permissionPolicy/policies/plan-mode-tool-approve.ts
D	packages/agent-core-v2/src/agent/permissionPolicy/policies/swarm-mode-agent-swarm-approve.ts
M	packages/agent-core-v2/src/agent/permissionPolicy/types.ts
R067	packages/agent-core-v2/src/agent/permissionPolicy/policies/exit-plan-mode-review-ask.ts	packages/agent-core-v2/src/agent/plan/exitPlanModeReview.ts
M	packages/agent-core-v2/src/agent/plan/plan.ts
M	packages/agent-core-v2/src/agent/plan/planOps.ts
M	packages/agent-core-v2/src/agent/plan/planService.ts
M	packages/agent-core-v2/src/agent/plan/tools/exit-plan-mode.ts
M	packages/agent-core-v2/src/agent/profile/profileService.ts
M	packages/agent-core-v2/src/agent/rpc/core-api.ts
M	packages/agent-core-v2/src/agent/rpc/rpcService.ts
M	packages/agent-core-v2/src/agent/swarm/swarmService.ts
M	packages/agent-core-v2/src/agent/swarm/tools/agent-swarm.ts
M	packages/agent-core-v2/src/agent/task/configSection.ts
A	packages/agent-core-v2/src/agent/task/printDefaults.ts
M	packages/agent-core-v2/src/agent/task/taskOps.ts
M	packages/agent-core-v2/src/agent/task/taskService.ts
A	packages/agent-core-v2/src/agent/toolApproval/toolApproval.ts
A	packages/agent-core-v2/src/agent/toolApproval/toolApprovalService.ts
M	packages/agent-core-v2/src/agent/toolDedupe/toolDedupeService.ts
A	packages/agent-core-v2/src/agent/toolExecutor/beforeToolExecuteEvent.ts
M	packages/agent-core-v2/src/agent/toolExecutor/toolExecutor.ts
M	packages/agent-core-v2/src/agent/toolExecutor/toolExecutorService.ts
M	packages/agent-core-v2/src/agent/toolExecutor/toolHooks.ts
M	packages/agent-core-v2/src/agent/toolRegistry/builtinToolsRegistrar.ts
M	packages/agent-core-v2/src/agent/toolRegistry/toolContribution.ts
M	packages/agent-core-v2/src/agent/toolRegistry/toolRegistry.ts
M	packages/agent-core-v2/src/agent/toolRegistry/toolRegistryService.ts
M	packages/agent-core-v2/src/agent/toolSelect/toolSelect.ts
M	packages/agent-core-v2/src/agent/toolSelect/toolSelectService.ts
M	packages/agent-core-v2/src/agent/toolSelect/tools/select-tools.ts
M	packages/agent-core-v2/src/agent/userTool/userTool.ts
M	packages/agent-core-v2/src/agent/userTool/userToolOps.ts
M	packages/agent-core-v2/src/agent/userTool/userToolService.ts
M	packages/agent-core-v2/src/app/agentFileCatalog/agentFile.ts
M	packages/agent-core-v2/src/app/agentFileCatalog/agentProfileFromFile.ts
M	packages/agent-core-v2/src/app/agentFileCatalog/types.ts
M	packages/agent-core-v2/src/app/agentProfileCatalog/agentProfileCatalog.ts
M	packages/agent-core-v2/src/app/auth/authService.ts
M	packages/agent-core-v2/src/app/auth/configSection.ts
M	packages/agent-core-v2/src/app/authLegacy/authLegacyService.ts
M	packages/agent-core-v2/src/app/bootstrap/bootstrap.ts
M	packages/agent-core-v2/src/app/config/configService.ts
M	packages/agent-core-v2/src/app/config/errors.ts
M	packages/agent-core-v2/src/app/hostFolderBrowser/hostFolderBrowserService.ts
A	packages/agent-core-v2/src/app/kosongConfig/builtInModelsDev.ts
A	packages/agent-core-v2/src/app/kosongConfig/configSection.ts
R079	packages/agent-core-v2/src/kosong/model/discovery.ts	packages/agent-core-v2/src/app/kosongConfig/discovery.ts
R071	packages/agent-core-v2/src/kosong/model/discoveryService.ts	packages/agent-core-v2/src/app/kosongConfig/discoveryService.ts
R090	packages/agent-core-v2/src/kosong/model/envOverlay.ts	packages/agent-core-v2/src/app/kosongConfig/envOverlay.ts
A	packages/agent-core-v2/src/app/kosongConfig/errors.ts
A	packages/agent-core-v2/src/app/kosongConfig/kosongConfig.ts
A	packages/agent-core-v2/src/app/kosongConfig/kosongConfigService.ts
A	packages/agent-core-v2/src/app/kosongConfig/modelsDev.ts
A	packages/agent-core-v2/src/app/kosongConfig/modelsDevImport.ts
A	packages/agent-core-v2/src/app/kosongConfig/modelsDevImportService.ts
A	packages/agent-core-v2/src/app/kosongConfig/modelsDevUpstream.ts
A	packages/agent-core-v2/src/app/kosongConfig/oauthTokenAdapter.ts
A	packages/agent-core-v2/src/app/kosongConfig/secondaryModelOverlay.ts
A	packages/agent-core-v2/src/app/projectLocalConfig/projectLocalConfig.ts
M	packages/agent-core-v2/src/app/sessionExport/sessionExportService.ts
M	packages/agent-core-v2/src/app/sessionIndex/sessionIndex.ts
M	packages/agent-core-v2/src/app/sessionIndex/sessionIndexService.ts
M	packages/agent-core-v2/src/app/sessionLegacy/sessionLegacyService.ts
M	packages/agent-core-v2/src/app/sessionLifecycle/sessionLifecycleService.ts
M	packages/agent-core-v2/src/app/skillCatalog/builtin/mcp-config.md
M	packages/agent-core-v2/src/app/web/webService.ts
R100	packages/agent-core-v2/src/app/workspaceRegistry/errors.ts	packages/agent-core-v2/src/app/workspace/errors.ts
R087	packages/agent-core-v2/src/app/workspaceRegistry/fileWorkspacePersistence.ts	packages/agent-core-v2/src/app/workspace/fileWorkspacePersistence.ts
R057	packages/agent-core-v2/src/app/workspaceRegistry/workspaceRegistry.ts	packages/agent-core-v2/src/app/workspace/workspace.ts
A	packages/agent-core-v2/src/app/workspace/workspaceAlias.ts
R081	packages/agent-core-v2/src/app/workspaceRegistry/workspacePersistence.ts	packages/agent-core-v2/src/app/workspace/workspacePersistence.ts
R066	packages/agent-core-v2/src/app/workspaceRegistry/workspaceRegistryService.ts	packages/agent-core-v2/src/app/workspace/workspaceService.ts
A	packages/agent-core-v2/src/app/workspaceAliases/workspaceAliases.ts
A	packages/agent-core-v2/src/app/workspaceAliases/workspaceAliasesService.ts
D	packages/agent-core-v2/src/app/workspaceLocalConfig/index.ts
D	packages/agent-core-v2/src/app/workspaceLocalConfig/workspaceLocalConfig.ts
D	packages/agent-core-v2/src/app/workspaceRegistry/workspaceQuery.ts
D	packages/agent-core-v2/src/app/workspaceRegistry/workspaceQueryService.ts
A	packages/agent-core-v2/src/app/workspaceSessions/workspaceSessions.ts
A	packages/agent-core-v2/src/app/workspaceSessions/workspaceSessionsService.ts
M	packages/agent-core-v2/src/errors.ts
M	packages/agent-core-v2/src/index.ts
M	packages/agent-core-v2/src/kosong/contract/errors.ts
M	packages/agent-core-v2/src/kosong/model/catalog.ts
M	packages/agent-core-v2/src/kosong/model/catalogService.ts
D	packages/agent-core-v2/src/kosong/model/configSection.ts
D	packages/agent-core-v2/src/kosong/model/discoveryConfigSection.ts
M	packages/agent-core-v2/src/kosong/model/inspection.ts
M	packages/agent-core-v2/src/kosong/model/model.ts
M	packages/agent-core-v2/src/kosong/model/modelAuth.ts
A	packages/agent-core-v2/src/kosong/model/modelOAuth.ts
M	packages/agent-core-v2/src/kosong/model/modelRequesterImpl.ts
M	packages/agent-core-v2/src/kosong/model/modelService.ts
M	packages/agent-core-v2/src/kosong/model/thinking.ts
M	packages/agent-core-v2/src/kosong/protocol/protocolBase.ts
M	packages/agent-core-v2/src/kosong/protocol/protocolTrait.ts
M	packages/agent-core-v2/src/kosong/provider/bases/openai/openai-legacy.ts
A	packages/agent-core-v2/src/kosong/provider/bases/openai/reasoning-key.ts
D	packages/agent-core-v2/src/kosong/provider/configSection.ts
M	packages/agent-core-v2/src/kosong/provider/protocolAdapterRegistry.ts
M	packages/agent-core-v2/src/kosong/provider/provider.ts
M	packages/agent-core-v2/src/kosong/provider/providerDefinition.ts
M	packages/agent-core-v2/src/kosong/provider/providerService.ts
M	packages/agent-core-v2/src/kosong/provider/providers/kimi/kimi.contrib.ts
A	packages/agent-core-v2/src/kosong/recordDiff.ts
M	packages/agent-core-v2/src/os/backends/node-local/tools/bash.ts
R082	packages/agent-core-v2/src/persistence/backends/node-fs/workspaceLocalConfigService.ts	packages/agent-core-v2/src/persistence/backends/node-fs/projectLocalConfigService.ts
M	packages/agent-core-v2/src/session/agentLifecycle/agentLifecycleService.ts
M	packages/agent-core-v2/src/session/btw/btwService.ts
M	packages/agent-core-v2/src/session/cron/tools/cron-create.md
M	packages/agent-core-v2/src/session/cron/tools/cron-create.ts
M	packages/agent-core-v2/src/session/cron/tools/cron-list.md
A	packages/agent-core-v2/src/session/interaction/interactionOps.ts
M	packages/agent-core-v2/src/session/interaction/interactionService.ts
M	packages/agent-core-v2/src/session/mcp/sessionMcp.ts
M	packages/agent-core-v2/src/session/mcp/sessionMcpService.ts
A	packages/agent-core-v2/src/session/sessionActivity/sessionActivity.ts
A	packages/agent-core-v2/src/session/sessionActivity/sessionActivityService.ts
M	packages/agent-core-v2/src/session/subagent/configSection.ts
A	packages/agent-core-v2/src/session/subagent/flag.ts
A	packages/agent-core-v2/src/session/subagent/secondaryModelWarning.ts
A	packages/agent-core-v2/src/session/subagent/secondaryModelWarningService.ts
M	packages/agent-core-v2/src/session/subagent/tools/agent.ts
M	packages/agent-core-v2/src/session/swarm/agentRunBatch.ts
M	packages/agent-core-v2/src/session/swarm/sessionSwarm.ts
M	packages/agent-core-v2/src/session/swarm/sessionSwarmService.ts
M	packages/agent-core-v2/src/session/workspaceCommand/workspaceCommandService.ts
M	packages/agent-core-v2/src/tool/toolContract.ts
M	packages/agent-core-v2/test/agent/fullCompaction/fullCompaction.test.ts
M	packages/agent-core-v2/test/agent/goal/goal.test.ts
M	packages/agent-core-v2/test/agent/goal/goalOps.test.ts
M	packages/agent-core-v2/test/agent/goal/injection/goalInjection.test.ts
A	packages/agent-core-v2/test/agent/goal/stubs.ts
M	packages/agent-core-v2/test/agent/goal/tools/goal-tools.test.ts
M	packages/agent-core-v2/test/agent/llmRequester/llmRequesterService.test.ts
M	packages/agent-core-v2/test/agent/loop/loop.test.ts
M	packages/agent-core-v2/test/agent/loop/stubs.ts
M	packages/agent-core-v2/test/agent/mcp/config-loader.test.ts
M	packages/agent-core-v2/test/agent/mcp/connection-manager.test.ts
A	packages/agent-core-v2/test/agent/mcp/fixtures/slow-tool-stdio-server.mjs
M	packages/agent-core-v2/test/agent/mcp/stubs.ts
M	packages/agent-core-v2/test/agent/media/tools/read-media.test.ts
A	packages/agent-core-v2/test/agent/media/videoResolver.test.ts
M	packages/agent-core-v2/test/agent/permissionGate/permissionGate.test.ts
M	packages/agent-core-v2/test/agent/permissionPolicy/permissionPolicyService.test.ts
M	packages/agent-core-v2/test/agent/permissionPolicy/policies/default-tool-approve.test.ts
D	packages/agent-core-v2/test/agent/permissionPolicy/policies/exit-plan-mode-review-ask.test.ts
D	packages/agent-core-v2/test/agent/permissionPolicy/policies/goal-start-review-ask.test.ts
D	packages/agent-core-v2/test/agent/permissionPolicy/policies/plan-mode-guard-deny.test.ts
M	packages/agent-core-v2/test/agent/permissionPolicy/stubs.ts
M	packages/agent-core-v2/test/agent/plan/plan.test.ts
A	packages/agent-core-v2/test/agent/plan/planGuard.test.ts
M	packages/agent-core-v2/test/agent/plan/planOps.test.ts
M	packages/agent-core-v2/test/agent/plan/tools/exit-plan-mode.test.ts
M	packages/agent-core-v2/test/agent/plan/tools/plan-tools-telemetry.test.ts
M	packages/agent-core-v2/test/agent/profile/thinking.test.ts
M	packages/agent-core-v2/test/agent/swarm/swarm.test.ts
M	packages/agent-core-v2/test/agent/task/taskOps.test.ts
M	packages/agent-core-v2/test/agent/task/taskService.test.ts
A	packages/agent-core-v2/test/agent/toolApproval/toolApproval.test.ts
M	packages/agent-core-v2/test/agent/toolDedupe/toolDedupe.test.ts
A	packages/agent-core-v2/test/agent/toolExecutor/stubs.ts
M	packages/agent-core-v2/test/agent/toolExecutor/toolExecutor.test.ts
M	packages/agent-core-v2/test/agent/toolSelect/toolSelect.e2e.test.ts
M	packages/agent-core-v2/test/agent/toolSelect/toolSelectService.test.ts
M	packages/agent-core-v2/test/agent/userTool/userTool.test.ts
M	packages/agent-core-v2/test/app/agentFileCatalog/agentFile.test.ts
M	packages/agent-core-v2/test/app/auth/auth.test.ts
M	packages/agent-core-v2/test/app/bootstrap/bootstrapService.test.ts
M	packages/agent-core-v2/test/app/config/config.test.ts
A	packages/agent-core-v2/test/app/config/configManifest.test.ts
R081	packages/agent-core-v2/test/kosong/model/discovery.test.ts	packages/agent-core-v2/test/app/kosongConfig/discovery.test.ts
R095	packages/agent-core-v2/test/kosong/model/envOverlay.test.ts	packages/agent-core-v2/test/app/kosongConfig/envOverlay.test.ts
A	packages/agent-core-v2/test/app/kosongConfig/kosongConfigService.test.ts
A	packages/agent-core-v2/test/app/kosongConfig/modelsDevImport.test.ts
A	packages/agent-core-v2/test/app/kosongConfig/secondaryModelOverlay.test.ts
M	packages/agent-core-v2/test/app/messageLegacy/messageLegacy.test.ts
M	packages/agent-core-v2/test/app/model/model.test.ts
M	packages/agent-core-v2/test/app/provider/provider.test.ts
M	packages/agent-core-v2/test/app/provider/stubs.ts
M	packages/agent-core-v2/test/app/sessionExport/sessionExport.test.ts
M	packages/agent-core-v2/test/app/sessionLegacy/sessionLegacy.test.ts
M	packages/agent-core-v2/test/app/sessionLifecycle/sessionLifecycle.test.ts
M	packages/agent-core-v2/test/app/web/web-fetch-service.test.ts
R085	packages/agent-core-v2/test/app/workspaceRegistry/workspaceRegistryService.test.ts	packages/agent-core-v2/test/app/workspace/workspaceService.test.ts
A	packages/agent-core-v2/test/app/workspaceAliases/workspaceAliasesService.test.ts
D	packages/agent-core-v2/test/app/workspaceRegistry/workspaceQueryService.test.ts
A	packages/agent-core-v2/test/app/workspaceSessions/workspaceSessionsService.test.ts
M	packages/agent-core-v2/test/harness/agent.ts
M	packages/agent-core-v2/test/index.test.ts
M	packages/agent-core-v2/test/kosong/model/catalog.test.ts
M	packages/agent-core-v2/test/kosong/model/modelService.test.ts
M	packages/agent-core-v2/test/kosong/provider/composition.test.ts
M	packages/agent-core-v2/test/kosong/provider/kimi.test.ts
M	packages/agent-core-v2/test/kosong/provider/providerService.test.ts
M	packages/agent-core-v2/test/kosong/stubs.ts
M	packages/agent-core-v2/test/os/backends/node-local/tools/bash.test.ts
M	packages/agent-core-v2/test/os/backends/node-local/tools/grep.test.ts
M	packages/agent-core-v2/test/session/agentLifecycle/agentLifecycle.test.ts
M	packages/agent-core-v2/test/session/btw/btw.test.ts
M	packages/agent-core-v2/test/session/interaction/interaction.test.ts
A	packages/agent-core-v2/test/session/sessionActivity/sessionActivityService.test.ts
A	packages/agent-core-v2/test/session/subagent/secondaryModelWarning.test.ts
M	packages/agent-core-v2/test/session/swarm/sessionSwarm.test.ts
M	packages/agent-core-v2/test/session/workspaceCommand/workspaceCommand.test.ts
M	packages/agent-core-v2/test/tool/tool.test.ts
A	packages/agent-core-v2/test/wire/wireManifest.test.ts
M	packages/agent-core/CHANGELOG.md
M	packages/agent-core/package.json
M	packages/agent-core/src/agent/config/index.ts
M	packages/agent-core/src/agent/config/thinking.ts
M	packages/agent-core/src/agent/context/index.ts
M	packages/agent-core/src/agent/cron/manager.ts
M	packages/agent-core/src/agent/index.ts
M	packages/agent-core/src/agent/tool/index.ts
M	packages/agent-core/src/agent/tool/types.ts
M	packages/agent-core/src/agent/turn/index.ts
A	packages/agent-core/src/agent/turn/media-resolve.ts
M	packages/agent-core/src/config/kimi-env-params.ts
M	packages/agent-core/src/config/model.ts
M	packages/agent-core/src/config/schema.ts
M	packages/agent-core/src/config/toml.ts
M	packages/agent-core/src/mcp/client-http.ts
M	packages/agent-core/src/mcp/client-shared.ts
M	packages/agent-core/src/mcp/client-sse.ts
M	packages/agent-core/src/mcp/client-stdio.ts
M	packages/agent-core/src/mcp/connection-manager.ts
M	packages/agent-core/src/rpc/core-api.ts
M	packages/agent-core/src/rpc/core-impl.ts
M	packages/agent-core/src/services/session/sessionService.ts
M	packages/agent-core/src/session/index.ts
M	packages/agent-core/src/session/provider-manager.ts
M	packages/agent-core/src/session/rpc.ts
M	packages/agent-core/src/skill/builtin/mcp-config.md
M	packages/agent-core/src/tools/builtin/file/read-media.ts
M	packages/agent-core/src/tools/builtin/select-tools.ts
M	packages/agent-core/src/tools/cron/cron-create.md
M	packages/agent-core/src/tools/cron/cron-create.ts
M	packages/agent-core/src/tools/cron/cron-list.md
M	packages/agent-core/src/tools/cron/scheduler.ts
M	packages/agent-core/src/tools/cron/types.ts
A	packages/agent-core/src/tools/support/video-delivery.ts
M	packages/agent-core/src/utils/abort.ts
M	packages/agent-core/test/agent/basic.test.ts
M	packages/agent-core/test/agent/compaction/full.test.ts
M	packages/agent-core/test/agent/config-state.test.ts
M	packages/agent-core/test/agent/config/thinking.test.ts
M	packages/agent-core/test/agent/cron/resume.test.ts
M	packages/agent-core/test/agent/tool-select.e2e.test.ts
M	packages/agent-core/test/config/configs.test.ts
M	packages/agent-core/test/config/kimi-env-params.test.ts
M	packages/agent-core/test/config/model-overrides.test.ts
M	packages/agent-core/test/harness/runtime-provider.test.ts
M	packages/agent-core/test/harness/runtime.test.ts
M	packages/agent-core/test/mcp/connection-manager.test.ts
A	packages/agent-core/test/mcp/fixtures/slow-tool-stdio-server.mjs
M	packages/agent-core/test/session/prompt-metadata.test.ts
M	packages/agent-core/test/tools/read-media.test.ts
M	packages/kap-server/CHANGELOG.md
M	packages/kap-server/package.json
M	packages/kap-server/src/protocol/error-codes.ts
M	packages/kap-server/src/protocol/rest-modelCatalog.ts
M	packages/kap-server/src/protocol/ws-control.ts
M	packages/kap-server/src/routes/connections.ts
M	packages/kap-server/src/routes/modelCatalog.ts
M	packages/kap-server/src/routes/prompts.ts
M	packages/kap-server/src/routes/sessions.ts
M	packages/kap-server/src/routes/skills.ts
M	packages/kap-server/src/routes/snapshot.ts
M	packages/kap-server/src/routes/transcript.ts
M	packages/kap-server/src/routes/workspaces.ts
M	packages/kap-server/src/services/legacyStatus/legacyStatus.ts
M	packages/kap-server/src/services/snapshot/snapshotReader.ts
M	packages/kap-server/src/services/transcript/coreBinding.ts
M	packages/kap-server/src/services/transcript/coreEventMap.ts
M	packages/kap-server/src/services/transcript/transcriptService.ts
M	packages/kap-server/src/start.ts
M	packages/kap-server/src/transport/ws/v1/sessionEventBroadcaster.ts
M	packages/kap-server/src/transport/ws/v1/wsConnectionV1.ts
M	packages/kap-server/test/__snapshots__/apiSurface.snapshot.test.ts.snap
M	packages/kap-server/test/modelCatalog.test.ts
A	packages/kap-server/test/modelCatalogCatalog.test.ts
A	packages/kap-server/test/modelCatalogProviderWrite.test.ts
M	packages/kap-server/test/prompts.test.ts
M	packages/kap-server/test/rpc.test.ts
M	packages/kap-server/test/services/transcript.test.ts
M	packages/kap-server/test/sessionEventBroadcaster.test.ts
M	packages/kap-server/test/snapshot.test.ts
M	packages/kap-server/test/snapshotReader.unit.test.ts
M	packages/kap-server/test/transcript.test.ts
M	packages/kap-server/test/wsConnectionV1.test.ts
M	packages/kap-server/tsdown.config.ts
M	packages/klient/CHANGELOG.md
M	packages/klient/README.md
M	packages/klient/examples/basic.ts
M	packages/klient/examples/context-usage.ts
A	packages/klient/examples/kosong-config-stress.ts
M	packages/klient/examples/smoke.ts
M	packages/klient/package.json
M	packages/klient/src/contract/agent/rpc.ts
A	packages/klient/src/contract/agent/services.ts
M	packages/klient/src/contract/global/catalog.ts
M	packages/klient/src/contract/global/events.ts
M	packages/klient/src/contract/global/plugins.ts
M	packages/klient/src/contract/global/providerDiscovery.ts
M	packages/klient/src/contract/global/workspaces.ts
M	packages/klient/src/contract/index.ts
A	packages/klient/src/contract/mcp.ts
M	packages/klient/src/contract/session/lifecycle.ts
M	packages/klient/src/contract/types.ts
M	packages/klient/src/core/channel.ts
M	packages/klient/src/core/facade/agent.ts
M	packages/klient/src/core/facade/global.ts
A	packages/klient/src/core/facade/kosong-types.ts
M	packages/klient/src/core/klient.ts
M	packages/klient/src/core/validation.ts
M	packages/klient/src/index.ts
M	packages/klient/src/transports/ipc/channel.ts
M	packages/klient/src/transports/ipc/host.ts
M	packages/klient/src/transports/memory/dispatcher.ts
M	packages/klient/src/transports/memory/index.ts
M	packages/klient/src/transports/memory/serviceRegistry.ts
M	packages/klient/test/contract-parity.ts
A	packages/klient/test/contract.test.ts
M	packages/klient/test/e2e/invalid-input-matrix.test.ts
M	packages/klient/test/facade.test.ts
M	packages/klient/test/helpers/conformance.ts
M	packages/kosong/CHANGELOG.md
M	packages/kosong/package.json
M	packages/kosong/src/catalog.ts
M	packages/kosong/src/providers/kimi.ts
M	packages/kosong/src/providers/openai-legacy.ts
A	packages/kosong/src/providers/reasoning-key.ts
M	packages/kosong/test/catalog.test.ts
M	packages/kosong/test/kimi.test.ts
M	packages/kosong/test/openai-common-errors.test.ts
M	packages/kosong/test/openai-legacy.test.ts
M	packages/node-sdk/CHANGELOG.md
M	packages/node-sdk/package.json
M	packages/node-sdk/src/rpc.ts
M	packages/pi-tui/CHANGELOG.md
M	packages/pi-tui/package.json
M	packages/protocol/CHANGELOG.md
M	packages/protocol/package.json
M	packages/protocol/src/__tests__/ws-control.test.ts
M	packages/protocol/src/error-codes.ts
M	packages/protocol/src/ws-control.ts
R057	packages/transcript/src/wire/events.ts	packages/transcript/src/contract/events.ts
R053	packages/transcript/src/wire/schema.ts	packages/transcript/src/contract/schema.ts
M	packages/transcript/src/granularity/filterOps.ts
M	packages/transcript/src/granularity/grade.ts
A	packages/transcript/src/history/foldFacts.ts
M	packages/transcript/src/history/groupTurns.ts
M	packages/transcript/src/index.ts
M	packages/transcript/src/model/attachment.ts
M	packages/transcript/src/model/frame.ts
M	packages/transcript/src/model/ids.ts
M	packages/transcript/src/model/interaction.ts
M	packages/transcript/src/model/item.ts
M	packages/transcript/src/model/meta.ts
A	packages/transcript/src/model/prompt.ts
M	packages/transcript/src/model/task.ts
M	packages/transcript/src/model/turn.ts
M	packages/transcript/src/ops/apply.ts
M	packages/transcript/src/ops/operation.ts
M	packages/transcript/src/store/agentTranscript.ts
M	packages/transcript/test/layers.test.ts
M	packages/transcript/test/store.test.ts
M	plugins/marketplace.json
M	plugins/official/kimi-datasource/CHANGELOG.md
M	plugins/official/kimi-datasource/SKILL.md
M	plugins/official/kimi-datasource/bin/kimi-datasource.mjs
M	plugins/official/kimi-datasource/kimi.plugin.json
```
</details>

## Grok raw-path outcome summary

- **Adopt:** 655 raw records with at least one open atomic behavior or support role; mixed-outcome paths stay open conservatively.
- **Already equivalent:** 18 raw records limited to queue-edit, session-owned completion-drain, and core undo/redo evidence.
- **Not applicable:** 3 official xAI npm-distribution records.
- **Temporarily deferred:** 0 Grok raw records.
- **Unclassified:** 0.

A raw-path `adopt` classification means the path contains an open behavior or supports one; it does **not** claim byte identity and does not convert every behavior in a mixed file into an open row. The final evidence column is a coarse thematic navigation aid unless it cites a numeric atomic ID such as `GB-A572-035`, `GB-69F0-016`, or `GB-6E38-018` from the authoritative inventory. Each raw added, modified, or deleted record is listed exactly once without rename inference.

## Complete 676-record Grok raw-tree ledger

| # | Raw status/path | Outcome | State | Evidence |
| ---: | --- | --- | --- | --- |
| 1 | `M` `Cargo.lock` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 2 | `M` `Cargo.toml` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 3 | `M` `SOURCE_REV` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 4 | `M` `crates/codegen/xai-chat-state/src/actor/request_builder.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 5 | `M` `crates/codegen/xai-chat-state/src/actor/state.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 6 | `M` `crates/codegen/xai-chat-state/src/actor/tests.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 7 | `M` `crates/codegen/xai-chat-state/src/commands.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 8 | `M` `crates/codegen/xai-chat-state/src/compaction_utils.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 9 | `M` `crates/codegen/xai-chat-state/src/types.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 10 | `M` `crates/codegen/xai-file-utils/src/queue.rs` | already equivalent | closed | `GB-A572-035`, `GB-A572-039`, `GB-69F0-016` |
| 11 | `M` `crates/codegen/xai-file-utils/src/storage_client.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 12 | `M` `crates/codegen/xai-grok-agent/src/builder.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 13 | `M` `crates/codegen/xai-grok-agent/src/config.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 14 | `M` `crates/codegen/xai-grok-agent/src/prompt/context.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 15 | `M` `crates/codegen/xai-grok-agent/src/prompt/user_message.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 16 | `M` `crates/codegen/xai-grok-config-types/src/lib.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 17 | `A` `crates/codegen/xai-grok-config/src/global_hook_sources.rs` | adopt | open | `GB-6E38-PROTECTED-HOOKS` |
| 18 | `A` `crates/codegen/xai-grok-config/src/global_hook_sources_tests.rs` | adopt | open | `GB-6E38-PROTECTED-HOOKS` |
| 19 | `M` `crates/codegen/xai-grok-config/src/lib.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 20 | `M` `crates/codegen/xai-grok-config/src/managed_cache.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 21 | `M` `crates/codegen/xai-grok-config/src/managed_cache/tests.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 22 | `M` `crates/codegen/xai-grok-config/src/managed_text/format.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 23 | `M` `crates/codegen/xai-grok-config/src/managed_text/mod.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 24 | `M` `crates/codegen/xai-grok-config/src/signed_policy.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 25 | `M` `crates/codegen/xai-grok-hooks/src/discovery.rs` | adopt | open | `GB-6E38-PROTECTED-HOOKS` |
| 26 | `M` `crates/codegen/xai-grok-mcp/src/servers.rs` | adopt | open | `GB-6E38-MCP-TIMEOUTS` |
| 27 | `M` `crates/codegen/xai-grok-models/default_models.json` | adopt | open | `GB-6E38-XAI-MODELS` |
| 28 | `M` `crates/codegen/xai-grok-pager-bin/Cargo.toml` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 29 | `M` `crates/codegen/xai-grok-pager-bin/src/main.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 30 | `M` `crates/codegen/xai-grok-pager-minimal/src/live.rs` | adopt | open | `GB-6E38-MINIMAL-BASH` |
| 31 | `M` `crates/codegen/xai-grok-pager-minimal/src/overlay.rs` | adopt | open | `GB-6E38-MINIMAL-BASH` |
| 32 | `M` `crates/codegen/xai-grok-pager-minimal/src/panel.rs` | adopt | open | `GB-6E38-MINIMAL-BASH` |
| 33 | `M` `crates/codegen/xai-grok-pager-pty-harness/Cargo.toml` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 34 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/content.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 35 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/flows.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 36 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/lib.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 37 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/pty.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 38 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/scenarios/idle_cost.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 39 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/scenarios/large_codeblock.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 40 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/scenarios/mixed_interaction.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 41 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/scenarios/resize_storm.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 42 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/scenarios/scroll_stress.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 43 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/scenarios/streaming_render.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 44 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/scripted.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 45 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/scroll_matrix/session.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 46 | `A` `crates/codegen/xai-grok-pager-pty-harness/tests/env_op_compile.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 47 | `A` `crates/codegen/xai-grok-pager-pty-harness/tests/privacy_banner_e2e.rs` | adopt | open | `GB-6E38-PRIVACY` |
| 48 | `M` `crates/codegen/xai-grok-pager-pty-harness/tests/prompt_history_durable_quit.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 49 | `M` `crates/codegen/xai-grok-pager-pty-harness/tests/scroll_correctness_ptyctl.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 50 | `M` `crates/codegen/xai-grok-pager-render/src/clipboard/mod.rs` | adopt | open | `GB-6E38-CLIPBOARD-CONFIRM` |
| 51 | `M` `crates/codegen/xai-grok-pager-render/src/render/draw.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 52 | `M` `crates/codegen/xai-grok-pager-render/src/terminal/tmux_probe.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 53 | `M` `crates/codegen/xai-grok-pager-render/src/util.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 54 | `M` `crates/codegen/xai-grok-pager/Cargo.toml` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 55 | `M` `crates/codegen/xai-grok-pager/README.md` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 56 | `A` `crates/codegen/xai-grok-pager/docs/tutorial/01-coming-from-another-tool.md` | adopt | open | `GB-6E38-TUTORIAL` |
| 57 | `A` `crates/codegen/xai-grok-pager/docs/tutorial/02-first-prompt.md` | adopt | open | `GB-6E38-TUTORIAL` |
| 58 | `A` `crates/codegen/xai-grok-pager/docs/tutorial/03-attach-and-paste.md` | adopt | open | `GB-6E38-TUTORIAL` |
| 59 | `A` `crates/codegen/xai-grok-pager/docs/tutorial/04-navigation.md` | adopt | open | `GB-6E38-TUTORIAL` |
| 60 | `A` `crates/codegen/xai-grok-pager/docs/tutorial/05-slash-commands.md` | adopt | open | `GB-6E38-TUTORIAL` |
| 61 | `A` `crates/codegen/xai-grok-pager/docs/tutorial/06-worktrees.md` | adopt | open | `GB-6E38-TUTORIAL` |
| 62 | `A` `crates/codegen/xai-grok-pager/docs/tutorial/07-plan-and-permissions.md` | adopt | open | `GB-6E38-TUTORIAL` |
| 63 | `A` `crates/codegen/xai-grok-pager/docs/tutorial/08-make-it-yours.md` | adopt | open | `GB-6E38-TUTORIAL` |
| 64 | `A` `crates/codegen/xai-grok-pager/docs/tutorial/09-where-next.md` | adopt | open | `GB-6E38-TUTORIAL` |
| 65 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/01-getting-started.md` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 66 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/03-keyboard-shortcuts.md` | adopt | open | `GB-6E38-SHORTCUTS` |
| 67 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/04-slash-commands.md` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 68 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/05-configuration.md` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 69 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/06-theming.md` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 70 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/07-mcp-servers.md` | adopt | open | `GB-6E38-MCP-TIMEOUTS` |
| 71 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/08-skills.md` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 72 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/09-plugins.md` | adopt | open | `GB-6E38-MARKETPLACE` |
| 73 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/10-hooks.md` | adopt | open | `GB-6E38-PROTECTED-HOOKS` |
| 74 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/11-custom-models.md` | adopt | open | `GB-6E38-CUSTOM-GATEWAYS` |
| 75 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/14-headless-mode.md` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 76 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/15-agent-mode.md` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 77 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/16-subagents.md` | adopt | open | `GB-6E38-SUBAGENT-MCP` |
| 78 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/17-sessions.md` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 79 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/18-sandbox.md` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 80 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/21-terminal-support.md` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 81 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/22-permissions-and-safety.md` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 82 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/README.md` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 83 | `M` `crates/codegen/xai-grok-pager/npm/grok/bin/grok` | not applicable | closed | `GB-A572-030` |
| 84 | `M` `crates/codegen/xai-grok-pager/npm/grok/bin/postinstall.js` | not applicable | closed | `GB-A572-030` |
| 85 | `M` `crates/codegen/xai-grok-pager/npm/grok/scripts/test-postinstall.js` | not applicable | closed | `GB-A572-030` |
| 86 | `M` `crates/codegen/xai-grok-pager/src/acp/tracker.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 87 | `M` `crates/codegen/xai-grok-pager/src/actions/defaults.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 88 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/interactions.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 89 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/mcp.rs` | adopt | open | `GB-6E38-MCP-TIMEOUTS` |
| 90 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/mod.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 91 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/permissions.rs` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 92 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/session_notification.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 93 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/settings.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 94 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/announcements.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 95 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/mod.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 96 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/permissions.rs` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 97 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/queue_and_adoption.rs` | already equivalent | closed | `GB-A572-035`, `GB-A572-039`, `GB-69F0-016` |
| 98 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/session_events.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 99 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/workflow_ingest.rs` | adopt | open | `GB-6E38-WORKFLOWS` |
| 100 | `M` `crates/codegen/xai-grok-pager/src/app/actions.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 101 | `M` `crates/codegen/xai-grok-pager/src/app/agent.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 102 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/input.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 103 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/interactions.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 104 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/links.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 105 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/mod.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 106 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/modals.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 107 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/notices.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 108 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/paste.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 109 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/plan.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 110 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/prompt.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 111 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/queue.rs` | already equivalent | closed | `GB-A572-035`, `GB-A572-039`, `GB-69F0-016` |
| 112 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/render.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 113 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/rewind.rs` | adopt | open | `GB-6E38-FORK-REWIND` |
| 114 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/session.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 115 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/workflows_overlay.rs` | adopt | open | `GB-6E38-WORKFLOWS` |
| 116 | `M` `crates/codegen/xai-grok-pager/src/app/app_view.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 117 | `M` `crates/codegen/xai-grok-pager/src/app/cli.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 118 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/cta.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 119 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/dashboard.rs` | adopt | open | `GB-6E38-DASHBOARD-TASKS` |
| 120 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/interject.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 121 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/modes.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 122 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/permissions.rs` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 123 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/prompt.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 124 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/queue.rs` | already equivalent | closed | `GB-A572-035`, `GB-A572-039`, `GB-69F0-016` |
| 125 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/router.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 126 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/session/fork.rs` | adopt | open | `GB-6E38-FORK-REWIND` |
| 127 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 128 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/session/load.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 129 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/settings/setters.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 130 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/settings/ui.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 131 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/status.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 132 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/task_result.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 133 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/auth.rs` | adopt | open | `GB-6E38-AUTH-RECOVERY` |
| 134 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/cta_e2e.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 135 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/dashboard.rs` | adopt | open | `GB-6E38-DASHBOARD-TASKS` |
| 136 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/mod.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 137 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/modes.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 138 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/prompt.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 139 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/router.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 140 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/foreign.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 141 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/fork.rs` | adopt | open | `GB-6E38-FORK-REWIND` |
| 142 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/lifecycle.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 143 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/load.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 144 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/modal.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 145 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/settings.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 146 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/status.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 147 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/task_result.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 148 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/turn.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 149 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/voice.rs` | adopt | open | `GB-6E38-VOICE-HELPER` |
| 150 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/transcript.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 151 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/turn.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 152 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/voice.rs` | adopt | open | `GB-6E38-VOICE-HELPER` |
| 153 | `M` `crates/codegen/xai-grok-pager/src/app/effects/helpers.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 154 | `M` `crates/codegen/xai-grok-pager/src/app/effects/mod.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 155 | `M` `crates/codegen/xai-grok-pager/src/app/effects/tests.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 156 | `M` `crates/codegen/xai-grok-pager/src/app/event_loop.rs` | adopt | open | `GB-6E38-LOOP-GUARD` |
| 157 | `M` `crates/codegen/xai-grok-pager/src/app/external_editor.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 158 | `M` `crates/codegen/xai-grok-pager/src/app/leader_cluster/mod.rs` | adopt | open | `GB-6E38-LEADER-STARTUP` |
| 159 | `M` `crates/codegen/xai-grok-pager/src/app/leader_cluster/scenarios.rs` | adopt | open | `GB-6E38-LEADER-STARTUP` |
| 160 | `M` `crates/codegen/xai-grok-pager/src/app/mod.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 161 | `M` `crates/codegen/xai-grok-pager/src/app/modals.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 162 | `M` `crates/codegen/xai-grok-pager/src/app/mouse.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 163 | `M` `crates/codegen/xai-grok-pager/src/app/queue_edit.rs` | already equivalent | closed | `GB-A572-035`, `GB-A572-039`, `GB-69F0-016` |
| 164 | `M` `crates/codegen/xai-grok-pager/src/app/session_startup.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 165 | `A` `crates/codegen/xai-grok-pager/src/app/session_title_resolve.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 166 | `A` `crates/codegen/xai-grok-pager/src/app/session_title_resolve_tests.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 167 | `M` `crates/codegen/xai-grok-pager/src/app/subagent.rs` | adopt | open | `GB-6E38-SUBAGENT-MCP` |
| 168 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/doctor_format.rs` | adopt | open | `GB-6E38-DOCTOR` |
| 169 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/doctor_format_tests.rs` | adopt | open | `GB-6E38-DOCTOR` |
| 170 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/fix.rs` | adopt | open | `GB-6E38-DOCTOR` |
| 171 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/fix_tests.rs` | adopt | open | `GB-6E38-DOCTOR` |
| 172 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/mod.rs` | adopt | open | `GB-6E38-DOCTOR` |
| 173 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/model.rs` | adopt | open | `GB-6E38-DOCTOR` |
| 174 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/probes/mod.rs` | adopt | open | `GB-6E38-DOCTOR` |
| 175 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/view.rs` | adopt | open | `GB-6E38-DOCTOR` |
| 176 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/view_tests.rs` | adopt | open | `GB-6E38-DOCTOR` |
| 177 | `M` `crates/codegen/xai-grok-pager/src/docs.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 178 | `M` `crates/codegen/xai-grok-pager/src/doctor_cmd/human.rs` | adopt | open | `GB-6E38-DOCTOR` |
| 179 | `M` `crates/codegen/xai-grok-pager/src/doctor_cmd/mod.rs` | adopt | open | `GB-6E38-DOCTOR` |
| 180 | `M` `crates/codegen/xai-grok-pager/src/doctor_cmd/tests.rs` | adopt | open | `GB-6E38-DOCTOR` |
| 181 | `M` `crates/codegen/xai-grok-pager/src/headless.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 182 | `M` `crates/codegen/xai-grok-pager/src/input/terminal_support.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 183 | `M` `crates/codegen/xai-grok-pager/src/lib.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 184 | `M` `crates/codegen/xai-grok-pager/src/minimal/api.rs` | adopt | open | `GB-6E38-MINIMAL-BASH` |
| 185 | `M` `crates/codegen/xai-grok-pager/src/plugin_cmd.rs` | adopt | open | `GB-6E38-MARKETPLACE` |
| 186 | `M` `crates/codegen/xai-grok-pager/src/settings/defs.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 187 | `M` `crates/codegen/xai-grok-pager/src/settings/registry.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 188 | `M` `crates/codegen/xai-grok-pager/src/slash/command.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 189 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/doctor.rs` | adopt | open | `GB-6E38-DOCTOR` |
| 190 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/mod.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 191 | `A` `crates/codegen/xai-grok-pager/src/slash/commands/tutorial.rs` | adopt | open | `GB-6E38-TUTORIAL` |
| 192 | `M` `crates/codegen/xai-grok-pager/src/slash/matcher.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 193 | `M` `crates/codegen/xai-grok-pager/src/slash/mod.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 194 | `M` `crates/codegen/xai-grok-pager/src/startup.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 195 | `M` `crates/codegen/xai-grok-pager/src/test_util.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 196 | `M` `crates/codegen/xai-grok-pager/src/tips/ssh_wrap.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 197 | `M` `crates/codegen/xai-grok-pager/src/tracing.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 198 | `A` `crates/codegen/xai-grok-pager/src/tutorial_docs.rs` | adopt | open | `GB-6E38-TUTORIAL` |
| 199 | `M` `crates/codegen/xai-grok-pager/src/views/agent.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 200 | `M` `crates/codegen/xai-grok-pager/src/views/agents_modal.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 201 | `M` `crates/codegen/xai-grok-pager/src/views/announcements.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 202 | `M` `crates/codegen/xai-grok-pager/src/views/dashboard/peek.rs` | adopt | open | `GB-6E38-DASHBOARD-TASKS` |
| 203 | `M` `crates/codegen/xai-grok-pager/src/views/dashboard/render.rs` | adopt | open | `GB-6E38-DASHBOARD-TASKS` |
| 204 | `M` `crates/codegen/xai-grok-pager/src/views/dashboard/row.rs` | adopt | open | `GB-6E38-DASHBOARD-TASKS` |
| 205 | `M` `crates/codegen/xai-grok-pager/src/views/dashboard/state.rs` | adopt | open | `GB-6E38-DASHBOARD-TASKS` |
| 206 | `M` `crates/codegen/xai-grok-pager/src/views/extensions_modal.rs` | adopt | open | `GB-6E38-MARKETPLACE` |
| 207 | `M` `crates/codegen/xai-grok-pager/src/views/mod.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 208 | `M` `crates/codegen/xai-grok-pager/src/views/modal.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 209 | `M` `crates/codegen/xai-grok-pager/src/views/picker.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 210 | `A` `crates/codegen/xai-grok-pager/src/views/privacy_banner.rs` | adopt | open | `GB-6E38-PRIVACY` |
| 211 | `M` `crates/codegen/xai-grok-pager/src/views/prompt_widget/mod.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 212 | `M` `crates/codegen/xai-grok-pager/src/views/question_view.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 213 | `M` `crates/codegen/xai-grok-pager/src/views/queue_pane.rs` | already equivalent | closed | `GB-A572-035`, `GB-A572-039`, `GB-69F0-016` |
| 214 | `M` `crates/codegen/xai-grok-pager/src/views/session_picker.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 215 | `M` `crates/codegen/xai-grok-pager/src/views/settings_modal/state.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 216 | `M` `crates/codegen/xai-grok-pager/src/views/settings_modal/tests.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 217 | `M` `crates/codegen/xai-grok-pager/src/views/shortcuts_help.rs` | adopt | open | `GB-6E38-SHORTCUTS` |
| 218 | `M` `crates/codegen/xai-grok-pager/src/views/slash_dropdown.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 219 | `M` `crates/codegen/xai-grok-pager/src/views/tasks_pane.rs` | adopt | open | `GB-6E38-DASHBOARD-TASKS` |
| 220 | `M` `crates/codegen/xai-grok-pager/src/views/turn_status.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 221 | `A` `crates/codegen/xai-grok-pager/src/views/tutorial.rs` | adopt | open | `GB-6E38-TUTORIAL` |
| 222 | `M` `crates/codegen/xai-grok-pager/src/views/welcome/mod.rs` | adopt | open | `GB-6E38-TUTORIAL` |
| 223 | `M` `crates/codegen/xai-grok-pager/src/views/workflows.rs` | adopt | open | `GB-6E38-WORKFLOWS` |
| 224 | `M` `crates/codegen/xai-grok-pager/src/voice/handle.rs` | adopt | open | `GB-6E38-VOICE-HELPER` |
| 225 | `M` `crates/codegen/xai-grok-pager/src/voice/mod.rs` | adopt | open | `GB-6E38-VOICE-HELPER` |
| 226 | `M` `crates/codegen/xai-grok-pager/tests/doctor_early_dispatch.rs` | adopt | open | `GB-6E38-DOCTOR` |
| 227 | `M` `crates/codegen/xai-grok-pager/tests/leader_pty_e2e/campaign_leader_mode_remote_dismiss_on_model_pick.rs` | adopt | open | `GB-6E38-LEADER-STARTUP` |
| 228 | `M` `crates/codegen/xai-grok-pager/tests/leader_pty_e2e/common.rs` | adopt | open | `GB-6E38-LEADER-STARTUP` |
| 229 | `M` `crates/codegen/xai-grok-pager/tests/leader_pty_e2e/leader_n_clients_shared_session.rs` | adopt | open | `GB-6E38-LEADER-STARTUP` |
| 230 | `M` `crates/codegen/xai-grok-pager/tests/leader_pty_e2e/leader_reattach_cancellation_roundtrips_durable_log.rs` | adopt | open | `GB-6E38-LEADER-STARTUP` |
| 231 | `M` `crates/codegen/xai-grok-pager/tests/leader_pty_e2e/leader_reattach_completion_roundtrips_durable_log.rs` | adopt | open | `GB-6E38-LEADER-STARTUP` |
| 232 | `M` `crates/codegen/xai-grok-pager/tests/leader_pty_e2e/leader_two_clients_shared_session.rs` | adopt | open | `GB-6E38-LEADER-STARTUP` |
| 233 | `M` `crates/codegen/xai-grok-pager/tests/pty_auto_mode.rs` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 234 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/agent_type_mismatch_no_keeps_current_session.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 235 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/agent_type_mismatch_yes_starts_new_session.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 236 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/auto_compact_top_row.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 237 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/background_task_reaped_on_quit.rs` | adopt | open | `GB-6E38-BACKGROUND-SHELL` |
| 238 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/bash_mode_file_completion_shell_like.rs` | adopt | open | `GB-6E38-BACKGROUND-SHELL` |
| 239 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/bash_mode_tab_completion_dropdown.rs` | adopt | open | `GB-6E38-BACKGROUND-SHELL` |
| 240 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/bracketed_ime_paste_skips_clipboard_image_linux.rs` | adopt | open | `GB-6E38-CLIPBOARD-CONFIRM` |
| 241 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/bracketed_ime_paste_skips_clipboard_image_macos.rs` | adopt | open | `GB-6E38-CLIPBOARD-CONFIRM` |
| 242 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/campaign_nudges_default_until_dismissed_by_model_pick.rs` | adopt | open | `GB-6E38-XAI-MODELS` |
| 243 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/campaign_remote_settings_nudge_and_dismiss.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 244 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/common.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 245 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/critical_announcement_session_banner_pty.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 246 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/ctrl_c_cancel_during_stream_recovers_cleanly.rs` | adopt | open | `GB-6E38-ESC-CANCEL` |
| 247 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/doubled_lines_out_of_band_repro.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 248 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/drag_enters_content_from_gap_pty.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 249 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/drag_from_above_prompt_strip_pty.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 250 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/drag_from_chrome_stays_block_pty.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 251 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/drag_over_gap_rows_does_not_freeze_head_pty.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 252 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/drag_select_autoscroll_full_scrollout_copy_pty.rs` | adopt | open | `GB-6E38-CLIPBOARD-CONFIRM` |
| 253 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/esc_cancels_running_turn_from_prompt_preserves_draft.rs` | adopt | open | `GB-6E38-ESC-CANCEL` |
| 254 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/esc_cancels_running_turn_from_scrollback.rs` | adopt | open | `GB-6E38-ESC-CANCEL` |
| 255 | `D` `crates/codegen/xai-grok-pager/tests/pty_e2e/esc_mid_turn_from_prompt_is_swallowed_preserves_draft.rs` | adopt | open | `GB-6E38-ESC-CANCEL` |
| 256 | `D` `crates/codegen/xai-grok-pager/tests/pty_e2e/esc_mid_turn_from_scrollback_is_swallowed.rs` | adopt | open | `GB-6E38-ESC-CANCEL` |
| 257 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/file_path_with_space_emits_full_osc8_hyperlink.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 258 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/folder_trust_cwd_is_home_git_repo_no_prompt.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 259 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/folder_trust_decline_quits_without_grant.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 260 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/folder_trust_feature_off_shows_no_question.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 261 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/folder_trust_home_git_repo_subdir_keys_on_subdir.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 262 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/folder_trust_question_renders_and_accept_persists_grant.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 263 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/forced_wheel_mode_env_scrolls_exact_rows.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 264 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/interjection_reaches_model_ctrl_l_in_vscode_family.rs` | adopt | open | `GB-6E38-XAI-MODELS` |
| 265 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/iterm_readline_editing.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 266 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/managed_policy_gate_refusal_reaches_real_terminal.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 267 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/mid_turn_slash_dropdown_esc_dismisses_not_cancel.rs` | adopt | open | `GB-6E38-ESC-CANCEL` |
| 268 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/middle_click_pastes_primary_linux.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 269 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_commits_thinking_body_to_scrollback.rs` | adopt | open | `GB-6E38-MINIMAL-BASH` |
| 270 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_ctrl_c_arms_and_quits.rs` | adopt | open | `GB-6E38-MINIMAL-BASH` |
| 271 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_ctrl_o_send_now_queued_apple_terminal.rs` | already equivalent | closed | `GB-A572-035`, `GB-A572-039`, `GB-69F0-016` |
| 272 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_double_esc_committed_queued_prompt_single_render.rs` | already equivalent | closed | `GB-A572-035`, `GB-A572-039`, `GB-69F0-016` |
| 273 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_esc_cancels_running_turn.rs` | adopt | open | `GB-6E38-MINIMAL-BASH` |
| 274 | `D` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_esc_mid_turn_is_swallowed.rs` | adopt | open | `GB-6E38-MINIMAL-BASH` |
| 275 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_external_editor_round_trip.rs` | adopt | open | `GB-6E38-MINIMAL-BASH` |
| 276 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_resize_preserves_committed_scrollback.rs` | adopt | open | `GB-6E38-MINIMAL-BASH` |
| 277 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_transcript_opens_in_pager.rs` | adopt | open | `GB-6E38-MINIMAL-BASH` |
| 278 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_transcript_pager_restore_no_artifacts.rs` | adopt | open | `GB-6E38-MINIMAL-BASH` |
| 279 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/mod.rs` | adopt | open | `GB-6E38-MINIMAL-BASH` |
| 280 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/misclassified_wheel_flood_does_not_teleport_viewport.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 281 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/mouse_reporting_toggle_sticky_persists_pty.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 282 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/nested_quote_drag_copy_excludes_bars_pty.rs` | adopt | open | `GB-6E38-CLIPBOARD-CONFIRM` |
| 283 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/prompt_suggestion_ghost_tab_accepts.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 284 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/quote_block_drag_copy_excludes_bars_pty.rs` | adopt | open | `GB-6E38-CLIPBOARD-CONFIRM` |
| 285 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/quote_block_raw_mode_copy_keeps_source_pty.rs` | adopt | open | `GB-6E38-CLIPBOARD-CONFIRM` |
| 286 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/read_tool_header_selection_copies_path_only_pty.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 287 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/reasoning_efforts_menu_renders_and_remaps_on_wire.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 288 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/recap_header_not_in_selection_pty.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 289 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/rename_title_shows_in_prompt_border.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 290 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/reparked_wait_repushes_buried_marker.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 291 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/requirements_version_failure_exits_2_with_guidance.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 292 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/resize_preserves_scroll_position.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 293 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/reverse_agent_type_mismatch_cursor_to_default.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 294 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/scroll.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 295 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/scroll_debug_hud_env_toggles_overlay.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 296 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/scroll_does_not_crash.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 297 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/send_now_tip_after_mid_turn_queue.rs` | already equivalent | closed | `GB-A572-035`, `GB-A572-039`, `GB-69F0-016` |
| 298 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/shift_tab_plan_nudge_from_always_approve_enters_plan.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 299 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/small_screen_tip_survives_slow_turn.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 300 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/spinner_reappears_after_wait_resumes.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 301 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/storage_upload_parks_on_401_and_drains_after_recovery.rs` | adopt | open | `GB-6E38-TOOL-MEDIA` |
| 302 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/stuck_drag_recovers_on_esc_pty.rs` | adopt | open | `GB-6E38-ESC-CANCEL` |
| 303 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/subscription_watch_and_gate_verify_pty.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 304 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/trackpad_flood_does_not_under_travel.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 305 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/undo_tip_resets_each_new_session.rs` | already equivalent | closed | `GB-A572-027C` (`E5`) |
| 306 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/undo_tip_seen_count_never_persisted.rs` | already equivalent | closed | `GB-A572-027C` (`E5`) |
| 307 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/undo_tip_session_cap_blocks_fourth_show.rs` | already equivalent | closed | `GB-A572-027C` (`E5`) |
| 308 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/verb_group_header_drag_copy_pty.rs` | adopt | open | `GB-6E38-CLIPBOARD-CONFIRM` |
| 309 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/wheel_burst_scrolls_viewport_without_frame_amplification.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 310 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/wheel_flood_paints_no_ghost_frames.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 311 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/wheel_overscroll_at_bottom_reengages_follow_mid_stream.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 312 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/wheel_scrolls_viewport_during_streaming_turn.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 313 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/word_select_tip_on_double_click_pty.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 314 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/zero_turn_model_switch_no_modal.rs` | adopt | open | `GB-6E38-XAI-MODELS` |
| 315 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e_clipboard.rs` | adopt | open | `GB-6E38-CLIPBOARD-CONFIRM` |
| 316 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e_queue.rs` | already equivalent | closed | `GB-A572-035`, `GB-A572-039`, `GB-69F0-016` |
| 317 | `M` `crates/codegen/xai-grok-pager/tests/pty_xtversion.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 318 | `M` `crates/codegen/xai-grok-pager/tests/settings_e2e.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 319 | `M` `crates/codegen/xai-grok-plugin-marketplace/Cargo.toml` | adopt | open | `GB-6E38-MARKETPLACE` |
| 320 | `M` `crates/codegen/xai-grok-plugin-marketplace/src/git.rs` | adopt | open | `GB-6E38-MARKETPLACE` |
| 321 | `M` `crates/codegen/xai-grok-sampler/src/actor/state.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 322 | `M` `crates/codegen/xai-grok-sampler/src/client.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 323 | `M` `crates/codegen/xai-grok-sampler/src/config.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 324 | `A` `crates/codegen/xai-grok-sampler/tests/request_query_and_headers.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 325 | `M` `crates/codegen/xai-grok-sampler/tests/test_actor.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 326 | `M` `crates/codegen/xai-grok-sampling-types/Cargo.toml` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 327 | `M` `crates/codegen/xai-grok-sampling-types/src/conversation.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 328 | `M` `crates/codegen/xai-grok-sampling-types/src/error.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 329 | `M` `crates/codegen/xai-grok-sampling-types/src/lib.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 330 | `M` `crates/codegen/xai-grok-sampling-types/src/serde_helpers.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 331 | `A` `crates/codegen/xai-grok-sampling-types/src/tool_overrides.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 332 | `M` `crates/codegen/xai-grok-sampling-types/src/types.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 333 | `M` `crates/codegen/xai-grok-sandbox/src/child_net.rs` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 334 | `M` `crates/codegen/xai-grok-sandbox/src/deny/mod.rs` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 335 | `A` `crates/codegen/xai-grok-sandbox/src/hook_write_deny.rs` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 336 | `A` `crates/codegen/xai-grok-sandbox/src/hook_write_deny_tests.rs` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 337 | `M` `crates/codegen/xai-grok-sandbox/src/lib.rs` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 338 | `M` `crates/codegen/xai-grok-sandbox/src/paths.rs` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 339 | `M` `crates/codegen/xai-grok-sandbox/src/profiles.rs` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 340 | `M` `crates/codegen/xai-grok-sandbox/tests/deny_paths_e2e.rs` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 341 | `M` `crates/codegen/xai-grok-shared/src/ui_config.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 342 | `M` `crates/codegen/xai-grok-shell-base/src/cpu_profile.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 343 | `M` `crates/codegen/xai-grok-shell-base/src/util/mod.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 344 | `M` `crates/codegen/xai-grok-shell-session-support/src/managed_mcp.rs` | adopt | open | `GB-6E38-MCP-TIMEOUTS` |
| 345 | `M` `crates/codegen/xai-grok-shell/CHANGELOG.md` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 346 | `M` `crates/codegen/xai-grok-shell/Cargo.toml` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 347 | `M` `crates/codegen/xai-grok-shell/README.md` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 348 | `M` `crates/codegen/xai-grok-shell/changelogs/0.2.108.json` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 349 | `M` `crates/codegen/xai-grok-shell/changelogs/0.2.108.md` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 350 | `M` `crates/codegen/xai-grok-shell/changelogs/0.2.109.json` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 351 | `M` `crates/codegen/xai-grok-shell/changelogs/0.2.109.md` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 352 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.110.json` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 353 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.110.md` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 354 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.111.json` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 355 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.111.md` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 356 | `M` `crates/codegen/xai-grok-shell/src/agent/activity.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 357 | `M` `crates/codegen/xai-grok-shell/src/agent/app.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 358 | `M` `crates/codegen/xai-grok-shell/src/agent/chat_modes.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 359 | `M` `crates/codegen/xai-grok-shell/src/agent/config.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 360 | `M` `crates/codegen/xai-grok-shell/src/agent/config_model_override_parse.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 361 | `M` `crates/codegen/xai-grok-shell/src/agent/folder_trust.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 362 | `M` `crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs` | adopt | open | `GB-6E38-XAI-MODELS` |
| 363 | `M` `crates/codegen/xai-grok-shell/src/agent/model_providers.rs` | adopt | open | `GB-6E38-CUSTOM-GATEWAYS` |
| 364 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/acp_agent.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 365 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 366 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/folder_trust_prompt.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 367 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/mod.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 368 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/prompt_response_meta_tests.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 369 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/session_lifecycle.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 370 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/subagent_coordinator.rs` | adopt | open | `GB-6E38-SUBAGENT-MCP` |
| 371 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/tests.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 372 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/tests/subagent_spawn_context_tests.rs` | adopt | open | `GB-6E38-SUBAGENT-MCP` |
| 373 | `M` `crates/codegen/xai-grok-shell/src/agent/relay.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 374 | `M` `crates/codegen/xai-grok-shell/src/agent/restore_code.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 375 | `D` `crates/codegen/xai-grok-shell/src/agent/subagent/coordinator_lifecycle.rs` | adopt | open | `GB-6E38-SUBAGENT-MCP` |
| 376 | `D` `crates/codegen/xai-grok-shell/src/agent/subagent/coordinator_query.rs` | adopt | open | `GB-6E38-SUBAGENT-MCP` |
| 377 | `M` `crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs` | adopt | open | `GB-6E38-SUBAGENT-MCP` |
| 378 | `M` `crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs` | adopt | open | `GB-6E38-SUBAGENT-MCP` |
| 379 | `M` `crates/codegen/xai-grok-shell/src/agent/subagent/tests/mod.rs` | adopt | open | `GB-6E38-SUBAGENT-MCP` |
| 380 | `M` `crates/codegen/xai-grok-shell/src/agent/subagent/tests/rest.rs` | adopt | open | `GB-6E38-SUBAGENT-MCP` |
| 381 | `M` `crates/codegen/xai-grok-shell/src/agent/subscription_check.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 382 | `M` `crates/codegen/xai-grok-shell/src/auth/auth_provider.rs` | adopt | open | `GB-6E38-AUTH-RECOVERY` |
| 383 | `M` `crates/codegen/xai-grok-shell/src/auth/auth_provider_tests.rs` | adopt | open | `GB-6E38-AUTH-RECOVERY` |
| 384 | `M` `crates/codegen/xai-grok-shell/src/auth/credential_provider.rs` | adopt | open | `GB-6E38-AUTH-RECOVERY` |
| 385 | `M` `crates/codegen/xai-grok-shell/src/auth/error.rs` | adopt | open | `GB-6E38-AUTH-RECOVERY` |
| 386 | `M` `crates/codegen/xai-grok-shell/src/auth/flow.rs` | adopt | open | `GB-6E38-AUTH-RECOVERY` |
| 387 | `M` `crates/codegen/xai-grok-shell/src/auth/manager.rs` | adopt | open | `GB-6E38-AUTH-RECOVERY` |
| 388 | `M` `crates/codegen/xai-grok-shell/src/auth/manager_tests.rs` | adopt | open | `GB-6E38-AUTH-RECOVERY` |
| 389 | `M` `crates/codegen/xai-grok-shell/src/auth/oidc/protocol.rs` | adopt | open | `GB-6E38-AUTH-RECOVERY` |
| 390 | `M` `crates/codegen/xai-grok-shell/src/auth/refresh/auth_backend_contract_tests.rs` | adopt | open | `GB-6E38-AUTH-RECOVERY` |
| 391 | `M` `crates/codegen/xai-grok-shell/src/auth/refresh/mod.rs` | adopt | open | `GB-6E38-AUTH-RECOVERY` |
| 392 | `M` `crates/codegen/xai-grok-shell/src/auth/refresh/oidc_refresher.rs` | adopt | open | `GB-6E38-AUTH-RECOVERY` |
| 393 | `M` `crates/codegen/xai-grok-shell/src/auth/refresh/oidc_refresher_tests.rs` | adopt | open | `GB-6E38-AUTH-RECOVERY` |
| 394 | `M` `crates/codegen/xai-grok-shell/src/claude_import.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 395 | `M` `crates/codegen/xai-grok-shell/src/config/mod.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 396 | `M` `crates/codegen/xai-grok-shell/src/config/tests.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 397 | `M` `crates/codegen/xai-grok-shell/src/extensions/auth.rs` | adopt | open | `GB-6E38-AUTH-RECOVERY` |
| 398 | `M` `crates/codegen/xai-grok-shell/src/extensions/bundle.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 399 | `M` `crates/codegen/xai-grok-shell/src/extensions/debug.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 400 | `M` `crates/codegen/xai-grok-shell/src/extensions/marketplace.rs` | adopt | open | `GB-6E38-MARKETPLACE` |
| 401 | `M` `crates/codegen/xai-grok-shell/src/extensions/mcp.rs` | adopt | open | `GB-6E38-MCP-TIMEOUTS` |
| 402 | `M` `crates/codegen/xai-grok-shell/src/extensions/session_admin.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 403 | `M` `crates/codegen/xai-grok-shell/src/extensions/session_updates.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 404 | `M` `crates/codegen/xai-grok-shell/src/extensions/skills.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 405 | `M` `crates/codegen/xai-grok-shell/src/extensions/task.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 406 | `M` `crates/codegen/xai-grok-shell/src/inspect/mod.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 407 | `M` `crates/codegen/xai-grok-shell/src/leader/client.rs` | adopt | open | `GB-6E38-LEADER-STARTUP` |
| 408 | `M` `crates/codegen/xai-grok-shell/src/leader/lock.rs` | adopt | open | `GB-6E38-LEADER-STARTUP` |
| 409 | `M` `crates/codegen/xai-grok-shell/src/leader/mod.rs` | adopt | open | `GB-6E38-LEADER-STARTUP` |
| 410 | `M` `crates/codegen/xai-grok-shell/src/leader/protocol.rs` | adopt | open | `GB-6E38-LEADER-STARTUP` |
| 411 | `M` `crates/codegen/xai-grok-shell/src/leader/server.rs` | adopt | open | `GB-6E38-LEADER-STARTUP` |
| 412 | `M` `crates/codegen/xai-grok-shell/src/leader/test_support.rs` | adopt | open | `GB-6E38-LEADER-STARTUP` |
| 413 | `M` `crates/codegen/xai-grok-shell/src/managed_config.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 414 | `M` `crates/codegen/xai-grok-shell/src/remote/client.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 415 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 416 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/goal.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 417 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/goal_support.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 418 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/hook_dispatch.rs` | adopt | open | `GB-6E38-PROTECTED-HOOKS` |
| 419 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/interjection.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 420 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/mcp.rs` | adopt | open | `GB-6E38-MCP-TIMEOUTS` |
| 421 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/model_switch.rs` | adopt | open | `GB-6E38-XAI-MODELS` |
| 422 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/notification_drain.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 423 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/prompt_build.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 424 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/prompt_queue.rs` | already equivalent | closed | `GB-A572-035`, `GB-A572-039`, `GB-69F0-016` |
| 425 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/recap.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 426 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/reminders.rs` | adopt | open | `GB-6E38-REMINDERS` |
| 427 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/run_loop.rs` | adopt | open | `GB-6E38-LOOP-GUARD` |
| 428 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 429 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/session_mode.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 430 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/session_setup.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 431 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/spawn.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 432 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/stop_gate.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 433 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/tasks_cancel.rs` | adopt | open | `GB-6E38-ESC-CANCEL` |
| 434 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/tool_calls.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 435 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/tool_dispatch.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 436 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/turn.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 437 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/turn_end.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 438 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/types.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 439 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/updates.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 440 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/workflow.rs` | adopt | open | `GB-6E38-WORKFLOWS` |
| 441 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/auth_error_no_retry_tests.rs` | adopt | open | `GB-6E38-AUTH-RECOVERY` |
| 442 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/auto_wake_suppression_tests.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 443 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/cancel_running_task_tests.rs` | adopt | open | `GB-6E38-ESC-CANCEL` |
| 444 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/fs_injection_regression_tests.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 445 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/idle_resume_tests.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 446 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/inline_auto_compact_flow_tests.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 447 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/laziness/laziness_integration_tests.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 448 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/memory_config_tests.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 449 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/observability_bridge_mapping_tests.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 450 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/plan_mode_edit_gate_tests.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 451 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/prompt_mode_transition_tests.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 452 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/prompt_queue_actor_tests.rs` | already equivalent | closed | `GB-A572-035`, `GB-A572-039`, `GB-69F0-016` |
| 453 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/recap_display_only_tests.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 454 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/reminder_policy_tests.rs` | adopt | open | `GB-6E38-REMINDERS` |
| 455 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/replay_buffer_send_update_tests.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 456 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/rewind_cross_compaction_tests.rs` | adopt | open | `GB-6E38-FORK-REWIND` |
| 457 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/support.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 458 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/turn_completion_emit_tests.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 459 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/usage_categories_tests.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 460 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/web_search_e2e_tests.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 461 | `M` `crates/codegen/xai-grok-shell/src/session/agent_rebuild.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 462 | `M` `crates/codegen/xai-grok-shell/src/session/commands.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 463 | `M` `crates/codegen/xai-grok-shell/src/session/compaction.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 464 | `M` `crates/codegen/xai-grok-shell/src/session/compaction_config.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 465 | `M` `crates/codegen/xai-grok-shell/src/session/goal_classifier.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 466 | `M` `crates/codegen/xai-grok-shell/src/session/goal_planner.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 467 | `M` `crates/codegen/xai-grok-shell/src/session/goal_strategist.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 468 | `M` `crates/codegen/xai-grok-shell/src/session/goal_summarizer.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 469 | `M` `crates/codegen/xai-grok-shell/src/session/handle.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 470 | `M` `crates/codegen/xai-grok-shell/src/session/helpers/session_compact.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 471 | `M` `crates/codegen/xai-grok-shell/src/session/image_normalize.rs` | adopt | open | `GB-6E38-TOOL-MEDIA` |
| 472 | `M` `crates/codegen/xai-grok-shell/src/session/managed_mcp.rs` | adopt | open | `GB-6E38-MCP-TIMEOUTS` |
| 473 | `M` `crates/codegen/xai-grok-shell/src/session/mcp_restart.rs` | adopt | open | `GB-6E38-MCP-TIMEOUTS` |
| 474 | `M` `crates/codegen/xai-grok-shell/src/session/mod.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 475 | `M` `crates/codegen/xai-grok-shell/src/session/persistence.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 476 | `M` `crates/codegen/xai-grok-shell/src/session/plan_mode.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 477 | `M` `crates/codegen/xai-grok-shell/src/session/prompt_parser.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 478 | `M` `crates/codegen/xai-grok-shell/src/session/slash_commands.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 479 | `M` `crates/codegen/xai-grok-shell/src/session/storage/jsonl/mod.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 480 | `M` `crates/codegen/xai-grok-shell/src/session/storage/jsonl/tests.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 481 | `M` `crates/codegen/xai-grok-shell/src/session/storage/mod.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 482 | `M` `crates/codegen/xai-grok-shell/src/session/storage/relocation/fs.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 483 | `M` `crates/codegen/xai-grok-shell/src/session/storage/relocation/mod.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 484 | `M` `crates/codegen/xai-grok-shell/src/session/storage/relocation/tests.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 485 | `A` `crates/codegen/xai-grok-shell/src/session/storage/relocation/view.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 486 | `M` `crates/codegen/xai-grok-shell/src/session/unified_list/mod.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 487 | `M` `crates/codegen/xai-grok-shell/src/session/user_message.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 488 | `M` `crates/codegen/xai-grok-shell/src/session/wire_tags.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 489 | `M` `crates/codegen/xai-grok-shell/src/session/workflow/host_service.rs` | adopt | open | `GB-6E38-WORKFLOWS` |
| 490 | `M` `crates/codegen/xai-grok-shell/src/session/workflow/manager.rs` | adopt | open | `GB-6E38-WORKFLOWS` |
| 491 | `M` `crates/codegen/xai-grok-shell/src/session/workflow/tracker.rs` | adopt | open | `GB-6E38-WORKFLOWS` |
| 492 | `M` `crates/codegen/xai-grok-shell/src/session/worktree.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 493 | `M` `crates/codegen/xai-grok-shell/src/terminal/adapter.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 494 | `A` `crates/codegen/xai-grok-shell/src/terminal/exit_watcher.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 495 | `M` `crates/codegen/xai-grok-shell/src/terminal/mod.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 496 | `A` `crates/codegen/xai-grok-shell/src/terminal/output_recorder.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 497 | `M` `crates/codegen/xai-grok-shell/src/test_support/lsp_runtime.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 498 | `M` `crates/codegen/xai-grok-shell/src/test_support/mod.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 499 | `M` `crates/codegen/xai-grok-shell/src/tools/config.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 500 | `M` `crates/codegen/xai-grok-shell/src/tools/notification_bridge.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 501 | `M` `crates/codegen/xai-grok-shell/src/tools/tool_context.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 502 | `M` `crates/codegen/xai-grok-shell/src/upload/gcs.rs` | adopt | open | `GB-6E38-TOOL-MEDIA` |
| 503 | `M` `crates/codegen/xai-grok-shell/src/upload/manifest.rs` | adopt | open | `GB-6E38-TOOL-MEDIA` |
| 504 | `M` `crates/codegen/xai-grok-shell/src/upload/trace.rs` | adopt | open | `GB-6E38-TOOL-MEDIA` |
| 505 | `M` `crates/codegen/xai-grok-shell/src/upload/turn.rs` | adopt | open | `GB-6E38-TOOL-MEDIA` |
| 506 | `M` `crates/codegen/xai-grok-shell/src/util/config/load.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 507 | `M` `crates/codegen/xai-grok-shell/src/util/config/mcp.rs` | adopt | open | `GB-6E38-MCP-TIMEOUTS` |
| 508 | `M` `crates/codegen/xai-grok-shell/src/util/config/persist.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 509 | `M` `crates/codegen/xai-grok-shell/src/util/config/resolve/auto_mode.rs` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 510 | `M` `crates/codegen/xai-grok-shell/src/util/config/resolve/toolset.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 511 | `M` `crates/codegen/xai-grok-shell/src/util/config/resolve/version.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 512 | `M` `crates/codegen/xai-grok-shell/src/util/config/settings_writes.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 513 | `M` `crates/codegen/xai-grok-shell/src/util/config/tips.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 514 | `M` `crates/codegen/xai-grok-shell/src/util/grok_auth_credentials.rs` | adopt | open | `GB-6E38-AUTH-RECOVERY` |
| 515 | `M` `crates/codegen/xai-grok-shell/src/util/hooks.rs` | adopt | open | `GB-6E38-PROTECTED-HOOKS` |
| 516 | `M` `crates/codegen/xai-grok-shell/src/util/mod.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 517 | `M` `crates/codegen/xai-grok-shell/tests/common/mod.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 518 | `M` `crates/codegen/xai-grok-shell/tests/team_managed_config.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 519 | `M` `crates/codegen/xai-grok-shell/tests/test_agent_type_invariant.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 520 | `M` `crates/codegen/xai-grok-shell/tests/test_auth_provider_e2e.rs` | adopt | open | `GB-6E38-AUTH-RECOVERY` |
| 521 | `M` `crates/codegen/xai-grok-shell/tests/test_built_binary_e2e.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 522 | `M` `crates/codegen/xai-grok-shell/tests/test_debug_logging.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 523 | `M` `crates/codegen/xai-grok-shell/tests/test_doom_loop_recovery.rs` | adopt | open | `GB-6E38-LOOP-GUARD` |
| 524 | `M` `crates/codegen/xai-grok-shell/tests/test_global_extra_headers_e2e.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 525 | `M` `crates/codegen/xai-grok-shell/tests/test_leader_death_repro.rs` | adopt | open | `GB-6E38-LEADER-STARTUP` |
| 526 | `M` `crates/codegen/xai-grok-shell/tests/test_leader_stdio_integration.rs` | adopt | open | `GB-6E38-LEADER-STARTUP` |
| 527 | `M` `crates/codegen/xai-grok-shell/tests/test_leader_version_skew.rs` | adopt | open | `GB-6E38-LEADER-STARTUP` |
| 528 | `M` `crates/codegen/xai-grok-shell/tests/test_refusal_stop_reason.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 529 | `M` `crates/codegen/xai-grok-shell/tests/test_registry_churn.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 530 | `M` `crates/codegen/xai-grok-shell/tests/test_stop_hook_e2e.rs` | adopt | open | `GB-6E38-PROTECTED-HOOKS` |
| 531 | `M` `crates/codegen/xai-grok-shell/tests/test_subagent_orphan_reconcile.rs` | adopt | open | `GB-6E38-SUBAGENT-MCP` |
| 532 | `M` `crates/codegen/xai-grok-shell/tests/test_summary_reasoning_effort.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 533 | `M` `crates/codegen/xai-grok-shell/tests/test_trusted_local_plugin_refresh_e2e.rs` | adopt | open | `GB-6E38-MARKETPLACE` |
| 534 | `M` `crates/codegen/xai-grok-shell/tests/test_vendor_compat.rs` | adopt | open | `GB-6E38-SESSION-LIFECYCLE` |
| 535 | `M` `crates/codegen/xai-grok-subagent-resolution/Cargo.toml` | adopt | open | `GB-6E38-SUBAGENT-MCP` |
| 536 | `A` `crates/codegen/xai-grok-subagent-resolution/src/definition.rs` | adopt | open | `GB-6E38-SUBAGENT-MCP` |
| 537 | `M` `crates/codegen/xai-grok-subagent-resolution/src/lib.rs` | adopt | open | `GB-6E38-SUBAGENT-MCP` |
| 538 | `M` `crates/codegen/xai-grok-subagent-resolution/src/types.rs` | adopt | open | `GB-6E38-SUBAGENT-MCP` |
| 539 | `M` `crates/codegen/xai-grok-telemetry/src/config.rs` | adopt | open | `GB-6E38-PRIVACY` |
| 540 | `M` `crates/codegen/xai-grok-telemetry/src/events.rs` | adopt | open | `GB-6E38-PRIVACY` |
| 541 | `M` `crates/codegen/xai-grok-telemetry/src/external/schema.rs` | adopt | open | `GB-6E38-PRIVACY` |
| 542 | `M` `crates/codegen/xai-grok-telemetry/src/external/tests.rs` | adopt | open | `GB-6E38-PRIVACY` |
| 543 | `M` `crates/codegen/xai-grok-telemetry/src/otel_layer/mod.rs` | adopt | open | `GB-6E38-PRIVACY` |
| 544 | `M` `crates/codegen/xai-grok-test-support/Cargo.toml` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 545 | `M` `crates/codegen/xai-grok-test-support/README.md` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 546 | `M` `crates/codegen/xai-grok-test-support/src/acp_client.rs` | adopt | open | `GB-6E38-ACP-EVENTS` |
| 547 | `M` `crates/codegen/xai-grok-test-support/src/env.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 548 | `M` `crates/codegen/xai-grok-test-support/src/headless.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 549 | `M` `crates/codegen/xai-grok-test-support/src/leader.rs` | adopt | open | `GB-6E38-LEADER-STARTUP` |
| 550 | `M` `crates/codegen/xai-grok-test-support/src/lib.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 551 | `M` `crates/codegen/xai-grok-test-support/src/mock_server.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 552 | `M` `crates/codegen/xai-grok-test-support/src/process.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 553 | `A` `crates/codegen/xai-grok-test-support/src/sandbox.rs` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 554 | `M` `crates/codegen/xai-grok-tools-api/build.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 555 | `M` `crates/codegen/xai-grok-tools-api/proto/grok-tools.proto` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 556 | `M` `crates/codegen/xai-grok-tools-api/src/config_validation.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 557 | `M` `crates/codegen/xai-grok-tools-api/src/lib.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 558 | `M` `crates/codegen/xai-grok-tools-api/tests/wire_shape.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 559 | `M` `crates/codegen/xai-grok-tools/Cargo.toml` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 560 | `M` `crates/codegen/xai-grok-tools/src/bridge.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 561 | `M` `crates/codegen/xai-grok-tools/src/computer/local/shell_state.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 562 | `M` `crates/codegen/xai-grok-tools/src/computer/local/terminal.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 563 | `M` `crates/codegen/xai-grok-tools/src/computer/types.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 564 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/bash/mod.rs` | adopt | open | `GB-6E38-BACKGROUND-SHELL` |
| 565 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/grep/mod.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 566 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/image_edit/mod.rs` | adopt | open | `GB-6E38-TOOL-MEDIA` |
| 567 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/image_gen/mod.rs` | adopt | open | `GB-6E38-TOOL-MEDIA` |
| 568 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/monitor/tool.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 569 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/monitor/types.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 570 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/read_file/mod.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 571 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/scheduler/actor.rs` | adopt | open | `GB-6E38-SCHEDULER` |
| 572 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/scheduler/create.rs` | adopt | open | `GB-6E38-SCHEDULER` |
| 573 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/scheduler/mod.rs` | adopt | open | `GB-6E38-SCHEDULER` |
| 574 | `A` `crates/codegen/xai-grok-tools/src/implementations/grok_build/scheduler/occurrence_journal.rs` | adopt | open | `GB-6E38-SCHEDULER` |
| 575 | `A` `crates/codegen/xai-grok-tools/src/implementations/grok_build/scheduler/occurrence_journal_tests.rs` | adopt | open | `GB-6E38-SCHEDULER` |
| 576 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/scheduler/types.rs` | adopt | open | `GB-6E38-SCHEDULER` |
| 577 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/search_replace/mod.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 578 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/backend.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 579 | `A` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/backend_tests.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 580 | `A` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/coordinator.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 581 | `A` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/coordinator/query.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 582 | `A` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/coordinator_state.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 583 | `A` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/coordinator_tests.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 584 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 585 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/types.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 586 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task_output/mod.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 587 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/video_gen/mod.rs` | adopt | open | `GB-6E38-TOOL-MEDIA` |
| 588 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build_concise/bash.rs` | adopt | open | `GB-6E38-BACKGROUND-SHELL` |
| 589 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build_hashline/grep.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 590 | `M` `crates/codegen/xai-grok-tools/src/implementations/opencode/bash/mod.rs` | adopt | open | `GB-6E38-BACKGROUND-SHELL` |
| 591 | `M` `crates/codegen/xai-grok-tools/src/implementations/opencode/edit/mod.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 592 | `M` `crates/codegen/xai-grok-tools/src/implementations/search_tool/mod.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 593 | `M` `crates/codegen/xai-grok-tools/src/implementations/task_output/tool.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 594 | `M` `crates/codegen/xai-grok-tools/src/implementations/web_search/client.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 595 | `M` `crates/codegen/xai-grok-tools/src/normalization.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 596 | `M` `crates/codegen/xai-grok-tools/src/notification/handle.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 597 | `M` `crates/codegen/xai-grok-tools/src/notification/handle_tests.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 598 | `M` `crates/codegen/xai-grok-tools/src/notification/mod.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 599 | `M` `crates/codegen/xai-grok-tools/src/notification/types.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 600 | `M` `crates/codegen/xai-grok-tools/src/persistence.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 601 | `M` `crates/codegen/xai-grok-tools/src/registry/proto_convert.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 602 | `M` `crates/codegen/xai-grok-tools/src/registry/types.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 603 | `M` `crates/codegen/xai-grok-tools/src/reminders/task_completion.rs` | already equivalent | closed | `GB-A572-018` (`E2`) |
| 604 | `M` `crates/codegen/xai-grok-tools/src/tool_taxonomy.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 605 | `M` `crates/codegen/xai-grok-tools/src/types/output.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 606 | `M` `crates/codegen/xai-grok-tools/src/types/resources.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 607 | `M` `crates/codegen/xai-grok-tools/src/types/schema.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 608 | `M` `crates/codegen/xai-grok-tools/src/types/template_renderer.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 609 | `M` `crates/codegen/xai-grok-tools/src/types/tool_io.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 610 | `M` `crates/codegen/xai-grok-tools/src/util/mod.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 611 | `A` `crates/codegen/xai-grok-tools/src/util/shell_env_policy.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 612 | `A` `crates/codegen/xai-grok-tools/src/util/shell_env_policy_tests.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 613 | `M` `crates/codegen/xai-grok-tools/tests/cgroup_memory_test.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 614 | `M` `crates/codegen/xai-grok-update/src/auto_update.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 615 | `M` `crates/codegen/xai-grok-update/src/lib.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 616 | `D` `crates/codegen/xai-grok-update/src/minimum_version.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 617 | `A` `crates/codegen/xai-grok-update/src/version_policy.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 618 | `M` `crates/codegen/xai-grok-version/Cargo.toml` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 619 | `M` `crates/codegen/xai-grok-voice/Cargo.toml` | adopt | open | `GB-6E38-VOICE-HELPER` |
| 620 | `M` `crates/codegen/xai-grok-voice/src/audio/capture.rs` | adopt | open | `GB-6E38-VOICE-HELPER` |
| 621 | `M` `crates/codegen/xai-grok-voice/src/audio/capture_linux.rs` | adopt | open | `GB-6E38-VOICE-HELPER` |
| 622 | `A` `crates/codegen/xai-grok-voice/src/audio/capture_subprocess.rs` | adopt | open | `GB-6E38-VOICE-HELPER` |
| 623 | `M` `crates/codegen/xai-grok-voice/src/audio/mod.rs` | adopt | open | `GB-6E38-VOICE-HELPER` |
| 624 | `A` `crates/codegen/xai-grok-voice/src/audio/pipe.rs` | adopt | open | `GB-6E38-VOICE-HELPER` |
| 625 | `A` `crates/codegen/xai-grok-voice/src/audio/protocol.rs` | adopt | open | `GB-6E38-VOICE-HELPER` |
| 626 | `M` `crates/codegen/xai-grok-voice/src/bin/voice_probe.rs` | adopt | open | `GB-6E38-VOICE-HELPER` |
| 627 | `M` `crates/codegen/xai-grok-voice/src/config.rs` | adopt | open | `GB-6E38-VOICE-HELPER` |
| 628 | `M` `crates/codegen/xai-grok-voice/src/lib.rs` | adopt | open | `GB-6E38-VOICE-HELPER` |
| 629 | `D` `crates/codegen/xai-grok-voice/src/pcm.rs` | adopt | open | `GB-6E38-VOICE-HELPER` |
| 630 | `M` `crates/codegen/xai-grok-voice/src/pipeline.rs` | adopt | open | `GB-6E38-VOICE-HELPER` |
| 631 | `M` `crates/codegen/xai-grok-voice/src/probe.rs` | adopt | open | `GB-6E38-VOICE-HELPER` |
| 632 | `M` `crates/codegen/xai-grok-workspace-client/src/lib.rs` | adopt | open | `GB-6E38-WORKSPACE-ERRORS` |
| 633 | `M` `crates/codegen/xai-grok-workspace-types/src/rpc/deploy.rs` | adopt | open | `GB-6E38-WORKSPACE-ERRORS` |
| 634 | `M` `crates/codegen/xai-grok-workspace-types/src/rpc/workspace.rs` | adopt | open | `GB-6E38-WORKSPACE-ERRORS` |
| 635 | `M` `crates/codegen/xai-grok-workspace/src/bin/workspace_server.rs` | adopt | open | `GB-6E38-WORKSPACE-ERRORS` |
| 636 | `M` `crates/codegen/xai-grok-workspace/src/config.rs` | adopt | open | `GB-6E38-CONFIG-COMPAT` |
| 637 | `M` `crates/codegen/xai-grok-workspace/src/diag_server.rs` | adopt | open | `GB-6E38-WORKSPACE-ERRORS` |
| 638 | `M` `crates/codegen/xai-grok-workspace/src/discovery.rs` | adopt | open | `GB-6E38-WORKSPACE-ERRORS` |
| 639 | `M` `crates/codegen/xai-grok-workspace/src/error.rs` | adopt | open | `GB-6E38-WORKSPACE-ERRORS` |
| 640 | `M` `crates/codegen/xai-grok-workspace/src/file_system/attach_file.rs` | adopt | open | `GB-6E38-WORKSPACE-ERRORS` |
| 641 | `M` `crates/codegen/xai-grok-workspace/src/folder_trust.rs` | adopt | open | `GB-6E38-WORKSPACE-ERRORS` |
| 642 | `M` `crates/codegen/xai-grok-workspace/src/handle.rs` | adopt | open | `GB-6E38-WORKSPACE-ERRORS` |
| 643 | `M` `crates/codegen/xai-grok-workspace/src/hub.rs` | adopt | open | `GB-6E38-WORKSPACE-ERRORS` |
| 644 | `M` `crates/codegen/xai-grok-workspace/src/hub_server.rs` | adopt | open | `GB-6E38-WORKSPACE-ERRORS` |
| 645 | `M` `crates/codegen/xai-grok-workspace/src/lib.rs` | adopt | open | `GB-6E38-WORKSPACE-ERRORS` |
| 646 | `M` `crates/codegen/xai-grok-workspace/src/permission/auto_mode.rs` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 647 | `M` `crates/codegen/xai-grok-workspace/src/permission/claude_settings.rs` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 648 | `A` `crates/codegen/xai-grok-workspace/src/permission/gate_preflight.rs` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 649 | `M` `crates/codegen/xai-grok-workspace/src/permission/hub_permission.rs` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 650 | `M` `crates/codegen/xai-grok-workspace/src/permission/manager.rs` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 651 | `M` `crates/codegen/xai-grok-workspace/src/permission/mod.rs` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 652 | `M` `crates/codegen/xai-grok-workspace/src/permission/policy.rs` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 653 | `M` `crates/codegen/xai-grok-workspace/src/permission/resolution.rs` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 654 | `M` `crates/codegen/xai-grok-workspace/src/permission/shell_access.rs` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 655 | `M` `crates/codegen/xai-grok-workspace/src/permission/types.rs` | adopt | open | `GB-6E38-PERMISSIONS-AUTO` |
| 656 | `M` `crates/codegen/xai-grok-workspace/src/session/checkpoint.rs` | adopt | open | `GB-6E38-FORK-REWIND` |
| 657 | `M` `crates/codegen/xai-grok-workspace/src/session/git.rs` | adopt | open | `GB-6E38-WORKSPACE-ERRORS` |
| 658 | `M` `crates/codegen/xai-grok-workspace/src/session/mod.rs` | adopt | open | `GB-6E38-WORKSPACE-ERRORS` |
| 659 | `M` `crates/codegen/xai-grok-workspace/src/session/tool_config.rs` | adopt | open | `GB-6E38-TOOL-OVERRIDES` |
| 660 | `M` `crates/codegen/xai-grok-workspace/src/upload/mod.rs` | adopt | open | `GB-6E38-TOOL-MEDIA` |
| 661 | `M` `crates/codegen/xai-grok-workspace/src/workspace_ops.rs` | adopt | open | `GB-6E38-WORKSPACE-ERRORS` |
| 662 | `M` `crates/codegen/xai-ratatui-textarea/examples/textarea_demo.rs` | already equivalent | closed | `GB-A572-027C` (`E5`) |
| 663 | `M` `crates/codegen/xai-ratatui-textarea/src/textarea.rs` | already equivalent | closed | `GB-A572-027C` (`E5`) |
| 664 | `M` `crates/codegen/xai-tty-utils/src/lib.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 665 | `M` `crates/codegen/xai-workflow/src/engine.rs` | adopt | open | `GB-6E38-WORKFLOWS` |
| 666 | `M` `crates/codegen/xai-workflow/src/journal.rs` | adopt | open | `GB-6E38-WORKFLOWS` |
| 667 | `M` `crates/common/xai-circuit-breaker/src/lib.rs` | adopt | open | `GB-6E38-LOOP-GUARD` |
| 668 | `M` `crates/common/xai-computer-hub-sdk/src/auth.rs` | adopt | open | `GB-6E38-APP-BUILDER` |
| 669 | `M` `crates/common/xai-computer-hub-sdk/src/connection.rs` | adopt | open | `GB-6E38-APP-BUILDER` |
| 670 | `M` `crates/common/xai-computer-hub-sdk/src/connection_borrow.rs` | adopt | open | `GB-6E38-APP-BUILDER` |
| 671 | `M` `crates/common/xai-computer-hub-sdk/src/demux.rs` | adopt | open | `GB-6E38-APP-BUILDER` |
| 672 | `M` `crates/common/xai-computer-hub-sdk/src/harness.rs` | adopt | open | `GB-6E38-APP-BUILDER` |
| 673 | `M` `crates/common/xai-computer-hub-sdk/src/metrics.rs` | adopt | open | `GB-6E38-APP-BUILDER` |
| 674 | `M` `crates/common/xai-computer-hub-sdk/src/oidc_provider.rs` | adopt | open | `GB-6E38-APP-BUILDER` |
| 675 | `M` `crates/common/xai-test-utils/src/git.rs` | adopt | open | `GB-6E38-VERSION-UPDATES` |
| 676 | `M` `crates/common/xai-tool-protocol/src/turn_hook.rs` | adopt | open | `GB-6E38-PROTECTED-HOOKS` |

## Validation and acknowledgement gate

- Focused Rust validation passed on the formatted committed candidate: 46 Kimi provider tests; 22 route-aware HTTP/DNS/proxy tests; 5 synchronous fuzzy tests; 18 history-search tests; 45 file-search tests; 3 Codex `ent26` presentation/raw-preservation tests; and the synthetic model-worker `WouldBlock` test.
- `CARGO_INCREMENTAL=0 cargo check -p xai-grok-pager-bin` passed. The complete fork-script suite passed 73 tests, the aggregate branding/provider/Codex-search/Warp/updater/workspace/workflow/secret contracts passed, `cargo fmt --all -- --check` passed, and `git diff --check` was clean.
- Strict committed-candidate validation passed with 97 feature path sets and 1,424/1,424 baseline-to-candidate downstream paths covered. The 12 immutable source records and their independent reviewed/latest-fetched state cross-checked successfully.
- Live provider calls were not run: they require explicitly entitled credentials and must not expose authenticated payloads. Thread exhaustion was validated through deterministic failure seams rather than exhausting the user’s machine.
- The prior Grok acknowledgement remains bound to reviewed `3af4d5d39897855bdcc74f23e690024a5dc05573`. `6e386420825bd44ae648c63e7c8cba12fcec9401` is **not acknowledgement-eligible** because 91 applicable atomic Grok behavior obligations remain open.
- No `coverage.upstream_acknowledgements` record or two-parent marker may be created for this target. A later refresh must carry every open stable ID forward or cite implementation and tests that close it.

## Provider isolation and legal review

- xAI, OpenAI Codex, Kimi Code, Z.AI Coding Plan, and Custom auth, endpoints, catalogs, usage, retry, hosted tools, compaction, and logout remain explicit. No credential or static-key fallback was introduced.
- No source was copied from Codex, OpenCode, Kimi, or Nucleo. Kimi reasoning compatibility and Codex plan presentation were independently adapted from observed behavior; thread hardening replaces unsafe mechanisms rather than porting implementation.
- No new third-party runtime dependency or license was introduced, so `THIRD-PARTY-NOTICES` and crate-local notices require no change.
- No credentials, authenticated responses, account IDs, token state, or private headers were read or recorded during this audit.
