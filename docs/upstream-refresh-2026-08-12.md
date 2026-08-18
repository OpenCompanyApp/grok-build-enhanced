# Release and upstream refresh audit — 2026-08-12

This audit verifies the published Homebrew version and extends the August 11
refresh to source heads pinned on August 12. It does not advance any Reviewed
revision, acknowledge an upstream snapshot, or publish a new release.

The audit boundary is Enhanced commit
`e9f43a0641b437e865c851c6b9012d0f1936e94b`, tree
`5c4f1ea0758cc98ab96debc113df5ea3e35f7b8c`, isolated on
`refresh/upstreams-20260812`. The separate August 9 adoption worktree remains
uncommitted and was not modified.

## Published release and Homebrew audit

- GitHub's public latest stable release is `v0.3.9`, published
  2026-08-08, neither draft nor prerelease.
- Its provenance binds the four native assets to source commit
  `7b00c4e924eb65448a0679954a62310759a152fe`.
- `origin/main`, `v0.3.9`, and local post-release commit `d971891` have the
  same application tree `57400debec4fbcd8fdb4ce869de29694667563be`.
  Therefore the settings RefCell race fix is already in the published binary
  even though the equivalent local commit has a different object identity.
- The release contains exactly four native binaries, `SHA256SUMS`, and
  `RELEASE-PROVENANCE.json`. Every binary URL returned HTTP 206 for a range
  request, all formula hashes match `SHA256SUMS`, and GitHub reports one
  attestation for each binary and the provenance file.
- `OpenCompanyApp/homebrew-tap` commit
  `f175a702dfb1eaaef392dc771b9e67732e97bc1c` publishes formula version
  `0.3.9` for all four native targets. Its push-triggered brew test-bot run
  31283264107 completed successfully.
- This Linux host has no `brew` executable, so a fresh local
  `brew update/upgrade/test` could not be repeated. The public tap formula,
  asset availability, hashes, attestations, and latest successful tap workflow
  provide the remote publication evidence.

No Homebrew mutation or republish was needed: the tap is already aligned with
the latest eligible release.

## Newly pinned heads

Every successful new head is a descendant of the August 11 pin.

| Source | August 11 pin | August 12 pin | Tree | Delta |
| --- | --- | --- | --- | --- |
| Grok Build | `b13fa526f5112c0b20dad5f1f2300d3d3b127895` | `be713136d2a69080743a3f6b3c72077057e5948f` | `ee8039b440cd38a62c1e007b5cd55c6bbe366aa4` | 1 commit / 211 paths |
| OpenAI Codex | `f2a6f2585c327251e6be647e47a3ba3e127ccff3` | `d6eefb26a6d3f610372a4ea4b8a59a2e382c731f` | `9423b14d3d43e689c6b36f3ae0a165066d2937ad` | 50 commits / 345 paths |
| OpenCode | `0d927ba03f36d7f87e3cdb2b6c1f34c44913a099` | `39fb919a054190498f6d5b7985bde231f93ad7a6` | `8cdf9b30e01c8957a94dc5001f94287fdf4a00fc` | 20 commits / 43 paths |
| Oh My Pi | `d3b22a0db6a4a0e2ef272a880e38286e0c466dc9` | `06aecdd51f07e689e970ceaa180abe2be0c14bbb` | `83c6d8c700438bc4746a0c4fa170932b785f3aa0` | 53 commits / 332 paths |
| Kimi Code | `619564dcf9ee10a3cfbf7ecbc764c6b9b63fc91b` | `719da946480261dc5b4228f522f24973d3b5bd68` | `83b1e9ca72fb8f5746d5cc6c3a4143eda9872c06` | 19 commits / 381 paths |
| CodexBar | `e5528d452d4f82cbd7e327246b9044e9c51d64e1` | `fc57a317cee4a8f84962c62c45e4502085f6fc79` | `08f0d371d5b83f36e54b88aafb2af7fbac6fe1a5` | 36 commits / 45 paths |
| models.dev | `1d0f9ba5a49e916ff2dc97b23fbc76820ab258b3` | `40058d7627db5900ffe15c2a2533d55dff52b667` | `e080d69e956a6f925610a4db005559af4da2242f` | 64 commits / 206 paths |

OpenCode Codex auth, Warp themes, Kimi CLI, Z.AI coding plugins, GLM-5, the
Z.AI usage helper, and Exa were unchanged. The tracked Z.AI SDK repository
still returns `Repository not found`; its immutable recorded pin remains
unchanged and no replacement identity was inferred.

## Recent refresh reconciliation

The August 9 worktree contains 153 staged files, 10 additional unstaged files,
and one untracked ledger. It claims closure of the sixteen `8a14c91`
behaviors, but it has not been committed or finally validated. It remains
candidate implementation evidence only.

The August 11 commit correctly carried 15 open obligations. This audit found
one omission in its behavior inventory: the upstream 1.0.1 changelog explicitly
declares two breaking behaviors that the raw paths had only classified under
generic integration:

- `/rewind` truncates conversation history only and asks for confirmation by
  default; and
- managed MCP servers are available only through the gateway catalog.

They are now explicit in `GB-BE71-REWIND-MCP`. No earlier obligation was
silently removed. The active queue now contains 49 stable records, of which 32
are open and 26 are open Grok adoption obligations.

## Grok behavior inventory

The incremental `b13fa526..be713136` snapshot adds or consolidates these
preserved-surface families:

- bounded subagent admission and authoritative read-only tool metadata;
- `grok du`, standalone worktree copy, and bounded large-repository Git work;
- literal sandbox grant normalization, notebook-rule filtering, and
  caller-scoped bundled skills;
- usage/session/context modals and memory-bearing trace exports;
- reference-video options and explicit ZDR diagnostics;
- channel-correct, non-blocking, native-architecture updater behavior, adapted
  only to fork-owned release routes;
- qualified same-name skill suggestions without transient flashes;
- bounded goal evaluation and process teardown;
- recap language, skill/envrc/session deletion, complete replay, and history
  search lifecycle fixes;
- notification hooks only for genuine user-attention waits;
- mid-turn steering, goal Send Now, and cancel-panel keep-running semantics;
- scrollback release recovery and selection lifecycle;
- actionable startup failures plus compatible workspace presence/status; and
- structured sampling errors and retry behavior behind provider boundaries.

`SOURCE_REV`, upstream packaging, and official distribution routes remain not
applicable. Every other raw path is mapped below to an open adoption obligation.
There is no eligible Grok acknowledgement.

## Provider and reference audit

| Source | Finding | Outcome |
| --- | --- | --- |
| OpenAI Codex | New direct-adapter-relevant behavior is reserved `root_turn_id` propagation and preservation of harness metadata through history/compaction. All Responses requests now use `store=false`, already true locally. Workload identity is a separate enterprise auth route. | Two new open obligations, one already-equivalent record, and scoped architecture exclusions. |
| OpenCode | Console usage, retry jitter, Copilot PDF support, generated catalogs, and unrelated providers changed; Codex OAuth and Responses wire did not. | not applicable |
| Kimi Code | SDK-internal retries are disabled so cancellation and bounded retry remain engine-owned. Enhanced already has no SDK retry layer; its sampler actor owns retry and parses provider retry hints. Other changes are replacement-agent/TUI/KAP architecture. | already equivalent for retry ownership; remainder not applicable |
| Oh My Pi | External-scratchpad reasoning-off controls, generic Responses chaining, tar handling, and replacement-harness work changed. No direct ChatGPT subscription requirement is established. | non-normative / not applicable |
| CodexBar | Codex fork-cost cache, Azure budget, UI, and release changes do not alter Z.AI research scope. | not applicable |
| models.dev | Third-party catalogs and pricing changed, including Grok 4.6 and Kimi records. | not applicable; authenticated first-party catalogs remain authoritative |

## Publication decision

A new release is not eligible. The latest application tree remains correctly
published as Homebrew `0.3.9`, while the newest audit has 32 open obligations,
including uncommitted August 9 implementation and newly pinned Grok/Codex
behavior. Reviewed pins were not advanced, no upstream acknowledgement was
created, and no tag, release, tap commit, or push was performed.

## Exhaustive incremental Grok raw-path ledger

The canonical `git diff-tree --raw -r --no-renames --no-abbrev` stream from
tree `0f26f4082a3b9602ec712b218e177626b2bf72e5` to
`ee8039b440cd38a62c1e007b5cd55c6bbe366aa4` has SHA-256
`6c97524fd584dd4873bf7e5df675599050c25a531b27b4541a50f1eecc614275`.
All 211 paths are classified exactly once.

| Row | Raw status, modes and path | Outcome | Obligation |
| ---: | --- | --- | --- |
| 1 | `M` `100644->100644` `Cargo.lock` | temporarily deferred | `GB-B13-INTEGRATION` |
| 2 | `M` `100644->100644` `SOURCE_REV` | not applicable | `GB-BE71-RELEASE` |
| 3 | `M` `100644->100644` `crates/codegen/xai-fast-worktree/src/copy/gitdir.rs` | temporarily deferred | `GB-BE71-DU-GIT` |
| 4 | `M` `100644->100644` `crates/codegen/xai-fast-worktree/src/copy/mod.rs` | temporarily deferred | `GB-BE71-DU-GIT` |
| 5 | `A` `000000->100644` `crates/codegen/xai-fast-worktree/src/copy/standalone.rs` | temporarily deferred | `GB-BE71-DU-GIT` |
| 6 | `A` `000000->100644` `crates/codegen/xai-fast-worktree/src/copy/standalone_tests.rs` | temporarily deferred | `GB-BE71-DU-GIT` |
| 7 | `M` `100644->100644` `crates/codegen/xai-fast-worktree/src/worktree/execute.rs` | temporarily deferred | `GB-BE71-DU-GIT` |
| 8 | `M` `100644->100644` `crates/codegen/xai-fast-worktree/src/worktree/mod.rs` | temporarily deferred | `GB-BE71-DU-GIT` |
| 9 | `M` `100644->100644` `crates/codegen/xai-file-utils/src/queue.rs` | temporarily deferred | `GB-BE71-SUBAGENT-TOOLS` |
| 10 | `M` `100644->100644` `crates/codegen/xai-grok-agent/src/config.rs` | temporarily deferred | `GB-BE71-SANDBOX-SKILLS` |
| 11 | `M` `100644->100644` `crates/codegen/xai-grok-agent/src/prompt/context.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 12 | `M` `100644->100644` `crates/codegen/xai-grok-agent/src/prompt/prompt_encrypted.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 13 | `M` `100644->100644` `crates/codegen/xai-grok-agent/src/prompt/template.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 14 | `M` `100644->100644` `crates/codegen/xai-grok-agent/templates/prompt.md` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 15 | `M` `100644->100644` `crates/codegen/xai-grok-agent/templates/subagent_prompt.md` | temporarily deferred | `GB-BE71-SUBAGENT-TOOLS` |
| 16 | `M` `100644->100644` `crates/codegen/xai-grok-config/src/lib.rs` | temporarily deferred | `GB-BE71-SANDBOX-SKILLS` |
| 17 | `M` `100644->100644` `crates/codegen/xai-grok-config/src/paths.rs` | temporarily deferred | `GB-BE71-SANDBOX-SKILLS` |
| 18 | `M` `100644->100644` `crates/codegen/xai-grok-pager-bin/Cargo.toml` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 19 | `M` `100644->100644` `crates/codegen/xai-grok-pager-bin/src/main.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 20 | `A` `000000->100644` `crates/codegen/xai-grok-pager-bin/tests/update_never_blocked_by_config.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 21 | `M` `100644->100644` `crates/codegen/xai-grok-pager-pty-harness/src/scenarios/empty_enter_send_now.rs` | temporarily deferred | `GB-BE71-STEERING` |
| 22 | `M` `100644->100644` `crates/codegen/xai-grok-pager-render/src/terminal/mod.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 23 | `M` `100644->100644` `crates/codegen/xai-grok-pager/Cargo.toml` | temporarily deferred | `GB-B13-INTEGRATION` |
| 24 | `M` `100644->100644` `crates/codegen/xai-grok-pager/docs/user-guide/03-keyboard-shortcuts.md` | temporarily deferred | `GB-B13-INTEGRATION` |
| 25 | `M` `100644->100644` `crates/codegen/xai-grok-pager/docs/user-guide/10-hooks.md` | temporarily deferred | `GB-BE71-NOTIFICATIONS` |
| 26 | `M` `100644->100644` `crates/codegen/xai-grok-pager/docs/user-guide/18-sandbox.md` | temporarily deferred | `GB-BE71-SANDBOX-SKILLS` |
| 27 | `M` `100644->100644` `crates/codegen/xai-grok-pager/docs/user-guide/24-monitoring-usage.md` | temporarily deferred | `GB-B13-INTEGRATION` |
| 28 | `M` `100755->100755` `crates/codegen/xai-grok-pager/scripts/install-enterprise.sh` | temporarily deferred | `GB-BE71-UPDATER` |
| 29 | `M` `100755->100755` `crates/codegen/xai-grok-pager/scripts/install.sh` | temporarily deferred | `GB-BE71-UPDATER` |
| 30 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/acp/model_state.rs` | temporarily deferred | `GB-B13-SESSION-EVENTS` |
| 31 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/acp_handler/settings.rs` | temporarily deferred | `GB-B13-SESSION-EVENTS` |
| 32 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/models.rs` | temporarily deferred | `GB-B13-SESSION-EVENTS` |
| 33 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/acp_handler/tests/subagents.rs` | temporarily deferred | `GB-BE71-SUBAGENT-TOOLS` |
| 34 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/agent_view/input.rs` | temporarily deferred | `GB-BE71-STEERING` |
| 35 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/agent_view/interactions.rs` | temporarily deferred | `GB-BE71-STEERING` |
| 36 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/agent_view/key_owner.rs` | temporarily deferred | `GB-BE71-STEERING` |
| 37 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/agent_view/key_owner_tests.rs` | temporarily deferred | `GB-BE71-STEERING` |
| 38 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/agent_view/links.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 39 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/agent_view/prompt.rs` | temporarily deferred | `GB-BE71-SLASH-SUGGEST` |
| 40 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/agent_view/selection.rs` | temporarily deferred | `GB-BE71-SELECTION` |
| 41 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/app_view.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 42 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/cli.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 43 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/dispatch/prompt.rs` | temporarily deferred | `GB-BE71-STEERING` |
| 44 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/dispatch/status.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 45 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/prompt.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 46 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/dispatch/tests/status.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 47 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/mod.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 48 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/app/mouse.rs` | temporarily deferred | `GB-BE71-SELECTION` |
| 49 | `A` `000000->100644` `crates/codegen/xai-grok-pager/src/app/startup_failure.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 50 | `A` `000000->100644` `crates/codegen/xai-grok-pager/src/app/startup_failure/render.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 51 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/memory_trace.rs` | temporarily deferred | `GB-BE71-MODALS-TRACE` |
| 52 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/notifications/hooks.rs` | temporarily deferred | `GB-BE71-NOTIFICATIONS` |
| 53 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/hook.rs` | temporarily deferred | `GB-BE71-NOTIFICATIONS` |
| 54 | `A` `000000->100644` `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/hook_tests.rs` | temporarily deferred | `GB-BE71-NOTIFICATIONS` |
| 55 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/scrollback/entry.rs` | temporarily deferred | `GB-BE71-SELECTION` |
| 56 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/scrollback/link_map.rs` | temporarily deferred | `GB-BE71-SELECTION` |
| 57 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/scrollback/render.rs` | temporarily deferred | `GB-BE71-SELECTION` |
| 58 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/scrollback/scrollback_pane.rs` | temporarily deferred | `GB-BE71-SELECTION` |
| 59 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/scrollback/state/groups.rs` | temporarily deferred | `GB-BE71-SELECTION` |
| 60 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/scrollback/state/layout.rs` | temporarily deferred | `GB-BE71-SELECTION` |
| 61 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/scrollback/state/mod.rs` | temporarily deferred | `GB-BE71-SELECTION` |
| 62 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/scrollback/state/nav.rs` | temporarily deferred | `GB-BE71-SELECTION` |
| 63 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/scrollback/state/selection.rs` | temporarily deferred | `GB-BE71-SELECTION` |
| 64 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/scrollback/state/types.rs` | temporarily deferred | `GB-BE71-SELECTION` |
| 65 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/scrollback/state/verb_group.rs` | temporarily deferred | `GB-BE71-SELECTION` |
| 66 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/scrollback/wrappers/entry_renderer.rs` | temporarily deferred | `GB-BE71-SELECTION` |
| 67 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/views/agent.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 68 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/views/dashboard/state.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 69 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/views/history_search.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 70 | `M` `100644->100644` `crates/codegen/xai-grok-pager/src/views/prompt_widget/tests.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 71 | `M` `100644->100644` `crates/codegen/xai-grok-pager/tests/pty_e2e/cancel_discards_buffered_interjection.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 72 | `M` `100644->100644` `crates/codegen/xai-grok-pager/tests/pty_e2e/empty_enter_force_sends_top_queued.rs` | temporarily deferred | `GB-BE71-STEERING` |
| 73 | `M` `100644->100644` `crates/codegen/xai-grok-pager/tests/pty_e2e/empty_enter_sends_top_not_last_of_two.rs` | temporarily deferred | `GB-BE71-STEERING` |
| 74 | `M` `100644->100644` `crates/codegen/xai-grok-pager/tests/pty_e2e/interjection_reaches_model_ctrl_l_in_vscode_family.rs` | temporarily deferred | `GB-BE71-STEERING` |
| 75 | `M` `100644->100644` `crates/codegen/xai-grok-pager/tests/pty_e2e/interjection_reaches_model_in_same_turn.rs` | temporarily deferred | `GB-BE71-STEERING` |
| 76 | `M` `100644->100644` `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_ctrl_o_send_now_queued_apple_terminal.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 77 | `A` `000000->100644` `crates/codegen/xai-grok-pager/tests/pty_e2e/permission_prompt_hook_chimes_only_on_real_wait.rs` | temporarily deferred | `GB-BE71-NOTIFICATIONS` |
| 78 | `M` `100644->100644` `crates/codegen/xai-grok-pager/tests/pty_e2e/queue_and_interjection_lifecycle.rs` | temporarily deferred | `GB-BE71-STEERING` |
| 79 | `A` `000000->100644` `crates/codegen/xai-grok-pager/tests/pty_e2e/stuck_drag_finishes_on_bare_motion_pty.rs` | temporarily deferred | `GB-BE71-SELECTION` |
| 80 | `M` `100644->100644` `crates/codegen/xai-grok-pager/tests/pty_e2e_scroll_selection.rs` | temporarily deferred | `GB-BE71-SELECTION` |
| 81 | `M` `100644->100644` `crates/codegen/xai-grok-pager/tests/pty_e2e_shell_tools.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 82 | `M` `100644->100644` `crates/codegen/xai-grok-plugin-marketplace/Cargo.toml` | temporarily deferred | `GB-B13-INTEGRATION` |
| 83 | `M` `100644->100644` `crates/codegen/xai-grok-plugin-marketplace/src/git.rs` | temporarily deferred | `GB-BE71-DU-GIT` |
| 84 | `M` `100644->100644` `crates/codegen/xai-grok-sampler/src/actor/request_task.rs` | temporarily deferred | `GB-BE71-SAMPLER-ERRORS` |
| 85 | `M` `100644->100644` `crates/codegen/xai-grok-sampler/src/client.rs` | temporarily deferred | `GB-BE71-SAMPLER-ERRORS` |
| 86 | `M` `100644->100644` `crates/codegen/xai-grok-sampler/src/events.rs` | temporarily deferred | `GB-BE71-SAMPLER-ERRORS` |
| 87 | `M` `100644->100644` `crates/codegen/xai-grok-sampler/src/retry.rs` | temporarily deferred | `GB-BE71-SAMPLER-ERRORS` |
| 88 | `M` `100644->100644` `crates/codegen/xai-grok-sampler/src/stream/collect.rs` | temporarily deferred | `GB-BE71-SAMPLER-ERRORS` |
| 89 | `M` `100644->100644` `crates/codegen/xai-grok-sampler/src/stream/messages.rs` | temporarily deferred | `GB-BE71-SAMPLER-ERRORS` |
| 90 | `M` `100644->100644` `crates/codegen/xai-grok-sampler/src/stream/messages_tests.rs` | temporarily deferred | `GB-BE71-SAMPLER-ERRORS` |
| 91 | `M` `100644->100644` `crates/codegen/xai-grok-sampler/src/stream/responses.rs` | temporarily deferred | `GB-BE71-SAMPLER-ERRORS` |
| 92 | `M` `100644->100644` `crates/codegen/xai-grok-sampler/tests/test_actor.rs` | temporarily deferred | `GB-BE71-SAMPLER-ERRORS` |
| 93 | `M` `100644->100644` `crates/codegen/xai-grok-sampling-types/src/error.rs` | temporarily deferred | `GB-BE71-SAMPLER-ERRORS` |
| 94 | `M` `100644->100644` `crates/codegen/xai-grok-sampling-types/src/lib.rs` | temporarily deferred | `GB-BE71-SAMPLER-ERRORS` |
| 95 | `A` `000000->100644` `crates/codegen/xai-grok-sandbox/src/allow_path.rs` | temporarily deferred | `GB-BE71-SANDBOX-SKILLS` |
| 96 | `M` `100644->100644` `crates/codegen/xai-grok-sandbox/src/deny/glob.rs` | temporarily deferred | `GB-BE71-SANDBOX-SKILLS` |
| 97 | `M` `100644->100644` `crates/codegen/xai-grok-sandbox/src/deny/mod.rs` | temporarily deferred | `GB-BE71-SANDBOX-SKILLS` |
| 98 | `M` `100644->100644` `crates/codegen/xai-grok-sandbox/src/lib.rs` | temporarily deferred | `GB-BE71-SANDBOX-SKILLS` |
| 99 | `M` `100644->100644` `crates/codegen/xai-grok-sandbox/src/profiles.rs` | temporarily deferred | `GB-BE71-SANDBOX-SKILLS` |
| 100 | `A` `000000->100644` `crates/codegen/xai-grok-sandbox/tests/read_write_trailing_glob_e2e.rs` | temporarily deferred | `GB-BE71-SANDBOX-SKILLS` |
| 101 | `M` `100644->100644` `crates/codegen/xai-grok-shell-base/src/util/grok_home.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 102 | `M` `100644->100644` `crates/codegen/xai-grok-shell/CHANGELOG.md` | temporarily deferred | `GB-B13-INTEGRATION` |
| 103 | `M` `100644->100644` `crates/codegen/xai-grok-shell/Cargo.toml` | temporarily deferred | `GB-B13-INTEGRATION` |
| 104 | `M` `100644->100644` `crates/codegen/xai-grok-shell/README.md` | temporarily deferred | `GB-B13-INTEGRATION` |
| 105 | `A` `000000->100644` `crates/codegen/xai-grok-shell/changelogs/1.0.1.json` | temporarily deferred | `GB-B13-INTEGRATION` |
| 106 | `A` `000000->100644` `crates/codegen/xai-grok-shell/changelogs/1.0.1.md` | temporarily deferred | `GB-B13-INTEGRATION` |
| 107 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/config.rs` | temporarily deferred | `GB-BE71-SANDBOX-SKILLS` |
| 108 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/init.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 109 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 110 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/mod.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 111 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/resource_telemetry.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 112 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/mvp_agent/tests.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 113 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs` | temporarily deferred | `GB-BE71-SUBAGENT-TOOLS` |
| 114 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs` | temporarily deferred | `GB-BE71-SUBAGENT-TOOLS` |
| 115 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/auth/flow.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 116 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/config/mod.rs` | temporarily deferred | `GB-BE71-SANDBOX-SKILLS` |
| 117 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/extensions/session_state.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 118 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/relay/sync.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 119 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/remote/pull.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 120 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/sampling/error.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 121 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 122 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/hook_dispatch.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 123 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/notification_drain.rs` | temporarily deferred | `GB-BE71-NOTIFICATIONS` |
| 124 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/prompt_build.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 125 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/recap.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 126 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/spawn.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 127 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/tasks_cancel.rs` | temporarily deferred | `GB-BE71-SUBAGENT-TOOLS` |
| 128 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/tool_calls.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 129 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/turn.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 130 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_impl/updates.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 131 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/auth_error_no_retry_tests.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 132 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/auto_wake_suppression_tests.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 133 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/cancel_running_task_tests.rs` | temporarily deferred | `GB-BE71-SUBAGENT-TOOLS` |
| 134 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/inline_auto_compact_flow_tests.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 135 | `A` `000000->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/permission_prompt_notification_tests.rs` | temporarily deferred | `GB-BE71-NOTIFICATIONS` |
| 136 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/prompt_queue_actor_tests.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 137 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/replay_buffer_send_update_tests.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 138 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/turn/auth_retry_budget_tests.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 139 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/turn/chat_history_integrity_tests.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 140 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/acp_session_tests/turn/disk_full_tests.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 141 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/compaction.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 142 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/helpers/session_compact.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 143 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/image_describe.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 144 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/persistence.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 145 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/persistence_tests.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 146 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/storage/jsonl/copy.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 147 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/storage/jsonl/mod.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 148 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/storage/jsonl/tests.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 149 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/storage/relocation/mod.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 150 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/storage/relocation/tests.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 151 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/storage/search.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 152 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/storage/search_db.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 153 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/storage/search_fts.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 154 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/session/templates/goal_summarizer_prompt.md` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 155 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/terminal/background_task.rs` | temporarily deferred | `GB-BE71-GOAL-RUNTIME` |
| 156 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/test_support/mod.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 157 | `M` `100644->100644` `crates/codegen/xai-grok-shell/src/upload/trace.rs` | temporarily deferred | `GB-BE71-MODALS-TRACE` |
| 158 | `M` `100644->100644` `crates/codegen/xai-grok-shell/tests/acp_harness/mod.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 159 | `A` `000000->100644` `crates/codegen/xai-grok-shell/tests/test_image_strip_recovery.rs` | temporarily deferred | `GB-BE71-SESSION-UX` |
| 160 | `M` `100644->100644` `crates/codegen/xai-grok-shell/tests/test_leader_soak.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 161 | `M` `100644->100644` `crates/codegen/xai-grok-telemetry/src/client.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 162 | `M` `100644->100644` `crates/codegen/xai-grok-telemetry/src/config.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 163 | `M` `100644->100644` `crates/codegen/xai-grok-telemetry/src/events/mod.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 164 | `M` `100644->100644` `crates/codegen/xai-grok-telemetry/src/session_ctx.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 165 | `M` `100644->100644` `crates/codegen/xai-grok-telemetry/src/startup.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 166 | `M` `100644->100644` `crates/codegen/xai-grok-test-support/src/resources.rs` | temporarily deferred | `GB-B13-INTEGRATION` |
| 167 | `M` `100644->100644` `crates/codegen/xai-grok-tools-api/src/slash_commands.rs` | temporarily deferred | `GB-BE71-SLASH-SUGGEST` |
| 168 | `M` `100644->100644` `crates/codegen/xai-grok-tools/src/implementations/grok_build/bash/mod.rs` | temporarily deferred | `GB-BE71-SUBAGENT-TOOLS` |
| 169 | `M` `100644->100644` `crates/codegen/xai-grok-tools/src/implementations/grok_build/list_dir/mod.rs` | temporarily deferred | `GB-BE71-DU-GIT` |
| 170 | `M` `100644->100644` `crates/codegen/xai-grok-tools/src/implementations/grok_build/mod.rs` | temporarily deferred | `GB-BE71-SUBAGENT-TOOLS` |
| 171 | `M` `100644->100644` `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs` | temporarily deferred | `GB-BE71-SUBAGENT-TOOLS` |
| 172 | `M` `100644->100644` `crates/codegen/xai-grok-tools/src/implementations/grok_build/video_gen/mod.rs` | temporarily deferred | `GB-BE71-VIDEO` |
| 173 | `M` `100644->100644` `crates/codegen/xai-grok-tools/src/lib.rs` | temporarily deferred | `GB-BE71-SUBAGENT-TOOLS` |
| 174 | `M` `100644->100644` `crates/codegen/xai-grok-tools/tests/test_subagent_soak.rs` | temporarily deferred | `GB-BE71-SUBAGENT-TOOLS` |
| 175 | `M` `100644->100644` `crates/codegen/xai-grok-update/Cargo.toml` | temporarily deferred | `GB-BE71-UPDATER` |
| 176 | `M` `100644->100644` `crates/codegen/xai-grok-update/src/auto_update.rs` | temporarily deferred | `GB-BE71-UPDATER` |
| 177 | `M` `100644->100644` `crates/codegen/xai-grok-update/src/version.rs` | temporarily deferred | `GB-BE71-UPDATER` |
| 178 | `M` `100644->100644` `crates/codegen/xai-grok-update/tests/test_concurrent_convergence.rs` | temporarily deferred | `GB-BE71-UPDATER` |
| 179 | `M` `100644->100644` `crates/codegen/xai-grok-update/tests/test_install_internal.rs` | temporarily deferred | `GB-BE71-UPDATER` |
| 180 | `M` `100644->100644` `crates/codegen/xai-grok-update/tests/test_install_sh.rs` | temporarily deferred | `GB-BE71-UPDATER` |
| 181 | `M` `100644->100644` `crates/codegen/xai-grok-version/Cargo.toml` | temporarily deferred | `GB-B13-INTEGRATION` |
| 182 | `A` `000000->100644` `crates/codegen/xai-grok-workspace-types/src/binding.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 183 | `M` `100644->100644` `crates/codegen/xai-grok-workspace-types/src/lib.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 184 | `M` `100644->100644` `crates/codegen/xai-grok-workspace-types/src/rpc/git.rs` | temporarily deferred | `GB-BE71-DU-GIT` |
| 185 | `M` `100644->100644` `crates/codegen/xai-grok-workspace-types/src/rpc/mod.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 186 | `A` `000000->100644` `crates/codegen/xai-grok-workspace-types/src/rpc/presence.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 187 | `M` `100644->100644` `crates/codegen/xai-grok-workspace/Cargo.toml` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 188 | `M` `100644->100644` `crates/codegen/xai-grok-workspace/src/activity.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 189 | `M` `100644->100644` `crates/codegen/xai-grok-workspace/src/bin/workspace_server.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 190 | `M` `100644->100644` `crates/codegen/xai-grok-workspace/src/handle.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 191 | `M` `100644->100644` `crates/codegen/xai-grok-workspace/src/hub_server.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 192 | `M` `100644->100644` `crates/codegen/xai-grok-workspace/src/permission/manager/mod.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 193 | `M` `100644->100644` `crates/codegen/xai-grok-workspace/src/permission/state.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 194 | `M` `100644->100644` `crates/codegen/xai-grok-workspace/src/preview_supervisor.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 195 | `M` `100644->100644` `crates/codegen/xai-grok-workspace/src/recovery.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 196 | `M` `100644->100644` `crates/codegen/xai-grok-workspace/src/session/mod.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 197 | `M` `100644->100644` `crates/codegen/xai-grok-workspace/src/session/tool_config.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 198 | `M` `100644->100644` `crates/codegen/xai-grok-workspace/src/status_config.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 199 | `M` `100644->100644` `crates/codegen/xai-grok-workspace/src/upload/mod.rs` | temporarily deferred | `GB-BE71-STARTUP-STATUS` |
| 200 | `M` `100644->100644` `crates/codegen/xai-ratatui-inline/src/terminal.rs` | temporarily deferred | `GB-BE71-SELECTION` |
| 201 | `M` `100644->100644` `crates/codegen/xai-ratatui-inline/src/tests.rs` | temporarily deferred | `GB-BE71-SELECTION` |
| 202 | `A` `000000->100644` `crates/codegen/xai-tty-utils/src/child_wait.rs` | temporarily deferred | `GB-BE71-GOAL-RUNTIME` |
| 203 | `A` `000000->100644` `crates/codegen/xai-tty-utils/src/child_wait_tests.rs` | temporarily deferred | `GB-BE71-GOAL-RUNTIME` |
| 204 | `M` `100644->100644` `crates/codegen/xai-tty-utils/src/lib.rs` | temporarily deferred | `GB-BE71-GOAL-RUNTIME` |
| 205 | `M` `100644->100644` `crates/codegen/xai-tty-utils/src/process_resources.rs` | temporarily deferred | `GB-BE71-GOAL-RUNTIME` |
| 206 | `M` `100644->100644` `crates/codegen/xai-tty-utils/src/process_scope.rs` | temporarily deferred | `GB-BE71-GOAL-RUNTIME` |
| 207 | `M` `100644->100644` `crates/common/xai-interjection-core/src/format.rs` | temporarily deferred | `GB-B13-WIRE` |
| 208 | `M` `100644->100644` `crates/common/xai-interjection-core/src/lib.rs` | temporarily deferred | `GB-B13-WIRE` |
| 209 | `M` `100644->100644` `crates/common/xai-tool-protocol/src/frames.rs` | temporarily deferred | `GB-B13-WIRE` |
| 210 | `M` `100644->100644` `crates/common/xai-tool-types/src/lib.rs` | temporarily deferred | `GB-BE71-SUBAGENT-TOOLS` |
| 211 | `M` `100644->100644` `crates/common/xai-tool-types/src/task.rs` | temporarily deferred | `GB-BE71-SUBAGENT-TOOLS` |
