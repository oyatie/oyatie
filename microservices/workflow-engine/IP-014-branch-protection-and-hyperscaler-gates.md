---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate-ready
phase: P01-durable-execution-substrate
impl_plan_id: IP-014-branch-protection-and-hyperscaler-gates
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow + ops-sre-reliability
acceptance_lanes: [oya-governance-per-microservice-layout, oya-governance-authority-cohesion]
---

# IP-014: branch-protection + hyperscaler-gates registration

## Intent

Update `.github/branch-protection.yaml` to require workflow-engine governance gates; register HG-WF-ENGINE in `/specs/hyperscaler-gates.json` per ADR-0123; create initial `release/workflow-engine/{staging,production}` ref pattern protection.

## ChangeSet boundary

Repo-wide cross-cutting (PHASE-01 §"branch-protection.yaml diff preview"); 2 spec files + 1 config file.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `.github/branch-protection.yaml` | update | Add required_status_checks per PHASE-01 §"branch-protection.yaml diff preview"; add `release/workflow-engine/*` pattern protection |
| `/specs/hyperscaler-gates.json` | update | Register HG-WF-ENGINE gate per ADR-0123 |
| Initial release-pointer refs | create (one-shot) | `release/workflow-engine/staging` + `release/workflow-engine/production` (initially pointing to first green dev SHA) |

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate authority-cohesion
cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice workflow-engine
```

## Test Plan

- Verify branch-protection YAML schema validates.
- Verify HG-WF-ENGINE registration via `oya gate inspect HG-WF-ENGINE`.
- Synthetic PR to dev: confirm all newly-added required-status-checks block merge if absent.

## Next IP

[`IP-015-deterministic-replay-lane.md`](IP-015-deterministic-replay-lane.md)

## References

- ADR-0123 (hyperscaler maturity claim gate)
- ADR-0130 (agentic SLO-gated promotion)
- ADR-0131 (per-microservice flat layout + workflow unbundle)
- `microservices/workflow-engine/PHASE-01-DURABLE-EXECUTION-SUBSTRATE.md` §"branch-protection.yaml diff preview"
