// Cedar 4.12 fragment — runtime-attestation
// Capability: policy (ADR-0615 §5, 24th registered capability)
// Binding ADRs: ADR-0280 §D-13.D (cell-distributed PDP), ADR-0702 (identity-authz apex)
// Fragment soak window: >= 60s per ADR-0294
// Fragment version: 2.0.0
// Reviewed: 2026-08-20 (adversarial review; v1.0.0 findings S1-S6 closed)
//
// Two bounds that v1.0.0 carried only inside permits, and therefore lost the moment any future
// permit forgot them. A bound worth having is worth a forbid.

// F13 — a PEP evaluates only its OWN cell's snapshot. Cell-distribution is not a real property
// until something refuses the cross-cell case.
forbid (
  principal,
  action == OyaPolicy::Action::"EvaluateAgainstSnapshot",
  resource
)
unless { principal in resource.cell };

// F14 — the runtime and distribution paths require a well-formed attested workload identity.
// v1.0.0 tested `spiffe_id != ""`, which a single space satisfied.
forbid (
  principal,
  action in [
    OyaPolicy::Action::"EvaluateAgainstSnapshot",
    OyaPolicy::Action::"ReadRebacTuple",
    OyaPolicy::Action::"DistributeSnapshot"
  ],
  resource
)
unless { principal.spiffe_id like "spiffe://oyatie/*" };
