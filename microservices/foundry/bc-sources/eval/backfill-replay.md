---
doc_class: BackfillReplayPlan
title: Backfill + Replay Plan
microservice: foundry-eval
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry + ops-sre-reliability
deciders: axis-foundry, ops-sre-reliability, council-privacy
related_adrs: [ADR-0024, ADR-0026, ADR-0131]
related_artifacts:
  - microservices/foundry-eval/capacity-model.md
  - microservices/foundry-eval/cost-budget.md
  - microservices/foundry-eval/policy/data-residency.md
review_cadence: quarterly
doc_status: published
---

# Backfill + Replay Plan (foundry-eval µservice)

## Purpose

Define how foundry-eval backfills historical eval signal when (a) a new capability is registered with eligible history; (b) a new eval-set version is promoted and prior runs should be re-classified; (c) a replay-cohort sample needs to be regenerated from production traces; (d) per-subject DEK shred forces partial re-derivation of cohort statistics.

## Backfill Triggers

### B-1: New capability registered with eligible history

When a previously-existing capability (e.g., un-Cosign-signed legacy capability now signed) registers an eval-set v1, the publish-gate cannot make a meaningful nightly-history claim until backfill is performed.

**Procedure:**
1. Capability owner authors eval-set v1 + signs.
2. Registry accepts v1.
3. Trigger backfill: `oya-foundry-eval-eval-runner-rest backfill --capability <cap> --from <date> --to <now>`.
4. Backfill orchestrator dispatches eval-set v1 against historical sampled traces (using replay-engine adapter) at a bounded rate (10 cases/min default to avoid GPU pool saturation).
5. Generated history persisted to ClickHouse with backfill flag.
6. Once N nightly-equivalent days populated, publish-gate considers history complete.

### B-2: New eval-set version promoted; re-classify historical runs

When an eval-set version is promoted that adds new cohort (e.g., new locale; new adversarial pattern), prior runs lack coverage for the new cohort. Backfill re-evaluates historical runs against the new cohort only.

**Procedure:**
1. Identify the new cohort delta.
2. Trigger `oya-foundry-eval-eval-runner-rest backfill-cohort --capability <cap> --from-version <prior> --to-version <new> --cohort <new-cohort>`.
3. Dispatch only new-cohort cases against historical golden outputs + provider routes.
4. Augment historical run records with new-cohort aggregate.
5. New aggregates retain the original run timestamp + carry `backfill_cohort_added_at` field.

### B-3: Replay-cohort sample regeneration

When the replay sample cohort needs refresh (e.g., quarterly refresh; or after sandbox image upgrade requiring re-replay), regenerate the cohort.

**Procedure:**
1. Sample selection per current sampling policy (random / error-paths / high-traffic / recent-week).
2. Per-subject DEK availability check (only un-shredded subjects eligible).
3. Replay execution.
4. ClickHouse persistence with `replay_cohort_generated_at` field.

### B-4: Per-subject DEK shred forces partial re-derivation

When DSR cascade shreds a subject's DEK, replay against that subject is structurally impossible. Affected cohort aggregates must be re-derived without the shredded subject's contribution.

**Procedure:**
1. EvalSubjectShred event consumed by parity-analyzer-worker.
2. Identify all aggregates touching the shredded subject's run-results.
3. Re-compute aggregate excluding shredded-subject contribution.
4. Emit `AggregateRederived{aggregate_id, reason="dsr_shred", original_aggregate_ref, rederived_at}`.
5. Per-subject contribution is not deleted (it's encrypted and unreplayable); aggregates above remain valid but new aggregates reflect post-shred boundary.

## Replay Execution

Per ADR-0024 §"Replay against past traces":
- Sample selection per `capabilities/replay-execute.yaml` cohort enum.
- Deterministic-seed cases mandatory in replay cohort (foundation of divergence assertion).
- Replay-trace fetch via S3 adapter; per-subject DEK unwrap via KMS.
- Divergence detection: structural equality where deterministic, semantic equality (via judge) where non-deterministic.

## Bounded Rate

| Operation | Rate |
|---|---|
| Backfill new-capability | 10 cases/min |
| Backfill new-cohort | 10 cases/min |
| Replay cohort regeneration | 100 cases/hour |
| DSR-induced aggregate re-derivation | as fast as DEK-shred enables (bounded by ClickHouse insert throughput) |

These bounds avoid GPU pool saturation + ClickHouse hot-shard pressure. Override via `--rate-multiplier` flag with 2-person rule per `policy/two-person-admin-ops.md`.

## Compliance Considerations

### EU AI Act §17

Backfill operations themselves emit §17 logging events; the historical eval-run records remain authoritative; backfill augments, not overwrites.

### GDPR Art. 25 (privacy-by-design)

Per-subject DEK availability check is mandatory before any replay; structural enforcement.

### HIPAA / KR PIPA

Backfill against synthetic-PHI fixtures unaffected; backfill against live-PHI in replay traces requires per-subject DEK + un-shredded subject status.

## Cost Implications

Per `cost-budget.md`:
- Backfill consumes provider tokens; bounded-rate prevents budget spike.
- Per-capability owner pays for backfill cohort tokens (allocates from per-capability budget).
- foundry shared budget pays for cross-capability replay-cohort regeneration.

## Verification

- `cargo run -p oya-dev-cli -- gate validate backfill-replay --microservice foundry-eval` exits 0.
- Per-quarter backfill report in `evidence/backfill/<year>/<quarter>.md`.
- DSR cascade re-derivation count tracked in dashboard.

## References

- ADR-0024 §"Replay against past traces".
- ADR-0024 §"Resolved 1" (per-subject DEK shred).
- `microservices/foundry-eval/capabilities/replay-execute.yaml`.
- `microservices/foundry-eval/capacity-model.md`.
- `microservices/foundry-eval/cost-budget.md`.
