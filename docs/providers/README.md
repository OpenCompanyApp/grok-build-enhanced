# Provider documentation

This directory contains fork-specific provider references, research, and
implementation designs. Each document declares its own status; its presence
here alone does not mean that a provider is implemented or supported.

Implemented experimental provider reference:

- [OpenAI Codex subscription provider reference](openai-codex-subscription-provider-reference.md)

Research for candidate providers that are not yet implemented:

- [Kimi Code plan integration research](kimi-code-integration-research.md)
- [Z.AI GLM Coding Plan integration research](zai-glm-coding-plan-integration-research.md)

Shipped provider behavior is documented in the
[pager user guide](../../crates/codegen/xai-grok-pager/docs/user-guide/), and
implemented fork features are summarized in the root [README](../../README.md).
Immutable source snapshots reviewed for these documents are recorded in
[`UPSTREAM_VERSIONS.md`](../../UPSTREAM_VERSIONS.md).

Keep ignored `inspiration/` checkouts, credentials, private approval messages,
and raw authenticated responses out of the repository. If source code is
ported rather than behavior independently implemented, update the applicable
third-party notices and preserve its license requirements.
