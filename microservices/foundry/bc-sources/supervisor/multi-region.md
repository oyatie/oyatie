---
doc_class: MultiRegionPlan
title: Multi-Region Topology + BCDR (foundry-supervisor)
microservice: foundry-supervisor
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-foundry-control-plane + cloud-iac + cloud-k8s
deciders: ops-sre-reliability, axis-foundry-control-plane, council-architecture, council-privacy
related_adrs: [ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/foundry-supervisor/policy/data-residency.md
  - microservices/foundry-supervisor/capacity-model.md
  - microservices/foundry-supervisor/cost-budget.md
  - microservices/foundry-supervisor/failure-modes.md
review_cadence: annually + on every pack activation
doc_status: published
---

# Multi-Region Topology + BCDR (foundry-supervisor µservice)

## Purpose

Define multi-region topology across 11 oyatie packs: pack-pinning, in-pack DR pair (where applicable), cross-pack replication-forbidden policy, BCDR posture, RPO/RTO per region, failover procedures.

## Topology Per Pack

| Pack | Primary region | DR pair (warm-standby) | Single-region? | Activation |
|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | — | YES | YES (M01) |
| pack-eu | eu-frankfurt-1 | eu-amsterdam-1 | DR pair | Conditional |
| pack-us | us-ashburn-1 | us-phoenix-1 | DR pair | Conditional |
| pack-us-healthcare | us-ashburn-1 | us-phoenix-1 | DR pair (HIPAA-isolated) | Conditional (post-BAA) |
| pack-jp | ap-tokyo-1 | — | YES | Conditional |
| pack-sg | ap-singapore-1 | — | YES | Conditional |
| pack-au | ap-sydney-1 | ap-melbourne-1 | DR pair | Conditional |
| pack-in | ap-hyderabad-1 | ap-mumbai-1 | DR pair | Conditional |
| pack-br | sa-saopaulo-1 | sa-vinhedo-1 | DR pair | Conditional |
| pack-ae | me-abudhabi-1 | me-dubai-1 | DR pair | Conditional |
| pack-ksa | me-jeddah-1 | me-riyadh-1 | DR pair | Conditional |

## In-Pack DR-Pair Architecture

```text
┌─ Pack <X> ────────────────────────────────────────────────────────────────┐
│                                                                          │
│  Primary region                          DR-pair region                  │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ Postgres primary (active)│  Patroni   │ Postgres replica (warm)  │    │
│  │  - sync replication      │ ◀────────▶ │  - promotable in ≤ 30s   │    │
│  │  - WAL archive to S3     │   intra-   │  - WAL replay continuous │    │
│  └──────────────────────────┘   pack     └──────────────────────────┘    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ Valkey Cluster (3×2)      │            │ Valkey Cluster (warm)     │    │
│  │  - AOF every-second      │            │  - AOF replay            │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ Kubernetes Operator      │            │ Kubernetes Operator      │    │
│  │ (active in primary; HA-  │            │  (warm-standby)          │    │
│  │  leader-elected)         │            │                          │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ REST + Worker (active)   │            │ REST + Worker (warm)     │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│                                                                          │
│  Global Traffic Manager (per-pack DNS):                                  │
│  - Health check on REST + Postgres-write paths                           │
│  - On failure: DNS failover → DR pair (TTL 60s)                          │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

### Replication

| Component | Mode | RPO | Cross-region |
|---|---|---|---|
| Postgres primary→replica | Synchronous streaming | ≤ 0 s | intra-pack only |
| Postgres WAL archive (S3) | Async (every-1-min cadence) | ≤ 1 min | intra-pack only |
| Valkey Cluster cross-shard replication | Async (every-second AOF) | ≤ 1 s | intra-pack only |
| Kubernetes CRDs | Native etcd consensus across AZs | ≤ 0 s | intra-pack only |
| OpenBao secret tree | Per-pack OpenBao instance; raft | ≤ 5 s | intra-pack only |
| Capability YAMLs | Git-versioned | 0 | global (tenant-owned repos) |
| Cedar policies | Git-versioned | 0 | global |
| Helm + IaC | Git-versioned | 0 | global |

### Cross-pack replication: FORBIDDEN by default

Per `policy/data-residency.md`. SCC + intra-pack DR exceptions documented inline.

## Failover Procedures

### Primary-region degraded (Sev-2)

1. Detect Mimir/Postgres/Redis `request_failures_total > threshold` ≥ 5min OR AZ-level OCI outage.
2. ops-sre-reliability on-call paged.
3. Verify failure scope (component-level vs region-level).
4. If component-level: scale unaffected components.
5. If region-level + DR pair: initiate DR failover (next subsection).
6. If region-level + no DR pair (pack-kr, pack-jp, pack-sg): graceful degradation; tenants notified; await OCI recovery.

### DR Failover (DR-pair packs)

| Phase | Step | Time budget |
|---|---|---|
| 1 | Verify DR-pair healthy | ≤ 2 min |
| 2 | Promote Postgres replica via Patroni (synchronous → primary) | ≤ 30 s |
| 3 | Promote Valkey Cluster (already in warm-standby) | ≤ 1 min |
| 4 | Promote Kubernetes Operator (lease-leadership re-runs) | ≤ 30 s |
| 5 | Update Global Traffic Manager DNS to DR-pair (TTL 60s) | ≤ 1 min |
| 6 | Update foundry-runtime + foundry-evidence configs to DR-pair endpoints | ≤ 5 min Helm rollout |
| 7 | Verify control-plane availability + kill-switch latency back within SLO | ≤ 5 min |
| 8 | Notify tenants of failover (per `incident-response.md`) | ≤ 30 min |
| 9 | Engage OCI on primary-region restoration | ongoing |
| **Total** | **end-to-end DR failover** | **≤ 35 min** (RTO target) |

RPO: ≤ 1 min (Postgres sync replication tail; Valkey AOF tail).
RTO: ≤ 35 min.

### Failback

Manual + scheduled (not auto-failback). Primary region must demonstrate ≥ 6 h healthy state before failback. Procedure mirrors DR failover in reverse.

## BCDR Exercise Cadence

| Exercise | Cadence | Scope | Owner |
|---|---|---|---|
| DR failover drill | Quarterly per DR-pair pack | Full failover + failback for one pack at a time | ops-sre-reliability |
| Cross-region restore drill (S3 WAL → restore + integrity check) | Monthly | Snapshot-restore validation | ops-sre-reliability |
| Tabletop exercise (regional outage scenario) | Annually | Full incident response + comms | ops-sre-reliability + leadership |
| Chaos engineering (random pod kill, AZ partition) | Continuous (Chaos Mesh) | Single AZ / pod-level | ops-sre-reliability |
| Vendor-failure-mode exercise (simulate OCI region outage) | Annually | All packs in that region | ops-sre-reliability |
| EU AI Act Art. 60 post-market monitoring drill | Quarterly | Verify supervision-event + post-market chain works | council-privacy + axis-foundry-control-plane |

## RPO / RTO Per Pack

| Pack | RPO | RTO | Fallback |
|---|---|---|---|
| pack-kr | ≤ 1 min (synchronous in-AZ; no DR pair) | ≤ 4 h (OCI region recovery; ap-seoul-1 has 3 AZs) | best-effort |
| pack-eu | ≤ 1 min | ≤ 35 min | – |
| pack-us | ≤ 1 min | ≤ 35 min | – |
| pack-us-healthcare | ≤ 1 min | ≤ 35 min | – |
| pack-jp | ≤ 1 min | ≤ 4 h | best-effort |
| pack-sg | ≤ 1 min | ≤ 4 h | best-effort |
| pack-au | ≤ 1 min | ≤ 35 min | – |
| pack-in | ≤ 1 min | ≤ 35 min | – |
| pack-br | ≤ 1 min | ≤ 35 min | – |
| pack-ae | ≤ 1 min | ≤ 35 min | – |
| pack-ksa | ≤ 1 min | ≤ 35 min | – |

Per-tenant SLA per tenant DPA. Single-region packs disclose weaker RTO at tenant onboarding.

## Tenant Notification

Per `incident-response.md` §"Tenant communications":
- Status page: updated within 5 min of failover initiation.
- Tenant operator email: ≤ 30 min for any Sev-1/2.
- Regulatory: per `compliance.md` enforced timelines (GDPR Art. 33 72h; HIPAA §164.404 60d; KR PIPA Art. 34 72h; etc.).
- **EU AI Act Art. 73**: serious incident reporting to EU AI Office on Sev-1 supervisor outage affecting high-risk Annex III tenant capabilities.

## Per-Pack BCDR Overlay

Per-pack BCDR specifics live at `regional-packs/<pack>/foundry-supervisor-multi-region-overlay.md`. Example: pack-eu must satisfy DORA (2022/2554) testing requirements when oyatie has EU financial-services tenants.

## Verification

- `cargo run -p oya-dev-cli -- gate validate multi-region-conformance --microservice foundry-supervisor` — exit 0.
- Quarterly DR-failover drill audit log.
- Annual third-party BCDR audit (ISO 22301 / NIST SP 800-34 / DORA alignment).

## References

- `microservices/foundry-supervisor/policy/data-residency.md`.
- `microservices/foundry-supervisor/capacity-model.md`.
- `microservices/foundry-supervisor/cost-budget.md`.
- `microservices/foundry-supervisor/failure-modes.md`.
- `microservices/foundry-supervisor/incident-response.md`.
- OCI region docs — `oracle.com/cloud/data-regions/`.
- PostgreSQL Patroni — `patroni.readthedocs.io`.
- Valkey Cluster — `redis.io/docs/management/scaling/`.
- ISO/IEC 22301:2019; NIST SP 800-34; EU DORA 2022/2554.
- EU AI Act 2024/1689 Art. 60 + Art. 73.
