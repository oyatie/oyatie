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
# Calls real git directly (never the lane shim). Does NOT use SWARM_BLESSED_PUSH —
# the shim denies push entirely; this script is the only admission path.
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

resolve_real_git() {
  if [[ -x /usr/bin/git ]]; then
    printf '%s\n' /usr/bin/git
    return 0
  fi
  PATH="/usr/bin:/bin:/opt/homebrew/bin:/usr/local/bin" command -v git
}

# Blessed scripts pin real git from the allowlist (ignore ambient GIT_REAL retarget).
GIT_REAL="$(resolve_real_git)"
if [[ ! -x "$GIT_REAL" ]]; then
  echo "claim-push: REFUSE — cannot resolve real git" >&2
  exit 127
fi

REPO_ROOT="$("$GIT_REAL" rev-parse --show-toplevel)"
ENVELOPES="${REPO_ROOT}/specs/integ-branch-envelopes.json"
if [[ ! -f "$ENVELOPES" ]]; then
  echo "claim-push: REFUSE — missing envelopes at ${ENVELOPES}" >&2
  exit 1
fi

# Dirty refuse (fix-1644-critic-rc): porcelain non-empty ⇒ REFUSE before fetch/push.
# One writer; ambient uncommitted edits must not ride a blessed lease push.
PORCELAIN="$("$GIT_REAL" -C "$REPO_ROOT" status --porcelain)"
if [[ -n "$PORCELAIN" ]]; then
  echo "claim-push: REFUSE — working tree dirty (git status --porcelain non-empty)" >&2
  printf '%s\n' "$PORCELAIN" | head -n 40 >&2
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
# Fast-forward onto the pinned integ tip needs no three-way: lease push already
# refuses if the tip moved. merge-tree against origin/dev's merge-base false-fires
# "added in both" on files born on the integ branch then edited in HEAD.
if [[ "$INTEG_TIP" == "$HEAD_SHA" ]] \
  || "$GIT_REAL" merge-base --is-ancestor "$INTEG_TIP" "$HEAD_SHA"; then
  echo "claim-push: merge-tree clean (HEAD fast-forward of integ tip)"
else
  MERGE_OUT="$("$GIT_REAL" merge-tree "$BASE_SHA" "$INTEG_TIP" "$HEAD_SHA" 2>&1 || true)"
  if printf '%s\n' "$MERGE_OUT" | grep -E -q '^(<<<<<<<|=======|>>>>>>>)|\+(<<<<<<<|=======|>>>>>>>)|changed in both|added in both|CONFLICT'; then
    echo "claim-push: REFUSE — merge-tree reports content conflict against ${BRANCH}" >&2
    printf '%s\n' "$MERGE_OUT" | head -n 80 >&2
    exit 1
  fi
  echo "claim-push: merge-tree clean"
fi

# Registered-root check: ROOT must map to this exact branch in envelopes#roots
# (or a plane whose branch equals BRANCH). Key membership alone is insufficient
# (e.g. root "grok" must not invent integ/grok when roots.grok.branch is integ/ci).
if ! python3 - "$ENVELOPES" "$ROOT" "$BRANCH" <<'PY'
import json, sys
path, root, branch = sys.argv[1:4]
with open(path) as f:
    e = json.load(f)
roots = e.get("roots", {})
planes = e.get("planes", {})
if root in roots:
    expected = roots[root].get("branch")
    if expected != branch:
        print(
            f"claim-push: REFUSE — roots[{root!r}].branch={expected!r} != {branch!r}",
            file=sys.stderr,
        )
        sys.exit(1)
    print(f"claim-push: envelope root ok for {branch}")
    sys.exit(0)
if any(isinstance(p, dict) and p.get("branch") == branch for p in planes.values()):
    print(f"claim-push: envelope plane ok for {branch}")
    sys.exit(0)
print(f"claim-push: REFUSE — {branch} not in envelopes#roots/#planes", file=sys.stderr)
sys.exit(1)
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
