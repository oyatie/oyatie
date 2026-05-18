---
doc_class: ImplementationPlan
impl_plan_id: IP-008-share-link-and-embed
milestone: M02-foundation
phase: P01-notes-foundation
status: pending
owner: axis-notes
acceptance_lanes: [cargo-check, cargo-test, oya-governance-port-location]
---

# IP-008: share-link + embed

## Intent

Land `oya-notes-share-link-*` (read-only share tokens; passphrase gate; TTL) + `oya-notes-embed-*` (drive-µservice attachment referencing).

## Share-Link Specs

- Token: 128-bit URL-safe random.
- Passphrase: optional; PBKDF2-SHA256 ≥ 600k iterations.
- TTL: configurable up to 1 year; default 7 days.
- Scope: ReadOnly.
- Personal-tier sharing reveals plaintext only when user pre-decrypts in SDK + re-encrypts under share-key + uploads ciphertext alongside share-link; default Personal sharing is metadata + title only.

## Embed Specs

- References drive µservice blob via `blob_ref`.
- MIME hint stored locally; full attachment fetched from drive.
- Cross-µservice via Workflow events (`DriveAttachmentRevoked` → mark embed broken).

## Acceptance Gates

```bash
cargo check -p oya-notes-share-link-kernel
cargo check -p oya-notes-embed-kernel
```

## ChangeSet metadata

```yaml
changeset_id: CS-NOTES-IP-008-share-link-and-embed
depends_on_changesets: [CS-NOTES-IP-003-note-store-kernel-domain]
parallel_safe_with_changesets: [CS-NOTES-IP-009-checklist-and-version-history, CS-NOTES-IP-010-search-and-graph-view]
enables: [CS-NOTES-IP-011-collab-edit-loro]
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Token 128-bit URL-safe random; cryptographic-strength RNG | `cargo nextest run -p oya-notes-share-link-domain -- token_entropy` |
| AC-02 | Passphrase KDF PBKDF2-SHA256 ≥ 600k iterations (OWASP ASVS v4 §2.4) | `cargo nextest run -p oya-notes-share-link-domain -- pbkdf2_iterations` |
| AC-03 | Personal-tier share-out path forbids server-side plaintext access | `cargo nextest run -p oya-notes-share-link-domain -- personal_tier_no_plaintext` |
| AC-04 | Embed binds `blob_ref` from drive µservice; `DriveAttachmentRevoked` marks embed broken | `cargo nextest run -p oya-notes-embed-domain -- attachment_revocation_cascade` |
| AC-05 | TTL refused beyond 1 year max | `cargo nextest run -p oya-notes-share-link-domain -- ttl_upper_bound` |

## Build Sequence

1. Kernel: `ShareLinkMinter`, `EmbedRegistry`, `PassphraseHasher` ports.
2. Domain: `ShareLink`, `Passphrase`, `EmbedReference`.
3. Usecase: `MintShareLink`, `VerifyShareLink`, `RevokeShareLink`, `BindEmbed`.
4. Cross-µservice event subscriber for `DriveAttachmentRevoked`.
5. `cargo nextest run -p oya-notes-share-link-* -p oya-notes-embed-*`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-notes FR | FR-08 (share-link), FR-09 (embed) |
| PRD-notes NFR | NFR security — share-link tokens; cross-tier context drift forbidden |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Timing oracle on passphrase compare | Constant-time compare via `subtle::ConstantTimeEq` |
| Embed reference dangles after drive object deleted | Cascade revocation via `DriveAttachmentRevoked` event |
| Share-link enumeration | Per-IP rate limit + audit-chain on invalid attempts |

## References

- OWASP ASVS v4 §2.4 (Credential storage).
- RFC 9106 (Argon2) — alternative KDF mentioned in PRD §Security.
- Standard Notes share-link reference (`standardnotes.com/help`).
- Obsidian Publish share semantics (`help.obsidian.md/Obsidian+Publish`).
- ADR-NOTES-0001 (E2E posture).

## Next IP

[`IP-009-checklist-and-version-history.md`](IP-009-checklist-and-version-history.md)
