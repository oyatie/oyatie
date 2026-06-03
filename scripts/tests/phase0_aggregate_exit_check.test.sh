#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

check="scripts/ci/assert-phase0-aggregate-exit.py"
good="specs/fixtures/phase0-exit-gate/tc-0.12-good-all-subconditions-green.json"
single_false="specs/fixtures/phase0-exit-gate/tc-0.12-bad-single-false-subconditions.json"
missing_required="specs/fixtures/phase0-exit-gate/tc-0.12-bad-missing-required-subcondition.json"
current_red="specs/fixtures/phase0-exit-gate/tc-0.12-current-red-p0-0-live-context-missing.json"

python3 "$check" --json > "$tmp_dir/good.json"
grep -Fq '"verdict": "PASS"' "$tmp_dir/good.json"
grep -Fq '"local_fixture_contract_proven": true' "$tmp_dir/good.json"
grep -Fq '"aggregate_exit_live": false' "$tmp_dir/good.json"
grep -Fq '"live_required_context_execution_proven": false' "$tmp_dir/good.json"
grep -Fq '"p0_0_green": false' "$tmp_dir/good.json"
grep -Fq '"phase0_complete": false' "$tmp_dir/good.json"
grep -Fq '"production_ready": false' "$tmp_dir/good.json"
grep -Fq '"hyperscaler_grade": false' "$tmp_dir/good.json"
python3 - <<'PY' "$tmp_dir/good.json"
import json, sys
x=json.load(open(sys.argv[1]))
assert x["required_subcondition_count"] == 32
assert x["single_false_case_count"] == 32
assert len(x["fixture_results"]) == 4
assert x["failures"] == []
PY

assert_fails_with() {
  local label="$1"
  local expected="$2"
  shift 2
  local out="$tmp_dir/${label}.json"
  set +e
  python3 "$check" "$@" --json > "$out" 2>&1
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

python3 "$check" --fixture "$current_red" --json > "$tmp_dir/current-red.json"
python3 - <<'PY' "$tmp_dir/current-red.json"
import json, sys
x=json.load(open(sys.argv[1]))
assert x["verdict"] == "PASS"
assert x["fixture_results"][0]["observed_verdict"] == "RED"
assert "false_or_non_true_subcondition" in x["fixture_results"][0]["violations"]
PY
python3 "$check" --fixture "$missing_required" --json > "$tmp_dir/missing-required.json"
python3 - <<'PY' "$tmp_dir/missing-required.json"
import json, sys
x=json.load(open(sys.argv[1]))
assert x["verdict"] == "PASS"
assert x["fixture_results"][0]["observed_verdict"] == "RED"
assert "missing_required_subcondition" in x["fixture_results"][0]["violations"]
PY

python3 - <<'PY' "$good" "$tmp_dir/bad-missing-ac00.json"
import json, sys
src, dst = sys.argv[1:]
data=json.load(open(src))
del data["subconditions"]["AC-0.0_green"]
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_missing_ac00 'missing_required_subcondition' --fixture "$tmp_dir/bad-missing-ac00.json"

python3 - <<'PY' "$good" "$tmp_dir/bad-live-context-false.json"
import json, sys
src, dst = sys.argv[1:]
data=json.load(open(src))
data["subconditions"]["p0_0_full_required_context_proven"] = False
data["claim_boundary"]={"p0_0_green": False, "phase0_complete": False}
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_live_context_false 'false_or_non_true_subcondition' --fixture "$tmp_dir/bad-live-context-false.json"

python3 - <<'PY' "$good" "$tmp_dir/bad-unknown-subcondition.json"
import json, sys
src, dst = sys.argv[1:]
data=json.load(open(src))
data["subconditions"]["unregistered_phase0_shortcut"] = True
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_unknown_subcondition 'unknown_subcondition' --fixture "$tmp_dir/bad-unknown-subcondition.json"

python3 - <<'PY' "$single_false" "$tmp_dir/bad-single-false-missing-case.json"
import json, sys
src, dst = sys.argv[1:]
data=json.load(open(src))
data["cases"] = [case for case in data["cases"] if case.get("forced_false") != "AC-0.17_claim_ceiling_green"]
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_single_false_missing_case 'missing_case_for_required_subcondition' --fixture "$tmp_dir/bad-single-false-missing-case.json"

python3 - <<'PY' "$single_false" "$tmp_dir/bad-single-false-multi-false.json"
import json, sys
src, dst = sys.argv[1:]
data=json.load(open(src))
data["cases"][0]["case_id"] = "BAD-single-false-case-hides-second-false"
data["cases"][0]["subconditions"]["AC-0.17_claim_ceiling_green"] = False
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_single_false_multi_false 'single_false_case_not_exactly_one_false_subcondition' --fixture "$tmp_dir/bad-single-false-multi-false.json"

python3 - <<'PY' "$current_red" "$tmp_dir/bad-red-claims-green.json"
import json, sys
src, dst = sys.argv[1:]
data=json.load(open(src))
data["claim_boundary"]={"p0_0_green": True, "phase0_complete": True}
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_red_claims_green 'fixture_claims_current_phase0_green' --fixture "$tmp_dir/bad-red-claims-green.json"

echo "phase0 aggregate-exit fixture checks passed"
