---
doc_class: Standard
title: Cursor Pagination (Canonical)
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-18
owner_team: council-architecture
deciders: council-architecture, axis-foundry, axis-all-microservices
related_adrs: [ADR-0150]
review_cadence: annually
doc_status: published
---

# Cursor Pagination (Canonical)

## Authority

ADR-0150-cursor-pagination-canonical landed this contract. AWS's
`NextToken` pattern and Stripe's `starting_after` / `ending_before`
patterns are the industry references. Every list-class REST endpoint
in every oyatie microservice MUST use the cursor-pagination contract
below. Offset-pagination is FORBIDDEN.

## Contract

### 1. Required query parameters

Every list endpoint accepts:

```
GET /v1/<collection>?cursor=<opaque>&page_size=<n>&order=<asc|desc>
```

- `cursor` — opaque base64-URL-encoded value; server-issued; never
  reverse-engineerable by clients.
- `page_size` — integer in `[1, 100]`; server may clamp; default 25.
- `order` — `asc | desc`; default per-collection (documented in
  OpenAPI description).

### 2. Required response envelope

```json
{
  "items": [...],
  "next_cursor": "eyJpZCI6IjAxSE0iLCJ0cyI6MTcwfQ",
  "has_more": true,
  "page_size": 25
}
```

`next_cursor` is null/absent on the final page (`has_more=false`).

### 3. Cursor opaqueness

Cursors are base64-URL-encoded payloads of the form:

```
{
  "tie_break_id": "<ulid>",
  "tie_break_ts": <unix-ms>,
  "scope_hash": "<sha256-of-filter-params>"
}
```

- The `scope_hash` binds the cursor to its originating filter set;
  re-using a cursor with different filters returns `400 Bad Request`
  with `Cursor-Scope-Mismatch`.
- The `tie_break_id` is a ULID/KSUID (ADR-0156) so ordering is stable
  even when two records share the same timestamp.

### 4. NO offset pagination

`?offset=` / `?page=` query parameters are FORBIDDEN. They scale O(N)
on the database; they yield duplicate/skipped rows on concurrent
writes; they leak result-set size to attackers.

### 5. Trait surface

Every microservice integrates `CursorPaginationKernel` from
`shared-cursor-pagination-kernel`:

```rust
pub trait CursorPaginationKernel: Send + Sync {
    type Item;
    type Filter;
    fn fetch_page(
        &self,
        cursor: Option<Cursor>,
        page_size: PageSize,
        filter: &Self::Filter,
    ) -> Result<Page<Self::Item>, PaginationError>;
}
```

### 6. OpenAPI declaration

Every list path declares:

```yaml
parameters:
  - $ref: '#/components/parameters/Cursor'
  - $ref: '#/components/parameters/PageSize'
responses:
  '200':
    schema:
      $ref: '#/components/schemas/CursorPage'
```

### 7. Validation

The `check-cursor-pagination-coverage` gate enforces that every
list endpoint in every microservice declares `cursor` + `page_size`
parameters and that no path declares an `offset` query parameter.

## References

- AWS API Reference — `NextToken` / `MaxResults` pattern.
- Stripe — `starting_after` / `ending_before` cursor pattern.
- Slack Web API — `cursor` pagination.
- ADR-0150-cursor-pagination-canonical.
