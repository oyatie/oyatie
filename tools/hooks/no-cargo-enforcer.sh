#!/usr/bin/env bash
# no-cargo-enforcer (PreToolUse:Bash) — Buck2 is the canonical build, test,
# script, CI, and CD execution tool. Non-blocking hook; governance policy enforces.
set -uo pipefail

payload="$(cat)"

cmd="$(printf '%s' "$payload" | python3 -c 'import sys,json
try:
    print(json.load(sys.stdin).get("tool_input",{}).get("command",""))
except Exception:
    print("")' 2>/dev/null || true)"

if printf '%s' "$cmd" | grep -Eq '(^|[;&|(]|[[:space:]])cargo[[:space:]]+(\+[^[:space:]]+[[:space:]]+)?(build|check|test|nextest|clippy|run|bench|deny|cyclonedx|install)([[:space:]]|$)'; then
  {
    echo "ℹ [no-cargo-enforcer] Cargo executable lanes are retired for active scripts/CI/CD/build."
    echo "Use Buck2 instead:"
    echo "    buck2 build //..."
    echo "    buck2 test  //..."
    echo "    infra/ci/buck2-affected-gate.sh origin/dev HEAD"
    echo "Allowed exceptions are narrow and evidence-labeled:"
    echo "    1. production release image/binary optimization only (release profile, target triple, size/codegen/allocator evidence, non-claim label)"
    echo "    2. cargo metadata/vendor for Buck2/Reindeer graph generation only; never merge authority"
    echo "This hook is advisory only; scripts/ci/enforce-buck2-authority.py is the automated scanner."
  } >&2
fi
exit 0
