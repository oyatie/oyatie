---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-analytics
microservice: analytics
status: Draft
date: 2026-05-18
owner_team: council-analytics
doc_status: draft
related_adrs:
  - ADR-0193
  - ADR-0184
  - ADR-0195
  - ADR-0337
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
---

# Analytics µservice — Product Requirements Document

**Status:** Draft (introduced 2026-05-18 by data-substrate batch)
**Owner:** council-analytics
**Layout:** Flat per ADR-0131
**Primary ADRs:** ADR-0193 (ClickHouse OLAP), ADR-0184 (storage tier), ADR-0195 (stream processing)

## 1. Purpose

The analytics µservice owns the tenant-facing OLAP analytics warehouse — the place where every tenant's dashboards, audit-log queries, billing rollups, and ops-portal aggregations are served from. It is the canonical home for any read workload whose shape is *wide aggregate over many rows in a columnar store*. Per ADR-0184, Tier 1 Postgres OLTP does not own this shape; per ADR-0193, ClickHouse 26.3 LTS does.

This µservice is intentionally distinct from:

- **observability** µservice — owns ops/SRE telemetry (Prometheus + Mimir + Loki + Tempo per ADR-0186). Observability stores fleet-internal signals; analytics stores tenant-visible business data.
- **foundry** µservice — owns AI substrate including Milvus vector retrieval per ADR-0192. Foundry serves embedding similarity; analytics serves wide aggregates.

The two-µservice split is deliberate: blast-radius isolation, capacity isolation, ownership clarity. A tenant query storm on analytics never starves SRE telemetry; an SRE telemetry storm on observability never starves tenant dashboards.

## 2. Personas

| Persona | Workload | Latency budget |
|---|---|---|
| Tenant admin viewing workflow execution dashboards | Per-tenant rollups over per-day windows | <500ms p99 |
| Tenant ops viewing audit-log search | Filter+paginate over per-tenant per-axis event stream | <800ms p99 |
| Tenant finance viewing billing rollup | Per-day per-resource counter aggregates | <1s p99 |
| Internal SRE viewing ops portal | Per-cell capacity utilization | <1s p99 |
| Internal compliance officer running regulator export | Multi-month event range with axis filter | minutes (bulk export) |
| Internal capacity planner viewing fleet-wide cost attribution | Cross-cell per-µservice spend | <2s p99 |

## 3. Goals and Non-Goals

### Goals

- Sub-second query latency for tenant-facing dashboards (>=p99 < 500ms for the standard dashboard shape).
- Multi-billion row capacity per tenant per cell.
- Per-tenant strict isolation — no tenant can see another tenant's data via any query path.
- Multi-region residency — KR / EU strict residency packs.
- 7-year retention for audit + billing (compliance).
- Cold-tier S3 disk for retention beyond hot window.
- Native Materialized View ingest for rolling rollups (ADR-0195 default tier).

### Non-Goals

- Transactional consistency (OLTP) — that's Tier 1 Postgres.
- Vector retrieval — that's Milvus per ADR-0192.
- Full-text search — that's Meilisearch per ADR-0184 Tier 4.
- Sub-millisecond cache — that's Valkey per ADR-0184 Tier 3.
- Stream-processing escalation (Flink) — that's per-µservice opt-in per ADR-0195.

## 4. Architecture summary

### 4.1 Components

- **ClickHouse cluster** (3 shards × 2 replicas; ClickHouse Keeper 3-node quorum). Helm at `microservices/analytics/iac/helm/clickhouse/`.
- **OLAP client kernel** (`oya-shared-olap-client-kernel`) — engine-agnostic port.
- **ClickHouse adapter crate** (`oya-shared-olap-clickhouse-adapter` — to be authored when this µservice's app is wired; not in this batch).
- **REST + gRPC API surface** at `crates/oya-analytics-api/` (out of this batch's scope — IP-007 + IP-008 + IP-015).
- **Per-tenant database bootstrap controller** — listens for tenant-onboarded events from the tenancy µservice and creates the `tenant_{tenant_id}` database + per-table grants.

### 4.2 Per-tenant isolation

Per ADR-0193 §"Multi-tenancy isolation":
- **Database-per-tenant.** Naming pattern `tenant_{tenant_id}`.
- **Row-level policies.** Layered defense for tables that legitimately share rows (rare; reserved for fleet-wide ops dashboards).
- **Per-tenant quotas.** `CREATE QUOTA tenant_{id}` with `MAX queries`, `MAX read_rows`, `MAX insert_rows` per ADR-0155 projection.

### 4.3 Ingest pipeline

Per ADR-0153 outbox pattern: source µservices emit events to their transactional outbox; the analytics CDC pipeline projects them into ClickHouse via the `Kafka` engine consuming from Pulsar's Kafka-protocol endpoint. Materialized Views on the source table emit rolled-up rows into target `AggregatingMergeTree` tables for sub-second dashboard freshness.

### 4.4 Cold tier

Per ADR-0193 §"TTL + partition rotation + cold tier": hot tier on local NVMe (CSI fast-class); cold tier on SeaweedFS S3-compat. Per-table TTL clause moves rows after the hot window (default 90 days).

### 4.5 Cross-cell federation

ClickHouse `Distributed` table engine routes cross-shard queries within a cell. Cross-cell federation (rare; for global ops aggregates) goes through the `remote()` function with explicit per-cell ClickHouse endpoint enumeration in the federated table's DDL. Tenant queries never federate across cells — tenant data is residency-bound per ADR-0049.

## 5. Capacity targets

- Per-cell: 100TB hot tier (NVMe) + 1PB cold tier (S3). Ingest 100K rows/sec sustained, 500K rows/sec burst.
- Query QPS: 10K qps fleet-wide per cell at p99 <500ms.
- Tenant ceiling: top tenant ≤ 10B rows per table; above that, capacity planner pages.
- Daily backup window: 4h overnight for daily incremental; weekend window for full.

## 6. Cost model (rough)

Per cell at sizing target (100TB hot + 1PB cold):
- 6 ClickHouse server nodes × c5n.4xlarge equivalent → ~$2K/month per cell on commodity.
- 3 Keeper nodes × c5n.large → ~$200/month.
- S3 cold tier at 1PB → ~$25K/month (highly compressible — actual: ~$8K/month after ClickHouse compression).
- Per-cell total ~$10K-$30K/month depending on cold-tier fill.

Detailed cost model at `microservices/analytics/cost-budget.md` (deferred to follow-on doc batch — see parent-wiring-todo).

## 7. Compliance posture

- **PII handling.** Per ADR-0156 PII registry — analytics serves PII-tagged columns only via Cedar-authorized read paths. Tenant audit-log query is allowed; cross-tenant aggregation that would surface PII is forbidden by Cedar.
- **GDPR DSR.** Per ADR-0038 — tenant offboard drops the `tenant_{tenant_id}` database; proof-of-erasure emitted.
- **Audit chain.** Every analytics query against the audit-log surface emits its own audit event (recursive — auditing the audit-log query — to prevent silent observation).

## 8. Non-Functional Requirements

### DR posture (ADR-0343)

- Service target: RTO p99 ≤ 14400s and RPO p99 ≤ 900s for the default analytics cluster, matching the doctrine already propagated into analytics IPs; regulated tenant packs may override downward by compliance floor.
- Compliance floors considered: SOC2-T2 RTO 14400s/RPO 900s, ISO27001-2022 RTO 14400s/RPO 3600s, HIPAA-2024 RTO 3600s/RPO 300s/multi-region true, and KR-PIPA resident-registration-number RTO 3600s/RPO 300s/multi-region true. Protected healthcare/KR-RRN tables use the stricter 3600s/300s floor; general tenant dashboards stay at 14400s/900s.
- Failover runbook reference: `runbooks/restore-drill.md`, `runbooks/keeper-quorum-recovery.md`, `runbooks/ingest-lag-burn.md`, and `runbooks/capacity-rebalance.md`.
- Multi-region posture: active-active only for packs whose floor or tenant contract requires it; default analytics remains cell-local with scheduled cross-cell aggregation because live tenant queries never federate across cells.
- Tenant-visible behavior: dashboards may show delayed rollups during restore, but audit-log search and billing rollups keep tenant/cell boundaries and never read from an unauthorized replica.

### Capacity model (ADR-0340)

- Per-tenant baseline: one ClickHouse database, 100GiB hot tier, 1TiB cold tier allocation, 10 query connections, and a quota envelope sized below the 10B rows-per-table ceiling.
- Scaling dimension: `insert_rows_per_second`, `query_qps`, `read_rows`, `retention_days`, `materialized_view_count`, and `cross_cell_export_job` drive placement and quota.
- Cell placement class: Tier-2 analytics cell by default; Tier-3 regulated cell for HIPAA/KR/EU-AI pack overlays; ADR-0337 routes lakehouse writes through the Iceberg/data-warehouse path while ClickHouse remains the serving compute layer.
- Autoscaling boundaries: minimum three shards x two replicas per cell at the current target; maximum per-tenant ceiling is 10B rows per table, 10K qps fleet-wide per cell, and 100TB hot/1PB cold per cell before capacity-rebalance pages.
- Tenant load profile served: tenant dashboards, audit-log search, billing rollups, regulator export, and internal cost planning share a columnar store without one tenant's aggregate storm starving another tenant.

### Sustainability + cost attribution (ADR-0344)

- Every analytics query, dashboard read, audit-log search, billing rollup, ingest projection, regulator export, and capacity-planning job emits `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside the audit row.
- Carbon-aware provider routing: yes for bulk exports, cold-tier compaction, scheduled cross-cell aggregation, and backup jobs; no for interactive tenant dashboards, audit-log search, HIPAA-EM, PCI-realtime-fraud, or protected high-risk audit paths.
- Tenant cost transparency surface: `cost-budget.md`, dashboard burn-rate views, and the FinOps portal expose storage tier, query class, ingest rows, cell, provider, and compliance_pack.
- Regulatory driver: CSRD, SB-253, and SEC climate-disclosure reporting require per-tenant analytical workload emissions; analytics is the aggregation surface, so its own reads must not disappear from the ledger.

### API versioning posture (ADR-0342)

- Public API version model: dashboard, audit-log, billing-rollup, regulator-export, and proto contracts use the YYYY-MM-DD carrier triplet: `Oyatie-API-Version: <date>`, `/api/analytics/<date>/...`, and proto3 `api_version` fields.
- SDK semver model: generated client SDKs publish `major.minor.patch`; semver major only follows breaking changes to supported date-versioned contracts.
- Support window: last N=3 public contract dates are supported for at least 180 days.
- Per-tenant pinning: yes for embedded dashboards, audit exports, and billing integrations.
- Internal-mesh exemption: yes; direct gRPC from application and billing services preserves ADR-0145 while tenant-facing contract carriers remain date-versioned.

## 9. Open questions

1. Does the tenant-facing dashboard surface live in the application µservice or in a new analytics-facing UI µservice? — Default: application µservice consumes via REST/gRPC; new UI surface deferred.
2. Cross-cell federation for global ops aggregates — performance vs simplicity? — Default: per-cell rollups + scheduled cross-cell aggregation jobs; live federation deferred.

## 10. Phase plan

- **PHASE-01: ANALYTICS-OLAP-BOOTSTRAP.** Stands up the cluster, the per-tenant bootstrap controller, the outbox→ClickHouse CDC pipeline, and the first three tenant dashboards (workflow execution metrics; audit-log search; billing rollup). 15 IPs. (PHASE-01 spec at `microservices/analytics/PHASE-01-ANALYTICS-OLAP-BOOTSTRAP.md`.)

## 11. References

See ADR list at the top.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-foundry-fitness-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `analytics` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `analytics` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 2 module pin(s) across 1 context(s).
- Scaling input: `per_query` with cell placement `Tier-3` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
