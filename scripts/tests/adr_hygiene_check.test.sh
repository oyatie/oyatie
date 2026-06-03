#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

check="scripts/ci/assert-adr-hygiene.py"
registry="specs/adr-hygiene-registry.json"
good_fixture="specs/fixtures/phase0-adr-hygiene/tc-adr-hygiene-good-renumbered-superseded-clean.json"
red_fixture="specs/fixtures/phase0-adr-hygiene/tc-adr-hygiene-bad-duplicate-adr-number.json"

PYTHONDONTWRITEBYTECODE=1 python3 "$check" --registry "$registry" --json > "$tmp_dir/good.json"
grep -Fq '"verdict": "PASS"' "$tmp_dir/good.json"
grep -Fq '"adr_hygiene_registry_published": true' "$tmp_dir/good.json"
grep -Fq '"adr_hygiene_fixture_contract_measured": true' "$tmp_dir/good.json"
grep -Fq '"fixture_count": 4' "$tmp_dir/good.json"
grep -Fq '"expected_green_fixture_count": 1' "$tmp_dir/good.json"
grep -Fq '"expected_red_fixture_count": 3' "$tmp_dir/good.json"
grep -Fq 'duplicate_adr_number' "$tmp_dir/good.json"
grep -Fq 'adr_0511_missing_superseded_by_adr_0513' "$tmp_dir/good.json"
grep -Fq 'superseded_reference_active_doc' "$tmp_dir/good.json"
grep -Fq '"full_adr_index_regenerated": false' "$tmp_dir/good.json"
grep -Fq '"p0_0_green": false' "$tmp_dir/good.json"
grep -Fq '"phase0_complete": false' "$tmp_dir/good.json"
python3 - <<'PY' "$tmp_dir/good.json"
import json, sys
payload=json.load(open(sys.argv[1]))
assert payload["verdict"] == "PASS"
assert payload["fixture_count"] == 4
assert payload["expected_red_fixture_count"] == 3
assert payload["decision_record_count"] >= 300
assert payload["active_doc_scan_count"] >= 10
assert payload["full_adr_index_regenerated"] is False
assert payload["p0_0_green"] is False
assert payload["phase0_complete"] is False
assert payload["failures"] == []
PY

test -f docs/decisions/ADR-0520-kafka-to-pulsar-via-kop.md
test ! -f docs/decisions/ADR-0377-kafka-to-pulsar-via-kop.md
grep -Fq 'id: ADR-0520' docs/decisions/ADR-0520-kafka-to-pulsar-via-kop.md
grep -Fq 'renumbered_from: ADR-0377' docs/decisions/ADR-0520-kafka-to-pulsar-via-kop.md
grep -Fq 'superseded_by: [ADR-0513]' docs/decisions/ADR-0511-ci-orchestration-argo-workflows-supersede-jenkins.md

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

python3 - <<'PY' "$red_fixture" "$tmp_dir/bad-red-made-clean.json"
import json, sys
src, dst = sys.argv[1:]
data=json.load(open(src))
data["decision_records"][1]["id"] = "ADR-0520"
data["decision_records"][1]["path"] = "docs/decisions/ADR-0520-kafka-to-pulsar-via-kop.md"
data["decision_records"][1]["renumbered_from"] = "ADR-0377"
json.dump(data, open(dst, "w"), indent=2)
PY
assert_fails_with bad_red_made_clean 'RED ADR hygiene fixture must produce violations' --registry "$registry" --fixture "$tmp_dir/bad-red-made-clean.json"

python3 - <<'PY' "$good_fixture" "$tmp_dir/bad-good-duplicate.json"
import json, sys
src, dst = sys.argv[1:]
data=json.load(open(src))
data["decision_records"][1]["id"] = "ADR-0377"
json.dump(data, open(dst, "w"), indent=2)
PY
assert_fails_with bad_good_duplicate 'GREEN ADR hygiene fixture produced violations' --registry "$registry" --fixture "$tmp_dir/bad-good-duplicate.json"

python3 - <<'PY' "$good_fixture" "$tmp_dir/bad-good-stale-reference.json"
import json, sys
src, dst = sys.argv[1:]
data=json.load(open(src))
data["active_documents"][0]["content"] = "VictoriaMetrics for metrics remains canonical."
json.dump(data, open(dst, "w"), indent=2)
PY
assert_fails_with bad_good_stale_reference 'GREEN ADR hygiene fixture produced violations' --registry "$registry" --fixture "$tmp_dir/bad-good-stale-reference.json"

echo "ADR hygiene checks passed"
