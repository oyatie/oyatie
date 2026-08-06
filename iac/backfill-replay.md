---
doc_class: ContractSpec
title: Backfill + Replay Contract
microservice: cloud-iac
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-cloud-iac
deciders: axis-cloud-iac, architecture-governance, ops-sre-reliability
related_adrs: [ADR-0139, ADR-0131]
related_artifacts:
  - iac/PRD.md
  - iac/capacity-model.md
  - /specs/per-microservice-flat-layout.json
review_cadence: annually
doc_status: published
---

# Backfill + Replay Contract (cloud-iac µservice)

## Purpose

Specify how cloud-iac handles three scenarios:
1. **Backfill** — a new µservice onboards; can the registry retroactively record its prior applies (typically not; cloud-iac is forward-only) but can scan its IaC sources and emit synthetic baseline records.
2. **Replay** — an existing apply needs re-execution (typically after a bug-fix in the iac-applier; OR after a SLSA verifier hardening; OR for post-incident reproduction).
3. **State reconciliation** — drift-detector finds the live cluster differs from git, and the µservice owner decides to update git (rather than revert live) — replay the apply with the new git head.

## Backfill

### Contract

When a new µservice onboards (new `microservices/<ms>/iac/...` source paths land via PR), the iac-registry-worker:

1. Receives the `MicroserviceRegistered` event.
2. Validates IaC source paths conform to the flat-layout convention (per ADR-0131).
3. Computes the baseline content digest for each declared chart + module + overlay.
4. Emits a synthetic `RenderCompleted` event tagged with `baseline=true` so consumers (audit-chain, cloud-governance-evidence) can distinguish from real renders.
5. Does NOT trigger an apply; the first apply occurs through the normal `EligibilityChanged → iac-applier` flow.
6. Records the baseline in iac-state-index with `current_sha=null` (no apply yet) but with the chart/module/overlay records populated.

### Constraints

- Backfill does NOT trigger applies. The first apply is always triggered by an explicit EligibilityChanged event from observability.
- Backfill is idempotent: re-running on the same µservice produces the same baseline digest.
- Cost: backfill is bounded by `O(N_charts_in_microservice × render_cost)`; typical µservice has 1–5 charts; cost < $0.01 per backfill.
- Per-µservice rate-limit: ≤ 1 backfill per µservice per 24h (anti-abuse; legitimate onboarding is once per µservice).

### Verification

- Integration test: scaffold a new µservice's IaC; assert MicroserviceRegistered event + synthetic RenderCompleted are emitted; assert iac-state-index has the chart/module/overlay records populated.
- Idempotency: re-running emits same baseline digest.

## Replay

### Contract

Replay re-executes a specific apply for a (microservice, sha, pack, environment) tuple. Triggers:

- Bug-fix in iac-applier or iac-validator (e.g., a Cedar policy bug that incorrectly refused a legitimate apply); operator decides to replay the apply with the corrected logic.
- Hardened SLSA verifier (e.g., updated Sigstore root) needs re-verification; operator replays all affected applies with the new verifier.
- Post-incident reproduction (e.g., "would the new policy have caught this incident?"); operator replays with the new policy in dry-run mode.
- iac-state-index restored from PITR; operator replays to fill the gap between WAL replay point and incident.

### Procedure

1. Operator invokes: cloud-native IaC controller/API `replay --microservice` workflow.
2. CLI requires 2-person rule + ops-security approval (replay can re-execute against current cluster state; must be audit-trail-bounded).
3. iac-renderer re-renders the manifest set from the source SHA.
4. iac-validator re-plans against current live state.
5. If `dry-run=true`: emit `ReplayDryRunCompleted{plan_diff_against_current}` event; do not mutate cluster.
6. If `dry-run=false`: emit `ApplyStarted` event tagged with `replayed=true, prior_apply_id=<>, reason=<rfc>`; orchestrate apply; emit `ApplyCompleted` tagged with replay metadata.
7. iac-state-index appends a new apply row with `replayed=true` label; previous apply row is preserved.
8. Audit-chain seal: the replay event distinguishes itself from the original apply.

### Constraints

- Replay does NOT mutate the original apply record in iac-state-index; it appends a new one.
- Replay against an SHA older than iac-state-index retention may not have all prior context; the operator is warned at CLI time.
- Replay output never overrides a downstream rollback; if the µservice has been rolled back since the original apply, replay against the new desired state, not the rolled-back state.
- Per-µservice replay rate limit: ≤ 5 replays per µservice per 24h.

### Verification

- Integration test: apply an SHA; then replay; verify identical cluster state + replay row in index.
- Audit-chain integrity: replay event sealed; original event remains sealed; chain reconstructable.

## State Reconciliation

### Contract

When drift-detector finds live ≠ git AND µservice owner decides to "git-blessed" the live state (rather than revert), the operator can replay the apply against the updated git head.

### Procedure

1. µservice owner opens a PR updating `microservices/<ms>/iac/...` to match the desired live state.
2. PR merges; observability evaluates SLO; eligibility verdict emits.
3. iac-applier consumes verdict; applies the new IaC (which matches live state).
4. Drift report clears within next drift cycle.

### Constraints

- This path is functionally identical to a normal apply; it's only distinguished by the operator's intent.
- The drift report that prompted the reconciliation is preserved in iac-state-index for audit purposes; not deleted on resolution.

## Cost Model

| Operation | Frequency | Estimated cost per call |
|---|---|---|
| Backfill on µservice onboarding | per-µservice-onboard | ~$0.01 (1 µservice, ~3 charts) per `capacity-model.md` |
| Replay (per-µservice) | per-bug-fix or per-incident | ~$0.05 (single tuple; rerender + revalidate + reapply) |
| State reconciliation (PR + normal apply) | per-drift-resolution | ~$0.01 (normal apply cost) |

Cost surfaced in `cost-budget.md` §"Cost-Optimisation Levers"; replay budgeted within applier compute envelope.

## Limitations

- Replay quality is bounded by iac-state-index retention (≥3y default; ≥6y HIPAA). Older applies may have insufficient context for full replay.
- Replay assumes deterministic chart rendering; if a chart references current timestamp / env var (non-deterministic), replay may produce different output. The replay output explicitly carries `renderer_version` to surface this risk.
- Backfill emits synthetic baseline records; consumers must respect the `baseline=true` flag to avoid treating these as real applies.

## References

- `iac/PRD.md` Open Question 4–5.
- `iac/capacity-model.md`.
- `iac/cost-budget.md`.
- `iac/contracts/asyncapi/cloud-iac-events.yaml`.
- ADR-0139; ADR-0131.
- `microservices/observability/backfill-replay.md` (parent template).
