---
doc_class: DPIA
status: accepted
date: 2026-05-20
owner: ops-sre-reliability
related_adrs:
  - ADR-0244
  - ADR-0251
  - ADR-0263
  - ADR-0276
  - ADR-0284
  - ADR-0292
companion_docs:
  - microservices/ops-dashboard-control-center/compliance.md
  - microservices/ops-dashboard-control-center/threat-model.md
  - microservices/ops-dashboard-control-center/ARCHITECTURE.md
planned_enforcement_ref: oya-governance-adr-adherence-matrix
---

# DPIA — ops-dashboard-control-center

Data Protection Impact Assessment per GDPR Article 35, KR-PIPA Article 33, and equivalent jurisdictional requirements.

## §1 Processing description

**Controller:** oyatie platform (reserved-namespace tenant per ADR-0242).  
**Processor:** ops-dashboard-control-center µservice.  
**Data subjects:** Platform operators (employees / contractors); platform tenants (indirectly, via posture views).  
**Purpose:** Internal ops administration — incident management, deployment approval, tenant-isolation posture, evidence-pack export, Cedar-fragment publish.

## §2 Data classes processed

| Data class | Examples | Basis | Retention |
|---|---|---|---|
| `PII_IDENTIFYING` | Operator name, email, employee ID in audit log | Legitimate interest (internal ops security) | 7yr audit log |
| `AUTHENTICATION` | Step-up auth tokens, session IDs, FIDO2 assertion | Contract (employment) | Session lifetime + 90d audit |
| `AUDIT` | Every operator action record, Cedar verdict, tenant_id | Legal obligation (SOC 2, ISO 27001, KR-ISMS) | 7yr per ADR-0263 retention class |
| `INTERNAL_ONLY` | Cluster health signals, deployment approval records | Legitimate interest | Configurable (default 1yr) |

## §3 Necessity and proportionality

Every data field collected is justified:
- Operator PII in audit log: required for non-repudiability (SOC 2 CC6.2; ISO 27001 A.9.4).
- Tenant posture data: scoped to operator's authorized tenant list; minimised to evidence refs, not raw tenant data.
- Session recording (T3 only): proportionate to risk level of T3 actions (Cedar fragment publish, rollback); reviewed only on incident or access review.

## §4 Risks and mitigations

| Risk | Severity | Mitigation |
|---|---|---|
| Operator audit log exposed to unauthorised party | High | `policy/auditor-scope.cedar` JIT scoping; no standing read access |
| Tenant posture pivot (operator reads another tenant's data) | Critical | Cedar default-deny; RLS at Postgres layer; `TenantScopeViolationDetected` alert |
| Session recording exfiltration | High | Encrypted at rest in OpenBao-managed key; accessible only to `oyatie.ops.forensics` |
| Insider exfiltration of bulk audit log | High | Per-operator rate limit; UEBA baseline; anomaly alert; bulk-export requires T3 step-up |
| Audit log tampering | Critical | Merkle-sealed per ADR-0028; tamper attempt triggers `PolicyViolationDetected` alert |

## §5 Residency

All PII processed in the operator's home region cell. Cross-region transfer of audit logs requires explicit pack overlay (`oya-pack-eu` GDPR Art. 46 transfer mechanism). KR operators: data stays in `ap-northeast-2`. EU operators: data stays in `eu-west-1`.

## §6 Data subject rights

| Right | Mechanism |
|---|---|
| Access (GDPR Art. 15 / KR-PIPA Art. 35) | Operator DSAR triggers evidence-pack export of own audit records |
| Erasure (GDPR Art. 17) | Blocked for audit records during legal hold; honoured for non-audit PII after retention period |
| Portability (GDPR Art. 20) | Signed JSONL export per `compliance.md §portability` |
| Objection (GDPR Art. 21) | Audit logging mandatory; operator cannot object to legal-obligation processing |

## §7 DPA consultation

DPA consultation required for `oya-pack-eu` tenants when session recording is enabled for EU-resident operators. Legal review ticket required before enabling in EU cell.
