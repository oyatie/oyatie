---
doc_class: Threat-Model
doc_id: TM-CELL-REBALANCER
microservice: cell-rebalancer
status: wave-15-zd-scaffold
date: 2026-05-21
owner_team: axis-platform-reliability + axis-tenancy + axis-governance
bounded_context: tenant-migration-across-cells
implementation_phase: documentation-and-contracts-only
rust_code_status: not-authored-in-this-wave
source_adr: ADR-0276
---

# STRIDE Threat Model: cell-rebalancer

## Assets
- tenant assignment epochs
- migration workflow state
- Cedar decisions
- audit-chain seals
- residency and compliance snapshots
- source and target cell ids
- operator and Foundry principals
- PostgreSQL and Valkey checkpoints
- api-gateway routing cutover records
- FinOps emission dimensions

## Spoofing
- Spoofing risk 01: cloud-iac interaction can trigger cedar-deny; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Spoofing risk 02: observability interaction can trigger source-quiesce-timeout; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Spoofing risk 03: audit-chain interaction can trigger transfer-lag-exceeded; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Spoofing risk 04: policy-engine interaction can trigger target-activation-failed; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Spoofing risk 05: api-gateway interaction can trigger audit-chain-emit-failed; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Spoofing risk 06: oya-shuffle-sharding interaction can trigger version-carrier-conflict; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Spoofing risk 07: oya-residency-domain interaction can trigger rollback-window-expired; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Spoofing risk 08: finops-portal interaction can trigger candidate-cell-ineligible; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Spoofing risk 09: tenancy interaction can trigger cedar-deny; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Spoofing risk 10: cloud-iac interaction can trigger source-quiesce-timeout; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Spoofing risk 11: observability interaction can trigger transfer-lag-exceeded; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Spoofing risk 12: audit-chain interaction can trigger target-activation-failed; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Spoofing risk 13: policy-engine interaction can trigger audit-chain-emit-failed; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Spoofing risk 14: api-gateway interaction can trigger version-carrier-conflict; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Spoofing risk 15: oya-shuffle-sharding interaction can trigger rollback-window-expired; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Spoofing risk 16: oya-residency-domain interaction can trigger candidate-cell-ineligible; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Spoofing risk 17: finops-portal interaction can trigger cedar-deny; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.

## Tampering
- Tampering risk 01: cloud-iac interaction can trigger cedar-deny; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Tampering risk 02: observability interaction can trigger source-quiesce-timeout; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Tampering risk 03: audit-chain interaction can trigger transfer-lag-exceeded; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Tampering risk 04: policy-engine interaction can trigger target-activation-failed; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Tampering risk 05: api-gateway interaction can trigger audit-chain-emit-failed; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Tampering risk 06: oya-shuffle-sharding interaction can trigger version-carrier-conflict; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Tampering risk 07: oya-residency-domain interaction can trigger rollback-window-expired; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Tampering risk 08: finops-portal interaction can trigger candidate-cell-ineligible; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Tampering risk 09: tenancy interaction can trigger cedar-deny; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Tampering risk 10: cloud-iac interaction can trigger source-quiesce-timeout; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Tampering risk 11: observability interaction can trigger transfer-lag-exceeded; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Tampering risk 12: audit-chain interaction can trigger target-activation-failed; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Tampering risk 13: policy-engine interaction can trigger audit-chain-emit-failed; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Tampering risk 14: api-gateway interaction can trigger version-carrier-conflict; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Tampering risk 15: oya-shuffle-sharding interaction can trigger rollback-window-expired; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Tampering risk 16: oya-residency-domain interaction can trigger candidate-cell-ineligible; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Tampering risk 17: finops-portal interaction can trigger cedar-deny; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.

## Repudiation
- Repudiation risk 01: cloud-iac interaction can trigger cedar-deny; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Repudiation risk 02: observability interaction can trigger source-quiesce-timeout; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Repudiation risk 03: audit-chain interaction can trigger transfer-lag-exceeded; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Repudiation risk 04: policy-engine interaction can trigger target-activation-failed; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Repudiation risk 05: api-gateway interaction can trigger audit-chain-emit-failed; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Repudiation risk 06: oya-shuffle-sharding interaction can trigger version-carrier-conflict; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Repudiation risk 07: oya-residency-domain interaction can trigger rollback-window-expired; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Repudiation risk 08: finops-portal interaction can trigger candidate-cell-ineligible; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Repudiation risk 09: tenancy interaction can trigger cedar-deny; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Repudiation risk 10: cloud-iac interaction can trigger source-quiesce-timeout; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Repudiation risk 11: observability interaction can trigger transfer-lag-exceeded; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Repudiation risk 12: audit-chain interaction can trigger target-activation-failed; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Repudiation risk 13: policy-engine interaction can trigger audit-chain-emit-failed; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Repudiation risk 14: api-gateway interaction can trigger version-carrier-conflict; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Repudiation risk 15: oya-shuffle-sharding interaction can trigger rollback-window-expired; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Repudiation risk 16: oya-residency-domain interaction can trigger candidate-cell-ineligible; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Repudiation risk 17: finops-portal interaction can trigger cedar-deny; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.

## Information Disclosure
- Information Disclosure risk 01: cloud-iac interaction can trigger cedar-deny; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Information Disclosure risk 02: observability interaction can trigger source-quiesce-timeout; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Information Disclosure risk 03: audit-chain interaction can trigger transfer-lag-exceeded; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Information Disclosure risk 04: policy-engine interaction can trigger target-activation-failed; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Information Disclosure risk 05: api-gateway interaction can trigger audit-chain-emit-failed; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Information Disclosure risk 06: oya-shuffle-sharding interaction can trigger version-carrier-conflict; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Information Disclosure risk 07: oya-residency-domain interaction can trigger rollback-window-expired; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Information Disclosure risk 08: finops-portal interaction can trigger candidate-cell-ineligible; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Information Disclosure risk 09: tenancy interaction can trigger cedar-deny; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Information Disclosure risk 10: cloud-iac interaction can trigger source-quiesce-timeout; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Information Disclosure risk 11: observability interaction can trigger transfer-lag-exceeded; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Information Disclosure risk 12: audit-chain interaction can trigger target-activation-failed; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Information Disclosure risk 13: policy-engine interaction can trigger audit-chain-emit-failed; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Information Disclosure risk 14: api-gateway interaction can trigger version-carrier-conflict; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Information Disclosure risk 15: oya-shuffle-sharding interaction can trigger rollback-window-expired; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Information Disclosure risk 16: oya-residency-domain interaction can trigger candidate-cell-ineligible; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Information Disclosure risk 17: finops-portal interaction can trigger cedar-deny; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.

## Denial of Service
- Denial of Service risk 01: cloud-iac interaction can trigger cedar-deny; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Denial of Service risk 02: observability interaction can trigger source-quiesce-timeout; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Denial of Service risk 03: audit-chain interaction can trigger transfer-lag-exceeded; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Denial of Service risk 04: policy-engine interaction can trigger target-activation-failed; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Denial of Service risk 05: api-gateway interaction can trigger audit-chain-emit-failed; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Denial of Service risk 06: oya-shuffle-sharding interaction can trigger version-carrier-conflict; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Denial of Service risk 07: oya-residency-domain interaction can trigger rollback-window-expired; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Denial of Service risk 08: finops-portal interaction can trigger candidate-cell-ineligible; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Denial of Service risk 09: tenancy interaction can trigger cedar-deny; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Denial of Service risk 10: cloud-iac interaction can trigger source-quiesce-timeout; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Denial of Service risk 11: observability interaction can trigger transfer-lag-exceeded; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Denial of Service risk 12: audit-chain interaction can trigger target-activation-failed; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Denial of Service risk 13: policy-engine interaction can trigger audit-chain-emit-failed; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Denial of Service risk 14: api-gateway interaction can trigger version-carrier-conflict; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Denial of Service risk 15: oya-shuffle-sharding interaction can trigger rollback-window-expired; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Denial of Service risk 16: oya-residency-domain interaction can trigger candidate-cell-ineligible; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Denial of Service risk 17: finops-portal interaction can trigger cedar-deny; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.

## Elevation of Privilege
- Elevation of Privilege risk 01: cloud-iac interaction can trigger cedar-deny; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Elevation of Privilege risk 02: observability interaction can trigger source-quiesce-timeout; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Elevation of Privilege risk 03: audit-chain interaction can trigger transfer-lag-exceeded; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Elevation of Privilege risk 04: policy-engine interaction can trigger target-activation-failed; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Elevation of Privilege risk 05: api-gateway interaction can trigger audit-chain-emit-failed; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Elevation of Privilege risk 06: oya-shuffle-sharding interaction can trigger version-carrier-conflict; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Elevation of Privilege risk 07: oya-residency-domain interaction can trigger rollback-window-expired; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Elevation of Privilege risk 08: finops-portal interaction can trigger candidate-cell-ineligible; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Elevation of Privilege risk 09: tenancy interaction can trigger cedar-deny; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Elevation of Privilege risk 10: cloud-iac interaction can trigger source-quiesce-timeout; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Elevation of Privilege risk 11: observability interaction can trigger transfer-lag-exceeded; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Elevation of Privilege risk 12: audit-chain interaction can trigger target-activation-failed; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Elevation of Privilege risk 13: policy-engine interaction can trigger audit-chain-emit-failed; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Elevation of Privilege risk 14: api-gateway interaction can trigger version-carrier-conflict; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Elevation of Privilege risk 15: oya-shuffle-sharding interaction can trigger rollback-window-expired; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Elevation of Privilege risk 16: oya-residency-domain interaction can trigger candidate-cell-ineligible; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.
- Elevation of Privilege risk 17: finops-portal interaction can trigger cedar-deny; mitigation is mTLS identity, Cedar pre-evaluation, persisted state versioning, and audit-chain evidence before success.

## Residual Risks
- Residual 01: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 02: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 03: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 04: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 05: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 06: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 07: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 08: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 09: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 10: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 11: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 12: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 13: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 14: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 15: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 16: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 17: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 18: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 19: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 20: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 21: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 22: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 23: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 24: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 25: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 26: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 27: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 28: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 29: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.
- Residual 30: downstream Rust implementation must prove race-free cutover and rollback under concurrent jobs with shared source or target cells.

## §autosharding-event-drift

Source ADRs: ADR-0346, ADR-0347, ADR-0348, ADR-0349.

Threat: autosharding, auto-rebalance, or dynamic sharding automation events can drift from the manifest-declared `sharding_automation` contract, causing a tenant to move to the wrong cell or shard, bypass residency/compliance filters, or leave no reversible audit trail. The threat covers spoofed control-plane principals, tampered threshold inputs, missing audit-chain rows, stale routing cutovers, denial amplification during hot-split/cold-merge, and privilege escalation through unauthorized cross-jurisdiction migration.

Required controls:
- ADR-0348: `oya-governance-sharding-automation-coverage` refuses any microservice manifest without complete autosharding, auto_rebalance, and dynamic_sharding sub-block declarations.
- ADR-0348: `oya-governance-autosharding-manual-mode-refusal` refuses `manual`; the canonical autosharding mode is `control_plane_driven`.
- ADR-0348: `oya-governance-auto-rebalance-residency-honored` requires auto-rebalance to honor residency and compliance packs; cross-jurisdiction migration requires an explicit Cedar permit per ADR-0243.
- ADR-0348: `oya-governance-dynamic-sharding-threshold-coverage` requires explicit hot-split and cold-merge thresholds; default-fill is rejected.
- ADR-0348: `oya-governance-audit-chain-emit-on-automation-events` requires every auto-rebalance, hot-split, and cold-merge event to emit per ADR-0263; `oya-governance-tenant-migration-reversibility` requires a rollback path.
- ADR-0346: `./bin/oya verify --ci-required` is the canonical local pre-push verifier and must mirror cargo fmt, cargo check, cargo clippy, cargo nextest, and `oya gate run-all` before returning success.
- ADR-0347: governance-owned checks use the `oya-governance-*` prefix; threat-model evidence must cite the governance lane names above without reintroducing stale lane vocabulary.
- ADR-0349: Jenkins/GitHub Actions parity and ArgoCD cosign/audit-chain lanes preserve the same controls in self-hostable CI/CD contexts.

Evidence required: every accepted automation event records event_type, tenant_id when tenant-level, cell_id, shard_id when shard-level, pre_state, post_state, residency_check_result, compliance_pack_check_result, cedar_permit_id when applicable, and initiated_by `control_plane:cell-orchestrator` in the audit-chain row. Residual risk remains until Wave 15-ZD proves race-free cutover and rollback under concurrent auto-rebalance, hot-split, and cold-merge jobs.
