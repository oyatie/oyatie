#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

check="scripts/ci/assert-trusted-target-inventory.py"
matrix="specs/phase0-automation-matrix.json"
coverage="specs/phase0-automation-coverage-registry.json"
schema="specs/phase0-trusted-target-inventory-schema.json"
good="specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.1a-good-trusted-target-inventory.json"
bad="specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.1a-bad-candidate-sourced-target-inventory.json"


python3 - <<'PY' "$matrix" "$coverage"
import json
import sys

matrix = json.load(open(sys.argv[1]))
coverage = json.load(open(sys.argv[2]))
rows = {row.get("id"): row for row in matrix.get("seed_rows", [])}
subjects = {subject.get("id"): subject for subject in coverage.get("coverage_subjects", [])}


def expect(condition, message):
    if not condition:
        raise SystemExit(message)


row = rows.get("AC-0.0-trusted-target-inventory")
expect(row is not None, "AC-0.0 trusted target-inventory row missing")
target = row.get("target_gate_or_controller", "")
expect("cloud-ci/oya-ci trusted target inventory controller" in target, "AC-0.0 trusted target row must preserve live cloud-ci/oya-ci controller")
expect("//:phase0-trusted-target-inventory-check" in target, "AC-0.0 trusted target row must name local Buck2 target")
expect(
    row.get("verification_command") == "buck2 build //:phase0-trusted-target-inventory-check",
    "AC-0.0 trusted target row must record Buck2 local verification command",
)
claim_boundary = row.get("claim_boundary", "")
expect("not live controller target authority" in claim_boundary, "AC-0.0 trusted target claim boundary must preserve live-controller non-claim")
expect("not P0.0 green" in claim_boundary, "AC-0.0 trusted target claim boundary must preserve P0.0 non-claim")
expect(row.get("no_new_oya_cli_surface") is True, "AC-0.0 trusted target inventory must not add an oya CLI surface")

subject = subjects.get("AC-0.0")
expect(subject is not None, "AC-0.0 coverage subject missing")
expect("AC-0.0-trusted-target-inventory" in subject.get("mapped_row_ids", []), "AC-0.0 coverage must map trusted target row")
commands = subject.get("verification_commands", {})
expect(
    commands.get("AC-0.0-trusted-target-inventory") == "buck2 build //:phase0-trusted-target-inventory-check",
    "AC-0.0 coverage subject must record trusted-target Buck2 local verification command",
)
coverage_note = subject.get("coverage_note", "")
expect("trusted target inventory" in coverage_note, "AC-0.0 coverage note must name trusted target inventory")
expect("not P0.0 green" in coverage_note, "AC-0.0 coverage note must preserve P0.0 non-claim")
PY

python3 "$check" --json > "$tmp_dir/good.json"
grep -Fq '"verdict": "PASS"' "$tmp_dir/good.json"
grep -Fq '"local_fixture_contract_proven": true' "$tmp_dir/good.json"
grep -Fq '"candidate_pr_bytes_are_data_only_locally_proven": true' "$tmp_dir/good.json"
grep -Fq '"trusted_target_inventory_live_authority_proven": false' "$tmp_dir/good.json"
grep -Fq '"trusted_controller_inventory_live": false' "$tmp_dir/good.json"
grep -Fq '"live_required_context_execution_proven": false' "$tmp_dir/good.json"
grep -Fq '"p0_0_green": false' "$tmp_dir/good.json"
grep -Fq '"phase0_complete": false' "$tmp_dir/good.json"

assert_fails_with() {
  local label="$1"
  local expected="$2"
  shift 2
  local out="$tmp_dir/${label}.json"
  set +e
  python3 "$check" "$@" --json > "$out" 2>&1
  local status=$?
  set -e
  if [ "$status" -eq 0 ]; then
    echo "expected $label to fail" >&2
    cat "$out" >&2
    exit 1
  fi
  grep -Fq '"verdict": "FAIL"' "$out"
  grep -Fq "$expected" "$out"
  grep -Fq '"p0_0_green": false' "$out"
  grep -Fq '"phase0_complete": false' "$out"
}

python3 - <<'PY' "$schema" "$tmp_dir/bad-schema-missing-inventory-source.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["required"] = [field for field in data["required"] if field != "inventory_source"]
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_schema_missing_inventory_source 'schema.required missing inventory_source' --schema "$tmp_dir/bad-schema-missing-inventory-source.json" --good-fixture "$good" --bad-fixture "$bad"

python3 - <<'PY' "$schema" "$tmp_dir/bad-schema-extra-inventory-source.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["properties"]["inventory_source"]["enum"].append("candidate_supplied_controller_state")
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_schema_extra_inventory_source 'schema.inventory_source enum must be exactly' --schema "$tmp_dir/bad-schema-extra-inventory-source.json" --good-fixture "$good" --bad-fixture "$bad"

python3 - <<'PY' "$good" "$tmp_dir/bad-good-fixture-extra-top-level-field.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["candidate_discovered_targets"] = ["root//:candidate-owned"]
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_good_fixture_extra_top_level_field 'unexpected top-level fields' --schema "$schema" --good-fixture "$tmp_dir/bad-good-fixture-extra-top-level-field.json" --bad-fixture "$bad"

python3 - <<'PY' "$good" "$tmp_dir/bad-good-fixture-short-candidate-sha.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["candidate_sha"] = "abc123"
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_good_fixture_short_candidate_sha 'candidate_sha must be a 40-character hexadecimal SHA' --schema "$schema" --good-fixture "$tmp_dir/bad-good-fixture-short-candidate-sha.json" --bad-fixture "$bad"

python3 - <<'PY' "$good" "$tmp_dir/bad-good-fixture-candidate-source.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["inventory_source"] = "candidate_pr_bytes"
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_good_fixture_candidate_source 'target_inventory_not_trusted' --schema "$schema" --good-fixture "$tmp_dir/bad-good-fixture-candidate-source.json" --bad-fixture "$bad"

python3 - <<'PY' "$good" "$tmp_dir/bad-good-fixture-malformed-target.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["test_targets"] = ["not-a-buck2-target"]
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_good_fixture_malformed_target 'malformed_buck2_target' --schema "$schema" --good-fixture "$tmp_dir/bad-good-fixture-malformed-target.json" --bad-fixture "$bad"

python3 - <<'PY' "$good" "$tmp_dir/bad-good-fixture-false-green-boundary.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["claim_boundary"]["p0_0_green"] = True
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_good_fixture_false_green_boundary 'green_claim_boundary_without_live_authority' --schema "$schema" --good-fixture "$tmp_dir/bad-good-fixture-false-green-boundary.json" --bad-fixture "$bad"

python3 - <<'PY' "$bad" "$tmp_dir/bad-red-fixture-missing-expected-trusted-target-violation.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["expected_violations"] = [v for v in data["expected_violations"] if v != "candidate_can_author_target_inventory"]
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_red_fixture_missing_expected_violation 'RED fixture expected_violations must include all trusted-target violation classes' --schema "$schema" --good-fixture "$good" --bad-fixture "$tmp_dir/bad-red-fixture-missing-expected-trusted-target-violation.json"

echo "phase0 trusted target-inventory fixture checks passed"
