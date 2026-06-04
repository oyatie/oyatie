#!/usr/bin/env sh
# Compatibility shim: on-prem bring-up routes through the Buck2-built oya binary.
set -eu
REPO_ROOT="${OYA_REPO_ROOT:-$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)}"
cd "$REPO_ROOT"
BUCK2="${BUCK2:-buck2}"
command -v "$BUCK2" >/dev/null 2>&1 || { echo "required command not found: $BUCK2" >&2; exit 127; }
exec "$BUCK2" run //oya/developer-sdk/crates/oya-dev-cli:oya -- ops onprem-bring-up --repo-root "$REPO_ROOT" "$@"
