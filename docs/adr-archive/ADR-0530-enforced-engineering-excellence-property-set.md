---
id: ADR-0530
title: "The enforced engineering-excellence property set: falsifiable structural gates + fenced advisory remainder over the owned AST/face substrate (doc-coverage, hermeticity, scalability, hyperscaler-patterns, 12-factor, dead-code/file/ref, doc-SSOT/anti-drift, optimization, maintainability)"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-06-08
door: one-way
owner: founder
supersedes: []
superseded_by: [ADR-0700]
depends_on: [ADR-0515, ADR-0517, ADR-0528, ADR-0529]
amends: [ADR-0515]
related: [ADR-0515, ADR-0516, ADR-0517, ADR-0527, ADR-0528, ADR-0529]
related_specs:
  - /specs/masterplan.json
milestone: W2
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0530: The enforced engineering-excellence property set

## Status

**Accepted — 2026-06-08 (founder-ruled; door: one-way).**

Extends the ADR-0515 floor gate family. Detail under Component 2 of ADR-0516; AST-substrate gates land
in W2 behind the `WorkAreaTree` interface (ADR-0517).

## Context

The founder named a set of engineering-excellence properties the platform should enforce. Each must be
split HONESTLY into a gated falsifiable structural core and a fenced advisory remainder — pretending an
un-mechanizable property is mechanized would defeat falsifiability and produce false confidence.

## Decision

Extend the floor-gate family (ADR-0515) with the named engineering-excellence property gates, all on
the same `Finding` / `remediate()` contract (ADR-0528) + shrink-only ratchet, each split into a gated
falsifiable structural core and a fenced advisory remainder (`advisory-until-infra` with an
`infra_prereq` corpus, never flipping the verdict).

FULLY GATED: (1) documentation — `cloud-ci-doc-coverage` (`undocumented_public_surface`, reusing
`unpropagated_decision`/`orphan_decision`); (2) hermeticity — generalize `automation-ratchet` into
`cloud-ci-hermeticity` (`nonhermetic_shell_step`, `prebuilt_artifact_carrier` — the
`out/*.elf`/`talos-init.elf` debt class, `ambient_io_in_gate`), decidable because every gate is already
a buck2 target.

PARTIALLY GATED (structural core only; remainder advisory): (3) scalability —
`cloud-ci-scalability-patterns` (`unbounded_fan_in`, `single_leader_unsharded`,
`synchronous_unbatched_fanout`); (4) hyperscaler-patterns — `cloud-ci-hyperscaler-patterns`
(`missing_readiness_probe`/`missing_liveness_probe`, `missing_resource_limits`,
`non_horizontal_scaler`, `singleton_without_leader_election`); (5) cloud-nativeness/12-factor —
`cloud-ci-twelve-factor` (`config_in_code`, `non_disposable_state`, `logs_not_to_stdout`,
`missing_graceful_shutdown`).

Plus the founder addendum: (6) dead-code/dead-file/stale-reference (extends `staleness-reaper`:
`dead_code`, `dead_file`, `stale_reference`; remediation = AutoGenerate reap-report + AutoFix-as-PR-
with-human-approve deletion, never silent); (7) doc-SSOT/anti-drift (`duplicate_doc_claim`,
`unreachable_doc`, `derived_doc_drift` — the mechanization of the masterplan-SSOT reachability
principle); (8) optimization/algorithm-hotspot (`algorithmic_hotspot`, mostly advisory, AutoFix only
for mechanical wins); (9) maintainability/idiomatic (`non_idiomatic` clippy-class, `unformatted`
rustfmt — the highest-confidence AutoFix tier; architectural-excellence stays advisory).

Each gate carries an `input_kind` (`producer-face` | `raw-corpus-collector` | `frozen-empty-meta`,
ADR-0527) and a config seam (closed-schema `oya-ci.toml` section) with a zero-config default
reproducing today's behavior (empty corpus ⇒ green; existing debt ⇒ frozen baseline, not a wall of new
RED).

## Drivers

- The settled vision's "one owned AST substrate read by every consumer" (ADR-0517): a gate = AST
  queries for hyperscaler/cloud-native patterns and anti-patterns.
- Falsifiability honesty: the un-mechanizable part is NOT pretended mechanizable.
- Zero-config adoptability for third-party repos.

## Alternatives considered

- **A single monolithic "quality score" gate** — rejected (un-falsifiable, subjective, FP-prone).
- **Gating the soft remainder immediately** — rejected: would flip on unproven false-positive rates,
  violating the ADR-0529 advisory-until-infra promotion proof.

## Consequences

AST-substrate gates land in W2 behind the `WorkAreaTree` interface (ADR-0517); every new gate must
ship advisory-until-infra and prove zero-FP on a labeled corpus before flipping blocking (ADR-0529);
the shrink-only ratchet (ADR-0515) freezes today's debt and only NEW violations block; generated
doc/ADR stubs carry `status: stub` with NO prose, and "exists" (blocking) is split from "is-filled"
(advisory `doc_stub_unfilled`) to defeat green-by-slop. **Extends the ADR-0515 floor family.**
door:one-way.

---
*Accepted 2026-06-08 (founder-ruled; door:one-way). Source:
AUTOMATED-QUALITY-ENFORCEMENT-AND-AUTOREMEDIATION-ARCHITECTURE.md (RATIFY-TO-ADR). Extends ADR-0515
floor family; gates over ADR-0517 AST substrate.*
