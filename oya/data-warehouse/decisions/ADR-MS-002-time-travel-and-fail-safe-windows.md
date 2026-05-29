---
adr_id: ADR-MS-002
microservice: data-warehouse
title: Time-travel and fail-safe windows
date: 2026-05-21
status: accepted
wave: Wave-15A-DATA-WAREHOUSE-FIX
defect_closed: F-D3-02
binding_adrs: [ADR-0244, ADR-0329, ADR-0331]
---

# ADR-MS-002 — Time-travel and fail-safe windows

## Context

Snowflake's flagship analytical recovery story is two layers:

- Time-travel: queryable history for up to 90 days at 1-second resolution
  (Enterprise edition default 90; Standard default 1).
- Fail-safe: an additional 7-day post-time-travel SRE-mediated recovery
  window that the customer cannot query directly.

Together they give the tenant up to 97 days of "you can recover from any
accident in the last quarter" assurance. BigQuery offers 7 days at table
level (configurable up to seven). Databricks offers Delta Lake time-travel
at version+timestamp granularity, bounded by `delta.logRetentionDuration`
+ `delta.deletedFileRetentionDuration`.

Wave-15A audit found data-warehouse had not authored these primitives at
all (F-D4-D-01). This decision picks the canonical windows.

## Decision

Time-travel and fail-safe are tenant-purchasable composable billing
components per ADR-0331:

| Knob | `demo_trial` | `paid` default | `paid` max |
|---|---|---|---|
| `time_travel_storage_days` | 0 | 7 | 90 |
| `fail_safe_storage_days` | 0 | 7 | 35 |

Rationale:

- `demo_trial` cannot use time-travel because it is a substance feature
  that costs storage and adds operational complexity; not appropriate for
  a free trial.
- `paid` default of 7+7 = 14 days matches Snowflake Standard edition.
- `paid` max of 90+35 = 125 days exceeds Snowflake Enterprise (90+7 = 97)
  on the fail-safe side; oyatie SRE chose 35 days to align with HIPAA
  audit response windows (30 days plus a 5-day SRE buffer).

Resolution semantics:

- Time-travel reads use the underlying lake format's timeline (Delta log,
  Iceberg snapshot, Hudi timeline). p99 ≤ 1 s (see
  `slos/time-travel-resolution.openslo.yaml`).
- Fail-safe reads require operator + SRE attestation under
  `emergency-services-bypass.cedar` (two-person rule, 24-hour auto-expire).

VACUUM / snapshot-expiration / Hudi cleaning honor the purchased window;
no automatic GC inside the window.

## Consequences

- Tenants who buy a 90-day time-travel pay storage cost proportional to
  90 days of write volume. The bill must be transparent.
- The fail-safe storage line item is priced higher than time-travel
  storage to reflect SRE on-call cost (no self-service query).
- Cross-region replication of the time-travel and fail-safe windows is
  delegated to `multi-region.md` SLO.

## Alternatives considered

- Fixed 7-day windows like BigQuery: rejected because it cuts off the
  HIPAA / SOC-2 audit use case at the knees.
- Snowflake Enterprise's exact 90+7 model: rejected because 7 days of
  fail-safe is too short for HIPAA OCR audit response times observed in
  the field.

End of ADR-MS-002.
