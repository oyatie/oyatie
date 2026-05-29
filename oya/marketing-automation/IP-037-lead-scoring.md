---
doc_class: ImplementationPlan
ip_id: IP-037-lead-scoring
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0263, ADR-0321, ADR-0328]
bounded_context: lead-scoring
journey_id: J-MA-37-inbound-lead-scoring
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-037: Lead Scoring

## Context

Marcus Chen and Pat Lee need lead scoring that combines demographic (account.arr, account.industry, contact.title), behavioral (page-views, email-clicks, form-submits), and predictive (intelligence-driven likelihood-to-convert) components — with score decay so historical engagement does not dominate current intent. HubSpot Lead Scoring and Marketo Lead Scoring are universal; Mailchimp Premium Predicted Demographics is a narrower primitive.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_lead_scoring_model` | `model_id` | `uuid primary key` | Model version id. |
| `marketing_lead_scoring_model` | `tenant_id` | `uuid not null` | Tenant partition. |
| `marketing_lead_scoring_model` | `version` | `int not null` | Monotonic per tenant. |
| `marketing_lead_scoring_model` | `demographic_formula` | `jsonb not null` | Field-weight pairs e.g. `{"account.arr": {">100k": 10, ">500k": 30}}`. |
| `marketing_lead_scoring_model` | `behavioral_formula` | `jsonb not null` | Event-weight pairs e.g. `{"page_view_pricing": 5, "form_submit_demo": 20}`. |
| `marketing_lead_scoring_model` | `predictive_component_enabled` | `boolean not null default false` | Calls intelligence µservice. |
| `marketing_lead_scoring_model` | `decay_half_life_days` | `int not null default 30` | Behavioral score decay. |
| `marketing_lead_score` | `score_id` | `uuid primary key` | Per-subject score row. |
| `marketing_lead_score` | `tenant_id` | `uuid not null` | Tenant partition. |
| `marketing_lead_score` | `subject_hash` | `text not null` | Subject ref. |
| `marketing_lead_score` | `model_id` | `uuid not null` | FK to model. |
| `marketing_lead_score` | `demographic_score` | `int not null default 0` | Demographic component. |
| `marketing_lead_score` | `behavioral_score` | `int not null default 0` | Behavioral component (decayed). |
| `marketing_lead_score` | `predictive_score` | `int` | Optional predictive component. |
| `marketing_lead_score` | `total_score` | `int not null` | Sum. |
| `marketing_lead_score` | `last_scored_hlc` | `hlc not null` | Last computation timestamp. |
| `marketing_lead_score_adjustment` | `adjustment_id` | `uuid primary key` | Manual adjustment audit row. |
| `marketing_lead_score_adjustment` | `score_id` | `uuid not null` | FK. |
| `marketing_lead_score_adjustment` | `delta` | `int not null` | Adjustment delta. |
| `marketing_lead_score_adjustment` | `reason` | `text not null` | Adjustment reason. |
| `marketing_lead_score_adjustment` | `principal_id` | `uuid not null` | Principal who applied. |

## API Endpoints

REST `POST /v1/marketing-automation/lead-scoring/models` defines a model.

REST `POST /v1/marketing-automation/lead-scoring/score/{subject_hash}:recalculate` recomputes a subject's score.

REST `POST /v1/marketing-automation/lead-scoring/score/{score_id}:manual-adjust`:

```json
{"delta": 15, "reason": "Operator: post-meeting positive signal not captured by behavioral events"}
```

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"marketing.ops"` | `marketingAutomation::DefineLeadScoringModel` | `MarketingLeadScoringModel::*` | `tenant_class`, `lead_scoring_models_count` |
| `Service::"behavioral-consumer"` | `marketingAutomation::ScoreSubject` | `MarketingLeadScore::*` | `subject_hash`, `model_version` |
| `User::"marketing.ops"` | `marketingAutomation::ManualAdjustScore` | `MarketingLeadScore::score_id` | `delta`, `reason` |

Demo-trial gate: `tenant_class == 'demo_trial' && lead_scoring_models_count >= 1` denies new model.

## Workflow Steps

1. `LoadActiveModel` reads tenant's active model.
2. `EvaluateDemographicComponent` walks demographic_formula against subject's ontology demographic traits.
3. `EvaluateBehavioralComponent` walks behavioral_formula against subject's behavioral-profile events, applying exponential decay with `decay_half_life_days`.
4. If `predictive_component_enabled`, `CallIntelligence` for predictive component over gRPC.
5. `SumScores` produces `total_score`.
6. `PersistScore` writes `marketing_lead_score` row (upsert per subject_hash).
7. `EmitScored` emits `EVT-MARKETING-LEAD-SCORED`.
8. On manual adjust, `ApplyAdjustment` writes adjustment row and recomputes total.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-LEAD-SCORING-MODEL-DEFINED` | `tenant_id`, `model_id`, `version`, `tenant_class` |
| `EVT-MARKETING-LEAD-SCORED` | `subject_hash`, `score_id`, `total_score`, `demographic_score`, `behavioral_score`, `predictive_score`, `model_version` |
| `EVT-MARKETING-LEAD-SCORE-MANUAL-ADJUSTED` | `score_id`, `adjustment_id`, `delta`, `reason`, `principal_id` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Score subject | 60 ms | 250 ms | 700 ms | 2000 rps/cell | 99.9% |
| Recalculate batch (10k subjects) | 30 s | 90 s | 180 s | 20 jobs/hour/cell | 99.9% |

## Failure Modes + Recovery

- Demographic trait missing on subject → use formula default (zero) and emit `EVT-MARKETING-LEAD-SCORING-TRAIT-MISSING` for ontology backfill.
- Behavioral event store lag → use last-known-good cursor and emit stale-marker.
- Intelligence unreachable for predictive → fall back to (demographic + behavioral) and emit `EVT-MARKETING-LEAD-SCORING-PREDICTIVE-FALLBACK`.

## Migration Notes

HubSpot Lead Scoring uses Manual + Predictive Lead Scoring; manual rules become `demographic_formula` + `behavioral_formula` entries. HubSpot Predictive (Enterprise) maps to predictive_component (Oyatie predictive runs on intelligence µservice).

Marketo Lead Scoring uses Score Change Triggers; preserved as behavioral_formula entries plus decay rules.

Mailchimp Premium Predicted Demographics is narrower; preserved as predictive_component.

## Cross-µservice Handoffs

- `ontology` supplies demographic trait registry.
- `behavioral-profile` supplies event log + decay queries.
- `intelligence` supplies predictive component prediction.
- `workflow-canvas` triggers can read score values (Score-Change-Trigger pattern).
- `audit-chain` seals events.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-037-lead-scoring.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-037-lead-scoring.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].
