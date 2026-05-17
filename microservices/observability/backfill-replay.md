---
doc_class: ContractSpec
title: Backfill + Replay Contract
microservice: observability
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-observability
deciders: axis-observability, council-architecture, ops-sre-reliability
related_adrs: [ADR-0130, ADR-0131]
related_artifacts:
  - microservices/observability/PRD.md (Open Question 21 — backfill / replay)
  - microservices/observability/capacity-model.md
  - /specs/agentic-slo-gated-promotion.json
review_cadence: annually
doc_status: published
---

# Backfill + Replay Contract (observability µservice)

## Purpose

Specify how the SLO engine handles two scenarios:
1. **Backfill** — a new OpenSLO manifest is authored for a µservice; can the engine compute the SLO retroactively against historical signal?
2. **Replay** — an existing eligibility verdict needs re-computation (e.g., after a bug fix in burn-rate math, or after a manual override needs to be reconciled with the truth).

## Backfill

### Contract

When a new OpenSLO manifest (or a modified one) lands at `microservices/<ms>/slos/<sli>.openslo.yaml`, the engine:

1. Receives the `OpenSloManifestUpdated` event (per `contracts/asyncapi/eligibility-events.yaml`).
2. Validates the manifest against OpenSLO v1.0 schema + the engine's PromQL feasibility check (every indicator expression is reachable against current Mimir data).
3. Computes the backfill window:
   - Default = `min(manifest.timeWindow, available_historical_data_in_mimir)`.
   - Per pack retention (per `data-residency.md`): up to 24mo of Mimir cold-tier metric history.
4. Runs the SLI expression as a Mimir range query over the backfill window with the same multi-window burn-rate model as live evaluation.
5. Emits `EligibilityChanged` events for each affected (microservice, source_sha, target_env) tuple in the backfill window. Verdicts are computed but tagged with `backfilled=true` label so consumers can distinguish.
6. Stores recording-rule results at lower resolution for cold-tier reads (1m resolution vs live 60s evaluator cadence).

### Constraints

- Backfill does NOT change historical promotion events. Rollback / promotion records are immutable in audit-chain; the backfill only fills in retroactive SLO assessment, not retroactive promotion gates.
- Backfill emits `EligibilityChanged` events with `verdict_kind=backfilled`. These are NOT consumed by promote-workflow (only `verdict_kind=live` events trigger fast-forward).
- Cost: backfill is computed once per OpenSLO change. The cost is bounded by `O(window × samples_per_sec × sli_count)` per `capacity-model.md` formulae.
- Per-tenant rate-limiting: a tenant cannot trigger more than 1 backfill per microservice per hour (anti-abuse).

### Verification

- Integration test: author a new OpenSLO manifest; verify the engine emits backfilled events spanning the expected window; verify recording-rules pre-aggregated.
- Idempotency: re-running the same backfill emits the same verdicts (deterministic).

## Replay

### Contract

Replay re-computes the verdict for a specific (microservice, source_sha, target_env) tuple. Triggers:

- Bug-fix in burn-rate math: replay invalidates old verdicts; emits new ones with the corrected math.
- Manual override reconciliation: after a manual override is used (per `runbooks/held-promotion-recovery.md` Path E), replay reconciles the "true" verdict against the override.
- Post-incident analysis: replay against alternate SLO targets to test "would a stricter SLO have caught this?"

### Procedure

1. Operator invokes: `cargo run -p oya-dev-cli -- vcs replay-eligibility --microservice <ms> --source-sha <sha> --target-env <env> --reason "<rfc>"`.
2. CLI requires 2-person rule + ops-security approval (replay can shift historical "truth" and must be audit-trail-bounded).
3. Engine recomputes the verdict against current OpenSLO manifest + current Mimir data.
4. Emits `EligibilityChanged` event with `verdict_kind=replayed`, `prior_verdict=<original>`, `reason=<rfc>`.
5. Audit-chain seal: the replay is itself sealed, distinguishing it from the original verdict.

### Constraints

- Replay does NOT mutate the original verdict record in Mimir; it appends a new one with `replayed=true` label.
- Replay cannot exceed Mimir retention (24mo cold-tier).
- Replay output never triggers retro-active promotion (no "we now declare yesterday's held verdict was actually eligible, so let's promote that SHA now"); promotions are only triggered by live verdicts on currently-deployed SHAs.

### Verification

- Integration test: induce a synthetic burn-rate, evaluate; then re-run replay with the same inputs; verify identical verdict.
- Audit-chain integrity: replay event is sealed; original event remains sealed; chain is reconstructable.

## Cost Model

| Operation | Frequency | Estimated cost per call |
|---|---|---|
| Backfill on new OpenSLO manifest | per-manifest-change | ~$0.50 (1 manifest, 30d window, single µservice) per `capacity-model.md` |
| Replay on bug-fix | per-engine-deploy | ~$10 (full re-eval across all microservices × windows) |
| Replay on manual override | per-override | ~$0.05 (single tuple) |

Cost surfaced in `cost-budget.md` §"Cost-Optimisation Levers" — backfill / replay are budgeted as part of the SLO engine's compute envelope.

## Limitations

- Backfill quality is bounded by the resolution of historical Mimir data. Pre-2026-05 data may be at lower resolution (depending on when Mimir was provisioned for this pack).
- Replay assumes deterministic SLI definitions; if an OpenSLO manifest references PromQL functions whose semantics changed between Mimir versions, the replay may produce different results from the original. The replay output explicitly carries `evaluator_version` to surface this.

## References

- `microservices/observability/PRD.md` Open Question 21.
- `microservices/observability/capacity-model.md`.
- `microservices/observability/cost-budget.md`.
- `microservices/observability/contracts/asyncapi/eligibility-events.yaml`.
- ADR-0130; `/specs/agentic-slo-gated-promotion.json`.
- Google SRE Workbook ch. 4–5 (SLO evaluation methodology).
