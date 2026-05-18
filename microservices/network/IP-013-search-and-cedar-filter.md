---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-network-foundation
impl_plan_id: IP-013-search-and-cedar-filter
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-network
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-shardability, oya-governance-cedar-policy-spec]
---

# IP-013: search BC end-to-end + multi-index Meilisearch + Cedar post-filter

## Intent

Author the `search` BC with six indexes (people, content, skills, jobs, companies, events). Per `slos/search-people-latency.openslo.yaml`, people-search p95 ≤ 250ms; per `slos/search-content-latency.openslo.yaml`, content-search p95 ≤ 500ms.

Cedar post-filter ensures the user only sees results they are entitled to read; PHI redaction applied in pack-us-healthcare overlay; minor-account excluded from recruiter / salary surfaces.

## Code Shape

```rust
// kernel/src/ports.rs
#[async_trait]
pub trait SearchIndex: Send + Sync {
    async fn search_people(&self, q: &Query) -> Result<Vec<ProfileHit>, SearchError>;
    async fn search_content(&self, q: &Query) -> Result<Vec<PostHit>, SearchError>;
    async fn search_skills(&self, q: &Query) -> Result<Vec<SkillHit>, SearchError>;
    async fn search_jobs(&self, q: &JobsQuery) -> Result<Vec<JobHit>, SearchError>;
    async fn search_companies(&self, q: &Query) -> Result<Vec<CompanyHit>, SearchError>;
    async fn search_events(&self, q: &Query) -> Result<Vec<EventHit>, SearchError>;
    async fn index_document(&self, index: &IndexName, doc: &SearchDoc) -> Result<(), SearchError>;
}

#[async_trait]
pub trait CedarSearchFilter: Send + Sync {
    async fn filter(&self, principal: &Principal, hits: Vec<Hit>) -> Result<Vec<Hit>, SearchError>;
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-network-search-kernel
cargo nextest run -p oya-network-search-adapter-meilisearch
cargo run -p oya-dev-cli -- gate validate shardability --microservice network
cargo run -p oya-dev-cli -- gate validate cedar-policy-spec --microservice network
```

## Test Plan

- Per-index sharding: 6 indexes partitioned per tenant; cross-tenant queries forbidden by Cedar.
- People-search p95 ≤ 250ms over 1M-profile golden-set.
- Content-search p95 ≤ 500ms.
- Jobs-search faceted: location + level + skills filter combinable.
- PHI redaction: pack-us-healthcare overlay strips PHI fields at indexer emission time.
- Minor-account: excluded from people-search results in recruiter context; excluded from salary-insights.
- Backfill-search: per `backfill-replay.md` §"Backfill (search index rebuild — multi-index)" verified.

## Halt Conditions

- Search p95 exceeds SLO target after tuning — add indexer workers + Meilisearch shards.

## Next IP

[`IP-014-recommender-fairness-and-bias-lane.md`](IP-014-recommender-fairness-and-bias-lane.md)

## References

- ADR-NET-0001 (storage).
- `microservices/network/slos/search-people-latency.openslo.yaml`.
- `microservices/network/slos/search-content-latency.openslo.yaml`.
- `microservices/network/policy/data-residency.md` (PHI redaction + minor protection).
- Meilisearch docs `docs.meilisearch.com`.
