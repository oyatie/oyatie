---
doc_class: Compliance
microservice: crm
status: reserved-wave-3-g-anchor
date: 2026-05-20
related_adrs:
  - ADR-0244
  - ADR-0251
  - ADR-0263
  - ADR-0315
companion_docs:
  - microservices/crm/PRD.md
  - microservices/crm/ARCHITECTURE.md
  - microservices/crm/manifest.json
---

# Compliance: Customer Relationship Management

## A. Scope
This compliance anchor covers Customer Relationship Management as part of SAP CRM parity. It declares the minimum control posture before Wave-3-G full artifact buildout.

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
- EVT-CRM-ACCOUNT_MASTER-CREATED
- EVT-CRM-ACCOUNT_MASTER-APPROVED
- EVT-CRM-ACCOUNT_MASTER-REVERSED
- EVT-CRM-OPPORTUNITY-CREATED
- EVT-CRM-OPPORTUNITY-APPROVED
- EVT-CRM-OPPORTUNITY-REVERSED
- EVT-CRM-QUOTE-CREATED
- EVT-CRM-QUOTE-APPROVED
- EVT-CRM-QUOTE-REVERSED
- EVT-CRM-SERVICE_CASE-CREATED
- EVT-CRM-SERVICE_CASE-APPROVED
- EVT-CRM-SERVICE_CASE-REVERSED
- EVT-CRM-CAMPAIGN-CREATED
- EVT-CRM-CAMPAIGN-APPROVED
- EVT-CRM-CAMPAIGN-REVERSED
- EVT-CRM-LOYALTY_LEDGER-CREATED
- EVT-CRM-LOYALTY_LEDGER-APPROVED
- EVT-CRM-LOYALTY_LEDGER-REVERSED

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
- Compliance trace 1: crm.account-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 2: crm.opportunity requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 3: crm.quote requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 4: crm.service-case requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 5: crm.campaign requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 6: crm.loyalty-ledger requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 7: crm.account-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 8: crm.opportunity requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 9: crm.quote requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 10: crm.service-case requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 11: crm.campaign requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 12: crm.loyalty-ledger requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 13: crm.account-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 14: crm.opportunity requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 15: crm.quote requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 16: crm.service-case requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 17: crm.campaign requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 18: crm.loyalty-ledger requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 19: crm.account-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 20: crm.opportunity requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 21: crm.quote requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 22: crm.service-case requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 23: crm.campaign requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 24: crm.loyalty-ledger requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 25: crm.account-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 26: crm.opportunity requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 27: crm.quote requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 28: crm.service-case requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 29: crm.campaign requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 30: crm.loyalty-ledger requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 31: crm.account-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 32: crm.opportunity requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 33: crm.quote requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 34: crm.service-case requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 35: crm.campaign requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 36: crm.loyalty-ledger requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 37: crm.account-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 38: crm.opportunity requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 39: crm.quote requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 40: crm.service-case requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 41: crm.campaign requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 42: crm.loyalty-ledger requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 43: crm.account-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 44: crm.opportunity requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 45: crm.quote requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 46: crm.service-case requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 47: crm.campaign requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 48: crm.loyalty-ledger requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 49: crm.account-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 50: crm.opportunity requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 51: crm.quote requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 52: crm.service-case requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 53: crm.campaign requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 54: crm.loyalty-ledger requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 55: crm.account-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 56: crm.opportunity requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 57: crm.quote requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 58: crm.service-case requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 59: crm.campaign requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 60: crm.loyalty-ledger requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 61: crm.account-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 62: crm.opportunity requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 63: crm.quote requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 64: crm.service-case requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 65: crm.campaign requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 66: crm.loyalty-ledger requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 67: crm.account-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 68: crm.opportunity requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 69: crm.quote requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 70: crm.service-case requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 71: crm.campaign requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 72: crm.loyalty-ledger requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 73: crm.account-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 74: crm.opportunity requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 75: crm.quote requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 76: crm.service-case requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 77: crm.campaign requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.

<!-- erp-second-pass:2026-05-21:start -->
## detection-substrate-binding
Customer Relationship Management binds detection to observability, audit-chain, policy, OpenBao, edge-WAF, and workflow-engine substrates. Every CRM bounded context emits signed audit events, metrics, traces, logs, and policy-decision evidence. Detection is compared against SAP CRM Customer Relationship Management | Oracle Fusion Sales and Service | Workday customer and service-workflow counterpart | NetSuite CRM | Microsoft Dynamics 365 Customer Engagement and is tenant-scoped before any operator sees it.

## insider-threat-controls
Insider controls require two-person approval for approve/reverse actions, segregation of duties between creator and approver, JIT OpenBao credentials with TTL no greater than 60 seconds, auditor read-only Cedar scopes, CI read-only scopes, and immutable evidence for every privileged action.

## threat-intelligence-feeds
Threat intelligence uses sanctioned-party, bot-score, credential-stuffing, exploit-CVE, supplier-risk, and jurisdiction-watch feeds. Feed decisions are advisory unless a Cedar policy explicitly permits enforcement. Emergency-services traffic bypasses visible challenge but not audit.

## key-rotation-cadence
Signing keys rotate every 90 days, ECH keys rotate every 90 days or faster after suspected exposure, OpenBao dynamic credentials expire within 60 seconds for provider credentials, and PQC certificate experiments are tracked without blocking classical fallback.

## crypto-agility-plan
Transport defaults to TLS 1.3 with HTTP/3, falls back to HTTP/2 and HTTP/1.1 in order, advertises ECH where terminated by the platform, and offers X25519MLKEM768 hybrid key agreement where peer support exists. The service never refuses a legitimate peer only because PQC or ECH is unavailable.

## critical-path-edge-cases
- emergency-services: Customer Relationship Management documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- account-recovery-lockout: Customer Relationship Management documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- financial-fraud-dispute-chargeback: Customer Relationship Management documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- elder-financial-abuse: Customer Relationship Management documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- healthcare-urgent-care-break-glass: Customer Relationship Management documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- whistleblower-ethics-report: Customer Relationship Management documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- press-freedom-journalist-source: Customer Relationship Management documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- domestic-violence-survivor-mode: Customer Relationship Management documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
<!-- erp-second-pass:2026-05-21:end -->
