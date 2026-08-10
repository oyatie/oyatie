#!/usr/bin/env bash
# integ-reset-remote.sh — server-side reset of integ/<root> to origin/dev.
#
# After squash-merge, reset the remote integ with a push refspec — no local
# `git reset` anywhere. Branch name persists; next wave reuses it.
#
# Usage: integ-reset-remote.sh <root> [remote]
# Example: integ-reset-remote.sh os
#
# Equivalent refspec:
#   git push --force-with-lease origin origin/dev:refs/heads/integ/<root>
set -euo pipefail

ROOT="${1:?usage: integ-reset-remote.sh <integ-root> [remote]}"
REMOTE="${2:-origin}"
BRANCH="integ/${ROOT}"

GIT_REAL="${GIT_REAL:-/usr/bin/git}"
if [[ ! -x "$GIT_REAL" ]]; then
  GIT_REAL="$(PATH="/usr/bin:/bin:/opt/homebrew/bin:/usr/local/bin" command -v git)"
fi

export SWARM_BLESSED_PUSH=1

echo "integ-reset-remote: fetching ${REMOTE}"
"$GIT_REAL" fetch --prune "$REMOTE"

if ! "$GIT_REAL" rev-parse --verify --quiet "${REMOTE}/dev^{commit}" >/dev/null; then
  echo "integ-reset-remote: missing ${REMOTE}/dev — fetch failed or remote has no dev" >&2
  exit 1
fi

echo "integ-reset-remote: ${REMOTE}/dev → refs/heads/${BRANCH} (--force-with-lease)"
"$GIT_REAL" push --force-with-lease \
  "$REMOTE" "${REMOTE}/dev:refs/heads/${BRANCH}"

echo "integ-reset-remote: ok — ${BRANCH} now matches ${REMOTE}/dev"
