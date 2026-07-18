# Grok Build Enhanced

**The unofficial daily-driver fork of Grok Build.**

Grok Build Enhanced is an unofficial daily-driver fork of Grok Build. It keeps
Grok Build's terminal agent and compatibility surfaces while integrating
carefully scoped provider, theme, tool, and user-experience enhancements.

> [!IMPORTANT]
> **Unofficial and independent.** Grok Build Enhanced is an unofficial
> daily-driver fork maintained independently by OpenCompanyApp. It is not
> affiliated with, endorsed by, or supported by xAI, SpaceXAI, OpenAI, or their
> affiliates. ChatGPT Codex subscription support follows current public
> open-source Codex client behavior and uses an experimental backend contract
> that may change without notice.

The executable remains `grok`. Existing `~/.grok` configuration, sessions,
model IDs, environment variables, Agent Client Protocol (ACP) identity, and the
responsive Grok braille symbol remain compatible.

| Area | Status in Enhanced |
| --- | --- |
| Grok Build TUI, agent loop, sessions, permissions, tools, headless mode, and ACP | Implemented; kept compatible with the upstream base |
| xAI login and models | Implemented upstream behavior; provider identity remains explicit |
| ChatGPT Codex subscription login, catalog, usage, fast mode, web/image tools | Implemented, experimental, and isolated from xAI credentials |
| Custom OpenAI-compatible endpoint path | Retained with explicit provider identity; custom entries use only their own configured credentials |
| Bundled Warp themes and theme UX | Implemented |
| Kimi Code managed provider | Researched/planned only; no Kimi runtime provider or login is shipped |
| Z.AI GLM Coding Plan provider | Researched/planned only; no GLM runtime provider or login is shipped |
| Enhanced release artifacts | Fork-owned macOS/Linux packaging pipeline implemented; no reviewed public release is currently claimed |
| Updates vs. upstream content | Enhanced update labels are fork-scoped; inherited announcements and release notes are labeled official xAI/upstream |

## Fork-owned terminal preview

This text preview represents the responsive welcome copy; runtime values are
shown as placeholders rather than fabricated release or account data. The
existing braille artwork appears when the terminal supports it and space allows.

```text
╭──────────────────────────────────────────────────────────────────────╮
│  [responsive Grok braille symbol]   Grok Build Enhanced              │
│     Enhanced <release> · upstream <base> · fork <revision>          │
│  The unofficial daily-driver fork of Grok Build.                    │
│                                     New worktree                     │
│                                     Resume session                   │
╰──────────────────────────────────────────────────────────────────────╯

Minimal mode: Grok Build Enhanced · <runtime provider/model/capabilities>
```

## Build and install Grok Build Enhanced

The fork source is hosted at
[github.com/OpenCompanyApp/grok-build-enhanced](https://github.com/OpenCompanyApp/grok-build-enhanced).
Build this repository before considering the official upstream installer; the
x.ai installer does **not** install Enhanced features.

Requirements:

- **Rust** — pinned by [`rust-toolchain.toml`](rust-toolchain.toml); `rustup`
  installs it automatically on first build.
- **protoc** — resolved through [`bin/protoc`](bin/protoc), or from
  `PATH` / `$PROTOC` as a fallback.
- macOS and Linux are supported build hosts. Windows builds are best-effort and
  are not currently tested from this tree.

Clone the fork and run it directly from the checkout:

```sh
git clone https://github.com/OpenCompanyApp/grok-build-enhanced.git
cd grok-build-enhanced
cargo run -p xai-grok-pager-bin
```

Or build a release artifact and install it as the compatible `grok` executable
on macOS/Linux:

```sh
cargo build -p xai-grok-pager-bin --release
mkdir -p "$HOME/.local/bin"
install -m 0755 target/release/xai-grok-pager "$HOME/.local/bin/grok"
"$HOME/.local/bin/grok" version
```

The Cargo artifact is named `xai-grok-pager`; distribution installs expose it
as `grok`. If `CARGO_TARGET_DIR` is set, copy from that target directory instead.
For Windows, build the same package and copy `target\release\xai-grok-pager.exe`
to a directory on `PATH` as `grok.exe`.

### Unofficial fork releases

Fork release assets, when published, are hosted only under
[`OpenCompanyApp/grok-build-enhanced`](https://github.com/OpenCompanyApp/grok-build-enhanced/releases).
The fork-owned pipeline builds native macOS and Linux binaries for Arm64 and
x86-64, names the installed executable `grok`, and attaches SHA-256 checksums
and GitHub provenance attestations. Adding the workflow does not itself publish
a release; a version tag or an explicitly confirmed manual run is required.
Until a reviewed fork release exists, build this repository from source.

Fork update checks and channels are labeled **Enhanced** and must not fall back
to official xAI release endpoints. An update is installable only when fork
release metadata and a matching artifact actually exist. Inherited
announcements and remote release notes are separate and explicitly labeled
**official xAI/upstream**. Persistent automatic update checks can be disabled in
`~/.grok/config.toml`:

```toml
[cli]
auto_update = false
```

## Official upstream installer

Use this only when you intentionally want the official xAI/SpaceXAI Grok Build
distribution rather than this fork:

```sh
curl -fsSL https://x.ai/cli/install.sh | bash   # macOS / Linux / Git Bash
irm https://x.ai/cli/install.ps1 | iex          # Windows PowerShell
```

Official upstream announcements, release notes, binaries, and changelog are
owned by xAI/SpaceXAI and are not Enhanced fork releases. See the
[official changelog](https://x.ai/build/changelog) and
[official Grok Build site](https://x.ai/cli).

## Enhanced additions

### ChatGPT Codex subscription provider (experimental)

An eligible ChatGPT account can supply its entitled Codex models without
reusing xAI authentication or static API keys:

```sh
grok login --provider openai-codex
grok models --provider openai-codex
grok -m openai-codex/<entitled-model-slug>
```

Use `--device-auth` for headless login and
`grok logout --provider openai-codex` to disconnect ChatGPT without changing
the xAI login. The authenticated account supplies the model, service-tier, and
reasoning-effort catalog; the fork does not hardcode entitlement claims.
Supported Codex sessions expose provider-scoped fast mode, usage state,
standalone web search/navigation, local protected `web_fetch`, image input with
catalog metadata that is currently descriptive rather than enforced, and
feature-gated image generation/editing. Availability remains dependent on
account metadata and server-side feature gates.

Read the implemented
[OpenAI Codex subscription provider reference](docs/providers/openai-codex-subscription-provider-reference.md)
and the
[authentication guide](crates/codegen/xai-grok-pager/docs/user-guide/02-authentication.md)
for experimental-contract and code-mode limitations.

### Themes, tools, and UX

Enhanced includes the packaged Warp theme corpus, provider-scoped Codex web and
image integrations, and focused terminal UX additions while preserving Grok
Build's existing tool names, permission model, sessions, and responsive braille
symbol. Third-party attribution is recorded in
[`THIRD-PARTY-NOTICES`](THIRD-PARTY-NOTICES) and crate-local notices.

### Candidate providers: research, not implementation

Kimi and GLM documents are integration research only. They do not register
provider IDs, add login commands, store credentials, or claim working managed
service support:

- [Kimi Code integration research — researched/planned, not implemented](docs/providers/kimi-code-integration-research.md)
- [Z.AI GLM Coding Plan integration research — researched/planned, not implemented](docs/providers/zai-glm-coding-plan-integration-research.md)
- [Provider documentation index](docs/providers/README.md)
- [Reviewed upstream revisions](UPSTREAM_VERSIONS.md)

## Upstream compatibility

Enhanced is an adapter-style fork, not a replacement application. It preserves:

- executable name `grok` and the `~/.grok` home/configuration layout;
- stored sessions and model IDs;
- existing `GROK_*` environment variables;
- ACP protocol and client identity;
- xAI authentication behavior, isolated from Codex authentication;
- the Grok agent loop, tools, permissions, headless mode, and TUI interaction;
- wide/narrow responsive layouts and the existing Grok braille symbol.

Fork metadata is additive and user-facing: Enhanced identity, upstream base
version, fork revision, and Codex compatibility version are shown only when
that metadata is compiled in. No wire protocol or provider identity is renamed.

## Documentation

The Enhanced-labeled, upstream-compatible user guide ships with the pager crate:
[`crates/codegen/xai-grok-pager/docs/user-guide/`](crates/codegen/xai-grok-pager/docs/user-guide/)
— getting started, keyboard shortcuts, slash commands, configuration, theming,
MCP servers, skills, plugins, hooks, headless mode, sandboxing, and more.

Fork provider documentation lives in [`docs/providers/`](docs/providers/).
Official upstream documentation is at
[docs.x.ai/build/overview](https://docs.x.ai/build/overview).

## Repository layout

| Path | Contents |
| --- | --- |
| `crates/codegen/xai-grok-pager-bin` | Composition root; builds the `xai-grok-pager` artifact |
| `crates/codegen/xai-grok-pager` | Full TUI, scrollback, prompt, modals, and welcome UI |
| `crates/codegen/xai-grok-pager-minimal` | Native-scrollback minimal mode |
| `crates/codegen/xai-grok-shell` | Agent runtime, leader, stdio, and headless entry points |
| `crates/codegen/xai-grok-tools` | Terminal, file, search, and other tool implementations |
| `crates/codegen/xai-grok-workspace` | Filesystem, VCS, execution, and checkpoints |
| `docs/providers` | Implemented provider reference plus clearly marked candidate research |
| `third_party` | Vendored upstream source/data; see notices |

> [!IMPORTANT]
> The root `Cargo.toml` is generated. Treat it as read-only and edit per-crate
> manifests instead.

## Development

```sh
cargo check -p xai-grok-pager-bin
cargo test -p xai-grok-pager
cargo test -p xai-grok-config
cargo clippy -p <crate>
cargo fmt --all
```

Previously grok-build-codex; renamed to reflect the broader Enhanced scope.

## Contributing

External contributions are not accepted. See [`CONTRIBUTING.md`](CONTRIBUTING.md).

## License

First-party code is licensed under the **Apache License, Version 2.0** — see
[`LICENSE`](LICENSE). Third-party and vendored code remains under its original
licenses. See [`THIRD-PARTY-NOTICES`](THIRD-PARTY-NOTICES),
[`crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md`](crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md),
[`crates/codegen/xai-grok-pager-render/THIRD_PARTY_NOTICES.md`](crates/codegen/xai-grok-pager-render/THIRD_PARTY_NOTICES.md),
and [`third_party/NOTICE`](third_party/NOTICE).
