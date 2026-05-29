# Plan: obs-tracing-client-w3c-traceparent-parser

**Crate**: `oya-shared-tracing-client-kernel`
**Lane**: observability
**Priority**: high
**Effort**: S

## Objective

Extend the pure zero-dep tracing-client kernel with a deterministic, hermetic W3C Trace Context
`traceparent` parser/validator. No new dependencies; hermetic unit tests only; no I/O.

## W3C Traceparent Canonical Form

```
00-<32-hex trace-id>-<16-hex parent-id>-<2-hex flags>
```

Rules enforced:
1. Exactly 4 dash-delimited fields
2. Version field must be `00` (lowercase)
3. trace-id: exactly 32 lowercase hex chars, MUST NOT be all-zeros
4. parent-id: exactly 16 lowercase hex chars, MUST NOT be all-zeros
5. flags: exactly 2 lowercase hex chars
6. Sampled bit = flags & 0x01 == 1

## Tasks

1. [x] Add `ParsedTraceparent { trace_id: String, parent_id: String, sampled: bool }` struct
2. [x] Add `pub fn parse_traceparent(value: &str) -> Result<ParsedTraceparent, TracingClientError>`
3. [x] Add `impl Traceparent { pub fn validate(&self) -> Result<ParsedTraceparent, TracingClientError> }`
4. [x] Add `ParsedTraceparent::to_header_value() -> String` for round-trip
5. [x] Write tests in `mod tests`:
   - valid sampled traceparent
   - valid non-sampled traceparent
   - wrong version -> MalformedTraceparent
   - wrong field count -> MalformedTraceparent
   - non-hex chars -> MalformedTraceparent
   - wrong-length fields -> MalformedTraceparent
   - all-zero trace-id -> MalformedTraceparent
   - all-zero parent-id -> MalformedTraceparent
   - empty input -> MissingTraceparent
   - round-trip of parsed value re-renders to original string
6. [x] Note that NoopTracingClient's all-zero value is intentionally invalid per spec (test-noted)
7. [x] `cargo check -p oya-shared-tracing-client-kernel --all-targets` green
8. [x] `cargo nextest run -p oya-shared-tracing-client-kernel` green
