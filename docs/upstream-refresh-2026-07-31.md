# Upstream refresh parity ledger — 2026-07-31

This ledger records the immutable source pins, behavior audit, compatibility
adaptations, and remaining obligations for the 2026-07-31 refresh. It continues
[`upstream-refresh-2026-07-25.md`](upstream-refresh-2026-07-25.md) and its
authoritative 99-row Grok inventory. Fetched source history is evidence only;
no upstream tree was merged or rebased into Enhanced.

The closure campaign reviewed exactly the frozen pins below and did not admit
later source commits. All 120 machine-tracked Grok obligations are closed with
named regression or raw-path evidence. The Codex, OpenCode, and Kimi provider
proofs are `offline-qualified` through synthetic credentials and hermetic local
endpoints; they are not live-provider qualification.

## Immutable boundary

- Pre-refresh candidate: `ff365bf`, tree
  `bb540cf1749017559994551c7569bc7d24ee4798`.
- Isolated branch/worktree: `refresh/upstreams-20260731` /
  `/home/ruttydm/Projects/worktrees/grok-build-enhanced-refresh-20260731`.
- Fetch timestamp recorded by the pin commit: `2026-07-31T16:23:43Z`.
- No rebase, push, tag, release, PR mutation, Homebrew mutation, or
  credential-bearing live request was performed.

## Source pins and inventories

| Source | Prior reviewed commit / tree | Closure commit / tree | Reviewed range | Changed records |
| --- | --- | --- | ---: | ---: |
| Grok Build | `3af4d5d39897855bdcc74f23e690024a5dc05573` / `e595174931be9bfb490aacf149e2c9cc0ca0ebba` | `dd04f397b1d02f2272b092555669dfba1f01bc85` / `c5c7bdcda32a828efa112883dcd5279ce78714ec` | 9 commits | 1,095 name-status; 1,101 raw paths |
| OpenAI Codex | `51200321eb7b862a29ffceaba8b19db1934a9b38` / `f776ca65baecd8157602572803d41ec92be9d7ab` | `2c005abb0765bfe3ef42a23fe88d5b806184fa83` / `603381757d9498a3a1b893f475e165a212c83ef1` | 346 commits | 1,554 name-status; 1,593 raw paths |
| OpenCode | `0317531906d3f3bb01cf33c16319870cfde9170c` / `e9344f8affc0b7f5f0537cb6c3ac09852d05f53a` | `e4bd9757a3a5dc7461d286000a19e9bd7df57c40` / `562ddf25ad0b20a738b5cfda9d9c930e8d6cbc30` | 161 commits | 373 name-status/raw paths |
| Kimi Code | `b5efba7abcaf4041f81ec520097a61e6546e8c50` / `26080b7faeed49746ea11bec5a92d0fa23a4e189` | `bfa00807c975fdc5b84dda32d47b16b09e8d42c1` / `01f75e4da31157beed6fe0d43d548e5556f1ca5c` | 88 commits | 1,599 name-status; 1,731 raw paths |
| CodexBar | `cc8da27cec92029a6435bfee4a703a719290234e` / `e41036396d949aed3c579a52af74b1b8bab780f6` | `8ef86077e70ac27d45ddddaf49e409824ccdf668` / `7ac89423e35d164324fcd86add0f885c170eb6d2` | 291 commits | 374 name-status/raw paths |

The incremental Grok range from the prior latest pin `6e386420825bd44ae648c63e7c8cba12fcec9401`
contains 6 commits, 691 rename-aware records, and 694 raw paths. Its exhaustive
raw `(mode,type,oid)` evidence digest is
`d1b2c82ba21536e59916851a90b75b3fc542b6b4ac224d493a468c838b50b02a`.
Incremental Codex contains 178 commits, 936 rename-aware records, and 973 raw
paths; incremental OpenCode contains 97 commits and 248 paths.

The following seven sources did not advance: OpenCode Codex auth
`bec2ad69…`, Warp themes `b3850442…`, Kimi CLI `4a550eff…`, Z.AI SDK
`ca5109c0…`, Z.AI coding plugins `0446d0bb…`, GLM-5 `436efa09…`, and the
Z.AI usage-browser reference `54cd1f33…`. They introduce no new behavior,
runtime provider, theme, research claim, or legal obligation in this pass.

## Grok carry-forward inventory

All 99 stable IDs and their exact behavior definitions in
[`upstream-refresh-2026-07-25-grok-behaviors.md`](upstream-refresh-2026-07-25-grok-behaviors.md)
are incorporated into this ledger; none disappeared. Nine formerly-open atomic
rows close in this pass:

| Stable ID | Classification/state | Local evidence |
| --- | --- | --- |
| `GB-A572-014` | adopt / closed | Managed MCP calls use the upstream 75-second bound with a pinned timeout test. |
| `GB-A572-020` | adopt / closed | `grok-agent-sdk` is accepted in the external telemetry client-identifier contract and documented by storage metadata. |
| `GB-A572-027` | adopt / closed | Shortcut help now names the actual undo/redo bindings and keeps platform variants distinct. |
| `GB-A572-034` | adopt / closed | Direct bang commands use the one-hour upstream timeout. |
| `GB-A572-044` | adopt / closed | Workflows default on while config, remote, and environment false remain kill switches. |
| `GB-69F0-004` | adopt / closed | Every string-form external-auth path uses the shared platform shell (`cmd /C` or `sh -c`). |
| `GB-69F0-015` | adopt / closed | Auth-info projects cached profile/team fields through expired xAI credentials without exposing token data. |
| `GB-69F0-019` | adopt / closed | Linux PipeWire capture invokes `pw-record --raw`. |
| `GB-6E38-017` | adopt / closed | xAI web search defaults to `grok-4.5` in both shell and workspace construction. |

The other **73 prior `adopt` rows are closed** against their unchanged July 25
acceptance criteria and the machine-readable evidence companion. The seven
prior `already equivalent` rows remain closed:
`GB-A572-009`, `GB-A572-018`, `GB-A572-027C`, `GB-A572-035`,
`GB-A572-039`, `GB-69F0-002`, and `GB-69F0-016`. The official xAI npm
distribution row `GB-A572-030` remains not applicable under the fork-owned
release rule.

## Grok behaviors introduced after the prior pin

| Stable ID | Observable behavior | Classification/state | Acceptance criterion |
| --- | --- | --- | --- |
| `GB-0731-001` | Let users enable and disable configured MCP servers without deleting their definitions. | adopt / closed | Port state, refresh, and UI contracts. |
| `GB-0731-002` | Copy a rendered plan from the plan approval surface. | adopt / closed | Port clipboard feedback and terminal fallbacks. |
| `GB-0731-003` | Recognize SuperGrok Plus consistently in subscription and capability presentation. | adopt / closed | Reconcile tier mapping and negative provider cases. |
| `GB-0731-004` | Enable server doom-loop recovery by default while retaining explicit kill switches. | adopt / closed | Composite resolver and precedence tests cover default/config/remote/env behavior. |
| `GB-0731-005` | Preserve terminal-gateway output and completion boundaries. | adopt / closed | Port ordered gateway replay and termination tests. |
| `GB-0731-006` | Surface malformed MCP configuration without losing healthy servers. | adopt / closed | Add tolerant parse and partial-success tests. |
| `GB-0731-007` | Dispatch `SessionEnd` hooks in headless lifecycle paths. | adopt / closed | Port once-only success/cancel/error coverage. |
| `GB-0731-008` | Keep paste chips and question input coherent across editing and submission. | adopt / closed | Port UI state and PTY coverage. |
| `GB-0731-009` | Render duration and log-output status consistently. | adopt / closed | Reconcile task/log display and snapshots. |
| `GB-0731-010` | Refuse to regress local message counts from a stale remote session record. | adopt / closed | Merge takes the maximum count; regression test pins stale-remote behavior. |
| `GB-0731-011` | Explain loop-stop outcomes without misleading prompts. | adopt / closed | Reconcile true-noop/stationarity turn-end text. |
| `GB-0731-012` | Suppress duplicate or inapplicable startup warnings. | adopt / closed | Port warning identity/lifecycle tests. |
| `GB-0731-013` | Preserve positional shell arguments, including `$@`, in persistent/static shell wrappers. | adopt / closed | Wrappers restore arguments before eval; shell tests cover both paths. |
| `GB-0731-014` | Show only genuinely backgrounded outstanding tasks in the background tray. | adopt / closed | Task snapshots carry `is_backgrounded`; workspace filters use the shared predicate. |
| `GB-0731-015` | Clean up child work when the owning parent process dies. | adopt / closed | Port parent-death teardown and cross-session negatives. |
| `GB-0731-016` | Keep plan/reasoning chrome correct in minimal mode. | adopt / closed | Port minimal-mode snapshots and PTYs. |
| `GB-0731-017` | Serialize auth-store mutation across processes. | adopt / closed | Path-scoped writer locks and cross-process refresh/logout race tests pass. |
| `GB-0731-018` | Preserve compatible behavior in legacy Alacritty terminals. | adopt / closed | Port capability detection and PTYs. |
| `GB-0731-019` | Avoid showing a paywall before subscription state is authoritative. | adopt / closed | Reconcile cold-start verification and stale-state tests. |
| `GB-0731-020` | Keep the UI responsive during cold initialization. | adopt / closed | Port staged initialization and delayed-service PTYs. |
| `GB-0731-021` | Bound memory while forking large session histories. | adopt / closed | Port streaming/bounded copy and large-history tests. |
| `GB-0731-022` | Bound worker creation under resource pressure. | adopt / closed | Reconcile remaining worker pools and EAGAIN tests. |
| `GB-0731-023` | Provide `/delete` for the intended session/history surface. | adopt / closed | Port confirmation, persistence, and current-session guards. |
| `GB-0731-024` | Degrade startup safely when the OS refuses new threads. | already equivalent / closed | The prior fallible-worker repair covers model, DNS, proxy, file/history, and required-startup paths. |
| `GB-0731-025` | Preserve Responses tool-result integrity when duplicate results arrive. | adopt / closed | Port normalization and malformed-history tests. |
| `GB-0731-026` | Avoid preview authentication cookie redirect loops. | adopt / closed | Port cookie lifecycle and redirect bounds. |
| `GB-0731-027` | Emit the intended stationarity nudge before terminal loop handling. | adopt / closed | Port nudge sequencing and telemetry tests. |
| `GB-0731-028` | Run external auth commands through the platform shell. | adopt / closed | Shared helper now covers interactive, refresh, identity, and named-provider execution. |
| `GB-0731-029` | Persist and render cancellation markers coherently. | adopt / closed | Port replay/resume/scrollback coverage. |
| `GB-0731-030` | Discover and manage LSP servers, including Roslyn behavior. | adopt / closed | Reconcile server config, lifecycle, and diagnostics. |
| `GB-0731-031` | Keep prompt cache keys stable across equivalent turns. | adopt / closed | Add end-to-end normal/401/history cache-key proofs. |
| `GB-0731-032` | Preserve full streaming-JSON output and boundaries. | adopt / closed | Port stream parser and headless contracts. |
| `GB-0731-033` | Expose `/undo` as the supported rewind command. | adopt / closed | Alias, user guide, and command tests retain `/rewind` compatibility. |
| `GB-0731-034` | Advertise slash commands appropriate to the active session mode. | adopt / closed | Reconcile mode transitions and command updates. |
| `GB-0731-035` | Refresh or relogin correctly after machine sleep. | adopt / closed | Sleep gate, dark-wake budget, in-flight drain, and provider-isolation tests pass. |
| `GB-0731-036` | Warn before draft/history operations that would discard work. | adopt / closed | Port confirmation and cancellation paths. |
| `GB-0731-037` | Commit settings enum changes consistently. | adopt / closed | Reconcile settings persistence and rollback. |
| `GB-0731-038` | Close settings correctly after deep-link navigation. | adopt / closed | Port navigation/state tests. |
| `GB-0731-039` | Load supported hooks declared in TOML. | adopt / closed | Reconcile trusted scopes and protected-source rules. |
| `GB-0731-040` | Export the supported session artifact to GitHub. | adopt / closed | Port local export generation; external publication still requires explicit authorization. |
| `GB-0731-041` | Project coding-data lock state accurately. | adopt / closed | Reconcile settings/auth projection without changing telemetry policy. |
| `GB-0731-042` | Include terminal-version metadata in ordinary telemetry. | adopt / closed | Port bounded detection and schema tests. |
| `GB-0731-043` | Keep the exit-plan approval barrier ordered with the active turn. | adopt / closed | Mixed write/exit order and permission-cancel race tests pass. |
| `GB-0731-044` | Honor configured extra certificate authorities. | adopt / closed | Existing provider HTTP leaf validates a capped PEM bundle and projects roots into reqwest 0.12/0.13 clients without disabling normal verification. |
| `GB-0731-045` | Discover and refresh remote managed skills. | adopt / closed | Port ownership, refresh, and trust-boundary tests. |

Atomic Grok summary: **0 open adopt**, **135 closed adopt**, **8 closed
already-equivalent**, **1 closed not-applicable**, **0 temporary deferrals**,
and **0 unclassified**, across 144 rows. The checked-in machine companion
tracks the campaign's exhaustive 120-ID adoption queue and records all 120 as
closed.

## Provider-reference behavior inventory

### OpenAI Codex

| Evidence ID | Classification/state | Local result or remaining gate |
| --- | --- | --- |
| `CDX-4C43-ROUTED-AUTH` / `CDX-PROXY-001` | already equivalent / offline-qualified | Hermetic refresh, revoke, Responses, and provider-route tests prove isolation and redaction. |
| `CDX-4C43-ENT26` | adopt / closed | Existing lossless raw plan plus Enterprise presentation remains covered. |
| `CDX-4C43-CATALOG` / `CDX-CATALOG-001` | already equivalent / offline-qualified | Auth changes replace the authoritative catalog without changing xAI state. |
| `CDX-4C43-ITEM-IDS` / `CDX-HISTORY-001` | already equivalent / offline-qualified | JSONL persistence, fork, compaction, and prompt-cache tests prove full-history continuity without `previous_response_id`. |
| `CDX-4C43-RETRY` / `CDX-RETRY-001` | adopt / offline-qualified | The request actor proves exactly-once 401 recovery, bounded 429 delay, stable cache identity, and redacted diagnostics. |
| `CDX-2C00-BUSINESS-PLANS` | adopt / closed | New business plan codes remain raw and receive friendly display labels. |
| `CDX-2C00-CATALOG-RENEW` | adopt / closed | Same-generation cache renewal is throttled while newer credential generations rebind. |
| `CDX-2C00-IMAGE-TURN` | adopt / closed offline | Generation/edit requests share one bounded sensitive Codex-only turn header across 401 replay. |
| `CDX-2C00-FREE-IMAGE-GATE` | adopt / closed | Exact `free` plan omits both image tools at spawn and provider/model switch; unknown plans fail open. |
| `CDX-2C00-COLLAB-MESSAGES` | adopt / closed | Authenticated catalog default/plan messages are bounded, identity-scoped, mode/model deduplicated, and cleared on provider changes; Grok plan safety remains authoritative. |
| `CDX-2C00-TOKEN-BUDGET` | not applicable / closed | Model-owned reminder/world-state architecture would replace preserved Grok compaction behavior. |
| `CDX-2C00-FRACTIONAL-HISTORY` | not applicable / closed | Enhanced already preserves usage percentages as `f64` and has no Codex SQLite thread store. |
| `CDX-2C00-INTERRUPT-ANALYTICS` | not applicable / closed | Codex-app analytics are outside the provider-adapter scope. |
| `CDX-2C00-NO-ADAPTER-DELTA` | not applicable / closed | Remaining app-server/TUI/MCP/code-mode/sandbox/release runtime is outside adapter scope. |

Previously closed Codex rows (`WS-RECOVERY`, `SERIALIZATION`, `CUSTOM-SEARCH`,
`CODE-MODE`, `MCP-REVISION`, catalog-matrix fields, and hosted-tool architecture)
remain unchanged. `CDX-HOSTED-IMAGE-001` is offline-qualified by exact-route,
sensitive-header, 401-binding, and xAI-negative tests; no live claim is made.

### OpenCode and OpenCode Codex auth

| Evidence ID | Classification/state | Result |
| --- | --- | --- |
| `OC-7534-CACHE-KEY` | already equivalent / closed | Existing cache-key compatibility remains. |
| `OC-7534-AUTH-REFETCH` / `OC-AUTH-001` | already equivalent / offline-qualified | Auth-change catalog replacement is provider- and credential-generation-scoped. |
| `OC-E4BD-INTERLEAVED` | adopt / offline-qualified | Bounded bare-string and valid provider-defined reasoning fields are accepted; invalid siblings are discarded and replay remains provider/model/generation scoped. |
| `OC-E4BD-NO-OTHER-CODEX-DELTA` | not applicable / closed | The remaining new changes affect other providers and OpenCode runtime, not the Codex adapter contract. |
| `OCAUTH-BEC2-UNCHANGED` | already equivalent / closed | Source did not advance. |

### Kimi Code and Z.AI research

| Evidence ID | Classification/state | Result |
| --- | --- | --- |
| `KIMI-BFA-QUOTA` | adopt / closed | Structured quota/balance/recharge 429s are fatal, redacted, and never retried. |
| `KIMI-USAGE-RESET` | already equivalent / closed | Exact minute/reset projection has a local regression test. |
| `KIMI-CATALOG-FALLBACK` | not applicable / closed | Generic static fallback would violate credential-bound entitlement discovery; cache replacement remains provider-bound. |
| `KIMI-OAUTH` | not applicable / closed | Enhanced intentionally remains an experimental API-key-only Kimi provider. |
| `KIMI-VIDEO-001` | adopt / offline-qualified | Typed local video content round-trips without remote IDs; Kimi uploads bounded supported files at request time and caches IDs only for the credential generation; other providers reject before network transmission. |
| `CODEXBAR-ZAI-PERCENT` | not applicable / closed | Percentage UI behavior is retained as research evidence only and does not establish a Z.AI runtime provider or product claim. |

No runtime Z.AI provider, login flow, credential, or product claim is inferred
from the unchanged research sources or CodexBar UI behavior.

## Validation

Validation completed against the formatted closure candidate:

- `xai-grok-shell`: 6,102 passed, 8 ignored; `xai-grok-tools`: 2,949 passed,
  7 ignored; `xai-grok-sampler`: 316 passed; `xai-grok-sampling-types`: 318
  passed.
- The fork-script governance suite passed 76/76, release and installer suites
  passed 5/5 and 15/15, npm lifecycle tests passed 35/35, and the Warp
  vendor-lock target passed 2/2.
- `cargo check --locked -p xai-grok-pager-bin`, aggregate fork contracts,
  formatting, raw-diff hygiene, strict ownership coverage, and acknowledgement
  preparation passed.
- DotSlash/protoc remains a local preflight and the existing CI installation
  path remains unchanged; no binary was vendored and no check was weakened.
- Live provider calls were not run. All provider qualification used synthetic
  credentials and local endpoints, and is recorded only as offline-qualified.

## Complete 1101 raw-path ledger

The acknowledgement target has parent
`500129c714ad1b10e6095481f4a8387a2ec52649`. The rows below are the
exhaustive recursive tree delta from reviewed tree
`e595174931be9bfb490aacf149e2c9cc0ca0ebba` to target tree
`c5c7bdcda32a828efa112883dcd5279ce78714ec`. `GB-0731-RAW` binds adopted
paths to the closed behavior inventory and its named regression evidence;
`GB-A572-030` retains the standing fork-owned-release exception for official
xAI npm packaging. The authenticated source ID is `grok-build-upstream`.

| Row | Raw path | Outcome | Evidence |
| ---: | --- | --- | --- |
| 1 | `M` `Cargo.lock` | adopt | `GB-0731-RAW` |
| 2 | `M` `Cargo.toml` | adopt | `GB-0731-RAW` |
| 3 | `M` `SOURCE_REV` | adopt | `GB-0731-RAW` |
| 4 | `M` `clippy.toml` | adopt | `GB-0731-RAW` |
| 5 | `M` `crates/build/xai-proto-build/src/lib.rs` | adopt | `GB-0731-RAW` |
| 6 | `M` `crates/codegen/xai-chat-state/src/actor/request_builder.rs` | adopt | `GB-0731-RAW` |
| 7 | `M` `crates/codegen/xai-chat-state/src/actor/state.rs` | adopt | `GB-0731-RAW` |
| 8 | `M` `crates/codegen/xai-chat-state/src/actor/tests.rs` | adopt | `GB-0731-RAW` |
| 9 | `M` `crates/codegen/xai-chat-state/src/commands.rs` | adopt | `GB-0731-RAW` |
| 10 | `M` `crates/codegen/xai-chat-state/src/compaction_utils.rs` | adopt | `GB-0731-RAW` |
| 11 | `M` `crates/codegen/xai-chat-state/src/types.rs` | adopt | `GB-0731-RAW` |
| 12 | `M` `crates/codegen/xai-chat-state/src/usage.rs` | adopt | `GB-0731-RAW` |
| 13 | `M` `crates/codegen/xai-crash-handler/src/handler.rs` | adopt | `GB-0731-RAW` |
| 14 | `M` `crates/codegen/xai-crash-handler/src/lib.rs` | adopt | `GB-0731-RAW` |
| 15 | `M` `crates/codegen/xai-crash-handler/src/symbolicate.rs` | adopt | `GB-0731-RAW` |
| 16 | `M` `crates/codegen/xai-crash-handler/tests/integration.rs` | adopt | `GB-0731-RAW` |
| 17 | `M` `crates/codegen/xai-fast-worktree/src/api.rs` | adopt | `GB-0731-RAW` |
| 18 | `M` `crates/codegen/xai-fast-worktree/src/auto_gc.rs` | adopt | `GB-0731-RAW` |
| 19 | `M` `crates/codegen/xai-fast-worktree/src/git/checkout.rs` | adopt | `GB-0731-RAW` |
| 20 | `M` `crates/codegen/xai-fast-worktree/src/git/mod.rs` | adopt | `GB-0731-RAW` |
| 21 | `M` `crates/codegen/xai-fast-worktree/src/git/worktree.rs` | adopt | `GB-0731-RAW` |
| 22 | `M` `crates/codegen/xai-fast-worktree/src/lib.rs` | adopt | `GB-0731-RAW` |
| 23 | `M` `crates/codegen/xai-fast-worktree/src/sync.rs` | adopt | `GB-0731-RAW` |
| 24 | `M` `crates/codegen/xai-file-utils/src/events/log.rs` | adopt | `GB-0731-RAW` |
| 25 | `M` `crates/codegen/xai-file-utils/src/events/mod.rs` | adopt | `GB-0731-RAW` |
| 26 | `M` `crates/codegen/xai-file-utils/src/events/tracker.rs` | adopt | `GB-0731-RAW` |
| 27 | `M` `crates/codegen/xai-file-utils/src/events/types.rs` | adopt | `GB-0731-RAW` |
| 28 | `M` `crates/codegen/xai-file-utils/src/queue.rs` | adopt | `GB-0731-RAW` |
| 29 | `M` `crates/codegen/xai-file-utils/src/storage_client.rs` | adopt | `GB-0731-RAW` |
| 30 | `M` `crates/codegen/xai-fsnotify/src/watcher.rs` | adopt | `GB-0731-RAW` |
| 31 | `M` `crates/codegen/xai-grok-agent/src/builder.rs` | adopt | `GB-0731-RAW` |
| 32 | `M` `crates/codegen/xai-grok-agent/src/config.rs` | adopt | `GB-0731-RAW` |
| 33 | `M` `crates/codegen/xai-grok-agent/src/discovery.rs` | adopt | `GB-0731-RAW` |
| 34 | `M` `crates/codegen/xai-grok-agent/src/error.rs` | adopt | `GB-0731-RAW` |
| 35 | `M` `crates/codegen/xai-grok-agent/src/plugins/hooks_adapter.rs` | adopt | `GB-0731-RAW` |
| 36 | `M` `crates/codegen/xai-grok-agent/src/prompt/context.rs` | adopt | `GB-0731-RAW` |
| 37 | `M` `crates/codegen/xai-grok-agent/src/prompt/user_message.rs` | adopt | `GB-0731-RAW` |
| 38 | `M` `crates/codegen/xai-grok-auth/src/lib.rs` | adopt | `GB-0731-RAW` |
| 39 | `M` `crates/codegen/xai-grok-auth/src/retry_middleware.rs` | adopt | `GB-0731-RAW` |
| 40 | `M` `crates/codegen/xai-grok-config-types/src/lib.rs` | adopt | `GB-0731-RAW` |
| 41 | `M` `crates/codegen/xai-grok-config-types/src/mcp.rs` | adopt | `GB-0731-RAW` |
| 42 | `A` `crates/codegen/xai-grok-config/src/global_hook_sources.rs` | adopt | `GB-0731-RAW` |
| 43 | `A` `crates/codegen/xai-grok-config/src/global_hook_sources_tests.rs` | adopt | `GB-0731-RAW` |
| 44 | `M` `crates/codegen/xai-grok-config/src/lib.rs` | adopt | `GB-0731-RAW` |
| 45 | `M` `crates/codegen/xai-grok-config/src/loader.rs` | adopt | `GB-0731-RAW` |
| 46 | `M` `crates/codegen/xai-grok-config/src/managed_cache.rs` | adopt | `GB-0731-RAW` |
| 47 | `M` `crates/codegen/xai-grok-config/src/managed_cache/claim_tests.rs` | adopt | `GB-0731-RAW` |
| 48 | `M` `crates/codegen/xai-grok-config/src/managed_cache/tests.rs` | adopt | `GB-0731-RAW` |
| 49 | `M` `crates/codegen/xai-grok-config/src/managed_text/format.rs` | adopt | `GB-0731-RAW` |
| 50 | `M` `crates/codegen/xai-grok-config/src/managed_text/mod.rs` | adopt | `GB-0731-RAW` |
| 51 | `M` `crates/codegen/xai-grok-config/src/managed_text/validator.rs` | adopt | `GB-0731-RAW` |
| 52 | `M` `crates/codegen/xai-grok-config/src/signed_policy.rs` | adopt | `GB-0731-RAW` |
| 53 | `M` `crates/codegen/xai-grok-config/src/signed_policy/tests.rs` | adopt | `GB-0731-RAW` |
| 54 | `A` `crates/codegen/xai-grok-extra-ca/Cargo.toml` | adopt | `GB-0731-RAW` |
| 55 | `A` `crates/codegen/xai-grok-extra-ca/src/lib.rs` | adopt | `GB-0731-RAW` |
| 56 | `A` `crates/codegen/xai-grok-extra-ca/src/lib_tests.rs` | adopt | `GB-0731-RAW` |
| 57 | `A` `crates/codegen/xai-grok-extra-ca/tests/extra_ca_invalid_file.rs` | adopt | `GB-0731-RAW` |
| 58 | `A` `crates/codegen/xai-grok-extra-ca/tests/extra_ca_oversized.rs` | adopt | `GB-0731-RAW` |
| 59 | `A` `crates/codegen/xai-grok-extra-ca/tests/extra_ca_valid_env.rs` | adopt | `GB-0731-RAW` |
| 60 | `A` `crates/codegen/xai-grok-extra-ca/tests/extra_ca_zero_certs.rs` | adopt | `GB-0731-RAW` |
| 61 | `M` `crates/codegen/xai-grok-hooks/Cargo.toml` | adopt | `GB-0731-RAW` |
| 62 | `M` `crates/codegen/xai-grok-hooks/src/config.rs` | adopt | `GB-0731-RAW` |
| 63 | `M` `crates/codegen/xai-grok-hooks/src/discovery.rs` | adopt | `GB-0731-RAW` |
| 64 | `M` `crates/codegen/xai-grok-hooks/src/dispatcher.rs` | adopt | `GB-0731-RAW` |
| 65 | `M` `crates/codegen/xai-grok-hooks/src/event.rs` | adopt | `GB-0731-RAW` |
| 66 | `M` `crates/codegen/xai-grok-hooks/src/runner/command.rs` | adopt | `GB-0731-RAW` |
| 67 | `M` `crates/codegen/xai-grok-hooks/src/runner/http.rs` | adopt | `GB-0731-RAW` |
| 68 | `M` `crates/codegen/xai-grok-hooks/src/runner/mod.rs` | adopt | `GB-0731-RAW` |
| 69 | `M` `crates/codegen/xai-grok-hooks/tests/integration.rs` | adopt | `GB-0731-RAW` |
| 70 | `M` `crates/codegen/xai-grok-http/Cargo.toml` | adopt | `GB-0731-RAW` |
| 71 | `M` `crates/codegen/xai-grok-http/src/lib.rs` | adopt | `GB-0731-RAW` |
| 72 | `M` `crates/codegen/xai-grok-mcp/Cargo.toml` | adopt | `GB-0731-RAW` |
| 73 | `M` `crates/codegen/xai-grok-mcp/src/credentials.rs` | adopt | `GB-0731-RAW` |
| 74 | `M` `crates/codegen/xai-grok-mcp/src/oauth.rs` | adopt | `GB-0731-RAW` |
| 75 | `M` `crates/codegen/xai-grok-mcp/src/servers.rs` | adopt | `GB-0731-RAW` |
| 76 | `M` `crates/codegen/xai-grok-mermaid/src/subprocess.rs` | adopt | `GB-0731-RAW` |
| 77 | `M` `crates/codegen/xai-grok-models/default_models.json` | adopt | `GB-0731-RAW` |
| 78 | `M` `crates/codegen/xai-grok-pager-bin/Cargo.toml` | adopt | `GB-0731-RAW` |
| 79 | `M` `crates/codegen/xai-grok-pager-bin/src/main.rs` | adopt | `GB-0731-RAW` |
| 80 | `M` `crates/codegen/xai-grok-pager-minimal/src/commit.rs` | adopt | `GB-0731-RAW` |
| 81 | `A` `crates/codegen/xai-grok-pager-minimal/src/commit_tests.rs` | adopt | `GB-0731-RAW` |
| 82 | `M` `crates/codegen/xai-grok-pager-minimal/src/full_view.rs` | adopt | `GB-0731-RAW` |
| 83 | `M` `crates/codegen/xai-grok-pager-minimal/src/live.rs` | adopt | `GB-0731-RAW` |
| 84 | `M` `crates/codegen/xai-grok-pager-minimal/src/overlay.rs` | adopt | `GB-0731-RAW` |
| 85 | `M` `crates/codegen/xai-grok-pager-minimal/src/panel.rs` | adopt | `GB-0731-RAW` |
| 86 | `M` `crates/codegen/xai-grok-pager-minimal/src/plan.rs` | adopt | `GB-0731-RAW` |
| 87 | `M` `crates/codegen/xai-grok-pager-pty-harness/Cargo.toml` | adopt | `GB-0731-RAW` |
| 88 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/content.rs` | adopt | `GB-0731-RAW` |
| 89 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/flows.rs` | adopt | `GB-0731-RAW` |
| 90 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/host_clipboard.rs` | adopt | `GB-0731-RAW` |
| 91 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/lib.rs` | adopt | `GB-0731-RAW` |
| 92 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/pty.rs` | adopt | `GB-0731-RAW` |
| 93 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/scenarios/idle_cost.rs` | adopt | `GB-0731-RAW` |
| 94 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/scenarios/large_codeblock.rs` | adopt | `GB-0731-RAW` |
| 95 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/scenarios/mixed_interaction.rs` | adopt | `GB-0731-RAW` |
| 96 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/scenarios/resize_storm.rs` | adopt | `GB-0731-RAW` |
| 97 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/scenarios/scroll_stress.rs` | adopt | `GB-0731-RAW` |
| 98 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/scenarios/streaming_render.rs` | adopt | `GB-0731-RAW` |
| 99 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/scripted.rs` | adopt | `GB-0731-RAW` |
| 100 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/scroll_matrix/session.rs` | adopt | `GB-0731-RAW` |
| 101 | `A` `crates/codegen/xai-grok-pager-pty-harness/tests/env_op_compile.rs` | adopt | `GB-0731-RAW` |
| 102 | `A` `crates/codegen/xai-grok-pager-pty-harness/tests/privacy_banner_e2e.rs` | adopt | `GB-0731-RAW` |
| 103 | `M` `crates/codegen/xai-grok-pager-pty-harness/tests/prompt_history_durable_quit.rs` | adopt | `GB-0731-RAW` |
| 104 | `M` `crates/codegen/xai-grok-pager-pty-harness/tests/scroll_correctness_ptyctl.rs` | adopt | `GB-0731-RAW` |
| 105 | `A` `crates/codegen/xai-grok-pager-pty-harness/tests/settings_locked_row_e2e.rs` | adopt | `GB-0731-RAW` |
| 106 | `M` `crates/codegen/xai-grok-pager-render/src/appearance/config.rs` | adopt | `GB-0731-RAW` |
| 107 | `M` `crates/codegen/xai-grok-pager-render/src/clipboard/mod.rs` | adopt | `GB-0731-RAW` |
| 108 | `M` `crates/codegen/xai-grok-pager-render/src/gboom/mod.rs` | adopt | `GB-0731-RAW` |
| 109 | `M` `crates/codegen/xai-grok-pager-render/src/glyphs.rs` | adopt | `GB-0731-RAW` |
| 110 | `M` `crates/codegen/xai-grok-pager-render/src/link_opener.rs` | adopt | `GB-0731-RAW` |
| 111 | `M` `crates/codegen/xai-grok-pager-render/src/render/draw.rs` | adopt | `GB-0731-RAW` |
| 112 | `A` `crates/codegen/xai-grok-pager-render/src/terminal/da2.rs` | adopt | `GB-0731-RAW` |
| 113 | `A` `crates/codegen/xai-grok-pager-render/src/terminal/kitty_keyboard.rs` | adopt | `GB-0731-RAW` |
| 114 | `M` `crates/codegen/xai-grok-pager-render/src/terminal/mod.rs` | adopt | `GB-0731-RAW` |
| 115 | `A` `crates/codegen/xai-grok-pager-render/src/terminal/term_version.rs` | adopt | `GB-0731-RAW` |
| 116 | `M` `crates/codegen/xai-grok-pager-render/src/terminal/test.rs` | adopt | `GB-0731-RAW` |
| 117 | `M` `crates/codegen/xai-grok-pager-render/src/terminal/tmux_probe.rs` | adopt | `GB-0731-RAW` |
| 118 | `M` `crates/codegen/xai-grok-pager-render/src/terminal/xtversion.rs` | adopt | `GB-0731-RAW` |
| 119 | `M` `crates/codegen/xai-grok-pager-render/src/util.rs` | adopt | `GB-0731-RAW` |
| 120 | `M` `crates/codegen/xai-grok-pager/Cargo.toml` | adopt | `GB-0731-RAW` |
| 121 | `M` `crates/codegen/xai-grok-pager/README.md` | adopt | `GB-0731-RAW` |
| 122 | `A` `crates/codegen/xai-grok-pager/benches/resize.rs` | adopt | `GB-0731-RAW` |
| 123 | `M` `crates/codegen/xai-grok-pager/docs/custom-hooks.md` | adopt | `GB-0731-RAW` |
| 124 | `A` `crates/codegen/xai-grok-pager/docs/tutorial/01-coming-from-another-tool.md` | adopt | `GB-0731-RAW` |
| 125 | `A` `crates/codegen/xai-grok-pager/docs/tutorial/02-first-prompt.md` | adopt | `GB-0731-RAW` |
| 126 | `A` `crates/codegen/xai-grok-pager/docs/tutorial/03-attach-and-paste.md` | adopt | `GB-0731-RAW` |
| 127 | `A` `crates/codegen/xai-grok-pager/docs/tutorial/04-navigation.md` | adopt | `GB-0731-RAW` |
| 128 | `A` `crates/codegen/xai-grok-pager/docs/tutorial/05-slash-commands.md` | adopt | `GB-0731-RAW` |
| 129 | `A` `crates/codegen/xai-grok-pager/docs/tutorial/06-worktrees.md` | adopt | `GB-0731-RAW` |
| 130 | `A` `crates/codegen/xai-grok-pager/docs/tutorial/07-plan-and-permissions.md` | adopt | `GB-0731-RAW` |
| 131 | `A` `crates/codegen/xai-grok-pager/docs/tutorial/08-make-it-yours.md` | adopt | `GB-0731-RAW` |
| 132 | `A` `crates/codegen/xai-grok-pager/docs/tutorial/09-where-next.md` | adopt | `GB-0731-RAW` |
| 133 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/01-getting-started.md` | adopt | `GB-0731-RAW` |
| 134 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/02-authentication.md` | adopt | `GB-0731-RAW` |
| 135 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/03-keyboard-shortcuts.md` | adopt | `GB-0731-RAW` |
| 136 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/04-slash-commands.md` | adopt | `GB-0731-RAW` |
| 137 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/05-configuration.md` | adopt | `GB-0731-RAW` |
| 138 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/06-theming.md` | adopt | `GB-0731-RAW` |
| 139 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/07-mcp-servers.md` | adopt | `GB-0731-RAW` |
| 140 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/08-skills.md` | adopt | `GB-0731-RAW` |
| 141 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/09-plugins.md` | adopt | `GB-0731-RAW` |
| 142 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/10-hooks.md` | adopt | `GB-0731-RAW` |
| 143 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/11-custom-models.md` | adopt | `GB-0731-RAW` |
| 144 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/14-headless-mode.md` | adopt | `GB-0731-RAW` |
| 145 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/15-agent-mode.md` | adopt | `GB-0731-RAW` |
| 146 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/16-subagents.md` | adopt | `GB-0731-RAW` |
| 147 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/17-sessions.md` | adopt | `GB-0731-RAW` |
| 148 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/18-sandbox.md` | adopt | `GB-0731-RAW` |
| 149 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/19-plan-mode.md` | adopt | `GB-0731-RAW` |
| 150 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/20-background-tasks.md` | adopt | `GB-0731-RAW` |
| 151 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/21-terminal-support.md` | adopt | `GB-0731-RAW` |
| 152 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/22-permissions-and-safety.md` | adopt | `GB-0731-RAW` |
| 153 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/23-dashboard.md` | adopt | `GB-0731-RAW` |
| 154 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/24-monitoring-usage.md` | adopt | `GB-0731-RAW` |
| 155 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/README.md` | adopt | `GB-0731-RAW` |
| 156 | `M` `crates/codegen/xai-grok-pager/npm/grok/bin/grok` | adopt | `GB-0731-RAW` |
| 157 | `M` `crates/codegen/xai-grok-pager/npm/grok/bin/postinstall.js` | adopt | `GB-0731-RAW` |
| 158 | `M` `crates/codegen/xai-grok-pager/npm/grok/scripts/test-postinstall.js` | adopt | `GB-0731-RAW` |
| 159 | `M` `crates/codegen/xai-grok-pager/src/acp/mod.rs` | adopt | `GB-0731-RAW` |
| 160 | `M` `crates/codegen/xai-grok-pager/src/acp/spawn.rs` | adopt | `GB-0731-RAW` |
| 161 | `M` `crates/codegen/xai-grok-pager/src/acp/tracker.rs` | adopt | `GB-0731-RAW` |
| 162 | `M` `crates/codegen/xai-grok-pager/src/actions/defaults.rs` | adopt | `GB-0731-RAW` |
| 163 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/background.rs` | adopt | `GB-0731-RAW` |
| 164 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/interactions.rs` | adopt | `GB-0731-RAW` |
| 165 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/mcp.rs` | adopt | `GB-0731-RAW` |
| 166 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/mod.rs` | adopt | `GB-0731-RAW` |
| 167 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/permissions.rs` | adopt | `GB-0731-RAW` |
| 168 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/prompt_origin.rs` | adopt | `GB-0731-RAW` |
| 169 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/queue.rs` | adopt | `GB-0731-RAW` |
| 170 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/session_notification.rs` | adopt | `GB-0731-RAW` |
| 171 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/settings.rs` | adopt | `GB-0731-RAW` |
| 172 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/announcements.rs` | adopt | `GB-0731-RAW` |
| 173 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/interjection.rs` | adopt | `GB-0731-RAW` |
| 174 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/mod.rs` | adopt | `GB-0731-RAW` |
| 175 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/permissions.rs` | adopt | `GB-0731-RAW` |
| 176 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/plan_mode.rs` | adopt | `GB-0731-RAW` |
| 177 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/queue_and_adoption.rs` | adopt | `GB-0731-RAW` |
| 178 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/session_events.rs` | adopt | `GB-0731-RAW` |
| 179 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/settings.rs` | adopt | `GB-0731-RAW` |
| 180 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/turn_completion.rs` | adopt | `GB-0731-RAW` |
| 181 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/workflow_ingest.rs` | adopt | `GB-0731-RAW` |
| 182 | `M` `crates/codegen/xai-grok-pager/src/app/actions.rs` | adopt | `GB-0731-RAW` |
| 183 | `M` `crates/codegen/xai-grok-pager/src/app/agent.rs` | adopt | `GB-0731-RAW` |
| 184 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/input.rs` | adopt | `GB-0731-RAW` |
| 185 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/interactions.rs` | adopt | `GB-0731-RAW` |
| 186 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/links.rs` | adopt | `GB-0731-RAW` |
| 187 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/mod.rs` | adopt | `GB-0731-RAW` |
| 188 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/modals.rs` | adopt | `GB-0731-RAW` |
| 189 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/notices.rs` | adopt | `GB-0731-RAW` |
| 190 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/paste.rs` | adopt | `GB-0731-RAW` |
| 191 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/plan.rs` | adopt | `GB-0731-RAW` |
| 192 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/prompt.rs` | adopt | `GB-0731-RAW` |
| 193 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/queue.rs` | adopt | `GB-0731-RAW` |
| 194 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/render.rs` | adopt | `GB-0731-RAW` |
| 195 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/rewind.rs` | adopt | `GB-0731-RAW` |
| 196 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/session.rs` | adopt | `GB-0731-RAW` |
| 197 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/viewer.rs` | adopt | `GB-0731-RAW` |
| 198 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/workflows_overlay.rs` | adopt | `GB-0731-RAW` |
| 199 | `M` `crates/codegen/xai-grok-pager/src/app/app_view.rs` | adopt | `GB-0731-RAW` |
| 200 | `M` `crates/codegen/xai-grok-pager/src/app/cli.rs` | adopt | `GB-0731-RAW` |
| 201 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/cta.rs` | adopt | `GB-0731-RAW` |
| 202 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/ctx.rs` | adopt | `GB-0731-RAW` |
| 203 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/dashboard.rs` | adopt | `GB-0731-RAW` |
| 204 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/interject.rs` | adopt | `GB-0731-RAW` |
| 205 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/modes.rs` | adopt | `GB-0731-RAW` |
| 206 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/permissions.rs` | adopt | `GB-0731-RAW` |
| 207 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/prompt.rs` | adopt | `GB-0731-RAW` |
| 208 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/queue.rs` | adopt | `GB-0731-RAW` |
| 209 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/router.rs` | adopt | `GB-0731-RAW` |
| 210 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/session/fork.rs` | adopt | `GB-0731-RAW` |
| 211 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs` | adopt | `GB-0731-RAW` |
| 212 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/session/load.rs` | adopt | `GB-0731-RAW` |
| 213 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/settings/setters.rs` | adopt | `GB-0731-RAW` |
| 214 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/settings/ui.rs` | adopt | `GB-0731-RAW` |
| 215 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/status.rs` | adopt | `GB-0731-RAW` |
| 216 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/task_result.rs` | adopt | `GB-0731-RAW` |
| 217 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/auth.rs` | adopt | `GB-0731-RAW` |
| 218 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/billing.rs` | adopt | `GB-0731-RAW` |
| 219 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/cta_e2e.rs` | adopt | `GB-0731-RAW` |
| 220 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/dashboard.rs` | adopt | `GB-0731-RAW` |
| 221 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/mod.rs` | adopt | `GB-0731-RAW` |
| 222 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/modes.rs` | adopt | `GB-0731-RAW` |
| 223 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/prompt.rs` | adopt | `GB-0731-RAW` |
| 224 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/router.rs` | adopt | `GB-0731-RAW` |
| 225 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/foreign.rs` | adopt | `GB-0731-RAW` |
| 226 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/fork.rs` | adopt | `GB-0731-RAW` |
| 227 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/lifecycle.rs` | adopt | `GB-0731-RAW` |
| 228 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/load.rs` | adopt | `GB-0731-RAW` |
| 229 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/modal.rs` | adopt | `GB-0731-RAW` |
| 230 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/settings.rs` | adopt | `GB-0731-RAW` |
| 231 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/status.rs` | adopt | `GB-0731-RAW` |
| 232 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/task_result.rs` | adopt | `GB-0731-RAW` |
| 233 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/turn.rs` | adopt | `GB-0731-RAW` |
| 234 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/voice.rs` | adopt | `GB-0731-RAW` |
| 235 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/transcript.rs` | adopt | `GB-0731-RAW` |
| 236 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/turn.rs` | adopt | `GB-0731-RAW` |
| 237 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/voice.rs` | adopt | `GB-0731-RAW` |
| 238 | `M` `crates/codegen/xai-grok-pager/src/app/display_refresh_startup.rs` | adopt | `GB-0731-RAW` |
| 239 | `M` `crates/codegen/xai-grok-pager/src/app/effects/helpers.rs` | adopt | `GB-0731-RAW` |
| 240 | `M` `crates/codegen/xai-grok-pager/src/app/effects/mod.rs` | adopt | `GB-0731-RAW` |
| 241 | `M` `crates/codegen/xai-grok-pager/src/app/effects/tests.rs` | adopt | `GB-0731-RAW` |
| 242 | `M` `crates/codegen/xai-grok-pager/src/app/event_loop.rs` | adopt | `GB-0731-RAW` |
| 243 | `M` `crates/codegen/xai-grok-pager/src/app/external_editor.rs` | adopt | `GB-0731-RAW` |
| 244 | `M` `crates/codegen/xai-grok-pager/src/app/leader_cluster/mod.rs` | adopt | `GB-0731-RAW` |
| 245 | `M` `crates/codegen/xai-grok-pager/src/app/leader_cluster/scenarios.rs` | adopt | `GB-0731-RAW` |
| 246 | `M` `crates/codegen/xai-grok-pager/src/app/mod.rs` | adopt | `GB-0731-RAW` |
| 247 | `M` `crates/codegen/xai-grok-pager/src/app/modals.rs` | adopt | `GB-0731-RAW` |
| 248 | `M` `crates/codegen/xai-grok-pager/src/app/mouse.rs` | adopt | `GB-0731-RAW` |
| 249 | `M` `crates/codegen/xai-grok-pager/src/app/queue_edit.rs` | adopt | `GB-0731-RAW` |
| 250 | `M` `crates/codegen/xai-grok-pager/src/app/screen_mode_relaunch.rs` | adopt | `GB-0731-RAW` |
| 251 | `M` `crates/codegen/xai-grok-pager/src/app/session_startup.rs` | adopt | `GB-0731-RAW` |
| 252 | `A` `crates/codegen/xai-grok-pager/src/app/session_title_resolve.rs` | adopt | `GB-0731-RAW` |
| 253 | `A` `crates/codegen/xai-grok-pager/src/app/session_title_resolve_tests.rs` | adopt | `GB-0731-RAW` |
| 254 | `M` `crates/codegen/xai-grok-pager/src/app/status_blocks.rs` | adopt | `GB-0731-RAW` |
| 255 | `M` `crates/codegen/xai-grok-pager/src/app/subagent.rs` | adopt | `GB-0731-RAW` |
| 256 | `M` `crates/codegen/xai-grok-pager/src/app/turn_completion.rs` | adopt | `GB-0731-RAW` |
| 257 | `M` `crates/codegen/xai-grok-pager/src/app/turn_completion/tests.rs` | adopt | `GB-0731-RAW` |
| 258 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/doctor_format.rs` | adopt | `GB-0731-RAW` |
| 259 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/doctor_format_tests.rs` | adopt | `GB-0731-RAW` |
| 260 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/fix.rs` | adopt | `GB-0731-RAW` |
| 261 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/fix_tests.rs` | adopt | `GB-0731-RAW` |
| 262 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/mod.rs` | adopt | `GB-0731-RAW` |
| 263 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/model.rs` | adopt | `GB-0731-RAW` |
| 264 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/probes/mod.rs` | adopt | `GB-0731-RAW` |
| 265 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/view.rs` | adopt | `GB-0731-RAW` |
| 266 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/view_tests.rs` | adopt | `GB-0731-RAW` |
| 267 | `M` `crates/codegen/xai-grok-pager/src/docs.rs` | adopt | `GB-0731-RAW` |
| 268 | `M` `crates/codegen/xai-grok-pager/src/doctor_cmd/human.rs` | adopt | `GB-0731-RAW` |
| 269 | `M` `crates/codegen/xai-grok-pager/src/doctor_cmd/json.rs` | adopt | `GB-0731-RAW` |
| 270 | `M` `crates/codegen/xai-grok-pager/src/doctor_cmd/mod.rs` | adopt | `GB-0731-RAW` |
| 271 | `M` `crates/codegen/xai-grok-pager/src/doctor_cmd/tests.rs` | adopt | `GB-0731-RAW` |
| 272 | `M` `crates/codegen/xai-grok-pager/src/headless.rs` | adopt | `GB-0731-RAW` |
| 273 | `A` `crates/codegen/xai-grok-pager/src/headless/cli.rs` | adopt | `GB-0731-RAW` |
| 274 | `A` `crates/codegen/xai-grok-pager/src/headless/ext_protocol.rs` | adopt | `GB-0731-RAW` |
| 275 | `A` `crates/codegen/xai-grok-pager/src/headless/ext_protocol_tests.rs` | adopt | `GB-0731-RAW` |
| 276 | `A` `crates/codegen/xai-grok-pager/src/headless/reducer/acp.rs` | adopt | `GB-0731-RAW` |
| 277 | `A` `crates/codegen/xai-grok-pager/src/headless/reducer/messages/mod.rs` | adopt | `GB-0731-RAW` |
| 278 | `A` `crates/codegen/xai-grok-pager/src/headless/reducer/messages/partial.rs` | adopt | `GB-0731-RAW` |
| 279 | `A` `crates/codegen/xai-grok-pager/src/headless/reducer/messages/state.rs` | adopt | `GB-0731-RAW` |
| 280 | `A` `crates/codegen/xai-grok-pager/src/headless/reducer/messages/tests/acp_reducer.rs` | adopt | `GB-0731-RAW` |
| 281 | `A` `crates/codegen/xai-grok-pager/src/headless/reducer/messages/tests/content.rs` | adopt | `GB-0731-RAW` |
| 282 | `A` `crates/codegen/xai-grok-pager/src/headless/reducer/messages/tests/init.rs` | adopt | `GB-0731-RAW` |
| 283 | `A` `crates/codegen/xai-grok-pager/src/headless/reducer/messages/tests/mod.rs` | adopt | `GB-0731-RAW` |
| 284 | `A` `crates/codegen/xai-grok-pager/src/headless/reducer/messages/tests/partial.rs` | adopt | `GB-0731-RAW` |
| 285 | `A` `crates/codegen/xai-grok-pager/src/headless/reducer/messages/tests/result_usage.rs` | adopt | `GB-0731-RAW` |
| 286 | `A` `crates/codegen/xai-grok-pager/src/headless/reducer/messages/tests/tool_calls.rs` | adopt | `GB-0731-RAW` |
| 287 | `A` `crates/codegen/xai-grok-pager/src/headless/reducer/messages/tests/web_search.rs` | adopt | `GB-0731-RAW` |
| 288 | `A` `crates/codegen/xai-grok-pager/src/headless/reducer/messages/usage.rs` | adopt | `GB-0731-RAW` |
| 289 | `A` `crates/codegen/xai-grok-pager/src/headless/reducer/messages/web_search.rs` | adopt | `GB-0731-RAW` |
| 290 | `A` `crates/codegen/xai-grok-pager/src/headless/reducer/messages/wire.rs` | adopt | `GB-0731-RAW` |
| 291 | `A` `crates/codegen/xai-grok-pager/src/headless/reducer/mod.rs` | adopt | `GB-0731-RAW` |
| 292 | `A` `crates/codegen/xai-grok-pager/src/headless_tests.rs` | adopt | `GB-0731-RAW` |
| 293 | `M` `crates/codegen/xai-grok-pager/src/input/mouse.rs` | adopt | `GB-0731-RAW` |
| 294 | `M` `crates/codegen/xai-grok-pager/src/input/mouse/tests.rs` | adopt | `GB-0731-RAW` |
| 295 | `M` `crates/codegen/xai-grok-pager/src/input/terminal_support.rs` | adopt | `GB-0731-RAW` |
| 296 | `M` `crates/codegen/xai-grok-pager/src/lib.rs` | adopt | `GB-0731-RAW` |
| 297 | `M` `crates/codegen/xai-grok-pager/src/mcp_cmd.rs` | adopt | `GB-0731-RAW` |
| 298 | `M` `crates/codegen/xai-grok-pager/src/minimal/api.rs` | adopt | `GB-0731-RAW` |
| 299 | `M` `crates/codegen/xai-grok-pager/src/models.rs` | adopt | `GB-0731-RAW` |
| 300 | `M` `crates/codegen/xai-grok-pager/src/notifications/hooks.rs` | adopt | `GB-0731-RAW` |
| 301 | `M` `crates/codegen/xai-grok-pager/src/notifications/sleep.rs` | adopt | `GB-0731-RAW` |
| 302 | `M` `crates/codegen/xai-grok-pager/src/plugin_cmd.rs` | adopt | `GB-0731-RAW` |
| 303 | `M` `crates/codegen/xai-grok-pager/src/scrollback/block.rs` | adopt | `GB-0731-RAW` |
| 304 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/agent.rs` | adopt | `GB-0731-RAW` |
| 305 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/bg_task.rs` | adopt | `GB-0731-RAW` |
| 306 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/btw.rs` | adopt | `GB-0731-RAW` |
| 307 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/context_info.rs` | adopt | `GB-0731-RAW` |
| 308 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/credit_limit.rs` | adopt | `GB-0731-RAW` |
| 309 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/session_event.rs` | adopt | `GB-0731-RAW` |
| 310 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/subagent.rs` | adopt | `GB-0731-RAW` |
| 311 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/system.rs` | adopt | `GB-0731-RAW` |
| 312 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/thinking.rs` | adopt | `GB-0731-RAW` |
| 313 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/edit.rs` | adopt | `GB-0731-RAW` |
| 314 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/execute.rs` | adopt | `GB-0731-RAW` |
| 315 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/lifecycle.rs` | adopt | `GB-0731-RAW` |
| 316 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/list_dir.rs` | adopt | `GB-0731-RAW` |
| 317 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/memory_search.rs` | adopt | `GB-0731-RAW` |
| 318 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/mod.rs` | adopt | `GB-0731-RAW` |
| 319 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/other.rs` | adopt | `GB-0731-RAW` |
| 320 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/read.rs` | adopt | `GB-0731-RAW` |
| 321 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/search.rs` | adopt | `GB-0731-RAW` |
| 322 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/search_tool.rs` | adopt | `GB-0731-RAW` |
| 323 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/use_tool.rs` | adopt | `GB-0731-RAW` |
| 324 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/web_fetch.rs` | adopt | `GB-0731-RAW` |
| 325 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/web_search.rs` | adopt | `GB-0731-RAW` |
| 326 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/user.rs` | adopt | `GB-0731-RAW` |
| 327 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/workflow.rs` | adopt | `GB-0731-RAW` |
| 328 | `M` `crates/codegen/xai-grok-pager/src/scrollback/entry.rs` | adopt | `GB-0731-RAW` |
| 329 | `M` `crates/codegen/xai-grok-pager/src/scrollback/render.rs` | adopt | `GB-0731-RAW` |
| 330 | `M` `crates/codegen/xai-grok-pager/src/scrollback/state/layout.rs` | adopt | `GB-0731-RAW` |
| 331 | `M` `crates/codegen/xai-grok-pager/src/scrollback/state/mod.rs` | adopt | `GB-0731-RAW` |
| 332 | `M` `crates/codegen/xai-grok-pager/src/scrollback/state/nav.rs` | adopt | `GB-0731-RAW` |
| 333 | `M` `crates/codegen/xai-grok-pager/src/scrollback/wrappers/entry_renderer.rs` | adopt | `GB-0731-RAW` |
| 334 | `M` `crates/codegen/xai-grok-pager/src/sessions_cmd.rs` | adopt | `GB-0731-RAW` |
| 335 | `M` `crates/codegen/xai-grok-pager/src/settings/defs.rs` | adopt | `GB-0731-RAW` |
| 336 | `M` `crates/codegen/xai-grok-pager/src/settings/mod.rs` | adopt | `GB-0731-RAW` |
| 337 | `M` `crates/codegen/xai-grok-pager/src/settings/registry.rs` | adopt | `GB-0731-RAW` |
| 338 | `M` `crates/codegen/xai-grok-pager/src/share_cmd.rs` | adopt | `GB-0731-RAW` |
| 339 | `M` `crates/codegen/xai-grok-pager/src/slash/acp_command.rs` | adopt | `GB-0731-RAW` |
| 340 | `M` `crates/codegen/xai-grok-pager/src/slash/command.rs` | adopt | `GB-0731-RAW` |
| 341 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/always_approve.rs` | adopt | `GB-0731-RAW` |
| 342 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/announcements.rs` | adopt | `GB-0731-RAW` |
| 343 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/auto.rs` | adopt | `GB-0731-RAW` |
| 344 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/btw.rs` | adopt | `GB-0731-RAW` |
| 345 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/cd.rs` | adopt | `GB-0731-RAW` |
| 346 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/copy.rs` | adopt | `GB-0731-RAW` |
| 347 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/dashboard.rs` | adopt | `GB-0731-RAW` |
| 348 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/debug.rs` | adopt | `GB-0731-RAW` |
| 349 | `A` `crates/codegen/xai-grok-pager/src/slash/commands/delete.rs` | adopt | `GB-0731-RAW` |
| 350 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/docs.rs` | adopt | `GB-0731-RAW` |
| 351 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/doctor.rs` | adopt | `GB-0731-RAW` |
| 352 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/edit_prompt.rs` | adopt | `GB-0731-RAW` |
| 353 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/effort.rs` | adopt | `GB-0731-RAW` |
| 354 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/expand.rs` | adopt | `GB-0731-RAW` |
| 355 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/export.rs` | adopt | `GB-0731-RAW` |
| 356 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/find.rs` | adopt | `GB-0731-RAW` |
| 357 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/fork.rs` | adopt | `GB-0731-RAW` |
| 358 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/help.rs` | adopt | `GB-0731-RAW` |
| 359 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/history.rs` | adopt | `GB-0731-RAW` |
| 360 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/jump.rs` | adopt | `GB-0731-RAW` |
| 361 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/loop_cmd.rs` | adopt | `GB-0731-RAW` |
| 362 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/mod.rs` | adopt | `GB-0731-RAW` |
| 363 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/model.rs` | adopt | `GB-0731-RAW` |
| 364 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/multiline.rs` | adopt | `GB-0731-RAW` |
| 365 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/plan.rs` | adopt | `GB-0731-RAW` |
| 366 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/privacy.rs` | adopt | `GB-0731-RAW` |
| 367 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/queue.rs` | adopt | `GB-0731-RAW` |
| 368 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/rewind.rs` | adopt | `GB-0731-RAW` |
| 369 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/screen_mode_switch.rs` | adopt | `GB-0731-RAW` |
| 370 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/settings_cmd.rs` | adopt | `GB-0731-RAW` |
| 371 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/share.rs` | adopt | `GB-0731-RAW` |
| 372 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/tasks.rs` | adopt | `GB-0731-RAW` |
| 373 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/theme.rs` | adopt | `GB-0731-RAW` |
| 374 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/timeline.rs` | adopt | `GB-0731-RAW` |
| 375 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/toggle_mouse_reporting.rs` | adopt | `GB-0731-RAW` |
| 376 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/transcript.rs` | adopt | `GB-0731-RAW` |
| 377 | `A` `crates/codegen/xai-grok-pager/src/slash/commands/tutorial.rs` | adopt | `GB-0731-RAW` |
| 378 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/usage.rs` | adopt | `GB-0731-RAW` |
| 379 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/voice.rs` | adopt | `GB-0731-RAW` |
| 380 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/workflows.rs` | adopt | `GB-0731-RAW` |
| 381 | `M` `crates/codegen/xai-grok-pager/src/slash/matcher.rs` | adopt | `GB-0731-RAW` |
| 382 | `M` `crates/codegen/xai-grok-pager/src/slash/mod.rs` | adopt | `GB-0731-RAW` |
| 383 | `A` `crates/codegen/xai-grok-pager/src/slash/mode_support.rs` | adopt | `GB-0731-RAW` |
| 384 | `A` `crates/codegen/xai-grok-pager/src/slash/mode_support_tests.rs` | adopt | `GB-0731-RAW` |
| 385 | `M` `crates/codegen/xai-grok-pager/src/slash/registry.rs` | adopt | `GB-0731-RAW` |
| 386 | `M` `crates/codegen/xai-grok-pager/src/startup.rs` | adopt | `GB-0731-RAW` |
| 387 | `M` `crates/codegen/xai-grok-pager/src/test_util.rs` | adopt | `GB-0731-RAW` |
| 388 | `M` `crates/codegen/xai-grok-pager/src/tips/ssh_wrap.rs` | adopt | `GB-0731-RAW` |
| 389 | `M` `crates/codegen/xai-grok-pager/src/tracing.rs` | adopt | `GB-0731-RAW` |
| 390 | `A` `crates/codegen/xai-grok-pager/src/tutorial_docs.rs` | adopt | `GB-0731-RAW` |
| 391 | `M` `crates/codegen/xai-grok-pager/src/views/agent.rs` | adopt | `GB-0731-RAW` |
| 392 | `M` `crates/codegen/xai-grok-pager/src/views/agents_modal.rs` | adopt | `GB-0731-RAW` |
| 393 | `M` `crates/codegen/xai-grok-pager/src/views/announcements.rs` | adopt | `GB-0731-RAW` |
| 394 | `M` `crates/codegen/xai-grok-pager/src/views/dashboard/peek.rs` | adopt | `GB-0731-RAW` |
| 395 | `M` `crates/codegen/xai-grok-pager/src/views/dashboard/render.rs` | adopt | `GB-0731-RAW` |
| 396 | `M` `crates/codegen/xai-grok-pager/src/views/dashboard/row.rs` | adopt | `GB-0731-RAW` |
| 397 | `M` `crates/codegen/xai-grok-pager/src/views/dashboard/state.rs` | adopt | `GB-0731-RAW` |
| 398 | `M` `crates/codegen/xai-grok-pager/src/views/extensions_modal.rs` | adopt | `GB-0731-RAW` |
| 399 | `M` `crates/codegen/xai-grok-pager/src/views/file_search/line_viewer.rs` | adopt | `GB-0731-RAW` |
| 400 | `M` `crates/codegen/xai-grok-pager/src/views/file_search/state.rs` | adopt | `GB-0731-RAW` |
| 401 | `M` `crates/codegen/xai-grok-pager/src/views/history_search.rs` | adopt | `GB-0731-RAW` |
| 402 | `M` `crates/codegen/xai-grok-pager/src/views/mod.rs` | adopt | `GB-0731-RAW` |
| 403 | `M` `crates/codegen/xai-grok-pager/src/views/modal.rs` | adopt | `GB-0731-RAW` |
| 404 | `M` `crates/codegen/xai-grok-pager/src/views/picker.rs` | adopt | `GB-0731-RAW` |
| 405 | `A` `crates/codegen/xai-grok-pager/src/views/privacy_banner.rs` | adopt | `GB-0731-RAW` |
| 406 | `M` `crates/codegen/xai-grok-pager/src/views/prompt_widget/mod.rs` | adopt | `GB-0731-RAW` |
| 407 | `M` `crates/codegen/xai-grok-pager/src/views/prompt_widget/tests.rs` | adopt | `GB-0731-RAW` |
| 408 | `M` `crates/codegen/xai-grok-pager/src/views/question_view.rs` | adopt | `GB-0731-RAW` |
| 409 | `M` `crates/codegen/xai-grok-pager/src/views/queue_pane.rs` | adopt | `GB-0731-RAW` |
| 410 | `M` `crates/codegen/xai-grok-pager/src/views/session_picker.rs` | adopt | `GB-0731-RAW` |
| 411 | `M` `crates/codegen/xai-grok-pager/src/views/settings_modal/input.rs` | adopt | `GB-0731-RAW` |
| 412 | `M` `crates/codegen/xai-grok-pager/src/views/settings_modal/render.rs` | adopt | `GB-0731-RAW` |
| 413 | `M` `crates/codegen/xai-grok-pager/src/views/settings_modal/state.rs` | adopt | `GB-0731-RAW` |
| 414 | `M` `crates/codegen/xai-grok-pager/src/views/settings_modal/tests.rs` | adopt | `GB-0731-RAW` |
| 415 | `M` `crates/codegen/xai-grok-pager/src/views/shortcuts_help.rs` | adopt | `GB-0731-RAW` |
| 416 | `M` `crates/codegen/xai-grok-pager/src/views/slash_dropdown.rs` | adopt | `GB-0731-RAW` |
| 417 | `M` `crates/codegen/xai-grok-pager/src/views/tasks_pane.rs` | adopt | `GB-0731-RAW` |
| 418 | `M` `crates/codegen/xai-grok-pager/src/views/turn_status.rs` | adopt | `GB-0731-RAW` |
| 419 | `A` `crates/codegen/xai-grok-pager/src/views/tutorial.rs` | adopt | `GB-0731-RAW` |
| 420 | `M` `crates/codegen/xai-grok-pager/src/views/welcome/mod.rs` | adopt | `GB-0731-RAW` |
| 421 | `A` `crates/codegen/xai-grok-pager/src/views/welcome/toast.rs` | adopt | `GB-0731-RAW` |
| 422 | `M` `crates/codegen/xai-grok-pager/src/views/workflows.rs` | adopt | `GB-0731-RAW` |
| 423 | `M` `crates/codegen/xai-grok-pager/src/voice/handle.rs` | adopt | `GB-0731-RAW` |
| 424 | `M` `crates/codegen/xai-grok-pager/src/voice/mod.rs` | adopt | `GB-0731-RAW` |
| 425 | `M` `crates/codegen/xai-grok-pager/src/worktree_cmd/mod.rs` | adopt | `GB-0731-RAW` |
| 426 | `M` `crates/codegen/xai-grok-pager/tests/doctor_early_dispatch.rs` | adopt | `GB-0731-RAW` |
| 427 | `M` `crates/codegen/xai-grok-pager/tests/leader_pty_e2e/campaign_leader_mode_remote_dismiss_on_model_pick.rs` | adopt | `GB-0731-RAW` |
| 428 | `M` `crates/codegen/xai-grok-pager/tests/leader_pty_e2e/common.rs` | adopt | `GB-0731-RAW` |
| 429 | `M` `crates/codegen/xai-grok-pager/tests/leader_pty_e2e/leader_n_clients_shared_session.rs` | adopt | `GB-0731-RAW` |
| 430 | `M` `crates/codegen/xai-grok-pager/tests/leader_pty_e2e/leader_reattach_cancellation_roundtrips_durable_log.rs` | adopt | `GB-0731-RAW` |
| 431 | `M` `crates/codegen/xai-grok-pager/tests/leader_pty_e2e/leader_reattach_completion_roundtrips_durable_log.rs` | adopt | `GB-0731-RAW` |
| 432 | `M` `crates/codegen/xai-grok-pager/tests/leader_pty_e2e/leader_two_clients_shared_session.rs` | adopt | `GB-0731-RAW` |
| 433 | `M` `crates/codegen/xai-grok-pager/tests/pty_auto_mode.rs` | adopt | `GB-0731-RAW` |
| 434 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/agent_type_mismatch_no_keeps_current_session.rs` | adopt | `GB-0731-RAW` |
| 435 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/agent_type_mismatch_yes_starts_new_session.rs` | adopt | `GB-0731-RAW` |
| 436 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/auto_compact_top_row.rs` | adopt | `GB-0731-RAW` |
| 437 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/background_task_reaped_on_quit.rs` | adopt | `GB-0731-RAW` |
| 438 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/bash_full_output_double_click_fold_pty.rs` | adopt | `GB-0731-RAW` |
| 439 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/bash_mode_file_completion_shell_like.rs` | adopt | `GB-0731-RAW` |
| 440 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/bash_mode_tab_completion_dropdown.rs` | adopt | `GB-0731-RAW` |
| 441 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/bracketed_ime_paste_skips_clipboard_image_linux.rs` | adopt | `GB-0731-RAW` |
| 442 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/bracketed_ime_paste_skips_clipboard_image_macos.rs` | adopt | `GB-0731-RAW` |
| 443 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/campaign_nudges_default_until_dismissed_by_model_pick.rs` | adopt | `GB-0731-RAW` |
| 444 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/campaign_remote_settings_nudge_and_dismiss.rs` | adopt | `GB-0731-RAW` |
| 445 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/common.rs` | adopt | `GB-0731-RAW` |
| 446 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/critical_announcement_session_banner_pty.rs` | adopt | `GB-0731-RAW` |
| 447 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/ctrl_c_cancel_during_stream_recovers_cleanly.rs` | adopt | `GB-0731-RAW` |
| 448 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/doubled_lines_out_of_band_repro.rs` | adopt | `GB-0731-RAW` |
| 449 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/drag_enters_content_from_gap_pty.rs` | adopt | `GB-0731-RAW` |
| 450 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/drag_from_above_prompt_strip_pty.rs` | adopt | `GB-0731-RAW` |
| 451 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/drag_from_chrome_stays_block_pty.rs` | adopt | `GB-0731-RAW` |
| 452 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/drag_over_gap_rows_does_not_freeze_head_pty.rs` | adopt | `GB-0731-RAW` |
| 453 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/drag_select_autoscroll_full_scrollout_copy_pty.rs` | adopt | `GB-0731-RAW` |
| 454 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/embedded_mode_boots_without_hanging_on_blocked_backend.rs` | adopt | `GB-0731-RAW` |
| 455 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/empty_enter_sends_top_not_last_of_two.rs` | adopt | `GB-0731-RAW` |
| 456 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/endline_park_is_markerless.rs` | adopt | `GB-0731-RAW` |
| 457 | `D` `crates/codegen/xai-grok-pager/tests/pty_e2e/endline_park_two_static_markers.rs` | adopt | `GB-0731-RAW` |
| 458 | `D` `crates/codegen/xai-grok-pager/tests/pty_e2e/endline_wakeups_are_markerless.rs` | adopt | `GB-0731-RAW` |
| 459 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/endline_wakeups_close_with_markers.rs` | adopt | `GB-0731-RAW` |
| 460 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/esc_cancels_running_turn_from_prompt_preserves_draft.rs` | adopt | `GB-0731-RAW` |
| 461 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/esc_cancels_running_turn_from_scrollback.rs` | adopt | `GB-0731-RAW` |
| 462 | `D` `crates/codegen/xai-grok-pager/tests/pty_e2e/esc_mid_turn_from_prompt_is_swallowed_preserves_draft.rs` | adopt | `GB-0731-RAW` |
| 463 | `D` `crates/codegen/xai-grok-pager/tests/pty_e2e/esc_mid_turn_from_scrollback_is_swallowed.rs` | adopt | `GB-0731-RAW` |
| 464 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/file_path_with_space_emits_full_osc8_hyperlink.rs` | adopt | `GB-0731-RAW` |
| 465 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/folder_trust_cwd_is_home_git_repo_no_prompt.rs` | adopt | `GB-0731-RAW` |
| 466 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/folder_trust_decline_quits_without_grant.rs` | adopt | `GB-0731-RAW` |
| 467 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/folder_trust_feature_off_shows_no_question.rs` | adopt | `GB-0731-RAW` |
| 468 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/folder_trust_home_git_repo_subdir_keys_on_subdir.rs` | adopt | `GB-0731-RAW` |
| 469 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/folder_trust_question_renders_and_accept_persists_grant.rs` | adopt | `GB-0731-RAW` |
| 470 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/forced_wheel_mode_env_scrolls_exact_rows.rs` | adopt | `GB-0731-RAW` |
| 471 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/interjection_reaches_model_ctrl_l_in_vscode_family.rs` | adopt | `GB-0731-RAW` |
| 472 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/iterm_readline_editing.rs` | adopt | `GB-0731-RAW` |
| 473 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/managed_policy_gate_refusal_reaches_real_terminal.rs` | adopt | `GB-0731-RAW` |
| 474 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/mid_turn_slash_dropdown_esc_dismisses_not_cancel.rs` | adopt | `GB-0731-RAW` |
| 475 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/middle_click_pastes_primary_linux.rs` | adopt | `GB-0731-RAW` |
| 476 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_commits_thinking_body_to_scrollback.rs` | adopt | `GB-0731-RAW` |
| 477 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_continue_reprints_transcript.rs` | adopt | `GB-0731-RAW` |
| 478 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_ctrl_c_arms_and_quits.rs` | adopt | `GB-0731-RAW` |
| 479 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_ctrl_o_send_now_queued_apple_terminal.rs` | adopt | `GB-0731-RAW` |
| 480 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_double_esc_committed_queued_prompt_single_render.rs` | adopt | `GB-0731-RAW` |
| 481 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_esc_cancels_running_turn.rs` | adopt | `GB-0731-RAW` |
| 482 | `D` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_esc_mid_turn_is_swallowed.rs` | adopt | `GB-0731-RAW` |
| 483 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_external_editor_round_trip.rs` | adopt | `GB-0731-RAW` |
| 484 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_parked_plan_commits_to_scrollback.rs` | adopt | `GB-0731-RAW` |
| 485 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_parked_plan_survives_quit.rs` | adopt | `GB-0731-RAW` |
| 486 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_resize_preserves_committed_scrollback.rs` | adopt | `GB-0731-RAW` |
| 487 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_thinking_is_visually_distinct_from_output.rs` | adopt | `GB-0731-RAW` |
| 488 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_transcript_opens_in_pager.rs` | adopt | `GB-0731-RAW` |
| 489 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_transcript_pager_restore_no_artifacts.rs` | adopt | `GB-0731-RAW` |
| 490 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/mod.rs` | adopt | `GB-0731-RAW` |
| 491 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/misclassified_wheel_flood_does_not_teleport_viewport.rs` | adopt | `GB-0731-RAW` |
| 492 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/mouse_reporting_toggle_sticky_persists_pty.rs` | adopt | `GB-0731-RAW` |
| 493 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/nested_quote_drag_copy_excludes_bars_pty.rs` | adopt | `GB-0731-RAW` |
| 494 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/plan_revise_empty_enter_does_not_approve.rs` | adopt | `GB-0731-RAW` |
| 495 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/prompt_suggestion_ghost_tab_accepts.rs` | adopt | `GB-0731-RAW` |
| 496 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/queued_message_renders_once_not_twice.rs` | adopt | `GB-0731-RAW` |
| 497 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/quote_block_drag_copy_excludes_bars_pty.rs` | adopt | `GB-0731-RAW` |
| 498 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/quote_block_raw_mode_copy_keeps_source_pty.rs` | adopt | `GB-0731-RAW` |
| 499 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/read_tool_header_selection_copies_path_only_pty.rs` | adopt | `GB-0731-RAW` |
| 500 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/reasoning_efforts_menu_renders_and_remaps_on_wire.rs` | adopt | `GB-0731-RAW` |
| 501 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/recap_header_not_in_selection_pty.rs` | adopt | `GB-0731-RAW` |
| 502 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/rename_title_shows_in_prompt_border.rs` | adopt | `GB-0731-RAW` |
| 503 | `D` `crates/codegen/xai-grok-pager/tests/pty_e2e/reparked_wait_repushes_buried_marker.rs` | adopt | `GB-0731-RAW` |
| 504 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/reparked_wait_stays_markerless.rs` | adopt | `GB-0731-RAW` |
| 505 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/requirements_version_failure_exits_2_with_guidance.rs` | adopt | `GB-0731-RAW` |
| 506 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/resize_preserves_scroll_position.rs` | adopt | `GB-0731-RAW` |
| 507 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/reverse_agent_type_mismatch_cursor_to_default.rs` | adopt | `GB-0731-RAW` |
| 508 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/scroll.rs` | adopt | `GB-0731-RAW` |
| 509 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/scroll_debug_hud_env_toggles_overlay.rs` | adopt | `GB-0731-RAW` |
| 510 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/scroll_does_not_crash.rs` | adopt | `GB-0731-RAW` |
| 511 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/send_now_tip_after_mid_turn_queue.rs` | adopt | `GB-0731-RAW` |
| 512 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/shift_tab_plan_nudge_from_always_approve_enters_plan.rs` | adopt | `GB-0731-RAW` |
| 513 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/small_screen_tip_survives_slow_turn.rs` | adopt | `GB-0731-RAW` |
| 514 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/spinner_reappears_after_wait_resumes.rs` | adopt | `GB-0731-RAW` |
| 515 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/storage_upload_parks_on_401_and_drains_after_recovery.rs` | adopt | `GB-0731-RAW` |
| 516 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/stuck_drag_recovers_on_esc_pty.rs` | adopt | `GB-0731-RAW` |
| 517 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/subscription_watch_and_gate_verify_pty.rs` | adopt | `GB-0731-RAW` |
| 518 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/trackpad_flood_does_not_under_travel.rs` | adopt | `GB-0731-RAW` |
| 519 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/undo_tip_resets_each_new_session.rs` | adopt | `GB-0731-RAW` |
| 520 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/undo_tip_seen_count_never_persisted.rs` | adopt | `GB-0731-RAW` |
| 521 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/undo_tip_session_cap_blocks_fourth_show.rs` | adopt | `GB-0731-RAW` |
| 522 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/verb_group_header_drag_copy_pty.rs` | adopt | `GB-0731-RAW` |
| 523 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/wheel_burst_scrolls_viewport_without_frame_amplification.rs` | adopt | `GB-0731-RAW` |
| 524 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/wheel_flood_paints_no_ghost_frames.rs` | adopt | `GB-0731-RAW` |
| 525 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/wheel_overscroll_at_bottom_reengages_follow_mid_stream.rs` | adopt | `GB-0731-RAW` |
| 526 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/wheel_scrolls_viewport_during_streaming_turn.rs` | adopt | `GB-0731-RAW` |
| 527 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/word_select_tip_on_double_click_pty.rs` | adopt | `GB-0731-RAW` |
| 528 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/zero_turn_model_switch_no_modal.rs` | adopt | `GB-0731-RAW` |
| 529 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e_clipboard.rs` | adopt | `GB-0731-RAW` |
| 530 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e_persistence.rs` | adopt | `GB-0731-RAW` |
| 531 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e_queue.rs` | adopt | `GB-0731-RAW` |
| 532 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e_smoke.rs` | adopt | `GB-0731-RAW` |
| 533 | `M` `crates/codegen/xai-grok-pager/tests/pty_xtversion.rs` | adopt | `GB-0731-RAW` |
| 534 | `M` `crates/codegen/xai-grok-pager/tests/settings_e2e.rs` | adopt | `GB-0731-RAW` |
| 535 | `M` `crates/codegen/xai-grok-plugin-marketplace/Cargo.toml` | adopt | `GB-0731-RAW` |
| 536 | `M` `crates/codegen/xai-grok-plugin-marketplace/src/git.rs` | adopt | `GB-0731-RAW` |
| 537 | `M` `crates/codegen/xai-grok-sampler/Cargo.toml` | adopt | `GB-0731-RAW` |
| 538 | `M` `crates/codegen/xai-grok-sampler/src/actor/state.rs` | adopt | `GB-0731-RAW` |
| 539 | `M` `crates/codegen/xai-grok-sampler/src/attribution.rs` | adopt | `GB-0731-RAW` |
| 540 | `M` `crates/codegen/xai-grok-sampler/src/client.rs` | adopt | `GB-0731-RAW` |
| 541 | `M` `crates/codegen/xai-grok-sampler/src/config.rs` | adopt | `GB-0731-RAW` |
| 542 | `M` `crates/codegen/xai-grok-sampler/src/events.rs` | adopt | `GB-0731-RAW` |
| 543 | `M` `crates/codegen/xai-grok-sampler/src/shared_http.rs` | adopt | `GB-0731-RAW` |
| 544 | `M` `crates/codegen/xai-grok-sampler/src/stream/chat_completions.rs` | adopt | `GB-0731-RAW` |
| 545 | `M` `crates/codegen/xai-grok-sampler/src/stream/collect.rs` | adopt | `GB-0731-RAW` |
| 546 | `M` `crates/codegen/xai-grok-sampler/src/stream/messages.rs` | adopt | `GB-0731-RAW` |
| 547 | `M` `crates/codegen/xai-grok-sampler/src/stream/messages_tests.rs` | adopt | `GB-0731-RAW` |
| 548 | `M` `crates/codegen/xai-grok-sampler/src/stream/responses.rs` | adopt | `GB-0731-RAW` |
| 549 | `A` `crates/codegen/xai-grok-sampler/tests/request_query_and_headers.rs` | adopt | `GB-0731-RAW` |
| 550 | `M` `crates/codegen/xai-grok-sampler/tests/test_actor.rs` | adopt | `GB-0731-RAW` |
| 551 | `M` `crates/codegen/xai-grok-sampling-types/Cargo.toml` | adopt | `GB-0731-RAW` |
| 552 | `M` `crates/codegen/xai-grok-sampling-types/src/conversation.rs` | adopt | `GB-0731-RAW` |
| 553 | `A` `crates/codegen/xai-grok-sampling-types/src/conversation/chat_completions.rs` | adopt | `GB-0731-RAW` |
| 554 | `A` `crates/codegen/xai-grok-sampling-types/src/conversation/chat_completions_tests.rs` | adopt | `GB-0731-RAW` |
| 555 | `A` `crates/codegen/xai-grok-sampling-types/src/conversation/messages.rs` | adopt | `GB-0731-RAW` |
| 556 | `A` `crates/codegen/xai-grok-sampling-types/src/conversation/messages_tests.rs` | adopt | `GB-0731-RAW` |
| 557 | `A` `crates/codegen/xai-grok-sampling-types/src/conversation/responses.rs` | adopt | `GB-0731-RAW` |
| 558 | `A` `crates/codegen/xai-grok-sampling-types/src/conversation/responses_tests.rs` | adopt | `GB-0731-RAW` |
| 559 | `A` `crates/codegen/xai-grok-sampling-types/src/conversation/test_support.rs` | adopt | `GB-0731-RAW` |
| 560 | `M` `crates/codegen/xai-grok-sampling-types/src/error.rs` | adopt | `GB-0731-RAW` |
| 561 | `M` `crates/codegen/xai-grok-sampling-types/src/lib.rs` | adopt | `GB-0731-RAW` |
| 562 | `M` `crates/codegen/xai-grok-sampling-types/src/messages.rs` | adopt | `GB-0731-RAW` |
| 563 | `M` `crates/codegen/xai-grok-sampling-types/src/serde_helpers.rs` | adopt | `GB-0731-RAW` |
| 564 | `A` `crates/codegen/xai-grok-sampling-types/src/tool_overrides.rs` | adopt | `GB-0731-RAW` |
| 565 | `M` `crates/codegen/xai-grok-sampling-types/src/types.rs` | adopt | `GB-0731-RAW` |
| 566 | `M` `crates/codegen/xai-grok-sandbox/src/child_net.rs` | adopt | `GB-0731-RAW` |
| 567 | `M` `crates/codegen/xai-grok-sandbox/src/deny/mod.rs` | adopt | `GB-0731-RAW` |
| 568 | `A` `crates/codegen/xai-grok-sandbox/src/hook_write_deny.rs` | adopt | `GB-0731-RAW` |
| 569 | `A` `crates/codegen/xai-grok-sandbox/src/hook_write_deny_tests.rs` | adopt | `GB-0731-RAW` |
| 570 | `M` `crates/codegen/xai-grok-sandbox/src/lib.rs` | adopt | `GB-0731-RAW` |
| 571 | `M` `crates/codegen/xai-grok-sandbox/src/paths.rs` | adopt | `GB-0731-RAW` |
| 572 | `M` `crates/codegen/xai-grok-sandbox/src/profiles.rs` | adopt | `GB-0731-RAW` |
| 573 | `M` `crates/codegen/xai-grok-sandbox/tests/deny_paths_e2e.rs` | adopt | `GB-0731-RAW` |
| 574 | `M` `crates/codegen/xai-grok-shared/src/clipboard.rs` | adopt | `GB-0731-RAW` |
| 575 | `M` `crates/codegen/xai-grok-shared/src/ui_config.rs` | adopt | `GB-0731-RAW` |
| 576 | `M` `crates/codegen/xai-grok-shell-base/src/cpu_profile.rs` | adopt | `GB-0731-RAW` |
| 577 | `M` `crates/codegen/xai-grok-shell-base/src/env.rs` | adopt | `GB-0731-RAW` |
| 578 | `M` `crates/codegen/xai-grok-shell-base/src/util/mod.rs` | adopt | `GB-0731-RAW` |
| 579 | `M` `crates/codegen/xai-grok-shell-session-support/src/managed_mcp.rs` | adopt | `GB-0731-RAW` |
| 580 | `M` `crates/codegen/xai-grok-shell/CHANGELOG.md` | adopt | `GB-0731-RAW` |
| 581 | `M` `crates/codegen/xai-grok-shell/Cargo.toml` | adopt | `GB-0731-RAW` |
| 582 | `M` `crates/codegen/xai-grok-shell/README.md` | adopt | `GB-0731-RAW` |
| 583 | `A` `crates/codegen/xai-grok-shell/benches/fork_copy.rs` | adopt | `GB-0731-RAW` |
| 584 | `M` `crates/codegen/xai-grok-shell/changelogs/0.2.108.json` | adopt | `GB-0731-RAW` |
| 585 | `M` `crates/codegen/xai-grok-shell/changelogs/0.2.108.md` | adopt | `GB-0731-RAW` |
| 586 | `M` `crates/codegen/xai-grok-shell/changelogs/0.2.109.json` | adopt | `GB-0731-RAW` |
| 587 | `M` `crates/codegen/xai-grok-shell/changelogs/0.2.109.md` | adopt | `GB-0731-RAW` |
| 588 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.110.json` | adopt | `GB-0731-RAW` |
| 589 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.110.md` | adopt | `GB-0731-RAW` |
| 590 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.111.json` | adopt | `GB-0731-RAW` |
| 591 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.111.md` | adopt | `GB-0731-RAW` |
| 592 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.112.json` | adopt | `GB-0731-RAW` |
| 593 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.112.md` | adopt | `GB-0731-RAW` |
| 594 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.113.json` | adopt | `GB-0731-RAW` |
| 595 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.113.md` | adopt | `GB-0731-RAW` |
| 596 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.114.json` | adopt | `GB-0731-RAW` |
| 597 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.114.md` | adopt | `GB-0731-RAW` |
| 598 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.115.json` | adopt | `GB-0731-RAW` |
| 599 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.115.md` | adopt | `GB-0731-RAW` |
| 600 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.116.json` | adopt | `GB-0731-RAW` |
| 601 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.116.md` | adopt | `GB-0731-RAW` |
| 602 | `M` `crates/codegen/xai-grok-shell/src/agent/activity.rs` | adopt | `GB-0731-RAW` |
| 603 | `M` `crates/codegen/xai-grok-shell/src/agent/app.rs` | adopt | `GB-0731-RAW` |
| 604 | `M` `crates/codegen/xai-grok-shell/src/agent/chat_modes.rs` | adopt | `GB-0731-RAW` |
| 605 | `M` `crates/codegen/xai-grok-shell/src/agent/config.rs` | adopt | `GB-0731-RAW` |
| 606 | `M` `crates/codegen/xai-grok-shell/src/agent/config_model_override_parse.rs` | adopt | `GB-0731-RAW` |
| 607 | `M` `crates/codegen/xai-grok-shell/src/agent/feedback_client.rs` | adopt | `GB-0731-RAW` |
| 608 | `M` `crates/codegen/xai-grok-shell/src/agent/folder_trust.rs` | adopt | `GB-0731-RAW` |
| 609 | `M` `crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs` | adopt | `GB-0731-RAW` |
| 610 | `M` `crates/codegen/xai-grok-shell/src/agent/handlers/session.rs` | adopt | `GB-0731-RAW` |
| 611 | `M` `crates/codegen/xai-grok-shell/src/agent/init.rs` | adopt | `GB-0731-RAW` |
| 612 | `M` `crates/codegen/xai-grok-shell/src/agent/mod.rs` | adopt | `GB-0731-RAW` |
| 613 | `M` `crates/codegen/xai-grok-shell/src/agent/model_providers.rs` | adopt | `GB-0731-RAW` |
| 614 | `M` `crates/codegen/xai-grok-shell/src/agent/models.rs` | adopt | `GB-0731-RAW` |
| 615 | `A` `crates/codegen/xai-grok-shell/src/agent/models/cache.rs` | adopt | `GB-0731-RAW` |
| 616 | `A` `crates/codegen/xai-grok-shell/src/agent/models/endpoint.rs` | adopt | `GB-0731-RAW` |
| 617 | `A` `crates/codegen/xai-grok-shell/src/agent/models/fetch.rs` | adopt | `GB-0731-RAW` |
| 618 | `A` `crates/codegen/xai-grok-shell/src/agent/models/resolution.rs` | adopt | `GB-0731-RAW` |
| 619 | `A` `crates/codegen/xai-grok-shell/src/agent/models/tests.rs` | adopt | `GB-0731-RAW` |
| 620 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/acp_agent.rs` | adopt | `GB-0731-RAW` |
| 621 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs` | adopt | `GB-0731-RAW` |
| 622 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/code_nav.rs` | adopt | `GB-0731-RAW` |
| 623 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/folder_trust_prompt.rs` | adopt | `GB-0731-RAW` |
| 624 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/mod.rs` | adopt | `GB-0731-RAW` |
| 625 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/prompt_response_meta_tests.rs` | adopt | `GB-0731-RAW` |
| 626 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/session_lifecycle.rs` | adopt | `GB-0731-RAW` |
| 627 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/subagent_coordinator.rs` | adopt | `GB-0731-RAW` |
| 628 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/tests.rs` | adopt | `GB-0731-RAW` |
| 629 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/tests/dhat_soak.rs` | adopt | `GB-0731-RAW` |
| 630 | `A` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/tests/process_scope_reclaim.rs` | adopt | `GB-0731-RAW` |
| 631 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/tests/subagent_spawn_context_tests.rs` | adopt | `GB-0731-RAW` |
| 632 | `A` `crates/codegen/xai-grok-shell/src/agent/otel_gate.rs` | adopt | `GB-0731-RAW` |
| 633 | `M` `crates/codegen/xai-grok-shell/src/agent/relay.rs` | adopt | `GB-0731-RAW` |
| 634 | `M` `crates/codegen/xai-grok-shell/src/agent/restore_code.rs` | adopt | `GB-0731-RAW` |
| 635 | `M` `crates/codegen/xai-grok-shell/src/agent/server.rs` | adopt | `GB-0731-RAW` |
| 636 | `M` `crates/codegen/xai-grok-shell/src/agent/session_registry_client.rs` | adopt | `GB-0731-RAW` |
| 637 | `D` `crates/codegen/xai-grok-shell/src/agent/subagent/coordinator_lifecycle.rs` | adopt | `GB-0731-RAW` |
| 638 | `D` `crates/codegen/xai-grok-shell/src/agent/subagent/coordinator_query.rs` | adopt | `GB-0731-RAW` |
| 639 | `M` `crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs` | adopt | `GB-0731-RAW` |
| 640 | `M` `crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs` | adopt | `GB-0731-RAW` |
| 641 | `M` `crates/codegen/xai-grok-shell/src/agent/subagent/tests/mod.rs` | adopt | `GB-0731-RAW` |
| 642 | `M` `crates/codegen/xai-grok-shell/src/agent/subagent/tests/rest.rs` | adopt | `GB-0731-RAW` |
| 643 | `M` `crates/codegen/xai-grok-shell/src/agent/subscription_check.rs` | adopt | `GB-0731-RAW` |
| 644 | `M` `crates/codegen/xai-grok-shell/src/auth/attribution.rs` | adopt | `GB-0731-RAW` |
| 645 | `M` `crates/codegen/xai-grok-shell/src/auth/auth_provider.rs` | adopt | `GB-0731-RAW` |
| 646 | `M` `crates/codegen/xai-grok-shell/src/auth/auth_provider_tests.rs` | adopt | `GB-0731-RAW` |
| 647 | `M` `crates/codegen/xai-grok-shell/src/auth/credential_provider.rs` | adopt | `GB-0731-RAW` |
| 648 | `M` `crates/codegen/xai-grok-shell/src/auth/error.rs` | adopt | `GB-0731-RAW` |
| 649 | `M` `crates/codegen/xai-grok-shell/src/auth/external_auth.rs` | adopt | `GB-0731-RAW` |
| 650 | `M` `crates/codegen/xai-grok-shell/src/auth/flow.rs` | adopt | `GB-0731-RAW` |
| 651 | `M` `crates/codegen/xai-grok-shell/src/auth/manager.rs` | adopt | `GB-0731-RAW` |
| 652 | `M` `crates/codegen/xai-grok-shell/src/auth/manager/enrichment.rs` | adopt | `GB-0731-RAW` |
| 653 | `M` `crates/codegen/xai-grok-shell/src/auth/manager/lock.rs` | adopt | `GB-0731-RAW` |
| 654 | `M` `crates/codegen/xai-grok-shell/src/auth/manager/sleep_gate.rs` | adopt | `GB-0731-RAW` |
| 655 | `M` `crates/codegen/xai-grok-shell/src/auth/manager_tests.rs` | adopt | `GB-0731-RAW` |
| 656 | `M` `crates/codegen/xai-grok-shell/src/auth/mod.rs` | adopt | `GB-0731-RAW` |
| 657 | `M` `crates/codegen/xai-grok-shell/src/auth/oidc/protocol.rs` | adopt | `GB-0731-RAW` |
| 658 | `M` `crates/codegen/xai-grok-shell/src/auth/oidc/refresh.rs` | adopt | `GB-0731-RAW` |
| 659 | `M` `crates/codegen/xai-grok-shell/src/auth/refresh/auth_backend_contract_tests.rs` | adopt | `GB-0731-RAW` |
| 660 | `M` `crates/codegen/xai-grok-shell/src/auth/refresh/mod.rs` | adopt | `GB-0731-RAW` |
| 661 | `M` `crates/codegen/xai-grok-shell/src/auth/refresh/oidc_refresher.rs` | adopt | `GB-0731-RAW` |
| 662 | `M` `crates/codegen/xai-grok-shell/src/auth/refresh/oidc_refresher_tests.rs` | adopt | `GB-0731-RAW` |
| 663 | `M` `crates/codegen/xai-grok-shell/src/auth/storage.rs` | adopt | `GB-0731-RAW` |
| 664 | `M` `crates/codegen/xai-grok-shell/src/claude_import.rs` | adopt | `GB-0731-RAW` |
| 665 | `M` `crates/codegen/xai-grok-shell/src/config/mod.rs` | adopt | `GB-0731-RAW` |
| 666 | `M` `crates/codegen/xai-grok-shell/src/config/reloader.rs` | adopt | `GB-0731-RAW` |
| 667 | `M` `crates/codegen/xai-grok-shell/src/config/tests.rs` | adopt | `GB-0731-RAW` |
| 668 | `M` `crates/codegen/xai-grok-shell/src/extensions/auth.rs` | adopt | `GB-0731-RAW` |
| 669 | `M` `crates/codegen/xai-grok-shell/src/extensions/bundle.rs` | adopt | `GB-0731-RAW` |
| 670 | `M` `crates/codegen/xai-grok-shell/src/extensions/debug.rs` | adopt | `GB-0731-RAW` |
| 671 | `M` `crates/codegen/xai-grok-shell/src/extensions/git.rs` | adopt | `GB-0731-RAW` |
| 672 | `M` `crates/codegen/xai-grok-shell/src/extensions/hooks.rs` | adopt | `GB-0731-RAW` |
| 673 | `M` `crates/codegen/xai-grok-shell/src/extensions/marketplace.rs` | adopt | `GB-0731-RAW` |
| 674 | `M` `crates/codegen/xai-grok-shell/src/extensions/mcp.rs` | adopt | `GB-0731-RAW` |
| 675 | `M` `crates/codegen/xai-grok-shell/src/extensions/notification.rs` | adopt | `GB-0731-RAW` |
| 676 | `M` `crates/codegen/xai-grok-shell/src/extensions/session_admin.rs` | adopt | `GB-0731-RAW` |
| 677 | `M` `crates/codegen/xai-grok-shell/src/extensions/session_updates.rs` | adopt | `GB-0731-RAW` |
| 678 | `M` `crates/codegen/xai-grok-shell/src/extensions/skills.rs` | adopt | `GB-0731-RAW` |
| 679 | `M` `crates/codegen/xai-grok-shell/src/extensions/task.rs` | adopt | `GB-0731-RAW` |
| 680 | `M` `crates/codegen/xai-grok-shell/src/extensions/usage.rs` | adopt | `GB-0731-RAW` |
| 681 | `M` `crates/codegen/xai-grok-shell/src/inspect/mod.rs` | adopt | `GB-0731-RAW` |
| 682 | `M` `crates/codegen/xai-grok-shell/src/leader/client.rs` | adopt | `GB-0731-RAW` |
| 683 | `A` `crates/codegen/xai-grok-shell/src/leader/in_process.rs` | adopt | `GB-0731-RAW` |
| 684 | `M` `crates/codegen/xai-grok-shell/src/leader/lock.rs` | adopt | `GB-0731-RAW` |
| 685 | `M` `crates/codegen/xai-grok-shell/src/leader/mod.rs` | adopt | `GB-0731-RAW` |
| 686 | `M` `crates/codegen/xai-grok-shell/src/leader/protocol.rs` | adopt | `GB-0731-RAW` |
| 687 | `M` `crates/codegen/xai-grok-shell/src/leader/server.rs` | adopt | `GB-0731-RAW` |
| 688 | `M` `crates/codegen/xai-grok-shell/src/leader/test_support.rs` | adopt | `GB-0731-RAW` |
| 689 | `M` `crates/codegen/xai-grok-shell/src/managed_config.rs` | adopt | `GB-0731-RAW` |
| 690 | `M` `crates/codegen/xai-grok-shell/src/mcp_doctor.rs` | adopt | `GB-0731-RAW` |
| 691 | `M` `crates/codegen/xai-grok-shell/src/remote/client.rs` | adopt | `GB-0731-RAW` |
| 692 | `M` `crates/codegen/xai-grok-shell/src/remote/mod.rs` | adopt | `GB-0731-RAW` |
| 693 | `A` `crates/codegen/xai-grok-shell/src/remote/skills_client.rs` | adopt | `GB-0731-RAW` |
| 694 | `M` `crates/codegen/xai-grok-shell/src/sampling/error.rs` | adopt | `GB-0731-RAW` |
| 695 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session.rs` | adopt | `GB-0731-RAW` |
| 696 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/goal.rs` | adopt | `GB-0731-RAW` |
| 697 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/goal_support.rs` | adopt | `GB-0731-RAW` |
| 698 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/hook_dispatch.rs` | adopt | `GB-0731-RAW` |
| 699 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/hooks_plugins.rs` | adopt | `GB-0731-RAW` |
| 700 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/interjection.rs` | adopt | `GB-0731-RAW` |
| 701 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/mcp.rs` | adopt | `GB-0731-RAW` |
| 702 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/model_switch.rs` | adopt | `GB-0731-RAW` |
| 703 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/notification_drain.rs` | adopt | `GB-0731-RAW` |
| 704 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/prompt_build.rs` | adopt | `GB-0731-RAW` |
| 705 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/prompt_queue.rs` | adopt | `GB-0731-RAW` |
| 706 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/recap.rs` | adopt | `GB-0731-RAW` |
| 707 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/reminders.rs` | adopt | `GB-0731-RAW` |
| 708 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/run_loop.rs` | adopt | `GB-0731-RAW` |
| 709 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs` | adopt | `GB-0731-RAW` |
| 710 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/session_mode.rs` | adopt | `GB-0731-RAW` |
| 711 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/session_setup.rs` | adopt | `GB-0731-RAW` |
| 712 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/spawn.rs` | adopt | `GB-0731-RAW` |
| 713 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/stop_gate.rs` | adopt | `GB-0731-RAW` |
| 714 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/tasks_cancel.rs` | adopt | `GB-0731-RAW` |
| 715 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/tool_calls.rs` | adopt | `GB-0731-RAW` |
| 716 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/tool_dispatch.rs` | adopt | `GB-0731-RAW` |
| 717 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/turn.rs` | adopt | `GB-0731-RAW` |
| 718 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/turn_end.rs` | adopt | `GB-0731-RAW` |
| 719 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/types.rs` | adopt | `GB-0731-RAW` |
| 720 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/updates.rs` | adopt | `GB-0731-RAW` |
| 721 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/workflow.rs` | adopt | `GB-0731-RAW` |
| 722 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/auth_error_no_retry_tests.rs` | adopt | `GB-0731-RAW` |
| 723 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/auto_wake_suppression_tests.rs` | adopt | `GB-0731-RAW` |
| 724 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/cancel_running_task_tests.rs` | adopt | `GB-0731-RAW` |
| 725 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/client_hooks_tests.rs` | adopt | `GB-0731-RAW` |
| 726 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/fs_injection_regression_tests.rs` | adopt | `GB-0731-RAW` |
| 727 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/idle_resume_tests.rs` | adopt | `GB-0731-RAW` |
| 728 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/inline_auto_compact_flow_tests.rs` | adopt | `GB-0731-RAW` |
| 729 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/laziness/laziness_integration_tests.rs` | adopt | `GB-0731-RAW` |
| 730 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/memory_config_tests.rs` | adopt | `GB-0731-RAW` |
| 731 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/observability_bridge_mapping_tests.rs` | adopt | `GB-0731-RAW` |
| 732 | `A` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/plan_exit_batch_barrier_tests.rs` | adopt | `GB-0731-RAW` |
| 733 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/plan_mode_edit_gate_tests.rs` | adopt | `GB-0731-RAW` |
| 734 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/prompt_mode_transition_tests.rs` | adopt | `GB-0731-RAW` |
| 735 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/prompt_queue_actor_tests.rs` | adopt | `GB-0731-RAW` |
| 736 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/recap_display_only_tests.rs` | adopt | `GB-0731-RAW` |
| 737 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/record_response_token_usage_tests.rs` | adopt | `GB-0731-RAW` |
| 738 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/reminder_policy_tests.rs` | adopt | `GB-0731-RAW` |
| 739 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/replay_buffer_send_update_tests.rs` | adopt | `GB-0731-RAW` |
| 740 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/rewind_cross_compaction_tests.rs` | adopt | `GB-0731-RAW` |
| 741 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/subagent_usage_fold_tests.rs` | adopt | `GB-0731-RAW` |
| 742 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/support.rs` | adopt | `GB-0731-RAW` |
| 743 | `A` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/turn/chat_history_integrity_tests.rs` | adopt | `GB-0731-RAW` |
| 744 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/turn_completion_emit_tests.rs` | adopt | `GB-0731-RAW` |
| 745 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/usage_categories_tests.rs` | adopt | `GB-0731-RAW` |
| 746 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/web_search_e2e_tests.rs` | adopt | `GB-0731-RAW` |
| 747 | `M` `crates/codegen/xai-grok-shell/src/session/agent_rebuild.rs` | adopt | `GB-0731-RAW` |
| 748 | `M` `crates/codegen/xai-grok-shell/src/session/commands.rs` | adopt | `GB-0731-RAW` |
| 749 | `M` `crates/codegen/xai-grok-shell/src/session/compaction.rs` | adopt | `GB-0731-RAW` |
| 750 | `M` `crates/codegen/xai-grok-shell/src/session/compaction_config.rs` | adopt | `GB-0731-RAW` |
| 751 | `M` `crates/codegen/xai-grok-shell/src/session/events.rs` | adopt | `GB-0731-RAW` |
| 752 | `M` `crates/codegen/xai-grok-shell/src/session/goal_classifier.rs` | adopt | `GB-0731-RAW` |
| 753 | `M` `crates/codegen/xai-grok-shell/src/session/goal_planner.rs` | adopt | `GB-0731-RAW` |
| 754 | `M` `crates/codegen/xai-grok-shell/src/session/goal_strategist.rs` | adopt | `GB-0731-RAW` |
| 755 | `M` `crates/codegen/xai-grok-shell/src/session/goal_summarizer.rs` | adopt | `GB-0731-RAW` |
| 756 | `M` `crates/codegen/xai-grok-shell/src/session/handle.rs` | adopt | `GB-0731-RAW` |
| 757 | `M` `crates/codegen/xai-grok-shell/src/session/helpers/session_compact.rs` | adopt | `GB-0731-RAW` |
| 758 | `M` `crates/codegen/xai-grok-shell/src/session/image_normalize.rs` | adopt | `GB-0731-RAW` |
| 759 | `M` `crates/codegen/xai-grok-shell/src/session/managed_mcp.rs` | adopt | `GB-0731-RAW` |
| 760 | `M` `crates/codegen/xai-grok-shell/src/session/mcp_restart.rs` | adopt | `GB-0731-RAW` |
| 761 | `M` `crates/codegen/xai-grok-shell/src/session/mcp_servers.rs` | adopt | `GB-0731-RAW` |
| 762 | `M` `crates/codegen/xai-grok-shell/src/session/merge.rs` | adopt | `GB-0731-RAW` |
| 763 | `M` `crates/codegen/xai-grok-shell/src/session/mod.rs` | adopt | `GB-0731-RAW` |
| 764 | `M` `crates/codegen/xai-grok-shell/src/session/persistence.rs` | adopt | `GB-0731-RAW` |
| 765 | `M` `crates/codegen/xai-grok-shell/src/session/persistence_tests.rs` | adopt | `GB-0731-RAW` |
| 766 | `M` `crates/codegen/xai-grok-shell/src/session/plan_mode.rs` | adopt | `GB-0731-RAW` |
| 767 | `M` `crates/codegen/xai-grok-shell/src/session/prompt_parser.rs` | adopt | `GB-0731-RAW` |
| 768 | `M` `crates/codegen/xai-grok-shell/src/session/signals.rs` | adopt | `GB-0731-RAW` |
| 769 | `M` `crates/codegen/xai-grok-shell/src/session/slash_commands.rs` | adopt | `GB-0731-RAW` |
| 770 | `M` `crates/codegen/xai-grok-shell/src/session/storage/jsonl/mod.rs` | adopt | `GB-0731-RAW` |
| 771 | `M` `crates/codegen/xai-grok-shell/src/session/storage/jsonl/tests.rs` | adopt | `GB-0731-RAW` |
| 772 | `M` `crates/codegen/xai-grok-shell/src/session/storage/mod.rs` | adopt | `GB-0731-RAW` |
| 773 | `M` `crates/codegen/xai-grok-shell/src/session/storage/relocation/fs.rs` | adopt | `GB-0731-RAW` |
| 774 | `M` `crates/codegen/xai-grok-shell/src/session/storage/relocation/mod.rs` | adopt | `GB-0731-RAW` |
| 775 | `M` `crates/codegen/xai-grok-shell/src/session/storage/relocation/tests.rs` | adopt | `GB-0731-RAW` |
| 776 | `A` `crates/codegen/xai-grok-shell/src/session/storage/relocation/view.rs` | adopt | `GB-0731-RAW` |
| 777 | `M` `crates/codegen/xai-grok-shell/src/session/storage/search.rs` | adopt | `GB-0731-RAW` |
| 778 | `M` `crates/codegen/xai-grok-shell/src/session/storage/search_fts.rs` | adopt | `GB-0731-RAW` |
| 779 | `A` `crates/codegen/xai-grok-shell/src/session/storage/search_recovery.rs` | adopt | `GB-0731-RAW` |
| 780 | `M` `crates/codegen/xai-grok-shell/src/session/storage/search_remote_sync.rs` | adopt | `GB-0731-RAW` |
| 781 | `M` `crates/codegen/xai-grok-shell/src/session/summary.rs` | adopt | `GB-0731-RAW` |
| 782 | `M` `crates/codegen/xai-grok-shell/src/session/telemetry.rs` | adopt | `GB-0731-RAW` |
| 783 | `A` `crates/codegen/xai-grok-shell/src/session/testkit/e2e.rs` | adopt | `GB-0731-RAW` |
| 784 | `A` `crates/codegen/xai-grok-shell/src/session/testkit/mod.rs` | adopt | `GB-0731-RAW` |
| 785 | `A` `crates/codegen/xai-grok-shell/src/session/testkit/synth/bench.rs` | adopt | `GB-0731-RAW` |
| 786 | `A` `crates/codegen/xai-grok-shell/src/session/testkit/synth/mod.rs` | adopt | `GB-0731-RAW` |
| 787 | `A` `crates/codegen/xai-grok-shell/src/session/testkit/synth/replay.rs` | adopt | `GB-0731-RAW` |
| 788 | `M` `crates/codegen/xai-grok-shell/src/session/unified_list/mod.rs` | adopt | `GB-0731-RAW` |
| 789 | `M` `crates/codegen/xai-grok-shell/src/session/unified_list/row.rs` | adopt | `GB-0731-RAW` |
| 790 | `M` `crates/codegen/xai-grok-shell/src/session/user_message.rs` | adopt | `GB-0731-RAW` |
| 791 | `M` `crates/codegen/xai-grok-shell/src/session/wire_tags.rs` | adopt | `GB-0731-RAW` |
| 792 | `M` `crates/codegen/xai-grok-shell/src/session/workflow/host_service.rs` | adopt | `GB-0731-RAW` |
| 793 | `M` `crates/codegen/xai-grok-shell/src/session/workflow/manager.rs` | adopt | `GB-0731-RAW` |
| 794 | `M` `crates/codegen/xai-grok-shell/src/session/workflow/tracker.rs` | adopt | `GB-0731-RAW` |
| 795 | `M` `crates/codegen/xai-grok-shell/src/session/worktree.rs` | adopt | `GB-0731-RAW` |
| 796 | `M` `crates/codegen/xai-grok-shell/src/session/worktree_pool.rs` | adopt | `GB-0731-RAW` |
| 797 | `M` `crates/codegen/xai-grok-shell/src/terminal/adapter.rs` | adopt | `GB-0731-RAW` |
| 798 | `A` `crates/codegen/xai-grok-shell/src/terminal/adapter_tests.rs` | adopt | `GB-0731-RAW` |
| 799 | `A` `crates/codegen/xai-grok-shell/src/terminal/exit_watcher.rs` | adopt | `GB-0731-RAW` |
| 800 | `M` `crates/codegen/xai-grok-shell/src/terminal/local_terminal.rs` | adopt | `GB-0731-RAW` |
| 801 | `M` `crates/codegen/xai-grok-shell/src/terminal/mod.rs` | adopt | `GB-0731-RAW` |
| 802 | `A` `crates/codegen/xai-grok-shell/src/terminal/output_recorder.rs` | adopt | `GB-0731-RAW` |
| 803 | `M` `crates/codegen/xai-grok-shell/src/terminal/pty_session.rs` | adopt | `GB-0731-RAW` |
| 804 | `M` `crates/codegen/xai-grok-shell/src/test_support/lsp_runtime.rs` | adopt | `GB-0731-RAW` |
| 805 | `M` `crates/codegen/xai-grok-shell/src/test_support/mod.rs` | adopt | `GB-0731-RAW` |
| 806 | `M` `crates/codegen/xai-grok-shell/src/tools/config.rs` | adopt | `GB-0731-RAW` |
| 807 | `M` `crates/codegen/xai-grok-shell/src/tools/notification_bridge.rs` | adopt | `GB-0731-RAW` |
| 808 | `M` `crates/codegen/xai-grok-shell/src/tools/tool_context.rs` | adopt | `GB-0731-RAW` |
| 809 | `M` `crates/codegen/xai-grok-shell/src/upload/gcs.rs` | adopt | `GB-0731-RAW` |
| 810 | `M` `crates/codegen/xai-grok-shell/src/upload/manifest.rs` | adopt | `GB-0731-RAW` |
| 811 | `M` `crates/codegen/xai-grok-shell/src/upload/trace.rs` | adopt | `GB-0731-RAW` |
| 812 | `M` `crates/codegen/xai-grok-shell/src/upload/turn.rs` | adopt | `GB-0731-RAW` |
| 813 | `M` `crates/codegen/xai-grok-shell/src/util/config/campaigns.rs` | adopt | `GB-0731-RAW` |
| 814 | `M` `crates/codegen/xai-grok-shell/src/util/config/load.rs` | adopt | `GB-0731-RAW` |
| 815 | `M` `crates/codegen/xai-grok-shell/src/util/config/mcp.rs` | adopt | `GB-0731-RAW` |
| 816 | `M` `crates/codegen/xai-grok-shell/src/util/config/mod.rs` | adopt | `GB-0731-RAW` |
| 817 | `M` `crates/codegen/xai-grok-shell/src/util/config/persist.rs` | adopt | `GB-0731-RAW` |
| 818 | `M` `crates/codegen/xai-grok-shell/src/util/config/resolve/auto_mode.rs` | adopt | `GB-0731-RAW` |
| 819 | `M` `crates/codegen/xai-grok-shell/src/util/config/resolve/features.rs` | adopt | `GB-0731-RAW` |
| 820 | `M` `crates/codegen/xai-grok-shell/src/util/config/resolve/toolset.rs` | adopt | `GB-0731-RAW` |
| 821 | `M` `crates/codegen/xai-grok-shell/src/util/config/resolve/version.rs` | adopt | `GB-0731-RAW` |
| 822 | `M` `crates/codegen/xai-grok-shell/src/util/config/settings_writes.rs` | adopt | `GB-0731-RAW` |
| 823 | `M` `crates/codegen/xai-grok-shell/src/util/config/tips.rs` | adopt | `GB-0731-RAW` |
| 824 | `M` `crates/codegen/xai-grok-shell/src/util/grok_auth_credentials.rs` | adopt | `GB-0731-RAW` |
| 825 | `M` `crates/codegen/xai-grok-shell/src/util/hooks.rs` | adopt | `GB-0731-RAW` |
| 826 | `A` `crates/codegen/xai-grok-shell/src/util/limits.rs` | adopt | `GB-0731-RAW` |
| 827 | `M` `crates/codegen/xai-grok-shell/src/util/mod.rs` | adopt | `GB-0731-RAW` |
| 828 | `M` `crates/codegen/xai-grok-shell/src/util/subprocess.rs` | adopt | `GB-0731-RAW` |
| 829 | `M` `crates/codegen/xai-grok-shell/src/util/user_identity.rs` | adopt | `GB-0731-RAW` |
| 830 | `M` `crates/codegen/xai-grok-shell/tests/common/mod.rs` | adopt | `GB-0731-RAW` |
| 831 | `A` `crates/codegen/xai-grok-shell/tests/session_fork_replay_memory.rs` | adopt | `GB-0731-RAW` |
| 832 | `M` `crates/codegen/xai-grok-shell/tests/session_load_perf.rs` | adopt | `GB-0731-RAW` |
| 833 | `D` `crates/codegen/xai-grok-shell/tests/team_managed_config.rs` | adopt | `GB-0731-RAW` |
| 834 | `M` `crates/codegen/xai-grok-shell/tests/test_agent_type_invariant.rs` | adopt | `GB-0731-RAW` |
| 835 | `A` `crates/codegen/xai-grok-shell/tests/test_auth_provider_command_e2e.rs` | adopt | `GB-0731-RAW` |
| 836 | `M` `crates/codegen/xai-grok-shell/tests/test_auth_provider_e2e.rs` | adopt | `GB-0731-RAW` |
| 837 | `M` `crates/codegen/xai-grok-shell/tests/test_built_binary_e2e.rs` | adopt | `GB-0731-RAW` |
| 838 | `M` `crates/codegen/xai-grok-shell/tests/test_debug_logging.rs` | adopt | `GB-0731-RAW` |
| 839 | `M` `crates/codegen/xai-grok-shell/tests/test_doom_loop_recovery.rs` | adopt | `GB-0731-RAW` |
| 840 | `M` `crates/codegen/xai-grok-shell/tests/test_global_extra_headers_e2e.rs` | adopt | `GB-0731-RAW` |
| 841 | `M` `crates/codegen/xai-grok-shell/tests/test_leader_death_repro.rs` | adopt | `GB-0731-RAW` |
| 842 | `A` `crates/codegen/xai-grok-shell/tests/test_leader_sandbox_confinement.rs` | adopt | `GB-0731-RAW` |
| 843 | `M` `crates/codegen/xai-grok-shell/tests/test_leader_soak.rs` | adopt | `GB-0731-RAW` |
| 844 | `M` `crates/codegen/xai-grok-shell/tests/test_leader_stdio_integration.rs` | adopt | `GB-0731-RAW` |
| 845 | `M` `crates/codegen/xai-grok-shell/tests/test_leader_version_skew.rs` | adopt | `GB-0731-RAW` |
| 846 | `A` `crates/codegen/xai-grok-shell/tests/test_nonblocking_startup.rs` | adopt | `GB-0731-RAW` |
| 847 | `A` `crates/codegen/xai-grok-shell/tests/test_nonblocking_startup_offline.rs` | adopt | `GB-0731-RAW` |
| 848 | `M` `crates/codegen/xai-grok-shell/tests/test_refusal_stop_reason.rs` | adopt | `GB-0731-RAW` |
| 849 | `M` `crates/codegen/xai-grok-shell/tests/test_registry_churn.rs` | adopt | `GB-0731-RAW` |
| 850 | `A` `crates/codegen/xai-grok-shell/tests/test_session_end_hook_e2e.rs` | adopt | `GB-0731-RAW` |
| 851 | `A` `crates/codegen/xai-grok-shell/tests/test_session_load_memory.rs` | adopt | `GB-0731-RAW` |
| 852 | `M` `crates/codegen/xai-grok-shell/tests/test_settings_refresh.rs` | adopt | `GB-0731-RAW` |
| 853 | `M` `crates/codegen/xai-grok-shell/tests/test_stop_hook_e2e.rs` | adopt | `GB-0731-RAW` |
| 854 | `M` `crates/codegen/xai-grok-shell/tests/test_subagent_orphan_reconcile.rs` | adopt | `GB-0731-RAW` |
| 855 | `M` `crates/codegen/xai-grok-shell/tests/test_summary_reasoning_effort.rs` | adopt | `GB-0731-RAW` |
| 856 | `A` `crates/codegen/xai-grok-shell/tests/test_tool_dispatch_duration_smoke.rs` | adopt | `GB-0731-RAW` |
| 857 | `M` `crates/codegen/xai-grok-shell/tests/test_trusted_local_plugin_refresh_e2e.rs` | adopt | `GB-0731-RAW` |
| 858 | `M` `crates/codegen/xai-grok-shell/tests/test_vendor_compat.rs` | adopt | `GB-0731-RAW` |
| 859 | `A` `crates/codegen/xai-grok-shell/tests/testkit_synth_roundtrip.rs` | adopt | `GB-0731-RAW` |
| 860 | `M` `crates/codegen/xai-grok-subagent-resolution/Cargo.toml` | adopt | `GB-0731-RAW` |
| 861 | `A` `crates/codegen/xai-grok-subagent-resolution/src/definition.rs` | adopt | `GB-0731-RAW` |
| 862 | `M` `crates/codegen/xai-grok-subagent-resolution/src/lib.rs` | adopt | `GB-0731-RAW` |
| 863 | `M` `crates/codegen/xai-grok-subagent-resolution/src/types.rs` | adopt | `GB-0731-RAW` |
| 864 | `M` `crates/codegen/xai-grok-telemetry/Cargo.toml` | adopt | `GB-0731-RAW` |
| 865 | `M` `crates/codegen/xai-grok-telemetry/src/client.rs` | adopt | `GB-0731-RAW` |
| 866 | `M` `crates/codegen/xai-grok-telemetry/src/config.rs` | adopt | `GB-0731-RAW` |
| 867 | `M` `crates/codegen/xai-grok-telemetry/src/events.rs` | adopt | `GB-0731-RAW` |
| 868 | `M` `crates/codegen/xai-grok-telemetry/src/external/mod.rs` | adopt | `GB-0731-RAW` |
| 869 | `M` `crates/codegen/xai-grok-telemetry/src/external/providers.rs` | adopt | `GB-0731-RAW` |
| 870 | `M` `crates/codegen/xai-grok-telemetry/src/external/schema.rs` | adopt | `GB-0731-RAW` |
| 871 | `M` `crates/codegen/xai-grok-telemetry/src/external/tests.rs` | adopt | `GB-0731-RAW` |
| 872 | `M` `crates/codegen/xai-grok-telemetry/src/otel_layer/mod.rs` | adopt | `GB-0731-RAW` |
| 873 | `M` `crates/codegen/xai-grok-telemetry/src/otel_layer/redact.rs` | adopt | `GB-0731-RAW` |
| 874 | `M` `crates/codegen/xai-grok-telemetry/src/otlp_http.rs` | adopt | `GB-0731-RAW` |
| 875 | `M` `crates/codegen/xai-grok-telemetry/src/unified_log.rs` | adopt | `GB-0731-RAW` |
| 876 | `M` `crates/codegen/xai-grok-test-support/Cargo.toml` | adopt | `GB-0731-RAW` |
| 877 | `M` `crates/codegen/xai-grok-test-support/README.md` | adopt | `GB-0731-RAW` |
| 878 | `M` `crates/codegen/xai-grok-test-support/src/acp_client.rs` | adopt | `GB-0731-RAW` |
| 879 | `M` `crates/codegen/xai-grok-test-support/src/env.rs` | adopt | `GB-0731-RAW` |
| 880 | `M` `crates/codegen/xai-grok-test-support/src/headless.rs` | adopt | `GB-0731-RAW` |
| 881 | `M` `crates/codegen/xai-grok-test-support/src/leader.rs` | adopt | `GB-0731-RAW` |
| 882 | `M` `crates/codegen/xai-grok-test-support/src/lib.rs` | adopt | `GB-0731-RAW` |
| 883 | `M` `crates/codegen/xai-grok-test-support/src/mock_server.rs` | adopt | `GB-0731-RAW` |
| 884 | `M` `crates/codegen/xai-grok-test-support/src/process.rs` | adopt | `GB-0731-RAW` |
| 885 | `A` `crates/codegen/xai-grok-test-support/src/resources.rs` | adopt | `GB-0731-RAW` |
| 886 | `A` `crates/codegen/xai-grok-test-support/src/sandbox.rs` | adopt | `GB-0731-RAW` |
| 887 | `M` `crates/codegen/xai-grok-tools-api/build.rs` | adopt | `GB-0731-RAW` |
| 888 | `M` `crates/codegen/xai-grok-tools-api/proto/grok-tools.proto` | adopt | `GB-0731-RAW` |
| 889 | `M` `crates/codegen/xai-grok-tools-api/src/config_validation.rs` | adopt | `GB-0731-RAW` |
| 890 | `M` `crates/codegen/xai-grok-tools-api/src/lib.rs` | adopt | `GB-0731-RAW` |
| 891 | `M` `crates/codegen/xai-grok-tools-api/src/slash_commands.rs` | adopt | `GB-0731-RAW` |
| 892 | `M` `crates/codegen/xai-grok-tools-api/tests/wire_shape.rs` | adopt | `GB-0731-RAW` |
| 893 | `M` `crates/codegen/xai-grok-tools/Cargo.toml` | adopt | `GB-0731-RAW` |
| 894 | `M` `crates/codegen/xai-grok-tools/src/attribution.rs` | adopt | `GB-0731-RAW` |
| 895 | `M` `crates/codegen/xai-grok-tools/src/bridge.rs` | adopt | `GB-0731-RAW` |
| 896 | `M` `crates/codegen/xai-grok-tools/src/computer/local/shell_state.rs` | adopt | `GB-0731-RAW` |
| 897 | `M` `crates/codegen/xai-grok-tools/src/computer/local/static_shell.rs` | adopt | `GB-0731-RAW` |
| 898 | `M` `crates/codegen/xai-grok-tools/src/computer/local/terminal.rs` | adopt | `GB-0731-RAW` |
| 899 | `M` `crates/codegen/xai-grok-tools/src/computer/types.rs` | adopt | `GB-0731-RAW` |
| 900 | `M` `crates/codegen/xai-grok-tools/src/implementations/codex/apply_patch/tool.rs` | adopt | `GB-0731-RAW` |
| 901 | `M` `crates/codegen/xai-grok-tools/src/implementations/codex/grep_files/tool.rs` | adopt | `GB-0731-RAW` |
| 902 | `M` `crates/codegen/xai-grok-tools/src/implementations/codex/list_dir/tool.rs` | adopt | `GB-0731-RAW` |
| 903 | `M` `crates/codegen/xai-grok-tools/src/implementations/codex/read_file/tool.rs` | adopt | `GB-0731-RAW` |
| 904 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/ask_user_question/mod.rs` | adopt | `GB-0731-RAW` |
| 905 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/bash/mod.rs` | adopt | `GB-0731-RAW` |
| 906 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/enter_plan_mode/mod.rs` | adopt | `GB-0731-RAW` |
| 907 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/exit_plan_mode/mod.rs` | adopt | `GB-0731-RAW` |
| 908 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/grep/mod.rs` | adopt | `GB-0731-RAW` |
| 909 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/image_edit/mod.rs` | adopt | `GB-0731-RAW` |
| 910 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/image_gen/mod.rs` | adopt | `GB-0731-RAW` |
| 911 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/kill_task/mod.rs` | adopt | `GB-0731-RAW` |
| 912 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/kill_task/terminal_command.rs` | adopt | `GB-0731-RAW` |
| 913 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/list_dir/mod.rs` | adopt | `GB-0731-RAW` |
| 914 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/lsp/mod.rs` | adopt | `GB-0731-RAW` |
| 915 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/mod.rs` | adopt | `GB-0731-RAW` |
| 916 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/monitor/tool.rs` | adopt | `GB-0731-RAW` |
| 917 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/monitor/types.rs` | adopt | `GB-0731-RAW` |
| 918 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/read_file/mod.rs` | adopt | `GB-0731-RAW` |
| 919 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/scheduler/actor.rs` | adopt | `GB-0731-RAW` |
| 920 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/scheduler/create.rs` | adopt | `GB-0731-RAW` |
| 921 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/scheduler/delete.rs` | adopt | `GB-0731-RAW` |
| 922 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/scheduler/list.rs` | adopt | `GB-0731-RAW` |
| 923 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/scheduler/mod.rs` | adopt | `GB-0731-RAW` |
| 924 | `A` `crates/codegen/xai-grok-tools/src/implementations/grok_build/scheduler/occurrence_journal.rs` | adopt | `GB-0731-RAW` |
| 925 | `A` `crates/codegen/xai-grok-tools/src/implementations/grok_build/scheduler/occurrence_journal_tests.rs` | adopt | `GB-0731-RAW` |
| 926 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/scheduler/types.rs` | adopt | `GB-0731-RAW` |
| 927 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/search_replace/mod.rs` | adopt | `GB-0731-RAW` |
| 928 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/backend.rs` | adopt | `GB-0731-RAW` |
| 929 | `A` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/backend_tests.rs` | adopt | `GB-0731-RAW` |
| 930 | `A` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/coordinator.rs` | adopt | `GB-0731-RAW` |
| 931 | `A` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/coordinator/query.rs` | adopt | `GB-0731-RAW` |
| 932 | `A` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/coordinator_state.rs` | adopt | `GB-0731-RAW` |
| 933 | `A` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/coordinator_tests.rs` | adopt | `GB-0731-RAW` |
| 934 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs` | adopt | `GB-0731-RAW` |
| 935 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/types.rs` | adopt | `GB-0731-RAW` |
| 936 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task_output/mod.rs` | adopt | `GB-0731-RAW` |
| 937 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task_output/terminal_command.rs` | adopt | `GB-0731-RAW` |
| 938 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task_output/wait_tasks.rs` | adopt | `GB-0731-RAW` |
| 939 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/todo/mod.rs` | adopt | `GB-0731-RAW` |
| 940 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/update_goal/mod.rs` | adopt | `GB-0731-RAW` |
| 941 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/video_gen/mod.rs` | adopt | `GB-0731-RAW` |
| 942 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/web_fetch/http.rs` | adopt | `GB-0731-RAW` |
| 943 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/web_fetch/mod.rs` | adopt | `GB-0731-RAW` |
| 944 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/web_search/mod.rs` | adopt | `GB-0731-RAW` |
| 945 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/workflow/mod.rs` | adopt | `GB-0731-RAW` |
| 946 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build_concise/bash.rs` | adopt | `GB-0731-RAW` |
| 947 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build_concise/read_file.rs` | adopt | `GB-0731-RAW` |
| 948 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build_concise/search_replace.rs` | adopt | `GB-0731-RAW` |
| 949 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build_hashline/edit/mod.rs` | adopt | `GB-0731-RAW` |
| 950 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build_hashline/grep.rs` | adopt | `GB-0731-RAW` |
| 951 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build_hashline/read_file.rs` | adopt | `GB-0731-RAW` |
| 952 | `A` `crates/codegen/xai-grok-tools/src/implementations/lsp/capabilities.rs` | adopt | `GB-0731-RAW` |
| 953 | `M` `crates/codegen/xai-grok-tools/src/implementations/lsp/client.rs` | adopt | `GB-0731-RAW` |
| 954 | `M` `crates/codegen/xai-grok-tools/src/implementations/lsp/config.rs` | adopt | `GB-0731-RAW` |
| 955 | `A` `crates/codegen/xai-grok-tools/src/implementations/lsp/diagnostics.rs` | adopt | `GB-0731-RAW` |
| 956 | `M` `crates/codegen/xai-grok-tools/src/implementations/lsp/dispatch.rs` | adopt | `GB-0731-RAW` |
| 957 | `A` `crates/codegen/xai-grok-tools/src/implementations/lsp/documents.rs` | adopt | `GB-0731-RAW` |
| 958 | `M` `crates/codegen/xai-grok-tools/src/implementations/lsp/manager.rs` | adopt | `GB-0731-RAW` |
| 959 | `M` `crates/codegen/xai-grok-tools/src/implementations/lsp/mod.rs` | adopt | `GB-0731-RAW` |
| 960 | `A` `crates/codegen/xai-grok-tools/src/implementations/lsp/pending.rs` | adopt | `GB-0731-RAW` |
| 961 | `A` `crates/codegen/xai-grok-tools/src/implementations/lsp/pull.rs` | adopt | `GB-0731-RAW` |
| 962 | `A` `crates/codegen/xai-grok-tools/src/implementations/lsp/refresh.rs` | adopt | `GB-0731-RAW` |
| 963 | `M` `crates/codegen/xai-grok-tools/src/implementations/lsp/restart.rs` | adopt | `GB-0731-RAW` |
| 964 | `M` `crates/codegen/xai-grok-tools/src/implementations/lsp/tests.rs` | adopt | `GB-0731-RAW` |
| 965 | `A` `crates/codegen/xai-grok-tools/src/implementations/lsp/tests/mock_servers.rs` | adopt | `GB-0731-RAW` |
| 966 | `A` `crates/codegen/xai-grok-tools/src/implementations/lsp/workspace_open.rs` | adopt | `GB-0731-RAW` |
| 967 | `M` `crates/codegen/xai-grok-tools/src/implementations/memory/get_tool.rs` | adopt | `GB-0731-RAW` |
| 968 | `M` `crates/codegen/xai-grok-tools/src/implementations/memory/search_tool.rs` | adopt | `GB-0731-RAW` |
| 969 | `M` `crates/codegen/xai-grok-tools/src/implementations/memory/types.rs` | adopt | `GB-0731-RAW` |
| 970 | `M` `crates/codegen/xai-grok-tools/src/implementations/opencode/bash/mod.rs` | adopt | `GB-0731-RAW` |
| 971 | `M` `crates/codegen/xai-grok-tools/src/implementations/opencode/edit/mod.rs` | adopt | `GB-0731-RAW` |
| 972 | `M` `crates/codegen/xai-grok-tools/src/implementations/opencode/glob/mod.rs` | adopt | `GB-0731-RAW` |
| 973 | `M` `crates/codegen/xai-grok-tools/src/implementations/opencode/grep/mod.rs` | adopt | `GB-0731-RAW` |
| 974 | `M` `crates/codegen/xai-grok-tools/src/implementations/opencode/read/mod.rs` | adopt | `GB-0731-RAW` |
| 975 | `M` `crates/codegen/xai-grok-tools/src/implementations/opencode/skill/mod.rs` | adopt | `GB-0731-RAW` |
| 976 | `M` `crates/codegen/xai-grok-tools/src/implementations/opencode/todowrite/mod.rs` | adopt | `GB-0731-RAW` |
| 977 | `M` `crates/codegen/xai-grok-tools/src/implementations/opencode/write/mod.rs` | adopt | `GB-0731-RAW` |
| 978 | `M` `crates/codegen/xai-grok-tools/src/implementations/search_tool/mod.rs` | adopt | `GB-0731-RAW` |
| 979 | `M` `crates/codegen/xai-grok-tools/src/implementations/skills/skill.rs` | adopt | `GB-0731-RAW` |
| 980 | `M` `crates/codegen/xai-grok-tools/src/implementations/task_output/tool.rs` | adopt | `GB-0731-RAW` |
| 981 | `M` `crates/codegen/xai-grok-tools/src/implementations/use_tool/mod.rs` | adopt | `GB-0731-RAW` |
| 982 | `M` `crates/codegen/xai-grok-tools/src/implementations/web_search/client.rs` | adopt | `GB-0731-RAW` |
| 983 | `M` `crates/codegen/xai-grok-tools/src/normalization.rs` | adopt | `GB-0731-RAW` |
| 984 | `M` `crates/codegen/xai-grok-tools/src/notification/handle.rs` | adopt | `GB-0731-RAW` |
| 985 | `M` `crates/codegen/xai-grok-tools/src/notification/handle_tests.rs` | adopt | `GB-0731-RAW` |
| 986 | `M` `crates/codegen/xai-grok-tools/src/notification/mod.rs` | adopt | `GB-0731-RAW` |
| 987 | `M` `crates/codegen/xai-grok-tools/src/notification/types.rs` | adopt | `GB-0731-RAW` |
| 988 | `M` `crates/codegen/xai-grok-tools/src/persistence.rs` | adopt | `GB-0731-RAW` |
| 989 | `M` `crates/codegen/xai-grok-tools/src/registry/proto_convert.rs` | adopt | `GB-0731-RAW` |
| 990 | `M` `crates/codegen/xai-grok-tools/src/registry/types.rs` | adopt | `GB-0731-RAW` |
| 991 | `M` `crates/codegen/xai-grok-tools/src/reminders/lsp_diagnostics.rs` | adopt | `GB-0731-RAW` |
| 992 | `M` `crates/codegen/xai-grok-tools/src/reminders/task_completion.rs` | adopt | `GB-0731-RAW` |
| 993 | `M` `crates/codegen/xai-grok-tools/src/tool_taxonomy.rs` | adopt | `GB-0731-RAW` |
| 994 | `M` `crates/codegen/xai-grok-tools/src/types/output.rs` | adopt | `GB-0731-RAW` |
| 995 | `M` `crates/codegen/xai-grok-tools/src/types/resources.rs` | adopt | `GB-0731-RAW` |
| 996 | `M` `crates/codegen/xai-grok-tools/src/types/schema.rs` | adopt | `GB-0731-RAW` |
| 997 | `M` `crates/codegen/xai-grok-tools/src/types/skill_discovery_tracker/mod.rs` | adopt | `GB-0731-RAW` |
| 998 | `M` `crates/codegen/xai-grok-tools/src/types/template_renderer.rs` | adopt | `GB-0731-RAW` |
| 999 | `M` `crates/codegen/xai-grok-tools/src/types/tool_io.rs` | adopt | `GB-0731-RAW` |
| 1000 | `M` `crates/codegen/xai-grok-tools/src/types/tool_metadata.rs` | adopt | `GB-0731-RAW` |
| 1001 | `M` `crates/codegen/xai-grok-tools/src/util/mod.rs` | adopt | `GB-0731-RAW` |
| 1002 | `A` `crates/codegen/xai-grok-tools/src/util/shell_env_policy.rs` | adopt | `GB-0731-RAW` |
| 1003 | `A` `crates/codegen/xai-grok-tools/src/util/shell_env_policy_tests.rs` | adopt | `GB-0731-RAW` |
| 1004 | `M` `crates/codegen/xai-grok-tools/tests/cgroup_memory_test.rs` | adopt | `GB-0731-RAW` |
| 1005 | `A` `crates/codegen/xai-grok-tools/tests/test_subagent_soak.rs` | adopt | `GB-0731-RAW` |
| 1006 | `M` `crates/codegen/xai-grok-update/src/auto_update.rs` | adopt | `GB-0731-RAW` |
| 1007 | `M` `crates/codegen/xai-grok-update/src/lib.rs` | adopt | `GB-0731-RAW` |
| 1008 | `D` `crates/codegen/xai-grok-update/src/minimum_version.rs` | adopt | `GB-0731-RAW` |
| 1009 | `A` `crates/codegen/xai-grok-update/src/version_policy.rs` | adopt | `GB-0731-RAW` |
| 1010 | `M` `crates/codegen/xai-grok-version/Cargo.toml` | adopt | `GB-0731-RAW` |
| 1011 | `M` `crates/codegen/xai-grok-voice/Cargo.toml` | adopt | `GB-0731-RAW` |
| 1012 | `M` `crates/codegen/xai-grok-voice/src/audio/capture.rs` | adopt | `GB-0731-RAW` |
| 1013 | `M` `crates/codegen/xai-grok-voice/src/audio/capture_linux.rs` | adopt | `GB-0731-RAW` |
| 1014 | `A` `crates/codegen/xai-grok-voice/src/audio/capture_subprocess.rs` | adopt | `GB-0731-RAW` |
| 1015 | `M` `crates/codegen/xai-grok-voice/src/audio/mod.rs` | adopt | `GB-0731-RAW` |
| 1016 | `A` `crates/codegen/xai-grok-voice/src/audio/pipe.rs` | adopt | `GB-0731-RAW` |
| 1017 | `A` `crates/codegen/xai-grok-voice/src/audio/protocol.rs` | adopt | `GB-0731-RAW` |
| 1018 | `M` `crates/codegen/xai-grok-voice/src/bin/voice_probe.rs` | adopt | `GB-0731-RAW` |
| 1019 | `M` `crates/codegen/xai-grok-voice/src/config.rs` | adopt | `GB-0731-RAW` |
| 1020 | `M` `crates/codegen/xai-grok-voice/src/lib.rs` | adopt | `GB-0731-RAW` |
| 1021 | `D` `crates/codegen/xai-grok-voice/src/pcm.rs` | adopt | `GB-0731-RAW` |
| 1022 | `M` `crates/codegen/xai-grok-voice/src/pipeline.rs` | adopt | `GB-0731-RAW` |
| 1023 | `M` `crates/codegen/xai-grok-voice/src/probe.rs` | adopt | `GB-0731-RAW` |
| 1024 | `M` `crates/codegen/xai-grok-workspace-client/src/lib.rs` | adopt | `GB-0731-RAW` |
| 1025 | `M` `crates/codegen/xai-grok-workspace-types/src/rpc/deploy.rs` | adopt | `GB-0731-RAW` |
| 1026 | `A` `crates/codegen/xai-grok-workspace-types/src/rpc/export_github.rs` | adopt | `GB-0731-RAW` |
| 1027 | `M` `crates/codegen/xai-grok-workspace-types/src/rpc/git.rs` | adopt | `GB-0731-RAW` |
| 1028 | `M` `crates/codegen/xai-grok-workspace-types/src/rpc/hooks.rs` | adopt | `GB-0731-RAW` |
| 1029 | `M` `crates/codegen/xai-grok-workspace-types/src/rpc/mod.rs` | adopt | `GB-0731-RAW` |
| 1030 | `M` `crates/codegen/xai-grok-workspace-types/src/rpc/workspace.rs` | adopt | `GB-0731-RAW` |
| 1031 | `M` `crates/codegen/xai-grok-workspace/src/activity.rs` | adopt | `GB-0731-RAW` |
| 1032 | `M` `crates/codegen/xai-grok-workspace/src/bin/workspace_server.rs` | adopt | `GB-0731-RAW` |
| 1033 | `M` `crates/codegen/xai-grok-workspace/src/config.rs` | adopt | `GB-0731-RAW` |
| 1034 | `M` `crates/codegen/xai-grok-workspace/src/daemonize.rs` | adopt | `GB-0731-RAW` |
| 1035 | `M` `crates/codegen/xai-grok-workspace/src/diag_server.rs` | adopt | `GB-0731-RAW` |
| 1036 | `M` `crates/codegen/xai-grok-workspace/src/discovery.rs` | adopt | `GB-0731-RAW` |
| 1037 | `M` `crates/codegen/xai-grok-workspace/src/error.rs` | adopt | `GB-0731-RAW` |
| 1038 | `A` `crates/codegen/xai-grok-workspace/src/export_github.rs` | adopt | `GB-0731-RAW` |
| 1039 | `M` `crates/codegen/xai-grok-workspace/src/file_system/attach_file.rs` | adopt | `GB-0731-RAW` |
| 1040 | `M` `crates/codegen/xai-grok-workspace/src/file_system/content.rs` | adopt | `GB-0731-RAW` |
| 1041 | `M` `crates/codegen/xai-grok-workspace/src/file_system/fuzzy.rs` | adopt | `GB-0731-RAW` |
| 1042 | `M` `crates/codegen/xai-grok-workspace/src/folder_trust.rs` | adopt | `GB-0731-RAW` |
| 1043 | `M` `crates/codegen/xai-grok-workspace/src/handle.rs` | adopt | `GB-0731-RAW` |
| 1044 | `M` `crates/codegen/xai-grok-workspace/src/hub.rs` | adopt | `GB-0731-RAW` |
| 1045 | `M` `crates/codegen/xai-grok-workspace/src/hub_auth.rs` | adopt | `GB-0731-RAW` |
| 1046 | `M` `crates/codegen/xai-grok-workspace/src/hub_server.rs` | adopt | `GB-0731-RAW` |
| 1047 | `M` `crates/codegen/xai-grok-workspace/src/lib.rs` | adopt | `GB-0731-RAW` |
| 1048 | `M` `crates/codegen/xai-grok-workspace/src/permission/auto_mode.rs` | adopt | `GB-0731-RAW` |
| 1049 | `M` `crates/codegen/xai-grok-workspace/src/permission/claude_settings.rs` | adopt | `GB-0731-RAW` |
| 1050 | `A` `crates/codegen/xai-grok-workspace/src/permission/gate_preflight.rs` | adopt | `GB-0731-RAW` |
| 1051 | `M` `crates/codegen/xai-grok-workspace/src/permission/hub_permission.rs` | adopt | `GB-0731-RAW` |
| 1052 | `M` `crates/codegen/xai-grok-workspace/src/permission/manager.rs` | adopt | `GB-0731-RAW` |
| 1053 | `M` `crates/codegen/xai-grok-workspace/src/permission/mod.rs` | adopt | `GB-0731-RAW` |
| 1054 | `M` `crates/codegen/xai-grok-workspace/src/permission/policy.rs` | adopt | `GB-0731-RAW` |
| 1055 | `M` `crates/codegen/xai-grok-workspace/src/permission/prompter.rs` | adopt | `GB-0731-RAW` |
| 1056 | `M` `crates/codegen/xai-grok-workspace/src/permission/resolution.rs` | adopt | `GB-0731-RAW` |
| 1057 | `M` `crates/codegen/xai-grok-workspace/src/permission/shell_access.rs` | adopt | `GB-0731-RAW` |
| 1058 | `M` `crates/codegen/xai-grok-workspace/src/permission/types.rs` | adopt | `GB-0731-RAW` |
| 1059 | `M` `crates/codegen/xai-grok-workspace/src/preview_supervisor.rs` | adopt | `GB-0731-RAW` |
| 1060 | `M` `crates/codegen/xai-grok-workspace/src/rpc_envelope.rs` | adopt | `GB-0731-RAW` |
| 1061 | `M` `crates/codegen/xai-grok-workspace/src/session/checkpoint.rs` | adopt | `GB-0731-RAW` |
| 1062 | `M` `crates/codegen/xai-grok-workspace/src/session/git.rs` | adopt | `GB-0731-RAW` |
| 1063 | `M` `crates/codegen/xai-grok-workspace/src/session/jj.rs` | adopt | `GB-0731-RAW` |
| 1064 | `M` `crates/codegen/xai-grok-workspace/src/session/mod.rs` | adopt | `GB-0731-RAW` |
| 1065 | `M` `crates/codegen/xai-grok-workspace/src/session/tool_config.rs` | adopt | `GB-0731-RAW` |
| 1066 | `M` `crates/codegen/xai-grok-workspace/src/upload/mod.rs` | adopt | `GB-0731-RAW` |
| 1067 | `M` `crates/codegen/xai-grok-workspace/src/workspace_ops.rs` | adopt | `GB-0731-RAW` |
| 1068 | `M` `crates/codegen/xai-ratatui-textarea/examples/textarea_demo.rs` | adopt | `GB-0731-RAW` |
| 1069 | `M` `crates/codegen/xai-ratatui-textarea/src/textarea.rs` | adopt | `GB-0731-RAW` |
| 1070 | `M` `crates/codegen/xai-system-power/src/lib.rs` | adopt | `GB-0731-RAW` |
| 1071 | `M` `crates/codegen/xai-system-power/src/linux.rs` | adopt | `GB-0731-RAW` |
| 1072 | `M` `crates/codegen/xai-system-power/src/macos.rs` | adopt | `GB-0731-RAW` |
| 1073 | `M` `crates/codegen/xai-system-power/src/windows.rs` | adopt | `GB-0731-RAW` |
| 1074 | `M` `crates/codegen/xai-tty-utils/src/lib.rs` | adopt | `GB-0731-RAW` |
| 1075 | `M` `crates/codegen/xai-tty-utils/src/process_scope.rs` | adopt | `GB-0731-RAW` |
| 1076 | `A` `crates/codegen/xai-tty-utils/src/runtime.rs` | adopt | `GB-0731-RAW` |
| 1077 | `M` `crates/codegen/xai-workflow/src/engine.rs` | adopt | `GB-0731-RAW` |
| 1078 | `M` `crates/codegen/xai-workflow/src/journal.rs` | adopt | `GB-0731-RAW` |
| 1079 | `M` `crates/common/xai-circuit-breaker/Cargo.toml` | adopt | `GB-0731-RAW` |
| 1080 | `A` `crates/common/xai-circuit-breaker/src/grpc.rs` | adopt | `GB-0731-RAW` |
| 1081 | `M` `crates/common/xai-circuit-breaker/src/lib.rs` | adopt | `GB-0731-RAW` |
| 1082 | `M` `crates/common/xai-computer-hub-sdk/Cargo.toml` | adopt | `GB-0731-RAW` |
| 1083 | `M` `crates/common/xai-computer-hub-sdk/src/auth.rs` | adopt | `GB-0731-RAW` |
| 1084 | `M` `crates/common/xai-computer-hub-sdk/src/connection.rs` | adopt | `GB-0731-RAW` |
| 1085 | `M` `crates/common/xai-computer-hub-sdk/src/connection_borrow.rs` | adopt | `GB-0731-RAW` |
| 1086 | `M` `crates/common/xai-computer-hub-sdk/src/demux.rs` | adopt | `GB-0731-RAW` |
| 1087 | `M` `crates/common/xai-computer-hub-sdk/src/harness.rs` | adopt | `GB-0731-RAW` |
| 1088 | `M` `crates/common/xai-computer-hub-sdk/src/metric_donate.rs` | adopt | `GB-0731-RAW` |
| 1089 | `M` `crates/common/xai-computer-hub-sdk/src/metrics.rs` | adopt | `GB-0731-RAW` |
| 1090 | `M` `crates/common/xai-computer-hub-sdk/src/oidc_provider.rs` | adopt | `GB-0731-RAW` |
| 1091 | `M` `crates/common/xai-grok-compaction/src/intra_compaction/compact.rs` | adopt | `GB-0731-RAW` |
| 1092 | `A` `crates/common/xai-grok-compaction/src/intra_compaction/fit.rs` | adopt | `GB-0731-RAW` |
| 1093 | `M` `crates/common/xai-grok-compaction/src/intra_compaction/mod.rs` | adopt | `GB-0731-RAW` |
| 1094 | `M` `crates/common/xai-grok-compaction/src/item.rs` | adopt | `GB-0731-RAW` |
| 1095 | `M` `crates/common/xai-test-utils/src/git.rs` | adopt | `GB-0731-RAW` |
| 1096 | `M` `crates/common/xai-tool-protocol/src/turn_hook.rs` | adopt | `GB-0731-RAW` |
| 1097 | `M` `crates/common/xai-tool-types/src/serde_lenient.rs` | adopt | `GB-0731-RAW` |
| 1098 | `M` `crates/common/xai-tool-types/src/task.rs` | adopt | `GB-0731-RAW` |
| 1099 | `M` `prod/mc/cli-chat-proxy-types/src/feedback_types.rs` | adopt | `GB-0731-RAW` |
| 1100 | `M` `prod/mc/cli-chat-proxy-types/src/lib.rs` | adopt | `GB-0731-RAW` |
| 1101 | `A` `prod/mc/cli-chat-proxy-types/src/team_managed_config_types.rs` | adopt | `GB-0731-RAW` |

## Acknowledgement and publication gate

The Grok adoption queue is zero, so the exact `dd04f397…` source pin is eligible
for one digest-bound, zero-tree-delta local acknowledgement marker after the
prospective first parent passes the manifest preparation check. Publication
actions remain unauthorized and are not part of this closure.
