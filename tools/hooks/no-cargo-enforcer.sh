#!/usr/bin/env bash
# no-cargo-enforcer (PreToolUse:Bash) — Buck2 is the canonical build & verify tool.
# Founder directive 2026-05-29: "stop using cargo". Buck2 takeover (memory: canonical-monorepo-pattern).
# BLOCKS:  cargo build|check|test|nextest|clippy|run|bench  (buck2 replaces these)
# ALLOWS:  cargo metadata|install|vendor|--version|tree|search  (buckify + reindeer inputs)
set -uo pipefail

payload="$(cat)"

# Extract the Bash command from the PreToolUse tool_input JSON (fall back to raw payload).
cmd="$(printf '%s' "$payload" | python3 -c 'import sys,json
try:
    print(json.load(sys.stdin).get("tool_input",{}).get("command",""))
except Exception:
    print("")' 2>/dev/null || true)"

# cargo, optional toolchain (+stable), then a build/verify subcommand → blocked.
if printf '%s' "$cmd" | grep -Eq '(^|[;&|(]|[[:space:]])cargo[[:space:]]+(\+[^[:space:]]+[[:space:]]+)?(build|check|test|nextest|clippy|run|bench)([[:space:]]|$)'; then
  {
    echo "🚫 BLOCKED: 'cargo build/check/test/clippy/run/bench' is RETIRED."
    echo "Buck2 is the canonical build & verify tool (founder 2026-05-29: 'stop using cargo'; memory: canonical-monorepo-pattern)."
    echo "Use instead:"
    echo "    buck2 build //...           # build"
    echo "    buck2 build //...[check]    # type-check  (cargo-check equiv: rustc --emit=metadata)"
    echo "    buck2 test  //...           # test"
    echo "    buck2 build //...[clippy]   # clippy"
    echo "    buck2 build //... --filter lint   # rustfmt"
    echo "Still allowed: cargo metadata / cargo install / cargo vendor (buckify + reindeer inputs)."
  } >&2
  exit 2
fi
exit 0
