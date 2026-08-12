# Upstream refresh rerun — 2026-08-12

This rerun starts from Enhanced commit
`b8208c6a9603ac3d34a1f5b9285146bb754dd2dc`, tree
`f284550441f192fe097d7e4a4ff68e4c98aa0a26`, in the isolated
`refresh/upstreams-20260812-r2` worktree. It freezes every tracked head again,
audits the four sources that advanced after the earlier August 12 audit, and
reconciles the active adoption queue. It does not advance a Reviewed revision,
create an upstream acknowledgement, or authorize publication.

## Frozen source heads

| Source | Frozen commit | Tree / availability | Change since the earlier August 12 audit |
| --- | --- | --- | --- |
| Grok Build | `be713136d2a69080743a3f6b3c72077057e5948f` | `ee8039b440cd38a62c1e007b5cd55c6bbe366aa4` | unchanged |
| OpenAI Codex | `3d7bb2dd2e834b4d26cf29a7c0163dd4fb5afb70` | `7e2cdec7e06f6efdc89bd720d19ebb3db7a7d5fd` | 2 commits / 28 paths |
| OpenCode | `dab2637217f188afca5e6631f67b935723e6218a` | `aa0ab4fb112485c0df2418a747ebd7def70cb39f` | 1 commit / 8 paths |
| OpenCode Codex auth | `bec2ad69b252ef4ad7dd33b9532ff8b4fdb6d016` | `1da59bae7069563b2817143567b57c78e5758300` | unchanged |
| Oh My Pi | `06aecdd51f07e689e970ceaa180abe2be0c14bbb` | `83c6d8c700438bc4746a0c4fa170932b785f3aa0` | unchanged |
| Warp themes | `82e51dcf9b47912d551107748ba3297a21b2eff3` | `2893387a4769db78ce4ef5294b8cc39bacd80616` | unchanged |
| Kimi Code | `719da946480261dc5b4228f522f24973d3b5bd68` | `83b1e9ca72fb8f5746d5cc6c3a4143eda9872c06` | unchanged |
| Kimi CLI | `cbc15c076d17f70fec9f89c90c0502e68657f505` | `e1d6d5b2827f8a14c2edc4bc8658ad5cf19d52e7` | unchanged |
| Z.AI Python SDK | `ca5109c0aa9bf173839be391b4b14aeadf9a9bf9` | tracked repository still returns `Repository not found` | retained immutable pin |
| Z.AI coding plugins | `0446d0bb0bc537d97d3ab3664c4b8b9c4a0e1254` | `efea84479dc67bc4af7d2c3b59b4aca8f5332899` | unchanged |
| GLM-5 | `25206af860c4ac10f6411c597c574f9b1c00e53c` | `573d8342bcfc2e21d27e210c47a99a4604fc39ee` | unchanged |
| CodexBar | `ee29794b9a1b6020ba97e3fd2303f3c9902a616c` | `b3d35c731e8692e66e5fbf1e01089fa0acdb7324` | 1 commit / 2 paths |
| Z.AI usage helper | `54cd1f33a703c417f2492ee1f21f22b3633a43c4` | `08b00849b96c5883a265f4d4d43e2836d01cdd9d` | unchanged |
| models.dev | `0370588c96e4eaeba4dfd5a4b387c0531302c4cc` | `0281d27fec187efb07ae1debe2e724def6552cba` | 1 commit / 1 path |
| Exa MCP server | `e64c11f2d3b4400ffbda8ccdd9658a450cc9d270` | `569db78ece8c6a13f6f4afeefe05e569a57cb09e` | unchanged |

Each successful changed pin is a descendant of the preceding pin. Fetches did
not move or clean any inspiration checkout.

## Delta classifications

| Source | New behavior | Classification |
| --- | --- | --- |
| OpenAI Codex `74004b5` | Carries model-catalog Node REPL policy fields into reserved turn metadata, including review turns. Enhanced has no Node REPL tool and does not advertise that Codex surface. | not applicable (`CDX-3D7B-SCOPE`) |
| OpenAI Codex `3d7bb2d` | Caches stable active-cell TUI layout measurements at the bottom of scrollback. Enhanced preserves the Grok TUI rather than the Codex TUI. | not applicable (`CDX-3D7B-SCOPE`) |
| OpenCode `dab2637` | Reworks V1/V2 compaction prompts, complete-message partitioning, and recent-tail budgeting for OpenCode's replacement agent harness. It changes neither Codex OAuth nor provider wire interoperability. | not applicable (`OC-DAB2-COMPACTION`) |
| CodexBar `ee29794` | Opens the 0.49.4 development version. | not applicable (`CODEXBAR-EE29-RELEASE`) |
| models.dev `0370588` | Adds an OpenRouter Liquid LFM2.5 model catalog entry. Authenticated first-party provider catalogs remain authoritative in Enhanced. | not applicable (`MODELS-0370-CATALOG`) |

The Codex Node REPL policy fields do not close the existing bounded turn
metadata obligation: that obligation concerns metadata used by Enhanced's
actual direct-subscription requests. Importing policy for a tool Enhanced does
not expose would add a false compatibility surface.

## Active queue and recent-refresh reconciliation

No stable obligation disappeared. The active ledger now contains 53 records:
32 remain open, including 26 Grok adoption obligations; the four added records
are closed scope classifications for this rerun.

The August 9 adoption worktree remains deliberately preserved at
`d971891ed28cc0aa0d64c1f3b61e7820b1ba9f96`. It still has 153 staged files,
10 tracked unstaged files, and one untracked audit ledger. Its staged candidate
contains 10,671 insertions and 4,811 deletions. Because it remains uncommitted,
has mixed staged and unstaged state, and has not passed final candidate
validation, it is evidence for `GB-8A14-LANDING`, not a reviewed boundary.

The earlier August 12 Grok pin is unchanged. Therefore the exhaustive
211-path `b13fa526..be713136` ledger and its raw digest
`6c97524fd584dd4873bf7e5df675599050c25a531b27b4541a50f1eecc614275`
remain the controlling Grok evidence. No second Grok path ledger is necessary
for a zero-tree-delta rerun.

## Provider matrix

| Surface | Result |
| --- | --- |
| xAI / Grok Build | Source tree unchanged; all prior preserved-surface obligations remain open and provider-isolated. |
| OpenAI Codex subscription | No new applicable wire or credential behavior. Existing turn-metadata, safety, image-limit, model-history, root-turn, and harness-history obligations remain open. |
| Kimi Code | Source tree unchanged; API-key auth, authenticated discovery, provider web capabilities, retry ownership, and logout remain isolated. |
| Generic custom providers | No changed source authorizes Codex/Kimi credentials, metadata, catalog state, or retry policy to fall through. |
| OpenCode / Oh My Pi | New OpenCode compaction behavior is replacement-harness architecture; Oh My Pi is unchanged and remains non-normative inspiration. |
| Z.AI research | No runtime provider, login route, credentials, or product claim is established. CodexBar's version bump changes no usage protocol. |

## Acknowledgement and publication decision

Grok acknowledgement remains ineligible because 26 Grok adoption obligations
are open. Reviewed pins remain unchanged. No source change in this rerun is an
applicable implementation gap, so no runtime code was ported.

This refresh request does not provide a release version or current-run
publication authorization. No push, tag, GitHub release, Homebrew tap mutation,
or pull-request mutation is permitted or performed.

## Exhaustive changed-source raw evidence

The exact no-renames raw streams are bound by these SHA-256 digests:

| Source range | Raw rows | SHA-256 |
| --- | ---: | --- |
| Codex `d6eefb26..3d7bb2dd` | 28 | `ffe483ee7dd964d0b8a12d1f7daf38c311e3bb926ba1a7cdcd108574f62ffc1a` |
| OpenCode `39fb919a..dab26372` | 8 | `5822c1ac6eebc8d11a2733d1483314c647a3b82f15d316f514eb721c1f1527ff` |
| CodexBar `fc57a317..ee29794b` | 2 | `66716e290cfb107e7dde61a6da8380d0a6b3459c7c52ea0b4c08414d471498ac` |
| models.dev `40058d76..0370588c` | 1 | `d0747ac78f9c8c1ded516da5f8dff04f63ae38f467c18e48c08b4d941c5df89c` |

Every changed path is classified by the corresponding closed obligation:

- `CDX-3D7B-SCOPE`: `codex-rs/app-server/tests/common/models_cache.rs`,
  `codex-rs/codex-api/tests/models_integration.rs`,
  `codex-rs/core/src/mcp_tool_call_tests.rs`,
  `codex-rs/core/src/responses_metadata.rs`,
  `codex-rs/core/src/session/review.rs`,
  `codex-rs/core/src/session/turn_context.rs`,
  `codex-rs/core/src/turn_metadata.rs`,
  `codex-rs/core/src/turn_metadata_tests.rs`,
  `codex-rs/core/tests/suite/agent_websocket.rs`,
  `codex-rs/core/tests/suite/auto_review.rs`,
  `codex-rs/core/tests/suite/model_switching.rs`,
  `codex-rs/core/tests/suite/models_cache_ttl.rs`,
  `codex-rs/core/tests/suite/personality.rs`,
  `codex-rs/core/tests/suite/remote_models.rs`,
  `codex-rs/core/tests/suite/review.rs`,
  `codex-rs/core/tests/suite/rmcp_client.rs`, the two changed compact-remote
  snapshots, `codex-rs/core/tests/suite/spawn_agent_description.rs`,
  `codex-rs/core/tests/suite/view_image.rs`,
  `codex-rs/models-manager/src/model_info.rs`,
  `codex-rs/protocol/src/openai_models.rs`,
  `codex-rs/tools/src/tool_config_tests.rs`,
  `codex-rs/tui/src/chatwidget.rs`,
  `codex-rs/tui/src/chatwidget/rendering.rs`,
  `codex-rs/tui/src/chatwidget/rendering_tests.rs`, the new active-cell
  rendering snapshot, and `codex-rs/tui/src/chatwidget/transcript.rs`.
- `OC-DAB2-COMPACTION`: `packages/core/src/plugin/agent.ts`,
  `packages/core/src/session/compaction.ts`,
  `packages/core/src/v1/config/config.ts`,
  `packages/core/test/session-compaction.test.ts`,
  `packages/core/test/session-runner.test.ts`,
  `packages/opencode/src/agent/prompt/compaction.txt`,
  `packages/opencode/src/session/compaction.ts`, and
  `packages/opencode/test/session/compaction.test.ts`.
- `CODEXBAR-EE29-RELEASE`: `CHANGELOG.md` and `version.env`.
- `MODELS-0370-CATALOG`:
  `providers/openrouter/models/liquid/lfm-2.5-2.6b:free.toml`.
