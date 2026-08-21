# Spec: social-feed-ranking-score

**Status**: draft  
**Vertical**: community  
**Crate**: `community-social-post-composition-usecase` (`crates/community-social-post-composition-usecase/`)  
**Branch**: `feat/task-social-feed-ranking-score-2026-05-28`  
**ADR authority**: ADR-0509 (flat single-crate service), ADR-0131 (flat layout)

---

## Objective

Extend the `community-social-post-composition-usecase` crate with a deterministic feed-ranking usecase. The new `feed_ranking` module provides:

1. `FeedRankInput` — a plain data struct carrying post reference, creation timestamp, and bounded engagement counter.
2. `score(input, now) -> u64` — a pure, integer-only score function combining recency decay and capped engagement signal.
3. `rank_feed(ctx, posts, now) -> Result<Vec<FeedRankInput>, SocialUsecaseError>` — orders a caller-supplied post slice by descending score with a deterministic stable tiebreak.

No I/O, no new crate, no root `Cargo.toml` edit.

---

## Vertical and ownership

| Attribute | Value |
|---|---|
| Vertical | community |
| Owning crate | `community-social-post-composition-usecase` |
| Lib name | `community_social_post_composition_usecase` |
| New module | `feed_ranking` (additive; `src/feed_ranking.rs` + `pub mod feed_ranking;` in `lib.rs`) |
| Existing dependencies | `community-social-domain`, `community-social-post-composition-api` |

---

## Mod layout (flat clean-arch)

```
crates/community-social-post-composition-usecase/
  src/
    lib.rs            ← add `pub mod feed_ranking;`; existing functions untouched
    feed_ranking.rs   ← new: FeedRankInput, score(), rank_feed()
```

No new crates. No changes to `Cargo.toml`. The single-`lib.rs` convention is extended with one additive module file, matching the flat clean-arch doctrine.

---

## Score formula

The score is computed in pure integer arithmetic — no floats, no platform-specific behaviour.

```
RECENCY_WEIGHT  = 86_400   (seconds in one day; mirrors story-purge TTL time model)
ENGAGEMENT_CAP  = 10_000   (bounds engagement signal; prevents score domination)

age_secs             = now.saturating_sub(created_at)
recency_component    = RECENCY_WEIGHT.saturating_sub(age_secs.min(RECENCY_WEIGHT))
engagement_component = engagement_count.min(ENGAGEMENT_CAP)

score = recency_component + engagement_component
```

### Properties

| Property | Guarantee |
|---|---|
| Monotonic in engagement | At fixed `now` and `created_at`, increasing `engagement_count` never decreases score (up to cap) |
| Monotonic-decreasing in age | At fixed engagement, increasing `age_secs` never increases score |
| Bounded | `0 ≤ score ≤ 96_400` (RECENCY_WEIGHT + ENGAGEMENT_CAP) |
| Integer-only | No floats; identical output on every platform and compiler version |
| Saturating arithmetic | No panics on overflow; `saturating_sub` used throughout |

---

## Public contract

### Types

```rust
/// Input to the feed-ranking usecase.
///
/// The caller is responsible for scoping this slice to the authorized context
/// before passing to `rank_feed`. Posts with an empty `post_id` are excluded
/// from the ranked output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeedRankInput {
    /// Opaque post identifier; used as stable tiebreak key (lexicographic ascending).
    pub post_id: String,
    /// Creation timestamp as Unix epoch seconds. Mirrors the `now: u64` time model
    /// used by `story_purge` / `plan_story_purge` in the same crate.
    pub created_at: u64,
    /// Bounded engagement signal (e.g. likes + comments + shares aggregated by caller).
    /// Values above ENGAGEMENT_CAP are treated as ENGAGEMENT_CAP.
    pub engagement_count: u64,
}
```

### Functions

```rust
/// Returns the deterministic integer score for a single post at a given wall-clock instant.
///
/// See crate-level formula documentation for RECENCY_WEIGHT, ENGAGEMENT_CAP, and
/// the monotonicity guarantees.
pub fn score(input: &FeedRankInput, now: u64) -> u64

/// Rank a slice of feed posts by descending score, with stable tiebreak on `post_id`.
///
/// # Context guard
/// `ctx.validate()` is called first. Any validation error is surfaced as
/// `Err(SocialUsecaseError::Api(...))`.
///
/// # Scope
/// The input slice is assumed to be pre-scoped by the caller (the authorized context
/// asserts session scope). Posts with an empty `post_id` are silently excluded.
///
/// # Ordering
/// Primary: descending `score(post, now)`.
/// Tiebreak: ascending `post_id` (lexicographic) — deterministic across repeated calls.
///
/// # Empty input
/// An empty slice returns `Ok(vec![])`.
pub fn rank_feed(
    ctx: &AuthorizedSocialContext,
    posts: &[FeedRankInput],
    now: u64,
) -> Result<Vec<FeedRankInput>, SocialUsecaseError>
```

### Error variants

| Variant | Trigger |
|---|---|
| `SocialUsecaseError::Api(SocialApiError::*)` | `ctx.validate()` fails (missing scope, principal, idempotency key, etc.) |

No new error variants are introduced.

---

## OpenAPI 3.2.0 surface

`rank_feed` is a pure in-process usecase function. No REST endpoint is added in this task. When a REST adapter exposes a feed-ranking endpoint in a future task, the adapter will define an OpenAPI 3.2.0 operation referencing this usecase. The anticipated shape is:

```yaml
# future adapter — informational only
paths:
  /social/feed/ranked:
    get:
      operationId: getRankedFeed
      summary: Return ranked feed posts for the authorized context
      parameters:
        - name: now
          in: query
          required: false
          schema: { type: integer, format: int64 }
      responses:
        "200":
          description: Ranked feed
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: "#/components/schemas/RankedFeedPost"
```

---

## proto3 surface

No gRPC surface is added in this task. Future gRPC adapter anticipated shape (informational):

```proto
// future adapter — informational only
syntax = "proto3";
package oya.social.v1;

service FeedRanking {
  rpc GetRankedFeed(RankedFeedRequest) returns (RankedFeedResponse);
}

message FeedRankEntry {
  string post_id        = 1;
  uint64 created_at     = 2;
  uint64 engagement_count = 3;
}

message RankedFeedRequest {
  repeated FeedRankEntry posts = 1;
  uint64 now                   = 2;
}

message RankedFeedResponse {
  repeated FeedRankEntry ranked_posts = 1;
}
```

---

## AsyncAPI 3.1.0 surface

No event channel is added in this task. Feed-ranking is a synchronous query; no event emission is expected.

---

## Testing strategy

All tests live in `src/feed_ranking.rs` inside `#[cfg(test)] mod tests`.

### ST1 tests — score monotonicity

| Test | Assertion |
|---|---|
| `score_monotonic_in_engagement_at_fixed_time` | Higher `engagement_count` → same or higher score (below cap) |
| `score_monotonic_decreasing_in_age_at_fixed_engagement` | Older `created_at` (larger age) → same or lower score |

### ST2 tests — rank_feed ordering and filtering

| Test | Assertion |
|---|---|
| `rank_feed_descending_score_ordering` | Result ordered highest score first |
| `rank_feed_stable_tiebreak_on_equal_score` | Equal-score posts ordered by ascending `post_id` |
| `rank_feed_excludes_empty_post_id` | Posts with `post_id == ""` absent from output |

### ST3 tests — edge cases

| Test | Assertion |
|---|---|
| `rank_feed_empty_returns_empty` | `&[]` → `Ok(vec![])` |
| `rank_feed_single_post_returns_single` | One-element slice → one-element result |
| `rank_feed_identical_input_same_ordering` | Two calls with identical input produce identical `post_id` ordering |

---

## Acceptance gate

```sh
cargo check -p community-social-post-composition-usecase --all-targets
cargo nextest run -p community-social-post-composition-usecase
```

Both commands must exit 0 with all tests passing.

---

## Boundaries and constraints

- **No new crate** — all additions are inside `community-social-post-composition-usecase`.
- **No root `Cargo.toml` edit** — workspace manifest is untouched.
- **No I/O** — `score` and `rank_feed` are pure functions.
- **No adjacent refactoring** — existing `compose_post`, `plan_story_purge`, and helper functions are not modified.
- **No floats** — integer-only arithmetic throughout; determinism is a hard requirement.
- **Hyperscaler-lens** — no new OSS dependencies introduced.
- **Reuse** — `AuthorizedSocialContext` and `SocialUsecaseError` reused directly from existing usecase and API crates.
