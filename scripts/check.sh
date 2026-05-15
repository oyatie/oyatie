#!/usr/bin/env bash
# CANONICAL ENTRY POINT (preferred):
#     cargo run -q -p oya-dev-cli -- gate run-all
#
# This script is retained as a transitional fixture per Wave 2 of the
# shell/python → Rust replacement program (audit row B-1,
# `evidence/audits/shell-python-replacement-audit-2026-05-15.md`). The
# canonical Rust implementation now lives at
# `crates/oya-dev-cli/src/commands/gate/run_all.rs`.
#
# The legacy command list below is preserved verbatim because three
# downstream content-validation gates read this script's body as their
# input data:
#   - `oya gate validate quality-lanes` reads it to verify each lane's
#     `check_command` (in `registry/quality/lanes.yaml`) is wired.
#   - `oya gate validate documentation-system` reads it to verify each
#     active doc-pipeline step (in `registry/docs/pipeline.tsv`) is wired.
#   - `oya gate validate supply-chain` reads it to verify the
#     `cargo deny check` / `cargo audit` tokens are present.
# A follow-up ADR will re-point those three gates at the Rust
# aggregator's `AGGREGATED_VALIDATE_LANES` constant (or an emitted
# manifest), after which this file will be `git rm`-ed.
set -euo pipefail

# Hyperscaler hermetic toolchain: prioritize rustup from Homebrew if present.
if [[ -d "/opt/homebrew/opt/rustup/bin" ]]; then
  export PATH="/opt/homebrew/opt/rustup/bin:${PATH}"
fi

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

scripts/check-stage0-application-shell-prereqs.py --self-test
scripts/render-m02-exit-checklist.py --check
scripts/render-master-plan-ledger.py --check
scripts/audit-master-plan-completion.py --check
cargo run -q -p oya-foundry-vcs-admission-gate-app
cargo run -q -p oya-dev-cli -- gate validate codeview-read-surface
cargo run -q -p oya-foundry-fitness-purpose-audit-app -- "docs/*.md" "docs/standards/*.md" "docs/runbooks/*.md" "registries/**/*.json" "specs/**/*.json" "contracts/**/*.json"
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
cargo run -p oya-dev-cli -- demo
cargo run -p oya-dev-cli -- gate validate typescript-workspace --lane typecheck
cargo run -p oya-dev-cli -- gate validate typescript-workspace --lane test
cargo run -p oya-dev-cli -- catalog validate
cargo run -p oya-dev-cli -- gate validate active-artifact-contract --emit-evidence evidence/active-artifact-contract-lane-run.json --emit-graph-edges registries/cross-cutting/graph/active-artifact-contract-edges.json
cargo run -p oya-dev-cli -- gate validate authority-cohesion
cargo run -p oya-dev-cli -- gate validate cedar-fragment-coverage --emit-evidence evidence/cedar-fragment-coverage-lane-run.json
cargo run -p oya-dev-cli -- gate validate openapi-rest-route-parity --emit-evidence evidence/openapi-rest-route-parity-lane-run.json
cargo run -p oya-dev-cli -- gate validate claim-ceiling
cargo run -p oya-dev-cli -- gate validate codeowners-mirror
cargo run -p oya-dev-cli -- gate validate cohesion
cargo run -p oya-dev-cli -- gate validate data-class
cargo run -p oya-dev-cli -- gate validate doc-catalog
cargo run -p oya-dev-cli -- gate validate documentation-system
cargo run -p oya-dev-cli -- doc mdbook
cargo run -p oya-dev-cli -- doc openapi
run_with_heartbeat "cargo run -p oya-dev-cli -- doc rustdoc" \
  cargo run -p oya-dev-cli -- doc rustdoc
cargo run -p oya-dev-cli -- doc adr-index
cargo run -p oya-dev-cli -- gate validate adr-citation
cargo run -p oya-dev-cli -- gate validate brand-residue
cargo run -p oya-dev-cli -- gate validate api-semver
cargo run -p oya-dev-cli -- gate validate supply-chain
cargo run -p oya-dev-cli -- gate validate supply-chain --require-adr0039-evidence
cargo run -p oya-dev-cli -- gate validate pr-traceability
cargo run -p oya-dev-cli -- gate validate cargo-prefix
cargo run -p oya-dev-cli --bin repoctl -- pre-push --verify-contract
# Retired `oya check ...` aggregate commands are covered by the gate lanes above.
cargo run -p oya-dev-cli -- gate validate quality-lanes
cargo run -p oya-dev-cli -- gate validate foundation-bypass
cargo run -p oya-dev-cli -- gate validate audit-chain-replay
cargo run -p oya-dev-cli -- gate validate foundry-capability-schema
cargo run -p oya-dev-cli -- gate validate foundry-eval
cargo run -p oya-dev-cli -- gate validate cross-tenant-access-fuzz
cargo run -p oya-dev-cli -- gate validate vendor-contract-recency
cargo run -p oya-dev-cli -- gate validate mobile-native
cargo run -p oya-dev-cli -- gate validate glossary-cross-doc-coverage
cargo run -p oya-dev-cli -- gate validate glossary-vocabulary
cargo run -p oya-dev-cli -- gate validate placeholder-debt
cargo run -p oya-dev-cli -- gate validate license-policy
cargo run -p oya-dev-cli -- gate validate plane-class
cargo run -p oya-dev-cli -- gate validate raci-team-coverage
cargo run -p oya-dev-cli -- gate validate readme-doc-coverage
cargo run -p oya-dev-cli -- gate validate runbook-index-resolves
cargo run -p oya-dev-cli -- gate validate runbook-freshness
cargo run -p oya-dev-cli -- gate validate release-supply-chain --phase pre-release
cargo run -p oya-dev-cli -- gate validate release-evidence-pack
cargo run -p oya-dev-cli -- gate validate slo-coverage
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
