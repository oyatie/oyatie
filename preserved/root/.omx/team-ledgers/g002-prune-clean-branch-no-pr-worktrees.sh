#!/usr/bin/env bash
set -euo pipefail

# Removes clean branch worktrees with no associated PR. Branch refs are preserved; only checkout directories are removed.
# Safe by default: without --execute, this script only revalidates and prints candidate actions.

mode="${1:---dry-run}"
if [[ "$mode" != "--dry-run" && "$mode" != "--execute" ]]; then
  echo "usage: $0 [--dry-run|--execute]" >&2
  exit 2
fi
if [[ "$mode" == "--execute" ]]; then
  echo "DESTRUCTIVE MODE: removing clean branch worktree checkouts only; branch refs are preserved." >&2
else
  echo "DRY RUN: validating clean branch no-PR worktrees; no worktrees will be removed." >&2
fi

candidates=(
  /Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-10
  /Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-1
  /Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-2
  /Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-3
  /Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-4
  /Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-5
  /Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-6
  /Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-7
  /Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-8
  /Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-9
  /Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_a6c6939e-488-1
  /Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_a6c6939e-488-3
  /Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_a6c6939e-488-4
  /Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_a6c6939e-488-5
  /Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_a6c6939e-488-6
  /Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_b306f25c-be3-1
  /Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_c4dfe460-b55-1
  /Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_c4dfe460-b55-2
  /Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_c4dfe460-b55-3
  /Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_c4dfe460-b55-4
  /Users/jasonlee/oyatie-worktrees/team-python-mjs-rust-leader-20260626T105958Z
  /Users/jasonlee/oyatie-worktrees/team-leader-ci-velocity-20260626T043322Z
  /Users/jasonlee/oyatie-worktrees/waveA-ast-transpiler-20260625173629
  /Users/jasonlee/oyatie-worktrees/waveA-collab-office-20260625173629
  /Users/jasonlee/oyatie-worktrees/waveA-crm-marketing-20260625173629
  /Users/jasonlee/oyatie-worktrees/ultragoal-wave-a-20260626T203951Z
  /Users/jasonlee/oyatie-worktrees/python-mjs-rust-20260626T105334Z
  /Users/jasonlee/oyatie-worktrees/ci-pr-cancel-20260626T025326Z
  /Users/jasonlee/oyatie-worktrees/waveA-cloud-ci-20260625173629
  /Users/jasonlee/oyatie-worktrees/waveA-iac-k8s-20260625173629
  /Users/jasonlee/oyatie-worktrees/waveA-kms-iam-20260625173629
  /Users/jasonlee/oyatie-worktrees/waveA-erp-20260625173629
)

removed=0
skipped=0
failed=0
for wt in "${candidates[@]}"; do
  echo "--- $wt"
  if [[ ! -d "$wt/.git" && ! -f "$wt/.git" ]]; then
    echo "skip: not a git worktree path"; skipped=$((skipped+1)); continue
  fi
  branch_ref="$(git -C "$wt" symbolic-ref -q HEAD || true)"
  if [[ -z "$branch_ref" ]]; then
    echo "skip: detached HEAD"; skipped=$((skipped+1)); continue
  fi
  status_out="$(git -C "$wt" status --short --branch)"
  echo "$status_out"
  dirty_count="$(awk 'BEGIN { c=0 } $0 !~ /^##/ && $0 != "" { c++ } END { print c }' <<< "$status_out")"
  if [[ "$dirty_count" != "0" ]]; then
    echo "skip: dirty_count=$dirty_count"; skipped=$((skipped+1)); continue
  fi
  if ! git show-ref --verify --quiet "$branch_ref"; then
    echo "skip: branch ref missing $branch_ref"; skipped=$((skipped+1)); continue
  fi
  echo "branch-ref-preserved: $branch_ref"
  if [[ "$mode" == "--execute" ]]; then
    if git worktree remove "$wt"; then echo "removed: $wt"; removed=$((removed+1)); else echo "failed-remove: $wt"; failed=$((failed+1)); fi
  else
    echo "would remove: $wt"
  fi
done
if [[ "$mode" == "--execute" ]]; then git worktree prune --verbose || true; fi
echo "summary: removed=$removed skipped=$skipped failed=$failed mode=$mode"