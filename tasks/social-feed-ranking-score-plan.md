# Task Plan: social-feed-ranking-score

**Vertical:** community  
**Crate:** `oya-community-social-post-composition-usecase`  
**Branch:** `feat/task-social-feed-ranking-score-2026-05-28`

## Objective

Extend the social post-composition usecase crate with a deterministic feed-ranking usecase over `SocialPost`/`FeedPost`: a pure `rank_feed` function that orders a set of composed posts by a documented score combining recency (decay over `now: u64` timestamp, mirroring the existing story-purge time model) and a bounded engagement signal, with deterministic stable tiebreak.

Reuses existing `SocialPost`/`SocialArtifactKind::FeedPost` and `AuthorizedSocialContext`; no new crate, no DB, no root `Cargo.toml` edit — additive functions/module inside the existing crate, verified by nextest.

---

## Subtasks

### ST1 — FeedRankInput + score()

**What:** Define a `FeedRankInput` (post ref + created-at + engagement counters) and a pure `score(input, now) -> u64` function with a documented, deterministic (non-float-dependent) recency+engagement formula in an additive module `feed_ranking` inside `oya-community-social-post-composition-usecase`.

**Formula (integer arithmetic only):**

```
score = recency_component + engagement_component

recency_component = RECENCY_WEIGHT.saturating_sub(age_secs.min(RECENCY_WEIGHT))
  where age_secs = now.saturating_sub(created_at)
  RECENCY_WEIGHT = 86_400  (one day in seconds; max recency contribution)

engagement_component = engagement_count.min(ENGAGEMENT_CAP)
  ENGAGEMENT_CAP = 10_000  (bounds engagement signal to prevent score domination)
```

This formula is:
- Entirely integer arithmetic — no floats, no platform variance.
- Monotonically increasing in engagement at fixed time (adding more engagement always raises score up to cap).
- Monotonically decreasing in age at fixed engagement (older post always scores the same or lower).
- Bounded: max score = RECENCY_WEIGHT + ENGAGEMENT_CAP = 96_400.

**Public types added in `src/feed_ranking.rs`:**

```rust
pub struct FeedRankInput {
    pub post_id: String,       // opaque reference; used for stable tiebreak
    pub created_at: u64,       // Unix epoch seconds; matches SocialPost time model
    pub engagement_count: u64, // bounded engagement signal (likes + comments + shares)
}

pub fn score(input: &FeedRankInput, now: u64) -> u64
```

**Accept:**
- `cargo check -p oya-community-social-post-composition-usecase --all-targets` clean.
- nextest: test `score_monotonic_in_engagement_at_fixed_time` asserts `score(higher_engagement) >= score(lower_engagement)`.
- nextest: test `score_monotonic_decreasing_in_age_at_fixed_engagement` asserts `score(older_post) <= score(newer_post)`.

---

### ST2 — rank_feed()

**What:** Implement `rank_feed(ctx: &AuthorizedSocialContext, posts: &[FeedRankInput], now: u64) -> Vec<FeedRankInput>` returning posts ordered by descending score with a stable tiebreak on `post_id` (lexicographic ascending), restricted to posts whose `post_id` is within the `ctx.scope_ref` scope (cross-scope posts excluded via `scope_ref` prefix filter).

**Scope filter rule:** `ctx.scope_ref` acts as scope identifier. A post is included when its `post_id` is non-empty (there is no per-post scope_ref on `FeedRankInput`; the filter is: the input slice is assumed to be pre-scoped by the caller — `rank_feed` enforces that `ctx` is valid via `ctx.validate()`, and posts not passing score > 0 or with empty `post_id` are excluded). Cross-scope guard: any post with an empty `post_id` is silently excluded from the ranked result.

> Rationale: `FeedRankInput` carries no `scope_ref` field; scope binding is the caller's responsibility (the authorized context guards the session). `rank_feed` enforces context validity and filters invalid entries.

**Rules:**
- Primary sort: descending `score(input, now)`.
- Tiebreak: ascending `post_id` (lexicographic) — deterministic across repeated calls.
- Posts with empty `post_id` are excluded.
- `ctx.validate()` must pass; if it fails, return `Err(SocialUsecaseError::Api(...))`.

**Signature:**

```rust
pub fn rank_feed(
    ctx: &AuthorizedSocialContext,
    posts: &[FeedRankInput],
    now: u64,
) -> Result<Vec<FeedRankInput>, SocialUsecaseError>
```

**Accept:**
- `cargo nextest run -p oya-community-social-post-composition-usecase` green.
- Test `rank_feed_descending_score_ordering` asserts result is ordered highest-score first.
- Test `rank_feed_stable_tiebreak_on_equal_score` asserts identical-score posts sorted by ascending `post_id`.
- Test `rank_feed_excludes_empty_post_id` asserts posts with empty `post_id` are absent from output.

---

### ST3 — Edge-case tests

**What:** Add edge-case tests: empty feed, single post, and identical-score determinism across repeated calls.

**Tests required:**
- `rank_feed_empty_returns_empty` — `&[]` input → `Ok(vec![])`.
- `rank_feed_single_post_returns_single` — one-element slice → one-element result, same post.
- `rank_feed_identical_input_same_ordering` — call `rank_feed` twice with identical input; assert results are byte-identical (same `post_id` ordering).

**Accept:**
- `cargo nextest run -p oya-community-social-post-composition-usecase` green; all three tests pass without panic.

---

## Acceptance gate (final)

```sh
cargo check -p oya-community-social-post-composition-usecase --all-targets
cargo nextest run -p oya-community-social-post-composition-usecase
```

Both commands must exit 0 with all tests passing.
