# Atomic Grok behavior inventory — 2026-07-25

This is the authoritative observable-behavior inventory for Grok Build `3af4d5d39897855bdcc74f23e690024a5dc05573`..`6e386420825bd44ae648c63e7c8cba12fcec9401`. It complements the exact 676-record raw-tree ledger in [`upstream-refresh-2026-07-25.md`](upstream-refresh-2026-07-25.md). Raw path ownership and observable parity are separate: a source file can support several behaviors with different outcomes.

The three immutable source commits are:

- `a5727c5960452e7527a154b25cb5bf00cda0545e` — IDs `GB-A572-*`;
- `69f0ba880aa98f55e3ac1dcc570e2f332f825fe2` — IDs `GB-69F0-*`; and
- `6e386420825bd44ae648c63e7c8cba12fcec9401` — IDs `GB-6E38-*`.

Current outcome count: **74 open `adopt`**, **17 closed `adopt`**, **7 closed `already equivalent`**, **1 closed `not applicable`**, **0 Grok temporary deferrals**, and **0 unclassified observable behaviors**. Open adoption is not a claim of parity and blocks advancing the reviewed Grok pin or creating an acknowledgement marker.

## Behaviors introduced by `a5727c5960…`

| Stable ID | Exact observable behavior | Upstream evidence cluster | Outcome |
| --- | --- | --- | --- |
| `GB-A572-001` | Present a non-blocking coding-data-sharing upsell without blocking ordinary agent use. | Pager privacy banner and agent notices; telemetry paths. | **adopt — open** |
| `GB-A572-002` | Consolidate actionable startup and environment remediation in Doctor. | Pager diagnostics, Doctor command, and startup paths. | **adopt — open** |
| `GB-A572-003` | In auto mode, evaluate fail-closed gate requests through the classifier instead of rejecting without a normal decision path. | Workspace permission auto-mode, preflight, manager, and resolution paths. | **adopt — closed** (`permission::manager::tests::auto_classifier_boundaries::fail_closed_gate_ask_defers_and_classifier_allow_runs`) |
| `GB-A572-004` | Coalesce concurrent marketplace-list fetches. | Shell marketplace and plugin-marketplace Git paths. | **adopt — open** |
| `GB-A572-005` | Remove a marketplace source by configured name, not only URL or path identity. | Pager plugin command, extensions modal, and shell marketplace paths. | **adopt — open** |
| `GB-A572-006` | Time out hung Git marketplace sources, keep refresh non-blocking, and let the modal recover. | Plugin-marketplace Git implementation and modal integration. | **adopt — open** |
| `GB-A572-007` | Carry a typed `error_kind` on failed workspace RPCs. | Workspace client, types, errors, hub, handle, server, and operations. | **adopt — open** |
| `GB-A572-009` | Report the real process exit code in completed background-shell notifications and final output. | Grok Build Bash implementation and notification output. | **already equivalent — closed (`E1`)** |
| `GB-A572-010` | Generate date-rollover reminders only for date-bearing templates. | Session reminder policy and tests. | **adopt — open** |
| `GB-A572-011` | Carry `toolOverrides` through wire types, session configuration, registry, and availability decisions. | Sampling types, tools API, shell session config, and tool registry. | **adopt — open** |
| `GB-A572-012` | Make a `Bash(git:*)` permission rule match every command in a chained shell command by prefix. | Workspace permission policy, shell access, sandbox paths, and tests. | **adopt — closed** (`configured_bash_git_allow_does_not_grant_chained_non_allowed_commands`) |
| `GB-A572-013` | Split prompt-trigger telemetry by trigger type and record classifier provenance. | Telemetry events and workspace auto-mode observability. | **adopt — open** |
| `GB-A572-014` | Give managed connector and MCP operations the upstream 75-second timeout while preserving bounded cancellation. | MCP servers and managed-MCP session support. | **adopt — open** |
| `GB-A572-015` | Let trusted recorded user approvals authorize a repeated equivalent action. | Workspace auto mode and pager ACP permission handling. | **adopt — closed** (`pre_decision_remember_gate_lets_grant_satisfy_ask_floor`) |
| `GB-A572-016` | Select and execute Doctor fixes from the TUI. | Pager diagnostics, Doctor dispatch, and early-dispatch tests. | **adopt — open** |
| `GB-A572-017` | Fall back to an ordinary permission prompt when the auto classifier times out or fails in transport. | Workspace auto-mode resolution and pager permission handling. | **adopt — closed** (`auto_classifier_transport_failure_reports_transport_error_source`; `auto_classifier_timeout_preserves_total_denial_limit`) |
| `GB-A572-018` | Drain completion events owned by the active session plus legacy unowned events while retaining foreign-session events. | Task-completion reminder buffer and shell turn loop. | **already equivalent — closed (`E2`)** |
| `GB-A572-020` | Identify storage and API requests as `client_identifier=grok-agent-sdk`. | File-utils storage client and shell credential construction. | **adopt — open** |
| `GB-A572-021` | Accept both workspace-teleport disable spellings and persist the canonical spelling. | Config load, resolve, persistence, settings, and teleport tests. | **adopt — open** |
| `GB-A572-022` | Journal one-shot scheduler occurrences durably so they do not repeat after restart. | Scheduler occurrence journal, actor, types, and tests. | **adopt — open** |
| `GB-A572-023` | Terminate a turn after 16 consecutive identical tool calls and reset the count when arguments change. | Shell run loop, doom-loop tests, and circuit breaker. | **adopt — open** |
| `GB-A572-024` | Copy checkpoint files and lineage when forking a compacted session so later rewind remains valid. | Pager fork and rewind, shell rewind storage, and workspace checkpoints. | **adopt — open** |
| `GB-A572-025` | Automatically focus a permission question that arrives while the user is in scrollback. | Pager ACP permission focus and PTY paths. | **adopt — closed** (`enqueue_while_scrollback_steals_focus_to_prompt`; `enqueue_while_scrollback_then_select_restores_scrollback`; `second_enqueue_does_not_resteal_if_user_returned_to_scrollback`) |
| `GB-A572-026` | Let one Esc cancel a running turn except when fullscreen Vim scrollback or an active overlay consumes Esc. | Pager input dispatch, shell cancellation, and Esc PTYs. | **adopt — open** |
| `GB-A572-027` | Accurately list Ctrl or Cmd+Z undo and redo bindings in keyboard help. | Shortcut-help view, docs, and undo-tip PTYs. | **adopt — open** |
| `GB-A572-027C` | Restore text, cursor, and elements through bounded core input undo and redo checkpoints. | `xai-ratatui-textarea` implementation and demo. | **already equivalent — closed (`E5`)** |
| `GB-A572-028` | Run macOS voice capture in a temporary helper process while preserving diagnostics, cancellation, permissions, and silence handling. | Voice capture subprocess, pipe, protocol, and pager voice paths. | **adopt — open** |
| `GB-A572-029` | Show active authentication mode and its management route in `/session-info` without revealing secrets. | Shell session-info, auth model, and pager presentation. | **adopt — open** |
| `GB-A572-030` | Install the official xAI npm binary under `$GROK_HOME/bin`. | Official `npm/grok` launcher, postinstall, and tests. | **not applicable — closed**: Enhanced must use fork-owned release metadata and assets. |
| `GB-A572-031` | Remove dashboard hover and click dead zones between rows. | Dashboard row, render, peek, state, and dispatch tests. | **adopt — open** |
| `GB-A572-032` | Point actionable startup warnings to `/doctor`. | Pager startup, diagnostics formatting, and Doctor tests. | **adopt — open** |
| `GB-A572-033` | Document and configure `[feedback.user]` author identity. | Pager configuration docs and shell feedback configuration. | **adopt — open** |
| `GB-A572-034` | Give bang commands the upstream one-hour timeout rather than the longer background timeout. | Grok and OpenCode Bash implementations and Bash-mode PTYs. | **adopt — open** |
| `GB-A572-035` | Prevent combine-queued editing from losing edits or releasing the hold during a running-edit race. | Pager queue edit and dispatch; shell prompt queue; queue PTYs. | **already equivalent — closed (`E3`)** |
| `GB-A572-036` | Recover interrupted or mirrored session relocations during startup. | Shell relocation storage, session startup, and pager lifecycle. | **adopt — open** |
| `GB-A572-037` | Resolve the remote privacy-notice rollout flag correctly. | Config types, agent settings, telemetry config, and privacy banner. | **adopt — open** |
| `GB-A572-038` | Avoid a computer-hub harness reference cycle that prevents idle connection eviction. | Computer-hub connection, borrow, demux, and harness. | **adopt — open** |
| `GB-A572-039` | Insert a newline for Shift+Enter and Alt+Enter while editing a queued prompt. | Pager queue input and queue PTYs. | **already equivalent — closed (`E3`)** |
| `GB-A572-040` | Honor imported Claude project permissions only after trusting the project folder. | Workspace Claude settings and folder trust; shell import flow. | **adopt — closed** (`untrusted_project_claude_permissions_are_not_honored`; `untrusted_project_config_toml_permissions_are_not_honored`) |
| `GB-A572-041` | Echo the originating `response.create.event_id` in ACP `response.created`. | Pager ACP tracker and notification; shell ACP updates and tests. | **adopt — open** |
| `GB-A572-042` | End startup with an actionable toast instead of hanging when session creation fails from a full disk. | Pager session startup and shell persistence and storage. | **adopt — open** |
| `GB-A572-044` | Enable dynamic workflows by default unless explicitly disabled. | Shell agent config, workflow ACP, views, docs, and tests. | **adopt — open** |
| `GB-A572-045` | Use a durable session-relocation transaction with interruption recovery. | Shell relocation state machine and persistence integration. | **adopt — open** |
| `GB-A572-047` | Surface and recover authentication failure during model-switch compaction. | Shell compaction and auth; pager model selection and auth tests. | **adopt — open** |
| `GB-A572-048` | Persist scheduler expiration and deletion state across restart. | Scheduler actor, journal, types, and tests. | **adopt — open** |
| `GB-A572-049` | Require explicit confirmation before removing MCP servers, plugins, hooks, or other extension items. | Extensions modal and shell extension handlers. | **adopt — open** |
| `GB-A572-050` | On expired auth during auto-compaction, log in, retry compaction, then submit the original prompt exactly once. | Pager auth and session events; shell session compaction. | **adopt — open** |
| `GB-A572-051` | Preserve hosted tools in recap requests while backend search is active. | Shell recap and tool-config tests. | **adopt — open** |

Four `a572…` raw changes are support rather than separate observable behaviors: redundant tonic and prost dependency removal, generated wire support for `toolOverrides`, shared test-process lifecycle helpers, and shared test-sandbox helpers. They remain covered by the raw-tree ledger and their governing behavior IDs.

## Behaviors introduced by `69f0ba880a…`

| Stable ID | Exact observable behavior | Upstream evidence cluster | Outcome |
| --- | --- | --- | --- |
| `GB-69F0-001` | Put `/ready` in a failed state after hub-connect failure for the configured dwell instead of reporting generic disconnection. | Workspace diagnostic server, workspace server, discovery, hub, and errors. | **adopt — open** |
| `GB-69F0-002` | Refresh expired or rejected xAI OIDC tokens, serialize refresh, adopt sibling-process rotations, and support 401 recovery. | Shell auth manager, credential provider, refresh, OIDC, and tests. | **already equivalent — closed (`E4`)** |
| `GB-69F0-003` | Record and replay ACP terminal output in order, including exit and termination boundaries. | Shell terminal recorder, watcher, adapter, ACP lifecycle, and pager rendering. | **adopt — open** |
| `GB-69F0-004` | Execute string-form auth-provider commands cross-platform instead of hard-coding `sh -c`. | Shell auth-provider implementation and tests. | **adopt — open** |
| `GB-69F0-005` | Make default `/resume` select native Grok sessions and hint when matching external sessions are hidden. | Pager session picker and load; shell unified session list and docs. | **adopt — open** |
| `GB-69F0-006` | Resolve `--resume` by session title as well as ID or path. | Pager session-title resolver, CLI, startup, and tests. | **adopt — open** |
| `GB-69F0-007` | Reject oversized app-builder deployment archives with an explicit reason. | Workspace deploy RPC, computer-hub SDK, and deploy tool. | **adopt — open** |
| `GB-69F0-008` | Render slash-command tag labels consistently from settings-driven data. | Config resolution and pager slash matcher, dropdown, and settings. | **adopt — open** |
| `GB-69F0-009` | Detect and fix supported tmux problems through Doctor. | Pager diagnostics, render tmux probe, fixes, and tests. | **adopt — open** |
| `GB-69F0-010` | Preserve explicit URL, query, environment headers, auth source, subprocess policy, and credential isolation for custom gateways across rebuilds. | Chat state, sampler client and config, shell model providers, and shell-env policy. | **adopt — open** |
| `GB-69F0-011` | Launch an opt-in persisted onboarding tour through `/tutorial` without blocking experienced users. | Tutorial docs, command, views, and welcome state. | **adopt — open** |
| `GB-69F0-012` | Enforce soft and required CLI version policies at runtime startup with appropriate warning or failure guidance. | Shell version config, update policy, startup, and PTYs. | **adopt — open** |
| `GB-69F0-013` | Keep privacy-banner environment overrides authoritative after live settings refresh. | Telemetry config, agent settings, and pager privacy dispatch. | **adopt — open** |
| `GB-69F0-014` | Let remote settings override the image-edit model independently of image generation and other providers. | Config types, agent config, image edit, and settings tests. | **adopt — open** |
| `GB-69F0-015` | Return cached profile, team, and principal fields from auth-info while the access token is expired. | Shell auth extension and auth manager. | **adopt — open** |
| `GB-69F0-016` | Expose stable queued-row edit controls and reconcile versioned, stale, or running-row edits safely. | Pager queue edit and shell shared prompt queue. | **already equivalent — closed (`E3`)** |
| `GB-69F0-017` | Keep orphan reconciliation fail-closed when no team identity is available. | Shell subagent coordinator and orphan tests. | **adopt — open** |
| `GB-69F0-018` | Disable Ctrl+Space and F8 voice activation through a setting without disabling unrelated input. | Voice config and pager input, settings, and voice tests. | **adopt — open** |
| `GB-69F0-019` | Invoke Linux `pw-record` with `--raw` for older PipeWire compatibility. | Linux voice capture. | **adopt — open** |
| `GB-69F0-020` | Reject invalid or unsafe marketplace Git URLs before fetching. | Plugin marketplace validation and pager and shell marketplace paths. | **adopt — open** |
| `GB-69F0-021` | Remove stale parameter names and removed tool names from generated and user tool documentation. | Tool schemas, templates, taxonomy, normalization, and pager docs. | **adopt — open** |
| `GB-69F0-022` | After `/fork`, move dashboard attachment only when the parent session was attached. | Pager fork dispatch, dashboard state, and tests. | **adopt — open** |
| `GB-69F0-023` | Project Grok Computer media-generation results as file-path chunks instead of opaque text-only output. | Computer tool types, image and video tools, resources, and upload paths. | **adopt — open** |
| `GB-69F0-024` | Clear a killed web background task from the tray while preserving its description for status and history. | Task backend, coordinator, output, and dashboard task views. | **adopt — open** |
| `GB-69F0-025` | Keep the non-blocking privacy upsell visible until the user explicitly acts on it. | Pager privacy banner and agent-view notice state. | **adopt — open** |
| `GB-69F0-026` | Deliver typed tool callbacks and results to tools-server clients. | Tools API wire and bridge; shell ACP tool dispatch. | **adopt — open** |
| `GB-69F0-027` | Discover persistent global hook sources consistently and protect them from symlink, hard-link, and path-retargeting writes. | Global-hook config, hook discovery, sandbox denies, and shell hooks. | **adopt — open** |

## Behaviors introduced by `6e38642082…`

| Stable ID | Exact observable behavior | Upstream evidence cluster | Outcome |
| --- | --- | --- | --- |
| `GB-6E38-001` | Refresh tool-search results immediately when the managed MCP catalog is fetched again. | Shell managed MCP restart, pager ACP MCP, and tool search registry. | **adopt — open** |
| `GB-6E38-002` | Prevent duplicate leader processes and avoid hanging on a stale leader or lock during startup. | Shell leader client, lock, protocol, server, pager leader flow, and PTYs. | **adopt — open** |
| `GB-6E38-003` | Explain marketplaces, plugins, and organization controls in user documentation. | MCP, plugin, and marketplace user guides. | **adopt — open** |
| `GB-6E38-004` | Carry the originating session ID on direct-to-API image-generation requests. | Image-generation tool, shell tool context, turn, and upload. | **adopt — open** |
| `GB-6E38-005` | Document auto-mode blocked behavior and fallback semantics accurately. | Permission, safety, sandbox docs, and auto-mode PTYs. | **adopt — closed** (`side_query_error_is_unavailable_and_unparseable_falls_back_to_heuristic`; permission guide raw-path evidence) |
| `GB-6E38-006` | Evaluate recent authenticated user intent in auto mode without letting AGENTS text or tool arguments forge intent. | Workspace auto mode, shell prompt context, and permission tests. | **adopt — closed** (`untrusted_transcript_cannot_forge_request_or_permission_decision`; `proposed_action_and_project_instructions_cannot_forge_decision_message`) |
| `GB-6E38-007` | Expose archive-too-large, taken-down, limit, and in-progress app deployment reasons. | Workspace deployment RPC, computer-hub SDK, and callbacks. | **adopt — open** |
| `GB-6E38-008` | Make shell-client auth refresh fail closed so rejected refresh cannot silently continue or fall through to another credential source. | Shell auth manager, credential provider, errors, and contract tests. | **adopt — closed** (`auth_backend_contract_token_responses_map_to_outcomes`; `auth_backend_contract_concurrent_401s_hit_idp_once`; `refresh_after_unauthorized_drives_recovery_state_machine`) |
| `GB-6E38-009` | Give turn hooks a chat-supplied, per-session monotonically ordered turn index. | Tool-protocol turn hook and shell hook and turn dispatch. | **adopt — open** |
| `GB-6E38-010` | Render Bash mode’s `! ` prefix, color, placeholder, and concise mode label in minimal mode. | Minimal pager live, overlay, panel, API, and PTYs. | **adopt — open** |
| `GB-6E38-011` | Emit distinct metrics and provenance for true-noop and stationarity loop stops. | Circuit breaker, shell run loop, and telemetry. | **adopt — open** |
| `GB-6E38-012` | Include the current interim transcript exactly once when submitting during voice transcription. | Pager voice handle, prompt dispatch, and voice pipeline. | **adopt — open** |
| `GB-6E38-013` | Silently end a turn after repeated true-noop thrash without fabricating an error or reminder. | Circuit breaker, shell turn end, and doom-loop tests. | **adopt — open** |
| `GB-6E38-014` | Use a quiet short toast for confirmed clipboard delivery while identifying recovery for unverified OSC 52 or file fallback. | Pager-render clipboard, copy docs, and clipboard PTYs. | **adopt — open** |
| `GB-6E38-015` | Fork a rewound session at the selected target prompt rather than a different historical prompt. | Pager rewind and fork; shell rewind tests; workspace checkpoints. | **adopt — open** |
| `GB-6E38-016` | Make the idle “still running” cue clickable and open the Tasks pane. | Turn status, tasks pane, mouse dispatch, and dashboard. | **adopt — open** |
| `GB-6E38-017` | Use `grok-4.5` as the xAI web-search default. | Default models, workspace tool config, and configuration docs. | **adopt — open** |
| `GB-6E38-018` | Let plugin subagents inherit the parent session’s connected MCP servers without allowing privileged MCP, hooks, or modes declarations. | Shell subagent request, spawn, coordinator, tests, and docs. | **adopt — open** |
| `GB-6E38-019` | Emit the no-op end-turn reminder only when system reminders are enabled. | Shell reminders, turn end, and reminder-policy tests. | **adopt — open** |
| `GB-6E38-020` | Emit credential-safe lifecycle telemetry for custom-gateway bridge create, replace, shutdown, and failure. | Shell model providers, agent rebuild, sampler state, and telemetry. | **adopt — open** |
| `GB-6E38-021` | Keep finalized voice text editable while voice capture and transcription remain open. | Pager voice handle, prompt dispatch, and voice pipeline. | **adopt — open** |
| `GB-6E38-022` | Move token-carrier metadata to turn-commit events and carry per-turn origin context through replay and commit. | Chat state, shell ACP turn updates, and tool-protocol hooks. | **adopt — open** |
| `GB-6E38-023` | Raise workflow scratch quotas to 10 MiB per file and 64 MiB total and allow failed runs to resume. | Workflow host service, manager, tracker, engine, and journal. | **adopt — open** |
| `GB-6E38-024` | Auto-advance workflow phases, show live agent status, and remove the obsolete budget meter. | Pager workflow overlay, shell workflow ingest, tracker, and engine. | **adopt — open** |

## Local equivalence evidence

### `E1` — completed background-shell exit codes

The local Grok Build Bash implementation sends `result.exit_code` in the completed notification and preserves it in `BashOutput`: [`xai-grok-tools/src/implementations/grok_build/bash/mod.rs`](../crates/codegen/xai-grok-tools/src/implementations/grok_build/bash/mod.rs). This closes only `GB-A572-009`; the one-hour bang-command timeout remains open.

### `E2` — session-owned completion drains

The local completion-buffer test proves the owning session drains its events plus legacy unowned events while foreign-session events remain buffered: [`xai-grok-tools/src/reminders/task_completion.rs`](../crates/codegen/xai-grok-tools/src/reminders/task_completion.rs). This closes only `GB-A572-018`.

### `E3` — queued prompt editing

The local pager and shell already use stable local and server IDs, one stash and restore owner, versioned edits and removals, running-row protection, newline insertion for Shift or Alt+Enter, and preservation when send-now becomes a benign no-op: [`xai-grok-pager/src/app/queue_edit.rs`](../crates/codegen/xai-grok-pager/src/app/queue_edit.rs) and [`xai-grok-shell/src/session/acp_session_impl/prompt_queue.rs`](../crates/codegen/xai-grok-shell/src/session/acp_session_impl/prompt_queue.rs). This closes `GB-A572-035`, `GB-A572-039`, and `GB-69F0-016`.

### `E4` — shell OIDC refresh

The local shell centralizes refresh mutation, serializes concurrent refreshes, adopts sibling-process rotations, records permanent failure, and exposes 401 recovery through the shell credential provider: [`auth/manager.rs`](../crates/codegen/xai-grok-shell/src/auth/manager.rs) and [`auth/credential_provider.rs`](../crates/codegen/xai-grok-shell/src/auth/credential_provider.rs). This closes `GB-69F0-002`, but not expired-profile projection, cross-platform auth commands, compact replay, or the stronger fail-closed shell-client contract.

### `E5` — core undo and redo

The local textarea binds undo and redo and restores text, cursor, and elements through bounded checkpoints: [`xai-ratatui-textarea/src/textarea.rs`](../crates/codegen/xai-ratatui-textarea/src/textarea.rs). This closes `GB-A572-027C`; shortcut-help labels remain open.

## Decisive open-parity evidence

The 91 open rows are not merely missing proof. Representative local contradictions include:

- relocation storage is explicitly inert, so relocation recovery and durable relocation remain open;
- plugin subagents deliberately discard the parent MCP pool, opposite `GB-6E38-018`;
- auth-info uses `current()`, which filters expired credentials, rather than `current_or_expired()`;
- string-form auth-provider commands still hard-code `sh -c`;
- the local xAI web-search default remains `grok-4.20-multi-agent`, not `grok-4.5`;
- workflows default off, retain 1 MiB and 8 MiB quotas, and treat failed runs as non-resumable;
- workspace readiness has no failed state or dwell contract;
- app deployment remains a disabled stub, which does not make its user-visible contracts inapplicable; and
- Linux `pw-record` does not receive `--raw`.

Every open row must gain implementation and contract-test evidence before the reviewed Grok pin can advance. A later ledger must carry these stable IDs forward or cite the closing commit and tests. No acknowledgement record or two-parent marker is eligible for `6e386420825bd44ae648c63e7c8cba12fcec9401`.
