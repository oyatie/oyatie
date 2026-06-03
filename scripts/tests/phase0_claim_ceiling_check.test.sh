#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

check="scripts/ci/assert-claim-ceiling.py"
claim_map="specs/phase0-claim-evidence-map.json"
contract="specs/hyperscaler-production-readiness-claim-contract.json"

python3 "$check" --json > "$tmp_dir/good.json"
grep -Fq '"verdict": "PASS"' "$tmp_dir/good.json"
grep -Fq '"local_fixture_contract_proven": true' "$tmp_dir/good.json"
grep -Fq '"claim_ceiling_live": false' "$tmp_dir/good.json"
grep -Fq '"protected_branch_authority_proven": false' "$tmp_dir/good.json"
grep -Fq '"status_mutation_performed": false' "$tmp_dir/good.json"
grep -Fq '"live_required_context_execution_proven": false' "$tmp_dir/good.json"
grep -Fq '"p0_0_green": false' "$tmp_dir/good.json"
grep -Fq '"phase0_complete": false' "$tmp_dir/good.json"
grep -Fq '"production_ready": false' "$tmp_dir/good.json"
grep -Fq '"hyperscaler_grade": false' "$tmp_dir/good.json"
grep -Fq 'TC-0.17-BAD-local-oya-authority-claim' "$tmp_dir/good.json"
python3 - <<'PY' "$tmp_dir/good.json"
import json, sys
data = json.load(open(sys.argv[1]))
assert data["claim_map_summary"]["violations"] == []
assert data["claim_map_summary"]["row_count"] >= 6
assert data["claim_map_summary"]["regulated_vocabulary_count"] >= 20
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

python3 - <<'PY' "$claim_map" "$tmp_dir/bad-missing-claim-row.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["seed_claim_rows"] = []
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_missing_claim_row 'regulated_vocabulary_without_claim_row' --claim-map "$tmp_dir/bad-missing-claim-row.json" --contract "$contract" --text 'The platform is production-ready and secure.'

python3 - <<'PY' "$claim_map" "$tmp_dir/bad-unknown-tier-missing-owner.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
row = data["seed_claim_rows"][0]
row["claim_tier"] = "aspirational_ready"
row["owner"] = ""
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_unknown_tier 'unknown_claim_tier' --claim-map "$tmp_dir/bad-unknown-tier-missing-owner.json" --contract "$contract"
assert_fails_with bad_missing_owner 'missing_or_empty_required_field' --claim-map "$tmp_dir/bad-unknown-tier-missing-owner.json" --contract "$contract"

python3 - <<'PY' "$claim_map" "$tmp_dir/bad-mechanical-local-oya.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
row = data["seed_claim_rows"][1]
row["claim_tier"] = "mechanically_enforced"
row["claim_text"] = "mechanically enforced by oya verify --ci-required"
row["allowed_language_now"] = "mechanically enforced"
row["current_evidence"] = ["local-only command", "legacy oya CLI invocation"]
row["missing_for_next_tier"] = []
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_mechanical_local_oya 'forbidden_local_or_oya_evidence_for_mechanical_claim' --claim-map "$tmp_dir/bad-mechanical-local-oya.json" --contract "$contract"

python3 - <<'PY' "$claim_map" "$tmp_dir/bad-performance-no-budget.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
row = data["seed_claim_rows"][-1]
row["claim_tier"] = "production_ready"
row["allowed_language_now"] = "production-ready"
row["current_evidence"] = ["advisory benchmark planned"]
row["missing_for_next_tier"] = []
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_performance_no_budget 'performance_claim_without_budget_or_measured_result' --claim-map "$tmp_dir/bad-performance-no-budget.json" --contract "$contract"

python3 - <<'PY' "$claim_map" "$tmp_dir/bad-performance-domain-only.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
row = data["seed_claim_rows"][-1]
row["claim_tier"] = "production_ready"
row["allowed_language_now"] = "production-ready"
row["current_evidence"] = ["PERF-CAPACITY", "performance_budget", "p95 budget declared"]
row["missing_for_next_tier"] = []
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_performance_domain_only 'performance_claim_without_budget_or_measured_result' --claim-map "$tmp_dir/bad-performance-domain-only.json" --contract "$contract"

python3 - <<'PY' "$claim_map" "$tmp_dir/bad-production-ready-tier-only.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
row = data["seed_claim_rows"][3]
row["claim_tier"] = "production_ready"
row["claim_text"] = "secure control plane"
row["allowed_language_now"] = "secure"
row["regulated_terms"] = ["secure"]
row["current_evidence"] = ["security review planned"]
row["missing_for_next_tier"] = ["budget plus measured-result evidence missing"]
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_production_ready_tier_only 'performance_claim_without_budget_or_measured_result' --claim-map "$tmp_dir/bad-production-ready-tier-only.json" --contract "$contract"

python3 - <<'PY' "$claim_map" "$tmp_dir/bad-hyperscaler-tier-only.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
row = data["seed_claim_rows"][3]
row["claim_tier"] = "hyperscaler_grade"
row["claim_text"] = "secure control plane"
row["allowed_language_now"] = "secure"
row["regulated_terms"] = ["secure"]
row["current_evidence"] = ["security review planned"]
row["missing_for_next_tier"] = ["budget plus measured-result evidence missing"]
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_hyperscaler_tier_only 'performance_claim_without_budget_or_measured_result' --claim-map "$tmp_dir/bad-hyperscaler-tier-only.json" --contract "$contract"

python3 - <<'PY' "$claim_map" "$tmp_dir/bad-capacity-breakpoint-only.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
row = data["seed_claim_rows"][-1]
row["claim_tier"] = "production_ready"
row["allowed_language_now"] = "production-ready"
row["current_evidence"] = ["capacity breakpoint"]
row["missing_for_next_tier"] = ["budget evidence missing"]
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_capacity_breakpoint_only 'performance_claim_without_budget_or_measured_result' --claim-map "$tmp_dir/bad-capacity-breakpoint-only.json" --contract "$contract"

python3 - <<'PY' "$claim_map" "$tmp_dir/bad-performance-combined-entry.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
row = data["seed_claim_rows"][-1]
row["claim_tier"] = "production_ready"
row["allowed_language_now"] = "production-ready"
row["current_evidence"] = ["p95 budget and load result"]
row["missing_for_next_tier"] = ["separate budget evidence entry and separate measured-result evidence entry required"]
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_performance_combined_entry 'performance_claim_without_budget_or_measured_result' --claim-map "$tmp_dir/bad-performance-combined-entry.json" --contract "$contract"

python3 - <<'PY' "$claim_map" "$tmp_dir/bad-unknown-regulated-term.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["seed_claim_rows"][0]["regulated_terms"] = ["magic-fast"]
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_unknown_regulated_term 'unknown_regulated_term' --claim-map "$tmp_dir/bad-unknown-regulated-term.json" --contract "$contract"

python3 - <<'PY' "specs/fixtures/phase0-claim-ceiling/tc-0.17-bad-ungrounded-production-ready.json" "$tmp_dir/bad-red-fixture-made-good.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["text"] = "Claim ceiling target/non-claim only."
data["claim_rows"] = [{
    "id": "GOOD-repaired-row",
    "artifact": dst,
    "claim_text": "target/non-claim only",
    "claim_tier": "target_non_claim",
    "allowed_language_now": "target/non-claim only",
    "regulated_terms": ["hyperscaler-grade"],
    "current_evidence": ["owner", "phase", "source_decision", "specific_gap_list", "blocking_path_to_next_tier"],
    "missing_for_next_tier": ["live evidence"],
    "owner": "platform-sre"
}]
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_red_fixture_made_good 'RED claim-ceiling fixture must produce violations' --claim-map "$claim_map" --contract "$contract" --fixture "$tmp_dir/bad-red-fixture-made-good.json"

echo "phase0 claim-ceiling fixture checks passed"
