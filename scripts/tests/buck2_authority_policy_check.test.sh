#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

check="scripts/ci/enforce-buck2-authority.py"
policy="$tmp_dir/buck2-authority-policy.json"
matrix="$tmp_dir/phase0-automation-matrix.json"
coverage="$tmp_dir/phase0-automation-coverage-registry.json"
parity="$tmp_dir/oya-ci-prow-capability-parity.json"
root_hub="$tmp_dir/root-hub-pointers.json"

cp specs/buck2-authority-policy.json "$policy"
cp specs/phase0-automation-matrix.json "$matrix"
cp specs/phase0-automation-coverage-registry.json "$coverage"
cp specs/oya-ci-prow-capability-parity.json "$parity"
cp specs/root-hub-pointers.json "$root_hub"

if [ "${BUCK2_AUTHORITY_FIXTURE_MODE:-0}" = "1" ]; then
  python3 - <<'PY' "$policy"
import json, sys
path=sys.argv[1]
data=json.load(open(path))
data["command_scan_files"] = []
data["command_scan_globs"] = []
data["status_context_scan_files"] = []
data["required_anchors"] = {}
data["required_glob_anchors"] = []
data["adr_amendment_files"] = []
json.dump(data, open(path, "w"))
PY
fi

run_check() {
  PYTHONDONTWRITEBYTECODE=1 OYA_REPO_ROOT="$repo_root" python3 "$check" \
    --policy "$policy" \
    --matrix "$matrix" \
    --coverage-registry "$coverage" \
    --prow-parity-registry "$parity" \
    --root-hub "$root_hub"
}

run_check > "$tmp_dir/good.json"
grep -Fq '"verdict": "PASS"' "$tmp_dir/good.json"
grep -Fq '"authority_context": "oya-ci-required"' "$tmp_dir/good.json"

reset_fixtures() {
  cp specs/buck2-authority-policy.json "$policy"
  cp specs/phase0-automation-matrix.json "$matrix"
  cp specs/phase0-automation-coverage-registry.json "$coverage"
  cp specs/oya-ci-prow-capability-parity.json "$parity"
  cp specs/root-hub-pointers.json "$root_hub"
  if [ "${BUCK2_AUTHORITY_FIXTURE_MODE:-0}" = "1" ]; then
    python3 - <<'PY' "$policy"
import json, sys
path=sys.argv[1]
data=json.load(open(path))
data["command_scan_files"] = []
data["command_scan_globs"] = []
data["status_context_scan_files"] = []
data["required_anchors"] = {}
data["required_glob_anchors"] = []
data["adr_amendment_files"] = []
json.dump(data, open(path, "w"))
PY
  fi
}

assert_fails_with() {
  local label="$1"
  local expected="$2"
  local out="$tmp_dir/${label}.out"
  set +e
  run_check > "$out" 2>&1
  local status=$?
  set -e
  if [ "$status" -eq 0 ]; then
    echo "expected $label to fail" >&2
    cat "$out" >&2
    exit 1
  fi
  grep -Fq "buck2-authority-policy: RED" "$out"
  grep -Fq "$expected" "$out"
}

reset_fixtures
python3 - <<'PY' "$parity"
import json, sys
path=sys.argv[1]
data=json.load(open(path))
data["required_capability_ids"]=[item for item in data["required_capability_ids"] if item != "prow-tide-merge-automation"]
data["capabilities"]=[item for item in data["capabilities"] if item.get("id") != "prow-tide-merge-automation"]
json.dump(data, open(path, "w"))
PY
assert_fails_with missing_required_capability "required_capability_ids missing prow-tide-merge-automation"

reset_fixtures
python3 - <<'PY' "$parity"
import json, copy, sys
path=sys.argv[1]
data=json.load(open(path))
data["capabilities"].append(copy.deepcopy(data["capabilities"][0]))
json.dump(data, open(path, "w"))
PY
assert_fails_with duplicate_capability "capabilities must have unique string ids"

reset_fixtures
python3 - <<'PY' "$parity"
import json, sys
path=sys.argv[1]
data=json.load(open(path))
data["capabilities"][0]["live_authority_claimed"] = True
json.dump(data, open(path, "w"))
PY
assert_fails_with live_authority_claim "live_authority_claimed must be false"

reset_fixtures
python3 - <<'PY' "$policy"
import json, sys
path=sys.argv[1]
data=json.load(open(path))
data["target_authority"]["required_context"] = "cargo-ci-required"
json.dump(data, open(path, "w"))
PY
assert_fails_with wrong_required_context "target_authority.required_context must be oya-ci-required"

reset_fixtures
python3 - <<'PY' "$parity"
import json, sys
path=sys.argv[1]
data=json.load(open(path))
data["claim_boundary"]["production_readiness"] = True
data["claim_boundary"]["hyperscaler_grade_readiness"] = True
json.dump(data, open(path, "w"))
PY
assert_fails_with false_production_claim "claim_boundary.production_readiness must be false"

reset_fixtures
python3 - <<'PY' "$parity"
import json, sys
path=sys.argv[1]
data=json.load(open(path))
data["excluded_or_superseded_upstream_components"]=[item for item in data["excluded_or_superseded_upstream_components"] if item.get("id") != "prow-gcsupload"]
json.dump(data, open(path, "w"))
PY
assert_fails_with missing_excluded_component "excluded_or_superseded_upstream_components missing prow-gcsupload"

reset_fixtures
python3 - <<'PY' "$policy"
import json, sys
path=sys.argv[1]
data=json.load(open(path))
data["target_authority"]["producer"] = "cloud-ci/oya-ci Rust Prow reimplementation trusted bridge"
json.dump(data, open(path, "w"))
PY
assert_fails_with missing_source_bound_producer "target_authority.producer must contain 'source-bound'"

reset_fixtures
python3 - <<'PY' "$parity"
import json, sys
path=sys.argv[1]
data=json.load(open(path))
data["required_capability_ids"]=["prow-plank-job-controller" if item == "prow-controller-manager-job-controller" else item for item in data["required_capability_ids"]]
for item in data["capabilities"]:
    if item.get("id") == "prow-controller-manager-job-controller":
        item["id"] = "prow-plank-job-controller"
        item["upstream_source"] = "https://docs.prow.k8s.io/docs/components/deprecated/plank/"
json.dump(data, open(path, "w"))
PY
assert_fails_with stale_plank_primary "required_capability_ids missing prow-controller-manager-job-controller"

reset_fixtures
python3 - <<'PY' "$root_hub"
import json, sys
path=sys.argv[1]
data=json.load(open(path))
data["entry_points"].pop("oya_ci_prow_capability_parity", None)
data["pointers"].pop("oya_ci_prow_capability_parity", None)
json.dump(data, open(path, "w"))
PY
assert_fails_with missing_root_pointer "entry_points must include oya_ci_prow_capability_parity"

echo "buck2 authority policy parity fixture tests passed"
