#!/usr/bin/env python3
from __future__ import annotations

import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
WORKFLOW_DIR = ROOT / ".github/workflows"
WOODPECKER_DIR = ROOT / ".woodpecker"
ACTION_SHA_RE = re.compile(r"^[0-9a-f]{40}$")


class CiContractTests(unittest.TestCase):
    def test_github_workflows_are_read_only_by_default_and_sha_pinned(self) -> None:
        for path in sorted(WORKFLOW_DIR.glob("*.yml")):
            text = path.read_text(encoding="utf-8")
            self.assertNotIn("pull_request_target", text, path.name)
            self.assertIn("permissions:\n  contents: read\n", text, path.name)
            for action, revision in re.findall(
                r"^\s*uses:\s*([^@\s]+)(?:@([^\s#]+))?", text, re.MULTILINE
            ):
                if action.startswith("./"):
                    continue
                self.assertIsNotNone(revision, f"{path.name}: {action} is unpinned")
                self.assertRegex(
                    revision or "", ACTION_SHA_RE, f"{path.name}: {action} is not SHA-pinned"
                )

        composite = (ROOT / ".github/actions/setup-rust-ci/action.yml").read_text(
            encoding="utf-8"
        )
        for action, revision in re.findall(
            r"^\s*uses:\s*([^@\s]+)@([^\s#]+)", composite, re.MULTILINE
        ):
            self.assertRegex(revision, ACTION_SHA_RE, f"composite action: {action}")

    def test_pr_and_qualification_workflows_do_not_consume_repository_secrets(self) -> None:
        for name in ("fork-contracts.yml", "deep-ci.yml", "rebase-qualification.yml"):
            text = (WORKFLOW_DIR / name).read_text(encoding="utf-8")
            self.assertNotIn("secrets.", text, name)
            self.assertNotIn("pull_request_target", text, name)
            self.assertNotIn("self-hosted", text, name)

    def test_cache_contract_never_persists_targets_or_auth_state(self) -> None:
        action = (ROOT / ".github/actions/setup-rust-ci/action.yml").read_text(
            encoding="utf-8"
        )
        cache_paths = action.partition("path: |")[2].partition("key:")[0]
        self.assertNotRegex(cache_paths, r"(?:^|/)target(?:/|$)")
        for forbidden in (".grok", ".codex", "auth.json", ".secrets"):
            self.assertNotIn(forbidden, cache_paths)
        self.assertIn("~/.cargo/registry/cache", cache_paths)
        self.assertIn("SCCACHE_GHA_ENABLED", (WORKFLOW_DIR / "fork-contracts.yml").read_text())

    def test_exact_rebase_workflow_binds_source_and_emits_evidence(self) -> None:
        text = (WORKFLOW_DIR / "rebase-qualification.yml").read_text(
            encoding="utf-8"
        )
        for required in (
            "confirm_qualification:",
            "orchestration_id:",
            "ref: ${{ inputs.source_sha }}",
            'git merge-base --is-ancestor "$REQUESTED_SHA"',
            "scripts/ci/qualification.py create",
            "rebase-qualification-${{ needs.prepare.outputs.source_sha }}",
            "x86_64-unknown-linux-gnu",
            "aarch64-unknown-linux-gnu",
            "aarch64-apple-darwin",
            "x86_64-apple-darwin",
        ):
            self.assertIn(required, text)

    def test_pull_request_ci_uses_impact_selected_groups_and_manual_native_escape_hatch(self) -> None:
        text = (WORKFLOW_DIR / "fork-contracts.yml").read_text(encoding="utf-8")
        self.assertIn("fromJSON(needs.static-contracts.outputs.broad_groups)", text)
        self.assertIn('base_sha=$(git rev-parse "$HEAD_SHA^"', text)
        self.assertIn("always() &&", text)
        self.assertIn("needs.rust-contracts.result == 'skipped'", text)
        self.assertIn("inputs.full_native == true", text)

    def test_deep_ci_has_cached_and_cold_paths(self) -> None:
        text = (WORKFLOW_DIR / "deep-ci.yml").read_text(encoding="utf-8")
        self.assertIn("schedule:", text)
        self.assertIn("cold-composed-check:", text)
        self.assertIn("SCCACHE_GHA_ENABLED: 'false'", text)
        self.assertIn("scripts/ci/run.sh policy", text)

    def test_woodpecker_orchestration_is_deployment_only(self) -> None:
        expected_secret = "grok_github_actions_token"
        for path in sorted(WOODPECKER_DIR.glob("*.yml")):
            text = path.read_text(encoding="utf-8")
            self.assertIn("event: deployment", text, path.name)
            self.assertIn("branch: main", text, path.name)
            self.assertNotIn("event: pull_request", text, path.name)
            self.assertNotIn("event: push", text, path.name)
            self.assertEqual(text.count("from_secret:"), 1, path.name)
            self.assertIn(expected_secret, text, path.name)
            self.assertNotIn("contents: write", text, path.name)
            self.assertNotIn("terraform", text.lower(), path.name)
            self.assertRegex(
                text,
                r"image: python:3\.14-alpine@sha256:[0-9a-f]{64}",
                path.name,
            )

    def test_release_workflow_accepts_safe_orchestration_correlation(self) -> None:
        text = (WORKFLOW_DIR / "release.yml").read_text(encoding="utf-8")
        self.assertIn("orchestration_id:", text)
        self.assertIn("inputs.orchestration_id", text)
        self.assertIn("orchestration_id must be 1-64 safe ASCII characters", text)
        self.assertIn("inputs.confirm_release == true", text)

    def test_local_ci_entrypoint_exposes_reviewed_groups(self) -> None:
        text = (ROOT / "scripts/ci/run.sh").read_text(encoding="utf-8")
        for group in (
            "static",
            "rust-contracts",
            "core",
            "ui",
            "pty",
            "libraries",
            "policy",
            "native",
        ):
            self.assertIn(group, text)
        self.assertIn("CARGO_INCREMENTAL", text)
        self.assertIn("cargo check --locked -p xai-grok-pager-bin", text)


if __name__ == "__main__":
    unittest.main()
