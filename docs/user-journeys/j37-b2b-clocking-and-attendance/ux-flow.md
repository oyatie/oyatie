---
doc_class: User-Journey-UX-Flow
journey_id: j37-b2b-clocking-and-attendance
status: Proposed
date: 2026-05-20
authority_tier: 3
persona: Marcus Chen
locale: en-US
tenant_scope: acme-b2b
platform_microservice_count_authority: 45
marketplace_settlement_invariant: marketplace-settles-all-tenant-deals
contract_surfaces:
  - OpenAPI 3.2.0
  - AsyncAPI 3.1.0
  - proto3
  - BNF v4.1
  - ADR-0105 13-layer
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0244
  - ADR-0263
  - ADR-0273
  - ADR-0292
  - ADR-0297
  - ADR-0299
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/user-journeys/CATALOG-j126-j150-ecosystem.md
  - microservices/payments/PRD.md
  - microservices/identity/PRD.md
  - microservices/workflow-engine/PRD.md
  - microservices/ontology/PRD.md
  - microservices/messenger/PRD.md
  - microservices/mail/PRD.md
  - microservices/community/PRD.md
microservices_touched:
  - workplace-integration
  - connect
  - payments
  - identity
  - observability
journey_number: j37
benchmark: Workday Time Tracking plus ADP Workforce Now export pattern
---

# j37-b2b-clocking-and-attendance UX flow

Purpose: Screen-by-screen flow for Marcus Chen to let a team clock in and out with workplace geofence proof and export payroll rows to ADP.

## Screen 1: entry point
Primary user: Marcus Chen; locale: en-US; tenant: acme-b2b.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
workplace-integration: clock-in-geofence contributes one visible state or background status to the entry point screen.
workplace-integration: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workplace-integration: error state gives a recovery action, support reference, audit id, and retry budget.
workplace-integration: success state links receipt, audit seal, and data-export location where applicable.
connect: adp-payroll-export contributes one visible state or background status to the entry point screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
payments: payroll-ledger-hold contributes one visible state or background status to the entry point screen.
payments: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
payments: error state gives a recovery action, support reference, audit id, and retry budget.
payments: success state links receipt, audit seal, and data-export location where applicable.
identity: worker-shift-principal contributes one visible state or background status to the entry point screen.
identity: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
identity: error state gives a recovery action, support reference, audit id, and retry budget.
identity: success state links receipt, audit seal, and data-export location where applicable.
observability: attendance-slo-traces contributes one visible state or background status to the entry point screen.
observability: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
observability: error state gives a recovery action, support reference, audit id, and retry budget.
observability: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: entry point passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: entry point uses en-US strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 2: tenant context switcher
Primary user: Marcus Chen; locale: en-US; tenant: acme-b2b.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
workplace-integration: clock-in-geofence contributes one visible state or background status to the tenant context switcher screen.
workplace-integration: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workplace-integration: error state gives a recovery action, support reference, audit id, and retry budget.
workplace-integration: success state links receipt, audit seal, and data-export location where applicable.
connect: adp-payroll-export contributes one visible state or background status to the tenant context switcher screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
payments: payroll-ledger-hold contributes one visible state or background status to the tenant context switcher screen.
payments: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
payments: error state gives a recovery action, support reference, audit id, and retry budget.
payments: success state links receipt, audit seal, and data-export location where applicable.
identity: worker-shift-principal contributes one visible state or background status to the tenant context switcher screen.
identity: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
identity: error state gives a recovery action, support reference, audit id, and retry budget.
identity: success state links receipt, audit seal, and data-export location where applicable.
observability: attendance-slo-traces contributes one visible state or background status to the tenant context switcher screen.
observability: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
observability: error state gives a recovery action, support reference, audit id, and retry budget.
observability: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: tenant context switcher passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: tenant context switcher uses en-US strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 3: identity and recovery confirmation
Primary user: Marcus Chen; locale: en-US; tenant: acme-b2b.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
workplace-integration: clock-in-geofence contributes one visible state or background status to the identity and recovery confirmation screen.
workplace-integration: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workplace-integration: error state gives a recovery action, support reference, audit id, and retry budget.
workplace-integration: success state links receipt, audit seal, and data-export location where applicable.
connect: adp-payroll-export contributes one visible state or background status to the identity and recovery confirmation screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
payments: payroll-ledger-hold contributes one visible state or background status to the identity and recovery confirmation screen.
payments: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
payments: error state gives a recovery action, support reference, audit id, and retry budget.
payments: success state links receipt, audit seal, and data-export location where applicable.
identity: worker-shift-principal contributes one visible state or background status to the identity and recovery confirmation screen.
identity: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
identity: error state gives a recovery action, support reference, audit id, and retry budget.
identity: success state links receipt, audit seal, and data-export location where applicable.
observability: attendance-slo-traces contributes one visible state or background status to the identity and recovery confirmation screen.
observability: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
observability: error state gives a recovery action, support reference, audit id, and retry budget.
observability: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: identity and recovery confirmation passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: identity and recovery confirmation uses en-US strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 4: primary work canvas
Primary user: Marcus Chen; locale: en-US; tenant: acme-b2b.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
workplace-integration: clock-in-geofence contributes one visible state or background status to the primary work canvas screen.
workplace-integration: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workplace-integration: error state gives a recovery action, support reference, audit id, and retry budget.
workplace-integration: success state links receipt, audit seal, and data-export location where applicable.
connect: adp-payroll-export contributes one visible state or background status to the primary work canvas screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
payments: payroll-ledger-hold contributes one visible state or background status to the primary work canvas screen.
payments: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
payments: error state gives a recovery action, support reference, audit id, and retry budget.
payments: success state links receipt, audit seal, and data-export location where applicable.
identity: worker-shift-principal contributes one visible state or background status to the primary work canvas screen.
identity: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
identity: error state gives a recovery action, support reference, audit id, and retry budget.
identity: success state links receipt, audit seal, and data-export location where applicable.
observability: attendance-slo-traces contributes one visible state or background status to the primary work canvas screen.
observability: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
observability: error state gives a recovery action, support reference, audit id, and retry budget.
observability: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: primary work canvas passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: primary work canvas uses en-US strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 5: review panel
Primary user: Marcus Chen; locale: en-US; tenant: acme-b2b.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
workplace-integration: clock-in-geofence contributes one visible state or background status to the review panel screen.
workplace-integration: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workplace-integration: error state gives a recovery action, support reference, audit id, and retry budget.
workplace-integration: success state links receipt, audit seal, and data-export location where applicable.
connect: adp-payroll-export contributes one visible state or background status to the review panel screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
payments: payroll-ledger-hold contributes one visible state or background status to the review panel screen.
payments: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
payments: error state gives a recovery action, support reference, audit id, and retry budget.
payments: success state links receipt, audit seal, and data-export location where applicable.
identity: worker-shift-principal contributes one visible state or background status to the review panel screen.
identity: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
identity: error state gives a recovery action, support reference, audit id, and retry budget.
identity: success state links receipt, audit seal, and data-export location where applicable.
observability: attendance-slo-traces contributes one visible state or background status to the review panel screen.
observability: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
observability: error state gives a recovery action, support reference, audit id, and retry budget.
observability: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: review panel passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: review panel uses en-US strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 6: approval or confirmation panel
Primary user: Marcus Chen; locale: en-US; tenant: acme-b2b.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
workplace-integration: clock-in-geofence contributes one visible state or background status to the approval or confirmation panel screen.
workplace-integration: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workplace-integration: error state gives a recovery action, support reference, audit id, and retry budget.
workplace-integration: success state links receipt, audit seal, and data-export location where applicable.
connect: adp-payroll-export contributes one visible state or background status to the approval or confirmation panel screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
payments: payroll-ledger-hold contributes one visible state or background status to the approval or confirmation panel screen.
payments: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
payments: error state gives a recovery action, support reference, audit id, and retry budget.
payments: success state links receipt, audit seal, and data-export location where applicable.
identity: worker-shift-principal contributes one visible state or background status to the approval or confirmation panel screen.
identity: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
identity: error state gives a recovery action, support reference, audit id, and retry budget.
identity: success state links receipt, audit seal, and data-export location where applicable.
observability: attendance-slo-traces contributes one visible state or background status to the approval or confirmation panel screen.
observability: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
observability: error state gives a recovery action, support reference, audit id, and retry budget.
observability: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: approval or confirmation panel passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: approval or confirmation panel uses en-US strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 7: counterparty or provider handoff
Primary user: Marcus Chen; locale: en-US; tenant: acme-b2b.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
workplace-integration: clock-in-geofence contributes one visible state or background status to the counterparty or provider handoff screen.
workplace-integration: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workplace-integration: error state gives a recovery action, support reference, audit id, and retry budget.
workplace-integration: success state links receipt, audit seal, and data-export location where applicable.
connect: adp-payroll-export contributes one visible state or background status to the counterparty or provider handoff screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
payments: payroll-ledger-hold contributes one visible state or background status to the counterparty or provider handoff screen.
payments: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
payments: error state gives a recovery action, support reference, audit id, and retry budget.
payments: success state links receipt, audit seal, and data-export location where applicable.
identity: worker-shift-principal contributes one visible state or background status to the counterparty or provider handoff screen.
identity: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
identity: error state gives a recovery action, support reference, audit id, and retry budget.
identity: success state links receipt, audit seal, and data-export location where applicable.
observability: attendance-slo-traces contributes one visible state or background status to the counterparty or provider handoff screen.
observability: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
observability: error state gives a recovery action, support reference, audit id, and retry budget.
observability: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: counterparty or provider handoff passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: counterparty or provider handoff uses en-US strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 8: settlement and notification panel
Primary user: Marcus Chen; locale: en-US; tenant: acme-b2b.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
workplace-integration: clock-in-geofence contributes one visible state or background status to the settlement and notification panel screen.
workplace-integration: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workplace-integration: error state gives a recovery action, support reference, audit id, and retry budget.
workplace-integration: success state links receipt, audit seal, and data-export location where applicable.
connect: adp-payroll-export contributes one visible state or background status to the settlement and notification panel screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
payments: payroll-ledger-hold contributes one visible state or background status to the settlement and notification panel screen.
payments: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
payments: error state gives a recovery action, support reference, audit id, and retry budget.
payments: success state links receipt, audit seal, and data-export location where applicable.
identity: worker-shift-principal contributes one visible state or background status to the settlement and notification panel screen.
identity: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
identity: error state gives a recovery action, support reference, audit id, and retry budget.
identity: success state links receipt, audit seal, and data-export location where applicable.
observability: attendance-slo-traces contributes one visible state or background status to the settlement and notification panel screen.
observability: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
observability: error state gives a recovery action, support reference, audit id, and retry budget.
observability: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: settlement and notification panel passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: settlement and notification panel uses en-US strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 9: audit detail panel
Primary user: Marcus Chen; locale: en-US; tenant: acme-b2b.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
workplace-integration: clock-in-geofence contributes one visible state or background status to the audit detail panel screen.
workplace-integration: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workplace-integration: error state gives a recovery action, support reference, audit id, and retry budget.
workplace-integration: success state links receipt, audit seal, and data-export location where applicable.
connect: adp-payroll-export contributes one visible state or background status to the audit detail panel screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
payments: payroll-ledger-hold contributes one visible state or background status to the audit detail panel screen.
payments: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
payments: error state gives a recovery action, support reference, audit id, and retry budget.
payments: success state links receipt, audit seal, and data-export location where applicable.
identity: worker-shift-principal contributes one visible state or background status to the audit detail panel screen.
identity: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
identity: error state gives a recovery action, support reference, audit id, and retry budget.
identity: success state links receipt, audit seal, and data-export location where applicable.
observability: attendance-slo-traces contributes one visible state or background status to the audit detail panel screen.
observability: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
observability: error state gives a recovery action, support reference, audit id, and retry budget.
observability: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: audit detail panel passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: audit detail panel uses en-US strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 10: error recovery panel
Primary user: Marcus Chen; locale: en-US; tenant: acme-b2b.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
workplace-integration: clock-in-geofence contributes one visible state or background status to the error recovery panel screen.
workplace-integration: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workplace-integration: error state gives a recovery action, support reference, audit id, and retry budget.
workplace-integration: success state links receipt, audit seal, and data-export location where applicable.
connect: adp-payroll-export contributes one visible state or background status to the error recovery panel screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
payments: payroll-ledger-hold contributes one visible state or background status to the error recovery panel screen.
payments: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
payments: error state gives a recovery action, support reference, audit id, and retry budget.
payments: success state links receipt, audit seal, and data-export location where applicable.
identity: worker-shift-principal contributes one visible state or background status to the error recovery panel screen.
identity: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
identity: error state gives a recovery action, support reference, audit id, and retry budget.
identity: success state links receipt, audit seal, and data-export location where applicable.
observability: attendance-slo-traces contributes one visible state or background status to the error recovery panel screen.
observability: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
observability: error state gives a recovery action, support reference, audit id, and retry budget.
observability: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: error recovery panel passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: error recovery panel uses en-US strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 11: mobile compact view
Primary user: Marcus Chen; locale: en-US; tenant: acme-b2b.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
workplace-integration: clock-in-geofence contributes one visible state or background status to the mobile compact view screen.
workplace-integration: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workplace-integration: error state gives a recovery action, support reference, audit id, and retry budget.
workplace-integration: success state links receipt, audit seal, and data-export location where applicable.
connect: adp-payroll-export contributes one visible state or background status to the mobile compact view screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
payments: payroll-ledger-hold contributes one visible state or background status to the mobile compact view screen.
payments: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
payments: error state gives a recovery action, support reference, audit id, and retry budget.
payments: success state links receipt, audit seal, and data-export location where applicable.
identity: worker-shift-principal contributes one visible state or background status to the mobile compact view screen.
identity: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
identity: error state gives a recovery action, support reference, audit id, and retry budget.
identity: success state links receipt, audit seal, and data-export location where applicable.
observability: attendance-slo-traces contributes one visible state or background status to the mobile compact view screen.
observability: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
observability: error state gives a recovery action, support reference, audit id, and retry budget.
observability: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: mobile compact view passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: mobile compact view uses en-US strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 12: completion receipt
Primary user: Marcus Chen; locale: en-US; tenant: acme-b2b.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
workplace-integration: clock-in-geofence contributes one visible state or background status to the completion receipt screen.
workplace-integration: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workplace-integration: error state gives a recovery action, support reference, audit id, and retry budget.
workplace-integration: success state links receipt, audit seal, and data-export location where applicable.
connect: adp-payroll-export contributes one visible state or background status to the completion receipt screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
payments: payroll-ledger-hold contributes one visible state or background status to the completion receipt screen.
payments: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
payments: error state gives a recovery action, support reference, audit id, and retry budget.
payments: success state links receipt, audit seal, and data-export location where applicable.
identity: worker-shift-principal contributes one visible state or background status to the completion receipt screen.
identity: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
identity: error state gives a recovery action, support reference, audit id, and retry budget.
identity: success state links receipt, audit seal, and data-export location where applicable.
observability: attendance-slo-traces contributes one visible state or background status to the completion receipt screen.
observability: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
observability: error state gives a recovery action, support reference, audit id, and retry budget.
observability: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: completion receipt passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: completion receipt uses en-US strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Interaction state matrix
UX check 1: on entry point, workplace-integration (clock-in-geofence) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 2: on tenant context switcher, connect (adp-payroll-export) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 3: on identity and recovery confirmation, payments (payroll-ledger-hold) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 4: on primary work canvas, identity (worker-shift-principal) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 5: on review panel, observability (attendance-slo-traces) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 6: on approval or confirmation panel, workplace-integration (clock-in-geofence) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 7: on counterparty or provider handoff, connect (adp-payroll-export) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 8: on settlement and notification panel, payments (payroll-ledger-hold) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 9: on audit detail panel, identity (worker-shift-principal) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 10: on error recovery panel, observability (attendance-slo-traces) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 11: on mobile compact view, workplace-integration (clock-in-geofence) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 12: on completion receipt, connect (adp-payroll-export) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 13: on entry point, payments (payroll-ledger-hold) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 14: on tenant context switcher, identity (worker-shift-principal) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 15: on identity and recovery confirmation, observability (attendance-slo-traces) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 16: on primary work canvas, workplace-integration (clock-in-geofence) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 17: on review panel, connect (adp-payroll-export) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 18: on approval or confirmation panel, payments (payroll-ledger-hold) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 19: on counterparty or provider handoff, identity (worker-shift-principal) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 20: on settlement and notification panel, observability (attendance-slo-traces) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 21: on audit detail panel, workplace-integration (clock-in-geofence) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 22: on error recovery panel, connect (adp-payroll-export) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 23: on mobile compact view, payments (payroll-ledger-hold) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 24: on completion receipt, identity (worker-shift-principal) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 25: on entry point, observability (attendance-slo-traces) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 26: on tenant context switcher, workplace-integration (clock-in-geofence) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 27: on identity and recovery confirmation, connect (adp-payroll-export) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 28: on primary work canvas, payments (payroll-ledger-hold) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 29: on review panel, identity (worker-shift-principal) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 30: on approval or confirmation panel, observability (attendance-slo-traces) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 31: on counterparty or provider handoff, workplace-integration (clock-in-geofence) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 32: on settlement and notification panel, connect (adp-payroll-export) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 33: on audit detail panel, payments (payroll-ledger-hold) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 34: on error recovery panel, identity (worker-shift-principal) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 35: on mobile compact view, observability (attendance-slo-traces) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 36: on completion receipt, workplace-integration (clock-in-geofence) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 37: on entry point, connect (adp-payroll-export) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 38: on tenant context switcher, payments (payroll-ledger-hold) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 39: on identity and recovery confirmation, identity (worker-shift-principal) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 40: on primary work canvas, observability (attendance-slo-traces) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 41: on review panel, workplace-integration (clock-in-geofence) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 42: on approval or confirmation panel, connect (adp-payroll-export) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 43: on counterparty or provider handoff, payments (payroll-ledger-hold) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 44: on settlement and notification panel, identity (worker-shift-principal) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 45: on audit detail panel, observability (attendance-slo-traces) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 46: on error recovery panel, workplace-integration (clock-in-geofence) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 47: on mobile compact view, connect (adp-payroll-export) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 48: on completion receipt, payments (payroll-ledger-hold) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 49: on entry point, identity (worker-shift-principal) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 50: on tenant context switcher, observability (attendance-slo-traces) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 51: on identity and recovery confirmation, workplace-integration (clock-in-geofence) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 52: on primary work canvas, connect (adp-payroll-export) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 53: on review panel, payments (payroll-ledger-hold) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 54: on approval or confirmation panel, identity (worker-shift-principal) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 55: on counterparty or provider handoff, observability (attendance-slo-traces) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 56: on settlement and notification panel, workplace-integration (clock-in-geofence) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 57: on audit detail panel, connect (adp-payroll-export) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
