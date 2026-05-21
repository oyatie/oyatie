---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M03-P03
title: Cloud Data + Billing + Observability + FinOps + Marketplace
status: in-progress (IP-001 provider-neutral data kernel green 2026-05-21; IP-002 regional-pack tax-invoice validation green 2026-05-21; provider/runtime adapters not claimed)
purpose: Bring data services, billing/tax, observability, FinOps, and marketplace to W-Cloud-Preview readiness.
execution_variant: merge-into-existing-crates
decided_at: "2026-05-17"
decided_by: user-directive-option-2
execution_variant_note: "Delta-1 adds DatabaseEngine+DataService+provision_data_service to oya-cloud-data-kernel::data_service (IP-001). No new crates scaffolded; no new workspace deps. Mirrors PR #60-#91 merge-variant pattern."
---

# M03-P03 — Cloud Data + Billing + Observability + FinOps + Marketplace

## Purpose
Per [`../../../../../.omx/notepad.md`](../../../../../.omx/notepad.md) 2026-05-11 checkpoints (billing tax, observability audit-read, FinOps, marketplace). Provider-agnostic: Postgres/Citus/pgvector/Valkey/Kafka/ClickHouse.

## Acceptance
- `cloud.data.{postgres,citus,pgvector,valkey,kafka,clickhouse}` surfaces stable.
- `cloud.billing.invoice.generate` per-region tax-invoice format.
- `cloud.observability.audit.read` + metric/log/trace/alert/dashboard contracts.
- `cloud.finops.report` per-tenant per-axis cost allocation.
- `cloud.marketplace.*` ISV onboarding + listing + private offers.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Cloud Data services kernel + adapters (provider-agnostic) | provider-neutral-kernel-green; provider runtime adapters not claimed | [`IP-001-cloud-data-adapters.md`](IP-001-cloud-data-adapters.md) |
| IP-002 | Cloud Billing tax-invoice + metering | regional-pack-tax-format-green; metering-outbox-runtime-pending | [`IP-002-billing-tax-metering.md`](IP-002-billing-tax-metering.md) |
| IP-003 | Cloud Observability audit-read + OpenTelemetry | partial | [`IP-003-observability-otel.md`](IP-003-observability-otel.md) |
| IP-004 | Cloud FinOps report + anomaly detection | partial | [`IP-004-finops-report.md`](IP-004-finops-report.md) |
| IP-005 | Cloud Marketplace ISV onboarding | partial | [`IP-005-marketplace-isv.md`](IP-005-marketplace-isv.md) |

## Estimated parallelism
5 agents; disjoint crate suffix.

## Symbols-touched
`crates/oya-cloud-{data,billing,observability,finops,marketplace}-*`.

## Agent-handoff
```
icm store -t context-oyatie -c "M03-P03 complete: cloud data + billing + observability + FinOps + marketplace stable" -i critical -k "M03,P03,cloud-data-billing,complete"
```
