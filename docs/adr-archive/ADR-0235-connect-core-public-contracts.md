---
id: ADR-0235
status: Superseded
deciders: communications-service-council, council-architecture, council-privacy
date: 2026-05-17
owner: communications-service-council
supersedes: []
superseded_by: [ADR-0709]
related:
  - ADR-0001
  - ADR-0003
  - ADR-0008
  - ADR-0059
  - ADR-0234
related_specs:
  - /specs/products/connect/suite.json
  - /specs/products/connect/mail.json
  - /specs/products/connect/calendar.json
  - /specs/products/connect/messenger.json
version: 1.0.0
purpose: Authorize the core public contracts added by the OP-11 industry audit and bind them to Workflow/Ontology mediation, audit emission, and dual-context safety rules.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0235: Core Public Contracts

## Status

Accepted - 2026-05-17.

## Context

PR #131 expands the existing platform, mail, messenger, and calendar PRDs after the OP-11 competitive audit. The PR introduces six public contracts:

- `connect.mail.alias.v1`
- `connect.perimeter-auth-result.v1`
- `connect.calendar.ical_feed.v1`
- `connect.calendar.video_call_link.v1`
- `connect.messenger.presence.v1`
- `connect.messenger.reaction.v1`

These contracts are useful only if they remain explicit product contracts rather than implicit fields inside narrative PRDs. They cross privacy, retention, legal-hold, email authentication, calendar federation, messenger presence, and reaction-retention behavior. That makes a decision record mandatory before merge.

## Decision

Accept the six contracts as **planning-stage public contracts** for core PRDs with these constraints:

- Contract use is advisory until the corresponding implementation crates, contract schemas, and validators land.
- All cross-product behavior must route through Workflow and Ontology mediation. Direct child-to-child calls are not allowed.
- Any contract that can reveal personal/work state must preserve immutable `context_kind` and `ownership_pillar` boundaries.
- `connect.perimeter-auth-result.v1` is emitted before child-app processing and must be audit-chain visible when it affects reject, quarantine, or legal-hold behavior.
- `connect.mail.alias.v1` may create privacy aliases, but alias ownership cannot be joined across pillars.
- `connect.calendar.ical_feed.v1` and `connect.calendar.video_call_link.v1` must expose free/busy and link lifecycle state without leaking personal event details.
- `connect.messenger.presence.v1` must be pillar-isolated; work tenants cannot observe personal presence.
- `connect.messenger.reaction.v1` inherits parent-message retention and legal-hold policy.

## Rejected Alternatives

- **Leave the contracts as decision-log prose only.** Rejected because public contracts need an addressable ADR for review, migration, and validator binding.
- **Use unmerged consolidation ADR numbers.** Rejected because those decisions are not on this branch and some proposed numbers collide with inherited Bominal ADRs.
- **Treat these contracts as implemented.** Rejected because PR #131 updates PRDs and evidence only; runtime crates, schemas, and validators remain follow-up work.
- **Expose presence, aliases, or calendar availability directly between child products.** Rejected because it would bypass the Workflow/Ontology mediation model and weaken the dual-context boundary.

## Consequences

- core PRDs must cite this ADR when naming the six contracts.
- Hyperscaler and competitive benchmark claims remain advisory until validators bind the contracts to implementation.
- Follow-up implementation must add schemas and tests before these contract names are used as shipped API commitments.
- Reviewers can block future PRs that widen these contracts without updating this ADR or superseding it.

## Verification

- `cargo run -p oya-dev-cli -- doc adr-index --write --format json`
- `cargo run -p oya-dev-cli -- gate validate product-prd-json --product specs/products/connect/suite.json --product specs/products/connect/mail.json --product specs/products/connect/calendar.json --product specs/products/connect/messenger.json`
- Reviewer-agent check for PR #131 must confirm the contracts remain advisory and route through Workflow/Ontology until implementation validators land.
