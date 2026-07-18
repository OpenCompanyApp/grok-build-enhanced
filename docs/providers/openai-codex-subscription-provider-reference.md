# OpenAI Codex subscription provider reference

> Status: implemented, experimental provider reference. Reconciled on
> 2026-07-18 against the checked-in fork implementation, tests, and normalized
> standalone-search contract. [`UPSTREAM_VERSIONS.md`][upstream-versions]
> records the reviewed external revisions.
>
> This is an unofficial Grok Build fork. Direct use of the ChatGPT Codex
> backend follows the compatibility contract implemented here; it is not a
> stable, general-purpose OpenAI API contract and does not imply OpenAI or xAI
> endorsement.

## Executive summary

OpenAI Codex subscription support is a first-class, provider-scoped adapter in
this fork. It lets an entitled ChatGPT account authenticate with OAuth, discover
its current Codex models, and use ChatGPT plan credits while retaining Grok
Build's agent loop, sessions, tools, permissions, subagents, compaction, and
TUI.

It is deliberately **not** implemented as:

- an OpenAI Platform API key;
- a custom model inferred from a base URL;
- a static access token copied from `~/.codex/auth.json`;
- a fallback through the xAI `AuthManager`; or
- an embedded copy of Codex app-server.

### Source authority

For “what this fork does,” the checked-in Rust code and tests are authoritative.
For the external service contract, use official OpenAI documentation and the
public `openai/codex` source, then verify with an entitled-account smoke test.

At this document's reconciliation date, `UPSTREAM_VERSIONS.md` records:

- `openai/codex` reviewed and latest-fetched at
  `315195492c80fdade38e917c18f9584efd599304`;
- OpenCode last reviewed at
  `1d2a7b4c860f6a29eb90bdda07757b2adf34ab61`, with
  `efb6cc2d4bf6332eb156709795d2b3a649198b65` only the latest fetched revision;
  and
- fork compatibility version `0.144.0`.

The focused diff from the prior reviewed Codex revision
`18110b810f0a328147f6cd85e6f1ab6414927366` to `315195...` contained no changes
to `codex-api/src/search.rs`, `codex-api/src/endpoint/search.rs`, or the
standalone web-search extension. The normalized reviewed contract is committed
at `fork/contracts/openai_codex_search_contract.json`; the offline checker also
supports extracting the immutable Git object without checking it out. OpenCode
remains an interoperability and provenance reference, not a runtime provider
identity.

The shortest operational sequence is:

```sh
cargo build -p xai-grok-pager-bin

# The build artifact is target/debug/xai-grok-pager. Installed builds use `grok`.
target/debug/xai-grok-pager login --provider openai-codex
target/debug/xai-grok-pager models --provider openai-codex
target/debug/xai-grok-pager -m openai-codex/<entitled-model-slug>
```

Use device authorization on a headless machine:

```sh
target/debug/xai-grok-pager login --provider openai-codex --device-auth
```

Disconnect only ChatGPT Codex, preserving any xAI login:

```sh
target/debug/xai-grok-pager logout --provider openai-codex
```

## Identity and endpoint map

| Concern | Implemented value |
| --- | --- |
| Provider ID | `openai_codex` |
| Credential source | `openai_codex_subscription` |
| Auth-store scope | `openai::codex` |
| Model namespace | `openai-codex/<slug>` |
| ChatGPT Codex base | `https://chatgpt.com/backend-api/codex` |
| Responses | `POST /responses` |
| Model catalog | `GET /models?client_version=<compatibility-version>` |
| Standalone search | `POST /alpha/search` |
| Image generation | `POST /images/generations` |
| Image editing | `POST /images/edits` |
| Subscription usage | `GET https://chatgpt.com/backend-api/wham/usage` |
| Usage management | `https://chatgpt.com/codex/settings/usage` |
| OAuth issuer | `https://auth.openai.com` |
| OAuth client originator | `grok_build_codex` |
| Current client compatibility version | `0.144.0` |

The provider and credential source are explicit Rust types in
[`xai-grok-sampling-types/src/provider.rs`][provider-types]. The auth/model CLI
accepts only `xai` and `openai-codex`; Kimi, Z.AI, and OpenCode are not CLI or
runtime provider identities. The generic `custom` runtime variant is distinct
and cannot acquire Codex subscription credentials.

A caller-selected base URL cannot turn another provider into Codex or cause
Codex credentials to be sent elsewhere. Production inference, catalog, search,
usage, and image clients allow only their canonical ChatGPT destinations;
narrow loopback seams exist only under tests or explicit test-support builds.

## Ownership boundaries

The split is intentional:

| Grok Build owns | ChatGPT Codex supplies |
| --- | --- |
| Agent and tool loop | Model inference |
| Conversation/session persistence | Entitled model catalog |
| Permission checks and sandboxing | Reasoning and service-tier metadata |
| Local file, terminal, patch, MCP, skill, and hook tools | Subscription search and image endpoints |
| Subagent coordinator and nesting limits | Plan usage/rate-limit state |
| TUI rendering, stop/cancel behavior, and compaction orchestration | OAuth identity and refreshed tokens |

Do not replace this boundary with Codex app-server without an explicit scope
change. App-server is an important compatibility reference, but it would
replace rather than extend major Grok Build behavior.

## Authentication

### Browser OAuth

`grok login --provider openai-codex` uses Authorization Code OAuth with PKCE
S256 and a cryptographically random state value. The client:

1. Binds a loopback callback on port `1455`, falling back to `1457`.
2. Registers `http://localhost:<port>/auth/callback` for the pending attempt.
3. Generates a 64-byte PKCE verifier, its SHA-256 challenge, and 32 random
   state bytes.
4. Opens the authorization URL at `auth.openai.com`.
5. Rejects missing, mismatched, or replayed state before exchanging the code.
6. Exchanges the code without following redirects.
7. Extracts and validates ChatGPT identity/workspace claims before storage.

The callback expires after ten minutes. A stray callback with the wrong state
does not consume the legitimate pending login. Callback responses send
`Cache-Control: no-store` and `Referrer-Policy: no-referrer` so the local result
page is not cached or used as referrer state.

### Device authorization

`--device-auth` implements the public Codex device flow:

1. `POST /api/accounts/deviceauth/usercode` obtains a user code and device
   authorization ID.
2. The user visits `https://auth.openai.com/codex/device`.
3. Grok polls `POST /api/accounts/deviceauth/token` at the bounded server
   interval.
4. It validates the server-returned PKCE verifier/challenge pair.
5. It exchanges the authorization code at `/oauth/token`.

The complete device flow times out after fifteen minutes and supports explicit
cancellation. A missing device endpoint is reported as unavailable rather than
silently falling back to a different credential mechanism.

The OAuth implementation lives in
[`xai-grok-shell/src/auth/codex/oauth.rs`][codex-oauth]. The official product
also documents browser and device login in [Codex authentication][openai-auth].

### Stored credential record

Codex credentials live inside `~/.grok/auth.json` under `openai::codex`. The
record contains:

- access, refresh, and ID tokens;
- access-token expiry and record timestamps;
- a monotonic revision/generation;
- a random local credential-record UUID;
- ChatGPT account ID and optional user ID;
- optional email and plan type;
- optional FedRAMP state; and
- unknown future fields preserved during read/modify/write.

The local UUID is random and not derived from a token, account, email, or user
ID. Secret wrappers always render as `<redacted>`. The auth file and persistent
provider lock are owner-only (`0600` on Unix), and writes use the existing
atomic scoped `AuthStore` path.

The implementation never imports or shares `~/.codex/auth.json`. That avoids
unsafe refresh-token rotation with another Codex client and prevents logout in
one product from silently mutating the other product's storage.

### Refresh and concurrent-process safety

An access token is refreshed when it expires within five minutes. Refresh is
serialized by:

- an in-process async mutex;
- a persistent kernel-backed `auth.json.openai-codex.lock`; and
- the auth file's atomic write lock.

Every outbound request revalidates the in-memory record against disk. A sibling
process logout or account switch therefore fails closed immediately instead of
continuing with an old bearer token.

Refresh responses may rotate the refresh token and may omit unchanged token
fields. Missing fields inherit from the previous valid record; identity must
remain unchanged. A different account, user, FedRAMP state, record UUID, or
unexpected generation is an account-change error rather than an opportunity to
adopt whichever credential happens to be current.

### Provider-scoped 401 recovery

Each credential-bearing Codex operation may recover one rejected request:

1. Pin the exact non-secret record UUID and generation that signed the request.
2. Reload the scoped credential from disk.
3. If a sibling already advanced the same record, adopt that generation.
4. Otherwise refresh once while holding the provider lock.
5. Retry once with a newly resolved auth snapshot.

A second 401 is terminal. A stale 401 cannot spend a newly rotated refresh
token twice, and recovery never enters xAI authentication. Streaming inference
is retried only before user-visible stream output has started; replaying a
partially consumed turn would duplicate text or tool calls.

Logout removes only the `openai::codex` scope, then attempts refresh- and
access-token revocation outside the locks. Local removal succeeds even if
network revocation is unavailable.

## Request headers and redaction

Every dynamic Codex request resolves a fresh auth snapshot and injects:

```text
Authorization: Bearer <access token>
ChatGPT-Account-ID: <workspace/account ID>
X-OpenAI-Fedramp: true                 # only for FedRAMP credentials
```

Product/compatibility clients additionally send the current `User-Agent`,
`originator: grok_build_codex`, and `version: 0.144.0`. Responses Lite requests
carry `x-openai-internal-codex-responses-lite: true`. Responses inference also
uses sensitive `session-id`, `thread-id`, and `x-client-request-id` values plus
an optional sensitive `x-codex-turn-state`; standalone search instead uses a
separate sensitive `x-codex-turn-metadata` header.

All credential and private request-identity headers are marked sensitive.
Arbitrary Codex response bodies are withheld from diagnostics; only bounded,
non-secret classifications such as status, usage limit, or encrypted-content
may surface. The following must never appear in logs, errors, hooks, trace
uploads, cache keys, or diagnostics:

- bearer, refresh, or ID tokens, including prefixes;
- ChatGPT account or user IDs;
- email addresses derived from identity tokens;
- FedRAMP state;
- session, thread, client-request, turn-state, or search turn-metadata values;
- raw authenticated response bodies; or
- raw credential-bearing header maps.

User `extra_headers` cannot override authorization, account, FedRAMP,
originator/version, Responses Lite, or Codex turn-state headers. Codex traffic
must not contain xAI-only `x-api-key`, `x-userid`, `x-email`, `x-grok-*`, or
`x-xai-*` headers.

## Endpoint allowlists, redirects, and response limits

Provider identity, destination, and credential binding are checked together.
Production clients are sealed to the OAuth paths on `auth.openai.com`, the
listed Codex paths under `https://chatgpt.com/backend-api/codex`, and the exact
`https://chatgpt.com/backend-api/wham/usage` usage URL. Loopback origins are
accepted only by scoped test/test-support seams. Every Codex client disables
redirect following so bearer/account/FedRAMP headers are never replayed to a
redirect target.

The checked-in application limits are not uniform:

| Operation | Timeout / size behavior |
| --- | --- |
| Responses inference and compaction | No provider-specific total response-body cap. Interactive SSE is guarded by the sampler's configurable idle timeout (300 seconds by default), not a total-byte limit. |
| Model catalog | 5-second request timeout; 8 MiB network body; 16 MiB on-disk cache envelope. |
| Subscription usage | 15-second request timeout; no provider-specific application body-size cap. |
| Standalone search | 60-second request timeout; 10 MiB response body, then bounded JSON depth/nodes/references/citations. |
| Image generation/edit response | 300-second total and 240-second read timeouts; 48 MiB response body, 32 MiB decoded image, 40 million pixels, and 8192-pixel maximum dimension. |

Input limits are separate from response limits. Structured search accepts a
16 KiB query or 64 KiB command object with bounded depth/nodes/domains; its
visible-history limits are listed below. Image editing accepts at most five
references, at most 24 MiB raw per reference and 48 MiB aggregate for Codex,
with an aggregate pixel cap of 178,956,970. Do not describe the uncapped
Responses or usage paths as if the catalog/search/image caps covered them.

## Model catalog and entitlements

Model discovery is authenticated and account-scoped. The client calls:

```text
GET https://chatgpt.com/backend-api/codex/models?client_version=0.144.0
```

It parses model visibility, priority, context and compaction limits, input
modalities, reasoning presets, default reasoning, service tiers, Responses Lite
support, tool mode, search capability, and multi-agent metadata. Only models
whose service metadata says they are visible in the picker are presented.

The catalog adapter stores `supports_image_input`, but the current attachment
pipeline does **not** consult that flag before adding user images to a Codex
turn. It is descriptive metadata today, not a client-side entitlement gate;
provider rejection remains possible for a selected model that cannot accept an
image.

There is no hardcoded entitlement fallback. A successful empty catalog is
authoritative. Network or authentication failure is distinct from “this
account has no models.”

### Catalog caching

- Freshness TTL: five minutes.
- Network response limit: 8 MiB.
- On-disk cache limit: 16 MiB.
- Conditional fetch: `ETag` and `If-None-Match`.
- Scope: a BLAKE3 digest over the opaque credential record, account/user
  identity, FedRAMP state, base URL, and client compatibility version.
- Publication: atomic and leased to the exact credential generation.

Raw account/user IDs are never used as cache filenames or persisted cache
scope values. Only a live authenticated `200` or matching `304` can publish or
relabel a catalog for the current generation. An old account's bytes cannot
become the offline fallback for a new account.

The implementation is in
[`xai-grok-shell/src/remote/openai_codex_catalog.rs`][codex-catalog].

### Current GPT-5.6 snapshot

The following is a dated compatibility snapshot from the official Codex model
catalog at `openai/codex` commit
`315195492c80fdade38e917c18f9584efd599304`. The authenticated runtime catalog
always wins.

| Model slug | Context | Default effort | Advertised efforts | Images | Fast | Multi-agent |
| --- | ---: | --- | --- | --- | --- | --- |
| `gpt-5.6-sol` | 372K | `low` | `low`, `medium`, `high`, `xhigh`, `max`, `ultra` | Yes | `priority` | v2 |
| `gpt-5.6-terra` | 372K | `medium` | `low`, `medium`, `high`, `xhigh`, `max`, `ultra` | Yes | `priority` | v2 |
| `gpt-5.6-luna` | 372K | `medium` | `low`, `medium`, `high`, `xhigh`, `max` | Yes | `priority` | v1 |

All three were marked `use_responses_lite: true` and `tool_mode:
code_mode_only` in that source snapshot. The “Images” column records catalog
metadata; it does not imply that this fork currently enforces that metadata
before forwarding attachments. Do not compile this table into model selection
logic. Use it only for regression testing and upstream-diff review.

## Responses transport

Inference uses the OpenAI Responses protocol at the ChatGPT Codex endpoint.
The existing sampler handles SSE text, reasoning, usage, function-call
arguments, function-call completion, search records, and terminal response
events.

Normal Codex defaults include:

- `stream: true` for interactive turns;
- `store: false`;
- encrypted reasoning content in the requested include set;
- removal of `temperature` from every Codex Responses request;
- a stable `prompt_cache_key` derived from conversation ID, falling back to
  session ID; and
- provider-specific reasoning/service-tier normalization at the final JSON
  boundary.

Temperature removal and prompt-cache-key insertion happen for Codex Responses
whether or not the selected model uses Responses Lite. The prompt cache key is
stable across OAuth refresh, 401 recovery, request IDs, and ordinary turns in
the same conversation. It intentionally changes with a new
conversation/session rather than including token or account material.

### Responses Lite shaping

When the selected catalog model advertises Responses Lite, the sampler
additionally:

- changes system-role input to developer-role input;
- strips unsupported image `detail` fields;
- moves Grok function schemas into a developer `additional_tools` input item;
- marks function schemas `strict: false`;
- moves instructions into a developer message;
- sets `parallel_tool_calls: false`;
- uses `tool_choice: "auto"` only when tools exist; and
- sets `reasoning.context: "all_turns"`.

Auxiliary tool-free requests retain an empty `additional_tools` item but omit
`tool_choice`. This matters for title generation and compaction, where `auto`
without a callable tool is rejected by the current backend.

The transformer is in
[`provider/openai_codex/responses.rs`][codex-responses]. It changes only the
wire contract; Grok's conversation store, registry, tool execution,
permissions, and loop remain authoritative.

### Turn state

The backend may return `x-codex-turn-state`. The sampler stores it against the
exact Grok request/turn ID and replays it only within that same turn. It is not
shared across turns, sessions, providers, or logs. Standalone search does not
receive it.

### Tool-call compatibility boundary

The official GPT-5.6 snapshot marks Sol, Terra, and Luna `code_mode_only` and
the official client normally exposes nested tools behind a V8-backed `exec`
custom tool plus `wait`. This fork intentionally does not port that runtime.
Instead it supplies Grok's existing tools directly through `additional_tools`.

This is the most important live-compatibility seam:

- Direct function calls continue through Grok's normal tool loop.
- A returned custom/freeform JavaScript call fails closed.
- It must never be treated as an xAI search call or executed by a shell/V8
  substitute.
- Every material Codex model-contract update requires a live function-tool
  smoke test with an entitled account.

## Reasoning effort, Max, and Ultra

The model picker and `/effort` accept only levels advertised by the current
catalog model. Extended levels are preserved even though the pinned
`async-openai` enum stops at `xhigh`:

- `max` is injected into the final Codex request JSON.
- `ultra` is displayed as an orchestration tier and sends the backend-supported
  `max` effort.
- Responses are normalized through a private metadata side channel so local
  state does not falsely report `xhigh` after using Max/Ultra.

For catalog models with multi-agent v2, Ultra also enables a turn-local
proactive delegation policy through Grok's existing `spawn_subagent` tool and
coordinator. Lower levels delegate only when explicitly requested. This does
not bypass Grok permission checks, nesting limits, worktree rules, or session
ownership. Luna's audited v1 metadata does not advertise Ultra.

The official [Codex model guidance][openai-models] describes Max as extra
single-task reasoning and Ultra as subagent-backed orchestration. The catalog,
not this paragraph, remains authoritative.

## Fast mode

Fast mode is a Codex subscription service tier, not a different model and not
an OpenAI API key billing mode.

```text
/fast             # toggle
/fast on
/fast off
/fast status
```

Persist a default with:

```toml
service_tier = "fast"

[features]
fast_mode = true
```

The UI/config name `fast` maps to the exact wire value
`service_tier: "priority"`. Explicit Standard is stored locally as `default`
but omitted from the request. Unknown tiers are omitted rather than forwarded.
Fast is available only when the authenticated model catalog advertises the
tier; it never affects xAI or custom-provider traffic.

The pre-existing 2026-07-17 product snapshot cites [official Fast mode
documentation][openai-speed] for about 1.5x speed and higher ChatGPT-credit
consumption: GPT-5.6 and GPT-5.5 at 2.5x Standard credits, GPT-5.4 at 2x. This
reconciliation did not re-check those multipliers. They are dated product
rules, not request constants; OpenAI API Priority pricing is a separate billing
contract.

## Web tools

### Modes, settings, and precedence

Codex sessions expose Grok's native `web_search` function without
`XAI_API_KEY`. The provider adapter posts to `{codex-base}/alpha/search`; it
does not expose Codex code mode, JavaScript execution, Codex exec, or a
provider-custom web tool. The mode defaults to `cached`:

| Mode | Provider wire value | Tool availability |
| --- | --- | --- |
| `cached` | `external_web_access: false` | `web_search` available |
| `indexed` | `external_web_access: "indexed"` | `web_search` available |
| `live` | `external_web_access: true` | `web_search` available |
| `disabled` | No request value | Codex `web_search` is not registered |

Configure the standalone adapter in `~/.grok/config.toml`:

```toml
[web_search]
mode = "cached"
search_context_size = "low" # low, medium, or high
allowed_domains = ["openai.com"]
blocked_domains = ["example.org"]

[web_search.user_location]
country = "US"
region = "California"
city = "San Francisco"
timezone = "America/Los_Angeles"

[web_search.image_settings]
max_results = 4
caption = true
```

Location is always sent as `type: "approximate"` and is omitted unless the
user configured it. `country` must be a two-letter code; `timezone` is locally
validated as a bounded IANA-style identifier. Grok does not infer location from
the IP address, OS, credentials, account metadata, or ambient timezone. The adapter always seals
`allowed_callers: ["direct"]`.

Use `--web-search-mode cached|indexed|live|disabled` to override local mode for
the process. `--search` is shorthand for live mode. Conflicting explicit CLI
values fail argument parsing. Resolution is fail-closed, in this order:

1. `--disable-web-search`, which removes both native web functions;
2. managed requirements, including a pinned mode and narrowing domain policy;
3. explicit CLI mode;
4. local configuration; and
5. the `cached` default.

Managed pins cannot be weakened by CLI flags. Blocklists are unioned and
subtract from every allowlist. Allowlist layers are intersected; exact domains
and subdomains retain the narrower common domain. Domains are normalized for
case, IDNA, and trailing dots, while IP literals, wildcard tricks, and an
explicitly empty intersection fail before network I/O. These settings affect
the Codex standalone-search adapter only; xAI and generic custom providers keep
their existing behavior.

### Native invocation and output budget

The command schema supports `search_query`, `image_query`, `open`, `click`,
`find`, PDF `screenshot`, `finance`, `weather`, `sports`, `time`, and
`response_length`. References returned by search may be passed only to the
standalone search operations. A literal URL sent to `web_search.open` stays on
the sealed provider endpoint; a URL sent to `web_fetch` stays in the local
fetch client. The shell never silently crosses or falls back between them.

When the current genuine user request explicitly requires current sources,
verification, citations, a URL lookup, or other clearly time-sensitive data,
the Codex turn must call `web_search` or `web_fetch` directly. If the model
stops without doing so, the shell injects one trusted corrective continuation.
A second no-tool stop ends normally with the stable
`required_browsing_not_invoked` failure. A real web call, including a failed
one, consumes the requirement and is never converted into a no-tool retry.
Synthetic reminders and external web text are not classified as user intent.

The shell—not the model—computes `max_output_tokens`. It reserves completion
runway below the compaction threshold, gives at most half of remaining safe
runway to web output, applies `short < medium < long` caps, clamps at 16,384,
and divides concurrent search calls deterministically in call order. Less than
1,024 useful tokens fails before network I/O with
`context_budget_exhausted`. Provider output is also truncated on a UTF-8
boundary if it exceeds the requested ceiling.

Search receives only a bounded visible-history projection:

- at most eight messages and 32 KiB total;
- one or two recent genuine user text messages;
- at most 4 KiB of assistant text between those messages; and
- no system/synthetic reminders, reasoning, tool traffic, credentials, images,
  or provider turn state.

A model-supplied `_grok_codex_context` field is always discarded. The shell
then injects fresh bounded context and sensitive `x-codex-turn-metadata` after
argument validation, even when no visible history exists. Search refuses
redirects, caps the response at 10 MiB, bounds result projection to 32
references, 64 citations, 4096 nodes, and depth 16, and owns exact-binding
one-shot 401 recovery. See [`web_search/backends/openai_codex.rs`][codex-search].

### Untrusted content and structured failures

Successful `web_search` and `web_fetch` results carry typed
`external_untrusted` provenance. The JSONL record stores original content plus
metadata; provider conversion adds the model-visible boundary, so marker text
is never parsed as trust state. Provenance survives resume, provider switches,
ordinary/verbatim compaction, flattened subagent context, ACP updates, and
post-tool hooks. Derived summaries are marked external-derived when any input
was external. ACP and hook metadata expose only source kind, derived state, and
a stable failure code—not bodies, URLs, queries, or credential state.

Both native web tools use bounded failure envelopes with `code`, `retryable`,
and safe advice. Stable codes include `domain_rejected`, `ssrf_blocked`,
`cross_host_redirect`, `unsupported_content`, `provider_unavailable`,
`reference_expired`, `authentication_required`, `response_too_large`,
`timeout`, `rate_limited`, `context_budget_exhausted`,
`required_browsing_not_invoked`, and `repeated_failure`.

Within one user turn, identical effective-tool/argument fingerprints share one
sanitized result. Successes and non-retryable failures replay without network.
A retryable failure gets at most one later network attempt; another identical
request returns `repeated_failure`. The ledger stores no raw URL, query, host,
or auth data and is cleared at turn exit. Telemetry contains aggregate dedup,
retry, repeated-failure, and local fetch cache counters only.

### Local web fetch

The provider-independent `web_fetch` keeps its own allowlist, DNS re-resolution,
body limits, MIME handling, same-host redirect validation, and SSRF policy.
Loopback is blocked by default. Local development may opt in with:

```toml
[toolset.web_fetch]
allow_loopback = true
```

That switch permits loopback only; private, link-local, and cloud metadata
addresses remain blocked, and every redirect is revalidated. It is local-only
and cannot be broadened by managed or remote policy.

Codex mode `disabled` removes the implicit Codex fetch default but does not
remove an explicitly enabled provider-independent fetch tool. The global kill
switch remains absolute:

```text
--disable-web-search   # disable both search and fetch
GROK_WEB_FETCH=0       # disable only local fetch
GROK_WEB_FETCH=1       # explicitly enable local fetch
```

A search-result domain may therefore work through subscription search `open`
or `click` but be rejected by local `web_fetch`. That distinction is
intentional.

## Image input, generation, and editing

Image input and image creation are separate capabilities:

- The authenticated catalog records whether a coding model advertises `image`
  input, but the current attachment pipeline does not enforce that flag before
  forwarding user images. Treat it as descriptive metadata and verify selected
  models live; unsupported input can still reach and be rejected by the
  provider.
- `image_gen` and `image_edit` are independent, separately feature-gated tools
  using the pinned `gpt-image-2` backend. Their availability does not follow
  the selected coding model's image-input metadata.

Codex image generation sends JSON to `/images/generations`. Editing sends one
or more bounded references to `/images/edits`. PNG, JPEG, and WebP edit inputs
are preserved byte-for-byte within safety limits instead of passing through
the lossy xAI Imagine preparation path. Grok aspect-ratio names are mapped to
model-valid canvases, and decoded output is bounded by bytes, pixels, and
dimensions before it is saved in the session image directory.

Image calls resolve dynamic Codex auth on every attempt, accept only Codex
auth/account/FedRAMP headers, refuse alternate production origins and
redirects, and recover one provider-scoped 401. They never treat an access
token as a static `api_key`.

The implemented ChatGPT Codex adapter exposes no video generation. xAI video
tools are unavailable while a Codex model is selected and return when the
session switches back to xAI. The existing dated product research cites
[Image generation][openai-images] for `gpt-image-2` and plan-dependent image
usage; re-check that external contract before changing the adapter.

## Usage and informational API-equivalent cost

In a Codex session:

```text
/usage              # alias: /cost
/usage show
/usage manage
```

The provider request to `/backend-api/wham/usage` exposes a sanitized subset of
current account state:

- primary and secondary rate-limit windows;
- server-provided usage percentages and reset times;
- purchased-credit availability and balance;
- spend-control limits;
- additional feature-specific rate limits;
- the current limit-reached type; and
- the available count of banked rate-limit resets, when present.

Window lengths are displayed from the response. The client does not assume
that they will always be five-hour and weekly windows. It does not derive a
ChatGPT invoice, dollar spend, or Fast-credit multiplier from these fields, and
it does not purchase or mutate plan credits.

Usage has a one-minute same-credential cache. Background failures back off from
one minute to at most fifteen minutes and may reuse only a snapshot belonging
to the exact current record. Explicit `/usage` bypasses background backoff when
there is no fresh cache. A 401 gets the same one-shot provider-owned recovery as
other Codex services.

The TUI also estimates what locally observed tokens would cost at Standard
OpenAI API rates. This is for comparison only; it is **not** ChatGPT
subscription spend, Fast-mode cost, or a provider invoice.

| Pricing basis verified 2026-07-17 | Input / 1M | Cached input / 1M | Output / 1M |
| --- | ---: | ---: | ---: |
| GPT-5.6 Sol | $5.00 | $0.50 | $30.00 |
| GPT-5.6 Terra | $2.50 | $0.25 | $15.00 |
| GPT-5.6 Luna | $1.00 | $0.10 | $6.00 |

The estimator accepts only exact provider-qualified model IDs. Unknown aliases
remain unpriced instead of borrowing a convenient rate. If auxiliary or
compaction calls were not visible to the local usage ledger, the UI labels the
total an observed lower bound. Rates live in
[`xai-grok-shell/src/auth/codex/pricing.rs`][codex-pricing] and must be reviewed
against [OpenAI API pricing][openai-api-pricing] before their verification date
is changed.

## Compaction, caching, and long sessions

Codex compaction uses the same provider-scoped Responses transport but a
deliberately tool-free request. The request:

- retains the stable session prompt-cache key;
- uses Responses Lite shaping when required;
- contains an empty `additional_tools` item;
- omits tool choice and temperature;
- streams sparse official response events;
- accepts text delivered only in `response.output_text.done`; and
- owns one exact-generation 401 recovery.

Compaction never sends xAI-specific headers. A usage-limit response is a
deterministic stop, not a transient retry loop. A network failure or an idle
stream before completion is transient and can use the bounded compaction retry
policy, but a partial summary is not silently accepted as complete.

The provider uses several different caches; do not conflate them:

| Cache/state | Key/scope | Behavior |
| --- | --- | --- |
| Prompt cache | Stable conversation ID, else session ID | Provider-side reuse; stable across refresh/retry |
| Catalog cache | Hashed credential identity + account context + endpoint/version | Five-minute local cache with authenticated ETag validation |
| Usage cache | Exact credential record/generation | One-minute local cache; never crosses account switch |
| Turn state | Exact request/turn ID | Replayed only inside the same turn; never persisted/logged |
| Search references | Search tool/session state | Retains public result URLs and navigation references |

If compaction repeatedly fails near a context limit, first run `/usage` and
inspect whether the provider reports a limit. Then distinguish a deterministic
usage/auth error from an idle/network failure. Increasing generic sampler
retries cannot fix an exhausted subscription window and can create confusing
UI loops if a partial stream is replayed.

## Failure and retry matrix

“Once” below means one provider-owned recovery budget bound to the exact
rejected credential record and generation. A retry is accepted only with the
same provider/source/record and a strictly newer generation; generic transport
retries do not create a second auth-recovery budget.

| Operation | Auth resolution | 401 policy | Redirect policy | Important terminal cases |
| --- | --- | --- | --- | --- |
| Responses inference | Every request/attempt | Reload/refresh/retry once before stream output | Refused for Codex credentials | Second 401, account change, usage limit, invalid stream/tool shape |
| Model catalog | Every fetch | Once | Refused | Empty authenticated catalog is valid; stale cross-account cache is forbidden |
| Usage | Every fetch | Once | Refused | Invalid schema, second 401, account change |
| Standalone search | Every call | Once | Refused | Oversized/deep result, invalid references, second 401 |
| Image generation/edit | Every call | Once | Refused | Wrong provider/origin, unsupported endpoint, unsafe image size/format |
| Compaction | Every attempt | Once | Refused | Usage limit, second 401, invalid terminal stream |

No row may delegate recovery to xAI. Error details may identify the provider,
consumer, HTTP status, and whether recovery was exhausted, but must not contain
credential material or account identity.

## Troubleshooting quick reference

### Login succeeds but no Codex models appear

- Run `grok models --provider openai-codex` again after the five-minute catalog
  window or restart the client.
- Treat a successful empty catalog as the account's current entitlement result.
- Check workspace/account selection during login; do not substitute the xAI
  credential or a Platform API key.
- Do not manually copy `~/.codex/auth.json` into Grok storage.

### Repeated “retrying” or duplicated assistant output

- Check `/usage` for an exhausted window before treating it as transport
  instability.
- Confirm the sampler does not retry a Codex request after text, reasoning, or
  a tool call has been emitted.
- Inspect terminal SSE events and stop reasons with redacted fixtures.
- Confirm a second 401 is terminal and that the request's credential generation
  is pinned.

### The model plans but stops before calling a tool

- Verify the model catalog still advertises Responses Lite/tool-mode metadata
  compatible with direct `additional_tools`.
- Confirm a non-empty tool registry caused `tool_choice: "auto"` to be sent.
- Inspect whether the service emitted a custom/freeform code-mode call. That is
  deliberately unsupported and must fail closed.
- Re-run a live direct-function tool smoke test; mock transport tests cannot
  prove current account/model behavior.

### Web search works but `web_fetch` rejects the URL

Use search `open` or `click`. Local fetch has a stricter fixed allowlist and
SSRF policy than the hosted subscription search service.

### Compaction fails repeatedly

- Check `/usage` and the exact provider error classification.
- Confirm the compaction request is tool-free and uses the stable session cache
  key.
- Distinguish idle timeout from a valid sparse stream whose text arrives in the
  `*.done` event.
- Never salvage a partial summary as if compaction completed.

## Implementation map

| Area | Primary files |
| --- | --- |
| Provider types/constants | [`xai-grok-sampling-types/src/provider.rs`][provider-types], [`xai-grok-sampling-types/src/openai_codex.rs`][provider-constants], [`xai-grok-version/src/lib.rs`][version-file] |
| OAuth, credentials, storage, manager, usage | [`xai-grok-shell/src/auth/codex/`][codex-auth-dir] |
| Model catalog | [`xai-grok-shell/src/remote/openai_codex_catalog.rs`][codex-catalog] |
| Sampler configuration/Responses transport | [`xai-grok-sampler/src/config.rs`][sampler-config], [`xai-grok-sampler/src/client.rs`][sampler-client], [`provider/openai_codex/`][codex-sampler-dir] |
| Search | [`web_search/backends/openai_codex.rs`][codex-search] |
| Search contract guardrail | [`fork/contracts/openai_codex_search_contract.json`][codex-search-contract], [`check_openai_codex_search_contract.py`][codex-search-checker] |
| Local fetch | [`xai-grok-tools/src/implementations/grok_build/web_fetch/`][web-fetch-dir] |
| Images | [`image_gen/mod.rs`][image-gen], [`image_edit/mod.rs`][image-edit] |
| Session wiring and provider switching | [`session/provider/openai_codex.rs`][codex-session-provider], [`session/agent_rebuild.rs`][agent-rebuild] |
| Compaction | [`xai-grok-shell/src/session/helpers/session_compact.rs`][session-compact] |
| `/fast`, `/usage`, `/cost` | [`xai-grok-pager/src/slash/commands/`][slash-commands] |
| User-facing auth guide | [`02-authentication.md`][auth-guide] |
| Attribution and modifications | [`THIRD-PARTY-NOTICES`][third-party-notices] |

## Verification matrix

### Automated, credential-free checks

Run focused tests before the broader binary check:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -I -B fork/scripts/check_openai_codex_search_contract.py
PYTHONDONTWRITEBYTECODE=1 python3 -I -B fork/scripts/tests/test_check_openai_codex_search_contract.py
PYTHONDONTWRITEBYTECODE=1 python3 -I -B fork/scripts/check_manifest.py --strict-coverage
cargo test -p xai-grok-pager --lib openai_codex_provider_parses_for_auth_and_models
cargo test -p xai-grok-shell --lib auth::codex
cargo test -p xai-grok-shell --lib remote::openai_codex_catalog
cargo test -p xai-grok-shell --lib session::provider::openai_codex
cargo test -p xai-grok-shell --lib codex_responses_compaction_tests
cargo test -p xai-grok-shell --test codex_auth_integration
cargo test -p xai-grok-sampler --lib codex
cargo test -p xai-grok-tools --lib codex
cargo test -p xai-grok-tools --lib web_search
cargo test -p xai-grok-pager --lib usage
cargo check -p xai-grok-pager-bin
```

The manifest command is read-only and validates committed `HEAD`, not
uncommitted worktree content. The Rust tests use synthetic credential records
and loopback servers; they must not read a real auth file or call the production
ChatGPT backend. The suite should cover at least:

- PKCE/state validation and device flow;
- token persistence, expiry, refresh rotation, and concurrent refresh;
- account extraction, account switching, logout isolation, and FedRAMP;
- exact URL/header bodies against loopback mock servers;
- complete header/credential redaction;
- catalog ETag, identity scoping, stale-result publication, and 401 recovery;
- Responses Lite request shaping, SSE reasoning/text/tool calls, and stop paths;
- prompt-cache stability and turn-state isolation;
- standalone search command round trips and reference extraction;
- image generation/edit auth, bounds, formats, and wrong-origin refusal;
- usage cache/backoff, rate windows, reset credits, and cost estimation;
- compaction sparse events, idle timeout, usage limit, and one-shot recovery;
- simultaneous xAI/Codex credentials and live model switching; and
- Fast/Standard service-tier isolation.

### Live entitled-account smoke test

The automated live search test is ignored **and** checks a second opt-in:

```sh
GROK_RUN_LIVE_CODEX_TESTS=1 \
  cargo test -p xai-grok-shell --lib \
  auth::codex::live_web_search_tests::live_codex_search_modes_settings_and_reference_flow \
  -- --ignored --exact --nocapture
```

Optionally set `GROK_LIVE_CODEX_MODEL=<entitled-slug>`; otherwise the scoped
catalog selects a visible entitled model. The test reads only Grok's existing
`~/.grok` Codex scope, never `~/.codex/auth.json` or a token environment
variable. It makes a catalog request plus up to four search/open requests,
which can consume ChatGPT plan credits and encounter account limits. It prints
only safe skip categories. Missing opt-in/auth/model entitlement and provider
rate limits skip; endpoint, schema, decoding, redirect, and credential-safety
failures do not.

Never print or capture token/account headers. With a test account or the user's
own private session, additionally verify:

1. Browser login and device login independently.
2. Authenticated catalog fetch and selection of Luna, Terra, and Sol when
   entitled.
3. Every advertised effort; confirm Ultra appears only where returned.
4. Long streaming text, reasoning, stop/cancel, and a normal follow-up.
5. Read, search, shell, patch, and at least one multi-step tool loop.
6. Search, open, click, find, image query, PDF screenshot, and one utility
   lookup.
7. Image attachment reading, `image_gen`, and `image_edit`.
8. `/fast` on/off/status and catalog eligibility.
9. `/usage`, `/usage manage`, rate-window resets, banked resets if present, and
   the clearly labeled API-equivalent comparison.
10. Context growth through successful compaction and a post-compaction tool
    call.
11. A forced/expired-token 401 that refreshes once without duplicate output.
12. Switching xAI -> Codex -> xAI without replacing either credential.
13. Two concurrent Grok processes sharing the scoped auth store without
    double-spending a rotated refresh token.

Record only pass/fail behavior, sanitized status codes, model slugs, and source
revision. Do not attach request dumps from a real credentialed run.

## Upstream maintenance procedure

[`UPSTREAM_VERSIONS.md`][upstream-versions] is the canonical review queue. Its
“latest fetched” revision is not automatically the “last reviewed” revision.

When `openai/codex` changes:

1. Fetch `origin/main` in the ignored `inspiration/openai-codex` checkout.
2. Update only the “latest fetched” column immediately.
3. Diff the reviewed and latest revisions across:
   - `codex-rs/login` for OAuth, storage, token claims, refresh, and revocation;
   - `codex-rs/model-provider*` and `models-manager/models.json` for endpoint,
     headers, catalog schema, efforts, tiers, modalities, tool mode, and
     multi-agent metadata;
   - `codex-rs/core/src/client*` and Codex API types for Responses/SSE,
     prompt caching, turn state, retries, compaction, rate limits, search, and
     images;
   - app-server tests for observable contract changes, without importing its
     application architecture; and
   - official auth, model, speed, image, usage, and pricing documentation.
4. Diff the tracked OpenCode Codex plugin as an independent interoperability
   reference, never as the primary contract.
5. Update `OPENAI_CODEX_COMPATIBILITY_VERSION` only after the request/header
   contract and catalog version are verified together.
6. Update this dated model/price snapshot, implementation comments, and tests.
7. Preserve the OpenAI Codex attribution and prominent modification notice in
   `THIRD-PARTY-NOTICES`; add notices for newly ported code before shipping.
8. Run the credential-free suite and the complete live smoke matrix.
9. Move “last reviewed” only after the diff is understood and compatibility
   changes are merged.

The ignored inspiration checkout may not be fast-forwarded to its fetched
remote-tracking ref. Prefer `git show <recorded-revision>:<path>` when auditing
the exact tracker revision.

## Reference sources

Primary sources:

- [OpenAI Codex authentication][openai-auth]
- [OpenAI Codex speed and Fast mode][openai-speed]
- [OpenAI Codex model guidance][openai-models]
- [OpenAI Codex image generation][openai-images]
- [OpenAI API pricing][openai-api-pricing]
- [`openai/codex` login at the ledger's latest-fetched snapshot][upstream-login]
- [`openai/codex` model catalog at that latest-fetched snapshot][upstream-models]
- [`openai/codex` model-provider crates at that snapshot][upstream-provider]
- [`openai/codex` client transport at that snapshot][upstream-client]
- [`openai/codex` app-server README at that snapshot][upstream-app-server]

These links are pinned review inputs. Their latest-fetched commit is not the
ledger's last-reviewed commit.

Secondary interoperability pointer (latest fetched, not promoted to reviewed):

- [OpenCode's Codex plugin at the ledger's latest-fetched revision][opencode-codex-plugin].
  OpenCode remains research/provenance input only, never a runtime provider.

The local `inspiration/` clones are research material only and must remain
gitignored. Never copy auth files or credentials out of them or the user's home
directory.

[agent-rebuild]: ../../crates/codegen/xai-grok-shell/src/session/agent_rebuild.rs
[auth-guide]: ../../crates/codegen/xai-grok-pager/docs/user-guide/02-authentication.md
[codex-auth-dir]: ../../crates/codegen/xai-grok-shell/src/auth/codex/
[codex-catalog]: ../../crates/codegen/xai-grok-shell/src/remote/openai_codex_catalog.rs
[codex-oauth]: ../../crates/codegen/xai-grok-shell/src/auth/codex/oauth.rs
[codex-pricing]: ../../crates/codegen/xai-grok-shell/src/auth/codex/pricing.rs
[codex-responses]: ../../crates/codegen/xai-grok-sampler/src/provider/openai_codex/responses.rs
[codex-sampler-dir]: ../../crates/codegen/xai-grok-sampler/src/provider/openai_codex/
[codex-search]: ../../crates/codegen/xai-grok-tools/src/implementations/web_search/backends/openai_codex.rs
[codex-search-checker]: ../../fork/scripts/check_openai_codex_search_contract.py
[codex-search-contract]: ../../fork/contracts/openai_codex_search_contract.json
[codex-session-provider]: ../../crates/codegen/xai-grok-shell/src/session/provider/openai_codex.rs
[image-edit]: ../../crates/codegen/xai-grok-tools/src/implementations/grok_build/image_edit/mod.rs
[image-gen]: ../../crates/codegen/xai-grok-tools/src/implementations/grok_build/image_gen/mod.rs
[openai-api-pricing]: https://developers.openai.com/api/docs/pricing
[openai-auth]: https://learn.chatgpt.com/docs/auth
[openai-images]: https://learn.chatgpt.com/docs/image-generation
[openai-models]: https://learn.chatgpt.com/docs/models
[openai-speed]: https://learn.chatgpt.com/docs/agent-configuration/speed
[opencode-codex-plugin]: https://github.com/anomalyco/opencode/blob/efb6cc2d4bf6332eb156709795d2b3a649198b65/packages/opencode/src/plugin/openai/codex.ts
[provider-constants]: ../../crates/codegen/xai-grok-sampling-types/src/openai_codex.rs
[provider-types]: ../../crates/codegen/xai-grok-sampling-types/src/provider.rs
[sampler-client]: ../../crates/codegen/xai-grok-sampler/src/client.rs
[sampler-config]: ../../crates/codegen/xai-grok-sampler/src/config.rs
[session-compact]: ../../crates/codegen/xai-grok-shell/src/session/helpers/session_compact.rs
[slash-commands]: ../../crates/codegen/xai-grok-pager/src/slash/commands/
[third-party-notices]: ../../THIRD-PARTY-NOTICES
[upstream-app-server]: https://github.com/openai/codex/blob/315195492c80fdade38e917c18f9584efd599304/codex-rs/app-server/README.md
[upstream-client]: https://github.com/openai/codex/blob/315195492c80fdade38e917c18f9584efd599304/codex-rs/core/src/client.rs
[upstream-login]: https://github.com/openai/codex/tree/315195492c80fdade38e917c18f9584efd599304/codex-rs/login
[upstream-models]: https://github.com/openai/codex/blob/315195492c80fdade38e917c18f9584efd599304/codex-rs/models-manager/models.json
[upstream-provider]: https://github.com/openai/codex/tree/315195492c80fdade38e917c18f9584efd599304/codex-rs/model-provider
[upstream-versions]: ../../UPSTREAM_VERSIONS.md
[version-file]: ../../crates/codegen/xai-grok-version/src/lib.rs
[web-fetch-dir]: ../../crates/codegen/xai-grok-tools/src/implementations/grok_build/web_fetch/
