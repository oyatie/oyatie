---
audit_class: microservice_feature_parity
batch: wave-3-batch-3.2
microservice: drive
service_path: microservices/drive
audit_date: 2026-05-20
counterparts: [Google Drive, Dropbox, Microsoft OneDrive]
status: landed
---

# Drive Feature-Parity Matrix — 2026-05-20

## Header

Scope: `microservices/drive/`.

Counterpart 1: Google Drive.

Counterpart 2: Dropbox.

Counterpart 3: Microsoft OneDrive.

Parity goal: union-coverage across the top three counterpart surfaces with Oyatie-specific security, audit, and deployment doctrine.

Current local product anchor: `microservices/drive/PRD.md:20-28`.

Local counterpart anchor: `microservices/drive/PRD.md:281-283`.

Local wider matrix anchor: `microservices/drive/competitor-parity-matrix.md:18-24`.

Contract anchor: `microservices/drive/contracts/openapi/drive.yaml:64-542`.

Event anchor: `microservices/drive/contracts/asyncapi/drive-events.yaml:29-90`.

gRPC anchor: `microservices/drive/contracts/proto/drive.proto:99-380`.

Doctrine amendment: no feature-tier deltas are authored in this batch.

Tenant-class rule: `demo_trial`, `paid`, and `revenue_share` receive the same quality bar; only caps, billing, deployment context, and contractual envelope differ.

## §1 Counterpart-1 Capability Surface — Google Drive

1. Google Drive capability family: cloud file storage and workspace document collaboration.
2. Google Drive baseline product: user and shared-drive storage with folders, files, permissions, search, preview, comments, and Workspace-native editing.
3. Local drive parity anchor: the PRD names Google Drive Workspace Drive as the first competitor at `PRD.md:281`.
4. Local competitor matrix anchor: Google Drive is C1 at `competitor-parity-matrix.md:22`.
5. Storage capability: large file storage with resumable upload.
6. Oyatie evidence: single-shot create plus multipart upload endpoints at `contracts/openapi/drive.yaml:65-91` and `contracts/openapi/drive.yaml:199-264`.
7. Oyatie evidence: upload service is present in gRPC at `contracts/proto/drive.proto:161-198`.
8. Folder capability: folder hierarchy, move, metadata, list, and permissions.
9. Oyatie evidence: folder REST endpoints at `contracts/openapi/drive.yaml:167-199`.
10. Oyatie evidence: `FolderHierarchy` gRPC service at `contracts/proto/drive.proto:146-157`.
11. Permission capability: file and folder grants, revocation, inherited permissions, and link access controls.
12. Oyatie evidence: permission REST endpoints at `contracts/openapi/drive.yaml:425-449`.
13. Oyatie evidence: Cedar policy references in OpenAPI at `contracts/openapi/drive.yaml:19-21`.
14. Sharing capability: public share links, password protection, expiration, view-count caps, revocation, and audit.
15. Oyatie evidence: share-link endpoints at `contracts/openapi/drive.yaml:365-404`.
16. Oyatie evidence: share-link ADR chooses Ed25519, HKDF, Argon2id, TTL, view cap, and revocation cascade at `decisions/ADR-DRIVE-0003-share-link-security-model.md:62-100`.
17. Search capability: filename search, full-text search, OCR-assisted search, ranking, and tenant boundaries.
18. Oyatie evidence: search endpoint at `contracts/openapi/drive.yaml:454-457`.
19. Oyatie evidence: search SLO p99 target at `slos/search-latency.openslo.yaml:16-39`.
20. Preview capability: image, PDF, Office, and video previews.
21. Oyatie evidence: preview endpoint at `contracts/openapi/drive.yaml:477-488`.
22. Oyatie evidence: preview ADR chooses gVisor, libvips, qpdf, pdf.js, LibreOffice, and ffmpeg at `decisions/ADR-DRIVE-0005-preview-pipeline-sandboxing.md:54-84`.
23. Download capability: first-byte latency, range support, signed URLs, CDN integration.
24. Oyatie evidence: download endpoints at `contracts/openapi/drive.yaml:269-305`.
25. Oyatie evidence: download SLO p99 target at `slos/download-first-byte-latency.openslo.yaml:16-39`.
26. Sync capability: desktop and mobile sync, selective sync, change detection, conflict handling.
27. Oyatie evidence: sync endpoints at `contracts/openapi/drive.yaml:328-351`.
28. Oyatie evidence: FastCDC and LBFS-style manifest exchange decision at `decisions/ADR-DRIVE-0002-content-defined-chunking-and-delta-sync.md:57-74`.
29. Google parity pressure: Workspace-native co-authoring is not proven by drive-owned artifacts.
30. Google parity pressure: live collaborative editing is outside current drive contracts.
31. Google parity pressure: the drive path does not show client UX files for desktop, mobile, or web drive surfaces.
32. Google parity pressure: search and preview are specified, but public UI quality, ranking behavior, and OCR extraction UX are not demonstrated.
33. Oyatie advantage claim: per-tenant CMK and per-file DEK are explicit at `decisions/ADR-DRIVE-001-tenant-cmk-kek-dek-envelope-encryption.md:48-73`.
34. Oyatie advantage claim: WORM and legal-hold semantics are drive-owned rather than delegated to a generic object store.
35. Oyatie advantage claim: cross-tenant sharing is Cedar-gated and audit-chain-backed in product doctrine.
36. Google comparison result: product-surface parity is planned, but UX and collaboration proof remain incomplete.

## §2 Counterpart-2 Capability Surface — Dropbox

1. Dropbox capability family: high-quality sync, desktop ergonomics, file history, shared folders, selective sync, smart/on-demand sync, and simple external sharing.
2. Local drive parity anchor: the PRD names Dropbox + Business as the second competitor at `PRD.md:282`.
3. Local competitor matrix anchor: Dropbox is C2 at `competitor-parity-matrix.md:23`.
4. Sync capability: efficient changed-block sync and conflict resolution.
5. Oyatie evidence: sync endpoints at `contracts/openapi/drive.yaml:328-351`.
6. Oyatie evidence: sync gRPC service at `contracts/proto/drive.proto:230-263`.
7. Oyatie evidence: FastCDC decision cites LBFS lineage and content-defined chunking at `decisions/ADR-DRIVE-0002-content-defined-chunking-and-delta-sync.md:35-57`.
8. Oyatie evidence: sync SLO target over 100 changed files at `slos/sync-delta-latency.openslo.yaml:16-38`.
9. Selective-sync capability: choose folders or files to materialize locally.
10. Oyatie evidence: competitor matrix marks selective sync parity at `competitor-parity-matrix.md:75-78`.
11. Parity gap: no drive-owned desktop client implementation file proves selective-sync UI or local filesystem integration.
12. Smart/on-demand sync capability: hydrate files on access and keep local stubs.
13. Parity gap: local competitor matrix marks smart sync as a later roadmap item at `competitor-parity-matrix.md:75-78`.
14. Large-file upload capability: resumable sessions, chunk retry, checksums, and retry-safe completion.
15. Oyatie evidence: REST upload sessions and chunk endpoints at `contracts/openapi/drive.yaml:199-264`.
16. Oyatie evidence: upload SLO for 1GB multipart at `slos/upload-multipart-throughput.openslo.yaml:16-38`.
17. Shared-folder capability: membership, inherited permissions, external collaboration, audit trail.
18. Oyatie evidence: permissions REST endpoints at `contracts/openapi/drive.yaml:425-449`.
19. Oyatie evidence: share event catalog at `contracts/asyncapi/drive-events.yaml:45-52`.
20. File-history capability: versions, retention, restore, and audit.
21. Oyatie evidence: file versions endpoint at `contracts/openapi/drive.yaml:151-167`.
22. Oyatie evidence: lifecycle event channels at `contracts/asyncapi/drive-events.yaml:30-37`.
23. Deleted-file recovery capability: trash, retention, and permanent purge safeguards.
24. Oyatie evidence: soft-delete endpoint blocks legal-hold and WORM hard-delete at `contracts/openapi/drive.yaml:145`.
25. Compliance locking capability: Dropbox has business governance; Oyatie has stronger WORM object-lock ambition.
26. Oyatie evidence: immutability endpoints at `contracts/openapi/drive.yaml:514-542`.
27. Oyatie evidence: immutability SLO targets 100% correctness at `slos/immutability-tier-correctness.openslo.yaml:16-45`.
28. Dropbox parity pressure: the user-visible sync client is the product; backend primitives alone do not prove Dropbox-grade ergonomics.
29. Dropbox parity pressure: conflict resolution needs deterministic local UX, not only server-side sync APIs.
30. Dropbox parity pressure: offline file journal is represented by an IP, but executable code evidence is not in the drive path.
31. Oyatie advantage claim: content-defined chunking is explicit and not opaque.
32. Oyatie advantage claim: cross-tenant and regulated workflows are part of the same service model.
33. Oyatie advantage claim: DLP, virus scanning, WORM, and CMK controls are integrated into drive instead of being add-on products.
34. Dropbox comparison result: core sync design is strong; client proof and smart/on-demand sync remain the largest parity gaps.

## §3 Counterpart-3 Capability Surface — Microsoft OneDrive

1. OneDrive capability family: Microsoft 365 document storage, SharePoint-backed libraries, Office preview/edit, co-authoring, Files On-Demand, compliance integration, and enterprise identity.
2. Local drive parity anchor: the PRD names Microsoft 365 OneDrive as the third competitor at `PRD.md:283`.
3. Local competitor matrix anchor: OneDrive is C3 at `competitor-parity-matrix.md:24`.
4. Office preview capability: view Office files without download.
5. Oyatie evidence: preview REST endpoint at `contracts/openapi/drive.yaml:477-488`.
6. Oyatie evidence: preview ADR chooses LibreOffice in gVisor for Office rendering at `decisions/ADR-DRIVE-0005-preview-pipeline-sandboxing.md:68-84`.
7. Office co-authoring capability: concurrent editing inside native productivity apps.
8. Parity gap: no drive contract exposes real-time co-authoring or document-edit operational transforms.
9. SharePoint library capability: folder, site, and group-backed document libraries with enterprise retention and policy.
10. Oyatie evidence: folder, permission, WORM, audit, and pack-residency surfaces exist, but not SharePoint-equivalent site/group library semantics.
11. Files On-Demand capability: local stubs, hydration on access, and OS shell integration.
12. Parity gap: local smart/on-demand sync is later roadmap at `competitor-parity-matrix.md:75-78`.
13. Identity integration capability: Microsoft identity, groups, conditional access, and sensitivity labels.
14. Oyatie evidence: OpenAPI names OIDC, tenant API keys, mTLS, and Cedar authorization at `contracts/openapi/drive.yaml:19-21`.
15. Oyatie evidence: policy docs include tenant scope, auditor scope, data residency, dual-context isolation, and public-read policies in inventory rows 145-150.
16. Compliance capability: retention, audit, legal hold, DLP, and regulated records.
17. Oyatie evidence: compliance pack coverage appears in `compliance.md:18-48`.
18. Oyatie evidence: DPIA risk controls are listed at `dpia.md:124-168`.
19. DLP capability: detect sensitive content before egress or durable serve.
20. Oyatie evidence: DLP and virus-scan correctness SLO at `slos/dlp-scan-correctness.openslo.yaml:16-44`.
21. Oyatie evidence: DLP/virus verdict API at `contracts/openapi/drive.yaml:498-505`.
22. Migration capability: migrate from existing drive systems and preserve metadata, checksums, and permissions.
23. Oyatie evidence: Google Drive migration playbook exists at inventory row 137.
24. Oyatie evidence: generic connect migration exists at `migration-from-connect.md:15-33`.
25. OneDrive parity pressure: Office-native co-authoring is a separate product-level experience, not just preview.
26. OneDrive parity pressure: SharePoint library semantics and Microsoft 365 policy inheritance are deeper than a drive-only folder model.
27. OneDrive parity pressure: enterprise client deployment evidence is not in the drive path.
28. Oyatie advantage claim: tenant-CMK custody and cryptoshred are explicit at `decisions/ADR-DRIVE-001-tenant-cmk-kek-dek-envelope-encryption.md:50-73`.
29. Oyatie advantage claim: cross-context isolation is stronger than generic folder ownership where personal and professional spaces must be segregated.
30. OneDrive comparison result: backend compliance and storage features are strong; productivity-suite collaboration and client surfaces remain the main gaps.

## §4 UNION-Coverage Matrix

| ID | Capability | Google Drive | Dropbox | Microsoft OneDrive | Oyatie Drive Status | Evidence |
|---|---|---:|---:|---:|---|---|
| U-001 | User file storage | yes | yes | yes | planned parity | `PRD.md:20-28`; `contracts/openapi/drive.yaml:65-91` |
| U-002 | Folder hierarchy | yes | yes | yes | planned parity | `contracts/openapi/drive.yaml:167-199` |
| U-003 | File metadata | yes | yes | yes | planned parity | `contracts/proto/drive.proto:66-87` |
| U-004 | File version listing | yes | yes | yes | planned parity | `contracts/openapi/drive.yaml:151-167` |
| U-005 | Soft delete or trash | yes | yes | yes | planned parity | `contracts/openapi/drive.yaml:145` |
| U-006 | Hard-delete guardrails | partial | partial | yes | ahead on policy | `contracts/openapi/drive.yaml:145`; `slos/immutability-tier-correctness.openslo.yaml:16-45` |
| U-007 | Multipart upload | yes | yes | yes | planned parity | `contracts/openapi/drive.yaml:199-264` |
| U-008 | Resumable upload | yes | yes | yes | planned parity | `contracts/openapi/drive.yaml:199-264` |
| U-009 | tus-compatible upload | no public default | no public default | no public default | additive | `contracts/openapi/drive.yaml:45-46` |
| U-010 | S3-compatible upload facade | partial | no | no | additive | `PRD.md:34-35`; `slos/upload-multipart-throughput.openslo.yaml:16-19` |
| U-011 | Range download | yes | yes | yes | planned parity | `contracts/openapi/drive.yaml:269-281` |
| U-012 | Signed download URL | yes | yes | yes | planned parity | `contracts/openapi/drive.yaml:297-305` |
| U-013 | CDN-backed download | yes | yes | yes | planned parity | `iac/helm/values.yaml:74-80` |
| U-014 | Public share link | yes | yes | yes | planned parity | `contracts/openapi/drive.yaml:365-404` |
| U-015 | Password-protected share link | yes | yes | yes | planned parity | `decisions/ADR-DRIVE-0003-share-link-security-model.md:72-83` |
| U-016 | Expiring share link | yes | yes | yes | planned parity | `decisions/ADR-DRIVE-0003-share-link-security-model.md:83-90` |
| U-017 | View-count-limited share link | partial | partial | partial | planned parity | `decisions/ADR-DRIVE-0003-share-link-security-model.md:89-95` |
| U-018 | Share-link revocation cascade | yes | yes | yes | planned parity | `decisions/ADR-DRIVE-0003-share-link-security-model.md:95-100` |
| U-019 | Per-file permissions | yes | yes | yes | planned parity | `contracts/openapi/drive.yaml:425-449` |
| U-020 | Per-folder permissions | yes | yes | yes | planned parity | `contracts/openapi/drive.yaml:425-449` |
| U-021 | Cross-tenant sharing | partial | partial | partial | ahead if implemented | `competitor-parity-matrix.md:65-69` |
| U-022 | Audit-chain-backed share access | partial | partial | yes | planned parity-plus | `contracts/asyncapi/drive-events.yaml:45-52` |
| U-023 | Desktop sync | yes | yes | yes | gap in client proof | `competitor-parity-matrix.md:75` |
| U-024 | Mobile sync | yes | yes | yes | gap in client proof | `competitor-parity-matrix.md:76` |
| U-025 | Selective sync | yes | yes | yes | planned parity | `competitor-parity-matrix.md:77` |
| U-026 | Smart/on-demand sync | yes | yes | yes | gap | `competitor-parity-matrix.md:78` |
| U-027 | Delta sync | partial | yes | partial | planned parity-plus | `decisions/ADR-DRIVE-0002-content-defined-chunking-and-delta-sync.md:57-74` |
| U-028 | Conflict detection | yes | yes | yes | planned parity | `contracts/asyncapi/drive-events.yaml:59-64` |
| U-029 | Conflict deterministic tie-break | partial | yes | partial | planned parity-plus | `iac/helm/values.yaml:90-94` |
| U-030 | Full-text search | yes | yes | yes | planned parity | `contracts/openapi/drive.yaml:454-457` |
| U-031 | Search per tenant | yes | yes | yes | planned parity | `slos/search-latency.openslo.yaml:16-20` |
| U-032 | OCR-assisted search | yes | partial | yes | planned gap until OCR handoff is normalized | `IP-011-search-index.md:17-20` |
| U-033 | Image preview | yes | yes | yes | planned parity | `decisions/ADR-DRIVE-0005-preview-pipeline-sandboxing.md:68-84` |
| U-034 | PDF preview | yes | yes | yes | planned parity | `decisions/ADR-DRIVE-0005-preview-pipeline-sandboxing.md:68-84` |
| U-035 | Office preview | yes | partial | yes | planned parity | `decisions/ADR-DRIVE-0005-preview-pipeline-sandboxing.md:68-84` |
| U-036 | Video preview | yes | yes | yes | planned parity | `decisions/ADR-DRIVE-0005-preview-pipeline-sandboxing.md:68-84` |
| U-037 | Preview sandboxing | opaque | opaque | opaque | ahead if implemented | `decisions/ADR-DRIVE-0005-preview-pipeline-sandboxing.md:54-84` |
| U-038 | Virus scanning | yes | yes | yes | planned parity | `contracts/openapi/drive.yaml:498-505` |
| U-039 | DLP scanning | yes | partial | yes | planned parity | `slos/dlp-scan-correctness.openslo.yaml:16-44` |
| U-040 | Quotas | yes | yes | yes | planned parity | `contracts/asyncapi/drive-events.yaml:86-90` |
| U-041 | Tenant CMK | partial | partial | yes | planned parity-plus | `decisions/ADR-DRIVE-001-tenant-cmk-kek-dek-envelope-encryption.md:50-73` |
| U-042 | Per-file DEK | opaque | opaque | opaque | ahead if implemented | `decisions/ADR-DRIVE-001-tenant-cmk-kek-dek-envelope-encryption.md:50-60` |
| U-043 | Cryptoshred | partial | partial | yes | planned parity-plus | `decisions/ADR-DRIVE-001-tenant-cmk-kek-dek-envelope-encryption.md:62-70` |
| U-044 | WORM retention | partial | partial | yes | planned parity-plus | `contracts/openapi/drive.yaml:514-542` |
| U-045 | Legal hold | yes | yes | yes | planned parity | `contracts/proto/drive.proto:379-416` |
| U-046 | Compliance packs | yes | yes | yes | planned parity | `compliance.md:18-48` |
| U-047 | GDPR data subject controls | yes | yes | yes | planned parity | `dpia.md:111-132` |
| U-048 | Data residency | yes | yes | yes | planned parity | `multi-region.md:18-50` |
| U-049 | Cross-pack restore controls | partial | partial | partial | additive | `backfill-replay.md:36-42`; `backfill-replay.md:97-100` |
| U-050 | Tenant onboarding migration | yes | yes | yes | planned parity | `backfill-replay.md:20-28` |
| U-051 | Google Drive migration | source-specific | no | no | planned source migration | `migration-playbooks/from-google-drive.md` |
| U-052 | Legacy internal migration | no | no | no | additive | `migration-from-connect.md:15-33` |
| U-053 | API REST surface | yes | yes | yes | planned parity | `contracts/openapi/drive.yaml:1-21` |
| U-054 | gRPC surface | no default | no default | partial | additive | `contracts/proto/drive.proto:99-380` |
| U-055 | Async event catalog | partial | partial | partial | additive | `contracts/asyncapi/drive-events.yaml:29-90` |
| U-056 | WebDAV facade | yes | partial | yes | planned parity | `contracts/openapi/drive.yaml:19-20` |
| U-057 | S3 facade | no | no | no | additive | `contracts/openapi/drive.yaml:19-20` |
| U-058 | Fine-grained audit events | yes | yes | yes | planned parity | `contracts/asyncapi/drive-events.yaml:29-90` |
| U-059 | Rate-limit and circuit-breaker invariant | yes | yes | yes | planned parity | `manifest.json:353-358` |
| U-060 | Backfill and replay | partial | partial | partial | planned parity-plus | `backfill-replay.md:49-91` |
| U-061 | Cost guardrails | opaque | opaque | opaque | additive | `cost-budget.md:60-67` |
| U-062 | Runbooks | yes | yes | yes | planned parity | inventory rows 152-158 |
| U-063 | Incident response | yes | yes | yes | planned parity | `incident-response.md:18-121` |
| U-064 | DPIA | yes | yes | yes | planned parity | `dpia.md:40-168` |
| U-065 | Multi-region residency | yes | yes | yes | planned parity | `multi-region.md:18-50` |
| U-066 | Object-store backend portability | no | no | no | additive | `decisions/ADR-DRIVE-0001-object-storage-substrate-selection.md` |
| U-067 | Garage backend | no | no | no | additive | `iac/helm/Chart.yaml:21-25` |
| U-068 | SeaweedFS backend | no | no | no | additive | `iac/helm/Chart.yaml:30-33` |
| U-069 | Postgres metadata model | opaque | opaque | opaque | planned | `cost-budget.md:28-30` |
| U-070 | Meilisearch/Tika extraction | no public default | no public default | no public default | planned | `iac/helm/Chart.yaml:41-47` |
| U-071 | Preview egress deny policy | opaque | opaque | opaque | additive | `IP-001-iac-bootstrap.md:36` |
| U-072 | gVisor RuntimeClass for preview | opaque | opaque | opaque | additive | `IP-001-iac-bootstrap.md:32`; `IP-001-iac-bootstrap.md:67` |
| U-073 | Tenant-class caps | no | no | no | missing | `policy/ci-scope.cedar:20`; `policy/ci-scope.cedar:47-48` |
| U-074 | OCI Always Free profile | no | no | no | missing evidence | `coherence-audit-2026-05-20.md:331` |
| U-075 | OpenTofu six-context deployability | no | no | no | missing evidence | `coherence-audit-2026-05-20.md:435-463` |

## §5 Family Summary

### §5.1 Storage And Metadata

1. Union capabilities: files, folders, versions, trash, metadata, content addressing, legal delete blocks.
2. Oyatie evidence is strong in contracts and PRD.
3. The REST contract covers create, list, get, update, trash, and versions.
4. The gRPC contract covers equivalent file and folder services.
5. The PRD includes content-addressing and storage backend portability.
6. The architecture and ADRs bind storage to tenant custody and object-store choice.
7. Gap: no executable Rust implementation files are under the drive path.
8. Gap: no root README maps the storage model for implementers.

### §5.2 Upload And Download

1. Union capabilities: resumable upload, multipart upload, chunk retry, signed download URL, range download, first-byte latency budget.
2. Oyatie evidence is strong in REST contracts and SLOs.
3. Upload SLO gives a 1GB multipart p99 target.
4. Download SLO gives a warm-cache p99 target and references a cold-cache separate budget.
5. Helm values set 5 TiB maximum multipart file size and FastCDC chunk defaults.
6. Gap: no context-specific network and egress budget appears for each deployable context.
7. Gap: no tenant-class usage cap maps upload/download volume to `demo_trial`, `paid`, or `revenue_share`.

### §5.3 Sync

1. Union capabilities: delta sync, selective sync, smart/on-demand sync, conflict handling, desktop and mobile clients.
2. Oyatie evidence is strong for server-side delta protocol.
3. FastCDC plus LBFS-style manifest exchange is a credible differentiator.
4. Sync SLO gives a p99 budget over a 100-changed-file workload.
5. AsyncAPI includes conflict detection events.
6. Gap: smart/on-demand sync is not current parity evidence.
7. Gap: client UX and filesystem integration evidence are missing.

### §5.4 Sharing And Permissions

1. Union capabilities: user grants, folder grants, public links, password links, expiring links, revocation, view caps, audit, external collaboration.
2. Oyatie evidence is strong in REST, gRPC, AsyncAPI, Cedar references, and share-link ADR.
3. Ed25519 plus HKDF and Argon2id is a detailed security design.
4. Revocation cascade is explicit.
5. Cross-tenant share with audit is a differentiator if implemented.
6. Gap: external collaboration UX and invitation lifecycle evidence are not present.
7. Gap: tenant-class commercial constraints for share-link volume and public egress are not defined.

### §5.5 Search And Preview

1. Union capabilities: filename search, full-text search, OCR, image preview, PDF preview, Office preview, video preview, preview isolation.
2. Oyatie evidence is strong for search and preview APIs.
3. Oyatie evidence is strong for sandbox design in the preview ADR.
4. Oyatie evidence is strong for OpenSLO latency targets.
5. Gap: OCR dependency name drifts between IP and manifest.
6. Gap: Office co-authoring is not part of the drive-owned contract.
7. Gap: preview quality benchmarks are internal and should be refreshed against current counterpart behavior.

### §5.6 Security, Compliance, And Custody

1. Union capabilities: encryption at rest, tenant key control, DLP, virus scan, audit, legal hold, WORM, retention, residency, incident response, DPIA.
2. Oyatie evidence is strong and product-specific.
3. Envelope encryption with CMK, KEK, and DEK is explicit.
4. DLP and virus-scan correctness is zero-tolerance.
5. WORM and legal hold are first-class surfaces.
6. Multi-region data-residency docs are explicit.
7. Gap: DPIA sign-off remains pending.
8. Gap: current tenant-class adoption is missing.

### §5.7 APIs, SDKs, And Migration

1. Union capabilities: REST API, developer SDK, migration tooling, event stream, import/export, admin controls.
2. Oyatie evidence is strong for REST, gRPC, and events.
3. SDK plan exists in inventory.
4. Rust SDK reference implementation exists in inventory.
5. Google Drive migration playbook exists in inventory.
6. Connect-to-drive migration exists and is specific.
7. Gap: there is no generated SDK provenance under the drive path.
8. Gap: public API compatibility constraints are not tied to tenant classes.

## §6 Headline Gap Analysis

1. Gap H-001: OpenTofu six-context deployability is missing from the drive path.
2. Evidence: `IP-001-iac-bootstrap.md:16-24` and `IP-001-iac-bootstrap.md:30-47` show Helm/Kustomize scope.
3. Impact: all counterpart parity claims remain product-planning claims until deployment posture catches up to canonical direction.
4. Fix: add context-specific OpenTofu modules and a manifest binding them to the drive deployable.
5. Gap H-002: tenant-class behavior is not expressed.
6. Evidence: `policy/ci-scope.cedar:20` uses `production`, while `policy/ci-scope.cedar:47-48` uses `synthetic` and `dev`.
7. Impact: the current docs cannot explain how `demo_trial`, `paid`, and `revenue_share` differ in caps, billing, context availability, and contractual SLO.
8. Fix: add tenant-class control rows to manifest, quota, cost, and SLO overlay docs.
9. Gap H-003: smart/on-demand sync is not current parity evidence.
10. Evidence: `competitor-parity-matrix.md:75-78`.
11. Impact: Dropbox and OneDrive remain ahead in visible sync ergonomics until client proof exists.
12. Fix: add client requirements, local-stub semantics, hydration API, conflict UI, and OS-specific test coverage.
13. Gap H-004: Office co-authoring is not represented.
14. Evidence: preview exists at `contracts/openapi/drive.yaml:477-488`, but no co-authoring contract appears in the drive API surface.
15. Impact: OneDrive remains ahead for productivity-suite collaboration.
16. Fix: explicitly mark co-authoring as external suite integration, out of scope, or future drive-owned capability.
17. Gap H-005: root README is absent.
18. Evidence: complete inventory in `coherence-audit-2026-05-20.md:§2.2`.
19. Impact: a new owner must reconstruct the service map from many files.
20. Fix: add a concise canonical index, or make manifest the machine entry point and point humans there.
21. Gap H-006: executable implementation evidence is absent under drive.
22. Evidence: inventory has no `src/` or `tests/` rows.
23. Impact: parity is currently documentation/contract maturity, not implementation maturity.
24. Fix: link generated crates, add manifest pointers, or land Rust source/test directories under the µservice.
25. Gap H-007: OCR handoff naming drifts.
26. Evidence: `IP-011-search-index.md:17-20`; `manifest.json:473-492`.
27. Impact: search parity depends on a downstream capability whose owner name is ambiguous.
28. Fix: normalize the dependency and event/API contract.
29. Gap H-008: old commercial-tier language remains in related docs.
30. Evidence: `coherence-audit-2026-05-20.md:§3.4.T`.
31. Impact: feature parity and performance docs can be misread as stratified quality.
32. Fix: retire old references and rewrite to tenant-class overlays.

## §7 Additive Oyatie Surface

1. Additive A-001: provider-neutral object-store backend strategy.
2. Evidence: ADR-DRIVE-0001 and Helm dependencies for Garage, MinIO, and SeaweedFS at `iac/helm/Chart.yaml:21-33`.
3. Competitive meaning: counterpart products are vertically integrated; Oyatie needs portability across six contexts.
4. Additive A-002: per-tenant CMK, rolling KEK, and per-file DEK envelope model.
5. Evidence: `decisions/ADR-DRIVE-001-tenant-cmk-kek-dek-envelope-encryption.md:50-73`.
6. Competitive meaning: stronger tenant custody story if implemented and tested.
7. Additive A-003: cross-tenant share with Cedar-gated audit-chain.
8. Evidence: `competitor-parity-matrix.md:65-69`.
9. Competitive meaning: supports professional-to-personal and inter-tenant workflows with policy evidence.
10. Additive A-004: WORM object-lock in a drive-class product.
11. Evidence: `contracts/openapi/drive.yaml:514-542`; `slos/immutability-tier-correctness.openslo.yaml:16-45`.
12. Competitive meaning: blends collaboration drive UX with regulated records behavior.
13. Additive A-005: explicit DLP and virus-scan correctness before durable serve.
14. Evidence: `slos/dlp-scan-correctness.openslo.yaml:16-44`.
15. Competitive meaning: avoids treating scan as a best-effort background add-on.
16. Additive A-006: FastCDC and LBFS-style delta sync as documented protocol.
17. Evidence: `decisions/ADR-DRIVE-0002-content-defined-chunking-and-delta-sync.md:57-74`.
18. Competitive meaning: aims to match Dropbox-style sync efficiency with transparent implementation.
19. Additive A-007: preview rendering in a hardened sandbox.
20. Evidence: `decisions/ADR-DRIVE-0005-preview-pipeline-sandboxing.md:54-84`.
21. Competitive meaning: converts preview from a convenience feature into an isolation-controlled ingestion boundary.
22. Additive A-008: cross-pack restore and residency-aware backfill.
23. Evidence: `backfill-replay.md:36-42`; `backfill-replay.md:97-100`.
24. Competitive meaning: supports global regulated tenants without silently crossing residency boundaries.
25. Additive A-009: event catalog for lifecycle, share, permission, sync, scan, legal hold, and quota events.
26. Evidence: `contracts/asyncapi/drive-events.yaml:29-90`.
27. Competitive meaning: enables workflow, audit, billing, and compliance subscribers without scraping storage logs.
28. Additive A-010: cost guardrails as part of service ownership.
29. Evidence: `cost-budget.md:60-67`.
30. Competitive meaning: supports usage-based and revenue-share economics once tenant-class semantics are added.

## §8 Readiness Verdict

1. Product-surface coverage: strong.
2. Contract-surface coverage: strong.
3. Operational-doc coverage: strong.
4. Compliance-doc coverage: strong.
5. Counterpart union breadth: broad enough to be credible.
6. Evidence maturity: mixed because implementation code and context IaC are absent under the drive path.
7. Google Drive parity: partial; storage, search, preview, and sharing are planned, while live collaboration and UX proof are missing.
8. Dropbox parity: partial; sync design is credible, while smart/on-demand sync and client proof are missing.
9. OneDrive parity: partial; Office preview and compliance are planned, while co-authoring and productivity-suite integration are missing.
10. Additive strategy: credible if OpenTofu, tenant-class controls, and executable Rust implementation evidence are landed.
11. Immediate action: close the two P1 deployability gaps before treating the drive docs as production-ready.
12. Secondary action: retire old commercial-tier language and adopt tenant-class overlays in quota, cost, SLO, onboarding, FAQ, migration, tutorial, and benchmark docs.
13. Stop condition for parity claim: all union rows either have executable evidence, a consciously scoped out decision, or a dated roadmap entry with owner and acceptance test.

## §9 Acceptance Coverage Addendum

1. A parity claim for Google Drive requires measured evidence for search relevance, preview fidelity, large-file upload behavior, and Workspace-like sharing workflows.
2. A parity claim for Dropbox requires measured evidence for sync latency, changed-byte savings, conflict resolution, selective sync, and smart/on-demand sync client behavior.
3. A parity claim for Microsoft OneDrive requires measured evidence for Office preview fidelity, path-limit handling, sync item scale, compliance holds, and library-style permission scale.
4. Drive has backend contract coverage for file, folder, upload, download, sync, share, permission, search, preview, scan, and immutability.
5. Drive has event coverage for lifecycle, access, share, permission, sync, scan, immutability, legal hold, and quota.
6. Drive has SLO coverage for metadata list, upload, download, search, preview, sync, share-link generation, scan correctness, and immutability correctness.
7. Drive does not yet have client test coverage for desktop sync.
8. Drive does not yet have client test coverage for mobile sync.
9. Drive does not yet have browser UI test coverage for file browsing, sharing, search, preview, or migration.
10. Drive does not yet have OpenTofu deployability evidence for all six canonical contexts.
11. Drive does not yet have a tenant-class caps matrix.
12. Drive does not yet have a path/name limits matrix comparable to Dropbox and OneDrive.
13. Drive does not yet have Office co-authoring scope resolution.
14. Drive does not yet have OCR handoff naming resolved between IP and manifest.
15. Acceptance pass condition: every top-three counterpart family has one direct contract, one SLO or benchmark, one operational runbook, and one client or API test reference.
16. Current acceptance state: backend and operations are strong; deployment, client, and tenant-class control surfaces need revision.
17. This addendum does not expand scope beyond the three-counterpart union; it clarifies what evidence would convert planned parity into proven parity.
