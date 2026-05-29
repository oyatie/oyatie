# IP-010 — ICU bundle compliance

**Status**: drafted
**Bounded contexts**: ICUBundleCompliance
**Owner**: axis-clinical-realtime
**Estimated effort**: 2-3 dev-weeks

## Slice 1: Bundle observation writer

- Per-bed observations: status compliant / non-compliant / not-applicable / deferred.
- Postgres-16 schema + audit-chain emit.

## Slice 2: Compliance scorer

- Per-bed / per-unit / per-shift rolling score.

## Slice 3: Bundle-alert emitter

- Overdue-element alerts via alarm-management routing chain.

## Slice 4: Bundles supported (8)

- HOB (head-of-bed)
- DVT prophylaxis
- SAT/SBT (spontaneous awakening / breathing trial)
- Glucose control (140-180 mg/dL window)
- Sedation interruption
- CAUTI prevention (indwelling-catheter-day count + removal-reminder)
- CLABSI prevention (central-line-day count + insertion-bundle audit)
- Mobility (FSS-ICU scoring)

## Slice 5: Quality-measure export (CMS format)

- CMS file emit for VAP, CAUTI, CLABSI metrics.
- KR MFDS quality-measures format overlay.

## Slice 6: Per-pack overlay

- KR / EU / pediatric-specific bundle defaults.

## Acceptance criteria

- Bundle-observation write p99 ≤ 200 ms.
- Compliance-score recompute p99 ≤ 1 s.
- CMS quality-measure export passes CMS validator.

## Dependencies

- IP-003 alarm-management for bundle-alert routing
- audit-chain µservice
- quality-measures-reporting µservice for CMS export hook

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/patient-monitoring/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/patient-monitoring/implementation-plans/IP-010-icu-bundle-compliance.md:43` - - Bundle-observation write p99 ≤ 200 ms.; `microservices/patient-monitoring/implementation-plans/IP-010-icu-bundle-compliance.md:44` - - Compliance-score recompute p99 ≤ 1 s..
