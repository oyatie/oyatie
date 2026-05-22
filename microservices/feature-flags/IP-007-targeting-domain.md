# IP-007 — Targeting Domain Crate

**microservice**: feature-flags
**bc**: targeting
**layer**: domain
**crate**: oya-feature-flags-targeting-domain
**status**: design-ready
**acceptance_status**: design-ready
**adrs**: ADR-0105, ADR-0131, ADR-0159, ADR-0218, ADR-0243, ADR-0244, ADR-0248, ADR-0263, ADR-0284
**companion_ips**: IP-006, IP-004

## Scope

Targeting rule management: CRUD for rules, cohort definitions, segment imports, rule validation, conflict detection. Enforces per-tenant rule budget. Emits audit events for rule mutations.

## Deliverables

| # | Artifact | Acceptance Criterion |
|---|----------|---------------------|
| 1 | `TargetingRuleRepository` trait | CRUD + list with cursor pagination; tenant-scoped |
| 2 | `TargetingRuleValidationService` | Validates rule syntax against `CedarPredicateEvaluator`; rejects malformed predicates |
| 3 | `CohortDefinitionService` | Create/update/delete cohort definitions; max 500 cohorts per tenant (per-tenant budget) |
| 4 | `RuleConflictDetector` | Detects overlapping rules that would produce non-deterministic evaluation order; emits warning event |
| 5 | Audit events | `TargetingRuleCreated`, `TargetingRuleUpdated`, `TargetingRuleDeleted`, `CohortUpdated` |
| 6 | Platform-owner name indirection | No hard-coded "oyatie" display strings per ADR-0284; all user-visible strings via localization keys |
| 7 | Tests | Conflict detection test; per-tenant rule budget enforcement (rule 501 returns `QuotaExceeded`) |

## Definition of Done

- `cargo test -p oya-feature-flags-targeting-domain` green
- Rule budget enforced: tenant cannot create >500 cohorts
- All user-facing strings localized (no "oyatie" literal in display fields)
- Audit events emitted for all 4 mutation types
