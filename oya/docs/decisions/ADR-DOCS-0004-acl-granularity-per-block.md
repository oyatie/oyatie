---
id: ADR-DOCS-0004
title: ACL granularity — per-block (Notion-style); whole-doc + named-range comments only (Google-Docs style) rejected
microservice: docs
status: Accepted
date: 2026-05-17
owner: axis-docs + ops-security
deciders: council-architecture, axis-docs, ops-security, council-privacy
supersedes: []
superseded_by: []
related: [ADR-0140 (retired per ADR-0145), ADR-0131, ADR-DOCS-0002]
related_artifacts:
  - microservices/docs/PRD.md (FR-07, FR-08, AC-04)
  - microservices/docs/policy/tenant-scope.cedar (per-block ACL section)
  - microservices/docs/policy/editor-isolation.md (per-block ACL enforcement layers)
  - microservices/docs/IP-010-sharing-and-permissions.md
purpose: |
  Settle ACL granularity: per-block (Notion-class differentiator) vs whole-doc
  (Google Docs / Word baseline). Closes PRD AC-04 + FR-07 + FR-08.
doc_status: published
---

# ADR-DOCS-0004: ACL granularity — per-block

## Status

Accepted — 2026-05-17.

## Context

The docs µservice's sharing-and-permissions BC governs who can read, comment, suggest, or edit a document. Three industry models exist:

1. **Whole-doc ACL** (Google Docs, Microsoft Word, Coda, Quip, Confluence, HackMD). Permissions apply to the entire document. Comments + suggestions reference text ranges within the doc but cannot have independent visibility.

2. **Per-block ACL** (Notion-only at production scale; partial in Coda). Each block carries its own ACL; private blocks within a shared doc are not visible to less-privileged grantees.

3. **Per-paragraph ACL with named ranges** (no production example). Conceptually possible but no major product implements it.

Per PRD §"Tenant Outcome 7" + PRD AC-04, the docs µservice positions itself as Notion-class. The per-block ACL is the load-bearing differentiator vs Google Docs (which has whole-doc only). Tenants who chose docs over Google Docs typically cite per-block ACL as a primary reason (per gtm-customer-success tenant interviews 2025-Q4).

Performance budget: per-block ACL is evaluated at every block-tree read. PRD §"Performance" requires `doc-open (warm) p99 ≤ 100ms` which budgets ≤ 50ms for ACL eval on a 100-block doc. Cedar policy eval is the bottleneck; caching is required.

Security: ACL bypass on any single block constitutes information disclosure (PRD §threat-model T-I-01). The invariant must be tested at every read path.

## Decision

Adopt **per-block ACL** as the canonical model:

### ACL model

Each block carries an `acl` field:

```rust
pub struct BlockAcl {
    visibility: BlockVisibility,           // Public | TeamVisible | Private
    principals: Vec<UserId>,               // explicit allow-list when Private
    teams: Vec<TeamId>,                    // explicit team allow-list when TeamVisible
    inherited_from_doc: bool,              // default true; false when explicit override
}
```

### Defaults

- `visibility = inherited_from_doc` for new blocks (matches parent doc's whole-doc ACL).
- Explicit override per block: tenant author marks a block `private` or `team_visible`.
- Doc-level ACL still exists (controls who can open the doc at all); per-block ACL refines within that admitted set.

### Enforcement layers (defence-in-depth)

| Layer | Mechanism | Refusal at |
|---|---|---|
| Cedar `tenant-scope.cedar` per-block-acl clauses | refuses block reads when principal not in block's `principals` | API request |
| Postgres RLS at block-level | predicate filters block-tree rows by per-block ACL | DB query |
| Application server-side stamping | block ACL never client-mutable | API request |
| LEAN check `oya-check-per-block-acl` | validates every block-read path applies ACL | PR time |
| Annual pen-test | red-team attempts per-block ACL bypass | Annually |

### Performance optimisation

- Per-(doc_id, principal_id) ACL projection cache in Valkey with TTL 5min + jitter.
- Cache invalidated on grant change via Workflow event subscription.
- Single-flight per (doc_id, principal_id) prevents cache-miss storm.
- p99 budget ≤ 50ms even at cold-cache; `oya-docs-per-block-acl-check-p99` SLO.

### Comment + suggestion inheritance

Comments + suggestions on a block inherit that block's ACL. A comment thread on a private block is visible only to principals with read access to the block.

### Cross-µservice embed-resolver ACL passthrough

Per the embed-resolver BC: when a doc embeds a workflow-studio canvas, the embed-resolver evaluates the SOURCE-side ACL using the embedding doc's principal. The source µservice (workflow-studio) returns 403 or redacted placeholder if denied. Per-block ACL on the embed's parent block does not bypass source-side ACL — both must admit.

## Alternatives Considered

### Alternative A — Whole-doc ACL only (Google Docs / Microsoft Word model)

- **Pros**:
  - Lower implementation cost.
  - Lower per-read evaluation cost (single ACL check per doc).
  - Industry-standard model; minimal user-onboarding friction.
- **Cons**:
  - **Loses primary differentiator vs Google Docs**: tenants chose docs for this exact capability.
  - **No way to share a doc but hide salary lines / IP sections / customer names**: very common HR-doc and design-doc use case.
  - **Per-block ACL is impossible to retrofit**: starting with whole-doc and adding per-block later breaks every consumer that pattern-matched on whole-doc semantics. Better to start strict.
- **Rejected reason**: loses differentiator + forecloses future use cases.

### Alternative B — Whole-doc ACL with named-range comment visibility (extension of Google Docs)

- **Pros**:
  - Lower implementation cost than full per-block.
  - Some flexibility (private comment threads).
- **Cons**:
  - Half-measure: doesn't admit per-block visibility of the content itself; only of the comment overlay.
  - Doesn't satisfy HR-doc / design-doc partial-visibility use cases.
  - Doesn't enable Notion-style per-page-with-private-subsections workflows.
- **Rejected reason**: insufficient flexibility; still loses differentiator.

### Alternative C — Per-section ACL (heading-aware)

- **Pros**:
  - Lower granularity than per-block but more flexible than whole-doc.
  - Heading-section boundary is a natural unit.
- **Cons**:
  - "Section" is not addressable by a stable ID; it's defined by heading hierarchy, which changes with edits.
  - Cannot hide a specific table row or callout without converting it to its own section (clunky UX).
  - Notion's per-block + nested-block model is more flexible at marginal additional cost.
- **Rejected reason**: addressing fragility; awkward UX; less flexible.

### Alternative D — Per-block ACL with scheduled-for-distinct-tracked-work enforcement (advisory only)

- **Pros**:
  - Allows shipping the data model now; enforcement enabled later.
- **Cons**:
  - Trains tenants on a non-load-bearing surface.
  - Creates an apparent privacy claim that isn't actually enforced; security regression risk if shipped to GA.
- **Rejected reason**: dishonest; if we ship per-block ACL in the UI, we must enforce it.

## Consequences

### Architectural

- `oya-docs-sharing-and-permissions-kernel` declares `BlockAcl` entity + `AclRepository` port.
- `oya-docs-block-types-kernel` includes `acl: BlockAcl` field on every `Block`.
- Postgres `blocks` table has per-row `acl_visibility`, `acl_principals` (jsonb array), `acl_teams` (jsonb array), `acl_inherited_from_doc` columns + RLS predicate.
- Cedar policy `tenant-scope.cedar` `per-block-acl` clauses (see policy file).
- LEAN check `oya-check-per-block-acl` validates application paths.

### Downstream impact

1. **PRD AC-04** — directly satisfied (verified by property test `cargo nextest run -p oya-docs-sharing-and-permissions-domain -- per_block_acl`).
2. **ADR-DOCS-0002 block-type system** — every block has an `acl` field; without per-block ACL the field would be useless.
3. **embed-resolver source-side ACL passthrough** — the per-block ACL is one of two ACL layers (the other is the source µservice's own ACL); both evaluated at fetch time.
4. **Comments + suggestions** — inherit parent block's ACL.
5. **Export pipeline** — exports respect per-block ACL of the exporting principal; private blocks redacted in export.
6. **search-within-doc** — per-block ACL filters search index per principal.
7. **`runbooks/share-acl-drift.md`** — handles per-block ACL bypass forensics.

### SLOs gaining new dimensions

- `docs.per_block_acl_check_seconds` — p99 ≤ 50ms.
- `docs.per_block_acl_violation_total` — Sev-1 if non-zero (mirrors workflow-studio AC-06 pattern for CRDT no-silent-loss).
- `docs.share_acl_enforcement_correctness` — 100% target invariant (per `slos/share-acl-enforcement-correctness.openslo.yaml`).

### Performance + cost

- Per-(doc_id, principal_id) ACL projection cache adds ~5% overhead per cell ($0.000002 per check; per `cost-budget.md`).
- Postgres index on `(tenant_id, document_id, acl_visibility)` accelerates per-block ACL queries.

### Risk register

- **Risk**: Per-block ACL eval slow on documents with > 10k blocks. **Mitigation**: cache projection per (doc, principal); single-flight; LEAN performance lane.
- **Risk**: UX confusion (tenants don't know which blocks are private). **Mitigation**: council-design-system surfaces visual indicator (badge / overlay) per `PRD §Open Questions #3`.
- **Risk**: Bypass via cross-µservice embed (embedded canvas leaks content). **Mitigation**: source-side ACL passthrough invariant (this ADR §"Cross-µservice embed-resolver ACL passthrough" + LEAN check `oya-check-embed-resolver-acl-passthrough`).

## References

- ADR-0140 (Cedar policy substrate).
- ADR-0131 (per-microservice layout).
- ADR-DOCS-0002 (block-type system; block-as-acl-unit).
- PRD `microservices/docs/PRD.md` FR-07, FR-08, AC-04.
- `microservices/docs/policy/tenant-scope.cedar` (per-block ACL Cedar rules).
- `microservices/docs/policy/editor-isolation.md` (per-block ACL enforcement layers).
- `microservices/docs/slos/share-acl-enforcement-correctness.openslo.yaml`.
- Notion API permissions — `developers.notion.com/reference/page` (per-block ACL reference).
- Google Docs sharing model — `developers.google.com/drive/api/guides/about-permissions` (whole-doc reference; rejected).
