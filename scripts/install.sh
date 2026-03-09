#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" >/dev/null 2>&1 && pwd || pwd)"
INSTALLER_LOCAL="$(cd "$SCRIPT_DIR/.." >/dev/null 2>&1 && pwd || pwd)/gloamy_install.sh"
BOOTSTRAP_LOCAL="$SCRIPT_DIR/bootstrap.sh"
REPO_URL="https://github.com/iBz-04/gloamy.git"

echo "[deprecated] scripts/install.sh -> ./gloamy_install.sh" >&2

if [[ -x "$INSTALLER_LOCAL" ]]; then
  exec "$INSTALLER_LOCAL" "$@"
fi

if [[ -f "$BOOTSTRAP_LOCAL" ]]; then
  exec "$BOOTSTRAP_LOCAL" "$@"
fi

if ! command -v git >/dev/null 2>&1; then
  echo "error: git is required for legacy install.sh remote mode" >&2
  exit 1
fi

TEMP_DIR="$(mktemp -d -t gloamy-install-XXXXXX)"
cleanup() {
  rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

git clone --depth 1 "$REPO_URL" "$TEMP_DIR" >/dev/null 2>&1

if [[ -x "$TEMP_DIR/gloamy_install.sh" ]]; then
  exec "$TEMP_DIR/gloamy_install.sh" "$@"
fi

if [[ -x "$TEMP_DIR/scripts/bootstrap.sh" ]]; then
  exec "$TEMP_DIR/scripts/bootstrap.sh" "$@"
fi

echo "error: gloamy_install.sh/bootstrap.sh was not found in the fetched revision." >&2
echo "Run the local bootstrap directly when possible:" >&2
echo "  ./gloamy_install.sh --help" >&2
exit 1
