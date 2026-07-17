# Upstream versions

Last checked: 2026-07-17T12:53:08Z

This file records both the source revision last reviewed for this fork and the
newest fetched revision. A difference is a review queue, not permission to
blindly copy upstream code.

| Project | Remote and tracked ref | Last reviewed / fork baseline | Latest fetched |
| --- | --- | --- | --- |
| OpenAI Codex CLI | `https://github.com/openai/codex.git` `main` | [`18110b810f0a328147f6cd85e6f1ab6414927366`](https://github.com/openai/codex/commit/18110b810f0a328147f6cd85e6f1ab6414927366) | [`315195492c80fdade38e917c18f9584efd599304`](https://github.com/openai/codex/commit/315195492c80fdade38e917c18f9584efd599304) |
| OpenCode | `https://github.com/anomalyco/opencode.git` `dev` | [`1d2a7b4c860f6a29eb90bdda07757b2adf34ab61`](https://github.com/anomalyco/opencode/commit/1d2a7b4c860f6a29eb90bdda07757b2adf34ab61) | [`efb6cc2d4bf6332eb156709795d2b3a649198b65`](https://github.com/anomalyco/opencode/commit/efb6cc2d4bf6332eb156709795d2b3a649198b65) |
| Grok Build upstream | `https://github.com/xai-org/grok-build.git` `main` | Fork base [`c1b5909ec707c069f1d21a93917af044e71da0d7`](https://github.com/OpenCompanyApp/grok-build-codex/commit/c1b5909ec707c069f1d21a93917af044e71da0d7) | [`8adf9013a0929e5c7f1d4e849492d2387837a28d`](https://github.com/xai-org/grok-build/commit/8adf9013a0929e5c7f1d4e849492d2387837a28d) |
| OpenCode Codex auth reference | `https://github.com/numman-ali/opencode-openai-codex-auth.git` `main` | [`bec2ad69b252ef4ad7dd33b9532ff8b4fdb6d016`](https://github.com/numman-ali/opencode-openai-codex-auth/commit/bec2ad69b252ef4ad7dd33b9532ff8b4fdb6d016) | same |
| Warp themes | `https://github.com/warpdotdev/themes.git` `main` | [`b385044250f1ed3c9379ab34a8fe82f02fdffaa4`](https://github.com/warpdotdev/themes/commit/b385044250f1ed3c9379ab34a8fe82f02fdffaa4) | same |

## Refresh procedure

1. Fetch `origin` in `inspiration/openai-codex`, `inspiration/opencode`, and
   `inspiration/warp-themes`, and fetch `upstream/main` in this repository.
2. Compare the old and new revisions, concentrating on login, auth storage,
   model-provider metadata, Responses transport, standalone search, image
   tools, usage limits, token refresh behavior, and Warp theme catalog/license
   changes.
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
