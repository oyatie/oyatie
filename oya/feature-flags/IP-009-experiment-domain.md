# IP-009 — Experiment Domain Crate

**microservice**: feature-flags
**bc**: experiment
**layer**: domain
**crate**: oya-feature-flags-experiment-domain
**status**: design-ready
**acceptance_status**: design-ready
**adrs**: ADR-0105, ADR-0131, ADR-0159, ADR-0218, ADR-0243, ADR-0244, ADR-0248, ADR-0263, ADR-0292, ADR-0307, ADR-0309
**companion_ips**: IP-008, IP-012, IP-020

## Scope

Experiment lifecycle management: create, activate, pause, conclude, re-attribute. Enforces Cedar step-up B for activation; prohibits EMERGENCY_SERVICES experiments without platform-safety-officer consent; prohibits MINOR_TARGETED without compliance pack.

## Deliverables

| # | Artifact | Acceptance Criterion |
|---|----------|---------------------|
| 1 | `ExperimentRepository` trait | CRUD + list with cursor pagination; tenant-scoped |
| 2 | `ExperimentLifecycleService` | State machine: draft → active → paused → concluded; `activate` requires `has_sample_size_estimate=true` per Cedar policy |
| 3 | `ExperimentConclusionService` | Calls `BayesianEngine` + `FrequentistEngine` + `MsprtEngine`; produces `ExperimentResult` with `winner`, `posterior_prob`, `feature_importance` (LIME/SHAP) |
| 4 | `ExperimentReAttributionService` | Replays `MetricEvent` stream for corrected attribution; DSAR-safe (user_id_hash only) |
| 5 | MINOR_TARGETED guard | Cedar FORBID if audience_type == MINOR_TARGETED and pack `oya-pack-eu-child-safety` not active |
| 6 | Audit events | `ExperimentCreated`, `ExperimentActivated`, `ExperimentPaused`, `ExperimentConcluded` |
| 7 | EU AI Act Art. 13 | `feature_importance` (LIME/SHAP) included in `ExperimentConcluded` event payload per ADR-0307 |
| 8 | Tests | MINOR_TARGETED guard rejects without compliance pack; re-attribution idempotent (twice → same result) |

## Definition of Done

- `cargo test -p oya-feature-flags-experiment-domain` green
- `activate` with `has_sample_size_estimate=false` returns `Cedar::Deny`
- EMERGENCY_SERVICES experiment without platform-safety-officer role returns `Cedar::Deny`
- `feature_importance` field present in all `ExperimentConcluded` events
