#!/usr/bin/env python3
"""Classify a Git diff into conservative CI test groups."""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
from dataclasses import dataclass, fields
from pathlib import Path

ZERO_SHA_RE = re.compile(r"^0+$")
SHA_RE = re.compile(r"^[0-9a-f]{40}$")

GLOBAL_PATHS = {
    "Cargo.lock",
    "Cargo.toml",
    "rust-toolchain.toml",
    "rustfmt.toml",
    "clippy.toml",
}
GLOBAL_PREFIXES = (
    ".cargo/",
    ".github/actions/",
    ".github/workflows/",
    "bin/",
    "crates/build/",
    "crates/common/",
    "prod/",
    "scripts/ci/",
    "third_party/",
)
CORE_PREFIXES = (
    "crates/codegen/xai-grok-auth/",
    "crates/codegen/xai-grok-config/",
    "crates/codegen/xai-grok-config-types/",
    "crates/codegen/xai-grok-provider-http/",
    "crates/codegen/xai-grok-sampler/",
    "crates/codegen/xai-grok-sampling-types/",
    "crates/codegen/xai-grok-shell/",
    "crates/codegen/xai-grok-tools/",
    "crates/codegen/xai-grok-tools-api/",
)
UI_PREFIXES = (
    "crates/codegen/xai-grok-markdown/",
    "crates/codegen/xai-grok-markdown-core/",
    "crates/codegen/xai-grok-mermaid/",
    "crates/codegen/xai-grok-pager/",
    "crates/codegen/xai-grok-pager-minimal/",
    "crates/codegen/xai-grok-pager-render/",
)
PTY_PREFIXES = (
    "crates/codegen/ptyctl/",
    "crates/codegen/ptyctl-cli/",
    "crates/codegen/xai-grok-pager/",
    "crates/codegen/xai-grok-pager-bin/",
    "crates/codegen/xai-grok-pager-pty-harness/",
    "crates/codegen/xai-tty-utils/",
)
NATIVE_PREFIXES = CORE_PREFIXES + UI_PREFIXES + PTY_PREFIXES + (
    "bin/",
    "crates/codegen/xai-grok-sandbox/",
    "crates/codegen/xai-system-power/",
    "scripts/release/",
)


@dataclass
class Impact:
    run_rust: bool = False
    run_core: bool = False
    run_ui: bool = False
    run_pty: bool = False
    run_libraries: bool = False
    run_native: bool = False
    full: bool = False
    reason: str = "no Rust-affecting paths"

    @classmethod
    def full_impact(cls, reason: str) -> "Impact":
        return cls(
            run_rust=True,
            run_core=True,
            run_ui=True,
            run_pty=True,
            run_libraries=True,
            run_native=True,
            full=True,
            reason=reason,
        )

    def outputs(self) -> dict[str, str]:
        result = {
            field.name: str(bool(getattr(self, field.name))).lower()
            for field in fields(self)
            if field.name != "reason"
        }
        broad_groups = [
            group
            for group, selected in (
                ("core", self.run_core),
                ("ui", self.run_ui),
                ("pty", self.run_pty),
                ("libraries", self.run_libraries),
            )
            if selected
        ]
        result["broad_groups"] = json.dumps(broad_groups, separators=(",", ":"))
        result["reason"] = "".join(
            character if character.isprintable() else " "
            for character in self.reason
        )
        return result


def is_manifest(path: str) -> bool:
    return path.endswith("/Cargo.toml") or path == "Cargo.toml"


def classify_paths(paths: list[str]) -> Impact:
    normalized = sorted(
        {
            path.strip().removeprefix("./")
            for path in paths
            if path.strip()
        }
    )
    if not normalized:
        return Impact(reason="empty diff")

    for path in normalized:
        if path in GLOBAL_PATHS or path.startswith(GLOBAL_PREFIXES) or is_manifest(path):
            return Impact.full_impact(f"global build or CI input changed: {path}")

    impact = Impact()
    reasons: list[str] = []
    for path in normalized:
        rust_source = path.endswith(".rs") or path.startswith("crates/") or path.startswith("prod/")
        if not rust_source:
            continue
        impact.run_rust = True
        if path.startswith(CORE_PREFIXES):
            impact.run_core = True
            reasons.append("core")
        elif path.startswith(UI_PREFIXES):
            impact.run_ui = True
            reasons.append("ui")
        elif path.startswith(PTY_PREFIXES):
            impact.run_pty = True
            reasons.append("pty")
        else:
            impact.run_libraries = True
            reasons.append("libraries")

        if path.startswith(PTY_PREFIXES):
            impact.run_pty = True
        if path.startswith(NATIVE_PREFIXES):
            impact.run_native = True

    if impact.run_rust:
        # The composed binary and focused provider contracts always run. Test the
        # leaf/reverse-dependency lane for core/UI changes as a conservative check.
        if impact.run_core or impact.run_ui or impact.run_pty:
            impact.run_libraries = True
        impact.reason = "selected Rust groups: " + ",".join(sorted(set(reasons)))
    return impact


def git_changed_paths(root: Path, base: str, head: str) -> tuple[list[str], str | None]:
    if not SHA_RE.fullmatch(head):
        return [], f"invalid head SHA: {head!r}"
    if not SHA_RE.fullmatch(base) or ZERO_SHA_RE.fullmatch(base):
        return [], "base SHA is missing or is the all-zero push sentinel"
    environment = os.environ.copy()
    environment["GIT_NO_REPLACE_OBJECTS"] = "1"
    process = subprocess.run(
        [
            "git",
            "diff",
            "--no-ext-diff",
            "--name-only",
            "-z",
            "--diff-filter=ACDMRTUXB",
            base,
            head,
            "--",
        ],
        cwd=root,
        env=environment,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if process.returncode != 0:
        return [], "git diff base could not be resolved"
    try:
        paths = [entry.decode("utf-8") for entry in process.stdout.split(b"\0") if entry]
    except UnicodeDecodeError:
        return [], "git diff contains a non-UTF-8 path"
    return paths, None


def write_outputs(path: Path, impact: Impact) -> None:
    with path.open("a", encoding="utf-8") as output:
        for key, value in impact.outputs().items():
            output.write(f"{key}={value}\n")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", required=True)
    parser.add_argument("--head", required=True)
    parser.add_argument("--github-output", type=Path)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[2])
    return parser


def main() -> int:
    args = build_parser().parse_args()
    paths, error = git_changed_paths(args.root, args.base.lower(), args.head.lower())
    impact = Impact.full_impact(error) if error else classify_paths(paths)
    if args.github_output:
        write_outputs(args.github_output, impact)
    for key, value in impact.outputs().items():
        print(f"{key}={value}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
