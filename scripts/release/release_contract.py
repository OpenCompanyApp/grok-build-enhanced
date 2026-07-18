#!/usr/bin/env python3
"""Build and verify the fork-owned GitHub Release asset contract."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import stat
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any, Iterable

sys.dont_write_bytecode = True

DISTRIBUTION = "Grok Build Enhanced"
RELEASE_REPOSITORY = "OpenCompanyApp/grok-build-enhanced"
COMPILED_BINARY = "xai-grok-pager"
INSTALLED_BINARY = "grok"
BUILD_PROFILE = "release-dist"
WORKFLOW_PATH = ".github/workflows/release.yml"
PROVENANCE_NAME = "RELEASE-PROVENANCE.json"
CHECKSUMS_NAME = "SHA256SUMS"
SHA_RE = re.compile(r"^(?:[0-9a-f]{40}|[0-9a-f]{64})$")
VERSION_RE = re.compile(
    r"^(0|[1-9][0-9]*)\."
    r"(0|[1-9][0-9]*)\."
    r"(0|[1-9][0-9]*)"
    r"(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$"
)
ACTION_SHA_RE = re.compile(r"^[0-9a-f]{40}$")


@dataclass(frozen=True)
class Target:
    runner: str
    os: str
    arch: str
    rust_target: str
    rustflags: str


# Native GitHub-hosted runners only. Windows and cross-compiled targets are
# intentionally absent until this fork has infrastructure that builds them.
# The Linux Arm64 override replaces the repository's host-optimized Neoverse
# V2 setting with the portable architecture baseline required for distribution.
TARGETS = (
    Target("macos-15", "macos", "aarch64", "aarch64-apple-darwin", ""),
    Target("macos-15-intel", "macos", "x86_64", "x86_64-apple-darwin", ""),
    Target(
        "ubuntu-24.04-arm",
        "linux",
        "aarch64",
        "aarch64-unknown-linux-gnu",
        "-C target-cpu=generic -C force-unwind-tables=yes",
    ),
    Target("ubuntu-22.04", "linux", "x86_64", "x86_64-unknown-linux-gnu", ""),
)


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def validate_repository(repository: str) -> str:
    if repository != RELEASE_REPOSITORY:
        raise ValueError(
            f"release repository must be {RELEASE_REPOSITORY}, got {repository!r}"
        )
    return repository


def validate_version(version: str) -> str:
    match = VERSION_RE.fullmatch(version)
    if match is None:
        raise ValueError(
            "version must be SemVer without a leading v or build metadata "
            f"(for example 1.2.3 or 1.2.3-alpha.1), got {version!r}"
        )
    prerelease = match.group(4)
    if prerelease:
        for identifier in prerelease.split("."):
            if identifier.isdigit() and len(identifier) > 1 and identifier.startswith("0"):
                raise ValueError(
                    f"numeric prerelease identifier has a leading zero: {identifier!r}"
                )
    return version


def validate_source_commit(source_commit: str) -> str:
    if SHA_RE.fullmatch(source_commit) is None:
        raise ValueError("source commit must be a lowercase 40- or 64-character object ID")
    return source_commit


def target_for(os_name: str, arch: str, rust_target: str | None = None) -> Target:
    matches = [target for target in TARGETS if target.os == os_name and target.arch == arch]
    if len(matches) != 1:
        raise ValueError(f"unsupported release target: {os_name}-{arch}")
    target = matches[0]
    if rust_target is not None and rust_target != target.rust_target:
        raise ValueError(
            f"{os_name}-{arch} must use Rust target {target.rust_target}, got {rust_target}"
        )
    return target


def asset_name(version: str, os_name: str, arch: str) -> str:
    validate_version(version)
    target_for(os_name, arch)
    return f"grok-{version}-{os_name}-{arch}"


def record_name(os_name: str, arch: str) -> str:
    target_for(os_name, arch)
    return f"asset-metadata-{os_name}-{arch}.json"


def canonical_json(value: Any) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode("utf-8")


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def require_regular_file(path: Path, label: str) -> os.stat_result:
    try:
        metadata = path.lstat()
    except FileNotFoundError as error:
        raise ValueError(f"{label} is missing: {path}") from error
    if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISREG(metadata.st_mode):
        raise ValueError(f"{label} must be a regular file, not a symlink: {path}")
    return metadata


def write_new_file(path: Path, data: bytes, mode: int = 0o644) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL
    descriptor = os.open(path, flags, mode)
    try:
        with os.fdopen(descriptor, "wb") as handle:
            descriptor = -1
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
    finally:
        if descriptor >= 0:
            os.close(descriptor)


def asset_record(
    *,
    version: str,
    repository: str,
    source_commit: str,
    os_name: str,
    arch: str,
    rust_target: str,
    asset_path: Path,
) -> dict[str, Any]:
    validate_version(version)
    validate_repository(repository)
    validate_source_commit(source_commit)
    target = target_for(os_name, arch, rust_target)
    metadata = require_regular_file(asset_path, "release asset")
    expected_name = asset_name(version, os_name, arch)
    if asset_path.name != expected_name:
        raise ValueError(f"asset must be named {expected_name}, got {asset_path.name}")
    if metadata.st_size <= 0:
        raise ValueError(f"release asset is empty: {asset_path}")
    return {
        "artifact": {
            "bytes": metadata.st_size,
            "mode": "0755",
            "name": expected_name,
            "sha256": sha256_file(asset_path),
        },
        "binary": {
            "compiled_as": COMPILED_BINARY,
            "installed_as": INSTALLED_BINARY,
        },
        "distribution": DISTRIBUTION,
        "repository": repository,
        "schema_version": 1,
        "source_commit": source_commit,
        "target": asdict(target),
        "version": version,
    }


def write_asset_record(args: argparse.Namespace) -> None:
    asset_path = Path(args.asset).resolve()
    output = Path(args.output)
    record = asset_record(
        version=args.version,
        repository=args.repository,
        source_commit=args.source_commit,
        os_name=args.os,
        arch=args.arch,
        rust_target=args.rust_target,
        asset_path=asset_path,
    )
    write_new_file(output, canonical_json(record))


def load_json(path: Path, label: str) -> Any:
    require_regular_file(path, label)
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValueError(f"{label} is not canonical JSON: {path}: {error}") from error


def expected_input_names(version: str) -> set[str]:
    return {
        *(asset_name(version, target.os, target.arch) for target in TARGETS),
        *(record_name(target.os, target.arch) for target in TARGETS),
    }


def list_regular_names(directory: Path, label: str) -> set[str]:
    try:
        entries = list(directory.iterdir())
    except FileNotFoundError as error:
        raise ValueError(f"{label} is missing: {directory}") from error
    names: set[str] = set()
    for path in entries:
        require_regular_file(path, f"{label} entry")
        if path.name in names:
            raise ValueError(f"duplicate {label} entry: {path.name}")
        names.add(path.name)
    return names


def validate_input_records(
    input_dir: Path, version: str, repository: str, source_commit: str
) -> list[dict[str, Any]]:
    expected_names = expected_input_names(version)
    actual_names = list_regular_names(input_dir, "asset staging directory")
    if actual_names != expected_names:
        missing = sorted(expected_names - actual_names)
        unexpected = sorted(actual_names - expected_names)
        raise ValueError(
            f"asset staging set mismatch; missing={missing}, unexpected={unexpected}"
        )

    records: list[dict[str, Any]] = []
    for target in TARGETS:
        binary = input_dir / asset_name(version, target.os, target.arch)
        record_path = input_dir / record_name(target.os, target.arch)
        actual_record = load_json(record_path, "asset metadata")
        expected_record = asset_record(
            version=version,
            repository=repository,
            source_commit=source_commit,
            os_name=target.os,
            arch=target.arch,
            rust_target=target.rust_target,
            asset_path=binary,
        )
        if actual_record != expected_record:
            raise ValueError(f"asset metadata does not match binary: {record_path}")
        if record_path.read_bytes() != canonical_json(actual_record):
            raise ValueError(f"asset metadata is not canonical JSON: {record_path}")
        records.append(actual_record)
    return records


def provenance_document(
    version: str,
    repository: str,
    source_commit: str,
    records: Iterable[dict[str, Any]],
) -> dict[str, Any]:
    assets = []
    for record in records:
        assets.append(
            {
                "arch": record["target"]["arch"],
                "bytes": record["artifact"]["bytes"],
                "name": record["artifact"]["name"],
                "os": record["target"]["os"],
                "runner": record["target"]["runner"],
                "rust_target": record["target"]["rust_target"],
                "rustflags": record["target"]["rustflags"],
                "sha256": record["artifact"]["sha256"],
            }
        )
    assets.sort(key=lambda item: item["name"])
    return {
        "assets": assets,
        "build": {
            "compiled_binary": COMPILED_BINARY,
            "features": ["release-dist"],
            "installed_binary": INSTALLED_BINARY,
            "native_targets_only": True,
            "profile": BUILD_PROFILE,
            "workflow": WORKFLOW_PATH,
        },
        "distribution": DISTRIBUTION,
        "release_tag": f"v{version}",
        "repository": repository,
        "schema_version": 1,
        "source_commit": source_commit,
    }


def checksums_bytes(directory: Path, names: Iterable[str]) -> bytes:
    lines = [f"{sha256_file(directory / name)}  {name}\n" for name in sorted(names)]
    return "".join(lines).encode("ascii")


def ensure_empty_output_directory(output_dir: Path) -> None:
    if output_dir.is_symlink():
        raise ValueError(f"output directory may not be a symlink: {output_dir}")
    if output_dir.exists():
        if not output_dir.is_dir():
            raise ValueError(f"output path must be a directory: {output_dir}")
        if any(output_dir.iterdir()):
            raise ValueError(f"output directory must be empty: {output_dir}")
    else:
        output_dir.mkdir(parents=True)


def assemble_release(args: argparse.Namespace) -> None:
    version = validate_version(args.version)
    repository = validate_repository(args.repository)
    source_commit = validate_source_commit(args.source_commit)
    input_dir = Path(args.input_dir).resolve()
    output_dir = Path(args.output_dir).resolve()
    records = validate_input_records(input_dir, version, repository, source_commit)
    ensure_empty_output_directory(output_dir)

    copied_names: list[str] = []
    try:
        for target in TARGETS:
            name = asset_name(version, target.os, target.arch)
            destination = output_dir / name
            shutil.copyfile(input_dir / name, destination)
            destination.chmod(0o755)
            copied_names.append(name)
        provenance = provenance_document(version, repository, source_commit, records)
        write_new_file(output_dir / PROVENANCE_NAME, canonical_json(provenance))
        checksum_subjects = [*copied_names, PROVENANCE_NAME]
        write_new_file(
            output_dir / CHECKSUMS_NAME,
            checksums_bytes(output_dir, checksum_subjects),
        )
        verify_release_directory(
            output_dir, version, repository, source_commit, require_modes=True
        )
    except Exception:
        shutil.rmtree(output_dir, ignore_errors=True)
        raise


def expected_release_names(version: str) -> set[str]:
    return {
        *(asset_name(version, target.os, target.arch) for target in TARGETS),
        PROVENANCE_NAME,
        CHECKSUMS_NAME,
    }


def verify_release_directory(
    directory: Path,
    version: str,
    repository: str,
    source_commit: str,
    *,
    require_modes: bool,
) -> None:
    validate_version(version)
    validate_repository(repository)
    validate_source_commit(source_commit)
    actual_names = list_regular_names(directory, "release directory")
    expected_names = expected_release_names(version)
    if actual_names != expected_names:
        raise ValueError(
            "release file set mismatch; "
            f"missing={sorted(expected_names - actual_names)}, "
            f"unexpected={sorted(actual_names - expected_names)}"
        )

    records = []
    for target in TARGETS:
        path = directory / asset_name(version, target.os, target.arch)
        metadata = require_regular_file(path, "release binary")
        if metadata.st_size <= 0:
            raise ValueError(f"release binary is empty: {path}")
        if require_modes and stat.S_IMODE(metadata.st_mode) != 0o755:
            raise ValueError(f"release binary mode must be 0755: {path}")
        records.append(
            {
                "artifact": {
                    "bytes": metadata.st_size,
                    "name": path.name,
                    "sha256": sha256_file(path),
                },
                "target": asdict(target),
            }
        )

    expected_provenance = provenance_document(
        version,
        repository,
        source_commit,
        (
            {
                "artifact": record["artifact"],
                "target": record["target"],
            }
            for record in records
        ),
    )
    provenance_path = directory / PROVENANCE_NAME
    actual_provenance = load_json(provenance_path, "release provenance")
    if actual_provenance != expected_provenance:
        raise ValueError("release provenance does not match release binaries")
    if provenance_path.read_bytes() != canonical_json(actual_provenance):
        raise ValueError("release provenance is not canonical JSON")

    subjects = sorted(expected_names - {CHECKSUMS_NAME})
    expected_checksums = checksums_bytes(directory, subjects)
    checksums_path = directory / CHECKSUMS_NAME
    if checksums_path.read_bytes() != expected_checksums:
        raise ValueError("SHA256SUMS does not exactly match the release file set")


def verify_release(args: argparse.Namespace) -> None:
    verify_release_directory(
        Path(args.release_dir).resolve(),
        args.version,
        args.repository,
        args.source_commit,
        require_modes=args.require_modes,
    )


def validate_updater_contract(root: Path) -> None:
    update_dir = root / "crates/codegen/xai-grok-update/src"
    version_source = (update_dir / "version.rs").read_text(encoding="utf-8")
    install_source = (update_dir / "auto_update.rs").read_text(encoding="utf-8")

    version_fragments = (
        'pub const GH_RELEASE_REPO: &str = "OpenCompanyApp/grok-build-enhanced";',
        '"https://api.github.com/repos/OpenCompanyApp/grok-build-enhanced/releases?per_page=100"',
        '"https://github.com/OpenCompanyApp/grok-build-enhanced/releases/download"',
        'format!("grok-{version}-{os}-{arch}")',
        'if installer != "gh-release"',
        'fetch_gh_release_version(&config.channel).await',
        'fetch_gh_release_version("stable")',
    )
    install_fragments = (
        'Some("gh-release")',
        'if installer == "gh-release"',
        'install_gh_release(target).await',
        'ensure_release_asset_exists(&version, release_api_url, os, arch).await?',
        'release_asset_name(&version, os, arch)',
        'gh_release_download(&download_url, &binary_path).await?',
    )
    for source, fragments in (
        (version_source, version_fragments),
        (install_source, install_fragments),
    ):
        for fragment in fragments:
            if fragment not in source:
                raise ValueError(
                    "fork updater routing contract changed; missing source fragment "
                    f"{fragment!r}"
                )

    if "xai-org-shared/grok-build" in version_source[:2_000]:
        raise ValueError("updater repository constant points at the old official release repo")

    sample_version = "1.2.3-alpha.1"
    updater_names = {
        f"grok-{sample_version}-{os_name}-{arch}"
        for os_name in ("macos", "linux")
        for arch in ("x86_64", "aarch64")
    }
    pipeline_names = {
        asset_name(sample_version, target.os, target.arch) for target in TARGETS
    }
    if pipeline_names != updater_names:
        raise ValueError(
            f"pipeline assets differ from updater selection: {sorted(pipeline_names)}"
        )


def workflow_job_block(workflow: str, job_name: str) -> str:
    match = re.search(
        rf"^  {re.escape(job_name)}:\n(.*?)(?=^  [a-z][a-z0-9_-]*:\n|\Z)",
        workflow,
        re.M | re.S,
    )
    if match is None:
        raise ValueError(f"release workflow is missing job {job_name!r}")
    return match.group(1)


def validate_workflow_contract(root: Path) -> None:
    workflow = (root / WORKFLOW_PATH).read_text(encoding="utf-8")
    trigger_block, separator, _jobs = workflow.partition("\njobs:\n")
    if not separator:
        raise ValueError("release workflow is missing jobs")
    if "pull_request_target" in workflow or re.search(
        r"^\s*pull_request\s*:", workflow, re.M
    ):
        raise ValueError("release workflow must not run for pull requests")
    if "schedule:" in trigger_block or "branches:" in trigger_block:
        raise ValueError("release workflow may run only for tags or manual dispatch")
    if "permissions:\n  contents: read\n" not in trigger_block:
        raise ValueError("release workflow must default to contents: read")
    if "secrets." in workflow:
        raise ValueError("release workflow must not consume repository secrets")

    for required in (
        "workflow_dispatch:",
        "tags:",
        "inputs.confirm_release == true",
        f"github.repository == '{RELEASE_REPOSITORY}'",
        "fromJSON(needs.prepare.outputs.matrix)",
        "scripts/release/release_contract.py matrix",
        "contents: write",
        "attestations: write",
        "id-token: write",
    ):
        if required not in workflow:
            raise ValueError(f"release workflow is missing guardrail {required!r}")
    if "xai-org-shared/grok-build" in workflow or "x.ai/cli" in workflow:
        raise ValueError(
            "release workflow must not reference an official upstream release channel"
        )
    if workflow.count(f"github.repository == '{RELEASE_REPOSITORY}'") != 5:
        raise ValueError("every release workflow job must have an exact fork-ownership guard")

    prepare = workflow_job_block(workflow, "prepare")
    validate = workflow_job_block(workflow, "validate")
    build = workflow_job_block(workflow, "build")
    attest = workflow_job_block(workflow, "attest")
    release = workflow_job_block(workflow, "release")
    if any("contents: write" in block for block in (prepare, validate, build, attest)):
        raise ValueError("only the final non-building release job may write contents")
    if workflow.count("contents: write") != 1:
        raise ValueError("release workflow must grant contents: write exactly once")
    for fragment in (
        "fetch-depth: 0",
        'git merge-base --is-ancestor "$THEMATIC_CHECKPOINT" "$source_commit"',
        'git merge-base --is-ancestor "$source_commit" refs/remotes/origin/main',
        "fork/scripts/check_manifest.py --strict-coverage",
        "fork/scripts/check_fork_contracts.py",
        "scripts/release/tests/test_release_pipeline.py",
    ):
        if fragment not in prepare:
            raise ValueError(f"release prepare job is missing candidate guard {fragment!r}")
    for fragment in (
        "needs: prepare",
        "ref: ${{ needs.prepare.outputs.source_commit }}",
        "custom_credentials_never_inherit_xai_session_or_global_keys",
        "codex_rejects_static_or_generic_header_credentials",
        "cargo check --locked -p xai-grok-pager-bin",
    ):
        if fragment not in validate:
            raise ValueError(f"release validation job is missing exact-SHA check {fragment!r}")
    if "needs: [prepare, validate]" not in build:
        raise ValueError("release builds must wait for exact-candidate validation")
    if "id-token: write" not in attest or "attestations: write" not in attest:
        raise ValueError("only the attestation job must receive provenance permissions")
    if "id-token: write" in release or "attestations: write" in release:
        raise ValueError("release job must not receive provenance permissions")
    if (
        "gh release upload" not in release
        or 'release create "$TAG"' not in release
        or 'gh "${create_args[@]}"' not in release
    ):
        raise ValueError("only the release job may publish GitHub Release assets")
    if any("gh release upload" in block for block in (prepare, validate, build, attest)):
        raise ValueError("a validation, build, or attestation job may not publish release assets")

    uses = re.findall(r"^\s*uses:\s*([^@\s]+)@([^\s#]+)", workflow, re.M)
    if not uses:
        raise ValueError("release workflow contains no pinned actions")
    for action, revision in uses:
        if ACTION_SHA_RE.fullmatch(revision) is None:
            raise ValueError(f"action {action} is not pinned to a full commit: {revision}")


def validate_contract(args: argparse.Namespace) -> None:
    validate_repository(args.repository)
    root = repository_root()
    validate_updater_contract(root)
    validate_workflow_contract(root)
    names = sorted(asset_name(args.version, target.os, target.arch) for target in TARGETS)
    print(
        f"release contract OK: {args.repository}; "
        f"updater assets={','.join(names)}"
    )


def matrix_json() -> str:
    matrix = {"include": [asdict(target) for target in TARGETS]}
    return json.dumps(matrix, separators=(",", ":"))


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    matrix = subparsers.add_parser("matrix", help="print the native runner matrix")
    matrix.set_defaults(handler=lambda _args: print(matrix_json()))

    name = subparsers.add_parser("asset-name", help="print one updater-compatible name")
    name.add_argument("--version", required=True)
    name.add_argument("--os", required=True)
    name.add_argument("--arch", required=True)
    name.add_argument("--rust-target", required=True)
    name.add_argument("--repository", required=True)
    name.set_defaults(
        handler=lambda args: (
            validate_repository(args.repository),
            target_for(args.os, args.arch, args.rust_target),
            print(asset_name(args.version, args.os, args.arch)),
        )
    )

    record = subparsers.add_parser("write-record", help="write one canonical asset record")
    record.add_argument("--version", required=True)
    record.add_argument("--repository", required=True)
    record.add_argument("--source-commit", required=True)
    record.add_argument("--os", required=True)
    record.add_argument("--arch", required=True)
    record.add_argument("--rust-target", required=True)
    record.add_argument("--asset", required=True)
    record.add_argument("--output", required=True)
    record.set_defaults(handler=write_asset_record)

    assemble = subparsers.add_parser(
        "assemble", help="assemble checksums and deterministic provenance"
    )
    assemble.add_argument("--version", required=True)
    assemble.add_argument("--repository", required=True)
    assemble.add_argument("--source-commit", required=True)
    assemble.add_argument("--input-dir", required=True)
    assemble.add_argument("--output-dir", required=True)
    assemble.set_defaults(handler=assemble_release)

    verify = subparsers.add_parser("verify", help="verify a complete release directory")
    verify.add_argument("--version", required=True)
    verify.add_argument("--repository", required=True)
    verify.add_argument("--source-commit", required=True)
    verify.add_argument("--release-dir", required=True)
    verify.add_argument("--require-modes", action="store_true")
    verify.set_defaults(handler=verify_release)

    contract = subparsers.add_parser(
        "contract", help="dry-run workflow ownership and updater asset selection"
    )
    contract.add_argument("--version", default="1.2.3-contract.1")
    contract.add_argument("--repository", default=RELEASE_REPOSITORY)
    contract.set_defaults(handler=validate_contract)

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        result = args.handler(args)
        if isinstance(result, tuple):
            # argparse lambdas above use a tuple only to sequence validations.
            del result
    except (OSError, ValueError) as error:
        parser.error(str(error))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
