#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/branch-protection-apply.sh [--check|--apply] [--repo OWNER/REPO] [--branch BRANCH] [--config PATH]

Synchronize GitHub required status-check protection with infra/branch-protection/dev.json.

Default mode is --check: read live branch protection, run the repo gate, and exit non-zero on drift.
--apply performs the GitHub mutation for required_status_checks only, then re-runs --check.

Auth: uses gh CLI auth or GH_TOKEN. GitHub requires Administration read for --check and
Administration write for --apply on branch-protection status-check endpoints.
USAGE
}

mode="check"
repo="${GITHUB_REPOSITORY:-jason931225/oyatie}"
branch="dev"
config="infra/branch-protection/dev.json"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --check)
      mode="check"
      shift
      ;;
    --apply)
      mode="apply"
      shift
      ;;
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
require_cmd cargo

if [[ ! -f "$config" ]]; then
  echo "branch-protection config not found: $config" >&2
  exit 66
fi

config_branch="$(jq -r '.branch // empty' "$config")"
if [[ -n "$config_branch" && "$config_branch" != "$branch" ]]; then
  echo "config branch ($config_branch) does not match requested branch ($branch)" >&2
  exit 65
fi

workdir="$(mktemp -d)"
cleanup() {
  rm -rf "$workdir"
}
trap cleanup EXIT

payload="$workdir/required-status-checks-payload.json"
live="$workdir/live-required-status-checks.json"

jq '{strict: .required_status_checks.strict, contexts: .required_status_checks.contexts}' \
  "$config" > "$payload"

if [[ "$(jq -r '.contexts | length' "$payload")" == "0" ]]; then
  echo "refusing empty required_status_checks.contexts from $config" >&2
  exit 65
fi

fetch_live() {
  gh api "repos/${repo}/branches/${branch}/protection/required_status_checks" \
    --jq '.contexts' > "$live"
}

print_delta() {
  jq -n \
    --slurpfile canonical_doc "$payload" \
    --slurpfile live_doc "$live" \
    '($canonical_doc[0].contexts) as $canonical |
     ($live_doc[0]) as $live |
     {
       canonical: $canonical,
       live: $live,
       missing_from_live: (($canonical - $live) | sort),
       extra_in_live: (($live - $canonical) | sort)
     }'
}

if [[ "$mode" == "apply" ]]; then
  echo "Applying required status-check contexts for ${repo}:${branch} from ${config}" >&2
  gh api --method PATCH \
    "repos/${repo}/branches/${branch}/protection/required_status_checks" \
    --input "$payload" >/dev/null
fi

fetch_live
print_delta
cargo run -q -p oya-dev-cli -- gate validate protection-context-match \
  --branch "$branch" \
  --live-required-contexts "$live"
