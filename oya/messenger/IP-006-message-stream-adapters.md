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

# IP-006: message-stream adapters (Postgres + Meilisearch + Valkey Streams (RESP3 wire-compatible))

## Intent

Implement `MessageStore` against Postgres; `MessageSearchIndex` against
Meilisearch (fallback Tantivy); `RealtimeBroadcaster` against Valkey Streams (RESP3 wire-compatible)
(fallback KeyDB). Per ADR-0105 Amendment 3 — three backend-qualified
`-adapter-<backend>` crates.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-messenger-message-stream-adapter-postgres/{src,migrations}/...` | create |
| `src/crates/oya-messenger-message-stream-adapter-meilisearch/src/...` | create |
| `src/crates/oya-messenger-message-stream-adapter-valkey-streams/src/...` | create |
| `tests/message_roundtrip_e2e.rs` | create — testcontainers Postgres + Meilisearch + Valkey |

## Code Shape

```rust
// adapter-valkey-streams/src/lib.rs
pub struct ValkeyStreamsBroadcaster {
    client: ValkeyClusterClient,
    stream_prefix: String,  // "msg:{pack}:{tenant}:{channel}"
}

#[async_trait]
impl RealtimeBroadcaster for ValkeyStreamsBroadcaster {
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
cargo nextest run -p oya-messenger-message-stream-adapter-valkey-streams
cargo nextest run --test message_roundtrip_e2e
```

## Test Plan

- E2E: POST message → Postgres write → Meilisearch index → Valkey Streams (RESP3 wire-compatible) xadd.
- Search filter: query with Cedar policy stub; verify out-of-scope channels not returned.
- Backpressure: Valkey Streams (RESP3 wire-compatible) full → producer blocks ≤ 100ms then 503.

## Next IP

[`IP-007-presence-bc.md`](IP-007-presence-bc.md)

## Wave 15 substance conversion — message stream adapters

### §A Problem

The message-stream domain is useless without durable write, searchable index, and realtime fanout adapters.
This IP closes the persistence/broadcast gap while keeping the domain invariants from IP-005 authoritative.

### §B Approach

Implement backend-qualified adapters for Postgres, Meilisearch/Tantivy, and Valkey Streams.
Each adapter consumes kernel ports and must preserve tenant/context partitioning plus Cedar search scope.

### §C Deliverables

- `src/crates/oya-messenger-message-stream-adapter-postgres/{src,migrations}/...`
- `src/crates/oya-messenger-message-stream-adapter-meilisearch/src/...`
- `src/crates/oya-messenger-message-stream-adapter-valkey-streams/src/...`
- `tests/message_roundtrip_e2e.rs`

### §D Implementation

1. Write messages to Postgres with tenant partition keys and RLS.
2. Index minimized documents into Meilisearch with context and channel ACL fields.
3. Publish canonical events to Valkey Streams for gateway fanout.
4. Reject or degrade on search index failure without losing canonical store writes.
5. Bound stream producer waits to 100ms and return 503 under sustained backpressure.
6. Prove search post-filter with Cedar in the roundtrip test.

### §E Acceptance

The e2e path must prove POST message to Postgres, index, Valkey xadd, scoped search, and backpressure behaviour.

### §F Evidence

Local anchors: `policy/tenant-scope.cedar`, `policy/channel-scope.cedar`, `slos/message-send-availability.openslo.yaml`,
`slos/search-latency.openslo.yaml`.

### §G Counterparts

Slack and Teams anchor durable enterprise message stores, Discord anchors fanout pressure, and Mattermost anchors
self-hosted storage; oyatie closes parity with explicit adapter contracts.
