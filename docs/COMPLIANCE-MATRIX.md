---
purpose: Oyatie — Compliance Matrix
doc_status: published
---

# Oyatie — Compliance Matrix

> **Status:** Draft v0.1 — 2026-05-09. The compliance matrix is the regulator × control × evidence × cadence × owner table that aggregates every regulator Oyatie binds to via regional packs and verticals.
> **Owner:** `ops-compliance`. Updates per [DOC-CATALOG.md `doc.compliance_matrix`](DOC-CATALOG.md) (monthly + per EVT-REGULATORY-CHANGE / EVT-AUDIT-FINDING).
> **Companion:** [`machine-readable/compliance.json`](machine-readable/compliance.json), [`PRIVACY-PROGRAM.md`](PRIVACY-PROGRAM.md), [`security-program/security-program.json`](security-program/security-program.json).

---

## 1. Reading guide

Each regulator gets a section. Each control inside the section is a row with:

- **Control ID** (per regulator's framework numbering)
- **Description** (one line)
- **Implementing surface** (which Oyatie axis or per-product PRD owns the implementation)
- **Evidence type** (audit-chain emission, screenshot, attestation, fuzz result, contract test, DPIA, …)
- **Evidence cadence** (continuous, daily, weekly, monthly, quarterly, annually, per-incident)
- **Owner team** (which `teams/<id>/CHARTER.md`)
- **Status** (in-design / preview / stable / GA-evidenced)

The single `platform-regulatory-kernel` (per [DESIGN §12.2 seam](DESIGN.md)) drives the *implementation*; this matrix tracks the *evidence*.

---

## 2. Cross-region regulator inventory

The full set Oyatie binds to via regional packs (initial v0.1 — expands as packs onboard):

| Regulator / Standard | Region(s) | Pack(s) | Verticals affected | First wave required |
|---|---|---|---|---|
| KR PIPA (개인정보보호법) | South Korea | `pack-kr` | All | W-Foundation (cross-axis privacy posture) |
| KR KISA Security Assessment | South Korea | `pack-kr` | SaaS, Cloud | W-Cloud-Preview |
| KR CSAP (Cloud Security Assurance Program) | South Korea | `pack-kr` | Cloud | W-Cloud-Stable |
| KR K-ISMS-P | South Korea | `pack-kr` | All commercial | W-Cloud-Preview |
| KR KCMVP (cryptographic module validation) | South Korea | `pack-kr` | Cloud (KMS), Security | W-Cloud-Preview (HSM lead 6-9 mo) |
| KR MFDS (식품의약품안전처) | South Korea | `pack-kr` | Healthcare | W-Vertical-Pilot (if healthcare) |
| KR FSC + 금감원 + 신용정보법 | South Korea | `pack-kr` | Fintech | per fintech vertical wave |
| KR KCC + 정보통신망법 | South Korea | `pack-kr` | All; ads heavy | per ads/comms wave |
| KR NIS | South Korea | `pack-kr` | Defense (out of scope per anti-scope), Public-sector | per public-sector wave |
| KR 청소년보호법 | South Korea | `pack-kr` | Education, ads, content | per education / ads wave |
| KR 의료법 + 의료광고심의 | South Korea | `pack-kr` | Healthcare, ads-medical | W-Vertical-Pilot (healthcare) |
| KR 약사법 | South Korea | `pack-kr` | Healthcare (pharma) | per healthcare wave |
| KR 망분리 | South Korea | `pack-kr` | Cloud (network), Fintech, Public-sector | W-Cloud-Preview |
| KR 전자세금계산서 (NTS) | South Korea | `pack-kr` | All (billing) | W-Cloud-Preview |
| KR 조달청 (Public Procurement) | South Korea | `pack-kr` | Public-sector, Cloud GA | W-Cloud-Stable |
| JP APPI (個人情報保護法) | Japan | `pack-jp` | All | W-Cloud-Preview |
| JP ISMAP (Information System Security Management & Assessment) | Japan | `pack-jp` | Cloud (gov) | W-Cloud-Stable |
| JP PMDA | Japan | `pack-jp` | Healthcare | per healthcare wave |
| JP JFSA | Japan | `pack-jp` | Fintech | per fintech wave |
| JP マイナンバー法 | Japan | `pack-jp` | SaaS (identity) | W-Cloud-Preview |
| JP 適格請求書 (e-invoicing) | Japan | `pack-jp` | All (billing) | W-Cloud-Preview |
| US HIPAA + HITECH | United States | `pack-us` | Healthcare | W-Vertical-Pilot (healthcare) |
| US PCI-DSS v4.0 | United States | `pack-us` | Fintech, all card-touching | per fintech wave |
| US SOX | United States | `pack-us` | All (public-co customers) | W-Cloud-Stable |
| US CCPA / CPRA | California | `pack-us` (state overlay) | All US tenants | W-Cloud-Preview |
| US State AG (NY SHIELD, IL BIPA, TX TDPSA, …) | per state | `pack-us` (state overlays) | All US tenants | per state |
| US FedRAMP (Moderate / High) | United States Federal | `pack-us-fed` (subset of `pack-us`) | Cloud (gov) | W-Cloud-Stable |
| US OCC + FFIEC | United States | `pack-us` | Fintech | per fintech wave |
| US FDA (21 CFR Part 11 + 820 + SaMD) | United States | `pack-us` | Healthcare devices | per healthcare wave |
| US 42 CFR Part 2 (substance use) | United States | `pack-us` | Healthcare | per healthcare wave |
| EU GDPR | European Union | `pack-eu` | All EU tenants | W-Cloud-Preview |
| EU DORA | European Union | `pack-eu` | Fintech | per fintech wave |
| EU AI Act | European Union | `pack-eu` | All AI surfaces (Foundry esp.) | W-Foundry-Preview |
| EU GAIA-X | European Union | `pack-eu` | Cloud | W-Cloud-Stable |
| EU EMA | European Union | `pack-eu` | Healthcare devices | per healthcare wave |
| EU NIS2 | European Union | `pack-eu` | Cloud, all | W-Cloud-Stable |
| EU eIDAS | European Union | `pack-eu` | SaaS (identity) | W-Cloud-Preview |
| EU e-Invoicing (per country: DE / IT / FR / PL / ES / …) | European Union | `pack-eu` (per-country overlays) | All (billing) | W-Cloud-Preview |
| EU DSA / DMA | European Union | `pack-eu` | Ads, marketplace, search | W-Search-Preview / W-Ads-Preview |
| EU MiCA | European Union | `pack-eu` | Fintech (crypto) | if applicable |
| IN DPDP Act (Digital Personal Data Protection) | India | `pack-in` | All | per IN wave |
| IN MeitY empanelment | India | `pack-in` | Cloud (gov) | per cloud wave |
| IN RBI | India | `pack-in` | Fintech | per fintech wave |
| IN CDSCO | India | `pack-in` | Healthcare | per healthcare wave |
| IN GST e-invoicing | India | `pack-in` | All (billing) | per IN wave |
| BR LGPD | Brazil | `pack-br` | All | per BR wave |
| BR ANS / ANVISA | Brazil | `pack-br` | Healthcare | per healthcare wave |
| BR BACEN | Brazil | `pack-br` | Fintech | per fintech wave |
| BR ICP-Brasil | Brazil | `pack-br` | SaaS (identity) | per BR wave |
| BR NF-e (electronic invoice) | Brazil | `pack-br` | All (billing) | per BR wave |
| KSA PDPL | Saudi Arabia | `pack-ksa` | All | per KSA wave |
| KSA NDMO | Saudi Arabia | `pack-ksa` | Cloud | per cloud wave |
| KSA SDAIA | Saudi Arabia | `pack-ksa` | All AI | per Foundry wave |
| KSA SAMA | Saudi Arabia | `pack-ksa` | Fintech | per fintech wave |
| KSA SFDA | Saudi Arabia | `pack-ksa` | Healthcare | per healthcare wave |
| UAE TDRA / ADGM / DIFC | United Arab Emirates | `pack-ae` | All (varies by zone) | per AE wave |
| UAE UAE-CB | United Arab Emirates | `pack-ae` | Fintech | per fintech wave |
| UAE FATOORA (e-invoicing) | United Arab Emirates | `pack-ae` | All (billing) | per AE wave |
| AU Privacy Act 1988 | Australia | `pack-au` | All | per AU wave |
| AU IRAP | Australia | `pack-au` | Cloud (gov) | per cloud wave |
| AU TGA | Australia | `pack-au` | Healthcare | per healthcare wave |
| AU ASIC | Australia | `pack-au` | Fintech | per fintech wave |
| SG PDPA | Singapore | `pack-sg` | All | per SG wave |
| SG MAS | Singapore | `pack-sg` | Fintech | per fintech wave |
| SG HSA | Singapore | `pack-sg` | Healthcare | per healthcare wave |
| SG IMDA | Singapore | `pack-sg` | Telecom, ads | per SG wave |
| **Cross-regional standards (no specific country)** | global | (all packs inherit) | All | per axis wave |
| ISO 27001 / 27017 / 27018 / 27701 | global | inherit | All | W-Cloud-Stable |
| SOC 2 Type II | global | inherit | All | W-Cloud-Stable |
| NIST CSF + SP 800-53 + 800-171 | global | inherit | All | W-Cloud-Stable |
| CSA STAR | global | inherit | Cloud | W-Cloud-Stable |
| FIPS 140-3 | global (US-led) | inherit | Crypto | W-Cloud-Preview |

---

## 3. Per-regulator detailed control matrix

> Each regulator below has a sub-table. Detail level: only enough for an auditor to find the evidence. Full attestations live in the per-pack regulatory archive.

### 3.1 KR PIPA (개인정보보호법) — illustrative deep-dive

| Article | Title | Implementing surface | Evidence | Cadence | Owner | Status |
|---|---|---|---|---|---|---|
| Art 15 | Lawful basis for collection | `platform-tenant-kernel` consent receipts; Data Use Boundary class taxonomy | Consent receipt audit-chain emission (per onboarding); class-annotation lint exit | continuous + per-onboarding | `council-privacy` + `axis-saas` | in-design |
| Art 17 | Cross-border transfer | `platform-tenant-kernel.residency`; cross-region replication policy | Per-tenant residency-binding evidence; regional-pack residency contract | continuous | `regional-packs/pack-kr` + `council-privacy` | in-design |
| Art 22 | Purpose-bound consent + granular controls | PRIVACY-PROGRAM §2.2.2 consent ladder; UI surfaces per regional pack | Consent-receipt audit; UI-screenshot attestation | continuous + monthly | `council-privacy` | in-design |
| Art 22-2 | Children under 14 | PRIVACY-PROGRAM §2.2.1 class 13 hard-deny; tenant-class override | Class-annotation lint; tenant onboarding declaration | continuous | `council-privacy` | in-design |
| Art 23 | Sensitive data | PRIVACY-PROGRAM class 12; HARD_DENY for ad targeting | Class-annotation lint; runtime guard | continuous | `council-privacy` | in-design |
| Art 28-8 | Cross-border transfer evidence | `pack-kr` residency contract; per-export audit-chain emission | Per-export audit; quarterly review | per-event + quarterly | `regional-packs/pack-kr` + `council-privacy` | in-design |
| Art 29 | Security measures | SECURITY-PROGRAM controls; KMS envelope encryption per ADR-0043; OpenBao per ADR-0043 | KMS audit-chain; OpenBao audit log; Trivy report | daily + per-incident | `ops-security` | in-design |
| Art 34 | Breach notification | INCIDENT-MANAGEMENT.md severity taxonomy; 72-hour PIPC notification | Incident postmortem; PIPC notification artifact | per-incident | `ops-sre-reliability` + `council-privacy` | in-design |
| Art 39-7 | DSR / withdrawal cascade | PRIVACY-PROGRAM §2.2.9 cascade; 30-day SLA | Proof-of-erasure record; DSR queue dashboard | per-DSR + monthly | `council-privacy` | in-design |

### 3.2 GDPR — illustrative deep-dive

| Article | Title | Implementing surface | Evidence | Cadence | Owner | Status |
|---|---|---|---|---|---|---|
| Art 5 | Principles (lawful, fair, purpose-limited, minimization, accuracy, storage limitation, integrity, accountability) | Cross-cutting | Per-axis principle-mapping doc; audit-chain | continuous | `council-privacy` | in-design |
| Art 6 | Lawful basis | `platform-tenant-kernel.consent` | Consent receipt | continuous + per-onboarding | `council-privacy` | in-design |
| Art 7 | Conditions for consent | UI surfaces per regional pack | UI screenshot attestation | continuous + monthly | `regional-packs/pack-eu` | in-design |
| Art 9 | Special categories (health, biometric, genetic, sex life, religion, political views) | Class 12 hard-deny | Lint + runtime guard | continuous | `council-privacy` | in-design |
| Art 17 | Right to erasure | DSR cascade | Proof-of-erasure | per-DSR | `council-privacy` | in-design |
| Art 22 | Automated decision-making | Foundry autonomy ceiling per ADR-0022; per-decision evidence | Audit-chain per agent step | continuous | `axis-foundry` + `council-privacy` | in-design |
| Art 25 | Data protection by design + default | Data Use Boundary ADR; class taxonomy; consent ladder | DPIA per-vertical | per-vertical | `council-privacy` | in-design |
| Art 30 | Records of processing | Per-tenant per-axis processing register | Auto-generated from catalog | continuous | `axis-foundry` (catalog) | in-design |
| Art 32 | Security of processing | SECURITY-PROGRAM | Trivy + Cosign + audit-chain | continuous | `ops-security` | in-design |
| Art 33 | Breach notification (72h) | INCIDENT-MANAGEMENT | Incident postmortem | per-incident | `ops-sre-reliability` + `council-privacy` | in-design |
| Art 35 | DPIA | Per-vertical DPIA template | DPIA artifact per vertical onboarding | per-vertical | `council-privacy` | in-design |

### 3.3 HIPAA — illustrative

| Rule | Title | Implementing surface | Evidence | Cadence |
|---|---|---|---|---|
| Privacy Rule §164.502 | Uses + disclosures | Class 2 (PHI) hard-deny for ads; class-annotation lint | continuous |
| Security Rule §164.312 | Technical safeguards | KMS encryption per ADR-0043; access control; audit-chain emission | continuous |
| Breach Notification §164.404 | 60-day notification | INCIDENT-MANAGEMENT severity 1-2 + HIPAA addendum | per-incident |
| 21 CFR Part 11 | Electronic records + signatures | `vertical-healthcare-app-electronic-signature-*` | per-document |

### 3.4 PCI-DSS v4.0 — illustrative

| Requirement | Title | Implementing surface | Evidence | Cadence |
|---|---|---|---|---|
| Req 3 | Protect stored cardholder data | Class 5 (PCI) hard-deny; KMS envelope encryption; tokenization service | continuous |
| Req 4 | Encrypt cardholder data in transit | mTLS via service mesh per ADR-0044; TLS 1.3 minimum | continuous |
| Req 6 | Develop and maintain secure systems | Trivy 4-layer per ADR-0039; Cosign signing per ADR-0039 | per-build |
| Req 8 | Identify and authenticate access | RBAC + Cedar policy + MFA for cardholder-data access | continuous |
| Req 10 | Track and monitor all access | Audit-chain per ADR-0003; per-access emission | continuous |
| Req 11 | Test security of systems regularly | Quarterly pen-test; annual red team; continuous fuzz harness | quarterly + annual |
| Req 12 | Information security policy | SECURITY-PROGRAM | annual review |

### 3.5 SOC 2 Type II — illustrative

| TSC | Title | Implementing surface | Evidence | Cadence |
|---|---|---|---|---|
| Security (Common Criteria) | CC1-CC9 | SECURITY-PROGRAM full coverage | continuous |
| Availability | CC7 + A1 | SLO-CATALOG + INCIDENT-MANAGEMENT + DR-CAPACITY | continuous |
| Processing Integrity | PI1 | Test-strategy + evidence-chain | per-release |
| Confidentiality | C1 | Data Use Boundary + KMS | continuous |
| Privacy | P1-P8 | PRIVACY-PROGRAM | continuous |

### 3.6 ISO 27001 / 27017 / 27018 / 27701 — control mapping

> **TODO v0.2** — full A.5 ... A.18 mapping per ISO 27001:2022 Annex A; cross-reference to existing controls. Same pattern for 27017 (cloud), 27018 (PII in cloud), 27701 (PIMS).

### 3.7 EU AI Act — illustrative

| Article | Title | Implementing surface | Evidence | Cadence |
|---|---|---|---|---|
| Art 6 | High-risk AI systems classification | Per-capability classification in `registry/capability-templates/` | per-capability | per-publish + quarterly |
| Art 9 | Risk management system | Foundry autonomy ceiling + eval harness | per-capability + monthly |
| Art 10 | Data governance | Data Use Boundary + per-tenant evidence | continuous |
| Art 11 | Technical documentation | Per-capability rustdoc + per-capability eval set | per-capability |
| Art 12 | Record-keeping | Audit-chain per ADR-0003 + per-step span emission | continuous |
| Art 13 | Transparency | Per-capability `transparency_disclosure:` field | per-capability |
| Art 14 | Human oversight | Autonomy tier T1-T4 per ADR-0022; break-glass | per-capability invocation |
| Art 15 | Accuracy + robustness + cybersecurity | Eval harness + red team + Trivy | per-capability + quarterly |

### 3.8 + Continuing per-regulator deep-dives

> **TODO v0.2** — fill the remaining regulators with the same control × evidence × cadence shape. Pattern is repeatable; the matrix expands as packs onboard.

---

## 4. Evidence portal

All evidence flows into `trust.oyatie.com` (per [DOCUMENTATION.md §3](DOCUMENTATION.md)). The portal:

- Per-regulator evidence pack download (filtered by tenant access)
- Auditor self-serve evidence regeneration (≤ 4 hours per PRD §4.1 metric)
- Continuous control monitoring per Issue #954
- Per-control test execution dashboards
- Public attestation summaries (SOC 2, ISO, ISMS-P) — full reports gated to authenticated auditors

---

## 5. Continuous compliance monitoring (per ADR-0050)

| Surface | What's continuously evidenced | Where |
|---|---|---|
| Per-capability invocation | Audit-chain emission with regulatory_packs_consumed | `platform-audit-chain` |
| Per-tenant DSR queue | Cascade SLA + proof-of-erasure | DSR dashboard |
| Per-pack regulatory drift | Per-pack regulator-watch lane | `regional-packs/<pack>/regulatory-watch.log` |
| Per-control test | Continuous control monitoring per #954 | trust.oyatie.com |
| Per-incident notification window | 24-72 hour gate per regulator | INCIDENT-MANAGEMENT |
| Per-vendor risk | Quarterly review per VENDOR-PARTNER-LEDGER | vendor portal |

---

## 6. Open questions

1. **Defense / drone scope** (per rename agent's user-input #1) — keep out of compliance scope or open a `pack-defense` separate vehicle?
2. **EU AI Act risk classification** per capability — who classifies (Foundry team or per-axis team)?
3. **Continuous control monitoring tool** — buy (Drata / Vanta / Secureframe) or build (`trust-portal`)? Recommendation: build (consistent with in-house preference per [TOOLCHAIN.md §3](TOOLCHAIN.md)).
4. **Per-state US compliance** (NY SHIELD, IL BIPA, TX TDPSA, CA CPRA) — how to handle 50-state overlay efficiently?
5. **Cross-regulator evidence reuse** — is there a meta-control framework (ISACA / SCF) that lets us emit once and map to many?

---

## 7. Sources scanned

- All consolidated docs at `docs/`
- ADRs 0028, 0111, 0125, 0131, 0132, 0140, 0156, 0157, 0161, 0162, 0186, 0188, 0190, 0205, 0225, 0228, 0230, 0231, 0232
- KR statutes: PIPA, KISA guidance, MFDS guidance, FSC guidance, KCC guidance, NIS guidance
- US: HIPAA, PCI-DSS v4.0, SOC 2 TSC, NIST CSF + 800-53, FedRAMP
- EU: GDPR, DORA, AI Act, NIS2, eIDAS, GAIA-X
- Cross: ISO 27001/27017/27018/27701, CSA STAR
- `/Users/jasonlee/oyatie/docs/raw/greenfield-cloud.md` §J KR-launch leaves
- `/Users/jasonlee/oyatie/docs/raw/greenfield-search.md` §H+L safety/KR
- `/Users/jasonlee/oyatie/docs/raw/greenfield-ads-analytics.md` §G Data Use Boundary + §H+K ads policy

*Footer regenerated whenever this doc is edited.*
