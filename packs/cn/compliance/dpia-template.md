---
doc_class: DPIA-Template
dpia_template_id: dpia-cn-pipl-v1
pack_id: CN-PIPL-2021
binding_adr: ADR-0251
regulation: PIPL Article 51 (personal information protection impact assessment)
version: "1.0.0"
status: Accepted
date: 2026-05-20
jurisdiction: CN
enforcement_status: advisory-until-pack-registry-substrate-lands
related_adrs:
  - ADR-0251
  - ADR-0242
  - ADR-0243
companion_docs:
  - packs/cn-pipl/README.md
  - packs/cn-pipl/manifest.json
  - docs/standards/documentation-rigor.md
planned_enforcement_ref: oya-governance-doc-rigor
---

# DPIA Template — CN-PIPL-2021
## Personal Information Protection Impact Assessment (个人信息保护影响评估)

> This template satisfies PIPL Article 51. Every tenant activating CN-PIPL-2021 MUST complete
> this assessment before activating any processing of PI_CN_PIPL or PI_CN_SENSITIVE data.
> The completed DPIA must be signed by the tenant's designated Data Protection Officer (or
> equivalent responsible person per PIPL Article 52) and retained for at least 3 years.

---

## §A — Processing Activity Description

**A.1 Activity name:**
_[Required: human-readable name for this processing activity]_

**A.2 Processing purpose(s):**
_[Required: list each purpose separately per PIPL Article 6 — specific, explicit, legitimate]_

| Purpose ID | Purpose Description | Legal Basis (PIPL Article 13) | Consent Record ID |
|---|---|---|---|
| `purpose-001` | | | |

**A.3 Data controller identity:**
_[Required: full legal name, registered address, ICP license number if applicable]_

**A.4 Data processor identity (if different):**
_[Required if processing is delegated: full legal name, contract reference, PIPL Article 21 commission agreement ref]_

**A.5 Categories of personal information processed:**

| Data Class | Sensitive PI? (PIPL Art. 28) | Volume (estimated subjects) | Retention Period |
|---|---|---|---|
| PI_CN_PIPL | No | | |
| PI_CN_SENSITIVE — Biometric | Yes (Art. 28) | | |
| PI_CN_SENSITIVE — Medical/Health | Yes (Art. 28) | | |
| PI_CN_SENSITIVE — Financial Account | Yes (Art. 28) | | |
| PI_CN_SENSITIVE — Location (tracking) | Yes (Art. 28) | | |
| PI_CN_SENSITIVE — Minor (under 14) | Yes (Art. 31) | | |
| PI_CN_SENSITIVE — Religious Belief | Yes (Art. 28) | | |

**A.6 Data subjects:**
_[Describe the categories of data subjects: consumers, employees, minors, etc.]_

**A.7 Third-party recipients:**
_[List all recipients with their legal basis for receiving data. If cross-border transfer: confirm CAC pathway per PIPL Article 38 — see §E.]_

---

## §B — Necessity and Proportionality Assessment

**B.1 Is the processing necessary to achieve the stated purpose?**
_[Required: explain why this data is the minimum necessary. PIPL Article 6 minimum necessary principle.]_

**B.2 Could a less privacy-invasive approach achieve the same purpose?**
_[Required: document alternatives considered and why rejected.]_

**B.3 Is the processing proportionate to the purpose?**
_[Required: explain the proportionality balance.]_

**B.4 Are automated decision-making or profiling involved?** (Yes / No)
If yes:
- PIPL Article 24 disclosure to data subjects required: ☐ Completed
- PIPL Article 44 right to explanation mechanism in place: ☐ Completed
- Opt-out mechanism provided: ☐ Completed

---

## §C — Risk Identification

For each risk below, rate: **Low / Medium / High / Critical**

| Risk Category | Likelihood | Impact | Net Risk | Mitigation |
|---|---|---|---|---|
| Unauthorized access to PI_CN_PIPL data | | | | |
| Data leakage via cross-border transfer (PIPL Art. 40 violation) | | | | |
| Consent not properly obtained or renewed (PIPL Art. 14) | | | | |
| Sensitive PI processed without separate consent (PIPL Art. 28) | | | | |
| Minor data processed without guardian consent (PIPL Art. 31) | | | | |
| DSAR response deadline exceeded (15 business days per Art. 45) | | | | |
| Breach notification deadline exceeded (72h per Art. 57) | | | | |
| Data retained beyond purpose (PIPL Art. 19 retention violation) | | | | |
| Automated decision violates Art. 24 fairness principle | | | | |
| Vendor / sub-processor non-compliance (PIPL Art. 21) | | | | |

**C.1 Residual risk assessment:**
_[After applying mitigations, what is the residual risk level? If any residual risk is High or Critical, escalate to senior management before activating.]_

---

## §D — Technical and Organisational Measures

**D.1 Encryption:**
- At rest: ☐ AES-256 (BYOK required per CN-PIPL-2021 pack — tenant-supplied KMS root key)
- In transit: ☐ TLS 1.3 minimum
- Key management: ☐ CAC-approved KMS (Alibaba Cloud KMS / Tencent Cloud KMS / Huawei Cloud DEW in mainland-CN region)

**D.2 Access control:**
- Cedar policy fragment `pack-cn-pipl-2021-sensitive-pi-handling` loaded: ☐
- Minimum-privilege access enforced: ☐
- Privileged access audit logging enabled: ☐

**D.3 Data localization:**
- All PI_CN_PIPL and PI_CN_SENSITIVE data stored in cn-pipl-eligible cells only: ☐
- Cell certification evidence on file: ☐ (cell_id: _________)
- Cedar fragment `pack-cn-pipl-2021-data-localization-enforcement` active: ☐

**D.4 Consent management:**
- Per-purpose consent collection implemented: ☐
- 12-month renewal mechanism active: ☐
- Withdrawal mechanism in place and easier than collection: ☐
- Consent records stored with tamper-detection (Merkle-sealed): ☐

**D.5 Breach notification:**
- Breach notification workflow `wf-breach-notification-cn-pipl-72h` deployed: ☐
- CAC notification endpoint configured: ☐
- Subject notification template approved by DPO: ☐

**D.6 DSAR infrastructure:**
- 15-business-day response SLA configured: ☐
- Cedar fragment `pack-cn-pipl-2021-dsar-response` active: ☐
- Subject identity verification mechanism in place: ☐

---

## §E — Cross-Border Data Transfer Assessment

> Complete this section only if any personal information will be transferred outside mainland China.
> If no cross-border transfer occurs, mark N/A and confirm data localization at §D.3.

**E.1 Is cross-border transfer of PI_CN_PIPL or PI_CN_SENSITIVE data required?** ☐ Yes / ☐ No / ☐ N/A

If Yes, which CAC pathway applies (select one):

☐ **Pathway 1 — CAC Security Assessment** (PIPL Article 38(1); mandatory for CIIOs and processors above volume threshold):
- CAC security assessment application reference: _______________
- Assessment result (Pass/Fail): _______________
- Assessment validity period: _______________

☐ **Pathway 2 — Personal Information Protection Certification** (PIPL Article 38(2)):
- Certifying institution (CAC-accredited): _______________
- Certificate number: _______________
- Certificate validity period: _______________

☐ **Pathway 3 — Standard Contractual Clauses** (PIPL Article 38(3); CAC Standard Contract Measures 2023):
- Counterparty name and jurisdiction: _______________
- Contract execution date: _______________
- Provincial CAC filing reference number: _______________
- Filing date: _______________

**E.2 Volume threshold check:**
Per CAC Measures 2022 Article 4, security assessment is MANDATORY (Pathway 1 only) if:
- CIIO status: ☐ Yes / ☐ No
- Exported sensitive PI subjects ≥ 10,000 per year: ☐ Yes / ☐ No
- Exported general PI subjects ≥ 1,000,000 total or ≥ 100,000 per year: ☐ Yes / ☐ No

If any above is Yes, only Pathway 1 is available.

---

## §F — Consultation and Sign-off

**F.1 DPO / Responsible Person consultation:**

| Role | Name | Date | Signature |
|---|---|---|---|
| Responsible Person (PIPL Article 52) | | | |
| Data Protection Officer (if separate) | | | |
| CN Legal Counsel | | | |
| CISO | | | |
| External auditor (if required) | | | |

**F.2 Escalation to senior management required?**
_[Required if any residual risk at §C is High or Critical.]_ ☐ Yes / ☐ No

If Yes:
- Senior management sign-off: _______________
- Date: _______________

**F.3 DPIA review cadence:**
This DPIA MUST be reviewed and re-signed:
- At least every 2 years
- Upon material change to the processing activity
- Upon a new regulatory guidance issuance by CAC or MIIT
- After any breach involving PI_CN_PIPL or PI_CN_SENSITIVE data

---

## §G — References

### Hyperscaler precedents for DPIA practice

**Alibaba Cloud** publishes a PIPL Data Protection Impact Assessment guidance document
(数据保护影响评估指南) requiring tenants processing PI_CN_SENSITIVE to complete a DPIA
before activating the Data Security Center (DSC) sensitive-data scan job. The Alibaba
guidance maps directly to PIPL Article 51's four trigger categories (automated decisions,
sensitive PI, cross-border transfer, high-volume processing) — the same four categories
this template covers in §B and §C.

**Tencent Cloud** requires DPIA completion as a gate before enabling Tencent Cloud's
privacy-enhanced computing (隐私计算) service for any data classified as sensitive PI under
PIPL Article 28. The Tencent model uses a risk matrix identical in structure to §C above.

**Microsoft Azure China (21Vianet)** provides a PIPL DPIA template through Compliance
Manager China, structured around the same §A–§F sections. Microsoft requires the DPIA
to be re-signed after any change to processing purpose, data categories, or third-party
recipients — matching the review cadence stated in §F.3 of this template.

### Regulatory citations

- Personal Information Protection Law (PIPL, 个人信息保护法), effective 2021-11-01
- PIPL Article 51 (personal information protection impact assessment obligation)
- PIPL Article 6 (purpose limitation and minimum necessary principle)
- PIPL Article 14 (separate per-purpose consent)
- PIPL Article 19 (retention limitation)
- PIPL Article 21 (third-party provision and commission processing)
- PIPL Article 24 (automated decision-making fairness and transparency)
- PIPL Article 28 (sensitive personal information — separate consent + heightened protection)
- PIPL Article 31 (minor protection — guardian consent for under-14)
- PIPL Article 38 (cross-border transfer conditions)
- PIPL Article 40 (data localization for CIIOs and above-threshold processors)
- PIPL Article 44 (right to explanation of automated decisions)
- PIPL Article 45 (DSAR — 15-business-day response)
- PIPL Article 51 (DPIA obligation)
- PIPL Article 52 (designated responsible person requirement)
- PIPL Article 57 (breach notification — 72h to CAC and subjects)
- CAC Measures on Security Assessment for Cross-Border Data Transfer, effective 2022-09-01
- CAC Standard Contract Measures for Cross-Border Transfer, effective 2023-06-01
- CAC Regulations on the Protection of Minors in Cyberspace, effective 2024-01-01
- ADR-0251 (compliance pack and cell certification levels — binding ADR for this template)
- `packs/cn-pipl/manifest.json` (pack declaration)
- `packs/cn-pipl/breach-notification-workflow.yaml` (breach notification procedure)
- `packs/cn-pipl/regulator-references.yaml` (CAC + MIIT contact channels)
