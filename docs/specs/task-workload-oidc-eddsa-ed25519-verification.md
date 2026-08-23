# Spec: workload-oidc-eddsa-ed25519-verification

## Objective

Extend `identity-workload-oidc-adapter` with EdDSA/Ed25519 (RFC 8037) JWS verification so that OIDC issuers that publish OKP Ed25519 JWKs can issue workload tokens verified by this adapter. All existing RS256/384/512 and ES256 paths remain unchanged.

## Crate Boundary

- Crate: `crates/identity-workload-oidc-adapter`
- No workspace member additions; no root `Cargo.toml` edits.
- Crypto backend: `aws-lc-rs` (already a dependency, `signature::ED25519` + `Ed25519KeyPair`).

## Flat-Clean-Architecture Mod Layout (ADR-0509)

```
src/
  lib.rs       — all validation logic + new EdDSA enum variants + OKP JWK material (extended)
  eddsa.rs     — EdDSA/Ed25519 signing harness + deterministic tests (cfg(test) only)
```

`eddsa.rs` is gated with `#[cfg(test)]` at the module declaration and exists solely to house the Ed25519 signing harness and tests (analogous to the ES256 harness already inline in `lib.rs`). This keeps the Ed25519-specific test infrastructure in its own focused file.

## Contract Changes

### `JwsAlg` enum — new variant

```rust
EdDsa,  // "EdDSA" — RFC 8037 Ed25519 over OKP keys
```

### `AlgFamily` enum — new variant

```rust
OkpEd25519,  // OKP crv=Ed25519 key family (RFC 8037)
```

### `JwkMaterial` enum — new variant

```rust
/// OKP Ed25519 public key: raw 32-byte base64url coordinate `x` (RFC 8037 §2).
OkpEd25519 {
    /// base64url-encoded 32-byte raw public key (the Edwards point).
    x: String,
},
```

### `Jwk` — new constructor

```rust
pub fn okp_ed25519(kid: impl Into<String>, x: impl Into<String>) -> Self
```

### `verify_signature` — new dispatch branch

- `(JwkMaterial::OkpEd25519 { x }, JwsAlg::EdDsa)` — decode `x` (must be 32 bytes), call `signature::UnparsedPublicKey::new(&signature::ED25519, x_bytes).verify(message, sig)`.
- Signature is the raw 64-byte Edwards signature (as produced by RFC 8037 / JWS).

## RFC 8725 Guard Applicability

| Guard | Applies to EdDSA? |
|---|---|
| `none` rejection (§2.1/§3.1) | Yes — unchanged pre-alg check |
| `HS*` key-confusion (§2.2/§3.1) | Yes — unchanged prefix check |
| `typ` check (§3.11) | Yes — same `accepted_token_types` config |
| `jku`/`x5u` SSRF guard (§3.8) | Yes — same `trusted_key_source_urls` config |
| alg-family binding (§2.2/§3.1) | Yes — `OkpEd25519` family; RSA/EC/OKP cross-use refused |

## Testing Strategy

### Unit tests (`src/lib.rs` inline or `src/eddsa.rs`)

- `valid_ed25519_token_projects_to_active_principal` — mint a genuine EdDSA token with `aws-lc-rs::signature::Ed25519KeyPair::generate()`, validate, assert principal fields.
- `tampered_ed25519_payload_fails_signature` — forge the payload, assert `SignatureInvalid`.
- `eddsa_against_rsa_kid_is_algorithm_mismatch` — present EdDSA token against RSA JWK.
- `eddsa_against_ec_kid_is_algorithm_mismatch` — present EdDSA token against EC P-256 JWK.
- `rsa_token_against_okp_kid_is_algorithm_mismatch` — present RS256 token against OKP JWK.
- `ed25519_alg_pin_mismatch_is_rejected` — JWK pinned to `RS256`, EdDSA token rejected.
- `ed25519_alg_pin_match_is_accepted` — JWK pinned to `EdDSA`, EdDSA token accepted.
- `malformed_okp_x_coord_is_rejected` — x not 32 bytes → `MalformedKey`.

### Integration tests (`tests/token_validation.rs`)

No changes required — existing integration tests remain in place.

## Observability / SLO

No new SLO metrics introduced. The `OidcValidationError` variants are sufficient for audit-chain telemetry at call sites.

## Cloud-Native / K8s Considerations

- Ed25519 keys are commonly used by issuers running on K8s (e.g. Kubernetes service account token signing with OIDC provider configuration `--service-account-key-file`). Supporting EdDSA broadens issuer compatibility for the workload identity stack.
- No external network calls; JWKS is static, injected at construction time.
