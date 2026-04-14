#!/usr/bin/env bash
set -euo pipefail

# Preflight helper for running a reproducible WebArena benchmark setup.
# This script does not run the full benchmark; it validates prerequisites and
# prints canonical next commands.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
WEBARENA_DIR="${WEBARENA_DIR:-$HOME/webarena}"

missing=0
for cmd in git python3 docker; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "[missing] $cmd"
    missing=1
  fi
done

if [[ $missing -ne 0 ]]; then
  echo
  echo "Prerequisites missing. Install the missing tools and re-run this script."
  echo "Required: git, python3, docker"
  exit 1
fi

echo "[ok] prerequisites present"

if [[ ! -d "$WEBARENA_DIR/.git" ]]; then
  echo "[step] cloning WebArena into $WEBARENA_DIR"
  git clone https://github.com/web-arena-x/webarena "$WEBARENA_DIR"
else
  echo "[ok] existing WebArena checkout found at $WEBARENA_DIR"
fi

cat <<'EOF'

Next steps (official WebArena flow):

1) Python environment and deps
   cd "$WEBARENA_DIR"
   python3 -m venv .venv
   source .venv/bin/activate
   pip install -r requirements.txt
   playwright install
   pip install -e .

2) Standalone websites (Docker)
   Follow environment setup in environment_docker/README.md
   Recommended path from upstream: pre-installed AMI or full dockerized setup.

3) Set benchmark website URLs
   export SHOPPING="<host>:7770"
   export SHOPPING_ADMIN="<host>:7780/admin"
   export REDDIT="<host>:9999"
   export GITLAB="<host>:8023"
   export MAP="<host>:3000"
   export WIKIPEDIA="<host>:8888/wikipedia_en_all_maxi_2022-05/A/User:The_other_Kiwix_guy/Landing"
   export HOMEPAGE="<host>:4399"

4) Generate task configs and auth cookies
   python scripts/generate_test_data.py
   mkdir -p .auth
   python browser_env/auto_login.py

5) Run a smoke benchmark (first 2 tasks)
   export OPENAI_API_KEY="<your-key>"
   python run.py \
     --instruction_path agent/prompts/jsons/p_cot_id_actree_2s.json \
     --test_start_idx 0 \
     --test_end_idx 2 \
     --model gpt-4o \
     --result_dir results/smoke

Notes:
- Upstream recommends AgentLab/BrowserGym for newer experiments.
- To benchmark Gloamy itself, a Gloamy-compatible WebArena agent adapter is required.
EOF

echo
echo "Saved helper in: $ROOT_DIR/scripts/benchmarks/webarena_preflight.sh"
