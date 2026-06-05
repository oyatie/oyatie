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

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

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

## ChangeSet metadata

```yaml
changeset_id: CS-DRIVE-IP-009-share-link
depends_on_changesets: [CS-DRIVE-IP-003-file-store-kernel-domain, CS-DRIVE-IP-010-permissions]
parallel_safe_with_changesets: [CS-DRIVE-IP-011-search-index, CS-DRIVE-IP-012-preview]
enables: [CS-DRIVE-IP-007-download]
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Ed25519 known-answer-tests (RFC 8032 §7) pass byte-for-byte | `cargo nextest run -p oya-drive-share-link-domain -- ed25519_kat` |
| AC-02 | Argon2id KDF parameters match RFC 9106 §4 recommended profile (m=64MiB, t=3, p=1) | `cargo nextest run -p oya-drive-share-link-domain -- argon2id_kat` |
| AC-03 | Strict-TTL: tokens past `exp` rejected without DB read; clock skew tolerance ≤ 30s | `cargo nextest run -p oya-drive-share-link-domain -- strict_ttl` |
| AC-04 | View-cap decrement atomic under concurrent access | `cargo nextest run -p oya-drive-share-link-adapter-postgres -- view_cap_race` |
| AC-05 | REST handler constant-time response on token-invalid vs token-expired | `cargo nextest run -p oya-drive-share-link-rest -- constant_time_response` |
| AC-06 | Enumeration rate-limit triggers at > 10 rps invalid-token per IP | `cargo nextest run -p oya-drive-share-link-rest -- enumeration_rate_limit` |

## Build Sequence

1. Kernel: `ShareLinkMinter`, `ShareLinkVerifier`, `RevocationCascade` ports.
2. Domain: `ShareLink`, `Passphrase`, `ViewBudget`, `Expiry`.
3. Usecase: `MintShareLink`, `VerifyShareLink`, `RevokeShareLink`, `CascadeRevoke`.
4. Postgres adapter; REST handler with constant-time responses.
5. `cargo nextest run -p oya-drive-share-link-*`.
6. `buck2 build //:quality-lane-registry-authority-check # lane=share-link-signing-conformance --microservice drive`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-drive FR | FR-04 (signed link) |
| PRD-drive NFR | NFR security — Ed25519 + Argon2id; NFR perf — link gen p95 ≤ 50ms |
| PRD-drive AC | AC-05 |
| ADR | ADR-DRIVE-0003 |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Timing oracle on passphrase compare | `subtle::ConstantTimeEq` (or equivalent) for all token + passphrase compares |
| Stale Ed25519 signing key remains valid after rotation | Keys carry `kid`; verifier refuses unknown / retired `kid` |
| View-cap race admits N+1 viewers | Postgres advisory lock + atomic UPDATE … RETURNING |

## References

- ADR-DRIVE-0003.
- PRD-drive §FR-04; AC-05.
- RFC 9106 (Argon2 password-hashing).
- RFC 8032 (Edwards-Curve Digital Signature Algorithm — Ed25519).
- OWASP ASVS v4 §2.4 (Credential storage).
- Google Drive sharing-link semantics (Google Workspace Help — "Share files from Google Drive").
