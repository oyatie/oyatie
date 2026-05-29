---
doc_class: ImplementationPlan
ip_id: IP-036-send-time-optimization
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0263, ADR-0321, ADR-0328]
bounded_context: send-time-optimization
journey_id: J-MA-36-per-recipient-send-time-prediction
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-036: Send-Time Optimization

## Context

Marcus Chen schedules a product-update email to a 30k-contact segment. HubSpot Send Time Optimization, Marketo Optimal Send Time, and Mailchimp Send Time Optimization all predict the best send window per recipient. This slice integrates with intelligence µservice for prediction and honors frequency-cap + deliverability admit decisions — counterparts do not honor cross-channel frequency cap because they cap per-channel-per-list rather than per-subject-per-purpose.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_sto_profile` | `profile_id` | `uuid primary key` | Per-subject STO profile. |
| `marketing_sto_profile` | `tenant_id` | `uuid not null` | Tenant partition. |
| `marketing_sto_profile` | `subject_hash` | `text not null` | Hashed subject ref. |
| `marketing_sto_profile` | `predicted_window` | `tstzrange` | Optimal send window. |
| `marketing_sto_profile` | `prediction_confidence` | `numeric(4,3)` | 0.000-1.000. |
| `marketing_sto_profile` | `fallback_window` | `tstzrange not null` | Used when confidence < 0.6. |
| `marketing_sto_profile` | `last_prediction_hlc` | `hlc not null` | When prediction was last computed. |
| `marketing_sto_profile` | `override_window` | `tstzrange` | Optional operator override. |

## API Endpoints

REST `POST /v1/marketing-automation/send-time-optimization/predict`:

```json
{
  "tenant_id": "...",
  "subject_hash": "h_abc123",
  "purpose": "product_marketing",
  "channel": "email"
}
```

Returns `{predicted_window, prediction_confidence, fallback_applied}`.

REST `POST /v1/marketing-automation/send-time-optimization/{profile_id}:override` lets an operator set `override_window`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `Service::"send-scheduler"` | `marketingAutomation::PredictSendWindow` | `MarketingSTO::*` | `subject_hash`, `purpose`, `channel`, `tenant_class` |
| `User::"marketing.ops"` | `marketingAutomation::OverrideSendWindow` | `MarketingSTO::profile_id` | `tenant_class` |

## Workflow Steps

1. `LoadSubjectProfile` reads or creates `marketing_sto_profile` row.
2. `RequestPredictionFromIntelligence` calls intelligence µservice via gRPC-over-HTTP/3 with subject behavior history + tenant cohort.
3. If `prediction_confidence < 0.6`, use `fallback_window` (default: tenant default send window, e.g., Tuesday 09:00-11:00 local).
4. `CheckFrequencyCapReservation` queries frequency-cap to ensure window does not collide with a denial window; if collision, shift to next admissible window.
5. `CheckDeliverabilityAdmit` queries deliverability for budget availability in the window; if budget exhausted, shift.
6. `EmitWindowPredicted` writes back `predicted_window` and emits audit event.

Decision branches:
- Intelligence unreachable → fall back to operator-configured tenant default; emit `EVT-MARKETING-STO-FALLBACK-APPLIED`.
- All future windows collide with frequency-cap denial → return `409 no_admissible_window` (operator must adjust frequency-cap or purpose).
- Override active → return override window directly without prediction call.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-STO-WINDOW-PREDICTED` | `tenant_id`, `subject_hash`, `predicted_window`, `prediction_confidence`, `tenant_class` |
| `EVT-MARKETING-STO-FALLBACK-APPLIED` | `tenant_id`, `subject_hash`, `fallback_reason` |
| `EVT-MARKETING-STO-WINDOW-OVERRIDDEN` | `profile_id`, `override_window`, `principal_id` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Predict window | 50 ms | 200 ms | 500 ms | 5000 rps/cell | 99.9% |
| Override window | 20 ms | 80 ms | 200 ms | 100 rps/cell | 99.95% |

## Failure Modes + Recovery

- Intelligence inference latency spike → fall back to tenant default within 200 ms hard timeout.
- Stale subject profile (no recent behavior) → use cohort-mean prediction; flag low confidence.
- Override window in the past → 422 `override_window_must_be_future`.

## Migration Notes

HubSpot STO + Marketo Optimal Send Time + Mailchimp STO all use vendor-proprietary ML models with no cross-vendor portability. Oyatie recomputes per-subject predictions from behavioral-profile events; first-pass predictions use cohort-mean during warmup period (first 90 days).

## Cross-µservice Handoffs

- `intelligence` provides per-subject prediction (PyTorch model served at intelligence µservice; this µservice is Rust-strict and calls intelligence over gRPC).
- `behavioral-profile` supplies per-subject behavior history.
- `frequency-cap` consulted for admissible-window check.
- `deliverability` consulted for budget availability.
- `audit-chain` seals events.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-036-send-time-optimization.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-036-send-time-optimization.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].
