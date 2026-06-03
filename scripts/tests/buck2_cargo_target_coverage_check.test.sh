#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

check="scripts/ci/assert-buck2-cargo-target-coverage.py"
spec="specs/buck2-cargo-target-coverage.json"

PYTHONDONTWRITEBYTECODE=1 python3 "$check" --json > "$tmp_dir/good.json"
grep -Fq '"verdict": "PASS"' "$tmp_dir/good.json"
grep -Fq '"target_coverage_measured": true' "$tmp_dir/good.json"
grep -Fq '"source_line_coverage_generated": false' "$tmp_dir/good.json"
grep -Fq '"live_required_context_execution_proven": false' "$tmp_dir/good.json"
grep -Fq '"p0_0_green": false' "$tmp_dir/good.json"
grep -Fq '"phase0_complete": false' "$tmp_dir/good.json"
python3 - <<'PY' "$tmp_dir/good.json"
import json, sys
x=json.load(open(sys.argv[1]))
assert x["verdict"] == "PASS"
assert x["workspace_member_count"] > 700
assert x["cargo_target_root_count"] >= x["workspace_member_count"]
assert x["buck2_mapped_target_root_count"] == x["cargo_target_root_count"]
assert x["missing_mappings"] == []
assert x["known_divergence_count"] == 0
assert x["p0_0_green"] is False
assert x["phase0_complete"] is False
PY

make_fixture_repo() {
  local dst="$1"
  mkdir -p "$dst/crates/app/src"
  cat > "$dst/Cargo.toml" <<'TOML'
[workspace]
members = ["crates/app"]
TOML
  cat > "$dst/crates/app/Cargo.toml" <<'TOML'
[package]
name = "fixture-app"
version = "0.1.0"
edition = "2024"
TOML
  cat > "$dst/crates/app/src/main.rs" <<'RS'
fn main() {
    println!("fixture");
}
RS
  mkdir -p "$dst/crates/app/src/bin"
  cat > "$dst/crates/app/src/bin/tool.rs" <<'RS'
fn main() {
    println!("tool");
}
RS
  cp "$spec" "$dst/spec.json"
}

bad_repo="$tmp_dir/bad-repo"
make_fixture_repo "$bad_repo"
set +e
PYTHONDONTWRITEBYTECODE=1 python3 "$check" --repo-root "$bad_repo" --cargo-toml Cargo.toml --spec spec.json --json > "$tmp_dir/bad-missing-buck.json" 2>&1
bad_rc=$?
set -e
if [ "$bad_rc" -eq 0 ]; then
  echo "expected missing BUCK fixture to fail" >&2
  cat "$tmp_dir/bad-missing-buck.json" >&2
  exit 1
fi
grep -Fq '"verdict": "FAIL"' "$tmp_dir/bad-missing-buck.json"
grep -Fq 'missing_buck2_target_root_mapping' "$tmp_dir/bad-missing-buck.json"
grep -Fq '"p0_0_green": false' "$tmp_dir/bad-missing-buck.json"

good_parent_repo="$tmp_dir/good-parent-repo"
make_fixture_repo "$good_parent_repo"
cat > "$good_parent_repo/BUCK" <<'BUCK'
rust_binary(
    name = "fixture-app",
    srcs = glob(["crates/app/src/**/*.rs"]),
    crate_root = "crates/app/src/main.rs",
    visibility = ["PUBLIC"],
)

rust_binary(
    name = "fixture-tool",
    srcs = glob(["crates/app/src/**/*.rs"]),
    crate_root = "crates/app/src/bin/tool.rs",
    visibility = ["PUBLIC"],
)
BUCK
PYTHONDONTWRITEBYTECODE=1 python3 "$check" --repo-root "$good_parent_repo" --cargo-toml Cargo.toml --spec spec.json --json > "$tmp_dir/good-parent.json"
grep -Fq '"verdict": "PASS"' "$tmp_dir/good-parent.json"
grep -Fq '"buck2_mapped_target_root_count": 2' "$tmp_dir/good-parent.json"

good_autobins_false_repo="$tmp_dir/good-autobins-false-repo"
make_fixture_repo "$good_autobins_false_repo"
cat > "$good_autobins_false_repo/crates/app/Cargo.toml" <<'TOML'
[package]
name = "fixture-app"
version = "0.1.0"
edition = "2024"
autobins = false

[[bin]]
name = "fixture-app"
path = "src/main.rs"
TOML
cat > "$good_autobins_false_repo/BUCK" <<'BUCK'
rust_binary(
    name = "fixture-app",
    srcs = glob(["crates/app/src/**/*.rs"]),
    crate_root = "crates/app/src/main.rs",
    visibility = ["PUBLIC"],
)
BUCK
PYTHONDONTWRITEBYTECODE=1 python3 "$check" --repo-root "$good_autobins_false_repo" --cargo-toml Cargo.toml --spec spec.json --json > "$tmp_dir/good-autobins-false.json"
grep -Fq '"verdict": "PASS"' "$tmp_dir/good-autobins-false.json"
grep -Fq '"cargo_target_root_count": 1' "$tmp_dir/good-autobins-false.json"

mutated_spec="$tmp_dir/spec-p0-green.json"
cp "$spec" "$mutated_spec"
python3 - <<'PY' "$mutated_spec"
from pathlib import Path
p=Path(__import__('sys').argv[1])
text=p.read_text().replace('"p0_0_green": false', '"p0_0_green": true')
p.write_text(text)
PY
set +e
PYTHONDONTWRITEBYTECODE=1 python3 "$check" --spec "$mutated_spec" --json > "$tmp_dir/bad-p0-green.json" 2>&1
claim_rc=$?
set -e
if [ "$claim_rc" -eq 0 ]; then
  echo "expected p0 green spec mutation to fail" >&2
  cat "$tmp_dir/bad-p0-green.json" >&2
  exit 1
fi
grep -Fq 'forbidden_true_or_missing_claim_p0_0_green' "$tmp_dir/bad-p0-green.json"

echo "buck2 cargo target coverage checks passed"
