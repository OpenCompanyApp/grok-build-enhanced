# Upstream refresh audit — 2026-07-19

This document records the evidence and compatibility decisions for the source
revisions fetched on 2026-07-19. A fetched revision is not an accepted base, and
this audit does not merge or rebase the disconnected Grok Build snapshot.

## Immutable boundary

- Pre-refresh Enhanced commit: `78fb13142394eee3d27c51173ff7c6beaff64e24`
- Pre-refresh tree: `ff10b49cb3cf43af195db91b637a7ba4969a3cd4`
- Local snapshot ref: `refs/fork/snapshots/2026-07-19-pre-upstream-refresh`
- Grok Build fork baseline: `c1b5909ec707c069f1d21a93917af044e71da0d7`
  (tree `54a0606cc804815975eda042f60cb3320cfb5347`)
- Previously fetched Grok snapshot: `8adf9013a0929e5c7f1d4e849492d2387837a28d`
- Newly pinned Grok snapshot: `ba76b0a683fa52e4e60685017b85905451be17bc`
  (tree `705f35f80767b0c4d71340a8a59a74a38f5cc01c`)

The baseline and fetched Grok history have no usable merge base. Raw
`(mode,type,oid)` tree comparison found 642 baseline-to-pinned changed paths:
467 cleanly adoptable relative to the Enhanced tree, 11 already equivalent,
and 164 changed on both sides and requiring deliberate reconciliation.

## Pinned provider references

| Source | Reviewed before this audit | Pinned for this audit |
| --- | --- | --- |
| OpenAI Codex | `315195492c80fdade38e917c18f9584efd599304` | `3e2f79727a4e8ddfc8e3acb838d496b121094b9e` |
| OpenCode | `1d2a7b4c860f6a29eb90bdda07757b2adf34ab61` | `67caf894e0843ee370e72839e8265e483233479b` |
| Kimi Code | `ada523ae6a06726c02ffcc7f35d01ee2216d309c` | `df6899553962d1764c9f4c3bec1b63c811cb425e` |
| CodexBar | `e6b2ea490cec47d122ba14e32bfc41e53d98422c` | `f8636cb37eb0f96d261604ee94e6481496aadfeb` |

Warp themes, the OpenCode Codex-auth reference, Kimi CLI, the Z.AI SDK and
plugins, GLM-5, and the Z.AI usage helper did not advance.

## OpenAI Codex contract result

The complete 60-commit Codex range was reviewed across login and refresh,
catalog/cache metadata, Responses wire behavior, turn state and SSE, service
tiers/Fast, compaction, hosted web and image tools, usage, retries, and logout.

Confirmed compatibility work:

1. Make retries progress-aware: retry bounded pre-output failures, but never
   replay after visible text, reasoning, or tool-call output.
2. Preserve opaque `comp_hash` catalog metadata through model state and session
   resume, and compact on incompatible non-empty hashes.
3. Honor catalog reasoning-summary support and defaults instead of always
   sending `concise`.
4. Enforce the selected coding model's image-input capability on a cloned
   request view without mutating persisted history.
5. Correct stale GPT-5.6 Sol/Terra/Luna context snapshots from 372K to 272K.

The existing Enhanced contracts for provider-scoped auth, sensitive headers,
Responses routing, turn-state continuation, service tiers/Fast, hosted web and
image generation, usage, and logout remain applicable and must not fall through
to another provider.

## Kimi Code, OpenCode, and CodexBar result

The Kimi adapter remains equivalent or deliberately stricter for authenticated
catalog discovery, API-key header isolation, Chat and Messages schemas,
thinking effort, hosted search/fetch, usage, retry classification, and logout.
The exact final Kimi range from `4f3c7240c4adc7c748e536bf578e468c1b5bcd7b`
to `df6899553962d1764c9f4c3bec1b63c811cb425e` changes provider-relevant code
only to add effective thinking effort to Kimi's own turn telemetry. Enhanced
already records selected model/reasoning configuration at its own telemetry
boundary; Kimi's app-server and agent-engine telemetry architecture is not a
runtime adapter contract.

The exact final OpenCode range adds an OpenCode Go Responses route and changes
its OpenAI header-timeout default from 10 seconds to 300 seconds. Enhanced does
not use the OpenCode gateway; its direct Codex retry/idle policy is owned and
tested locally. The range does not establish Kimi or Z.AI adapter work.

No Kimi or Z.AI provider implementation changed in the pinned CodexBar range.
CodexBar remains a schema corroboration source only. Browser cookies, CLI/device
identity, arbitrary proxy endpoints, and non-redacted response-body logging are
not imported.

A future unknown Kimi thinking-effort string remains a watch item. The current
catalog's `none`, `minimal`, `low`, `medium`, `high`, `xhigh`, `max`, and
`ultra` values are preserved exactly; no speculative pass-through is added
without catalog evidence.

## Grok Build adoption decisions

### Adopted or selected for this refresh

- prompt/cancel mailbox ordering;
- commit-aware durable session append and acknowledgement boundaries;
- provider-safe ACP session state/import, with local credential bindings removed
  from exports and ignored on import;
- endpoint-scoped xAI bearer use and terminal failed-refresh behavior;
- shell redirect/source/environment/protected-target confirmation floors;
- repository-content trust gates and prompt-tag neutralization;
- plugin immutable revisions, managed-policy integrity, and Git operand checks;
- MCP OAuth issuer checks, strict qualified tool IDs, and owner-only credentials;
- sensitive-file permission hardening and privacy-safe defaults;
- bounded provider-error presentation without raw response-body logging;
- bounded subagent/session registries and cancellation cleanup;
- scheduler create/update schema compatibility while preserving legacy persisted
  one-shot tasks;
- truthful clipboard delivery/fallback behavior and terminal-native syntax
  polarity where they can be integrated without replacing Enhanced themes.

### Already equivalent or stronger

- Enhanced web-fetch DNS resolution, per-hop pinning, redirect revalidation, and
  private-address rejection;
- primary xAI auth storage permissions;
- disabled telemetry defaults;
- provider-hosted Kimi and Z.AI web tools remaining separate from local-network
  web-fetch permission;
- Kimi, Codex, Z.AI, xAI, and custom-provider credential identity remaining
  explicit.

### Parity correction and durable obligations

A 2026-07-20 review found that the previous section incorrectly combined
applicable Grok behavior, unsafe implementation mechanisms, and out-of-scope
reference architectures in one terminal "deferred" bucket. The omissions were
reviewed rather than accidental, but the first four decisions were inconsistent
with the fork vision: integration or hardening work can justify only a tracked
temporary deferral for behavior on a preserved Grok surface.

The following obligations were carried forward from Grok snapshot
`ba76b0a683fa52e4e60685017b85905451be17bc`. Their owner is the Grok Build
Enhanced maintainers. The correction work below closes them with durable commit
and test evidence that later refreshes must retain.

#### Implemented Grok adoption obligations

- **`GB-PARITY-001` — scheduler background loop-subagents**
  - **Status:** closed by `0ed211a` and `ce186c5`; adopted.
  - **Source:**
    `crates/codegen/xai-grok-tools/src/implementations/grok_build/scheduler/{actor,create,types}.rs`,
    `crates/codegen/xai-grok-shell/src/tools/notification_bridge.rs`, and
    `crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs` in the
    pinned Grok snapshot.
  - **Blocker and impact:** v0.2.4 has foreground task firing but lacks recurring
    background child execution, overlap prevention, bounded resume chains, and
    the foreground compatibility option.
  - **Closure:** schema/persistence migration landed in `0ed211a`; background
    execution, chain management, and client projection landed in `ce186c5`.
  - **Acceptance evidence:** provider/session-bound child creation, no-overlap
    queries, completed-child resume, ten-iteration and prompt-change rollover,
    4,000-character completion bounds, kill-switch behavior, persistence
    migration, and TUI/minimal/headless/ACP tests.
- **`GB-PARITY-002` — Stop/SubagentStop resampling gates**
  - **Status:** closed by `d819a8a`; adopted.
  - **Source:** `crates/codegen/xai-grok-hooks/src/{event,dispatcher}.rs`,
    `crates/codegen/xai-grok-shell/src/session/acp_session/hooks.rs`, and
    `crates/codegen/xai-grok-shell/src/session/acp_session_impl/{stop_gate,turn}.rs`
    in the pinned Grok snapshot.
  - **Blocker and impact:** v0.2.4 observes these events but cannot let a hook
    reject genuine completion and return feedback for another model round.
  - **Closure:** `d819a8a` integrates awaited parent/subagent completion gates,
    feedback continuation, force-stop, fail-open handling, and the eight-round cap.
  - **Acceptance evidence:** deterministic hook aggregation, explicit
    force-stop, failure-open timeout/error handling, an eight-round cap, correct
    parent/subagent boundaries, and tests proving no resample on cancellation,
    refusal, provider error, or max-turn termination.
- **`GB-PARITY-003` — active task and scheduler context in stop hooks**
  - **Status:** closed by `d819a8a`; adopted with Enhanced safety adaptation.
  - **Source:** `crates/codegen/xai-grok-hooks/src/event.rs`,
    `crates/codegen/xai-grok-tools/src/implementations/grok_build/{task,scheduler}/types.rs`,
    `crates/codegen/xai-grok-shell/src/tools/notification_bridge.rs`, and
    `crates/codegen/xai-grok-shell/src/session/acp_session/hooks.rs` in the
    pinned Grok snapshot.
  - **Blocker and impact:** external stop hooks lack upstream work context, but
    an unbounded raw prompt projection would create a credential and payload
    exposure risk.
  - **Closure:** `d819a8a` adds session-owned work descriptors, Unicode-safe
    field bounds, deterministic ordering, and a 64 KiB fail-open envelope.
  - **Acceptance evidence:** structurally secret-free descriptors, exclusion of
    provider-private turn state, Unicode-safe per-field clipping, an aggregate
    envelope limit, deterministic ordering, aligned client/file/HTTP payloads,
    and adversarial redaction and size tests.
- **`GB-PARITY-004` — minimal-mode `/btw`**
  - **Status:** closed by `67fc94b`; adopted.
  - **Source:** `crates/codegen/xai-grok-pager/src/minimal/api.rs`,
    `crates/codegen/xai-grok-pager-minimal/src/live.rs`,
    `crates/codegen/xai-grok-pager/src/app/dispatch/notes.rs`, and the minimal
    input, modal, geometry, and `/btw` correlation paths in the pinned Grok
    snapshot.
  - **Blocker and impact:** `/btw` exists in the full TUI, but minimal mode lacks
    its panel lifecycle, making the same Grok command mode-dependent.
  - **Closure:** `67fc94b` adds correlated requests, modal lifecycle, geometry,
    interaction, rendering tests, and native-scrollback persistence.
  - **Acceptance evidence:** request correlation, stale-response suppression,
    modal suspension/restoration, focus/scroll/link behavior, native-scrollback
    persistence, and deterministic narrow/normal-terminal rendering tests.

#### Adapted capability obligation

- **`GB-ADAPT-001` — xAI speech BYOK**
  - **Status:** closed by `744c053`, `2638a58`, and `9cbada3`; adopted without
    the upstream process-global key-publication mechanism.
  - **Source:** `crates/codegen/xai-grok-shell/src/auth/manager.rs` and
    `crates/codegen/xai-grok-pager/src/voice/auth.rs` in the pinned Grok
    snapshot.
  - **Blocker and impact:** process-wide publication of the first model-owned key
    is unsafe in a mixed-provider process, but an xAI user should still be able
    to use an eligible xAI credential for speech.
  - **Closure:** `744c053` adds connection-local model eligibility and canonical
    routing, `2638a58` binds each recording command to its model, and `9cbada3`
    exposes only qualified xAI session refresh to the pager.
  - **Acceptance evidence:** xAI-only, canonical-endpoint, session/model-scoped
    credential resolution with sensitive headers and tests for mixed providers,
    concurrent sessions, model switching, logout, and missing credentials.

#### Standing scope and safety decisions

- **`GB-SCOPE-001` — declarative sandbox website policy:** **not applicable as
  a security feature** while the upstream types have no runtime enforcement.
  Exposing inert protection would violate truthful security boundaries. Reopen
  only with platform enforcement and fail-closed tests, or for a narrowly
  documented wire/schema need that cannot be mistaken for enforcement.
- **`GB-SCOPE-002` — generic xAI billing/upsell wording:** **not applicable as
  written** under explicit provider identity. `2b3f282` retains xAI guidance
  only for positively identified xAI failures and adds negative Codex, Kimi,
  Z.AI, custom, unknown, and generic-provider coverage.
- **`CODEX-SCOPE-001` — Codex app-server architecture:** **not applicable**
  because Codex is a provider compatibility reference, not a replacement Grok
  application. Audio and realtime behavior must still be inspected separately
  on future refreshes; adopt it only if it maps to an existing Grok surface and
  an entitled subscription wire contract, not by importing the app server.
- **`KIMI-SCOPE-001` — Kimi app server, agent engine, and replacement TUI:**
  **not applicable** because they replace preserved Grok architecture. Kimi
  auth, catalog, streaming, hosted tools, usage, retries, and logout remain
  provider-adapter obligations.

#### Separate policy inconsistency

- **`ZAI-POLICY-001` — research-only policy versus runtime ownership:** open for
  an explicit maintainer decision. `AGENTS.md` says Z.AI remains research-only
  and must not establish runtime credentials or product claims, while
  `fork/manifest.json` owns runtime auth, provider, web/vision, and usage feature
  units for Z.AI Coding Plan. This correction does not silently choose either
  policy. Reconcile the root scope statement, actual runtime behavior, provider
  documentation, and manifest ownership in a separate reviewed change.

The immutable Grok baseline remains unchanged. The fetched snapshot remains a
review target rather than a replacement base. The closed items above document
both the v0.2.4 behavior gaps and the corrective implementation evidence; they
do not retroactively claim that path review alone established parity.

## Thematic implementation commits

The reviewed refresh was integrated as the following linear, focused series:

- `f90b01f` `chore(fork): record fetched upstream revisions`
- `39f5926` `fix(agent): order prompt and cancel dispatch`
- `4305006` `fix(auth): default coding retention to opt out`
- `60ddecf` `fix(codex): prevent retry replay after output`
- `f84b57d` `fix(mcp): harden auth and tool boundaries`
- `d211bee` `fix(security): harden repository trust boundaries`
- `c51ec11` `fix(codex): enforce coding model image input capability`
- `cf3109d` `feat(codex): persist catalog compatibility metadata`
- `edc4ec0` `fix(auth): isolate credential-bearing provider routes`
- `18e5f19` `fix(session): make persistence crash durable`
- `e12f8b3` `fix(permission): enforce managed execution floors`
- `35a9491` `fix(refresh): reconcile provider metadata fixtures`
- `9ae8e41` `fix(session): preserve queued updates after append failure`
- `cabf4b5` `fix(theme): preserve terminal syntax polarity`
- `45be8f6` `fix(clipboard): report retrievable copy delivery`
- `079ada7` `fix(scheduler): support in-place task updates`
- `2da9066` `fix(plugins): enforce immutable marketplace revisions`
- `4b41e6e` `fix(session): bound and clean up session registries`
- `f1a374a` `fix(security): harden managed policy integrity`
- `df02c7b` `fix(plugins): redact credentials from git diagnostics`
- `83700da` `test(clipboard): update welcome delivery fixtures`

The parity correction was implemented afterward as a separate focused series:

- `8a2e72f` `docs(fork): make upstream parity deferrals durable`
- `67fc94b` `feat(minimal): add correlated btw panel parity`
- `744c053` `feat(voice): scope xAI speech BYOK credentials`
- `2b3f282` `fix(errors): gate xAI guidance on provider identity`
- `2638a58` `fix(voice): carry model scope with recording commands`
- `0ed211a` `feat(scheduler): add background loop task schema`
- `ce186c5` `feat(scheduler): run recurring loops in background`
- `d819a8a` `feat(hooks): integrate bounded stop resampling gates`
- `9cbada3` `fix(voice): expose only qualified xAI session auth`
- `2230ec2` `fix(pager): close parity regression gaps`

The final plugin diagnostic follow-up is deliberately separate from the pinned
upstream port: it closes an Enhanced credential-safety gap where credentialed
Git URLs or subprocess stderr could otherwise reach logs and error displays.

## Validation recorded before publication

Every implementation unit passed formatting, diff hygiene, focused contract
tests, strict ownership, and a binary workspace check in its isolated worktree.
The integrated clipboard candidate additionally passed 57 renderer clipboard
tests, its telemetry serialization contract, and
`CARGO_INCREMENTAL=0 cargo check -p xai-grok-pager-bin`.

The candidate also passed:

- 63 fork-script unit tests;
- fork branding, provider, updater, workflow, secret, Codex-search, and Warp
  contracts;
- five release-pipeline tests and fifteen install-script tests;
- the `0.2.4` release contract for the exact four fork-owned native assets;
- the integrated managed-policy, marketplace, scheduler, clipboard, theme,
  registry-churn, provider-isolation, version, updater, and welcome suites;
- `CARGO_INCREMENTAL=0 cargo check --locked -p xai-grok-pager-bin`; and
- strict manifest ownership at `83700da` with all 860 downstream paths covered.

The integrated matrix found three stale welcome-view test fixtures still
passing the former clipboard boolean. Commit `83700da` updates only those
fixtures to the delivery-aware optional value. All 53 welcome tests and the
remaining binary and ownership gates then passed.

Safe credential-gated headless qualification used the version-stamped candidate
binary without printing credentials, account identifiers, authenticated
payloads, session identifiers, or raw headers:

- the authenticated Codex catalog exposed GPT-5.6 Luna, and a Luna turn in live
  search mode reached `EndTurn` with two correlated nonempty `web_search`
  results, no tool errors, and no credential-like captured material; and
- the authenticated Kimi catalog exposed K3, and a K3 high-reasoning turn
  reached `EndTurn` with one correlated nonempty hosted-search result and one
  correlated nonempty hosted-fetch result, no tool errors, and no
  credential-like captured material.

The tag-triggered release workflow must still verify the frozen source binding,
build all four native assets, assemble and check `SHA256SUMS` and
`RELEASE-PROVENANCE.json`, create GitHub OIDC attestations, and publish the
allowlisted release set.
