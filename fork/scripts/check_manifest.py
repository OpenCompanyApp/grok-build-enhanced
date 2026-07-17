#!/usr/bin/env python3
"""Validate fork/manifest.json without changing the repository or its refs."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
import unicodedata
from dataclasses import dataclass, field
from pathlib import Path, PurePosixPath
from typing import Any, Callable
from urllib.parse import urlsplit

sys.dont_write_bytecode = True

SHA1_RE = re.compile(r"^[0-9a-f]{40}$")
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
ID_RE = re.compile(r"^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$")
SAFE_REF_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._/-]*$")
SHA1_SEARCH_RE = re.compile(r"(?<![0-9A-Fa-f])([0-9A-Fa-f]{40})(?![0-9A-Fa-f])")
GLOB_META = frozenset("*?[]{}")

KNOWN_CHECK_IDS = (
    "patch-stack-objects",
    "patch-stack-fingerprints",
    "upstream-revisions",
    "feature-paths",
    "downstream-coverage",
)


class DuplicateKeyError(ValueError):
    """Raised when JSON contains an ambiguous duplicate object key."""


class GitError(RuntimeError):
    """Raised when a read-only Git query cannot be completed."""


@dataclass
class Outcome:
    summary: str
    errors: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)
    uncovered: list[str] = field(default_factory=list)


@dataclass
class CheckContext:
    document: dict[str, Any]
    repo_root: Path
    upstream_versions: Path
    strict_coverage: bool
    _changed_paths: list[str] | None = None

    @property
    def baseline(self) -> str:
        return self.document["patch_stack"]["baseline"]["commit"]

    @property
    def frozen_tip(self) -> str:
        return self.document["patch_stack"]["frozen_tip"]["commit"]

    def git(self, *arguments: str, check: bool = True) -> subprocess.CompletedProcess[bytes]:
        env = os.environ.copy()
        env["GIT_OPTIONAL_LOCKS"] = "0"
        env["GIT_NO_LAZY_FETCH"] = "1"
        env["GIT_NO_REPLACE_OBJECTS"] = "1"
        command = [
            "git",
            "--no-pager",
            "-c",
            "core.quotePath=false",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "submodule.recurse=false",
            "-C",
            str(self.repo_root),
            *arguments,
        ]
        try:
            result = subprocess.run(
                command,
                check=False,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                env=env,
            )
        except OSError as error:
            raise GitError(f"unable to run git: {error}") from error
        if check and result.returncode != 0:
            detail = result.stderr.decode("utf-8", errors="replace").strip()
            if not detail:
                detail = f"git exited with status {result.returncode}"
            raise GitError(detail)
        return result

    def changed_paths(self) -> list[str]:
        if self._changed_paths is None:
            payload = self.git(
                "diff",
                "--name-only",
                "--no-renames",
                "--no-ext-diff",
                "--no-textconv",
                "-z",
                self.baseline,
                self.frozen_tip,
                "--",
            ).stdout
            raw_paths = payload.split(b"\0")
            if raw_paths and raw_paths[-1] == b"":
                raw_paths.pop()
            try:
                decoded = [raw.decode("utf-8") for raw in raw_paths]
            except UnicodeDecodeError as error:
                raise GitError(
                    "the downstream diff contains a non-UTF-8 path that cannot be represented in JSON"
                ) from error
            self._changed_paths = sorted(set(decoded), key=lambda path: path.encode("utf-8"))
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
        errors.append(f"{location} must be a lowercase 64-hex SHA-256 digest")
        return None
    return string


def _validate_id(value: Any, location: str, errors: list[str]) -> str | None:
    string = _string(value, location, errors)
    if string is not None and ID_RE.fullmatch(string) is None:
        errors.append(
            f"{location} must use lowercase kebab-case beginning with a letter"
        )
        return None
    return string


def validate_repo_path(value: Any, *, allow_glob: bool) -> str | None:
    """Return a path-safety error, or None when the repository path is safe."""

    if not isinstance(value, str) or not value:
        return "must be a non-empty string"
    if value != value.strip():
        return "must not have leading or trailing whitespace"
    if any(unicodedata.category(character) in {"Cc", "Cf"} for character in value):
        return "must not contain control or formatting characters"
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
    """Compile the checker's documented, slash-aware repository glob syntax."""

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
        if isinstance(version, bool) or not isinstance(version, int) or version != 1:
            errors.append("schema_version must be the integer 1")

    patch_stack = _as_object(document.get("patch_stack"), "patch_stack", errors)
    if patch_stack is not None:
        allowed = {"baseline", "frozen_tip", "fingerprints"}
        _unknown_keys(patch_stack, allowed, "patch_stack", errors)
        _require_keys(patch_stack, allowed, "patch_stack", errors)
        if "baseline" in patch_stack:
            _validate_revision_object(patch_stack["baseline"], "patch_stack.baseline", errors)
        if "frozen_tip" in patch_stack:
            _validate_revision_object(patch_stack["frozen_tip"], "patch_stack.frozen_tip", errors)
        if "fingerprints" in patch_stack:
            fingerprints = _as_object(
                patch_stack["fingerprints"], "patch_stack.fingerprints", errors
            )
            if fingerprints is not None:
                fingerprint_keys = {
                    "algorithm",
                    "diff_sha256",
                    "name_status_sha256",
                    "numstat_sha256",
                }
                _unknown_keys(
                    fingerprints, fingerprint_keys, "patch_stack.fingerprints", errors
                )
                _require_keys(
                    fingerprints, fingerprint_keys, "patch_stack.fingerprints", errors
                )
                if fingerprints.get("algorithm") != "sha256":
                    errors.append("patch_stack.fingerprints.algorithm must be 'sha256'")
                for key in (
                    "diff_sha256",
                    "name_status_sha256",
                    "numstat_sha256",
                ):
                    if key in fingerprints:
                        _validate_sha256(
                            fingerprints[key], f"patch_stack.fingerprints.{key}", errors
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
                _validate_tracked_ref(
                    source["tracked_ref"], f"{location}.tracked_ref", errors
                )
            if "reviewed" in source:
                _validate_revision_object(source["reviewed"], f"{location}.reviewed", errors)
            if "latest_fetched" in source:
                _validate_revision_object(
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
        if "allow_uncovered" in coverage and not isinstance(
            coverage["allow_uncovered"], bool
        ):
            errors.append("coverage.allow_uncovered must be a boolean")

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


def _git_object_exists(context: CheckContext, revision: str) -> tuple[bool, str]:
    result = context.git("cat-file", "-e", revision, check=False)
    if result.returncode == 0:
        return True, ""
    detail = result.stderr.decode("utf-8", errors="replace").strip()
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
        actual_tree = context.git(
            "rev-parse", "--verify", f"{expected['commit']}^{{tree}}"
        ).stdout.decode("ascii").strip()
        if actual_tree != expected["tree"]:
            errors.append(
                f"{label} tree mismatch: expected {expected['tree']}, got {actual_tree}"
            )
    ancestry = context.git(
        "merge-base",
        "--is-ancestor",
        context.baseline,
        context.frozen_tip,
        check=False,
    )
    if ancestry.returncode == 1:
        errors.append("patch_stack.baseline is not an ancestor of patch_stack.frozen_tip")
    elif ancestry.returncode != 0:
        detail = ancestry.stderr.decode("utf-8", errors="replace").strip()
        errors.append(f"could not check patch-stack ancestry: {detail}")
    return Outcome("baseline, frozen tip, trees, and ancestry verified", sorted(errors))


def _diff_payload(context: CheckContext, kind: str) -> bytes:
    common = ["--no-renames", "--no-ext-diff", "--no-textconv"]
    if kind == "diff":
        arguments = ["diff", "--binary", "--full-index", *common]
    elif kind == "name_status":
        arguments = ["diff", "--name-status", *common, "-z"]
    elif kind == "numstat":
        arguments = ["diff", "--numstat", *common, "-z"]
    else:
        raise AssertionError(f"unsupported fingerprint kind: {kind}")
    return context.git(
        *arguments, context.baseline, context.frozen_tip, "--"
    ).stdout


def check_patch_stack_fingerprints(context: CheckContext) -> Outcome:
    expected = context.document["patch_stack"]["fingerprints"]
    keys = {
        "diff": "diff_sha256",
        "name_status": "name_status_sha256",
        "numstat": "numstat_sha256",
    }
    errors: list[str] = []
    for kind, manifest_key in keys.items():
        actual = hashlib.sha256(_diff_payload(context, kind)).hexdigest()
        if actual != expected[manifest_key]:
            errors.append(
                f"{manifest_key} mismatch: expected {expected[manifest_key]}, got {actual}"
            )
    return Outcome("canonical diff fingerprints verified", sorted(errors))


def parse_upstream_versions(path: Path) -> dict[str, tuple[str, str]]:
    try:
        text = path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as error:
        raise ValueError(f"cannot read {path}: {error}") from error

    records: dict[str, tuple[str, str]] = {}
    for line in text.splitlines():
        stripped = line.strip()
        if not stripped.startswith("|") or not stripped.endswith("|"):
            continue
        cells = [cell.strip() for cell in stripped[1:-1].split("|")]
        if len(cells) != 4 or cells[0] in {"Project", "---"}:
            continue
        project, _remote, reviewed_cell, latest_cell = cells
        if set(project) <= {"-", ":", " "}:
            continue
        reviewed_matches = SHA1_SEARCH_RE.findall(reviewed_cell)
        if not reviewed_matches:
            continue
        reviewed = reviewed_matches[0].lower()
        latest_matches = SHA1_SEARCH_RE.findall(latest_cell)
        if latest_matches:
            latest = latest_matches[0].lower()
        elif latest_cell.strip().lower() == "same":
            latest = reviewed
        else:
            raise ValueError(
                f"project {project!r} has no full latest-fetched SHA or 'same' marker"
            )
        if project in records:
            raise ValueError(f"duplicate upstream project row {project!r}")
        records[project] = (reviewed, latest)
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
        reviewed, latest = records[project]
        source = sources[project]
        if source["reviewed"]["commit"] != reviewed:
            errors.append(
                f"{project} reviewed revision mismatch: ledger {reviewed}, "
                f"manifest {source['reviewed']['commit']}"
            )
        if source["latest_fetched"]["commit"] != latest:
            errors.append(
                f"{project} latest-fetched revision mismatch: ledger {latest}, "
                f"manifest {source['latest_fetched']['commit']}"
            )

    grok_source = sources.get("Grok Build upstream")
    if grok_source is not None:
        baseline = context.document["patch_stack"]["baseline"]
        if grok_source["reviewed"] != baseline:
            errors.append(
                "Grok Build upstream reviewed commit/tree must equal patch_stack.baseline"
            )
    return Outcome(
        f"{len(records)} upstream project revision pair(s) cross-checked",
        sorted(errors),
    )


def _checkout_file_status(repo_root: Path, relative_path: str) -> str | None:
    candidate = repo_root.joinpath(*relative_path.split("/"))
    try:
        resolved_root = repo_root.resolve(strict=True)
        resolved_candidate = candidate.resolve(strict=True)
        resolved_candidate.relative_to(resolved_root)
    except FileNotFoundError:
        return "does not exist in the checkout"
    except (OSError, ValueError):
        return "resolves outside the repository checkout"
    if not resolved_candidate.is_file():
        return "is not a file in the checkout"
    return None


def check_feature_paths(context: CheckContext) -> Outcome:
    changed_paths = context.changed_paths()
    errors: list[str] = []
    frozen_cache: dict[str, tuple[bool, str]] = {}
    checkout_cache: dict[str, str | None] = {}

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
                if path not in checkout_cache:
                    checkout_cache[path] = _checkout_file_status(context.repo_root, path)
                checkout_error = checkout_cache[path]
                if checkout_error is not None:
                    errors.append(
                        f"feature {feature_id!r} {kind} entry {path!r} {checkout_error}"
                    )

                if path not in frozen_cache:
                    frozen_cache[path] = _git_object_exists(
                        context, f"{context.frozen_tip}:{path}"
                    )
                exists, detail = frozen_cache[path]
                if not exists:
                    errors.append(
                        f"feature {feature_id!r} {kind} entry {path!r} is absent at "
                        f"the frozen tip: {detail}"
                    )

    return Outcome(
        f"{len(context.document['features'])} feature path set(s) and anchors verified",
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
    summary = f"{covered_count}/{len(changed_paths)} downstream path(s) covered"
    if not uncovered:
        return Outcome(summary, uncovered=[])

    message = (
        f"{len(uncovered)} downstream path(s) remain uncovered by feature ownership"
    )
    allow_uncovered = context.document["coverage"]["allow_uncovered"]
    if context.strict_coverage or not allow_uncovered:
        return Outcome(summary, errors=[message], uncovered=uncovered)
    return Outcome(summary, warnings=[message], uncovered=uncovered)


CHECKS: dict[str, Callable[[CheckContext], Outcome]] = {
    "patch-stack-objects": check_patch_stack_objects,
    "patch-stack-fingerprints": check_patch_stack_fingerprints,
    "upstream-revisions": check_upstream_revisions,
    "feature-paths": check_feature_paths,
    "downstream-coverage": check_downstream_coverage,
}


def _argument_parser() -> argparse.ArgumentParser:
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
        "--strict-coverage",
        action="store_true",
        help="treat uncovered downstream paths as errors even when the manifest allows them",
    )
    return parser


def main(arguments: list[str] | None = None) -> int:
    parser = _argument_parser()
    options = parser.parse_args(arguments)
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

    try:
        document = load_manifest(manifest_path)
    except ValueError as error:
        print(f"ERROR manifest: {error}")
        return 1

    schema_errors = validate_schema(document)
    if schema_errors:
        for error in schema_errors:
            print(f"ERROR schema: {error}")
        print(f"manifest validation failed with {len(schema_errors)} schema error(s)")
        return 1

    context = CheckContext(
        document=document,
        repo_root=repo_root,
        upstream_versions=upstream_versions,
        strict_coverage=options.strict_coverage,
    )
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


if __name__ == "__main__":
    raise SystemExit(main())
