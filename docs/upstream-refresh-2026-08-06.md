# Upstream refresh parity ledger — 2026-08-06

This ledger records the immutable fetch pins, the safe adoption boundary, the
provider-reference review queue, and every still-open Grok behavior family for
the August 6 refresh. It continues the checked-in
[August 2 ledger](upstream-refresh-2026-08-02.md). Fetched history is evidence;
it was not merged, rebased, or acknowledged as reviewed upstream history.

## Immutable boundary

- Starting Enhanced commit: `8887826302b79e17d31267cc47be06dce2b4c579`.
- Isolated candidate: branch `refresh/upstreams-20260806-r2`, worktree
  `/home/ruttydm/Projects/worktrees/grok-build-enhanced-refresh-20260806-r2`.
- Audit timestamp: `2026-08-06T09:43:14Z`.
- Root `Cargo.toml` remained byte-identical, SHA-256
  `28a3ea7e1c859729a0c5cf77f87ff7f0ece319a576697b274917359e11be480b`.
- No push, tag, release, pull-request mutation, live-provider request, or
  credential import was performed.

## Source pins

| Source | Prior reviewed | Latest fetched | Commits | Changed paths | Raw digest |
| --- | --- | --- | ---: | ---: | --- |
| Grok Build | `dd04f397b1d02f2272b092555669dfba1f01bc85` | `a5589e958437d79e13db026eedcb1720bffd4063` | 5 | 633 | `30141f979978f528cf42f7cdd43ab943fadd65d91fdf4ad9355bfc9257bc73bc` |
| OpenAI Codex | `2c005abb0765bfe3ef42a23fe88d5b806184fa83` | `7a0e974e08c798d1e8d59d407aeb6e24db1313af` | 185 | 1,040 | `64e366f35fffdadcb178f6deda60b79e20dabf5930ea6b5ef40680380adea6c8` |
| OpenCode | `1882c33827cf0ce5c948b69ab5a87ed8f6790cf8` | `def7220bfc65b84046e597e9be772eae81f663ff` | 67 | 363 | `3b4554ffd2f7211263bd2addb945c2c7c3ec634090b8c4fd1645204fc60205df` |
| Oh My Pi | `01c1f91ff529c6af3fc27724a8ba429d83d41aed` | `1e492d6ff9b8d4412591942b11fe06e1395ae80f` | 211 | 668 | `4c8e29b2735f96ce29d3fd8e7bc7c4184ff23200282f21e6ecf770b36e3496` |
| Warp themes | `b385044250f1ed3c9379ab34a8fe82f02fdffaa4` | `82e51dcf9b47912d551107748ba3297a21b2eff3` | 1 | 3 | `a3112ac32855e644d8d0a7b30348be69233855255a85bc065ba66542cf5d5f75` |
| Kimi Code | `bfa00807c975fdc5b84dda32d47b16b09e8d42c1` | `013203421df03b655cc04d1095e4e41d83c2ae44` | 58 | 1,583 | `771b800c1e633d76381e6288ba4fc6c8f1bc358f3459b2951d90028d4f52a818` |
| Kimi CLI | `4a550effdfcb29a25a5d325bf935296cc50cd417` | `cbc15c076d17f70fec9f89c90c0502e68657f505` | 2 | 9 | `1d65f0635c819c8d11d271a5dc149d79742f476f18bb41c3ee0eaea795586bfe` |
| CodexBar | `cc8da27cec92029a6435bfee4a703a719290234e` | `005a71f550cd8351522744043a3cd5f9311f717d` | 459 | 1,091 | `e3f6608730ae93ce0c0ae6965435786dd263a520bc86a9058a99c2a3e87d6668` |
| models.dev | `f67be44f095a4ab24ceab33c3907317bb0375087` | `1d09b08b8c9d83ac4a59d38299f54630c83e802f` | 137 | 1,805 | `f0c8414a4cdbae833babd579493ae767769033190c8931a5fbeebd79e13bc3ea` |

OpenCode Codex auth, Exa MCP, Z.AI coding plugins, GLM-5, and the Z.AI
usage-browser reference did not advance. Exa renamed its tracked branch from
`master` to `main` without changing the pinned commit. The configured Z.AI
Python SDK remote returned “repository not found”; its reviewed/latest pin
remains `ca5109c0aa9bf173839be391b4b14aeadf9a9bf9` and no runtime claim was made.

## Safe adoption boundary

The complete `dd04f397…a4221165` behavior inventory and its 165-row raw-path
ledger are in the August 2 ledger. Its 19 adopted, 3 already-equivalent, and 6
not-applicable decisions were carried onto this candidate as commit `68b352e`.
The locked pager build passes at that boundary.

Warp commit `82e51dcf` adds only `standard/hadar_theme.yaml` and its upstream
README/preview. The deterministic vendor tool copied the YAML byte-for-byte,
updated the 341-theme manifest, and advanced all build, package, and legal
revision locks. Classification: **adopt / closed** (`WARP-82E5-HADAR`).

## Grok behavior inventory after `a4221165`

The four later snapshots are linear and independently pinned:

| Snapshot | Parent | Changed paths | Raw digest |
| --- | --- | ---: | --- |
| `780d1388fff103ff0db0d8c14de65af6225b4860` | `a4221165824e5b1f5c4c10b7459f65e78dd6448d` | 323 | `da017e8802174d9242e5fa0cedf77f45845786c7695887bc7a9f8fd10445e4ae` |
| `e5478eff1e4050558e12e1328b85e6616632efb6` | `780d1388fff103ff0db0d8c14de65af6225b4860` | 73 | `5592afca640ca549598949a30467225810fed6a8b7ba57f937d80f48764d737e` |
| `ed6d543643628663873c5de28298e022ed634238` | `e5478eff1e4050558e12e1328b85e6616632efb6` | 168 | `0075cd67fbb8ea2253ebbcec5d64065719823c44470c96af2099778ad87665a4` |
| `a5589e958437d79e13db026eedcb1720bffd4063` | `ed6d543643628663873c5de28298e022ed634238` | 232 | `7e87725555a7d297acd4a03133a1606a52560b8e4bb9ec798c696adcc1bd0600` |

Each row below is **temporarily deferred / open**. The shared owner is
“Grok preserved surfaces”; the blocker is the same-source command,
persistence, roster, and session-storage schema migration crossing Enhanced’s
provider-bound credential/session extensions. Target: the next upstream parity
milestone, no later than the next refresh. Acceptance: port the observable
behavior without cross-provider credential fallback, keep the root manifest
generated, and pass focused tests plus the locked pager build. Intended tests:
the named upstream regression tests adapted to Enhanced, provider-isolation
negative tests for any touched auth/session path, and the existing relevant
crate suites.

### `780d1388` / 0.2.118

| Stable ID | Observable behavior and user impact while open |
| --- | --- |
| `GB-780D-001` | Dashboard and welcome-list session deletion; users lack the new complete idle/current-session lifecycle semantics. |
| `GB-780D-002` | Shortcut help for prompt history and conversation search; help may lag actual bindings. |
| `GB-780D-003` | tmux reduced-color diagnosis and repair; affected terminals may retain degraded colors. |
| `GB-780D-004` | `/btw` overload retry; transient overload can terminate a side question. |
| `GB-780D-005` | Temporary session-sharing disablement; the fork must verify the same safety gate. |
| `GB-780D-006` | Stop/Ctrl+C cancellation during compaction; cancellation parity needs the later command schema. |
| `GB-780D-007` | Automatic recap de-duplication; a recap can be projected twice in an uncovered edge case. |
| `GB-780D-008` | Background wait timeout schema/ceiling agreement; descriptions can diverge from execution bounds. |
| `GB-780D-009` | Fast background-task completion state; a task may remain shown as running. |
| `GB-780D-010` | Plan-mode indicator removal after approval; stale mode UI can linger. |
| `GB-780D-011` | Plan-preview scrollbar dragging; mouse interaction remains incomplete. |
| `GB-780D-012` | Additional inference context-length classification; some compaction-recoverable failures may surface directly. |

### `e5478eff` / 0.2.119

| Stable ID | Observable behavior and user impact while open |
| --- | --- |
| `GB-E547-001` | Editable free-form always-allow bash glob patterns; users retain the older scope editor. |
| `GB-E547-002` | Response-top jump arrow for long answers; navigation remains less direct. |
| `GB-E547-003` | Broader safe auto-mode read-only git and append approvals; extra prompts remain. |
| `GB-E547-004` | Mermaid buttons in plan previews; open/copy affordances remain absent. |
| `GB-E547-005` | Dead gateway socket detection and recovery; stale connections can take longer to recover. |
| `GB-E547-006` | Tab navigation within question cards; focus can move to scrollback. |
| `GB-E547-007` | Reject pasted garbage in the resume picker; invalid text can still reach session loading. |
| `GB-E547-008` | Bounded background completion messages; large logs can create oversized completion text. |
| `GB-E547-009` | Plan scrollbar border clicks and Terminal.app rendering; mouse/stripe defects remain. |
| `GB-E547-010` | Expired external-auth interactive recovery; affected providers can repeat a silent 401 path. |
| `GB-E547-011` | `/btw` reuse of the parent cached prefix; side questions miss the new latency optimization. |
| `GB-E547-012` | Faster doctor/tmux startup with no live tmux processes; startup retains redundant probing. |

### `ed6d5436` / 0.2.120

| Stable ID | Observable behavior and user impact while open |
| --- | --- |
| `GB-ED6D-001` | Pre-session model-picker status/menu update; the selected model can appear stale before first prompt. |
| `GB-ED6D-002` | Changes-panel refresh after an agent commit; the panel can show stale unstaged state. |
| `GB-ED6D-003` | Full background-log size and read hint with a captured prefix; completion metadata can under-report output. |
| `GB-ED6D-004` | Clear GitHub-export error for old hibernated sessions; users receive a generic failure. |

### `a5589e95` post-0.2.120 snapshot

| Stable ID | Observable behavior and user impact while open |
| --- | --- |
| `GB-A558-001` | Generated protobuf `Debug` redaction; generated diagnostics need proof that annotated fields cannot expose secrets. |
| `GB-A558-002` | Environment/OSC/tmux appearance resolution; theme selection can miss terminal-advertised appearance. |
| `GB-A558-003` | Central input-key ownership and queue reordering; overlapping TUI handlers can disagree on ownership/order. |
| `GB-A558-004` | Structured bounded provider-error parsing and shared edge retry policy; some gateway errors can classify inconsistently. |
| `GB-A558-005` | First-party xAI API-key validity probe before advertisement; an invalid environment key can be offered prematurely. |
| `GB-A558-006` | Session close/delete hard-stop, attach settling, replica finalization, and old-thread drain budgets; lifecycle races remain unqualified. |
| `GB-A558-007` | Side-call persistence, last-turn summaries, recap reconciliation, and disk-full surfacing; session durability needs a provider-safe schema port. |
| `GB-A558-008` | MCP expired-config re-enable and managed-server reconciliation; a repaired MCP entry may remain disabled or stale. |
| `GB-A558-009` | Auto-mode security findings and restore-fetch workflow; repository restore diagnostics remain incomplete. |
| `GB-A558-010` | Remaining markdown, selection, tmux notification, remote pull, and worktree/session polish in the 232-path snapshot; preserved-surface edge cases remain unqualified until their upstream tests are mapped. |

Summary after the carried August 2 boundary: **1 adopted/closed**, **38
temporarily-deferred/open**, and **0 unclassified behavior families**. Because
open Grok adoption obligations remain, `Reviewed` stays at `dd04f397…` and no
upstream acknowledgement record or marker merge may be created.

## Provider-reference inventory

These sources are normative only inside their declared adapter or research
scope. Their reviewed pins remain unchanged unless explicitly noted.

| Stable ID | Classification/state | Result |
| --- | --- | --- |
| `CDX-7A0E-AUTH` | temporarily deferred / open | Codex added managed login-method/workspace policy and process-scoped ChatGPT routing. Enhanced has provider-scoped preferred-auth controls but has not mapped the new managed policy into its direct subscription adapter. |
| `CDX-7A0E-WIRE` | not applicable / closed | Responses-lite namespaced tools, dual-WebSocket code mode, Guardian, apps/plugins, app-server pagination, and Codex TUI/storage changes do not replace Grok’s agent, tool, permission, TUI, or session architecture. |
| `CDX-7A0E-USAGE` | already equivalent / offline-qualified | Enhanced already records provider-reported input/output/cache/reasoning usage without sharing it across providers; existing synthetic provider tests remain the closure gate. |
| `OC-DEF7-RETRY` | temporarily deferred / open | OpenCode expanded retryable message patterns and ACP cache-write usage. Enhanced uses typed provider-specific retry/usage handling; exact new patterns need a bounded compatibility review. |
| `OC-DEF7-APP` | not applicable / closed | Desktop/TUI/localization/server-proxy work is outside the interoperability adapter. |
| `KIMI-0132-ADAPTER` | already equivalent / offline-qualified | The advanced tree is dominated by Kimi’s v2 app engine, MCP, UI, minidb, and server work. No new API-key catalog, inference, usage, or hosted-web wire requirement was found beyond existing isolated bindings. |
| `KIMI-CBC1-BETA` | already equivalent / closed | Legacy Kimi now omits an empty `anthropic-beta` header. Enhanced always sends the non-empty pinned context-management beta on Messages and forbids that header on Chat Completions. |
| `OMP-1E49-HARNESS` | not applicable / closed | Oh My Pi’s 211 commits are non-normative coding-harness/app-engine inspiration and do not authorize replacing Grok surfaces. |
| `MODELS-1D09-CATALOG` | not applicable / closed | models.dev metadata does not override authenticated provider catalogs or establish new runtime models automatically. |
| `CODEXBAR-005A-ZAI` | not applicable / closed | Usage presentation remains research only and establishes no Z.AI provider, credential path, login, or product claim. |
| `WARP-82E5-HADAR` | adopt / closed | Hadar is vendored byte-for-byte with updated manifest, legal notices, and package/build revision locks. |

Open provider rows are review obligations, not evidence of credential leakage or
cross-provider fallback. They prevent the corresponding provider `Reviewed`
pins from advancing. For `CDX-7A0E-AUTH`, the owner is “Codex provider auth,”
the blocker is the missing Enhanced configuration contract for administrator
managed login/workspace restrictions, the target/deadline is the next provider
refresh, and acceptance requires fail-closed method/workspace enforcement plus
synthetic login, refresh, logout, and xAI/Kimi-negative tests. For
`OC-DEF7-RETRY`, the owner is “Provider interoperability,” the blocker is a
bounded mapping from newly added free-text patterns into typed Enhanced errors,
the target/deadline is the next provider refresh, and acceptance requires exact
retry-veto, attempt-bound, cache-write usage, and foreign-provider negative
tests. Their impact is incomplete compatibility proof, not a known production
credential-routing defect.

## Validation record

| Check | Result |
| --- | --- |
| `CARGO_INCREMENTAL=0 cargo check --locked -p xai-grok-pager-bin` at the carried adoption boundary | passed |
| `cargo fmt --all -- --check` | passed |
| Warp vendoring security/reproducibility unit tests | 5 passed |
| `xai-grok-pager-render` `warp_vendor_build_validation` | 2 passed |
| Warp corpus check | passed: 341 themes; category counts and revision matched |
| `python3 -I -B fork/scripts/check_fork_contracts.py` | passed |
| `python3 -I -B fork/scripts/check_manifest.py --strict-coverage` | passed before the final count-lock correction; rerun in the handoff commit |
| `CARGO_INCREMENTAL=0 cargo check --locked -p xai-grok-pager-bin` after vendoring | passed |
| Root `Cargo.toml` generated-manifest guard | unchanged, hash recorded above |
| Warp vendor plan/apply | 1 add, 2 generated updates, 0 removals; 341 themes |

Live Codex and Kimi requests remain credential-gated and were not attempted.
