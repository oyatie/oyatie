#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
script="$repo_root/scripts/pr-review-workflow-pr-head-check.sh"
source_workflow="$repo_root/.github/workflows/pr-review.yml"
if [[ ! -f "$source_workflow" ]]; then
  echo "SKIP: .github/workflows/pr-review.yml is not present in this checkout"
  exit 0
fi
tmpdir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmpdir"
}
trap cleanup EXIT

make_repo() {
  local source_file="$1"
  local repo="$2"
  mkdir -p "$repo/.github/workflows"
  cp "$source_file" "$repo/.github/workflows/pr-review.yml"
  (
    cd "$repo"
    git init -q
    git config user.email "tests@example.invalid"
    git config user.name "tests"
    git add .github/workflows/pr-review.yml
    git commit -q -m "seed pr-review workflow"
    git update-ref refs/remotes/origin/dev HEAD
  )
}

good_repo="$tmpdir/good"
make_repo "$source_workflow" "$good_repo"
(
  cd "$good_repo"
  "$script" --skip-remote-freshness
)

stale_workflow="$tmpdir/stale-pr-review.yml"
cat > "$stale_workflow" <<'YAML'
name: oya-governance-pr-review
on:
  workflow_run:
    workflows:
      - pr-tests
jobs:
  oya-pr-review:
    name: oya-pr-review
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
        with:
          ref: ${{ github.event.workflow_run.head_sha }}
      - run: echo "implicit workflow_run job check only"
YAML

stale_repo="$tmpdir/stale"
make_repo "$stale_workflow" "$stale_repo"
(
  cd "$stale_repo"
  if "$script" --skip-remote-freshness 2>"$tmpdir/stale.err"; then
    echo "expected stale default-branch pr-review workflow to fail" >&2
    exit 1
  fi
)
grep -q 'cannot prove PR-head oya-pr-review Check Run semantics' "$tmpdir/stale.err"
grep -q 'workflow_run.head_sha' "$tmpdir/stale.err"
grep -q 'headRefOid' "$tmpdir/stale.err"
grep -q 'head_sha' "$tmpdir/stale.err"

candidate_worktree_repo="$tmpdir/candidate-worktree"
make_repo "$stale_workflow" "$candidate_worktree_repo"
cp "$source_workflow" "$candidate_worktree_repo/.github/workflows/pr-review.yml"
(
  cd "$candidate_worktree_repo"
  "$script" --source worktree
  git add .github/workflows/pr-review.yml
  git commit -q -m "candidate pr-review workflow"
  "$script" --source head
)

stale_headsha_workflow="$tmpdir/stale-headsha-pr-review.yml"
cat > "$stale_headsha_workflow" <<'YAML'
name: oya-governance-pr-review
on:
  workflow_run:
    workflows:
      - pr-tests
jobs:
  oya-pr-review:
    name: oya-pr-review
    runs-on: ubuntu-latest
    steps:
      - name: Resolve live PR head
        id: pr
        env:
          GH_REPO: ${{ github.repository }}
        run: |
          gh pr view 1 --json headRefOid --jq '.headRefOid'
      - uses: actions/checkout@v6
        with:
          ref: ${{ steps.pr.outputs.pr_head_sha }}
      - name: Fan out review
        run: |
          buck2 run //oya/intelligence/crates/oya-intelligence-subagent-runtime-app:oya-intelligence-subagent-runtime-app-bin -- fan-out \
            --change-id "${{ steps.pr.outputs.pr_head_sha }}"
      - name: Publish oya-pr-review
        env:
          HEAD_SHA: ${{ github.event.workflow_run.head_sha }}
        run: |
          gh api -X POST "repos/${GITHUB_REPOSITORY}/check-runs" \
            -f name="oya-pr-review" \
            -f head_sha="${HEAD_SHA}"
YAML

stale_headsha_repo="$tmpdir/stale-headsha"
make_repo "$stale_headsha_workflow" "$stale_headsha_repo"
(
  cd "$stale_headsha_repo"
  if "$script" --skip-remote-freshness 2>"$tmpdir/stale-headsha.err"; then
    echo "expected stale Check Run HEAD_SHA binding to fail" >&2
    exit 1
  fi
)
grep -q 'Check Run HEAD_SHA' "$tmpdir/stale-headsha.err"
grep -q 'workflow_run.head_sha' "$tmpdir/stale-headsha.err"

missing_repo="$tmpdir/missing-workflow"
mkdir -p "$missing_repo"
(
  cd "$missing_repo"
  git init -q
  git config user.email "tests@example.invalid"
  git config user.name "tests"
  echo "seed" > README.md
  git add README.md
  git commit -q -m "seed without workflow"
  git update-ref refs/remotes/origin/dev HEAD
  if "$script" --skip-remote-freshness 2>"$tmpdir/missing-workflow.err"; then
    echo "expected missing default-branch pr-review workflow to fail" >&2
    exit 1
  fi
)
grep -q 'could not read .github/workflows/pr-review.yml' "$tmpdir/missing-workflow.err"

stale_remote_repo="$tmpdir/stale-remote"
make_repo "$repo_root/.github/workflows/pr-review.yml" "$stale_remote_repo"
(
  cd "$stale_remote_repo"
  initial="$(git rev-parse HEAD)"
  git remote add origin "$tmpdir/remote.git"
  git init -q --bare "$tmpdir/remote.git"
  git push -q origin HEAD:refs/heads/dev
  echo "# remote advanced" >> .github/workflows/pr-review.yml
  git add .github/workflows/pr-review.yml
  git commit -q -m "advance remote"
  git push -q origin HEAD:refs/heads/dev
  git update-ref refs/remotes/origin/dev "$initial"
  if "$script" 2>"$tmpdir/stale-remote.err"; then
    echo "expected stale origin/dev tracking ref to fail" >&2
    exit 1
  fi
)
grep -q 'local refs/remotes/origin/dev is stale' "$tmpdir/stale-remote.err"

echo "pr-review-workflow-pr-head-check tests passed"
