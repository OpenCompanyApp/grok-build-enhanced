"""Tests for the fail-closed, read-only fork manifest checker."""

from __future__ import annotations

import copy
import os
import sys
import unittest
from pathlib import Path

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

    def test_valid_manifest_is_complete_read_only_and_tree_authoritative(self) -> None:
        before = self.fixture.repository_state()
        bytecode_before = self.fixture.bytecode_artifacts()
        result = self.fixture.run_checker("--strict-coverage")
        bytecode_after = self.fixture.bytecode_artifacts()
        after = self.fixture.repository_state()

        self.assertEqual(
            set(before),
            {"head", "refs", "worktree", "gitdir", "common_dir", "manifest", "ledger"},
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(result.stderr, "")
        self.assertIn("OK patch-stack-objects:", result.stdout)
        self.assertIn("OK patch-stack-history:", result.stdout)
        self.assertIn("1 thematic history attestation(s) authenticated", result.stdout)
        self.assertIn(
            "remote/ref/revision record(s) cross-checked; no external tree mappings claimed",
            result.stdout,
        )
        self.assertIn(
            "OK downstream-coverage: 3/3 downstream path(s) covered", result.stdout
        )
        self.assertNotIn("UNCOVERED\t", result.stdout)
        self.assertEqual(before, after)
        self.assertEqual(bytecode_before, bytecode_after)

    def test_coverage_is_always_fail_closed_and_bytewise_sorted(self) -> None:
        document = self.fixture.make_document()
        document["features"][0]["paths"] = ["src/integration.txt"]
        self.fixture.write_manifest(document)

        result = self.fixture.run_checker()
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("ERROR downstream-coverage: 2 downstream path(s)", result.stdout)
        self.assertEqual(
            [line for line in result.stdout.splitlines() if line.startswith("UNCOVERED\t")],
            ["UNCOVERED\tsrc/a-owned.txt", "UNCOVERED\tsrc/z-owned.txt"],
        )

        permissive = self.fixture.make_document()
        permissive["coverage"]["allow_uncovered"] = True
        self.assert_schema_failure(
            permissive, "coverage.allow_uncovered must be false"
        )

    def test_object_ids_and_digests_require_full_lowercase_hex(self) -> None:
        mutations = [
            (
                "short source commit",
                lambda document: document["sources"][0]["reviewed"].__setitem__(
                    "commit", "abc123"
                ),
                "reviewed.commit must be a lowercase full 40-hex",
            ),
            (
                "uppercase tree",
                lambda document: document["patch_stack"]["baseline"].__setitem__(
                    "tree", "A" * 40
                ),
                "baseline.tree must be a lowercase full 40-hex",
            ),
            (
                "short digest",
                lambda document: document["patch_stack"]["history"]["exact"].__setitem__(
                    "series_sha256", "abc123"
                ),
                "series_sha256 must be a lowercase full 64-hex",
            ),
            (
                "uppercase digest",
                lambda document: document["patch_stack"]["history"]["exact"][
                    "commits"
                ][0].__setitem__("delta_sha256", "A" * 64),
                "delta_sha256 must be a lowercase full 64-hex",
            ),
        ]
        for label, mutate, expected in mutations:
            with self.subTest(label=label):
                document = self.fixture.make_document()
                mutate(document)
                self.assert_schema_failure(document, expected)

    def test_controls_nul_formatting_and_surrogates_are_rejected(self) -> None:
        mutations = [
            (
                "nul",
                lambda document: document["sources"][0].__setitem__(
                    "project", "Fixture\x00Source"
                ),
                "must not contain controls",
            ),
            (
                "formatting",
                lambda document: document["patch_stack"]["history"].__setitem__(
                    "delta_format", "git\u202etree-delta-v1"
                ),
                "must not contain controls",
            ),
            (
                "surrogate",
                lambda document: document["features"][0].__setitem__(
                    "description", "unsafe \ud800 surrogate"
                ),
                "must not contain controls",
            ),
            (
                "paragraph separator",
                lambda document: document["features"][0].__setitem__(
                    "description", "unsafe\u2029separator"
                ),
                "must not contain controls",
            ),
        ]
        for label, mutate, expected in mutations:
            with self.subTest(label=label):
                document = self.fixture.make_document()
                mutate(document)
                self.assert_schema_failure(document, expected)

    def test_source_feature_and_attestation_identities_are_unique(self) -> None:
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

        duplicate_attestation = self.fixture.make_document()
        duplicate_attestation["patch_stack"]["history"][
            "thematic_equivalents"
        ].append(
            copy.deepcopy(
                duplicate_attestation["patch_stack"]["history"][
                    "thematic_equivalents"
                ][0]
            )
        )
        self.assert_schema_failure(duplicate_attestation, "duplicate ID")
        self.assert_schema_failure(duplicate_attestation, "duplicate candidate commit")

    def test_unsafe_paths_and_globs_are_rejected(self) -> None:
        unsafe_patterns = [
            ("../escape", "'..' path components"),
            ("/absolute", "repository-relative"),
            ("src\\windows.txt", "not backslashes"),
            ("src/\u202efile.txt", "controls, NUL, formatting"),
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

    def test_missing_integration_and_legal_anchors_are_fatal(self) -> None:
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

    def test_manifest_commands_are_never_executed(self) -> None:
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
            '{"schema_version": 2, "schema_version": 2}\n', encoding="utf-8"
        )
        result = self.fixture.run_checker()
        self.assertEqual(result.returncode, 1)
        self.assertIn("duplicate JSON key", result.stdout)

    def test_upstream_remote_ref_and_revisions_are_cross_checked_exactly(self) -> None:
        mutations = [
            (
                {"remote": "https://example.invalid/other.git"},
                "remote mismatch",
            ),
            ({"tracked_ref": "release"}, "tracked ref mismatch"),
            ({"latest": "f" * 40}, "latest-fetched revision mismatch"),
        ]
        for values, expected in mutations:
            with self.subTest(expected=expected):
                self.fixture.write_upstream_versions(**values)
                result = self.fixture.run_checker()
                self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
                self.assertIn("ERROR upstream-revisions:", result.stdout)
                self.assertIn(expected, result.stdout)
                self.fixture.write_upstream_versions()

    def test_external_tree_claims_are_rejected_not_reported_as_verified(self) -> None:
        document = self.fixture.make_document()
        document["sources"][0]["reviewed"]["tree"] = self.fixture.baseline_tree
        self.assert_schema_failure(document, "reviewed contains unsupported key 'tree'")

        self.fixture.reset_document()
        self.fixture.write_manifest()
        result = self.fixture.run_checker()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("no external tree mappings claimed", result.stdout)
        self.assertNotIn("external tree verified", result.stdout.lower())

    def test_ledger_rejects_uppercase_and_inconsistent_link_revisions(self) -> None:
        self.fixture.write_upstream_versions(reviewed=self.fixture.baseline.upper())
        uppercase = self.fixture.run_checker()
        self.assertEqual(uppercase.returncode, 1)
        self.assertIn("must use lowercase full 40-hex IDs", uppercase.stdout)

        self.fixture.write_upstream_versions()
        text = self.fixture.upstream_path.read_text(encoding="utf-8")
        text = text.replace(
            f"commit/{self.fixture.baseline}", f"commit/{'e' * 40}", 1
        )
        self.fixture.upstream_path.write_text(text, encoding="utf-8")
        inconsistent = self.fixture.run_checker()
        self.assertEqual(inconsistent.returncode, 1)
        self.assertIn("contains inconsistent revision IDs", inconsistent.stdout)

    def test_tree_and_ordered_history_attestations_are_immutable(self) -> None:
        mutations = [
            (
                "record tree",
                lambda document: document["patch_stack"]["history"]["exact"][
                    "commits"
                ][0].__setitem__("tree", "0" * 40),
                "ordered commit/tree/delta record 0 mismatch",
            ),
            (
                "delta digest",
                lambda document: document["patch_stack"]["history"]["exact"][
                    "commits"
                ][0].__setitem__("delta_sha256", "0" * 64),
                "ordered commit/tree/delta record 0 mismatch",
            ),
            (
                "series digest",
                lambda document: document["patch_stack"]["history"]["exact"].__setitem__(
                    "series_sha256", "0" * 64
                ),
                "series_sha256 mismatch",
            ),
            (
                "commit count",
                lambda document: (
                    document["patch_stack"]["history"]["exact"]["commits"].pop(0),
                    document["patch_stack"]["history"]["exact"].__setitem__(
                        "commit_count", 1
                    ),
                ),
                "history commit count mismatch",
            ),
        ]
        for label, mutate, expected in mutations:
            with self.subTest(label=label):
                document = self.fixture.make_document()
                mutate(document)
                self.fixture.write_manifest(document)
                result = self.fixture.run_checker()
                self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
                self.assertIn(expected, result.stdout)

        wrong_frozen_tree = self.fixture.make_document()
        wrong_frozen_tree["patch_stack"]["frozen_tip"]["tree"] = "0" * 40
        self.assert_schema_failure(
            wrong_frozen_tree, "candidate.tree must equal patch_stack.frozen_tip.tree"
        )

        reordered = self.fixture.make_document()
        reordered["patch_stack"]["history"]["exact"]["commits"].reverse()
        self.assert_schema_failure(reordered, "final commit record must equal")

    def test_net_zero_add_delete_history_is_rejected(self) -> None:
        candidate = self.fixture.create_net_zero_history()
        document = self.fixture.make_document()
        document["patch_stack"]["history"]["thematic_equivalents"].append(
            self.fixture.attestation_for(
                candidate, attestation_id="fixture-net-zero-history"
            )
        )
        self.fixture.write_manifest(document)

        result = self.fixture.run_checker()
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("net-zero intermediate churn", result.stdout)
        self.assertIn("src/transient.txt", result.stdout)

    def test_empty_commit_history_is_rejected(self) -> None:
        candidate = self.fixture.create_empty_history()
        document = self.fixture.make_document()
        document["patch_stack"]["history"]["thematic_equivalents"].append(
            self.fixture.attestation_for(
                candidate, attestation_id="fixture-empty-history"
            )
        )
        self.fixture.write_manifest(document)

        result = self.fixture.run_checker()
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("is empty because its tree equals its parent tree", result.stdout)

    def test_side_parent_merge_history_is_rejected(self) -> None:
        candidate = self.fixture.create_merge_history()
        document = self.fixture.make_document()
        document["patch_stack"]["history"]["thematic_equivalents"].append(
            self.fixture.synthetic_merge_attestation(
                candidate, attestation_id="fixture-merge-history"
            )
        )
        self.fixture.write_manifest(document)

        result = self.fixture.run_checker()
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("side-parent merges are forbidden", result.stdout)

    def test_deprecated_grafts_cannot_forge_baseline_parentage(self) -> None:
        candidate = self.fixture.commit_tree(
            self.fixture.tip_tree, [], "fixture unrelated root with frozen tree"
        )
        grafts = self.fixture.repo / ".git" / "info" / "grafts"
        grafts.write_text(
            f"{candidate} {self.fixture.baseline}\n", encoding="ascii"
        )
        document = self.fixture.make_document()
        document["patch_stack"]["history"]["thematic_equivalents"].append(
            self.fixture.synthetic_merge_attestation(
                candidate, attestation_id="fixture-forged-graft-history"
            )
        )
        self.fixture.write_manifest(document)

        result = self.fixture.run_checker()
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("reached root commit", result.stdout)

    def test_checkout_integration_symlink_is_rejected(self) -> None:
        self.fixture.symlink_repo_path("src/integration.txt", "../LICENSE")
        result = self.fixture.run_checker()
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("integration_paths entry 'src/integration.txt' is a symlink", result.stdout)

    def test_checkout_legal_symlink_is_rejected(self) -> None:
        self.fixture.symlink_repo_path("LICENSE", "src/integration.txt")
        result = self.fixture.run_checker()
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("legal_paths entry 'LICENSE' is a symlink", result.stdout)

    def test_frozen_integration_symlink_mode_is_rejected(self) -> None:
        self.fixture.adopt_symlink_frozen_tip("src/integration.txt", "../LICENSE")
        self.fixture.write_repo_file("src/integration.txt", "downstream\n")
        result = self.fixture.run_checker()
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("integration_paths entry 'src/integration.txt' is a symlink at the frozen tip", result.stdout)

    def test_frozen_legal_symlink_mode_is_rejected(self) -> None:
        self.fixture.adopt_symlink_frozen_tip("LICENSE", "src/integration.txt")
        self.fixture.write_repo_file("LICENSE", "fixture license\n")
        result = self.fixture.run_checker()
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("legal_paths entry 'LICENSE' is a symlink at the frozen tip", result.stdout)

    def test_hostile_diff_config_and_attributes_cannot_change_tree_proofs(self) -> None:
        sentinel = self.fixture.root / "external-diff-ran"
        hostile = self.fixture.root / "hostile-diff.sh"
        hostile.write_text(
            f"#!/bin/sh\n: > '{sentinel}'\nexit 97\n", encoding="utf-8"
        )
        hostile.chmod(0o755)
        self.fixture.git("config", "diff.external", str(hostile))
        self.fixture.git("config", "diff.hostile.command", str(hostile))
        self.fixture.git("config", "diff.hostile.textconv", str(hostile))
        self.fixture.git("config", "filter.hostile.clean", str(hostile))
        self.fixture.git("config", "filter.hostile.smudge", str(hostile))
        self.fixture.git("config", "diff.renames", "copies")
        self.fixture.write_repo_file(
            ".gitattributes", "* diff=hostile filter=hostile -text\n"
        )
        before = self.fixture.repository_state()

        result = self.fixture.run_checker(
            extra_env={"GIT_EXTERNAL_DIFF": str(hostile), "LANG": "en_US.UTF-8"}
        )
        after = self.fixture.repository_state()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("OK patch-stack-history:", result.stdout)
        self.assertFalse(sentinel.exists())
        self.assertEqual(before, after)

    def test_hostile_cwd_and_pythonpath_are_isolated_without_bytecode(self) -> None:
        hostile_cwd = self.fixture.root / "hostile-cwd"
        hostile_cwd.mkdir()
        hostile_modules = self.fixture.root / "hostile-pythonpath"
        hostile_modules.mkdir()
        sentinel = self.fixture.root / "pythonpath-ran"
        payload = (
            "from pathlib import Path\n"
            f"Path({str(sentinel)!r}).write_text('ran', encoding='utf-8')\n"
        )
        (hostile_modules / "sitecustomize.py").write_text(payload, encoding="utf-8")
        (hostile_modules / "json.py").write_text(payload, encoding="utf-8")
        before = self.fixture.repository_state()
        bytecode_before = self.fixture.bytecode_artifacts()

        result = self.fixture.run_checker(
            cwd=hostile_cwd,
            extra_env={
                "PYTHONPATH": str(hostile_modules),
                "PYTHONHOME": str(hostile_modules),
            },
        )
        bytecode_after = self.fixture.bytecode_artifacts()
        after = self.fixture.repository_state()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertFalse(sentinel.exists())
        self.assertEqual(before, after)
        self.assertEqual(bytecode_before, bytecode_after)

    def test_missing_feature_pattern_match_is_fatal(self) -> None:
        document = self.fixture.make_document()
        document["features"][0]["paths"].append("src/not-in-diff.txt")
        self.fixture.write_manifest(document)
        result = self.fixture.run_checker()
        self.assertEqual(result.returncode, 1)
        self.assertIn("path pattern matches no downstream path", result.stdout)


if __name__ == "__main__":
    unittest.main()
