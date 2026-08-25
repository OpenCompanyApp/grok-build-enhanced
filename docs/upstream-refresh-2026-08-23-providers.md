# Provider-reference refresh audit — 2026-08-23

This audit records the non-Grok reference ranges fetched alongside the pinned
Grok Build refresh. These repositories are normative only within the provider,
interoperability, or research scopes declared in `AGENTS.md`; no source tree was
merged or copied wholesale.

| Source | Reviewed → fetched | Commits | Raw paths | Raw-path SHA-256 | Classification |
| --- | --- | ---: | ---: | --- | --- |
| OpenAI Codex | `e482cc66` → `c9b19deb` | 48 | 398 | `3688d79bac1b5fad3542fef44dfaf0fb34cd6bdf4c7ae0a4764d063798415fae` | Adapter reference reviewed; Lite wire correction adopted below, remaining app-server/Guardian/computer-use work is outside the isolated adapter or already equivalent. |
| OpenCode | `1b937c86` → `3a31c4ea` | 8 | 34 | `13a505133ccd74a0b7f2762fe44a6ced19c755df0f758cf3ecad117c18c69ba1` | Interoperability reviewed; generic OpenAI-compatible providers already avoid OpenAI-only `textVerbosity` injection. |
| Oh My Pi | `9350b799` → `160ed439` | 509 | 656 | `48d881ec97f3c3eac4e1f6d79fa2036ff880080ee5684f6b389c718e73775e6d` | Non-normative coding-harness evaluation only; no application architecture adopted. |
| Kimi Code | `d4e0ad4b` → `ea0626ad` | 5 | 242 | `eeab362bdfe5d9977562eb588483b9e88818801f7bddd1340cacd8a42ecf5924` | No Kimi Code auth, catalog, or request-wire change applicable to the isolated provider adapter. |
| CodexBar | `f74117` → `4b14ed` | 20 | 100 | `24aa2a900e0efcf5e95afffc0b5b826249f049bc8827025497a828dd6b9c9755` | Z.AI usage research only; release/UI changes do not establish a provider contract. |
| models.dev | `a166d7` → `59d624` | 101 | 116 | `3ca2677b13ebb04c3ab8683a7aa51cd4d5d93274331440e304f998604eda6ff2` | Catalog hint reviewed; no first-party Z.AI GLM-5.3 record or runtime contract appeared. |

The Codex raw-path digest above covers `(status, old mode, new mode, old OID,
new OID, path)` records for the exact range, as do the other recorded digests.
The fetched target trees are:

- Codex `bc73a12ff0e38cb19cc067c0d737a7b8010a0f93`
- OpenCode `01667473a962417551be84b2010ddd1816cad269`
- Oh My Pi `98fd2ff0daed4775f96a3c76cf016ff92381ddfe`
- Kimi Code `21eaee2bcbab7882c92505189a84c9134a598cf1`
- CodexBar `cdb22d6776c187fef95b8f5ccc9c605f06ed7666`
- models.dev `0bc27466d57a86b0dc5d44e6167c019eddb42fd5`

## ChatGPT Codex 400 correction

Enhanced had set `parallel_tool_calls=true` even after rewriting a request to
the ChatGPT Codex Responses Lite endpoint. The pinned official Codex source
continues to require `parallel_tool_calls=false` when `use_responses_lite` is
active. Enhanced now applies that endpoint-specific value while preserving the
ordinary Responses behavior, provider-owned auth, turn state, request identity,
and Lite `additional_tools` contract. The focused OpenAI Codex sampler suite is
the acceptance test. Live validation remains credential and entitlement gated.

## Z.AI GLM Coding Plan / GLM-5.3 boundary

The Z.AI SDK, coding-plugin, GLM-5, CodexBar usage-helper, and usage-browser
research pins did not advance in this refresh. No authenticated first-party
GLM-5.3 coding-plan wire contract was available in the reviewed sources.
Accordingly, and in accordance with repository policy, Z.AI GLM Coding Plan
remains research-only: this refresh adds no Z.AI runtime identity, login,
credential storage, or product-support claim. Generic custom-provider behavior
remains generic and must not consume another provider's credentials.
