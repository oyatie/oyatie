#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

work="$tmp_dir/work"
mirror="$tmp_dir/github-mirror.git"

git init -q "$work"
git -C "$work" config user.email "queue-test@example.invalid"
git -C "$work" config user.name "queue-test"

printf 'base\n' > "$work/shared.txt"
git -C "$work" add shared.txt
git -C "$work" commit -q -m "base"
base_commit="$(git -C "$work" rev-parse HEAD)"

git init -q --bare "$mirror"
git -C "$work" remote add origin "$tmp_dir/forgejo-origin-does-not-exist.git"
git -C "$work" remote add github-mirror "$mirror"
git -C "$work" push -q github-mirror HEAD:refs/heads/dev

git -C "$work" checkout -q -b pr-455
printf 'pr\n' > "$work/pr.txt"
git -C "$work" add pr.txt
git -C "$work" commit -q -m "pr"
head_commit="$(git -C "$work" rev-parse HEAD)"
git -C "$work" push -q github-mirror HEAD:refs/pull/455/head

cat > "$tmp_dir/prs.json" <<JSON
[
  {
    "number": 455,
    "headRefName": "feat/pinned-head",
    "headRefOid": "${head_commit}",
    "isDraft": false,
    "title": "remote-select guard"
  }
]
JSON

(
  cd "$work"
  "$repo_root/scripts/check-sequential-pr-merge-conflicts.sh" \
    --base-branch dev \
    --base-ref "$base_commit" \
    --start-pr 455 \
    --end-pr 455 \
    --pr-json "$tmp_dir/prs.json" \
    --fetch-remote github-mirror > "$tmp_dir/pass.out"
)

grep -Fq "fetch_remote=github-mirror" "$tmp_dir/pass.out"
grep -Fq "sequential PR merge simulation passed: 1 PRs modeled" "$tmp_dir/pass.out"

set +e
(
  cd "$work"
  "$repo_root/scripts/check-sequential-pr-merge-conflicts.sh" \
    --base-branch dev \
    --base-ref "$base_commit" \
    --start-pr 455 \
    --end-pr 455 \
    --pr-json "$tmp_dir/prs.json" > "$tmp_dir/fail.out" 2> "$tmp_dir/fail.err"
)
status=$?
set -e

if [ "$status" -eq 0 ]; then
  echo "expected default origin fetch to fail when origin is the non-GitHub Forgejo remote" >&2
  exit 1
fi

grep -Fq "failed to fetch PR #455 head from remote origin" "$tmp_dir/fail.err"
grep -Fq "pass --fetch-remote for the GitHub mirror when origin is Forgejo" "$tmp_dir/fail.err"

printf 'check-sequential-pr-merge-conflicts fetch-remote tests passed\n'
