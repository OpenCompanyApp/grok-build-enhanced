# Grok Build 19d4 exhaustive refresh evidence — 2026-08-20

This is the digest-bound review evidence for the audited Grok Build refresh used by Grok Build Enhanced. The review compares immutable raw tree entries rather than assuming shared history.

## Immutable source identities

- source ID: `grok-build-upstream`
- reviewed commit: `9fabadea800fa6e2ed8ec91c4f45f02b7e2504f4`
- reviewed tree: `668a9b611622a7571c7b86297bcc80838e5c02e3`
- target commit: `19d42e35c07a9c9244f03f6df0c4c353f970d4f9`
- target tree: `0ffa95d769d4d7b1f04c8d60d3305946471b7559`
- target parent: `d92c5b0b8582fda358de1f97446aa74af44a464f`
- exact `git diff --no-renames --raw --abbrev=40` SHA-256: `9b7088fea90e122cb3d83703aa2359a139dbef70bee60ad8bf64fa611982f4a5`
- generated target/root `Cargo.toml` SHA-256: `d2be467e684dc97beb0cc6054053c8a1997bbfdd298011ba3ee2fc93208368c7`

## Review outcome

All 231 changed raw paths are classified. The 226 applicable paths are adopted into Enhanced's preserved Grok surfaces. They cover refs-only session HEAD resolution; projected clones; hook environment contracts; MCP icons and consent recording; durable subagent attempt recovery; video ZDR behavior; queued goal input; browser-style selection; an optional command or structured status line; objective-named goal CI oracles; remembered and granular permissions; model-family compaction; prompt pinning; sandbox and scheduled-loop deletion; mail links; subagent tool isolation; sibling-worktree safety; two-tier stationarity; paused workflows; bounded sibling-token adoption; configurable startup timeout; and preview-state long polling.

The five release-only paths are not applicable: `SOURCE_REV`, the official xAI binary/version manifest changes, and official 1.0.6 changelog payloads do not replace Enhanced's fork-owned versions, release notes, or update routes. No path remains temporarily deferred. Direct Codex, Kimi Code, xAI, and custom-provider credential ownership remains isolated; no raw credential value, token prefix, account identifier, FedRAMP state, or provider-private response is added to diagnostics.

The prior-refresh residue audit examined all retained refresh worktrees. The August 9 dirty worktree remains preserved as forensic evidence: 153 of its 160 paths are reachable in the published first-parent history, five are superseded audit artifacts, and two absent paths were documentation-index links for already-landed dashboard and monitoring pages. Those two links are adopted in this refresh. The cumulative obligation ledger retains 120 closed and eight offline-qualified obligations, with zero open or temporarily deferred Grok obligations.

## Complete 231 raw-path ledger

| Row | Raw path | Outcome | Evidence |
| ---: | --- | --- | --- |
| 1 | `M` `Cargo.lock` | adopt | `GB-19D-INTEGRATION` |
| 2 | `M` `Cargo.toml` | adopt | `GB-19D-INTEGRATION` |
| 3 | `M` `SOURCE_REV` | not applicable | `GB-19D-RELEASE` |
| 4 | `M` `crates/codegen/xai-chat-state/src/compaction_utils.rs` | adopt | `GB-19D-INTEGRATION` |
| 5 | `M` `crates/codegen/xai-chat-state/src/compaction_utils_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 6 | `M` `crates/codegen/xai-chat-state/src/handle.rs` | adopt | `GB-19D-INTEGRATION` |
| 7 | `M` `crates/codegen/xai-fast-worktree/src/api.rs` | adopt | `GB-19D-INTEGRATION` |
| 8 | `M` `crates/codegen/xai-fast-worktree/src/api/gc.rs` | adopt | `GB-19D-INTEGRATION` |
| 9 | `M` `crates/codegen/xai-fast-worktree/src/auto_gc.rs` | adopt | `GB-19D-INTEGRATION` |
| 10 | `M` `crates/codegen/xai-fast-worktree/src/git/mod.rs` | adopt | `GB-19D-INTEGRATION` |
| 11 | `M` `crates/codegen/xai-fast-worktree/src/git/safety/git_dir.rs` | adopt | `GB-19D-INTEGRATION` |
| 12 | `M` `crates/codegen/xai-fast-worktree/src/git/worktree.rs` | adopt | `GB-19D-INTEGRATION` |
| 13 | `M` `crates/codegen/xai-grok-agent/src/builder.rs` | adopt | `GB-19D-INTEGRATION` |
| 14 | `M` `crates/codegen/xai-grok-agent/src/plugins/hooks_adapter.rs` | adopt | `GB-19D-INTEGRATION` |
| 15 | `M` `crates/codegen/xai-grok-config-types/src/lib.rs` | adopt | `GB-19D-INTEGRATION` |
| 16 | `M` `crates/codegen/xai-grok-config/src/config_override.rs` | adopt | `GB-19D-INTEGRATION` |
| 17 | `M` `crates/codegen/xai-grok-hooks/src/config.rs` | adopt | `GB-19D-INTEGRATION` |
| 18 | `M` `crates/codegen/xai-grok-hooks/src/env_expand.rs` | adopt | `GB-19D-INTEGRATION` |
| 19 | `M` `crates/codegen/xai-grok-hooks/src/runner/command.rs` | adopt | `GB-19D-INTEGRATION` |
| 20 | `M` `crates/codegen/xai-grok-hooks/src/runner/http.rs` | adopt | `GB-19D-INTEGRATION` |
| 21 | `M` `crates/codegen/xai-grok-markdown/src/url_scan.rs` | adopt | `GB-19D-INTEGRATION` |
| 22 | `M` `crates/codegen/xai-grok-mcp/src/servers.rs` | adopt | `GB-19D-INTEGRATION` |
| 23 | `M` `crates/codegen/xai-grok-mcp/src/servers_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 24 | `M` `crates/codegen/xai-grok-pager-bin/Cargo.toml` | not applicable | `GB-19D-RELEASE` |
| 25 | `M` `crates/codegen/xai-grok-pager-bin/src/main.rs` | adopt | `GB-19D-INTEGRATION` |
| 26 | `M` `crates/codegen/xai-grok-pager-render/src/appearance/mod.rs` | adopt | `GB-19D-INTEGRATION` |
| 27 | `M` `crates/codegen/xai-grok-pager-render/src/render/osc8.rs` | adopt | `GB-19D-INTEGRATION` |
| 28 | `M` `crates/codegen/xai-grok-pager/Cargo.toml` | adopt | `GB-19D-INTEGRATION` |
| 29 | `M` `crates/codegen/xai-grok-pager/docs/custom-hooks.md` | adopt | `GB-19D-INTEGRATION` |
| 30 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/03-keyboard-shortcuts.md` | adopt | `GB-19D-INTEGRATION` |
| 31 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/05-configuration.md` | adopt | `GB-19D-INTEGRATION` |
| 32 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/10-hooks.md` | adopt | `GB-19D-INTEGRATION` |
| 33 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/16-subagents.md` | adopt | `GB-19D-INTEGRATION` |
| 34 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/22-permissions-and-safety.md` | adopt | `GB-19D-INTEGRATION` |
| 35 | `A` `crates/codegen/xai-grok-pager/docs/user-guide/25-status-line.md` | adopt | `GB-19D-INTEGRATION` |
| 36 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/README.md` | adopt | `GB-19D-INTEGRATION` |
| 37 | `M` `crates/codegen/xai-grok-pager/src/acp/mod.rs` | adopt | `GB-19D-INTEGRATION` |
| 38 | `M` `crates/codegen/xai-grok-pager/src/acp/tracker.rs` | adopt | `GB-19D-INTEGRATION` |
| 39 | `M` `crates/codegen/xai-grok-pager/src/acp/tracker_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 40 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/session_notification.rs` | adopt | `GB-19D-INTEGRATION` |
| 41 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/interactions.rs` | adopt | `GB-19D-INTEGRATION` |
| 42 | `M` `crates/codegen/xai-grok-pager/src/app/actions.rs` | adopt | `GB-19D-INTEGRATION` |
| 43 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/links.rs` | adopt | `GB-19D-INTEGRATION` |
| 44 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/mod.rs` | adopt | `GB-19D-INTEGRATION` |
| 45 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/paste.rs` | adopt | `GB-19D-INTEGRATION` |
| 46 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/prompt.rs` | adopt | `GB-19D-INTEGRATION` |
| 47 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/queue.rs` | adopt | `GB-19D-INTEGRATION` |
| 48 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/render.rs` | adopt | `GB-19D-INTEGRATION` |
| 49 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/session.rs` | adopt | `GB-19D-INTEGRATION` |
| 50 | `A` `crates/codegen/xai-grok-pager/src/app/agent_view/task_status_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 51 | `M` `crates/codegen/xai-grok-pager/src/app/app_view.rs` | adopt | `GB-19D-INTEGRATION` |
| 52 | `M` `crates/codegen/xai-grok-pager/src/app/app_view_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 53 | `A` `crates/codegen/xai-grok-pager/src/app/connect_timeout.rs` | adopt | `GB-19D-INTEGRATION` |
| 54 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/task_result.rs` | adopt | `GB-19D-INTEGRATION` |
| 55 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/mod.rs` | adopt | `GB-19D-INTEGRATION` |
| 56 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/settings.rs` | adopt | `GB-19D-INTEGRATION` |
| 57 | `A` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/status_line.rs` | adopt | `GB-19D-INTEGRATION` |
| 58 | `M` `crates/codegen/xai-grok-pager/src/app/effects/mod.rs` | adopt | `GB-19D-INTEGRATION` |
| 59 | `M` `crates/codegen/xai-grok-pager/src/app/event_loop.rs` | adopt | `GB-19D-INTEGRATION` |
| 60 | `M` `crates/codegen/xai-grok-pager/src/app/mod.rs` | adopt | `GB-19D-INTEGRATION` |
| 61 | `M` `crates/codegen/xai-grok-pager/src/app/mouse.rs` | adopt | `GB-19D-INTEGRATION` |
| 62 | `M` `crates/codegen/xai-grok-pager/src/app/queue_edit.rs` | adopt | `GB-19D-INTEGRATION` |
| 63 | `M` `crates/codegen/xai-grok-pager/src/app/signal_handler.rs` | adopt | `GB-19D-INTEGRATION` |
| 64 | `M` `crates/codegen/xai-grok-pager/src/app/startup_failure/render.rs` | adopt | `GB-19D-INTEGRATION` |
| 65 | `A` `crates/codegen/xai-grok-pager/src/app/status_line.rs` | adopt | `GB-19D-INTEGRATION` |
| 66 | `A` `crates/codegen/xai-grok-pager/src/app/status_line/command.rs` | adopt | `GB-19D-INTEGRATION` |
| 67 | `A` `crates/codegen/xai-grok-pager/src/app/status_line/command_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 68 | `A` `crates/codegen/xai-grok-pager/src/app/status_line/metrics.rs` | adopt | `GB-19D-INTEGRATION` |
| 69 | `A` `crates/codegen/xai-grok-pager/src/app/status_line/metrics_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 70 | `A` `crates/codegen/xai-grok-pager/src/app/status_line_policy.rs` | adopt | `GB-19D-INTEGRATION` |
| 71 | `A` `crates/codegen/xai-grok-pager/src/app/status_line_policy_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 72 | `A` `crates/codegen/xai-grok-pager/src/app/status_line_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 73 | `M` `crates/codegen/xai-grok-pager/src/docs.rs` | adopt | `GB-19D-INTEGRATION` |
| 74 | `M` `crates/codegen/xai-grok-pager/src/lib.rs` | adopt | `GB-19D-INTEGRATION` |
| 75 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/markdown_content.rs` | adopt | `GB-19D-INTEGRATION` |
| 76 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/edit.rs` | adopt | `GB-19D-INTEGRATION` |
| 77 | `M` `crates/codegen/xai-grok-pager/src/scrollback/state/layout.rs` | adopt | `GB-19D-INTEGRATION` |
| 78 | `M` `crates/codegen/xai-grok-pager/src/scrollback/state/mod.rs` | adopt | `GB-19D-INTEGRATION` |
| 79 | `M` `crates/codegen/xai-grok-pager/src/scrollback/state/nav.rs` | adopt | `GB-19D-INTEGRATION` |
| 80 | `A` `crates/codegen/xai-grok-pager/src/scrollback/state/pin_reserve.rs` | adopt | `GB-19D-INTEGRATION` |
| 81 | `M` `crates/codegen/xai-grok-pager/src/settings/defs.rs` | adopt | `GB-19D-INTEGRATION` |
| 82 | `M` `crates/codegen/xai-grok-pager/src/settings/registry.rs` | adopt | `GB-19D-INTEGRATION` |
| 83 | `M` `crates/codegen/xai-grok-pager/src/unified_log.rs` | adopt | `GB-19D-INTEGRATION` |
| 84 | `M` `crates/codegen/xai-grok-pager/src/views/agent.rs` | adopt | `GB-19D-INTEGRATION` |
| 85 | `M` `crates/codegen/xai-grok-pager/src/views/agent_status.rs` | adopt | `GB-19D-INTEGRATION` |
| 86 | `A` `crates/codegen/xai-grok-pager/src/views/agent_status_task_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 87 | `M` `crates/codegen/xai-grok-pager/src/views/dashboard/state.rs` | adopt | `GB-19D-INTEGRATION` |
| 88 | `M` `crates/codegen/xai-grok-pager/src/views/dashboard/state_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 89 | `M` `crates/codegen/xai-grok-pager/src/views/mod.rs` | adopt | `GB-19D-INTEGRATION` |
| 90 | `M` `crates/codegen/xai-grok-pager/src/views/prompt_widget/mod.rs` | adopt | `GB-19D-INTEGRATION` |
| 91 | `M` `crates/codegen/xai-grok-pager/src/views/prompt_widget/tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 92 | `A` `crates/codegen/xai-grok-pager/src/views/status_line/mod.rs` | adopt | `GB-19D-INTEGRATION` |
| 93 | `A` `crates/codegen/xai-grok-pager/src/views/status_line/sanitize.rs` | adopt | `GB-19D-INTEGRATION` |
| 94 | `A` `crates/codegen/xai-grok-pager/src/views/status_line/sanitize_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 95 | `A` `crates/codegen/xai-grok-pager/src/views/status_line/segments.rs` | adopt | `GB-19D-INTEGRATION` |
| 96 | `A` `crates/codegen/xai-grok-pager/src/views/status_line/segments_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 97 | `A` `crates/codegen/xai-grok-pager/src/views/status_line/tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 98 | `M` `crates/codegen/xai-grok-pager/src/views/tasks_pane.rs` | adopt | `GB-19D-INTEGRATION` |
| 99 | `A` `crates/codegen/xai-grok-pager/src/views/tasks_pane_status_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 100 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/common.rs` | adopt | `GB-19D-INTEGRATION` |
| 101 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/connect_ui_timeout_env_override.rs` | adopt | `GB-19D-INTEGRATION` |
| 102 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_feedback_session_gate_and_pane.rs` | adopt | `GB-19D-INTEGRATION` |
| 103 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/shift_selection_key_encodings.rs` | adopt | `GB-19D-INTEGRATION` |
| 104 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e_clipboard.rs` | adopt | `GB-19D-INTEGRATION` |
| 105 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e_smoke.rs` | adopt | `GB-19D-INTEGRATION` |
| 106 | `M` `crates/codegen/xai-grok-pager/tests/settings_e2e.rs` | adopt | `GB-19D-INTEGRATION` |
| 107 | `M` `crates/codegen/xai-grok-shared/Cargo.toml` | adopt | `GB-19D-INTEGRATION` |
| 108 | `M` `crates/codegen/xai-grok-shared/src/ui_config.rs` | adopt | `GB-19D-INTEGRATION` |
| 109 | `M` `crates/codegen/xai-grok-shell/CHANGELOG.md` | adopt | `GB-19D-INTEGRATION` |
| 110 | `M` `crates/codegen/xai-grok-shell/Cargo.toml` | adopt | `GB-19D-INTEGRATION` |
| 111 | `A` `crates/codegen/xai-grok-shell/changelogs/1.0.6.json` | not applicable | `GB-19D-RELEASE` |
| 112 | `A` `crates/codegen/xai-grok-shell/changelogs/1.0.6.md` | not applicable | `GB-19D-RELEASE` |
| 113 | `M` `crates/codegen/xai-grok-shell/src/agent/config.rs` | adopt | `GB-19D-INTEGRATION` |
| 114 | `M` `crates/codegen/xai-grok-shell/src/agent/config_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 115 | `M` `crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs` | adopt | `GB-19D-INTEGRATION` |
| 116 | `M` `crates/codegen/xai-grok-shell/src/agent/models.rs` | adopt | `GB-19D-INTEGRATION` |
| 117 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/acp_agent.rs` | adopt | `GB-19D-INTEGRATION` |
| 118 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs` | adopt | `GB-19D-INTEGRATION` |
| 119 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/mod.rs` | adopt | `GB-19D-INTEGRATION` |
| 120 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/session_setup.rs` | adopt | `GB-19D-INTEGRATION` |
| 121 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/subagent_coordinator.rs` | adopt | `GB-19D-INTEGRATION` |
| 122 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 123 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/tests/subagent_spawn_context_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 124 | `M` `crates/codegen/xai-grok-shell/src/agent/relay.rs` | adopt | `GB-19D-INTEGRATION` |
| 125 | `A` `crates/codegen/xai-grok-shell/src/agent/subagent/attempt_store/accounting.rs` | adopt | `GB-19D-INTEGRATION` |
| 126 | `A` `crates/codegen/xai-grok-shell/src/agent/subagent/attempt_store/accounting_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 127 | `M` `crates/codegen/xai-grok-shell/src/agent/subagent/attempt_store/codec.rs` | adopt | `GB-19D-INTEGRATION` |
| 128 | `M` `crates/codegen/xai-grok-shell/src/agent/subagent/attempt_store/codec_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 129 | `A` `crates/codegen/xai-grok-shell/src/agent/subagent/attempt_store/completion.rs` | adopt | `GB-19D-INTEGRATION` |
| 130 | `A` `crates/codegen/xai-grok-shell/src/agent/subagent/attempt_store/completion_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 131 | `A` `crates/codegen/xai-grok-shell/src/agent/subagent/attempt_store/decoder.rs` | adopt | `GB-19D-INTEGRATION` |
| 132 | `A` `crates/codegen/xai-grok-shell/src/agent/subagent/attempt_store/decoder_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 133 | `A` `crates/codegen/xai-grok-shell/src/agent/subagent/attempt_store/intent.rs` | adopt | `GB-19D-INTEGRATION` |
| 134 | `A` `crates/codegen/xai-grok-shell/src/agent/subagent/attempt_store/intent_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 135 | `M` `crates/codegen/xai-grok-shell/src/agent/subagent/attempt_store/mod.rs` | adopt | `GB-19D-INTEGRATION` |
| 136 | `A` `crates/codegen/xai-grok-shell/src/agent/subagent/attempt_store/recovery.rs` | adopt | `GB-19D-INTEGRATION` |
| 137 | `A` `crates/codegen/xai-grok-shell/src/agent/subagent/attempt_store/recovery_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 138 | `A` `crates/codegen/xai-grok-shell/src/agent/subagent/attempt_store/rewind.rs` | adopt | `GB-19D-INTEGRATION` |
| 139 | `A` `crates/codegen/xai-grok-shell/src/agent/subagent/attempt_store/rewind_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 140 | `M` `crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs` | adopt | `GB-19D-INTEGRATION` |
| 141 | `M` `crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs` | adopt | `GB-19D-INTEGRATION` |
| 142 | `M` `crates/codegen/xai-grok-shell/src/agent/subagent/tests/rest.rs` | adopt | `GB-19D-INTEGRATION` |
| 143 | `M` `crates/codegen/xai-grok-shell/src/agent/subscription_check.rs` | adopt | `GB-19D-INTEGRATION` |
| 144 | `M` `crates/codegen/xai-grok-shell/src/auth/manager.rs` | adopt | `GB-19D-INTEGRATION` |
| 145 | `M` `crates/codegen/xai-grok-shell/src/auth/manager/lock.rs` | adopt | `GB-19D-INTEGRATION` |
| 146 | `M` `crates/codegen/xai-grok-shell/src/auth/manager/remedy.rs` | adopt | `GB-19D-INTEGRATION` |
| 147 | `M` `crates/codegen/xai-grok-shell/src/auth/manager_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 148 | `A` `crates/codegen/xai-grok-shell/src/extensions/consent.rs` | adopt | `GB-19D-INTEGRATION` |
| 149 | `M` `crates/codegen/xai-grok-shell/src/extensions/mcp.rs` | adopt | `GB-19D-INTEGRATION` |
| 150 | `M` `crates/codegen/xai-grok-shell/src/extensions/mod.rs` | adopt | `GB-19D-INTEGRATION` |
| 151 | `M` `crates/codegen/xai-grok-shell/src/extensions/notification.rs` | adopt | `GB-19D-INTEGRATION` |
| 152 | `M` `crates/codegen/xai-grok-shell/src/leader/protocol.rs` | adopt | `GB-19D-INTEGRATION` |
| 153 | `M` `crates/codegen/xai-grok-shell/src/leader/server.rs` | adopt | `GB-19D-INTEGRATION` |
| 154 | `M` `crates/codegen/xai-grok-shell/src/leader/server_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 155 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session.rs` | adopt | `GB-19D-INTEGRATION` |
| 156 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/mcp.rs` | adopt | `GB-19D-INTEGRATION` |
| 157 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/model_switch.rs` | adopt | `GB-19D-INTEGRATION` |
| 158 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/notification_drain.rs` | adopt | `GB-19D-INTEGRATION` |
| 159 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/prompt_queue.rs` | adopt | `GB-19D-INTEGRATION` |
| 160 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/run_loop.rs` | adopt | `GB-19D-INTEGRATION` |
| 161 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs` | adopt | `GB-19D-INTEGRATION` |
| 162 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/spawn.rs` | adopt | `GB-19D-INTEGRATION` |
| 163 | `A` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/status_line.rs` | adopt | `GB-19D-INTEGRATION` |
| 164 | `A` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/status_line_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 165 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/turn.rs` | adopt | `GB-19D-INTEGRATION` |
| 166 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/turn_end.rs` | adopt | `GB-19D-INTEGRATION` |
| 167 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/auth_error_no_retry_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 168 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/cancel_running_task_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 169 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/goal/goal_planner_e2e_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 170 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/idle_resume_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 171 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/inline_auto_compact_flow_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 172 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/memory_config_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 173 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/prompt_queue_actor_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 174 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/replay_buffer_send_update_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 175 | `A` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/status_line_payload_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 176 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/support.rs` | adopt | `GB-19D-INTEGRATION` |
| 177 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/turn/chat_history_integrity_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 178 | `M` `crates/codegen/xai-grok-shell/src/session/commands.rs` | adopt | `GB-19D-INTEGRATION` |
| 179 | `M` `crates/codegen/xai-grok-shell/src/session/compaction.rs` | adopt | `GB-19D-INTEGRATION` |
| 180 | `M` `crates/codegen/xai-grok-shell/src/session/compaction_inline_auto_compact_flow_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 181 | `M` `crates/codegen/xai-grok-shell/src/session/fs_watch.rs` | adopt | `GB-19D-INTEGRATION` |
| 182 | `M` `crates/codegen/xai-grok-shell/src/session/handle.rs` | adopt | `GB-19D-INTEGRATION` |
| 183 | `M` `crates/codegen/xai-grok-shell/src/session/prompt_parser.rs` | adopt | `GB-19D-INTEGRATION` |
| 184 | `M` `crates/codegen/xai-grok-shell/src/session/telemetry/permission.rs` | adopt | `GB-19D-INTEGRATION` |
| 185 | `M` `crates/codegen/xai-grok-shell/src/session/templates/goal_planner_prompt.md` | adopt | `GB-19D-INTEGRATION` |
| 186 | `M` `crates/codegen/xai-grok-shell/src/session/templates/goal_verifier_prompt.md` | adopt | `GB-19D-INTEGRATION` |
| 187 | `M` `crates/codegen/xai-grok-shell/src/test_support/lsp_runtime.rs` | adopt | `GB-19D-INTEGRATION` |
| 188 | `M` `crates/codegen/xai-grok-shell/src/util/config/persist_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 189 | `M` `crates/codegen/xai-grok-shell/src/util/config/resolve/tool_approvals.rs` | adopt | `GB-19D-INTEGRATION` |
| 190 | `M` `crates/codegen/xai-grok-shell/tests/test_mcp_permission_persistence.rs` | adopt | `GB-19D-INTEGRATION` |
| 191 | `A` `crates/codegen/xai-grok-status-line/Cargo.toml` | adopt | `GB-19D-INTEGRATION` |
| 192 | `A` `crates/codegen/xai-grok-status-line/src/config.rs` | adopt | `GB-19D-INTEGRATION` |
| 193 | `A` `crates/codegen/xai-grok-status-line/src/config_test_support.rs` | adopt | `GB-19D-INTEGRATION` |
| 194 | `A` `crates/codegen/xai-grok-status-line/src/config_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 195 | `A` `crates/codegen/xai-grok-status-line/src/context.rs` | adopt | `GB-19D-INTEGRATION` |
| 196 | `A` `crates/codegen/xai-grok-status-line/src/context_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 197 | `A` `crates/codegen/xai-grok-status-line/src/lib.rs` | adopt | `GB-19D-INTEGRATION` |
| 198 | `A` `crates/codegen/xai-grok-status-line/testdata/status_wire.json` | adopt | `GB-19D-INTEGRATION` |
| 199 | `M` `crates/codegen/xai-grok-subagent-resolution/src/config.rs` | adopt | `GB-19D-INTEGRATION` |
| 200 | `M` `crates/codegen/xai-grok-telemetry/src/events/mod.rs` | adopt | `GB-19D-INTEGRATION` |
| 201 | `M` `crates/codegen/xai-grok-telemetry/src/events/permission_analytics.rs` | adopt | `GB-19D-INTEGRATION` |
| 202 | `M` `crates/codegen/xai-grok-telemetry/src/external/tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 203 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs` | adopt | `GB-19D-INTEGRATION` |
| 204 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/video_gen/mod.rs` | adopt | `GB-19D-INTEGRATION` |
| 205 | `M` `crates/codegen/xai-grok-tools/src/persistence.rs` | adopt | `GB-19D-INTEGRATION` |
| 206 | `M` `crates/codegen/xai-grok-tools/src/registry/types.rs` | adopt | `GB-19D-INTEGRATION` |
| 207 | `M` `crates/codegen/xai-grok-version/Cargo.toml` | not applicable | `GB-19D-RELEASE` |
| 208 | `M` `crates/codegen/xai-grok-workspace-daemon/src/preview_supervisor.rs` | adopt | `GB-19D-INTEGRATION` |
| 209 | `M` `crates/codegen/xai-grok-workspace-types/src/rpc/workspace.rs` | adopt | `GB-19D-INTEGRATION` |
| 210 | `M` `crates/codegen/xai-grok-workspace/src/bin/workspace_server.rs` | adopt | `GB-19D-INTEGRATION` |
| 211 | `M` `crates/codegen/xai-grok-workspace/src/handle.rs` | adopt | `GB-19D-INTEGRATION` |
| 212 | `M` `crates/codegen/xai-grok-workspace/src/handle_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 213 | `M` `crates/codegen/xai-grok-workspace/src/hub_server.rs` | adopt | `GB-19D-INTEGRATION` |
| 214 | `M` `crates/codegen/xai-grok-workspace/src/hub_server_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 215 | `M` `crates/codegen/xai-grok-workspace/src/permission/manager/mod.rs` | adopt | `GB-19D-INTEGRATION` |
| 216 | `M` `crates/codegen/xai-grok-workspace/src/permission/prompter.rs` | adopt | `GB-19D-INTEGRATION` |
| 217 | `M` `crates/codegen/xai-grok-workspace/src/permission/state.rs` | adopt | `GB-19D-INTEGRATION` |
| 218 | `M` `crates/codegen/xai-grok-workspace/src/permission/types.rs` | adopt | `GB-19D-INTEGRATION` |
| 219 | `M` `crates/codegen/xai-grok-workspace/src/session/git.rs` | adopt | `GB-19D-INTEGRATION` |
| 220 | `M` `crates/codegen/xai-grok-workspace/src/session/git_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 221 | `M` `crates/codegen/xai-grok-workspace/src/session/mod.rs` | adopt | `GB-19D-INTEGRATION` |
| 222 | `M` `crates/codegen/xai-grok-workspace/src/session/tool_config.rs` | adopt | `GB-19D-INTEGRATION` |
| 223 | `M` `crates/codegen/xai-grok-workspace/src/status_config.rs` | adopt | `GB-19D-INTEGRATION` |
| 224 | `M` `crates/codegen/xai-grok-workspace/src/worktree/mod.rs` | adopt | `GB-19D-INTEGRATION` |
| 225 | `M` `crates/codegen/xai-ratatui-textarea/src/editor.rs` | adopt | `GB-19D-INTEGRATION` |
| 226 | `M` `crates/codegen/xai-ratatui-textarea/src/editor_keys.rs` | adopt | `GB-19D-INTEGRATION` |
| 227 | `M` `crates/codegen/xai-ratatui-textarea/src/textarea.rs` | adopt | `GB-19D-INTEGRATION` |
| 228 | `M` `crates/codegen/xai-ratatui-textarea/src/textarea_tests.rs` | adopt | `GB-19D-INTEGRATION` |
| 229 | `M` `crates/common/xai-tool-types/src/task.rs` | adopt | `GB-19D-INTEGRATION` |
| 230 | `M` `third_party/NOTICE` | adopt | `GB-19D-INTEGRATION` |
| 231 | `M` `third_party/README.md` | adopt | `GB-19D-INTEGRATION` |
