#!/usr/bin/env bash
# claim-push.sh — blessed integrator push of HEAD → integ/<root> with lease.
#
# Usage: claim-push.sh [--check] <root> [remote]
# Example: claim-push.sh os
#          claim-push.sh --check specs
#          claim-push.sh ci origin
#
# --check: run envelope + merge-tree preflight only; do not push (claim-mechanical).
#
# Calls real git (never the lane shim). Sets SWARM_BLESSED_PUSH=1 in case PATH
# still has the shim ahead of GIT_REAL resolution for nested helpers.
#
# Re-runs read-only merge-tree against the pinned integ tip BEFORE push so an
# ambient HEAD change after agent Claim cannot publish an unauthorized tip
# (fix-1644-review / deliver merge-tree).
set -euo pipefail

CHECK_ONLY=0
if [[ "${1:-}" == "--check" ]]; then
  CHECK_ONLY=1
  shift
fi

ROOT="${1:?usage: claim-push.sh [--check] <integ-root> [remote]}"
REMOTE="${2:-origin}"
BRANCH="integ/${ROOT}"

GIT_REAL="${GIT_REAL:-/usr/bin/git}"
if [[ ! -x "$GIT_REAL" ]]; then
  GIT_REAL="$(PATH="/usr/bin:/bin:/opt/homebrew/bin:/usr/local/bin" command -v git)"
fi

export SWARM_BLESSED_PUSH=1

REPO_ROOT="$("$GIT_REAL" rev-parse --show-toplevel)"
ENVELOPES="${REPO_ROOT}/specs/integ-branch-envelopes.json"
if [[ ! -f "$ENVELOPES" ]]; then
  echo "claim-push: REFUSE — missing envelopes at ${ENVELOPES}" >&2
  exit 1
fi

# Pin the lease to the tip we last observed BEFORE fetch. An ambient fetch that
# moves origin/integ/<root> must not silently raise the lease baseline and
# authorize overwriting a tip we never reviewed (`--force-with-lease=<ref>:<expect>`).
EXPECTED=""
if "$GIT_REAL" rev-parse --verify --quiet "${REMOTE}/${BRANCH}^{commit}" >/dev/null; then
  EXPECTED="$("$GIT_REAL" rev-parse "${REMOTE}/${BRANCH}^{commit}")"
fi

echo "claim-push: fetching ${REMOTE}"
"$GIT_REAL" fetch --prune "$REMOTE"

DEV_TIP="$("$GIT_REAL" rev-parse "${REMOTE}/dev^{commit}")"
HEAD_SHA="$("$GIT_REAL" rev-parse 'HEAD^{commit}')"
INTEG_TIP="${EXPECTED:-$DEV_TIP}"

BASE_SHA="$("$GIT_REAL" merge-base "$DEV_TIP" "$HEAD_SHA")"
echo "claim-push: merge-tree preflight BASE=${BASE_SHA:0:12} INTEG_TIP=${INTEG_TIP:0:12} HEAD=${HEAD_SHA:0:12}"
MERGE_OUT="$("$GIT_REAL" merge-tree "$BASE_SHA" "$INTEG_TIP" "$HEAD_SHA" 2>&1 || true)"
if printf '%s\n' "$MERGE_OUT" | grep -E -q '^(<<<<<<<|=======|>>>>>>>)|changed in both|CONFLICT'; then
  echo "claim-push: REFUSE — merge-tree reports content conflict against ${BRANCH}" >&2
  printf '%s\n' "$MERGE_OUT" | head -n 80 >&2
  exit 1
fi
echo "claim-push: merge-tree clean"

# Soft registered-root check: ROOT must appear in envelopes#roots or #planes.
if ! python3 - "$ENVELOPES" "$ROOT" "$BRANCH" <<'PY'
import json, sys
path, root, branch = sys.argv[1:4]
with open(path) as f:
    e = json.load(f)
roots = e.get("roots", {})
planes = e.get("planes", {})
ok = root in roots or any(p.get("branch") == branch for p in planes.values())
if not ok:
    print(f"claim-push: REFUSE — {branch} not in envelopes#roots/#planes", file=sys.stderr)
    sys.exit(1)
print(f"claim-push: envelope root/plane ok for {branch}")
PY
then
  exit 1
fi

if (( CHECK_ONLY == 1 )); then
  echo "claim-push: --check OK — envelope + merge-tree clean for ${BRANCH} (no push)"
  exit 0
fi

echo "claim-push: pushing HEAD → ${REMOTE}/${BRANCH} (--force-with-lease)"
if [[ -n "${EXPECTED}" ]]; then
  "$GIT_REAL" push --force-with-lease="refs/heads/${BRANCH}:${EXPECTED}" \
    "$REMOTE" "HEAD:refs/heads/${BRANCH}"
else
  # First push — no prior tip to lease against.
  "$GIT_REAL" push "$REMOTE" "HEAD:refs/heads/${BRANCH}"
fi

echo "claim-push: ok — ${REMOTE}/${BRANCH}"
