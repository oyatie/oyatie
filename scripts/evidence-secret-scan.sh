#!/usr/bin/env bash
# Evidence-secret-scan: fail-on-match scanner for high-signal credential shapes
# inside evidence/multispectrum/ files and evidence/audit-chain.jsonl.
#
# Patterns are intentionally narrow to keep false-positives low:
#   - Anthropic / OpenAI key shape  : sk-[A-Za-z0-9_-]{40,}
#   - AWS access key id              : AKIA[0-9A-Z]{16}
#   - GitHub personal access token   : ghp_[A-Za-z0-9]{36}
#   - GitHub OAuth / fine-grained    : gho_[A-Za-z0-9]{36}, github_pat_[A-Za-z0-9_]{82}
#   - Slack bot token                : xoxb-[0-9]+-[0-9]+-[A-Za-z0-9]+
#   - PEM private key opening line   : -----BEGIN [A-Z ]+PRIVATE KEY-----
#   - Raw bearer-token header        : Authorization: Bearer <20+ chars>
#   - JWT-like compact serialization : eyJ[A-Za-z0-9_-]{20,}\.eyJ[A-Za-z0-9_-]{20,}\.
#
# Exit 0 on no match (clean); exit 1 if any pattern matches in any target path.
# Stdout shows the path + line of each match so the PR author can locate and
# redact. No automatic redaction is performed.
#
# Invocation: scripts/evidence-secret-scan.sh <path1> [<path2> ...]
# Targets are scanned recursively if they are directories.
#
# F-EVIDENCE-SECRET-SCAN (high; created 2026-05-14) → resolved 2026-05-17.

set -eu

if [ "$#" -eq 0 ]; then
  echo "usage: $0 <evidence-path> [<evidence-path> ...]" >&2
  exit 2
fi

PATTERNS=(
  # Anthropic/OpenAI key shape; require word-boundary before `sk-` to avoid
  # matching path-like tokens such as `cs-high-risk-...` or `risk-classification-...`
  # whose internal `sk-` substring would otherwise trigger a false positive.
  '(^|[^A-Za-z0-9_-])sk-[A-Za-z0-9_-]{40,}'
  'AKIA[0-9A-Z]{16}'
  'ghp_[A-Za-z0-9]{36}'
  'gho_[A-Za-z0-9]{36}'
  'github_pat_[A-Za-z0-9_]{82}'
  'xoxb-[0-9]+-[0-9]+-[A-Za-z0-9]+'
  '-----BEGIN [A-Z ]+PRIVATE KEY-----'
  'Authorization:[[:space:]]+Bearer[[:space:]]+[A-Za-z0-9._-]{20,}'
  'eyJ[A-Za-z0-9_-]{20,}\.eyJ[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]+'
)

JOINED_PATTERN="$(printf '%s|' "${PATTERNS[@]}")"
JOINED_PATTERN="${JOINED_PATTERN%|}"

found=0
for target in "$@"; do
  if [ ! -e "$target" ]; then
    echo "evidence-secret-scan: target not found: $target (skipping)" >&2
    continue
  fi
  if [ -d "$target" ]; then
    match_output="$(grep -rEn "$JOINED_PATTERN" "$target" || true)"
  else
    match_output="$(grep -En "$JOINED_PATTERN" "$target" || true)"
  fi
  if [ -n "$match_output" ]; then
    echo "evidence-secret-scan: matches in $target:"
    printf '%s\n' "$match_output"
    found=1
  fi
done

if [ "$found" -ne 0 ]; then
  echo "evidence-secret-scan: FAIL — credential-shape tokens detected; redact and re-commit" >&2
  exit 1
fi

echo "evidence-secret-scan: PASS — no credential-shape tokens detected across $# target(s)"
