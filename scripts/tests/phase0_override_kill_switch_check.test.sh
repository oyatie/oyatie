#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

check="scripts/ci/assert-override-kill-switch.py"
matrix="specs/phase0-automation-matrix.json"
coverage="specs/phase0-automation-coverage-registry.json"
schema="specs/phase0-override-packet-schema.json"
good="specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0-good-cloud-ci-required-and-isolated.json"
bad="specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.2-bad-override-without-ttl-audit.json"


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


row = rows.get("AC-0.0-override-kill-switch")
expect(row is not None, "AC-0.0 override/kill-switch row missing")
target = row.get("target_gate_or_controller", "")
expect("cloud-ci/oya-ci override controller" in target, "AC-0.0 override row must preserve live cloud-ci/oya-ci override controller")
expect("//:phase0-override-kill-switch-check" in target, "AC-0.0 override row must name local Buck2 target")
expect(
    row.get("verification_command") == "buck2 build //:phase0-override-kill-switch-check",
    "AC-0.0 override row must record Buck2 local verification command",
)
claim_boundary = row.get("claim_boundary", "")
expect("not live protected-flow override authority" in claim_boundary, "AC-0.0 override claim boundary must preserve live-override non-claim")
expect("not P0.0 green" in claim_boundary, "AC-0.0 override claim boundary must preserve P0.0 non-claim")
expect(row.get("no_new_oya_cli_surface") is True, "AC-0.0 override must not add an oya CLI surface")

subject = subjects.get("AC-0.0")
expect(subject is not None, "AC-0.0 coverage subject missing")
expect("AC-0.0-override-kill-switch" in subject.get("mapped_row_ids", []), "AC-0.0 coverage must map override row")
commands = subject.get("verification_commands", {})
expect(
    commands.get("AC-0.0-override-kill-switch") == "buck2 build //:phase0-override-kill-switch-check",
    "AC-0.0 coverage subject must record override Buck2 local verification command",
)
coverage_note = subject.get("coverage_note", "")
expect("override/kill-switch" in coverage_note, "AC-0.0 coverage note must name override/kill-switch")
expect("not P0.0 green" in coverage_note, "AC-0.0 coverage note must preserve P0.0 non-claim")
PY

python3 "$check" --json > "$tmp_dir/good.json"
grep -Fq '"verdict": "PASS"' "$tmp_dir/good.json"
grep -Fq '"local_fixture_contract_proven": true' "$tmp_dir/good.json"
grep -Fq '"live_required_context_execution_proven": false' "$tmp_dir/good.json"
grep -Fq '"protected_flow_override_live": false' "$tmp_dir/good.json"
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

python3 - <<'PY' "$schema" "$tmp_dir/bad-schema-missing-no-new-oya-cli.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["required"] = [field for field in data["required"] if field != "no_new_oya_cli_surface"]
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_schema_missing_required 'schema.required missing no_new_oya_cli_surface' --schema "$tmp_dir/bad-schema-missing-no-new-oya-cli.json" --good-baseline-fixture "$good" --bad-baseline-fixture "$bad"

python3 - <<'PY' "$good" "$tmp_dir/bad-good-fixture-missing-ttl.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["override_packet"].pop("ttl_expires_at", None)
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_good_fixture_missing_ttl 'GOOD override packet has violations' --schema "$schema" --good-baseline-fixture "$tmp_dir/bad-good-fixture-missing-ttl.json" --bad-baseline-fixture "$bad"

python3 - <<'PY' "$good" "$tmp_dir/bad-good-fixture-new-oya-cli.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["override_packet"]["no_new_oya_cli_surface"] = False
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_good_fixture_new_oya_cli 'override_new_oya_cli_surface' --schema "$schema" --good-baseline-fixture "$tmp_dir/bad-good-fixture-new-oya-cli.json" --bad-baseline-fixture "$bad"

python3 - <<'PY' "$bad" "$tmp_dir/bad-red-fixture-missing-expected-override-violation.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["expected_violations"] = [v for v in data["expected_violations"] if v != "override_new_oya_cli_surface"]
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_red_fixture_missing_expected_violation 'RED fixture expected_violations must include all override violation classes' --schema "$schema" --good-baseline-fixture "$good" --bad-baseline-fixture "$tmp_dir/bad-red-fixture-missing-expected-override-violation.json"

echo "phase0 override/kill-switch fixture checks passed"
