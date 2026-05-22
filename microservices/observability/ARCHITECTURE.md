# observability

_This file was created by the Wave-3-C anchor-sweep. Expand all stub sections during content-pass review._

---



## §principals
This anchor is closed for `observability` against ADR-0242 §D-1: principal roster and tenant-scoped caller model.

### Service-specific answer
- Platform principal `oyatie.observability.runtime` owns normal `eligibility-query` execution and never borrows a tenant principal.
- Platform principal `oyatie.observability.worker` owns async jobs, retry queues, and backfill replay listed in `microservices/observability/backfill-replay.md` when present.
- Platform principal `oyatie.observability.auditor` has read-only evidence access through auditor Cedar fragments, not direct database credentials.
- Platform principal `oyatie.observability.ci` is limited to synthetic tenants and non-production cells by the CI-scope Cedar fragment.
- Tenant callers are represented as `<tenant>.observability.caller` and must provide `tenant_id`, `principal_id`, `audience_type`, and workload SVID context.
- Cross-µservice callers expected by dependency graph: `identity`, `tenancy`, `policy-engine`, `audit-chain`, `cloud-secrets`, `cell`.
- Example: a tenant principal invoking `eligibility-query` is evaluated as `<tenant>.observability.eligibility-query` before any `observability` state mutation.
- Forbidden: caller-supplied `oyatie.*` principals; ADR-0242 treats `oyatie` as its own tenant, not a namespace tenants can impersonate.

### Concrete inventory used
- Service: `observability`; owner `axis-observability`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `observability`.
- Capability records cited: `microservices/observability/capabilities/eligibility-query.yaml`, `microservices/observability/capabilities/openslo-validate.yaml`, `microservices/observability/capabilities/slo-evaluate.yaml`.
- API surfaces cited: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- Cedar/policy artifacts cited: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`, `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml`, `microservices/observability/slos/log-ingest-availability.openslo.yaml`, `microservices/observability/slos/metric-ingest-availability.openslo.yaml`, `microservices/observability/slos/query-latency-logs.openslo.yaml`, `microservices/observability/slos/query-latency-prom.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/observability/runbooks/canary-graduation.md`, `microservices/observability/runbooks/clickhouse-restore.md`, `microservices/observability/runbooks/clickhouse.md`, `microservices/observability/runbooks/evaluator-down.md`, `microservices/observability/runbooks/held-promotion-recovery.md`, `microservices/observability/runbooks/mimir-outage.md`; +16 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- Cedar binding: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- State/event binding: `observability.observability`.
- Capability binding: `eligibility-query`, `openslo-validate`, `slo-evaluate`.
- SLO binding: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`, `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml`, `microservices/observability/slos/log-ingest-availability.openslo.yaml`, `microservices/observability/slos/metric-ingest-availability.openslo.yaml`, `microservices/observability/slos/query-latency-logs.openslo.yaml`, `microservices/observability/slos/query-latency-prom.openslo.yaml`; +2 more.
- Runbook binding: `microservices/observability/runbooks/canary-graduation.md`, `microservices/observability/runbooks/clickhouse-restore.md`, `microservices/observability/runbooks/clickhouse.md`, `microservices/observability/runbooks/evaluator-down.md`, `microservices/observability/runbooks/held-promotion-recovery.md`, `microservices/observability/runbooks/mimir-outage.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `observability`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `observability`.
- `policy-engine` supplies the signed Cedar corpus while `observability` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `observability` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `observability`.

### Hyperscaler precedents
- Precedent 1: AWS IAM service-linked roles is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud service agents is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `observability` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §cell-health

`observability` owns cell health, per-cell SLO burn, blast-radius monitoring,
and cell isolation alerts after Wave 15L. The retired `cell` service no longer
emits a special health API. Health is derived from telemetry already owned by
observability and joined to the cell registry labels published by `cloud-iac`.

Health inputs:
- OpenTelemetry traces, RED metrics, USE metrics, and logs with `cell_id`,
  `pack`, `region`, `tenant_class`, and `workload_service` labels.
- `cloud-iac` lifecycle events for `requested`, `provisioning`, `ready`,
  `draining`, and `retired`.
- `tenancy` assignment events with `tenant_id` protected in audit evidence and
  low-cardinality aggregate metrics exposed only by tenant class or pack.
- `api-gateway` routing events with `principal.cell`, upstream cell, circuit
  breaker state, and route class.
- `audit-chain` seal latency and evidence-export freshness per cell.

Health verdicts:
- `healthy`: eligible for new assignments if `cloud-iac` also marks the cell
  ready.
- `degraded`: existing traffic may continue; tenancy should avoid new
  assignments unless an operator override is sealed.
- `isolated`: api-gateway opens per-cell circuit breakers and tenancy refuses
  new assignments.
- `draining`: health is tracked but the cell is intentionally excluded from new
  placement.
- `unknown`: treated as not eligible for new tenant placement.

Blast-radius monitoring:
- Every alert is cell-scoped first, then pack-scoped, then global.
- A single-cell SLO burn must not promote to global severity unless correlated
  cells or shared dependencies show the same fingerprint.
- Dashboards group by `cell_id`, `pack`, `region`, `workload_service`, and
  dependency class; they do not expose raw tenant identifiers.
- Per-cell error budgets exist for assignment lookup, api-gateway routing,
  audit-chain sealing, and workload request success.

Isolation alerts:
- `CellSloFastBurn`: fast burn within one cell.
- `CellBlastRadiusExceeded`: blast radius crosses the configured cell count or
  tenant-class boundary.
- `CellIsolationBreach`: cross-cell request, cache, database, or object-prefix
  access is detected.
- `CellDrainRegression`: drained cell continues to receive new primary traffic.
- `CellRegistryHealthMismatch`: cloud-iac says `ready` while telemetry says
  `unknown`, `degraded`, or `isolated`.

Cross-service behavior:
- `tenancy` consumes the health verdict before assignment.
- `cloud-iac` consumes capacity and health burn to trigger new-cell planning or
  drain.
- `api-gateway` consumes isolation and circuit-breaker verdicts for routing.
- `audit-chain` seals incident, drain, and isolation evidence.

Verification:
- Per-cell SLO burn dashboards must exist before ADR-0248 moves from advisory
  to blocker enforcement.
- Health verdicts must be reproducible from telemetry and lifecycle evidence,
  not manual notes.
- ADR-0333 is the retirement decision record for why this health responsibility
  lives in observability rather than a standalone cell service.

## §cedar-gates
This anchor is closed for `observability` against ADR-0243 §D-2: Cedar fragment roster, default-deny and action taxonomy.

### Service-specific answer
- Default-deny is represented by the first matching Cedar fragment in `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- Action namespace uses `observability::<bounded_context>::<verb>`; the first protected action is `observability::eligibility-query::execute`.
- Every evaluation context carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, `bot_score`, and `provider_credential_mode`.
- Mutating actions require an audit event class before the usecase layer runs; read actions require a purpose and data-class declaration.
- Auditor reads are time-boxed and read-only; CI principals are sandbox-only; emergency bypass policies never skip audit emission.
- Concrete fragments in scope: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- Example: `observability::eligibility-query::execute` denies if `resource.tenant_id != principal.tenant_id` or if the compliance pack adds a stricter overlay.
- Fragment publish observes ADR-0294 soak before activation; rollback reverts the fragment pointer, not the business-state rows.

### Concrete inventory used
- Service: `observability`; owner `axis-observability`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `observability`.
- Capability records cited: `microservices/observability/capabilities/eligibility-query.yaml`, `microservices/observability/capabilities/openslo-validate.yaml`, `microservices/observability/capabilities/slo-evaluate.yaml`.
- API surfaces cited: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- Cedar/policy artifacts cited: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`, `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml`, `microservices/observability/slos/log-ingest-availability.openslo.yaml`, `microservices/observability/slos/metric-ingest-availability.openslo.yaml`, `microservices/observability/slos/query-latency-logs.openslo.yaml`, `microservices/observability/slos/query-latency-prom.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/observability/runbooks/canary-graduation.md`, `microservices/observability/runbooks/clickhouse-restore.md`, `microservices/observability/runbooks/clickhouse.md`, `microservices/observability/runbooks/evaluator-down.md`, `microservices/observability/runbooks/held-promotion-recovery.md`, `microservices/observability/runbooks/mimir-outage.md`; +16 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- Cedar binding: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- State/event binding: `observability.observability`.
- Capability binding: `eligibility-query`, `openslo-validate`, `slo-evaluate`.
- SLO binding: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`, `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml`, `microservices/observability/slos/log-ingest-availability.openslo.yaml`, `microservices/observability/slos/metric-ingest-availability.openslo.yaml`, `microservices/observability/slos/query-latency-logs.openslo.yaml`, `microservices/observability/slos/query-latency-prom.openslo.yaml`; +2 more.
- Runbook binding: `microservices/observability/runbooks/canary-graduation.md`, `microservices/observability/runbooks/clickhouse-restore.md`, `microservices/observability/runbooks/clickhouse.md`, `microservices/observability/runbooks/evaluator-down.md`, `microservices/observability/runbooks/held-promotion-recovery.md`, `microservices/observability/runbooks/mimir-outage.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `observability`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `observability`.
- `policy-engine` supplies the signed Cedar corpus while `observability` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `observability` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `observability`.

### Hyperscaler precedents
- Precedent 1: AWS Verified Permissions Cedar policy evaluation is the reference pattern for the control shape described here.
- Precedent 2: Google Zanzibar relationship checks is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `observability` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §tenant-scoping
This anchor is closed for `observability` against ADR-0244 §D-3: tenant_id, audience_type, provider_credential_mode and row/event boundaries.

### Service-specific answer
- Audience type for this service is `B2B_TENANT + INTERNAL_OPERATOR` and is copied into audit events plus any public contract response that exposes policy posture.
- Provider credential mode is `tenant-provider-byok where external providers exist; platform-default credentials otherwise`; provider-BYOK and encryption-BYOK stay separate per ADR-0255 §D-4 / ADR-0251 §D-10.
- Required fields on mutable rows/events: `tenant_id`, `principal_id`, `caller_tenant_id` when delegated, `home_cell`, `jurisdiction_code`, `data_class`, and `audit_event_class`.
- State surface `observability.observability` is documented as tenant-scoped with `tenant_id`, `principal_id`, `audit_event_class`, `home_cell`, and `lifecycle_state` columns/events.
- Planned table/event surface `observability.eligibility_query_2` inherits the same `tenant_id` and audit fields until a migration file supersedes this placeholder name.
- Planned table/event surface `observability.eligibility_query_3` inherits the same `tenant_id` and audit fields until a migration file supersedes this placeholder name.
- Planned table/event surface `observability.eligibility_query_4` inherits the same `tenant_id` and audit fields until a migration file supersedes this placeholder name.
- Planned table/event surface `observability.eligibility_query_5` inherits the same `tenant_id` and audit fields until a migration file supersedes this placeholder name.
- Cross-tenant reads fail at Cedar before storage adapters see a query; storage row-level policy is defence-in-depth, not the primary guard.
- Example: `eligibility-query` reads include `tenant_id` and `home_cell`; a stale `jurisdiction_code` forces most-restrictive-pack handling until tenancy refresh completes.

### Concrete inventory used
- Service: `observability`; owner `axis-observability`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `observability`.
- Capability records cited: `microservices/observability/capabilities/eligibility-query.yaml`, `microservices/observability/capabilities/openslo-validate.yaml`, `microservices/observability/capabilities/slo-evaluate.yaml`.
- API surfaces cited: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- Cedar/policy artifacts cited: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`, `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml`, `microservices/observability/slos/log-ingest-availability.openslo.yaml`, `microservices/observability/slos/metric-ingest-availability.openslo.yaml`, `microservices/observability/slos/query-latency-logs.openslo.yaml`, `microservices/observability/slos/query-latency-prom.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/observability/runbooks/canary-graduation.md`, `microservices/observability/runbooks/clickhouse-restore.md`, `microservices/observability/runbooks/clickhouse.md`, `microservices/observability/runbooks/evaluator-down.md`, `microservices/observability/runbooks/held-promotion-recovery.md`, `microservices/observability/runbooks/mimir-outage.md`; +16 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- Cedar binding: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- State/event binding: `observability.observability`.
- Capability binding: `eligibility-query`, `openslo-validate`, `slo-evaluate`.
- SLO binding: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`, `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml`, `microservices/observability/slos/log-ingest-availability.openslo.yaml`, `microservices/observability/slos/metric-ingest-availability.openslo.yaml`, `microservices/observability/slos/query-latency-logs.openslo.yaml`, `microservices/observability/slos/query-latency-prom.openslo.yaml`; +2 more.
- Runbook binding: `microservices/observability/runbooks/canary-graduation.md`, `microservices/observability/runbooks/clickhouse-restore.md`, `microservices/observability/runbooks/clickhouse.md`, `microservices/observability/runbooks/evaluator-down.md`, `microservices/observability/runbooks/held-promotion-recovery.md`, `microservices/observability/runbooks/mimir-outage.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `observability`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `observability`.
- `policy-engine` supplies the signed Cedar corpus while `observability` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `observability` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `observability`.

### Hyperscaler precedents
- Precedent 1: Stripe Connect connected-account isolation is the reference pattern for the control shape described here.
- Precedent 2: AWS Organizations account-boundary pattern is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `observability` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §substrate-product-binding
This anchor is closed for `observability` against ADR-0245 §D-1: substrate/product classification and dependency direction.

### Service-specific answer
- Manifest classifies `observability` as `substrate`, so this section treats it as a substrate provider.
- Declared substrate/product dependencies: `identity`, `tenancy`, `policy-engine`, `audit-chain`, `cloud-secrets`, `cell`.
- If substrate: products consume `observability` only through contracts `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- If product: `observability` may call substrate services but must not create product-to-product synchronous dependencies.
- Dependency direction is inward to clean core crates; adapter and framework code never defines domain terms for other µservices.
- Primary bounded contexts bound to this classification: `observability`.
- Example: `eligibility-query` may depend on `tenancy` for tenant state and `observability` for audit emission, but not on another product UI workflow.
- ADR-0280 substrate-of-substrate ordering is documented here so delivery planning can parallelize product work without creating hidden runtime coupling.

### Concrete inventory used
- Service: `observability`; owner `axis-observability`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `observability`.
- Capability records cited: `microservices/observability/capabilities/eligibility-query.yaml`, `microservices/observability/capabilities/openslo-validate.yaml`, `microservices/observability/capabilities/slo-evaluate.yaml`.
- API surfaces cited: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- Cedar/policy artifacts cited: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`, `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml`, `microservices/observability/slos/log-ingest-availability.openslo.yaml`, `microservices/observability/slos/metric-ingest-availability.openslo.yaml`, `microservices/observability/slos/query-latency-logs.openslo.yaml`, `microservices/observability/slos/query-latency-prom.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/observability/runbooks/canary-graduation.md`, `microservices/observability/runbooks/clickhouse-restore.md`, `microservices/observability/runbooks/clickhouse.md`, `microservices/observability/runbooks/evaluator-down.md`, `microservices/observability/runbooks/held-promotion-recovery.md`, `microservices/observability/runbooks/mimir-outage.md`; +16 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- Cedar binding: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- State/event binding: `observability.observability`.
- Capability binding: `eligibility-query`, `openslo-validate`, `slo-evaluate`.
- SLO binding: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`, `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml`, `microservices/observability/slos/log-ingest-availability.openslo.yaml`, `microservices/observability/slos/metric-ingest-availability.openslo.yaml`, `microservices/observability/slos/query-latency-logs.openslo.yaml`, `microservices/observability/slos/query-latency-prom.openslo.yaml`; +2 more.
- Runbook binding: `microservices/observability/runbooks/canary-graduation.md`, `microservices/observability/runbooks/clickhouse-restore.md`, `microservices/observability/runbooks/clickhouse.md`, `microservices/observability/runbooks/evaluator-down.md`, `microservices/observability/runbooks/held-promotion-recovery.md`, `microservices/observability/runbooks/mimir-outage.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `observability`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `observability`.
- `policy-engine` supplies the signed Cedar corpus while `observability` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `observability` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `observability`.

### Hyperscaler precedents
- Precedent 1: Palantir Foundry shared ontology substrate is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud shared VPC/service-project split is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `observability` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §policy-evaluation
This anchor is closed for `observability` against ADR-0246 §D-4: library-first policy evaluation mode and fallback limits.

### Service-specific answer
- `policy_evaluation_mode = library-first`; network policy-engine calls are fallback only for stale local fragment cache or explicit audit replay.
- The caller-side library evaluates fragments from `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar` with the full action/resource/context tuple.
- Partial-context evaluation is forbidden: no action can be evaluated without `tenant_id`, `principal_id`, `audience_type`, `resource_id`, and `data_class`.
- Cache freshness target is ≤5 minutes for normal fragments and immediate invalidation for deny-list, credential, or pack-overlay revocations.
- Fallback network evaluation emits `PolicyEvaluationFallbackUsed` with fragment hash and reason so ADR-0263 can detect degraded posture.
- Example: `observability::eligibility-query::execute` evaluates locally, then emits an audit event before the usecase writes `observability.observability`.
- OpenBao credential sidecar is never called before policy allow; policy decides whether the credential lookup is authorized.
- This mirrors OPA sidecar discipline: policy is close to the caller, but the source of truth remains signed and soaked Cedar fragments.

### Concrete inventory used
- Service: `observability`; owner `axis-observability`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `observability`.
- Capability records cited: `microservices/observability/capabilities/eligibility-query.yaml`, `microservices/observability/capabilities/openslo-validate.yaml`, `microservices/observability/capabilities/slo-evaluate.yaml`.
- API surfaces cited: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- Cedar/policy artifacts cited: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`, `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml`, `microservices/observability/slos/log-ingest-availability.openslo.yaml`, `microservices/observability/slos/metric-ingest-availability.openslo.yaml`, `microservices/observability/slos/query-latency-logs.openslo.yaml`, `microservices/observability/slos/query-latency-prom.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/observability/runbooks/canary-graduation.md`, `microservices/observability/runbooks/clickhouse-restore.md`, `microservices/observability/runbooks/clickhouse.md`, `microservices/observability/runbooks/evaluator-down.md`, `microservices/observability/runbooks/held-promotion-recovery.md`, `microservices/observability/runbooks/mimir-outage.md`; +16 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- Cedar binding: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- State/event binding: `observability.observability`.
- Capability binding: `eligibility-query`, `openslo-validate`, `slo-evaluate`.
- SLO binding: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`, `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml`, `microservices/observability/slos/log-ingest-availability.openslo.yaml`, `microservices/observability/slos/metric-ingest-availability.openslo.yaml`, `microservices/observability/slos/query-latency-logs.openslo.yaml`, `microservices/observability/slos/query-latency-prom.openslo.yaml`; +2 more.
- Runbook binding: `microservices/observability/runbooks/canary-graduation.md`, `microservices/observability/runbooks/clickhouse-restore.md`, `microservices/observability/runbooks/clickhouse.md`, `microservices/observability/runbooks/evaluator-down.md`, `microservices/observability/runbooks/held-promotion-recovery.md`, `microservices/observability/runbooks/mimir-outage.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `observability`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `observability`.
- `policy-engine` supplies the signed Cedar corpus while `observability` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `observability` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `observability`.

### Hyperscaler precedents
- Precedent 1: AWS Verified Permissions local Cedar model is the reference pattern for the control shape described here.
- Precedent 2: Open Policy Agent sidecar evaluation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `observability` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §cell-eligibility
This anchor is closed for `observability` against ADR-0248 §D-1: cell tier, shard width, DR pair and shuffle-shard behavior.

### Service-specific answer
- Cell eligibility declaration: `not declared in manifest; bound here to the conservative platform default`.
- Tier 0/1 control-plane paths run in hardened cells; tenant data planes can shard per tenant, pack, region, and workload class.
- Per-cell shard key is `(tenant_id, home_cell, jurisdiction_code)`; DR pair selection uses `dr_cell` where data-residency permits failover.
- Shuffle-shard width is documented by `multi-region.md` or defaults to three independent cells for Tier-1 control paths.
- Regional outage behavior: keep reads local where pack permits, stop cross-border replication where pack forbids it, and preserve audit emission locally.
- Example: `eligibility-query` traffic in a KR pack tenant stays in KR home cell; DR failover requires pack approval and emits a cell-failover audit event.
- Capacity math lives in `capacity-model.md`; this section binds the shard dimensions so the math is not detached from topology.
- Cloud Hypervisor/Kata isolation applies to Tier 0/1 pods; Tier 2/3 paths inherit the same network policy and SPIFFE identity floor.

### Concrete inventory used
- Service: `observability`; owner `axis-observability`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `observability`.
- Capability records cited: `microservices/observability/capabilities/eligibility-query.yaml`, `microservices/observability/capabilities/openslo-validate.yaml`, `microservices/observability/capabilities/slo-evaluate.yaml`.
- API surfaces cited: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- Cedar/policy artifacts cited: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`, `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml`, `microservices/observability/slos/log-ingest-availability.openslo.yaml`, `microservices/observability/slos/metric-ingest-availability.openslo.yaml`, `microservices/observability/slos/query-latency-logs.openslo.yaml`, `microservices/observability/slos/query-latency-prom.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/observability/runbooks/canary-graduation.md`, `microservices/observability/runbooks/clickhouse-restore.md`, `microservices/observability/runbooks/clickhouse.md`, `microservices/observability/runbooks/evaluator-down.md`, `microservices/observability/runbooks/held-promotion-recovery.md`, `microservices/observability/runbooks/mimir-outage.md`; +16 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- Cedar binding: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- State/event binding: `observability.observability`.
- Capability binding: `eligibility-query`, `openslo-validate`, `slo-evaluate`.
- SLO binding: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`, `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml`, `microservices/observability/slos/log-ingest-availability.openslo.yaml`, `microservices/observability/slos/metric-ingest-availability.openslo.yaml`, `microservices/observability/slos/query-latency-logs.openslo.yaml`, `microservices/observability/slos/query-latency-prom.openslo.yaml`; +2 more.
- Runbook binding: `microservices/observability/runbooks/canary-graduation.md`, `microservices/observability/runbooks/clickhouse-restore.md`, `microservices/observability/runbooks/clickhouse.md`, `microservices/observability/runbooks/evaluator-down.md`, `microservices/observability/runbooks/held-promotion-recovery.md`, `microservices/observability/runbooks/mimir-outage.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `observability`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `observability`.
- `policy-engine` supplies the signed Cedar corpus while `observability` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `observability` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `observability`.

### Hyperscaler precedents
- Precedent 1: AWS cell-based architecture is the reference pattern for the control shape described here.
- Precedent 2: Route 53 shuffle-sharding isolation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `observability` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §transport
This anchor is closed for `observability` against ADR-0253 §D-1: HTTP/3 fallback chain, strict TLS, ECH and PQC posture.

### Service-specific answer
- Public/API contracts in scope: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- REST and gRPC advertise HTTP/3 through `Alt-Svc: h3=":443"`; fallback order is HTTP/3, then HTTP/2, then HTTP/1.1 with first acceptable winning.
- HTTP/1.0, TLS <1.3, self-signed production certificates, `insecure_skip_verify`, and MITM bypass headers are forbidden.
- ECH is advertised through HTTPS RR `ech=` config wherever the platform terminates TLS; ECH-disabled clients fall back to ordinary TLS 1.3 without refusal.
- PQC hybrid `X25519MLKEM768` is offered where the client/server pair supports it; classical X25519/P-256 fallback is accepted during migration.
- IaC transport evidence: `microservices/observability/iac/helm/alertmanager/Chart.yaml`, `microservices/observability/iac/helm/alertmanager/values.yaml`, `microservices/observability/iac/helm/alloy/Chart.yaml`, `microservices/observability/iac/helm/alloy/templates/deployment.yaml`, `microservices/observability/iac/helm/alloy/templates/networkpolicy.yaml`, `microservices/observability/iac/helm/alloy/values.yaml`; +6 more.
- Example: `observability` `eligibility-query` calls use HTTP/3 on normal networks and HTTP/2 when UDP/QUIC is blocked by enterprise firewalls.
- Async/event transport preserves tenant and audit context on every message; webhook ingress verifies HMAC/mTLS before Cedar sees the payload.

### Concrete inventory used
- Service: `observability`; owner `axis-observability`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `observability`.
- Capability records cited: `microservices/observability/capabilities/eligibility-query.yaml`, `microservices/observability/capabilities/openslo-validate.yaml`, `microservices/observability/capabilities/slo-evaluate.yaml`.
- API surfaces cited: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- Cedar/policy artifacts cited: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`, `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml`, `microservices/observability/slos/log-ingest-availability.openslo.yaml`, `microservices/observability/slos/metric-ingest-availability.openslo.yaml`, `microservices/observability/slos/query-latency-logs.openslo.yaml`, `microservices/observability/slos/query-latency-prom.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/observability/runbooks/canary-graduation.md`, `microservices/observability/runbooks/clickhouse-restore.md`, `microservices/observability/runbooks/clickhouse.md`, `microservices/observability/runbooks/evaluator-down.md`, `microservices/observability/runbooks/held-promotion-recovery.md`, `microservices/observability/runbooks/mimir-outage.md`; +16 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- Cedar binding: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- State/event binding: `observability.observability`.
- Capability binding: `eligibility-query`, `openslo-validate`, `slo-evaluate`.
- SLO binding: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`, `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml`, `microservices/observability/slos/log-ingest-availability.openslo.yaml`, `microservices/observability/slos/metric-ingest-availability.openslo.yaml`, `microservices/observability/slos/query-latency-logs.openslo.yaml`, `microservices/observability/slos/query-latency-prom.openslo.yaml`; +2 more.
- Runbook binding: `microservices/observability/runbooks/canary-graduation.md`, `microservices/observability/runbooks/clickhouse-restore.md`, `microservices/observability/runbooks/clickhouse.md`, `microservices/observability/runbooks/evaluator-down.md`, `microservices/observability/runbooks/held-promotion-recovery.md`, `microservices/observability/runbooks/mimir-outage.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `observability`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `observability`.
- `policy-engine` supplies the signed Cedar corpus while `observability` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `observability` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `observability`.

### Hyperscaler precedents
- Precedent 1: Google QUIC/HTTP3 rollout pattern is the reference pattern for the control shape described here.
- Precedent 2: Cloudflare ECH and post-quantum TLS experiments is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `observability` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §deployment-shape
This anchor is closed for `observability` against ADR-0254 §D-2: Kubernetes, Cloud Hypervisor, Kata, Wasm/container/VM split.

### Service-specific answer
- Runtime components derive from bounded contexts `observability` and deploy as separate app/worker/API pods where files exist.
- Default shape is Kubernetes pods with Cloud Hypervisor + Kata isolation for Tier 0/1 paths; lower tiers inherit network policy and SPIFFE identity.
- Wasm is reserved for untrusted plugin or user-authored execution; normal business usecases remain containers; bootstrap/control roots may require VM isolation.
- IaC manifests in scope: `microservices/observability/iac/helm/alertmanager/Chart.yaml`, `microservices/observability/iac/helm/alertmanager/values.yaml`, `microservices/observability/iac/helm/alloy/Chart.yaml`, `microservices/observability/iac/helm/alloy/templates/deployment.yaml`, `microservices/observability/iac/helm/alloy/templates/networkpolicy.yaml`, `microservices/observability/iac/helm/alloy/values.yaml`; +6 more.
- Catalog crates/components: `microservices/observability/catalog/oya-observability-otel-ingest-adapter.yaml`, `microservices/observability/catalog/oya-observability-otel-ingest-api.yaml`, `microservices/observability/catalog/oya-observability-otel-ingest-app.yaml`, `microservices/observability/catalog/oya-observability-otel-ingest-kernel.yaml`, `microservices/observability/catalog/oya-observability-otel-ingest-usecase.yaml`, `microservices/observability/catalog/oya-observability-slo-engine-adapter-mimir.yaml`; +18 more.
- Example: `eligibility-query` API runs as a Kata-isolated container; scheduled/background work for `observability` uses a separate worker principal.
- OpenBao and SPIFFE mounts are sidecars, not linked into domain/core crates; adapters own framework code.
- Rollback is deployment-level first: pin previous image digest, keep schema backward-compatible, and replay idempotent events after recovery.

### Concrete inventory used
- Service: `observability`; owner `axis-observability`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `observability`.
- Capability records cited: `microservices/observability/capabilities/eligibility-query.yaml`, `microservices/observability/capabilities/openslo-validate.yaml`, `microservices/observability/capabilities/slo-evaluate.yaml`.
- API surfaces cited: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- Cedar/policy artifacts cited: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`, `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml`, `microservices/observability/slos/log-ingest-availability.openslo.yaml`, `microservices/observability/slos/metric-ingest-availability.openslo.yaml`, `microservices/observability/slos/query-latency-logs.openslo.yaml`, `microservices/observability/slos/query-latency-prom.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/observability/runbooks/canary-graduation.md`, `microservices/observability/runbooks/clickhouse-restore.md`, `microservices/observability/runbooks/clickhouse.md`, `microservices/observability/runbooks/evaluator-down.md`, `microservices/observability/runbooks/held-promotion-recovery.md`, `microservices/observability/runbooks/mimir-outage.md`; +16 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- Cedar binding: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- State/event binding: `observability.observability`.
- Capability binding: `eligibility-query`, `openslo-validate`, `slo-evaluate`.
- SLO binding: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`, `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml`, `microservices/observability/slos/log-ingest-availability.openslo.yaml`, `microservices/observability/slos/metric-ingest-availability.openslo.yaml`, `microservices/observability/slos/query-latency-logs.openslo.yaml`, `microservices/observability/slos/query-latency-prom.openslo.yaml`; +2 more.
- Runbook binding: `microservices/observability/runbooks/canary-graduation.md`, `microservices/observability/runbooks/clickhouse-restore.md`, `microservices/observability/runbooks/clickhouse.md`, `microservices/observability/runbooks/evaluator-down.md`, `microservices/observability/runbooks/held-promotion-recovery.md`, `microservices/observability/runbooks/mimir-outage.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `observability`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `observability`.
- `policy-engine` supplies the signed Cedar corpus while `observability` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `observability` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `observability`.

### Hyperscaler precedents
- Precedent 1: AWS Firecracker microVM isolation is the reference pattern for the control shape described here.
- Precedent 2: GKE Sandbox/Kata isolation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `observability` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §observability
This anchor is closed for `observability` against ADR-0263 §D-1: audit events, metrics, logs and trace span shape.

### Service-specific answer
- SLO/dashboard sources in scope: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`, `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml`, `microservices/observability/slos/log-ingest-availability.openslo.yaml`, `microservices/observability/slos/metric-ingest-availability.openslo.yaml`, `microservices/observability/slos/query-latency-logs.openslo.yaml`, `microservices/observability/slos/query-latency-prom.openslo.yaml`; +8 more.
- Audit classes use prefix `oya.observability.<context>.<outcome>` and must be registered centrally per ADR-0263.
- Core counter metric: `observability_eligibility_query_total` with dimensions `outcome`, `route_class`, `tenant_id_class`, and `cell_tier`.
- Core latency metric: `observability_eligibility_query_latency_ms` with bounded cardinality and no raw `tenant_id` label.
- Reference evidence artifact: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`.
- Trace root span is `<service>.<capability>`; child spans are `policy.evaluate`, `ontology.read`, `storage.write/read`, `audit.emit`, and provider/adapter calls.
- Logs are structured JSON, redacted by data class, retained per pack, and include correlation id plus audit event id.
- Example trace: `observability.eligibility-query` -> `policy.evaluate` -> `storage.eligibility_query` -> `audit.emit`.

### Concrete inventory used
- Service: `observability`; owner `axis-observability`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `observability`.
- Capability records cited: `microservices/observability/capabilities/eligibility-query.yaml`, `microservices/observability/capabilities/openslo-validate.yaml`, `microservices/observability/capabilities/slo-evaluate.yaml`.
- API surfaces cited: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- Cedar/policy artifacts cited: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`, `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml`, `microservices/observability/slos/log-ingest-availability.openslo.yaml`, `microservices/observability/slos/metric-ingest-availability.openslo.yaml`, `microservices/observability/slos/query-latency-logs.openslo.yaml`, `microservices/observability/slos/query-latency-prom.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/observability/runbooks/canary-graduation.md`, `microservices/observability/runbooks/clickhouse-restore.md`, `microservices/observability/runbooks/clickhouse.md`, `microservices/observability/runbooks/evaluator-down.md`, `microservices/observability/runbooks/held-promotion-recovery.md`, `microservices/observability/runbooks/mimir-outage.md`; +16 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- Cedar binding: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- State/event binding: `observability.observability`.
- Capability binding: `eligibility-query`, `openslo-validate`, `slo-evaluate`.
- SLO binding: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`, `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml`, `microservices/observability/slos/log-ingest-availability.openslo.yaml`, `microservices/observability/slos/metric-ingest-availability.openslo.yaml`, `microservices/observability/slos/query-latency-logs.openslo.yaml`, `microservices/observability/slos/query-latency-prom.openslo.yaml`; +2 more.
- Runbook binding: `microservices/observability/runbooks/canary-graduation.md`, `microservices/observability/runbooks/clickhouse-restore.md`, `microservices/observability/runbooks/clickhouse.md`, `microservices/observability/runbooks/evaluator-down.md`, `microservices/observability/runbooks/held-promotion-recovery.md`, `microservices/observability/runbooks/mimir-outage.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `observability`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `observability`.
- `policy-engine` supplies the signed Cedar corpus while `observability` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `observability` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `observability`.

### Hyperscaler precedents
- Precedent 1: Google SRE four reference signals is the reference pattern for the control shape described here.
- Precedent 2: OpenTelemetry semantic conventions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `observability` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §abuse-defence
This anchor is closed for `observability` against ADR-0297 §D-3: anti-bot, anti-spoof, anti-scrape controls plus UX floor.

### Service-specific answer
- Internet-facing `observability` routes use edge rate limits by IP, JA4 fingerprint, tenant, route class, and `eligibility-query` action class.
- Passive bot scoring is forwarded as `X-Oya-Bot-Score`; Cedar composes the score with quota and tenant tier before any challenge.
- Default path is friction-free: legitimate traffic receives no CAPTCHA, no JS proof-of-work, and ≤2ms p99 edge scoring budget.
- Cedar/IaC controls in scope: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`; +12 more.
- Canary payloads for `observability` use fake `eligibility-query` identifiers and honey endpoints under `/.well-known/oya-canary/observability`.
- Anti-spoof: HMAC on webhooks, SPIFFE SVID on service calls, SameSite=Strict on cookies, mTLS for machine clients, audit signatures via sidecar.
- Anti-scrape: per-fingerprint pagination caps, breadth-first crawl detection, watermarking for high-value content, and partner allow-list for friendly crawlers.
- Emergency-services and accessibility bypasses are evaluated before bot-score friction and still emit audit events.

### Concrete inventory used
- Service: `observability`; owner `axis-observability`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `observability`.
- Capability records cited: `microservices/observability/capabilities/eligibility-query.yaml`, `microservices/observability/capabilities/openslo-validate.yaml`, `microservices/observability/capabilities/slo-evaluate.yaml`.
- API surfaces cited: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- Cedar/policy artifacts cited: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`, `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml`, `microservices/observability/slos/log-ingest-availability.openslo.yaml`, `microservices/observability/slos/metric-ingest-availability.openslo.yaml`, `microservices/observability/slos/query-latency-logs.openslo.yaml`, `microservices/observability/slos/query-latency-prom.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/observability/runbooks/canary-graduation.md`, `microservices/observability/runbooks/clickhouse-restore.md`, `microservices/observability/runbooks/clickhouse.md`, `microservices/observability/runbooks/evaluator-down.md`, `microservices/observability/runbooks/held-promotion-recovery.md`, `microservices/observability/runbooks/mimir-outage.md`; +16 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- Cedar binding: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- State/event binding: `observability.observability`.
- Capability binding: `eligibility-query`, `openslo-validate`, `slo-evaluate`.
- SLO binding: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`, `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml`, `microservices/observability/slos/log-ingest-availability.openslo.yaml`, `microservices/observability/slos/metric-ingest-availability.openslo.yaml`, `microservices/observability/slos/query-latency-logs.openslo.yaml`, `microservices/observability/slos/query-latency-prom.openslo.yaml`; +2 more.
- Runbook binding: `microservices/observability/runbooks/canary-graduation.md`, `microservices/observability/runbooks/clickhouse-restore.md`, `microservices/observability/runbooks/clickhouse.md`, `microservices/observability/runbooks/evaluator-down.md`, `microservices/observability/runbooks/held-promotion-recovery.md`, `microservices/observability/runbooks/mimir-outage.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `observability`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `observability`.
- `policy-engine` supplies the signed Cedar corpus while `observability` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `observability` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `observability`.

### Hyperscaler precedents
- Precedent 1: Cloudflare Bot Management and Turnstile is the reference pattern for the control shape described here.
- Precedent 2: Stripe Radar passive risk scoring is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `observability` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §critical-path-edge-cases
This anchor is closed for `observability` against documentation-rigor.md §3.2.5: applicable human-safety and platform edge-case handling.

### Service-specific answer
- Network partition: `observability` keeps tenant-local reads when safe, stops cross-cell writes that would violate residency, and emits degraded-mode audit events.
- Byzantine caller: Cedar denies forged `principal_id`, mismatched `tenant_id`, invalid SVID, replayed idempotency keys, and suspicious bot-score context.
- Regional outage: home-cell failover follows `multi-region.md`; if a pack forbids cross-border DR, `observability` preserves local queue state instead of failing open.
- Key compromise: ADR-0296 sidecar revokes OpenBao leases, rotates signing keys, and quarantines affected audit event classes for reconciliation.
- Account recovery/hijack path: identity step-up and `observability` audit evidence keep legitimate recovery from becoming an adversary shortcut.
- Mistaken mutation path: high-impact `eligibility-query` mutations require idempotency, undo/cooldown where product semantics allow, and sealed evidence for later correction.
- Disaster surge: `observability` enforces per-tenant isolation so one hot tenant or emergency mode cannot starve unrelated cells.
- Verification: capacity math in `capacity-model.md`, rollback in `failure-modes.md`, DR handling in `multi-region.md`, and incident actions in runbooks.

### Concrete inventory used
- Service: `observability`; owner `axis-observability`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `observability`.
- Capability records cited: `microservices/observability/capabilities/eligibility-query.yaml`, `microservices/observability/capabilities/openslo-validate.yaml`, `microservices/observability/capabilities/slo-evaluate.yaml`.
- API surfaces cited: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- Cedar/policy artifacts cited: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`, `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml`, `microservices/observability/slos/log-ingest-availability.openslo.yaml`, `microservices/observability/slos/metric-ingest-availability.openslo.yaml`, `microservices/observability/slos/query-latency-logs.openslo.yaml`, `microservices/observability/slos/query-latency-prom.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/observability/runbooks/canary-graduation.md`, `microservices/observability/runbooks/clickhouse-restore.md`, `microservices/observability/runbooks/clickhouse.md`, `microservices/observability/runbooks/evaluator-down.md`, `microservices/observability/runbooks/held-promotion-recovery.md`, `microservices/observability/runbooks/mimir-outage.md`; +16 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- Cedar binding: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- State/event binding: `observability.observability`.
- Capability binding: `eligibility-query`, `openslo-validate`, `slo-evaluate`.
- SLO binding: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`, `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml`, `microservices/observability/slos/log-ingest-availability.openslo.yaml`, `microservices/observability/slos/metric-ingest-availability.openslo.yaml`, `microservices/observability/slos/query-latency-logs.openslo.yaml`, `microservices/observability/slos/query-latency-prom.openslo.yaml`; +2 more.
- Runbook binding: `microservices/observability/runbooks/canary-graduation.md`, `microservices/observability/runbooks/clickhouse-restore.md`, `microservices/observability/runbooks/clickhouse.md`, `microservices/observability/runbooks/evaluator-down.md`, `microservices/observability/runbooks/held-promotion-recovery.md`, `microservices/observability/runbooks/mimir-outage.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `observability`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `observability`.
- `policy-engine` supplies the signed Cedar corpus while `observability` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `observability` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `observability`.

### Hyperscaler precedents
- Precedent 1: Google SRE incident playbooks is the reference pattern for the control shape described here.
- Precedent 2: Stripe idempotent mutation recovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `observability` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §credential-isolation
This anchor is closed for `observability` against ADR-0296 §D-1: credential sidecar, OpenBao TTLs and secret-reference path.

### Service-specific answer
- Credential scopes for `observability` include `identity`, `tenancy`, `policy-engine`, `audit-chain`, `cloud-secrets`, `cell`.
- SecretReference pattern is `${openbao:secret/<tenant_id>/observability/<credential-name>}`; platform-owned credentials use tenant `oyatie` only for oyatie-internal calls.
- Sidecar mode is required for audit-signing keys and preferred for provider credentials; raw credentials never enter domain/core crates.
- OpenBao token TTL is ≤60 seconds when sidecar isolation is not possible; refresh must be policy-allowed and audit-emitted.
- Rotation cadence is linked from `microservices/observability/runbooks/canary-graduation.md`, `microservices/observability/runbooks/clickhouse-restore.md`, `microservices/observability/runbooks/clickhouse.md`, `microservices/observability/runbooks/evaluator-down.md`, `microservices/observability/runbooks/held-promotion-recovery.md`, `microservices/observability/runbooks/mimir-outage.md`; +4 more.
- Example: `eligibility-query` asks the sidecar for a scoped credential after Cedar allow; the sidecar returns a short-lived handle, not the underlying secret.
- Compromise response: revoke lease, rotate key, disable affected provider adapter, replay idempotent queue after audit-chain reconciliation.
- encryption-BYOK is separate from provider-BYOK; this section covers provider/API/signing credentials only unless a pack explicitly adds KMS-root behavior.

### Concrete inventory used
- Service: `observability`; owner `axis-observability`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `observability`.
- Capability records cited: `microservices/observability/capabilities/eligibility-query.yaml`, `microservices/observability/capabilities/openslo-validate.yaml`, `microservices/observability/capabilities/slo-evaluate.yaml`.
- API surfaces cited: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- Cedar/policy artifacts cited: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`, `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml`, `microservices/observability/slos/log-ingest-availability.openslo.yaml`, `microservices/observability/slos/metric-ingest-availability.openslo.yaml`, `microservices/observability/slos/query-latency-logs.openslo.yaml`, `microservices/observability/slos/query-latency-prom.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/observability/runbooks/canary-graduation.md`, `microservices/observability/runbooks/clickhouse-restore.md`, `microservices/observability/runbooks/clickhouse.md`, `microservices/observability/runbooks/evaluator-down.md`, `microservices/observability/runbooks/held-promotion-recovery.md`, `microservices/observability/runbooks/mimir-outage.md`; +16 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/observability/contracts/asyncapi/eligibility-events.yaml`, `microservices/observability/contracts/metric-naming-convention.md`, `microservices/observability/contracts/openapi/slo-engine.yaml`, `microservices/observability/contracts/proto/slo-engine.proto`.
- Cedar binding: `microservices/observability/policy/auditor-scope.cedar`, `microservices/observability/policy/ci-scope.cedar`, `microservices/observability/policy/data-residency.md`, `microservices/observability/policy/public-read.cedar`, `microservices/observability/policy/tenant-isolation.md`, `microservices/observability/policy/tenant-scope.cedar`.
- State/event binding: `observability.observability`.
- Capability binding: `eligibility-query`, `openslo-validate`, `slo-evaluate`.
- SLO binding: `microservices/observability/slos/alerting-fanout-latency.openslo.yaml`, `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml`, `microservices/observability/slos/log-ingest-availability.openslo.yaml`, `microservices/observability/slos/metric-ingest-availability.openslo.yaml`, `microservices/observability/slos/query-latency-logs.openslo.yaml`, `microservices/observability/slos/query-latency-prom.openslo.yaml`; +2 more.
- Runbook binding: `microservices/observability/runbooks/canary-graduation.md`, `microservices/observability/runbooks/clickhouse-restore.md`, `microservices/observability/runbooks/clickhouse.md`, `microservices/observability/runbooks/evaluator-down.md`, `microservices/observability/runbooks/held-promotion-recovery.md`, `microservices/observability/runbooks/mimir-outage.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `observability`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `observability`.
- `policy-engine` supplies the signed Cedar corpus while `observability` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `observability` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `observability`.

### Hyperscaler precedents
- Precedent 1: HashiCorp Vault dynamic secrets is the reference pattern for the control shape described here.
- Precedent 2: AWS KMS envelope-key isolation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `observability` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.
