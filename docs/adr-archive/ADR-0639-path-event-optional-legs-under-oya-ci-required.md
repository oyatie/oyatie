---
id: ADR-0639
title: "Path- and event-conditional constituents under the single oya-ci-required fan-in"
status: Superseded
doc_status: published
planning_impact: true
deciders: founder
owner: council-architecture
date: 2026-08-05
door: two-way
supersedes: []
superseded_by: [ADR-700]
depends_on: [ADR-0515, ADR-0554, ADR-0630]
amends: [ADR-0515]
related: [ADR-0539, ADR-0541, ADR-0555, ADR-0636]
related_specs:
  - /.github/workflows/oya-ci-required.yml
  - /specs/phase0-ci-enforcement-baseline.json
milestone: W0
---

# ADR-0639: Path- and event-conditional constituents under the single `oya-ci-required` fan-in

## Baseline version header

| Authority | Version this ADR is authored against | Status at authoring (2026-08-05) |
|---|---|---|
| Repository baseline | `origin/dev@a1bd1f14a` | CAS proof-cell packaging promoted (#1558). |
| Protected context | `oya-ci-required` (ADR-0515) | Single required merge context; unchanged by this decision. |
| Runner substrate | ADR-0630 ARC interim | Dual-worker general capacity is amended separately on ADR-0630. |
| Path-optional pilot | PR #1562 (live-postgres) | Implementation intent; must cite this ADR when merged. |

## Status

**Accepted — 2026-08-05.** Founder-accepted plan for CI path/event tiers under the singleton fan-in. This decision authorizes **classified optional legs** inside `oya-ci-required`. It does **not** authorize a second required context, docs-only free passes over admission gates, warm CAS, or RE.

## Context

The single fan-in workflow schedules almost every expensive job on every PR. That is correct for admission faces and total-accounting, but it forces docs-only and path-irrelevant PRs to occupy scarce self-hosted runners (including the disk-constrained live-postgres cell). Asterinas-style multi-workflow path filters show specialty lanes should not all sit on the PR critical path; ADR-0515 still forbids multiple protected contexts.

ADR-0554 already permits CONE vs fail-closed FULL **inside** the affected-set job. That is orthogonal: it does not skip the job. This ADR defines when a **registered fan-in job** may be skipped on pull_request without reddening the aggregate, while trunk remains full-proof.

## Decision

### D1 — Singleton preserved

Branch protection and merge admission continue to require **only** the context name `oya-ci-required`. No second required check for “docs CI,” “fast CI,” or “full CI.” Optional legs remain **constituents** of the same fan-in aggregator.

### D2 — Leg classes

Every job listed in the `oya-ci-required` fan-in `needs:` list MUST be classified as exactly one of:

| Class | Symbol | Meaning |
|-------|--------|---------|
| Always-on admission | **A-always** | Must `success` on PR and trunk. |
| Path-optional on PR | **P-path** | May be skipped on `pull_request` when the change cone is path-irrelevant; **must run** on `push` / `merge_group` / `workflow_dispatch` to `dev` (trunk full proof). |
| Event-optional | **E-event** | Runs only for named events (e.g. trusted push cache-writer). |
| Schedule-only | **S-schedule** | Not required for PR merge; not a fan-in blocker for ordinary PRs. |

**Initial classification (v1 — may grow only via amendment or successor ADR):**

- **A-always:** producer-regen, freshness, registry-drift, cloud-ci-firewall (baseline ratchet), generated-output-diff-policy, buck2 (hermetic + affected tests), gate-affected-target-set, ADR census (when registered), and the fan-in job itself.
- **P-path:** gate-live-postgres-adapters, gate-live-postgres-facades (durable postgres / shared paths / ARC live-postgres / this workflow file — exact prefixes owned by the workflow).
- **E-event:** cache-writer identity (trusted `dev` push only) — already practiced.
- **S-schedule:** none required for merge in v1 (reserved for future heavy axes).

**Docs-only PRs remain A-always for the A-set.** Thinning buck2 or firewall for pure `docs/**` is **out of scope** until a later Accepted decision with false-green analysis.

### D3 — Fail-closed uncertainty

When path relevance cannot be resolved (missing base ref, empty/failed triple-dot diff, tool error), a **P-path** leg MUST **run**, not skip. Fail-open to *work*, never fail-open to *skip*.

### D4 — Fan-in algebra

For the aggregate `oya-ci-required` job to be green:

1. Every **A-always** leg that was scheduled: `result == success`.
2. Every **P-path** leg: `result == success` OR `result == skipped`.
3. Every **E-event** / **S-schedule** leg that was not scheduled: treated as non-blocking for that event.
4. Any scheduled leg with `result == failure` or `cancelled` (except explicit cancel-in-progress supersession of the whole run): **red**.

Skipped does **not** mean retired. Jobs remain registered; skip rate SHOULD be observable.

### D5 — No silent gate deletion

Path-optional is not removal from the gate registry, OWNERS, or accounting. Deleting a job from fan-in requires the same discipline as any admission change (registration + dual review + evidence).

### D6 — Cancel-in-progress

The workflow MAY use a per-PR concurrency group with `cancel-in-progress: true` so superseded heads do not hold scarce runners. Cancel of a superseded attempt is not a merge-green substitute for a later successful run on the current head.

### D7 — Relation to ADR-0554

ADR-0554 CONE/FULL selection remains binding **inside** gate-affected-target-set. This ADR does not add a skip path for that job in v1.

## Consequences

- Live-postgres path-filter (#1562 class) gains Accepted authority under the singleton.
- Docs PRs still pay A-always cost until capacity and future ADRs change the A-set.
- Trunk and merge_group retain full durable postgres proof.
- Agents must not claim “docs CI green without admission gates.”

## Alternatives considered

- **Multiple required contexts (docs vs full).** Rejected: ADR-0515 multi-producer deadlock class; re-opens 0513-shaped mistakes.
- **Skip all non-docs paths including firewall/buck2 for docs/**.** Rejected for v1: false-green risk on faces, accounting, and census projections that docs PRs still touch.
- **Advisory path-filter only.** Rejected: does not free the maxRunners=1 postgres cell.

## Follow-ups

1. Land postgres path-filter implementation citing this ADR.
2. Dual-worker capacity (ADR-0630 amend) so A-always queue depth drops.
3. Concurrency cancel-in-progress on `oya-ci-required`.
4. Optional later: docs-cone study for further A-set thinning with explicit false-green analysis.

## Plan reference

`.grok/programs/hyperscaler-delivery-lanes/PLAN-CI-PATH-TIER-AND-CAPACITY.md` (founder-accepted 2026-08-05).
