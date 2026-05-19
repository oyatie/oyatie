#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
script="$repo_root/scripts/github-actions-required-secrets-check.sh"
tmpdir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmpdir"
}
trap cleanup EXIT

fake_bin="$tmpdir/bin"
mkdir -p "$fake_bin"

write_fake_gh() {
  local secrets_json="$1"
  cat > "$fake_bin/gh" <<SH
#!/usr/bin/env bash
set -euo pipefail
if [[ "\$1 \$2" == "secret list" ]]; then
  cat <<'JSON'
$secrets_json
JSON
  exit 0
fi
if [[ "\$1 \$2" == "repo view" ]]; then
  echo "jason931225/oyatie"
  exit 0
fi
echo "unexpected gh invocation: \$*" >&2
exit 99
SH
  chmod +x "$fake_bin/gh"
}

branch_check="$tmpdir/branch-protection-check.sh"
cat > "$branch_check" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" > "${BRANCH_CHECK_ARGS:?}"
SH
chmod +x "$branch_check"

export PATH="$fake_bin:$PATH"
export BRANCH_CHECK_ARGS="$tmpdir/branch-check-args.txt"

write_fake_gh '[{"name":"OYA_BRANCH_PROTECTION_READ_TOKEN"}]'
"$script" \
  --repo jason931225/oyatie \
  --branch dev \
  --config infra/branch-protection/dev.json \
  --branch-protection-check-script "$branch_check"
grep -q -- '--check --repo jason931225/oyatie --branch dev --config infra/branch-protection/dev.json' "$BRANCH_CHECK_ARGS"

write_fake_gh '[]'
if "$script" --repo jason931225/oyatie --branch-protection-check-script "$branch_check" 2>"$tmpdir/missing.err"; then
  echo "expected missing secret to fail" >&2
  exit 1
fi
grep -q 'required GitHub Actions secret OYA_BRANCH_PROTECTION_READ_TOKEN is not visible' "$tmpdir/missing.err"

write_fake_gh '[{"name":"OYA_BRANCH_PROTECTION_READ_TOKEN"}]'
cat > "$branch_check" <<'SH'
#!/usr/bin/env bash
echo "drift remains" >&2
exit 7
SH
chmod +x "$branch_check"
if "$script" --repo jason931225/oyatie --branch-protection-check-script "$branch_check" 2>"$tmpdir/drift.err"; then
  echo "expected branch-protection drift to fail" >&2
  exit 1
fi

echo "github-actions-required-secrets-check tests passed"
