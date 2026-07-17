#!/usr/bin/env bash
# Read-only verification for the frozen downstream patch stack.
set -euo pipefail

export GIT_OPTIONAL_LOCKS=0
export GIT_NO_LAZY_FETCH=1
export GIT_NO_REPLACE_OBJECTS=1
export PYTHONDONTWRITEBYTECODE=1
export LC_ALL=C

git_read() {
    command git --no-pager -c core.fsmonitor=false -c submodule.recurse=false "$@"
}

readonly DEFAULT_BASELINE="c1b5909ec707c069f1d21a93917af044e71da0d7"
readonly DEFAULT_BASELINE_TREE="54a0606cc804815975eda042f60cb3320cfb5347"
readonly DEFAULT_FROZEN_TIP="da3076874292c538bfc4efeede0cf517c2e975f0"
readonly DEFAULT_FROZEN_TREE="bbeaf9575ffa5f7bcd75b8352a178455f04c98c7"
readonly DEFAULT_DIFF_SHA256="a847902433770c47890c321d4fc5683bffd6420c138a9b481ec803c98098a4e9"
readonly DEFAULT_NAME_STATUS_SHA256="c5711d5e25ee349e9fbfe2da9fd8a2364ca22492843f63741a8eb816177fbdff"
readonly DEFAULT_NUMSTAT_SHA256="49ed966d9cca3f7635a3423388d7e5235a5ea14d92c72b3d99c73aa778261b32"

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
DEFAULT_REPO_ROOT="$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd -P)"

usage() {
    cat <<'EOF'
Usage: fork/scripts/verify_patch_stack.sh [options]

Read-only checks for a candidate downstream patch stack. Values supplied on the
command line override fork/manifest.json; pinned constants are the final
fallback. The candidate defaults to the frozen tip.

Options:
  --repo-root PATH              Git worktree to inspect
  --manifest PATH               Manifest providing pinned defaults
  --baseline REV                Baseline commit or safe literal ref
  --frozen-tip REV              Frozen downstream commit or safe literal ref
  --candidate REV               Candidate commit or safe literal ref
  --expected-baseline-tree SHA  Full expected baseline tree ID
  --expected-tree SHA           Full expected frozen/candidate tree ID
  --diff-sha256 SHA             Expected canonical binary-diff SHA-256
  --name-status-sha256 SHA      Expected canonical name-status SHA-256
  --numstat-sha256 SHA          Expected canonical numstat SHA-256
  --skip-clean-check            Do not require cleanliness when candidate is HEAD
  --no-range-diff               Suppress range-diff diagnostics
  -h, --help                    Show this help

This script never fetches, creates or moves refs, checks out, resets, cleans,
stashes, rebases, commits, or pushes.
EOF
}

fatal() {
    printf 'ERROR: %s\n' "$*" >&2
    exit 2
}

repo_argument="$DEFAULT_REPO_ROOT"
manifest_argument=""
manifest_explicit=0
baseline_override=""
frozen_override=""
candidate_override=""
baseline_tree_override=""
expected_tree_override=""
diff_hash_override=""
name_status_hash_override=""
numstat_hash_override=""
skip_clean_check=0
show_range_diff=1

while (($#)); do
    case "$1" in
        --repo-root)
            (($# >= 2)) || fatal "--repo-root requires a value"
            repo_argument=$2
            shift 2
            ;;
        --manifest)
            (($# >= 2)) || fatal "--manifest requires a value"
            manifest_argument=$2
            manifest_explicit=1
            shift 2
            ;;
        --baseline)
            (($# >= 2)) || fatal "--baseline requires a value"
            baseline_override=$2
            shift 2
            ;;
        --frozen-tip)
            (($# >= 2)) || fatal "--frozen-tip requires a value"
            frozen_override=$2
            shift 2
            ;;
        --candidate)
            (($# >= 2)) || fatal "--candidate requires a value"
            candidate_override=$2
            shift 2
            ;;
        --expected-baseline-tree)
            (($# >= 2)) || fatal "--expected-baseline-tree requires a value"
            baseline_tree_override=$2
            shift 2
            ;;
        --expected-tree)
            (($# >= 2)) || fatal "--expected-tree requires a value"
            expected_tree_override=$2
            shift 2
            ;;
        --diff-sha256)
            (($# >= 2)) || fatal "--diff-sha256 requires a value"
            diff_hash_override=$2
            shift 2
            ;;
        --name-status-sha256)
            (($# >= 2)) || fatal "--name-status-sha256 requires a value"
            name_status_hash_override=$2
            shift 2
            ;;
        --numstat-sha256)
            (($# >= 2)) || fatal "--numstat-sha256 requires a value"
            numstat_hash_override=$2
            shift 2
            ;;
        --skip-clean-check)
            skip_clean_check=1
            shift
            ;;
        --no-range-diff)
            show_range_diff=0
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        --)
            shift
            (($# == 0)) || fatal "positional arguments are not accepted"
            ;;
        *)
            fatal "unknown argument: $1"
            ;;
    esac
done

if ! repo_root="$(git_read -C "$repo_argument" rev-parse --show-toplevel 2>/dev/null)"; then
    fatal "not a Git worktree: $repo_argument"
fi
repo_root="$(CDPATH= cd -- "$repo_root" && pwd -P)"
if [[ -z "$manifest_argument" ]]; then
    manifest_path="$repo_root/fork/manifest.json"
else
    manifest_path=$manifest_argument
fi

manifest_value() {
    local key=$1
    python3 - "$manifest_path" "$key" <<'PY'
import json
import sys

class DuplicateKeyError(ValueError):
    pass

def no_duplicates(pairs):
    result = {}
    for name, value in pairs:
        if name in result:
            raise DuplicateKeyError(f"duplicate JSON key: {name!r}")
        result[name] = value
    return result

paths = {
    "algorithm": ("patch_stack", "fingerprints", "algorithm"),
    "baseline_commit": ("patch_stack", "baseline", "commit"),
    "baseline_tree": ("patch_stack", "baseline", "tree"),
    "frozen_commit": ("patch_stack", "frozen_tip", "commit"),
    "frozen_tree": ("patch_stack", "frozen_tip", "tree"),
    "diff_sha256": ("patch_stack", "fingerprints", "diff_sha256"),
    "name_status_sha256": (
        "patch_stack",
        "fingerprints",
        "name_status_sha256",
    ),
    "numstat_sha256": ("patch_stack", "fingerprints", "numstat_sha256"),
}
try:
    requested = paths[sys.argv[2]]
    with open(sys.argv[1], encoding="utf-8") as manifest_file:
        value = json.load(manifest_file, object_pairs_hook=no_duplicates)
    for component in requested:
        value = value[component]
    if not isinstance(value, str) or not value or "\n" in value or "\r" in value:
        raise ValueError(f"{'.'.join(requested)} must be a non-empty single-line string")
except (OSError, KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
    print(f"cannot read manifest value {sys.argv[2]!r}: {error}", file=sys.stderr)
    raise SystemExit(1)
print(value)
PY
}

manifest_baseline=""
manifest_baseline_tree=""
manifest_frozen=""
manifest_frozen_tree=""
manifest_diff_hash=""
manifest_name_status_hash=""
manifest_numstat_hash=""
if [[ -f "$manifest_path" ]]; then
    manifest_algorithm="$(manifest_value algorithm)" || fatal "invalid manifest: $manifest_path"
    [[ "$manifest_algorithm" == "sha256" ]] || fatal "manifest fingerprint algorithm must be sha256"
    manifest_baseline="$(manifest_value baseline_commit)" || fatal "invalid manifest: $manifest_path"
    manifest_baseline_tree="$(manifest_value baseline_tree)" || fatal "invalid manifest: $manifest_path"
    manifest_frozen="$(manifest_value frozen_commit)" || fatal "invalid manifest: $manifest_path"
    manifest_frozen_tree="$(manifest_value frozen_tree)" || fatal "invalid manifest: $manifest_path"
    manifest_diff_hash="$(manifest_value diff_sha256)" || fatal "invalid manifest: $manifest_path"
    manifest_name_status_hash="$(manifest_value name_status_sha256)" || fatal "invalid manifest: $manifest_path"
    manifest_numstat_hash="$(manifest_value numstat_sha256)" || fatal "invalid manifest: $manifest_path"
elif ((manifest_explicit)); then
    fatal "manifest does not exist: $manifest_path"
fi

baseline="${baseline_override:-${manifest_baseline:-$DEFAULT_BASELINE}}"
frozen_tip="${frozen_override:-${manifest_frozen:-$DEFAULT_FROZEN_TIP}}"
candidate="${candidate_override:-$frozen_tip}"
expected_baseline_tree="${baseline_tree_override:-${manifest_baseline_tree:-$DEFAULT_BASELINE_TREE}}"
expected_tree="${expected_tree_override:-${manifest_frozen_tree:-$DEFAULT_FROZEN_TREE}}"
expected_diff_hash="${diff_hash_override:-${manifest_diff_hash:-$DEFAULT_DIFF_SHA256}}"
expected_name_status_hash="${name_status_hash_override:-${manifest_name_status_hash:-$DEFAULT_NAME_STATUS_SHA256}}"
expected_numstat_hash="${numstat_hash_override:-${manifest_numstat_hash:-$DEFAULT_NUMSTAT_SHA256}}"

safe_revision() {
    local revision=$1
    [[ "$revision" =~ ^[A-Za-z0-9][A-Za-z0-9._/-]*$ ]] &&
        [[ "$revision" != *..* ]] &&
        [[ "$revision" != *//* ]] &&
        [[ "$revision" != *@\{* ]] &&
        [[ "$revision" != */ ]] &&
        [[ "$revision" != *.lock ]]
}

full_sha1() {
    [[ "$1" =~ ^[0-9a-f]{40}$ ]]
}

full_sha256() {
    [[ "$1" =~ ^[0-9a-f]{64}$ ]]
}

safe_revision "$baseline" || fatal "unsafe baseline revision: $baseline"
safe_revision "$frozen_tip" || fatal "unsafe frozen-tip revision: $frozen_tip"
safe_revision "$candidate" || fatal "unsafe candidate revision: $candidate"
full_sha1 "$expected_baseline_tree" || fatal "expected baseline tree must be a full lowercase SHA"
full_sha1 "$expected_tree" || fatal "expected tree must be a full lowercase SHA"
full_sha256 "$expected_diff_hash" || fatal "diff fingerprint must be a lowercase SHA-256"
full_sha256 "$expected_name_status_hash" || fatal "name-status fingerprint must be a lowercase SHA-256"
full_sha256 "$expected_numstat_hash" || fatal "numstat fingerprint must be a lowercase SHA-256"

resolve_commit() {
    git_read -C "$repo_root" rev-parse --verify "$1^{commit}" 2>/dev/null
}

baseline_commit="$(resolve_commit "$baseline")" || fatal "baseline does not resolve to a commit: $baseline"
frozen_commit="$(resolve_commit "$frozen_tip")" || fatal "frozen tip does not resolve to a commit: $frozen_tip"
candidate_commit="$(resolve_commit "$candidate")" || fatal "candidate does not resolve to a commit: $candidate"

failures=0
pass_check() {
    printf 'PASS: %s\n' "$1"
}
fail_check() {
    printf 'FAIL: %s\n' "$1" >&2
    failures=$((failures + 1))
}

printf 'Repository: %s\n' "$repo_root"
printf 'Baseline:   %s\n' "$baseline_commit"
printf 'Frozen tip: %s\n' "$frozen_commit"
printf 'Candidate:  %s\n' "$candidate_commit"

actual_baseline_tree="$(git_read -C "$repo_root" rev-parse --verify "$baseline_commit^{tree}")"
actual_frozen_tree="$(git_read -C "$repo_root" rev-parse --verify "$frozen_commit^{tree}")"
actual_candidate_tree="$(git_read -C "$repo_root" rev-parse --verify "$candidate_commit^{tree}")"

if [[ "$actual_baseline_tree" == "$expected_baseline_tree" ]]; then
    pass_check "baseline tree is exactly $expected_baseline_tree"
else
    fail_check "baseline tree mismatch (expected $expected_baseline_tree, got $actual_baseline_tree)"
fi
if [[ "$actual_frozen_tree" == "$expected_tree" ]]; then
    pass_check "frozen-tip tree is exactly $expected_tree"
else
    fail_check "frozen-tip tree mismatch (expected $expected_tree, got $actual_frozen_tree)"
fi
if [[ "$actual_candidate_tree" == "$expected_tree" ]]; then
    pass_check "candidate tree is exactly $expected_tree"
else
    fail_check "candidate tree mismatch (expected $expected_tree, got $actual_candidate_tree)"
fi

check_ancestry() {
    local descendant=$1
    local label=$2
    if git_read -C "$repo_root" merge-base --is-ancestor "$baseline_commit" "$descendant"; then
        pass_check "baseline is an ancestor of $label"
    else
        fail_check "baseline is not an ancestor of $label"
    fi
}
check_ancestry "$frozen_commit" "frozen tip"
check_ancestry "$candidate_commit" "candidate"

if ((skip_clean_check)); then
    printf 'INFO: clean-status check explicitly skipped\n'
elif [[ "$(git_read -C "$repo_root" rev-parse --is-bare-repository)" == "true" ]]; then
    printf 'INFO: clean-status check is not applicable to a bare repository\n'
else
    head_commit="$(git_read -C "$repo_root" rev-parse --verify HEAD^{commit})"
    if [[ "$candidate_commit" == "$head_commit" ]]; then
        worktree_status="$(git_read -C "$repo_root" status --porcelain=v1 --untracked-files=all)"
        if [[ -z "$worktree_status" ]]; then
            pass_check "candidate is checked-out HEAD and the worktree/index are clean"
        else
            fail_check "candidate is checked-out HEAD but the worktree or index is not clean"
        fi
    else
        printf 'INFO: clean-status check is not applicable because candidate is not checked-out HEAD\n'
    fi
fi

set +e
git_read -C "$repo_root" diff --quiet --no-ext-diff --no-textconv \
    "$frozen_commit" "$candidate_commit" --
tree_diff_status=$?
set -e
if ((tree_diff_status == 0)); then
    pass_check "candidate tree diff is exactly equal to the frozen ref"
elif ((tree_diff_status == 1)); then
    fail_check "candidate tree diff is not equal to the frozen ref"
else
    fail_check "git could not compare candidate and frozen trees (status $tree_diff_status)"
fi

sha256_stream() {
    python3 -c '
import hashlib
import sys
hasher = hashlib.sha256()
while True:
    chunk = sys.stdin.buffer.read(1024 * 1024)
    if not chunk:
        break
    hasher.update(chunk)
print(hasher.hexdigest())
'
}

fingerprint() {
    local kind=$1
    local right_commit=$2
    case "$kind" in
        diff)
            git_read -c core.quotePath=false -C "$repo_root" diff \
                --binary --full-index --no-renames --no-ext-diff --no-textconv \
                "$baseline_commit" "$right_commit" -- | sha256_stream
            ;;
        name-status)
            git_read -c core.quotePath=false -C "$repo_root" diff \
                --name-status --no-renames --no-ext-diff --no-textconv -z \
                "$baseline_commit" "$right_commit" -- | sha256_stream
            ;;
        numstat)
            git_read -c core.quotePath=false -C "$repo_root" diff \
                --numstat --no-renames --no-ext-diff --no-textconv -z \
                "$baseline_commit" "$right_commit" -- | sha256_stream
            ;;
        *)
            return 2
            ;;
    esac
}

check_fingerprint() {
    local kind=$1
    local expected=$2
    local frozen_hash candidate_hash
    if ! frozen_hash="$(fingerprint "$kind" "$frozen_commit")"; then
        fail_check "could not calculate frozen $kind fingerprint"
        return
    fi
    if ! candidate_hash="$(fingerprint "$kind" "$candidate_commit")"; then
        fail_check "could not calculate candidate $kind fingerprint"
        return
    fi
    if [[ "$frozen_hash" == "$expected" ]]; then
        pass_check "frozen $kind fingerprint matches $expected"
    else
        fail_check "frozen $kind fingerprint mismatch (expected $expected, got $frozen_hash)"
    fi
    if [[ "$candidate_hash" == "$expected" ]]; then
        pass_check "candidate $kind fingerprint matches $expected"
    else
        fail_check "candidate $kind fingerprint mismatch (expected $expected, got $candidate_hash)"
    fi
    if [[ "$candidate_hash" == "$frozen_hash" ]]; then
        pass_check "candidate $kind fingerprint equals the frozen ref"
    else
        fail_check "candidate $kind fingerprint differs from the frozen ref"
    fi
}

check_fingerprint diff "$expected_diff_hash"
check_fingerprint name-status "$expected_name_status_hash"
check_fingerprint numstat "$expected_numstat_hash"

if ((show_range_diff)); then
    printf '%s\n' '--- range-diff diagnostics (non-authoritative) ---'
    if ! git_read -c color.ui=false -C "$repo_root" range-diff \
        --no-ext-diff --no-textconv \
        "$baseline_commit..$frozen_commit" "$baseline_commit..$candidate_commit"; then
        printf '%s\n' 'INFO: range-diff diagnostics were unavailable; exact checks above remain authoritative.'
    fi
    printf '%s\n' '--- end range-diff diagnostics ---'
fi

if ((failures)); then
    printf 'Patch-stack verification failed with %d check failure(s).\n' "$failures" >&2
    exit 1
fi
printf '%s\n' 'Patch-stack verification succeeded.'
