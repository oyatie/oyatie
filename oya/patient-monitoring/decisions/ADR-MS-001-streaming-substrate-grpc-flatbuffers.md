# ADR-MS-001 — Streaming substrate: gRPC bidi with FlatBuffers payload

**Status**: Accepted
**Date**: 2026-05-21
**Microservice**: patient-monitoring
**Scope**: µservice-internal
**Author**: axis-clinical-realtime
**Binding to**: ADR-0253 (HTTP/3 + QUIC default), ADR-0248 (Amazon-shape cellular),
ADR-0332 (clinical-realtime substrate parent ADR), ADR-0244 (tenant scoping),
ADR-0252 (HLC default).

---

## 1. Context

The patient-monitoring µservice must stream high-frequency physiologic waveforms (50-1000 Hz
ECG, PLETH, ART, ICP, CVP, RESP, EtCO2, EEG, etc.) and 1-Hz numeric vitals from bedside
acquisition nodes to central-station displays, mobile-clinician devices, AsyncAPI subscribers,
and ML inference services. The clinician-safety contract demands:

- End-to-end latency p99 ≤ 250 ms for numeric vitals
- Waveform jitter p99 ≤ 8 ms (consecutive batches)
- Zero waveform loss for alarm-episode windows
- Lossless reconstruction available for any waveform overlapping an alarm-fire event

Existing options surveyed:

1. **Kafka-only fan-out**: durable, at-least-once, but adds 20-200 ms broker queueing
   variance at production durability settings; jitter floor too high.
2. **Pub/sub WebSockets**: lower jitter but no schema enforcement; PHI handling fragile.
3. **gRPC unary + polling**: pulls overhead beyond budget; not push.
4. **gRPC server-streaming (one-way)**: works for waveforms but no client-driven backpressure.
5. **gRPC bidi-streaming with Protobuf**: client-driven backpressure + push; Protobuf
   per-message allocation cost is significant for 500 Hz streams (per-sample heap allocations).
6. **gRPC bidi-streaming with FlatBuffers**: client-driven backpressure + push + zero-copy
   deserialization (contiguous sample arrays). This is the chosen option.
7. **Custom UDP + reliable layer**: rejected because TLS + HTTP/3 + multiplexing already exist
   in QUIC; reinventing them costs more than gRPC over QUIC.

The Philips IntelliVue + GE CARESCAPE + Mindray BeneVision incumbents use vendor-proprietary
binary protocols over TCP (or vendor-specific Ethernet). Their per-message overhead is
comparable to FlatBuffers; their network shape is comparable to gRPC over QUIC. Our choice is
isomorphic to theirs and equally low-latency, but uses open standards.

## 2. Decision

The patient-monitoring µservice uses **gRPC bidi streaming over HTTP/3 + QUIC with FlatBuffers
payload** for:

- `WaveformService/StreamWaveform` (bidi)
- `VitalSignsService/StreamVitalSigns` (bidi)
- `AlarmService/StreamAlarms` (server-streaming with client ack)
- `CentralStationService/SubscribeUnitView` (server-streaming)

For non-realtime fan-out (data-warehouse, audit-chain, analytics, ml-platform training data
collection), the µservice **emits AsyncAPI events** via the stream-platform µservice
(Kafka/Redpanda durable substrate).

**Protocol choices in detail:**

| Component | Choice | Rationale |
|---|---|---|
| Wire format | FlatBuffers (schema in `contracts/proto/`-equivalent .fbs files) | Zero-copy on both server and client |
| Transport | HTTP/3 over QUIC | < 1 ms head-of-line blocking recovery; per ADR-0253 |
| TLS | TLS 1.3 mTLS | Per ADR-0254 |
| Auth | OAuth2-bearer + Cedar evaluation at boundary | Per ADR-0243 |
| Time-of-record | HLC stamp at ingest, propagated through stream | Per ADR-0252 |
| Tenant binding | Every message carries `tenant_id`; Cedar scope-checked | Per ADR-0244 |
| Backpressure | Per-channel watermark + lossy-policy decision (drop oldest waveform batch; never drop alarm) | Patient-safety contract |

## 3. Consequences

### 3.1 Positive

- Sub-50 ms typical waveform end-to-end latency (5-ms parse + 5-ms canonicalize + 2-ms gRPC
  enqueue + 3-ms QUIC network + 30-ms client render).
- Zero per-sample heap allocation on both server and client (FlatBuffers contiguous arrays).
- Standard tooling: tonic + flatbuffers crates + protoc/flatc + cargo.
- QUIC's connection resumption keeps clinician-app reconnect < 1 s.
- Backpressure model is patient-safety-aligned: never drop alarms, only drop oldest waveform
  batches when client is overrun.

### 3.2 Negative

- FlatBuffers tooling is less mature than Protobuf in some ecosystems (Rust ecosystem is
  fine; cross-language consumers must use FlatBuffers code generator for their language).
- Schema evolution requires care: FlatBuffers offers forward/backward compat but field
  ordering matters; per ADR-MS-001 §5 we enforce schema-evolution review at every change.
- HTTP/3 + QUIC requires UDP transport; on-prem networks may need UDP-allow firewall rules
  for the in-room → Tier-3 → Tier-2 hops. Mitigation: TCP-fallback path documented in
  `runbooks/`.

### 3.3 Neutral

- Kafka remains in the µservice as the AsyncAPI substrate; we are not replacing it for
  durable fan-out.
- The local-cell ring buffer (4h per bed) provides durability backstop independent of
  Kafka; the µservice is resilient to Kafka outages on the hot path.

## 4. Implementation

### 4.1 Crate layout

```
crates/patient-monitoring-stream-substrate/
  src/lib.rs                     # core stream-server abstractions
  src/waveform_service.rs        # WaveformService gRPC impl
  src/vital_signs_service.rs     # VitalSignsService gRPC impl
  src/alarm_service.rs           # AlarmService gRPC impl
  src/central_station_service.rs # CentralStationService gRPC impl
  src/flatbuffers/               # generated FlatBuffers Rust code
  src/backpressure.rs            # per-channel watermark + lossy policy
  src/ring_buffer.rs             # local-cell ring buffer (mmap NVMe)
```

### 4.2 Build pipeline

- `build.rs` invokes `flatc` to generate Rust bindings from `.fbs` schemas.
- `tonic-build` generates gRPC service stubs from `contracts/proto/patient-monitoring.proto`.

### 4.3 Test posture

- Unit tests: FlatBuffers round-trip, backpressure policy, ring-buffer correctness.
- Integration tests: gRPC client/server end-to-end with simulated bedside device.
- Latency tests: histogram per-hop budget; CI lane `lane-patient-monitoring-streaming-perf`.
- Chaos tests: broker outage, cell-migration cutover, network hiccup; results in
  `evidence/chaos/`.

## 5. Schema evolution policy

- Every new field added to a FlatBuffers schema must be appended (never reordered).
- Removed fields are deprecated, not removed.
- Schema changes require sign-off from axis-clinical-realtime + axis-clinical-shared.
- Per-tenant schema overrides forbidden (the substrate is single-schema; pack overlays
  attach metadata fields only).

## 6. References

- ADR-0253 HTTP/3 + QUIC default
- ADR-0248 Amazon-shape cellular architecture
- ADR-0332 Clinical-realtime substrate
- ADR-0244 Tenant-as-universal-scoping-primitive
- ADR-0252 HLC default; TrueTime tier opt-in
- gRPC bidi streaming spec (https://grpc.io/docs/what-is-grpc/core-concepts/)
- FlatBuffers spec (https://google.github.io/flatbuffers/)
- HTTP/3 RFC 9114
- QUIC RFC 9000
- IEEE 11073-10101 / 11073-20601 (device frame formats)
- IHE PCD-WCM (waveform content module)
