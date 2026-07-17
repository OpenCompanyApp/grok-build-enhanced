# Downstream compatibility manifest

This directory records the immutable inputs and initial path ownership for the
OpenCompanyApp downstream patch stack. It is compatibility metadata, not a
release mechanism and not an alternative source of upstream-version truth.
`UPSTREAM_VERSIONS.md` remains the human-reviewed upstream ledger.

The Phase 2 snapshot is pinned to:

- Grok Build baseline `c1b5909ec707c069f1d21a93917af044e71da0d7`;
- frozen downstream tip `da3076874292c538bfc4efeede0cf517c2e975f0`;
- frozen tip tree `bbeaf9575ffa5f7bcd75b8352a178455f04c98c7`.

All scripts in this directory are read-only checkers. They do not fetch, create
or move refs, checkout, reset, stash, clean, rebase, commit, or push. They never
interpret a command from JSON. The manifest names checks by ID, and
`check_manifest.py` maps the supported IDs to code compiled into the checker.

## Manifest contract

`manifest.json` contains:

- full Git commit and tree object IDs for the baseline and frozen tip;
- SHA-256 fingerprints of canonical binary, name-status, and numstat diffs;
- reviewed and latest-fetched revisions and trees for every project in
  `UPSTREAM_VERSIONS.md`;
- stable source and feature IDs;
- safe repository-relative feature globs plus literal integration and legal
  anchors; and
- a deliberate `allow_uncovered` policy for incremental ownership coverage.

Feature paths describe ownership of the downstream diff from the pinned
baseline to the frozen tip. Coverage is intentionally incomplete in this first
manifest. Unowned changed paths are warnings while `allow_uncovered` is true,
but the checker always prints every uncovered path in bytewise-sorted order.
Use `--strict-coverage` to make the same list fatal. Do not add exclusions merely
to silence that report; add a feature owner when ownership is understood.

Path patterns use `/` separators. `*` and `?` stay within one path component;
`**` is accepted only as a complete component. Absolute paths, parent traversal,
empty components, control characters, backslashes, `.git`, and shell-style
expansion syntax are rejected. Integration and legal anchors are literal files
that must exist both in the checkout and at the frozen tip.

The fingerprints are SHA-256 hashes of these exact, read-only Git byte streams,
with `core.quotePath=false`, `--no-renames`, `--no-ext-diff`, and
`--no-textconv` applied:

- `git diff --binary --full-index <baseline> <tip> --`;
- `git diff --name-status -z <baseline> <tip> --`;
- `git diff --numstat -z <baseline> <tip> --`.

## Checks

Run the standard-library-only test suite and manifest checker from the repository
root:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s fork/scripts/tests -v
PYTHONDONTWRITEBYTECODE=1 python3 fork/scripts/check_manifest.py
```

Strict ownership coverage is available without changing the manifest:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 fork/scripts/check_manifest.py --strict-coverage
```

Verify the frozen patch stack with:

```sh
fork/scripts/verify_patch_stack.sh
```

The verifier defaults its candidate to the frozen tip. Supply an explicit
candidate when evaluating another commit:

```sh
fork/scripts/verify_patch_stack.sh --candidate <full-commit-or-safe-ref>
```

Run `fork/scripts/verify_patch_stack.sh --help` for repository, manifest,
baseline, frozen-tip, expected-tree, fingerprint, clean-status, and range-diff
options. A candidate that resolves to the checked-out `HEAD` is also required to
have a clean index and worktree unless `--skip-clean-check` is explicitly used.
Range-diff output is diagnostic only; exact tree, diff, and fingerprint checks
remain authoritative.

## Snapshot, rebase, and publication are separate

1. **Snapshot:** preserve the downstream tip and record its full commit, tree,
   and fingerprints. Do not move a branch or publish as part of snapshotting.
2. **Rebase:** work in a disposable branch or isolated worktree. Keep the frozen
   snapshot reachable and compare the candidate to it with explicit verifier
   arguments.
3. **Publication:** review the verified result, then perform any push as a
   separately authorized operation. These scripts never push. If a force update
   is explicitly approved, use `--force-with-lease` and never rewrite upstream,
   baseline, or frozen snapshot refs.
