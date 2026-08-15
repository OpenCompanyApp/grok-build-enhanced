# Upstream refresh and GLM-5.3 research — 2026-08-15

This run starts from Enhanced commit
`9b6caf8c69ea4abeec24a3b49cff957f5bc4ecf3`, tree
`693e6271d413fa7b3da3bd7fcf2153460a08e711`, on the isolated
`release/glm53-20260815` branch. The user's active
`agent/fix-settings-refcell-race` checkout and its modified/untracked files
were not changed, stashed, cleaned, rebased, or used as a publication source.

The run fetches the tracked source heads, records public first-party GLM-5.3
Coding Plan contracts as research, corrects documentation that contradicted the
fork's research-only Z.AI boundary, and carries every parity obligation forward.
It does not add a Z.AI runtime provider, accept credentials, advance a Reviewed
revision, create an upstream acknowledgement, or authorize a release from an
ineligible history.

## Frozen advancing source heads

| Source | Frozen commit | Tree | Change from prior fetched pin |
| --- | --- | --- | --- |
| Grok Build | `d6a22a1aed70b58d30a0f82a1a2a76ce1301631e` | `49776b0e96d342b09770f140dee7078ff037b346` | 1 commit / 202 paths |
| OpenAI Codex | `c4941302c73c6322b153bba13ac0a9f4396301d6` | `d67ff5e31ff25a69d01b3cc67fedcdaae2ebfd20` | 32 commits / 235 paths |
| Kimi Code | `6b72345f8bb03487e3bcc05b541e65484818428c` | `d325b1924ed35fea50f3317541c4c1cffcdf180b` | 1 commit / 5 paths |
| CodexBar | `f15f142a7787143a4a991ed4ff54c3057ae412ba` | `594c92ff2432d013c8d62947bd0d69ec49d371b3` | 19 commits / 33 paths |
| models.dev | `65db14442d690d1e21d2d6e739ccf6451386d4ab` | `80411aae1bbb9daaf2a257f567ced3a713abf88b` | 41 commits / 248 paths |

Every advancing pin is a descendant of its preceding fetched pin. OpenCode,
OpenCode Codex auth, Oh My Pi, Warp themes, Kimi CLI, Z.AI coding plugins,
GLM-5, the Z.AI usage helper, and Exa did not advance. The tracked Z.AI Python
SDK URL again returned `Repository not found`; its immutable recorded pin was
retained and no replacement identity was inferred.

Only **Latest fetched** and the check date move in the provenance records.
Reviewed remains unchanged for every source.

## GLM-5.3 Coding Plan research

Z.AI's first-party public documentation now says GLM-5.3 is live for Coding
Plan Lite, Pro, and Max. The public contracts recorded in the provider research
note include:

- model `glm-5.3` and explicit `glm-5.3[1m]` selection;
- a 1,048,576-token context window and 131,072-token maximum output;
- Coding Plan reasoning mappings to `low`, `high`, and `max`;
- the documented Codex, Anthropic-compatible, and other OpenAI-compatible
  Coding Plan base URLs;
- Coding Plan input/cache/output credit multipliers; and
- documented automatic routing of older GLM-5.2 and GLM-5.1 requests to
  GLM-5.3.

The public model guide separately labels general GLM-5.3 API availability as
coming soon. The research note preserves that distinction instead of inferring
uniform availability across Z.AI products.

The fork vision says Z.AI GLM Coding Plan remains research only. Consequently,
this update adds no provider enum, login/logout command, credential store, model
catalog, usage endpoint, hosted tool, or product support claim. Existing README
text that said the removed Z.AI runtime was implemented and live-qualified was
corrected to match the standing policy and `docs/providers/README.md`.

## Grok behavior inventory

Grok's `eb267fef..d6a22a1a` synced-monorepo range changes preserved surfaces.
All applicable behavior remains adopt-by-default and is explicitly deferred:

| Obligation | Observable behavior and decision |
| --- | --- |
| `GB-D6A2-ACP-REASONING` | **temporarily deferred** — pass reasoning effort through ACP session/new and session/load metadata with provider-local validation. |
| `GB-D6A2-TUI-BIDI-STATUS` | **temporarily deferred** — logical Arabic/Persian bidi rendering and copy; preparation-tool names; queue/todo badge removal; and bounds-safe empty-list scrolling. |
| `GB-D6A2-CONFIG-HOME` | **temporarily deferred** — `GROK_CONFIG` / `GROK_CONFIG_PATH`, shared standard home lookup, and typed memory configuration. |
| `GB-D6A2-PERMISSIONS-MCP` | **temporarily deferred** — exact managed MCP matching, pre-session permission mode, dynamic edit classification, and IPv6 allow-entry validation. |
| `GB-D6A2-AGENT-SUBAGENT` | **temporarily deferred** — accurate hook denial, doom-loop threshold 64, closed-pipe/null-descriptor handling, and bounded subagent transcript/media eviction. |
| `GB-D6A2-GATEWAY-WORKSPACE` | **temporarily deferred** — computer-session git source, atomic embedded response creation, and publish-time sharing settings. |
| `GB-D6A2-TELEMETRY` | **temporarily deferred** — interactive/headless client mode and managed external OTEL mTLS with redaction. |
| `GB-D6A2-INTEGRATION` | **temporarily deferred** — crate graph, generated surfaces, CI, documentation, and cross-platform integration for the preceding groups. |
| `GB-D6A2-RELEASE` | **not applicable** — upstream source revision, release automation, and official installer ownership cannot replace fork-owned routes. |

Each open item has an owner, impact, blocker, 2026-08-22 deadline, acceptance
criteria, and intended tests in `fork/parity/current.json`.

## Closed Grok adoption: 8a14 preserved surfaces

Commit `08499fc6d94bc86df60e46a4084ab76f5013f823` lands the sixteen
preserved-surface behaviors carried from the `8a14c91d` snapshot, including
bounded replay and shutdown, durable session loading, worktree identity,
headless interaction, managed MCP cleanup, skill-path suggestions, workspace
`.envrc` deadlines, and truthful notification delivery.

Focused validation executed 83 tests with no failures: 13 pager session-load
barrier tests, 47 shell replay tests, 9 skill-path tests, 12 workspace `.envrc`
tests, and both opt-in real-PTY shutdown deadline tests. The composed pager also
passes `cargo check` and a direct production-binary build. `GB-8A14-LANDING`
therefore closes as **adopt**; no skipped PTY check is counted as evidence.

Commit `3c13d1526df91ce5220f3a1e7080feb24b3b4b58` adopts the focused
`75e73f3d` behaviors without replacing Enhanced's provider-isolated agent
startup or fork-owned release metadata. Dashboard worktree identity now uses
the session/probe OR semantics, Home/End follows logical wrapped lines, hook
write-deny enforcement and child-network launch policy are platform-gated, and
HEAD-to-working diff statistics include untracked files.

Validation passed 15 dashboard tests, the wrapped-input regression, the
untracked-file diff-stat regression, 6 hook write-deny tests, the Linux launch
guard test, and a full `x86_64-pc-windows-gnu` sandbox library check using an
unprivileged temporary MinGW toolchain. `GB-B13-WORKTREE`,
`GB-B13-PORTABILITY`, and `GB-B13-INPUT` therefore close as **adopt**.

Commit `52e4fbff` adopts the remaining `b13fa526` preserved-surface behavior.
Session titles are bounded, sanitized, pinned, resettable, persistent, and
version-gated across local and remote paths; abandoned persistent-agent boot
slots are generation-safe; runtime blocking pools are capped and prewarmed;
workspace RPC activity is classified and versioned; and ordered session,
task-result, status, summary, and interjection semantics are retained without
cross-provider credential fallback.

Focused validation passed 10 runtime tests, 97 workspace activity/version
tests, 68 shell title/boot/persistence/interjection/summary tests, 239 pager
title/session-event/task-result/turn-completion tests, and all 37 pager-bin
tests. Formatting, pager-bin checking, and strict manifest coverage also pass.
`GB-B13-TITLES`, `GB-B13-AGENT-BOOT`, `GB-B13-RUNTIME`,
`GB-B13-WORKSPACE-RPC`, `GB-B13-SESSION-EVENTS`, `GB-B13-WIRE`, and
`GB-B13-INTEGRATION` therefore close as **adopt**.

## Inspiration-source decisions

| Source | Decision |
| --- | --- |
| OpenAI Codex | The range covers persistent-exec pagination, workload identity, Guardian, hooks/plugins, environment ownership, TUI startup, gRPC, sandbox, and application architecture. No direct ChatGPT subscription wire-contract delta was found. **not applicable** as `CDX-C494-SCOPE`. |
| Kimi Code | Printing/copying the Kimi application's fork-resume command is outside the isolated API-key adapter. **not applicable** as `KIMI-6B72-RESUME`. |
| CodexBar | No new Z.AI usage contract appears; Codex, Cursor, Grok, widget, release, and UI changes remain outside its research role. **not applicable** as `CODEXBAR-F15F-RESEARCH`. |
| models.dev | Two NanoGPT rows name GLM-5.3 Preview and GLM-5.3 Preview Thinking with a one-million-token context. They corroborate research but are not authoritative provider contracts. **not applicable** as `MODELS-65DB-GLM53`; all other generated catalog changes are `MODELS-65DB-CATALOG`. |

## Durable ledger result

The campaign contains 92 stable obligations:

- 12 closed **adopt** items;
- 7 closed **already equivalent** items;
- 31 closed **not applicable** items; and
- 42 open **temporarily deferred** items.

All 78 prior IDs are carried forward. The open set contains 34 Grok adoption
obligations and 8 Codex adapter/harness obligations. No prior deferral vanished
or changed classification without closure evidence.

## Exhaustive raw evidence

`docs/upstream-refresh-2026-08-15-paths.json` records full old/new modes and
40-hex object IDs, status, path, stable obligation, and classification for each
raw row produced by `git diff --no-renames --raw --abbrev=40`.

| Source range | Raw rows | SHA-256 of exact raw stream |
| --- | ---: | --- |
| Grok `eb267fef..d6a22a1a` | 202 | `efab787aa57e26012468f412a0cd020053c1b6e9cca4b38c62b9ffa525f3d6e7` |
| Codex `6bed2134..c4941302` | 235 | `15ef3ad89f14fafbbd0f7217d15790961b48c1875538f762d62b5b704df1d446` |
| Kimi `d96cd037..6b72345f` | 5 | `2d8248157e91252afdd29a8927d9a4dfd7a83cb4019ee68a4db492c8ebf1fe0e` |
| CodexBar `24be9995..f15f142a` | 33 | `797bc6f78759d80f3992e1f2298c65617eb1af3d5c1ee72f6c50c2e4645f624e` |
| models.dev `a25d0e1f..65db1444` | 248 | `ad19a39f8a3a75cf551f75e143d50ddf26e3e3e26767f573903f4f31ccb24b5a` |

The sidecar has 723 rows total. Each row has exactly one stable obligation and
classification; no rename inference or abbreviated object identity is used.

## Acknowledgement and publication decision

Grok acknowledgement is ineligible because 34 Grok adoption obligations remain
open after this refresh. The publication workflow requires an eligible audited
acknowledgement marker whose first-parent tree is unchanged. There is therefore
no Reviewed-pin advancement, prepare step, marker merge, release tag, GitHub
release, Homebrew update, or downstream publication from this candidate.
