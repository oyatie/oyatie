// FD-001 shared platform contracts — PBAC template seed.
// Policy-as-data: the control plane links this template per grant
// (?principal, ?resource), instead of authoring ad-hoc policies. Precedent:
// Amazon Verified Permissions policy templates. The structural
// tenant-isolation forbid in platform-policies.cedar overrides every link,
// so a template instantiation can never grant across a tenant boundary.
permit (
  principal == ?principal,
  action == OyaPlatform::Action::"ReadResource",
  resource == ?resource
);
