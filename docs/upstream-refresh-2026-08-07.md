# Upstream refresh parity ledger — 2026-08-07

This ledger records the immutable August 7 fetch pins, the one narrow provider
wire adoption, and every still-open Grok and provider-reference obligation. It
continues the checked-in [August 6 ledger](upstream-refresh-2026-08-06.md).
Fetched history is evidence; no upstream tree was merged, rebased, or
acknowledged as reviewed history.

## Immutable boundary

- Starting audited candidate: `4530ba0a48877cfef999f8511b7d42ec854c3cb5`,
  tree `56d021ab37b4a6cff5f229613afc10421c7eb228`.
- Isolated candidate: branch `refresh/upstreams-20260807`, worktree
  `/home/ruttydm/Projects/worktrees/grok-build-enhanced-refresh-20260807`.
- Audit timestamp: `2026-08-07T13:29:56Z`.
- Root `Cargo.toml` remained byte-identical, SHA-256
  `28a3ea7e1c859729a0c5cf77f87ff7f0ece319a576697b274917359e11be480b`.
- No push, tag, release, pull-request mutation, live-provider request, or
  credential import was performed.

## Newly fetched source deltas

Counts and digests in this table cover yesterday's latest-fetched pin through
today's immutable pin, not the older reviewed-to-latest queue.

| Source | Prior fetched | Latest fetched | Commits | Changed paths | Raw digest |
| --- | --- | --- | ---: | ---: | --- |
| Grok Build | `a5589e958437d79e13db026eedcb1720bffd4063` | `393430ee4934bc791b0d538f304a21691c517433` | 1 | 263 | `944fbbb9f329218e52bc62be17cfef3931a5d6c5e6f8fbf1334b37c30071514a` |
| OpenAI Codex | `7a0e974e08c798d1e8d59d407aeb6e24db1313af` | `a4b129eb3e1a6929c09d6e2e1af0638122c56f0d` | 42 | 227 | `bf86b215e239729e66104bc19613730bcc3d5552e9d15b0116bd57ef88d2f7e1` |
| OpenCode | `def7220bfc65b84046e597e9be772eae81f663ff` | `284214c78d32a09fd9c729bdefc07be50f74eb40` | 26 | 301 | `33b91b791ef41e2c210f8f014c43f40e755a2562ec3dfdf463162120146b991d` |
| Oh My Pi | `1e492d6ff9b8d4412591942b11fe06e1395ae80f` | `39477ba39bfbdc6be2cfff0efde979dd32bd7eb7` | 237 | 533 | `325be6267fc77601c87c5947f5b02a5d2d67b42a263c4a579f3d7750f4833c6e` |
| Kimi Code | `013203421df03b655cc04d1095e4e41d83c2ae44` | `0b2e803d5e71afaab45212bb2ee6117ecbf8bbc9` | 18 | 706 | `51432077e6a2bda9b7f6854b058072910827e620f36b73334d23d362a46d95a8` |
| CodexBar | `005a71f550cd8351522744043a3cd5f9311f717d` | `22b24b885693e890af52df15c29f7ca024904c74` | 23 | 43 | `b711dde9c61fd0d99825ccb655241821b5f439c9323b79ac324d8b7be85324b1` |
| models.dev | `1d09b08b8c9d83ac4a59d38299f54630c83e802f` | `433e98fb61999384bc7b6cf7470dc9bad81f8d2a` | 112 | 171 | `764cffaa4301b400b8c4c97fa3264e4f4c5851b3ee54cba3f8ecf94b06c0ade0` |

The exact fetched tree identities are:

| Source | Latest fetched tree |
| --- | --- |
| Grok Build | `35b3a320462942c297e6d8d3e8a8a2558c835c9d` |
| OpenAI Codex | `559a6b0d4afb8a2e7ff08eb1ea2ffc4a7afd0035` |
| OpenCode | `be70599523b41cbef24c0b9675cfdf6232c54a0a` |
| OpenCode Codex auth | `1da59bae7069563b2817143567b57c78e5758300` |
| Oh My Pi | `33998b143a518cfcd7df4566f7baa57b78dfa8e4` |
| Warp themes | `2893387a4769db78ce4ef5294b8cc39bacd80616` |
| Kimi Code | `b7cc7a93e7197be170a852b0bdf2aabb304cbf77` |
| Kimi CLI | `e1d6d5b2827f8a14c2edc4bc8658ad5cf19d52e7` |
| Z.AI coding plugins | `efea84479dc67bc4af7d2c3b59b4aca8f5332899` |
| GLM-5 | `8ac85a6098dc83ebd539a9093442e8192fbf052c` |
| CodexBar | `0f0adc34719d177b98ed5fd010736141215914f7` |
| Z.AI usage browser | `08b00849b96c5883a265f4d4d43e2836d01cdd9d` |
| models.dev | `aa53a470caccaf2eb4a5274d5b7121489e17e5f0` |
| Exa MCP | `7d76165a926eace7cb6bc19972a1c08a0e58c856` |

Every successfully fetched advancing source retained its recorded reviewed
revision as an ancestor of the pinned head. The Grok reviewed pin is likewise
an ancestor of `393430ee`; the disconnected-history safety rule still applies
because that ancestry exists in the fetched upstream line, not Enhanced's
first-parent history.

OpenCode Codex auth, Warp themes, Kimi CLI, Exa MCP, Z.AI coding
plugins, GLM-5, and the Z.AI usage-browser reference did not advance. The
configured Z.AI Python SDK remote again returned “repository not found”; its
reviewed/latest pin remains `ca5109c0aa9bf173839be391b4b14aeadf9a9bf9`
and no runtime claim was made.

## Safe provider-wire adoption

OpenAI Codex commit `270d93268` adds `x-codex-routing-hint` with the selected
model and optional service tier. Enhanced now creates that header only after
the direct ChatGPT subscription provider has passed its endpoint and dynamic
credential-binding checks. The header is provider-owned: generic header
injection cannot spoof it, and xAI, Kimi, and Custom requests never receive it.
The focused tests cover ordinary and priority-tier values, hostile pre-auth
injection replacement, invalid values, and negative provider routing.

Classification: **adopt / closed** (`CDX-A4B1-ROUTING`). This is an
independent compatibility implementation; no Codex source was copied.

## Carried Grok obligations

All 38 August 6 Grok deferrals remain **temporarily deferred / open**. No
closure evidence landed in this refresh. Their observable impacts, common
owner, blocker, acceptance criteria, deadline, and intended tests are
incorporated unchanged from the [August 6 ledger](upstream-refresh-2026-08-06.md#780d1388--02118).
The stable IDs remain live:

| Source snapshot | Carried stable IDs |
| --- | --- |
| `780d1388` / 0.2.118 | `GB-780D-001`, `GB-780D-002`, `GB-780D-003`, `GB-780D-004`, `GB-780D-005`, `GB-780D-006`, `GB-780D-007`, `GB-780D-008`, `GB-780D-009`, `GB-780D-010`, `GB-780D-011`, `GB-780D-012` |
| `e5478eff` / 0.2.119 | `GB-E547-001`, `GB-E547-002`, `GB-E547-003`, `GB-E547-004`, `GB-E547-005`, `GB-E547-006`, `GB-E547-007`, `GB-E547-008`, `GB-E547-009`, `GB-E547-010`, `GB-E547-011`, `GB-E547-012` |
| `ed6d5436` / 0.2.120 | `GB-ED6D-001`, `GB-ED6D-002`, `GB-ED6D-003`, `GB-ED6D-004` |
| `a5589e95` snapshot | `GB-A558-001`, `GB-A558-002`, `GB-A558-003`, `GB-A558-004`, `GB-A558-005`, `GB-A558-006`, `GB-A558-007`, `GB-A558-008`, `GB-A558-009`, `GB-A558-010` |

## Grok 0.2.121 behavior inventory

Grok commit `393430ee4934bc791b0d538f304a21691c517433` is a single
21,121-addition monorepo sync. Its 46 release entries and the additional
user-visible behavior found in code and documentation are exhaustively mapped
below. Each row is **temporarily deferred / open**.

The shared owner is “Grok preserved surfaces.” The blocker is the coordinated
command, session-storage, task-roster, permission, ACP, and UI schema migration
on top of Enhanced's provider-scoped credential/session extensions. Target:
the next upstream parity milestone, no later than the next refresh.
Acceptance: port the observable behavior without cross-provider credential
fallback, keep the root manifest generated, and pass the named upstream
regression tests adapted to Enhanced plus provider-isolation negative tests for
every touched auth/session seam and the locked pager build.

| Stable ID | Pinned upstream path families | Observable behavior and user impact while open |
| --- | --- | --- |
| `GB-3934-001` | `xai-grok-pager/src/app/{agent_view,dispatch,app_view}.rs`, `views/{dashboard,welcome}` | Dashboard summaries, grouped Extensions/Skills, single-agent navigation, `/new`, `exit`/`quit`, and `/delete` lifecycle polish; dashboard state and navigation can remain stale or less informative. |
| `GB-3934-002` | `xai-grok-pager-render/src/render/image_overlay*`, `xai-grok-pager/src/scrollback/blocks/tool/web_fetch.rs`, shell MCP paths | Large MCP image-result preservation and re-enable visibility; screenshots can truncate/corrupt and repaired disabled servers can remain hidden. |
| `GB-3934-003` | `xai-grok-tools/src/implementations/grok_build/task/**`, shell subagent/workflow paths, pager queue/turn paths | Background-subagent keep-working reminders, lossless/reorderable queued prompts/images/slash commands, cancel wake suppression, and bounded task/workflow admission; queued work can disappear, restart after cancellation, or over-admit children. |
| `GB-3934-004` | pager and shell ACP/session lifecycle paths, `xai-computer-hub-sdk/src/{connection,server}.rs` | Restored-child resume, transcript-free reattach, explicit close, conversation-only remote resume unless `--restore-code`, and instant empty-session exit; remote lifecycle races and slow teardown remain. |
| `GB-3934-005` | `xai-grok-shell/src/session/storage/search*`, session fork/load paths | Memory-bounded large-session forks and the split local FTS bootstrap/content index with cross-directory UUID lookup; large forks can over-allocate and resume search can miss or stall. |
| `GB-3934-006` | `xai-fast-worktree/src/**`, pager `git_info.rs`/`worktree_cmd/**`, shell workflow host paths | Hand-initialized default-branch detection and bounded large/shallow repository restore; restore and worktree operations can choose incorrectly or hang. |
| `GB-3934-007` | pager `main.rs`, shell agent/config/init and authentication paths | No project-directory prompt outside projects and a real validity check before advertising first-party xAI environment login; startup can ask an unnecessary question or skip login on an invalid xAI key. |
| `GB-3934-008` | pager plan, model, settings, agent-view render/input, and session paths | Model picker/command palette availability during plan review and accurate resumed/transitioned plan-agent-ask indicators; review controls and displayed mode can be wrong. |
| `GB-3934-009` | pager `slash/commands/feedback.rs`, actions/effects, `views/question_view.rs`, PTY feedback tests | Dedicated bare `/feedback` pane, inline submission, and composer-mode preservation; feedback retains the older prompt-mode behavior. |
| `GB-3934-010` | pager-render theme/line/highlight paths, minimal overlay, pager selection/turn-status/terminal paths | SSH/tmux auto-theme detection, Voice/Finance cards, narrow markdown table reflow, selectable pinned headers, CJK selection edges, wrapped-diff syntax, and terminal-mode reset; these TUI/rendering defects remain. |
| `GB-3934-011` | pager `slash/**`, `views/{prompt_widget,question_view,slash_dropdown}.rs`, shell slash/inspect paths | Consistent blocking-card Tab/Esc, Enter-to-run slash selection, dashboard session gating, and visible built-in/qualified skill collision provenance; keyboard ownership and colliding command identity remain ambiguous. |
| `GB-3934-012` | `xai-grok-workspace/src/permission/**`, shell `session/telemetry/permission.rs`, Grok bash-tool paths | Complete/expandable bash permission scripts, centralized request classification/reasons, and permission decision analytics; users can approve against incomplete-looking context and permission evidence remains fragmented. |
| `GB-3934-013` | shell sampler-turn/tool-call/error paths, pager turn-status and `/btw` overlay paths | Clean provider error banners, broader bounded server-error retry, and fully wrapped `/btw` errors; transient failures and provider diagnostics remain less consistent. |
| `GB-3934-014` | pager dashboard/agent-view/session paths, shell recap/feedback/memory paths, `xai-chat-state/src/compaction_utils.rs` | Previous-turn answer/finding summaries and busy/new-turn-safe recap reconciliation; summaries can reflect activity rather than outcome and recaps can appear mid-turn. |
| `GB-3934-015` | pager `app/dispatch/rewind.rs`, `views/rewind.rs`, shared `ui_config.rs`, session docs | `/rewind` becomes conversation-only, leaves files untouched, and gains a persisted “Confirm before rewind” choice; Enhanced still exposes the older file-restoring contract and lacks the confirmation setting. |
| `GB-3934-016` | pager `acp/**` and `app/acp_handler/**`, shell ACP session paths, computer-hub connection/server paths | ACP version-mismatch detection plus reconnect/close notification settling; skewed or reconnecting clients can fail without the new guided recovery and drain guarantees. |
| `GB-3934-017` | pager `disk_usage_cmd/**`, `fs_size*`, `worktree_cmd/**`, CLI/lib and grok-home tests | `grok du` / `grok disk-usage` with filesystem-aware, registry-qualified text/JSON reporting and safe cleanup guidance; users lack the new storage diagnosis surface. |
| `GB-3934-018` | `xai-grok-telemetry/src/{startup,events/**,external/**}`, shell telemetry paths and monitoring docs | Startup phase/timeout metrics and structured permission telemetry while preserving ordinary upstream telemetry policy; operators lack the new startup and permission observability. |

The duplicate `/btw` wrapping release entry maps once to `GB-3934-013`.
Internal refactors with no separate observable contract—module moves,
generated dependency locks, helper extraction, and test-only changes—map to
the behavior row they support. There are **18 new open families and 0
unclassified 0.2.121 behaviors**.

After carry-forward, the Grok queue is **56 temporarily-deferred/open**. Grok
`Reviewed` therefore stays at `dd04f397b1d02f2272b092555669dfba1f01bc85`;
no upstream acknowledgement record or marker merge may be created.

## Provider-reference inventory

These sources are normative only inside their declared adapter,
interoperability, or research scope.

| Stable ID | Classification/state | Result |
| --- | --- | --- |
| `CDX-7A0E-AUTH` | temporarily deferred / open | Carried unchanged: managed login-method/workspace policy still lacks a fail-closed Enhanced configuration contract. |
| `CDX-A4B1-ROUTING` | adopt / closed | Added the sealed subscription-only model/service-tier routing hint with spoofing and foreign-provider negative tests. |
| `CDX-A4B1-AGENT-ID` | not applicable / closed | Agent-identity JWT AuthAPI/JWKS endpoint overrides belong to a Codex auth mode Enhanced does not implement; they do not authorize another credential path. |
| `CDX-A4B1-APP` | not applicable / closed | Codex MCP runtime, app-server, TUI export/archive, rollout migration, code-mode, sandbox, environment provisioning, and host-skill-loader work does not replace Grok application surfaces. |
| `OC-DEF7-RETRY` | temporarily deferred / open | Carried unchanged: the earlier retry-pattern/cache-write compatibility mapping remains incomplete. |
| `OC-2842-CHRONOLOGY` | not applicable / closed | The new message ordering, compaction serialization, truncation cleanup, cursor, desktop, localization, and app UI changes belong to OpenCode's application architecture. |
| `KIMI-0B2E-ADAPTER` | already equivalent / offline-qualified | The new range changes Kimi's v2 DI/app engine, session outcome indexing, compaction accounting, UI, plugins, and global MCP-auth status. It introduces no new API-key catalog, inference, usage, hosted-web, or media wire requirement beyond Enhanced's isolated adapter. |
| `OMP-3947-HARNESS` | not applicable / closed | The 237 commits remain non-normative harness/app-engine inspiration; auth, retry, catalog, MCP, UI, and desktop changes do not replace Grok surfaces or provider adapters. |
| `MODELS-433E-CATALOG` | not applicable / closed | New third-party metadata, including `grok-4.1-fast` and several Kimi records, does not override authenticated first-party catalogs or auto-establish runtime models. |
| `CODEXBAR-22B2-ZAI` | not applicable / closed | Dashboard/serve and usage-presentation changes remain research only and establish no Z.AI provider, credential path, login, or product claim. |

The carried open provider rows retain their August 6 owner, blocker, impact,
deadline, acceptance criteria, and intended negative tests. They prevent the
corresponding provider `Reviewed` pins from advancing; they are not evidence of
a known credential-routing defect.

## Validation record

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | passed |
| Focused `xai-grok-sampler` routing-hint tests | 4 passed: sealed ordinary/priority wire values, spoof replacement, invalid-value omission, and xAI/Kimi/Custom negatives |
| `CARGO_INCREMENTAL=0 cargo check --locked -p xai-grok-pager-bin` | pending final run |
| `python3 -I -B fork/scripts/check_fork_contracts.py` | pending final run |
| `python3 -I -B fork/scripts/check_manifest.py --strict-coverage` | pending final run |
| Root `Cargo.toml` generated-manifest guard | unchanged; hash recorded above |

Live Codex and Kimi requests remain credential-gated and were not attempted.
