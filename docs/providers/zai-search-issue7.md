# Z.AI search result decoding — issue #7

The [bug report](https://github.com/OpenCompanyApp/grok-build-enhanced/issues/7)
reports that `web_search` fails on Enhanced 0.3.20 with GLM-5.3 and GLM-5.3-Flash,
while `web_fetch` works. The error comes from the search result projection.

## Evidence and scope

The [provider documentation](https://docs.z.ai/devpack/mcp/search-mcp-server)
defines the Coding Plan Search MCP endpoint and result fields. A separate
[client report](https://github.com/can1357/oh-my-pi/issues/8000) documents results
encoded as a JSON string containing JSON, rather than a directly encoded array.
Enhanced currently performs one JSON decode, then accepts only an array or a
`search_result` / `results` array. A string therefore produces the exact reported
unsupported-format error. The fixtures here are independently constructed
synthetic data; no credentials or authenticated response captures are retained.

Classification: **adopt** the string-wrapped search result representation within
the existing Z.AI provider adapter. Decode at most two additional string layers
(three JSON decodes total), then apply the existing strict result projection.
Keep the 256 KiB input bound, domain filtering of both text and citations,
URL validation, and generic errors that never include provider response text.
Do not add a raw-text fallback or change endpoints, authentication, model IDs,
MCP negotiation, other providers, or web reader behavior.

The implementation is independent; no reference implementation was copied.
Source review pins and the existing parity obligations remain unchanged.

## Validation

The new wrapped-result regression failed on the original parser with the exact
reported `Z.AI search returned an unsupported result format` error. After the
fix, all 24 `xai-grok-tools --lib zai` tests passed, including three new search
cases and the existing MCP negotiation and credential-isolation tests.

`cargo check --locked -p xai-grok-pager-bin` passed with incremental builds off
and generic ARM64 CPU flags appropriate for the Pi. Formatting, fork contracts,
five release-pipeline tests, fifteen installer tests, and the proposed 0.3.21
release asset contract passed. Publication still requires a confirmed version,
committed ownership validation, native release CI, and Homebrew checks.

The release workflow now runs the Z.AI tools test filter; main CI already runs
it. Live subscription acceptance is not claimed by synthetic tests.
