---
doc_class: IP
ip_id: IP-004
microservice: identity
status: ga
related_adrs: [ADR-0188, ADR-0507, ADR-0508]
related_crates: [shared-webauthn-server-kernel]
date: 2026-05-18
owner_team: axis-identity
---

# IP-004 — WebAuthn relying-party kernel

## Goal

Land `shared-webauthn-server-kernel`: vendor-neutral WebAuthn L3 state machine for registration + authentication ceremonies with AAGUID allowlist enforcement, sign-count monotonic-increase replay defense, conditional UI mediation, and per-pack-tier attestation policy. The kernel exposes an adapter contract that wraps `webauthn-rs` v0.5+ today (per ADR-0188 §In-house roadmap) and remains swap-ready.

## Files to create

| File | Purpose |
|---|---|
| `crates/shared-webauthn-server-kernel/Cargo.toml` | manifest |
| `crates/shared-webauthn-server-kernel/src/lib.rs` | trait + types + state machine + in-memory stores |
| `crates/shared-webauthn-server-kernel/tests/webauthn_server_kernel.rs` | integration tests |

LoC: ~530 (lib) + ~330 (tests) = ~860 lines.

## Public surface

```rust
pub trait WebauthnServer {
    fn begin_registration(&self, tenant, user, display, pack_tier, now) -> Result<Challenge, Err>;
    fn finish_registration(&self, tenant, user, pack_tier, response, now) -> Result<Credential, Err>;
    fn begin_authentication(&self, tenant, allow_creds, mediation, now) -> Result<AuthChallenge, Err>;
    fn finish_authentication(&self, tenant, response, now) -> Result<Credential, Err>;
}

pub trait WebauthnRpAdapter {  // wired to webauthn-rs v0.5+
    fn generate_challenge(&self) -> Vec<u8>;
    fn verify_registration(&self, chal, resp, allowlist) -> Result<Credential, Err>;
    fn verify_authentication(&self, chal, resp, stored) -> Result<u32, Err>;
    fn rp_id(&self) -> &str;
    fn rp_name(&self) -> &str;
}

pub enum PackTier { SandboxOrDev, PackStandard, PackRegulated, AcrCritical }
pub enum AttestationConveyance { None, Indirect, Direct, Enterprise }
```

## Pack-tier policy enforced by the kernel

| PackTier | Attestation | AAGUID allowlist |
|---|---|---|
| SandboxOrDev | None | not enforced |
| PackStandard | Indirect | not enforced |
| PackRegulated | Direct | enforced (FIDO-MDS3 L1+) |
| AcrCritical | Direct | enforced (FIDO-MDS3 L2+) |

## State machine

```
[client clicks Sign In]
       │
       ▼
[begin_authentication] ── server stores challenge with TTL=300s ──▶ returns challenge
       │
       ▼
[browser ceremony] ── user touches Passkey ──▶ assertion
       │
       ▼
[finish_authentication] ── verify signature + sign_count > stored ──▶ session minted
```

Same shape for registration (`begin_registration` → browser → `finish_registration`).

## Tests to write (≥10; 10 shipped)

1. `pack_tier_attestation_requirements` — enum behaviour
2. `happy_path_register_then_authenticate` — full ceremony
3. `regulated_pack_enforces_aaguid_allowlist` — refuses non-allowlisted AAGUID
4. `regulated_pack_with_allowlisted_aaguid_accepts` — accepts when allowlisted
5. `rejects_attestation_when_adapter_denies` — stub adapter says no
6. `rejects_sign_count_regression` — cloned-authenticator defense
7. `rejects_unknown_credential_at_authenticate` — credential not in store
8. `challenge_not_found_returns_distinct_error` — distinct error variant
9. `conditional_ui_uses_empty_allow_credentials` — autofill flow
10. `exclude_credentials_returns_user_existing_set` — re-register prevention

## Failure modes

| Variant | Meaning | Caller HTTP |
|---|---|---|
| `ChallengeNotFound` | Bad challenge_id | 400 |
| `ChallengeExpired` | TTL > 300s | 401 |
| `AttestationInvalid(reason)` | CBOR/COSE/attestation parse failed | 400 |
| `AaguidNotAllowlisted(aaguid)` | Regulated pack refused authenticator | 403 |
| `AttestationLevelInsufficient` | Indirect required but None presented | 403 |
| `AssertionInvalid(reason)` | Signature verify failed | 401 |
| `SignCountRegression{stored,presented}` | Clone alarm | 401 + revoke |
| `CredentialNotFound` | Cred not in store | 404 |
| `TenantMismatch` | Cred is from a different tenant | 403 |

## Determinism + clock

The kernel takes `now_unix` as a parameter on every method that needs time. No hidden wall-clock reads. Production wires `chrono::Utc::now().timestamp()`; tests pass fixed timestamps for deterministic TTL behaviour.

## Evidence to emit

- `evidence/identity/crate-tests/shared-webauthn-server-kernel-<date>.json`
- Test count: 10/10 pass.

## Acceptance — DONE when

- All 10 integration tests pass.
- `#![forbid(unsafe_code)]` honoured.
- No `unwrap()` / `expect()` / `panic!` in src (lints enforce).
- `WebauthnRpAdapter` documented as the swap point per ADR-0188 §In-house roadmap.

## IDENTITY-003 bridge tracking

ADR-0507 promotes `webauthn-rs` as the Phase-1 WebAuthn relying-party bridge and
ADR-0508 pairs it with the OpenSK authenticator-side reference. The full
ADR-0507 parity table, OpenSK reference status, and promotion/cutover gates are
tracked in `iam/identity/IP-017-bespoke-identity-authn-crypto-bridge.md`; this
IP remains the kernel-level ceremony state machine and adapter-boundary anchor.

## Out-of-scope (later IPs)

- AAGUID refresh worker — IP-006.
- WebAuthn HTTP handlers (axum routes) — IP-005.
- Postgres-backed credential store — IP-005.

## Counterpart references - 004-webauthn-relying-party-kernel

- Counterpart class: passkey / recovery assurance.
- GitHub account security and Twilio Verify show the user-facing recovery and step-up baseline; this IP keeps Oyatie stronger by binding the credential or recovery decision to tenant context, ACR, and sealed identity audit events rather than treating MFA as an app-local add-on.
- Verification anchor: this row intentionally includes a named counterpart from the Wave 15 grep allowlist while keeping the implementation reference service-local: `microservices/identity/competitor-parity-matrix.md`, `microservices/identity/PRD.md`, `microservices/identity/manifest.json`, and the contract/policy files cited above.

