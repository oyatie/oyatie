---
doc_class: PolicySpec
title: Data Residency Contract
microservice: ontology
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-ontology
deciders: council-privacy, ops-security, axis-ontology, gtm-customer-success
related_adrs: [ADR-0117, ADR-0130, ADR-0131, ADR-0140]
related_artifacts:
  - microservices/ontology/threat-model.md (T-I-01, T-T-04, T-L-09)
  - microservices/ontology/dpia.md (R-11)
  - microservices/ontology/policy/type-isolation.md
  - microservices/ontology/multi-region.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Data Residency Contract (ontology µservice)

## Purpose

Define which jurisdictions' tenant Object Type instances + Link Type instances + Action receipts + audit-chain seals live in which Postgres + ClickHouse cluster; the cross-pack replication policy; the legal-transfer mechanisms that gate exceptions. Authoritative artifact reviewed by EU DPAs (GDPR Arts. 44–50), KR PIPC (PIPA Art. 28 + Art. 23-2), HIPAA Covered Entity counsel (BAA), and equivalent supervisory authorities in every active pack.

## Residency Model

### Default: pack-pinning

Every tenant is assigned a primary pack at onboarding. Telemetry-equivalent for Ontology is the typed entity universe — Object Type instances + Link Type instances + Action receipts + audit-chain seals all stored in the pack's region-pinned cluster. Cross-pack movement is **forbidden by default**.

| Pack | Primary region(s) | Cluster footprint | Activated? |
|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | kr-pg-citus-1, kr-clickhouse-1, kr-valkey-1, kr-kafka-1 | YES (M02b launch tenant) |
| pack-eu | OCI eu-frankfurt-1 + eu-amsterdam-1 (DR pair) | eu-pg-citus-{1,2}, eu-clickhouse-{1,2}, eu-valkey-{1,2}, eu-kafka-{1,2} | Conditional (activated when first EU tenant signs SCC) |
| pack-us | OCI us-ashburn-1 + us-phoenix-1 (DR pair) | us-pg-citus-{1,2}, us-clickhouse-{1,2}, us-valkey-{1,2}, us-kafka-{1,2} | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) + us-phoenix-1 | us-hc-pg-citus-{1,2}, us-hc-clickhouse-{1,2}; isolated from non-HC pack-us | Conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | jp-pg-citus-1, jp-clickhouse-1, ... | Conditional |
| pack-sg | OCI ap-singapore-1 | sg-pg-citus-1, ... | Conditional |
| pack-au | OCI ap-sydney-1 + ap-melbourne-1 | au-pg-citus-{1,2}, ... | Conditional |
| pack-in | OCI ap-hyderabad-1 + ap-mumbai-1 | in-pg-citus-{1,2}, ... | Conditional (DPDPA 2023) |
| pack-br | OCI sa-saopaulo-1 + sa-vinhedo-1 | br-pg-citus-{1,2}, ... | Conditional (LGPD) |
| pack-ae | OCI me-abudhabi-1 + me-dubai-1 | ae-pg-citus-{1,2}, ... | Conditional |
| pack-ksa | OCI me-jeddah-1 + me-riyadh-1 | ksa-pg-citus-{1,2}, ... | Conditional (KSA NCA cloud-residency) |

"Activated?" updated at first-tenant onboarding per pack; activation triggers re-review of this document + per-pack threat-model overlay + DPIA overlay.

### Pack-assignment routing

```text
Tenant onboarding
    ↓
gtm-customer-success: collects tenant's HQ jurisdiction + regulated-data declarations
    ↓
Pack-router (Cedar policy in cloud-iac + ontology):
    - HQ jurisdiction → primary pack
    - Regulated-data flag (PHI, KR-FSS, EU-resident, etc.) → may force secondary pack
    - Conflict: ops-legal escalation
    ↓
OpenBao assigns tenant → pack
    ↓
SDK at tenant µservice pinned to pack endpoint
    ↓
Object Type / Link Type / Action / Function reads + writes flow to pack cluster; never cross-pack
```

Routing encoded in Cedar at `microservices/ontology/policy/pack-routing.cedar` (Slice D follow-up).

### Per-tenant tenant_scope influences capacity, not residency

`tenant_scope` (trial / production / sandbox / internal) affects capacity allocation within the pack but never moves data across packs.

## Cross-Pack Replication Policy

### Default: forbidden

Cross-pack replication of any tenant data is forbidden by default:

- Postgres Object Type tables: replicate within-pack only (RF=3 across AZs).
- Citus shards: replicate within-pack only.
- ClickHouse history-mirror: replicate within-pack only.
- Kafka topics (outbox): replicate within-pack only.
- Audit-chain seals: replicate within-pack only (each pack has its own audit-chain seal authority).
- Schema-registry (Valkey + Postgres): configuration is global; per-pack Helm values; instance state per-pack.
- Cedar policy fragments + Object Type schema definitions: git-versioned; global (these are configuration, not tenant data).

### Exception: tenant-executed SCCs (GDPR transfer mechanism)

Cross-border transfer of EU-resident data permitted only when the tenant has executed an active SCC per GDPR Arts. 44–46:

1. Active SCC on file at `microservices/ontology/legal/transfer-register.md` (Slice D).
2. Receiving-pack jurisdiction has adequate-decision (GDPR Art. 45) or equivalent safeguard.
3. Transfer-purpose limited to specifically-named processing.
4. Audit-chain-emitted SCC-acknowledgement at moment of transfer.
5. Cedar `CrossPackTransferGrant` in caller's claims (issued via 2-person rule).

### Exception: HIPAA BAA + DR failover

Covered Entity tenants in pack-us-healthcare may have DR pair us-ashburn-1 + us-phoenix-1; failover within the pair is intra-region from HIPAA perspective (both HIPAA-eligible). Cross-region (us → eu) failover is NOT authorised without separate tenant agreement.

### Exception: BCDR exercise (controlled, scheduled)

For BCDR validation: controlled intra-pack cross-region restore drills permitted (pack-eu eu-frankfurt → eu-amsterdam; pack-us us-ashburn → us-phoenix). Cross-pack BCDR is not authorised.

## Jurisdiction Tagging on Every Row

In addition to `tenant_id`, every Object Type row carries:

```sql
ALTER TABLE <object_type> ADD COLUMN jurisdiction TEXT NOT NULL CHECK (
  jurisdiction IN ('kr', 'eu', 'us', 'us-hc', 'jp', 'sg', 'au', 'in', 'br', 'ae', 'ksa')
);
ALTER TABLE <object_type> ADD COLUMN pack TEXT NOT NULL;
ALTER TABLE <object_type> ADD COLUMN data_class TEXT NOT NULL;
ALTER TABLE <object_type> ADD COLUMN property_tier TEXT NOT NULL;
```

Properties:
- `jurisdiction` set by SDK on every write based on tenant's pack assignment.
- Tampering attempts (column override from untrusted code paths) detected by adapter enforcement: the writer's bound tenant_id determines jurisdiction; client-set override refused.
- Per-tenant retention policy keys on `(tenant_id, jurisdiction, data_class)`.
- `pack` redundant with `jurisdiction` for routing convenience.

## Retention by Jurisdiction × Data Class

Retention windows = MAX of:
- Asset-class default (per `threat-model.md` §"Assets & Data Classification").
- Pack legal minimum (statutory retention).
- Tenant-contracted retention (DPA-declared).

| Pack | Data class | Minimum statutory | Default applied |
|---|---|---|---|
| pack-kr | `BEHAVIORAL_TENANT_PRODUCT` | KR commercial code: 5 years | 5y |
| pack-kr | `SENSITIVE_PIPA_ART23` | PIPA Art. 28: bounded; erasure on request | 1y default; honour erasure |
| pack-kr | `AUDIT` (Action receipts + audit chain) | PIPA Enforcement Decree Art. 30: ≥ 1y | 3y for production-tier (KR-FSS guidance) |
| pack-eu | `PII_IDENTIFYING` | GDPR Art. 17: bounded; right-to-erasure 30d | bounded; honour erasure |
| pack-eu | `AUDIT` | bounded by purpose; documented in ROPA | 2y default |
| pack-us-healthcare | `PHI` | HIPAA + state Medical Records Retention | MAX(HIPAA 6y, state, tenant DPA) |
| pack-us-healthcare | `AUDIT` | HIPAA §164.316(b)(2): 6y | 6y |
| pack-jp | `PII_IDENTIFYING` | APPI: bounded; honour deletion request | bounded |
| pack-au | `PII_IDENTIFYING` | Privacy Act APP 11 + APP 12 | bounded |
| pack-in | `PII_IDENTIFYING` | DPDPA 2023 §8(1)(g) | bounded |
| pack-br | `PII_IDENTIFYING` | LGPD Art. 16 | bounded |
| (all packs) | `SECRET` | rotate per ISO 27001 A.5.17 | 30d API keys, 90d signing keys |

The CI lane `oya-foundry-fitness-ontology-retention-conformance` validates Postgres + ClickHouse retention configs against this table.

## DSR (Data Subject Request) Cascade

Right-to-erasure (GDPR Art. 17 / PIPA Art. 36 / DPDPA §12 / LGPD Art. 18(V)-(VI)) honoured via `oya-ontology-dsr-cascade-runner`:

1. Tenant raises DSR on behalf of end-user.
2. DSR runner enumerates every Object Type table for the tenant; identifies rows whose `subject_hash` matches.
3. Tombstones rows (soft-delete + 30-day grace; hard-delete after grace).
4. Per-Object-Type completeness manifest persisted; subject hash recorded in audit chain.
5. Audit-chain seal: `dsr_executed{tenant, subject_hash, removed_object_type_count, residual_object_types[], executed_at}`.
6. Tenant notified within 30d SLA per GDPR; some packs shorter (KR 30d, BR 15d, EU 30d); strictest SLA applies.

Limitations (documented in DPIA R-08):
- Data older than retention window may already be deleted before DSR processed.
- Audit-chain seals immutable (subject hash + tier removed; seal record itself remains for non-repudiation per ADR-0028).
- Link Types where subject is referenced indirectly require multi-hop scan; cap at 5 hops.

## Per-Pack Overlay Sections

### pack-kr (KR PIPA + PIPC)

- **PIPA Art. 28 (storage period)**: bounded; sensitive data minimal retention.
- **PIPA Art. 23-2 (sensitive cross-border)**: forbidden by default; requires consent of data subject (sensitive tenant data covered by tenant DPA).
- **PIPC Notice 2020-7 (overseas-transfer notification)**: oyatie's pack-kr residency guarantee acknowledged in tenant DPA.
- **KR-FSS sector** (financial-services tenants): audit log retention ≥ 5y; encrypted at rest with KMS keys in KR-resident KMS.

### pack-eu (GDPR + EDPB + Schrems II)

- **GDPR Arts. 44–46 transfer**: SCC-only; Adequacy decision via EU-list; Schrems-II-compliant supplementary measures (pseudonymisation + EU-controlled KMS).
- **EDPB Recommendations 01/2020**: supplementary measures at `microservices/ontology/legal/schrems-supplementary-measures.md` (Slice D).
- **GDPR Art. 32 + 25**: pseudonymisation + EU-resident-key encryption are appropriate TOMs.

### pack-us-healthcare (HIPAA)

- **45 CFR §164.530(j) (Records retention)**: ≥ 6y from creation or last effective date.
- **HIPAA-eligible regions only**: OCI us-ashburn-1 + us-phoenix-1 per Oracle HIPAA-compliance attestation.
- **BAA-required**: tenant must sign BAA before pack-us-healthcare ingest enabled.
- **Permitted Uses + Disclosures**: TPO; operations scope covers Ontology entity management.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/data-residency-overlay.md`. Pack-pinning + cross-pack-replication-forbidden invariants apply universally.

## Verification

- `cargo run -p oya-dev-cli -- gate validate retention-conformance --microservice ontology` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate pack-routing-conformance --microservice ontology` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate cross-pack-transfer-allowed-only-with-scc --microservice ontology` — exit 0.
- Annual residency audit: confirm each tenant's data location matches assigned pack.
- Quarterly chaos drill: induce a cross-pack write attempt; verify rejection + alerting.

## References

- ADR-0117: Cloud-native infrastructure (residency).
- ADR-0130: Agentic SLO-gated promotion (release pointers; per-pack pinning).
- ADR-0131: Per-microservice flat layout.
- `microservices/ontology/threat-model.md` T-I-01 + T-T-04 + T-L-09.
- `microservices/ontology/dpia.md` R-08 + R-11 + R-13 + §2.2.
- `microservices/ontology/policy/type-isolation.md`.
- `microservices/ontology/multi-region.md`.
- `microservices/ontology/legal/{transfer-register, schrems-supplementary-measures, baa-template, dpa-template, sub-processors, ropa}.md` (Slice D).
- `regional-packs/<pack>/data-residency-overlay.md` (per-pack).
- Oracle Cloud Infrastructure region docs.
- GDPR Arts. 44–50; EDPB Recommendations 01/2020.
- KR PIPA Art. 23-2 + Art. 28 + PIPC Notice 2020-7.
- HIPAA 45 CFR §164.530(j); LGPD Art. 16; DPDPA 2023 §8(1)(g).
