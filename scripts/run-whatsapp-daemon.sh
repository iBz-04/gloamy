#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if [[ -f .env ]]; then
  set -a
  # shellcheck source=/dev/null
  source .env
  set +a
fi

: "${GLOAMY_CONFIG_DIR:=/Users/ibz/.gloamy}"
: "${GLOAMY_WORKSPACE:=/Users/ibz/Desktop/reademy}"

exec cargo run --features whatsapp-web -- daemon
