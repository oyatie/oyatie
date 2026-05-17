---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-008
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community
related_adrs: [ADR-0105, ADR-0126, ADR-0131]
doc_status: published
---

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
