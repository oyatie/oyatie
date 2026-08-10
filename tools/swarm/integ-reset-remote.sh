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
# Fail-closed: refuses to reset unless the integ PR has landed on trunk.
# Squash merges do NOT preserve integ tip ancestry on origin/dev, so ancestry
# alone is insufficient — also accept when the integ tip's tree appears on
# recent origin/dev commits (squash land proof).
set -euo pipefail

ROOT="${1:?usage: integ-reset-remote.sh <integ-root> [remote]}"
REMOTE="${2:-origin}"
BRANCH="integ/${ROOT}"

resolve_real_git() {
  if [[ -x /usr/bin/git ]]; then
    printf '%s\n' /usr/bin/git
    return 0
  fi
  PATH="/usr/bin:/bin:/opt/homebrew/bin:/usr/local/bin" command -v git
}

GIT_REAL="$(resolve_real_git)"
if [[ ! -x "$GIT_REAL" ]]; then
  echo "integ-reset-remote: REFUSE — cannot resolve real git" >&2
  exit 127
fi

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
INTEG_TREE="$("$GIT_REAL" rev-parse "${INTEG_TIP}^{tree}")"

landed=0
if "$GIT_REAL" merge-base --is-ancestor "${INTEG_TIP}" "${DEV_TIP}"; then
  landed=1
  echo "integ-reset-remote: land proof — ${BRANCH} tip is ancestor of ${REMOTE}/dev (FF/merge)"
else
  # Squash land: tip SHA is not an ancestor, but its tree landed on trunk.
  if "$GIT_REAL" log --format='%T' -n 200 "${DEV_TIP}" | grep -Fxq "${INTEG_TREE}"; then
    landed=1
    echo "integ-reset-remote: land proof — ${BRANCH} tree ${INTEG_TREE:0:12} appears on recent ${REMOTE}/dev (squash)"
  fi
fi

if (( landed != 1 )); then
  echo "integ-reset-remote: REFUSE — ${BRANCH} tip ${INTEG_TIP:0:12} not proven landed on ${REMOTE}/dev ${DEV_TIP:0:12}" >&2
  echo "integ-reset-remote: need ancestor (FF/merge) OR integ tip tree on recent origin/dev (squash)" >&2
  echo "integ-reset-remote: land the integ PR (squash-merge into dev) before resetting the durable branch" >&2
  exit 1
fi

echo "integ-reset-remote: ${REMOTE}/dev → refs/heads/${BRANCH} (--force-with-lease=${INTEG_TIP:0:12})"
"$GIT_REAL" push --force-with-lease="refs/heads/${BRANCH}:${INTEG_TIP}" \
  "$REMOTE" "${REMOTE}/dev:refs/heads/${BRANCH}"

echo "integ-reset-remote: ok — ${BRANCH} now matches ${REMOTE}/dev"
