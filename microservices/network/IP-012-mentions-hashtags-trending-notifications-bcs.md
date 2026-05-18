---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-network-foundation
impl_plan_id: IP-012-mentions-hashtags-trending-notifications-bcs
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-network
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-shardability]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012: mentions + hashtags + trending-topics + notifications BCs end-to-end

## Intent

Land four BCs together (engagement-fanout pipeline):

- `mentions`: @-mention parse; Ontology lookup over Person + Company + Skill + Hashtag; fanout to notifications + cross-µservice bridges.
- `hashtags`: #tag parse; per-tag corpus; Professional-context trending input emission.
- `trending-topics`: Windowed trend compute over hashtags + entities; per-tenant per-pack ranking; sybil-detector verdict applied per FM-18 mitigation.
- `notifications`: Real-time WebSocket + digest worker; per-recipient idempotency; backpressure-coalesced; sharded fanout for 30k-300k connection accounts.

## Code Shape

```rust
// notifications kernel/src/ports.rs
#[async_trait]
pub trait NotificationDispatcher: Send + Sync {
    async fn dispatch(&self, n: Notification) -> Result<(), NotifError>;
    async fn subscribe(&self, tenant_id: &TenantId, user: &UserRef, from_seq: u64) -> impl Stream<Item = Notification>;
    async fn coalesce_digest(&self, tenant_id: &TenantId, user: &UserRef, window: Duration) -> Result<DigestBucket, NotifError>;
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-network-mentions-kernel
cargo nextest run -p oya-network-hashtags-kernel
cargo nextest run -p oya-network-trending-topics-kernel
cargo nextest run -p oya-network-notifications-kernel
cargo run -p oya-dev-cli -- gate validate shardability --microservice network
```

## Test Plan

- @mention of Person + Company + Skill + Hashtag resolves to Ontology object; fan out notification.
- Hashtag corpus accumulates posts; trending-topics worker computes top-N over 1h + 24h windows.
- Notification fanout for 30k-follower account: p99 ≤ 2s per `slos/notification-fanout-latency.openslo.yaml`.
- Notification fanout for 300k-follower account: shard across cells.
- Sybil-detector verdict applied: synthetic-trend dropped from trending output.
- Per-recipient idempotency: replayed notification de-duplicates.

## Halt Conditions

- Notification fanout p99 > 2s on 30k-follower account — escalate to capacity-model review; add notification-worker replicas.

## Next IP

[`IP-013-search-and-cedar-filter.md`](IP-013-search-and-cedar-filter.md)

## References

- ADR-NET-0001 (storage).
- `microservices/network/capacity-model.md` §"Notification Fanout Sizing".
- `microservices/network/runbooks/endorsement-storm-throttle.md` (paired sybil-defense pattern).
