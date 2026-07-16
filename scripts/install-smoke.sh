#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: bash scripts/install-smoke.sh ARCHIVE" >&2
  exit 2
fi

archive="$1"
root="$(mktemp -d)"
trap 'rm -rf "$root"' EXIT
tar -xzf "$archive" -C "$root"

binary=""
for candidate in "$root"/*/lens "$root"/*/lens.exe; do
  if [[ -f "$candidate" ]]; then
    binary="$candidate"
    break
  fi
done
if [[ -z "$binary" ]]; then
  echo "archive does not contain a Lens binary" >&2
  exit 1
fi

workspace="$(dirname "$binary")"
"$binary" --help >/dev/null
"$binary" --no-open --port 39127 "$workspace" > "$root/lens.log" 2>&1 &
pid=$!
cleanup() {
  kill -INT "$pid" 2>/dev/null || true
  wait "$pid" 2>/dev/null || true
}
trap cleanup EXIT
for attempt in $(seq 1 30); do
  if curl --fail --silent --output /dev/null http://127.0.0.1:39127/; then
    echo "install smoke test passed"
    exit 0
  fi
  sleep 1
done
cat "$root/lens.log"
exit 1
