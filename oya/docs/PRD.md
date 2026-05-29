---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-docs
microservice: docs
status: Accepted
sales_segment: shared-substrate + hero-product
tier: tenant-facing
milestone_first_ship: M03-connect-dissolution
bominal_source: [ADR-0208-connect-dual-context-unified-channel-hub, ADR-0215-retention-legal-hold-dual-context]
related_adrs: [ADR-0056, ADR-0105, ADR-0106, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0134, ADR-0140 (retired per ADR-0145), ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345]
related_specs: [/specs/microservices/docs.json, /specs/per-microservice-flat-layout.json, /specs/agentic-slo-gated-promotion.json]
date: 2026-05-17
owner_team: axis-docs
doc_status: published
---

# PRD-docs: Collaborative Document µservice

## Purpose

The `docs` µservice is oyatie's native collaborative document substrate — the Google Docs / Microsoft Word Web / Notion-pages / Coda parallel. Per ADR-0132 (product-platform + bundle dissolution) and ADR-0135 (unbundle), docs is a standalone tenant-facing µservice owning: rich-text document authoring; real-time multi-user collaboration via CRDT (aligned with workflow-studio ADR-WS-0001); block-based document model (paragraph, heading, list, table, image, embed, code-block, math, callout); comments + suggestions (track-changes); version history; per-block ACL; cross-document embedding (workflow-studio nodes, sheets cells); document import/export (DOCX/Markdown/HTML/PDF/EPUB/LaTeX); AI writing assist (T1 grammar + summary; T2 translation); accessibility (WCAG 2.2 AA).

The µservice carries dual-context (Personal / Professional) per ADR-0135; document content never crosses context boundaries except via explicit sharing grant.

Bominal inheritance: ADR-0208 dual-context unified-channel hub + ADR-0215 retention + legal-hold overlays inherited 1:1 per `feedback_bominal_inheritance_precedence.md`; oyatie additions captured below.

## Tenant Value

- **Tenant Outcome 1 — First-party document authoring without third-party dependency.** Tenants do not need Google Docs / Microsoft 365 / Notion accounts; the µservice is a native first-party rich-text substrate.
- **Tenant Outcome 2 — Real-time multi-user collaboration with zero silent edit loss.** CRDT-based merge (Loro 1.x per ADR-DOCS-0001 + ADR-WS-0001) guarantees that two operators' non-conflicting edits merge automatically; conflicting edits surface a structured conflict resolution UI.
- **Tenant Outcome 3 — Block-based document model.** Notion-style block primitives (heading, list, table, image, embed, code-block, math, callout) enable structured authoring + per-block ACL.
- **Tenant Outcome 4 — Comments + suggestions (track-changes).** Reviewer flow with inline comments, suggested edits accepted/rejected by author, suggestion authorship preserved in audit-chain.
- **Tenant Outcome 5 — Document portability via DOCX / Markdown / HTML / PDF/A / EPUB.** GDPR Art. 20 honoured via export; tenants migrate from / coexist with Microsoft 365 + Google Docs.
- **Tenant Outcome 6 — Cross-document embedding without coupling.** Workflow-studio canvas nodes, sheets cells, slides decks embedded with policy-bounded refresh; embeds never bypass tenant isolation.
- **Internal Outcome 7 — Dual-context separation.** Personal vs Professional documents isolated at data-class + Cedar-policy + type-system boundary; cross-context inference structurally impossible.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | author | to create a document with title + initial content | I can start writing | document-store | Must |
| FR-02 | author | to add typed blocks (paragraph / heading / list / table / image / embed / code / math / callout) | I can author structured content | document-store + block-types | Must |
| FR-03 | co-author | to edit the same document concurrently with another author, with all non-conflicting edits merged automatically | real-time collaboration works | collab-crdt | Must |
| FR-04 | reviewer | to leave a comment anchored to a text range | feedback discussion is captured | comments-and-suggestions | Must |
| FR-05 | reviewer | to propose a suggested edit (track-changes) that the author can accept or reject | proofreading workflow works | comments-and-suggestions | Must |
| FR-06 | author | to see version history of the document and revert to a prior version | recovery + audit | version-history | Must |
| FR-07 | author | to set per-block visibility (private to author / team / public) | sensitive sections protected | sharing-and-permissions | Must |
| FR-08 | author | to share the document with explicit users + roles (view / comment / edit) | controlled collaboration | sharing-and-permissions | Must |
| FR-09 | author | to export the document as PDF/A / DOCX / Markdown / HTML / EPUB | data-portability (GDPR Art. 20) | export-import | Must |
| FR-10 | tenant operator | to import a DOCX / Markdown / HTML file as a new document | one-shot migration | export-import | Must |
| FR-11 | author | to embed a workflow-studio canvas / sheets cell / slides deck | composition without coupling | embed-resolver | Must |
| FR-12 | reader | to search within a document for text + structured filters | findability | document-search | Must |
| FR-13 | author | to attach a file (image / PDF / video) to a document | rich media authoring | attachments | Must |
| FR-14 | author | to receive grammar suggestions + auto-summary | AI writing assist | (cross-cutting; T1) | Must |
| FR-15 | tenant compliance officer | to put a professional document under legal hold | content preserved past retention | document-store | Must |
| FR-16 | tenant operator | to receive a webhook on document lifecycle change | downstream Workflow can react | (cross-cutting) | Must |
| FR-17 | author | to author math equations (KaTeX / MathJax) + citations (BibTeX) | academic + technical authoring | block-types | Should |
| FR-18 | author | to render documents accessible to screen readers (WCAG 2.2 AA) | accessibility compliance | block-types | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Document open (cold) | ≤150ms | ≤300ms | ≤700ms | Postgres + S3 read; first paint |
| Document open (warm) | ≤30ms | ≤100ms | ≤250ms | Valkey cache hit |
| Save | ≤30ms | ≤100ms | ≤300ms | CRDT op commit + audit emit |
| Collab cursor sync | ≤40ms | ≤150ms | ≤400ms | aligns with workflow-studio §"Performance" |
| Search-within-doc | ≤30ms | ≤100ms | ≤250ms | per-doc full-text index |
| Doc-list (1000 docs) | ≤50ms | ≤200ms | ≤500ms | paginated; cursor-based |
| Export PDF (50-page doc) | ≤1s | ≤3s | ≤7s | Pandoc/WeasyPrint pipeline (gVisor sandbox) |
| Export DOCX | ≤500ms | ≤2s | ≤5s | Pandoc |
| Import DOCX (50-page doc) | ≤1s | ≤3s | ≤7s | Pandoc + sanitisation |
| Comment post | ≤30ms | ≤100ms | ≤250ms | indexed insert |
| Attachment upload (10MB) | ≤500ms | ≤2s | ≤5s | S3 streaming + ClamAV |

### Security

- All document payloads encrypted-at-rest under tenant-DEK (per Bominal ADR-0111 envelope encryption) in Professional context; Personal context uses E2E where the tenant has declared E2E.
- All export pipeline workers (Pandoc + WeasyPrint + Chromium-headless) run in gVisor sandbox; per-export tmpfs; output validated against output-type schema before emission.
- All attachment uploads scanned by ClamAV (default) or OPSWAT MetaDefender (pack-us-healthcare); macros disabled in DOCX import; HTML import scrubbed by `ammonia` (Rust) with strict allowlist.
- All embed-resolver fetches mTLS-bound to source µservice; embeds carry pack-tag; cross-pack embed refused at Cedar layer.
- All AI-assist inference uses tenant-DEK-wrapped prompts; no cross-tenant training.

### Audit + Compliance

- Every `DocumentCreated / DocumentEdited / DocumentShared / DocumentExported / CommentPosted / SuggestionAcceptedRejected / VersionRestored / LegalHoldApplied` emits an audit-chain record (Merkle + Ed25519 per Bominal ADR-0028).
- Legal-hold preserves document content + edit history + comment thread + audit chain past retention expiry.
- Per-jurisdiction retention (KR PIPA / EU GDPR / US sector-specific) computed per ADR-0140 Cedar pack overlay.
- WCAG 2.2 AA accessibility evidence emitted per export.

### Availability + SLO

- Availability target: 99.95 % monthly for document-read path; 99.9 % for write path; 99.9 % for export pipeline.
- RTO ≤ 15 min; RPO ≤ 60 s (Postgres logical replication + S3 cross-AZ).
- Embed-resolver degrades to "stale snapshot — source unavailable" rather than block.

### Data residency

- Tenant data pinned to the tenant's region per ADR-0117 + ADR-0140; cross-region replication forbidden by default; SCC-gated when activated.

### DR Posture (ADR-0343)

- RTO/RPO target: manifest `dr` declares `rto_p99_seconds=900` and `rpo_p99_seconds=60` for document-read/write state. HIPAA-2024 (3600s/300s), SOC2-T2 (14400s/900s), ISO27001-2022 (14400s/3600s), KR-PIPA-2023-amendment (14400s/900s), and KR-CSAP-v3.1 (3600s/900s) are looser, so the effective docs bound remains 900s RTO and 60s RPO.
- failover_runbook: `runbooks/dr-failover.md`; manifest backup substrate is `postgres_wal_g`, `object_storage_versioned`, and `valkey`.
- multi_region_active_active: true, with manifest replication shape `active-active-multi-az-cross-region-warm`; cross-pack replication remains forbidden unless the tenant enables SCC-gated residency flow.
- WHY: co-authors need their document, comments, suggestions, and legal-hold lineage restored without silent edit loss or cross-pack leakage.

### Capacity Model (ADR-0340)

- Per-tenant baseline: manifest `capacity_model` declares 0.12 vCPU, 384Mi RAM, 25Gi storage, 3 Valkey connections, 3 Postgres connections, and 6 outbound HTTP connections per tenant.
- Scaling dimension: `per_user`; collaborative editing sessions, CRDT spooling, exports, and attachment resolution scale with active editors rather than raw request count.
- Cell placement class: Tier-3, matching manifest `capacity_model.cell_placement_class`, because docs is a collaborative application surface rather than tenant-customer code execution.
- Autoscaling boundaries: document-store REST min 5 / max 100, collab worker min 5 / max 200, export/import gVisor worker min 10 / max 200, with worker queues gating scale-out before Postgres/Valkey saturation.
- WHY: docs has a read-heavy collaboration profile but expensive export/import spikes, so steady editor capacity and sandboxed worker capacity need independent limits.

### Sustainability + Cost Attribution (ADR-0344)

- Every audit-chain row emits `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, and `region` for create/edit/share/export/comment/suggestion/version/legal-hold events.
- Provider routing affected by carbon: yes for export/import, embed refresh, and AI-assist queues; no for interactive document open/save, legal-hold application, HIPAA emergency disclosure, or ACL enforcement.
- Per-tenant transparency surface: FinOps portal shows document storage, edit-event volume, export/import worker time, embed refresh volume, AI-assist calls, and attachment scan cost by tenant/capability/provider/cell/compliance_pack.
- WHY: document authoring produces many small audit events and occasional high-cost conversions, so CSRD, SB-253, and SEC climate disclosure need both low-latency exclusions and batch-job cost visibility.

### API Versioning Posture (ADR-0342)

- Public API version model: YYYY-MM-DD carrier triplet via `Oyatie-Version` header, `/v/<YYYY-MM-DD>` URL prefix, and proto3 `oyatie_version` field for document, block, comment, sharing, export/import, embed, and lifecycle event contracts.
- SDK semver model: major.minor.patch for editor SDKs, embed clients, and export/import clients.
- Support window: last N=3 public API versions for at least 180 days, with export/import schema deprecation visible to tenant admins.
- Per-tenant pinning supported: yes, so regulated tenants can pin document and export behavior during validation windows.
- Internal-mesh exemption: yes; direct gRPC with drive, sheets, slides, workflow, and foundry remains exempt under ADR-0145 when the contract is internal-only.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename for new crates). Eight primary BCs.

| BC | Crate family | Purpose | Key entities |
|---|---|---|---|
| `document-store` | `oya-docs-document-store-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-s3,rest,worker,sdk,app}` | Document persistence; metadata; content blobs; tenant-DEK envelope; legal hold | `Document`, `BlockTree`, `RetentionPolicyRef`, `LegalHoldRef` |
| `collab-crdt` | `oya-docs-collab-crdt-{kernel,domain,usecase,api,adapter,adapter-valkey,worker,sdk,app}` | CRDT op stream; presence; cursor sync; conflict surfacing | `CrdtOp`, `PresenceSnapshot`, `CrdtMergeEngine`, `Conflict` |
| `block-types` | `oya-docs-block-types-{kernel,domain,usecase,api,adapter,sdk,app}` | Block-type schema + renderers + sanitisation (paragraph, heading, list, table, image, embed, code, math, callout) | `Block`, `BlockKind`, `InlineStyle`, `RenderedBlock` |
| `comments-and-suggestions` | `oya-docs-comments-and-suggestions-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,app}` | Comment threads + suggestion (track-changes) state machine + anchor stability across edits | `Comment`, `Thread`, `Suggestion`, `Anchor` |
| `version-history` | `oya-docs-version-history-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,app}` | Versioned snapshots + revert; CRDT op log compaction | `Version`, `Snapshot`, `RevertOp` |
| `sharing-and-permissions` | `oya-docs-sharing-and-permissions-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,app}` | Per-doc + per-block ACL; share-link issuance; role-based access | `ShareGrant`, `Role`, `BlockAcl`, `ShareLink` |
| `export-import` | `oya-docs-export-import-{kernel,domain,usecase,api,adapter,adapter-pandoc,adapter-weasyprint,adapter-chromium,rest,worker,app}` | DOCX / Markdown / HTML / PDF/A / EPUB / LaTeX pipelines under gVisor | `ExportJob`, `ImportJob`, `FormatProfile` |
| `embed-resolver` | `oya-docs-embed-resolver-{kernel,domain,usecase,api,adapter,rest,worker,app}` | Cross-µservice embed fetching with policy-bounded refresh; stale-fallback | `Embed`, `EmbedSource`, `RefreshSnapshot` |

Naming justification (one of eight; same shape applies to others) — `document-store`:

```
NAME: oya-docs-document-store-<layer>
JUSTIFICATION:
- microservice = docs: this µservice; ADR-0056 v4.1 flat BNF + ADR-0131 per-microservice
  folder. No shared|vertical bisection.
- bc-tokens = document-store: primary BC for document persistence; siblings (collab-crdt,
  block-types, comments-and-suggestions, version-history, sharing-and-permissions,
  export-import, embed-resolver) justify explicit BC token per ADR-0056 v4.1
  BC-optionality rule.
- layer = <layer>: one crate per layer per ADR-0105 13-value canonical enum.
  - kernel: port-trait + entity types (Document, BlockTree, RetentionPolicyRef,
    LegalHoldRef, DocumentContext{Personal|Professional}). Zero I/O. data_class annotations.
  - domain: pure document-invariant math (block-tree ordering, ACL coverage, hold
    coverage).
  - usecase (per ADR-0106): orchestrators (create-document, edit-block, share-document,
    apply-legal-hold, expire-retention) reading via ports.
  - api: protocol-neutral typed contracts.
  - adapter: protocol-neutral implementations of kernel ports.
  - adapter-postgres: backend-qualified adapter (per ADR-0105 Amendment 3
    *-adapter-<backend> pattern); implements DocumentRepository against Postgres with RLS.
  - adapter-s3: backend-qualified adapter for content-blob storage (S3-compatible).
  - rest: HTTP + WebSocket handler/route layer.
  - worker: long-lived background workers (retention sweep, version compaction).
  - sdk: client library for tenants + workflow consumers.
  - app: composition root binary.
- exemptions claimed: none.
```

(Equivalent justifications recorded for the other seven BCs at `microservices/docs/specs/naming-justification.md`.)

Layer mapping table per BC (13-layer enum from ADR-0105; `usecase` per ADR-0106):

| BC | kernel | domain | usecase | api | adapter | adapter-postgres | adapter-s3 | adapter-valkey | adapter-pandoc | adapter-weasyprint | adapter-chromium | rest | worker | sdk | app |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `document-store` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | ✓ | ✓ | ✓ | ✓ |
| `collab-crdt` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | — | — | — | — | ✓ | ✓ | ✓ |
| `block-types` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | ✓ | ✓ |
| `comments-and-suggestions` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | ✓ | ✓ | — | ✓ |
| `version-history` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | ✓ | — | ✓ |
| `sharing-and-permissions` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | ✓ | — | — | ✓ |
| `export-import` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ |
| `embed-resolver` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | ✓ | ✓ | — | ✓ |

Total crates introduced by this µservice: **65**.

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated per Bominal ADR-0028):

| Port trait | Kernel crate | Implemented in | Data classes touched |
|---|---|---|---|
| `DocumentRepository` | `oya-docs-document-store-kernel` | `-adapter-postgres` | `PERSONAL_DOC_CONTENT` + `PROFESSIONAL_DOC_CONTENT` (per-context envelope encryption) |
| `BlockBlobStore` | `oya-docs-document-store-kernel` | `-adapter-s3` | `PERSONAL_DOC_CONTENT` + `PROFESSIONAL_DOC_CONTENT` |
| `CrdtMergeEngine` | `oya-docs-collab-crdt-kernel` | `-adapter` (Loro 1.x wrapping) | `PERSONAL_DOC_CONTENT` + `PROFESSIONAL_DOC_CONTENT` |
| `PresenceBroadcast` | `oya-docs-collab-crdt-kernel` | `-adapter-valkey` | `BEHAVIORAL_TENANT_PRODUCT` |
| `BlockSchemaRegistry` | `oya-docs-block-types-kernel` | `-adapter` | `INTERNAL_ONLY` |
| `CommentRepository` | `oya-docs-comments-and-suggestions-kernel` | `-adapter-postgres` | `PII_IDENTIFYING` (commenter identity) |
| `SuggestionStateMachine` | `oya-docs-comments-and-suggestions-kernel` | `-adapter-postgres` | `PERSONAL_DOC_CONTENT` + `PROFESSIONAL_DOC_CONTENT` |
| `VersionStore` | `oya-docs-version-history-kernel` | `-adapter-postgres` | `AUDIT` |
| `AclRepository` | `oya-docs-sharing-and-permissions-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT` |
| `ShareLinkIssuer` | `oya-docs-sharing-and-permissions-kernel` | `-adapter` | `SECRET` (share-link tokens) |
| `PandocConverter` | `oya-docs-export-import-kernel` | `-adapter-pandoc` | `PERSONAL_DOC_CONTENT` + `PROFESSIONAL_DOC_CONTENT` |
| `PdfRenderer` | `oya-docs-export-import-kernel` | `-adapter-weasyprint` (default) / `-adapter-chromium` (high-fidelity) | `PERSONAL_DOC_CONTENT` + `PROFESSIONAL_DOC_CONTENT` |
| `AttachmentScanner` | `oya-docs-document-store-kernel` | `-adapter` (ClamAV / OPSWAT) | `BEHAVIORAL_TENANT_PRODUCT` |
| `EmbedResolver` | `oya-docs-embed-resolver-kernel` | `-adapter` | `BEHAVIORAL_TENANT_PRODUCT` |
| `RetentionPolicyResolver` | `oya-docs-document-store-kernel` | `-adapter` (resolves to `tenancy` µservice via Workflow) | `AUDIT` |
| `LegalHoldStore` | `oya-docs-document-store-kernel` | `-adapter-postgres` | `AUDIT` |

Data-class enforcement: every kernel struct field carries a `#[data_class(...)]` annotation; the `oya-check-data-class` LEAN lane refuses unannotated fields.

Cross-product rule: `docs` MUST NOT import another product µservice crate at any layer. Cross-product flows go through Workflow (events) or Ontology (entity reads/writes). Consumed µservices: `tenancy` (tenant + identity resolution), `audit-chain` (seal emission), `mail` (share-via-email), `messenger` (share-to-channel + mention-from-doc), `workflow-studio` (CRDT library + collaboration substrate share via published port trait re-export only), `workflow-engine` (doc-as-trigger), `drive` (attachment storage hierarchy), `observability` (telemetry). LEAN-A2 CI lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice docs`
- `oya gate validate lean-a2 --microservice docs`
- `oya gate validate port-location --microservice docs`
- `oya gate validate layer-correctness --microservice docs`
- `oya gate validate per-microservice-layout --microservice docs`
- `oya gate validate statelessness --microservice docs`
- `oya gate validate shardability --microservice docs`
- `oya gate validate hyperscaler-maturity --microservice docs`
- `oya gate validate crdt-no-silent-loss --microservice docs` (NEW; mirrors AC-06 of workflow-studio)
- `oya gate validate acl-enforcement-correctness --microservice docs` (NEW)
- `oya gate validate export-sandbox-conformance --microservice docs` (NEW; gVisor + tmpfs)
- `oya gate validate wcag-22-aa-conformance --microservice docs` (NEW)
- `oya gate validate ooxml-import-fidelity --microservice docs` (NEW; per ADR-DOCS-0006)

## Integration via Workflow + Ontology

### Workflow events produced

| Event | Topic | Trigger | Consumed by | Idempotency key |
|---|---|---|---|---|
| `DocumentCreated` | `docs.document.lifecycle.v1` | new document written | audit-chain, workflow-engine (triggers), ontology | `document_id` |
| `DocumentEdited` | `docs.document.lifecycle.v1` | block-tree mutation committed | audit-chain | `document_id + version` |
| `DocumentArchived` | `docs.document.lifecycle.v1` | soft-delete | audit-chain | `document_id + archived_at` |
| `DocumentShared` | `docs.sharing.v1` | share grant issued | audit-chain, mail (share-via-email handoff), messenger (share-to-channel) | `document_id + grantee_id` |
| `DocumentShareRevoked` | `docs.sharing.v1` | share grant revoked | audit-chain, embed-resolver (cache purge) | `document_id + grantee_id + revoked_at` |
| `CommentPosted` | `docs.comments.v1` | comment created | audit-chain, mail (notification), messenger (mention) | `comment_id` |
| `SuggestionAccepted` / `SuggestionRejected` | `docs.suggestions.v1` | suggestion state transition | audit-chain | `suggestion_id + state` |
| `DocumentExported` | `docs.export.v1` | export pipeline emits final blob | audit-chain | `export_job_id` |
| `DocumentImported` | `docs.import.v1` | import pipeline finalises | audit-chain | `import_job_id` |
| `EmbedRefreshed` | `docs.embed.v1` | embed source re-fetched | observability | `document_id + embed_id + fetched_at` |
| `LegalHoldApplied` / `LegalHoldReleased` | `audit.docs.legal_hold.v1` | hold transition | audit-chain, governance | `document_id + hold_id` |
| `VersionRestored` | `docs.version.v1` | revert applied | audit-chain | `document_id + version_to + version_from` |

### Workflow events consumed

| Event | Producer | Handler | Action |
|---|---|---|---|
| `TenantOnboarded` | `tenancy` | document-store usecase | provision tenant-DEK; create default workspace |
| `TenantOffboarded` | `tenancy` | document-store usecase | mark documents for retention sweep / legal-hold scan |
| `MailDeliveryFailed` | `mail` | sharing usecase | retry / surface "delivery-failed" recipient card on share notification |
| `MessengerMentionResolved` | `messenger` | sharing usecase | bind mention to comment thread |
| `WorkflowTrigger` | `workflow-engine` | document-store usecase | doc-bound automation (e.g., "publish doc when approved") |
| `WorkflowStudioDefinitionPublished` | `workflow-studio` | embed-resolver usecase | invalidate embedded canvas snapshots referencing changed definition |
| `SheetsCellChanged` | `sheets` | embed-resolver usecase | invalidate embedded cell snapshots |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit |
|---|---|---|---|
| `Document{document_id, tenant, context, title_hashed, ...}` | `authored_by→Person`, `lives_in→Tenant` | `document-store` | Ed25519 |
| `Block{block_id, document_id, kind, position}` | `block_of→Document` | `document-store` | Ed25519 |
| `Comment{comment_id, document_id, author_id, anchor_hash}` | `comment_on→Document`, `comment_by→Person` | `comments-and-suggestions` | Ed25519 |
| `Suggestion{suggestion_id, document_id, author_id, state}` | `suggested_on→Document` | `comments-and-suggestions` | Ed25519 |
| `ShareGrant{grant_id, document_id, grantee_ref, role}` | `granted_on→Document`, `granted_to→Person` | `sharing-and-permissions` | Ed25519 |
| `Version{version_id, document_id, version_sha, sealed_at}` | `version_of→Document` | `version-history` | Ed25519 |
| `LegalHold{hold_id, document_id, opened_by, opened_at}` | `holds→Document` | `document-store` | Ed25519 |

### Ontology reads

| Object | Read by | Query shape |
|---|---|---|
| `User` (tenant directory) | `document-store`, `sharing-and-permissions`, `comments-and-suggestions` | by `(tenant_id, user_id)` |
| `Tenant` | `document-store`, `embed-resolver` (cross-µservice embed resolution) | by `tenant_id` |
| `RetentionPolicy` | `document-store` | by `(tenant_id, pack)` |
| `WorkflowDefinition` | `embed-resolver` | by `definition_id` (for workflow-studio embeds) |
| `SheetCell` | `embed-resolver` | by `(workbook_id, cell_ref)` |

## Competitive Benchmark

| Competitor | Product | Parity dimensions | Primary source |
|---|---|---|---|
| Google Docs | Workspace | block-aware editing; CRDT-like merge; commenting; suggestions; export DOCX/PDF | `developers.google.com/docs/api` |
| Microsoft Word Web | M365 | OOXML round-trip; Copilot AI assist; suggestion mode; comments | `learn.microsoft.com/graph/api/resources/document` |
| Notion | Notion pages | block-based model; per-block ACL; embedding; database integration | `developers.notion.com` |
| Coda | Coda docs | doc-as-app; table embeds; integrations | `coda.io/developers` |
| Quip (Salesforce) | Quip | collab; comments; spreadsheet embed | `quip.com/dev/automation` |
| Dropbox Paper | Paper | lightweight; embed-friendly; markdown export | `developers.dropbox.com/paper` |
| ONLYOFFICE | ONLYOFFICE Docs | OOXML-first; self-hostable; collab over WebSocket | `api.onlyoffice.com` |
| Collabora Online | Collabora | LibreOffice-derived; self-hostable; OOXML/ODF | `sdk.collaboraonline.com` |
| Etherpad | Etherpad | OT-based plain-text collab (legacy reference) | `etherpad.org` |
| HackMD | HackMD | Markdown-first; technical docs | `hackmd.io/api/v1` |
| Confluence | Atlassian | enterprise wiki + docs; permissions | `developer.atlassian.com/cloud/confluence` |
| Obsidian Publish | Obsidian | markdown-first publishing | `help.obsidian.md/Obsidian+Publish/Publish` |
| Craft | Craft Docs | block-based; Apple-ecosystem polish | (consumer; no public API) |
| Roam Research | Roam | graph-style; bidirectional links | `roamresearch.com/help` |

Key parity gaps to close (ordered):

1. **CRDT zero-silent-loss + cross-µservice library alignment** — Loro per ADR-DOCS-0001 mirroring workflow-studio ADR-WS-0001. **Differentiator** vs. Google Docs OT-based; matches Notion's Yjs and Quip's OT internally.
2. **Per-block ACL** — Notion-class; Google Docs only has whole-doc + commented-range. **Differentiator vs Google Docs.**
3. **Cross-document embedding via policy-bounded resolver** — workflow-studio canvases, sheets cells, slides decks. **Differentiator.**
4. **Dual-context (Personal / Professional) isolation enforced structurally** — no competitor enforces context-separation in code; tenant-policy only. **Differentiator.**
5. **OOXML import-export round-trip fidelity** — must hit best-effort fidelity per ADR-DOCS-0006; Microsoft Word Web parity target.
6. **PDF/A archival-grade export** — PDF/A-1b + PDF/A-2u per legal-evidence tenants; only Microsoft Word + LibreOffice/Collabora cover this today.
7. **AI writing assist with EU AI Act conformance** — per ADR-DOCS-0005; refused at Cedar layer in HR-context per pack-eu.
8. **WCAG 2.2 AA accessibility** — Notion + Coda are partial; Google Docs is the bar.

## Performance Targets (canonical bench surface)

| Metric | Target | Verification |
|---|---|---|
| Document-open (cold) p99 | ≤ 300ms | `cargo bench -p oya-docs-document-store-adapter-postgres -- doc_open_cold` |
| Document-open (warm) p99 | ≤ 100ms | `cargo bench -p oya-docs-document-store-adapter-postgres -- doc_open_warm` |
| Save p99 | ≤ 100ms | `cargo bench -p oya-docs-document-store-usecase -- save` |
| Collab cursor sync p99 | ≤ 150ms | `cargo bench -p oya-docs-collab-crdt-adapter-valkey -- cursor_sync` |
| Search-within-doc p99 | ≤ 100ms | `cargo bench -p oya-docs-document-store-domain -- search_in_doc` |
| Export PDF (50-page) p99 | ≤ 3s | `cargo bench -p oya-docs-export-import-adapter-weasyprint -- export_pdf_50p` |
| Export DOCX p99 | ≤ 2s | `cargo bench -p oya-docs-export-import-adapter-pandoc -- export_docx` |
| Import DOCX (50-page) p99 | ≤ 3s | `cargo bench -p oya-docs-export-import-adapter-pandoc -- import_docx_50p` |

Error budget: monthly 99.95% availability → ~22 min/month.

## Horizontal Scalability

State strategy (per Bominal ADR-0019): `mixed`. Postgres (document metadata + per-tenant RLS); S3 (block content blobs + attachments; per-tenant prefix); Valkey (collab presence + CRDT op fan-out + per-doc cache; per-tenant key prefix); stateless workers for export + import + version compaction + embed-refresh + retention-sweep.

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Active documents | 1M | 10M | Postgres connection pool > 70% |
| Concurrent editor sessions | 50k | 500k | rest-pod CPU > 70% or WS gateway lease pressure |
| Edits/s | 5k | 50k | document-store-rest p99 > 100ms |
| Export jobs concurrent | 50 | 500 | gVisor worker queue depth > 5min |
| Import jobs concurrent | 20 | 200 | worker queue depth > 5min |
| Attachment uploads/s | 100 | 1k | object-storage write p99 > 1s |
| Embed-refresh/s | 500 | 5k | embed-resolver worker queue > 60s |

Scale-out policy:
- Kubernetes HPA: rest pods scale on CPU > 70%; min 5, max 100.
- Postgres: per-tenant logical shard; cross-cell replication-factor 3 with Patroni.
- Valkey: cluster mode; per-tenant key prefix; eviction policy `volatile-lru` for collab presence; persistent for CRDT op spool.
- S3: per-pack bucket; per-tenant prefix; Object Lock for legal-hold blobs.
- Export-pipeline workers: pre-warmed pool of 10 gVisor sandboxes; cold-start ≤ 800ms.

Cross-region: M03 launches in KR (ap-seoul-1); M04 expands to EU + US per ADR-0117 jurisdiction pack.

Sharding: documents partitioned by `(tenant_id, document_id_first_byte)`; comments partitioned by `document_id`; versions partitioned by `document_id`.

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Document create + initial block-tree write completes within p99 ≤ 100ms | `cargo bench` |
| AC-02 | Round-trip byte-equality: load(emit(canvas-doc)) is byte-equal to the original for 100-doc reference corpus | `cargo nextest -p oya-docs-document-store-domain -- round_trip_byte_equality` |
| AC-03 | OOXML import → CRDT → OOXML export preserves >= 95% of original DOCX features on the Microsoft test corpus | `cargo nextest -p oya-docs-export-import-adapter-pandoc -- ooxml_roundtrip` |
| AC-04 | Per-block ACL: a block marked `private` is never returned in a query by a principal lacking the per-block grant | `cargo nextest -p oya-docs-sharing-and-permissions-domain -- per_block_acl` |
| AC-05 | Two concurrent editors making non-conflicting edits both observe each other's edits within p99 ≤ 150ms | E2E `tests/e2e/concurrent-editors.rs` |
| AC-06 | CRDT never silently loses an edit: every accepted op is reachable from final state OR surfaced as conflict | `cargo nextest -p oya-docs-collab-crdt-domain -- never_silent_loss` (mirrors workflow-studio AC-06) |
| AC-07 | Personal-context document content NEVER appears in Professional-context query | `cargo nextest -p oya-docs-document-store-domain -- context_isolation` |
| AC-08 | Tenant-DEK envelope encryption applied to Professional document content; verified at rest | `tests/e2e/encryption-at-rest.rs` |
| AC-09 | Export pipeline runs in gVisor sandbox with tmpfs only; verified by escape-attempt test | `cargo nextest -p oya-docs-export-import-adapter-pandoc -- gvisor_escape_blocked` |
| AC-10 | PDF export passes PDF/A-1b validator (veraPDF) | `cargo nextest -p oya-docs-export-import-adapter-weasyprint -- pdfa_validation` |
| AC-11 | WCAG 2.2 AA: every export passes axe-core + Pa11y validation | E2E `tests/e2e/wcag-22-aa.rs` |
| AC-12 | Audit-chain seal emitted for every doc lifecycle + share + comment + suggestion + version + export + import | `cargo nextest -p oya-docs-document-store-app -- audit_chain_emission` |
| AC-13 | Legal-hold preserves document + edit history + comments + audit-chain past retention expiry | `cargo nextest -p oya-docs-document-store-domain -- legal_hold` |
| AC-14 | Attachment upload is scanned (ClamAV / OPSWAT) before persistence; malware refused | `cargo nextest -p oya-docs-document-store-adapter -- attachment_scan` |
| AC-15 | Embed-resolver returns stale snapshot when source unavailable; never blocks the document open | `cargo nextest -p oya-docs-embed-resolver-domain -- stale_fallback` |
| AC-16 | `oya gate validate per-microservice-layout --microservice docs` exit 0 | ADR-0131 lane |

## Open Questions

| # | Question | Owner | Target |
|---|---|---|---|
| 1 | CRDT op log compaction cadence (continuous vs nightly vs version-aligned) — defaults to version-aligned per ADR-DOCS-0001; revisit if storage cost dominates | axis-docs | subsequent-to-M03-completion |
| 2 | PDF rendering backend default — WeasyPrint vs Chromium-headless — settled in ADR-DOCS-0003 (WeasyPrint default; Chromium high-fidelity opt-in) | axis-docs | resolved |
| 3 | Per-block ACL UX surfacing in the editor (badge / overlay / split-view) — defer to council-design-system | council-design-system | M03-onward1 |
| 4 | Math-equation rendering library — KaTeX (fast) vs MathJax (accuracy) — KaTeX default; MathJax fallback for unsupported macros | axis-docs | resolved |
| 5 | Federation with external Google Docs / Word source-of-truth — migration-only at GA; coexistence mode subsequent-to-M04-completion | council-product | subsequent-to-M04-completion |
| 6 | Document publishing surface (public-read URL vs share-link only) — share-link only at GA; public-read subsequent-to-M04-completion | council-product | subsequent-to-M04-completion |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum | layer authority |
| ADR-0106 | application→usecase | layer rename |
| ADR-0117 | Cloud-native infrastructure | data residency |
| ADR-0135 | unbundle (parallel session) | dual-context inheritance |
| ADR-0139 | Agentic SLO-gated promotion | gate authority |
| ADR-0131 | Per-microservice flat layout | layout authority |
| ADR-0132 | Product-platform + bundle dissolution | µservice independence |
| ADR-0133 | Industry-best-practice conformance | hyperscaler-grade bar |
| ADR-0134 | dissolution Strangler migration | migration policy |
| ADR-0140 | Cedar policy enforcement | policy substrate |
| Bominal ADR-0208 | dual-context unified-channel hub | inherited 1:1 |
| Bominal ADR-0215 | retention + legal-hold dual-context | inherited 1:1 |
| ADR-WS-0001 | workflow-studio CRDT library selection (Loro) | cross-µservice CRDT alignment authority |
| ADR-DOCS-0001 | CRDT library selection (Loro; aligns with WS-0001) | this µservice |
| ADR-DOCS-0002 | Block-type system (block-based per Notion) | this µservice |
| ADR-DOCS-0003 | Export pipeline architecture (Pandoc + WeasyPrint default) | this µservice |
| ADR-DOCS-0004 | ACL granularity (per-block; Notion-style) | this µservice |
| ADR-DOCS-0005 | AI writing assist EU AI Act bounds | this µservice |
| ADR-DOCS-0006 | DOCX import fidelity policy (best-effort fidelity) | this µservice |

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `docs` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `docs` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 3 module pin(s) across 1 context(s).
- Scaling input: `per_user` with cell placement `Tier-3` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
