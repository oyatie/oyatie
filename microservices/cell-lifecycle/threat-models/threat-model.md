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
