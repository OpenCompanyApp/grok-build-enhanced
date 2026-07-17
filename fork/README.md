# Downstream compatibility manifest

This directory records immutable history attestations and complete frozen-tree
path ownership for the OpenCompanyApp downstream patch stack. It is
compatibility metadata, not a release mechanism and not an alternative source
of upstream-version truth. `UPSTREAM_VERSIONS.md` remains the human-reviewed
upstream ledger.

The Phase 2 snapshot is pinned to:

- Grok Build baseline commit `c1b5909ec707c069f1d21a93917af044e71da0d7`
  and tree `54a0606cc804815975eda042f60cb3320cfb5347`;
- exact frozen commit `da3076874292c538bfc4efeede0cf517c2e975f0`;
- frozen tree `bbeaf9575ffa5f7bcd75b8352a178455f04c98c7`;
- the explicitly allowed thematic-equivalent candidate
  `7ed3320c797852ed5894f1fc2fca7cee6827768f`, currently named by
  `archive/tree-equivalent-da30768`.

All scripts in this directory are read-only checkers. They do not fetch, create
or move refs, check out, reset, stash, clean, rebase, commit, or push. They never
interpret a command from JSON. The manifest names checks by ID, and
`check_manifest.py` maps supported IDs to code compiled into the checker.

## Manifest and history contract

`manifest.json` contains:

- full lowercase Git commit and tree IDs for the local baseline and frozen tip;
- an ordered exact-history attestation and an explicit allowlist of thematic
  history attestations;
- exact remote URLs, tracked refs, and reviewed/latest-fetched commit IDs for
  every project in `UPSTREAM_VERSIONS.md`;
- stable source and feature IDs;
- safe repository-relative feature globs plus literal integration and legal
  anchors; and
- a fail-closed coverage policy requiring every frozen downstream path to have
  an owner.

Exact verification requires the candidate commit itself to equal the frozen
commit. A different commit is never accepted merely because it has the same
final tree. Thematic equivalence is a separate, explicit mode: the candidate
must equal a manifest-pinned candidate commit, end at the frozen tree, descend
linearly from the baseline, and match the complete ordered series attestation.
The history checker rejects merge commits (including side parents), empty
commits, incorrect commit counts, and any intermediate path churn whose final
entry is identical to its baseline entry. That last rule rejects add/delete and
revert sequences hidden by a net-zero final tree. Parent and tree headers are
read from raw commit objects, with replacement objects disabled, so graph
presentation, deprecated grafts, and range-diff cannot authenticate parentage.

The authoritative delta format is `git-tree-delta-v1`. For each parent/child
tree pair, the checker reads `git ls-tree -r -z --full-tree`, compares raw path
entries, and hashes:

1. the ASCII format name followed by NUL; then
2. for every changed raw path in bytewise order: path, old mode/type/object ID,
   and new mode/type/object ID.

Every field is encoded as an unsigned eight-byte big-endian length followed by
its bytes. A missing old or new entry is an empty field. The
`git-linear-history-v1` series digest uses the same framing for baseline
commit/tree, candidate commit/tree, commit count, and each ordered
commit/parent/tree/delta digest. This serialization is based only on immutable
Git tree and commit objects; it does not use porcelain diff output and is
independent of diff drivers, textconv, attributes, rename settings, locale, or
path quoting.

Range-diff remains optional human-readable diagnostics. It is explicitly
non-authoritative and is never accepted as proof. Diagnostics are suppressed
when local Git configuration declares an external diff or clean/smudge/process
transform, rather than risk executing a configured command.

The source records intentionally contain no reviewed/latest external tree IDs.
Those external objects are not mapped into this repository, so the checker only
cross-checks the ledger's remote, tracked ref, and commit text exactly. It
reports that no external tree mappings are claimed; it does not print an OK
claim for unverified external trees.

## Ownership and anchors

Feature paths describe ownership of the downstream tree delta from the pinned
baseline to the frozen tip. All 652 frozen changed paths have at least one
feature owner, and `coverage.allow_uncovered` is permanently `false`. The
checker prints any future uncovered path in bytewise order and fails. Do not add
exclusions merely to silence that report; add a feature owner only after
ownership is understood.

Path patterns use `/` separators. `*` and `?` stay within one path component;
`**` is accepted only as a complete component. Absolute paths, parent traversal,
empty components, control/formatting/surrogate characters, backslashes, `.git`,
and shell-style expansion syntax are rejected. Integration and legal anchors
are literal files. Every checkout component is inspected with `lstat`, and the
frozen entry is read from NUL-delimited tree data. Symlinks and non-regular Git
modes are rejected in both places.

## Isolated checks

Use isolated Python mode, disable bytecode, and remove ambient Python paths. A
fully sanitized invocation from the repository root is:

```sh
SAFE_PATH=/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin
PYTHON=$(PATH="$SAFE_PATH" command -v python3)
GIT=$(PATH="$SAFE_PATH" command -v git)
/usr/bin/env -i PATH="$SAFE_PATH" HOME=/nonexistent-fork-guardrail-home \
  XDG_CONFIG_HOME=/nonexistent-fork-guardrail-xdg LANG=C LC_ALL=C \
  PYTHONDONTWRITEBYTECODE=1 FORK_GUARDRAIL_GIT="$GIT" \
  "$PYTHON" -I -B -m unittest discover -s fork/scripts/tests -v
/usr/bin/env -i PATH="$SAFE_PATH" HOME=/nonexistent-fork-guardrail-home \
  XDG_CONFIG_HOME=/nonexistent-fork-guardrail-xdg LANG=C LC_ALL=C \
  PYTHONDONTWRITEBYTECODE=1 FORK_GUARDRAIL_GIT="$GIT" \
  "$PYTHON" -I -B fork/scripts/check_manifest.py --strict-coverage
```

`verify_patch_stack.sh` is a thin launcher. It parses no manifest fields in the
shell, replaces the environment, omits `PYTHONPATH`/`PYTHONHOME`, and invokes the
same Python implementation with `-I -B`. Exact verification is the default:

```sh
fork/scripts/verify_patch_stack.sh --no-range-diff
```

Authenticate the pinned archive history only through explicit thematic mode:

```sh
fork/scripts/verify_patch_stack.sh \
  --candidate archive/tree-equivalent-da30768 \
  --history-mode thematic-equivalence \
  --attestation archive-tree-equivalent-da30768 \
  --no-range-diff
```

A safe candidate ref is resolved once to a full commit ID; all proof operations
then use that immutable ID. When the candidate is checked-out `HEAD`, the
verifier checks the index and raw worktree blobs without porcelain diff,
attributes, filters, or external commands. Use `--skip-clean-check` only when a
review explicitly makes cleanliness inapplicable.

## Snapshot, rebase, and publication are separate

1. **Snapshot:** preserve the downstream tip and record its immutable commit,
   tree, and ordered history attestation. Do not move a branch or publish as
   part of snapshotting.
2. **Rebase:** work in a disposable branch or isolated worktree. Keep the frozen
   snapshot reachable and verify the candidate against the manifest.
3. **Publication:** review the verified result, then perform any push as a
   separately authorized operation. These scripts never push. If a force update
   is explicitly approved, use `--force-with-lease` and never rewrite upstream,
   baseline, or frozen snapshot refs.
