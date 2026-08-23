# Spec: W3C Traceparent Parser for shared-tracing-client-kernel

**Slug**: obs-tracing-client-w3c-traceparent-parser
**Crate**: `shared-tracing-client-kernel`
**Lane**: observability
**Priority**: high
**Effort**: S

## Summary

Add a pure, zero-dependency, deterministic W3C Trace Context `traceparent` parser to the
`shared-tracing-client-kernel` crate. The parser enforces the W3C canonical form, rejects
invalid inputs with typed errors, and exposes a `ParsedTraceparent` value type with a round-trip
renderer.

## Public API

### `ParsedTraceparent`

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedTraceparent {
    pub trace_id: String,   // 32 lowercase hex chars
    pub parent_id: String,  // 16 lowercase hex chars
    pub sampled: bool,      // flags & 0x01 == 1
}

impl ParsedTraceparent {
    /// Render back to the canonical traceparent header value.
    pub fn to_header_value(&self) -> String;
}
```

### `parse_traceparent`

```rust
pub fn parse_traceparent(value: &str) -> Result<ParsedTraceparent, TracingClientError>
```

Returns `MissingTraceparent` for empty input. Returns `MalformedTraceparent(value.to_string())`
for any W3C violation. Returns `Ok(ParsedTraceparent { ... })` on success.

### `Traceparent::validate`

```rust
impl Traceparent {
    pub fn validate(&self) -> Result<ParsedTraceparent, TracingClientError>
}
```

Delegates to `parse_traceparent(&self.0)`.

## W3C Validation Rules

| Rule | Violation |
|------|-----------|
| Empty string | `MissingTraceparent` |
| Not exactly 4 dash-delimited fields | `MalformedTraceparent` |
| Version != "00" | `MalformedTraceparent` |
| trace-id not exactly 32 lowercase hex chars | `MalformedTraceparent` |
| trace-id is all zeros | `MalformedTraceparent` |
| parent-id not exactly 16 lowercase hex chars | `MalformedTraceparent` |
| parent-id is all zeros | `MalformedTraceparent` |
| flags not exactly 2 lowercase hex chars | `MalformedTraceparent` |

## Sampled Flag

The `sampled` field is `true` when `flags_byte & 0x01 == 1` (the W3C "sampled" trace flag).

## NoopTracingClient Note

`NoopTracingClient::inject` injects an all-zero traceparent
(`00-00000000000000000000000000000000-0000000000000000-00`) which is intentionally invalid per the
W3C spec (all-zero trace-id and parent-id are rejected by `parse_traceparent`). This is the
correct no-op behavior for compile-time wiring — production impls will inject a real span context.

## Round-trip Property

For any `value` that parses successfully:
```
parse_traceparent(value).unwrap().to_header_value() == value
```

## Acceptance Criteria

- [x] Valid sampled traceparent parses correctly with `sampled = true`
- [x] Valid non-sampled traceparent parses correctly with `sampled = false`
- [x] Wrong version yields `MalformedTraceparent`
- [x] Wrong field count (3 or 5 fields) yields `MalformedTraceparent`
- [x] Non-hex characters in any field yield `MalformedTraceparent`
- [x] Wrong-length fields yield `MalformedTraceparent`
- [x] All-zero trace-id yields `MalformedTraceparent`
- [x] All-zero parent-id yields `MalformedTraceparent`
- [x] Empty input yields `MissingTraceparent`
- [x] Round-trip: parsed value re-renders to the original string
- [x] `#![forbid(unsafe_code)]` preserved
- [x] No new dependencies added
- [x] No I/O; hermetic unit tests only
