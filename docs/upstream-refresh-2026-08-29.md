# Upstream refresh and Z.AI GLM Coding Plan runtime — 2026-08-29

This refresh was performed on the isolated
`refresh/upstreams-20260829` worktree from Enhanced commit
`c7365c996d6f9be460b0b082a96780840ad5e34a`. It fetched and pinned every
tracked source without merging an upstream tree. The pre-existing working-tree
changes in the original checkout were not copied, modified, or discarded.

## Pinned source ranges

Raw hashes below cover the exact output of
`git diff --no-renames --raw --abbrev=40 FROM TO` between range endpoints.
Commit counts are ancestry-path counts. A difference between Reviewed and
Latest fetched remains a visible review queue.

| Source | Range | Commits | Raw paths | Raw SHA-256 | Target tree | Outcome |
| --- | --- | ---: | ---: | --- | --- | --- |
| Grok Build | `07b2f714` → `bc7f02ed` | 4 | 1,615 | `f1b4a7fb3e2205f8945b345f43ea9d7d61f7a815e87f46ad5055a4221af5a7cb` | `1f9266ee49f4f1d82450b45f10f68a7dd58b23ef` | fetched; preserved-surface adoption is deferred below |
| OpenAI Codex | `c9b19deb` → `6478a751` | 308 | 1,463 | `1dd482636c6981b775b27c336db58af1f40965d3f578c9956b1f7ba7f09d52a2` | `80ce47826a4dd0d72a49b2768952ea22757d5c0f` | fetched; adapter and observable-behavior review remains queued |
| OpenCode | `3a31c4ea` → `dc4449df` | 87 | 332 | `d0ef4ff5e1d4ea9b727309c91270170a37a61fe482a6ad7af6117b217a5139ab` | `3194589e295e3f97eb841f16880544156d0d17a7` | fetched; Codex interoperability review remains queued |
| Oh My Pi | `160ed439` → `33cc6b9a` | 988 | 1,057 | `880f444eedb03afc999645676743c3101a41eef88d5512a75216868804e69e3f` | `61638f26504aa69543412b543a6020eef58a8287` | fetched; non-normative harness evaluation remains queued |
| Warp themes | `6cc44d7b` → `4154072d` | 1 | 10 | `5d69c1b1d4fe03ce01e56705a2c64777cedb71628d02e1f5036ba5e506772024` | `0ba990236c4ad1d2026815bdde9463e2b1e07c61` | reviewed and adopted |
| Kimi Code | `ea0626ad` → `9d2304c2` | 64 | 973 | `af4ad7834189ac27d5ba0bdc41f0243c1556b9477b6779e7faf8dda198a44986` | `d498910c580d30715e234782efb334e60fff7908` | fetched; provider-contract review remains queued |
| GLM-5 | `25206af8` → `414ad9eb` | 5 | 5 | `e85ec64a13e71d8d943fbba1454412bf90f7fc129737012175a16881bfeec169` | `1d7a08fc54c804352e8bd9634dab609bc42f2d70` | reviewed; GLM-5.3 model evidence adopted |
| CodexBar | `4b14ed9c` → `ce447135` | 102 | 382 | `c2971f0aca8d2c385ae536180dfe4e2d228c6fe5a41616b19679f686e2f50143` | `e7137268e4be12aa92c38a807e0422ee3af9af6d` | fetched; Z.AI usage research remains queued |
| models.dev | `59d624d4` → `efdbe97a` | 434 | 1,310 | `b0c20edfdf94ad2deaa8b9b9821b229700708a4cc5dedfc789dee6293548773b` | `3f0c47702c49b94e0f7fe2320b2f0dca58b6866f` | fetched; third-party catalog metadata is non-authoritative |

OpenCode Codex auth, Kimi CLI, Z.AI coding plugins, the Z.AI usage-browser
reference, and Exa were unchanged. The configured Z.AI Python SDK repository
again returned `Repository not found`; its immutable `ca5109c0` pin was
retained and no replacement repository was inferred.

## Grok Build parity

The endpoint tree comparison contains 1,615 distinct raw paths. Every row is
recorded exactly once in
[`upstream-refresh-2026-08-29-paths.json`](upstream-refresh-2026-08-29-paths.json)
with its modes, object IDs, status, stable obligation, and classification.

The four upstream snapshots span Grok 1.0.9 through 1.0.12 and introduce or
change observable behavior in these preserved areas:

| Stable ID | Paths | Classification | Behavior queue |
| --- | ---: | --- | --- |
| `GB-BC7F-SESSION-ACP` | 283 | temporarily deferred | Session persistence, model/session restore, ACP, headless resume visibility, turn footer/duration restoration, usage, and compaction accuracy. |
| `GB-BC7F-AGENT-LIFECYCLE` | 179 | temporarily deferred | Auto mode, permission defaults, subagent concurrency/rate limits, hook blocking/defer/additional-context behavior, MCP retry/startup, and tool lifecycle. |
| `GB-BC7F-TUI-INPUT` | 779 | temporarily deferred | Full/minimal mode switching, slash-command recency/workflows, paste/mouse/Kitty/input handling, scrollback, prompt status, and rendering fixes. |
| `GB-BC7F-WORKSPACE` | 53 | temporarily deferred | Shallow/local worktree cloning, workspace classification, filesystem watching, PTY, and Git behavior. |
| `GB-BC7F-INTEGRATION` | 320 | temporarily deferred | Remaining configuration, authentication, telemetry, diagnostics, generated manifests, locks, tests, and cross-cutting integration. |
| `GB-BC7F-RELEASE` | 1 | not applicable | `SOURCE_REV` remains fork-owned release metadata. |

A disposable three-way port trial produced hundreds of conflicts across broad
downstream provider and application changes. The trial worktree and branch were
removed. That evidence rules out a safe mechanical import; it does not make the
observable behaviors inapplicable. The five adoption groups remain owned by
the Enhanced maintainers, target the next refresh or `v0.3.15` no later than
2026-09-05, and require focused parity tests plus the full binary check before
closure.

Because temporary Grok adoption deferrals remain, `bc7f02ed` is not marked
Reviewed and no zero-tree-delta upstream acknowledgement merge was created.

## Provider reference queues

- `CDX-6478-AUTH-CATALOG-WIRE` keeps the 308-commit Codex authentication,
  catalog, Responses/Responses Lite, compaction, hosted-tool, usage, retry, and
  error matrix open. The earlier `CDX-E482-CONTEXT-WINDOW-ID` obligation is
  carried forward and remains overdue.
- `CDX-6478-APP-BEHAVIOR` keeps Codex application changes open where their
  observable behavior may map to an existing Grok surface; Codex app-server or
  TUI architecture is not imported.
- `OC-DC44-CODEX-INTEROP`, `KIMI-9D23-PROVIDER`, and
  `OMP-33CC-HARNESS` keep the advancing OpenCode, Kimi, and Oh My Pi ranges
  explicit instead of silently advancing Reviewed.
- `CODEXBAR-CE44-ZAI-USAGE` keeps the CodexBar range as usage research. It does
  not enable a quota endpoint or forward a Coding Plan credential.
- `MODELS-EFDB-CATALOG` is not applicable to authenticated runtime selection:
  third-party generated metadata cannot override first-party provider
  contracts.

## Z.AI GLM Coding Plan adoption

Repository policy now authorizes an experimental, explicit Z.AI Coding Plan
API-key provider. The implementation was independently integrated against
current public Z.AI documentation and the reviewed GLM-5.3 source range; no
credential, authenticated payload, or unreviewed source code was copied.

The adopted runtime has:

- provider identity `zai_coding_plan`, credential source
  `zai_coding_plan_api_key`, auth scope `zai::coding-plan::global`, and optional
  `Z_AI_API_KEY` fallback;
- the fixed global endpoint `https://api.z.ai/api/coding/paas/v4`, Chat
  Completions only, one sensitive bearer header, no redirects, protected-header
  sealing, bounded response bodies, and fixed-shape error diagnostics;
- the audited `glm-5.3`, `glm-5.3[1m]`, `glm-5.3-flash`, and
  `glm-5.3-flash[1m]` catalog, with text/image capability separation and
  `low`/`high`/`max` effort normalization;
- CLI login, model listing, logout, auth-store preservation/repair, session and
  ACP routing, credential-record attestation, and auxiliary-model binding; and
- no browser/OAuth login, Open Platform or BigModel endpoint fallback, usage
  endpoint, or Z.AI-hosted MCP credential forwarding.

The adapter is offline-qualified because no entitled credential was available.
Live inference and live rejection behavior remain credential-gated and are not
claimed as tested.

## Warp and GLM source closure

Warp commit `4154072d` only moves the byte-identical Paper Botanical light and
dark YAML themes from `standard` to `warp_bundled` (and moves previews, which
Enhanced does not ship). The deterministic vendor tool applied the two YAML
moves and regenerated the vendor manifest without changing theme bytes.

GLM-5 commit `414ad9eb` adds the GLM-5.3 and GLM-5.3-Flash model records and
documents their `low`, `high`, and `max` reasoning levels with `max` as the
default. Combined with current first-party Z.AI Coding Plan and API references,
that range supports the runtime catalog and request mapping described in the
provider reference. Warp and GLM-5 may therefore advance Reviewed after their
focused validation passes; the other advancing sources remain queued.

## Validation

The isolated candidate passed:

- `cargo fmt --all -- --check`;
- `cargo test -p xai-grok-sampler --lib zai_coding_plan` — 6 passed;
- `cargo test -p xai-grok-shell --lib zai_coding_plan` — 10 passed;
- the Z.AI active credential-record pinning session test — 1 passed;
- the Z.AI CLI provider parsing test — 1 passed;
- `vendor_warp_themes.py --check --revision 4154072d…` — 343 files and the
  audited `178/135/8/1/21` category split accepted;
- the Warp vendor build-validation target — 2 passed; and
- `CARGO_INCREMENTAL=0 cargo check -p xai-grok-pager-bin`.

The strict ownership checker is run against the committed thematic candidate,
because new manifest integration paths are intentionally required to exist in
candidate history. Live Z.AI authentication and inference were not run without
an entitled credential. Existing unused shared-HTTP and pager mode-switch
warnings remain outside this change.
