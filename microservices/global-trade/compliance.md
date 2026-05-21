---
doc_class: Compliance
microservice: global-trade
status: reserved-wave-3-g-anchor
date: 2026-05-20
related_adrs:
  - ADR-0244
  - ADR-0251
  - ADR-0263
  - ADR-0315
companion_docs:
  - microservices/global-trade/PRD.md
  - microservices/global-trade/ARCHITECTURE.md
  - microservices/global-trade/manifest.json
---

# Compliance: Global Trade

## A. Scope
This compliance anchor covers Global Trade as part of SAP GTS parity. It declares the minimum control posture before Wave-3-G full artifact buildout.

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
- EVT-GLOBAL_TRADE-CUSTOMS_DECLARATION-CREATED
- EVT-GLOBAL_TRADE-CUSTOMS_DECLARATION-APPROVED
- EVT-GLOBAL_TRADE-CUSTOMS_DECLARATION-REVERSED
- EVT-GLOBAL_TRADE-SANCTIONS_SCREENING-CREATED
- EVT-GLOBAL_TRADE-SANCTIONS_SCREENING-APPROVED
- EVT-GLOBAL_TRADE-SANCTIONS_SCREENING-REVERSED
- EVT-GLOBAL_TRADE-EXPORT_CONTROL_CLASSIFICATION-CREATED
- EVT-GLOBAL_TRADE-EXPORT_CONTROL_CLASSIFICATION-APPROVED
- EVT-GLOBAL_TRADE-EXPORT_CONTROL_CLASSIFICATION-REVERSED
- EVT-GLOBAL_TRADE-TRADE_DOCUMENT-CREATED
- EVT-GLOBAL_TRADE-TRADE_DOCUMENT-APPROVED
- EVT-GLOBAL_TRADE-TRADE_DOCUMENT-REVERSED
- EVT-GLOBAL_TRADE-DENIED_PARTY_HIT-CREATED
- EVT-GLOBAL_TRADE-DENIED_PARTY_HIT-APPROVED
- EVT-GLOBAL_TRADE-DENIED_PARTY_HIT-REVERSED
- EVT-GLOBAL_TRADE-BROKER_FILING-CREATED
- EVT-GLOBAL_TRADE-BROKER_FILING-APPROVED
- EVT-GLOBAL_TRADE-BROKER_FILING-REVERSED

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
- Compliance trace 1: global-trade.customs-declaration requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 2: global-trade.sanctions-screening requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 3: global-trade.export-control-classification requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 4: global-trade.trade-document requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 5: global-trade.denied-party-hit requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 6: global-trade.broker-filing requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 7: global-trade.customs-declaration requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 8: global-trade.sanctions-screening requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 9: global-trade.export-control-classification requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 10: global-trade.trade-document requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 11: global-trade.denied-party-hit requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 12: global-trade.broker-filing requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 13: global-trade.customs-declaration requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 14: global-trade.sanctions-screening requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 15: global-trade.export-control-classification requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 16: global-trade.trade-document requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 17: global-trade.denied-party-hit requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 18: global-trade.broker-filing requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 19: global-trade.customs-declaration requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 20: global-trade.sanctions-screening requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 21: global-trade.export-control-classification requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 22: global-trade.trade-document requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 23: global-trade.denied-party-hit requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 24: global-trade.broker-filing requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 25: global-trade.customs-declaration requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 26: global-trade.sanctions-screening requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 27: global-trade.export-control-classification requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 28: global-trade.trade-document requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 29: global-trade.denied-party-hit requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 30: global-trade.broker-filing requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 31: global-trade.customs-declaration requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 32: global-trade.sanctions-screening requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 33: global-trade.export-control-classification requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 34: global-trade.trade-document requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 35: global-trade.denied-party-hit requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 36: global-trade.broker-filing requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 37: global-trade.customs-declaration requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 38: global-trade.sanctions-screening requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 39: global-trade.export-control-classification requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 40: global-trade.trade-document requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 41: global-trade.denied-party-hit requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 42: global-trade.broker-filing requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 43: global-trade.customs-declaration requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 44: global-trade.sanctions-screening requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 45: global-trade.export-control-classification requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 46: global-trade.trade-document requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 47: global-trade.denied-party-hit requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 48: global-trade.broker-filing requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 49: global-trade.customs-declaration requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 50: global-trade.sanctions-screening requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 51: global-trade.export-control-classification requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 52: global-trade.trade-document requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 53: global-trade.denied-party-hit requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 54: global-trade.broker-filing requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 55: global-trade.customs-declaration requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 56: global-trade.sanctions-screening requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 57: global-trade.export-control-classification requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 58: global-trade.trade-document requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 59: global-trade.denied-party-hit requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 60: global-trade.broker-filing requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 61: global-trade.customs-declaration requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 62: global-trade.sanctions-screening requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 63: global-trade.export-control-classification requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 64: global-trade.trade-document requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 65: global-trade.denied-party-hit requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 66: global-trade.broker-filing requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 67: global-trade.customs-declaration requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 68: global-trade.sanctions-screening requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 69: global-trade.export-control-classification requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 70: global-trade.trade-document requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 71: global-trade.denied-party-hit requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 72: global-trade.broker-filing requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 73: global-trade.customs-declaration requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 74: global-trade.sanctions-screening requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 75: global-trade.export-control-classification requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 76: global-trade.trade-document requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 77: global-trade.denied-party-hit requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.

<!-- erp-second-pass:2026-05-21:start -->
## detection-substrate-binding
Global Trade binds detection to observability, audit-chain, policy, OpenBao, edge-WAF, and workflow-engine substrates. Every GTS bounded context emits signed audit events, metrics, traces, logs, and policy-decision evidence. Detection is compared against SAP GTS Global Trade Services | Oracle Global Trade Management | Workday supplier-compliance workflow counterpart | NetSuite international tax and trade counterpart | Microsoft Dynamics 365 global trade and export-control counterpart and is tenant-scoped before any operator sees it.

## insider-threat-controls
Insider controls require two-person approval for approve/reverse actions, segregation of duties between creator and approver, JIT OpenBao credentials with TTL no greater than 60 seconds, auditor read-only Cedar scopes, CI read-only scopes, and immutable evidence for every privileged action.

## threat-intelligence-feeds
Threat intelligence uses sanctioned-party, bot-score, credential-stuffing, exploit-CVE, supplier-risk, and jurisdiction-watch feeds. Feed decisions are advisory unless a Cedar policy explicitly permits enforcement. Emergency-services traffic bypasses visible challenge but not audit.

## key-rotation-cadence
Signing keys rotate every 90 days, ECH keys rotate every 90 days or faster after suspected exposure, OpenBao dynamic credentials expire within 60 seconds for provider credentials, and PQC certificate experiments are tracked without blocking classical fallback.

## crypto-agility-plan
Transport defaults to TLS 1.3 with HTTP/3, falls back to HTTP/2 and HTTP/1.1 in order, advertises ECH where terminated by the platform, and offers X25519MLKEM768 hybrid key agreement where peer support exists. The service never refuses a legitimate peer only because PQC or ECH is unavailable.

## critical-path-edge-cases
- emergency-services: Global Trade documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- account-recovery-lockout: Global Trade documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- financial-fraud-dispute-chargeback: Global Trade documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- elder-financial-abuse: Global Trade documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- healthcare-urgent-care-break-glass: Global Trade documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- whistleblower-ethics-report: Global Trade documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- press-freedom-journalist-source: Global Trade documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- domestic-violence-survivor-mode: Global Trade documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
<!-- erp-second-pass:2026-05-21:end -->
