# Upstream refresh — 2026-08-14

This refresh starts from Enhanced commit
`3e4d24879687105c7d13fe5078aa3ccee2627980`, tree
`44c048f68d80360a64a492fb5448279d0e9a31df`, on the isolated
`refresh/upstreams-20260814` branch. The user's active
`agent/fix-settings-refcell-race` checkout and its modified/untracked branding
files were not changed, stashed, cleaned, or rebased. The earlier August 9 Grok
adoption worktree was also preserved as user-owned, unvalidated evidence.

The run fetches and freezes exact upstream identities, audits both the existing
review queue and the newly advancing ranges, adopts one bounded Codex wire
change, and carries every remaining obligation forward. It does not advance a
Reviewed revision, create an upstream acknowledgement, or authorize
publication.

## Frozen source heads

| Source | Frozen commit | Tree / availability | Change from prior fetched pin |
| --- | --- | --- | --- |
| Grok Build | `eb267feff13129e568df38fb6fdf0ceb65f735d6` | `eaa84c2e1e8b7792eee8fa8c13ddeffee6aa38d6` | 2 commits / 594 paths |
| OpenAI Codex | `6bed213411d1250686cc162feb4324975c12158a` | `94324a0a8a4db4781665150a5abc5b34baed524f` | 129 commits / 706 paths |
| OpenCode | `4643e65ad6334de3e4e68dedc201d5fbb828c9fe` | `f83d19cb1d169c4aed87bd72ade3ba845e2a4656` | 25 commits / 89 paths |
| OpenCode Codex auth | `bec2ad69b252ef4ad7dd33b9532ff8b4fdb6d016` | `1da59bae7069563b2817143567b57c78e5758300` | unchanged |
| Oh My Pi | `ffd53ff92a6f575d499730475a73460dd7cc2eea` | `2ebad34cc83edeb5a0dc41d8ed1344eef442aa11` | 365 commits / 728 paths |
| Warp themes | `82e51dcf9b47912d551107748ba3297a21b2eff3` | `2893387a4769db78ce4ef5294b8cc39bacd80616` | unchanged |
| Kimi Code | `d96cd037702637305422222e985139e51ff83c8c` | `09d266e8e05b4f1b377eb390c8aa18ef2b42e7e1` | 33 commits / 566 paths |
| Kimi CLI | `cbc15c076d17f70fec9f89c90c0502e68657f505` | `e1d6d5b2827f8a14c2edc4bc8658ad5cf19d52e7` | unchanged |
| Z.AI Python SDK | `ca5109c0aa9bf173839be391b4b14aeadf9a9bf9` | tracked URL returned `Repository not found` | retained immutable pin |
| Z.AI coding plugins | `0446d0bb0bc537d97d3ab3664c4b8b9c4a0e1254` | `efea84479dc67bc4af7d2c3b59b4aca8f5332899` | unchanged |
| GLM-5 | `25206af860c4ac10f6411c597c574f9b1c00e53c` | `573d8342bcfc2e21d27e210c47a99a4604fc39ee` | unchanged |
| CodexBar | `24be9995fb8b182ead850f1afba3c2806085bb52` | `725d28e3568fed984e081fe9328b31e2bac959a8` | 225 commits / 192 paths |
| Z.AI usage helper | `54cd1f33a703c417f2492ee1f21f22b3633a43c4` | `08b00849b96c5883a265f4d4d43e2836d01cdd9d` | unchanged |
| models.dev | `a25d0e1f35368f685476e30fef5101d00801fc53` | `068e8b9e429724457532ae5d1c7d444439b2528c` | 139 commits / 493 paths |
| Exa MCP server | `e64c11f2d3b4400ffbda8ccdd9658a450cc9d270` | `569db78ece8c6a13f6f4afeefe05e569a57cb09e` | unchanged |

Every available new pin is a descendant of the preceding fetched pin and its
Reviewed pin. Fetching did not move any inspiration checkout. The first fetch
attempt used the isolated worktree's absent ignored `inspiration/` directory;
it changed no state and was repeated against the established absolute ignored
checkouts. The unavailable Z.AI SDK identity was not replaced by inference.

Only **Latest fetched** and the check date move in the provenance records.
Reviewed remains `afbc0fb7` for Grok, `8e4b1044` for Codex, `284214c7` for
OpenCode, and the previously recorded revisions for every other source.

## Review-queue continuity

The required Reviewed-to-previous-fetched audit is retained and rechecked by
reference rather than silently collapsed:

- Grok `afbc0fb7..be713136` is covered by the August 11 and August 12 ledgers,
  including the exhaustive 238-path and 211-path raw audits and all durable
  `GB-8A14-*`, `GB-B13-*`, and `GB-BE71-*` obligations.
- Codex, OpenCode, Kimi, Oh My Pi, CodexBar, and models.dev reviewed-to-prior
  ranges are covered by the August 7, August 11, August 12, and August 12 rerun
  ledgers. No prior open ID has disappeared or been reclassified without
  closure evidence.
- The August 9 implementation candidate remains mixed staged/unstaged state in
  its separate worktree. It is evidence for `GB-8A14-LANDING`, not a reviewed
  implementation boundary, and this run does not copy or overwrite it.

## Grok behavior inventory

Grok's `e5fd4816..eb267fef` range changes two release snapshots (1.0.2 and
1.0.3). Observable behavior on preserved surfaces remains adopt-by-default.
The raw-path sidecar assigns all 594 paths exactly once; these thematic rows
separately inventory the behavior rather than treating path ownership as parity.

| Obligation | Observable behavior and decision |
| --- | --- |
| `GB-EB26-SESSION-IMAGE` | **temporarily deferred** — resolve existing relative markdown links from session cwd; heal and persist stripped invalid images; capture and persist bound image capabilities/conversation metadata; and share image byte budgets with compaction. |
| `GB-EB26-AGENT-QUEUE` | **temporarily deferred** — terminalize and wait for stale live children; wake the model after a UI kill that has no tool result; make subagent overlay stop and Ctrl-C cancel/reap children; use a default 32-turn doom-loop limit; prevent queue promotion during row editing; frame post-cancel follow-ups as interjections; show subagent name/count wait activity and allow parallel prompt writes; deliver queued messages immediately through interjection; and preserve late lifecycle events. |
| `GB-EB26-HOOKS` | **temporarily deferred** — display the first stderr diagnostic, propagate `updatedInput` from PreToolUse, and allow a hook to stop the user turn. |
| `GB-EB26-PERMISSIONS-AUTH` | **temporarily deferred** — poison failed auth attempts across straddled token consumption, honor explicit narrow automatic grants, and include user intent in Auto-mode security context. |
| `GB-EB26-TUI-INPUT` | **temporarily deferred** — accept Flameshot/file image pastes and unfocused paste/drag input; retain startup typeahead; use soft-wrap-aware word selection and paragraph triple-click; and support queued-row pager commands. |
| `GB-EB26-TUI-STATUS` | **temporarily deferred** — keep legacy Ctrl+4 dashboard access; apply automatic display-refresh cadence; make `/session-info` copyable and draggable; report tool-call writing activity; expose free-tier and SuperGrok Plus upgrade UI including Apple Terminal Ctrl+O behavior; disable session search when configured; and present the bundled grok-4.6 default. |
| `GB-EB26-HARNESS` | **temporarily deferred** — skip live-subagent session-root scans; preserve plan drafts through approval; record scheduled-task silent expiry as a typed transcript reason; add browser-verification, communication, UI-verification, and project-rule guidance; retire Direct Mode documentation; bind client filesystem operations to session cwd; and preserve framed cancellation continuation. |
| `GB-EB26-WORKSPACE` | **temporarily deferred** — walk provisioned mounts for prompt discovery, graphing, and fsnotify; make boundary commits for dirty trees; and add EnsureBinding, MergeToMain, Push, and start-from-bindings workspace operations with bound cwd behavior. |
| `GB-EB26-TOOLS-MCP` | **temporarily deferred** — export `GROK_SESSION_ID` to tools/MCP without credentials; kill and reap ripgrep on cancellation; adopt MCP protocol 2025-11-25; echo bound tool overrides; configure allowed/excluded web-search domains; and suppress inline citations for backend-hosted search. |
| `GB-EB26-PROVIDER-XAI` | **temporarily deferred** — unify PromptMetadata features, key Responses caches by conversation, attach image-capability metadata, integrate grok-4.6 catalog defaults, and handle xAI subscription/up-sell state without crossing into Codex, Kimi, or custom-provider identity. |
| `GB-EB26-INTEGRATION` | **temporarily deferred** — integrate the crate graph, locks, generated manifests, tests, docs, feature wiring, and Windows `USERPROFILE` Grok-home behavior needed by the preceding groups. |
| `GB-EB26-RELEASE` | **not applicable** — upstream `SOURCE_REV`, official xAI versioning, installers, and release metadata cannot replace fork-owned update and release routes. |

The eleven new Grok deferrals have owner, blocker, impact, deadline,
acceptance criteria, and intended tests in `fork/parity/current.json`. Their
2026-08-21 deadline keeps them open for the next adoption campaign; it is not a
terminal scope decision.

## OpenAI Codex provider matrix

| Upstream behavior | Local behavior | Action / evidence |
| --- | --- | --- |
| Login, refresh, account state, token storage, and logout | No changed direct-subscription auth contract in this range; Enhanced retains account-generation and provider-qualified credential isolation. | Existing obligations and negative provider-isolation tests remain controlling; no auth material was read or logged. |
| Enable `parallel_tool_calls` for every model prompt, independent of catalog flags (`86b1123`) | Responses Lite forced `false`; the Grok agent already supports parallel dispatch. | **adopted** as `CDX-6BED-PARALLEL`: the final Codex-only JSON boundary now sends `true` for Lite and non-Lite Codex requests, while xAI/custom requests remain unchanged. Focused tests cover the rewrite and provider scope. |
| Move WebSocket model ETag metadata and retain conditional model caching (`8bb8d602`) | Enhanced uses authenticated HTTP discovery rather than Codex WebSockets, but already stores ETags, sends conditional fetches, reuses 304 caches, and renews only the matching live account generation. | **already equivalent** as `CDX-6BED-ETAG`; catalog ETag/304 and credential-generation tests are closure evidence. |
| Add authoritative per-thread usage query totals, credits/USD, and model/reasoning/speed groups (`842fae26`, `f1a1fce2`, `1e71e35d`) | Enhanced has provider-qualified local token accounting and estimated API pricing, but no authoritative subscription thread-credit query. | **temporarily deferred** as `CDX-6BED-THREAD-USAGE`; requires account/thread isolation, redaction, unsupported-response handling, and explicit authoritative-versus-estimated labeling. |
| Add model retirement/upgrade times and catalog-supplied multi-agent instructions (`5cc65ecb`, `395723b2`) | Authenticated model discovery lacks these optional fields; the Grok harness owns agent behavior. | **temporarily deferred** as `CDX-6BED-CATALOG`; adapt optional metadata into the provider-isolated catalog and apply instructions only on Codex turns. |
| Client-authored developer metadata, creation/world-state history, interrupted recovery, and compaction/history projection | The range extends work already represented by the open metadata and harness-history campaigns. | Paths remain assigned to `CDX-F2A6-METADATA` and `CDX-D6EE-HARNESS-HISTORY`; stable IDs and deadlines carry forward. |
| Service tier/Fast, Responses Lite URL/body, visibility and continuation state, `comp_hash`, hosted web/image tools, SSE/errors, retry/idempotency/timeout/cancellation/auth recovery | No additional applicable wire-contract change was found beyond the rows above. | Existing implementation/tests remain controlling; no false closure was added. |
| Guardian V2, app server/TUI, gRPC code mode, Bedrock, workload identity, Windows sandbox, plugin analytics, and SDK architecture | Enhanced does not import replacement application architecture. No changed behavior in these paths maps to an unrepresented existing Grok or direct-provider surface. | **not applicable** as `CDX-6BED-SCOPE`; all 520 assigned raw paths remain auditable in the sidecar. |

## Kimi and interoperability matrices

| Source | Upstream behavior -> local behavior -> decision |
| --- | --- |
| Kimi Code | Agent-core-v2 RPC/swarm/session refactors, declarative subagent pools, auto-title, OAuth application flows, MCP auth state, KAP marketplace/capability/task fields, TUI, and generated web assets do not change the scoped API-key adapter. Authenticated catalog, headers, request/stream schema, thinking effort, hosted web search/fetch, usage, retry/error ownership, and logout have no new contract delta. **not applicable** as `KIMI-D96C-SCOPE`; existing Kimi isolation evidence remains controlling. |
| OpenCode | xAI reasoning effort now accepts string levels and passes `xhigh`; Enhanced already maps xAI Max to wire `xhigh` and rejects Codex-only Ultra for xAI. **already equivalent** as `OC-4643-XAI-REASONING`. OpenCode Go web search is already served by provider-independent Exa with fail-closed domain filtering, **already equivalent** as `OC-4643-GO-WEB`. Kimi prompt replacement, Grok endpoint docs, generated catalogs, UI, release, and replacement-harness changes are **not applicable** as `OC-4643-SCOPE`. |
| Oh My Pi | `x-codex-beta-features: remote_compaction_v2` belongs to Oh My Pi's explicit V2 endpoint and is non-normative, so it establishes no Enhanced Codex contract (`OMP-FFD5-CODEX-REF`). Static aliases, cache/usage behavior, hosted-search/PDF/LSP/DAP features, tests, TUI, and replacement harness remain evaluation material only (`OMP-FFD5-HARNESS`). Both are **not applicable**. |
| CodexBar | No Z.AI/GLM usage-protocol change appears in the advancing range; Codex, Claude, Grok, OpenCode, and UI work is outside CodexBar's tracked Z.AI research role. **not applicable** as `CODEXBAR-24BE-RESEARCH`. |
| models.dev | New/changed Kimi K3, Grok 4.6, GLM 5.3 coding-plan, and other third-party catalog rows are research hints. Authenticated first-party catalogs remain authoritative and no Z.AI runtime provider is established. **not applicable** as `MODELS-A25D-CATALOG`. |

OpenCode Codex auth, Warp themes, Kimi CLI, Z.AI coding plugins, GLM-5,
Z.AI usage helper, and Exa did not advance. Their prior attestations remain
valid. The unavailable Z.AI SDK pin likewise creates no runtime provider,
login command, credential, or product claim.

## Durable ledger result

The current campaign contains 78 stable obligations:

- 1 closed **adopt** item;
- 7 closed **already equivalent** items;
- 25 closed **not applicable** items; and
- 45 open **temporarily deferred** items.

All 53 prior IDs are carried forward. The open set includes 37 Grok adoption
obligations and 8 Codex adapter/harness obligations. The earlier Grok candidate
was not duplicated into this audit branch because its user-owned mixed state is
not a validated implementation boundary; the new 594-path Grok range also
depends on that campaign. The bounded parallel-call change is independently
owned and tested here. Thread usage and new catalog metadata remain explicit,
reviewable Codex work rather than being hidden as scope exclusions.

## Exhaustive raw evidence

`docs/upstream-refresh-2026-08-14-paths.json` records full old/new modes and
40-hex object IDs, status, path, stable obligation, and classification for each
raw row produced by `git diff --no-renames --raw --abbrev=40`. The evidence is
owned by the focused `august14-refresh-audit` manifest feature unit.

| Source range | Raw rows | SHA-256 of exact raw stream |
| --- | ---: | --- |
| Grok `be713136..eb267fef` | 594 | `e7876fee44cda60e527fb60c5fd93f8ad32d6c6404c90fad9b89553ce180d2d1` |
| Codex `3d7bb2dd..6bed2134` | 706 | `5a7ddc99dfafb17d72a86e6a0761ab7047bfc9cd6f0b9c9af0c54826956b0924` |
| OpenCode `dab26372..4643e65a` | 89 | `b290a2e475c803609942091e8247cb54c34870f50912d1b8c297bf74914bb0af` |
| Oh My Pi `06aecdd5..ffd53ff9` | 728 | `50d0a2f0f7a6d8a8ac1d382928d0a96b07bc0a18f72fe645f5e664d56b1ceadf` |
| Kimi `719da946..d96cd037` | 566 | `0e62060e8358097b9f73d7e4a35422d9989d42ba54635652e0d1b72666ab8247` |
| CodexBar `ee29794b..24be9995` | 192 | `745ac479411b97f1c3e2d5de63d0f99e203c940892926ea61cf3cbe01c83b04c` |
| models.dev `0370588c..a25d0e1f` | 493 | `c121cbad4b8702621a09ad12c70ebf8160ecb64113265f9ed4b32cbf011aac9f` |

The sidecar has 3,368 rows total. Each row has exactly one outcome and a stable
obligation ID; no rename inference or abbreviated object identity is used.

## Acknowledgement and publication decision

Grok acknowledgement is ineligible because 37 Grok adoption obligations are
open. There is therefore no acknowledgement declaration, prepare step, marker
merge, or Reviewed-pin advancement. No release version or publication
authorization was provided, so no push, tag, release, Homebrew, or pull-request
state is changed.

## Validation

The isolated candidate passed:

- `cargo fmt` and `git diff --check`;
- all 9 focused `xai-grok-sampler` Codex Responses unit tests;
- `CARGO_INCREMENTAL=0 cargo check -p xai-grok-pager-bin`;
- all 51 `test_check_manifest.py` tests;
- JSON/schema and durable-obligation consistency checks; and
- independent reproduction of all 3,368 raw sidecar rows and seven stream
  digests from the pinned Git objects.

The binary check emitted only the existing warning that pager-render's
`build.rs` appears as both its build script and the
`warp_vendor_build_validation` integration-test target. The committed audit
candidate `e294761596f9c097b09219fcacc27617659bb879` also passed
`check_manifest.py --strict-coverage`: all 110 feature path sets and all
2,219 baseline-to-candidate downstream paths were owned, all source records
cross-checked, and the 78-item current ledger retained 45 explicit open items.
No live-provider test is attempted because this audit neither needs nor
authorizes access to entitled credentials or authenticated payloads.
