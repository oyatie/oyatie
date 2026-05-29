# Plan: community-vote-ledger-hot-controversy-ranking

## Objective

Add a deterministic integer-only Reddit-style ranking kernel to `oya-community-post-store-domain`:
- `hot_score(&self, created_at: u64, now: u64) -> i64` on `VoteLedger`
- `controversy_score(&self) -> u64` on `VoteLedger`
- `rank_posts(&[(post_id, &VoteLedger, created_at)], now) -> Vec<String>` pure fn

## Requirements Analysis

### hot_score
- Blends net tally (up - down) with age-decay recency term
- Mirrors `feed_ranking.rs` RECENCY_WEIGHT=86_400 model
- Saturating integer arithmetic only — no floats, no panic on overflow
- Net tally reuses existing `tally()` method
- Formula: `tally().saturating_add(recency_term as i64)` where recency_term = `RECENCY_WEIGHT.saturating_sub(age_secs.min(RECENCY_WEIGHT))`
- Returns `i64` to support negative net tallies

### controversy_score
- Rewards near-equal up/down split
- Formula: `min(up_count, down_count)` — 0 when all one-directional, maximal at equal split
- Returns `u64` (always non-negative)

### rank_posts
- Input: slice of `(post_id: &str, ledger: &VoteLedger, created_at: u64)`
- Output: `Vec<String>` of post_ids ordered by `hot_score` descending
- Tiebreak: ascending post_id (lexicographic, stable)
- Excludes entries with empty post_id
- Pure/deterministic — no I/O, no async

### Edge Cases
- Empty ledger: `hot_score` = recency only, `controversy_score` = 0
- All-up ledger: `controversy_score` = 0 (min(n,0) = 0)
- All-down ledger: `controversy_score` = 0
- Equal up/down: `controversy_score` = n/2 (maximal for n total votes)
- post older than RECENCY_WEIGHT: recency_term = 0
- Overflow: saturating_add/saturating_sub throughout
- Stable tiebreak: ascending post_id

## Subtasks

1. [ ] Write plan (this file)
2. [ ] Write spec at `docs/specs/task-community-vote-ledger-hot-controversy-ranking.md`
3. [ ] Add `ranking` mod to `src/lib.rs` with failing tests (red phase)
4. [ ] Implement `hot_score`, `controversy_score`, `rank_posts` (green phase)
5. [ ] Verify `cargo check -p oya-community-post-store-domain --all-targets`
6. [ ] Verify `cargo nextest run -p oya-community-post-store-domain`
7. [ ] Self-review (correctness / architecture / security / perf / cloud-native)
8. [ ] Simplify pass; re-verify
9. [ ] Commit + push + PR

## Acceptance Criteria

- net upvote raises `hot_score` compared to equal vote count
- older posts decay monotonically at fixed tally
- `controversy_score` is 0 when all votes one-directional
- `controversy_score` is maximal at equal up/down
- `rank_posts` is deterministic across repeated calls
- `rank_posts` excludes empty post_id
- all-integer saturating math, no panic on overflow
- >= 8 unit tests covering: monotonicity, decay floor, controversy symmetry, empty-ledger, stable-tiebreak
