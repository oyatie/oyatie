---
doc_class: PolicySpec
title: Data Residency Contract — cloud-iac
microservice: cloud-iac
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: privacy-governance + axis-cloud-iac
deciders: privacy-governance, ops-security, axis-cloud-iac, gtm-customer-success
related_adrs: [ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/cloud-iac/threat-model.md (T-T-04, T-I-01; cross-pack state replication threats)
  - microservices/cloud-iac/dpia.md (R-10)
  - microservices/cloud-iac/policy/iac-isolation.md
  - microservices/cloud-iac/multi-region.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Data Residency Contract (cloud-iac µservice)

## Purpose

Define which jurisdictions' Terraform/OpenTofu state files + iac-state-index records + apply audit events live in which pack region, the cross-pack replication policy, and the legal-transfer mechanisms that gate any exception. This document is the canonical residency artifact reviewed by EU DPAs (Arts. 44–50), the Korean PIPC (PIPA Art. 23-2 + Art. 28), HIPAA tenants' Covered Entity counsel (BAA), and equivalent supervisory authorities.

## Residency Model

### Default: pack-pinning

Every µservice's IaC state + apply-state index + apply audit retention is assigned a primary pack at first-apply. The state stays in that pack's region-pinned Postgres + state-bucket. Cross-pack movement is **forbidden by default**.

| Pack | Primary region(s) | Cloud-iac footprint | Activated? |
|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | kr-iac-state-index-pg-1, kr-iac-statestore-1 | YES (M01 launch) |
| pack-eu | OCI eu-frankfurt-1 + eu-amsterdam-1 | eu-iac-state-index-pg-{1,2}, eu-iac-statestore-{1,2} | Conditional (first EU tenant SCC) |
| pack-us | OCI us-ashburn-1 + us-phoenix-1 | us-iac-state-index-pg-{1,2}, us-iac-statestore-{1,2} | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | us-hc-iac-state-index-pg-1, us-hc-iac-statestore-1; isolated | Conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | jp-iac-state-index-pg-1, … | Conditional |
| pack-sg | OCI ap-singapore-1 | sg-iac-state-index-pg-1, … | Conditional |
| pack-au | OCI ap-sydney-1 + ap-melbourne-1 | au-iac-state-index-pg-{1,2}, … | Conditional |
| pack-in | OCI ap-hyderabad-1 + ap-mumbai-1 | in-iac-state-index-pg-{1,2}, … | Conditional (DPDPA 2023) |
| pack-br | OCI sa-saopaulo-1 + sa-vinhedo-1 | br-iac-state-index-pg-{1,2}, … | Conditional (LGPD) |
| pack-ae | OCI me-abudhabi-1 + me-dubai-1 | ae-iac-state-index-pg-{1,2}, … | Conditional |
| pack-ksa | OCI me-jeddah-1 + me-riyadh-1 | ksa-iac-state-index-pg-{1,2}, … | Conditional (KSA NCA cloud-residency) |

### Pack-assignment routing

Pack-pinning is inherited from the µservice's tenant assignment (per observability data-residency.md). cloud-iac honours the parent µservice's pack and stores state + apply-state index per pack.

```text
Tenant onboarding → gtm-customer-success → primary pack assignment
    ↓
Workload µservices are pack-pinned
    ↓
cloud-iac's iac-applier-worker reads µservice→pack mapping from OpenBao tenant-resolver
    ↓
OpenTofu state + iac-state-index entry written to that pack's regional store
    ↓
Cross-pack apply forbidden; integration test catches misroute (per FM-13 in observability failure-modes.md)
```

Routing encoded as Cedar policy at `policy/pack-routing.cedar` (Slice D).

## Cross-Pack Replication Policy

### Default: forbidden

Cross-pack replication of any of the following is forbidden by default:

- OpenTofu state files (per-pack object storage; SSE-KMS).
- iac-state-index Postgres records (per-pack primary + read-replica).
- Apply audit events (audit-chain replicated within pack only; cross-pack audit-chain federation is owned by `audit-chain` µservice's residency contract, not cloud-iac).
- ApplyExecuted / RenderCompleted / Rollback events (per-pack Mimir).
- ArgoCD Application records (per-pack ArgoCD instance).
- Sigstore Rekor entries (public transparency log; global by design but no tenant-identifying data).

### Exception: tenant-executed SCCs (GDPR)

Cross-border transfer of EU-resident state + audit data is permitted only when the tenant has executed an active SCC or equivalent transfer mechanism per GDPR Arts. 44–46. Requires:

1. Active SCC on file at `microservices/cloud-iac/legal/transfer-register.md` (Slice D).
2. Receiving-pack jurisdiction has adequate-decision or equivalent safeguard.
3. Transfer-purpose limited to specifically-named processing (e.g., "DR failover to pack-us").
4. Audit-chain emission at the moment of transfer.

### Exception: HIPAA BAA + DR failover

Covered Entity tenants in pack-us-healthcare may have DR pair us-ashburn-1 + us-phoenix-1; failover within the pair is intra-region from a HIPAA perspective. Cross-region (us → eu) failover NOT authorised without separate tenant agreement.

### Exception: BCDR exercise

For BCDR validation, controlled cross-region restore drills are permitted within DR-pair packs (pack-eu eu-frankfurt → eu-amsterdam; pack-us us-ashburn → us-phoenix). Cross-pack BCDR not authorised.

## iac-state-index Jurisdiction Labels

Every apply-state row in iac-state-index Postgres carries:

```sql
CREATE TABLE apply_state_index (
  microservice         text not null,
  pack                 text not null check (pack ~ '^pack-[a-z-]+$'),
  jurisdiction         text not null check (jurisdiction in ('kr','eu','us','us-hc','jp','sg','au','in','br','ae','ksa')),
  environment          text not null check (environment in ('dev','staging','production')),
  current_sha          text not null check (current_sha ~ '^[a-f0-9]{40}$'),
  prior_sha            text,
  applied_at           timestamptz not null,
  applied_by           text not null,
  signature            bytea not null,
  data_class           text not null default 'AUDIT',
  pack_pinned          boolean not null default true,
  PRIMARY KEY (microservice, pack, environment, applied_at)
);

-- Append-only enforcement
CREATE TRIGGER apply_state_index_append_only
BEFORE UPDATE OR DELETE ON apply_state_index
FOR EACH ROW EXECUTE FUNCTION refuse_mutation();
```

Properties:
- `pack` and `jurisdiction` redundantly encode residency for routing convenience.
- `pack_pinned=true` is the default; setting `false` requires Cedar policy entitlement (SCC-bound) + audit-chain seal.
- Mimir-side apply metrics carry `pack` and `jurisdiction` labels for retention enforcement.

## Retention by Jurisdiction × Data Class

Apply-state index + apply audit retention (cloud-iac is `AUDIT` data; retention windows are MAX of legal minimum + tenant-contracted):

| Pack | Data class | Minimum statutory | Default applied |
|---|---|---|---|
| pack-kr | `AUDIT` (apply ledger) | KR commercial code: 5y; KR-FSS: 5y for finance tenants | 5y |
| pack-kr | `BEHAVIORAL_TENANT_PRODUCT` (drift reports per tenant) | KR commercial code: 5y | 5y |
| pack-eu | `AUDIT` | bounded by purpose per Art. 5(1)(e); documented in ROPA | 2y default; 3y for finance |
| pack-us-healthcare | `AUDIT` | HIPAA §164.316(b)(2): 6y | 6y |
| pack-us-healthcare | `BEHAVIORAL_TENANT_PRODUCT` | varies by state | 6y |
| pack-jp | `AUDIT` | APPI: bounded; honour deletion request | 3y |
| pack-au | `AUDIT` | Privacy Act APP 11 | 3y |
| pack-in | `AUDIT` | DPDPA 2023 §8(1)(g) | 3y |
| pack-br | `AUDIT` | LGPD Art. 16 | 3y |
| (all packs) | `SECRET` (cluster kubeconfigs, ArgoCD tokens, state-encryption keys) | rotate per ISO 27001 A.5.17 | 24h kubeconfigs, 90d signing keys |

The CI lane `oya-governance-retention-conformance` validates apply-state index retention against this table.

## DSR Cascade

cloud-iac is downstream of the workload µservices; DSR (right-to-erasure) requests for tenant-bound data are handled by the workload µservices first. cloud-iac's apply-state index does NOT typically carry data-subject data directly. Where apply-state references hashed tenant identifiers, the DSR cascade per `microservices/observability/data-residency.md` applies (Mimir + Loki + Tempo deletion API), and the cloud-iac side updates the index to reflect.

Cloud-iac does not generally accept DSR for its own audit chain — audit-chain records are mandatory retention per Art. 6(1)(c) legal obligation; erasure is refused for the audit-chain itself per GDPR Art. 17(3)(b) (legal-obligation exception). This is disclosed in the tenant DPA.

## Per-Pack Overlay Sections

### pack-kr (KR PIPA + PIPC)

- **PIPA Art. 28 (storage period limitation)**: bounded; sensitive data minimal.
- **PIPA Art. 23-2 (sensitive data cross-border)**: forbidden by default; tenant DPA must authorise any cross-pack movement.
- **PIPC Notice 2020-7 (overseas-transfer notification)**: pack-kr residency guarantee acknowledged in tenant DPA.
- **KR-FSS sector guidance** (financial-services tenants): audit log retention ≥ 5y; encrypted at rest with KMS keys in KR-resident KMS.

### pack-eu (GDPR + EDPB + Schrems II)

- **GDPR Arts. 44–46**: SCC-only; Adequacy decision via EU list; Schrems-II supplementary measures.
- **EDPB Recommendations 01/2020 (post-Schrems-II)**: pseudonymisation + EU-resident-key encryption documented at `legal/schrems-supplementary-measures.md` (Slice D).
- **GDPR Art. 32 + 25**: pseudonymisation + EU-resident-key encryption "appropriate technical measures" for pack-eu state.

### pack-us-healthcare (HIPAA)

- **45 CFR §164.530(j)**: ≥ 6y from creation or last effective date.
- **HIPAA-eligible regions only**: OCI us-ashburn-1 + us-phoenix-1 per Oracle attestation.
- **BAA-required**: tenant must sign BAA before pack-us-healthcare apply enabled.
- **Permitted Uses + Disclosures**: TPO; cloud-iac operates under Operations scope.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/cloud-iac-residency-overlay.md`. Pack-pinning + cross-pack-replication-forbidden invariants apply universally.

## Verification

- cloud-ci/oya-ci governance gate `retention-conformance` is green in the branch-protected `oya-ci-required` context — exit 0.
- cloud-ci/oya-ci governance gate `pack-routing-conformance` is green in the branch-protected `oya-ci-required` context — exit 0.
- cloud-ci/oya-ci governance gate `cross-pack-state-replication-forbidden` is green in the branch-protected `oya-ci-required` context — exit 0.
- Annual residency audit: confirm each µservice's apply-state location matches its assigned pack.
- Quarterly chaos drill: induce a cross-pack state-write attempt; verify rejection + alerting.

## References

- ADR-0117: Cloud-native infrastructure (residency).
- ADR-0139: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout.
- `microservices/cloud-iac/threat-model.md` T-T-04 + T-I-01.
- `microservices/cloud-iac/dpia.md` R-10.
- `microservices/cloud-iac/policy/iac-isolation.md`.
- `microservices/cloud-iac/multi-region.md`.
- `microservices/observability/policy/data-residency.md` (parent µservice residency reference).
- `legal/{transfer-register, schrems-supplementary-measures, baa-template, dpa-template, sub-processors, ropa}.md` (Slice D).
- Oracle Cloud Infrastructure region documentation.
- GDPR Arts. 44–50.
- EDPB Recommendations 01/2020.
- KR PIPA Art. 23-2 + Art. 28 + PIPC Notice 2020-7.
- HIPAA 45 CFR §164.530(j).
- LGPD Art. 16 + Art. 33.
- DPDPA 2023 §8(1)(g).
