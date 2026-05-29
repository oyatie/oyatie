# IP-006 — ML inference: deterioration + sepsis

**Status**: drafted
**ADR binding**: ADR-MS-003
**Bounded contexts**: DeteriorationPrediction + SepsisEarlyWarning
**Owner**: axis-clinical-realtime + axis-ml-platform
**Estimated effort**: 6-7 dev-weeks

## Slice 1: Feature builder

- Roll-up of vital + lab + waveform features over configurable windows.
- Per-parameter normalization + missingness imputation.

## Slice 2: Rule-based scorers

- NEWS2 / MEWS / PEWS / qSOFA / SOFA / APACHE-IV / SAPS-3.

## Slice 3: LightGBM-rs forward pass

- Rothman-Index-analog + Epic-DI-analog ensembles.
- Per-tenant fine-tuning models.

## Slice 4: Platt-scaling calibration

- Per-tenant quarterly recalibration pipeline.

## Slice 5: Rule augmentation (ML cannot suppress)

- Boundary check: ML may not lower priority of `critical` or `life-threatening`
  rule-engine output.

## Slice 6: Inference lineage emit

- Per-prediction lineage_id with input_snapshot_hash + feature_vector_hash +
  model_version + model_card_hash + raw_score + calibrated_score → audit-chain.

## Slice 7: Fallback path

- ML-unavailable → rule-based fallback within budget.

## Slice 8: Drift detection

- Per-feature distribution drift; per-tenant alert on drift.

## Slice 9: Model card

- `models/deterioration/MODEL-CARD.md` and `models/sepsis/MODEL-CARD.md`.
- Subgroup performance + intended/contraindicated use + monitoring metrics.

## Slice 10: SaMD evidence packets

- IEC 62304 lifecycle artifacts under `evidence/samd/`.

## Acceptance criteria

- Inference p99 ≤ 200 ms per bed (per SLO).
- AUROC ≥ 0.82 deterioration / ≥ 0.85 sepsis on out-of-distribution test fold.
- Calibration: ECE ≤ 0.05.
- Model card + risk file complete.
- EU AI Act lineage retention 10Y verified.

## Dependencies

- ADR-MS-003 accepted ✅
- ml-platform µservice ready
- audit-chain µservice ready
- emr + lab + healthcare-integration µservices for feature ingest

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/patient-monitoring/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `1800s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `EU-AI-ACT-2024-HIGH-RISK` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=1800`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `object_storage_versioned`, `milvus_snapshot`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/patient-monitoring/implementation-plans/IP-006-ml-inference-deterioration-sepsis.md:56` - - Inference p99 ≤ 200 ms per bed (per SLO)..
