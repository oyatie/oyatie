<!-- WAVE 15J SCRUB COMPLETION REPORT
  µservice: analytics
  capability_tiers_directory_deleted: yes
  manifest_tier_fields_removed: 2
  prd_md_tier_references_scrubbed: 0
  architecture_md_tier_references_scrubbed: 1
  compliance_md_pack_tier_references_scrubbed: 0
  total_files_modified: 28
  total_lines_changed: 262
  ADR_0316_citations_replaced_with_0329_0330_0331: 2
  cellular_tier_references_preserved: 12 (per ADR-0248)
  halt_cleanly: yes
-->

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- `microservices/analytics/specs/IP-014-self-slo-burn-rate-alerts.md`

Counterpart-fact preservations:
- none

Files renamed (git mv):
- none
## Wave 15-doctrine-propagation-IPs (2026-05-21)

- Bucket: D4-BUCKET-5.
- Agent: wave-d-d4-bucket-5-codex.
- Scope: trigger-based doctrine propagation only; unmatched IPs were left unchanged.
- IPs scanned: 25.
- Trigger A matched: 4.
- Trigger B matched: 18.
- Trigger C matched: 25.
- Trigger D matched: 0.
- IPs unmatched: 0.

### IP changes
- `microservices/analytics/IP-journey-j100-pack-rollout-first-action.md` — added Sustainability emission.
- `microservices/analytics/IP-journey-j91-us-msb-mtl-overlay.md` — added DR posture, Sustainability emission.
- `microservices/analytics/IP-journey-j92-br-lgpd-us-parent-dsar.md` — added Sustainability emission.
- `microservices/analytics/IP-journey-j93-in-dpdpa-rbi-overlay.md` — added DR posture, Sustainability emission.
- `microservices/analytics/IP-journey-j94-sox404-public-company-controls.md` — added DR posture, Sustainability emission.
- `microservices/analytics/IP-journey-j95-iso27001-soc2-annual-audit.md` — added Sustainability emission.
- `microservices/analytics/IP-journey-j96-ksa-uae-mena-onboarding.md` — added Sustainability emission.
- `microservices/analytics/IP-journey-j97-sg-pdpa-mas-tenant.md` — added Sustainability emission.
- `microservices/analytics/IP-journey-j98-au-privacy-apra-cps234.md` — added Sustainability emission.
- `microservices/analytics/IP-journey-j99-multi-pack-conflict-resolution.md` — added Sustainability emission.
- `microservices/analytics/specs/IP-001-clickhouse-cluster-iac.md` — added DR posture, Sustainability emission.
- `microservices/analytics/specs/IP-002-per-tenant-database-bootstrap.md` — added DR posture, Sustainability emission.
- `microservices/analytics/specs/IP-003-olap-client-adapter-scaffold.md` — added DR posture, Sustainability emission.
- `microservices/analytics/specs/IP-004-outbox-cdc-ingest-pipeline.md` — added DR posture, Sustainability emission.
- `microservices/analytics/specs/IP-005-materialized-view-canon.md` — added DR posture, Sustainability emission.
- `microservices/analytics/specs/IP-006-cold-tier-s3-ttl.md` — added DR posture, Sustainability emission.
- `microservices/analytics/specs/IP-007-tenant-dashboard-api.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/analytics/specs/IP-008-audit-log-query-api.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/analytics/specs/IP-009-billing-rollup-pipeline.md` — added DR posture, Sustainability emission.
- `microservices/analytics/specs/IP-010-cross-cell-federation.md` — added DR posture, Sustainability emission.
- `microservices/analytics/specs/IP-011-per-tenant-quota-enforcement.md` — added DR posture, Sustainability emission.
- `microservices/analytics/specs/IP-012-backup-restore-drill.md` — added DR posture, Sustainability emission.
- `microservices/analytics/specs/IP-013-regulator-export-evidence-pack.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/analytics/specs/IP-014-self-slo-burn-rate-alerts.md` — added DR posture, Sustainability emission.
- `microservices/analytics/specs/IP-015-app-composition-root.md` — added API Versioning, DR posture, Sustainability emission.

### Follow-up
- `microservices/analytics/manifest.json#dr` is absent; DR sections use `specs/compliance-pack-floors.json` floors and must be reconciled when the D-2 manifest DR block lands.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: set default analytics to 14400s/900s under ADR-0343 while documenting 3600s/300s for protected HIPAA/KR-RRN tables. Alternative considered: promote the old IP-012 24h RPO backup language into the PRD; rejected because SOC2 and protected packs require tighter data-loss windows. Cost: restore drills and protected-table replication require more evidence than nightly backup cadence alone.
- Capacity model: expressed ADR-0340 as one tenant database, 100GiB hot/1TiB cold baseline, 10 query connections, 10B rows/table ceiling, 10K qps per cell, and 100TB hot/1PB cold cell ceiling. Alternative considered: only cite fleet ClickHouse node count; rejected because tenant admission must be enforceable. Cost: quota enforcement and capacity-rebalance must track tenant rows, query rate, retention, and materialized views.
- Sustainability + cost attribution: required ADR-0344 fields on analytics reads, ingest projections, rollups, exports, and backup jobs. Alternative considered: count analytics only as a sink for other services' emissions; rejected because the analytics read/export itself consumes energy and must appear in regulator exports. Cost: dashboards and audit-log queries carry FinOps dimensions in addition to normal audit evidence.
- API versioning posture: added ADR-0342 date carrier triplet and SDK semver for dashboard, audit-log, billing, and regulator-export clients. Alternative considered: leave application callers on implicit internal contracts; rejected because embedded tenant dashboards and exports are public-facing. Cost: contract compatibility across three versions for at least 180 days.


## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- baseline_cpu_per_tenant: 0.35 vCPU; baseline_ram_per_tenant: 1024 MiB; storage_per_tenant: 40 GB.
- connections_per_tenant: valkey=2, postgres=4, outbound_http=6.
- scaling_dimension: per_query; cell_placement_class: Tier-3.
- ADR: ADR-0340 capacity-model doctrine plus ADR-0248 cell criticality numbering.
- Why: 0.35 vCPU / 1024 MiB / 40 GB fits query-heavy dashboards, rollups, and export staging without warehouse-sized table ownership.
- Rejected: per_user was rejected because one user can launch expensive analytic queries and exports.
- Cost: Tier-3 placement keeps analytic apps out of substrate cells while retaining enough capacity for regulated dashboard workloads.

### Block 2: dr
- rto_p99_seconds: 3600; rpo_p99_seconds: 300; multi_region_active_active: true.
- backup_substrate: clickhouse_iceberg_layered, postgres_wal_g, object_storage_versioned, valkey; failover_runbook: runbooks/restore-drill.md; replication_shape: active-active-multi-az-cross-region.
- ADR: ADR-0343 recoverability doctrine and compliance-pack floors.
- Why: RTO 3600s / RPO 300s honors HIPAA-class data availability while accepting analytic rebuild time from warehouse/source snapshots.
- Rejected: RPO 3600s was rejected because audit-log search and exported reporting would lose too much regulated state.
- Cost: Recovery SLOs now require drill evidence that proves the declared substrate set, not only service process restart.

### Block 3: pod_runtime_tier
- pod_runtime_tier: 2; evidence: microservices/analytics/PRD.md, microservices/analytics/ARCHITECTURE.md, microservices/analytics/contracts/openapi-v1.yaml, microservices/analytics/specs/IP-007-tenant-dashboard-api.md.
- ADR: ADR-0338 pod runtime tier doctrine and ADR-0340 D-6 cell/runtime co-variance.
- Why: Analytics is a tenant-facing first-party analytic application over governed datasets; it does not execute tenant code or own key/audit-chain substrate duties.
- Rejected: Tier 1 was rejected because analytics reads governed tenant data but does not own the substrate transport, key, or audit chain.
- Cost: Admission, scheduling, and isolation tests must preserve this tier when runtime surfaces move.

### Block 4: tenant_version_pinning
- declared_versions: 2025-11-21, 2026-02-21, 2026-05-21; default_version: 2026-05-21.
- supported_window_size: 3; supported_window_minimum_days: 180; supports_per_tenant_pinning: true.
- ADR: ADR-0342 tenant version pinning doctrine.
- Why: Public contracts are tenant-visible and must remain selectable across the minimum support window.
- Rejected: single-version analytic APIs were rejected because dashboard and export consumers need stable tenant-pinned contracts.
- Cost: Release work must carry compatibility tests and deprecation-calendar updates before any breaking contract change.

### Block 5: consumes_upstream_oss
- consumes_upstream_oss: clickhouse, postgresql, valkey, kafka, cedar, openbao, opentofu.
- oss_stewardship_class_overrides: none; registry defaults in specs/oss-stewardship-registry.json remain authoritative.
- ADR: ADR-0345 OSS stewardship doctrine.
- Why: ClickHouse is the serving engine, with Postgres, Valkey, Kafka, Cedar, OpenBao, and OpenTofu backing metadata, cache/events, policy, secrets, and IaC.
- Rejected: service-local stewardship classes without registry backing.
- Cost: CVE response ownership must follow the registry/default ownership for every declared upstream.

### Block 6: iac_module_invocations
- iac_module_invocations: oci-guest/k8s-namespace-bootstrap@v1, oci-guest/secrets-bootstrap@v1.
- ADR: ADR-0339 shared IaC module doctrine.
- Why: Namespace and secret module declarations are enough for the app layer; warehouse primitives stay owned by data-warehouse.
- Rejected: declaring warehouse KMS modules here was rejected because analytics consumes rather than owns encrypted lake state.
- Cost: Cloud primitive changes now flow through shared module pins instead of service-local drift.
