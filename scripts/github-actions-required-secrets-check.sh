#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/github-actions-required-secrets-check.sh [--repo OWNER/REPO] [--branch BRANCH] [--config PATH] [--secret NAME] [--branch-protection-check-script PATH]

Fail closed when a required GitHub Actions secret is missing before an agent pushes
a PR branch. This is the local preflight counterpart to the hosted
oya-governance-protection-context-match workflow.

Default secret: OYA_BRANCH_PROTECTION_READ_TOKEN
Default branch: dev
Default config: infra/branch-protection/dev.json

The script is read-only. It checks secret visibility with `gh secret list`, then
delegates live required-status-check drift detection to scripts/branch-protection-apply.sh --check.
USAGE
}

repo="${GITHUB_REPOSITORY:-}"
branch="dev"
config="infra/branch-protection/dev.json"
secret_name="OYA_BRANCH_PROTECTION_READ_TOKEN"
branch_protection_check_script="scripts/branch-protection-apply.sh"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo)
      repo="${2:-}"
      shift 2
      ;;
    --branch)
      branch="${2:-}"
      shift 2
      ;;
    --config)
      config="${2:-}"
      shift 2
      ;;
    --secret)
      secret_name="${2:-}"
      shift 2
      ;;
    --branch-protection-check-script)
      branch_protection_check_script="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "required command not found: $1" >&2
    exit 127
  fi
}

require_cmd gh
require_cmd jq

if [[ -z "$repo" ]]; then
  repo="$(gh repo view --json nameWithOwner --jq '.nameWithOwner' 2>/dev/null || true)"
fi
if [[ -z "$repo" ]]; then
  echo "::error::could not determine GitHub repository; pass --repo OWNER/REPO or set GITHUB_REPOSITORY" >&2
  exit 64
fi

if [[ -z "$secret_name" ]]; then
  echo "::error::secret name cannot be empty" >&2
  exit 64
fi

if [[ ! -x "$branch_protection_check_script" ]]; then
  echo "::error::branch-protection check script is not executable: $branch_protection_check_script" >&2
  exit 66
fi

secrets_json="$(gh secret list --repo "$repo" --app actions --json name 2>/tmp/oya-gh-secret-list-error.$$)" || {
  status=$?
  echo "::error::could not list GitHub Actions secrets for $repo; required to prove ${secret_name} exists before push" >&2
  cat "/tmp/oya-gh-secret-list-error.$$" >&2 || true
  rm -f "/tmp/oya-gh-secret-list-error.$$"
  exit "$status"
}
rm -f "/tmp/oya-gh-secret-list-error.$$"

if ! jq -e --arg secret "$secret_name" 'any(.[]?; .name == $secret)' >/dev/null <<<"$secrets_json"; then
  echo "::error::required GitHub Actions secret ${secret_name} is not visible for ${repo}; oya-governance-protection-context-match will fail before evaluating branch-protection drift" >&2
  exit 1
fi

"$branch_protection_check_script" --check --repo "$repo" --branch "$branch" --config "$config"
