---
doc_class: IP
ip_id: IP-003
microservice: identity
status: ga
related_adrs: [ADR-0187, ADR-0145]
date: 2026-05-18
owner_team: axis-identity
---

# IP-003 — OIDC issuer Zitadel adapter

## Goal

Wire the `oya-shared-oidc-client-kernel` trait to a concrete Zitadel-talking adapter that fetches JWKS via Zitadel `/oauth/v2/keys`, verifies RS256 / ES256 with `aws-lc-rs`, caches JWKS for 24h with on-miss refresh, exposes JWKS refresh hook for emergency rotation, and emits audit events for verification failures that have security significance (signature-invalid, expired, audience-mismatch).

## Files to create

| File | Purpose |
|---|---|
| `crates/oya-identity-oidc-issuer-adapter-zitadel/Cargo.toml` | adapter crate manifest; deps: `aws-lc-rs`, `reqwest`, kernel |
| `crates/oya-identity-oidc-issuer-adapter-zitadel/src/lib.rs` | `ZitadelOidcAdapter` implementing `OidcClient` |
| `crates/oya-identity-oidc-issuer-adapter-zitadel/src/jwks_cache.rs` | TTL'd cache with refresh-on-miss |
| `crates/oya-identity-oidc-issuer-adapter-zitadel/src/aws_lc_verifier.rs` | `JwsVerifier` impl using aws-lc-rs |
| `crates/oya-identity-oidc-issuer-adapter-zitadel/tests/zitadel_adapter.rs` | integration tests with mock Zitadel server |

## Adapter responsibilities

1. **JWKS fetch**: GET `https://identity-<pack>.oyatie.dev/oauth/v2/keys` with mTLS; parse into `Jwks`.
2. **Cache**: 24h TTL; on `UnknownKid` error from kernel, refresh-once then retry.
3. **Signature verify**: RS256 (RSA-PKCS1-v1.5-SHA-256) and ES256 (ECDSA-P256-SHA-256) via aws-lc-rs.
4. **Discovery probe**: GET `/.well-known/openid-configuration`; verify `issuer` matches pack-pinned issuer URL.
5. **Audit emission**: on `SignatureInvalid` or `Expired`, emit `IdentityOidcVerifyFailed` event to audit-emitter.

## Tests to write

| Test | Mechanism |
|---|---|
| `verifies_against_live_zitadel_mock` | spawn mock server with known signing key + JWKS |
| `refreshes_jwks_on_unknown_kid` | issue token with rotated kid; assert JWKS refresh happened once |
| `does_not_infinite_refresh` | even on persistent unknown kid, refresh only once per second |
| `rejects_when_zitadel_discovery_issuer_mismatch` | mock returns wrong issuer; adapter refuses bootstrap |
| `rs256_signature_valid_path` | known good RS256 token verifies |
| `es256_signature_valid_path` | known good ES256 token verifies |
| `signature_tampering_rejected` | flip one bit in payload → signature invalid |
| `expired_token_emits_audit` | verify a 5-minute-expired token; audit emission observed |
| `audience_mismatch_emits_audit` | verify with wrong audience; audit emission observed |
| `mtls_required_to_jwks_endpoint` | non-mTLS fetch is refused |

## Failure-mode handling

- **JWKS endpoint down**: serve from cache up to 7×TTL; alert at 2×TTL.
- **Discovery mismatch on bootstrap**: refuse to start; ops-security pages.
- **Signature verify exception**: panic forbidden; map to `OidcError::SignatureInvalid`.

## Evidence to emit

- `evidence/identity/adapter-tests/zitadel-<date>.json`
- `evidence/identity/jwks-cache-stats/<pack>/<date>.json` (cache-hit-rate, refresh-count)
- `evidence/identity/discovery-probe/<pack>/<date>.json`

## Acceptance — DONE when

- 10+ tests passing, including the mock-server integration test.
- aws-lc-rs version pinned to ≥1.x LTS; vendor-recency lane clean.
- Discovery probe matches `https://identity-<pack>.oyatie.dev` for every pack.
- JWKS fetch hot path p99 < 8ms.

## Cross-references

- ADR-0187 §"What Zitadel issues"
- ADR-0173-vendor-lock-in §"open-standard preservation"
- RFC 8725 (JWT BCP)
