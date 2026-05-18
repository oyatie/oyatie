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

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

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

Per ADR-0139 (agentic SLO-gated promotion): all 15 AC-IDs in PRD-sites
green; SLO eligibility verdict `eligible` for `sites` µservice over
`dev → staging` window; reviewer-agent APPROVE on each ChangeSet;
per-changeset evidence committed at
`microservices/sites/evidence/multispectrum/*.json`.

## ChangeSet metadata

```yaml
changeset_id: CS-SITES-IP-015-hg-sites-maturity-claim
depends_on_changesets: [CS-SITES-IP-012-policy-dpia-threat-model, CS-SITES-IP-013-contracts-and-capabilities, CS-SITES-IP-014-dashboards-runbooks-slos]
parallel_safe_with_changesets: []
enables: []
acceptance_status: ga
phase_exit: true
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | HG-SITES registered in `registry/hyperscaler-maturity-claims.json` with all 9 SLO refs + 11 BC refs + 11 pack overlay refs | `cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims --microservice sites` |
| AC-02 | All 15 PRD-sites AC-IDs green | per-AC gate audit |
| AC-03 | SLO eligibility verdict `eligible` over `dev → staging` 30d window | ADR-0139 ledger query |
| AC-04 | Reviewer-agent APPROVE on every ChangeSet in sites phase | per ADR-0111 + ADR-0112 |
| AC-05 | Per-changeset evidence committed at `microservices/sites/evidence/multispectrum/*.json` | filesystem + audit-chain |

## Build Sequence

1. Append HG-SITES entry to `registry/hyperscaler-maturity-claims.json`.
2. Register HG-SITES in `crates/oya-foundry-gate-catalog-domain`.
3. Add branch-protection required status check for HG-SITES on sites releases to `dev`.
4. Run `cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims --microservice sites`.
5. Confirm SLO ledger eligibility for 30 days.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-sites AC | AC-01..AC-15 (entire surface) |
| ADR | ADR-0123, ADR-0139, ADR-0133 |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| HG-SITES marked green while a load-bearing AC silently failing | Per-AC gate must be green before claim emits |
| 30d SLO window resets prematurely on infra incident | Burn-rate cool-down policy per ADR-0139 |
| Reviewer-agent APPROVE bypassed for emergency hotfix | Hotfix carries waiver record + post-hoc review obligation |

## References

- ADR-0123 (hyperscaler-maturity claim gate).
- ADR-0139 (agentic SLO-gated promotion).
- ADR-0133 (industry best-practice conformance program).
- `registry/hyperscaler-maturity-claims.json`.
- Google SRE Book — "Implementing SLOs" chapter (Beyer et al., O'Reilly 2016).
- AWS Well-Architected Framework — Reliability Pillar.
