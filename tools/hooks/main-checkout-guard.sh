#!/usr/bin/env bash
# main-checkout-guard (PreToolUse:Bash)
#
# Irreducible glue only: Claude/Codex hooks execute commands, so this shim locates
# the Rust guard binary and execs it. Policy lives in tools/oya-checkout-guard-app.
set -uo pipefail

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "$repo_root" ]; then
  echo "main-checkout-guard warning: repository root not found; allowing command" >&2
  exit 0
fi

guard_bin="$repo_root/tools/hooks/bin/oya-checkout-guard"
if [ ! -x "$guard_bin" ]; then
  echo "main-checkout-guard warning: $guard_bin is missing; run buck2 build //tools/oya-checkout-guard-app:oya-checkout-guard --out tools/hooks/bin/oya-checkout-guard; allowing command" >&2
  exit 0
fi

exec "$guard_bin"
