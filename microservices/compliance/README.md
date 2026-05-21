---
doc_class: Reference
shape: Reference
length_cap: 300
microservice: compliance
companion_docs:
  - microservices/compliance/ARCHITECTURE.md
  - microservices/compliance/PRD.md
related_adrs:
  - ADR-0209
  - ADR-0212
inbound_citations:
  - docs/DOC-CATALOG.md
---

# compliance

The compliance substrate. Per-pack overlay registry + DPIA orchestration + breach-notification
workflow + regulator-audit-evidence surface + cell-certification attestation +
compliance-control mapping. In-house replacement for Drata / Vanta / Tugboat Logic / OneTrust /
AuditBoard / ServiceNow GRC / AWS Audit Manager.

## Entry points

- `PRD.md` — product requirements.
- `ARCHITECTURE.md` — architecture walkthrough.
- `threat-model.md` — STRIDE-style threats.
- `dpia.md` — Article 35 DPIA.
- `compliance.md` — pack overlays + control mapping.
- `runbooks/` — operational procedures.

## Hyperscaler precedents

AWS Audit Manager, Drata, Vanta, Tugboat Logic, OneTrust, AuditBoard, ServiceNow GRC.

## Bounded contexts

`pack-registry` / `dpia-orchestration` / `breach-notification-workflow` /
`regulator-audit-evidence` / `cell-certification-attestation` / `compliance-control-mapping`.

## Tenant Class Model

compliance follows ADR-0330. The service no longer models customer capability
levels. `tenant_class` is either `demo_trial` or `paid`; paid commercial shape
is composed from `billing_components` (`revenue_share`, `per_seat`,
`per_usage`). Pack activation, residency, regulator evidence, and air-gap
custody are expressed through `compliance_pack` and `cell_topology`, not a
customer ladder.

Canonical model: `docs/decisions/ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md`.
