---
doc_class: MultiRegionPlan
title: Multi-Region Topology + BCDR
microservice: cell
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-cell-substrate + cloud-iac + cloud-k8s
deciders: ops-sre-reliability, axis-cell-substrate, council-architecture, council-privacy
related_adrs: [ADR-0117, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/cell/policy/data-residency.md
  - microservices/cell/capacity-model.md
  - microservices/cell/cost-budget.md
  - microservices/cell/failure-modes.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Multi-Region Topology + BCDR (cell µservice)

## Purpose

Define multi-region topology for the cell substrate across the 11 oyatie packs: pack-pinning, in-pack DR pair where applicable, cross-pack replication-forbidden policy, BCDR posture, RPO/RTO targets per region, failover procedures.

## Topology Per Pack

| Pack | Primary region | DR pair (warm-standby) | Single-region? | Activation |
|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | — | YES | YES (M01 launch) |
| pack-eu | OCI eu-frankfurt-1 | OCI eu-amsterdam-1 | DR pair | Conditional |
| pack-us | OCI us-ashburn-1 | OCI us-phoenix-1 | DR pair | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA) | OCI us-phoenix-1 (HIPAA) | DR pair; isolated | Conditional (post-BAA) |
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
┌─ Pack <X> ───────────────────────────────────────────────────────────────┐
│                                                                          │
│  Primary region                          DR-pair region                  │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ K8s management cluster   │            │ K8s management cluster   │    │
│  │  (Cluster API; active)   │  pairing   │  (warm standby; 0.5×)    │    │
│  │  - Cluster CRDs primary  │ ◀────────▶ │  - Cluster CRDs mirror   │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ Workload clusters        │            │ Workload clusters        │    │
│  │ (active; one per cell)   │ replic.    │ (warm; standby;          │    │
│  │ - cell-* namespaces      │ ◀────────▶ │   reduced replica)       │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ Postgres cell-registry   │ streaming  │ Postgres replica         │    │
│  │  (primary)               │ ◀────────▶ │   (read-replica + DR)    │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ SPIRE server (active)    │   sync     │ SPIRE server (standby)   │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│                                                                          │
│  Global Traffic Manager (per-pack DNS):                                  │
│  - Health check on primary's Postgres write path + management API       │
│  - On failure: DNS failover → DR-pair (≤ 60s TTL)                       │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

### Replication

| Component | Mode | RPO | Cross-region |
|---|---|---|---|
| Postgres cell-registry | Streaming replication primary→replica | ≤ 30 s | intra-pack only |
| Cluster API Cluster CRDs | etcd snapshot + replay; controllers re-reconcile | ≤ 5 min | intra-pack only |
| Per-cell workload PVs | Backed by OCI Block Volume cross-region replication where pack supports | ≤ 5 min | intra-pack only |
| Per-cell S3 prefixes | OCI Object Storage cross-region replication (CRR) | ≤ 5 min | intra-pack only |
| SPIRE bundle | Federated trust bundle replicated across regions | ≤ 30 s | intra-pack only |
| OpenBao cell-credentials | OpenBao Raft replication | ≤ 30 s | intra-pack only |
| Audit-chain seals | Replicated via audit-chain µservice | per audit-chain SLO | intra-pack only |
| Cell-substrate operator pods | redeployed in DR region during failover | n/a | declarative |
| `registry/cell-assignment.jsonl` | Git-versioned (global; not tenant data) | 0 | global |
| Cedar policies | Git-versioned | 0 | global |
| Helm + Kustomize manifests | Git-versioned | 0 | global |

### Cross-pack replication: FORBIDDEN by default

Per `policy/data-residency.md`. Narrow exceptions: tenant-executed SCCs (GDPR); HIPAA BAA intra-region pair only. **EU-resident tenant cells never reach a non-EU region without Schrems-II-compatible SCC + supplementary measures.**

## Failover Procedures

### Primary-region degraded (Sev-2)

1. Detection: Postgres or Cluster API failure for ≥ 5min; or AZ-level OCI outage announced.
2. ops-sre-reliability on-call paged.
3. Verify failure scope (component-level vs region-level).
4. If component-level: scale out unaffected components; await OCI recovery.
5. If region-level + DR pair exists: initiate DR failover.
6. If region-level + no DR pair (pack-kr / pack-jp / pack-sg): graceful degradation; tenants notified; await OCI region recovery.

### DR Failover (packs with DR pair)

| Phase | Step | Time budget |
|---|---|---|
| 1 | Verify DR-pair region healthy (Postgres / Cluster API / SPIRE reachable) | ≤ 2 min |
| 2 | Promote DR-pair Postgres replica to primary (CloudNativePG promotion) | ≤ 2 min |
| 3 | Promote DR-pair Cluster API management cluster to active | ≤ 5 min |
| 4 | Update Global Traffic Manager: DNS to DR-pair endpoints | ≤ 1 min (TTL 60s) |
| 5 | Spin up DR-pair cell-substrate operator pods to full replica count | ≤ 10 min |
| 6 | Workload clusters re-attach to DR-pair management cluster (via Cluster API move) | ≤ 15 min |
| 7 | Verify reads + writes flowing to DR-pair | ≤ 5 min |
| 8 | Tenant communications (per `incident-response.md`) | ≤ 30 min |
| 9 | Engage with OCI on primary-region restoration | ongoing |
| **Total** | **end-to-end DR failover** | **≤ 45 min** (RTO target) |

RPO: ≤ 5 min (worst case from Postgres streaming-replication catch-up + S3 CRR cadence).
RTO: ≤ 45 min (DR failover complete; tenant cell-assignment reads stable on DR-pair).

### Failback (after primary region recovers)

Manual + scheduled (not auto). Primary region must demonstrate ≥ 6h healthy state before failback. Procedure mirrors DR failover in reverse.

## BCDR Exercise Cadence

| Exercise | Cadence | Scope | Owner |
|---|---|---|---|
| DR failover drill (controlled, off-hours) | Quarterly per pack with DR pair | Full failover + failback for one pack | ops-sre-reliability |
| Postgres restore drill | Monthly | Restore + integrity check on a sample of cell-registry | ops-sre-reliability |
| Cluster API move drill (CRD migration) | Quarterly | clusterctl move between mgmt clusters | cloud-k8s |
| Tabletop exercise (regional outage) | Annually | Full incident-response + comms | ops-sre-reliability + leadership |
| Chaos engineering (pod kill, AZ partition) | Continuous (Chaos Mesh) | Single-AZ / pod-level | ops-sre-reliability |
| Vendor-failure-mode exercise | Annually | Simulated OCI region outage | ops-sre-reliability |

## RPO / RTO Per Pack

| Pack | RPO target | RTO target | Single-region fallback |
|---|---|---|---|
| pack-kr | ≤ 30 s (intra-region streaming; no DR pair) | ≤ 4h (OCI region recovery; ap-seoul-1 has 3 AZs) | best-effort; OCI region SLA |
| pack-eu | ≤ 5 min (cross-region) | ≤ 45 min | – |
| pack-us | ≤ 5 min | ≤ 45 min | – |
| pack-us-healthcare | ≤ 5 min | ≤ 45 min | – |
| pack-jp / pack-sg | ≤ 30 s (no DR pair) | ≤ 4h | best-effort |
| pack-au / pack-in / pack-br / pack-ae / pack-ksa | ≤ 5 min | ≤ 45 min | – |

Per-tenant RPO/RTO commitments part of tenant SLA (per tenant DPA at `legal/dpa-template.md`).

## Tenant Notification

Per `incident-response.md` §"Tenant communications":
- Status page (public) updated ≤ 5 min of failover initiation.
- Tenant operator email ≤ 30 min for Sev-1 / Sev-2.
- Regulatory notification per `compliance.md` timelines (GDPR Art. 33 72h; HIPAA §164.404 60d; KR PIPA Art. 34 72h; NIS2 24h initial when applicable).

## Per-Pack BCDR Overlay

Per-pack BCDR specifics at `regional-packs/<pack>/cell-multi-region-overlay.md`. Example: pack-eu must satisfy DORA testing requirements when oyatie has EU financial-services tenants.

## Verification

- `cargo run -p oya-dev-cli -- gate validate cell-multi-region-conformance` — exit 0; deployed topology matches.
- Quarterly DR-failover drill audit log: success vs failure trend.
- Annual third-party BCDR audit: alignment with ISO 22301 / NIST SP 800-34 / DORA.

## References

- `microservices/cell/policy/data-residency.md`.
- `microservices/cell/capacity-model.md`.
- `microservices/cell/cost-budget.md`.
- `microservices/cell/failure-modes.md`.
- `microservices/cell/incident-response.md`.
- Bominal ADR-0009; ADR-0019.
- Kubernetes Cluster API DR — `cluster-api.sigs.k8s.io/clusterctl/commands/move.html`.
- CloudNativePG DR — `cloudnative-pg.io/documentation/current/disaster-recovery/`.
- ISO 22301; NIST SP 800-34; EU DORA Regulation 2022/2554.
