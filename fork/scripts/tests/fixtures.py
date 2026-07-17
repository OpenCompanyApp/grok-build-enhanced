"""Standard-library-only Git fixtures shared by the fork checker tests."""

from __future__ import annotations

import copy
import hashlib
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True

TESTS_DIR = Path(__file__).resolve().parent
SCRIPTS_DIR = TESTS_DIR.parent
CHECK_MANIFEST = SCRIPTS_DIR / "check_manifest.py"
VERIFY_PATCH_STACK = SCRIPTS_DIR / "verify_patch_stack.sh"
KNOWN_CHECKS = [
    "patch-stack-objects",
    "patch-stack-fingerprints",
    "upstream-revisions",
    "feature-paths",
    "downstream-coverage",
]


class ForkFixture:
    """A two-commit repository plus external manifest and upstream ledger."""

    def __init__(self) -> None:
        self._temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self._temporary_directory.name)
        self.repo = self.root / "repo"
        self.repo.mkdir()
        self.manifest_path = self.root / "manifest.json"
        self.upstream_path = self.root / "UPSTREAM_VERSIONS.md"

        self.git("init", "--quiet")
        self.git("config", "user.name", "Fork Fixture")
        self.git("config", "user.email", "fixture@example.invalid")
        self.write_repo_file("LICENSE", "fixture license\n")
        self.write_repo_file("src/integration.txt", "baseline\n")
        self.git("add", "--", "LICENSE", "src/integration.txt")
        self.git("commit", "--quiet", "-m", "fixture baseline")
        self.baseline = self.git_text("rev-parse", "HEAD")
        self.baseline_tree = self.git_text("rev-parse", "HEAD^{tree}")

        self.write_repo_file("src/integration.txt", "downstream\n")
        self.write_repo_file("src/a-uncovered.txt", "a\n")
        self.write_repo_file("src/z-uncovered.txt", "z\n")
        self.git("add", "--", "src")
        self.git("commit", "--quiet", "-m", "fixture downstream")
        self.tip = self.git_text("rev-parse", "HEAD")
        self.tip_tree = self.git_text("rev-parse", "HEAD^{tree}")

        self.document = self.make_document()
        self.write_manifest()
        self.write_upstream_versions()

    def cleanup(self) -> None:
        self._temporary_directory.cleanup()

    def env(self) -> dict[str, str]:
        environment = os.environ.copy()
        environment["GIT_OPTIONAL_LOCKS"] = "0"
        environment["PYTHONDONTWRITEBYTECODE"] = "1"
        environment["LC_ALL"] = "C"
        return environment

    def git(self, *arguments: str, check: bool = True) -> subprocess.CompletedProcess[bytes]:
        result = subprocess.run(
            ["git", "-C", str(self.repo), *arguments],
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=self.env(),
        )
        if check and result.returncode != 0:
            raise AssertionError(
                f"git {' '.join(arguments)} failed: "
                f"{result.stderr.decode('utf-8', errors='replace')}"
            )
        return result

    def git_text(self, *arguments: str) -> str:
        return self.git(*arguments).stdout.decode("ascii").strip()

    def write_repo_file(self, relative_path: str, content: str) -> None:
        path = self.repo.joinpath(*relative_path.split("/"))
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")

    def fingerprint(self, kind: str, right_commit: str | None = None) -> str:
        right = right_commit or self.tip
        common = ["--no-renames", "--no-ext-diff", "--no-textconv"]
        if kind == "diff":
            arguments = ["diff", "--binary", "--full-index", *common]
        elif kind == "name_status":
            arguments = ["diff", "--name-status", *common, "-z"]
        elif kind == "numstat":
            arguments = ["diff", "--numstat", *common, "-z"]
        else:
            raise ValueError(kind)
        payload = self.git(
            "-c",
            "core.quotePath=false",
            *arguments,
            self.baseline,
            right,
            "--",
        ).stdout
        return hashlib.sha256(payload).hexdigest()

    def make_document(self) -> dict[str, Any]:
        return {
            "schema_version": 1,
            "patch_stack": {
                "baseline": {
                    "commit": self.baseline,
                    "tree": self.baseline_tree,
                },
                "frozen_tip": {"commit": self.tip, "tree": self.tip_tree},
                "fingerprints": {
                    "algorithm": "sha256",
                    "diff_sha256": self.fingerprint("diff"),
                    "name_status_sha256": self.fingerprint("name_status"),
                    "numstat_sha256": self.fingerprint("numstat"),
                },
            },
            "sources": [
                {
                    "id": "fixture-source",
                    "project": "Fixture Source",
                    "remote": "https://example.invalid/fixture.git",
                    "tracked_ref": "main",
                    "reviewed": {
                        "commit": self.baseline,
                        "tree": self.baseline_tree,
                    },
                    "latest_fetched": {
                        "commit": self.tip,
                        "tree": self.tip_tree,
                    },
                }
            ],
            "checks": list(KNOWN_CHECKS),
            "coverage": {"allow_uncovered": True},
            "features": [
                {
                    "id": "fixture-feature",
                    "description": "Fixture integration ownership.",
                    "source_ids": ["fixture-source"],
                    "paths": ["src/integration.txt"],
                    "integration_paths": ["src/integration.txt"],
                    "legal_paths": ["LICENSE"],
                }
            ],
        }

    def reset_document(self) -> dict[str, Any]:
        self.document = self.make_document()
        return self.document

    def document_copy(self) -> dict[str, Any]:
        return copy.deepcopy(self.document)

    def write_manifest(self, document: dict[str, Any] | None = None) -> None:
        if document is not None:
            self.document = document
        self.manifest_path.write_text(
            json.dumps(self.document, indent=2, sort_keys=False) + "\n",
            encoding="utf-8",
        )

    def write_upstream_versions(
        self, *, reviewed: str | None = None, latest: str | None = None
    ) -> None:
        reviewed_revision = reviewed or self.baseline
        latest_revision = latest or self.tip
        self.upstream_path.write_text(
            "\n".join(
                [
                    "# Upstream versions",
                    "",
                    "| Project | Remote and tracked ref | Last reviewed / fork baseline | Latest fetched |",
                    "| --- | --- | --- | --- |",
                    (
                        "| Fixture Source | `https://example.invalid/fixture.git` `main` "
                        f"| [`{reviewed_revision}`](https://example.invalid/reviewed) "
                        f"| [`{latest_revision}`](https://example.invalid/latest) |"
                    ),
                    "",
                ]
            ),
            encoding="utf-8",
        )

    def run_checker(self, *arguments: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
                sys.executable,
                str(CHECK_MANIFEST),
                "--repo-root",
                str(self.repo),
                "--manifest",
                str(self.manifest_path),
                "--upstream-versions",
                str(self.upstream_path),
                *arguments,
            ],
            check=False,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=self.env(),
        )

    def run_verifier(self, *arguments: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
                "bash",
                str(VERIFY_PATCH_STACK),
                "--repo-root",
                str(self.repo),
                "--manifest",
                str(self.manifest_path),
                *arguments,
            ],
            check=False,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=self.env(),
        )

    def tracked_state(self) -> tuple[str, bytes, bytes, tuple[str, ...]]:
        head = self.git_text("rev-parse", "HEAD")
        status = self.git("status", "--porcelain=v1", "--untracked-files=all").stdout
        refs = self.git("show-ref").stdout
        files = tuple(
            sorted(
                str(path.relative_to(self.repo))
                for path in self.repo.rglob("*")
                if path.is_file() and ".git" not in path.relative_to(self.repo).parts
            )
        )
        return head, status, refs, files
