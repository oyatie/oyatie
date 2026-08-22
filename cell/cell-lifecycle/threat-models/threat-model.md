# STRIDE Threat Model: cell-lifecycle

Scope: logical Cell state machine, lifecycle API, Postgres registry/history, Valkey hot lookup, Cedar permits, audit-chain evidence, and dependency receipts.

## Spoofing
- Threat 1: fake Foundry principal.
  Mitigation: require Cedar decision, idempotency key, HLC ordering, audit-chain seal, least-privilege action, and dependency receipt validation for fake Foundry principal.
- Threat 2: forged operator identity.
  Mitigation: require Cedar decision, idempotency key, HLC ordering, audit-chain seal, least-privilege action, and dependency receipt validation for forged operator identity.
- Threat 3: replayed Cedar decision token.
  Mitigation: require Cedar decision, idempotency key, HLC ordering, audit-chain seal, least-privilege action, and dependency receipt validation for replayed Cedar decision token.
- Threat 4: spoofed dependency receipt.
  Mitigation: require Cedar decision, idempotency key, HLC ordering, audit-chain seal, least-privilege action, and dependency receipt validation for spoofed dependency receipt.
## Tampering
- Threat 1: history row rewrite.
  Mitigation: require Cedar decision, idempotency key, HLC ordering, audit-chain seal, least-privilege action, and dependency receipt validation for history row rewrite.
- Threat 2: gate snapshot hash swap.
  Mitigation: require Cedar decision, idempotency key, HLC ordering, audit-chain seal, least-privilege action, and dependency receipt validation for gate snapshot hash swap.
- Threat 3: Valkey stale overwrite.
  Mitigation: require Cedar decision, idempotency key, HLC ordering, audit-chain seal, least-privilege action, and dependency receipt validation for Valkey stale overwrite.
- Threat 4: request idempotency collision.
  Mitigation: require Cedar decision, idempotency key, HLC ordering, audit-chain seal, least-privilege action, and dependency receipt validation for request idempotency collision.
## Repudiation
- Threat 1: operator denies emergency drain.
  Mitigation: require Cedar decision, idempotency key, HLC ordering, audit-chain seal, least-privilege action, and dependency receipt validation for operator denies emergency drain.
- Threat 2: automation lacks proposal trace.
  Mitigation: require Cedar decision, idempotency key, HLC ordering, audit-chain seal, least-privilege action, and dependency receipt validation for automation lacks proposal trace.
- Threat 3: missing audit-chain event.
  Mitigation: require Cedar decision, idempotency key, HLC ordering, audit-chain seal, least-privilege action, and dependency receipt validation for missing audit-chain event.
- Threat 4: unlinked incident id.
  Mitigation: require Cedar decision, idempotency key, HLC ordering, audit-chain seal, least-privilege action, and dependency receipt validation for unlinked incident id.
## Information Disclosure
- Threat 1: evidence pack leaks tenant data.
  Mitigation: require Cedar decision, idempotency key, HLC ordering, audit-chain seal, least-privilege action, and dependency receipt validation for evidence pack leaks tenant data.
- Threat 2: lifecycle list exposes restricted pack info.
  Mitigation: require Cedar decision, idempotency key, HLC ordering, audit-chain seal, least-privilege action, and dependency receipt validation for lifecycle list exposes restricted pack info.
- Threat 3: Cedar refusal leaks policy internals.
  Mitigation: require Cedar decision, idempotency key, HLC ordering, audit-chain seal, least-privilege action, and dependency receipt validation for Cedar refusal leaks policy internals.
- Threat 4: logs include secret receipts.
  Mitigation: require Cedar decision, idempotency key, HLC ordering, audit-chain seal, least-privilege action, and dependency receipt validation for logs include secret receipts.
## Denial of Service
- Threat 1: promotion gate validator saturation.
  Mitigation: require Cedar decision, idempotency key, HLC ordering, audit-chain seal, least-privilege action, and dependency receipt validation for promotion gate validator saturation.
- Threat 2: Postgres lock contention.
  Mitigation: require Cedar decision, idempotency key, HLC ordering, audit-chain seal, least-privilege action, and dependency receipt validation for Postgres lock contention.
- Threat 3: Valkey cache stampede.
  Mitigation: require Cedar decision, idempotency key, HLC ordering, audit-chain seal, least-privilege action, and dependency receipt validation for Valkey cache stampede.
- Threat 4: audit-chain outage blocks transitions.
  Mitigation: require Cedar decision, idempotency key, HLC ordering, audit-chain seal, least-privilege action, and dependency receipt validation for audit-chain outage blocks transitions.
## Elevation of Privilege
- Threat 1: generic ops principal promotes T0.
  Mitigation: require Cedar decision, idempotency key, HLC ordering, audit-chain seal, least-privilege action, and dependency receipt validation for generic ops principal promotes T0.
- Threat 2: drain without evidence permit.
  Mitigation: require Cedar decision, idempotency key, HLC ordering, audit-chain seal, least-privilege action, and dependency receipt validation for drain without evidence permit.
- Threat 3: decommission before resident zero.
  Mitigation: require Cedar decision, idempotency key, HLC ordering, audit-chain seal, least-privilege action, and dependency receipt validation for decommission before resident zero.
- Threat 4: automation edits routing or provisioning.
  Mitigation: require Cedar decision, idempotency key, HLC ordering, audit-chain seal, least-privilege action, and dependency receipt validation for automation edits routing or provisioning.

## §autosharding-event-drift

Source ADRs: ADR-0346, ADR-0347, ADR-0348, ADR-0349.

Threat: autosharding, auto-rebalance, or dynamic sharding automation events can drift from the manifest-declared `sharding_automation` contract, causing a tenant to move to the wrong cell or shard, bypass residency/compliance filters, or leave no reversible audit trail. The threat covers spoofed control-plane principals, tampered threshold inputs, missing audit-chain rows, stale routing cutovers, denial amplification during hot-split/cold-merge, and privilege escalation through unauthorized cross-jurisdiction migration.

Required controls:
- ADR-0348: `governance-sharding-automation-coverage` refuses any microservice manifest without complete autosharding, auto_rebalance, and dynamic_sharding sub-block declarations.
- ADR-0348: `governance-autosharding-manual-mode-refusal` refuses `manual`; the canonical autosharding mode is `control_plane_driven`.
- ADR-0348: `governance-auto-rebalance-residency-honored` requires auto-rebalance to honor residency and compliance packs; cross-jurisdiction migration requires an explicit Cedar permit per ADR-0243.
- ADR-0348: `governance-dynamic-sharding-threshold-coverage` requires explicit hot-split and cold-merge thresholds; default-fill is rejected.
- ADR-0348: `governance-audit-chain-emit-on-automation-events` requires every auto-rebalance, hot-split, and cold-merge event to emit per ADR-0263; `governance-tenant-migration-reversibility` requires a rollback path.
- ADR-0346: `./bin/oya verify --ci-required` is the canonical local pre-push verifier and must mirror cargo fmt, cargo check, cargo clippy, cargo nextest, and `oya gate run-all` before returning success.
- ADR-0347: governance-owned checks use the `governance-*` prefix; threat-model evidence must cite the governance lane names above without reintroducing stale lane vocabulary.
- ADR-0349: Jenkins/GitHub Actions parity and ArgoCD cosign/audit-chain lanes preserve the same controls in self-hostable CI/CD contexts.

Evidence required: every accepted automation event records event_type, tenant_id when tenant-level, cell_id, shard_id when shard-level, pre_state, post_state, residency_check_result, compliance_pack_check_result, cedar_permit_id when applicable, and initiated_by `control_plane:cell-orchestrator` in the audit-chain row. Residual risk remains until Wave 15-ZD proves race-free cutover and rollback under concurrent auto-rebalance, hot-split, and cold-merge jobs.
