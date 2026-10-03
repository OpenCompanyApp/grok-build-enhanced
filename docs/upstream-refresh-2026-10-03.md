# Upstream refresh — 2026-10-03

Status: fetch and scoped compatibility review completed; full adoption remains
open. Release version **0.3.20** was explicitly confirmed for this run.

## Boundary and pins

Work is isolated on `fix/xai-client-version-20261003`, based on release main
`b499fb7a28f2327c6ffea390ab33f77c57a6d7a0`. The fresh source audit starts at
`4eab8cc3bd980a434d1331b8378cf15fb690c7bf` after the independently committed
xAI compatibility fix. Original user-owned changes remain untouched.

[Exact source pins and raw-diff hashes](upstream-refresh-2026-10-03-pins.json)
freeze this run. Sixteen fetches succeeded; the Z.AI SDK URL still returns
Repository not found, so its previous pin is retained. Only Latest fetched
advances; no Reviewed revision or upstream acknowledgement is advanced.

| Source | Paths since Reviewed | Paths since September 30 |
| --- | ---: | ---: |
| openai-codex | 6399 | 1394 |
| opencode | 556 | 127 |
| grok-build-upstream | 4213 | 0 |
| opencode-codex-auth | 0 | 0 |
| oh-my-pi | 7127 | 1004 |
| warp-themes | 0 | 0 |
| ghostty | 253 | 29 |
| herdr | 1437 | 41 |
| kimi-code | 3364 | 0 |
| kimi-cli | 39 | 0 |
| zai-sdk-python | unavailable | unavailable |
| zai-coding-plugins | 0 | 0 |
| glm-5 | 0 | 0 |
| codexbar | 2686 | 444 |
| zai-usage-helper | 0 | 0 |
| models-dev | 4533 | 326 |
| exa-mcp-server | 0 | 0 |

Grok's commit and tree are unchanged. Its exhaustive September 30
[raw-path inventory](upstream-refresh-2026-09-30-grok-paths.json) remains the
same reviewed-to-target evidence; the latest-to-new range has zero paths.
This does not close the existing Grok behavior deferrals. No unreviewed tree
has been merged, rebased, or acknowledged.

## Scoped behavior review

| Source / behavior | Classification and evidence |
| --- | --- |
| Grok xAI request version gate | **adopt**: [compatibility repair](providers/xai-version-gate-2026-10-03.md) separates the required upstream 1.0.13 wire floor from Enhanced 0.3.x releases; covers inference and auxiliary calls, mock 426 regression, and provider isolation. |
| Codex catalog minimum versions | **already equivalent** for this new range: `codex-rs/models-manager/models.json` is unchanged; the existing 0.155.0 catalog compatibility retains the September 30 model audit. |
| Codex API-key catalog opt-out | **not applicable** to the explicit subscription adapter: `models-manager/src/manager.rs` now hides previously fetched API-key catalogs after opt-out. Enhanced subscription discovery does not accept API-key credentials. |
| Codex Bedrock service tiers | **not applicable** to this provider adapter: `core/src/client.rs` now selects only Bedrock-advertised tiers. Enhanced Codex subscription identity cannot become Bedrock. |
| Codex Responses Lite empty tools | **temporarily deferred**, carried under `CDX-6478-AUTH-CATALOG-WIRE`: `core/src/client.rs` omits the empty `additional_tools` item; Enhanced still emits it for tool-free auxiliary calls. Required parity test: empty tools omit the item while nonempty tools and instructions retain ordering; review together with existing Lite envelope obligations. |
| Codex failed-event retry advice | **temporarily deferred**, carried under `CDX-6478-AUTH-CATALOG-WIRE`: `codex-api/src/sse/responses_error.rs` now reads structured retry headers, prefers them to rate-limit text, and safely parses floating delays. Required work: map only bounded safe retry advice into Enhanced errors, preserving no-retry-after-visible-output and credential redaction; add malformed/overflow/header-precedence and partial-output tests. |
| OpenCode interoperability | **already equivalent for the new provider/auth range**: the 127 changed paths include stats model UI, but no provider/auth integration paths; prior Reviewed-to-target obligation `OC-DC44-CODEX-INTEROP` remains open. |
| Kimi and Z.AI/GLM | **already equivalent for newly fetched ranges**: Kimi Code/legacy, Z.AI coding plugins, GLM and usage helper heads are unchanged. No new provider runtime contract is inferred from the unavailable SDK. |
| Warp | **already equivalent**: vendor source head is unchanged. |
| Ghostty theme contract | **not applicable** changes in emulator internals: OSC 105 parsing and mouse-shape reset handling; Enhanced's theme sync emits/query-colors rather than implementing a terminal emulator. No new theme-sync endpoint/config schema appeared in the changed paths. |
| Herdr pane/theme contract | **already equivalent for inspected contract surfaces**: pane initial-color acquisition defers the first render snapshot, with upstream tests preserving baseline colors and OSC 10/11 foreground/background. Other changes concern pane/PTY lifecycle, allocation and agent detection inside the external multiplexer, outside this fork's theme adapter. |
| CodexBar Z.AI usage | **already equivalent for the new Z.AI-specific range**: no changed path names under Z.AI/GLM provider surfaces; earlier `CODEXBAR-CE44-ZAI-USAGE` audit remains open. |
| models.dev | **not applicable as entitlement authority**: new hosted-provider model entries and metadata are research references; subscription catalogs remain provider-authoritative. The new Grok Imagine Video 1.5 Lite catalog entry does not establish a Coding Plan model ID. |
| Oh My Pi | Non-normative evaluation only: tool parsing now preserves parse errors instead of treating malformed JSON as lenient arguments, carries additional post-tool context, and drops malformed Codex replay calls. Keep `OMP-33CC-HARNESS` open for harness evaluation; do not import its agent engine. |

The Codex findings above extend the existing wire obligation, owned by fork
maintainers, with the exact source pin and paths in the evidence file. The
blocker is coordinated validation of Lite auxiliary calls and retry safety at
the stream/output boundary; target is the next provider-refresh milestone.
Acceptance requires the listed tests and unchanged provider isolation. These
are open work, not completed parity or terminal scope exclusions.

The rest of the 1,394-path Codex, 1,004-path harness and earlier source queues
are not represented as exhaustively behavior-reviewed by this scoped pass.
Reviewed stays conservative. The focused xAI repair is the release scope.

## Carried obligations

All prior open IDs remain in `fork/parity/current.json`:

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

No ID is removed or implicitly closed by a newer fetch. No new acknowledgement
marker is permitted while the Grok deferrals remain open.

## Validation and publication

The compatibility audit records focused test results. Final release validation
includes strict first-parent ownership, provider isolation, binary compilation,
release contracts, all native assets with checksums/provenance/attestations,
and Homebrew style, strict online audit, formula test and installed upgrade.
