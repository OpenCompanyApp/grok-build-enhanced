---
name: refresh-upstreams
description: Safely refresh and audit Grok Build Enhanced against tracked Grok Build, OpenAI Codex, Kimi Code, OpenCode, Z.AI, GLM, Warp, and related source repositories without blindly rebasing disconnected snapshots. Use when the user says upstream changed, asks to check provider compatibility or refresh source revisions, or runs /refresh-upstreams.
---

# Refresh upstreams

Refresh source provenance, audit compatibility, and port confirmed gaps while preserving Grok Build Enhanced's downstream ownership and provider isolation.

## Operating rules

- Read repository `AGENTS.md` and any deeper instruction files first.
- Treat `fork/manifest.json` as the ownership and source-history authority and `UPSTREAM_VERSIONS.md` as its human-readable provenance ledger. Manifest coverage does not prove behavior parity.
- Treat user-visible Grok Build behavior on the fork's preserved surfaces as adopt-by-default. Implementation difficulty is not a valid terminal exclusion.
- Classify architecture separately from observable behavior. Replacing an unsafe or incompatible mechanism does not remove an applicable behavior obligation.
- Never expose, copy, log, or commit credentials, authenticated payloads, account IDs, token state, or auth files from `~/.grok`, `~/.codex`, or inspiration repositories.
- Keep **snapshot**, **refresh/audit**, and **publication** separate.
- Never hand-edit root `Cargo.toml`.
- Do not force-push, tag, publish, or mutate a PR unless the user explicitly authorizes publication for this run.
- Pin exact revisions at the start. If a remote advances mid-audit, leave the later head for the next run.

## 1. Establish a safe boundary

1. Inspect `git status`, active branch, worktrees, remotes, and pending diffs.
2. If the worktree is dirty, determine whether it is coherent in-progress work. Never discard or overwrite it.
3. Finish and validate the current thematic change or preserve it on a dedicated commit/worktree before refreshing sources.
4. Record the pre-refresh commit and tree IDs. Create refresh work only in a disposable branch or isolated worktree based on that exact commit.
5. Do not content-merge or rebase upstream into `main`. After the audit and all applicable Grok behavior have landed, the narrowly authenticated zero-tree-delta acknowledgement in section 7 is the only permitted upstream merge.

## 2. Fetch and pin sources

1. Read tracked remotes, refs, reviewed revisions, and latest-fetched revisions from `fork/manifest.json` and `UPSTREAM_VERSIONS.md`.
2. Fetch `upstream/main` for Grok Build and the configured tracked refs in ignored `inspiration/` checkouts. Fetch only; do not merge, reset, or copy source yet.
3. Record each fetched commit and tree ID and whether the reviewed revision is an ancestor.
4. Update only **Latest fetched** and the check timestamp immediately after a successful fetch. Do not advance **Reviewed** during this phase.
5. Keep ignored inspiration checkouts and generated build output out of commits.

## 3. Audit Grok Build snapshots

Grok Build may be republished from a monorepo without a useful merge base. Compare raw `(mode,type,oid)` tree entries and path contents instead of relying on ordinary history or rename detection.

Audit both:

- the last reviewed/fork baseline to the previously fetched-but-unreviewed revision;
- that revision to the newly pinned head.

Map every changed path to a `fork/manifest.json` feature unit, then separately inventory every changed observable behavior. Path ownership is necessary but is not evidence that the local product is behaviorally equivalent.

Record the behavior inventory in a checked-in parity ledger for the pinned source revision. Use exactly these outcomes:

- `adopt` — the default for Grok behavior on a preserved surface;
- `already equivalent` — cite the local implementation and parity tests;
- `not applicable` — cite a standing scope, security, or legal rule; or
- `temporarily deferred` — create a durable open obligation rather than a terminal audit bucket.

A temporary deferral must include a stable ID, pinned source revision and paths, owner, blocker, user impact, target milestone or next-refresh deadline, acceptance criteria, and intended tests. Carry every open ID into later refresh ledgers until implementation and evidence close it. Fail the audit if an earlier deferral disappears without closure evidence.

Do not classify a behavior as inapplicable merely because its upstream architecture cannot be imported. First decide whether the observable behavior belongs on a preserved Grok surface; if it does, adopt it through an architecture and security model compatible with the fork.

Review in dependency order:

1. session persistence and ACP contracts;
2. agent loop, permissions, and auto mode;
3. hooks and lifecycle continuation;
4. scheduler semantics;
5. shell/TUI, clipboard, themes, and syntax rendering;
6. generated manifests, locks, tests, docs, and legal files.

At every seam, preserve xAI, Codex, Kimi, Z.AI, and generic custom-provider identity. Upstream xAI behavior must not acquire another provider's credentials, endpoints, model catalog, usage, retry, hosted tools, or logout state.

## 4. Audit provider contracts

Use independent read-only subagents for Grok path ownership and each provider range when the user permits delegation. Give every agent exact pinned revisions and require evidence with local paths/commits. Reconcile all findings in the main session before editing.

### OpenAI Codex

Compare the last reviewed Codex revision with the pinned head using this matrix:

- login, refresh, account state, storage, and logout;
- model catalog/cache, context limits, compaction threshold, reasoning effort, service tiers/Fast, Responses Lite, visibility, and `comp_hash`;
- Responses URL, sensitive headers, instructions, reasoning, tools, service tier, storage/include fields, continuation state, SSE events, and errors;
- active-model compaction, previous-model downshift fallback, remote compaction, and cache isolation;
- hosted web/image tools;
- usage/limits and auxiliary accounting;
- retries, idempotency, timeout, cancellation, and auth recovery.

Record `upstream behavior -> local behavior -> action/test`. Do not import Codex app-server or TUI architecture. Still audit the behavior delivered by those changes: when it maps to an existing Grok surface or provider-wire contract, record an adoption or adapted-implementation obligation instead of excluding it by architecture label.

### Kimi Code

Check authenticated catalog/model metadata, API/auth headers, request/stream schema, thinking effort, hosted web search/fetch, usage, retry/errors, and logout isolation. Do not import Kimi's app server, agent engine, replacement TUI, or telemetry policy. Audit their observable behavior separately and adopt only behavior that belongs to the existing Grok application or Kimi provider-adapter surface.

### Z.AI and GLM

If tracked heads are unchanged, attest that and stop. If changed, inspect only established research/provider-contract surfaces. Research revisions do not authorize a new runtime provider, login flow, credentials, or product claims.

### Other references

Audit OpenCode/auth references when Codex auth or interoperability changed, and Warp themes only when the theme corpus or licenses changed.

## 5. Port confirmed gaps

1. Check in the exhaustive behavior-parity ledger before or with implementation; no changed behavior may remain unclassified.
2. Port confirmed applicable behavior as small thematic commits; never make one broad `sync upstream` commit.
3. Keep source licenses, crate-local notices, and modification notices current when code is ported.
4. Add every newly changed downstream path to a focused manifest feature unit before committing.
5. Close an `adopt` or `temporarily deferred` ledger item only with implementation and test evidence that satisfies its acceptance criteria.
6. Advance **Reviewed** only after the relevant diff, behavior inventory, compatibility effect, tests, notices, and ownership are complete. Explicit temporary deferrals may remain open, but they must be visible in the checked-in ledger and refresh summary; **Reviewed** must never conceal an unclassified gap.

## 6. Validate

For every thematic unit:

```sh
cargo fmt
CARGO_INCREMENTAL=0 cargo check -p xai-grok-pager-bin
PYTHONDONTWRITEBYTECODE=1 python3 -I -B \
  fork/scripts/check_manifest.py --strict-coverage
```

Also run:

- focused tests for touched crates and provider/session contracts;
- negative tests proving credentials/endpoints cannot cross providers;
- applicable release-contract, generated-manifest, and vendor-lock checks;
- ACP/session restore, headless, hook/scheduler, and affected TUI smoke tests.

Live provider tests require explicitly entitled credentials. Assert only redacted outcomes and never print authenticated request/response bodies. Codex coverage should include catalog, a normal turn, Fast off/on, compaction, usage, and supported hosted tools. Kimi coverage should include catalog, a normal turn, thinking effort, and hosted web search/fetch.

## 7. Acknowledge the audited Grok snapshot

This stage is only for the tracked `grok-build-upstream` source and only after all earlier stages pass. It records reviewed ancestry so GitHub's fork comparison reflects the audit without importing the disconnected upstream tree.

1. Require an exhaustive parity ledger for the exact pinned Grok commit, manifest ownership for every changed path, passing focused and binary validation, and **no open temporary Grok adoption deferral** for that snapshot.
2. Add a `coverage.upstream_acknowledgements` record containing the `grok-build-upstream` source ID; exact target commit/tree; previous reviewed commit/tree; exact target parent list; and a checked-in evidence path plus SHA-256 digest. The evidence must classify every raw added, modified, or deleted upstream path exactly once and contain zero Grok deferrals.
3. Commit all implementation, policy, ledger, source-review, and manifest changes on the linear prospective first-parent history. Validate that committed tree with `check_manifest.py --strict-coverage --prepare-upstream-acknowledgements`; never prepare from uncommitted evidence or a declaration absent from the candidate tree.
4. Create one two-parent marker with Git's `ours` **strategy** (`-s ours`, never `-X ours`). Its first parent is the validated Enhanced commit; its second parent is the exact pinned Grok commit. Use `--allow-unrelated-histories` only for the first disconnected marker.
5. Include exactly this trailer, substituting the full pinned commit:

   ```text
   Fork-Upstream-Acknowledgement: grok-build-upstream@<40-hex-commit>
   ```

6. Prove the marker tree equals its first-parent tree, the second-parent tree equals the manifest record, the evidence cites the pin, and the strict checker authenticates the first-parent path plus marker. Also prove the pinned commit is now an ancestor of the candidate.
7. Never acknowledge a merely fetched revision, an unclassified behavior set, a snapshot with an open Grok adoption deferral, or a remote head that advanced after the audit pin.

## 8. Publish the release and Homebrew formula

Run this stage only when the user explicitly authorizes publication and confirms the version for the current run. Snapshot, audit, acknowledgement, and publication remain separate decisions.

1. Re-run the release contract for the selected version and verify the acknowledgement marker is the publication candidate. Push `main` without force, create and push the annotated version tag, and monitor the tag-triggered release workflow to completion.
2. Verify the public release is neither draft nor prerelease and contains exactly the four native `grok-<version>-<os>-<arch>` binaries plus `SHA256SUMS` and `RELEASE-PROVENANCE.json`. Verify every checksum, source commit/provenance binding, and GitHub attestation without exposing authenticated data.
3. Update the shared `OpenCompanyApp/homebrew-tap` formula for all four platform URLs and release digests. Read the formula before editing; change only the version, URLs, and hashes unless the install contract itself changed.
4. Run `brew style`, `brew audit --strict --online`, and `brew test` for `opencompanyapp/tap/grok-build-enhanced`. Commit and push the tap only after those checks pass and only under the publication authorization.
5. Run `brew update` and `brew upgrade opencompanyapp/tap/grok-build-enhanced`, then verify `brew list --versions`, both `grok version` and the `agent` symlink, and that Homebrew no longer reports the formula outdated.
6. Never fall back to official xAI assets, npm packages, installer buckets, or mismatched release versions.

## 9. Finish safely

1. Compare the candidate tree with the pre-refresh snapshot.
2. Verify every changed path has manifest ownership, every changed behavior has a parity-ledger outcome, and all source revisions match the provenance records.
3. Reconcile prior open deferral IDs: carry them forward, or cite the commit and tests that close them.
4. Summarize adopted, equivalent, inapplicable, and temporarily deferred behavior; list stable deferral IDs, commits, acknowledgement parent, and validation evidence.
5. Remove every Cargo `target/` directory generated in the checkout or worktrees used for the refresh, plus temporary release downloads. Do not remove artifacts from unrelated active worktrees without explicit authorization.
6. Confirm the project and tap are aligned with their remotes and preserve any pre-existing user-owned worktree changes exactly.
7. Never force-push from refresh tooling.
