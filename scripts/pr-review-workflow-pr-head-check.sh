#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/pr-review-workflow-pr-head-check.sh [--source remote-default|head|worktree] [--remote REMOTE] [--branch BRANCH] [--workflow PATH] [--skip-remote-freshness]

Fail closed when the default-branch pr-review workflow cannot be proven to post
the required `oya-pr-review` Check Run onto the live PR head SHA.

Default source: remote-default
Default remote: origin
Default branch: dev
Default workflow: .github/workflows/pr-review.yml

The script is read-only. By default (`--source remote-default`) it verifies that
refs/remotes/<remote>/<branch> matches `git ls-remote <remote>
refs/heads/<branch>` before inspecting the workflow from that remote-tracking
ref. Pass --skip-remote-freshness only in tests or other already-frozen ref
contexts.

Use `--source worktree` for local/PR preflight of the candidate workflow before
it reaches the default branch. Use `--source head` for an immutable local commit
snapshot.
USAGE
}

remote="origin"
branch="dev"
workflow=".github/workflows/pr-review.yml"
skip_remote_freshness=0
source_mode="remote-default"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --remote)
      remote="${2:-}"
      shift 2
      ;;
    --branch)
      branch="${2:-}"
      shift 2
      ;;
    --workflow)
      workflow="${2:-}"
      shift 2
      ;;
    --source)
      source_mode="${2:-}"
      shift 2
      ;;
    --skip-remote-freshness)
      skip_remote_freshness=1
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

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "required command not found: $1" >&2
    exit 127
  fi
}

require_cmd git
require_cmd awk

if [[ -z "$remote" || -z "$branch" || -z "$workflow" ]]; then
  echo "::error::remote, branch, and workflow cannot be empty" >&2
  exit 64
fi

case "$source_mode" in
  remote-default)
    remote_ref="refs/remotes/${remote}/${branch}"
    remote_branch_ref="refs/heads/${branch}"

    if ! local_head="$(git rev-parse --verify "${remote_ref}^{commit}" 2>/dev/null)"; then
      echo "::error::missing ${remote_ref}; fetch ${remote}/${branch} before running ci-required verification" >&2
      exit 66
    fi

    if [[ "$skip_remote_freshness" -eq 0 ]]; then
      if ! remote_head="$(git ls-remote --exit-code "$remote" "$remote_branch_ref" 2>/tmp/oya-pr-review-ls-remote-error.$$ | awk '{print $1}')"; then
        echo "::error::could not read ${remote_branch_ref} from remote ${remote}; refusing stale default-branch workflow proof" >&2
        cat "/tmp/oya-pr-review-ls-remote-error.$$" >&2 || true
        rm -f "/tmp/oya-pr-review-ls-remote-error.$$"
        exit 1
      fi
      rm -f "/tmp/oya-pr-review-ls-remote-error.$$"
      if [[ -z "$remote_head" ]]; then
        echo "::error::remote ${remote} did not return ${remote_branch_ref}; refusing default-branch workflow proof" >&2
        exit 66
      fi
      if [[ "$remote_head" != "$local_head" ]]; then
        echo "::error::local ${remote_ref} is stale (${local_head}); remote ${remote_branch_ref} is ${remote_head}; fetch before push" >&2
        exit 1
      fi
    fi

    if ! workflow_text="$(git show "${remote_ref}:${workflow}" 2>/tmp/oya-pr-review-workflow-show-error.$$)"; then
      echo "::error::could not read ${workflow} from ${remote_ref}; default branch cannot emit required oya-pr-review proof" >&2
      cat "/tmp/oya-pr-review-workflow-show-error.$$" >&2 || true
      rm -f "/tmp/oya-pr-review-workflow-show-error.$$"
      exit 66
    fi
    rm -f "/tmp/oya-pr-review-workflow-show-error.$$"
    workflow_label="${remote_ref}:${workflow}"
    ;;
  head)
    if ! local_head="$(git rev-parse --verify "HEAD^{commit}" 2>/dev/null)"; then
      echo "::error::could not resolve HEAD before running pr-review workflow proof" >&2
      exit 66
    fi
    if ! workflow_text="$(git show "HEAD:${workflow}" 2>/tmp/oya-pr-review-workflow-show-error.$$)"; then
      echo "::error::could not read ${workflow} from HEAD; candidate branch cannot emit required oya-pr-review proof" >&2
      cat "/tmp/oya-pr-review-workflow-show-error.$$" >&2 || true
      rm -f "/tmp/oya-pr-review-workflow-show-error.$$"
      exit 66
    fi
    rm -f "/tmp/oya-pr-review-workflow-show-error.$$"
    workflow_label="HEAD:${workflow}"
    ;;
  worktree)
    if ! local_head="$(git rev-parse --verify "HEAD^{commit}" 2>/dev/null)"; then
      echo "::error::could not resolve HEAD before running pr-review workflow proof" >&2
      exit 66
    fi
    if [[ ! -f "$workflow" ]]; then
      echo "::error::could not read ${workflow} from working tree; candidate branch cannot emit required oya-pr-review proof" >&2
      exit 66
    fi
    workflow_text="$(cat "$workflow")"
    workflow_label="working-tree:${workflow}"
    ;;
  *)
    echo "::error::invalid --source ${source_mode}; expected remote-default, head, or worktree" >&2
    exit 64
    ;;
esac

missing=()
require_marker() {
  local marker="$1"
  local reason="$2"
  if ! grep -Fq -- "$marker" <<<"$workflow_text"; then
    missing+=("${reason} (${marker})")
  fi
}

require_marker "name: oya-pr-review" "job/check name must remain oya-pr-review"
require_marker 'GH_REPO: ${{ github.repository }}' "workflow must bind GitHub CLI calls to the repository"
require_marker "headRefOid" "workflow must resolve the live PR head SHA"
require_marker 'ref: ${{ steps.pr.outputs.pr_head_sha }}' "checkout must use the resolved PR head SHA"
require_marker '--change-id "${{ steps.pr.outputs.pr_head_sha }}"' "review runtime change-id must use the PR head SHA"
require_marker 'gh api -X POST "repos/${GITHUB_REPOSITORY}/check-runs"' "workflow must post an explicit Check Run"
require_marker '-f name="oya-pr-review"' "explicit Check Run name must match branch protection"
require_marker 'HEAD_SHA: ${{ steps.pr.outputs.pr_head_sha }}' "explicit Check Run HEAD_SHA env must use the PR head SHA"
require_marker '-f head_sha="${HEAD_SHA}"' "explicit Check Run must attach to the PR head SHA"

if grep -Fq -- 'ref: ${{ github.event.workflow_run.head_sha }}' <<<"$workflow_text"; then
  missing+=("checkout still uses workflow_run.head_sha instead of the live PR head")
fi
if grep -Fq -- '--change-id "${{ github.event.workflow_run.head_sha }}"' <<<"$workflow_text"; then
  missing+=("review runtime change-id still uses workflow_run.head_sha instead of the live PR head")
fi
if grep -Eq '^[[:space:]]*HEAD_SHA:[[:space:]]*\$\{\{ github\.event\.workflow_run\.head_sha \}\}' <<<"$workflow_text"; then
  missing+=("explicit Check Run HEAD_SHA still uses workflow_run.head_sha instead of the live PR head")
fi

if [[ "${#missing[@]}" -gt 0 ]]; then
  echo "::error::${workflow_label} cannot prove PR-head oya-pr-review Check Run semantics" >&2
  for item in "${missing[@]}"; do
    echo "  - ${item}" >&2
  done
  exit 1
fi

echo "pr-review workflow PR-head check-run semantics verified for ${workflow_label} (${local_head})"
