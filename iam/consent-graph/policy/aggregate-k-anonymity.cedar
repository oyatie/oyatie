// Aggregate-mode k-anonymity enforcement policy
// Authority: ADR-SVC-CG-003, IP-011 §5.

// 1. Permit aggregate.read when agreement is in Aggregate mode + matching scope
permit (
  principal in Tenant::"{{grantee_tenant_id}}",
  action == Action::"aggregate.read",
  resource is Resource
)
when {
  resource.entity_type == "{{entity_type}}" &&
  resource.grantor_tenant_id == "{{grantor_tenant_id}}" &&
  context.purpose_of_use == "{{purpose_of_use}}" &&
  context has "k_observed" &&
  context.k_observed >= {{k_anonymity}}
};

// 2. Forbid below-k buckets (grantor side suppresses, but defense-in-depth here)
forbid (
  principal,
  action == Action::"aggregate.read",
  resource is Resource
)
when {
  context has "k_observed" &&
  context.k_observed < {{k_anonymity}}
};

// 3. Forbid DP-noise-disabled requests if agreement requires DP
forbid (
  principal,
  action == Action::"aggregate.read",
  resource is Resource
)
when {
  context has "dp_required" &&
  context.dp_required == true &&
  (!(context has "dp_noise_applied") ||
   context.dp_noise_applied != true)
};

// 4. Forbid windows older than 1y (aggregate freshness ceiling)
forbid (
  principal,
  action == Action::"aggregate.read",
  resource is Resource
)
when {
  context has "window_end_iso" &&
  context.request_time > datetime(context.window_end_iso) &&
  duration(context.request_time, datetime(context.window_end_iso)) > duration("365d")
};

// 5. Forbid sensitive cohorts with k<10 (per compliance.md sensitive categories)
forbid (
  principal,
  action,
  resource
)
when {
  context has "is_sensitive_category" &&
  context.is_sensitive_category == true &&
  context has "k_observed" &&
  context.k_observed < 10
};

// 6. Forbid if differential-privacy budget exhausted
forbid (
  principal,
  action,
  resource
)
when {
  context has "dp_budget_remaining" &&
  context.dp_budget_remaining <= 0.0
};
