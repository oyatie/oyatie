---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-hr
microservice: hr
status: foundation-slice-in-progress
date: 2026-05-23
owner_team: axis-enterprise
related_adrs: [ADR-0131, ADR-0132, ADR-0315]
canonical_machine_spec: specs/microservices/hr.json
---

# PRD-hr: HR microservice

The canonical machine-readable product spec is `specs/microservices/hr.json`. This Markdown PRD exists only because `/specs/per-microservice-flat-layout.json` currently requires `microservices/<ms>/PRD.md` during the Markdown-retirement transition.

The HR microservice owns employee/employment records, organization and manager relationships, HR lifecycle evidence, and labor-compliance obligation detection. It composes with Payroll, Accounting, Workflow, Ontology, Tenancy, and Audit Chain by typed refs/events rather than by becoming an enterprise-suite service.
