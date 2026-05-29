---
doc_class: Implementation-Plan
ip_id: IP-journey-j133-rif-compliance-and-litigation-hold
journey_ref: docs/user-journeys/j133-hr-conducts-layoff-with-dignity-and-compliance/
status: draft
date: 2026-05-20
microservice: compliance
related_adrs: [ADR-0311, ADR-0244, ADR-0263]
---

# IP — Compliance's role in j133 RIF compliance + litigation hold

## Scope

Compliance is the regulatory authority for the RIF cascade. Provides:
- Disparate-impact analysis fairness gate (calls intelligence; verdict required green)
- Per-jurisdiction pack overlay resolution (WARN, OWBPA, KSchG, KR LSA, IN ID Act)
- OWBPA 21-day consider window + 7-day revoke window enforcement
- Litigation-hold flagging (apply + lift)
- Pre-execution clearance check (preflight before rif_execute)
- Per-jurisdiction citation injection into mail templates

## Acceptance criteria

1. `POST /compliance/rif/preflight` API.
2. `POST /compliance/litigation-hold/apply` + `POST /compliance/litigation-hold/lift` APIs.
3. `POST /compliance/owbpa-window/check` API for ≥40 US-AUS cohort.
4. Per-jurisdiction overlay resolution.
5. SLO: P95 preflight ≤ 250ms.

## Atomic deliverables

| Step | Change | Verification |
|---|---|---|
| 1 | Register pack-us-warn-act + pack-us-owbpa + pack-us-litigation-hold-baseline | Pack tests pass |
| 2 | Implement `POST /compliance/rif/preflight` | T-101..T-105 pass |
| 3 | Implement `POST /compliance/litigation-hold/apply` | T-701 passes |
| 4 | Implement `POST /compliance/litigation-hold/lift` | T-703 passes |
| 5 | Implement `POST /compliance/owbpa-window/check` | T-203 passes |
| 6 | Implement per-jurisdiction overlay resolver | T-001 sub-step passes |
| 7 | Wire audit-chain: RifComplianceCleared + LitigationHoldApplied + LitigationHoldLifted + OwbpaWindowChecked + RifJurisdictionOverlayResolved | Registry green |

## APIs

### `POST /compliance/rif/preflight`

- Body: `{event_id, tenant_id, model_ref (DEI scorer)}`
- Cedar: `b2b.compliance.rif_preflight`
- Response: `{verdict: PASS|FAIL|PARTIAL, issues[]}`

Preflight checks:
- pack-us-warn-act ACTIVE
- pack-us-owbpa ACTIVE (if any ≥40 US-AUS in cohort)
- pack-eu-anti-discrimination-baseline ACTIVE
- pack-de-kschg-baseline ACTIVE (if any DE-BER in cohort)
- pack-kr-labor-standards-act-amendment ACTIVE (if any KR-SEO)
- pack-in-industrial-disputes-act ACTIVE (if any IN-BLR)
- pack-us-litigation-hold-baseline ACTIVE
- All conformity certificates valid
- DEI model in PRODUCTION stage with green baseline

### `POST /compliance/litigation-hold/apply`

- Body: `{employee_principal, hold_reason, hold_scope, hold_duration_days}`
- Cedar: `b2b.compliance.litigation_hold_apply`
- Response: `{hold_id, applied_at, expires_at}`

### `POST /compliance/litigation-hold/lift`

- Body: `{hold_id, lift_reason}`
- Cedar: `b2b.compliance.litigation_hold_lift`
- Response: `{lift_id, lifted_at}`

### `POST /compliance/owbpa-window/check`

- Body: `{employee_principal, mutual_release_offer_date}`
- Cedar: (internal)
- Response: `{consider_window_expires_at, revoke_window_expires_at, window_compliance: PASS|FAIL}`

## Cedar permits

```cedar
// b2b.compliance.rif_preflight.cedar
permit (
  principal,
  action == Action::"b2b.compliance.rif_preflight",
  resource is RifEvent
) when {
  principal.audience_type in ["B2B_HR_ADMIN", "oyatie:workflow-engine:internal"]
};
```

```cedar
// b2b.compliance.litigation_hold_apply.cedar
permit (
  principal,
  action == Action::"b2b.compliance.litigation_hold_apply",
  resource is EmployeeRecord
) when {
  principal in [User::"naomi-legal@marcus-tenant.legal"] &&
  context.litigation_anticipated_documented == true &&
  context.tenant.compliance_pack_active("pack-us-litigation-hold-baseline") &&
  context.audit_session_open == true
};
```

## Pack registry

| Pack ID | Jurisdiction | Source |
|---|---|---|
| pack-us-warn-act | US | WARN Act 1988 |
| pack-us-owbpa | US | Older Workers Benefit Protection Act 1990 |
| pack-us-title-vii-baseline | US | Civil Rights Act 1964 Title VII |
| pack-us-litigation-hold-baseline | US | Federal Rules of Civil Procedure 37(e) |
| pack-de-kschg-baseline | DE | Kündigungsschutzgesetz |
| pack-eu-works-council-baseline | EU | Directive 2009/38/EC |
| pack-eu-anti-discrimination-baseline | EU | Directive 2000/78/EC + 2000/43/EC |
| pack-kr-labor-standards-act-amendment | KR | LSA 2026 amendment |
| pack-in-industrial-disputes-act | IN | ID Act 1947 |

## Dependencies

- **intelligence** (DEI scorer)
- **identity** (employee record lookup)
- **drive** (retention-pack enforcement; litigation-hold suspension)
- **tenancy** (works-council pack lookup)
- **audit-chain** (EmitSealed)
- **workflow-engine** (orchestration)

## Observability

| Metric | Type | Labels |
|---|---|---|
| `oya_compliance_rif_preflight_total` | counter | tenant_id, verdict |
| `oya_compliance_litigation_hold_applied_total` | counter | jurisdiction |
| `oya_compliance_owbpa_window_check_total` | counter | verdict |
| `oya_compliance_overlay_resolve_ms` | histogram | jurisdiction |

## SLOs

- P50 preflight: 100ms; P95: 250ms
- P50 overlay resolve: 50ms; P95: 150ms
- Litigation-hold apply: > 99.99% success
- OWBPA window check: 100% correctness

## Failure modes

| Failure | Recovery |
|---|---|
| Pack version drift | Use cascade-start hash-pinned version |
| DEI scorer unavailable | Preflight FAIL; halt; alert |
| Litigation-hold pack missing | Halt hold; alert Naomi |
| Conformity certificate expired | FAIL; halt cascade |

## Test gates

- T-101..T-105 (preflight)
- T-203 (OWBPA window)
- T-701, T-702, T-703 (litigation hold)
- T-901 (per-jurisdiction citation)

## Notes

- Per ADR-0311, litigation hold applies to tenant-owned data only; personal-tenant data is NOT in scope without ADR-0312 court warrant.
- Per ADR-0244, compliance recognizes B2B_HR_ADMIN audience-type for RIF preflight.
- Per ADR-0263, every preflight + hold event sealed.

— end of IP —

## Completion expansion — j133 compliance IP rigor pass

Journey context: 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade.
Service role: pack overlay, regulator mapping, legal basis matrix, and retention policy composition.
Mapped services in this journey: workflow-engine, mail, messenger, payments, finops-portal, identity, tenancy, community, drive, compliance.
ADR anchors: ADR-0244, ADR-0299, ADR-0311, ADR-0313, ADR-0317, ADR-0320.
This IP is sized as a single reviewable implementation slice and remains compatible with the 56-µservice flat layout.

Implementation task 001: in compliance, define the Cedar policy change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 001: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 001: add property coverage proving compliance and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 001: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 002: in compliance, define the OpenAPI 3.2.0 contract change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 002: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 002: add contract coverage proving compliance and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 002: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 003: in compliance, define the AsyncAPI 3.1.0 event change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 003: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 003: add integration coverage proving compliance and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 003: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 004: in compliance, define the proto3 port change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 004: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 004: add replay coverage proving compliance and finops-portal agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 004: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 005: in compliance, define the Postgres/RLS storage change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 005: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 005: add load coverage proving compliance and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 005: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 006: in compliance, define the audit-chain emission change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 006: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 006: add chaos coverage proving compliance and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 006: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 007: in compliance, define the dashboard projection change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 007: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 007: add negative authorization coverage proving compliance and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 007: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 008: in compliance, define the runbook hook change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 008: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 008: add multi-region coverage proving compliance and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 008: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 009: in compliance, define the integration fixture change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 009: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 009: add pack-overlay coverage proving compliance and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 009: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 010: in compliance, define the domain model change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 010: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 010: add unit coverage proving compliance and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 010: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 01: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 011: in compliance, define the Cedar policy change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 011: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 011: add property coverage proving compliance and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 011: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 012: in compliance, define the OpenAPI 3.2.0 contract change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 012: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 012: add contract coverage proving compliance and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 012: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 013: in compliance, define the AsyncAPI 3.1.0 event change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 013: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 013: add integration coverage proving compliance and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 013: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 014: in compliance, define the proto3 port change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 014: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 014: add replay coverage proving compliance and finops-portal agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 014: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 015: in compliance, define the Postgres/RLS storage change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 015: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 015: add load coverage proving compliance and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 015: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 016: in compliance, define the audit-chain emission change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 016: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 016: add chaos coverage proving compliance and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 016: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 017: in compliance, define the dashboard projection change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 017: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 017: add negative authorization coverage proving compliance and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 017: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 018: in compliance, define the runbook hook change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 018: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 018: add multi-region coverage proving compliance and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 018: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 019: in compliance, define the integration fixture change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 019: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 019: add pack-overlay coverage proving compliance and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 019: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 020: in compliance, define the domain model change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 020: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 020: add unit coverage proving compliance and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 020: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 02: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 021: in compliance, define the Cedar policy change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 021: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 021: add property coverage proving compliance and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 021: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 022: in compliance, define the OpenAPI 3.2.0 contract change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 022: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 022: add contract coverage proving compliance and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 022: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 023: in compliance, define the AsyncAPI 3.1.0 event change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 023: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 023: add integration coverage proving compliance and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 023: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 024: in compliance, define the proto3 port change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 024: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 024: add replay coverage proving compliance and finops-portal agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 024: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 025: in compliance, define the Postgres/RLS storage change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 025: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 025: add load coverage proving compliance and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 025: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 026: in compliance, define the audit-chain emission change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 026: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 026: add chaos coverage proving compliance and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 026: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 027: in compliance, define the dashboard projection change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 027: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 027: add negative authorization coverage proving compliance and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 027: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 028: in compliance, define the runbook hook change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 028: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 028: add multi-region coverage proving compliance and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 028: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 029: in compliance, define the integration fixture change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 029: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 029: add pack-overlay coverage proving compliance and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 029: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 030: in compliance, define the domain model change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 030: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 030: add unit coverage proving compliance and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 030: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 03: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 031: in compliance, define the Cedar policy change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 031: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 031: add property coverage proving compliance and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 031: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 032: in compliance, define the OpenAPI 3.2.0 contract change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 032: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 032: add contract coverage proving compliance and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 032: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 033: in compliance, define the AsyncAPI 3.1.0 event change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 033: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 033: add integration coverage proving compliance and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 033: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 034: in compliance, define the proto3 port change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 034: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 034: add replay coverage proving compliance and finops-portal agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 034: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 035: in compliance, define the Postgres/RLS storage change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 035: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 035: add load coverage proving compliance and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 035: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 036: in compliance, define the audit-chain emission change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 036: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 036: add chaos coverage proving compliance and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 036: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 037: in compliance, define the dashboard projection change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 037: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 037: add negative authorization coverage proving compliance and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 037: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 038: in compliance, define the runbook hook change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 038: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 038: add multi-region coverage proving compliance and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 038: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 039: in compliance, define the integration fixture change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 039: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 039: add pack-overlay coverage proving compliance and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 039: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 040: in compliance, define the domain model change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 040: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 040: add unit coverage proving compliance and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 040: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 04: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 041: in compliance, define the Cedar policy change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 041: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 041: add property coverage proving compliance and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 041: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 042: in compliance, define the OpenAPI 3.2.0 contract change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 042: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 042: add contract coverage proving compliance and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 042: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 043: in compliance, define the AsyncAPI 3.1.0 event change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 043: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 043: add integration coverage proving compliance and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 043: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 044: in compliance, define the proto3 port change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 044: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 044: add replay coverage proving compliance and finops-portal agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 044: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 045: in compliance, define the Postgres/RLS storage change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 045: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 045: add load coverage proving compliance and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 045: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 046: in compliance, define the audit-chain emission change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 046: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 046: add chaos coverage proving compliance and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 046: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 047: in compliance, define the dashboard projection change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 047: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 047: add negative authorization coverage proving compliance and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 047: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 048: in compliance, define the runbook hook change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 048: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 048: add multi-region coverage proving compliance and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 048: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 049: in compliance, define the integration fixture change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 049: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 049: add pack-overlay coverage proving compliance and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 049: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 050: in compliance, define the domain model change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 050: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 050: add unit coverage proving compliance and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 050: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 05: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 051: in compliance, define the Cedar policy change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 051: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 051: add property coverage proving compliance and mail agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 051: document observability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 052: in compliance, define the OpenAPI 3.2.0 contract change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 052: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 052: add contract coverage proving compliance and messenger agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 052: document scalability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 053: in compliance, define the AsyncAPI 3.1.0 event change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 053: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 053: add integration coverage proving compliance and payments agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 053: document performance impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 054: in compliance, define the proto3 port change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 054: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 054: add replay coverage proving compliance and finops-portal agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 054: document optimization impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 055: in compliance, define the Postgres/RLS storage change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 055: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.
Verification 055: add load coverage proving compliance and identity agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 055: document code quality impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 056: in compliance, define the audit-chain emission change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 056: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0311 scope grants it; refusals are success states, not exceptions.
Verification 056: add chaos coverage proving compliance and tenancy agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 056: document security impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 057: in compliance, define the dashboard projection change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 057: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0313 scope grants it; refusals are success states, not exceptions.
Verification 057: add negative authorization coverage proving compliance and community agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 057: document privacy impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 058: in compliance, define the runbook hook change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 058: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0317 scope grants it; refusals are success states, not exceptions.
Verification 058: add multi-region coverage proving compliance and drive agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 058: document residency impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 059: in compliance, define the integration fixture change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 059: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0320 scope grants it; refusals are success states, not exceptions.
Verification 059: add pack-overlay coverage proving compliance and compliance agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 059: document rollback impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Implementation task 060: in compliance, define the domain model change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 060: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0244 scope grants it; refusals are success states, not exceptions.
Verification 060: add unit coverage proving compliance and workflow-engine agree on contract version, Cedar decision, audit event class, and replay cursor.
Rigor 060: document maintainability impact, capacity bound, rollback behavior, SLO signal, and operator evidence needed for an intern to implement safely.
Contract checkpoint 06: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, and audit-chain schemas must be validated before the slice can be marked done.
Implementation task 061: in compliance, define the Cedar policy change for 200-person RIF with dignity, jurisdiction overlays, severance, and offboarding cascade; inputs include tenant_id, actor_id, audience_type, purpose, case_id, idempotency_key, and traceparent.
Boundary rule 061: compliance MUST refuse cross-tenant or personal-surface access unless an explicit ADR-0299 scope grants it; refusals are success states, not exceptions.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/compliance/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/compliance/IP-journey-j133-rif-compliance-and-litigation-hold.md` matched `SLO, multi-region, payment`; anchors `microservices/compliance/runbooks/phi-access-anomaly.md, crates/oya-shared-compliance-evidence-kernel/src/lib.rs`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/compliance/IP-journey-j133-rif-compliance-and-litigation-hold.md` matched `finops, emission`; anchors `microservices/compliance/manifest.json, crates/oya-shared-compliance-evidence-kernel/src/lib.rs`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.
