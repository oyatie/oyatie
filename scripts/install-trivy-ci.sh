#!/usr/bin/env bash
set -euo pipefail

# Install the pinned Trivy release used by Oyatie's provider-execution and
# supply-chain lanes. The checksum verification is intentionally local to this
# script so every workflow lane gets the same supply-chain posture.

TRIVY_VERSION="0.70.0"
TRIVY_ARCHIVE="trivy_${TRIVY_VERSION}_Linux-64bit.tar.gz"
BASE_URL="https://github.com/aquasecurity/trivy/releases/download/v${TRIVY_VERSION}"
INSTALL_DIR="/usr/local/bin"

tmpdir=$(mktemp -d)
cleanup() { rm -rf "$tmpdir"; }
trap cleanup EXIT

cd "$tmpdir"
curl --fail --location --silent --show-error --output "$TRIVY_ARCHIVE" "$BASE_URL/$TRIVY_ARCHIVE"
curl --fail --location --silent --show-error --output trivy_checksums.txt "$BASE_URL/trivy_${TRIVY_VERSION}_checksums.txt"
grep -E "[ *]${TRIVY_ARCHIVE}$" trivy_checksums.txt > selected-checksum.txt
sha256sum -c selected-checksum.txt
tar -xzf "$TRIVY_ARCHIVE" trivy
if [[ -w "$INSTALL_DIR" ]]; then
  install -m 0755 trivy "$INSTALL_DIR/trivy"
else
  sudo install -m 0755 trivy "$INSTALL_DIR/trivy"
fi
trivy --version
