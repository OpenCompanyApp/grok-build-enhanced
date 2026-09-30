# Model catalog audit — 2026-09-30

This is a scoped model-discovery update, not completion of the upstream refresh.
The outstanding Grok and provider obligations in `fork/parity/current.json`
remain open. The [refresh record](../upstream-refresh-2026-09-30.md) separately
tracks fetched snapshots and the remaining behavior audit.

## Model inventory and changes

| Provider | Current coding models checked | Local result |
| --- | --- | --- |
| ChatGPT Codex | `gpt-6.1-sol`, `gpt-6-astra`, `gpt-6-sol`, `gpt-6-luna` | Adopt the current minimum catalog client version; retain authenticated dynamic discovery, visibility, reasoning, context, and tier metadata. |
| Z.AI Coding Plan | `glm-5.3`, `glm-5.3-flash` | Already included and selectable in v0.3.18, including Flash image input and current-catalog precedence over old login caches. Existing `[1m]` compatibility routes remain intact. |
| Kimi Code | `k3`, `k3-256k`, `kimi-for-coding` (K2.8 Preview), `kimi-for-coding-highspeed` | Existing authenticated catalog is not restricted by model ID. It maps provider-supplied context, protocol, reasoning, and image capabilities. |
| xAI | `grok-4.7` | Add the documented 500,000-token model and update bundled coding/search/image-description/summary defaults. Preserve `grok-4.6`, `grok-4.5`, and `grok-build`. |

This inventory covers the fork's established coding providers, not every model
sold by every vendor. A model's public API availability does not establish a
ChatGPT subscription entitlement. No API-key fallback or unauthenticated Codex
model injection was added. Public API context windows do not override a Codex
account's active catalog window. Server discovery and explicit user settings
continue to take precedence over bundled xAI defaults.

## Codex catalog compatibility evidence

Source: `openai/codex@49be2c7ab029434dc9545cf85a7a41fd5d3beb1a`,
[`codex-rs/models-manager/models.json`](https://github.com/openai/codex/blob/49be2c7ab029434dc9545cf85a7a41fd5d3beb1a/codex-rs/models-manager/models.json).
The public metadata declares minimum client versions `0.153.0` for Astra and
Sol 6.1, and `0.155.0` for GPT-6 Sol and Luna. Enhanced previously requested
catalogs with `client_version=0.144.0`, below those model gates. The compatibility
constant now uses `0.155.0`, shared by catalog query, request version headers,
and existing provider integrations. The existing cache scope includes the client
version, so older-version catalog data is not silently reused.

The audited model metadata remains compatible with the current schema: model
IDs are dynamic; `list`/`hide` visibility remains authoritative; reasoning levels
include low through max and, where advertised, ultra; priority service tiers,
input modalities, active/max context windows and compaction metadata are retained.
The fork's existing ultra-to-max wire mapping and Responses transport remain in
place. This does not establish adoption of new Codex application architecture,
async tool execution, or every newly published service-tier feature.

Regression tests exercise the production default query/version header with a
synthetic catalog containing the current GPT-6 IDs, and map these models through
the existing provider-bound catalog representation. They check identity, endpoint,
Responses routing, reasoning, image support, visibility and absence of embedded
credentials. Existing catalog tests cover account/cache isolation, sensitive
headers, redaction, and auth recovery.

## Other provider evidence

- [GLM-5.3-Flash documentation](https://docs.z.ai/guides/vlm/glm-5.3-flash):
  confirms the model ID, Coding Plan availability, image input, 1M context,
  128K output and enabled thinking. FlashX is explicitly outside the plan;
  the existing transport rejection regression now covers that name and a
  foreign-provider model as well. No new Z.AI runtime route is required.
- [Coding Plan overview](https://docs.z.ai/devpack/overview): lists GLM-5.3
  and GLM-5.3-Flash. Account quota remains provider-controlled.
- [Kimi model configuration](https://www.kimi.com/code/docs/en/kimi-code/models.html):
  lists the four model IDs above. Their account-dependent context windows
  continue to come from authenticated discovery; no static entitlement was added.
- [Grok 4.7 documentation](https://docs.x.ai/developers/grok-4-7) and
  [release notes](https://docs.x.ai/developers/release-notes): confirm the
  current model, tool support and reasoning levels. The product-specific Fast
  variant remains remote-catalog controlled; no guessed API slug was introduced.
- [Sol 6.1](https://developers.openai.com/api/docs/models/gpt-6.1-sol) and
  [Astra](https://developers.openai.com/api/docs/models/gpt-6-astra): confirm
  the public API IDs and Responses tool support, independently of subscription
  access. Subscription metadata comes from the Codex source/catalog above.

All implementation changes are independent edits to existing fork code. No
upstream instructions, model prompts, source code, or assets were copied. Existing
source notices remain intact. Latest fetched advances for sources actually fetched;
full-source Reviewed revisions do not advance on the strength of this scoped audit.

## Follow-up: “5.3 Flash Fast”

The September 30 follow-up checked the requested fast variant against primary
provider documentation. Z.AI names its faster Flash offering `glm-5.3-flashx`;
the [Flash/FlashX documentation](https://docs.z.ai/guides/vlm/glm-5.3-flash)
explicitly says FlashX is not yet available on the Coding Plan. The existing
Coding Plan transport therefore continues to reject it.

[Baseten also advertises GLM-5.3 Fast](https://www.baseten.co/library/glm-53-fast/)
as a separate Model API offering. That page does not establish a Z.AI Coding
Plan alias. No verified `glm-5.3-flash-fast` Coding Plan ID or Fast service-tier
parameter was found in the official Z.AI pages checked. The user subsequently clarified that only models available through the Coding
Plan are wanted, so neither separately hosted Fast nor FlashX is added. No
subscription credential may be reused for it. No runtime change was made for
this ambiguous name; the prior runtime validation remains applicable.

## Validation and release status

The xai-grok-models test, all five xai-grok-version tests, and all 110
xai-grok-shell catalog tests pass, including both new GPT-6 catalog regressions.
Strict ownership, fork contracts, five release-pipeline tests, fifteen installer
tests, and the 343-theme Warp vendor lock pass. The release contract accepts
version 0.3.19, selected for the focused model release. This validation
record does not itself establish publication.
All 30 Codex transport tests, seven Z.AI transport tests, and the explicit
Codex static/generic-credential rejection test pass (154 focused Rust tests
in total). `cargo fmt --all -- --check` passes. The required
`cargo check --locked -p xai-grok-pager-bin` passes (7m07s).
Rust validation uses locked dependencies with `CARGO_INCREMENTAL=0`,
`CARGO_PROFILE_DEV_DEBUG=0`, and `CARGO_PROFILE_TEST_DEBUG=0`.
No live credential-bearing provider requests have been made, so entitled-account
availability is not asserted. No credentials or authenticated payloads were read
or included in this report.

Homebrew validation uses the official ARM64 image
`ghcr.io/homebrew/brew@sha256:63f9a03880f954b2dc05ee0b8e93903dae2f161883af0072d77f476303fc834a`,
with Homebrew 7.0.7. Its installation is isolated from the host. The existing
v0.3.18 formula is installed first so a candidate upgrade can be tested. Baseline
`brew style`, `brew audit --strict --online`, `brew test`, `brew list --versions`,
`grok version`, and `agent version` all pass. The container is named
`gbe-brew-models-20260930`; the local tap checkout is the sibling
`homebrew-tap-models-20260930` worktree.

These results validate the Homebrew environment and installed baseline only.
A new formula still requires release assets, all four verified hashes, repeated
style/audit/test checks, and the installed v0.3.18-to-new-version upgrade check.
No release tag, project push, or tap push has been made.

## Publication scope confirmed after validation

The release version is 0.3.19. The Z.AI scope is every model available through
the Coding Plan, retaining the existing GLM-5.3 and GLM-5.3-Flash routes.
Publication is a focused downstream model update on the validated first-parent
history. It does not complete the full upstream refresh or acknowledge a newly
fetched Grok snapshot. The twelve carried parity obligations remain open.
Release assets, attestations, formula checks, and installed upgrade validation
will be verified during publication.
