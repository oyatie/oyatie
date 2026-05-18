---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-sites-foundation
impl_plan_id: IP-015-hg-sites-maturity-claim
status: pending
execution_unit: ChangeSet
owner: axis-sites
acceptance_lanes: [oya-governance-hyperscaler-maturity-claims]
---

# IP-015: HG-SITES hyperscaler-maturity claim entry

## Intent

Register the HG-SITES hyperscaler-maturity claim entry per ADR-0123 + ADR-0133. Bind to all 9 SLOs, all 7 runbooks, all 11 BCs, all 11 pack overlays. Acceptance: HG-SITES passes at p99 SLOs sustained 30d in dev cluster.

## ChangeSet boundary

1 entry in `registry/hyperscaler-maturity-claims.json` + cross-references to all sites artifacts.

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims --microservice sites
```

## Phase-exit gate

Per ADR-0130 (agentic SLO-gated promotion): all 15 AC-IDs in PRD-sites
green; SLO eligibility verdict `eligible` for `sites` µservice over
`dev → staging` window; reviewer-agent APPROVE on each ChangeSet;
per-changeset evidence committed at
`microservices/sites/evidence/multispectrum/*.json`.

## References

- ADR-0123 (hyperscaler-maturity claim gate).
- ADR-0130 (agentic SLO-gated promotion).
- ADR-0133 (industry best-practice conformance program).
- `registry/hyperscaler-maturity-claims.json`.
