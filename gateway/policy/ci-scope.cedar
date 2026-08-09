// api-gateway µservice — CI-scope fragment
// Cedar v4.2 LTS
// Authority: ADR-0295 (bootstrap CI SPIFFE + kill-switch).

// =============================================================================
// PERMIT — CI lane SPIFFE SVID reads route config
// =============================================================================

permit (
  principal in CIPrincipal::?p,
  action in [
    Action::"Route::list",
    Action::"Route::describe",
    Action::"CanaryWeight::read",
    Action::"BlueGreen::read"
  ],
  resource
)
when {
  principal has spiffe_id &&
  principal.spiffe_id like "spiffe://oyatie.ci/*" &&
  principal has svid_age_seconds &&
  principal.svid_age_seconds < 900
};

// =============================================================================
// PERMIT — CI canary lane shifts weight (write, ≤5% delta)
// =============================================================================

permit (
  principal in CIPrincipal::?p,
  action == Action::"CanaryWeight::shift",
  resource in CanaryWeightResource::?r
)
when {
  principal has spiffe_id &&
  principal.spiffe_id like "spiffe://oyatie.ci/canary-shifter/*" &&
  resource has weight_delta_percent &&
  resource.weight_delta_percent <= 5 &&
  principal has approval_count &&
  principal.approval_count >= 1
};

// =============================================================================
// FORBID — CI lane mutates production-blue-green without dual approval
// =============================================================================

forbid (
  principal in CIPrincipal::?p,
  action == Action::"BlueGreen::swap",
  resource in BlueGreenResource::?r
)
when {
  resource has environment &&
  resource.environment == "production" &&
  principal has approval_count &&
  principal.approval_count < 2
};

// =============================================================================
// FORBID — CI lane attempting to bypass kill-switch
// =============================================================================

forbid (
  principal in CIPrincipal::?p,
  action,
  resource
)
when {
  principal has kill_switch_state &&
  principal.kill_switch_state == "engaged"
};

// =============================================================================
// FORBID — CI lane SVID older than 15 min
// =============================================================================

forbid (
  principal in CIPrincipal::?p,
  action,
  resource
)
when {
  principal has svid_age_seconds &&
  principal.svid_age_seconds > 900
};
