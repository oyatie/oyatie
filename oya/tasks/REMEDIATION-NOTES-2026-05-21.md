<!-- WAVE 15J SCRUB COMPLETION REPORT
  µservice: tasks
  capability_tiers_directory_deleted: yes
  manifest_tier_fields_removed: 5
  prd_md_tier_references_scrubbed: 2
  architecture_md_tier_references_scrubbed: 16
  compliance_md_pack_tier_references_scrubbed: 0
  total_files_modified: 26
  total_lines_changed: 687 scrub-local estimate
  ADR_0316_citations_replaced_with_0329_0330_0331: 5
  cellular_tier_references_preserved: 10 (per ADR-0248)
  halt_cleanly: yes
-->

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- `microservices/tasks/AUDIT-FINDINGS-2026-05-18.json`
- `microservices/tasks/IP-001-iac-bootstrap.md`
- `microservices/tasks/IP-002-cargo-workspace-bootstrap.md`
- `microservices/tasks/IP-010-view-engine-and-board-realtime.md`
- `microservices/tasks/PHASE-01-TASKS-FOUNDATION.md`
- `microservices/tasks/PRD.md`
- `microservices/tasks/catalog/oya-tasks-view-engine-adapter-valkey.yaml`
- `microservices/tasks/decisions/ADR-TASKS-0004-view-engine-and-board-realtime.md`
- `microservices/tasks/decisions/ADR-TSK-001-priority-queue-architecture-with-fairness-guarantees.md`
- `microservices/tasks/iac/helm/tasks/templates/networkpolicy.yaml`
- `microservices/tasks/iac/helm/tasks/values.yaml`
- `microservices/tasks/manifest.json`
- `microservices/tasks/migration-from-connect.md`
- `microservices/tasks/threat-model.md`

Counterpart-fact preservations:
- none

Files renamed (git mv):
- `microservices/tasks/catalog/oya-tasks-view-engine-adapter-redis.yaml` -> `microservices/tasks/catalog/oya-tasks-view-engine-adapter-valkey.yaml`


## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- baseline_cpu_per_tenant: 0.22 vCPU; baseline_ram_per_tenant: 512 MiB; storage_per_tenant: 10 GB.
- connections_per_tenant: valkey=2, postgres=4, outbound_http=8.
- scaling_dimension: per_user; cell_placement_class: Tier-3.
- ADR: ADR-0340 capacity-model doctrine plus ADR-0248 cell criticality numbering.
- Why: 0.22 vCPU / 512 MiB / 10 GB fits collaborative task/project state, recurrence materialization, and search-index updates per active tenant user.
- Rejected: per_request was rejected because recurrence and dependency graph work are user/project-state shaped rather than simple request throughput.
- Cost: Tier-3 placement keeps productivity workloads in app cells while preserving stable tenant data recovery targets.

### Block 2: dr
- rto_p99_seconds: 900; rpo_p99_seconds: 60; multi_region_active_active: false.
- backup_substrate: postgres_wal_g, valkey, object_storage_versioned; failover_runbook: runbooks/custom-field-schema-migration.md; replication_shape: active-passive-cross-region-continuous.
- ADR: ADR-0343 recoverability doctrine and compliance-pack floors.
- Why: RTO 900s / RPO 60s follows documented task recovery posture so schema migrations and search rebuilds do not lose task changes.
- Rejected: RPO 300s was rejected because task updates are operational records, not disposable cache state.
- Cost: Recovery SLOs now require drill evidence that proves the declared substrate set, not only service process restart.

### Block 3: pod_runtime_tier
- pod_runtime_tier: 2; evidence: microservices/tasks/PRD.md, microservices/tasks/ARCHITECTURE.md, microservices/tasks/IP-003-task-store-kernel-domain.md, microservices/tasks/contracts/openapi/tasks.yaml.
- ADR: ADR-0338 pod runtime tier doctrine and ADR-0340 D-6 cell/runtime co-variance.
- Why: Tasks is a first-party tenant productivity application; it stores tenant task data but does not execute tenant code or own substrate key/audit responsibilities.
- Rejected: Tier 1 was rejected because Tasks consumes tenant data but does not own substrate-level data-plane controls.
- Cost: Admission, scheduling, and isolation tests must preserve this tier when runtime surfaces move.

### Block 4: tenant_version_pinning
- declared_versions: 2025-11-21, 2026-02-21, 2026-05-21; default_version: 2026-05-21.
- supported_window_size: 3; supported_window_minimum_days: 180; supports_per_tenant_pinning: true.
- ADR: ADR-0342 tenant version pinning doctrine.
- Why: Public contracts are tenant-visible and must remain selectable across the minimum support window.
- Rejected: unpinned task APIs were rejected because tenant workflows and integrations depend on stable task contract versions.
- Cost: Release work must carry compatibility tests and deprecation-calendar updates before any breaking contract change.

### Block 5: consumes_upstream_oss
- consumes_upstream_oss: postgresql, valkey, cedar, openbao, opentofu.
- oss_stewardship_class_overrides: none; registry defaults in specs/oss-stewardship-registry.json remain authoritative.
- ADR: ADR-0345 OSS stewardship doctrine.
- Why: Postgres, Valkey, Cedar, OpenBao, and OpenTofu cover task persistence, cache/leases, policy, secret references, and IaC.
- Rejected: service-local stewardship classes without registry backing.
- Cost: CVE response ownership must follow the registry/default ownership for every declared upstream.

### Block 6: iac_module_invocations
- iac_module_invocations: oci-guest/k8s-namespace-bootstrap@v1, oci-guest/secrets-bootstrap@v1.
- ADR: ADR-0339 shared IaC module doctrine.
- Why: Namespace and secret modules are enough for the service; search rebuild dependencies remain internal to the task app layer.
- Rejected: declaring DNS was rejected because the assigned manifests do not show Tasks as the shared public edge endpoint.
- Cost: Cloud primitive changes now flow through shared module pins instead of service-local drift.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: RTO 900s/RPO 60s, active-active false, runbook `runbooks/custom-field-schema-migration.md`, ADR-0343. Alternative considered: generic HIPAA floor 3600s/300s; rejected because task writes and legal holds already require tighter recovery. Cost: warm active-passive DR substrate and quarterly restore evidence.
- Capacity model: 0.22 vCPU, 512 MiB RAM, 10 GB storage, Postgres 4, Valkey 2, outbound 8, `per_user`, Tier-3, ADR-0340/ADR-0341. Alternative considered: per-request sizing from board traffic; rejected because task/project footprint drives durable pressure. Cost: reserved cell capacity for daily collaboration and import/search bursts.
- Sustainability + cost attribution: task audit rows emit `cost_usd_minor_units`, `co2_grams`, and `watt_hours`, ADR-0344. Alternative considered: carbon routing for every task call; rejected for interactive writes, dependency-cycle checks, legal hold, and policy-denied paths. Cost: extra audit dimensions and FinOps rollups.
- API versioning posture: date carrier triplet plus SDK semver, last 3 versions for 180 days, tenant pinning enabled, ADR-0342. Alternative considered: path-only `v1`; rejected because tenant integrations need pinned migration windows. Cost: compatibility tests across three live contract dates.
