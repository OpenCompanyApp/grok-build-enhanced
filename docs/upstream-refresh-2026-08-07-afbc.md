# Upstream refresh parity ledger — 2026-08-07 afbc

This ledger closes the carried Grok Build behavior queue through
`afbc0fb710320c7add294c2106d447ecc3e3af2e` while preserving Enhanced's
provider isolation, release routes, branding, and generated-root contract.
Upstream history was reviewed as immutable evidence; no upstream tree was
content-merged or used to replace the Enhanced architecture.

## Immutable boundary

- Reviewed-from Grok commit:
  `dd04f397b1d02f2272b092555669dfba1f01bc85`, tree
  `c5c7bdcda32a828efa112883dcd5279ce78714ec`.
- Target Grok commit:
  `afbc0fb710320c7add294c2106d447ecc3e3af2e`, tree
  `99e3e7c4d8a6c0214101c99e5cedded0325e96be`.
- Target sole parent:
  `393430ee4934bc791b0d538f304a21691c517433`.
- Authenticated source ID: `grok-build-upstream`.
- The seven reviewed upstream snapshots are `a4221165`, `780d1388`,
  `e5478eff`, `ed6d5436`, `a5589e95`, `393430ee`, and `afbc0fb7`.
- Canonical reviewed-tree-to-target-tree raw-path digest:
  `35e037d25208ba84edc1987faa3345e8f2da03659220657ada79b6c2744f965f`.
- The generated root `Cargo.toml` remained byte-identical, SHA-256
  `28a3ea7e1c859729a0c5cf77f87ff7f0ece319a576697b274917359e11be480b`.
- No credential-bearing live request, push, tag, release, or pull-request
  mutation was performed.

## Behavior closure

All preserved Grok surfaces in the carried A422, 780D, E547, ED6D, A558,
3934, and AFBC inventories are adopted or shown equivalent in the Enhanced
implementation. The implementation includes the session and process lifecycle
settling, split session search and bounded fork copy, Git ODB/GitGate restore,
task admission and task-log bounds, permission decision analytics, provider
error normalization, xAI key validation, ACP recovery, usage/memory/disk TUI
surfaces, terminal-native theme behavior, narrow-table rendering, update smoke
verification, low-level checkout watcher isolation, and pre-truncation MCP
image harvesting.

Architecture-only source moves are classified with their observable behavior
family. Version/changelog/SOURCE_REV artifacts remain fork-owned release
metadata. The empty ignored Chrome E2E scaffold has no runtime behavior. There
are no temporarily deferred Grok adoption rows in this ledger.

The upstream bearer-fragment helper is specifically not applicable under the
fork credential-safety contract: 401 attribution retains only whether a bearer
was present, never token prefixes, suffixes, hashes, or other token-derived
identifiers. The retry behavior itself remains adopted through a non-secret
request marker.

## Provider-reference audit

The refreshed OpenAI Codex reference is adopted only at the direct subscription
adapter seams: managed login/workspace policy fails closed before credential
hydration, base-instruction provenance is retained, payload diagnostics stay
bounded and redacted, and pre-output connection failures reconnect with capped
backoff outside the ordinary retry budget. Plugin host, replacement app-server,
code-mode, and Codex-only sandbox architecture remain out of scope.

OpenCode's compatible retry/error and ACP cache-write usage corrections are
adopted. Kimi Code's latest global MCP-auth probe does not change the isolated
API-key provider wire contract. Kimi CLI, Oh My Pi, CodexBar, models.dev, and
Exa changes remain non-normative or research-only outside their declared
adapter/interoperability scope; none creates a new credential route, runtime
provider, or product claim.

## Validation record

The offline candidate is qualified by the following reproducible checks:

- `cargo test -p xai-grok-shell --lib`: 6,338 passed, 0 failed, 8 ignored;
- `cargo test -p xai-grok-pager --lib`: 8,259 passed, 0 failed, 10 ignored;
- `cargo test -p xai-grok-tools --lib`: 2,998 passed, 0 failed, 7 ignored;
- `CARGO_INCREMENTAL=0 cargo check -p xai-grok-pager-bin`: passed;
- `cargo fmt --all` and `git diff --check`: passed;
- fork contracts, the OpenAI Codex search contract, patch-stack verification,
  parity-ledger validation, and prospective strict ownership coverage: passed;
- generated root `Cargo.toml` SHA-256: unchanged at
  `28a3ea7e1c859729a0c5cf77f87ff7f0ece319a576697b274917359e11be480b`.

Credential-gated live Codex/Kimi calls are intentionally excluded. No secret,
authenticated payload, or credential-derived identifier was used by these
offline validations.

## Complete 798 raw-path ledger

The rows below are the exhaustive recursive tree delta from Grok tree
`c5c7bdcda32a828efa112883dcd5279ce78714ec` to
`99e3e7c4d8a6c0214101c99e5cedded0325e96be`. Status is computed from raw
`(mode,type,oid)` entries, without rename inference.

| Row | Raw path | Outcome | Evidence |
| ---: | --- | --- | --- |
| 1 | `M` `Cargo.lock` | adopt | `GB-AFBC-RAW` |
| 2 | `M` `Cargo.toml` | not applicable | `GB-A422-028` |
| 3 | `M` `SOURCE_REV` | not applicable | `GB-A422-028` |
| 4 | `M` `clippy.toml` | adopt | `GB-AFBC-RAW` |
| 5 | `M` `crates/build/xai-proto-build/Cargo.toml` | adopt | `GB-AFBC-RAW` |
| 6 | `A` `crates/build/xai-proto-build/src/debug_redact.rs` | adopt | `GB-AFBC-RAW` |
| 7 | `M` `crates/build/xai-proto-build/src/lib.rs` | adopt | `GB-AFBC-RAW` |
| 8 | `A` `crates/build/xai-proto-build/test_data/debug_redact_plain.proto` | adopt | `GB-AFBC-RAW` |
| 9 | `A` `crates/build/xai-proto-build/test_data/debug_redact_test.proto` | adopt | `GB-AFBC-RAW` |
| 10 | `M` `crates/codegen/ptyctl/src/pty.rs` | adopt | `GB-AFBC-RAW` |
| 11 | `M` `crates/codegen/ptyctl/src/styled.rs` | adopt | `GB-AFBC-RAW` |
| 12 | `M` `crates/codegen/ptyctl/src/term.rs` | adopt | `GB-AFBC-RAW` |
| 13 | `M` `crates/codegen/xai-chat-state/src/compaction_utils.rs` | adopt | `GB-AFBC-RAW` |
| 14 | `M` `crates/codegen/xai-codebase-graph/src/scope_graph/graph.rs` | adopt | `GB-AFBC-RAW` |
| 15 | `M` `crates/codegen/xai-fast-worktree/src/db/mod.rs` | adopt | `GB-AFBC-RAW` |
| 16 | `M` `crates/codegen/xai-fast-worktree/src/db/tests.rs` | adopt | `GB-AFBC-RAW` |
| 17 | `M` `crates/codegen/xai-fast-worktree/src/discovery.rs` | adopt | `GB-AFBC-RAW` |
| 18 | `M` `crates/codegen/xai-fast-worktree/src/lib.rs` | adopt | `GB-AFBC-RAW` |
| 19 | `M` `crates/codegen/xai-file-utils/src/queue.rs` | adopt | `GB-AFBC-RAW` |
| 20 | `M` `crates/codegen/xai-file-utils/src/s3.rs` | adopt | `GB-AFBC-RAW` |
| 21 | `M` `crates/codegen/xai-file-utils/src/storage_client.rs` | adopt | `GB-AFBC-RAW` |
| 22 | `A` `crates/codegen/xai-fsnotify/src/checkout.rs` | adopt | `GB-AFBC-RAW` |
| 23 | `A` `crates/codegen/xai-fsnotify/src/checkout_tests.rs` | adopt | `GB-AFBC-RAW` |
| 24 | `M` `crates/codegen/xai-fsnotify/src/lib.rs` | adopt | `GB-AFBC-RAW` |
| 25 | `M` `crates/codegen/xai-fsnotify/src/watcher.rs` | adopt | `GB-AFBC-RAW` |
| 26 | `A` `crates/codegen/xai-grok-auth/src/bearer_fragment.rs` | not applicable | `GB-AFBC-RAW` |
| 27 | `M` `crates/codegen/xai-grok-auth/src/lib.rs` | adopt | `GB-AFBC-RAW` |
| 28 | `M` `crates/codegen/xai-grok-auth/src/retry_middleware.rs` | adopt | `GB-AFBC-RAW` |
| 29 | `M` `crates/codegen/xai-grok-config-types/src/lib.rs` | adopt | `GB-AFBC-RAW` |
| 30 | `M` `crates/codegen/xai-grok-http/src/lib.rs` | adopt | `GB-AFBC-RAW` |
| 31 | `M` `crates/codegen/xai-grok-markdown/Cargo.toml` | adopt | `GB-AFBC-RAW` |
| 32 | `M` `crates/codegen/xai-grok-markdown/src/hyperlinks.rs` | adopt | `GB-AFBC-RAW` |
| 33 | `M` `crates/codegen/xai-grok-markdown/src/parse.rs` | adopt | `GB-AFBC-RAW` |
| 34 | `M` `crates/codegen/xai-grok-markdown/src/render.rs` | adopt | `GB-AFBC-RAW` |
| 35 | `M` `crates/codegen/xai-grok-pager-bin/Cargo.toml` | adopt | `GB-AFBC-RAW` |
| 36 | `M` `crates/codegen/xai-grok-pager-bin/src/main.rs` | adopt | `GB-AFBC-RAW` |
| 37 | `M` `crates/codegen/xai-grok-pager-minimal/src/live.rs` | adopt | `GB-AFBC-RAW` |
| 38 | `M` `crates/codegen/xai-grok-pager-minimal/src/overlay.rs` | adopt | `GB-AFBC-RAW` |
| 39 | `M` `crates/codegen/xai-grok-pager-minimal/src/panel.rs` | adopt | `GB-AFBC-RAW` |
| 40 | `M` `crates/codegen/xai-grok-pager-minimal/src/plan.rs` | adopt | `GB-AFBC-RAW` |
| 41 | `M` `crates/codegen/xai-grok-pager-pty-harness/src/pty.rs` | adopt | `GB-AFBC-RAW` |
| 42 | `M` `crates/codegen/xai-grok-pager-render/src/appearance/config.rs` | adopt | `GB-AFBC-RAW` |
| 43 | `M` `crates/codegen/xai-grok-pager-render/src/gboom/mod.rs` | adopt | `GB-AFBC-RAW` |
| 44 | `M` `crates/codegen/xai-grok-pager-render/src/render/image_overlay.rs` | adopt | `GB-AFBC-RAW` |
| 45 | `M` `crates/codegen/xai-grok-pager-render/src/render/image_overlay/content.rs` | adopt | `GB-AFBC-RAW` |
| 46 | `M` `crates/codegen/xai-grok-pager-render/src/render/image_overlay/tests.rs` | adopt | `GB-AFBC-RAW` |
| 47 | `M` `crates/codegen/xai-grok-pager-render/src/render/line_utils.rs` | adopt | `GB-AFBC-RAW` |
| 48 | `M` `crates/codegen/xai-grok-pager-render/src/render/scrollbar.rs` | adopt | `GB-AFBC-RAW` |
| 49 | `M` `crates/codegen/xai-grok-pager-render/src/terminal/mod.rs` | adopt | `GB-AFBC-RAW` |
| 50 | `A` `crates/codegen/xai-grok-pager-render/src/terminal/tmux.rs` | adopt | `GB-AFBC-RAW` |
| 51 | `M` `crates/codegen/xai-grok-pager-render/src/terminal/tmux_probe.rs` | adopt | `GB-AFBC-RAW` |
| 52 | `M` `crates/codegen/xai-grok-pager-render/src/theme/cache.rs` | adopt | `GB-AFBC-RAW` |
| 53 | `A` `crates/codegen/xai-grok-pager-render/src/theme/env_appearance.rs` | adopt | `GB-AFBC-RAW` |
| 54 | `M` `crates/codegen/xai-grok-pager-render/src/theme/grokday.rs` | adopt | `GB-AFBC-RAW` |
| 55 | `M` `crates/codegen/xai-grok-pager-render/src/theme/groknight.rs` | adopt | `GB-AFBC-RAW` |
| 56 | `M` `crates/codegen/xai-grok-pager-render/src/theme/mod.rs` | adopt | `GB-AFBC-RAW` |
| 57 | `M` `crates/codegen/xai-grok-pager-render/src/theme/osc11.rs` | adopt | `GB-AFBC-RAW` |
| 58 | `M` `crates/codegen/xai-grok-pager-render/src/theme/oscura.rs` | adopt | `GB-AFBC-RAW` |
| 59 | `M` `crates/codegen/xai-grok-pager-render/src/theme/rosepine.rs` | adopt | `GB-AFBC-RAW` |
| 60 | `M` `crates/codegen/xai-grok-pager-render/src/theme/system_appearance.rs` | adopt | `GB-AFBC-RAW` |
| 61 | `M` `crates/codegen/xai-grok-pager-render/src/theme/terminal_default.rs` | adopt | `GB-AFBC-RAW` |
| 62 | `M` `crates/codegen/xai-grok-pager-render/src/theme/tokyonight.rs` | adopt | `GB-AFBC-RAW` |
| 63 | `M` `crates/codegen/xai-grok-pager-render/src/util.rs` | adopt | `GB-AFBC-RAW` |
| 64 | `M` `crates/codegen/xai-grok-pager/Cargo.toml` | adopt | `GB-AFBC-RAW` |
| 65 | `M` `crates/codegen/xai-grok-pager/docs/tutorial/01-coming-from-another-tool.md` | adopt | `GB-AFBC-RAW` |
| 66 | `M` `crates/codegen/xai-grok-pager/docs/tutorial/05-slash-commands.md` | adopt | `GB-AFBC-RAW` |
| 67 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/02-authentication.md` | adopt | `GB-AFBC-RAW` |
| 68 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/03-keyboard-shortcuts.md` | adopt | `GB-AFBC-RAW` |
| 69 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/04-slash-commands.md` | adopt | `GB-AFBC-RAW` |
| 70 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/05-configuration.md` | adopt | `GB-AFBC-RAW` |
| 71 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/06-theming.md` | adopt | `GB-AFBC-RAW` |
| 72 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/08-skills.md` | adopt | `GB-AFBC-RAW` |
| 73 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/17-sessions.md` | adopt | `GB-AFBC-RAW` |
| 74 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/18-sandbox.md` | adopt | `GB-AFBC-RAW` |
| 75 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/19-plan-mode.md` | adopt | `GB-AFBC-RAW` |
| 76 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/21-terminal-support.md` | adopt | `GB-AFBC-RAW` |
| 77 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/22-permissions-and-safety.md` | adopt | `GB-AFBC-RAW` |
| 78 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/23-dashboard.md` | adopt | `GB-AFBC-RAW` |
| 79 | `M` `crates/codegen/xai-grok-pager/docs/user-guide/24-monitoring-usage.md` | adopt | `GB-AFBC-RAW` |
| 80 | `M` `crates/codegen/xai-grok-pager/src/acp/mod.rs` | adopt | `GB-AFBC-RAW` |
| 81 | `M` `crates/codegen/xai-grok-pager/src/acp/spawn.rs` | adopt | `GB-AFBC-RAW` |
| 82 | `A` `crates/codegen/xai-grok-pager/src/acp/version_mismatch.rs` | adopt | `GB-AFBC-RAW` |
| 83 | `A` `crates/codegen/xai-grok-pager/src/acp/version_mismatch_tests.rs` | adopt | `GB-AFBC-RAW` |
| 84 | `M` `crates/codegen/xai-grok-pager/src/actions/defaults.rs` | adopt | `GB-AFBC-RAW` |
| 85 | `M` `crates/codegen/xai-grok-pager/src/actions/mod.rs` | adopt | `GB-AFBC-RAW` |
| 86 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/background.rs` | adopt | `GB-AFBC-RAW` |
| 87 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/interactions.rs` | adopt | `GB-AFBC-RAW` |
| 88 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/mod.rs` | adopt | `GB-AFBC-RAW` |
| 89 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/permissions.rs` | adopt | `GB-AFBC-RAW` |
| 90 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/prompt_origin.rs` | adopt | `GB-AFBC-RAW` |
| 91 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/queue.rs` | adopt | `GB-AFBC-RAW` |
| 92 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/session_notification.rs` | adopt | `GB-AFBC-RAW` |
| 93 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/background_tasks.rs` | adopt | `GB-AFBC-RAW` |
| 94 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/mod.rs` | adopt | `GB-AFBC-RAW` |
| 95 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/permissions.rs` | adopt | `GB-AFBC-RAW` |
| 96 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/queue_and_adoption.rs` | adopt | `GB-AFBC-RAW` |
| 97 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/session_events.rs` | adopt | `GB-AFBC-RAW` |
| 98 | `M` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/turn_completion.rs` | adopt | `GB-AFBC-RAW` |
| 99 | `A` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/version_mismatch.rs` | adopt | `GB-AFBC-RAW` |
| 100 | `M` `crates/codegen/xai-grok-pager/src/app/actions.rs` | adopt | `GB-AFBC-RAW` |
| 101 | `M` `crates/codegen/xai-grok-pager/src/app/agent.rs` | adopt | `GB-AFBC-RAW` |
| 102 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/cta.rs` | adopt | `GB-AFBC-RAW` |
| 103 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/input.rs` | adopt | `GB-AFBC-RAW` |
| 104 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/interactions.rs` | adopt | `GB-AFBC-RAW` |
| 105 | `A` `crates/codegen/xai-grok-pager/src/app/agent_view/key_owner.rs` | adopt | `GB-AFBC-RAW` |
| 106 | `A` `crates/codegen/xai-grok-pager/src/app/agent_view/key_owner_tests.rs` | adopt | `GB-AFBC-RAW` |
| 107 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/links.rs` | adopt | `GB-AFBC-RAW` |
| 108 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/mod.rs` | adopt | `GB-AFBC-RAW` |
| 109 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/modals.rs` | adopt | `GB-AFBC-RAW` |
| 110 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/panes.rs` | adopt | `GB-AFBC-RAW` |
| 111 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/paste.rs` | adopt | `GB-AFBC-RAW` |
| 112 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/plan.rs` | adopt | `GB-AFBC-RAW` |
| 113 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/prompt.rs` | adopt | `GB-AFBC-RAW` |
| 114 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/queue.rs` | adopt | `GB-AFBC-RAW` |
| 115 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/render.rs` | adopt | `GB-AFBC-RAW` |
| 116 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/rewind.rs` | adopt | `GB-AFBC-RAW` |
| 117 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/selection.rs` | adopt | `GB-AFBC-RAW` |
| 118 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/session.rs` | adopt | `GB-AFBC-RAW` |
| 119 | `M` `crates/codegen/xai-grok-pager/src/app/agent_view/viewer.rs` | adopt | `GB-AFBC-RAW` |
| 120 | `A` `crates/codegen/xai-grok-pager/src/app/agent_view/viewer_tests.rs` | adopt | `GB-AFBC-RAW` |
| 121 | `M` `crates/codegen/xai-grok-pager/src/app/app_view.rs` | adopt | `GB-AFBC-RAW` |
| 122 | `M` `crates/codegen/xai-grok-pager/src/app/cli.rs` | adopt | `GB-AFBC-RAW` |
| 123 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/auth.rs` | adopt | `GB-AFBC-RAW` |
| 124 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/billing.rs` | adopt | `GB-AFBC-RAW` |
| 125 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/ctx.rs` | adopt | `GB-AFBC-RAW` |
| 126 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/dashboard.rs` | adopt | `GB-AFBC-RAW` |
| 127 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/interject.rs` | adopt | `GB-AFBC-RAW` |
| 128 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/mod.rs` | adopt | `GB-AFBC-RAW` |
| 129 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/modes.rs` | adopt | `GB-AFBC-RAW` |
| 130 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/notes.rs` | adopt | `GB-AFBC-RAW` |
| 131 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/permissions.rs` | adopt | `GB-AFBC-RAW` |
| 132 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/prompt.rs` | adopt | `GB-AFBC-RAW` |
| 133 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/queue.rs` | adopt | `GB-AFBC-RAW` |
| 134 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/rewind.rs` | adopt | `GB-AFBC-RAW` |
| 135 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/router.rs` | adopt | `GB-AFBC-RAW` |
| 136 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/session/foreign.rs` | adopt | `GB-AFBC-RAW` |
| 137 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/session/fork.rs` | adopt | `GB-AFBC-RAW` |
| 138 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs` | adopt | `GB-AFBC-RAW` |
| 139 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/session/load.rs` | adopt | `GB-AFBC-RAW` |
| 140 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/settings/setters.rs` | adopt | `GB-AFBC-RAW` |
| 141 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/settings/ui.rs` | adopt | `GB-AFBC-RAW` |
| 142 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/status.rs` | adopt | `GB-AFBC-RAW` |
| 143 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/task_result.rs` | adopt | `GB-AFBC-RAW` |
| 144 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/billing.rs` | adopt | `GB-AFBC-RAW` |
| 145 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/dashboard.rs` | adopt | `GB-AFBC-RAW` |
| 146 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/mod.rs` | adopt | `GB-AFBC-RAW` |
| 147 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/modes.rs` | adopt | `GB-AFBC-RAW` |
| 148 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/notes.rs` | adopt | `GB-AFBC-RAW` |
| 149 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/permissions.rs` | adopt | `GB-AFBC-RAW` |
| 150 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/prompt.rs` | adopt | `GB-AFBC-RAW` |
| 151 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/rewind.rs` | adopt | `GB-AFBC-RAW` |
| 152 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/router.rs` | adopt | `GB-AFBC-RAW` |
| 153 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/fork.rs` | adopt | `GB-AFBC-RAW` |
| 154 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/lifecycle.rs` | adopt | `GB-AFBC-RAW` |
| 155 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/load.rs` | adopt | `GB-AFBC-RAW` |
| 156 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/modal.rs` | adopt | `GB-AFBC-RAW` |
| 157 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/take_deferred.rs` | adopt | `GB-AFBC-RAW` |
| 158 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/settings.rs` | adopt | `GB-AFBC-RAW` |
| 159 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/status.rs` | adopt | `GB-AFBC-RAW` |
| 160 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/task_result.rs` | adopt | `GB-AFBC-RAW` |
| 161 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/turn.rs` | adopt | `GB-AFBC-RAW` |
| 162 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/transcript.rs` | adopt | `GB-AFBC-RAW` |
| 163 | `M` `crates/codegen/xai-grok-pager/src/app/dispatch/turn.rs` | adopt | `GB-AFBC-RAW` |
| 164 | `M` `crates/codegen/xai-grok-pager/src/app/effects/helpers.rs` | adopt | `GB-AFBC-RAW` |
| 165 | `M` `crates/codegen/xai-grok-pager/src/app/effects/mod.rs` | adopt | `GB-AFBC-RAW` |
| 166 | `M` `crates/codegen/xai-grok-pager/src/app/effects/tests.rs` | adopt | `GB-AFBC-RAW` |
| 167 | `A` `crates/codegen/xai-grok-pager/src/app/error_display.rs` | adopt | `GB-AFBC-RAW` |
| 168 | `M` `crates/codegen/xai-grok-pager/src/app/event_loop.rs` | adopt | `GB-AFBC-RAW` |
| 169 | `M` `crates/codegen/xai-grok-pager/src/app/foreign_sessions.rs` | adopt | `GB-AFBC-RAW` |
| 170 | `M` `crates/codegen/xai-grok-pager/src/app/leader_cluster/mod.rs` | adopt | `GB-AFBC-RAW` |
| 171 | `M` `crates/codegen/xai-grok-pager/src/app/mod.rs` | adopt | `GB-AFBC-RAW` |
| 172 | `M` `crates/codegen/xai-grok-pager/src/app/modals.rs` | adopt | `GB-AFBC-RAW` |
| 173 | `M` `crates/codegen/xai-grok-pager/src/app/mouse.rs` | adopt | `GB-AFBC-RAW` |
| 174 | `M` `crates/codegen/xai-grok-pager/src/app/roster.rs` | adopt | `GB-AFBC-RAW` |
| 175 | `M` `crates/codegen/xai-grok-pager/src/app/session_startup.rs` | adopt | `GB-AFBC-RAW` |
| 176 | `M` `crates/codegen/xai-grok-pager/src/app/session_title_resolve_tests.rs` | adopt | `GB-AFBC-RAW` |
| 177 | `M` `crates/codegen/xai-grok-pager/src/app/turn_completion.rs` | adopt | `GB-AFBC-RAW` |
| 178 | `M` `crates/codegen/xai-grok-pager/src/app/turn_completion/tests.rs` | adopt | `GB-AFBC-RAW` |
| 179 | `M` `crates/codegen/xai-grok-pager/src/app/xt_filter.rs` | adopt | `GB-AFBC-RAW` |
| 180 | `M` `crates/codegen/xai-grok-pager/src/config_toml_edit.rs` | adopt | `GB-AFBC-RAW` |
| 181 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/doctor_format_tests.rs` | adopt | `GB-AFBC-RAW` |
| 182 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/fix.rs` | adopt | `GB-AFBC-RAW` |
| 183 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/fix_tests.rs` | adopt | `GB-AFBC-RAW` |
| 184 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/mod.rs` | adopt | `GB-AFBC-RAW` |
| 185 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/model.rs` | adopt | `GB-AFBC-RAW` |
| 186 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/probes/mod.rs` | adopt | `GB-AFBC-RAW` |
| 187 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/probes/tmux.rs` | adopt | `GB-AFBC-RAW` |
| 188 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/view.rs` | adopt | `GB-AFBC-RAW` |
| 189 | `M` `crates/codegen/xai-grok-pager/src/diagnostics/view_tests.rs` | adopt | `GB-AFBC-RAW` |
| 190 | `A` `crates/codegen/xai-grok-pager/src/disk_usage_cmd/display.rs` | adopt | `GB-AFBC-RAW` |
| 191 | `A` `crates/codegen/xai-grok-pager/src/disk_usage_cmd/mod.rs` | adopt | `GB-AFBC-RAW` |
| 192 | `A` `crates/codegen/xai-grok-pager/src/disk_usage_cmd/tests.rs` | adopt | `GB-AFBC-RAW` |
| 193 | `M` `crates/codegen/xai-grok-pager/src/doctor_cmd/tests.rs` | adopt | `GB-AFBC-RAW` |
| 194 | `A` `crates/codegen/xai-grok-pager/src/fs_size.rs` | adopt | `GB-AFBC-RAW` |
| 195 | `A` `crates/codegen/xai-grok-pager/src/fs_size_tests.rs` | adopt | `GB-AFBC-RAW` |
| 196 | `M` `crates/codegen/xai-grok-pager/src/git_info.rs` | adopt | `GB-AFBC-RAW` |
| 197 | `M` `crates/codegen/xai-grok-pager/src/headless.rs` | adopt | `GB-AFBC-RAW` |
| 198 | `M` `crates/codegen/xai-grok-pager/src/headless/ext_protocol.rs` | adopt | `GB-AFBC-RAW` |
| 199 | `M` `crates/codegen/xai-grok-pager/src/headless/ext_protocol_tests.rs` | adopt | `GB-AFBC-RAW` |
| 200 | `M` `crates/codegen/xai-grok-pager/src/headless_tests.rs` | adopt | `GB-AFBC-RAW` |
| 201 | `M` `crates/codegen/xai-grok-pager/src/input/key.rs` | adopt | `GB-AFBC-RAW` |
| 202 | `M` `crates/codegen/xai-grok-pager/src/lib.rs` | adopt | `GB-AFBC-RAW` |
| 203 | `M` `crates/codegen/xai-grok-pager/src/memory_trace.rs` | adopt | `GB-AFBC-RAW` |
| 204 | `M` `crates/codegen/xai-grok-pager/src/models.rs` | adopt | `GB-AFBC-RAW` |
| 205 | `M` `crates/codegen/xai-grok-pager/src/notifications/protocol.rs` | adopt | `GB-AFBC-RAW` |
| 206 | `M` `crates/codegen/xai-grok-pager/src/notifications/tmux.rs` | adopt | `GB-AFBC-RAW` |
| 207 | `D` `crates/codegen/xai-grok-pager/src/project_picker/mod.rs` | adopt | `GB-AFBC-RAW` |
| 208 | `D` `crates/codegen/xai-grok-pager/src/project_picker/sources.rs` | adopt | `GB-AFBC-RAW` |
| 209 | `M` `crates/codegen/xai-grok-pager/src/pty_wrap.rs` | adopt | `GB-AFBC-RAW` |
| 210 | `A` `crates/codegen/xai-grok-pager/src/recent_dirs.rs` | adopt | `GB-AFBC-RAW` |
| 211 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/context_info.rs` | adopt | `GB-AFBC-RAW` |
| 212 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/markdown_content.rs` | adopt | `GB-AFBC-RAW` |
| 213 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/session_event.rs` | adopt | `GB-AFBC-RAW` |
| 214 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/edit.rs` | adopt | `GB-AFBC-RAW` |
| 215 | `M` `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/web_fetch.rs` | adopt | `GB-AFBC-RAW` |
| 216 | `M` `crates/codegen/xai-grok-pager/src/scrollback/scrollback_pane.rs` | adopt | `GB-AFBC-RAW` |
| 217 | `M` `crates/codegen/xai-grok-pager/src/scrollback/selection.rs` | adopt | `GB-AFBC-RAW` |
| 218 | `M` `crates/codegen/xai-grok-pager/src/scrollback/state/nav.rs` | adopt | `GB-AFBC-RAW` |
| 219 | `M` `crates/codegen/xai-grok-pager/src/scrollback/sticky.rs` | adopt | `GB-AFBC-RAW` |
| 220 | `M` `crates/codegen/xai-grok-pager/src/scrollback/text_selection.rs` | adopt | `GB-AFBC-RAW` |
| 221 | `M` `crates/codegen/xai-grok-pager/src/scrollback/types.rs` | adopt | `GB-AFBC-RAW` |
| 222 | `M` `crates/codegen/xai-grok-pager/src/settings/defs.rs` | adopt | `GB-AFBC-RAW` |
| 223 | `M` `crates/codegen/xai-grok-pager/src/settings/registry.rs` | adopt | `GB-AFBC-RAW` |
| 224 | `M` `crates/codegen/xai-grok-pager/src/slash/acp_command.rs` | adopt | `GB-AFBC-RAW` |
| 225 | `M` `crates/codegen/xai-grok-pager/src/slash/command.rs` | adopt | `GB-AFBC-RAW` |
| 226 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/delete.rs` | adopt | `GB-AFBC-RAW` |
| 227 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/exit.rs` | adopt | `GB-AFBC-RAW` |
| 228 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/feedback.rs` | adopt | `GB-AFBC-RAW` |
| 229 | `M` `crates/codegen/xai-grok-pager/src/slash/commands/mod.rs` | adopt | `GB-AFBC-RAW` |
| 230 | `M` `crates/codegen/xai-grok-pager/src/slash/mod.rs` | adopt | `GB-AFBC-RAW` |
| 231 | `M` `crates/codegen/xai-grok-pager/src/slash/registry.rs` | adopt | `GB-AFBC-RAW` |
| 232 | `M` `crates/codegen/xai-grok-pager/src/test_util.rs` | adopt | `GB-AFBC-RAW` |
| 233 | `M` `crates/codegen/xai-grok-pager/src/trace_cmd.rs` | adopt | `GB-AFBC-RAW` |
| 234 | `M` `crates/codegen/xai-grok-pager/src/views/agent.rs` | adopt | `GB-AFBC-RAW` |
| 235 | `M` `crates/codegen/xai-grok-pager/src/views/block_viewer.rs` | adopt | `GB-AFBC-RAW` |
| 236 | `M` `crates/codegen/xai-grok-pager/src/views/btw_overlay.rs` | adopt | `GB-AFBC-RAW` |
| 237 | `M` `crates/codegen/xai-grok-pager/src/views/dashboard/mod.rs` | adopt | `GB-AFBC-RAW` |
| 238 | `M` `crates/codegen/xai-grok-pager/src/views/dashboard/render.rs` | adopt | `GB-AFBC-RAW` |
| 239 | `M` `crates/codegen/xai-grok-pager/src/views/dashboard/row.rs` | adopt | `GB-AFBC-RAW` |
| 240 | `M` `crates/codegen/xai-grok-pager/src/views/dashboard/state.rs` | adopt | `GB-AFBC-RAW` |
| 241 | `M` `crates/codegen/xai-grok-pager/src/views/extensions_modal.rs` | adopt | `GB-AFBC-RAW` |
| 242 | `M` `crates/codegen/xai-grok-pager/src/views/file_search/line_viewer.rs` | adopt | `GB-AFBC-RAW` |
| 243 | `M` `crates/codegen/xai-grok-pager/src/views/list_pane/state/methods.rs` | adopt | `GB-AFBC-RAW` |
| 244 | `M` `crates/codegen/xai-grok-pager/src/views/list_pane/state/mod.rs` | adopt | `GB-AFBC-RAW` |
| 245 | `M` `crates/codegen/xai-grok-pager/src/views/memory_modal.rs` | adopt | `GB-AFBC-RAW` |
| 246 | `M` `crates/codegen/xai-grok-pager/src/views/mod.rs` | adopt | `GB-AFBC-RAW` |
| 247 | `M` `crates/codegen/xai-grok-pager/src/views/modal.rs` | adopt | `GB-AFBC-RAW` |
| 248 | `M` `crates/codegen/xai-grok-pager/src/views/permission_view.rs` | adopt | `GB-AFBC-RAW` |
| 249 | `M` `crates/codegen/xai-grok-pager/src/views/picker.rs` | adopt | `GB-AFBC-RAW` |
| 250 | `M` `crates/codegen/xai-grok-pager/src/views/prompt_widget/mod.rs` | adopt | `GB-AFBC-RAW` |
| 251 | `M` `crates/codegen/xai-grok-pager/src/views/question_view.rs` | adopt | `GB-AFBC-RAW` |
| 252 | `M` `crates/codegen/xai-grok-pager/src/views/rewind.rs` | adopt | `GB-AFBC-RAW` |
| 253 | `M` `crates/codegen/xai-grok-pager/src/views/session_picker.rs` | adopt | `GB-AFBC-RAW` |
| 254 | `M` `crates/codegen/xai-grok-pager/src/views/session_title.rs` | adopt | `GB-AFBC-RAW` |
| 255 | `M` `crates/codegen/xai-grok-pager/src/views/settings_modal/state.rs` | adopt | `GB-AFBC-RAW` |
| 256 | `M` `crates/codegen/xai-grok-pager/src/views/settings_modal/tests.rs` | adopt | `GB-AFBC-RAW` |
| 257 | `M` `crates/codegen/xai-grok-pager/src/views/shortcuts_bar.rs` | adopt | `GB-AFBC-RAW` |
| 258 | `M` `crates/codegen/xai-grok-pager/src/views/shortcuts_help.rs` | adopt | `GB-AFBC-RAW` |
| 259 | `M` `crates/codegen/xai-grok-pager/src/views/slash_dropdown.rs` | adopt | `GB-AFBC-RAW` |
| 260 | `M` `crates/codegen/xai-grok-pager/src/views/turn_status.rs` | adopt | `GB-AFBC-RAW` |
| 261 | `A` `crates/codegen/xai-grok-pager/src/views/usage_modal.rs` | adopt | `GB-AFBC-RAW` |
| 262 | `M` `crates/codegen/xai-grok-pager/src/views/welcome/hero_box.rs` | adopt | `GB-AFBC-RAW` |
| 263 | `M` `crates/codegen/xai-grok-pager/src/views/welcome/mod.rs` | adopt | `GB-AFBC-RAW` |
| 264 | `A` `crates/codegen/xai-grok-pager/src/views/welcome/workspace_mode.rs` | adopt | `GB-AFBC-RAW` |
| 265 | `M` `crates/codegen/xai-grok-pager/src/worktree_cmd/display.rs` | adopt | `GB-AFBC-RAW` |
| 266 | `M` `crates/codegen/xai-grok-pager/src/worktree_cmd/mod.rs` | adopt | `GB-AFBC-RAW` |
| 267 | `M` `crates/codegen/xai-grok-pager/src/wrap_cmd.rs` | adopt | `GB-AFBC-RAW` |
| 268 | `M` `crates/codegen/xai-grok-pager/tests/grok_home_paths.rs` | adopt | `GB-AFBC-RAW` |
| 269 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/auto_wake_cancel_preserves_queued_user_prompt.rs` | adopt | `GB-AFBC-RAW` |
| 270 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/auto_wake_cancel_via_esc_preserves_queued_user_prompt.rs` | adopt | `GB-AFBC-RAW` |
| 271 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/auto_wake_cancel_via_stop_click_preserves_queued_user_prompt.rs` | adopt | `GB-AFBC-RAW` |
| 272 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/cancel_then_resend_prompt_appears_once.rs` | adopt | `GB-AFBC-RAW` |
| 273 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/common.rs` | adopt | `GB-AFBC-RAW` |
| 274 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/feedback_slash_opens_descriptive_pane.rs` | adopt | `GB-AFBC-RAW` |
| 275 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/iterm_readline_editing.rs` | adopt | `GB-AFBC-RAW` |
| 276 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/mcp_menu_loads_servers_in_non_project_dir.rs` | adopt | `GB-AFBC-RAW` |
| 277 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/mcp_menu_loads_servers_in_project_dir.rs` | adopt | `GB-AFBC-RAW` |
| 278 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_feedback_session_gate_and_pane.rs` | adopt | `GB-AFBC-RAW` |
| 279 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_quit_resets_bracketed_paste.rs` | adopt | `GB-AFBC-RAW` |
| 280 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/mod.rs` | adopt | `GB-AFBC-RAW` |
| 281 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/plan_scrollbar_grab_zone_pty.rs` | adopt | `GB-AFBC-RAW` |
| 282 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/question_tab_cycles_answers.rs` | adopt | `GB-AFBC-RAW` |
| 283 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/queue_reorder_local_row_above_server_row.rs` | adopt | `GB-AFBC-RAW` |
| 284 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/queue_reorder_moves_row_up.rs` | adopt | `GB-AFBC-RAW` |
| 285 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/response_top_indicator_pty.rs` | adopt | `GB-AFBC-RAW` |
| 286 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/sticky_header_drag_copy_pty.rs` | adopt | `GB-AFBC-RAW` |
| 287 | `A` `crates/codegen/xai-grok-pager/tests/pty_e2e/wrap_appearance_env_advertised_through_shell.rs` | adopt | `GB-AFBC-RAW` |
| 288 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e_queue.rs` | adopt | `GB-AFBC-RAW` |
| 289 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e_scroll_selection.rs` | adopt | `GB-AFBC-RAW` |
| 290 | `M` `crates/codegen/xai-grok-pager/tests/pty_e2e_smoke.rs` | adopt | `GB-AFBC-RAW` |
| 291 | `M` `crates/codegen/xai-grok-pager/tests/scenarios/inline_edit_dismiss_returns_to_editor.yaml` | adopt | `GB-AFBC-RAW` |
| 292 | `M` `crates/codegen/xai-grok-pager/tests/scenarios/inline_edit_resubmit.yaml` | adopt | `GB-AFBC-RAW` |
| 293 | `M` `crates/codegen/xai-grok-pager/tests/scenarios/inline_edit_unchanged_exit.yaml` | adopt | `GB-AFBC-RAW` |
| 294 | `M` `crates/codegen/xai-grok-pager/tests/scripted_scenarios.rs` | adopt | `GB-AFBC-RAW` |
| 295 | `M` `crates/codegen/xai-grok-pager/tests/settings_e2e.rs` | adopt | `GB-AFBC-RAW` |
| 296 | `A` `crates/codegen/xai-grok-pager/tests/signal_errno_preservation.rs` | adopt | `GB-AFBC-RAW` |
| 297 | `M` `crates/codegen/xai-grok-sampler/Cargo.toml` | adopt | `GB-AFBC-RAW` |
| 298 | `M` `crates/codegen/xai-grok-sampler/src/actor/request_task.rs` | adopt | `GB-AFBC-RAW` |
| 299 | `M` `crates/codegen/xai-grok-sampler/src/attribution.rs` | adopt | `GB-AFBC-RAW` |
| 300 | `M` `crates/codegen/xai-grok-sampler/src/client.rs` | adopt | `GB-AFBC-RAW` |
| 301 | `M` `crates/codegen/xai-grok-sampler/src/events.rs` | adopt | `GB-AFBC-RAW` |
| 302 | `M` `crates/codegen/xai-grok-sampler/src/handle.rs` | adopt | `GB-AFBC-RAW` |
| 303 | `M` `crates/codegen/xai-grok-sampler/src/lib.rs` | adopt | `GB-AFBC-RAW` |
| 304 | `M` `crates/codegen/xai-grok-sampler/src/retry.rs` | adopt | `GB-AFBC-RAW` |
| 305 | `M` `crates/codegen/xai-grok-sampler/src/stream/collect.rs` | adopt | `GB-AFBC-RAW` |
| 306 | `M` `crates/codegen/xai-grok-sampler/tests/cf_edge_error_message.rs` | adopt | `GB-AFBC-RAW` |
| 307 | `M` `crates/codegen/xai-grok-sampling-types/Cargo.toml` | adopt | `GB-AFBC-RAW` |
| 308 | `M` `crates/codegen/xai-grok-sampling-types/src/conversation.rs` | adopt | `GB-AFBC-RAW` |
| 309 | `M` `crates/codegen/xai-grok-sampling-types/src/error.rs` | adopt | `GB-AFBC-RAW` |
| 310 | `M` `crates/codegen/xai-grok-sampling-types/src/lib.rs` | adopt | `GB-AFBC-RAW` |
| 311 | `A` `crates/codegen/xai-grok-sampling-types/src/provider_error.rs` | adopt | `GB-AFBC-RAW` |
| 312 | `M` `crates/codegen/xai-grok-sampling-types/src/types.rs` | adopt | `GB-AFBC-RAW` |
| 313 | `M` `crates/codegen/xai-grok-sandbox/src/deny/glob.rs` | adopt | `GB-AFBC-RAW` |
| 314 | `M` `crates/codegen/xai-grok-sandbox/src/deny/mod.rs` | adopt | `GB-AFBC-RAW` |
| 315 | `M` `crates/codegen/xai-grok-sandbox/src/lib.rs` | adopt | `GB-AFBC-RAW` |
| 316 | `M` `crates/codegen/xai-grok-sandbox/tests/deny_paths_e2e.rs` | adopt | `GB-AFBC-RAW` |
| 317 | `M` `crates/codegen/xai-grok-shared/src/ui_config.rs` | adopt | `GB-AFBC-RAW` |
| 318 | `M` `crates/codegen/xai-grok-shell-base/Cargo.toml` | adopt | `GB-AFBC-RAW` |
| 319 | `M` `crates/codegen/xai-grok-shell/CHANGELOG.md` | not applicable | `GB-A422-028` |
| 320 | `M` `crates/codegen/xai-grok-shell/Cargo.toml` | adopt | `GB-AFBC-RAW` |
| 321 | `M` `crates/codegen/xai-grok-shell/README.md` | adopt | `GB-AFBC-RAW` |
| 322 | `M` `crates/codegen/xai-grok-shell/benches/fork_copy.rs` | adopt | `GB-AFBC-RAW` |
| 323 | `M` `crates/codegen/xai-grok-shell/benches/session_list.rs` | adopt | `GB-AFBC-RAW` |
| 324 | `A` `crates/codegen/xai-grok-shell/benches/skills_watcher_startup.rs` | adopt | `GB-AFBC-RAW` |
| 325 | `M` `crates/codegen/xai-grok-shell/changelogs/0.2.115.md` | not applicable | `GB-A422-028` |
| 326 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.117.json` | not applicable | `GB-A422-028` |
| 327 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.117.md` | not applicable | `GB-A422-028` |
| 328 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.118.json` | not applicable | `GB-A422-028` |
| 329 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.118.md` | not applicable | `GB-A422-028` |
| 330 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.119.json` | not applicable | `GB-A422-028` |
| 331 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.119.md` | not applicable | `GB-A422-028` |
| 332 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.120.json` | not applicable | `GB-A422-028` |
| 333 | `A` `crates/codegen/xai-grok-shell/changelogs/0.2.120.md` | not applicable | `GB-A422-028` |
| 334 | `A` `crates/codegen/xai-grok-shell/changelogs/1.0.0.json` | not applicable | `GB-A422-028` |
| 335 | `A` `crates/codegen/xai-grok-shell/changelogs/1.0.0.md` | not applicable | `GB-A422-028` |
| 336 | `M` `crates/codegen/xai-grok-shell/src/active_sessions.rs` | adopt | `GB-AFBC-RAW` |
| 337 | `M` `crates/codegen/xai-grok-shell/src/agent/activity.rs` | adopt | `GB-AFBC-RAW` |
| 338 | `M` `crates/codegen/xai-grok-shell/src/agent/app.rs` | adopt | `GB-AFBC-RAW` |
| 339 | `M` `crates/codegen/xai-grok-shell/src/agent/auth_method.rs` | adopt | `GB-AFBC-RAW` |
| 340 | `M` `crates/codegen/xai-grok-shell/src/agent/chat_modes.rs` | adopt | `GB-AFBC-RAW` |
| 341 | `M` `crates/codegen/xai-grok-shell/src/agent/config.rs` | adopt | `GB-AFBC-RAW` |
| 342 | `M` `crates/codegen/xai-grok-shell/src/agent/ext_parsers.rs` | adopt | `GB-AFBC-RAW` |
| 343 | `M` `crates/codegen/xai-grok-shell/src/agent/feedback_client.rs` | adopt | `GB-AFBC-RAW` |
| 344 | `M` `crates/codegen/xai-grok-shell/src/agent/folder_trust.rs` | adopt | `GB-AFBC-RAW` |
| 345 | `M` `crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs` | adopt | `GB-AFBC-RAW` |
| 346 | `M` `crates/codegen/xai-grok-shell/src/agent/handlers/session.rs` | adopt | `GB-AFBC-RAW` |
| 347 | `M` `crates/codegen/xai-grok-shell/src/agent/handlers/workspaces.rs` | adopt | `GB-AFBC-RAW` |
| 348 | `M` `crates/codegen/xai-grok-shell/src/agent/init.rs` | adopt | `GB-AFBC-RAW` |
| 349 | `M` `crates/codegen/xai-grok-shell/src/agent/mod.rs` | adopt | `GB-AFBC-RAW` |
| 350 | `M` `crates/codegen/xai-grok-shell/src/agent/models.rs` | adopt | `GB-AFBC-RAW` |
| 351 | `M` `crates/codegen/xai-grok-shell/src/agent/models/resolution.rs` | adopt | `GB-AFBC-RAW` |
| 352 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/acp_agent.rs` | adopt | `GB-AFBC-RAW` |
| 353 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs` | adopt | `GB-AFBC-RAW` |
| 354 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/code_nav.rs` | adopt | `GB-AFBC-RAW` |
| 355 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/folder_trust_prompt.rs` | adopt | `GB-AFBC-RAW` |
| 356 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/heap_profile.rs` | adopt | `GB-AFBC-RAW` |
| 357 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/mod.rs` | adopt | `GB-AFBC-RAW` |
| 358 | `A` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/replay.rs` | adopt | `GB-AFBC-RAW` |
| 359 | `A` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/replay_tests.rs` | adopt | `GB-AFBC-RAW` |
| 360 | `A` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/resource_telemetry.rs` | adopt | `GB-AFBC-RAW` |
| 361 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/session_lifecycle.rs` | adopt | `GB-AFBC-RAW` |
| 362 | `A` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/session_registry.rs` | adopt | `GB-AFBC-RAW` |
| 363 | `A` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/session_setup.rs` | adopt | `GB-AFBC-RAW` |
| 364 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/subagent_coordinator.rs` | adopt | `GB-AFBC-RAW` |
| 365 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/tests.rs` | adopt | `GB-AFBC-RAW` |
| 366 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/tests/dhat_soak.rs` | adopt | `GB-AFBC-RAW` |
| 367 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/tests/process_scope_reclaim.rs` | adopt | `GB-AFBC-RAW` |
| 368 | `A` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/tests/session_resume_close_tests.rs` | adopt | `GB-AFBC-RAW` |
| 369 | `M` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/tests/subagent_spawn_context_tests.rs` | adopt | `GB-AFBC-RAW` |
| 370 | `M` `crates/codegen/xai-grok-shell/src/agent/otel_gate.rs` | adopt | `GB-AFBC-RAW` |
| 371 | `M` `crates/codegen/xai-grok-shell/src/agent/proxy.rs` | adopt | `GB-AFBC-RAW` |
| 372 | `M` `crates/codegen/xai-grok-shell/src/agent/relay.rs` | adopt | `GB-AFBC-RAW` |
| 373 | `M` `crates/codegen/xai-grok-shell/src/agent/roster.rs` | adopt | `GB-AFBC-RAW` |
| 374 | `M` `crates/codegen/xai-grok-shell/src/agent/server.rs` | adopt | `GB-AFBC-RAW` |
| 375 | `M` `crates/codegen/xai-grok-shell/src/agent/session_config.rs` | adopt | `GB-AFBC-RAW` |
| 376 | `M` `crates/codegen/xai-grok-shell/src/agent/session_registry_client.rs` | adopt | `GB-AFBC-RAW` |
| 377 | `M` `crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs` | adopt | `GB-AFBC-RAW` |
| 378 | `M` `crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs` | adopt | `GB-AFBC-RAW` |
| 379 | `M` `crates/codegen/xai-grok-shell/src/agent/subagent/tests/mod.rs` | adopt | `GB-AFBC-RAW` |
| 380 | `A` `crates/codegen/xai-grok-shell/src/auth/api_key_probe.rs` | adopt | `GB-AFBC-RAW` |
| 381 | `M` `crates/codegen/xai-grok-shell/src/auth/attribution.rs` | adopt | `GB-AFBC-RAW` |
| 382 | `M` `crates/codegen/xai-grok-shell/src/auth/config.rs` | adopt | `GB-AFBC-RAW` |
| 383 | `M` `crates/codegen/xai-grok-shell/src/auth/credential_provider.rs` | adopt | `GB-AFBC-RAW` |
| 384 | `M` `crates/codegen/xai-grok-shell/src/auth/devbox_login_stub.rs` | adopt | `GB-AFBC-RAW` |
| 385 | `M` `crates/codegen/xai-grok-shell/src/auth/device_code.rs` | adopt | `GB-AFBC-RAW` |
| 386 | `M` `crates/codegen/xai-grok-shell/src/auth/error.rs` | adopt | `GB-AFBC-RAW` |
| 387 | `M` `crates/codegen/xai-grok-shell/src/auth/external_auth.rs` | adopt | `GB-AFBC-RAW` |
| 388 | `M` `crates/codegen/xai-grok-shell/src/auth/flow.rs` | adopt | `GB-AFBC-RAW` |
| 389 | `M` `crates/codegen/xai-grok-shell/src/auth/manager.rs` | adopt | `GB-AFBC-RAW` |
| 390 | `M` `crates/codegen/xai-grok-shell/src/auth/manager/enrichment.rs` | adopt | `GB-AFBC-RAW` |
| 391 | `A` `crates/codegen/xai-grok-shell/src/auth/manager/remedy.rs` | adopt | `GB-AFBC-RAW` |
| 392 | `M` `crates/codegen/xai-grok-shell/src/auth/manager/sleep_gate.rs` | adopt | `GB-AFBC-RAW` |
| 393 | `M` `crates/codegen/xai-grok-shell/src/auth/manager_tests.rs` | adopt | `GB-AFBC-RAW` |
| 394 | `M` `crates/codegen/xai-grok-shell/src/auth/mod.rs` | adopt | `GB-AFBC-RAW` |
| 395 | `M` `crates/codegen/xai-grok-shell/src/auth/model.rs` | adopt | `GB-AFBC-RAW` |
| 396 | `M` `crates/codegen/xai-grok-shell/src/auth/oidc/protocol.rs` | adopt | `GB-AFBC-RAW` |
| 397 | `M` `crates/codegen/xai-grok-shell/src/auth/oidc/refresh.rs` | adopt | `GB-AFBC-RAW` |
| 398 | `M` `crates/codegen/xai-grok-shell/src/auth/recovery.rs` | adopt | `GB-AFBC-RAW` |
| 399 | `M` `crates/codegen/xai-grok-shell/src/auth/refresh/external_refresher.rs` | adopt | `GB-AFBC-RAW` |
| 400 | `M` `crates/codegen/xai-grok-shell/src/auth/refresh/oidc_refresher.rs` | adopt | `GB-AFBC-RAW` |
| 401 | `D` `crates/codegen/xai-grok-shell/src/bin/trace_classify.rs` | adopt | `GB-AFBC-RAW` |
| 402 | `M` `crates/codegen/xai-grok-shell/src/bundle.rs` | adopt | `GB-AFBC-RAW` |
| 403 | `M` `crates/codegen/xai-grok-shell/src/claude_import.rs` | adopt | `GB-AFBC-RAW` |
| 404 | `M` `crates/codegen/xai-grok-shell/src/claude_import_state.rs` | adopt | `GB-AFBC-RAW` |
| 405 | `M` `crates/codegen/xai-grok-shell/src/config/mod.rs` | adopt | `GB-AFBC-RAW` |
| 406 | `M` `crates/codegen/xai-grok-shell/src/config/reloader.rs` | adopt | `GB-AFBC-RAW` |
| 407 | `M` `crates/codegen/xai-grok-shell/src/config/tests.rs` | adopt | `GB-AFBC-RAW` |
| 408 | `M` `crates/codegen/xai-grok-shell/src/config/watcher.rs` | adopt | `GB-AFBC-RAW` |
| 409 | `M` `crates/codegen/xai-grok-shell/src/extensions/bundle.rs` | adopt | `GB-AFBC-RAW` |
| 410 | `M` `crates/codegen/xai-grok-shell/src/extensions/chat_conversation_history.rs` | adopt | `GB-AFBC-RAW` |
| 411 | `M` `crates/codegen/xai-grok-shell/src/extensions/code_nav.rs` | adopt | `GB-AFBC-RAW` |
| 412 | `M` `crates/codegen/xai-grok-shell/src/extensions/debug.rs` | adopt | `GB-AFBC-RAW` |
| 413 | `M` `crates/codegen/xai-grok-shell/src/extensions/feedback.rs` | adopt | `GB-AFBC-RAW` |
| 414 | `M` `crates/codegen/xai-grok-shell/src/extensions/fs.rs` | adopt | `GB-AFBC-RAW` |
| 415 | `M` `crates/codegen/xai-grok-shell/src/extensions/git.rs` | adopt | `GB-AFBC-RAW` |
| 416 | `M` `crates/codegen/xai-grok-shell/src/extensions/hooks.rs` | adopt | `GB-AFBC-RAW` |
| 417 | `M` `crates/codegen/xai-grok-shell/src/extensions/hunk_tracker.rs` | adopt | `GB-AFBC-RAW` |
| 418 | `M` `crates/codegen/xai-grok-shell/src/extensions/jj.rs` | adopt | `GB-AFBC-RAW` |
| 419 | `M` `crates/codegen/xai-grok-shell/src/extensions/marketplace.rs` | adopt | `GB-AFBC-RAW` |
| 420 | `M` `crates/codegen/xai-grok-shell/src/extensions/mcp.rs` | adopt | `GB-AFBC-RAW` |
| 421 | `M` `crates/codegen/xai-grok-shell/src/extensions/memory.rs` | adopt | `GB-AFBC-RAW` |
| 422 | `M` `crates/codegen/xai-grok-shell/src/extensions/mod.rs` | adopt | `GB-AFBC-RAW` |
| 423 | `M` `crates/codegen/xai-grok-shell/src/extensions/notification.rs` | adopt | `GB-AFBC-RAW` |
| 424 | `M` `crates/codegen/xai-grok-shell/src/extensions/plugins.rs` | adopt | `GB-AFBC-RAW` |
| 425 | `M` `crates/codegen/xai-grok-shell/src/extensions/pr.rs` | adopt | `GB-AFBC-RAW` |
| 426 | `M` `crates/codegen/xai-grok-shell/src/extensions/repair.rs` | adopt | `GB-AFBC-RAW` |
| 427 | `M` `crates/codegen/xai-grok-shell/src/extensions/rewind.rs` | adopt | `GB-AFBC-RAW` |
| 428 | `M` `crates/codegen/xai-grok-shell/src/extensions/routing.rs` | adopt | `GB-AFBC-RAW` |
| 429 | `M` `crates/codegen/xai-grok-shell/src/extensions/search.rs` | adopt | `GB-AFBC-RAW` |
| 430 | `M` `crates/codegen/xai-grok-shell/src/extensions/session_admin.rs` | adopt | `GB-AFBC-RAW` |
| 431 | `M` `crates/codegen/xai-grok-shell/src/extensions/session_search.rs` | adopt | `GB-AFBC-RAW` |
| 432 | `M` `crates/codegen/xai-grok-shell/src/extensions/session_state.rs` | adopt | `GB-AFBC-RAW` |
| 433 | `M` `crates/codegen/xai-grok-shell/src/extensions/skills.rs` | adopt | `GB-AFBC-RAW` |
| 434 | `M` `crates/codegen/xai-grok-shell/src/extensions/suggest/file_provider.rs` | adopt | `GB-AFBC-RAW` |
| 435 | `M` `crates/codegen/xai-grok-shell/src/extensions/suggest/history_provider.rs` | adopt | `GB-AFBC-RAW` |
| 436 | `M` `crates/codegen/xai-grok-shell/src/extensions/suggest/mod.rs` | adopt | `GB-AFBC-RAW` |
| 437 | `M` `crates/codegen/xai-grok-shell/src/extensions/suggest/path_provider.rs` | adopt | `GB-AFBC-RAW` |
| 438 | `M` `crates/codegen/xai-grok-shell/src/extensions/task.rs` | adopt | `GB-AFBC-RAW` |
| 439 | `M` `crates/codegen/xai-grok-shell/src/extensions/terminal.rs` | adopt | `GB-AFBC-RAW` |
| 440 | `M` `crates/codegen/xai-grok-shell/src/extensions/worktree.rs` | adopt | `GB-AFBC-RAW` |
| 441 | `M` `crates/codegen/xai-grok-shell/src/heap_profile/monitor.rs` | adopt | `GB-AFBC-RAW` |
| 442 | `M` `crates/codegen/xai-grok-shell/src/inspect/mod.rs` | adopt | `GB-AFBC-RAW` |
| 443 | `M` `crates/codegen/xai-grok-shell/src/leader/client.rs` | adopt | `GB-AFBC-RAW` |
| 444 | `M` `crates/codegen/xai-grok-shell/src/leader/lock.rs` | adopt | `GB-AFBC-RAW` |
| 445 | `M` `crates/codegen/xai-grok-shell/src/leader/mod.rs` | adopt | `GB-AFBC-RAW` |
| 446 | `M` `crates/codegen/xai-grok-shell/src/leader/protocol.rs` | adopt | `GB-AFBC-RAW` |
| 447 | `M` `crates/codegen/xai-grok-shell/src/leader/server.rs` | adopt | `GB-AFBC-RAW` |
| 448 | `M` `crates/codegen/xai-grok-shell/src/leader/transport.rs` | adopt | `GB-AFBC-RAW` |
| 449 | `M` `crates/codegen/xai-grok-shell/src/lib.rs` | adopt | `GB-AFBC-RAW` |
| 450 | `M` `crates/codegen/xai-grok-shell/src/managed_config.rs` | adopt | `GB-AFBC-RAW` |
| 451 | `M` `crates/codegen/xai-grok-shell/src/managed_config/tests.rs` | adopt | `GB-AFBC-RAW` |
| 452 | `M` `crates/codegen/xai-grok-shell/src/plugin.rs` | adopt | `GB-AFBC-RAW` |
| 453 | `M` `crates/codegen/xai-grok-shell/src/relay/sync.rs` | adopt | `GB-AFBC-RAW` |
| 454 | `M` `crates/codegen/xai-grok-shell/src/remote/agent.rs` | adopt | `GB-AFBC-RAW` |
| 455 | `M` `crates/codegen/xai-grok-shell/src/remote/chat_models_client.rs` | adopt | `GB-AFBC-RAW` |
| 456 | `M` `crates/codegen/xai-grok-shell/src/remote/client.rs` | adopt | `GB-AFBC-RAW` |
| 457 | `M` `crates/codegen/xai-grok-shell/src/remote/conversations_client.rs` | adopt | `GB-AFBC-RAW` |
| 458 | `M` `crates/codegen/xai-grok-shell/src/remote/mod.rs` | adopt | `GB-AFBC-RAW` |
| 459 | `M` `crates/codegen/xai-grok-shell/src/remote/pull.rs` | adopt | `GB-AFBC-RAW` |
| 460 | `M` `crates/codegen/xai-grok-shell/src/remote/skills_client.rs` | adopt | `GB-AFBC-RAW` |
| 461 | `M` `crates/codegen/xai-grok-shell/src/remote/sync.rs` | adopt | `GB-AFBC-RAW` |
| 462 | `M` `crates/codegen/xai-grok-shell/src/remote/workspaces_client.rs` | adopt | `GB-AFBC-RAW` |
| 463 | `M` `crates/codegen/xai-grok-shell/src/sampling/conversation.rs` | adopt | `GB-AFBC-RAW` |
| 464 | `A` `crates/codegen/xai-grok-shell/src/sampling/conversation_tests.rs` | adopt | `GB-AFBC-RAW` |
| 465 | `M` `crates/codegen/xai-grok-shell/src/sampling/error.rs` | adopt | `GB-AFBC-RAW` |
| 466 | `M` `crates/codegen/xai-grok-shell/src/session/acp_conversion.rs` | adopt | `GB-AFBC-RAW` |
| 467 | `M` `crates/codegen/xai-grok-shell/src/session/acp_mcp.rs` | adopt | `GB-AFBC-RAW` |
| 468 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session.rs` | adopt | `GB-AFBC-RAW` |
| 469 | `A` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/auth_retry.rs` | adopt | `GB-AFBC-RAW` |
| 470 | `A` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/auth_retry_tests.rs` | adopt | `GB-AFBC-RAW` |
| 471 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/goal.rs` | adopt | `GB-AFBC-RAW` |
| 472 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/goal_support.rs` | adopt | `GB-AFBC-RAW` |
| 473 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/hook_dispatch.rs` | adopt | `GB-AFBC-RAW` |
| 474 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/interjection.rs` | adopt | `GB-AFBC-RAW` |
| 475 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/laziness_classifier.rs` | adopt | `GB-AFBC-RAW` |
| 476 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/mcp.rs` | adopt | `GB-AFBC-RAW` |
| 477 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/memory_dream.rs` | adopt | `GB-AFBC-RAW` |
| 478 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/notification_drain.rs` | adopt | `GB-AFBC-RAW` |
| 479 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/prompt_queue.rs` | adopt | `GB-AFBC-RAW` |
| 480 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/recap.rs` | adopt | `GB-AFBC-RAW` |
| 481 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/rewind.rs` | adopt | `GB-AFBC-RAW` |
| 482 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/run_loop.rs` | adopt | `GB-AFBC-RAW` |
| 483 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs` | adopt | `GB-AFBC-RAW` |
| 484 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/session_mode.rs` | adopt | `GB-AFBC-RAW` |
| 485 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/session_setup.rs` | adopt | `GB-AFBC-RAW` |
| 486 | `A` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/side_call.rs` | adopt | `GB-AFBC-RAW` |
| 487 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/slash_exec.rs` | adopt | `GB-AFBC-RAW` |
| 488 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/spawn.rs` | adopt | `GB-AFBC-RAW` |
| 489 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/stop_gate.rs` | adopt | `GB-AFBC-RAW` |
| 490 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/tasks_cancel.rs` | adopt | `GB-AFBC-RAW` |
| 491 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/tool_calls.rs` | adopt | `GB-AFBC-RAW` |
| 492 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/tool_dispatch.rs` | adopt | `GB-AFBC-RAW` |
| 493 | `A` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/tool_layer_images.rs` | adopt | `GB-AFBC-RAW` |
| 494 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/turn.rs` | adopt | `GB-AFBC-RAW` |
| 495 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/turn_end.rs` | adopt | `GB-AFBC-RAW` |
| 496 | `A` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/turn_summary.rs` | adopt | `GB-AFBC-RAW` |
| 497 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/types.rs` | adopt | `GB-AFBC-RAW` |
| 498 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/updates.rs` | adopt | `GB-AFBC-RAW` |
| 499 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/auth_error_no_retry_tests.rs` | adopt | `GB-AFBC-RAW` |
| 500 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/auto_wake_suppression_tests.rs` | adopt | `GB-AFBC-RAW` |
| 501 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/cancel_running_task_tests.rs` | adopt | `GB-AFBC-RAW` |
| 502 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/idle_resume_tests.rs` | adopt | `GB-AFBC-RAW` |
| 503 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/inline_auto_compact_flow_tests.rs` | adopt | `GB-AFBC-RAW` |
| 504 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/interjection_actor_tests.rs` | adopt | `GB-AFBC-RAW` |
| 505 | `A` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/mcp_connecting_reminder_tests.rs` | adopt | `GB-AFBC-RAW` |
| 506 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/memory_config_tests.rs` | adopt | `GB-AFBC-RAW` |
| 507 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/prompt_mode_transition_tests.rs` | adopt | `GB-AFBC-RAW` |
| 508 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/prompt_queue_actor_tests.rs` | adopt | `GB-AFBC-RAW` |
| 509 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/recap_display_only_tests.rs` | adopt | `GB-AFBC-RAW` |
| 510 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/replay_buffer_send_update_tests.rs` | adopt | `GB-AFBC-RAW` |
| 511 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/support.rs` | adopt | `GB-AFBC-RAW` |
| 512 | `A` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/tool_layer_images_bridge_tests.rs` | adopt | `GB-AFBC-RAW` |
| 513 | `A` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/turn/auth_retry_budget_tests.rs` | adopt | `GB-AFBC-RAW` |
| 514 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/turn/chat_history_integrity_tests.rs` | adopt | `GB-AFBC-RAW` |
| 515 | `A` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/turn/disk_full_tests.rs` | adopt | `GB-AFBC-RAW` |
| 516 | `M` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/turn_completion_emit_tests.rs` | adopt | `GB-AFBC-RAW` |
| 517 | `M` `crates/codegen/xai-grok-shell/src/session/acp_types.rs` | adopt | `GB-AFBC-RAW` |
| 518 | `M` `crates/codegen/xai-grok-shell/src/session/agent_rebuild.rs` | adopt | `GB-AFBC-RAW` |
| 519 | `M` `crates/codegen/xai-grok-shell/src/session/announcement_state.rs` | adopt | `GB-AFBC-RAW` |
| 520 | `M` `crates/codegen/xai-grok-shell/src/session/chat_persistence.rs` | adopt | `GB-AFBC-RAW` |
| 521 | `M` `crates/codegen/xai-grok-shell/src/session/commands.rs` | adopt | `GB-AFBC-RAW` |
| 522 | `M` `crates/codegen/xai-grok-shell/src/session/compaction.rs` | adopt | `GB-AFBC-RAW` |
| 523 | `M` `crates/codegen/xai-grok-shell/src/session/compaction_config.rs` | adopt | `GB-AFBC-RAW` |
| 524 | `M` `crates/codegen/xai-grok-shell/src/session/events.rs` | adopt | `GB-AFBC-RAW` |
| 525 | `M` `crates/codegen/xai-grok-shell/src/session/export.rs` | adopt | `GB-AFBC-RAW` |
| 526 | `M` `crates/codegen/xai-grok-shell/src/session/feedback.rs` | adopt | `GB-AFBC-RAW` |
| 527 | `M` `crates/codegen/xai-grok-shell/src/session/feedback_manager.rs` | adopt | `GB-AFBC-RAW` |
| 528 | `M` `crates/codegen/xai-grok-shell/src/session/file_system.rs` | adopt | `GB-AFBC-RAW` |
| 529 | `M` `crates/codegen/xai-grok-shell/src/session/fs_watch.rs` | adopt | `GB-AFBC-RAW` |
| 530 | `M` `crates/codegen/xai-grok-shell/src/session/goal_classifier.rs` | adopt | `GB-AFBC-RAW` |
| 531 | `M` `crates/codegen/xai-grok-shell/src/session/goal_evaluator.rs` | adopt | `GB-AFBC-RAW` |
| 532 | `M` `crates/codegen/xai-grok-shell/src/session/goal_tracker.rs` | adopt | `GB-AFBC-RAW` |
| 533 | `M` `crates/codegen/xai-grok-shell/src/session/handle.rs` | adopt | `GB-AFBC-RAW` |
| 534 | `M` `crates/codegen/xai-grok-shell/src/session/helpers/chat.rs` | adopt | `GB-AFBC-RAW` |
| 535 | `M` `crates/codegen/xai-grok-shell/src/session/helpers/full_replace_compaction.rs` | adopt | `GB-AFBC-RAW` |
| 536 | `M` `crates/codegen/xai-grok-shell/src/session/helpers/memory_flush.rs` | adopt | `GB-AFBC-RAW` |
| 537 | `M` `crates/codegen/xai-grok-shell/src/session/helpers/mod.rs` | adopt | `GB-AFBC-RAW` |
| 538 | `M` `crates/codegen/xai-grok-shell/src/session/helpers/session_compact.rs` | adopt | `GB-AFBC-RAW` |
| 539 | `M` `crates/codegen/xai-grok-shell/src/session/helpers/session_recap.rs` | adopt | `GB-AFBC-RAW` |
| 540 | `A` `crates/codegen/xai-grok-shell/src/session/helpers/turn_summary.rs` | adopt | `GB-AFBC-RAW` |
| 541 | `M` `crates/codegen/xai-grok-shell/src/session/image_describe.rs` | adopt | `GB-AFBC-RAW` |
| 542 | `M` `crates/codegen/xai-grok-shell/src/session/image_normalize.rs` | adopt | `GB-AFBC-RAW` |
| 543 | `M` `crates/codegen/xai-grok-shell/src/session/inference_metrics.rs` | adopt | `GB-AFBC-RAW` |
| 544 | `M` `crates/codegen/xai-grok-shell/src/session/managed_mcp.rs` | adopt | `GB-AFBC-RAW` |
| 545 | `M` `crates/codegen/xai-grok-shell/src/session/mcp_dispatcher.rs` | adopt | `GB-AFBC-RAW` |
| 546 | `M` `crates/codegen/xai-grok-shell/src/session/mcp_restart.rs` | adopt | `GB-AFBC-RAW` |
| 547 | `M` `crates/codegen/xai-grok-shell/src/session/mcp_servers.rs` | adopt | `GB-AFBC-RAW` |
| 548 | `M` `crates/codegen/xai-grok-shell/src/session/memory/hooks.rs` | adopt | `GB-AFBC-RAW` |
| 549 | `M` `crates/codegen/xai-grok-shell/src/session/memory_state.rs` | adopt | `GB-AFBC-RAW` |
| 550 | `M` `crates/codegen/xai-grok-shell/src/session/merge.rs` | adopt | `GB-AFBC-RAW` |
| 551 | `M` `crates/codegen/xai-grok-shell/src/session/mod.rs` | adopt | `GB-AFBC-RAW` |
| 552 | `M` `crates/codegen/xai-grok-shell/src/session/normalize_cache.rs` | adopt | `GB-AFBC-RAW` |
| 553 | `M` `crates/codegen/xai-grok-shell/src/session/notifications.rs` | adopt | `GB-AFBC-RAW` |
| 554 | `M` `crates/codegen/xai-grok-shell/src/session/pending_interaction.rs` | adopt | `GB-AFBC-RAW` |
| 555 | `M` `crates/codegen/xai-grok-shell/src/session/persistence.rs` | adopt | `GB-AFBC-RAW` |
| 556 | `M` `crates/codegen/xai-grok-shell/src/session/persistence_tests.rs` | adopt | `GB-AFBC-RAW` |
| 557 | `M` `crates/codegen/xai-grok-shell/src/session/plan_mode.rs` | adopt | `GB-AFBC-RAW` |
| 558 | `M` `crates/codegen/xai-grok-shell/src/session/prompt_history.rs` | adopt | `GB-AFBC-RAW` |
| 559 | `M` `crates/codegen/xai-grok-shell/src/session/prompt_parser.rs` | adopt | `GB-AFBC-RAW` |
| 560 | `M` `crates/codegen/xai-grok-shell/src/session/restore_stub.rs` | adopt | `GB-AFBC-RAW` |
| 561 | `M` `crates/codegen/xai-grok-shell/src/session/result.rs` | adopt | `GB-AFBC-RAW` |
| 562 | `M` `crates/codegen/xai-grok-shell/src/session/signals.rs` | adopt | `GB-AFBC-RAW` |
| 563 | `M` `crates/codegen/xai-grok-shell/src/session/slash_commands.rs` | adopt | `GB-AFBC-RAW` |
| 564 | `A` `crates/codegen/xai-grok-shell/src/session/storage/jsonl/copy.rs` | adopt | `GB-AFBC-RAW` |
| 565 | `A` `crates/codegen/xai-grok-shell/src/session/storage/jsonl/copy_tests.rs` | adopt | `GB-AFBC-RAW` |
| 566 | `M` `crates/codegen/xai-grok-shell/src/session/storage/jsonl/mod.rs` | adopt | `GB-AFBC-RAW` |
| 567 | `M` `crates/codegen/xai-grok-shell/src/session/storage/jsonl/tests.rs` | adopt | `GB-AFBC-RAW` |
| 568 | `M` `crates/codegen/xai-grok-shell/src/session/storage/mod.rs` | adopt | `GB-AFBC-RAW` |
| 569 | `M` `crates/codegen/xai-grok-shell/src/session/storage/search.rs` | adopt | `GB-AFBC-RAW` |
| 570 | `A` `crates/codegen/xai-grok-shell/src/session/storage/search_bootstrap.rs` | adopt | `GB-AFBC-RAW` |
| 571 | `A` `crates/codegen/xai-grok-shell/src/session/storage/search_bootstrap_tests.rs` | adopt | `GB-AFBC-RAW` |
| 572 | `A` `crates/codegen/xai-grok-shell/src/session/storage/search_content.rs` | adopt | `GB-AFBC-RAW` |
| 573 | `A` `crates/codegen/xai-grok-shell/src/session/storage/search_content_tests.rs` | adopt | `GB-AFBC-RAW` |
| 574 | `A` `crates/codegen/xai-grok-shell/src/session/storage/search_db.rs` | adopt | `GB-AFBC-RAW` |
| 575 | `M` `crates/codegen/xai-grok-shell/src/session/storage/search_fts.rs` | adopt | `GB-AFBC-RAW` |
| 576 | `D` `crates/codegen/xai-grok-shell/src/session/storage/search_remote_sync.rs` | adopt | `GB-AFBC-RAW` |
| 577 | `M` `crates/codegen/xai-grok-shell/src/session/storage/summary_write.rs` | adopt | `GB-AFBC-RAW` |
| 578 | `D` `crates/codegen/xai-grok-shell/src/session/telemetry.rs` | adopt | `GB-AFBC-RAW` |
| 579 | `A` `crates/codegen/xai-grok-shell/src/session/telemetry/mod.rs` | adopt | `GB-AFBC-RAW` |
| 580 | `A` `crates/codegen/xai-grok-shell/src/session/telemetry/permission.rs` | adopt | `GB-AFBC-RAW` |
| 581 | `M` `crates/codegen/xai-grok-shell/src/session/tool_index.rs` | adopt | `GB-AFBC-RAW` |
| 582 | `M` `crates/codegen/xai-grok-shell/src/session/unified_list/cursor.rs` | adopt | `GB-AFBC-RAW` |
| 583 | `M` `crates/codegen/xai-grok-shell/src/session/unified_list/facets.rs` | adopt | `GB-AFBC-RAW` |
| 584 | `M` `crates/codegen/xai-grok-shell/src/session/unified_list/mod.rs` | adopt | `GB-AFBC-RAW` |
| 585 | `M` `crates/codegen/xai-grok-shell/src/session/unified_list/row.rs` | adopt | `GB-AFBC-RAW` |
| 586 | `M` `crates/codegen/xai-grok-shell/src/session/user_message.rs` | adopt | `GB-AFBC-RAW` |
| 587 | `M` `crates/codegen/xai-grok-shell/src/session/wire_tags.rs` | adopt | `GB-AFBC-RAW` |
| 588 | `M` `crates/codegen/xai-grok-shell/src/session/workflow/host_service.rs` | adopt | `GB-AFBC-RAW` |
| 589 | `M` `crates/codegen/xai-grok-shell/src/session/workflow/manager.rs` | adopt | `GB-AFBC-RAW` |
| 590 | `M` `crates/codegen/xai-grok-shell/src/session/workflow/tracker.rs` | adopt | `GB-AFBC-RAW` |
| 591 | `M` `crates/codegen/xai-grok-shell/src/session/worktree.rs` | adopt | `GB-AFBC-RAW` |
| 592 | `M` `crates/codegen/xai-grok-shell/src/terminal/adapter.rs` | adopt | `GB-AFBC-RAW` |
| 593 | `M` `crates/codegen/xai-grok-shell/src/terminal/background_task.rs` | adopt | `GB-AFBC-RAW` |
| 594 | `M` `crates/codegen/xai-grok-shell/src/terminal/local_terminal.rs` | adopt | `GB-AFBC-RAW` |
| 595 | `M` `crates/codegen/xai-grok-shell/src/terminal/mod.rs` | adopt | `GB-AFBC-RAW` |
| 596 | `M` `crates/codegen/xai-grok-shell/src/terminal/pty_session.rs` | adopt | `GB-AFBC-RAW` |
| 597 | `M` `crates/codegen/xai-grok-shell/src/terminal/streaming_local_terminal.rs` | adopt | `GB-AFBC-RAW` |
| 598 | `M` `crates/codegen/xai-grok-shell/src/test_support/lsp_runtime.rs` | adopt | `GB-AFBC-RAW` |
| 599 | `M` `crates/codegen/xai-grok-shell/src/test_support/mod.rs` | adopt | `GB-AFBC-RAW` |
| 600 | `M` `crates/codegen/xai-grok-shell/src/tools/config.rs` | adopt | `GB-AFBC-RAW` |
| 601 | `M` `crates/codegen/xai-grok-shell/src/tools/mod.rs` | adopt | `GB-AFBC-RAW` |
| 602 | `M` `crates/codegen/xai-grok-shell/src/tools/notification_bridge.rs` | adopt | `GB-AFBC-RAW` |
| 603 | `A` `crates/codegen/xai-grok-shell/src/tools/task_completed_frame.rs` | adopt | `GB-AFBC-RAW` |
| 604 | `A` `crates/codegen/xai-grok-shell/src/tools/task_completed_frame_tests.rs` | adopt | `GB-AFBC-RAW` |
| 605 | `M` `crates/codegen/xai-grok-shell/src/tools/todo.rs` | adopt | `GB-AFBC-RAW` |
| 606 | `M` `crates/codegen/xai-grok-shell/src/tools/tool_context.rs` | adopt | `GB-AFBC-RAW` |
| 607 | `D` `crates/codegen/xai-grok-shell/src/trace_classifier/mod.rs` | adopt | `GB-AFBC-RAW` |
| 608 | `M` `crates/codegen/xai-grok-shell/src/upload/turn.rs` | adopt | `GB-AFBC-RAW` |
| 609 | `M` `crates/codegen/xai-grok-shell/src/util/config/announcements.rs` | adopt | `GB-AFBC-RAW` |
| 610 | `M` `crates/codegen/xai-grok-shell/src/util/config/campaigns.rs` | adopt | `GB-AFBC-RAW` |
| 611 | `M` `crates/codegen/xai-grok-shell/src/util/config/hints.rs` | adopt | `GB-AFBC-RAW` |
| 612 | `M` `crates/codegen/xai-grok-shell/src/util/config/load.rs` | adopt | `GB-AFBC-RAW` |
| 613 | `M` `crates/codegen/xai-grok-shell/src/util/config/mcp.rs` | adopt | `GB-AFBC-RAW` |
| 614 | `A` `crates/codegen/xai-grok-shell/src/util/config/mcp_reenable.rs` | adopt | `GB-AFBC-RAW` |
| 615 | `M` `crates/codegen/xai-grok-shell/src/util/config/mod.rs` | adopt | `GB-AFBC-RAW` |
| 616 | `M` `crates/codegen/xai-grok-shell/src/util/config/permissions.rs` | adopt | `GB-AFBC-RAW` |
| 617 | `M` `crates/codegen/xai-grok-shell/src/util/config/persist.rs` | adopt | `GB-AFBC-RAW` |
| 618 | `M` `crates/codegen/xai-grok-shell/src/util/config/resolve/auto_mode.rs` | adopt | `GB-AFBC-RAW` |
| 619 | `M` `crates/codegen/xai-grok-shell/src/util/config/resolve/compaction.rs` | adopt | `GB-AFBC-RAW` |
| 620 | `M` `crates/codegen/xai-grok-shell/src/util/config/resolve/crash_handler.rs` | adopt | `GB-AFBC-RAW` |
| 621 | `M` `crates/codegen/xai-grok-shell/src/util/config/resolve/display_refresh.rs` | adopt | `GB-AFBC-RAW` |
| 622 | `M` `crates/codegen/xai-grok-shell/src/util/config/resolve/mcp.rs` | adopt | `GB-AFBC-RAW` |
| 623 | `M` `crates/codegen/xai-grok-shell/src/util/config/resolve/system_prompt.rs` | adopt | `GB-AFBC-RAW` |
| 624 | `M` `crates/codegen/xai-grok-shell/src/util/config/resolve/tool_approvals.rs` | adopt | `GB-AFBC-RAW` |
| 625 | `M` `crates/codegen/xai-grok-shell/src/util/config/resolve/toolset.rs` | adopt | `GB-AFBC-RAW` |
| 626 | `M` `crates/codegen/xai-grok-shell/src/util/config/resolve/version.rs` | adopt | `GB-AFBC-RAW` |
| 627 | `M` `crates/codegen/xai-grok-shell/src/util/config/settings_writes.rs` | adopt | `GB-AFBC-RAW` |
| 628 | `M` `crates/codegen/xai-grok-shell/src/util/config/tips.rs` | adopt | `GB-AFBC-RAW` |
| 629 | `M` `crates/codegen/xai-grok-shell/src/util/config/worktree.rs` | adopt | `GB-AFBC-RAW` |
| 630 | `A` `crates/codegen/xai-grok-shell/src/util/dual_clock.rs` | adopt | `GB-AFBC-RAW` |
| 631 | `A` `crates/codegen/xai-grok-shell/src/util/dual_clock_tests.rs` | adopt | `GB-AFBC-RAW` |
| 632 | `M` `crates/codegen/xai-grok-shell/src/util/hooks.rs` | adopt | `GB-AFBC-RAW` |
| 633 | `M` `crates/codegen/xai-grok-shell/src/util/limits.rs` | adopt | `GB-AFBC-RAW` |
| 634 | `M` `crates/codegen/xai-grok-shell/src/util/mod.rs` | adopt | `GB-AFBC-RAW` |
| 635 | `A` `crates/codegen/xai-grok-shell/tests/acp_harness/mod.rs` | adopt | `GB-AFBC-RAW` |
| 636 | `A` `crates/codegen/xai-grok-shell/tests/acp_session_setup_wire.rs` | adopt | `GB-AFBC-RAW` |
| 637 | `A` `crates/codegen/xai-grok-shell/tests/external_auth_conforming_provider.rs` | adopt | `GB-AFBC-RAW` |
| 638 | `A` `crates/codegen/xai-grok-shell/tests/external_auth_expired_credential.rs` | adopt | `GB-AFBC-RAW` |
| 639 | `A` `crates/codegen/xai-grok-shell/tests/test_fork_copy_memory.rs` | adopt | `GB-AFBC-RAW` |
| 640 | `M` `crates/codegen/xai-grok-shell/tests/test_leader_soak.rs` | adopt | `GB-AFBC-RAW` |
| 641 | `M` `crates/codegen/xai-grok-shell/tests/test_leader_stdio_integration.rs` | adopt | `GB-AFBC-RAW` |
| 642 | `M` `crates/codegen/xai-grok-shell/tests/test_mcp_permission_persistence.rs` | adopt | `GB-AFBC-RAW` |
| 643 | `M` `crates/codegen/xai-grok-shell/tests/test_registry_churn.rs` | adopt | `GB-AFBC-RAW` |
| 644 | `M` `crates/codegen/xai-grok-shell/tests/test_sampling_client.rs` | adopt | `GB-AFBC-RAW` |
| 645 | `M` `crates/codegen/xai-grok-shell/tests/test_session_load_memory.rs` | adopt | `GB-AFBC-RAW` |
| 646 | `M` `crates/codegen/xai-grok-telemetry/Cargo.toml` | adopt | `GB-AFBC-RAW` |
| 647 | `D` `crates/codegen/xai-grok-telemetry/src/events.rs` | adopt | `GB-AFBC-RAW` |
| 648 | `A` `crates/codegen/xai-grok-telemetry/src/events/mod.rs` | adopt | `GB-AFBC-RAW` |
| 649 | `A` `crates/codegen/xai-grok-telemetry/src/events/permission_analytics.rs` | adopt | `GB-AFBC-RAW` |
| 650 | `M` `crates/codegen/xai-grok-telemetry/src/external/config.rs` | adopt | `GB-AFBC-RAW` |
| 651 | `M` `crates/codegen/xai-grok-telemetry/src/external/emit.rs` | adopt | `GB-AFBC-RAW` |
| 652 | `M` `crates/codegen/xai-grok-telemetry/src/external/mod.rs` | adopt | `GB-AFBC-RAW` |
| 653 | `M` `crates/codegen/xai-grok-telemetry/src/external/providers.rs` | adopt | `GB-AFBC-RAW` |
| 654 | `M` `crates/codegen/xai-grok-telemetry/src/external/schema.rs` | adopt | `GB-AFBC-RAW` |
| 655 | `M` `crates/codegen/xai-grok-telemetry/src/external/tests.rs` | adopt | `GB-AFBC-RAW` |
| 656 | `M` `crates/codegen/xai-grok-telemetry/src/instrumentation.rs` | adopt | `GB-AFBC-RAW` |
| 657 | `M` `crates/codegen/xai-grok-telemetry/src/lib.rs` | adopt | `GB-AFBC-RAW` |
| 658 | `M` `crates/codegen/xai-grok-telemetry/src/otel_layer/mod.rs` | adopt | `GB-AFBC-RAW` |
| 659 | `M` `crates/codegen/xai-grok-telemetry/src/otlp_http.rs` | adopt | `GB-AFBC-RAW` |
| 660 | `M` `crates/codegen/xai-grok-telemetry/src/session_ctx.rs` | adopt | `GB-AFBC-RAW` |
| 661 | `A` `crates/codegen/xai-grok-telemetry/src/startup.rs` | adopt | `GB-AFBC-RAW` |
| 662 | `M` `crates/codegen/xai-grok-telemetry/src/unified_log.rs` | adopt | `GB-AFBC-RAW` |
| 663 | `A` `crates/codegen/xai-grok-telemetry/tests/external_otlp_grpc_tls.rs` | adopt | `GB-AFBC-RAW` |
| 664 | `M` `crates/codegen/xai-grok-telemetry/tests/otlp_collector/mod.rs` | adopt | `GB-AFBC-RAW` |
| 665 | `M` `crates/codegen/xai-grok-test-support/src/lib.rs` | adopt | `GB-AFBC-RAW` |
| 666 | `M` `crates/codegen/xai-grok-test-support/src/mock_server.rs` | adopt | `GB-AFBC-RAW` |
| 667 | `M` `crates/codegen/xai-grok-test-support/src/resources.rs` | adopt | `GB-AFBC-RAW` |
| 668 | `M` `crates/codegen/xai-grok-test-support/src/sandbox.rs` | adopt | `GB-AFBC-RAW` |
| 669 | `M` `crates/codegen/xai-grok-tools-api/proto/grok-tools.proto` | adopt | `GB-AFBC-RAW` |
| 670 | `M` `crates/codegen/xai-grok-tools/Cargo.toml` | adopt | `GB-AFBC-RAW` |
| 671 | `M` `crates/codegen/xai-grok-tools/build.rs` | adopt | `GB-AFBC-RAW` |
| 672 | `M` `crates/codegen/xai-grok-tools/src/attribution.rs` | adopt | `GB-AFBC-RAW` |
| 673 | `M` `crates/codegen/xai-grok-tools/src/bridge.rs` | adopt | `GB-AFBC-RAW` |
| 674 | `A` `crates/codegen/xai-grok-tools/src/computer/local/lifecycle.rs` | adopt | `GB-AFBC-RAW` |
| 675 | `M` `crates/codegen/xai-grok-tools/src/computer/local/mod.rs` | adopt | `GB-AFBC-RAW` |
| 676 | `M` `crates/codegen/xai-grok-tools/src/computer/local/terminal.rs` | adopt | `GB-AFBC-RAW` |
| 677 | `A` `crates/codegen/xai-grok-tools/src/computer/local/terminal_snapshot_tests.rs` | adopt | `GB-AFBC-RAW` |
| 678 | `M` `crates/codegen/xai-grok-tools/src/computer/mod.rs` | adopt | `GB-AFBC-RAW` |
| 679 | `A` `crates/codegen/xai-grok-tools/src/computer/task_log.rs` | adopt | `GB-AFBC-RAW` |
| 680 | `A` `crates/codegen/xai-grok-tools/src/computer/task_log_tests.rs` | adopt | `GB-AFBC-RAW` |
| 681 | `M` `crates/codegen/xai-grok-tools/src/computer/types.rs` | adopt | `GB-AFBC-RAW` |
| 682 | `M` `crates/codegen/xai-grok-tools/src/implementations/codex/apply_patch/apply.rs` | adopt | `GB-AFBC-RAW` |
| 683 | `M` `crates/codegen/xai-grok-tools/src/implementations/codex/apply_patch/tool.rs` | adopt | `GB-AFBC-RAW` |
| 684 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/bash/mod.rs` | adopt | `GB-AFBC-RAW` |
| 685 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/grep/mod.rs` | adopt | `GB-AFBC-RAW` |
| 686 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/scheduler/actor.rs` | adopt | `GB-AFBC-RAW` |
| 687 | `A` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/admission.rs` | adopt | `GB-AFBC-RAW` |
| 688 | `A` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/admission_tests.rs` | adopt | `GB-AFBC-RAW` |
| 689 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/backend.rs` | adopt | `GB-AFBC-RAW` |
| 690 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/coordinator.rs` | adopt | `GB-AFBC-RAW` |
| 691 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/coordinator/query.rs` | adopt | `GB-AFBC-RAW` |
| 692 | `A` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/coordinator/queue.rs` | adopt | `GB-AFBC-RAW` |
| 693 | `A` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/coordinator/spawn.rs` | adopt | `GB-AFBC-RAW` |
| 694 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/coordinator_state.rs` | adopt | `GB-AFBC-RAW` |
| 695 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/coordinator_tests.rs` | adopt | `GB-AFBC-RAW` |
| 696 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs` | adopt | `GB-AFBC-RAW` |
| 697 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/types.rs` | adopt | `GB-AFBC-RAW` |
| 698 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task_output/mod.rs` | adopt | `GB-AFBC-RAW` |
| 699 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task_output/terminal_command.rs` | adopt | `GB-AFBC-RAW` |
| 700 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task_output/wait_tasks.rs` | adopt | `GB-AFBC-RAW` |
| 701 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/todo/mod.rs` | adopt | `GB-AFBC-RAW` |
| 702 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/update_goal/mod.rs` | adopt | `GB-AFBC-RAW` |
| 703 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build/workflow/mod.rs` | adopt | `GB-AFBC-RAW` |
| 704 | `M` `crates/codegen/xai-grok-tools/src/implementations/grok_build_concise/bash.rs` | adopt | `GB-AFBC-RAW` |
| 705 | `M` `crates/codegen/xai-grok-tools/src/implementations/opencode/glob/mod.rs` | adopt | `GB-AFBC-RAW` |
| 706 | `M` `crates/codegen/xai-grok-tools/src/implementations/opencode/grep/mod.rs` | adopt | `GB-AFBC-RAW` |
| 707 | `M` `crates/codegen/xai-grok-tools/src/implementations/opencode/todowrite/mod.rs` | adopt | `GB-AFBC-RAW` |
| 708 | `M` `crates/codegen/xai-grok-tools/src/implementations/search_tool/mod.rs` | adopt | `GB-AFBC-RAW` |
| 709 | `M` `crates/codegen/xai-grok-tools/src/implementations/skills/skill.rs` | adopt | `GB-AFBC-RAW` |
| 710 | `M` `crates/codegen/xai-grok-tools/src/implementations/task_output/tool.rs` | adopt | `GB-AFBC-RAW` |
| 711 | `M` `crates/codegen/xai-grok-tools/src/implementations/use_tool/mod.rs` | adopt | `GB-AFBC-RAW` |
| 712 | `M` `crates/codegen/xai-grok-tools/src/implementations/web_search/client.rs` | adopt | `GB-AFBC-RAW` |
| 713 | `M` `crates/codegen/xai-grok-tools/src/lib.rs` | adopt | `GB-AFBC-RAW` |
| 714 | `M` `crates/codegen/xai-grok-tools/src/registry/types.rs` | adopt | `GB-AFBC-RAW` |
| 715 | `M` `crates/codegen/xai-grok-tools/src/reminders/task_completion.rs` | adopt | `GB-AFBC-RAW` |
| 716 | `M` `crates/codegen/xai-grok-tools/src/types/context.rs` | adopt | `GB-AFBC-RAW` |
| 717 | `M` `crates/codegen/xai-grok-tools/src/types/output.rs` | adopt | `GB-AFBC-RAW` |
| 718 | `M` `crates/codegen/xai-grok-tools/src/util/base64_images.rs` | adopt | `GB-AFBC-RAW` |
| 719 | `M` `crates/codegen/xai-grok-tools/src/util/env.rs` | adopt | `GB-AFBC-RAW` |
| 720 | `M` `crates/codegen/xai-grok-tools/src/util/mcp_truncate.rs` | adopt | `GB-AFBC-RAW` |
| 721 | `M` `crates/codegen/xai-grok-tools/src/util/mod.rs` | adopt | `GB-AFBC-RAW` |
| 722 | `M` `crates/codegen/xai-grok-tools/src/util/truncate.rs` | adopt | `GB-AFBC-RAW` |
| 723 | `A` `crates/codegen/xai-grok-tools/tests/browser_tab_chrome_e2e.rs` | not applicable | `GB-AFBC-007` |
| 724 | `M` `crates/codegen/xai-grok-tools/tests/test_subagent_soak.rs` | adopt | `GB-AFBC-RAW` |
| 725 | `M` `crates/codegen/xai-grok-update/src/auto_update.rs` | adopt | `GB-AFBC-RAW` |
| 726 | `M` `crates/codegen/xai-grok-update/tests/test_blitz_cancel.rs` | adopt | `GB-AFBC-RAW` |
| 727 | `M` `crates/codegen/xai-grok-update/tests/test_install_internal.rs` | adopt | `GB-AFBC-RAW` |
| 728 | `M` `crates/codegen/xai-grok-version/Cargo.toml` | not applicable | `GB-A422-028` |
| 729 | `M` `crates/codegen/xai-grok-workspace-types/src/rpc/envelope.rs` | adopt | `GB-AFBC-RAW` |
| 730 | `A` `crates/codegen/xai-grok-workspace-types/src/rpc/export.rs` | adopt | `GB-AFBC-RAW` |
| 731 | `M` `crates/codegen/xai-grok-workspace-types/src/rpc/git.rs` | adopt | `GB-AFBC-RAW` |
| 732 | `M` `crates/codegen/xai-grok-workspace-types/src/rpc/mod.rs` | adopt | `GB-AFBC-RAW` |
| 733 | `A` `crates/codegen/xai-grok-workspace-types/src/rpc/repos.rs` | adopt | `GB-AFBC-RAW` |
| 734 | `M` `crates/codegen/xai-grok-workspace/Cargo.toml` | adopt | `GB-AFBC-RAW` |
| 735 | `M` `crates/codegen/xai-grok-workspace/src/activity.rs` | adopt | `GB-AFBC-RAW` |
| 736 | `M` `crates/codegen/xai-grok-workspace/src/diag_server.rs` | adopt | `GB-AFBC-RAW` |
| 737 | `M` `crates/codegen/xai-grok-workspace/src/error.rs` | adopt | `GB-AFBC-RAW` |
| 738 | `M` `crates/codegen/xai-grok-workspace/src/file_system/git_status.rs` | adopt | `GB-AFBC-RAW` |
| 739 | `M` `crates/codegen/xai-grok-workspace/src/folder_trust.rs` | adopt | `GB-AFBC-RAW` |
| 740 | `A` `crates/codegen/xai-grok-workspace/src/git_odb.rs` | adopt | `GB-AFBC-RAW` |
| 741 | `M` `crates/codegen/xai-grok-workspace/src/handle.rs` | adopt | `GB-AFBC-RAW` |
| 742 | `M` `crates/codegen/xai-grok-workspace/src/hub.rs` | adopt | `GB-AFBC-RAW` |
| 743 | `M` `crates/codegen/xai-grok-workspace/src/hub_server.rs` | adopt | `GB-AFBC-RAW` |
| 744 | `M` `crates/codegen/xai-grok-workspace/src/lib.rs` | adopt | `GB-AFBC-RAW` |
| 745 | `D` `crates/codegen/xai-grok-workspace/src/permission/auto_mode.rs` | adopt | `GB-AFBC-RAW` |
| 746 | `A` `crates/codegen/xai-grok-workspace/src/permission/auto_mode/mod.rs` | adopt | `GB-AFBC-RAW` |
| 747 | `A` `crates/codegen/xai-grok-workspace/src/permission/auto_mode/security_findings.rs` | adopt | `GB-AFBC-RAW` |
| 748 | `A` `crates/codegen/xai-grok-workspace/src/permission/auto_mode/security_findings_tests.rs` | adopt | `GB-AFBC-RAW` |
| 749 | `M` `crates/codegen/xai-grok-workspace/src/permission/bash_command_splitting.rs` | adopt | `GB-AFBC-RAW` |
| 750 | `M` `crates/codegen/xai-grok-workspace/src/permission/exec_risk.rs` | adopt | `GB-AFBC-RAW` |
| 751 | `M` `crates/codegen/xai-grok-workspace/src/permission/gate_preflight.rs` | adopt | `GB-AFBC-RAW` |
| 752 | `M` `crates/codegen/xai-grok-workspace/src/permission/hub_permission.rs` | adopt | `GB-AFBC-RAW` |
| 753 | `D` `crates/codegen/xai-grok-workspace/src/permission/manager.rs` | adopt | `GB-AFBC-RAW` |
| 754 | `A` `crates/codegen/xai-grok-workspace/src/permission/manager/mod.rs` | adopt | `GB-AFBC-RAW` |
| 755 | `A` `crates/codegen/xai-grok-workspace/src/permission/manager/reasons.rs` | adopt | `GB-AFBC-RAW` |
| 756 | `A` `crates/codegen/xai-grok-workspace/src/permission/manager/request_classification.rs` | adopt | `GB-AFBC-RAW` |
| 757 | `M` `crates/codegen/xai-grok-workspace/src/permission/mod.rs` | adopt | `GB-AFBC-RAW` |
| 758 | `M` `crates/codegen/xai-grok-workspace/src/permission/policy.rs` | adopt | `GB-AFBC-RAW` |
| 759 | `M` `crates/codegen/xai-grok-workspace/src/permission/prompter.rs` | adopt | `GB-AFBC-RAW` |
| 760 | `M` `crates/codegen/xai-grok-workspace/src/permission/resolution.rs` | adopt | `GB-AFBC-RAW` |
| 761 | `M` `crates/codegen/xai-grok-workspace/src/permission/rules.rs` | adopt | `GB-AFBC-RAW` |
| 762 | `M` `crates/codegen/xai-grok-workspace/src/permission/shell_access.rs` | adopt | `GB-AFBC-RAW` |
| 763 | `M` `crates/codegen/xai-grok-workspace/src/permission/state.rs` | adopt | `GB-AFBC-RAW` |
| 764 | `M` `crates/codegen/xai-grok-workspace/src/permission/types.rs` | adopt | `GB-AFBC-RAW` |
| 765 | `M` `crates/codegen/xai-grok-workspace/src/preview_supervisor.rs` | adopt | `GB-AFBC-RAW` |
| 766 | `A` `crates/codegen/xai-grok-workspace/src/restore_fetch.rs` | adopt | `GB-AFBC-RAW` |
| 767 | `A` `crates/codegen/xai-grok-workspace/src/restore_fetch_tests.rs` | adopt | `GB-AFBC-RAW` |
| 768 | `M` `crates/codegen/xai-grok-workspace/src/rpc_envelope.rs` | adopt | `GB-AFBC-RAW` |
| 769 | `M` `crates/codegen/xai-grok-workspace/src/session/git.rs` | adopt | `GB-AFBC-RAW` |
| 770 | `A` `crates/codegen/xai-grok-workspace/src/session/git_gate.rs` | adopt | `GB-AFBC-RAW` |
| 771 | `A` `crates/codegen/xai-grok-workspace/src/session/git_gate_tests.rs` | adopt | `GB-AFBC-RAW` |
| 772 | `M` `crates/codegen/xai-grok-workspace/src/session/mod.rs` | adopt | `GB-AFBC-RAW` |
| 773 | `M` `crates/codegen/xai-grok-workspace/src/workspace_ops.rs` | adopt | `GB-AFBC-RAW` |
| 774 | `M` `crates/codegen/xai-grok-workspace/src/worktree/mod.rs` | adopt | `GB-AFBC-RAW` |
| 775 | `M` `crates/codegen/xai-ratatui-textarea/examples/textarea_demo.rs` | adopt | `GB-AFBC-RAW` |
| 776 | `M` `crates/codegen/xai-sqlite-journal/src/lib.rs` | adopt | `GB-AFBC-RAW` |
| 777 | `M` `crates/codegen/xai-token-estimation/src/lib.rs` | adopt | `GB-AFBC-RAW` |
| 778 | `M` `crates/codegen/xai-tty-utils/Cargo.toml` | adopt | `GB-AFBC-RAW` |
| 779 | `M` `crates/codegen/xai-tty-utils/src/lib.rs` | adopt | `GB-AFBC-RAW` |
| 780 | `A` `crates/codegen/xai-tty-utils/src/process_resources.rs` | adopt | `GB-AFBC-RAW` |
| 781 | `M` `crates/common/xai-circuit-breaker/src/retry_policy.rs` | adopt | `GB-AFBC-RAW` |
| 782 | `M` `crates/common/xai-computer-hub-sdk/src/connection.rs` | adopt | `GB-AFBC-RAW` |
| 783 | `M` `crates/common/xai-computer-hub-sdk/src/connection_borrow.rs` | adopt | `GB-AFBC-RAW` |
| 784 | `M` `crates/common/xai-computer-hub-sdk/src/harness.rs` | adopt | `GB-AFBC-RAW` |
| 785 | `M` `crates/common/xai-computer-hub-sdk/src/metrics.rs` | adopt | `GB-AFBC-RAW` |
| 786 | `M` `crates/common/xai-computer-hub-sdk/src/pool.rs` | adopt | `GB-AFBC-RAW` |
| 787 | `M` `crates/common/xai-computer-hub-sdk/src/server.rs` | adopt | `GB-AFBC-RAW` |
| 788 | `M` `crates/common/xai-grok-compaction/src/code_compaction/failure.rs` | adopt | `GB-AFBC-RAW` |
| 789 | `M` `crates/common/xai-grok-compaction/src/code_compaction/sample.rs` | adopt | `GB-AFBC-RAW` |
| 790 | `M` `crates/common/xai-grok-compaction/src/reminder.rs` | adopt | `GB-AFBC-RAW` |
| 791 | `M` `crates/common/xai-tool-protocol/src/frames.rs` | adopt | `GB-AFBC-RAW` |
| 792 | `M` `crates/common/xai-tool-protocol/src/lib.rs` | adopt | `GB-AFBC-RAW` |
| 793 | `M` `crates/common/xai-tool-runtime/src/notification.rs` | adopt | `GB-AFBC-RAW` |
| 794 | `M` `crates/common/xai-tool-runtime/tests/notification_serde.rs` | adopt | `GB-AFBC-RAW` |
| 795 | `M` `crates/common/xai-tool-types/src/lib.rs` | adopt | `GB-AFBC-RAW` |
| 796 | `M` `crates/common/xai-tool-types/src/task.rs` | adopt | `GB-AFBC-RAW` |
| 797 | `M` `prod/mc/cli-chat-proxy-types/src/team_managed_config_types.rs` | adopt | `GB-AFBC-RAW` |
| 798 | `M` `rust-toolchain.toml` | adopt | `GB-AFBC-RAW` |
