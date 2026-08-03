#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import subprocess
import sys
import unittest
from pathlib import Path
from unittest import mock

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location("ci_impact", ROOT / "scripts/ci/impact.py")
assert SPEC and SPEC.loader
impact_module = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = impact_module
SPEC.loader.exec_module(impact_module)


class ImpactTests(unittest.TestCase):
    def test_documentation_only_diff_skips_rust(self) -> None:
        impact = impact_module.classify_paths(["docs/ci-cd.md", "README.md"])
        self.assertFalse(impact.run_rust)
        self.assertFalse(impact.run_native)
        self.assertFalse(impact.full)

    def test_global_manifest_change_fails_safe_to_full(self) -> None:
        impact = impact_module.classify_paths(["Cargo.lock"])
        self.assertTrue(impact.full)
        self.assertTrue(impact.run_core)
        self.assertTrue(impact.run_ui)
        self.assertTrue(impact.run_pty)
        self.assertTrue(impact.run_libraries)
        self.assertTrue(impact.run_native)

    def test_ci_or_pinned_tool_change_fails_safe_to_full(self) -> None:
        for path in (
            "scripts/ci/run.sh",
            "bin/protoc",
            ".github/workflows/fork-contracts.yml",
            "./.cargo/config.toml",
        ):
            with self.subTest(path=path):
                impact = impact_module.classify_paths([path])
                self.assertTrue(impact.full)

    def test_core_change_selects_core_reverse_dependencies_and_native(self) -> None:
        impact = impact_module.classify_paths(
            ["crates/codegen/xai-grok-shell/src/agent/config.rs"]
        )
        self.assertTrue(impact.run_rust)
        self.assertTrue(impact.run_core)
        self.assertTrue(impact.run_libraries)
        self.assertTrue(impact.run_native)
        self.assertFalse(impact.run_ui)
        self.assertEqual(impact.outputs()["broad_groups"], '["core","libraries"]')

    def test_leaf_change_selects_only_library_lane(self) -> None:
        impact = impact_module.classify_paths(
            ["crates/codegen/xai-token-estimation/src/lib.rs"]
        )
        self.assertTrue(impact.run_rust)
        self.assertTrue(impact.run_libraries)
        self.assertFalse(impact.run_core)
        self.assertFalse(impact.run_ui)
        self.assertFalse(impact.run_pty)
        self.assertFalse(impact.run_native)

    def test_pager_change_selects_ui_pty_libraries_and_native(self) -> None:
        impact = impact_module.classify_paths(
            ["crates/codegen/xai-grok-pager/src/app/cli.rs"]
        )
        self.assertTrue(impact.run_ui)
        self.assertTrue(impact.run_pty)
        self.assertTrue(impact.run_libraries)
        self.assertTrue(impact.run_native)

    def test_github_output_reason_contains_no_control_characters(self) -> None:
        outputs = impact_module.Impact(reason="line-one\r\nline-two\x7f").outputs()
        self.assertEqual(outputs["reason"], "line-one  line-two ")

    def test_git_paths_are_nul_delimited_and_newlines_remain_in_one_path(self) -> None:
        completed = subprocess.CompletedProcess(
            args=["git"],
            returncode=0,
            stdout=b"docs/name\nwith-newline.md\0crates/example/src/lib.rs\0",
            stderr=b"",
        )
        with mock.patch.object(impact_module.subprocess, "run", return_value=completed) as run:
            paths, error = impact_module.git_changed_paths(
                ROOT,
                "a" * 40,
                "b" * 40,
            )

        self.assertIsNone(error)
        self.assertEqual(
            paths,
            ["docs/name\nwith-newline.md", "crates/example/src/lib.rs"],
        )
        command = run.call_args.args[0]
        self.assertIn("--no-ext-diff", command)
        self.assertIn("-z", command)
        self.assertEqual(run.call_args.kwargs["env"]["GIT_NO_REPLACE_OBJECTS"], "1")


if __name__ == "__main__":
    unittest.main()
