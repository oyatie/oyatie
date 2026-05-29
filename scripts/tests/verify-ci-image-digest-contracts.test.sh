#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmpdir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmpdir"
}
trap cleanup EXIT

mkdir -p "$tmpdir/microservices/demo/ci"

cat >"$tmpdir/microservices/demo/ci/Jenkinsfile" <<'GROOVY'
String requireRealCiImageDigest(String variableName) {
  String digest = env[variableName]?.trim()
  if (!digest) {
    throw new IllegalStateException("${variableName} must be set before rendering the Jenkins Kubernetes agent pod")
  }
  if (!(digest ==~ /^sha256:[0-9a-f]{64}$/) || digest ==~ /^sha256:0{64}$/) {
    throw new IllegalStateException("${variableName} must be a real non-zero sha256 digest")
  }
  return digest
}

pipeline {
  agent {
    kubernetes {
      yaml """
containers:
  - name: rust
    image: registry.oyatie.dev/ci/rust:stable@${requireRealCiImageDigest('OYA_CI_RUST_IMAGE_DIGEST')}
"""
    }
  }
}
GROOVY
bash "$repo_root/scripts/verify-ci-image-digest-contracts.sh" "$tmpdir" >/dev/null


cat >"$tmpdir/microservices/demo/ci/Jenkinsfile" <<'GROOVY'
String requireRealCiImageDigest(String variableName) {
  String digest = env[variableName]?.trim()
  if (!digest) {
    throw new IllegalStateException("${variableName} must be set before rendering the Jenkins Kubernetes agent pod")
  }
  if (!(digest ==~ /^sha256:[0-9a-f]{64}$/) || digest ==~ /^sha256:0+$/) {
    throw new IllegalStateException("${variableName} must be a real non-zero sha256 digest")
  }
  return digest
}

pipeline {
  agent {
    kubernetes {
      yaml """
containers:
  - name: rust
    image: registry.oyatie.dev/ci/rust:stable@${requireRealCiImageDigest('OYA_CI_RUST_IMAGE_DIGEST')}
"""
    }
  }
}
GROOVY
if bash "$repo_root/scripts/verify-ci-image-digest-contracts.sh" "$tmpdir" >/dev/null 2>&1; then
  echo "expected zero-digest OR bypass to fail" >&2
  exit 1
fi

cat >"$tmpdir/microservices/demo/ci/Jenkinsfile" <<'GROOVY'
pipeline {
  agent {
    kubernetes {
      yaml """
containers:
  - name: rust
    image: registry.oyatie.dev/ci/rust:stable@${env.OYA_CI_RUST_IMAGE_DIGEST}
"""
    }
  }
}
GROOVY
if bash "$repo_root/scripts/verify-ci-image-digest-contracts.sh" "$tmpdir" >/dev/null 2>&1; then
  echo "expected raw env digest interpolation to fail" >&2
  exit 1
fi

echo "verify-ci-image-digest-contracts tests passed"
