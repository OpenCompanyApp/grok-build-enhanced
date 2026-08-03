#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
MODULE_PATH = REPOSITORY_ROOT / "fork/scripts/check_fork_contracts.py"
SPEC = importlib.util.spec_from_file_location("check_fork_contracts", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
contracts = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = contracts
SPEC.loader.exec_module(contracts)


class WorkflowPinContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary_directory.name)
        self.original_root = contracts.ROOT
        contracts.ROOT = self.root
        (self.root / ".github/actions/setup-rust-ci").mkdir(parents=True)
        (self.root / ".github/workflows").mkdir(parents=True)
        self._write_valid_automation()

    def tearDown(self) -> None:
        contracts.ROOT = self.original_root
        self.temporary_directory.cleanup()

    def _write(self, relative: str, content: str) -> None:
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")

    def _write_valid_automation(self) -> None:
        setup_reference = "    uses: ./.github/actions/setup-rust-ci\n"
        dotslash_reference = f"    uses: {contracts.DOTSLASH_ACTION}\n"
        self._write(
            ".github/actions/setup-rust-ci/action.yml",
            f"runs:\n  steps:\n{dotslash_reference}",
        )
        self._write(
            ".github/workflows/fork-contracts.yml",
            setup_reference * 3,
        )
        self._write(
            ".github/workflows/deep-ci.yml",
            setup_reference * 3 + dotslash_reference,
        )
        self._write(
            ".github/workflows/rebase-qualification.yml",
            setup_reference * 3,
        )
        self._write(
            ".github/workflows/release.yml",
            dotslash_reference * 2,
        )

    def test_accepts_pinned_actions_reached_through_local_composite(self) -> None:
        contracts.check_workflow_pins()

    def test_rejects_unpinned_external_action_in_workflow(self) -> None:
        release = self.root / ".github/workflows/release.yml"
        release.write_text(
            release.read_text(encoding="utf-8") + "    uses: actions/checkout@v6\n",
            encoding="utf-8",
        )

        with self.assertRaisesRegex(contracts.ContractError, "not pinned to a full SHA"):
            contracts.check_workflow_pins()

    def test_rejects_missing_cached_setup_lane(self) -> None:
        self._write(
            ".github/workflows/rebase-qualification.yml",
            "    uses: ./.github/actions/setup-rust-ci\n" * 2,
        )

        with self.assertRaisesRegex(contracts.ContractError, "cached Rust setup"):
            contracts.check_workflow_pins()


if __name__ == "__main__":
    unittest.main()
