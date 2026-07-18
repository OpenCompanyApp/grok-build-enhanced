# Z.AI GLM Coding Plan integration research

> Status: research and implementation design only. Z.AI Coding Plan support
> described here is not yet implemented. The source revisions and endpoint
> observations below are prior research; this reconciliation did not fetch,
> re-review, or re-probe them and leaves
> [`UPSTREAM_VERSIONS.md`](../../UPSTREAM_VERSIONS.md) unchanged.
>
> The project owner reports written confirmation from Z.AI that Grok Build is
> approved and will be added to the supported-tool list. That private
> correspondence is project context, not public evidence that this repository
> already implements the provider or that public support or endorsement is in
> effect. Keep the email outside the repository and describe public listing as
> pending until the official documentation is updated.

## Executive conclusion

Z.AI GLM Coding Plan could fit this fork as a future first-class subscription
provider alongside the currently implemented xAI and experimental OpenAI Codex
providers. Kimi Code is a separate, unimplemented research proposal. Every
identifier and command in this document is proposed design notation only: the
checked-in provider and CLI enums contain no Z.AI variant, and no runtime path
can select or load Z.AI credentials. The recommended future architecture is:

- Proposed provider ID: `zai_coding_plan`.
- Proposed credential source: a Z.AI Coding Plan API key.
- Proposed credential scope: `zai::coding-plan::global`.
- Protocol: OpenAI-compatible Chat Completions.
- Base URL: `https://api.z.ai/api/coding/paas/v4`.
- Dynamic model discovery from authenticated `GET /models`.
- Dedicated adapters for Coding Plan Web Search, Web Reader, Zread, and Vision
  MCP services.
- Provider-specific quota reporting from the monitoring endpoints used by
  current Z.AI clients.
- A distinct `zai_open_platform` provider for pay-as-you-go inference, image
  generation, and other general API services.
- Grok Build retains its existing agent loop, tools, sessions, subagents,
  compaction, permissions, and TUI.

The inference transport itself is relatively conventional. The main
engineering risks are exact preserved-reasoning round trips, Z.AI's singular
`tool_stream` request field, old MCP protocol negotiation, HTTP 200 business
error envelopes, and keeping Coding Plan services separate from pay-as-you-go
endpoints.

## Source snapshot

This research reviewed the following gitignored `inspiration/` checkouts:

- [zai-org/z-ai-sdk-python][zai-sdk-python] at
  `ca5109c0aa9bf173839be391b4b14aeadf9a9bf9`.
- [zai-org/zai-coding-plugins][zai-coding-plugins] at
  `0446d0bb0bc537d97d3ab3664c4b8b9c4a0e1254`.
- [zai-org/GLM-5][glm-5-github] at
  `436efa09bc868a6922e307624189e7018406beb9`.
- [anomalyco/opencode][opencode-github] only as an interoperability reference.
  The canonical ledger records
  `1d2a7b4c860f6a29eb90bdda07757b2adf34ab61` as last reviewed and
  `efb6cc2d4bf6332eb156709795d2b3a649198b65` as latest fetched; this
  reconciliation inspected neither and does not promote the latter.
- [steipete/CodexBar][codexbar-github] at
  `e6b2ea490cec47d122ba14e32bfc41e53d98422c`.
- [nniicckk6/zai-extention][zai-usage-helper] at
  `54cd1f33a703c417f2492ee1f21f22b3633a43c4`.

Official Z.AI documentation is the source of truth for plan models, inference,
MCP services, and pricing. OpenCode is a useful OpenAI-compatible transport
reference. CodexBar and the browser usage helper are community references for
the currently undocumented monitoring endpoints; their schemas must be
treated as drift-prone.

If code is ported rather than behavior independently implemented, preserve the
source repository's copyright, license, attribution, and prominent
modification requirements. Do not ship the ignored checkouts.

## Coding Plan versus Open Platform

These are separate products and must be separate provider identities:

| Product | Base URL | Billing | Intended use |
| --- | --- | --- | --- |
| Z.AI Coding Plan | `https://api.z.ai/api/coding/paas/v4` | Subscription quota | Interactive coding agents |
| Z.AI Anthropic compatibility | `https://api.z.ai/api/anthropic` | Subscription quota | Claude-compatible coding agents |
| Z.AI Open Platform | `https://api.z.ai/api/paas/v4` | Pay per use | General API applications |
| BigModel CN Coding Plan | `https://open.bigmodel.cn/api/coding/paas/v4` | Regional subscription quota | China-region coding agents |
| BigModel CN Anthropic compatibility | `https://open.bigmodel.cn/api/anthropic` | Regional subscription quota | China-region Claude-compatible agents |

Using the general Open Platform endpoint can deduct ordinary balance or return
business error `1113` even when the user owns a Coding Plan. Coding Plan quota
does not automatically fall through to pay-as-you-go balance. See the
[Coding Plan tool integration guide][zai-tools] and [FAQ][zai-faq].

Recommended provider identities are:

```text
zai_coding_plan
zai_open_platform
zhipuai_coding_plan
zhipuai_open_platform
```

Global and China-region credentials must never be inferred solely from a base
URL or silently substituted for one another.

## Current plan snapshot

The current official plan starts at $18 per month and exposes three tiers:

| Tier | Approximate five-hour prompts | Approximate weekly prompts | Monthly Search/Reader/Zread calls |
| --- | ---: | ---: | ---: |
| Lite | 80 | 400 | 100 |
| Pro | 400 | 2,000 | 1,000 |
| Max | 1,600 | 8,000 | 4,000 |

One user prompt is estimated to drive 15-20 internal model calls. These are
product estimates, not stable limits to compile into the client. Always prefer
the authenticated quota response.

GLM-5.2 and GLM-5-Turbo normally consume quota at 3x during peak hours and 2x
off-peak. A limited promotion currently reduces off-peak consumption to 1x
through the end of September 2026. Peak hours are 14:00-18:00 UTC+8. These
rules belong in dated documentation and usage metadata, not permanent model
logic. See the current [Coding Plan overview][zai-overview].

## Endpoint map

| Purpose | Request | Contract |
| --- | --- | --- |
| Chat Completions | `POST https://api.z.ai/api/coding/paas/v4/chat/completions` | Coding Plan OpenAI compatibility |
| Model catalog | `GET https://api.z.ai/api/coding/paas/v4/models` | Authenticated account catalog |
| Web Search MCP | `https://api.z.ai/api/mcp/web_search_prime/mcp` | Remote Streamable HTTP MCP |
| Web Reader MCP | `https://api.z.ai/api/mcp/web_reader/mcp` | Remote Streamable HTTP MCP |
| Zread MCP | `https://api.z.ai/api/mcp/zread/mcp` | Remote Streamable HTTP MCP |
| Vision MCP | Local `npx @z_ai/mcp-server@latest` | Local stdio MCP backed by GLM-4.6V |
| Quota monitor | `GET https://api.z.ai/api/monitor/usage/quota/limit` | Current client behavior; undocumented |
| Model usage | `GET https://api.z.ai/api/monitor/usage/model-usage` | Current client behavior; undocumented |
| Pay-go Reader | `POST https://api.z.ai/api/paas/v4/reader` | General API, not plan Reader MCP |
| Pay-go Search | `POST https://api.z.ai/api/paas/v4/web_search` | General API, not plan Search MCP |
| Image generation | `POST https://api.z.ai/api/paas/v4/images/generations` | General pay-go API |

Unauthenticated probes on 2026-07-17 confirmed that the Coding Plan `/models`
endpoint exists and returns an authentication failure. The monitoring and MCP
endpoints returned HTTP 200 with a JSON authentication error envelope instead
of an HTTP 401. Consequently every Z.AI client path must inspect both HTTP
status and the JSON business result.

## Authentication and credential isolation

Coding Plan uses an API key:

```text
Authorization: Bearer <Z.AI API key>
```

There is no refresh token or public third-party OAuth flow required for this
integration. ZCode supports account authorization for its own application, but
does not publish a reusable third-party authorization contract. Grok should
not emulate or reverse-engineer ZCode login. See [ZCode model
configuration][zcode-configuration].

Proposed future UX:

```text
grok login --provider zai-coding-plan
grok logout --provider zai-coding-plan
grok auth status --provider zai-coding-plan
grok models --provider zai-coding-plan
```

These commands are design examples; the current binary does not accept the
`zai-coding-plan` provider.

Login should prompt for the one-time-visible API key, validate it against the
Coding Plan catalog and quota endpoints, then store it under
`zai::coding-plan::global`. Logout removes only that record. Selecting a Z.AI
model must not replace xAI, Codex, Kimi, or ordinary Z.AI Open Platform
credentials.

Use `Z_AI_API_KEY` as the preferred environment fallback. `ZHIPU_API_KEY` can
be recognized as a clearly documented legacy/OpenCode alias, with explicit
precedence and no value logging. The key must be dynamically injected into
inference, MCP, and usage requests; it must not be copied into `config.toml` or
a custom model definition.

## Models and discovery

The current official Coding Plan model set is:

| Model | Intended use | Context | Maximum output | Input |
| --- | --- | ---: | ---: | --- |
| `glm-5.2` | Highest-intelligence complex coding | 1,000,000 | 131,072 | Text |
| `glm-5-turbo` | Faster flagship coding | Discover dynamically | Discover dynamically | Text |
| `glm-4.7` | Routine coding and quota efficiency | 200,000 | 131,072 | Text |

Older FAQ pages and third-party catalogs contain GLM-5.1, GLM-4.5-Air,
GLM-5V-Turbo, and other historical entries. They are not reliable entitlement
sources. The current [Coding Plan overview][zai-overview] lists GLM-5.2,
GLM-5-Turbo, and GLM-4.7.

Model discovery should:

1. Fetch authenticated `GET /models` with the scoped plan key.
2. Parse the account's actual returned IDs.
3. Intersect them with capabilities the adapter understands.
4. Cache results briefly and persist a last-known catalog.
5. Mark fallback results as stale.
6. Refresh once on model errors `1211` or `1311`.
7. Never redirect an unavailable plan model to pay-as-you-go inference.

For OpenAI-compatible tools the 1M model ID is `glm-5.2`. The
`glm-5.2[1m]` suffix is an Anthropic/Claude compatibility alias and should not
appear in Grok's model catalog. Image support must remain disabled for the
Coding Plan text endpoint. See [How to Switch Models][zai-model-switching].

## Thinking levels

GLM-5.2 accepts compatibility aliases but has two effective reasoning
strengths plus disabled thinking:

| Grok selection | Request | Effective GLM behavior |
| --- | --- | --- |
| `off`, `none`, `minimal` | `thinking.type = disabled` or effort `none` | No thinking |
| `low`, `medium`, `high` | `reasoning_effort = high` | High |
| `xhigh`, `max`, `ultracode` | `reasoning_effort = max` | Max |

The API default is `max`. Z.AI maps `low` and `medium` to `high`, and `xhigh`
to `max`. Grok should display `off`, `high`, and `max` as the honest native
choices while accepting its cross-provider aliases. See the
[Chat Completion reference][zai-chat-completion].

A GLM request should explicitly contain the selected behavior, for example:

```json
{
  "thinking": {
    "type": "enabled",
    "clear_thinking": false
  },
  "reasoning_effort": "max"
}
```

OpenCode was consulted only as transport research for this proposed request
shape. It is not a runtime provider in this fork, and its behavior must be
re-verified at a ledger-recorded revision before any implementation is based on
it.

## Preserved and interleaved reasoning

Z.AI enables interleaved thinking and Coding Plan preserved thinking. During
tool use, the model can reason, call a tool, inspect the result, and reason
again. Historical `reasoning_content` must be returned:

- Completely.
- Unmodified.
- In the original sequence.
- On the correct assistant/tool-call message.

Missing, truncated, rewritten, or reordered reasoning can degrade agent
performance and cache hits. See the official [Thinking Mode
documentation][zai-thinking].

Grok already streams `reasoning_content` and folds reasoning items onto Chat
Completions assistant messages. One incompatibility must be addressed:
[`conversation_to_chat_messages`](../../crates/codegen/xai-grok-sampling-types/src/conversation.rs)
currently joins consecutive reasoning siblings with a newline. That can mutate
the exact provider-produced sequence. GLM support needs tests proving exact
round-trip identity and, if necessary, a representation that preserves the
original boundaries or raw concatenation.

Compaction may summarize old history, but it must preserve the complete active
assistant reasoning, tool-call, tool-result chain leading into the next model
request. If exact preserved thinking is lost after old-history compaction, the
client should treat the compacted prefix as a new context boundary rather than
fabricating provider reasoning.

## Chat Completions and tool streaming

Required request behavior:

- Use the Coding Plan Chat Completions endpoint.
- Send `stream: true`.
- Request final usage information where supported.
- Send `thinking` and normalized `reasoning_effort` for GLM-5.2.
- Preserve `reasoning_content` on historical assistant turns.
- Send `tool_stream: true` when streaming function-call arguments.
- Keep `tool_choice` at the supported `auto` behavior.
- Enforce the documented maximum of 128 functions.
- Never inject xAI request headers or Codex Responses fields.

Z.AI uses the singular field:

```json
{"tool_stream": true}
```

Grok's current generic option injects the xAI-specific plural
`stream_tool_calls`. Z.AI therefore needs a provider-specific request shaper,
not another unconditional toggle. See the [Chat Completion
reference][zai-chat-completion].

Streaming tests must cover:

- Separate text and `reasoning_content` deltas.
- Multiple parallel calls interleaved by tool-call index.
- Function names, IDs, and JSON arguments split across arbitrary chunks.
- Empty initial chunks.
- Tool calls surrounded by interleaved reasoning.
- Final usage present only in the last event.
- `[DONE]` termination.
- Abnormal inference termination represented through `finish_reason` instead
  of an ordinary JSON error response.
- Cancellation and idle timeout without retry loops.

The existing Chat Completions collector, cached-token normalizer, and idle
timeout are useful foundations. The outgoing wire policy must be provider
owned.

## Context caching

Z.AI caching is implicit. There is no documented stable cache-key request
parameter. The service automatically recognizes reusable prefixes and reports:

```text
usage.prompt_tokens_details.cached_tokens
```

Stable system prompts, exact conversation history, unmodified preserved
reasoning, and consistent tool schemas improve hits. Even small formatting
changes may reduce cache effectiveness. Cache entries expire, but Z.AI does
not publish a dependable TTL. See [Context Caching][zai-cache].

Grok already normalizes `cached_tokens`. GLM integration should add:

- Session cache-hit percentage in usage and cost displays.
- Tests proving cached tokens survive the stream and conversation accounting.
- Stable serialization of system prompts, messages, reasoning, and tool
  schemas.
- Cache regression tests across resume, tool calls, model switching, and
  compaction.
- Credential-gated repeated-prefix live testing outside deterministic CI.

Do not claim a cache hit merely because the prefix is identical; only the
server's reported `cached_tokens` is authoritative.

## Web Search MCP

Endpoint:

```text
https://api.z.ai/api/mcp/web_search_prime/mcp
```

Tool:

```text
webSearchPrime
```

It returns page titles, URLs, summaries, site names, icons, and other search
metadata, and is intended for current web information such as releases, news,
weather, and prices. See the [Web Search MCP guide][zai-search-mcp].

Grok should expose it through its native web-search experience so search
results, citations, permissions, cancellation, and tool events remain
consistent across providers. Result normalization should preserve the source
URL and provider metadata while deduplicating canonical URLs.

The documented legacy SSE configuration places the API key in the query
string. Never use it. Query-string credentials leak into shell history, proxy
logs, telemetry, crash reports, and screenshots. Use Streamable HTTP with a
sensitive `Authorization` header.

## Web Reader MCP and local WebFetch

Endpoint:

```text
https://api.z.ai/api/mcp/web_reader/mcp
```

Tool:

```text
webReader
```

Reader returns the page title, main content, metadata, and links. It is a
hosted content-extraction service rather than a raw HTTP fetcher. See the
[Web Reader MCP guide][zai-reader-mcp].

Do not silently replace Grok's existing WebFetch:

- Local WebFetch is quota-free.
- It already has SSRF protection, redirect controls, caching, and support for
  non-HTML artifacts.
- Z.AI Reader uses the shared monthly MCP allowance.
- Reader may succeed on anti-bot pages that local fetch cannot parse.
- Local fetch is a useful fallback when MCP is unavailable or exhausted.

Recommended configuration:

```toml
[tools.web]
search = "zai"
reader = "auto" # auto | zai | local
```

`auto` can prefer Reader for ordinary public HTML and fall back to WebFetch on
quota, extraction, protocol, or service failure. Before sending any URL to the
remote Reader, Grok must still reject unsupported schemes, credentials in
URLs, localhost, private/link-local addresses, and prohibited redirects.

The direct `POST /api/paas/v4/reader` API is a separate pay-as-you-go service.
It offers options such as output format, cache bypass, retained images, image
summaries, and link summaries, but it must not be assumed to consume Coding
Plan Reader quota. See the [direct Reader API][zai-reader-api].

## Zread MCP

Endpoint:

```text
https://api.z.ai/api/mcp/zread/mcp
```

Tools:

- `search_doc`: search repository documentation, issues, PRs, news, and
  contributors.
- `get_repo_structure`: retrieve repository directories and file lists.
- `read_file`: read a complete file from a public GitHub repository.

See the [Zread MCP guide][zai-zread-mcp]. Zread is provider-specific repository
research and should not be disguised as ordinary URL fetching. It currently
targets public repositories; private GitHub authentication is not part of the
documented service.

Search, Reader, and Zread share the tier's monthly MCP allowance. `/usage`
should show the combined pool and the per-tool `usageDetails` returned by the
monitoring endpoint.

## Remote MCP protocol quirks

The three remote services are documented as Streamable HTTP, but current
field reports show a protocol negotiation incompatibility with some Rust MCP
clients:

1. The client requests MCP protocol `2025-03-26`.
2. Z.AI replies successfully but advertises `2024-11-05`.
3. The response includes `mcp-session-id`.
4. The client must send `notifications/initialized` with that session ID.
5. Some clients close the transport at this step even though the same sequence
   succeeds manually.

This has been reproduced for Search, Reader, and Zread in
[openai/codex issue 5619][codex-zai-mcp-issue]. Grok currently uses `rmcp`, so a
live credential smoke test is required before claiming compatibility.

If the current library fails, implement a narrow Z.AI compatibility transport
that:

- Accepts the negotiated `2024-11-05` version.
- Preserves and returns `mcp-session-id`.
- Sends `notifications/initialized` before `tools/list` or `tools/call`.
- Accepts `application/json` and `text/event-stream` responses.
- Parses authentication/business failures inside HTTP 200 responses.
- Coalesces reconnects and uses bounded backoff.
- Never emits raw request headers, query tokens, session IDs, or response
  bodies containing sensitive input.

A bundled proxy should be a last-resort workaround rather than the product
architecture. Live MCP verification should be credential-gated and separate
from deterministic mock CI because the service is network- and quota-coupled.

## Vision MCP and image understanding

GLM-5.2 on the Coding Plan endpoint is text-only. Image and video
understanding are delivered through the local Vision MCP server:

```text
npx -y @z_ai/mcp-server@latest
```

Environment:

```text
Z_AI_API_KEY=<scoped Coding Plan key>
Z_AI_MODE=ZAI
```

The current npm release observed during research is `0.1.4`. Its package
metadata declares Node 18+, while the official guide requires Node 22+. Grok
should enforce the stricter documented Node 22 minimum. The official guide
also warns that cached old npm versions can cause problems, so the doctor
command should report the resolved package version and offer a cache-clean or
`@latest` remediation. See [Vision MCP][zai-vision-mcp].

Available tools include:

- `ui_to_artifact`.
- `extract_text_from_screenshot`.
- `diagnose_error_screenshot`.
- `understand_technical_diagram`.
- `analyze_data_visualization`.
- `ui_diff_check`.
- `image_analysis`.
- `video_analysis` for MP4, MOV, or M4V content up to 8 MB.

Outside Claude Code, pasted images commonly bypass Vision MCP and are sent
directly to the selected model. Grok must deliberately resolve the attachment
to a safe local path and invoke the appropriate Vision MCP tool. Tool
permissions should make that local file access visible.

Vision consumes the ordinary five-hour plan resource pool rather than the
monthly Search/Reader/Zread pool.

## Image generation and other pay-go media

Coding Plan Vision provides understanding, not image generation. Z.AI's
general API exposes:

```text
POST https://api.z.ai/api/paas/v4/images/generations
```

with models including `glm-image` and CogView-4. Current prices are $0.015 per
GLM-Image result and $0.01 per CogView-4 result. See [GLM-Image][zai-image] and
[current pricing][zai-pricing].

The correct architecture is:

- Image understanding: Coding Plan Vision MCP.
- Image generation: separate `zai_open_platform` credential and cost.
- No automatic fallback from plan quota to pay-go generation.
- Grok's generic `image_gen` tool can select Z.AI only when that separate
  provider is configured.

The UI must not imply that GLM-Image generation is included in the Coding Plan.

## Usage and quota reporting

ZCode displays:

- Five-hour prompt pool.
- Weekly quota.
- Monthly MCP quota.
- Per-model token consumption.
- Search, Reader, and other MCP calls.
- Reset times and remaining allowance.

See [ZCode Usage Stats][zcode-usage]. Current community clients obtain the same
data through:

```text
GET https://api.z.ai/api/monitor/usage/quota/limit
GET https://api.z.ai/api/monitor/usage/model-usage?startTime=...&endTime=...
```

The first response currently uses an envelope with `code`, `msg`, `success`,
and `data.limits`. Limit records may contain:

- `type`: `TOKENS_LIMIT` or `TIME_LIMIT`.
- `unit` and `number` describing the window.
- `usage`, `currentValue`, and `remaining`.
- `percentage`.
- `usageDetails` with model or MCP tool codes.
- `nextResetTime` as epoch milliseconds.

Multiple `TOKENS_LIMIT` records should be classified by window length: the
shorter one is normally the five-hour pool and the longer one the weekly
limit. `TIME_LIMIT` has been used as the monthly MCP marker; its numeric unit
must not be interpreted naively as a literal one-minute window. See the
current [CodexBar Z.AI provider][codexbar-zai].

`/usage` should display:

```text
Z.AI Coding Plan - Pro

5-hour pool       38% used    resets in 2h 14m
Weekly quota      61% used    resets Jul 20 11:42
Monthly MCP       212 / 1000  resets Aug 1

MCP calls
  Web Search       91
  Web Reader      103
  Zread             18

Recent model usage
  GLM-5.2         8.3M tokens
  GLM-4.7         1.1M tokens
```

Only display reset timestamps actually returned by the service. No confirmed
banked-reset or rollover contract was found. Model usage is optional and
non-fatal. The monitoring schema is undocumented, so unknown fields and
missing limit categories must not break the TUI.

BigModel China team usage can require query selectors and
`Bigmodel-Organization` / `Bigmodel-Project` headers. That should be a later,
explicit team-account feature rather than guessed from empty responses.

## API-equivalent cost display

Current pay-as-you-go prices per million tokens are:

| Model | Uncached input | Cached input | Output |
| --- | ---: | ---: | ---: |
| GLM-5.2 | $1.40 | $0.26 | $4.40 |
| GLM-5-Turbo | $1.20 | $0.24 | $4.00 |
| GLM-4.7 | $0.60 | $0.11 | $2.20 |

Pay-go Web Search is currently $0.01 per use. These values come from
[Z.AI pricing][zai-pricing] and must be stored as dated metadata rather than
assumed permanent constants.

Grok's cost gates should distinguish:

- Actual request billing: Coding Plan quota.
- API-equivalent token cost for entertainment/comparison.
- Savings from `cached_tokens` at the current cached-input rate.
- Search/Reader/Zread quota calls, which are not token prices.
- Coding Plan quota multipliers, which are not dollar prices.

If a model alias or price is unknown, show the token counts and mark the cost
estimate unavailable rather than silently using another model's rate.

## Error classification and retry policy

Z.AI has HTTP status codes plus business codes in JSON. Important business
codes include:

| Code | Meaning | Grok behavior |
| ---: | --- | --- |
| 1001 | Missing authentication header | Reload scoped key once, then request login |
| 1002-1004 | Invalid or expired authentication | Reload once; static key cannot refresh |
| 1113 | Insufficient pay-go balance | Stop; diagnose wrong endpoint or balance |
| 1211 | Model does not exist | Refresh catalog once, then stop |
| 1261 | Prompt exceeds maximum | Compact or report; do not blind retry |
| 1305 | Rate limited | Bounded exponential backoff with jitter |
| 1308 | Usage limit reached | Stop and show returned reset time |
| 1309 | Coding Plan expired | Stop and direct user to subscription status |
| 1310 | Weekly or monthly limit exhausted | Stop and show returned reset time |
| 1311 | Plan lacks model entitlement | Refresh catalog once, then explain |
| 1312 | Model overloaded | Bounded backoff or user-selected model switch |
| 1313 | Fair-use restriction | Stop; no automated retry |

See the official [error reference][zai-errors]. Authentication, monitoring, and
MCP paths must also recognize envelopes such as:

```json
{
  "code": 1001,
  "msg": "Authentication parameter not received in Header, unable to authenticate",
  "success": false
}
```

even when the HTTP status is 200.

Provider recovery must never invoke xAI AuthManager. Static Coding Plan keys
have no refresh operation. Quota and account errors must terminate the attempt
instead of entering a generic retry loop.

## Mapping into this fork

1. Add `ProviderId::ZaiCodingPlan` and a Z.AI plan API-key credential source in
   [`provider.rs`](../../crates/codegen/xai-grok-sampling-types/src/provider.rs).
2. Add `ZaiOpenPlatform` separately so pay-go and subscription requests cannot
   share endpoints accidentally.
3. Store the plan key under a region-specific scope such as
   `zai::coding-plan::global`.
4. Add a Chat Completions sampler profile using the canonical coding endpoint.
5. Replace provider-specific `is_openai_codex()` branching with capability or
   request-policy abstractions where additional providers now need equivalent
   hooks.
6. Add the Z.AI request shaper for `thinking`, `reasoning_effort`, and
   `tool_stream`.
7. Make reasoning round trips exact across tool calls, resume, and compaction.
8. Add authenticated model discovery and entitlement-aware selection.
9. Generalize the web configuration currently selected in
   [`spawn.rs`](../../crates/codegen/xai-grok-shell/src/session/acp_session_impl/spawn.rs)
   so Z.AI Search/Reader/Zread can use dynamic scoped authentication.
10. Validate the existing `rmcp` transport and add a Z.AI compatibility path if
    protocol negotiation fails.
11. Add Vision MCP startup, doctor, file-path routing, permissions, and package
    version reporting.
12. Generalize `/usage` around provider quota snapshots and add dated
    API-equivalent pricing.
13. Add provider-scoped login, logout, model selection, redaction, and docs.

## Delivery sequence

### Phase 1: provider and credentials

- Provider and credential identities.
- Global/BigModel region identity.
- Login, logout, status, and redaction.
- Exact URL/header tests proving provider isolation.

### Phase 2: catalog and inference

- Authenticated dynamic model catalog.
- GLM-5.2 capability and thinking-level metadata.
- Chat Completions request shaping.
- Text, reasoning, tool streaming, usage, stops, and cancellation.
- Provider-owned error classification and bounded recovery.

### Phase 3: sessions and caching

- Exact preserved-reasoning representation.
- Tool-chain round trips.
- Resume and compaction boundaries.
- Cached-token and cache-rate accounting.

### Phase 4: remote web services

- Search, Reader, and Zread MCP adapters.
- Native Grok tool/result/citation normalization.
- Reader versus local WebFetch policy.
- MCP protocol compatibility and reconnect behavior.
- Monthly quota visibility.

### Phase 5: vision and media

- Node/package doctor and local Vision MCP lifecycle.
- Safe attachment-to-file-path routing.
- Image/video tool permissions and output rendering.
- Separate pay-go Z.AI image-generation provider.

### Phase 6: usage and UX

- Five-hour, weekly, and monthly MCP quota snapshots.
- Per-tool and optional per-model activity.
- Reset timestamps.
- API-equivalent token, cache, search, and image costs.
- User guide and troubleshooting documentation.

### Phase 7: live qualification

- Headless inference with a real approved Coding Plan key.
- Long reasoning/tool loops and parallel calls.
- Session resume, compaction, and repeated-prefix caching.
- Search, Reader, Zread, and Vision MCP handshakes and calls.
- Rate-limit, quota-exhaustion, model-overload, and account-switching behavior.
- Full log and telemetry redaction review.

## Acceptance gates

Tests must cover:

- Exact Coding Plan request URL and `Authorization` header.
- No xAI, Codex, Kimi, or Open Platform headers on plan requests.
- Simultaneous xAI, Codex, Kimi, Z.AI plan, and Z.AI pay-go credentials.
- Region isolation and logout isolation.
- Authenticated model discovery, stale fallback, and entitlement refresh.
- GLM-5.2 1M context without the Anthropic `[1m]` alias.
- Every thinking alias and its actual high/max/off mapping.
- Exact, unmodified `reasoning_content` across tool calls.
- Preserved reasoning through session persistence and active-chain compaction.
- Fragmented and parallel streamed tool calls using `tool_stream`.
- Text, reasoning, final usage, `[DONE]`, stop reasons, and cancellation.
- Cached-token parsing and API-equivalent cost calculations.
- HTTP errors and HTTP 200 business-error envelopes.
- No retry loops for authentication, expiry, entitlement, or exhausted quota.
- Bounded retry/backoff for rate limits, overload, network errors, and 5xx.
- Five-hour, weekly, and MCP quota schema variants and reset timestamps.
- MCP version negotiation, session ID, initialized notification, tool listing,
  tool calls, reconnects, and cancellation.
- Search result normalization, citations, deduplication, and domain filters.
- Reader extraction, local WebFetch fallback, SSRF, redirects, and quota errors.
- Zread public-repository validation and result rendering.
- Vision Node/package checks, image path routing, video limits, and permissions.
- Separation of plan Vision from pay-go image generation.
- Complete credential, header, identity, session, request, and response
  redaction.

## Remaining live-test requirements

No credential was accessed or printed during this research. Before declaring
the provider production-ready, test with an approved Coding Plan account:

- Authenticated `/models` contents and capability metadata.
- Exact GLM-5.2 request and streamed response shapes.
- Tool-call argument streaming with `tool_stream=true`.
- Byte-exact preserved reasoning accepted on a follow-up tool turn.
- Cache hits on repeated stable prefixes.
- `/usage` monitoring payloads for the user's tier.
- Search, Reader, and Zread MCP negotiation through the current Rust client.
- Vision MCP install, startup, file-path analysis, and quota effects.
- Error bodies for invalid key, model entitlement, quota exhaustion, overload,
  and expired subscription.

These live checks must remain credential-gated and must not record API keys,
raw authorization headers, private account identifiers, prompts, preserved
reasoning, or sensitive tool results.

[zai-overview]: https://docs.z.ai/devpack/overview
[zai-tools]: https://docs.z.ai/devpack/tool/others
[zai-faq]: https://docs.z.ai/devpack/faq
[zai-model-switching]: https://docs.z.ai/devpack/latest-model
[zai-chat-completion]: https://docs.z.ai/api-reference/llm/chat-completion
[zai-thinking]: https://docs.z.ai/guides/capabilities/thinking-mode
[zai-cache]: https://docs.z.ai/guides/capabilities/cache
[zai-search-mcp]: https://docs.z.ai/devpack/mcp/search-mcp-server
[zai-reader-mcp]: https://docs.z.ai/devpack/mcp/reader-mcp-server
[zai-zread-mcp]: https://docs.z.ai/devpack/mcp/zread-mcp-server
[zai-vision-mcp]: https://docs.z.ai/devpack/mcp/vision-mcp-server
[zai-reader-api]: https://docs.z.ai/api-reference/tools/web-reader
[zai-search-api]: https://docs.z.ai/api-reference/tools/web-search
[zai-image]: https://docs.z.ai/guides/image/glm-image
[zai-pricing]: https://docs.z.ai/guides/overview/pricing
[zai-errors]: https://docs.z.ai/api-reference/api-code
[zcode-configuration]: https://zcode.z.ai/en/docs/configuration
[zcode-usage]: https://zcode.z.ai/en/docs/usage-stats
[codex-zai-mcp-issue]: https://github.com/openai/codex/issues/5619
[zai-sdk-python]: https://github.com/zai-org/z-ai-sdk-python
[zai-coding-plugins]: https://github.com/zai-org/zai-coding-plugins
[glm-5-github]: https://github.com/zai-org/GLM-5
[opencode-github]: https://github.com/anomalyco/opencode
[codexbar-github]: https://github.com/steipete/CodexBar
[codexbar-zai]: https://github.com/steipete/CodexBar/blob/e6b2ea490cec47d122ba14e32bfc41e53d98422c/docs/zai.md
[zai-usage-helper]: https://github.com/nniicckk6/zai-extention
