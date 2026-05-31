#!/usr/bin/env bash
# arm-auto-merge.sh — arm Forgejo auto-merge by enforcing the CI gate via
# branch protection on `dev`, idempotently (create-or-update).
#
# Once this runs, a PR author/maintainer can enable "Auto Merge (when checks
# pass)" on a PR and Forgejo will merge it automatically the moment the
# branch-protection-required status check (`oya-ci-gate`) turns green.
#
# WHAT THIS DOES
#   1. GET  the existing `dev` branch-protection rule.
#   2. If absent -> POST a new rule; if present -> PATCH it in place.
#      The rule sets:
#        enable_status_check    = true
#        status_check_contexts  = ["oya-ci-gate"]   (context the controller posts)
#   3. Prints the resulting effective rule (status-check fields only).
#
# This script is IDEMPOTENT: re-running it converges the `dev` rule to the
# desired state whether or not a rule already exists.
#
# AUTH
#   FORGEJO_TOKEN  (required, read from env, NEVER echoed) — a Forgejo access
#                  token with repo administration scope. Sent as:
#                      Authorization: token ${FORGEJO_TOKEN}
#
# CONFIG (overridable via env for testing)
#   FORGEJO_BASE_URL   default http://forgejo.oya-forge.svc.cluster.local:3000
#   FORGEJO_OWNER      default oya-admin
#   FORGEJO_REPO       default oyatie
#   PROTECTED_BRANCH   default dev
#   REQUIRED_CONTEXT   default oya-ci-gate
#
# USAGE
#   FORGEJO_TOKEN=*** ./scripts/ci/arm-auto-merge.sh
#   FORGEJO_TOKEN=*** ./scripts/ci/arm-auto-merge.sh --dry-run
#
# NOTE: this script only arms the GATE (branch protection). Enabling auto-merge
# on an individual PR is a separate, per-PR call — see the "PER-PR AUTO-MERGE"
# section at the bottom of this file and docs/ci/auto-merge-flow.md.

set -euo pipefail

FORGEJO_BASE_URL="${FORGEJO_BASE_URL:-http://forgejo.oya-forge.svc.cluster.local:3000}"
FORGEJO_OWNER="${FORGEJO_OWNER:-oya-admin}"
FORGEJO_REPO="${FORGEJO_REPO:-oyatie}"
PROTECTED_BRANCH="${PROTECTED_BRANCH:-dev}"
REQUIRED_CONTEXT="${REQUIRED_CONTEXT:-oya-ci-gate}"

DRY_RUN=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) DRY_RUN=1; shift ;;
    -h|--help)
      sed -n '2,46p' "$0"
      exit 0
      ;;
    *)
      echo "unknown arg: $1" >&2
      exit 64
      ;;
  esac
done

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "required command not found: $1" >&2
    exit 127
  fi
}
require_cmd curl
require_cmd jq

if [[ -z "${FORGEJO_TOKEN:-}" ]]; then
  echo "FORGEJO_TOKEN is required (Forgejo access token with repo admin scope)." >&2
  echo "It is read from the environment and is never echoed." >&2
  exit 1
fi

API="${FORGEJO_BASE_URL%/}/api/v1"
BP_COLLECTION="${API}/repos/${FORGEJO_OWNER}/${FORGEJO_REPO}/branch_protections"
BP_ITEM="${BP_COLLECTION}/${PROTECTED_BRANCH}"

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
  jq -n \
    --arg branch "${PROTECTED_BRANCH}" \
    --arg ctx "${REQUIRED_CONTEXT}" \
    '{
       branch_name: $branch,
       enable_status_check: true,
       status_check_contexts: [ $ctx ]
     }'
}

if [[ ${DRY_RUN} -eq 1 ]]; then
  echo "[arm-auto-merge] --dry-run: would GET ${BP_ITEM}"
  echo "[arm-auto-merge] --dry-run: if 404 -> POST ${BP_COLLECTION}; else PATCH ${BP_ITEM}"
  echo "[arm-auto-merge] --dry-run: desired payload:"
  desired_payload
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
jq '{branch_name, enable_status_check, status_check_contexts}' <<<"${result_body}"

echo "[arm-auto-merge] gate armed: '${REQUIRED_CONTEXT}' is now a required status check on '${PROTECTED_BRANCH}'."
echo "[arm-auto-merge] PRs can now enable Auto Merge (when checks pass); see docs/ci/auto-merge-flow.md"

# ---------------------------------------------------------------------------
# PER-PR AUTO-MERGE (reference — NOT executed by this script)
# ---------------------------------------------------------------------------
# Forgejo's auto-merge ("Auto Merge — merge when all checks succeed") is enabled
# per PR by the author/maintainer. Because `enable_status_check=true` with
# `status_check_contexts=["oya-ci-gate"]` is now set on `dev`, the auto-merge is
# gated on that context turning green.
#
# Forgejo exposes this via the merge endpoint with a scheduling flag:
#
#   POST ${API}/repos/${FORGEJO_OWNER}/${FORGEJO_REPO}/pulls/{index}/merge
#   Authorization: token ${FORGEJO_TOKEN}
#   Content-Type: application/json
#   {
#     "Do": "squash",                 # merge style: merge|rebase|rebase-merge|squash|manually-merged
#     "merge_when_checks_succeed": true,
#     "delete_branch_after_merge": true
#   }
#
# When required checks are already green the PR merges immediately; otherwise
# Forgejo schedules the merge and completes it automatically once `oya-ci-gate`
# reports success. (Field name is `merge_when_checks_succeed` on Forgejo's
# MergePullRequestOption; older Gitea-derived builds used `MergeWhenChecksSucceed`.
# Confirm against your deployed Forgejo's /api/swagger before scripting it.)
#
# Example (uncomment + provide PR_INDEX to actually arm a specific PR):
#   # PR_INDEX=123
#   # curl -sS -X POST \
#   #   -H "Authorization: token ${FORGEJO_TOKEN}" \
#   #   -H "Content-Type: application/json" \
#   #   "${API}/repos/${FORGEJO_OWNER}/${FORGEJO_REPO}/pulls/${PR_INDEX}/merge" \
#   #   --data '{"Do":"squash","merge_when_checks_succeed":true,"delete_branch_after_merge":true}'
