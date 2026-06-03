#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

check="scripts/ci/assert-rust-llvm-coverage-runner-contract.py"
spec="specs/rust-llvm-coverage-runner-contract.json"

PYTHONDONTWRITEBYTECODE=1 python3 "$check" --json > "$tmp_dir/good.json"
grep -Fq '"verdict": "PASS"' "$tmp_dir/good.json"
grep -Fq '"coverage_runner_contract_proven": true' "$tmp_dir/good.json"
grep -Fq '"coverage_report_generated": false' "$tmp_dir/good.json"
grep -Fq '"coverage_budget_enforced": false' "$tmp_dir/good.json"
grep -Fq '"live_required_context_execution_proven": false' "$tmp_dir/good.json"
grep -Fq '"p0_0_green": false' "$tmp_dir/good.json"
grep -Fq '"phase0_complete": false' "$tmp_dir/good.json"
python3 - <<'PY' "$tmp_dir/good.json"
import json, sys
x=json.load(open(sys.argv[1]))
assert x["verdict"] == "PASS"
assert x["failures"] == []
assert x["coverage_runner_contract_proven"] is True
assert x["coverage_report_generated"] is False
PY

assert_fails_with() {
  local label="$1"
  local expected="$2"
  local mutated="$tmp_dir/${label}.json"
  cp "$spec" "$mutated"
  shift 2
  python3 - <<'PY' "$mutated" "$@"
from pathlib import Path
import sys
path=Path(sys.argv[1])
text=path.read_text()
for spec in sys.argv[2:]:
    old, new = spec.split("=>", 1)
    if old not in text:
        raise SystemExit(f"mutation source not found: {old}")
    text=text.replace(old, new)
path.write_text(text)
PY
  local out="$tmp_dir/${label}.out"
  set +e
  PYTHONDONTWRITEBYTECODE=1 python3 "$check" --spec "$mutated" --json > "$out" 2>&1
  local rc=$?
  set -e
  if [ "$rc" -eq 0 ]; then
    echo "expected $label to fail" >&2
    cat "$out" >&2
    exit 1
  fi
  grep -Fq '"verdict": "FAIL"' "$out"
  grep -Fq "$expected" "$out"
  grep -Fq '"p0_0_green": false' "$out"
  grep -Fq '"phase0_complete": false' "$out"
}

assert_fails_with missing_instrument_flag 'missing_instrument_coverage_flag' \
  'rustc -C instrument-coverage=>rustc without coverage instrumentation'

assert_fails_with missing_profile_env 'missing_llvm_profile_file_env' \
  'LLVM_PROFILE_FILE=>PROFILE_FILE'

assert_fails_with missing_collision_guard 'missing_profile_collision_guard' \
  '%m-%p=>%p'

assert_fails_with missing_profdata 'missing_llvm_profdata_tool' \
  'llvm-profdata=>profile-merge-tool'

assert_fails_with missing_llvm_cov 'missing_llvm_cov_tool' \
  'llvm-cov=>coverage-export-tool'

assert_fails_with tarpaulin_boundary_missing 'tarpaulin_noncanonical_boundary_missing' \
  'Tarpaulin is not required CI/PR coverage evidence for this monorepo=>alternative coverage tool is canonical'

assert_fails_with generated_report_claim 'forbidden_true_or_missing_claim_coverage_report_generated' \
  '"coverage_report_generated": false=>"coverage_report_generated": true'

echo "rust LLVM coverage runner contract checks passed"
