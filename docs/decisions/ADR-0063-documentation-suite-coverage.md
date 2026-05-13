# ADR-0063: Documentation suite coverage — every planned feature ships with a complete doc suite, CI-enforced

> **Status:** Accepted
> **Owner:** `council-architecture`
> **Date:** 2026-05-13
> **Related:** ADR-0001, ADR-0011, ADR-0056, ADR-0058, ADR-0059, ADR-0060, ADR-0062, ADR-0064

---

## Context

Per user instruction 2026-05-13: "Ensure that every planned feature makes it into the master plan with all the necessary and full documentation suite following. Make sure this is enforced."

Oyatie's autonomous-implementation charter (per `feedback_autonomous_decision_principles.md`) requires that a phase ships complete or doesn't ship. "Complete" means **code + docs + tests + evidence land in the same commit** — no stubs, no placeholders, no deferrals.

Templates already exist at `docs/templates/`: ADR, PRD, microservice record, bounded-context registration, phase-spec, impl-plan, milestone-readme. What was missing: an enforced contract that every µservice ships with the full set, not just an ad-hoc subset.

This ADR closes that gap.

---

## Decision

### 1. The canonical artifact suite

For every µservice registered in `[workspace.metadata.oya.microservices]`, the following artifacts MUST exist before the µservice's introducing-phase can pass its exit gate:

| Artifact | Path convention | Template |
|---|---|---|
| Microservice record | `docs/microservices/<microservice>.md` | `docs/templates/microservice-template.md` |
| Product Requirements (canonical, pack-neutral) | `docs/prds/<microservice>.md` | `docs/templates/prd-template.md` |
| Naming-scope ADR | `docs/decisions/ADR-NNNN-microservice-<microservice>.md` | `docs/templates/adr-template.md` |
| Bounded-context registrations (one per BC) | `docs/bounded-contexts/<microservice>-<bc>.md` | `docs/templates/bounded-context-registration-template.md` |
| Phase-Specs (≥1 referencing the µservice) | `.omc/plans/milestones/M*/phases/*/phase-spec.md` | `docs/templates/phase-spec-template.md` |
| Impl-Plans (one per IP) | `.omc/plans/milestones/M*/phases/*/impl-plan.md` | `docs/templates/impl-plan-template.md` |

### 2. The per-localization-pack overlay suite

For every (pack × µservice) pair in pack scope (declared in `docs/localization-packs/<pack>/pack.yaml`, per ADR-0064):

| Artifact | Path convention |
|---|---|
| Pack overlay PRD | `docs/prds/<microservice>-<pack>.md` (required when pack adds material scope) |
| Pack regulatory ADR | `docs/decisions/ADR-NNNN-<pack>-<microservice>-regulatory.md` |
| Pack acceptance evidence | `docs/localization-packs/<pack>/evidence/<microservice>.md` |

### 3. Per-milestone artifacts

| Artifact | Path convention |
|---|---|
| Milestone README | `.omc/plans/milestones/M<NN>-<slug>/README.md` |
| Acceptance evidence bundle | `.omc/plans/milestones/M<NN>-<slug>/acceptance-evidence/` |

### 4. Section-completeness checks

Every PRD MUST contain:

- `## Competitive Benchmark` (per ADR-0062 quality bar)
- `## Performance Targets` (per ADR-0062 perf bar)
- `## Horizontal Scalability` (per ADR-0062 scale bar)
- `## Bounded Contexts` enumerating each BC the µservice owns

Every Phase-Spec frontmatter MUST declare:

- `acceptance_lanes:` (list of fitness lanes the phase must green)
- `depends_on:` (predecessor phase list)
- `entry_gate:` and `exit_gate:` prose

Every Impl-Plan MUST contain:

- `## Concrete File Targets` (table)
- `## Code Shape` (port traits, DDL, key types)
- `## Acceptance Gates` (cargo + fitness lane invocations)
- `## Load test` (k6 / locust / vegeta script meeting the µservice perf targets)
- `## Grit Claim Symbols` (list of symbols + TTL)
- `## ICM Rows to Emit` (phase-start, phase-complete)

### 5. Enforcement — `oya-check-doc-coverage-cli` (LEAN-A5)

Crate: `crates/oya-check-doc-coverage/` (BNF-exempt; `oya-check-*` namespace).

Lane registered in `registry/quality/lanes.yaml` as `lean-a5-doc-coverage`. Runs on every PR.

Algorithm:

1. Read `[workspace.metadata.oya.microservices]` for canonical µservice list
2. For each µservice, verify every row in §1 exists; collect missing as violations
3. Read `docs/localization-packs/INDEX.md` for active packs; for each, read `pack.yaml` for µservice scope
4. For each (pack × µservice) pair in pack scope, verify every row in §2 exists; collect missing
5. Per-milestone directory in `.omc/plans/milestones/`, verify §3 exists
6. Section-completeness: parse markdown of every PRD / Phase-Spec / Impl-Plan; verify §4 sections present and non-empty
7. Mode `--report-only` (default until M02-P22): print violations, exit 0
8. Mode `--blocker` (post-M02-P22): exit nonzero if any violation

Coverage snapshot (auto-emitted): `docs/DOC-COVERAGE.md`.

### 6. Suite-completeness is a phase exit gate

A phase that introduces a new µservice or new BC is **not Complete** until the doc-coverage lane is green for that µservice. Per scope-completion rule (`feedback_autonomous_decision_principles.md`): no stubs, no placeholders, no deferrals.

The doc suite ships in the same commit that introduces the µservice. A PR may not split code and docs across commits if the code introduces a new µservice or BC.

### 7. Retired-µservice path

A µservice removed from `[workspace.metadata.oya.microservices]` MUST have its docs physically removed in the same commit (no marked-retired flags). Per `feedback_autonomous_implementation_artifacts.md`: stale information is removed rather than marked as retired. The lane verifies no orphaned doc files reference retired µservices.

---

## Consequences

**Positive:**

- Autonomous-execution agents have a deterministic doc contract to satisfy. They cannot mark a phase complete with partial docs.
- New µservices cannot land without naming justification, PRD with benchmarks, BC registrations, and ADR — eliminating the "code merges then docs lag" antipattern.
- Localization packs (per ADR-0064) get a first-class doc surface; pack-specific regulatory work has a home and an enforcement gate.
- The CI lane is the source of truth for coverage; `docs/DOC-COVERAGE.md` becomes a snapshot, not a hand-maintained table that drifts.

**Negative:**

- Substantial up-front authoring cost for the 59 canonical µservices currently in the catalog with incomplete docs. Mitigation: parallel-dispatch executors per cluster (Healthcare / Enterprise / FinTech / Industrial / Substrate / Cloud / Connect Personal) to author the missing PRDs/ADRs/microservice records under the canonical templates.
- The lane runtime grows with µservice count; budget at 600s for the full workspace (with caching on unchanged directories).

**Neutral:**

- The lane is `--report-only` until M02-P22 (so M02 substrate work can proceed without doc-coverage blocking), then BLOCKER thereafter. This mirrors the LEAN-A1..A4 ramp pattern (per M01-P05 hardening).

---

## Compliance

CI lane: `lean-a5-doc-coverage`.

Owner team: `council-architecture` (matrix authority) + `axis-foundry` (lane implementation).

First green window: M02-P22 exit gate (lane flips to BLOCKER).

---

## References

- ADR-0056 (BNF v4.1; `[workspace.metadata.oya.microservices]` source of truth)
- ADR-0058 (flat µservice catalog)
- ADR-0062 (quality/performance/scalability bar; mandates PRD sections)
- ADR-0064 (canonical base + localization packs; per-pack overlay requirement)
- `docs/MASTERPLAN.md` §13.5 (Documentation suite coverage)
- `docs/templates/` (canonical artifact templates)
- `feedback_autonomous_decision_principles.md` (scope-completion rule)
- `feedback_autonomous_implementation_artifacts.md` (stale removed in reality)
