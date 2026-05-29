# IP-009 — Waveform archive + tiered storage

**Status**: drafted
**Bounded contexts**: WaveformArchive
**Owner**: axis-clinical-realtime
**Estimated effort**: 3-4 dev-weeks

## Slice 1: Hot tier writer

- Per-tenant prefix; ZSTD-compressed FlatBuffers batches; 7-day local retention.

## Slice 2: Warm tier lifecycle policy

- 7-30 day retention; same store, lifecycle-policy-bound.

## Slice 3: Cold tier ETL

- Migration to ClickHouse columnar; per-channel decimation 1:4 with lossless
  reconstruction marker for alarm-episode subset.

## Slice 4: Alarm-episode pinning

- Lifecycle override to 7Y retention for ± 30 s of alarm-fire.

## Slice 5: Retrieval API

- `WaveformService.RetrieveWaveform` with tier hint (hot/warm/cold).
- p99 ≤ 2 s hot; ≤ 10 s warm; ≤ 90 s cold.

## Slice 6: Cohort de-identified extract

- Research IRB extracts; consent-graph aggregate consent gate.

## Slice 7: Playback replay

- Server-streaming reconstruction at configurable rate.

## Acceptance criteria

- Hot retrieval p99 ≤ 2 s.
- Lossless reconstruction for alarm-episode windows verified.
- Cohort extract de-identification audited.

## Dependencies

- IP-001 streaming substrate
- ClickHouse cluster ready
- Object storage hot/warm tier

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/patient-monitoring/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/patient-monitoring/implementation-plans/IP-009-waveform-archive-tiered-storage.md:28` - - p99 ≤ 2 s hot; ≤ 10 s warm; ≤ 90 s cold.; `microservices/patient-monitoring/implementation-plans/IP-009-waveform-archive-tiered-storage.md:40` - - Hot retrieval p99 ≤ 2 s..
