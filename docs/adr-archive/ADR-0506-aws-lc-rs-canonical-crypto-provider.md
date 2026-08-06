---
id: ADR-0506
status: Superseded
planning_impact: false
deciders: founder, council-architecture
date: 2026-05-28
owner: council-architecture
supersedes: []
superseded_by: [ADR-709]
related: [ADR-0482]
door: two-way
---

# ADR-0506 — aws-lc-rs canonical crypto provider (Phase-1) + oya-crypto Tier-4 bespoke destination

## Status

Accepted (2026-05-28).

## Context

`ring` has been the de facto Rust crypto backend across oyatie services — used
directly for JWS signature verification (ECDSA/RSA), SHA-256 fingerprinting, and
constant-time credential comparison. It is the default rustls backend until rustls
v0.23 introduced an explicit provider-selection API.

`aws-lc-rs` is AWS's drop-in replacement for `ring`, backed by AWS-LC — a
hardened, FIPS-validatable fork of BoringSSL that AWS uses internally across
its own infrastructure. It exports ring-compatible module paths (`digest`,
`signature`, `constant_time`, `rand`, `aead`), enabling in-place migration with
minimal call-site churn.

Rustls v0.27+ supports selecting the crypto provider via a feature flag: replacing
`features = ["ring"]` with `features = ["aws-lc-rs"]` on `rustls`/`hyper-rustls`
is a one-line swap per crate.

Founder direction 2026-05-28: "https://github.com/aws/aws-lc-rs is likely more
performant than ring as a dependency. we can plan to replace with our own crypto
lib later."

Per [[bespoke-over-oss-doctrine]], this establishes the Phase-1 (OSS bridge) →
Tier-4 (bespoke oya-crypto) phasing pattern. oya-crypto is the long-term
destination, unlocked after kubers Phase-B kernel proofs land and FIPS 140-3
module validation completes.

## Hyperscaler-lens pre-check

| Criterion | Result |
|---|---|
| Active upstream | PASS — AWS-funded, actively maintained at github.com/aws/aws-lc-rs |
| License clean | PASS — Apache-2.0 + ISC; OSI-clean; no SSPL/BSL/Commons-Clause |
| Fully self-hostable | PASS — pure library; no managed service dependency; links against AWS-LC C library vendored in-crate |
| Hyperscaler-internal equivalent | PASS — AWS literally uses AWS-LC internally; this is the hyperscaler-internal pattern |

## Decision

1. **aws-lc-rs is the canonical Phase-1 crypto backend** across all oyatie Rust
   services. It replaces `ring` as the workspace-level crypto primitive.

2. **rustls/hyper-rustls configured with the `aws-lc-rs` provider feature**:
   `features = ["ring", ...]` → `features = ["aws-lc-rs", ...]` on all
   `hyper-rustls` (and future `rustls`) dep declarations.

3. **Direct `ring` deps in prod code migrated to `aws-lc-rs`**: call sites use
   `aws_lc_rs::` module prefix. API is compatible (same `digest`, `signature`,
   `constant_time` paths).

4. **`ring` retained in dev-dependencies only** where test code mints JWT tokens
   using `ring::signature::EcdsaKeyPair` (key-generation ergonomics differ in
   aws-lc-rs v1). These are marked with `TODO(ADR-0506)` for follow-up migration.

5. **oya-crypto is the Tier-4 bespoke destination** (see ADR-0482 Bespoke
   Substrate Roadmap). It lives alongside kubers Phase-B (proof-gated Rust kernel)
   and is NOT admissible before those gates pass.

6. **`aws-lc-rs` is the bridge indefinitely** — it is the FIPS path without
   bespoke validation work, providing immediate operational value.

## Feature parity target for future oya-crypto

Required per [[bespoke-over-oss-doctrine]] — every bespoke ADR must include this
table. oya-crypto must reach minimum parity before migration cutover from
aws-lc-rs is considered.

| Feature | OSS-substrate (Phase-1: aws-lc-rs) | Bespoke minimum bar (oya-crypto) | Phase |
|---|---|---|---|
| AEAD | AES-128-GCM, AES-256-GCM, ChaCha20-Poly1305 | Same + AES-256-GCM-SIV | 4 |
| Key exchange | X25519, P-256 ECDH | Same + X448, P-384 | 4 |
| Signatures | Ed25519, ECDSA P-256/P-384, RSA-PSS | Same + Ed448, ML-DSA (post-quantum) | 4 |
| Hash | SHA-256, SHA-384, SHA-512, SHA3 family | Same + BLAKE3 | 4 |
| KDF | HKDF, PBKDF2 | Same + Argon2id | 4 |
| MAC | HMAC | Same + KMAC | 4 |
| Hardware accel | AES-NI, SHA-NI, ARMv8 crypto ext | Same + GPU offload, RDMA inline | 4 |
| Constant-time | All primitives | Formal verification (Kani/Creusot proofs) | 4 |
| FIPS | FIPS 140-3 validatable (AWS-LC path) | FIPS 140-3 + formal proof artifacts | 4 |
| Post-quantum | Kyber768 (hybrid) | ML-KEM full + ML-DSA + hybrid migration | 4 |

## Bridge and migration

aws-lc-rs is the bridge indefinitely. Cutover to oya-crypto is gated on:

- (a) kubers Phase-B kernel proofs landing (Rust hardware kernel admissible)
- (b) oya-crypto FIPS 140-3 module validation complete
- (c) hyperscaler-readiness gate #10 (security isolation) conformance evidence
  per `/Users/jasonlee/Developer/kubers` gate definitions

## Consequences

- rustls/hyper-rustls config change repo-wide (`"ring"` feature → `"aws-lc-rs"`)
- `ring` prod deps removed from `oya-identity-workload-oidc-adapter` and
  `oya-llm-gateway-rest`; `aws-lc-rs` added to `[workspace.dependencies]`
- Source call sites updated: `ring::` → `aws_lc_rs::` in prod code
- `ring` retained as dev-dependency in `oya-identity-workload-rest` and
  `oya-identity-workload-app` for test JWT minting (deferred call-site migration
  marked `TODO(ADR-0506)`)
- Lockfile churn: aws-lc-rs pulls in the vendored AWS-LC C library (build-time
  only; no runtime managed dependency)
- ~5-15% TLS handshake throughput improvement expected (AWS published benchmarks
  at github.com/aws/aws-lc-rs)
- FIPS 140-3 path available without bespoke validation work — enables future
  compliance posture at no additional cost
- **Zero-ring realized (G002, 2026-06-13):** the final ring-feature activator in
  the workspace was sqlx's `runtime-tokio-rustls` (it pulls `tls-rustls-ring`,
  which enables rustls's `ring` feature). Pinning sqlx to `runtime-tokio` +
  `tls-rustls-aws-lc-rs` removes that activation; `cargo tree -i ring --target
  all` now prints nothing and `buck2 cquery "deps(//...)" | grep ring-0.17`
  returns 0. The orphaned ring-0.17 third-party BUCK targets + the rustls/
  rustls-webpki `ring` features/deps were purged. A residual `name = "ring"`
  stanza remains in `Cargo.lock` ONLY as an unreachable phantom: `reqwest`'s
  optional, never-enabled `http3`/`quinn` -> `quinn-proto` chain pins ring's
  version in the lock graph (Cargo retains resolved optional-dep closures even
  when the feature is off; Cargo.lock stores versions, not feature activation).
  It is in no BUCK target and no build graph. The dev-dependency `ring` retained
  for JWT test minting (above) was already removed in earlier slices; no prod or
  dev `ring` remains active.

## Enforcement (G002, 2026-06-13)

The zero-ring-activation invariant above was previously only *described* here; it
is now **mechanically enforced** by a new single-concern cloud-ci gate (ADR-0132
no-grouping; founder doctrine "flag-only/manual = incomplete; construction >
reaction; automate everything automatable"):

- **Gate:** `ci/facade/crypto-backend-policy`
  (gate_id `cloud-ci-crypto-backend-purity`), wired into the `gate-affected-set`
  matrix of `.github/workflows/oya-ci-required.yml`.
- **What it asserts:** the forbidden crypto backend(s) — at minimum `ring`,
  policy-driven in `crypto-backend-purity-policy.json` — are never **ACTIVATED**
  in the workspace's feature-resolved dependency graph; `aws-lc-rs` is the
  mandated backend.
- **The signal is feature-resolved ACTIVATION, not the dependency SUPERSET.** The
  gate runs `cargo tree -i <crate> --target all` (the inverse, feature-resolved
  view that prunes an optional-dependency edge whose activating feature is off)
  and FAILS iff a forbidden backend has ≥1 activated dependent. It deliberately
  does **not** inspect `Cargo.lock` text nor `cargo metadata`'s
  `resolve.nodes[].dependencies`/`resolve.nodes[].deps[]` lists: those retain the
  unactivated optional-dep `ring` phantom (see the previous consequence —
  `reqwest`'s off `http3`/`quinn` → `quinn-proto` chain; `rustls-webpki`'s off
  `ring` feature) and would **false-RED** on a harmless stanza that is in no
  build graph and is never compiled. **Cargo.lock stores resolved versions,
  including optional deps, not feature activation** (restating the invariant the
  gate is built around). The gate thus distinguishes an **activated** ring (FAIL)
  from the lock-superset phantom (OK).
- **Proven, not always-pass:** RED/GREEN tests ship with the gate — a GREEN test
  asserts the live tree has zero activated ring (born-blocking green today), and
  a RED test asserts the gate FAILS on a fixture where a crate activates ring;
  the live gate binary additionally fails closed (exit 1) when pointed at a
  forbidden crate that is genuinely activated.
- **The Cargo.lock `ring` phantom remains** as an unactivated optional-dep entry.
  Its full removal requires removing `reqwest` entirely (direct `reqwest 0.13` in
  ~11 workspace crates + the transitive `reqwest`/`opentelemetry-otlp`
  reqwest-blocking-client edge) in favour of the hyper-rustls (aws-lc-rs) client
  already declared at the workspace seam (root `Cargo.toml` "Doctrine: hyper
  client, not reqwest"). That is a large multi-crate slice tracked as the
  **reqwest→hyper migration** friction (`FRIC-1781530000`); the lock phantom is
  harmless because it is never built, and this gate enforces the invariant that
  actually matters (zero ring activation) in the interim.

### Accounting (ADR-0555 born-accounting justification_ref)

This ADR is the justification anchor for the new gate's tracked surfaces (each
file path is cited explicitly so the accounting-registry producer maps it to
`justification_ref: ADR-0506`):

- `ci/facade/crypto-backend-policy/Cargo.toml`
- `ci/facade/crypto-backend-policy/BUCK`
- `ci/facade/crypto-backend-policy/crypto-backend-purity-policy.json`
- `ci/facade/crypto-backend-policy/src/lib.rs`
- `ci/facade/crypto-backend-policy/src/main.rs`
- `ci/facade/crypto-backend-policy/tests/crypto_backend_purity.rs`

The gate is OWNED via the inherited `ci/OWNERS`
(`cloud-ci-platform`, the same ownership seed as every sibling gate) and
REACHABLE via cargo workspace membership + its BUCK target (the
`reachable_from: ["cargo-members"]` class kernel-purity also uses).

## Related

- ADR-0482 — Bespoke Substrate Roadmap (Tier 1-4); oya-crypto added as Tier-4
  entry with bridge=aws-lc-rs
- [[bespoke-over-oss-doctrine]] — Phase-1 OSS bridge → Tier-N bespoke pattern
- [[hyperscaler-lens-architectural-filter]] — pre-check table above
- [[kubers-canonical-substrate]] — kubers Phase-B is the unlock gate for oya-crypto
