# shared-connector-kernel

The enterprise integration substrate — the trait + types every external
SaaS connector implements.

## Layer

Layer 1 (kernel) per ADR-0148 layered architecture discipline. No I/O, no
network, no codegen, no async runtime.

## What lives here

* `Connector` trait — the contract every `connector-<vendor>-adapter`
  satisfies.
* `ConnectorCtx` — per-call carrier: `TenantId`, `PrincipalId`,
  `SecretReference` (OpenBao path), `TraceContext`, `AuditSealHandle`.
* `EntityDoc` + `EntityValue` + `PatchOp` — provider-neutral entity shape.
* `Cursor` + `Page` — pagination per ADR-0150 cursor-pagination.
* `IdempotencyKey` — at-most-once enforcement per ADR-0149.
* `ConnectorCapabilities` — honest verb declaration.
* `RateLimitDescriptor` — published per-provider budget.
* `AuthScheme` — `OAuth2 | ApiKey | MutualTls | SignedJwt`.
* `HealthReport`, `Event`, `EventStream` — runtime contracts.
* `OntologyProjection` — Ontology-µservice projection envelope.
* `ConnectorError` — unified error vocabulary.

## Why a kernel?

* **Adapter swap.** Callers depend on `Connector`, not on Workday/Salesforce.
* **Multi-tenant.** Every call carries `TenantId` + per-tenant
  `SecretReference` into OpenBao.
* **Audit.** Every call must seal through `AuditSealHandle` (ADR-0145).
* **Ontology.** Adapters publish projections so the Ontology µservice can
  materialize Employee/Customer/Encounter etc. The
  `check-ontology-projection-coverage` gate enforces coverage.
* **Rate-limit honesty.** Adapters publish budgets; callers schedule.

## Tests

`cargo test -p shared-connector-kernel` — 16 unit tests covering id
construction, secret redaction, idempotency bounds, capability defaults,
audit-seal receipts, error enum distinct, ontology projection builder,
context round-trip.
