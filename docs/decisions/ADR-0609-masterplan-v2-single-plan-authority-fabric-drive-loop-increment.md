---
id: ADR-0609
title: "Masterplan v2 single plan authority + fabric drive-loop increment (four plan gates wired into oya-ci-required)"
status: Accepted
planning_impact: true
deciders: founder
date: 2026-07-02
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
amended_by: [ADR-0619]
depends_on: [ADR-0515, ADR-0516, ADR-0548, ADR-0551, ADR-0555]
amends: []
related: [ADR-0363, ADR-0364, ADR-0517, ADR-0521, ADR-0537, ADR-0552]
related_specs:
  - /specs/masterplan.json
  - /specs/fabric-drive-loop-state.json
  - /specs/root-hub-pointers.json
milestone: W0
---

# ADR-0609: Masterplan v2 single plan authority + fabric drive-loop increment (four plan gates wired into oya-ci-required)

## Status

**Proposed - 2026-07-02 (authored for founder sign-off, ADR-0605..0608 gate-ADR convention).** The
zero-based re-derived sequencing this consolidation carries was already
ratified by the founder before any execution-wave dispatch; the durable ratification record is
`evidence/goals/masterplan-v2-sequencing-founder-ratification-20260702.json` and the machine-checked
record is `specs/masterplan.json` `masterplan_v2.sequencing.founder_ratification` (enforced by the
`masterplan_sequencing_invalid` lane).

## Context

Oyatie's plan authority was fragmented across seven live-looking surfaces (`specs/masterplan.json`
v1 fragments, `specs/master-plan-sequencing.json`, the planning-closure contract + status ledger,
`docs/MASTERPLAN.md`, `docs/ROADMAP.md`, `.omc/ultragoal/goals.json`) plus repo-local and global
agent-harness stores (`.omc/`, `.omx/`, and `.gjc/`). Duplicate authority rots by
discipline, not by mechanism: status claims drifted from evidence, projections went stale by hand,
and superseded surfaces kept being read as live. ADR-0516 names the Agentic Delivery Fabric as the
apex product; this consolidation is a fabric increment: one evidence-audited plan authority plus a
two-plane drive loop that executes ready work through mechanically disjoint parallel lanes.

## Decision

1. **Single plan authority.** `/specs/masterplan.json` `masterplan_v2` is the SOLE live plan
   authority: one `MPV2-*` work-item ID space, an explicit dependency DAG, program-sharded
   coverage, per-claim evidence refs, and an auditable `surface_dispositions` ledger that absorbs
   or archives-with-provenance every legacy surface. Surviving human-facing surfaces
   (`docs/MASTERPLAN.md`) are GENERATED projections. ADR-0619 removes the source-specific external
   board import and its 934 unverified claims. Any future external claim surface must be
   provider-neutral and cannot attain verified completion without recorded completion evidence.

2. **Two-plane drive loop.** The durable plane lives in-repo and PR-governed under
   `plan/fabric-loop/` (cards + per-pass flow-metrics ledger), written only through
   `tools/oya-fabric-loop-state-app` ports whose traits model the owned cloud-ci destination
   (`specs/fabric-drive-loop-state.json`); the operational claim/heartbeat plane is repo-adjacent
   and gitignored. Lane parallelism is decided by the mechanical path/ownership-overlap detector,
   never judgment. Proof runs: `evidence/goals/fabric-loop-e2e-proof-run-20260702.json` and
   `evidence/goals/fabric-loop-parallel-lanes-proof-run-20260702.json`.

3. **Four masterplan plan gates, wired blocking into the ONE required context.** The plan
   authority is enforced by four owned-Rust gate lanes in the cross-artifact-agreement gate crate
   (`ci/facade/cross-artifact-agreement`):
   - **structural** — work-item ID uniqueness, dependency-DAG acyclicity, dangling-reference
     detection (`masterplan_work_item_id_collision`, `masterplan_dependency_dag_invalid`);
   - **projection-freshness** — every generated projection must re-derive byte-identically from
     `masterplan_v2` (`masterplan_projection_stale`;
     `ci/facade/cross-artifact-agreement/src/projection_rederivation.rs`);
   - **plan-vs-evidence** — verified-completion claims require recorded completion evidence;
     dangling/retired/malformed refs fail closed (`masterplan_plan_evidence_unrecorded`;
     `ci/facade/cross-artifact-agreement/src/plan_evidence_crosscheck.rs`);
   - **read-contract / entry-surface** — read contracts on surviving artifacts, exact
     entry-surface equality with `/specs/root-hub-pointers.json`, and the archive-marker
     resurrection sweep (`masterplan_read_contract_invalid`, `masterplan_entry_surface_invalid`;
     `ci/facade/cross-artifact-agreement/src/read_surface_resurrection.rs`).

   These lanes run inside the `gate · cross-artifact-agreement` matrix leg of
   `.github/workflows/oya-ci-required.yml` and therefore fan into the SINGLE protected
   `oya-ci-required` context (ADR-0515): green IFF every lane is green, blocking for every PR
   against `dev`. Each lane is born-blocking via live-corpus self-tests and pinned by isolated
   frozen RED fixtures.

### Artifact registration (exact paths)

This ADR is the justification anchor (ADR-0555 accounting) for the artifacts this increment adds:

```
ci/facade/cross-artifact-agreement/src/plan_evidence_crosscheck.rs
ci/facade/cross-artifact-agreement/src/projection_rederivation.rs
ci/facade/cross-artifact-agreement/src/read_surface_resurrection.rs
evidence/goals/fabric-loop-e2e-proof-run-20260702.json
evidence/goals/fabric-loop-parallel-lanes-proof-run-20260702.json
evidence/goals/masterplan-v2-external-board-import-retirement-closure-20260720.json
evidence/goals/masterplan-v2-sequencing-founder-ratification-20260702.json
evidence/goals/masterplan-gates-ci-wiring-20260702.json
plan/fabric-loop/cards/MPV2-0000.C001.json
plan/fabric-loop/cards/MPV2-0000.C002.json
plan/fabric-loop/cards/MPV2-0000.C003.json
plan/fabric-loop/cards/MPV2-0000.C004.json
plan/fabric-loop/cards/MPV2-0000.C005.json
plan/fabric-loop/cards/MPV2-0000.C006.json
plan/fabric-loop/cards/MPV2-0000.C007.json
plan/fabric-loop/flow-metrics/passes/pass-00000000000000000001.json
plan/fabric-loop/flow-metrics/passes/pass-00000000000000000002.json
specs/fabric-drive-loop-state.json
specs/fixtures/cross-artifact-agreement/tc-XA-bad-masterplan-dangling-dependency-ref.json
specs/fixtures/cross-artifact-agreement/tc-XA-bad-masterplan-dependency-cycle.json
specs/fixtures/cross-artifact-agreement/tc-XA-bad-masterplan-dependency-dag.json
specs/fixtures/cross-artifact-agreement/tc-XA-bad-masterplan-duplicate-work-item-id.json
specs/fixtures/cross-artifact-agreement/tc-XA-bad-masterplan-entry-surface-resurrected-superseded.json
specs/fixtures/cross-artifact-agreement/tc-XA-bad-masterplan-entry-surface-unbounded.json
specs/fixtures/cross-artifact-agreement/tc-XA-bad-masterplan-evidence-dangling-ref.json
specs/fixtures/cross-artifact-agreement/tc-XA-bad-masterplan-evidence-retired-surface.json
specs/fixtures/cross-artifact-agreement/tc-XA-bad-masterplan-evidence-unrecorded-done-claim.json
specs/fixtures/cross-artifact-agreement/tc-XA-bad-masterplan-plan-evidence-drift.json
specs/fixtures/cross-artifact-agreement/tc-XA-bad-masterplan-program-coverage.json
specs/fixtures/cross-artifact-agreement/tc-XA-bad-masterplan-projection-hand-edited.json
specs/fixtures/cross-artifact-agreement/tc-XA-bad-masterplan-projection-stale-ledger.json
specs/fixtures/cross-artifact-agreement/tc-XA-bad-masterplan-read-contract-resurrected-roadmap.json
specs/fixtures/cross-artifact-agreement/tc-XA-bad-masterplan-sequencing-unratified.json
specs/fixtures/cross-artifact-agreement/tc-XA-good-masterplan-read-surface-archive-clean.json
specs/fixtures/cross-artifact-agreement/tc-XA-good-masterplan-sequencing-ratified.json
tools/oya-fabric-loop-state-app/BUCK
tools/oya-fabric-loop-state-app/Cargo.toml
tools/oya-fabric-loop-state-app/OWNERS
tools/oya-fabric-loop-state-app/src/lib.rs
tools/oya-fabric-loop-state-app/src/main.rs
tools/oya-fabric-loop-state-app/tests/contract.rs
```

The surviving fabric-loop tool crate is capability-registered under the `build/` meta home
(`specs/capability-registry.json`), ownership-seeded by its `OWNERS` marker, and the
`plan/fabric-loop/` durable-plane tree is reachability-registered by prefix in
`specs/reachability-registry.json`.

## Decision drivers

- SSOT integrity: no duplicate or contradicting plan authority may survive; single-writer mutation
  path enforced by machine, not discipline (anti-staleness doctrine, ADR-0548).
- Evidence over assertion: every status claim traces to recorded completion evidence; unverified
  done-claims never surface as done (ADR-0364 evidence posture).
- Fabric alignment: advances ADR-0516 components (drive loop, lanes, improvement layer) instead of
  creating a parallel system; cutover target is the owned cloud-ci loop-state service.

## Alternatives considered

- **Status quo (seven live surfaces + harness stores).** Pros: zero migration cost. Cons: duplicate
  authority, evidence drift, stale projections read as live. Rejected: the defect class this ADR
  closes.
- **New N+1 plan artifact beside the old ones.** Pros: cheap to author. Cons: worsens fragmentation;
  violates the no-N+1 constraint. Rejected.
- **Standalone gate crate per plan lane (four new crates).** Pros: per-lane check-run attribution.
  Cons: four crates duplicating the cross-artifact corpus assembly; the lanes ARE cross-artifact
  agreement checks; more fan-in wiring surface. Rejected: lanes live in the existing
  cross-artifact-agreement gate with per-lane violation codes and fixtures.

## Why chosen

Satisfies the consolidation acceptance criteria (single ID space, explicit DAG, evidence-audited
status, generated projections, bounded entry surface); honors ADR-0515 (one canonical CI context),
ADR-0551 (frozen-baseline ratchet), ADR-0555 (structural accounting), and ADR-0548
(pipeline-as-product: gates ship as data-driven owned Rust with fixtures); beats the alternatives
by removing duplicate authority instead of adding coordination process.

## Consequences

### Positive
- One plan authority, machine-enforced: duplicate IDs, DAG cycles, dangling refs, stale
  projections, unevidenced done-claims, and resurrected superseded surfaces all fail `oya-ci-required`.
- The drive loop's durable plane is PR-governed and byte-re-derivable, so loop state cannot drift
  from the plan silently.

### Negative
- `docs/MASTERPLAN.md` edits now REQUIRE regenerating through the projection writer; hand edits go
  red. Contributors must learn the single-writer path.
- The justification resolver (ADR-0555) matches exact paths: future per-card shard files under
  `plan/fabric-loop/` will need either an ADR mention or a prefix-aware justification follow-up
  (see Follow-ups #1).

### Operational
- The four plan lanes run inside the `gate · cross-artifact-agreement` matrix leg; remediation text
  ships as DATA in the gate-disposition table. Red means: fix the plan artifact (or regenerate the
  projection); never hand-edit generated faces.
- Loop-state operational plane stays repo-adjacent/gitignored; cutover to the cloud-ci-owned
  loop-state service follows the six-gate + shadow-parity criteria in
  `specs/fabric-drive-loop-state.json#cutover_target`.

## Follow-ups

1. Prefix-aware justification for registered fabric-loop trees (owner: cloud-ci-platform; tracked
   as drive-loop backlog card `plan/fabric-loop/cards/MPV2-0000.C007.json`): teach the ADR-0555
   justification resolver that a reachability-registered prefix with a declared ADR anchor
   justifies its members, so per-card shard files do not accrue per-file ADR-mention debt.
2. Owned cloud-ci loop-state service cutover (MPV2-0034 / `specs/fabric-drive-loop-state.json#cutover_target`).

## References

- ADR-0515 (one canonical CI), ADR-0516 (Agentic Delivery Fabric), ADR-0521 (staged fabric
  roadmap), ADR-0548 (pipeline-as-product), ADR-0551 (frozen baseline), ADR-0555 (structural
  accounting), ADR-0363 (agent coordination on plain git).
- Evidence: `evidence/goals/masterplan-v2-sequencing-founder-ratification-20260702.json`,
  `evidence/goals/fabric-loop-e2e-proof-run-20260702.json`,
  `evidence/goals/fabric-loop-parallel-lanes-proof-run-20260702.json`,
  `evidence/goals/masterplan-gates-ci-wiring-20260702.json` (red probe runs 28592595372/28594633232
  blocking PR #1181; green oya-ci-required run 28596752891 admitting PR #1182).
