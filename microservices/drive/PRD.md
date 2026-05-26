---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-drive
microservice: drive
status: Accepted
sales_segment: shared-substrate + suite-app
tier: tenant-facing
milestone_first_ship: M02-product-tier-foundation
bominal_source: [ADR-Bominal-workspace-drive, ADR-Bominal-connect-files]
related_adrs: [ADR-0056, ADR-0105, ADR-0106, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0140 (retired per ADR-0145), ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345, ADR-DRIVE-0001, ADR-DRIVE-0002, ADR-DRIVE-0003, ADR-DRIVE-0004, ADR-DRIVE-0005, ADR-DRIVE-0006]
related_specs: [/specs/microservices/workspace/drive.json, /specs/per-microservice-flat-layout.json, /specs/agentic-slo-gated-promotion.json]
date: 2026-05-17
owner_team: axis-drive
doc_status: published
---

# PRD-drive: Drive — file storage + sync µservice

## Purpose

The `drive` µservice is oyatie's native object/file storage, hierarchical folder, multipart resumable upload, range-download, content-defined-chunking delta-sync, shared-link, fine-grained permission, full-text search, preview/thumbnail, virus-scan, DLP, encryption-at-rest, immutability-tier (WORM), and per-tenant quota substrate. Per ADR-0132 (no-suite forward policy) and ADR-0135 (Connect unbundle parallel session) drive is a standalone tenant-facing µservice — separate from docs/sheets/slides editing surfaces — owning bytes-at-rest, bytes-in-flight, hierarchy, sharing, sync, preview, DLP, retention, and immutability.

The µservice carries dual-context (Personal / Professional) per parallel ADR-0238; bytes never cross context boundaries except via explicit share-link issuance or policy-bound projection.

Bominal inheritance: Bominal's `workspace.drive` + `connect.files` ADRs are inherited 1:1 per `feedback_bominal_inheritance_precedence.md`; oyatie additions captured below.

Strangler precedent: the legacy `oya-connect-drive-domain` crate at `crates/oya-connect-drive-domain/` migrates via `microservices/drive/migration-from-connect.md` and `deprecation-notice.md` per ADR-0135 / ADR-0134.

## Tenant Value

- **Tenant Outcome 1 — Storage without third-party dependency.** Tenants do not need Google Drive / Dropbox / OneDrive / Box / iCloud Drive / Proton Drive / Nextcloud / pCloud / Sync.com / MEGA accounts; the µservice is a native first-party storage substrate.
- **Tenant Outcome 2 — Open protocol surface.** S3-compatible API (AWS SigV4) + WebDAV (RFC 4918) + HTTP Range Requests (RFC 9110) + resumable upload (S3 multipart + tus.io 1.0); any S3 / WebDAV / tus client interoperates with no custom SDK.
- **Tenant Outcome 3 — Native delta-sync.** Content-defined-chunking (FastCDC) + LBFS-style delta-sync; desktop / mobile clients sync efficiently across slow links without re-uploading whole files.
- **Tenant Outcome 4 — End-to-end encryption (opt-in).** Personal-context tenants can opt into client-side E2E encryption; Professional-context tenants always have tenant-DEK envelope at rest.
- **Tenant Outcome 5 — WORM immutability tier.** Legal-hold + per-pack retention floors with object-lock semantics matching AWS S3 Object Lock (compliance mode); SEC 17a-4(f) + FINRA 4511 + HIPAA §164.316 ready.
- **Tenant Outcome 6 — Per-tenant quota + usage telemetry.** Tenants see real-time storage + bandwidth usage; soft + hard quotas; archive-tier auto-tiering.
- **Internal Outcome 7 — Cross-µservice file backend.** docs / sheets / slides store their authoritative bytes here; mail attaches via attachment-bridge; messenger embeds file-share-links; workflow-engine triggers on file-change events.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | tenant operator | to upload a file via multipart resumable (RFC 7233 + S3 multipart + tus 1.0) | 100GB uploads survive flaky links | upload | Must |
| FR-02 | tenant operator | to download a file with HTTP Range support (RFC 9110) | streaming + partial fetch works | download | Must |
| FR-03 | tenant operator | to organise files in nested folders | hierarchy is human-legible | folder | Must |
| FR-04 | tenant operator | to share a file/folder via signed link (public / password / expiring / view-count cap) | external collab works without granting accounts | share-link | Must |
| FR-05 | tenant operator | to set per-folder + per-file permissions (read/comment/edit/manage) with inheritance | fine-grained access control | permissions | Must |
| FR-06 | desktop/mobile client | to sync only changed chunks via delta-sync (FastCDC + LBFS) | low-bandwidth sync works | sync | Must |
| FR-07 | tenant operator | to soft-delete files into Trash with retention before purge | accidental deletion recoverable | trash | Must |
| FR-08 | tenant operator | to search files by filename + full-text content via Apache Tika + Meilisearch | findability at 1M-file scale | search | Must |
| FR-09 | tenant operator | to preview image/PDF/Office/video without downloading | quick triage | preview | Must |
| FR-10 | tenant operator | to be protected against malware via ClamAV/OPSWAT scan on every upload | upload safety | virus-scan | Must |
| FR-11 | tenant operator | to apply DLP scan before any share-out leaves the tenant | data-loss prevention | dlp | Must |
| FR-12 | tenant operator | to put files under WORM (immutable) tier with retention floor | SEC 17a-4(f) / FINRA 4511 / HIPAA §164.316 compliance | immutability | Must |
| FR-13 | tenant operator | to view real-time quota + bandwidth usage; receive soft+hard threshold alerts | capacity awareness | quota-usage | Must |
| FR-14 | tenant operator | to subscribe to webhooks on file-change events | downstream Workflow can react | (cross-cutting) | Must |
| FR-15 | tenant operator | to transfer ownership of files/folders | offboarding workflows | permissions | Must |
| FR-16 | tenant operator | to grant third-party app access via OAuth scope (docs/sheets/slides) | first-class app ecosystem | third-party-app | Must |
| FR-17 | tenant operator | to backup + restore full file tree | DR + portability | backup-restore | Must |
| FR-18 | tenant operator | to access "Shared with me" view across tenants (with cross-tenant invite) | cross-tenant collab | shared-with-me | Should |
| FR-19 | tenant operator | to opt into client-side E2E encryption for personal-pillar | zero-knowledge storage | encryption-e2e | Should |
| FR-20 | tenant operator | to audit-log every read / write / share | compliance + forensic | (cross-cutting) | Must |
| FR-21 | tenant compliance officer | to put a professional file under legal hold | preserves content past retention expiry | immutability | Must |
| FR-22 | sync client | to detect conflicts deterministically (last-writer-wins with deterministic tie-break) | predictable sync semantics | sync | Must |
| FR-23 | tenant operator | to prune older versions per per-tenant version-retention policy | storage cost control | versioning | Should |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| File-list folder (1k entries) | ≤ 40ms | ≤ 150ms | ≤ 400ms | Postgres + index on `(parent_folder_id, name)`; cache-hit > 80% |
| Upload multipart 1GB (parallel chunks) | ≤ 30s | ≤ 90s | ≤ 180s | per-chunk 8MiB FastCDC; ≥ 4 concurrent parts |
| Download first-byte (warm CDN) | ≤ 50ms | ≤ 100ms | ≤ 250ms | per-cell edge cache |
| Download first-byte (cold) | ≤ 200ms | ≤ 500ms | ≤ 1s | direct S3 / Garage / Ceph RGW fetch |
| Search query (1M-file corpus) | ≤ 100ms | ≤ 400ms | ≤ 1s | Meilisearch + Tika-indexed |
| Sync delta (100 changed files) | ≤ 10s | ≤ 30s | ≤ 60s | rolling-hash diff over LBFS |
| Share-link generation | ≤ 20ms | ≤ 50ms | ≤ 150ms | HMAC + KDF Argon2id |
| Preview render (image 4K) | ≤ 250ms | ≤ 1s | ≤ 2.5s | libvips + ImageMagick |
| Preview render (PDF 100p first page) | ≤ 400ms | ≤ 1s | ≤ 3s | qpdf + Mozilla pdf.js |
| Preview render (Office 50p first page) | ≤ 600ms | ≤ 1s | ≤ 4s | LibreOffice in gVisor sandbox |
| Virus-scan correctness | n/a | 100% | 100% | every upload scanned before promotion to durable |
| Immutability-tier correctness | n/a | 100% | 100% | WORM violation is zero-tolerance |

### Security

- All bytes-at-rest encrypted under tenant-DEK envelope (per Bominal ADR-0111) when in Professional context; Personal context supports opt-in client-side E2E (libsodium secretstream).
- All share-link signatures use Ed25519 + HKDF-derived signing keys per share-link generation; password-protected links use Argon2id (RFC 9106).
- All ingress is TLS 1.3 + per-tenant API key + RBAC; S3-API ingress uses AWS SigV4 + tenant-bound access-key.
- All uploads pass through ClamAV + OPSWAT MetaDefender scan before being promoted from staging bucket to durable bucket; quarantined objects never reach durable storage.
- DLP scan runs on every share-out (link generation OR cross-tenant transfer); flagged content blocks the share until tenant-policy resolves.
- Office preview render runs in gVisor sandbox with no network + no host filesystem access; preview output is rasterised (PNG) to prevent macro-execution leakage.

### Audit + Compliance

- Every `FileUploaded / FileDownloaded / FileShared / ShareLinkAccessed / PermissionChanged / VersionPruned / FileMovedToTrash / FilePurged / LegalHoldApplied / ImmutabilityTierEntered` emits an audit-chain record (Merkle + Ed25519 per Bominal ADR-0028).
- Legal-hold preserves file + version history + share-link history past retention expiry.
- Immutability tier honours object-lock semantics matching AWS S3 Object Lock compliance mode; even tenant-root cannot purge an object under WORM.
- Per-jurisdiction retention (KR PIPA / EU GDPR / US SEC 17a-4 / HIPAA / etc.) computed per ADR-0140 Cedar pack overlay.

### Availability + SLO

- Availability target: 99.95 % monthly for read path; 99.9 % for write path.
- RTO ≤ 15 min; RPO ≤ 60 s (Postgres logical replication + S3 cross-AZ replication).
- Cross-region replication for backup-restore (off by default; SCC-gated when activated per ADR-0117).

### Data residency

- Tenant bytes pinned to the tenant's pack region per ADR-0117 + ADR-0140; cross-region replication forbidden by default; SCC-gated when activated.

### DR Posture (ADR-0343)

- RTO/RPO target: manifest `dr` declares `rto_p99_seconds=900` and `rpo_p99_seconds=60` for file metadata, folder state, permissions, and version manifests. HIPAA-2024 (3600s/300s), SOC2-T2 (14400s/900s), NIS2-aligned operational continuity, ISO27001-2022 (14400s/3600s), and KR-CSAP-v3.1 (3600s/900s) leave the effective drive bound at 900s RTO and 60s RPO.
- failover_runbook: `runbooks/dr-failover.md`; manifest backup substrate is `postgres_wal_g`, `object_storage_versioned`, `seaweedfs_replicated`, and `valkey`.
- multi_region_active_active: true, with manifest replication shape `active-active-multi-az-cross-region-warm`; immutable object bytes remain pack-pinned during promotion.
- WHY: tenants can continue file listing, ownership, quota, and WORM/legal-hold decisions while large object recovery proceeds without breaking residency.

### Capacity Model (ADR-0340)

- Per-tenant baseline: manifest `capacity_model` declares 0.4 vCPU, 1024Mi RAM, 51200Gi storage, 4 Valkey connections, 4 Postgres connections, and 8 outbound HTTP connections per tenant.
- Scaling dimension: `per_request`; file-list, upload, download, sync, preview-render, and DLP scan traffic drive load while byte storage remains intentionally large because drive is the durable file source of record.
- Cell placement class: Tier-3, matching manifest `capacity_model.cell_placement_class`, because drive is a high-throughput product/substrate surface rather than tenant-customer code execution.
- Autoscaling boundaries: service Helm min 3 / max 100 for REST surfaces, worker pools scale on queue depth, and object-store/postgres/Valkey scale-out triggers follow `capacity-model.md` saturation guardrails.
- WHY: drive serves both user-facing file operations and substrate byte storage, so scale must follow file count, byte volume, and scan queues rather than user count alone.

### Sustainability + Cost Attribution (ADR-0344)

- Every audit-chain row emits `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, and `region` for upload, download, share, permission, version, trash, purge, legal-hold, immutability, preview, virus-scan, and DLP events.
- Provider routing affected by carbon: yes for preview renders, archive tiering, retention compaction, and scan backlogs; no for DLP enforcement, upload promotion, WORM/legal-hold, malware quarantine, or interactive download.
- Per-tenant transparency surface: FinOps portal shows storage GB-months by tier, bandwidth, upload/download requests, preview CPU, scan jobs, WORM objects, and cross-product byte consumption by tenant/capability/provider/cell/compliance_pack.
- WHY: drive is the largest byte-at-rest cost center in this bucket, so CSRD, SB-253, and SEC climate disclosure need tenant-visible storage and egress attribution without delaying security gates.

### API Versioning Posture (ADR-0342)

- Public API version model: YYYY-MM-DD carrier triplet via `Oyatie-Version` header, `/v/<YYYY-MM-DD>` URL prefix, and proto3 `oyatie_version` field for file, folder, upload, download, sync, share-link, permissions, search, preview, and immutability contracts.
- SDK semver model: major.minor.patch for S3/WebDAV/tus bridges, first-party clients, and cross-service file SDKs.
- Support window: last N=3 public API versions for at least 180 days; object-lock and share-link schemas cannot be removed inside an active retention window.
- Per-tenant pinning supported: yes, including regulated tenants validating S3-compatible, WebDAV, and resumable-upload flows.
- Internal-mesh exemption: yes; direct gRPC consumed by docs/sheets/slides/mail/messenger remains exempt under ADR-0145 when it is internal-only.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename for new crates). 11 primary BCs.

| BC | Crate family | Purpose | Key entities |
|---|---|---|---|
| `file-store` | `oya-drive-file-store-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-s3,adapter-garage,adapter-seaweedfs,rest,worker,sdk,app}` | Object/file persistence; content-addressing; version history; tenant-DEK encryption | `File`, `FileVersion`, `ContentAddress`, `RetentionPolicyRef`, `LegalHoldRef` |
| `folder-hierarchy` | `oya-drive-folder-hierarchy-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,app}` | Nested folder tree; per-folder permission inheritance | `Folder`, `FolderPermission`, `FolderTree` |
| `upload` | `oya-drive-upload-{kernel,domain,usecase,api,adapter,adapter-valkey,adapter-s3,rest,worker,app}` | Multipart resumable; chunk staging; virus-scan pipeline | `UploadSession`, `Chunk`, `ChunkRef` |
| `download` | `oya-drive-download-{kernel,domain,usecase,api,adapter,adapter-s3,rest,app}` | Range-request serving; signed-URL minting; CDN steering | `DownloadTicket`, `RangeRequest`, `SignedDownloadUrl` |
| `sync` | `oya-drive-sync-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,sdk,app}` | Content-defined chunking; delta protocol; conflict resolution | `SyncSession`, `ChunkManifest`, `DeltaSet`, `SyncConflict` |
| `share-link` | `oya-drive-share-link-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,app}` | Signed-link minting; password / expiring / view-cap | `ShareLink`, `LinkSignature`, `LinkAccessRecord` |
| `permissions` | `oya-drive-permissions-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,app}` | Per-file + per-folder ACL; inheritance + per-file override | `Permission`, `PermissionScope`, `OwnershipTransfer` |
| `search-index` | `oya-drive-search-index-{kernel,domain,usecase,api,adapter,adapter-meilisearch,adapter-tika,rest,worker,app}` | Filename + full-text indexing | `SearchIndex`, `IndexJob`, `TikaExtract` |
| `preview` | `oya-drive-preview-{kernel,domain,usecase,api,adapter,adapter-libvips,adapter-qpdf,adapter-libreoffice,adapter-ffmpeg,rest,worker,app}` | Thumbnail + preview rendering in sandbox | `PreviewArtifact`, `RenderJob`, `SandboxedRender` |
| `dlp-virus-scan` | `oya-drive-dlp-virus-scan-{kernel,domain,usecase,api,adapter,adapter-clamav,adapter-opswat,worker,app}` | Virus scan + DLP scan pipeline | `ScanJob`, `Verdict`, `QuarantineRecord` |
| `immutability-tier` | `oya-drive-immutability-tier-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,app}` | WORM retention; legal hold; object-lock semantics | `ImmutabilityRecord`, `RetentionFloor`, `LegalHoldRef` |

Naming justification (representative; same shape applies to others) — `file-store`:

```
NAME: oya-drive-file-store-<layer>
JUSTIFICATION:
- microservice = drive: this µservice; ADR-0056 v4.1 flat BNF + ADR-0131 per-microservice
  folder. No shared|vertical bisection.
- bc-tokens = file-store: primary BC for byte-at-rest persistence; siblings (folder-hierarchy,
  upload, download, sync, share-link, permissions, search-index, preview, dlp-virus-scan,
  immutability-tier) justify explicit BC token per ADR-0056 v4.1 BC-optionality rule.
- layer = <layer>: one crate per layer per ADR-0105 13-value canonical enum.
  - kernel: port-trait + entity types (File, FileVersion, ContentAddress,
    RetentionPolicyRef, LegalHoldRef, FileContext{Personal|Professional}). Zero I/O.
    data_class annotations.
  - domain: pure invariant math (content-address derivation, version ordering, retention
    arithmetic, hold coverage).
  - usecase (per ADR-0106): orchestrators (put-file, get-file, delete-file, apply-legal-
    hold, expire-retention) reading via ports.
  - api: protocol-neutral typed contracts.
  - adapter: protocol-neutral implementations of kernel ports.
  - adapter-postgres: backend-qualified adapter (per ADR-0105 Amendment 3) for metadata.
  - adapter-s3: backend-qualified adapter for S3-compatible object store (Garage / SeaweedFS
    / SeaweedFS / Ceph RGW pluggable per ADR-DRIVE-0001).
  - adapter-garage / adapter-seaweedfs: alternate backend-qualified adapters per
    ADR-DRIVE-0001 backend-pluggability matrix.
  - rest: HTTP handler/route layer; exposes both the oya-native API and the S3-compat facade.
  - worker: long-lived workers (retention sweep, hold cascade, version pruner).
  - sdk: client library for tenants + workflow consumers.
  - app: composition root binary.
- exemptions claimed: none.
```

(Equivalent justifications recorded for the other ten BCs at `microservices/drive/specs/naming-justification.md`.)

Layer mapping table per BC (13-layer enum from ADR-0105; `usecase` per ADR-0106). Checkmark = crate exists at GA.

| BC | kernel | domain | usecase | api | adapter | adapter-postgres | adapter-valkey | adapter-s3 | adapter-garage | adapter-seaweedfs | adapter-meilisearch | adapter-tika | adapter-clamav | adapter-opswat | adapter-libvips | adapter-qpdf | adapter-libreoffice | adapter-ffmpeg | rest | worker | sdk | app |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `file-store` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | ✓ | ✓ | ✓ | ✓ |
| `folder-hierarchy` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — | — | ✓ |
| `upload` | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | ✓ | ✓ | — | ✓ |
| `download` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | — | — | — | — | — | — | — | — | — | — | ✓ | — | — | ✓ |
| `sync` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | ✓ | ✓ | ✓ |
| `share-link` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | ✓ | — | ✓ |
| `permissions` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — | — | ✓ |
| `search-index` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | ✓ | ✓ | — | — | — | — | — | — | ✓ | ✓ | — | ✓ |
| `preview` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ |
| `dlp-virus-scan` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | ✓ | ✓ | — | — | — | — | — | ✓ | — | ✓ |
| `immutability-tier` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — | ✓ |

Total crates introduced by this µservice: **89**.

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated per Bominal ADR-0028):

| Port trait | Kernel crate | Implemented in | Data classes touched |
|---|---|---|---|
| `FileRepository` | `oya-drive-file-store-kernel` | `-adapter-postgres` (metadata) + `-adapter-s3` / `-adapter-garage` / `-adapter-seaweedfs` (bytes) | `PERSONAL_FILE_CONTENT` + `PROFESSIONAL_FILE_CONTENT` (per-context envelope encryption) |
| `ContentAddressDeriver` | `oya-drive-file-store-kernel` | `-adapter` (Rabin / BuzHash / FastCDC selectable per ADR-DRIVE-0002) | `INTERNAL_ONLY` |
| `FolderTree` | `oya-drive-folder-hierarchy-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT` |
| `UploadSessionStore` | `oya-drive-upload-kernel` | `-adapter-valkey` (in-flight) + `-adapter-s3` (chunk staging) | `PERSONAL_FILE_CONTENT` + `PROFESSIONAL_FILE_CONTENT` (transient) |
| `DownloadUrlSigner` | `oya-drive-download-kernel` | `-adapter` (Ed25519 HKDF) | `SECRET` (signing key) |
| `ChunkManifest` | `oya-drive-sync-kernel` | `-adapter-postgres` | `INTERNAL_ONLY` |
| `DeltaProtocol` | `oya-drive-sync-kernel` | `-adapter` (LBFS rolling hash) | `INTERNAL_ONLY` |
| `ShareLinkSigner` | `oya-drive-share-link-kernel` | `-adapter` (Ed25519 + Argon2id) | `SECRET` + `PII_QUASI_IDENTIFIER` (link bound to recipient) |
| `PermissionResolver` | `oya-drive-permissions-kernel` | `-adapter-postgres` | `AUDIT` |
| `SearchIndexer` | `oya-drive-search-index-kernel` | `-adapter-meilisearch` + `-adapter-tika` | `PERSONAL_FILE_CONTENT` + `PROFESSIONAL_FILE_CONTENT` (full-text) |
| `PreviewRenderer` | `oya-drive-preview-kernel` | `-adapter-libvips` (image) + `-adapter-qpdf` (PDF) + `-adapter-libreoffice` (Office) + `-adapter-ffmpeg` (video) | `PERSONAL_FILE_CONTENT` + `PROFESSIONAL_FILE_CONTENT` |
| `VirusScanner` | `oya-drive-dlp-virus-scan-kernel` | `-adapter-clamav` (primary) + `-adapter-opswat` (multi-engine; pack-us-healthcare + pack-eu) | `PERSONAL_FILE_CONTENT` + `PROFESSIONAL_FILE_CONTENT` |
| `DlpClassifier` | `oya-drive-dlp-virus-scan-kernel` | `-adapter` (in-tree rules + ML model handoff to foundry-runtime) | `PERSONAL_FILE_CONTENT` + `PROFESSIONAL_FILE_CONTENT` |
| `RetentionPolicyResolver` | `oya-drive-immutability-tier-kernel` | `-adapter-postgres` (resolves to `tenancy` µservice via Workflow) | `AUDIT` |
| `LegalHoldStore` | `oya-drive-immutability-tier-kernel` | `-adapter-postgres` | `AUDIT` |
| `ImmutabilityGuard` | `oya-drive-immutability-tier-kernel` | `-adapter` (object-lock enforcement) | `AUDIT` |

Data-class enforcement: every kernel struct field carries a `#[data_class(...)]` annotation; the `oya-check-data-class` LEAN lane refuses unannotated fields.

Cross-product rule: `drive` MUST NOT import another product µservice crate at any layer. Cross-product flows go through Workflow (events) or Ontology (entity reads/writes). Consumed µservices: `tenancy` (tenant + identity resolution), `audit-chain` (seal emission), `mail` (attachment-bridge from outside drive → email send via Workflow), `messenger` (share-link embed in channel), `observability` (telemetry), `foundry-runtime` (T1 OCR / auto-tag / smart-search ML handoff). LEAN-A2 CI lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice drive`
- `oya gate validate lean-a2 --microservice drive`
- `oya gate validate port-location --microservice drive`
- `oya gate validate layer-correctness --microservice drive`
- `oya gate validate per-microservice-layout --microservice drive`
- `oya gate validate statelessness --microservice drive`
- `oya gate validate shardability --microservice drive`
- `oya gate validate hyperscaler-maturity --microservice drive`
- `oya gate validate s3-sigv4-conformance --microservice drive` (NEW)
- `oya gate validate webdav-conformance --microservice drive` (NEW)
- `oya gate validate worm-immutability-correctness --microservice drive` (NEW)

## Integration via Workflow + Ontology

### Workflow events produced

| Event | Topic | Trigger | Consumed by | Idempotency key |
|---|---|---|---|---|
| `FileUploaded` | `drive.file.lifecycle.v1` | new file/version written | search-index, audit-chain, workflow-engine, dlp-virus-scan | `file_id + version` |
| `FileUpdated` | `drive.file.lifecycle.v1` | metadata mutation | search-index, audit-chain | `file_id + version` |
| `FileMovedToTrash` | `drive.file.lifecycle.v1` | soft-delete | audit-chain, observability | `file_id + trashed_at` |
| `FilePurged` | `drive.file.lifecycle.v1` | hard-delete (post-retention) | audit-chain | `file_id + purged_at` |
| `FileDownloaded` | `drive.file.access.v1` | read | audit-chain | `file_id + download_ticket` |
| `ShareLinkIssued` | `drive.share.v1` | share-link minted | audit-chain, workflow-engine, dlp-virus-scan | `link_id` |
| `ShareLinkAccessed` | `drive.share.v1` | someone fetched via signed link | audit-chain, observability | `link_id + access_at + ip_hashed` |
| `ShareLinkRevoked` | `drive.share.v1` | revocation | audit-chain | `link_id + revoked_at` |
| `PermissionChanged` | `drive.permissions.v1` | ACL mutation | audit-chain, ontology | `file_or_folder_id + change_id` |
| `SyncDeltaApplied` | `drive.sync.v1` | delta-set committed | observability | `session_id + delta_hash` |
| `SyncConflictDetected` | `drive.sync.v1` | conflict observed | observability, requester | `session_id + file_id` |
| `VirusDetected` | `drive.scan.v1` | scan verdict = malicious | observability, ops-security, requester | `scan_job_id` |
| `DlpFlagRaised` | `drive.scan.v1` | DLP scan flagged content | observability, council-privacy, requester | `scan_job_id` |
| `ImmutabilityTierEntered` | `audit.drive.immutability.v1` | WORM tier applied | audit-chain, compliance | `file_id + tier_at` |
| `LegalHoldApplied` / `LegalHoldReleased` | `audit.drive.legal_hold.v1` | hold transition | audit-chain, governance | `file_id + hold_id` |
| `QuotaThresholdCrossed` | `drive.quota.v1` | soft/hard threshold crossed | audit-chain, observability, tenant | `tenant_id + level + crossed_at` |

### Workflow events consumed

| Event | Producer | Handler | Action |
|---|---|---|---|
| `TenantOnboarded` | `tenancy` | file-store usecase | provision tenant-DEK; create root folder; set default pack-retention |
| `TenantOffboarded` | `tenancy` | file-store usecase | mark files for retention sweep / legal-hold scan |
| `MailAttachmentRequested` | `mail` | download usecase | mint short-lived signed download URL for attachment-bridge |
| `MessengerFileShareEmbedded` | `messenger` | share-link usecase | mint signed-link bound to channel + viewer scope |
| `WorkflowTrigger` | `workflow-engine` | file-store usecase | file-change-trigger automation (e.g., "on FileUploaded with mime=video/*, kick off transcode job") |
| `FoundryRuntimeOcrComplete` | `foundry-runtime` | search-index usecase | persist OCR text into Tika extract record |
| `FoundryRuntimeAutoTagComplete` | `foundry-runtime` | file-store usecase | apply suggested tags (user-confirmable; T1 reversibility window) |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit |
|---|---|---|---|
| `File{file_id, tenant, context, parent_folder_id, content_address, version, ...}` | `file_of→Tenant`, `version_of→File` | `file-store` | Ed25519 |
| `Folder{folder_id, tenant, parent_folder_id, path, ...}` | `folder_of→Tenant`, `parent→Folder` | `folder-hierarchy` | Ed25519 |
| `ShareLink{link_id, file_or_folder_id, ttl, view_cap, ...}` | `share_of→File`, `share_of→Folder` | `share-link` | Ed25519 |
| `Permission{permission_id, file_or_folder_id, principal, scope}` | `permits→User`, `permits→Group` | `permissions` | Ed25519 |
| `ImmutabilityRecord{record_id, file_id, retention_floor, mode}` | `worm_of→File` | `immutability-tier` | Ed25519 |
| `LegalHold{hold_id, file_id, opened_by, opened_at}` | `holds→File` | `immutability-tier` | Ed25519 |

### Ontology reads

| Object | Read by | Query shape |
|---|---|---|
| `User` (tenant directory) | `permissions`, `share-link`, `file-store` | by `(tenant_id, user_id)` |
| `Group` (tenant groups) | `permissions` | by `(tenant_id, group_id)` |
| `Tenant` | `file-store`, `quota` | by `tenant_id` |
| `RetentionPolicy` | `file-store`, `immutability-tier` | by `(tenant_id, pack, file_class)` |

## Competitive Benchmark

| Competitor | Product | Parity dimensions | Primary source |
|---|---|---|---|
| Google Drive | Workspace Drive | folder hierarchy; share-link; per-folder permissions; OCR; smart-search; CDN download | `developers.google.com/drive` |
| Dropbox | Dropbox + Business | delta-sync (rsync); selective sync; smart-sync; paper preview | `developers.dropbox.com/docs/api` |
| OneDrive | Microsoft 365 OneDrive | Office preview; SharePoint integration; co-authoring | `learn.microsoft.com/onedrive/dev` |
| Box | Box Enterprise | granular permissions (7 levels); governance; DLP | `developer.box.com` |
| iCloud Drive | Apple iCloud | E2E (advanced data protection); deduplication; native macOS/iOS sync | `developer.apple.com/icloud` |
| Proton Drive | Proton AG | E2E by default (libsodium); zero-knowledge | `proton.me/drive/business` |
| Tresorit | Swiss zero-knowledge | E2E by default; FIPS 140-2 KMS | `tresorit.com/business` |
| Nextcloud | Self-hosted open-source | WebDAV; pluggable storage; LDAP; pluggable preview | `docs.nextcloud.com` |
| pCloud | pCloud Business | client-side crypto (paid); lifetime storage; WebDAV | `docs.pcloud.com` |
| Sync.com | Sync.com Business | E2E by default; SOC 2 Type II; HIPAA | `sync.com/help/api` |
| MEGA | MEGA Cloud | E2E by default; pre/post-quantum crypto roadmap | `mega.io/developers` |
| AWS S3 | AWS S3 + Workspaces | S3 API (SigV4); Object Lock (WORM); Glacier tiering | `docs.aws.amazon.com/s3` |
| Wasabi | Wasabi Hot Cloud Storage | S3 API; immutability; no egress fees | `docs.wasabi.com` |
| Backblaze B2 | Backblaze B2 | S3-compat; lifecycle; lower-cost archive | `backblaze.com/b2/docs/` |
| Internxt | Internxt Drive | E2E + zero-knowledge; Spain-resident | `internxt.com/developers` |

Key parity gaps to close (ordered):

1. **WORM immutability tier matching AWS S3 Object Lock compliance mode + SEC 17a-4(f) + FINRA 4511 + HIPAA §164.316** — Box has it; Dropbox doesn't. **Differentiator vs Dropbox-class.**
2. **E2E for personal-pillar matching Proton Drive / Tresorit / MEGA** — Google Drive doesn't have it; Dropbox doesn't have it default. **Differentiator vs incumbents.**
3. **Cross-tenant "Shared with me" with policy-bounded disclosure** — none of the competitors gate cross-org sharing with Cedar-policy + audit-chain. **Differentiator.**
4. **Dual-context (Personal / Professional) isolation enforced structurally** — no competitor enforces context-separation in code. **Differentiator.**
5. **S3-compatible API parity** — required for AWS / Wasabi / Backblaze SDK reuse; passes `s3cmd` / `aws s3` / `mc` (SeaweedFS Client) end-to-end.
6. **WebDAV (RFC 4918) read+write parity** — required for Nextcloud / pCloud / macOS Finder / Windows Explorer native mount.
7. **Delta-sync (FastCDC + LBFS) matching Dropbox** — Google Drive doesn't have it; required for low-bandwidth desktop sync.
8. **Per-folder + per-file permission inheritance + override matching Box** — Box ships 7 access levels; we ship 4 (read/comment/edit/manage) with per-file override.
9. **Native preview parity (image / PDF / Office / video) matching Google Drive** — covered via libvips + qpdf + LibreOffice-in-gVisor + ffmpeg.
10. **Object-store backend pluggability (S3 / Garage / SeaweedFS / SeaweedFS / Ceph RGW)** — gives operator choice between centralised (S3) and edge-distributed (Garage / SeaweedFS) deployment; ADR-DRIVE-0001.

## Performance Targets (canonical bench surface)

| Metric | Target | Verification |
|---|---|---|
| File-list folder (1k entries) p99 | ≤ 150ms | `cargo bench -p oya-drive-folder-hierarchy-adapter-postgres -- folder_list` |
| Upload multipart 1GB p99 | ≤ 90s | `cargo bench -p oya-drive-upload-usecase -- multipart_1gb` |
| Download first-byte (warm CDN) p99 | ≤ 100ms | `cargo bench -p oya-drive-download-adapter-s3 -- first_byte_warm` |
| Search query (1M-file corpus) p99 | ≤ 400ms | `cargo bench -p oya-drive-search-index-adapter-meilisearch -- query_1m` |
| Sync delta (100 changed files) p99 | ≤ 30s | `cargo bench -p oya-drive-sync-usecase -- delta_100files` |
| Share-link generation p99 | ≤ 50ms | `cargo bench -p oya-drive-share-link-usecase -- mint` |
| Preview render image 4K p99 | ≤ 1s | `cargo bench -p oya-drive-preview-adapter-libvips -- image_4k` |
| Virus-scan correctness | 100% | `cargo nextest -p oya-drive-dlp-virus-scan-domain -- virus_correctness` |
| WORM immutability correctness | 100% | `cargo nextest -p oya-drive-immutability-tier-domain -- worm_correctness` |

Error budget: monthly 99.95% availability → ~22 min/month.

## Horizontal Scalability

State strategy (per Bominal ADR-0019): `mixed`. Postgres (metadata; per-tenant RLS); Valkey (upload-session in-flight + delta-sync cache; per-tenant key prefix); S3-compatible object store (bytes; per-tenant prefix); Meilisearch (full-text index; per-tenant index); stateless workers for retention sweep + version pruner + preview renderer + virus scanner + DLP scanner.

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Active tenants | 50k | 500k | Postgres connection pool > 70% |
| Files stored | 1B | 10B | object-store list latency > 100ms |
| Bytes stored | 5PB | 50PB | object-store free-space < 30% |
| Uploads/s | 500 | 5k | upload-rest p99 > 100ms |
| Downloads/s | 5k | 50k | download-rest p99 > 200ms; CDN-miss > 30% |
| Sync delta/s | 100 | 1k | sync worker queue depth > 60s |
| Share-link mints/s | 1k | 10k | share-link-rest p99 > 100ms |
| Preview renders/s | 20 | 200 | preview worker queue > 60s |
| Virus scans/s | 50 | 500 | scan worker queue > 60s |

Scale-out policy:
- Kubernetes HPA: rest pods scale on CPU > 70%; min 3, max 100.
- Postgres: per-tenant logical shard; cross-cell replication-factor 3 with Patroni.
- Valkey: cluster mode; per-tenant key prefix; eviction policy `allkeys-lru` for upload-session + delta-sync cache.
- Object store: per-cell deployment (S3 / Garage / SeaweedFS); per-tenant prefix; replication-factor 3.
- Pre-warmed pool: 10 standby pods; cold-start ≤ 700ms.

Cross-region: M02 launches in KR (ap-seoul-1); M03 expands to EU + US per ADR-0117 jurisdiction pack.

Sharding: files partitioned by `(tenant_id, file_id_prefix_4)`; folders by `tenant_id`; sync sessions by `session_id`; share-links by `link_id`.

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Multipart resumable upload of 1GB completes within p99 ≤ 90s | `cargo bench` |
| AC-02 | S3 SigV4 conformance: `aws s3 cp`, `mc cp`, `s3cmd put` work end-to-end against the µservice S3-compat facade | `tests/e2e/s3-conformance.rs` |
| AC-03 | WebDAV RFC 4918 conformance: macOS Finder + Windows Explorer + davfs2 mount + Cyberduck end-to-end | `tests/e2e/webdav-clients.rs` |
| AC-04 | Delta-sync (FastCDC + LBFS) transfers only changed bytes; 100MB file with 1KB diff transfers ≤ 8KB on the wire | `cargo nextest -p oya-drive-sync-domain -- delta_minimum_bytes` |
| AC-05 | Share-link with password protection + expiry + view-cap honoured deterministically | `cargo nextest -p oya-drive-share-link-domain -- enforce_constraints` |
| AC-06 | Permissions inheritance + per-file override resolved correctly across 5-level folder depth | `cargo nextest -p oya-drive-permissions-domain -- inheritance_5levels` |
| AC-07 | Personal-context bytes NEVER appear in Professional-context list/search/preview | `cargo nextest -p oya-drive-file-store-domain -- context_isolation` |
| AC-08 | Tenant-DEK envelope encryption applied to Professional file content; verified at rest | `tests/e2e/encryption-at-rest.rs` |
| AC-09 | WORM (immutability) tier refuses purge even by tenant-root principal | `cargo nextest -p oya-drive-immutability-tier-domain -- worm_refuses_root` |
| AC-10 | Legal-hold preserves file + version history + share-link history past retention expiry | `cargo nextest -p oya-drive-immutability-tier-domain -- legal_hold` |
| AC-11 | Virus-scan blocks malicious upload from reaching durable bucket; EICAR test signature triggers quarantine | `cargo nextest -p oya-drive-dlp-virus-scan-domain -- eicar_quarantine` |
| AC-12 | DLP scan blocks share-out for flagged content; flagged content remains in tenant-only scope | `cargo nextest -p oya-drive-dlp-virus-scan-domain -- dlp_blocks_share` |
| AC-13 | Office preview render in gVisor sandbox refuses network egress + host filesystem access | `tests/e2e/preview-sandbox-isolation.rs` |
| AC-14 | Audit-chain seal emitted for every file lifecycle + share + permission + scan verdict + immutability transition | `cargo nextest -p oya-drive-file-store-app -- audit_chain_emission` |
| AC-15 | `oya gate validate per-microservice-layout --microservice drive` exit 0 | ADR-0131 lane |
| AC-16 | Cross-tenant "Shared with me" returns only files where explicit cross-tenant share grant exists; never raw enumeration | `cargo nextest -p oya-drive-share-link-domain -- cross_tenant_minimum_necessary` |
| AC-17 | E2E client-side encryption mode: bytes uploaded with libsodium secretstream remain unreadable server-side; preview falls back to "client-only" | `tests/e2e/e2e-personal-pillar.rs` |

## Open Questions

| # | Question | Owner | Target |
|---|---|---|---|
| 1 | Per-tenant object-store backend choice — should we offer Garage (edge-distributed) as a tenant tier option, or pin to S3-compat at the cell level? | council-architecture | resolved by ADR-DRIVE-0001 |
| 2 | Content-defined-chunking algorithm — FastCDC (chosen) vs Rabin fingerprint vs BuzHash; corpus benchmarking | axis-drive | resolved by ADR-DRIVE-0002 |
| 3 | Client-side E2E for Personal pillar — secretstream (libsodium) chosen; how do we handle full-text search of E2E files? | axis-drive + foundry-runtime | "E2E files indexed client-side only; server-side search returns metadata only"; subsequent-to-GA-tier-promotion refinement |
| 4 | Office preview sandbox isolation — gVisor (chosen) vs Firecracker vs Kata Containers | ops-security | resolved by ADR-DRIVE-0005 |
| 5 | Cross-pack file replication (e.g., KR tenant collaborating with EU tenant) — currently forbidden; revisit when sufficient SCC + tenant DPA tooling matures | council-privacy | subsequent-to-M04-completion |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum | layer authority |
| ADR-0106 | application→usecase | layer rename |
| ADR-0117 | Cloud-native infrastructure | data residency |
| ADR-0135 | Connect unbundle (parallel session) | dual-context inheritance + Strangler precedent |
| ADR-0139 | Agentic SLO-gated promotion | gate authority |
| ADR-0131 | Per-microservice flat layout | layout authority |
| ADR-0132 | Product-suite + bundle dissolution | µservice independence |
| ADR-0133 | Industry-best-practice conformance | hyperscaler-grade bar |
| ADR-0134 | Connect dissolution Strangler migration | migration policy (re-used for oya-connect-drive-domain → oya-drive-*) |
| ADR-0140 | Cedar policy enforcement | policy substrate |
| ADR-DRIVE-0001 | Object-storage substrate selection | per-cell backend choice |
| ADR-DRIVE-0002 | Content-defined-chunking + delta-sync | chunk algorithm |
| ADR-DRIVE-0003 | Share-link security model | TTL + KDF + view-cap |
| ADR-DRIVE-0004 | Encryption-at-rest + E2E | envelope + client-side |
| ADR-DRIVE-0005 | Preview pipeline sandboxing | gVisor isolation |
| ADR-DRIVE-0006 | Immutability + WORM policy | object-lock semantics |
| Bominal `workspace.drive` | Workspace Drive | inherited 1:1 |
| Bominal `connect.files` | Connect files-and-attachments | inherited 1:1 |

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `drive` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `drive` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 3 module pin(s) across 1 context(s).
- Scaling input: `per_request` with cell placement `Tier-3` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
