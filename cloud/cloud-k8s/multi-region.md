---
doc_class: MultiRegionPlan
title: Multi-Region Topology + BCDR
microservice: cloud-k8s
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-cloud + cloud-iac
deciders: ops-sre-reliability, axis-cloud, council-architecture, council-privacy
related_adrs: [ADR-0117, ADR-0121, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/cloud-k8s/policy/data-residency.md
  - microservices/cloud-k8s/capacity-model.md
  - microservices/cloud-k8s/cost-budget.md
  - microservices/cloud-k8s/failure-modes.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Multi-Region Topology + BCDR (cloud-k8s µservice)

## Purpose

Define the multi-region topology for cloud-k8s across the 11 oyatie packs: pack-pinning, in-pack DR pair (where applicable), cross-pack replication-forbidden policy, BCDR posture, RPO/RTO per region, failover procedures, and intra-pack DR procedures. Authoritative reference for ops-sre-reliability during region outages and for auditors verifying business-continuity claims.

## Topology Per Pack

| Pack | Primary cluster (region) | DR-pair cluster (warm) | Single-region? | Activation |
|---|---|---|---|---|
| pack-kr | `kr-cluster-1` (OCI ap-seoul-1 + on-prem KR primary cell per ADR-0121) | — | YES | YES (M01 launch) |
| pack-eu | `eu-cluster-1` (eu-frankfurt-1) | `eu-cluster-2` (eu-amsterdam-1) | DR pair | Conditional (post-SCC) |
| pack-us | `us-cluster-1` (us-ashburn-1) | `us-cluster-2` (us-phoenix-1) | DR pair | Conditional |
| pack-us-healthcare | `us-hc-cluster-1` (us-ashburn-1 HIPAA-eligible) | `us-hc-cluster-2` (us-phoenix-1) | DR pair | Conditional (post-BAA) |
| pack-jp | `jp-cluster-1` (ap-tokyo-1) | — | YES | Conditional |
| pack-sg | `sg-cluster-1` (ap-singapore-1) | — | YES | Conditional |
| pack-au | `au-cluster-1` (ap-sydney-1) | `au-cluster-2` (ap-melbourne-1) | DR pair | Conditional |
| pack-in | `in-cluster-1` (ap-hyderabad-1) | `in-cluster-2` (ap-mumbai-1) | DR pair | Conditional |
| pack-br | `br-cluster-1` (sa-saopaulo-1) | `br-cluster-2` (sa-vinhedo-1) | DR pair | Conditional |
| pack-ae | `ae-cluster-1` (me-abudhabi-1) | `ae-cluster-2` (me-dubai-1) | DR pair | Conditional |
| pack-ksa | `ksa-cluster-1` (me-jeddah-1) | `ksa-cluster-2` (me-riyadh-1) | DR pair | Conditional |

## In-Pack DR-Pair Architecture

```text
┌─ Pack <X> ─────────────────────────────────────────────────────────────────┐
│                                                                            │
│  Primary region                          DR-pair region                    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐      │
│  │ Cluster (active)         │            │ Cluster (warm-standby)   │      │
│  │  - kubeadm 1.35          │  Istio     │  - same versions         │      │
│  │  - 3-node etcd HA        │ multi-     │  - 3-node etcd HA (small)│      │
│  │  - 17+ worker nodes      │ cluster    │  - 10 pre-warmed workers │      │
│  │  - 1.0× capacity         │ mesh +     │  - 0.6× capacity         │      │
│  │  - Cilium + Istio + CSI  │ mTLS only  │  - same components       │      │
│  └──────────────────────────┘            └──────────────────────────┘      │
│  ┌──────────────────────────┐            ┌──────────────────────────┐      │
│  │ etcd snapshot store      │  CRR       │ etcd snapshot store      │      │
│  │  (per-pack object store) │ ◀────────▶ │  (replicated)            │      │
│  └──────────────────────────┘            └──────────────────────────┘      │
│  ┌──────────────────────────┐            ┌──────────────────────────┐      │
│  │ PV backends:             │  CRR       │ PV backends (replicated) │      │
│  │  block / object / file   │ ◀────────▶ │                          │      │
│  └──────────────────────────┘            └──────────────────────────┘      │
│                                                                            │
│  Per-pack Global Traffic Manager (DNS):                                    │
│  - Health check on primary cluster's api-proxy + Envoy ingress             │
│  - On failure: DNS failover → DR-pair endpoints (≤ 60s TTL)                │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

### Replication

| Component | Mode | RPO | Cross-region scope |
|---|---|---|---|
| etcd snapshots (object) | Async via OCI Object S3 CRR | ≤ 5 min | intra-pack only |
| PV block-volume | Backend-native async CRR | ≤ 15 min | intra-pack only |
| PV object | Backend-native CRR | ≤ 5 min | intra-pack only |
| PV file | Backend-native sync where supported | ≤ 1 min | intra-pack only |
| Cilium + Istio + CSI configuration | Git-versioned + Kustomize-deployed on both sides | 0 (declarative) | intra-pack |
| Kyverno ClusterPolicies | Git-versioned | 0 | global (policy is not tenant data) |
| Cedar fragments | Git-versioned | 0 | global |
| Workspace Cargo.toml + IaC | Git-versioned | 0 | global |

### Cross-pack replication: FORBIDDEN

Per `policy/data-residency.md`, no tenant data crosses pack boundaries. Narrow exceptions (tenant-executed SCCs; tenant-specific BCDR exercise) documented inline. **EU-resident tenant data never reaches a non-EU cluster without Schrems-II-compatible SCC + supplementary measures on file.**

## Failover Procedures

### Primary-region degraded (Sev-2)

1. Detection: kube-apiserver / etcd / Cilium / Istio component health degrades; AZ-level OCI outage.
2. ops-sre-reliability on-call paged.
3. Verify scope (component-level vs region-level).
4. Component-level: scale-out unaffected components; await OCI recovery (per `failure-modes.md`).
5. Region-level + DR pair: initiate DR failover (below).
6. Region-level + no DR pair (pack-kr / pack-jp / pack-sg): graceful degradation; tenants notified; await OCI region recovery.

### DR Failover (packs with DR pair)

| Phase | Step | Time budget |
|---|---|---|
| 1 | Verify DR-pair cluster healthy (control-plane Ready + sample pod scheduling works) | ≤ 2 min |
| 2 | Restore most-recent etcd snapshot to DR-pair etcd (if needed) | ≤ 15 min |
| 3 | Scale DR-pair worker nodes from 0.6× → 1.0× via HPA + node-add | ≤ 10 min |
| 4 | Promote DR-pair PV backends from secondary to primary | ≤ 10 min |
| 5 | Update Global Traffic Manager: DNS records → DR-pair Envoy gateway | ≤ 1 min (TTL 60s) |
| 6 | Update workload µservices' deployment manifests (if endpoints embedded) | ≤ 5 min |
| 7 | Verify mesh federation: DR-pair istiod is now primary; cross-cluster mTLS still valid | ≤ 5 min |
| 8 | Verify pod scheduling resumes; tenants reachable via new endpoint | ≤ 5 min |
| 9 | Notify tenants of failover (status page + email per `incident-response.md`) | ≤ 30 min |
| 10 | Engage OCI on primary-region restoration | ongoing |
| **Total** | **end-to-end DR failover** | **≤ 50 min** (RTO target) |

RPO: ≤ 15 min (PV block backend); ≤ 5 min (etcd snapshot CRR + PV object/file CRR).
RTO: ≤ 50 min (DR failover complete; tenant traffic stable on DR-pair).

### Failback (after primary region recovers)

Failback is **manual** and scheduled. Primary region must demonstrate ≥ 6h of healthy state before failback initiated. Procedure mirrors DR failover in reverse; warm-standby (now primary) becomes warm again.

## BCDR Exercise Cadence

| Exercise | Cadence | Scope | Owner |
|---|---|---|---|
| DR failover drill (controlled, off-hours) | Quarterly per pack with DR pair | Full failover + failback for one pack at a time | ops-sre-reliability |
| etcd snapshot-restore drill | Monthly | Snapshot integrity + restore-to-fresh-cluster | ops-sre-reliability |
| Tabletop exercise (regional outage scenario) | Annually | Full incident-response + comms + executive briefing | ops-sre-reliability + leadership |
| Chaos engineering injection (random node kill, AZ partition) | Continuous (Chaos Mesh) | Single AZ / pod / node level | ops-sre-reliability |
| Vendor-failure-mode exercise (simulate OCI region outage) | Annually | All packs with that region | ops-sre-reliability |
| Kubeadm upgrade dry-run in DR cluster | Per minor-version-bump | Validate upgrade procedure | axis-cloud |

## RPO / RTO Per Pack

| Pack | RPO target | RTO target | Single-region fallback |
|---|---|---|---|
| pack-kr | ≤ 5 min (intra-region replication only) | ≤ 4h (OCI region recovery) | best-effort; OCI region SLA |
| pack-eu | ≤ 5 min (CRR) | ≤ 50 min (DR failover) | – |
| pack-us | ≤ 5 min | ≤ 50 min | – |
| pack-us-healthcare | ≤ 5 min | ≤ 50 min | – |
| pack-jp | ≤ 5 min | ≤ 4h | best-effort |
| pack-sg | ≤ 5 min | ≤ 4h | best-effort |
| pack-au | ≤ 5 min | ≤ 50 min | – |
| pack-in | ≤ 5 min | ≤ 50 min | – |
| pack-br | ≤ 5 min | ≤ 50 min | – |
| pack-ae | ≤ 5 min | ≤ 50 min | – |
| pack-ksa | ≤ 5 min | ≤ 50 min | – |

Per-tenant RPO/RTO commitments are part of tenant SLA (per tenant DPA at `legal/dpa-template.md`). Packs without DR pair (pack-kr, pack-jp, pack-sg) have weaker RTO commitments disclosed at tenant onboarding.

## Tenant Notification

Notified at failover initiation per `incident-response.md` §"Tenant communications":
- **Status page (public)**: updated ≤ 5 min of failover initiation.
- **Tenant operator email**: sent ≤ 30 min for any Sev-1/2 affecting a tenant's pack.
- **Customer-facing message-template**: provided in tenant onboarding portal.
- **Regulatory notification**: per `compliance.md` (GDPR Art. 33 72h; HIPAA §164.404 60d; KR PIPA Art. 34 72h; etc.).

## Per-Pack BCDR Overlay

Per-pack BCDR specifics at `regional-packs/<pack>/cloud-k8s-multi-region-overlay.md`. Example: pack-eu must satisfy DORA (Regulation 2022/2554) testing requirements when EU financial-services tenants are in scope; pack-au must satisfy APRA-CPS 232 (business-continuity) for financial-services tenants.

## Verification

- `cargo run -p oya-dev-cli -- gate validate multi-region-conformance --microservice cloud-k8s` — exit 0.
- Quarterly DR-failover drill audit log: success vs failure rate trend.
- Annual third-party BCDR audit: alignment with ISO 22301 / NIST SP 800-34 / DORA / APRA-CPS 232.

## References

- `microservices/cloud-k8s/policy/data-residency.md`.
- `microservices/cloud-k8s/capacity-model.md`.
- `microservices/cloud-k8s/cost-budget.md`.
- `microservices/cloud-k8s/failure-modes.md`.
- `microservices/cloud-k8s/incident-response.md`.
- `regional-packs/<pack>/cloud-k8s-multi-region-overlay.md` (per-pack).
- OCI region documentation — `oracle.com/cloud/data-regions/`.
- Kubernetes DR — `kubernetes.io/docs/tasks/administer-cluster/`.
- Istio multi-cluster — `istio.io/latest/docs/setup/install/multicluster/`.
- ISO/IEC 22301:2019 (Business continuity); NIST SP 800-34; EU DORA Regulation 2022/2554; APRA-CPS 232.

---

## ADR-0158 Multi-Region Disposition Statement

**Disposition: `active_passive` per cell (cluster control-plane is per-cell).**

Per ADR-0158, the cloud-k8s µservice's cluster control-plane runs per cell. Cross-cell failover uses the global anycast routing layer; intra-cell HA via 3+ control-plane nodes per kubeadm topology (per ADR-0121).

| Property | Value |
|---|---|
| Disposition | `active_passive` per cell |
| RPO (intra-region) | ≤ 30 seconds |
| RTO (intra-region) | ≤ 5 minutes (Kubernetes control-plane leader-election) |
| Sovereign-pin behavior | cells deploy per sovereign region only |
| Cross-region transaction policy | forbidden (control-plane is per-cell) |

## ADR-0164 Sovereign Cloud / Air-Gapped Deployment Variant

Per ADR-0164, the cloud-k8s µservice ships a per-pack air-gap variant for sovereign packs. The variant flips the following dependencies:

### Container registry (in-cell)

- **Tool**: Harbor 2.x (CNCF graduated) at `registry.{cell}.svc.cluster.local`.
- **Image pull policy**: `IfNotPresent`.
- **Image reference rewrite**: `registry.{cell}.svc.cluster.local/oya/<ms>:<tag>` (kustomize component rewrites).
- **Pre-flight mirror job**: external build registry → per-cell Harbor BEFORE the cell loses external egress. Helm chart at `microservices/cloud-iac/iac/helm/harbor-mirror/`.
- **Signature verification**: Sigstore Cosign + Kyverno admission controller (per ADR-0146 + SLSA L3).

### No external API egress

- NetworkPolicy + Cilium L7 egress deny all external hosts by default.
- DNS resolution via in-cell CoreDNS.
- NTP via in-cell chrony.
- OCSP / CRL via in-cell PKI.
- Telemetry: Datadog / Honeycomb / New Relic forbidden in air-gap mode; observability µservice (Prometheus/Mimir + Tempo + Loki) is the only sink.

### CI runner option

- Sovereign tenants may require in-region CI runners. Per-pack overlay points deploy pipeline at in-region self-hosted runners in a separate "build cell" with the same air-gap shape.

### Pack matrix (cloud-k8s perspective)

| Pack | `air_gap` | Container registry |
|---|---|---|
| `pack-eu-sovereign-airgap` | true | in-cell Harbor (EU region) |
| `pack-kr-fsc` | true | in-cell Harbor (KR region) |
| `pack-kr-public` | true | in-cell Harbor (KR region) |
| `pack-ksa` | true | in-cell Harbor (KSA region) |
| `pack-uae` | true | in-cell Harbor (UAE region) |
| `pack-us-gov` | true | in-cell Harbor (US-Gov region) |
| `pack-us-shared` | false | external (ghcr.io / gcr.io) |
| `pack-eu` | false | external (EU-region only) |
| `pack-kr` | false | external (KR-region only) |
| `pack-jp` | false | external (JP-region only) |

CI lane `oya gate validate air-gap-overlay` enforces (a) air-gap packs reference no external host in ServiceEntry / NetworkPolicy egress, (b) image refs rewritten to in-cell Harbor, (c) foundry-providers external LLM client code absent from air-gap pack image build, (d) per-pack compliance attestation present at `microservices/governance/catalog/pack-{name}-air-gap-attestation.md`.

See `/specs/sovereign-cloud-air-gapped-canonical.json` for the canonical declaration.

## ADR-0161 Canonical StorageClass + CSI Driver Matrix

Per ADR-0161, this pack ships canonical StorageClass manifests at `iac/kustomize/components/storage-classes/` (catalog) and per-pack overlays at `iac/kustomize/components/pack-{name}/` (CSI driver binding).

Canonical names declared at workload µservice level: `oya-pg-hot`, `oya-pg-warm`, `oya-pg-cold`, `oya-valkey-hot`, `oya-s3-warm`, `oya-s3-cold`.

Per-pack overlay binds each canonical name to a concrete CSI driver per the matrix in `/specs/csi-storage-class-canonical.json`. CI lane `oya gate validate storage-class-canonical` enforces.
