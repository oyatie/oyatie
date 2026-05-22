---
doc_class: ImplementationPlan
ip_id: IP-040-abm-target-account
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0245, ADR-0263, ADR-0321, ADR-0328]
bounded_context: abm
journey_id: J-MA-40-account-based-marketing
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-040: ABM Target Account

## Context

Diana Alvarez runs ABM for a B2B client targeting Fortune 500 accounts. ABM is the account-level overlay of marketing-automation: target accounts (a subset of crm.account-master), account-level scoring, account-level workflow triggers, and intent-data ingestion (Bombora / G2 / 6sense / Demandbase). HubSpot ABM + Marketo Account-Based Marketing are the primary references; Mailchimp does not have a direct ABM primitive.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_abm_target_account` | `target_id` | `uuid primary key` | Target account row. |
| `marketing_abm_target_account` | `tenant_id` | `uuid not null` | Tenant partition. |
| `marketing_abm_target_account` | `crm_account_id` | `uuid not null` | FK to crm.account-master. |
| `marketing_abm_target_account` | `priority_band` | `text not null` | tier_1 / tier_2 / tier_3 (ABM target priority, NOT capability tier nor cell tier). |
| `marketing_abm_target_account` | `targeted_at_hlc` | `hlc not null` | When account was targeted. |
| `marketing_abm_account_score` | `score_id` | `uuid primary key` | Account score row. |
| `marketing_abm_account_score` | `target_id` | `uuid not null` | FK. |
| `marketing_abm_account_score` | `demographic_component` | `int not null default 0` | Firmographic score. |
| `marketing_abm_account_score` | `behavioral_component` | `int not null default 0` | Aggregated subject engagement at account. |
| `marketing_abm_account_score` | `intent_component` | `int not null default 0` | From intent-data adapter. |
| `marketing_abm_account_score` | `total_score` | `int not null` | Sum. |
| `marketing_abm_account_score` | `last_scored_hlc` | `hlc not null` | HLC. |
| `marketing_abm_intent_signal` | `signal_id` | `uuid primary key` | Per-intent-signal row. |
| `marketing_abm_intent_signal` | `target_id` | `uuid not null` | FK. |
| `marketing_abm_intent_signal` | `source_provider` | `text not null` | bombora / g2 / 6sense / demandbase. |
| `marketing_abm_intent_signal` | `topic` | `text not null` | Intent topic. |
| `marketing_abm_intent_signal` | `strength` | `int not null` | 0-100. |
| `marketing_abm_intent_signal` | `recorded_at_hlc` | `hlc not null` | HLC. |

## API Endpoints

REST `POST /v1/marketing-automation/abm/target-accounts`:

```json
{"tenant_id": "...", "crm_account_id": "...", "priority_band": "tier_1"}
```

REST `POST /v1/marketing-automation/abm/intent-signals`:

```json
{"target_id": "...", "source_provider": "bombora", "topic": "marketing automation", "strength": 85}
```

REST `POST /v1/marketing-automation/abm/{target_id}:score` recomputes.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"marketing.ops"` | `marketingAutomation::TargetAccount` | `MarketingABMTargetAccount::*` | `tenant_class`, `targeted_accounts_count` |
| `Service::"intent-adapter"` | `marketingAutomation::IngestIntentSignal` | `MarketingABMIntentSignal::*` | `source_provider`, `target_id` |

Demo-trial gate: `tenant_class == 'demo_trial' && targeted_accounts_count >= 25` denies new target.

## Workflow Steps

1. `ValidateCrmAccount` confirms crm_account_id exists via crm.account-master contract.
2. `AuthorizeTarget` calls Cedar.
3. `PersistTarget` writes row.
4. On intent signal, `IngestIntentSignal` writes row + recomputes score.
5. On score, `ComputeDemographic` reads crm.account-master firmographic fields.
6. `ComputeBehavioral` aggregates subject behavioral-profile events at the account.
7. `ComputeIntent` sums recent intent signals (decay 30 days).
8. `EmitScored` emits `EVT-MARKETING-ABM-ACCOUNT-SCORED`.
9. If total_score crosses threshold, fire workflow-canvas account-level trigger.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-ABM-ACCOUNT-TARGETED` | `target_id`, `crm_account_id`, `priority_band`, `tenant_class` |
| `EVT-MARKETING-ABM-INTENT-INGESTED` | `target_id`, `signal_id`, `source_provider`, `topic`, `strength` |
| `EVT-MARKETING-ABM-ACCOUNT-SCORED` | `target_id`, `total_score`, `demographic_component`, `behavioral_component`, `intent_component` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Target account | 60 ms | 250 ms | 600 ms | 100 rps/cell | 99.95% |
| Ingest intent signal | 30 ms | 150 ms | 400 ms | 1000 rps/cell | 99.9% |
| Score account | 100 ms | 500 ms | 1.5 s | 200 jobs/min/cell | 99.9% |

## Failure Modes + Recovery

- CRM account not found → 404 with hint to crm.account-master creation flow.
- Intent provider rate limit → exponential backoff; replay buffered signals.
- Score recompute race → CAS on `score_id`.

## Migration Notes

HubSpot ABM exports Target Account list; Marketo ABM exports Named Accounts. Both preserve as `marketing_abm_target_account` rows. Intent-data history is preserved per `source_provider`.

## Cross-µservice Handoffs

- `crm.account-master` is the account source-of-truth.
- `behavioral-profile` supplies subject behavior aggregated to account.
- `workflow-canvas` consumes account-level triggers.
- `intelligence` predicts account-level conversion likelihood.
- `audit-chain` seals events.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-040-abm-target-account.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-040-abm-target-account.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].
