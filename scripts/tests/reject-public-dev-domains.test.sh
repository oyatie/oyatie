#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmpdir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmpdir"
}
trap cleanup EXIT

mkdir -p "$tmpdir/specs" "$tmpdir/microservices/demo/iac" "$tmpdir/infra/talos"

cat >"$tmpdir/specs/public.json" <<'JSON'
{"$id": "https://docs.oyatie.com/specs/public.json", "api": "https://api.oyatie.com/v1"}
JSON
bash "$repo_root/scripts/reject-public-dev-domains.sh" "$tmpdir" >/dev/null

cat >"$tmpdir/specs/public.json" <<'JSON'
{"$id": "https://oyatie.dev/specs/public.json"}
JSON
if bash "$repo_root/scripts/reject-public-dev-domains.sh" "$tmpdir" >/dev/null 2>&1; then
  echo "expected root oyatie.dev public schema URL to fail" >&2
  exit 1
fi
cat >"$tmpdir/specs/public.json" <<'JSON'
{"$id": "https://oyatie.com/specs/public.json"}
JSON
bash "$repo_root/scripts/reject-public-dev-domains.sh" "$tmpdir" >/dev/null

cat >"$tmpdir/specs/public.json" <<'JSON'
{"contact": "axis-demo@oyatie.dev"}
JSON
if bash "$repo_root/scripts/reject-public-dev-domains.sh" "$tmpdir" >/dev/null 2>&1; then
  echo "expected public oyatie.dev email to fail" >&2
  exit 1
fi
cat >"$tmpdir/specs/public.json" <<'JSON'
{"contact": "axis-demo@oyatie.com"}
JSON
bash "$repo_root/scripts/reject-public-dev-domains.sh" "$tmpdir" >/dev/null

cat >"$tmpdir/specs/public.json" <<'JSON'
{"$id": "https://docs.oyatie.dev/specs/public.json"}
JSON
if bash "$repo_root/scripts/reject-public-dev-domains.sh" "$tmpdir" >/dev/null 2>&1; then
  echo "expected docs.oyatie.dev to fail" >&2
  exit 1
fi
cat >"$tmpdir/specs/public.json" <<'JSON'
{"$id": "https://docs.oyatie.com/specs/public.json"}
JSON

cat >"$tmpdir/microservices/demo/openapi.yaml" <<'YAML'
servers:
  - url: https://api.oyatie.dev/v1
YAML
if bash "$repo_root/scripts/reject-public-dev-domains.sh" "$tmpdir" >/dev/null 2>&1; then
  echo "expected public api.oyatie.dev to fail" >&2
  exit 1
fi
cat >"$tmpdir/microservices/demo/openapi.yaml" <<'YAML'
servers:
  - url: https://api.oyatie.com/v1
YAML


cat >"$tmpdir/microservices/demo/ech-config.yaml" <<'YAML'
public_name: connector.oyatie.dev
YAML
if bash "$repo_root/scripts/reject-public-dev-domains.sh" "$tmpdir" >/dev/null 2>&1; then
  echo "expected bare public connector.oyatie.dev to fail" >&2
  exit 1
fi
cat >"$tmpdir/microservices/demo/ech-config.yaml" <<'YAML'
public_name: connector.oyatie.com
YAML

cat >"$tmpdir/microservices/demo/identity.md" <<'MD'
JWKS: https://identity-<pack>.oyatie.dev/oauth/v2/keys
MD
if bash "$repo_root/scripts/reject-public-dev-domains.sh" "$tmpdir" >/dev/null 2>&1; then
  echo "expected templated identity-<pack>.oyatie.dev to fail" >&2
  exit 1
fi
cat >"$tmpdir/microservices/demo/identity.md" <<'MD'
JWKS: https://identity-<pack>.oyatie.com/oauth/v2/keys
MD

cat >"$tmpdir/microservices/demo/iac/helm-template.yaml" <<'YAML'
hostnames:
  - identity-{{ .Values.packLabel }}.oyatie.dev
YAML
if bash "$repo_root/scripts/reject-public-dev-domains.sh" "$tmpdir" >/dev/null 2>&1; then
  echo "expected deployable templated identity host to fail" >&2
  exit 1
fi
cat >"$tmpdir/microservices/demo/iac/helm-template.yaml" <<'YAML'
hostnames:
  - identity-{{ .Values.packLabel }}.oyatie.com
YAML

cat >"$tmpdir/microservices/demo/iac/rate-limit.yaml" <<'YAML'
vhost: { name: "identity.oyatie.dev:443" }
YAML
if bash "$repo_root/scripts/reject-public-dev-domains.sh" "$tmpdir" >/dev/null 2>&1; then
  echo "expected global identity.oyatie.dev vhost to fail" >&2
  exit 1
fi
cat >"$tmpdir/microservices/demo/iac/rate-limit.yaml" <<'YAML'
vhost: { name: "identity.oyatie.com:443" }
YAML

cat >"$tmpdir/microservices/demo/iac/gateway.yaml" <<'YAML'
gateway:
  hostnames:
    - messenger-kr.oyatie.dev
YAML
if bash "$repo_root/scripts/reject-public-dev-domains.sh" "$tmpdir" >/dev/null 2>&1; then
  echo "expected deployable Gateway hostnames .dev to fail" >&2
  exit 1
fi
cat >"$tmpdir/microservices/demo/iac/gateway.yaml" <<'YAML'
gateway:
  hostnames:
    - messenger-kr.oyatie.com
YAML

cat >"$tmpdir/microservices/demo/iac/cloud.yaml" <<'YAML'
hostnames:
  - intelligence.oya.cloud
YAML
if bash "$repo_root/scripts/reject-public-dev-domains.sh" "$tmpdir" >/dev/null 2>&1; then
  echo "expected non-owned oya.cloud deployable host to fail" >&2
  exit 1
fi
cat >"$tmpdir/microservices/demo/iac/cloud.yaml" <<'YAML'
hostnames:
  - intelligence.oyatie.com
YAML

cat >"$tmpdir/infra/talos/bootstrap.yaml" <<'YAML'
endpoint: https://join.oyatie.dev/config
YAML
bash "$repo_root/scripts/reject-public-dev-domains.sh" "$tmpdir" >/dev/null

cat >"$tmpdir/microservices/demo/runbook.md" <<'MD'
internal smoke: https://demo.internal.oyatie.dev/health
MD
bash "$repo_root/scripts/reject-public-dev-domains.sh" "$tmpdir" >/dev/null

echo "reject-public-dev-domains tests passed"
