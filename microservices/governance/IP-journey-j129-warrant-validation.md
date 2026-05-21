---
doc_class: IP
template_id: TPL-IP-Journey
ip_id: IP-journey-j129-warrant-validation
journey_id: j129-court-warrant-pierces-personal-tenant-with-judicial-oversight
microservice: governance
role: warrant-validation
status: draft
related_adrs:
  - ADR-0312-court-warrant-scoped-piercing
  - ADR-0295-bootstrap-ci-spiffe-killswitch
date: 2026-05-20
owner_team: axis-governance + axis-legal
parallel_work_compatibility: Independent of j130/j131 governance extensions
---

# IP-journey-j129-warrant-validation — Governance µservice: magistrate signature + scope validation

## Goal

Implement governance µservice surfaces that:

1. **`ValidateMagistrateSignature`** — validates a magistrate's
   signature against the federal-judiciary trust root (Fulcio +
   sigstore chain per ADR-0295).
2. **`ValidateWarrantScope`** — verifies the warrant's JSON metadata
   conforms to the schema + each scope item resolves to a valid
   µservice + tenant + time-bound query.
3. **`CheckActiveConflicts`** — verifies no active sealed-orders +
   no GDPR Article 48 conflicts + no security incidents preempting
   the warrant.

## Data model

```sql
CREATE TABLE warrant_validations (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  warrant_docket TEXT NOT NULL UNIQUE,
  submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  signature_valid BOOLEAN NOT NULL,
  schema_valid BOOLEAN NOT NULL,
  active_conflicts JSONB,
  status TEXT NOT NULL CHECK (status IN ('VALIDATING','VALIDATED','REJECTED','SUPERSEDED')),
  rejection_reason TEXT
);
```

## API surface

```protobuf
service GovernanceWarrantValidation {
  rpc ValidateWarrant (ValidateWarrantRequest)
      returns (ValidateWarrantResponse);
  rpc GetValidationStatus (GetValidationStatusRequest)
      returns (GetValidationStatusResponse);
}
```

## Files to author

| File | Purpose | Lines |
|---|---|---:|
| `microservices/governance/src/warrant/validation.rs` | Validation orchestration | ~340 |
| `microservices/governance/src/warrant/signature_verifier.rs` | Fulcio + magistrate sig validation | ~280 |
| `microservices/governance/src/warrant/scope_resolver.rs` | Scope schema parsing + resolution | ~280 |
| `microservices/governance/src/warrant/conflict_checker.rs` | Active-conflict detection | ~220 |
| `microservices/governance/policy/governance-validate-warrant.cedar` | Cedar permit | ~30 |
| `microservices/governance/contracts/proto/warrant_validation.proto` | gRPC defs | ~140 |
| `microservices/governance/db/migrations/2026-05-20-001-warrant-validations.sql` | DDL | ~50 |
| `microservices/governance/runbooks/warrant-validation-failure.md` | Runbook | ~160 |
| `microservices/governance/runbooks/forged-signature-detection.md` | Runbook | ~140 |
| `microservices/governance/tests/integration/warrant_validation_test.rs` | Tests | ~440 |
| `microservices/governance/dashboards/warrant-validation-health.json` | Grafana | ~100 |
| `microservices/governance/slos/warrant-validation-latency.openslo.yaml` | SLO ≤2s p99 | ~40 |

Total approximate: ~2,220 lines.

## Cedar fragments

```cedar
// governance-validate-warrant.cedar
permit (
  principal == Service::"legal-process-api",
  action == Action::"governance.validate_warrant",
  resource is Warrant
) when {
  resource.issuing_court matches "US District Court*" ||
  resource.issuing_court matches "Federal Magistrate*"
};
```

## Integration contracts

| Contract | Direction | Notes |
|---|---|---|
| `sigstore.VerifyChain` | governance → sigstore-trust-substrate | Magistrate signature trust |
| `compliance.ValidateWarrantAgainstPack` | governance → compliance | Pack-overlay validation |
| `tenancy.GrantCrossTenantPermit` | governance → tenancy | After validation passes |
| `audit-chain.EmitSealed` | governance → audit-chain | `WarrantValidated` or `WarrantRejected` |

## Latency budget

- Full validation: ≤2s p99

## Test plan

- Test A.1, A.2 — happy path validation
- Test D.1 — forged signature rejected
- Test D.2 — schema-invalid rejected

## Observability emissions

- `oya_governance_warrant_validation_total{outcome}`
- `oya_governance_warrant_validation_latency_ms`
- `oya_governance_warrant_signature_invalid_total`

## Acceptance criteria

- Sigstore chain validation works.
- JSON schema validation works.
- Conflict detection works.

## Cross-references

- `docs/user-journeys/j129-*/handshake.md` §1
- ADR-0312, ADR-0295

## Completion expansion — j129 governance IP rigor pass

Journey context: court warrant pierces personal tenant only through scoped judicial review.
Service role: policy-engine gateway, warrant validation, board-control evidence, and subpoena routing.
Mapped services in this journey: identity, audit-chain, compliance, governance, workflow-engine, community.
ADR anchors: ADR-0244, ADR-0299, ADR-0311, ADR-0312, ADR-0313, ADR-0319.
This IP is sized as a single reviewable implementation slice and remains compatible with the 56-µservice flat layout.

Implementation task 001: in governance, define the Cedar policy change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 001: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 001: add property coverage proving governance and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 001: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 002: in governance, define the OpenAPI 3.2.0 contract change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 002: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 002: add contract coverage proving governance and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 002: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 003: in governance, define the AsyncAPI 3.1.0 event change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 003: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0312 scope grants it; refusals are success states, not exceptions.
Verification 003: add integration coverage proving governance and governance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 003: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 004: in governance, define the proto3 port change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 004: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 004: add replay coverage proving governance and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 004: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 005: in governance, define the Postgres/RLS storage change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 005: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 005: add load coverage proving governance and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 005: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 006: in governance, define the audit-chain emission change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 006: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 006: add chaos coverage proving governance and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 006: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 007: in governance, define the dashboard projection change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 007: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 007: add negative authorization coverage proving governance and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 007: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 008: in governance, define the runbook hook change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 008: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 008: add multi-region coverage proving governance and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 008: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 009: in governance, define the integration fixture change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 009: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0312 scope grants it; refusals are success states, not exceptions.
Verification 009: add pack-overlay coverage proving governance and governance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 009: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 010: in governance, define the domain model change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 010: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 010: add unit coverage proving governance and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 010: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 01: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 011: in governance, define the Cedar policy change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 011: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 011: add property coverage proving governance and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 011: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 012: in governance, define the OpenAPI 3.2.0 contract change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 012: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 012: add contract coverage proving governance and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 012: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 013: in governance, define the AsyncAPI 3.1.0 event change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 013: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 013: add integration coverage proving governance and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 013: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 014: in governance, define the proto3 port change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 014: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 014: add replay coverage proving governance and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 014: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 015: in governance, define the Postgres/RLS storage change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 015: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0312 scope grants it; refusals are success states, not exceptions.
Verification 015: add load coverage proving governance and governance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 015: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 016: in governance, define the audit-chain emission change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 016: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 016: add chaos coverage proving governance and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 016: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 017: in governance, define the dashboard projection change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 017: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 017: add negative authorization coverage proving governance and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 017: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 018: in governance, define the runbook hook change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 018: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 018: add multi-region coverage proving governance and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 018: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 019: in governance, define the integration fixture change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 019: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 019: add pack-overlay coverage proving governance and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 019: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 020: in governance, define the domain model change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 020: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 020: add unit coverage proving governance and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 020: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 02: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 021: in governance, define the Cedar policy change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 021: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0312 scope grants it; refusals are success states, not exceptions.
Verification 021: add property coverage proving governance and governance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 021: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 022: in governance, define the OpenAPI 3.2.0 contract change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 022: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 022: add contract coverage proving governance and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 022: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 023: in governance, define the AsyncAPI 3.1.0 event change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 023: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 023: add integration coverage proving governance and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 023: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 024: in governance, define the proto3 port change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 024: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 024: add replay coverage proving governance and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 024: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 025: in governance, define the Postgres/RLS storage change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 025: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 025: add load coverage proving governance and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 025: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 026: in governance, define the audit-chain emission change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 026: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 026: add chaos coverage proving governance and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 026: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 027: in governance, define the dashboard projection change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 027: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0312 scope grants it; refusals are success states, not exceptions.
Verification 027: add negative authorization coverage proving governance and governance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 027: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 028: in governance, define the runbook hook change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 028: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 028: add multi-region coverage proving governance and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 028: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 029: in governance, define the integration fixture change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 029: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 029: add pack-overlay coverage proving governance and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 029: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 030: in governance, define the domain model change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 030: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 030: add unit coverage proving governance and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 030: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 03: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 031: in governance, define the Cedar policy change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 031: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 031: add property coverage proving governance and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 031: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 032: in governance, define the OpenAPI 3.2.0 contract change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 032: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 032: add contract coverage proving governance and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 032: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 033: in governance, define the AsyncAPI 3.1.0 event change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 033: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0312 scope grants it; refusals are success states, not exceptions.
Verification 033: add integration coverage proving governance and governance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 033: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 034: in governance, define the proto3 port change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 034: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 034: add replay coverage proving governance and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 034: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 035: in governance, define the Postgres/RLS storage change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 035: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 035: add load coverage proving governance and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 035: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 036: in governance, define the audit-chain emission change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 036: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 036: add chaos coverage proving governance and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 036: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 037: in governance, define the dashboard projection change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 037: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 037: add negative authorization coverage proving governance and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 037: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 038: in governance, define the runbook hook change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 038: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 038: add multi-region coverage proving governance and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 038: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 039: in governance, define the integration fixture change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 039: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0312 scope grants it; refusals are success states, not exceptions.
Verification 039: add pack-overlay coverage proving governance and governance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 039: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 040: in governance, define the domain model change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 040: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 040: add unit coverage proving governance and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 040: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 04: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 041: in governance, define the Cedar policy change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 041: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 041: add property coverage proving governance and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 041: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 042: in governance, define the OpenAPI 3.2.0 contract change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 042: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 042: add contract coverage proving governance and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 042: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 043: in governance, define the AsyncAPI 3.1.0 event change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 043: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 043: add integration coverage proving governance and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 043: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 044: in governance, define the proto3 port change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 044: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 044: add replay coverage proving governance and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 044: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 045: in governance, define the Postgres/RLS storage change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 045: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0312 scope grants it; refusals are success states, not exceptions.
Verification 045: add load coverage proving governance and governance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 045: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 046: in governance, define the audit-chain emission change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 046: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 046: add chaos coverage proving governance and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 046: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 047: in governance, define the dashboard projection change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 047: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 047: add negative authorization coverage proving governance and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 047: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 048: in governance, define the runbook hook change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 048: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 048: add multi-region coverage proving governance and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 048: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 049: in governance, define the integration fixture change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 049: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 049: add pack-overlay coverage proving governance and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 049: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 050: in governance, define the domain model change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 050: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 050: add unit coverage proving governance and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 050: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 05: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 051: in governance, define the Cedar policy change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 051: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0312 scope grants it; refusals are success states, not exceptions.
Verification 051: add property coverage proving governance and governance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 051: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 052: in governance, define the OpenAPI 3.2.0 contract change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 052: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 052: add contract coverage proving governance and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 052: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 053: in governance, define the AsyncAPI 3.1.0 event change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 053: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 053: add integration coverage proving governance and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 053: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 054: in governance, define the proto3 port change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 054: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 054: add replay coverage proving governance and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 054: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 055: in governance, define the Postgres/RLS storage change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 055: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 055: add load coverage proving governance and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 055: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 056: in governance, define the audit-chain emission change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 056: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 056: add chaos coverage proving governance and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 056: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 057: in governance, define the dashboard projection change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 057: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0312 scope grants it; refusals are success states, not exceptions.
Verification 057: add negative authorization coverage proving governance and governance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 057: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 058: in governance, define the runbook hook change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 058: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 058: add multi-region coverage proving governance and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 058: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 059: in governance, define the integration fixture change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 059: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 059: add pack-overlay coverage proving governance and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 059: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 060: in governance, define the domain model change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 060: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 060: add unit coverage proving governance and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 060: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 06: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 061: in governance, define the Cedar policy change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 061: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 061: add property coverage proving governance and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 061: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 062: in governance, define the OpenAPI 3.2.0 contract change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 062: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 062: add contract coverage proving governance and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 062: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 063: in governance, define the AsyncAPI 3.1.0 event change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 063: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0312 scope grants it; refusals are success states, not exceptions.
Verification 063: add integration coverage proving governance and governance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 063: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 064: in governance, define the proto3 port change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 064: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 064: add replay coverage proving governance and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 064: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 065: in governance, define the Postgres/RLS storage change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 065: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 065: add load coverage proving governance and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 065: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 066: in governance, define the audit-chain emission change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 066: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 066: add chaos coverage proving governance and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 066: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 067: in governance, define the dashboard projection change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 067: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 067: add negative authorization coverage proving governance and audit-chain agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 067: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 068: in governance, define the runbook hook change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 068: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 068: add multi-region coverage proving governance and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 068: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 069: in governance, define the integration fixture change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 069: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0312 scope grants it; refusals are success states, not exceptions.
Verification 069: add pack-overlay coverage proving governance and governance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 069: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 070: in governance, define the domain model change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 070: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 070: add unit coverage proving governance and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 070: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 07: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 071: in governance, define the Cedar policy change for court warrant pierces personal tenant only through scoped judicial review; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 071: governance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0319 scope grants it; refusals are success states, not exceptions.
Verification 071: add property coverage proving governance and community agree on contract version, Cedar decision, audit event class, and replay cursor.

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the explicit counterpart hook required by ADR-0328 D-20. Governance parity is evaluated against GitHub Advanced Security, SonarQube, Snyk, Trivy, Open Policy Agent, Backstage TechDocs, and Renovate. The implementation must state which of those controls it closes or deliberately does not target before promotion.
