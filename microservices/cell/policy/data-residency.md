---
doc_class: PolicySpec
title: Data Residency Contract
microservice: cell
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-cell-substrate
deciders: council-privacy, ops-security, axis-cell-substrate, gtm-customer-success
related_adrs: [ADR-0117, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/cell/threat-model.md (T-S-04, T-T-02, T-I-01; cross-pack threats)
  - microservices/cell/dpia.md (R-03; cross-pack-misroute risk)
  - microservices/cell/policy/cell-boundary.md
  - microservices/cell/multi-region.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Data Residency Contract (cell µservice)

## Purpose

Define which jurisdictions' tenant data lives in which cell-set (which pack region), the cross-pack assignment policy, and the legal-transfer mechanisms that gate any exception. Reviewed by EU DPAs (GDPR Arts. 44–50), KR PIPC (PIPA Arts. 28 + 23-2), HIPAA tenants' Covered Entity counsel, and equivalent regulators in every active pack.

## Residency Model

### Default: pack-pinning per tenant

Every tenant is assigned a primary pack at onboarding (per `microservices/tenancy/policy/data-residency.md`). The tenant's cells are then placed in the pack's region(s). Cross-pack cell assignment is **forbidden by default**.

| Pack | Primary region(s) | Cell-set footprint | Activated? |
|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | kr-cell-set-1 | YES (M01 launch) |
| pack-eu | OCI eu-frankfurt-1 + eu-amsterdam-1 (DR pair) | eu-cell-set-{1,2} | Conditional (first EU tenant SCC) |
| pack-us | OCI us-ashburn-1 + us-phoenix-1 | us-cell-set-{1,2} | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | us-hc-cell-set-1; isolated | Conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | jp-cell-set-1 | Conditional |
| pack-sg | OCI ap-singapore-1 | sg-cell-set-1 | Conditional |
| pack-au | OCI ap-sydney-1 + ap-melbourne-1 | au-cell-set-{1,2} | Conditional |
| pack-in | OCI ap-hyderabad-1 + ap-mumbai-1 | in-cell-set-{1,2} | Conditional (DPDPA 2023) |
| pack-br | OCI sa-saopaulo-1 + sa-vinhedo-1 | br-cell-set-{1,2} | Conditional (LGPD) |
| pack-ae | OCI me-abudhabi-1 + me-dubai-1 | ae-cell-set-{1,2} | Conditional |
| pack-ksa | OCI me-jeddah-1 + me-riyadh-1 | ksa-cell-set-{1,2} | Conditional (NCA cloud-residency) |

"Activated?" updated at first-tenant onboarding per pack; activation triggers re-review of this document + per-pack overlays.

### Pack-assignment enforcement at scheduler

```text
Tenant onboarding
    ↓
tenancy: assigns tenant.pack at onboarding (per tenant's HQ jurisdiction + regulated-data flags)
    ↓
TenantOnboarded event → cell.scheduler
    ↓
scheduler: filters candidate cells WHERE cell.pack == tenant.pack
    ↓
binpack placement: best-fit cell within filtered set
    ↓
tenant-assignment write: row inserted; Postgres RLS re-checks pack at commit
    ↓
CellAssigned event → audit-chain Ed25519 seal
```

Cross-pack assignment attempts are refused at three layers (Cedar at REST → Postgres RLS at commit → audit-chain anomaly detection post-commit).

### Per-tenant `cell_scope` influences capacity, not residency

`cell_scope` (shared / dedicated / hipaa-dedicated / sandbox / internal) affects capacity + isolation density within the pack but does not move data across packs.

## Cross-Pack Assignment Policy

### Default: forbidden

Cross-pack cell assignment is forbidden by default. Specifically:

- Tenant→cell assignment: pack-pinned at scheduler decision time.
- Cell migration: only between cells in the same pack.
- Cell-registry writes: per-pack Postgres shard; cross-pack writes refused server-side.
- Host-pool: nodes never bound to a cell outside their pack.

### Exception: tenant-executed SCCs (GDPR Art. 44–46)

Cross-border tenant rehome (rare; e.g., tenant moves HQ jurisdiction) is permitted only when:

1. Active SCC on file at `microservices/cell/legal/transfer-register.md`.
2. Receiving-pack jurisdiction has adequacy decision (GDPR Art. 45) or equivalent safeguard.
3. Migration purpose limited to specifically-named processing (e.g., "tenant relocation to EU pack").
4. Audit-chain-emitted SCC acknowledgement at moment of rehome.
5. Council-privacy + ops-security 2-person rule.
6. Tenant operator explicit consent recorded.

### Exception: HIPAA BAA + DR failover within pack-us-healthcare

Covered Entity tenants in pack-us-healthcare may have cell migration between us-ashburn-1 ↔ us-phoenix-1 (both HIPAA-eligible) within the same pack. Cross-region (us → eu) cell migration is NOT authorised without separate tenant agreement.

### Exception: BCDR exercise (controlled, scheduled)

Controlled intra-pack DR exercises permitted in pack-eu (eu-frankfurt ↔ eu-amsterdam), pack-us (us-ashburn ↔ us-phoenix), etc. Cross-pack BCDR is not authorised.

## Cell Tagging by Jurisdiction

Every cell record carries jurisdiction labels for routing + retention enforcement:

```text
cell_label:
  jurisdiction: kr | eu | us | us-hc | jp | sg | au | in | br | ae | ksa
  pack:         pack-kr | pack-eu | pack-us | ... (mirrors jurisdiction with pack-prefix)
  cell_scope:   shared | dedicated | hipaa-dedicated | sandbox | internal
  cell_state:   requested | provisioning | ready | draining | decommissioned
```

Tampering attempts (cell-label injection from untrusted code) detected by Postgres RLS + Cedar policy.

## Retention by Jurisdiction × Data Class

| Class | Jurisdiction | Hot retention | Cold retention |
|---|---|---|---|
| `SENSITIVE_PIPA_ART23` cell-assignment | kr | 2y hot | indefinite cold; salt rotation 12mo |
| `AUDIT` cell-lifecycle events | kr | indefinite | indefinite |
| `AUDIT` cell-lifecycle events | us-hc | indefinite | ≥ 6y per HIPAA §164.316(b)(2) |
| `AUDIT` cell-lifecycle events | eu | indefinite | indefinite (eIDAS retention) |
| `BEHAVIORAL_TENANT_PRODUCT` migration plans | all packs | 90d hot | 2y cold |
| `INTERNAL_ONLY` cell metadata | all packs | indefinite | indefinite |
| `SECRET` per-cell credentials | all packs | TTL 30d (rotation) | n/a |

Soft-deletion windows (cell + cell-assignment + host-row) ≥ 30d before destructive delete; enables DSR + accidental-delete recovery.

## DSR Cascade (tenant erasure)

When a tenant's `tenancy` row enters `right_to_erasure_requested` state, the cell substrate cascades:

1. `tenant-assignment` BC marks the assignment `releasing`; releases compute + data references to workload µservices.
2. Once all workload µservices acknowledge tenant data deletion (per-µservice DSR contract), `tenant-assignment` flips to `released`.
3. If the cell hosted only this tenant + `cell_scope` is `dedicated`/`hipaa-dedicated`: cell enters `draining` then `decommissioning` lifecycle states.
4. Postgres schema for the cell dropped after 30d soft-delete window.
5. S3 prefix marked for deletion; actual delete after 30d retention-override review.
6. Audit-chain seal at every step; DSR ticket auto-updates with cascade completion.

SLA: end-to-end ≤ 30 days from tenant DSR request → cell schema dropped (longer if multiple cells share); GDPR Art. 17 satisfied.

## Aggregate Aggregations + DP Posture

When cell-utilisation aggregates are surfaced publicly (e.g., capacity dashboards for marketing):

- Per-cell tenant-id is stripped before aggregation.
- DP noise injected: ε ≤ 1.0 per query.
- Cross-tenant adjacency NEVER surfaced (which tenant lives next to which).
- Public dashboards declared in `dashboards/cell-utilization.json` use only `tenant:oya-aggregate` tenant with DP-noise label.

## Verification

- `cargo run -p oya-dev-cli -- gate validate cell-cross-pack-refusal` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate cell-postgres-rls-conformance` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate cedar-fragment-coverage --microservice cell` — exit 0.
- Annual residency audit: third-party validation of pack-pinning + cross-pack refusal.
- Quarterly chaos drill: cross-pack write attempt → assert refusal.

## Per-Pack Overlay

### pack-kr

- KR PIPA Art. 28 (cross-border transfer): tenant data MUST stay in KR cell-set.
- KR PIPA Art. 23-2 (sensitive data outside-of-KR transfer): forbidden without explicit consent; SCC equivalent not currently recognised by KR PIPC.
- KR-FSS tenants: 5y audit retention; KMS-in-KR.

### pack-us-healthcare

- HIPAA §164.314(a)(1) BAA: cell substrate is part of BAA scope; per-tenant BAA documented.
- HIPAA §164.502(e): minimum-necessary disclosure; cell is HIPAA-eligible OCI region only.
- Cross-region (us → eu) migration NOT authorised.

### pack-eu

- GDPR Arts. 44–50 transfers: SCC-only for cross-border; intra-EU adequate.
- Schrems II compatibility: pack-eu cells never replicate to US without supplementary measures.
- NIS2 when applicable.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/cell-residency-overlay.md`.

## References

- ADR-0117 (cloud-native infra + residency).
- ADR-0130 (SLO gate).
- ADR-0131 (per-µservice).
- Bominal ADR-0009 (cell architecture); ADR-0019 (runtime catalog).
- `microservices/cell/threat-model.md`.
- `microservices/cell/dpia.md`.
- `microservices/cell/multi-region.md`.
- `microservices/cell/policy/cell-boundary.md`.
- GDPR Arts. 44–50.
- KR PIPA Arts. 28 + 23-2.
- HIPAA §164.314 + §164.502.
- KR PIPC + EDPB cross-border transfer guidance.
