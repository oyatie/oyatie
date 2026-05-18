---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-network-foundation
impl_plan_id: IP-006-feed-timeline-and-reactions-bcs
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-network
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-shardability, oya-governance-statelessness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: feed-timeline + reactions BCs end-to-end

## Intent

Land both BCs:

- `feed-timeline`: Redis hot-cache + Postgres canonical; fanout-on-write for hot-tier accounts (>10k connections), fanout-on-read for cold-tier; heuristic ranker in P01 (ML scheduled-for-distinct-tracked-work to P03 per sibling ADR-SOC-0001 pattern); chronological fallback; ranker explanation API per EU DSA Art. 27.
- `reactions`: extended Professional set (like, celebrate, insightful, curious, funny, support, love); conflict-free counter; Redis-buffered + Postgres flush.

## Code Shape

```rust
// feed-timeline kernel/src/ports.rs
#[async_trait]
pub trait FeedCache: Send + Sync {
    async fn render(&self, tenant_id: &TenantId, user: &UserRef, mode: FeedMode, limit: u32) -> Result<Feed, FeedError>;
    async fn invalidate(&self, tenant_id: &TenantId, user: &UserRef) -> Result<(), FeedError>;
    async fn fanout_on_write(&self, post: &ProfessionalPost, followers: &[UserRef]) -> Result<(), FeedError>;
}

#[async_trait]
pub trait RankerClient: Send + Sync {
    async fn rank(&self, candidate_posts: &[PostRef], target_user: &UserRef) -> Result<Vec<(PostRef, f64, RankerExplanation)>, RankerError>;
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-network-feed-timeline-kernel
cargo nextest run -p oya-network-feed-timeline-adapter-redis
cargo nextest run -p oya-network-reactions-kernel
cargo run -p oya-dev-cli -- gate validate shardability --microservice network
cargo run -p oya-dev-cli -- gate validate statelessness --microservice network
```

## Test Plan

- Fanout-on-write for hot-tier user with 30k connections: latency p99 ≤ 5s.
- Fanout-on-read for cold-tier user: feed render p95 ≤ 200ms (meets `slos/feed-render-latency.openslo.yaml`).
- Reaction conflict-free counter under concurrent inc/dec: final count matches set membership.
- Ranker explanation returns contributing signals per EU AI Act Art. 50 + EU DSA Art. 27.
- Chronological fallback: always available even when ranker degrades.

## Halt Conditions

- Feed render p95 > 200ms after tuning — escalate to capacity-model review.

## Next IP

[`IP-007-endorsement-engine-bc.md`](IP-007-endorsement-engine-bc.md)

## References

- ADR-NET-0001 (storage); ADR-NET-0002 (ranker bounds; EU AI Act).
- `microservices/network/capacity-model.md` §"Redis Sizing".
- Sibling ADR-SOC-0001 (feed-ranking strategy).
