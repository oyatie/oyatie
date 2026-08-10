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
#
# Fail-closed: refuses to reset unless the current integ tip is already an
# ancestor of origin/dev (i.e. the integration PR has landed on trunk).
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

if ! "$GIT_REAL" rev-parse --verify --quiet "${REMOTE}/${BRANCH}^{commit}" >/dev/null; then
  echo "integ-reset-remote: missing ${REMOTE}/${BRANCH} — nothing to reset" >&2
  exit 1
fi

INTEG_TIP="$("$GIT_REAL" rev-parse "${REMOTE}/${BRANCH}^{commit}")"
DEV_TIP="$("$GIT_REAL" rev-parse "${REMOTE}/dev^{commit}")"

if ! "$GIT_REAL" merge-base --is-ancestor "${INTEG_TIP}" "${DEV_TIP}"; then
  echo "integ-reset-remote: REFUSE — ${BRANCH} tip ${INTEG_TIP:0:12} is not an ancestor of ${REMOTE}/dev ${DEV_TIP:0:12}" >&2
  echo "integ-reset-remote: land the integ PR (squash-merge into dev) before resetting the durable branch" >&2
  exit 1
fi

echo "integ-reset-remote: ${REMOTE}/dev → refs/heads/${BRANCH} (--force-with-lease=${INTEG_TIP:0:12})"
"$GIT_REAL" push --force-with-lease="refs/heads/${BRANCH}:${INTEG_TIP}" \
  "$REMOTE" "${REMOTE}/dev:refs/heads/${BRANCH}"

echo "integ-reset-remote: ok — ${BRANCH} now matches ${REMOTE}/dev"
