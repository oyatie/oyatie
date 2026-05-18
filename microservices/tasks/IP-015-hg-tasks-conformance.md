---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-tasks-foundation
impl_plan_id: IP-015-hg-tasks-conformance
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-tasks + ops-platform-engineering
acceptance_lanes: [hg-registration, branch-protection, oya-governance-microservice-registration, slo-coverage]
---

# IP-015: HG-tasks conformance — registration + branch protection + SLO coverage gates

## Intent

Register `tasks` in the hyperscaler-gate (HG) microservice catalogue
per ADR-0130 (SLO-gated promotion). Wire the per-microservice
branch-protection rules per `.github/branch-protection.yaml`. Confirm
every PRD-declared NFR metric maps to one of the 9 SLO files under
`microservices/tasks/slos/` and that every SLO maps to a recording
rule + burn-rate alert in `iac/helm/tasks/templates/prometheusrule.yaml`.

This IP is the final gate before phase exit: it forces SLO coverage,
branch protection, and `microservice-registration` to be green.

## ChangeSet boundary

Updates to `.github/branch-protection.yaml`,
`registry/microservices-catalog.json` (or equivalent), and
verification scripts. No production code change.

## Crate Naming

n/a.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `.github/branch-protection.yaml` | updated | adds tasks µservice rule |
| `registry/microservices-catalog.json` | updated | registers `tasks` |
| `microservices/tasks/slos/README.md` | created | per-SLO map to PRD NFR |
| `microservices/tasks/tests/conformance/hg.rs` | created | HG-class conformance |

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate microservice-registration --microservice tasks
cargo run -p oya-dev-cli -- gate validate slo-coverage --microservice tasks
cargo run -p oya-dev-cli -- gate validate branch-protection --microservice tasks
cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity --microservice tasks
```

## Test Plan

- Every NFR metric in PRD §"Performance" has a matching SLO file under
  `slos/`.
- Every SLO has a matching `record:` + `alert:` rule in
  `prometheusrule.yaml`.
- Branch protection requires every M03 lane to be green before merge.
- HG-registration includes pack tags, owner team, capability tier.

## Halt Conditions

- SLO coverage gap — refuse to enter phase-exit; no `tasks` µservice
  goes past dev without SLO authoring per ADR-0130.
- Branch-protection gap — refuse.

## Next IP

End of phase — promotes to M03 dev rollout per ADR-0130.

## References

- ADR-0130 (agentic SLO-gated promotion).
- ADR-0131 (per-microservice flat layout).
- ADR-0133 (industry-best-practice conformance).
- `docs/standards/observability-slo.md`.
