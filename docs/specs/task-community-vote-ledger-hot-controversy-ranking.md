# Spec: community-vote-ledger-hot-controversy-ranking

## Objective

Add a deterministic, integer-only Reddit-style hot/controversy ranking kernel
to `community-post-store-domain`. Pure domain logic, no I/O, no async, no
new crate dependencies.

## Crate Boundary

All changes confined to `crates/community-post-store-domain/`.
No root `Cargo.toml` edit. No new workspace member.

## Mod Layout (flat clean-arch)

```
crates/community-post-store-domain/src/
  lib.rs          (existing — add `pub mod ranking; pub use ranking::rank_posts;`)
  ranking.rs      (NEW — hot_score, controversy_score, rank_posts)
```

## Public API Contracts

### `VoteLedger::hot_score(&self, created_at: u64, now: u64) -> i64`

Blends the net vote tally with a recency decay term. Integer-only saturating
arithmetic; mirrors the 86_400s recency-weight model.

- `age_secs = now.saturating_sub(created_at)`
- `recency_term = RECENCY_WEIGHT.saturating_sub(age_secs.min(RECENCY_WEIGHT))`
  where `RECENCY_WEIGHT: u64 = 86_400`
- `hot_score = tally().saturating_add(recency_term as i64)`

Properties:
- Net upvote raises `hot_score` vs zero-vote post at same age.
- Older posts decay monotonically: at fixed tally, `hot_score` decreases as
  `age_secs` increases until `recency_term` floors at 0.
- No panic on any `u64`/`i64` input combination.

### `VoteLedger::controversy_score(&self) -> u64`

Rewards near-equal up/down vote splits.

- `up = count of VoteKind::Up receipts`
- `down = count of VoteKind::Down receipts`
- `controversy_score = min(up, down).saturating_mul(up.saturating_add(down))`

Properties:
- 0 when all votes are one-directional (min == 0).
- Strictly greater than 0 whenever both up and down are non-zero.
- Monotonically increases as the split becomes more balanced at equal total.
- No panic; saturating mul guards u64 overflow.

### `pub fn rank_posts(entries: &[(&str, &VoteLedger, u64)], now: u64) -> Vec<String>`

Returns post_ids ordered by `hot_score` descending with stable ascending
lexicographic post_id tiebreak. Excludes entries with empty (blank) post_id.

- Input tuple: `(post_id, ledger, created_at)`.
- Deterministic: same input always yields identical output ordering.
- Pure: no I/O, no side effects.

## Contracts / Observability / SLO

This is a pure domain kernel. No HTTP, gRPC, or event-bus surface. No SLO
authoring required for a domain-only slice (per ADR-0130 applicability scope:
SLO required for promoted service endpoints, not for isolated domain functions).

## Testing Strategy

All tests in `#[cfg(test)]` inline module in `ranking.rs`.

Required test cases (>= 8):

1. `hot_score_net_upvote_raises_score` — post with 5 up vs 0 down scores higher
   than 0-vote post at same age.
2. `hot_score_decay_monotonic` — same tally, older post scores lower.
3. `hot_score_decay_floor` — post older than RECENCY_WEIGHT has recency_term=0;
   score equals tally only.
4. `hot_score_empty_ledger` — empty ledger at `now == created_at` gives
   `RECENCY_WEIGHT as i64`.
5. `controversy_score_zero_when_one_directional` — all-up or all-down gives 0.
6. `controversy_score_maximal_at_equal_split` — equal up/down gives higher
   score than lopsided split with same total.
7. `rank_posts_deterministic_stable_tiebreak` — two posts with identical
   hot_score: lower lexicographic post_id ranked first.
8. `rank_posts_excludes_empty_post_id` — entry with `""` post_id omitted.
9. `rank_posts_higher_hot_score_wins` — post with more recent/higher tally
   ranked first.
10. `controversy_score_no_panic_on_large_counts` — saturating mul with large
    up/down counts does not panic.

## Security / Privacy

- `hot_score` and `controversy_score` operate on aggregate counts only — no
  per-voter data is exposed by the ranking functions.
- `rank_posts` accepts `&VoteLedger` references; callers retain ownership and
  `Classified` tagging of the underlying data.

## Cloud-Native Readiness

- Pure kernel: embeds in any service binary without runtime config.
- No heap allocation beyond the output `Vec<String>`.
- Deterministic output supports reproducible audit trails.
