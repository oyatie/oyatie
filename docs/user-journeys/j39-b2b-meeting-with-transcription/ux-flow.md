---
doc_class: User-Journey-UX-Flow
journey_id: j39-b2b-meeting-with-transcription
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
  - meet
  - intelligence
  - recordings
  - drive
  - notes
  - observability
journey_number: j39
benchmark: Google Meet recording plus Microsoft Teams transcript retention pattern
---

# j39-b2b-meeting-with-transcription UX flow

Purpose: Screen-by-screen flow for Marcus Chen to host a large review meeting, transcribe it, archive the recording, and make the transcript searchable in Notes.

## Screen 1: entry point
Primary user: Marcus Chen; locale: en-US; tenant: acme-b2b.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
meet: quarterly-review-room contributes one visible state or background status to the entry point screen.
meet: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
meet: error state gives a recovery action, support reference, audit id, and retry budget.
meet: success state links receipt, audit seal, and data-export location where applicable.
intelligence: transcription-summarization contributes one visible state or background status to the entry point screen.
intelligence: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
intelligence: error state gives a recovery action, support reference, audit id, and retry budget.
intelligence: success state links receipt, audit seal, and data-export location where applicable.
recordings: immutable-recording contributes one visible state or background status to the entry point screen.
recordings: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
recordings: error state gives a recovery action, support reference, audit id, and retry budget.
recordings: success state links receipt, audit seal, and data-export location where applicable.
drive: archive-folder contributes one visible state or background status to the entry point screen.
drive: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
drive: error state gives a recovery action, support reference, audit id, and retry budget.
drive: success state links receipt, audit seal, and data-export location where applicable.
notes: transcript-search-index contributes one visible state or background status to the entry point screen.
notes: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
notes: error state gives a recovery action, support reference, audit id, and retry budget.
notes: success state links receipt, audit seal, and data-export location where applicable.
observability: meeting-telemetry contributes one visible state or background status to the entry point screen.
observability: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
observability: error state gives a recovery action, support reference, audit id, and retry budget.
observability: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: entry point passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: entry point uses en-US strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 2: tenant context switcher
Primary user: Marcus Chen; locale: en-US; tenant: acme-b2b.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
meet: quarterly-review-room contributes one visible state or background status to the tenant context switcher screen.
meet: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
meet: error state gives a recovery action, support reference, audit id, and retry budget.
meet: success state links receipt, audit seal, and data-export location where applicable.
intelligence: transcription-summarization contributes one visible state or background status to the tenant context switcher screen.
intelligence: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
intelligence: error state gives a recovery action, support reference, audit id, and retry budget.
intelligence: success state links receipt, audit seal, and data-export location where applicable.
recordings: immutable-recording contributes one visible state or background status to the tenant context switcher screen.
recordings: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
recordings: error state gives a recovery action, support reference, audit id, and retry budget.
recordings: success state links receipt, audit seal, and data-export location where applicable.
drive: archive-folder contributes one visible state or background status to the tenant context switcher screen.
drive: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
drive: error state gives a recovery action, support reference, audit id, and retry budget.
drive: success state links receipt, audit seal, and data-export location where applicable.
notes: transcript-search-index contributes one visible state or background status to the tenant context switcher screen.
notes: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
notes: error state gives a recovery action, support reference, audit id, and retry budget.
notes: success state links receipt, audit seal, and data-export location where applicable.
observability: meeting-telemetry contributes one visible state or background status to the tenant context switcher screen.
observability: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
observability: error state gives a recovery action, support reference, audit id, and retry budget.
observability: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: tenant context switcher passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: tenant context switcher uses en-US strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 3: identity and recovery confirmation
Primary user: Marcus Chen; locale: en-US; tenant: acme-b2b.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
meet: quarterly-review-room contributes one visible state or background status to the identity and recovery confirmation screen.
meet: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
meet: error state gives a recovery action, support reference, audit id, and retry budget.
meet: success state links receipt, audit seal, and data-export location where applicable.
intelligence: transcription-summarization contributes one visible state or background status to the identity and recovery confirmation screen.
intelligence: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
intelligence: error state gives a recovery action, support reference, audit id, and retry budget.
intelligence: success state links receipt, audit seal, and data-export location where applicable.
recordings: immutable-recording contributes one visible state or background status to the identity and recovery confirmation screen.
recordings: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
recordings: error state gives a recovery action, support reference, audit id, and retry budget.
recordings: success state links receipt, audit seal, and data-export location where applicable.
drive: archive-folder contributes one visible state or background status to the identity and recovery confirmation screen.
drive: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
drive: error state gives a recovery action, support reference, audit id, and retry budget.
drive: success state links receipt, audit seal, and data-export location where applicable.
notes: transcript-search-index contributes one visible state or background status to the identity and recovery confirmation screen.
notes: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
notes: error state gives a recovery action, support reference, audit id, and retry budget.
notes: success state links receipt, audit seal, and data-export location where applicable.
observability: meeting-telemetry contributes one visible state or background status to the identity and recovery confirmation screen.
observability: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
observability: error state gives a recovery action, support reference, audit id, and retry budget.
observability: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: identity and recovery confirmation passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: identity and recovery confirmation uses en-US strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 4: primary work canvas
Primary user: Marcus Chen; locale: en-US; tenant: acme-b2b.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
meet: quarterly-review-room contributes one visible state or background status to the primary work canvas screen.
meet: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
meet: error state gives a recovery action, support reference, audit id, and retry budget.
meet: success state links receipt, audit seal, and data-export location where applicable.
intelligence: transcription-summarization contributes one visible state or background status to the primary work canvas screen.
intelligence: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
intelligence: error state gives a recovery action, support reference, audit id, and retry budget.
intelligence: success state links receipt, audit seal, and data-export location where applicable.
recordings: immutable-recording contributes one visible state or background status to the primary work canvas screen.
recordings: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
recordings: error state gives a recovery action, support reference, audit id, and retry budget.
recordings: success state links receipt, audit seal, and data-export location where applicable.
drive: archive-folder contributes one visible state or background status to the primary work canvas screen.
drive: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
drive: error state gives a recovery action, support reference, audit id, and retry budget.
drive: success state links receipt, audit seal, and data-export location where applicable.
notes: transcript-search-index contributes one visible state or background status to the primary work canvas screen.
notes: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
notes: error state gives a recovery action, support reference, audit id, and retry budget.
notes: success state links receipt, audit seal, and data-export location where applicable.
observability: meeting-telemetry contributes one visible state or background status to the primary work canvas screen.
observability: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
observability: error state gives a recovery action, support reference, audit id, and retry budget.
observability: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: primary work canvas passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: primary work canvas uses en-US strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 5: review panel
Primary user: Marcus Chen; locale: en-US; tenant: acme-b2b.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
meet: quarterly-review-room contributes one visible state or background status to the review panel screen.
meet: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
meet: error state gives a recovery action, support reference, audit id, and retry budget.
meet: success state links receipt, audit seal, and data-export location where applicable.
intelligence: transcription-summarization contributes one visible state or background status to the review panel screen.
intelligence: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
intelligence: error state gives a recovery action, support reference, audit id, and retry budget.
intelligence: success state links receipt, audit seal, and data-export location where applicable.
recordings: immutable-recording contributes one visible state or background status to the review panel screen.
recordings: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
recordings: error state gives a recovery action, support reference, audit id, and retry budget.
recordings: success state links receipt, audit seal, and data-export location where applicable.
drive: archive-folder contributes one visible state or background status to the review panel screen.
drive: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
drive: error state gives a recovery action, support reference, audit id, and retry budget.
drive: success state links receipt, audit seal, and data-export location where applicable.
notes: transcript-search-index contributes one visible state or background status to the review panel screen.
notes: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
notes: error state gives a recovery action, support reference, audit id, and retry budget.
notes: success state links receipt, audit seal, and data-export location where applicable.
observability: meeting-telemetry contributes one visible state or background status to the review panel screen.
observability: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
observability: error state gives a recovery action, support reference, audit id, and retry budget.
observability: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: review panel passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: review panel uses en-US strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 6: approval or confirmation panel
Primary user: Marcus Chen; locale: en-US; tenant: acme-b2b.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
meet: quarterly-review-room contributes one visible state or background status to the approval or confirmation panel screen.
meet: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
meet: error state gives a recovery action, support reference, audit id, and retry budget.
meet: success state links receipt, audit seal, and data-export location where applicable.
intelligence: transcription-summarization contributes one visible state or background status to the approval or confirmation panel screen.
intelligence: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
intelligence: error state gives a recovery action, support reference, audit id, and retry budget.
intelligence: success state links receipt, audit seal, and data-export location where applicable.
recordings: immutable-recording contributes one visible state or background status to the approval or confirmation panel screen.
recordings: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
recordings: error state gives a recovery action, support reference, audit id, and retry budget.
recordings: success state links receipt, audit seal, and data-export location where applicable.
drive: archive-folder contributes one visible state or background status to the approval or confirmation panel screen.
drive: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
drive: error state gives a recovery action, support reference, audit id, and retry budget.
drive: success state links receipt, audit seal, and data-export location where applicable.
notes: transcript-search-index contributes one visible state or background status to the approval or confirmation panel screen.
notes: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
notes: error state gives a recovery action, support reference, audit id, and retry budget.
notes: success state links receipt, audit seal, and data-export location where applicable.
observability: meeting-telemetry contributes one visible state or background status to the approval or confirmation panel screen.
observability: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
observability: error state gives a recovery action, support reference, audit id, and retry budget.
observability: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: approval or confirmation panel passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: approval or confirmation panel uses en-US strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 7: counterparty or provider handoff
Primary user: Marcus Chen; locale: en-US; tenant: acme-b2b.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
meet: quarterly-review-room contributes one visible state or background status to the counterparty or provider handoff screen.
meet: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
meet: error state gives a recovery action, support reference, audit id, and retry budget.
meet: success state links receipt, audit seal, and data-export location where applicable.
intelligence: transcription-summarization contributes one visible state or background status to the counterparty or provider handoff screen.
intelligence: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
intelligence: error state gives a recovery action, support reference, audit id, and retry budget.
intelligence: success state links receipt, audit seal, and data-export location where applicable.
recordings: immutable-recording contributes one visible state or background status to the counterparty or provider handoff screen.
recordings: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
recordings: error state gives a recovery action, support reference, audit id, and retry budget.
recordings: success state links receipt, audit seal, and data-export location where applicable.
drive: archive-folder contributes one visible state or background status to the counterparty or provider handoff screen.
drive: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
drive: error state gives a recovery action, support reference, audit id, and retry budget.
drive: success state links receipt, audit seal, and data-export location where applicable.
notes: transcript-search-index contributes one visible state or background status to the counterparty or provider handoff screen.
notes: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
notes: error state gives a recovery action, support reference, audit id, and retry budget.
notes: success state links receipt, audit seal, and data-export location where applicable.
observability: meeting-telemetry contributes one visible state or background status to the counterparty or provider handoff screen.
observability: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
observability: error state gives a recovery action, support reference, audit id, and retry budget.
observability: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: counterparty or provider handoff passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: counterparty or provider handoff uses en-US strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 8: settlement and notification panel
Primary user: Marcus Chen; locale: en-US; tenant: acme-b2b.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
meet: quarterly-review-room contributes one visible state or background status to the settlement and notification panel screen.
meet: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
meet: error state gives a recovery action, support reference, audit id, and retry budget.
meet: success state links receipt, audit seal, and data-export location where applicable.
intelligence: transcription-summarization contributes one visible state or background status to the settlement and notification panel screen.
intelligence: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
intelligence: error state gives a recovery action, support reference, audit id, and retry budget.
intelligence: success state links receipt, audit seal, and data-export location where applicable.
recordings: immutable-recording contributes one visible state or background status to the settlement and notification panel screen.
recordings: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
recordings: error state gives a recovery action, support reference, audit id, and retry budget.
recordings: success state links receipt, audit seal, and data-export location where applicable.
drive: archive-folder contributes one visible state or background status to the settlement and notification panel screen.
drive: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
drive: error state gives a recovery action, support reference, audit id, and retry budget.
drive: success state links receipt, audit seal, and data-export location where applicable.
notes: transcript-search-index contributes one visible state or background status to the settlement and notification panel screen.
notes: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
notes: error state gives a recovery action, support reference, audit id, and retry budget.
notes: success state links receipt, audit seal, and data-export location where applicable.
observability: meeting-telemetry contributes one visible state or background status to the settlement and notification panel screen.
observability: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
observability: error state gives a recovery action, support reference, audit id, and retry budget.
observability: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: settlement and notification panel passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: settlement and notification panel uses en-US strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 9: audit detail panel
Primary user: Marcus Chen; locale: en-US; tenant: acme-b2b.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
meet: quarterly-review-room contributes one visible state or background status to the audit detail panel screen.
meet: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
meet: error state gives a recovery action, support reference, audit id, and retry budget.
meet: success state links receipt, audit seal, and data-export location where applicable.
intelligence: transcription-summarization contributes one visible state or background status to the audit detail panel screen.
intelligence: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
intelligence: error state gives a recovery action, support reference, audit id, and retry budget.
intelligence: success state links receipt, audit seal, and data-export location where applicable.
recordings: immutable-recording contributes one visible state or background status to the audit detail panel screen.
recordings: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
recordings: error state gives a recovery action, support reference, audit id, and retry budget.
recordings: success state links receipt, audit seal, and data-export location where applicable.
drive: archive-folder contributes one visible state or background status to the audit detail panel screen.
drive: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
drive: error state gives a recovery action, support reference, audit id, and retry budget.
drive: success state links receipt, audit seal, and data-export location where applicable.
notes: transcript-search-index contributes one visible state or background status to the audit detail panel screen.
notes: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
notes: error state gives a recovery action, support reference, audit id, and retry budget.
notes: success state links receipt, audit seal, and data-export location where applicable.
observability: meeting-telemetry contributes one visible state or background status to the audit detail panel screen.
observability: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
observability: error state gives a recovery action, support reference, audit id, and retry budget.
observability: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: audit detail panel passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: audit detail panel uses en-US strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 10: error recovery panel
Primary user: Marcus Chen; locale: en-US; tenant: acme-b2b.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
meet: quarterly-review-room contributes one visible state or background status to the error recovery panel screen.
meet: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
meet: error state gives a recovery action, support reference, audit id, and retry budget.
meet: success state links receipt, audit seal, and data-export location where applicable.
intelligence: transcription-summarization contributes one visible state or background status to the error recovery panel screen.
intelligence: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
intelligence: error state gives a recovery action, support reference, audit id, and retry budget.
intelligence: success state links receipt, audit seal, and data-export location where applicable.
recordings: immutable-recording contributes one visible state or background status to the error recovery panel screen.
recordings: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
recordings: error state gives a recovery action, support reference, audit id, and retry budget.
recordings: success state links receipt, audit seal, and data-export location where applicable.
drive: archive-folder contributes one visible state or background status to the error recovery panel screen.
drive: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
drive: error state gives a recovery action, support reference, audit id, and retry budget.
drive: success state links receipt, audit seal, and data-export location where applicable.
notes: transcript-search-index contributes one visible state or background status to the error recovery panel screen.
notes: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
notes: error state gives a recovery action, support reference, audit id, and retry budget.
notes: success state links receipt, audit seal, and data-export location where applicable.
observability: meeting-telemetry contributes one visible state or background status to the error recovery panel screen.
observability: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
observability: error state gives a recovery action, support reference, audit id, and retry budget.
observability: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: error recovery panel passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: error recovery panel uses en-US strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 11: mobile compact view
Primary user: Marcus Chen; locale: en-US; tenant: acme-b2b.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
meet: quarterly-review-room contributes one visible state or background status to the mobile compact view screen.
meet: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
meet: error state gives a recovery action, support reference, audit id, and retry budget.
meet: success state links receipt, audit seal, and data-export location where applicable.
intelligence: transcription-summarization contributes one visible state or background status to the mobile compact view screen.
intelligence: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
intelligence: error state gives a recovery action, support reference, audit id, and retry budget.
intelligence: success state links receipt, audit seal, and data-export location where applicable.
recordings: immutable-recording contributes one visible state or background status to the mobile compact view screen.
recordings: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
recordings: error state gives a recovery action, support reference, audit id, and retry budget.
recordings: success state links receipt, audit seal, and data-export location where applicable.
drive: archive-folder contributes one visible state or background status to the mobile compact view screen.
drive: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
drive: error state gives a recovery action, support reference, audit id, and retry budget.
drive: success state links receipt, audit seal, and data-export location where applicable.
notes: transcript-search-index contributes one visible state or background status to the mobile compact view screen.
notes: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
notes: error state gives a recovery action, support reference, audit id, and retry budget.
notes: success state links receipt, audit seal, and data-export location where applicable.
observability: meeting-telemetry contributes one visible state or background status to the mobile compact view screen.
observability: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
observability: error state gives a recovery action, support reference, audit id, and retry budget.
observability: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: mobile compact view passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: mobile compact view uses en-US strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Screen 12: completion receipt
Primary user: Marcus Chen; locale: en-US; tenant: acme-b2b.
The screen must show the active tenant context before any irreversible action.
The screen must not explain platform internals; it exposes only the action, status, evidence, and next safe choice.
meet: quarterly-review-room contributes one visible state or background status to the completion receipt screen.
meet: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
meet: error state gives a recovery action, support reference, audit id, and retry budget.
meet: success state links receipt, audit seal, and data-export location where applicable.
intelligence: transcription-summarization contributes one visible state or background status to the completion receipt screen.
intelligence: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
intelligence: error state gives a recovery action, support reference, audit id, and retry budget.
intelligence: success state links receipt, audit seal, and data-export location where applicable.
recordings: immutable-recording contributes one visible state or background status to the completion receipt screen.
recordings: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
recordings: error state gives a recovery action, support reference, audit id, and retry budget.
recordings: success state links receipt, audit seal, and data-export location where applicable.
drive: archive-folder contributes one visible state or background status to the completion receipt screen.
drive: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
drive: error state gives a recovery action, support reference, audit id, and retry budget.
drive: success state links receipt, audit seal, and data-export location where applicable.
notes: transcript-search-index contributes one visible state or background status to the completion receipt screen.
notes: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
notes: error state gives a recovery action, support reference, audit id, and retry budget.
notes: success state links receipt, audit seal, and data-export location where applicable.
observability: meeting-telemetry contributes one visible state or background status to the completion receipt screen.
observability: loading state has fixed dimensions, no layout shift, and no hidden critical warning.
observability: error state gives a recovery action, support reference, audit id, and retry budget.
observability: success state links receipt, audit seal, and data-export location where applicable.
Accessibility: completion receipt passes WCAG 2.2 AA, keyboard operation, visible focus, reduced-motion, and screen-reader labels.
Internationalization: completion receipt uses en-US strings, ISO currency display, local date ordering, and no embedded English-only legal copy.
## Interaction state matrix
UX check 1: on entry point, meet (quarterly-review-room) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 2: on tenant context switcher, intelligence (transcription-summarization) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 3: on identity and recovery confirmation, recordings (immutable-recording) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 4: on primary work canvas, drive (archive-folder) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 5: on review panel, notes (transcript-search-index) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 6: on approval or confirmation panel, observability (meeting-telemetry) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 7: on counterparty or provider handoff, meet (quarterly-review-room) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
UX check 8: on settlement and notification panel, intelligence (transcription-summarization) supports idle, loading, partial, denied, degraded, retrying, and complete states without losing Marcus Chen's context.
