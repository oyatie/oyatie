---
doc_class: ImplementationPlan
ip_id: IP-044-behavioral-profile
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0263, ADR-0321, ADR-0328]
bounded_context: behavioral-profile
journey_id: J-MA-44-per-contact-behavior-aggregation
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-044: Behavioral Profile

## Context

Behavioral profile aggregates per-contact behavior events (page-views, email-opens, content-downloads, form-submits, in-app actions, custom-events) over time with HLC stamps and derives traits used by segments + lead-scoring. HubSpot Behavioral Event (Enterprise) + Marketo Activity Log + Mailchimp Audience Insights are the references. Distinct from segment (segment.materialize maps predicate → membership; behavioral-profile maps subject → trait history).

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_behavioral_event` | `event_id` | `uuid primary key` | Event row. |
| `marketing_behavioral_event` | `tenant_id` | `uuid not null` | Tenant partition. |
| `marketing_behavioral_event` | `subject_hash` | `text not null` | Subject ref. |
| `marketing_behavioral_event` | `event_class` | `text not null` | From tenant event-schema registry (page_view, email_open, form_submit, custom, etc.). |
| `marketing_behavioral_event` | `event_properties` | `jsonb not null` | Per-event-class payload (validated by registry). |
| `marketing_behavioral_event` | `occurred_at_hlc` | `hlc not null` | HLC stamp. |
| `marketing_behavioral_trait_rule` | `rule_id` | `uuid primary key` | Trait derivation rule. |
| `marketing_behavioral_trait_rule` | `tenant_id` | `uuid not null` | Tenant. |
| `marketing_behavioral_trait_rule` | `trait_name` | `text not null` | Derived trait name. |
| `marketing_behavioral_trait_rule` | `rule_expr` | `jsonb not null` | Aggregation expression. |
| `marketing_behavioral_trait` | `trait_id` | `uuid primary key` | Derived trait row. |
| `marketing_behavioral_trait` | `subject_hash` | `text not null` | Subject. |
| `marketing_behavioral_trait` | `rule_id` | `uuid not null` | FK. |
| `marketing_behavioral_trait` | `value` | `jsonb not null` | Derived value. |
| `marketing_behavioral_trait` | `derived_at_hlc` | `hlc not null` | HLC. |

## API Endpoints

REST `POST /v1/marketing-automation/behavioral-profile/events`:

```json
{
  "tenant_id": "...",
  "subject_hash": "h_abc123",
  "event_class": "pricing_page_view",
  "event_properties": {"page_url": "https://acme.io/pricing", "duration_seconds": 47, "scrolled_to_bottom": true}
}
```

REST `POST /v1/marketing-automation/behavioral-profile/trait-rules` defines a derivation rule:

```json
{
  "trait_name": "high_intent_pricing_engagement",
  "rule_expr": {
    "where": {"event_class": "pricing_page_view"},
    "aggregate": "count_within_days",
    "window_days": 14,
    "threshold": 3,
    "output": "boolean"
  }
}
```

REST `GET /v1/marketing-automation/behavioral-profile/{subject_hash}/traits` returns derived traits.

REST `POST /v1/marketing-automation/behavioral-profile/{subject_hash}:recompute` triggers recomputation.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `Service::"event-consumer"` | `marketingAutomation::IngestBehavioralEvent` | `MarketingBehavioralEvent::*` | `event_class`, `tenant_class`, `daily_event_count` |
| `User::"marketing.ops"` | `marketingAutomation::DefineTraitRule` | `MarketingBehavioralTraitRule::*` | `tenant_class` |

Demo-trial gate: `tenant_class == 'demo_trial' && daily_event_count >= 1000` denies further ingestion.

## Workflow Steps

1. `ValidateEventClass` against tenant event-schema registry.
2. `ValidateEventProperties` per event_class schema.
3. `PersistEvent` writes row partition by (tenant_id, time bucket).
4. `EvaluateAffectedRules` finds trait rules that match `event_class`.
5. `RecomputeTrait` per matched rule.
6. `EmitIngestion` emits `EVT-MARKETING-BEHAVIOR-EVENT-INGESTED`; emits `EVT-MARKETING-BEHAVIOR-TRAIT-DERIVED` if trait value changed.
7. Downstream subscribers: segment.materialize (consumes traits), lead-scoring (consumes behavioral component).

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-BEHAVIOR-EVENT-INGESTED` | `event_id`, `subject_hash`, `event_class`, `tenant_class` |
| `EVT-MARKETING-BEHAVIOR-TRAIT-DERIVED` | `subject_hash`, `trait_name`, `value`, `derived_at_hlc` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Ingest event | 15 ms | 60 ms | 150 ms | 50000 events/s/cell | 99.99% |
| Derive trait | 30 ms | 120 ms | 300 ms | 10000 derivations/s/cell | 99.95% |
| Read traits | 10 ms | 40 ms | 100 ms | 5000 rps/cell | 99.99% |

## Failure Modes + Recovery

- Event schema registry drift → unknown event_class quarantined with `EVT-MARKETING-BEHAVIOR-EVENT-QUARANTINED`; operator resolves schema.
- Trait rule compile failure → 422 at rule definition; rule does not persist.
- Recompute storm (registry change affects 10M subjects) → admission control; batch recompute over window.

## Migration Notes

HubSpot Behavioral Event (Enterprise) export uses HubSpot Custom Behavioral Event API; events preserved verbatim with `migrated_from: hubspot`. Marketo Custom Activity export similar. Mailchimp Audience Insights are pre-aggregated; lossy migration (subject-level events not recoverable beyond 90 days).

## Cross-µservice Handoffs

- `ontology` provides event-schema registry.
- `segment` consumes traits for predicate evaluation.
- `lead-scoring` consumes behavioral aggregates.
- `attribution` consumes events as touch events.
- `audit-chain` seals every event + trait derivation.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-044-behavioral-profile.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-044-behavioral-profile.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-044-behavioral-profile.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-044-behavioral-profile.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
