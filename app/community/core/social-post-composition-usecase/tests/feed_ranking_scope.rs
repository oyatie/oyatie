//! Integration tests for feed-ranking scope-filtering (ST2 acceptance criterion).
//!
//! Acceptance: "posts outside ctx.scope_ref are filtered out" — cross-scope/cross-tenant
//! isolation must be enforced by rank_feed, not assumed from the caller.
//!
//! These tests are RED: FeedRankInput currently has no scope_ref field and rank_feed
//! performs no scope filtering.  They MUST fail until the GREEN stage adds:
//!   1. A `scope_ref: String` field to FeedRankInput.
//!   2. Filtering logic in rank_feed that drops posts whose scope_ref != ctx.scope_ref.

use community_social_post_composition_api::{AuthorizedSocialContext, SocialApiContext};
use community_social_post_composition_usecase::feed_ranking::{FeedRankInput, rank_feed};

fn personal_ctx(scope: &str) -> AuthorizedSocialContext {
    AuthorizedSocialContext {
        context: SocialApiContext::Personal,
        scope_ref: format!("person:{}", scope),
        principal_ref: format!("user:{}", scope),
        idempotency_key: "idem".into(),
        policy_decision_ref: "cedar:allow:feed-rank".into(),
        audit_correlation_id: "audit".into(),
    }
}

fn make_scoped_post(
    post_id: &str,
    scope_ref: &str,
    created_at: u64,
    engagement_count: u64,
) -> FeedRankInput {
    FeedRankInput {
        post_id: post_id.into(),
        scope_ref: scope_ref.into(),
        created_at,
        engagement_count,
    }
}

/// ST2 core: rank_feed must exclude posts whose scope_ref does not match ctx.scope_ref.
/// Cross-tenant leakage is a security violation — posts from tenant:other must never
/// appear in a feed ranked for person:alice.
#[test]
fn rank_feed_excludes_posts_from_different_scope() {
    let now = 1_000_000u64;
    let ctx = personal_ctx("alice");
    let posts = vec![
        make_scoped_post("mine", "person:alice", now - 100, 50),
        make_scoped_post("theirs", "person:bob", now - 100, 50), // different scope — must be excluded
        make_scoped_post("tenant-post", "tenant:acme", now - 100, 50), // wrong context type — must be excluded
    ];
    let result = rank_feed(&ctx, &posts, now).unwrap();
    assert_eq!(
        result.len(),
        1,
        "only posts matching ctx.scope_ref should appear"
    );
    assert_eq!(result[0].post_id, "mine");
}

/// ST2 boundary: all posts in-scope are retained and ordered correctly.
#[test]
fn rank_feed_retains_all_in_scope_posts() {
    let now = 1_000_000u64;
    let ctx = personal_ctx("alice");
    let posts = vec![
        make_scoped_post("p1", "person:alice", now - 500, 10),
        make_scoped_post("p2", "person:alice", now - 100, 200),
        make_scoped_post("out", "person:bob", now - 50, 9999), // high score but wrong scope
    ];
    let result = rank_feed(&ctx, &posts, now).unwrap();
    assert_eq!(result.len(), 2, "both in-scope posts retained");
    // p2 should rank first (fresher + more engagement)
    assert_eq!(result[0].post_id, "p2");
    assert_eq!(result[1].post_id, "p1");
}

/// ST2 edge: when all posts are out-of-scope the result is an empty vec (not an error).
#[test]
fn rank_feed_all_out_of_scope_returns_empty() {
    let now = 1_000_000u64;
    let ctx = personal_ctx("alice");
    let posts = vec![
        make_scoped_post("x", "person:bob", now - 100, 50),
        make_scoped_post("y", "tenant:acme", now - 200, 100),
    ];
    let result = rank_feed(&ctx, &posts, now).unwrap();
    assert!(
        result.is_empty(),
        "all out-of-scope posts must be dropped, result must be empty"
    );
}

/// ST2 + ST3: scope filtering is stable — repeated calls with same input produce identical output.
#[test]
fn rank_feed_scope_filter_is_deterministic_across_repeated_calls() {
    let now = 1_000_000u64;
    let ctx = personal_ctx("alice");
    let posts = vec![
        make_scoped_post("c", "person:alice", now - 5000, 300),
        make_scoped_post("a", "person:alice", now - 1000, 100),
        make_scoped_post("b", "person:alice", now - 3000, 300),
        make_scoped_post("out", "person:bob", now - 10, 9999),
    ];
    let first = rank_feed(&ctx, &posts, now).unwrap();
    let second = rank_feed(&ctx, &posts, now).unwrap();
    assert_eq!(
        first, second,
        "scope-filtered ranking must be byte-identical on repeated calls"
    );
    // out-of-scope post must not appear
    assert!(first.iter().all(|p| p.post_id != "out"));
}
