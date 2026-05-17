#!/usr/bin/env bash
set -euo pipefail

base_branch="dev"
base_ref=""
start_pr="111"
target_pr=""
limit="200"
apply="0"

usage() {
  cat <<'USAGE'
Usage: scripts/repair-sequential-pr-queue.sh --target-pr <number> [options]

Options:
  --base-branch <branch>   GitHub PR base branch to inspect (default: dev)
  --base-ref <ref>         Local git ref used as the starting virtual base
                           (default: origin/<base-branch>)
  --start-pr <number>      First PR number in the active numeric queue
                           (default: 111; no PR is excluded by default)
  --target-pr <number>     Queue PR branch to refresh with all earlier open PR heads
  --limit <number>         Maximum open PRs to query from GitHub (default: 200)
  --apply                  Push the repaired target branch if all guards pass

Default mode is dry-run. Apply mode never force-pushes, refuses fork PRs, refuses
draft target PRs, stops on unresolved merge conflicts, and rechecks that the
remote target head did not move before pushing.
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
    --target-pr)
      target_pr="${2:?missing --target-pr value}"
      shift 2
      ;;
    --limit)
      limit="${2:?missing --limit value}"
      shift 2
      ;;
    --apply)
      apply="1"
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

for numeric in start_pr target_pr limit; do
  value="${!numeric}"
  case "$value" in
    ''|*[!0-9]*)
      echo "--${numeric//_/-} must be a positive integer" >&2
      exit 2
      ;;
  esac
done

if [ "$target_pr" -lt "$start_pr" ]; then
  echo "--target-pr must be greater than or equal to --start-pr" >&2
  exit 2
fi

if ! command -v gh >/dev/null 2>&1; then
  echo "gh is required" >&2
  exit 2
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required" >&2
  exit 2
fi

if [ -z "$base_ref" ]; then
  base_ref="origin/${base_branch}"
fi

if ! git rev-parse --verify --quiet "$base_ref^{commit}" >/dev/null; then
  echo "base ref is not a commit: $base_ref" >&2
  exit 2
fi

tmp_dir="$(mktemp -d)"
cleanup() {
  if [ -n "${repair_worktree:-}" ] && git worktree list --porcelain | grep -Fqx "worktree ${repair_worktree}"; then
    git worktree remove --force "$repair_worktree" >/dev/null 2>&1 || true
  fi
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

prs_file="$tmp_dir/prs.json"
ordered_file="$tmp_dir/ordered.tsv"

gh pr list \
  --state open \
  --base "$base_branch" \
  --limit "$limit" \
  --json number,headRefName,headRefOid,isDraft,isCrossRepository,title \
  > "$prs_file"

jq -r --argjson start "$start_pr" --argjson target "$target_pr" '
  sort_by(.number)
  | .[]
  | select(.number >= $start and .number <= $target)
  | [.number, .headRefName, .headRefOid, (.isDraft // false), (.isCrossRepository // false), (.title // "")]
  | @tsv
' "$prs_file" > "$ordered_file"

if ! awk -F'\t' -v target="$target_pr" '$1 == target { found=1 } END { exit found ? 0 : 1 }' "$ordered_file"; then
  echo "::error::target PR #${target_pr} is not open against ${base_branch}" >&2
  exit 1
fi

target_row="$(awk -F'\t' -v target="$target_pr" '$1 == target { print; exit }' "$ordered_file")"
IFS=$'\t' read -r _target_number target_head_ref target_head_oid target_is_draft target_is_cross target_title <<< "$target_row"

if [ "$target_is_draft" = "true" ]; then
  echo "target PR #${target_pr} is draft; refusing queue repair"
  exit 0
fi

if [ "$target_is_cross" = "true" ]; then
  echo "::error::target PR #${target_pr} is from a fork; refusing branch mutation" >&2
  exit 1
fi

prior_count="$(awk -F'\t' -v target="$target_pr" '$1 < target { count++ } END { print count + 0 }' "$ordered_file")"
if [ "$prior_count" -eq 0 ]; then
  echo "target PR #${target_pr} is already the queue floor; no earlier open PR heads to merge"
  exit 0
fi

while IFS=$'\t' read -r number _head_ref _head_oid _is_draft _is_cross _title; do
  git fetch --no-tags origin "+refs/pull/${number}/head:refs/remotes/pr/${number}" >/dev/null 2>&1
done < "$ordered_file"

repair_worktree="$tmp_dir/worktree"
git worktree add --detach "$repair_worktree" "refs/remotes/pr/${target_pr}" >/dev/null
git -C "$repair_worktree" config user.name "oyatie-queue-repair"
git -C "$repair_worktree" config user.email "queue-repair@users.noreply.github.com"

while IFS=$'\t' read -r number head_ref head_oid is_draft _is_cross title; do
  if [ "$number" -ge "$target_pr" ]; then
    break
  fi
  if [ "$is_draft" = "true" ]; then
    echo "prior PR #${number} is draft; not repairing past an unready lower queue entry"
    exit 0
  fi
  prior_ref="refs/remotes/pr/${number}"
  if git -C "$repair_worktree" merge-base --is-ancestor "$prior_ref" HEAD; then
    echo "PR #${target_pr} already contains PR #${number}: ${head_ref} (${head_oid:0:8})"
    continue
  fi
  echo "merging prior PR #${number} into PR #${target_pr}: ${head_ref} (${head_oid:0:8}) ${title}"
  if ! git -C "$repair_worktree" merge --no-ff "$prior_ref" \
    -m "Refresh PR ${target_pr} after queue PR ${number}" \
    -m "Constraint: PR #${number} precedes PR #${target_pr} in the active numeric merge queue.
Rejected: Waiting for manual conflict discovery | the queue repair lane can safely absorb non-conflicting lower heads.
Confidence: high
Scope-risk: narrow
Directive: Do not force-push; stop and route to fix-loop if this merge conflicts.
Tested: automated queue repair will run sequential simulation through PR #${target_pr} before pushing.
Not-tested: full repository test suite."; then
    echo "::error::queue repair conflict while merging PR #${number} into PR #${target_pr}" >&2
    git -C "$repair_worktree" diff --name-only --diff-filter=U >&2 || true
    exit 1
  fi
done < "$ordered_file"

new_head="$(git -C "$repair_worktree" rev-parse HEAD)"
repaired_prs="$tmp_dir/repaired-prs.json"
jq --argjson target "$target_pr" --arg new_head "$new_head" '
  map(if .number == $target then .headRefOid = $new_head else . end)
' "$prs_file" > "$repaired_prs"

scripts/check-sequential-pr-merge-conflicts.sh \
  --base-branch "$base_branch" \
  --base-ref "$base_ref" \
  --start-pr "$start_pr" \
  --end-pr "$target_pr" \
  --pr-json "$repaired_prs" \
  --no-fetch

if [ "$apply" != "1" ]; then
  echo "dry-run: would push ${new_head} to ${target_head_ref} for PR #${target_pr}"
  exit 0
fi

latest_head="$(gh pr view "$target_pr" --json headRefOid --jq '.headRefOid')"
if [ "$latest_head" != "$target_head_oid" ]; then
  echo "::error::target PR #${target_pr} moved while repairing (${target_head_oid} -> ${latest_head}); refusing push" >&2
  exit 1
fi

git -C "$repair_worktree" push origin "HEAD:refs/heads/${target_head_ref}"
echo "queue repair pushed for PR #${target_pr}: ${target_head_ref} ${target_head_oid:0:8} -> ${new_head:0:8}"
