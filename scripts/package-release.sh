#!/usr/bin/env bash

set -euo pipefail

usage() {
  printf '%s\n' "Usage: scripts/package-release.sh [--target TARGET] [--output DIRECTORY]"
}

host_target() {
  rustc -vV | awk '/^host: / { print $2; exit }'
}

package_version() {
  awk '
    $0 == "[package]" { in_package = 1; next }
    in_package && /^\[/ { exit }
    in_package && /^version[[:space:]]*=/ {
      match($0, /"[^"]+"/)
      print substr($0, RSTART + 1, RLENGTH - 2)
      exit
    }
  ' Cargo.toml
}

target="$(host_target)"
output_directory="dist"

while (($# > 0)); do
  case "$1" in
    --target)
      target="${2:?--target requires a Rust target triple}"
      shift 2
      ;;
    --output)
      output_directory="${2:?--output requires a directory}"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      usage >&2
      exit 2
      ;;
  esac
done

case "$target" in
  *-linux-*|*-apple-darwin)
    binary_name="lens"
    ;;
  *-pc-windows-gnu|*-pc-windows-msvc)
    binary_name="lens.exe"
    ;;
  *)
    printf 'Lens release packaging does not support target: %s\n' "$target" >&2
    exit 2
    ;;
esac

version="$(package_version)"
if [[ -z "$version" ]]; then
  printf '%s\n' 'Could not read the Lens package version from Cargo.toml.' >&2
  exit 1
fi

package_name="lens-${version}-${target}"
archive="${output_directory}/${package_name}.tar.gz"
checksum="${archive}.sha256"
if [[ -e "$archive" || -e "$checksum" ]]; then
  printf '%s\n' "Refusing to overwrite an existing release artifact in ${output_directory}." >&2
  exit 1
fi

staging_directory="$(mktemp -d)"
trap 'rm -rf "$staging_directory"' EXIT

cargo build --locked --release --target "$target"

binary="target/${target}/release/${binary_name}"
if [[ ! -f "$binary" ]]; then
  printf '%s\n' "Expected release binary was not created: ${binary}" >&2
  exit 1
fi

mkdir -p "$output_directory"
archive_directory="${staging_directory}/${package_name}"
mkdir -p "$archive_directory"
cp "$binary" "${archive_directory}/${binary_name}"
if [[ "$binary_name" == "lens" ]]; then
  chmod 755 "${archive_directory}/lens"
fi
cp README.md LICENSE "$archive_directory"
tar -C "$staging_directory" -czf "$archive" "$package_name"
(
  cd "$output_directory"
  sha256sum "${package_name}.tar.gz" > "${package_name}.tar.gz.sha256"
)

printf 'Created %s and %s\n' "$archive" "$checksum"
