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

## Next IP

[`IP-009-checklist-and-version-history.md`](IP-009-checklist-and-version-history.md)
