#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

check="scripts/ci/run-rust-llvm-coverage-smoke.py"
source_fixture="specs/fixtures/rust-llvm-coverage-smoke/branchy.rs"

PYTHONDONTWRITEBYTECODE=1 python3 "$check" --source "$source_fixture" --out "$tmp_dir/good.json" > "$tmp_dir/good.stdout"
grep -Fq '"verdict": "PASS"' "$tmp_dir/good.json"
grep -Fq '"fixture_coverage_smoke_generated": true' "$tmp_dir/good.json"
grep -Fq '"production_coverage_report_generated": false' "$tmp_dir/good.json"
grep -Fq '"coverage_budget_enforced": false' "$tmp_dir/good.json"
grep -Fq '"live_required_context_execution_proven": false' "$tmp_dir/good.json"
grep -Fq '"p0_0_green": false' "$tmp_dir/good.json"
grep -Fq '"phase0_complete": false' "$tmp_dir/good.json"
grep -Fq '"ambient_path_llvm_tools_required": false' "$tmp_dir/good.json"
grep -Fq 'rustc -C instrument-coverage' "$tmp_dir/good.json"
grep -Fq 'llvm-profdata merge -sparse' "$tmp_dir/good.json"
grep -Fq 'export --format=text' "$tmp_dir/good.json"
python3 - <<'PY' "$tmp_dir/good.json"
import json, sys
payload=json.load(open(sys.argv[1]))
assert payload["verdict"] == "PASS"
assert payload["failures"] == []
assert payload["fixture_coverage_smoke_generated"] is True
assert payload["production_coverage_report_generated"] is False
assert payload["coverage_budget_enforced"] is False
assert payload["live_required_context_execution_proven"] is False
assert payload["fixture_line_coverage_percent"] == 100
assert payload["fixture_region_coverage_percent"] == 100
assert payload["profraw_count"] >= 1
assert payload["profile_collision_guard"] == "%m-%p"
assert "llvm-profdata" in payload["profdata_operation"]
assert "TOTAL" in payload["text_report"]
PY

assert_fails_with() {
  local label="$1"
  local expected="$2"
  shift 2
  local out="$tmp_dir/${label}.json"
  set +e
  PYTHONDONTWRITEBYTECODE=1 "$@" --out "$out" > "$tmp_dir/${label}.stdout" 2>&1
  local rc=$?
  set -e
  if [ "$rc" -eq 0 ]; then
    echo "expected $label to fail" >&2
    cat "$out" >&2 || true
    exit 1
  fi
  grep -Fq '"verdict": "FAIL"' "$out"
  grep -Fq "$expected" "$out"
  grep -Fq '"production_coverage_report_generated": false' "$out"
  grep -Fq '"p0_0_green": false' "$out"
  grep -Fq '"phase0_complete": false' "$out"
}

assert_fails_with missing_source 'missing_source_file' \
  python3 "$check" --source "$tmp_dir/no-such.rs"

mkdir -p "$tmp_dir/empty-llvm-bin"
assert_fails_with missing_profdata 'missing_llvm_profdata' \
  env OYA_LLVM_BIN="$tmp_dir/empty-llvm-bin" python3 "$check" --source "$source_fixture"

echo "rust LLVM coverage smoke checks passed"
