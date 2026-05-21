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
    "oya-governance-protection-context-match"
  ]
}
JSON

jq '{contexts: .required_status_checks.contexts}' \
  infra/branch-protection/dev.json > "$tmp_dir/live-match.json"

cat > "$tmp_dir/bin/gh" <<'EOF_GH'
#!/usr/bin/env bash
set -euo pipefail
case "${OYA_TEST_GH_MODE:-missing}" in
  missing|match)
    if [ "$1" = "repo" ] && [ "${2:-}" = "view" ]; then
      echo "jason931225/oyatie"
      exit 0
    fi
    if [ "$1" = "api" ] && [[ "${2:-}" == repos/*/branches/dev/protection/required_status_checks ]]; then
      if [ -n "${OYA_EXPECT_BRANCH_PROTECTION_TOKEN:-}" ] && [ "${GH_TOKEN:-}" != "$OYA_EXPECT_BRANCH_PROTECTION_TOKEN" ]; then
        echo "expected branch-protection API to use GH_TOKEN=${OYA_EXPECT_BRANCH_PROTECTION_TOKEN}, got ${GH_TOKEN:-<unset>}" >&2
        exit 42
      fi
      cat "${OYA_TEST_LIVE_CONTEXTS:?}"
      exit 0
    fi
    if [ "$1" = "pr" ] && [ "${2:-}" = "list" ]; then
      if [ -n "${OYA_EXPECT_DEFAULT_TOKEN:-}" ] && [ "${GH_TOKEN:-}" != "$OYA_EXPECT_DEFAULT_TOKEN" ]; then
        echo "expected PR API to keep default GH_TOKEN=${OYA_EXPECT_DEFAULT_TOKEN}, got ${GH_TOKEN:-<unset>}" >&2
        exit 43
      fi
      echo "[]"
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
chmod +x "$tmp_dir/bin/gh"

run_automerge_fail_closed() {
  local mode="$1"
  local live_contexts="$2"
  local out="$tmp_dir/${mode}.out"
  local err="$tmp_dir/${mode}.err"
  set +e
  PATH="$tmp_dir/bin:$PATH" \
    OYA_TEST_GH_MODE="$mode" \
    OYA_TEST_LIVE_CONTEXTS="$live_contexts" \
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

run_automerge_success() {
  local mode="$1"
  local live_contexts="$2"
  local out="$tmp_dir/${mode}.out"
  local err="$tmp_dir/${mode}.err"
  PATH="$tmp_dir/bin:$PATH" \
    OYA_TEST_GH_MODE="$mode" \
    OYA_TEST_LIVE_CONTEXTS="$live_contexts" \
    scripts/trigger-next-queue-automerge.sh --base-ref HEAD --dry-run >"$out" 2>"$err"
}

run_automerge_success_split_token() {
  local out="$tmp_dir/split-token.out"
  local err="$tmp_dir/split-token.err"
  PATH="$tmp_dir/bin:$PATH" \
    GH_TOKEN="write-token" \
    OYA_BRANCH_PROTECTION_READ_TOKEN="admin-read-token" \
    OYA_EXPECT_DEFAULT_TOKEN="write-token" \
    OYA_EXPECT_BRANCH_PROTECTION_TOKEN="admin-read-token" \
    OYA_TEST_GH_MODE="match" \
    OYA_TEST_LIVE_CONTEXTS="$tmp_dir/live-match.json" \
    scripts/trigger-next-queue-automerge.sh --base-ref HEAD --dry-run >"$out" 2>"$err"
}

run_automerge_fail_closed missing "$tmp_dir/live-missing.json"
grep -Fq "live branch-protection required contexts drift" "$tmp_dir/missing.err"
grep -Fq "missing_from_live=" "$tmp_dir/missing.err"
grep -Fq "oya-pr-review" "$tmp_dir/missing.err"

run_automerge_fail_closed forbidden "$tmp_dir/live-missing.json"
grep -Fq "Administration read permission" "$tmp_dir/forbidden.err"
grep -Fq "Resource not accessible by integration" "$tmp_dir/forbidden.err"

run_automerge_success match "$tmp_dir/live-match.json"
grep -Fq "live branch-protection required contexts match" "$tmp_dir/match.out"
grep -Fq "no open PR remains at or after queue floor #1" "$tmp_dir/match.out"

run_automerge_success_split_token
grep -Fq "live branch-protection required contexts match" "$tmp_dir/split-token.out"
grep -Fq "no open PR remains at or after queue floor #1" "$tmp_dir/split-token.out"

printf 'trigger-next-queue-automerge required-context drift guard tests passed\n'
