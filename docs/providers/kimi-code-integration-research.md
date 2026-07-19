# Kimi Code plan provider reference and research

> Status: **implemented, experimental** as of 2026-07-19. The runtime follows
> the API-key, catalog, inference, usage, hosted-search, and hosted-fetch design
> below. Initial credential-gated qualification now covers the currently
> entitled Chat Completions catalog, K3 inference, usage, and model-driven web
> tools; the broader protocol/session matrix remains before this can be called
> production-ready. On 2026-07-18 the ignored Kimi Code and OpenCode checkouts
> were fast-forwarded and their exact latest fetched revisions were recorded in
> [`UPSTREAM_VERSIONS.md`](../../UPSTREAM_VERSIONS.md) and
> [`fork/manifest.json`](../../fork/manifest.json). The reviewed revisions
> remain pinned separately: a latest-fetched revision is a review queue, not an
> accepted implementation baseline.
>
> This repository is an unofficial Grok Build fork. Its Kimi Code integration
> identifies itself truthfully as Grok Build and does not imply endorsement by
> Moonshot AI.

## Executive conclusion

Kimi Code is implemented as a first-class provider alongside xAI and OpenAI
Codex while retaining Grok Build's existing agent loop, tools, sessions,
permissions, subagents, ACP, headless mode, and TUI. The implementation uses:

- Runtime provider ID: `kimi_code`.
- Credential source: a Kimi Code subscription API key created in the Kimi Code
  Console, stored under the isolated `kimi::code` scope.
- Provider identity and protocol are separate: the provider is always
  `kimi_code`, while each authenticated catalog entry selects either
  OpenAI-compatible Chat Completions or Anthropic-compatible Messages.
- Canonical Kimi REST base URL: `https://api.kimi.com/coding/v1`; a Messages
  adapter must account for Anthropic SDK-style base-path and beta routing
  semantics without changing provider identity.
- Dynamic model discovery from authenticated `GET /models`; the catalog's
  `protocol` field is authoritative and must never be guessed from a model ID.
- Grok Build retains its existing agent loop, tools, sessions, subagents,
  compaction, permissions, and TUI.
- Kimi-specific handling is added for preserved reasoning, cache affinity,
  streaming usage, quotas, web services, and model entitlements.

The implementation deliberately does not ship official-CLI-style OAuth. The
open-source CLI has a correct device-code flow, but Kimi's public third-party
documentation directs other coding agents to use a Console API key, and its
policy explicitly forbids spoofing client identity. Grok Build must identify
itself honestly. See the [Kimi Code overview][kimi-overview] and
[community guidelines][kimi-guidelines].

## Source snapshot

Moonshot AI's current upstream reference implementation is
[MoonshotAI/kimi-code][kimi-code-github], a TypeScript monorepo. This research reviewed:

- `MoonshotAI/kimi-code` main snapshot
  `ada523ae6a06726c02ffcc7f35d01ee2216d309c`, whose package version was
  `0.27.0`, dated 2026-07-17. The `0.27.0` release tag points to a different
  commit, so this is intentionally described as a main snapshot rather than
  the release commit.
- Latest fetched Kimi Code main
  `4f3c7240c4adc7c748e536bf578e468c1b5bcd7b`, dated 2026-07-18. The three
  intervening commits change web-command foreground behavior, permission copy,
  and the VS Code package release; they do not change the provider, catalog,
  authentication, inference, usage, hosted-search, fetch, or files adapters.
- OpenCode's reviewed reference remains
  `1d2a7b4c860f6a29eb90bdda07757b2adf34ab61`; latest fetched is
  `b8142c7aa8f88222873fb79d636e312e28037c2d`. Its relevant drift confirms that
  explicit Anthropic-compatible effort metadata, including K3 `max`, should be
  passed through rather than replaced with a legacy token-budget guess.
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

The official pages are currently in a product transition and disagree on the
quota pool: the Kimi Code membership page still describes a coming dedicated
coding plan and shared membership limits, while the current Help Center pricing
page says Kimi Code already has its own separate credit pool. Prices, plan
names, pool semantics, and quota multipliers must therefore remain dated
product documentation, not compiled behavior. Runtime truth must come from
authenticated usage and model responses. See
[current pricing][kimi-membership-pricing] and the
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
| Anthropic Messages | Effective request `POST https://api.kimi.com/coding/v1/messages`; catalog-routed models currently use beta Messages semantics | Public third-party protocol and required whenever `/models` selects `protocol: anthropic` |
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

## Authentication and CLI

The shipped provider UX is:

```text
KIMI_API_KEY="<plan-key>" grok login --provider kimi-code
grok models --provider kimi-code
grok -m kimi-code/<model-id>
grok logout --provider kimi-code
```

For safer automation, the login command also accepts the API key on piped
standard input. It deliberately refuses an interactive echoed prompt. Login
validates the key against authenticated `GET /models` before storing it under
the separate `kimi::code` scope in `~/.grok/auth.json`. The model cache is bound
to a random local credential-record ID, contains no API key, and is removed on
logout. `KIMI_API_KEY` is also a runtime fallback, but running the login command
is required to persist the authenticated catalog for normal model selection.
This consumes the user's subscription; it is not a pay-as-you-go Open Platform
key.

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

The current official client enables preserved thinking by default when
thinking is on. For the Kimi Chat Completions protocol it sends
`thinking: {"type":"enabled","effort":...,"keep":"all"}` and replays even an
explicitly empty `reasoning_content` field on historical assistant tool-call
messages. Disabling thinking sends `thinking.type = "disabled"` and, according
to the product documentation, changes the effective backing model.

One compatibility seam needs live verification: Kimi Code documentation also
describes K3 using `reasoning_effort`, while the current official Kimi adapter
sends the `thinking` object. The implementation should follow authenticated
catalog metadata, be mock-tested for both shapes, and be confirmed with a real
plan key before release.

### Anthropic Messages compatibility requirements

`GET /models` can select `protocol: anthropic` for an entitled model, including
when the credential is a hand-configured plan API key. That selection must not
be silently routed through Chat Completions. The Messages adapter must:

- Keep `ProviderId::KimiCode` while selecting `ApiBackend::Messages`.
- Use Kimi's `x-api-key` authentication shape, suppress `Authorization`, and
  add only the required Anthropic protocol version/beta metadata plus the
  truthful Grok Build `User-Agent`.
- Reproduce the effective `/coding/v1/messages` beta route used by the official
  client without treating an SDK base-URL convention as a provider URL.
- Send Kimi-compatible thinking enable/disable and `output_config.effort`
  values from the catalog's valid/default efforts.
- Preserve prior thinking blocks. The current official client represents
  `keep = "all"` on this route with a beta `context_management`
  `clear_thinking_20251015` edit.
- Use `metadata.user_id` as the Messages cache-affinity analogue of
  `prompt_cache_key`, populated only from a Grok-generated opaque session key.
- Preserve tool-use/result pairing, thinking signatures, cache-token usage,
  cancellation, and beta stream events through the existing normalized
  conversation model.

Both protocols should be designed in the first inference milestone. A
developer-preview build may ship Chat Completions first only if it fails closed
with an actionable error for every catalog entry that selects Anthropic. General
availability should require live qualification of both protocols unless real
plan-key catalogs prove no entitled selectable entry uses Messages.

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

The implementation consequently:

1. Preserves Grok's existing `web_search` and `web_fetch` tool UX.
2. Enables Kimi-hosted `POST /search` automatically for a Kimi session unless
   the user or managed policy explicitly disables web access.
3. Tries Kimi-hosted `POST /fetch` first and retains Grok's local SSRF-safe
   fetcher as the fallback for transport and ordinary service failures.
4. Applies Grok's URL validation and SSRF checks before hosted fetch, and maps
   unsupported navigation commands to explicit errors rather than inventing
   Kimi operations.
5. Preserves bounded source URLs and citations through the normal Grok event
   stream.
6. Surfaces `401`, membership (`402`/`403`), and quota/concurrency (`429`)
   failures distinctly; hosted fetch does not silently mask these with local
   fallback.
7. Sends only a truthful Grok `User-Agent` and bearer auth. It does not send the
   official client's `X-Msh-Platform`, device identity, or tool-call identity
   headers.

Hosted search is intentionally **not default-disabled**: it is part of the
experimental provider contract and is ready for the credential-gated live test
specified below.

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

## Mapping implemented in this fork

1. `ProviderId::KimiCode` and `CredentialSourceId::KimiCodeApiKey` establish a
   first-class provider/credential boundary.
2. Provider-aware auth storage keeps `kimi::code` separate from `openai::codex`
   and every xAI scope, preserves unknown records, writes atomically, and marks
   wire header values sensitive.
3. The Kimi sampler constructor and session binder seal the canonical endpoint,
   select bearer versus `x-api-key` from the catalog protocol, and reject xAI,
   Codex, official-Kimi-CLI, or user-supplied protected headers.
4. Static Kimi keys return “not refreshable” after `401`; retries never route
   through xAI or Codex authentication.
5. Dedicated Chat Completions and beta Messages policies reuse Grok's normalized
   conversation and streaming layers. They shape completion limits, thinking,
   effort, preserved-thinking context management, streaming usage, cache
   affinity, body limits, and redacted diagnostics.
6. Authenticated `GET /models` is cached under an opaque local credential ID.
   The per-model `protocol`, context window, reasoning menu, image capability,
   and user-visible identity are authoritative; model IDs are namespaced as
   `kimi-code/<id>`.
7. Model switching, persistence, compaction, subagents, memory isolation, and
   web resources preserve provider ownership without replacing Grok's native
   runtime.
8. Hosted search is enabled for Kimi sessions. Hosted fetch is preferred with a
   local safe fallback, except auth/membership/quota failures remain explicit.
9. `GET /usages` feeds the interactive `/usage` presentation with weekly and
   rolling-window limits, reset hints, and Extra Usage wallet/cap data. A dated
   API-equivalent cost estimate remains intentionally unavailable while the
   published hosted-search prices conflict.
10. Login, logout, and model discovery coexist with xAI and Codex credentials.
    Catalog image-input capability is honored; Kimi video upload and standalone
    generation are not claimed or implemented.

## Delivery and qualification status

### Phase 1: provider and credentials — implemented

- Provider identity and scoped API-key storage.
- Login, logout, account switching, and redaction.
- Exact URL and header tests proving provider isolation.

### Phase 2: models and inference — implemented, Chat live-qualified

- Authenticated dynamic catalog.
- Entitlement-aware model picker.
- Catalog-selected Kimi Chat Completions and beta Messages shaping.
- Preserved thinking, reasoning effort, streaming, parallel tools, stops,
  cancellation, and usage parsing across both protocols.
- The 2026-07-19 entitled catalog exposed three Chat Completions models; K3
  completed a real headless turn through the provider-isolated ACP auth path.
  Messages remains contract-tested but awaits an entitled catalog entry for a
  live protocol qualification.

### Phase 3: caching and sessions — implemented at the transport/session layer

- Stable cache keys across resume.
- Cached-token accounting.
- Model and effort cache invalidation.
- Provider-aware compaction and context limits.

### Phase 4: tools and media — hosted web and image input implemented

- Catalog-gated image input uses Grok's existing multimodal path.
- Web search and fetch use hosted-first behavior, explicit auth/quota errors,
  bounded citations, redirect blocking, and the SSRF-safe local fetch fallback.
- Kimi video upload through `/files` remains unimplemented; the fork does not
  expose Kimi video or image generation.

### Phase 5: usage and UX — plan usage implemented

- Interactive `/usage` shows plan windows, reset hints, and Extra Usage wallet
  and monthly-cap data; `/usage manage` opens the Kimi Code Console.
- API-equivalent costs remain labeled unavailable instead of choosing between
  conflicting hosted-search prices or conflating plan usage with API spend.
- The user guide documents the experimental contract and troubleshooting
  boundaries.

### Phase 6: live qualification — initial matrix completed 2026-07-19

Completed without logging credentials, authenticated bodies, prompts, account
identifiers, or local credential-record identifiers:

- Piped-stdin login and authenticated discovery of three entitled models.
- K3 headless inference through ACP with no xAI-auth fallthrough.
- Successful `/usages`, hosted `/search`, and hosted `/fetch` response-shape
  checks.
- One real model-driven `web_search` call and one `web_fetch` call, each with a
  nonempty, non-error tool result persisted through the ordinary agent loop.
- Capture scans found no credential or identity material in process output or
  persisted tool events.

Still pending are long tool loops, resume, compaction, measurable cache hits,
rate-limit/quota qualification, same-provider model switching, media input, and
a live Messages route if an entitled catalog exposes one. Device OAuth remains
out of scope unless Moonshot approves third-party client use.

## Acceptance gates

Tests should cover:

- Exact request URLs and headers.
- No xAI or Codex-specific headers on Kimi requests.
- Simultaneous xAI, Codex, and Kimi credentials.
- Account switching without replacing another provider's login.
- Dynamic K3, Standard, and HighSpeed discovery, entitlements, and per-model
  Chat Completions versus Messages routing.
- Fail-closed behavior for unknown catalog protocols and unqualified beta
  Messages routes.
- Reasoning preservation across tool calls, including empty preserved-thinking
  fields and Messages context-management replay.
- Interleaved parallel tool-call streams.
- Correct stop, truncation, cancellation, and retry exhaustion behavior.
- Invalid-key versus entitlement `401` classification.
- Transient versus quota `429` classification.
- Cache hits and model or effort cache invalidation.
- Resume and compaction under 256K, 1M, and 2 MB body constraints.
- Image input, format conversion, limits, and model capability gating.
- Hosted web search, citations, fetch fallback, and SSRF protection.
- Weekly, five-hour, reset, and Extra Usage payload drift.
- Clear unavailable labeling for API-equivalent cost until the hosted-tool
  pricing conflict is resolved and a dated estimator is implemented.
- Complete credential, token, identity, and request-body redaction.

The 2026-07-19 live qualification confirmed that `/usages`, `/search`, and
`/fetch` accept the stored third-party Kimi Code plan API key. Remaining
credential-gated unknowns include video upload, an entitled Messages route, and
the longer-running session, cache, and quota cases listed above. Any future
qualification must keep the same no-credential, no-identity, and no-authenticated-
payload logging boundary.

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
