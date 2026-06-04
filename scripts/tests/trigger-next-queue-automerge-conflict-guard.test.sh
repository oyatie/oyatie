#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

mkdir -p "$tmp_dir/bin"

cat > "$tmp_dir/bin/gh" <<'EOF_GH'
#!/usr/bin/env bash
set -euo pipefail

printf '%s\n' "$*" >> "${OYA_TEST_GH_LOG:?}"

if [ "$1" = "repo" ] && [ "${2:-}" = "view" ]; then
  printf 'jason931225/oyatie\n'
  exit 0
fi

if [ "$1" = "api" ] && [[ "${2:-}" == repos/*/branches/dev/protection/required_status_checks ]]; then
  cat "${OYA_TEST_LIVE_CONTEXTS:?}"
  exit 0
fi

if [ "$1" = "api" ] && [[ "${2:-}" == repos/*/commits/* ]]; then
  printf '{"verified":true,"reason":"valid"}\n'
  exit 0
fi

if [ "$1" = "pr" ] && [ "${2:-}" = "list" ]; then
  cat "${OYA_TEST_PRS:?}"
  exit 0
fi

if [ "$1" = "pr" ] && [ "${2:-}" = "view" ]; then
  cat "${OYA_TEST_PR_STATE:?}"
  exit 0
fi

if [ "$1" = "pr" ] && [ "${2:-}" = "checks" ]; then
  cat "${OYA_TEST_CHECKS:?}"
  exit 0
fi

if [ "$1" = "pr" ] && [ "${2:-}" = "merge" ]; then
  if [ -n "${OYA_TEST_GUARD_MARKER:-}" ] && [ ! -s "$OYA_TEST_GUARD_MARKER" ]; then
    printf 'gh pr merge reached before sequential conflict guard marker: %s\n' "$*" >&2
    exit 98
  fi
  {
    if [ -n "${OYA_TEST_GUARD_MARKER:-}" ]; then
      printf 'guard_marker=%s\n' "$(cat "$OYA_TEST_GUARD_MARKER")"
    fi
    printf '%s\n' "$*"
  } > "${OYA_TEST_MERGE_CALLED:?}"
  exit 0
fi

printf 'unexpected gh invocation: %s\n' "$*" >&2
exit 99
EOF_GH
chmod +x "$tmp_dir/bin/gh"

cat > "$tmp_dir/live-contexts.json" <<'JSON'
{
  "strict": false,
  "contexts": [
    "github-lane-unlocker-required"
  ]
}
JSON

cat > "$tmp_dir/checks.json" <<'JSON'
[
  {
    "name": "oya-pr-review",
    "bucket": "pass",
    "state": "SUCCESS",
    "workflow": "review"
  }
]
JSON

setup_queue_repo() {
  local scenario="$1"
  local work="$tmp_dir/work-${scenario}"
  local mirror="$tmp_dir/github-mirror-${scenario}.git"

  git init -q "$work"
  git -C "$work" config user.email "queue-test@example.invalid"
  git -C "$work" config user.name "queue-test"

  mkdir -p "$work/scripts/ci" "$work/infra/branch-protection"
  install -m 0755 "$repo_root/scripts/trigger-next-queue-automerge.sh" \
    "$work/scripts/trigger-next-queue-automerge.sh"
  install -m 0755 "$repo_root/scripts/check-sequential-pr-merge-conflicts.sh" \
    "$work/scripts/check-sequential-pr-merge-conflicts.sh"
  cp "$repo_root/scripts/check-sequential-pr-merge-conflicts.rs" \
    "$work/scripts/check-sequential-pr-merge-conflicts.rs"
  cp "$repo_root/scripts/ci/assert-result-bundle-output.rs" \
    "$work/scripts/ci/assert-result-bundle-output.rs"
  mv "$work/scripts/check-sequential-pr-merge-conflicts.sh" \
    "$work/scripts/check-sequential-pr-merge-conflicts.real.sh"
  # Local sequencing regression guard: the wrapper delegates to the real guard and
  # writes the marker only after that guard exits successfully. The fake gh merge
  # path refuses to record a non-dry-run merge without this marker. This is local
  # harness evidence, not a live cloud-ci/oya-ci authority claim.
  cat > "$work/scripts/check-sequential-pr-merge-conflicts.sh" <<'EOF_CHECK'
#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
"$script_dir/check-sequential-pr-merge-conflicts.real.sh" "$@"
if [ -n "${OYA_TEST_GUARD_MARKER:-}" ]; then
  printf 'guard passed: %s\n' "$*" > "$OYA_TEST_GUARD_MARKER"
fi
EOF_CHECK
  chmod +x "$work/scripts/check-sequential-pr-merge-conflicts.sh"
  cp "$repo_root/infra/branch-protection/dev.json" "$work/infra/branch-protection/dev.json"

  printf 'base\n' > "$work/shared.txt"
  git -C "$work" add shared.txt
  git -C "$work" commit -q -m "base"
  local base_commit
  base_commit="$(git -C "$work" rev-parse HEAD)"

  git -C "$work" checkout -q -b pr-455 "$base_commit"
  if [ "$scenario" = "conflict" ]; then
    printf 'pr-side\n' > "$work/shared.txt"
    git -C "$work" add shared.txt
  else
    printf 'pr-only\n' > "$work/pr.txt"
    git -C "$work" add pr.txt
  fi
  git -C "$work" commit -q -m "pr-455"
  local head_commit
  head_commit="$(git -C "$work" rev-parse HEAD)"

  git -C "$work" checkout -q -b dev "$base_commit"
  printf 'dev-side\n' > "$work/shared.txt"
  git -C "$work" add shared.txt
  git -C "$work" commit -q -m "dev advances"

  git init -q --bare "$mirror"
  git -C "$work" remote add origin "$tmp_dir/forgejo-origin-${scenario}.git"
  git -C "$work" remote add github-mirror "$mirror"
  git -C "$work" push -q github-mirror dev:refs/heads/dev
  git -C "$work" push -q github-mirror "$head_commit:refs/pull/455/head"

  cat > "$tmp_dir/${scenario}-prs.json" <<JSON
[
  {
    "number": 455,
    "headRefName": "feat/conflict-guard-${scenario}",
    "headRefOid": "${head_commit}",
    "isDraft": false,
    "title": "conflict guard ${scenario}"
  }
]
JSON

  cat > "$tmp_dir/${scenario}-state.json" <<JSON
{
  "isDraft": false,
  "mergeable": "MERGEABLE",
  "mergeStateStatus": "CLEAN",
  "reviewDecision": "APPROVED",
  "headRefOid": "${head_commit}"
}
JSON

  printf '%s\n' "$work"
}

run_trigger() {
  local scenario="$1"
  local work="$2"
  local dry_run="${3:-0}"
  local out="$tmp_dir/${scenario}.out"
  local err="$tmp_dir/${scenario}.err"
  local log="$tmp_dir/${scenario}-gh.log"
  local merge_called="$tmp_dir/${scenario}-merge-called"
  local guard_marker="$tmp_dir/${scenario}-guard-passed"
  local trigger_args=(
    --base-branch dev
    --base-ref dev
    --start-pr 455
    --limit 20
    --required-contexts-config infra/branch-protection/dev.json
    --fetch-remote github-mirror
  )
  rm -f "$log" "$merge_called" "$guard_marker"
  if [ "$dry_run" = "1" ]; then
    trigger_args+=(--dry-run)
  fi

  set +e
  (
    cd "$work"
    PATH="$tmp_dir/bin:$PATH" \
      OYA_TEST_GH_LOG="$log" \
      OYA_TEST_LIVE_CONTEXTS="$tmp_dir/live-contexts.json" \
      OYA_TEST_PRS="$tmp_dir/${scenario}-prs.json" \
      OYA_TEST_PR_STATE="$tmp_dir/${scenario}-state.json" \
      OYA_TEST_CHECKS="$tmp_dir/checks.json" \
      OYA_TEST_MERGE_CALLED="$merge_called" \
      OYA_TEST_GUARD_MARKER="$guard_marker" \
      scripts/trigger-next-queue-automerge.sh "${trigger_args[@]}"
  ) >"$out" 2>"$err"
  local status=$?
  set -e

  printf '%s\n' "$status" > "$tmp_dir/${scenario}.status"
}

clean_dry_work="$(setup_queue_repo clean-dry)"
run_trigger clean-dry "$clean_dry_work" 1
clean_dry_status="$(cat "$tmp_dir/clean-dry.status")"
if [ "$clean_dry_status" -ne 0 ]; then
  echo "expected clean queue candidate to reach dry-run auto-merge after conflict guard" >&2
  cat "$tmp_dir/clean-dry.out" >&2
  cat "$tmp_dir/clean-dry.err" >&2
  exit 1
fi
grep -Fq "sequential PR merge simulation passed: 1 PRs modeled" "$tmp_dir/clean-dry.out"
grep -Fq "dry-run: gh pr merge 455 --squash --auto --match-head-commit" "$tmp_dir/clean-dry.out"
grep -Fq "guard passed:" "$tmp_dir/clean-dry-guard-passed"
if [ -e "$tmp_dir/clean-dry-merge-called" ]; then
  echo "dry-run clean scenario must not invoke gh pr merge" >&2
  exit 1
fi

clean_real_work="$(setup_queue_repo clean-real)"
run_trigger clean-real "$clean_real_work" 0
clean_real_status="$(cat "$tmp_dir/clean-real.status")"
if [ "$clean_real_status" -ne 0 ]; then
  echo "expected clean queue candidate to invoke fake gh pr merge after conflict guard" >&2
  cat "$tmp_dir/clean-real.out" >&2
  cat "$tmp_dir/clean-real.err" >&2
  exit 1
fi
grep -Fq "sequential PR merge simulation passed: 1 PRs modeled" "$tmp_dir/clean-real.out"
grep -Fq "auto-merge enabled for bottom-most queue PR #455" "$tmp_dir/clean-real.out"
grep -Fq "guard_marker=guard passed:" "$tmp_dir/clean-real-merge-called"
grep -Fq "pr merge 455 --squash --auto --match-head-commit" "$tmp_dir/clean-real-merge-called"
if grep -Fq "dry-run: gh pr merge" "$tmp_dir/clean-real.out"; then
  echo "non-dry-run clean scenario unexpectedly used dry-run path" >&2
  exit 1
fi

conflict_work="$(setup_queue_repo conflict)"
run_trigger conflict "$conflict_work" 0
conflict_status="$(cat "$tmp_dir/conflict.status")"
if [ "$conflict_status" -eq 0 ]; then
  echo "expected conflicting queue candidate to fail before auto-merge" >&2
  cat "$tmp_dir/conflict.out" >&2
  cat "$tmp_dir/conflict.err" >&2
  exit 1
fi
grep -Fq "queue candidate: PR #455" "$tmp_dir/conflict.out"
grep -Fq "checking PR #455" "$tmp_dir/conflict.out"
grep -Fq "::error::sequential merge conflict at PR #455" "$tmp_dir/conflict.err"
if [ -e "$tmp_dir/conflict-merge-called" ]; then
  echo "conflict scenario invoked gh pr merge despite sequential guard failure" >&2
  cat "$tmp_dir/conflict-merge-called" >&2
  exit 1
fi
if [ -e "$tmp_dir/conflict-guard-passed" ]; then
  echo "conflict scenario recorded a sequential guard pass despite merge-tree conflict" >&2
  cat "$tmp_dir/conflict-guard-passed" >&2
  exit 1
fi
if grep -Fq "pr merge" "$tmp_dir/conflict-gh.log"; then
  echo "conflict scenario logged gh pr merge despite sequential guard failure" >&2
  cat "$tmp_dir/conflict-gh.log" >&2
  exit 1
fi

printf 'trigger-next-queue-automerge conflict guard tests passed\n'
