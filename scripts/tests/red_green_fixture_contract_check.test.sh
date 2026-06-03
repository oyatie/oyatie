#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

check="scripts/ci/assert-red-green-fixture-contract.py"
spec="specs/red-green-fixture-contract.json"

PYTHONDONTWRITEBYTECODE=1 python3 "$check" --json > "$tmp_dir/good.json"
grep -Fq '"verdict": "PASS"' "$tmp_dir/good.json"
grep -Fq '"red_green_fixture_contract_measured": true' "$tmp_dir/good.json"
grep -Fq '"entry_count": 19' "$tmp_dir/good.json"
grep -Fq '"green_marker_count": 19' "$tmp_dir/good.json"
grep -Fq '"red_marker_count": 19' "$tmp_dir/good.json"
grep -Fq '"live_required_context_execution_proven": false' "$tmp_dir/good.json"
grep -Fq '"p0_0_green": false' "$tmp_dir/good.json"
grep -Fq '"phase0_complete": false' "$tmp_dir/good.json"
python3 - <<'PY' "$tmp_dir/good.json"
import json, sys
payload=json.load(open(sys.argv[1]))
assert payload["verdict"] == "PASS"
assert payload["entry_count"] >= 15
assert payload["buck2_target_count"] == payload["entry_count"]
assert payload["green_marker_count"] >= payload["entry_count"]
assert payload["red_marker_count"] >= payload["entry_count"]
assert payload["failures"] == []
assert payload["p0_0_green"] is False
assert payload["phase0_complete"] is False
PY

assert_fails_with() {
  local label="$1"
  local expected="$2"
  shift 2
  local mutated="$tmp_dir/${label}.json"
  cp "$spec" "$mutated"
  python3 - <<'PY' "$mutated" "$@"
import json, sys
from pathlib import Path
path=Path(sys.argv[1])
mode=sys.argv[2]
data=json.loads(path.read_text())
if mode == "p0-green":
    data["claim_boundary"]["p0_0_green"] = True
elif mode == "remove-red-marker":
    data["fixture_contract_entries"][0]["red_markers"] = []
elif mode == "stale-marker-text":
    data["fixture_contract_entries"][0]["green_markers"][0]["contains"] = "definitely-not-present"
elif mode == "missing-target":
    data["fixture_contract_entries"][0]["buck2_target"] = "//:missing-red-green-target"
else:
    raise SystemExit(f"unknown mode {mode}")
path.write_text(json.dumps(data, indent=2) + "\n")
PY
  set +e
  PYTHONDONTWRITEBYTECODE=1 python3 "$check" --spec "$mutated" --json > "$tmp_dir/${label}.out" 2>&1
  local rc=$?
  set -e
  if [ "$rc" -eq 0 ]; then
    echo "expected $label to fail" >&2
    cat "$tmp_dir/${label}.out" >&2
    exit 1
  fi
  grep -Fq "$expected" "$tmp_dir/${label}.out"
  grep -Fq '"p0_0_green": false' "$tmp_dir/${label}.out"
  grep -Fq '"phase0_complete": false' "$tmp_dir/${label}.out"
}

assert_fails_with p0-green 'forbidden_true_or_missing_claim_p0_0_green' p0-green
assert_fails_with remove-red-marker 'missing_red_markers' remove-red-marker
assert_fails_with stale-marker-text 'marker_text_missing' stale-marker-text
assert_fails_with missing-target 'buck2_target_missing' missing-target

echo "red-green fixture contract checks passed"
