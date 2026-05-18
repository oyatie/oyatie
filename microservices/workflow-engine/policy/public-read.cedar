// workflow-engine µservice — public-read policy fragment
// Cedar v4 schema
//
// Purpose: define what (if anything) is publicly readable from the workflow-engine
//          µservice. Default posture: NOTHING is publicly readable. This file exists
//          to make that explicit and to be auditable.
// Owner:   ops-security + council-privacy + axis-workflow
// References: threat-model.md §"T-I-04" (replay-debugger leak) + DPIA R-13
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny — public access is refused.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT — anonymous schema validation (OpenSLO/OpenAPI-class)
// =============================================================================
//
// The workflow-engine exposes anonymous schema validation for workflow specs
// to enable tenant onboarding tooling. Validation returns shape correctness;
// it does NOT execute the spec or persist any state.

permit (
  principal == AnonymousPrincipal::"anonymous",
  action == Action::"validate_workflow_spec_schema",
  resource == SchemaValidationEndpoint::"workflow_spec_schema_v1"
);

// =============================================================================
// PERMIT — public status (engine health, no tenant data)
// =============================================================================
//
// Engine health endpoint returns up/down + version + cluster-aggregate active runs;
// no per-tenant data. This is consumed by status.oyatie.dev.

permit (
  principal == AnonymousPrincipal::"anonymous",
  action == Action::"read_public_health",
  resource == PublicEndpoint::"engine_health"
);

// =============================================================================
// FORBID — explicitly nothing else is publicly readable
// =============================================================================

forbid (
  principal == AnonymousPrincipal::"anonymous",
  action in [
    Action::"read_workflow_spec",
    Action::"read_workflow_run",
    Action::"read_step_execution",
    Action::"read_workflow_event",
    Action::"read_replay_session",
    Action::"read_run_analytics",
    Action::"list_workflow_specs",
    Action::"list_workflow_runs",
    Action::"submit_workflow_spec",
    Action::"start_workflow_run",
    Action::"publish_workflow_event",
    Action::"subscribe_workflow_event",
    Action::"replay_workflow_run"
  ],
  resource
);
