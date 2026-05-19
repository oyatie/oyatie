// Cedar policy: auto-assign-employment-context
// Microservice: tasks
// Purpose: refuse T2 auto-assign when work allocation is in employment context
// until a per-pack conformity-assessment artifact is on file.
// Owner: axis-tasks + council-privacy + council-ai-safety
// Status: Accepted (2026-05-19)
// Related: ADR-TASKS-0006, capabilities/T2-auto.yaml
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =====================================================================================
// REGULATED-AI EMPLOYMENT-CONTEXT AUTO-ASSIGN REFUSAL.
// =====================================================================================

forbid (
  principal,
  action == Action::"TaskT2AutoAssign",
  resource
)
when {
  principal.tenant_id == resource.tenant_id &&
  context.employment_context == true &&
  context.context_kind == "employment" &&
  !(context has auto_assign_conformity_assessment_evidence_on_file &&
    context.auto_assign_conformity_assessment_evidence_on_file == true)
};
