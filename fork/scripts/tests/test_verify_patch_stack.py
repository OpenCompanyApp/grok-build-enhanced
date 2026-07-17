"""Tests for the safe, read-only patch-stack verifier."""

from __future__ import annotations

import sys
import unittest

sys.dont_write_bytecode = True

from fixtures import ForkFixture


class VerifyPatchStackTests(unittest.TestCase):
    def setUp(self) -> None:
        self.fixture = ForkFixture()
        self.addCleanup(self.fixture.cleanup)

    def test_success_is_read_only_and_includes_range_diff_diagnostics(self) -> None:
        before = self.fixture.tracked_state()
        result = self.fixture.run_verifier()
        after = self.fixture.tracked_state()

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(result.stderr, "")
        self.assertIn("PASS: baseline tree is exactly", result.stdout)
        self.assertIn("PASS: candidate tree diff is exactly equal", result.stdout)
        self.assertIn("PASS: candidate name-status fingerprint matches", result.stdout)
        self.assertIn("PASS: candidate numstat fingerprint matches", result.stdout)
        self.assertIn("--- range-diff diagnostics (non-authoritative) ---", result.stdout)
        self.assertIn("Patch-stack verification succeeded.", result.stdout)
        self.assertEqual(before, after)

    def test_dirty_checked_out_candidate_fails_unless_explicitly_skipped(self) -> None:
        self.fixture.write_repo_file("untracked.txt", "dirty\n")
        dirty = self.fixture.run_verifier("--no-range-diff")
        self.assertEqual(dirty.returncode, 1, dirty.stdout + dirty.stderr)
        self.assertIn("worktree or index is not clean", dirty.stderr)

        skipped = self.fixture.run_verifier(
            "--skip-clean-check", "--no-range-diff"
        )
        self.assertEqual(skipped.returncode, 0, skipped.stdout + skipped.stderr)
        self.assertIn("clean-status check explicitly skipped", skipped.stdout)

    def test_candidate_with_extra_commit_fails_exact_and_fingerprint_checks(self) -> None:
        self.fixture.write_repo_file("src/candidate-extra.txt", "candidate drift\n")
        self.fixture.git("add", "--", "src/candidate-extra.txt")
        self.fixture.git("commit", "--quiet", "-m", "candidate drift")

        result = self.fixture.run_verifier("--candidate", "HEAD")
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("candidate tree mismatch", result.stderr)
        self.assertIn("candidate tree diff is not equal", result.stderr)
        self.assertIn("candidate diff fingerprint mismatch", result.stderr)
        self.assertIn("candidate name-status fingerprint mismatch", result.stderr)
        self.assertIn("range-diff diagnostics", result.stdout)

    def test_wrong_expected_tree_and_hashes_are_fatal(self) -> None:
        wrong_tree = self.fixture.run_verifier(
            "--expected-tree", "0" * 40, "--no-range-diff"
        )
        self.assertEqual(wrong_tree.returncode, 1)
        self.assertIn("frozen-tip tree mismatch", wrong_tree.stderr)
        self.assertIn("candidate tree mismatch", wrong_tree.stderr)

        wrong_hash = self.fixture.run_verifier(
            "--numstat-sha256", "0" * 64, "--no-range-diff"
        )
        self.assertEqual(wrong_hash.returncode, 1)
        self.assertIn("frozen numstat fingerprint mismatch", wrong_hash.stderr)
        self.assertIn("candidate numstat fingerprint mismatch", wrong_hash.stderr)

    def test_cli_revision_overrides_are_validated_before_git(self) -> None:
        result = self.fixture.run_verifier(
            "--candidate", "HEAD^{tree}", "--no-range-diff"
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn("unsafe candidate revision", result.stderr)

        short_tree = self.fixture.run_verifier(
            "--expected-tree", "abc123", "--no-range-diff"
        )
        self.assertEqual(short_tree.returncode, 2)
        self.assertIn("expected tree must be a full lowercase SHA", short_tree.stderr)

    def test_malformed_manifest_is_rejected_without_falling_back(self) -> None:
        self.fixture.manifest_path.write_text(
            '{"patch_stack": {"baseline": {"commit": "bad"}}}\n',
            encoding="utf-8",
        )
        result = self.fixture.run_verifier("--no-range-diff")
        self.assertEqual(result.returncode, 2)
        self.assertIn("invalid manifest", result.stderr)

    def test_help_documents_read_only_scope(self) -> None:
        result = self.fixture.run_verifier("--help")
        self.assertEqual(result.returncode, 0)
        self.assertIn("candidate defaults to the frozen tip", result.stdout)
        self.assertIn("never fetches", result.stdout)
        self.assertEqual(result.stderr, "")


if __name__ == "__main__":
    unittest.main()
