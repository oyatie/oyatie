#!/usr/bin/env bash
set -euo pipefail

base_branch="dev"
base_ref=""
start_pr="1"
end_pr=""
skip_prs=""
limit="200"
pr_json=""
fetch_heads="1"

usage() {
  cat <<'USAGE'
Usage: scripts/check-sequential-pr-merge-conflicts.sh [options]

Options:
  --base-branch <branch>   GitHub PR base branch to inspect (default: dev)
  --base-ref <ref>         Local git ref used as the starting virtual base
                           (default: origin/<base-branch>)
  --start-pr <number>      First PR number in the numeric merge sequence
                           (default: 1; no PR is excluded by default)
  --end-pr <number>        Last PR number to include in the numeric merge
                           sequence (default: no upper bound)
  --skip-prs <csv>         Explicit one-off PR numbers to skip, e.g. "109,130"
                           (default: empty)
  --limit <number>         Maximum open PRs to query from GitHub (default: 200)
  --pr-json <path>         Read PR list JSON from a file instead of gh pr list
  --no-fetch               Do not fetch refs/pull/<N>/head before simulation

The script simulates open PRs in ascending PR-number order by repeatedly
running `git merge-tree --write-tree` and materializing a temporary virtual
merge commit with `git commit-tree`. It fails at the first conflict and prints
the conflict file list.
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
    --end-pr)
      end_pr="${2:?missing --end-pr value}"
      shift 2
      ;;
    --skip-prs)
      skip_prs="${2:-}"
      shift 2
      ;;
    --limit)
      limit="${2:?missing --limit value}"
      shift 2
      ;;
    --pr-json)
      pr_json="${2:?missing --pr-json value}"
      shift 2
      ;;
    --no-fetch)
      fetch_heads="0"
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

case "$start_pr" in
  ''|*[!0-9]*)
    echo "--start-pr must be a positive integer" >&2
    exit 2
    ;;
esac

if [ -n "$end_pr" ]; then
  case "$end_pr" in
    ''|*[!0-9]*)
      echo "--end-pr must be a positive integer" >&2
      exit 2
      ;;
  esac
  if [ "$end_pr" -lt "$start_pr" ]; then
    echo "--end-pr must be greater than or equal to --start-pr" >&2
    exit 2
  fi
fi

case "$limit" in
  ''|*[!0-9]*)
    echo "--limit must be a positive integer" >&2
    exit 2
    ;;
esac

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

export GIT_AUTHOR_NAME="${GIT_AUTHOR_NAME:-oyatie-queue-simulator}"
export GIT_AUTHOR_EMAIL="${GIT_AUTHOR_EMAIL:-queue-simulator@users.noreply.github.com}"
export GIT_COMMITTER_NAME="${GIT_COMMITTER_NAME:-$GIT_AUTHOR_NAME}"
export GIT_COMMITTER_EMAIL="${GIT_COMMITTER_EMAIL:-$GIT_AUTHOR_EMAIL}"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

prs_file="$tmp_dir/prs.json"
skip_file="$tmp_dir/skip.txt"
printf '%s\n' "$skip_prs" | tr ',' '\n' | awk 'NF {gsub(/^ +| +$/, ""); print}' > "$skip_file"

if [ -n "$pr_json" ]; then
  cp "$pr_json" "$prs_file"
else
  if ! command -v gh >/dev/null 2>&1; then
    echo "gh is required unless --pr-json is supplied" >&2
    exit 2
  fi
  gh pr list \
    --state open \
    --base "$base_branch" \
    --limit "$limit" \
    --json number,headRefName,headRefOid,isDraft,title \
    > "$prs_file"
fi

ordered_file="$tmp_dir/ordered.tsv"
jq -r --argjson start "$start_pr" --rawfile skip "$skip_file" '
  ($skip | split("\n") | map(select(length > 0) | tonumber)) as $skip_numbers
  | sort_by(.number)
  | .[]
  | select(.number >= $start)
  | select((.number as $n | $skip_numbers | index($n)) | not)
  | [.number, .headRefName, .headRefOid, (.isDraft // false), (.title // "")]
  | @tsv
' "$prs_file" > "$ordered_file"

if [ -n "$end_pr" ]; then
  awk -F'\t' -v end="$end_pr" '$1 <= end' "$ordered_file" > "$ordered_file.end"
  mv "$ordered_file.end" "$ordered_file"
fi

if [ ! -s "$ordered_file" ]; then
  echo "sequential PR merge simulation: no open PRs matched base=${base_branch} start_pr=${start_pr}"
  exit 0
fi

virtual_head="$(git rev-parse "$base_ref^{commit}")"
count=0

echo "sequential PR merge simulation"
echo "base_branch=${base_branch}"
echo "base_ref=${base_ref}"
echo "base_commit=${virtual_head}"
echo "start_pr=${start_pr}"
echo "end_pr=${end_pr:-<none>}"
echo "skip_prs=${skip_prs:-<none>}"

while IFS=$'\t' read -r number head_ref_name head_oid is_draft title; do
  count=$((count + 1))
  pr_ref="refs/remotes/pr/${number}"
  if [ "$fetch_heads" = "1" ]; then
    git fetch --no-tags origin "+refs/pull/${number}/head:${pr_ref}" >/dev/null 2>&1
    head_ref="$pr_ref"
    fetched_head="$(git rev-parse "${head_ref}^{commit}")"
    if [ "$fetched_head" != "$head_oid" ]; then
      echo "::error::PR #${number} moved while fetching (${head_oid} -> ${fetched_head}); refusing stale queue simulation" >&2
      exit 1
    fi
  else
    head_ref="$head_oid"
  fi

  if ! git rev-parse --verify --quiet "${head_ref}^{commit}" >/dev/null; then
    echo "::error::PR #${number} head is not available locally: ${head_ref}" >&2
    exit 1
  fi

  head_commit="$(git rev-parse "${head_ref}^{commit}")"
  echo "checking PR #${number}: ${head_ref_name} (${head_oid:0:8}) draft=${is_draft} ${title}"
  merge_output="$tmp_dir/merge-${number}.out"
  if ! git merge-tree --write-tree "$virtual_head" "$head_ref" > "$merge_output" 2>&1; then
    echo "::error::sequential merge conflict at PR #${number} (${head_ref_name})" >&2
    echo "conflict files:" >&2
    awk -F'\t' '/^[0-9]{6} [0-9a-f]+ [123]\t/ {print $2}' "$merge_output" | sort -u >&2
    echo "merge-tree output:" >&2
    cat "$merge_output" >&2
    exit 1
  fi

  tree_id="$(head -n 1 "$merge_output")"
  parents=(-p "$virtual_head")
  if [ "$head_commit" != "$virtual_head" ]; then
    parents+=(-p "$head_commit")
  fi
  virtual_head="$(printf 'sequential merge simulation PR #%s\n' "$number" | git commit-tree "$tree_id" "${parents[@]}")"
done < "$ordered_file"

echo "sequential PR merge simulation passed: ${count} PRs modeled; virtual_head=${virtual_head}"
