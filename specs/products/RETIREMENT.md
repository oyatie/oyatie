# `specs/products/` — RETIRED 2026-05-18

The `specs/products/` directory is retired. All per-µservice spec files have
flattened into `/specs/microservices/<ms>.json` per ADR-0132 (no-suite
forward-policy) + ADR-0131 (per-µservice flat layout) + the 2026-05-18 user
directive *"retire products terminology"*.

## What moved

| Old path | New path |
|---|---|
| `specs/products/ontology.json` | `specs/microservices/ontology.json` |
| `specs/products/workflow.json` | `specs/microservices/workflow.json` |
| `specs/products/workflow-studio.json` | `specs/microservices/workflow-studio.json` |
| `specs/products/connect/mail.json` | `specs/microservices/mail.json` |
| `specs/products/connect/messenger.json` | `specs/microservices/messenger.json` |
| `specs/products/connect/calendar.json` | `specs/microservices/calendar.json` |
| `specs/products/connect/suite.json` | `specs/microservices/connect-suite.json` |
| `specs/products/enterprise/hr.json` | `specs/microservices/hr.json` |
| `specs/products/enterprise/payroll.json` | `specs/microservices/payroll.json` |
| `specs/products/enterprise/accounting.json` | `specs/microservices/accounting.json` |
| `specs/products/enterprise/suite.json` | `specs/microservices/enterprise-suite.json` |

## Authority

- ADR-0131 — per-µservice flat layout.
- ADR-0132 — no-suite forward-policy.
- ADR-0135 — Connect super-app expansion (Connect umbrella retires; per-µservice
  specs live at `/specs/microservices/`).
- User directive 2026-05-18: "retire products terminology".

This directory is preserved as a tombstone only. Do not add new files here.
