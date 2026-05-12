#!/usr/bin/env bash
set -euo pipefail

# ADR-0039 execution lane for release-capable CI runners. The local
# `oya gate validate supply-chain --require-adr0039-evidence` command verifies
# this script is wired. Release runners execute it only after artifacts/images
# exist, then run `oya gate validate release-supply-chain --phase release`
# against signed evidence records before publishing.

manifest=${1:-registry/release/images.yaml}
artifacts_dir=${OYA_SUPPLY_CHAIN_ARTIFACTS_DIR:-artifacts/supply-chain}
rekor_url=${OYA_REKOR_URL:-https://rekor.sigstore.dev}
issuer=${OYA_COSIGN_OIDC_ISSUER:-https://token.actions.githubusercontent.com}
identity_regexp=${OYA_COSIGN_IDENTITY_REGEXP:-https://github.com/.+/.+/.github/workflows/.+@refs/tags/v.+}

need_tool() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required ADR-0039 tool: $1" >&2
    exit 127
  fi
}

release_images() {
  awk '
    /^[[:space:]]*#/ { next }
    /^[[:space:]]*-[[:space:]]*ref:[[:space:]]*/ {
      sub(/^[[:space:]]*-[[:space:]]*ref:[[:space:]]*/, "")
      gsub(/["\047]/, "")
      print
      next
    }
    /^[[:space:]]*ref:[[:space:]]*/ {
      sub(/^[[:space:]]*ref:[[:space:]]*/, "")
      gsub(/["\047]/, "")
      print
      next
    }
    /^[[:space:]]*-[[:space:]]*[[:alnum:]_.\/:@-]+[[:space:]]*$/ {
      sub(/^[[:space:]]*-[[:space:]]*/, "")
      print
    }
  ' "$manifest"
}

need_tool trivy
need_tool cosign

if [[ ! -f "$manifest" ]]; then
  echo "release image manifest not found: $manifest" >&2
  exit 66
fi

mkdir -p "$artifacts_dir/sbom"

trivy fs --severity HIGH,CRITICAL --exit-code 1 .
trivy config --severity HIGH,CRITICAL --exit-code 1 infra/
trivy fs --scanners vuln,secret,license --format sarif --output "$artifacts_dir/trivy.sarif" .
trivy fs --format spdx-json --output "$artifacts_dir/sbom/oyatie.spdx.json" .
trivy fs --format cyclonedx --output "$artifacts_dir/sbom/oyatie.cyclonedx.json" .

mapfile -t images < <(release_images | sed '/^[[:space:]]*$/d')
if [[ ${#images[@]} -eq 0 ]]; then
  echo "release image manifest has no image refs: $manifest" >&2
  exit 65
fi

for image in "${images[@]}"; do
  trivy image --severity HIGH,CRITICAL --exit-code 1 "$image"
  cosign sign --yes "$image"
  cosign verify \
    --rekor-url "$rekor_url" \
    --certificate-oidc-issuer "$issuer" \
    --certificate-identity-regexp "$identity_regexp" \
    "$image"
  cosign attest --yes --predicate "$artifacts_dir/sbom/oyatie.spdx.json" --type spdx "$image"
  cosign attest --yes --predicate "$artifacts_dir/sbom/oyatie.cyclonedx.json" --type cyclonedx "$image"
  cosign attest --yes --predicate "$artifacts_dir/trivy.sarif" --type vuln "$image"
done
