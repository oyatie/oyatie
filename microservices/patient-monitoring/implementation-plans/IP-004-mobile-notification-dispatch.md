# IP-004 — Mobile notification dispatch

**Status**: drafted
**Bounded contexts**: MobileNotification
**Owner**: axis-clinical-realtime
**Estimated effort**: 2-3 dev-weeks

## Slice 1: Clinician device registry

- Postgres-16: `clinician_device_registration` with token, channel, opt-in.
- iOS / Android / WebPush registration flows.

## Slice 2: APNs dispatcher

- Rust + `a2` crate.
- TLS cert rotation via cloud-kms.

## Slice 3: FCM dispatcher

- Rust + `fcm-rust` crate.
- OAuth2 service-account key via cloud-kms.

## Slice 4: WebPush dispatcher

- Rust + `web-push` crate.
- VAPID key rotation via cloud-kms.

## Slice 5: SMS + pager gateway

- Twilio + AWS SNS fallback for SMS.
- Spok + + Vocera pager-gateway SDK wrappers.

## Slice 6: Routing + dispatch tracking

- Per-alarm routing per AlarmRoutingPolicy.
- Dispatch log: t_dispatched, t_delivered, t_acked.

## Acceptance criteria

- Alarm-fire → mobile-notification p99 ≤ 3 s (per SLO).
- iOS + Android + WebPush + SMS + Pager channels exercised in CI.
- Dispatch log per HIPAA + 21 CFR Part 11.

## Dependencies

- IP-003 smart-alarm engine
- cloud-kms µservice
- audit-chain µservice

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/patient-monitoring/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/patient-monitoring/implementation-plans/IP-004-mobile-notification-dispatch.md:40` - - Alarm-fire → mobile-notification p99 ≤ 3 s (per SLO)..
