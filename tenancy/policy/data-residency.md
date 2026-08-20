---
doc_class: PolicySpec
title: Data Residency Contract
microservice: tenancy
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-tenancy
deciders: council-privacy, ops-security, axis-tenancy, gtm-customer-success
related_adrs: [ADR-0018, ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/tenancy/threat-model.md (T-I-04 / T-T-04 / T-L-10; cross-pack misroute threats)
  - microservices/tenancy/dpia.md (R-04 cross-border misroute)
  - microservices/tenancy/policy/rls-isolation.md
  - microservices/tenancy/multi-region.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Data Residency Contract (tenancy µservice)

## Purpose

Define which jurisdictions' tenant data lives in which Postgres + Citus + Patroni cluster, the cross-pack replication policy, and the legal-transfer mechanisms that gate any exception. **Tenancy is the load-bearing residency authority** — when tenancy assigns a tenant to a pack, every downstream µservice inherits that residency commitment via the pack tag on the JWT + on Workflow events.

This document is the canonical residency artifact reviewed by EU DPAs (per GDPR Arts. 44–50), the Korean PIPC (per PIPA Art. 28 + Art. 23-2), HIPAA tenants' Covered Entity counsel (per BAA), and equivalent supervisory authorities in every active pack.

## Residency Model

### Default: pack-pinning at creation time

Every tenant is assigned a primary pack at creation. The tenant's metadata + RLS-policy state + cell-assignment record are stored in the pack's region-pinned Postgres + Citus cluster. Cross-pack movement is **forbidden by default** and structurally prevented: tenancy crates do not implement any cross-pack write path; cross-pack DB connections are not configured in any tenancy Helm chart's values.

| Pack | Primary region(s) | Postgres + Citus footprint | Activated? |
|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | kr-postgres-{primary,sync,sync}, kr-citus-coord, kr-citus-workers-N | YES (M01 launch tenant) |
| pack-eu | OCI eu-frankfurt-1 + eu-amsterdam-1 (DR pair) | eu-postgres-{primary,sync,sync,async,async} cross-AZ, eu-citus cluster + DR mirror | Conditional (first EU tenant SCC) |
| pack-us | OCI us-ashburn-1 + us-phoenix-1 (DR pair) | us-postgres + us-citus cluster + DR | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | us-hc-postgres + us-hc-citus; isolated from non-HC pack-us | Conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | jp-postgres + jp-citus | Conditional |
| pack-sg | OCI ap-singapore-1 | sg-postgres + sg-citus | Conditional |
| pack-au | OCI ap-sydney-1 + ap-melbourne-1 | au-postgres + au-citus + DR | Conditional |
| pack-in | OCI ap-hyderabad-1 + ap-mumbai-1 | in-postgres + in-citus + DR | Conditional (DPDPA 2023) |
| pack-br | OCI sa-saopaulo-1 + sa-vinhedo-1 | br-postgres + br-citus + DR | Conditional (LGPD) |
| pack-ae | OCI me-abudhabi-1 + me-dubai-1 | ae-postgres + ae-citus + DR | Conditional |
| pack-ksa | OCI me-jeddah-1 + me-riyadh-1 | ksa-postgres + ksa-citus + DR | Conditional (KSA NCA cloud-residency requirements) |

The "Activated?" column is updated at first-tenant onboarding per pack; activation triggers re-review of this document + the per-pack threat-model overlay + DPIA overlay.

### Pack-assignment routing

```text
Tenant onboarding
    ↓
gtm-customer-success: collects tenant HQ jurisdiction + regulated-data declarations
    ↓
Pack-router (Cedar policy in cloud-iac at policy/pack-routing.cedar):
    - HQ jurisdiction → primary pack
    - Regulated-data flag (PHI, KR-FSS, EU-resident, etc.) → may force secondary pack
    - Conflict: ops-legal escalation
    ↓
OpenBao assigns canonical_tenant_id → pack
    ↓
Tenancy creates Tenant{tenant_id, jurisdiction_code, pack, ...} record in the pack's Postgres
    ↓
JWT carries `pack` claim (advisory; tenancy-rest verifies pack on every request)
    ↓
All telemetry + lifecycle events tagged with `pack`; never cross-pack
```

### Per-tenant tenant_scope influences capacity, not residency

Pack-pinning is invariant per tenant; `tenant_scope` (trial / production / sandbox / internal) affects capacity allocation within the pack but does not move data across packs.

### Jurisdiction code immutability

Once a tenant is created, its `jurisdiction_code` is **immutable**. Changing jurisdiction requires:
1. New tenant_id under the new jurisdiction.
2. Tenant-supervised migration of operational data via tenant-API export + re-import (oyatie does not move data across packs as a service).
3. DSR cascade on the old tenant_id concurrent with the new tenant's activation.
4. DPO + ops-security 2-person rule + audit-chain seal on the entire flow.

This restriction prevents accidental + adversarial jurisdiction-shopping.

## Cross-Pack Replication Policy

### Default: forbidden

Cross-pack replication of any tenant data is forbidden by default. Specifically:

- Postgres replication: only within-pack (Patroni primary → sync + async replicas all in the same pack region or DR pair within-pack).
- Citus shard moves: only within-pack (between coordinator + workers within the pack cluster).
- Audit-chain seals: replicate within-pack only (each pack has its own audit-chain instance per the `audit-chain` µservice's residency contract).
- Tenancy Cargo workspace + Helm charts: configuration is global (git-tracked); operational state is per-pack.

### Exception: tenant-executed SCCs (GDPR transfer mechanism)

Cross-border transfer of EU-resident data is permitted only when the tenant has executed an active Standard Contractual Clause (SCC) or equivalent transfer mechanism per GDPR Arts. 44–46. Requires:

1. Active SCC on file at `microservices/tenancy/legal/transfer-register.md`.
2. Receiving-pack jurisdiction has adequate-decision (GDPR Art. 45) or equivalent safeguard.
3. Transfer-purpose limited to specifically-named processing (e.g., "DR failover to pack-us"); ad-hoc transfer not authorised.
4. Audit-chain-emitted SCC-acknowledgement at the moment of transfer (every transfer event sealed).
5. Schrems-II supplementary technical measures in place (pseudonymisation + encryption-at-rest with EU-controlled KMS keys).

### Exception: HIPAA BAA + DR failover

Covered Entity tenants in pack-us-healthcare may have DR pair us-ashburn-1 + us-phoenix-1; failover between the pair is intra-region from a HIPAA perspective (both HIPAA-eligible OCI regions). Cross-region (us → eu) failover is NOT authorised without separate tenant agreement.

### Exception: BCDR exercise (controlled, scheduled)

For BCDR validation, controlled cross-region restore drills are permitted in pack-eu (eu-frankfurt-1 → eu-amsterdam-1) and pack-us (us-ashburn-1 → us-phoenix-1) — intra-pack only. Cross-pack BCDR is not authorised.

## Per-Pack Jurisdiction Tagging

Every tenant record carries jurisdiction + pack tags for routing + retention enforcement:

```yaml
tenant_metadata:
  tenant_id: tenant:<hashed-id>
  jurisdiction_code: KR | EU | US | US-HC | JP | SG | AU | IN | BR | AE | KSA
  pack:              pack-kr | pack-eu | pack-us | pack-us-healthcare | ...
  cell_id:           <cell-uuid>   (within-pack assignment)
  shard_key:         <consistent-hash>
  data_classes:      [SENSITIVE_PIPA_ART23, BEHAVIORAL_TENANT_PRODUCT, AUDIT, ...]
```

Properties:
- `jurisdiction_code` is set at tenant creation; tampering attempts (label-injection from untrusted code paths) detected by tenancy adapter's input validation.
- `pack` is derived from `jurisdiction_code` via the Cedar pack-routing fragment; redundant for routing convenience.
- Postgres + Citus per-pack retention policy keys on `(tenant_id, jurisdiction_code, data_class)` to apply correct retention windows.

## Retention by Jurisdiction × Data Class

Retention windows are the MAX of:
- Asset-class default (per `threat-model.md` §"Assets & Data Classification").
- Pack legal minimum (statutory retention).
- Tenant-contracted retention (DPA-declared).

| Pack | Data class | Minimum statutory | Default applied |
|---|---|---|---|
| pack-kr | `BEHAVIORAL_TENANT_PRODUCT` (tenant metadata) | KR commercial code: 5y after deletion | 7y default; aligned with statutory + DSR audit horizon |
| pack-kr | `SENSITIVE_PIPA_ART23` | PIPA Art. 28: bounded; erasure on request | 30d post-deletion for DSR audit; honour erasure-on-request |
| pack-kr | `AUDIT` (lifecycle + RLS install) | PIPA Enforcement Decree Art. 30: ≥ 1y | 3y default (KR-FSS sector ≥ 5y) |
| pack-kr | Proof-of-erasure certificates | regulator-disclosable; indefinite | indefinite |
| pack-eu | `PII_IDENTIFYING` (operator OIDC subject) | GDPR Art. 17: bounded; right-to-erasure within 30d | bounded; honour erasure |
| pack-eu | `BEHAVIORAL_TENANT_PRODUCT` | bounded by purpose | 7y default |
| pack-eu | `AUDIT` | bounded by purpose; documented in ROPA | 2y default; 6y for financial-services tenants under DORA |
| pack-us-healthcare | tenant metadata referencing PHI | depends on tenant's state Medical Records Retention law | use MAX(HIPAA 6y, state, tenant DPA) |
| pack-us-healthcare | `AUDIT` | HIPAA §164.316(b)(2): 6y | 6y |
| pack-jp | `PII_IDENTIFYING` | APPI: bounded; honour deletion request | bounded |
| pack-au | `PII_IDENTIFYING` | Privacy Act APP 11 + APP 12: bounded | bounded |
| pack-in | `PII_IDENTIFYING` | DPDPA 2023 §8(1)(g) (storage limitation) | bounded |
| pack-br | `PII_IDENTIFYING` | LGPD Art. 16 | bounded |
| pack-ae | `PII_IDENTIFYING` | UAE PDPL Art. 6 | bounded |
| pack-ksa | `PII_IDENTIFYING` | KSA PDPL Art. 6 | bounded; SAMA may extend for financial-services |
| (all packs) | `SECRET` | rotate per ISO 27001 A.5.17 cadence | 30d JWT signing keys; 90d Postgres replication password; 30d API tokens |

The CI lane `oya-governance-tenancy-retention-conformance` (NEW) validates Postgres + Citus retention configs against this table.

## DSR (Data Subject Request) Cascade

Tenancy is the **load-bearing authority** for DSR cascade. Per `dsr-cascade` BC:

1. Tenant raises DSR on behalf of their end-user (joint controllership per Art. 26) OR oyatie operator submits DSR on tenant behalf.
2. DSR runner submits DSR to tenancy's `dsr-cascade-rest`; DPO + ops-security 2-person rule.
3. `tenancy-dsr-cascade-worker` emits `TenantDeletionRequested` Workflow event consumed by every µservice with tenant-scoped data.
4. Each µservice executes its own DSR handler against its own data; emits `ErasureReceipt{microservice, tenant_id, data_classes_erased, residual_data_basis_if_any, signed_at}`.
5. `tenancy-dsr-cascade-worker` aggregates receipts into a Merkle tree; root signed as `ProofOfErasure{request_id, merkle_root, sealed_at, microservices_count}`.
6. Tenant + (if applicable) supervisory authority notified per per-pack SLA:
   - GDPR Art. 12(3): 30d default; tenancy honours within 30d for pack-eu.
   - KR PIPA Art. 36: 30d.
   - DPDPA §12: 30d.
   - LGPD Art. 19: 15d.
7. Audit-chain seals: every receipt + every proof-of-erasure Merkle root.

Limitations (documented in DPIA R-05 + R-12):
- Data older than statutory retention may already be deleted before DSR processed.
- A µservice without a registered DSR handler causes halt-and-escalate (per LEAN check `oya-governance-dsr-handler-conformance`).

## Per-Pack Overlay Sections

### pack-kr (KR PIPA + PIPC)

- **PIPA Art. 28 (storage period limitation)**: bounded; sensitive data minimal retention.
- **PIPA Art. 23-2 (sensitive data cross-border)**: forbidden by default; requires explicit consent for sensitive data crossing borders.
- **PIPC Notice 2020-7 (overseas-transfer notification)**: oyatie's pack-kr residency guarantee acknowledged in tenant DPA.
- **KR PIPA Art. 36 (right-to-deletion)**: DSR cascade within 30d.
- **KR-FSS sector guidance** (financial-services tenants): audit log retention ≥ 5y; encrypted at rest with KMS keys in KR-resident KMS.

### pack-eu (GDPR + EDPB + Schrems II + NIS2 + DORA)

- **GDPR Arts. 44–46 transfer mechanisms**: SCC-only; Adequacy decision via EU-list; Schrems-II-compliant supplementary technical measures (pseudonymisation + EU-resident KMS keys).
- **EDPB Recommendations 01/2020 (post-Schrems-II)**: supplementary measures documented in `legal/schrems-supplementary-measures.md`.
- **GDPR Art. 32 + 25**: pseudonymisation + EU-resident-key encryption.
- **NIS2 (2022/2555)**: 24h + 72h + 1mo reporting timelines for in-scope incidents.
- **DORA (2022/2554)**: financial-services tenants engage DORA ICT-risk register + testing program.

### pack-us-healthcare (HIPAA)

- **45 CFR §164.530(j) (Records retention)**: ≥ 6y.
- **HIPAA-eligible regions only**: OCI us-ashburn-1 + us-phoenix-1 per Oracle HIPAA-compliance attestation.
- **BAA-required**: tenant must sign BAA before pack-us-healthcare onboarding enabled.
- **Permitted Uses + Disclosures**: TPO; tenancy substrate falls under Operations.

### pack-jp (APPI)

- **APPI Art. 24 (cross-border)**: pack-jp data JP-resident.
- **APPI Art. 26-2 (breach notification)**: 72h to PPC.

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Each pack's overlay at `regional-packs/<pack>/tenancy-residency-overlay.md` carries the local data-residency law's citations. Pack-pinning + cross-pack-replication-forbidden invariants apply universally.

## Verification

- `cargo run -p oya-dev-cli -- gate validate tenancy-retention-conformance` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate tenancy-pack-routing-conformance` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate cross-pack-transfer-allowed-only-with-scc` — exit 0.
- Annual residency audit: confirm each tenant's data location matches its assigned pack.
- Quarterly chaos drill: induce a cross-pack write attempt; verify rejection + alerting.

## References

- ADR-0018 (Bominal): tenancy + RLS posture.
- ADR-0117: Cloud-native infrastructure (residency).
- ADR-0139: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout.
- `microservices/tenancy/threat-model.md` T-I-04 + T-T-04 + T-L-10.
- `microservices/tenancy/dpia.md` R-04 + R-11 + R-15 + §2.2.
- `microservices/tenancy/policy/rls-isolation.md`.
- `microservices/tenancy/multi-region.md`.
- `microservices/tenancy/legal/{transfer-register, schrems-supplementary-measures, baa-template, dpa-template, sub-processors, ropa}.md`.
- `regional-packs/<pack>/tenancy-residency-overlay.md` (per-pack).
- Oracle Cloud Infrastructure region documentation.
- GDPR Arts. 44–50.
- EDPB Recommendations 01/2020.
- KR PIPA Art. 23-2 + Art. 28 + Art. 36 + PIPC Notice 2020-7.
- HIPAA 45 CFR §164.530(j) + §164.316(b)(2).
- LGPD Art. 16 + Art. 19 + Art. 33.
- DPDPA 2023 §8(1)(g) + §12.
