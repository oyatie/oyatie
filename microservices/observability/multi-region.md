---
doc_class: MultiRegionPlan
title: Multi-Region Topology + BCDR
microservice: observability
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-observability + cloud-iac + cloud-k8s
deciders: ops-sre-reliability, axis-observability, council-architecture, council-privacy
related_adrs: [ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/observability/policy/data-residency.md
  - microservices/observability/capacity-model.md
  - microservices/observability/cost-budget.md
  - microservices/observability/failure-modes.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Multi-Region Topology + BCDR (observability µservice)

## Purpose

Define the multi-region topology for observability across the 11 oyatie packs: pack-pinning, in-pack DR pair (where applicable), cross-pack replication-forbidden policy, BCDR posture, RPO/RTO targets per region, failover procedures. This document is the authoritative reference for ops-sre-reliability on-call during region outages and for auditors verifying business-continuity claims.

## Topology Per Pack

| Pack | Primary region | DR pair region (warm-standby) | Single-region? | Activation status |
|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | — | YES (single-region; geographic constraint) | YES (M01 launch) |
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
│  │ Mimir cluster (active)   │            │ Mimir cluster (warm)     │    │
│  │  - distributor / ingester│   replic   │  - same components       │    │
│  │  - object storage (S3)   │ ◀────────▶ │  - replicated S3 bucket  │    │
│  │  - ruler / compactor     │   intra-   │  - rule files mirror     │    │
│  │  - HA RF=3 within region │   pack     │  - 0.6× capacity         │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ Loki + Tempo + Pyroscope │            │ Loki + Tempo + Pyroscope │    │
│  │ same shape               │            │ warm-standby (0.6×)      │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ Grafana + Alertmanager + │            │ Grafana mirror (read-only│    │
│  │ OnCall (active)          │            │ during normal ops)       │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│                                                                          │
│  Global Traffic Manager (per-pack DNS):                                  │
│  - Health check on primary's Mimir/Loki/Tempo write paths                │
│  - On failure: DNS failover → DR pair (≤ 60s TTL)                        │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

### Replication

| Component | Mode | RPO | Cross-region |
|---|---|---|---|
| Mimir blocks | Async via S3 cross-region replication (CRR) | ≤ 5 min | intra-pack only |
| Loki chunks | Async via S3 CRR | ≤ 5 min | intra-pack only |
| Tempo blocks | Async via S3 CRR | ≤ 5 min | intra-pack only |
| Pyroscope profiles | Async via S3 CRR | ≤ 5 min | intra-pack only |
| Grafana Postgres metadata | Streaming replication primary→replica | ≤ 30 s | intra-pack only |
| OnCall Postgres state | Streaming replication | ≤ 30 s | intra-pack only |
| Mimir recording rules | Git-versioned + Helm-deployed on both sides | 0 (declarative) | intra-pack only |
| OpenSLO manifests | Git-versioned | 0 | global (manifests are not tenant data) |
| Cedar policies | Git-versioned | 0 | global |
| Workspace Cargo.toml + IaC | Git-versioned | 0 | global |

### Cross-pack replication: FORBIDDEN by default

Per `data-residency.md`, no telemetry crosses pack boundaries. The narrow exceptions (tenant-executed SCCs for GDPR transfers; tenant-specific BCDR exercise) are documented inline. **EU-resident tenant data never reaches a non-EU region without a Schrems-II-compatible SCC + supplementary measures on file.**

## Failover Procedures

### Primary-region degraded (Sev-2)

1. Detection: Mimir/Loki/Tempo `request_failures_total > threshold` for ≥ 5min; or AZ-level OCI outage announced.
2. ops-sre-reliability on-call paged.
3. Verify the failure scope (component-level vs region-level).
4. If component-level: scale out unaffected components; await OCI recovery (see `failure-modes.md` FM-01).
5. If region-level + DR pair exists: initiate DR failover (Step §"DR Failover" below).
6. If region-level + no DR pair (pack-kr / pack-jp / pack-sg): graceful degradation; tenants notified; await OCI region recovery.

### DR Failover (packs with DR pair)

| Phase | Step | Time budget |
|---|---|---|
| 1 | Verify DR-pair region is healthy (Mimir/Loki/Tempo write paths reachable) | ≤ 2 min |
| 2 | Promote DR-pair Mimir/Loki/Tempo to active (HPA scales to primary capacity) | ≤ 10 min |
| 3 | Update Global Traffic Manager: DNS records to DR-pair endpoints | ≤ 1 min (TTL 60s) |
| 4 | Update workload µservices' OTel collector configs (Grafana Alloy) to send to DR-pair endpoints | ≤ 5 min (Helm rollout) |
| 5 | Verify ingest is flowing to DR-pair; eligibility verdicts resuming | ≤ 5 min |
| 6 | Promote DR-pair Grafana to active (Postgres replica promoted to primary) | ≤ 5 min |
| 7 | Promote DR-pair Alertmanager + OnCall | ≤ 5 min |
| 8 | Verify two-channel corroboration (Mimir write + OnCall paging both working) | ≤ 2 min |
| 9 | Notify tenants of failover (status page + email per `incident-response.md`) | ≤ 30 min |
| 10 | Engage with OCI on primary-region restoration | ongoing |
| **Total** | **end-to-end DR failover** | **≤ 35 min** (RTO target) |

RPO: ≤ 5 min (object-storage S3 CRR cadence; some ingester-buffered samples may be lost).
RTO: ≤ 35 min (DR failover complete; tenant traffic stable on DR-pair region).

### Failback (after primary region recovers)

Failback to primary is **manual** and scheduled (not auto-failback) per industry best-practice; the primary region must demonstrate ≥ 6h of healthy state before failback initiated. Procedure mirrors DR Failover steps in reverse, with the warm-standby (now primary) becoming warm again.

## BCDR Exercise Cadence

| Exercise | Cadence | Scope | Owner |
|---|---|---|---|
| DR failover drill (controlled, off-hours) | Quarterly per pack with DR pair | Full failover + failback for one pack at a time | ops-sre-reliability |
| Cross-region restore drill (S3 → DR-pair → restore validation) | Monthly | Snapshot-restore + integrity check on a subset of blocks | ops-sre-reliability |
| Tabletop exercise (regional outage scenario) | Annually | Full incident-response + comms + executive briefing | ops-sre-reliability + leadership |
| Chaos engineering injection (random pod kill, AZ partition) | Continuous (Chaos Mesh) | Single AZ / pod-level | ops-sre-reliability |
| Vendor-failure-mode exercise (simulate OCI region outage) | Annually | All packs with that region | ops-sre-reliability |

## RPO / RTO Per Pack

| Pack | RPO target | RTO target | Single-region fallback |
|---|---|---|---|
| pack-kr | ≤ 5 min (intra-region replication only; no DR pair) | ≤ 4h (depends on OCI region recovery; ap-seoul-1 has 3 AZs) | best-effort; OCI region SLA |
| pack-eu | ≤ 5 min (CRR) | ≤ 35 min (DR failover) | – |
| pack-us | ≤ 5 min | ≤ 35 min | – |
| pack-us-healthcare | ≤ 5 min | ≤ 35 min | – |
| pack-jp | ≤ 5 min (no DR pair) | ≤ 4h | best-effort |
| pack-sg | ≤ 5 min (no DR pair) | ≤ 4h | best-effort |
| pack-au | ≤ 5 min | ≤ 35 min | – |
| pack-in | ≤ 5 min | ≤ 35 min | – |
| pack-br | ≤ 5 min | ≤ 35 min | – |
| pack-ae | ≤ 5 min | ≤ 35 min | – |
| pack-ksa | ≤ 5 min | ≤ 35 min | – |

Per-tenant RPO/RTO commitments are part of the tenant SLA (per tenant DPA at `legal/dpa-template.md`, Slice D). Packs without DR pair (pack-kr, pack-jp, pack-sg) have weaker RTO commitments which are disclosed at tenant onboarding.

## Tenant Notification

Tenants are notified at failover initiation per the comms template in `incident-response.md` §"Tenant communications":

- **Status page (public)**: updated within 5 min of failover initiation.
- **Tenant operator email**: sent within 30 min for any Sev-1/2 affecting a tenant's pack.
- **Customer-facing message-template** (for the tenant to forward to its end-users): provided in tenant operator's onboarding portal.
- **Regulatory notification**: per `compliance.md` enforced timelines (GDPR Art. 33 72h; HIPAA §164.404 60d; KR PIPA Art. 34 72h; etc.).

## Per-Pack BCDR Overlay

Per-pack BCDR specifics (region capabilities, specific OCI service mappings, local-regulator BCDR requirements) live at `regional-packs/<pack>/multi-region-overlay.md`. Example: pack-eu must satisfy DORA (Digital Operational Resilience Act 2022/2554) testing requirements when oyatie has EU financial-services tenants in scope.

## Verification

- `cargo run -p oya-dev-cli -- gate validate multi-region-conformance` — exit 0; deployed topology matches this document for every active pack.
- Quarterly DR-failover drill audit log: success vs failure rate trend.
- Annual third-party BCDR audit: alignment with ISO 22301 / NIST SP 800-34 / DORA.

## References

- `microservices/observability/policy/data-residency.md`.
- `microservices/observability/capacity-model.md`.
- `microservices/observability/cost-budget.md`.
- `microservices/observability/failure-modes.md`.
- `microservices/observability/incident-response.md`.
- `regional-packs/<pack>/multi-region-overlay.md` (per-pack).
- OCI region documentation — `oracle.com/cloud/data-regions/`.
- Grafana Mimir HA + DR — `grafana.com/docs/mimir/latest/manage/run-production-environment/`.
- ISO/IEC 22301:2019 (Business continuity).
- NIST SP 800-34 (Contingency planning).
- EU DORA Regulation 2022/2554.
