# ADR-MS-003 — Deterioration + sepsis ML stack: gradient-boosted ensemble with federated training

**Status**: Accepted
**Date**: 2026-05-21
**Microservice**: patient-monitoring
**Scope**: µservice-internal
**Author**: axis-clinical-realtime + axis-ml-platform
**Binding to**: ADR-0255 (BYOK + intelligence two-layer), ADR-0244 (tenant scoping),
ADR-0251 (compliance pack + cell certification), ADR-MS-002 (smart-alarm engine),
ADR-0332 (clinical-realtime substrate).

---

## 1. Context

The patient-monitoring µservice must predict clinical deterioration 4-12 hours ahead of
bedside calls (rapid-response activations, code-blue calls). It must also predict sepsis
≥ 6 hours before clinical recognition (Surviving Sepsis Campaign 1-hour bundle targets
benefit from earlier detection).

The model class must satisfy:

- **IEC 62304 SaMD Class C** lifecycle (life-supporting/sustaining software)
- **EU AI Act high-risk** lineage + transparency + bias reporting
- **KR MFDS Medical Device** approval pathway
- **Per-tenant fine-tuning** (academic medical centers have different mortality profiles
  than community hospitals)
- **Federated training across tenants** (no raw data leaves the tenant cell)
- **Inference latency p99 ≤ 200 ms** per bed per evaluation
- **Calibration**: predicted probabilities must be well-calibrated (Platt scaling)
- **Sensitivity ≥ 0.85 / specificity ≥ 0.70** on out-of-distribution test fold

Existing approaches surveyed:

1. **Deep neural network (LSTM/Transformer on raw vital-signs sequences)**: highest
   theoretical accuracy but inference latency challenging at the 5-min-per-bed cadence
   at 5,000-bed-per-cell scale (would require GPU at every cell); opacity is a regulatory
   risk. Rejected as primary.
2. **Rule-based only (NEWS2/MEWS/qSOFA/SOFA)**: interpretable, well-validated, low latency.
   Insufficient on its own — clinical literature shows ML augmentation improves AUROC
   by 0.05-0.12.
3. **Gradient-boosted ensemble (LightGBM / XGBoost class)**: strong tabular performance,
   inference latency excellent, interpretable via SHAP, supports per-tenant fine-tuning,
   amenable to federated training via secure aggregation of leaf statistics. Chosen.
4. **Rothman Index commercial license**: proprietary; expensive; no per-tenant fine-tuning;
   not consistent with substrate doctrine. Rejected (we ship an oyatie-trained analog).
5. **Epic Deterioration Index commercial license**: bound to Epic EMR; not portable;
   rejected (we ship an oyatie-trained analog).

## 2. Decision

The patient-monitoring µservice uses a **gradient-boosted ensemble** (LightGBM-class,
implemented as `light-gbm-rs` Rust port per strict-Rust policy) for:

- **Rothman-Index-analog deterioration score**
- **Epic-Deterioration-Index-analog deterioration score** (alternative primary, configurable
  per tenant)
- **Sepsis-watch score** (augmenting Sepsis-3 rule path)

**Architecture**:

- Inference runs in `crates/patient-monitoring-ml-inference-app/` and consumes models from
  the `ml-platform` µservice's model registry.
- Per-tenant fine-tuning models live at `tenant/<tenant_id>/models/<model_class>/v<n>`.
- Federated training across consenting tenants via secure aggregation (per the
  `ml-platform` µservice's federated-training substrate).
- Calibration via Platt scaling at the µservice boundary (re-calibrated quarterly per
  tenant).
- Inference lineage emitted per prediction: `{input_snapshot_hash, feature_vector_hash,
  model_version, model_card_hash, raw_score, calibrated_score, rule_augmentation_outcome,
  decision, lineage_id}` → audit-chain.

**Rule augmentation contract** (per ADR-MS-002):

- The rule-based scores (NEWS2/MEWS/PEWS/qSOFA/SOFA/APACHE-IV/SAPS-3) are computed in
  parallel and emitted as auxiliary fields in the `DeteriorationScore`.
- The ML score is the primary alert-driver; the rule scores are the safety floor.
- If the ML inference is unavailable, the system **falls back to rule-based scoring**
  (NEWS2 for general acute; MEWS for ED; PEWS for PICU; qSOFA + SOFA for sepsis).
- The ML score **cannot suppress** a rule-engine `critical` or `life-threatening` output.

## 3. Model lifecycle (IEC 62304 SaMD Class C)

Per IEC 62304:2015 + IEC 82304:2016 + ISO 14971:2019, the deterioration + sepsis models are
classified **SaMD Class C** (life-supporting; failure may cause serious injury or death).
Lifecycle activities:

| Phase | Activity | Evidence |
|---|---|---|
| Plan | Model lifecycle plan, traceability matrix | `evidence/samd/lifecycle-plan.md` |
| Requirements | Performance + interpretability + bias requirements | `evidence/samd/requirements.md` |
| Architecture | This ADR | This file |
| Risk | Risk file (per ISO 14971) | `evidence/samd/risk-file.md` |
| Verification | Out-of-distribution test fold AUROC / AUPRC / calibration | `evidence/samd/verification.md` |
| Validation | Prospective IRB study at lead site | `evidence/samd/validation.md` |
| Change control | Algorithm change-control plan (per FDA AI/ML SaMD guidance) | `evidence/samd/change-control.md` |
| Post-market | Production monitoring + drift detection | `evidence/samd/post-market.md` |

## 4. Risk file (per ISO 14971)

Top hazards:

| # | Hazard | Severity | Likelihood | Mitigation | Residual risk |
|---|---|---|---|---|---|
| H01 | Model under-predicts deterioration (false negative) | catastrophic | low | Rule-based safety floor (NEWS2/MEWS/qSOFA always computed in parallel); inference-unavailable → rule fallback | acceptable |
| H02 | Model over-predicts (false positive) → alarm fatigue | moderate | medium | Smart-alarm engine dedup; per-tenant calibration; threshold tuning per pack | acceptable |
| H03 | Model bias by demographic subgroup | high | medium | Subgroup performance reporting (model card); federated training samples diverse cohorts | acceptable |
| H04 | Model drift over time | high | medium | Quarterly recalibration; production drift detection alerts | acceptable |
| H05 | Adversarial input (corrupted device telemetry) | high | low | Validity-check upstream (smart-alarm validity primitive); inference receives only validated features | acceptable |
| H06 | Inference latency spike | moderate | low | Rule-based fallback at p99 budget breach | acceptable |
| H07 | Model registry compromise | catastrophic | very-low | Per-tenant KMS; signed model artifacts; model-card hash verified at load | acceptable |
| H08 | Federated training data leakage | catastrophic | low | Secure aggregation; differential-privacy noise; no raw data leaves tenant cell | acceptable |

## 5. Risk management plan (per ISO 14971)

- Risk assessment renewed annually + on any major model change.
- Hazard analysis renewed on every algorithm change-control event.
- Post-market surveillance via `quality-measures-reporting` µservice publishes sensitivity
  + specificity + AUROC + AUPRC + calibration drift per quarter per tenant.

## 6. EU AI Act high-risk compliance

Per EU AI Act 2024 Annex III, healthcare ML deterioration prediction is classified
**high-risk**. Required documentation:

- **Risk management system**: ISO 14971 risk file (see §4).
- **Data governance**: training-data provenance + de-identification chain; model card
  documents subgroup performance.
- **Technical documentation**: this ADR + ADR-MS-002 + the SaMD lifecycle artifacts.
- **Record-keeping**: per-inference lineage logged 10Y to audit-chain.
- **Transparency**: model card published; clinical-decision-support µservice exposes
  prediction rationale.
- **Human oversight**: every prediction is advisory; clinician must confirm any
  protocol-triggered action (e.g., code-sepsis activation requires two-factor confirmation).
- **Accuracy + robustness + cybersecurity**: out-of-distribution test fold; adversarial
  robustness testing; signed model artifacts.

## 7. Federated training (across consenting tenants)

Per ADR-0255 §D-4 (BYOK + intelligence two-layer) and ADR-0244 (tenant scoping), federated
training:

- Each consenting tenant computes local LightGBM leaf statistics on de-identified
  in-cell data.
- Statistics are aggregated via secure aggregation (additively secret-shared) at the
  `ml-platform` µservice's federated-training substrate.
- Aggregated leaf statistics are used to build the global ensemble.
- No raw data leaves the tenant cell.
- Tenant participation opt-in is recorded via `policies/federated-training-opt-in.cedar`
  (tenant-administrator authority required).
- Federated-training rounds are gated by a quorum (default ≥ 5 tenants, ≥ 100K bed-days
  contributed).
- Differential-privacy noise added at the aggregation step (ε ≤ 2.0 default; configurable
  per pack).

## 8. Consequences

### 8.1 Positive

- Strong tabular ML performance (AUROC targets achievable per literature).
- Low inference latency p99 ≤ 200 ms.
- Interpretable via SHAP at the per-prediction level (exposed in clinical-decision-support
  µservice).
- Per-tenant fine-tuning + federated training improves performance for under-represented
  populations.
- IEC 62304 SaMD Class C and EU AI Act high-risk evidence packets producible.

### 8.2 Negative

- LightGBM-rs is a Rust port; ecosystem is younger than Python's LightGBM. Mitigation: we
  test against the canonical Python LightGBM on identical data and pin parity to ≤ 0.5%
  AUROC delta.
- Federated training adds complexity; secure aggregation requires careful key management
  (per cloud-kms µservice).
- Quarterly recalibration adds ops burden. Mitigation: automated re-calibration pipeline.

### 8.3 Neutral

- Rule-based scores (NEWS2/MEWS/etc.) remain the safety floor; ML augments but cannot
  replace.

## 9. Implementation

### 9.1 Crate layout

```
crates/patient-monitoring-ml-inference-app/
  src/main.rs                     # inference service
  src/feature_builder.rs          # vital + lab feature rollup
  src/lightgbm_forward.rs         # LightGBM forward pass (light-gbm-rs)
  src/calibration.rs              # Platt scaling
  src/rule_scorer.rs              # NEWS2/MEWS/PEWS/qSOFA/SOFA/APACHE-IV/SAPS-3
  src/augmentation.rs             # rule-augmentation logic (cannot-suppress invariant)
  src/lineage.rs                  # inference lineage emit
  src/fallback.rs                 # ML-unavailable → rule-fallback path
  src/drift.rs                    # production drift detection

crates/patient-monitoring-ml-training-client/
  src/main.rs                     # federated-training client
  src/secure_aggregation_client.rs # secure aggregation w/ ml-platform substrate
  src/leaf_statistics.rs          # LightGBM leaf-stat computation
```

### 9.2 Test posture

- Unit tests: feature builder, LightGBM forward, calibration, rule scorers, augmentation
  invariants, lineage emit.
- Integration tests: end-to-end inference against MIMIC-IV + eICU-CRD held-out fold;
  AUROC / AUPRC / calibration metrics published per release.
- Federated-training tests: simulated 5-tenant federation; secure aggregation correctness;
  differential-privacy noise calibration.
- Drift-detection tests: synthetic drift injection (covariate shift); detector
  sensitivity + specificity.
- Adversarial tests: poisoned input via corrupted device telemetry; verify validity-check
  upstream catches.

## 10. References

- ADR-0255 BYOK + intelligence two-layer
- ADR-0244 Tenant scoping
- ADR-0251 Compliance pack + cell certification
- ADR-MS-002 Smart-alarm engine
- ADR-0332 Clinical-realtime substrate
- IEC 62304:2015 (medical device software lifecycle)
- IEC 82304-1:2016 (health software)
- ISO 14971:2019 (risk management for medical devices)
- FDA AI/ML-based SaMD action plan + change-control guidance
- EU AI Act 2024 Annex III (high-risk classification)
- KR MFDS Medical Device 2024
- MIMIC-IV / eICU-CRD (training data)
- Surviving Sepsis Campaign 1-hour bundle (clinical reference)
- LightGBM: Ke et al., NeurIPS 2017
- SHAP: Lundberg + Lee, NIPS 2017
- Secure aggregation: Bonawitz et al., ACM CCS 2017
