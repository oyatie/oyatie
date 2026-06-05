---
doc_class: BackfillReplay
title: Backfill + Replay (Historical Lane Re-execution)
microservice: governance
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry + ops-security
deciders: axis-foundry, ops-security, council-architecture
related_adrs: [ADR-0110, ADR-0133]
related_artifacts:
  - microservices/governance/runbooks/industry-baseline-refresh.md
  - microservices/governance/runbooks/evidence-replay.md
  - microservices/governance/PRD.md
review_cadence: quarterly + per major rule-pack change
doc_status: published
---

# Backfill + Replay: governance µservice

## Purpose

Re-run historical lanes against new policy (rule-pack updates, baseline pin updates, new lanes). Distinct from `runbooks/evidence-replay.md` (which serves *existing* sealed evidence to auditors); this document covers *re-execution* of lanes producing *new* verdicts on *old* code.

## When to invoke

- **Policy update**: a rule pack version bumps (e.g., `oya-check-supply-chain` tightens SLSA enforcement). Need to re-evaluate historical PRs against the new rule pack.
- **New lane addition**: a new lane (e.g., `oya-check-industry-best-practice-conformance` at M01) needs historical baseline.
- **Baseline pin update** (per quarterly refresh; stricter direction): retroactively classify which historical PRs would fail under new pin.
- **Forensic investigation**: an incident requires re-running specific lanes against specific SHAs.

## Distinction from evidence-replay

| Operation | When | Output |
|---|---|---|
| **Backfill (this doc)** | New policy needs historical scoring | NEW verdicts; NEW Findings; NEW evidence |
| **Replay** (`runbooks/evidence-replay.md`) | Auditor or SME requests existing evidence | EXISTING signed Findings; no new verdicts |

## Pre-conditions

- Backfill requires a documented purpose. Default: refused.
- Per ADR-0133 §"first-run amnesty": historical violations from legacy SHAs are tagged `legacy-grandfathered` and are NOT retroactively blocking (do not refuse historical merges); they ARE remediation-tracked.
- Backfill writes its Findings to a separate `view = "backfill"` namespace; production admission-gate ignores backfill Findings.
- Per-backfill ADR required if results will be used in tenant-visible artifacts.

## Procedure

### Step 1 — Plan

Author a backfill plan document at `microservices/governance/audit/backfill-<id>.md`:

- Purpose (why backfill is needed).
- Scope: lanes to re-run + SHA range + µservice scope.
- Expected volume + cost (per `cost-budget.md` per-PR breakdown × N).
- Storage decision: where do new Findings persist (Postgres `findings_backfill` partition).
- Acceptance criteria.
- Approver: council-architecture (for any backfill touching production-tier policy).

### Step 2 — Provision

```bash
# Create a separate backfill workspace through the governance control plane.
# Inputs: backfill_id=<backfill-id>, plan_path=microservices/governance/audit/backfill-<backfill-id>.md
# Verification: Buck2/Prow evidence records the backfill workspace id.
```

Provisions:
- Postgres `findings_backfill_<id>` schema (separate from `findings`).
- S3 evidence bucket prefix `backfill/<id>/`.
- Logical lane-runner pool tag `backfill-<id>` (allows pool isolation; rate-limit independent of production lane runs).

### Step 3 — Dispatch

```bash
# Dispatch through the governance control plane.
# Inputs: backfill_id=<backfill-id>, lanes=<lane-list>, sha_range=<git-range>,
# microservices=<ms-list>, concurrency=10
```

Behaviour:
- Iterates SHAs in range.
- Dispatches lane runs against the historical workspace (re-creating the workspace at that SHA per lane run).
- Concurrency bounded; backfill never starves production lane runs (ARC pool fairness).

### Step 4 — Monitor

```bash
# Query control-plane backfill status for backfill_id=<backfill-id>.
```

Outputs:
- SHAs processed / total.
- Estimated time-to-completion.
- Per-lane verdict distribution.

Grafana dashboard `governance-backfill-<id>` is auto-created.

### Step 5 — Analyse

```bash
# Export the signed analysis artifact from the governance control plane.
# Output: evidence/multispectrum/backfill-<id>-analysis.json
```

Surfaces:
- Per-rule retroactive violation count.
- Per-µservice affected SHA count.
- Suggested remediation IP series.

### Step 6 — Remediation IP series

Filed at `microservices/governance/IP-M01-AUDIT-<axis>-<NNN>.md` per ADR-0133 §"Operational" pattern.

Per-IP scope: close ONE retroactive violation pattern across the affected µservices.

### Step 7 — Decommission

```bash
# Close backfill_id=<backfill-id> after the Buck2/Prow evidence bundle is archived.
```

Outcomes:
- Findings move to `findings_backfill_<id>_archived` (read-only).
- S3 prefix moves to cold-tier.
- Backfill record retained at `microservices/governance/audit/backfill-<id>.md`.

## Examples (representative)

### Example A — `oya-check-supply-chain` tightens to SLSA Build L4

Per ADR-#### (future): SLSA Build L3 → L4 upgrade.

- Plan: re-run `oya-check-supply-chain` against last 90 days of PRs.
- Expected volume: ~10k SHAs × ~5s per re-run = ~14 hours wall-clock at concurrency 10.
- Cost: ~$15 per `cost-budget.md` per-PR median × 10k SHAs.
- Expected outcome: ~5% of SHAs surface retroactive violation; remediation IP series filed; council-architecture approves new pin going forward.

### Example B — New `oya-check-industry-best-practice-conformance` lane (M01 launch)

- Plan: re-run new lane against last 30 days of PRs to establish baseline.
- Expected volume: ~3k SHAs × ~3s = ~2.5 hours at concurrency 10.
- Cost: ~$5 per `cost-budget.md` per-PR median × 3k SHAs.
- Expected outcome: per-axis baseline established; remediation IP series filed for legacy violations (with `legacy-grandfathered` severity per ADR-0133 §"Operational" first-run amnesty).

## Performance + cost discipline

| Constraint | Rule |
|---|---|
| Concurrency | Bounded; backfill must not starve production lane runs (ARC pool fairness 30% max for backfill) |
| Retention of backfill Findings | 1 year hot + 7 years cold (lower than production Findings) |
| Cost cap per backfill | $1000 per backfill without explicit ops-finops approval |
| Schedule | Off-peak only (between 0200–0700 KST) for large backfills |

## Per-policy decision rules

| Policy direction | Backfill outcome |
|---|---|
| Stricter rule | Findings retained at `legacy-grandfathered` severity; remediation tracked; not retroactively blocking |
| Softer rule | No backfill needed (relaxation never adds findings) |
| New lane | Findings retained at `legacy-grandfathered` severity; first-run amnesty per ADR-0133 |
| Backwards-incompatible (e.g., new BC introduced) | ADR-#### required; backfill approach decided per-ADR |

## Verification

- `buck2 build //:quality-lane-registry-authority-check # lane=backfill-replay-determinism` — exit 0; re-run of completed backfill produces same Findings (Invariant 1 + Invariant 11).
- Per-backfill audit record at `microservices/governance/audit/backfill-<id>.md`.

## References

- `microservices/governance/runbooks/evidence-replay.md` (existing-evidence replay; not this).
- `microservices/governance/runbooks/industry-baseline-refresh.md` (quarterly cadence).
- ADR-0110 (ChangeSet state machine).
- ADR-0133 §"Operational" first-run amnesty.
- `microservices/governance/policy/lane-execution.md` Invariant 1 + 11.
- `microservices/observability/backfill-replay.md` (shape reference).
