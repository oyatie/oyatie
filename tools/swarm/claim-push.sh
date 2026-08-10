#!/usr/bin/env bash
# claim-push.sh — blessed integrator push of HEAD → integ/<root> with lease.
#
# Usage: claim-push.sh <root> [remote]
# Example: claim-push.sh os
#          claim-push.sh ci origin
#
# Calls real git (never the lane shim). Sets SWARM_BLESSED_PUSH=1 in case PATH
# still has the shim ahead of GIT_REAL resolution for nested helpers.
set -euo pipefail

ROOT="${1:?usage: claim-push.sh <integ-root> [remote]}"
REMOTE="${2:-origin}"
BRANCH="integ/${ROOT}"

GIT_REAL="${GIT_REAL:-/usr/bin/git}"
if [[ ! -x "$GIT_REAL" ]]; then
  GIT_REAL="$(PATH="/usr/bin:/bin:/opt/homebrew/bin:/usr/local/bin" command -v git)"
fi

export SWARM_BLESSED_PUSH=1

# Pin the lease to the tip we last observed BEFORE fetch. An ambient fetch that
# moves origin/integ/<root> must not silently raise the lease baseline and
# authorize overwriting a tip we never reviewed (`--force-with-lease=<ref>:<expect>`).
EXPECTED=""
if "$GIT_REAL" rev-parse --verify --quiet "${REMOTE}/${BRANCH}^{commit}" >/dev/null; then
  EXPECTED="$("$GIT_REAL" rev-parse "${REMOTE}/${BRANCH}^{commit}")"
fi

echo "claim-push: fetching ${REMOTE}"
"$GIT_REAL" fetch --prune "$REMOTE"

echo "claim-push: pushing HEAD → ${REMOTE}/${BRANCH} (--force-with-lease)"
if [[ -n "${EXPECTED}" ]]; then
  "$GIT_REAL" push --force-with-lease="refs/heads/${BRANCH}:${EXPECTED}" \
    "$REMOTE" "HEAD:refs/heads/${BRANCH}"
else
  # First push — no prior tip to lease against.
  "$GIT_REAL" push "$REMOTE" "HEAD:refs/heads/${BRANCH}"
fi

echo "claim-push: ok — ${REMOTE}/${BRANCH}"
