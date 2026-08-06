---
doc_class: Pack-README
pack_id: CN-PIPL-2021
version: "1.0.0"
binding_adr: ADR-0251
regulation: Personal Information Protection Law of the People's Republic of China (PIPL), effective 2021-11-01
jurisdiction: CN
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0251
  - ADR-0242
  - ADR-0243
  - ADR-0248
  - ADR-0255
companion_docs:
  - packs/cn-pipl/manifest.json
  - packs/cn-pipl/breach-notification-workflow.yaml
  - packs/cn-pipl/dpia-template.md
  - packs/cn-pipl/regulator-references.yaml
  - docs/decisions/ADR-0708-platform-foundations-live-apex.md
planned_enforcement_ref: oya-governance-doc-rigor
---

# CN-PIPL-2021 Compliance Pack

## §A — Overview and Scope

### A.1 What is this pack?

CN-PIPL-2021 is the oyatie compliance pack for the **Personal Information Protection Law of the
People's Republic of China** (个人信息保护法, PIPL), effective 2021-11-01. It is one of the
foundational pack primitives defined in ADR-0251 (compliance pack and cell certification levels).

This pack is **tenant-opt-in only**. oyatie's own platform data does not flow through
`cn-pipl-eligible` cells. The pack is offered to tenants that explicitly choose to operate
data planes in mainland China to serve PRC-resident users. Activating this pack binds the tenant
to the full PIPL obligation set.

### A.2 What PIPL does

PIPL is China's comprehensive personal information protection framework, analogous in scope
to the EU GDPR. Key structural parallels and divergences:

| Dimension | PIPL | EU GDPR | Divergence |
|---|---|---|---|
| Lawful bases | Article 13: consent, contract, legal obligation, vital interests, public interest, legitimate interests (narrowly scoped) | Article 6: same categories | PIPL legitimate-interests basis is significantly narrower than GDPR; "other circumstances" requires State organ involvement |
| Consent granularity | Article 14: separate per-purpose consent required; bundled consent invalid | Article 7: freely given, specific, informed, unambiguous | Both require granular consent; PIPL enforces separation more rigidly |
| Data minimisation | Article 6: minimum necessary principle | Article 5(1)(c): data minimisation | Functionally equivalent |
| Retention | Article 19: minimum period necessary for purpose; records ≥3 years | Article 5(1)(e): storage limitation | Broadly equivalent; PIPL 3-year records retention is explicit |
| Data subject rights | Articles 44–50: access, copy, portability (15 business days), correction, erasure, explanation of automated decisions | Articles 15–22: analogous rights | PIPL 15-business-day DSAR deadline vs GDPR 1-month; PIPL explicit automated-decision explanation right |
| Cross-border transfer | Article 38: security assessment / certification / SCC pathways | Chapter V: adequacy / SCCs / BCRs | PIPL adds mandatory CAC security assessment for CIIOs and large-volume processors; no adequacy decisions issued yet |
| Data localization | Article 40: CIIOs and above-threshold processors must store in China | No blanket localization (sector-specific only in EU) | PIPL localization requirement is a major architectural constraint |
| Sensitive PI | Article 28: biometric, religious, medical, financial, tracking location, under-14 minor data | Article 9: special categories (biometric, health, religious, etc.) | Broadly similar; PIPL adds financial accounts and tracking location as sensitive; GDPR adds racial/ethnic origin, political opinions, trade union membership |
| Minors | Article 31: under-14 requires guardian consent | GDPR Article 8: 13–16 (member-state varying) | PIPL age threshold is 14 (not 13 or 16) |
| Breach notification | Article 57: notify CAC and subjects without undue delay; max 72h | Articles 33–34: 72h to supervisory authority | Both 72h to regulator; PIPL explicitly adds subjects in same deadline |
| DPIA | Article 51: impact assessment for high-risk processing | Article 35: DPIA for high-risk processing | Functionally equivalent; PIPL assessment includes automated decisions, sensitive PI, and cross-border transfer |
| Designated responsible person | Article 52: processors above threshold must designate a responsible person | Article 37: DPO required for certain controllers | PIPL threshold-based; GDPR criteria-based |

### A.3 Scope: what oyatie tenants this applies to

This pack is **in-scope** for any tenant that:
1. Processes personal information of PRC-resident data subjects.
2. Operates data planes in mainland China (any `cn-pipl-eligible` cell).
3. Provides services accessible to users within mainland China.

This pack is **out of scope** for:
- oyatie's own platform operations (oyatie is a reserved-namespace tenant per ADR-0242 and
  does not activate this pack for its own data).
- Tenants operating exclusively outside mainland China with no PRC-resident data subjects.
- Hong Kong, Macau, and Taiwan, which have separate data protection frameworks (PDPO for HK,
  PDPA for TW) not covered by this pack.

---

## §B — Architecture

### B.1 How this pack integrates with the oyatie substrate

The CN-PIPL-2021 pack follows the ADR-0251 compliance-pack primitive pattern:

```
Tenant activates CN-PIPL-2021
  → Admission gate checks cell_eligibility.requires_certification == ["CN-PIPL-eligible"]
  → Tenant placed only in cn-pipl-eligible cells (mainland-China data-plane)
  → Cedar policy engine loads 6 fragments from packs/cn-pipl/cedar/
  → All PI_CN_PIPL and PI_CN_SENSITIVE processing gated by Cedar
  → Audit chain emits CnPipl* events per action
  → Breach saga activates wf-breach-notification-cn-pipl-72h on SecurityIncidentConfirmed
  → Regulator evidence emitted on annual cadence (dsa_minor_risk_mitigation_assessment_annual
    is not applicable for CN; cadence is annual for PIPL records-of-processing)
```

### B.2 Cell certification requirement

Tenants activating CN-PIPL-2021 MUST be hosted in cells carrying the `cn-pipl-eligible`
certification level (defined in ADR-0251 §D-4). A `cn-pipl-eligible` cell requires:

1. **Data-plane in mainland China only.** Infrastructure physically located in one of:
   Beijing, Shanghai, Guangzhou, Shenzhen, Hangzhou, Chengdu, Zhangjiakou, or other
   CAC-recognized mainland-China data centre regions.
2. **CAC security assessment completed.** The cell operator has completed the CAC security
   assessment under the CAC Measures on Security Assessment for Cross-Border Data Transfer
   (2022-09-01) for the cell's operating context.
3. **CAC-approved KMS.** Key Management Service is one of the CAC-recognized providers
   operating mainland-China KMS endpoints:
   - Alibaba Cloud KMS (cn-* regions)
   - Tencent Cloud KMS (ap-beijing, ap-shanghai, ap-guangzhou regions)
   - Huawei Cloud DEW (cn-* regions)
4. **Mainland-China-resident operations staff.** Control-plane access staff for the cell
   hold PRC citizenship and are physically based in mainland China, satisfying CAC
   personnel requirements for critical information infrastructure operators.

### B.3 Data class extensions

This pack introduces two new data classes to the oyatie data-class registry (ADR-0099):

| Data Class ID | Description | PIPL Reference |
|---|---|---|
| `PI_CN_PIPL` | General personal information as defined by PIPL Article 4: any information relating to identified or identifiable natural persons, recorded by electronic or other means, excluding anonymized information. | PIPL Article 4 |
| `PI_CN_SENSITIVE` | Sensitive personal information per PIPL Article 28: biometric data, religious beliefs, specially-designated medical and health information, financial accounts, tracking location information, and personal information of persons under 14. Requires separate consent, DPIA, and heightened protection. | PIPL Article 28, Article 31 |

### B.4 Cedar fragments loaded

| Fragment ID | Purpose | Key Rules | PIPL Articles |
|---|---|---|---|
| `pack-cn-pipl-2021-data-localization-enforcement` | Enforces that CN-PIPL tenants only activate and operate in cn-pipl-eligible cells; blocks cell migration to non-CN cells | Forbid cell::activate in non-cn-pipl-eligible cells; forbid cell::migrate to non-CN targets | Art. 40 |
| `pack-cn-pipl-2021-consent-gating` | Per-purpose consent validation, 12-month renewal enforcement, withdrawal honoring | Forbid PI processing without active per-purpose consent; forbid processing on expired (>12mo) consent; forbid on withdrawn consent | Art. 13, 14, 15 |
| `pack-cn-pipl-2021-cross-border-transfer-gating` | Blocks cross-border data transfer unless one of the three PIPL Article 38 pathways is attested | Forbid data::cross-border-transfer unless CAC assessment OR certification OR SCC filed | Art. 38 |
| `pack-cn-pipl-2021-minor-protection` | Under-14 guardian consent, no profiling/automated decisions for under-14, no cross-border transfer for under-14 data | Forbid PI processing for under-14 without guardian consent; forbid profiling/automated decisions; forbid cross-border for under-14 | Art. 31; CAC Minors Regs 2024 |
| `pack-cn-pipl-2021-sensitive-pi-handling` | DPIA required before sensitive PI processing; separate category-specific consent; encryption confirmation | Forbid PI_CN_SENSITIVE processing without DPIA; forbid without category-matched consent; forbid without encryption confirmed | Art. 28, 55 |
| `pack-cn-pipl-2021-dsar-response` | DSAR access/copy/portability/erasure for data subjects; 15-business-day deadline enforcement | Permit DSAR actions for authenticated subjects; forbid operator deferral after day 15; permit erasure unless legal hold; permit automated-decision explanation | Art. 44, 45, 47 |

---

## §C — Consent Architecture

PIPL Article 14 requires **separate, individual, informed consent for each processing purpose**.
Bundled consent ("by using this service you agree to all data processing") is explicitly invalid.

### C.1 Per-purpose consent record schema

Each consent record stored in the oyatie consent substrate carries:

```json
{
  "consent_id": "uuid",
  "tenant_id": "uuid",
  "subject_id": "uuid",
  "purpose_id": "purpose-001",
  "purpose_description": "Service provision — user account management",
  "data_classes": ["PI_CN_PIPL"],
  "collected_at": "2026-05-20T10:00:00Z",
  "expires_at": "2027-05-20T10:00:00Z",
  "status": "active",
  "withdrawn": false,
  "withdrawn_at": null,
  "collection_method": "explicit-opt-in",
  "language": "zh-CN",
  "version": "consent-template-cn-pipl-v1"
}
```

### C.2 12-month renewal

Per CAC guidance and this pack's policy, consent records older than 12 months are treated as
expired. The consent substrate emits `CnPiplConsentRenewalRequired` events 30 days before
expiry. Tenants must implement a renewal UX surface. The Cedar fragment
`pack-cn-pipl-2021-consent-gating` denies processing on expired consent.

### C.3 Withdrawal mechanism

PIPL Article 15 mandates that withdrawal must be no harder than collection. Implementation:
- Withdrawal mechanism MUST be accessible from the same surface where consent was collected.
- Withdrawal takes effect immediately upon submission (no cooling-off period).
- Cedar fragment blocks processing within seconds of withdrawal being recorded in the consent store.

---

## §D — Cross-Border Transfer

PIPL Article 38 provides three pathways for cross-border transfer of personal information.
No cross-border transfer is permitted without one of these pathways being attested.

### D.1 Pathway 1 — CAC Security Assessment

**Required for:**
- Critical Information Infrastructure Operators (CIIOs)
- Processors transferring sensitive PI of ≥10,000 subjects/year
- Processors transferring general PI of ≥1,000,000 total or ≥100,000 per year

**How to complete:**
1. Submit to the CAC cross-border assessment portal: `https://cbdt.cac.gov.cn`
2. Assessment review period: typically 45 business days (extendable by CAC)
3. Result valid for 2 years; renew within 60 days before expiry
4. Store assessment reference number in the tenant's `cac_security_assessment_ref` field

### D.2 Pathway 2 — Personal Information Protection Certification

**Available for:**
- Processors below CIIO/volume thresholds
- Preferred for smaller-scale cross-border scenarios

**How to complete:**
1. Engage CCRC (China Cybersecurity Review Technology and Certification Center): `https://www.ccrc.org.cn`
2. Certification scope assessment: 6–12 months typically
3. Certificate valid for 3 years
4. Store certificate number in `personal_information_protection_cert_ref` field

### D.3 Pathway 3 — Standard Contractual Clauses

**Available for:**
- Processors below CIIO/volume thresholds where Pathway 1 is not mandatory
- Cross-border transfers with a specific counterparty

**How to complete:**
1. Execute the CAC Standard Contract for Cross-Border Transfer of Personal Information
   (effective 2023-06-01; template at: `https://www.cac.gov.cn/standard-contract`)
2. File with the provincial-level CAC office (省级网信办) within 10 business days of execution
3. Obtain filing reference number; store in `standard_contractual_clauses_filing_ref` field

### D.4 Hyperscaler reference: how AWS China handles cross-border

AWS China (Sinnet/NWCD) requires tenants who need to replicate data from cn-* regions to
non-CN regions to: (1) complete the CAC security assessment themselves (the ICP license
is held by Sinnet/NWCD, not the tenant), (2) configure AWS S3 Cross-Region Replication
only after attestation is on file in the tenant's compliance documentation, and (3) use
AWS KMS China keys for all replicated objects with a separate key hierarchy for non-CN
destinations. This is the exact pattern CN-PIPL-2021 implements via the cross-border-transfer-gating Cedar fragment.

---

## §E — Breach Notification (PIPL Article 57, 72-hour cadence)

PIPL Article 57 requires: "upon occurrence of or likely occurrence of personal information
leakage, tampering, or loss, personal information processors shall immediately take remedial
measures and inform the competent authorities and the affected personal information subjects."
The maximum time from awareness to notification is **72 hours**.

### E.1 Workflow summary

The breach notification workflow `wf-breach-notification-cn-pipl-72h` orchestrates:

| Stage | Deadline | Key Action |
|---|---|---|
| Detect | T+1h | Confirm breach; classify data classes; set T=0 awareness timestamp |
| Triage | T+4h | Scope analysis; notify internal stakeholders |
| Emergency Response | T+6h | Isolate systems; preserve evidence |
| CAC Notification | T+72h | File with CAC incident portal; obtain confirmation number |
| Subject Notification | Immediately after CAC | Per-subject notification via preferred channel |
| Remediation | T+14d | Track all remediation actions to completion |
| Post-Mortem | T+30d | Seal final report in audit chain |

### E.2 CAC notification required fields

Per PIPL Article 57 and CAC guidance: incident type, awareness timestamp, affected data
classes, estimated subject count, affected mainland-China regions, suspected cause,
cross-border egress indicator, emergency response measures taken, and responsible person
contact details.

---

## §F — Data Subject Rights (PIPL Articles 44–50)

| Right | PIPL Article | Deadline | Cedar Fragment | Audit Event |
|---|---|---|---|---|
| Access and copy | Article 45 | 15 business days | pack-cn-pipl-2021-dsar-response | CnPiplDsarAccessPermit |
| Portability | Article 45 | 15 business days | pack-cn-pipl-2021-dsar-response | CnPiplDsarAccessPermit |
| Correction | Article 46 | Reasonable time | (via governance DSAR cascade) | CnPiplDsarCorrectionPermit |
| Erasure | Article 47 | Reasonable time | pack-cn-pipl-2021-dsar-response | CnPiplDsarErasurePermit |
| Explanation of automated decisions | Article 44 | Reasonable time | pack-cn-pipl-2021-dsar-response | CnPiplDsarAutomatedDecisionExplanationPermit |
| Restrict / Object | Article 44 | Immediately | pack-cn-pipl-2021-consent-gating | CnPiplConsentWithdrawnDeny (on withdrawal path) |

### F.1 Identity verification requirement

All DSAR requests require subject identity verification before processing. Acceptable methods
for PRC-resident users:
- WeChat / Alipay real-name verification (via official SDK integration)
- National ID card (居民身份证) verification via MIIT identity verification API
- Mobile number verification (registered to national ID per Telecom real-name rules)

---

## §G — References

### Binding documents

- **ADR-0251** — Compliance Pack and Cell Certification Levels (primary binding ADR; defines
  the pack primitive, cert-level matrix including `cn-pipl-eligible`, and schema)
- **ADR-0242** — oyatie-is-a-tenant doctrine (defines reserved-namespace tenancy scope)
- **ADR-0243** — Cedar as universal gate (Cedar gating architecture)
- **ADR-0248** — Amazon cellular architecture (cell topology; cn-pipl-eligible cells are
  Tier-1 cells in mainland-China data-plane regions)
- **ADR-0255** — Intelligence two-layer substrate (provider BYOK; `provider_byok_required: true`
  on this pack per ADR-0255 §D-4 for regulated-tier tenants using LLM features in CN cells)

### Pack files

- `packs/cn-pipl/manifest.json` — machine-readable pack declaration (conforms to
  `/specs/compliance-pack-schema.json`)
- `packs/cn-pipl/cedar/data-localization-enforcement.cedar` — data localization rules
- `packs/cn-pipl/cedar/consent-gating.cedar` — consent validation and renewal
- `packs/cn-pipl/cedar/cross-border-transfer-gating.cedar` — cross-border transfer controls
- `packs/cn-pipl/cedar/minor-protection.cedar` — under-14 guardian consent and restrictions
- `packs/cn-pipl/cedar/sensitive-pi-handling.cedar` — sensitive PI category handling
- `packs/cn-pipl/cedar/dsar-response.cedar` — data subject rights enforcement
- `packs/cn-pipl/breach-notification-workflow.yaml` — 72h breach notification workflow
- `packs/cn-pipl/dpia-template.md` — DPIA template per PIPL Article 51
- `packs/cn-pipl/regulator-references.yaml` — CAC + MIIT contact channels
- `packs/cn-pipl/CHANGELOG.md` — pack version history

### Regulatory citations

- Personal Information Protection Law (PIPL, 个人信息保护法), effective 2021-11-01.
  Full text (official): `https://www.gov.cn/xinwen/2021-08/20/content_5632486.htm`
- Cybersecurity Law of the PRC (网络安全法), effective 2017-06-01
- Data Security Law of the PRC (数据安全法), effective 2021-09-01
- CAC Measures on Security Assessment for Cross-Border Data Transfer, effective 2022-09-01.
  `https://www.cac.gov.cn/2022-07/07/c_1658803993082790.htm`
- CAC Standard Contract Measures for Cross-Border Transfer of Personal Information,
  effective 2023-06-01. `https://www.cac.gov.cn/2023-02/24/c_1679401231557627.htm`
- CAC Regulations on the Protection of Minors in Cyberspace, effective 2024-01-01.
  `https://www.cac.gov.cn/2023-10/16/c_1699339588638990.htm`
- MIIT Rules on Protection of User Personal Information by Telecom and Internet Service
  Providers (2013, amended through 2022)

### Hyperscaler precedents

- **AWS China (Sinnet/NWCD):** PIPL compliance documentation at `https://www.amazonaws.cn/en/compliance/`
- **Alibaba Cloud:** PIPL compliance toolkit and Data Security Center at `https://www.alibabacloud.com/trust-center/pipl`
- **Tencent Cloud:** Compliance center at `https://www.tencentcloud.com/document/product/1078`
- **Microsoft Azure China (21Vianet):** PIPL guidance at `https://docs.azure.cn/zh-cn/articles/azure-operations-guide/others/aog-others-howto-understand-pipl`

### F13 compliance gate

This pack was created to close the P1 finding in
`evidence/debate/keystone-bundle-2026-05-20-F13-compliance-r1.json`:
> "China PIPL / CAC Minor Cyberspace Regulations 2024 — PRC jurisdiction scope is
> ambiguous; if in-scope, separate consent + CAC security assessment + minor cyberspace
> rules are P0 gaps; if out-of-scope, the boundary should be explicit in ADR-0251
> jurisdiction table."

This pack makes the scope decision explicit: **CN-PIPL-2021 is offered to opt-in tenants;
oyatie's own data is out of scope**. The pack provides the full control surface for tenants
that choose to operate PRC-resident data planes.
