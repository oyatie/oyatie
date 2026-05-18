---
id: ADR-SITES-0005
status: Accepted
date: 2026-05-17
microservice: sites
deciders: axis-sites, council-architecture
owner: axis-sites
supersedes: []
superseded_by: []
related:
  - ADR-0056
  - ADR-0105
  - ADR-0131
  - ADR-0133
  - ADR-SITES-0001
related_artifacts:
  - microservices/sites/PRD.md §FR-11, AC-04
  - microservices/sites/IP-009-cms-collection.md
  - microservices/sites/contracts/openapi/sites.yaml (CollectionType, Entry schemas)
purpose: |
  Define the CMS-collection data model. CMS-collection (e.g., Article,
  Product, Team Member) is the structured-content surface; tenants
  model schemas + entries + relationships.
---

# ADR-SITES-0005: CMS-collection data model — hybrid portable-text + relational; Meilisearch for search; Sanity-only rejected; Strapi-only rejected

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

CMS-collection is a Must-have FR per PRD-sites §FR-11: tenants define
typed collections (e.g., Article, Product, Team Member) with fields +
relationships; URL-bind entries (e.g., `/blog/[slug]` resolves to an
Article entry). Three industry models:

1. **Sanity portable-text**: rich text encoded as a JSON array of
   typed nodes (`[{_type: 'block', children: [...]}, {_type: 'image', ...}]`).
   Excellent for unstructured rich content; weak for relational
   fields (you have to embed references manually).
2. **Strapi-style relational**: SQL-relational schema; fields are
   typed columns; relationships are explicit foreign keys. Excellent
   for structured + relational content; weak for rich text (typically
   stores Markdown blob).
3. **Contentful**: hybrid; rich text as portable-text-like nodes;
   relationships as typed links. The closest to what we want.
4. **WordPress posts + custom-fields**: rich text as HTML blob;
   custom fields as key-value. Legacy; weak typing.
5. **Storyblok**: portable-text + visual editor; component-based.

Per PRD-sites §FR-11, tenants need both structured fields (Product
has `name`, `price`, `sku`, `category`) AND rich content (Article has
`title`, `body` portable-text, `author` relationship). A pure-
portable-text or pure-relational model is insufficient.

Search target: site-search p95 ≤ 300ms across 5k pages. Postgres FTS
is the legacy choice but doesn't scale to multi-tenant cell with
flexible field-set queries. Meilisearch is the modern open-source
option (per ADR-SITES-0001 sibling search-substrate decision).

## Decision

The sites µservice ships a **hybrid CMS-collection data model**:
- **Structured fields** in Postgres relational columns (per field-
  definition `name`, `type`, `required`).
- **Rich content fields** stored as portable-text JSON in a JSONB
  column (Sanity-style).
- **Relationships** as explicit foreign keys to other entries (in
  the same or different collections).
- **Search index** in per-tenant Meilisearch (per ADR-SITES-0005-
  companion decision: Meilisearch 0.10.0 LTS).

Concrete bindings:
- **Schema**: `CollectionType{collection_id, site_id, name, fields:
  [FieldDefinition{name, type, required}], schema_version,
  schema_hash}`.
- **Field types**: `string`, `text` (Markdown), `number`, `boolean`,
  `datetime`, `image`, `link`, `relationship` (with `collection_id`
  reference), `portable_text` (rich content).
- **Schema versioning**: forward-compatible additive changes
  (`add_field`) are bumps without migration; breaking changes
  (`remove_field`, `change_type`) require explicit migration script
  + LEAN refuse on missing migration.
- **Postgres**: per-tenant RLS; partition by `(collection_id)` for
  large collections.
- **Meilisearch**: per-tenant index named `<tenant_id>__<site_id>`;
  reindex worker triggered on `CmsEntryWritten` / `CmsCollectionUpdated`
  events.
- **Crate**: `oya-sites-cms-collection-adapter-postgres` (backend-
  qualified per ADR-0105 Amendment 3) + `oya-sites-search-adapter-
  meilisearch` (backend-qualified).

## Alternatives Considered

### A. Pure Sanity portable-text

- **Pros**:
  - Excellent rich-text editing UX.
  - Battle-tested at Sanity scale (10k+ customers).
  - MIT licence.
- **Cons**:
  - Weak for relational fields; relationships have to be embedded
    or reference-resolved at render time → slow + complex.
  - Field-typing is weak; tenant-side schema enforcement.
  - Search has to scan portable-text for terms (expensive).
- **Rejected** as the sole model; portable-text reused for rich
  content within the hybrid.

### B. Pure Strapi-style relational

- **Pros**:
  - Strong typing.
  - Excellent for relational data.
  - Postgres-native.
- **Cons**:
  - Rich content as Markdown blob is weak (no granular block-level
    structure).
  - Doesn't align with Loro CRDT (per ADR-SITES-0001) which is
    portable-text-shaped at the block level.
- **Rejected** as the sole model; relational reused for structured
  fields within the hybrid.

### C. Contentful's hybrid

- **Pros**:
  - Proven model (Contentful is a major player).
- **Cons**:
  - Proprietary; can't directly adopt their schema.
- **Used as inspiration** for the hybrid model; not directly adopted.

### D. WordPress posts + custom-fields

- **Pros**:
  - WordPress import compatibility (post-M04 import path).
- **Cons**:
  - HTML blob is weakly-typed; no portable-text alignment with Loro.
  - Custom-fields key-value is anti-typed.
- **Rejected** as native model; WordPress-import will translate to
  the hybrid model.

### E. Hybrid portable-text + relational  ← **CHOSEN**

- **Pros**:
  - Best of both: rich content gets portable-text; structured fields
    get relational columns; relationships get FKs.
  - Postgres + Meilisearch substrate-portable.
  - Aligns with Loro CRDT block model (Sanity-style portable-text
    matches block-tree shape).
- **Cons**:
  - More complex than either pure model.
  - Schema migrations are non-trivial.
- **Accepted**.

## Consequences

### Positive

- **Tenants get both structured + rich content** in one collection model.
- **Postgres + Meilisearch substrate-portable**.
- **Aligns with Loro CRDT** (per ADR-SITES-0001) for block-level
  collab editing of portable-text fields.
- **Site-search p95 ≤ 300ms achievable** via per-tenant Meilisearch
  index.

### Negative

- **Schema migration complexity**: breaking changes require explicit
  migration; LEAN refuse on missing migration.
- **Meilisearch ops surface**: per-tenant index management; reindex
  worker; sharding.
- **Cross-collection relationships** require explicit resolver at
  render time (no JOIN in Meilisearch); GraphQL-style batch resolver
  added at API layer.

### Operational

- **`oya-sites-cms-collection-worker`**: handles schema migrations +
  reindex triggers.
- **`oya-sites-search-worker`**: reindexes Meilisearch on
  `CmsEntryWritten` / `CmsCollectionUpdated`.
- **LEAN `oya-check-cms-schema-migration-coverage`**: refuses
  breaking schema changes without migration scripts.

### Regulatory

- **GDPR Art. 17 erasure**: erasing an Entry cascades to Postgres
  (DELETE) + Meilisearch index (REMOVE) + audit-chain seal.
- **HIPAA**: PHI fields data-class-tagged; Cedar refuses anonymous
  rendering.
- **WCAG 2.2 AA**: portable-text fields validated at publish time.

## Verification

- [ ] **Schema version monotonicity** —
  `cargo nextest run -p oya-sites-cms-collection-domain -- schema_version_monotonic`.
- [ ] **CMS-query p95 ≤ 150ms (1000-entry collection)** —
  `cargo bench -p oya-sites-cms-collection-adapter-postgres -- query_1000`.
- [ ] **Per-tenant Meilisearch isolation** —
  `cargo nextest run -p oya-sites-search-adapter-meilisearch -- per_tenant_index_isolation`.
- [ ] **Site-search p95 ≤ 300ms** —
  `cargo bench -p oya-sites-search-adapter-meilisearch -- query`.

## References

- Sanity portable-text — `sanity.io/docs/presenting-block-text`.
- Strapi field-definition model — `strapi.io/docs`.
- Contentful entries + content-types — `contentful.com/developers/docs`.
- Storyblok component model — `storyblok.com/docs`.
- Meilisearch — `meilisearch.com/docs`.
- ADR-0105 Amendment 3 (backend-qualified adapters).
- ADR-SITES-0001 (Loro CRDT).
- `microservices/sites/PRD.md` §FR-11, AC-04.
- `microservices/sites/IP-009-cms-collection.md`.
- `microservices/sites/IP-010-search-meilisearch.md`.
