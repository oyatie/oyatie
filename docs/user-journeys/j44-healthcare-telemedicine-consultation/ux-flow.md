---
doc_class: User-Journey-UX-Flow
journey_id: j44-healthcare-telemedicine-consultation
status: Proposed
date: 2026-05-20
authority_tier: 3
persona: Yejin Park
locale: ko-KR
tenant_scope: seoul-hospital-healthcare
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
  - meet
  - intelligence
  - notes
  - connect
  - compliance
  - audit-chain
journey_number: j44
benchmark: Teladoc virtual visit plus Epic FHIR export pattern
---

# j44-healthcare-telemedicine-consultation UX flow

Purpose: Screen-by-screen flow for Yejin Park to run a virtual consultation, transcribe it, capture the clinical note, and export to EHR.

## Screen 1: entry point
Primary user: Yejin Park; locale: ko-KR; tenant: seoul-hospital-healthcare.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
meet: telemedicine-room contributes one visible state or background status to the entry point screen.
meet: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
meet: error state gives a recovery action, support reference, audit id, and retry budget.
meet: success state links receipt, audit seal, and data-export location where applicable.
intelligence: clinical-transcription contributes one visible state or background status to the entry point screen.
intelligence: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
intelligence: error state gives a recovery action, support reference, audit id, and retry budget.
intelligence: success state links receipt, audit seal, and data-export location where applicable.
notes: consult-note contributes one visible state or background status to the entry point screen.
notes: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
notes: error state gives a recovery action, support reference, audit id, and retry budget.
notes: success state links receipt, audit seal, and data-export location where applicable.
connect: ehr-export contributes one visible state or background status to the entry point screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
compliance: hipaa-consult-overlay contributes one visible state or background status to the entry point screen.
compliance: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
compliance: error state gives a recovery action, support reference, audit id, and retry budget.
compliance: success state links receipt, audit seal, and data-export location where applicable.
audit-chain: consult-seal contributes one visible state or background status to the entry point screen.
audit-chain: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
audit-chain: error state gives a recovery action, support reference, audit id, and retry budget.
audit-chain: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: entry point passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: entry point uses ko-KR strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 2: tenant context switcher
Primary user: Yejin Park; locale: ko-KR; tenant: seoul-hospital-healthcare.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
meet: telemedicine-room contributes one visible state or background status to the tenant context switcher screen.
meet: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
meet: error state gives a recovery action, support reference, audit id, and retry budget.
meet: success state links receipt, audit seal, and data-export location where applicable.
intelligence: clinical-transcription contributes one visible state or background status to the tenant context switcher screen.
intelligence: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
intelligence: error state gives a recovery action, support reference, audit id, and retry budget.
intelligence: success state links receipt, audit seal, and data-export location where applicable.
notes: consult-note contributes one visible state or background status to the tenant context switcher screen.
notes: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
notes: error state gives a recovery action, support reference, audit id, and retry budget.
notes: success state links receipt, audit seal, and data-export location where applicable.
connect: ehr-export contributes one visible state or background status to the tenant context switcher screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
compliance: hipaa-consult-overlay contributes one visible state or background status to the tenant context switcher screen.
compliance: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
compliance: error state gives a recovery action, support reference, audit id, and retry budget.
compliance: success state links receipt, audit seal, and data-export location where applicable.
audit-chain: consult-seal contributes one visible state or background status to the tenant context switcher screen.
audit-chain: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
audit-chain: error state gives a recovery action, support reference, audit id, and retry budget.
audit-chain: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: tenant context switcher passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: tenant context switcher uses ko-KR strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 3: identity and recovery confirmation
Primary user: Yejin Park; locale: ko-KR; tenant: seoul-hospital-healthcare.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
meet: telemedicine-room contributes one visible state or background status to the identity and recovery confirmation screen.
meet: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
meet: error state gives a recovery action, support reference, audit id, and retry budget.
meet: success state links receipt, audit seal, and data-export location where applicable.
intelligence: clinical-transcription contributes one visible state or background status to the identity and recovery confirmation screen.
intelligence: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
intelligence: error state gives a recovery action, support reference, audit id, and retry budget.
intelligence: success state links receipt, audit seal, and data-export location where applicable.
notes: consult-note contributes one visible state or background status to the identity and recovery confirmation screen.
notes: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
notes: error state gives a recovery action, support reference, audit id, and retry budget.
notes: success state links receipt, audit seal, and data-export location where applicable.
connect: ehr-export contributes one visible state or background status to the identity and recovery confirmation screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
compliance: hipaa-consult-overlay contributes one visible state or background status to the identity and recovery confirmation screen.
compliance: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
compliance: error state gives a recovery action, support reference, audit id, and retry budget.
compliance: success state links receipt, audit seal, and data-export location where applicable.
audit-chain: consult-seal contributes one visible state or background status to the identity and recovery confirmation screen.
audit-chain: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
audit-chain: error state gives a recovery action, support reference, audit id, and retry budget.
audit-chain: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: identity and recovery confirmation passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: identity and recovery confirmation uses ko-KR strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 4: primary work canvas
Primary user: Yejin Park; locale: ko-KR; tenant: seoul-hospital-healthcare.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
meet: telemedicine-room contributes one visible state or background status to the primary work canvas screen.
meet: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
meet: error state gives a recovery action, support reference, audit id, and retry budget.
meet: success state links receipt, audit seal, and data-export location where applicable.
intelligence: clinical-transcription contributes one visible state or background status to the primary work canvas screen.
intelligence: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
intelligence: error state gives a recovery action, support reference, audit id, and retry budget.
intelligence: success state links receipt, audit seal, and data-export location where applicable.
notes: consult-note contributes one visible state or background status to the primary work canvas screen.
notes: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
notes: error state gives a recovery action, support reference, audit id, and retry budget.
notes: success state links receipt, audit seal, and data-export location where applicable.
connect: ehr-export contributes one visible state or background status to the primary work canvas screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
compliance: hipaa-consult-overlay contributes one visible state or background status to the primary work canvas screen.
compliance: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
compliance: error state gives a recovery action, support reference, audit id, and retry budget.
compliance: success state links receipt, audit seal, and data-export location where applicable.
audit-chain: consult-seal contributes one visible state or background status to the primary work canvas screen.
audit-chain: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
audit-chain: error state gives a recovery action, support reference, audit id, and retry budget.
audit-chain: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: primary work canvas passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: primary work canvas uses ko-KR strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 5: review panel
Primary user: Yejin Park; locale: ko-KR; tenant: seoul-hospital-healthcare.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
meet: telemedicine-room contributes one visible state or background status to the review panel screen.
meet: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
meet: error state gives a recovery action, support reference, audit id, and retry budget.
meet: success state links receipt, audit seal, and data-export location where applicable.
intelligence: clinical-transcription contributes one visible state or background status to the review panel screen.
intelligence: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
intelligence: error state gives a recovery action, support reference, audit id, and retry budget.
intelligence: success state links receipt, audit seal, and data-export location where applicable.
notes: consult-note contributes one visible state or background status to the review panel screen.
notes: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
notes: error state gives a recovery action, support reference, audit id, and retry budget.
notes: success state links receipt, audit seal, and data-export location where applicable.
connect: ehr-export contributes one visible state or background status to the review panel screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
compliance: hipaa-consult-overlay contributes one visible state or background status to the review panel screen.
compliance: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
compliance: error state gives a recovery action, support reference, audit id, and retry budget.
compliance: success state links receipt, audit seal, and data-export location where applicable.
audit-chain: consult-seal contributes one visible state or background status to the review panel screen.
audit-chain: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
audit-chain: error state gives a recovery action, support reference, audit id, and retry budget.
audit-chain: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: review panel passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: review panel uses ko-KR strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 6: approval or confirmation panel
Primary user: Yejin Park; locale: ko-KR; tenant: seoul-hospital-healthcare.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
meet: telemedicine-room contributes one visible state or background status to the approval or confirmation panel screen.
meet: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
meet: error state gives a recovery action, support reference, audit id, and retry budget.
meet: success state links receipt, audit seal, and data-export location where applicable.
intelligence: clinical-transcription contributes one visible state or background status to the approval or confirmation panel screen.
intelligence: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
intelligence: error state gives a recovery action, support reference, audit id, and retry budget.
intelligence: success state links receipt, audit seal, and data-export location where applicable.
notes: consult-note contributes one visible state or background status to the approval or confirmation panel screen.
notes: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
notes: error state gives a recovery action, support reference, audit id, and retry budget.
notes: success state links receipt, audit seal, and data-export location where applicable.
connect: ehr-export contributes one visible state or background status to the approval or confirmation panel screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
compliance: hipaa-consult-overlay contributes one visible state or background status to the approval or confirmation panel screen.
compliance: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
compliance: error state gives a recovery action, support reference, audit id, and retry budget.
compliance: success state links receipt, audit seal, and data-export location where applicable.
audit-chain: consult-seal contributes one visible state or background status to the approval or confirmation panel screen.
audit-chain: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
audit-chain: error state gives a recovery action, support reference, audit id, and retry budget.
audit-chain: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: approval or confirmation panel passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: approval or confirmation panel uses ko-KR strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 7: counterparty or provider handoff
Primary user: Yejin Park; locale: ko-KR; tenant: seoul-hospital-healthcare.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
meet: telemedicine-room contributes one visible state or background status to the counterparty or provider handoff screen.
meet: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
meet: error state gives a recovery action, support reference, audit id, and retry budget.
meet: success state links receipt, audit seal, and data-export location where applicable.
intelligence: clinical-transcription contributes one visible state or background status to the counterparty or provider handoff screen.
intelligence: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
intelligence: error state gives a recovery action, support reference, audit id, and retry budget.
intelligence: success state links receipt, audit seal, and data-export location where applicable.
notes: consult-note contributes one visible state or background status to the counterparty or provider handoff screen.
notes: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
notes: error state gives a recovery action, support reference, audit id, and retry budget.
notes: success state links receipt, audit seal, and data-export location where applicable.
connect: ehr-export contributes one visible state or background status to the counterparty or provider handoff screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
compliance: hipaa-consult-overlay contributes one visible state or background status to the counterparty or provider handoff screen.
compliance: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
compliance: error state gives a recovery action, support reference, audit id, and retry budget.
compliance: success state links receipt, audit seal, and data-export location where applicable.
audit-chain: consult-seal contributes one visible state or background status to the counterparty or provider handoff screen.
audit-chain: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
audit-chain: error state gives a recovery action, support reference, audit id, and retry budget.
audit-chain: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: counterparty or provider handoff passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: counterparty or provider handoff uses ko-KR strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 8: settlement and notification panel
Primary user: Yejin Park; locale: ko-KR; tenant: seoul-hospital-healthcare.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
meet: telemedicine-room contributes one visible state or background status to the settlement and notification panel screen.
meet: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
meet: error state gives a recovery action, support reference, audit id, and retry budget.
meet: success state links receipt, audit seal, and data-export location where applicable.
intelligence: clinical-transcription contributes one visible state or background status to the settlement and notification panel screen.
intelligence: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
intelligence: error state gives a recovery action, support reference, audit id, and retry budget.
intelligence: success state links receipt, audit seal, and data-export location where applicable.
notes: consult-note contributes one visible state or background status to the settlement and notification panel screen.
notes: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
notes: error state gives a recovery action, support reference, audit id, and retry budget.
notes: success state links receipt, audit seal, and data-export location where applicable.
connect: ehr-export contributes one visible state or background status to the settlement and notification panel screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
compliance: hipaa-consult-overlay contributes one visible state or background status to the settlement and notification panel screen.
compliance: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
compliance: error state gives a recovery action, support reference, audit id, and retry budget.
compliance: success state links receipt, audit seal, and data-export location where applicable.
audit-chain: consult-seal contributes one visible state or background status to the settlement and notification panel screen.
audit-chain: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
audit-chain: error state gives a recovery action, support reference, audit id, and retry budget.
audit-chain: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: settlement and notification panel passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: settlement and notification panel uses ko-KR strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 9: audit detail panel
Primary user: Yejin Park; locale: ko-KR; tenant: seoul-hospital-healthcare.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
meet: telemedicine-room contributes one visible state or background status to the audit detail panel screen.
meet: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
meet: error state gives a recovery action, support reference, audit id, and retry budget.
meet: success state links receipt, audit seal, and data-export location where applicable.
intelligence: clinical-transcription contributes one visible state or background status to the audit detail panel screen.
intelligence: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
intelligence: error state gives a recovery action, support reference, audit id, and retry budget.
intelligence: success state links receipt, audit seal, and data-export location where applicable.
notes: consult-note contributes one visible state or background status to the audit detail panel screen.
notes: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
notes: error state gives a recovery action, support reference, audit id, and retry budget.
notes: success state links receipt, audit seal, and data-export location where applicable.
connect: ehr-export contributes one visible state or background status to the audit detail panel screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
compliance: hipaa-consult-overlay contributes one visible state or background status to the audit detail panel screen.
compliance: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
compliance: error state gives a recovery action, support reference, audit id, and retry budget.
compliance: success state links receipt, audit seal, and data-export location where applicable.
audit-chain: consult-seal contributes one visible state or background status to the audit detail panel screen.
audit-chain: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
audit-chain: error state gives a recovery action, support reference, audit id, and retry budget.
audit-chain: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: audit detail panel passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: audit detail panel uses ko-KR strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 10: error recovery panel
Primary user: Yejin Park; locale: ko-KR; tenant: seoul-hospital-healthcare.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
meet: telemedicine-room contributes one visible state or background status to the error recovery panel screen.
meet: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
meet: error state gives a recovery action, support reference, audit id, and retry budget.
meet: success state links receipt, audit seal, and data-export location where applicable.
intelligence: clinical-transcription contributes one visible state or background status to the error recovery panel screen.
intelligence: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
intelligence: error state gives a recovery action, support reference, audit id, and retry budget.
intelligence: success state links receipt, audit seal, and data-export location where applicable.
notes: consult-note contributes one visible state or background status to the error recovery panel screen.
notes: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
notes: error state gives a recovery action, support reference, audit id, and retry budget.
notes: success state links receipt, audit seal, and data-export location where applicable.
connect: ehr-export contributes one visible state or background status to the error recovery panel screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
compliance: hipaa-consult-overlay contributes one visible state or background status to the error recovery panel screen.
compliance: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
compliance: error state gives a recovery action, support reference, audit id, and retry budget.
compliance: success state links receipt, audit seal, and data-export location where applicable.
audit-chain: consult-seal contributes one visible state or background status to the error recovery panel screen.
audit-chain: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
audit-chain: error state gives a recovery action, support reference, audit id, and retry budget.
audit-chain: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: error recovery panel passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: error recovery panel uses ko-KR strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 11: mobile compact view
Primary user: Yejin Park; locale: ko-KR; tenant: seoul-hospital-healthcare.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
meet: telemedicine-room contributes one visible state or background status to the mobile compact view screen.
meet: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
meet: error state gives a recovery action, support reference, audit id, and retry budget.
meet: success state links receipt, audit seal, and data-export location where applicable.
intelligence: clinical-transcription contributes one visible state or background status to the mobile compact view screen.
intelligence: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
intelligence: error state gives a recovery action, support reference, audit id, and retry budget.
intelligence: success state links receipt, audit seal, and data-export location where applicable.
notes: consult-note contributes one visible state or background status to the mobile compact view screen.
notes: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
notes: error state gives a recovery action, support reference, audit id, and retry budget.
notes: success state links receipt, audit seal, and data-export location where applicable.
connect: ehr-export contributes one visible state or background status to the mobile compact view screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
compliance: hipaa-consult-overlay contributes one visible state or background status to the mobile compact view screen.
compliance: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
compliance: error state gives a recovery action, support reference, audit id, and retry budget.
compliance: success state links receipt, audit seal, and data-export location where applicable.
audit-chain: consult-seal contributes one visible state or background status to the mobile compact view screen.
audit-chain: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
audit-chain: error state gives a recovery action, support reference, audit id, and retry budget.
audit-chain: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: mobile compact view passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: mobile compact view uses ko-KR strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 12: completion receipt
Primary user: Yejin Park; locale: ko-KR; tenant: seoul-hospital-healthcare.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
meet: telemedicine-room contributes one visible state or background status to the completion receipt screen.
meet: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
meet: error state gives a recovery action, support reference, audit id, and retry budget.
meet: success state links receipt, audit seal, and data-export location where applicable.
intelligence: clinical-transcription contributes one visible state or background status to the completion receipt screen.
intelligence: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
intelligence: error state gives a recovery action, support reference, audit id, and retry budget.
intelligence: success state links receipt, audit seal, and data-export location where applicable.
notes: consult-note contributes one visible state or background status to the completion receipt screen.
notes: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
notes: error state gives a recovery action, support reference, audit id, and retry budget.
notes: success state links receipt, audit seal, and data-export location where applicable.
connect: ehr-export contributes one visible state or background status to the completion receipt screen.
connect: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
connect: error state gives a recovery action, support reference, audit id, and retry budget.
connect: success state links receipt, audit seal, and data-export location where applicable.
compliance: hipaa-consult-overlay contributes one visible state or background status to the completion receipt screen.
compliance: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
compliance: error state gives a recovery action, support reference, audit id, and retry budget.
compliance: success state links receipt, audit seal, and data-export location where applicable.
audit-chain: consult-seal contributes one visible state or background status to the completion receipt screen.
audit-chain: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
audit-chain: error state gives a recovery action, support reference, audit id, and retry budget.
audit-chain: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: completion receipt passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: completion receipt uses ko-KR strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Interaction state matrix
UX check 1: on entry point, meet (telemedicine-room) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Yejin Park's context.
UX check 2: on tenant context switcher, intelligence (clinical-transcription) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Yejin Park's context.
UX check 3: on identity and recovery confirmation, notes (consult-note) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Yejin Park's context.
UX check 4: on primary work canvas, connect (ehr-export) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Yejin Park's context.
UX check 5: on review panel, compliance (hipaa-consult-overlay) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Yejin Park's context.
UX check 6: on approval or confirmation panel, audit-chain (consult-seal) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Yejin Park's context.
UX check 7: on counterparty or provider handoff, meet (telemedicine-room) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Yejin Park's context.
UX check 8: on settlement and notification panel, intelligence (clinical-transcription) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Yejin Park's context.
