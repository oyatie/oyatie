---
doc_class: DPIA
template_id: TPL-DPIA
microservice: payments
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-20
owner_team: council-privacy + axis-payments + ops-compliance + dpo
deciders: council-privacy, axis-payments, ops-compliance, dpo, council-finance
methodology: GDPR Article 35 (EU) + KR-PIPA Article 33 (KR) + UK GDPR + LGPD Art. 38 + Brazil DPIA + ICO DPIA template
related_adrs:
  - ADR-0244
  - ADR-0246
  - ADR-0251
  - ADR-0272
  - ADR-0273
  - ADR-0276
  - ADR-0292
companion_docs:
  - microservices/payments/PRD.md
  - microservices/payments/ARCHITECTURE.md
  - microservices/payments/threat-model.md
  - microservices/payments/compliance.md
review_cadence: annual + on any new data-class / new region / new processor
diataxis_quadrant: explanation
doc_status: published
---

# DPIA — payments µservice

> Data Protection Impact Assessment per GDPR Art. 35 + KR-PIPA Art. 33 + LGPD Art. 38 + ICO DPIA template. Annual review + on any new region / processor / data-class.

---

## §1. Identify the need for a DPIA (Art. 35(3) triggers)

| GDPR Art. 35(3) trigger | Applies to payments? | Reason |
|---|---|---|
| (a) Systematic + extensive evaluation incl. profiling | Yes | Fraud-scoring + dispute-risk profiling per charge. |
| (b) Large-scale processing of special-category data | Partial | Bank-account-holder names are personal data; not Art. 9 special-category. |
| (c) Systematic monitoring of publicly accessible area | No | Payments processes private transactions. |
| KR-PIPA Art. 33(1) — sensitive-info handling | No | Payments stores tokens, not raw card-data (PSP-tokenised). |
| LGPD Art. 38 — high-risk processing | Yes | Financial profiling at scale. |

DPIA is **required** per GDPR Art. 35(3)(a) and LGPD Art. 38.

## §2. Describe the processing

### 2.1 Nature of processing

| Operation | Personal data | Purpose | Legal basis (GDPR Art. 6) |
|---|---|---|---|
| Charge authorisation | Tokenised card-id, billing address (country + ZIP for AVS), name, email, IP, behavioural fingerprint | Authorise a payment instrument with the PSP | Art. 6(1)(b) contract performance |
| Refund | Charge-id, refund-reason | Reverse a transaction | Art. 6(1)(b) contract performance |
| Payout | Bank-account-holder name, IBAN / bank-code, address, tax-id (where applicable) | Disburse funds to merchant / sub-merchant / creator | Art. 6(1)(b) contract performance |
| Sub-merchant KYC | Document hashes (NOT raw documents — PSP holds raw), legal-entity name, beneficial-owner names + DoB + address (for AML) | KYC / KYB compliance | Art. 6(1)(c) legal obligation + Art. 9(2)(f) AML carve-out |
| Dispute evidence | IP-address-at-charge, browser-fingerprint-at-charge, shipping-address, photo-of-product, order-history | Defend a chargeback | Art. 6(1)(f) legitimate interest (defending a transaction) |
| Subscription billing | Customer-id, payment-method-id, billing-cycle | Recurring payment | Art. 6(1)(b) contract performance |
| Fraud-scoring | Behavioural fingerprint, JA4+ fingerprint, transaction-pattern features | Fraud detection | Art. 6(1)(f) legitimate interest |
| Audit-trail | Principal SVID, action, timestamp, tenant_id | Regulatory + dispute audit | Art. 6(1)(c) legal obligation |

### 2.2 Scope of processing

- **Data subjects**: end-consumers (B2C-personal), tenant operators (B2B-work), sub-merchants (creators / sellers / app-developers), partner agencies acting on behalf of multiple tenants.
- **Volume estimate at GA**: ~1M consumer transactions/day at launch → ~50M/day at year-3 (per [`capacity-model.md`](capacity-model.md)).
- **Geographic scope**: every region with a deployed cell — EU, US, KR, JP, SG, AU, IN, BR, AE, KSA, CN (CN-PIPL data-residency constrained).
- **Sources**: data subject directly (via tenant surface SDKs); PSP responses; KYC document-verification providers.

### 2.3 Retention

| Data class | Retention | Why |
|---|---|---|
| Tokenised card-id (PSP-tokenised) | 7 years post-last-transaction | Tax + KR-FSS audit requirements |
| Bank account number for payout | Until subject revokes + audit window expires (7y for KR-FSS / 10y for KR-EFTA) | Regulatory |
| Sub-merchant KYC document hashes | 5 years post-account-closure | AML / CTF |
| Dispute evidence | 13 months post-resolution (typical card-network chargeback window + grace) | Card-network rules |
| Fraud-scoring features (aggregated) | 18 months | Detection-effectiveness |
| Audit-trail | 7 years (KR-FSS) / 10 years (KR-EFTA) | Regulatory |

### 2.4 Processors + sub-processors

| Processor | Role | Region | DPA / SCC |
|---|---|---|---|
| Stripe | PSP | US-base + per-region | DPA signed; SCCs for cross-region |
| Adyen | PSP | EU + global | DPA signed; SCCs for cross-region |
| Toss Payments | PSP | KR | DPA signed; KR-PIPA data-processor agreement |
| KakaoPay | PSP | KR | DPA signed |
| LINE Pay | PSP | JP / TW / TH | DPA signed |
| WeChat Pay | PSP | CN | DPA signed; CN-PIPL cross-border-no |
| Alipay | PSP | CN / global | DPA signed; CN-PIPL cross-border-no |
| Sift / similar fraud-ML | Fraud-scoring assist (post-MVP) | US | DPA + SCCs |
| Stripe Identity / equivalent | KYC verifier | per-PSP | inherits the PSP DPA |

## §3. Necessity + proportionality

| Question | Answer |
|---|---|
| Is the processing necessary to achieve the purpose? | Yes — payment processing requires the data classes listed. |
| Could the purpose be achieved with less data? | Where possible we tokenise (PAN never touches us). Email is optional; if collected, declared in consent. |
| Are individuals informed? | Yes — per-tenant privacy notice + per-purpose consent per ADR-0272. |
| Are individuals' rights respected? | Yes — see §5. |

## §4. Consult interested parties

| Party | When | Notes |
|---|---|---|
| Data subjects | Per-purpose consent at checkout per ADR-0272 | "Process payment", "Fraud detection", "Regulatory audit" purposes presented separately |
| Tenant operators | Privacy DPA at tenant onboarding | Tenant attests they have lawful basis to share consumer data |
| DPO | DPIA review | Annual + on material change |
| ICO / KR-PIPC / EDPB as needed | Prior-consultation triggers | Only if residual risk > High after mitigation |

## §5. Data-subject rights

| Right (GDPR Arts. 12-22) | How payments fulfils |
|---|---|
| Art. 15 — right of access | Subject calls tenant's privacy portal → tenant calls payments via `Subject::Access` Cedar permit → payments returns the subject's charges / refunds / payouts subset |
| Art. 16 — right to rectification | Limited to bank-account / billing-address fields; PSP-side data follows PSP's rectification process |
| Art. 17 — right to erasure | **Limited** by Art. 17(3)(b) legal obligation + (e) defence of legal claims — financial records retained per §2.3 retention table; subject is informed |
| Art. 18 — right to restriction | Supported — `payments.processing.restricted` flag on subject |
| Art. 20 — right to portability | Per ADR-0276 portability format — subject's charges / refunds / payouts exported in JSON |
| Art. 21 — right to object | Marketing-attribution profiling can be objected to; fraud-scoring is legitimate-interest with override threshold |
| Art. 22 — right re. automated decision-making | Fraud-scoring is automated; subject can request human review on decline |

## §6. Risk assessment

| # | Risk | Likelihood | Severity | Pre-mitigation | Post-mitigation |
|---:|---|:--:|:--:|---|---|
| R1 | PAN leakage in logs / traces | Medium | Catastrophic | High | Low (PSP-tokenised; redaction lint; PCI scope SAQ-A) |
| R2 | Cross-tenant disclosure of payout balances | Low | High | Medium | Low (Cedar default-deny + row-level scoping) |
| R3 | Subject-access request leaks more than the subject is entitled to | Low | High | Medium | Low (per-subject Cedar scope) |
| R4 | Sub-processor (PSP) data-residency violation | Low | High | Medium | Low (per-PSP region pinning; CN-PIPL → CN-cell-only) |
| R5 | Fraud-scoring profiling without subject knowledge | Medium | Medium | Medium | Low (per-purpose consent + transparency) |
| R6 | Dispute-evidence bundle over-shares PII | Medium | High | High | Low (PII-minimisation lint per `docs/standards/dispute-representment-minimisation.md`) |
| R7 | Bank-account / KYB document leak | Low | Catastrophic | Critical | Low (object-storage SSE-KMS-tenant; OpenBao for raw secrets; never in-process beyond TTL) |
| R8 | Retention drift (data kept beyond §2.3) | Medium | Medium | Medium | Low (CronJob purges past-retention rows; quarterly retention-audit) |
| R9 | Cross-border data transfer without SCC | Low | High | Medium | Low (SCCs signed for every cross-region PSP; CN-PIPL no-transfer for CN flows) |
| R10 | Minor (<13 / 14-17) makes payment | Medium | High (COPPA penalty) | High | Low (refuse <13 per ADR-0292; KOSA / EU-U18 restrictions) |

Residual risk after mitigation: **Acceptable**. No GDPR Art. 36 prior-consultation triggers.

## §7. Pack-overlay considerations

### EU (pack-eu-psd2-sca)

- SCA (PSD2 RTS) — required for EU subjects at threshold; dynamic-linking; step-up flow per [`compliance.md`](compliance.md).
- GDPR + ePrivacy — consent + transparency per ADR-0272.
- e-money licence — per-EU-jurisdiction; oyatie operates as facilitator only.

### KR (pack-kr-fss)

- KR-PIPA Arts. 23 / 24 / 29-2 — sensitive-data + unique-identifier + breach-notification.
- KR-EFTA Art. 21-3 — security duty for e-finance operators; payments inherits via Toss / KakaoPay DPA.
- 10-year retention for e-financial records.

### US (pack-us-state-mtl + pack-ccpa-cpra-2023)

- CCPA + CPRA — right-to-know, right-to-delete (subject to financial-record exemption), right-to-opt-out-of-sale; payments never sells data.
- Per-US-state money-transmitter requirements — registration + reporting; oyatie operates as a facilitator above each PSP's MTL.

### CN (pack-cn-pipl-2021)

- PIPL Arts. 38-43 — cross-border transfer restrictions; WeChat Pay / Alipay flows are CN-cell-only with NO cross-border data egress.
- PBoC payment regs — per-CN-licence requirements.

### BR (pack-br-lgpd-finance)

- LGPD Arts. 6 / 11 / 14 / 18 / 33 — base lawful basis + subject rights.
- BACEN Res. 4.893/2021 — cybersecurity + outsourcing risk.

### Minor-protection (per ADR-0292)

- **<13 (COPPA / EU U-13)**: refuse all payments. Audit event `oya.payments.minor.refused-coppa`.
- **14-17 (KOSA / EU U-18)**: allow subset (no recurring; no >$50; parental-consent flag required).
- **18+**: no payments-side age restriction.

## §8. Sign-off

| Role | Owner | Sign-off date | Notes |
|---|---|---|---|
| DPO | tbd | (pending — review pre-M02-exit) | |
| council-privacy | yes | 2026-05-20 | Initial publication |
| axis-payments | yes | 2026-05-20 | |
| ops-compliance | yes | 2026-05-20 | |

## §9. References

- [GDPR Art. 35 — DPIA](https://gdpr-info.eu/art-35-gdpr/).
- [KR-PIPA Art. 33 — Impact assessment](https://www.law.go.kr).
- [LGPD Art. 38](https://www.gov.br/anpd).
- [ICO DPIA template](https://ico.org.uk/for-organisations/guide-to-data-protection/guide-to-the-general-data-protection-regulation-gdpr/data-protection-impact-assessments-dpias/).
- [`threat-model.md`](threat-model.md).
- [`compliance.md`](compliance.md).
- [ADR-0272 — cookie consent per-purpose](../../docs/decisions/ADR-0272-cookie-consent-per-purpose.md).
- [ADR-0276 — backup portability GDPR Art. 20](../../docs/decisions/ADR-0276-backup-portability-gdpr-art-20.md).
- [ADR-0292 — minor user doctrine](../../docs/decisions/ADR-0292-minor-user-doctrine.md).
