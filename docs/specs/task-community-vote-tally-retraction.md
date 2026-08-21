# Spec: community-vote-tally-retraction

**Status:** SPEC  
**Vertical:** community  
**Lane:** `feat/task-community-vote-tally-retraction-2026-05-28`  
**Crate:** `community-post-store-domain` (`crates/community-post-store-domain/`)

---

## Objective

Extend the community post-store domain crate with vote-retraction, net-score tally, and action-aware moderation logic. All changes are pure domain functions or methods on existing types — no I/O, no persistence, no new crates, no root `Cargo.toml` edits.

---

## Vertical Context

The `community` vertical owns the post-store service (`oya-community-post-store-*` crate family). The domain crate (`community-post-store-domain`) holds all business invariants as pure Rust: `CommunityPost`, `VoteLedger`, `VoteReceipt`, `ModerationAction`, and `CommunityError`.

---

## Scope Boundary

This task operates exclusively on:

- `crates/community-post-store-domain/src/lib.rs`
- `docs/specs/task-community-vote-tally-retraction.md` (this file)
- `tasks/community-vote-tally-retraction-plan.md`

No other crates, no new workspace members, no root `Cargo.toml`.

---

## Mod Layout (flat clean-arch)

The crate is a single-file library (`src/lib.rs`). All domain types and their `impl` blocks live in this file, consistent with the hyperscaler single-crate-per-service + mod-based subsystems doctrine (ADR-0509). No new files are needed for this task.

```
src/
  lib.rs        ← all domain types + impl blocks + unit tests
```

---

## Type Contracts

### CommunityError additions

```rust
pub enum CommunityError {
    Invalid,
    MissingAnonymousDisclosurePolicy,
    SelfVoteForbidden,
    DuplicateVote,
    ModerationNeedsEvidence,
    NoSuchVote,                         // NEW — cvt-1
}
```

### VoteLedger::retract (cvt-1)

```rust
impl VoteLedger {
    /// Remove a previously cast vote receipt for the given voter.
    ///
    /// # Errors
    /// - `CommunityError::Invalid` — `voter_ref` is empty or whitespace.
    /// - `CommunityError::NoSuchVote` — no receipt with this `voter_ref` in the ledger.
    pub fn retract(
        &mut self,
        voter_ref: &str,
        post: &CommunityPost,
    ) -> Result<(), CommunityError>;
}
```

Invariants:
- Uses the existing `ne()` guard for empty-string validation.
- Finds the receipt by matching `receipt.voter_ref == voter_ref` and removes it from `self.receipts.value`.
- Does not re-check self-vote; that guard belongs on `cast`.

### VoteLedger::tally (cvt-2)

```rust
impl VoteLedger {
    /// Net score: count of Up receipts minus count of Down receipts.
    ///
    /// Returns `0` for an empty ledger. Deterministic because receipts are
    /// stored in a `BTreeSet` ordered by `(voter_ref, vote_id, kind)`.
    pub fn tally(&self) -> i64;
}
```

Invariants:
- Pure read; takes `&self`.
- No sorting needed; `BTreeSet` guarantees stable order.
- Result type is `i64` (net can be negative).

### moderation_case (cvt-3)

```rust
/// Create a moderation audit record for a post.
///
/// # Action semantics
/// - `Allow` — requires only non-empty `policy_ref`; `evidence_ref` may be
///   empty; returns `Classified<String>` tagged `DataClass::InternalOnly`.
/// - `Hide` — requires both non-empty `policy_ref` and non-empty
///   `evidence_ref`; returns `Classified<String>` tagged `DataClass::Audit`.
/// - `Remove` — same requirements as `Hide` (non-empty evidence mandatory);
///   returns `Classified<String>` tagged `DataClass::Audit`.
///
/// # Errors
/// - `CommunityError::Invalid` — `policy_ref` is empty for any action.
/// - `CommunityError::ModerationNeedsEvidence` — `evidence_ref` is empty
///   when action is `Hide` or `Remove`.
pub fn moderation_case(
    post: &CommunityPost,
    action: ModerationAction,
    policy_ref: String,
    evidence_ref: String,
) -> Result<Classified<String>, CommunityError>;
```

The `let _ = action;` discard is removed. The function branches on `action` and classifies the returned audit ref accordingly.

---

## Data Classification

| Return path         | Tag                   | Rationale                                      |
|---------------------|-----------------------|------------------------------------------------|
| `Remove` / `Hide`   | `DataClass::Audit`    | Evidence ref is an audit record requiring audit-class tagging |
| `Allow`             | `DataClass::InternalOnly` | Policy ref is internal metadata, not audit evidence |

Both `DataClass::Audit` and `DataClass::InternalOnly` are existing variants in `oya-data-boundary-kernel`. `DataClass::Audit` converts to `DataClassification::Operational(OperationalDataClass::Audit)` via `DataClassification::from_data_class`.

---

## Testing Strategy

All tests live in the `#[cfg(test)] mod tests` block at the bottom of `src/lib.rs`, following existing crate conventions. The `#![cfg_attr(test, allow(clippy::unwrap_used, ...))]` gate already permits `.unwrap()` in test helpers.

### cvt-1 tests
- `retract_removes_existing_receipt` — cast then retract; assert `receipts` is empty.
- `retract_second_time_errors_no_such_vote` — retract twice; second → `NoSuchVote`.
- `retract_empty_voter_ref_errors_invalid` — empty string → `Invalid`.
- `cast_still_raises_duplicate_vote` — unmodified idempotency regression.

### cvt-2 tests
- `tally_empty_ledger_is_zero` — `tally() == 0`.
- `tally_mixed_receipts_net_score` — 3 Up + 2 Down → `1`.
- `tally_all_up` — 2 Up → `2`.
- `tally_all_down` — 2 Down → `-2`.

### cvt-3 tests
- `moderation_remove_without_evidence_errors` — `Remove` + `""` → `ModerationNeedsEvidence`.
- `moderation_remove_with_evidence_tagged_audit` — `Remove` + evidence → `Audit` classification.
- `moderation_allow_passes_without_evidence` — `Allow` + `""` evidence → `Ok`.
- `moderation_allow_tagged_internal_only` — `Allow` result has `InternalOnly` classification.
- `moderation_hide_still_requires_evidence` — `Hide` + `""` → `ModerationNeedsEvidence` (regression).

---

## Verification Gates

```
cargo check -p community-post-store-domain --all-targets
cargo nextest run -p community-post-store-domain
```

Both must be green. No bare `cargo check --workspace` (masks test/feature breaks per project rules).

---

## Non-goals

- No proto3 / OpenAPI contracts — this is pure domain logic with no HTTP/gRPC surface in this crate.
- No async, no I/O.
- No new crates or workspace members.
- No changes to any other crate.
