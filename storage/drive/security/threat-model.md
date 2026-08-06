---
doc_class: ThreatModel
microservice: drive
version: 1.0.0
status: Proposed
date: 2026-05-20
owner: axis-drive + council-security
related_oyatie_adrs:
  - ADR-0003
  - ADR-0009
  - ADR-0145
  - ADR-0243
  - ADR-0244
  - ADR-0263
  - ADR-0297
  - ADR-0313
  - ADR-0319
---

# Drive Security Threat Model

This document covers the drive substrate security posture for files, folders,
versions, share links, per-tenant CMK custody, preview/OCR pipelines, DLP,
malware scanning, search indexing, sync, and immutable retention. Drive stores
large volumes of tenant-controlled content, so its threat model treats every
file byte, derived preview, extracted text, thumbnail, and search token as
sensitive until policy and classification prove otherwise.

## Asset Inventory

### Named Data Classes

| Asset ID | Named data class | Description | Primary store | Security objective |
|---|---|---|---|---|
| DRIVE-A01 | FileObjectBytes | Original uploaded bytes for professional and personal files. | Object store | Prevent exfiltration, tampering, and malware persistence. |
| DRIVE-A02 | FileMetadataRecord | File name, MIME type, size, parent folder, owner, retention labels. | Postgres metadata | Prevent enumeration and cross-tenant disclosure. |
| DRIVE-A03 | FileVersionChain | Ordered immutable or mutable file version history and content hashes. | Metadata DB and object store | Prevent file-versioning attacks. |
| DRIVE-A04 | ShareLinkRecord | Signed link, recipient binding, TTL, view/download cap, password KDF metadata. | Share-link store | Prevent public link takeover and overexposure. |
| DRIVE-A05 | TenantCmkEnvelope | Per-tenant CMK/KEK/DEK envelope and rotation state. | OpenBao/KMS | Protect per-tenant CMK key custody. |
| DRIVE-A06 | UploadSessionState | Multipart/tus chunks, idempotency key, staging object, checksum. | Valkey and staging bucket | Prevent chunk injection and staging leakage. |
| DRIVE-A07 | PermissionAcl | User, group, role, inherited folder, external recipient, and link permissions. | Permissions store | Prevent unauthorized read/write. |
| DRIVE-A08 | SearchIndexToken | Extracted text tokens, filename tokens, OCR tokens, index partition state. | Search index | Prevent PII leakage and index poisoning. |
| DRIVE-A09 | PreviewArtifact | Thumbnail, PDF first page, Office render, media preview, OCR text sidecar. | Preview cache bucket | Prevent derived-content leakage. |
| DRIVE-A10 | MalwareScanVerdict | ClamAV/OPSWAT verdict, quarantine state, signature version, release decision. | Scan store and audit-chain | Prevent malware upload and unsafe release. |
| DRIVE-A11 | DlpFindingRecord | PII/PHI/secret detection result, rule id, reviewer decision. | DLP store and audit-chain | Prevent OCR-based PII leak and policy bypass. |
| DRIVE-A12 | ImmutabilityRetentionRecord | WORM tier, legal hold, retention floor, retention override attempts. | Retention store and audit-chain | Prevent evidence deletion. |
| DRIVE-A13 | SyncDeltaRecord | Client delta, conflict state, hash basis, device/session. | Sync store | Prevent sync poisoning and stale overwrite. |
| DRIVE-A14 | AuditEmissionEnvelope | ADR-0263 envelope with tenant_id, trace_id, span_id, audit_id, schema_version, source_microservice. | audit-chain | Preserve detection and non-repudiation. |

### Named External Interfaces

| Interface ID | Interface | Entry point | Principal | Notes |
|---|---|---|---|---|
| DRIVE-I01 | File Upload API | `../contracts/openapi/drive.yaml` | Authenticated user or SDK | Supports multipart/tus and staged object writes. |
| DRIVE-I02 | File Download API | `../contracts/openapi/drive.yaml` | Authenticated user, SDK, or signed link viewer | Range reads and signed URL issue. |
| DRIVE-I03 | Share Link API | `../IP-009-share-link.md` | Owner or admin | Creates, revokes, and accesses share links. |
| DRIVE-I04 | Permission API | `../IP-010-permissions.md` | Owner or admin | Mutates ACLs and inherited folder permissions. |
| DRIVE-I05 | Sync API | `../IP-008-sync.md` | Authenticated sync client | Applies client deltas and resolves conflicts. |
| DRIVE-I06 | Preview/OCR Pipeline | `../IP-012-preview.md` | Worker | Produces previews and text extraction. |
| DRIVE-I07 | DLP/Malware Pipeline | `../IP-013-dlp-virus-scan.md` | Worker | Scans uploads and quarantine releases. |
| DRIVE-I08 | Immutability Tier | `../IP-014-immutability-tier.md` | Retention worker | Enforces WORM and legal hold. |
| DRIVE-I09 | Object Store Adapter | `../IP-003-file-store-adapters.md` | Drive service | S3/Garage/SeaweedFS object storage. |
| DRIVE-I10 | Audit Event Bridge | `../contracts/asyncapi/drive-events.yaml` | Drive service | Emits sealed file, share, scan, and retention events. |

### Named Dependencies

| Dependency ID | Dependency | Use | Failure impact | Guardrail |
|---|---|---|---|---|
| DRIVE-D01 | Object storage | File bytes, preview cache, staging chunks | File loss or exfiltration | `../runbooks/object-storage-degraded.md`. |
| DRIVE-D02 | Postgres metadata | Folder, ACL, version, retention metadata | Cross-tenant metadata leak | Tenant scope and RLS. |
| DRIVE-D03 | OpenBao/KMS | Tenant CMK/KEK/DEK envelope | Mass data exposure | `../decisions/ADR-DRIVE-001-tenant-cmk-kek-dek-envelope-encryption.md`. |
| DRIVE-D04 | Cedar policy-engine | ACL, tenant, link, DLP, context gates | Broken access control | `../policy/tenant-scope.cedar`. |
| DRIVE-D05 | ClamAV/OPSWAT | Malware scanning | Unsafe file release | `../runbooks/virus-scan-rollback.md`. |
| DRIVE-D06 | Tika/OCR/preview tools | Text extraction and preview | PII leak or sandbox escape | `../decisions/ADR-DRIVE-0005-preview-pipeline-sandboxing.md`. |
| DRIVE-D07 | Search index | Content and filename search | Index leak or poisoning | `../IP-011-search-index.md`. |
| DRIVE-D08 | Valkey/cache | Upload and sync session state | Stale chunks or replay | `../runbooks/upload-multipart-stuck.md`. |
| DRIVE-D09 | audit-chain | Sealed evidence | Repudiation and incident gaps | ADR-0003 and ADR-0263. |
| DRIVE-D10 | identity | User/session and external recipient binding | Share-link or ACL impersonation | Identity service policies. |

## Trust Boundaries

| Boundary ID | Named boundary | Crosses from | Crosses to | Primary concern |
|---|---|---|---|---|
| DRIVE-B01 | Public file API boundary | Browser, SDK, sync client | Drive REST/gRPC ingress | Hostile metadata, upload flood, auth bypass. |
| DRIVE-B02 | Share-link public boundary | Anonymous or recipient-bound viewer | Share-link resolver | File exfiltration through signed link. |
| DRIVE-B03 | Tenant boundary | Tenant A files/folders | Tenant B files/folders | Cross-tenant ACL or storage prefix leak. |
| DRIVE-B04 | Personal/work boundary | Personal drive context | Professional drive context | Employer access to personal files or reverse leakage. |
| DRIVE-B05 | Object-store boundary | Drive worker | Object storage | Prefix escape, signed URL overgrant, object tamper. |
| DRIVE-B06 | CMK custody boundary | Drive service | OpenBao/KMS | Per-tenant key theft or wrong key use. |
| DRIVE-B07 | Upload staging boundary | External upload chunks | Staging bucket and Valkey | Chunk injection, replay, incomplete object exposure. |
| DRIVE-B08 | Version-chain boundary | File mutation request | Version graph and hashes | Rollback, fork, or overwrite attacks. |
| DRIVE-B09 | DLP/malware boundary | Newly uploaded file | Scanner and quarantine | Malware upload and unsafe release. |
| DRIVE-B10 | Preview/OCR boundary | File bytes | Sandbox renderer and OCR text sidecar | Embedded image PII leak or sandbox escape. |
| DRIVE-B11 | Search boundary | Extracted tokens | Search index | Index poisoning or plaintext leakage. |
| DRIVE-B12 | Sync boundary | Client deltas | Server conflict resolver | Stale overwrite and malicious delta. |
| DRIVE-B13 | Retention boundary | File lifecycle | Immutability/legal hold engine | Evidence deletion or retention bypass. |
| DRIVE-B14 | Audit boundary | Drive state change | audit-chain emission bridge | Missing audit_id or wrong tenant_id. |

## STRIDE Analysis

### Spoofing

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| DRIVE-S01 | ShareLinkRecord | DRIVE-B02 | Attacker guesses, steals, or replays shared link. | File exfiltration via shared link. |
| DRIVE-S02 | PermissionAcl | DRIVE-B03 | Caller spoofs tenant or group membership. | Cross-tenant file access. |
| DRIVE-S03 | TenantCmkEnvelope | DRIVE-B06 | Workload presents wrong tenant identity to KMS. | Decrypts with wrong CMK or denies legitimate tenant. |
| DRIVE-S04 | UploadSessionState | DRIVE-B07 | Attacker claims another user's upload session. | Chunk injection or object takeover. |
| DRIVE-S05 | SyncDeltaRecord | DRIVE-B12 | Malicious client spoofs device or sync cursor. | Unauthorized overwrite or data loss. |
| DRIVE-S06 | MalwareScanVerdict | DRIVE-B09 | Scanner result is spoofed as clean. | Malware release. |
| DRIVE-S07 | AuditEmissionEnvelope | DRIVE-B14 | Drive event is emitted under false source_microservice or tenant_id. | Forensic confusion. |

### Tampering

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| DRIVE-T01 | FileObjectBytes | DRIVE-B05 | Object bytes are altered after upload. | Integrity loss and malware insertion. |
| DRIVE-T02 | FileVersionChain | DRIVE-B08 | Version-chain attack rolls back or forks history. | Evidence tamper or lost update. |
| DRIVE-T03 | ShareLinkRecord | DRIVE-B02 | TTL, recipient binding, or view cap is extended. | Overbroad file exposure. |
| DRIVE-T04 | TenantCmkEnvelope | DRIVE-B06 | CMK envelope or key version is altered. | Data loss or unauthorized decrypt. |
| DRIVE-T05 | SearchIndexToken | DRIVE-B11 | Index poisoning hides sensitive file or injects false result. | Search and eDiscovery integrity loss. |
| DRIVE-T06 | DlpFindingRecord | DRIVE-B09 | DLP verdict or OCR extraction result is edited. | PII/PHI leak. |
| DRIVE-T07 | ImmutabilityRetentionRecord | DRIVE-B13 | Retention floor or WORM state is weakened. | Legal hold failure. |
| DRIVE-T08 | SyncDeltaRecord | DRIVE-B12 | Delta replay overwrites newer file version. | Data loss. |

### Repudiation

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| DRIVE-R01 | ShareLinkRecord | DRIVE-B02 | Actor denies creating or accessing public link. | Incident attribution gap. |
| DRIVE-R02 | PermissionAcl | DRIVE-B03 | Admin denies granting external access. | Authorization audit gap. |
| DRIVE-R03 | FileVersionChain | DRIVE-B08 | User denies uploading malicious or altered version. | Evidence ambiguity. |
| DRIVE-R04 | MalwareScanVerdict | DRIVE-B09 | Reviewer denies quarantine release. | Malware incident ambiguity. |
| DRIVE-R05 | TenantCmkEnvelope | DRIVE-B06 | Operator denies key rotation or CMK access. | Custody gap. |
| DRIVE-R06 | ImmutabilityRetentionRecord | DRIVE-B13 | Actor denies retention override. | Compliance gap. |

### Information Disclosure

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| DRIVE-I01 | FileObjectBytes | DRIVE-B02 | Shared link exposes file beyond intended recipient or TTL. | File exfiltration. |
| DRIVE-I02 | TenantCmkEnvelope | DRIVE-B06 | CMK, DEK, or key handle leaks. | Tenant-wide file disclosure. |
| DRIVE-I03 | PreviewArtifact | DRIVE-B10 | Preview or OCR sidecar exposes content to wrong user. | Derived-content disclosure. |
| DRIVE-I04 | SearchIndexToken | DRIVE-B11 | OCR text from embedded image leaks PII through search. | OCR-based PII leak. |
| DRIVE-I05 | FileMetadataRecord | DRIVE-B03 | Cross-tenant filename or folder metadata enumeration. | Privacy and competitive intelligence leak. |
| DRIVE-I06 | AuditEmissionEnvelope | DRIVE-B14 | ADR-0263 logs contain file names, object keys, or raw OCR text. | Observability privacy breach. |

### Denial of Service

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| DRIVE-DOS01 | UploadSessionState | DRIVE-B07 | Multipart upload flood consumes staging and cache. | Upload outage. |
| DRIVE-DOS02 | MalwareScanVerdict | DRIVE-B09 | Archive bomb or large sample saturates scanner. | Delivery and preview delay. |
| DRIVE-DOS03 | PreviewArtifact | DRIVE-B10 | Malformed file triggers expensive preview/OCR loop. | Worker exhaustion. |
| DRIVE-DOS04 | SearchIndexToken | DRIVE-B11 | Expensive search or reindex flood overloads index. | Search outage. |
| DRIVE-DOS05 | TenantCmkEnvelope | DRIVE-B06 | KMS/OpenBao latency blocks decrypt/encrypt. | File read/write outage. |
| DRIVE-DOS06 | FileObjectBytes | DRIVE-B05 | Object store degradation prevents read/write. | Tenant file outage. |

### Elevation of Privilege

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| DRIVE-E01 | PermissionAcl | DRIVE-B03 | User grants themselves owner/admin through ACL mutation. | Unauthorized file control. |
| DRIVE-E02 | ShareLinkRecord | DRIVE-B02 | Anonymous link escalates from view to download/edit. | Unauthorized mutation or exfiltration. |
| DRIVE-E03 | TenantCmkEnvelope | DRIVE-B06 | Worker identity gains CMK access for another tenant. | Cross-tenant decrypt. |
| DRIVE-E04 | DlpFindingRecord | DRIVE-B09 | Reviewer releases own quarantined file. | DLP bypass. |
| DRIVE-E05 | ImmutabilityRetentionRecord | DRIVE-B13 | Non-legal role disables WORM/legal hold. | Evidence deletion. |
| DRIVE-E06 | PreviewArtifact | DRIVE-B10 | Preview sandbox escape obtains service identity. | Lateral movement. |

## DREAD Scoring

| Rank | Threat ID | Threat | Damage | Reproducibility | Exploitability | Affected users | Discoverability | Total |
|---|---|---|---:|---:|---:|---:|---:|---:|
| 1 | DRIVE-I02 | Per-tenant CMK/DEK custody compromise. | 10 | 7 | 7 | 10 | 7 | 41 |
| 2 | DRIVE-I01 | Shared-link file exfiltration. | 9 | 9 | 8 | 8 | 7 | 41 |
| 3 | DRIVE-E03 | Worker identity decrypts another tenant's files. | 10 | 7 | 6 | 10 | 7 | 40 |
| 4 | DRIVE-T02 | File-versioning rollback or fork attack. | 9 | 8 | 7 | 8 | 7 | 39 |
| 5 | DRIVE-E06 | Preview sandbox escape. | 10 | 6 | 7 | 8 | 7 | 38 |
| 6 | DRIVE-I04 | OCR-based PII leak through embedded image text. | 8 | 8 | 7 | 8 | 7 | 38 |
| 7 | DRIVE-S01 | Shared link replay or takeover. | 8 | 9 | 8 | 7 | 6 | 38 |
| 8 | DRIVE-T07 | Retention or WORM weakening. | 9 | 6 | 6 | 8 | 7 | 36 |
| 9 | DRIVE-DOS02 | Archive bomb saturates scanners. | 8 | 9 | 8 | 7 | 4 | 36 |
| 10 | DRIVE-T06 | DLP verdict tampering. | 9 | 7 | 6 | 8 | 6 | 36 |
| 11 | DRIVE-DOS01 | Multipart upload flood. | 7 | 9 | 8 | 7 | 4 | 35 |
| 12 | DRIVE-T01 | Object bytes altered after upload. | 9 | 6 | 5 | 8 | 6 | 34 |
| 13 | DRIVE-E01 | ACL mutation to owner/admin. | 8 | 7 | 6 | 7 | 6 | 34 |
| 14 | DRIVE-DOS05 | KMS/OpenBao latency blocks files. | 8 | 7 | 5 | 9 | 4 | 33 |
| 15 | DRIVE-I05 | Cross-tenant filename enumeration. | 6 | 8 | 7 | 7 | 5 | 33 |

## Attack Trees

### Opportunistic Adversary: Shared-Link Exfiltration

- Goal: access a file without authorization.
  - Path O1: discover a share URL from browser history, chat, email, or referrer.
  - Path O2: replay link before TTL or view cap expires.
  - Path O3: brute weak password if share link uses password protection.
  - Path O4: bypass recipient binding if anonymous access was enabled.
  - Path O5: download file or preview before owner revokes.
- Required break: link entropy, TTL, and recipient binding are insufficient.
- Required break: `ShareLinkAccessed` telemetry does not alert on unusual ASN.
- Detection pivot: `ShareLinkIssued`, `ShareLinkAccessed`, `AbuseDefenceScrapeBlocked`.

### Targeted Adversary: CMK Custody Attack

- Goal: decrypt tenant file corpus.
  - Path T1: obtain worker identity or OpenBao token.
  - Path T2: call KMS/OpenBao for tenant CMK or DEK unwrap.
  - Path T3: use object-store prefix listing to enumerate file objects.
  - Path T4: decrypt file bytes outside normal access path.
  - Path T5: suppress key access telemetry.
- Required break: tenant-bound key policy allows wrong workload or tenant.
- Required break: ADR-0263 event missing audit_id or source_microservice.
- Detection pivot: key access anomaly, `AbuseDefenceAttestationFailed`, and drive decrypt metric.

### Insider Adversary: Version-Chain Tamper

- Goal: replace or erase incriminating file version.
  - Path I1: gain owner/admin role.
  - Path I2: upload benign replacement with old timestamp.
  - Path I3: force conflict resolver to mark malicious version as obsolete.
  - Path I4: purge prior version before legal hold applies.
  - Path I5: poison search index to hide version evidence.
- Required break: version hashes are not append-only or audit-sealed.
- Required break: retention floor can be lowered without legal role.
- Detection pivot: `FileUpdated`, `SyncConflictDetected`, `ImmutabilityTierEntered`.

### Nation-State Adversary: Preview/OCR Sandbox Attack

- Goal: use document parsing to exfiltrate PII or execute code.
  - Path N1: upload crafted Office/PDF/image file.
  - Path N2: trigger preview or OCR worker.
  - Path N3: exploit parser bug or external resource fetch.
  - Path N4: read OCR sidecar or preview cache across tenant.
  - Path N5: pivot using worker identity to object store or KMS.
- Required break: preview sandbox allows network or host filesystem access.
- Required break: OCR sidecar is indexed without source ACL binding.
- Detection pivot: preview worker crash, `DlpFlagRaised`, `AbuseDefenceAttestationFailed`.

## Mitigations Currently In Place

| Threat ID | Named mitigation | ADR or policy | Named code path or doc |
|---|---|---|---|
| DRIVE-S01 | High-entropy signed links, TTL, recipient binding, and revoke audit. | ADR-0243 | `../decisions/ADR-DRIVE-0003-share-link-security-model.md`; `../runbooks/share-link-takeover-incident.md`. |
| DRIVE-I01 | Share access emits event and enforces scope before object read. | ADR-0263 | `../contracts/asyncapi/drive-events.yaml`; `../IP-009-share-link.md`. |
| DRIVE-I02 | Tenant CMK/KEK/DEK envelope with OpenBao/KMS custody. | ADR-0244 | `../decisions/ADR-DRIVE-001-tenant-cmk-kek-dek-envelope-encryption.md`. |
| DRIVE-E03 | Tenant-bound workload identity and KMS path isolation. | ADR-0243 | `../policy/tenant-scope.cedar`; `../policy/data-residency.md`. |
| DRIVE-T02 | Version hashes, event emission, and conflict detection. | ADR-0003 | `../IP-008-sync.md`; `../contracts/asyncapi/drive-events.yaml`. |
| DRIVE-T07 | WORM/immutability tier and legal hold events. | ADR-0003 | `../IP-014-immutability-tier.md`; `../runbooks/immutability-tier-violation.md`. |
| DRIVE-DOS02 | Scanner quarantine, archive handling, and rollback playbook. | ADR-0297 | `../runbooks/virus-scan-rollback.md`. |
| DRIVE-I04 | OCR output inherits source ACL and DLP classification. | ADR-0243 | `../IP-012-preview.md`; `../IP-013-dlp-virus-scan.md`. |
| DRIVE-E06 | Preview sandboxing with no network and restricted filesystem. | ADR-0243 | `../decisions/ADR-DRIVE-0005-preview-pipeline-sandboxing.md`. |
| DRIVE-DOS01 | Upload session TTL, chunk checksum, and stuck upload runbook. | ADR-0297 | `../runbooks/upload-multipart-stuck.md`. |
| DRIVE-T06 | DLP finding is sealed and reviewer release is audited. | ADR-0263 | `../runbooks/dlp-quarantine-release.md`. |
| DRIVE-DOS06 | Object store degraded runbook and storage SLOs. | ADR-0009 | `../runbooks/object-storage-degraded.md`. |

## Residual Risks Accepted

| Risk ID | Residual risk | Risk owner | Compensating control | Review trigger |
|---|---|---|---|---|
| DRIVE-RR01 | Users can intentionally publish files through public share links. | axis-drive | TTL defaults, recipient binding, watermarking, and owner notifications. | Share policy change. |
| DRIVE-RR02 | Tenant CMK custody depends on OpenBao/KMS availability and policy correctness. | council-security | Key access audit, tenant-bound paths, and emergency key freeze. | KMS incident. |
| DRIVE-RR03 | Malware scanners can miss zero-day payloads. | ops-security | Multi-engine scan for regulated packs and quarantine rollback. | Malware false negative. |
| DRIVE-RR04 | OCR can extract PII from images that users did not classify. | council-privacy | DLP on extracted text and inherited ACL enforcement. | OCR pipeline change. |
| DRIVE-RR05 | Preview tools carry parser CVE risk. | axis-drive | Sandbox isolation and fast disable flag. | Preview CVE. |
| DRIVE-RR06 | Sync clients can hold stale state while offline. | axis-drive | Conflict detection and user-visible conflict resolution. | Sync conflict spike. |
| DRIVE-RR07 | Search index term frequency can reveal sensitive patterns. | council-privacy | Tenant-partitioned encrypted index and minimal telemetry. | Search backend change. |
| DRIVE-RR08 | Legal hold and WORM policies can increase storage blast radius during abuse. | ops-legal | Cost alerts and hold scope review. | Legal hold expansion. |
| DRIVE-RR09 | Object store vendor outage can block file operations. | ops-sre-reliability | Degraded-mode runbook and multi-region posture. | Object store SLO burn. |
| DRIVE-RR10 | External recipient identity binding can be weak when recipient has no Oyatie account. | axis-drive | Password KDF, view caps, and link revoke. | External sharing expansion. |

## Specific Telemetry for Detection

ADR-0263 detection telemetry must include `tenant_id`, `sub_scope_path`,
`event_id`, `trace_id`, `span_id`, `audit_id`, `schema_version`,
`source_microservice`, `cell_id`, and `jurisdiction_code` for state-changing
drive events. Cedar denial events include policy id, principal, action,
resource, decision, and denied reason.

| Threat ID | Detection telemetry | ADR-0263 class or service event | Signal |
|---|---|---|---|
| DRIVE-S01 | Share link access from unusual ASN, high download count, recipient mismatch. | `ShareLinkAccessed`, `AbuseDefenceScrapeBlocked` | Shared-link exfiltration. |
| DRIVE-I02 | Key access outside normal service path or tenant mismatch. | `AbuseDefenceAttestationFailed`, key access audit event | CMK custody compromise. |
| DRIVE-T02 | Version fork, rollback, or conflict after legal hold. | `FileUpdated`, `SyncConflictDetected`, `ImmutabilityTierEntered` | Version-chain attack. |
| DRIVE-DOS02 | Scanner timeout, archive bomb, signature rollback. | `VirusDetected`, `AbuseDefenceRateLimitHit` | Malware upload or scanner DoS. |
| DRIVE-I04 | OCR-derived PII finding, embedded image text, DLP rule match. | `DlpFlagRaised`, `DlpQuarantined` | OCR-based PII leak. |
| DRIVE-E06 | Preview worker crash, sandbox denial, unexpected network attempt. | `AbuseDefenceAttestationFailed`, preview worker alert | Sandbox escape attempt. |
| DRIVE-T07 | Retention override attempt or WORM state change. | `ImmutabilityTierEntered`, `OfficeBoundaryAttemptDenied` | Retention tamper. |
| DRIVE-I05 | Filename enumeration, folder listing anomaly, cross-tenant deny. | `OfficeBoundaryAttemptDenied`, `ConglomeratePersonalTenantBoundaryRefused` | Metadata leak attempt. |
| DRIVE-DOS01 | Staging bucket growth, chunk checksum failures, session TTL spikes. | `AbuseDefenceQuotaExceeded`, upload session alert | Multipart upload flood. |
| DRIVE-E01 | ACL owner/admin mutation by unexpected actor. | `PermissionChanged`, `OfficeBoundaryClearanceRequested` | Privilege escalation. |
| DRIVE-T06 | DLP verdict changed or release without reviewer separation. | `DlpFlagRaised`, `OfficeBoundaryClearanceApproved` | DLP tamper. |
| DRIVE-DOS06 | Object store latency, error rate, read/write failure. | `AbuseDefenceVendorOutage`, object-storage SLO burn | Storage outage. |

## Threat Coverage Ledger

### DRIVE-COV01: Shared-link coverage

- Threats covered: DRIVE-S01, DRIVE-I01, DRIVE-T03, DRIVE-E02.
- Asset coverage: ShareLinkRecord and FileObjectBytes.
- Boundary coverage: DRIVE-B02 and DRIVE-B14.
- Required control evidence: signed link entropy, TTL, view cap, recipient binding, revoke event.
- Detection evidence: `ShareLinkIssued`, `ShareLinkAccessed`, `ShareLinkRevoked`.

### DRIVE-COV02: CMK custody coverage

- Threats covered: DRIVE-I02, DRIVE-E03, DRIVE-T04, DRIVE-DOS05.
- Asset coverage: TenantCmkEnvelope.
- Boundary coverage: DRIVE-B06.
- Required control evidence: OpenBao/KMS policy, tenant-bound key path, key rotation audit, failure runbook.
- Detection evidence: key access audit, HSM/KMS latency, and `AbuseDefenceAttestationFailed`.

### DRIVE-COV03: Version-chain coverage

- Threats covered: DRIVE-T02, DRIVE-R03, DRIVE-T08.
- Asset coverage: FileVersionChain and SyncDeltaRecord.
- Boundary coverage: DRIVE-B08 and DRIVE-B12.
- Required control evidence: content hashes, append-only version events, conflict detection.
- Detection evidence: `FileUpdated`, `SyncDeltaApplied`, and `SyncConflictDetected`.

### DRIVE-COV04: Malware upload coverage

- Threats covered: DRIVE-T01, DRIVE-DOS02, DRIVE-S06.
- Asset coverage: FileObjectBytes and MalwareScanVerdict.
- Boundary coverage: DRIVE-B09 and DRIVE-B05.
- Required control evidence: scan-before-release, quarantine state, scanner signature version.
- Detection evidence: `VirusDetected` and virus-scan rollback trigger.

### DRIVE-COV05: OCR PII coverage

- Threats covered: DRIVE-I03, DRIVE-I04, DRIVE-T06.
- Asset coverage: PreviewArtifact, SearchIndexToken, DlpFindingRecord.
- Boundary coverage: DRIVE-B10 and DRIVE-B11.
- Required control evidence: OCR sidecar ACL inheritance, DLP on extracted text, no raw OCR in telemetry.
- Detection evidence: `DlpFlagRaised`, PII scrubber failure, and preview worker alert.

### DRIVE-COV06: Preview sandbox coverage

- Threats covered: DRIVE-E06, DRIVE-DOS03.
- Asset coverage: PreviewArtifact.
- Boundary coverage: DRIVE-B10.
- Required control evidence: sandbox runtime, no network, no host filesystem, parser CVE response.
- Detection evidence: sandbox violation event and preview error rate.

### DRIVE-COV07: Permission coverage

- Threats covered: DRIVE-S02, DRIVE-E01, DRIVE-R02.
- Asset coverage: PermissionAcl and FileMetadataRecord.
- Boundary coverage: DRIVE-B03 and DRIVE-B04.
- Required control evidence: Cedar ACL gate, inherited permission evaluation, permission change audit.
- Detection evidence: `PermissionChanged` and `OfficeBoundaryAttemptDenied`.

### DRIVE-COV08: Retention coverage

- Threats covered: DRIVE-T07, DRIVE-E05, DRIVE-R06.
- Asset coverage: ImmutabilityRetentionRecord.
- Boundary coverage: DRIVE-B13.
- Required control evidence: WORM transition, legal-hold invariant, retention override deny.
- Detection evidence: `ImmutabilityTierEntered`, `LegalHoldApplied`, and immutability runbook trigger.

### DRIVE-COV09: Upload staging coverage

- Threats covered: DRIVE-S04, DRIVE-DOS01.
- Asset coverage: UploadSessionState.
- Boundary coverage: DRIVE-B07.
- Required control evidence: chunk checksum, upload TTL, idempotency key, staging prefix isolation.
- Detection evidence: upload session anomaly and `AbuseDefenceQuotaExceeded`.

### DRIVE-COV10: Telemetry privacy coverage

- Threats covered: DRIVE-I06, DRIVE-S07.
- Asset coverage: AuditEmissionEnvelope.
- Boundary coverage: DRIVE-B14.
- Required control evidence: ADR-0263 PII scrubbing, audit_id on state changes, no raw file path or OCR text in logs.
- Detection evidence: log schema validator and audit-chain completeness SLO.

## Incident Response Playbook References

| Incident class | Runbook |
|---|---|
| Shared link takeover or public exposure | `../runbooks/share-link-takeover-incident.md` |
| Object storage degraded or unavailable | `../runbooks/object-storage-degraded.md` |
| Multipart upload stuck or staging exhaustion | `../runbooks/upload-multipart-stuck.md` |
| Malware scanner rollback or unsafe release | `../runbooks/virus-scan-rollback.md` |
| DLP quarantine release | `../runbooks/dlp-quarantine-release.md` |
| Sync conflict and stale overwrite | `../runbooks/sync-conflict-resolution.md` |
| Immutability tier violation | `../runbooks/immutability-tier-violation.md` |

## Cross-References

- Root service architecture: `../ARCHITECTURE.md`.
- Product requirements: `../PRD.md`.
- Drive events contract: `../contracts/asyncapi/drive-events.yaml`.
- Drive OpenAPI contract: `../contracts/openapi/drive.yaml`.
- Object storage decision: `../decisions/ADR-DRIVE-0001-object-storage-substrate-selection.md`.
- Share-link security decision: `../decisions/ADR-DRIVE-0003-share-link-security-model.md`.
- Encryption decision: `../decisions/ADR-DRIVE-0004-encryption-at-rest-and-e2e.md`.
- Tenant CMK decision: `../decisions/ADR-DRIVE-001-tenant-cmk-kek-dek-envelope-encryption.md`.
- Preview sandboxing decision: `../decisions/ADR-DRIVE-0005-preview-pipeline-sandboxing.md`.
- Immutability decision: `../decisions/ADR-DRIVE-0006-immutability-and-worm-policy.md`.
- Share-link implementation packet: `../IP-009-share-link.md`.
- Permissions implementation packet: `../IP-010-permissions.md`.
- Search index implementation packet: `../IP-011-search-index.md`.
- Preview implementation packet: `../IP-012-preview.md`.
- DLP and virus scan implementation packet: `../IP-013-dlp-virus-scan.md`.
- Tenant scope policy: `../policy/tenant-scope.cedar`.
- Public read policy: `../policy/public-read.cedar`.
- Dual-context policy: `../policy/dual-context-isolation.md`.
- ADR-0263 observability emission contract: `../../../docs/decisions/ADR-0706-observability-live-apex.md`.
- ADR-0243 Cedar as universal gate: `../../../docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- ADR-0244 tenant as universal scoping primitive: `../../../docs/decisions/ADR-0702-identity-authz-live-apex.md`.
- ADR-0297 abuse defence baseline: `../../../docs/decisions/ADR-0700-ci-admission-live-apex.md`.

## Checkpoint Notes

- This document does not modify drive decisions or runbooks.
- It references existing incident playbooks instead of editing them.
- It treats previews, OCR, thumbnails, and search tokens as derived sensitive data.
- It assumes all file state-changing operations emit audit_id per ADR-0263.
