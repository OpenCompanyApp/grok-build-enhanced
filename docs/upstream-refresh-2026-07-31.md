# Upstream refresh parity ledger — 2026-07-31

This ledger records the immutable source pins, behavior audit, compatibility
adaptations, and remaining obligations for the 2026-07-31 refresh. It continues
[`upstream-refresh-2026-07-25.md`](upstream-refresh-2026-07-25.md) and its
authoritative 99-row Grok inventory. Fetched source history is evidence only;
no upstream tree was merged or rebased into Enhanced.

Reviewed revisions intentionally remain unchanged. The combined Grok inventory
now contains 144 classified observable behaviors and still has **120 open
`adopt` obligations**. Codex and OpenCode retain explicit live/proxy/history
proof obligations, and Kimi video prompts retain one stable temporary deferral.
The Grok acknowledgement gate is therefore closed.

## Immutable boundary

- Pre-refresh candidate: `ff365bf`, tree
  `bb540cf1749017559994551c7569bc7d24ee4798`.
- Isolated branch/worktree: `refresh/upstreams-20260731` /
  `/home/ruttydm/Projects/worktrees/grok-build-enhanced-refresh-20260731`.
- Fetch timestamp recorded by the pin commit: `2026-07-31T16:23:43Z`.
- No merge, rebase, acknowledgement marker, push, tag, release, PR mutation,
  Homebrew mutation, or credential-bearing live request was performed.

## Source pins and inventories

| Source | Reviewed commit / tree | Latest fetched commit / tree | Reviewed range | Changed records |
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

The other **82 prior `adopt` rows remain open** with their July 25 acceptance
criteria unchanged. The seven prior `already equivalent` rows remain closed:
`GB-A572-009`, `GB-A572-018`, `GB-A572-027C`, `GB-A572-035`,
`GB-A572-039`, `GB-69F0-002`, and `GB-69F0-016`. The official xAI npm
distribution row `GB-A572-030` remains not applicable under the fork-owned
release rule.

## Grok behaviors introduced after the prior pin

| Stable ID | Observable behavior | Classification/state | Evidence or remaining gate |
| --- | --- | --- | --- |
| `GB-0731-001` | Let users enable and disable configured MCP servers without deleting their definitions. | adopt / open | Port state, refresh, and UI contracts. |
| `GB-0731-002` | Copy a rendered plan from the plan approval surface. | adopt / open | Port clipboard feedback and terminal fallbacks. |
| `GB-0731-003` | Recognize SuperGrok Plus consistently in subscription and capability presentation. | adopt / open | Reconcile tier mapping and negative provider cases. |
| `GB-0731-004` | Enable server doom-loop recovery by default while retaining explicit kill switches. | adopt / closed | Composite resolver and precedence tests cover default/config/remote/env behavior. |
| `GB-0731-005` | Preserve terminal-gateway output and completion boundaries. | adopt / open | Port ordered gateway replay and termination tests. |
| `GB-0731-006` | Surface malformed MCP configuration without losing healthy servers. | adopt / open | Add tolerant parse and partial-success tests. |
| `GB-0731-007` | Dispatch `SessionEnd` hooks in headless lifecycle paths. | adopt / open | Port once-only success/cancel/error coverage. |
| `GB-0731-008` | Keep paste chips and question input coherent across editing and submission. | adopt / open | Port UI state and PTY coverage. |
| `GB-0731-009` | Render duration and log-output status consistently. | adopt / open | Reconcile task/log display and snapshots. |
| `GB-0731-010` | Refuse to regress local message counts from a stale remote session record. | adopt / closed | Merge takes the maximum count; regression test pins stale-remote behavior. |
| `GB-0731-011` | Explain loop-stop outcomes without misleading prompts. | adopt / open | Reconcile true-noop/stationarity turn-end text. |
| `GB-0731-012` | Suppress duplicate or inapplicable startup warnings. | adopt / open | Port warning identity/lifecycle tests. |
| `GB-0731-013` | Preserve positional shell arguments, including `$@`, in persistent/static shell wrappers. | adopt / closed | Wrappers restore arguments before eval; shell tests cover both paths. |
| `GB-0731-014` | Show only genuinely backgrounded outstanding tasks in the background tray. | adopt / closed | Task snapshots carry `is_backgrounded`; workspace filters use the shared predicate. |
| `GB-0731-015` | Clean up child work when the owning parent process dies. | adopt / open | Port parent-death teardown and cross-session negatives. |
| `GB-0731-016` | Keep plan/reasoning chrome correct in minimal mode. | adopt / open | Port minimal-mode snapshots and PTYs. |
| `GB-0731-017` | Serialize auth-store mutation across processes. | adopt / open | Add writer lock and sibling login/refresh/logout races per provider. |
| `GB-0731-018` | Preserve compatible behavior in legacy Alacritty terminals. | adopt / open | Port capability detection and PTYs. |
| `GB-0731-019` | Avoid showing a paywall before subscription state is authoritative. | adopt / open | Reconcile cold-start verification and stale-state tests. |
| `GB-0731-020` | Keep the UI responsive during cold initialization. | adopt / open | Port staged initialization and delayed-service PTYs. |
| `GB-0731-021` | Bound memory while forking large session histories. | adopt / open | Port streaming/bounded copy and large-history tests. |
| `GB-0731-022` | Bound worker creation under resource pressure. | adopt / open | Reconcile remaining worker pools and EAGAIN tests. |
| `GB-0731-023` | Provide `/delete` for the intended session/history surface. | adopt / open | Port confirmation, persistence, and current-session guards. |
| `GB-0731-024` | Degrade startup safely when the OS refuses new threads. | already equivalent / closed | The prior fallible-worker repair covers model, DNS, proxy, file/history, and required-startup paths. |
| `GB-0731-025` | Preserve Responses tool-result integrity when duplicate results arrive. | adopt / open | Port normalization and malformed-history tests. |
| `GB-0731-026` | Avoid preview authentication cookie redirect loops. | adopt / open | Port cookie lifecycle and redirect bounds. |
| `GB-0731-027` | Emit the intended stationarity nudge before terminal loop handling. | adopt / open | Port nudge sequencing and telemetry tests. |
| `GB-0731-028` | Run external auth commands through the platform shell. | adopt / closed | Shared helper now covers interactive, refresh, identity, and named-provider execution. |
| `GB-0731-029` | Persist and render cancellation markers coherently. | adopt / open | Port replay/resume/scrollback coverage. |
| `GB-0731-030` | Discover and manage LSP servers, including Roslyn behavior. | adopt / open | Reconcile server config, lifecycle, and diagnostics. |
| `GB-0731-031` | Keep prompt cache keys stable across equivalent turns. | adopt / open | Add end-to-end normal/401/history cache-key proofs. |
| `GB-0731-032` | Preserve full streaming-JSON output and boundaries. | adopt / open | Port stream parser and headless contracts. |
| `GB-0731-033` | Expose `/undo` as the supported rewind command. | adopt / closed | Alias, user guide, and command tests retain `/rewind` compatibility. |
| `GB-0731-034` | Advertise slash commands appropriate to the active session mode. | adopt / open | Reconcile mode transitions and command updates. |
| `GB-0731-035` | Refresh or relogin correctly after machine sleep. | adopt / open | Add clock-jump and provider-isolation tests. |
| `GB-0731-036` | Warn before draft/history operations that would discard work. | adopt / open | Port confirmation and cancellation paths. |
| `GB-0731-037` | Commit settings enum changes consistently. | adopt / open | Reconcile settings persistence and rollback. |
| `GB-0731-038` | Close settings correctly after deep-link navigation. | adopt / open | Port navigation/state tests. |
| `GB-0731-039` | Load supported hooks declared in TOML. | adopt / open | Reconcile trusted scopes and protected-source rules. |
| `GB-0731-040` | Export the supported session artifact to GitHub. | adopt / open | Port local export generation; external publication still requires explicit authorization. |
| `GB-0731-041` | Project coding-data lock state accurately. | adopt / open | Reconcile settings/auth projection without changing telemetry policy. |
| `GB-0731-042` | Include terminal-version metadata in ordinary telemetry. | adopt / open | Port bounded detection and schema tests. |
| `GB-0731-043` | Keep the exit-plan approval barrier ordered with the active turn. | adopt / closed | Mixed write/exit order and permission-cancel race tests pass. |
| `GB-0731-044` | Honor configured extra certificate authorities. | adopt / open | Port scoped TLS roots without weakening provider routing. |
| `GB-0731-045` | Discover and refresh remote managed skills. | adopt / open | Port ownership, refresh, and trust-boundary tests. |

Atomic Grok summary: **111 open adopt**, **24 closed adopt**, **8 closed
already-equivalent**, **1 closed not-applicable**, **0 temporary deferrals**, and
**0 unclassified**, across 144 rows.

## Provider-reference behavior inventory

### OpenAI Codex

| Evidence ID | Classification/state | Local result or remaining gate |
| --- | --- | --- |
| `CDX-4C43-ROUTED-AUTH` / `CDX-PROXY-001` | already equivalent / open proof | Provider-owned clients exist; refresh/revoke/Responses proxy integration proof remains. |
| `CDX-4C43-ENT26` | adopt / closed | Existing lossless raw plan plus Enterprise presentation remains covered. |
| `CDX-4C43-CATALOG` / `CDX-CATALOG-001` | already equivalent / open proof | Auth/catalog refresh exists; entitlement-change and xAI-negative proof remains. |
| `CDX-4C43-ITEM-IDS` / `CDX-HISTORY-001` | already equivalent / open proof | Full-history HTTP design remains; persist/resume/compact continuity proof remains. |
| `CDX-4C43-RETRY` / `CDX-RETRY-001` | adopt / implementation closed; actor proof open | Structured 429 retry veto propagates through classification and events; the prior full request-actor proof obligation remains. |
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
remain unchanged. Hosted image-tool credential-gated live proof remains open.

### OpenCode and OpenCode Codex auth

| Evidence ID | Classification/state | Result |
| --- | --- | --- |
| `OC-7534-CACHE-KEY` | already equivalent / closed | Existing cache-key compatibility remains. |
| `OC-7534-AUTH-REFETCH` / `OC-AUTH-001` | already equivalent / open proof | OAuth/catalog refetch isolation proof carries forward. |
| `OC-E4BD-INTERLEAVED` | adopt / open | OpenCode accepts bare-string and arbitrary-field interleaved reasoning metadata. Enhanced currently recognizes only the fixed Kimi fields, so vendor-defined fields such as `reasoning_text` remain an interoperability obligation. |
| `OC-E4BD-NO-OTHER-CODEX-DELTA` | not applicable / closed | The remaining new changes affect other providers and OpenCode runtime, not the Codex adapter contract. |
| `OCAUTH-BEC2-UNCHANGED` | already equivalent / closed | Source did not advance. |

### Kimi Code and Z.AI research

| Evidence ID | Classification/state | Result |
| --- | --- | --- |
| `KIMI-BFA-QUOTA` | adopt / closed | Structured quota/balance/recharge 429s are fatal, redacted, and never retried. |
| `KIMI-USAGE-RESET` | already equivalent / closed | Exact minute/reset projection has a local regression test. |
| `KIMI-CATALOG-FALLBACK` | not applicable / closed | Generic static fallback would violate credential-bound entitlement discovery; cache replacement remains provider-bound. |
| `KIMI-OAUTH` | not applicable / closed | Enhanced intentionally remains an experimental API-key-only Kimi provider. |
| `KIMI-VIDEO-001` | temporarily deferred / open | Provider-hosted video prompt parity remains due 2026-08-01 with the prior owner, acceptance criteria, and tests unchanged. |
| `CODEXBAR-ZAI-PERCENT` | already equivalent / closed | Z.AI percentage fallback and clamping have explicit tests. |

No runtime Z.AI provider, login flow, credential, or product claim is inferred
from the unchanged research sources or CodexBar UI behavior.

## Validation

Validation completed against the final formatted source candidate:

- Focused Rust regressions passed: 21 shell tests for collaboration metadata,
  free-plan gating, catalog renewal, business-plan preservation, auth-provider
  working directories, platform shells, workflows/doom-loop defaults, session
  merge, Kimi/Z.AI usage, and image dispatch; 6 tools tests for background-state
  filtering, Codex image headers/schema redaction, xAI isolation, and shell
  arguments; 4 pager command/shortcut/plan tests; 3 Kimi quota tests; the
  sampling retry-veto test; the workspace background-task RPC test; the managed
  MCP timeout test; and the focused model and Linux voice-capture checks.
- `CARGO_INCREMENTAL=0 cargo check -p xai-grok-pager-bin` passed. Cargo emitted
  only the known non-fatal warning that `xai-grok-pager-render/build.rs` is both
  the build script and the `warp_vendor_build_validation` integration target.
- All 73 fork-script guardrail tests and all 20 release/installer tests passed.
  The aggregate fork contracts passed branding, provider isolation, Codex
  search, Warp locks, updater routes, generated-workspace discipline, workflow
  pins, and tracked-secret hygiene; the offline Codex search contract passed.
- `cargo fmt --all -- --check`, `git diff --check`, and strict committed-tree
  ownership validation passed after the thematic commits below.
- Live provider calls were not run. They require explicitly entitled
  credentials and must not expose authenticated payloads or credential state.

## Acknowledgement and publication gate

The 120 open Grok adoption rows, provider proof obligations, and Kimi temporary
deferral forbid advancing any reviewed pin or creating a Grok acknowledgement
marker. `latest_fetched` remains a review queue. Publication actions were not
authorized and are not part of this refresh.
