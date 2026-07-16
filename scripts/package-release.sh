#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: bash scripts/package-release.sh TARGET [OUTPUT_DIR]" >&2
  exit 2
fi

target="$1"
output_dir="${2:-dist}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

version="$(node --input-type=module -e "import fs from 'node:fs'; const text = fs.readFileSync('Cargo.toml', 'utf8'); const match = text.match(/^version\\s*=\\s*\\\"([^\\\"]+)\\\"/m); if (!match) process.exit(1); process.stdout.write(match[1]);")"
cargo build --locked --release --target "$target"

binary="target/$target/release/lens"
if [[ "$target" == *windows* ]]; then
  binary="${binary}.exe"
fi
if [[ ! -f "$binary" ]]; then
  echo "release binary not found: $binary" >&2
  exit 1
fi

archive_root="$output_dir/lens-${version}-${target}"
rm -rf "$archive_root"
mkdir -p "$archive_root"
cp "$binary" "$archive_root/"
cp README.md "$archive_root/"
archive="$output_dir/lens-${version}-${target}.tar.gz"
mkdir -p "$output_dir"
tar -czf "$archive" -C "$output_dir" "lens-${version}-${target}"
if command -v sha256sum >/dev/null 2>&1; then
  sha256sum "$archive" > "${archive}.sha256"
else
  shasum -a 256 "$archive" > "${archive}.sha256"
fi
echo "created $archive and ${archive}.sha256"
