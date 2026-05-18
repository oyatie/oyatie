---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-team-channels-dm-threads
impl_plan_id: IP-007-presence-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-messenger
acceptance_lanes: [cargo-nextest, ws-soak-test]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: presence BC (Redis + WebSocket gateway + LiveKit signaling)

## Intent

Implement `PresenceStore` against Redis Cluster; `WebSocketGateway` per
RFC 6455 + Envoy WebSocket upgrade; LiveKit signaling adapter for huddle
voice/video (later phase huddle BC). Per-tenant connection registry with
sharding by `tenant_id mod N`.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-messenger-presence-{kernel,domain,usecase,api,adapter-redis,adapter-websocket,adapter-livekit,worker,sdk,app}/...` | create |
| `tests/ws_soak.rs` | create — 100k connection soak |

## Code Shape

```rust
// adapter-websocket/src/gateway.rs
pub struct Tungstenite WebSocketGateway {
    registry: Arc<RwLock<HashMap<(TenantId, UserRef), Sender<Frame>>>>,
}

impl WebSocketGateway for TungsteniteGateway {
    async fn fanout(&self, target: FanoutTarget, frame: Frame) -> Result<()> {
        // RFC 6455 textual frames; backpressure via bounded sender.
        ...
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-messenger-presence-adapter-redis
cargo nextest run -p oya-messenger-presence-adapter-websocket
cargo nextest run --test ws_soak  # 100k connections; latency p99 ≤ 100ms
```

## Test Plan

- Soak: 100k concurrent WebSocket connections per gateway pod; p99 fanout ≤ 100ms.
- Reconnect storm: 10k clients simultaneously reconnect; jittered backoff respected.
- Presence transitions: Online→Away→DnD→Offline propagate ≤ 200ms p99.

## Next IP

[`IP-008-file-attachment-bc.md`](IP-008-file-attachment-bc.md)
