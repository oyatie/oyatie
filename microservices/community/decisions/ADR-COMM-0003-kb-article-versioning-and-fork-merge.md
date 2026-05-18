---
id: ADR-COMM-0003
status: Accepted
date: 2026-05-17
microservice: community
deciders: axis-community, council-architecture, council-privacy, axis-ontology
owner: axis-community
supersedes: []
superseded_by: []
related:
  - ADR-0028
  - ADR-0105
  - ADR-0135
  - ADR-0131
  - ADR-0132
related_artifacts:
  - microservices/community/PRD.md (FR-03, FR-11, §"Audit + Compliance")
  - microservices/community/PHASE-01-COMMUNITY-SUBSTRATE.md (IP-008 kb-article-store)
  - microservices/community/IP-008-kb-article-store-s3.md
  - microservices/community/capabilities/post-create.yaml
  - microservices/community/policy/tenant-scope.cedar
purpose: Close PRD-community FR-03's open versioning question — fix the canonical KB article versioning model (Wikipedia-style immutable revision history with tenant-scoped editorial review), explicitly rejecting Git-style branch/PR/merge as out of scope for the M02 surface.
---

# ADR-COMM-0003: KB article versioning — Wikipedia-style immutable revision history with tenant-scoped editorial review; no branch/PR/merge

## Status

Accepted — 2026-05-17.

## Context

PRD-community FR-03 commits the µservice to "long-form curated content with attachment store (S3-backed), revisions, and tenant-controlled publication review" — replacing Confluence-class workflows. FR-11 commits to cross-product ontology links inside KB articles. The PRD does not, however, fix the *versioning model* — and the model determines the entire shape of the BC (`kb-article-store`).

Three versioning shapes compete:

1. **Wikipedia-style immutable revision history**: every edit is a new revision row; revisions are immutable; current revision is the "head pointer"; rollback = advance head pointer backwards; merge = N/A (no branching); review = pre-publication state machine (Draft → Pending Review → Published → Archived).
2. **Notion-style page versioning**: a page is a tree of blocks; each edit mutates blocks; "page history" is a synthetic playback of mutations; no branching; less auditable because the block-mutation log is internal.
3. **Git-style branch / fork / pull-request / merge**: articles are forkable; collaborators branch + edit + propose merge; merge requires conflict resolution; head-of-main is the "published" version.

Forces:

- **Auditability + audit-chain seal compatibility**: PRD §"Audit + Compliance" requires every `KBArticlePublished` event to be sealed within 1 s. Wikipedia-style is trivially compatible (each revision is an event). Notion-style requires synthesizing a sealable event from a block-mutation diff, which is messy. Git-style requires per-merge event + per-fork event + per-branch event, which multiplies the audit-chain QPS.
- **Tenant editorial review**: PRD demands "tenant-controlled publication review" — i.e., a draft → review → published state machine. Wikipedia + Notion both support; Git-style requires forking the review concept into "MR approved" which is an additional concept tenants must learn.
- **Concurrent edit conflicts**: KB articles in tenant settings are *occasionally* concurrently edited (tens-of-edits-per-day, not GitHub-pull-request scale). Wikipedia-style uses last-write-wins per revision with a conflict-detected warning at submit time. Notion-style uses CRDT (Yjs / Automerge) for live collaborative edit. Git-style demands merge-conflict UX.
- **Cross-product ontology links (FR-11)**: links are stored on the revision, not the article; rebinding links during a rollback must be deterministic. Wikipedia-style is trivially deterministic. Git-style across-branch link rebinding is genuinely hard (which branch's link wins?).
- **Industry pattern fit**: oyatie is replacing Confluence-class workflows, not GitHub-class workflows. Confluence + Notion + Wikipedia all use revision-history-without-branching. GitHub-style KB is a niche pattern (mostly for technical documentation in software shops).
- **Storage model fit with IP-008**: IP-008 commits S3-backed attachment store with resumable multipart upload. Revision-store-as-rows + attachment-pointer-as-FK is a clean PostgreSQL+S3 separation. Git-style would push toward content-addressed object storage (git-LFS analogue), which is over-engineered for the use case.

## Decision

The KB article store ships a **Wikipedia-style immutable revision history with tenant-scoped editorial review**:

1. **Article entity** in `oya-community-kb-article-store-kernel`:
   ```
   Article {
     id: ArticleId,
     tenant_id: TenantId,
     space_id: SpaceId,
     current_revision_id: RevisionId,     // head pointer
     publication_state: PublicationState, // Draft | PendingReview | Published | Archived
     created_at: Timestamp,
     updated_at: Timestamp,
   }
   Revision {
     id: RevisionId,
     article_id: ArticleId,
     parent_revision_id: Option<RevisionId>,  // linear chain; never branches
     author_id: AuthorId,
     body_markdown: String,
     ontology_link_ids: Vec<OntologyLinkId>,
     attachment_ids: Vec<AttachmentId>,
     reviewer_id: Option<ReviewerId>,
     created_at: Timestamp,
     // IMMUTABLE after insert
   }
   ```

2. **Publication state machine** in `-domain` (typed; transitions are not free-form):
   ```
   Draft ──submit──▶ PendingReview ──approve──▶ Published
     ▲                    │                          │
     │                    └──reject──▶ Draft         │
     │                                               │
     └───────────── archive ────────────── Archived ◀┘
   ```
   - `submit` requires `current_revision.author_id == submitter_id` OR `submitter has editorial_role`.
   - `approve` requires `submitter_id != reviewer_id` (two-eyes for publication) + Cedar `Action::"approve_kb_article"`.
   - `archive` requires Cedar `Action::"archive_kb_article"`.
   - Every state transition emits an audit-chain seal (`KBArticlePublished`, `KBArticleRejected`, `KBArticleArchived`).

3. **Revision append is the only write verb**. There is no "edit revision" verb. To change the head, you append a new revision and advance the head pointer. The previous revision is permanently retrievable.

4. **Rollback** is implemented as "append a new revision whose body == revision N's body, with `revert_of: revision_n_id` metadata, then advance head." This preserves the audit-chain (the rollback is itself a sealed revision event) and the ontology-link-rebinding is automatic because links live on the rolled-forward revision.

5. **Concurrent edit conflicts** are detected by optimistic concurrency control:
   - Edit request carries `parent_revision_id` of the revision the editor branched from.
   - Submit fails if `parent_revision_id != article.current_revision_id` → the editor is shown the latest revision and asked to re-resolve.
   - This is the Wikipedia "edit conflict" UX; familiar to anyone who has ever edited a wiki.

6. **No branching, no forking, no merging.** The decision is explicit: if a tenant needs collaborative parallel editing on the same article, they create a draft article (own ID, own revision chain) and reference the original article in body. Merging two articles is a manual content operation (copy-paste) followed by archiving the source.

7. **Attachments are immutable per revision.** Adding or removing an attachment requires a new revision. S3 objects are content-addressed by hash; deduplication across revisions happens at the S3 layer; no attachment is mutated in place.

8. **Cross-product ontology links** are revision-scoped. Rolling back a revision rolls back its link set. Links are first-class entities in `oya-ontology` resolved by ID; only the binding is article-revision-scoped.

9. **CRDT for live-edit cursor** is out of scope at M02. The Wikipedia conflict-detection-at-submit model is sufficient for tenant-scale concurrent-edit volume (≤ tens-per-day-per-article).

## Alternatives Considered

### A. Notion-style block-mutation page versioning
- Pros: live-collaborative-edit UX (cursors, comments-on-blocks); modern feel; large knowledge-worker market familiarity.
- Cons: block-mutation log is internal, not auditable as discrete events; sealing into the audit-chain is messy (every mutation? every save? every minute?); ontology-link rebinding during rollback is intractable because links live on blocks that may have been moved / split / merged; CRDT (Yjs) implementation cost is high; storage cost is high (every block is its own row).
- Rejected: audit-chain incompatibility + ontology rebinding complexity outweigh UX benefit.

### B. Git-style branch / fork / pull-request / merge
- Pros: powerful collaboration; perfect for documentation-in-code; software-engineering-team-familiar.
- Cons: tenants are not software-engineering teams (mostly); merge-conflict UX is a known UX cliff for non-technical users; pull-request lifecycle is an additional concept; cross-branch ontology-link rebinding is genuinely undecidable; storage model wants content-addressed objects which is incompatible with the IP-008 Postgres+S3 separation; ~3× implementation cost vs. Wikipedia-style.
- Rejected: implementation cost + UX mismatch vs. tenant audience.

### C. Confluence-style flat document with manual version-history dropdown (no immutability)
- Pros: simplest; tenants familiar with Confluence; minimal storage.
- Cons: edits are mutable in place; audit-chain seal is "save event" not "revision event" which is less precise; rollback semantics are fuzzy (does it restore deleted attachments? what about ontology links?); not actually competitive with the Wikipedia / Notion frontier.
- Rejected: under-engineered; loses to Wikipedia-style on auditability for marginal simplification.

### D. Append-only event-sourced edit log (CRDT-as-log, with materialised view as the current article)
- Pros: maximally auditable (every keypress is an event); CRDT means concurrent edits "just merge"; perfect replay.
- Cons: every-keypress event volume is enormous; ontology link bindings are a function-of-folded-log which is hard to reason about; sealing per keystroke into the audit-chain is impossible (would melt the audit-chain); over-engineered for tenant KB use case.
- Rejected: scale + audit-chain throughput mismatch.

## Consequences

### Positive

- Audit-chain seal model is natural: each revision is one event; FR-03 + §"Audit + Compliance" satisfied directly.
- Storage model is clean: Postgres row per revision, S3 object per attachment, FKs between them; IP-008 stays as written.
- Rollback semantics are trivially defined: "advance head pointer" + "current revision is a revert-of marker"; no special-case code.
- Ontology link rebinding during rollback is deterministic (links live on revision).
- Tenant editorial review (Draft → Pending → Published) is industry-canonical (Wikipedia, Confluence-with-workflow, Notion); tenants do not need to learn a new mental model.
- No CRDT complexity at M02; can be added later as a separate adapter without changing the kernel data model.

### Negative

- Concurrent live-edit UX (multiple cursors in the same article) is *not* supported. Mitigated by submit-time conflict detection (Wikipedia-style) which is acceptable for tenant-scale concurrent-edit volume.
- Branch / fork / merge use cases (parallel proposals for the same article) require manual workarounds (draft second article, reference original, archive after merge). Tenants used to GitHub-class workflows may complain. Accepted; if demand emerges, ADR-COMM-NNNN can re-open with branch support as an additive feature.
- Storage grows linearly with revision count. Mitigated by a 7-year revision-retention cap (per PRD §"Audit + Compliance" KB articles indefinite + revisions sealed → revision-bodies older than 7 y compress to short-form summary; the audit-chain seal of the original revision remains).

### Operational

- Cargo workspace: `Article`, `Revision`, `PublicationState` types in `oya-community-kb-article-store-kernel`; state-machine transitions in `-domain`; submit/approve/reject/archive use cases in `-usecase`; Postgres schema in IP-008 (already authored).
- New runbook `runbooks/kb-attachment-restore.md` (already exists) covers attachment restore after S3 outage; documents that the revision row is the source-of-truth and attachments dereference from there.
- Dashboards: pending-review depth panel added to a future `dashboards/kb-article-pipeline.json`.
- CI lane `community-kb-revision-immutability`: BLOCKS PRs that allow `UPDATE community.kb_article_revisions` after insert.

### Regulatory

- **GDPR Art. 17 right to erasure**: when a tenant member exercises erasure, the author_id field is tombstoned to `redacted:<hash>` across all their revisions; the revision body is *not* deleted (it is community-contributed content under tenant licence). The audit-chain seal records the tombstoning event.
- **EU DSA Art. 14** internal complaint mechanism: the publication-state machine's `reject` transition is the analogue of a moderator denial; appeals re-enter at `PendingReview` with a different reviewer.
- **HIPAA 45 CFR §164.312** (when pack-us-healthcare is active): KB articles containing PHI are flagged at submit time; review must be performed by a HIPAA-trained reviewer; the audit-chain seal records the reviewer's HIPAA-role membership.
- **KR PIPA Art. 21** (retention): pack-kr default retention is indefinite for KB articles + their revisions; bulk erasure on tenant churn requires the same per-revision tombstoning treatment.

## References

- Wikipedia revision history specification — `https://en.wikipedia.org/wiki/Help:Page_history`
- Wikipedia edit-conflict UX — `https://en.wikipedia.org/wiki/Help:Edit_conflict`
- Notion-style block versioning (architectural blog) — `https://www.notion.com/blog/data-model-behind-notion`
- Git data model + branching (Pro Git book, Ch. 10) — `https://git-scm.com/book/en/v2/Git-Internals-Git-Objects`
- Confluence version-history feature documentation — `https://confluence.atlassian.com/`
- CRDTs (Conflict-free Replicated Data Types) reference — Shapiro et al., INRIA — `https://hal.inria.fr/inria-00609399`
- Yjs / Automerge CRDT libraries
- ADR-0028 — audit-chain sealing
- ADR-0135 — Connect-unbundle
- ADR-0131 — Per-microservice flat layout
- `microservices/community/PRD.md` FR-03, FR-11
- `microservices/community/IP-008-kb-article-store-s3.md`
- `microservices/community/policy/tenant-scope.cedar`
