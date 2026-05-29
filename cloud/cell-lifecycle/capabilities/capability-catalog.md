# Capability Catalog: cell-lifecycle

| Capability | Tier | Risk | Description | Evidence |
| --- | --- | --- | --- | --- |
| register-cell | T1 | limited | Register logical cell after cloud-iac readiness. | cloud_iac_receipt_id + audit_chain_event_id |
| activate-cell | T1 | limited | Activate registered cell after telemetry bootstrap. | observability_window_id + evidence_pack_id |
| promote-cell | T0 | high | Promote through ADR-0204 tiers with ADR-0266 gates. | gate_snapshot_sha256 + Cedar permit |
| drain-cell | T0 | high | Enter Draining and trigger cell-rebalancer. | drain evidence pack + rebalancer receipt |
| decommission-cell | T0 | high | Finalize Draining cell after resident_count zero. | tenancy snapshot + audit-chain seal |
| list-cells | T2 | minimal | List filtered lifecycle summaries for operators. | Cedar ReadLifecycle decision |
| read-lifecycle-history | T1 | limited | Return immutable history references. | audit-chain event references |

## Wave 15-ZF-16 Sharding Automation Capability Rows

Doctrine refs: ADR-0346, ADR-0347, ADR-0348, ADR-0349.

| Capability | Tier | Risk | Description | Evidence |
| --- | --- | --- | --- | --- |
| autosharding.autosharded | T2 | high | AUTOSHARDING — tenant→cell/shard placement is computed by the control plane automatically, with no human operator picking placement; inputs are capacity_model (ADR-0340) + compliance_pack constraints (ADR-0251) + ResidencyClass + cell_placement_class (Tier 0..4 per ADR-0248) + the shuffle-sharding algorithm in the oya-shuffle-sharding crate (ADR-0333). | ADR-0346 `oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix; ADR-0347 `oya-governance-lane-prefix-vocabulary`; ADR-0348 `oya-governance-sharding-automation-coverage` + `oya-governance-autosharding-manual-mode-refusal`; ADR-0349 `oya-governance-jenkins-github-actions-parity`. |
| autosharding.auto_rebalance | T2 | critical | AUTO-REBALANCE — when cell load skews beyond promotion-gate criteria, the cell-orchestrator automatically migrates tenants from hot cells to cooler cells; tenant migration honors residency + compliance pack constraints; cross-jurisdiction migration requires an explicit Cedar permit per ADR-0243; migration is observable, reversible, and audit-chain-emit per ADR-0263. | ADR-0346 `oya-governance-oya-verify-ci-step-exit-semantics`; ADR-0347 `oya-governance-lane-prefix-vocabulary`; ADR-0348 `oya-governance-auto-rebalance-residency-honored` + `oya-governance-audit-chain-emit-on-automation-events` + `oya-governance-tenant-migration-reversibility`; ADR-0349 `oya-governance-deploy-audit-chain-emit`. |
| autosharding.dynamic_sharding | T2 | critical | DYNAMIC SHARDING — shard count within a cell adjusts based on load: HOT-SPLIT when shard p99 latency exceeds SLO OR capacity utilization exceeds 80% (defaults; per-microservice override), COLD-MERGE when adjacent shards both run below 20% utilization for more than 24 hours (defaults; per-microservice override); both operations are atomic + audit-emit. | ADR-0346 `oya-governance-oya-verify-ci-mirror-coverage`; ADR-0347 `oya-governance-lane-prefix-vocabulary`; ADR-0348 `oya-governance-dynamic-sharding-threshold-coverage` + `oya-governance-audit-chain-emit-on-automation-events`; ADR-0349 `oya-governance-jenkins-github-actions-parity`. |
