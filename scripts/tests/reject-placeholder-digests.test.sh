#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmpdir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmpdir"
}
trap cleanup EXIT

mkdir -p "$tmpdir/microservices/demo/iac/k8s/helm" "$tmpdir/crates/demo/tests"

cat >"$tmpdir/microservices/demo/iac/k8s/helm/deployment.yaml" <<'YAML'
image: registry.oyatie.dev/demo@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef
YAML

bash "$repo_root/scripts/reject-placeholder-digests.sh" "$tmpdir" >/dev/null

zeros="$(python3 - <<'PY'
print("0" * 64)
PY
)"
cat >"$tmpdir/microservices/demo/iac/k8s/helm/deployment.yaml" <<YAML
image: registry.oyatie.dev/demo@sha256:${zeros}
YAML
if bash "$repo_root/scripts/reject-placeholder-digests.sh" "$tmpdir" >/dev/null 2>&1; then
  echo "expected all-zero digest to fail" >&2
  exit 1
fi

cat >"$tmpdir/microservices/demo/iac/k8s/helm/deployment.yaml" <<'YAML'
image: registry.oyatie.dev/demo@sha256:1111111111111111111111111111111111111111111111111111111111111111
YAML
if bash "$repo_root/scripts/reject-placeholder-digests.sh" "$tmpdir" >/dev/null 2>&1; then
  echo "expected repeated-one digest to fail" >&2
  exit 1
fi
cat >"$tmpdir/microservices/demo/iac/k8s/helm/deployment.yaml" <<'YAML'
image: registry.oyatie.dev/demo@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef
YAML

cat >"$tmpdir/microservices/demo/iac/k8s/helm/deployment.yaml" <<'YAML'
image: registry.oyatie.dev/demo:release-digest-required
YAML
if bash "$repo_root/scripts/reject-placeholder-digests.sh" "$tmpdir" >/dev/null 2>&1; then
  echo "expected release placeholder tag to fail" >&2
  exit 1
fi
cat >"$tmpdir/microservices/demo/iac/k8s/helm/deployment.yaml" <<'YAML'
image: registry.oyatie.dev/demo@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef
YAML

cat >"$tmpdir/microservices/demo/iac/k8s/helm/deployment.yaml" <<'YAML'
annotations:
  oyatie.com/image-digest: release-signed-image-digest-required
YAML
if bash "$repo_root/scripts/reject-placeholder-digests.sh" "$tmpdir" >/dev/null 2>&1; then
  echo "expected release placeholder annotation to fail" >&2
  exit 1
fi
cat >"$tmpdir/microservices/demo/iac/k8s/helm/deployment.yaml" <<'YAML'
image: registry.oyatie.dev/demo@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef
YAML

cat >"$tmpdir/crates/demo/tests/fixture.txt" <<YAML
merkle-sha256:${zeros}
YAML
bash "$repo_root/scripts/reject-placeholder-digests.sh" "$tmpdir" >/dev/null

echo "reject-placeholder-digests tests passed"
