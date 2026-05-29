# IP-013 — AI marketplace vendor-neutral adapter layer

`scope: oya-imaging-ai-marketplace-app + oya-imaging-ai-dispatch-worker + oya-imaging-deidentification-worker + per-vendor adapter crates`
`wave_target: 19-imaging-ai`
`adr_binding: ADR-MS-002 + ADR-0251 + `policies/ai-model-can-read-deidentified.cedar``

## Objective

Stand up the vendor-neutral CADe/CADx fan-out marketplace covering ≥15 AI vendors (Aidoc, Viz.ai, Cleerly, Rad AI, Annalise.ai, Lunit, Qure.ai, Zebra Medical, Arterys, Caption Health, RapidAI, Subtle Medical, Imagia, Behold.ai, ScreenPoint). PHI de-identification per HIPAA Safe Harbor + ISO/TS 25237 BEFORE vendor egress. FDA / CE clearance metadata enforced via Cedar.

## Scope

1. `AiVendorPort` trait + per-vendor adapter crates.
2. `oya-imaging-deidentification-worker` per HIPAA Safe Harbor 18-identifier removal + ISO/TS 25237 pseudonymization.
3. Cedar gate via `policies/ai-model-can-read-deidentified.cedar`.
4. Drift detection (per FR-AI-005): per-vendor PPV / sensitivity / specificity week-over-week.
5. FDA / CE / KFDA / PMDA / ANVISA clearance metadata stored per vendor model version.
6. Off-label inference Cedar-denied.
7. Stroke LVO NPV ≥98% validation cohort.
8. Mammography CAD on synthesized 2D + DBT slices.
9. Dispatch p95 < 500ms (FR-AI-002).

## Acceptance criteria

- ≥15 vendors integrated at GA.
- Dispatch p95 < 500ms.
- De-identification test: HIPAA Safe Harbor 18 identifiers stripped + ISO/TS 25237 pseudonym map preserved.
- Off-label Cedar deny test passes.
- Drift detector raises alert at >10% week-over-week PPV drop.
- Stroke LVO validation cohort NPV ≥98%.

## Dependencies

- IP-001, IP-005, IP-009.
- `cloud-iam` Cedar policy.

## Risks

- Vendor-API drift; mitigate with per-vendor regression set.
- De-identification correctness on private tags; mitigate with strict allow-list.

## Estimated effort

- 16–24 person-weeks (15 vendors × ~1 person-week + de-identification + drift detection).

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/imaging/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/imaging/implementation-plans/IP-013-ai-marketplace.md:9` - Stand up the vendor-neutral CADe/CADx fan-out marketplace covering ≥15 AI vendors (Aidoc, Viz.ai, Cleerly, Rad AI, Annalise.ai, Lunit, Qure.ai, Zebra Medical, Arterys,....
