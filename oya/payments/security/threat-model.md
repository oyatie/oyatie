---
doc_class: ThreatModel
microservice: payments
version: 1.0.0
status: Proposed
date: 2026-05-20
owner: axis-payments + ops-fraud + council-security
related_oyatie_adrs:
  - ADR-0003
  - ADR-0009
  - ADR-0145
  - ADR-0243
  - ADR-0244
  - ADR-0263
  - ADR-0297
  - ADR-0313
  - ADR-0319
---

# Payments Security Threat Model

This document covers the payments substrate security posture for charge,
refund, payout, dispute, subscription, PSP adapter, KYC/KYB, sub-merchant,
settlement, and webhook surfaces. Payments is a Tier-1 financial substrate:
control failures can create PCI-DSS exposure, fraudulent money movement,
chargeback loss, sanctions/AML exposure, and audit-chain disputes across every
tenant that monetizes through Oyatie.

## Asset Inventory

### Named Data Classes

| Asset ID | Named data class | Description | Primary store | Security objective |
|---|---|---|---|---|
| PAY-A01 | PspTokenReference | PSP token, payment method id, customer id, and card brand metadata; PAN must not be stored. | PSP and payments DB reference | Keep Oyatie outside raw card-data exfiltration scope. |
| PAY-A02 | ChargeAuthorizationRecord | Amount, currency, merchant, authorization status, idempotency key, risk decision. | Payments ledger DB | Prevent forged or duplicated charges. |
| PAY-A03 | RefundRecord | Refund request, reason, approver, PSP refund id, reversal status. | Payments ledger DB | Prevent refund abuse. |
| PAY-A04 | PayoutRecord | Seller/merchant payout, beneficiary, settlement schedule, bank token reference. | Payments ledger DB | Prevent mule payouts and beneficiary tamper. |
| PAY-A05 | DisputeEvidenceBundle | Representment files, customer communication, delivery proof, escalation state. | Object storage and dispute DB | Prevent chargeback fraud and dispute tamper. |
| PAY-A06 | SubMerchantKycProfile | KYB/KYC identity, sanctions screening, ownership, risk tier, restriction state. | KYC/KYB store | Prevent mule accounts and laundering. |
| PAY-A07 | ThreeDsChallengeState | 3DS authentication result, ECI, liability shift, challenge status. | Payments DB | Prevent 3DS fail-open and forged SCA. |
| PAY-A08 | BinAttackSignal | BIN range, velocity, decline codes, issuer region, device fingerprint. | Fraud telemetry and audit-chain | Detect card testing and BIN attacks. |
| PAY-A09 | PspCredentialSecret | Stripe/Adyen/Toss or other PSP credentials, webhook signing secret, API key. | OpenBao | Prevent PSP credential theft. |
| PAY-A10 | WebhookEventRecord | PSP webhook payload, signature status, idempotency key, event timestamp. | Webhook DB and audit-chain | Prevent spoofed or replayed PSP updates. |
| PAY-A11 | SettlementReconciliationRecord | PSP settlement file, internal ledger postings, FX, fees, variance. | Settlement store | Prevent financial misstatement. |
| PAY-A12 | AbuseDecisionRecord | Cedar fraud/abuse decision, bot score, risk score, manual review state. | Policy log and audit-chain | Prevent fraud-control bypass. |
| PAY-A13 | AuditEmissionEnvelope | ADR-0263 envelope with tenant_id, trace_id, span_id, audit_id, schema_version, source_microservice. | audit-chain | Preserve non-repudiation and incident joins. |

### Named External Interfaces

| Interface ID | Interface | Entry point | Principal | Notes |
|---|---|---|---|---|
| PAY-I01 | Charge API | `../contracts/openapi-v1.yaml` | Tenant app or user | Authorizes and captures charges through PSP. |
| PAY-I02 | Refund API | `../contracts/openapi-v1.yaml` | Tenant operator or automation | Issues refund with Cedar policy. |
| PAY-I03 | Payout API | `../contracts/openapi-v1.yaml` | Tenant operator or settlement worker | Moves funds to sub-merchant or seller. |
| PAY-I04 | Dispute API | `../contracts/openapi-v1.yaml` | Tenant operator, fraud role, PSP webhook | Manages chargeback and representment. |
| PAY-I05 | PSP Webhook | `../contracts/asyncapi-v1.yaml` | Stripe/Adyen/Toss/etc. | Public inbound signed event path. |
| PAY-I06 | PSP Adapter | `../contracts/psp-adapter-trait.md` | Payments service | Calls external PSP APIs. |
| PAY-I07 | 3DS/SCA Challenge | `../capabilities/charge.yaml` | Browser, issuer ACS, PSP | Confirms liability shift and SCA. |
| PAY-I08 | KYC/KYB Onboarding | `../capabilities/sub-merchant-onboarding.yaml` | Tenant or sub-merchant | Collects identity and risk data. |
| PAY-I09 | Dispute Evidence Upload | `../runbooks/dispute-escalation.md` | Tenant operator | Stores evidence bundle. |
| PAY-I10 | Audit Event Bridge | `../contracts/asyncapi-v1.yaml` | Payments service | Emits charge/refund/payout/dispute events. |

### Named Dependencies

| Dependency ID | Dependency | Use | Failure impact | Guardrail |
|---|---|---|---|---|
| PAY-D01 | PSP providers | Card tokenization, auth, capture, refund, payout | Payment outage or PSP truth divergence | `../runbooks/psp-outage.md`. |
| PAY-D02 | OpenBao | PSP credentials and webhook signing secrets | PSP credential compromise | `../iac/openbao/payments-policy.hcl`. |
| PAY-D03 | Cedar policy-engine | Charge, refund, dispute, payout, onboarding authorization | Broken financial access control | `../policy/charge-authorization.cedar`. |
| PAY-D04 | Fraud/risk rules | BIN, mule, velocity, chargeback risk | Fraud loss or false positives | `../dashboards/fraud-signals.md`. |
| PAY-D05 | Ledger/CRDB | Charge/refund/payout state | Financial integrity failure | `../runbooks/refund-mismatch.md`. |
| PAY-D06 | Object storage | Dispute evidence and KYC documents | Evidence leak or tamper | Evidence bundle sealing. |
| PAY-D07 | audit-chain | Financial event sealing | Repudiation and audit failure | ADR-0003 and ADR-0263. |
| PAY-D08 | identity | Operator and tenant authentication | Unauthorized financial mutation | Step-up and tenant scope. |
| PAY-D09 | sanctions/AML providers | Screening and monitoring | Mule accounts or regulatory failure | `../runbooks/aml-suspicious-activity-detected.md`. |
| PAY-D10 | observability | Fraud, PSP, reconciliation detection | Missed loss signals | `../dashboards/payments-overview.json`. |

## Trust Boundaries

| Boundary ID | Named boundary | Crosses from | Crosses to | Primary concern |
|---|---|---|---|---|
| PAY-B01 | Public payment API boundary | Browser, tenant app, SDK | Payments ingress | Hostile input, BIN testing, idempotency abuse. |
| PAY-B02 | PCI tokenization boundary | Browser/PSP hosted fields | PSP token reference | Raw PAN must not enter Oyatie systems. |
| PAY-B03 | Tenant boundary | Tenant A financial objects | Tenant B financial objects | Cross-tenant financial disclosure or mutation. |
| PAY-B04 | PSP adapter boundary | Payments service | External PSP API | Credential theft, response tamper, provider outage. |
| PAY-B05 | PSP webhook boundary | External PSP webhook | Webhook handler | Spoofed or replayed payment events. |
| PAY-B06 | OpenBao credential boundary | Payments adapter | PSP credential secret path | PSP key custody compromise. |
| PAY-B07 | Fraud/risk boundary | Charge/payout request | Risk engine and Cedar policies | Fraud decision bypass. |
| PAY-B08 | 3DS/SCA boundary | Issuer/ACS challenge | Charge flow | 3DS fail-open or forged liability shift. |
| PAY-B09 | KYC/KYB boundary | Sub-merchant applicant | Onboarding and screening pipeline | Mule accounts and sanctions evasion. |
| PAY-B10 | Dispute evidence boundary | Tenant evidence upload | Dispute object store and workflow | Dispute escalation tampering. |
| PAY-B11 | Settlement boundary | PSP settlement reports | Internal ledger reconciliation | Financial misstatement. |
| PAY-B12 | Audit boundary | Payments state change | audit-chain emission bridge | Missing audit_id or wrong tenant_id. |
| PAY-B13 | Information-barrier boundary | Front/middle/back-office role | Payment records and decisions | Improper office-scope access. |

## STRIDE Analysis

### Spoofing

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| PAY-S01 | WebhookEventRecord | PAY-B05 | Attacker sends spoofed PSP webhook for charge/refund/dispute state. | Ledger corruption. |
| PAY-S02 | PspTokenReference | PAY-B02 | Fake or stolen payment method token is presented as valid. | Fraudulent charge attempt. |
| PAY-S03 | SubMerchantKycProfile | PAY-B09 | Mule account applicant spoofs identity or ownership. | AML and payout risk. |
| PAY-S04 | PayoutRecord | PAY-B03 | Caller spoofs tenant or sub-merchant beneficiary. | Unauthorized funds movement. |
| PAY-S05 | ThreeDsChallengeState | PAY-B08 | Forged 3DS result claims liability shift. | SCA bypass and chargeback exposure. |
| PAY-S06 | PspCredentialSecret | PAY-B06 | Workload spoofs adapter identity to obtain PSP key. | PSP credential compromise. |
| PAY-S07 | AuditEmissionEnvelope | PAY-B12 | Event emitted as wrong tenant or source_microservice. | Financial forensic gap. |

### Tampering

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| PAY-T01 | ChargeAuthorizationRecord | PAY-B01 | Amount, currency, or merchant id is changed after client confirmation. | Overcharge or undercharge. |
| PAY-T02 | RefundRecord | PAY-B03 | Refund amount or approver is altered. | Refund theft. |
| PAY-T03 | PayoutRecord | PAY-B09 | Beneficiary or payout schedule is altered. | Mule payout. |
| PAY-T04 | DisputeEvidenceBundle | PAY-B10 | Evidence is changed after submission or escalation. | Chargeback loss and audit failure. |
| PAY-T05 | ThreeDsChallengeState | PAY-B08 | 3DS challenge failure is stored as success. | Fail-open SCA. |
| PAY-T06 | WebhookEventRecord | PAY-B05 | PSP event idempotency or timestamp is modified. | Replay or duplicate mutation. |
| PAY-T07 | SettlementReconciliationRecord | PAY-B11 | PSP settlement variance is hidden or rewritten. | Financial misstatement. |
| PAY-T08 | AbuseDecisionRecord | PAY-B07 | Fraud/risk decision is downgraded or removed. | Fraud control bypass. |

### Repudiation

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| PAY-R01 | ChargeAuthorizationRecord | PAY-B01 | Customer or tenant denies charge initiation. | Dispute handling gap. |
| PAY-R02 | RefundRecord | PAY-B03 | Operator denies approving refund. | Fraud investigation gap. |
| PAY-R03 | PayoutRecord | PAY-B09 | Sub-merchant denies payout beneficiary update. | Funds recovery gap. |
| PAY-R04 | DisputeEvidenceBundle | PAY-B10 | Tenant denies altering dispute evidence. | Chargeback loss. |
| PAY-R05 | WebhookEventRecord | PAY-B05 | PSP event cannot be tied to verified signature and audit_id. | PSP truth ambiguity. |
| PAY-R06 | SettlementReconciliationRecord | PAY-B11 | Finance cannot prove reconciliation variance handling. | Audit finding. |

### Information Disclosure

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| PAY-I01 | PspTokenReference | PAY-B02 | Raw PAN or CVV enters Oyatie logs or DB, expanding PCI scope. | Card-data exfiltration and PCI-DSS breach. |
| PAY-I02 | PspCredentialSecret | PAY-B06 | PSP API key or webhook secret leaks. | Unauthorized PSP actions. |
| PAY-I03 | SubMerchantKycProfile | PAY-B09 | KYC documents are exposed to wrong tenant or role. | Identity and regulatory breach. |
| PAY-I04 | DisputeEvidenceBundle | PAY-B10 | Evidence bundle exposes customer PII or card artifacts. | Privacy breach. |
| PAY-I05 | SettlementReconciliationRecord | PAY-B11 | Settlement files leak fees, balances, bank refs. | Financial disclosure. |
| PAY-I06 | AuditEmissionEnvelope | PAY-B12 | ADR-0263 telemetry includes PAN, CVV, bank account, or raw evidence. | Observability privacy and PCI breach. |

### Denial of Service

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| PAY-DOS01 | ChargeAuthorizationRecord | PAY-B01 | BIN attack or card testing floods auth path. | PSP cost, issuer decline spike, tenant checkout outage. |
| PAY-DOS02 | WebhookEventRecord | PAY-B05 | Webhook flood exhausts handler and idempotency store. | State lag and duplicate risk. |
| PAY-DOS03 | PspCredentialSecret | PAY-B06 | OpenBao latency blocks PSP calls. | Payment outage. |
| PAY-DOS04 | PspTokenReference | PAY-B04 | PSP outage or rate limit blocks charge/refund. | Checkout and refund outage. |
| PAY-DOS05 | DisputeEvidenceBundle | PAY-B10 | Evidence upload flood exhausts object storage. | Dispute deadline miss. |
| PAY-DOS06 | SettlementReconciliationRecord | PAY-B11 | Large settlement file or reconciliation retry storm. | Finance close delay. |

### Elevation of Privilege

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| PAY-E01 | RefundRecord | PAY-B03 | Operator self-approves high-value refund. | Financial loss. |
| PAY-E02 | PayoutRecord | PAY-B09 | User escalates to payout admin or changes bank token. | Funds diversion. |
| PAY-E03 | DisputeEvidenceBundle | PAY-B10 | User escalates dispute state or deletes adverse evidence. | Chargeback fraud. |
| PAY-E04 | ThreeDsChallengeState | PAY-B08 | Tenant disables 3DS or SCA for high-risk charge. | Regulatory and chargeback exposure. |
| PAY-E05 | SubMerchantKycProfile | PAY-B09 | Restricted sub-merchant reactivates itself. | Mule account. |
| PAY-E06 | SettlementReconciliationRecord | PAY-B11 | Finance-read role mutates ledger reconciliation. | Financial misstatement. |

## DREAD Scoring

| Rank | Threat ID | Threat | Damage | Reproducibility | Exploitability | Affected users | Discoverability | Total |
|---|---|---|---:|---:|---:|---:|---:|---:|
| 1 | PAY-I01 | Raw card-data exfiltration or PCI scope breach. | 10 | 8 | 7 | 10 | 8 | 43 |
| 2 | PAY-S01 | Spoofed PSP webhook corrupts ledger state. | 10 | 8 | 8 | 9 | 7 | 42 |
| 3 | PAY-E02 | Payout beneficiary or admin escalation diverts funds. | 10 | 7 | 7 | 9 | 7 | 40 |
| 4 | PAY-S03 | Mule account passes onboarding. | 10 | 8 | 7 | 8 | 7 | 40 |
| 5 | PAY-DOS01 | BIN attack/card testing flood. | 8 | 10 | 9 | 8 | 5 | 40 |
| 6 | PAY-T05 | 3DS failure stored as success. | 9 | 7 | 7 | 8 | 7 | 38 |
| 7 | PAY-T04 | Dispute evidence tampered. | 8 | 8 | 7 | 8 | 6 | 37 |
| 8 | PAY-I02 | PSP credential leak. | 10 | 6 | 6 | 9 | 6 | 37 |
| 9 | PAY-T07 | Settlement reconciliation tamper. | 9 | 6 | 6 | 8 | 7 | 36 |
| 10 | PAY-E01 | Self-approved refund. | 8 | 7 | 7 | 7 | 6 | 35 |
| 11 | PAY-DOS04 | PSP outage blocks charge/refund. | 8 | 8 | 5 | 9 | 5 | 35 |
| 12 | PAY-S05 | Forged 3DS liability shift. | 8 | 7 | 6 | 7 | 6 | 34 |
| 13 | PAY-R05 | PSP event lacks verified signature and audit_id. | 8 | 7 | 6 | 7 | 6 | 34 |
| 14 | PAY-I03 | KYC profile leak. | 8 | 6 | 6 | 8 | 5 | 33 |
| 15 | PAY-DOS05 | Dispute evidence upload flood misses deadline. | 7 | 7 | 7 | 7 | 4 | 32 |

## Attack Trees

### Opportunistic Adversary: BIN Attack

- Goal: validate stolen card numbers and find accepted BIN ranges.
  - Path O1: submit many low-value authorization attempts.
  - Path O2: rotate device fingerprint and source IP.
  - Path O3: exploit weak idempotency key handling to avoid duplicate detection.
  - Path O4: detect issuer response timing and decline code differences.
  - Path O5: sell validated card data or use successful token for larger charge.
- Required break: ADR-0297 rate limits and fraud velocity controls fail.
- Required break: `BinAttackSignal` is not correlated with PSP declines.
- Detection pivot: `ChargeDeclined`, `ChargeErrored`, `AbuseDefenceRateLimitHit`, `AbuseDefenceQuotaExceeded`.

### Targeted Adversary: Dispute Escalation Tampering

- Goal: win chargeback fraud or hide adverse evidence.
  - Path T1: open dispute or wait for PSP dispute webhook.
  - Path T2: upload curated evidence bundle.
  - Path T3: alter evidence after submission or before escalation deadline.
  - Path T4: escalate dispute state without fraud-review authority.
  - Path T5: suppress chain-of-custody audit.
- Required break: object hash and audit-chain seal missing on evidence.
- Required break: dispute Cedar policy allows self-approval.
- Detection pivot: `DisputeOpened`, `DisputeEvidenceSubmitted`, `DisputeResolved`, `OfficeBoundaryAttemptDenied`.

### Insider Adversary: Mule Payout Route

- Goal: move tenant or platform funds to mule account.
  - Path I1: create or compromise sub-merchant profile.
  - Path I2: bypass KYB/sanctions screening or alter beneficial owner.
  - Path I3: change payout beneficiary or schedule.
  - Path I4: approve payout without four-eyes control.
  - Path I5: reconcile settlement variance as expected fee.
- Required break: KYC/KYB restriction and payout Cedar policy fail.
- Required break: settlement reconciliation does not alert on variance.
- Detection pivot: `SubMerchantOnboarded`, `SubMerchantRestricted`, `PayoutInitiated`, `PayoutFailed`.

### Nation-State Adversary: PSP Credential and PCI Expansion

- Goal: steal PSP credentials or force card data into Oyatie-controlled systems.
  - Path N1: compromise adapter pod or OpenBao policy.
  - Path N2: retrieve PSP key or webhook signing secret.
  - Path N3: manipulate hosted fields integration to send PAN to Oyatie endpoint.
  - Path N4: issue unauthorized PSP calls or exfiltrate token mapping.
  - Path N5: hide traces by tampering telemetry or audit_id.
- Required break: PAN/CVV scrubber and input contract fail.
- Required break: adapter identity is not tenant/path constrained.
- Detection pivot: PII scrubber failure, OpenBao audit, `AbuseDefenceAttestationFailed`, PSP anomaly.

## Mitigations Currently In Place

| Threat ID | Named mitigation | ADR or policy | Named code path or doc |
|---|---|---|---|
| PAY-I01 | PSP tokenization boundary; raw PAN/CVV forbidden in request logs and DB. | ADR-0263 | `../contracts/openapi-v1.yaml`; `../runbooks/pci-incident-response.md`. |
| PAY-S01 | HMAC verification, replay window, idempotency key, PSP event id. | ADR-0145 | `../contracts/asyncapi-v1.yaml`; `../runbooks/psp-failover-cascade-execution.md`. |
| PAY-DOS01 | BIN velocity, per-tenant and per-device rate limits, decline pattern detection. | ADR-0297 | `../dashboards/fraud-signals.md`; `../policy/abuse-defence.cedar`. |
| PAY-S03 | KYB/KYC, sanctions, risk tier, sub-merchant restriction. | ADR-0243 | `../policy/sub-merchant-onboarding.cedar`; `../runbooks/kyc-aml-screening-pipeline-stall.md`. |
| PAY-E02 | Payout authorization Cedar policy and beneficiary change audit. | ADR-0244 | `../policy/payout-authorization.cedar`; `../runbooks/payout-failed.md`. |
| PAY-T05 | 3DS result binding to charge and fail-closed policy for high risk. | ADR-0243 | `../policy/charge-authorization.cedar`; `../capabilities/charge.yaml`. |
| PAY-T04 | Dispute evidence hashing and chain-of-custody seal. | ADR-0003 | `../policy/dispute-authorization.cedar`; `../runbooks/dispute-escalation.md`. |
| PAY-E01 | Refund authorization policy, value threshold, and four-eyes approval. | ADR-0243 | `../policy/refund-authorization.cedar`; `../runbooks/refund-mismatch.md`. |
| PAY-T07 | Settlement reconciliation dashboard and variance investigation. | ADR-0263 | `../dashboards/settlement-reconciliation.json`. |
| PAY-I02 | PSP credentials stored in OpenBao and accessed through scoped sidecar. | ADR-0243 | `../iac/openbao/payments-policy.hcl`. |
| PAY-DOS04 | PSP outage runbooks and adapter failover. | ADR-0145 | `../runbooks/psp-outage.md`; `../decisions/ADR-PAY-001-multi-psp-routing-with-failover-cascade.md`. |
| PAY-E06 | Finance-read role cannot mutate reconciliation state. | ADR-0319 | `../policy/auditor-scope.cedar`. |

## Residual Risks Accepted

| Risk ID | Residual risk | Risk owner | Compensating control | Review trigger |
|---|---|---|---|---|
| PAY-RR01 | PSP-hosted fields still depend on correct front-end integration. | axis-payments | PCI fixture, PII scrubber, and canary PAN rejection. | Checkout UI change. |
| PAY-RR02 | BIN attacks can impose PSP cost before blocking catches up. | ops-fraud | Dynamic rate limits and issuer decline correlation. | BIN velocity spike. |
| PAY-RR03 | Mule accounts can pass initial KYB with high-quality stolen identity. | ops-fraud | Ongoing transaction monitoring and payout holds. | AML alert. |
| PAY-RR04 | 3DS providers can be unavailable or ambiguous. | axis-payments | Fail-closed for high risk and documented low-risk exemption. | 3DS outage. |
| PAY-RR05 | Chargeback evidence may contain customer PII. | council-privacy | Evidence minimization and sealed access trail. | Dispute export. |
| PAY-RR06 | PSP webhook semantics can change. | axis-payments | Contract tests and staged adapter rollout. | PSP API version change. |
| PAY-RR07 | Settlement reconciliation lag can create temporary financial uncertainty. | council-finance | Daily variance dashboard and manual hold. | Variance threshold. |
| PAY-RR08 | Fraud controls can false-positive legitimate tenants. | ops-fraud | Manual review queue and tenant appeal. | Fraud false-positive surge. |
| PAY-RR09 | Cross-border payout rules can change faster than policy packs. | ops-compliance | Jurisdiction pack review and payout freeze. | Regulatory update. |
| PAY-RR10 | Emergency-services bypass policy can conflict with payment holds. | council-security | Emergency bypass Cedar policy and post-event review. | Emergency bypass event. |

## Specific Telemetry for Detection

ADR-0263 detection telemetry must include `tenant_id`, `sub_scope_path`,
`event_id`, `trace_id`, `span_id`, `audit_id`, `schema_version`,
`source_microservice`, `cell_id`, and `jurisdiction_code` for state-changing
payments events. Cedar denial events include policy id, principal, action,
resource, decision, and denied reason.

| Threat ID | Detection telemetry | ADR-0263 class or service event | Signal |
|---|---|---|---|
| PAY-I01 | PAN/CVV pattern in request/log, hosted-field bypass, PCI scrubber failure. | `AbuseDefenceCanaryRecovered`, log-schema failure | Card-data exfiltration scope breach. |
| PAY-S01 | HMAC failure, replay timestamp, duplicate PSP event id. | `AbuseDefenceSpoofDetected`, payment webhook event | Spoofed PSP webhook. |
| PAY-DOS01 | BIN velocity, decline code spread, issuer-region burst. | `ChargeDeclined`, `AbuseDefenceRateLimitHit` | BIN attack or card testing. |
| PAY-S03 | KYB anomaly, sanctions hit, linked beneficiary reuse. | `SubMerchantOnboarded`, `SubMerchantRestricted` | Mule account. |
| PAY-E02 | Beneficiary update, payout schedule change, new bank token. | `PayoutScheduled`, `PayoutInitiated`, `OfficeBoundaryClearanceRequested` | Payout fraud. |
| PAY-T05 | 3DS fail status followed by capture, missing ECI, liability mismatch. | `ChargeAuthorized`, Cedar denied reason | 3DS fail-open. |
| PAY-T04 | Evidence hash mismatch, late evidence replacement, escalation override. | `DisputeEvidenceSubmitted`, `DisputeResolved` | Dispute tampering. |
| PAY-E01 | High-value refund self-approval or unusual refund ratio. | `RefundIssued`, `OfficeBoundaryAttemptDenied` | Refund abuse. |
| PAY-T07 | Settlement variance, missing PSP row, ledger mismatch. | reconciliation dashboard event | Settlement tamper or PSP drift. |
| PAY-I02 | PSP key read outside adapter, webhook secret read, OpenBao anomaly. | `AbuseDefenceAttestationFailed`, OpenBao audit | PSP credential theft. |
| PAY-DOS04 | PSP timeout, provider failover, adapter error surge. | `AbuseDefenceVendorOutage`, `ChargeErrored` | PSP outage or attack. |
| PAY-E06 | Finance role attempts mutation. | `OfficeBoundaryAttemptDenied`, Cedar deny | Reconciliation privilege escalation. |

## Threat Coverage Ledger

### PAY-COV01: PCI boundary coverage

- Threats covered: PAY-I01, PAY-S02.
- Asset coverage: PspTokenReference and AuditEmissionEnvelope.
- Boundary coverage: PAY-B02 and PAY-B12.
- Required control evidence: hosted field/tokenization path, PAN/CVV rejection, telemetry scrubber.
- Detection evidence: PII scrubber failure and PCI incident runbook trigger.

### PAY-COV02: PSP webhook coverage

- Threats covered: PAY-S01, PAY-T06, PAY-R05, PAY-DOS02.
- Asset coverage: WebhookEventRecord.
- Boundary coverage: PAY-B05.
- Required control evidence: HMAC verification, timestamp window, idempotency key, duplicate PSP event deny.
- Detection evidence: HMAC failure metric, `AbuseDefenceSpoofDetected`, and webhook replay alert.

### PAY-COV03: BIN attack coverage

- Threats covered: PAY-DOS01 and PAY-S02.
- Asset coverage: BinAttackSignal and ChargeAuthorizationRecord.
- Boundary coverage: PAY-B01 and PAY-B07.
- Required control evidence: velocity buckets, issuer decline spread, bot score, per-tenant limit.
- Detection evidence: `ChargeDeclined`, fraud-signals dashboard, and `AbuseDefenceRateLimitHit`.

### PAY-COV04: Mule account coverage

- Threats covered: PAY-S03, PAY-E05, PAY-E02.
- Asset coverage: SubMerchantKycProfile and PayoutRecord.
- Boundary coverage: PAY-B09.
- Required control evidence: KYB/KYC screening, sanctions check, beneficiary change review, payout hold.
- Detection evidence: `SubMerchantOnboarded`, `SubMerchantRestricted`, and AML suspicious activity alert.

### PAY-COV05: 3DS fail-open coverage

- Threats covered: PAY-S05, PAY-T05, PAY-E04.
- Asset coverage: ThreeDsChallengeState and ChargeAuthorizationRecord.
- Boundary coverage: PAY-B08.
- Required control evidence: ECI validation, challenge result binding, fail-closed high-risk rule.
- Detection evidence: charge authorization event plus Cedar denied reason.

### PAY-COV06: Dispute tamper coverage

- Threats covered: PAY-T04, PAY-E03, PAY-R04.
- Asset coverage: DisputeEvidenceBundle.
- Boundary coverage: PAY-B10.
- Required control evidence: evidence hash, immutable object, role separation, chain-of-custody seal.
- Detection evidence: `DisputeEvidenceSubmitted`, hash mismatch alert, and dispute escalation runbook.

### PAY-COV07: Refund abuse coverage

- Threats covered: PAY-T02, PAY-E01, PAY-R02.
- Asset coverage: RefundRecord.
- Boundary coverage: PAY-B03 and PAY-B12.
- Required control evidence: value threshold, four-eyes approval, refund policy, audit_id.
- Detection evidence: `RefundIssued`, `RefundFailed`, and office-boundary clearance events.

### PAY-COV08: Settlement integrity coverage

- Threats covered: PAY-T07, PAY-R06, PAY-E06, PAY-DOS06.
- Asset coverage: SettlementReconciliationRecord.
- Boundary coverage: PAY-B11 and PAY-B13.
- Required control evidence: immutable PSP settlement input, variance dashboard, finance role separation.
- Detection evidence: settlement-reconciliation dashboard and Cedar deny.

### PAY-COV09: PSP credential coverage

- Threats covered: PAY-I02, PAY-S06, PAY-DOS03.
- Asset coverage: PspCredentialSecret.
- Boundary coverage: PAY-B06 and PAY-B04.
- Required control evidence: OpenBao path scoping, short-lived access, adapter identity, key rotation.
- Detection evidence: OpenBao audit, adapter error surge, and `AbuseDefenceAttestationFailed`.

### PAY-COV10: Fraud decision coverage

- Threats covered: PAY-T08, PAY-DOS01, PAY-RR08.
- Asset coverage: AbuseDecisionRecord.
- Boundary coverage: PAY-B07.
- Required control evidence: Cedar decision log, risk score provenance, manual review state, tenant appeal trail.
- Detection evidence: fraud dashboard, Cedar deny, and `AbuseDefenceQuotaExceeded`.

## Incident Response Playbook References

| Incident class | Runbook |
|---|---|
| PCI incident or card-data exposure | `../runbooks/pci-incident-response.md` |
| PSP outage | `../runbooks/psp-outage.md` |
| PSP failover cascade | `../runbooks/psp-failover-cascade-execution.md` |
| Fraud spike or BIN attack | `../runbooks/fraud-spike-detected.md` |
| Chargeback investigation | `../runbooks/chargeback-cascade-investigation.md` |
| Dispute escalation | `../runbooks/dispute-escalation.md` |
| AML suspicious activity | `../runbooks/aml-suspicious-activity-detected.md` |
| KYC/AML pipeline stall | `../runbooks/kyc-aml-screening-pipeline-stall.md` |
| Payout failure | `../runbooks/payout-failed.md` |
| Refund mismatch | `../runbooks/refund-mismatch.md` |
| Double charge | `../runbooks/double-charge-detected.md` |
| Elder financial abuse | `../runbooks/elder-financial-abuse.md` |

## Cross-References

- Root service architecture: `../ARCHITECTURE.md`.
- Product requirements: `../PRD.md`.
- Payments OpenAPI contract: `../contracts/openapi-v1.yaml`.
- Payments AsyncAPI contract: `../contracts/asyncapi-v1.yaml`.
- PSP adapter trait: `../contracts/psp-adapter-trait.md`.
- Charge capability: `../capabilities/charge.yaml`.
- Refund capability: `../capabilities/refund.yaml`.
- Payout capability: `../capabilities/payout.yaml`.
- Dispute capability: `../capabilities/dispute.yaml`.
- Sub-merchant onboarding capability: `../capabilities/sub-merchant-onboarding.yaml`.
- Multi-PSP routing decision: `../decisions/ADR-PAY-001-multi-psp-routing-with-failover-cascade.md`.
- Charge authorization Cedar: `../policy/charge-authorization.cedar`.
- Refund authorization Cedar: `../policy/refund-authorization.cedar`.
- Payout authorization Cedar: `../policy/payout-authorization.cedar`.
- Dispute authorization Cedar: `../policy/dispute-authorization.cedar`.
- Sub-merchant onboarding Cedar: `../policy/sub-merchant-onboarding.cedar`.
- Abuse defence Cedar: `../policy/abuse-defence.cedar`.
- Fraud dashboard: `../dashboards/fraud-signals.md`.
- Settlement dashboard: `../dashboards/settlement-reconciliation.json`.
- ADR-0263 observability emission contract: `../../../docs/decisions/ADR-0706-observability-live-apex.md`.
- ADR-0243 Cedar as universal gate: `../../../docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- ADR-0244 tenant as universal scoping primitive: `../../../docs/decisions/ADR-0702-identity-authz-live-apex.md`.
- ADR-0297 abuse defence baseline: `../../../docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- ADR-0319 information barrier: `../../../docs/decisions/ADR-0709-general-live-apex.md`.

## Checkpoint Notes

- This document does not modify payments decisions or runbooks.
- It treats raw PAN/CVV entry into Oyatie systems as an incident, not a supported state.
- It assumes all financial mutations emit audit_id and tenant_id per ADR-0263.
- It accepts that fraud controls combine deterministic Cedar denies with risk scoring and manual review.
