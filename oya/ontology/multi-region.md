---
doc_class: MultiRegionPlan
title: Multi-Region Topology + BCDR
microservice: ontology
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-ontology + cloud-iac + cloud-k8s
deciders: ops-sre-reliability, axis-ontology, council-architecture, council-privacy
related_adrs: [ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/ontology/policy/data-residency.md
  - microservices/ontology/capacity-model.md
  - microservices/ontology/cost-budget.md
  - microservices/ontology/failure-modes.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Multi-Region Topology + BCDR (ontology µservice)

## Purpose

Define multi-region topology for ontology across the 11 oyatie packs: pack-pinning, in-pack DR pair (where applicable), cross-pack replication-forbidden policy, BCDR posture, RPO/RTO per region, failover procedures. Authoritative reference for ops-sre-reliability on-call during region outages and for auditors verifying business-continuity claims.

## Topology Per Pack

| Pack | Primary region | DR pair (warm-standby) | Single-region? | Activation status |
|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | — | YES (geographic constraint) | YES (M02b launch) |
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

## In-Pack DR-Pair Architecture

For packs with a DR pair:

```text
┌─ Pack <X> ────────────────────────────────────────────────────────────────┐
│                                                                          │
│  Primary region                          DR-pair region                  │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ Postgres + Citus (active)│            │ Postgres + Citus (warm)  │    │
│  │  - coordinator + workers │   stream   │  - coordinator + workers │    │
│  │  - RF=3 within region    │ ◀────────▶ │  - logical replication   │    │
│  │  - PITR to object store  │ replic     │  - 0.6× capacity         │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ ClickHouse + Valkey +    │            │ ClickHouse + Valkey +    │    │
│  │ Kafka (active)           │            │ Kafka (warm)             │    │
│  │ same shape               │            │ 0.6× capacity            │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ Ontology Layer-B pods    │            │ Ontology Layer-B pods    │    │
│  │ (active)                 │            │ (warm-standby; ≥ 1 of    │    │
│  │                          │            │ each critical app)       │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│                                                                          │
│  Global Traffic Manager (per-pack DNS):                                  │
│  - Health check on Postgres coordinator + ontology Layer-B app pods      │
│  - On failure: DNS failover → DR pair (≤ 60s TTL)                        │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

### Replication

| Component | Mode | RPO | Cross-region |
|---|---|---|---|
| Postgres coordinator + workers | Streaming logical replication primary→replica | ≤ 1 s | intra-pack only |
| Postgres PITR backups | S3 CRR (cross-region replication within pack) | ≤ 5 min | intra-pack only |
| ClickHouse history-mirror | ReplicatedMergeTree + S3 CRR | ≤ 5 min | intra-pack only |
| Valkey schema-registry cache | Sentinel + replication | ≤ 30 s | intra-pack only |
| Kafka KRaft outbox | Inter-broker replication + S3 tiered storage CRR | ≤ 5 s | intra-pack only |
| Audit-chain Merkle journal | Postgres replication + S3 CRR | ≤ 1 s | intra-pack only |
| Object Type schemas | Git-versioned + Helm-deployed on both sides | 0 (declarative) | global (schemas are config, not tenant data) |
| Cedar policies | Git-versioned | 0 | global |
| Workspace Cargo.toml + IaC | Git-versioned | 0 | global |

### Cross-pack replication: FORBIDDEN by default

Per `policy/data-residency.md`, no tenant data crosses pack boundaries. Narrow exceptions (tenant-executed SCCs for GDPR transfers; tenant-specific BCDR exercise) are documented inline. **EU-resident tenant Object Types never reach a non-EU region without a Schrems-II-compatible SCC + supplementary measures on file.**

## Failover Procedures

### Primary-region degraded (Sev-2)

1. Detection: Postgres / ClickHouse `request_failures_total > threshold` for ≥ 5 min; or AZ-level OCI outage announced.
2. ops-sre-reliability on-call paged.
3. Verify failure scope (component-level vs region-level).
4. If component-level: scale out unaffected components; await OCI recovery (see `failure-modes.md` FM-01).
5. If region-level + DR pair exists: initiate DR failover.
6. If region-level + no DR pair (pack-kr / pack-jp / pack-sg): graceful degradation; tenants notified; await OCI region recovery.

### DR Failover (packs with DR pair)

| Phase | Step | Time budget |
|---|---|---|
| 1 | Verify DR-pair region healthy (Postgres + ClickHouse + Kafka write paths reachable) | ≤ 2 min |
| 2 | Promote Postgres logical replica to primary on DR pair; verify Citus shards present | ≤ 10 min |
| 3 | Update Global Traffic Manager: DNS records to DR-pair endpoints | ≤ 1 min (TTL 60 s) |
| 4 | Update workload µservices' SDK config (Helm rollout to point at DR-pair endpoints) | ≤ 5 min |
| 5 | Verify Object Type writes resume; Function reads succeed against DR-pair | ≤ 5 min |
| 6 | Promote DR-pair ClickHouse to active (already replicated via ReplicatedMergeTree) | ≤ 5 min |
| 7 | Promote DR-pair Valkey + Kafka (already replicated) | ≤ 5 min |
| 8 | Verify audit-chain seal cadence resumes on DR pair | ≤ 2 min |
| 9 | Notify tenants of failover (status page + email per `incident-response.md`) | ≤ 30 min |
| 10 | Engage OCI on primary-region restoration | ongoing |
| **Total** | **end-to-end DR failover** | **≤ 35 min** (RTO target) |

RPO: ≤ 5 min (S3 CRR cadence; some recent commits may not yet be replicated).
RTO: ≤ 35 min (DR failover complete; tenant traffic stable on DR-pair).

### Failback (after primary region recovers)

Failback to primary is **manual** and scheduled per industry best-practice; the primary region must demonstrate ≥ 6h healthy state before failback initiated. Procedure mirrors DR failover in reverse, with the warm-standby (now primary) becoming warm again.

## BCDR Exercise Cadence

| Exercise | Cadence | Scope | Owner |
|---|---|---|---|
| DR failover drill (controlled, off-hours) | Quarterly per pack with DR pair | Full failover + failback for one pack at a time | ops-sre-reliability |
| Cross-region restore drill (S3 → DR-pair → restore validation) | Monthly | Snapshot-restore + integrity check on a subset of Postgres shards | ops-sre-reliability |
| Tabletop exercise (regional outage scenario) | Annually | Full incident-response + comms + executive briefing | ops-sre-reliability + leadership |
| Chaos engineering injection (random pod kill, AZ partition) | Continuous (Chaos Mesh) | Single AZ / pod-level | ops-sre-reliability |
| Vendor-failure-mode exercise (simulate OCI region outage) | Annually | All packs with that region | ops-sre-reliability |

## RPO / RTO Per Pack

| Pack | RPO target | RTO target | Single-region fallback |
|---|---|---|---|
| pack-kr | ≤ 5 s (intra-region streaming replication only; no DR pair) | ≤ 4 h (depends on OCI region recovery) | best-effort; OCI region SLA |
| pack-eu | ≤ 5 s | ≤ 35 min (DR failover) | – |
| pack-us | ≤ 5 s | ≤ 35 min | – |
| pack-us-healthcare | ≤ 5 s | ≤ 35 min | – |
| pack-jp | ≤ 5 s (no DR pair) | ≤ 4 h | best-effort |
| pack-sg | ≤ 5 s (no DR pair) | ≤ 4 h | best-effort |
| pack-au | ≤ 5 s | ≤ 35 min | – |
| pack-in | ≤ 5 s | ≤ 35 min | – |
| pack-br | ≤ 5 s | ≤ 35 min | – |
| pack-ae | ≤ 5 s | ≤ 35 min | – |
| pack-ksa | ≤ 5 s | ≤ 35 min | – |

Per-tenant RPO/RTO commitments are part of tenant SLA (per tenant DPA at `legal/dpa-template.md`). Packs without DR pair have weaker RTO disclosed at onboarding.

## Tenant Notification

Tenants are notified at failover initiation per the comms template in `incident-response.md` §"Tenant communications":

- **Status page (public)**: updated within 5 min of failover.
- **Tenant operator email**: sent within 30 min for any Sev-1/2 affecting tenant's pack.
- **Customer-facing message template**: provided in tenant onboarding portal.
- **Regulatory notification**: per `compliance.md` enforced timelines (GDPR Art. 33 72h; HIPAA §164.404 60d; KR PIPA Art. 34 72h; etc.).

## Per-Pack BCDR Overlay

Per-pack BCDR specifics (region capabilities, local-regulator BCDR requirements) live at `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/multi-region-overlay.md`. Example: pack-eu must satisfy DORA (Digital Operational Resilience Act 2022/2554) testing requirements for EU financial-services tenants.

## Verification

- `buck2 build //:quality-lane-registry-authority-check # lane=multi-region-conformance --microservice ontology` — exit 0; deployed topology matches this document.
- Quarterly DR-failover drill audit log: success vs failure rate.
- Annual third-party BCDR audit: alignment with ISO 22301 / NIST SP 800-34 / DORA.

## References

- `microservices/ontology/policy/data-residency.md`.
- `microservices/ontology/capacity-model.md`.
- `microservices/ontology/cost-budget.md`.
- `microservices/ontology/failure-modes.md`.
- `microservices/ontology/incident-response.md`.
- `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/multi-region-overlay.md` (per-pack).
- OCI region documentation — `oracle.com/cloud/data-regions/`.
- Postgres + Citus HA — `docs.citusdata.com`.
- ClickHouse ReplicatedMergeTree + S3 — `clickhouse.com/docs/en/engines/table-engines/mergetree-family/replication`.
- ISO/IEC 22301:2019 (Business continuity).
- NIST SP 800-34 (Contingency planning).
- EU DORA Regulation 2022/2554.
