#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

check="scripts/ci/assert-rust-testing-standard.py"
doc="docs/standards/testing.md"

PYTHONDONTWRITEBYTECODE=1 python3 "$check" --json > "$tmp_dir/good.json"
grep -Fq '"verdict": "PASS"' "$tmp_dir/good.json"
grep -Fq '"standard_contract_proven": true' "$tmp_dir/good.json"
grep -Fq '"coverage_runner_implemented": false' "$tmp_dir/good.json"
grep -Fq '"mutation_lane_implemented": false' "$tmp_dir/good.json"
grep -Fq '"live_required_context_execution_proven": false' "$tmp_dir/good.json"
grep -Fq '"protected_branch_authority_proven": false' "$tmp_dir/good.json"
grep -Fq '"p0_0_green": false' "$tmp_dir/good.json"
grep -Fq '"phase0_complete": false' "$tmp_dir/good.json"
grep -Fq '"production_ready": false' "$tmp_dir/good.json"
grep -Fq '"hyperscaler_grade": false' "$tmp_dir/good.json"
python3 - <<'PY' "$tmp_dir/good.json"
import json, sys
x=json.load(open(sys.argv[1]))
assert x["verdict"] == "PASS"
assert x["failures"] == []
assert x["anchor_count"] == x["anchors_present"]
assert x["coverage_runner_implemented"] is False
assert x["mutation_lane_implemented"] is False
PY

assert_fails_with() {
  local label="$1"
  local expected="$2"
  local mutated="$tmp_dir/${label}.md"
  cp "$doc" "$mutated"
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
  local out="$tmp_dir/${label}.json"
  set +e
  PYTHONDONTWRITEBYTECODE=1 python3 "$check" --doc "$mutated" --json > "$out" 2>&1
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

assert_fails_with missing_buck2_llvm 'missing_buck2_native_llvm_coverage_policy' \
  'Buck2-native LLVM source-based coverage=>ad-hoc coverage'

assert_fails_with tarpaulin_canonicalized 'tarpaulin_canonicalized' \
  'Tarpaulin is not the canonical coverage surface=>Tarpaulin is the canonical coverage surface'

assert_fails_with missing_profile_file 'missing_llvm_profile_file' \
  'LLVM_PROFILE_FILE=>PROFILE_FILE'

assert_fails_with local_mutation_not_advisory 'local_cargo_mutation_not_advisory' \
  'Local Cargo mutation output is advisory=>Local Cargo mutation output is authoritative'

assert_fails_with missing_reindeer_generated_buck 'missing_reindeer_generated_buck' \
  'reindeer-style generation=>manual vendoring' \
  'generated-BUCK path=>hand-written path'

assert_fails_with forbidden_green_claim 'forbidden_true_claim_p0_0_green' \
  '## 12. Sources scanned=>p0_0_green=true\n\n## 12. Sources scanned'

echo "rust testing standard checks passed"
