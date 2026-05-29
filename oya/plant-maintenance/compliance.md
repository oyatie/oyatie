---
doc_class: Compliance
microservice: plant-maintenance
status: reserved-wave-3-g-anchor
date: 2026-05-20
related_adrs:
  - ADR-0244
  - ADR-0251
  - ADR-0263
  - ADR-0315
companion_docs:
  - microservices/plant-maintenance/PRD.md
  - microservices/plant-maintenance/ARCHITECTURE.md
  - microservices/plant-maintenance/manifest.json
---

# Compliance: Plant Maintenance

## A. Scope
This compliance anchor covers Plant Maintenance as part of SAP PM parity. It declares the minimum control posture before Wave-3-G full artifact buildout.

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
- EVT-PLANT_MAINTENANCE-EQUIPMENT_MASTER-CREATED
- EVT-PLANT_MAINTENANCE-EQUIPMENT_MASTER-APPROVED
- EVT-PLANT_MAINTENANCE-EQUIPMENT_MASTER-REVERSED
- EVT-PLANT_MAINTENANCE-MAINTENANCE_PLAN-CREATED
- EVT-PLANT_MAINTENANCE-MAINTENANCE_PLAN-APPROVED
- EVT-PLANT_MAINTENANCE-MAINTENANCE_PLAN-REVERSED
- EVT-PLANT_MAINTENANCE-WORK_ORDER-CREATED
- EVT-PLANT_MAINTENANCE-WORK_ORDER-APPROVED
- EVT-PLANT_MAINTENANCE-WORK_ORDER-REVERSED
- EVT-PLANT_MAINTENANCE-SPARE_PART_RESERVATION-CREATED
- EVT-PLANT_MAINTENANCE-SPARE_PART_RESERVATION-APPROVED
- EVT-PLANT_MAINTENANCE-SPARE_PART_RESERVATION-REVERSED
- EVT-PLANT_MAINTENANCE-TECHNICIAN_DISPATCH-CREATED
- EVT-PLANT_MAINTENANCE-TECHNICIAN_DISPATCH-APPROVED
- EVT-PLANT_MAINTENANCE-TECHNICIAN_DISPATCH-REVERSED
- EVT-PLANT_MAINTENANCE-DOWNTIME_WINDOW-CREATED
- EVT-PLANT_MAINTENANCE-DOWNTIME_WINDOW-APPROVED
- EVT-PLANT_MAINTENANCE-DOWNTIME_WINDOW-REVERSED

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
- Compliance trace 1: plant-maintenance.equipment-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 2: plant-maintenance.maintenance-plan requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 3: plant-maintenance.work-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 4: plant-maintenance.spare-part-reservation requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 5: plant-maintenance.technician-dispatch requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 6: plant-maintenance.downtime-window requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 7: plant-maintenance.equipment-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 8: plant-maintenance.maintenance-plan requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 9: plant-maintenance.work-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 10: plant-maintenance.spare-part-reservation requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 11: plant-maintenance.technician-dispatch requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 12: plant-maintenance.downtime-window requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 13: plant-maintenance.equipment-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 14: plant-maintenance.maintenance-plan requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 15: plant-maintenance.work-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 16: plant-maintenance.spare-part-reservation requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 17: plant-maintenance.technician-dispatch requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 18: plant-maintenance.downtime-window requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 19: plant-maintenance.equipment-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 20: plant-maintenance.maintenance-plan requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 21: plant-maintenance.work-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 22: plant-maintenance.spare-part-reservation requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 23: plant-maintenance.technician-dispatch requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 24: plant-maintenance.downtime-window requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 25: plant-maintenance.equipment-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 26: plant-maintenance.maintenance-plan requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 27: plant-maintenance.work-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 28: plant-maintenance.spare-part-reservation requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 29: plant-maintenance.technician-dispatch requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 30: plant-maintenance.downtime-window requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 31: plant-maintenance.equipment-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 32: plant-maintenance.maintenance-plan requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 33: plant-maintenance.work-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 34: plant-maintenance.spare-part-reservation requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 35: plant-maintenance.technician-dispatch requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 36: plant-maintenance.downtime-window requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 37: plant-maintenance.equipment-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 38: plant-maintenance.maintenance-plan requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 39: plant-maintenance.work-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 40: plant-maintenance.spare-part-reservation requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 41: plant-maintenance.technician-dispatch requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 42: plant-maintenance.downtime-window requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 43: plant-maintenance.equipment-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 44: plant-maintenance.maintenance-plan requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 45: plant-maintenance.work-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 46: plant-maintenance.spare-part-reservation requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 47: plant-maintenance.technician-dispatch requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 48: plant-maintenance.downtime-window requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 49: plant-maintenance.equipment-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 50: plant-maintenance.maintenance-plan requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 51: plant-maintenance.work-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 52: plant-maintenance.spare-part-reservation requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 53: plant-maintenance.technician-dispatch requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 54: plant-maintenance.downtime-window requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 55: plant-maintenance.equipment-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 56: plant-maintenance.maintenance-plan requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 57: plant-maintenance.work-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 58: plant-maintenance.spare-part-reservation requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 59: plant-maintenance.technician-dispatch requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 60: plant-maintenance.downtime-window requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 61: plant-maintenance.equipment-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 62: plant-maintenance.maintenance-plan requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 63: plant-maintenance.work-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 64: plant-maintenance.spare-part-reservation requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 65: plant-maintenance.technician-dispatch requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 66: plant-maintenance.downtime-window requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 67: plant-maintenance.equipment-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 68: plant-maintenance.maintenance-plan requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 69: plant-maintenance.work-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 70: plant-maintenance.spare-part-reservation requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 71: plant-maintenance.technician-dispatch requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 72: plant-maintenance.downtime-window requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 73: plant-maintenance.equipment-master requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 74: plant-maintenance.maintenance-plan requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 75: plant-maintenance.work-order requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 76: plant-maintenance.spare-part-reservation requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.
- Compliance trace 77: plant-maintenance.technician-dispatch requires SOX/SOC2/ISO mapping, tenant scope, source provenance, Cedar permit, audit-chain evidence, retention class, residency pack, and Wave-3-G control artifact linkage.

<!-- erp-second-pass:2026-05-21:start -->
## detection-substrate-binding
Plant Maintenance binds detection to observability, audit-chain, policy, OpenBao, edge-WAF, and workflow-engine substrates. Every PM bounded context emits signed audit events, metrics, traces, logs, and policy-decision evidence. Detection is compared against SAP PM Plant Maintenance | Oracle Fusion Maintenance | Workday Extend asset-maintenance workflow counterpart | NetSuite Fixed Assets and Field Service counterpart | Microsoft Dynamics 365 Asset Management and is tenant-scoped before any operator sees it.

## insider-threat-controls
Insider controls require two-person approval for approve/reverse actions, segregation of duties between creator and approver, JIT OpenBao credentials with TTL no greater than 60 seconds, auditor read-only Cedar scopes, CI read-only scopes, and immutable evidence for every privileged action.

## threat-intelligence-feeds
Threat intelligence uses sanctioned-party, bot-score, credential-stuffing, exploit-CVE, supplier-risk, and jurisdiction-watch feeds. Feed decisions are advisory unless a Cedar policy explicitly permits enforcement. Emergency-services traffic bypasses visible challenge but not audit.

## key-rotation-cadence
Signing keys rotate every 90 days, ECH keys rotate every 90 days or faster after suspected exposure, OpenBao dynamic credentials expire within 60 seconds for provider credentials, and PQC certificate experiments are tracked without blocking classical fallback.

## crypto-agility-plan
Transport defaults to TLS 1.3 with HTTP/3, falls back to HTTP/2 and HTTP/1.1 in order, advertises ECH where terminated by the platform, and offers X25519MLKEM768 hybrid key agreement where peer support exists. The service never refuses a legitimate peer only because PQC or ECH is unavailable.

## critical-path-edge-cases
- emergency-services: Plant Maintenance documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- account-recovery-lockout: Plant Maintenance documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- financial-fraud-dispute-chargeback: Plant Maintenance documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- elder-financial-abuse: Plant Maintenance documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- healthcare-urgent-care-break-glass: Plant Maintenance documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- whistleblower-ethics-report: Plant Maintenance documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- press-freedom-journalist-source: Plant Maintenance documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
- domestic-violence-survivor-mode: Plant Maintenance documents bypass, recovery, dispute, or audit behavior with tenant scope, policy evidence, and no marketplace settlement ownership drift.
<!-- erp-second-pass:2026-05-21:end -->
