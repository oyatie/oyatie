---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-013-age-verification-and-profile-verification
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social + council-privacy
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-pack-aware-age-gate]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-013: age-verification + profile-verification BCs

## Intent

Two privacy-sensitive BCs in one ChangeSet because they share signup-flow
integration and minor-protection regulatory cross-mapping:

- **age-verification**: pack-aware age-gate at signup; minor-account flow
  with parental consent attestation; isolated `social_age_attestations`
  Postgres table with Cedar-restricted access (per FM-15 mitigation).
- **profile-verification**: verification badge issuance + revocation;
  per-tenant policy (handle uniqueness, trademark reservation, government /
  organisation verification); audit-chain seal per badge mutation.

## ChangeSet boundary

`age-verification` + `profile-verification` BCs.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-social-age-verification-kernel/src/{ports,entities,errors}.rs` | create |
| `src/crates/oya-social-age-verification-domain/src/{age_attestation,age_bracket,minor_protection_policy}.rs` | create |
| `src/crates/oya-social-age-verification-usecase/src/{attest,verify_minor,parental_consent}.rs` | create |
| `src/crates/oya-social-age-verification-adapter-postgres/src/repository.rs` | create — isolated table |
| `src/crates/oya-social-age-verification-adapter-postgres/migrations/0001_init.sql` | create |
| `src/crates/oya-social-profile-verification-kernel/src/{ports,entities,errors}.rs` | create |
| `src/crates/oya-social-profile-verification-domain/src/{verification_request,verification_badge,revocation_event}.rs` | create |
| `src/crates/oya-social-profile-verification-usecase/src/{issue,revoke}.rs` | create |
| `src/crates/oya-social-profile-verification-adapter-postgres/src/repository.rs` | create |
| `tests/age_gate_pack_eu.rs` | create — AC-10 E2E |
| `tests/age_gate_pack_kr.rs` | create |
| `tests/profile_verification_e2e.rs` | create |

## Pack-Aware Age Gates

| Pack | Threshold | Source |
|---|---|---|
| pack-eu | 16y (member states may lower to 13y) | GDPR Art. 8 |
| pack-us | 13y | COPPA 15 USC §6501 |
| pack-us-healthcare | 13y (COPPA) + HIPAA-eligibility flag | COPPA + HIPAA §164.502(g) |
| pack-kr | 14y | KR 청소년 보호법 + PIPA Art. 8 |
| pack-jp | 13y | APPI |
| pack-sg | 13y | PDPA |
| pack-au | 13y | Privacy Act + Online Safety Act 2021 |
| pack-in | 18y | DPDPA 2023 §9 (note: stricter than most packs) |
| pack-br | 12y | LGPD Art. 14 |
| pack-ae | 13y | UAE PDPL |
| pack-ksa | 13y | PDPL |

## Acceptance Gates

```bash
cargo nextest run -p oya-social-age-verification-kernel
cargo nextest run -p oya-social-profile-verification-kernel
cargo run -p oya-dev-cli -- gate validate pack-aware-age-gate --microservice social
```

## Test Plan

- AC-10 E2E: minor signup on pack-eu requires parental consent attestation.
- pack-in highest threshold (18y) enforced.
- Age-attestation table access bound by Cedar `age_verification_reader` entitlement (FM-15 mitigation).
- Verification badge issued + revoked → audit-chain seal.
- Handle uniqueness scope per ADR-SOC successor-IP (PRD Open Question 5).

## Halt Conditions

- Age-attestation table access by non-entitled principal — Sev-1 (FM-15).

## Next IP

[`IP-014-observability-slo.md`](IP-014-observability-slo.md)
