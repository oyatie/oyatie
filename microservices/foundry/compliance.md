---
doc_class: COMPLIANCE
microservice: foundry
status: Accepted
date: 2026-05-18
owner_team: council-privacy + axis-foundry + ops-compliance
related_adrs: [ADR-0117, ADR-0136, ADR-0137]
---

# Compliance Mapping — foundry (consolidated)

## Regulatory frameworks in scope

| Framework | Jurisdiction | Scope of applicability |
|---|---|---|
| GDPR | EU (pack-eu) | Tenant data processing, DSR, Art.30 records |
| PIPA | KR (pack-kr; M01 launch) | Art.23 sensitive data; Art.39 cross-border |
| HIPAA | US-HC (pack-us-healthcare) | §164.312 technical safeguards; §164.316 retention |
| CCPA / CPRA | US (pack-us) | Consumer rights + opt-out |
| EU AI Act | EU (pack-eu) | High-risk AI categorisation; conformity assessment |
| KISA / K-ISMS | KR (pack-kr) | Korean cybersecurity certification |
| SOC 2 Type II | Global | Security + Availability + Confidentiality TSCs |
| ISO 27001 | Global | ISMS controls |
| FedRAMP | US gov (subsequent-to-M01-completion) | Moderate baseline |

## Control mapping (cross-BC)

| Control | GDPR | PIPA | HIPAA | EU AI Act | SOC 2 | Resident BC |
|---|---|---|---|---|---|---|
| Audit logging of all access | Art.30 | Art.29 | §164.312(b) | Art.12 | CC4.1 | evidence (canonical) |
| Encryption at rest | Art.32 | Art.29 | §164.312(a)(2)(iv) | — | CC6.7 | all BCs (per-BC adapters) |
| Encryption in transit | Art.32 | Art.29 | §164.312(e) | — | CC6.7 | all BCs (mTLS) |
| Access controls (RBAC + Cedar) | Art.32 | Art.29 | §164.312(a)(1) | Art.10 | CC6.1 | all BCs |
| DSR endpoints (access/rectification/erasure) | Art.15–17 | Art.35–37 | — | — | — | runtime+evidence (canonical) |
| Retention enforcement | Art.5(1)(e) | Art.21 | §164.316(b)(2) | — | CC4.2 | evidence (canonical) |
| High-risk AI conformity record | — | — | — | Art.12 + Annex IV | — | evidence + supervisor |
| Kill-switch / human-oversight | — | — | — | Art.14 | — | supervisor (canonical) |
| Provider risk management | Art.28 | Art.26 | §164.314(a) | Art.16 | CC9.2 | providers (canonical) |
| Eval+monitoring of model performance | — | — | — | Art.15 | — | eval (canonical) |
| Guardrail evidence of safety | — | — | — | Art.9 | — | guardrails+evidence |

## Per-BC compliance mappings

| BC | Primary compliance scope | Archive |
|---|---|---|
| runtime | Data subject rights cascade (session-state); per-tenant retention | `bc-sources/runtime/compliance.md` |
| supervisor | Human oversight (EU AI Act Art.14); supervision audit | `bc-sources/supervisor/compliance.md` |
| eval | Model-quality monitoring (EU AI Act Art.15); synthetic-PHI rule | `bc-sources/eval/compliance.md` |
| evidence | Retention; regulator-export; audit-chain | `bc-sources/evidence/compliance.md` |
| guardrails | Safety filter evidence (EU AI Act Art.9); content-safety rules | `bc-sources/guardrails/compliance.md` |
| providers | Provider risk management; credential isolation; cross-border data flow per Art.28+Art.16 | `bc-sources/providers/compliance.md` |

## Per-pack overlays

| Pack | Jurisdiction overlay | Notes |
|---|---|---|
| pack-kr | PIPA + KISA | M01 launch; OCI ap-seoul-1; cross-pack flow forbidden |
| pack-eu | GDPR + EU AI Act | Post-M01; per-DPO sign-off prior to promotion |
| pack-us | CCPA + CPRA | Post-M01 |
| pack-us-healthcare | HIPAA + state laws | 6y retention; BAA required with tenant |
| pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa | per-jurisdiction overlays | Post-M01 expansion |

## Audit cadence

- Internal: every 90 days, SOC 2 + ISO controls walked by ops-compliance.
- External: SOC 2 Type II annual audit by Big-4 firm; HIPAA assessment when
  pack-us-healthcare opens; ISO 27001 annual surveillance.
- Regulator-driven: ad-hoc per regulator-export API; EU AI Act conformity
  assessment when a capability crosses the high-risk threshold (declared
  via `evidence/regulator-export` profile).

## References

- ADR-0117: Data-residency + jurisdiction codes.
- ADR-0136 / ADR-0137: foundry topology authority.
- ADR-0028: Audit-chain Ed25519+Merkle.
- Bominal ADR-NNN: DSR cascade.
- `bc-sources/<bc>/compliance.md` — per-BC full mapping.
