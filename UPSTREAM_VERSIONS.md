# Upstream versions

Last checked: 2026-08-29

This file records both the source revision last reviewed for this fork and the
newest fetched revision. A difference is a review queue, not permission to
blindly copy upstream code.

| Project | Remote and tracked ref | Last reviewed / fork baseline | Latest fetched |
| --- | --- | --- | --- |
| OpenAI Codex CLI | `https://github.com/openai/codex.git` `main` | Reviewed [`c9b19deb09c1841ce7acc33ddb96276030936a29`](https://github.com/openai/codex/commit/c9b19deb09c1841ce7acc33ddb96276030936a29) | same |
| OpenCode | `https://github.com/anomalyco/opencode.git` `dev` | Reviewed [`3a31c4ea801915c0b050df4b3842997ea62b6e93`](https://github.com/anomalyco/opencode/commit/3a31c4ea801915c0b050df4b3842997ea62b6e93) | same |
| models.dev | `https://github.com/sst/models.dev.git` `dev` | Reviewed [`59d624d4fb0c5c4f6fd44d1f701f17db1bd6ae4b`](https://github.com/sst/models.dev/commit/59d624d4fb0c5c4f6fd44d1f701f17db1bd6ae4b) | same |
| Exa MCP server | `https://github.com/exa-labs/exa-mcp-server.git` `main` | Reviewed [`15ffb50519e719dc791cdc750ce5ed1934c0a1ed`](https://github.com/exa-labs/exa-mcp-server/commit/15ffb50519e719dc791cdc750ce5ed1934c0a1ed) | same |
| Grok Build upstream | `https://github.com/xai-org/grok-build.git` `main` | Reviewed [`07b2f7144fd5c5c9d3dd1966937a87852d2dbdb8`](https://github.com/xai-org/grok-build/commit/07b2f7144fd5c5c9d3dd1966937a87852d2dbdb8) | same |
| OpenCode Codex auth reference | `https://github.com/numman-ali/opencode-openai-codex-auth.git` `main` | [`bec2ad69b252ef4ad7dd33b9532ff8b4fdb6d016`](https://github.com/numman-ali/opencode-openai-codex-auth/commit/bec2ad69b252ef4ad7dd33b9532ff8b4fdb6d016) | same |
| Oh My Pi coding harness | `https://github.com/can1357/oh-my-pi.git` `main` | Reviewed [`160ed439ac0df594347e7d7018b813a7ffdb5e81`](https://github.com/can1357/oh-my-pi/commit/160ed439ac0df594347e7d7018b813a7ffdb5e81) | same |
| Warp themes | `https://github.com/warpdotdev/themes.git` `main` | Reviewed [`4154072de03bea5dd070afa947ba6a6f2310ed5d`](https://github.com/warpdotdev/themes/commit/4154072de03bea5dd070afa947ba6a6f2310ed5d) | same |
| Kimi Code | `https://github.com/MoonshotAI/kimi-code.git` `main` | Reviewed [`ea0626ad48ee318045a22490d52c86be7d086033`](https://github.com/MoonshotAI/kimi-code/commit/ea0626ad48ee318045a22490d52c86be7d086033) | same |
| Kimi CLI (legacy reference) | `https://github.com/MoonshotAI/kimi-cli.git` `main` | [`cbc15c076d17f70fec9f89c90c0502e68657f505`](https://github.com/MoonshotAI/kimi-cli/commit/cbc15c076d17f70fec9f89c90c0502e68657f505) | same |
| Z.AI Python SDK | `https://github.com/zai-org/z-ai-sdk-python.git` `main` | [`ca5109c0aa9bf173839be391b4b14aeadf9a9bf9`](https://github.com/zai-org/z-ai-sdk-python/commit/ca5109c0aa9bf173839be391b4b14aeadf9a9bf9) | same |
| Z.AI coding plugins | `https://github.com/zai-org/zai-coding-plugins.git` `main` | [`0446d0bb0bc537d97d3ab3664c4b8b9c4a0e1254`](https://github.com/zai-org/zai-coding-plugins/commit/0446d0bb0bc537d97d3ab3664c4b8b9c4a0e1254) | same |
| GLM-5 model reference | `https://github.com/zai-org/GLM-5.git` `main` | Reviewed [`414ad9eb891b05b5d7d51d573939bfe9ce538223`](https://github.com/zai-org/GLM-5/commit/414ad9eb891b05b5d7d51d573939bfe9ce538223) | same |
| CodexBar Z.AI usage reference | `https://github.com/steipete/CodexBar.git` `main` | Reviewed [`4b14ed9c57d3506d1455b2736a1d1a8ff2b9c718`](https://github.com/steipete/CodexBar/commit/4b14ed9c57d3506d1455b2736a1d1a8ff2b9c718) | same |
| Z.AI usage browser reference | `https://github.com/nniicckk6/zai-extention.git` `main` | [`54cd1f33a703c417f2492ee1f21f22b3633a43c4`](https://github.com/nniicckk6/zai-extention/commit/54cd1f33a703c417f2492ee1f21f22b3633a43c4) | same |

The 2026-08-23 fetch of the tracked Z.AI Python SDK URL returned `Repository
not found`. Its immutable recorded pin was not changed, and no replacement
repository identity was inferred.

The 2026-08-29 GLM/Warp review advances the GLM-5 and Warp references. It
authorizes an offline-qualified, provider-isolated Z.AI GLM Coding Plan API-key
runtime and adopts Warp’s byte-identical Paper Botanical category move. All
other source review/fetch states remain unchanged in these thematic commits.

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
