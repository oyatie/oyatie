# Task Plan: community-vote-tally-retraction

**Vertical:** community  
**Crate:** `oya-community-post-store-domain`  
**Branch:** `feat/task-community-vote-tally-retraction-2026-05-28`

## Objective

Extend the community post-store domain crate with three targeted additions:

1. Vote retraction on `VoteLedger` (remove a previously cast receipt).
2. Net-score tally on `VoteLedger` (upvotes − downvotes, pure and deterministic).
3. Action-aware `moderation_case` — branch on `ModerationAction::Remove` vs `Allow`/`Hide`, remove the `let _ = action;` discard, enforce evidence for `Remove`, tag the audit ref correctly.

All additions are pure domain logic (no I/O, no new crates, no root `Cargo.toml` edits).

---

## Subtasks

### cvt-1 — VoteLedger::retract

**What:** Add `VoteLedger::retract(&mut self, voter_ref: &str, post: &CommunityPost) -> Result<(), CommunityError>`.

**Rules:**
- Returns `CommunityError::Invalid` when `voter_ref` is empty/whitespace.
- Adds new variant `CommunityError::NoSuchVote` returned when no receipt with the given `voter_ref` exists in `receipts`.
- Removes the matching receipt on success.
- Existing `cast` guards (self-vote, duplicate) are untouched.

**Tests required:**
- `retract_removes_existing_receipt` — cast then retract; receipts set is empty afterward.
- `retract_second_time_errors_no_such_vote` — cast, retract, retract again → `NoSuchVote`.
- `cast_still_raises_duplicate_vote` — existing idempotency contract unbroken after retract is added.
- `retract_empty_voter_ref_errors_invalid` — empty string → `CommunityError::Invalid`.

**Accept:** `cargo check -p oya-community-post-store-domain --all-targets` passes; `cargo nextest run -p oya-community-post-store-domain` green.

---

### cvt-2 — VoteLedger::tally

**What:** Add `VoteLedger::tally(&self) -> i64` computing `upvotes − downvotes` over all receipts.

**Rules:**
- Deterministic regardless of insertion order (receipts stored in `BTreeSet`, so order is stable).
- Empty ledger tallies to `0`.
- Return type is `i64` (no new struct needed; the subtask says "or a NetScore struct" — keep it simple with `i64`).

**Tests required:**
- `tally_empty_ledger_is_zero` — `VoteLedger::new(&post).tally() == 0`.
- `tally_mixed_receipts_net_score` — construct a ledger with 3 Up + 2 Down votes; assert tally == `1`.
- `tally_all_up` — 2 Up → `2`.
- `tally_all_down` — 2 Down → `-2`.

**Accept:** `cargo nextest run -p oya-community-post-store-domain` green.

---

### cvt-3 — Action-aware moderation_case

**What:** Remove `let _ = action;` and branch on `ModerationAction`.

**Rules:**
- `ModerationAction::Remove` — requires non-empty `evidence_ref`; returns `CommunityError::ModerationNeedsEvidence` if empty; on success returns `Classified<String>` tagged `DataClass::Audit`.
- `ModerationAction::Allow` — passes with `policy_ref` only (evidence_ref may be empty); returns `Classified<String>` tagged `DataClass::InternalOnly`.
- `ModerationAction::Hide` — keeps existing behavior (evidence required, tagged `DataClass::Audit`). *(Current blanket logic already requires non-empty evidence and tags Audit; Hide retains this.)*
- `policy_ref` must remain non-empty for all paths.

**Tests required:**
- `moderation_remove_without_evidence_errors` — `Remove` + empty `evidence_ref` → `ModerationNeedsEvidence`.
- `moderation_remove_with_evidence_tagged_audit` — `Remove` + non-empty evidence → `Classified` with `DataClass::Audit` classification.
- `moderation_allow_passes_without_evidence` — `Allow` + empty evidence_ref + non-empty `policy_ref` → `Ok`.
- `moderation_allow_tagged_internal_only` — `Allow` result has `DataClass::InternalOnly` classification.
- `moderation_hide_still_requires_evidence` — existing Hide behavior regression test (preserve existing test or add explicit one).

**Accept:** `cargo check -p oya-community-post-store-domain --all-targets` + `cargo nextest run -p oya-community-post-store-domain` both green.

---

## Acceptance Summary

| Subtask | Gate |
|---------|------|
| cvt-1   | `cargo check --all-targets` green; 4 new tests green |
| cvt-2   | `cargo nextest run` green; 4 new tally tests green |
| cvt-3   | `cargo check --all-targets` + `cargo nextest run` green; 5 new/updated tests green |

No new crates. No root `Cargo.toml` edits. No I/O.
