# Plan: cursor-pagination-inmemory-reference-impl

## Objective
Implement a pure in-memory reference implementation of `CursorPaginationKernel` in
`oya-shared-cursor-pagination-kernel`, replacing the `SkeletonNotYetImplemented("fetch_page")`
placeholder with a real deterministic pager.

## Acceptance Criteria (from backlog)
1. `fetch_page` returns correctly bounded pages honoring `PageSize` clamp `[1, 100]`.
2. `next_cursor` is opaque base64-URL and round-trips (encode → decode → same offset).
3. Reusing a cursor against a mismatched filter yields `PaginationError::CursorScopeMismatch`.
4. Final page sets `has_more=false` and `next_cursor=None`.
5. Offset pagination absent (`ADR-0150`).
6. Hermetic unit tests only (no I/O), all existing tests still pass.
7. `#![forbid(unsafe_code)]` retained.
8. No new workspace member; no root `Cargo.toml` edit; no other crate touched.

## Design

### Cursor Encoding
- Internal cursor payload: `CursorPayload { offset: u64, scope_hash: String }`.
- Serialized as `{offset}:{scope_hash}` (deterministic, no external deps).
- Base64-URL encoded using a pure-std implementation (RFC 4648 §5, no padding).

### `scope_hash` Binding
- Computed from the filter's `scope_hash()` method on `ScopedFilter` trait.
- Mismatch between recorded scope and active filter → `PaginationError::CursorScopeMismatch`.

### `InMemoryCursorPaginator<T, F>`
- Holds an owned `Vec<T>` (cloned items) and a `PhantomData<F>`.
- `T: Clone`, `F: ScopedFilter`.
- `fetch_page(cursor, page_size, filter)`:
  1. Decode cursor (if Some) → validate scope_hash matches `filter.scope_hash()`.
  2. Slice `[offset .. offset + page_size]` from the stored items.
  3. Compute `next_cursor` for the next page if items remain after the slice.
  4. Return `Page { items, next_cursor, has_more, page_size }`.

### `ScopedFilter` trait
- `fn scope_hash(&self) -> String` — deterministic hash of the filter set.
- Implemented by consumers; test uses a simple `StringFilter(String)`.

## Implementation Steps
1. [x] Write plan (this file).
2. [x] Write spec (`docs/specs/task-cursor-pagination-inmemory-reference-impl.md`).
3. [x] Implement red tests in `src/lib.rs`.
4. [x] Implement green production code in `src/lib.rs`.
5. [x] `cargo check -p oya-shared-cursor-pagination-kernel --all-targets` → clean.
6. [x] `cargo nextest run -p oya-shared-cursor-pagination-kernel` → all green.
7. [x] Self-review: fix Critical/High.
8. [x] Simplify (behavior-preserving), re-run nextest.
