#!/usr/bin/env bash
set -euo pipefail

pin_rust_toolchain() {
  local rustc_path
  local toolchain_bin

  if command -v rustup >/dev/null 2>&1; then
    rustc_path="$(rustup which rustc)"
  else
    rustc_path="$(command -v rustc)"
  fi

  toolchain_bin="$(dirname "${rustc_path}")"
  export PATH="${toolchain_bin}:${PATH}"
}

pin_rust_toolchain

run_with_heartbeat() {
  local label="$1"
  shift
  "$@" &
  local pid="$!"
  local elapsed=0
  while kill -0 "${pid}" 2>/dev/null; do
    sleep 5
    elapsed=$((elapsed + 5))
    if kill -0 "${pid}" 2>/dev/null; then
      printf '[check] still running after %ss: %s\n' "${elapsed}" "${label}"
    fi
  done
  wait "${pid}"
}

cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo machete
cargo audit
if ! command -v cargo-nextest >/dev/null 2>&1 && ! cargo nextest --help >/dev/null 2>&1; then
  printf 'cargo-nextest is required by docs/standards/testing.md; install cargo-nextest before running scripts/check.sh\n' >&2
  exit 1
fi
cargo nextest run --workspace --all-features --no-fail-fast
cargo run -p oya-tooling-cli-dev-runtime -- demo
cargo run -p oya-tooling-cli-dev-runtime -- gate validate typescript-workspace --lane typecheck
cargo run -p oya-tooling-cli-dev-runtime -- gate validate typescript-workspace --lane test
cargo run -p oya-tooling-cli-dev-runtime -- catalog validate
cargo run -p oya-tooling-cli-dev-runtime -- gate validate authority-cohesion
cargo run -p oya-tooling-cli-dev-runtime -- gate validate claim-ceiling
cargo run -p oya-tooling-cli-dev-runtime -- gate validate codeowners-mirror
cargo run -p oya-tooling-cli-dev-runtime -- gate validate cohesion
cargo run -p oya-tooling-cli-dev-runtime -- gate validate constitution-cite-coverage
cargo run -p oya-tooling-cli-dev-runtime -- gate validate data-class
cargo run -p oya-tooling-cli-dev-runtime -- gate validate doc-catalog
cargo run -p oya-tooling-cli-dev-runtime -- gate validate documentation-system
cargo run -p oya-tooling-cli-dev-runtime -- doc mdbook
cargo run -p oya-tooling-cli-dev-runtime -- doc openapi
run_with_heartbeat "cargo run -p oya-tooling-cli-dev-runtime -- doc rustdoc" \
  cargo run -p oya-tooling-cli-dev-runtime -- doc rustdoc
cargo run -p oya-tooling-cli-dev-runtime -- doc adr-index
cargo run -p oya-tooling-cli-dev-runtime -- gate validate adr-citation
cargo run -p oya-tooling-cli-dev-runtime -- gate validate brand-residue
cargo run -p oya-tooling-cli-dev-runtime -- gate validate api-semver
cargo run -p oya-tooling-cli-dev-runtime -- gate validate supply-chain
cargo run -p oya-tooling-cli-dev-runtime -- gate validate supply-chain --require-adr0039-evidence
cargo run -p oya-tooling-cli-dev-runtime -- gate validate pr-traceability
cargo run -p oya-tooling-cli-dev-runtime -- gate validate cargo-prefix
cargo run -p oya-tooling-cli-dev-runtime --bin repoctl -- pre-push --verify-contract
cargo run -p oya-tooling-cli-dev-runtime -- gate validate quality-lanes
cargo run -p oya-tooling-cli-dev-runtime -- gate validate foundation-bypass
cargo run -p oya-tooling-cli-dev-runtime -- gate validate audit-chain-replay
cargo run -p oya-tooling-cli-dev-runtime -- gate validate foundry-capability-schema
cargo run -p oya-tooling-cli-dev-runtime -- gate validate foundry-eval
cargo run -p oya-tooling-cli-dev-runtime -- gate validate cross-tenant-access-fuzz
cargo run -p oya-tooling-cli-dev-runtime -- gate validate vendor-contract-recency
cargo run -p oya-tooling-cli-dev-runtime -- gate validate mobile-native
cargo run -p oya-tooling-cli-dev-runtime -- gate validate glossary-cross-doc-coverage
cargo run -p oya-tooling-cli-dev-runtime -- gate validate glossary-vocabulary
cargo run -p oya-tooling-cli-dev-runtime -- gate validate placeholder-debt
cargo run -p oya-tooling-cli-dev-runtime -- gate validate license-policy
cargo run -p oya-tooling-cli-dev-runtime -- gate validate plane-class
cargo run -p oya-tooling-cli-dev-runtime -- gate validate raci-team-coverage
cargo run -p oya-tooling-cli-dev-runtime -- gate validate readme-doc-coverage
cargo run -p oya-tooling-cli-dev-runtime -- gate validate runbook-index-resolves
cargo run -p oya-tooling-cli-dev-runtime -- gate validate runbook-freshness
cargo run -p oya-tooling-cli-dev-runtime -- gate validate release-supply-chain --phase pre-release
cargo run -p oya-tooling-cli-dev-runtime -- gate validate release-evidence-pack
cargo run -p oya-tooling-cli-dev-runtime -- gate validate slo-coverage
scripts/check-architecture-boundaries.sh --self-test
scripts/check-architecture-boundaries.sh
scripts/check-product-index.py
cargo deny check
python3 - <<'PY'
import json
import pathlib
for path in sorted(pathlib.Path('docs/machine-readable').glob('*.json')):
    json.loads(path.read_text())
print('machine-readable JSON parse check passed')
PY
