# Plan: community-moderation-queue-triage

**Vertical:** community  
**Crate:** `oya-community-post-store-usecase`  
**Branch:** `feat/task-community-moderation-queue-triage-2026-05-28`

## Subtasks

### ST1 — Add `ModerationQueue` type and `enqueue` function

Additive module `src/moderation_queue.rs` inside `oya-community-post-store-usecase`.

**What to build:**
- `ModerationQueueEntry` struct holding: `post_id`, `verb` (ModerationVerb mapped to severity), `evidence_strength` (non-empty evidence_ref → `Strong`, absent/allow → `None`), `report_count` (u32, default 0 for single-report path), `idempotency_key`, `audit_correlation_id`, `policy_decision_ref`, `tenant_scope_ref`, `principal_ref`.
- `QueueSeverity` enum: `Remove > Hide > Allow` (ord derived, Remove = highest).
- `ModerationQueue` struct: `Vec<ModerationQueueEntry>` (in-memory, pure).
- `enqueue(ctx, receipt, verb, evidence_ref, report_count) -> Result<(), CommunityUsecaseError>`: validates `ctx`, derives severity from `ModerationVerb`, rejects `Hide`/`Remove` when `evidence_ref` is empty (returns `CommunityUsecaseError::Domain(CommunityError::ModerationNeedsEvidence)`), inserts entry.

**Acceptance:**
- `cargo check -p oya-community-post-store-usecase --all-targets` clean.
- `enqueue` rejects cases lacking audit evidence → covered by nextest unit test.

---

### ST2 — Implement deterministic `next_case` / `drain` ordered-drain

**What to build:**
- `next_case(&self) -> Option<&ModerationQueueEntry>`: returns reference to the highest-priority entry without mutation.
- `drain_ordered(&self) -> Vec<&ModerationQueueEntry>`: returns all entries sorted descending by `(severity, evidence_strength, report_count)`, stable on equal priority by `idempotency_key` lexicographic ascending (documented tiebreak).
- Ordering contract: `Remove > Hide > Allow`; within same verb: `Strong evidence > None`; within same evidence tier: higher `report_count` first; final tiebreak: `idempotency_key` ascending (stable, deterministic).
- Neither function mutates `audit_correlation_id` or `idempotency_key`.

**Acceptance:**
- `cargo nextest run -p oya-community-post-store-usecase` passes.
- Test asserts Remove-before-Hide-before-Allow ordering.
- Test asserts stable tiebreak on equal severity.
- Test asserts `idempotency_key` and `audit_correlation_id` pass through unchanged.

---

### ST3 — Edge-case tests

**What to build:**
- Empty-queue: `next_case` returns `None`; `drain_ordered` returns empty vec without panic.
- Duplicate idempotency: `enqueue` with same `idempotency_key` is a no-op (entry already present) — returns `Ok(())` silently (idempotent, not an error).
- Tenant/principal scoping: `enqueue` with mismatched `tenant_scope_ref` vs ctx returns `CommunityUsecaseError::TenantMismatch`; principal mismatch returns `CommunityUsecaseError::PrincipalMismatch` consistent with existing `moderate_post` guards.

**Acceptance:**
- `cargo nextest run -p oya-community-post-store-usecase` green with new tests.
- Empty drain yields `None`/empty without panic.
- Duplicate enqueue is a no-op asserted by test.

---

## Acceptance Summary

| Gate | Command |
|------|---------|
| Compile clean | `cargo check -p oya-community-post-store-usecase --all-targets` |
| All tests green | `cargo nextest run -p oya-community-post-store-usecase` |
| No new crate | Root `Cargo.toml` unchanged |
| No DB / no adapter | Pure in-memory domain logic only |
