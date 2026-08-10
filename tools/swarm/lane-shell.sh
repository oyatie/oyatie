#!/usr/bin/env bash
# lane-shell.sh — start a worker shell with swarm shims first on PATH.
#
# Exports:
#   PATH          — tools/swarm/shim-bin ahead of the rest (git/cargo/buck2)
#   GIT_REAL      — absolute path to real git (shim forwards here)
#   SWARM_LANE=1  — marker for tooling
# Does NOT set SWARM_ORCHESTRATOR (workers stay denied for cargo/buck2).
#
# Usage:
#   ./tools/swarm/lane-shell.sh              # interactive $SHELL
#   ./tools/swarm/lane-shell.sh -- cmd...    # run one command under shims
set -euo pipefail

SWARM_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SHIM_BIN="${SWARM_DIR}/shim-bin"

# Relative symlinks so basename($0) stays git/cargo/buck2 without rewriting
# tracked targets to absolute machine paths (absolute targets dirty the tree).
mkdir -p "$SHIM_BIN"
(
  cd "$SHIM_BIN"
  ln -sfn ../git-shim git
  ln -sfn ../toolguard cargo
  ln -sfn ../toolguard buck2
)
chmod +x "$SWARM_DIR"/{git-shim,toolguard,check-daemon,claim-push.sh,integ-reset-remote.sh,lane-shell.sh}

# Resolve real git once for the shim.
if [[ -z "${GIT_REAL:-}" ]]; then
  if [[ -x /usr/bin/git ]]; then
    export GIT_REAL=/usr/bin/git
  else
    export GIT_REAL="$(PATH="/usr/bin:/bin:/opt/homebrew/bin:/usr/local/bin" command -v git)"
  fi
fi

export PATH="${SHIM_BIN}:${PATH}"
export SWARM_LANE=1
unset SWARM_ORCHESTRATOR || true

echo "lane-shell: PATH shims active (git/cargo/buck2)  GIT_REAL=${GIT_REAL}" >&2
echo "lane-shell: read err.txt at main checkout root; do not run cargo/buck2" >&2

if [[ "${1:-}" == "--" ]]; then
  shift
  exec "$@"
fi

exec "${SHELL:-/bin/bash}" "$@"
