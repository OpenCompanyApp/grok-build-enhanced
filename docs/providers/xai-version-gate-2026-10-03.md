# xAI version-gate compatibility fix — 2026-10-03

The reported HTTP 426 rejects Enhanced 0.3.18 against an upstream Grok CLI
minimum of 1.0.13. Enhanced 0.3.19 also sends its distribution version on this
wire field, so upgrading within the Enhanced release sequence cannot fix it.

## Evidence and scope

Pinned reference: Grok Build `2bdd1d6a6369de0e8c68132ea4539e9abd9e14a8`
(the existing September 30 snapshot; no source revision is advanced).
`crates/codegen/xai-grok-sampler/src/client.rs` explicitly identifies
`x-grok-client-version` as the proxy version gate. Upstream pager identity
feeds its upstream package version into it. Enhanced instead compiles
`GROK_VERSION` from its independent release tag, causing the mismatch.
The reference package version is 1.0.45; the compatibility floor selected
here is only the reported required 1.0.13. This does not assert adoption of
all upstream 1.0.13 or 1.0.45 application behavior.

## Scoped behavior ledger

| Behavior | Decision | Implementation / evidence |
| --- | --- | --- |
| xAI wire version belongs to the upstream compatibility sequence | adopt | Dedicated `XAI_CLIENT_COMPATIBILITY_VERSION = 1.0.13`; canonical xAI sampling routes use it even when caller metadata contains Enhanced 0.3.18 or 0.3.20. |
| Auxiliary xAI calls face the same version gate | adopt | Apply the same constant to auth, catalog, remote calls, subscription/billing/consent, tools, storage, embeddings, managed MCP and internal OTLP request headers. |
| Installed release identity and updater routes | already equivalent | Keep `VERSION`, origin User-Agent, release metadata, telemetry resource identity, ACP and fork updater versions intact. Version text/JSON additionally label xAI compatibility. |
| Other providers must not receive xAI version identity | already equivalent | Existing canonical-origin and provider header filtering remains; exercise the fake-server provider matrix and credential isolation tests. |

Regression `xai_version_gate_uses_compatibility_not_enhanced_release` builds
actual requests for both canonical xAI inference routes with absent, 0.3.18,
and 0.3.20 caller versions. Version/report tests distinguish wire compatibility
from distribution identity. Test compilation is stamped `GROK_VERSION=0.3.18`
to reproduce the reported fork version without making it the wire version.

No provider credentials or authenticated response captures are used. Local
request construction and mock tests do not establish live account entitlement
or acceptance by every production endpoint.

This is a focused repair, not a completed upstream refresh. The 12 open IDs
in `fork/parity/current.json` remain unchanged. No reviewed source revision or
upstream acknowledgement is advanced. Changes are independently implemented;
existing source licenses and notices remain intact.

## Existing test correction

The full sampler suite exposed a stale wire-test assertion introduced before
`7607152d` fixed Responses Lite. The production boundary and its focused test
already require `parallel_tool_calls=false` for Lite, but the integration test
still expected true. Correct only that assertion; Codex production behavior is
unchanged. The same test continues to prove that xAI headers do not cross the
Codex boundary.

## Validation

Passed before publication:

- 382 sampler tests, including the mock 426 reproduction and corrected Codex
  wire assertion; 6 version tests; 1 storage-header test; 82 shell xAI/provider
  tests; and 1 CLI version-report test: **472 tests passed**.
- `CARGO_INCREMENTAL=0 cargo check -p xai-grok-pager-bin`, with the simulated
  `GROK_VERSION=0.3.18` stamp and generic ARM64 CPU settings.
- Formatting, fork contracts, strict first-parent ownership (137 features,
  2,802/2,802 downstream paths), 5 release-pipeline tests, 15 installer tests,
  and the v0.3.20 release contract.

The full CLI check and version-report test completed after resuming the
interrupted host session, reusing the task-owned build artifacts. Publication
version v0.3.20 was explicitly confirmed. Release assets and installed Homebrew
upgrade are separately verified after the tag-triggered workflow completes.
