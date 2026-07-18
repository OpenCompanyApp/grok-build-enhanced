# Getting Started

> [!IMPORTANT]
> This guide ships with **Grok Build Enhanced**, the unofficial OpenCompanyApp
> daily-driver fork. It preserves the `grok` executable and `~/.grok`
> compatibility surfaces but is not affiliated with or supported by
> xAI/SpaceXAI or OpenAI.

Grok Build Enhanced is a terminal-based AI coding assistant built on Grok Build.
It runs as a TUI (Terminal User Interface) that understands your codebase,
executes shell commands, edits files, searches the web, and manages tasks. You
can use it interactively, run it headlessly for scripting and CI/CD, or
integrate it into editors through the Agent Client Protocol (ACP).

---

## Install Grok Build Enhanced

The fork source and reviewed release assets live only at
[`OpenCompanyApp/grok-build-enhanced`](https://github.com/OpenCompanyApp/grok-build-enhanced).
On macOS, install the fork-owned Homebrew cask:

```bash
brew install --cask OpenCompanyApp/tap/grok-build-enhanced
```

Homebrew owns cask upgrades. Enhanced detects the Caskroom installation,
disables direct-download background updates, and delegates an explicit
`grok update` to `brew`. Use `brew upgrade --cask OpenCompanyApp/tap/grok-build-enhanced`
or `brew uninstall --cask OpenCompanyApp/tap/grok-build-enhanced` directly when
preferred.

The curl installer supports macOS and glibc-based Linux on Arm64 and x86-64:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://raw.githubusercontent.com/OpenCompanyApp/grok-build-enhanced/main/install.sh | sh
```

It requires no `sudo`. The installer validates fork-owned release provenance,
verifies SHA-256 checksums, smoke-tests the Enhanced identity and version, and
publishes the selected binary atomically under `~/.grok`. Existing
configuration, credentials, and sessions are preserved. Restart your shell if
prompted, then verify the active executable:

```bash
grok version                 # must begin with: Grok Build Enhanced
type -a grok                 # inspect PATH order if another grok is installed
```

To pin an exact release, pass a strict SemVer after `sh -s --`:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://raw.githubusercontent.com/OpenCompanyApp/grok-build-enhanced/main/install.sh \
  | sh -s -- --version 0.2.0
```

Other options include `--bin-dir PATH`, `--no-modify-path`, and `--force`.
Download and inspect the repository's [`install.sh`](../../../../../install.sh)
before running it if preferred; `sh install.sh --help` prints the complete
interface.

Update or check for updates through the fork-owned GitHub Release route:

```bash
grok update
grok upgrade --check
```

Rerunning the curl installer is also safe and idempotent. Enhanced updates
require an exact `grok-<version>-<os>-<arch>` asset from the fork repository and
never fall back to xAI installers, npm, or official artifact buckets.

To uninstall the managed executables and completions without deleting saved
configuration, credentials, or sessions:

```bash
rm -f "$HOME/.grok/bin/grok" "$HOME/.grok/bin/agent"
rm -f "$HOME/.grok/downloads"/grok-*-macos-* \
      "$HOME/.grok/downloads"/grok-*-linux-*
rm -f "$HOME/.grok/completions/bash/grok.bash" \
      "$HOME/.grok/completions/zsh/_grok" \
      "${XDG_CONFIG_HOME:-$HOME/.config}/fish/completions/grok.fish"
```

Remove the clearly marked `grok build enhanced installer` block from your shell
profile if it was added. Do not remove all of `~/.grok` unless you intentionally
want to delete user data. If no reviewed release asset matches the platform,
build from source instead:

```bash
git clone https://github.com/OpenCompanyApp/grok-build-enhanced.git
cd grok-build-enhanced
cargo build --locked --release -p xai-grok-pager-bin
mkdir -p "$HOME/.local/bin"
install -m 0755 target/release/xai-grok-pager "$HOME/.local/bin/grok"
```

### Official upstream distribution (not Enhanced)

Use the following commands **only** when you intentionally want to replace this
fork with the official xAI/SpaceXAI Grok Build distribution:

```bash
curl -fsSL https://x.ai/cli/install.sh | bash   # macOS/Linux/Git Bash
```

```powershell
irm https://x.ai/cli/install.ps1 | iex          # Windows PowerShell
```

These installers do not contain Enhanced Codex integration, bundled Warp-theme
work, fork branding, or fork-owned updates.

---

## First Launch

Start Grok by running:

```bash
grok
```

On first launch, Grok opens your browser to authenticate with grok.com. After you sign in, Grok stores your credentials in `~/.grok/auth.json`, where they persist across sessions. Grok refreshes your credentials automatically and prompts you to sign in again when they can no longer be renewed.

If you prefer API key authentication (e.g., for CI/CD or environments without a browser), set the `XAI_API_KEY` environment variable instead:

```bash
export XAI_API_KEY="xai-..."
grok
```

See [Authentication](02-authentication.md) for the full set of auth options including OIDC, external auth providers, and device code flow.

---

## Basic Interaction

Once authenticated, Grok presents a full-screen TUI with two main areas:

- **Scrollback** -- the conversation history showing your prompts, Grok's responses, tool calls, file edits, and more.
- **Prompt** -- the input area at the bottom where you type messages.

Type a message and press `Enter` to send it. Grok reads files, runs commands, and edits code as needed. Each tool run streams into the scrollback in real time.

Press `Tab` to move focus between the prompt and the scrollback. While a turn is running, `Ctrl+C` cancels it (or clears a non-empty draft first); `Esc` is a no-op mid-turn. Idle, press `Esc` twice within 800ms to clear a non-empty prompt, or (with an empty prompt and conversation messages) to open rewind — see [Keyboard Shortcuts](03-keyboard-shortcuts.md#escape). With the scrollback focused, use the arrow keys to select entries and to collapse or expand them. To navigate with `j`/`k` and fold with `h`/`l` instead, enable Vim mode.

### File References

Use `@` in your prompt to attach files:

```
@src/main.rs              # Attach a file
@src/main.rs:10-50        # Attach lines 10-50
@src/                     # Browse a directory
```

The `@` operator opens a fuzzy file picker. By default it respects `.gitignore` and hides dotfiles. Prefix with `!` to search hidden files:

```
@!.github                 # Search hidden files
@!.env                    # Attach a .env file
```

### Permissions

By default, Grok asks for permission before executing shell commands or editing files. You can approve individually or toggle always-approve mode:

- Press `Ctrl+O` to toggle always-approve mode
- Use the `--yolo` flag at launch: `grok --yolo`
- Type `/always-approve` in the prompt to toggle the mode

---

## Key Concepts

### Sessions

Every conversation is a **session**. Sessions are automatically saved to `~/.grok/sessions/` and can be resumed later. Each session tracks the full conversation history, tool calls, file edits, and task state.

- Start a new session: `Ctrl+N` or `/new`
- Resume a previous session: `/resume` in the TUI, or `--resume <ID>` from the CLI
- Continue the most recent session: `grok -c`

### Scrollback

The scrollback is the main display area. It shows:

- **User prompts** -- your messages, rendered as sticky headers
- **Agent messages** -- Grok's responses with full markdown rendering and syntax highlighting
- **Thinking blocks** -- Grok's reasoning process (collapsible)
- **Tool calls** -- file edits (with inline diffs), command executions, search results, and more
- **Task lists** -- TODO items tracking progress

Collapse or expand the selected entry with the `Left`/`Right` arrow keys (or `h`/`l` and `e` in Vim mode). In Vim mode, press `y` to copy its content and `Y` to copy its metadata (for example, the command that ran). Press `Enter` to open it in the fullscreen viewer (in any mode).

### Tools

Grok has built-in tools for:

| Tool | Description |
|------|-------------|
| `read_file` / `search_replace` | Read and edit files with line-precise changes |
| `grep` | Regex search across your codebase (powered by ripgrep) |
| `list_dir` | List directory contents |
| `run_terminal_command` | Execute shell commands |
| `web_search` / `web_fetch` | Search the web and fetch URLs |
| `todo_write` | Create and manage task lists |
| `spawn_subagent` | Spawn parallel subagent sessions |
| `memory_search` | Search cross-session memory |

Tools can be extended with [MCP servers](05-configuration.md#mcp-servers) for integrations like GitHub, databases, and more.

### Slash Commands

Type `/` in the prompt to access commands. These provide quick actions without writing a full prompt:

```
/model grok-build                 # Switch model
/compact                          # Compress conversation history
/always-approve                   # Toggle always-approve mode
/new                              # Start a new session
```

See [Slash Commands](04-slash-commands.md) for the complete reference.

---

## Common Launch Options

```bash
# Launch the interactive TUI and submit an initial prompt as the first turn
grok "fix the failing auth test and run it"

# Initial prompt in a new git worktree. Use --worktree=<name> (with `=`) so the
# prompt isn't swallowed as the worktree name — `grok -w "refactor module X"`
# would treat "refactor module X" as the worktree label, not the prompt.
grok --worktree=feat "refactor module X"

# Base the worktree on a specific branch (e.g. main) instead of the current HEAD:
grok -w --ref main "implement feature from main"


# Start in a specific project directory
grok --cwd ~/projects/my-app

# Add project-specific rules
grok --rules "Always use TypeScript. Prefer functional components."

# Auto-approve all tool executions
grok --yolo

# Use a specific model
grok -m grok-build

# Resume a previous session
grok --resume <session-id>

# Continue the most recent session
grok -c

# Experimental scrollback-native render mode. Sticky: plain `grok` reopens in
# the mode last chosen via --minimal/--fullscreen (or /minimal//fullscreen).
grok --minimal

# Back to the standard fullscreen TUI (and make it sticky again)
grok --fullscreen

# Headless mode (for scripts)
grok -p "Explain this codebase"
```

---

## Headless Mode

Run Grok non-interactively for scripting, CI/CD, and automation:

```bash
grok -p "Your prompt here"
```

Output formats:

| Format | Flag | Description |
|--------|------|-------------|
| `plain` | (default) | Human-readable text |
| `json` | `--output-format json` | Single JSON object with `text`, `stopReason`, `sessionId`, and `requestId` |
| `streaming-json` | `--output-format streaming-json` | NDJSON event stream for real-time processing |

Example CI/CD usage:

```bash
grok -p "Review changes for bugs" --output-format json --yolo | jq -r '.text'
```

---

## Project Rules (AGENTS.md)

Add per-project instructions by creating an `AGENTS.md` file in your repository. Grok reads these files and injects their contents as a project-instructions message at the start of the conversation:

```
~/.grok/AGENTS.md           # Global rules (apply to all projects)
<repo-root>/AGENTS.md       # Repository-level rules
<cwd>/AGENTS.md             # Directory-level rules (highest priority)
```

Deeper files take precedence. Grok also reads `CLAUDE.md` files for compatibility.

---

## Where to Go Next

| Document | What You Will Learn |
|----------|-------------------|
| [Authentication](02-authentication.md) | Browser login, API keys, OIDC, external auth, device code flow |
| [Keyboard Shortcuts](03-keyboard-shortcuts.md) | Complete reference for all key bindings |
| [Slash Commands](04-slash-commands.md) | All available `/` commands |
| [Configuration](05-configuration.md) | config.toml, pager.toml, environment variables |
