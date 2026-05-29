---
id: ADR-AN-001
title: "Analytics TTL policy is workload-class specific"
status: Accepted
date: 2026-05-18
microservice: analytics
related_oyatie_adrs:
  - ADR-0003
  - ADR-0193
  - ADR-0195
  - ADR-0243
  - ADR-0244
  - ADR-0251
  - ADR-0263
decision_owner: council-analytics + ops-compliance + ops-finops
---

# ADR-AN-001: Analytics TTL policy is workload-class specific

## Context

- The named architectural pressure is `retention-by-purpose-not-retention-by-table-default`.
- Analytics stores audit events, business KPIs, billing rollups, telemetry rollups, and materialized-view intermediates.
- ADR-0193 establishes TTL, partition rotation, and cold-tier requirements.
- ADR-0195 establishes ClickHouse materialized views as the default streaming aggregation tier.
- ADR-0244 requires every row to carry tenant scope.
- ADR-0251 requires compliance-pack retention overlays.
- Prior incident class `fleet-wide-ttl-under-retained-audit` deleted audit records before SOC 2 review.
- Prior incident class `kpi-over-retention-cost-spike` kept low-value dashboard metrics for 7 years.
- Prior incident class `manual-partition-drop-no-receipt` removed data without proof-of-erasure evidence.
- Prior incident class `mv-intermediate-immortal` let AggregatingMergeTree state tables grow without deletion.
- Audit logs carry SOC 2 CC7.2 and ISO/IEC 27001 A.8.15 obligations.
- Billing rollups carry tax and dispute windows under IRS Rev. Proc. 97-22, VAT Directive 2006/112/EC Art. 242, and KR Framework Act on National Taxes Art. 85-3.
- Business KPIs carry operational utility but become low-signal after one year.
- Proof-of-erasure receipts are compliance evidence and cannot be deleted.
- A single retention value would either under-retain audit or over-retain KPIs.
- Retention must be visible in DDL, not hidden in a runbook.
- Retention must map to data class and workload class.
- Retention must produce metrics and dashboards.
- Retention must allow pack overlays without changing source code.
- Retention must be enforceable by CI before migrations land.
- Retention must be reversible only through a new ADR.
- The implementation must be buildable from this ADR by an intern.

## Decision

- We choose `workload-class TTL`.
- The named pattern is `purpose-bound retention with DDL-enforced deletion`.
- The analytics primary warehouse is ClickHouse 26.3 LTS.
- ClickHouse hot storage is NVMe.
- ClickHouse cold storage is S3-compatible object storage.
- OCI Object Storage is the default cold tier for OCI packs.
- Cloudflare R2 is not used for analytics retention.
- PostgreSQL 16.6 stores retention policy metadata and proof-of-erasure receipts.
- The workload class is mandatory on every table comment.
- The workload class is mandatory in migration metadata.
- `audit_log` hot tier retention is 90 days.
- `audit_log` cold tier retention is 7 years.
- `audit_log` final delete is 7 years.
- `business_kpi` hot tier retention is 90 days.
- `business_kpi` cold tier retention is 1 year.
- `business_kpi` final delete is 1 year.
- `billing_rollup_daily` hot tier retention is 30 days.
- `billing_rollup_daily` cold tier retention is 7 years.
- `billing_rollup_daily` final delete is 7 years.
- `billing_rollup_monthly` hot tier retention is 30 days.
- `billing_rollup_monthly` cold tier retention is 7 years.
- `billing_rollup_monthly` final delete is 7 years.
- `telemetry_rollup` hot tier retention is 30 days.
- `telemetry_rollup` cold tier retention is 1 year.
- `telemetry_rollup` final delete is 1 year.
- `mv_intermediate` hot tier retention is 90 days.
- `mv_intermediate` has no cold tier.
- `mv_intermediate` final delete is 90 days.
- `erasure_receipt` hot tier retention is permanent.
- `erasure_receipt` has no cold tier.
- `erasure_receipt` final delete is never.
- Pack overlays may lengthen audit and billing retention.
- Pack overlays may shorten KPI retention when law requires minimization.
- No overlay may delete proof-of-erasure receipts.
- TTL clauses are emitted at table creation time.
- Retention changes use additive migrations that preserve evidence.
- Manual partition drops are forbidden outside the retention controller.
- Cedar action `analytics.retention_policy.apply` gates retention changes.
- Cedar action `analytics.retention_partition.drop` gates partition deletion.
- Cedar action `analytics.retention_receipt.read` gates receipt access.
- Retention deletion emits audit-chain events.

## Alternatives Considered

### Single fleet-wide seven-year retention

- Pro: compliance-safe for audit and billing.
- Pro: easy to explain.
- Pro: fewer policy branches.
- Con: over-retains business KPIs.
- Con: storage cost grows unnecessarily.
- Con: conflicts with GDPR Art. 5(1)(e) storage limitation.
- Con: makes tenant erasure explanations weaker.
- Tradeoff: conservative retention but poor minimization and FinOps.
- Rejected.

### Single fleet-wide one-year retention

- Pro: low storage cost.
- Pro: strong minimization.
- Pro: simple operations.
- Con: under-retains audit logs.
- Con: under-retains billing rollups.
- Con: violates contract audit windows.
- Con: fails tax-dispute use cases.
- Tradeoff: cheap but non-compliant.
- Rejected.

### Tenant-configurable arbitrary TTL

- Pro: maximum tenant flexibility.
- Pro: enterprise customers can match internal policy.
- Pro: fewer pack-specific defaults.
- Con: tenants can misconfigure compliance-critical retention.
- Con: support matrix explodes.
- Con: CI cannot reason about arbitrary values.
- Tradeoff: flexibility but governance risk.
- Rejected.

### Manual partition drop runbook

- Pro: no DDL complexity.
- Pro: operators can react case by case.
- Pro: easy early launch.
- Con: fragile and non-repeatable.
- Con: no automatic proof-of-erasure.
- Con: misses new tables.
- Tradeoff: operational simplicity but poor evidence.
- Rejected.

### Object-storage lifecycle only

- Pro: uses vendor-native lifecycle rules.
- Pro: reduces ClickHouse TTL complexity.
- Pro: good for cold blobs.
- Con: hot table rows remain ungoverned.
- Con: ClickHouse metadata and projections can diverge.
- Con: proof-of-erasure is harder to join to table schema.
- Tradeoff: good cold-tier cleanup but incomplete warehouse retention.
- Rejected.

## Consequences

- Positive: retention is tied to workload purpose.
- Positive: audit and billing are retained for required windows.
- Positive: KPI and telemetry storage cost is bounded.
- Positive: TTL is visible in ClickHouse DDL.
- Positive: proof-of-erasure receipts survive deletion.
- Negative: table creation requires more metadata.
- Negative: pack overlays add migration-test complexity.
- Negative: retention controller becomes compliance-critical.
- Negative: permanent receipt tables need separate capacity planning.
- Neutral: future workload classes require ADR amendment.
- Neutral: tenant-specific extensions can be added only through governed overlays.
- Follow-up work: implement `oya-governance-ttl-presence`.
- Follow-up work: add retention controller reconciliation.
- Follow-up work: add dashboard export for regulator evidence.
- Follow-up work: add cold-tier restore drill.

## Implementation Notes

- Data shape `AnalyticsRetentionPolicyV1` contains `workload_class`.
- Data shape `AnalyticsRetentionPolicyV1` contains `hot_days`.
- Data shape `AnalyticsRetentionPolicyV1` contains `cold_days`.
- Data shape `AnalyticsRetentionPolicyV1` contains `delete_days`.
- Data shape `AnalyticsRetentionPolicyV1` contains `cold_storage_policy`.
- Data shape `AnalyticsRetentionPolicyV1` contains `pack_overlay_id`.
- Data shape `AnalyticsRetentionPolicyV1` contains `legal_basis`.
- Data shape `AnalyticsRetentionPolicyV1` contains `evidence_retention_required`.
- Data shape `AnalyticsRetentionReceiptV1` contains `receipt_id`.
- Data shape `AnalyticsRetentionReceiptV1` contains `tenant_id`.
- Data shape `AnalyticsRetentionReceiptV1` contains `database_name`.
- Data shape `AnalyticsRetentionReceiptV1` contains `table_name`.
- Data shape `AnalyticsRetentionReceiptV1` contains `partition_id`.
- Data shape `AnalyticsRetentionReceiptV1` contains `workload_class`.
- Data shape `AnalyticsRetentionReceiptV1` contains `deleted_at`.
- Data shape `AnalyticsRetentionReceiptV1` contains `row_count_estimate`.
- Data shape `AnalyticsRetentionReceiptV1` contains `ttl_policy_version`.
- Data shape `AnalyticsRetentionReceiptV1` contains `audit_event_id`.
- ClickHouse table comments use `workload_class=<value>;ttl_policy=<version>`.
- ClickHouse storage policy is `hot_cold`.
- ClickHouse disk names are `nvme_hot` and `object_cold`.
- TTL syntax moves hot rows to `object_cold`.
- TTL syntax deletes after final retention date.
- Example audit DDL uses `TTL emitted_at + INTERVAL 90 DAY TO DISK 'object_cold', emitted_at + INTERVAL 7 YEAR DELETE`.
- Example KPI DDL uses `TTL bucket_start + INTERVAL 90 DAY TO DISK 'object_cold', bucket_start + INTERVAL 1 YEAR DELETE`.
- Example MV intermediate DDL uses `TTL bucket_start + INTERVAL 90 DAY DELETE`.
- API endpoint `GET /v1/internal/retention/policies` lists policies.
- API endpoint `POST /v1/internal/retention/reconcile` reconciles DDL to policy.
- API endpoint `POST /v1/internal/retention/drop-partition` executes governed deletion.
- API endpoint `GET /v1/internal/retention/receipts/{receipt_id}` returns evidence.
- API endpoint `GET /v1/tenants/{tenant_id}/analytics/retention` exposes tenant-visible policy.
- Cedar principal for policy apply is `Oyatie::Principal::Service("analytics-retention-controller")`.
- Cedar principal for partition drop is `Oyatie::Principal::Service("analytics-retention-controller")`.
- Cedar principal for receipt read is `Oyatie::Principal::Service("analytics-compliance-api")`.
- Cedar resource for policy is `Analytics::RetentionPolicy`.
- Cedar resource for partition is `Analytics::Partition`.
- Cedar resource for receipt is `Analytics::RetentionReceipt`.
- Example permit: principal `analytics-retention-controller`, action `analytics.retention_policy.apply`, resource `Analytics::RetentionPolicy::"audit_log"`, context `{pack_id:"gdpr-eu", delete_days:2555, workload_class:"audit_log"}`.
- Example permit: principal `analytics-retention-controller`, action `analytics.retention_partition.drop`, resource `Analytics::Partition::"tenant_01.audit_events.202601"`, context `{workload_class:"audit_log", age_days:2556, receipt_required:true}`.
- Example forbid: same drop action with context `{workload_class:"audit_log", age_days:400}`.
- Example forbid: any drop action with context `{workload_class:"erasure_receipt"}`.
- SLO `analytics-retention-reconcile.openslo.yaml` sets reconciliation p99 <= 15 minutes per cell.
- SLO `analytics-proof-of-erasure.openslo.yaml` sets receipt emission p99 <= 5 minutes after partition drop.
- SLO `analytics-cold-tier-restore.openslo.yaml` sets cold restore p95 <= 4 hours.
- Failure mode `ttl_clause_missing` blocks migration.
- Failure mode `receipt_missing_after_drop` opens Sev-1.
- Failure mode `cold_tier_write_failed` stops inserts for affected table.
- Failure mode `pack_overlay_invalid` blocks deploy.
- Failure mode `manual_partition_drop_detected` opens Sev-1 and freezes retention worker.

## Verification

- Test `retention_policy_requires_workload_class` validates metadata.
- Test `audit_log_ttl_is_seven_years` validates DDL.
- Test `business_kpi_ttl_is_one_year` validates DDL.
- Test `billing_rollup_ttl_is_seven_years` validates DDL.
- Test `mv_intermediate_ttl_is_ninety_days` validates DDL.
- Test `erasure_receipt_never_deleted` validates policy.
- Test `manual_partition_drop_forbidden` validates Cedar.
- Test `partition_drop_emits_receipt` validates evidence.
- Test `pack_overlay_can_lengthen_audit_retention` validates overlay.
- Test `pack_overlay_cannot_delete_receipts` validates invariant.
- Metric `oya_analytics_retention_reconcile_lag_seconds` must stay below 900.
- Metric `oya_analytics_ttl_missing_total` must remain zero.
- Metric `oya_analytics_retention_receipt_missing_total` must remain zero.
- Metric `oya_analytics_cold_tier_bytes` tracks FinOps.
- Dashboard `analytics-retention.json` shows policy drift and deletion receipts.
- Dashboard `analytics-cold-tier.json` shows object storage growth and restore drills.
- Dashboard `analytics-compliance-evidence.json` shows audit, billing, and receipt coverage.
- CI check `oya-governance-ttl-presence` validates every ClickHouse table.
- CI check `analytics-retention-cedar` validates permits and forbids.
- CI check `analytics-retention-ddl-fixtures` validates generated DDL.
- CI check `analytics-retention-pack-overlays` validates GDPR, KR, HIPAA, and LGPD overlays.
- Chaos test blocks object storage and expects inserts to fail safely.
- Restore drill runs quarterly against a 90-day-old audit partition.
- Regulator evidence export runs monthly and archives receipts for 7 years.

## References

- ADR-0003: Audit-chain and evidence emission.
- ADR-0193: Analytics storage, TTL, partition rotation, and cold tier.
- ADR-0195: Materialized views as stream-processing default.
- ADR-0243: Cedar as Universal Gate.
- ADR-0244: Tenant as universal scoping primitive.
- ADR-0251: Compliance pack cell certification levels.
- ADR-0263: Observability emission contract.
- ClickHouse 26.3 LTS TTL documentation.
- ClickHouse storage policies and disks documentation.
- OCI Object Storage lifecycle documentation.
- PostgreSQL 16.6 documentation.
- GDPR Art. 5(1)(e) and Art. 30.
- SOC 2 CC7.2.
- ISO/IEC 27001:2022 A.8.15.
- IRS Rev. Proc. 97-22.
- VAT Directive 2006/112/EC Art. 242.
- KR Framework Act on National Taxes Art. 85-3.
- NIST SP 800-53 Rev. 5 AU-11 and SI-12.
