---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-intelligence-two-layer-mvp
impl_plan_id: IP-017-streaming-websocket-transport
status: pending
owner: axis-intelligence
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest]
related_adrs: [ADR-0255, ADR-0253]
---

# IP-017: Streaming transport — WebSocket

## Intent

Implement WebSocket streaming transport for the intelligence dispatch surface. Required for
bidirectional audio streaming (OpenAI Realtime API / Google Live API), interactive assistant
sessions, and low-latency consumer UX. Runs over HTTP/3 WebTransport when peer supports it;
falls back to HTTP/1.1 WebSocket upgrade.

## Concrete file targets

| Path | Action |
|---|---|
| `crates/oya-intelligence-model-routing-rest/src/ws_handler.rs` | create |
| `crates/oya-intelligence-model-routing-rest/src/ws_frame.rs` | create |
| `crates/oya-intelligence-model-routing-rest/src/ws_session.rs` | create |

## Code shape

```rust
pub async fn ws_dispatch_handler(
    ws: WebSocketUpgrade,
    State(deps): State<Arc<IntelligenceDeps>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| ws_session(socket, deps))
}

async fn ws_session(socket: WebSocket, deps: Arc<IntelligenceDeps>) {
    // 1. Receive initial DispatchRequest frame
    // 2. Cedar authorization check
    // 3. dispatch_stream() → forward chunks as binary frames
    // 4. Receive follow-up user turns (interactive session)
    // 5. On close frame: commit audit-tap + seal
}
```

## Key implementation notes

- Session bound to tenant_id + audience_tag at handshake; cannot change mid-session.
- Each message frame carries `tenant_id` + `request_id` for audit correlation.
- Idle timeout: 5 min (configurable per tenant tier).
- Audio frames: PCM16 / Opus, chunked; for Apple Foundation Models on-device, WebSocket to local SDK.

## Acceptance gates

```bash
cargo nextest run -p oya-intelligence-model-routing-rest -- ws
cargo run -p oya-dev-cli -- gate validate streaming-websocket-session-audit --microservice intelligence
```

## References

- `microservices/intelligence/ARCHITECTURE.md §3.2`.
- IP-016 (SSE transport — share stream_audit_tap).
- ADR-0253 (HTTP/3 + QUIC; WebTransport).

## Wave 15 substance conversion — WebSocket streaming transport

### §A Problem

SSE is sufficient for many answer streams, but realtime audio, bidirectional tool state, and consumer UX reconnect
need a WebSocket/WebTransport-compatible contract.
This IP closes the bidirectional stream gap without letting UI code talk to providers directly.

### §B Approach

Define canonical WebSocket frames for dispatch input, stream chunks, client abort, heartbeat, and terminal status.
The server still invokes the library-first dispatch pipeline and provider adapters; WebSocket is only transport.

### §C Deliverables

- `crates/oya-intelligence-model-routing-rest/src/ws.rs`
- shared `chunk_codec.rs` with IP-016
- AsyncAPI entries in `contracts/asyncapi/intelligence-events-v1.yaml`
- fuzz/slow-reader tests for malformed and stalled frames

### §D Implementation

1. Authenticate and authorize the WebSocket upgrade with the same tenant/audience checks as REST.
2. Accept only bounded dispatch frames with explicit `envelope_id`.
3. Emit heartbeat and backpressure signals without leaking prompt content.
4. Normalize provider deltas through the same `DispatchChunk` codec as SSE.
5. Process abort frames by cancelling provider calls and writing audit terminal state.
6. Close slow readers before they can exhaust gateway memory.

### §E Acceptance

Nextest plus WebSocket fuzzing must prove malformed frames cannot panic, backpressure closes slow readers, and audit
terminal state exists for aborts.

### §F Evidence

Local anchors: `contracts/asyncapi/intelligence-events-v1.yaml`, `slos/streaming-throughput.openslo.yaml`,
`runbooks/model-router-stall-investigation.md`, and ADR-0253 transport profile in `manifest.json`.

### §G Counterparts

OpenAI Realtime and Google Live provide realtime streams; oyatie closes the equivalent capability with tenant-scoped
WebSocket frames, Cedar admission, and audit-first cancellation.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.
