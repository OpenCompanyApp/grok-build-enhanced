# Z.AI GLM Coding Plan interoperability research

> Status: **research only**. Grok Build Enhanced does not ship a Z.AI runtime
> provider, login command, credential scope, model catalog, usage surface, or
> hosted-tool integration. The identifiers and request shapes below describe
> upstream systems and possible interoperability constraints; they are not
> supported configuration or product claims.

## Scope

This note records public-source research useful when evaluating provider
interoperability. It does not authorize implementation. The fork's runtime
providers remain xAI, ChatGPT Codex subscription, Kimi Code, and generic custom
transport. Adding another runtime provider would require a separate fork-vision
decision, complete credential-isolation design, licensing review, tests, and
explicit user approval.

The reviewed sources are tracked in `UPSTREAM_VERSIONS.md`:

- `zai-org/z-ai-sdk-python` for public API request conventions;
- `zai-org/zai-coding-plugins` for coding-tool interoperability research;
- `zai-org/GLM-5` for model metadata research;
- CodexBar and the Z.AI usage-browser repository for UI/schema research only;
- `sst/models.dev` for non-authoritative catalog corroboration only.

Ignored source checkouts belong under `inspiration/`. Credentials, private
correspondence, authenticated response captures, and unreviewed source must
never be committed.

## GLM-5.3 Coding Plan status (2026-08-15)

Z.AI's public documentation now lists GLM-5.3 as available to all Coding Plan
tiers (Lite, Pro, and Max). This fork records the launch as a research input; it
does not add the model to an Enhanced runtime catalog.

The documented compatibility surface is:

- model ID `glm-5.3`, with `glm-5.3[1m]` as an optional explicit one-million-
  token-context selection;
- a 1,048,576-token context window and 131,072-token maximum output;
- reasoning levels `low`, `high`, and `max`, with Coding Plan's Codex mapping
  treating minimal/light/low as low, medium/high as high, and
  xhigh/max/ultra as max;
- `https://api.z.ai/api/v1` for Codex-compatible clients,
  `https://api.z.ai/api/anthropic` for Anthropic-compatible clients, and
  `https://api.z.ai/api/coding/paas/v4` for other OpenAI-compatible clients;
  and
- Coding Plan credit multipliers of 6.9 for input, 1.7 for cache reads, and 24
  for output. Public documentation says older GLM-5.2 and GLM-5.1 requests are
  automatically routed to GLM-5.3.

The public model guide describes GLM-5.3 API availability as coming soon while
the Coding Plan documentation says the model is already live for plan users.
That distinction must remain explicit in any later evaluation; Coding Plan
availability is not evidence that every API product or endpoint has the same
launch state.

Primary public references:

- [GLM-5.3 model guide](https://docs.z.ai/guides/llm/glm-5.3)
- [Coding Plan model selection and reasoning mapping](https://docs.z.ai/devpack/latest-model)
- [Coding Plan overview and credit multipliers](https://docs.z.ai/devpack/overview)

## Observed public contracts

Public materials distinguish the global Coding Plan service from Open Platform
pay-as-you-go and BigModel China routes. Any future adapter evaluation would
therefore need explicit endpoint ownership and must reject credential or state
fallback between those services.

The researched Coding Plan API resembles OpenAI-compatible Chat Completions and
has exposed provider-specific reasoning and tool-stream fields. Public examples
also describe authenticated model discovery and hosted MCP-style tools. These
observations are intentionally not mapped to runtime enums, CLI commands,
environment variables, auth records, or user-facing capabilities in this fork.

If this research is revisited, the minimum security questions are:

- Can every credential header be marked sensitive and kept out of logs,
  snapshots, hooks, diagnostics, and subprocess arguments?
- Can credentials, retry state, caches, usage data, and logout remain isolated
  from xAI, Codex, Kimi, and custom-provider state?
- Can every URL and hosted-tool route be pinned to the intended service without
  static-key or generic-provider fallback?
- Can malformed HTTP 200 business-error envelopes, quota responses, and
  reasoning metadata fail safely and remain bounded?
- Can local media and subprocess integrations preserve the existing workspace,
  permission, and path-trust boundaries?

## Non-goals and standing decision

Research does not establish a `zai_coding_plan` provider identity or credential
source. It does not establish `grok login`, `grok logout`, or `grok models`
commands for Z.AI; a model namespace; monitoring or quota claims; Search,
Reader, Zread, or Vision tools; or official endorsement.

No Z.AI package is bundled, downloaded, or launched by Grok Build Enhanced.
The ordinary custom-provider transport also does not infer Z.AI identity or
receive credentials from any other provider.

## Provenance discipline

Update the reviewed source revisions only after reviewing the relevant diff,
compatibility impact, notices, and this research note. If code is ever ported
rather than independently implemented, preserve its license and add the
required notices before landing it. Until the fork vision changes explicitly,
all Z.AI source entries remain research references only.
