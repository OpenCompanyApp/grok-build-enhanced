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

## Validation and release status

The xai-grok-models test and all five xai-grok-version tests pass. Catalog, provider,
binary, release-contract, and Homebrew validation results are pending.
No live credential-bearing provider requests have been made, so entitled-account
availability is not asserted. No credentials or authenticated payloads were read
or included in this report.

Homebrew validation uses the official ARM64 image
`ghcr.io/homebrew/brew@sha256:63f9a03880f954b2dc05ee0b8e93903dae2f161883af0072d77f476303fc834a`,
with Homebrew 7.0.7. Its installation is isolated from the host. The existing
v0.3.18 formula is installed first so a candidate upgrade can be tested.
