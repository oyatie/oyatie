# Ops Dashboard / Control Center Tenant Isolation

## Rule

The control center may show tenant isolation posture, but it must not become a cross-tenant data browser. Tenant views are scoped by operator role, tenant claim, support entitlement, and Cedar policy.

## Data handling

- Tenant posture responses expose status, observed timestamp, and evidence refs.
- Raw tenant payloads remain in owning microservices.
- Evidence-pack export requires explicit scope and time window.
- Break-glass access requires a ticket and emits audit-chain evidence.

## Acceptance criteria

- Tenant posture reads fail closed when operator tenant scope and resource tenant differ.
- Cross-tenant aggregation is limited to internal fleet health signals with no tenant payload.
- Every tenant-scoped command includes idempotency key and audit seal.
