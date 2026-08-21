// Cedar 4.12 fragment — runtime-grants
// Capability: policy (ADR-0615 §5, 24th registered capability)
// Binding ADRs: ADR-0280 §D-13.D/E (C0 face), ADR-0191 (edge vs origin)
// Fragment soak window: >= 60s per ADR-0294
// Fragment version: 1.0.0
// Reviewed: 2026-08-20
//
// The C0-face permits: what the per-cell runtime and its distributor may do. These are the hot
// paths, and they are the ones static-stability.cedar constrains.

// P6 — an attested PEP workload evaluates against its cell-local snapshot.
permit (
  principal,
  action == OyaPolicy::Action::"EvaluateAgainstSnapshot",
  resource
)
when {
  principal.role == "pep-workload" &&
  principal.tenant_id == resource.tenant_id &&
  principal.spiffe_id != ""
};

// P7 — an attested PEP workload reads ReBAC tuples for its own tenant.
permit (
  principal,
  action == OyaPolicy::Action::"ReadRebacTuple",
  resource
)
when {
  principal.role == "pep-workload" &&
  principal.tenant_id == resource.tenant_id &&
  principal.spiffe_id != ""
};

// P8 — the distributor propagates signed snapshots. F4 stops it propagating unverified ones.
permit (
  principal,
  action == OyaPolicy::Action::"DistributeSnapshot",
  resource
)
when {
  principal.role == "policy-distributor" &&
  principal.tenant_id == resource.tenant_id
};
