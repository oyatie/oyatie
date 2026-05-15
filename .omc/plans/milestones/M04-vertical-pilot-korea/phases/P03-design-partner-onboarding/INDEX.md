---
purpose: Onboard the first KR Group design-partner tenant; author tenant-specific workflows; activate Foundry agents on tenant data.
---

---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M04-P03
title: Design-Partner Tenant Onboarding + Workflow Authoring
status: stub
purpose: Onboard the first KR Group design-partner tenant; author tenant-specific workflows; activate Foundry agents on tenant data.
---

# M04-P03 — Design-Partner Onboarding

## Purpose
Per [`../../../../../docs/PRD.md`](../../../../../docs/PRD.md) §4.1 row "KR Group Payroll tenants live: ≥ 3 design-partner groups".

## Acceptance
- First KR Group tenant onboarded via `tenant.create` SPEC §2 row with KR pack binding.
- Tenant-specific workflows authored via Workflow Studio (SaaS axis); ≥ 5 capabilities invoked end-to-end per typical business day.
- Foundry agents activated under autonomy ceiling T1-T3 (T4 disabled by default for actuation).
- Consent receipts emitted per [`../../../../../docs/SPEC.md`](../../../../../docs/SPEC.md) §2 `consent.receipt.emit`.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Tenant onboarding via Workflow Studio | stub | [`IP-001-tenant-onboarding.md`](IP-001-tenant-onboarding.md) |
| IP-002 | Tenant-specific workflow authoring | stub | [`IP-002-tenant-workflows.md`](IP-002-tenant-workflows.md) |
| IP-003 | Foundry agents activation under autonomy ceiling | stub | [`IP-003-foundry-agents-activation.md`](IP-003-foundry-agents-activation.md) |

## Estimated parallelism
3 agents; gtm-customer-success runs onboarding while axis-foundry activates agents while axis-saas configures Workflow Studio.

## Symbols-touched
`crates/oya-platform-tenant-app::onboard`, `crates/oya-saas-workflow-app::author`, `crates/oya-foundry-capability-registry-app::activate`.

## Agent-handoff
```
icm store -t context-oyatie -c "M04-P03 complete: first KR Group tenant live; workflows authored; Foundry agents active" -i critical -k "M04,P03,design-partner,kr-group,complete"
```
