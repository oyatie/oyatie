---
doc_class: Capability-Catalog
doc_id: CAP-CLOUD-K8S
microservice: cloud-k8s
status: wave-15-zf-16-doctrine-propagation
date: 2026-05-21
owner_team: axis-cloud-k8s
source_adrs:
  - ADR-0346
  - ADR-0347
  - ADR-0348
  - ADR-0349
---

# Capability Catalog: cloud-k8s

## Wave 15-ZF-16 Sharding Automation Capability Rows

Doctrine refs: ADR-0346, ADR-0347, ADR-0348, ADR-0349.

| Capability | Tier | Risk | Description | Evidence |
| --- | --- | --- | --- | --- |
| autosharding.autosharded | T2 | high | AUTOSHARDING — tenant→cell/shard placement is computed by the control plane automatically, with no human operator picking placement; inputs are capacity_model (ADR-0340) + compliance_pack constraints (ADR-0251) + ResidencyClass + cell_placement_class (Tier 0..4 per ADR-0248) + the shuffle-sharding algorithm in the oya-shuffle-sharding crate (ADR-0333). | ADR-0346 legacy/local-feedback provenance after ADR-0515: `oya verify` / `./bin/oya verify --ci-required` output is optional local evidence only; protected merge authority is `oya-ci-required`|
| autosharding.auto_rebalance | T2 | critical | AUTO-REBALANCE — when cell load skews beyond promotion-gate criteria, the cell-orchestrator automatically migrates tenants from hot cells to cooler cells; tenant migration honors residency + compliance pack constraints; cross-jurisdiction migration requires an explicit Cedar permit per ADR-0243; migration is observable, reversible, and audit-chain-emit per ADR-0263. | ADR-0346 `oya-governance-oya-verify-ci-step-exit-semantics`; ADR-0347 `oya-governance-lane-prefix-vocabulary`; ADR-0348 `oya-governance-auto-rebalance-residency-honored` + `oya-governance-audit-chain-emit-on-automation-events` + `oya-governance-tenant-migration-reversibility`; ADR-0349 `oya-governance-deploy-audit-chain-emit`. |
| autosharding.dynamic_sharding | T2 | critical | DYNAMIC SHARDING — shard count within a cell adjusts based on load: HOT-SPLIT when shard p99 latency exceeds SLO OR capacity utilization exceeds 80% (defaults; per-microservice override), COLD-MERGE when adjacent shards both run below 20% utilization for more than 24 hours (defaults; per-microservice override); both operations are atomic + audit-emit. | ADR-0346 `oya-governance-oya-verify-ci-mirror-coverage`; ADR-0347 `oya-governance-lane-prefix-vocabulary`; ADR-0348 `oya-governance-dynamic-sharding-threshold-coverage` + `oya-governance-audit-chain-emit-on-automation-events`; ADR-0349 `oya-governance-jenkins-github-actions-parity`. |
