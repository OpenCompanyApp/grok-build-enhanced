# Kimi Code plan integration research

> Status: research and implementation design only. Kimi Code support described
> here is not yet implemented. The source revisions below are prior research
> already recorded in [`UPSTREAM_VERSIONS.md`](../../UPSTREAM_VERSIONS.md); this
> reconciliation did not fetch or re-review them and leaves the upstream ledger
> unchanged.
>
> This repository is an unofficial Grok Build fork. Any future Kimi Code
> integration must identify itself truthfully and must not imply endorsement by
> Moonshot AI.

## Executive conclusion

Kimi Code could fit this fork as a possible future provider alongside xAI and
OpenAI Codex. Every identifier and command in this document is proposed design
notation only: the checked-in provider and CLI enums contain no Kimi variant,
and no runtime path can select or load Kimi credentials. The recommended future
integration path is:

- Proposed provider ID: `kimi_code`.
- Credential source: a Kimi Code subscription API key created in the Kimi Code
  Console.
- Protocol: OpenAI-compatible Chat Completions.
- Base URL: `https://api.kimi.com/coding/v1`.
- Dynamic model discovery from authenticated `GET /models`.
- Grok Build retains its existing agent loop, tools, sessions, subagents,
  compaction, permissions, and TUI.
- Kimi-specific handling is added for preserved reasoning, cache affinity,
  streaming usage, quotas, web services, and model entitlements.

The initial implementation should not ship official-CLI-style OAuth. The
open-source CLI has a correct device-code flow, but Kimi's public third-party
documentation directs other coding agents to use a Console API key, and its
policy explicitly forbids spoofing client identity. Grok Build must identify
itself honestly. See the [Kimi Code overview][kimi-overview] and
[community guidelines][kimi-guidelines].

## Source snapshot

Moonshot AI's current upstream reference implementation is
[MoonshotAI/kimi-code][kimi-code-github], a TypeScript monorepo. This research
reviewed:

- `MoonshotAI/kimi-code` main snapshot
  `ada523ae6a06726c02ffcc7f35d01ee2216d309c`, whose package version was
  `0.27.0`, dated 2026-07-17. The `0.27.0` release tag points to a different
  commit, so this is intentionally described as a main snapshot rather than
  the release commit.
- The legacy Python [MoonshotAI/kimi-cli][kimi-cli-github] at
  `4a550effdfcb29a25a5d325bf935296cc50cd417`, release `1.49.0`, dated
  2026-07-16.

The current documentation treats the Python client as legacy, although its
repository still receives release maintenance. Current `kimi-code` behavior is
therefore the primary source of truth; the older client is useful for historical
edge cases.

The checked-out reference repositories live in the gitignored `inspiration/`
directory. The current source is MIT-licensed and the legacy source is
Apache-2.0. If code is ported rather than behavior independently implemented,
the relevant copyright, license, attribution, and prominent modification
requirements must be preserved.

## Kimi Code subscription versus Kimi Open Platform

These are separate products with non-interchangeable credentials:

| Product | Base URL | Billing | Intended use |
| --- | --- | --- | --- |
| Kimi Code subscription | `https://api.kimi.com/coding/v1` | Membership quota | Interactive coding agents |
| Kimi Open Platform | `https://api.moonshot.ai/v1` | Pay per token | Applications, automation, and enterprise integration |

Using an Open Platform key against Kimi Code, or the reverse, returns an
authentication error. Kimi Code is expressly permitted in third-party coding
agents for personal, interactive use, but not batch automation, resale, shared
reverse proxies, or repackaging as a service. See the
[third-party agent instructions][kimi-third-party] and
[usage guidelines][kimi-guidelines].

### Current subscription snapshot

The shared-membership pricing published at the time of research is:

| Plan | Monthly | Annual effective | Kimi Code allocation |
| --- | ---: | ---: | ---: |
| Adagio | Free | - | None |
| Moderato | $19 | $15/month | 1x |
| Allegretto | $39 | $31/month | 5x |
| Allegro | $99 | $79/month | 15x |
| Vivace | $199 | $159/month | 30x |

Kimi says a separate coding-focused membership system is coming, with dedicated
coding quota and removal of the shared monthly total cap. Prices, plan names,
and quota multipliers must therefore remain documentation, not compiled
behavior. See [current pricing][kimi-membership-pricing] and the
[transition notice and quota rules][kimi-membership].

Quota behavior at the time of research includes:

- A weekly quota refreshing every seven days from the subscription date.
- A rolling five-hour rate window.
- Shared usage across devices and API keys.
- No rollover of unused weekly quota.
- Optional Extra Usage after subscription quota exhaustion.
- An optional monthly Extra Usage spending cap.
- Approximately 300-1,200 requests per five-hour window, depending on plan.
- Up to 30 concurrent streams.

The request counts are approximate product guidance, not constants to encode.

## Models and thinking

The subscription service currently exposes three stable aliases:

| Subscription model ID | Backing family | Context | Thinking | Entitlement |
| --- | --- | ---: | --- | --- |
| `k3` | Kimi K3 | 256K or up to 1M | `low`, `high`, `max`; default `max` | Moderato+; 1M on Allegretto+ |
| `kimi-for-coding` | Kimi K2.7 Code Standard | 256K | Always on | All Kimi Code members |
| `kimi-for-coding-highspeed` | Kimi K2.7 Code HighSpeed | 256K | Always on | Allegretto+ |

HighSpeed is the same coding model at roughly 5-6x output speed but consumes
about 3x subscription quota. It is a distinct model alias, not a generic HTTP
fast-mode flag. Grok's model picker or fast-mode control should switch between
`kimi-for-coding` and `kimi-for-coding-highspeed` only when both are returned by
the authenticated catalog. K3 has no documented HighSpeed alias.

K3's effort compatibility mapping is:

- `ultra`, `xhigh`, `max` -> `max`.
- `medium`, `high` -> `high`.
- `minimum`, `light`, `low` -> `low`.
- Unset -> `max`.
- `none` -> thinking disabled.
- Unknown values -> HTTP 400.

Disabling thinking on K3 or K2.7 currently routes the request to K2.6. The TUI
should warn that this changes the effective model rather than presenting it as
merely "K3 without reasoning." Switching either model or reasoning effort
invalidates the existing prompt cache. See the
[current model contract][kimi-models].

Model IDs must come from the account-authenticated catalog. The official client
parses context length, image and video capability, tool support, thinking mode,
valid effort levels, default effort, and protocol choice from `GET /models`
rather than maintaining a stale list. See the current client implementation in
[`managed-kimi-code.ts`][kimi-managed-models].

## Endpoint and authentication map

| Purpose | Request | Stability |
| --- | --- | --- |
| Chat Completions | `POST https://api.kimi.com/coding/v1/chat/completions` | Public Kimi Code contract |
| Anthropic Messages | `POST https://api.kimi.com/coding/v1/messages` | Public, but unnecessary for initial Grok support |
| Model catalog | `GET https://api.kimi.com/coding/v1/models` | Current official client behavior |
| Subscription usage | `GET https://api.kimi.com/coding/v1/usages` | Current official client behavior; not a stable public API |
| Hosted web search | `POST https://api.kimi.com/coding/v1/search` | Current official managed-client behavior |
| Hosted URL fetch | `POST https://api.kimi.com/coding/v1/fetch` | Current official managed-client behavior |
| Device authorization | `POST https://auth.kimi.com/api/oauth/device_authorization` | Official CLI behavior |
| OAuth token and refresh | `POST https://auth.kimi.com/api/oauth/token` | Official CLI behavior |

For the recommended future API-key path, every request would need:

```text
Authorization: Bearer <Kimi Code Console API key>
User-Agent: grok-build/<version> (...)
```

No xAI headers, Codex headers, `ChatGPT-Account-ID`, or official Kimi CLI
identity should appear.

The current Kimi CLI adds `X-Msh-Platform: kimi_code_cli` and stable device
headers for its own OAuth requests. Grok must not send `kimi_code_cli`. See the
current [`identity.ts` implementation][kimi-identity].

## Authentication recommendation

A future first-class UX could be:

```text
grok login --provider kimi-code
grok logout --provider kimi-code
grok models --provider kimi-code
```

These are proposed commands; the current binary does not accept the
`kimi-code` provider.

Login should direct the user to the Kimi Code Console, prompt for the
one-time-visible plan API key, validate it against `/models`, and store it under
a separate scope such as `kimi::code`. This consumes the user's subscription;
it is not a pay-as-you-go Open Platform key.

The official client's OAuth implementation includes:

- RFC 8628 device-code login.
- `authorization_pending` and `slow_down` handling.
- Rotating refresh tokens.
- In-process refresh coalescing.
- Cross-process refresh locking.
- Atomic mode-0600 token persistence.
- Peer-process token reread after lock acquisition.
- Revoked-token tombstones.

See [`oauth.ts`][kimi-oauth], [`oauth-manager.ts`][kimi-oauth-manager], and
[`storage.ts`][kimi-oauth-storage]. Its client ID and `kimi_code_cli` platform
identity belong to the official client. Device OAuth should only be added after
Moonshot confirms third-party use of that public client or supplies an approved
Grok Build client and platform identity.

## Chat Completions compatibility requirements

Grok already supports the general Chat Completions shape, but Kimi needs a
dedicated request and stream adapter.

Required details:

- Send `max_completion_tokens`, not legacy `max_tokens`.
- Enable `stream_options: {"include_usage": true}`.
- Parse text and `reasoning_content` deltas.
- Preserve `reasoning_content` on historical assistant tool-call messages.
- Handle multiple parallel tool calls whose argument fragments are interleaved
  by index.
- Preserve tool-call IDs and normalize them to Kimi's 64-character limit.
- Reject duplicate tool names before sending.
- Normalize and dereference tool JSON Schemas.
- Parse usage from top-level `usage` or `choices[0].usage`.
- Map `stop`, `tool_calls`, `length`, and `content_filter` correctly.
- Capture `x-trace-id` for diagnostics without treating it as authentication
  material.
- Never log bodies containing images, prompts, credentials, or preserved
  reasoning.

Preserved reasoning is mandatory. Kimi returns HTTP 400 if thinking is enabled
and an assistant tool-call message lacks `reasoning_content`. The official
adapter deliberately emits even an empty reasoning field when preserved
thinking requires it. See [`kimi.ts`][kimi-provider] and the
[official error reference][kimi-errors].

One compatibility seam needs live verification: Kimi Code documentation
describes K3 using `reasoning_effort`, while the current official Kimi adapter
sends a `thinking` object through Chat Completions. The implementation should be
model-aware and mock-tested, then confirmed with a real plan key before release.

## Caching

Kimi makes `prompt_cache_key` particularly important. Its API documentation
says coding agents should use a stable session or task identifier, preserve it
across exit and resume, and provide it for good Kimi Code Plan cache behavior.
See the [Chat Completions contract][kimi-chat-api].

Recommended behavior:

- Derive the cache key from Grok's stable conversation or session identifier.
- Reuse it when a saved session resumes.
- Namespace it by provider, model family, and thinking configuration.
- Rotate the namespace when the model or reasoning effort changes.
- Preserve historical message bytes as consistently as possible.
- Parse `cached_tokens` from either the top-level usage object or
  `prompt_tokens_details.cached_tokens`.
- Display cache-hit tokens and cache-hit percentage in `/usage`.
- Add a headless test that sends the same long-prefix session twice and verifies
  that the second request reports cached tokens.

The official usage parser is in [`openai-common.ts`][kimi-usage-parser].

## Web search and fetch

Kimi's official CLI implements web access as host-side tools, which matches
Grok's architecture:

- Search: `POST /search` with `{"text_query":"..."}`.
- Fetch: `POST /fetch` with `{"url":"..."}`.
- Search returns title, URL, snippet, date, and site name.
- Fetch requests `text/markdown`.
- Both correlate the model tool call through `X-Msh-Tool-Call-Id`.
- Hosted fetch falls back to a local SSRF-protected fetcher on managed-service
  failure.

See [`moonshot-web-search.ts`][kimi-web-search] and
[`moonshot-fetch-url.ts`][kimi-fetch-url].

The official client automatically provisions `/search` and `/fetch` for
managed OAuth, while its hand-configured API-key refresh only updates model
aliases and leaves services untouched. These endpoints are therefore not yet a
guaranteed public contract for plan API keys.

Grok should consequently:

1. Preserve its existing `WebSearch`, `open`, `find`, and `WebFetch` tool UX.
2. Add a Kimi-hosted backend, but enable it only after live API-key verification.
3. Retain Grok's local SSRF-safe fetch fallback.
4. Map `open` and `find` through fetched content rather than inventing
   unsupported Kimi operations.
5. Preserve source URLs and citations through the normal Grok event stream.
6. Surface quota and auth failures distinctly instead of silently looping.
7. Keep an alternate configurable search backend if hosted Kimi search is
   unavailable.

Kimi Open Platform also has a `$web_search` built-in, but its documentation
currently warns that this functionality is being updated. Using Grok's explicit
host tools gives better observability, permissions, citation handling, and
retry control.

There is a pricing inconsistency: Kimi's Help Center says pay-as-you-go web
search costs $0.004 per call, while an older Platform page says $0.005 and
labels itself outdated. The API-equivalent cost display should leave hosted
search cost marked unknown until Kimi resolves this. See
[Help Center pricing][kimi-api-pricing] and
[Platform web-search pricing][kimi-web-pricing].

## Images, video, and image generation

K3 and K2.7 support image input. K2.7 also advertises video input. The model
catalog should remain authoritative.

Image input can use:

- Standard `image_url` content parts.
- Public URLs where allowed.
- `data:image/...;base64,...`.
- `ms://<file_id>` after upload.

Local filesystem paths cannot be sent directly. Grok should read, validate,
resize or compress where needed, and encode them as data URIs. The Kimi Code
API also imposes a 2 MB total JSON request-body limit, which may become the real
constraint before the advertised token window.

The official provider uploads video through `POST /files` with `purpose=video`,
then sends `ms://<file-id>`. See [`kimi-files.ts`][kimi-files].

There is no documented public Kimi image-generation API endpoint. Current Kimi
model capabilities are input-oriented vision, not bitmap output. The correct
architecture is:

- Kimi reads and reasons over images natively.
- Kimi may call Grok's existing `image_gen` tool like any other tool.
- Image generation remains authenticated and billed through its own image
  provider.
- The product must not claim that the Kimi Code subscription itself includes
  API image generation.

The official Kimi API endpoint inventory contains Chat Completions, models,
token estimation, balance, and files, but no image-generation endpoint. See the
[API overview][kimi-api-overview].

## Usage and hypothetical API cost

The official client calls `GET /usages` and parses:

- Weekly limit.
- Rolling-window limits such as five-hour usage.
- Used, remaining, and total.
- Reset timestamps or TTLs.
- Extra Usage balance.
- Extra Usage monthly cap.
- Extra Usage monthly spend.
- Currency.

Its parser intentionally accepts multiple field spellings because the payload
has changed over time. See [`managed-usage.ts`][kimi-managed-usage].

Grok `/usage` should show:

- Current-session input, output, and cached tokens.
- Cache-hit percentage.
- Weekly and five-hour plan usage when `/usages` accepts the credential.
- Reset times.
- Extra Usage wallet and cap.
- A console link when account-level usage is unavailable.
- A separately labeled API-equivalent estimate.

Current API-equivalent prices per million tokens are:

| Current underlying model | Cached input | Uncached input | Output |
| --- | ---: | ---: | ---: |
| Kimi K3 | $0.30 | $3.00 | $15.00 |
| Kimi K2.7 Code | $0.19 | $0.95 | $4.00 |
| Kimi K2.7 Code HighSpeed | $0.38 | $1.90 | $8.00 |

Sources: [K3 API pricing][kimi-k3-pricing],
[K2.7 pricing][kimi-k27-pricing], and the
[K2.7 public pricing explainer][kimi-k27-explainer].

Because `kimi-for-coding` is a stable rolling alias, the UI must state that the
estimate uses the current K2.7 equivalent as of a particular date. It cannot
assume that alias will always point to K2.7.

## Error and retry policy

Kimi status codes require message-aware classification:

- `401`, invalid or revoked API key: do not retry.
- `401`, model or plan entitlement: do not refresh; explain missing K3, 1M, or
  HighSpeed entitlement.
- `402`, membership verification unavailable: bounded retry.
- `403`, weekly or billing-cycle quota exhausted: do not retry.
- `403`, access terminated: do not retry.
- `429`, engine overload or concurrency: retry with bounded backoff.
- `429`, five-hour or monthly quota exhausted: do not retry; show reset and
  usage information.
- `400`, missing reasoning, duplicate tools, body over 2 MB, context overflow,
  or invalid image: fix locally or report.
- `5xx`: bounded transient retry.

Static API keys have no refresh operation. If OAuth is eventually approved,
only OAuth credentials should receive provider-scoped reload, refresh, and
retry-once behavior.

This distinction is essential to avoid blind retry loops. See the
[Kimi Code error reference][kimi-errors].

## Mapping into this fork

1. Add `ProviderId::KimiCode` and a Kimi subscription API-key credential source
   beside the existing definitions in
   [`provider.rs`](../../crates/codegen/xai-grok-sampling-types/src/provider.rs).
2. Store credentials under `kimi::code`, entirely separate from
   `openai::codex` and xAI.
3. Add `SamplerConfig::kimi_code(model)` using Chat Completions and the
   canonical Kimi Code URL. The existing request-auth hook in
   [`config.rs`](../../crates/codegen/xai-grok-sampler/src/config.rs) is the
   correct seam.
4. Generalize provider-owned 401 recovery. It is currently explicitly gated to
   Codex in
   [`request_task.rs`](../../crates/codegen/xai-grok-sampler/src/actor/request_task.rs).
   The abstraction should support "provider owns recovery," while Kimi API keys
   return "not refreshable."
5. Add a Kimi Chat Completions request shaper and stream parser.
6. Add an authenticated dynamic Kimi catalog.
7. Make model switching, spawn, compaction, media, usage, and web configuration
   capability-driven. Adding a matching set of `is_kimi_code()` branches beside
   every `is_openai_codex()` branch would become brittle.
8. Add Kimi-hosted search and fetch backends behind an experimental capability
   until plan-key access is verified.
9. Extend `/usage` and API-equivalent pricing with Kimi-specific data.
10. Add login, logout, and model UX without replacing the user's xAI or Codex
    credentials.

## Delivery sequence

### Phase 1: provider and credentials

- Provider identity and scoped API-key storage.
- Login, logout, account switching, and redaction.
- Exact URL and header tests proving provider isolation.

### Phase 2: models and inference

- Authenticated dynamic catalog.
- Entitlement-aware model picker.
- Kimi Chat Completions shaping.
- Reasoning, streaming, parallel tools, stops, cancellation, and usage parsing.

### Phase 3: caching and sessions

- Stable cache keys across resume.
- Cached-token accounting.
- Model and effort cache invalidation.
- Provider-aware compaction and context limits.

### Phase 4: tools and media

- Image input and optional video upload.
- Web search and fetch with hosted-first and safe fallback behavior.
- Citation, permission, cancellation, and tool-call correlation tests.

### Phase 5: usage and UX

- Plan `/usage`, resets, and Extra Usage.
- API-equivalent cost estimates with dated pricing metadata.
- User guide, experimental-contract disclaimer, and troubleshooting guidance.

### Phase 6: live qualification

- Headless live testing with a real Kimi Code plan key.
- Long tool loops, session resume, compaction, cache hits, rate limits, model
  switching, web access, and media input.
- Device OAuth only after Moonshot approval.

## Acceptance gates

Tests should cover:

- Exact request URLs and headers.
- No xAI or Codex-specific headers on Kimi requests.
- Simultaneous xAI, Codex, and Kimi credentials.
- Account switching without replacing another provider's login.
- Dynamic K3, Standard, and HighSpeed discovery and entitlements.
- Reasoning preservation across tool calls.
- Interleaved parallel tool-call streams.
- Correct stop, truncation, cancellation, and retry exhaustion behavior.
- Invalid-key versus entitlement `401` classification.
- Transient versus quota `429` classification.
- Cache hits and model or effort cache invalidation.
- Resume and compaction under 256K, 1M, and 2 MB body constraints.
- Image input, format conversion, limits, and model capability gating.
- Hosted web search, citations, fetch fallback, and SSRF protection.
- Weekly, five-hour, reset, and Extra Usage payload drift.
- API-equivalent cost calculations and stale-pricing warnings.
- Complete credential, token, identity, and request-body redaction.

The remaining credential-gated unknowns are whether `/usages`, `/search`,
`/fetch`, and video uploads accept a third-party Kimi Code plan API key exactly
as they accept managed OAuth. They must be live-tested without printing or
recording the credential before those features are considered production-ready.

[kimi-overview]: https://www.kimi.com/code/docs/en/
[kimi-guidelines]: https://www.kimi.com/code/docs/en/kimi-code/community-guidelines.html
[kimi-code-github]: https://github.com/MoonshotAI/kimi-code
[kimi-cli-github]: https://github.com/MoonshotAI/kimi-cli
[kimi-third-party]: https://www.kimi.com/code/docs/en/third-party-tools/other-coding-agents.html
[kimi-membership-pricing]: https://www.kimi.com/help/membership/membership-pricing
[kimi-membership]: https://www.kimi.com/code/docs/en/kimi-code/membership.html
[kimi-models]: https://www.kimi.com/code/docs/en/kimi-code/models.html
[kimi-errors]: https://www.kimi.com/code/docs/en/kimi-code/error-reference.html
[kimi-chat-api]: https://platform.kimi.ai/docs/api/chat
[kimi-api-overview]: https://platform.kimi.ai/docs/api/overview
[kimi-api-pricing]: https://www.kimi.com/help/kimi-api/api-pricing
[kimi-web-pricing]: https://platform.kimi.ai/docs/pricing/tools
[kimi-k3-pricing]: https://platform.kimi.ai/docs/pricing/chat-k3
[kimi-k27-pricing]: https://platform.kimi.ai/docs/pricing/chat-k27-code
[kimi-k27-explainer]: https://www.kimi.com/resources/kimi-k2-7-code-pricing
[kimi-managed-models]: https://github.com/MoonshotAI/kimi-code/blob/ada523ae6a06726c02ffcc7f35d01ee2216d309c/packages/oauth/src/managed-kimi-code.ts#L415-L521
[kimi-identity]: https://github.com/MoonshotAI/kimi-code/blob/ada523ae6a06726c02ffcc7f35d01ee2216d309c/packages/oauth/src/identity.ts#L18-L97
[kimi-oauth]: https://github.com/MoonshotAI/kimi-code/blob/ada523ae6a06726c02ffcc7f35d01ee2216d309c/packages/oauth/src/oauth.ts#L117-L289
[kimi-oauth-manager]: https://github.com/MoonshotAI/kimi-code/blob/ada523ae6a06726c02ffcc7f35d01ee2216d309c/packages/oauth/src/oauth-manager.ts#L250-L390
[kimi-oauth-storage]: https://github.com/MoonshotAI/kimi-code/blob/ada523ae6a06726c02ffcc7f35d01ee2216d309c/packages/oauth/src/storage.ts#L1-L117
[kimi-provider]: https://github.com/MoonshotAI/kimi-code/blob/ada523ae6a06726c02ffcc7f35d01ee2216d309c/packages/kosong/src/providers/kimi.ts#L116-L200
[kimi-usage-parser]: https://github.com/MoonshotAI/kimi-code/blob/ada523ae6a06726c02ffcc7f35d01ee2216d309c/packages/kosong/src/providers/openai-common.ts#L157-L188
[kimi-web-search]: https://github.com/MoonshotAI/kimi-code/blob/ada523ae6a06726c02ffcc7f35d01ee2216d309c/packages/agent-core/src/tools/providers/moonshot-web-search.ts#L39-L129
[kimi-fetch-url]: https://github.com/MoonshotAI/kimi-code/blob/ada523ae6a06726c02ffcc7f35d01ee2216d309c/packages/agent-core/src/tools/providers/moonshot-fetch-url.ts#L32-L127
[kimi-files]: https://github.com/MoonshotAI/kimi-code/blob/ada523ae6a06726c02ffcc7f35d01ee2216d309c/packages/kosong/src/providers/kimi-files.ts#L31-L150
[kimi-managed-usage]: https://github.com/MoonshotAI/kimi-code/blob/ada523ae6a06726c02ffcc7f35d01ee2216d309c/packages/oauth/src/managed-usage.ts#L1-L225
