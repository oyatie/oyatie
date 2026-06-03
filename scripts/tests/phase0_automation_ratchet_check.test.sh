#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

check="scripts/ci/assert-automation-ratchet.py"
matrix="specs/phase0-automation-matrix.json"
coverage="specs/phase0-automation-coverage-registry.json"

python3 "$check" --json > "$tmp_dir/good.json"
grep -Fq '"verdict": "PASS"' "$tmp_dir/good.json"
grep -Fq '"local_fixture_contract_proven": true' "$tmp_dir/good.json"
grep -Fq '"coverage_registry_local_static_proven": true' "$tmp_dir/good.json"
grep -Fq '"automation_ratchet_live": false' "$tmp_dir/good.json"
grep -Fq '"protected_branch_authority_proven": false' "$tmp_dir/good.json"
grep -Fq '"status_mutation_performed": false' "$tmp_dir/good.json"
grep -Fq '"p0_0_green": false' "$tmp_dir/good.json"
grep -Fq '"phase0_complete": false' "$tmp_dir/good.json"
grep -Fq 'TC-0.16-BAD-oya-cli-authority' "$tmp_dir/good.json"
python3 - <<'PY' "$tmp_dir/good.json"
import json, sys
data = json.load(open(sys.argv[1]))
assert data["coverage_registry_summary"]["unmapped_row_ids"] == []
assert data["coverage_registry_summary"]["missing_mapped_row_ids"] == []
assert data["matrix_summary"]["violations"] == []
PY

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

python3 - <<'PY' "$matrix" "$tmp_dir/bad-missing-required-row.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["required_seed_row_ids"] = [row_id for row_id in data["required_seed_row_ids"] if row_id != "AC-0.16-automation-ratchet"]
data["seed_rows"] = [row for row in data["seed_rows"] if row.get("id") != "AC-0.16-automation-ratchet"]
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_missing_required_row 'missing_required_row_id' --matrix "$tmp_dir/bad-missing-required-row.json" --coverage-registry "$coverage"

python3 - <<'PY' "$matrix" "$tmp_dir/bad-duplicate-unknown-missing.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
rows = data["seed_rows"]
rows[1]["id"] = rows[0]["id"]
rows[1]["classification"] = "automated_some_day"
rows[1]["owner"] = ""
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_duplicate_unknown_missing 'duplicate_row_id' --matrix "$tmp_dir/bad-duplicate-unknown-missing.json" --coverage-registry "$coverage"
assert_fails_with bad_duplicate_unknown_missing_unknown 'unknown_classification' --matrix "$tmp_dir/bad-duplicate-unknown-missing.json" --coverage-registry "$coverage"
assert_fails_with bad_duplicate_unknown_missing_field 'missing_or_empty_required_field' --matrix "$tmp_dir/bad-duplicate-unknown-missing.json" --coverage-registry "$coverage"

python3 - <<'PY' "$matrix" "$tmp_dir/bad-oya-cli-authority.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["seed_rows"][0]["target_gate_or_controller"] = "oya gate run-all --ci-required"
data["seed_rows"][0]["no_new_oya_cli_surface"] = False
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_oya_cli_authority 'blocking_invariant_mapped_to_oya_cli' --matrix "$tmp_dir/bad-oya-cli-authority.json" --coverage-registry "$coverage"

python3 - <<'PY' "$matrix" "$tmp_dir/bad-requirement-oya-authority.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["seed_rows"][0]["requirement"] = "Protected branch required context is satisfied by oya gate run-all --ci-required."
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_requirement_oya_authority 'blocking_invariant_mapped_to_oya_cli' --matrix "$tmp_dir/bad-requirement-oya-authority.json" --coverage-registry "$coverage"

python3 - <<'PY' "$matrix" "$tmp_dir/bad-source-artifact-oya-authority.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["seed_rows"][0]["source_artifact"] = "oya gate run-all --ci-required"
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_source_artifact_oya_authority 'blocking_invariant_mapped_to_oya_cli' --matrix "$tmp_dir/bad-source-artifact-oya-authority.json" --coverage-registry "$coverage"

python3 - <<'PY' "$coverage" "$tmp_dir/bad-coverage-note-oya-authority.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["coverage_subjects"][0]["coverage_note"] = "Operator may use oya gate run-all --ci-required as required-context evidence."
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_coverage_note_oya_authority 'blocking_invariant_mapped_to_oya_cli' --matrix "$matrix" --coverage-registry "$tmp_dir/bad-coverage-note-oya-authority.json"

python3 - <<'PY' "$matrix" "$tmp_dir/bad-human-judgment-no-reason.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
for row in data["seed_rows"]:
    if row.get("id") == "PROCESS-reviewer-multispectrum-evidence":
        row.pop("human_judgment_reason", None)
        row["enforceable_or_automatable"] = False
        break
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_human_judgment_no_reason 'human_judgment_missing_irreducible_reason' --matrix "$tmp_dir/bad-human-judgment-no-reason.json" --coverage-registry "$coverage"

python3 - <<'PY' "$coverage" "$tmp_dir/bad-coverage-unmapped-row.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
for subject in data["coverage_subjects"]:
    if subject["id"] == "AC-0.0":
        subject["mapped_row_ids"] = [row for row in subject["mapped_row_ids"] if row != "AC-0.0-structured-result-bundle"]
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_coverage_unmapped_row 'coverage_row_unmapped' --matrix "$matrix" --coverage-registry "$tmp_dir/bad-coverage-unmapped-row.json"

python3 - <<'PY' "$coverage" "$tmp_dir/bad-coverage-unknown-row.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["coverage_subjects"][0]["mapped_row_ids"].append("MISSING-row-id")
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_coverage_unknown_row 'coverage_mapped_row_missing' --matrix "$matrix" --coverage-registry "$tmp_dir/bad-coverage-unknown-row.json"

python3 - <<'PY' "$coverage" "$tmp_dir/bad-coverage-green-claim.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["claim_boundary"]["p0_0_green"] = True
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_coverage_green_claim 'green_claim_boundary_without_live_authority' --matrix "$matrix" --coverage-registry "$tmp_dir/bad-coverage-green-claim.json"

python3 - <<'PY' "specs/fixtures/phase0-automation-ratchet/tc-0.16-bad-oya-cli-authority.json" "$tmp_dir/bad-oya-fixture-made-good.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
row = data["rows"][0]
row["target_gate_or_controller"] = "cloud-ci-automation-ratchet"
row["evidence_path"] = "specs/phase0-automation-matrix.json"
row["no_new_oya_cli_surface"] = True
data["expected_violations"] = ["blocking_invariant_mapped_to_oya_cli"]
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_oya_fixture_made_good 'RED automation-ratchet fixture must produce violations' --matrix "$matrix" --coverage-registry "$coverage" --fixture "$tmp_dir/bad-oya-fixture-made-good.json"

echo "phase0 automation-ratchet fixture checks passed"
