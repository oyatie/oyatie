---
id: ADR-DRIVE-001
title: Per-Tenant CMK with Rolling KEK and Per-File DEK Envelope Encryption
status: Accepted
date: 2026-05-20
microservice: drive
related_oyatie_adrs:
  - docs/decisions/ADR-0002-tenant-and-identity-kernel.md
  - docs/decisions/ADR-0003-audit-chain-and-evidence-emission.md
  - docs/decisions/ADR-0007-cedar-authorization-policy-and-persona-tier.md
  - docs/decisions/ADR-0008-data-use-boundary.md
  - docs/decisions/ADR-0043-secrets-management-openbao-and-hsm-per-cell.md
decision_owner: axis-drive
---

# ADR-DRIVE-001: Per-Tenant CMK with Rolling KEK and Per-File DEK Envelope Encryption

## Context

- Drive stores files, previews, folder hierarchy, permissions, sync journals, immutability tiers, DLP state, and search metadata.
- The service architecture identifies `file-store`, `folder-hierarchy`, `permissions`, `preview`, `dlp-virus-scan`, and `immutability-tier` bounded contexts.
- Existing decisions include object storage, chunking, share-link security, and encryption, but this ADR binds the tenant-CMK / KEK / per-file-DEK custody model.
- Existing runbooks include `object-storage-degraded.md`, `upload-multipart-stuck.md`, `share-link-takeover-incident.md`, and `immutability-tier-violation.md`.
- Named precedent: AWS S3 SSE-KMS uses per-object data keys under KMS-managed customer keys.
- Named precedent: Google Cloud Storage CMEK separates object encryption from key management authority.
- Named precedent: Box KeySafe and Microsoft 365 Customer Key show enterprise-driven key custody expectations for document platforms.
- Constraint DRIVE-C1: tenant ownership, principal identity, and home cell must be derived from ADR-0002.
- Constraint DRIVE-C2: key generation, wrapping, unwrap, rotation, and denial must emit audit evidence per ADR-0003.
- Constraint DRIVE-C3: file read, preview, download, share, and rewrap actions must pass Cedar per ADR-0007.
- Constraint DRIVE-C4: data classes and purpose permissions must follow ADR-0008, especially PHI, PCI, PII, and sensitive PIPA classes.
- Constraint DRIVE-C5: CMK and KEK custody must use OpenBao / HSM per ADR-0043.
- Constraint DRIVE-C6: object storage compromise cannot reveal file plaintext.
- Constraint DRIVE-C7: OpenBao compromise cannot decrypt content without object-store ciphertext and envelope rows.
- Constraint DRIVE-C8: per-file sharing cannot copy or fork the tenant CMK into recipient tenants.
- Constraint DRIVE-C9: cross-tenant deal rooms and evidence vaults need bounded access to ciphertext without tenant-wide key sharing.
- Constraint DRIVE-C10: DLP and virus scanning must operate before encryption, in tenant-controlled scanning enclaves, or on client-provided plaintext.
- Constraint DRIVE-C11: WORM and legal-hold files must remain decryptable for authorized tenants after KEK rotation.
- Constraint DRIVE-C12: tenant offboarding must cryptoshred active content without deleting audit-chain evidence.
- Constraint DRIVE-C13: rewrap must be online, resumable, idempotent, and observable.
- Constraint DRIVE-C14: preview cache and thumbnail cache must not use weaker encryption than originals.
- Constraint DRIVE-C15: sync clients must be able to resume multipart uploads without exposing DEKs.
- The architecture must support tenant-managed keys later without changing file metadata shape.
- The architecture must support object-store migrations without changing DEK ciphertext.
- The architecture must support sovereign packs that require CMK and object storage in the same cell.
- The architecture must keep search index plaintext eligibility explicit and policy-bound.
- The architecture must keep encryption state visible enough for support without exposing secrets.

## Decision

- Use envelope encryption for all Drive file objects, preview objects, thumbnails, and sync-journal object payloads.
- Generate one random per-file DEK per object version, not per logical file.
- Wrap each DEK with a rolling tenant KEK.
- Wrap each tenant KEK with a tenant CMK held in OpenBao / HSM-backed custody.
- Represent CMK as tenant-scoped key authority, KEK as rotation epoch, and DEK as object-version encryption key.
- Store encrypted DEK material in Drive metadata, never in object body metadata alone.
- Store `cmk_ref`, `kek_epoch`, `dek_ciphertext`, `dek_wrap_alg`, and `encryption_context_hash` on each file version row.
- Use AES-256-GCM for object payload encryption by default.
- Use XChaCha20-Poly1305 only for clients or packs that cannot use AES-GCM safely.
- Include tenant id, file id, version id, object digest, retention class, and data class in authenticated associated data.
- Use object-version immutability for legal hold; rewrap updates envelope rows but never mutates object payload bytes.
- Rotate KEK every 30 days by default.
- Rotate KEK immediately on tenant key-compromise event.
- Rotate CMK only through tenant lifecycle ceremony because CMK rotation affects all KEK wraps.
- Rewrap DEKs lazily for cold data and eagerly for hot or high-risk data.
- Rewrap priority is determined by `data_class`, `retention_class`, `last_accessed_at`, and pack policy.
- Cryptoshred tenant content by disabling and then destroying CMK material after retention and legal-hold checks.
- Keep key tombstone metadata after cryptoshred for audit and billing reconciliation.
- For cross-tenant sharing, grant read authorization through Cedar and share-link capability; do not rewrap DEK to recipient tenant CMK unless an explicit transfer-of-ownership workflow completes.
- For ownership transfer, create a new file version under recipient tenant CMK after policy approval and audit evidence.
- For evidence vaults, bind file versions to the originating tenant CMK and expose ciphertext plus custody proofs to authorized regulator workflows.
- Use OpenBao transit for KEK wrap and unwrap where non-exportability is required.
- Use sidecar materialization with <=60 second TTL only for per-object DEK decrypt inside Drive workers.
- Reject plaintext object uploads unless the upload session can produce an encryption envelope before final commit.
- Make encryption posture part of the file version state machine.

## Alternatives Considered

### One Tenant-Wide Data Key

- Pros: minimal metadata.
- Pros: easy upload and download paths.
- Pros: fast cryptoshred.
- Cons: one key compromise decrypts every file version.
- Cons: cannot rewrap hot data separately from cold data.
- Cons: poor evidence for per-file custody.
- Rejected because the blast radius is too broad for Drive.

### Object Store Managed Encryption Only

- Pros: offloads encryption to object storage.
- Pros: simple operations.
- Pros: good baseline for commodity storage.
- Cons: object-store admin compromise can expose plaintext or key grants.
- Cons: tenant custody evidence depends on provider-specific controls.
- Cons: cross-provider portability is weak.
- Rejected because Drive must own tenant CMK semantics independently of object storage provider.

### Client-Side Encryption Only

- Pros: strongest server-side confidentiality.
- Pros: OpenBao compromise cannot reveal content.
- Pros: aligns with personal vault use cases.
- Cons: preview, DLP, virus scanning, search, and legal hold become much harder.
- Cons: enterprise recovery becomes tenant-specific client tooling.
- Cons: regulated workflows require service-side evidence of encryption state.
- Rejected as the default; it can be an additional vault mode layered over this server-side envelope model.

### Per-Share Rewrap to Recipient Tenant

- Pros: recipient tenant gets independent custody.
- Pros: revocation can destroy recipient wraps.
- Pros: aligns with some data-room products.
- Cons: casual sharing creates many envelope rows.
- Cons: revocation semantics are misleading after recipient download.
- Cons: cross-tenant rewrap can violate originating retention and residency rules.
- Rejected for normal sharing; retained only for explicit transfer-of-ownership workflows.

## Consequences

- Positive: object store compromise alone cannot decrypt file contents.
- Positive: tenant key compromise can be scoped to KEK epochs and affected file versions.
- Positive: legal hold can keep payload immutable while rewrap progresses.
- Positive: tenant offboarding can cryptoshred without deleting audit evidence.
- Positive: cross-tenant sharing preserves originating custody unless ownership transfer is explicit.
- Positive: preview and thumbnail encryption follows the same control model.
- Positive: key rotation can be prioritized by risk instead of full-scan urgency.
- Positive: support can diagnose encryption state from metadata without key access.
- Negative: every download depends on envelope metadata integrity and KEK unwrap availability.
- Negative: rewrap backlog can accumulate after emergency rotation.
- Negative: corrupted envelope rows can make a file version unrecoverable even if object bytes are intact.
- Negative: DLP and preview services must coordinate with encryption state and scanning enclaves.
- Negative: object version count increases key and metadata storage cost.
- Neutral: client-side vault mode remains possible for users who accept reduced service-side features.
- Neutral: per-file DEK rotation means logical files may have multiple encryption epochs.
- Neutral: tenant-managed external KMS can map to CMK provider later without changing `FileVersionEnvelope`.
- Neutral: immutable files can still be rewrapped because the ciphertext payload is unchanged.
- Neutral: search index eligibility remains separate policy, not implied by encryption status.

## Implementation Notes

- Data shape `TenantCmk`: `{tenant_id, cmk_id, provider, openbao_ref, state, home_cell, created_at, disabled_at}`.
- Data shape `TenantKek`: `{tenant_id, kek_epoch, cmk_id, wrap_ref, state, activates_at, retires_at, compromised_at}`.
- Data shape `FileVersionEnvelope`: `{tenant_id, file_id, version_id, object_ref, dek_ciphertext, kek_epoch, cmk_id, aad_hash, algorithm, created_at}`.
- Data shape `RewrapJob`: `{tenant_id, job_id, from_kek_epoch, to_kek_epoch, selector, state, cursor, last_error, audit_event_id}`.
- Data shape `CryptoshredPlan`: `{tenant_id, cmk_id, legal_hold_clearance_ref, retention_clearance_ref, scheduled_destroy_at, approved_by}`.
- OpenBao path: `secret/<tenant_id>/drive/cmk/<cmk_id>`.
- OpenBao path: `transit/keys/<tenant_id>-drive-kek-<kek_epoch>`.
- REST endpoint `POST /v1/drive/files/{file_id}/versions` requires encryption envelope before commit.
- REST endpoint `GET /v1/drive/files/{file_id}/versions/{version_id}/envelope` returns metadata only, never DEK plaintext.
- REST endpoint `POST /v1/drive/keys/kek/rotate` starts KEK rotation.
- REST endpoint `POST /v1/drive/keys/rewrap-jobs` starts risk-scoped DEK rewrap.
- REST endpoint `POST /v1/drive/tenants/{tenant_id}/cryptoshred-plan` schedules CMK destruction.
- REST endpoint `POST /v1/drive/files/{file_id}/transfer-ownership` creates a recipient-custody version.
- AsyncAPI channel `drive.encryption.kek.rotated.v1` publishes KEK activation.
- AsyncAPI channel `drive.encryption.rewrap.started.v1` publishes rewrap start.
- AsyncAPI channel `drive.encryption.rewrap.completed.v1` publishes completion and count.
- AsyncAPI channel `drive.encryption.cryptoshred.scheduled.v1` publishes pending CMK destruction.
- Cedar permit `drive::file::decrypt` requires file permission, tenant scope, and pack residency match.
- Cedar forbid `drive::file::decrypt` when `resource.envelope_state == "cryptoshredded"`.
- Cedar permit `drive::key::rotate_kek` requires tenant admin step-up and no active incident freeze.
- Cedar permit `drive::file::transfer_ownership` requires owner tenant, recipient tenant, and compliance approval.
- Audit event `EVT-DRIVE-DEK-WRAPPED` includes envelope hash and KEK epoch.
- Audit event `EVT-DRIVE-KEK-ROTATED` includes old and new epochs.
- Audit event `EVT-DRIVE-REWRAP-JOB-COMPLETED` includes selected file count and failure count.
- Audit event `EVT-DRIVE-CMK-CRYPTOSHRED-SCHEDULED` includes legal-hold clearance reference.
- Metric `drive_envelope_unwrap_latency_ms` tracks KEK unwrap latency by cell.
- Metric `drive_rewrap_backlog_versions` tracks remaining file versions by risk bucket.
- Metric `drive_cryptoshred_blocked_total` counts legal-hold or retention blockers.
- Metric `drive_envelope_corruption_total` counts metadata/object digest mismatches.
- Capacity math: for 1 billion file versions and 30-day KEK rotation, eager full rewrap is impossible at 386 versions/s without 30-day lag; risk-based lazy rewrap is required.
- Capacity math: hot-set rewrap of 50 million versions in 24 hours needs 579 versions/s; allocate 2,000 versions/s per region with backpressure.
- Rollback path: failed KEK activation restores previous active KEK pointer and pauses new uploads.
- Rollback path: rewrap jobs are idempotent because object payload bytes and version ids are stable.
- Multi-region path: object replicas can move only where CMK unwrap is legally available.
- Sovereign path: KR, EU, FedRAMP-High, and CN-PIPL packs require CMK, KEK, envelope rows, and object bytes in approved cell sets.
- Versioning: `FileVersionEnvelope` schema v1 supports additive fields only.
- Deprecation: encryption algorithms require 365-day read support after write deprecation unless a critical cryptographic break forces shorter migration.

## Verification

- Unit test `file_version_requires_envelope_before_commit` rejects plaintext object finalization.
- Unit test `envelope_aad_includes_tenant_file_version_and_digest` proves authenticated data coverage.
- Unit test `cross_tenant_share_does_not_rewrap_dek` verifies normal sharing preserves originating CMK.
- Unit test `ownership_transfer_creates_new_recipient_envelope` verifies explicit transfer semantics.
- Unit test `cryptoshred_blocked_by_legal_hold` prevents destructive CMK action.
- Property test `rewrap_job_is_idempotent_across_retries` generates crash and resume points.
- Property test `kek_epoch_selection_matches_file_version_time` covers old and new uploads during rotation.
- Fuzz test `envelope_parser_rejects_malformed_ciphertext` covers metadata corruption.
- Integration test `download_requires_cedar_and_openbao_unwrap` proves policy and custody gates compose.
- Integration test `preview_cache_uses_own_envelope` verifies thumbnails are not weaker than originals.
- Integration test `dlp_scanner_cannot_decrypt_without_scanning_grant` verifies least privilege.
- Integration test `evidence_vault_export_is_ciphertext_plus_custody` verifies regulator workflow shape.
- Load test `hot_set_rewrap_50m_versions_per_day` validates 2,000 versions/s target.
- Load test `download_unwrap_p95_under_15ms` validates key path latency.
- Chaos test `openbao_partition_denies_decrypt_no_plaintext_fallback` proves fail-closed behavior.
- Chaos test `object_store_corruption_detected_by_aad_hash` proves digest enforcement.
- Metric SLO: `drive_envelope_unwrap_latency_ms` p95 below 15 ms.
- Metric SLO: `drive_rewrap_backlog_versions` drains high-risk bucket within 24 hours.
- Metric SLO: `drive_envelope_corruption_total` pages immediately above zero for active files.
- Audit check: every KEK rotation emits `EVT-DRIVE-KEK-ROTATED`.
- Audit check: every cryptoshred plan has legal-hold and retention clearance references.
- Static check: no API returns `dek_plaintext`, `kek_plaintext`, or `cmk_material`.
- Static check: OpenBao refs include tenant and drive path segments.
- Contract check: OpenAPI marks envelope APIs as metadata-only.

