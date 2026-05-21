#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/branch-protection-apply.sh [--check|--apply] [--repo OWNER/REPO] [--branch BRANCH] [--config PATH]

Synchronize GitHub required status-check protection with infra/branch-protection/dev.json,
including configured app_id overrides for required checks such as oya-pr-review.

Default mode is --check: read live branch protection, run the repo gate, and exit non-zero on drift.
--apply performs the GitHub mutation for required_status_checks only, then re-runs --check.

Auth: uses gh CLI auth or GH_TOKEN. GitHub requires Administration read for --check and
Administration write for --apply on branch-protection status-check endpoints.
USAGE
}

mode="check"
repo="${GITHUB_REPOSITORY:-jason931225/oyatie}"
branch="dev"
config="infra/branch-protection/dev.json"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --check)
      mode="check"
      shift
      ;;
    --apply)
      mode="apply"
      shift
      ;;
    --repo)
      repo="${2:-}"
      shift 2
      ;;
    --branch)
      branch="${2:-}"
      shift 2
      ;;
    --config)
      config="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "required command not found: $1" >&2
    exit 127
  fi
}

require_cmd gh
require_cmd jq
require_cmd cargo

if [[ -n "${GITHUB_ACTIONS:-}" && -z "${GH_TOKEN:-}" ]]; then
  echo "::error::OYA_BRANCH_PROTECTION_READ_TOKEN is required; GitHub branch-protection status-check APIs require Administration read permission, which GITHUB_TOKEN cannot request." >&2
  exit 1
fi

if [[ ! -f "$config" ]]; then
  echo "branch-protection config not found: $config" >&2
  exit 66
fi

config_branch="$(jq -r '.branch // empty' "$config")"
if [[ -n "$config_branch" && "$config_branch" != "$branch" ]]; then
  echo "config branch ($config_branch) does not match requested branch ($branch)" >&2
  exit 65
fi

workdir="$(mktemp -d)"
cleanup() {
  rm -rf "$workdir"
}
trap cleanup EXIT

payload="$workdir/required-status-checks-payload.json"
live="$workdir/live-required-status-checks.json"

jq '
  .required_status_checks as $required
  | ($required.app_id_overrides // {}) as $app_id_overrides
  | {
      strict: $required.strict,
      checks: (
        $required.contexts
        | map(
            {context: .}
            + (if $app_id_overrides[.] != null then {app_id: $app_id_overrides[.]} else {} end)
          )
      )
    }
' \
  "$config" > "$payload"

if [[ "$(jq -r '.checks | length' "$payload")" == "0" ]]; then
  echo "refusing empty required_status_checks.contexts from $config" >&2
  exit 65
fi

fetch_live() {
  gh api "repos/${repo}/branches/${branch}/protection/required_status_checks" > "$live"
}

print_delta() {
  jq -n \
    --slurpfile canonical_doc "$config" \
    --slurpfile live_doc "$live" \
    '($canonical_doc[0].required_status_checks.contexts) as $canonical |
     ($live_doc[0].contexts) as $live |
     {
       canonical: $canonical,
       live: $live,
       missing_from_live: (($canonical - $live) | sort),
       extra_in_live: (($live - $canonical) | sort),
       app_binding_drift: (
         ($canonical_doc[0].required_status_checks.app_id_overrides // {}) as $overrides
         | ($live_doc[0].checks // []) as $live_checks
         | [
             $overrides
             | to_entries[]
             | .key as $context
             | .value as $expected_app_id
             | ($live_checks | map(select(.context == $context)) | .[0]? // null) as $live_check
             | if $live_check == null then
                 {
                   context: $context,
                   expected_app_id: $expected_app_id,
                   live_app_id: "missing-check-binding"
                 }
               elif $expected_app_id == -1 and (($live_check.app_id // null) == null or $live_check.app_id == -1) then
                 empty
               elif $expected_app_id == ($live_check.app_id // null) then
                 empty
               else
                 {
                   context: $context,
                   expected_app_id: $expected_app_id,
                   live_app_id: ($live_check.app_id // null)
                 }
               end
           ]
       )
     }'
}

check_app_binding_drift() {
  local drift
  drift="$(print_delta | jq -c '.app_binding_drift')"
  if [[ "$drift" != "[]" ]]; then
    echo "required status-check app binding drift: $drift" >&2
    exit 1
  fi
}

if [[ "$mode" == "apply" ]]; then
  echo "Applying required status-check contexts for ${repo}:${branch} from ${config}" >&2
  gh api --method PATCH \
    "repos/${repo}/branches/${branch}/protection/required_status_checks" \
    --input "$payload" >/dev/null
fi

fetch_live
print_delta
check_app_binding_drift
cargo run -q -p oya-dev-cli -- gate validate protection-context-match \
  --branch "$branch" \
  --applied-branch-protection "$config" \
  --live-required-contexts "$live"
