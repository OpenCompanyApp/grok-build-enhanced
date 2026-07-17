"""Tests for exact and explicitly attested read-only history verification."""

from __future__ import annotations

import copy
import sys
import unittest
from pathlib import Path

sys.dont_write_bytecode = True

from fixtures import ForkFixture


class VerifyPatchStackTests(unittest.TestCase):
    def setUp(self) -> None:
        self.fixture = ForkFixture()
        self.addCleanup(self.fixture.cleanup)

    def test_exact_success_is_read_only_and_range_diff_is_only_diagnostic(self) -> None:
        before = self.fixture.repository_state()
        bytecode_before = self.fixture.bytecode_artifacts()
        result = self.fixture.run_verifier()
        bytecode_after = self.fixture.bytecode_artifacts()
        after = self.fixture.repository_state()

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(result.stderr, "")
        self.assertIn("PASS: exact candidate commit identity equals", result.stdout)
        self.assertIn("PASS: candidate tree is exactly", result.stdout)
        self.assertIn("raw index/worktree proof is clean", result.stdout)
        self.assertIn(
            "--- range-diff diagnostics (non-authoritative; never proof) ---",
            result.stdout,
        )
        self.assertIn("Patch-stack verification succeeded.", result.stdout)
        self.assertEqual(before, after)
        self.assertEqual(bytecode_before, bytecode_after)

    def test_explicit_thematic_equivalence_authenticates_the_pinned_series(self) -> None:
        before = self.fixture.repository_state()
        result = self.fixture.run_verifier(
            "--candidate",
            self.fixture.thematic,
            "--history-mode",
            "thematic-equivalence",
            "--attestation",
            "fixture-thematic-equivalent",
            "--no-range-diff",
        )
        after = self.fixture.repository_state()

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(result.stderr, "")
        self.assertIn(
            "candidate commit equals thematic attestation 'fixture-thematic-equivalent'",
            result.stdout,
        )
        self.assertIn("PASS: candidate tree is exactly", result.stdout)
        self.assertIn("Patch-stack verification succeeded.", result.stdout)
        self.assertEqual(before, after)

    def test_safe_annotated_ref_is_resolved_to_the_pinned_commit(self) -> None:
        ref_name = "archive/tree-equivalent-fixture"
        self.fixture.git(
            "tag", "-a", ref_name, "-m", "fixture archive", self.fixture.thematic
        )
        before = self.fixture.repository_state()
        result = self.fixture.run_verifier(
            "--candidate",
            ref_name,
            "--history-mode",
            "thematic-equivalence",
            "--attestation",
            "fixture-thematic-equivalent",
            "--no-range-diff",
        )
        after = self.fixture.repository_state()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn(f"Candidate:    {self.fixture.thematic}", result.stdout)
        self.assertEqual(before, after)

    def test_exact_mode_rejects_different_commit_with_identical_tree(self) -> None:
        result = self.fixture.run_verifier(
            "--candidate", self.fixture.unpinned, "--no-range-diff"
        )
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn(
            "exact mode requires candidate commit to equal the manifest frozen commit",
            result.stderr,
        )
        self.assertIn("PASS: candidate tree is exactly", result.stdout)

    def test_thematic_mode_rejects_unpinned_same_tree_history(self) -> None:
        result = self.fixture.run_verifier(
            "--candidate",
            self.fixture.unpinned,
            "--history-mode",
            "thematic-equivalence",
            "--attestation",
            "fixture-thematic-equivalent",
            "--no-range-diff",
        )
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn(
            "candidate commit does not equal thematic attestation",
            result.stderr,
        )
        self.assertIn("PASS: candidate tree is exactly", result.stdout)

    def test_thematic_attestation_is_mandatory_and_must_be_known(self) -> None:
        missing = self.fixture.run_verifier(
            "--candidate",
            self.fixture.thematic,
            "--history-mode",
            "thematic-equivalence",
            "--no-range-diff",
        )
        self.assertEqual(missing.returncode, 2, missing.stdout + missing.stderr)
        self.assertIn("--attestation is required", missing.stderr)

        unknown = self.fixture.run_verifier(
            "--candidate",
            self.fixture.thematic,
            "--history-mode",
            "thematic-equivalence",
            "--attestation",
            "not-pinned",
            "--no-range-diff",
        )
        self.assertEqual(unknown.returncode, 2, unknown.stdout + unknown.stderr)
        self.assertIn("unknown thematic attestation ID", unknown.stderr)

        exact_with_attestation = self.fixture.run_verifier(
            "--attestation", "fixture-thematic-equivalent", "--no-range-diff"
        )
        self.assertEqual(exact_with_attestation.returncode, 2)
        self.assertIn("only valid in thematic-equivalence mode", exact_with_attestation.stderr)

    def test_thematic_mode_rejects_attested_net_zero_churn(self) -> None:
        candidate = self.fixture.create_net_zero_history()
        document = self.fixture.make_document()
        document["patch_stack"]["history"]["thematic_equivalents"].append(
            self.fixture.attestation_for(
                candidate, attestation_id="fixture-net-zero-history"
            )
        )
        self.fixture.write_manifest(document)

        result = self.fixture.run_verifier(
            "--candidate",
            candidate,
            "--history-mode",
            "thematic-equivalence",
            "--attestation",
            "fixture-net-zero-history",
            "--no-range-diff",
        )
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("net-zero intermediate churn", result.stderr)

    def test_dirty_candidate_fails_raw_proof_unless_explicitly_skipped(self) -> None:
        self.fixture.write_repo_file("untracked.txt", "dirty\n")
        dirty = self.fixture.run_verifier("--no-range-diff")
        self.assertEqual(dirty.returncode, 1, dirty.stdout + dirty.stderr)
        self.assertIn("raw index/worktree proof is not clean", dirty.stderr)
        self.assertIn("untracked worktree path 'untracked.txt'", dirty.stderr)

        skipped = self.fixture.run_verifier(
            "--skip-clean-check", "--no-range-diff"
        )
        self.assertEqual(skipped.returncode, 0, skipped.stdout + skipped.stderr)
        self.assertIn("clean-status check explicitly skipped", skipped.stdout)

    def test_staged_candidate_drift_is_detected_without_porcelain_diff(self) -> None:
        self.fixture.write_repo_file("src/integration.txt", "staged drift\n")
        self.fixture.git("add", "--", "src/integration.txt")
        result = self.fixture.run_verifier("--no-range-diff")
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("index entry 'src/integration.txt' differs", result.stderr)

    def test_candidate_with_extra_commit_fails_identity_and_tree(self) -> None:
        self.fixture.write_repo_file("src/candidate-extra.txt", "candidate drift\n")
        self.fixture.git("add", "--", "src/candidate-extra.txt")
        self.fixture.git("commit", "--quiet", "-m", "candidate drift")

        result = self.fixture.run_verifier("--candidate", "HEAD", "--no-range-diff")
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("exact mode requires candidate commit", result.stderr)
        self.assertIn("candidate tree mismatch", result.stderr)

    def test_cli_candidate_syntax_is_validated_before_git(self) -> None:
        result = self.fixture.run_verifier(
            "--candidate", "HEAD^{tree}", "--no-range-diff"
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn("unsafe candidate revision", result.stderr)

        option_like = self.fixture.run_verifier(
            "--candidate=-c", "--no-range-diff"
        )
        self.assertEqual(option_like.returncode, 2)
        self.assertIn("unsafe candidate revision", option_like.stderr)

        dotted = self.fixture.run_verifier(
            "--candidate", "refs/./hidden", "--no-range-diff"
        )
        self.assertEqual(dotted.returncode, 2)
        self.assertIn("unsafe candidate revision", dotted.stderr)

    def test_manifest_values_are_python_parsed_and_never_shell_executed(self) -> None:
        sentinel = self.fixture.root / "manifest-command-ran"
        document = self.fixture.make_document()
        document["patch_stack"]["baseline"]["commit"] = f"$(touch {sentinel})"
        document["command"] = f"touch {sentinel}"
        self.fixture.write_manifest(document)

        result = self.fixture.run_verifier("--no-range-diff")
        self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
        self.assertIn("ERROR schema:", result.stderr)
        self.assertFalse(sentinel.exists())

    def test_malformed_or_duplicate_manifest_never_falls_back(self) -> None:
        self.fixture.manifest_path.write_text(
            '{"patch_stack": {"baseline": {"commit": "bad"}}}\n',
            encoding="utf-8",
        )
        malformed = self.fixture.run_verifier("--no-range-diff")
        self.assertEqual(malformed.returncode, 2)
        self.assertIn("ERROR schema:", malformed.stderr)

        self.fixture.manifest_path.write_text(
            '{"schema_version": 2, "schema_version": 2}\n', encoding="utf-8"
        )
        duplicate = self.fixture.run_verifier("--no-range-diff")
        self.assertEqual(duplicate.returncode, 2)
        self.assertIn("duplicate JSON key", duplicate.stderr)

    def test_hostile_diff_attributes_and_filters_are_never_executed(self) -> None:
        sentinel = self.fixture.root / "hostile-git-command-ran"
        hostile = self.fixture.root / "hostile-git-command.sh"
        hostile.write_text(
            f"#!/bin/sh\n: > '{sentinel}'\nexit 97\n", encoding="utf-8"
        )
        hostile.chmod(0o755)
        self.fixture.git("config", "diff.external", str(hostile))
        self.fixture.git("config", "diff.hostile.command", str(hostile))
        self.fixture.git("config", "diff.hostile.textconv", str(hostile))
        self.fixture.git("config", "filter.hostile.clean", str(hostile))
        self.fixture.git("config", "filter.hostile.smudge", str(hostile))
        self.fixture.write_repo_file(
            ".gitattributes", "* diff=hostile filter=hostile -text\n"
        )
        exclude = self.fixture.repo / ".git" / "info" / "exclude"
        with exclude.open("a", encoding="utf-8") as exclude_file:
            exclude_file.write("\n.gitattributes\n")
        before = self.fixture.repository_state()

        result = self.fixture.run_verifier(
            extra_env={"GIT_EXTERNAL_DIFF": str(hostile)}
        )
        after = self.fixture.repository_state()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("raw index/worktree proof is clean", result.stdout)
        self.assertIn("range-diff diagnostics", result.stdout)
        self.assertFalse(sentinel.exists())
        self.assertEqual(before, after)

    def test_launcher_isolates_hostile_cwd_path_and_pythonpath_without_bytecode(self) -> None:
        hostile_cwd = self.fixture.root / "hostile-cwd"
        hostile_cwd.mkdir()
        hostile_bin = self.fixture.root / "hostile-bin"
        hostile_bin.mkdir()
        hostile_modules = self.fixture.root / "hostile-pythonpath"
        hostile_modules.mkdir()
        sentinel = self.fixture.root / "hostile-launcher-ran"
        shell_payload = f"#!/bin/sh\n: > '{sentinel}'\nexit 97\n"
        for name in ("python3", "git"):
            executable = hostile_bin / name
            executable.write_text(shell_payload, encoding="utf-8")
            executable.chmod(0o755)
        python_payload = (
            "from pathlib import Path\n"
            f"Path({str(sentinel)!r}).write_text('ran', encoding='utf-8')\n"
        )
        (hostile_modules / "sitecustomize.py").write_text(
            python_payload, encoding="utf-8"
        )
        (hostile_modules / "json.py").write_text(python_payload, encoding="utf-8")
        shell_startup = hostile_modules / "shell-startup.sh"
        shell_startup.write_text(shell_payload, encoding="utf-8")
        before = self.fixture.repository_state()
        bytecode_before = self.fixture.bytecode_artifacts()

        result = self.fixture.run_verifier(
            "--no-range-diff",
            cwd=hostile_cwd,
            extra_env={
                "PATH": str(hostile_bin),
                "PYTHONPATH": str(hostile_modules),
                "PYTHONHOME": str(hostile_modules),
                "BASH_ENV": str(shell_startup),
                "ENV": str(shell_startup),
            },
        )
        bytecode_after = self.fixture.bytecode_artifacts()
        after = self.fixture.repository_state()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertFalse(sentinel.exists())
        self.assertEqual(before, after)
        self.assertEqual(bytecode_before, bytecode_after)

    def test_help_documents_modes_and_read_only_scope(self) -> None:
        result = self.fixture.run_verifier("--help")
        self.assertEqual(result.returncode, 0)
        self.assertIn("candidate defaults to the frozen tip", result.stdout)
        self.assertIn("thematic-equivalence", result.stdout)
        self.assertIn("never fetches", result.stdout)
        self.assertIn("Range-diff output is diagnostic and never proof", result.stdout)
        self.assertEqual(result.stderr, "")


if __name__ == "__main__":
    unittest.main()
