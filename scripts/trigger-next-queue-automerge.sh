#!/usr/bin/env bash
set -euo pipefail

base_branch="dev"
base_ref=""
start_pr="1"
after_pr="0"
limit="200"
merge_method="squash"
required_review_check="oya-pr-review"
dry_run="0"
require_verified_head="1"

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
  --merge-method <method>      squash, merge, or rebase (default: squash)
  --required-review-check <id> Required review check name (default: oya-pr-review)
  --allow-unverified-head      Do not require GitHub-verified signed head commit
  --dry-run                    Print the selected PR and checks without enabling auto-merge

This script advances only the bottom-most open PR in the active queue. It never
force-pushes, never writes to the base branch, never skips a lower open queue PR,
and only enables GitHub auto-merge after the review check has passed, the head
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
    --required-review-check)
      required_review_check="${2:?missing --required-review-check value}"
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
  squash|merge|rebase) ;;
  *)
    echo "--merge-method must be one of: squash, merge, rebase" >&2
    exit 2
    ;;
esac

if ! command -v gh >/dev/null 2>&1; then
  echo "gh is required" >&2
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
  --limit "$limit"

merge_flag="--${merge_method}"
if [ "$dry_run" = "1" ]; then
  echo "dry-run: gh pr merge ${number} ${merge_flag} --auto --match-head-commit ${head_oid}"
  exit 0
fi

gh pr merge "$number" "$merge_flag" --auto --match-head-commit "$head_oid"
echo "auto-merge enabled for bottom-most queue PR #${number}"
