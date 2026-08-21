// Cedar 4.12 fragment — static-stability
// Capability: policy (ADR-0615 §5, 24th registered capability)
// Binding ADRs: ADR-0280 §D-13.E (static-stability invariant)
// Fragment soak window: >= 60s per ADR-0294
// Fragment version: 2.0.0
// Reviewed: 2026-08-20 (adversarial review; v1.0.0 findings S1-S6 closed)
//
// The invariant: "a stale snapshot must DENY or route to the authoritative shard — never silently
// authorize." These forbids are the DENY half. The route half is a runtime concern specified in
// policy/CONTRACT.md, because Cedar answers allow/deny and cannot express "ask someone else".
//
// 300s is the ABSOLUTE ceiling. v1.0.0 let the caller declare `max_staleness_seconds` with no cap,
// so a PEP passing i64::MAX evaluated a 30-day-old non-authoritative snapshot to Allow. A bound the
// constrained party chooses is not a bound.

// F2 — a tuple read served from a snapshot that is both stale past the ceiling AND not the
// authoritative copy is refused. Non-authoritative + fresh is fine; authoritative + stale is fine
// (it IS the truth). Only the conjunction is unsafe.
//
// This reads the snapshot through `resource.served_from`. v1.0.0 asked the CALLER for
// `context.snapshot_age_seconds`, i.e. it asked the party being constrained to report the fact that
// constrains it — a request could always claim age 0.
@id("F2")
forbid (
  principal,
  action == OyaPolicy::Action::"ReadRebacTuple",
  resource
)
when {
  resource.served_from.age_seconds > 300 && !resource.served_from.is_authoritative
};

// F3 — evaluating a snapshot older than the caller's declared tolerance is refused. The caller may
// only tighten below the ceiling, never loosen past it; F3b is what makes that true.
@id("F3")
forbid (
  principal,
  action == OyaPolicy::Action::"EvaluateAgainstSnapshot",
  resource
)
unless {
  context has max_staleness_seconds &&
  resource.age_seconds <= context.max_staleness_seconds
};

// F3b — the absolute ceiling, independent of anything the caller asserts. Also covers distribution:
// v1.0.0 gated distribution on signature only, so an ancient non-authoritative snapshot propagated
// fleet-wide.
@id("F3b")
forbid (
  principal,
  action in [
    OyaPolicy::Action::"EvaluateAgainstSnapshot",
    OyaPolicy::Action::"DistributeSnapshot"
  ],
  resource
)
when { resource.age_seconds > 300 && !resource.is_authoritative };

// F4 — an unverified snapshot never serves and never propagates. Verification precedes use,
// mirroring the load order in iam/adapters/cloud-pdp-bundle-file: verify, THEN parse.
@id("F4")
forbid (
  principal,
  action in [
    OyaPolicy::Action::"EvaluateAgainstSnapshot",
    OyaPolicy::Action::"DistributeSnapshot"
  ],
  resource
)
unless { resource.signature_verified };

// F4b — the same rule on the tuple read path, which reaches its snapshot indirectly.
@id("F4b")
forbid (
  principal,
  action == OyaPolicy::Action::"ReadRebacTuple",
  resource
)
unless { resource.served_from.signature_verified };
