---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-payroll
microservice: payroll
status: foundation-slice-in-progress
date: 2026-05-23
owner_team: axis-enterprise
related_adrs: [ADR-0131, ADR-0132, ADR-0315]
canonical_machine_spec: specs/microservices/payroll.json
---

# PRD-payroll: Payroll microservice

The canonical machine-readable product spec is `specs/microservices/payroll.json`. This Markdown PRD exists only because `/specs/per-microservice-flat-layout.json` currently requires `microservices/<ms>/PRD.md` during the Markdown-retirement transition.

The Payroll microservice owns payroll runs, payees, wage ledgers, close promotion evidence, statutory-export evidence envelopes, and payroll-to-accounting journal drafts. It composes with HR and Accounting by typed refs and balanced drafts rather than by becoming an tenant-rbac service.
