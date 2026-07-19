# Grok Build Enhanced repository guide

## Fork vision

**Grok Build Enhanced** is the unofficial daily-driver fork of Grok Build. Its
purpose is deliberately narrow:

- preserve Grok Build's agent loop, sessions, tools, permissions, TUI, headless
  mode, and Agent Client Protocol behavior;
- add explicit, isolated ChatGPT Codex subscription and Kimi Code providers;
- package the audited Warp theme corpus and retain terminal-native theme
  behavior;
- present clear Enhanced branding without renaming compatibility surfaces;
- own the fork's GitHub release and update routes; and
- keep downstream work as small thematic commits with machine-checked ownership
  so future disconnected upstream refreshes remain reviewable.

The executable remains `grok`, the Cargo artifact remains `xai-grok-pager`, and
configuration/session storage remains under `~/.grok`. Preserve existing model
IDs, `GROK_*` environment variables, ACP identity, and the responsive Grok
braille symbol.

## Scope and non-goals

- Treat direct ChatGPT Codex and Kimi Code backend integrations as
  experimental. They are scoped provider adapters, not replacement applications
  or embedded upstream app servers.
- Keep xAI, OpenAI Codex, Kimi Code, and generic custom-provider identities
  explicit. Provider auth, retry, model discovery, usage, tools, and logout must
  never fall through to another provider's credentials or static API keys.
- Kimi Code is an experimental API-key provider with authenticated catalog
  discovery and provider-hosted web capabilities. Z.AI GLM Coding Plan remains
  research only and does not establish a runtime provider, login command,
  credentials, or product claims.
- Do not introduce fork policy that suppresses or reshapes ordinary upstream
  telemetry and permission details. Users decide whether to enable supported
  telemetry. Security boundaries that prevent credentials from being logged or
  sent to the wrong provider remain mandatory.
- Do not replace the upstream TUI, permission model, session format, tool names,
  or update semantics with a parallel application architecture.

## Credential and data safety

Never log, upload, place in hooks, or include in diagnostics API keys; bearer,
refresh, or ID tokens; token prefixes; account IDs; FedRAMP state; Codex turn
state; or raw credential headers. Mark credential `HeaderValue`s sensitive and test
redaction. Do not import or commit `~/.codex/auth.json`, `~/.grok/auth.json`,
private approval messages, authenticated response captures, or any other
secret-bearing material.

These are provider-isolation and credential-wire requirements, not a broader
telemetry/privacy product policy.

## Upstream and inspiration sources

`UPSTREAM_VERSIONS.md` is the human-readable provenance ledger.
`fork/manifest.json` mirrors its reviewed/latest source identities for
machine-checking. Update **Latest fetched** when a source is fetched; update
**Reviewed** only after the relevant diff, compatibility impact, tests, and
notices have been reviewed.

Primary sources and inspiration repositories are:

| Purpose | Repository / tracked ref |
| --- | --- |
| Grok Build upstream | `https://github.com/xai-org/grok-build.git` `main` |
| OpenAI Codex compatibility reference | `https://github.com/openai/codex.git` `main` |
| OpenCode interoperability reference | `https://github.com/anomalyco/opencode.git` `dev` |
| OpenCode Codex auth reference | `https://github.com/numman-ali/opencode-openai-codex-auth.git` `main` |
| Warp themes | `https://github.com/warpdotdev/themes.git` `main` |
| Kimi Code provider reference | `https://github.com/MoonshotAI/kimi-code.git` `main` |
| Kimi CLI legacy reference | `https://github.com/MoonshotAI/kimi-cli.git` `main` |
| Z.AI SDK research | `https://github.com/zai-org/z-ai-sdk-python.git` `main` |
| Z.AI coding-plugin research | `https://github.com/zai-org/zai-coding-plugins.git` `main` |
| GLM-5 model research | `https://github.com/zai-org/GLM-5.git` `main` |
| CodexBar Z.AI usage research | `https://github.com/steipete/CodexBar.git` `main` |
| Z.AI usage-browser research | `https://github.com/nniicckk6/zai-extention.git` `main` |

Local checkouts belong under ignored `inspiration/`. Never commit inspiration
clones or copy auth files, credentials, private data, or unreviewed source into
the fork. Preserve source licenses and update `THIRD-PARTY-NOTICES`, crate-local
notices, and modification notices when code is ported rather than independently
implemented.

## Immutable baselines and ownership manifest

Treat `fork/manifest.json` as the authoritative machine-readable ownership and
history contract. At schema version 2 it records:

- Grok Build fork baseline commit
  `c1b5909ec707c069f1d21a93917af044e71da0d7`, tree
  `54a0606cc804815975eda042f60cb3320cfb5347`;
- frozen original downstream tip
  `da3076874292c538bfc4efeede0cf517c2e975f0`, tree
  `bbeaf9575ffa5f7bcd75b8352a178455f04c98c7`;
- thematic tree-equivalent checkpoint
  `7ed3320c797852ed5894f1fc2fca7cee6827768f`, with the same frozen tree; and
- latest fetched disconnected Grok Build snapshot
  `8adf9013a0929e5c7f1d4e849492d2387837a28d`, which is a review target rather
  than an automatically accepted base.

Every downstream path between the baseline and a publication candidate must be
owned by a focused manifest feature unit. Keep integration paths narrow and
legal paths explicit. Validate with:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -I -B \
  fork/scripts/check_manifest.py --strict-coverage
```

The root `Cargo.toml` is generated. Never hand-edit it; change per-crate
manifests and regenerate only through the documented upstream process.

## Snapshot, refresh, and publication discipline

Keep these as separate operations:

1. **Snapshot:** create an immutable commit/tree identity without rebasing,
   moving a branch, or pushing.
2. **Refresh/rebase:** work only in a disposable branch or isolated worktree;
   preserve the frozen snapshot and compare raw `(mode,type,oid)` tree entries
   when upstream history has no usable merge base.
3. **Publish:** merge and push only the reviewed candidate. Never force-push
   from snapshot or refresh tooling. If history rewriting is ever explicitly
   approved, use `--force-with-lease` only against the reviewed destination and
   never rewrite upstream, baseline, or frozen snapshot refs.

The fork release repository is `OpenCompanyApp/grok-build-enhanced`. Production
update checks and downloads must use fork-owned release metadata and matching
`grok-<version>-<os>-<arch>` assets, never official xAI installers, npm packages,
or artifact buckets as a fallback.

## Validation and repository hygiene

- Format Rust with `cargo fmt`.
- Run focused tests for touched crates, then at least
  `CARGO_INCREMENTAL=0 cargo check -p xai-grok-pager-bin`.
- Live-test Codex changes only with entitled credentials and without printing
  credentials or authenticated payloads.
- Run release contracts, Warp vendor-lock checks, generated-manifest checks,
  and strict fork ownership checks for their respective changes.
- Report environmental and credential-gated gaps honestly.
- Do not commit ignored inspiration clones, Cargo targets, generated build
  artifacts, credentials, or secret files.
- Keep `LICENSE`, `THIRD-PARTY-NOTICES`, and crate-local notices intact.
- Do not push, tag, publish a release, or mutate pull-request state unless the
  user explicitly authorizes that operation.
