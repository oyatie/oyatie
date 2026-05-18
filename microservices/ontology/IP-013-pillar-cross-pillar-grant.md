---
doc_class: ImplementationPlan
ip_id: IP-013
title: pillar (org / person pillar enforcement + cross-pillar grant Cedar flow)
microservice: ontology
phase: P01-typed-entity-substrate
status: pending
owner_team: axis-ontology + council-privacy
date: 2026-05-17
depends_on: [IP-006]
acceptance_lanes:
  - cargo-check
  - cargo-clippy
  - cargo-nextest
  - oya-foundry-fitness-cedar-coverage
related_artifacts:
  - microservices/ontology/src/crates/oya-ontology-pillar-{kernel,domain,usecase}/
  - microservices/ontology/policy/pillar.cedar
doc_status: published
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-013: pillar (org / person + cross-pillar grant)

## Intent

Author the pillar BC per Bominal ADR-0132 — typed Object Types declared at `org-pillar` or `person-pillar`; cross-pillar reads forbidden unless explicit Cedar `CrossPillarGrant` issued via 2-person rule.

## Scope

In-scope:
- `oya-ontology-pillar-{kernel,domain,usecase}` crates (no adapter — pure logic via Cedar).
- `pillar.cedar` policy fragment authored.
- Cross-pillar grant data model: principal, allowed_pillars, data_class_cap, expires_at (≤ 30 d), signed_by[2].
- 2-person rule enforcement at grant issuance.
- Audit-chain emit on grant issued + used + revoked + expired.

## Implementation

| Step | Action |
|---|---|
| 1 | Scaffold 3 crates |
| 2 | Author `pillar.cedar` policy fragment |
| 3 | Author cross-pillar grant data model + issuance flow |
| 4 | Wire 2-person rule (Cedar policy requires `signed_by_two_principals` claim) |
| 5 | Tests: cross-pillar read without grant refused; with grant permitted; expired grant refused |

## Verification

- `cargo nextest run -p oya-ontology-pillar-domain --test pillar_isolation` — exit 0.
- `oya gate validate cedar-coverage --microservice ontology` includes pillar.cedar — exit 0.
- 2-person rule: grant with 1 signatory refused.

## References

- Bominal ADR-0132 (pillars).
- ADR-0140 (retired per ADR-0145) (Cedar).
- `microservices/ontology/policy/type-isolation.md` §"Pillar Isolation Invariants" TI-11..TI-13.
