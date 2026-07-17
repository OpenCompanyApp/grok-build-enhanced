#!/usr/bin/env python3
"""Deterministically vendor the audited Warp theme corpus from a local Git repo.

The tool never fetches from a remote. ``--check`` validates the checked-in
corpus without a source checkout, ``--plan`` compares a local Git archive with
that corpus without writing, and only ``--apply`` replaces the destination.
Theme YAML is validated as UTF-8 but copied byte-for-byte from ``git archive``.
"""

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
import tarfile
import tempfile
import unicodedata
from collections.abc import Iterable, Mapping
from dataclasses import dataclass
from io import BytesIO
from pathlib import Path, PurePosixPath
from typing import Any

REPOSITORY_ROOT = Path(__file__).absolute().parents[2]
DEFAULT_DESTINATION = (
    REPOSITORY_ROOT
    / "crates"
    / "codegen"
    / "xai-grok-pager-render"
    / "assets"
    / "warp-themes"
)
UPSTREAM_SOURCE = "https://github.com/warpdotdev/themes.git"
CATEGORIES = (
    "base16",
    "standard",
    "special_edition",
    "stradicat",
    "warp_bundled",
)
# YAML used by upstream repository tooling is outside the theme corpus. Keep
# this ignore list explicit so a newly introduced theme category cannot be
# silently omitted during an audit.
IGNORED_SOURCE_YAML_ROOTS = frozenset({".github", ".trunk", ".warp", "scripts"})
LOCAL_ROOT_FILES = frozenset(
    {"LICENSE", "README.md", "UPSTREAM_REVISION", "VENDOR_MANIFEST.json"}
)
MANIFEST_SCHEMA_VERSION = 1
REVISION_RE = re.compile(r"[0-9a-f]{40}\Z")
OBJECT_ID_RE = re.compile(r"[0-9a-f]{40}\Z")
SHA256_RE = re.compile(r"[0-9a-f]{64}\Z")
WINDOWS_ABSOLUTE_RE = re.compile(r"[A-Za-z]:")
PORTABLE_COMPONENT_RE = re.compile(r"[A-Za-z0-9._-]+\Z")
WINDOWS_RESERVED_NAMES = frozenset(
    {"CON", "PRN", "AUX", "NUL"}
    | {f"COM{index}" for index in range(1, 10)}
    | {f"LPT{index}" for index in range(1, 10)}
)
WINDOWS_RESERVED_CHARACTERS = frozenset('<>:"|?*')
BIDI_CONTROL_CODEPOINTS = frozenset(
    {0x061C, 0x200E, 0x200F, *range(0x202A, 0x202F), *range(0x2066, 0x206A)}
)
MAX_PORTABLE_COMPONENT_BYTES = 255
MAX_PORTABLE_PATH_BYTES = 4_096
MAX_THEME_BYTES = 1 * 1024 * 1024
MAX_LICENSE_BYTES = 256 * 1024
MAX_README_BYTES = 256 * 1024
MAX_MANIFEST_BYTES = 4 * 1024 * 1024
MAX_THEME_COUNT = 5_000
MAX_TOTAL_SOURCE_BYTES = 32 * 1024 * 1024
MAX_ARCHIVE_MEMBERS = MAX_THEME_COUNT + len(CATEGORIES) + 8


class VendorError(Exception):
    """A validation or deterministic-vendoring failure."""


@dataclass(frozen=True)
class TreeEntry:
    path: str
    size: int
    object_id: str


@dataclass(frozen=True)
class Corpus:
    revision: str
    themes: Mapping[str, bytes]
    license_bytes: bytes


@dataclass(frozen=True)
class DestinationState:
    files: Mapping[str, bytes]
    themes: Mapping[str, bytes]


def _validate_revision(revision: str, *, label: str = "revision") -> str:
    if not REVISION_RE.fullmatch(revision):
        raise VendorError(
            f"{label} must be an exact 40-character lowercase hexadecimal commit ID"
        )
    return revision


def _validate_utf8(data: bytes, *, label: str) -> None:
    try:
        data.decode("utf-8", errors="strict")
    except UnicodeDecodeError as error:
        raise VendorError(f"{label} is not valid UTF-8: {error}") from error


def _validate_portable_component(component: str, *, label: str) -> None:
    if component in {"", ".", ".."}:
        raise VendorError(f"{label} contains an empty or traversal component: {component!r}")
    if unicodedata.normalize("NFC", component) != component:
        raise VendorError(f"{label} component is not Unicode NFC: {component!r}")
    if any(ord(character) in BIDI_CONTROL_CODEPOINTS for character in component):
        raise VendorError(f"{label} component contains a bidi control: {component!r}")
    if any(ord(character) < 32 or ord(character) == 127 for character in component):
        raise VendorError(f"{label} component contains a control character: {component!r}")
    if any(character in WINDOWS_RESERVED_CHARACTERS for character in component):
        raise VendorError(
            f"{label} component contains a Windows-reserved character: {component!r}"
        )
    if component.endswith((".", " ")):
        raise VendorError(
            f"{label} component has a Windows-unsafe trailing dot or space: {component!r}"
        )
    windows_stem = component.split(".", 1)[0].upper()
    if windows_stem in WINDOWS_RESERVED_NAMES:
        raise VendorError(
            f"{label} component uses a Windows-reserved device name: {component!r}"
        )
    if not PORTABLE_COMPONENT_RE.fullmatch(component):
        raise VendorError(
            f"{label} component must use only portable ASCII letters, digits, '.', '_', "
            f"or '-': {component!r}"
        )
    if len(component.encode("ascii")) > MAX_PORTABLE_COMPONENT_BYTES:
        raise VendorError(
            f"{label} component exceeds {MAX_PORTABLE_COMPONENT_BYTES} bytes: {component!r}"
        )


def _safe_relative_path(raw_path: bytes | str, *, label: str) -> str:
    if isinstance(raw_path, bytes):
        try:
            path = raw_path.decode("utf-8", errors="strict")
        except UnicodeDecodeError as error:
            raise VendorError(f"{label} contains a non-UTF-8 path") from error
    else:
        path = raw_path
        try:
            path.encode("utf-8", errors="strict")
        except UnicodeEncodeError as error:
            raise VendorError(f"{label} contains a non-UTF-8 path") from error

    if not path:
        raise VendorError(f"{label} contains an empty path")
    if "\\" in path:
        raise VendorError(f"{label} path uses a backslash: {path!r}")
    if path.startswith("/") or WINDOWS_ABSOLUTE_RE.match(path):
        raise VendorError(f"{label} contains an absolute path: {path!r}")
    if len(path.encode("utf-8")) > MAX_PORTABLE_PATH_BYTES:
        raise VendorError(f"{label} path exceeds {MAX_PORTABLE_PATH_BYTES} bytes: {path!r}")

    pure = PurePosixPath(path)
    parts = path.split("/")
    if pure.is_absolute() or any(part in {"", ".", ".."} for part in parts):
        raise VendorError(f"{label} contains path traversal: {path!r}")
    for part in parts:
        _validate_portable_component(part, label=label)
    return path


def _collision_key(path: str) -> str:
    return unicodedata.normalize("NFC", path).casefold()


def _reject_case_collisions(paths: Iterable[str], *, label: str) -> None:
    seen: dict[str, str] = {}
    for path in paths:
        key = _collision_key(path)
        previous = seen.get(key)
        if previous is not None and previous != path:
            raise VendorError(
                f"{label} contains a case-insensitive path collision: "
                f"{previous!r} and {path!r}"
            )
        seen[key] = path


def _theme_path_parts(path: str, *, label: str) -> tuple[str, str]:
    parts = path.split("/")
    suffix = PurePosixPath(path).suffix
    if len(parts) == 1 and suffix.lower() in {".yaml", ".yml"}:
        raise VendorError(f"{label} rejects a YAML file at the corpus root: {path!r}")
    if suffix.lower() in {".yaml", ".yml"} and suffix not in {".yaml", ".yml"}:
        raise VendorError(f"{label} requires a lowercase YAML/YML extension: {path!r}")
    if len(parts) != 2 or parts[0] not in CATEGORIES or suffix not in {".yaml", ".yml"}:
        raise VendorError(
            f"{label} theme path must be one YAML/YML file directly below an "
            f"allowlisted category: {path!r}"
        )
    if not PurePosixPath(parts[1]).stem:
        raise VendorError(f"{label} theme filename has an empty stem: {path!r}")
    return parts[0], PurePosixPath(parts[1]).stem


def _run_git(source_repo: Path, arguments: list[str], *, binary: bool = True) -> bytes | str:
    environment = os.environ.copy()
    environment.update(
        {
            "GIT_TERMINAL_PROMPT": "0",
            "GIT_OPTIONAL_LOCKS": "0",
            # A partial clone may otherwise contact its promisor remote while
            # `ls-tree -l` resolves blob sizes. Vendoring must stay offline.
            "GIT_NO_LAZY_FETCH": "1",
            # Replacement refs are mutable local state. Both the environment
            # and global option pin every source command to the repository's
            # actual object graph rather than refs/replace/* overlays.
            "GIT_NO_REPLACE_OBJECTS": "1",
            "LC_ALL": "C",
        }
    )
    command = ["git", "--no-replace-objects", "-C", os.fspath(source_repo), *arguments]
    try:
        completed = subprocess.run(
            command,
            check=False,
            capture_output=True,
            timeout=60,
            env=environment,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise VendorError(f"failed to run local git command: {error}") from error
    if completed.returncode != 0:
        stderr = completed.stderr.decode("utf-8", errors="replace").strip()
        raise VendorError(
            f"local git command failed ({' '.join(arguments[:2])}): "
            f"{stderr or f'exit {completed.returncode}'}"
        )
    if binary:
        return completed.stdout
    try:
        return completed.stdout.decode("ascii", errors="strict")
    except UnicodeDecodeError as error:
        raise VendorError("local git command returned non-ASCII metadata") from error


def _verify_local_commit(source_repo: Path, revision: str) -> None:
    if not source_repo.exists():
        raise VendorError(f"source repository does not exist: {source_repo}")
    if not source_repo.is_dir():
        raise VendorError(f"source repository is not a directory: {source_repo}")
    resolved = _run_git(
        source_repo,
        ["rev-parse", "--verify", f"{revision}^{{commit}}"],
        binary=False,
    ).strip()
    if resolved != revision:
        raise VendorError(
            f"source repository resolved {revision} to unexpected commit {resolved!r}"
        )


def _parse_source_tree(source_repo: Path, revision: str) -> tuple[list[TreeEntry], TreeEntry]:
    raw_tree = _run_git(
        source_repo,
        ["ls-tree", "-rlz", "--full-tree", revision],
    )
    assert isinstance(raw_tree, bytes)

    themes: list[TreeEntry] = []
    license_entry: TreeEntry | None = None
    source_paths: set[str] = set()
    logical_theme_ids: dict[str, str] = {}
    total_size = 0

    for raw_record in raw_tree.split(b"\0"):
        if not raw_record:
            continue
        try:
            raw_metadata, raw_path = raw_record.split(b"\t", 1)
            mode, object_type, raw_object_id, raw_size = raw_metadata.split()
        except ValueError as error:
            raise VendorError("git ls-tree returned malformed metadata") from error
        try:
            object_id = raw_object_id.decode("ascii", errors="strict")
        except UnicodeDecodeError as error:
            raise VendorError("git ls-tree returned a non-ASCII object ID") from error
        if not OBJECT_ID_RE.fullmatch(object_id):
            raise VendorError(
                f"git ls-tree returned a non-SHA-1 object ID: {object_id!r}"
            )
        path = _safe_relative_path(raw_path, label="source tree")
        if path in source_paths:
            raise VendorError(f"source tree contains a duplicate path: {path!r}")
        source_paths.add(path)

        is_regular = object_type == b"blob" and mode == b"100644"
        is_symlink = object_type == b"blob" and mode == b"120000"
        try:
            size = int(raw_size) if raw_size != b"-" else -1
        except ValueError as error:
            raise VendorError(f"source tree has an invalid size for {path!r}") from error

        parts = path.split("/")
        in_category = parts[0] in CATEGORIES
        suffix = PurePosixPath(path).suffix
        yaml_like = suffix.lower() in {".yaml", ".yml"}

        if path == "LICENSE":
            if license_entry is not None:
                raise VendorError("source tree contains more than one LICENSE entry")
            if not is_regular:
                kind = "symlink" if is_symlink else "special or executable entry"
                raise VendorError(f"source LICENSE must be a regular 0644 file, not a {kind}")
            if size <= 0 or size > MAX_LICENSE_BYTES:
                raise VendorError(
                    f"source LICENSE size {size} is outside the allowed range "
                    f"1..{MAX_LICENSE_BYTES}"
                )
            license_entry = TreeEntry(path=path, size=size, object_id=object_id)
            total_size += size
            continue

        if yaml_like:
            if len(parts) == 1:
                raise VendorError(
                    f"source tree rejects a YAML file at the repository root: {path!r}"
                )
            if not in_category:
                if parts[0] in IGNORED_SOURCE_YAML_ROOTS:
                    continue
                raise VendorError(
                    f"source tree contains YAML in a non-allowlisted category: {path!r}"
                )
            category, stem = _theme_path_parts(path, label="source tree")
            if not is_regular:
                kind = "symlink" if is_symlink else "special or executable entry"
                raise VendorError(f"source theme {path!r} must be a regular 0644 file, not a {kind}")
            if size <= 0 or size > MAX_THEME_BYTES:
                raise VendorError(
                    f"source theme {path!r} size {size} is outside the allowed range "
                    f"1..{MAX_THEME_BYTES}"
                )
            logical_id = _collision_key(f"{category}/{stem}")
            previous = logical_theme_ids.get(logical_id)
            if previous is not None:
                raise VendorError(
                    f"source themes map to the same case-insensitive theme ID: "
                    f"{previous!r} and {path!r}"
                )
            logical_theme_ids[logical_id] = path
            themes.append(TreeEntry(path=path, size=size, object_id=object_id))
            total_size += size
            continue

        # Preview SVGs, images, category READMEs, and all unrelated repository
        # files are deliberately not selected. A symlink/special entry inside
        # an allowlisted category is still rejected rather than silently
        # skipped, so it can never become an extraction or packaging hazard.
        if in_category and not is_regular:
            kind = "symlink" if is_symlink else "special or executable entry"
            raise VendorError(
                f"source category contains a rejected {kind}: {path!r}"
            )

    if license_entry is None:
        raise VendorError("source tree is missing the root LICENSE file")
    if not themes:
        raise VendorError("source tree contains no allowlisted theme YAML")
    if len(themes) > MAX_THEME_COUNT:
        raise VendorError(
            f"source tree has {len(themes)} themes; limit is {MAX_THEME_COUNT}"
        )
    if total_size > MAX_TOTAL_SOURCE_BYTES:
        raise VendorError(
            f"selected source data is {total_size} bytes; limit is {MAX_TOTAL_SOURCE_BYTES}"
        )

    themes.sort(key=lambda entry: entry.path)
    _reject_case_collisions(
        [license_entry.path, *(entry.path for entry in themes)],
        label="selected source tree",
    )
    selected_categories = {entry.path.split("/", 1)[0] for entry in themes}
    missing_categories = set(CATEGORIES) - selected_categories
    if missing_categories:
        raise VendorError(
            "source tree is missing allowlisted categories: "
            + ", ".join(sorted(missing_categories))
        )
    return themes, license_entry


def _read_git_archive(
    source_repo: Path,
    revision: str,
    themes: list[TreeEntry],
    license_entry: TreeEntry,
) -> Corpus:
    selected = [license_entry, *themes]
    archive_bytes = _run_git(
        source_repo,
        ["archive", "--format=tar", revision, "--", *(entry.path for entry in selected)],
    )
    assert isinstance(archive_bytes, bytes)

    expected = {entry.path: entry for entry in selected}
    extracted: dict[str, bytes] = {}
    seen_members: set[str] = set()
    try:
        archive = tarfile.open(fileobj=BytesIO(archive_bytes), mode="r:")
    except tarfile.TarError as error:
        raise VendorError(f"git archive returned an invalid tar stream: {error}") from error

    with archive:
        members = archive.getmembers()
        if len(members) > MAX_ARCHIVE_MEMBERS:
            raise VendorError(
                f"git archive has {len(members)} members; limit is {MAX_ARCHIVE_MEMBERS}"
            )
        for member in members:
            member_path = _safe_relative_path(member.name, label="git archive")
            if member_path in seen_members:
                raise VendorError(f"git archive contains a duplicate member: {member_path!r}")
            seen_members.add(member_path)

            if member.isdir():
                if member_path not in CATEGORIES:
                    raise VendorError(
                        f"git archive contains a rejected directory: {member_path!r}"
                    )
                continue
            if not member.isreg():
                raise VendorError(
                    f"git archive contains a symlink or special member: {member_path!r}"
                )
            expected_entry = expected.get(member_path)
            if expected_entry is None:
                if "/" not in member_path:
                    raise VendorError(
                        f"git archive contains a rejected root file: {member_path!r}"
                    )
                raise VendorError(
                    f"git archive contains a file outside the strict allowlist: {member_path!r}"
                )
            # Git records the selected blobs as non-executable 100644 files.
            # `git archive` deliberately applies its tar umask and may expose
            # those as 0644 or 0664, so reject executable/special members here
            # without treating harmless group-write normalization as content.
            if member.mode & 0o7000 or member.mode & 0o111:
                raise VendorError(
                    f"git archive member {member_path!r} has rejected mode "
                    f"{member.mode & 0o7777:o}"
                )
            if member.size != expected_entry.size:
                raise VendorError(
                    f"git archive size for {member_path!r} is {member.size}; "
                    f"git tree reported {expected_entry.size}"
                )
            extracted_file = archive.extractfile(member)
            if extracted_file is None:
                raise VendorError(f"git archive could not read {member_path!r}")
            data = extracted_file.read(expected_entry.size + 1)
            if len(data) != expected_entry.size:
                raise VendorError(
                    f"git archive returned {len(data)} bytes for {member_path!r}; "
                    f"expected {expected_entry.size}"
                )
            actual_object_id = _git_blob_object_id(data)
            if actual_object_id != expected_entry.object_id:
                raise VendorError(
                    f"git archive bytes for {member_path!r} hash to Git blob "
                    f"{actual_object_id}; no-replace ls-tree recorded "
                    f"{expected_entry.object_id}"
                )
            _validate_utf8(data, label=f"git archive member {member_path!r}")
            extracted[member_path] = data

    missing = sorted(set(expected) - set(extracted))
    if missing:
        raise VendorError("git archive omitted selected files: " + ", ".join(missing))
    _reject_case_collisions(extracted, label="git archive")
    return Corpus(
        revision=revision,
        themes={entry.path: extracted[entry.path] for entry in themes},
        license_bytes=extracted[license_entry.path],
    )


def _load_source_corpus(source_repo: Path, revision: str) -> Corpus:
    revision = _validate_revision(revision)
    _verify_local_commit(source_repo, revision)
    themes, license_entry = _parse_source_tree(source_repo, revision)
    return _read_git_archive(source_repo, revision, themes, license_entry)


def _git_blob_object_id(data: bytes) -> str:
    header = f"blob {len(data)}\0".encode("ascii")
    return hashlib.sha1(header + data, usedforsecurity=False).hexdigest()


def _sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _manifest_object(corpus: Corpus) -> dict[str, Any]:
    category_counts = {category: 0 for category in CATEGORIES}
    files: list[dict[str, Any]] = []
    for path in sorted(corpus.themes):
        category, _stem = _theme_path_parts(path, label="manifest")
        data = corpus.themes[path]
        category_counts[category] += 1
        files.append(
            {
                "path": path,
                "bytes": len(data),
                "sha256": _sha256(data),
            }
        )
    return {
        "schema_version": MANIFEST_SCHEMA_VERSION,
        "source": UPSTREAM_SOURCE,
        "revision": corpus.revision,
        "theme_count": len(files),
        "category_counts": category_counts,
        "license": {
            "path": "LICENSE",
            "bytes": len(corpus.license_bytes),
            "sha256": _sha256(corpus.license_bytes),
        },
        "files": files,
    }


def _manifest_bytes(corpus: Corpus) -> bytes:
    return (
        json.dumps(_manifest_object(corpus), ensure_ascii=False, indent=2) + "\n"
    ).encode("utf-8")


def _reject_duplicate_json_pairs(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise VendorError(f"vendor manifest contains duplicate JSON key {key!r}")
        result[key] = value
    return result


def _require_exact_keys(value: Mapping[str, Any], expected: set[str], *, label: str) -> None:
    actual = set(value)
    if actual != expected:
        missing = sorted(expected - actual)
        extra = sorted(actual - expected)
        details: list[str] = []
        if missing:
            details.append("missing " + ", ".join(missing))
        if extra:
            details.append("unexpected " + ", ".join(extra))
        raise VendorError(f"{label} keys are invalid ({'; '.join(details)})")


def _require_integer(value: Any, *, label: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise VendorError(f"{label} must be a non-negative integer")
    return value


def _validate_hash_record(record: Any, *, label: str) -> tuple[str, int, str]:
    if not isinstance(record, dict):
        raise VendorError(f"{label} must be a JSON object")
    _require_exact_keys(record, {"path", "bytes", "sha256"}, label=label)
    path = record["path"]
    digest = record["sha256"]
    if not isinstance(path, str):
        raise VendorError(f"{label}.path must be a string")
    path = _safe_relative_path(path, label=label)
    byte_count = _require_integer(record["bytes"], label=f"{label}.bytes")
    if not isinstance(digest, str) or not SHA256_RE.fullmatch(digest):
        raise VendorError(f"{label}.sha256 must be 64 lowercase hexadecimal characters")
    return path, byte_count, digest


def _parse_manifest(data: bytes) -> dict[str, Any]:
    if len(data) > MAX_MANIFEST_BYTES:
        raise VendorError(
            f"vendor manifest is {len(data)} bytes; limit is {MAX_MANIFEST_BYTES}"
        )
    _validate_utf8(data, label="vendor manifest")
    try:
        parsed = json.loads(
            data.decode("utf-8"),
            object_pairs_hook=_reject_duplicate_json_pairs,
        )
    except VendorError:
        raise
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise VendorError(f"vendor manifest is not valid JSON: {error}") from error
    if not isinstance(parsed, dict):
        raise VendorError("vendor manifest root must be a JSON object")
    _require_exact_keys(
        parsed,
        {
            "schema_version",
            "source",
            "revision",
            "theme_count",
            "category_counts",
            "license",
            "files",
        },
        label="vendor manifest",
    )
    if parsed["schema_version"] != MANIFEST_SCHEMA_VERSION:
        raise VendorError(
            f"vendor manifest schema_version must be {MANIFEST_SCHEMA_VERSION}"
        )
    if parsed["source"] != UPSTREAM_SOURCE:
        raise VendorError(
            f"vendor manifest source must be {UPSTREAM_SOURCE!r}, got {parsed['source']!r}"
        )
    if not isinstance(parsed["revision"], str):
        raise VendorError("vendor manifest revision must be a string")
    _validate_revision(parsed["revision"], label="vendor manifest revision")
    _require_integer(parsed["theme_count"], label="vendor manifest theme_count")

    category_counts = parsed["category_counts"]
    if not isinstance(category_counts, dict):
        raise VendorError("vendor manifest category_counts must be a JSON object")
    _require_exact_keys(category_counts, set(CATEGORIES), label="vendor manifest category_counts")
    for category in CATEGORIES:
        _require_integer(
            category_counts[category],
            label=f"vendor manifest category_counts.{category}",
        )

    license_path, _license_bytes, _license_hash = _validate_hash_record(
        parsed["license"], label="vendor manifest license"
    )
    if license_path != "LICENSE":
        raise VendorError("vendor manifest license path must be exactly 'LICENSE'")

    records = parsed["files"]
    if not isinstance(records, list):
        raise VendorError("vendor manifest files must be a JSON array")
    record_paths: list[str] = []
    logical_ids: set[str] = set()
    for index, record in enumerate(records):
        path, byte_count, _digest = _validate_hash_record(
            record, label=f"vendor manifest files[{index}]"
        )
        category, stem = _theme_path_parts(path, label="vendor manifest")
        if byte_count <= 0 or byte_count > MAX_THEME_BYTES:
            raise VendorError(
                f"vendor manifest size for {path!r} is outside 1..{MAX_THEME_BYTES}"
            )
        logical_id = _collision_key(f"{category}/{stem}")
        if logical_id in logical_ids:
            raise VendorError(f"vendor manifest contains a duplicate theme ID for {path!r}")
        logical_ids.add(logical_id)
        record_paths.append(path)
    if record_paths != sorted(record_paths):
        raise VendorError("vendor manifest file records must be sorted by path")
    if len(set(record_paths)) != len(record_paths):
        raise VendorError("vendor manifest contains duplicate file paths")
    _reject_case_collisions(record_paths, label="vendor manifest")
    return parsed


def _read_regular_file(path: Path, *, limit: int, label: str) -> bytes:
    try:
        metadata = path.lstat()
    except OSError as error:
        raise VendorError(f"failed to inspect {label} {path}: {error}") from error
    if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISREG(metadata.st_mode):
        raise VendorError(f"{label} must be a regular file, not a symlink or special file: {path}")
    if metadata.st_size > limit:
        raise VendorError(f"{label} is {metadata.st_size} bytes; limit is {limit}: {path}")
    try:
        data = path.read_bytes()
    except OSError as error:
        raise VendorError(f"failed to read {label} {path}: {error}") from error
    if len(data) != metadata.st_size:
        raise VendorError(f"{label} changed while it was being read: {path}")
    return data


def _inspect_destination(destination: Path, *, require_complete: bool) -> DestinationState:
    try:
        root_metadata = destination.lstat()
    except OSError as error:
        raise VendorError(f"failed to inspect destination {destination}: {error}") from error
    if stat.S_ISLNK(root_metadata.st_mode) or not stat.S_ISDIR(root_metadata.st_mode):
        raise VendorError(f"destination must be a real directory, not a symlink: {destination}")

    files: dict[str, bytes] = {}
    themes: dict[str, bytes] = {}
    observed_paths: list[str] = []
    observed_categories: set[str] = set()

    try:
        root_entries = sorted(os.scandir(destination), key=lambda entry: entry.name)
    except OSError as error:
        raise VendorError(f"failed to list destination {destination}: {error}") from error

    for entry in root_entries:
        name = _safe_relative_path(entry.name, label="destination")
        observed_paths.append(name)
        try:
            entry_metadata = entry.stat(follow_symlinks=False)
        except OSError as error:
            raise VendorError(f"failed to inspect destination entry {entry.path}: {error}") from error
        if stat.S_ISLNK(entry_metadata.st_mode):
            raise VendorError(f"destination rejects symlink {name!r}")

        if stat.S_ISDIR(entry_metadata.st_mode):
            if name not in CATEGORIES:
                raise VendorError(f"destination rejects non-allowlisted directory {name!r}")
            observed_categories.add(name)
            try:
                category_entries = sorted(os.scandir(entry.path), key=lambda item: item.name)
            except OSError as error:
                raise VendorError(f"failed to list destination category {entry.path}: {error}") from error
            for theme_entry in category_entries:
                relative = _safe_relative_path(
                    f"{name}/{theme_entry.name}", label="destination"
                )
                _theme_path_parts(relative, label="destination")
                observed_paths.append(relative)
                try:
                    theme_metadata = theme_entry.stat(follow_symlinks=False)
                except OSError as error:
                    raise VendorError(
                        f"failed to inspect destination theme {theme_entry.path}: {error}"
                    ) from error
                if stat.S_ISLNK(theme_metadata.st_mode) or not stat.S_ISREG(theme_metadata.st_mode):
                    raise VendorError(
                        f"destination theme must be a regular file, not a symlink, "
                        f"directory, or special file: {relative!r}"
                    )
                data = _read_regular_file(
                    Path(theme_entry.path),
                    limit=MAX_THEME_BYTES,
                    label="destination theme",
                )
                if not data:
                    raise VendorError(f"destination theme is empty: {relative!r}")
                _validate_utf8(data, label=f"destination theme {relative!r}")
                themes[relative] = data
                files[relative] = data
            continue

        if not stat.S_ISREG(entry_metadata.st_mode):
            raise VendorError(f"destination rejects special root entry {name!r}")
        if name not in LOCAL_ROOT_FILES:
            if PurePosixPath(name).suffix.lower() in {".yaml", ".yml"}:
                raise VendorError(f"destination rejects a root YAML file: {name!r}")
            raise VendorError(f"destination rejects non-allowlisted root file {name!r}")
        limit = {
            "LICENSE": MAX_LICENSE_BYTES,
            "README.md": MAX_README_BYTES,
            "UPSTREAM_REVISION": 256,
            "VENDOR_MANIFEST.json": MAX_MANIFEST_BYTES,
        }[name]
        data = _read_regular_file(destination / name, limit=limit, label="destination root file")
        _validate_utf8(data, label=f"destination root file {name!r}")
        files[name] = data

    _reject_case_collisions(observed_paths, label="destination")
    logical_ids: dict[str, str] = {}
    for path in themes:
        category, stem = _theme_path_parts(path, label="destination")
        key = _collision_key(f"{category}/{stem}")
        previous = logical_ids.get(key)
        if previous is not None:
            raise VendorError(
                f"destination themes map to the same case-insensitive theme ID: "
                f"{previous!r} and {path!r}"
            )
        logical_ids[key] = path

    if len(themes) > MAX_THEME_COUNT:
        raise VendorError(
            f"destination has {len(themes)} themes; limit is {MAX_THEME_COUNT}"
        )
    if sum(map(len, themes.values())) > MAX_TOTAL_SOURCE_BYTES:
        raise VendorError(
            f"destination theme bytes exceed {MAX_TOTAL_SOURCE_BYTES}"
        )

    if require_complete:
        missing_root = LOCAL_ROOT_FILES - set(files)
        if missing_root:
            raise VendorError(
                "destination is missing required root files: "
                + ", ".join(sorted(missing_root))
            )
        missing_categories = set(CATEGORIES) - observed_categories
        if missing_categories:
            raise VendorError(
                "destination is missing categories: "
                + ", ".join(sorted(missing_categories))
            )
    return DestinationState(files=files, themes=themes)


def _validate_checked_destination(
    destination: Path, *, expected_revision: str | None = None
) -> Corpus:
    state = _inspect_destination(destination, require_complete=True)
    manifest_bytes = state.files["VENDOR_MANIFEST.json"]
    manifest = _parse_manifest(manifest_bytes)
    revision = manifest["revision"]
    if expected_revision is not None and revision != expected_revision:
        raise VendorError(
            f"destination revision {revision} does not match requested revision "
            f"{expected_revision}"
        )
    expected_marker = f"{revision}\n".encode("ascii")
    if state.files["UPSTREAM_REVISION"] != expected_marker:
        raise VendorError(
            "UPSTREAM_REVISION must contain exactly the manifest revision and one newline"
        )

    corpus = Corpus(
        revision=revision,
        themes=state.themes,
        license_bytes=state.files["LICENSE"],
    )
    expected_manifest = _manifest_bytes(corpus)
    if manifest_bytes != expected_manifest:
        raise VendorError(
            "VENDOR_MANIFEST.json is not the canonical manifest for the checked-in bytes"
        )
    return corpus


def _candidate_files(corpus: Corpus, readme_bytes: bytes) -> dict[str, bytes]:
    _validate_utf8(readme_bytes, label="preserved README.md")
    if len(readme_bytes) > MAX_README_BYTES:
        raise VendorError(
            f"preserved README.md is {len(readme_bytes)} bytes; limit is {MAX_README_BYTES}"
        )
    result = dict(corpus.themes)
    result.update(
        {
            "LICENSE": corpus.license_bytes,
            "README.md": readme_bytes,
            "UPSTREAM_REVISION": f"{corpus.revision}\n".encode("ascii"),
            "VENDOR_MANIFEST.json": _manifest_bytes(corpus),
        }
    )
    return result


def _diff_files(
    current: Mapping[str, bytes], candidate: Mapping[str, bytes]
) -> tuple[list[str], list[str], list[str]]:
    current_paths = set(current)
    candidate_paths = set(candidate)
    added = sorted(candidate_paths - current_paths)
    removed = sorted(current_paths - candidate_paths)
    changed = sorted(
        path
        for path in current_paths & candidate_paths
        if current[path] != candidate[path]
    )
    return added, changed, removed


def _print_plan(
    destination: Path,
    revision: str,
    added: list[str],
    changed: list[str],
    removed: list[str],
) -> None:
    print(f"Warp themes plan for {destination}")
    print(f"  source:   {UPSTREAM_SOURCE}")
    print(f"  revision: {revision}")
    print(f"  changes:  {len(added)} add, {len(changed)} update, {len(removed)} remove")
    for action, paths in (("ADD", added), ("UPDATE", changed), ("REMOVE", removed)):
        for path in paths:
            print(f"  {action:<6} {path}")


def _write_staging_tree(staging: Path, files: Mapping[str, bytes]) -> None:
    for category in CATEGORIES:
        (staging / category).mkdir(mode=0o755)
    for relative, data in sorted(files.items()):
        safe = _safe_relative_path(relative, label="staging output")
        output = staging.joinpath(*safe.split("/"))
        output.parent.mkdir(mode=0o755, parents=True, exist_ok=True)
        try:
            with output.open("xb") as handle:
                handle.write(data)
                handle.flush()
                os.fsync(handle.fileno())
            output.chmod(0o644)
        except OSError as error:
            raise VendorError(f"failed to write staging file {output}: {error}") from error


def _atomic_replace(destination: Path, candidate: Mapping[str, bytes], revision: str) -> None:
    parent = destination.parent
    try:
        parent_metadata = parent.lstat()
    except OSError as error:
        raise VendorError(f"failed to inspect destination parent {parent}: {error}") from error
    if stat.S_ISLNK(parent_metadata.st_mode) or not stat.S_ISDIR(parent_metadata.st_mode):
        raise VendorError(f"destination parent must be a real directory: {parent}")

    staging = Path(tempfile.mkdtemp(prefix=f".{destination.name}.tmp-", dir=parent))
    backup: Path | None = None
    try:
        _write_staging_tree(staging, candidate)
        _validate_checked_destination(staging, expected_revision=revision)

        backup = Path(tempfile.mkdtemp(prefix=f".{destination.name}.backup-", dir=parent))
        backup.rmdir()
        try:
            os.replace(destination, backup)
            try:
                os.replace(staging, destination)
            except BaseException as install_error:
                try:
                    os.replace(backup, destination)
                except BaseException as rollback_error:
                    # Never delete the only preserved copy of the old corpus if
                    # rollback itself fails. Surface its exact location for
                    # manual recovery and exempt it from finally cleanup.
                    preserved_backup = backup
                    backup = None
                    raise VendorError(
                        f"failed to install {destination} and could not restore it; "
                        f"the previous corpus remains at {preserved_backup}: "
                        f"{rollback_error}"
                    ) from install_error
                backup = None
                raise
        except OSError as error:
            raise VendorError(f"failed to atomically replace {destination}: {error}") from error
        shutil.rmtree(backup)
        backup = None
    finally:
        if staging.exists():
            shutil.rmtree(staging, ignore_errors=True)
        if backup is not None and backup.exists():
            # If replacement succeeded but cleanup failed, never remove the new
            # destination. The backup contains only the pre-apply corpus.
            shutil.rmtree(backup, ignore_errors=True)


def _prepare_plan(
    destination: Path, source_repo: Path, revision: str
) -> tuple[Corpus, dict[str, bytes], DestinationState, tuple[list[str], list[str], list[str]]]:
    current = _inspect_destination(destination, require_complete=False)
    readme = current.files.get("README.md")
    if readme is None:
        raise VendorError(
            "destination must already contain the human-maintained README.md; "
            "the vendor tool does not create or rewrite legal/attribution text"
        )
    corpus = _load_source_corpus(source_repo, revision)
    candidate = _candidate_files(corpus, readme)
    changes = _diff_files(current.files, candidate)
    return corpus, candidate, current, changes


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Vendor Warp themes deterministically from a local Git repository; "
            "this tool performs no network operations."
        )
    )
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument(
        "--check",
        action="store_true",
        help="validate the checked-in manifest and corpus without a source checkout",
    )
    mode.add_argument(
        "--plan",
        action="store_true",
        help="read a local git archive and report changes without writing",
    )
    mode.add_argument(
        "--apply",
        action="store_true",
        help="read a local git archive and atomically replace the destination",
    )
    parser.add_argument(
        "--source-repo",
        type=Path,
        help="local warpdotdev/themes Git repository (required for --plan/--apply)",
    )
    parser.add_argument(
        "--revision",
        help=(
            "exact 40-character lowercase commit ID (required for --plan/--apply; "
            "optional expected revision for --check)"
        ),
    )
    parser.add_argument(
        "--destination",
        type=Path,
        default=DEFAULT_DESTINATION,
        help=f"vendored corpus destination (default: {DEFAULT_DESTINATION})",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = _build_parser()
    arguments = parser.parse_args(argv)
    destination = Path(os.path.abspath(os.fspath(arguments.destination)))

    try:
        if arguments.check:
            if arguments.source_repo is not None:
                raise VendorError("--source-repo is not used with offline --check")
            expected_revision = None
            if arguments.revision is not None:
                expected_revision = _validate_revision(arguments.revision)
            corpus = _validate_checked_destination(
                destination, expected_revision=expected_revision
            )
            counts = _manifest_object(corpus)["category_counts"]
            count_text = ", ".join(
                f"{category}={counts[category]}" for category in CATEGORIES
            )
            print(
                f"Warp themes check passed: {len(corpus.themes)} themes "
                f"({count_text}), revision {corpus.revision}"
            )
            return 0

        if arguments.source_repo is None:
            raise VendorError("--source-repo is required with --plan and --apply")
        if arguments.revision is None:
            raise VendorError("--revision is required with --plan and --apply")
        revision = _validate_revision(arguments.revision)
        source_repo = Path(os.path.abspath(os.fspath(arguments.source_repo)))
        corpus, candidate, _current, changes = _prepare_plan(
            destination, source_repo, revision
        )
        added, changed, removed = changes
        _print_plan(destination, revision, added, changed, removed)
        if arguments.plan:
            print("Plan is read-only; no files were written.")
            return 0

        if not any(changes):
            print("Apply skipped; destination already matches the local git archive.")
            return 0
        _atomic_replace(destination, candidate, corpus.revision)
        print(
            f"Applied {len(corpus.themes)} byte-preserved themes from "
            f"{corpus.revision}."
        )
        return 0
    except (VendorError, OSError) as error:
        print(f"vendor_warp_themes.py: error: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
