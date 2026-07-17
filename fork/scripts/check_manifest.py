#!/usr/bin/env python3
"""Validate fork guardrails without changing a worktree, index, object store, or refs."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import stat
import subprocess
import sys
import unicodedata
from dataclasses import dataclass, field
from pathlib import Path, PurePosixPath
from typing import Any, Callable, Iterable
from urllib.parse import urlsplit

sys.dont_write_bytecode = True

SHA1_RE = re.compile(r"^[0-9a-f]{40}$")
HEX_OBJECT_NAME_RE = re.compile(r"^[0-9A-Fa-f]+$")
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
ID_RE = re.compile(r"^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$")
SAFE_REF_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._/-]*$")
SHA1_SEARCH_RE = re.compile(r"(?<![0-9A-Fa-f])([0-9A-Fa-f]{40})(?![0-9A-Fa-f])")
REMOTE_REF_CELL_RE = re.compile(r"^`([^`]+)`\s+`([^`]+)`$")
GLOB_META = frozenset("*?[]{}")
UNSAFE_UNICODE_CATEGORIES = frozenset({"Cc", "Cf", "Cs", "Zl", "Zp"})
REGULAR_GIT_MODES = frozenset({"100644", "100755"})
SERIES_FORMAT = "git-linear-history-v1"
DELTA_FORMAT = "git-tree-delta-v1"
MAX_ATTESTED_COMMITS = 10_000
MAX_COVERAGE_COMMITS = 10_000

KNOWN_CHECK_IDS = (
    "patch-stack-objects",
    "patch-stack-history",
    "coverage-candidate",
    "upstream-revisions",
    "feature-paths",
    "downstream-coverage",
)


class DuplicateKeyError(ValueError):
    """Raised when JSON contains an ambiguous duplicate object key."""


class GitError(RuntimeError):
    """Raised when a read-only Git plumbing query cannot be completed."""


@dataclass(frozen=True)
class TreeEntry:
    mode: str
    object_type: str
    oid: str


@dataclass(frozen=True)
class ComputedCommit:
    commit: str
    parent: str
    tree: str
    delta_sha256: str

    def manifest_record(self) -> dict[str, str]:
        return {
            "commit": self.commit,
            "tree": self.tree,
            "delta_sha256": self.delta_sha256,
        }


@dataclass
class HistoryAnalysis:
    records: list[ComputedCommit] = field(default_factory=list)
    series_sha256: str = ""
    errors: list[str] = field(default_factory=list)


@dataclass
class Outcome:
    summary: str
    errors: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)
    uncovered: list[str] = field(default_factory=list)


@dataclass(frozen=True)
class UpstreamRecord:
    remote: str
    tracked_ref: str
    reviewed: str
    latest_fetched: str


@dataclass
class CheckContext:
    document: dict[str, Any]
    repo_root: Path
    upstream_versions: Path
    strict_coverage: bool
    git_binary: str = ""
    coverage_candidate: str = ""
    checked_out_head: str = ""
    _tree_cache: dict[str, dict[bytes, TreeEntry]] = field(default_factory=dict)
    _commit_cache: dict[str, tuple[str, tuple[str, ...]]] = field(default_factory=dict)
    _changed_paths: list[str] | None = None

    def __post_init__(self) -> None:
        if self.git_binary:
            candidate = self.git_binary
        else:
            candidate = os.environ.get("FORK_GUARDRAIL_GIT", "")
        if candidate:
            git_path = Path(candidate)
            if (
                not git_path.is_absolute()
                or not git_path.is_file()
                or not os.access(git_path, os.X_OK)
            ):
                raise GitError("FORK_GUARDRAIL_GIT must name an absolute Git executable")
            self.git_binary = str(git_path)
        else:
            resolved = shutil.which("git")
            if resolved is None:
                raise GitError("git is not available")
            self.git_binary = str(Path(resolved).resolve())

    @property
    def baseline(self) -> str:
        return self.document["patch_stack"]["baseline"]["commit"]

    @property
    def baseline_tree(self) -> str:
        return self.document["patch_stack"]["baseline"]["tree"]

    @property
    def frozen_tip(self) -> str:
        return self.document["patch_stack"]["frozen_tip"]["commit"]

    @property
    def frozen_tree(self) -> str:
        return self.document["patch_stack"]["frozen_tip"]["tree"]

    @property
    def coverage_tree(self) -> str:
        if not self.coverage_candidate:
            raise GitError("coverage candidate has not been resolved")
        return self.commit_tree(self.coverage_candidate)

    def set_coverage_candidate(self, candidate: str, checked_out_head: str) -> None:
        if SHA1_RE.fullmatch(candidate) is None:
            raise GitError(f"invalid resolved coverage candidate: {candidate!r}")
        if SHA1_RE.fullmatch(checked_out_head) is None:
            raise GitError(f"invalid resolved checked-out HEAD: {checked_out_head!r}")
        self.coverage_candidate = candidate
        self.checked_out_head = checked_out_head
        self._changed_paths = None

    def git_environment(self) -> dict[str, str]:
        binary_dir = str(Path(self.git_binary).parent)
        return {
            "PATH": os.pathsep.join((binary_dir, os.defpath)),
            "HOME": "/nonexistent-fork-guardrail-home",
            "XDG_CONFIG_HOME": "/nonexistent-fork-guardrail-xdg",
            "LANG": "C",
            "LC_ALL": "C",
            "GIT_OPTIONAL_LOCKS": "0",
            "GIT_NO_LAZY_FETCH": "1",
            "GIT_NO_REPLACE_OBJECTS": "1",
            "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_CONFIG_GLOBAL": os.devnull,
            "GIT_CONFIG_SYSTEM": os.devnull,
            "GIT_ATTR_NOSYSTEM": "1",
            "GIT_LITERAL_PATHSPECS": "1",
            "GIT_TERMINAL_PROMPT": "0",
        }

    def git_command(self, *arguments: str) -> list[str]:
        return [
            self.git_binary,
            "--no-pager",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "submodule.recurse=false",
            "-c",
            "core.untrackedCache=false",
            "-c",
            "gc.auto=0",
            "-c",
            "maintenance.auto=false",
            "-c",
            "core.quotePath=false",
            "-c",
            "core.warnAmbiguousRefs=true",
            "-c",
            "color.ui=false",
            "-c",
            f"core.attributesFile={os.devnull}",
            "-c",
            f"core.excludesFile={os.devnull}",
            "-c",
            f"core.hooksPath={os.devnull}",
            "-C",
            str(self.repo_root),
            *arguments,
        ]

    def git(
        self,
        *arguments: str,
        check: bool = True,
        input_bytes: bytes | None = None,
    ) -> subprocess.CompletedProcess[bytes]:
        try:
            result = subprocess.run(
                self.git_command(*arguments),
                check=False,
                input=input_bytes,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                env=self.git_environment(),
            )
        except OSError as error:
            raise GitError(f"unable to run git: {error}") from error
        if check and result.returncode != 0:
            detail = result.stderr.decode("utf-8", errors="backslashreplace").strip()
            if not detail:
                detail = f"git exited with status {result.returncode}"
            raise GitError(detail)
        return result

    def git_passthrough(self, *arguments: str) -> int:
        try:
            return subprocess.run(
                self.git_command(*arguments),
                check=False,
                env=self.git_environment(),
            ).returncode
        except OSError as error:
            raise GitError(f"unable to run git: {error}") from error

    def resolve_commit(self, revision: str) -> str:
        output = self.git("rev-parse", "--verify", f"{revision}^{{commit}}").stdout
        try:
            commit = output.decode("ascii").strip()
        except UnicodeDecodeError as error:
            raise GitError("git returned a non-ASCII commit ID") from error
        if SHA1_RE.fullmatch(commit) is None:
            raise GitError(f"git returned an invalid full commit ID: {commit!r}")
        return commit

    def commit_metadata(self, commit: str) -> tuple[str, tuple[str, ...]]:
        cached = self._commit_cache.get(commit)
        if cached is not None:
            return cached
        payload = self.git("cat-file", "commit", commit).stdout
        header, separator, _message = payload.partition(b"\n\n")
        if not separator:
            raise GitError(f"commit {commit} has no header/message separator")
        trees: list[str] = []
        parents: list[str] = []
        for line in header.split(b"\n"):
            if line.startswith(b"tree "):
                raw_value = line[5:]
                try:
                    trees.append(raw_value.decode("ascii"))
                except UnicodeDecodeError as error:
                    raise GitError(f"commit {commit} has a non-ASCII tree ID") from error
            elif line.startswith(b"parent "):
                raw_value = line[7:]
                try:
                    parents.append(raw_value.decode("ascii"))
                except UnicodeDecodeError as error:
                    raise GitError(f"commit {commit} has a non-ASCII parent ID") from error
        if len(trees) != 1 or SHA1_RE.fullmatch(trees[0]) is None:
            raise GitError(f"commit {commit} has an invalid tree header")
        if any(SHA1_RE.fullmatch(parent) is None for parent in parents):
            raise GitError(f"commit {commit} has an invalid parent header")
        metadata = (trees[0], tuple(parents))
        self._commit_cache[commit] = metadata
        return metadata

    def commit_tree(self, commit: str) -> str:
        return self.commit_metadata(commit)[0]

    def tree_map(self, tree: str) -> dict[bytes, TreeEntry]:
        cached = self._tree_cache.get(tree)
        if cached is not None:
            return cached
        payload = self.git("ls-tree", "-r", "-z", "--full-tree", tree).stdout
        entries: dict[bytes, TreeEntry] = {}
        records = payload.split(b"\0")
        if records and records[-1] == b"":
            records.pop()
        for record in records:
            try:
                metadata, path = record.split(b"\t", 1)
                mode_raw, type_raw, oid_raw = metadata.split(b" ", 2)
                mode = mode_raw.decode("ascii")
                object_type = type_raw.decode("ascii")
                oid = oid_raw.decode("ascii")
            except (ValueError, UnicodeDecodeError) as error:
                raise GitError("git ls-tree returned malformed data") from error
            if not path or b"\0" in path or path in entries:
                raise GitError("git ls-tree returned an empty or duplicate path")
            if SHA1_RE.fullmatch(oid) is None:
                raise GitError(f"git ls-tree returned an invalid object ID: {oid!r}")
            entries[path] = TreeEntry(mode, object_type, oid)
        self._tree_cache[tree] = entries
        return entries

    def changed_paths(self) -> list[str]:
        if self._changed_paths is None:
            baseline_entries = self.tree_map(self.baseline_tree)
            candidate_entries = self.tree_map(self.coverage_tree)
            changed_raw = sorted(
                path
                for path in set(baseline_entries) | set(candidate_entries)
                if baseline_entries.get(path) != candidate_entries.get(path)
            )
            try:
                changed = [path.decode("utf-8") for path in changed_raw]
            except UnicodeDecodeError as error:
                raise GitError(
                    "the baseline/candidate trees contain a non-UTF-8 path that cannot be represented in JSON"
                ) from error
            self._changed_paths = changed
        return self._changed_paths


def _reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise DuplicateKeyError(f"duplicate JSON key: {key!r}")
        result[key] = value
    return result


def load_manifest(path: Path) -> dict[str, Any]:
    try:
        with path.open("r", encoding="utf-8") as manifest_file:
            value = json.load(manifest_file, object_pairs_hook=_reject_duplicate_keys)
    except (OSError, UnicodeError, json.JSONDecodeError, DuplicateKeyError) as error:
        raise ValueError(f"cannot read {path}: {error}") from error
    if not isinstance(value, dict):
        raise ValueError(f"{path} must contain a JSON object")
    return value


def _contains_unsafe_unicode(value: str) -> bool:
    return any(
        unicodedata.category(character) in UNSAFE_UNICODE_CATEGORIES
        for character in value
    )


def _unknown_keys(
    value: dict[str, Any], allowed: set[str], location: str, errors: list[str]
) -> None:
    for key in sorted(set(value) - allowed):
        errors.append(f"{location} contains unsupported key {key!r}")


def _require_keys(
    value: dict[str, Any], required: set[str], location: str, errors: list[str]
) -> None:
    for key in sorted(required - set(value)):
        errors.append(f"{location} is missing required key {key!r}")


def _as_object(value: Any, location: str, errors: list[str]) -> dict[str, Any] | None:
    if not isinstance(value, dict):
        errors.append(f"{location} must be an object")
        return None
    return value


def _as_list(value: Any, location: str, errors: list[str]) -> list[Any] | None:
    if not isinstance(value, list):
        errors.append(f"{location} must be an array")
        return None
    return value


def _string(
    value: Any, location: str, errors: list[str], *, nonempty: bool = True
) -> str | None:
    if not isinstance(value, str):
        errors.append(f"{location} must be a string")
        return None
    if nonempty and not value:
        errors.append(f"{location} must not be empty")
        return None
    if _contains_unsafe_unicode(value):
        errors.append(f"{location} must not contain controls, NUL, formatting characters, or surrogates")
        return None
    return value


def _string_list(
    value: Any, location: str, errors: list[str], *, nonempty: bool = True
) -> list[str]:
    items = _as_list(value, location, errors)
    if items is None:
        return []
    if nonempty and not items:
        errors.append(f"{location} must not be empty")
    strings: list[str] = []
    for index, item in enumerate(items):
        string = _string(item, f"{location}[{index}]", errors)
        if string is not None:
            strings.append(string)
    seen: set[str] = set()
    for item in strings:
        if item in seen:
            errors.append(f"{location} contains duplicate value {item!r}")
        seen.add(item)
    return strings


def _validate_full_sha(value: Any, location: str, errors: list[str]) -> str | None:
    string = _string(value, location, errors)
    if string is not None and SHA1_RE.fullmatch(string) is None:
        errors.append(f"{location} must be a lowercase full 40-hex Git object ID")
        return None
    return string


def _validate_sha256(value: Any, location: str, errors: list[str]) -> str | None:
    string = _string(value, location, errors)
    if string is not None and SHA256_RE.fullmatch(string) is None:
        errors.append(f"{location} must be a lowercase full 64-hex SHA-256 digest")
        return None
    return string


def _validate_id(value: Any, location: str, errors: list[str]) -> str | None:
    string = _string(value, location, errors)
    if string is not None and ID_RE.fullmatch(string) is None:
        errors.append(f"{location} must use lowercase kebab-case beginning with a letter")
        return None
    return string


def validate_repo_path(value: Any, *, allow_glob: bool) -> str | None:
    """Return a path-safety error, or None when the repository path is safe."""

    if not isinstance(value, str) or not value:
        return "must be a non-empty string"
    if value != value.strip():
        return "must not have leading or trailing whitespace"
    if _contains_unsafe_unicode(value):
        return "must not contain controls, NUL, formatting characters, or surrogates"
    if "\\" in value:
        return "must use '/' separators, not backslashes"
    if value.startswith(("/", "~")) or re.match(r"^[A-Za-z]:", value):
        return "must be repository-relative"
    if ":" in value:
        return "must not contain ':'"
    if value.endswith("/") or "//" in value:
        return "must not contain empty path components"

    components = value.split("/")
    if any(component in {"", ".", ".."} for component in components):
        return "must not contain empty, '.' or '..' path components"
    if any(component.lower() == ".git" for component in components):
        return "must not address Git administrative paths"
    if str(PurePosixPath(value)) != value:
        return "must be in normalized POSIX form"

    if not allow_glob and any(character in GLOB_META for character in value):
        return "must be a literal path without glob metacharacters"
    if allow_glob:
        if any(character in value for character in "[]{}"):
            return "uses unsupported shell-style expansion syntax"
        for component in components:
            if "**" in component and component != "**":
                return "may use '**' only as a complete path component"
    return None


def compile_repo_glob(pattern: str) -> re.Pattern[str]:
    output: list[str] = ["^"]
    index = 0
    while index < len(pattern):
        if pattern.startswith("**/", index):
            output.append("(?:[^/]+/)*")
            index += 3
        elif pattern.startswith("**", index):
            output.append(".*")
            index += 2
        elif pattern[index] == "*":
            output.append("[^/]*")
            index += 1
        elif pattern[index] == "?":
            output.append("[^/]")
            index += 1
        else:
            output.append(re.escape(pattern[index]))
            index += 1
    output.append("$")
    return re.compile("".join(output))


def _validate_revision_object(value: Any, location: str, errors: list[str]) -> None:
    revision = _as_object(value, location, errors)
    if revision is None:
        return
    allowed = {"commit", "tree"}
    _unknown_keys(revision, allowed, location, errors)
    _require_keys(revision, allowed, location, errors)
    if "commit" in revision:
        _validate_full_sha(revision["commit"], f"{location}.commit", errors)
    if "tree" in revision:
        _validate_full_sha(revision["tree"], f"{location}.tree", errors)


def _validate_source_revision(value: Any, location: str, errors: list[str]) -> None:
    revision = _as_object(value, location, errors)
    if revision is None:
        return
    _unknown_keys(revision, {"commit"}, location, errors)
    _require_keys(revision, {"commit"}, location, errors)
    if "commit" in revision:
        _validate_full_sha(revision["commit"], f"{location}.commit", errors)


def _validate_commit_record(value: Any, location: str, errors: list[str]) -> None:
    record = _as_object(value, location, errors)
    if record is None:
        return
    keys = {"commit", "tree", "delta_sha256"}
    _unknown_keys(record, keys, location, errors)
    _require_keys(record, keys, location, errors)
    if "commit" in record:
        _validate_full_sha(record["commit"], f"{location}.commit", errors)
    if "tree" in record:
        _validate_full_sha(record["tree"], f"{location}.tree", errors)
    if "delta_sha256" in record:
        _validate_sha256(record["delta_sha256"], f"{location}.delta_sha256", errors)


def _validate_attestation(
    value: Any,
    location: str,
    errors: list[str],
    *,
    thematic: bool,
) -> dict[str, Any] | None:
    attestation = _as_object(value, location, errors)
    if attestation is None:
        return None
    keys = {"commit_count", "series_sha256", "commits"}
    if thematic:
        keys |= {"id", "candidate"}
    _unknown_keys(attestation, keys, location, errors)
    _require_keys(attestation, keys, location, errors)
    if thematic and "id" in attestation:
        _validate_id(attestation["id"], f"{location}.id", errors)
    if thematic and "candidate" in attestation:
        _validate_revision_object(attestation["candidate"], f"{location}.candidate", errors)

    count = attestation.get("commit_count")
    if isinstance(count, bool) or not isinstance(count, int):
        errors.append(f"{location}.commit_count must be an integer")
    elif count < 1 or count > MAX_ATTESTED_COMMITS:
        errors.append(
            f"{location}.commit_count must be between 1 and {MAX_ATTESTED_COMMITS}"
        )
    if "series_sha256" in attestation:
        _validate_sha256(attestation["series_sha256"], f"{location}.series_sha256", errors)
    records = _as_list(attestation.get("commits"), f"{location}.commits", errors)
    if records is not None:
        if not records:
            errors.append(f"{location}.commits must not be empty")
        record_commits: list[str] = []
        for index, record in enumerate(records):
            _validate_commit_record(record, f"{location}.commits[{index}]", errors)
            if isinstance(record, dict) and isinstance(record.get("commit"), str):
                record_commits.append(record["commit"])
        for duplicate in sorted(
            {commit for commit in record_commits if record_commits.count(commit) > 1}
        ):
            errors.append(f"{location}.commits contains duplicate commit {duplicate!r}")
        if isinstance(count, int) and not isinstance(count, bool) and count != len(records):
            errors.append(
                f"{location}.commit_count {count} does not match {len(records)} commit record(s)"
            )
    return attestation


def _validate_remote(value: Any, location: str, errors: list[str]) -> None:
    remote = _string(value, location, errors)
    if remote is None:
        return
    try:
        parsed = urlsplit(remote)
    except ValueError:
        errors.append(f"{location} must be a well-formed absolute HTTPS URL")
        return
    if parsed.scheme != "https" or not parsed.netloc or not parsed.path:
        errors.append(f"{location} must be an absolute HTTPS URL")
    if parsed.username is not None or parsed.password is not None:
        errors.append(f"{location} must not embed credentials")
    if parsed.query or parsed.fragment:
        errors.append(f"{location} must not contain a query or fragment")


def _validate_tracked_ref(value: Any, location: str, errors: list[str]) -> None:
    tracked_ref = _string(value, location, errors)
    if tracked_ref is None:
        return
    forbidden = (
        SAFE_REF_RE.fullmatch(tracked_ref) is None
        or ".." in tracked_ref
        or "//" in tracked_ref
        or "@{" in tracked_ref
        or tracked_ref.endswith(("/", ".", ".lock"))
        or any(component.startswith(".") for component in tracked_ref.split("/"))
    )
    if forbidden:
        errors.append(f"{location} is not a safe literal Git ref name")


def validate_schema(document: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    top_level = {"schema_version", "patch_stack", "sources", "checks", "coverage", "features"}
    _unknown_keys(document, top_level, "manifest", errors)
    _require_keys(document, top_level, "manifest", errors)

    if "schema_version" in document:
        version = document["schema_version"]
        if isinstance(version, bool) or not isinstance(version, int) or version != 2:
            errors.append("schema_version must be the integer 2")

    patch_stack = _as_object(document.get("patch_stack"), "patch_stack", errors)
    frozen: dict[str, Any] | None = None
    if patch_stack is not None:
        keys = {"baseline", "frozen_tip", "history"}
        _unknown_keys(patch_stack, keys, "patch_stack", errors)
        _require_keys(patch_stack, keys, "patch_stack", errors)
        if "baseline" in patch_stack:
            _validate_revision_object(patch_stack["baseline"], "patch_stack.baseline", errors)
        if "frozen_tip" in patch_stack:
            _validate_revision_object(patch_stack["frozen_tip"], "patch_stack.frozen_tip", errors)
            if isinstance(patch_stack["frozen_tip"], dict):
                frozen = patch_stack["frozen_tip"]
        history = _as_object(patch_stack.get("history"), "patch_stack.history", errors)
        if history is not None:
            history_keys = {
                "series_format",
                "delta_format",
                "exact",
                "thematic_equivalents",
            }
            _unknown_keys(history, history_keys, "patch_stack.history", errors)
            _require_keys(history, history_keys, "patch_stack.history", errors)
            series_value = _string(
                history.get("series_format"), "patch_stack.history.series_format", errors
            )
            if series_value is not None and series_value != SERIES_FORMAT:
                errors.append(f"patch_stack.history.series_format must be {SERIES_FORMAT!r}")
            delta_value = _string(
                history.get("delta_format"), "patch_stack.history.delta_format", errors
            )
            if delta_value is not None and delta_value != DELTA_FORMAT:
                errors.append(f"patch_stack.history.delta_format must be {DELTA_FORMAT!r}")
            exact = _validate_attestation(
                history.get("exact"), "patch_stack.history.exact", errors, thematic=False
            )
            equivalents = _as_list(
                history.get("thematic_equivalents"),
                "patch_stack.history.thematic_equivalents",
                errors,
            )
            thematic_ids: list[str] = []
            thematic_candidates: list[str] = []
            if equivalents is not None:
                if not equivalents:
                    errors.append("patch_stack.history.thematic_equivalents must not be empty")
                for index, equivalent in enumerate(equivalents):
                    location = f"patch_stack.history.thematic_equivalents[{index}]"
                    parsed = _validate_attestation(
                        equivalent, location, errors, thematic=True
                    )
                    if parsed is not None and isinstance(parsed.get("id"), str):
                        thematic_ids.append(parsed["id"])
                    if parsed is not None and frozen is not None:
                        candidate = parsed.get("candidate")
                        if isinstance(candidate, dict):
                            if isinstance(candidate.get("commit"), str):
                                thematic_candidates.append(candidate["commit"])
                            if candidate.get("tree") != frozen.get("tree"):
                                errors.append(
                                    f"{location}.candidate.tree must equal patch_stack.frozen_tip.tree"
                                )
                            if candidate.get("commit") == frozen.get("commit"):
                                errors.append(
                                    f"{location}.candidate.commit must differ from the exact frozen commit"
                                )
                        records = parsed.get("commits")
                        if isinstance(records, list) and records and isinstance(records[-1], dict):
                            if isinstance(candidate, dict) and (
                                records[-1].get("commit") != candidate.get("commit")
                                or records[-1].get("tree") != candidate.get("tree")
                            ):
                                errors.append(
                                    f"{location} final commit record must equal its candidate commit/tree"
                                )
                for duplicate in sorted(
                    {item for item in thematic_ids if thematic_ids.count(item) > 1}
                ):
                    errors.append(
                        f"patch_stack.history.thematic_equivalents contains duplicate ID {duplicate!r}"
                    )
                for duplicate in sorted(
                    {
                        commit
                        for commit in thematic_candidates
                        if thematic_candidates.count(commit) > 1
                    }
                ):
                    errors.append(
                        "patch_stack.history.thematic_equivalents contains duplicate "
                        f"candidate commit {duplicate!r}"
                    )
            if exact is not None and frozen is not None:
                records = exact.get("commits")
                if isinstance(records, list) and records and isinstance(records[-1], dict):
                    if (
                        records[-1].get("commit") != frozen.get("commit")
                        or records[-1].get("tree") != frozen.get("tree")
                    ):
                        errors.append(
                            "patch_stack.history.exact final commit record must equal patch_stack.frozen_tip"
                        )

    source_items = _as_list(document.get("sources"), "sources", errors)
    source_ids: list[str] = []
    source_projects: list[str] = []
    if source_items is not None:
        if not source_items:
            errors.append("sources must not be empty")
        source_keys = {
            "id",
            "project",
            "remote",
            "tracked_ref",
            "reviewed",
            "latest_fetched",
        }
        for index, item in enumerate(source_items):
            location = f"sources[{index}]"
            source = _as_object(item, location, errors)
            if source is None:
                continue
            _unknown_keys(source, source_keys, location, errors)
            _require_keys(source, source_keys, location, errors)
            if "id" in source:
                source_id = _validate_id(source["id"], f"{location}.id", errors)
                if source_id is not None:
                    source_ids.append(source_id)
            if "project" in source:
                project = _string(source["project"], f"{location}.project", errors)
                if project is not None:
                    source_projects.append(project)
            if "remote" in source:
                _validate_remote(source["remote"], f"{location}.remote", errors)
            if "tracked_ref" in source:
                _validate_tracked_ref(source["tracked_ref"], f"{location}.tracked_ref", errors)
            if "reviewed" in source:
                _validate_source_revision(source["reviewed"], f"{location}.reviewed", errors)
            if "latest_fetched" in source:
                _validate_source_revision(
                    source["latest_fetched"], f"{location}.latest_fetched", errors
                )
    for duplicate in sorted(
        {source_id for source_id in source_ids if source_ids.count(source_id) > 1}
    ):
        errors.append(f"sources contains duplicate source ID {duplicate!r}")
    for duplicate in sorted(
        {project for project in source_projects if source_projects.count(project) > 1}
    ):
        errors.append(f"sources contains duplicate project {duplicate!r}")

    check_ids = _string_list(document.get("checks"), "checks", errors)
    for check_id in check_ids:
        if check_id not in KNOWN_CHECK_IDS:
            errors.append(f"checks contains unknown hard-coded check ID {check_id!r}")
    for missing in sorted(set(KNOWN_CHECK_IDS) - set(check_ids)):
        errors.append(f"checks is missing required hard-coded check ID {missing!r}")

    coverage = _as_object(document.get("coverage"), "coverage", errors)
    if coverage is not None:
        _unknown_keys(coverage, {"allow_uncovered"}, "coverage", errors)
        _require_keys(coverage, {"allow_uncovered"}, "coverage", errors)
        if coverage.get("allow_uncovered") is not False:
            errors.append("coverage.allow_uncovered must be false for fail-closed ownership")

    feature_items = _as_list(document.get("features"), "features", errors)
    feature_ids: list[str] = []
    known_source_ids = set(source_ids)
    if feature_items is not None:
        if not feature_items:
            errors.append("features must not be empty")
        feature_keys = {
            "id",
            "description",
            "source_ids",
            "paths",
            "integration_paths",
            "legal_paths",
        }
        for index, item in enumerate(feature_items):
            location = f"features[{index}]"
            feature = _as_object(item, location, errors)
            if feature is None:
                continue
            _unknown_keys(feature, feature_keys, location, errors)
            _require_keys(feature, feature_keys, location, errors)
            if "id" in feature:
                feature_id = _validate_id(feature["id"], f"{location}.id", errors)
                if feature_id is not None:
                    feature_ids.append(feature_id)
            if "description" in feature:
                _string(feature["description"], f"{location}.description", errors)
            references = _string_list(
                feature.get("source_ids"), f"{location}.source_ids", errors
            )
            for reference in references:
                if ID_RE.fullmatch(reference) is None:
                    errors.append(
                        f"{location}.source_ids contains invalid source ID {reference!r}"
                    )
                elif reference not in known_source_ids:
                    errors.append(
                        f"{location}.source_ids references unknown source ID {reference!r}"
                    )
            for key, allow_glob in (
                ("paths", True),
                ("integration_paths", False),
                ("legal_paths", False),
            ):
                paths = _string_list(feature.get(key), f"{location}.{key}", errors)
                for path_index, path in enumerate(paths):
                    safety_error = validate_repo_path(path, allow_glob=allow_glob)
                    if safety_error is not None:
                        errors.append(
                            f"{location}.{key}[{path_index}] {safety_error}: {path!r}"
                        )
    for duplicate in sorted(
        {feature_id for feature_id in feature_ids if feature_ids.count(feature_id) > 1}
    ):
        errors.append(f"features contains duplicate feature ID {duplicate!r}")

    return sorted(set(errors))


def _pack_field(hasher: Any, value: bytes) -> None:
    hasher.update(len(value).to_bytes(8, "big"))
    hasher.update(value)


def canonical_tree_delta(
    before: dict[bytes, TreeEntry], after: dict[bytes, TreeEntry]
) -> tuple[str, set[bytes]]:
    changed = {
        path
        for path in set(before) | set(after)
        if before.get(path) != after.get(path)
    }
    hasher = hashlib.sha256()
    hasher.update(DELTA_FORMAT.encode("ascii") + b"\0")
    for path in sorted(changed):
        old = before.get(path)
        new = after.get(path)
        fields = (
            path,
            b"" if old is None else old.mode.encode("ascii"),
            b"" if old is None else old.object_type.encode("ascii"),
            b"" if old is None else old.oid.encode("ascii"),
            b"" if new is None else new.mode.encode("ascii"),
            b"" if new is None else new.object_type.encode("ascii"),
            b"" if new is None else new.oid.encode("ascii"),
        )
        for field_value in fields:
            _pack_field(hasher, field_value)
    return hasher.hexdigest(), changed


def canonical_series_sha256(
    baseline: str,
    baseline_tree: str,
    candidate: str,
    candidate_tree: str,
    records: Iterable[ComputedCommit],
) -> str:
    materialized = list(records)
    hasher = hashlib.sha256()
    hasher.update(SERIES_FORMAT.encode("ascii") + b"\0")
    for value in (
        baseline,
        baseline_tree,
        candidate,
        candidate_tree,
        str(len(materialized)),
    ):
        _pack_field(hasher, value.encode("ascii"))
    for record in materialized:
        for value in (
            record.commit,
            record.parent,
            record.tree,
            record.delta_sha256,
        ):
            _pack_field(hasher, value.encode("ascii"))
    return hasher.hexdigest()


def _display_path(path: bytes) -> str:
    return path.decode("utf-8", errors="backslashreplace")


def _safe_git_path(path: bytes) -> bool:
    return (
        bool(path)
        and not path.startswith(b"/")
        and b"\0" not in path
        and all(component not in {b"", b".", b".."} for component in path.split(b"/"))
        and all(component.lower() != b".git" for component in path.split(b"/"))
    )


def _index_entries(context: CheckContext) -> dict[bytes, TreeEntry]:
    payload = context.git("ls-files", "--stage", "-z").stdout
    entries: dict[bytes, TreeEntry] = {}
    records = payload.split(b"\0")
    if records and records[-1] == b"":
        records.pop()
    for record in records:
        try:
            metadata, path = record.split(b"\t", 1)
            mode_raw, oid_raw, stage_raw = metadata.split(b" ", 2)
            mode = mode_raw.decode("ascii")
            oid = oid_raw.decode("ascii")
            stage = stage_raw.decode("ascii")
        except (ValueError, UnicodeDecodeError) as error:
            raise GitError("git ls-files returned malformed index data") from error
        if not _safe_git_path(path) or path in entries:
            raise GitError("git ls-files returned an unsafe or duplicate index path")
        if stage != "0":
            raise GitError(
                f"index path {_display_path(path)!r} has unresolved stage {stage}"
            )
        if SHA1_RE.fullmatch(oid) is None:
            raise GitError(f"git ls-files returned an invalid object ID: {oid!r}")
        object_type = "commit" if mode == "160000" else "blob"
        entries[path] = TreeEntry(mode, object_type, oid)
    return entries


def _worktree_blob_oid(repo_root: Path, path: bytes, mode: str) -> tuple[str | None, str | None]:
    current = os.fsencode(repo_root)
    components = path.split(b"/")
    for component in components[:-1]:
        current += b"/" + component
        try:
            parent_metadata = os.lstat(current)
        except FileNotFoundError:
            return None, "has a missing parent directory in the worktree"
        except OSError as error:
            return None, f"has a parent that cannot be inspected with lstat: {error}"
        if stat.S_ISLNK(parent_metadata.st_mode):
            return None, "has a symlink parent directory in the worktree"
        if not stat.S_ISDIR(parent_metadata.st_mode):
            return None, "has a non-directory parent component in the worktree"

    filesystem_path = current + b"/" + components[-1]
    try:
        metadata = os.lstat(filesystem_path)
    except FileNotFoundError:
        return None, "is deleted from the worktree"
    except OSError as error:
        return None, f"cannot be inspected with lstat: {error}"

    if mode == "120000":
        if not stat.S_ISLNK(metadata.st_mode):
            return None, "is not the symlink recorded in the index"
        try:
            payload = os.readlink(filesystem_path)
        except OSError as error:
            return None, f"symlink target cannot be read: {error}"
        payload_bytes = payload if isinstance(payload, bytes) else os.fsencode(payload)
        hasher = hashlib.sha1(usedforsecurity=False)
        hasher.update(f"blob {len(payload_bytes)}\0".encode("ascii"))
        hasher.update(payload_bytes)
        return hasher.hexdigest(), None

    if mode not in REGULAR_GIT_MODES:
        return None, f"uses unsupported index mode {mode} for a clean-worktree proof"
    if not stat.S_ISREG(metadata.st_mode):
        return None, "is not the regular file recorded in the index"
    expected_executable = mode == "100755"
    actual_executable = bool(metadata.st_mode & 0o111)
    if actual_executable != expected_executable:
        return None, "has executable-mode drift from the index"

    hasher = hashlib.sha1(usedforsecurity=False)
    hasher.update(f"blob {metadata.st_size}\0".encode("ascii"))
    bytes_read = 0
    try:
        with open(filesystem_path, "rb") as worktree_file:
            while True:
                chunk = worktree_file.read(1024 * 1024)
                if not chunk:
                    break
                bytes_read += len(chunk)
                hasher.update(chunk)
    except OSError as error:
        return None, f"cannot be read: {error}"
    if bytes_read != metadata.st_size:
        return None, "changed size while the clean-worktree proof was reading it"
    return hasher.hexdigest(), None


def authenticate_clean_checkout(context: CheckContext, candidate_tree: str) -> list[str]:
    """Authenticate index/worktree cleanliness without diff drivers or filters."""

    errors: list[str] = []
    index_entries = _index_entries(context)
    candidate_entries = context.tree_map(candidate_tree)
    if index_entries != candidate_entries:
        changed = sorted(
            path
            for path in set(index_entries) | set(candidate_entries)
            if index_entries.get(path) != candidate_entries.get(path)
        )
        for path in changed:
            errors.append(
                f"index entry {_display_path(path)!r} differs from checked-out HEAD"
            )

    for path, entry in sorted(index_entries.items()):
        if entry.mode == "160000":
            errors.append(
                f"gitlink {_display_path(path)!r} cannot be authenticated by the raw clean-worktree proof"
            )
            continue
        actual_oid, error = _worktree_blob_oid(context.repo_root, path, entry.mode)
        if error is not None:
            errors.append(f"worktree path {_display_path(path)!r} {error}")
        elif actual_oid != entry.oid:
            errors.append(
                f"worktree path {_display_path(path)!r} content differs from the index"
            )

    untracked_payload = context.git(
        "ls-files", "--others", "--exclude-standard", "-z"
    ).stdout
    untracked_paths = untracked_payload.split(b"\0")
    if untracked_paths and untracked_paths[-1] == b"":
        untracked_paths.pop()
    for path in sorted(untracked_paths):
        if not _safe_git_path(path):
            errors.append("git ls-files returned an unsafe untracked path")
        else:
            errors.append(f"untracked worktree path {_display_path(path)!r}")
    return errors


def analyze_linear_history(
    context: CheckContext,
    candidate: str,
    expected_count: int,
    *,
    reject_net_zero: bool,
) -> HistoryAnalysis:
    analysis = HistoryAnalysis()
    reverse_chain: list[tuple[str, str]] = []
    current = candidate
    seen: set[str] = set()

    while current != context.baseline:
        if current in seen:
            analysis.errors.append("history contains a parent cycle")
            return analysis
        seen.add(current)
        if len(reverse_chain) > expected_count:
            analysis.errors.append(
                f"history exceeds manifest commit_count {expected_count} before reaching baseline"
            )
            return analysis
        _tree, parents = context.commit_metadata(current)
        if len(parents) != 1:
            if not parents:
                analysis.errors.append(
                    f"history reached root commit {current} before the pinned baseline"
                )
            else:
                analysis.errors.append(
                    f"history commit {current} has {len(parents)} parents; side-parent merges are forbidden"
                )
            return analysis
        parent = parents[0]
        reverse_chain.append((current, parent))
        current = parent

    reverse_chain.reverse()
    if len(reverse_chain) != expected_count:
        analysis.errors.append(
            f"history commit count mismatch: expected {expected_count}, got {len(reverse_chain)}"
        )

    previous_commit = context.baseline
    previous_tree = context.baseline_tree
    baseline_entries = context.tree_map(context.baseline_tree)
    previous_entries = baseline_entries
    touched_paths: set[bytes] = set()

    for commit, parent in reverse_chain:
        if parent != previous_commit:
            analysis.errors.append(
                f"history is not a contiguous linear series at commit {commit}"
            )
            return analysis
        tree = context.commit_tree(commit)
        if tree == previous_tree:
            analysis.errors.append(
                f"history commit {commit} is empty because its tree equals its parent tree"
            )
        current_entries = context.tree_map(tree)
        delta_sha256, changed = canonical_tree_delta(previous_entries, current_entries)
        touched_paths.update(changed)
        analysis.records.append(
            ComputedCommit(
                commit=commit,
                parent=parent,
                tree=tree,
                delta_sha256=delta_sha256,
            )
        )
        previous_commit = commit
        previous_tree = tree
        previous_entries = current_entries

    if reverse_chain and previous_commit != candidate:
        analysis.errors.append("history traversal did not end at the requested candidate")
        return analysis

    if reject_net_zero:
        final_entries = previous_entries
        net_zero_paths = sorted(
            path
            for path in touched_paths
            if baseline_entries.get(path) == final_entries.get(path)
        )
        for path in net_zero_paths:
            analysis.errors.append(
                "history contains net-zero intermediate churn for path "
                f"{_display_path(path)!r}"
            )

    candidate_tree = context.commit_tree(candidate)
    analysis.series_sha256 = canonical_series_sha256(
        context.baseline,
        context.baseline_tree,
        candidate,
        candidate_tree,
        analysis.records,
    )
    return analysis


def compare_attestation(
    label: str,
    attestation: dict[str, Any],
    analysis: HistoryAnalysis,
) -> list[str]:
    errors = [f"{label}: {error}" for error in analysis.errors]
    expected_records = attestation["commits"]
    actual_records = [record.manifest_record() for record in analysis.records]
    if actual_records != expected_records:
        maximum = max(len(actual_records), len(expected_records))
        for index in range(maximum):
            expected = expected_records[index] if index < len(expected_records) else None
            actual = actual_records[index] if index < len(actual_records) else None
            if expected != actual:
                errors.append(
                    f"{label}: ordered commit/tree/delta record {index} mismatch: "
                    f"expected {expected!r}, got {actual!r}"
                )
    if analysis.series_sha256 != attestation["series_sha256"]:
        errors.append(
            f"{label}: series_sha256 mismatch: expected {attestation['series_sha256']}, "
            f"got {analysis.series_sha256 or '<unavailable>'}"
        )
    return errors


def _git_object_exists(context: CheckContext, revision: str) -> tuple[bool, str]:
    result = context.git("cat-file", "-e", revision, check=False)
    if result.returncode == 0:
        return True, ""
    detail = result.stderr.decode("utf-8", errors="backslashreplace").strip()
    return False, detail or f"git exited with status {result.returncode}"


def check_patch_stack_objects(context: CheckContext) -> Outcome:
    patch_stack = context.document["patch_stack"]
    errors: list[str] = []
    for label in ("baseline", "frozen_tip"):
        expected = patch_stack[label]
        exists, detail = _git_object_exists(context, f"{expected['commit']}^{{commit}}")
        if not exists:
            errors.append(f"{label} commit is unavailable: {detail}")
            continue
        actual_tree = context.commit_tree(expected["commit"])
        if actual_tree != expected["tree"]:
            errors.append(
                f"{label} tree mismatch: expected {expected['tree']}, got {actual_tree}"
            )
    return Outcome(
        "baseline and frozen commit objects match their authoritative trees",
        sorted(errors),
    )


def check_patch_stack_history(context: CheckContext) -> Outcome:
    history = context.document["patch_stack"]["history"]
    errors: list[str] = []
    exact = history["exact"]
    exact_analysis = analyze_linear_history(
        context,
        context.frozen_tip,
        exact["commit_count"],
        reject_net_zero=True,
    )
    errors.extend(compare_attestation("exact frozen history", exact, exact_analysis))

    equivalents = history["thematic_equivalents"]
    for attestation in equivalents:
        candidate = attestation["candidate"]
        analysis = analyze_linear_history(
            context,
            candidate["commit"],
            attestation["commit_count"],
            reject_net_zero=True,
        )
        if context.commit_tree(candidate["commit"]) != candidate["tree"]:
            errors.append(
                f"thematic attestation {attestation['id']!r}: candidate tree does not match manifest"
            )
        errors.extend(
            compare_attestation(
                f"thematic attestation {attestation['id']!r}",
                attestation,
                analysis,
            )
        )
    return Outcome(
        f"exact history and {len(equivalents)} thematic history attestation(s) authenticated",
        sorted(set(errors)),
    )


def check_coverage_candidate(context: CheckContext) -> Outcome:
    """Require a raw, single-parent path from the candidate to an attested checkpoint."""

    checkpoints = {
        context.frozen_tip: "exact frozen checkpoint",
        **{
            attestation["candidate"]["commit"]: (
                f"thematic checkpoint {attestation['id']!r}"
            )
            for attestation in context.document["patch_stack"]["history"][
                "thematic_equivalents"
            ]
        },
    }
    current = context.coverage_candidate
    seen: set[str] = set()
    followup_count = 0
    errors: list[str] = []

    # Reading the tree up front proves that the resolved candidate is a usable
    # immutable commit/tree pair; feature and coverage checks reuse this cache.
    context.tree_map(context.coverage_tree)

    while current not in checkpoints:
        if current in seen:
            errors.append("coverage candidate history contains a parent cycle")
            break
        seen.add(current)
        if followup_count >= MAX_COVERAGE_COMMITS:
            errors.append(
                "coverage candidate history exceeds the maximum supported "
                f"post-checkpoint length of {MAX_COVERAGE_COMMITS} commits"
            )
            break
        _tree, parents = context.commit_metadata(current)
        if len(parents) != 1:
            if not parents:
                errors.append(
                    f"coverage candidate history reached root commit {current} "
                    "before an authenticated frozen or thematic checkpoint"
                )
            else:
                errors.append(
                    f"coverage candidate history commit {current} has {len(parents)} "
                    "parents; post-checkpoint merges are forbidden"
                )
            break
        current = parents[0]
        followup_count += 1

    if errors:
        return Outcome("coverage candidate lineage could not be authenticated", errors)

    checkpoint = checkpoints[current]
    return Outcome(
        f"committed candidate {context.coverage_candidate} descends linearly from "
        f"the {checkpoint} through {followup_count} post-checkpoint commit(s); "
        "checkpoint history is authenticated separately"
    )


def _ledger_revision(cell: str, project: str, label: str) -> str:
    matches = SHA1_SEARCH_RE.findall(cell)
    if not matches:
        raise ValueError(f"project {project!r} has no full {label} revision")
    if any(SHA1_RE.fullmatch(match) is None for match in matches):
        raise ValueError(
            f"project {project!r} {label} revision must use lowercase full 40-hex IDs"
        )
    unique = set(matches)
    if len(unique) != 1:
        raise ValueError(
            f"project {project!r} {label} cell contains inconsistent revision IDs"
        )
    return matches[0]


def parse_upstream_versions(path: Path) -> dict[str, UpstreamRecord]:
    try:
        text = path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as error:
        raise ValueError(f"cannot read {path}: {error}") from error

    records: dict[str, UpstreamRecord] = {}
    for line in text.splitlines():
        stripped = line.strip()
        if not stripped.startswith("|") or not stripped.endswith("|"):
            continue
        cells = [cell.strip() for cell in stripped[1:-1].split("|")]
        if len(cells) != 4 or cells[0] in {"Project", "---"}:
            continue
        project, remote_cell, reviewed_cell, latest_cell = cells
        if set(project) <= {"-", ":", " "}:
            continue
        if _contains_unsafe_unicode(project):
            raise ValueError(f"project name contains unsafe Unicode: {project!r}")
        remote_match = REMOTE_REF_CELL_RE.fullmatch(remote_cell)
        if remote_match is None:
            raise ValueError(
                f"project {project!r} remote/ref cell must contain exactly two code spans"
            )
        remote, tracked_ref = remote_match.groups()
        ledger_errors: list[str] = []
        _validate_remote(remote, f"project {project!r} remote", ledger_errors)
        _validate_tracked_ref(
            tracked_ref, f"project {project!r} tracked ref", ledger_errors
        )
        if ledger_errors:
            raise ValueError("; ".join(ledger_errors))
        reviewed = _ledger_revision(reviewed_cell, project, "reviewed")
        if latest_cell.strip().lower() == "same":
            latest = reviewed
        else:
            latest = _ledger_revision(latest_cell, project, "latest-fetched")
        if project in records:
            raise ValueError(f"duplicate upstream project row {project!r}")
        records[project] = UpstreamRecord(remote, tracked_ref, reviewed, latest)
    if not records:
        raise ValueError(f"no upstream project rows found in {path}")
    return records


def check_upstream_revisions(context: CheckContext) -> Outcome:
    errors: list[str] = []
    try:
        records = parse_upstream_versions(context.upstream_versions)
    except ValueError as error:
        return Outcome("upstream ledger could not be parsed", [str(error)])

    sources = {source["project"]: source for source in context.document["sources"]}
    for missing in sorted(set(records) - set(sources)):
        errors.append(f"manifest is missing upstream project {missing!r}")
    for extra in sorted(set(sources) - set(records)):
        errors.append(f"manifest project {extra!r} is absent from UPSTREAM_VERSIONS.md")
    for project in sorted(set(records) & set(sources)):
        record = records[project]
        source = sources[project]
        comparisons = (
            ("remote", source["remote"], record.remote),
            ("tracked ref", source["tracked_ref"], record.tracked_ref),
            ("reviewed revision", source["reviewed"]["commit"], record.reviewed),
            (
                "latest-fetched revision",
                source["latest_fetched"]["commit"],
                record.latest_fetched,
            ),
        )
        for label, manifest_value, ledger_value in comparisons:
            if manifest_value != ledger_value:
                errors.append(
                    f"{project} {label} mismatch: ledger {ledger_value!r}, "
                    f"manifest {manifest_value!r}"
                )

    grok_source = sources.get("Grok Build upstream")
    if grok_source is not None and (
        grok_source["reviewed"]["commit"] != context.baseline
    ):
        errors.append(
            "Grok Build upstream reviewed commit must equal patch_stack.baseline.commit"
        )
    return Outcome(
        f"{len(records)} upstream remote/ref/revision record(s) cross-checked; no external tree mappings claimed",
        sorted(errors),
    )


def _checkout_regular_file_status(repo_root: Path, relative_path: str) -> str | None:
    current = repo_root
    components = relative_path.split("/")
    for index, component in enumerate(components):
        current = current / component
        try:
            metadata = os.lstat(current)
        except FileNotFoundError:
            return "does not exist in the checkout"
        except OSError as error:
            return f"cannot be inspected with lstat: {error}"
        if stat.S_ISLNK(metadata.st_mode):
            return "is a symlink in the checkout"
        if index < len(components) - 1 and not stat.S_ISDIR(metadata.st_mode):
            return "has a non-directory parent component in the checkout"
    if not stat.S_ISREG(metadata.st_mode):
        return "is not a regular file in the checkout"
    return None


def _candidate_regular_file_status(
    candidate_entries: dict[bytes, TreeEntry],
    relative_path: str,
    candidate: str,
) -> str | None:
    entry = candidate_entries.get(relative_path.encode("utf-8"))
    if entry is None:
        return f"is absent at coverage candidate {candidate}"
    if entry.mode == "120000":
        return f"is a symlink at coverage candidate {candidate}"
    if entry.mode not in REGULAR_GIT_MODES or entry.object_type != "blob":
        return (
            f"is not a regular file at coverage candidate {candidate} "
            f"(mode {entry.mode}, type {entry.object_type})"
        )
    return None


def check_feature_paths(context: CheckContext) -> Outcome:
    changed_paths = context.changed_paths()
    errors: list[str] = []
    candidate_entries = context.tree_map(context.coverage_tree)
    candidate_cache: dict[str, str | None] = {}
    checkout_cache: dict[str, str | None] = {}
    candidate_is_checked_out = context.coverage_candidate == context.checked_out_head

    for feature in context.document["features"]:
        feature_id = feature["id"]
        compiled = [(pattern, compile_repo_glob(pattern)) for pattern in feature["paths"]]
        for pattern, matcher in compiled:
            if not any(matcher.fullmatch(path) for path in changed_paths):
                errors.append(
                    f"feature {feature_id!r} path pattern matches no downstream path: {pattern!r}"
                )

        for path in feature["integration_paths"]:
            if not any(matcher.fullmatch(path) for _pattern, matcher in compiled):
                errors.append(
                    f"feature {feature_id!r} integration path is not owned by its paths: {path!r}"
                )

        for kind in ("integration_paths", "legal_paths"):
            for path in feature[kind]:
                if path not in candidate_cache:
                    candidate_cache[path] = _candidate_regular_file_status(
                        candidate_entries, path, context.coverage_candidate
                    )
                candidate_error = candidate_cache[path]
                if candidate_error is not None:
                    errors.append(
                        f"feature {feature_id!r} {kind} entry {path!r} {candidate_error}"
                    )

                if candidate_is_checked_out:
                    if path not in checkout_cache:
                        checkout_cache[path] = _checkout_regular_file_status(
                            context.repo_root, path
                        )
                    checkout_error = checkout_cache[path]
                    if checkout_error is not None:
                        errors.append(
                            f"feature {feature_id!r} {kind} entry {path!r} {checkout_error}"
                        )

    return Outcome(
        f"{len(context.document['features'])} feature path set(s) and committed "
        f"regular-file anchors verified at candidate {context.coverage_candidate}",
        sorted(set(errors)),
    )


def check_downstream_coverage(context: CheckContext) -> Outcome:
    changed_paths = context.changed_paths()
    matchers = [
        compile_repo_glob(pattern)
        for feature in context.document["features"]
        for pattern in feature["paths"]
    ]
    uncovered = [
        path
        for path in changed_paths
        if not any(matcher.fullmatch(path) for matcher in matchers)
    ]
    uncovered.sort(key=lambda path: path.encode("utf-8"))
    covered_count = len(changed_paths) - len(uncovered)
    summary = (
        f"{covered_count}/{len(changed_paths)} baseline-to-candidate downstream "
        f"path(s) covered at {context.coverage_candidate}"
    )
    if not uncovered:
        return Outcome(summary, uncovered=[])
    message = f"{len(uncovered)} downstream path(s) remain uncovered by feature ownership"
    return Outcome(summary, errors=[message], uncovered=uncovered)


CHECKS: dict[str, Callable[[CheckContext], Outcome]] = {
    "patch-stack-objects": check_patch_stack_objects,
    "patch-stack-history": check_patch_stack_history,
    "coverage-candidate": check_coverage_candidate,
    "upstream-revisions": check_upstream_revisions,
    "feature-paths": check_feature_paths,
    "downstream-coverage": check_downstream_coverage,
}


def _checker_argument_parser() -> argparse.ArgumentParser:
    repository_default = Path(__file__).resolve().parents[2]
    parser = argparse.ArgumentParser(
        description="Validate the downstream compatibility manifest using read-only checks."
    )
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=repository_default,
        help="repository worktree root (default: inferred from this script)",
    )
    parser.add_argument(
        "--manifest",
        type=Path,
        help="manifest path (default: <repo-root>/fork/manifest.json)",
    )
    parser.add_argument(
        "--upstream-versions",
        type=Path,
        help="upstream ledger path (default: <repo-root>/UPSTREAM_VERSIONS.md)",
    )
    parser.add_argument(
        "--coverage-candidate",
        help=(
            "full commit OID or safe literal ref, resolved once "
            "(default: checked-out committed HEAD)"
        ),
    )
    parser.add_argument(
        "--strict-coverage",
        action="store_true",
        help="assert the manifest's mandatory fail-closed coverage policy",
    )
    return parser


def _load_checked_document(manifest_path: Path) -> tuple[dict[str, Any] | None, list[str]]:
    try:
        document = load_manifest(manifest_path)
    except ValueError as error:
        return None, [str(error)]
    schema_errors = validate_schema(document)
    if schema_errors:
        return None, schema_errors
    return document, []


def checker_main(arguments: list[str]) -> int:
    options = _checker_argument_parser().parse_args(arguments)
    repo_root = options.repo_root.resolve()
    manifest_path = (
        options.manifest.resolve()
        if options.manifest is not None
        else repo_root / "fork" / "manifest.json"
    )
    upstream_versions = (
        options.upstream_versions.resolve()
        if options.upstream_versions is not None
        else repo_root / "UPSTREAM_VERSIONS.md"
    )

    document, load_errors = _load_checked_document(manifest_path)
    if document is None:
        for error in load_errors:
            print(f"ERROR schema: {error}")
        print(f"manifest validation failed with {len(load_errors)} schema error(s)")
        return 1

    candidate_ref = options.coverage_candidate or "HEAD"
    if not _safe_coverage_candidate(candidate_ref):
        print(f"ERROR coverage-candidate: unsafe candidate revision: {candidate_ref!r}")
        return 2

    try:
        context = CheckContext(
            document=document,
            repo_root=repo_root,
            upstream_versions=upstream_versions,
            strict_coverage=options.strict_coverage,
        )
        checked_out_head = context.resolve_commit("HEAD")
        candidate_commit = _resolve_coverage_candidate(
            context, candidate_ref, checked_out_head
        )
        context.set_coverage_candidate(candidate_commit, checked_out_head)
    except GitError as error:
        print(f"ERROR coverage-candidate: candidate does not resolve to a commit: {error}")
        return 1

    error_count = 0
    for check_id in document["checks"]:
        checker = CHECKS[check_id]
        try:
            outcome = checker(context)
        except (GitError, OSError, ValueError) as error:
            outcome = Outcome("check could not complete", [str(error)])
        if outcome.errors:
            for error in sorted(outcome.errors):
                print(f"ERROR {check_id}: {error}")
            error_count += len(outcome.errors)
        else:
            print(f"OK {check_id}: {outcome.summary}")
        for warning in sorted(outcome.warnings):
            print(f"WARNING {check_id}: {warning}")
        for path in outcome.uncovered:
            print(f"UNCOVERED\t{path}")

    if error_count:
        print(f"manifest validation failed with {error_count} check error(s)")
        return 1
    print("manifest validation succeeded")
    return 0


def _safe_candidate_ref(value: str) -> bool:
    return (
        bool(value)
        and not _contains_unsafe_unicode(value)
        and SAFE_REF_RE.fullmatch(value) is not None
        and ".." not in value
        and "//" not in value
        and "@{" not in value
        and not value.endswith(("/", ".", ".lock"))
        and all(not component.startswith(".") for component in value.split("/"))
    )


def _safe_coverage_candidate(value: str) -> bool:
    if not _safe_candidate_ref(value):
        return False
    if SHA1_RE.fullmatch(value) is not None:
        return True
    return HEX_OBJECT_NAME_RE.fullmatch(value) is None


def _resolve_coverage_candidate(
    context: CheckContext, value: str, checked_out_head: str
) -> str:
    if value == "HEAD":
        return checked_out_head
    if SHA1_RE.fullmatch(value) is not None:
        return context.resolve_commit(value)

    result = context.git(
        "rev-parse",
        "--verify",
        "--symbolic-full-name",
        value,
        check=False,
    )
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", errors="backslashreplace").strip()
        raise GitError(detail or f"candidate ref {value!r} does not exist")
    try:
        symbolic_names = result.stdout.decode("ascii").splitlines()
    except UnicodeDecodeError as error:
        raise GitError("candidate ref resolved to a non-ASCII name") from error
    if (
        len(symbolic_names) != 1
        or not symbolic_names[0].startswith("refs/")
        or not _safe_candidate_ref(symbolic_names[0])
    ):
        raise GitError(
            f"coverage candidate {value!r} is not a full OID or unambiguous literal ref"
        )
    return context.resolve_commit(symbolic_names[0])


def _verifier_argument_parser() -> argparse.ArgumentParser:
    repository_default = Path(__file__).resolve().parents[2]
    parser = argparse.ArgumentParser(
        prog="fork/scripts/verify_patch_stack.sh",
        description=(
            "Authenticate an exact frozen commit or an explicitly attested "
            "thematic-equivalent linear history using read-only Git plumbing."
        ),
        epilog=(
            "The candidate defaults to the frozen tip. This verifier never fetches, "
            "creates or moves refs, checks out, resets, cleans, stashes, rebases, "
            "commits, or pushes. Range-diff output is diagnostic and never proof."
        ),
    )
    parser.add_argument("--repo-root", type=Path, default=repository_default)
    parser.add_argument("--manifest", type=Path)
    parser.add_argument(
        "--candidate",
        help="safe literal ref or full commit; resolved exactly once (default: frozen commit)",
    )
    parser.add_argument(
        "--history-mode",
        choices=("exact", "thematic-equivalence"),
        default="exact",
    )
    parser.add_argument(
        "--attestation",
        help="manifest thematic attestation ID; required in thematic-equivalence mode",
    )
    parser.add_argument("--skip-clean-check", action="store_true")
    parser.add_argument("--no-range-diff", action="store_true")
    return parser


def _print_verifier_failure(message: str, failures: list[str]) -> None:
    failures.append(message)
    print(f"FAIL: {message}", file=sys.stderr)


def _external_transform_config(context: CheckContext) -> list[str]:
    """Return configured external transforms that make range-diff unsafe."""

    result = context.git(
        "config",
        "--includes",
        "--name-only",
        "-z",
        "--list",
        check=False,
    )
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", errors="backslashreplace").strip()
        raise GitError(detail or "git config could not inspect external transforms")
    raw_keys = result.stdout.split(b"\0")
    if raw_keys and raw_keys[-1] == b"":
        raw_keys.pop()
    external: set[str] = set()
    for raw_key in raw_keys:
        key = raw_key.decode("utf-8", errors="backslashreplace")
        normalized = key.casefold()
        is_filter_command = normalized.startswith("filter.") and normalized.endswith(
            (".clean", ".smudge", ".process")
        )
        is_diff_command = normalized == "diff.external" or (
            normalized.startswith("diff.")
            and normalized.endswith((".command", ".textconv"))
        )
        if is_filter_command or is_diff_command:
            external.add(key)
    return sorted(external)


def verifier_main(arguments: list[str]) -> int:
    options = _verifier_argument_parser().parse_args(arguments)
    repo_root = options.repo_root.resolve()
    manifest_path = (
        options.manifest.resolve()
        if options.manifest is not None
        else repo_root / "fork" / "manifest.json"
    )
    document, load_errors = _load_checked_document(manifest_path)
    if document is None:
        for error in load_errors:
            print(f"ERROR schema: {error}", file=sys.stderr)
        return 2

    try:
        context = CheckContext(
            document=document,
            repo_root=repo_root,
            upstream_versions=repo_root / "UPSTREAM_VERSIONS.md",
            strict_coverage=True,
        )
    except GitError as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 2

    candidate_ref = options.candidate or context.frozen_tip
    if not _safe_candidate_ref(candidate_ref):
        print(f"ERROR: unsafe candidate revision: {candidate_ref!r}", file=sys.stderr)
        return 2
    try:
        candidate_commit = context.resolve_commit(candidate_ref)
    except GitError as error:
        print(f"ERROR: candidate does not resolve to a commit: {error}", file=sys.stderr)
        return 2

    print(f"Repository:   {repo_root}")
    print(f"Baseline:     {context.baseline}")
    print(f"Frozen tip:   {context.frozen_tip}")
    print(f"Candidate:    {candidate_commit}")
    print(f"History mode: {options.history_mode}")

    failures: list[str] = []
    for checker in (check_patch_stack_objects, check_patch_stack_history):
        try:
            outcome = checker(context)
        except (GitError, OSError, ValueError) as error:
            _print_verifier_failure(f"manifest proof prerequisite failed: {error}", failures)
            continue
        for error in outcome.errors:
            _print_verifier_failure(error, failures)
        if not outcome.errors:
            print(f"PASS: {outcome.summary}")

    selected_attestation: dict[str, Any] | None = None
    if options.history_mode == "exact":
        if options.attestation is not None:
            print("ERROR: --attestation is only valid in thematic-equivalence mode", file=sys.stderr)
            return 2
        if candidate_commit != context.frozen_tip:
            _print_verifier_failure(
                "exact mode requires candidate commit to equal the manifest frozen commit",
                failures,
            )
        else:
            print("PASS: exact candidate commit identity equals the frozen commit")
    else:
        if options.attestation is None:
            print(
                "ERROR: --attestation is required in thematic-equivalence mode",
                file=sys.stderr,
            )
            return 2
        for attestation in document["patch_stack"]["history"]["thematic_equivalents"]:
            if attestation["id"] == options.attestation:
                selected_attestation = attestation
                break
        if selected_attestation is None:
            print(
                f"ERROR: unknown thematic attestation ID: {options.attestation!r}",
                file=sys.stderr,
            )
            return 2
        pinned_commit = selected_attestation["candidate"]["commit"]
        if candidate_commit != pinned_commit:
            _print_verifier_failure(
                f"candidate commit does not equal thematic attestation {options.attestation!r}",
                failures,
            )
        else:
            print(
                f"PASS: candidate commit equals thematic attestation {options.attestation!r}"
            )

    try:
        candidate_tree = context.commit_tree(candidate_commit)
        if candidate_tree != context.frozen_tree:
            _print_verifier_failure(
                f"candidate tree mismatch: expected {context.frozen_tree}, got {candidate_tree}",
                failures,
            )
        else:
            print(f"PASS: candidate tree is exactly {context.frozen_tree}")
    except GitError as error:
        _print_verifier_failure(f"candidate tree could not be read: {error}", failures)

    if options.skip_clean_check:
        print("INFO: clean-status check explicitly skipped")
    else:
        try:
            bare = context.git("rev-parse", "--is-bare-repository").stdout.strip()
            if bare == b"true":
                print("INFO: clean-status check is not applicable to a bare repository")
            else:
                head = context.resolve_commit("HEAD")
                if candidate_commit == head:
                    clean_errors = authenticate_clean_checkout(
                        context, context.commit_tree(candidate_commit)
                    )
                    if clean_errors:
                        _print_verifier_failure(
                            "candidate is checked-out HEAD but the raw index/worktree proof is not clean",
                            failures,
                        )
                        for clean_error in clean_errors:
                            _print_verifier_failure(clean_error, failures)
                    else:
                        print(
                            "PASS: candidate is checked-out HEAD and the raw index/worktree proof is clean"
                        )
                else:
                    print(
                        "INFO: clean-status check is not applicable because candidate is not checked-out HEAD"
                    )
        except GitError as error:
            _print_verifier_failure(f"clean-status check failed: {error}", failures)

    if not options.no_range_diff:
        print("--- range-diff diagnostics (non-authoritative; never proof) ---")
        try:
            external_transforms = _external_transform_config(context)
            if external_transforms:
                print(
                    "INFO: range-diff diagnostics suppressed rather than execute configured "
                    "external Git transforms: " + ", ".join(external_transforms)
                )
            else:
                status_code = context.git_passthrough(
                    "range-diff",
                    "--no-ext-diff",
                    "--no-textconv",
                    f"{context.baseline}..{context.frozen_tip}",
                    f"{context.baseline}..{candidate_commit}",
                )
                if status_code != 0:
                    print(
                        "INFO: range-diff diagnostics were unavailable; immutable proof checks remain authoritative."
                    )
        except GitError as error:
            print(f"INFO: range-diff diagnostics were unavailable: {error}")
        print("--- end range-diff diagnostics ---")

    if failures:
        print(
            f"Patch-stack verification failed with {len(failures)} check failure(s).",
            file=sys.stderr,
        )
        return 1
    print("Patch-stack verification succeeded.")
    return 0


def main(arguments: list[str] | None = None) -> int:
    actual_arguments = list(sys.argv[1:] if arguments is None else arguments)
    if actual_arguments and actual_arguments[0] == "__verify_patch_stack__":
        return verifier_main(actual_arguments[1:])
    return checker_main(actual_arguments)


if __name__ == "__main__":
    raise SystemExit(main())
