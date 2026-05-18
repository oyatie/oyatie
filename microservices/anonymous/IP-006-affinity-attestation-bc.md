---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-shared-substrate
phase: P02-anonymous-foundation
impl_plan_id: IP-006-affinity-attestation-bc
status: pending
execution_unit: ChangeSet
owner: axis-anonymous + ops-security
acceptance_lanes: [cargo-check, cargo-test, oya-governance-k-anonymity-floor, oya-governance-identity-attribute-discard]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: Affinity-attestation BC end-to-end (OIDC + SAML + ZK-proof)

## Intent

Author the full affinity-attestation vertical slice: kernel + domain + usecase + adapter-oidc + adapter-saml + rest + sdk. Implements the BBS+ selective-disclosure flow per ADR-ANON-0002 + `policy/affinity-attestation-verification.md`.

## ChangeSet

- 8 crates per layer
- Issuer registry (Postgres) + key-management
- OIDC adapter (corporate IdPs)
- SAML adapter (enterprise IdPs)
- ZK-proof verifier (rust-bls 0.5)

## Acceptance

- BBS+ verify roundtrip test passes
- k-anonymity floor refusal test passes (k=50 / k=20 / k=10 thresholds)
- Identity-attribute discard test passes (PRD I2 invariant)
- Performance: p95 ≤ 500ms per PRD SLO
