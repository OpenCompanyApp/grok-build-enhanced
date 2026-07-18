#!/usr/bin/env python3
"""Local dry-run and regression tests for the fork release pipeline."""

from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace

sys.dont_write_bytecode = True
SCRIPT_DIR = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPT_DIR))

import release_contract as contract  # noqa: E402


class ReleaseContractTests(unittest.TestCase):
    def test_asset_names_exactly_match_updater_contract(self) -> None:
        version = "2.3.4-alpha.5"
        self.assertEqual(
            sorted(
                contract.asset_name(version, target.os, target.arch)
                for target in contract.TARGETS
            ),
            [
                "grok-2.3.4-alpha.5-linux-aarch64",
                "grok-2.3.4-alpha.5-linux-x86_64",
                "grok-2.3.4-alpha.5-macos-aarch64",
                "grok-2.3.4-alpha.5-macos-x86_64",
            ],
        )
        contract.validate_updater_contract(contract.repository_root())

    def test_workflow_is_pinned_and_fork_owned(self) -> None:
        contract.validate_workflow_contract(contract.repository_root())
        self.assertEqual(
            contract.RELEASE_REPOSITORY, "OpenCompanyApp/grok-build-enhanced"
        )
        self.assertNotIn("windows", contract.matrix_json().lower())

    def test_pinned_protoc_supports_every_native_release_runner(self) -> None:
        contract.validate_protoc_contract(contract.repository_root())

    def test_package_and_assembly_are_byte_deterministic(self) -> None:
        version = "2.3.4-alpha.5"
        source_commit = "a" * 40
        package_script = SCRIPT_DIR / "package_asset.sh"

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / contract.COMPILED_BINARY
            source.write_bytes(b"deterministic-test-binary\x00\x01\n")
            source.chmod(0o755)
            staging = root / "staged"
            staging.mkdir()

            for target in contract.TARGETS:
                packaged = root / f"package-{target.os}-{target.arch}"
                subprocess.run(
                    [
                        "bash",
                        str(package_script),
                        version,
                        target.os,
                        target.arch,
                        target.rust_target,
                        str(source),
                        str(packaged),
                        source_commit,
                        contract.RELEASE_REPOSITORY,
                    ],
                    check=True,
                    capture_output=True,
                    text=True,
                )
                for path in packaged.iterdir():
                    shutil.copyfile(path, staging / path.name)

            releases = []
            for suffix in ("one", "two"):
                output = root / f"release-{suffix}"
                args = SimpleNamespace(
                    version=version,
                    repository=contract.RELEASE_REPOSITORY,
                    source_commit=source_commit,
                    input_dir=str(staging),
                    output_dir=str(output),
                )
                contract.assemble_release(args)
                contract.verify_release_directory(
                    output,
                    version,
                    contract.RELEASE_REPOSITORY,
                    source_commit,
                    require_modes=True,
                )
                releases.append(output)

            first, second = releases
            self.assertEqual(
                sorted(path.name for path in first.iterdir()),
                sorted(path.name for path in second.iterdir()),
            )
            for path in first.iterdir():
                self.assertEqual(path.read_bytes(), (second / path.name).read_bytes())

    def test_package_rejects_official_upstream_repository(self) -> None:
        package_script = SCRIPT_DIR / "package_asset.sh"
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / contract.COMPILED_BINARY
            source.write_bytes(b"not-a-real-binary")
            source.chmod(0o755)
            result = subprocess.run(
                [
                    "bash",
                    str(package_script),
                    "1.2.3",
                    "linux",
                    "x86_64",
                    "x86_64-unknown-linux-gnu",
                    str(source),
                    str(root / "output"),
                    "b" * 40,
                    "xai-org-shared/grok-build",
                ],
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn(contract.RELEASE_REPOSITORY, result.stderr)
            self.assertFalse((root / "output").exists())


if __name__ == "__main__":
    unittest.main()
