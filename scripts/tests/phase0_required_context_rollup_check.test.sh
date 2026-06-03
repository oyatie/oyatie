#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

check="scripts/ci/assert-pr-required-context.py"
fixtures="specs/fixtures/phase0-required-context-rollup"

python3 "$check" --input "$fixtures/good-oya-ci-required-success.json" --json > "$tmp_dir/good.json"
grep -Fq '"verdict": "PASS"' "$tmp_dir/good.json"
grep -Fq '"required_context_proven": true' "$tmp_dir/good.json"
grep -Fq '"required_context_trusted_producer": true' "$tmp_dir/good.json"

python3 "$check" --input "$fixtures/good-nested-cloud-ci-oya-ci-success.json" --json > "$tmp_dir/good-nested.json"
grep -Fq '"verdict": "PASS"' "$tmp_dir/good-nested.json"
grep -Fq '"required_context_proven": true' "$tmp_dir/good-nested.json"
grep -Fq '"required_context_trusted_producer": true' "$tmp_dir/good-nested.json"

assert_fails_with_reason() {
  local fixture="$1"
  local reason="$2"
  local out="$tmp_dir/${fixture%.json}.json"
  set +e
  python3 "$check" --input "$fixtures/$fixture" --json > "$out" 2>&1
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

assert_fails_with_reason bad-no-checks-reported.json no_status_checks_reported
assert_fails_with_reason bad-missing-oya-ci-required.json missing_required_context
assert_fails_with_reason bad-oya-ci-required-failure.json required_context_not_success
assert_fails_with_reason bad-oya-ci-required-completed-failure.json required_context_not_success
assert_fails_with_reason bad-oya-ci-required-success-missing-producer.json missing_required_context_producer
assert_fails_with_reason bad-oya-ci-required-success-untrusted-producer.json untrusted_required_context_producer

echo "phase0 required-context rollup check tests passed"
