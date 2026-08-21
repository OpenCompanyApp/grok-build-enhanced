# Provider and prior-refresh audit — 2026-08-21

This audit refreshes every tracked provider, interoperability, theme, and research source after the 2026-08-20 Grok `19d42e35c07a9c9244f03f6df0c4c353f970d4f9` acknowledgement. Grok itself did not advance, so the existing zero-tree-delta acknowledgement remains authoritative and no new acknowledgement merge is created.

## Prior-refresh residue

Every retained refresh worktree was inspected again before this audit. All are clean except `grok-build-enhanced-refresh-20260809`, which still has 153 staged paths, ten unstaged paths, and one untracked ledger. It remains intentionally untouched as forensic evidence.

The dirty worktree has 160 distinct changed or untracked paths. Of those, 153 working-tree blobs are exactly reachable from `origin/main`. The remaining seven records are two intentional test-file deletions plus five superseded source-pin, manifest, parity, checker, or refresh-ledger drafts. The two documentation-index omissions identified on 2026-08-20 are present on `origin/main`. The only clean branch with non-patch-equivalent commits is the August 2 branch: its two unique commits contain obsolete source pins and audit documentation, while its runtime adoption commit is patch-equivalent to `origin/main`. No runtime implementation, provider behavior, or open parity obligation is stranded in an older refresh worktree.

The frozen obligation ledger still contains 120 closed Grok obligations and eight provider checks qualified for offline execution. Before this campaign the current ledger contained 107 closed obligations and no open deferrals.

## Immutable fetched identities

| Source | Previous reviewed | Audited head |
| --- | --- | --- |
| Grok Build | `19d42e35c07a9c9244f03f6df0c4c353f970d4f9` | unchanged |
| OpenAI Codex | `478dbe9df0a33141d265db5977947cc432e7fe85` | `e482cc66aeeedcb9f333a1f5a0a554eb5aea4b36` |
| OpenCode | `b155b15694dbcc6768f11d2f25cc2bdd1f738ab4` | `1b937c860b6fd8a83e69f916b1236515aa17ea0d` |
| Oh My Pi | `72000acfeb902e21816252699482887f34d1a5a4` | `9350b7990d26ebf69a604edc82d8558ef04adf30` |
| Warp themes | `82e51dcf9b47912d551107748ba3297a21b2eff3` | `6cc44d7b32baaf979a249056e35fff834cb39547` |
| Kimi Code | `3899079a2c851bd0b3f1cbf1d3d2fd9026fc6abb` | `d4e0ad4b2d04d676b6d139ee320ea162289d3f4b` |
| CodexBar | `e92d89ab20037e4b3ba2028e22a4f5aef2929ab1` | `f74117aeb7a9ee02a78c0f08ca354ff26b2292e0` |
| models.dev | `4b494b2702c0155e596765ea3f0d007a688f7fbb` | `a166d7e2bec7603d54e2ed1c4a79432089c4a657` |
| Exa MCP server | `66bacbe4afd35a7e1671be9ab55c2b6bf60aff34` | `15ffb50519e719dc791cdc750ce5ed1934c0a1ed` |

OpenCode Codex auth, Kimi CLI, Z.AI coding plugins, GLM-5, and the Z.AI usage helper were unchanged. The configured Z.AI SDK URL again returned `Repository not found`; its recorded immutable identity remains unchanged and no replacement repository was inferred.

## Changed-source inventory

Hashes are SHA-256 over exact `git diff --no-renames --raw --abbrev=40` output.

| Source | Commits | Raw paths | Target tree | Raw hash |
| --- | ---: | ---: | --- | --- |
| OpenAI Codex | 72 | 481 | `038ae323a08dec8794e54eba856090bffd1a1f0f` | `ce1c3b8c997024e1c8b6136500edfc17fbcfe2dae4b33179565c4250c9d5160b` |
| OpenCode | 31 | 118 | `71f8f8c767a008b5d9cc18cf094e331960196be7` | `6b7f5aaebaaf2c0fe32a6214085d06fe658e84bf25c676787196d13873a6eb22` |
| Oh My Pi | 220 | 469 | `0ca3c8c28f8abf001777a0b1d46908eff585b581` | `33fd1905a85f4c7b143465033f632a788a534d59e9e90fd28194f129cc63c4f2` |
| Warp themes | 1 | 5 | `a4d063cf3801566f9ae4fc4bf57556218eb07a15` | `0063ceb662aa6b6d2b8622772f7b47c96f958e109d419fa14f519d05a37d0a12` |
| Kimi Code | 16 | 428 | `5e91933be7d02114c4a5352fab417bb10fdca254` | `838f39060136c35ca430972f37a0be0fafbcc00c7e1b732e5f9e03a54b4dfffd` |
| CodexBar | 15 | 84 | `4007ca996d887763551eef34c1b58203841837c5` | `9ae50dda96a8e62310cd114b7a46b110beeb4d994879309df6b51b80a11aa4e8` |
| models.dev | 85 | 666 | `d27e678e3db31fbc73424173a83ce9aace2fc894` | `e7d46b06d9bf8a5ce1d1608752bf2959525c14d96edc6771702182a0d482b235` |
| Exa MCP server | 2 | 2 | `d1dfd2ab3e111bc200b1df04795cc2b3e505b2a8` | `c1ee1ac1be862334156de10964295aff5a0073f807327672c91f5bfd6c2235fd` |

## Provider behavior matrix

| Upstream behavior | Enhanced behavior | Outcome |
| --- | --- | --- |
| Codex adds a stable `context_window_id` to normal and compaction request metadata and rotates it after compaction. | Enhanced sends a bounded, provider-owned turn envelope but has no context-window UUID lifecycle and currently omits the envelope on compaction requests. | **temporarily deferred** as `CDX-E482-CONTEXT-WINDOW-ID`. |
| Codex refreshes model definitions while retaining minimum client version `0.144.0`; adds context-window response metadata, custom-provider turn-cost telemetry, standalone tool-output handling, and Bedrock Responses compaction. | Authenticated Codex catalog metadata remains authoritative; Enhanced already identifies as `0.144.0`, preserves opaque tool results, separates authoritative subscription usage from estimates, and uses provider-scoped compaction. Bedrock and custom-provider orchestration do not alter the subscription adapter. | **already equivalent** or **not applicable** under `CDX-E482-EQUIVALENCE` and `CDX-E482-SCOPE`. |
| Codex changes app-server, realtime, Guardian, unified-exec, executor MCP, sandbox, analytics, and official release behavior. | Enhanced retains Grok's agent loop, TUI, tools, permissions, SSE sampler, and fork-owned release route. | **not applicable** under the standing architecture and release scope. |
| OpenCode continues unknown finish reasons, retries raw network variants and xAI capacity streams, and surfaces subagent failures. | Enhanced keeps unknown provider finish data non-terminal until the stream closes, classifies bounded transport/capacity retry signals, and owns subagent display in the Grok application. No Codex OAuth or credential-wire change occurred. | **already equivalent** for provider retry behavior; application changes are **not applicable**. |
| Oh My Pi propagates token-derived Codex residency to response, compaction, image, and search requests; preserves explicit refresh after auth retry; and restores discovery cache headers. | One account-bound `CodexAuthSnapshot` supplies a sensitive residency header to sampler, catalog, search, image, and tool adapters; credential generation scopes caches and refresh recovery. Credential headers are never persisted into the model cache. | **already equivalent**; Oh My Pi remains non-normative harness evidence. |
| Kimi Code changes tower/subagent/session behavior, server API documentation, TUI, marketplace, generated web assets, and usage presentation. | No authenticated catalog, API-key header, inference stream, reasoning, hosted web, usage, retry, or logout contract changed in the range. | **not applicable** to the scoped Kimi adapter; existing provider isolation remains equivalent. |
| Warp adds Paper Botanical light and dark themes without changing its license. | Both YAML themes are byte-preserved in the deterministic vendor corpus; previews and source README changes remain unshipped. | **adopt** as `WARP-6CC4-PAPER-BOTANICAL`. |
| CodexBar adds xAI spend and Z.AI balance research; models.dev refreshes third-party catalogs; Exa changes release workflows. | Research sources cannot create a Z.AI runtime provider, third-party catalogs cannot override authenticated first-party catalogs, and Exa packaging does not change the MCP client contract. | **not applicable**. |

## Open provider obligation

`CDX-E482-CONTEXT-WINDOW-ID` is owned by the Codex adapter. The blocker is the absence of a process-local context-window generation in the Grok session and compaction paths. Until it lands, provider-side diagnostics and policy correlation cannot distinguish pre- and post-compaction model-visible contexts. Acceptance requires one opaque identifier per model-visible context, reuse across normal/tool-follow-up and compaction requests, rotation only after a successful compaction, omission for non-Codex providers, reserved-key protection, and no persistence or logging of the identifier. The target is the next refresh or `v0.3.13`, no later than 2026-08-28, with normal-turn, tool-follow-up, compaction-rotation, resume/fork, malformed-metadata, and cross-provider negative tests.

## Outcome

Every fetched delta is classified. Two Warp themes are adopted, existing provider behavior is retained with explicit evidence, and one Codex provider-wire obligation remains openly deferred. There are no Grok changes or Grok deferrals, so the existing Grok acknowledgement remains valid. The eight credential/network-gated provider checks remain offline-qualified and unchanged.
