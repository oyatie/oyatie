// Break-glass overlay for us-healthcare pack: emergency-treatment purpose
// Authority: break-glass.md §2.1, HIPAA 45 CFR §164.512(a)(b).
// Compiled on break-glass session open; expires at session.expiry; revoked by audit-officer review.

// 1. Permit emergency-treatment read with mandatory advice
permit (
  principal in Tenant::"{{grantee_tenant_id}}",
  action in [Action::"project.read", Action::"attested.query"],
  resource is Resource
)
when {
  context.purpose_of_use == "emergency-treatment" &&
  context has "break_glass_session_id" &&
  context has "valid_until_iso" &&
  context.request_time <= datetime(context.valid_until_iso) &&
  resource.entity_type in [
    "PatientEligibility",
    "PatientCoverage",
    "PatientCareTeam"
  ]
}
advice {
  alert: "break-glass-review-required",
  alert_target: "audit-officer@{{grantor_tenant_id}}",
  severity: "P1"
};

// 2. Two-person rule: forbid until counter-signer attestation present
forbid (
  principal,
  action,
  resource
)
when {
  context.purpose_of_use == "emergency-treatment" &&
  (!(context has "counter_signer_attested") ||
   context.counter_signer_attested != true)
};

// 3. Forbid above per-session read cap (default 1000)
forbid (
  principal,
  action,
  resource
)
when {
  context.purpose_of_use == "emergency-treatment" &&
  context has "session_read_count" &&
  context.session_read_count >= 1000
};

// 4. Forbid after 24h session expiry
forbid (
  principal,
  action,
  resource
)
when {
  context has "valid_until_iso" &&
  context.request_time > datetime(context.valid_until_iso)
};

// 5. Forbid if agreement is revoked (break-glass cannot resurrect revoked grants)
forbid (
  principal,
  action,
  resource
)
when {
  context has "agreement_state" &&
  (context.agreement_state == "Revoked" || context.agreement_state == "Expired")
};
