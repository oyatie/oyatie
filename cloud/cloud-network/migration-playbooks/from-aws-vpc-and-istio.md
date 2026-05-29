# Migration playbook — AWS VPC + Istio service mesh → Oyatie `cloud-network`

Audience: a network/platform team running production workloads on AWS VPC with Istio (or Linkerd) service mesh for east-west
mTLS + L7 policy. Goal: migrate to `cloud-network` without service interruption and with policy parity.

## Phase 0 — Inventory (Day 0…7)

### AWS VPC inventory

1. Export VPCs + subnets + route tables + NACLs + security groups:
   ```bash
   for r in us-east-1 us-east-2 eu-west-1; do
     aws ec2 describe-vpcs --region $r > "vpcs-$r.json"
     aws ec2 describe-subnets --region $r > "subnets-$r.json"
     aws ec2 describe-route-tables --region $r > "routes-$r.json"
     aws ec2 describe-network-acls --region $r > "nacls-$r.json"
     aws ec2 describe-security-groups --region $r > "sgs-$r.json"
   done
   ```
2. Export TGW attachments + Direct peers:
   ```bash
   aws ec2 describe-transit-gateways > tgw.json
   aws directconnect describe-connections > dx.json
   ```
3. Export NAT Gateways + per-NAT throughput:
   ```bash
   aws ec2 describe-nat-gateways > nat-gateways.json
   ```

### Istio inventory

1. Export AuthorizationPolicies + PeerAuthentications:
   ```bash
   kubectl get authorizationpolicy -A -o yaml > istio-authz.yaml
   kubectl get peerauthentication -A -o yaml > istio-peerauth.yaml
   ```
2. Export VirtualServices + DestinationRules:
   ```bash
   kubectl get virtualservice -A -o yaml > istio-vs.yaml
   kubectl get destinationrule -A -o yaml > istio-dr.yaml
   ```
3. Export Gateway resources + cert-manager Certificates.

## Phase 1 — Tenant + VPC provisioning (Day 7…14)

Choose a CIDR that preserves your on-prem hybrid connectivity space. If your AWS VPC is `10.40.0.0/16` and your on-prem is
`192.168.0.0/16`, you can either match or pick a fresh CIDR:
```bash
./bin/oya network vpc create \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --region us-east-2 \
  --cidr 10.140.0.0/20 \
  --availability-zones 3 \
  --enable-ipv6 true \
  --hybrid-on-prem-cidr 192.168.0.0/16
```

`cloud-network` provisions the underlying AWS VPC via `cloud-iac` + Crossplane; you can see it in the AWS console as a normal VPC,
but its policy + mesh are managed via `cloud-network`.

## Phase 2 — Istio AuthorizationPolicy → Cedar translation (Day 14…35)

Translate each Istio AuthorizationPolicy to a Cedar policy:
```bash
./bin/oya network migrate istio-authz-to-cedar \
  --input istio-authz.yaml \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --output policies/acme/cedar-from-istio/
```

The migrator handles ~90 % of cases; the rest are flagged with `_translation_note` for manual review. Common manual cases:
- Istio `rules.from.source.notRequestPrincipals` — Cedar uses `principal !in ...` patterns; sometimes needs splitting into two policies.
- Istio `rules.when.key: request.headers[...]` with regex — Cedar supports `like` patterns but with different syntax.

Lint + push (to a shadow policy set):
```bash
./bin/oya policy lint --tenant oyatie.b2b.midmarket.acme-corp policies/acme/cedar-from-istio/
./bin/oya network policy push --tenant oyatie.b2b.midmarket.acme-corp --policy-dir policies/acme/cedar-from-istio/ --mode shadow
```

In shadow mode, the policy is evaluated on every flow but only logged — no enforcement. Run for 7-14 d to validate translation
parity.

## Phase 3 — Dual-mesh phase (Day 35…56)

For each service, configure dual-mesh:
1. Keep Istio sidecar running (legacy mesh path).
2. Enable Cilium L7 enforcement (new mesh path).
3. Both meshes terminate mTLS; the inner application sees plain HTTP.

This costs ~6-8 % CPU per pod because of double-encryption — accept the cost during migration.

Compare policy decisions between meshes:
```bash
./bin/oya network migrate mesh-decision-divergence \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --legacy istio \
  --since "24h ago"
```

Goal: zero divergence beyond expected (some Istio policies have permit-by-default semantics that Cedar makes explicit).

## Phase 4 — Cilium-only cut-over per service (Day 56…84)

Per service, remove the Istio sidecar:
```bash
kubectl label namespace acme-product istio-injection=disabled --overwrite
kubectl rollout restart deployment -n acme-product
```

Verify:
```bash
./bin/oya network service status --tenant oyatie.b2b.midmarket.acme-corp --service product-svc
```

Expected: `mesh: cilium-only, mtls: spiffe-svid, throughput: 22 Gbps p95`.

Move policy from shadow to enforce:
```bash
./bin/oya network policy push --tenant oyatie.b2b.midmarket.acme-corp --policy-dir policies/acme/cedar-from-istio/ --mode enforce
```

## Phase 5 — Decommission Istio (Day 84…112)

After all services migrated + 30 d clean run:
1. Remove Istio control plane (`istioctl uninstall --purge`).
2. Reclaim sidecar resource budget (~6-8 % per pod).
3. Cancel any Istio-vendor support contract.

## Phase 6 — Decommission AWS Network Firewall / WAF / NAT Gateway (Day 112+)

Where `cloud-network`'s built-in policy + ingress + NAT replace AWS-specific services:
1. Disable AWS Network Firewall (saves ~$0.395/h/endpoint + per-GB processing).
2. Disable AWS WAF (saves $5/web-ACL/mo + $1/rule/mo).
3. Keep AWS NAT Gateway only for non-Oyatie workloads; `cloud-network` provisions its own NAT.

## Rollback strategy

Within Phase 3 dual-mesh:
- Set `cloud-network` policies back to shadow mode.
- Istio remains authoritative; cost is double-mesh CPU only.

After Phase 4 cilium-only:
- Re-enable Istio sidecar injection (`istio-injection=enabled`).
- Roll back deployments.
- Set Cedar policies back to shadow.
- Plan: 2-4 h per service.

After Phase 5 decommission: rollback requires reinstalling Istio. Plan 1-2 d.

## What you gain

- 35-54 % TCO reduction vs AWS VPC + TGW + Shield + WAF + GA combo.
- 70 % more pod throughput vs Istio sidecar mesh (22 Gbps vs 7.6 Gbps at FIPS-off).
- 20-40× faster per-packet policy decisions (1.4 µs vs 64 µs).
- Bundled mesh + policy + NAT + mTLS — no per-component vendor licensing.
- HTTP/3 default for ingress.
- Cedar-policy authority (portable across clouds + on-prem).
- BLAKE3 audit chain for flow logs.

## What you give up

- AWS-native PrivateLink ergonomics for AWS-only customers.
- Mature Istio observability ecosystem (Kiali, Jaeger pre-wired) — Oyatie surfaces `observability` µservice instead.
- F5 / Palo Alto / Check Point appliance integrations (Oyatie's adapter library is at v1; smaller catalog).
- AWS Marketplace one-click installs.
