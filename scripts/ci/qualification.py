#!/usr/bin/env python3
"""Create and verify exact-source GitHub Actions qualification records."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

SCHEMA_VERSION = 1
REPOSITORY = "OpenCompanyApp/grok-build-enhanced"
DEFAULT_WORKFLOW = ".github/workflows/rebase-qualification.yml"
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
TIMESTAMP_RE = re.compile(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$")
RESULT_NAMES = ("static", "rust_contracts", "broad", "native")
ALLOWED_RESULTS = {"success"}
ALLOWED_TARGETS = {
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
}


class QualificationError(ValueError):
    """Raised when qualification evidence is incomplete or inconsistent."""


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def git_output(root: Path, *args: str) -> str:
    process = subprocess.run(
        ["git", "--no-replace-objects", *args],
        cwd=root,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if process.returncode != 0:
        raise QualificationError(f"git {' '.join(args)} failed")
    return process.stdout.strip()


def rustc_identity() -> dict[str, str]:
    process = subprocess.run(
        ["rustc", "-vV"],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if process.returncode != 0:
        raise QualificationError("rustc -vV failed")
    lines = process.stdout.splitlines()
    values: dict[str, str] = {"rustc": lines[0]} if lines else {}
    for line in lines[1:]:
        key, separator, value = line.partition(": ")
        if separator and key in {"commit-hash", "commit-date", "host", "release", "LLVM version"}:
            values[key.replace(" ", "_")] = value
    required = {"rustc", "commit-hash", "host", "release"}
    if not required.issubset(values):
        raise QualificationError("rustc identity is incomplete")
    return values


def parse_results(entries: list[str]) -> dict[str, str]:
    parsed: dict[str, str] = {}
    for entry in entries:
        name, separator, value = entry.partition("=")
        if not separator or name not in RESULT_NAMES or value not in ALLOWED_RESULTS:
            raise QualificationError(f"invalid qualification result: {entry!r}")
        if name in parsed:
            raise QualificationError(f"duplicate qualification result: {name}")
        parsed[name] = value
    missing = sorted(set(RESULT_NAMES) - set(parsed))
    if missing:
        raise QualificationError(f"missing qualification results: {','.join(missing)}")
    return {name: parsed[name] for name in RESULT_NAMES}


def validate_record(
    record: Any,
    *,
    source_sha: str | None = None,
    source_tree: str | None = None,
    repository: str = REPOSITORY,
    workflow_digest: str | None = None,
    run_id: int | None = None,
    run_attempt: int | None = None,
    release_builds: bool | None = None,
) -> dict[str, Any]:
    if not isinstance(record, dict) or record.get("schema_version") != SCHEMA_VERSION:
        raise QualificationError("qualification schema is invalid")
    if record.get("repository") != repository:
        raise QualificationError("qualification repository does not match")

    source = record.get("source")
    if not isinstance(source, dict):
        raise QualificationError("qualification source is missing")
    commit = source.get("commit")
    tree = source.get("tree")
    if not isinstance(commit, str) or not SHA_RE.fullmatch(commit):
        raise QualificationError("qualification source commit is invalid")
    if not isinstance(tree, str) or not SHA_RE.fullmatch(tree):
        raise QualificationError("qualification source tree is invalid")
    if source_sha is not None and commit != source_sha:
        raise QualificationError("qualification source commit does not match request")
    if source_tree is not None and tree != source_tree:
        raise QualificationError("qualification source tree does not match request")

    inputs = record.get("inputs")
    if not isinstance(inputs, dict):
        raise QualificationError("qualification inputs are missing")
    lock_digest = inputs.get("cargo_lock_sha256")
    if not isinstance(lock_digest, str) or not SHA256_RE.fullmatch(lock_digest):
        raise QualificationError("qualification Cargo.lock digest is invalid")
    recorded_release_builds = inputs.get("release_builds")
    if not isinstance(recorded_release_builds, bool):
        raise QualificationError("qualification release-build flag is invalid")
    if release_builds is not None and recorded_release_builds is not release_builds:
        raise QualificationError("qualification release-build flag does not match request")

    workflow = record.get("workflow")
    if not isinstance(workflow, dict) or workflow.get("path") != DEFAULT_WORKFLOW:
        raise QualificationError("qualification workflow path is invalid")
    digest = workflow.get("sha256")
    if not isinstance(digest, str) or not SHA256_RE.fullmatch(digest):
        raise QualificationError("qualification workflow digest is invalid")
    if workflow_digest is not None and digest != workflow_digest:
        raise QualificationError("qualification workflow digest does not match checkout")
    recorded_run_id = workflow.get("run_id")
    if not isinstance(recorded_run_id, int) or recorded_run_id <= 0:
        raise QualificationError("qualification run ID is invalid")
    if run_id is not None and recorded_run_id != run_id:
        raise QualificationError("qualification run ID does not match artifact run")
    recorded_run_attempt = workflow.get("run_attempt")
    if not isinstance(recorded_run_attempt, int) or recorded_run_attempt <= 0:
        raise QualificationError("qualification run attempt is invalid")
    if run_attempt is not None and recorded_run_attempt != run_attempt:
        raise QualificationError("qualification run attempt does not match artifact run")

    results = record.get("results")
    if not isinstance(results, dict) or set(results) != set(RESULT_NAMES):
        raise QualificationError("qualification results are incomplete")
    if any(results[name] not in ALLOWED_RESULTS for name in RESULT_NAMES):
        raise QualificationError("qualification contains a non-success result")

    targets = record.get("targets")
    if (
        not isinstance(targets, list)
        or set(targets) != ALLOWED_TARGETS
        or len(targets) != len(ALLOWED_TARGETS)
    ):
        raise QualificationError("qualification native target matrix is incomplete")

    toolchain = record.get("toolchain")
    required_toolchain_fields = {"rustc", "commit-hash", "host", "release"}
    if not isinstance(toolchain, dict) or not required_toolchain_fields.issubset(toolchain):
        raise QualificationError("qualification toolchain identity is incomplete")
    if any(
        not isinstance(toolchain[field], str) or not toolchain[field]
        for field in required_toolchain_fields
    ):
        raise QualificationError("qualification toolchain identity is invalid")
    if not toolchain["rustc"].startswith("rustc ") or not SHA_RE.fullmatch(
        toolchain["commit-hash"]
    ):
        raise QualificationError("qualification toolchain identity is invalid")

    created_at = record.get("created_at")
    if not isinstance(created_at, str) or not TIMESTAMP_RE.fullmatch(created_at):
        raise QualificationError("qualification timestamp is invalid")
    try:
        datetime.fromisoformat(created_at.removesuffix("Z") + "+00:00")
    except ValueError as error:
        raise QualificationError("qualification timestamp is invalid") from error
    return record


def create_record(args: argparse.Namespace) -> None:
    root = args.root.resolve()
    source_sha = args.source_sha
    if not SHA_RE.fullmatch(source_sha):
        raise QualificationError("source SHA must be 40 lowercase hexadecimal characters")
    actual = git_output(root, "rev-parse", "HEAD")
    if actual != source_sha:
        raise QualificationError("checked-out HEAD does not match requested source SHA")
    source_tree = git_output(root, "rev-parse", "HEAD^{tree}")

    workflow_path = root / DEFAULT_WORKFLOW
    lock_path = root / "Cargo.lock"
    results = parse_results(args.result)
    targets = sorted(set(args.target))
    if set(targets) != ALLOWED_TARGETS:
        raise QualificationError("create requires the complete native target matrix")

    record = {
        "created_at": datetime.now(timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z"),
        "inputs": {
            "cargo_lock_sha256": sha256_file(lock_path),
            "release_builds": args.release_builds,
        },
        "repository": args.repository,
        "results": results,
        "schema_version": SCHEMA_VERSION,
        "source": {"commit": source_sha, "tree": source_tree},
        "targets": targets,
        "toolchain": rustc_identity(),
        "workflow": {
            "path": DEFAULT_WORKFLOW,
            "run_attempt": args.run_attempt,
            "run_id": args.run_id,
            "sha256": sha256_file(workflow_path),
        },
    }
    validate_record(record, source_sha=source_sha, source_tree=source_tree, repository=args.repository)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(record, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"qualification record written for {source_sha} ({source_tree})")


def verify_record(args: argparse.Namespace) -> None:
    root = args.root.resolve()
    record = json.loads(args.file.read_text(encoding="utf-8"))
    workflow_digest = None
    if not args.allow_workflow_drift:
        workflow_digest = sha256_file(root / DEFAULT_WORKFLOW)
    validate_record(
        record,
        source_sha=args.source_sha.lower() if args.source_sha else None,
        source_tree=args.source_tree.lower() if args.source_tree else None,
        repository=args.repository,
        workflow_digest=workflow_digest,
    )
    print(f"qualification record verified for {record['source']['commit']}")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.set_defaults(root=Path(__file__).resolve().parents[2])
    subparsers = parser.add_subparsers(dest="command", required=True)

    create = subparsers.add_parser("create")
    create.add_argument("--source-sha", required=True)
    create.add_argument("--repository", default=REPOSITORY)
    create.add_argument("--run-id", required=True, type=int)
    create.add_argument("--run-attempt", required=True, type=int)
    create.add_argument("--release-builds", action="store_true")
    create.add_argument("--result", action="append", default=[], required=True)
    create.add_argument("--target", action="append", default=[], required=True)
    create.add_argument("--output", type=Path, required=True)
    create.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[2])
    create.set_defaults(handler=create_record)

    verify = subparsers.add_parser("verify")
    verify.add_argument("--file", type=Path, required=True)
    verify.add_argument("--source-sha")
    verify.add_argument("--source-tree")
    verify.add_argument("--repository", default=REPOSITORY)
    verify.add_argument("--allow-workflow-drift", action="store_true")
    verify.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[2])
    verify.set_defaults(handler=verify_record)
    return parser


def main() -> int:
    args = build_parser().parse_args()
    try:
        args.handler(args)
    except (OSError, json.JSONDecodeError, QualificationError) as error:
        print(f"error: {error}", file=__import__("sys").stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
