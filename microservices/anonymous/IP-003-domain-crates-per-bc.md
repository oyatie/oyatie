---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-shared-substrate
phase: P02-anonymous-foundation
impl_plan_id: IP-003-domain-crates-per-bc
status: pending
execution_unit: ChangeSet
owner: axis-anonymous
acceptance_lanes: [cargo-check, cargo-test, oya-governance-layer-purity]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003: Domain crates per BC (pure logic)

## Intent

Author 11 domain crates implementing pure business logic for each BC. Domain crates depend on kernel only; they contain functions that operate on kernel types (e.g., `compute_wilson_lower_bound`, `verify_k_anonymity_floor`, `compose_moderation_envelope`).

## Crate set

One per BC: `oya-anonymous-<bc>-domain` (11 crates).

## Notable algorithms

- **vote-engine-domain**: Wilson lower-bound ranking + reddit-style time decay (per ADR-COMM-0002 inherited)
- **affinity-attestation-domain**: k-anonymity floor enforcement (k=50 geo / k=20 employer / k=10 small-employer fallback) per Sweeney 2002
- **content-moderation-domain**: chain-of-responsibility composition per ADR-COMM-0001 inherited
- **retention-policy-domain**: 30/60/90-day tier arithmetic + hard-delete worker scheduling
- **blind-signatures-domain**: protocol selection + commitment math (without holding any secret key)

## Acceptance

- 11 domain crates compile
- Property tests pass: voting idempotency, k-anonymity floor, retention tier transitions
