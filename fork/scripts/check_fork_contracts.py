#!/usr/bin/env python3
"""Credential-free static contracts for Grok Build Enhanced.

This checker intentionally covers fork identity, provider isolation sentinels,
Warp provenance, updater routing, generated-workspace discipline, workflow
pinning, and tracked secret-file hygiene. It does not impose telemetry or
privacy product policy.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MANIFEST_PATH = ROOT / "fork/manifest.json"
WARP_REVISION = "b385044250f1ed3c9379ab34a8fe82f02fdffaa4"
ACTION_SHA = re.compile(r"^[0-9a-f]{40}$")


class ContractError(RuntimeError):
    """Raised when a fork contract no longer holds."""


def read(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


def require_fragment(relative: str, fragment: str) -> None:
    if fragment not in read(relative):
        raise ContractError(f"{relative} is missing required contract {fragment!r}")


def check_branding_and_charter() -> None:
    require_fragment("README.md", "# Grok Build Enhanced")
    require_fragment(
        "README.md", "The unofficial daily-driver fork of Grok Build."
    )
    require_fragment("AGENTS.md", "## Fork vision")
    require_fragment("AGENTS.md", "## Upstream and inspiration sources")
    require_fragment("AGENTS.md", "fork/manifest.json")
    for identity in (
        "c1b5909ec707c069f1d21a93917af044e71da0d7",
        "da3076874292c538bfc4efeede0cf517c2e975f0",
        "7ed3320c797852ed5894f1fc2fca7cee6827768f",
        "8adf9013a0929e5c7f1d4e849492d2387837a28d",
    ):
        require_fragment("AGENTS.md", identity)
    require_fragment(
        "crates/codegen/xai-grok-version/src/lib.rs",
        'pub const ENHANCED_PRODUCT_NAME: &str = "Grok Build Enhanced";',
    )
    require_fragment(
        "crates/codegen/xai-grok-version/src/lib.rs",
        'pub const UPSTREAM_BASE_VERSION: &str = env!("CARGO_PKG_VERSION");',
    )
    require_fragment(
        "crates/codegen/xai-grok-pager/docs/user-guide/01-getting-started.md",
        "OpenCompanyApp/grok-build-enhanced",
    )
    require_fragment(
        "crates/codegen/xai-grok-pager/npm/grok/README.md",
        "**not** a Grok Build Enhanced distribution route",
    )


def check_provider_isolation_sentinels() -> None:
    sentinels = {
        "crates/codegen/xai-grok-sampler/src/client.rs":
            "codex_rejects_static_or_generic_header_credentials",
        "crates/codegen/xai-grok-tools/src/types/api_key_provider.rs":
            "codex_auth_resolution_rejects_generic_static_key_fallback",
        "crates/codegen/xai-grok-tools/src/implementations/grok_build/video_gen/mod.rs":
            "authorization_header_values_are_always_sensitive",
        "crates/codegen/xai-grok-shell/src/agent/config.rs":
            "custom_credentials_never_inherit_xai_session_or_global_keys",
        "crates/codegen/xai-grok-shell/src/session/provider/openai_codex.rs":
            "custom_runtime_drops_xai_credentials_and_generic_tool_auth",
        "crates/codegen/xai-grok-shell/src/session/acp_session_tests/auth_error_no_retry_tests.rs":
            "reconstruct_full_config_drops_cross_provider_custom_credential",
        "crates/codegen/xai-grok-shell/src/session/acp_session_impl/session_setup.rs":
            "idle_model_metadata_refresh_requires_xai_provider_identity",
        "crates/codegen/xai-grok-pager/docs/user-guide/11-custom-models.md":
            'provider = "custom"',
        "crates/codegen/xai-grok-shell/src/session/acp_session_tests/media_gen_auth_retry_tests.rs":
            "provider_owned_codex_401_never_falls_through_to_xai_auth_manager",
        "crates/codegen/xai-grok-shell/src/agent/models.rs":
            "provider_scoped_fallback_never_crosses_from_codex_to_xai",
    }
    for path, test_name in sentinels.items():
        require_fragment(path, test_name)


def check_warp_lock(manifest: dict[str, object]) -> None:
    sources = {
        source["id"]: source
        for source in manifest.get("sources", [])
        if isinstance(source, dict) and "id" in source
    }
    warp = sources.get("warp-themes")
    if not isinstance(warp, dict):
        raise ContractError("fork manifest is missing the warp-themes source")
    reviewed = warp.get("reviewed")
    latest = warp.get("latest_fetched")
    if not isinstance(reviewed, dict) or reviewed.get("commit") != WARP_REVISION:
        raise ContractError("Warp reviewed revision changed without updating the lock")
    if not isinstance(latest, dict) or latest.get("commit") != WARP_REVISION:
        raise ContractError("Warp latest-fetched revision differs from the reviewed lock")
    for path in (
        "UPSTREAM_VERSIONS.md",
        "crates/codegen/xai-grok-pager-render/assets/warp-themes/UPSTREAM_REVISION",
        "crates/codegen/xai-grok-pager-render/assets/warp-themes/VENDOR_MANIFEST.json",
        "crates/codegen/xai-grok-pager-render/build.rs",
    ):
        require_fragment(path, WARP_REVISION)


def check_updater_routes() -> None:
    version = read("crates/codegen/xai-grok-update/src/version.rs")
    updater = read("crates/codegen/xai-grok-update/src/auto_update.rs")
    for fragment in (
        'pub const GH_RELEASE_REPO: &str = "OpenCompanyApp/grok-build-enhanced";',
        "OpenCompanyApp/grok-build-enhanced/releases?per_page=100",
        "OpenCompanyApp/grok-build-enhanced/releases/download",
        'if installer != "gh-release"',
        "fetch_gh_release_version(&config.channel).await",
    ):
        if fragment not in version:
            raise ContractError(f"fork updater discovery is missing {fragment!r}")
    for fragment in (
        'Some("gh-release")',
        'if installer == "gh-release"',
        "install_gh_release(target).await",
        "ensure_release_asset_exists(&version, release_api_url, os, arch).await?",
        "gh_release_download(&download_url, &binary_path).await?",
    ):
        if fragment not in updater:
            raise ContractError(f"fork updater installation is missing {fragment!r}")


def check_generated_workspace(manifest: dict[str, object]) -> None:
    patch_stack = manifest.get("patch_stack")
    if not isinstance(patch_stack, dict):
        raise ContractError("fork manifest has no patch_stack object")
    baseline = patch_stack.get("baseline")
    if not isinstance(baseline, dict) or not isinstance(baseline.get("commit"), str):
        raise ContractError("fork manifest has no immutable baseline commit")
    result = subprocess.run(
        ["git", "diff", "--quiet", baseline["commit"], "--", "Cargo.toml"],
        cwd=ROOT,
        check=False,
    )
    if result.returncode == 1:
        raise ContractError(
            "root Cargo.toml differs from the immutable baseline; edit crate manifests instead"
        )
    if result.returncode not in (0, 1):
        raise ContractError("could not compare generated Cargo.toml to the baseline")


def check_workflow_pins() -> None:
    workflows = sorted((ROOT / ".github/workflows").glob("*.y*ml"))
    if not workflows:
        raise ContractError("repository has no GitHub workflows")
    uses_pattern = re.compile(r"^\s*uses:\s*([^@\s]+)@([^\s#]+)", re.MULTILINE)
    for workflow in workflows:
        text = workflow.read_text(encoding="utf-8")
        for action, revision in uses_pattern.findall(text):
            if not ACTION_SHA.fullmatch(revision):
                raise ContractError(
                    f"{workflow.relative_to(ROOT)} action {action} is not pinned to a full SHA"
                )


def check_tracked_secret_files() -> None:
    output = subprocess.check_output(["git", "ls-files", "-z"], cwd=ROOT)
    forbidden_names = {
        ".env",
        "auth.json",
        "credentials.json",
        "id_rsa",
        "id_ed25519",
        ".npmrc",
    }
    forbidden_suffixes = (".pem", ".p12", ".pfx")
    for raw_path in output.decode("utf-8").split("\0"):
        if not raw_path:
            continue
        name = Path(raw_path).name.lower()
        if name in forbidden_names or name.endswith(forbidden_suffixes):
            raise ContractError(f"tracked secret-bearing filename is not allowed: {raw_path}")


def main() -> int:
    try:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        check_branding_and_charter()
        check_provider_isolation_sentinels()
        check_warp_lock(manifest)
        check_updater_routes()
        check_generated_workspace(manifest)
        check_workflow_pins()
        check_tracked_secret_files()
    except (ContractError, OSError, subprocess.SubprocessError, json.JSONDecodeError) as error:
        print(f"fork contract validation failed: {error}", file=sys.stderr)
        return 1
    print("fork contracts OK: branding, providers, Warp, updater, workspace, workflows, secrets")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
