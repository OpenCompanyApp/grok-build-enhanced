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

### Deliberately deferred

- Grok scheduler background loop-subagents: this requires provider/model/binding
  inheritance and parent-switch isolation coverage before it can replace the
  existing foreground fire path.
- Stop/SubagentStop resampling gates and full task/scheduler prompt projection:
  this introduces a new external hook data surface. It needs a separate
  credential-redaction and request-size design before adoption.
- Minimal-mode `/btw`: this is a separate product feature cluster, not a
  compatibility requirement for the audited provider and security contracts.
- Declarative sandbox network-policy types without runtime enforcement.
- Process-wide publication of a model-owned key for xAI speech services.
- xAI billing/upsell wording on generic provider errors.
- Codex audio/realtime/app-server and Kimi app-server/agent-engine/TUI behavior.

These deferrals are explicit compatibility decisions, not unreviewed omissions.
The immutable Grok baseline remains unchanged; the fetched snapshot is recorded
as an audited source target rather than a replacement base.

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
- five release-pipeline tests and fifteen install-script tests; and
- the `0.2.4` release contract for the exact four fork-owned native assets.

Final integrated provider-isolation tests, focused suites, full binary check,
strict manifest check, safe credential-gated smoke tests, and release workflow
asset/attestation verification are required immediately before and after
publication. No validation may print credentials, account identifiers,
authenticated payloads, or raw credential headers.
