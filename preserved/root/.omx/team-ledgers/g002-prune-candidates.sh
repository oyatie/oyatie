#!/usr/bin/env bash
set -euo pipefail

# Generated from .omx/context/g002-intake/worktree-classification-20260626T2250Z.json
# Removes only candidates that revalidate clean and whose HEAD is ancestor of origin/dev.
# Safe by default: without --execute, this script only revalidates and prints candidate actions.

mode="${1:---dry-run}"
if [[ "$mode" != "--dry-run" && "$mode" != "--execute" ]]; then
  echo "usage: $0 [--dry-run|--execute]" >&2
  exit 2
fi

if [[ "$mode" == "--execute" ]]; then
  echo "DESTRUCTIVE MODE: removing only candidates that still validate clean and merged into origin/dev." >&2
else
  echo "DRY RUN: validating candidates; no worktrees will be removed." >&2
fi

git fetch --prune origin dev >/dev/null 2>&1 || true

candidates=(
  /Users/jasonlee/oyatie-worktrees/team-leader-ci12-c2-20260626T045700Z/.omx/team/group-c2-opus-xhigh-c-9f982c4a/worktrees/worker-1
  /Users/jasonlee/oyatie-worktrees/team-leader-ci12-c2-20260626T045700Z/.omx/team/group-c2-opus-xhigh-c-9f982c4a/worktrees/worker-2
  /Users/jasonlee/oyatie-worktrees/team-leader-ci12-c2-20260626T045700Z/.omx/team/group-c2-opus-xhigh-c-9f982c4a/worktrees/worker-3
  /Users/jasonlee/oyatie-worktrees/team-leader-ci12-c2-20260626T045700Z/.omx/team/group-c2-opus-xhigh-c-9f982c4a/worktrees/worker-4
  /Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/execute-team-bound-ul-9c35bb10/worktrees/worker-1
  /Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/execute-team-bound-ul-9c35bb10/worktrees/worker-3
  /Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/execute-team-bound-ul-9c35bb10/worktrees/worker-4
  /Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/execute-team-bound-ul-9c35bb10/worktrees/worker-5
  /Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/wavea-back-fixed-g015-9c35bb10/worktrees/worker-1
  /Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/wavea-back-fixed-g015-9c35bb10/worktrees/worker-2
  /Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/wavea-back-fixed-g015-9c35bb10/worktrees/worker-3
  /Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/wavea-back-fixed-g015-9c35bb10/worktrees/worker-4
  /Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/wavea-back-fixed-g015-9c35bb10/worktrees/worker-5
  /Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/wavea-front-fixed-g01-9c35bb10/worktrees/worker-1
  /Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/wavea-front-fixed-g01-9c35bb10/worktrees/worker-2
  /Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/wavea-front-fixed-g01-9c35bb10/worktrees/worker-3
  /Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/wavea-front-fixed-g01-9c35bb10/worktrees/worker-4
  /Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/wavea-front-fixed-g01-9c35bb10/worktrees/worker-5
  /Users/jasonlee/oyatie-worktrees/python-mjs-rust-worker4-anchor-sweep-20260626T112327Z
  /Users/jasonlee/oyatie-worktrees/advisory-claude-reviewers-20260625T223854Z
  /Users/jasonlee/oyatie-worktrees/team-leader-waveb-pr-wip-20260626T023204Z
  /Users/jasonlee/oyatie-worktrees/team-leader-ci12-c2-20260626T045700Z
  /Users/jasonlee/oyatie-worktrees/team-leader-ci12-b-20260626T044133Z
  /Users/jasonlee/oyatie-worktrees/team-leader-ci12-c-20260626T044133Z
  /Users/jasonlee/oyatie-worktrees/waveA-market-billing-20260625173629
  /Users/jasonlee/oyatie-worktrees/team-leader-ci12-20260626T044133Z
  /Users/jasonlee/oyatie-worktrees/python-mjs-rust-20260626T105028Z
  /Users/jasonlee/oyatie-worktrees/waveA-hr-payroll-20260625173629
  /Users/jasonlee/oyatie-worktrees/waveA-kernel-os-20260625173629
  /Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625
)

removed=0
skipped=0
failed=0
for wt in "${candidates[@]}"; do
  echo "--- $wt"
  if [[ ! -d "$wt/.git" && ! -f "$wt/.git" ]]; then
    echo "skip: not a git worktree path"
    skipped=$((skipped+1))
    continue
  fi
  status_out="$(git -C "$wt" status --short --branch)"
  echo "$status_out"
  dirty_count="$(awk 'BEGIN { c=0 } $0 !~ /^##/ && $0 != "" { c++ } END { print c }' <<< "$status_out")"
  if [[ "$dirty_count" != "0" ]]; then
    echo "skip: dirty_count=$dirty_count"
    skipped=$((skipped+1))
    continue
  fi
  if ! git -C "$wt" merge-base --is-ancestor HEAD refs/remotes/origin/dev; then
    echo "skip: HEAD not ancestor of origin/dev"
    skipped=$((skipped+1))
    continue
  fi
  if [[ "$mode" == "--execute" ]]; then
    if git worktree remove "$wt"; then
      echo "removed: $wt"
      removed=$((removed+1))
    else
      echo "failed-remove: $wt"
      failed=$((failed+1))
    fi
  else
    echo "would remove: $wt"
  fi
done

if [[ "$mode" == "--execute" ]]; then
  git worktree prune --verbose || true
fi

echo "summary: removed=$removed skipped=$skipped failed=$failed mode=$mode"