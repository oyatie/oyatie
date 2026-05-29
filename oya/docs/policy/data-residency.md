---
doc_class: PolicyContract
template_id: TPL-POLICY
microservice: docs
policy_id: POLICY-data-residency
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-security + axis-docs
related_adrs: [ADR-0117, ADR-0140 (retired per ADR-0145)]
doc_status: published
---

# Data Residency Policy — docs µservice

## Purpose

Define where docs data may be stored, processed, and replicated; how cross-border transfer is gated; and how each jurisdiction-pack's residency invariant is enforced.

## Residency Invariants

### Invariant DR-01 — Pack-pinned primary storage

> Every tenant's document-metadata Postgres rows + content blobs in S3 + Valkey CRDT spool live in exactly one `pack-<jurisdiction>` cluster. The pack is determined at tenant onboarding and pinned in the `tenant_registry` (owned by `tenancy` µservice).

| Pack | Region | Substrate |
|---|---|---|
| pack-kr | OCI ap-seoul-1 | Postgres + S3 + Valkey cluster KR-resident |
| pack-eu | OCI eu-frankfurt-1 | Postgres + S3 + Valkey cluster EU-resident |
| pack-us | OCI us-ashburn-1 + us-phoenix-1 | Postgres + S3 + Valkey cluster US-resident |
| pack-us-healthcare | OCI us-ashburn-1 (BAA-eligible) | Postgres + S3 + Valkey cluster US-resident, HIPAA-compliant |
| pack-jp | OCI ap-tokyo-1 | Postgres + S3 + Valkey cluster JP-resident |
| pack-sg | OCI ap-singapore-1 | Postgres + S3 + Valkey cluster SG-resident |
| pack-au | OCI ap-sydney-1 | Postgres + S3 + Valkey cluster AU-resident |
| pack-in | OCI ap-mumbai-1 | Postgres + S3 + Valkey cluster IN-resident |
| pack-br | OCI sa-saopaulo-1 | Postgres + S3 + Valkey cluster BR-resident |
| pack-ae | OCI me-dubai-1 | Postgres + S3 + Valkey cluster AE-resident |
| pack-ksa | OCI me-jeddah-1 | Postgres + S3 + Valkey cluster KSA-resident |

### Invariant DR-02 — No default cross-pack replication

> Postgres replication factor is 3 within a single region's cluster (same pack). S3 cross-region replication enabled only within-jurisdiction. Cross-pack replication is FORBIDDEN by default. Logical replication slots, dump/restore, or any other cross-pack path is refused at infrastructure layer (Postgres `pg_hba.conf` + S3 bucket policy + network policy) and at LEAN-check layer (`oya-check-cross-pack-replication-prohibition`).

### Invariant DR-03 — Cross-border transfer gated by SCC

> Cross-pack data flow (e.g., a KR tenant embedding a workflow-studio canvas owned by EU tenant) is permitted ONLY when:
> 1. The tenant has executed Standard Contractual Clauses (SCCs) per GDPR Arts. 44–46 (or equivalent pack-local provision).
> 2. The transfer is recorded in `microservices/docs/legal/transfer-register.md`.
> 3. The transfer is embed-snapshot-bound (cross-tenant embed projection only; never raw doc content; per Invariant DR-04).
> 4. The tenant's DPA template carries the SCC clause.

### Invariant DR-04 — Cross-µservice embed cross-pack projection

> When a doc in pack-A embeds a workflow-studio canvas / sheets cell / slides deck owned by pack-B tenant, the embed-resolver creates:
> - In pack-A: only an embed snapshot record (Embed { source_ref, snapshot_blob_ref, refreshed_at }).
> - In pack-B: source-side ACL is evaluated and a snapshot is returned via mTLS — raw source data never crosses the pack boundary except as the bounded snapshot.
>
> The snapshot is treated as an internal resource of pack-A; subsequent reads do not re-cross pack boundary except on refresh.

### Invariant DR-05 — REST + WebSocket ingress routes by tenant pack

> The REST + WebSocket ingress (`document-store-rest`, `collab-crdt-worker`) routes by per-tenant pack tag derived from OIDC issuer + per-tenant API-key binding. Misroute is refused (HTTP 403 + audit emission).

### Invariant DR-06 — Backup + cold storage residency

> Backup snapshots + cold-storage exports remain in the same pack as the source. S3 Object Lock for legal-hold blobs is pack-resident. Cross-region backup replication is allowed ONLY for disaster-recovery within the same jurisdiction family (e.g., us-ashburn-1 → us-phoenix-1 both in pack-us).

### Invariant DR-07 — Audit-chain seal storage

> Audit-chain seals emitted by docs are persisted by the `audit-chain` µservice; that µservice's residency policy governs. Docs inherits the constraint that audit-chain seal records for pack-A docs remain in pack-A audit-chain cluster.

### Invariant DR-08 — Export pipeline residency

> The gVisor-sandboxed export workers (Pandoc / WeasyPrint / Chromium) per ADR-DOCS-0003 are pack-resident. Per-export tmpfs is pack-resident. Output blobs are stored in pack-resident S3.

### Invariant DR-09 — LLM inference (AI writing-assist) residency

> Foundry-runtime LLM calls for T0/T1/T2 capabilities use the pack-resident model deployment. Cross-pack model invocation FORBIDDEN at the foundry-runtime layer. Tenant-DEK-wrapped prompts ensure cross-tenant training-data isolation.

## Per-Pack Detail

### pack-kr (KR PIPA + ISMS-P + 전자문서법)

- **PIPA Art. 17 (cross-border transfer)**: forbids cross-border transfer without explicit consent. Default cross-pack = forbidden.
- **PIPA Art. 23-2 (sensitive data cross-border)**: requires explicit consent at tenant-of-tenant level. Sensitive-flagged docs never cross pack-kr.
- **PIPA Art. 28-2 (data destruction)**: retention upper bounds enforced; non-essential data destroyed within statutory minimum.
- **전자문서법 Arts. 5/6/7**: audit-chain Ed25519 + Merkle satisfies electronic-document integrity + storage + e-signature equivalence.
- **PIPC enforcement**: tenant DPA includes pack-kr addendum specifying KR-resident retention.

### pack-eu (GDPR + EDPB + Schrems II + EU AI Act + eIDAS PAdES)

- **GDPR Arts. 44–50 (transfers)**: SCC-only for cross-pack.
- **Schrems II**: transfer impact assessment (TIA) required when cross-pack involves non-adequate jurisdictions; TIA template at `legal/tia-template.md`.
- **EDPB Recommendations 01/2020**: supplementary measures (encryption-in-transit + tenant-DEK encryption-at-rest) implemented.
- **NIS2**: when oyatie crosses threshold, incident-reporting timelines apply.
- **EU AI Act Annex III §3**: T1/T2 HR-context REFUSED at Cedar layer per ADR-DOCS-0005.
- **eIDAS PAdES**: PDF export with B-LT level signature for legal-evidence tenants.

### pack-us-healthcare (HIPAA + state)

- **45 CFR §164.502(e) BAA**: BAA-bound tenant data stays in BAA-eligible region.
- **HIPAA breach notification ≤ 60 days**: integrated.
- **State-level**: CCPA / CMIA / NY SHIELD overlays per `compliance.md`.
- **OPSWAT MetaDefender**: attachment scanner pack-resident; pack-us-healthcare HIPAA-grade alternative to default ClamAV.

### pack-jp (APPI)

- **APPI Art. 17 (purpose)**: declared at onboarding.
- **APPI Art. 21 (cross-border)**: pack-jp JP-resident.
- **APPI Art. 27 (cross-border consent)**: explicit at onboarding.

### pack-sg (PDPA + MAS Notice 644)

- **PDPA Part IV Retention Limitation**: retention bounded per asset table.
- **PDPA Part VI Transfer Limitation**: SCC-equivalent.
- **MAS Notice 644**: for financial-services tenants.

### pack-au (Privacy Act 1988 APP)

- **APP 8 (cross-border)**: tenant-DPA includes APP 8 clause.
- **APP 11 (security)**: encryption + audit-chain satisfies.
- **APRA-CPS 234**: for financial-services tenants.

### pack-in (DPDPA 2023)

- **§16 (cross-border transfer)**: as of 2026-05, DPDPA cross-border whitelist pending; default residency in-IN until clarified.
- **§9 (children's data)**: parental consent verification inherited from tenant.

### pack-br (LGPD)

- **Arts. 33–36 (transfers)**: ANPD-approved SCCs.
- **Art. 38 RIPD**: this DPIA + threat-model satisfies.

### pack-ae (UAE PDPL) / pack-ksa (KSA PDPL)

- **UAE PDPL Art. 22 / KSA PDPL Art. 29 (cross-border)**: SCC-equivalent + DPA-approved.
- **Hijri calendar**: ICU4X dual-calendar rendering in document timestamps + audit-chain metadata.
- **KSA Sharia retention**: per-tenant retention extension supported; refusal of premature deletion logged in audit-chain.

## Enforcement Layers

| Layer | Mechanism | Refusal at |
|---|---|---|
| Tenant onboarding | tenancy µservice assigns + pins pack | Onboarding gate |
| Network | Postgres `pg_hba.conf` + S3 bucket policy + Kubernetes NetworkPolicy refuses cross-pack ingress | Network |
| Application | Pack tag in OIDC + per-tenant API-key binding; ingress routes by tag | API request |
| Embed-resolver | source-side ACL evaluated cross-pack; snapshot-only returned | Cross-µservice mTLS |
| LEAN CI | `oya-check-cross-pack-replication-prohibition`, `oya-check-pack-pinning-coverage` | PR time |
| Audit | every Workflow event carries `pack_tag`; cross-pack flows emit explicit transfer record | Per-event |

## Verification

| Check | Cadence | Owner |
|---|---|---|
| LEAN: cross-pack replication prohibition | per-PR | axis-docs |
| LEAN: pack-pinning coverage | per-PR | axis-docs |
| LEAN: embed-resolver cross-pack snapshot-only | per-PR | axis-docs |
| Pen-test: cross-pack routing bypass | Annually | ops-security |
| Backup-residency audit | Quarterly | ops-sre-reliability |
| SCC compliance: transfer register review | Quarterly | council-privacy |
| LLM inference pack-residency audit | Quarterly | foundry-runtime + council-privacy |

## References

- ADR-0117: data residency.
- ADR-0140: Cedar policy.
- ADR-DOCS-0003 (export pipeline residency).
- ADR-DOCS-0005 (LLM inference residency).
- `multi-region.md`, `compliance.md`, `legal/transfer-register.md`, `legal/dpa-template.md`, `legal/tia-template.md`.
- GDPR Arts. 44–50; EDPB Recommendations 01/2020.
- KR PIPA Arts. 17, 23-2, 28-2.
- HIPAA 45 CFR §164.502(e); 45 CFR Part 164 Subpart D.
- APPI Arts. 17, 21, 27.
- PDPA, MAS Notice 644, APP, APRA-CPS 234, DPDPA, LGPD, UAE PDPL, KSA PDPL.
- EU AI Act Regulation (EU) 2024/1689 Annex III §3.
- eIDAS Regulation 910/2014 (PAdES).
