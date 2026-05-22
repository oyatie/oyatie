---
doc_class: ImplementationPlan
ip_id: IP-035-a-b-test
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0263, ADR-0321, ADR-0328]
bounded_context: a-b-test
journey_id: J-MA-35-variant-test-framework
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-035: A/B Test Framework

## Context

Marcus Chen runs subject-line A/B tests on his trial-nurture email. HubSpot A/B Test, Marketo A/B Test, and Mailchimp Premium A/B Testing cover subject-line + content + send-time variants. This slice supports email + landing-page + workflow-canvas variant testing with statistical-significance auto-winner-selection.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_a_b_test` | `test_id` | `uuid primary key` | A/B test id. |
| `marketing_a_b_test` | `tenant_id` | `uuid not null` | Tenant partition. |
| `marketing_a_b_test` | `subject_kind` | `text not null` | email / landing-page / workflow-canvas. |
| `marketing_a_b_test` | `subject_id` | `uuid not null` | FK to email_id / page_id / canvas_id. |
| `marketing_a_b_test` | `variant_set` | `jsonb not null` | Array of {variant_id, allocation_bps} summing to 10000. |
| `marketing_a_b_test` | `significance_threshold` | `numeric(4,3) not null default 0.950` | 0.950 = 95% significance. |
| `marketing_a_b_test` | `winner_selection_rule` | `text not null` | auto_at_significance / manual / hold. |
| `marketing_a_b_test` | `status` | `text not null` | configured / running / concluded / aborted. |
| `marketing_a_b_test` | `winner_variant_id` | `uuid` | Set on conclude. |
| `marketing_a_b_test` | `p_value` | `numeric(7,6)` | Set on conclude. |
| `marketing_a_b_test_observation` | `observation_id` | `uuid primary key` | Per-variant per-subject observation. |
| `marketing_a_b_test_observation` | `test_id` | `uuid not null` | FK. |
| `marketing_a_b_test_observation` | `variant_id` | `uuid not null` | Allocated variant. |
| `marketing_a_b_test_observation` | `subject_hash` | `text not null` | Hashed subject. |
| `marketing_a_b_test_observation` | `outcome` | `text not null` | conversion / no_conversion / pending. |

## API Endpoints

REST `POST /v1/marketing-automation/a-b-tests`:

```json
{
  "tenant_id": "...",
  "subject_kind": "email",
  "subject_id": "email-uuid",
  "variant_set": [
    {"variant_id": "v1", "allocation_bps": 5000},
    {"variant_id": "v2", "allocation_bps": 5000}
  ],
  "significance_threshold": 0.950,
  "winner_selection_rule": "auto_at_significance"
}
```

REST `POST /v1/marketing-automation/a-b-tests/{test_id}:allocate` allocates a subject to a variant (returns variant_id).

REST `POST /v1/marketing-automation/a-b-tests/{test_id}:record-outcome` records conversion for a subject.

REST `POST /v1/marketing-automation/a-b-tests/{test_id}:conclude` computes winner.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"marketing.ops"` | `marketingAutomation::ConfigureABTest` | `MarketingABTest::*` | `tenant_class`, `active_a_b_tests_count` |
| `Service::"a-b-allocator"` | `marketingAutomation::AllocateVariant` | `MarketingABTest::test_id` | `subject_hash`, `consistent_hash_strategy` |

Demo-trial gate: `tenant_class == 'demo_trial' && active_a_b_tests_count >= 1` denies configure.

## Workflow Steps

1. `ValidateVariantSet` checks allocations sum to 10000.
2. `ValidateSubjectExists` confirms subject_id resolves per subject_kind.
3. `AuthorizeConfigure` calls Cedar.
4. `PersistTestConfig` writes `marketing_a_b_test` row.
5. On allocate, `ConsistentHashAllocator` deterministically maps (test_id, subject_hash) → variant_id by allocation_bps.
6. On outcome record, `RecordObservation` writes per-subject row.
7. On conclude, `ComputeStatisticalSignificance` runs (Z-test for binary outcomes; t-test for continuous); if `p_value < (1 - significance_threshold)` and `winner_selection_rule == 'auto_at_significance'`, set winner.
8. `SealConclude` emits `EVT-MARKETING-AB-TEST-CONCLUDED`.

Decision branches:
- Allocation sum != 10000 → 422 `allocation_sum_mismatch`.
- Subject already allocated to different variant → return prior variant (sticky allocation via consistent hash).
- Insufficient sample size on conclude → 409 `insufficient_sample`.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-AB-TEST-CONFIGURED` | `tenant_id`, `test_id`, `subject_kind`, `subject_id`, `variant_count`, `tenant_class` |
| `EVT-MARKETING-AB-TEST-ALLOCATED` | `test_id`, `subject_hash`, `variant_id`, `allocation_strategy: consistent_hash` |
| `EVT-MARKETING-AB-TEST-OUTCOME-RECORDED` | `test_id`, `observation_id`, `variant_id`, `outcome` |
| `EVT-MARKETING-AB-TEST-CONCLUDED` | `test_id`, `winner_variant_id`, `p_value`, `sample_size_per_variant`, `tenant_class` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Allocate variant | 5 ms | 25 ms | 60 ms | 10000 rps/cell | 99.99% |
| Record outcome | 10 ms | 50 ms | 150 ms | 5000 rps/cell | 99.99% |
| Compute significance + conclude | 200 ms | 1 s | 3 s | 10 jobs/min/cell | 99.9% |

## Failure Modes + Recovery

- Consistent-hash drift on tenant_id renumbering → freeze tests during tenant migration window.
- Statistical test outlier → require minimum sample size (default 1000 per variant) before auto-conclude eligibility.
- Concurrent conclude → CAS on status; second conclude is idempotent.

## Migration Notes

HubSpot A/B Test stores variant configuration alongside the parent email; migration extracts the variant set and recreates as an Oyatie a-b-test record. HubSpot uses two-variant tests; multivariate tests on HubSpot Marketing Hub Enterprise also supported.

Marketo A/B Test stores in Email Program metadata; multivariate (3+ variants) preserved.

Mailchimp A/B Testing (Premium) stores in Campaign metadata; preserve sample-size threshold + significance.

## Cross-µservice Handoffs

- `intelligence` provides statistical-significance calculation library.
- `audit-chain` seals every lifecycle event.
- `finops` consumes per_usage `ab_test_runs` meter.
- Parent aggregates (email / landing-page / workflow-canvas) receive winner assignment.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-035-a-b-test.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-035-a-b-test.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-035-a-b-test.md` matched [`finops`, `per_usage`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-035-a-b-test.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
