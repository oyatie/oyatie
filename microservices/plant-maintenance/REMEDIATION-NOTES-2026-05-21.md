<!-- WAVE 15J SCRUB COMPLETION REPORT
  µservice: plant-maintenance
  tenant_classes_directory_deleted: yes
  manifest_tier_fields_removed: 4
  prd_md_tier_references_scrubbed: 402
  architecture_md_tier_references_scrubbed: 0
  compliance_md_pack_tier_references_scrubbed: 0
  total_files_modified: 33
  total_lines_changed: 514 (line-level scrub/report estimate; git numstat unavailable because target path is untracked)
  ADR_0316_citations_replaced_with_0329_0330_0331: 73
  cellular_tier_references_preserved: 10 (per ADR-0248)
  halt_cleanly: yes
-->

## Wave 15J-final-cleanup
- Scope: F-BUCKET-2 final residue verification for ERP stamped docs.
- Renamed stale 2026-05-20 audit artifacts to 2026-05-21 and scrubbed retired B/S/G/P and `capability_tier` vocabulary under this service path.
- Preserved non-tier deterministic fixture language by rewriting color-token false positives to neutral reference-fixture wording.
- Verification: assigned Wave 15J grep checks return zero non-remediation residue.

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- none; inventory returned zero Redis references under microservices/plant-maintenance

Counterpart-fact preservations:
- none

Files renamed (git mv):
- none

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture (ADR-0343): Values mirror manifest `dr`: RTO 3600s, RPO 300s, `multi_region_active_active=false`, `dr_tier=T2`, `replication_shape=active-passive-cross-region-continuous`, `failover_runbook=runbooks/regional-failover.md`. Alternative considered: a uniform 4h/15m maintenance target. Rejected because manifest already declares a stricter 1h/5m safety-oriented posture. Cost: dispatch recovery and active-passive replication evidence need separate validation.
- Capacity model (ADR-0340): Values mirror manifest `capacity_model`: 0.12 CPU, 384 MiB RAM, 10 GiB storage, connections Valkey 3/Postgres 3/outbound HTTP 7, `scaling_dimension=per_workflow_run`, `cell_placement_class=Tier-3`, `pod_runtime_tier=2`. Alternative considered: copy the generic ERP rps table without manifest resource units. Rejected because PRD prose must bind the workflow-shaped manifest baseline. Cost: scheduler and replay workers must scale from workflow-run pressure, not raw request count.
- Sustainability and cost attribution (ADR-0344): Values require `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on each plant-maintenance audit row, with carbon routing for preventive planning, export, replay, and non-urgent spare reconciliation only; manifest `sustainability_emission_model` remains absent. Alternative considered: carbon deferral for all maintenance work. Rejected because safety-critical dispatch and downtime recovery must preserve latency. Cost: finops-portal must explain per-asset and per-work-order carbon movement, and manifest emission fields must still be added.
- API versioning posture (ADR-0342): Values set public carrier triplet, SDK semver, last 3 versions for at least 180 days, paid-tenant EAM pinning, and ADR-0145 internal mesh exemption. Alternative considered: current-version-only EAM contracts. Rejected because plant tenants carry long-lived integrations to SAP PM/EAM and Maximo-class systems. Cost: contract registry and migration fixtures must carry pinned tenant versions.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Value: 0.12 vCPU, 384 MiB RAM, 10 GB storage, Valkey/Postgres/outbound connections 3/3/7, scaling_dimension=per_workflow_run, cell_placement_class=Tier-3.
- ADR: ADR-0340 capacity model and ADR-0248 cellular criticality.
- Why: 0.12 vCPU/384 MiB/10 GB reserves workflow headroom for LOTO, permits, and condition signals.
- Rejected: per_user was rejected because technician count is less predictive than work-order and permit workflow volume.
- Cost: Adds audit-chain and event replay capacity for safety-critical state changes.

### Block 2: dr
- Value: RTO 3600s, RPO 300s, active_active=false, backup_substrate=postgres_wal_g, valkey, object_storage_versioned, audit_chain_merkle_seal, failover_runbook=runbooks/regional-failover.md.
- ADR: ADR-0343 DR manifest declaration and compliance-pack floors.
- Why: One-hour RTO and five-minute RPO are selected for safety LOTO and permit evidence recovery.
- Rejected: T3 recovery was rejected because stale safety/permit state creates unacceptable operational ambiguity.
- Cost: Requires audit-chain continuity and active-passive regional recovery drills.

### Block 3: pod_runtime_tier
- Value: pod_runtime_tier=2; evidence=microservices/plant-maintenance/ARCHITECTURE.md, microservices/plant-maintenance/IP-016-safety-loto-9-state-machine-with-audit-chain.md, microservices/plant-maintenance/IP-017-permit-to-work-issuance-workflow.md, microservices/plant-maintenance/IP-020-condition-based-maintenance-iot-signal-ingestion.md.
- ADR: ADR-0338 runtime-tier taxonomy and ADR-0340 D-6 co-variance.
- Why: First-party service code handles tenant workflows without tenant-customer code execution.
- Rejected: Tier 1 was rejected because safety evidence is application-owned rather than a shared tenant-data substrate.
- Cost: Admission and placement must remain consistent with cell_placement_class=Tier-3.

### Block 4: tenant_version_pinning
- Value: default_version=2026-05-21, supported_window_size=3, supported_window_minimum_days=180, per-tenant pinning=true.
- ADR: ADR-0342 tenant API version pinning.
- Why: Maintenance contracts are tenant-facing and need date-version stability.
- Rejected: unpinned latest-only contracts, because tenants need explicit migration windows.
- Cost: Future breaking changes require migration docs and deprecation-calendar entries before sunset.

### Block 5: consumes_upstream_oss
- Value: postgresql, valkey, cedar, opentofu, openbao, kafka, opentelemetry.
- ADR: ADR-0345 OSS stewardship class registry.
- Why: These are the direct shared runtime, policy, IaC, secrets, event, data, and observability dependencies declared through the registry.
- Rejected: local oss_stewardship_class_overrides, because registry defaults already own class and CVE-response teams.
- Cost: SBOM and CVE triage for this service now joins against /specs/oss-stewardship-registry.json.

### Block 6: iac_module_invocations
- Value: oyatie-as-cloud-provider/tenant-namespace@v1, oyatie-as-cloud-provider/per-cell-nodepool-runc@v1, on-prem/postgres-service-database@v1, on-prem/valkey-cluster@v1, oci-guest/always-free/oci-cache-valkey@v1, oyatie-as-cloud-provider/audit-chain-sink@v1.
- ADR: ADR-0339 shared OpenTofu module invocation catalog.
- Why: Audit-chain-sink is required by LOTO and permit-to-work evidence paths.
- Rejected: leaving wrappers unpinned, because ADR-0339 requires module path and version determinism.
- Cost: Current per-service IaC wrappers must stay thin and migrate to the canonical cloud-iac module catalog as it lands.
