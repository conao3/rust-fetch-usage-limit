#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "[1/3] nix build .#default"
nix build .#default >/dev/null

echo "[2/3] check exit code when ANTHROPIC_OAUTH_API_KEY is missing"
set +e
OUTPUT=$(nix run .#default -- claude 2>&1)
STATUS=$?
set -e

echo "$OUTPUT"
if [ "$STATUS" -ne 2 ]; then
  echo "expected exit code 2, got $STATUS"
  exit 1
fi

echo "[3/4] check codex subcommand is exposed"
HELP_OUTPUT=$(nix run .#default -- --help 2>&1)
echo "$HELP_OUTPUT" | grep -q "codex"

echo "[4/4] package verification passed"
