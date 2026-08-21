// Cedar 4.12 fragment — cross-tenant-isolation
// Capability: policy (ADR-0615 §5, 24th registered capability)
// Binding ADRs: ADR-0280 §D-13.D, ADR-0615 §5
// Fragment soak window: >= 60s per ADR-0294
// Fragment version: 1.0.0
// Reviewed: 2026-08-20
//
// F-CROSS-TENANT. Defence in depth: every permit in this bundle already binds the tenant, and this
// forbid makes a permit that FORGETS to bind it non-exploitable. Applies to every action and every
// resource type in the namespace — all six resource entities carry `tenant_id` by schema.
//
// Note on shape: this file deliberately does NOT open with a bare
// `forbid (principal, action, resource);`. Cedar is default-deny by construction, so a bare forbid
// adds no denial — but because forbid unconditionally overrides permit, in the CONSOLIDATED bundle
// (cedar/policies.cedar) it would nullify every permit in every other fragment. See
// policy/cedar/README.md for the executable demonstration.

// F1 — a principal may never act on another tenant's authorization state.
forbid (
  principal,
  action,
  resource
)
when { principal.tenant_id != resource.tenant_id };
