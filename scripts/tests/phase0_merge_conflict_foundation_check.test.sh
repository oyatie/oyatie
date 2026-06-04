#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

check="scripts/ci/assert-phase0-merge-conflict-foundation.py"
registry="specs/generated-artifact-registry.json"
matrix="specs/phase0-automation-matrix.json"
coverage="specs/phase0-automation-coverage-registry.json"

python3 - <<'PY' "$matrix" "$coverage"
import json
import sys

matrix = json.load(open(sys.argv[1]))
coverage = json.load(open(sys.argv[2]))
rows = {row.get("id"): row for row in matrix.get("seed_rows", [])}
row = rows.get("AC-0.15-merge-conflict-foundation")

def expect(condition, message):
    if not condition:
        raise SystemExit(message)

expect(row is not None, "AC-0.15-merge-conflict-foundation row missing")
expect(row.get("target_gate_or_controller") == "//:phase0-merge-conflict-foundation-check", "AC-0.15 target must map directly to //:phase0-merge-conflict-foundation-check")
expect(row.get("verification_command") == "buck2 build //:phase0-merge-conflict-foundation-check", "AC-0.15 verification command must be Buck2-native")
expect("Phase-1 Tide batching" in row.get("claim_boundary", ""), "AC-0.15 claim boundary must preserve Phase-1 Tide batching non-claim")
expect(row.get("no_new_oya_cli_surface") is True, "AC-0.15 must not add an oya CLI surface")

p09 = rows.get("P0.9-generated-artifact-registry")
expect(p09 is not None, "P0.9-generated-artifact-registry row missing")
expect(p09.get("target_gate_or_controller") == "//:phase0-merge-conflict-foundation-check", "P0.9 target must map directly to //:phase0-merge-conflict-foundation-check")
expect(p09.get("verification_command") == "buck2 build //:phase0-merge-conflict-foundation-check", "P0.9 verification command must be Buck2-native")
expect("full-repo generated-artifact drift coverage" in p09.get("claim_boundary", ""), "P0.9 claim boundary must preserve full-repo drift non-claim")
expect("not live required-context authority" in p09.get("claim_boundary", ""), "P0.9 claim boundary must preserve live-authority non-claim")
expect(p09.get("no_new_oya_cli_surface") is True, "P0.9 must not add an oya CLI surface")

coverage_subjects = {subject.get("id"): subject for subject in coverage.get("coverage_subjects", [])}
p09_subject = coverage_subjects.get("P0.9-generated-artifact-registry")
expect(p09_subject is not None, "P0.9 coverage subject missing")
expect("P0.9-generated-artifact-registry" in p09_subject.get("mapped_row_ids", []), "P0.9 coverage subject must map to P0.9 row")
expect(p09_subject.get("verification_command") == "buck2 build //:phase0-merge-conflict-foundation-check", "P0.9 coverage command must be Buck2-native")
expect("//:phase0-merge-conflict-foundation-check" in p09_subject.get("coverage_note", ""), "P0.9 coverage note must name exact Buck2 target")
expect("full-repo generated-artifact coverage remains false" in p09_subject.get("coverage_note", ""), "P0.9 coverage note must preserve full-repo coverage non-claim")
expect("live required-context authority" in p09_subject.get("coverage_note", ""), "P0.9 coverage note must preserve live-authority non-claim")
PY

PYTHONDONTWRITEBYTECODE=1 python3 "$check" --registry "$registry" --json > "$tmp_dir/good.json"
grep -Fq '"verdict": "PASS"' "$tmp_dir/good.json"
grep -Fq '"generated_artifact_registry_published": true' "$tmp_dir/good.json"
grep -Fq '"merge_tree_fixture_contract_measured": true' "$tmp_dir/good.json"
grep -Fq '"registered_artifact_count": 1' "$tmp_dir/good.json"
grep -Fq '"fixture_count": 5' "$tmp_dir/good.json"
grep -Fq '"expected_green_fixture_count": 1' "$tmp_dir/good.json"
grep -Fq '"expected_red_fixture_count": 4' "$tmp_dir/good.json"
grep -Fq 'TC-0.15-GOOD-clean-merge-tree-generated-registry' "$tmp_dir/good.json"
grep -Fq 'TC-0.15-BAD-path-overlap' "$tmp_dir/good.json"
grep -Fq 'path_overlap_without_review' "$tmp_dir/good.json"
grep -Fq 'generated_artifact_missing_registry' "$tmp_dir/good.json"
grep -Fq 'phase1_tide_batched_projection_overclaim' "$tmp_dir/good.json"
grep -Fq 'merge_tree_conflict' "$tmp_dir/good.json"
grep -Fq '"live_required_context_execution_proven": false' "$tmp_dir/good.json"
grep -Fq '"phase1_tide_batching_claimed": false' "$tmp_dir/good.json"
grep -Fq '"full_repo_generated_artifact_coverage_proven": false' "$tmp_dir/good.json"
grep -Fq '"p0_0_green": false' "$tmp_dir/good.json"
grep -Fq '"phase0_complete": false' "$tmp_dir/good.json"
python3 - <<'PY' "$tmp_dir/good.json"
import json, sys
payload=json.load(open(sys.argv[1]))
assert payload["verdict"] == "PASS"
assert payload["registered_artifact_count"] >= 1
assert payload["taxonomy_count"] >= 7
assert payload["fixture_count"] == payload["expected_green_fixture_count"] + payload["expected_red_fixture_count"]
assert payload["expected_red_fixture_count"] >= 4
assert payload["failures"] == []
assert payload["phase1_tide_batching_claimed"] is False
assert payload["full_repo_generated_artifact_coverage_proven"] is False
assert payload["p0_0_green"] is False
assert payload["phase0_complete"] is False
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
  grep -Fq '"phase1_tide_batching_claimed": false' "$out"
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

python3 - <<'PY' "$registry" "$tmp_dir/bad-missing-target.json"
import json, sys
src, dst = sys.argv[1:]
data=json.load(open(src))
data["automated_chain"] = [item for item in data["automated_chain"] if "phase0-merge-conflict-foundation-check" not in item]
json.dump(data, open(dst, "w"), indent=2)
PY
assert_fails_with bad_missing_target 'missing_automated_chain_token://:phase0-merge-conflict-foundation-check' --registry "$tmp_dir/bad-missing-target.json"

python3 - <<'PY' "$registry" "$tmp_dir/bad-missing-source.json"
import json, sys
src, dst = sys.argv[1:]
data=json.load(open(src))
data["registered_artifacts"][0]["source_paths"] = ["missing-generated-source.toml"]
json.dump(data, open(dst, "w"), indent=2)
PY
assert_fails_with bad_missing_source 'artifact_source_path_missing_or_invalid' --registry "$tmp_dir/bad-missing-source.json"

python3 - <<'PY' "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-path-overlap.json" "$tmp_dir/bad-red-made-clean.json"
import json, sys
src, dst = sys.argv[1:]
data=json.load(open(src))
data["lanes"][1]["owned_paths"] = ["specs/phase0-automation-coverage-registry.json"]
json.dump(data, open(dst, "w"), indent=2)
PY
assert_fails_with bad_red_made_clean 'RED merge-conflict fixture must produce violations' --registry "$registry" --fixture "$tmp_dir/bad-red-made-clean.json"

python3 - <<'PY' "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-good-clean-merge-tree-generated-registry.json" "$tmp_dir/bad-green-now-conflicts.json"
import json, sys
src, dst = sys.argv[1:]
data=json.load(open(src))
data["merge_tree_simulation"]["result"] = "conflict"
json.dump(data, open(dst, "w"), indent=2)
PY
assert_fails_with bad_green_now_conflicts 'GREEN merge-conflict fixture produced violations' --registry "$registry" --fixture "$tmp_dir/bad-green-now-conflicts.json"

echo "phase0 merge-conflict foundation checks passed"
