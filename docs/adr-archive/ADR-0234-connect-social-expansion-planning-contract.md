---
id: ADR-0234
status: Superseded
deciders: communications-service-council, council-architecture, council-design-system, ops-security
date: 2026-05-17
owner: communications-service-council
supersedes: []
superseded_by: [ADR-700]
related:
  - ADR-0001
  - ADR-0029
  - ADR-0056
  - ADR-0105
  - ADR-0123
related_specs:
  - /specs/masterplan.json
  - /specs/products/connect/suite.json
  - /specs/products/connect/social.json
  - /specs/products/connect/shorts.json
  - /microservices/community/PRD.md
  - /specs/products/connect/anonymous.json
version: 1.0.0
purpose: Record the PR #130 expansion as a planning contract, not an implemented production or enforcement claim.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0234: Social Expansion Planning Contract

## Status

Accepted — 2026-05-17.

## Context

already has a dual-context architecture and a product-platform role in the flat Oyatie catalog. PR #130 expands the planning surface from the existing mail, messenger, and calendar PRDs to four additional sub-products:

- `community-social`
- `connect-shorts`
- `connect-network` (retired by Wave 15K into `community`)
- `connect-anonymous`

The expansion is high-risk because it crosses consumer social UX, professional identity, creator/recommendation systems, anonymous workplace discussion, HR-adjacent aggregate analytics, legal-hold reveal paths, and personal/work context boundaries. It also introduces planned crate names, design-system components, competitive benchmark rows, and hyperscaler bar targets before the implementation crates and validators exist.

The ADR number intentionally avoids `ADR-0126`. The inherited Bominal ADR map already uses `ADR-0126` for employment classification, so an Oyatie decision with that number would create ambiguous cross-repo citations.

## Decision

Accept the expansion PRDs as a **planning contract** for PR #130, with these constraints:

- The new sub-products are catalog/planning surfaces only until their crates, validators, gates, and CI lanes land.
- `industry_patterns_adopted`, `anti_patterns_avoided`, `hyperscaler_bar`, and `production_readiness_gates` are advisory unless a concrete validator exists in this repo.
- Planned crate names must carry BNF and ADR-0105 layer-enum bindings before merge.
- Cross-product behavior must route through Workflow and Ontology. Direct cross-microservice calls remain forbidden by the cohesion thesis.
- `connect-anonymous` requires a structured threat model before merge because anonymity, legal reveal, HSM/vault handling, operator misuse, and side-channel risk are product-defining concerns.
- HR-adjacent aggregate dashboards remain Anonymous UX primitives, with explicit Enterprise HR review before implementation.

## Rejected Alternatives

- **Land the four PRDs as implemented scope.** Rejected because no `oya-community-social-*`, `oya-connect-shorts-*`, retired-network community successor, or `oya-community-anonymous-*` crates ship in PR #130.
- **Use `ADR-0126` for this decision.** Rejected because it conflicts with the inherited Bominal ADR numbering map.
- **Keep BLAKE3/HSM/four-eyes claims as superiority claims.** Rejected because they are planning targets until implementation and threat-model tests land.
- **Treat social/shorts/community-successor/anonymous as one monolith.** Rejected because each surface has distinct context, safety, privacy, scale, and UX contracts.

## Consequences

- Masterplan and platform references cite `ADR-0234`, not `ADR-0126`.
- PR #130 can merge only if it marks unenforced claims as advisory and carries evidence for the planning contract.
- Follow-up implementation must add the dedicated validators before any production readiness or hyperscaler maturity claim can rely on these PRDs.
- Anonymous workplace discussion remains blocked from GA until threat-model tests, legal-hold controls, and aggregate-only UX gates exist.

## Verification

- `cargo run -p oya-dev-cli -- gate validate product-prd-json --product specs/products/connect/social.json --product specs/products/connect/shorts.json --product microservices/community/PRD.md --product specs/products/connect/anonymous.json --product specs/products/connect/suite.json`
- `jq empty docs/machine-readable/decisions.json specs/products/connect/social.json specs/products/connect/shorts.json specs/products/connect/anonymous.json`
- Reviewer-agent check for PR #130 must confirm no unbounded enforcement, cryptography, or cross-context claims remain.
