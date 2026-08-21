---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-tenancy-substrate-stable
impl_plan_id: IP-007-isolation-policy-jwt-issuer
status: in-progress
owner: axis-tenancy + ops-security
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-jwt-key-fingerprint-advertised]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: Isolation-policy JWT issuer + verifier

## Intent

> **Delivery note (2026-08-20).** Implemented in tenancy/core/isolation-policy as `tenancy-isolation-policy` (module `src/claims.rs`), collapsed into that ONE crate
> as a module tree rather than this plan's multi-crate fan-out: the capability is capped at 12 crates
> and `Cargo.lock` is a hub path owned by `integ/build`, so neither a new crate nor a new dependency
> was available to this lane. Landed: the tenant-scoped claims model and its validation rules over an issuer port. Deferred and named as a gap in the crate's `lib.rs` header:
> signature signing and verification, which need a crypto dependency the frozen lockfile forbids — this module validates claim SHAPE only. The crate names in the tables below are this plan's original
> proposal, not what shipped.


JWT issuance + verification subsystem within `oya-tenancy-isolation-policy-*`. Ed25519 signing key from OpenBao; 30d rotation cadence; old pubkey 30d grace; fingerprint advertised via Workflow. Algorithm-confusion-attack hardening (Invariant JWT-01).

## Concrete File Targets

| Path | Action |
|---|---|
| `oya-tenancy-isolation-policy-kernel/src/jwt_ports.rs` | update — `JwtIssuer`, `JwtVerifier`, `SigningKeyStore` traits |
| `oya-tenancy-isolation-policy-adapter/src/jwt_issuer.rs` | create — OpenBao-backed Ed25519 sign |
| `oya-tenancy-isolation-policy-adapter/src/jwt_verifier.rs` | create — local pubkey cache; refresh on JwtSigningKeyRotated event |
| `oya-tenancy-isolation-policy-adapter/src/openbao_signing_key_store.rs` | create — Ed25519 keypair fetch + rotation cron |
| `oya-tenancy-isolation-policy-worker/src/rotation_worker.rs` | create — 30d rotation + fingerprint Workflow emit |
| `oya-tenancy-isolation-policy-rest/src/jwt_routes.rs` | create — `POST /jwts`, `POST /jwts/verify`, `GET /jwt-fingerprints` |

## Code Shape

```rust
// jwt_verifier.rs
pub fn verify_jwt(jwt: &str, pubkeys: &PubKeyCache) -> Result<JwtClaims, JwtError> {
    use jsonwebtoken::{Algorithm, Validation, decode, DecodingKey};
    let header = jsonwebtoken::decode_header(jwt)?;
    // Invariant JWT-01: explicit alg whitelist
    if header.alg != Algorithm::EdDSA {
        return Err(JwtError::AlgorithmRejected);  // refuse alg=none, HS*, RS*
    }
    let kid = header.kid.ok_or(JwtError::MissingKid)?;
    let pubkey = pubkeys.lookup(&kid).ok_or(JwtError::UnknownKey)?;
    let mut validation = Validation::new(Algorithm::EdDSA);
    validation.set_issuer(&[format!("oya-tenancy-{pack}-{env}")]);
    validation.set_audience(&["oyatie-internal"]);
    let token = decode::<JwtClaims>(jwt, &DecodingKey::from_ed_der(pubkey), &validation)?;
    Ok(token.claims)
}
```

```rust
// rotation_worker.rs
pub async fn rotation_cycle(deps: &Deps) -> anyhow::Result<()> {
    let new_keypair = deps.openbao.generate_ed25519(/* pack, env */).await?;
    let prev_fingerprint = deps.openbao.current_fingerprint(/* pack, env */).await?;
    let new_fingerprint = new_keypair.fingerprint();
    deps.openbao.promote_previous(/* pack, env */).await?;  // old → previous mount
    deps.openbao.install_current(new_keypair).await?;       // new → current mount
    deps.event_sink.emit(JwtSigningKeyRotatedEvent {
        pack, env, prev_fingerprint, new_fingerprint, emergency: false, revoke_previous: false, rotated_at: now(), ..
    }).await?;
    deps.audit_chain.seal(...).await?;
    Ok(())
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-tenancy-isolation-policy-adapter --test jwt_verifier
cargo nextest run -p oya-tenancy-isolation-policy-worker --test jwt_rotation
cargo run -p oya-dev-cli -- gate validate jwt-key-fingerprint-advertised
```

## Test Plan

- `test_alg_none_rejected`, `test_alg_hs256_rejected`, `test_alg_rs256_rejected`, `test_alg_eddsa_accepted` (algorithm-confusion-attack defence).
- `test_invalid_iss_rejected`, `test_invalid_aud_rejected`, `test_expired_jwt_rejected`.
- `test_rotation_emits_fingerprint_event`.
- Pen-test stub: synthetic forgery attempts.

## Halt Conditions

- Any verifier code path accepts `alg != EdDSA` — refuse merge.
- Rotation cron does not emit fingerprint event — refuse merge.

## Next IP

[`IP-008-cell-assignment-controller.md`](IP-008-cell-assignment-controller.md)
