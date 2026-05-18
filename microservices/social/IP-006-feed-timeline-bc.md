---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-006-feed-timeline-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-shardability, oya-governance-port-location]
---

# IP-006: feed-timeline BC (kernel → domain → usecase → adapter-postgres + adapter-redis + worker + rest + sdk + app)

## Intent

Author the full `feed-timeline` BC: chronological + heuristic-algorithmic feed
materialisation with fanout-on-write for hot-tier accounts (>10k followers) and
fanout-on-read for cold-tier. Redis hot-cache for per-user feed slices; Postgres
authoritative store. EU AI Act Art. 27 ranking-explanation API.

ML-driven ranking is P03 (depends on foundry-runtime); P01 ships chronological
+ heuristic (recency × engagement × follow-proximity).

## ChangeSet boundary

`feed-timeline` BC end-to-end.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-social-feed-timeline-kernel/src/{ports,entities,errors}.rs` | create |
| `src/crates/oya-social-feed-timeline-domain/src/{feed_entry,ranking_signal,fanout_plan,rank_snapshot,ranking_heuristic}.rs` | create |
| `src/crates/oya-social-feed-timeline-usecase/src/{render,fanout_on_write,fanout_on_read}.rs` | create |
| `src/crates/oya-social-feed-timeline-adapter-postgres/src/repository.rs` | create |
| `src/crates/oya-social-feed-timeline-adapter-redis/src/cache.rs` | create |
| `src/crates/oya-social-feed-timeline-worker/src/{fanout_writer,cache_rebuilder}.rs` | create |
| `src/crates/oya-social-feed-timeline-rest/src/handlers.rs` | create |
| `tests/feed_timeline_e2e.rs` | create |

## Code Shape

```rust
// kernel/src/ports.rs
#[async_trait]
pub trait FeedCache: Send + Sync {
    async fn get_slice(&self, tenant_id: &TenantId, user_ref: &UserRef, cursor: Cursor, limit: usize)
        -> Result<FeedSlice, FeedError>;
    async fn write_entry(&self, tenant_id: &TenantId, user_ref: &UserRef, entry: FeedEntry)
        -> Result<(), FeedError>;
    async fn invalidate(&self, tenant_id: &TenantId, user_ref: &UserRef) -> Result<(), FeedError>;
}

// domain/src/ranking_heuristic.rs
pub fn rank_score(post: &Post, recency_minutes: u64, engagement_signal: f64, follow_proximity: f64) -> f64 {
    let recency_decay = 1.0 / (1.0 + (recency_minutes as f64 / 360.0));
    let weighted = 0.5 * recency_decay + 0.3 * engagement_signal + 0.2 * follow_proximity;
    weighted.clamp(0.0, 1.0)
}
```

## Fanout Strategy

| Account follower tier | Strategy | Latency |
|---|---|---|
| < 1k followers (cold) | fanout-on-read | feed-render queries Postgres at fetch time |
| 1k–10k (warm) | hybrid | hot-followers fanout-on-write; cold-followers fanout-on-read |
| > 10k (hot/celebrity) | fanout-on-write | precomputed feed slice in Redis per follower |

## Acceptance Gates

```bash
cargo nextest run -p oya-social-feed-timeline-kernel
cargo nextest run -p oya-social-feed-timeline-domain
cargo nextest run -p oya-social-feed-timeline-adapter-redis
cargo run -p oya-dev-cli -- gate validate shardability --microservice social
```

## Test Plan

- Heuristic ranking unit tests (recency decay, weighted score bounds).
- Fanout-on-write E2E: 10k follower account → precomputed Redis slices verified.
- Fanout-on-read E2E: cold account → on-demand Postgres query under p95 ≤ 200ms.
- Feed-cache invalidation on post-delete + tombstone propagation.
- EU AI Act Art. 27 ranking_explanation API exposes contributing signals.

## Halt Conditions

- Feed slice exceeds memory per Redis shard — re-shard.
- Fanout-on-write queue depth > 100k per cell — escalate; auto-degrade to fanout-on-read.

## Next IP

[`IP-007-reactions-bc.md`](IP-007-reactions-bc.md)

## References

- ADR-SOC-0001 (feed-ranking-algorithm).
- `microservices/social/runbooks/feed-cache-rebuild.md`.
- EU AI Act Arts. 13, 27 + EU DSA Art. 27 (recommender transparency).
