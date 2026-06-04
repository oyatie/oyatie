#!/usr/bin/env bash
set -euo pipefail
BUCK2="${BUCK2:-buck2}"
command -v "$BUCK2" >/dev/null 2>&1 || { echo "required command not found: $BUCK2" >&2; exit 127; }
exec "$BUCK2" run //oya/developer-sdk/crates/oya-dev-cli:oya -- supply-chain install-trivy "$@"
