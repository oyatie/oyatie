---
doc_class: Threat-Model
doc_id: TM-CLOUD-DATA-WAVE-15-ZF-11
microservice: cloud-data
status: wave-15-zf-doctrine-propagation
date: 2026-05-21
owner_team: inherited-from-manifest
source_adrs:
  - ADR-0346
  - ADR-0347
  - ADR-0348
  - ADR-0515
---

# STRIDE Threat Model: cloud-data

## §autosharding-event-drift

Source ADRs: ADR-0346, ADR-0347, ADR-0348, ADR-0515.

Threat: autosharding, auto-rebalance, or dynamic sharding automation events can drift from the manifest-declared `sharding_automation` contract, causing a tenant to move to the wrong cell or shard, bypass residency/compliance filters, or leave no reversible audit trail. The threat covers spoofed control-plane principals, tampered threshold inputs, missing audit-chain rows, stale routing cutovers, denial amplification during hot-split/cold-merge, and privilege escalation through unauthorized cross-jurisdiction migration.

Required controls:
- ADR-0348: `oya-governance-sharding-automation-coverage` refuses any microservice manifest without complete autosharding, auto_rebalance, and dynamic_sharding sub-block declarations.
- ADR-0348: `oya-governance-autosharding-manual-mode-refusal` refuses `manual`; the canonical autosharding mode is `control_plane_driven`.
- ADR-0348: `oya-governance-auto-rebalance-residency-honored` requires auto-rebalance to honor residency and compliance packs; cross-jurisdiction migration requires an explicit Cedar permit per ADR-0243.
- ADR-0348: `oya-governance-dynamic-sharding-threshold-coverage` requires explicit hot-split and cold-merge thresholds; default-fill is rejected.
- ADR-0348: `oya-governance-audit-chain-emit-on-automation-events` requires every auto-rebalance, hot-split, and cold-merge event to emit per ADR-0263; `oya-governance-tenant-migration-reversibility` requires a rollback path.
- ADR-0346: `./bin/oya verify --ci-required` is the canonical local pre-push verifier and must mirror cargo fmt, cargo check, cargo clippy, cargo nextest, and `oya gate run-all` before returning success.
- ADR-0347: governance-owned checks use the `oya-governance-*` prefix; threat-model evidence must cite the governance lane names above without reintroducing stale lane vocabulary.
- ADR-0515: the single protected `oya-ci-required` context is merge authority; Argo CD cosign/audit-chain controls apply only where the bridge/reference adapter is separately authorized.

Evidence required: every accepted automation event records event_type, tenant_id when tenant-level, cell_id, shard_id when shard-level, pre_state, post_state, residency_check_result, compliance_pack_check_result, cedar_permit_id when applicable, and initiated_by `control_plane:cell-orchestrator` in the audit-chain row. Residual risk remains until Wave 15-ZD proves race-free cutover and rollback under concurrent auto-rebalance, hot-split, and cold-merge jobs.
