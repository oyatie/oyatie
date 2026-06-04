#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

check="scripts/ci/assert-service-root-classifier.py"
inventory="specs/service-inventory.json"
packets="specs/phase0-structural-packets.json"
matrix="specs/phase0-automation-matrix.json"
coverage="specs/phase0-automation-coverage-registry.json"
good_fixture="specs/fixtures/phase0-service-root-classifier/tc-service-root-good-seed.json"
red_fixture="specs/fixtures/phase0-service-root-classifier/tc-service-root-bad-layout-sprawl.json"

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
    (candidate for candidate in matrix["seed_rows"] if candidate.get("id") == "AC-0.1-service-inventory"),
    None,
)
expect(row is not None, "AC-0.1 service-inventory row missing")
expect(
    row.get("target_gate_or_controller") == "//:service-root-classifier-check",
    "AC-0.1 service-inventory row must map to exact Buck2 target",
)
expect(
    row.get("verification_command") == "buck2 build //:service-root-classifier-check",
    "AC-0.1 service-inventory row must record exact verification command",
)
claim_boundary = row.get("claim_boundary", "")
expect(
    "full nested crate coverage" in claim_boundary,
    "AC-0.1 service-inventory row must preserve nested-coverage non-claim",
)
expect(
    "not live required-context authority" in claim_boundary,
    "AC-0.1 service-inventory row must preserve live-authority non-claim",
)
expect(
    "post-migration pure split" in claim_boundary,
    "AC-0.1 service-inventory row must preserve post-migration non-claim",
)
expect(
    row.get("no_new_oya_cli_surface") is True,
    "AC-0.1 service-inventory row must refuse new oya CLI authority",
)

subject = next(
    (candidate for candidate in coverage["coverage_subjects"] if candidate.get("id") == "AC-0.1"),
    None,
)
expect(subject is not None, "AC-0.1 coverage subject missing")
expect(
    subject.get("verification_command") == "buck2 build //:service-root-classifier-check",
    "AC-0.1 coverage subject must record exact verification command",
)
coverage_note = subject.get("coverage_note", "")
expect(
    "//:service-root-classifier-check" in coverage_note,
    "AC-0.1 coverage subject must name the exact Buck2 target",
)
expect(
    "full nested crate coverage" in coverage_note,
    "AC-0.1 coverage subject must preserve nested-coverage non-claim",
)
expect(
    "live required-context authority false" in coverage_note,
    "AC-0.1 coverage subject must preserve live-authority non-claim",
)

for row_id, full_index_claim in [
    ("P0.6-pack-root-classifier", "authorized shared roots"),
    ("AC-0.7-service-layout-sprawl", "post-migration pure split"),
]:
    row = next((candidate for candidate in matrix["seed_rows"] if candidate.get("id") == row_id), None)
    expect(row is not None, f"{row_id} row missing")
    expect(
        row.get("target_gate_or_controller") == "//:service-root-classifier-check",
        f"{row_id} row must map to exact Buck2 target",
    )
    expect(
        row.get("verification_command") == "buck2 build //:service-root-classifier-check",
        f"{row_id} row must record exact verification command",
    )
    claim_boundary = row.get("claim_boundary", "")
    expect(
        full_index_claim in claim_boundary,
        f"{row_id} row must preserve classifier-specific non-claim",
    )
    expect(
        "not live required-context authority" in claim_boundary,
        f"{row_id} row must preserve live-authority non-claim",
    )
    expect(
        row.get("no_new_oya_cli_surface") is True,
        f"{row_id} row must refuse new oya CLI authority",
    )

for subject_id, row_id, coverage_claim in [
    ("P0.6-pack-root-classifier", "P0.6-pack-root-classifier", "authorized shared roots"),
    ("AC-0.7", "AC-0.7-service-layout-sprawl", "post-migration pure split remains false"),
]:
    subject = next((candidate for candidate in coverage["coverage_subjects"] if candidate.get("id") == subject_id), None)
    expect(subject is not None, f"{subject_id} coverage subject missing")
    expect(
        row_id in subject.get("mapped_row_ids", []),
        f"{subject_id} coverage subject must map to {row_id}",
    )
    expect(
        subject.get("verification_command") == "buck2 build //:service-root-classifier-check",
        f"{subject_id} coverage subject must record exact verification command",
    )
    coverage_note = subject.get("coverage_note", "")
    expect(
        "//:service-root-classifier-check" in coverage_note,
        f"{subject_id} coverage subject must name the exact Buck2 target",
    )
    expect(
        coverage_claim in coverage_note,
        f"{subject_id} coverage subject must preserve classifier-specific non-claim",
    )
    expect(
        "live required-context authority" in coverage_note,
        f"{subject_id} coverage subject must preserve live-authority non-claim",
    )
PY

PYTHONDONTWRITEBYTECODE=1 python3 "$check" --inventory "$inventory" --packets "$packets" --json > "$tmp_dir/good.json"
grep -Fq '"verdict": "PASS"' "$tmp_dir/good.json"
grep -Fq '"service_inventory_published": true' "$tmp_dir/good.json"
grep -Fq '"service_root_classifier_measured": true' "$tmp_dir/good.json"
grep -Fq '"closed_world_root_count": 8' "$tmp_dir/good.json"
grep -Fq '"fixture_count": 8' "$tmp_dir/good.json"
grep -Fq '"expected_green_fixture_count": 1' "$tmp_dir/good.json"
grep -Fq '"expected_red_fixture_count": 7' "$tmp_dir/good.json"
grep -Fq 'service_inventory_entry_missing' "$tmp_dir/good.json"
grep -Fq 'service_layout_sprawl' "$tmp_dir/good.json"
grep -Fq 'service_root_outside_closed_world' "$tmp_dir/good.json"
grep -Fq 'retired_real_token_live_field' "$tmp_dir/good.json"
grep -Fq 'structural_packet_missing_required_family' "$tmp_dir/good.json"
grep -Fq 'duplicate_service_across_roots' "$tmp_dir/good.json"
grep -Fq 'underscore_crate_name' "$tmp_dir/good.json"
grep -Fq '"full_service_inventory_coverage_proven": false' "$tmp_dir/good.json"
grep -Fq '"post_migration_pure_split_proven": false' "$tmp_dir/good.json"
grep -Fq '"p0_0_green": false' "$tmp_dir/good.json"
grep -Fq '"phase0_complete": false' "$tmp_dir/good.json"
python3 - <<'PY' "$tmp_dir/good.json"
import json, sys
payload=json.load(open(sys.argv[1]))
assert payload["verdict"] == "PASS"
assert payload["inventory_entry_count"] == payload["observed_direct_child_dir_count"]
assert payload["inventory_entry_count"] >= 250
assert payload["structural_packet_count"] >= 6
assert payload["fixture_count"] == payload["expected_green_fixture_count"] + payload["expected_red_fixture_count"]
assert payload["expected_red_fixture_count"] >= 7
assert payload["full_service_inventory_coverage_proven"] is False
assert payload["post_migration_pure_split_proven"] is False
assert payload["structural_shards_executed"] is False
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

python3 - <<'PY' "$inventory" "$tmp_dir/bad-p0-green.json"
import json, sys
src, dst = sys.argv[1:]
data=json.load(open(src))
data["claim_boundary"]["p0_0_green"] = True
json.dump(data, open(dst, "w"), indent=2)
PY
assert_fails_with bad_p0_green 'forbidden_true_or_missing_claim_p0_0_green' --inventory "$tmp_dir/bad-p0-green.json" --packets "$packets"

python3 - <<'PY' "$inventory" "$tmp_dir/bad-missing-entry.json"
import json, sys
src, dst = sys.argv[1:]
data=json.load(open(src))
removed=data["inventory_entries"][0]["source_path"]
data["inventory_entries"] = [entry for entry in data["inventory_entries"] if entry.get("source_path") != removed]
json.dump(data, open(dst, "w"), indent=2)
PY
assert_fails_with bad_missing_entry 'service_inventory_entry_missing' --inventory "$tmp_dir/bad-missing-entry.json" --packets "$packets"

python3 - <<'PY' "$packets" "$tmp_dir/bad-missing-family.json"
import json, sys
src, dst = sys.argv[1:]
data=json.load(open(src))
data["structural_packets"] = [packet for packet in data["structural_packets"] if not packet.get("packet_id", "").startswith("P0.6d-BNF-")]
json.dump(data, open(dst, "w"), indent=2)
PY
assert_fails_with bad_missing_family 'structural_packet_missing_required_family' --inventory "$inventory" --packets "$tmp_dir/bad-missing-family.json"

python3 - <<'PY' "$red_fixture" "$tmp_dir/bad-red-made-clean.json"
import json, sys
src, dst = sys.argv[1:]
data=json.load(open(src))
data["candidate_paths"][0]["path"] = "oya/payments"
data["candidate_paths"][0]["crate_name"] = "payments"
data["inventory_entry_paths"] = ["oya/payments"]
json.dump(data, open(dst, "w"), indent=2)
PY
assert_fails_with bad_red_made_clean 'RED service-root fixture must produce violations' --inventory "$inventory" --packets "$packets" --fixture "$tmp_dir/bad-red-made-clean.json"

python3 - <<'PY' "$good_fixture" "$tmp_dir/bad-good-sprawl-real.json"
import json, sys
src, dst = sys.argv[1:]
data=json.load(open(src))
data["candidate_paths"][0]["path"] = "platforms/accounting"
data["inventory_entry_paths"][0] = "platforms/accounting"
data["live_status_fields"]["maturity_status"] = "REAL"
json.dump(data, open(dst, "w"), indent=2)
PY
assert_fails_with bad_good_sprawl_real 'GREEN service-root fixture produced violations' --inventory "$inventory" --packets "$packets" --fixture "$tmp_dir/bad-good-sprawl-real.json"

echo "service-root classifier checks passed"
