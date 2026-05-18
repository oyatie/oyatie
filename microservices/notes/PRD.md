---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-notes
microservice: notes
status: Accepted
sales_segment: connect-suite-product
tier: hero-product
milestone_first_ship: M02-foundation
bominal_source: []
related_adrs: [ADR-0008, ADR-0056, ADR-0063, ADR-0064, ADR-0105, ADR-0106, ADR-0117, ADR-0126, ADR-0130, ADR-0131, ADR-0132, ADR-0133]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/agentic-slo-gated-promotion.json]
date: 2026-05-17
owner_team: axis-notes
doc_status: published
---

# PRD-notes: Personal-First Notes + Bidirectional Knowledge Graph + E2E-Default Capture

## Purpose

The `notes` µservice is the **short-form personal-notes + knowledge-capture** surface that ships under parallel ADR-0126 (Connect super-app expansion) and ADR-0132 (suite-and-bundle dissolution) as a stand-alone hero µservice. It is NET-NEW; no `oya-connect-notes-*` legacy crates exist.

The µservice owns: **note (Markdown body + YAML frontmatter), notebook/stack (loose hierarchy), tag (multi-tag per note + tag-graph), backlink (Obsidian-style `[[wikilink]]` bidirectional), daily-note (auto-created per day), template, web-clipper-bridge, share-link (read-only by default), embed (image + video + file via drive µservice), checklist, reminder (cross-µservice to tasks), folder (optional; flat-by-default), version-history (linear), search (cross-note + tag-faceted; client-side for E2E notes), graph-view (Obsidian-style force-directed; client-side render), import (Apple Notes export + Evernote ENEX + OneNote + Notion Markdown + Bear + Obsidian vault), export (Markdown + PDF + JSON portable), collab-edit (optional per note; Loro CRDT align), AI-summarize (T1), AI-suggest-tag (T1), AI-link-suggest (T1)**, across the dual-context model (Personal B2C + Professional B2B).

This µservice is a **hero product**, end-user-facing through Workflow Studio shell and standalone notes clients (web + desktop + mobile + Web Extension browser clipper). It is also consumable as a shared substrate by other oyatie products via the `notes.note.v1` Workflow events and the `Note`/`Tag` Ontology object types.

### Differentiation from `docs`

| Dimension | docs µservice | notes µservice |
|---|---|---|
| Length | long-form (multi-page documents) | short-form (single-screen-most-of-the-time) |
| Editing model | rich-text WYSIWYG + commenting + revision branches | Markdown-first + plaintext-first + linear version-history |
| Sharing | collaborative-by-default with full ACL + share-workflow | personal-by-default; share is exception with read-only links |
| Encryption | tenant-DEK envelope; admin-readable under four-eyes | **E2E client-derived for Personal-tier (default ON)**; tenant-DEK for Professional-tier (default OFF, admin policy override) |
| Bidirectional linking | optional outline-references | **first-class `[[wikilink]]` + tag-graph + force-directed graph view** |
| Daily/template | not a primary affordance | first-class daily-note + template gallery |
| Export | DOCX + PDF + HTML + Markdown | Markdown + frontmatter + JSON-canonical + PDF |
| Import | DOCX + GDocs + ODT | Apple Notes / Evernote ENEX / OneNote / Notion / Bear / Obsidian vault |
| AI assist | scoped paragraph-rewrite, table-of-contents, etc. | **summarize + tag-suggest + link-suggest**; STRICTLY refused on E2E notes |
| Collaboration | always-on Loro CRDT | opt-in per note; default solo |
| Web clipper | not in scope | **first-class browser extension capture** |

The privacy posture is sharper than docs: notes are *first-thought capture* (cf. Obsidian, Standard Notes, Apple Notes Lockable notes), and the Personal pillar defaults to **E2E ON**, matching Standard Notes' and Apple Notes' on-device-encryption defaults.

## Tenant Value

- **Tenant Outcome 1 — One inbox for thought capture.** Tenants get Apple Notes / Obsidian / Bear-class capture inside the same shell as mail, calendar, docs, tasks, messenger — without context-switching to a separate app.
- **Tenant Outcome 2 — Privacy-by-default for personal capture.** Personal-tier notes are E2E-encrypted by default; oyatie operators + tenant admins MUST NOT have plaintext access. The Personal pillar of the dual-context model is preserved as a *structural* property, not a setting.
- **Tenant Outcome 3 — Bidirectional knowledge graph.** Obsidian/Roam-class `[[wikilinks]]` resolve in real time; tag-graph + force-directed graph view render a vault of 5,000 notes in under one second.
- **Tenant Outcome 4 — Capture-everywhere.** First-class browser extension (Chrome MV3 + Firefox MV3 + Safari Web Extensions) clips highlighted text + URL + metadata into the inbox in ≤ 500ms p95.
- **Tenant Outcome 5 — Reminders flow to tasks.** A note line beginning with `- [ ]` becomes a checklist item; a `@due(2026-06-01)` annotation surfaces in the `tasks` µservice via a typed Workflow event.
- **Tenant Outcome 6 — Optional real-time collab.** Two users can open the same Professional-tier note; Loro CRDT (workflow-studio ADR-WS-0001 alignment) reconciles concurrent edits without server-side merge.
- **Tenant Outcome 7 — Portable on day one.** Every note exports to Markdown + frontmatter or JSON Canonical (RFC 8785); every import roundtrips Apple Notes / Evernote ENEX / OneNote / Notion / Bear / Obsidian vault formats.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | end-user | to create a new note with Markdown body + frontmatter | I capture thoughts fast | note-store | Must |
| FR-02 | end-user | to open an existing note in ≤ 50ms p95 (warm) | I do not lose flow | note-store | Must |
| FR-03 | end-user | to multi-tag a note (≤ 256 tags per note) and browse the tag-graph | I find by association | tag-graph | Must |
| FR-04 | end-user | to write `[[wikilink]]` and have backlinks auto-populate bidirectionally | knowledge graph stays consistent | backlink-graph | Must |
| FR-05 | end-user | to auto-create today's daily-note on first access of the day | journaling cadence works | daily-note | Must |
| FR-06 | end-user | to apply a template (meeting-notes / book-notes / recipe / etc.) | structure is reusable | template-gallery | Must |
| FR-07 | end-user | to clip web content via browser extension in ≤ 500ms | capture-everywhere works | web-clipper-bridge | Must |
| FR-08 | end-user | to share a note as read-only link with optional passphrase | sharing is opt-in | share-link | Must |
| FR-09 | end-user | to embed images + videos + files referenced via drive µservice | rich notes | embed | Should |
| FR-10 | end-user | to write `- [ ]` checklist items and convert to tasks | reminders flow | checklist | Must |
| FR-11 | end-user | to render math via `$$LaTeX$$` (KaTeX) | technical notes work | render | Should |
| FR-12 | end-user | to view linear version-history per note + roll back | mistakes are recoverable | version-history | Must |
| FR-13 | end-user | to search across notes (full-text + tag-faceted) | recall works | search | Must |
| FR-14 | end-user | to view a force-directed graph of the vault | structural overview works | graph-view | Should |
| FR-15 | end-user | to import an Obsidian vault / ENEX / Apple Notes export | onboarding works | import | Must |
| FR-16 | end-user | to export to Markdown + frontmatter or JSON Canonical | data portability is real | export | Must |
| FR-17 | end-user | to opt a Professional note into real-time collab (Loro CRDT) | live editing works when wanted | collab-edit | Should |
| FR-18 | end-user | to invoke summarize / tag-suggest / link-suggest on Professional notes | AI assist works when opted in | ai-assist | Should |
| FR-19 | end-user | to switch personal/professional persona | dual-context isolation is preserved | note-store | Must |
| FR-20 | tenant-admin | to configure pack-aware retention bounds for Professional notes | regulatory bounds hold | note-store | Must |
| FR-21 | tenant-admin | to disable AI assist for the tenant or opt-in for Professional notes | governance is configurable | ai-assist | Must |
| FR-22 | Workflow Studio | to consume `NoteCreated` / `NoteTagged` / `ChecklistItemDone` events | downstream automation works | note-store + tag-graph | Must |
| FR-23 | tasks µservice | to receive `ChecklistItemEmitted` events to materialise tasks | note-to-task bridge works | checklist | Must |
| FR-24 | compliance-officer | to issue eDiscovery hold on Professional notes | regulatory request is satisfied | note-store | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p95 | p99 | p999 | Notes |
|---|---|---|---|---|---|
| Note-open (warm) | ≤ 20ms | ≤ 50ms | ≤ 100ms | ≤ 250ms | Postgres + Redis hot-cache; client-side render for E2E |
| Note-create | ≤ 15ms | ≤ 30ms | ≤ 60ms | ≤ 150ms | Insert + audit-chain seal (Professional only) |
| Sync-after-edit (single client) | ≤ 100ms | ≤ 250ms | ≤ 500ms | ≤ 1s | WebSocket delta + persist |
| Tag-search | ≤ 40ms | ≤ 100ms | ≤ 250ms | ≤ 500ms | Tag adjacency index |
| Full-text-search (Professional only) | ≤ 80ms | ≤ 200ms | ≤ 500ms | ≤ 1s | Meilisearch 0.10.0 |
| Graph-render (5k-note vault) | ≤ 400ms | ≤ 1s | ≤ 2s | ≤ 4s | Client-side WebGL force-directed |
| Daily-note auto-create on first access | ≤ 30ms | ≤ 80ms | ≤ 100ms | ≤ 200ms | Template materialisation |
| Web-clipper capture | ≤ 200ms | ≤ 500ms | ≤ 1s | ≤ 2s | Extension → REST → quick ack |
| AI summarize (T1, non-E2E) | ≤ 2s | ≤ 5s | ≤ 8s | ≤ 15s | Foundry-runtime medium model |
| AI tag-suggest (T1, non-E2E) | ≤ 500ms | ≤ 1s | ≤ 2s | ≤ 4s | Foundry-runtime small model |
| AI link-suggest (T1, non-E2E) | ≤ 800ms | ≤ 1.5s | ≤ 3s | ≤ 5s | Foundry-runtime + embedding lookup |

### Security

- **Personal-tier notes are E2E-encrypted by default** (MLS RFC 9420; openmls 0.6 align messenger ADR-MSGR-0002). The Personal-pillar default-ON E2E posture is sharper than docs because notes are first-thought capture (cf. Apple Notes Lockable notes; Standard Notes default).
- **Professional-tier notes are tenant-DEK envelope-encrypted** by default (Bominal ADR-0111); admin disclosure requires four-eyes audit (Bominal ADR-0215).
- **AI assist is STRUCTURALLY forbidden on E2E notes** — server has no plaintext to call against; client-side assist only (and only if SDK opted-in). The Cedar `ai-assist-scope.cedar` policy enforces this as an unconditional `forbid` on `Action::ai_call` over E2E resources. See ADR-NOTES-0005.
- Cedar v4.2 default-deny at every endpoint.
- All WebSocket connections mTLS-terminated; per-tenant API token bound at OpenBao with rotation 30d.
- Web clipper traffic carries a per-installation token bound at extension install; rotation 90d.
- Share-link tokens are 128-bit URL-safe random; optional passphrase via PBKDF2-SHA-256 ≥ 600k iterations (OWASP ASVS v4 §2.4).
- Cross-tier context drift forbidden: a Personal note cannot be converted to Professional (or vice versa) at runtime — context binding is set at note creation and immutable. See `policy/dual-context-isolation.md`.
- FIPS 140-3 validated crypto modules for E2E key material on supported platforms.

### Audit + Compliance

- Every Professional note create / edit / delete / share-link emit / share-link revoke / four-eyes-disclosure event writes an audit-chain record (Merkle / Ed25519 per Bominal ADR-0028).
- Personal-tier notes emit audit-chain records **only for sharing events** (share-link create / revoke / access); routine personal capture does NOT emit (per ePrivacy + privacy-by-design + KR PIPA Art. 23 individual-rights protection).
- Professional-context disclosure (admin reads plaintext for compliance) requires two distinct approving principals + reason code (per Bominal ADR-0215).
- Retention: per-pack bounds in `policy/data-residency.md`. KR PIPA work-mail floor satisfied. GDPR storage-limitation honored. HIPAA pack: PHI retention 6 years where applicable.
- eDiscovery export bundles note + version-history + attachment-refs + audit-chain seal under `runbooks/import-pipeline-failure.md` (referenced inverse direction).

### Availability + SLO

- Availability target: 99.95 % monthly for note-open + note-create.
- Sync-after-edit + graph-render best-effort 99.9 % monthly.
- RTO: ≤ 15 min for note-store. RPO: ≤ 5 min (cross-region replication within pack for Professional store; Personal-tier ciphertext is replication-safe).

### Data residency

- Per-tenant pack pinning per ADR-0117. Personal-context user data follows the personal-residency model (per-user); Professional follows tenant-residency.
- Cross-pack note routing forbidden by default; no federation seam at MVP.

### Accessibility

- WCAG 2.2 AA conformance for editor + graph-view + clipper popup.
- Keyboard-only operation for all primary affordances (capture, tag, link, graph navigation).
- Screen-reader support for graph-view via alternate-list-mode toggle.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename). Layers used: `kernel`, `domain`, `usecase`, `api`, `adapter`, `adapter-postgres`, `adapter-redis`, `adapter-s3`, `adapter-meilisearch`, `adapter-loro`, `adapter-mls`, `rest`, `worker`, `sdk`, `app`.

| BC | Crate family | Purpose | Key entities |
|---|---|---|---|
| `note-store` | `oya-notes-note-store-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,sdk,app}` | Note + notebook CRUD; per-note RBAC; retention policy; eDiscovery hold; context_kind invariant | `Note`, `Notebook`, `NoteVersion`, `RetentionPolicy`, `Hold` |
| `tag-graph` | `oya-notes-tag-graph-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk,app}` | Tag CRUD; multi-tag per note; tag-graph adjacency; tag rename + merge | `Tag`, `TagEdge`, `NoteTag` |
| `backlink-graph` | `oya-notes-backlink-graph-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk,app}` | `[[wikilink]]` parse; bidirectional adjacency materialisation; orphan + dangling-link detection | `Backlink`, `LinkResolution`, `OrphanReport` |
| `daily-note` | `oya-notes-daily-note-{kernel,domain,usecase,api,adapter,sdk,app}` | Per-user daily-note timeline; auto-create on first access | `DailyNote`, `DailyTimeline` |
| `template-gallery` | `oya-notes-template-gallery-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk,app}` | Built-in + user-authored templates; template materialisation | `Template`, `TemplateInstance` |
| `web-clipper-bridge` | `oya-notes-web-clipper-bridge-{kernel,domain,usecase,api,adapter,rest,sdk}` | Browser extension auth; clip ingest; URL-canonicalisation; metadata extract | `ClipperInstallation`, `Clip`, `ClipMetadata` |
| `share-link` | `oya-notes-share-link-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,sdk,app}` | Read-only share tokens; passphrase gating; revocation; access audit | `ShareLink`, `ShareToken`, `ShareAccessEvent` |
| `embed` | `oya-notes-embed-{kernel,domain,usecase,api,adapter,adapter-s3,sdk,app}` | Drive-µservice attachment referencing; client-side render hints | `EmbedRef`, `MimeHint` |
| `checklist` | `oya-notes-checklist-{kernel,domain,usecase,api,adapter,worker,sdk,app}` | Checklist parse from note body; `ChecklistItemEmitted` Workflow events to tasks | `ChecklistItem`, `ChecklistEmission` |
| `version-history` | `oya-notes-version-history-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk,app}` | Linear append-only version timeline; restore-to-version | `NoteVersion`, `VersionPointer` |
| `search-index` | `oya-notes-search-index-{kernel,domain,usecase,api,adapter-meilisearch,worker,sdk,app}` | Professional-tier full-text + tag-faceted; respects Cedar | `SearchDoc`, `SearchFacet` |
| `graph-view-data` | `oya-notes-graph-view-data-{kernel,domain,usecase,api,adapter,sdk,app}` | Server-side graph-data assembly (node + edge JSON); rendering is client-side | `GraphSnapshot`, `GraphEdge` |
| `collab-edit` | `oya-notes-collab-edit-{kernel,domain,usecase,api,adapter,adapter-loro,worker,sdk,app}` | Opt-in Loro CRDT session for Professional notes (E2E refused) | `CollabSession`, `LoroOp`, `Cursor` |
| `import-pipeline` | `oya-notes-import-pipeline-{kernel,domain,usecase,api,adapter,worker,sdk,app}` | Apple Notes / ENEX / OneNote / Notion / Bear / Obsidian vault ingest | `ImportJob`, `ImportSourceFormat` |
| `export-pipeline` | `oya-notes-export-pipeline-{kernel,domain,usecase,api,adapter,worker,sdk,app}` | Markdown + frontmatter + JSON Canonical + PDF emission | `ExportJob`, `ExportFormat` |
| `ai-assist` | `oya-notes-ai-assist-{kernel,domain,usecase,api,adapter,worker,sdk,app}` | T0/T1/T2 summarize, tag-suggest, link-suggest; E2E-refusal invariant | `AssistRequest`, `AssistResult`, `E2ERefusal` |
| `e2e-key-management` | `oya-notes-e2e-key-management-{kernel,domain,usecase,api,adapter,adapter-mls,sdk,app}` | Personal-tier client-derived MLS keys; epoch advance; recovery primitives | `KeyPackage`, `Epoch`, `RecoverySeed` |

Naming justification — `note-store`:

```
NAME: oya-notes-note-store-<layer>
JUSTIFICATION:
- microservice = notes: per ADR-0131 per-microservice flat layout.
- bc-tokens = note-store: primary BC. ADR-0056 v4.1 BC-optionality rule honoured.
- layer = <layer>: ADR-0105 13-value canonical enum; ADR-0106 usecase rename.
- exemptions claimed: -adapter-postgres / -adapter-meilisearch / -adapter-loro / -adapter-mls are canonical *-adapter-<backend> per ADR-0105 Amendment 3.
```

Total crates introduced: **111** across 17 BCs (counts per BC sum to 111).

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated per Bominal ADR-0028):

| Port trait | Kernel crate | Implementation | Data classes touched |
|---|---|---|---|
| `NoteRepository` | `oya-notes-note-store-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT`, `PII_IDENTIFYING` (sometimes), `PHI` (pack-us-healthcare) |
| `TagRepository` | `oya-notes-tag-graph-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT` |
| `BacklinkRepository` | `oya-notes-backlink-graph-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT` |
| `DailyNoteRepository` | `oya-notes-daily-note-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT` |
| `TemplateRepository` | `oya-notes-template-gallery-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT` |
| `ShareLinkRepository` | `oya-notes-share-link-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT`, `AUDIT` |
| `WebClipperGateway` | `oya-notes-web-clipper-bridge-kernel` | `-adapter` (REST) | `BEHAVIORAL_TENANT_PRODUCT` |
| `EmbedResolver` | `oya-notes-embed-kernel` | `-adapter-s3` (drive client) | `BEHAVIORAL_TENANT_PRODUCT`, sometimes `PII_IDENTIFYING` / `PHI` |
| `ChecklistEmitter` | `oya-notes-checklist-kernel` | `-adapter` (Workflow event client) | `BEHAVIORAL_TENANT_PRODUCT` |
| `VersionHistoryStore` | `oya-notes-version-history-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT`, `AUDIT` |
| `SearchIndex` | `oya-notes-search-index-kernel` | `-adapter-meilisearch` | `BEHAVIORAL_TENANT_PRODUCT` |
| `GraphSnapshotBuilder` | `oya-notes-graph-view-data-kernel` | `-adapter` | `BEHAVIORAL_TENANT_PRODUCT` |
| `CollabSessionStore` | `oya-notes-collab-edit-kernel` | `-adapter-loro` | `BEHAVIORAL_TENANT_PRODUCT` |
| `ImportSourceParser` | `oya-notes-import-pipeline-kernel` | `-adapter` (per-format) | `BEHAVIORAL_TENANT_PRODUCT`, `PII_IDENTIFYING` |
| `ExportSinkWriter` | `oya-notes-export-pipeline-kernel` | `-adapter` (per-format) | `BEHAVIORAL_TENANT_PRODUCT` |
| `AssistInvoker` | `oya-notes-ai-assist-kernel` | `-adapter` (foundry-runtime client) | `BEHAVIORAL_TENANT_PRODUCT`, `PII_IDENTIFYING` |
| `E2EKeyStore` | `oya-notes-e2e-key-management-kernel` | `-adapter-mls` (openmls 0.6) | `INTERNAL_ONLY` (no plaintext; keys are client-derived) |
| `CedarNotePolicy` | `oya-notes-note-store-kernel` | `-adapter` (Cedar evaluator) | `INTERNAL_ONLY` |
| `AuditChainClient` | (cross-BC) `-kernel` per BC | (cross-BC) `-adapter` to audit-chain µservice | `AUDIT` |

Data-class enforcement: `oya-check-data-class` LEAN lane refuses unannotated fields.

Cross-product rule: `notes` MUST NOT import any other product µservice crate at any layer. Cross-product flows go through Workflow (events) or Ontology (entity reads/writes). LEAN-A2 lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice notes` — dependency-direction
- `oya gate validate lean-a2 --microservice notes` — cross-product-refusal
- `oya gate validate port-location --microservice notes`
- `oya gate validate layer-correctness --microservice notes`
- `oya gate validate per-microservice-layout --microservice notes`
- `oya gate validate statelessness --microservice notes`
- `oya gate validate shardability --microservice notes`
- `oya gate validate authority-cohesion --microservice notes` (HG-NOTES)
- `oya gate validate dual-context-isolation --microservice notes` (per parallel ADR-0126)
- `oya gate validate e2e-ai-refusal --microservice notes` (NEW; per ADR-NOTES-0005)

## Integration via Workflow + Ontology

### Workflow events produced

| Event type | Trigger | Consumed by | State machine |
|---|---|---|---|
| `NoteCreated` | end-user creates note | search-index (Professional only), backlink-graph, audit-chain (Professional), ontology | append-only |
| `NoteEdited` | end-user edits note | search-index, backlink-graph, version-history, audit-chain (Professional) | append-only delta |
| `NoteDeleted` | end-user / admin deletes | search-index purge, retention-purge worker, audit-chain | tombstone |
| `NoteTagged` / `NoteUntagged` | tag mutation | tag-graph, search-index, downstream workflow engines | append-only |
| `BacklinkResolved` / `BacklinkBroken` | `[[wikilink]]` parsed or orphaned | backlink-graph, graph-view-data, ontology | append-only |
| `ChecklistItemEmitted` | `- [ ]` line parsed | tasks µservice | append-only |
| `ChecklistItemDone` | `- [x]` line toggled | tasks µservice | append-only |
| `ShareLinkCreated` / `ShareLinkRevoked` | share-link action | audit-chain (Personal + Professional) | append-only |
| `ShareLinkAccessed` | external read | audit-chain, share-link store | append-only |
| `DailyNoteCreated` | first-access-of-day auto-create | downstream workflow engines (optional) | append-only |
| `WebClipperCaptured` | extension submits | search-index (Professional), backlink-graph | append-only |
| `ImportJobCompleted` / `ExportJobCompleted` | pipeline finish | audit-chain (Professional), notification | append-only |
| `AiAssistInvoked` / `AiAssistResultDelivered` | T1 call | audit-chain (Professional with consent), evidence-topic | append-only |
| `FourEyesDisclosureExecuted` (Professional only) | admin pair approves PII read | audit-chain | append-only |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `OntologyEntityChanged` (Person/Tag/Notebook) | ontology | tag-graph, backlink-graph | refresh resolution cache |
| `TenantRetentionPolicyUpdated` | tenancy | note-store | reassign retention bounds |
| `DriveAttachmentRevoked` | drive | embed | mark embed-ref broken |
| `TaskCompleted` | tasks | checklist | re-render checklist item state |
| `AuditChainSealed` | audit-chain | (read-only) | confirm audit-write durability |

### Ontology writes

| Object Type | Written by BC | Audit trail |
|---|---|---|
| `Note{note_id, tenant_id, context_kind, title, tag_refs, retention_policy_id}` | `note-store` | Ed25519 (Professional only) |
| `Tag{tag_id, tenant_id, name, color, parent_tag_id}` | `tag-graph` | Ed25519 (Professional only) |
| `Backlink{from_note_id, to_note_id, kind: explicit | tag}` | `backlink-graph` | Ed25519 (Professional only) |

Personal-tier writes to Ontology are MINIMAL by design (only opaque `Note{note_id, user_id, context_kind: Personal}` — no title, no body, no tags) per parallel ADR-0126 + DCI-05.

### Ontology reads

| Object Type | Read by BC | Query shape |
|---|---|---|
| `Person`, `Team` | `share-link`, `ai-assist` | `find_by(@-handle, tenant_id)` |
| `RetentionPolicy` | `note-store` | `lookup(tenant_id, context_kind)` |
| `TenantContext` | every BC | tenant scope + pack jurisdiction lookup |
| `DriveAttachment` | `embed` | `lookup(blob_ref, tenant_id)` |

## Competitive Benchmark

| Competitor | Product / feature | Parity dimensions | Primary source |
|---|---|---|---|
| Apple Notes | iCloud Notes + Lockable notes | E2E-when-locked; folders; tags; freeform sketch | `support.apple.com/notes` |
| Google Keep | Color-coded notes + tags + reminders | tag + reminder model; mobile-first | `support.google.com/keep` |
| Microsoft OneNote | Notebooks + sections + pages + ink | structured hierarchy; OneDrive sync; ink | `support.microsoft.com/onenote` |
| Notion (quick-note subset) | Quick capture + databases + blocks | block model; database queries | `notion.so/help` |
| Bear | Markdown + tags + nested tags | Markdown-first; tag-tree | `bear.app/faq` |
| Obsidian | Local-first Markdown vault + `[[wikilinks]]` + graph view | bidirectional links; graph view; plugin ecosystem | `obsidian.md/help` |
| Standard Notes | E2E-by-default Markdown notes | default-on E2E; extensions; minimal | `standardnotes.com/help` |
| Evernote | Notebooks + tags + web clipper + search | web clipper UX; OCR-on-images; search | `evernote.com/help` |
| Roam Research | Bidirectional outliner + block-references | block-level references; daily notes | `roamresearch.com/help` |
| Logseq | Local-first outliner + Markdown + graph | block-level + outliner + Markdown duality | `docs.logseq.com` |
| Joplin | OSS Markdown notes + sync + E2E | E2E sync; multi-backend | `joplinapp.org/help` |
| Simplenote | Plaintext notes; minimal | minimalism; speed; sync | `simplenote.com/help` |
| Drafts | Capture-first iOS/macOS | capture-first; actions | `getdrafts.com/help` |
| Craft Docs | Block-based; rich-text; export | clean block model; export | `craft.do/help` |
| Reflect | Networked notes + AI | AI tag + link suggest; daily notes | `reflect.app/help` |
| Heptabase | Whiteboards + cards + notes | spatial; card model | `heptabase.com/help` |
| Tana | Supertags + outliner | tag-as-type model | `tana.inc/help` |
| Mem | AI-organised notes | AI-first organisation | `mem.ai/help` |
| Saga | Block + connected notes | links + tasks together | `saga.so/help` |
| NotePlan | Markdown + tasks + calendar | task-integrated notes | `noteplan.co/help` |
| Boost Note | OSS Markdown notes | OSS Notion-alike | `boostnote.io/help` |

Key parity gaps to close (ordered by priority):

1. **E2E-by-default on Personal tier as a structural property** — Standard Notes does this; Apple Notes does this for "Lockable" subset; most others do not. Target: compile-time + LEAN-lane enforcement with Cedar `forbid` on AI calls over E2E.
2. **Bidirectional link + graph view + daily-note triad as first-class** — Obsidian / Roam / Logseq own this; Apple Notes / Keep do not. Target: real-time backlink resolution + 5k-note graph render ≤ 1s p95.
3. **Native Workflow + Ontology integration** — competitors expose iCloud sync / Markdown files. Target: typed Workflow events + Ontology object writes.
4. **OpenSLO + agentic gate** — none of the competitors gate feature rollouts on SLO compliance.
5. **Multi-pack residency + per-pack regulatory overlays** — none of the consumer-grade notes apps do this.
6. **AI-refusal on E2E** — Mem / Reflect do AI; none cleanly refuse on E2E. Target: structural impossibility + audit-chain proof.

## Performance Targets

(See Performance NFR table above.)

Error budget:
- Monthly error budget for note-open: 0.05 % (≈ 22 min/month).
- Burn-rate alarm on `notes.note-open.availability` is 14.4× burn rate over 1h.
- Error budget policy: `microservices/notes/runbooks/error-budget-policy.md` (deferred to follow-on IP).

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `mixed`. Postgres for note metadata + non-E2E body; S3 for attachments + E2E ciphertext; Redis for sync-session + hot-cache; Meilisearch for non-E2E full-text; Loro CRDT for opt-in collab.

**Active-active compatibility**: stateless REST + Postgres logical-replicated within pack; Redis primary-replica HA; S3 cross-AZ replication.

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Active notes accounts | 50k | 500k | API CPU > 70 % |
| Notes/sec creates | 1k | 20k | Postgres write IOPS > 70 % |
| Notes per user | 1k | 100k | per-user vault cardinality limit hit |
| Tags per tenant | 100k | 10M | tag-graph adjacency cardinality |
| Attachments/day | 100k | 1M | drive-µservice rate-limit |
| Search index size (Professional only) | 50GB | 2TB | shard count exceeded |

Scale-out policy:
- HPA on REST pods: CPU > 70 %, min 4, max 100 replicas.
- Postgres shard-by-tenant once cell hits 500k notes/sec aggregate.
- Redis cluster sharding by `(tenant_id, user_id) mod N`.

Sharding:
- Note metadata partitions by `(tenant_id, user_id, year-month)` for Personal; `(tenant_id, notebook_id, year-month)` for Professional.
- Tag adjacency partitions by `(tenant_id, tag_id)`.
- Backlink adjacency partitions by `(tenant_id, from_note_id)`.
- `oya-check-shardability-cli` lane verifies partition keys are present in every kernel struct.

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | A note-create + tag + `[[wikilink]]` + backlink roundtrip completes within p99 < 100ms note-open | `microservices/notes/tests/e2e/note-create-link.rs` |
| AC-02 | A Personal-tier note cannot be promoted to Professional at runtime | `tests/e2e/dual-context-isolation.rs` |
| AC-03 | AI summarize / tag-suggest / link-suggest invoked on a Personal (E2E) note returns 403 + emits `oya_notes_ai_call_blocked_e2e_total` | `tests/e2e/ai-e2e-refusal.rs` |
| AC-04 | Web-clipper capture from Chrome MV3 extension finishes p95 < 500ms | `tests/e2e/web-clipper-capture.rs` |
| AC-05 | Daily-note auto-creates on first access of the day with template applied | `tests/e2e/daily-note-auto.rs` |
| AC-06 | Graph-view renders 5k-note vault p95 < 1s (client-benchmark) | `tests/e2e/graph-render-5k.rs` |
| AC-07 | Import an Obsidian vault (1k notes + tag-graph + `[[wikilinks]]`) and roundtrip export-import equivalence | `tests/e2e/obsidian-vault-roundtrip.rs` |
| AC-08 | Import an Evernote ENEX archive (10MB + 500 notes) | `tests/e2e/enex-import.rs` |
| AC-09 | Share-link created with passphrase requires passphrase + invokes share-access audit-chain seal | `tests/e2e/share-link-passphrase.rs` |
| AC-10 | Professional note four-eyes disclosure requires two distinct approving principals + audit-chain seal | `tests/e2e/four-eyes-disclosure.rs` |
| AC-11 | `oya gate validate per-microservice-layout --microservice notes` exit 0 | ADR-0131 lane |
| AC-12 | `oya gate validate authority-cohesion --microservice notes` exit 0 | ADR-0133 lane; HG-NOTES registered |
| AC-13 | `oya gate validate dual-context-isolation --microservice notes` exit 0 | per parallel ADR-0126 |
| AC-14 | `oya gate validate e2e-ai-refusal --microservice notes` exit 0 | per ADR-NOTES-0005 |
| AC-15 | Loro collab session: two clients edit same Professional note concurrently; converged state matches reference implementation | `tests/e2e/loro-collab-convergence.rs` |
| AC-16 | Markdown export round-trips via JSON Canonical (RFC 8785) byte-identical when re-exported | `tests/e2e/export-roundtrip-canonical.rs` |

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | Client-side encrypted-search index design — encrypted-inverted-index in IndexedDB with per-note token-bloom-filters vs. trapdoor-permutation OPE — pick a primary | council-privacy + axis-notes | closed in ADR-NOTES-0004 |
| 2 | AI tag-suggest semantics — multi-label classification head vs. retrieval-over-existing-tag-vocab — pick a primary | axis-foundry-runtime + axis-notes | open; follow-on capability ADR |
| 3 | Web-clipper extension distribution — Chrome Web Store + Firefox AMO + Safari Web Extensions + Edge Add-ons all on day 1, or staged | axis-notes + ops-security | follow-on IP |
| 4 | Daily-note timezone authority — user-local vs. tenant-default — pick a primary | axis-notes | open; UX research follow-on |
| 5 | Public-share-link OG metadata leakage — should the OG preview render a snippet of the note body or only the title — pick a primary | council-privacy | follow-on policy decision |
| 6 | Mobile-platform offline-edit conflict semantics — last-writer-wins vs. CRDT-merge-on-reconnect for non-collab notes | axis-notes | follow-on ADR |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0008 | Data Use Boundary | personal/professional data-use invariants |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0063 | Documentation suite coverage | doc-coverage CI lane |
| ADR-0064 | Canonical base + localization packs | pack pattern |
| ADR-0105 | 13-layer enum | layer authority |
| ADR-0106 | application → usecase | layer naming |
| ADR-0117 | Data residency packs | residency authority |
| ADR-0126 | Connect dual-context (parallel) | dual-context isolation source |
| ADR-0130 | Agentic SLO-gated promotion | gates notes releases |
| ADR-0131 | Per-microservice flat layout | this PRD authored under it |
| ADR-0132 | Suite-and-bundle dissolution | factored Connect into surfaces |
| ADR-0133 | Industry best-practice conformance | HG-NOTES under this |
| ADR-NOTES-0001 | E2E encryption default on Personal tier | sealed by this PRD's NFR + DCI-03 |
| ADR-NOTES-0002 | Bidirectional link + graph storage | sealed by FR-04 + FR-14 |
| ADR-NOTES-0003 | CRDT library for optional collab | sealed by FR-17 (Loro 1.x) |
| ADR-NOTES-0004 | Search architecture respecting E2E | sealed by FR-13 + Open Q #1 |
| ADR-NOTES-0005 | AI assist bounds + E2E invariant | sealed by FR-21 + NFR Security + AC-03 |
| ADR-NOTES-0006 | Portable export + import format | sealed by FR-15 + FR-16 |
| ADR-MSGR-0002 | Messenger E2E key escrow tier-split | paired privacy posture |
| ADR-MAIL-0001 | Mail Personal-pillar E2E key recovery | paired Personal-pillar privacy |
| ADR-WS-0001 | Workflow-studio Loro CRDT (sibling) | Loro alignment source |
| Bominal ADR-0028 | Audit-chain Merkle + Ed25519 | inherited |
| Bominal ADR-0111 | Ciphertext property type + envelope encryption | inherited |
| Bominal ADR-0208 | Connect dual-context unified channel hub | inherited via ADR-0126 |
| Bominal ADR-0215 | Connect retention legal-hold dual-context | inherited via ADR-0126 |
