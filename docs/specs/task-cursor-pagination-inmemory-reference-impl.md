# Spec: cursor-pagination-inmemory-reference-impl

**Crate**: `shared-cursor-pagination-kernel`
**Lane**: foundation
**Priority**: high
**Effort**: M
**ADR**: ADR-0150 (opaque cursor pagination, offset FORBIDDEN)

## Context

The kernel crate defined the `CursorPaginationKernel` trait and supporting
types (`Cursor`, `PageSize`, `Page`, `PaginationError`) but left
`fetch_page` as `SkeletonNotYetImplemented`. This slice delivers a pure
in-memory reference implementation suitable for testing consumers without I/O.

## Public surface added

### `cursor` module (src/cursor.rs)

```rust
/// Encode raw bytes as base64-URL (no padding).
pub fn encode(bytes: &[u8]) -> String;

/// Decode base64-URL string (no padding) to bytes.
/// Returns `Err(PaginationError::CursorMalformed(_))` on invalid input.
pub fn decode(s: &str) -> Result<Vec<u8>, PaginationError>;

/// Compute a stable u64 scope hash for a string.
pub fn scope_hash(scope: &str) -> u64;
```

### `inmemory` module (src/inmemory.rs)

```rust
/// Pure in-memory reference implementation of `CursorPaginationKernel`.
///
/// `T`  — item type (Clone).
/// `F`  — filter predicate `Fn(&T, &FilterSpec) -> bool`.
/// `FilterSpec` — caller-supplied filter type that must implement `Display`
///                so its canonical string form is used as the `scope_hash`
///                input.
pub struct InMemoryCursorPaginator<T, F> { ... }

impl<T, F> InMemoryCursorPaginator<T, F>
where
    T: Clone,
    F: Fn(&T, &String) -> bool + Send + Sync,
{
    pub fn new(items: Vec<T>, filter_fn: F) -> Self;
}
```

`CursorPaginationKernel` is implemented for `InMemoryCursorPaginator<T, F>`
with `Item = T` and `Filter = String`.

### Cursor payload layout

The opaque cursor wraps a JSON-serialised struct:
```
{"offset": <u64>, "scope": <u64>}
```
serialised to bytes, then base64-URL encoded (no padding). This is an
implementation detail; callers treat `Cursor` as opaque.

## Behaviour specification

| Scenario | Expected outcome |
|---|---|
| `cursor=None`, page_size=3, 10 items matching filter | items[0..3], has_more=true, next_cursor=Some(_) |
| Continue with returned cursor | items[3..6], has_more=true |
| Last page (items[9..10]) | has_more=false, next_cursor=None |
| cursor from filter A reused with filter B | `PaginationError::CursorScopeMismatch` |
| Malformed cursor string | `PaginationError::CursorMalformed` |
| page_size=0 | `PaginationError::PageSizeOutOfBounds` |
| page_size=101 | `PaginationError::PageSizeOutOfBounds` |

## Constraints

- No external crates added.
- `#![forbid(unsafe_code)]` retained on crate root.
- No offset pagination exposed (`offset` is cursor-internal only).
- Hermetic tests only (no I/O, no async).
