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


## A. Problem
`IP-008: share-link + embed` is not a generic implementation packet; it closes the `008 share link and embed` gap for `notes` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: Note, PersonalNoteRef, ProfessionalNoteRef, tag-graph, backlink graph, Loro CRDT, MLS key package, share-link, E2E refusal.

## B. Approach
Capture/share paths keep notes personal-by-default: clipper tokens are per-installation, share-links are read-only/revocable, and Personal sharing requires client-side re-encryption. The implementation must keep the µservice boundary intact: contracts remain under `microservices/notes/contracts/openapi/notes.yaml` / `microservices/notes/contracts/proto/notes.proto`, policy decisions remain in `microservices/notes/policy/tenant-scope.cedar`, operational proof remains in `microservices/notes/slos/note-open-latency.openslo.yaml`, and the parity claim is checked against `microservices/notes/competitor-parity-matrix.md`.

## C. Deliverables
- `microservices/notes/PRD.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/ARCHITECTURE.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/contracts/openapi/notes.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/contracts/proto/notes.proto` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/contracts/asyncapi/notes-events.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/policy/tenant-scope.cedar` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/slos/note-open-latency.openslo.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/runbooks/sync-conflict-resolution.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/catalog/oya-notes-note-store-kernel.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/competitor-parity-matrix.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/catalog/oya-notes-web-clipper-bridge-kernel.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/catalog/oya-notes-share-link-kernel.yaml` — verify/update as the authoritative artifact for this IP.
- Named code targets declared by this IP and `manifest.json` must be created only when the implementation PR actually adds the crates/types; this scrub does not pretend source files exist.

## D. Implementation Steps
1. Read `microservices/notes/PRD.md` and `microservices/notes/ARCHITECTURE.md` to confirm the bounded context, tenant class, and first-ship milestone for `notes`.
2. Diff the declared contract in `microservices/notes/contracts/openapi/notes.yaml` and `microservices/notes/contracts/proto/notes.proto` against the IP title so every endpoint/message has a matching domain type or explicit backlog gap.
3. Check `microservices/notes/policy/tenant-scope.cedar` plus adjacent Cedar/policy files before adding any mutation, share, webhook, agent, AI, or cross-tenant path.
4. Wire observability to `microservices/notes/slos/note-open-latency.openslo.yaml` and the relevant dashboard/runbook; no acceptance claim counts without a metric or sealed evidence path.
5. Update the catalog/capability record such as `microservices/notes/catalog/oya-notes-note-store-kernel.yaml` so the service registry can discover the new boundary.
6. Run the IP-specific test/gate commands listed above; if a source crate is absent, record the absent crate as implementation debt rather than faking a green result.

## E. Acceptance
- Local artifact links resolve for `microservices/notes/PRD.md`, `microservices/notes/ARCHITECTURE.md`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/policy/tenant-scope.cedar`, `microservices/notes/slos/note-open-latency.openslo.yaml`, and `microservices/notes/competitor-parity-matrix.md`.
- The implementation exposes no cross-tenant, cross-pack, credential, E2E, or vendor-call path without the policy file cited in this IP.
- At least one targeted unit/contract/gate command verifies the named behavior, and any skipped command is documented with the missing artifact.
- The final PR includes evidence that counterpart parity is improved or explicitly marks the remaining gap.

## F. Evidence
- `microservices/notes/PRD.md`
- `microservices/notes/ARCHITECTURE.md`
- `microservices/notes/contracts/openapi/notes.yaml`
- `microservices/notes/contracts/proto/notes.proto`
- `microservices/notes/contracts/asyncapi/notes-events.yaml`
- `microservices/notes/policy/tenant-scope.cedar`
- `microservices/notes/slos/note-open-latency.openslo.yaml`
- `microservices/notes/runbooks/sync-conflict-resolution.md`
- `microservices/notes/catalog/oya-notes-note-store-kernel.yaml`
- `microservices/notes/competitor-parity-matrix.md`
- `microservices/notes/competitor-parity-matrix.md` — counterpart gap table used for the comparison below.

## G. Counterparts
| Counterpart pressure | Oyatie closure for this IP |
|---|---|
| Apple Notes, Google Keep, OneNote, Notion, Bear, Obsidian, Standard Notes, Evernote, Roam, Logseq, Joplin, Reflect, Tana, Mem, and Heptabase | Notion and OneNote define workspace/collab parity; Obsidian/Roam/Logseq define backlink and graph parity; Standard Notes and Apple Notes define privacy pressure; Evernote/Bear/Google Keep define capture/import expectations. This IP closes the relevant gap by binding `008 share link and embed` to concrete `notes` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
