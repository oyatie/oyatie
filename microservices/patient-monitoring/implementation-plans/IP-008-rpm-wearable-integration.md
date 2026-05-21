# IP-008 — RPM + wearable integration

**Status**: drafted
**Bounded contexts**: RPM + WearableIntegration
**Owner**: axis-clinical-realtime + axis-rpm-vertical
**Estimated effort**: 5-6 dev-weeks

## Slice 1: RPM enrollment service

- Patient enrollment + program selection + consent capture.
- Cedar `rpm-patient-can-view-own` + `rpm-caregiver-designated` integration.

## Slice 2: Apple HealthKit connector

- CMS-API client; webhook + polling modes.
- iOS app deep-link for consent + device-pairing.

## Slice 3: Fitbit / Garmin / Withings connectors

- Web API v1 / Garmin Health API / Withings Public Cloud API.

## Slice 4: Oura / Polar / Whoop / Samsung Health connectors

- Oura Cloud / Polar Accesslink / Whoop API / Samsung Health SDK.

## Slice 5: Dexcom + Abbott LibreView CGM connectors

- G7 Share API + LibreView Public API.

## Slice 6: Bluetooth GATT direct

- Heart Rate Service, Pulse Oximeter Service, Blood Pressure Service,
  Glucose Service.

## Slice 7: RPM patient portal (B2C)

- Patient self-view of own vitals + program adherence.

## Slice 8: RPM care-coordinator dashboard (B2B)

- Prioritized patient list by escalation score.

## Slice 9: Adherence scoring

- Expected-readings vs. observed; degraded-adherence event emit.

## Slice 10: Consent revocation flow

- Suspends ingestion immediately on consent-graph event.

## Acceptance criteria

- 12 wearable connectors functional.
- RPM ingest p99 ≤ 30 s.
- Consent revocation suspends within 60 s.

## Dependencies

- IP-001 + IP-003 + consent-graph µservice

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/patient-monitoring/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/patient-monitoring/implementation-plans/IP-008-rpm-wearable-integration.md:54` - - RPM ingest p99 ≤ 30 s..
