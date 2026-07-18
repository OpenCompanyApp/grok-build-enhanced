# Official Grok npm package (upstream compatibility material)

> [!WARNING]
> This inherited npm directory describes the official xAI/SpaceXAI package. It
> is **not** a Grok Build Enhanced distribution route and installing it replaces
> the fork with the official build. Enhanced is published only from
> [`OpenCompanyApp/grok-build-enhanced`](https://github.com/OpenCompanyApp/grok-build-enhanced/releases)
> using exact native GitHub Release assets, or built from that source tree.

Bring official Grok Build into your terminal. Fast, flicker-free CLI built for
plans, subagents, and parallel work.

**[Official homepage](https://x.ai/cli)** | **[Official documentation](https://docs.x.ai/build/overview)**

## Install the official upstream distribution

```bash
curl -fsSL https://x.ai/cli/install.sh | bash
```

Or install with npm:

```bash
npm i -g @xai-official/grok
```

## Get Started

```bash
# Launch the interactive TUI
grok

# Run a single task
grok -p "Explain this codebase"
```

On first launch, Grok opens your browser to authenticate. For CI or headless environments, use an API key from [console.x.ai](https://console.x.ai):

```bash
export XAI_API_KEY="xai-..."
```

## Update the official upstream package

```bash
grok update
```

Or, for this official npm distribution:

```bash
npm i -g @xai-official/grok@latest
```

## Supported Platforms

| Platform | Architecture |
|---|---|
| macOS | Apple Silicon (arm64) |
| Linux | x86_64, arm64 |
| Windows | x86_64 |

## Documentation

For full documentation including configuration, MCP servers, custom models, headless mode, agent mode, and more, visit [docs.x.ai/build/overview](https://docs.x.ai/build/overview).

## Feedback

Run `/feedback` inside Grok to report issues or send feedback directly.
