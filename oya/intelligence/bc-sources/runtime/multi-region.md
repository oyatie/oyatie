---
doc_class: MultiRegionPlan
title: Multi-Region Topology + BCDR
microservice: foundry-runtime
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-foundry-runtime + cloud-iac + cloud-k8s
deciders: ops-sre-reliability, axis-foundry-runtime, council-architecture, council-privacy
related_adrs: [ADR-0025, ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/intelligence-runtime/policy/data-residency.md
  - microservices/intelligence-runtime/capacity-model.md
  - microservices/intelligence-runtime/cost-budget.md
  - microservices/intelligence-runtime/failure-modes.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Multi-Region Topology + BCDR (foundry-runtime µservice)

## Purpose

Define multi-region topology for foundry-runtime across 11 oyatie packs: pack-pinning, in-pack DR pair (where applicable), cross-pack-replication-forbidden policy, BCDR posture, RPO/RTO per region, failover procedures. Authoritative reference for ops-sre-reliability on-call during region outages and for auditors verifying business-continuity claims.

## Topology Per Pack

| Pack | Primary region | DR pair region (warm-standby) | Single-region? | Activation |
|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | — | YES | YES (M01 launch) |
| pack-eu | OCI eu-frankfurt-1 | OCI eu-amsterdam-1 | DR pair | Conditional |
| pack-us | OCI us-ashburn-1 | OCI us-phoenix-1 | DR pair | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | OCI us-phoenix-1 (HIPAA-eligible) | DR pair; isolated from non-HC pack-us | Conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | — | YES | Conditional |
| pack-sg | OCI ap-singapore-1 | — | YES | Conditional |
| pack-au | OCI ap-sydney-1 | OCI ap-melbourne-1 | DR pair | Conditional |
| pack-in | OCI ap-hyderabad-1 | OCI ap-mumbai-1 | DR pair | Conditional |
| pack-br | OCI sa-saopaulo-1 | OCI sa-vinhedo-1 | DR pair | Conditional |
| pack-ae | OCI me-abudhabi-1 | OCI me-dubai-1 | DR pair | Conditional |
| pack-ksa | OCI me-jeddah-1 | OCI me-riyadh-1 | DR pair | Conditional |

## In-Pack DR-Pair Architecture

For packs with a DR pair:

```text
┌─ Pack <X> ────────────────────────────────────────────────────────────────┐
│                                                                          │
│  Primary region                          DR-pair region                  │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ Runtime cluster (active) │            │ Runtime cluster (warm)   │    │
│  │  - executor / orchestr.  │   replic   │  - same components       │    │
│  │  - pool warm pods        │ ◀────────▶ │  - 0.6× capacity         │    │
│  └──────────────────────────┘   intra-   └──────────────────────────┘    │
│  ┌──────────────────────────┐   pack     ┌──────────────────────────┐    │
│  │ Valkey cluster (primary)  │            │ Valkey warm-standby       │    │
│  │  - 6 shards × RF 2       │            │   0.6× shard count       │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ Postgres (primary + repl)│            │ Postgres replica         │    │
│  │  - streaming replication │            │  - promoted on failover  │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│                                                                          │
│  Global Traffic Manager (per-pack DNS):                                  │
│  - Health check on primary's executor + Valkey + Postgres                 │
│  - On failure: DNS failover → DR pair (≤60s TTL)                         │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

### Replication

| Component | Mode | RPO | Cross-region |
|---|---|---|---|
| Valkey AOF + RDB snapshots | Async via S3 cross-region replication (intra-pack) | ≤30s | intra-pack only |
| Postgres streaming replication | Synchronous within-AZ + async cross-AZ within pack | ≤5s | intra-pack only |
| Capability mirror | Pulled from foundry-supervisor (pack-pinned scope) | per supervisor RPO | intra-pack only |
| Invocation lifecycle records | Postgres replication | ≤5s | intra-pack only |
| Cedar policies + Helm values + descriptor schemas | Git-versioned | 0 (declarative) | global (artifacts, not tenant data) |

### Cross-pack replication: FORBIDDEN by default

Per `policy/data-residency.md`, no tenant session-state or invocation record crosses pack boundaries. Narrow exceptions (tenant-executed SCCs for GDPR; tenant-specific BCDR exercise) documented inline. **EU-resident tenant data never reaches a non-EU region without Schrems-II-compatible SCC + supplementary measures on file.**

## Failover Procedures

### Primary-region degraded (Sev-2)

1. Detection: runtime `request_failures_total > threshold` for ≥5min; or AZ-level OCI outage announced.
2. ops-sre-reliability on-call paged.
3. Verify failure scope (component-level vs region-level).
4. If component-level: scale unaffected; await OCI recovery (see `failure-modes.md` FM-01).
5. If region-level + DR pair: initiate DR failover (Step §"DR Failover").
6. If region-level + no DR pair (pack-kr / pack-jp / pack-sg): graceful degradation; tenants notified; await OCI recovery.

### DR Failover (packs with DR pair)

| Phase | Step | Time budget |
|---|---|---|
| 1 | Verify DR-pair region is healthy (executor + Valkey + Postgres reachable) | ≤2min |
| 2 | Promote DR-pair runtime cluster to active (HPA scales to primary capacity) | ≤10min |
| 3 | Promote DR-pair Postgres replica to primary | ≤5min |
| 4 | Promote DR-pair Valkey warm-standby to primary | ≤5min |
| 5 | Update Global Traffic Manager: DNS records to DR-pair endpoints | ≤1min (TTL 60s) |
| 6 | Update workload µservices' invocation client configs (Helm rollout) | ≤5min |
| 7 | Verify dispatch is flowing to DR-pair; invocation lifecycle records emitting | ≤5min |
| 8 | Verify two-channel corroboration (audit-chain emission + OnCall paging both working) | ≤2min |
| 9 | Notify tenants of failover (status page + email per `incident-response.md`) | ≤30min |
| 10 | Engage with OCI on primary-region restoration | ongoing |
| **Total** | **end-to-end DR failover** | **≤35min** (RTO target) |

RPO: ≤30s for session-state (Valkey AOF cadence); ≤5s for Postgres lifecycle records.
RTO: ≤35min (DR failover complete; tenant traffic stable on DR-pair region).

### Failback (after primary region recovers)

Failback is **manual** + scheduled; primary must demonstrate ≥6h healthy state before failback initiated. Mirrors DR Failover steps in reverse.

## BCDR Exercise Cadence

| Exercise | Cadence | Scope | Owner |
|---|---|---|---|
| DR failover drill (controlled, off-hours) | Quarterly per DR-paired pack | Full failover + failback | ops-sre-reliability |
| Cross-region restore drill (Postgres snapshot → DR-pair → restore validation) | Monthly | Snapshot-restore + integrity check | ops-sre-reliability |
| Tabletop exercise (regional outage scenario) | Annually | Full incident-response + comms + executive | ops-sre-reliability + leadership |
| Chaos engineering injection (pod kill, AZ partition) | Continuous (Chaos Mesh) | Single AZ / pod-level | ops-sre-reliability |
| Vendor-failure exercise (simulate OCI region outage) | Annually | All packs with that region | ops-sre-reliability |
| AutonomyGate bypass attempt rehearsal | Quarterly | Per-pack | ops-security |

## RPO / RTO Per Pack

| Pack | RPO target | RTO target | Single-region fallback |
|---|---|---|---|
| pack-kr | ≤30s (intra-region only) | ≤4h (OCI region SLA-dependent) | best-effort |
| pack-eu | ≤30s (CRR) | ≤35min (DR failover) | – |
| pack-us | ≤30s | ≤35min | – |
| pack-us-healthcare | ≤30s | ≤35min | – |
| pack-jp | ≤30s (no DR pair) | ≤4h | best-effort |
| pack-sg | ≤30s | ≤4h | best-effort |
| pack-au | ≤30s | ≤35min | – |
| pack-in | ≤30s | ≤35min | – |
| pack-br | ≤30s | ≤35min | – |
| pack-ae | ≤30s | ≤35min | – |
| pack-ksa | ≤30s | ≤35min | – |

Per-tenant RPO/RTO commitments part of tenant SLA (per DPA at `legal/dpa-template.md`).

## Tenant Notification

Per `incident-response.md` §"Tenant communications":
- **Status page (public)**: updated within 5min of failover initiation.
- **Tenant operator email**: within 30min for any Sev-1/2 affecting tenant's pack.
- **Customer-facing message template**: provided in tenant onboarding portal.
- **Regulatory notification**: per `compliance.md` enforced timelines (GDPR Art. 33 72h; HIPAA §164.404 60d; KR PIPA Art. 34 72h; etc.).

## Per-Pack BCDR Overlay

Per-pack BCDR specifics at `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/foundry-runtime-multi-region-overlay.md`. Example: pack-eu must satisfy DORA (Digital Operational Resilience Act 2022/2554) testing requirements when oyatie has EU financial-services tenants in scope.

## Verification

- `cargo run -p oya-dev-cli -- gate validate multi-region-conformance --microservice foundry-runtime` — exit 0; deployed topology matches this document for every active pack.
- Quarterly DR-failover drill audit log: success vs failure rate.
- Annual third-party BCDR audit: alignment with ISO 22301 / NIST SP 800-34 / DORA.

## References

- `microservices/intelligence-runtime/policy/data-residency.md`.
- `microservices/intelligence-runtime/capacity-model.md`; `cost-budget.md`; `failure-modes.md`; `incident-response.md`.
- `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/foundry-runtime-multi-region-overlay.md`.
- OCI region documentation — `oracle.com/cloud/data-regions/`.
- Valkey 8.1 (Redis wire-compat) HA — `redis.io/docs/management/replication/`.
- Postgres 16 LTS replication — `postgresql.org/docs/16/high-availability.html`.
- ISO/IEC 22301:2019; NIST SP 800-34; EU DORA Regulation 2022/2554.
