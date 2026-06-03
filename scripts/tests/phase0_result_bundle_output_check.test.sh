#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

check="scripts/ci/assert-result-bundle-output.py"
schema="specs/phase0-ci-enforcement-result-schema.json"
current_red="specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0-current-red-gap-result.json"
false_green="specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.4-bad-result-bundle-false-green.json"

python3 "$check" --json > "$tmp_dir/good.json"
grep -Fq '"verdict": "PASS"' "$tmp_dir/good.json"
grep -Fq '"local_fixture_contract_proven": true' "$tmp_dir/good.json"
grep -Fq '"structured_result_bundle_live": false' "$tmp_dir/good.json"
grep -Fq '"trusted_status_producer_live": false' "$tmp_dir/good.json"
grep -Fq '"protected_branch_authority_proven": false' "$tmp_dir/good.json"
grep -Fq '"status_mutation_performed": false' "$tmp_dir/good.json"
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

python3 - <<'PY' "$schema" "$tmp_dir/bad-schema-missing-producer.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["required"] = [field for field in data["required"] if field != "producer"]
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_schema_missing_producer 'schema.required missing producer' --schema "$tmp_dir/bad-schema-missing-producer.json" --current-red-fixture "$current_red" --false-green-fixture "$false_green"

python3 - <<'PY' "$schema" "$tmp_dir/bad-schema-context-enum.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["properties"]["required_context"]["enum"].append("legacy-oya-verify")
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_schema_context_enum 'schema.required_context enum must be exactly' --schema "$tmp_dir/bad-schema-context-enum.json" --current-red-fixture "$current_red" --false-green-fixture "$false_green"

python3 - <<'PY' "$schema" "$tmp_dir/bad-schema-sha-pattern.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["properties"]["candidate_sha"].pop("pattern", None)
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_schema_sha_pattern 'schema.candidate_sha must require 40 hexadecimal characters' --schema "$tmp_dir/bad-schema-sha-pattern.json" --current-red-fixture "$current_red" --false-green-fixture "$false_green"

python3 - <<'PY' "$current_red" "$tmp_dir/bad-current-red-extra-top-level-field.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["candidate_authored_summary"] = "should fail local schema-shape guard"
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_current_red_extra_top_level_field 'unexpected top-level fields' --schema "$schema" --current-red-fixture "$tmp_dir/bad-current-red-extra-top-level-field.json" --false-green-fixture "$false_green"

python3 - <<'PY' "$current_red" "$tmp_dir/bad-current-red-invalid-sha.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["candidate_sha"] = "not-a-sha"
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_current_red_invalid_sha 'candidate_sha must be a 40-character hexadecimal SHA' --schema "$schema" --current-red-fixture "$tmp_dir/bad-current-red-invalid-sha.json" --false-green-fixture "$false_green"

python3 - <<'PY' "$current_red" "$tmp_dir/bad-current-red-empty-provenance-sources.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["provenance"]["sources"] = []
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_current_red_empty_provenance_sources 'provenance.sources must be a non-empty string array' --schema "$schema" --current-red-fixture "$tmp_dir/bad-current-red-empty-provenance-sources.json" --false-green-fixture "$false_green"

python3 - <<'PY' "$current_red" "$tmp_dir/bad-current-red-claims-green.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["claim_boundary"]["p0_0_green"] = True
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_current_red_claims_green 'current RED result bundle must keep p0_0_green=false and phase0_complete=false' --schema "$schema" --current-red-fixture "$tmp_dir/bad-current-red-claims-green.json" --false-green-fixture "$false_green"

python3 - <<'PY' "$current_red" "$tmp_dir/bad-current-red-live-looking-producer.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["required_context"] = "oya-ci-required"
data["producer"] = {
  "context": "oya-ci-required",
  "kind": "oya-ci-controller",
  "trusted_control_state": True,
  "candidate_bytes_policy": "untrusted_input_only",
  "gate_definition_source": "trusted_dev_or_controller_state"
}
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_current_red_live_looking_producer 'current RED result bundle must expose missing-context, untrusted-producer, candidate-bytes, and candidate-sourced violations' --schema "$schema" --current-red-fixture "$tmp_dir/bad-current-red-live-looking-producer.json" --false-green-fixture "$false_green"

python3 - <<'PY' "$false_green" "$tmp_dir/bad-false-green-boundary-not-exercised.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["claim_boundary"] = {"p0_0_green": False, "phase0_complete": False}
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_false_green_boundary_not_exercised 'false-green result bundle must exercise p0_0_green=true and phase0_complete=true' --schema "$schema" --current-red-fixture "$current_red" --false-green-fixture "$tmp_dir/bad-false-green-boundary-not-exercised.json"

python3 - <<'PY' "$current_red" "$tmp_dir/bad-current-red-empty-fixture-results.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["fixture_results"] = []
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_current_red_empty_fixture_results 'current RED result bundle must remain schema-shaped and non-empty' --schema "$schema" --current-red-fixture "$tmp_dir/bad-current-red-empty-fixture-results.json" --false-green-fixture "$false_green"

python3 - <<'PY' "$false_green" "$tmp_dir/bad-false-green-fixture-result-matches-red.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["fixture_results"][0]["observed_verdict"] = "RED"
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_false_green_fixture_result_matches_red 'false-green result bundle must expose all required false-green violation classes' --schema "$schema" --current-red-fixture "$current_red" --false-green-fixture "$tmp_dir/bad-false-green-fixture-result-matches-red.json"

python3 - <<'PY' "$false_green" "$tmp_dir/bad-false-green-red-expected-nonempty-violations.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["fixture_results"][0]["violations"] = ["missing_cloud_ci_required_context"]
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_false_green_red_expected_nonempty_violations 'false-green result bundle must expose all required false-green violation classes' --schema "$schema" --current-red-fixture "$current_red" --false-green-fixture "$tmp_dir/bad-false-green-red-expected-nonempty-violations.json"

echo "phase0 result-bundle output fixture checks passed"
