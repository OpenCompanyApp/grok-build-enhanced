# Downstream compatibility manifest

This directory records immutable checkpoint-history attestations and complete
baseline-to-candidate path ownership for the OpenCompanyApp downstream patch
stack. It is compatibility metadata, not a release mechanism and not an
alternative source of upstream-version truth. `UPSTREAM_VERSIONS.md` remains
the human-reviewed upstream ledger.

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
- a fail-closed coverage policy requiring every baseline-to-coverage-candidate
  downstream path to have an owner.

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

Coverage-candidate authentication is deliberately separate from those frozen
attestations. `check_manifest.py` resolves the checked-out committed `HEAD` by
default, or resolves one explicitly supplied full OID or safe literal ref once,
and then uses only the resulting commit ID. It walks raw parent headers until it
reaches the exact frozen commit or a pinned thematic checkpoint. Ordinary
post-checkpoint commits must have exactly one parent. The only merge exception is
a declared two-parent upstream acknowledgement whose second parent is the exact
audited source pin, whose tree equals its first-parent tree, and whose commit
message ends in the exact acknowledgement trailer. Roots before a checkpoint,
undeclared or content-changing side-parent merges, grafts, and replacement
objects fail. The checkpoint's complete exact/thematic attestation remains a
separate mandatory check. The manifest therefore does not pin each evolving
maintenance HEAD.

### Audited upstream acknowledgements

An acknowledgement declaration binds a source ID and exact target commit/tree to
its previous reviewed commit/tree, the target's exact parent list, and a regular
checked-in evidence blob with a SHA-256 digest. The checker independently compares
the two upstream trees as raw path entries and requires the Markdown evidence to
classify every added, modified, or deleted raw path exactly once. Any missing,
extra, duplicate, status-mismatched, unclassified, or temporarily deferred Grok
path fails authentication.

The declaration and evidence must be committed before the marker. Validate that
prospective first parent with:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -I -B \
  fork/scripts/check_manifest.py --strict-coverage \
  --prepare-upstream-acknowledgements
```

The preparation flag permits declared-but-not-yet-merged acknowledgements only
while checking their prospective first-parent tree. The marker is then created
with Git's `ours` strategy (`-s ours`, never `-X ours`) and exactly two parents.
Its final trailer block must contain exactly:

```text
Fork-Upstream-Acknowledgement: <source-id>@<40-hex-commit>
```

Normal strict validation after the marker rejects unused declarations. It also
reads the declaration and evidence from the marker's own first-parent tree, so a
later follow-up cannot retroactively authorize or rewrite an earlier merge.
Source `reviewed` state may advance after an audit; `patch_stack.baseline` remains
the immutable historical fork baseline and never moves with source review state.

Feature-pattern matches, committed anchors, and fail-closed coverage all use
the raw delta from the pinned baseline tree to the resolved coverage-candidate
tree. An anchor must be a regular blob in that committed candidate. If the
candidate is checked-out `HEAD`, each anchor's checkout components must also be
regular and free of symlinks. An uncommitted or untracked path can neither enter
the tree delta nor satisfy a missing committed anchor.

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

Ordinary source records intentionally contain only reviewed/latest commit IDs;
they do not claim external tree mappings. A Grok acknowledgement is the narrow
exception: it records an exact audited target tree and previous reviewed tree,
and the checker authenticates both available Git objects, exact target parents,
ancestry, raw path delta, and evidence binding. The upstream ledger's remote,
tracked ref, and reviewed/latest commit text remain cross-checked exactly.

## Ownership and anchors

Feature paths describe ownership of the downstream tree delta from the pinned
baseline to the resolved committed coverage candidate. At this revision all 674
baseline-to-`HEAD` changed paths have at least one feature owner, and
`coverage.allow_uncovered` is permanently `false`. The checker prints any future
uncovered path in bytewise order and fails. Do not add exclusions merely to
silence that report; add a feature owner only after ownership is understood.

Path patterns use `/` separators. `*` and `?` stay within one path component;
`**` is accepted only as a complete component. Absolute paths, parent traversal,
empty components, control/formatting/surrogate characters, backslashes, `.git`,
and shell-style expansion syntax are rejected. Integration and legal anchors
are literal files. Candidate entries are read from NUL-delimited tree data and
must be regular blobs. When the candidate is checked-out `HEAD`, every checkout
component is also inspected with `lstat`; symlinks and non-regular modes are
rejected.

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

The manifest check defaults coverage to the committed `HEAD`, not worktree or
untracked content. To review another linear descendant, pass a full commit OID
or safe literal ref explicitly while retaining the same isolated environment:

```sh
COVERAGE_CANDIDATE=refs/heads/reviewed-candidate
/usr/bin/env -i PATH="$SAFE_PATH" HOME=/nonexistent-fork-guardrail-home \
  XDG_CONFIG_HOME=/nonexistent-fork-guardrail-xdg LANG=C LC_ALL=C \
  PYTHONDONTWRITEBYTECODE=1 FORK_GUARDRAIL_GIT="$GIT" \
  "$PYTHON" -I -B fork/scripts/check_manifest.py --strict-coverage \
  --coverage-candidate "$COVERAGE_CANDIDATE"
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
