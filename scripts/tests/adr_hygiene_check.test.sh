#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

check="scripts/ci/assert-adr-hygiene.py"
registry="specs/adr-hygiene-registry.json"
matrix="specs/phase0-automation-matrix.json"
coverage="specs/phase0-automation-coverage-registry.json"
good_fixture="specs/fixtures/phase0-adr-hygiene/tc-adr-hygiene-good-renumbered-superseded-clean.json"
red_fixture="specs/fixtures/phase0-adr-hygiene/tc-adr-hygiene-bad-duplicate-adr-number.json"

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
    (candidate for candidate in matrix["seed_rows"] if candidate.get("id") == "AC-0.3-adr-hygiene"),
    None,
)
expect(row is not None, "AC-0.3 ADR hygiene row missing")
expect(
    row.get("target_gate_or_controller") == "//:adr-hygiene-check",
    "AC-0.3 ADR hygiene row must map to exact Buck2 target",
)
expect(
    row.get("verification_command") == "buck2 build //:adr-hygiene-check",
    "AC-0.3 ADR hygiene row must record exact verification command",
)
claim_boundary = row.get("claim_boundary", "")
expect(
    "full ADR index regeneration" in claim_boundary,
    "AC-0.3 ADR hygiene row must preserve full-index non-claim",
)
expect(
    "not live required-context authority" in claim_boundary,
    "AC-0.3 ADR hygiene row must preserve live-authority non-claim",
)
expect(
    row.get("no_new_oya_cli_surface") is True,
    "AC-0.3 ADR hygiene row must refuse new oya CLI authority",
)

subject = next(
    (candidate for candidate in coverage["coverage_subjects"] if candidate.get("id") == "AC-0.3"),
    None,
)
expect(subject is not None, "AC-0.3 coverage subject missing")
expect(
    subject.get("verification_command") == "buck2 build //:adr-hygiene-check",
    "AC-0.3 coverage subject must record exact verification command",
)
coverage_note = subject.get("coverage_note", "")
expect(
    "//:adr-hygiene-check" in coverage_note,
    "AC-0.3 coverage subject must name the exact Buck2 target",
)
expect(
    "full ADR index regeneration" in coverage_note,
    "AC-0.3 coverage subject must preserve full-index non-claim",
)
expect(
    "live required-context authority remain false" in coverage_note,
    "AC-0.3 coverage subject must preserve live-authority non-claim",
)
PY

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
