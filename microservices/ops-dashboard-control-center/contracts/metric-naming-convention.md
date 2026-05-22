---
doc_class: Contract
contract_id: CNT-OPS-004
status: active
date: 2026-05-20
owner: ops-sre-reliability
related_adrs: [ADR-0263, ADR-0130, ADR-0248]
---

# Metric Naming Convention — ops-dashboard-control-center

## Purpose

Canonical metric naming contract for all Prometheus metrics emitted by the
`ops-dashboard-control-center` µservice. All new metrics MUST follow this
convention; CI lane `oya-governance-metric-naming` enforces compliance.

## Namespace

All metrics use the prefix `oya_ops_control_center_`.

## Naming Schema

```
oya_ops_control_center_<subsystem>_<measurement>[_<unit>]
```

- `subsystem` — one of: `admin`, `cedar`, `tenant`, `cell`, `oncall`, `finops`,
  `adr`, `audit`, `ueba`, `session`, `pack`
- `measurement` — snake_case noun or noun phrase describing what is measured
- `unit` — optional SI unit suffix (`_seconds`, `_bytes`, `_total`, `_ratio`)

Suffixes follow Prometheus conventions:
- Counters end in `_total`
- Histograms omit the unit suffix (use `_bucket`, `_sum`, `_count` auto-generated)
- Gauges use descriptive noun (no suffix)

## Canonical Metric Register

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `oya_ops_control_center_admin_actions_total` | Counter | `action`, `principal_type`, `step_up_class`, `verdict` | Admin actions evaluated by Cedar |
| `oya_ops_control_center_cedar_eval_duration_seconds` | Histogram | `fragment_id`, `action` | Cedar eval latency per action |
| `oya_ops_control_center_cedar_eval_errors_total` | Counter | `fragment_id`, `error_type` | Cedar eval errors per fragment |
| `oya_ops_control_center_step_up_challenge_duration_seconds` | Histogram | `step_up_class` | Step-up auth challenge latency |
| `oya_ops_control_center_step_up_failures_total` | Counter | `step_up_class`, `reason` | Step-up auth failures |
| `oya_ops_control_center_tenant_scope_violations_total` | Counter | `principal_type`, `attempted_tenant_id` | Cross-tenant pivot attempts (Cedar FORBID) |
| `oya_ops_control_center_audit_events_emitted_total` | Counter | `event_class`, `sealed` | Audit events by class and seal status |
| `oya_ops_control_center_audit_seal_lag_seconds` | Histogram | `event_class` | Time from emission to Merkle seal |
| `oya_ops_control_center_ueba_anomaly_score` | Gauge | `principal_id` | Current UEBA anomaly score per operator |
| `oya_ops_control_center_session_recording_active` | Gauge | `principal_id` | 1 if T3 session recording active |
| `oya_ops_control_center_cell_health_score` | Gauge | `cell_id`, `region`, `tier` | Current cell health score (0.0–1.0) |
| `oya_ops_control_center_spiffe_svid_ttl_seconds` | Gauge | `cell_id`, `workload` | Remaining SVID TTL |
| `oya_ops_control_center_cilium_drops_total` | Counter | `cell_id`, `policy_name`, `direction` | Cilium policy drops per cell |
| `oya_ops_control_center_oncall_handoff_pending` | Gauge | `region` | Handoff records awaiting acknowledgement |
| `oya_ops_control_center_oncall_missed_handoffs_total` | Counter | `region` | Handoff acks not received within 15min |
| `oya_ops_control_center_finops_cost_usd` | Gauge | `cell_id`, `tenant_id`, `region` | Current period cost in USD |
| `oya_ops_control_center_finops_budget_utilization_ratio` | Gauge | `cell_id`, `tenant_id` | Budget utilization (0.0–1.0) |
| `oya_ops_control_center_adr_queue_depth` | Gauge | `status` | ADR promotion queue depth by status |
| `oya_ops_control_center_pack_fragment_soak_elapsed_seconds` | Gauge | `fragment_id`, `state` | Elapsed soak time per Cedar fragment |
| `oya_ops_control_center_request_duration_seconds` | Histogram | `handler`, `method`, `status_code` | HTTP handler latency |
| `oya_ops_control_center_request_total` | Counter | `handler`, `method`, `status_code` | HTTP requests by handler |

## Label Conventions

- `principal_type` values: `INTERNAL_OPS`, `PARTNER_AGENCY_OPS`, `AUDITOR`, `EMERGENCY_SERVICES`, `FOUNDRY_PIPELINE`
- `verdict` values: `PERMIT`, `FORBID`, `ERROR`
- `step_up_class` values: `T2_TOTP_OR_PASSKEY`, `T3_HARDWARE_KEY`, `T3_HARDWARE_KEY_QUORUM_2`
- `event_class` values: the 16 classes defined in ARCHITECTURE.md §observability
- All label values are lowercase snake_case; no PII in label values

## Cardinality Budget

Maximum label cardinality per metric: 10,000 unique label-value combinations.
High-cardinality fields (e.g., `principal_id`, `trace_id`) MUST NOT be used as
Prometheus label values; use exemplars or log correlation instead.

## SLO Alignment

SLO indicator queries reference metrics from this register verbatim. Any rename
requires a contract version bump and corresponding SLO file update.
