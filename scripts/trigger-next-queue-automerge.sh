#!/usr/bin/env bash
set -euo pipefail

base_branch="dev"
base_ref=""
start_pr="1"
after_pr="0"
limit="200"
merge_method="squash"
fetch_remote="${GITHUB_FETCH_REMOTE:-}"
required_review_check="oya-pr-review"
dry_run="0"
require_verified_head="1"
required_contexts_config=""

usage() {
  cat <<'USAGE'
Usage: scripts/trigger-next-queue-automerge.sh [options]

Options:
  --base-branch <branch>       GitHub PR base branch to inspect (default: dev)
  --base-ref <ref>             Local git ref used as the starting merge base
                               (default: origin/<base-branch>)
  --start-pr <number>          First PR number in the active numeric queue
                               (default: 1; no PR is excluded by default)
  --after-pr <number>          Merged PR number that triggered this tick
                               (default: 0; select queue floor)
  --limit <number>             Maximum open PRs to query from GitHub (default: 200)
  --merge-method <method>      squash only in P0.0 (default: squash)
  --fetch-remote <remote>      Git remote used by the sequential conflict guard
                               to fetch refs/pull/<N>/head. Defaults to
                               GITHUB_FETCH_REMOTE, otherwise origin; when origin
                               is not GitHub and github-mirror is a GitHub
                               remote, github-mirror is selected automatically.
  --required-review-check <id> Required review check name (default: oya-pr-review)
  --required-contexts-config <path>
                              Canonical branch-protection contexts JSON
                              (default: infra/branch-protection/<base-branch>.json)
  --allow-unverified-head      Do not require GitHub-verified signed head commit
  --dry-run                    Print the selected PR and checks without enabling auto-merge

This script advances only the bottom-most open PR in the active queue. It never
force-pushes, never writes to the base branch, never skips a lower open queue PR,
and only enables GitHub auto-merge after live branch-protection required
contexts match the canonical repo policy, the review check has passed, the head
commit is verified, and the selected PR is conflict-clean against the current
base.
USAGE
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --base-branch)
      base_branch="${2:?missing --base-branch value}"
      shift 2
      ;;
    --base-ref)
      base_ref="${2:?missing --base-ref value}"
      shift 2
      ;;
    --start-pr)
      start_pr="${2:?missing --start-pr value}"
      shift 2
      ;;
    --after-pr)
      after_pr="${2:?missing --after-pr value}"
      shift 2
      ;;
    --limit)
      limit="${2:?missing --limit value}"
      shift 2
      ;;
    --merge-method)
      merge_method="${2:?missing --merge-method value}"
      shift 2
      ;;
    --fetch-remote)
      fetch_remote="${2:?missing --fetch-remote value}"
      shift 2
      ;;
    --required-review-check)
      required_review_check="${2:?missing --required-review-check value}"
      shift 2
      ;;
    --required-contexts-config)
      required_contexts_config="${2:?missing --required-contexts-config value}"
      shift 2
      ;;
    --allow-unverified-head)
      require_verified_head="0"
      shift
      ;;
    --dry-run)
      dry_run="1"
      shift
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

for numeric in start_pr after_pr limit; do
  value="${!numeric}"
  case "$value" in
    ''|*[!0-9]*)
      echo "--${numeric//_/-} must be a non-negative integer" >&2
      exit 2
      ;;
  esac
done

case "$merge_method" in
  squash) ;;
  *)
    echo "--merge-method is fixed to squash for P0.0 GitHub auto-merge scheduling" >&2
    exit 2
    ;;
esac

if ! command -v gh >/dev/null 2>&1; then
  echo "gh is required" >&2
  exit 2
fi

remote_url_contains_github() {
  local remote="$1"
  local url
  url="$(git remote get-url "$remote" 2>/dev/null || true)"
  [ -n "$url" ] && printf '%s\n' "$url" | grep -Eiq '(^|[:/@])github\.com[:/]'
}

if [ -z "$fetch_remote" ]; then
  fetch_remote="origin"
  if ! remote_url_contains_github "$fetch_remote" && remote_url_contains_github "github-mirror"; then
    fetch_remote="github-mirror"
  fi
fi

if [ -z "$fetch_remote" ]; then
  echo "--fetch-remote must not be empty" >&2
  exit 2
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required" >&2
  exit 2
fi

check_verified_head() {
  local head_oid="$1"
  local repo verification_json verified reason

  repo="$(gh repo view --json nameWithOwner --jq '.nameWithOwner')"
  verification_json="$(gh api "repos/${repo}/commits/${head_oid}" --jq '.commit.verification')"
  verified="$(printf '%s' "$verification_json" | jq -r '.verified // false')"
  reason="$(printf '%s' "$verification_json" | jq -r '.reason // ""')"

  if [ "$verified" != "true" ] || [ "$reason" != "valid" ]; then
    echo "PR head ${head_oid:0:8} is not GitHub-verified (verified=${verified} reason=${reason:-unknown}); not enabling auto-merge"
    return 1
  fi

  echo "PR head ${head_oid:0:8} signature verified by GitHub"
}

check_live_required_contexts() {
  local scratch_dir="$1"
  local repo live_file live_err canonical_file
  local missing extra live_count canonical_count

  if [ -z "$required_contexts_config" ]; then
    required_contexts_config="infra/branch-protection/${base_branch}.json"
  fi

  canonical_file="$required_contexts_config"
  if [ ! -f "$canonical_file" ]; then
    echo "::error::canonical required-context config not found: ${canonical_file}; refusing auto-merge" >&2
    exit 1
  fi

  if ! jq -e '.required_status_checks.contexts | type == "array" and length > 0' "$canonical_file" >/dev/null; then
    echo "::error::canonical required-context config has no required_status_checks.contexts: ${canonical_file}; refusing auto-merge" >&2
    exit 1
  fi

  repo="$(gh repo view --json nameWithOwner --jq '.nameWithOwner')"
  live_file="$scratch_dir/live-required-status-checks.json"
  live_err="$scratch_dir/live-required-status-checks.err"

  set +e
  gh api "repos/${repo}/branches/${base_branch}/protection/required_status_checks" > "$live_file" 2> "$live_err"
  live_status=$?
  set -e

  if [ "$live_status" -ne 0 ]; then
    echo "::error::cannot read live branch-protection required contexts for ${repo}:${base_branch}; refusing auto-merge. Provide GH_TOKEN/gh auth with Administration read permission for branch protection, then rerun. gh api exit=${live_status}" >&2
    if [ -s "$live_err" ]; then
      sed 's/^/gh api: /' "$live_err" >&2
    fi
    exit 1
  fi

  if ! jq -e '.contexts | type == "array" and length > 0' "$live_file" >/dev/null; then
    echo "::error::live branch-protection response for ${repo}:${base_branch} has no contexts array; refusing auto-merge" >&2
    exit 1
  fi

  missing="$(jq -n --slurpfile canonical "$canonical_file" --slurpfile live "$live_file" '
    (($canonical[0].required_status_checks.contexts - $live[0].contexts) | sort)
  ')"
  extra="$(jq -n --slurpfile canonical "$canonical_file" --slurpfile live "$live_file" '
    (($live[0].contexts - $canonical[0].required_status_checks.contexts) | sort)
  ')"
  canonical_count="$(jq -r '.required_status_checks.contexts | length' "$canonical_file")"
  live_count="$(jq -r '.contexts | length' "$live_file")"

  if [ "$missing" != "[]" ] || [ "$extra" != "[]" ]; then
    echo "::error::live branch-protection required contexts drift from ${canonical_file}; refusing auto-merge so next-queue cannot inherit weakened or stale protection" >&2
    echo "canonical_context_count=${canonical_count} live_context_count=${live_count}" >&2
    echo "missing_from_live=$(printf '%s' "$missing" | jq -cr '.')" >&2
    echo "extra_in_live=$(printf '%s' "$extra" | jq -cr '.')" >&2
    exit 1
  fi

  echo "live branch-protection required contexts match ${canonical_file} (${live_count} contexts)"
}

if [ -z "$base_ref" ]; then
  base_ref="origin/${base_branch}"
fi

if ! git rev-parse --verify --quiet "$base_ref^{commit}" >/dev/null; then
  echo "base ref is not a commit: $base_ref" >&2
  exit 2
fi

if [ "$after_pr" -gt 0 ] && [ "$after_pr" -lt "$start_pr" ]; then
  echo "merged PR #${after_pr} is below active queue start #${start_pr}; no queue tick"
  exit 0
fi

queue_floor="$start_pr"
if [ "$after_pr" -ge "$start_pr" ]; then
  queue_floor=$((after_pr + 1))
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

check_live_required_contexts "$tmp_dir"

prs_file="$tmp_dir/prs.json"
gh pr list \
  --state open \
  --base "$base_branch" \
  --limit "$limit" \
  --json number,headRefName,headRefOid,isDraft,title \
  > "$prs_file"

if [ "$after_pr" -gt 0 ]; then
  blocked_lower="$(jq -r --argjson start "$start_pr" --argjson after "$after_pr" '
    sort_by(.number)
    | .[]
    | select(.number >= $start and .number <= $after)
    | "#\(.number)"
  ' "$prs_file" | paste -sd "," -)"
  if [ -n "$blocked_lower" ]; then
    echo "::error::open lower queue PR(s) remain before trigger PR #${after_pr}: ${blocked_lower}" >&2
    exit 1
  fi
fi

candidate_json="$(jq -c --argjson floor "$queue_floor" '
  sort_by(.number)
  | map(select(.number >= $floor))
  | .[0] // empty
' "$prs_file")"

if [ -z "$candidate_json" ]; then
  echo "no open PR remains at or after queue floor #${queue_floor}"
  exit 0
fi

number="$(printf '%s' "$candidate_json" | jq -r '.number')"
head_ref_name="$(printf '%s' "$candidate_json" | jq -r '.headRefName')"
head_oid="$(printf '%s' "$candidate_json" | jq -r '.headRefOid')"
is_draft="$(printf '%s' "$candidate_json" | jq -r '.isDraft // false')"
title="$(printf '%s' "$candidate_json" | jq -r '.title // ""')"

echo "queue candidate: PR #${number} ${head_ref_name} (${head_oid:0:8}) draft=${is_draft} ${title}"

if [ "$is_draft" = "true" ]; then
  echo "bottom-most queue PR #${number} is draft; not enabling auto-merge and not skipping ahead"
  exit 0
fi

pr_state="$(gh pr view "$number" --json isDraft,mergeable,mergeStateStatus,reviewDecision,headRefOid)"
current_head="$(printf '%s' "$pr_state" | jq -r '.headRefOid')"
merge_state="$(printf '%s' "$pr_state" | jq -r '.mergeStateStatus // ""')"
mergeable="$(printf '%s' "$pr_state" | jq -r '.mergeable // ""')"
review_decision="$(printf '%s' "$pr_state" | jq -r '.reviewDecision // ""')"

if [ "$current_head" != "$head_oid" ]; then
  echo "::error::PR #${number} head changed while selecting (${head_oid} -> ${current_head})" >&2
  exit 1
fi

if [ "$review_decision" = "CHANGES_REQUESTED" ]; then
  echo "PR #${number} has CHANGES_REQUESTED; review fix-loop owns the next update"
  exit 0
fi

if [ "$require_verified_head" = "1" ]; then
  if ! check_verified_head "$head_oid"; then
    exit 0
  fi
fi

checks_json="$tmp_dir/checks.json"
checks_err="$tmp_dir/checks.err"
set +e
gh pr checks "$number" --json name,bucket,state,workflow > "$checks_json" 2> "$checks_err"
checks_status=$?
set -e
if [ "$checks_status" -ne 0 ] && [ "$checks_status" -ne 8 ]; then
  if grep -qi "no checks reported" "$checks_err"; then
    echo "no checks reported for PR #${number}; not enabling auto-merge"
    exit 0
  fi
  cat "$checks_err" >&2
  exit "$checks_status"
fi

if [ ! -s "$checks_json" ]; then
  echo "no check data returned for PR #${number}; not enabling auto-merge"
  exit 0
fi

review_bucket="$(jq -r --arg check "$required_review_check" '
  [.[] | select(.name == $check)] | .[0].bucket // ""
' "$checks_json")"

if [ -n "$required_review_check" ] && [ "$review_bucket" != "pass" ]; then
  echo "required review check ${required_review_check} is not passing for PR #${number} (bucket=${review_bucket:-missing}); not enabling auto-merge"
  exit 0
fi

if [ "$merge_state" = "DIRTY" ] || [ "$mergeable" = "CONFLICTING" ]; then
  echo "::error::PR #${number} is currently conflicting according to GitHub (mergeable=${mergeable} state=${merge_state})" >&2
  exit 1
fi

scripts/check-sequential-pr-merge-conflicts.sh \
  --base-branch "$base_branch" \
  --base-ref "$base_ref" \
  --start-pr "$number" \
  --end-pr "$number" \
  --limit "$limit" \
  --fetch-remote "$fetch_remote"

merge_flag="--squash"
if [ "$dry_run" = "1" ]; then
  echo "dry-run: gh pr merge ${number} ${merge_flag} --auto --match-head-commit ${head_oid}"
  exit 0
fi

gh pr merge "$number" "$merge_flag" --auto --match-head-commit "$head_oid"
echo "auto-merge enabled for bottom-most queue PR #${number}"
