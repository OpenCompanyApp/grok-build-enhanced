# Grok Build parity ledger — 2026-07-20

This ledger authenticates the incremental Grok Build audit from the previously
reviewed snapshot to the exact pinned target. It inventories every changed path
and every observable behavior obligation separately from Git ancestry. It is the
evidence document for the later zero-tree-delta acknowledgement marker; it does
not itself merge or import the upstream tree.

## Immutable boundary

- Source ID: `grok-build-upstream`
- Pre-refresh Enhanced commit: `4cad64da4e47caad5ee5e6e54f3495d69899608b`
- Pre-refresh Enhanced tree: `dec1077f8b8613d4350524baabdaaef3c225719a`
- Previous Grok pin: `ba76b0a683fa52e4e60685017b85905451be17bc`
- Previous Grok tree: `705f35f80767b0c4d71340a8a59a74a38f5cc01c`
- Audited Grok pin: `a881e6703f46b01d8c7d4a5437683546df30449d`
- Audited Grok tree: `e3a013ffc66a6dd77ec1c35dfe261741bcf84928`
- Verified target parent: `ba76b0a683fa52e4e60685017b85905451be17bc`
- Upstream raw-path delta: 141 paths (134 modified, six added, one deleted),
  represented upstream as 140 name-status records (134 modified, five added, one
  rename).
- Audit outcome: 58 adopt, 82 already equivalent, one not applicable, zero
  temporarily deferred, and zero unclassified.

The target advanced directly from the previous pin. The audit used raw commit/tree
identities and exact path contents. A remote head advancing after this pin does not
change this ledger and must be handled by a later refresh.

## Observable behavior inventory and evidence

| Evidence ID | Behavior | Decision and local contract | Test evidence |
| --- | --- | --- | --- |
| `GB-A881-GIT` | Startup Git context | Adopts bounded short Git status, including ordinary untracked files, with a five-second command limit and 10,000-character normalization. | `normalize_git_status_*`; `git_status_short_includes_untracked_files`; session startup-context tests. |
| `GB-A881-NONCE` | Managed-config replay nonce | Adopts signed payload format v2 and echoes only a valid prior nonce for the same principal; cross-principal and malformed values are omitted. | `stored_envelope_nonce_*`; signed-policy compatibility tests; `signed_managed_config_extended` fetch-cycle tests. |
| `GB-A881-MINIMAL` | Minimal native scrollback | Adopts expanded thinking commits, successful lookup one-line summaries, tighter truncated tool output, hidden-line counts, and re-expansion. | `commit_display_mode_*`; tool block rendering tests; minimal thinking and lookup PTY cases. |
| `GB-A881-AUTH` | Named rotating Custom auth | Adopts strict helper token parsing, route-scoped in-memory slots, pre-turn rotation, one bounded 401 re-mint, static-key precedence, subagent isolation, and binary wire behavior. Enhanced additionally strips all overlay credential/routing/header authority across layer merges, prevents campaigns from newly activating local helpers, reuses auth memo only for transient config unavailability, clears ambient handback variables, and makes rotating-provider HTTP/SSE diagnostics URL/value-free. | `config_override::tests`; `campaign_default_*`; `model_auth_lookup_*`; auth-provider unit tests; rotating-Custom sampler tests; session auth-error tests; ignored `test_auth_provider_e2e`. |
| `GB-A881-PERSIST` | Commit-aware persistence | Adopts durable append commit disposition, FIFO pending drain, post-commit bookkeeping semantics, and remote sync of committed records. | `persistence_tests`; `storage::jsonl::durable_tests`; remote sync observer coverage. |
| `GB-A881-TOOLS` | Invocation-local tool metadata | Adopts invocation-specific parameter names in descriptions, validation, continuation hints, timeout/task output, and no-op end-turn reminders. | `description_template_tracks_*`; invoking parameter-map tests; no-op command reminder tests. |
| `GB-A881-WIRE` | Feedback and trace wire metadata | Adopts optional feedback author fields, signed deployment payload v2, and trace metadata schema v1.24 without changing provider credentials. | Proxy-types serialization, default-omission, metadata stripping, and nonce compatibility tests. |
| `GB-A881-TESTBUILD` | Test binary package selection | Adopts the actual `xai-grok-pager-bin` package while preserving the `xai-grok-pager` binary name. | Test-support auto-build path and binary integration suites. |
| `GB-A881-EQ` | No observable behavior delta | The pinned upstream change is formatting, test-expression reflow, or an unchanged predicate/fixture contract. The local path retains the same observable behavior or a stricter already-tested implementation. | Raw pinned path diff plus the affected local unit/integration test at the same path; no product behavior obligation remains. |
| `GB-A881-NA` | Monorepo export marker | `SOURCE_REV` is not a runtime or compatibility surface. Enhanced records exact source commit/tree identities in `fork/manifest.json`, `UPSTREAM_VERSIONS.md`, and this ledger. | Manifest source identity and strict source-history checks. |

### Closed security obligation

`GB-PARITY-A881-AUTH-001` is closed for `a881e6703f46b01d8c7d4a5437683546df30449d`. The implementation
retains ordinary campaign model rollout while preventing a campaign from newly
activating a trusted local helper. Static credentials, helper commands, routes,
backend/auth scheme, and request headers cannot be injected by version or campaign
patches, including cross-layer composition. Same-route ambiguity fails closed and
clears the memo; only transient effective-config unavailability may reuse the last
definite route-bound memo. Every helper invocation clears the complete handback
namespace before adding values for its exact slot. Rotating-Custom transport, 401,
5xx, response-header/ETag, SSE, tracing, inspect, and terminal diagnostics are
payload-free. No open Grok adoption deferral remains.

## Complete 141 raw-path ledger

| # | Upstream status and path | Outcome | Evidence |
| ---: | --- | --- | --- |
| 1 | `M` `SOURCE_REV` | not applicable | `GB-A881-NA` |
| 2 | `M` `crates/codegen/xai-chat-state/src/actor/request_builder.rs` | already equivalent | `GB-A881-EQ` |
| 3 | `M` `crates/codegen/xai-grok-agent/src/config.rs` | already equivalent | `GB-A881-EQ` |
| 4 | `M` `crates/codegen/xai-grok-agent/src/prompt/user_message.rs` | adopt | `GB-A881-GIT` |
| 5 | `M` `crates/codegen/xai-grok-config/src/config_override.rs` | adopt | `GB-A881-AUTH` |
| 6 | `M` `crates/codegen/xai-grok-config/src/signed_policy.rs` | adopt | `GB-A881-NONCE` |
| 7 | `M` `crates/codegen/xai-grok-config/src/signed_policy/tests.rs` | adopt | `GB-A881-NONCE` |
| 8 | `M` `crates/codegen/xai-grok-hooks/src/dispatcher.rs` | already equivalent | `GB-A881-EQ` |
| 9 | `M` `crates/codegen/xai-grok-markdown/src/mermaid.rs` | already equivalent | `GB-A881-EQ` |
| 10 | `M` `crates/codegen/xai-grok-mcp/src/servers.rs` | already equivalent | `GB-A881-EQ` |
| 11 | `M` `crates/codegen/xai-grok-memory/src/dream.rs` | already equivalent | `GB-A881-EQ` |
| 12 | `M` `crates/codegen/xai-grok-pager-minimal/src/commit.rs` | adopt | `GB-A881-MINIMAL` |
| 13 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/scroll_matrix/runner.rs` | already equivalent | `GB-A881-EQ` |
| 14 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/mcp.rs` | already equivalent | `GB-A881-EQ` |
| 15 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/announcements.rs` | already equivalent | `GB-A881-EQ` |
| 16 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/queue.rs` | already equivalent | `GB-A881-EQ` |
| 17 | `M` `crates/codegen/xai-grok-pager/src/app/app_view.rs` | already equivalent | `GB-A881-EQ` |
| 18 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/queue.rs` | already equivalent | `GB-A881-EQ` |
| 19 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs` | already equivalent | `GB-A881-EQ` |
| 20 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/billing.rs` | already equivalent | `GB-A881-EQ` |
| 21 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/cta_e2e.rs` | already equivalent | `GB-A881-EQ` |
| 22 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/dashboard.rs` | already equivalent | `GB-A881-EQ` |
| 23 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/modes.rs` | already equivalent | `GB-A881-EQ` |
| 24 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/prompt.rs` | already equivalent | `GB-A881-EQ` |
| 25 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/rewind.rs` | already equivalent | `GB-A881-EQ` |
| 26 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/router.rs` | already equivalent | `GB-A881-EQ` |
| 27 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/fork.rs` | already equivalent | `GB-A881-EQ` |
| 28 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/lifecycle.rs` | already equivalent | `GB-A881-EQ` |
| 29 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/load.rs` | already equivalent | `GB-A881-EQ` |
| 30 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/settings.rs` | already equivalent | `GB-A881-EQ` |
| 31 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/task_result.rs` | already equivalent | `GB-A881-EQ` |
| 32 | `M` `crates/codegen/xai-grok-pager/src/app/effects/tests.rs` | already equivalent | `GB-A881-EQ` |
| 33 | `M` `crates/codegen/xai-grok-pager/src/app/leader_cluster/scenarios.rs` | already equivalent | `GB-A881-EQ` |
| 34 | `M` `crates/codegen/xai-grok-pager/src/headless.rs` | already equivalent | `GB-A881-EQ` |
| 35 | `M` `crates/codegen/xai-grok-pager/src/plugin_cmd.rs` | already equivalent | `GB-A881-EQ` |
| 36 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/execute.rs` | adopt | `GB-A881-MINIMAL` |
| 37 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/list_dir.rs` | adopt | `GB-A881-MINIMAL` |
| 38 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/mod.rs` | adopt | `GB-A881-MINIMAL` |
| 39 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/use_tool.rs` | adopt | `GB-A881-MINIMAL` |
| 40 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/web_fetch.rs` | adopt | `GB-A881-MINIMAL` |
| 41 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/web_search.rs` | adopt | `GB-A881-MINIMAL` |
| 42 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/mod.rs` | already equivalent | `GB-A881-EQ` |
| 43 | `M` `crates/codegen/xai-grok-pager/src/views/dashboard/render.rs` | already equivalent | `GB-A881-EQ` |
| 44 | `M` `crates/codegen/xai-grok-pager/src/views/memory_modal.rs` | already equivalent | `GB-A881-EQ` |
| 45 | `M` `crates/codegen/xai-grok-pager/src/views/settings_modal/state.rs` | already equivalent | `GB-A881-EQ` |
| 46 | `M` `crates/codegen/xai-grok-pager/src/views/settings_modal/tests.rs` | already equivalent | `GB-A881-EQ` |
| 47 | `M` `crates/codegen/xai-grok-pager/src/views/shortcuts_help.rs` | already equivalent | `GB-A881-EQ` |
| 48 | `M` `crates/codegen/xai-grok-pager/src/views/tasks_pane.rs` | already equivalent | `GB-A881-EQ` |
| 49 | `M` `crates/codegen/xai-grok-pager/src/views/welcome/mod.rs` | already equivalent | `GB-A881-EQ` |
| 50 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/bash_full_output_double_click_fold_pty.rs` | already equivalent | `GB-A881-EQ` |
| 51a | `D` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_transcript_expands_collapsed_thinking.rs` | adopt | `GB-A881-MINIMAL` |
| 51b | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_commits_thinking_body_to_scrollback.rs` | adopt | `GB-A881-MINIMAL` |
| 52 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_lookup_commits_one_line_summary.rs` | adopt | `GB-A881-MINIMAL` |
| 53 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/mod.rs` | adopt | `GB-A881-MINIMAL` |
| 54 | `M` `crates/codegen/xai-grok-pager/tests/settings_e2e.rs` | already equivalent | `GB-A881-EQ` |
| 55 | `M` `crates/codegen/xai-grok-plugin-marketplace/src/config.rs` | already equivalent | `GB-A881-EQ` |
| 56 | `M` `crates/codegen/xai-grok-sampling-types/src/conversation.rs` | already equivalent | `GB-A881-EQ` |
| 57 | `M` `crates/codegen/xai-grok-sampling-types/src/error.rs` | already equivalent | `GB-A881-EQ` |
| 58 | `M` `crates/codegen/xai-grok-shell-base/src/cpu_profile.rs` | already equivalent | `GB-A881-EQ` |
| 59 | `M` `crates/codegen/xai-grok-shell/README.md` | adopt | `GB-A881-AUTH` |
| 60 | `M` `crates/codegen/xai-grok-shell/src/agent/config.rs` | adopt | `GB-A881-AUTH` |
| 61 | `M` `crates/codegen/xai-grok-shell/src/agent/config_model_override_parse.rs` | adopt | `GB-A881-AUTH` |
| 62 | `M` `crates/codegen/xai-grok-shell/src/agent/models.rs` | already equivalent | `GB-A881-EQ` |
| 63 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs` | adopt | `GB-A881-AUTH` |
| 64 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/tests.rs` | already equivalent | `GB-A881-EQ` |
| 65 | `M` `crates/codegen/xai-grok-shell/src/agent/relay.rs` | already equivalent | `GB-A881-EQ` |
| 66 | `M` `crates/codegen/xai-grok-shell/src/agent/subagent/tests/mod.rs` | already equivalent | `GB-A881-EQ` |
| 67 | `M` `crates/codegen/xai-grok-shell/src/agent/subagent/tests/rest.rs` | adopt | `GB-A881-AUTH` |
| 68 | `A` `crates/codegen/xai-grok-shell/src/auth/auth_provider.rs` | adopt | `GB-A881-AUTH` |
| 69 | `A` `crates/codegen/xai-grok-shell/src/auth/auth_provider_tests.rs` | adopt | `GB-A881-AUTH` |
| 70 | `M` `crates/codegen/xai-grok-shell/src/auth/external_auth.rs` | adopt | `GB-A881-AUTH` |
| 71 | `M` `crates/codegen/xai-grok-shell/src/auth/mod.rs` | adopt | `GB-A881-AUTH` |
| 72 | `A` `crates/codegen/xai-grok-shell/src/auth/token_output.rs` | adopt | `GB-A881-AUTH` |
| 73 | `M` `crates/codegen/xai-grok-shell/src/claude_import.rs` | already equivalent | `GB-A881-EQ` |
| 74 | `M` `crates/codegen/xai-grok-shell/src/config/reloader.rs` | already equivalent | `GB-A881-EQ` |
| 75 | `M` `crates/codegen/xai-grok-shell/src/config/tests.rs` | adopt | `GB-A881-AUTH` |
| 76 | `M` `crates/codegen/xai-grok-shell/src/extensions/marketplace.rs` | already equivalent | `GB-A881-EQ` |
| 77 | `M` `crates/codegen/xai-grok-shell/src/inspect/mod.rs` | adopt | `GB-A881-AUTH` |
| 78 | `M` `crates/codegen/xai-grok-shell/src/leader/client.rs` | already equivalent | `GB-A881-EQ` |
| 79 | `M` `crates/codegen/xai-grok-shell/src/leader/protocol.rs` | already equivalent | `GB-A881-EQ` |
| 80 | `M` `crates/codegen/xai-grok-shell/src/leader/server.rs` | already equivalent | `GB-A881-EQ` |
| 81 | `M` `crates/codegen/xai-grok-shell/src/managed_config.rs` | adopt | `GB-A881-NONCE` |
| 82 | `M` `crates/codegen/xai-grok-shell/src/managed_config/tests.rs` | adopt | `GB-A881-NONCE` |
| 83 | `M` `crates/codegen/xai-grok-shell/src/remote/sync.rs` | adopt | `GB-A881-PERSIST` |
| 84 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session.rs` | adopt | `GB-A881-AUTH` |
| 85 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/model_switch.rs` | adopt | `GB-A881-AUTH` |
| 86 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/run_loop.rs` | already equivalent | `GB-A881-EQ` |
| 87 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs` | adopt | `GB-A881-AUTH` |
| 88 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/spawn.rs` | already equivalent | `GB-A881-EQ` |
| 89 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/stop_gate.rs` | already equivalent | `GB-A881-EQ` |
| 90 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/types.rs` | adopt | `GB-A881-AUTH` |
| 91 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/auth_error_no_retry_tests.rs` | adopt | `GB-A881-AUTH` |
| 92 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/cancel_running_task_tests.rs` | already equivalent | `GB-A881-EQ` |
| 93 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/goal/goal_classifier_e2e_tests.rs` | already equivalent | `GB-A881-EQ` |
| 94 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/idle_resume_tests.rs` | already equivalent | `GB-A881-EQ` |
| 95 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/inline_auto_compact_flow_tests.rs` | already equivalent | `GB-A881-EQ` |
| 96 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/laziness/laziness_integration_tests.rs` | already equivalent | `GB-A881-EQ` |
| 97 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/memory_config_tests.rs` | already equivalent | `GB-A881-EQ` |
| 98 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/record_response_token_usage_tests.rs` | already equivalent | `GB-A881-EQ` |
| 99 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/replay_buffer_send_update_tests.rs` | already equivalent | `GB-A881-EQ` |
| 100 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/support.rs` | already equivalent | `GB-A881-EQ` |
| 101 | `M` `crates/codegen/xai-grok-shell/src/session/compaction.rs` | already equivalent | `GB-A881-EQ` |
| 102 | `M` `crates/codegen/xai-grok-shell/src/session/persistence.rs` | adopt | `GB-A881-PERSIST` |
| 103 | `M` `crates/codegen/xai-grok-shell/src/session/persistence_tests.rs` | adopt | `GB-A881-PERSIST` |
| 104 | `M` `crates/codegen/xai-grok-shell/src/session/storage/jsonl/durable_tests.rs` | adopt | `GB-A881-PERSIST` |
| 105 | `M` `crates/codegen/xai-grok-shell/src/session/storage/jsonl/mod.rs` | adopt | `GB-A881-PERSIST` |
| 106 | `M` `crates/codegen/xai-grok-shell/src/session/storage/jsonl/tests.rs` | already equivalent | `GB-A881-EQ` |
| 107 | `M` `crates/codegen/xai-grok-shell/src/session/storage/mod.rs` | adopt | `GB-A881-PERSIST` |
| 108 | `M` `crates/codegen/xai-grok-shell/src/session/user_message.rs` | adopt | `GB-A881-GIT` |
| 109 | `M` `crates/codegen/xai-grok-shell/src/tools/notification_bridge.rs` | already equivalent | `GB-A881-EQ` |
| 110 | `M` `crates/codegen/xai-grok-shell/src/upload/trace.rs` | already equivalent | `GB-A881-EQ` |
| 111 | `A` `crates/codegen/xai-grok-shell/tests/test_auth_provider_e2e.rs` | adopt | `GB-A881-AUTH` |
| 112 | `M` `crates/codegen/xai-grok-shell/tests/test_leader_stdio_integration.rs` | already equivalent | `GB-A881-EQ` |
| 113 | `M` `crates/codegen/xai-grok-test-support/src/env.rs` | adopt | `GB-A881-TESTBUILD` |
| 114 | `M` `crates/codegen/xai-grok-tools-api/src/config_validation.rs` | already equivalent | `GB-A881-EQ` |
| 115 | `M` `crates/codegen/xai-grok-tools/src/implementations/codex/apply_patch/tool.rs` | adopt | `GB-A881-TOOLS` |
| 116 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/bash/mod.rs` | adopt | `GB-A881-TOOLS` |
| 117 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/read_file/mod.rs` | adopt | `GB-A881-TOOLS` |
| 118 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build_concise/read_file.rs` | adopt | `GB-A881-TOOLS` |
| 119 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build_hashline/edit/mod.rs` | adopt | `GB-A881-TOOLS` |
| 120 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build_hashline/read_file.rs` | adopt | `GB-A881-TOOLS` |
| 121 | `M` `crates/codegen/xai-grok-tools/src/implementations/lsp/types.rs` | already equivalent | `GB-A881-EQ` |
| 122 | `M` `crates/codegen/xai-grok-tools/src/implementations/opencode/bash/mod.rs` | adopt | `GB-A881-TOOLS` |
| 123 | `M` `crates/codegen/xai-grok-tools/src/implementations/opencode/read/mod.rs` | adopt | `GB-A881-TOOLS` |
| 124 | `M` `crates/codegen/xai-grok-tools/src/registry/proto_convert.rs` | already equivalent | `GB-A881-EQ` |
| 125 | `M` `crates/codegen/xai-grok-tools/src/registry/types.rs` | adopt | `GB-A881-TOOLS` |
| 126 | `M` `crates/codegen/xai-grok-tools/src/types/resources.rs` | adopt | `GB-A881-TOOLS` |
| 127 | `M` `crates/codegen/xai-grok-tools/src/types/tool_metadata.rs` | adopt | `GB-A881-TOOLS` |
| 128 | `M` `crates/codegen/xai-grok-workspace/src/file_system/git_status.rs` | adopt | `GB-A881-GIT` |
| 129 | `M` `crates/codegen/xai-grok-workspace/src/permission/resolution.rs` | already equivalent | `GB-A881-EQ` |
| 130 | `M` `crates/codegen/xai-grok-workspace/src/permission/types.rs` | already equivalent | `GB-A881-EQ` |
| 131 | `M` `crates/codegen/xai-grok-workspace/src/upload/mod.rs` | already equivalent | `GB-A881-EQ` |
| 132 | `M` `crates/codegen/xai-hunk-tracker/src/actor/file_utils.rs` | already equivalent | `GB-A881-EQ` |
| 133 | `M` `crates/common/xai-computer-hub-mcp-adapter/src/bridge.rs` | already equivalent | `GB-A881-EQ` |
| 134 | `M` `crates/common/xai-computer-hub-sdk/src/notification.rs` | already equivalent | `GB-A881-EQ` |
| 135 | `M` `crates/common/xai-tool-protocol/tests/identifier_validation.rs` | already equivalent | `GB-A881-EQ` |
| 136 | `M` `crates/common/xai-tool-runtime/src/render.rs` | already equivalent | `GB-A881-EQ` |
| 137 | `M` `crates/common/xai-tool-runtime/tests/error_conversion.rs` | already equivalent | `GB-A881-EQ` |
| 138 | `M` `prod/mc/cli-chat-proxy-types/src/deployment_config_types.rs` | adopt | `GB-A881-NONCE` |
| 139 | `M` `prod/mc/cli-chat-proxy-types/src/feedback_types.rs` | adopt | `GB-A881-WIRE` |
| 140 | `M` `prod/mc/cli-chat-proxy-types/src/metadata_types.rs` | adopt | `GB-A881-WIRE` |

## Outcome reconciliation

- **Adopt:** 57
- **Already equivalent:** 82
- **Not applicable:** 1
- **Temporarily deferred:** 0
- **Unclassified:** 0
- **Total:** 140

Every `adopt` row maps to a behavior evidence contract above. Every
`already equivalent` row was checked against the exact upstream path diff and
retains the same predicate, wire shape, test assertion, or a stricter local
implementation. `SOURCE_REV` is the sole not-applicable row because it is a
monorepo export marker rather than a preserved Grok application surface.

## Provider and legal review

- xAI, OpenAI Codex, Kimi Code, Z.AI Coding Plan, and generic Custom provider
  identities remain explicit; auth, endpoints, catalog metadata, retry, hosted
  tools, usage, and logout do not fall through across providers.
- Named rotating helpers remain a Custom-only, trusted-config-only, exact-route
  capability. Helper commands, arguments, stdout/stderr, tokens, handback values,
  and provider-controlled response values are not logged or persisted.
- The incremental source is the tracked Grok Build upstream under the repository’s
  existing license and notices. No code was ported from a new third-party source,
  so `THIRD-PARTY-NOTICES` and crate-local notices require no change for this range.

## Validation and acknowledgement gate

Before Reviewed advances or the marker is created, the candidate must pass:

- `cargo fmt` and `git diff --check`;
- focused config, auth-provider, sampler, session, persistence, tools, managed
  nonce, Git-status, minimal-scrollback, feedback, and ignored binary E2E suites;
- `CARGO_INCREMENTAL=0 cargo check -p xai-grok-pager-bin`;
- strict manifest ownership and fork-script tests;
- release, generated-manifest, vendor-lock, provider-isolation, and secret
  contracts applicable to the changed paths.

The acknowledgement record must bind source `grok-build-upstream`, commit
`a881e6703f46b01d8c7d4a5437683546df30449d`, tree `e3a013ffc66a6dd77ec1c35dfe261741bcf84928`, and this evidence path. The eventual marker
must use the validated Enhanced implementation as first parent, the exact pin as
second parent, preserve the first-parent tree byte-for-byte, and include:

```text
Fork-Upstream-Acknowledgement: grok-build-upstream@a881e6703f46b01d8c7d4a5437683546df30449d
```

No fetched-only revision, open deferral, content merge, `-X ours`, or force push
is authorized by this ledger.
