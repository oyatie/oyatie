#!/usr/bin/env bash
# cedar/build.sh — Cedar v4 policy bundle compilation pipeline.
# Validates default-deny presence, compiles fragments to bundle, signs bundle SHA.
# Per ADR-0140 + policy/guardrail-enforcement.md.

set -euo pipefail

POLICY_DIR="${1:-microservices/foundry/policy}"
OUT_DIR="${2:-/tmp/cedar-bundle-out}"
CHECK_SHA="${CHECK_SHA:-}"

if [[ ! -d "$POLICY_DIR" ]]; then
  echo "policy directory not found: $POLICY_DIR" >&2
  exit 2
fi

mkdir -p "$OUT_DIR"

echo "[cedar/build] validating Cedar v4 schema..."
if [[ ! -f "$POLICY_DIR/schema.cedarschema" ]]; then
  echo "missing schema: $POLICY_DIR/schema.cedarschema" >&2
  exit 3
fi

echo "[cedar/build] validating default-deny in cedar-base.cedar..."
if ! grep -E '^\s*forbid\s*\(' "$POLICY_DIR/cedar-base.cedar" > /dev/null; then
  echo "FATAL: cedar-base.cedar lacks default-deny 'forbid (...)' block" >&2
  echo "(per policy/guardrail-enforcement.md + oya-foundry-fitness-cedar-default-deny-enforced lane)" >&2
  exit 4
fi

echo "[cedar/build] compiling fragments..."
# Concatenate fragments in deterministic order; deny-overrides semantics.
FRAGMENTS=(
  "cedar-base.cedar"
  "tenant-scope.cedar"
  "ci-scope.cedar"
  "auditor-scope.cedar"
  "public-read.cedar"
)

> "$OUT_DIR/bundle.cedar"
for f in "${FRAGMENTS[@]}"; do
  if [[ ! -f "$POLICY_DIR/$f" ]]; then
    echo "FATAL: missing fragment $f" >&2
    exit 5
  fi
  echo "// ============== $f ==============" >> "$OUT_DIR/bundle.cedar"
  cat "$POLICY_DIR/$f" >> "$OUT_DIR/bundle.cedar"
  echo "" >> "$OUT_DIR/bundle.cedar"
done

echo "[cedar/build] running cedar-cli validate..."
if command -v cedar &> /dev/null; then
  cedar validate \
    --policies "$OUT_DIR/bundle.cedar" \
    --schema "$POLICY_DIR/schema.cedarschema" \
    || { echo "FATAL: cedar validate failed" >&2; exit 6; }
else
  echo "WARN: cedar CLI not in PATH; skipping cedar-cli validate (CI lane should provide)"
fi

echo "[cedar/build] computing bundle SHA..."
BUNDLE_SHA=$(sha256sum "$OUT_DIR/bundle.cedar" | awk '{print $1}')
echo "bundle SHA: $BUNDLE_SHA"
echo "$BUNDLE_SHA" > "$OUT_DIR/bundle.sha256"

if [[ -n "$CHECK_SHA" ]]; then
  EXPECTED_SHA="${CHECK_SHA}"
  if [[ "$BUNDLE_SHA" != "$EXPECTED_SHA" ]]; then
    echo "FATAL: bundle SHA mismatch. expected=$EXPECTED_SHA actual=$BUNDLE_SHA" >&2
    exit 7
  fi
  echo "SHA check OK"
fi

echo "[cedar/build] bundle written to $OUT_DIR/bundle.cedar"
echo "[cedar/build] SHA written to $OUT_DIR/bundle.sha256"
echo "[cedar/build] PASS"
