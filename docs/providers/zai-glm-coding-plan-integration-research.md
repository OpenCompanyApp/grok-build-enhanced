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
- CodexBar and the Z.AI usage-browser repository for UI/schema research only.

Ignored source checkouts belong under `inspiration/`. Credentials, private
correspondence, authenticated response captures, and unreviewed source must
never be committed.

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
