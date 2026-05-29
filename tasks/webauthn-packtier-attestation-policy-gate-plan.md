# Plan: webauthn-packtier-attestation-policy-gate

## Goal

Add a pure deterministic `evaluate_attestation_policy` function to `oya-shared-webauthn-server-kernel` that enforces `PackTier`-level attestation-conveyance + AAGUID-allowlist policy against a `Credential`. Additive only — no changes to existing traits, ceremony logic, or sign-count paths.

## Steps

1. Add `src/attestation_policy.rs` mod with `PolicyInput`, `PolicyVerdict`, and `evaluate_attestation_policy` fn.
2. Re-export the new types from `src/lib.rs`.
3. Add `#[test]` cases in `tests/webauthn_server_kernel.rs` covering:
   - All four tiers with matching conveyance (admit)
   - Conveyance under-provisioning (reject)
   - Zero-AAGUID rejection for regulated/critical tiers
   - Non-zero AAGUID not in allowlist (reject)
   - Non-zero AAGUID in allowlist (admit)
   - SandboxOrDev and PackStandard ignore allowlist
4. `cargo check -p oya-shared-webauthn-server-kernel --all-targets` — green
5. `cargo nextest run -p oya-shared-webauthn-server-kernel` — green
6. Self-review: fix any Critical/High issues
7. Simplify if needed; re-run nextest

## Acceptance criteria (from spec)

- No new deps (serde-only)
- Panic-free
- Tests cover all four tiers, zero-AAGUID rejection for regulated/critical, allowlist hit vs miss, attestation-conveyance under-provisioning rejection
- Existing `WebauthnServer`/adapter traits and sign-count logic untouched (additive)
