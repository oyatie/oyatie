---
doc_class: PolicyContract
template_id: TPL-POLICY
microservice: sites
policy_id: POLICY-data-residency
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-security + axis-sites
related_adrs: [ADR-0117, ADR-0140, ADR-SITES-0003, ADR-SITES-0004]
doc_status: published
---

# Data Residency Policy — sites µservice

## Purpose

Define where sites data may be stored, processed, and replicated; how
cross-border transfer is gated; and how each jurisdiction-pack's
residency invariant is enforced.

## Residency Invariants

### Invariant DR-01 — Pack-pinned primary storage

> Every tenant's site/page/cms-collection rows live in exactly one
> `pack-<jurisdiction>` Postgres cluster. Per-tenant Meilisearch
> indexes live in the same pack. Per-tenant S3 published-artifact
> prefixes live in the same pack. The pack is determined at tenant
> onboarding and pinned in the `tenant_registry` (owned by `tenancy`
> µservice).

| Pack | Region | Substrate |
|---|---|---|
| pack-kr | OCI ap-seoul-1 | Postgres + Redis + Meilisearch + S3 KR-resident |
| pack-eu | OCI eu-frankfurt-1 | Postgres + Redis + Meilisearch + S3 EU-resident |
| pack-us | OCI us-ashburn-1 + us-phoenix-1 | Postgres + Redis + Meilisearch + S3 US-resident |
| pack-us-healthcare | OCI us-ashburn-1 (BAA-eligible) | Postgres + Redis + Meilisearch + S3 US-resident, HIPAA-eligible |
| pack-jp | OCI ap-tokyo-1 | Postgres + Redis + Meilisearch + S3 JP-resident |
| pack-sg | OCI ap-singapore-1 | Postgres + Redis + Meilisearch + S3 SG-resident |
| pack-au | OCI ap-sydney-1 | Postgres + Redis + Meilisearch + S3 AU-resident |
| pack-in | OCI ap-mumbai-1 | Postgres + Redis + Meilisearch + S3 IN-resident |
| pack-br | OCI sa-saopaulo-1 | Postgres + Redis + Meilisearch + S3 BR-resident |
| pack-ae | OCI me-jeddah-1 / me-dubai-1 | Postgres + Redis + Meilisearch + S3 ME-resident |
| pack-ksa | OCI me-jeddah-1 | Postgres + Redis + Meilisearch + S3 KSA-resident |

### Invariant DR-02 — No default cross-pack replication

> Postgres replication factor is 3 within a single region's cluster
> (same pack). Cross-pack replication is FORBIDDEN by default.
> Meilisearch + S3 cross-pack replication is FORBIDDEN by default.
> Logical replication slots, dump/restore, or any other cross-pack path
> is refused at infrastructure layer (Postgres `pg_hba.conf` + network
> policy) and at LEAN-check layer
> (`oya-check-cross-pack-replication-prohibition`).

### Invariant DR-03 — Cross-border transfer gated by SCC

> Cross-pack data flow is permitted ONLY when:
> 1. The tenant has executed Standard Contractual Clauses (SCCs) per
>    GDPR Arts. 44–46 (or equivalent pack-local provision: KR PIPA
>    Art. 23-2; APPI Art. 21; LGPD Art. 33; UAE PDPL Art. 22; KSA
>    PDPL Art. 29).
> 2. The transfer is recorded in `microservices/sites/legal/transfer-register.md`.
> 3. The transfer scope is limited to published-public content (which
>    is, by tenant choice, already public).
> 4. The tenant's DPA template carries the SCC clause.

### Invariant DR-04 — CDN edge anycast vs origin residency

> CDN edges may serve published-public pages from any geographic
> location (anycast); this is by tenant choice (publication is
> public-by-design). HOWEVER:
> - The origin pages live ONLY in the tenant's pack.
> - Editor authoring is confined to the tenant's pack.
> - Non-public (intranet/private/draft) pages NEVER traverse CDN
>   edges outside the pack.

### Invariant DR-05 — Custom-domain TLS cert residency

> ACME private keys are stored only in the tenant's pack's OpenBao
> instance. Cert renewal is performed by the pack's
> domain-binding-worker. Cross-pack cert sharing is FORBIDDEN.

### Invariant DR-06 — Backup + cold storage residency

> Backup snapshots + cold-storage exports remain in the same pack as
> the source. Cross-region backup replication is allowed ONLY for
> disaster-recovery within the same jurisdiction family (e.g.,
> us-ashburn-1 → us-phoenix-1 both in pack-us).

### Invariant DR-07 — Audit-chain seal storage

> Audit-chain seals emitted by sites are persisted by the `audit-chain`
> µservice; that µservice's residency policy governs. Sites inherits
> the constraint that audit-chain seal records for pack-A pages remain
> in pack-A audit-chain cluster.

### Invariant DR-08 — AI-page-build LLM inference residency

> T2 AI-page-build prompts are sent to LLM providers. The prompt
> ciphertext (tenant-DEK-wrapped) may transit cross-region to the
> provider — BUT:
> - The provider receives ciphertext only via foundry-runtime
>   private-inference channel.
> - For pack-eu tenants, only EU-resident LLM providers may be
>   selected.
> - For pack-us-healthcare tenants, only BAA-on-file LLM providers may
>   be selected.
> - Tenant must consent at T2 enablement; refusal is honoured.

## Per-Pack Detail

### pack-kr (KR PIPA + ISMS-P + 전자문서법)

- **PIPA Art. 17 (cross-border transfer)**: default cross-pack = forbidden.
- **PIPA Art. 23-2 (sensitive data cross-border)**: requires explicit consent at tenant-of-tenant level. Sensitive-flagged content never cross pack-kr.
- **PIPA Art. 28-2 (data destruction)**: retention upper bounds enforced; non-essential data destroyed within statutory minimum.
- **PIPC enforcement**: tenant DPA includes pack-kr addendum specifying KR-resident retention.
- **Korean localisation**: pack-kr ships Korean default locale `ko-KR`; calendar in Lunar overlay; KR-PIPA-flagged data class.

### pack-eu (GDPR + EDPB + Schrems II + EU AI Act + EU DSA + ePrivacy + eIDAS + NIS2)

- **GDPR Arts. 44–50 (transfers)**: SCC-only for cross-pack.
- **Schrems II**: transfer impact assessment (TIA) required when cross-pack involves non-adequate jurisdictions; TIA template at `legal/tia-template.md`.
- **EDPB Recommendations 01/2020**: supplementary measures (encryption-in-transit + tenant-DEK at rest) implemented.
- **EU AI Act**: T2 AI-page-build EU-LLM-only routing per Invariant DR-08.
- **EU DSA Art. 14**: publish-refusal transparency.
- **ePrivacy Art. 5(3)**: consent banner required for non-strictly-necessary cookies.
- **eIDAS Art. 26**: audit-chain Ed25519 satisfies AdES for signed-Sites.

### pack-us-healthcare (HIPAA + state)

- **45 CFR §164.502(e) BAA**: BAA-bound tenant data stays in BAA-eligible region.
- **HIPAA breach notification ≤ 60 days**: integrated.
- **State-level**: CCPA / CMIA / NY SHIELD overlays per `compliance.md`.
- **ADA Title III + Section 508**: WCAG 2.2 AA refuse-publish at < 100% for patient-portal sites.

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

- **§16 (cross-border transfer)**: as of 2026-05, DPDPA cross-border list pending; default residency in-IN until clarified.
- **§9 (children's data)**: parental consent verification inherited from tenant.

### pack-br (LGPD)

- **Arts. 33–36 (transfers)**: ANPD-approved SCCs.
- **Art. 38 RIPD**: this DPIA + threat-model satisfies.

### pack-ae (UAE PDPL) / pack-ksa (KSA PDPL)

- **UAE PDPL Art. 22 / KSA PDPL Art. 29 (cross-border)**: SCC-equivalent + DPA-approved.
- **Hijri calendar overlay**: tenant content + scheduling honour Hijri dates via ICU4X.

## Enforcement Layers

| Layer | Mechanism | Refusal at |
|---|---|---|
| Tenant onboarding | tenancy µservice assigns + pins pack | Onboarding gate |
| Network | Postgres `pg_hba.conf` + Kubernetes NetworkPolicy refuses cross-pack ingress | Network |
| Application | Pack tag in OIDC + per-tenant API-key binding; ingress routes by tag | API request |
| LEAN CI | `oya-check-cross-pack-replication-prohibition`, `oya-check-pack-pinning-coverage` | PR time |
| Audit | every Workflow event carries `pack_tag`; cross-pack flows emit explicit transfer record | Per-event |
| CDN | per-pack edge selection; geo-fenced for non-public content | Edge selection |

## Verification

| Check | Cadence | Owner |
|---|---|---|
| LEAN: cross-pack replication prohibition | per-PR | axis-sites |
| LEAN: pack-pinning coverage | per-PR | axis-sites |
| LEAN: CDN edge per-pack selection | per-PR | axis-sites |
| Pen-test: cross-pack routing bypass | Annually | ops-security |
| Backup-residency audit | Quarterly | ops-sre-reliability |
| SCC compliance: transfer register review | Quarterly | council-privacy |
| T2 LLM provider per-pack selection audit | Quarterly | council-privacy |

## References

- ADR-0117: data residency.
- ADR-0140: Cedar policy.
- ADR-SITES-0003: CDN substrate.
- ADR-SITES-0004: ACME + custom domain.
- `multi-region.md`, `compliance.md`, `legal/transfer-register.md`,
  `legal/dpa-template.md`, `legal/tia-template.md`.
- GDPR Arts. 44–50; EDPB Recommendations 01/2020.
- KR PIPA Arts. 17, 23-2, 28-2.
- HIPAA 45 CFR §164.502(e); 45 CFR Part 164 Subpart D.
- APPI Arts. 17, 21, 27.
- PDPA, MAS Notice 644, APP, APRA-CPS 234, DPDPA, LGPD, UAE PDPL, KSA PDPL.
- EU AI Act Regulation (EU) 2024/1689.
- EU DSA Regulation (EU) 2022/2065.
- ePrivacy Directive 2002/58/EC.
- eIDAS Regulation (EU) 910/2014.
- NIS2 Directive (EU) 2022/2555.
- ADA Title III + Section 508.
