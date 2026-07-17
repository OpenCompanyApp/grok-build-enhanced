#!/bin/sh
# Thin launcher for the isolated, read-only Python patch-stack verifier.
set -eu

script_source=$0
case $script_source in
    /*) ;;
    *) script_source=$PWD/$script_source ;;
esac
script_dir=${script_source%/*}
if [ "$script_dir" = "$script_source" ]; then
    script_dir=$PWD
fi
script_dir=$(CDPATH= cd -P "$script_dir" && pwd -P) || {
    printf '%s\n' 'ERROR: cannot resolve verifier script directory' >&2
    exit 2
}

readonly safe_path="/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
PATH=$safe_path
export PATH

python_binary=$(command -v python3 || true)
git_binary=$(command -v git || true)
case $python_binary in
    /*) [ -x "$python_binary" ] ;;
    *) false ;;
esac || {
    printf '%s\n' 'ERROR: python3 is unavailable on the sanitized PATH' >&2
    exit 2
}
case $git_binary in
    /*) [ -x "$git_binary" ] ;;
    *) false ;;
esac || {
    printf '%s\n' 'ERROR: git is unavailable on the sanitized PATH' >&2
    exit 2
}

exec /usr/bin/env -i \
    PATH="$safe_path" \
    HOME=/nonexistent-fork-guardrail-home \
    XDG_CONFIG_HOME=/nonexistent-fork-guardrail-xdg \
    LANG=C \
    LC_ALL=C \
    PYTHONDONTWRITEBYTECODE=1 \
    FORK_GUARDRAIL_GIT="$git_binary" \
    "$python_binary" -I -B "$script_dir/check_manifest.py" \
    __verify_patch_stack__ "$@"
