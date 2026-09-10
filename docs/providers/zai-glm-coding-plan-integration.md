# Z.AI GLM Coding Plan provider

> Status: **implemented and experimental; offline-qualified**. Grok Build
> Enhanced ships a provider-scoped API-key adapter and audited GLM-5.3 catalog.
> Search/Reader, quota display, Zread, and opt-in Vision MCP are implemented.
> Live subscription qualification remains credential-gated.

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

Startup merges the current audited catalog with credential-bound cached entries;
an old login cache cannot hide newly shipped models or override their metadata.
The audited models are also visible before login, under Z.AI Coding Plan.
Visibility is not authentication: selecting or running them still requires a
provider-scoped Coding Plan key, never another provider's key. GLM models listed
under OpenCode Go or custom providers are separate routes and are not relabeled.
If an older installation only shows GLM-5.2, update the executable actually
selected by your shell (`command -v grok` and `grok version`), run
`grok models --provider zai-coding-plan` to refresh its saved catalog, then fully
quit and restart Grok. A separately installed `~/.local/bin/grok` can take
precedence over Homebrew's executable. Model listing does not prove entitlement
or perform a live inference request.

| Model | Input | Context | Max output | Reasoning |
| --- | --- | ---: | ---: | --- |
| `glm-5.3` | text | 1,000,000 | 131,072 | `low`, `high`, `max` |
| `glm-5.3[1m]` | text | 1,000,000 | 131,072 | `low`, `high`, `max` |
| `glm-5.3-flash` | text and images | 1,000,000 | 131,072 | `low`, `high`, `max` |
| `glm-5.3-flash[1m]` | text and images | 1,000,000 | 131,072 | `low`, `high`, `max` |

Z.AI now documents both Flash IDs as available to Coding Plan users. Enhanced
qualifies text plus OpenAI-compatible `image_url` blocks (remote URLs and Base64
data URLs) through its existing attachment path. Although the upstream model
also advertises video and file inputs, Enhanced does not claim those input
paths until they have separate harness and wire qualification.

The default reasoning effort is `max`. Grok maps `none`, `minimal`, and `low`
to the provider's `low`; `medium` and `high` to `high`; and `xhigh`, `max`, and
`ultra` to `max`. It sends `thinking.type = enabled` and
`thinking.clear_thinking = false`; GLM-5.3 does not support disabling reasoning.

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

## Coding Plan tools and quota

The 2026-09-10 integration keeps Grok's agent loop, tool schemas, permissions,
headless mode, sessions, and ACP transport. It does not embed another agent.

| Surface | Route and behavior |
| --- | --- |
| `web_search` | Z.AI Search MCP; projects structured results into Grok text and citations |
| `web_fetch` | Z.AI Reader MCP; Grok URL/domain/SSRF checks run before the provider call |
| `/usage` | Numeric-only Coding Plan quota rows, separate from session token totals and xAI billing |
| `/usage manage` | Z.AI subscription management |
| `zread_search_doc` | Search a public GitHub repository's documentation |
| `zread_get_repo_structure` | Inspect a public repository tree |
| `zread_read_file` | Read a repository-relative file; never a local file substitute |
| Vision MCP | Opt-in image/video tools and `zai_vision_doctor`; native Flash images work without it |

Search, Reader, and Zread use fixed `api.z.ai/api/mcp/.../mcp` endpoints,
initialize MCP, acknowledge initialization, inspect the advertised catalog, and
invoke only the expected tool names. Search handles the documented
`webSearchPrime` and advertised `web_search_prime` spellings and selects
`query` versus `search_query` from the advertised schema. Responses are bounded.
Configured domain policy filters both rendered search text and citations;
unsupported result shapes fail closed. Reader does not silently fall back to
xAI, a paid Open Platform API, or an unrelated service. Navigation-only search
commands are not supported by Z.AI Search; use `web_fetch` for URLs.

Quota uses the official Z.AI usage plugin's
`GET https://api.z.ai/api/monitor/usage/quota/limit` contract. This endpoint
receives a sensitive **bare API key** Authorization header, unlike the Bearer
header used for inference and MCP. No browser cookie or OAuth token is used.
Model-token, model-credit, and MCP quota percentages are displayed without
inventing reset dates or treating absent fields as zero usage. Optional
model/tool historical series are not needed for the quota display.

Enable Vision explicitly:

```sh
GROK_ZAI_VISION_MCP=1 grok -m 'zai-coding-plan/glm-5.3-flash'
```

Node.js 22+ and npm are required. The adapter installs
`@z_ai/mcp-server@0.1.4` into a temporary private directory with an empty
credential environment and lifecycle scripts disabled. It checks the package
version, resolved tarball URL, and published SHA-512 integrity recorded by npm
before launching node with only the scoped Z.AI key. The child has a bounded
stdio exchange and is terminated on completion/cancellation. The package is
not bundled in the Rust executable. Local images/videos must resolve beneath
the workspace or session directory; remote media URLs and escaping symlinks
are rejected. Download media through the normal permissioned, SSRF-safe tools
first. Videos are limited to 8 MiB. This is an opt-in external process, not an
OS sandbox or a claim that all transitive npm dependencies have been audited.

Provider changes remove Zread/Vision resources and tool definitions. Resumed
and same-provider switched sessions preserve their credential record; a
different key requires explicit rebinding. Generic API-key readers cannot
obtain the Z.AI key.

### Remaining credential-gated acceptance

The 2026-09-11 catalog fix passed 56 focused Z.AI tests, 90 model-manager
tests, and 27 auth-method tests. Coverage includes logged-out picker visibility
in both xAI auth modes, stale-cache upgrades, current metadata precedence,
foreign-cache rejection, and runtime rejection of foreign keys. The wider
configuration suite passed 337/338 tests; its unchanged
`known_non_serde_config_paths_are_not_reported_unused` assertion fails on
`marketplace.plugin_cta_marketplace`, outside the model-catalog path.

Offline validation on 2026-09-10 passed the 51 focused Z.AI tests (7 sampler,
23 shell, 21 tools), plus the auth-method (27), provider-media-switch (16),
web-search (49), web-fetch (143), and authentication-error (44) regression
suites. These suite counts overlap and are not a unique-test total. Fork
contracts, strict committed-tree ownership coverage, formatting, and
`CARGO_INCREMENTAL=0 cargo check -p xai-grok-pager-bin` also passed. Test
binaries were linked with the Rust toolchain's bundled LLD because the system
linker caused severe memory pressure; repository build settings were unchanged.

No entitled credential was present on 2026-09-10. Do not infer current live
qualification from the historical July implementation. Before claiming the
live matrix complete, verify: Flash text/images; a streamed reasoning/tool
roundtrip; Search, Reader and all three Zread calls; Vision doctor plus one
image/video call when enabled; quota display; resume, provider switch,
compaction, headless/ACP and a subagent; invalid-key/quota/entitlement error
handling. Use synthetic inputs and retain only pass/fail evidence, never
credentials or authenticated response captures.

## Primary sources

- [Coding Plan model switching, endpoint, and effort mapping](https://docs.z.ai/devpack/latest-model)
- [GLM-5.3 model and reasoning contract](https://docs.z.ai/guides/llm/glm-5.3)
- [GLM-5.3-Flash multimodal contract](https://docs.z.ai/guides/vlm/glm-5.3-flash)
- [Chat Completions API](https://docs.z.ai/api-reference/llm/chat-completion)
- [API error codes and response shape](https://docs.z.ai/api-reference/api-code)
- [Coding Plan quick start](https://docs.z.ai/devpack/quick-start)
- [Search MCP](https://docs.z.ai/devpack/mcp/search-mcp-server), [Reader MCP](https://docs.z.ai/devpack/mcp/reader-mcp-server), [Zread MCP](https://docs.z.ai/devpack/mcp/zread-mcp-server), [Vision MCP](https://docs.z.ai/devpack/mcp/vision-mcp-server)
- [Official usage plugin](https://github.com/zai-org/zai-coding-plugins/blob/main/plugins/glm-plan-usage/skills/usage-query-skill/scripts/query-usage.mjs)
- [Pinned Vision package metadata](https://registry.npmjs.org/@z_ai%2fmcp-server/0.1.4)

Repository source pins and their separate Reviewed/Latest fetched states remain
in [`UPSTREAM_VERSIONS.md`](../../UPSTREAM_VERSIONS.md). Ignored checkouts,
credentials, authenticated response captures, and private data must not be
committed.
