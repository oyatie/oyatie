# Migration playbook — AWS Route 53 + NS1 Managed DNS → Oyatie `cloud-network-dns`

Audience: a DNS team running AWS Route 53 as the primary authoritative DNS provider and NS1 for advanced traffic-management
(Pulsar latency-routing, multi-region health-checks). Goal: migrate to `cloud-network-dns` without TTL-based outage windows.

## Phase 0 — Inventory (Day 0…5)

### From Route 53

1. List all hosted zones:
   ```bash
   aws route53 list-hosted-zones > route53-zones.json
   ```
2. For each zone, export records:
   ```bash
   jq -r '.HostedZones[].Id' route53-zones.json | while read zid; do
     aws route53 list-resource-record-sets --hosted-zone-id "$zid" > "rrset-${zid#/hostedzone/}.json"
   done
   ```
3. Export health-checks + their attached records:
   ```bash
   aws route53 list-health-checks > route53-hc.json
   ```
4. Export DNSSEC status per zone:
   ```bash
   jq -r '.HostedZones[].Id' route53-zones.json | while read zid; do
     aws route53 get-dnssec --hosted-zone-id "$zid" > "dnssec-${zid#/hostedzone/}.json"
   done
   ```

### From NS1

1. Export zones + records:
   ```bash
   ns1-cli zone list > ns1-zones.json
   jq -r '.[].zone' ns1-zones.json | while read z; do
     ns1-cli zone export "$z" --format bind > "ns1-$z.bind"
   done
   ```
2. Export monitoring (health-checks) and Pulsar configs:
   ```bash
   ns1-cli monitor list > ns1-monitors.json
   ns1-cli pulsar list > ns1-pulsar.json
   ```

## Phase 1 — Tenant + zone shadow import (Day 5…14)

```bash
./bin/oya dns tenant register --tenant oyatie.b2b.midmarket.acme-corp --tenant-class paid
```

Import each Route 53 zone into `cloud-network-dns` as a shadow zone (not yet delegated):
```bash
./bin/oya dns migrate route53-import \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --input rrset-Z3AABBCC.json \
  --shadow

./bin/oya dns migrate ns1-import \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --bind-input ns1-acme-software.io.bind \
  --shadow
```

The migrator translates:
- Route 53 record sets → `cloud-network-dns` records.
- Route 53 health-checks → `cloud-network-dns` health-checks.
- Route 53 geo + latency routing → equivalent `cloud-network-dns` routing policies.
- NS1 Pulsar → latency-routing-with-RUM-correlation (paid tenant_class feature).

Lint:
```bash
./bin/oya dns zone lint --tenant oyatie.b2b.midmarket.acme-corp --zone acme-software.io
```

## Phase 2 — DNSSEC migration (Day 14…28)

If the source zone is DNSSEC-signed, you have two options:

**Option A (preferred): re-sign with new keys.**

1. Pre-publish the new DS at the registrar alongside the old DS (parent zone now lists both KSKs).
2. Enable DNSSEC in `cloud-network-dns` with a new ECDSA-P256 KSK.
3. Wait 24 h for caches to refresh on both DS records.
4. Cut over (Phase 4).
5. Remove old DS from registrar after Phase 5.

**Option B: KSK-import for zero re-validation.**

```bash
./bin/oya dns migrate ksk-import \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --zone acme-software.io \
  --ksk-pkcs11-export ksk-export.p11
```

The KSK material is imported into `cloud-network-dns`'s HSM (paid-only at paid; paid tenant_class allows software-key import). The chain
of trust at the parent is preserved.

## Phase 3 — Dual-NS shadow phase (Day 28…42)

For each zone, configure both NS sets at the registrar:
```
acme-software.io NS ns-123.awsdns-12.org (Route 53)
acme-software.io NS ns-456.awsdns-34.com (Route 53)
acme-software.io NS ns1.oyatie.dns.net   (cloud-network-dns)
acme-software.io NS ns2.oyatie.dns.net   (cloud-network-dns)
```

DNS clients will round-robin across all 4 NS records (per RFC 1035). `cloud-network-dns` mirrors every Route 53 change in
shadow mode for the duration.

Run divergence telemetry:
```bash
./bin/oya dns migrate divergence-report \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --zone acme-software.io \
  --since "24h ago"
```

Expected: < 0.1 % divergence (acceptable variance from health-check timing differences).

## Phase 4 — Cut-over (Day 42…56)

1. Reduce all record TTLs to 60 s 24 h before cut-over.
2. Remove Route 53 NS records from registrar:
   ```
   acme-software.io NS ns1.oyatie.dns.net
   acme-software.io NS ns2.oyatie.dns.net
   acme-software.io NS ns3.oyatie.dns.net
   acme-software.io NS ns4.oyatie.dns.net
   ```
3. Keep Route 53 zones populated (do not delete) — safety net.
4. Monitor query traffic in `cloud-network-dns`:
   ```bash
   ./bin/oya dns query-log query --tenant oyatie.b2b.midmarket.acme-corp --since "1h ago" --group-by source_pop
   ```
5. After 24 h with > 95 % of traffic on Oyatie NS, restore standard TTLs.

## Phase 5 — Decommission Route 53 + NS1 (Day 56+)

After 30 d clean run:
- Delete Route 53 zones (`aws route53 delete-hosted-zone --id ...`).
- Disable NS1 zones.
- Cancel NS1 subscription.

## Rollback strategy

Within Phase 3 dual-NS:
- Just remove the Oyatie NS records from the registrar; Route 53/NS1 remain authoritative.
- Cost: TTL window for stragglers (~24 h at TTL=60s).

After Phase 4 cut-over:
- Re-add Route 53/NS1 NS records to registrar (mix-mode).
- Roll back the registrar to pre-Phase-4 state.
- Plan: 1-2 h, plus 24-48 h global cache invalidation.

## What you gain

- DoQ + ODoH GA at paid (vendors mostly don't ship these).
- HSM-bound DNSSEC signing at paid.
- Cedar policy authority + audit-chain anchoring.
- City-level geo + RUM-correlated latency steering bundled (NS1 Pulsar equivalent).
- HTTP/3 default for DoH.
- Per-tenant compliance pack overlays.
- Multi-zone tenant scoping with single API.

## What you give up

- AWS Route 53's $5-130/mo pricing vs `cloud-network-dns`'s $640/mo for the paid tenant_class; only justify at scale or sovereign requirements.
- Route 53's 100+ AWS-native integration depth.
- NS1 Pulsar's RUM-correlation maturity (we ship a similar mechanism but with less production tuning).
- Public self-service signup; you need a tenant + tier.
