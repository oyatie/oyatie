---
doc_class: ImplementationPlan
template_id: TPL-IMPL
microservice: workflow-engine
milestone: M02b-substrate-ready
phase: P01-durable-execution-substrate
impl_plan_id: IP-014-branch-protection-and-hyperscaler-gates
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow
co_owners: [ops-sre-reliability, axis-governance]
date: 2026-05-18
related_adrs: [ADR-0123, ADR-0139, ADR-0131]
acceptance_lanes: [oya-governance-per-microservice-layout, oya-governance-authority-cohesion, hyperscaler-maturity-claims, oya-vcs-promotion-readiness]
depends_on: [IP-013]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-014 — Branch protection + hyperscaler-gates registration

## Goal

Wire the workflow-engine µservice into the platform's governance + hyperscaler-claim infrastructure:

1. Update `.github/branch-protection.yaml` so workflow-engine governance gates are required status checks on `dev` + `staging`.
2. Register `HG-WF-ENGINE` in `/specs/hyperscaler-gates.json` per ADR-0123 so the hyperscaler-maturity-claim gate evaluates workflow-engine against the 4-investor overlay.
3. Create initial `release/workflow-engine/{staging,production}` ref pattern protection so release-pointer promotion is gated.
4. Author the per-engine HG checklist + bind it to the gate validator.

## Files to create or modify

| Path | Action | Line range (approx) |
|---|---|---|
| `.github/branch-protection.yaml` | edit | +30 LoC; add `oya-governance-per-microservice-layout`, `oya-governance-authority-cohesion`, `hyperscaler-maturity-claims-workflow-engine` to required status checks; add `release/workflow-engine/*` ref pattern protection block |
| `/specs/hyperscaler-gates.json` | edit | +60 LoC; register `HG-WF-ENGINE` entry with claim cells + evidence pointers + maturity targets per ADR-0123 |
| `microservices/workflow-engine/specs/hg-wf-engine-checklist.json` | create | ~120 LoC; per-cell claim checklist (durability / observability / multi-tenancy / governance / cost / sustainability) |
| `microservices/workflow-engine/dashboards/hg-wf-engine-claim-status.json` | create | ~180 LoC; Grafana dashboard for HG claim health |
| `crates/oya-dev-cli/src/hyperscaler_maturity_claims_gate.rs` | edit | +40 LoC; wire HG-WF-ENGINE into the gate validator |
| `microservices/workflow-engine/runbooks/branch-protection-rollback.md` | create | ~80 LoC operator playbook |
| Initial release-pointer refs | create (one-shot) | `release/workflow-engine/staging` + `.../production` pointing at first green dev SHA |
| `microservices/workflow-engine/decisions/ADR-0123.md` | append §"HG-WF-ENGINE registered" | +6 LoC |

## Code shape

`/specs/hyperscaler-gates.json` (excerpt — HG-WF-ENGINE entry):

```json
{
  "HG-WF-ENGINE": {
    "claim_id": "hyperscaler-grade-durable-execution",
    "microservice": "workflow-engine",
    "adr": "ADR-0123",
    "investor_overlay": ["durability", "observability", "multi-tenancy", "governance"],
    "evidence_pointers": [
      "evidence/microservices/workflow-engine/durability-mtbf-{date}.json",
      "evidence/microservices/workflow-engine/observability-burn-rate-coverage-{date}.json",
      "evidence/microservices/workflow-engine/cell-isolation-scan-{date}.json"
    ],
    "maturity_targets": {
      "durability": { "mtbf_p99_hours": 720, "checkpoint_p95_ms": 50 },
      "observability": { "sli_coverage_percent": 95, "burn_rate_alert_coverage_percent": 100 },
      "multi_tenancy": { "noisy_neighbor_p99_blast_radius_percent": 0.1 },
      "governance": { "cell_boundary_violations": 0 }
    }
  }
}
```

## Tests to write (acceptance)

| Test name | File | Asserts |
|---|---|---|
| `branch_protection_yaml_schema_validates` | crates/oya-dev-cli/tests/branch_protection_schema.rs | YAML conforms to GitHub's branch-protection schema |
| `hg_wf_engine_registered_and_inspectable` | crates/oya-dev-cli/tests/hyperscaler_gates.rs | `oya gate inspect HG-WF-ENGINE` returns the registered entry |
| `synthetic_pr_to_dev_blocks_without_required_checks` | crates/oya-dev-cli/tests/pr_gate_integration.rs | Synthetic PR missing one required check → merge blocked |
| `release_pointer_pattern_protection_active` | crates/oya-dev-cli/tests/release_pointer_protection.rs | Direct push to `release/workflow-engine/production` → rejected |
| `hg_claim_evidence_pointers_resolve` | crates/oya-dev-cli/tests/hg_evidence_resolution.rs | Each `evidence_pointers` path exists or has a known placeholder |
| `maturity_targets_match_prd` | crates/oya-dev-cli/tests/hg_maturity_alignment.rs | PRD §"Hyperscaler claim" targets match the registered JSON |

## Evidence to emit

- `evidence/microservices/workflow-engine/branch-protection-active-{date}.json` — verified active status checks
- `evidence/microservices/workflow-engine/hg-wf-engine-registered-{date}.json` — `oya gate inspect HG-WF-ENGINE` snapshot
- Audit-chain seal: `oya audit-chain seal --kind hyperscaler-claim-registration --ms workflow-engine --window 30d`
- Metrics: `oya_hyperscaler_gate_claim_status{claim_id="HG-WF-ENGINE",cell}`, `oya_branch_protection_required_checks_total{branch}`

## Rollback procedure

1. Revert ChangeSet for `.github/branch-protection.yaml`, `/specs/hyperscaler-gates.json`, and per-µservice spec files.
2. Roll back HG-WF-ENGINE registration via `oya gate unregister HG-WF-ENGINE` (idempotent).
3. Remove release-pointer ref protection via `gh api -X DELETE /repos/<owner>/<repo>/branches/release%2Fworkflow-engine%2Fproduction/protection`.
4. Banner displayed in workflow-engine dashboard: "Hyperscaler claim gate paused — see rollback evidence".
5. Emit rollback evidence JSON.

## Blocking dependencies

- IP-013 — prior workflow-engine bring-up IPs complete (engine workload ready to be claim-evaluated).
- ADR-0123 — hyperscaler maturity claim gate.
- ADR-0131 — per-µservice flat layout (governance gate consumer).
- ADR-0139 — agentic SLO-gated promotion (release-pointer consumer).

## Acceptance gates

```bash
cargo run -p oya-dev-cli -- gate validate authority-cohesion
cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice workflow-engine
cargo run -p oya-dev-cli -- gate validate oya-vcs-promotion-readiness --microservice workflow-engine
```

## Halt conditions

- Branch-protection YAML schema invalid: STOP, governance-critical.
- HG-WF-ENGINE evidence pointer resolves to non-existent path: STOP, claim integrity violated.
- Release-pointer protection inactive (direct push succeeds): STOP, governance-critical.

## Exit criteria

1. All 6 tests green.
2. `authority-cohesion`, `hyperscaler-maturity-claims`, `per-microservice-layout`, `oya-vcs-promotion-readiness` lanes green for workflow-engine.
3. Synthetic PR to dev blocked when any required check missing.
4. Release-pointer pattern protection active and verified.
5. Evidence ledger sealed.
6. HG claim dashboard published.
7. Runbook published.
8. ADR-0123 status updated.

## Next IP

[`IP-015-deterministic-replay-lane.md`](IP-015-deterministic-replay-lane.md)

## References

- ADR-0123 — hyperscaler maturity claim gate.
- ADR-0139 — agentic SLO-gated promotion.
- ADR-0131 — per-microservice flat layout.
- ADR-0064 — canonical base + localization overlay.
- microservices/workflow-engine/PHASE-01-DURABLE-EXECUTION-SUBSTRATE.md §"branch-protection.yaml diff preview".
- GitHub branch-protection API — `https://docs.github.com/en/rest/branches/branch-protection`.
