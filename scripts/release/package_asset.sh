#!/usr/bin/env bash
# Package one native xai-grok-pager build under the updater's raw grok asset name.
set -euo pipefail

usage() {
    cat >&2 <<'EOF'
Usage: package_asset.sh VERSION OS ARCH RUST_TARGET BINARY OUTPUT_DIR SOURCE_COMMIT REPOSITORY
EOF
}

if [[ $# -ne 8 ]]; then
    usage
    exit 2
fi

version=$1
os_name=$2
arch=$3
rust_target=$4
binary=$5
output_dir=$6
source_commit=$7
repository=$8
script_dir=$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
contract="$script_dir/release_contract.py"

if [[ ! -f "$contract" || -L "$contract" ]]; then
    echo "release contract script must be a regular file: $contract" >&2
    exit 1
fi
if [[ ! -f "$binary" || -L "$binary" ]]; then
    echo "compiled binary must be a regular file, not a symlink: $binary" >&2
    exit 1
fi
if [[ ! -x "$binary" ]]; then
    echo "compiled binary is not executable: $binary" >&2
    exit 1
fi
if [[ -L "$output_dir" ]]; then
    echo "output directory may not be a symlink: $output_dir" >&2
    exit 1
fi

asset_name=$(
    python3 "$contract" asset-name \
        --version "$version" \
        --os "$os_name" \
        --arch "$arch" \
        --rust-target "$rust_target" \
        --repository "$repository"
)
record_name="asset-metadata-${os_name}-${arch}.json"

umask 022
mkdir -p -- "$output_dir"
output_dir=$(CDPATH= cd -- "$output_dir" && pwd -P)
destination="$output_dir/$asset_name"
record="$output_dir/$record_name"
temporary="$output_dir/.${asset_name}.tmp.$$"

if [[ -e "$destination" || -L "$destination" || -e "$record" || -L "$record" ]]; then
    echo "refusing to replace an existing packaged asset or record in $output_dir" >&2
    exit 1
fi

cleanup() {
    rm -f -- "$temporary"
}
trap cleanup EXIT HUP INT TERM

cp -- "$binary" "$temporary"
chmod 0755 "$temporary"
mv -- "$temporary" "$destination"

if ! python3 "$contract" write-record \
    --version "$version" \
    --repository "$repository" \
    --source-commit "$source_commit" \
    --os "$os_name" \
    --arch "$arch" \
    --rust-target "$rust_target" \
    --asset "$destination" \
    --output "$record"; then
    rm -f -- "$destination" "$record"
    exit 1
fi

printf 'packaged %s\n' "$destination"
