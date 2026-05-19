#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

mkdir -p "$tmp_dir/bin"

cat > "$tmp_dir/live-missing.json" <<'JSON'
{
  "strict": false,
  "contexts": [
    "cargo-fmt",
    "cargo-check",
    "cargo-clippy",
    "cargo-nextest",
    "oya-vcs-admission",
    "oya-vcs-provider-execution",
    "oya-foundry-fitness-supply-chain",
    "oya-foundry-fitness-cohesion",
    "oya-foundry-fitness-api-semver",
    "oya-foundry-fitness-protection-context-match"
  ]
}
JSON

cat > "$tmp_dir/gh" <<'EOF_GH'
#!/usr/bin/env bash
set -euo pipefail
case "${OYA_TEST_GH_MODE:-missing}" in
  missing)
    if [ "$1" = "repo" ] && [ "${2:-}" = "view" ]; then
      echo "jason931225/oyatie"
      exit 0
    fi
    if [ "$1" = "api" ] && [[ "${2:-}" == repos/*/branches/dev/protection/required_status_checks ]]; then
      cat "${OYA_TEST_LIVE_CONTEXTS:?}"
      exit 0
    fi
    ;;
  forbidden)
    if [ "$1" = "repo" ] && [ "${2:-}" = "view" ]; then
      echo "jason931225/oyatie"
      exit 0
    fi
    if [ "$1" = "api" ] && [[ "${2:-}" == repos/*/branches/dev/protection/required_status_checks ]]; then
      echo "Resource not accessible by integration" >&2
      exit 1
    fi
    ;;
esac

echo "unexpected gh invocation: $*" >&2
exit 99
EOF_GH
chmod +x "$tmp_dir/gh"

run_automerge() {
  local mode="$1"
  local out="$tmp_dir/${mode}.out"
  local err="$tmp_dir/${mode}.err"
  set +e
  PATH="$tmp_dir/bin:$PATH" \
    OYA_TEST_GH_MODE="$mode" \
    OYA_TEST_LIVE_CONTEXTS="$tmp_dir/live-missing.json" \
    scripts/trigger-next-queue-automerge.sh --base-ref HEAD --dry-run >"$out" 2>"$err"
  status=$?
  set -e
  if [ "$status" -eq 0 ]; then
    echo "expected ${mode} scenario to fail closed before automerge" >&2
    cat "$out" >&2
    cat "$err" >&2
    exit 1
  fi
}

run_automerge missing
grep -Fq "live branch-protection required contexts drift" "$tmp_dir/missing.err"
grep -Fq "missing_from_live=" "$tmp_dir/missing.err"
grep -Fq "oya-pr-review" "$tmp_dir/missing.err"

run_automerge forbidden
grep -Fq "Administration read permission" "$tmp_dir/forbidden.err"
grep -Fq "Resource not accessible by integration" "$tmp_dir/forbidden.err"

printf 'trigger-next-queue-automerge required-context drift guard tests passed\n'
