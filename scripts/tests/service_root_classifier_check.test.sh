#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

check="scripts/ci/assert-service-root-classifier.py"
inventory="specs/service-inventory.json"
packets="specs/phase0-structural-packets.json"
good_fixture="specs/fixtures/phase0-service-root-classifier/tc-service-root-good-seed.json"
red_fixture="specs/fixtures/phase0-service-root-classifier/tc-service-root-bad-layout-sprawl.json"

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
