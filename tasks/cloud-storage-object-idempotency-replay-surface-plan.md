# Plan: cloud-storage-object-idempotency-replay-surface

Vertical: cloud  
Crate: `oya-cloud-storage-object-api`  
Branch: `feat/task-cloud-storage-object-idempotency-replay-surface-2026-05-28`

---

## Subtasks

### [cso-1] Public lookup/peek on `CloudStorageObjectPutIdempotencyLedger`

**Goal**: Add a public method that returns a typed view of a recorded ledger entry by
`(tenant_id, principal_id, surface, idempotency_key)` without mutating the ledger or
leaking private types (`CloudStorageObjectPutLedgerEntry`,
`CloudStorageObjectRequestFingerprint`).

**Changes**:
- Define `CloudStorageObjectPutIdempotencyEntry` — a public projection struct carrying
  only the fields callers need: `idempotency_key: String` and
  `outcome: CloudStorageObjectReplayOutcome` (defined in cso-2).
- Add `CloudStorageObjectPutIdempotencyLedger::peek(tenant_id, principal_id, surface,
  idempotency_key) -> Option<CloudStorageObjectPutIdempotencyEntry>` that reads
  `self.entries` by constructing the private key and projects via the shared
  `replay_outcome_for` helper (defined in cso-2).
- `entries` field remains `pub(crate)` at most; private types never appear in public
  signatures.

**Acceptance**:
- `cargo check -p oya-cloud-storage-object-api --all-targets` passes.
- New public method compiles; no private type leaks into public API.
- No signature change to `put_cloud_storage_object_from_api`.

---

### [cso-2] Typed replay-outcome enum + shared decision helper

**Goal**: Define `CloudStorageObjectReplayOutcome` and wire the existing replay branch
of `put_cloud_storage_object_from_api` through a single private helper so replay/conflict
logic has exactly one source of truth.

**Changes**:
- Define public enum:
  ```rust
  pub enum CloudStorageObjectReplayOutcome {
      Replayed { response: CloudStorageObjectPutSuccessResponse },
      Conflict { idempotency_key: String },
  }
  ```
- Add private `fn replay_outcome_for(entry: &CloudStorageObjectPutLedgerEntry,
  fingerprint: &CloudStorageObjectRequestFingerprint, idempotency_key: &str)
  -> CloudStorageObjectReplayOutcome` that encodes the same-fingerprint / different-
  fingerprint decision currently inlined in `put_cloud_storage_object_from_api`.
- Replace the two-branch inline in `put_cloud_storage_object_from_api` with a call to
  `replay_outcome_for`; convert the outcome to the existing `Result` shape (no behaviour
  change).
- `peek` (cso-1) calls `replay_outcome_for` to build its projection.

**Acceptance**:
- Existing `put_cloud_storage_object_from_api` behaviour unchanged for new-key,
  same-fingerprint, and different-fingerprint cases.
- Conflict path still returns `CloudStorageObjectApiError::IdempotencyKeyReused`.
- `cargo check -p oya-cloud-storage-object-api --all-targets` passes.

---

### [cso-3] First `#[cfg(test)]` module in `src/lib.rs`

**Goal**: Add inline unit tests directly in `src/lib.rs` that exercise the new public
surface against a stub `CloudStorageCatalog`; prove record/replay/conflict/lookup
semantics in four distinct test cases.

**Changes** (inside `src/lib.rs`, gated `#[cfg(test)]`):
- `test_first_put_records_and_returns_created` — first PUT to a fresh ledger records
  entry and returns `Ok(Created)`.
- `test_replay_same_fingerprint_no_catalog_mutation` — second PUT with same request
  replays the same `Ok` response; catalog object count stays at 1.
- `test_conflict_different_fingerprint_yields_idempotency_key_reused` — second PUT
  with same key but mutated body yields
  `Err(CloudStorageObjectApiError::IdempotencyKeyReused { .. })`.
- `test_peek_reflects_each_state` — after record and after conflict attempt, `peek`
  returns `Some(entry)` whose `outcome` matches the recorded result.

Tests use the same fixture helpers pattern established in
`tests/cloud_storage_object_api.rs` (BTreeMap-backed catalog, `#[allow(clippy::unwrap_used)]`).

**Acceptance**:
- `cargo nextest run -p oya-cloud-storage-object-api` is green with >= 4 new test cases.
- `cargo check -p oya-cloud-storage-object-api --all-targets` passes.

---

## Acceptance Summary

| Check | Command |
|---|---|
| Compile clean (all targets) | `cargo check -p oya-cloud-storage-object-api --all-targets` |
| Tests green | `cargo nextest run -p oya-cloud-storage-object-api` |
| No private type leak | reviewed via `cargo check` — no `CloudStorageObjectPutLedgerEntry` / `CloudStorageObjectRequestFingerprint` in public fn signatures |
| Behaviour unchanged | existing integration tests in `tests/cloud_storage_object_api.rs` continue to pass |
