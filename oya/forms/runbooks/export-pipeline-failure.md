---
doc_class: Runbook
title: Export pipeline failure (CSV / XLSX / JSON / Sheets-bridge)
microservice: forms
severity: "Sev-2 (cluster) / Sev-3 (single tenant)"
status: Accepted
owner_team: ops-sre-reliability + axis-forms
date: 2026-05-17
related_artifacts:
  - microservices/forms/failure-modes.md FM-04
  - microservices/forms/threat-model.md §"T-D-06"
  - microservices/forms/slos/export-csv-latency.openslo.yaml
doc_status: published
---

# Runbook: Export pipeline failure

## Purpose

Export-worker streams Forms responses to CSV / XLSX / JSON / sheets-bridge. Failures here block tenant compliance workflows (DSAR export, regulatory return, ad-hoc analytics). This runbook diagnoses + recovers.

## Trigger

ONE of:

1. **`oya_forms_export_queue_depth > 10k`** ≥ 10 min.
2. **`oya_forms_export_csv_latency_seconds{quantile="0.95"} > 5.0` ≥ 5 min** (over budget per `slos/export-csv-latency.openslo.yaml`).
3. **`oya_forms_export_failure_total` rate > 1/min**.
4. **`oya_forms_export_pii_unredacted_total > 0`** (P0 — separate runbook `pii-leak-incident-p0.md`).
5. **Object storage upload error rate spike**.

## Severity

- Cluster-wide export queue backed up: Sev-2.
- Single-tenant queue: Sev-3.
- PII leak: Sev-1 → escalate to `pii-leak-incident-p0.md`.

## Impact

- Tenant cannot complete DSR export within SLA.
- Tenant ad-hoc analytics blocked.
- Sheets-bridge stale (response-bridge to sheets µservice lags).

## Pre-checks

1. Queue depth + per-tenant breakdown: `dashboards/response-pipeline.json` panel "export queue by tenant top-N".
2. Worker HPA state: `kubectl -n forms get hpa export-worker`.
3. Object storage health: OCI Object Storage status page.
4. Postgres read replica: any read backpressure?

## Recovery Path A — Queue backed up (legitimate volume)

| Step | Action |
|---|---|
| 1 | Scale export-worker HPA: `kubectl -n forms scale deployment/export-worker --replicas 15`. |
| 2 | Verify object storage write throughput. |
| 3 | If single-tenant >> others: apply per-tenant export quota: `cargo run -p oya-dev-cli -- forms export-quota --tenant <id> --max-concurrent 3`. |
| 4 | Tenant comms if queue growth sustained. |

## Recovery Path B — Object storage outage

| Step | Action |
|---|---|
| 1 | Verify OCI Object Storage status page. |
| 2 | Switch to fallback bucket (different AD/region within pack): `cargo run -p oya-dev-cli -- forms export-bucket --pack <pack> --fallback`. |
| 3 | Tenant comms: exports queued; will deliver when storage recovers. |
| 4 | Monitor; revert to primary when OCI recovers. |

## Recovery Path C — Postgres read backpressure

| Step | Action |
|---|---|
| 1 | Add read-replica: `kubectl -n forms scale statefulset/postgres-replica --replicas 4`. |
| 2 | Route export-worker to read replica only (not primary). |
| 3 | If shard-skew: per ADR-0164 cell migration. |

## Recovery Path D — Sheets-bridge failure

| Step | Action |
|---|---|
| 1 | Verify sheets µservice health: `microservices/sheets/dashboards/`. |
| 2 | If sheets degraded: engage sheets on-call. |
| 3 | Forms-side: queue sheets-bridge updates; emit `FormSheetBridgeRetry` event. |
| 4 | Per-tenant comms if sustained. |

## Recovery Path E — Export PII-redaction failure

CRITICAL: if redaction fails, escalate to Sev-1 per `pii-leak-incident-p0.md`. This runbook does NOT cover PII leak; see linked runbook.

## Verification

After recovery:
- `oya_forms_export_queue_depth < 1k`.
- `oya_forms_export_csv_latency_seconds{quantile="0.95"} ≤ 5.0`.
- `oya_forms_export_failure_total` rate < 0.1/min.
- Object storage write success rate ≥ 99.99%.

## Post-incident updates

- Postmortem within 5 business days.
- Per-tenant quota tuning per `capacity-model.md`.
- If recurring: capacity expansion ticket.

## References

- `failure-modes.md` FM-04.
- `threat-model.md` T-D-06.
- `slos/export-csv-latency.openslo.yaml`.
- `runbooks/pii-leak-incident-p0.md` (escalation path).
- ADR-0164 cell-pinning.
- OCI Object Storage SLA — `docs.oracle.com/iaas/Content/Object/`.
