---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P18-IP-005
title: Canary controller + per-cell cohort + thresholds + rollback
status: scaffolded
tier: S
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions:
  - reqwest OR ureq (HTTP client for OTel signal pull) — decide at impl
source_adr: ../../../../../../docs/decisions/ADR-0114-canary-observability-rollback.md
depends_on:
  - M01-P18-IP-001
purpose: Land the canary observability gate that conditions dev→staging→production auto-promotion + the dual rollback mechanism (canonical revert PR + emergency force-rewind).
---

# M01-P18-IP-005 — Canary controller + per-cell cohort + thresholds + rollback

## Scope

Implement ADR-0114 wave-A:

- New kernel `oya-foundry-canary-controller-kernel` — pure-domain
  threshold evaluator + verdict emitter. Closed enum
  `CanaryVerdict { Promote, Rollback, ExtendObservation, Escalate }`.
- New app `oya-foundry-canary-controller-app` — 30 s eval cadence
  loop. Pulls signals (latency p99, error rate, SLO breach,
  per-product KPI) from OpenTelemetry collector. Compares canary
  cells vs control cells. Emits verdict events to
  changeset-event-log (per IP-001).
- New registry `registry/cells/canary-cohort.yaml` — per-product
  cohort + expansion stages.
- New registry `registry/canary/thresholds.yaml` — per-product
  ratios + observation windows.
- New CI lane `oya-governance-canary-signal-emission` —
  asserts every product publishes the 4 signal classes.

Rollback mechanism is wave-B scope (gating workflows on verdict +
`oya canary rollback` / `oya canary force-rewind` subcommands).
This IP ships the EMITTER only; consumers wire in IP-006.

## Dependencies

- M01-P18-IP-001 (changeset-state kernel) — for verdict events.
- OpenTelemetry collector per cell (assumed; cells without OTel
  fail-closed → controller emits `EXTEND_OBSERVATION` indefinitely
  on missing-signal class).

## Acceptance

- Kernel exposes `evaluate_canary_verdict(signals: &SignalSnapshot,
  thresholds: &Thresholds) -> CanaryVerdict` with closed-enum
  return.
- 6 unit tests cover: clean signals → Promote; latency breach →
  Rollback; error breach → Rollback; SLO breach → Rollback;
  ambiguous → ExtendObservation; observation-timeout → Escalate.
- `registry/cells/canary-cohort.yaml` seeded with Foundry + VCS
  initial cohorts (1 canary cell each, stage-1 5%).
- `registry/canary/thresholds.yaml` seeded with conservative v1
  defaults from ADR-0114.
- Signal-emission lane asserts every PRD-declared product
  publishes 4 signal classes via OTel; baseline-zero on day 1
  (failure surface is "products missing OTel signal coverage").
- Smoke test: synthetic SignalSnapshot with canary p99 = control
  p99 × 1.30 → controller emits Rollback verdict; verdict
  appears in changeset-event-log within 2 evaluation cycles
  (≤60 s).

## Symbols to grit-claim

- `crates/oya-foundry-canary-controller-kernel/src/lib.rs::*`
- `tools/oya-foundry-canary-controller-app/src/main.rs::main`
- `tools/oya-foundry-canary-controller-app/src/{signal_pull,verdict_emit,cell_target_ref}.rs::*`
- `registry/cells/canary-cohort.yaml::*`
- `registry/canary/thresholds.yaml::*`
- `crates/oya-dev-cli/src/canary_signal_emission_gate.rs::*` (new
  module for the signal-emission lane runner)

## Exit evidence

- `/evidence/agentic-vcs-pipeline/ip-005-canary-controller-verdict-smoke.json`
- `/evidence/agentic-vcs-pipeline/ip-005-signal-emission-lane-baseline.json`
- `/evidence/agentic-vcs-pipeline/ip-005-otel-signal-class-coverage.json`
