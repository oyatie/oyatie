#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

mkdir -p "$tmpdir/good" "$tmpdir/bad" "$tmpdir/good/evidence/vcs" "$tmpdir/good/gateway/adapters/netsuite-connector"
cat > "$tmpdir/good/platform.md" <<'GOOD'
Tenant/RBAC packaging uses entitlement_set_id, test_set_id, and eval_set_id.
Cryptographic cipher suite and C-suite wording are allowed where technically correct.
NetSuite remains an allowed vendor name.
Legitimate cloud platform, developer platform, and platform-approved font policy wording are allowed.
GOOD
cat > "$tmpdir/good/evidence/vcs/cs-ent-suite-historical.json" <<'GOOD'
{"historical":"cs-ent-suite evidence is immutable"}
GOOD
cat > "$tmpdir/good/gateway/adapters/netsuite-connector/README.md" <<'GOOD'
NetSuite adapter keeps vendor spelling.
GOOD
cat > "$tmpdir/good/connector-iac.yaml" <<'GOOD'
name: oya-connector
repository: oya-connector
spiffe: spiffe://oyatie.dev/ns/connector/sa/connector-app
host: connector.oyatie.com
dns: connector.svc.cluster.local
GOOD
"$repo_root/scripts/reject-retired-grouping-wording.sh" "$tmpdir/good" >/tmp/reject-retired-grouping-good.out

cat > "$tmpdir/bad/legacy.yaml" <<'BAD'
product_class: "suite"
suite_id: connect
former_platform: "Platform uses platform_id and platform-app"
former_module: "Module uses module_id"
former_lowercase: "enterprise platform, connect platform, enterprise module, connect product, and connect suite wrappers are retired"
name: oya-connect
repository: oya-connect
spiffe: spiffe://oyatie.dev/ns/connect/sa/connect-webhook-receiver-edge
host: connect.oyatie.app
dns: connect.svc.cluster.local
BAD
if "$repo_root/scripts/reject-retired-grouping-wording.sh" "$tmpdir/bad" >/tmp/reject-retired-grouping-bad.out 2>/tmp/reject-retired-grouping-bad.err; then
  echo 'expected retired grouping wording gate to fail on legacy identifiers' >&2
  exit 1
fi
grep -q 'Retired grouping wording found' /tmp/reject-retired-grouping-bad.err
printf 'reject-retired-grouping-wording tests passed\n'
