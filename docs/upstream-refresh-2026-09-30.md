# Upstream refresh — 2026-09-30

Status: source fetch and scoped model audit completed; full behavior adoption
remains open. This record is not acknowledgement evidence.

A [subsequent reference audit](upstream-refresh-2026-09-30-follow-up.md) closes
the Exa review after focused tests and records bounded terminal-theme findings.
The original fetch-time tables below remain historical evidence.

## Boundary and preservation

Work is isolated on `release/models-20260930`, based on mainline
`c162abc66648ead54a3da233db523c692d0a5f5d` (tree
`f999ffdf9cdd516de3d39987d27ec9d873359c74`). The user's original checkout,
tracked changes, and untracked assets remain untouched. No upstream tree has
been merged or rebased. The September 29 checkpoint at
`e91d0f4f328f371e04cd64df95db75edf4f853b7` remains intact; its three evidence
files and focused ownership entry are carried into this branch.

## Fetch evidence

[Exact commit/tree pins and raw-diff hashes](upstream-refresh-2026-09-30-pins.json)
record each successful fetch. Source heads are frozen for this run; later remote
advances belong to another refresh. The tracked Z.AI SDK URL remains unavailable
(`Repository not found`); its old pin is retained without inferring a replacement.

| Source | Paths changed since Reviewed | Paths changed since September 29 |
| --- | ---: | ---: |
| openai-codex | 6102 | 810 |
| opencode | 517 | 21 |
| grok-build-upstream | 4213 | 1028 |
| opencode-codex-auth | 0 | 0 |
| oh-my-pi | 6984 | 925 |
| warp-themes | 0 | 0 |
| ghostty | 239 | 48 |
| herdr | 1432 | 257 |
| kimi-code | 3364 | 130 |
| kimi-cli | 39 | 0 |
| zai-sdk-python | unavailable | unavailable |
| zai-coding-plugins | 0 | 0 |
| glm-5 | 0 | 0 |
| codexbar | 2547 | 87 |
| zai-usage-helper | 0 | 0 |
| models-dev | 4365 | 343 |
| exa-mcp-server | 4 | 0 |

The GLM Reviewed pin carries forward the completed September 29 two-path review.
Unchanged reviewed source heads attest that no new source audit is needed for
those ranges. The Exa head is unchanged from the bounded September 29 inspection;
its Reviewed pin remains conservative. Other advancing sources remain review
queues; fetching does not establish parity.

## Grok raw inventory

[Raw tree entries](upstream-refresh-2026-09-30-grok-paths.json) compare literal
mode/type/object identities, without rename inference, in both required ranges
and in the full reviewed-to-target range:

- `07b2f7144fd5c5c9d3dd1966937a87852d2dbdb8` → `f0e3be1100ef5252488e3be8bb0e91cf68d8c305`: 3,828 paths; 2,019 match existing ownership units.
- `f0e3be1100ef5252488e3be8bb0e91cf68d8c305` → `2bdd1d6a6369de0e8c68132ea4539e9abd9e14a8`: 1,028 paths; 442 match existing ownership units.
- `07b2f7144fd5c5c9d3dd1966937a87852d2dbdb8` → `2bdd1d6a6369de0e8c68132ea4539e9abd9e14a8`: 4,213 paths; 2,032 match existing ownership units.

This is an exhaustive raw-path inventory, not an exhaustive behavior ledger.
Empty owner sets identify paths requiring focused ownership if adopted; they are
not scope exclusions. The new Grok range alone changes sessions, ACP, lifecycle,
TUI, tools and generated crates and cannot be marked Reviewed from a model audit.
No newly fetched Grok snapshot is acknowledged.

## Scoped model behavior ledger

The [model audit](providers/model-catalog-2026-09-30.md) records exact primary
sources, provider identity, and intended regression evidence:

| ID | Outcome | Behavior/action |
| --- | --- | --- |
| MODEL-20260930-CODEX-DISCOVERY | adopt | Meet current public catalog minimum client versions so authenticated catalogs can return Astra, Sol 6.1, Sol and Luna. Preserve dynamic entitlement and metadata. |
| MODEL-20260930-XAI47 | adopt | Add documented Grok 4.7 and update bundled defaults while preserving legacy IDs and remote/user precedence. |
| MODEL-20260930-GLM53-FLASH | already equivalent | Existing Coding Plan Flash catalog, image-input wire shape and stale-cache precedence cover the requested model. |
| MODEL-20260930-GLM53-FLASHX | not applicable | Official Z.AI docs exclude FlashX from the Coding Plan. Never route a subscription key to a pay-as-you-go endpoint. |
| MODEL-20260930-KIMI | already equivalent | Authenticated catalog accepts all four documented Kimi IDs, including K2.8 under its existing alias, with provider-authoritative context and thinking metadata. |

The model-specific outcomes do not close broader provider or Grok obligations.
Runtime test results are recorded in the model audit after execution.

## Carried open obligations

`fork/parity/current.json` is unchanged; existing owners, acceptance criteria,
blockers, test plans and deadlines remain visible. These IDs remain open:

- `CDX-E482-CONTEXT-WINDOW-ID`
- `GB-BC7F-SESSION-ACP`
- `GB-BC7F-AGENT-LIFECYCLE`
- `GB-BC7F-TUI-INPUT`
- `GB-BC7F-WORKSPACE`
- `GB-BC7F-INTEGRATION`
- `CDX-6478-AUTH-CATALOG-WIRE`
- `CDX-6478-APP-BEHAVIOR`
- `OC-DC44-CODEX-INTEROP`
- `KIMI-9D23-PROVIDER`
- `OMP-33CC-HARNESS`
- `CODEXBAR-CE44-ZAI-USAGE`

`ZAI-414A-RUNTIME` remains offline-qualified rather than fully live-qualified.
No deferred behavior is silently reclassified as out of scope. Full upstream
review, thematic adoption and closure tests remain required for acknowledgement.

## Validation and publication

The user authorized a model update and Homebrew release and separately requested
this refresh. A model release must be described as a downstream model update,
not a completed upstream refresh. No publication or new acknowledgement is claimed
by this document. The focused model publication is separate from the refresh skill's
acknowledgement/publication stages; those full-refresh gates remain unmet.

Homebrew's official ARM64 container solves the absent-host-Homebrew environment:
v0.3.18 installed; `brew style`, `brew audit --strict --online`, `brew test`,
`brew list --versions`, `grok version` and `agent version` passed. Those baseline
checks are not a substitute for repeating them against a new formula and upgrade.

Validation passed: 154 focused Rust tests (model/version, catalog, Codex and Z.AI
transport, and credential rejection), binary `cargo check`, formatting, strict
ownership, fork/generated-workspace contracts, five release-pipeline tests,
fifteen installer tests, and the 343-theme vendor lock. The release contract
accepts version 0.3.19. The subsequent user clarification selects the focused
model release and restricts Z.AI additions to Coding Plan availability.
Implementation is recorded in `d88a666f1ac19f0ebc26b6b6245b25d7109167ed`;
fetch evidence is separate in `3c948d7a`. No authenticated provider payloads
or credentials were read or recorded.

At this validation checkpoint the model candidate is local; no new
acknowledgement, tag, project push, release, or tap mutation has occurred. This run's Cargo target directory is
removed after validation; the isolated Homebrew baseline is retained for the
pending installed upgrade check.
