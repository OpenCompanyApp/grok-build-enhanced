# Upstream versions

Last checked: 2026-07-20T23:24:17Z

This file records both the source revision last reviewed for this fork and the
newest fetched revision. A difference is a review queue, not permission to
blindly copy upstream code.

| Project | Remote and tracked ref | Last reviewed / fork baseline | Latest fetched |
| --- | --- | --- | --- |
| OpenAI Codex CLI | `https://github.com/openai/codex.git` `main` | [`3e2f79727a4e8ddfc8e3acb838d496b121094b9e`](https://github.com/openai/codex/commit/3e2f79727a4e8ddfc8e3acb838d496b121094b9e) | same |
| OpenCode | `https://github.com/anomalyco/opencode.git` `dev` | [`67caf894e0843ee370e72839e8265e483233479b`](https://github.com/anomalyco/opencode/commit/67caf894e0843ee370e72839e8265e483233479b) | same |
| Grok Build upstream | `https://github.com/xai-org/grok-build.git` `main` | Reviewed [`a881e6703f46b01d8c7d4a5437683546df30449d`](https://github.com/xai-org/grok-build/commit/a881e6703f46b01d8c7d4a5437683546df30449d) | same |
| OpenCode Codex auth reference | `https://github.com/numman-ali/opencode-openai-codex-auth.git` `main` | [`bec2ad69b252ef4ad7dd33b9532ff8b4fdb6d016`](https://github.com/numman-ali/opencode-openai-codex-auth/commit/bec2ad69b252ef4ad7dd33b9532ff8b4fdb6d016) | same |
| Warp themes | `https://github.com/warpdotdev/themes.git` `main` | [`b385044250f1ed3c9379ab34a8fe82f02fdffaa4`](https://github.com/warpdotdev/themes/commit/b385044250f1ed3c9379ab34a8fe82f02fdffaa4) | same |
| Kimi Code | `https://github.com/MoonshotAI/kimi-code.git` `main` | [`df6899553962d1764c9f4c3bec1b63c811cb425e`](https://github.com/MoonshotAI/kimi-code/commit/df6899553962d1764c9f4c3bec1b63c811cb425e) | same |
| Kimi CLI (legacy reference) | `https://github.com/MoonshotAI/kimi-cli.git` `main` | [`4a550effdfcb29a25a5d325bf935296cc50cd417`](https://github.com/MoonshotAI/kimi-cli/commit/4a550effdfcb29a25a5d325bf935296cc50cd417) | same |
| Z.AI Python SDK | `https://github.com/zai-org/z-ai-sdk-python.git` `main` | [`ca5109c0aa9bf173839be391b4b14aeadf9a9bf9`](https://github.com/zai-org/z-ai-sdk-python/commit/ca5109c0aa9bf173839be391b4b14aeadf9a9bf9) | same |
| Z.AI coding plugins | `https://github.com/zai-org/zai-coding-plugins.git` `main` | [`0446d0bb0bc537d97d3ab3664c4b8b9c4a0e1254`](https://github.com/zai-org/zai-coding-plugins/commit/0446d0bb0bc537d97d3ab3664c4b8b9c4a0e1254) | same |
| GLM-5 model reference | `https://github.com/zai-org/GLM-5.git` `main` | [`436efa09bc868a6922e307624189e7018406beb9`](https://github.com/zai-org/GLM-5/commit/436efa09bc868a6922e307624189e7018406beb9) | same |
| CodexBar Z.AI usage reference | `https://github.com/steipete/CodexBar.git` `main` | [`f8636cb37eb0f96d261604ee94e6481496aadfeb`](https://github.com/steipete/CodexBar/commit/f8636cb37eb0f96d261604ee94e6481496aadfeb) | same |
| Z.AI usage browser reference | `https://github.com/nniicckk6/zai-extention.git` `main` | [`54cd1f33a703c417f2492ee1f21f22b3633a43c4`](https://github.com/nniicckk6/zai-extention/commit/54cd1f33a703c417f2492ee1f21f22b3633a43c4) | same |

## Refresh procedure

1. Fetch `origin` in `inspiration/openai-codex`, `inspiration/opencode`,
   `inspiration/warp-themes`, `inspiration/kimi-code`,
   `inspiration/kimi-cli`, `inspiration/zai-sdk-python`,
   `inspiration/zai-coding-plugins`, `inspiration/glm-5`,
   `inspiration/codexbar`, and `inspiration/zai-usage-helper`; fetch
   `upstream/main` in this repository.
2. Compare the old and new revisions, concentrating on login, auth storage,
   model-provider metadata, Responses and Chat Completions transport,
   standalone search, image tools, usage limits, token refresh behavior,
   Kimi model and managed-service contracts, Z.AI model and MCP contracts,
   Z.AI monitoring schema drift, and Warp theme catalog/license changes.
3. Update **Latest fetched** immediately. Update **Last reviewed** only after
   the relevant diff has been read and any required compatibility changes and
   notices have been applied and tested.
4. Keep the ignored `inspiration/` clones out of commits. Never import
   credentials or `~/.codex/auth.json`.

An inspiration checkout may lag its fetched remote-tracking ref. At the check
above, `inspiration/openai-codex` was at
`5331d20f6ef9b80ee4153132a70d4989780d916d` while `origin/main` was at the
newer revision recorded in the table. Fast-forward the ignored checkout before
reading it, or inspect the recorded revision explicitly with commands such as
`git show <revision>:<path>`.

The xAI upstream may be republished from a monorepo without a usable merge
base. In that case compare the relevant paths or release snapshots directly
instead of assuming a normal linear Git history.
