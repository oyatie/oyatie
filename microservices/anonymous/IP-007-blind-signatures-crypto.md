---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-shared-substrate
phase: P02-anonymous-foundation
impl_plan_id: IP-007-blind-signatures-crypto
status: pending
execution_unit: ChangeSet
owner: ops-security + axis-anonymous
acceptance_lanes: [cargo-check, cargo-test, oya-governance-fips-boundary-lint, oya-governance-cryptographic-vector-test]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: Blind-signatures crypto crate (BBS+ + ring 0.17 behind feature flag)

## Intent

Author the load-bearing cryptographic primitive crate per ADR-ANON-0001. Two adapters behind feature flags:
- `adapter-rust-bls` (default): BBS+ over BLS12-381 (FIPS 140-3 Level 3 in air-gapped HSM path)
- `adapter-ring`: Ed25519 issuer-key registration (FIPS 140-3 Level 1)

## ChangeSet

- `src/blind-signatures-kernel/*`
- `src/blind-signatures-domain/*`
- `src/blind-signatures-usecase/*`
- `src/blind-signatures-adapter-rust-bls/*`
- `src/blind-signatures-adapter-ring/*`

## Acceptance

- IRTF CFRG `draft-irtf-cfrg-bbs-signatures` vector tests pass
- FIPS boundary lint passes
- Sign-verify roundtrip < 200ms p95
- Selective-disclosure proof verify < 100ms p95
- Key-ceremony hooks tested (Shamir 3-of-5 split + reconstitution)
