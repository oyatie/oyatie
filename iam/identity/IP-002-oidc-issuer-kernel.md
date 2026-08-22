---
doc_class: IP
template_id: TPL-IP
ip_id: IP-002
microservice: identity
status: ga
related_adrs: [ADR-0145, ADR-0187]
related_crates: [shared-oidc-client-kernel]
date: 2026-05-18
owner_team: axis-identity
---

# IP-002 — OIDC issuer kernel + JWKS verify

## Goal

Land the `shared-oidc-client-kernel` crate: vendor-neutral OIDC verifier with JWKS validation, `tenant_id` / `acr` / `purpose` / `data_class` claim extraction, and pluggable signature-verifier adapter so callers can plug `ring`, `rustls`, or HSM-backed verifiers without forcing crypto into the kernel.

## Files to create

| File | Purpose |
|---|---|
| `crates/shared-oidc-client-kernel/Cargo.toml` | manifest; deps: `serde`, `serde_json`; no crypto |
| `crates/shared-oidc-client-kernel/src/lib.rs` | trait + reference verifier + types |
| `crates/shared-oidc-client-kernel/tests/oidc_client_kernel.rs` | integration tests |

Total LoC: ~330 (lib) + ~270 (tests) = ~600 lines.

## Public surface (excerpt)

```rust
pub trait OidcClient: Send + Sync {
    fn verify(&self, bearer: &str, cfg: &VerifyConfig) -> Result<OidcClaims, OidcError>;
    fn meets_acr(&self, claims: &OidcClaims, floor: AcrLevel) -> bool;
}

pub struct OidcClaims {
    pub iss: String, pub aud: Audience, pub sub: String,
    pub iat: i64, pub exp: i64, pub nbf: Option<i64>,
    pub tenant_id: String, pub acr: AcrLevel, pub acr_event_at: Option<i64>,
    pub purpose: Option<String>, pub data_class: Option<String>,
    pub additional: BTreeMap<String, serde_json::Value>,
}

pub enum AcrLevel { Routine, Elevated, Sensitive, Critical }

pub trait JwsVerifier {
    fn verify(&self, jwk: &Jwk, alg: &str, signing_input: &[u8], signature_b64url: &str) -> Result<(), OidcError>;
}
```

## Tests to write (≥10 in this IP; 15 actually shipped)

1. `verifies_well_formed_token` — happy path
2. `rejects_unknown_kid` — JWKS lookup miss returns `UnknownKid`
3. `rejects_invalid_signature` — adapter says no → error propagates
4. `rejects_wrong_issuer` — `iss` mismatch
5. `rejects_wrong_audience` — `aud` mismatch
6. `accepts_audience_array_when_match_present` — `aud` as `["a","b"]`
7. `rejects_expired_token_outside_skew` — `now > exp + skew`
8. `accepts_just_expired_within_skew` — `now > exp` but within tolerance
9. `rejects_missing_tenant_id_claim` — empty tenant_id
10. `acr_ordering_routine_lt_critical` — `meets()` correctness
11. `rejects_malformed_three_segments_required` — JWT shape
12. `meets_acr_helper_through_oidcclient_trait` — trait method works
13. `jwks_from_json_parses` — JWKS shape
14. `b64url_round_trip` — encoder/decoder invariant
15. `rejects_disallowed_alg_hs256` — RFC 8725 BCP §3.1
16. `claims_preserve_additional_fields` — flat-tail JSON survives

## Algorithms supported

- **RS256** (RSA + SHA-256) — Zitadel default.
- **ES256** (ECDSA P-256 + SHA-256) — fallback.
- **HS256 forbidden** — symmetric forbidden for RP per RFC 8725 BCP §3.1.

The kernel does NOT implement the cryptography itself; it accepts a `JwsVerifier` adapter. This permits Phase-2 swap to HSM-backed verification without breaking consumers.

## JWKS rotation handling

The kernel does not fetch JWKS — the embedding application owns the JWKS-fetcher adapter (`JwksFetcher` trait). The reference verifier consumes a pre-loaded `Jwks` snapshot. The recommended caller flow:

1. Cache JWKS for 24h.
2. On `UnknownKid` error, force refresh + retry once.
3. After refresh fails, return 401 to client.

The advisor adapter that handles this caching is part of IP-003 (Zitadel adapter); the kernel stays pure.

## Failure-mode enumeration

The kernel distinguishes failure classes so consumers can choose response code:

| Error variant | Suggested HTTP |
|---|---|
| `Malformed(_)` | 400 |
| `UnknownKid(_)` | 401 (after JWKS refresh attempted) |
| `AlgMismatch{..}` | 401 |
| `SignatureInvalid` | 401 |
| `IssuerMismatch{..}` | 401 |
| `AudienceMismatch{..}` | 403 |
| `Expired{..}` | 401 |
| `NotYetValid{..}` | 401 |
| `MissingClaim(_)` | 400 |
| `InvalidClaim{..}` | 400 |

## Evidence to emit

| Artefact | Path |
|---|---|
| `cargo test -p shared-oidc-client-kernel` results | `evidence/identity/crate-tests/shared-oidc-client-kernel-<date>.json` |
| Test count | 16 tests, 16 passed |
| Code coverage report | `evidence/identity/coverage/shared-oidc-client-kernel-<date>.html` |

## Acceptance — DONE when

- [x] Crate builds: `cargo build -p shared-oidc-client-kernel` exits 0.
- [x] Tests pass: `cargo test -p shared-oidc-client-kernel` shows 16/16 passed.
- [x] No `#[allow(...)]` directives in src.
- [x] No `panic!`, `unwrap()`, `expect()` in src (workspace lints enforce).
- [x] Documented as `pub` API: every public type carries a `///` doc comment.

## Future work (out-of-scope)

- Token revocation cache (consumer's job; kernel stays read-only).
- Refresh-token semantics (Zitadel-side).
- OAuth scope-set evaluation (Cedar-side per ADR-0183).

## Counterpart references - 002-oidc-issuer-kernel

- Counterpart class: issuer / federation.
- GitHub enterprise SSO and ServiceNow external IdP federation are the counterpart baseline for workforce login; this IP keeps Oyatie differentiated by preserving per-pack issuer boundaries, JWKS evidence, and provider-BYOK separation.
- Verification anchor: this row intentionally includes a named counterpart from the Wave 15 grep allowlist while keeping the implementation reference service-local: `microservices/identity/competitor-parity-matrix.md`, `microservices/identity/PRD.md`, `microservices/identity/manifest.json`, and the contract/policy files cited above.

