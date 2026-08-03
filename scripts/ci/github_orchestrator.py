#!/usr/bin/env python3
"""Dispatch and verify exact Grok GitHub Actions runs from trusted Woodpecker CI."""

from __future__ import annotations

import argparse
import importlib.util
import io
import json
import os
import re
import ssl
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
import zipfile
from pathlib import Path
from typing import Any

QUALIFICATION_PATH = Path(__file__).resolve().with_name("qualification.py")
QUALIFICATION_SPEC = importlib.util.spec_from_file_location(
    "_grok_ci_qualification", QUALIFICATION_PATH
)
if QUALIFICATION_SPEC is None or QUALIFICATION_SPEC.loader is None:
    raise RuntimeError("could not load the qualification verifier")
qualification = importlib.util.module_from_spec(QUALIFICATION_SPEC)
sys.modules[QUALIFICATION_SPEC.name] = qualification
QUALIFICATION_SPEC.loader.exec_module(qualification)

API_ROOT = "https://api.github.com"
REPOSITORY = "OpenCompanyApp/grok-build-enhanced"
QUALIFICATION_WORKFLOW = "rebase-qualification.yml"
RELEASE_WORKFLOW = "release.yml"
SAFE_ID_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$")
BRANCH_RE = re.compile(r"^(?:main|rebase/[A-Za-z0-9._/-]+|refresh/[A-Za-z0-9._/-]+)$")
TAG_RE = re.compile(r"^v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$")
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
MAX_JSON_BYTES = 2 * 1024 * 1024
MAX_QUALIFICATION_ZIP_BYTES = 2 * 1024 * 1024
EXPECTED_RELEASE_ASSET_TEMPLATES = (
    "grok-{version}-linux-aarch64",
    "grok-{version}-linux-x86_64",
    "grok-{version}-macos-aarch64",
    "grok-{version}-macos-x86_64",
    "RELEASE-PROVENANCE.json",
    "SHA256SUMS",
)


class OrchestrationError(RuntimeError):
    """Raised when a remote run cannot be bound to the requested operation."""


class SafeRedirectHandler(urllib.request.HTTPRedirectHandler):
    """Strip GitHub API authorization before following a cross-host asset redirect."""

    def redirect_request(
        self,
        request: urllib.request.Request,
        file_pointer: Any,
        code: int,
        message: str,
        headers: Any,
        new_url: str,
    ) -> urllib.request.Request | None:
        redirected = super().redirect_request(request, file_pointer, code, message, headers, new_url)
        if redirected is None:
            return None
        old_url = urllib.parse.urlparse(request.full_url)
        new_url_parts = urllib.parse.urlparse(new_url)
        if new_url_parts.scheme != "https":
            raise OrchestrationError("GitHub redirect URL must use HTTPS")
        old_origin = (old_url.scheme, old_url.hostname, old_url.port)
        new_origin = (new_url_parts.scheme, new_url_parts.hostname, new_url_parts.port)
        if old_origin != new_origin:
            for name in ("Authorization", "X-GitHub-Api-Version"):
                redirected.remove_header(name)
                redirected.unredirected_hdrs.pop(name.lower(), None)
        return redirected


class GitHubClient:
    def __init__(self, token: str) -> None:
        if not token or any(character.isspace() for character in token):
            raise OrchestrationError("GITHUB_ACTIONS_TOKEN is missing or malformed")
        context = ssl.create_default_context()
        self.opener = urllib.request.build_opener(
            urllib.request.HTTPSHandler(context=context), SafeRedirectHandler()
        )
        self.token = token

    def request(
        self,
        method: str,
        path_or_url: str,
        *,
        payload: dict[str, Any] | None = None,
        accept: str = "application/vnd.github+json",
        max_bytes: int = MAX_JSON_BYTES,
    ) -> tuple[int, bytes]:
        url = path_or_url if path_or_url.startswith("https://") else f"{API_ROOT}{path_or_url}"
        parsed = urllib.parse.urlparse(url)
        if parsed.scheme != "https":
            raise OrchestrationError("GitHub request URL must use HTTPS")
        body = None
        if payload is not None:
            body = json.dumps(payload, separators=(",", ":")).encode("utf-8")
        headers = {
            "Accept": accept,
            "Authorization": f"Bearer {self.token}",
            "User-Agent": "grok-build-enhanced-woodpecker-orchestrator/1",
            "X-GitHub-Api-Version": "2022-11-28",
        }
        if body is not None:
            headers["Content-Type"] = "application/json"
        request = urllib.request.Request(url, data=body, headers=headers, method=method)
        try:
            with self.opener.open(request, timeout=60) as response:
                data = response.read(max_bytes + 1)
                if len(data) > max_bytes:
                    raise OrchestrationError("GitHub response exceeded the safe size limit")
                return response.status, data
        except urllib.error.HTTPError as error:
            # Never echo response bodies; API errors can contain user-controlled data.
            raise OrchestrationError(f"GitHub API request failed with HTTP {error.code}") from None
        except urllib.error.URLError as error:
            raise OrchestrationError(f"GitHub API transport failed: {error.reason}") from None

    def json(
        self,
        method: str,
        path: str,
        *,
        payload: dict[str, Any] | None = None,
    ) -> Any:
        _status, body = self.request(method, path, payload=payload)
        if not body:
            return None
        try:
            return json.loads(body)
        except json.JSONDecodeError as error:
            raise OrchestrationError("GitHub API returned malformed JSON") from error


def validate_repository(repository: str) -> str:
    if repository != REPOSITORY:
        raise OrchestrationError(f"repository must be exactly {REPOSITORY}")
    return repository


def validate_workflow_ref(workflow_ref: str) -> str:
    if workflow_ref != "main":
        raise OrchestrationError("workflow ref must be exactly main")
    return workflow_ref


def validate_orchestration_id(value: str) -> str:
    if not SAFE_ID_RE.fullmatch(value):
        raise OrchestrationError("orchestration ID must be 1-64 safe ASCII characters")
    return value


def validate_source(source_sha: str, source_branch: str) -> tuple[str, str]:
    if not SHA_RE.fullmatch(source_sha):
        raise OrchestrationError("source SHA must be 40 lowercase hexadecimal characters")
    if not BRANCH_RE.fullmatch(source_branch) or ".." in source_branch or "//" in source_branch:
        raise OrchestrationError("source branch must be main, rebase/*, or refresh/*")
    return source_sha, source_branch


def validate_release_tag(tag: str) -> str:
    if not TAG_RE.fullmatch(tag):
        raise OrchestrationError("release tag must be vX.Y.Z or vX.Y.Z-prerelease")
    return tag


def workflow_path(repository: str, workflow: str, suffix: str = "") -> str:
    encoded = urllib.parse.quote(workflow, safe="")
    return f"/repos/{repository}/actions/workflows/{encoded}{suffix}"


def dispatch_workflow(
    client: GitHubClient,
    repository: str,
    workflow: str,
    ref: str,
    inputs: dict[str, str | bool],
) -> None:
    client.request(
        "POST",
        workflow_path(repository, workflow, "/dispatches"),
        payload={"ref": ref, "inputs": inputs},
    )


def find_dispatched_run(
    client: GitHubClient,
    repository: str,
    workflow: str,
    expected_title: str,
    *,
    timeout_seconds: int,
    poll_interval: int,
) -> dict[str, Any]:
    deadline = time.monotonic() + timeout_seconds
    while time.monotonic() < deadline:
        query = urllib.parse.urlencode({"event": "workflow_dispatch", "per_page": 30})
        response = client.json("GET", workflow_path(repository, workflow, f"/runs?{query}"))
        runs = response.get("workflow_runs", []) if isinstance(response, dict) else []
        for run in runs:
            if (
                isinstance(run, dict)
                and run.get("display_title") == expected_title
                and run.get("event") == "workflow_dispatch"
                and str(run.get("path", "")).endswith(f"/{workflow}")
            ):
                run_id = run.get("id")
                if not isinstance(run_id, int) or run_id <= 0:
                    raise OrchestrationError("matched GitHub run has an invalid ID")
                print(f"github-run discovered id={run_id} status={run.get('status', 'unknown')}", flush=True)
                return run
        time.sleep(poll_interval)
    raise OrchestrationError("timed out waiting for the dispatched GitHub run to appear")


def wait_for_run(
    client: GitHubClient,
    repository: str,
    run_id: int,
    *,
    timeout_seconds: int,
    poll_interval: int,
) -> dict[str, Any]:
    deadline = time.monotonic() + timeout_seconds
    previous: tuple[Any, Any] | None = None
    while time.monotonic() < deadline:
        run = client.json("GET", f"/repos/{repository}/actions/runs/{run_id}")
        if not isinstance(run, dict):
            raise OrchestrationError("GitHub run response is malformed")
        state = (run.get("status"), run.get("conclusion"))
        if state != previous:
            print(
                f"github-run id={run_id} status={state[0]} conclusion={state[1] or 'pending'}",
                flush=True,
            )
            previous = state
        if state[0] == "completed":
            if state[1] != "success":
                raise OrchestrationError(f"GitHub run {run_id} completed with {state[1]}")
            return run
        time.sleep(poll_interval)
    raise OrchestrationError(f"timed out waiting for GitHub run {run_id}")


def commit_tree(client: GitHubClient, repository: str, source_sha: str) -> str:
    record = client.json("GET", f"/repos/{repository}/git/commits/{source_sha}")
    tree = record.get("tree", {}).get("sha") if isinstance(record, dict) else None
    if not isinstance(tree, str) or not SHA_RE.fullmatch(tree):
        raise OrchestrationError("GitHub commit response has no valid tree identity")
    return tree


def qualification_from_zip(data: bytes) -> dict[str, Any]:
    try:
        with zipfile.ZipFile(io.BytesIO(data)) as archive:
            members = archive.infolist()
            if len(members) != 1 or members[0].filename != "qualification.json":
                raise OrchestrationError("qualification artifact contains an unexpected file set")
            member = members[0]
            if member.file_size > MAX_JSON_BYTES or member.is_dir():
                raise OrchestrationError("qualification artifact member is invalid")
            unix_file_type = (member.external_attr >> 16) & 0o170000
            if unix_file_type not in (0, 0o100000):
                raise OrchestrationError("qualification artifact member must be a regular file")
            return json.loads(archive.read(member))
    except (zipfile.BadZipFile, json.JSONDecodeError) as error:
        raise OrchestrationError("qualification artifact is malformed") from error


def fetch_qualification(
    client: GitHubClient,
    repository: str,
    run_id: int,
    source_sha: str,
) -> dict[str, Any]:
    response = client.json("GET", f"/repos/{repository}/actions/runs/{run_id}/artifacts?per_page=100")
    artifacts = response.get("artifacts", []) if isinstance(response, dict) else []
    expected_name = f"rebase-qualification-{source_sha}"
    matches = [artifact for artifact in artifacts if isinstance(artifact, dict) and artifact.get("name") == expected_name]
    if len(matches) != 1 or matches[0].get("expired") is not False:
        raise OrchestrationError("exact qualification artifact is missing, duplicated, or expired")
    archive_url = matches[0].get("archive_download_url")
    if not isinstance(archive_url, str) or not archive_url.startswith(f"{API_ROOT}/"):
        raise OrchestrationError("qualification artifact download URL is invalid")
    _status, data = client.request(
        "GET",
        archive_url,
        accept="application/vnd.github+json",
        max_bytes=MAX_QUALIFICATION_ZIP_BYTES,
    )
    return qualification_from_zip(data)


def verify_release(client: GitHubClient, repository: str, tag: str) -> None:
    release = client.json("GET", f"/repos/{repository}/releases/tags/{urllib.parse.quote(tag, safe='')}")
    if not isinstance(release, dict) or release.get("tag_name") != tag:
        raise OrchestrationError("published GitHub Release does not match requested tag")
    if release.get("draft") is not False:
        raise OrchestrationError("published GitHub Release is still a draft")
    version = tag[1:]
    expected = {template.format(version=version) for template in EXPECTED_RELEASE_ASSET_TEMPLATES}
    assets = release.get("assets")
    if not isinstance(assets, list) or len(assets) != len(expected):
        raise OrchestrationError("published GitHub Release asset allowlist does not match")
    names = [asset.get("name") for asset in assets if isinstance(asset, dict)]
    if len(names) != len(expected) or len(set(names)) != len(expected) or set(names) != expected:
        raise OrchestrationError("published GitHub Release asset allowlist does not match")


def dispatch_qualification(args: argparse.Namespace, client: GitHubClient) -> None:
    repository = validate_repository(args.repository)
    workflow_ref = validate_workflow_ref(args.workflow_ref)
    orchestration_id = validate_orchestration_id(args.orchestration_id)
    source_sha, source_branch = validate_source(args.source_sha, args.source_branch)
    tree = commit_tree(client, repository, source_sha)
    title = f"Rebase qualification {source_sha} ({orchestration_id})"
    dispatch_workflow(
        client,
        repository,
        QUALIFICATION_WORKFLOW,
        workflow_ref,
        {
            "source_branch": source_branch,
            "source_sha": source_sha,
            "orchestration_id": orchestration_id,
            "confirm_qualification": True,
            "release_builds": args.release_builds,
        },
    )
    run = find_dispatched_run(
        client,
        repository,
        QUALIFICATION_WORKFLOW,
        title,
        timeout_seconds=args.discovery_timeout,
        poll_interval=args.poll_interval,
    )
    run = wait_for_run(
        client,
        repository,
        run["id"],
        timeout_seconds=args.run_timeout,
        poll_interval=args.poll_interval,
    )
    run_attempt = run.get("run_attempt")
    if not isinstance(run_attempt, int) or run_attempt <= 0:
        raise OrchestrationError("matched GitHub run has an invalid attempt number")
    record = fetch_qualification(client, repository, run["id"], source_sha)
    qualification.validate_record(
        record,
        source_sha=source_sha,
        source_tree=tree,
        repository=repository,
        workflow_digest=qualification.sha256_file(Path(qualification.DEFAULT_WORKFLOW)),
        run_id=run["id"],
        run_attempt=run_attempt,
        release_builds=args.release_builds,
    )
    print(f"qualification verified source={source_sha} tree={tree} run={run['id']}")


def dispatch_release(args: argparse.Namespace, client: GitHubClient) -> None:
    repository = validate_repository(args.repository)
    workflow_ref = validate_workflow_ref(args.workflow_ref)
    orchestration_id = validate_orchestration_id(args.orchestration_id)
    tag = validate_release_tag(args.release_tag)
    title = f"Grok Build Enhanced release {tag} ({orchestration_id})"
    dispatch_workflow(
        client,
        repository,
        RELEASE_WORKFLOW,
        workflow_ref,
        {
            "release_tag": tag,
            "orchestration_id": orchestration_id,
            "confirm_release": True,
        },
    )
    run = find_dispatched_run(
        client,
        repository,
        RELEASE_WORKFLOW,
        title,
        timeout_seconds=args.discovery_timeout,
        poll_interval=args.poll_interval,
    )
    run = wait_for_run(
        client,
        repository,
        run["id"],
        timeout_seconds=args.run_timeout,
        poll_interval=args.poll_interval,
    )
    verify_release(client, repository, tag)
    print(f"release verified tag={tag} run={run['id']}")


def add_common_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--repository", default=REPOSITORY)
    parser.add_argument("--workflow-ref", default="main")
    parser.add_argument("--orchestration-id", required=True)
    parser.add_argument("--poll-interval", type=int, default=20)
    parser.add_argument("--discovery-timeout", type=int, default=180)
    parser.add_argument("--run-timeout", type=int, default=6 * 60 * 60)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    qualification_parser = subparsers.add_parser("dispatch-qualification")
    add_common_arguments(qualification_parser)
    qualification_parser.add_argument("--source-sha", required=True)
    qualification_parser.add_argument("--source-branch", required=True)
    qualification_parser.add_argument("--release-builds", action="store_true")
    qualification_parser.set_defaults(handler=dispatch_qualification)

    release_parser = subparsers.add_parser("dispatch-release")
    add_common_arguments(release_parser)
    release_parser.add_argument("--release-tag", required=True)
    release_parser.set_defaults(handler=dispatch_release)
    return parser


def main() -> int:
    args = build_parser().parse_args()
    if args.poll_interval < 5 or args.discovery_timeout < 30 or args.run_timeout < 60:
        print("error: orchestration timeouts are below safe minimums", file=sys.stderr)
        return 2
    try:
        client = GitHubClient(os.environ.get("GITHUB_ACTIONS_TOKEN", ""))
        args.handler(args, client)
    except (OSError, OrchestrationError, qualification.QualificationError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
