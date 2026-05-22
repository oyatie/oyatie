# IP-001 — Streaming substrate (gRPC + FlatBuffers)

**Status**: drafted
**ADR binding**: ADR-MS-001
**Bounded contexts**: VitalSignsStream + Waveform
**Owner**: axis-clinical-realtime
**Estimated effort**: 4-5 dev-weeks

## Slice 1: FlatBuffers schema authoring + Rust code-gen

- Author `contracts/proto/vital-signs.fbs` and `waveform.fbs` covering sample, batch, envelope.
- Wire `build.rs` to invoke `flatc` and emit `src/flatbuffers/` generated Rust.
- Round-trip unit tests.

## Slice 2: gRPC service skeletons via tonic

- Implement `VitalSignsService` and `WaveformService` stubs.
- Bidi streaming with hello-world envelope round-trip.

## Slice 3: Envelope canonicalization + HLC stamping

- HLC clock per ADR-0252; per-channel monotone ordering.
- Tenant + cell stamping per ADR-0244.

## Slice 4: Backpressure + local-cell ring buffer

- Per-channel watermark; lossy policy (drop oldest waveform batch; never drop alarm).
- mmap NVMe ring buffer: 4h per bed; per-bed-session slot allocation.

## Slice 5: AsyncAPI fan-out wiring

- Emit `vital.streamed` and `waveform.streamed` to stream-platform µservice.
- CloudEvents envelope per `contracts/asyncapi.yaml`.

## Slice 6: Integration test against simulated bedside

- HL7v2 simulator emits 1-Hz vitals + 500-Hz ECG.
- E2E latency histogram; CI lane `lane-patient-monitoring-streaming-perf`.
- Target: vital p99 ≤ 250 ms, waveform jitter p99 ≤ 8 ms.

## Acceptance criteria

- All slices land with green CI on Tier-1 OSes (Talos, RHEL-9, Oracle-Linux-9, SUSE-15-SP6).
- Latency histogram matches SLO targets in `slos/`.
- Chaos test: Kafka outage; ring buffer + gRPC continue serving for ≥ 1 hour.

## Dependencies

- ADR-MS-001 accepted ✅
- tonic + flatbuffers + zstd-rs + object_store crates available in workspace
- stream-platform µservice ready

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/patient-monitoring/implementation-plans/IP-001-streaming-substrate-grpc-flatbuffers.md:33` - - CloudEvents envelope per `contracts/asyncapi.yaml`..
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/patient-monitoring/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `valkey`, `postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/patient-monitoring/implementation-plans/IP-001-streaming-substrate-grpc-flatbuffers.md:39` - - Target: vital p99 ≤ 250 ms, waveform jitter p99 ≤ 8 ms.; `microservices/patient-monitoring/implementation-plans/IP-001-streaming-substrate-grpc-flatbuffers.md:44` - - Latency histogram matches SLO targets in `slos/`..
