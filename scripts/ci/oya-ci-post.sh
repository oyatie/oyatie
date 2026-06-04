#!/usr/bin/env bash
# Local bridge that posts the single P0.0 target status context after Buck2
# authority checks. This is local/bridge evidence only; destination authority is
# cloud-ci/oya-ci oya-ci-required from trusted controller state.
set -euo pipefail

CONTEXT="oya-ci-required"
BASE_REF="${OYA_CI_BASE_REF:-origin/dev}"
DRY_RUN=0
PR_NUMBER=""
HEAD_SHA=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --pr)
      PR_NUMBER="$2"
      shift 2
      ;;
    --sha)
      HEAD_SHA="$2"
      shift 2
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    -h|--help)
      sed -n '1,32p' "$0"
      exit 0
      ;;
    *)
      echo "unknown arg: $1" >&2
      exit 64
      ;;
  esac
done

REPO_OWNER_REPO="$(gh repo view --json nameWithOwner --jq .nameWithOwner)"
echo "[oya-ci-post] repo: ${REPO_OWNER_REPO}"

if [[ -z "${HEAD_SHA}" ]]; then
  if [[ -z "${PR_NUMBER}" ]]; then
    PR_NUMBER="$(gh pr view --json number --jq .number 2>/dev/null || true)"
    if [[ -z "${PR_NUMBER}" ]]; then
      echo "[oya-ci-post] no PR detected for current branch; pass --pr <N>" >&2
      exit 65
    fi
  fi
  HEAD_SHA="$(gh pr view "${PR_NUMBER}" --json headRefOid --jq .headRefOid)"
fi
echo "[oya-ci-post] PR #${PR_NUMBER:-?} head_sha: ${HEAD_SHA}"

post_status() {
  local state="$1"
  local description="$2"
  if [[ ${DRY_RUN} -eq 1 ]]; then
    echo "[dry-run] ${state} ${CONTEXT}: ${description}"
    return 0
  fi
  jq -n \
    --arg state "${state}" \
    --arg context "${CONTEXT}" \
    --arg description "${description}" \
    '{state: $state, context: $context, description: $description}' \
    | gh api -X POST "repos/${REPO_OWNER_REPO}/statuses/${HEAD_SHA}" --input - >/dev/null
  echo "[posted] ${state}: ${CONTEXT} — ${description}"
}

LOG="${TMPDIR:-/tmp}/oya-ci-post-buck2.log"
post_status pending "Buck2 authority + affected build/test running"
set +e
{
  python3 scripts/ci/enforce-buck2-authority.py --policy specs/buck2-authority-policy.json
  infra/ci/buck2-affected-gate.sh "${BASE_REF}" HEAD
} >"${LOG}" 2>&1
VERIFY_EXIT=$?
set -e
tail -80 "${LOG}"

if [[ ${VERIFY_EXIT} -eq 0 ]]; then
  post_status success "Buck2 authority + affected build/test passed"
  exit 0
fi

post_status failure "Buck2 authority or affected build/test failed"
exit 1
