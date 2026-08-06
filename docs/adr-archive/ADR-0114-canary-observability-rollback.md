---
status: Superseded
deciders: council-foundry-vcs, council-observability, council-architecture
date: 2026-05-16
owner: council-foundry-vcs
supersedes: []
superseded_by: [ADR-706]
related:
  - ADR-0110-changeset-state-machine.md
  - ADR-0111-merge-queue-projected-state-fix-at-any-stage.md
  - ADR-0112-webhook-driven-intelligence-agent-invocation.md
  - ADR-0113-vcs-orchestrator-end-to-end.md
purpose: Define the canary observability gate that conditions dev→staging→production auto-promotion + the rollback mechanism when canary fails.
---

# ADR-0114: Canary observability gate + rollback

## Context

The FINAL-FINAL branch pipeline (per
`feedback_branch_pipeline_final_final`, implemented 2026-05-16)
auto-promotes `dev → staging → production`. The promotion
workflows (`promote-dev-to-staging.yml`, `promote-staging-to-production.yml`)
land 2026-05-16 as UNCONDITIONAL fast-forwards.

That's wrong long-term. Per the FINAL-FINAL model:
- Layer 3 staging: "canary cohort exposure begins here"
- Layer 4 production: "full rollout"

The implicit invariant: **staging traffic on a canary cohort must
PASS observability thresholds before production promotion fires.**
ADR-0110/0111/0112/0113 all mark canary observability as FUTURE
without specifying it. ADR-0114 locks the design before
implementation begins on `oya-intelligence-canary-controller-{kernel,app}`
or the cell-cohort registry.

## Decision

A canary gate runs between every promotion event. The gate emits
one of four verdicts: `PROMOTE`, `ROLLBACK`, `EXTEND_OBSERVATION`,
`ESCALATE`. The verdict conditions whether the downstream
promotion workflow advances.

### 1. Cohort selection (per-cell)

Oyatie's cell architecture (per ADR-0033 + cell-domain crates)
gives a natural canary mechanism: each cell is independent;
different cells can pin different staging-or-production refs.

Cohort definition: a YAML registry at
`registry/cells/canary-cohort.yaml`:

```yaml
products:
  oya-foundry:
    canary_cells:
      - cell-canary-foundry-1     # smallest cell, internal-only
    expansion_stages:
      - { stage: 1, cell_count: 1,  percent_of_total: 0.05 }
      - { stage: 2, cell_count: 5,  percent_of_total: 0.10 }
      - { stage: 3, cell_count: 25, percent_of_total: 0.25 }
      - { stage: 4, cell_count: "all", percent_of_total: 1.00 }
    observation_window_per_stage_seconds: 900   # 15 min default
  oya-workflow-studio:
    canary_cells:
      - cell-canary-workflow-1
    expansion_stages: [ ... same shape ... ]
    observation_window_per_stage_seconds: 1800  # 30 min for higher-stakes product
```

Each product has its OWN canary cohort + expansion schedule + obs
window. Foundry and Workflow Studio can be at different stages
independently.

### 2. Signal sources

The canary controller subscribes to **four signal classes** per
canary cell vs control cell (same product, same region, on
production ref):

| Signal | Source | Aggregation |
|---|---|---|
| Latency | OpenTelemetry per-endpoint p99 | 5-minute rolling window per cell |
| Error rate | HTTP 5xx + Rust panic events + audit-chain `Err` returns | per-cell per-minute count / total |
| SLO breach | `registry/slo/*.yaml` defined per-product | live SLO evaluator output |
| Per-product KPI | Workflow Studio: failed-flow rate. Foundry: agent-invocation failure rate. Workflow Engine: step-retry rate. | per-product registry-defined |

Signal collection is a separate lane
(`oya-governance-canary-signal-emission`, in wave-A scope)
that asserts every product publishes the 4 signal classes.

### 3. Threshold logic

Per-product thresholds in `registry/canary/thresholds.yaml`:

```yaml
products:
  oya-foundry:
    latency_canary_to_control_ratio_rollback: 1.20    # canary p99 > control p99 × 1.20 → ROLLBACK
    error_rate_canary_to_control_ratio_rollback: 1.50  # canary error rate > control × 1.50 → ROLLBACK
    slo_breach_count_rollback: 1                       # any SLO breach → ROLLBACK
    kpi_canary_to_control_ratio_rollback: 1.30
    promote_clean_window_seconds: 900                  # 15 min zero anomalies → PROMOTE
    extend_observation_max_seconds: 3600               # extend up to 1 h before ESCALATE
```

The thresholds are deliberately conservative for v1; per-product
tuning is iterative (ADR-amended).

### 4. Decision protocol

The canary controller evaluates every 30 s. Decision tree:

```
For each (product, stage):
  signals = collect last 5-min window for canary cells vs control cells
  if any threshold breached:
    if first_breach_at + grace_period (60s) > now:
      verdict = EXTEND_OBSERVATION   # wait one more eval window
    else:
      verdict = ROLLBACK
  elif clean_window_elapsed >= promote_clean_window_seconds:
    verdict = PROMOTE                # advance to next stage
  elif observation_elapsed >= extend_observation_max_seconds:
    verdict = ESCALATE               # request human review
  else:
    verdict = EXTEND_OBSERVATION
```

The verdict is emitted as a `changeset-event-log` row (per ADR-0110)
on the staging→production transition's behalf, with
`emitted_by: oya-intelligence-canary-controller-app`.

### 5. Rollback mechanism

Two paths, both signed + audited:

**Canonical rollback (per-ADR-0110 monotonicity):** a rollback is
a NEW changeset that reverts the offending commit. The canary
controller opens a PR titled `revert: <original-changeset-id>
canary failure` against `dev`. That PR goes through the normal
pipeline (CI → review → merge-queue → promote). Pros: monotonic
state-machine preserved. Cons: full pipeline cycle (~30 min) to
land the revert.

**Emergency rewind:** `oya canary force-rewind --target <ref>
--to-sha <sha> --justification <text>` rewinds the staging or
production ref to a prior SHA. This:
- Requires a human signing key (Ed25519, per ADR-0058).
- Records a special `canary-emergency-rewind` event in the
  changeset-event-log AND in `registry/canary/rewind-log.json`.
- Alarms via `oya-governance-canary-emergency-rewind-frequency`
  lane (alerts if >2 rewinds per 30 days — signal that canary
  controller is unreliable + needs investigation).
- Force-push is technically allowed for emergency rewind only via a
  short-lived branch-protection bypass (deployed via `gh api
  branches/<ref>/protection -X PATCH allow_force_pushes=true`
  for 15 min, then auto-reverted). Captured in
  `registry/canary/rewind-log.json` for audit.

Most failures use canonical rollback; emergency rewind is for
production breakages that can't wait 30 min.

### 6. Cohort expansion (graduated)

Per-stage expansion runs sequentially, gated by the same verdict
protocol at each stage:

```
stage 1 (1 cell, 5%)  → observation → PROMOTE → 
stage 2 (5 cells, 10%) → observation → PROMOTE → 
stage 3 (25 cells, 25%) → observation → PROMOTE → 
stage 4 (all cells, 100%) → terminal "produced" state on the changeset
```

Each stage's observation window is per-product (see registry
example above). At any stage, ROLLBACK reverses ALL exposed
cells back to production-HEAD, not just the most-recent stage.

### 7. Cell-architecture integration

- Each cell stores `target_ref: dev | staging | production`
  in its config.
- Canary controller's PROMOTE verdict updates the cell's
  `target_ref` (e.g., flips a canary cell from `production` to
  `staging` at stage 1; flips all-cells from `staging` to
  `production` at terminal stage).
- The cell deployment infrastructure (per ADR-0033) observes
  `target_ref` changes and re-deploys.
- Per-product canary controllers are independent — Foundry can
  be at stage 3 while Workflow Studio is at stage 1, OR Foundry
  is rolled back while Workflow Studio is fully promoted.

## Consequences

### Positive

- Canary failures contained to ≤5% of traffic (stage 1) before
  any human notice. Mean Time To Detect: <15 min.
- Per-product independence — bad Workflow Studio canary doesn't
  block Foundry promotion.
- Mode-canonical: every promotion is either auto-PROMOTE or
  auto-ROLLBACK or ESCALATE-to-human. No "we forgot to check"
  failure class.
- Emergency rewind is alarmed; over-use signals an underlying
  problem.

### Negative

- New crate family
  (`oya-intelligence-canary-controller-{kernel,app}` + signal-emission
  lane + thresholds registry + rewind-log) — ~600 LOC + 4 new
  lanes.
- Observability backend assumed (OpenTelemetry collector
  per-cell). Today's cells don't all emit OTel; backfill required.
- Per-product threshold tuning is iterative; v1 conservative
  defaults may rollback too aggressively. Each false-rollback
  costs a pipeline cycle (~30 min).
- The "control cells" baseline assumes there ARE control cells
  (i.e., not every cell is canary at the same time). Stage 4
  full-rollout collapses the control set to zero; the controller
  switches to historical-baseline comparison at that point.

### Neutral

- Two rollback paths (canonical + emergency) is the right trade
  given canary failure latency requirements differ from normal
  pipeline cadence. Documented as such; not a code-path
  multiplication.

## Implementation sequencing

- **Wave A** (this ADR Accepted):
  1. `oya-intelligence-canary-controller-kernel` — pure-domain
     threshold evaluator + verdict emitter.
  2. `oya-intelligence-canary-controller-app` — runner; subscribes to
     signal sources, emits verdicts every 30 s, writes
     `changeset-event-log` rows.
  3. `registry/cells/canary-cohort.yaml` + `registry/canary/thresholds.yaml`
     — initial config for the first 2 products (Foundry + VCS).
  4. `oya-governance-canary-signal-emission` lane — asserts
     every product publishes the 4 signal classes.
- **Wave B**:
  - Gate the promotion workflows on the controller's verdict.
    `promote-dev-to-staging.yml` + `promote-staging-to-production.yml`
    add a step that queries the controller endpoint and refuses
    to advance if verdict ≠ PROMOTE.
  - `oya canary rollback` subcommand for canonical rollback PR
    creation.
  - `oya canary force-rewind` subcommand with human-signature
    requirement + automatic protection-bypass-then-restore.
- **Wave C**:
  - `oya-governance-canary-emergency-rewind-frequency` lane
    (alarm).
  - `oya-governance-canary-thresholds-tuned` lane (asserts
    every product's thresholds have been reviewed in the last 90
    days — catches stale config).
  - Per-product KPI registry (Workflow Studio failed-flow rate,
    Foundry agent-invocation failure rate, etc).

## Naming justification

- `oya-intelligence-canary-controller-{kernel,app}` — `oya-foundry-`
  product, `canary-controller` concept, role suffix.
- Lane id `oya-governance-canary-signal-emission` — fitness
  family prefix.
- Registry paths `registry/cells/canary-cohort.yaml` +
  `registry/canary/thresholds.yaml` — namespaced under `cells/`
  and `canary/` (matching existing registry conventions like
  `vcs/`, `quality/`).
- Subcommands `oya canary rollback` + `oya canary force-rewind`
  — extends the `oya canary` namespace (new); kebab-case verb
  per ADR-0105.

## Open questions

1. Should canary cohort `cell_count` be absolute or percentage?
   **Decision: both, per-stage** — early stages benefit from
   absolute counts (1, 5, 25 cells); later stages benefit from
   percentages (75%, 100%). Schema supports both.
2. What if the canary controller itself fails? **Decision: it's
   a fail-CLOSED gate** — if the controller endpoint doesn't
   respond within 10 s, the promotion workflow refuses to advance
   (treats it as `EXTEND_OBSERVATION` indefinitely until controller
   recovers). Aligns with `feedback_no_silent_regression`.
3. Should canary verdicts feed back into IP-005 (CI fix-loop)?
   **Decision: YES** — a ROLLBACK verdict on canary failure is
   surfaced as an "agent-fixable failure" the next time the
   reverted changeset's author re-attempts (the failure-evidence
   bundle includes the canary signal that triggered ROLLBACK).
4. What's the contract during a cell-deployment in-flight (cells
   are still rolling, not all canary cells on staging yet)?
   **Decision: controller waits for `cell_deployment_complete`
   event from the cell-deployment infra (per ADR-0033) before
   beginning observation**. Half-deployed cohorts produce
   unreliable signals.
