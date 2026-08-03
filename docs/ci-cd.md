# CI/CD and exact-source qualification

Grok Build Enhanced uses GitHub-hosted runners as its build execution plane and
Woodpecker as an optional trusted orchestration plane. The repository is public,
so expensive Rust and native-platform work belongs on ephemeral GitHub runners,
not on Ginger's shared CPX31 control-plane host.

## Architecture

```text
pull request / owner push
        |
        v
GitHub Actions
  static -> focused Rust -> broad Linux groups -> selected native checks
        |
        +-- nightly/weekly deep CI
        +-- exact rebase qualification artifact
        +-- native release + GitHub OIDC attestation

trusted deployment event
        |
        v
Woodpecker on Ginger
  dispatch -> watch state changes -> verify exact SHA/artifact/release
```

Woodpecker does not compile this workspace. Its Grok workflows make lightweight
GitHub API calls only. Ginger infrastructure checks, Terraform plans, notifier
publishing, and platform operations remain owned by `ginger-infra` and retain
their existing one-host safety policy.

## GitHub workflows

| Workflow | Purpose |
| --- | --- |
| `.github/workflows/fork-contracts.yml` | Required static, focused Rust, broad Linux, and path-selected native checks |
| `.github/workflows/deep-ci.yml` | Scheduled/manual broad, policy, native, and cache-disabled checks |
| `.github/workflows/rebase-qualification.yml` | Manual exact-SHA full matrix and digest-bound `qualification.json` evidence |
| `.github/workflows/release.yml` | Existing exact-tag native build, attestation, and immutable GitHub Release |

All third-party GitHub actions are pinned to full commit IDs. Pull-request jobs
use read-only repository permissions, do not use `pull_request_target`, receive
no secrets, and never run on Ginger's self-hosted agent.

### Caching

The local composite action `.github/actions/setup-rust-ci/action.yml` caches only
Cargo registry/git downloads and configures `sccache` with GitHub's cache
backend. Cache keys include OS, architecture, logical job namespace, and
`Cargo.lock`.

The following are never cached:

- Cargo `target/` trees;
- `~/.grok`, `~/.codex`, or authentication/session state;
- provider responses or credentials;
- release assets that must be rebuilt by `release.yml`; or
- mutable cross-ref R2/S3 compiler directories.

Every Rust job keeps `CARGO_INCREMENTAL=0`. A cache miss or outage may make a
job slower but must not change correctness.

## Smart local versus remote validation

Local compilation remains fully supported. Developers and coding agents should
choose the smallest local lane that gives useful feedback, then use GitHub for
broad or native qualification.

```sh
# No Rust compilation; run this for every meaningful change.
scripts/ci/run.sh static

# Provider/security/updater changes and the composed binary.
scripts/ci/run.sh rust-contracts

# Focused logical groups.
scripts/ci/run.sh core
scripts/ci/run.sh ui
scripts/ci/run.sh pty
scripts/ci/run.sh libraries

# Expensive workspace-wide policy lane.
scripts/ci/run.sh policy

# Must run on a matching native host.
scripts/ci/run.sh native aarch64-apple-darwin
```

Recommended decision rule:

1. Run formatting/static contracts and the directly affected test locally.
2. If a local Cargo target is already warm and the focused check is cheap, use
   it; CI is not a prohibition on local compilation.
3. For cross-workspace, native ARM/macOS, full PTY, clippy, cold, or release
   evidence, push an owner-controlled candidate and use GitHub Actions.
4. For an upstream refresh, do not repeatedly build every thematic commit.
   Qualify the exact final candidate remotely.
5. Agents must still follow `AGENTS.md`: remove targets they generated before
   finishing their task. Do not delete an unrelated active worktree's target.

To isolate an agent's local output explicitly:

```sh
export CARGO_TARGET_DIR="${TMPDIR:-/tmp}/grok-ci-${USER}-$$"
trap 'rm -rf "$CARGO_TARGET_DIR"' EXIT
scripts/ci/run.sh rust-contracts
```

Only use that pattern for a path created by the current process; never interpolate
an untrusted or empty path into cleanup.

## Change classification

`scripts/ci/impact.py` compares the exact base/head Git identities. Documentation
only changes skip Rust. Toolchain, lockfile, manifest, build, common-crate, and CI
changes fail safe to the full Rust/native scope. Core, UI, PTY, and library path
classes are explicit and unit tested.

Changed-file classification is an optimization. A missing/all-zero base, invalid
SHA, unresolvable diff, workspace manifest, or CI input selects full coverage.
The exact rebase and release workflows do not use path-based reduction.

## Exact rebase qualification

Trigger `.github/workflows/rebase-qualification.yml` with:

- `source_branch`: `main`, `rebase/*`, or `refresh/*`;
- `source_sha`: the exact 40-character commit contained in that branch;
- `orchestration_id`: a unique safe correlation ID;
- `confirm_qualification: true`; and
- optional `release_builds: true` for release-dist builds on all native hosts.

The workflow validates the immutable source, runs static/focused/broad/native
lanes, and uploads `rebase-qualification-<sha>` containing one
`qualification.json`. The record binds:

- repository, commit, and tree;
- `Cargo.lock` and workflow digests;
- workflow run ID/attempt;
- Rust compiler identity;
- all four native target identities;
- whether release builds were requested; and
- successful conclusions for every required lane.

Verify downloaded evidence locally with:

```sh
python3 -I -B scripts/ci/qualification.py verify \
  --file qualification.json \
  --source-sha <40-character-sha> \
  --source-tree <40-character-tree>
```

A result for another SHA/tree, an incomplete target matrix, a failed/skipped
lane, or workflow drift is not valid qualification.

## Woodpecker orchestration

The two repository-local workflows are deployment-only:

- `.woodpecker/github-qualification.yml`, target `github-qualification`;
- `.woodpecker/github-release.yml`, target `github-release`.

They use `scripts/ci/github_orchestrator.py` to dispatch, discover, monitor, and
verify GitHub runs. The monitor prints state changes only. It never fetches job
logs, prints API response bodies, or exposes the token.

### Required Woodpecker repository configuration

These are operator actions and are intentionally not performed by repository
code:

1. Activate `OpenCompanyApp/grok-build-enhanced` in Woodpecker.
2. Add repository secret `grok_github_actions_token` only to trusted deployment
   events. Never expose it to pull requests or pushes.
3. Use a fine-grained GitHub credential restricted to this repository with:
   - Actions: read/write;
   - Contents: read;
   - no Contents write, administration, secrets, or release-edit permission.
4. Permit deployment events and set the project pipeline timeout above the
   orchestrator's six-hour wait budget (for example, 420 minutes).
5. Keep the repository untrusted: no privileged steps, host volumes, or Docker
   socket mounts are required.
6. Keep the Ginger notifier read-only; it reports Woodpecker state but does not
   fetch GitHub/Woodpecker logs or add retry/cancel/release controls.

Qualification deployment parameters:

```text
GROK_QUALIFICATION_SHA=<40-character-sha>
GROK_QUALIFICATION_BRANCH=rebase/<name>
GROK_QUALIFICATION_RELEASE_BUILDS=false
```

Release deployment parameter:

```text
GROK_RELEASE_TAG=vX.Y.Z
```

The workflows derive their unique orchestration ID from the Woodpecker pipeline
number. Inputs are validated before use. The GitHub token is sent only in HTTPS
API authorization headers and is stripped before cross-host artifact redirects.

## Release flow

1. Qualify the exact commit and merge it to `main`.
2. Create the exact release tag under the repository's snapshot/publication
   rules.
3. Trigger Woodpecker deployment target `github-release` with
   `GROK_RELEASE_TAG`.
4. Woodpecker dispatches `release.yml` with an orchestration ID and waits.
5. GitHub revalidates the tag/source, builds all four native assets, smoke tests,
   attests, and publishes.
6. Woodpecker verifies the published tag and exact six-file asset allowlist.

`release.yml` still supports tag push and manual dispatch. Before making
Woodpecker the sole production trigger, choose exactly one trigger policy and
update the release contract; do not race tag-triggered and dispatched runs.

No script in this repository creates a tag, pushes a branch, changes branch
protection, provisions a secret, or deploys Ginger infrastructure.

## Suggested required checks

Keep status names stable once configured. Use the aggregate `CI result` for
ordinary pull requests because path-selected Rust/native jobs may legitimately be
skipped. Require `Exact rebase qualification result` only on the protected
refresh/publication path where the full matrix always runs.

Repository settings are shared external state and must be changed separately by
an authorized operator after the workflows have completed a successful dry run.
