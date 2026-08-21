# Spec: community-moderation-queue-triage

**Vertical:** community  
**Crate:** `community-post-store-usecase`  
**ADR governance:** ADR-0509 (single-crate-per-service, mod-based subsystems)  
**Layout authority:** ADR-0131 (per-microservice flat layout)

---

## Objective

Extend `community-post-store-usecase` with a deterministic, evidence-gated moderation-queue triage usecase. The queue accepts moderation outcomes produced by the existing `moderate_post` / `moderation_case` surface, assigns a severity/priority ordering (`Remove > Hide > Allow` with evidence-strength and report-count tiebreaks), and exposes a pure ordered-drain function yielding the next case for a reviewer. No new crate, no DB, no root `Cargo.toml` change — additive module within the existing crate.

---

## Vertical & Boundaries

- **Vertical:** community / post-store  
- **Crate boundary:** `community-post-store-usecase` (`crates/community-post-store-usecase/`)  
- **Must not touch:** root `Cargo.toml`, any other crate, any adapter or app layer  
- **No persistence:** pure in-memory domain logic; caller is responsible for persisting queue state if needed

---

## Module Layout (flat-clean-arch)

```
crates/community-post-store-usecase/src/
  lib.rs                     ← existing; re-exports moderation_queue module
  moderation_queue.rs        ← NEW: ModerationQueue, enqueue, next_case, drain_ordered
```

`lib.rs` gains one line: `pub mod moderation_queue;`

---

## Domain Contracts

### `QueueSeverity` (enum, Ord)

```rust
pub enum QueueSeverity { Allow, Hide, Remove }
// Ord: Remove > Hide > Allow
```

### `EvidenceStrength` (enum, Ord)

```rust
pub enum EvidenceStrength { None, Strong }
// Ord: Strong > None
```

### `ModerationQueueEntry` (struct)

| Field | Type | Description |
|-------|------|-------------|
| `post_id` | `String` | Stable post identifier |
| `severity` | `QueueSeverity` | Derived from `ModerationVerb` |
| `evidence_strength` | `EvidenceStrength` | `Strong` if evidence_ref non-empty |
| `report_count` | `u32` | Caller-supplied signal count |
| `idempotency_key` | `String` | Copied from ctx; dedup key |
| `audit_correlation_id` | `String` | Copied from ctx; MUST NOT mutate |
| `policy_decision_ref` | `String` | Copied from ctx |
| `tenant_scope_ref` | `String` | Copied from ctx |
| `principal_ref` | `String` | Copied from ctx |

### `ModerationQueue` (struct)

```rust
pub struct ModerationQueue { entries: Vec<ModerationQueueEntry> }
```

Pure in-memory. No external I/O.

---

## Function Contracts

### `enqueue`

```rust
pub fn enqueue(
    queue: &mut ModerationQueue,
    ctx: &AuthorizedCommunityContext,
    post_id: String,
    verb: ModerationVerb,
    evidence_ref: &str,
    report_count: u32,
) -> Result<(), CommunityUsecaseError>
```

**Pre-conditions:**
1. `ctx.validate()` must pass → `CommunityUsecaseError::Api(...)` on failure.
2. `post_id` must not be empty → `CommunityUsecaseError::Domain(CommunityError::Invalid)`.
3. `verb` is `Hide` or `Remove` and `evidence_ref.trim().is_empty()` → `CommunityUsecaseError::Domain(CommunityError::ModerationNeedsEvidence)`.
4. Duplicate `idempotency_key` → no-op, return `Ok(())` (idempotent).

**Post-conditions:**
- Entry inserted with `severity` derived from verb, `evidence_strength` derived from `evidence_ref`.
- `idempotency_key` and `audit_correlation_id` copied verbatim from ctx.

### `next_case`

```rust
pub fn next_case(queue: &ModerationQueue) -> Option<&ModerationQueueEntry>
```

Returns a reference to the highest-priority entry (same ordering as `drain_ordered`). Returns `None` on empty queue. Non-mutating.

### `drain_ordered`

```rust
pub fn drain_ordered(queue: &ModerationQueue) -> Vec<&ModerationQueueEntry>
```

Returns all entries sorted by the following stable, documented tiebreak (descending priority):

1. `severity` descending: `Remove > Hide > Allow`
2. `evidence_strength` descending: `Strong > None`
3. `report_count` descending: higher count first
4. `idempotency_key` ascending: lexicographic, guarantees determinism on full equality

Non-mutating. Does not touch `audit_correlation_id` or `idempotency_key`.

---

## Protocol / API Contracts

This is a pure domain usecase module. It has no REST or gRPC surface of its own; callers (adapter/rest/grpc layers) invoke `enqueue` after `moderate_post` returns a `ModerationReceipt`, and `drain_ordered` / `next_case` when serving a reviewer queue endpoint.

The existing `ModerationQueueService` proto binding in `community-post-store-api` (`proto_service: "ModerationQueueService"`, `proto_rpc: "ApplyAction"`) provides the external contract anchor. This spec does not extend the proto surface.

**AsyncAPI channel (existing):** `community.moderation.actioned`  
**OpenAPI 3.2.0 operation (existing):** `applyModerationAction` on `ModerationQueueService`

---

## Error Propagation

All errors use existing `CommunityUsecaseError` variants:

| Scenario | Error |
|----------|-------|
| Ctx validation fails | `CommunityUsecaseError::Api(CommunityApiError::*)` |
| Empty post_id | `CommunityUsecaseError::Domain(CommunityError::Invalid)` |
| Hide/Remove without evidence | `CommunityUsecaseError::Domain(CommunityError::ModerationNeedsEvidence)` |
| Tenant mismatch (future guard) | `CommunityUsecaseError::TenantMismatch` |
| Duplicate idempotency_key | `Ok(())` — no-op (idempotent) |

---

## Testing Strategy

All tests are `#[cfg(test)]` inside `src/moderation_queue.rs` using `cargo nextest`.

| Test | Validates |
|------|-----------|
| `enqueue_rejects_hide_without_evidence` | ST1 evidence gate |
| `enqueue_rejects_remove_without_evidence` | ST1 evidence gate |
| `enqueue_allow_succeeds_without_evidence` | ST1 allow path |
| `drain_ordered_remove_before_hide_before_allow` | ST2 severity ordering |
| `drain_ordered_stable_tiebreak_on_equal_severity` | ST2 stable tiebreak |
| `audit_fields_pass_through_unchanged` | ST2 immutability guarantee |
| `next_case_empty_queue_returns_none` | ST3 empty queue |
| `drain_ordered_empty_queue_returns_empty_vec` | ST3 empty queue |
| `enqueue_duplicate_idempotency_key_is_noop` | ST3 idempotency |
| `enqueue_tenant_mismatch_checked_via_ctx_validate` | ST3 scoping |

---

## Audit / Idempotency Invariants Preserved

- `idempotency_key`: carried verbatim from `AuthorizedCommunityContext`; dedup key; never modified.
- `audit_correlation_id`: carried verbatim; never modified by sort or drain.
- `policy_decision_ref`: carried verbatim.
- Evidence-gating mirrors `moderation_case` in the domain crate: `Hide`/`Remove` require non-empty evidence.

---

## Constraints

- No new crate; no root `Cargo.toml` edit.
- No database, no async runtime, no I/O of any kind.
- No new external dependencies beyond what `community-post-store-usecase` already has.
- No changes to existing functions (`create_post`, `cast_vote`, `moderate_post`, `map_mode`, `map_moderation`).
