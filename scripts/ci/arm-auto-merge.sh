#!/usr/bin/env bash
# arm-auto-merge.sh — arm Forgejo auto-merge by enforcing the cloud-ci/oya-ci
# required context via branch protection on `dev`, idempotently
# (create-or-update), and optionally schedule per-PR auto-merge.
#
# Once this runs, a PR author/maintainer can enable "Auto Merge (when checks
# pass)" on a PR and Forgejo will merge it automatically the moment the
# branch-protection-required status check (`oya-ci-required`) turns green.
#
# WHAT THIS DOES
#   1. GET  the existing `dev` branch-protection rule.
#   2. If absent -> POST a new rule; if present -> PATCH it in place.
#      The rule sets:
#        enable_status_check    = true
#        status_check_contexts  = ["oya-ci-required"]   (context oya-ci posts)
#   3. Optionally POST /pulls/{index}/merge with
#        merge_when_checks_succeed = true
#        delete_branch_after_merge = true
#        head_commit_id            = <expected PR head SHA>
#   4. Prints the resulting effective rule (status-check fields only).
#
# This script is IDEMPOTENT: re-running it converges the `dev` rule to the
# desired state whether or not a rule already exists.
#
# AUTH
#   FORGEJO_TOKEN  (required except --dry-run, read from env, NEVER echoed) —
#                  a Forgejo access token with repo administration scope. Sent as:
#                      Authorization: token ${FORGEJO_TOKEN}
#
# CONFIG (overridable via env for testing)
#   FORGEJO_BASE_URL   default http://forgejo.oya-forge.svc.cluster.local:3000
#   FORGEJO_OWNER      default oya-admin
#   FORGEJO_REPO       default oyatie
#   PROTECTED_BRANCH   default dev
#   REQUIRED_CONTEXT   fixed oya-ci-required (env override fails closed)
#
# USAGE
#   FORGEJO_TOKEN=*** ./scripts/ci/arm-auto-merge.sh
#   ./scripts/ci/arm-auto-merge.sh --dry-run
#   FORGEJO_TOKEN=*** ./scripts/ci/arm-auto-merge.sh \
#     --pr-index 123 --head-commit <sha>
#
# NOTE: This is Forgejo bridge automation only. Protected-branch merge and
# Phase-0 exit authority remain the cloud-ci/oya-ci `oya-ci-required` required
# context, not local `oya verify`, `oya gate`, Cargo, or this script's stdout.

set -euo pipefail

FORGEJO_BASE_URL="${FORGEJO_BASE_URL:-http://forgejo.oya-forge.svc.cluster.local:3000}"
FORGEJO_OWNER="${FORGEJO_OWNER:-oya-admin}"
FORGEJO_REPO="${FORGEJO_REPO:-oyatie}"
PROTECTED_BRANCH="${PROTECTED_BRANCH:-dev}"
if [[ -n "${REQUIRED_CONTEXT:-}" && "${REQUIRED_CONTEXT}" != "oya-ci-required" ]]; then
  echo "REQUIRED_CONTEXT is fixed to oya-ci-required for P0.0 auto-merge authority; refusing ${REQUIRED_CONTEXT}" >&2
  exit 64
fi
REQUIRED_CONTEXT="oya-ci-required"

DRY_RUN=0
PR_INDEX=""
HEAD_COMMIT=""
MERGE_METHOD="squash"
DELETE_BRANCH_AFTER_MERGE="true"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) DRY_RUN=1; shift ;;
    --pr-index)
      PR_INDEX="${2:-}"
      shift 2
      ;;
    --head-commit)
      HEAD_COMMIT="${2:-}"
      shift 2
      ;;
    --merge-method)
      MERGE_METHOD="${2:-}"
      shift 2
      ;;
    --delete-branch-after-merge)
      DELETE_BRANCH_AFTER_MERGE="${2:-}"
      shift 2
      ;;
    -h|--help)
      sed -n '2,60p' "$0"
      cat <<'USAGE'

Options:
  --dry-run                         Print target payloads without requiring a token.
  --pr-index <number>               Also schedule that Forgejo PR for auto-merge.
  --head-commit <sha>               Required with --pr-index; stale-head guard.
  --merge-method <method>           squash only in P0.0 (default: squash).
  --delete-branch-after-merge bool  true only in P0.0 (default: true).
USAGE
      exit 0
      ;;
    *)
      echo "unknown arg: $1" >&2
      exit 64
      ;;
  esac
done

case "${PR_INDEX}" in
  ""|*[!0-9]*)
    if [[ -n "${PR_INDEX}" ]]; then
      echo "--pr-index must be a positive integer" >&2
      exit 64
    fi
    ;;
esac

case "${MERGE_METHOD}" in
  squash) ;;
  *)
    echo "--merge-method is fixed to squash for P0.0 Forgejo auto-merge scheduling" >&2
    exit 64
    ;;
esac

case "${DELETE_BRANCH_AFTER_MERGE}" in
  true) ;;
  *)
    echo "--delete-branch-after-merge is fixed to true for P0.0 auto-merge branch cleanup" >&2
    exit 64
    ;;
esac

if [[ -n "${PR_INDEX}" ]]; then
  if [[ -z "${HEAD_COMMIT}" ]]; then
    echo "--head-commit is required with --pr-index so Forgejo cannot arm auto-merge for a moved PR head." >&2
    exit 64
  fi
  case "${HEAD_COMMIT}" in
    *[!0-9a-fA-F]*|"")
      echo "--head-commit must be a hex commit id" >&2
      exit 64
      ;;
  esac
  if (( ${#HEAD_COMMIT} != 40 && ${#HEAD_COMMIT} != 64 )); then
    echo "--head-commit must be a full SHA-1 (40 hex) or SHA-256 (64 hex) commit id" >&2
    exit 64
  fi
fi

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "required command not found: $1" >&2
    exit 127
  fi
}
require_cmd curl
require_cmd python3

if [[ ${DRY_RUN} -ne 1 && -z "${FORGEJO_TOKEN:-}" ]]; then
  echo "FORGEJO_TOKEN is required (Forgejo access token with repo admin scope)." >&2
  echo "It is read from the environment and is never echoed." >&2
  exit 1
fi

API="${FORGEJO_BASE_URL%/}/api/v1"
BP_COLLECTION="${API}/repos/${FORGEJO_OWNER}/${FORGEJO_REPO}/branch_protections"
BP_ITEM="${BP_COLLECTION}/${PROTECTED_BRANCH}"
PR_ITEM_ENDPOINT="${API}/repos/${FORGEJO_OWNER}/${FORGEJO_REPO}/pulls/${PR_INDEX}"
PR_MERGE_ENDPOINT="${API}/repos/${FORGEJO_OWNER}/${FORGEJO_REPO}/pulls/${PR_INDEX}/merge"

# Curl wrapper. The token lives only in this function's argv to `curl`;
# it is never printed. Emits the HTTP status code on the last line so the
# caller can branch on it without leaking the body into logs uncontrolled.
forge_api() {
  local method="$1"; shift
  local url="$1"; shift
  curl -sS \
    -X "${method}" \
    -H "Authorization: token ${FORGEJO_TOKEN}" \
    -H "Content-Type: application/json" \
    -H "Accept: application/json" \
    -w $'\n%{http_code}' \
    "$@" \
    "${url}"
}

http_code_of() { tail -n1 <<<"$1"; }
body_of()      { sed '$d'   <<<"$1"; }

echo "[arm-auto-merge] forge: ${FORGEJO_BASE_URL}"
echo "[arm-auto-merge] repo:  ${FORGEJO_OWNER}/${FORGEJO_REPO}"
echo "[arm-auto-merge] branch: ${PROTECTED_BRANCH}"
echo "[arm-auto-merge] required status check context: ${REQUIRED_CONTEXT}"

# Desired branch-protection payload: enforce the single gate context.
# (branch_name is only required for the create/POST shape; PATCH ignores it.)
desired_payload() {
  # Build the JSON with python3 (stdlib only). The branch name and required
  # context are passed via EXPORTED env vars and read with os.environ — they are
  # never interpolated into the python source string.
  PROTECTED_BRANCH="${PROTECTED_BRANCH}" \
  REQUIRED_CONTEXT="${REQUIRED_CONTEXT}" \
  python3 -c '
import json, os
print(json.dumps({
    "branch_name": os.environ["PROTECTED_BRANCH"],
    "enable_status_check": True,
    "status_check_contexts": [os.environ["REQUIRED_CONTEXT"]],
}))
'
}

auto_merge_payload() {
  MERGE_METHOD="${MERGE_METHOD}" \
  HEAD_COMMIT="${HEAD_COMMIT}" \
  DELETE_BRANCH_AFTER_MERGE="${DELETE_BRANCH_AFTER_MERGE}" \
  python3 -c '
import json, os
print(json.dumps({
    "Do": os.environ["MERGE_METHOD"],
    "merge_when_checks_succeed": True,
    "delete_branch_after_merge": os.environ["DELETE_BRANCH_AFTER_MERGE"] == "true",
    "head_commit_id": os.environ["HEAD_COMMIT"],
}))
'
}

validate_pr_ready_for_auto_merge() {
  local pr_resp pr_code pr_body parsed actual_head mergeable

  echo "[arm-auto-merge] GET ${PR_ITEM_ENDPOINT}"
  pr_resp="$(forge_api GET "${PR_ITEM_ENDPOINT}")"
  pr_code="$(http_code_of "${pr_resp}")"
  if [[ "${pr_code}" != "200" ]]; then
    echo "[arm-auto-merge] PR refresh failed (HTTP ${pr_code})" >&2
    body_of "${pr_resp}" >&2
    exit 1
  fi

  pr_body="$(body_of "${pr_resp}")"
  parsed="$(python3 -c '
import json, sys
d = json.load(sys.stdin)
head = d.get("head") or {}
head_sha = head.get("sha") or ""
mergeable = d.get("mergeable", None)
if mergeable is True:
    mergeable_text = "true"
elif mergeable is False:
    mergeable_text = "false"
else:
    mergeable_text = "null"
print(f"{head_sha}\t{mergeable_text}")
' <<<"${pr_body}")"
  IFS=$'\t' read -r actual_head mergeable <<<"${parsed}"

  if [[ -z "${actual_head}" ]]; then
    echo "[arm-auto-merge] PR refresh did not include head.sha; refusing to schedule auto-merge" >&2
    exit 1
  fi
  if [[ "${actual_head}" != "${HEAD_COMMIT}" ]]; then
    echo "[arm-auto-merge] current PR head ${actual_head} does not match expected ${HEAD_COMMIT}; refusing stale auto-merge" >&2
    exit 1
  fi
  if [[ "${mergeable}" != "true" ]]; then
    echo "[arm-auto-merge] PR is not mergeable according to Forgejo (mergeable=${mergeable}); refusing auto-merge scheduling" >&2
    exit 1
  fi

  echo "[arm-auto-merge] PR #${PR_INDEX} head and mergeability guard passed"
}

if [[ ${DRY_RUN} -eq 1 ]]; then
  echo "[arm-auto-merge] --dry-run: would GET ${BP_ITEM}"
  echo "[arm-auto-merge] --dry-run: if 404 -> POST ${BP_COLLECTION}; else PATCH ${BP_ITEM}"
  echo "[arm-auto-merge] --dry-run: desired payload:"
  desired_payload
  if [[ -n "${PR_INDEX}" ]]; then
    echo "[arm-auto-merge] --dry-run: would POST ${PR_MERGE_ENDPOINT}"
    echo "[arm-auto-merge] --dry-run: PR auto-merge payload:"
    auto_merge_payload
  fi
  exit 0
fi

# 1) GET first (create-or-update probe).
echo "[arm-auto-merge] GET ${BP_ITEM}"
get_resp="$(forge_api GET "${BP_ITEM}")"
get_code="$(http_code_of "${get_resp}")"

if [[ "${get_code}" == "200" ]]; then
  # 2a) Rule exists -> PATCH it in place.
  echo "[arm-auto-merge] existing rule found (HTTP 200); PATCH ${BP_ITEM}"
  patch_resp="$(forge_api PATCH "${BP_ITEM}" --data "$(desired_payload)")"
  patch_code="$(http_code_of "${patch_resp}")"
  if [[ "${patch_code}" != "200" ]]; then
    echo "[arm-auto-merge] PATCH failed (HTTP ${patch_code})" >&2
    body_of "${patch_resp}" >&2
    exit 1
  fi
  result_body="$(body_of "${patch_resp}")"
elif [[ "${get_code}" == "404" ]]; then
  # 2b) No rule -> POST a new one.
  echo "[arm-auto-merge] no rule for ${PROTECTED_BRANCH} (HTTP 404); POST ${BP_COLLECTION}"
  post_resp="$(forge_api POST "${BP_COLLECTION}" --data "$(desired_payload)")"
  post_code="$(http_code_of "${post_resp}")"
  if [[ "${post_code}" != "201" && "${post_code}" != "200" ]]; then
    echo "[arm-auto-merge] POST failed (HTTP ${post_code})" >&2
    body_of "${post_resp}" >&2
    exit 1
  fi
  result_body="$(body_of "${post_resp}")"
else
  echo "[arm-auto-merge] unexpected GET status (HTTP ${get_code})" >&2
  body_of "${get_resp}" >&2
  exit 1
fi

# 3) Print the effective status-check fields (no token, no secrets).
echo "[arm-auto-merge] effective rule (status-check fields):"
python3 -c '
import json, sys
d = json.load(sys.stdin)
print(json.dumps({
    "branch_name": d.get("branch_name"),
    "enable_status_check": d.get("enable_status_check"),
    "status_check_contexts": d.get("status_check_contexts"),
}, indent=2))
' <<<"${result_body}"

echo "[arm-auto-merge] gate armed: '${REQUIRED_CONTEXT}' is now a required status check on '${PROTECTED_BRANCH}'."
echo "[arm-auto-merge] PRs can now enable Auto Merge (when checks pass); see docs/ci/auto-merge-flow.md"

if [[ -n "${PR_INDEX}" ]]; then
  echo "[arm-auto-merge] scheduling PR #${PR_INDEX} for Forgejo auto-merge after '${REQUIRED_CONTEXT}' succeeds"
  validate_pr_ready_for_auto_merge
  merge_resp="$(forge_api POST "${PR_MERGE_ENDPOINT}" --data "$(auto_merge_payload)")"
  merge_code="$(http_code_of "${merge_resp}")"
  if [[ "${merge_code}" != "200" && "${merge_code}" != "202" && "${merge_code}" != "204" ]]; then
    echo "[arm-auto-merge] PR auto-merge scheduling failed (HTTP ${merge_code})" >&2
    body_of "${merge_resp}" >&2
    exit 1
  fi
  echo "[arm-auto-merge] auto-merge scheduled for PR #${PR_INDEX} at expected head ${HEAD_COMMIT}"
fi
