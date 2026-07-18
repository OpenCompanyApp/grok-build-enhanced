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
| Enhanced release artifacts | Fork-owned stable `v0.2.1` release for macOS/Linux, with SHA-256 checksums and GitHub artifact attestations |
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

## Install Grok Build Enhanced

The fork source and release assets are hosted only under
[`OpenCompanyApp/grok-build-enhanced`](https://github.com/OpenCompanyApp/grok-build-enhanced).
The official xAI installer does **not** install Enhanced features.

### Homebrew Cask (macOS)

Install the latest stable Enhanced release from the fork-owned tap:

```sh
brew install --cask OpenCompanyApp/tap/grok-build-enhanced
```

This fully qualified command taps only the requested cask. Homebrew owns the
Caskroom binary and its `grok` and `agent` links; Enhanced detects that ownership
and does not run its direct-download updater in the background. Upgrade or
remove the cask with:

```sh
brew upgrade --cask OpenCompanyApp/tap/grok-build-enhanced
brew uninstall --cask OpenCompanyApp/tap/grok-build-enhanced
```

An explicit `grok update` from a Homebrew installation delegates to the same
Homebrew cask. Exact version pinning remains available through the curl
installer.

### Curl installer

Install the latest stable Enhanced release on macOS or glibc-based Linux:

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://raw.githubusercontent.com/OpenCompanyApp/grok-build-enhanced/main/install.sh | sh
```

The installer supports Arm64 and x86-64, requires no `sudo`, and preserves
existing `~/.grok` configuration and sessions. It:

- resolves the latest stable release from fork-owned GitHub metadata;
- validates the Enhanced distribution, repository, release tag, and native
  asset name;
- verifies both release provenance and the binary against `SHA256SUMS`;
- smoke-tests the downloaded binary for the expected Enhanced identity/version;
- installs versioned binaries under `~/.grok/downloads` and atomically points
  `~/.grok/bin/grok` (plus the compatible `agent` alias) at the selected version;
- generates bash, zsh, and fish completions when supported; and
- adds an idempotent, clearly marked `~/.grok/bin` block to a recognized shell
  profile when that directory is not already on `PATH`.

Restart the shell if prompted, then verify which executable is active:

```sh
grok version                 # must begin with: Grok Build Enhanced
type -a grok                 # inspect PATH order if another grok is installed
```

Pin an exact stable or prerelease version with strict SemVer:

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://raw.githubusercontent.com/OpenCompanyApp/grok-build-enhanced/main/install.sh \
  | sh -s -- --version 0.2.1
```

Useful installer options are:

```text
--version VERSION   Install an exact release instead of the latest stable one
--bin-dir PATH      Also link grok and agent into an absolute directory
--no-modify-path    Do not edit a shell startup file
--force             Replace conflicting files (directories are never replaced)
```

Pass options after `sh -s --`, or use `GROK_INSTALL_VERSION`,
`GROK_INSTALL_BIN_DIR`, `GROK_INSTALL_NO_MODIFY_PATH=1`, and
`GROK_INSTALL_FORCE=1`. To review the installer before running it, download
[`install.sh`](install.sh), inspect it, and invoke `sh install.sh --help`.

### Update, verify, or remove the curl installation

Use the updater-compatible managed layout after installation:

```sh
grok update
```

You can also rerun the curl installer; installation and shell-profile changes
are idempotent. Fork update checks are labeled **Enhanced** and use only
fork-owned release metadata and matching artifacts. Persistent automatic update
checks can be disabled in `~/.grok/config.toml`:

```toml
[cli]
auto_update = false
```

GitHub CLI users can additionally verify the attestation for a retained binary:

```sh
gh attestation verify "$HOME/.grok/downloads/grok-0.2.1-macos-aarch64" \
  --repo OpenCompanyApp/grok-build-enhanced
```

To remove installer-managed binaries and completions while preserving
configuration, credentials, and sessions in `~/.grok`, run:

```sh
rm -f "$HOME/.grok/bin/grok" "$HOME/.grok/bin/agent"
rm -f "$HOME/.grok/downloads"/grok-*-macos-* \
      "$HOME/.grok/downloads"/grok-*-linux-*
rm -f "$HOME/.grok/completions/bash/grok.bash" \
      "$HOME/.grok/completions/zsh/_grok" \
      "${XDG_CONFIG_HOME:-$HOME/.config}/fish/completions/grok.fish"
```

Also remove the block between the
`grok build enhanced installer` markers in `.zshrc`, `.bashrc`,
`.bash_profile`, or `.config/fish/config.fish` if the installer added it. If you
used `--bin-dir`, remove its `grok` and `agent` links as well. Do not delete all
of `~/.grok` unless you intentionally want to remove saved configuration,
credentials, and sessions.

### Build from source

Source builds require Rust (pinned by [`rust-toolchain.toml`](rust-toolchain.toml))
and `protoc` (resolved through [`bin/protoc`](bin/protoc), `PATH`, or `$PROTOC`).
Clone the fork and run it directly:

```sh
git clone https://github.com/OpenCompanyApp/grok-build-enhanced.git
cd grok-build-enhanced
cargo run -p xai-grok-pager-bin
```

Or install a local release build as the compatible `grok` executable:

```sh
cargo build -p xai-grok-pager-bin --release
mkdir -p "$HOME/.local/bin"
install -m 0755 target/release/xai-grok-pager "$HOME/.local/bin/grok"
"$HOME/.local/bin/grok" version
```

The Cargo artifact remains `xai-grok-pager`; distributions expose it as `grok`.
If `CARGO_TARGET_DIR` is set, copy from that target directory instead. Windows
builds are best-effort and are not currently published by the fork release
pipeline; build the same package and place `xai-grok-pager.exe` on `PATH` as
`grok.exe`.

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
reusing xAI authentication or static API keys.

1. Start the browser OAuth login and complete the ChatGPT sign-in in the browser:

   ```sh
   grok login --provider openai-codex
   ```

   On a headless machine, use device authorization instead and follow the
   displayed URL and code:

   ```sh
   grok login --provider openai-codex --device-auth
   ```

2. List the Codex models currently available to the signed-in account:

   ```sh
   grok models --provider openai-codex
   ```

3. Start Grok with one of the returned model slugs:

   ```sh
   grok -m openai-codex/<entitled-model-slug>
   ```

Disconnect only the ChatGPT Codex subscription login, without changing the xAI
login, with:

```sh
grok logout --provider openai-codex
```

The authenticated account supplies the model, service-tier, and reasoning-effort
catalog; the fork does not hardcode entitlement claims.

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
