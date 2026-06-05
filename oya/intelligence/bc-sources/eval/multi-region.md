---
doc_class: MultiRegionPlan
title: Multi-Region Plan
microservice: foundry-eval
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-foundry + council-privacy
deciders: ops-sre-reliability, axis-foundry, council-privacy, council-architecture
related_adrs: [ADR-0024, ADR-0117, ADR-0131]
related_artifacts:
  - microservices/intelligence-eval/capacity-model.md
  - microservices/intelligence-eval/cost-budget.md
  - microservices/intelligence-eval/policy/data-residency.md
review_cadence: per pack activation + annually
doc_status: published
---

# Multi-Region Plan (foundry-eval µservice)

## Purpose

Define per-pack deployment topology, cross-region replication policy, failover procedure, RPO/RTO per region, and residency enforcement. Aligns with ADR-0117 (cloud-native residency) and `policy/data-residency.md`.

## Per-Pack Topology

| Pack | Primary | DR pair | Active components per region |
|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | none (single-region) | Full stack |
| pack-eu | OCI eu-frankfurt-1 | OCI eu-amsterdam-1 (DR) | Primary: full stack; DR: warm-standby ClickHouse + S3 replica + Postgres replica |
| pack-us | OCI us-ashburn-1 | OCI us-phoenix-1 (DR) | Same pattern |
| pack-us-healthcare | HIPAA-eligible US region | HIPAA-eligible US DR | Same pattern; BAA per-region |
| pack-jp | OCI ap-tokyo-1 | none | Full stack |
| pack-sg | OCI ap-singapore-1 | none | Full stack |
| pack-au | OCI ap-sydney-1 | OCI ap-melbourne-1 (DR) | Same pattern |
| pack-in | OCI ap-hyderabad-1 | OCI ap-mumbai-1 (DR) | Same pattern |
| pack-br | OCI sa-saopaulo-1 | OCI sa-vinhedo-1 (DR) | Same pattern |
| pack-ae | OCI me-dubai-1 | none | Full stack |
| pack-ksa | OCI me-jeddah-1 | none | Full stack |

## Cross-Region Replication

### Within Pack (DR pair)

- **Eval-set manifests** (Postgres + S3): synchronous replication; RPO 0; RTO ≤ 5 min for Postgres failover.
- **Baseline outputs** (S3): asynchronous cross-region replication; RPO ≤ 5 min.
- **Replay traces** (S3): asynchronous cross-region replication; RPO ≤ 15 min.
- **ClickHouse parity_analytics**: cross-region streaming replica with 1-second async lag; RPO ≤ 5 min.
- **Per-subject DEKs (KMS)**: per-pack multi-region KMS keyring; DEKs replicated; KEK does not leave the pack.

### Across Pack

**Forbidden by default.** Allowed only with:
1. Tenant-executed SCC (or equivalent local provision).
2. Recorded in `legal/transfer-register.md`.
3. council-privacy + ops-security approval.
4. Pack-routing override flag at tenant level in `tenancy`.

Exceptions for cross-pack:
- **Cosign-verified eval-sets**: read-only replication of eval-set manifests across packs allowed (no tenant data; only the eval-set authoring artifact). Signature integrity preserved.

## Failover Procedure (Per Pack with DR)

### Automated Failover

| Scenario | Detection | Action | RTO |
|---|---|---|---|
| Primary region complete outage | DR pair health probes detect primary unreachable for ≥ 60s | DR cluster promotes Postgres replica; eval-runner-worker resumes on DR; route traffic via Envoy DNS update | ≤ 5 min |
| Primary ClickHouse outage | ClickHouse cluster heath fails | DR replica promoted to primary | ≤ 5 min |
| Primary S3 outage | S3 read/write failure rate > 50% sustained 60s | Route S3 reads to DR mirror | ≤ 2 min |

### Manual Failover

When primary region planned-maintenance overrun or partial degradation:
1. Engage `runbooks/region-failover.md` (cross-cuts `cloud-k8s` µservice).
2. 2-person rule per `policy/two-person-admin-ops.md`.
3. Pre-failover replication catchup verified.
4. DNS cutover via Envoy.
5. Health-verify each component post-cutover.

## RTO / RPO Summary

| Component | RPO | RTO | Notes |
|---|---|---|---|
| Eval-set metadata (Postgres) | 0 | 5 min | Synchronous replication |
| Baseline outputs (S3) | 5 min | 2 min | Async replication |
| Replay traces (S3) | 15 min | 2 min | Async replication |
| ClickHouse parity_analytics | 5 min | 5 min | Cross-region streaming |
| Per-subject DEKs (KMS) | 0 | 5 min | Multi-region keyring |
| Cosign / Rekor verification | 0 | 15 min | Switch to mirrored Rekor |

## Pack-Specific Notes

### pack-kr / pack-jp / pack-sg / pack-ae / pack-ksa (single-region)

- No DR pair available within residency boundary.
- Recovery from full regional outage: provider-recovery-dependent; RTO ≤ 4h typical.
- Cold-backup off-region (within pack residency): for pack-kr, a daily snapshot to a secondary KR availability domain.
- Mitigation: per ADR-0117, single-region packs accept higher RTO; tenant SLA reflects.

### pack-eu (EU AI Act high-risk)

- DR pair eu-frankfurt-1 ↔ eu-amsterdam-1 (both within EU).
- EU AI Act §17 logging: cross-region replication preserves §17 evidence integrity.
- DR failover does not trigger Art. 33 breach notification (controlled operation).

### pack-us-healthcare

- DR pair both HIPAA-eligible regions; BAA covers both.
- HIPAA §164.316(b)(2) 6y retention replicated across DR pair.
- DR failover audit-chain-emitted; no tenant impact.

## DR Drills

Quarterly DR drill per pack with DR pair:

| Drill | Cadence | Owner |
|---|---|---|
| Postgres replica promotion + reversal | Q1, Q3 | ops-sre-reliability |
| ClickHouse cross-region failover | Q2, Q4 | ops-sre-reliability + axis-foundry |
| S3 mirror cutover | Q1, Q3 | ops-sre-reliability |
| Full-pack failover (planned maintenance) | annually | ops-sre-reliability + axis-foundry |

Each drill produces a postmortem-equivalent report in `evidence/dr-drills/<year>/<quarter>-<pack>.md`.

## Cost Implications

Per `cost-budget.md`:
- DR-pair packs: 1.5× base cost (1.0× primary + 0.5× warm-standby).
- Single-region packs: 1.0× base.
- Cross-region replication egress: included in base for within-pack; charged separately if SCC-approved cross-pack.

## Verification

- `buck2 build //:quality-lane-registry-authority-check # lane=multi-region --microservice foundry-eval` exits 0.
- Quarterly DR drill report in `evidence/dr-drills/`.
- Per-tenant residency log review (sample 100 records / quarter).

## References

- ADR-0117 (cloud-native infrastructure + residency).
- `microservices/intelligence-eval/policy/data-residency.md`.
- `microservices/intelligence-eval/capacity-model.md`.
- `microservices/intelligence-eval/cost-budget.md`.
- `microservices/intelligence-eval/runbooks/clickhouse-rebalance.md`.
- `microservices/intelligence-eval/runbooks/baseline-output-restore.md`.
