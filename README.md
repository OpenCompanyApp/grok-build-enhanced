> [!IMPORTANT]
> This is an unofficial OpenCompanyApp fork of Grok Build. It is not affiliated
> with, endorsed by, or supported by OpenAI, xAI, or SpaceXAI. Native ChatGPT
> Codex subscription support in this fork follows current public open-source
> Codex client behavior and uses an experimental backend contract that may
> change without notice. The x.ai installer below installs the official upstream
> build, not this fork; build this repository from source to use fork features.

<div align="center">

<h1>
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://media.x.ai/v1/website/spacexai-symbol-white-transparent-0c31957f.png">
    <source media="(prefers-color-scheme: light)" srcset="https://media.x.ai/v1/website/spacexai-symbol-black-transparent-6435cf42.png">
    <img alt="SpaceXAI logo" src="https://media.x.ai/v1/website/spacexai-symbol-black-transparent-6435cf42.png" width="96">
  </picture>
  <br>
  Grok Build (<code>grok</code>)
</h1>

**Grok Build** is SpaceXAI's terminal-based AI coding agent. It runs as a
full-screen TUI that understands your codebase, edits files, executes shell
commands, searches the web, and manages long-running tasks — interactively,
headlessly for scripting/CI, or embedded in editors via the Agent Client
Protocol (ACP).

[Installing the released binary](#installing-the-released-binary) ·
[Building from source](#building-from-source) ·
[Documentation](#documentation) ·
[Repository layout](#repository-layout) ·
[Development](#development) ·
[Contributing](#contributing) ·
[License](#license)

![Grok Build TUI](https://media.x.ai/v1/website/universe-tui-screenshot-6f7a0837.png)

**Learn more about Grok Build at [x.ai/cli](https://x.ai/cli)**

This repository contains the Rust source for the `grok` CLI/TUI and its agent
runtime. It is synced periodically from the SpaceXAI monorepo.

</div>

---

## Installing the released binary

Prebuilt binaries are published for macOS, Linux, and Windows:

```sh
curl -fsSL https://x.ai/cli/install.sh | bash   # macOS / Linux / Git Bash
irm https://x.ai/cli/install.ps1 | iex          # Windows PowerShell
grok --version
```

See the [changelog](https://x.ai/build/changelog) for the latest fixes,
features, and improvements in each release. Those installers, packages, and
release notes are official upstream channels and do not install Grok Build
Enhanced.

### Unofficial fork releases

Fork release assets, when published, are hosted only under
[`OpenCompanyApp/grok-build-enhanced`](https://github.com/OpenCompanyApp/grok-build-enhanced/releases).
The fork-owned pipeline builds native macOS and Linux binaries for Arm64 and
x86-64, names the installed executable `grok`, and attaches SHA-256 checksums
and GitHub provenance attestations. Adding the workflow does not itself publish
a release; a version tag or an explicitly confirmed manual run is required.
Until a reviewed fork release exists, build this repository from source.

## Building from source

Requirements:

- **Rust** — the toolchain is pinned by [`rust-toolchain.toml`](rust-toolchain.toml);
  `rustup` installs it automatically on first build.
- **protoc** — proto codegen resolves [`bin/protoc`](bin/protoc) (a
  [dotslash](https://dotslash-cli.com) launcher) or falls back to a `protoc` on
  `PATH` / `$PROTOC`.
- macOS and Linux are supported build hosts; Windows builds are best-effort
  and not currently tested from this tree.

```sh
cargo run -p xai-grok-pager-bin              # build + launch the TUI
cargo build -p xai-grok-pager-bin --release  # release binary: target/release/xai-grok-pager
cargo check -p xai-grok-pager-bin            # fast validation
```

The binary artifact is named `xai-grok-pager`; official installs ship it as
`grok`. On first launch it opens your browser to authenticate — see the
[authentication guide](crates/codegen/xai-grok-pager/docs/user-guide/02-authentication.md).

### ChatGPT Codex subscription (unofficial fork)

This fork keeps Grok Build's TUI, sessions, agent loop, permissions, and tools
while allowing an eligible ChatGPT account to supply entitled Codex models:

```sh
grok login --provider openai-codex
grok models --provider openai-codex
grok -m openai-codex/<entitled-model-slug>
```

Use `--device-auth` on the login command in a headless environment, and run
`grok logout --provider openai-codex` to disconnect ChatGPT without changing
the xAI login. In an active supported Codex session, use `/fast` to toggle or
`/fast on`, `/fast off`, and `/fast status` explicitly; the preference is
persisted for later sessions and never affects xAI or custom-provider requests. Fast is roughly 1.5x Standard
speed and currently consumes subscription credits at a higher rate (2.5x for
GPT-5.6/5.5 and 2x for GPT-5.4). Models, service tiers, and effort menus are
discovered from the authenticated account rather than hardcoded. Current
catalogs may advertise GPT-5.6 Sol and Terra through Ultra, and Luna through
Max. Catalog image-input capability
controls reading attachments; the separately authenticated `gpt-image-2`
generation/editing tools are exposed whenever their feature gates are enabled.
Codex sessions also expose provider-scoped standalone web search and navigation
(`search_query`, `image_query`, `open`, `click`, `find`, PDF screenshots, and
the current utility lookups) without `XAI_API_KEY`. Local `web_fetch` defaults
on for Codex while retaining its SSRF and fixed-domain protections; use search
`open`/`click` for result domains that are not locally allowlisted.
See the authentication guide for the experimental-contract and code-mode
compatibility limitations.

## Documentation

Full online documentation is available at
[docs.x.ai/build/overview](https://docs.x.ai/build/overview).

The user guide ships with the pager crate:
[`crates/codegen/xai-grok-pager/docs/user-guide/`](crates/codegen/xai-grok-pager/docs/user-guide/)
— getting started, keyboard shortcuts, slash commands, configuration, theming,
MCP servers, skills, plugins, hooks, headless mode, sandboxing, and more.

Fork-specific provider references and candidate-integration research live under
[`docs/providers/`](docs/providers/). Each document declares whether it covers
implemented experimental behavior or research for an unimplemented provider.
The candidate documents do not add runtime provider identities: those remain
xAI, experimental OpenAI Codex, and the existing generic custom-provider path.

## Repository layout

| Path | Contents |
|------|----------|
| `crates/codegen/xai-grok-pager-bin` | Composition-root package; builds the `xai-grok-pager` binary |
| `crates/codegen/xai-grok-pager` | The TUI: scrollback, prompt, modals, rendering |
| `crates/codegen/xai-grok-shell` | Agent runtime + leader/stdio/headless entry points |
| `crates/codegen/xai-grok-tools` | Tool implementations (terminal, file edit, search, ...) |
| `crates/codegen/xai-grok-workspace` | Host filesystem, VCS, execution, checkpoints |
| `crates/codegen/...` | The rest of the CLI crate closure (config, MCP, markdown, sandbox, ...) |
| `crates/common/`, `crates/build/`, `prod/mc/` | Small shared leaf crates pulled in by the closure |
| `third_party/` | Vendored upstream source (Mermaid diagram stack) — see below |

> [!IMPORTANT]
> The root `Cargo.toml` (workspace members, dependency versions, lints,
> profiles) is **generated** — treat it as read-only. Prefer editing per-crate
> `Cargo.toml` files.

## Development

```sh
cargo check -p <crate>        # always target specific crates; full-workspace builds are slow
cargo test -p xai-grok-config # per-crate tests
cargo clippy -p <crate>       # lint config: clippy.toml at the repo root
cargo fmt --all               # rustfmt.toml at the repo root
```

## Contributing

> [!NOTE]
> External contributions are not accepted. See [`CONTRIBUTING.md`](CONTRIBUTING.md).

## License

First-party code in this repository is licensed under the **Apache License,
Version 2.0** — see [`LICENSE`](LICENSE).

Third-party and vendored code remains under its original licenses. See:

- [`THIRD-PARTY-NOTICES`](THIRD-PARTY-NOTICES) — crates.io / git dependencies,
  bundled UI themes, and **in-tree source ports** (including openai/codex and
  sst/opencode tool implementations)
- [`crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md`](crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md)
  — crate-local notice for the codex and opencode ports (license texts +
  Apache §4(b) change notice)
- [`crates/codegen/xai-grok-pager-render/THIRD_PARTY_NOTICES.md`](crates/codegen/xai-grok-pager-render/THIRD_PARTY_NOTICES.md)
  — crate-local notice for the packaged Warp theme corpus
- [`third_party/NOTICE`](third_party/NOTICE) — vendored source/data index,
  including the crate-local Warp theme corpus
