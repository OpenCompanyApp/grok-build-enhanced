# Upstream refresh parity ledger — 2026-08-02

This ledger records the immutable source pins, behavioral audit, compatibility
adaptations, and closure evidence for the August 2 refresh. It continues
[the July 31 ledger](upstream-refresh-2026-07-31.md). Fetched history is
evidence only; no upstream tree was content-merged or rebased into Enhanced.

The audit is frozen at the exact revisions below. Later commits require a
separate refresh. Provider checks use synthetic credentials and local endpoints
only and are described as `offline-qualified`, never live-qualified.

## Immutable boundary

- Starting Enhanced commit: `b6313a50c6812612be27617cb1929a8fbb203a23`
  after the pin-only commit.
- Isolated branch/worktree: `refresh/upstreams-20260802` /
  `/home/ruttydm/Projects/worktrees/grok-build-enhanced-refresh-20260802`.
- Fetch timestamp: `2026-08-02T13:51:00Z`.
- Root `Cargo.toml` remained byte-identical, SHA-256
  `28a3ea7e1c859729a0c5cf77f87ff7f0ece319a576697b274917359e11be480b`.
- No push, tag, release, PR mutation, live-provider request, or credential
  import was performed.

## Source pins and inventories

| Source | Prior reviewed | Frozen revision | Commits | Changed paths | Raw digest |
| --- | --- | --- | ---: | ---: | --- |
| Grok Build | `dd04f397b1d02f2272b092555669dfba1f01bc85` | `a4221165824e5b1f5c4c10b7459f65e78dd6448d` | 1 | 165 | `122b5f1c70ac1ca329a18bf26e613c13f85c8fd51fd43a4943a9226156a4c2fc` |
| OpenAI Codex | `2c005abb0765bfe3ef42a23fe88d5b806184fa83` | `582569998181aad08a88bacc151a94b2048a5d1f` | 32 | 309 | `1217e05631fdddee0dfcebfd52674ccef0aa18ded8929378f1b2cdb6fa3d3cd6` |
| OpenCode | `e4bd9757a3a5dc7461d286000a19e9bd7df57c40` | `32f278b48f1a495611165d8a9f1ace0b512933e2` | 7 | 87 | `17e83d514c87da1f7cc6c850f249d72e896ad6397c4e7fd866fc2abc0a021e69` |
| Kimi Code | `bfa00807c975fdc5b84dda32d47b16b09e8d42c1` | `e22479a62eed9c3b78a67b313f4332c2c0ba9670` | 2 | 6 | `f917acf7723987823b8d5646d23af1b4a7460c39d34bed13528a701024590877` |
| CodexBar research | `cc8da27cec92029a6435bfee4a703a719290234e` | `78523f4ad890893851219c5f5d41139a60a3139a` | 302 | 392 | `34a6c647c8af2c46d7b20083d6df11409c24494ed87735398faccdc338d51558` |

OpenCode Codex auth, Warp themes, Kimi CLI, the Z.AI SDK and coding-plugin
references, GLM-5, and the Z.AI usage-browser reference did not advance.

## Grok behavior inventory

| Stable ID | Observable behavior | Classification/state | Local result |
| --- | --- | --- | --- |
| `GB-A422-001` | Internal local-workspace CLI/environment intent and acknowledgement metadata | not applicable / closed | The public Cargo product excludes the internal local-workspace feature; Enhanced retains its existing local build-session contract and does not advertise the unavailable mode. |
| `GB-A422-002` | Internal Sandbox/local welcome choice and source-aware history | not applicable / closed | The picker is part of the same unpublished internal feature and is not exposed by the public upstream or Enhanced default feature graph. |
| `GB-A422-003` | Internal own-mode gateway supervisor | not applicable / closed | The public snapshot declares this only for its internal `default-bazel` graph but omits the referenced `gateway_bridge` module; Enhanced does not invent or ship that unpublished app-server architecture. Existing local build sessions remain unchanged. |
| `GB-A422-004` | Session registry and resource reconciliation | adopt / closed | Registry replacement, churn, and shutdown paths were ported with bounded ownership. |
| `GB-A422-005` | Value-free credential provenance | adopt / closed | Each request records only sent/missing/unknown; no token material or fragment enters errors, callbacks, or logs. |
| `GB-A422-006` | Bounded auth recovery across suspension | adopt / closed | Structured retry budget and dual-clock recovery were ported. The upstream loopback end-to-end fixture is represented by unit and sampler provenance coverage instead: Enhanced must not make a loopback URL trusted for xAI session credentials, even in this provider-neutral offline qualification campaign. |
| `GB-A422-007` | Recognize current context-overflow wording | adopt / closed | Shared sampling classification accepts the new bounded phrase. |
| `GB-A422-008` | Respect overload/529 retry signals and vetoes | adopt / closed | Retry classification honors explicit veto and provider overload status. |
| `GB-A422-009` | Cancel in-flight compaction | adopt / closed | Cancellation reaches Chat, Responses, Messages, recap, and full-replace compaction. |
| `GB-A422-010` | Equality-reminder wording contract | already equivalent / closed | Enhanced already mirrors the exact shared reminder wording. |
| `GB-A422-011` | Subagent-completion reminder wording contract | already equivalent / closed | Existing formatter and regression coverage preserve the shared wording. |
| `GB-A422-012` | Configurable bounded wait-task timeout | adopt / closed | Wait schema and execution context carry the bounded duration. |
| `GB-A422-013` | Watch project skill changes without startup races | adopt / closed | Watcher startup, debounce, and refresh lifecycle were ported. |
| `GB-A422-014` | Reap completed PTY children | adopt / closed | PTY wrappers now perform the nonblocking reap. |
| `GB-A422-015` | Preserve nested OS errors in TLS diagnostics | adopt / closed | HTTP error traversal retains actionable OS causes without credential data. |
| `GB-A422-016` | Redacted typed login failures | adopt / closed | Authentication errors are typed and display only bounded structural context. |
| `GB-A422-017` | Workspace activity/idle transitions | adopt / closed | Activity state and idle projection were ported with race coverage. |
| `GB-A422-018` | Computer-hub SDK request round-trip timing | adopt / closed | Connection metrics use request-scoped RTT and liveness state. |
| `GB-A422-019` | Protect `.grok` configuration edits | adopt / closed | Shell permission checks preserve the protected configuration boundary. |
| `GB-A422-020` | Dashboard session deletion | adopt / closed | Confirmation, current-session guards, persistence, and dashboard state were ported. |
| `GB-A422-021` | Activity for ended sessions | adopt / closed | Ended records retain coherent last-activity projection. |
| `GB-A422-022` | Show Ctrl+. in shortcut help | adopt / closed | Help and mode-aware input handling reflect the actual binding. |
| `GB-A422-023` | Dual-clock elapsed duration | adopt / closed | UI durations stay monotonic across wall-clock jumps. |
| `GB-A422-024` | Typed workspace/RPC errors | adopt / closed | Stable kinds cross the workspace envelope without provider text leakage. |
| `GB-A422-025` | Usage visibility contract | already equivalent / closed | Enhanced already presents the ordinary upstream usage details. |
| `GB-A422-026` | Internal PI-only file-descriptor build plumbing | not applicable / closed | The unpublished/internal build target is outside the public Cargo artifact and fork package graph. |
| `GB-A422-027` | `prod/mc` managed-service type changes | not applicable / closed | Managed xAI service internals are outside the fork runtime and provider adapters. |
| `GB-A422-028` | Upstream version, changelog, source-revision, and generated-root bookkeeping | not applicable / closed | Enhanced owns branding, versions, releases, generated root manifest, and provenance ledgers. |

Summary: **19 adopt/closed**, **3 already-equivalent/closed**, **6
not-applicable/closed**, **0 deferred**, and **0 unclassified**. The
`local-workspace` exception is source-bound: upstream's ordinary Cargo default
also excludes it, and the public tree contains call sites but no module
definition. It does not weaken any pre-existing Enhanced local build behavior.

## Provider-reference inventory

### OpenAI Codex

| Stable ID | Classification/state | Result |
| --- | --- | --- |
| `CDX-5825-001` | already equivalent / closed | Enhanced already binds MCP tool metadata to the owning session/request and never replays it across providers. |
| `CDX-5825-002` | already equivalent / closed | JSONL history has a single writer and preserves fork/resume/compaction integrity. |
| `CDX-5825-003` | already equivalent / offline-qualified | The July route, refresh/revoke, catalog, history, retry, hosted-image, and Kimi reasoning/video proofs were rerun against the refreshed tree. |
| `CDX-5825-004` | not applicable / closed | Codex user-message admission and realtime acknowledgement belong to its app/agent engine. |
| `CDX-5825-005` | not applicable / closed | Codex MCP elicitation auto-review belongs to its permission and app-server architecture. |
| `CDX-5825-006` | not applicable / closed | Remote plugin search, bundles, and marketplace migration do not replace Enhanced MCP/managed-skill architecture. |
| `CDX-5825-007` | not applicable / closed | Realtime delegation transitions are outside the provider adapter. |
| `CDX-5825-008` | not applicable / closed | Codex TUI key chords and redraw optimizations do not replace the Grok TUI. |
| `CDX-5825-009` | not applicable / closed | Codex exec-server dispatching is outside the adapter. |
| `CDX-5825-010` | not applicable / closed | Codex image-preparation analytics are app telemetry, not provider wire behavior. |
| `CDX-5825-011` | not applicable / closed | Thread sections, paginated summaries, and state-DB picker behavior belong to Codex storage/app-server. |
| `CDX-5825-012` | not applicable / closed | Sandboxed V8 code mode does not replace Grok tools or sandbox. |
| `CDX-5825-013` | not applicable / closed | `--approve-for-me` is a Codex CLI permission policy and is not imported. |
| `CDX-5825-014` | not applicable / closed | External-session and Cursor-skill migration are outside the provider adapter. |
| `CDX-5825-015` | not applicable / closed | Windows Bazel, release, generated schema, and package bookkeeping are source-owned. |

### OpenCode

| Stable ID | Classification/state | Result |
| --- | --- | --- |
| `OC-32F2-001` | not applicable / closed | Stale prompt-control repair is OpenCode application UI state, not the Codex interoperability wire. |
| `OC-32F2-002` | not applicable / closed | Go/Zen data-policy documentation does not alter Enhanced provider contracts or telemetry policy. |
| `OC-32F2-003` | not applicable / closed | Version, translations, lockfile, and generated package updates contain no adapter delta. |

The prior `OC-E4BD-INTERLEAVED` proof remains offline-qualified and was
rerun; bounded provider-defined reasoning remains scoped to provider, model,
and credential generation.

### Kimi Code and CodexBar research

| Stable ID | Classification/state | Result |
| --- | --- | --- |
| `KIMI-E224-001` | already equivalent / closed | Git-test signing suppression changes only Kimi's test harness. |
| `KIMI-E224-002` | not applicable / closed | KAP server experimental-flag metadata is app-server state outside the API-key provider adapter. |
| `CODEXBAR-7852-001` | not applicable / closed | Multi-account usage presentation remains research evidence and does not establish a Z.AI runtime provider, credential path, or product claim. |

## Validation

All provider qualification used synthetic credentials and local endpoints; it
is `offline-qualified`, not live-provider qualification. Rust tests ran with a
fixed isolated `HOME`, `XDG_CONFIG_HOME`, and `GROK_HOME`. The large async test
binaries required `RUST_MIN_STACK=16777216`; production defaults were not
changed.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | passed |
| `xai-grok-shell` library suite | 6,135 passed; 8 ignored; 0 failed |
| `xai-grok-pager` library suite | 8,023 passed; 10 ignored; 0 failed |
| `xai-grok-sampler` library suite | 318 passed; 0 failed |
| `xai-grok-sampling-types` library suite | 322 passed; 0 failed |
| `xai-grok-tools` library suite | 2,956 passed; 7 ignored; 0 failed |
| `xai-grok-workspace` hermetic serial library suite | 1,595 passed; 0 failed |
| `xai-computer-hub-sdk` library suite | 217 passed; 0 failed |
| `xai-grok-http` library suite | 10 passed; 0 failed |
| Compaction, telemetry, workspace-types, tool-protocol, and tool-types libraries | 609 passed; 0 failed |
| `CARGO_INCREMENTAL=0 cargo check --locked -p xai-grok-pager-bin` | passed |
| `python3 -I -B -m unittest discover -s fork/scripts/tests -v` | 78 passed |
| `python3 -I -B fork/scripts/check_fork_contracts.py` | passed: branding, providers, Codex search, Warp, updater, workspace, workflows, and secrets |

The workspace suite was run serially because its daemon tests share process
resources; the isolated home also prevents user configuration from changing
permission fixtures. DotSlash/protoc remained a local preflight dependency and
the existing CI installation path was retained.

## Complete 165 raw-path ledger

The rows below are the exhaustive recursive tree delta from Grok tree
`c5c7bdcda32a828efa112883dcd5279ce78714ec` to
`a8c8b7dd2967c2bc84ded26493d142111afb36ed`. The canonical raw
`(mode,type,oid)` stream has SHA-256
`122b5f1c70ac1ca329a18bf26e613c13f85c8fd51fd43a4943a9226156a4c2fc`.
The authenticated source ID is `grok-build-upstream`; target commit
`a4221165824e5b1f5c4c10b7459f65e78dd6448d` has the exact sole parent
`dd04f397b1d02f2272b092555669dfba1f01bc85`.

| Row | Raw path | Outcome | Evidence |
| ---: | --- | --- | --- |
| 1 | `M` `Cargo.lock` | adopt | `GB-A422-RAW` |
| 2 | `M` `Cargo.toml` | not applicable | `GB-A422-028` |
| 3 | `M` `SOURCE_REV` | not applicable | `GB-A422-028` |
| 4 | `M` `clippy.toml` | adopt | `GB-A422-RAW` |
| 5 | `M` `crates/codegen/ptyctl/src/pty.rs` | adopt | `GB-A422-RAW` |
| 6 | `M` `crates/codegen/xai-grok-http/src/lib.rs` | adopt | `GB-A422-RAW` |
| 7 | `M` `crates/codegen/xai-grok-pager-bin/Cargo.toml` | adopt | `GB-A422-RAW` |
| 8 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/pty.rs` | adopt | `GB-A422-RAW` |
| 9 | `M` `crates/codegen/xai-grok-pager/Cargo.toml` | adopt | `GB-A422-RAW` |
| 10 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/03-keyboard-shortcuts.md` | adopt | `GB-A422-RAW` |
| 11 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/04-slash-commands.md` | adopt | `GB-A422-RAW` |
| 12 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/17-sessions.md` | adopt | `GB-A422-RAW` |
| 13 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/23-dashboard.md` | adopt | `GB-A422-RAW` |
| 14 | `M` `crates/codegen/xai-grok-pager/src/actions/defaults.rs` | adopt | `GB-A422-RAW` |
| 15 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/interactions.rs` | adopt | `GB-A422-RAW` |
| 16 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/permissions.rs` | adopt | `GB-A422-RAW` |
| 17 | `M` `crates/codegen/xai-grok-pager/src/app/actions.rs` | adopt | `GB-A422-RAW` |
| 18 | `M` `crates/codegen/xai-grok-pager/src/app/agent.rs` | adopt | `GB-A422-RAW` |
| 19 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/input.rs` | adopt | `GB-A422-RAW` |
| 20 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/interactions.rs` | adopt | `GB-A422-RAW` |
| 21 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/mod.rs` | adopt | `GB-A422-RAW` |
| 22 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/prompt.rs` | adopt | `GB-A422-RAW` |
| 23 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/render.rs` | adopt | `GB-A422-RAW` |
| 24 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/session.rs` | adopt | `GB-A422-RAW` |
| 25 | `M` `crates/codegen/xai-grok-pager/src/app/app_view.rs` | adopt | `GB-A422-RAW` |
| 26 | `M` `crates/codegen/xai-grok-pager/src/app/cli.rs` | adopt | `GB-A422-RAW` |
| 27 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/ctx.rs` | adopt | `GB-A422-RAW` |
| 28 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/dashboard.rs` | adopt | `GB-A422-RAW` |
| 29 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/prompt.rs` | adopt | `GB-A422-RAW` |
| 30 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/router.rs` | adopt | `GB-A422-RAW` |
| 31 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/session/foreign.rs` | adopt | `GB-A422-RAW` |
| 32 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs` | adopt | `GB-A422-RAW` |
| 33 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/session/load.rs` | adopt | `GB-A422-RAW` |
| 34 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/task_result.rs` | adopt | `GB-A422-RAW` |
| 35 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/dashboard.rs` | adopt | `GB-A422-RAW` |
| 36 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/mod.rs` | adopt | `GB-A422-RAW` |
| 37 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/lifecycle.rs` | adopt | `GB-A422-RAW` |
| 38 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/load.rs` | adopt | `GB-A422-RAW` |
| 39 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/task_result.rs` | adopt | `GB-A422-RAW` |
| 40 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/turn.rs` | adopt | `GB-A422-RAW` |
| 41 | `M` `crates/codegen/xai-grok-pager/src/app/effects/helpers.rs` | adopt | `GB-A422-RAW` |
| 42 | `M` `crates/codegen/xai-grok-pager/src/app/effects/mod.rs` | adopt | `GB-A422-RAW` |
| 43 | `M` `crates/codegen/xai-grok-pager/src/app/effects/tests.rs` | adopt | `GB-A422-RAW` |
| 44 | `M` `crates/codegen/xai-grok-pager/src/app/event_loop.rs` | adopt | `GB-A422-RAW` |
| 45 | `M` `crates/codegen/xai-grok-pager/src/app/leader_cluster/mod.rs` | adopt | `GB-A422-RAW` |
| 46 | `M` `crates/codegen/xai-grok-pager/src/app/mod.rs` | adopt | `GB-A422-RAW` |
| 47 | `M` `crates/codegen/xai-grok-pager/src/app/modals.rs` | adopt | `GB-A422-RAW` |
| 48 | `M` `crates/codegen/xai-grok-pager/src/app/session_startup.rs` | adopt | `GB-A422-RAW` |
| 49 | `M` `crates/codegen/xai-grok-pager/src/app/xt_filter.rs` | adopt | `GB-A422-RAW` |
| 50 | `M` `crates/codegen/xai-grok-pager/src/pty_wrap.rs` | adopt | `GB-A422-RAW` |
| 51 | `M` `crates/codegen/xai-grok-pager/src/views/dashboard/mod.rs` | adopt | `GB-A422-RAW` |
| 52 | `M` `crates/codegen/xai-grok-pager/src/views/dashboard/render.rs` | adopt | `GB-A422-RAW` |
| 53 | `M` `crates/codegen/xai-grok-pager/src/views/dashboard/row.rs` | adopt | `GB-A422-RAW` |
| 54 | `M` `crates/codegen/xai-grok-pager/src/views/dashboard/state.rs` | adopt | `GB-A422-RAW` |
| 55 | `M` `crates/codegen/xai-grok-pager/src/views/modal.rs` | adopt | `GB-A422-RAW` |
| 56 | `M` `crates/codegen/xai-grok-pager/src/views/question_view.rs` | adopt | `GB-A422-RAW` |
| 57 | `M` `crates/codegen/xai-grok-pager/src/views/session_picker.rs` | adopt | `GB-A422-RAW` |
| 58 | `M` `crates/codegen/xai-grok-pager/src/views/shortcuts_help.rs` | adopt | `GB-A422-RAW` |
| 59 | `M` `crates/codegen/xai-grok-pager/src/views/welcome/hero_box.rs` | adopt | `GB-A422-RAW` |
| 60 | `M` `crates/codegen/xai-grok-pager/src/views/welcome/mod.rs` | adopt | `GB-A422-RAW` |
| 61 | `A` `crates/codegen/xai-grok-pager/src/views/welcome/workspace_mode.rs` | not applicable | `GB-A422-002` |
| 62 | `M` `crates/codegen/xai-grok-sampler/src/actor/request_task.rs` | adopt | `GB-A422-RAW` |
| 63 | `M` `crates/codegen/xai-grok-sampler/src/client.rs` | adopt | `GB-A422-RAW` |
| 64 | `M` `crates/codegen/xai-grok-sampler/src/events.rs` | adopt | `GB-A422-RAW` |
| 65 | `M` `crates/codegen/xai-grok-sampler/src/handle.rs` | adopt | `GB-A422-RAW` |
| 66 | `M` `crates/codegen/xai-grok-sampler/src/retry.rs` | adopt | `GB-A422-RAW` |
| 67 | `M` `crates/codegen/xai-grok-sampler/src/stream/collect.rs` | adopt | `GB-A422-RAW` |
| 68 | `M` `crates/codegen/xai-grok-sampling-types/src/error.rs` | adopt | `GB-A422-RAW` |
| 69 | `M` `crates/codegen/xai-grok-sampling-types/src/lib.rs` | adopt | `GB-A422-RAW` |
| 70 | `M` `crates/codegen/xai-grok-shell-base/Cargo.toml` | adopt | `GB-A422-RAW` |
| 71 | `M` `crates/codegen/xai-grok-shell/CHANGELOG.md` | not applicable | `GB-A422-028` |
| 72 | `M` `crates/codegen/xai-grok-shell/Cargo.toml` | adopt | `GB-A422-RAW` |
| 73 | `A` `crates/codegen/xai-grok-shell/benches/skills_watcher_startup.rs` | adopt | `GB-A422-RAW` |
| 74 | `M` `crates/codegen/xai-grok-shell/changelogs/0.2.115.md` | not applicable | `GB-A422-028` |
| 75 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.117.json` | not applicable | `GB-A422-028` |
| 76 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.117.md` | not applicable | `GB-A422-028` |
| 77 | `M` `crates/codegen/xai-grok-shell/src/agent/handlers/session.rs` | adopt | `GB-A422-RAW` |
| 78 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/acp_agent.rs` | adopt | `GB-A422-RAW` |
| 79 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs` | adopt | `GB-A422-RAW` |
| 80 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/code_nav.rs` | adopt | `GB-A422-RAW` |
| 81 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/mod.rs` | adopt | `GB-A422-RAW` |
| 82 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/session_lifecycle.rs` | adopt | `GB-A422-RAW` |
| 83 | `A` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/session_registry.rs` | adopt | `GB-A422-RAW` |
| 84 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/tests.rs` | adopt | `GB-A422-RAW` |
| 85 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/tests/dhat_soak.rs` | adopt | `GB-A422-RAW` |
| 86 | `M` `crates/codegen/xai-grok-shell/src/auth/device_code.rs` | adopt | `GB-A422-RAW` |
| 87 | `M` `crates/codegen/xai-grok-shell/src/auth/flow.rs` | adopt | `GB-A422-RAW` |
| 88 | `M` `crates/codegen/xai-grok-shell/src/auth/manager.rs` | adopt | `GB-A422-RAW` |
| 89 | `M` `crates/codegen/xai-grok-shell/src/auth/manager/sleep_gate.rs` | adopt | `GB-A422-RAW` |
| 90 | `M` `crates/codegen/xai-grok-shell/src/auth/manager_tests.rs` | adopt | `GB-A422-RAW` |
| 91 | `M` `crates/codegen/xai-grok-shell/src/config/watcher.rs` | adopt | `GB-A422-RAW` |
| 92 | `M` `crates/codegen/xai-grok-shell/src/extensions/feedback.rs` | adopt | `GB-A422-RAW` |
| 93 | `M` `crates/codegen/xai-grok-shell/src/extensions/notification.rs` | adopt | `GB-A422-RAW` |
| 94 | `M` `crates/codegen/xai-grok-shell/src/extensions/session_admin.rs` | adopt | `GB-A422-RAW` |
| 95 | `M` `crates/codegen/xai-grok-shell/src/sampling/error.rs` | adopt | `GB-A422-RAW` |
| 96 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session.rs` | adopt | `GB-A422-RAW` |
| 97 | `A` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/auth_retry.rs` | adopt | `GB-A422-RAW` |
| 98 | `A` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/auth_retry_tests.rs` | adopt | `GB-A422-RAW` |
| 99 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/recap.rs` | adopt | `GB-A422-RAW` |
| 100 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs` | adopt | `GB-A422-RAW` |
| 101 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/spawn.rs` | adopt | `GB-A422-RAW` |
| 102 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/tasks_cancel.rs` | adopt | `GB-A422-RAW` |
| 103 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/turn.rs` | adopt | `GB-A422-RAW` |
| 104 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/types.rs` | adopt | `GB-A422-RAW` |
| 105 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/auth_error_no_retry_tests.rs` | adopt | `GB-A422-RAW` |
| 106 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/cancel_running_task_tests.rs` | adopt | `GB-A422-RAW` |
| 107 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/idle_resume_tests.rs` | adopt | `GB-A422-RAW` |
| 108 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/inline_auto_compact_flow_tests.rs` | adopt | `GB-A422-RAW` |
| 109 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/memory_config_tests.rs` | adopt | `GB-A422-RAW` |
| 110 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/replay_buffer_send_update_tests.rs` | adopt | `GB-A422-RAW` |
| 111 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/support.rs` | adopt | `GB-A422-RAW` |
| 112 | `A` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/turn/auth_retry_budget_tests.rs` | adopt | `GB-A422-RAW` |
| 113 | `M` `crates/codegen/xai-grok-shell/src/session/commands.rs` | adopt | `GB-A422-RAW` |
| 114 | `M` `crates/codegen/xai-grok-shell/src/session/compaction.rs` | adopt | `GB-A422-RAW` |
| 115 | `M` `crates/codegen/xai-grok-shell/src/session/compaction_config.rs` | adopt | `GB-A422-RAW` |
| 116 | `M` `crates/codegen/xai-grok-shell/src/session/helpers/full_replace_compaction.rs` | adopt | `GB-A422-RAW` |
| 117 | `M` `crates/codegen/xai-grok-shell/src/session/helpers/session_compact.rs` | adopt | `GB-A422-RAW` |
| 118 | `M` `crates/codegen/xai-grok-shell/src/session/persistence.rs` | adopt | `GB-A422-RAW` |
| 119 | `M` `crates/codegen/xai-grok-shell/src/session/unified_list/mod.rs` | adopt | `GB-A422-RAW` |
| 120 | `M` `crates/codegen/xai-grok-shell/src/terminal/pty_session.rs` | adopt | `GB-A422-RAW` |
| 121 | `M` `crates/codegen/xai-grok-shell/src/test_support/mod.rs` | adopt | `GB-A422-RAW` |
| 122 | `A` `crates/codegen/xai-grok-shell/src/util/dual_clock.rs` | adopt | `GB-A422-RAW` |
| 123 | `A` `crates/codegen/xai-grok-shell/src/util/dual_clock_tests.rs` | adopt | `GB-A422-RAW` |
| 124 | `M` `crates/codegen/xai-grok-shell/src/util/mod.rs` | adopt | `GB-A422-RAW` |
| 125 | `M` `crates/codegen/xai-grok-shell/tests/test_leader_soak.rs` | adopt | `GB-A422-RAW` |
| 126 | `M` `crates/codegen/xai-grok-shell/tests/test_registry_churn.rs` | adopt | `GB-A422-RAW` |
| 127 | `M` `crates/codegen/xai-grok-shell/tests/test_sampling_client.rs` | adopt | `GB-A422-RAW` |
| 128 | `M` `crates/codegen/xai-grok-telemetry/Cargo.toml` | adopt | `GB-A422-RAW` |
| 129 | `M` `crates/codegen/xai-grok-telemetry/src/events.rs` | adopt | `GB-A422-RAW` |
| 130 | `M` `crates/codegen/xai-grok-telemetry/src/session_ctx.rs` | adopt | `GB-A422-RAW` |
| 131 | `M` `crates/codegen/xai-grok-telemetry/src/unified_log.rs` | adopt | `GB-A422-RAW` |
| 132 | `M` `crates/codegen/xai-grok-test-support/src/mock_server.rs` | adopt | `GB-A422-RAW` |
| 133 | `M` `crates/codegen/xai-grok-tools/Cargo.toml` | not applicable | `GB-A422-026` |
| 134 | `M` `crates/codegen/xai-grok-tools/build.rs` | not applicable | `GB-A422-026` |
| 135 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task_output/mod.rs` | adopt | `GB-A422-RAW` |
| 136 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task_output/terminal_command.rs` | adopt | `GB-A422-RAW` |
| 137 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task_output/wait_tasks.rs` | adopt | `GB-A422-RAW` |
| 138 | `M` `crates/codegen/xai-grok-tools/src/registry/types.rs` | adopt | `GB-A422-RAW` |
| 139 | `M` `crates/codegen/xai-grok-tools/src/reminders/task_completion.rs` | adopt | `GB-A422-RAW` |
| 140 | `M` `crates/codegen/xai-grok-tools/src/types/context.rs` | adopt | `GB-A422-RAW` |
| 141 | `M` `crates/codegen/xai-grok-version/Cargo.toml` | not applicable | `GB-A422-028` |
| 142 | `A` `crates/codegen/xai-grok-workspace-types/src/rpc/export.rs` | adopt | `GB-A422-RAW` |
| 143 | `M` `crates/codegen/xai-grok-workspace-types/src/rpc/mod.rs` | adopt | `GB-A422-RAW` |
| 144 | `M` `crates/codegen/xai-grok-workspace/Cargo.toml` | adopt | `GB-A422-RAW` |
| 145 | `M` `crates/codegen/xai-grok-workspace/src/activity.rs` | adopt | `GB-A422-RAW` |
| 146 | `M` `crates/codegen/xai-grok-workspace/src/error.rs` | adopt | `GB-A422-RAW` |
| 147 | `M` `crates/codegen/xai-grok-workspace/src/handle.rs` | adopt | `GB-A422-RAW` |
| 148 | `M` `crates/codegen/xai-grok-workspace/src/hub_server.rs` | adopt | `GB-A422-RAW` |
| 149 | `M` `crates/codegen/xai-grok-workspace/src/lib.rs` | adopt | `GB-A422-RAW` |
| 150 | `M` `crates/codegen/xai-grok-workspace/src/permission/manager.rs` | adopt | `GB-A422-RAW` |
| 151 | `M` `crates/codegen/xai-grok-workspace/src/permission/shell_access.rs` | adopt | `GB-A422-RAW` |
| 152 | `M` `crates/codegen/xai-grok-workspace/src/preview_supervisor.rs` | adopt | `GB-A422-RAW` |
| 153 | `M` `crates/codegen/xai-grok-workspace/src/rpc_envelope.rs` | adopt | `GB-A422-RAW` |
| 154 | `M` `crates/common/xai-computer-hub-sdk/src/connection.rs` | adopt | `GB-A422-RAW` |
| 155 | `M` `crates/common/xai-computer-hub-sdk/src/harness.rs` | adopt | `GB-A422-RAW` |
| 156 | `M` `crates/common/xai-computer-hub-sdk/src/metrics.rs` | adopt | `GB-A422-RAW` |
| 157 | `M` `crates/common/xai-computer-hub-sdk/src/server.rs` | adopt | `GB-A422-RAW` |
| 158 | `M` `crates/common/xai-grok-compaction/src/code_compaction/failure.rs` | adopt | `GB-A422-RAW` |
| 159 | `M` `crates/common/xai-grok-compaction/src/code_compaction/sample.rs` | adopt | `GB-A422-RAW` |
| 160 | `M` `crates/common/xai-grok-compaction/src/reminder.rs` | adopt | `GB-A422-RAW` |
| 161 | `M` `crates/common/xai-tool-protocol/src/frames.rs` | adopt | `GB-A422-RAW` |
| 162 | `M` `crates/common/xai-tool-protocol/src/lib.rs` | adopt | `GB-A422-RAW` |
| 163 | `M` `crates/common/xai-tool-types/src/lib.rs` | adopt | `GB-A422-RAW` |
| 164 | `M` `crates/common/xai-tool-types/src/task.rs` | adopt | `GB-A422-RAW` |
| 165 | `M` `prod/mc/cli-chat-proxy-types/src/team_managed_config_types.rs` | not applicable | `GB-A422-027` |
