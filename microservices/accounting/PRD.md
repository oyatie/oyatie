---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-accounting
microservice: accounting
status: foundation-slice-in-progress
date: 2026-05-23
owner_team: axis-enterprise
related_adrs: [ADR-0131, ADR-0132, ADR-0315]
canonical_machine_spec: specs/microservices/accounting.json
---

# PRD-accounting: Accounting microservice

The canonical machine-readable product spec is `specs/microservices/accounting.json`. This Markdown PRD exists only because `/specs/per-microservice-flat-layout.json` currently requires `microservices/<ms>/PRD.md` during the Markdown-retirement transition.

The Accounting microservice owns journal vouchers, period-open posting guards, payroll posting intake, VAT workflow evidence, AP approval gates, and close-evidence refusal. It composes with Payroll, HR, Workflow, Tenancy, and Audit Chain by refs/events rather than by becoming an tenant-rbac service.
