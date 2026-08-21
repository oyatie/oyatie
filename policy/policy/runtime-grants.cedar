// Cedar 4.12 fragment — runtime-grants
// Capability: policy (ADR-0615 §5, 24th registered capability)
// Binding ADRs: ADR-0280 §D-13.D/E (C0 face), ADR-0191 (edge vs origin)
// Fragment soak window: >= 60s per ADR-0294
// Fragment version: 2.0.0
// Reviewed: 2026-08-20 (adversarial review; v1.0.0 findings S1-S6 closed)
//
// The C0-face permits: the hot paths, and the ones static-stability.cedar constrains. Each repeats
// the freshness and signature bounds its forbid enforces, so that an absent attribute denies
// instead of deleting the guard and leaving the permit standing.

// P6 — an attested PEP workload evaluates against its OWN CELL's snapshot.
//
// `principal in resource.cell` is the cell binding. v1.0.0 declared `cell_id` on both Tenant and
// Snapshot and then never referenced it in any policy, so the cell-distributed model (ADR-0280
// §D-13.D) had no enforcement at all — a PEP in cell c1 could evaluate cell c99's snapshot.
permit (
  principal,
  action == OyaPolicy::Action::"EvaluateAgainstSnapshot",
  resource
)
when {
  principal in OyaPolicy::Role::"pep-workload" &&
  principal in resource.owner_tenant &&
  principal in resource.cell &&
  principal.spiffe_id like "spiffe://oyatie/*" &&
  resource.signature_verified &&
  resource.age_seconds <= context.max_staleness_seconds &&
  (resource.age_seconds <= 300 || resource.is_authoritative) &&
  context.token_age_seconds <= 900
};

// P7 — an attested PEP workload reads ReBAC tuples for its own tenant, from a verified and
// sufficiently fresh snapshot.
permit (
  principal,
  action == OyaPolicy::Action::"ReadRebacTuple",
  resource
)
when {
  principal in OyaPolicy::Role::"pep-workload" &&
  principal in resource.owner_tenant &&
  principal.spiffe_id like "spiffe://oyatie/*" &&
  resource.served_from.signature_verified &&
  (resource.served_from.age_seconds <= 300 || resource.served_from.is_authoritative) &&
  context.token_age_seconds <= 900
};

// P8 — the distributor propagates signed, sufficiently fresh snapshots.
//
// Attestation is required here too. v1.0.0 demanded a SPIFFE id for the local read paths but not
// for distribution, which has the higher blast radius — the asymmetry was backwards.
permit (
  principal,
  action == OyaPolicy::Action::"DistributeSnapshot",
  resource
)
when {
  principal in OyaPolicy::Role::"policy-distributor" &&
  principal in resource.owner_tenant &&
  principal.spiffe_id like "spiffe://oyatie/*" &&
  resource.signature_verified &&
  (resource.age_seconds <= 300 || resource.is_authoritative) &&
  context.token_age_seconds <= 900
};
