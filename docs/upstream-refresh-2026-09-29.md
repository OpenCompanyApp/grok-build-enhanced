# Upstream refresh evidence — 2026-09-29

Status: incomplete audit; not a release candidate. No Grok Reviewed advancement,
acknowledgement, push, tag, release, or Homebrew publication is claimed.

## Boundary

The refresh uses isolated branch `refresh/upstreams-20260929`, based on published
mainline commit `c162abc66648ead54a3da233db523c692d0a5f5d`, tree
`f999ffdf9cdd516de3d39987d27ec9d873359c74`. The original checkout at
`4e457e859db47db03d079abe64c73eda874db802` and all its tracked/untracked local
changes remain untouched. This avoids regressing the 99 mainline-only commits
relative to that older feature branch. No upstream tree was merged or rebased.

## Exact source pins

The [pin inventory](upstream-refresh-2026-09-29-pins.json) records commit/tree
identities, ancestry checks, and raw no-rename diff digests. Successful fetches
advance Latest fetched only, except the completed GLM review below.

| Source | Changed paths since Reviewed | Fetch/review state |
| --- | ---: | --- |
| openai-codex | 5947 | review remains open |
| opencode | 514 | review remains open |
| grok-build-upstream | 3828 | review remains open |
| opencode-codex-auth | 0 | unchanged reviewed head |
| oh-my-pi | 6768 | review remains open |
| warp-themes | 0 | unchanged reviewed head |
| ghostty | 213 | review remains open |
| herdr | 1290 | review remains open |
| kimi-code | 3329 | review remains open |
| kimi-cli | 39 | review remains open |
| zai-sdk-python | unknown | tracked URL unavailable; old pin preserved |
| zai-coding-plugins | 0 | unchanged reviewed head |
| glm-5 | 2 | GLM review completed below |
| codexbar | 2519 | review remains open |
| zai-usage-helper | 0 | unchanged reviewed head |
| models-dev | 4255 | review remains open |
| exa-mcp-server | 4 | review remains open |

The Z.AI SDK tracked URL returns `Repository not found`; no replacement was
inferred. Fetching this source and reviewing any new revision remains unavailable.

## Grok raw ranges and adoption queue

The [raw tree inventory](upstream-refresh-2026-09-29-grok-paths.json) compares
literal `(mode,type,oid)` entries without rename inference. It records:

- `07b2f7144fd5c5c9d3dd1966937a87852d2dbdb8` → `37949780c144e37df692e3d669051a21fec24f20`: 3,340 changed paths.
- `37949780c144e37df692e3d669051a21fec24f20` → `f0e3be1100ef5252488e3be8bb0e91cf68d8c305`: 2,027 changed paths.
- `07b2f7144fd5c5c9d3dd1966937a87852d2dbdb8` → `f0e3be1100ef5252488e3be8bb0e91cf68d8c305`: 3,828 changed paths.

These are exhaustive path inventories, not exhaustive behavior classifications.
They cannot authorize acknowledgement. Existing feature-pattern matches are
recorded per raw path: 2019 paths have existing owners; 1809 require
focused ownership decisions if adopted. These matches do not classify behavior. The earlier report's 2,998-path count for
Reviewed→September 9 differs from this run's reproducible raw no-rename count of
3,340; this inventory preserves the actual entry identities rather than relying
on the earlier prose count.

Initial session/ACP inspection confirms substantive adoption work remains:
`acp_conversion.rs` adds safe UTF-8 task-ID abbreviation and additional tool-output
projection; new `active_agent_message_presentation.rs` and its regression test
introduce content-free pending subagent-message titles. These observations are
not claimed as a complete inventory or as locally implemented parity.

Largest changed upstream crate groups:

- `crates/codegen/xai-grok-pager`: 996 paths.
- `crates/codegen/xai-grok-shell`: 858 paths.
- `crates/codegen/xai-grok-pager-pty-harness`: 332 paths.
- `crates/codegen/xai-grok-tools`: 249 paths.
- `crates/codegen/xai-grok-workspace`: 132 paths.
- `crates/codegen/xai-grok-telemetry`: 125 paths.
- `crates/codegen/xai-grok-pager-render`: 91 paths.
- `crates/common/xai-tool-protocol`: 74 paths.
- `crates/codegen/xai-grok-test-support`: 67 paths.
- `crates/codegen/xai-fast-worktree`: 59 paths.

## Completed bounded GLM review

`414ad9eb891b05b5d7d51d573939bfe9ce538223` →
`c8ad661c6cf4cb0a78064987bc42f97e14355929` changes exactly two paths:

- `resources/wechat.png`: binary image replacement; **not applicable** to the
  declared Z.AI provider-contract/model-research scope.
- `skills/glm-master-skill/SKILL.md`: renames the linked image-generation skill
  from `glm-image-gen` to `glm-image-generation`; **not applicable** to this
  fork's established Coding Plan provider contract.

No model, endpoint, inference, authentication, usage, tool wire contract, source
license, or notice changes occur in this range. No code or assets were imported.
Reviewed advances for GLM only. Validation is the complete two-path raw diff and
full text diff inspection; no new runtime tests are needed for an unimported image
and documentation link.

## Bounded Exa inspection

`15ffb50519e719dc791cdc750ce5ed1934c0a1ed` →
`f3d71fb6b0ff4b4683f108f05bc2bae61a9f7e97` changes four paths, all inspected:

- `.github/workflows/publish-mcp-registry.yml`: upstream MCP registry publication,
  outside the fork's owned release routes; **not applicable**.
- `package.json`: registry identity becomes `ai.exa/exa`; the adapter does not
  resolve MCP registry names; **not applicable** to its endpoint-bound requests.
- `server.json`: registry metadata switches from SSE-labelled/npm packaging to a
  remote-only Streamable HTTP declaration at the unchanged `https://mcp.exa.ai/mcp`.
  Local `web_search/backends/exa.rs::execute` already posts JSON-RPC to that endpoint
  and accepts JSON and SSE replies. Existing test
  `projects_json_and_sse_without_provider_metadata` covers both response forms.
- `skills/exa-agent/SKILL.md`: display-name change only; **not applicable** to the
  fork's provider adapter.

No transport implementation changed upstream. Reviewed remains unchanged pending
focused local test execution; source inspection alone is not claimed as validation.

## Carried obligations

`fork/parity/current.json` is preserved byte-for-byte. Existing owners, blockers,
acceptance criteria, intended tests, and overdue deadlines remain visible. No
obligation is silently closed or rescheduled:

- `CDX-E482-CONTEXT-WINDOW-ID`
- `GB-BC7F-SESSION-ACP`
- `GB-BC7F-AGENT-LIFECYCLE`
- `GB-BC7F-TUI-INPUT`
- `GB-BC7F-WORKSPACE`
- `GB-BC7F-INTEGRATION`
- `CDX-6478-AUTH-CATALOG-WIRE`
- `CDX-6478-APP-BEHAVIOR`
- `OC-DC44-CODEX-INTEROP`
- `KIMI-9D23-PROVIDER`
- `OMP-33CC-HARNESS`
- `CODEXBAR-CE44-ZAI-USAGE`

The separately recorded `ZAI-414A-RUNTIME` remains offline-qualified, not fully
live-qualified. New source ranges still need exhaustive behavior inventory,
thematic ports, provider-isolation tests, and closure evidence. Implementation
size is not a scope exclusion.

## Publication and validation

Latest observed stable GitHub release: `v0.3.18`. The tap already targets
`v0.3.18` for all four native platforms, and all four formula digests match the
public release SHA256SUMS. The public release is neither draft nor prerelease and
has exactly the four expected binaries plus SHA256SUMS and RELEASE-PROVENANCE.json.
This metadata comparison is not a fresh binary-download or attestation validation.
No new release version has been selected or confirmed. No Homebrew executable was
found on this Linux aarch64
host, so style/audit/test and installed-upgrade checks have not run.

The refresh skill requires no open temporary Grok adoption deferral before the
acknowledgement, and requires the acknowledgement as publication candidate.
Those gates are not satisfied. Publication authorization does not establish
behavior parity or waive required tests.

Passed:

- `cargo fmt --all -- --check`;
- strict fork manifest validation at `25ddb8ce`: 2,793/2,793 downstream paths
  covered, 132 feature units, 12 open obligations retained;
- fork contracts (branding, providers, Codex search, Warp, updater, workspace,
  workflows, and secret-handling contracts);
- all 5 release-pipeline tests and all 15 installer tests;
- Warp vendor lock: 343 themes at the unchanged reviewed pin;
- release contract against existing version 0.3.18, without publishing anything;
- `git diff --check`.

`CARGO_INCREMENTAL=0 cargo check -p xai-grok-pager-bin` passed.
All three raw inventory ranges were independently rechecked against Git objects
after ownership annotation. These checks validate the local baseline
and evidence bookkeeping, not parity with newly fetched upstream behavior.
No live provider requests or private credential reads were performed.

The Cargo `target/` directory generated in this isolated worktree was removed
after validation. No build artifacts from other worktrees were removed. The
original checkout still has the same four tracked modifications and the same
untracked `assets/` and `t3.json`; none were staged or modified by this run.
