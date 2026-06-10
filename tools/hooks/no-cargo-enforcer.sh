#!/usr/bin/env bash
# no-cargo-enforcer (PreToolUse:Bash) — Buck2 is the canonical build & verify tool.
# Founder directive 2026-05-29: "stop using cargo". Buck2 takeover (memory: canonical-monorepo-pattern).
# BLOCKS:  cargo build|check|test|nextest|clippy|run|bench  (buck2 replaces these)
# ALLOWS:  cargo metadata|install|vendor|--version|tree|search  (buckify + reindeer inputs)
set -uo pipefail

payload="$(cat)"

# Extract the Bash command from the PreToolUse tool_input JSON without an external
# interpreter (interpreter-free runtime hook hot path; the oya-ci-required gate
# pipeline (ADR-0515) owns manifest/config drift enforcement). Empty extraction
# simply means no command matched.
cmd="$(printf '%s' "$payload" \
  | tr '\n' ' ' \
  | sed -nE 's/.*"command"[[:space:]]*:[[:space:]]*"(([^"\\]|\\.)*)".*/\1/p' \
  | head -n 1 \
  | sed -E 's/\\n/ /g; s/\\t/ /g; s/\\r/ /g; s/\\"/"/g; s/\\\\/\\/g')"

# Strip quoted string arguments before pattern-matching so that forbidden tool
# names appearing inside message text (git commit -m "...cargo build...", gh pr
# comment --body "...", omc team api send-message --input '{"body":"..."}') do
# not produce false positives.  Tradeoff: quoted invocations such as
# bash -c "cargo build" are no longer caught here — acceptable because hooks are
# the last-stop safety net per enforcement-layering doctrine (AMENDMENT 5); the
# canonical CI gates remain the authority.
cmd_stripped="$(printf '%s' "$cmd" | sed -E "s/'[^']*'//g; s/\"[^\"]*\"//g")"

# cargo, optional toolchain (+stable), then a build/verify subcommand → blocked.
if printf '%s' "$cmd_stripped" | grep -Eq '(^|[;&|(]|[[:space:]])cargo[[:space:]]+(\+[^[:space:]]+[[:space:]]+)?(build|check|test|nextest|clippy|run|bench)([[:space:]]|$)'; then
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
