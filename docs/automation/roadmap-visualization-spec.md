---
doc_class: VisualizationSpec
shape: visualization
length_cap: 200
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Render the wave sequence with milestone gates as a Mermaid Gantt chart.
  Source: MASTERPLAN.md §3 + each milestone INDEX.md + each phase INDEX.md.
  Lift to `docs/visualization/roadmap.md`. Auto-derived; no hand-painted Gantt.
planned_enforcement_ref: governance-roadmap-viz
extends_crates:
  - intelligence-architecture-map-kernel
  - intelligence-mdbook-kernel
companion_docs:
  - INDEX.md
  - dependency-graph-spec.md
  - ../../docs/MASTERPLAN.md
doc_status: published
---

# Visualization spec: roadmap (Mermaid Gantt)

> **ADRs:** ADR-0052, ADR-0053, ADR-0054.

## 1. Purpose

The roadmap is the single answer to "when does what ship." Today the answer is fragmented across `docs/ROADMAP.md`, `MASTERPLAN.md`, and individual milestone INDEXes. This pipeline lifts it to a single visual artifact, auto-derived, refreshed on every plan edit.

## 2. Inputs

- `docs/MASTERPLAN.md` §3 Milestone index table (canonical post-Stage-1-Wave-1 lift).
- Each milestone INDEX `.omc/plans/milestones/<MNN>/INDEX.md` frontmatter (status, wave, start_date_target, end_date_target).
- Each phase INDEX `.omc/plans/milestones/<MNN>/phases/<PNN>/INDEX.md` frontmatter (start_after, est_duration_days, status).
- `docs/ROADMAP.md` for wave-gate criteria.

## 3. Frontmatter conventions (additions)

Milestone INDEX adds:

```yaml
start_date_target: 2026-06-01
end_date_target: 2026-09-30
status: open | active | gated | done
```

Phase INDEX adds:

```yaml
start_after: M01-P03   # phase id this phase waits on
est_duration_days: 21
status: open | active | done
```

## 4. Output rendering

### 4.1 Primary: Mermaid Gantt

```mermaid
gantt
  title Oyatie waves
  dateFormat YYYY-MM-DD
  axisFormat %Y-Q%q

  section M01 Foundation
  M01-P01 tenancy        :done,     m01p1, 2026-04-01, 30d
  M01-P02 identity       :active,   m01p2, after m01p1, 30d
  M01-P03 audit-chain    :          m01p3, after m01p2, 30d

  section M02 Foundry-Preview
  M02-P01 capability-reg :          m02p1, after m01p3, 45d
  M02-P02 policy-cedar   :          m02p2, after m02p1, 30d

  section M-CC Cross-cutting
```

Sections grouped by milestone; M-CC threaded as its own section with `:crit` tag where dependencies are critical-path.

### 4.2 Secondary: D2 wave-gate diagram

A separate D2 view showing wave-gate edges (M01 → M02 → M03 → M04 → M05 → M06) with gate-criteria labels. Each gate node lists the acceptance bullets from `docs/ROADMAP.md` §2.

### 4.3 Cumulative-progress chart

A Mermaid `xychart-beta` showing cumulative phase completion over time, broken by milestone.

## 5. Wave-gate annotation

Each wave gate is rendered as a diamond node containing:
- Wave id (e.g. `W-Foundry-Preview`).
- Gate criteria list (lifted verbatim from `docs/ROADMAP.md` §2 row).
- Current pass/fail state (from `governance-lane-rollup` aggregate).

## 6. Validation gates (`governance-roadmap-viz`)

1. **Frontmatter completeness.** Every milestone/phase INDEX has the required date/duration fields (BLOCKER).
2. **DAG validity.** `start_after:` references resolve; no cycles (BLOCKER).
3. **Status validity.** `status:` ∈ {`open`, `active`, `gated`, `done`} (BLOCKER).
4. **Wave coverage.** Every wave in `docs/ROADMAP.md` §2 has at least one phase assigned (HIGH).
5. **Generated drift.** Committed roadmap viz differs from re-rendered (BLOCKER).
6. **Critical-path integrity.** The critical path (M01 → M02 → ... → M06) renders as a continuous chain; gap → HIGH.

## 7. Trigger matrix

| Event | Action |
|---|---|
| Per-PR touching `docs/MASTERPLAN.md` or any milestone/phase INDEX | Re-render; lane runs. |
| Per-PR touching `docs/ROADMAP.md` | Re-render (wave-gate criteria may change). |
| Nightly | Full re-render; archive weekly snapshot for trend. |
| On phase `status:` transition to `done` | Re-render with green styling; auto-notify stakeholders. |

## 8. Snapshot history

Each Friday 17:00 KST, the pipeline archives the current Gantt SVG to `docs/visualization/archive/roadmap-<YYYY-Www>.svg`. The history powers retrospective "how did we trend" reviews at fortnightly stakeholder syncs.

## 9. Out-of-scope

- Per-engineer assignment (covered by team-internal tooling, not surfaced here).
- Risk-overlay on the Gantt (future enhancement; will cross from `docs/RISK-REGISTER.md`).
- Customer-facing roadmap (separate hand-curated `docs/CUSTOMER-ROADMAP.md`).
