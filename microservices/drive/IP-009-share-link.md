---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-drive-foundation
impl_plan_id: IP-009-share-link
status: pending
execution_unit: ChangeSet
owner: axis-drive + ops-security
acceptance_lanes: [cargo-build, cargo-nextest, oya-governance-share-link-signing-conformance]
---

# IP-009: share-link BC — Ed25519 + Argon2id + strict-TTL + view-cap + revocation cascade

## Intent

Stand up `oya-drive-share-link-*` BC per ADR-DRIVE-0003. Ed25519 + HKDF signing; Argon2id password KDF; strict-TTL; atomic view-cap; revocation cascade.

## Crates

`oya-drive-share-link-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,app}` (10 crates).

## Acceptance Gates

```bash
cargo nextest run -p oya-drive-share-link-domain -- ed25519_kat
cargo nextest run -p oya-drive-share-link-domain -- argon2id_kat
cargo nextest run -p oya-drive-share-link-domain -- strict_ttl
cargo nextest run -p oya-drive-share-link-adapter-postgres -- view_cap_race
cargo nextest run -p oya-drive-share-link-rest -- constant_time_response
cargo nextest run -p oya-drive-share-link-rest -- enumeration_rate_limit
```

## References

- ADR-DRIVE-0003.
- PRD-drive §FR-04; AC-05.
- RFC 9106 Argon2; RFC 8032 EdDSA.
