# Upstream refresh audit — 2026-08-11

This ledger records the August 11 fetch and compatibility audit without advancing
Reviewed revisions. The pre-refresh Enhanced boundary is
`d971891ed28cc0aa0d64c1f3b61e7820b1ba9f96`, tree
`57400debec4fbcd8fdb4ce869de29694667563be`. Work was isolated on
`refresh/upstreams-20260811`.

The earlier `8a14c91` Grok adoption remains coherent but uncommitted and
unvalidated in the separate `refresh/upstreams-20260809` worktree. It
is evidence, not accepted history. This run does not overwrite it, claim its
changes as landed, or layer a newer source snapshot into that dirty worktree.

## Frozen source pins

All successful heads were pinned once at fetch time. Every changed successful
pin is a descendant of the recorded reviewed revision.

| Source | Reviewed | Pinned latest | Pinned tree | Result |
| --- | --- | --- | --- | --- |
| Grok Build | `afbc0fb710320c7add294c2106d447ecc3e3af2e` | `b13fa526f5112c0b20dad5f1f2300d3d3b127895` | `0f26f4082a3b9602ec712b218e177626b2bf72e5` | advanced, 3 commits / 238 raw paths |
| OpenAI Codex | `8e4b10446eed7bafb39d8a469f9be25a41f4864f` | `f2a6f2585c327251e6be647e47a3ba3e127ccff3` | `94a8c8ca952ad27deeed3380ce97188767945ff0` | advanced, 89 commits / 649 paths |
| OpenCode | `284214c78d32a09fd9c729bdefc07be50f74eb40` | `0d927ba03f36d7f87e3cdb2b6c1f34c44913a099` | `e749e4c946cf0eca237143472882a104bbbbcdb8` | advanced, 14 commits / 70 paths |
| Oh My Pi | `0e8142ad0e3189b5b51b49fd3434354683ba1b01` | `d3b22a0db6a4a0e2ef272a880e38286e0c466dc9` | `0bdda3ed30bea0cac79d338ed6a0fe728db8fb91` | advanced, 254 commits / 967 paths |
| Kimi Code | `437a1b8ba1b7e0f6662bdadc669564fdc58c3f5a` | `619564dcf9ee10a3cfbf7ecbc764c6b9b63fc91b` | `ca2e6ae3cf404ac9498e85896b466d4b85c154e4` | advanced, 11 commits / 106 paths |
| GLM-5 | `436efa09bc868a6922e307624189e7018406beb9` | `25206af860c4ac10f6411c597c574f9b1c00e53c` | `573d8342bcfc2e21d27e210c47a99a4604fc39ee` | advanced, link-only change |
| CodexBar | `22b24b885693e890af52df15c29f7ca024904c74` | `e5528d452d4f82cbd7e327246b9044e9c51d64e1` | `92bf650ca4076e6fabaf3936ca6f1563214b6743` | advanced, research only |
| models.dev | `ac01bd90859928691e2e8e65df5cf390ffb1539e` | `1d0f9ba5a49e916ff2dc97b23fbc76820ab258b3` | `90d5589cc499a612565f77f6c634839221e59b71` | advanced, catalog reference |
| Exa MCP server | `394f9210ed16d3e25d328e1e6db285824caedc04` | `e64c11f2d3b4400ffbda8ccdd9658a450cc9d270` | `569db78ece8c6a13f6f4afeefe05e569a57cb09e` | advanced, tool annotation only |

OpenCode Codex auth, Warp themes, Kimi CLI, Z.AI coding plugins, and the Z.AI
usage helper were unchanged. The tracked Z.AI Python SDK URL returned
`Repository not found`; its recorded reviewed/latest pin remains
`ca5109c0aa9bf173839be391b4b14aeadf9a9bf9`. No fallback repository
identity was inferred.

## Grok behavior inventory

The authoritative range is `afbc0fb7..b13fa526`. The prior candidate
range `afbc0fb7..8a14c91` contains the sixteen preserved-surface
behaviors inventoried in the August 9 candidate ledger: signal-safe memory-trace
waiting, standalone-worktree continuity, goal-mode Send Now, non-blocking
startup, bounded quit draining, skill-path suggestions, scroll anchoring,
headless HITL, bounded envrc evaluation, OOM attribution, bounded resume replay,
subagent-aware deletion, extracted-skill refresh, honest notification
acknowledgements, and managed-MCP cleanup. None is treated as closed until that
candidate is committed and passes its focused tests.

The incremental `8a14c91..b13fa526` range adds these observable
families:

- sticky worktree identity when the dashboard's live probe is incomplete;
- non-Linux sandbox build/identity correctness;
- logical Home/End behavior on soft-wrapped input;
- sanitized, bounded, manually pinned session rename/reset behavior across TUI,
  ACP, persistence, export, pull, and remote writeback;
- persistent-agent boot-slot race recovery;
- bounded and prewarmed Tokio blocking pools plus EAGAIN/runtime-build
  containment;
- read-versus-mutation RPC activity classification, server-version reporting,
  and idle-withhold semantics;
- session notification, task-result, and status ordering consistency;
- stable empty-summary and unfinished-task reminder wire behavior; and
- integration locks/tests needed to qualify those behaviors.

`SOURCE_REV` is not applicable because Enhanced owns release
identity. All other behavior is adopt-by-default and remains explicitly
deferred; no Grok acknowledgement is permitted for this pin.

## Durable open obligations

Target for every item is before the next source refresh, and no later than
2026-08-18. These obligations remain open until the acceptance criteria and
tests are committed on the candidate first-parent history.

| ID | Owner | Behavior | Blocker and user impact | Acceptance criteria and intended tests |
| --- | --- | --- | --- | --- |
| `GB-8A14-LANDING` | Grok parity | Land the sixteen `8a14c91` preserved-surface adoptions. | The implementation is still uncommitted in an isolated worktree; users lack a validated parity candidate. | Split/commit thematic changes; run focused session, ACP, headless, hook, MCP, TUI and workspace tests plus binary and strict ownership checks. |
| `GB-B13-WORKTREE` | TUI/session | Preserve sticky standalone-worktree identity. | Depends on the 8a14 worktree/session base; dashboard labels can regress after an incomplete live probe. | Port the OR-merge semantics and dashboard regressions. |
| `GB-B13-PORTABILITY` | Sandbox | Keep hook write-deny and child-network code correct on non-Linux targets. | Platform cfg changes must be reconciled with Enhanced sandbox policy; non-Linux builds may fail or expose inconsistent diagnostics. | Add Linux and non-Linux compile/tests without weakening hook identity checks. |
| `GB-B13-INPUT` | TUI input | Make Home/End operate on logical lines across soft wraps. | Needs terminal regression coverage; navigation can jump to visual rather than logical boundaries. | Port textarea behavior and soft-wrap tests. |
| `GB-B13-TITLES` | Session/TUI | Adopt bounded sanitized manual rename/reset, persistence, hydration, export and remote synchronization. | Crosses ACP, TUI, storage and remote seams; titles can be clobbered, stale, or contain control characters. | Port with scalar/byte bounds, control/bidi stripping, manual-pin metadata, FIFO persistence, reset-to-auto and authenticated version-gated remote tests. |
| `GB-B13-AGENT-BOOT` | Agent runtime | Reclaim abandoned persistent-agent boot slots safely. | Requires leader/runtime reconciliation; a dropped boot sender can strand waiters. | Port generation-guarded boot slots and stale-guard race tests. |
| `GB-B13-RUNTIME` | Runtime | Bound/prewarm blocking threads and contain EAGAIN/runtime-build failures. | Must preserve existing runtime topology; resource pressure can abort startup or detached work. | Port capped pools, prewarm/release, child re-exec and low-thread-limit tests. |
| `GB-B13-WORKSPACE-RPC` | Workspace | Classify RPC activity and report/gate by server version without idle races. | Protocol changes need backward compatibility; reads may incorrectly prevent idle or old servers may receive unsupported writes. | Port activity classes, version fields/gates, mutation stamping and idle-window tests. |
| `GB-B13-SESSION-EVENTS` | ACP/TUI | Preserve ordered session notifications, task results and status transitions. | Depends on the 8a14 event-loop changes; users can see stale or reordered status. | Port session-event, router, status and task-result regressions. |
| `GB-B13-WIRE` | Session protocol | Preserve explicit empty summaries and unfinished-task reminders. | Wire semantics must remain backward compatible; empty and omitted values can diverge. | Add serde/round-trip and interjection tests. |
| `GB-B13-INTEGRATION` | Grok parity | Reconcile remaining lock, docs, tests and cross-cutting integration paths. | These paths span multiple behavior families and cannot be accepted independently of their owners. | No unmatched raw path, passing focused suites, cargo fmt/check and strict manifest coverage. |
| `CDX-F2A6-METADATA` | Codex adapter | Send bounded provider-owned Responses turn metadata, including sandbox mode, without accepting reserved-key override. | Enhanced has scoped hosted-tool metadata but no equivalent normal-turn envelope; server-side policy context may be incomplete. | Implement provider-scoped header/body metadata with bounds/reserved keys and xAI/Kimi negative tests. |
| `CDX-F2A6-SAFETY` | Codex adapter | Parse safety-buffering signals from the current response.metadata shape. | The direct sampler does not classify this nested metadata; Codex safety buffering may be invisible. | Add bounded SSE parsing, precedence/malformed tests and no cross-provider handling. |
| `CDX-F2A6-IMAGE-LIMIT` | Codex hosted image | Surface image-generation quota exhaustion with reset metadata. | Current tool errors do not expose the typed image_gen usage-limit outcome. | Map only image_gen usage-limit errors, retain redaction, and test normal/other-limit failures. |
| `CDX-F2A6-MODEL-HISTORY` | Codex compaction | Retain the model identity needed when history spans model switches and compaction. | Local raw history is not yet proven equivalent to upstream response-item envelopes; fallback/compaction can lose model provenance. | Add provider/model/credential-bound history envelopes or prove equivalence through switch, persist, resume and compact tests. |

## Provider/reference audit

| Source | Upstream behavior | Local behavior | Outcome |
| --- | --- | --- | --- |
| OpenAI Codex | No login, refresh, logout, catalog, service-tier, retry, idempotency, timeout or cancellation wire change in the pinned range. The relevant adapter changes are Responses metadata, nested safety buffering, typed image quota failures and model-history envelopes. | Existing subscription auth/catalog isolation and Responses Lite `reasoning.context=all_turns` plus catalog-driven reasoning summaries remain intact. | Four adoption obligations above; app-server, gRPC code-mode, Guardian, plugin, skill and TUI architecture is not applicable. |
| OpenCode | DeepSeek sampling defaults, Muse prompt routing, default-model UI and generated assets changed. | No Codex OAuth, catalog, usage, retry or Responses wire seam changed. | not applicable to Codex interoperability |
| Kimi Code | Session-local profile isolation, V2 lifecycle/steer behavior, TUI task activity and KAP heartbeat changed. | Enhanced already owns per-session agent definitions; no authenticated catalog, headers, request/stream, thinking, hosted web, usage, retry or logout contract changed. | already equivalent for profile isolation; remaining app/server architecture not applicable |
| Oh My Pi | Added opaque-model Responses Lite context, default reasoning summaries, wider web providers and harness lifecycle fixes. | Enhanced already applies Lite `all_turns` independent of model name and uses catalog-driven `reasoning.summary`. | already equivalent for the two Codex wire checks; remaining replacement harness is non-normative |
| GLM-5 | Documentation link update only. | Research-only source. | not applicable |
| CodexBar | Z.AI pace/quota and unrelated application/provider changes. | Z.AI remains research only; no runtime provider is authorized. | not applicable |
| models.dev | Catalog additions/corrections, including Kimi/GLM metadata. | Authenticated first-party catalogs remain authoritative. | not applicable |
| Exa MCP server | Added `openWorldHint` annotations to search/fetch tool definitions. | No Exa request/response wire or credential behavior changed. | not applicable to runtime contract |

## Exhaustive Grok raw-path ledger

The canonical `git diff-tree --raw -r --no-renames --no-abbrev`
stream from tree `99e3e7c4d8a6c0214101c99e5cedded0325e96be` to
`0f26f4082a3b9602ec712b218e177626b2bf72e5` has SHA-256
`00c33feb938bf2fe9c6eb5d6f6e2d945a880f222a151fd1d337189be7a8dcaf6`. The 238 paths below are classified exactly once.

| Row | Raw status, modes and path | Outcome | Obligation |
| ---: | --- | --- | --- |
| 1 | `M` `100644->100644` `Cargo.lock` | temporarily deferred | `GB-B13-INTEGRATION` |
| 2 | `M` `100644->100644` `SOURCE_REV` | not applicable | `GB-B13-REL` |
| 3 | `M` `100644->100644` `crates/codegen/xai-grok-agent/src/builder.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 4 | `M` `100644->100644` `crates/codegen/xai-grok-agent/src/prompt/prompt_encrypted.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 5 | `M` `100644->100644` `crates/codegen/xai-grok-agent/templates/prompt.md` | temporarily deferred | `GB-8A14-LANDING` |
| 6 | `M` `100644->100644` `crates/codegen/xai-grok-pager-bin/src/main.rs` | temporarily deferred | `GB-B13-RUNTIME` |
| 7 | `A` `000000->100644` `crates/codegen/xai-grok-pager-pty-harness/tests/exit_timeout.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 8 | `M` `100644->100644` `crates/codegen/xai-grok-pager/docs/user-guide/04-slash-commands.md` | temporarily deferred | `GB-B13-INTEGRATION` |
| 9 | `M` `100644->100644` `crates/codegen/xai-grok-pager/docs/user-guide/05-configuration.md` | temporarily deferred | `GB-8A14-LANDING` |
| 10 | `M` `100644->100644` `crates/codegen/xai-grok-pager/docs/user-guide/07-mcp-servers.md` | temporarily deferred | `GB-8A14-LANDING` |
| 11 | `M` `100644->100644` `crates/codegen/xai-grok-pager/docs/user-guide/17-sessions.md` | temporarily deferred | `GB-B13-INTEGRATION` |
| 12 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/acp/leader_bridge.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 13 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/acp/mod.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 14 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/acp/spawn.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 15 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/acp_handler/background.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 16 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/acp_handler/mod.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 17 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/acp_handler/queue.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 18 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/acp_handler/session_notification.rs` | temporarily deferred | `GB-B13-SESSION-EVENTS` |
| 19 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/interjection.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 20 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/session_events.rs` | temporarily deferred | `GB-B13-SESSION-EVENTS` |
| 21 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/subagents.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 22 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/actions.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 23 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/agent_view/mod.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 24 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/agent_view/queue.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 25 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/agent_view/render.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 26 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/agent_view/session.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 27 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/app_view.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 28 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/dispatch/dashboard.rs` | temporarily deferred | `GB-B13-WORKTREE` |
| 29 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/dispatch/prompt.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 30 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/dispatch/queue.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 31 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/dispatch/router.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 32 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/dispatch/session/fork.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 33 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 34 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/dispatch/session/load.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 35 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/dispatch/session/modal.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 36 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/dispatch/task_result.rs` | temporarily deferred | `GB-B13-SESSION-EVENTS` |
| 37 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/dashboard.rs` | temporarily deferred | `GB-B13-WORKTREE` |
| 38 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/mod.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 39 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/prompt.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 40 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/router.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 41 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/fork.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 42 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/lifecycle.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 43 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/load.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 44 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/status.rs` | temporarily deferred | `GB-B13-SESSION-EVENTS` |
| 45 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/task_result.rs` | temporarily deferred | `GB-B13-SESSION-EVENTS` |
| 46 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/effects/helpers.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 47 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/effects/mod.rs` | temporarily deferred | `GB-B13-SESSION-EVENTS` |
| 48 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/event_loop.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 49 | `A` `000000->100644` `crates/codegen/xai-grok-pager/src/app/exit_timeout.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 50 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/mod.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 51 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/modals.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 52 | `A` `000000->100644` `crates/codegen/xai-grok-pager/src/app/session_load_barrier.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 53 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/session_startup.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 54 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/session_title_resolve_tests.rs` | temporarily deferred | `GB-B13-TITLES` |
| 55 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/signal_handler.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 56 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/subagent.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 57 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/git_info.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 58 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/headless.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 59 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/headless/ext_protocol.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 60 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/headless/ext_protocol_tests.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 61 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/headless_tests.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 62 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/mcp_cmd.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 63 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/memory_trace.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 64 | `A` `000000->100644` `crates/codegen/xai-grok-pager/src/memory_trace_signal_topology_tests.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 65 | `A` `000000->100644` `crates/codegen/xai-grok-pager/src/memory_trace_wait.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 66 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/models.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 67 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/scrollback/state/layout.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 68 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/scrollback/state/mod.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 69 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/slash/command.rs` | temporarily deferred | `GB-B13-TITLES` |
| 70 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/slash/commands/announcements.rs` | temporarily deferred | `GB-B13-TITLES` |
| 71 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/slash/commands/debug.rs` | temporarily deferred | `GB-B13-TITLES` |
| 72 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/slash/commands/docs.rs` | temporarily deferred | `GB-B13-TITLES` |
| 73 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/slash/commands/doctor.rs` | temporarily deferred | `GB-B13-TITLES` |
| 74 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/slash/commands/effort.rs` | temporarily deferred | `GB-B13-TITLES` |
| 75 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/slash/commands/mod.rs` | temporarily deferred | `GB-B13-TITLES` |
| 76 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/slash/commands/model.rs` | temporarily deferred | `GB-B13-TITLES` |
| 77 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/slash/commands/rename.rs` | temporarily deferred | `GB-B13-TITLES` |
| 78 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/slash/commands/theme.rs` | temporarily deferred | `GB-B13-TITLES` |
| 79 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/slash/commands/toggle_mouse_reporting.rs` | temporarily deferred | `GB-B13-TITLES` |
| 80 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/slash/commands/workflows.rs` | temporarily deferred | `GB-B13-TITLES` |
| 81 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/slash/mod.rs` | temporarily deferred | `GB-B13-TITLES` |
| 82 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/test_util.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 83 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/views/dashboard/state.rs` | temporarily deferred | `GB-B13-WORKTREE` |
| 84 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/views/mcps_modal.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 85 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/views/prompt_widget/mod.rs` | temporarily deferred | `GB-B13-TITLES` |
| 86 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/views/session_title.rs` | temporarily deferred | `GB-B13-TITLES` |
| 87 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/worktree_cmd/mod.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 88 | `M` `100644->100644` `crates/codegen/xai-grok-pager/tests/pty_e2e/rename_title_shows_in_prompt_border.rs` | temporarily deferred | `GB-B13-TITLES` |
| 89 | `A` `000000->100644` `crates/codegen/xai-grok-pager/tests/pty_e2e/scroll_anchor_holds_parked_marker_during_live_stream.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 90 | `M` `100644->100644` `crates/codegen/xai-grok-pager/tests/pty_e2e_scroll_selection.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 91 | `M` `100644->100644` `crates/codegen/xai-grok-sandbox/src/child_net.rs` | temporarily deferred | `GB-B13-PORTABILITY` |
| 92 | `M` `100644->100644` `crates/codegen/xai-grok-sandbox/src/hook_write_deny.rs` | temporarily deferred | `GB-B13-PORTABILITY` |
| 93 | `M` `100644->100644` `crates/codegen/xai-grok-sandbox/src/lib.rs` | temporarily deferred | `GB-B13-PORTABILITY` |
| 94 | `M` `100644->100644` `crates/codegen/xai-grok-shell-session-support/Cargo.toml` | temporarily deferred | `GB-8A14-LANDING` |
| 95 | `M` `100644->100644` `crates/codegen/xai-grok-shell-session-support/src/managed_mcp.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 96 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/app.rs` | temporarily deferred | `GB-B13-AGENT-BOOT` |
| 97 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/handlers/mod.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 98 | `A` `000000->100644` `crates/codegen/xai-grok-shell/src/agent/handlers/models.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 99 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/init.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 100 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/models.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 101 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/models/tests.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 102 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/acp_agent.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 103 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 104 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/folder_trust_prompt.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 105 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/mod.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 106 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/replay.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 107 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/replay_tests.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 108 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/session_lifecycle.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 109 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/session_setup.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 110 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/subagent_coordinator.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 111 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/tests.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 112 | `A` `000000->100644` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/tests/session_rename_tests.rs` | temporarily deferred | `GB-B13-TITLES` |
| 113 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/tests/subagent_spawn_context_tests.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 114 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/server.rs` | temporarily deferred | `GB-B13-AGENT-BOOT` |
| 115 | `A` `000000->100644` `crates/codegen/xai-grok-shell/src/agent/server_tests.rs` | temporarily deferred | `GB-B13-AGENT-BOOT` |
| 116 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/session_registry_client.rs` | temporarily deferred | `GB-B13-AGENT-BOOT` |
| 117 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 118 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 119 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/builtin.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 120 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/cli_models.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 121 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/extensions/mcp.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 122 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/extensions/notification.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 123 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/extensions/session_admin.rs` | temporarily deferred | `GB-B13-TITLES` |
| 124 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/mcp_doctor.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 125 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/remote/pull.rs` | temporarily deferred | `GB-B13-TITLES` |
| 126 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/remote/pull_smoke_test.rs` | temporarily deferred | `GB-B13-TITLES` |
| 127 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/remote/sync.rs` | temporarily deferred | `GB-B13-TITLES` |
| 128 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 129 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/goal.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 130 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/goal_support.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 131 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/hooks_plugins.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 132 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/interjection.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 133 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/mcp.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 134 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/prompt_queue.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 135 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/spawn.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 136 | `A` `000000->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/spawn_runtime_containment_tests.rs` | temporarily deferred | `GB-B13-RUNTIME` |
| 137 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/tool_calls.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 138 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/updates.rs` | temporarily deferred | `GB-B13-SESSION-EVENTS` |
| 139 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/cancel_running_task_tests.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 140 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/goal/goal_planner_e2e_tests.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 141 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/idle_resume_tests.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 142 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/inline_auto_compact_flow_tests.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 143 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/interjection_actor_tests.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 144 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/interjection_tests.rs` | temporarily deferred | `GB-B13-SESSION-EVENTS` |
| 145 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/memory_config_tests.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 146 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/prompt_queue_actor_tests.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 147 | `D` `100644->000000` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/reactive_managed_reauth_e2e_tests.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 148 | `D` `100644->000000` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/reactive_managed_reauth_tests.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 149 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/replay_buffer_send_update_tests.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 150 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/support.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 151 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/compaction.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 152 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/export.rs` | temporarily deferred | `GB-B13-TITLES` |
| 153 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/fork.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 154 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/goal_planner.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 155 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/goal_strategist.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 156 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/goal_summarizer.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 157 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/goal_tracker.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 158 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/handle.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 159 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/managed_mcp.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 160 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/mcp_dispatcher.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 161 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/mcp_servers.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 162 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/persistence.rs` | temporarily deferred | `GB-B13-TITLES` |
| 163 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/persistence_tests.rs` | temporarily deferred | `GB-B13-TITLES` |
| 164 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/storage/jsonl/mod.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 165 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/storage/mod.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 166 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/storage/relocation/mod.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 167 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/storage/relocation/tests.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 168 | `A` `000000->100644` `crates/codegen/xai-grok-shell/src/session/storage/replay.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 169 | `A` `000000->100644` `crates/codegen/xai-grok-shell/src/session/storage/replay_tests.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 170 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/storage/summary_write.rs` | temporarily deferred | `GB-B13-TITLES` |
| 171 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/summary.rs` | temporarily deferred | `GB-B13-TITLES` |
| 172 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/wire_tags.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 173 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/test_support/lsp_runtime.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 174 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/tools/notification_bridge.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 175 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/tools/tool_context.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 176 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/util/config/mcp.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 177 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/util/config/mcp_reenable.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 178 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/util/config/resolve/toolset.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 179 | `M` `100644->100644` `crates/codegen/xai-grok-shell/tests/session_fork_replay_memory.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 180 | `A` `000000->100644` `crates/codegen/xai-grok-shell/tests/test_mcp_doctor_isolation.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 181 | `M` `100644->100644` `crates/codegen/xai-grok-telemetry/Cargo.toml` | temporarily deferred | `GB-8A14-LANDING` |
| 182 | `M` `100644->100644` `crates/codegen/xai-grok-telemetry/src/startup.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 183 | `M` `100644->100644` `crates/codegen/xai-grok-test-support/README.md` | temporarily deferred | `GB-B13-INTEGRATION` |
| 184 | `M` `100644->100644` `crates/codegen/xai-grok-test-support/src/inference_override.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 185 | `M` `100644->100644` `crates/codegen/xai-grok-test-support/src/leader.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 186 | `M` `100644->100644` `crates/codegen/xai-grok-test-support/src/mock_server.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 187 | `M` `100644->100644` `crates/codegen/xai-grok-tools/src/computer/local/file_system.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 188 | `M` `100644->100644` `crates/codegen/xai-grok-tools/src/computer/local/mock_fs.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 189 | `M` `100644->100644` `crates/codegen/xai-grok-tools/src/computer/local/terminal.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 190 | `M` `100644->100644` `crates/codegen/xai-grok-tools/src/computer/types.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 191 | `M` `100644->100644` `crates/codegen/xai-grok-tools/src/implementations/grok_build/ask_user_question/format.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 192 | `M` `100644->100644` `crates/codegen/xai-grok-tools/src/implementations/grok_build/ask_user_question/mod.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 193 | `M` `100644->100644` `crates/codegen/xai-grok-tools/src/implementations/grok_build/read_file/mod.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 194 | `M` `100644->100644` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/backend.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 195 | `M` `100644->100644` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/coordinator.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 196 | `M` `100644->100644` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/coordinator_tests.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 197 | `M` `100644->100644` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/types.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 198 | `M` `100644->100644` `crates/codegen/xai-grok-tools/src/types/skill_discovery_tracker/conditional.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 199 | `M` `100644->100644` `crates/codegen/xai-grok-tools/src/types/skill_discovery_tracker/mod.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 200 | `A` `000000->100644` `crates/codegen/xai-grok-tools/src/types/skill_discovery_tracker/skill_path_suggestion.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 201 | `A` `000000->100644` `crates/codegen/xai-grok-tools/src/types/skill_discovery_tracker/skill_path_suggestion_tests.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 202 | `M` `100644->100644` `crates/codegen/xai-grok-workspace-client/Cargo.toml` | temporarily deferred | `GB-B13-WORKSPACE-RPC` |
| 203 | `M` `100644->100644` `crates/codegen/xai-grok-workspace-client/src/lib.rs` | temporarily deferred | `GB-B13-WORKSPACE-RPC` |
| 204 | `M` `100644->100644` `crates/codegen/xai-grok-workspace-types/src/rpc/agents_md.rs` | temporarily deferred | `GB-B13-WORKSPACE-RPC` |
| 205 | `M` `100644->100644` `crates/codegen/xai-grok-workspace-types/src/rpc/code_nav.rs` | temporarily deferred | `GB-B13-WORKSPACE-RPC` |
| 206 | `M` `100644->100644` `crates/codegen/xai-grok-workspace-types/src/rpc/export_github.rs` | temporarily deferred | `GB-B13-WORKSPACE-RPC` |
| 207 | `M` `100644->100644` `crates/codegen/xai-grok-workspace-types/src/rpc/fs.rs` | temporarily deferred | `GB-B13-WORKSPACE-RPC` |
| 208 | `M` `100644->100644` `crates/codegen/xai-grok-workspace-types/src/rpc/git.rs` | temporarily deferred | `GB-B13-WORKSPACE-RPC` |
| 209 | `M` `100644->100644` `crates/codegen/xai-grok-workspace-types/src/rpc/hooks.rs` | temporarily deferred | `GB-B13-WORKSPACE-RPC` |
| 210 | `M` `100644->100644` `crates/codegen/xai-grok-workspace-types/src/rpc/hunks.rs` | temporarily deferred | `GB-B13-WORKSPACE-RPC` |
| 211 | `M` `100644->100644` `crates/codegen/xai-grok-workspace-types/src/rpc/mod.rs` | temporarily deferred | `GB-B13-WORKSPACE-RPC` |
| 212 | `M` `100644->100644` `crates/codegen/xai-grok-workspace-types/src/rpc/repos.rs` | temporarily deferred | `GB-B13-WORKSPACE-RPC` |
| 213 | `M` `100644->100644` `crates/codegen/xai-grok-workspace-types/src/rpc/search.rs` | temporarily deferred | `GB-B13-WORKSPACE-RPC` |
| 214 | `M` `100644->100644` `crates/codegen/xai-grok-workspace-types/src/rpc/session.rs` | temporarily deferred | `GB-B13-WORKSPACE-RPC` |
| 215 | `M` `100644->100644` `crates/codegen/xai-grok-workspace-types/src/rpc/skills.rs` | temporarily deferred | `GB-B13-WORKSPACE-RPC` |
| 216 | `M` `100644->100644` `crates/codegen/xai-grok-workspace-types/src/rpc/workspace.rs` | temporarily deferred | `GB-B13-WORKSPACE-RPC` |
| 217 | `M` `100644->100644` `crates/codegen/xai-grok-workspace-types/src/rpc/worktree.rs` | temporarily deferred | `GB-B13-WORKSPACE-RPC` |
| 218 | `M` `100644->100644` `crates/codegen/xai-grok-workspace/Cargo.toml` | temporarily deferred | `GB-8A14-LANDING` |
| 219 | `M` `100644->100644` `crates/codegen/xai-grok-workspace/src/activity.rs` | temporarily deferred | `GB-B13-WORKSPACE-RPC` |
| 220 | `M` `100644->100644` `crates/codegen/xai-grok-workspace/src/bin/workspace_server.rs` | temporarily deferred | `GB-B13-WORKSPACE-RPC` |
| 221 | `M` `100644->100644` `crates/codegen/xai-grok-workspace/src/envrc.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 222 | `M` `100644->100644` `crates/codegen/xai-grok-workspace/src/handle.rs` | temporarily deferred | `GB-B13-WORKSPACE-RPC` |
| 223 | `M` `100644->100644` `crates/codegen/xai-grok-workspace/src/hub_server.rs` | temporarily deferred | `GB-B13-WORKSPACE-RPC` |
| 224 | `M` `100644->100644` `crates/codegen/xai-grok-workspace/src/permission/claude_settings.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 225 | `M` `100644->100644` `crates/codegen/xai-grok-workspace/src/permission/resolution.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 226 | `M` `100644->100644` `crates/codegen/xai-grok-workspace/src/session/git.rs` | temporarily deferred | `GB-B13-WORKTREE` |
| 227 | `M` `100644->100644` `crates/codegen/xai-grok-workspace/src/status_config.rs` | temporarily deferred | `GB-B13-WORKSPACE-RPC` |
| 228 | `M` `100644->100644` `crates/codegen/xai-grok-workspace/src/workspace_ops.rs` | temporarily deferred | `GB-B13-WORKSPACE-RPC` |
| 229 | `M` `100644->100644` `crates/codegen/xai-ratatui-textarea/src/textarea.rs` | temporarily deferred | `GB-B13-INPUT` |
| 230 | `M` `100644->100644` `crates/codegen/xai-tty-utils/Cargo.toml` | temporarily deferred | `GB-B13-RUNTIME` |
| 231 | `M` `100644->100644` `crates/codegen/xai-tty-utils/src/lib.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 232 | `M` `100644->100644` `crates/codegen/xai-tty-utils/src/runtime.rs` | temporarily deferred | `GB-B13-RUNTIME` |
| 233 | `A` `000000->100644` `crates/codegen/xai-tty-utils/src/runtime_eagain_tests.rs` | temporarily deferred | `GB-B13-RUNTIME` |
| 234 | `A` `000000->100644` `crates/codegen/xai-tty-utils/src/runtime_tests.rs` | temporarily deferred | `GB-B13-RUNTIME` |
| 235 | `M` `100644->100644` `crates/common/xai-computer-hub-sdk/src/error.rs` | temporarily deferred | `GB-8A14-LANDING` |
| 236 | `M` `100644->100644` `crates/common/xai-interjection-core/src/format.rs` | temporarily deferred | `GB-B13-WIRE` |
| 237 | `M` `100644->100644` `crates/common/xai-tool-protocol/src/frames.rs` | temporarily deferred | `GB-B13-WIRE` |
| 238 | `M` `100644->100644` `crates/common/xai-tracing/src/grpc_client.rs` | temporarily deferred | `GB-8A14-LANDING` |

## Validation and publication boundary

This refresh intentionally changes provenance and audit evidence only. No
provider credential was used, no authenticated payload was inspected, and no
source code was copied from an inspiration checkout. Reviewed revisions remain
unchanged. Because open Grok adoption obligations remain, the prospective
acknowledgement check, zero-tree-delta merge marker, publication, push, tag and
release stages are forbidden for this candidate.
