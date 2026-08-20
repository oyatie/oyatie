// Cedar v4.2 LTS fragment — experiment-design-authorization
// Microservice: feature-flags
// Binding ADR: ADR-0243 + ADR-0159
// Fragment soak window: ≥60s per ADR-0294
// Fragment version: 1.0.0
// Reviewed: 2026-05-20

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: product-manager or experiment-designer may create experiments
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal in Role::"experiment-designer",
  action == Action::"ExperimentCreate",
  resource is Experiment
)
when {
  principal.tenant_id == resource.tenant_id
  && principal.mfa_verified == true
};

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: experiment-designer may activate experiments — step-up Class B
// Activation routes live traffic; higher risk than creation
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal in Role::"experiment-designer",
  action == Action::"ExperimentActivate",
  resource is Experiment
)
when {
  principal.tenant_id == resource.tenant_id
  && principal.mfa_verified == true
  && context.step_up_class >= 2    // Class B: affects live traffic
  && resource.lifecycle_state == "draft"
  && resource.has_sample_size_estimate == true  // must have power analysis before activation
};

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: experiment-designer may conclude experiments
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal in Role::"experiment-designer",
  action == Action::"ExperimentConclude",
  resource is Experiment
)
when {
  principal.tenant_id == resource.tenant_id
  && principal.mfa_verified == true
  && resource.lifecycle_state == "active"
};

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: any principal with FlagEvaluate permit may read experiment assignments
// (read-only; evaluation context only)
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal,
  action == Action::"ExperimentAssignmentRead",
  resource is ExperimentAssignment
)
when {
  principal.tenant_id == resource.tenant_id
};

// ─────────────────────────────────────────────────────────────────────────────
// FORBID: cross-tenant experiment reads
// ─────────────────────────────────────────────────────────────────────────────
forbid (
  principal,
  action in [Action::"ExperimentCreate", Action::"ExperimentActivate", Action::"ExperimentConclude", Action::"ExperimentAssignmentRead"],
  resource
)
unless {
  principal.tenant_id == resource.tenant_id
};

// ─────────────────────────────────────────────────────────────────────────────
// FORBID: experiments on EMERGENCY_SERVICES audience_type require explicit approval
// Emergency-services surfaces must not be used as A/B experiment subjects
// ─────────────────────────────────────────────────────────────────────────────
forbid (
  principal,
  action == Action::"ExperimentActivate",
  resource is Experiment
)
when {
  resource.target_audience_type == "EMERGENCY_SERVICES"
  && !(principal in Role::"platform-safety-officer")
};

// ─────────────────────────────────────────────────────────────────────────────
// FORBID: activation blocked if minor-protection requirements not met
// Per ADR-0292: experiments targeting MINOR audience require compliance declaration
// ─────────────────────────────────────────────────────────────────────────────
forbid (
  principal,
  action == Action::"ExperimentActivate",
  resource is Experiment
)
when {
  resource.target_audience_type == "MINOR_TARGETED"
  && resource.minor_compliance_pack == ""
};
