---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M01-foundation
phase: P01-ci-fitness-consolidation
status: Active
entry_gate: |
  ADR-0131 + ADR-0132 + ADR-0133 accepted; /specs/per-microservice-flat-layout.json + /specs/industry-best-practice-conformance.json published; Buck2 target graph and Rust package metadata ready to accept the 36 new umbrella crates + the migrated ~50 oya-check-* crates under microservices/governance/src/crates/.
exit_gate: |
  All 15 IPs merged; the full ~50-lane fitness suite executes on every PR under microservices/governance/; oya-check-industry-best-practice-conformance is a BLOCKER required_status_check on dev; oya-check-per-microservice-layout is a BLOCKER required_status_check on dev; oya-check-aggregation-index-generation is a BLOCKER required_status_check on dev; lane bypass refused without break-glass; Buck2/Prow quality-lane jobs pass for per-microservice-layout, authority-cohesion, and HG-GOV registrations in /specs/hyperscaler-gates.json.
depends_on:
  - milestone: M01-foundation
    phase: prior phases per master-plan-sequencing
    reason: Buck2 target graph + branch-protection shadow + Rust package metadata must precede gate authoring
owner_team: platform-governance
related_adrs: [ADR-0131, ADR-0132, ADR-0133]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/industry-best-practice-conformance.json]
date: 2026-05-17
doc_status: published
---

# P01-ci-fitness-consolidation: Scaffold governance µservice + adopt ~50 oya-check-* + activate industry-best-practice-conformance lane

## Purpose

This phase ships the `governance` µservice as the home of every CI-fitness/governance-lane in oyatie. It does three things:

1. **Scaffolds** the µservice (umbrella BCs `lane-runtime` + `policy-engine` + `evidence-emitter` + `aggregation-indexer` — 36 new crates across 4 BCs × 9 layers per ADR-0105).
2. **Migrates** the first batch of 10 historical `oya-check-*` crates from shared `libs/oya-check-*` surfaces to governance-owned package/target metadata per ADR-0131 IP-M01-MIGR-014. The remaining ~40 crates migrate in subsequent IPs/phases — tier-A first ten in IP-002..IP-011, tier-B in IP-012..IP-013, tier-C in IP-014..IP-015.
3. **Activates** the ADR-0133 `oya-check-industry-best-practice-conformance` BLOCKER lane on `dev` along with the ADR-0131 sibling lanes (`per-microservice-layout`, `aggregation-index-generation`).

It is delivered as one phase in M01-foundation because every other oyatie µservice depends on the fitness suite being available at PR-time to advance past `dev` per the bootstrap-order policy in `/specs/per-microservice-flat-layout.json`.

This phase advances master-plan principles:
- Hyperscaler-grade in every practice (industry-cited baselines + continuous conformance per ADR-0133).
- Nothing scheduled-for-distinct-tracked-work (every FUTURE-marked stub in the prior `oya-check-*` lane wrappers is decommissioned by this phase).
- No silent regression (per-axis BLOCKER lanes refuse drift at PR-time).
- Per-microservice flat layout (this phase migrates the ~50 check crates into their owning µservice folder).

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `governance` | `lane-runtime`, `policy-engine`, `evidence-emitter`, `aggregation-indexer`, **+ ~50 bundled check crates** | All under `microservices/governance/` per ADR-0131 | `oya-governance-{lane-runtime,policy-engine,evidence-emitter,aggregation-indexer}-{kernel,domain,usecase,api,adapter,rest,worker,sdk,app}` (36 new) + 50 migrated `oya-check-<topic>` (existing names retained per ADR-0131 §"Crate naming inside each `microservices/<ms>/crates/` subtree is unchanged") |

Plus these repo-wide artifacts (cross-cutting per ADR-0131):
- `.github/branch-protection.yaml` — add `oya-check-industry-best-practice-conformance`, `oya-check-per-microservice-layout`, `oya-check-aggregation-index-generation` to `required_status_checks` on `dev`.
- `.github/workflows/governance-suite.yml` (NEW) — matrix-fanout per-lane execution.
- `BUCK` + Rust package metadata — register the 36 new umbrella crates + migrate the 50 `oya-check-*` package descriptors to `microservices/governance/src/crates/oya-check-<topic>`; Cargo manifests are compatibility metadata only, while Buck2 remains build/test/check authority.
- `/specs/hyperscaler-gates.json` — register HG-GOV gate per ADR-0123.
- `/specs/industry-best-practice-conformance.json` — pin initial baselines (SLSA v1.0, NIST SSDF SP 800-218 rev 1, OWASP ASVS 4.0.3, Google SRE Workbook 2nd ed., AWS WAF 2024, Azure WAF 2024-12, OpenSLO v1.0, OpenTelemetry semconv LTS).
- `docs/standards/agentic-dev-team-optimization.md` — already authored per ADR-0133 §"Output Artifacts"; this phase wires its eight principles into the conformance lane.

Naming justifications for the new crate families are in `microservices/governance/PRD.md` §"Bounded Contexts".

### Out-of-scope

- Migration of the remaining ~40 `oya-check-*` crates beyond tier-A (10 in this phase) — owned by successor-IP phases P02..P04 under M01 (`phases-roadmap.md` successor-IP).
- Per-µservice lane-subset selection (run only relevant lanes per PR) — PRD Open Question 4; scheduled-for-distinct-tracked-work to a successor-IP ADR.
- Finding severity escalation policy (WARN-stacking → BLOCKER) — PRD Open Question 6; scheduled-for-distinct-tracked-work.
- Cross-region replication of Finding + audit-chain storage — scheduled-for-distinct-tracked-work to subsequent-to-M01-completion per `multi-region.md` §"Roadmap".
- External-auditor JIT scope tooling beyond the evidence-export tool — PRD Open Question 7; covered by `runbooks/evidence-replay.md`.

## Implementation Plans

Ordered list. Each IP is an executable ChangeSet under this phase folder. Dependencies inline.

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| [`IP-001-scaffold-umbrella-bcs.md`](IP-001-scaffold-umbrella-bcs.md) | Create the 36 new umbrella crates (4 BCs × 9 layers); workspace registration; catalog rows | pending | platform-governance | — |
| [`IP-002-migrate-tier-a-check-crates-batch-1.md`](IP-002-migrate-tier-a-check-crates-batch-1.md) | Move `oya-check-{lean-a1,lean-a2,port-location,layer-correctness,naming-bnf-v41}` → governance-owned package/target metadata | pending | platform-governance | IP-001 |
| [`IP-003-migrate-tier-a-check-crates-batch-2.md`](IP-003-migrate-tier-a-check-crates-batch-2.md) | Move `oya-check-{data-class,supply-chain,license-policy,placeholder-debt,brand-residue}` | pending | platform-governance | IP-001 |
| [`IP-004-lane-runtime-kernel-domain.md`](IP-004-lane-runtime-kernel-domain.md) | `oya-governance-lane-runtime-{kernel,domain}` — port traits + matrix-fanout math | pending | platform-governance | IP-001 |
| [`IP-005-lane-runtime-usecase-adapter-rest.md`](IP-005-lane-runtime-usecase-adapter-rest.md) | `-usecase` orchestrator + `-adapter` Prow/Buck2 job fanout with GitHub shadow compatibility + `-rest` HTTP surface | pending | platform-governance | IP-004 |
| [`IP-006-policy-engine-kernel-domain.md`](IP-006-policy-engine-kernel-domain.md) | `oya-governance-policy-engine-{kernel,domain}` — Rule/RulePack entities + decision algebra | pending | platform-governance | IP-001 |
| [`IP-007-policy-engine-usecase-adapter.md`](IP-007-policy-engine-usecase-adapter.md) | `-usecase` 6-axis evaluator + `-adapter` TOML/YAML reader + HTTPS baseline-diff client | pending | platform-governance | IP-006 |
| [`IP-008-evidence-emitter-kernel-domain.md`](IP-008-evidence-emitter-kernel-domain.md) | `oya-governance-evidence-emitter-{kernel,domain}` — Finding entities + canonical-JSON + Merkle | pending | ops-security | IP-001 |
| [`IP-009-evidence-emitter-adapter-rest-worker.md`](IP-009-evidence-emitter-adapter-rest-worker.md) | Postgres + S3 adapter; HTTPS replay surface; long-lived seal worker | pending | ops-security | IP-008 |
| [`IP-010-aggregation-indexer-full-stack.md`](IP-010-aggregation-indexer-full-stack.md) | `oya-governance-aggregation-indexer-{kernel,domain,usecase,adapter,rest,worker,sdk,app}` — full BC | pending | platform-governance | IP-001 |
| [`IP-011-industry-best-practice-conformance-lane.md`](IP-011-industry-best-practice-conformance-lane.md) | New BLOCKER lane on dev; reads `/specs/industry-best-practice-conformance.json`; refuses unaudited new artifacts | pending | council-architecture | IP-006, IP-007 |
| [`IP-012-per-microservice-layout-lane.md`](IP-012-per-microservice-layout-lane.md) | New BLOCKER lane on dev; refuses out-of-layout artifacts; ADR-0131 sibling | pending | platform-governance | IP-005 |
| [`IP-013-aggregation-index-generation-lane.md`](IP-013-aggregation-index-generation-lane.md) | New BLOCKER lane on dev; refuses hand-edits of central indices; ADR-0131 sibling | pending | platform-governance | IP-010 |
| [`IP-014-observability-slo-authoring.md`](IP-014-observability-slo-authoring.md) | Author OpenSLO manifests for governance under `microservices/governance/slos/` (self-observability) | pending | platform-governance + axis-observability | IP-009 |
| [`IP-015-runbooks-iac-finalization.md`](IP-015-runbooks-iac-finalization.md) | Author 6 runbooks; finalize CUE/KRM packages (lane-runner-pool, postgres, evidence-store); register HG-GOV | pending | ops-sre-reliability | IP-005, IP-009 |

Coverage check vs. ADR-0131 §"Migration DAG → IP-M01-MIGR-014" + ADR-0133 §"Output Artifacts": all four umbrella BCs (IP-004..IP-010), first 10 tier-A `oya-check-*` migrations (IP-002 + IP-003), industry-best-practice-conformance lane (IP-011), per-microservice-layout lane (IP-012), aggregation-index-generation lane (IP-013), OpenSLO self-observability (IP-014), runbooks + IaC + HG-GOV registration (IP-015). The remaining ~40 `oya-check-*` migrations queue for phases P02..P04 under M01.

## Acceptance Gates

All gates must pass before `exit_gate` is declared.

### Buck2 / oya-ci gates (exit 0 required)

```bash
buck2 build //:buck2-authority-policy-check
buck2 build //:repo-hygiene-automation-check
buck2 build //:quality-lane-registry-authority-check
buck2 build //:oya-ci-prowjob-registry-check
buck2 build //:rust-llvm-coverage-policy-check
```

### Fitness lane gates

```bash
buck2 build //:quality-lane-registry-authority-check # registers lean-a1, lean-a2, port-location, layer-correctness, per-microservice-layout, statelessness, shardability, industry-best-practice-conformance, aggregation-index-generation, authority-cohesion, and hyperscaler-maturity-claims
```

### Substrate gates introduced by this phase

```bash
# Run full ~50-lane fitness suite through trusted Prow/Buck2 status.
buck2 build //:quality-lane-registry-authority-check //:oya-ci-prowjob-registry-check

# Verify lane bypass is refused without break-glass
gh pr merge <N> --admin   # expected: rejected unless break-glass record present
```

### Per-IP Test Coverage Threshold

| Layer class | Line coverage | Branch coverage |
|---|---|---|
| kernel | 90% | 80% |
| domain | 90% | 85% |
| usecase | 85% | 75% |
| api | 85% | 70% |
| adapter | 80% | 70% |
| rest | 80% | 65% |
| worker | 80% | 65% |
| sdk | 85% | 70% |
| app | 70% | 60% |
| migrated `oya-check-*` | 80% (no change from pre-migration) | 70% |

Coverage is LLVM source-based coverage emitted by Buck2/Prow; Cargo coverage tooling is compatibility-only local metadata and is not merge authority.

## Halt Conditions

- Any IP introduces a BNF v4.1 naming violation — refer to `feedback_naming_justification.md`; halt IP, refactor, resume.
- Any IP introduces a cross-product import — LEAN-A2 refuses; halt, route through Workflow + Ontology, resume.
- Aggregation-indexer regen non-deterministic across two consecutive runs — halt; investigate ordering rules; resume only when deterministic.
- Lane runner timeout > 10s p99 on a single lane — halt; profile; optimize; resume.
- Bootstrap paradox: governance's own conformance lane fails the conformance check on itself — halt; rely on synthetic-probe fallback per PRD Open Question 3; route through ADR-0133 §"Operational" first-run amnesty.

## Phase Exit Evidence

Multispectrum evidence (per `docs/AGENTS.md` §"changeset"):
- `microservices/governance/evidence/multispectrum/P01-exit-*.json` — per-axis verdict.
- `evidence/audits/industry-best-practice-conformance/2026-Q2.json` — first quarterly refresh snapshot.

## References

- ADR-0131 §"Migration DAG → IP-M01-MIGR-014" — bundle decision authority.
- ADR-0132 §"product-platform-and-bundle dissolution" — naming authority.
- ADR-0133 §"Decision" + §"Output Artifacts" — 6-axis program authority.
- ADR-0123 (hyperscaler-maturity-claim-gate) — HG-GOV registers.
- ADR-0056 BNF v4.1; ADR-0105 13-layer enum; ADR-0106 application → usecase rename.
- `feedback_clean_architecture_requirements.md`; `feedback_quality_performance_scalability_bar.md`; `feedback_no_silent_regression.md`.
