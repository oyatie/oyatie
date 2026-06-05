---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-intelligence-two-layer-mvp
impl_plan_id: IP-016-streaming-sse-transport
status: pending
owner: axis-intelligence
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest]
related_adrs: [ADR-0255, ADR-0253]
---

# IP-016: Streaming transport — SSE

## Intent

Implement SSE (Server-Sent Events) streaming transport for the intelligence REST handler.
Callers receive token-by-token streaming via `text/event-stream`. Backpressure via bounded
channel. Audit-tap deferred to final chunk seal. UX-floor: high-fidelity streaming baseline
with no challenge interrupts.

## Concrete file targets

| Path | Action |
|---|---|
| `crates/oya-intelligence-model-routing-rest/src/sse_handler.rs` | create |
| `crates/oya-intelligence-model-routing-rest/src/sse_frame.rs` | create |
| `crates/oya-intelligence-model-routing-rest/src/stream_audit_tap.rs` | create |

## Code shape

```rust
pub async fn sse_dispatch_handler(
    State(deps): State<Arc<IntelligenceDeps>>,
    Json(req): Json<DispatchRequest>,
) -> impl IntoResponse {
    let stream = deps.dispatch_usecase.dispatch_stream(req).await?;
    Sse::new(stream.map(|chunk| {
        chunk.map(|c| Event::default().data(serde_json::to_string(&c)?))
    }))
    .keep_alive(KeepAlive::default())
}
```

## Key implementation notes

- Final chunk includes `audit_seal_id` — client can verify audit was committed.
- Keep-alive heartbeat every 15 s to avoid proxy timeouts.
- HTTP/3 QUIC transport avoids head-of-line blocking for concurrent streams.
- Graceful degradation: if HTTP/3 unavailable, fall to h2 multiplexed stream, then h1.1 chunked.
- UX-floor invariant: zero challenge on the SSE stream path for clean bot-score.

## Acceptance gates

```bash
cargo nextest run -p oya-intelligence-model-routing-rest -- sse
buck2 build //:quality-lane-registry-authority-check # lane=streaming-sse-high-fidelity --microservice intelligence
```

## References

- `microservices/intelligence/ARCHITECTURE.md §3.2` (network-opt-in dispatch).
- ADR-0253 (HTTP/3 + QUIC default).
- ADR-0255 amendment (library-first; SSE is network-opt-in path).

## Wave 15 substance conversion — SSE streaming transport

### §A Problem

The dispatch API advertises first-token and streaming-throughput SLOs, but provider adapters cannot expose raw
OpenAI, Anthropic, Google, or Bedrock delta formats to callers.
This IP closes the HTTP/SSE contract gap between provider chunks, audit tap, UX Layer-B, and tenant cancellation.

### §B Approach

Implement a canonical `DispatchChunk` SSE stream in the REST layer backed by the same in-process dispatch pipeline.
Provider deltas are normalized before they reach `brand-ux-surface`, and every terminal event is correlated with
the `CallRecord` emitted by IP-009/IP-022.

### §C Deliverables

- `crates/oya-intelligence-model-routing-rest/src/sse.rs`
- `crates/oya-intelligence-model-routing-rest/src/chunk_codec.rs`
- OpenAPI stream fixtures in `contracts/openapi/intelligence-v1.yaml`
- disconnect, retry, and ordered-completion tests

### §D Implementation

1. Define SSE events for `chunk`, `citation`, `refusal`, `usage`, `done`, and `error`.
2. Preserve chunk ordering and idempotency with monotonic chunk ids.
3. Convert direct-provider deltas into canonical chunks.
4. Abort provider calls when the client disconnects and emit an audit terminal state.
5. Tie first-token timings to `slos/first-token-latency.openslo.yaml`.
6. Keep raw prompts and completions out of high-cardinality metrics.

### §E Acceptance

Nextest must prove ordered chunks, graceful disconnect, and terminal audit emission; the governance gate must validate
the OpenAPI streaming schema.

### §F Evidence

Local anchors: `contracts/openapi/intelligence-v1.yaml`, `slos/first-token-latency.openslo.yaml`,
`slos/streaming-throughput.openslo.yaml`, `runbooks/model-inference-timeout-investigation.md`.

### §G Counterparts

OpenAI, Anthropic, Google, and AWS Bedrock all expose streaming; oyatie closes parity by normalizing them into one
Cedar/audit-aware SSE contract.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/intelligence/IP-016-streaming-sse-transport.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=300`, `rpo_p99_seconds=60` from manifest.json#rpo_rto.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), EU-AI-ACT-2024-HIGH-RISK(1800s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-PIPA-2023-amendment(14400s/900s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/intelligence/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`, `microservices/intelligence/slos/streaming-throughput.openslo.yaml`, `microservices/intelligence/policy/abuse-defence.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/intelligence/IP-016-streaming-sse-transport.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/intelligence/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
