# Repository guide

- Read this file, `README.md`, and `UPSTREAM_VERSIONS.md` before changing the
  fork. Treat direct ChatGPT Codex backend integration as experimental.
- The root `Cargo.toml` is generated. Change per-crate manifests; never
  hand-edit the generated workspace manifest.
- Keep provider identity explicit. Codex auth, retry, model discovery, usage,
  tools, and logout must never fall through to xAI auth or static API keys.
- Never log, upload, or place in hooks bearer/refresh/ID tokens, token prefixes,
  account IDs, FedRAMP state, turn state, or raw credential headers. Mark
  credential `HeaderValue`s sensitive and test redaction.
- `inspiration/` is ignored reference material. Record fetched/reviewed commits
  in `UPSTREAM_VERSIONS.md`; do not ship copied auth files or credentials.
- Preserve Grok Build's agent loop, sessions, tools, permissions, and TUI.
  Provider work should be a scoped adapter, not a replacement application.
- Preserve `LICENSE`, `THIRD-PARTY-NOTICES`, and crate-local notices. Attribute
  ported code and record prominent modifications where the source license asks.
- Keep downstream snapshotting, rebasing, and publishing as three explicit,
  separate operations:
  1. Snapshot the current downstream tip with an immutable commit/tree identity;
     snapshot creation must not rebase, move a branch, or push.
  2. Rebase only in a disposable branch or isolated worktree, preserve the
     frozen snapshot, and verify the candidate against it before publication.
  3. Publish only after review as a separate, explicitly authorized step. Never
     force-push from snapshot or rebase tooling; if authorization is given, use
     `--force-with-lease` against the reviewed destination and never rewrite an
     upstream, baseline, or frozen snapshot ref.
- Format with `cargo fmt`. Run focused tests for touched crates, then at least
  `cargo check -p xai-grok-pager-bin`; live-test Codex changes without printing
  credentials. Report environmental or credential-gated gaps honestly.
- Do not commit ignored inspiration clones, build artifacts, or secrets. Do not
  commit or push unless the user explicitly requests it.
