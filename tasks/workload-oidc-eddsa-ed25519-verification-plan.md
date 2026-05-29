# Plan: workload-oidc-eddsa-ed25519-verification

## Objective

Add EdDSA/Ed25519 (RFC 8037) JWS verification to `oya-identity-workload-oidc-adapter` alongside the existing RS256/384/512 + ES256 support, using `aws-lc-rs` as the sole crypto backend.

## Requirements Analysis

### Standards

- **RFC 8037** — CFRG Elliptic Curves for JOSE: defines OKP key type, `crv=Ed25519`, `x` (32-byte base64url raw public key), no `y`. The `alg=EdDSA` value covers all OKP curves.
- **RFC 8725** — JWT Best Current Practices: existing guards (none rejection, HS* key-confusion, typ check, jku/x5u guard, alg-family binding) must apply equally to EdDSA.
- **RFC 7517/7518** — JWK/JWA: OKP key type (`kty=OKP`) with `crv` discriminator.
- **aws-lc-rs** — `signature::ED25519` constant + `UnparsedPublicKey::new(&ED25519, raw_32_bytes).verify(msg, sig)`. Public key is the raw 32-byte Edwards point. Signature is 64 bytes.

### Edge Cases

1. `alg=EdDSA` with a non-Ed25519 `crv` (e.g. Ed448) — reject as `UnsupportedAlgorithm` since we only support `crv=Ed25519`.
2. `x` coordinate not exactly 32 bytes after base64url decode — `MalformedKey`.
3. Signature not exactly 64 bytes — `SignatureInvalid` (aws-lc-rs will reject).
4. An `EdDSA` token presented against an RSA or EC P-256 JWK — `AlgorithmMismatch` (family mismatch guard).
5. RSA/EC token presented against an OKP JWK — `AlgorithmMismatch` (family mismatch guard).
6. `alg` pin: a JWK with `alg=RS256` presented against an EdDSA token — `AlgorithmMismatch`.

### Acceptance Criteria

- `JwsAlg::EdDsa` variant parses `"EdDSA"` string.
- `AlgFamily::OkpEd25519` variant; `JwsAlg::EdDsa.family()` returns it.
- `JwkMaterial::OkpEd25519 { x: String }` variant with raw 32-byte base64url public key.
- `Jwk::okp_ed25519(kid, x)` constructor.
- `Jwk::family()` returns `AlgFamily::OkpEd25519` for the new variant.
- `verify_signature` dispatches to `signature::ED25519` path for `EdDsa` + `OkpEd25519`.
- All existing RFC 8725 guards apply to EdDSA tokens.
- `src/eddsa.rs` contains deterministic Ed25519 signing harness + tests mirroring the ES256 harness.
- All existing tests continue to pass.
- New tests: valid Ed25519 token validates to active principal, tampered payload fails, algorithm mismatch (EdDSA against RSA/EC key), alg pin respected.

## Ordered Subtasks

1. **Write plan** (this file).
2. **Write spec** (`docs/specs/task-workload-oidc-eddsa-ed25519-verification.md`).
3. **RED phase**: Add `src/eddsa.rs` with failing test stubs; run `cargo check --all-targets` to confirm compile-fail.
4. **GREEN phase**: Extend `src/lib.rs` with `JwsAlg::EdDsa`, `AlgFamily::OkpEd25519`, `JwkMaterial::OkpEd25519 { x }`, `Jwk::okp_ed25519`, `verify_signature` EdDSA branch; implement `src/eddsa.rs` signing harness + tests. Run `cargo nextest run -p oya-identity-workload-oidc-adapter`.
5. **REVIEW**: Check correctness, security (RFC 8725 guards), performance, cloud-native readiness. Fix criticals.
6. **SIMPLIFY**: Behavior-preserving cleanup; re-run nextest.
7. **COMMIT + SHIP**: Scoped `git add`; push; open PR.
