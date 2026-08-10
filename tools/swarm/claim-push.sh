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

echo "claim-push: fetching ${REMOTE}"
"$GIT_REAL" fetch --prune "$REMOTE"

# Re-verify at the moment of push — stale green is not authorization.
# Lease the remote integ tip so a concurrent claim fails closed.
echo "claim-push: pushing HEAD → ${REMOTE}/${BRANCH} (--force-with-lease)"
"$GIT_REAL" push --force-with-lease="refs/heads/${BRANCH}" \
  "$REMOTE" "HEAD:refs/heads/${BRANCH}"

echo "claim-push: ok — ${REMOTE}/${BRANCH}"
