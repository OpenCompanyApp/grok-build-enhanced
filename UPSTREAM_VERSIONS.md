# Upstream versions

Last checked: 2026-08-20

This file records both the source revision last reviewed for this fork and the
newest fetched revision. A difference is a review queue, not permission to
blindly copy upstream code.

| Project | Remote and tracked ref | Last reviewed / fork baseline | Latest fetched |
| --- | --- | --- | --- |
| OpenAI Codex CLI | `https://github.com/openai/codex.git` `main` | [`478dbe9df0a33141d265db5977947cc432e7fe85`](https://github.com/openai/codex/commit/478dbe9df0a33141d265db5977947cc432e7fe85) | same |
| OpenCode | `https://github.com/anomalyco/opencode.git` `dev` | [`b155b15694dbcc6768f11d2f25cc2bdd1f738ab4`](https://github.com/anomalyco/opencode/commit/b155b15694dbcc6768f11d2f25cc2bdd1f738ab4) | same |
| models.dev | `https://github.com/sst/models.dev.git` `dev` | [`4b494b2702c0155e596765ea3f0d007a688f7fbb`](https://github.com/sst/models.dev/commit/4b494b2702c0155e596765ea3f0d007a688f7fbb) | same |
| Exa MCP server | `https://github.com/exa-labs/exa-mcp-server.git` `main` | [`66bacbe4afd35a7e1671be9ab55c2b6bf60aff34`](https://github.com/exa-labs/exa-mcp-server/commit/66bacbe4afd35a7e1671be9ab55c2b6bf60aff34) | same |
| Grok Build upstream | `https://github.com/xai-org/grok-build.git` `main` | Reviewed [`19d42e35c07a9c9244f03f6df0c4c353f970d4f9`](https://github.com/xai-org/grok-build/commit/19d42e35c07a9c9244f03f6df0c4c353f970d4f9) | same |
| OpenCode Codex auth reference | `https://github.com/numman-ali/opencode-openai-codex-auth.git` `main` | [`bec2ad69b252ef4ad7dd33b9532ff8b4fdb6d016`](https://github.com/numman-ali/opencode-openai-codex-auth/commit/bec2ad69b252ef4ad7dd33b9532ff8b4fdb6d016) | same |
| Oh My Pi coding harness | `https://github.com/can1357/oh-my-pi.git` `main` | [`72000acfeb902e21816252699482887f34d1a5a4`](https://github.com/can1357/oh-my-pi/commit/72000acfeb902e21816252699482887f34d1a5a4) | same |
| Warp themes | `https://github.com/warpdotdev/themes.git` `main` | [`82e51dcf9b47912d551107748ba3297a21b2eff3`](https://github.com/warpdotdev/themes/commit/82e51dcf9b47912d551107748ba3297a21b2eff3) | same |
| Kimi Code | `https://github.com/MoonshotAI/kimi-code.git` `main` | [`3899079a2c851bd0b3f1cbf1d3d2fd9026fc6abb`](https://github.com/MoonshotAI/kimi-code/commit/3899079a2c851bd0b3f1cbf1d3d2fd9026fc6abb) | same |
| Kimi CLI (legacy reference) | `https://github.com/MoonshotAI/kimi-cli.git` `main` | [`cbc15c076d17f70fec9f89c90c0502e68657f505`](https://github.com/MoonshotAI/kimi-cli/commit/cbc15c076d17f70fec9f89c90c0502e68657f505) | same |
| Z.AI Python SDK | `https://github.com/zai-org/z-ai-sdk-python.git` `main` | [`ca5109c0aa9bf173839be391b4b14aeadf9a9bf9`](https://github.com/zai-org/z-ai-sdk-python/commit/ca5109c0aa9bf173839be391b4b14aeadf9a9bf9) | same |
| Z.AI coding plugins | `https://github.com/zai-org/zai-coding-plugins.git` `main` | [`0446d0bb0bc537d97d3ab3664c4b8b9c4a0e1254`](https://github.com/zai-org/zai-coding-plugins/commit/0446d0bb0bc537d97d3ab3664c4b8b9c4a0e1254) | same |
| GLM-5 model reference | `https://github.com/zai-org/GLM-5.git` `main` | [`25206af860c4ac10f6411c597c574f9b1c00e53c`](https://github.com/zai-org/GLM-5/commit/25206af860c4ac10f6411c597c574f9b1c00e53c) | same |
| CodexBar Z.AI usage reference | `https://github.com/steipete/CodexBar.git` `main` | [`e92d89ab20037e4b3ba2028e22a4f5aef2929ab1`](https://github.com/steipete/CodexBar/commit/e92d89ab20037e4b3ba2028e22a4f5aef2929ab1) | same |
| Z.AI usage browser reference | `https://github.com/nniicckk6/zai-extention.git` `main` | [`54cd1f33a703c417f2492ee1f21f22b3633a43c4`](https://github.com/nniicckk6/zai-extention/commit/54cd1f33a703c417f2492ee1f21f22b3633a43c4) | same |

The 2026-08-20 fetch of the tracked Z.AI Python SDK URL returned `Repository
not found`. Its immutable recorded pin was not changed, and no replacement
repository identity was inferred.

The 2026-08-20 review closes every fetched source range. Enhanced retains
provider-authoritative catalogs and isolated Codex/Kimi credentials, adds
friendly Codex education-plan labels and sensitive managed-residency routing,
and adopts the complete Grok `19d42e35` behavior range. Application-server,
OAuth-UI, TUI, release, and generic credential-header changes in inspiration
sources remain outside their declared adapter or research scope. The exact
classification and raw hashes are recorded in
`docs/upstream-refresh-2026-08-20-providers.md`; no source remains queued.

## Refresh procedure

1. Fetch `origin` in `inspiration/openai-codex`, `inspiration/opencode`,
   `inspiration/oh-my-pi`, `inspiration/warp-themes`, `inspiration/kimi-code`,
   `inspiration/kimi-cli`, `inspiration/zai-sdk-python`,
   `inspiration/zai-coding-plugins`, `inspiration/glm-5`,
   `inspiration/codexbar`, `inspiration/zai-usage-helper`,
   `inspiration/models-dev`, and `inspiration/exa-mcp-server`; fetch
   `upstream/main` in this repository.
2. Compare the old and new revisions, concentrating on login, auth storage,
   model-provider metadata, Responses and Chat Completions transport,
   standalone search, image tools, tool/model regression harnesses, language
   intelligence and debugger integrations, usage limits, token refresh behavior,
   Kimi model and managed-service contracts, Z.AI model and MCP contracts,
   Z.AI monitoring schema drift, and Warp theme catalog/license changes.
3. Update **Latest fetched** immediately. Update **Last reviewed** only after
   the relevant diff has been read and any required compatibility changes and
   notices have been applied and tested.
4. Keep the ignored `inspiration/` clones out of commits. Never import
   credentials or `~/.codex/auth.json`.

An inspiration checkout may lag its fetched remote-tracking ref. Inspect the
recorded revision explicitly with commands such as `git show <revision>:<path>`
instead of assuming the ignored checkout is at the fetched head.

The xAI upstream may be republished from a monorepo without a usable merge
base. In that case compare the relevant paths or release snapshots directly
instead of assuming a normal linear Git history.
