#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

check="scripts/ci/assert-tenant-pipeline-isolation.py"
contract="specs/toolchain-tenant-isolation-fixtures.json"
good="specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0-good-cloud-ci-required-and-isolated.json"
bad="specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.3-bad-cross-tenant-shared-cache.json"

python3 "$check" --json > "$tmp_dir/good.json"
grep -Fq '"verdict": "PASS"' "$tmp_dir/good.json"
grep -Fq '"local_fixture_contract_proven": true' "$tmp_dir/good.json"
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

python3 - <<'PY' "$contract" "$tmp_dir/bad-contract-missing-audit-events.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["required_separation_surfaces"] = [s for s in data["required_separation_surfaces"] if s != "audit_events"]
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_contract_missing_surface 'contract.required_separation_surfaces missing audit_events' --contract "$tmp_dir/bad-contract-missing-audit-events.json" --good-baseline-fixture "$good" --bad-baseline-fixture "$bad"

python3 - <<'PY' "$good" "$tmp_dir/bad-good-fixture-shared-cache.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["tenant_pipeline_model"]["shared_surfaces"] = ["caches"]
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_good_fixture_shared_cache 'GREEN tenant model has violations' --contract "$contract" --good-baseline-fixture "$tmp_dir/bad-good-fixture-shared-cache.json" --bad-baseline-fixture "$bad"

python3 - <<'PY' "$bad" "$tmp_dir/bad-red-fixture-missing-expected-tenant-violation.json"
import json, sys
src, dst = sys.argv[1:]
data = json.load(open(src))
data["expected_violations"] = [v for v in data["expected_violations"] if v != "tenant_surfaces_shared"]
json.dump(data, open(dst, "w"))
PY
assert_fails_with bad_red_fixture_missing_expected_violation 'RED fixture expected_violations must include all tenant isolation violation classes' --contract "$contract" --good-baseline-fixture "$good" --bad-baseline-fixture "$tmp_dir/bad-red-fixture-missing-expected-tenant-violation.json"

echo "phase0 tenant-isolation fixture checks passed"
