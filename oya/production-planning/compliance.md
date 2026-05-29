---
doc_class: Compliance
microservice: production-planning
status: reserved-wave-3-g-anchor
date: 2026-05-20
related_adrs:
  - ADR-0244
  - ADR-0251
  - ADR-0263
  - ADR-0315
companion_docs:
  - microservices/production-planning/PRD.md
  - microservices/production-planning/ARCHITECTURE.md
  - microservices/production-planning/manifest.json
---

# Compliance: Production Planning

## A. Scope
This compliance anchor covers Production Planning as part of SAP PP parity. It declares the minimum control posture before Wave-3-G full artifact buildout.

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
- EVT-PRODUCTION_PLANNING-BOM_REVISION-CREATED
- EVT-PRODUCTION_PLANNING-BOM_REVISION-APPROVED
- EVT-PRODUCTION_PLANNING-BOM_REVISION-REVERSED
- EVT-PRODUCTION_PLANNING-MRP_RUN-CREATED
- EVT-PRODUCTION_PLANNING-MRP_RUN-APPROVED
- EVT-PRODUCTION_PLANNING-MRP_RUN-REVERSED
- EVT-PRODUCTION_PLANNING-CAPACITY_CALENDAR-CREATED
- EVT-PRODUCTION_PLANNING-CAPACITY_CALENDAR-APPROVED
- EVT-PRODUCTION_PLANNING-CAPACITY_CALENDAR-REVERSED
- EVT-PRODUCTION_PLANNING-ROUTING_STEP-CREATED
- EVT-PRODUCTION_PLANNING-ROUTING_STEP-APPROVED
- EVT-PRODUCTION_PLANNING-ROUTING_STEP-REVERSED
- EVT-PRODUCTION_PLANNING-PRODUCTION_ORDER-CREATED
- EVT-PRODUCTION_PLANNING-PRODUCTION_ORDER-APPROVED
- EVT-PRODUCTION_PLANNING-PRODUCTION_ORDER-REVERSED
- EVT-PRODUCTION_PLANNING-SHOP_FLOOR_RELEASE-CREATED
- EVT-PRODUCTION_PLANNING-SHOP_FLOOR_RELEASE-APPROVED
- EVT-PRODUCTION_PLANNING-SHOP_FLOOR_RELEASE-REVERSED

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
- Compliance trace 1: production-planning.bom-revision requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 2: production-planning.mrp-run requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 3: production-planning.capacity-calendar requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 4: production-planning.routing-step requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 5: production-planning.production-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 6: production-planning.shop-floor-release requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 7: production-planning.bom-revision requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 8: production-planning.mrp-run requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 9: production-planning.capacity-calendar requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 10: production-planning.routing-step requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 11: production-planning.production-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 12: production-planning.shop-floor-release requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 13: production-planning.bom-revision requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 14: production-planning.mrp-run requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 15: production-planning.capacity-calendar requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 16: production-planning.routing-step requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 17: production-planning.production-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 18: production-planning.shop-floor-release requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 19: production-planning.bom-revision requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 20: production-planning.mrp-run requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 21: production-planning.capacity-calendar requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 22: production-planning.routing-step requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 23: production-planning.production-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 24: production-planning.shop-floor-release requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 25: production-planning.bom-revision requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 26: production-planning.mrp-run requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 27: production-planning.capacity-calendar requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 28: production-planning.routing-step requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 29: production-planning.production-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 30: production-planning.shop-floor-release requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 31: production-planning.bom-revision requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 32: production-planning.mrp-run requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 33: production-planning.capacity-calendar requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 34: production-planning.routing-step requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 35: production-planning.production-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 36: production-planning.shop-floor-release requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 37: production-planning.bom-revision requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 38: production-planning.mrp-run requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 39: production-planning.capacity-calendar requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 40: production-planning.routing-step requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 41: production-planning.production-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 42: production-planning.shop-floor-release requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 43: production-planning.bom-revision requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 44: production-planning.mrp-run requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 45: production-planning.capacity-calendar requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 46: production-planning.routing-step requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 47: production-planning.production-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 48: production-planning.shop-floor-release requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 49: production-planning.bom-revision requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 50: production-planning.mrp-run requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 51: production-planning.capacity-calendar requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 52: production-planning.routing-step requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 53: production-planning.production-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 54: production-planning.shop-floor-release requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 55: production-planning.bom-revision requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 56: production-planning.mrp-run requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 57: production-planning.capacity-calendar requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 58: production-planning.routing-step requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 59: production-planning.production-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 60: production-planning.shop-floor-release requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 61: production-planning.bom-revision requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 62: production-planning.mrp-run requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 63: production-planning.capacity-calendar requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 64: production-planning.routing-step requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 65: production-planning.production-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 66: production-planning.shop-floor-release requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 67: production-planning.bom-revision requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 68: production-planning.mrp-run requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 69: production-planning.capacity-calendar requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 70: production-planning.routing-step requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 71: production-planning.production-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 72: production-planning.shop-floor-release requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 73: production-planning.bom-revision requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 74: production-planning.mrp-run requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 75: production-planning.capacity-calendar requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 76: production-planning.routing-step requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 77: production-planning.production-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.

<!-- erp-second-pass:2026-05-21:start -->
## detection-substrate-binding
Production Planning binds detection to observability, audit-chain, policy, OpenBao, edge-WAF, and workflow-engine substrates. Every PP bounded context emits signed audit events, metrics, traces, logs, and policy-decision evidence. Detection is compared against SAP PP Production Planning | Oracle Fusion Cloud Manufacturing | Workday Adaptive Planning production-capacity counterpart | NetSuite Manufacturing WIP and Routings | Microsoft Dynamics 365 Supply Chain Management and is tenant-scoped before any operator sees it.

## insider-threat-controls
Insider controls require two-person approval for approve/reverse actions, segregation of duties between creator and approver, JIT OpenBao credentials with TTL no greater than 60 seconds, auditor read-only Cedar scopes, CI read-only scopes, and immutable evidence for every privileged action.

## threat-intelligence-feeds
Threat intelligence uses sanctioned-party, bot-score, credential-stuffing, exploit-CVE, supplier-risk, and jurisdiction-watch feeds. Feed decisions are advisory unless a Cedar policy explicitly permits enforcement. Emergency-services traffic bypasses visible challenge but not audit.

## key-rotation-cadence
Signing keys rotate every 90 days, ECH keys rotate every 90 days or faster after suspected exposure, OpenBao dynamic credentials expire within 60 seconds for provider credentials, and PQC certificate experiments are tracked without blocking classical fallback.

## crypto-agility-plan
Transport defaults to TLS 1.3 with HTTP/3, falls back to HTTP/2 and HTTP/1.1 in order, advertises ECH where terminated by the platform, and offers X25519MLKEM768 hybrid key agreement where peer support exists. The service never refuses a legitimate peer only because PQC or ECH is unavailable.

## critical-path-edge-cases
- emergency-services: Production Planning documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- account-recovery-lockout: Production Planning documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- financial-fraud-dispute-chargeback: Production Planning documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- elder-financial-abuse: Production Planning documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- healthcare-urgent-care-break-glass: Production Planning documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- whistleblower-ethics-report: Production Planning documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- press-freedom-journalist-source: Production Planning documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- domestic-violence-survivor-mode: Production Planning documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
<!-- erp-second-pass:2026-05-21:end -->
