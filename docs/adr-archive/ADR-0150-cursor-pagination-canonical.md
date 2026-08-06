---
id: ADR-0150
status: Superseded
superseded_by: [ADR-709]
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0150: Cursor Pagination Canonical

- Status: Accepted
- Date: 2026-05-18
- Deciders: council-architecture, axis-foundry, axis-all-microservices
- Tier-A hyperscaler pattern: AWS NextToken + Stripe starting_after

## Context

Offset pagination (`?offset=N&limit=M`) scales O(N) on the database,
yields duplicate/skipped rows under concurrent writes, and leaks the
result-set size to attackers. AWS, Stripe, Slack, GitHub, and every
serious SaaS API use OPAQUE cursor pagination instead.

PR #143 review (Fix-Agent-I) flagged that several µservice OpenAPI
specs still expose offset parameters or do not declare any
pagination at all on list endpoints.

## Decision

Adopt opaque cursor pagination as MANDATORY on every list endpoint
in every µservice; offset pagination is BANNED.

1. The canonical spec is
   `docs/standards/cursor-pagination-canonical.md`.
2. The trait surface lives in
   `crates/oya-shared-cursor-pagination-kernel/`.
3. Every µservice OpenAPI 3.2.0 list path declares
   `cursor` + `page_size` parameters via the canonical
   `Cursor` + `PageSize` component refs.
4. Compliance is enforced by the new
   `oya-check-cursor-pagination-coverage` gate, wired into
   `gate run-all`.

## Consequences

Positive:
- Stable pagination under concurrent writes.
- Constant-time per-page DB queries.
- No result-set-size leakage.

Negative:
- Per-µservice cursor codec work.
- Clients that depended on `offset=` must migrate (none in
  production yet).

## Alternatives considered

- Keep offset pagination — REJECTED, scales O(N).
- Server-only cursors with no client opacity guarantee — REJECTED,
  invites tampering.
- Keyset pagination without `scope_hash` — REJECTED, allows cursor
  reuse across mismatched filters.

## References

- AWS API Reference — NextToken pattern.
- Stripe — starting_after cursor pagination.
- GitHub REST API — cursor-based pagination.
- docs/standards/cursor-pagination-canonical.md.
- crates/oya-shared-cursor-pagination-kernel/.
- crates/oya-check-cursor-pagination-coverage/.
