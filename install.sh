#!/bin/sh
# Install Grok Build Enhanced from fork-owned GitHub Release assets.
#
# Usage:
#   curl --proto '=https' --tlsv1.2 -LsSf \
#     https://raw.githubusercontent.com/OpenCompanyApp/grok-build-enhanced/main/install.sh | sh
#   curl .../install.sh | sh -s -- --version 0.2.0
#
# This installer supports macOS and glibc-based Linux on Arm64 and x86-64.
# It never downloads from the official xAI installer, npm, or artifact buckets.

set -eu
umask 077

PRODUCT="Grok Build Enhanced"
REPOSITORY="OpenCompanyApp/grok-build-enhanced"
RELEASES_BASE="${GROK_INSTALL_TEST_RELEASES_BASE_URL:-https://github.com/${REPOSITORY}/releases}"
VERSION="${GROK_INSTALL_VERSION:-}"
FRONT_BIN_DIR="${GROK_INSTALL_BIN_DIR:-}"
MODIFY_PATH=1
FORCE=0

case "${GROK_INSTALL_NO_MODIFY_PATH:-}" in
    1|true|TRUE|yes|YES|on|ON) MODIFY_PATH=0 ;;
esac
case "${GROK_INSTALL_FORCE:-}" in
    1|true|TRUE|yes|YES|on|ON) FORCE=1 ;;
esac

say() {
    printf '%s\n' "$*" >&2
}

fail() {
    say "Error: $*"
    exit 1
}

usage() {
    cat <<'EOF'
Install Grok Build Enhanced from its fork-owned GitHub Release.

Usage: install.sh [OPTIONS]

Options:
  --version VERSION   Install an exact release instead of the latest stable one
  --bin-dir PATH      Also link grok and agent into this absolute directory
  --no-modify-path    Do not add ~/.grok/bin to a shell startup file
  --force             Replace conflicting files in managed bin directories
  -h, --help          Show this help

Environment equivalents:
  GROK_INSTALL_VERSION, GROK_INSTALL_BIN_DIR,
  GROK_INSTALL_NO_MODIFY_PATH=1, GROK_INSTALL_FORCE=1

The canonical installation remains under ~/.grok so `grok update` can perform
atomic fork-owned updates. Existing ~/.grok configuration and sessions are
preserved.
EOF
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --version)
            [ "$#" -ge 2 ] || fail "--version requires a value"
            VERSION=$2
            shift 2
            ;;
        --version=*)
            VERSION=${1#*=}
            shift
            ;;
        --bin-dir)
            [ "$#" -ge 2 ] || fail "--bin-dir requires a value"
            FRONT_BIN_DIR=$2
            shift 2
            ;;
        --bin-dir=*)
            FRONT_BIN_DIR=${1#*=}
            shift
            ;;
        --no-modify-path)
            MODIFY_PATH=0
            shift
            ;;
        --force)
            FORCE=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        --)
            shift
            break
            ;;
        *)
            fail "unknown option: $1"
            ;;
    esac
done
[ "$#" -eq 0 ] || fail "unexpected positional arguments: $*"

[ -n "${HOME:-}" ] || fail "HOME is not set"
command -v curl >/dev/null 2>&1 || fail "curl is required"

GROK_HOME_DIR="${GROK_HOME:-$HOME/.grok}"
DOWNLOAD_DIR="$GROK_HOME_DIR/downloads"
MANAGED_BIN_DIR="$GROK_HOME_DIR/bin"

case "$GROK_HOME_DIR" in
    /*) ;;
    *) fail "GROK_HOME must be an absolute path: $GROK_HOME_DIR" ;;
esac
if [ -n "$FRONT_BIN_DIR" ]; then
    case "$FRONT_BIN_DIR" in
        /*) ;;
        *) fail "--bin-dir must be an absolute path: $FRONT_BIN_DIR" ;;
    esac
fi

validate_version() (
    value=$1
    case "$value" in
        ''|*+*|*/*|*[!0-9A-Za-z.-]*) exit 1 ;;
    esac

    core=${value%%-*}
    if [ "$core" = "$value" ]; then
        prerelease=
    else
        prerelease=${value#"$core"-}
        [ -n "$prerelease" ] || exit 1
    fi

    old_ifs=$IFS
    IFS=.
    set -- $core
    IFS=$old_ifs
    [ "$#" -eq 3 ] || exit 1
    for identifier do
        case "$identifier" in
            ''|*[!0-9]*) exit 1 ;;
            0) ;;
            0*) exit 1 ;;
        esac
    done

    if [ -n "$prerelease" ]; then
        old_ifs=$IFS
        IFS=.
        set -- $prerelease
        IFS=$old_ifs
        [ "$#" -gt 0 ] || exit 1
        for identifier do
            case "$identifier" in
                ''|*[!0-9A-Za-z-]*) exit 1 ;;
            esac
            case "$identifier" in
                *[!0-9]*) ;;
                0) ;;
                0*) exit 1 ;;
            esac
        done
    fi
)

case "$(uname -s)" in
    Darwin) OS=macos ;;
    Linux) OS=linux ;;
    *) fail "unsupported operating system: $(uname -s)" ;;
esac

case "$(uname -m)" in
    arm64|aarch64) ARCH=aarch64 ;;
    x86_64|amd64) ARCH=x86_64 ;;
    *) fail "unsupported architecture: $(uname -m)" ;;
esac

if [ -n "$VERSION" ]; then
    validate_version "$VERSION" \
        || fail "invalid version '$VERSION' (expected strict SemVer without build metadata)"
fi

TEMP_DIR=$(mktemp -d "${TMPDIR:-/tmp}/grok-build-enhanced-install.XXXXXX") \
    || fail "could not create a temporary directory"
BINARY_TMP=
LINK_TMP=

cleanup() {
    [ -z "${BINARY_TMP:-}" ] || rm -f -- "$BINARY_TMP"
    [ -z "${LINK_TMP:-}" ] || rm -f -- "$LINK_TMP"
    rm -rf -- "$TEMP_DIR"
}
trap cleanup 0
trap 'exit 1' 1 2 3 15

curl_download() {
    url=$1
    destination=$2
    curl \
        --fail \
        --location \
        --silent \
        --show-error \
        --retry 3 \
        --retry-delay 1 \
        --connect-timeout 15 \
        --max-time 1200 \
        --proto '=https' \
        --proto-redir '=https' \
        --tlsv1.2 \
        --output "$destination" \
        "$url"
}

json_string_field() {
    field=$1
    file=$2
    awk -v wanted="\"${field}\"" '
        {
            line = $0
            sub(/^[[:space:]]*/, "", line)
            separator = index(line, ":")
            if (separator == 0 || substr(line, 1, separator - 1) != wanted) {
                next
            }
            value = substr(line, separator + 1)
            sub(/^[[:space:]]*\"/, "", value)
            sub(/\"[[:space:]]*,?[[:space:]]*$/, "", value)
            print value
            count++
        }
        END {
            if (count != 1) {
                exit 1
            }
        }
    ' "$file"
}

PROVENANCE="$TEMP_DIR/RELEASE-PROVENANCE.json"
if [ -n "$VERSION" ]; then
    PROVENANCE_URL="$RELEASES_BASE/download/v${VERSION}/RELEASE-PROVENANCE.json"
else
    say "Resolving the latest stable Grok Build Enhanced release..."
    PROVENANCE_URL="$RELEASES_BASE/latest/download/RELEASE-PROVENANCE.json"
fi
curl_download "$PROVENANCE_URL" "$PROVENANCE" \
    || fail "could not download release provenance from $PROVENANCE_URL"

METADATA_REPOSITORY=$(json_string_field repository "$PROVENANCE") \
    || fail "release provenance has no unique repository"
METADATA_DISTRIBUTION=$(json_string_field distribution "$PROVENANCE") \
    || fail "release provenance has no unique distribution"
METADATA_TAG=$(json_string_field release_tag "$PROVENANCE") \
    || fail "release provenance has no unique release tag"
case "$METADATA_TAG" in
    v*) METADATA_VERSION=${METADATA_TAG#v} ;;
    *) fail "release provenance tag '$METADATA_TAG' does not start with 'v'" ;;
esac

validate_version "$METADATA_VERSION" \
    || fail "release provenance contains invalid version '$METADATA_VERSION'"
if [ -z "$VERSION" ]; then
    VERSION=$METADATA_VERSION
    case "$VERSION" in
        *-*) fail "the latest release resolved to prerelease version '$VERSION'" ;;
    esac
elif [ "$METADATA_VERSION" != "$VERSION" ]; then
    fail "release provenance version '$METADATA_VERSION' does not match requested '$VERSION'"
fi
[ "$METADATA_REPOSITORY" = "$REPOSITORY" ] \
    || fail "release provenance belongs to '$METADATA_REPOSITORY', not '$REPOSITORY'"
[ "$METADATA_DISTRIBUTION" = "$PRODUCT" ] \
    || fail "release provenance identifies '$METADATA_DISTRIBUTION', not '$PRODUCT'"
[ "$METADATA_TAG" = "v$VERSION" ] \
    || fail "release provenance tag '$METADATA_TAG' does not match 'v$VERSION'"

ASSET="grok-${VERSION}-${OS}-${ARCH}"
ASSET_RECORD_COUNT=$(grep -F "\"name\": \"$ASSET\"" "$PROVENANCE" | wc -l | tr -d '[:space:]')
[ "$ASSET_RECORD_COUNT" = 1 ] \
    || fail "release provenance does not contain exactly one '$ASSET' asset"

RELEASE_BASE="$RELEASES_BASE/download/v${VERSION}"
CHECKSUMS="$TEMP_DIR/SHA256SUMS"
curl_download "$RELEASE_BASE/SHA256SUMS" "$CHECKSUMS" \
    || fail "could not download SHA256SUMS for v$VERSION"

checksum_for() {
    name=$1
    file=$2
    awk -v wanted="$name" '
        $2 == wanted {
            count++
            checksum = $1
        }
        END {
            if (count == 1) {
                print checksum
            } else {
                exit 1
            }
        }
    ' "$file"
}

validate_checksum() {
    checksum=$1
    [ "${#checksum}" -eq 64 ] || return 1
    case "$checksum" in
        *[!0-9a-f]*) return 1 ;;
    esac
}

hash_file() {
    file=$1
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$file" | awk '{ print $1 }'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$file" | awk '{ print $1 }'
    else
        return 1
    fi
}

EXPECTED_PROVENANCE_SHA=$(checksum_for RELEASE-PROVENANCE.json "$CHECKSUMS") \
    || fail "SHA256SUMS must contain exactly one RELEASE-PROVENANCE.json entry"
validate_checksum "$EXPECTED_PROVENANCE_SHA" \
    || fail "SHA256SUMS contains an invalid provenance checksum"
ACTUAL_PROVENANCE_SHA=$(hash_file "$PROVENANCE") \
    || fail "sha256sum or shasum is required"
[ "$ACTUAL_PROVENANCE_SHA" = "$EXPECTED_PROVENANCE_SHA" ] \
    || fail "release provenance checksum verification failed"

EXPECTED_ASSET_SHA=$(checksum_for "$ASSET" "$CHECKSUMS") \
    || fail "SHA256SUMS must contain exactly one '$ASSET' entry"
validate_checksum "$EXPECTED_ASSET_SHA" \
    || fail "SHA256SUMS contains an invalid checksum for '$ASSET'"

mkdir -p -- "$DOWNLOAD_DIR" "$MANAGED_BIN_DIR"
BINARY_TMP="$DOWNLOAD_DIR/.${ASSET}.$$.tmp"
rm -f -- "$BINARY_TMP"
say "Downloading $PRODUCT $VERSION ($OS-$ARCH)..."
curl_download "$RELEASE_BASE/$ASSET" "$BINARY_TMP" \
    || fail "could not download release asset '$ASSET'"
chmod 0755 "$BINARY_TMP"

ACTUAL_ASSET_SHA=$(hash_file "$BINARY_TMP") \
    || fail "sha256sum or shasum is required"
[ "$ACTUAL_ASSET_SHA" = "$EXPECTED_ASSET_SHA" ] \
    || fail "checksum verification failed for '$ASSET'"

SMOKE_OUTPUT="$TEMP_DIR/version-output.txt"
smoke_test() {
    binary=$1
    output=$2
    "$binary" version </dev/null >"$output" 2>&1 &
    smoke_pid=$!
    attempts=0
    while kill -0 "$smoke_pid" 2>/dev/null; do
        if [ "$attempts" -ge 100 ]; then
            kill -TERM "$smoke_pid" 2>/dev/null || :
            wait "$smoke_pid" 2>/dev/null || :
            return 1
        fi
        sleep 0.1
        attempts=$((attempts + 1))
    done

    if wait "$smoke_pid"; then
        smoke_status=0
    else
        smoke_status=$?
    fi
    [ "$smoke_status" -eq 0 ] || return 1
    grep -F -x "$PRODUCT $VERSION" "$output" >/dev/null 2>&1
}

smoke_test "$BINARY_TMP" "$SMOKE_OUTPUT" \
    || fail "downloaded asset failed the Enhanced identity/version smoke test"

managed_link_is_replaceable() {
    link=$1
    [ -e "$link" ] || [ -L "$link" ] || return 0
    if [ -L "$link" ]; then
        target=$(readlink "$link" 2>/dev/null || printf '')
        case "$target" in
            ../downloads/grok-*|"$DOWNLOAD_DIR"/grok-*) return 0 ;;
        esac
    fi
    [ "$FORCE" -eq 1 ] || return 1
    return 0
}

for managed_link in "$MANAGED_BIN_DIR/grok" "$MANAGED_BIN_DIR/agent"; do
    if [ -d "$managed_link" ]; then
        fail "refusing to replace directory or directory symlink '$managed_link'"
    fi
    managed_link_is_replaceable "$managed_link" \
        || fail "refusing to replace unmanaged path '$managed_link' (use --force to override)"
done

if [ -n "$FRONT_BIN_DIR" ] && [ "$FRONT_BIN_DIR" != "$MANAGED_BIN_DIR" ]; then
    mkdir -p -- "$FRONT_BIN_DIR"
    for name in grok agent; do
        front_link="$FRONT_BIN_DIR/$name"
        if [ -d "$front_link" ]; then
            fail "refusing to replace directory or directory symlink '$front_link'"
        fi
        if [ -e "$front_link" ] || [ -L "$front_link" ]; then
            if [ -L "$front_link" ]; then
                front_target=$(readlink "$front_link" 2>/dev/null || printf '')
            else
                front_target=
            fi
            if [ "$front_target" != "$MANAGED_BIN_DIR/$name" ] && [ "$FORCE" -ne 1 ]; then
                fail "refusing to replace '$front_link' (use --force to override)"
            fi
        fi
    done
fi

FINAL_BINARY="$DOWNLOAD_DIR/$ASSET"
if [ -L "$FINAL_BINARY" ] || { [ -e "$FINAL_BINARY" ] && [ ! -f "$FINAL_BINARY" ]; }; then
    fail "release destination is not a regular file: $FINAL_BINARY"
fi
if [ -f "$FINAL_BINARY" ]; then
    EXISTING_SHA=$(hash_file "$FINAL_BINARY" 2>/dev/null || printf '')
else
    EXISTING_SHA=
fi
if [ "$EXISTING_SHA" = "$EXPECTED_ASSET_SHA" ]; then
    rm -f -- "$BINARY_TMP"
    BINARY_TMP=
else
    mv -f -- "$BINARY_TMP" "$FINAL_BINARY"
    BINARY_TMP=
fi
chmod 0755 "$FINAL_BINARY"

atomic_link() {
    target=$1
    link=$2
    LINK_TMP="${link}.tmp.$$"
    rm -f -- "$LINK_TMP"
    ln -s "$target" "$LINK_TMP"
    mv -f -- "$LINK_TMP" "$link"
    LINK_TMP=
}

RELATIVE_TARGET="../downloads/$ASSET"
atomic_link "$RELATIVE_TARGET" "$MANAGED_BIN_DIR/grok"
atomic_link "$RELATIVE_TARGET" "$MANAGED_BIN_DIR/agent"

if [ -n "$FRONT_BIN_DIR" ] && [ "$FRONT_BIN_DIR" != "$MANAGED_BIN_DIR" ]; then
    for name in grok agent; do
        atomic_link "$MANAGED_BIN_DIR/$name" "$FRONT_BIN_DIR/$name"
    done
fi

# Generate completions without leaving empty files after an unsupported shell.
COMPLETIONS_DIR="$GROK_HOME_DIR/completions"
mkdir -p -- "$COMPLETIONS_DIR/bash" "$COMPLETIONS_DIR/zsh"
completion_tmp="$COMPLETIONS_DIR/bash/.grok.bash.$$"
if "$MANAGED_BIN_DIR/grok" completions bash >"$completion_tmp" 2>/dev/null; then
    chmod 0644 "$completion_tmp"
    mv -f -- "$completion_tmp" "$COMPLETIONS_DIR/bash/grok.bash"
else
    rm -f -- "$completion_tmp"
fi
completion_tmp="$COMPLETIONS_DIR/zsh/._grok.$$"
if "$MANAGED_BIN_DIR/grok" completions zsh >"$completion_tmp" 2>/dev/null; then
    chmod 0644 "$completion_tmp"
    mv -f -- "$completion_tmp" "$COMPLETIONS_DIR/zsh/_grok"
else
    rm -f -- "$completion_tmp"
fi
FISH_COMPLETIONS_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/fish/completions"
if mkdir -p -- "$FISH_COMPLETIONS_DIR" 2>/dev/null; then
    completion_tmp="$FISH_COMPLETIONS_DIR/.grok.fish.$$"
    if "$MANAGED_BIN_DIR/grok" completions fish >"$completion_tmp" 2>/dev/null; then
        chmod 0644 "$completion_tmp"
        mv -f -- "$completion_tmp" "$FISH_COMPLETIONS_DIR/grok.fish"
    else
        rm -f -- "$completion_tmp"
    fi
fi

path_contains() {
    directory=$1
    case ":${PATH:-}:" in
        *:"$directory":*) return 0 ;;
        *) return 1 ;;
    esac
}

PATH_READY=0
path_contains "$MANAGED_BIN_DIR" && PATH_READY=1
if [ -n "$FRONT_BIN_DIR" ] && path_contains "$FRONT_BIN_DIR"; then
    PATH_READY=1
fi
PROFILE_UPDATED=0

if [ "$PATH_READY" -eq 0 ] && [ "$MODIFY_PATH" -eq 1 ]; then
    if [ "$GROK_HOME_DIR" = "$HOME/.grok" ]; then
        shell_name=$(basename "${SHELL:-}")
        case "$shell_name" in
            zsh)
                profile="$HOME/.zshrc"
                profile_block='# >>> grok build enhanced installer >>>
export PATH="$HOME/.grok/bin:$PATH"
fpath=("$HOME/.grok/completions/zsh" $fpath)
autoload -Uz compinit && compinit -C
# <<< grok build enhanced installer <<<'
                ;;
            bash)
                if [ "$OS" = macos ]; then
                    profile="$HOME/.bash_profile"
                else
                    profile="$HOME/.bashrc"
                fi
                profile_block='# >>> grok build enhanced installer >>>
export PATH="$HOME/.grok/bin:$PATH"
[ -r "$HOME/.grok/completions/bash/grok.bash" ] && . "$HOME/.grok/completions/bash/grok.bash"
# <<< grok build enhanced installer <<<'
                ;;
            fish)
                profile="$HOME/.config/fish/config.fish"
                profile_block='# >>> grok build enhanced installer >>>
fish_add_path "$HOME/.grok/bin"
# <<< grok build enhanced installer <<<'
                ;;
            *)
                profile=
                profile_block=
                ;;
        esac
        if [ -n "$profile" ]; then
            mkdir -p -- "$(dirname "$profile")"
            if ! grep -F '# >>> grok build enhanced installer >>>' "$profile" >/dev/null 2>&1; then
                printf '\n%s\n' "$profile_block" >>"$profile"
                PROFILE_UPDATED=1
            fi
        fi
    fi
fi

say ""
say "Installed $PRODUCT $VERSION"
say "  Binary: $MANAGED_BIN_DIR/grok"
say "  Updates: grok update"

if command -v grok >/dev/null 2>&1; then
    RESOLVED_GROK=$(command -v grok)
    case "$RESOLVED_GROK" in
        "$MANAGED_BIN_DIR/grok"|"$FRONT_BIN_DIR/grok") ;;
        *)
            say ""
            say "Warning: '$RESOLVED_GROK' currently appears before the Enhanced install on PATH."
            say "Run 'type -a grok' after restarting your shell to inspect PATH order."
            ;;
    esac
fi

if [ "$PATH_READY" -eq 1 ]; then
    say ""
    say "Run 'grok version' to verify the Enhanced installation."
elif [ "$PROFILE_UPDATED" -eq 1 ]; then
    say ""
    say "Restart your shell, then run 'grok version'."
else
    say ""
    say "Add '$MANAGED_BIN_DIR' to PATH, then run 'grok version':"
    say "  export PATH=\"$MANAGED_BIN_DIR:\$PATH\""
fi
