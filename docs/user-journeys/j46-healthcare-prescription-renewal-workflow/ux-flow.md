---
doc_class: User-Journey-UX-Flow
journey_id: j46-healthcare-prescription-renewal-workflow
status: Proposed
date: 2026-05-20
authority_tier: 3
persona: Yejin Park
locale: ko-KR
tenant_scope: yejin-personal-health
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
  - workflow-studio
  - workflow-engine
  - mail
  - identity
  - connect
  - compliance
journey_number: j46
benchmark: Epic MyChart refill request plus pharmacy eRx routing pattern
---

# j46-healthcare-prescription-renewal-workflow UX flow

Purpose: Screen-by-screen flow for Yejin Park to request an Rx renewal in Workflow Studio, route to a prescribing doctor, then to a pharmacy.

## Screen 1: entry point
Primary user: Yejin Park; locale: ko-KR; tenant: yejin-personal-health.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
workflow-studio: rx-renewal-template contributes one visible state or background status to the entry point screen.
workflow-studio: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workflow-studio: error state gives a recovery action, support reference, audit id, and retry budget.
workflow-studio: success state links receipt, audit seal, and data-export location where applicable.
workflow-engine: prescriber-routing contributes one visible state or background status to the entry point screen.
workflow-engine: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workflow-engine: error state gives a recovery action, support reference, audit id, and retry budget.
workflow-engine: success state links receipt, audit seal, and data-export location where applicable.
mail: rx-status-messaging contributes one visible state or background status to the entry point screen.
mail: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
mail: error state gives a recovery action, support reference, audit id, and retry budget.
mail: success state links receipt, audit seal, and data-export location where applicable.
identity: patient-prescriber-resolution contributes one visible state or background status to the entry point screen.
identity: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
identity: error state gives a recovery action, support reference, audit id, and retry budget.
identity: success state links receipt, audit seal, and data-export location where applicable.
connect: pharmacy-adapter contributes one visible state or background status to the entry point screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
compliance: rx-overlay contributes one visible state or background status to the entry point screen.
compliance: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
compliance: error state gives a recovery action, support reference, audit id, and retry budget.
compliance: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: entry point passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: entry point uses ko-KR strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 2: tenant context switcher
Primary user: Yejin Park; locale: ko-KR; tenant: yejin-personal-health.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
workflow-studio: rx-renewal-template contributes one visible state or background status to the tenant context switcher screen.
workflow-studio: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workflow-studio: error state gives a recovery action, support reference, audit id, and retry budget.
workflow-studio: success state links receipt, audit seal, and data-export location where applicable.
workflow-engine: prescriber-routing contributes one visible state or background status to the tenant context switcher screen.
workflow-engine: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workflow-engine: error state gives a recovery action, support reference, audit id, and retry budget.
workflow-engine: success state links receipt, audit seal, and data-export location where applicable.
mail: rx-status-messaging contributes one visible state or background status to the tenant context switcher screen.
mail: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
mail: error state gives a recovery action, support reference, audit id, and retry budget.
mail: success state links receipt, audit seal, and data-export location where applicable.
identity: patient-prescriber-resolution contributes one visible state or background status to the tenant context switcher screen.
identity: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
identity: error state gives a recovery action, support reference, audit id, and retry budget.
identity: success state links receipt, audit seal, and data-export location where applicable.
connect: pharmacy-adapter contributes one visible state or background status to the tenant context switcher screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
compliance: rx-overlay contributes one visible state or background status to the tenant context switcher screen.
compliance: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
compliance: error state gives a recovery action, support reference, audit id, and retry budget.
compliance: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: tenant context switcher passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: tenant context switcher uses ko-KR strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 3: identity and recovery confirmation
Primary user: Yejin Park; locale: ko-KR; tenant: yejin-personal-health.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
workflow-studio: rx-renewal-template contributes one visible state or background status to the identity and recovery confirmation screen.
workflow-studio: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workflow-studio: error state gives a recovery action, support reference, audit id, and retry budget.
workflow-studio: success state links receipt, audit seal, and data-export location where applicable.
workflow-engine: prescriber-routing contributes one visible state or background status to the identity and recovery confirmation screen.
workflow-engine: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workflow-engine: error state gives a recovery action, support reference, audit id, and retry budget.
workflow-engine: success state links receipt, audit seal, and data-export location where applicable.
mail: rx-status-messaging contributes one visible state or background status to the identity and recovery confirmation screen.
mail: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
mail: error state gives a recovery action, support reference, audit id, and retry budget.
mail: success state links receipt, audit seal, and data-export location where applicable.
identity: patient-prescriber-resolution contributes one visible state or background status to the identity and recovery confirmation screen.
identity: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
identity: error state gives a recovery action, support reference, audit id, and retry budget.
identity: success state links receipt, audit seal, and data-export location where applicable.
connect: pharmacy-adapter contributes one visible state or background status to the identity and recovery confirmation screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
compliance: rx-overlay contributes one visible state or background status to the identity and recovery confirmation screen.
compliance: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
compliance: error state gives a recovery action, support reference, audit id, and retry budget.
compliance: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: identity and recovery confirmation passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: identity and recovery confirmation uses ko-KR strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 4: primary work canvas
Primary user: Yejin Park; locale: ko-KR; tenant: yejin-personal-health.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
workflow-studio: rx-renewal-template contributes one visible state or background status to the primary work canvas screen.
workflow-studio: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workflow-studio: error state gives a recovery action, support reference, audit id, and retry budget.
workflow-studio: success state links receipt, audit seal, and data-export location where applicable.
workflow-engine: prescriber-routing contributes one visible state or background status to the primary work canvas screen.
workflow-engine: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workflow-engine: error state gives a recovery action, support reference, audit id, and retry budget.
workflow-engine: success state links receipt, audit seal, and data-export location where applicable.
mail: rx-status-messaging contributes one visible state or background status to the primary work canvas screen.
mail: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
mail: error state gives a recovery action, support reference, audit id, and retry budget.
mail: success state links receipt, audit seal, and data-export location where applicable.
identity: patient-prescriber-resolution contributes one visible state or background status to the primary work canvas screen.
identity: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
identity: error state gives a recovery action, support reference, audit id, and retry budget.
identity: success state links receipt, audit seal, and data-export location where applicable.
connect: pharmacy-adapter contributes one visible state or background status to the primary work canvas screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
compliance: rx-overlay contributes one visible state or background status to the primary work canvas screen.
compliance: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
compliance: error state gives a recovery action, support reference, audit id, and retry budget.
compliance: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: primary work canvas passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: primary work canvas uses ko-KR strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 5: review panel
Primary user: Yejin Park; locale: ko-KR; tenant: yejin-personal-health.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
workflow-studio: rx-renewal-template contributes one visible state or background status to the review panel screen.
workflow-studio: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workflow-studio: error state gives a recovery action, support reference, audit id, and retry budget.
workflow-studio: success state links receipt, audit seal, and data-export location where applicable.
workflow-engine: prescriber-routing contributes one visible state or background status to the review panel screen.
workflow-engine: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workflow-engine: error state gives a recovery action, support reference, audit id, and retry budget.
workflow-engine: success state links receipt, audit seal, and data-export location where applicable.
mail: rx-status-messaging contributes one visible state or background status to the review panel screen.
mail: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
mail: error state gives a recovery action, support reference, audit id, and retry budget.
mail: success state links receipt, audit seal, and data-export location where applicable.
identity: patient-prescriber-resolution contributes one visible state or background status to the review panel screen.
identity: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
identity: error state gives a recovery action, support reference, audit id, and retry budget.
identity: success state links receipt, audit seal, and data-export location where applicable.
connect: pharmacy-adapter contributes one visible state or background status to the review panel screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
compliance: rx-overlay contributes one visible state or background status to the review panel screen.
compliance: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
compliance: error state gives a recovery action, support reference, audit id, and retry budget.
compliance: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: review panel passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: review panel uses ko-KR strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 6: approval or confirmation panel
Primary user: Yejin Park; locale: ko-KR; tenant: yejin-personal-health.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
workflow-studio: rx-renewal-template contributes one visible state or background status to the approval or confirmation panel screen.
workflow-studio: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workflow-studio: error state gives a recovery action, support reference, audit id, and retry budget.
workflow-studio: success state links receipt, audit seal, and data-export location where applicable.
workflow-engine: prescriber-routing contributes one visible state or background status to the approval or confirmation panel screen.
workflow-engine: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workflow-engine: error state gives a recovery action, support reference, audit id, and retry budget.
workflow-engine: success state links receipt, audit seal, and data-export location where applicable.
mail: rx-status-messaging contributes one visible state or background status to the approval or confirmation panel screen.
mail: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
mail: error state gives a recovery action, support reference, audit id, and retry budget.
mail: success state links receipt, audit seal, and data-export location where applicable.
identity: patient-prescriber-resolution contributes one visible state or background status to the approval or confirmation panel screen.
identity: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
identity: error state gives a recovery action, support reference, audit id, and retry budget.
identity: success state links receipt, audit seal, and data-export location where applicable.
connect: pharmacy-adapter contributes one visible state or background status to the approval or confirmation panel screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
compliance: rx-overlay contributes one visible state or background status to the approval or confirmation panel screen.
compliance: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
compliance: error state gives a recovery action, support reference, audit id, and retry budget.
compliance: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: approval or confirmation panel passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: approval or confirmation panel uses ko-KR strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 7: counterparty or provider handoff
Primary user: Yejin Park; locale: ko-KR; tenant: yejin-personal-health.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
workflow-studio: rx-renewal-template contributes one visible state or background status to the counterparty or provider handoff screen.
workflow-studio: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workflow-studio: error state gives a recovery action, support reference, audit id, and retry budget.
workflow-studio: success state links receipt, audit seal, and data-export location where applicable.
workflow-engine: prescriber-routing contributes one visible state or background status to the counterparty or provider handoff screen.
workflow-engine: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workflow-engine: error state gives a recovery action, support reference, audit id, and retry budget.
workflow-engine: success state links receipt, audit seal, and data-export location where applicable.
mail: rx-status-messaging contributes one visible state or background status to the counterparty or provider handoff screen.
mail: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
mail: error state gives a recovery action, support reference, audit id, and retry budget.
mail: success state links receipt, audit seal, and data-export location where applicable.
identity: patient-prescriber-resolution contributes one visible state or background status to the counterparty or provider handoff screen.
identity: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
identity: error state gives a recovery action, support reference, audit id, and retry budget.
identity: success state links receipt, audit seal, and data-export location where applicable.
connect: pharmacy-adapter contributes one visible state or background status to the counterparty or provider handoff screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
compliance: rx-overlay contributes one visible state or background status to the counterparty or provider handoff screen.
compliance: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
compliance: error state gives a recovery action, support reference, audit id, and retry budget.
compliance: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: counterparty or provider handoff passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: counterparty or provider handoff uses ko-KR strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 8: settlement and notification panel
Primary user: Yejin Park; locale: ko-KR; tenant: yejin-personal-health.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
workflow-studio: rx-renewal-template contributes one visible state or background status to the settlement and notification panel screen.
workflow-studio: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workflow-studio: error state gives a recovery action, support reference, audit id, and retry budget.
workflow-studio: success state links receipt, audit seal, and data-export location where applicable.
workflow-engine: prescriber-routing contributes one visible state or background status to the settlement and notification panel screen.
workflow-engine: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workflow-engine: error state gives a recovery action, support reference, audit id, and retry budget.
workflow-engine: success state links receipt, audit seal, and data-export location where applicable.
mail: rx-status-messaging contributes one visible state or background status to the settlement and notification panel screen.
mail: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
mail: error state gives a recovery action, support reference, audit id, and retry budget.
mail: success state links receipt, audit seal, and data-export location where applicable.
identity: patient-prescriber-resolution contributes one visible state or background status to the settlement and notification panel screen.
identity: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
identity: error state gives a recovery action, support reference, audit id, and retry budget.
identity: success state links receipt, audit seal, and data-export location where applicable.
connect: pharmacy-adapter contributes one visible state or background status to the settlement and notification panel screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
compliance: rx-overlay contributes one visible state or background status to the settlement and notification panel screen.
compliance: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
compliance: error state gives a recovery action, support reference, audit id, and retry budget.
compliance: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: settlement and notification panel passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: settlement and notification panel uses ko-KR strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 9: audit detail panel
Primary user: Yejin Park; locale: ko-KR; tenant: yejin-personal-health.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
workflow-studio: rx-renewal-template contributes one visible state or background status to the audit detail panel screen.
workflow-studio: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workflow-studio: error state gives a recovery action, support reference, audit id, and retry budget.
workflow-studio: success state links receipt, audit seal, and data-export location where applicable.
workflow-engine: prescriber-routing contributes one visible state or background status to the audit detail panel screen.
workflow-engine: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workflow-engine: error state gives a recovery action, support reference, audit id, and retry budget.
workflow-engine: success state links receipt, audit seal, and data-export location where applicable.
mail: rx-status-messaging contributes one visible state or background status to the audit detail panel screen.
mail: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
mail: error state gives a recovery action, support reference, audit id, and retry budget.
mail: success state links receipt, audit seal, and data-export location where applicable.
identity: patient-prescriber-resolution contributes one visible state or background status to the audit detail panel screen.
identity: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
identity: error state gives a recovery action, support reference, audit id, and retry budget.
identity: success state links receipt, audit seal, and data-export location where applicable.
connect: pharmacy-adapter contributes one visible state or background status to the audit detail panel screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
compliance: rx-overlay contributes one visible state or background status to the audit detail panel screen.
compliance: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
compliance: error state gives a recovery action, support reference, audit id, and retry budget.
compliance: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: audit detail panel passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: audit detail panel uses ko-KR strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 10: error recovery panel
Primary user: Yejin Park; locale: ko-KR; tenant: yejin-personal-health.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
workflow-studio: rx-renewal-template contributes one visible state or background status to the error recovery panel screen.
workflow-studio: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workflow-studio: error state gives a recovery action, support reference, audit id, and retry budget.
workflow-studio: success state links receipt, audit seal, and data-export location where applicable.
workflow-engine: prescriber-routing contributes one visible state or background status to the error recovery panel screen.
workflow-engine: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workflow-engine: error state gives a recovery action, support reference, audit id, and retry budget.
workflow-engine: success state links receipt, audit seal, and data-export location where applicable.
mail: rx-status-messaging contributes one visible state or background status to the error recovery panel screen.
mail: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
mail: error state gives a recovery action, support reference, audit id, and retry budget.
mail: success state links receipt, audit seal, and data-export location where applicable.
identity: patient-prescriber-resolution contributes one visible state or background status to the error recovery panel screen.
identity: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
identity: error state gives a recovery action, support reference, audit id, and retry budget.
identity: success state links receipt, audit seal, and data-export location where applicable.
connect: pharmacy-adapter contributes one visible state or background status to the error recovery panel screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
compliance: rx-overlay contributes one visible state or background status to the error recovery panel screen.
compliance: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
compliance: error state gives a recovery action, support reference, audit id, and retry budget.
compliance: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: error recovery panel passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: error recovery panel uses ko-KR strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 11: mobile compact view
Primary user: Yejin Park; locale: ko-KR; tenant: yejin-personal-health.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
workflow-studio: rx-renewal-template contributes one visible state or background status to the mobile compact view screen.
workflow-studio: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workflow-studio: error state gives a recovery action, support reference, audit id, and retry budget.
workflow-studio: success state links receipt, audit seal, and data-export location where applicable.
workflow-engine: prescriber-routing contributes one visible state or background status to the mobile compact view screen.
workflow-engine: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workflow-engine: error state gives a recovery action, support reference, audit id, and retry budget.
workflow-engine: success state links receipt, audit seal, and data-export location where applicable.
mail: rx-status-messaging contributes one visible state or background status to the mobile compact view screen.
mail: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
mail: error state gives a recovery action, support reference, audit id, and retry budget.
mail: success state links receipt, audit seal, and data-export location where applicable.
identity: patient-prescriber-resolution contributes one visible state or background status to the mobile compact view screen.
identity: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
identity: error state gives a recovery action, support reference, audit id, and retry budget.
identity: success state links receipt, audit seal, and data-export location where applicable.
connect: pharmacy-adapter contributes one visible state or background status to the mobile compact view screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
compliance: rx-overlay contributes one visible state or background status to the mobile compact view screen.
compliance: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
compliance: error state gives a recovery action, support reference, audit id, and retry budget.
compliance: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: mobile compact view passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: mobile compact view uses ko-KR strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 12: completion receipt
Primary user: Yejin Park; locale: ko-KR; tenant: yejin-personal-health.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
workflow-studio: rx-renewal-template contributes one visible state or background status to the completion receipt screen.
workflow-studio: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workflow-studio: error state gives a recovery action, support reference, audit id, and retry budget.
workflow-studio: success state links receipt, audit seal, and data-export location where applicable.
workflow-engine: prescriber-routing contributes one visible state or background status to the completion receipt screen.
workflow-engine: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
workflow-engine: error state gives a recovery action, support reference, audit id, and retry budget.
workflow-engine: success state links receipt, audit seal, and data-export location where applicable.
mail: rx-status-messaging contributes one visible state or background status to the completion receipt screen.
mail: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
mail: error state gives a recovery action, support reference, audit id, and retry budget.
mail: success state links receipt, audit seal, and data-export location where applicable.
identity: patient-prescriber-resolution contributes one visible state or background status to the completion receipt screen.
identity: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
identity: error state gives a recovery action, support reference, audit id, and retry budget.
identity: success state links receipt, audit seal, and data-export location where applicable.
connect: pharmacy-adapter contributes one visible state or background status to the completion receipt screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
compliance: rx-overlay contributes one visible state or background status to the completion receipt screen.
compliance: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
compliance: error state gives a recovery action, support reference, audit id, and retry budget.
compliance: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: completion receipt passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: completion receipt uses ko-KR strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Interaction state matrix
UX check 1: on entry point, workflow-studio (rx-renewal-template) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Yejin Park's context.
UX check 2: on tenant context switcher, workflow-engine (prescriber-routing) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Yejin Park's context.
UX check 3: on identity and recovery confirmation, mail (rx-status-messaging) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Yejin Park's context.
UX check 4: on primary work canvas, identity (patient-prescriber-resolution) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Yejin Park's context.
UX check 5: on review panel, connect (pharmacy-adapter) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Yejin Park's context.
UX check 6: on approval or confirmation panel, compliance (rx-overlay) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Yejin Park's context.
UX check 7: on counterparty or provider handoff, workflow-studio (rx-renewal-template) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Yejin Park's context.
UX check 8: on settlement and notification panel, workflow-engine (prescriber-routing) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Yejin Park's context.
