---
doc_class: MultiRegion
template_id: TPL-MULTI-REGION
microservice: identity
status: Accepted
date: 2026-05-18
owner_team: axis-identity + ops-sre-reliability
related_adrs: [ADR-0152, ADR-0158, ADR-0171, ADR-0179, ADR-0187]
---

# Multi-Region — identity µservice

## Posture

Per ADR-0179 sovereign-cloud-per-regional-pack, identity is **per-pack active-active within a pack region** and **NEVER cross-pack**. There is no global IdP. Each pack has its own Zitadel Instance, its own Postgres event-store, its own user/credential database, its own JWKS, and its own audit-chain replica.

This is the same posture Google ran for years (Workspace per-region IAM) and that Microsoft runs for Entra ID Sovereign Cloud. Sovereign customers refuse to allow their citizens' authentication state to traverse the boundary; the right architecture is per-pack isolation, not cross-region replication.

## Within-pack active-active

Each pack runs Zitadel across ≥3 availability zones in its primary region:

| Pack | Primary region | AZ count | Failover region |
|---|---|---|---|
| pack-kr | KR-Seoul (3 AZs) | 3 | KR-Busan (PITR + warm standby) |
| pack-eu | EU-Frankfurt (3 AZs) | 3 | EU-Dublin (PITR + warm standby) |
| pack-us | US-East (3 AZs) | 3 | US-West (PITR + warm standby) |
| pack-us-healthcare | US-East HIPAA (3 AZs) | 3 | US-West HIPAA (PITR + warm standby) |
| pack-jp | JP-Tokyo (3 AZs) | 3 | JP-Osaka |
| pack-sg | SG-Singapore (3 AZs) | 3 | none (single region MOR per pack-sg charter) |
| pack-au | AU-Sydney (3 AZs) | 3 | AU-Melbourne |
| pack-in | IN-Mumbai (3 AZs) | 3 | IN-Hyderabad |
| pack-br | BR-São Paulo (3 AZs) | 3 | BR-Rio |
| pack-ae | AE-Dubai (3 AZs) | 3 | AE-Abu Dhabi |
| pack-ksa | KSA-Riyadh (3 AZs) | 3 | none (sovereign single-region) |

## Replication topology within a pack

```
                    [Envoy Gateway edge]
                          |
                          v
                   [Istio waypoint]
                          |
                  +-------+-------+
                  v       v       v
            [Zitadel] [Zitadel] [Zitadel]   (3 replicas across 3 AZs)
                  |       |       |
                  +-------+-------+
                          |
                   [pgcat connection pool]
                          |
                  +-------+-------+
                  v       v       v
          [Pg primary] [Pg replica] [Pg replica]  (1 primary + 2 replicas, sync)
                  |
                  v
            [WAL archive to S3-compatible per-pack bucket]
                  |
                  v
            [Warm standby in failover region — PITR-restorable]
```

Postgres replication is **synchronous** within the primary region (zero RPO between AZs) and **asynchronous** to the failover region (RPO ≤30s).

## JWKS replication

JWKS is read-only and idempotent. Each Zitadel replica serves the same JWKS (eventually consistent via Postgres replication). Consumers cache JWKS for 24h with respect-Cache-Control honored; rotation is signalled by adding a new `kid` to the JWKS doc and the old `kid` remaining for a 24h grace.

Per-pack JWKS endpoint URL pattern: `https://identity-<pack>.oyatie.com/oauth/v2/keys`. There is no global `https://identity.oyatie.com` endpoint; consumer Cedar policy refuses bearer tokens whose `iss` claim does not match the consumer's own pack.

## SCIM provisioning per-region

SCIM endpoint per pack: `https://identity-<pack>.oyatie.com/scim/v2/<tenant>`. Enterprise IdPs (Okta / Entra / Workspace) configure one SCIM connection per pack-tenant.

A tenant that operates in two packs (e.g., a US multinational with an EU subsidiary) registers TWO SCIM connections — one to pack-us, one to pack-eu. User records in pack-us and pack-eu are independent; the same `email` is allowed (different scope per pack).

## Active-active failover scenarios

### Scenario 1 — AZ failure within a pack

- 1 of 3 AZs in primary region fails.
- Zitadel replicas in that AZ go down; the other 2 AZs continue serving.
- Postgres primary may failover to a replica in another AZ (Patroni-driven; ≤30s).
- RTO: ≤30s (within RPO=0 budget per ADR-0152 realtime tier).
- Detection: `zitadel-instance-health` SLO target 0.9999.
- Runbook: `identity-az-failover` (automatic; runbook-as-code).

### Scenario 2 — Primary region partial outage

- Primary region degraded; multiple AZs unstable.
- Promote warm-standby in failover region to read-write.
- DNS update (pack-routing): `identity-<pack>.oyatie.com` weighted-to failover region.
- Sessions stay valid (JWT signature still verifies; JWKS replicated); new sign-ins go to failover.
- RTO: ≤5min (DNS propagation + warm-standby promotion).
- RPO: ≤30s (async replication lag).
- Runbook: `identity-region-failover` (operator-mediated).

### Scenario 3 — Complete pack region down (cataclysm)

- Both primary and failover regions of a pack offline.
- Sovereign rule forbids cross-pack failover; this is "wait for region recovery" by design.
- Communication: status page (ADR-0168) shows the pack outage.
- Customers receive incident notice; sessions expire per ACR policy.
- Recovery: when region returns, replay from WAL archive in offsite tape.

## Cross-pack tenant operation (legitimate use case)

A multinational tenant operating in pack-us, pack-eu, pack-us-healthcare:

- Three independent oyatie tenant records (`tenant_us`, `tenant_eu`, `tenant_us_hc`).
- Three independent identity provisioning paths (three SCIM bearers).
- The SAME end-user email (alice@acme.com) MAY be provisioned in all three; they are three distinct users from the µservice's perspective.
- The tenant's central IT desk drives this; not the user.

## DNS + routing

Per-pack DNS records:
```
identity-kr.oyatie.dev    A     <pack-kr-vip>
identity-eu.oyatie.dev    A     <pack-eu-vip>
identity-us.oyatie.dev    A     <pack-us-vip>
identity-us-healthcare.oyatie.dev  A  <pack-us-hc-vip>
...
```

Global `identity.oyatie.com` is a 404 page that redirects to a per-pack selector (locale + IP-geo informed).

## Connection draining on rolling deploy

- HPA + PDB ensures ≥2 Zitadel replicas always up during rolling deploy.
- Envoy waypoint drains 30s before pod termination.
- JWT verification remains valid throughout (signing key unchanged unless JWKS rotation event).

## Disaster recovery drill schedule

| Drill | Cadence | Owner |
|---|---|---|
| identity-az-failover | quarterly per pack | ops-sre-reliability |
| identity-region-failover | semi-annually per pack | ops-sre-reliability + axis-identity |
| identity-pack-cataclysm-recovery (tabletop) | annually per pack | council-architecture |
| jwks-rotation-emergency-revoke | quarterly | ops-security |
