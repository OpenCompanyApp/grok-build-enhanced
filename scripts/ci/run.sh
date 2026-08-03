#!/usr/bin/env bash
set -Eeuo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}"
export PYTHONDONTWRITEBYTECODE="${PYTHONDONTWRITEBYTECODE:-1}"

usage() {
  cat <<'EOF'
Usage: scripts/ci/run.sh <group> [arguments]

Groups:
  static                   Credential-free Python, manifest, formatting, and hygiene checks
  rust-contracts           Focused provider isolation, updater, and composed-binary checks
  core                     Config, provider, sampler, shell, session, and tools libraries
  ui                       Pager, rendering, markdown, and minimal-mode libraries
  pty                      Pager and PTY integration tests with bounded test concurrency
  libraries                Remaining workspace packages and doctests
  policy                   Workspace all-target check and clippy with warnings denied
  native <target> [build]  Native composed check; optionally build release-dist

Every group is directly runnable by developers and agents. CI supplies an isolated
CARGO_TARGET_DIR; local callers may retain their normal target for iterative work.
EOF
}

run_static() {
  python3 -I -B -m unittest discover -s fork/scripts/tests -v
  python3 -I -B -m unittest discover -s scripts/ci/tests -v
  python3 -I -B fork/scripts/check_manifest.py --strict-coverage
  python3 -I -B fork/scripts/check_fork_contracts.py
  python3 -I -B scripts/release/tests/test_release_pipeline.py
  python3 -I -B scripts/release/tests/test_install_script.py
  cargo fmt --all -- --check
  git diff --check
}

run_rust_contracts() {
  cargo test --locked -p xai-grok-sampler --lib \
    'client::tests::codex_rejects_static_or_generic_header_credentials' -- --exact
  cargo test --locked -p xai-grok-tools --lib \
    'types::api_key_provider::tests::codex_auth_resolution_rejects_generic_static_key_fallback' -- --exact
  cargo test --locked -p xai-grok-tools --lib \
    'implementations::grok_build::video_gen::tests::authorization_header_values_are_always_sensitive' -- --exact
  cargo test --locked -p xai-grok-tools --lib \
    'implementations::grok_build::video_gen::tests::extra_authorization_header_cannot_disable_redaction' -- --exact
  cargo test --locked -p xai-grok-shell --lib \
    'session::acp_session::model_switch::provider_media_switch_tests::image_resource_and_definitions_follow_provider_switches' -- --exact
  cargo test --locked -p xai-grok-shell --lib \
    'agent::config::tests::custom_credentials_never_inherit_xai_session_or_global_keys' -- --exact
  cargo test --locked -p xai-grok-shell --lib \
    'session::provider::openai_codex::tests::custom_runtime_drops_xai_credentials_and_generic_tool_auth' -- --exact
  cargo test --locked -p xai-grok-shell --lib \
    'session::acp_session::auth_error_no_retry_tests::reconstruct_full_config_drops_cross_provider_custom_credential' -- --exact
  cargo test --locked -p xai-grok-shell --lib \
    'session::acp_session::session_setup::tests::idle_model_metadata_refresh_requires_xai_provider_identity' -- --exact
  cargo test --locked -p xai-grok-shell --lib \
    'session::acp_session::media_gen_auth_retry_tests::provider_owned_codex_401_never_falls_through_to_xai_auth_manager' -- --exact
  cargo test --locked -p xai-grok-shell --lib \
    'agent::models::tests::provider_scoped_fallback_never_crosses_from_codex_to_xai' -- --exact

  cargo test --locked -p xai-grok-version
  cargo test --locked -p xai-grok-update --lib \
    'version::tests::enhanced_release_selection_requires_the_exact_native_asset' -- --exact
  cargo test --locked -p xai-grok-update --lib \
    'version::tests::github_release_lookup_uses_public_api_metadata' -- --exact
  cargo test --locked -p xai-grok-update --test test_fork_release_routing
  cargo test --locked -p xai-grok-pager --lib \
    'app::cli::tests::upgrade_alias_uses_the_enhanced_update_command' -- --exact
  cargo test --locked -p xai-grok-pager --lib 'views::welcome::tests'
  cargo check --locked -p xai-grok-pager-bin
}

run_core() {
  local test_threads="${CI_TEST_THREADS:-2}"
  cargo test --locked --lib \
    -p xai-grok-auth \
    -p xai-grok-config \
    -p xai-grok-config-types \
    -p xai-grok-provider-http \
    -p xai-grok-sampler \
    -p xai-grok-sampling-types \
    -p xai-grok-shell \
    -p xai-grok-tools \
    -p xai-grok-tools-api -- \
    --test-threads="$test_threads"
}

run_ui() {
  local test_threads="${CI_TEST_THREADS:-2}"
  cargo test --locked --lib \
    -p xai-grok-markdown \
    -p xai-grok-markdown-core \
    -p xai-grok-mermaid \
    -p xai-grok-pager \
    -p xai-grok-pager-minimal \
    -p xai-grok-pager-render -- \
    --test-threads="$test_threads"
}

run_pty() {
  local test_threads="${CI_TEST_THREADS:-2}"
  cargo test --locked -p xai-grok-pager-pty-harness --tests -- \
    --test-threads="$test_threads"
  cargo test --locked -p xai-grok-pager --test '*' -- \
    --test-threads="$test_threads"
}

run_libraries() {
  local test_threads="${CI_TEST_THREADS:-2}"
  local -a excludes=(
    --exclude xai-grok-auth
    --exclude xai-grok-config
    --exclude xai-grok-config-types
    --exclude xai-grok-markdown
    --exclude xai-grok-markdown-core
    --exclude xai-grok-mermaid
    --exclude xai-grok-pager
    --exclude xai-grok-pager-bin
    --exclude xai-grok-pager-minimal
    --exclude xai-grok-pager-pty-harness
    --exclude xai-grok-pager-render
    --exclude xai-grok-provider-http
    --exclude xai-grok-sampler
    --exclude xai-grok-sampling-types
    --exclude xai-grok-shell
    --exclude xai-grok-tools
    --exclude xai-grok-tools-api
  )
  cargo test --locked --workspace "${excludes[@]}" -- \
    --test-threads="$test_threads"
  cargo test --locked --workspace --doc "${excludes[@]}"
}

run_policy() {
  cargo check --locked --workspace --all-targets
  cargo clippy --locked --workspace --all-targets -- -D warnings
}

run_native() {
  local target="${1:-}"
  local mode="${2:-check}"
  if [[ -z "$target" ]]; then
    printf 'error: native requires a Rust target\n' >&2
    return 2
  fi

  local host
  host="$(rustc -vV | sed -n 's/^host: //p')"
  if [[ "$host" != "$target" ]]; then
    printf 'error: native qualification requires host %s, got %s\n' "$target" "$host" >&2
    return 1
  fi

  cargo check --locked -p xai-grok-pager-bin --target "$target"
  if [[ "$mode" == "build" ]]; then
    local version
    version="$(cargo metadata --locked --no-deps --format-version 1 | python3 -c '
import json, sys
for package in json.load(sys.stdin)["packages"]:
    if package["name"] == "xai-grok-pager-bin":
        print(package["version"])
        break
else:
    raise SystemExit("xai-grok-pager-bin package is missing")
')"
    export GROK_VERSION="$version"
    cargo build --locked --profile release-dist \
      --package xai-grok-pager-bin \
      --features release-dist \
      --target "$target"
    local target_dir="${CARGO_TARGET_DIR:-target}"
    "$target_dir/$target/release-dist/xai-grok-pager" --version | grep -F -- "$version" >/dev/null
  elif [[ "$mode" != "check" ]]; then
    printf 'error: native mode must be check or build, got %s\n' "$mode" >&2
    return 2
  fi
}

case "${1:-}" in
  static) run_static ;;
  rust-contracts) run_rust_contracts ;;
  core) run_core ;;
  ui) run_ui ;;
  pty) run_pty ;;
  libraries) run_libraries ;;
  policy) run_policy ;;
  native) shift; run_native "$@" ;;
  -h|--help|help|"") usage ;;
  *)
    printf 'error: unknown CI group: %s\n' "$1" >&2
    usage >&2
    exit 2
    ;;
esac
