#!/usr/bin/env python3
"""Offline security and reproducibility tests for vendor_warp_themes.py."""

from __future__ import annotations

import importlib.util
import io
import os
import stat
import subprocess
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path
from unittest import mock

SCRIPT = Path(__file__).resolve().parents[1] / "vendor_warp_themes.py"
SPEC = importlib.util.spec_from_file_location("vendor_warp_themes_tested", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
vendor = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = vendor
SPEC.loader.exec_module(vendor)


def run_git(repo: Path, *arguments: str) -> bytes:
    environment = os.environ.copy()
    environment.update(
        {
            "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_CONFIG_GLOBAL": os.devnull,
            "GIT_TERMINAL_PROMPT": "0",
            "LC_ALL": "C",
        }
    )
    completed = subprocess.run(
        ["git", "-C", os.fspath(repo), *arguments],
        check=False,
        capture_output=True,
        env=environment,
        timeout=30,
    )
    if completed.returncode != 0:
        raise AssertionError(
            f"git {' '.join(arguments)} failed: "
            f"{completed.stderr.decode('utf-8', errors='replace')}"
        )
    return completed.stdout


def create_source_repo(root: Path) -> tuple[Path, str]:
    repo = root / "source"
    repo.mkdir()
    run_git(repo, "init", "-q")
    run_git(repo, "config", "user.name", "Offline Test")
    run_git(repo, "config", "user.email", "offline@example.invalid")
    run_git(repo, "config", "commit.gpgsign", "false")
    (repo / "LICENSE").write_bytes(b"synthetic license\n")
    for category in vendor.CATEGORIES:
        category_root = repo / category
        category_root.mkdir()
        (category_root / f"{category}.yaml").write_bytes(
            f"name: synthetic-{category}\n".encode("utf-8")
        )
    # These are deliberately excluded from the selected archive.
    (repo / "base16" / "preview.png").write_bytes(b"preview\n")
    (repo / "standard" / "README.md").write_bytes(b"category documentation\n")
    (repo / ".trunk").mkdir()
    (repo / ".trunk" / "trunk.yaml").write_bytes(b"tooling: true\n")
    run_git(repo, "add", "-A")
    run_git(repo, "commit", "-qm", "source")
    revision = run_git(repo, "rev-parse", "HEAD").decode("ascii").strip()
    return repo, revision


def snapshot(root: Path) -> dict[str, tuple[int, bytes]]:
    result: dict[str, tuple[int, bytes]] = {}
    for path in sorted(root.rglob("*")):
        if path.is_file() and not path.is_symlink():
            relative = path.relative_to(root).as_posix()
            result[relative] = (stat.S_IMODE(path.stat().st_mode), path.read_bytes())
    return result


class VendorWarpThemesTests(unittest.TestCase):
    def test_every_source_git_command_disables_replacement_objects(self) -> None:
        completed = subprocess.CompletedProcess(
            args=[], returncode=0, stdout=b"ok\n", stderr=b""
        )
        with mock.patch.object(vendor.subprocess, "run", return_value=completed) as run:
            output = vendor._run_git(Path("source"), ["rev-parse", "HEAD"], binary=False)
        self.assertEqual(output, "ok\n")
        command = run.call_args.args[0]
        environment = run.call_args.kwargs["env"]
        self.assertEqual(command[:2], ["git", "--no-replace-objects"])
        self.assertEqual(environment["GIT_NO_REPLACE_OBJECTS"], "1")
        self.assertEqual(environment["GIT_NO_LAZY_FETCH"], "1")
        self.assertEqual(environment["GIT_OPTIONAL_LOCKS"], "0")
        self.assertEqual(environment["GIT_TERMINAL_PROMPT"], "0")

    def test_replacement_ref_cannot_change_selected_bytes(self) -> None:
        with tempfile.TemporaryDirectory(prefix="warp-replacement-test-") as raw:
            repo, revision = create_source_repo(Path(raw))
            original = run_git(repo, "show", f"{revision}:base16/base16.yaml")
            changed = b"name: replacement-base16\n"
            (repo / "base16" / "base16.yaml").write_bytes(changed)
            run_git(repo, "add", "base16/base16.yaml")
            run_git(repo, "commit", "-qm", "replacement")
            replacement = run_git(repo, "rev-parse", "HEAD").decode("ascii").strip()
            run_git(repo, "replace", revision, replacement)

            self.assertEqual(
                run_git(repo, "show", f"{revision}:base16/base16.yaml"), changed
            )
            corpus = vendor._load_source_corpus(repo, revision)
            self.assertEqual(corpus.themes["base16/base16.yaml"], original)
            self.assertNotEqual(corpus.themes["base16/base16.yaml"], changed)

    def test_archive_bytes_must_match_no_replace_tree_object_ids(self) -> None:
        license_bytes = b"license\n"
        expected_theme = b"good\n"
        archive_theme = b"evil\n"
        license_entry = vendor.TreeEntry(
            path="LICENSE",
            size=len(license_bytes),
            object_id=vendor._git_blob_object_id(license_bytes),
        )
        theme_entry = vendor.TreeEntry(
            path="base16/theme.yaml",
            size=len(expected_theme),
            object_id=vendor._git_blob_object_id(expected_theme),
        )

        stream = io.BytesIO()
        with tarfile.open(fileobj=stream, mode="w:") as archive:
            directory = tarfile.TarInfo("base16")
            directory.type = tarfile.DIRTYPE
            directory.mode = 0o755
            archive.addfile(directory)
            for name, data in (("LICENSE", license_bytes), (theme_entry.path, archive_theme)):
                member = tarfile.TarInfo(name)
                member.mode = 0o644
                member.size = len(data)
                archive.addfile(member, io.BytesIO(data))

        with mock.patch.object(vendor, "_run_git", return_value=stream.getvalue()):
            with self.assertRaisesRegex(vendor.VendorError, "no-replace ls-tree"):
                vendor._read_git_archive(
                    Path("source"),
                    "a" * 40,
                    [theme_entry],
                    license_entry,
                )

    def test_hostile_or_nonportable_components_are_rejected(self) -> None:
        rejected = (
            "base16/cafe\u0301.yaml",  # not NFC
            "base16/caf\u00e9.yaml",  # NFC, but outside conservative ASCII
            "base16/evil\u202eyaml.yaml",  # bidi override
            "base16/CON.yaml",
            "base16/nul.txt.yaml",
            "base16/has:colon.yaml",
            "base16/has space.yaml",
            "base16/trailing./theme.yaml",
            "base16/trailing /theme.yaml",
        )
        for candidate in rejected:
            with self.subTest(candidate=candidate):
                with self.assertRaises(vendor.VendorError):
                    vendor._safe_relative_path(candidate, label="test path")

        for accepted in (
            "LICENSE",
            ".trunk/trunk.yaml",
            "base16/base16_3024.yaml",
            "special_edition/first_light.yaml",
        ):
            with self.subTest(accepted=accepted):
                self.assertEqual(
                    vendor._safe_relative_path(accepted, label="test path"), accepted
                )

    def test_git_archive_and_apply_are_reproducible(self) -> None:
        with tempfile.TemporaryDirectory(prefix="warp-reproducibility-test-") as raw:
            root = Path(raw)
            repo, revision = create_source_repo(root)
            first = vendor._load_source_corpus(repo, revision)
            second = vendor._load_source_corpus(repo, revision)
            self.assertEqual(first, second)
            self.assertEqual(vendor._manifest_bytes(first), vendor._manifest_bytes(second))
            self.assertEqual(len(first.themes), len(vendor.CATEGORIES))
            self.assertNotIn("base16/preview.png", first.themes)

            destinations = [root / "destination-a", root / "destination-b"]
            readme = b"human-maintained attribution\n"
            candidate = vendor._candidate_files(first, readme)
            for destination in destinations:
                destination.mkdir()
                (destination / "README.md").write_bytes(readme)
                vendor._atomic_replace(destination, candidate, revision)
                vendor._validate_checked_destination(
                    destination, expected_revision=revision
                )
            self.assertEqual(snapshot(destinations[0]), snapshot(destinations[1]))
            self.assertEqual((destinations[0] / "README.md").read_bytes(), readme)


if __name__ == "__main__":
    unittest.main()
