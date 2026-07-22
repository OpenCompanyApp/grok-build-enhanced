"""Standard-library-only Git fixtures shared by the fork guardrail tests."""

from __future__ import annotations

import copy
import hashlib
import json
import os
import shutil
import stat
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
SERIES_FORMAT = "git-linear-history-v1"
DELTA_FORMAT = "git-tree-delta-v1"
COVERAGE_HISTORY_FORMAT = "git-first-parent-with-audited-upstream-merges-v1"
ACKNOWLEDGEMENT_TRAILER = "Fork-Upstream-Acknowledgement"
KNOWN_CHECKS = [
    "patch-stack-objects",
    "patch-stack-history",
    "coverage-candidate",
    "upstream-revisions",
    "feature-paths",
    "downstream-coverage",
]


def _pack_field(hasher: Any, value: bytes) -> None:
    hasher.update(len(value).to_bytes(8, "big"))
    hasher.update(value)


class ForkFixture:
    """A repository with exact, equivalent, and unpinned same-tree histories."""

    def __init__(self) -> None:
        self._temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self._temporary_directory.name)
        self.repo = self.root / "repo"
        self.repo.mkdir()
        self.manifest_path = self.root / "manifest.json"
        self.upstream_path = self.root / "UPSTREAM_VERSIONS.md"
        self.git_binary = str(Path(shutil.which("git") or "").resolve())
        if not Path(self.git_binary).is_file():
            raise RuntimeError("git is required for guardrail fixtures")

        self.git("init", "--quiet")
        self.git("config", "user.name", "Fork Fixture")
        self.git("config", "user.email", "fixture@example.invalid")
        self.write_repo_file("LICENSE", "fixture license\n")
        self.write_repo_file("src/integration.txt", "baseline\n")
        self.git("add", "--", "LICENSE", "src/integration.txt")
        self.git("commit", "--quiet", "-m", "fixture baseline")
        self.baseline = self.git_text("rev-parse", "HEAD")
        self.baseline_tree = self.git_text("rev-parse", "HEAD^{tree}")

        self.write_repo_file("src/integration.txt", "downstream phase one\n")
        self.write_repo_file("src/a-owned.txt", "a\n")
        self.git("add", "--", "src")
        self.git("commit", "--quiet", "-m", "fixture downstream phase one")
        self.exact_first = self.git_text("rev-parse", "HEAD")

        self.write_repo_file("src/integration.txt", "downstream\n")
        self.write_repo_file("src/z-owned.txt", "z\n")
        self.git("add", "--", "src")
        self.git("commit", "--quiet", "-m", "fixture downstream phase two")
        self.tip = self.git_text("rev-parse", "HEAD")
        self.tip_tree = self.git_text("rev-parse", "HEAD^{tree}")

        self.git("checkout", "--quiet", "--detach", self.baseline)
        self.write_repo_file("src/integration.txt", "thematic phase one\n")
        self.write_repo_file("src/z-owned.txt", "z\n")
        self.git("add", "--", "src")
        self.git("commit", "--quiet", "-m", "fixture thematic phase one")
        self.thematic_first = self.git_text("rev-parse", "HEAD")
        self.write_repo_file("src/integration.txt", "downstream\n")
        self.write_repo_file("src/a-owned.txt", "a\n")
        self.git("add", "--", "src")
        self.git("commit", "--quiet", "-m", "fixture thematic phase two")
        self.thematic = self.git_text("rev-parse", "HEAD")
        if self.git_text("rev-parse", "HEAD^{tree}") != self.tip_tree:
            raise AssertionError("thematic fixture did not reproduce the exact final tree")
        self.git("checkout", "--quiet", "--detach", self.tip)

        self.unpinned = self.commit_tree(
            self.tip_tree,
            [self.baseline],
            "fixture unpinned same-tree history",
        )
        self.previous_upstream = self.commit_tree(
            self.baseline_tree,
            [],
            "fixture disconnected previous upstream snapshot",
        )
        self.upstream = self.commit_tree(
            self.tip_tree,
            [self.previous_upstream],
            "fixture disconnected audited upstream snapshot",
        )
        self.document = self.make_document()
        self.write_manifest()
        self.write_upstream_versions()

    def cleanup(self) -> None:
        self._temporary_directory.cleanup()

    def env(self, extra: dict[str, str] | None = None) -> dict[str, str]:
        environment = os.environ.copy()
        environment.update(
            {
                "GIT_OPTIONAL_LOCKS": "0",
                "GIT_NO_LAZY_FETCH": "1",
                "GIT_NO_REPLACE_OBJECTS": "1",
                "PYTHONDONTWRITEBYTECODE": "1",
                "LC_ALL": "C",
                "FORK_GUARDRAIL_GIT": self.git_binary,
            }
        )
        if extra:
            environment.update(extra)
        return environment

    def git(
        self,
        *arguments: str,
        check: bool = True,
        input_bytes: bytes | None = None,
    ) -> subprocess.CompletedProcess[bytes]:
        result = subprocess.run(
            [self.git_binary, "-C", str(self.repo), *arguments],
            check=False,
            input=input_bytes,
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

    def path(self, relative_path: str) -> Path:
        return self.repo.joinpath(*relative_path.split("/"))

    def write_repo_file(self, relative_path: str, content: str) -> None:
        path = self.path(relative_path)
        path.parent.mkdir(parents=True, exist_ok=True)
        if path.is_symlink():
            path.unlink()
        path.write_text(content, encoding="utf-8")

    def remove_repo_path(self, relative_path: str) -> None:
        path = self.path(relative_path)
        if path.is_symlink() or path.is_file():
            path.unlink()
        elif path.is_dir():
            shutil.rmtree(path)

    def symlink_repo_path(self, relative_path: str, target: str) -> None:
        path = self.path(relative_path)
        path.parent.mkdir(parents=True, exist_ok=True)
        self.remove_repo_path(relative_path)
        os.symlink(target, path)

    def commit_tree(self, tree: str, parents: list[str], message: str) -> str:
        arguments = ["commit-tree", tree]
        for parent in parents:
            arguments.extend(("-p", parent))
        arguments.extend(("-m", message))
        return self.git_text(*arguments)

    def tree_entries(self, tree: str) -> dict[bytes, tuple[str, str, str]]:
        payload = self.git("ls-tree", "-r", "-z", "--full-tree", tree).stdout
        entries: dict[bytes, tuple[str, str, str]] = {}
        for record in payload.rstrip(b"\0").split(b"\0") if payload else []:
            metadata, path = record.split(b"\t", 1)
            mode_raw, type_raw, oid_raw = metadata.split(b" ", 2)
            entries[path] = (
                mode_raw.decode("ascii"),
                type_raw.decode("ascii"),
                oid_raw.decode("ascii"),
            )
        return entries

    def canonical_delta(self, before_tree: str, after_tree: str) -> str:
        before = self.tree_entries(before_tree)
        after = self.tree_entries(after_tree)
        changed = sorted(
            path
            for path in set(before) | set(after)
            if before.get(path) != after.get(path)
        )
        hasher = hashlib.sha256()
        hasher.update(DELTA_FORMAT.encode("ascii") + b"\0")
        for path in changed:
            old = before.get(path)
            new = after.get(path)
            values = (
                path,
                b"" if old is None else old[0].encode("ascii"),
                b"" if old is None else old[1].encode("ascii"),
                b"" if old is None else old[2].encode("ascii"),
                b"" if new is None else new[0].encode("ascii"),
                b"" if new is None else new[1].encode("ascii"),
                b"" if new is None else new[2].encode("ascii"),
            )
            for value in values:
                _pack_field(hasher, value)
        return hasher.hexdigest()

    def linear_records(self, candidate: str) -> list[dict[str, str]]:
        reverse_chain: list[tuple[str, str]] = []
        current = candidate
        while current != self.baseline:
            tokens = self.git_text("rev-list", "--parents", "-n", "1", current).split()
            if len(tokens) != 2 or tokens[0] != current:
                raise ValueError(f"{candidate} is not a single-parent series from baseline")
            reverse_chain.append((current, tokens[1]))
            current = tokens[1]
        reverse_chain.reverse()

        records: list[dict[str, str]] = []
        previous_tree = self.baseline_tree
        for commit, parent in reverse_chain:
            tree = self.git_text("rev-parse", f"{commit}^{{tree}}")
            records.append(
                {
                    "commit": commit,
                    "parent": parent,
                    "tree": tree,
                    "delta_sha256": self.canonical_delta(previous_tree, tree),
                }
            )
            previous_tree = tree
        return records

    def series_sha256(self, candidate: str, records: list[dict[str, str]]) -> str:
        candidate_tree = self.git_text("rev-parse", f"{candidate}^{{tree}}")
        hasher = hashlib.sha256()
        hasher.update(SERIES_FORMAT.encode("ascii") + b"\0")
        for value in (
            self.baseline,
            self.baseline_tree,
            candidate,
            candidate_tree,
            str(len(records)),
        ):
            _pack_field(hasher, value.encode("ascii"))
        for record in records:
            for key in ("commit", "parent", "tree", "delta_sha256"):
                _pack_field(hasher, record[key].encode("ascii"))
        return hasher.hexdigest()

    def attestation_for(
        self, candidate: str, *, attestation_id: str | None = None
    ) -> dict[str, Any]:
        records = self.linear_records(candidate)
        result: dict[str, Any] = {
            "commit_count": len(records),
            "series_sha256": self.series_sha256(candidate, records),
            "commits": [
                {
                    "commit": record["commit"],
                    "tree": record["tree"],
                    "delta_sha256": record["delta_sha256"],
                }
                for record in records
            ],
        }
        if attestation_id is not None:
            result = {
                "id": attestation_id,
                "candidate": {
                    "commit": candidate,
                    "tree": self.git_text("rev-parse", f"{candidate}^{{tree}}"),
                },
                **result,
            }
        return result

    def synthetic_merge_attestation(
        self, candidate: str, *, attestation_id: str
    ) -> dict[str, Any]:
        candidate_tree = self.git_text("rev-parse", f"{candidate}^{{tree}}")
        record = {
            "commit": candidate,
            "parent": self.baseline,
            "tree": candidate_tree,
            "delta_sha256": self.canonical_delta(self.baseline_tree, candidate_tree),
        }
        return {
            "id": attestation_id,
            "candidate": {"commit": candidate, "tree": candidate_tree},
            "commit_count": 1,
            "series_sha256": self.series_sha256(candidate, [record]),
            "commits": [
                {
                    "commit": record["commit"],
                    "tree": record["tree"],
                    "delta_sha256": record["delta_sha256"],
                }
            ],
        }

    def make_document(self) -> dict[str, Any]:
        return {
            "schema_version": 3,
            "patch_stack": {
                "baseline": {
                    "commit": self.baseline,
                    "tree": self.baseline_tree,
                },
                "frozen_tip": {"commit": self.tip, "tree": self.tip_tree},
                "history": {
                    "series_format": SERIES_FORMAT,
                    "delta_format": DELTA_FORMAT,
                    "exact": self.attestation_for(self.tip),
                    "thematic_equivalents": [
                        self.attestation_for(
                            self.thematic,
                            attestation_id="fixture-thematic-equivalent",
                        )
                    ],
                },
            },
            "sources": [
                {
                    "id": "fixture-source",
                    "project": "Fixture Source",
                    "remote": "https://example.invalid/fixture.git",
                    "tracked_ref": "main",
                    "reviewed": {"commit": self.upstream},
                    "latest_fetched": {"commit": self.upstream},
                }
            ],
            "checks": list(KNOWN_CHECKS),
            "coverage": {
                "allow_uncovered": False,
                "history_format": COVERAGE_HISTORY_FORMAT,
                "upstream_acknowledgements": [],
            },
            "features": [
                {
                    "id": "fixture-feature",
                    "description": "Fixture integration ownership.",
                    "source_ids": ["fixture-source"],
                    "paths": [
                        "src/a-owned.txt",
                        "src/integration.txt",
                        "src/z-owned.txt",
                    ],
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
        self,
        *,
        project: str | None = None,
        remote: str | None = None,
        tracked_ref: str | None = None,
        reviewed: str | None = None,
        latest: str | None = None,
    ) -> None:
        project_value = project or "Fixture Source"
        remote_value = remote or "https://example.invalid/fixture.git"
        ref_value = tracked_ref or "main"
        reviewed_revision = reviewed or self.upstream
        latest_revision = latest or self.upstream
        self.upstream_path.write_text(
            "\n".join(
                [
                    "# Upstream versions",
                    "",
                    "| Project | Remote and tracked ref | Last reviewed / fork baseline | Latest fetched |",
                    "| --- | --- | --- | --- |",
                    (
                        f"| {project_value} | `{remote_value}` `{ref_value}` "
                        f"| [`{reviewed_revision}`](https://example.invalid/commit/{reviewed_revision}) "
                        f"| [`{latest_revision}`](https://example.invalid/commit/{latest_revision}) |"
                    ),
                    "",
                ]
            ),
            encoding="utf-8",
        )

    def run_checker(
        self,
        *arguments: str,
        cwd: Path | None = None,
        extra_env: dict[str, str] | None = None,
    ) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
                sys.executable,
                "-I",
                "-B",
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
            cwd=cwd or self.root,
            env=self.env(extra_env),
        )

    def run_verifier(
        self,
        *arguments: str,
        cwd: Path | None = None,
        extra_env: dict[str, str] | None = None,
    ) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
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
            cwd=cwd or self.root,
            env=self.env(extra_env),
        )

    def create_net_zero_history(self) -> str:
        self.git("checkout", "--quiet", "--detach", self.baseline)
        self.write_repo_file("src/transient.txt", "must not disappear silently\n")
        self.git("add", "--", "src/transient.txt")
        self.git("commit", "--quiet", "-m", "fixture transient add")
        self.write_repo_file("src/integration.txt", "downstream\n")
        self.write_repo_file("src/a-owned.txt", "a\n")
        self.write_repo_file("src/z-owned.txt", "z\n")
        self.remove_repo_path("src/transient.txt")
        self.git("add", "-A", "--", "src")
        self.git("commit", "--quiet", "-m", "fixture transient delete and final tree")
        candidate = self.git_text("rev-parse", "HEAD")
        if self.git_text("rev-parse", "HEAD^{tree}") != self.tip_tree:
            raise AssertionError("net-zero fixture final tree mismatch")
        self.git("checkout", "--quiet", "--detach", self.tip)
        return candidate

    def create_empty_history(self) -> str:
        first = self.commit_tree(
            self.tip_tree, [self.baseline], "fixture final tree before empty commit"
        )
        return self.commit_tree(self.tip_tree, [first], "fixture forbidden empty commit")

    def create_merge_history(self) -> str:
        return self.commit_tree(
            self.tip_tree,
            [self.baseline, self.unpinned],
            "fixture forbidden side-parent merge",
        )

    def create_upstream_acknowledgement_merge(
        self,
        *,
        preserve_first_parent_tree: bool = True,
        trailer: str | None = None,
    ) -> tuple[str, dict[str, str]]:
        self.git("checkout", "--quiet", "--detach", self.tip)
        evidence = "\n".join(
            [
                "# Fixture upstream parity ledger",
                "",
                "Source ID: `fixture-source`",
                f"Previous commit: `{self.previous_upstream}`",
                f"Previous tree: `{self.baseline_tree}`",
                f"Target commit: `{self.upstream}`",
                f"Target tree: `{self.tip_tree}`",
                f"Target parent: `{self.previous_upstream}`",
                "",
                "## Complete 3 raw-path ledger",
                "",
                "| # | Upstream status and path | Outcome | Evidence |",
                "| ---: | --- | --- | --- |",
                "| 1 | `A` `src/a-owned.txt` | adopt | `FIXTURE` |",
                "| 2 | `M` `src/integration.txt` | adopt | `FIXTURE` |",
                "| 3 | `A` `src/z-owned.txt` | adopt | `FIXTURE` |",
                "",
            ]
        )
        self.write_repo_file("src/integration.txt", evidence)
        self.git("add", "--", "src/integration.txt")
        self.git("commit", "--quiet", "-m", "fixture upstream audit evidence")
        first_parent = self.git_text("rev-parse", "HEAD")
        first_parent_tree = self.git_text("rev-parse", "HEAD^{tree}")
        marker_tree = first_parent_tree if preserve_first_parent_tree else self.tip_tree
        expected_trailer = (
            f"{ACKNOWLEDGEMENT_TRAILER}: fixture-source@{self.upstream}"
        )
        marker = self.commit_tree(
            marker_tree,
            [first_parent, self.upstream],
            "fixture audited upstream acknowledgement\n\n"
            + (expected_trailer if trailer is None else trailer),
        )
        acknowledgement = {
            "source_id": "fixture-source",
            "commit": self.upstream,
            "tree": self.tip_tree,
            "reviewed_from": {
                "commit": self.previous_upstream,
                "tree": self.baseline_tree,
            },
            "target_parents": [self.previous_upstream],
            "evidence": {
                "path": "src/integration.txt",
                "sha256": hashlib.sha256(evidence.encode("utf-8")).hexdigest(),
            },
        }
        return marker, acknowledgement

    def create_successive_upstream_acknowledgement_merges(
        self,
    ) -> tuple[str, list[dict[str, Any]], str]:
        """Create two zero-delta markers for one advancing source."""

        first_marker, first = self.create_upstream_acknowledgement_merge()
        second_upstream = self.commit_tree(
            self.tip_tree,
            [self.upstream],
            "fixture second audited upstream snapshot",
        )
        evidence = "\n".join(
            [
                "# Fixture second upstream parity ledger",
                "",
                "Source ID: `fixture-source`",
                f"Previous commit: `{self.upstream}`",
                f"Previous tree: `{self.tip_tree}`",
                f"Target commit: `{second_upstream}`",
                f"Target tree: `{self.tip_tree}`",
                f"Target parent: `{self.upstream}`",
                "",
                "## Complete 0 raw-path ledger",
                "",
                "| # | Upstream status and path | Outcome | Evidence |",
                "| ---: | --- | --- | --- |",
                "",
            ]
        )

        self.git("checkout", "--quiet", "--detach", first_marker)
        self.write_repo_file("src/integration.txt", evidence)
        self.git("add", "--", "src/integration.txt")
        self.git("commit", "--quiet", "-m", "fixture second upstream audit evidence")
        first_parent = self.git_text("rev-parse", "HEAD")
        first_parent_tree = self.git_text("rev-parse", "HEAD^{tree}")
        trailer = f"{ACKNOWLEDGEMENT_TRAILER}: fixture-source@{second_upstream}"
        second_marker = self.commit_tree(
            first_parent_tree,
            [first_parent, second_upstream],
            f"fixture second audited upstream acknowledgement\n\n{trailer}",
        )
        second = {
            "source_id": "fixture-source",
            "commit": second_upstream,
            "tree": self.tip_tree,
            "reviewed_from": {
                "commit": self.upstream,
                "tree": self.tip_tree,
            },
            "target_parents": [self.upstream],
            "evidence": {
                "path": "src/integration.txt",
                "sha256": hashlib.sha256(evidence.encode("utf-8")).hexdigest(),
            },
        }
        return second_marker, [first, second], second_upstream

    def adopt_symlink_frozen_tip(self, relative_path: str, target: str) -> None:
        self.git("checkout", "--quiet", "--detach", self.tip)
        self.symlink_repo_path(relative_path, target)
        self.git("add", "-A", "--", relative_path)
        self.git("commit", "--quiet", "-m", f"fixture symlink anchor {relative_path}")
        self.tip = self.git_text("rev-parse", "HEAD")
        self.tip_tree = self.git_text("rev-parse", "HEAD^{tree}")
        self.thematic = self.commit_tree(
            self.tip_tree,
            [self.baseline],
            f"fixture thematic symlink tree {relative_path}",
        )
        self.unpinned = self.commit_tree(
            self.tip_tree,
            [self.baseline],
            f"fixture unpinned symlink tree {relative_path}",
        )
        self.reset_document()
        if relative_path == "LICENSE":
            self.document["features"][0]["paths"].append("LICENSE")
        self.write_manifest()
        self.write_upstream_versions()

    def _snapshot_tree(
        self, root: Path, *, exclude_top_level_git: bool = False
    ) -> tuple[tuple[str, tuple[Any, ...]], ...]:
        records: list[tuple[str, tuple[Any, ...]]] = []

        def visit(path: Path, relative: str) -> None:
            metadata = os.lstat(path)
            common = (
                stat.S_IFMT(metadata.st_mode),
                stat.S_IMODE(metadata.st_mode),
                metadata.st_size,
                metadata.st_mtime_ns,
                metadata.st_nlink,
            )
            if stat.S_ISLNK(metadata.st_mode):
                records.append((relative, common + ("link", os.readlink(path))))
                return
            if stat.S_ISREG(metadata.st_mode):
                digest = hashlib.sha256(path.read_bytes()).hexdigest()
                records.append((relative, common + ("file", digest)))
                return
            if stat.S_ISDIR(metadata.st_mode):
                records.append((relative, common + ("dir",)))
                with os.scandir(path) as iterator:
                    children = sorted(iterator, key=lambda entry: os.fsencode(entry.name))
                for child in children:
                    if exclude_top_level_git and not relative and child.name == ".git":
                        continue
                    child_relative = child.name if not relative else f"{relative}/{child.name}"
                    visit(Path(child.path), child_relative)
                return
            records.append((relative, common + ("special",)))

        visit(root, "")
        return tuple(records)

    def repository_state(self) -> dict[str, Any]:
        git_dir = Path(self.git_text("rev-parse", "--absolute-git-dir"))
        common_raw = Path(self.git_text("rev-parse", "--git-common-dir"))
        common_dir = (
            common_raw
            if common_raw.is_absolute()
            else (self.repo / common_raw).resolve()
        )
        state: dict[str, Any] = {
            "head": self.git_text("rev-parse", "HEAD"),
            "refs": self.git("show-ref", check=False).stdout,
            "worktree": self._snapshot_tree(
                self.repo, exclude_top_level_git=True
            ),
            "gitdir": self._snapshot_tree(git_dir),
            "common_dir": self._snapshot_tree(common_dir),
            "manifest": self._snapshot_tree(self.manifest_path),
            "ledger": self._snapshot_tree(self.upstream_path),
        }
        return state

    def bytecode_artifacts(self) -> tuple[str, ...]:
        roots = (self.root, SCRIPTS_DIR)
        artifacts: set[str] = set()
        for root in roots:
            for path in root.rglob("*"):
                if path.name == "__pycache__" or path.suffix in {".pyc", ".pyo"}:
                    artifacts.add(str(path))
        return tuple(sorted(artifacts))
