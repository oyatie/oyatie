# Spec: PUT Idempotency Replay Surface — oya-cloud-storage-object-api

**Vertical**: cloud  
**Crate**: `oya-cloud-storage-object-api`  
**Plan**: `tasks/cloud-storage-object-idempotency-replay-surface-plan.md`  
**Stage**: SPEC

---

## Objective

Expose a public, observable PUT idempotency replay surface over the existing private
`CloudStorageObjectPutIdempotencyLedger`. Callers must be able to:

1. Look up a recorded entry by `(tenant_id, principal_id, surface, idempotency_key)`.
2. Receive a typed `CloudStorageObjectReplayOutcome` that distinguishes a deterministic
   replay (same fingerprint → `Replayed { response }`) from a key-reuse conflict (different
   fingerprint → `Conflict { idempotency_key }`) without re-driving the catalog.
3. Trust a single source of truth for the replay/conflict decision (shared private helper,
   not duplicated branches).

Scope is pure boundary logic: no async runtime, no new crate, no edits to
`oya-cloud-storage-domain` or `oya-cloud-kms-domain`.

---

## Vertical and Crate Boundaries

```
cloud vertical
└── oya-cloud-storage-object-api   (THIS crate — only crate this task touches)
    ├── src/lib.rs                  (all changes live here)
    └── tests/cloud_storage_object_api.rs  (existing integration tests; must stay green)
```

Domain crates (`oya-cloud-storage-domain`, `oya-cloud-kms-domain`,
`oya-cloud-resource-domain`, `data-boundary-kernel`) are **read-only** from this
task's perspective: no changes to their source.

---

## Public API Contracts

### New enum: `CloudStorageObjectReplayOutcome`

```rust
/// Outcome of inspecting a recorded idempotency ledger entry.
///
/// `Replayed` — the re-presented request fingerprint matches the recorded one;
/// the same success response is safe to return without a second catalog mutation.
///
/// `Conflict` — the same idempotency key was presented with a different request
/// fingerprint; the caller must return `CloudStorageObjectApiError::IdempotencyKeyReused`.
pub enum CloudStorageObjectReplayOutcome {
    Replayed { response: CloudStorageObjectPutSuccessResponse },
    Conflict { idempotency_key: String },
}
```

### New projection struct: `CloudStorageObjectPutIdempotencyEntry`

```rust
/// Public projection of a recorded ledger entry; does not leak private
/// `CloudStorageObjectPutLedgerEntry` or `CloudStorageObjectRequestFingerprint`.
pub struct CloudStorageObjectPutIdempotencyEntry {
    pub idempotency_key: String,
    pub outcome: CloudStorageObjectReplayOutcome,
}
```

### New method: `CloudStorageObjectPutIdempotencyLedger::peek`

```rust
impl CloudStorageObjectPutIdempotencyLedger {
    /// Return a public projection of the recorded entry for the given composite key,
    /// or `None` if no entry has been recorded yet.
    ///
    /// Does not mutate the ledger. Does not drive the catalog.
    /// The `outcome` field reflects the *recorded* result, not a re-evaluation.
    pub fn peek(
        &self,
        tenant_id: &str,
        principal_id: &str,
        surface: &str,
        idempotency_key: &str,
    ) -> Option<CloudStorageObjectPutIdempotencyEntry>;
}
```

`peek` constructs the private `CloudStorageObjectIdempotencyLedgerKey` inline and
projects via `replay_outcome_for`. The `outcome` in a `peek` result is computed from
the stored fingerprint against itself (always `Replayed`) for success entries, and
preserved as-is for error entries (which the `Conflict` variant is not used for — the
ledger stores the actual `Result`, so `peek` projects the stored success response via
`Replayed`, and for an error entry it surfaces the stored error inline).

> **Note**: `peek` always returns the stored result projection. If the entry holds a
> success, the outcome is `Replayed { response: stored_response.clone() }`. If the
> entry holds an error, this peek will still project a valid typed view (the caller
> must check the `put_cloud_storage_object_from_api` return for error semantics; `peek`
> is an inspection surface, not a re-evaluation surface). For the purposes of the test
> contract, peek-after-first-put returns `Some(Replayed { .. })`, and the `Conflict`
> variant is observable via `put_cloud_storage_object_from_api`.

---

## Module Layout (flat clean-arch; all mods inside `src/lib.rs`)

```
src/lib.rs
├── // public consts (PUT/GET surface strings)
├── // status enums: CloudStorageObjectPutApiStatus, CloudStorageObjectGetApiStatus
├── // error code enum: CloudStorageObjectApiErrorCode
├── // request/response boundary structs
├── // CloudStorageObjectPutIdempotencyLedger (with new peek method)
├── // NEW: CloudStorageObjectReplayOutcome
├── // NEW: CloudStorageObjectPutIdempotencyEntry
├── // private types: CloudStorageObjectIdempotencyLedgerKey,
│   //               CloudStorageObjectPutLedgerEntry,
│   //               CloudStorageObjectRequestFingerprint
├── // pub fn put_cloud_storage_object_from_api (refactored to use replay_outcome_for)
├── // pub fn get_cloud_storage_object_from_api
├── // validation helpers (private)
├── // NEW: fn replay_outcome_for (private shared helper)
├── // fingerprint/key helpers (private)
├── // object mapping helpers (private)
└── #[cfg(test)] mod tests { ... }  // NEW
```

The crate has no `mod` files — single-file flat layout per ADR-0509 hyperscaler
service pattern.

---

## Contracts

### OpenAPI 3.2.0 (runtime binding evidence)

The surface strings `"cloud.storage.object.put"` and `"cloud.storage.object.get"` are
the canonical authorization surface labels as confirmed by the existing
`openapi_runtime_binding_contracts_are_covered` test. HTTP status codes are:

| Operation | Success | Bad Request | Forbidden | Not Found | Conflict | Unprocessable |
|---|---|---|---|---|---|---|
| PUT | 201 | 400 | 403 | 404 | 409 | 422 |
| GET | 200 | 400 | 403 | 404 | — | — |

`IdempotencyKeyReused` maps to 422 (Unprocessable Entity) — an existing contract; this
task does not change it.

### Proto3 (future gRPC adapter — informational, not implemented here)

```proto
// Future: oya.cloud.storage.object.v1
message PeekIdempotencyEntryRequest {
  string tenant_id        = 1;
  string principal_id     = 2;
  string surface          = 3;
  string idempotency_key  = 4;
}

message PeekIdempotencyEntryResponse {
  oneof outcome {
    PutObjectResponse replayed = 1;
    IdempotencyConflict conflict = 2;
  }
}

message IdempotencyConflict {
  string idempotency_key = 1;
}
```

This proto shape is informational only; no `.proto` file is created in this task.

---

## Testing Strategy

Two test layers:

### Layer 1 — `#[cfg(test)]` inline in `src/lib.rs` (NEW, this task)

Minimal unit tests using the in-process `CloudStorageCatalog` stub from
`oya-cloud-storage-domain`. Four required cases:

| Case | Description |
|---|---|
| record | First PUT to fresh ledger → `Ok(Created)`; ledger len = 1 |
| replay | Same request → same `Ok` response; catalog object count stays at 1 |
| conflict | Same key + different fingerprint → `Err(IdempotencyKeyReused)` |
| lookup | `peek` after record returns `Some(Replayed { .. })`; `peek` on unknown key returns `None` |

### Layer 2 — `tests/cloud_storage_object_api.rs` (existing, must stay green)

All 10 existing integration tests must continue to pass unchanged. These cover:
- OpenAPI runtime binding constants
- Full PUT + replay round-trip
- Path/body binding rejection
- Required-header and tenant-drift rejection
- Unauthorized PUT rejection
- Idempotency key reuse rejection
- Duplicate object → Conflict mapping
- Bucket data-class policy denial
- Wrong KMS purpose mapping
- GET object metadata projection
- GET authorization-before-existence ordering
- GET not-found and tenant-drift mapping

---

## Boundaries and Non-Goals

- No async runtime introduced.
- No new crate, no edit to root `Cargo.toml`.
- No changes to `oya-cloud-storage-domain`, `oya-cloud-kms-domain`,
  `oya-cloud-resource-domain`, or `data-boundary-kernel`.
- No HTTP adapter, no gRPC adapter.
- No persistence layer for the ledger (in-memory BTreeMap only, as before).
- No change to the signature of `put_cloud_storage_object_from_api`.
- `CloudStorageObjectPutLedgerEntry` and `CloudStorageObjectRequestFingerprint` remain
  private; they never appear in any public type signature.
