#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

check="scripts/ci/assert-status-enum-drift.py"
registry="specs/status-enum-registry.json"
matrix="specs/phase0-automation-matrix.json"
coverage="specs/phase0-automation-coverage-registry.json"
good_fixture="specs/fixtures/phase0-status-enum-drift/tc-status-enum-good-aligned.json"
red_fixture="specs/fixtures/phase0-status-enum-drift/tc-status-enum-bad-status-drift.json"

python3 - <<'PY' "$matrix" "$coverage"
import json
import sys

matrix_path, coverage_path = sys.argv[1:]
matrix = json.load(open(matrix_path))
coverage = json.load(open(coverage_path))


def expect(condition, message):
    if not condition:
        raise SystemExit(message)


row = next(
    (candidate for candidate in matrix["seed_rows"] if candidate.get("id") == "AC-0.2-status-enum-drift"),
    None,
)
expect(row is not None, "AC-0.2 status-enum drift row missing")
expect(
    row.get("target_gate_or_controller") == "//:status-enum-drift-check",
    "AC-0.2 status-enum drift row must map to exact Buck2 target",
)
expect(
    row.get("verification_command") == "buck2 build //:status-enum-drift-check",
    "AC-0.2 status-enum drift row must record exact verification command",
)
claim_boundary = row.get("claim_boundary", "")
expect(
    "full manifest/PRD conformance" in claim_boundary,
    "AC-0.2 status-enum drift row must preserve full-manifest non-claim",
)
expect(
    "not live required-context authority" in claim_boundary,
    "AC-0.2 status-enum drift row must preserve live-authority non-claim",
)
expect(
    row.get("no_new_oya_cli_surface") is True,
    "AC-0.2 status-enum drift row must refuse new oya CLI authority",
)

subject = next(
    (candidate for candidate in coverage["coverage_subjects"] if candidate.get("id") == "AC-0.2"),
    None,
)
expect(subject is not None, "AC-0.2 coverage subject missing")
expect(
    subject.get("verification_command") == "buck2 build //:status-enum-drift-check",
    "AC-0.2 coverage subject must record exact verification command",
)
coverage_note = subject.get("coverage_note", "")
expect(
    "//:status-enum-drift-check" in coverage_note,
    "AC-0.2 coverage subject must name the exact Buck2 target",
)
expect(
    "full manifest/PRD conformance" in coverage_note,
    "AC-0.2 coverage subject must preserve full-manifest non-claim",
)
expect(
    "live required-context authority remain false" in coverage_note,
    "AC-0.2 coverage subject must preserve live-authority non-claim",
)
PY

PYTHONDONTWRITEBYTECODE=1 python3 "$check" --registry "$registry" --json > "$tmp_dir/good.json"
grep -Fq '"verdict": "PASS"' "$tmp_dir/good.json"
grep -Fq '"status_enum_registry_published": true' "$tmp_dir/good.json"
grep -Fq '"status_drift_fixture_contract_measured": true' "$tmp_dir/good.json"
grep -Fq '"axis_count": 3' "$tmp_dir/good.json"
grep -Fq '"fixture_count": 5' "$tmp_dir/good.json"
grep -Fq '"expected_green_fixture_count": 1' "$tmp_dir/good.json"
grep -Fq '"expected_red_fixture_count": 4' "$tmp_dir/good.json"
grep -Fq 'invalid_status_enum_value' "$tmp_dir/good.json"
grep -Fq 'retired_real_token_live_field' "$tmp_dir/good.json"
grep -Fq 'spec_code_manifest_mismatch' "$tmp_dir/good.json"
grep -Fq 'status_drift_mismatch' "$tmp_dir/good.json"
grep -Fq '"full_manifest_prd_conformance_proven": false' "$tmp_dir/good.json"
grep -Fq '"status_drift_live_gate_proven": false' "$tmp_dir/good.json"
grep -Fq '"p0_0_green": false' "$tmp_dir/good.json"
grep -Fq '"phase0_complete": false' "$tmp_dir/good.json"
python3 - <<'PY' "$tmp_dir/good.json"
import json, sys
payload=json.load(open(sys.argv[1]))
assert payload["verdict"] == "PASS"
assert payload["axis_count"] == 3
assert payload["allowed_value_count"] >= 15
assert payload["fixture_count"] == payload["expected_green_fixture_count"] + payload["expected_red_fixture_count"]
assert payload["expected_red_fixture_count"] >= 4
assert payload["full_manifest_prd_conformance_proven"] is False
assert payload["status_drift_live_gate_proven"] is False
assert payload["p0_0_green"] is False
assert payload["phase0_complete"] is False
assert payload["failures"] == []
PY

assert_fails_with() {
  local label="$1"
  local expected="$2"
  shift 2
  local out="$tmp_dir/${label}.out"
  set +e
  PYTHONDONTWRITEBYTECODE=1 python3 "$check" "$@" --json > "$out" 2>&1
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

python3 - <<'PY' "$registry" "$tmp_dir/bad-p0-green.json"
import json, sys
src, dst = sys.argv[1:]
data=json.load(open(src))
data["claim_boundary"]["p0_0_green"] = True
json.dump(data, open(dst, "w"), indent=2)
PY
assert_fails_with bad_p0_green 'forbidden_true_or_missing_claim_p0_0_green' --registry "$tmp_dir/bad-p0-green.json"

python3 - <<'PY' "$registry" "$tmp_dir/bad-real-allowed.json"
import json, sys
src, dst = sys.argv[1:]
data=json.load(open(src))
data["axes"]["maturity"]["allowed_values"].append("REAL")
json.dump(data, open(dst, "w"), indent=2)
PY
assert_fails_with bad_real_allowed 'retired_real_token_allowed:maturity' --registry "$tmp_dir/bad-real-allowed.json"

python3 - <<'PY' "$red_fixture" "$tmp_dir/bad-red-made-clean.json"
import json, sys
src, dst = sys.argv[1:]
data=json.load(open(src))
data["spec_manifest_pairs"][0]["manifest_status_fields"] = dict(data["spec_manifest_pairs"][0]["spec_status_fields"])
json.dump(data, open(dst, "w"), indent=2)
PY
assert_fails_with bad_red_made_clean 'RED status-enum fixture must produce violations' --registry "$registry" --fixture "$tmp_dir/bad-red-made-clean.json"

python3 - <<'PY' "$good_fixture" "$tmp_dir/bad-good-invalid-status.json"
import json, sys
src, dst = sys.argv[1:]
data=json.load(open(src))
data["status_fields"]["maturity_status"] = "REAL"
json.dump(data, open(dst, "w"), indent=2)
PY
assert_fails_with bad_good_invalid_status 'GREEN status-enum fixture produced violations' --registry "$registry" --fixture "$tmp_dir/bad-good-invalid-status.json"

echo "status-enum drift checks passed"
