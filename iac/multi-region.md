---
doc_class: MultiRegionPlan
title: Multi-Region Topology + BCDR
microservice: cloud-iac
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-cloud-iac + cloud-k8s + cloud-secrets
deciders: ops-sre-reliability, axis-cloud-iac, architecture-governance, privacy-governance
related_adrs: [ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - iac/policy/data-residency.md
  - iac/capacity-model.md
  - iac/cost-budget.md
  - iac/failure-modes.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Multi-Region Topology + BCDR (cloud-iac µservice)

## Purpose

Define the multi-region topology for cloud-iac across the 11 oyatie packs: pack-pinning, in-pack DR pair (where applicable), cross-pack replication-forbidden policy, BCDR posture, RPO/RTO targets per region, failover procedures. Authoritative reference for ops-sre-reliability on-call during region outages and for auditors verifying business-continuity claims.

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
│  │ ArgoCD (active)          │            │ ArgoCD (warm-standby)    │    │
│  │  - app-controller HA     │   replic   │  - same; 0.6× capacity   │    │
│  │  - repo-server / server  │ ◀────────▶ │  - valkey sentinel cluster│    │
│  │  - valkey sentinel cluster│   intra-   │                          │    │
│  └──────────────────────────┘   pack     └──────────────────────────┘    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ Postgres iac-state-index │            │ Postgres replica         │    │
│  │  primary + read-replica  │  streaming │  + read-replica (warm)   │    │
│  │  + WAL streaming to S3   │            │  + WAL receive           │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ OpenTofu state buckets   │  S3 CRR    │ State buckets (replica)  │    │
│  │  (SSE-KMS per pack)      │ ◀────────▶ │                          │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ Flux + Helm-ctrl +       │            │ Flux + Helm-ctrl +       │    │
│  │ Kustomize-ctrl (active)  │            │ Kustomize-ctrl (warm)    │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ Layer-B workers          │            │ Layer-B workers (warm)   │    │
│  │ (renderer/validator/     │            │  0.6× replica count      │    │
│  │  applier/rollback/       │            │                          │    │
│  │  registry)               │            │                          │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│                                                                          │
│  Global Traffic Manager (per-pack DNS):                                  │
│  - Health check on ArgoCD application-controller + iac-registry-rest     │
│  - On failure: DNS failover → DR pair (TTL 60s)                          │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

### Replication

| Component | Mode | RPO | Cross-region |
|---|---|---|---|
| Postgres iac-state-index | Streaming replication primary→replica + S3 archive WAL | ≤ 30s | intra-pack only |
| OpenTofu state buckets | Async via S3 cross-region replication (CRR) | ≤ 5min | intra-pack only |
| ArgoCD application records | etcd-backed; replicated via valkey sentinel | ≤ 30s | intra-pack only |
| ArgoCD Applications (manifest content) | Git-versioned; source of truth | 0 | global (manifests not tenant data) |
| Helm chart artifacts | Sigstore Rekor public log + per-pack registry cache | 0 (declarative) | intra-pack mirror only |
| iac-state-index Postgres backups | S3 archive per pack with versioning | ≤ 5min | intra-pack only |
| Cedar policies | Git-versioned | 0 | global |
| Workspace Cargo.toml + IaC sources | Git-versioned | 0 | global |
| Apply audit events (audit-chain) | per audit-chain µservice's residency contract | inherits | inherits |

### Cross-pack replication: FORBIDDEN by default

Per `policy/data-residency.md`, no cloud-iac state crosses pack boundaries. Narrow exceptions (tenant-executed SCCs for GDPR; tenant-specific BCDR exercise) documented inline.

## Failover Procedures

### Primary-region degraded (Sev-2)

1. Detection: ArgoCD or Postgres `request_failures_total > threshold` for ≥ 5min; or AZ-level OCI outage announced.
2. ops-sre-reliability on-call paged.
3. Verify failure scope (component-level vs region-level).
4. If component-level: scale out unaffected components; await OCI recovery (see `failure-modes.md` FM-05 / FM-12).
5. If region-level + DR pair exists: initiate DR failover (Step §"DR Failover" below).
6. If region-level + no DR pair (pack-kr / pack-jp / pack-sg): graceful degradation; tenants notified; await OCI region recovery.

### DR Failover (packs with DR pair)

| Phase | Step | Time budget |
|---|---|---|
| 1 | Verify DR-pair region is healthy (ArgoCD + Postgres write paths reachable) | ≤ 2min |
| 2 | Promote DR-pair Postgres replica to primary (streaming-replication promote) | ≤ 5min |
| 3 | Promote DR-pair ArgoCD application-controller to active (HPA scales to primary capacity) | ≤ 10min |
| 4 | Update Global Traffic Manager: DNS records to DR-pair endpoints | ≤ 1min (TTL 60s) |
| 5 | Update workload-cluster kubeconfigs (re-point applier to DR-pair ArgoCD) | ≤ 5min Helm rollout |
| 6 | Verify Layer-B workers reconnected and processing | ≤ 5min |
| 7 | Promote DR-pair Flux + Helm-controller + Kustomize-controller | ≤ 5min |
| 8 | Verify drift-detection resumes; apply-state index writes resume | ≤ 5min |
| 9 | Notify tenants per `incident-response.md` template | ≤ 30min |
| 10 | Engage OCI on primary-region restoration | ongoing |
| **Total** | **end-to-end DR failover** | **≤ 35min** (RTO target) |

RPO: ≤ 5min (S3 CRR cadence; Postgres streaming-replication lag ≤ 30s).
RTO: ≤ 35min.

### Failback (after primary region recovers)

Failback to primary is **manual** and scheduled (not auto-failback) per industry best-practice; primary region must non-production samplenstrate ≥ 6h healthy state before failback initiated. Procedure mirrors DR Failover in reverse.

## BCDR Exercise Cadence

| Exercise | Cadence | Scope | Owner |
|---|---|---|---|
| DR failover drill (controlled, off-hours) | Quarterly per pack with DR pair | Full failover + failback for one pack | ops-sre-reliability |
| Cross-region restore drill (S3 → DR-pair → restore validation) | Monthly | Snapshot-restore + integrity check on a subset of state | ops-sre-reliability |
| Tabletop exercise (regional outage scenario) | Annually | Full incident-response + comms + executive briefing | ops-sre-reliability + leadership |
| Chaos engineering injection (random pod kill, AZ partition) | Continuous (Chaos Mesh) | Single AZ / pod-level | ops-sre-reliability |
| Vendor-failure-mode exercise (simulate OCI region outage) | Annually | All packs with that region | ops-sre-reliability |
| Sigstore upstream-outage drill (FM-07) | Annually | Cache-replay against Rekor cache | ops-security |

## RPO / RTO Per Pack

| Pack | RPO target | RTO target | Single-region fallback |
|---|---|---|---|
| pack-kr | ≤ 5min (intra-region replication only) | ≤ 4h (depends on OCI region recovery; ap-seoul-1 has 3 AZs) | best-effort; OCI region SLA |
| pack-eu | ≤ 5min (CRR + streaming) | ≤ 35min (DR failover) | – |
| pack-us | ≤ 5min | ≤ 35min | – |
| pack-us-healthcare | ≤ 5min | ≤ 35min | – |
| pack-jp | ≤ 5min (no DR pair) | ≤ 4h | best-effort |
| pack-sg | ≤ 5min (no DR pair) | ≤ 4h | best-effort |
| pack-au | ≤ 5min | ≤ 35min | – |
| pack-in | ≤ 5min | ≤ 35min | – |
| pack-br | ≤ 5min | ≤ 35min | – |
| pack-ae | ≤ 5min | ≤ 35min | – |
| pack-ksa | ≤ 5min | ≤ 35min | – |

Per-tenant RPO/RTO commitments part of tenant SLA; packs without DR pair have weaker RTO commitments disclosed at onboarding.

## Tenant Notification

Per `incident-response.md` §"Tenant communications":

- **Status page (public)**: updated within 5min of failover initiation.
- **Tenant operator email**: sent within 30min for Sev-1/2 affecting pack.
- **Regulatory notification**: per `compliance.md` enforced timelines.

## Per-Pack BCDR Overlay

Per-pack BCDR specifics at `regional-packs/<pack>/cloud-iac-multi-region-overlay.md`. Example: pack-eu must satisfy DORA (Digital Operational Resilience Act 2022/2554) testing requirements when oyatie has EU financial-services tenants in scope.

## Verification

- cloud-ci/oya-ci governance gate `multi-region-conformance` for --microservice cloud-iac is green in the branch-protected `oya-ci-required` context — exit 0; deployed topology matches this document for every active pack.
- Quarterly DR-failover drill audit log: success vs failure rate trend.
- Annual third-party BCDR audit: alignment with ISO 22301 / NIST SP 800-34 / DORA.

## References

- `iac/policy/data-residency.md`.
- `iac/capacity-model.md`.
- `iac/cost-budget.md`.
- `iac/failure-modes.md`.
- `iac/incident-response.md`.
- `microservices/observability/multi-region.md` (parent template).
- OCI region documentation — `oracle.com/cloud/data-regions/`.
- ArgoCD HA — `argo-cd.readthedocs.io/en/stable/operator-manual/high_availability/`.
- ISO/IEC 22301:2019 (Business continuity).
- NIST SP 800-34 (Contingency planning).
- EU DORA Regulation 2022/2554.
