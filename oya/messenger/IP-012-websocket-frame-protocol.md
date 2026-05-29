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

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

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

## Wave 15 substance conversion — WebSocket frame protocol

### §A Problem

Messenger realtime quality depends on a stable wire protocol; raw JSON frames per feature would create drift and
make AsyncAPI conformance meaningless.
This IP closes the canonical frame protocol for messages, presence, reactions, ack/error handling, and heartbeat.

### §B Approach

Define `WsFrame` in the WebSocket adapter and bind it to `contracts/asyncapi/messenger-events.yaml`.
Frames are tenant-scoped and sequence-aware, with bounded channels and strict malformed-frame handling.

### §C Deliverables

- `src/crates/oya-messenger-presence-adapter-websocket/src/frames.rs`
- `src/crates/oya-messenger-presence-adapter-websocket/src/codec.rs`
- `tests/ws_frame_fuzz.rs`
- AsyncAPI conformance fixtures

### §D Implementation

1. Encode frame variants for message, presence, reaction, ping, pong, ack, and error.
2. Attach sequence numbers and tenant/context metadata to state-changing frames.
3. Reject oversized or unknown frame types before deserialization into domain objects.
4. Disconnect slow readers after bounded backpressure.
5. Emit ping every 30 seconds and close missing-pong sessions within 60 seconds.
6. Keep protocol errors auditable without logging message bodies.

### §E Acceptance

Fuzzing must run without panics, AsyncAPI validation must pass, and slow-reader tests must prove gateway slot cleanup.

### §F Evidence

Local anchors: `contracts/asyncapi/messenger-events.yaml`, `slos/websocket-fanout-latency.openslo.yaml`,
`runbooks/websocket-storm.md`.

### §G Counterparts

Slack realtime messaging, Discord gateway frames, and Matrix events anchor realtime protocol expectations; oyatie
closes parity with a contracted, tenant-scoped WebSocket protocol.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/messenger/contracts/openapi/messenger.yaml`, `microservices/messenger/contracts/asyncapi/messenger-events.yaml`, `microservices/messenger/contracts/proto/messenger.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.
