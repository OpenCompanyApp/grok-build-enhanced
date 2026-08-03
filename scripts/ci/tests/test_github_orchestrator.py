#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import io
import json
import sys
import unittest
import urllib.request
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

orchestrator_spec = importlib.util.spec_from_file_location(
    "github_orchestrator", ROOT / "scripts/ci/github_orchestrator.py"
)
assert orchestrator_spec and orchestrator_spec.loader
orchestrator = importlib.util.module_from_spec(orchestrator_spec)
sys.modules[orchestrator_spec.name] = orchestrator
orchestrator_spec.loader.exec_module(orchestrator)


class FakeClient:
    def __init__(self, release: dict[str, object]) -> None:
        self.release = release

    def json(self, method: str, path: str, *, payload: object = None) -> object:
        self.last = (method, path, payload)
        return self.release


class OrchestratorTests(unittest.TestCase):
    def test_exact_repository_is_required(self) -> None:
        self.assertEqual(
            orchestrator.validate_repository(orchestrator.REPOSITORY),
            orchestrator.REPOSITORY,
        )
        with self.assertRaisesRegex(orchestrator.OrchestrationError, "exactly"):
            orchestrator.validate_repository("xai-org/grok-build")
        self.assertEqual(orchestrator.validate_workflow_ref("main"), "main")
        with self.assertRaisesRegex(orchestrator.OrchestrationError, "exactly main"):
            orchestrator.validate_workflow_ref("rebase/unreviewed")

    def test_source_and_orchestration_inputs_are_strict(self) -> None:
        sha, branch = orchestrator.validate_source("a" * 40, "rebase/upstream-1")
        self.assertEqual(sha, "a" * 40)
        self.assertEqual(branch, "rebase/upstream-1")
        self.assertEqual(orchestrator.validate_orchestration_id("woodpecker-q-12"), "woodpecker-q-12")
        for invalid in ("../main", "rebase//bad", "feature/not-allowed"):
            with self.assertRaises(orchestrator.OrchestrationError):
                orchestrator.validate_source("a" * 40, invalid)
        with self.assertRaises(orchestrator.OrchestrationError):
            orchestrator.validate_source("A" * 40, "main")
        with self.assertRaises(orchestrator.OrchestrationError):
            orchestrator.validate_orchestration_id("bad id")

    def test_release_tag_validation(self) -> None:
        self.assertEqual(orchestrator.validate_release_tag("v1.2.3-rc.1"), "v1.2.3-rc.1")
        for invalid in ("1.2.3", "v01.2.3", "v1.2"):
            with self.assertRaises(orchestrator.OrchestrationError):
                orchestrator.validate_release_tag(invalid)

    def test_qualification_zip_must_contain_one_regular_json_file(self) -> None:
        record = {"schema_version": 1}
        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, "w") as archive:
            archive.writestr("qualification.json", json.dumps(record))
        self.assertEqual(orchestrator.qualification_from_zip(buffer.getvalue()), record)

        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, "w") as archive:
            archive.writestr("qualification.json", "{}")
            archive.writestr("extra.txt", "unexpected")
        with self.assertRaisesRegex(
            orchestrator.OrchestrationError, "unexpected file set"
        ):
            orchestrator.qualification_from_zip(buffer.getvalue())

        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, "w") as archive:
            symlink = zipfile.ZipInfo("qualification.json")
            symlink.create_system = 3
            symlink.external_attr = 0o120777 << 16
            archive.writestr(symlink, "target")
        with self.assertRaisesRegex(orchestrator.OrchestrationError, "regular file"):
            orchestrator.qualification_from_zip(buffer.getvalue())

    def test_release_asset_allowlist_is_exact(self) -> None:
        tag = "v1.2.3"
        expected = [
            {"name": template.format(version="1.2.3")}
            for template in orchestrator.EXPECTED_RELEASE_ASSET_TEMPLATES
        ]
        client = FakeClient({"tag_name": tag, "draft": False, "assets": expected})
        orchestrator.verify_release(client, orchestrator.REPOSITORY, tag)

        client = FakeClient(
            {"tag_name": tag, "draft": False, "assets": expected + [{"name": "unexpected"}]}
        )
        with self.assertRaisesRegex(orchestrator.OrchestrationError, "allowlist"):
            orchestrator.verify_release(client, orchestrator.REPOSITORY, tag)

        duplicated = expected[:-1] + [expected[0]]
        client = FakeClient({"tag_name": tag, "draft": False, "assets": duplicated})
        with self.assertRaisesRegex(orchestrator.OrchestrationError, "allowlist"):
            orchestrator.verify_release(client, orchestrator.REPOSITORY, tag)

    def test_redirects_strip_auth_cross_origin_and_reject_https_downgrade(self) -> None:
        request = urllib.request.Request(
            "https://api.github.com/source",
            headers={
                "Authorization": "Bearer secret",
                "X-GitHub-Api-Version": "2022-11-28",
            },
        )
        handler = orchestrator.SafeRedirectHandler()
        redirected = handler.redirect_request(
            request,
            None,
            302,
            "Found",
            {},
            "https://objects.githubusercontent.com/artifact",
        )
        self.assertIsNotNone(redirected)
        self.assertIsNone(redirected.get_header("Authorization"))
        self.assertIsNone(redirected.get_header("X-Github-Api-Version"))

        with self.assertRaisesRegex(orchestrator.OrchestrationError, "HTTPS"):
            handler.redirect_request(
                request,
                None,
                302,
                "Found",
                {},
                "http://api.github.com/insecure",
            )

    def test_missing_token_is_rejected_without_echo(self) -> None:
        with self.assertRaisesRegex(orchestrator.OrchestrationError, "missing"):
            orchestrator.GitHubClient("")


if __name__ == "__main__":
    unittest.main()
