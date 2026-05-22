---
doc_class: Compliance
microservice: treasury
status: reserved-wave-3-g-anchor
date: 2026-05-20
related_adrs:
  - ADR-0244
  - ADR-0251
  - ADR-0263
  - ADR-0315
companion_docs:
  - microservices/treasury/PRD.md
  - microservices/treasury/ARCHITECTURE.md
  - microservices/treasury/manifest.json
---

# Compliance: Treasury

## A. Scope
This compliance anchor covers Treasury as part of SAP TRM parity. It declares the minimum control posture before Wave-3-G full artifact buildout.

## B. Control Families
- SOX 404: authorization, segregation of duties, evidence retention, control testing, and change approval.
- SOC 2: security, availability, processing integrity, confidentiality, and privacy.
- ISO 27001: asset inventory, access control, cryptography, logging, supplier relationship, incident response, and continuity.
- GDPR/LGPD/CPRA: data subject rights, lawful basis, retention, portability, deletion, and cross-border transfer controls.
- Jurisdictional tax: invoice, withholding, VAT/GST/sales tax, payroll or trade tax evidence where applicable.
- Industry packs: banking, insurance, healthcare, public sector, automotive, utilities, oil, pharma, and retail overlays activate only through pack metadata.

## C. Data Classification
- Tenant identifiers: confidential operational data.
- Source-system identifiers: confidential migration provenance.
- Financial or operational postings: regulated business records when applicable.
- Personal data: PII subject to jurisdictional pack rules.
- Trade, quality, maintenance, workforce, or lease evidence: regulated records when activated by pack.

## D. Audit Events
- EVT-TREASURY-CASH_POSITION-CREATED
- EVT-TREASURY-CASH_POSITION-APPROVED
- EVT-TREASURY-CASH_POSITION-REVERSED
- EVT-TREASURY-LIQUIDITY_FORECAST-CREATED
- EVT-TREASURY-LIQUIDITY_FORECAST-APPROVED
- EVT-TREASURY-LIQUIDITY_FORECAST-REVERSED
- EVT-TREASURY-BANK_ACCOUNT-CREATED
- EVT-TREASURY-BANK_ACCOUNT-APPROVED
- EVT-TREASURY-BANK_ACCOUNT-REVERSED
- EVT-TREASURY-DEBT_INSTRUMENT-CREATED
- EVT-TREASURY-DEBT_INSTRUMENT-APPROVED
- EVT-TREASURY-DEBT_INSTRUMENT-REVERSED
- EVT-TREASURY-FX_EXPOSURE-CREATED
- EVT-TREASURY-FX_EXPOSURE-APPROVED
- EVT-TREASURY-FX_EXPOSURE-REVERSED
- EVT-TREASURY-HEDGE_DESIGNATION-CREATED
- EVT-TREASURY-HEDGE_DESIGNATION-APPROVED
- EVT-TREASURY-HEDGE_DESIGNATION-REVERSED

## E. Required Evidence
- Cedar policy decision log for every mutation.
- Audit-chain event id for every critical state transition.
- Source-system row provenance for every migration import.
- Workflow run id for every approval, exception, reversal, or remediation.
- Data-residency decision for every regulated record.
- OpenBao reference for every secret or credential dependency.

## F. Risk Register
- Cross-tenant leakage: mitigated by ADR-0244 tenant scoping and Cedar default deny.
- Silent posting corruption: mitigated by idempotency, reversal events, reconciliation reports, and audit-chain evidence.
- Source-system mismatch: mitigated by dry-run import, checksums, and rejected-row queues.
- Jurisdictional non-compliance: mitigated by pack activation rules and compliance evidence exports.
- Operator overreach: mitigated by least privilege, break-glass evidence, and dual-control approval.

## G. Wave-3-G Follow-Up
Wave-3-G must add service-specific threat model, DPIA, Cedar files, auditor-scope policy, data-residency policy, CI-scope policy, runbooks, SLOs, dashboards, and scorecards.
- Compliance trace 1: treasury.cash-position requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 2: treasury.liquidity-forecast requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 3: treasury.bank-account requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 4: treasury.debt-instrument requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 5: treasury.fx-exposure requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 6: treasury.hedge-designation requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 7: treasury.cash-position requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 8: treasury.liquidity-forecast requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 9: treasury.bank-account requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 10: treasury.debt-instrument requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 11: treasury.fx-exposure requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 12: treasury.hedge-designation requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 13: treasury.cash-position requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 14: treasury.liquidity-forecast requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 15: treasury.bank-account requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 16: treasury.debt-instrument requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 17: treasury.fx-exposure requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 18: treasury.hedge-designation requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 19: treasury.cash-position requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 20: treasury.liquidity-forecast requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 21: treasury.bank-account requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 22: treasury.debt-instrument requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 23: treasury.fx-exposure requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 24: treasury.hedge-designation requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 25: treasury.cash-position requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 26: treasury.liquidity-forecast requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 27: treasury.bank-account requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 28: treasury.debt-instrument requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 29: treasury.fx-exposure requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 30: treasury.hedge-designation requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 31: treasury.cash-position requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 32: treasury.liquidity-forecast requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 33: treasury.bank-account requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 34: treasury.debt-instrument requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 35: treasury.fx-exposure requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 36: treasury.hedge-designation requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 37: treasury.cash-position requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 38: treasury.liquidity-forecast requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 39: treasury.bank-account requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 40: treasury.debt-instrument requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 41: treasury.fx-exposure requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 42: treasury.hedge-designation requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 43: treasury.cash-position requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 44: treasury.liquidity-forecast requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 45: treasury.bank-account requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 46: treasury.debt-instrument requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 47: treasury.fx-exposure requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 48: treasury.hedge-designation requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 49: treasury.cash-position requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 50: treasury.liquidity-forecast requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 51: treasury.bank-account requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 52: treasury.debt-instrument requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 53: treasury.fx-exposure requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 54: treasury.hedge-designation requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 55: treasury.cash-position requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 56: treasury.liquidity-forecast requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 57: treasury.bank-account requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 58: treasury.debt-instrument requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 59: treasury.fx-exposure requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 60: treasury.hedge-designation requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 61: treasury.cash-position requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 62: treasury.liquidity-forecast requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 63: treasury.bank-account requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 64: treasury.debt-instrument requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 65: treasury.fx-exposure requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 66: treasury.hedge-designation requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 67: treasury.cash-position requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 68: treasury.liquidity-forecast requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 69: treasury.bank-account requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 70: treasury.debt-instrument requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 71: treasury.fx-exposure requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 72: treasury.hedge-designation requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 73: treasury.cash-position requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 74: treasury.liquidity-forecast requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 75: treasury.bank-account requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 76: treasury.debt-instrument requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 77: treasury.fx-exposure requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.

<!-- erp-second-pass:2026-05-21:start -->
## detection-substrate-binding
Treasury binds detection to observability, audit-chain, policy, OpenBao, edge-WAF, and workflow-engine substrates. Every TRM bounded context emits signed audit events, metrics, traces, logs, and policy-decision evidence. Detection is compared against SAP TRM Treasury and Risk Management | Oracle Fusion Cash Management | Workday Financial Management cash and treasury counterpart | NetSuite Cash Management | Microsoft Dynamics 365 Finance Cash and Bank and is tenant-scoped before any operator sees it.

## insider-threat-controls
Insider controls require two-person approval for approve/reverse actions, segregation of duties between creator and approver, JIT OpenBao credentials with TTL no greater than 60 seconds, auditor read-only Cedar scopes, CI read-only scopes, and immutable evidence for every privileged action.

## threat-intelligence-feeds
Threat intelligence uses sanctioned-party, bot-score, credential-stuffing, exploit-CVE, supplier-risk, and jurisdiction-watch feeds. Feed decisions are advisory unless a Cedar policy explicitly permits enforcement. Emergency-services traffic bypasses visible challenge but not audit.

## key-rotation-cadence
Signing keys rotate every 90 days, ECH keys rotate every 90 days or faster after suspected exposure, OpenBao dynamic credentials expire within 60 seconds for provider credentials, and PQC certificate experiments are tracked without blocking classical fallback.

## crypto-agility-plan
Transport defaults to TLS 1.3 with HTTP/3, falls back to HTTP/2 and HTTP/1.1 in order, advertises ECH where terminated by the platform, and offers X25519MLKEM768 hybrid key agreement where peer support exists. The service never refuses a legitimate peer only because PQC or ECH is unavailable.

## critical-path-edge-cases
- emergency-services: Treasury documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- account-recovery-lockout: Treasury documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- financial-fraud-dispute-chargeback: Treasury documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- elder-financial-abuse: Treasury documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- healthcare-urgent-care-break-glass: Treasury documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- whistleblower-ethics-report: Treasury documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- press-freedom-journalist-source: Treasury documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- domestic-violence-survivor-mode: Treasury documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
<!-- erp-second-pass:2026-05-21:end -->
