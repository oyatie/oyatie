# IP-007 — Code-blue activation + playback

**Status**: drafted
**Bounded contexts**: CodeBlue + WaveformArchive
**Owner**: axis-clinical-realtime
**Estimated effort**: 2-3 dev-weeks

## Slice 1: Activation FSM

- Manual / auto-non-ack / suggested-sepsis / suggested-deterioration paths.
- Cedar fail-OPEN per ADR-0332.

## Slice 2: Waveform pin (± 30 min)

- Object-storage lifecycle override to 7Y for the bed's waveform window.
- Lossless reconstruction marker preserved.

## Slice 3: Pager dispatch (highest priority)

- Code-blue team pager via highest-priority channel.

## Slice 4: Central-station highlight

- Red overlay + audible alarm on unit-view.

## Slice 5: Playback service

- `WaveformService.StartReplay` server-streaming.
- Configurable playback rate (1x / 2x / 4x / 8x).

## Slice 6: Post-event debrief export

- Bundle: activation event + all waveforms + all vitals + all alarms in window.
- CMS quality-measures export hook.

## Acceptance criteria

- Activation → team-pager p99 ≤ 2 s (per SLO).
- Waveform-pin completes within 5 s.
- Playback replay matches original to bit-level for alarm-episode windows.

## Dependencies

- IP-001 + IP-003 + IP-004 ready

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/patient-monitoring/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/patient-monitoring/implementation-plans/IP-007-code-blue-activation-playback.md:38` - - Activation → team-pager p99 ≤ 2 s (per SLO)..
