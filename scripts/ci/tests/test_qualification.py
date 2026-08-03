#!/usr/bin/env python3
from __future__ import annotations

import copy
import importlib.util
import subprocess
import sys
import unittest
from pathlib import Path
from unittest import mock

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location(
    "ci_qualification", ROOT / "scripts/ci/qualification.py"
)
assert SPEC and SPEC.loader
qualification = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = qualification
SPEC.loader.exec_module(qualification)

COMMIT = "1" * 40
TREE = "2" * 40
DIGEST = "3" * 64


def valid_record() -> dict[str, object]:
    return {
        "created_at": "2026-08-02T12:00:00Z",
        "inputs": {"cargo_lock_sha256": DIGEST, "release_builds": False},
        "repository": qualification.REPOSITORY,
        "results": {
            "static": "success",
            "rust_contracts": "success",
            "broad": "success",
            "native": "success",
        },
        "schema_version": qualification.SCHEMA_VERSION,
        "source": {"commit": COMMIT, "tree": TREE},
        "targets": sorted(qualification.ALLOWED_TARGETS),
        "toolchain": {
            "rustc": "rustc 1.92.0",
            "commit-hash": "4" * 40,
            "host": "x86_64-unknown-linux-gnu",
            "release": "1.92.0",
        },
        "workflow": {
            "path": qualification.DEFAULT_WORKFLOW,
            "run_attempt": 1,
            "run_id": 42,
            "sha256": DIGEST,
        },
    }


class QualificationTests(unittest.TestCase):
    def test_valid_record_is_accepted(self) -> None:
        record = valid_record()
        self.assertIs(
            qualification.validate_record(
                record,
                source_sha=COMMIT,
                source_tree=TREE,
                workflow_digest=DIGEST,
                run_id=42,
                run_attempt=1,
                release_builds=False,
            ),
            record,
        )

    def test_wrong_source_is_rejected(self) -> None:
        with self.assertRaisesRegex(
            qualification.QualificationError, "source commit does not match"
        ):
            qualification.validate_record(valid_record(), source_sha="a" * 40)

    def test_non_success_result_is_rejected(self) -> None:
        record = copy.deepcopy(valid_record())
        record["results"]["native"] = "failure"  # type: ignore[index]
        with self.assertRaisesRegex(
            qualification.QualificationError, "non-success result"
        ):
            qualification.validate_record(record)

    def test_incomplete_native_matrix_is_rejected(self) -> None:
        record = copy.deepcopy(valid_record())
        record["targets"] = ["x86_64-unknown-linux-gnu"]
        with self.assertRaisesRegex(
            qualification.QualificationError, "target matrix is incomplete"
        ):
            qualification.validate_record(record)

    def test_workflow_drift_is_rejected(self) -> None:
        with self.assertRaisesRegex(
            qualification.QualificationError, "does not match checkout"
        ):
            qualification.validate_record(
                valid_record(), workflow_digest="f" * 64
            )

    def test_run_and_release_inputs_must_match_request(self) -> None:
        for arguments, message in (
            ({"run_id": 43}, "run ID does not match"),
            ({"run_attempt": 2}, "run attempt does not match"),
            ({"release_builds": True}, "release-build flag does not match"),
        ):
            with self.subTest(arguments=arguments):
                with self.assertRaisesRegex(qualification.QualificationError, message):
                    qualification.validate_record(valid_record(), **arguments)

    def test_toolchain_and_timestamp_are_structurally_validated(self) -> None:
        record = copy.deepcopy(valid_record())
        record["toolchain"]["commit-hash"] = "not-a-hash"  # type: ignore[index]
        with self.assertRaisesRegex(qualification.QualificationError, "toolchain"):
            qualification.validate_record(record)

        record = copy.deepcopy(valid_record())
        record["created_at"] = "not-a-timestampZ"
        with self.assertRaisesRegex(qualification.QualificationError, "timestamp"):
            qualification.validate_record(record)

    def test_git_identity_ignores_replacement_objects(self) -> None:
        completed = subprocess.CompletedProcess(
            args=["git"],
            returncode=0,
            stdout="a" * 40 + "\n",
            stderr="",
        )
        with mock.patch.object(
            qualification.subprocess,
            "run",
            return_value=completed,
        ) as run:
            self.assertEqual(qualification.git_output(ROOT, "rev-parse", "HEAD"), "a" * 40)

        self.assertEqual(
            run.call_args.args[0][:2],
            ["git", "--no-replace-objects"],
        )

    def test_result_parser_requires_every_lane(self) -> None:
        with self.assertRaisesRegex(
            qualification.QualificationError, "missing qualification results"
        ):
            qualification.parse_results(["static=success"])


if __name__ == "__main__":
    unittest.main()
