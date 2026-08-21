// Cedar 4.12 fragment — cross-tenant-isolation
// Capability: policy (ADR-0615 §5, 24th registered capability)
// Binding ADRs: ADR-0280 §D-13.D, ADR-0615 §5
// Fragment soak window: >= 60s per ADR-0294
// Fragment version: 2.0.0
// Reviewed: 2026-08-20 (adversarial review; v1.0.0 findings S1-S6 closed)
//
// F1 — a principal may never act on another tenant's authorization state.
//
// This is MEMBERSHIP (`principal in resource.owner_tenant`), not a string comparison. v1.0.0 wrote
// `principal.tenant_id != resource.tenant_id`, which an adversarial review defeated in one step: a
// principal whose entity parent was tenant A but whose `tenant_id` attribute said B was authorized
// against B, and this forbid — reading the same caller-supplied field as the permit it backstops —
// did not fire. A backstop derived from the same untrusted input as the thing it guards is not a
// backstop. `owner_tenant` is an entity reference and membership is structural.
//
// Note it is `unless`, not `when`: a request whose principal is in NO tenant is forbidden, rather
// than slipping through a `!=` that never becomes true.
@id("F1")
forbid (
  principal,
  action,
  resource
)
unless { principal in resource.owner_tenant };
