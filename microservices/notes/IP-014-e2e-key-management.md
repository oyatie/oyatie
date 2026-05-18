---
doc_class: ImplementationPlan
impl_plan_id: IP-014-e2e-key-management
milestone: M02-foundation
phase: P01-notes-foundation
status: pending
owner: axis-notes + council-privacy
acceptance_lanes: [cargo-check, cargo-test, oya-governance-port-location, openmls-version-pin]
---

# IP-014: e2e-key-management (openmls 0.6) + recovery seed UX

## Intent

Land `oya-notes-e2e-key-management-{kernel,domain,usecase,api,adapter,adapter-mls,sdk,app}`. Implements:
- Server: public KeyPackage distribution + commit message routing + revocation list (TTL 1m).
- Client SDK: MLS RFC 9420 key derivation via openmls 0.6 (TS via Wasm; Swift via openmls Swift bridge; Kotlin via openmls JVM; Rust native).
- Recovery seed UX: 24-word BIP39-style; double-confirmation at onboarding.

## Server-Side Constraints

- NEVER stores private key material.
- NEVER decrypts ciphertext.
- Stores `key_package_bytes` (public) + `commit_bytes` (opaque).

## Client SDK Recovery UX

- Recovery seed presented at onboarding; user must explicitly confirm receipt ("I have stored the seed safely") + acknowledge tradeoff ("Loss = permanent destruction").
- Recovery flow on new device: prompt for seed → derive root key → register new device-bound KeyPackage.

## Acceptance Gates

```bash
cargo check -p oya-notes-e2e-key-management-kernel
cargo check -p oya-notes-e2e-key-management-adapter-mls
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
```

## Next IP

[`IP-015-hg-notes-conformance.md`](IP-015-hg-notes-conformance.md)
