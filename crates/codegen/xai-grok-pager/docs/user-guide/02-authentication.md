# Authentication

Grok supports several authentication methods, including interactive browser login, ChatGPT Codex subscription login, enterprise single sign-on (SSO), and headless CI/CD runners. xAI and ChatGPT Codex credentials are stored in separate scopes and can remain signed in at the same time.

---

## Browser Login (Default)

On first launch, Grok opens your browser to authenticate with grok.com:

```bash
grok
```

Grok stores credentials in `~/.grok/auth.json` and reuses them across sessions. Grok refreshes access tokens automatically in the background. When a token can't be refreshed, Grok prompts you to sign in again. Credentials without a server-provided expiry fall back to a 30-day lifetime.

### Re-authenticate

To switch accounts or resolve an authentication problem, run:

```bash
grok login
```

Running `grok login` starts the sign-in flow again, replacing your cached session. By default, it opens your browser and signs in through SpaceXAI OAuth at `auth.x.ai`. Pass a flag to select a different flow:

| Flag | Description |
|------|-------------|
| `--oauth` | Sign in through SpaceXAI OAuth at `auth.x.ai`. This is the default, so the flag is optional. |
| `--device-auth` (alias `--device-code`) | Sign in with the device-code flow for headless or remote environments. |

To sign out of xAI, run `grok logout`. Add `--provider openai-codex` to clear
only the independent ChatGPT Codex credential, as described below.

---

## ChatGPT Codex Subscription

This unofficial fork can use models and image capabilities included with an
eligible ChatGPT Codex subscription. It keeps the Grok Build agent loop, tools,
sessions, and TUI; ChatGPT supplies the selected Codex model rather than an
OpenAI Platform API key.

Sign in through the browser:

```bash
grok login --provider openai-codex
```

ACP/editor clients can choose the advertised **ChatGPT Codex subscription**
authentication method; when no saved credential exists, Grok starts the same
browser-PKCE flow. This remains independent from the selected xAI auth method.

For SSH, containers, and other headless environments, use the device flow:

```bash
grok login --provider openai-codex --device-auth
```

After authentication, list the models currently available to that ChatGPT
account and select a namespaced model:

```bash
grok models --provider openai-codex
grok -m openai-codex/<model-slug>
```

The model list is fetched from the authenticated Codex service; this fork does
not maintain a hardcoded entitlement list. To disconnect ChatGPT without
affecting the xAI login:

```bash
grok logout --provider openai-codex
```

Current Codex catalogs can advertise the GPT-5.6 Sol, Terra, and Luna variants.
They are not hardcoded into this fork: an entitled variant appears under its
`openai-codex/` model ID only when the authenticated service returns it. The
model picker and `/effort` menu likewise use that model's advertised reasoning
levels. In the catalog audited for this implementation, Sol and Terra advertise
`low`, `medium`, `high`, `xhigh`, `max`, and `ultra`; Luna advertises `low`,
`medium`, `high`, `xhigh`, and `max`. The authenticated service remains
authoritative if those menus change. In the current public Codex client,
`ultra` is a client orchestration tier rather than a backend effort value; this
fork keeps it in the Sol/Terra UI menu but sends the service-supported `max`
wire effort. On Codex v2 catalogs, Ultra also activates a turn-local proactive
delegation policy through Grok Build's existing subagent tool and coordinator;
lower levels use explicit-request-only delegation. This does not launch Codex
app-server or change Grok Build's permission checks, nesting limits, sessions,
or tool implementations.

To preserve Grok Build's existing agent loop, sessions, permission checks, and
tool implementations, this fork currently exposes Grok tools as direct function
schemas inside the Codex Responses Lite `additional_tools` item. The public
OpenAI Codex client currently marks GPT-5.6 Sol, Terra, and Luna as
`code_mode_only` and normally wraps its nested tools behind a V8-backed `exec`
custom tool plus `wait`. That V8 code-mode runtime is intentionally not ported
into this fork. If the service returns a custom/freeform tool call instead of a
direct function call, Grok fails that turn safely rather than treating it as an
xAI search or executing generated JavaScript. Direct-function behavior still
requires live validation with an entitled ChatGPT account and remains part of
the experimental integration contract.

Codex access, refresh, and identity tokens are stored under the dedicated
`openai::codex` scope in `~/.grok/auth.json`. They are refreshed per request and
are never treated as permanent model API keys. This integration does not read
or copy credentials from `~/.codex/auth.json`.

Codex sessions can read attached/local images through the existing multimodal
file path when the selected catalog model advertises image-input support.
`image_gen` and `image_edit` are separate `gpt-image-2` capabilities: they are
exposed whenever their feature gates are enabled and dynamic Codex
authentication is available, even if a future coding model is text-only.
Requested aspect ratios are mapped to exact, model-valid canvases for the
current Codex image endpoint. ChatGPT Codex does not currently expose a
subscription video-generation endpoint, so the xAI video tools are unavailable
while a Codex model is selected and return when the session switches back to
xAI.

Codex subscription sessions also expose Grok Build's `web_search` function
without requiring `XAI_API_KEY`. The function follows the current public Codex
standalone-search contract and sends provider-authenticated requests to
`{codex-base}/alpha/search`. It supports ordinary and image queries, opening a
search reference or URL, following numbered links, in-page find, PDF
screenshots, finance, weather, sports, time, response sizing, and repeated
search/navigation calls. Result URLs are retained as citations for the TUI and
the model can pass returned references to later `open`, `click`, `find`, or
`screenshot` operations. Each request reloads the scoped ChatGPT credential;
an authentication rejection gets at most one provider-owned refresh/retry and
is never routed through xAI authentication.

To match the public Codex client without copying provider-private response
state, standalone search receives only a bounded text projection of the last
two visible user turns and the assistant text between them. System context,
synthetic reminders, images, reasoning records, tool traffic, credentials,
account identifiers, and `x-codex-turn-state` are not forwarded. Grok adds a
bounded `x-codex-turn-metadata` correlation header containing only its stable
session/turn identifiers, selected model, and reasoning effort.

The local `web_fetch` tool is enabled by default for Codex only when no local,
managed, remote, or environment setting made an explicit choice. Its existing
SSRF protection and fixed domain allowlist remain in force, so an arbitrary
search-result domain may be rejected by `web_fetch`; use the subscription
search tool's `open`/`click` commands for those results. `--disable-web-search`
disables both web tools. `GROK_WEB_FETCH=0` disables only local URL fetching;
`GROK_WEB_FETCH=1` still opts it in explicitly. xAI sessions retain their
existing web-search and opt-in web-fetch behavior.

Search citations intentionally retain their complete source URLs so the model
can navigate and cite them. User-configured tool lifecycle hooks receive tool
arguments and results, including those URLs; treat such hooks as an explicit
external-data boundary and review their destinations accordingly.

> [!WARNING]
> Direct ChatGPT Codex backend integration is experimental. It follows the
> behavior of the current public `openai/codex` client and is not a stable,
> general-purpose OpenAI API contract. Availability depends on the authenticated
> account and can change independently of this fork.

---

## API Key

For CI/CD, automation, or environments without browser access, use an API key from [console.x.ai](https://console.x.ai):

```bash
export XAI_API_KEY="xai-..."
grok
```

Grok uses the API key as a fallback when no session token is active. If you have already signed in interactively, the stored session token takes precedence. To fall back to the API key, run `grok logout` or delete `~/.grok/auth.json`.

---

## OIDC (Customer SSO)

Authenticate developers through your own Identity Provider (IdP) -- such as Okta, Azure AD, or Auth0 -- instead of grok.com.

### 1. Register a public client in your IdP

- Grant type: Authorization Code with PKCE (Proof Key for Code Exchange)
- Redirect URI: `http://127.0.0.1/callback` -- a loopback address. Grok binds a random port at sign-in time, and most IdPs treat the loopback redirect as port-agnostic per [RFC 8252](https://tools.ietf.org/html/rfc8252).
- No client secret. PKCE replaces it.

### 2. Configure the CLI

Via config file:

```toml
# ~/.grok/config.toml
[grok_com_config.oidc]
issuer = "https://acme.okta.com"
client_id = "0oa1b2c3d4e5f6g7h8i9"
```

Or via environment variables:

```bash
export GROK_OIDC_ISSUER="https://acme.okta.com"
export GROK_OIDC_CLIENT_ID="0oa1b2c3d4e5f6g7h8i9"
```

You can also override the API endpoint to point at your own proxy:

```bash
export GROK_CLI_CHAT_PROXY_BASE_URL="https://grok-proxy.acme.com/v1"
```

### 3. Run `grok`

The CLI discovers endpoints via `{issuer}/.well-known/openid-configuration`, opens the IdP login page, and stores tokens in `~/.grok/auth.json`. Tokens auto-refresh silently via the stored `refresh_token`.

### Optional fields

| Field | Default | Notes |
|-------|---------|-------|
| `scopes` | `["openid", "profile", "email", "offline_access", "api:access"]` | `offline_access` enables silent token refresh |
| `audience` | None | Required by some IdPs (e.g., Auth0) |

---

## External Auth Provider

When browser-based login isn't possible -- for example, on sandboxed VMs, CI runners, or air-gapped networks -- delegate authentication to an external binary or script.

### How It Works

```
+--------------+     sh -c     +------------------------+
|     Grok     |-------------->|  your auth binary      |
|              |               |                        |
|  reads       |<-- stdout ----|  prints token          |
|  auth.json   |               |                        |
|              |   (stderr)    |  prints status/URLs    |--> surfaced to user
+--------------+               +------------------------+
```

1. Grok runs your command via `sh -c "<command>"`
2. Your binary runs whatever auth flow it needs (SSO, device code, certificate exchange)
3. **stderr** carries human-readable output, such as login URLs and status messages. Grok reads stderr and surfaces it to the user; in the TUI, it turns the first `https://` URL into a clickable sign-in link.
4. **stdout** is captured by Grok and saved as the access token
5. Exit 0 = success; exit non-zero = Grok falls back to interactive login

### The stdout / stderr Contract

| Stream | What to print | Who sees it |
|--------|---------------|-------------|
| **stdout** | The token -- nothing else | Grok (parsed and stored in auth.json) |
| **stderr** | Login URLs, status messages, errors | The user (Grok reads stderr and shows the sign-in URL as a clickable link in the TUI) |

**Do not print anything to stdout except the token.** No progress messages, no debug output. Grok reads stdout, trims surrounding whitespace, and parses the result as a token.

### stdout Token Format

**Bare string** -- just the raw token:

```
eyJhbGciOiJSUzI1NiIs...
```

**JSON** -- with optional refresh token and expiry:

```json
{"access_token": "eyJhbGciOi...", "refresh_token": "ref-tok", "expires_in": 3600}
```

Use JSON if your tokens expire and you want Grok to automatically re-run the binary before expiry.

### Configuration

Via config file:

```toml
# ~/.grok/config.toml
[auth]
auth_provider_command = "/usr/local/bin/my-auth-provider"
auth_provider_label = "Acme Corp"   # optional -- customizes the TUI login button
auth_token_ttl = 3600               # optional -- token lifetime in seconds
```

Or via environment variables:

```bash
export GROK_AUTH_PROVIDER_COMMAND="/usr/local/bin/my-auth-provider"
export GROK_AUTH_PROVIDER_LABEL="Acme Corp"
export GROK_AUTH_TOKEN_TTL=3600
```

### Token Refresh

When Grok needs to refresh an expired token, it re-runs your binary with `GROK_AUTH_EXPIRED=1` set in the environment. Your binary can use this to take a faster silent-refresh path:

```bash
#!/bin/sh
if [ "$GROK_AUTH_EXPIRED" = "1" ]; then
    echo "Refreshing token..." >&2
    TOKEN=$(my-company-auth --refresh --silent)
else
    echo "Authenticating via Acme Corp SSO..." >&2
    TOKEN=$(my-company-auth --login --interactive)
fi

if [ -z "$TOKEN" ]; then
    echo "Authentication failed" >&2
    exit 1
fi

echo "{\"access_token\": \"$TOKEN\", \"expires_in\": 3600}"
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `GROK_AUTH_PROVIDER_COMMAND` | Path to your auth binary |
| `GROK_AUTH_PROVIDER_LABEL` | Display name on the TUI login screen (e.g., "Acme Corp") |
| `GROK_AUTH_TOKEN_TTL` | Token lifetime in seconds (for bare-string tokens without `expires_in`) |
| `GROK_AUTH_EXPIRED` | Set to `1` by Grok when re-running the binary for token refresh |
| `GROK_AUTH_EARLY_INVALIDATION_SECS` | Seconds before expiry to proactively refresh (default: 300) |

---

## Device Code Flow

For headless environments (SSH sessions, Docker containers, remote VMs) where no browser is available locally:

```bash
grok login --device-auth    # or: grok login --device-code
```

This prints a URL and code to the terminal. Open the URL on any device, enter the code, and complete authentication. Grok polls until the login is confirmed.

You can also implement the device-code flow through an [External Auth Provider](#external-auth-provider) for full control.

For ChatGPT Codex, qualify the provider so the device flow does not replace the
xAI session:

```bash
grok login --provider openai-codex --device-auth
```

---

## Automatic Credential Refresh

Grok automatically refreshes expired credentials:

- **Before expiry:** If your auth provider returned `expires_in` (JSON output) or you set `auth_token_ttl`, Grok re-runs the auth binary ~5 minutes before expiry.
- **On auth error:** If the server returns 401 Unauthorized, Grok refreshes the credentials and retries the request.
- **OIDC:** If a `refresh_token` is available, Grok silently refreshes via your IdP without re-opening the browser.
- **ChatGPT Codex:** Grok reloads the provider-scoped credential, refreshes a rotating token family once under a cross-process lock, and retries only before a response stream has started.

Tune the refresh buffer:

```bash
# Refresh 5 minutes before expiry (default)
export GROK_AUTH_EARLY_INVALIDATION_SECS=300

# Disable the proactive buffer: refresh at expiry or on a 401 (set to 0)
export GROK_AUTH_EARLY_INVALIDATION_SECS=0
```

---

## Hot Reload

Grok picks up changes to `~/.grok/auth.json` automatically. If you update credentials externally (for example, with a script that writes new tokens), Grok uses the new credentials on the next API call without a restart.

---

## Auth Precedence

Grok resolves credentials for each request in this order, highest to lowest:

1. **Per-model `api_key` or `env_key`** -- set under `[model.<name>]` in `config.toml`. Wins whenever present.
2. **Active session token** -- obtained through browser, OIDC/OAuth2, or external-provider login and stored in `~/.grok/auth.json`.
3. **`XAI_API_KEY`** -- fallback when no session token is active.

This precedence applies to xAI and ordinary custom models. A model whose ID is
under `openai-codex/` always uses the separately scoped ChatGPT Codex OAuth
credential. It never inherits a per-model key, the xAI session, or
`XAI_API_KEY`.

When more than one login flow is configured, Grok populates the session token from the first available source, highest to lowest:

1. **External auth provider** (`auth_provider_command`)
2. **Enterprise OIDC** -- when OIDC is configured, through `[grok_com_config.oidc]` in `config.toml` or the `GROK_OIDC_ISSUER` and `GROK_OIDC_CLIENT_ID` environment variables
3. **SpaceXAI OAuth2 browser login** -- the default

During a session, the active method handles all mid-session refreshes.

---

## Troubleshooting

### Debug logging

Set `RUST_LOG` to control the verbosity of the file log and headless stderr output. (The TUI's on-screen tracing pane uses a fixed filter and ignores `RUST_LOG`.) In the TUI, file logging defaults to `DEBUG`; in headless mode (`-p`), `RUST_LOG` defaults to `off` so only the answer is printed — set `RUST_LOG=error` (or broader) to see logs on stderr.

In the TUI, set `GROK_LOG_FILE` to an absolute path to write logs to that file:

```bash
GROK_LOG_FILE=/tmp/grok.log RUST_LOG=debug grok
tail -f /tmp/grok.log
```

`GROK_LOG_FILE` is treated as a literal file path. A relative value such as `1` writes a file named `1` in the current directory.

In headless mode, logs go to stderr. Redirect them to a file:

```bash
RUST_LOG=debug grok -p "hello" 2> /tmp/grok.log
```

### Common log messages

| Log message | What it means |
|-------------|---------------|
| `auth: running external auth provider` | Grok is running your binary |
| `auth: external auth provider returned fresh token` | Grok parsed and stored the token |
| `auth: external auth provider failed` | Binary exited non-zero or stdout was empty |
| `auth: external auth provider timed out (likely needs interactive auth), killing` | Binary did not exit before the timeout and was killed |
| `auth: failed to start external auth provider` | Command could not be spawned (binary not found) |

### Common fixes

- **"Authentication failed"** -- Run `grok logout` to clear cached credentials, then `grok login` to sign in again.
- **Token expires too quickly** -- Set `auth_token_ttl` or return `expires_in` in your auth provider's JSON output.
- **OIDC redirect fails** -- Ensure your IdP allows loopback redirect URIs (`http://127.0.0.1/callback`).
- **External auth provider not found** -- Check that the `auth_provider_command` path is correct and the binary is executable.
- **Codex authentication required** -- Run `grok login --provider openai-codex`; this does not alter the xAI login.
- **Codex account changed** -- Re-select the Codex model after signing in. Existing sessions are not silently rebound to a different ChatGPT account.
