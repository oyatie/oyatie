---
doc_class: FAQ
microservice: drive
persona: drive-engineer + storage-platform-engineer + encryption-engineer
related_adrs: [ADR-DRIVE-001, ADR-DRIVE-0001, ADR-DRIVE-0002, ADR-DRIVE-0003, ADR-DRIVE-0004, ADR-DRIVE-0005, ADR-DRIVE-0006]
date: 2026-05-20
doc_status: published
---

# Drive Engineer FAQ — drive

## Why per-tenant CMK + per-file DEK envelope instead of a single tenant-wide data key?

Per ADR-DRIVE-001 § Alternatives Considered. A single tenant-wide data key would:

1. Allow one key compromise to decrypt every file version in the tenant.
2. Make hot-data rewrap indistinguishable from cold-data rewrap.
3. Provide weak per-file custody evidence.

The CMK / KEK / DEK envelope (per file version):

- **CMK** (Customer Master Key): tenant-scoped key authority, lives in OpenBao + HSM.
- **KEK** (Key Encryption Key): rotation epoch; wraps DEKs; 30-day default rotation.
- **DEK** (Data Encryption Key): per-file-version random key; encrypts the actual payload.

One DEK compromise affects exactly one file version. One KEK compromise affects the file versions wrapped under that epoch (limited by lazy rewrap). One CMK compromise is the worst case but requires HSM compromise — and HSM compromise + access to KEK ciphertext + envelope rows + object bytes (all separated by Cedar + OpenBao path scoping).

## What's the AAD (authenticated associated data) and why does it matter?

Per ADR-DRIVE-001 § Decision: "Include tenant id, file id, version id, object digest, retention class, and data class in authenticated associated data."

AAD binds the DEK + ciphertext to the file's context. If an attacker swaps an object's payload bytes (e.g., replaces a file's blob in SeaweedFS with another tenant's blob), the AEAD decryption FAILS because the AAD includes file_id + version_id + object_digest. This protects against:

- Object-store admin malicious rewrites.
- Cross-tenant blob mix-up (e.g., bug in storage layer).
- Replay attacks (same DEK against different file).

The AAD hash is stored in the envelope row + verified on every decrypt.

## How does cross-tenant sharing preserve originating custody?

Per ADR-DRIVE-001 § Decision: "For cross-tenant sharing, grant read authorization through Cedar and share-link capability; do not rewrap DEK to recipient tenant CMK unless an explicit transfer-of-ownership workflow completes."

For normal sharing:

1. Share-link is issued by the originating tenant.
2. The DEK stays wrapped under the originating tenant's KEK + CMK.
3. The recipient's read is gated by Cedar `drive::file::cross_tenant_read`.
4. Read flow: recipient → drive-api → Cedar evaluates → originating-tenant OpenBao unwraps KEK → DEK + payload decrypted → stream to recipient.
5. The recipient NEVER gets a wrapped DEK; they get plaintext stream only for the share scope.

For ownership transfer (per IP-journey-j127):

1. Explicit transfer ceremony via `drive::file::transfer_ownership`.
2. New file version created under recipient tenant CMK (new DEK + recipient's KEK + recipient's CMK).
3. Originating tenant's version remains for audit + retention (immutable; per ADR-DRIVE-0006 WORM if pack requires).

## What's the WORM/immutability story (ADR-DRIVE-0006)?

Per ADR-DRIVE-0006. WORM = Write Once Read Many. File versions in WORM mode:

- Payload bytes immutable after first write.
- Cannot be deleted until retention window expires.
- Can be rewrapped (envelope row updated with new KEK epoch) but payload + AAD stay constant.
- Legal hold extends retention indefinitely.

Use cases:

- SEC Rule 17a-4 (financial records: 7 y immutable).
- HIPAA mandatory minimum (6 y).
- SOX 404 (7 y).
- GDPR DPIA evidence retention (varies).

Cedar `drive::file::write` is forbidden when `resource.immutability_state == "worm-active"` + retention window not expired. Cryptoshred is delayed until retention + legal-hold both cleared.

## How does content-defined chunking work (ADR-DRIVE-0002)?

Per ADR-DRIVE-0002 + IP-006-upload. FastCDC algorithm:

1. Sliding window over file content.
2. Compute rolling hash; chunk boundary when hash matches bitmask.
3. Target chunk size 4 MiB; min 1 MiB; max 16 MiB.
4. Per-chunk content-addressable hash (BLAKE3-256).
5. Server stores unique chunks once (dedup).
6. Per-file manifest references chunk hashes in order.

Result: editing a 1 GB file by changing 1 line uploads ~ 4 MiB (one chunk) instead of 1 GB. Sync clients benefit hugely on document workflows.

Each chunk is independently encrypted with its own DEK (per ADR-DRIVE-001). Dedup across users is per-tenant scoped (cross-tenant dedup would violate isolation).

## How does the share-link capability format work?

Per ADR-DRIVE-0003. Share-link is an Ed25519-signed JWT:

```json
{
  "tenant_id": "acme-corp",
  "file_id": "f_acme_001",
  "version_id": "v_acme_001_5",  // optional; if absent, latest version
  "permissions": "viewer",
  "expires_at": "2026-08-20T00:00:00Z",
  "max_views": 10,
  "current_views": 0,             // server-tracked
  "watermark_policy": "email-tagged",
  "issuer_principal": "u-alice@acme-corp",
  "sig_algorithm": "ed25519"
}
```

The token is signed by the tenant's share-link signing key (per-tenant; OpenBao-rooted). The receiver presents the token + their email (for watermarking) + IP. The server verifies signature + checks expiration + checks max-views + applies watermark.

Revocation: tenant admin can revoke the share-link; the server adds it to a revocation list (Bloom-filtered for fast denial). Revoked tokens fail validation.

## What's the preview sandbox model (ADR-DRIVE-0005)?

Per ADR-DRIVE-0005 + IP-012-preview. Preview generation requires decrypted plaintext (LibreOffice + ImageMagick read raw bytes). Per-preview-job flow:

1. Cedar `drive::file::preview` ✓.
2. Spawn ephemeral Cloud Hypervisor sandbox pod (Kata Containers per ADR-0254 KS#13).
3. Sandbox pod has read-only ephemeral filesystem + 30s wall-clock budget + 2 GiB memory cap.
4. Decrypted file bytes streamed in via tmpfs.
5. LibreOffice + ImageMagick render preview.
6. Preview bytes streamed out + re-encrypted under the same envelope model.
7. Sandbox pod terminated; tmpfs zeroed.

The sandbox cannot reach the network. Memory is zeroed before pod termination. Per-tenant scheduling prevents cross-tenant resource sharing.

## How is DLP scanning compatible with E2EE?

Per ADR-DRIVE-001 Constraint DRIVE-C10: "DLP and virus scanning must operate before encryption, in tenant-controlled scanning enclaves, or on client-provided plaintext."

Three modes:

1. **Pre-encryption server-side** (demo_trial/paid default): client uploads plaintext → DLP + virus scan run on the server → if passes, server encrypts + stores. Risk: brief plaintext window on the server.
2. **Tenant-controlled enclave** (paid): client uploads plaintext to a tenant-deployed Cloud Hypervisor scanning pod (inside the tenant's compliance boundary) → scan in enclave → server only sees ciphertext.
3. **Client-side scan** (compliance_pack option): client runs DLP + virus scan locally before upload; server never sees plaintext.

Tenant chooses per pack. HIPAA + FedRAMP-High typically use mode 2 or 3. EU-GDPR Art 9 often uses mode 3.

## What's the cryptoshred ceremony at compliance_pack?

Per ADR-DRIVE-001 § Decision + IP-cryptoshred-001. Tenant offboarding flow:

1. Tenant requests offboarding via `tenancy::lifecycle::transition` to `offboarding`.
2. `compliance` µservice evaluates retention + legal-hold clearance.
3. After retention window + legal-hold cleared, `drive::cryptoshred::plan` is approved.
4. Scheduled destroy time (typically 30 d future) gives tenant rollback window.
5. At destroy time: CMK destroyed in HSM (HSM emits destruction attestation).
6. All KEKs unwrapped under that CMK become useless; all DEKs become useless; all object bytes become decryption-impossible.
7. Audit-chain emits `EVT-DRIVE-CMK-CRYPTOSHRED-COMPLETED` with HSM attestation + destruction timestamp.
8. Regulator-observable attestation (per compliance_pack requirement): regulator can attend ceremony in person or via cryptographic witness.

After cryptoshred, the tenant's data is unrecoverable. Audit metadata + audit-chain events persist for compliance.

## How does Bring-Your-Own-KMS work (paid)?

Per ADR-DRIVE-001 § Implementation Notes neutral consequence + IP-byok-001. Tenant can opt to host the CMK externally:

- **AWS KMS** (with tenant's AWS account).
- **GCP Cloud KMS**.
- **Azure Key Vault** (with Customer Key).
- **Tenant-managed HSM** (CloudHSM, Azure Dedicated HSM, GCP Cloud HSM).

Integration:

1. Tenant configures their external KMS endpoint + IAM role for oyatie.
2. oyatie's `drive` µservice calls external KMS for KEK wrap/unwrap.
3. CMK lives in tenant's KMS; oyatie never sees CMK material.
4. Tenant can revoke oyatie's IAM permission at any time → all encrypted files instantly inaccessible (effective cryptoshred).
5. Performance: external KMS adds ~ 20-50 ms per unwrap; mitigated by KEK caching per IP-008 (≤ 60 s lease).

## What's the difference between immutability and legal hold?

Per ADR-DRIVE-0006. Both prevent deletion, but:

- **Immutability** (WORM): tenant-configured retention class (e.g., "7 y from creation"); deterministic; auto-released after retention window.
- **Legal hold**: pack-driven; usually open-ended; cannot be auto-released; requires explicit release ceremony with legal counsel approval.

Immutability + legal hold compose: legal hold extends an immutable file indefinitely. The pack `compliance` µservice determines which packs require legal hold (e.g., SEC 17a-4 financial records).

## How does evidence-vault export work (paid)?

Per ADR-DRIVE-001 § Decision + IP-journey-j17. For regulator/auditor workflows:

1. Auditor requests export via `drive::evidence::export`.
2. Cedar evaluates with court-order evidence + tenant approval.
3. Export bundle contains:
   - File ciphertext + envelope rows.
   - Audit-chain trail (creation, modifications, accesses, share-links).
   - DKIM-signed evidence attestation.
   - Pack-residency + retention metadata.
4. Auditor receives ciphertext + custody proofs.
5. Tenant's legal-hold appliance decrypts within the tenant's compliance boundary.

The server does NOT decrypt for evidence-vault export at compliance_pack. At paid, the server CAN decrypt with court-order Cedar permission (pack-dependent).

## How are file previews encrypted?

Per ADR-DRIVE-001 Constraint DRIVE-C14: "Preview cache and thumbnail cache must not use weaker encryption than originals."

Preview encryption uses the same envelope model:

- Per-preview DEK (random).
- Wrapped under the same KEK epoch as the parent file's version.
- Same CMK + AAD pattern.

This means previews are subject to the same KEK rotation + cryptoshred + WORM rules as originals. Some paid tenants choose to encrypt previews under a separate "preview-class" CMK for explicit isolation.

## What's the difference between drive and the audit-chain µservice?

- `drive`: file storage + preview + share + WORM + DLP. Stores file content + metadata.
- `audit-chain`: cryptographic append-only event log. Stores audit-chain events for all µservices (including drive).

Every drive operation emits to audit-chain. Audit-chain stores the EVT records (Ed25519-signed); drive stores the file content (envelope-encrypted). They are decoupled but linked by `audit_event_id` references.
