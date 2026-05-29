---
doc_class: MultiRegionPlan
title: Multi-Region Topology + BCDR
microservice: application
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-application + cloud-iac + cloud-k8s
deciders: ops-sre-reliability, axis-application, council-architecture, council-privacy
related_adrs: [ADR-0117, ADR-0123, ADR-0131]
related_artifacts:
  - microservices/application/policy/data-residency.md
  - microservices/application/capacity-model.md
  - microservices/application/cost-budget.md
  - microservices/application/failure-modes.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Multi-Region Topology + BCDR (application µservice)

## Purpose

Define the multi-region topology for the Application Shell across the 11
oyatie packs, the in-pack DR-pair posture, cross-pack replication-forbidden
policy, BCDR controls, RPO/RTO targets, failover procedures, and DNS +
cert orchestration. This is the authoritative reference for
ops-sre-reliability on-call during region outages and for auditors
verifying business-continuity claims.

## Topology Per Pack

| Pack | Primary region | DR pair (warm-standby) | Single-region? | Activation |
|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | — | YES (geographic constraint) | YES (M03 launch) |
| pack-eu | OCI eu-frankfurt-1 | OCI eu-amsterdam-1 | DR pair | Conditional (first EU tenant SCC) |
| pack-us | OCI us-ashburn-1 | OCI us-phoenix-1 | DR pair | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | OCI us-phoenix-1 (HIPAA-eligible) | DR pair; isolated from non-HC pack-us | Conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | — | YES | Conditional |
| pack-sg | OCI ap-singapore-1 | — | YES | Conditional |
| pack-au | OCI ap-sydney-1 | OCI ap-melbourne-1 | DR pair | Conditional |
| pack-in | OCI ap-hyderabad-1 | OCI ap-mumbai-1 | DR pair | Conditional |
| pack-br | OCI sa-saopaulo-1 | OCI sa-vinhedo-1 | DR pair | Conditional |
| pack-ae | OCI me-abudhabi-1 | OCI me-dubai-1 | DR pair | Conditional |
| pack-ksa | OCI me-jeddah-1 | OCI me-riyadh-1 | DR pair | Conditional |

## DNS + TLS

Per-pack ingress hostname:

```
<tenant-hash>.app.oyatie.dev   →  pack-specific ingress IP
                                   (OCI Traffic Management with health check + geo-routing)
```

- DNS: OCI Traffic Management with primary + DR ingress IP per pack, health-checked.
- TLS: ACME (Let's Encrypt) per-pack cert; rotated every 90 days.
- HSTS preload (`max-age=31536000; includeSubDomains; preload`).
- HTTP/3 enabled with HTTP/2 + HTTP/1.1 fallback.
- Pinned cipher suite: TLS 1.3 ECDHE-X25519 only.

## In-Pack DR-Pair Architecture

For packs with a DR pair:

```text
┌─ Pack <X> ───────────────────────────────────────────────────────────────┐
│                                                                          │
│  Primary region                              DR-pair region              │
│  ┌──────────────────────────┐                ┌──────────────────────────┐│
│  │ Ingress LB (active)      │                │ Ingress LB (warm)        ││
│  │ shell-routing            │                │ shell-routing (idle)     ││
│  │ tenant-context           │                │ tenant-context (idle)    ││
│  │ auth-gateway             │                │ auth-gateway (idle)      ││
│  │ module-loader            │                │ module-loader (idle)     ││
│  │ frontend-bundle-serve    │                │ frontend-bundle-serve    ││
│  │ Postgres primary         │   sync repl→   │ Postgres standby         ││
│  │ Valkey master            │   AOF/RDB repl │ Valkey replica           ││
│  │ OCI CDN region           │   ←→ object →  │ OCI CDN DR region        ││
│  └──────────────────────────┘                └──────────────────────────┘│
│             │                                          │                 │
│             └───── per-pack OCI Traffic Management ────┘                 │
│                            (geo + health-check failover)                 │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

### Sync posture

- Postgres + Citus: synchronous streaming replication primary → standby
  (1 standby promotes on primary failure).
- Valkey: AOF + replica streaming; eventual-consistency window ≤ 1 s.
- CDN: per-region origin; both regions can serve identical public-class
  assets; tenant content path goes through primary origin only.
- KMS: per-pack keyring replicated across primary + DR via OCI KMS.

### RPO / RTO targets

| Pack tier | RPO | RTO | Notes |
|---|---|---|---|
| pack-kr single-region | n/a (single AZ failover) | ≤ 15 s in-region | accepted single-region risk per `policy/data-residency.md`; KR geographic constraint |
| pack-eu / pack-us / pack-au / pack-in / pack-br / pack-ae / pack-ksa DR-pair | ≤ 5 s | ≤ 30 s | warm-standby promotion |
| pack-us-healthcare DR-pair | ≤ 5 s | ≤ 30 s | HIPAA-eligible regions only |
| pack-jp / pack-sg single-region | n/a (in-region AZ failover) | ≤ 15 s | accepted single-region risk |

## Cross-pack policy

Cross-pack replication of any Application Shell data is **FORBIDDEN**.
The CDN edge is the only globally-distributed surface and serves
public-class assets only. See `policy/data-residency.md`.

## Failover Procedure

### Automatic (primary-region degradation, in-pack)

1. OCI Traffic Management health check declares primary unhealthy
   (3 failed probes over 30 s).
2. DNS records flip to DR region; TTL = 30 s (so worst-case 60-s
   stale-DNS).
3. Postgres standby promotes (synchronous replication ensures no data
   loss; RPO ≤ 5 s).
4. Valkey replica promotes; in-flight sessions may force re-sign-in
   for ≤ 1 % of users (acceptable per RTO budget).
5. CDN serves from DR origin shield; warm cache.
6. observability fires `region.failover` event; on-call paged.

### Manual (planned maintenance)

Per `runbooks/auth-gateway-restart.md` §"Region failover (planned)":

1. Announce maintenance window 7 days ahead.
2. Increase DR region capacity 1 h ahead.
3. Pause auth-gateway worker on primary; let in-flight sessions drain.
4. DNS flip; verify DR taking traffic.
5. Run maintenance on primary.
6. Reverse flip when complete.

## Capacity for DR

DR region pre-provisioned at 60 % of primary capacity (cost vs. RTO
trade-off). HPA scales to 100 % within ≤ 5 min of failover. Cost
overhead per DR-pair pack: ~60 % of primary cost (see `cost-budget.md`).

## Single-region packs (pack-kr / pack-jp / pack-sg)

Geographic / regulatory constraint: only one OCI region available in
jurisdiction. Mitigation:

- Multi-AZ within region (3 AZs).
- Cross-AZ Postgres + Valkey + ingress LB.
- Object storage with cross-AZ replication.
- Accepted Sev-1 risk: full-region OCI outage = pack-kr outage.
  Tracked in risk register; tenants warned in DPA.

## DSR Cascade Across Regions

When a DSR (erasure) request lands:
- Within-pack: as described in `policy/data-residency.md` §"DSR Cascade".
- Cross-pack: never occurs (data never crosses packs).

## References

- ADR-0117 packs.
- `microservices/application/policy/data-residency.md`.
- `microservices/observability/multi-region.md` (precedent).
