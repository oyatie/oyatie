---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-008
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community
related_adrs: [ADR-0105, ADR-0135, ADR-0131]
doc_status: published
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-008 — kb-article-store + adapter-s3

## Intent

Ship the KB article BC with long-form curated articles, revisions, and S3-backed attachments with resumable multipart upload and inline ClamAV scan.

## Scope

- Types: `Article`, `Attachment`, `Revision`, `PublicationState`.
- Storage: Postgres (article + revisions) + S3 (attachments, per-tenant prefix).
- Operations: `create_article`, `edit_article`, `publish_article`, `archive_article`, `upload_attachment`, `read_article`.
- Resumable multipart upload via S3 SDK.
- Inline ClamAV scan before publication.
- sha256 recorded; S3 object lock; per-attachment integrity check.

## Deliverables

- Crate set: kernel + domain + usecase + api + adapter + adapter-postgres + adapter-s3 + rest + sdk + app.
- ClamAV integration via sidecar.

## Acceptance

- KB article publish p99 ≤ 500 ms (excl. attachment upload time).
- Attachment scan completes inline; reject on infected.
- sha256 verified on every read.
- Resumable upload tested with 1 GB chunked upload.

## Owner

axis-community + ops-iac.

## Wave 15 substance conversion

### A. Problem this IP closes

Community's KB surface must cover Notion-style long-form authoring, Zendesk Help Center public articles, GitHub Discussions documentation answers, and internal enterprise onboarding content.
The old shell named S3 and ClamAV but did not bind article states, revisions, attachment integrity, publication latency, restore runbooks, or counterpart rows to the real service artifacts.
This IP closes the KB article and attachment path behind `createKbArticle`, `publishKbArticle`, `readKbArticle`, and streaming `UploadAttachment`.

### B. Approach

Use Postgres for article metadata and immutable revision history, object storage for attachments, and a sidecar or adapter boundary for malware scanning before publication.
Store tenant, space, author, article state, revision, ontology links, attachment checksum, object-lock reference, and audit event class.
Keep article body revisions immutable; publication state changes are new rows/events, not destructive updates.
The S3-compatible adapter must be object-store-agnostic enough to satisfy oyatie public cloud, guest cloud, on-prem, colo, and own IaaS contexts under ADR-0328.

### C. Deliverables

- Add crates `oya-community-kb-article-store-kernel`, `domain`, `usecase`, `api`, `adapter`, `adapter-postgres`, `adapter-s3`, `rest`, `sdk`, and `app`.
- Update catalog files under `microservices/community/catalog/oya-community-kb-article-store-*.yaml`.
- Add metadata migrations for `kb_articles`, `kb_article_revisions`, `kb_attachments`, and `kb_attachment_scan_results`.
- Add attachment object key convention under `tenant_id/space_id/article_id/revision/attachment_id`.
- Add malware scan adapter boundary and checksum verification on every read.
- Bind SLO `microservices/community/slos/kb-article-publish-latency.openslo.yaml`.
- Update `microservices/community/runbooks/kb-attachment-restore.md`.

### D. Implementation steps

1. Map OpenAPI `GET/POST /kb/articles` and `GET/PATCH /kb/articles/{article_id}` if present to API structs.
2. Map proto `KbArticleStoreService` and messages `KbArticle`, `Attachment`, and upload chunks.
3. Define article state transitions: draft, published, archived, with revision append-only behavior.
4. Implement attachment upload as staged object, scan, checksum, object-lock, then attach to revision.
5. Reject publication if malware scan is missing, failed, or stale.
6. Emit `community.kb.article.published` and `community.kb.article.archived` events from state transitions.
7. Add restore drill using object checksum and metadata revision pointer.
8. Add tenant-prefix isolation tests and object key traversal rejection tests.
9. Add public-read policy tests for help-center articles while keeping internal articles private.
10. Record object-store context dependencies in OpenTofu module outputs or as explicit gaps.

### E. Acceptance

- Published article revisions are immutable and previous versions remain readable by authorized auditors.
- Attachment read verifies checksum before streaming bytes.
- Malware-positive fixture is rejected before publication.
- Public KB access passes only through `policy/public-read.cedar` and never exposes private workplace spaces.
- Restore runbook can reconstruct article metadata plus object reference from backup evidence.

### F. Evidence

- `microservices/community/contracts/proto/community.proto` `KbArticleStoreService`, `KbArticle`, and `Attachment`.
- `microservices/community/contracts/asyncapi/community-events.yaml` KB article events.
- `microservices/community/slos/kb-article-publish-latency.openslo.yaml`.
- `microservices/community/runbooks/kb-attachment-restore.md`.
- `microservices/community/competitor-parity-matrix.md` Notion/Confluence/Zendesk KB references in PRD and secondary forum matrix.

### G. Counterpart closure

| Counterpart | KB expectation | This IP closure |
|---|---|---|
| Notion | long-form teamspace articles with revisions | immutable article revisions and ontology links |
| Zendesk Help Center | public help articles and attachment serving | public-read Cedar path plus object integrity |
| GitHub Discussions | accepted answers can graduate to docs | events and revisions support KB publication |
| Confluence | enterprise auditability for knowledge pages | audit event class and restore drill evidence |

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/community/IP-008-kb-article-store-s3.md` matched `asyncapi, .proto`; contract files `microservices/community/contracts/openapi/community.yaml, microservices/community/contracts/asyncapi/community-events.yaml, microservices/community/contracts/proto/community.proto`; type anchor `microservices/community/manifest.json`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/community/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/community/IP-008-kb-article-store-s3.md` matched `p99, SLO`; anchors `microservices/community/manifest.json`; type anchor `microservices/community/manifest.json`.
