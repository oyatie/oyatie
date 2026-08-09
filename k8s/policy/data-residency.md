---
doc_class: PolicySpec
title: Data Residency Contract — etcd + PV per pack
microservice: cloud-k8s
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-cloud
deciders: council-privacy, ops-security, axis-cloud, gtm-customer-success
related_adrs: [ADR-0117, ADR-0121, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/cloud-k8s/threat-model.md (T-I-01, T-T-01; cross-pack misroute)
  - microservices/cloud-k8s/dpia.md (R-11 cross-border-misroute)
  - microservices/cloud-k8s/policy/cluster-isolation.md
  - microservices/cloud-k8s/multi-region.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Data Residency Contract (cloud-k8s µservice)

## Purpose

Define which jurisdictions' tenant workload data lives in which cluster's etcd + PV, the cross-pack replication policy, and the legal-transfer mechanisms that gate exceptions. Authoritative reference reviewed by EU DPAs (GDPR Arts. 44–50), KR PIPC (PIPA Art. 28 + Art. 23-2), HIPAA Covered Entity counsel, KR-CSAP, and equivalent regional supervisory authorities.

## Residency Model

### Default: per-pack cluster pinning

Every tenant assigned a primary pack at onboarding. Workload + etcd + PV for that tenant resides in the pack's cluster. Cross-pack movement **forbidden by default**.

| Pack | Primary cluster | DR-pair cluster | Cluster footprint | Activated? |
|---|---|---|---|---|
| pack-kr | `kr-cluster-1` (OCI ap-seoul-1 + on-prem KR primary cell) | — | one cluster, multi-AZ within region | YES (M01 launch) |
| pack-eu | `eu-cluster-1` (eu-frankfurt-1) | `eu-cluster-2` (eu-amsterdam-1) | DR pair | Conditional (post-SCC) |
| pack-us | `us-cluster-1` (us-ashburn-1) | `us-cluster-2` (us-phoenix-1) | DR pair | Conditional |
| pack-us-healthcare | `us-hc-cluster-1` (us-ashburn-1 HIPAA-eligible) | `us-hc-cluster-2` (us-phoenix-1) | DR pair; isolated from non-HC | Conditional (post-BAA) |
| pack-jp | `jp-cluster-1` (ap-tokyo-1) | — | one cluster, multi-AZ | Conditional |
| pack-sg | `sg-cluster-1` (ap-singapore-1) | — | one cluster, multi-AZ | Conditional |
| pack-au | `au-cluster-1` (ap-sydney-1) | `au-cluster-2` (ap-melbourne-1) | DR pair | Conditional |
| pack-in | `in-cluster-1` (ap-hyderabad-1) | `in-cluster-2` (ap-mumbai-1) | DR pair | Conditional (DPDPA 2023) |
| pack-br | `br-cluster-1` (sa-saopaulo-1) | `br-cluster-2` (sa-vinhedo-1) | DR pair | Conditional (LGPD) |
| pack-ae | `ae-cluster-1` (me-abudhabi-1) | `ae-cluster-2` (me-dubai-1) | DR pair | Conditional |
| pack-ksa | `ksa-cluster-1` (me-jeddah-1) | `ksa-cluster-2` (me-riyadh-1) | DR pair | Conditional (KSA NCA) |

The "Activated?" column updates at first-tenant onboarding per pack; activation triggers re-review of this document + per-pack threat-model overlay + DPIA overlay.

### Pack-assignment routing (workload scheduling)

```text
Tenant onboarding (gtm)
    ↓
collects: HQ jurisdiction, regulated-data declarations
    ↓
Pack-router (Cedar policy in cloud-iac µservice):
    - HQ jurisdiction → primary pack
    - Regulated-data flag (PHI, KR-FSS, EU-resident) → may force specific pack
    - Conflict: ops-legal escalation
    ↓
OpenBao assigns tenant → pack
    ↓
Workload µservice deploy specifies `pack_assignment`
    ↓
cloud-k8s `cluster-bootstrap` validates `pack_assignment` matches cluster's pack
    ↓
Schedule into tenant-<hashed-id> namespace in pack cluster
    ↓
etcd + PV in pack only; never cross-pack
```

Routing encoded as Cedar policy at `microservices/cloud-k8s/policy/pack-routing.cedar` (companion to tenant-scope).

## etcd Residency

### Per-cluster etcd

Each cluster has its own etcd quorum (1 node at M01, 3 nodes after M04 per ADR-0121). etcd state is jurisdiction-pinned by virtue of cluster pinning:

- pack-kr etcd → KR data centers only
- pack-eu etcd → EU data centers only
- ...

### etcd encryption-at-rest

Per `policy/cluster-isolation.md` CI-13: kube-apiserver `--encryption-provider-config` uses KMS envelope. KMS key per pack region; key escrow stays within pack jurisdiction. Direct disk read yields ciphertext.

### etcd snapshot storage

Snapshots uploaded to per-pack object storage (`cloud-iac` µservice provisions). Cross-pack snapshot replication **forbidden**. Snapshot retention 14d hot + 90d cold (per pack residency).

## Persistent Volume Residency

### Default: per-pack backend

Each cluster's CSI drivers provision PV from per-pack storage backends:

| Pack | Block-volume backend | Object backend | File backend |
|---|---|---|---|
| pack-kr | OCI Block Volume (KR) + Ceph RBD on-prem | OCI Object (KR) + SeaweedFS on-prem | OCI File (KR) + CephFS on-prem |
| pack-eu | OCI Block Volume (EU) | OCI Object (EU) | OCI File (EU) |
| pack-us | OCI Block Volume (US) | OCI Object (US) | OCI File (US) |
| pack-us-healthcare | OCI Block Volume (HIPAA-eligible) | OCI Object (HIPAA-eligible) | OCI File (HIPAA-eligible) |
| (each pack) | OCI Block Volume (pack region) | OCI Object (pack region) | OCI File (pack region) |

PV lifecycle (provisioning, attach, detach, delete, snapshot) confined to pack cluster + pack backend.

### Cross-pack PV replication: FORBIDDEN

PV replication / migration across packs is forbidden by default. Intra-pack DR-pair replication (e.g., pack-eu eu-frankfurt → eu-amsterdam) is permitted via backend-native CRR.

### PV at-rest encryption

Per-backend:
- Block-volume: backend-side AES-256-GCM with KMS-managed key (per-pack KMS).
- Object: backend-side SSE-KMS.
- File: backend-side encryption-at-rest where supported.

CSI StorageClass enforces encryption requirement; PVCs without encryption-required attribute are refused for tenant namespaces.

## Per-Pack Mimir / Loki / Tempo Tagging

cloud-k8s emits operational metrics + logs + audit events that carry the tenant + pack labels:

```text
metric_label:
  cluster: kr-cluster-1 | eu-cluster-1 | ...
  pack: pack-kr | pack-eu | pack-us | ...
  jurisdiction: kr | eu | us | us-hc | jp | sg | au | in | br | ae | ksa
  tenant_id: tenant:<hashed-id>
  data_class: <Bominal ADR-0028 taxonomy value>
```

These propagate to `observability` µservice's Mimir / Loki / Tempo where per-tenant retention applies (per `observability/policy/data-residency.md`).

## Cross-Pack Replication Policy

### Default: forbidden

- etcd state: forbidden (per-pack pin).
- PV data: forbidden (per-pack backend).
- Container images: Harbor mirror per-pack region; image registry replication intra-region only.
- Kyverno + Cilium + Istio ClusterPolicy: configuration is **global** (git-versioned); each pack applies its own copy at deploy time.
- Audit records: forwarded to per-pack `audit-chain` µservice instance.

### Exception: tenant-executed SCCs (GDPR transfer)

Cross-border transfer of EU-resident workload data permitted only with active SCC per GDPR Arts. 44–46. Requirements:
1. Active SCC in `microservices/cloud-k8s/legal/transfer-register.md`.
2. Receiving pack jurisdiction adequate-decision OR equivalent safeguard.
3. Transfer-purpose specifically named (e.g., "BCDR exercise eu-frankfurt → eu-amsterdam"). Ad-hoc forbidden.
4. audit-chain-emitted SCC-acknowledgement per transfer event.

### Exception: HIPAA BAA + DR

Covered Entity tenants in pack-us-healthcare: DR pair us-ashburn-1 + us-phoenix-1 (both HIPAA-eligible). Cross-region (us → eu) DR NOT authorised without separate tenant agreement.

### Exception: BCDR exercise

Controlled DR failover drills within pack DR pairs are permitted (pack-eu intra-pack, pack-us intra-pack, etc.). Cross-pack BCDR not authorised.

## Retention by Jurisdiction × Asset Class

Retention is the MAX of: cluster default; pack legal minimum; tenant-contracted retention.

| Pack | Asset | Minimum statutory | Default applied |
|---|---|---|---|
| pack-kr | Audit log | KR commercial code: 5y; KR-FSS sector: 5y | 5y |
| pack-kr | etcd snapshot | n/a | 14d hot + 90d cold |
| pack-kr | PV | tenant-declared (per workload µservice DPA) | per declaration |
| pack-eu | Audit log | bounded by purpose | 2y default |
| pack-eu | etcd snapshot | n/a | 14d hot + 90d cold |
| pack-us-healthcare | Audit log | HIPAA §164.316(b)(2): 6y | 6y |
| pack-us-healthcare | etcd snapshot | covers PHI metadata | 14d hot + 6y cold |
| pack-us-healthcare | PV | per BAA + state med-records-retention law | MAX(HIPAA 6y, state, tenant DPA) |
| pack-jp | Audit log | APPI implied minimums | 2y default |
| pack-jp | PV | tenant-declared | per declaration |
| pack-au | Audit log | APRA-CPS 234 (when finance) | 5y for finance; 2y default |
| pack-in | Audit log | DPDPA §11 | 2y default |
| pack-br | Audit log | LGPD Art. 16 | 2y default |
| pack-ae | Audit log | UAE PDPL implied | 2y default |
| pack-ksa | Audit log | KSA PDPL + SAMA CSF (when finance) | 5y for finance; 2y default |

LEAN check `oya-governance-retention-conformance` validates per-pack etcd snapshot + audit-log retention configs against this matrix.

## DSR Cascade

Right-to-erasure (GDPR Art. 17 / PIPA Art. 36 / DPDPA §12 / LGPD Art. 18(V)) honoured via `oya-dsr-cascade-runner`:

1. Tenant raises DSR on behalf of end-user (joint controllership).
2. DSR runner identifies end-user identifiers in:
   - kube-apiserver audit log (filter by user-id in pod-spec env vars; refer to workload µservice for redaction)
   - PV contents (per workload µservice's DSR procedure)
3. cloud-k8s scopes are minimal: only pod-spec metadata mentions of user-ids; the workload data inside containers/PVs is the workload µservice's DSR territory.
4. cloud-k8s's contribution: redact user-ids from kube-apiserver audit log (using audit-policy redact transformer) for forward records; historical records remain audit-chain-immutable per Bominal ADR-0028.
5. SLA: 30d from request.

Limitations (per DPIA R-07):
- Historical audit records pre-DSR are immutable (audit-chain seal); we redact forward records only.
- PV contents owned by workload µservice; cloud-k8s coordinates via DSR runner.

## Per-Pack Overlays

### pack-kr (KR PIPA + ISMS-P + CSAP)

- **PIPA Art. 28 (storage period)**: bounded per matrix.
- **PIPA Art. 23-2 (sensitive cross-border)**: forbidden by default; SCC + consent required.
- **PIPC Notice 2020-7 (overseas-transfer notification)**: pack-kr residency guarantee in tenant DPA.
- **KR-FSS sector**: audit log ≥ 5y; KMS keys in KR-resident KMS.
- **KR CSAP**: pack-kr cluster + pack-kr etcd + pack-kr PV (all in-country).

### pack-eu (GDPR + EDPB + Schrems II)

- **GDPR Arts. 44–46**: SCC-only; Adequacy decision via EU-list; supplementary measures (pseudonymisation + EU-resident KMS) per Schrems II.
- **EDPB Recommendations 01/2020**: supplementary measures at `microservices/cloud-k8s/legal/schrems-supplementary-measures.md`.
- **GDPR Arts. 25 + 32**: pseudonymisation (hashed tenant-id) + EU-resident-KMS encryption-at-rest constitute "appropriate technical measures."

### pack-us-healthcare (HIPAA)

- **45 CFR §164.530(j) (retention)**: ≥ 6y for audit-relevant records.
- **HIPAA-eligible regions only**: OCI us-ashburn-1 + us-phoenix-1 per Oracle attestation.
- **BAA required**: tenant signs BAA before pack-us-healthcare onboarding.
- **TPO scope**: cluster operations fall under Operations.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/cloud-k8s-residency-overlay.md`.

## Verification

- `cargo run -p oya-dev-cli -- gate validate data-residency-conformance --microservice cloud-k8s` — exit 0.
- Quarterly per-pack residency audit.
- Annual cross-border transfer review.

## References

- `microservices/cloud-k8s/threat-model.md`.
- `microservices/cloud-k8s/dpia.md`.
- `microservices/cloud-k8s/policy/cluster-isolation.md`.
- `microservices/cloud-k8s/multi-region.md`.
- ADR-0117, ADR-0121, ADR-0139, ADR-0131.
- GDPR Arts. 44–50; EDPB Recommendations 01/2020.
- KR PIPA Arts. 23, 23-2, 28; PIPC Notice 2020-7.
- HIPAA §164.530(j); 45 CFR Part 164.
- KR-CSAP cloud-residency guidance.
