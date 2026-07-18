# Provider documentation

This directory contains fork-specific provider references, research, and
implementation designs. A document or upstream source entry is not evidence
that a runtime provider exists.

## Implemented and planned matrix

The checked-in Rust provider and CLI enums are authoritative for runtime
identity:
[`ProviderId`](../../crates/codegen/xai-grok-sampling-types/src/provider.rs) and
[`AuthProviderArg`](../../crates/codegen/xai-grok-pager/src/app/cli.rs).

| Entry | Status in this fork | Runtime identity / credential behavior |
| --- | --- | --- |
| xAI | Implemented upstream provider | Runtime `xai`; remains the default CLI auth provider. |
| [OpenAI Codex subscription](openai-codex-subscription-provider-reference.md) | **Implemented, experimental** | Runtime `openai_codex` with the isolated `openai_codex_subscription` credential source and `openai::codex` auth scope. |
| Generic custom transport | Implemented transport path, not a product-specific account integration | Runtime `custom`; it is not inferred to be Codex and receives neither Codex subscription credentials nor generic xAI tool credentials. |
| [Kimi Code](kimi-code-integration-research.md) | **Unimplemented research/design** | `kimi_code` and its example CLI spelling are proposals only; neither is a runtime or CLI provider identity. |
| [Z.AI GLM Coding Plan](zai-glm-coding-plan-integration-research.md) | **Unimplemented research/design** | `zai_coding_plan` and its example CLI spelling are proposals only; neither is a runtime or CLI provider identity. |
| OpenCode | **Not a provider candidate implemented by this fork** | Upstream interoperability/provenance research only; there is no `opencode` runtime identity, login, catalog, session, or credential scope. |

Shipped provider behavior is also documented in the
[pager user guide](../../crates/codegen/xai-grok-pager/docs/user-guide/).
The root [README](../../README.md) is a summary, not the provider contract.
Reviewed and latest-fetched upstream revisions are tracked separately in
[`UPSTREAM_VERSIONS.md`](../../UPSTREAM_VERSIONS.md); “latest fetched” does not
mean “reviewed.”

Keep ignored `inspiration/` checkouts, credentials, private approval messages,
and raw authenticated responses out of the repository. If source code is
ported rather than behavior independently implemented, update the applicable
third-party notices and preserve its license requirements.
