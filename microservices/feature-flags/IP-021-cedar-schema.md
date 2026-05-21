# IP-021 — Cedar Schema (policy/schema.cedarschema)

**microservice**: feature-flags
**bc**: policy
**layer**: policy
**status**: design-ready
**acceptance_status**: design-ready
**adrs**: ADR-0105, ADR-0131, ADR-0183, ADR-0243, ADR-0244, ADR-0294
**companion_ips**: IP-018

## Scope

Canonical Cedar schema file defining all entity types (Principal, Flag, Experiment, KillSwitch, PackOverride, AuditRecord, Rollout) and all actions (21 total) used across the 10 Cedar policy fragments.

## Deliverables

| # | Artifact | Acceptance Criterion |
|---|----------|---------------------|
| 1 | `policy/schema.cedarschema` | Validates all 10 Cedar fragments: `cedar validate --schema policy/schema.cedarschema policy/*.cedar` exits 0 |
| 2 | Entity types | Principal (role, step_up_class, spiffe_id, pack_overlay_agent_attestation), Flag (pack_locked_fields, is_emergency_services_flag), Experiment (has_sample_size_estimate, audience_type), KillSwitch (is_life_safety_locked), PackOverride (disables_emergency_services), AuditRecord (audit_window), Rollout (slo_gate_passed, slo_breach_detected) |
| 3 | Actions (21) | FlagCreate/Read/Evaluate/Update/Archive/Delete/Undo; ExperimentCreate/Read/Activate/Pause/Conclude; KillSwitchEngage/Disengage; PackOverrideApply/Read; AuditRead/Export; RolloutAdvance/Rollback |
| 4 | CI gate | `lean-a7-cedar-fragments` runs `cedar validate` on all fragments; gate green closes F-2026-05-20-010 |

## Definition of Done

- `cedar validate --schema policy/schema.cedarschema policy/*.cedar` exits 0 for all 10 fragments
- Scorecard override `lean-a7-cedar-fragments` status changes from DEFERRED to PASSING
- `scorecards/overrides.json` updated to reflect PASSING status
