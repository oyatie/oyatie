# IP-005 — Central-station render

**Status**: drafted
**Bounded contexts**: CentralStation
**Owner**: axis-clinical-realtime + axis-frontend
**Estimated effort**: 4-5 dev-weeks

## Slice 1: Unit-view backend gRPC server

- `CentralStationService.SubscribeUnitView` server-streaming.
- 4/8/16/32 bed grid configurable.
- Per-bed snapshot: latest vitals + most-critical-alarm + deterioration score + coverage.

## Slice 2: Linux SDL2 kiosk frontend

- Rust + SDL2 + wgpu shader pipeline.
- 4K render at ≥ 25 Hz.

## Slice 3: Windows WinUI 3 frontend

- C#/.NET WinUI 3 + Rust FFI gRPC client.
- Per global memory: Windows frontend authorized via WinUI 3.

## Slice 4: macOS Apple Silicon kiosk

- SwiftUI + Metal compositor + Rust gRPC FFI client.
- Apple Silicon M5+ only per global memory.

## Slice 5: iOS / Android clinician app

- SwiftUI + Jetpack Compose front-ends.
- Rust gRPC client via FFI.

## Slice 6: Drill-down bed-detail view

- Full waveform + full trend + alarm history.

## Acceptance criteria

- 8-bed render p99 ≤ 400 ms (per SLO).
- Live waveform refresh ≥ 25 Hz.
- Drill-down p99 ≤ 600 ms.

## Dependencies

- IP-001 streaming substrate
- IP-003 smart-alarm engine (for alarm overlay)
- IP-007 deterioration prediction (for score overlay)

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/patient-monitoring/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/patient-monitoring/implementation-plans/IP-005-central-station-render.md:40` - - 8-bed render p99 ≤ 400 ms (per SLO).; `microservices/patient-monitoring/implementation-plans/IP-005-central-station-render.md:42` - - Drill-down p99 ≤ 600 ms..
