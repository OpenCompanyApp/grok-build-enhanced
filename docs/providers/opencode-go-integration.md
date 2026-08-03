# OpenCode Go provider

OpenCode Go is a first-class, experimental provider adapter in Grok Build
Enhanced. It preserves the Grok agent loop and UI while isolating Go models,
credentials, catalog state, inference routes, and search behavior from xAI,
ChatGPT Codex, Kimi Code, and custom providers.

## Setup

```sh
grok login --provider opencode-go
grok models --provider opencode-go
grok logout --provider opencode-go
```

Login reads `GROK_OPENCODE_GO_API_KEY`, then the compatibility variable
`OPENCODE_API_KEY`, or a key piped on standard input. The key is stored in the
existing `~/.grok/auth.json` under the independent `opencode::go` scope. The
login command does not import OpenCode cookies, OpenCode auth files, Zen
credentials, or any other provider's state. A public catalog fetch is not
presented as key validation; the key is validated by the first inference.

## Fixed provider contract

| Concern | Contract |
| --- | --- |
| Runtime provider | `open_code_go` (`opencode-go/` model namespace) |
| Credential source | `open_code_go_api_key` |
| Auth scope | `opencode::go` |
| Inference root | `https://opencode.ai/zen/go/v1` only |
| Catalog | Public `/models`, intersected with the audited static route/capability registry |
| Chat and Responses auth | Sensitive `Authorization: Bearer …` |
| Messages auth | Sensitive `x-api-key` plus `anthropic-version: 2023-06-01` |
| Redirects | Refused for every credential-bearing request |
| Logout | Removes only the OpenCode Go credential record |

Catalog responses are bounded to 1 MiB and 256 records. Unknown live model IDs
are ignored until their route and capabilities are audited. Deprecated audited
IDs remain readable in cache metadata but are not user-selectable. The adapter
does not scrape undocumented quota endpoints and does not create a Zen runtime
provider.

## Web search default

OpenCode Go sessions enable the keyless hosted Exa MCP endpoint
`https://mcp.exa.ai/mcp` by default. The existing global web-search disable
switch still wins. Search remains the existing `web_search` tool and therefore
keeps Enhanced's ordinary read-only permission policy; there is no first-use
provider prompt.

The client sends only the query and fixed bounded options to
`web_search_exa`: `type=auto`, eight results, `livecrawl=fallback`, and 10,000
context characters. It sends no Go or xAI credential. Redirects are refused,
the request timeout is 25 seconds, response bodies are limited to 4 MiB, and
rendered output is limited to 256 KiB. JSON and SSE MCP responses are accepted.
Because this transport has no domain-filter contract, a non-empty
`allowed_domains` request fails closed instead of silently performing an
unfiltered search.

## Media

The dynamic catalog exposes only capabilities present in the audited
models.dev snapshot. Image input uses the existing URL/data-URL path. Audio,
video, and PDF inputs use durable local path plus MIME metadata and are read
only at request time, with a 100 MiB per-file ceiling. MP3/WAV audio is accepted
only for advertised Chat Completions models; PDF is projected as a file input;
and the audited video MIME set is projected through Chat or Messages routes.
The selected model's capability and backend are checked before file I/O.

Other providers reject OpenCode Go audio/document content before network I/O.
Existing Kimi video upload behavior remains independent. No encoded media or
remote identifier is written to session history.

## Frozen research inputs

This implementation is independently written against these reviewed behavior
references:

| Source | Revision |
| --- | --- |
| OpenCode | `1882c33827cf0ce5c948b69ab5a87ed8f6790cf8` |
| models.dev | `f67be44f095a4ab24ceab33c3907317bb0375087` |
| Exa MCP server | `a664592b5dd7c5598b70158c771dcc5c2a4fb2c1` |

These revisions bound the audited model names, protocol routing, media flags,
and Exa tool contract. Later changes require a separate refresh review.
