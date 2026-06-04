#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

head_sha="abc123def4567890abc123def4567890abc123de"
stale_sha="def4567890abc123def4567890abc123def45678"

mkdir -p "$tmp_dir/bin"
cat > "$tmp_dir/bin/curl" <<'FAKE_CURL'
#!/usr/bin/env bash
set -euo pipefail
method="GET"
data=""
url=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    -X)
      method="${2:?}"
      shift 2
      ;;
    --data)
      data="${2:?}"
      shift 2
      ;;
    -H|-w|--max-time)
      shift 2
      ;;
    -s|-S|-sS)
      shift
      ;;
    *)
      url="$1"
      shift
      ;;
  esac
done

status_body() {
  printf '%s\n%s' "$1" "$2"
}

case "${method} ${url}" in
  "GET "*"/branch_protections/dev")
    status_body '{"branch_name":"dev","enable_status_check":true,"status_check_contexts":["oya-ci-required"]}' 200
    ;;
  "PATCH "*"/branch_protections/dev")
    printf '%s\n' "$data" > "${OYA_TEST_PATCH_PAYLOAD:?}"
    status_body '{"branch_name":"dev","enable_status_check":true,"status_check_contexts":["oya-ci-required"]}' 200
    ;;
  "GET "*"/pulls/455")
    case "${OYA_TEST_PR_MODE:?}" in
      good)
        status_body '{"head":{"sha":"abc123def4567890abc123def4567890abc123de"},"mergeable":true}' 200
        ;;
      stale)
        status_body '{"head":{"sha":"def4567890abc123def4567890abc123def45678"},"mergeable":true}' 200
        ;;
      conflict)
        status_body '{"head":{"sha":"abc123def4567890abc123def4567890abc123de"},"mergeable":false}' 200
        ;;
      unknown)
        status_body '{"head":{"sha":"abc123def4567890abc123def4567890abc123de"},"mergeable":null}' 200
        ;;
      *)
        echo "unexpected OYA_TEST_PR_MODE=${OYA_TEST_PR_MODE}" >&2
        exit 97
        ;;
    esac
    ;;
  "POST "*"/pulls/455/merge")
    printf '%s\n' "$data" > "${OYA_TEST_MERGE_PAYLOAD:?}"
    status_body '{"scheduled":true}' 202
    ;;
  *)
    echo "unexpected curl invocation: method=${method} url=${url}" >&2
    exit 98
    ;;
esac
FAKE_CURL
chmod +x "$tmp_dir/bin/curl"

scripts/ci/arm-auto-merge.sh \
  --dry-run \
  --pr-index 455 \
  --head-commit "$head_sha" \
  > "$tmp_dir/dry-run.out"

grep -Fq '"status_check_contexts": ["oya-ci-required"]' "$tmp_dir/dry-run.out"
grep -Fq "would POST" "$tmp_dir/dry-run.out"
grep -Fq '/api/v1/repos/oya-admin/oyatie/pulls/455/merge' "$tmp_dir/dry-run.out"
grep -Fq '"Do": "squash"' "$tmp_dir/dry-run.out"
grep -Fq '"merge_when_checks_succeed": true' "$tmp_dir/dry-run.out"
grep -Fq '"delete_branch_after_merge": true' "$tmp_dir/dry-run.out"
grep -Fq "\"head_commit_id\": \"${head_sha}\"" "$tmp_dir/dry-run.out"

run_expect_fail() {
  local name="$1"
  local expected="$2"
  shift 2
  local out="$tmp_dir/${name}.out"
  local err="$tmp_dir/${name}.err"
  set +e
  "$@" > "$out" 2> "$err"
  local status=$?
  set -e
  if [ "$status" -eq 0 ]; then
    echo "expected ${name} to fail closed" >&2
    cat "$out" >&2
    cat "$err" >&2
    exit 1
  fi
  grep -Fq -- "$expected" "$err"
}

run_expect_fail missing-head '--head-commit is required with --pr-index' \
  scripts/ci/arm-auto-merge.sh --dry-run --pr-index 455

run_expect_fail short-head '--head-commit must be a full SHA-1 (40 hex) or SHA-256 (64 hex) commit id' \
  scripts/ci/arm-auto-merge.sh --dry-run --pr-index 455 --head-commit abc123d

run_expect_fail context-override 'REQUIRED_CONTEXT is fixed to oya-ci-required' \
  env REQUIRED_CONTEXT=legacy-context scripts/ci/arm-auto-merge.sh --dry-run

run_expect_fail unsafe-method '--merge-method is fixed to squash' \
  scripts/ci/arm-auto-merge.sh --dry-run --pr-index 455 --head-commit "$head_sha" --merge-method merge

run_expect_fail unsafe-delete-branch '--delete-branch-after-merge is fixed to true' \
  scripts/ci/arm-auto-merge.sh --dry-run --pr-index 455 --head-commit "$head_sha" --delete-branch-after-merge false

PATH="$tmp_dir/bin:$PATH" \
  FORGEJO_TOKEN=test-token \
  OYA_TEST_PR_MODE=good \
  OYA_TEST_PATCH_PAYLOAD="$tmp_dir/good.patch.json" \
  OYA_TEST_MERGE_PAYLOAD="$tmp_dir/good.merge.json" \
  scripts/ci/arm-auto-merge.sh --pr-index 455 --head-commit "$head_sha" > "$tmp_dir/good.out"

grep -Fq 'PR #455 head and mergeability guard passed' "$tmp_dir/good.out"
grep -Fq "auto-merge scheduled for PR #455 at expected head ${head_sha}" "$tmp_dir/good.out"
grep -Fq '"status_check_contexts": ["oya-ci-required"]' "$tmp_dir/good.patch.json"
grep -Fq '"merge_when_checks_succeed": true' "$tmp_dir/good.merge.json"
grep -Fq '"head_commit_id": "abc123def4567890abc123def4567890abc123de"' "$tmp_dir/good.merge.json"

for mode in stale conflict unknown; do
  rm -f "$tmp_dir/${mode}.merge.json"
  out="$tmp_dir/${mode}.out"
  err="$tmp_dir/${mode}.err"
  set +e
  PATH="$tmp_dir/bin:$PATH" \
    FORGEJO_TOKEN=test-token \
    OYA_TEST_PR_MODE="$mode" \
    OYA_TEST_PATCH_PAYLOAD="$tmp_dir/${mode}.patch.json" \
    OYA_TEST_MERGE_PAYLOAD="$tmp_dir/${mode}.merge.json" \
    scripts/ci/arm-auto-merge.sh --pr-index 455 --head-commit "$head_sha" > "$out" 2> "$err"
  status=$?
  set -e
  if [ "$status" -eq 0 ]; then
    echo "expected ${mode} PR guard to fail closed" >&2
    cat "$out" >&2
    exit 1
  fi
  if [ -e "$tmp_dir/${mode}.merge.json" ]; then
    echo "${mode} guard must fail before POST /pulls/455/merge" >&2
    cat "$tmp_dir/${mode}.merge.json" >&2
    exit 1
  fi
  case "$mode" in
    stale) grep -Fq 'does not match expected' "$err" ;;
    conflict) grep -Fq 'mergeable=false' "$err" ;;
    unknown) grep -Fq 'mergeable=null' "$err" ;;
  esac
done

if grep -Fq 'oya-ci-gate' scripts/ci/arm-auto-merge.sh docs/ci/auto-merge-flow.md docs/ci/forge-of-record.md; then
  echo "active auto-merge docs/scripts must not reference stale oya-ci-gate" >&2
  exit 1
fi

printf 'forgejo auto-merge-after-ci dry-run and guarded scheduling tests passed\n'
