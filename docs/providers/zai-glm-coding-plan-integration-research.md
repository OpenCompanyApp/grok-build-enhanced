# Z.AI GLM Coding Plan integration research

> Status: **implemented, experimental, and initially live-qualified**. The
> runtime provider, isolated API-key auth, authenticated catalog, Chat
> Completions inference, exact reasoning replay, streamed tool calls, usage UX,
> Search/Reader/Zread MCP, and opt-in pinned Vision MCP are checked in.
> Deterministic credential-free tests cover the provider contracts, and an
> entitled account completed the synthetic live matrix documented below on
> 2026-07-19. The source revisions remain the reviewed implementation
> provenance; broader failure-mode and interactive qualification is still open.
>
> Private provider correspondence must stay outside the repository and is not
> public evidence of support or endorsement. Public distribution of this
> provider remains gated until Z.AI's official supported-tool documentation is
> updated.

## Executive conclusion

Z.AI GLM Coding Plan is implemented as a first-class experimental subscription
provider alongside xAI, OpenAI Codex, and Kimi Code. Its runtime contract is:

- Provider ID: `zai_coding_plan`.
- Credential source: an isolated Z.AI Coding Plan API key.
- Credential scope: `zai::coding-plan::global`.
- Protocol: OpenAI-compatible Chat Completions.
- Base URL: `https://api.z.ai/api/coding/paas/v4`.
- Dynamic model discovery from authenticated `GET /models`.
- Dedicated Coding Plan Search, Reader, Zread, and opt-in Vision MCP adapters.
- Provider-specific quota reporting from the monitoring endpoints used by
  current Z.AI clients.
- No `zai_open_platform` runtime is shipped: pay-go inference and image
  generation remain explicitly outside this provider.
- Grok Build retains its existing agent loop, tools, sessions, subagents,
  compaction, permissions, headless mode, ACP, and TUI.

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
- [anomalyco/opencode][opencode-github] only as an interoperability reference,
  reviewed and latest-fetched at
  `849c2598abc7d2b40261e74b5826bc74ffc78308`. The 2026-07-21 audit found
  generic context-overflow, attachment-wire, and Custom Mistral-ID behavior but
  no Z.AI-specific runtime contract change.
- [steipete/CodexBar][codexbar-github] at
  `f8636cb37eb0f96d261604ee94e6481496aadfeb`.
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
| Vision MCP | Local `npx --yes @z_ai/mcp-server@0.1.4` | Opt-in, exact-pinned stdio MCP backed by GLM-4.6V |
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

Implemented UX:

```text
grok login --provider zai-coding-plan
grok logout --provider zai-coding-plan
grok models --provider zai-coding-plan
grok -m zai-coding-plan/<entitled-model-id>
```

Login reads the key from `Z_AI_API_KEY` or bounded standard input without
terminal echo, validates it against the Coding Plan catalog before writing, and
stores it under `zai::coding-plan::global`. Logout removes only that record and
its credential-bound model cache. Selecting a Z.AI model never replaces xAI,
Codex, Kimi, custom-provider, Open Platform, or BigModel China credentials.
ACP advertises a dedicated Z.AI Coding Plan auth method using the same flow.

`Z_AI_API_KEY` is the only environment fallback. The implementation does not
accept `ZHIPU_API_KEY`, because global and China-region credentials must remain
explicitly separate. The key is dynamically injected into inference, MCP,
usage, and the opt-in local Vision child; it is not copied into `config.toml` or
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

OpenCode was consulted only as transport research for this request shape. It
is not a runtime provider in this fork. The implementation is independently
owned and the ledger-recorded source revision remains provenance rather than a
runtime dependency.

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
- Local fetch is a useful fallback when MCP extraction, protocol, transport, or
  service handling fails. Authentication, quota, and explicit rejection errors
  remain visible.

Runtime routing follows the active `zai-coding-plan/<model-id>` provider
identity; no separate Z.AI endpoint or static credential configuration is
accepted. `--disable-web-search` removes Search while leaving the independently
authenticated Reader and Zread capabilities available, subject to existing web
policy and WebFetch configuration.

Reader can handle ordinary public HTML and fall back to WebFetch on
extraction, protocol, transport, or service failure. Authentication, quota, and
explicit provider rejection errors remain visible rather than being hidden by
fallback. Before sending any URL to the remote Reader, Grok must still reject
unsupported schemes, credentials in URLs, localhost, private/link-local
addresses, and prohibited redirects.

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
[openai/codex issue 5619][codex-zai-mcp-issue]. Grok therefore uses a narrow
provider-owned compatibility transport rather than routing these services
through the generic `rmcp` client. The 2026-07-19 live matrix completed the full
handshake and a tool call on all three endpoints. It also found that the live
Search catalog advertises `web_search_prime` while the public guide names
`webSearchPrime`; the adapter accepts only those two exact spellings and invokes
whichever one the authenticated `tools/list` response advertised.

The compatibility transport:

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
npx --yes @z_ai/mcp-server@0.1.4
```

Environment:

```text
Z_AI_API_KEY=<scoped Coding Plan key>
Z_AI_MODE=ZAI
```

The audited npm release is `0.1.4`, with recorded integrity
`sha512-jPLBKJaTIy7HGYI0VuAaFJIjU3dq5z09CYZNr3QYoHYhCQ2dr5D6qp93oEVxNyvex643dICB7WloHbph2EzlVg==`.
Its package metadata declares Node 18+, while the official guide requires Node
22+; Grok enforces the stricter requirement. `zai_vision_doctor` reports Node,
`npx`, the exact package pin, and the recorded integrity without resolving or
printing credentials. Exact pinning is intentional: the integration never
executes `@latest`. See [Vision MCP][zai-vision-mcp].

Available tools include:

- `ui_to_artifact`.
- `extract_text_from_screenshot`.
- `diagnose_error_screenshot`.
- `understand_technical_diagram`.
- `analyze_data_visualization`.
- `ui_diff_check`.
- `analyze_image` in the pinned npm package (`image_analysis` in older
  official examples).
- `analyze_video` for MP4, MOV, or M4V content up to 8 MiB (`video_analysis`
  in older official examples).

Outside Claude Code, pasted images commonly bypass Vision MCP and are sent
directly to the selected model. Grok deliberately persists normalized
attachments under the private session directory, invokes the pinned Vision MCP,
and replaces inline image content with a textual description before Coding Plan
inference. Model-visible Vision tools are read-only and exist only when
`GROK_ZAI_VISION_MCP=1`; local paths must canonicalize beneath the workspace or
session roots.

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

The implementation is split across focused provider boundaries:

1. `ProviderId::ZaiCodingPlan`, `ZaiCodingPlanApiKey`, and the
   `zai::coding-plan::global` record establish isolated identity and storage.
2. Authenticated catalog discovery maps only returned Coding Plan models into
   the `zai-coding-plan/` namespace and binds the cache to an opaque credential
   record.
3. A provider-owned Chat Completions shaper enforces the canonical Coding Plan
   endpoint, sensitive bearer-only headers, `thinking`, normalized
   `reasoning_effort`, singular `tool_stream`, and the 128-function limit.
4. The shared conversation representation replays provider reasoning without
   introducing separators, while session routing preserves provider identity
   through resume, model switching, compaction, subagents, headless mode, and
   ACP.
5. A narrow Z.AI Streamable HTTP MCP client implements initialization, session
   header propagation, initialized notification, tool listing, tool calls, JSON
   and SSE responses, bounded bodies, fixed-shape errors, and dynamic auth for
   Search, Reader, and Zread.
6. Reader is layered behind Grok's existing URL and SSRF validation and keeps
   local WebFetch as the ordinary protocol/transport/service fallback.
7. Vision is explicit opt-in, exact-pinned, Node-checked, process-isolated, and
   rooted to workspace/session media. Automatic attachments are transcribed
   before text-only Coding Plan inference.
8. `/usage` projects returned five-hour, weekly, monthly MCP, reset, model, and
   tool information without guessing pay-go spend; `/usage manage` is
   provider-specific.
9. No `ZaiOpenPlatform` or BigModel China runtime identity is shipped. Their
   credentials, endpoints, and pay-go image generation cannot be selected by
   this provider.

## Delivery status

### Implemented and deterministically qualified

- Provider and credential identity, login/logout, redaction, storage, and
  environment fallback.
- Authenticated model catalog, cache binding, model selection, context/reasoning
  metadata, and canonical Chat Completions inference.
- Text, exact reasoning replay, fragmented/parallel streamed function calls,
  final usage, stop/error handling, and bounded retry classification.
- Session resume, active-chain compaction behavior, model switching, subagents,
  headless mode, and ACP provider routing.
- Search MCP, Reader MCP with protected local fallback, and prefixed Zread
  tools, all with provider-scoped dynamic auth.
- Opt-in Vision MCP doctor, exact package pin/integrity, bounded stdio
  handshake, process cleanup, safe media paths, automatic attachments, and
  eight provider media-analysis tools.
- Five-hour, weekly, monthly MCP, recent activity, reset-time, and management
  UX.

### Deliberately outside this provider

- Z.AI Open Platform pay-go inference or image generation.
- BigModel China Coding Plan or team-organization credentials.
- API-equivalent dollar estimates when no maintained, dated price mapping is
  available. The UI reports them as unavailable rather than borrowing another
  provider's rates.

### Entitled live qualification (2026-07-19)

A synthetic, telemetry-disabled matrix used disposable Grok homes and workspaces
and retained no authenticated response captures. It qualified:

- isolated login, authenticated discovery of eight entitled catalog entries,
  logout, and preservation of unrelated provider scopes;
- selectable `glm-4.7`, `glm-5-turbo`, and `glm-5.2` inference, streamed
  reasoning, local read/write calls, a two-read parallel-call request, headless
  resume/model switching, and an agent-profile model override;
- the Rust usage client and quota parser against the account's returned limit
  shape;
- complete Search, Reader, and Zread Streamable HTTP handshakes and calls,
  including all three prefixed Zread tools;
- the opt-in Vision doctor and an actual local-image analysis through the exact
  package pin; and
- ACP inference, manual session compaction, post-compaction context recovery,
  and namespaced live model switching.

An extended GLM 5.2 final pass then qualified:

- invalid-key login rejection without credential/cache writes or credential
  echoing;
- exact JSON inference, streamed reasoning, local read/write, sequential
  read-then-write, and two parallel reads completed in exactly two sampling
  rounds (one multi-call response and one final response);
- multi-turn resume with a later tool call, profile routing, logout cleanup, and
  provider-qualified usage attribution to GLM 5.2;
- three headless turns (two resumed) over an approximately 35 KiB stable
  prefix, with provider-reported cache-read counts of 21,184, 22,720, and
  22,784 tokens;
- Search, Reader, deliberate loopback/SSRF rejection, all three Zread tools,
  Vision doctor, explicit image analysis, automatic image-attachment routing,
  and actionable failure when Vision opt-in was absent;
- ACP reasoning and local tool traffic before manual compaction, context
  recovery after a model round trip away from and back to GLM 5.2, persisted
  namespaced selection, and the session-bound billing extension; and
- valid returned quota rows plus recent GLM 5.2 activity.

The credential was supplied only over stdin or sensitive request headers. A
post-run exact-value scan found no credential in captured stdout or stderr; raw
model reasoning, tool results, session IDs, account metadata, and provider
payloads were not retained in the repository.

### Remaining live qualification

- Interactive TUI behavior, live subagent execution, substantially longer
  reasoning/tool loops, and heavier repeated-compaction pressure.
- Rate-limit, quota-exhaustion, overload, model-entitlement, expiry, and account
  switching responses; unlike invalid-key rejection, these were not forced
  against the live account.
- Quota-effect reconciliation across repeated Search, Reader, Zread, and Vision
  calls.
- A broader credential-bearing log and telemetry review under those unavailable
  failure conditions, without retaining authenticated payloads.

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

The source research itself accessed no credential. The dated synthetic matrix
above later exercised an entitled account without printing the key or retaining
raw authenticated payloads. Before declaring the provider production-ready,
still qualify:

- byte-exact preserved reasoning acceptance across a live multi-tool replay,
  beyond the deterministic replay suite and streamed-reasoning smoke test;
- substantially longer repeated-compaction and tool-loop stress beyond the
  qualified stable-prefix cache and tool-bearing compaction cases;
- quota deltas attributable to Search, Reader, Zread, and Vision calls;
- interactive TUI and live subagent behavior; and
- fixed-shape handling for model-entitlement loss, quota exhaustion, overload,
  rate limits, account changes, and expired subscriptions. Invalid-key login
  rejection is already live-qualified.

These checks must remain credential-gated and must not record API keys, raw
authorization headers, private account identifiers, prompts, preserved
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
[codexbar-zai]: https://github.com/steipete/CodexBar/blob/f8636cb37eb0f96d261604ee94e6481496aadfeb/docs/zai.md
[zai-usage-helper]: https://github.com/nniicckk6/zai-extention
