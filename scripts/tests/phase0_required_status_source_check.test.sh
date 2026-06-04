#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

check="scripts/ci/assert-required-status-source.py"
fixtures="specs/fixtures/phase0-required-status-source"

python3 "$check" --input "$fixtures/good-bound-expected-source-app.json" --expected-app-id 12345 --json > "$tmp_dir/good.json"
grep -Fq '"verdict": "PASS"' "$tmp_dir/good.json"
grep -Fq '"trusted_source_app_proven": true' "$tmp_dir/good.json"

assert_fails_with_reason() {
  local fixture="$1"
  local reason="$2"
  shift 2
  local out="$tmp_dir/${fixture%.json}.json"
  set +e
  python3 "$check" --input "$fixtures/$fixture" "$@" --json > "$out" 2>&1
  local status=$?
  set -e
  if [ "$status" -eq 0 ]; then
    echo "expected $fixture to fail" >&2
    cat "$out" >&2
    exit 1
  fi
  grep -Fq '"verdict": "FAIL"' "$out"
  grep -Fq "\"reason\": \"$reason\"" "$out"
  grep -Fq '"p0_0_green": false' "$out"
  grep -Fq '"phase0_complete": false' "$out"
}

assert_fails_with_reason good-bound-expected-source-app.json expected_source_app_id_not_configured
assert_fails_with_reason bad-contexts-only-no-checks-array.json missing_required_status_checks_checks_array --expected-app-id 12345
assert_fails_with_reason bad-null-source-app.json missing_required_status_source_app --expected-app-id 12345
assert_fails_with_reason bad-wildcard-any-source-app.json wildcard_required_status_source_app --expected-app-id 12345
assert_fails_with_reason bad-wrong-source-app.json wrong_required_status_source_app --expected-app-id 12345
assert_fails_with_reason bad-missing-required-context.json missing_required_context --expected-app-id 12345
assert_fails_with_reason bad-required-context-not-in-checks-array.json required_context_not_in_checks_array --expected-app-id 12345

echo "phase0 required-status source binding tests passed"
