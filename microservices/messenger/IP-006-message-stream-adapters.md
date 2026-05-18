---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-team-channels-dm-threads
impl_plan_id: IP-006-message-stream-adapters
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-messenger
acceptance_lanes: [cargo-nextest, e2e-message-roundtrip]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: message-stream adapters (Postgres + Meilisearch + Valkey Streams (Redis wire-compat))

## Intent

Implement `MessageStore` against Postgres; `MessageSearchIndex` against
Meilisearch (fallback Tantivy); `RealtimeBroadcaster` against Valkey Streams (Redis wire-compat)
(fallback KeyDB). Per ADR-0105 Amendment 3 — three backend-qualified
`-adapter-<backend>` crates.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-messenger-message-stream-adapter-postgres/{src,migrations}/...` | create |
| `src/crates/oya-messenger-message-stream-adapter-meilisearch/src/...` | create |
| `src/crates/oya-messenger-message-stream-adapter-redis-streams/src/...` | create |
| `tests/message_roundtrip_e2e.rs` | create — testcontainers Postgres + Meilisearch + Valkey |

## Code Shape

```rust
// adapter-redis-streams/src/lib.rs
pub struct RedisStreamsBroadcaster {
    client: redis::cluster::ClusterClient,
    stream_prefix: String,  // "msg:{pack}:{tenant}:{channel}"
}

#[async_trait]
impl RealtimeBroadcaster for RedisStreamsBroadcaster {
    async fn broadcast(&self, event: BroadcastEvent) -> Result<(), BroadcastError> {
        let key = format!("{}:{}:{}", self.stream_prefix, event.tenant_id, event.channel_id);
        self.client.xadd(&key, "*", &event.fields()).await?;
        Ok(())
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-messenger-message-stream-adapter-postgres
cargo nextest run -p oya-messenger-message-stream-adapter-meilisearch
cargo nextest run -p oya-messenger-message-stream-adapter-redis-streams
cargo nextest run --test message_roundtrip_e2e
```

## Test Plan

- E2E: POST message → Postgres write → Meilisearch index → Valkey Streams (Redis wire-compat) xadd.
- Search filter: query with Cedar policy stub; verify out-of-scope channels not returned.
- Backpressure: Valkey Streams (Redis wire-compat) full → producer blocks ≤ 100ms then 503.

## Next IP

[`IP-007-presence-bc.md`](IP-007-presence-bc.md)
