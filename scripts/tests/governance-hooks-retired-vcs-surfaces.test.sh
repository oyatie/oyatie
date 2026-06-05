#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

canonical_primitives="$repo_root/specs/canonical-primitives.json"
jq -e '
  ._meta.purpose
  | contains("Single source of truth for canonical primitives")
  and contains("no SessionStart runtime renderer")
' "$canonical_primitives" >/dev/null
grep -q 'OpenAPI 3.2.0' "$canonical_primitives"
grep -q 'AsyncAPI 3.1.0' "$canonical_primitives"
if jq -e '
  [
    paths(scalars) as $path
    | select($path != ["_meta", "supersedes"])
    | getpath($path)
    | select(type == "string" and contains("tools/hooks/_canonical-primitives.md"))
  ]
  | length > 0
' "$canonical_primitives" >/dev/null; then
  echo "canonical primitives still actively point at retired markdown canonical primitives outside the supersedes tombstone" >&2
  exit 1
fi
if grep -Eiq 'oya[[:space:]]+(git|vcs|gate|verify)|\.\/bin\/oya|bin/oya|oya --help|Oya CLI|oya CLI' "$canonical_primitives"; then
  echo "canonical primitives still emit retired wrapper command guidance" >&2
  exit 1
fi
if [ -e "$repo_root/tools/hooks/stale-tool-suggester.sh" ]; then
  echo "stale-tool-suggester hook should be deleted with retired wrapper command guidance" >&2
  exit 1
fi

tmpdir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmpdir"
}
trap cleanup EXIT

no_cargo_output="$tmpdir/no-cargo.out"
set +e
printf '%s\n' '{"tool_input":{"command":"cargo test --workspace"}}' \
  | bash "$repo_root/tools/hooks/no-cargo-enforcer.sh" >"$no_cargo_output" 2>&1
no_cargo_status=$?
set -e
if [ "$no_cargo_status" -ne 0 ]; then
  echo "no-cargo hook must be advisory/non-blocking, got exit $no_cargo_status" >&2
  cat "$no_cargo_output" >&2
  exit 1
fi
grep -q 'Cargo executable lanes are retired' "$no_cargo_output"
grep -q 'advisory only' "$no_cargo_output"
if grep -q 'python3' "$repo_root/tools/hooks/no-cargo-enforcer.sh"; then
  echo "no-cargo hook must not invoke Python on the runtime hot path" >&2
  exit 1
fi

no_cargo_fmt_output="$tmpdir/no-cargo-fmt.out"
printf '%s\n' '{"tool_input":{"command":"cargo fmt --check"}}' \
  | bash "$repo_root/tools/hooks/no-cargo-enforcer.sh" >"$no_cargo_fmt_output" 2>&1
grep -q 'run pinned rustfmt directly' "$no_cargo_fmt_output"

no_cargo_search_output="$tmpdir/no-cargo-search.out"
printf '%s\n' '{"tool_input":{"command":"rg -n '\''cargo fmt|cargo test'\'' docs specs"}}' \
  | bash "$repo_root/tools/hooks/no-cargo-enforcer.sh" >"$no_cargo_search_output" 2>&1
if [ -s "$no_cargo_search_output" ]; then
  echo "quoted search strings mentioning Cargo must not trigger no-cargo advisory" >&2
  cat "$no_cargo_search_output" >&2
  exit 1
fi

no_cargo_escaped_search_output="$tmpdir/no-cargo-escaped-search.out"
printf '%s\n' '{"tool_input":{"command":"rg -n \"cargo fmt\" docs specs"}}' \
  | bash "$repo_root/tools/hooks/no-cargo-enforcer.sh" >"$no_cargo_escaped_search_output" 2>&1
if [ -s "$no_cargo_escaped_search_output" ]; then
  echo "escaped quoted search strings mentioning Cargo must not trigger no-cargo advisory" >&2
  cat "$no_cargo_escaped_search_output" >&2
  exit 1
fi

rustfmt_output="$tmpdir/no-cargo-rustfmt.out"
printf '%s\n' '{"tool_input":{"command":"rustfmt --edition 2024 --check tools/hooks/spec-version-pin-suggester/src/main.rs"}}' \
  | bash "$repo_root/tools/hooks/no-cargo-enforcer.sh" >"$rustfmt_output" 2>&1
if [ -s "$rustfmt_output" ]; then
  echo "direct pinned rustfmt must remain allowed by no-cargo advisory" >&2
  cat "$rustfmt_output" >&2
  exit 1
fi

rustc --edition=2024 -D warnings \
  "$repo_root/scripts/ci/assert-agent-hook-runtime-manifest.rs" \
  -o "$tmpdir/assert-agent-hook-runtime-manifest"
OYA_REPO_ROOT="$repo_root" "$tmpdir/assert-agent-hook-runtime-manifest" --json >/dev/null

if rg -n \
  'Preferred drop-in surface: oya git|policy ratchet compatibility|policy-ratchet|route through `oya git`|Top-level subcommands: git|oya-git cutover|migrate plain git/drop-in docs toward oya git|oya[[:space:]]+(git|vcs|gate|verify)|\.\/bin\/oya|bin/oya|oya --help|Oya CLI|oya CLI' \
  "$repo_root/tools/hooks" \
  "$repo_root/tools/hook-bootstrap" \
  "$repo_root/.codex/hooks.json" \
  "$repo_root/.claude/settings.json" \
  "$repo_root/.gemini/settings.json"; then
  echo "active governance hook surface still contains retired wrapper guidance" >&2
  exit 1
fi

if rg -n '_canonical-primitives\.md' \
  "$repo_root/tools/hooks" \
  "$repo_root/tools/hook-bootstrap" \
  "$repo_root/tools/agent-skills/AGENTS.md" \
  "$repo_root/tools/agent-skills/INHERITANCE.md"; then
  echo "active hook guidance still references retired markdown canonical primitives" >&2
  exit 1
fi

echo "governance hook retired VCS/wrapper surface tests passed"
