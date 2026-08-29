# Z.AI GLM Coding Plan provider

> Status: **implemented and experimental; offline-qualified**. Grok Build
> Enhanced ships a provider-scoped API-key adapter and audited GLM-5.3 catalog.
> Live inference remains credential-gated. Usage and provider-hosted MCP tools
> are not enabled.

## Runtime contract

| Concern | Implemented value |
| --- | --- |
| Provider ID | `zai_coding_plan` |
| Credential source | `zai_coding_plan_api_key` |
| Auth-store scope | `zai::coding-plan::global` |
| Environment fallback | `Z_AI_API_KEY` |
| Model namespace | `zai-coding-plan/<model-id>` |
| Base URL | `https://api.z.ai/api/coding/paas/v4` |
| Inference | `POST /chat/completions` |
| Authentication | one sensitive `Authorization: Bearer …` header |

The provider identity is never inferred from a custom base URL. Its API key
cannot fall through to or from xAI, ChatGPT Codex, Kimi Code, OpenCode Go,
Z.AI Open Platform pay-as-you-go, BigModel China, or generic custom-provider
credentials.

## Login and model selection

Store a key through standard input or a protected environment source:

```sh
secure-key-command | grok login --provider zai-coding-plan

export Z_AI_API_KEY="$(secure-key-command)"
grok login --provider zai-coding-plan
unset Z_AI_API_KEY
```

The login command validates the credential's local bearer-header shape and
stores it in `~/.grok/auth.json` under the dedicated scope. It does not claim
to validate entitlement before the first inference request because Z.AI does
not document a Coding Plan model-discovery endpoint.

List the audited static catalog and start a session with a namespaced ID:

```sh
grok models --provider zai-coding-plan
grok -m 'zai-coding-plan/glm-5.3'
grok -m 'zai-coding-plan/glm-5.3-flash[1m]'
```

| Model | Input | Context | Max output | Reasoning |
| --- | --- | ---: | ---: | --- |
| `glm-5.3` | text | 1,000,000 | 131,072 | `low`, `high`, `max` |
| `glm-5.3[1m]` | text | 1,000,000 | 131,072 | `low`, `high`, `max` |
| `glm-5.3-flash` | text and images | 1,000,000 | 131,072 | `low`, `high`, `max` |
| `glm-5.3-flash[1m]` | text and images | 1,000,000 | 131,072 | `low`, `high`, `max` |

The default reasoning effort is `max`. Grok maps `none`, `minimal`, and `low`
to the provider's `low`; `medium` and `high` to `high`; and `xhigh`, `max`, and
`ultra` to `max`. It always sends `thinking.type = enabled`; GLM-5.3 does not
support disabling reasoning.

Logout removes only this provider's record and catalog cache:

```sh
grok logout --provider zai-coding-plan
```

## Security and wire behavior

The provider binder rebuilds the canonical endpoint, Chat Completions backend,
bearer scheme, and dynamic request-auth source from the stored provider record.
It clears restored static keys and foreign headers, rejects redirects, rejects
unapproved headers, and requires one sensitive bearer header bound to the same
opaque credential record. Responses are size-bounded and provider error bodies
and trace identifiers are reduced to fixed, non-secret diagnostics.

Function tools use the documented maximum of 128 and `tool_choice = auto`.
Streaming requests enable `tool_stream` and request a final usage chunk. The
existing Grok agent loop, tools, permissions, sessions, TUI, headless mode, and
ACP behavior remain unchanged.

## Qualification boundary

The adapter has unit and integration coverage for credential redaction,
provider-scope preservation, logout repair, model metadata, effort mapping,
request shaping, header sealing, retry classification, bounded bodies, CLI
parsing, session binding, and credential non-fallback. The 2026-08-29 refresh
did not have an entitled Z.AI credential, so live inference and authentication
rejection behavior remain unqualified.

Usage/quota endpoints and Z.AI-hosted Search, Reader, Zread, and Vision MCP
services are intentionally absent. Their public existence does not authorize
credential forwarding or product claims without a separate contract audit and
test matrix.

## Primary sources

- [Coding Plan model switching, endpoint, and effort mapping](https://docs.z.ai/devpack/latest-model)
- [GLM-5.3 model and reasoning contract](https://docs.z.ai/guides/llm/glm-5.3)
- [GLM-5.3-Flash multimodal contract](https://docs.z.ai/guides/vlm/glm-5.3-flash)
- [Chat Completions API](https://docs.z.ai/api-reference/llm/chat-completion)
- [API error codes and response shape](https://docs.z.ai/api-reference/api-code)
- [Coding Plan quick start](https://docs.z.ai/devpack/quick-start)

Repository source pins and their separate Reviewed/Latest fetched states remain
in [`UPSTREAM_VERSIONS.md`](../../UPSTREAM_VERSIONS.md). Ignored checkouts,
credentials, authenticated response captures, and private data must not be
committed.
