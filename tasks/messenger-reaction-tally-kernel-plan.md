# Plan: messenger-reaction-tally-kernel

## Objective
Add a `reaction_tally` module (pure kernel, no I/O) to `oya-messenger-domain` that builds on
`MessageReaction`/`ReactionSet` and provides deterministic, BTreeMap-ordered per-emoji aggregation
plus an idempotent `toggle(actor, emoji)` operation.

## Requirements Analysis

### Core Behaviour
- `ReactionTally` ingests a slice of `&MessageReaction` or borrows a `&ReactionSet` and produces
  a BTreeMap keyed by emoji with `TallyEntry { count, actors: BTreeSet<String> }` per key.
- `toggle(actor_ref, emoji)` adds the pair when absent, removes it when present (idempotent).
  Re-derives `count` deterministically from the actor roster after every mutation.
- `ReactionTally::from_reactions(slice)` and `ReactionTally::from_reaction_set(set, message_id)`
  are the two construction paths.

### Data-Class Tagging (Classified wrappers)
- `TallyEntry.actors`: `Classified<BTreeSet<String>>` tagged `PII_IDENTIFYING` (actor refs).
- `TallyEntry.count`: `Classified<u64>` tagged `INTERNAL_ONLY`.
- The emoji key itself lives in the BTreeMap key (plain `String`, not wrapped), because it is the
  map discriminant, not stored as a standalone field.
- `ReactionTally` is a plain struct (no top-level Classified wrapper needed).

### Edge Cases
- Empty input → empty tally.
- Duplicate `(actor, emoji)` in input slice: same actor+emoji pair is counted once
  (idempotent — consistent with `ReactionSet` semantics).
- Toggle add then remove → round-trip back to prior state.
- Toggle is idempotent per direction: two removes leave the entry absent (not error).

### No I/O
No async, no trait objects, no serde, no external crates beyond what already exists in
`oya-messenger-domain`'s Cargo.toml.

## Subtasks (ordered)

1. [x] Write `tasks/messenger-reaction-tally-kernel-plan.md` (this file).
2. [ ] Write `docs/specs/task-messenger-reaction-tally-kernel.md`.
3. [ ] Write RED tests in `crates/oya-messenger-domain/src/reaction_tally.rs` — confirm
       `cargo check -p oya-messenger-domain --all-targets` fails (missing module).
4. [ ] Implement `ReactionTally` + `TallyEntry` in `reaction_tally.rs`.
5. [ ] Wire `pub mod reaction_tally; pub use reaction_tally::*;` in `lib.rs`.
6. [ ] Verify GREEN: `cargo check -p oya-messenger-domain --all-targets` +
       `cargo nextest run -p oya-messenger-domain`.
7. [ ] Self-review (correctness / security / cloud-native-readiness).
8. [ ] Simplify pass; re-run nextest.
9. [ ] Commit + push + PR.

## Acceptance Criteria
- New module wired in `lib.rs` (pub mod + re-export).
- Tally over mixed reactions yields stable emoji-ordered counts.
- `toggle` add-then-remove returns to prior state (round-trip).
- Duplicate `(actor, emoji)` never double-counts.
- Empty input yields empty tally.
- All assertions via `#[cfg(test)]` unit tests.
- No new workspace member, no root Cargo.toml edit, no async/I/O.
