"""Tests for the read-only fork manifest checker."""

from __future__ import annotations

import copy
import sys
import unittest

sys.dont_write_bytecode = True

from fixtures import ForkFixture


class CheckManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.fixture = ForkFixture()
        self.addCleanup(self.fixture.cleanup)

    def assert_schema_failure(
        self, document: dict[str, object], expected_fragment: str
    ) -> None:
        self.fixture.write_manifest(document)
        result = self.fixture.run_checker()
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("ERROR schema:", result.stdout)
        self.assertIn(expected_fragment, result.stdout)
        self.assertEqual(result.stderr, "")

    def test_valid_manifest_reports_uncovered_paths_in_bytewise_order(self) -> None:
        before = self.fixture.tracked_state()
        result = self.fixture.run_checker()
        after = self.fixture.tracked_state()

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(result.stderr, "")
        self.assertIn("OK patch-stack-objects:", result.stdout)
        self.assertIn("OK patch-stack-fingerprints:", result.stdout)
        self.assertIn("OK upstream-revisions:", result.stdout)
        self.assertIn("OK feature-paths:", result.stdout)
        self.assertIn("OK downstream-coverage: 1/3 downstream path(s) covered", result.stdout)
        self.assertIn("WARNING downstream-coverage: 2 downstream path(s)", result.stdout)
        uncovered = [
            line.removeprefix("UNCOVERED\t")
            for line in result.stdout.splitlines()
            if line.startswith("UNCOVERED\t")
        ]
        self.assertEqual(uncovered, ["src/a-uncovered.txt", "src/z-uncovered.txt"])
        self.assertEqual(before, after)

    def test_strict_coverage_turns_the_complete_uncovered_list_into_an_error(self) -> None:
        result = self.fixture.run_checker("--strict-coverage")
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("ERROR downstream-coverage: 2 downstream path(s)", result.stdout)
        self.assertEqual(
            [line for line in result.stdout.splitlines() if line.startswith("UNCOVERED\t")],
            ["UNCOVERED\tsrc/a-uncovered.txt", "UNCOVERED\tsrc/z-uncovered.txt"],
        )

    def test_short_or_non_lowercase_object_ids_are_rejected(self) -> None:
        mutations = [
            ("short commit", "abc123", "reviewed.commit must be a lowercase full"),
            (
                "uppercase tree",
                "A" * 40,
                "reviewed.tree must be a lowercase full",
            ),
        ]
        for label, value, expected in mutations:
            with self.subTest(label=label):
                document = self.fixture.make_document()
                if "commit" in label:
                    document["sources"][0]["reviewed"]["commit"] = value
                else:
                    document["sources"][0]["reviewed"]["tree"] = value
                self.assert_schema_failure(document, expected)

    def test_source_and_feature_ids_are_unique_and_references_resolve(self) -> None:
        duplicate_source = self.fixture.make_document()
        duplicate_source["sources"].append(copy.deepcopy(duplicate_source["sources"][0]))
        self.assert_schema_failure(duplicate_source, "duplicate source ID 'fixture-source'")

        duplicate_feature = self.fixture.make_document()
        duplicate_feature["features"].append(
            copy.deepcopy(duplicate_feature["features"][0])
        )
        self.assert_schema_failure(
            duplicate_feature, "duplicate feature ID 'fixture-feature'"
        )

        missing_source = self.fixture.make_document()
        missing_source["features"][0]["source_ids"] = ["missing-source"]
        self.assert_schema_failure(
            missing_source, "references unknown source ID 'missing-source'"
        )

    def test_unsafe_paths_and_globs_are_rejected(self) -> None:
        unsafe_patterns = [
            ("../escape", "'..' path components"),
            ("/absolute", "repository-relative"),
            ("src\\windows.txt", "not backslashes"),
            ("src/\u202efile.txt", "control or formatting characters"),
            ("src/**suffix", "only as a complete path component"),
            (".git/config", "Git administrative paths"),
            ("src/{a,b}.txt", "unsupported shell-style expansion"),
        ]
        for pattern, expected in unsafe_patterns:
            with self.subTest(pattern=pattern):
                document = self.fixture.make_document()
                document["features"][0]["paths"] = [pattern]
                self.assert_schema_failure(document, expected)

        literal_glob = self.fixture.make_document()
        literal_glob["features"][0]["integration_paths"] = ["src/*.txt"]
        self.assert_schema_failure(literal_glob, "literal path without glob")

    def test_integration_and_legal_anchors_must_exist(self) -> None:
        document = self.fixture.make_document()
        document["features"][0]["paths"].append("src/missing-integration.txt")
        document["features"][0]["integration_paths"] = [
            "src/missing-integration.txt"
        ]
        document["features"][0]["legal_paths"] = ["MISSING-NOTICE"]
        self.fixture.write_manifest(document)

        result = self.fixture.run_checker()
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("matches no downstream path", result.stdout)
        self.assertIn("does not exist in the checkout", result.stdout)
        self.assertIn("is absent at the frozen tip", result.stdout)

    def test_manifest_check_ids_are_hard_coded_and_commands_are_not_executed(self) -> None:
        sentinel = self.fixture.root / "must-not-exist"
        document = self.fixture.make_document()
        document["checks"][-1] = f"touch {sentinel}"
        document["command"] = f"touch {sentinel}"
        self.assert_schema_failure(document, "unknown hard-coded check ID")
        self.assertFalse(sentinel.exists())

    def test_unknown_nested_keys_and_duplicate_json_keys_are_rejected(self) -> None:
        document = self.fixture.make_document()
        document["patch_stack"]["command"] = "git reset --hard"
        self.assert_schema_failure(document, "unsupported key 'command'")

        self.fixture.manifest_path.write_text(
            '{"schema_version": 1, "schema_version": 1}\n', encoding="utf-8"
        )
        result = self.fixture.run_checker()
        self.assertEqual(result.returncode, 1)
        self.assertIn("duplicate JSON key", result.stdout)

    def test_upstream_revision_mismatch_is_fatal(self) -> None:
        self.fixture.write_upstream_versions(latest="f" * 40)
        result = self.fixture.run_checker()
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("ERROR upstream-revisions:", result.stdout)
        self.assertIn("latest-fetched revision mismatch", result.stdout)

    def test_tree_and_fingerprint_mismatches_are_fatal(self) -> None:
        tree_document = self.fixture.make_document()
        tree_document["patch_stack"]["frozen_tip"]["tree"] = "0" * 40
        self.fixture.write_manifest(tree_document)
        tree_result = self.fixture.run_checker()
        self.assertEqual(tree_result.returncode, 1)
        self.assertIn("frozen_tip tree mismatch", tree_result.stdout)

        fingerprint_document = self.fixture.make_document()
        fingerprint_document["patch_stack"]["fingerprints"]["numstat_sha256"] = (
            "0" * 64
        )
        self.fixture.write_manifest(fingerprint_document)
        fingerprint_result = self.fixture.run_checker()
        self.assertEqual(fingerprint_result.returncode, 1)
        self.assertIn("numstat_sha256 mismatch", fingerprint_result.stdout)

    def test_missing_feature_pattern_match_is_fatal(self) -> None:
        document = self.fixture.make_document()
        document["features"][0]["paths"].append("src/not-in-diff.txt")
        self.fixture.write_manifest(document)
        result = self.fixture.run_checker()
        self.assertEqual(result.returncode, 1)
        self.assertIn("path pattern matches no downstream path", result.stdout)


if __name__ == "__main__":
    unittest.main()
