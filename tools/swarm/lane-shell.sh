#!/usr/bin/env bash
# lane-shell.sh — start a worker shell with swarm shims first on PATH.
#
# Exports:
#   PATH          — tools/swarm/shim-bin ahead of the rest (git/cargo/buck2)
#   GIT_REAL      — absolute path to real git (shim forwards here; pinned here)
#   SWARM_LANE=1  — marker for tooling
# Unsets:
#   SWARM_BLESSED_PUSH — never inherit a push-bypass flag into a worker shell
#   SWARM_ORCHESTRATOR — workers stay denied for cargo/buck2
#
# Refuses:
#   main checkout (must be a linked station under .worktrees/)
#   ambient GIT_REAL overrides (always re-resolve from a fixed allowlist)
#
# Usage:
#   ./tools/swarm/lane-shell.sh              # interactive $SHELL
#   ./tools/swarm/lane-shell.sh -- cmd...    # run one command under shims
set -euo pipefail

SWARM_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SHIM_BIN="${SWARM_DIR}/shim-bin"

resolve_real_git() {
  if [[ -x /usr/bin/git ]]; then
    printf '%s\n' /usr/bin/git
    return 0
  fi
  PATH="/usr/bin:/bin:/opt/homebrew/bin:/usr/local/bin" command -v git
}

# Resolve real git BEFORE putting shims on PATH (avoid recursion).
# Ambient GIT_REAL retarget is an env-escape — refuse rather than silently honor.
_RESOLVED_GIT="$(resolve_real_git || true)"
if [[ -z "${_RESOLVED_GIT}" || ! -x "${_RESOLVED_GIT}" ]]; then
  echo "lane-shell: REFUSE — cannot resolve real git" >&2
  exit 127
fi
if [[ -n "${GIT_REAL:-}" && "${GIT_REAL}" != "${_RESOLVED_GIT}" ]]; then
  echo "lane-shell: REFUSE — ambient GIT_REAL=${GIT_REAL} is banned in lane shells (env-escape)" >&2
  echo "lane-shell: real git is pinned from /usr/bin/git (or PATH allowlist); do not override" >&2
  exit 1
fi
case "${_RESOLVED_GIT}" in
  /usr/bin/git|/bin/git|/opt/homebrew/bin/git|/usr/local/bin/git) ;;
  *)
    echo "lane-shell: REFUSE — resolved git ${_RESOLVED_GIT} is outside the fixed allowlist" >&2
    exit 1
    ;;
esac
GIT_REAL="${_RESOLVED_GIT}"
export GIT_REAL
unset _RESOLVED_GIT

# Isolation: workers must not share the main checkout index.
REPO_ROOT="$("$GIT_REAL" rev-parse --show-toplevel 2>/dev/null || true)"
if [[ -z "${REPO_ROOT}" ]]; then
  echo "lane-shell: REFUSE — not inside a git worktree" >&2
  exit 1
fi
case "${REPO_ROOT}" in
  */.worktrees/*) ;;
  *)
    echo "lane-shell: REFUSE — main checkout detected at ${REPO_ROOT}" >&2
    echo "lane-shell: start from a linked station under .worktrees/<lane> (one writer per worktree)" >&2
    exit 1
    ;;
esac

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

export PATH="${SHIM_BIN}:${PATH}"
export SWARM_LANE=1
unset SWARM_ORCHESTRATOR || true
# Close env-escape: a parent shell exporting SWARM_BLESSED_PUSH=1 must not
# authorize bare `git push` inside the lane (shim denies push entirely; still unset).
unset SWARM_BLESSED_PUSH || true

echo "lane-shell: PATH shims active (git/cargo/buck2)  GIT_REAL=${GIT_REAL}" >&2
echo "lane-shell: SWARM_BLESSED_PUSH unset; push only via tools/swarm/claim-push.sh" >&2
echo "lane-shell: read err.txt at main checkout root; do not run cargo/buck2" >&2

if [[ "${1:-}" == "--" ]]; then
  shift
  exec "$@"
fi

exec "${SHELL:-/bin/bash}" "$@"
