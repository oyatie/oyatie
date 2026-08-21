// Cedar 4.12 fragment — static-stability
// Capability: policy (ADR-0615 §5, 24th registered capability)
// Binding ADRs: ADR-0280 §D-13.E (static-stability invariant)
// Fragment soak window: >= 60s per ADR-0294
// Fragment version: 1.0.0
// Reviewed: 2026-08-20
//
// The invariant, verbatim: "a stale snapshot must DENY or route to the authoritative shard — never
// silently authorize." These forbids are the DENY half expressed as policy. The route-to-
// authoritative half is a runtime concern and is specified in policy/CONTRACT.md, because Cedar
// answers allow/deny and cannot express "ask someone else".

// F2 — a read served from a snapshot that is both stale beyond its bound AND not the authoritative
// copy is refused. Non-authoritative + fresh is fine; authoritative + stale is fine (it IS the
// truth). Only the conjunction is unsafe.
forbid (
  principal,
  action == OyaPolicy::Action::"ReadRebacTuple",
  resource
)
when {
  context.snapshot_age_seconds > 300 && !context.snapshot_is_authoritative
};

// F3 — evaluating against a snapshot older than the caller's declared tolerance is refused. The
// caller states the bound; the plane enforces it. A PEP that needs strong freshness sets it low.
forbid (
  principal,
  action == OyaPolicy::Action::"EvaluateAgainstSnapshot",
  resource
)
when { resource.age_seconds > context.max_staleness_seconds };

// F4 — an unverified snapshot never serves and never propagates. Signature verification precedes
// use, mirroring the load order in iam/adapters/cloud-pdp-bundle-file: verify, THEN parse.
forbid (
  principal,
  action in [
    OyaPolicy::Action::"EvaluateAgainstSnapshot",
    OyaPolicy::Action::"DistributeSnapshot"
  ],
  resource
)
unless { resource.signature_verified };
