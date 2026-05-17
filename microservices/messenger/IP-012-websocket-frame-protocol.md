---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-team-channels-dm-threads
impl_plan_id: IP-012-websocket-frame-protocol
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-messenger
acceptance_lanes: [cargo-nextest, ws-fuzz, asyncapi-conformance]
---

# IP-012: WebSocket frame protocol + AsyncAPI 3.1.0 conformance

## Intent

Define the WebSocket frame protocol per RFC 6455 + the AsyncAPI 3.1.0
descriptor at `contracts/asyncapi/messenger-events.yaml`. Frame types:
`ws-message`, `ws-presence`, `ws-reaction`, `ws-ping/pong`, `ws-ack`,
`ws-error`. Backpressure via bounded mpsc; idle ping every 30s.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-messenger-presence-adapter-websocket/src/frames.rs` | create |
| `src/crates/oya-messenger-presence-adapter-websocket/src/codec.rs` | create |
| `tests/ws_frame_fuzz.rs` | create — cargo-fuzz target |

## Code Shape

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum WsFrame {
    Message(MessageFrame),
    Presence(PresenceFrame),
    Reaction(ReactionFrame),
    Ping { ts: i64 },
    Pong { ts: i64 },
    Ack { seq: u64 },
    Error { code: String, message: String },
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-messenger-presence-adapter-websocket
cargo fuzz run ws_frame_fuzz -- -max_total_time=300
oya gate validate asyncapi-spec --microservice messenger
```

## Test Plan

- Fuzz: 5min cargo-fuzz run; no panic on malformed input.
- Slow reader: client stalls; gateway drops connection after 30s, frees slot.
- Ping/pong: gateway sends ping; missing pong → connection closed within 60s.

## Next IP

[`IP-013-search-and-cedar-filter.md`](IP-013-search-and-cedar-filter.md)
