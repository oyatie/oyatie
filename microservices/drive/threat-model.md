---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: drive
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-drive + ops-security
deciders: council-architecture, ops-security, axis-drive, council-privacy
methodology: STRIDE + LINDDUN + OWASP Top 10 (2021) + OWASP API Top 10 (2023) + OWASP ASVS v4.0.3 + NIST SP 800-154
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0126, ADR-0130, ADR-0131, ADR-0132, ADR-0140, ADR-DRIVE-0001, ADR-DRIVE-0002, ADR-DRIVE-0003, ADR-DRIVE-0004, ADR-DRIVE-0005, ADR-DRIVE-0006]
review_cadence: quarterly + on every BC architectural change
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1-CC6.8, CC7.1-CC7.5, CC8.1"
  - "ISO 27001:2022: A.5.7-A.5.34, A.8.2-A.8.34"
  - "GDPR Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 44"
  - "OWASP ASVS v4.0.3"
  - "CIS Kubernetes Benchmark v1.9.0"
  - "FIPS 140-3 (for KMS / OpenBao)"
  - "NIST SP 800-57 (key management)"
  - "SLSA L3 (supply chain)"
  - "NIST SSDF (PO + PS + PW + RV)"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 15/17/18/22-2/23/24/25/28/29/29-2", "KR-ISMS-P §2.1-2.12", "KR 전자문서법 Arts. 5/6/7", "KR-FSS supervisory regulations (5y retention for financial-sector)"]
  pack-us: ["SEC 17a-4(f)", "FINRA Rule 4511", "CCPA / CPRA"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308/§164.310/§164.312/§164.314/§164.316/§164.502"]
  pack-eu: ["GDPR Arts. 25 + 32 + 35 + 44-50", "eIDAS 910/2014", "NIS2 2022/2555", "EU AI Act Regulation 2024/1689"]
  pack-jp: ["APPI Arts. 17/18/20/21/23/24/26-2/27"]
doc_status: published
---

# Threat Model: drive µservice

## Purpose

Identify, classify, and mitigate threats to the drive µservice's confidentiality, integrity, availability, and privacy posture. Drive carries dual-context bytes (Personal + Professional files), shared-link surfaces, third-party-app OAuth grants, and immutable WORM tier. A compromise here cascades into mass data exfiltration, regulatory breach (HIPAA / GDPR / SEC 17a-4), and tenant trust loss. This document is the canonical security artifact reviewed by SOC 2 / ISO 27001 / GDPR DPAs.

## Scope

### In-scope

All components introduced for the drive µservice across the eleven bounded contexts, deployed in the tenant workload cluster:

| Layer-A (adopted OSS) | Layer-B (oyatie-owned) |
|---|---|
| Garage 1.0.x (S3-compatible edge-distributed object store; primary) | `oya-drive-file-store-*` (13 crates) |
| MinIO RELEASE.2024-08 (S3-compatible single-cluster; secondary) | `oya-drive-folder-hierarchy-*` (8 crates) |
| SeaweedFS 3.x (archive tier) | `oya-drive-upload-*` (10 crates) |
| Postgres 16 LTS (metadata) | `oya-drive-download-*` (8 crates) |
| Redis 7.4 LTS (upload-session + delta-sync cache) | `oya-drive-sync-*` (10 crates) |
| Meilisearch 1.10.x (full-text index) | `oya-drive-share-link-*` (10 crates) |
| Apache Tika 2.9.x (content extraction) | `oya-drive-permissions-*` (8 crates) |
| ClamAV 1.4.x (virus scan) | `oya-drive-search-index-*` (10 crates) |
| OPSWAT MetaDefender (multi-engine scan; healthcare + EU packs) | `oya-drive-preview-*` (12 crates) |
| libvips 8.15.x (image preview) | `oya-drive-dlp-virus-scan-*` (9 crates) |
| qpdf 11.x + Mozilla pdf.js 4.x (PDF preview) | `oya-drive-immutability-tier-*` (8 crates) |
| LibreOffice 24.x in gVisor (Office preview) | |
| ffmpeg 7.x (video preview) | |
| OpenBao Transit (envelope encryption KMS) | |
| libsodium 1.0.20 (client-side E2E secretstream) | |
| Argon2id (RFC 9106) (password-protected share-link KDF) | |
| Ed25519 (share-link + audit-chain signature) | |
| gVisor 2026-04 runtime (Office preview sandbox) | |

### Out-of-scope

- Underlying Kubernetes / IaaS layer (owned by `cloud-k8s`).
- Mail delivery / attachment-bridge transport (owned by `mail` µservice).
- Tenancy / identity (owned by `tenancy` µservice).
- Audit-chain seal infrastructure (owned by `audit-chain` µservice).
- Observability collectors (owned by `observability` µservice).
- foundry-runtime ML (OCR, auto-tag, smart-search inference) — drive only consumes via Workflow.

## Trust Boundaries

```text
┌─ Internet ─────────────────────────────────────────────────────────────────────────────┐
│                                                                                        │
│  Tenant ops      Customer SDK     S3/WebDAV/tus clients   Share-link viewers (signed) │
│         │                │                │                          │                 │
│         │ (OIDC+MFA)     │ (API key)      │ (SigV4 / Basic+TLS)      │ (HMAC link)    │
│         ▼                ▼                ▼                          ▼                 │
│  ┌─ Public ingress (Envoy + WAF + DDoS + rate limit) ─────────────────────────────┐    │
│  └────────────────────────────────────────────────────────────────────────────────┘    │
│                                            │                                           │
└────────────────────────────────────────────│───────────────────────────────────────────┘
                                             ▼
┌─ Tenant workload cluster ──────────────────────────────────────────────────────────────┐
│                                                                                        │
│  Trust boundary 1: External → REST + S3-API + WebDAV + tus ingress                     │
│  ┌─ drive-file-store-rest ──┐ ┌─ drive-upload-rest ──┐ ┌─ drive-download-rest ───┐     │
│  │ OIDC + RLS + Cedar       │ │ multipart staging    │ │ signed URL + range     │     │
│  └──────────────────────────┘ └──────────────────────┘ └────────────────────────┘     │
│  ┌─ drive-share-link-rest ──┐ ┌─ drive-permissions-rest ┐ ┌─ drive-search-rest ──┐    │
│  │ Argon2id KDF + signed    │ │ Cedar policy gate       │ │ tenant-scoped Meili │    │
│  └──────────────────────────┘ └─────────────────────────┘ └─────────────────────┘    │
│                                                                                        │
│  Trust boundary 2: REST → Postgres (per-tenant RLS + tenant-DEK)                       │
│                                                                                        │
│  Trust boundary 3: REST → Redis (upload-session + delta-sync cache, per-tenant prefix) │
│                                                                                        │
│  Trust boundary 4: REST → Object store (Garage / MinIO / SeaweedFS; per-tenant prefix) │
│                                                                                        │
│  Trust boundary 5: REST → Meilisearch (per-tenant index; cross-tenant query refused)   │
│                                                                                        │
│  Trust boundary 6: Worker → ClamAV / OPSWAT (scan; quarantine bucket isolation)        │
│                                                                                        │
│  Trust boundary 7: Worker → Preview-renderer (gVisor sandbox; no net + no host FS)     │
│                                                                                        │
│  Trust boundary 8: Cross-tenant share-link → external viewer (over signed URL only;    │
│       Cedar `cross-tenant-share-grant` policy)                                         │
│                                                                                        │
│  Trust boundary 9: drive ↔ mail µservice (Workflow event; attachment-bridge minting)   │
│                                                                                        │
│  Trust boundary 10: drive ↔ messenger µservice (Workflow event; file-share embed)      │
│                                                                                        │
│  Trust boundary 11: drive ↔ foundry-runtime (Workflow event; OCR/auto-tag handoff)     │
│                                                                                        │
│  Trust boundary 12: Workers (retention sweep + version pruner + WORM scan) → DB + obj  │
│       (SPIFFE-identity bound; not user-callable)                                       │
│                                                                                        │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

Twelve trust boundaries.

## Assets & Data Classification

Per Bominal ADR-0028 + `oya-check-data-class` LEAN lane.

| Asset | Class | Sensitivity | Retention | Authoritative store |
|---|---|---|---|---|
| File bytes (Professional context) | `PROFESSIONAL_FILE_CONTENT` (tenant-DEK envelope encrypted) | Critical | per jurisdiction + legal hold | object store + Postgres metadata |
| File bytes (Personal context) | `PERSONAL_FILE_CONTENT` (optional client-side E2E) | Critical | per jurisdiction + legal hold | object store + Postgres metadata |
| File metadata (name, size, mime, parent_folder_id, version chain) | `BEHAVIORAL_TENANT_PRODUCT` | High | per file retention | Postgres |
| Folder hierarchy | `BEHAVIORAL_TENANT_PRODUCT` | Medium | per tenant retention | Postgres |
| Permission ACLs | `AUDIT` + `BEHAVIORAL_TENANT_PRODUCT` | High | append-only | Postgres + audit-chain |
| Share-link records (link_id, signed-blob, ttl, view-cap) | `SECRET` (signing key) + `PII_QUASI_IDENTIFIER` (link bound to recipient identity in cross-tenant case) | Critical | per-link retention | Postgres + audit-chain |
| Upload session state (in-flight chunks) | `PERSONAL_FILE_CONTENT` / `PROFESSIONAL_FILE_CONTENT` (per source) | Critical | transient (≤ 24h) | Redis + object store staging |
| Sync session + chunk manifest | `INTERNAL_ONLY` (chunk hashes only; never plaintext bytes) | Medium | per-session ≤ 7d | Postgres + Redis |
| Search index (full-text + filename) | `PERSONAL_FILE_CONTENT` / `PROFESSIONAL_FILE_CONTENT` (per-tenant index) | Critical | per file retention | Meilisearch + Tika |
| Preview artifacts (thumbnail / first-page render) | inherits from source file | High | LRU cache; ≤ 30d | object store cache bucket |
| Virus scan verdict + quarantine record | `AUDIT` | Critical | append-only ≥ 7y | Postgres + audit-chain |
| DLP scan verdict | `AUDIT` | High | append-only per pack | Postgres + audit-chain |
| Tenant-DEK | `SECRET` | Critical | OpenBao 90d rotation | OpenBao |
| Share-link signing key (per-tenant or per-link Ed25519) | `SECRET` | Critical | OpenBao 30d rotation | OpenBao |
| Per-tenant API key | `SECRET` | Critical | OpenBao 30d rotation | OpenBao |
| Audit-chain seal records | `AUDIT` | High | append-only | audit-chain µservice |
| Immutability records + retention floor | `AUDIT` | Critical | append-only; preserved past retention | Postgres + audit-chain |
| Legal-hold records | `AUDIT` | Critical | append-only; preserved past retention | Postgres + audit-chain |

## Actors

| Actor | Trust | Auth | Capability |
|---|---|---|---|
| Tenant operator (human) | Untrusted external | OIDC + MFA | RW own tenant's files / folders / shares |
| Customer SDK / app (machine) | Untrusted external | per-tenant API key (30d rotation) | RW own tenant via SDK / REST |
| S3-API client (`aws s3`, `mc`, `s3cmd`) | Untrusted external | SigV4 + tenant-bound access-key | RW own tenant's bucket-shaped namespace |
| WebDAV client (Finder, Explorer, davfs2, Cyberduck) | Untrusted external | Basic Auth over TLS + per-tenant credential | RW own tenant via RFC 4918 |
| tus 1.0 client | Untrusted external | per-tenant API key + OIDC token | upload-resumable only |
| Sync client (desktop / mobile) | Untrusted external | per-device OIDC + refresh-token (30d) | bi-directional sync of opted-in folders |
| Share-link viewer (external) | Untrusted external | signed link (HMAC + Argon2id when password-protected) | read-only on a specific file/folder; subject to view-cap + TTL + revocation |
| Remote tenant (cross-tenant share invite) | Semi-trusted | mTLS internal mesh + Cedar grant | access to explicitly-shared files via signed link |
| Workflow µservice | Trusted internal | mTLS + SPIFFE | trigger file-change automation |
| Mail µservice | Trusted internal | mTLS + SPIFFE | attachment-bridge minting |
| Messenger µservice | Trusted internal | mTLS + SPIFFE | file-share embed minting |
| Foundry-runtime µservice | Trusted internal | mTLS + SPIFFE | OCR / auto-tag / smart-search inference |
| Tenancy µservice | Trusted internal | mTLS + SPIFFE | identity resolution |
| Audit-chain µservice | Trusted internal | mTLS + SPIFFE | seal emission |
| Worker (retention sweep / version pruner / WORM scan / scan worker / preview worker) | Trusted internal | SPIFFE + OpenBao SA token | RW on file metadata + object store; scan-quarantine |
| Council-architecture / ops-security | Trusted internal | OIDC + MFA + JIT | admin-level access |
| External auditor (SOC 2 / ISO 27001 / SEC 17a-4) | Read-only time-boxed | OIDC + MFA + JIT ≤ 4h | read-only |
| Attacker (opportunistic / targeted) | Untrusted | none | — |
| Insider (accidental / malicious) | Trusted internal | OIDC + MFA | mitigated via PR review + LEAN gates + audit-chain |

## STRIDE Threat Catalog

Each threat: ID; asset; description; likelihood (L/M/H); impact (L/M/H); risk; mitigations; owner; residual; framework controls.

### Spoofing

**T-S-01 — Attacker forges S3 SigV4 signature using stolen access-key**
- Asset: S3-API REST
- L M / I H / Risk H
- Mitigations:
  - Per-tenant access-key bound to `(tenant_id, principal_id, device_id)`; rotation 30d; revocation on suspicion.
  - SigV4 signature validated against canonicalised request per AWS spec; replay protection via `X-Amz-Date` + 15-minute window.
  - Anomaly detection on per-key access patterns; suspicious patterns trigger forced re-auth.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2; ISO 27001 A.5.15, A.5.17, A.8.5; OWASP API1:2023 (Broken Object Level Authorization)

**T-S-02 — Attacker forges share-link signature**
- Asset: share-link signed-blob
- L L / I H / Risk M
- Mitigations:
  - Ed25519 signature over `(link_id, file_id, expires_at, view_cap_remaining, scope)`; signing key per-tenant HKDF-derived from OpenBao master.
  - Password-protected links: Argon2id (memory_cost=64MiB, time_cost=3, parallelism=4) per RFC 9106.
  - Link revocation cascade: invalidates in-memory + emits revocation event (Workflow); subsequent access returns 410 Gone.
- Owner: axis-drive + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2; ISO 27001 A.8.5, A.8.24; OWASP API2:2023 (Broken Authentication)

**T-S-03 — WebDAV / S3 client impersonates another tenant via stolen credential**
- Asset: WebDAV + S3 ingress
- L M / I H / Risk H
- Mitigations:
  - Per-tenant credential bound; rotation 30d.
  - Tenant claim non-modifiable (server-side mapping; not client header).
  - Rate-limit + anomaly detection.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.5.17

**T-S-04 — Sync client refresh-token theft**
- Asset: sync-client OIDC refresh-token
- L M / I H / Risk H
- Mitigations:
  - DPoP-bound refresh token per OAuth 2.1; revocation on device-removal.
  - Per-device fingerprint binding; mismatch triggers forced re-auth.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.17; OWASP ASVS v4.0.3 §3.4

### Tampering

**T-T-01 — Upload chunk tampering during multipart**
- Asset: multipart upload staging bucket
- L M / I H / Risk H
- Mitigations:
  - Per-chunk SHA-256 checksum computed client-side + server-side; mismatch refused.
  - SigV4 signs chunk content hash; server validates.
  - Staging bucket isolated from durable bucket; promotion happens only after virus scan + content-address derivation + manifest validation.
- Owner: axis-drive + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.7, CC7.1; ISO 27001 A.8.24, A.8.25, A.8.28; GDPR Art. 32(1)(b)

**T-T-02 — Tenant-DEK substitution on file read (downgrade attack)**
- Asset: tenant-DEK envelope encryption
- L L / I H / Risk M
- Mitigations:
  - Envelope encryption per Bominal ADR-0111; ciphertext records carry binding to DEK ID + signed integrity check.
  - DEK rotation event re-encrypts; old DEKs maintained for read-only past-record decryption only.
  - LEAN check `oya-check-dek-binding-integrity` validates ciphertext binding.
- Owner: ops-security + cloud-secrets
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.8.24, A.8.25; GDPR Art. 32(1)(a); FIPS 140-3

**T-T-03 — WORM (immutability) bypass via raw DB write**
- Asset: WORM immutability tier
- L L / I H / Risk M
- Mitigations:
  - Postgres role for application has no UPDATE / DELETE permission on `immutability_record` table.
  - Object-store backend enforces object-lock at the storage layer (Garage `bucket.object-lock=COMPLIANCE`; MinIO `mc retention set compliance`; S3 Object Lock compliance mode).
  - Periodic integrity scan: compare hold-set vs storage layer; mismatch alerts.
  - Even tenant-root cannot release WORM before retention floor expires.
- Owner: ops-security + compliance + axis-drive
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.27, A.8.4; SEC 17a-4(f); FINRA 4511; HIPAA §164.316

**T-T-04 — Audit-chain seal omission for file mutation**
- Asset: audit emission
- L L / I H / Risk M
- Mitigations:
  - Every file write path emits via `audit-chain` µservice port; LEAN check `oya-check-audit-emission-coverage` refuses build if any usecase mutating files skips emission.
  - Audit-chain µservice acks emission; missing acks trigger `held` SLO state via observability.
- Owner: audit-chain + axis-drive
- Residual: L
- Frameworks: SOC 2 CC4.1, CC7.2, CC8.1; ISO 27001 A.5.28, A.8.15; GDPR Art. 5(2), Art. 30

**T-T-05 — Content-defined-chunking boundary drift causes incorrect dedup**
- Asset: chunk manifest
- L L / I M / Risk L
- Mitigations:
  - FastCDC parameters fixed at `(MIN=4KiB, AVG=8KiB, MAX=16KiB, gear-table v1)`.
  - LEAN check `oya-check-cdc-parameters-pinned` refuses parameter drift.
  - Corpus tests on rolling-hash boundaries.
- Owner: axis-drive
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.8.32

### Repudiation

**T-R-01 — Tenant operator denies sharing a file**
- Asset: share-link issuance chain
- L L / I M / Risk L-M
- Mitigations:
  - Every share-link mint carries actor SPIFFE-identity + Ed25519 audit-chain seal.
  - Share-link history retained for 7y minimum.
- Owner: axis-drive + audit-chain
- Residual: L
- Frameworks: SOC 2 CC4.1; ISO 27001 A.5.28, A.8.15

**T-R-02 — User disputes download/access ("I never downloaded that file")**
- Asset: download audit record
- L M / I M / Risk M
- Mitigations:
  - Every download path emits audit record with download-ticket + recorder IP hash + UA hash.
  - Audit chain Ed25519-sealed; replayable.
- Owner: axis-drive
- Residual: L
- Frameworks: SOC 2 CC4.1; ISO 27001 A.8.15

### Information Disclosure

**T-I-01 — Share-link enumeration via brute-force**
- Asset: share-link space
- L H / I H / Risk H
- Mitigations:
  - Link IDs are 256-bit cryptographic randoms (URL-safe base64).
  - Rate limit per IP + per `From` header on share-link verification endpoint.
  - Anomaly detection on per-IP enumeration patterns; auto-block.
  - Audit-chain logs every link-access attempt with verdict.
- Owner: ops-security + axis-drive
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6, CC6.7; ISO 27001 A.5.15, A.8.5, A.8.21; OWASP API3:2023 (Broken Object Property Level Authorization)

**T-I-02 — Personal-context file leaks into Professional-context list/search/preview**
- Asset: dual-context isolation
- L M / I H / Risk H
- Mitigations:
  - Context field non-nullable + immutable post-creation; Cedar policy `dual-context-isolation.md` refuses cross-context read.
  - Rust type system: separate types `PersonalFile` vs `ProfessionalFile`; no shared parent struct that allows leakage.
  - LEAN check `oya-check-context-isolation` validates no usecase reads both contexts in same query.
  - Per-context Meilisearch index; cross-context query at server-side refused.
- Owner: axis-drive + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.3; GDPR Art. 5(1)(b), 25

**T-I-03 — Cross-tenant share-link leaks more than file content (path / metadata)**
- Asset: cross-tenant share access record
- L M / I M / Risk M
- Mitigations:
  - Cross-tenant share returns only file content + minimal metadata (filename, size); not folder path, not sibling listing, not permission graph.
  - Cedar policy `cross-tenant-share-grant` type-narrows projection.
  - LEAN check `oya-check-cross-tenant-share-projection` refuses build if projection includes folder path or sibling enumeration.
- Owner: axis-drive + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.5.15, A.8.12; GDPR Arts. 5(1)(f), 25, 32; KR PIPA Art. 23

**T-I-04 — Search index leaks across tenants**
- Asset: Meilisearch full-text index
- L M / I H / Risk H
- Mitigations:
  - Per-tenant Meilisearch index; index name includes hashed tenant ID.
  - Cross-tenant query refused at API layer; LEAN check `oya-check-search-tenant-scoped` validates.
  - Meilisearch ACL: per-tenant API key with `documents.get` scope only.
- Owner: axis-drive + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.12

**T-I-05 — Preview render leaks via container escape (gVisor)**
- Asset: preview renderer
- L L / I H / Risk M
- Mitigations:
  - gVisor 2026-04 + seccomp BPF restrictive profile; no network egress + no host filesystem.
  - Preview output rasterised (PNG); no upstream macro execution leaks.
  - CIS Kubernetes Benchmark v1.9.0 enforced on the preview-worker pod.
  - Periodic chaos exercise: deliberately malicious Office file (Macro VBA + remote payload) — verify rasterised output + no egress.
- Owner: ops-security + axis-drive
- Residual: L (gVisor CVEs remain a residual)
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.8.28; CIS K8s 4.6, 5.2.5

**T-I-06 — Tenant-DEK leaked via log emission**
- Asset: encryption keys
- L M / I H / Risk H
- Mitigations:
  - DEK wrapped in `Secret<T>` type with stripped `Debug` impl; never serializable.
  - Secret-scanner CI lane scans every commit + log emission.
  - Rotation: 90d for tenant-DEK; rotation event re-encrypts active records.
- Owner: ops-security + cloud-secrets
- Residual: M (human-error baseline)
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.5.17, A.8.7, A.8.12; GDPR Art. 32

**T-I-07 — Object-store ACL drift makes tenant bucket public**
- Asset: object-store ACL
- L M / I H / Risk H
- Mitigations:
  - All tenant buckets created with `private` ACL by default; LEAN check `oya-check-object-store-acl-private` refuses chart drift.
  - Periodic ACL scan; any non-private ACL triggers Sev-2 alert + auto-revert.
- Owner: ops-security + axis-drive
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.8.12

**T-I-08 — Resumable upload session leaks chunk bytes via cache poisoning**
- Asset: upload session in-flight chunks
- L L / I M / Risk L-M
- Mitigations:
  - Redis ACL per-tenant key prefix; cross-tenant read forbidden.
  - Session bound to OIDC subject; resumption requires re-auth on expired ticket.
- Owner: axis-drive
- Residual: L
- Frameworks: SOC 2 CC6.6; ISO 27001 A.8.21

### Denial of Service

**T-D-01 — Upload storm: malicious tenant submits 1000s of large concurrent uploads**
- Asset: upload pipeline + object-store
- L M / I H / Risk H
- Mitigations:
  - Per-tenant upload quota (concurrent + total/24h).
  - Per-tenant rate limit on upload-rest.
  - Cost-meter: cumulative bytes budgeted; excess returns 429.
  - Backpressure to object-store; pipeline degrades to "queued" rather than crashing.
- Owner: ops-sre-reliability + axis-drive
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.6, A.8.14; GDPR Art. 32(1)(c)

**T-D-02 — Share-link mass enumeration triggers signing-key CPU starvation**
- Asset: share-link verification path
- L M / I H / Risk H
- Mitigations:
  - Argon2id parameters tuned for ≤ 50ms / verification at p99; per-IP rate limit at 100 verifications/minute.
  - Stampede protection: single-flight per `(link_id, IP)` window.
  - Pre-compute Ed25519 verification key in-memory; key rotation does not require restart.
- Owner: ops-security + ops-sre-reliability
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.30, A.8.6

**T-D-03 — Object-store cell loss → degraded download**
- Asset: Garage / MinIO / SeaweedFS backend
- L L / I H / Risk M
- Mitigations:
  - Replication-factor 3; one cell loss tolerated transparently.
  - Cross-cell replication via Garage layout (or MinIO erasure code); rebuild on cell rejoin.
  - Runbook `object-storage-degraded.md` for two-cell loss.
- Owner: ops-sre-reliability
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.30, A.8.14

**T-D-04 — Preview render queue saturation (large PDF / Office file flood)**
- Asset: preview worker queue
- L M / I M / Risk M
- Mitigations:
  - Per-tenant preview quota; max 100 concurrent renders/tenant.
  - Per-render timeout (image 5s, PDF 30s, Office 60s, video 120s).
  - Backpressure: queue depth > 300 → defer with "preview-not-yet-available" rather than block.
- Owner: ops-sre-reliability + axis-drive
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.8.6, A.8.14

**T-D-05 — Virus-scan worker saturation (large file flood)**
- Asset: ClamAV / OPSWAT worker pool
- L M / I H / Risk M
- Mitigations:
  - Per-tenant scan budget; scans deferred under load with "uploaded; pending scan" state.
  - Files in pending-scan state cannot be shared or downloaded by non-uploader.
  - Auto-scale scan worker pool on queue depth > 60s.
- Owner: ops-security + ops-sre-reliability
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.8.6

**T-D-06 — Sync delta-protocol amplification (malicious chunk-list crafted to maximise server CPU)**
- Asset: sync usecase
- L M / I M / Risk M
- Mitigations:
  - Chunk-list size bounded (max 100k chunks/session); per-tenant rate limit.
  - LBFS delta computation bounded by O(n log n) on chunk count; max 30s timeout.
- Owner: axis-drive
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.8.6

### Elevation of Privilege

**T-E-01 — Permission inheritance bug grants more than intended**
- Asset: permissions resolver
- L M / I H / Risk H
- Mitigations:
  - Cedar policy unit tests cover 5-level depth + override interactions; AC-06 covered in CI.
  - LEAN check `oya-check-permission-resolver-tested` refuses build if Cedar test corpus shrinks.
  - Annual pen-test against permission edge cases.
- Owner: axis-drive + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.3; ISO 27001 A.5.15, A.8.3; OWASP API5:2023 (Broken Function Level Authorization)

**T-E-02 — Third-party OAuth app escalates beyond granted scope**
- Asset: third-party-app OAuth surface
- L M / I H / Risk H
- Mitigations:
  - Per-grant Cedar policy refuses out-of-scope action.
  - OAuth scope tokens carry signed binding to allowed file IDs (not blanket "all files in tenant").
  - User-facing consent screen lists exact scope; auditor-visible.
- Owner: ops-security + axis-drive
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.3; OWASP API1:2023

**T-E-03 — Worker SA token leaked → arbitrary file writes / scan bypass**
- Asset: worker ServiceAccount
- L L / I H / Risk M
- Mitigations:
  - SA token bound to pod identity; rotation 24h.
  - Network policy: worker → DB + object store only; not user-facing.
  - Worker writes scoped to system-emitted events (retention sweep, version pruner, WORM scan, scan worker, preview worker); user-facing writes go via REST.
- Owner: ops-security + axis-drive
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.7

**T-E-04 — Legal-hold bypass via raw object-store delete**
- Asset: legal-hold preservation
- L L / I H / Risk M
- Mitigations:
  - Object-store role for application has no `DeleteObject` on hold-bound prefix; only soft-delete via metadata flag.
  - Object-lock retention floor at storage layer; even tenant-root cannot purge before floor expiry.
  - Hard-delete restricted to a `purge-with-2-person-rule` admin script audited via audit-chain.
  - Periodic integrity scan: compare hold-set vs storage layer; mismatch alerts.
- Owner: ops-security + compliance + axis-drive
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.27, A.8.4; SEC 17a-4(f); FINRA 4511; HIPAA §164.316

## LINDDUN Privacy Catalog

| ID | Category | Asset | Description | Mitigation | Residual |
|---|---|---|---|---|---|
| T-L-01 | Linkability | file ownership history | repeated edits link a user to a file's content drift | tenant-DEK + access controls; cross-tenant aggregations require explicit grant + audit | M (legitimate use case) |
| T-L-02 | Identifiability | filename + path patterns | "Q3-Performance-Review-AliceTan.pdf" identifies individual | redaction in share-link metadata; per-file privacy classification | L |
| T-L-03 | Non-repudiation | upload/download chain | end-user disputes upload authorship | HMAC-bound request signature + audit chain | L |
| T-L-04 | Detectability | upload-time patterns | burst of uploads correlates with business events (M&A diligence pattern) | reasonable disclosure (tenant onboarding); no broader mitigation possible | M |
| T-L-05 | Disclosure | public share-link exposure | share-link is internet-accessible; tenant misconfig may over-expose | per-tenant API key required for mint; default private; password + view-cap + TTL options; LEAN check on bare-link drift | L |
| T-L-06 | Unawareness | end-user (the tenant's user) of cross-tenant share | end-user may not know their file is shared externally | tenant DPA mandates upstream disclosure; default opt-out for cross-tenant | M-H (joint controllership) |
| T-L-07 | Non-compliance | GDPR Art. 17 right-to-erasure | erasure of a file across versions + share-link history | DSR cascade: scan all versions + audit; tombstone records; preserve audit-chain seal; legal hold may override | M (best-effort within hold) |
| T-L-08 | Linkability (cross-µservice) | drive↔mail bridge | attachment-bridge link drive file to mail thread | minimum-necessary metadata across boundary; bridge token short-lived (5 min) | L |

## Mitigations Catalog

| Mitigation | Type | Owner | Verification |
|---|---|---|---|
| Postgres per-tenant RLS | Preventive | axis-drive | `oya-check-rls-coverage` LEAN lane |
| Tenant-DEK envelope encryption | Preventive | cloud-secrets | DEK binding integrity check |
| Cedar `dual-context-isolation.md` | Preventive | ops-security | policy unit-tests |
| Cross-tenant share projection type-narrowing | Preventive | axis-drive | LEAN check + pen-test |
| Argon2id + Ed25519 share-link signing | Preventive | axis-drive | KAT test vectors + entropy audit |
| Object-store object-lock (WORM) | Preventive | axis-drive + compliance | integrity scan + AC-09 |
| ClamAV + OPSWAT virus scan pipeline | Preventive | ops-security | EICAR signature test + per-build conformance |
| DLP rules + foundry-runtime ML handoff | Preventive | council-privacy + foundry-runtime | DLP corpus test |
| gVisor sandbox on preview workers | Preventive | ops-security | egress test + macro-execution test |
| FastCDC fixed parameters | Preventive | axis-drive | LEAN check |
| Per-tenant rate limits (upload / download / share-link verify / scan / sync) | Preventive (DoS) | ops-sre-reliability | metrics |
| Ed25519 audit-chain seal | Detective + non-repudiation | audit-chain | per-event emission |
| SA-token rotation 24h, DEK 90d, API key 30d | Preventive | cloud-secrets | OpenBao audit |
| 2-person rule on hard-delete | Preventive (insider) | ops-security | OpenBao JIT |
| DSR cascade runner | Compliance | council-privacy | DSR queue SLO |

## Residual Risk Acceptance

| Risk ID | Residual | Why accepted | Re-review |
|---|---|---|---|
| T-I-05 (gVisor escape) | L | gVisor CVE backlog acceptable at 2026-05; chaos exercise quarterly | Quarterly |
| T-I-06 (DEK leak via logs) | M | human-error baseline | Quarterly |
| T-L-01 (linkability) | M | legitimate file ownership use case | Annually |
| T-L-04 (detectability via timing) | M | tenant business reality | Annually |
| T-L-06 (joint-controllership unawareness) | M-H | tenant-of-tenant disclosure responsibility | Annually |
| T-L-07 (right-to-erasure best-effort) | M | hold-vs-erasure tension | Annually |

Sign-off:
- council-architecture: `pending`
- ops-security: `pending`
- council-privacy: `pending`

## Per-Pack Overlays

### pack-kr (KR PIPA + ISMS-P + 전자문서법 + KR-FSS)

| Threat | STRIDE/LINDDUN | Rationale | Mitigation |
|---|---|---|---|
| T-KR-01 | I — Information disclosure | KR PIPA Art. 17 cross-border transfer must be SCC-gated; drive cross-region forbidden default | per-pack data residency pinning |
| T-KR-02 | N — Non-compliance | KR-FSS 5y retention for financial-sector tenants; WORM tier enforces | retention floor enforced at `immutability-tier-domain`; legal-hold extends |
| T-KR-03 | I — Linkability | KR PIPA Art. 23 sensitive PII in file content (health/political/etc.) | data-class `SENSITIVE_PIPA_ART23` annotation; Cedar refuses cross-tenant disclosure |
| T-KR-04 | T — Tampering | 전자문서법 (Electronic Document Act) requires audit-chain integrity | Ed25519 + Merkle audit-chain per Bominal ADR-0028 |

### pack-us (CCPA / CPRA + SEC 17a-4(f) + FINRA 4511)

| Threat | STRIDE/LINDDUN | Rationale | Mitigation |
|---|---|---|---|
| T-US-01 | T — Tampering | SEC 17a-4(f) requires WORM storage for broker-dealer records | object-store compliance-mode object-lock per ADR-DRIVE-0006 |
| T-US-02 | I — Discovery | FRCP Rule 26(b)(1) discovery requires file export with chain-of-custody | legal-hold + eDiscovery export + audit-chain |
| T-US-03 | S — Spoofing | CCPA / CPRA right-to-access requires identity verification | OIDC + tenant-API-key + (optional) hardware-token verification for export |
| T-US-04 | N — Non-compliance | FINRA 4511 retention period | WORM tier with FINRA-compliant retention floor |

### pack-us-healthcare (HIPAA + BAA)

| Threat | STRIDE/LINDDUN | Rationale | Mitigation |
|---|---|---|---|
| T-HC-01 | I — Information disclosure | HIPAA 45 CFR §164.502(b) minimum-necessary | data-class `PHI` on every covered field; Cedar refuses out-of-scope; OPSWAT multi-engine scan |
| T-HC-02 | N — Non-compliance | HIPAA 45 CFR §164.312(a)(2)(iv) encryption | Tenant-DEK envelope at rest; TLS 1.3 in transit; OpenBao FIPS 140-3 module |
| T-HC-03 | T — Audit-chain | HIPAA 45 CFR §164.312(b) audit controls | Ed25519 + Merkle audit-chain |
| T-HC-04 | N — Retention | HIPAA §164.316 documentation retention ≥ 6y | WORM tier with 6y retention floor |

### pack-eu (GDPR + ePrivacy + EU AI Act + eIDAS + NIS2)

| Threat | STRIDE/LINDDUN | Rationale | Mitigation |
|---|---|---|---|
| T-EU-01 | I — Information disclosure | GDPR Art. 6(1)(a) lawful-basis for cross-tenant share requires explicit consent | Cedar-gated cross-tenant share; consent recorded in audit-chain |
| T-EU-02 | N — Non-compliance | GDPR Art. 17 right-to-erasure must reconcile with legal-hold | erasure refused while hold active; tenant notified with concrete reason |
| T-EU-03 | N — EU AI Act | T1 auto-tag / T1 OCR / T2 auto-organize must register risk class | T1 = limited-risk Annex III §3 N/A; T2 auto-organize in HR-context REFUSED at Cedar layer pending ADR-DRIVE-XXXX conformity assessment |
| T-EU-04 | T — Cross-border | GDPR Chapter V cross-border transfers require SCCs | per-pack data residency; cross-pack transfers SCC-gated |
| T-EU-05 | T — eIDAS audit-chain | eIDAS 910/2014 Art. 26 AdES | audit-chain Ed25519 satisfies |

### pack-jp (APPI), pack-sg (PDPA), pack-au (Privacy Act), pack-in (DPDPA), pack-br (LGPD), pack-ae (UAE PDPL), pack-ksa (KSA PDPL)

Per-pack overlays at `regional-packs/<pack>/drive-overlay.md`.

## Re-review Triggers

- Any change to dual-context isolation invariant.
- Any new object-store backend addition / removal.
- Any change to WORM / object-lock semantics.
- Any new pack activation.
- Quarterly scheduled.
- Post-incident.
- Pen-test or audit finding.

## References

- ADR-0028 (Bominal): Audit chain (Merkle + Ed25519).
- ADR-0056, ADR-0105, ADR-0106, ADR-0117, ADR-0126, ADR-0130, ADR-0131, ADR-0132, ADR-0140.
- ADR-DRIVE-0001 through ADR-DRIVE-0006.
- `microservices/drive/PRD.md`, `dpia.md`, `compliance.md`, `policy/*.cedar`, `policy/dual-context-isolation.md`.
- AWS S3 SigV4 + Object Lock spec; tus.io 1.0; RFC 4918 WebDAV; RFC 9106 Argon2.
- OWASP ASVS v4.0.3; OWASP API Top 10 (2023); CIS Kubernetes Benchmark v1.9.0.
- NIST SP 800-57; NIST SP 800-154; NIST SSDF; SLSA L3.
- Microsoft Threat Modeling (STRIDE), LINDDUN privacy.
- SEC 17a-4(f); FINRA 4511; HIPAA 45 CFR §164.316.
