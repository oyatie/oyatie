---
doc_class: Capability-Catalog
doc_id: CAP-CELL-REBALANCER
microservice: cell-rebalancer
status: wave-15-zd-scaffold
date: 2026-05-21
owner_team: axis-platform-reliability + axis-tenancy + axis-governance
bounded_context: tenant-migration-across-cells
implementation_phase: documentation-and-contracts-only
rust_code_status: not-authored-in-this-wave
source_adr: ADR-0276
---

# Capability Catalog: cell-rebalancer

## Catalog Entries
### rebalance-job-create
- tier: T1
- risk_class: high
- description: Create a bounded RebalanceJob for eligible tenants.
- owner: axis-platform-reliability
- cedar_required: true
- audit_chain_required: true
- tenant_classes: demo_trial, paid
- compliance_packs: soc2, hipaa, gdpr, csap, pci
- evidence: API response, AsyncAPI event, audit-chain seal, OpenSLO metric
### rebalance-job-abort
- tier: T1
- risk_class: high
- description: Abort or emergency-abort an active job within blast-radius caps.
- owner: axis-platform-reliability
- cedar_required: true
- audit_chain_required: true
- tenant_classes: demo_trial, paid
- compliance_packs: soc2, hipaa, gdpr, csap, pci
- evidence: API response, AsyncAPI event, audit-chain seal, OpenSLO metric
### tenant-migration-execute
- tier: T1
- risk_class: critical
- description: Move one tenant from source cell to target cell with rollback evidence.
- owner: axis-platform-reliability
- cedar_required: true
- audit_chain_required: true
- tenant_classes: demo_trial, paid
- compliance_packs: soc2, hipaa, gdpr, csap, pci
- evidence: API response, AsyncAPI event, audit-chain seal, OpenSLO metric
### migration-history-read
- tier: T2
- risk_class: medium
- description: Expose tenant-scoped migration history.
- owner: axis-platform-reliability
- cedar_required: true
- audit_chain_required: true
- tenant_classes: demo_trial, paid
- compliance_packs: soc2, hipaa, gdpr, csap, pci
- evidence: API response, AsyncAPI event, audit-chain seal, OpenSLO metric
### compliance-pack-validate
- tier: T1
- risk_class: critical
- description: Validate target-cell pack compatibility before mutation.
- owner: axis-platform-reliability
- cedar_required: true
- audit_chain_required: true
- tenant_classes: demo_trial, paid
- compliance_packs: soc2, hipaa, gdpr, csap, pci
- evidence: API response, AsyncAPI event, audit-chain seal, OpenSLO metric
### residency-validate
- tier: T1
- risk_class: critical
- description: Validate source and target residency domains.
- owner: axis-platform-reliability
- cedar_required: true
- audit_chain_required: true
- tenant_classes: demo_trial, paid
- compliance_packs: soc2, hipaa, gdpr, csap, pci
- evidence: API response, AsyncAPI event, audit-chain seal, OpenSLO metric
### audit-evidence-emit
- tier: T0
- risk_class: critical
- description: Emit sealed audit-chain rows for every transition.
- owner: axis-platform-reliability
- cedar_required: true
- audit_chain_required: true
- tenant_classes: demo_trial, paid
- compliance_packs: soc2, hipaa, gdpr, csap, pci
- evidence: API response, AsyncAPI event, audit-chain seal, OpenSLO metric
### foundry-auto-rebalance-authorize
- tier: T1
- risk_class: high
- description: Permit bounded automation through oyatie.foundry.cell-rebalancer.
- owner: axis-platform-reliability
- cedar_required: true
- audit_chain_required: true
- tenant_classes: demo_trial, paid
- compliance_packs: soc2, hipaa, gdpr, csap, pci
- evidence: API response, AsyncAPI event, audit-chain seal, OpenSLO metric

## Capability Template Fields
- Field 01: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 02: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 03: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 04: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 05: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 06: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 07: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 08: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 09: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 10: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 11: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 12: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 13: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 14: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 15: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 16: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 17: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 18: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 19: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 20: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 21: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 22: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 23: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 24: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 25: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 26: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 27: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 28: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 29: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 30: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 31: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 32: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 33: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 34: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 35: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 36: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 37: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 38: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 39: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 40: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 41: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 42: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 43: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 44: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 45: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 46: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 47: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 48: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 49: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 50: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 51: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 52: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 53: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 54: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 55: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 56: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 57: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 58: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 59: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.
- Field 60: capability id, authority ADR, owning persona, action, resource, Cedar fragment, SLO, evidence row, rollback path, and downstream test obligation are mandatory for every cell-rebalancer capability.

## Wave 15-ZF-16 Sharding Automation Capability Rows

Doctrine refs: ADR-0346, ADR-0347, ADR-0348, ADR-0349.

| Capability | Tier | Risk | Description | Evidence |
| --- | --- | --- | --- | --- |
| autosharding.autosharded | T2 | high | AUTOSHARDING — tenant→cell/shard placement is computed by the control plane automatically, with no human operator picking placement; inputs are capacity_model (ADR-0340) + compliance_pack constraints (ADR-0251) + ResidencyClass + cell_placement_class (Tier 0..4 per ADR-0248) + the shuffle-sharding algorithm in the oya-shuffle-sharding crate (ADR-0333). | ADR-0346 legacy/local-feedback provenance after ADR-0515: `oya verify` / `./bin/oya verify --ci-required` output is optional local evidence only; protected merge authority is `oya-ci-required`|
| autosharding.auto_rebalance | T2 | critical | AUTO-REBALANCE — when cell load skews beyond promotion-gate criteria, the cell-orchestrator automatically migrates tenants from hot cells to cooler cells; tenant migration honors residency + compliance pack constraints; cross-jurisdiction migration requires an explicit Cedar permit per ADR-0243; migration is observable, reversible, and audit-chain-emit per ADR-0263. | ADR-0346 `oya-governance-oya-verify-ci-step-exit-semantics`; ADR-0347 `oya-governance-lane-prefix-vocabulary`; ADR-0348 `oya-governance-auto-rebalance-residency-honored` + `oya-governance-audit-chain-emit-on-automation-events` + `oya-governance-tenant-migration-reversibility`; ADR-0349 `oya-governance-deploy-audit-chain-emit`. |
| autosharding.dynamic_sharding | T2 | critical | DYNAMIC SHARDING — shard count within a cell adjusts based on load: HOT-SPLIT when shard p99 latency exceeds SLO OR capacity utilization exceeds 80% (defaults; per-microservice override), COLD-MERGE when adjacent shards both run below 20% utilization for more than 24 hours (defaults; per-microservice override); both operations are atomic + audit-emit. | ADR-0346 `oya-governance-oya-verify-ci-mirror-coverage`; ADR-0347 `oya-governance-lane-prefix-vocabulary`; ADR-0348 `oya-governance-dynamic-sharding-threshold-coverage` + `oya-governance-audit-chain-emit-on-automation-events`; ADR-0349 `oya-governance-jenkins-github-actions-parity`. |
