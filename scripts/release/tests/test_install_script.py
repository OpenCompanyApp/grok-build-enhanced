#!/usr/bin/env python3
"""Hermetic tests for the fork-owned curl installer."""

from __future__ import annotations

import json
import os
import stat
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.dont_write_bytecode = True
REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
RELEASE_SCRIPT_DIR = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(RELEASE_SCRIPT_DIR))

import release_contract as contract  # noqa: E402

INSTALLER = REPOSITORY_ROOT / "install.sh"
RELEASES_BASE_URL = "https://fixtures.invalid/releases"
SOURCE_COMMIT = "a" * 40


class InstallerHarness:
    def __init__(self, version: str = "1.2.3") -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.home = self.root / "home"
        self.home.mkdir()
        self.fake_bin = self.root / "fake-bin"
        self.fake_bin.mkdir()
        self.release_root = self.root / "releases"
        self.release_root.mkdir()
        self.url_log = self.root / "curl-urls.txt"
        self.version = version
        self.asset_names: list[str] = []
        self._write_fake_commands()
        self.write_release()

    def close(self) -> None:
        self.temporary.cleanup()

    def _write_fake_commands(self) -> None:
        fake_curl = self.fake_bin / "curl"
        fake_curl.write_text(
            """#!/bin/sh
set -eu
output=
url=
while [ "$#" -gt 0 ]; do
    case "$1" in
        --output)
            output=$2
            shift 2
            ;;
        --retry|--retry-delay|--connect-timeout|--max-time|--proto|--proto-redir)
            shift 2
            ;;
        --*)
            shift
            ;;
        *)
            url=$1
            shift
            ;;
    esac
done
[ -n "$output" ] || exit 91
[ -n "$url" ] || exit 92
printf '%s\n' "$url" >>"$FAKE_CURL_LOG"
case "$url" in
    "$FAKE_RELEASES_BASE_URL/latest/download/RELEASE-PROVENANCE.json")
        source="$FAKE_RELEASE_ROOT/download/v$FAKE_LATEST_VERSION/RELEASE-PROVENANCE.json"
        ;;
    "$FAKE_RELEASES_BASE_URL"/download/*)
        relative=${url#"$FAKE_RELEASES_BASE_URL"/}
        source="$FAKE_RELEASE_ROOT/$relative"
        ;;
    *)
        printf 'unexpected URL: %s\n' "$url" >&2
        exit 93
        ;;
esac
[ -f "$source" ] || {
    printf 'missing fixture for URL: %s\n' "$url" >&2
    exit 94
}
cp "$source" "$output"
""",
            encoding="utf-8",
        )
        fake_curl.chmod(0o755)

        fake_uname = self.fake_bin / "uname"
        fake_uname.write_text(
            """#!/bin/sh
case "${1:-}" in
    -s) printf '%s\n' "$FAKE_UNAME_SYSTEM" ;;
    -m) printf '%s\n' "$FAKE_UNAME_MACHINE" ;;
    *) exit 95 ;;
esac
""",
            encoding="utf-8",
        )
        fake_uname.chmod(0o755)

    def write_release(
        self,
        *,
        identity: str = contract.DISTRIBUTION,
        reported_version: str | None = None,
    ) -> None:
        release_dir = self.release_root / "download" / f"v{self.version}"
        release_dir.mkdir(parents=True, exist_ok=True)
        reported_version = reported_version or self.version
        binary = f"""#!/bin/sh
case "${{1:-}}" in
    version)
        printf '%s\\n' '{identity} {reported_version}'
        ;;
    completions)
        printf '# completion for %s\\n' "${{2:-unknown}}"
        ;;
    *)
        exit 0
        ;;
esac
""".encode()

        records = []
        self.asset_names = []
        for target in contract.TARGETS:
            name = contract.asset_name(self.version, target.os, target.arch)
            asset = release_dir / name
            asset.write_bytes(binary)
            asset.chmod(0o755)
            self.asset_names.append(name)
            records.append(
                contract.asset_record(
                    version=self.version,
                    repository=contract.RELEASE_REPOSITORY,
                    source_commit=SOURCE_COMMIT,
                    os_name=target.os,
                    arch=target.arch,
                    rust_target=target.rust_target,
                    asset_path=asset,
                )
            )

        provenance = contract.provenance_document(
            self.version,
            contract.RELEASE_REPOSITORY,
            SOURCE_COMMIT,
            records,
        )
        (release_dir / contract.PROVENANCE_NAME).write_bytes(
            contract.canonical_json(provenance)
        )
        self.refresh_checksums()

    @property
    def release_dir(self) -> Path:
        return self.release_root / "download" / f"v{self.version}"

    def refresh_checksums(self) -> None:
        names = [*self.asset_names, contract.PROVENANCE_NAME]
        (self.release_dir / contract.CHECKSUMS_NAME).write_bytes(
            contract.checksums_bytes(self.release_dir, names)
        )

    def run(
        self,
        *arguments: str,
        system: str = "Darwin",
        machine: str = "arm64",
        extra_env: dict[str, str] | None = None,
    ) -> subprocess.CompletedProcess[str]:
        environment = os.environ.copy()
        for name in (
            "GROK_HOME",
            "GROK_INSTALL_BIN_DIR",
            "GROK_INSTALL_FORCE",
            "GROK_INSTALL_NO_MODIFY_PATH",
            "GROK_INSTALL_VERSION",
        ):
            environment.pop(name, None)
        environment.update(
            {
                "FAKE_CURL_LOG": str(self.url_log),
                "FAKE_LATEST_VERSION": self.version,
                "FAKE_RELEASE_ROOT": str(self.release_root),
                "FAKE_RELEASES_BASE_URL": RELEASES_BASE_URL,
                "FAKE_UNAME_MACHINE": machine,
                "FAKE_UNAME_SYSTEM": system,
                "GROK_INSTALL_TEST_RELEASES_BASE_URL": RELEASES_BASE_URL,
                "HOME": str(self.home),
                "PATH": f"{self.fake_bin}{os.pathsep}{os.environ.get('PATH', '')}",
                "SHELL": "/bin/zsh",
                "TMPDIR": str(self.root),
                "XDG_CONFIG_HOME": str(self.home / ".config"),
            }
        )
        if extra_env:
            environment.update(extra_env)
        return subprocess.run(
            ["/bin/sh", str(INSTALLER), *arguments],
            check=False,
            capture_output=True,
            text=True,
            timeout=30,
            env=environment,
        )

    def urls(self) -> list[str]:
        if not self.url_log.exists():
            return []
        return self.url_log.read_text(encoding="utf-8").splitlines()


class InstallScriptTests(unittest.TestCase):
    def test_script_is_executable_and_valid_posix_shell(self) -> None:
        mode = INSTALLER.stat().st_mode
        self.assertTrue(mode & stat.S_IXUSR)
        result = subprocess.run(
            ["/bin/sh", "-n", str(INSTALLER)],
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_production_release_route_is_explicitly_fork_owned(self) -> None:
        source = INSTALLER.read_text(encoding="utf-8")
        self.assertIn('REPOSITORY="OpenCompanyApp/grok-build-enhanced"', source)
        self.assertIn(
            "https://github.com/${REPOSITORY}/releases",
            source,
        )
        for forbidden in ("https://x.ai", "registry.npmjs.org", "storage.googleapis.com"):
            self.assertNotIn(forbidden, source)

    def test_supported_target_matrix_uses_exact_release_assets(self) -> None:
        cases = (
            ("Darwin", "arm64", "macos", "aarch64"),
            ("Darwin", "x86_64", "macos", "x86_64"),
            ("Linux", "aarch64", "linux", "aarch64"),
            ("Linux", "amd64", "linux", "x86_64"),
        )
        for system, machine, os_name, arch in cases:
            with self.subTest(system=system, machine=machine):
                harness = InstallerHarness()
                try:
                    result = harness.run(
                        "--no-modify-path", system=system, machine=machine
                    )
                    self.assertEqual(result.returncode, 0, result.stderr)
                    asset = f"grok-{harness.version}-{os_name}-{arch}"
                    installed = harness.home / ".grok" / "downloads" / asset
                    self.assertTrue(installed.is_file())
                    self.assertTrue(os.access(installed, os.X_OK))
                    self.assertEqual(
                        os.readlink(harness.home / ".grok" / "bin" / "grok"),
                        f"../downloads/{asset}",
                    )
                    self.assertEqual(
                        os.readlink(harness.home / ".grok" / "bin" / "agent"),
                        f"../downloads/{asset}",
                    )
                    self.assertEqual(
                        harness.urls(),
                        [
                            f"{RELEASES_BASE_URL}/latest/download/{contract.PROVENANCE_NAME}",
                            f"{RELEASES_BASE_URL}/download/v{harness.version}/"
                            f"{contract.CHECKSUMS_NAME}",
                            f"{RELEASES_BASE_URL}/download/v{harness.version}/{asset}",
                        ],
                    )
                finally:
                    harness.close()

    def test_pinned_prerelease_uses_only_the_exact_tag(self) -> None:
        harness = InstallerHarness("2.3.4-alpha.5")
        try:
            result = harness.run(
                "--version", harness.version, "--no-modify-path", system="Linux", machine="x86_64"
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(
                harness.urls()[0],
                f"{RELEASES_BASE_URL}/download/v{harness.version}/{contract.PROVENANCE_NAME}",
            )
            self.assertNotIn("/latest/", "\n".join(harness.urls()))
        finally:
            harness.close()

    def test_latest_rejects_a_prerelease(self) -> None:
        harness = InstallerHarness("2.0.0-rc.1")
        try:
            result = harness.run("--no-modify-path")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("latest release resolved to prerelease", result.stderr)
            self.assertEqual(len(harness.urls()), 1)
        finally:
            harness.close()

    def test_reinstall_is_idempotent_and_preserves_user_data(self) -> None:
        harness = InstallerHarness()
        try:
            grok_home = harness.home / ".grok"
            sessions = grok_home / "sessions"
            sessions.mkdir(parents=True)
            config = grok_home / "config.toml"
            marker = sessions / "keep-me"
            config.write_text("theme = 'Dracula'\n", encoding="utf-8")
            marker.write_text("session data\n", encoding="utf-8")

            first = harness.run()
            second = harness.run()
            self.assertEqual(first.returncode, 0, first.stderr)
            self.assertEqual(second.returncode, 0, second.stderr)
            self.assertEqual(config.read_text(encoding="utf-8"), "theme = 'Dracula'\n")
            self.assertEqual(marker.read_text(encoding="utf-8"), "session data\n")

            profile = (harness.home / ".zshrc").read_text(encoding="utf-8")
            self.assertEqual(
                profile.count("# >>> grok build enhanced installer >>>"), 1
            )
            self.assertIn('export PATH="$HOME/.grok/bin:$PATH"', profile)
            self.assertTrue(
                (grok_home / "completions" / "bash" / "grok.bash").is_file()
            )
            self.assertTrue((grok_home / "completions" / "zsh" / "_grok").is_file())
            self.assertTrue(
                (harness.home / ".config" / "fish" / "completions" / "grok.fish").is_file()
            )
        finally:
            harness.close()

    def test_no_modify_path_leaves_shell_profile_untouched(self) -> None:
        harness = InstallerHarness()
        try:
            profile = harness.home / ".zshrc"
            profile.write_text("# existing profile\n", encoding="utf-8")
            result = harness.run("--no-modify-path")
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(
                profile.read_text(encoding="utf-8"), "# existing profile\n"
            )
            self.assertIn("Add '", result.stderr)
        finally:
            harness.close()

    def test_binary_checksum_mismatch_is_not_published(self) -> None:
        harness = InstallerHarness()
        try:
            asset = harness.release_dir / f"grok-{harness.version}-macos-aarch64"
            asset.write_bytes(asset.read_bytes() + b"tampered\n")
            result = harness.run("--no-modify-path")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("checksum verification failed", result.stderr)
            self.assertFalse(
                (harness.home / ".grok" / "downloads" / asset.name).exists()
            )
            self.assertFalse((harness.home / ".grok" / "bin" / "grok").exists())
        finally:
            harness.close()

    def test_provenance_checksum_mismatch_is_rejected(self) -> None:
        harness = InstallerHarness()
        try:
            provenance = harness.release_dir / contract.PROVENANCE_NAME
            provenance.write_bytes(provenance.read_bytes() + b"\n")
            result = harness.run("--no-modify-path")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("provenance checksum verification failed", result.stderr)
            self.assertFalse((harness.home / ".grok" / "downloads").exists())
        finally:
            harness.close()

    def test_wrong_binary_identity_fails_the_smoke_test(self) -> None:
        harness = InstallerHarness()
        try:
            harness.write_release(identity="Not Grok Build Enhanced")
            result = harness.run("--no-modify-path")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("identity/version smoke test", result.stderr)
            expected = (
                harness.home
                / ".grok"
                / "downloads"
                / f"grok-{harness.version}-macos-aarch64"
            )
            self.assertFalse(expected.exists())
            self.assertFalse((harness.home / ".grok" / "bin" / "grok").exists())
        finally:
            harness.close()

    def test_release_provenance_must_remain_fork_owned(self) -> None:
        harness = InstallerHarness()
        try:
            path = harness.release_dir / contract.PROVENANCE_NAME
            provenance = json.loads(path.read_text(encoding="utf-8"))
            provenance["repository"] = "xai-org/grok-build"
            path.write_bytes(contract.canonical_json(provenance))
            harness.refresh_checksums()
            result = harness.run("--no-modify-path")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("not 'OpenCompanyApp/grok-build-enhanced'", result.stderr)
            self.assertEqual(len(harness.urls()), 1)
        finally:
            harness.close()

    def test_unmanaged_link_conflict_requires_force(self) -> None:
        harness = InstallerHarness()
        try:
            managed_bin = harness.home / ".grok" / "bin"
            managed_bin.mkdir(parents=True)
            conflict = managed_bin / "grok"
            conflict.write_text("do not replace\n", encoding="utf-8")

            refused = harness.run("--no-modify-path")
            self.assertNotEqual(refused.returncode, 0)
            self.assertIn("use --force to override", refused.stderr)
            self.assertEqual(conflict.read_text(encoding="utf-8"), "do not replace\n")

            forced = harness.run("--force", "--no-modify-path")
            self.assertEqual(forced.returncode, 0, forced.stderr)
            self.assertTrue(conflict.is_symlink())
            self.assertTrue((managed_bin / "agent").is_symlink())
        finally:
            harness.close()

    def test_force_never_replaces_a_directory(self) -> None:
        harness = InstallerHarness()
        try:
            conflict = harness.home / ".grok" / "bin" / "grok"
            conflict.mkdir(parents=True)
            result = harness.run("--force", "--no-modify-path")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("refusing to replace directory", result.stderr)
            self.assertTrue(conflict.is_dir())
            expected = (
                harness.home
                / ".grok"
                / "downloads"
                / f"grok-{harness.version}-macos-aarch64"
            )
            self.assertFalse(expected.exists())
        finally:
            harness.close()

    def test_custom_bin_dir_is_preflighted_before_publication(self) -> None:
        harness = InstallerHarness()
        try:
            front_bin = harness.root / "front-bin"
            front_bin.mkdir()
            conflict = front_bin / "grok"
            conflict.write_text("other grok\n", encoding="utf-8")

            refused = harness.run(
                "--bin-dir", str(front_bin), "--no-modify-path"
            )
            self.assertNotEqual(refused.returncode, 0)
            self.assertEqual(conflict.read_text(encoding="utf-8"), "other grok\n")
            self.assertFalse((harness.home / ".grok" / "bin" / "grok").exists())

            forced = harness.run(
                "--bin-dir", str(front_bin), "--force", "--no-modify-path"
            )
            self.assertEqual(forced.returncode, 0, forced.stderr)
            self.assertEqual(
                os.readlink(front_bin / "grok"),
                str(harness.home / ".grok" / "bin" / "grok"),
            )
            self.assertEqual(
                os.readlink(front_bin / "agent"),
                str(harness.home / ".grok" / "bin" / "agent"),
            )
        finally:
            harness.close()

    def test_invalid_version_fails_before_any_download(self) -> None:
        harness = InstallerHarness()
        try:
            result = harness.run("--version", "01.2.3", "--no-modify-path")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("invalid version", result.stderr)
            self.assertEqual(harness.urls(), [])
        finally:
            harness.close()


if __name__ == "__main__":
    unittest.main()
