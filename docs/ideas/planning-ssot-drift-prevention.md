# Planning SSOT — Drift Prevention

## Problem Statement
How might we keep `specs/masterplan.json` a faithful planning single-source-of-truth — automatically catching any ADR/spec/plan that drifts out of it — without the gate becoming gameable, high-maintenance, or just another doc that rots?

## Recommended Direction
**masterplan.json is the one planning authority; ADRs + canonical specs bind into it; plan.md/todo.md are projections; .omx/plans is scratch** (deep-interview decision). Drift is prevented by a **strict, curated, blocking `planning-ssot-coverage` gate** built as **Direction A: frontmatter-bound + bidirectional + supersession-aware**.

Binding lives in each ADR's own frontmatter (`planning_impact: true` + `masterplan_ref`), so the binding is co-located with the decision (no separate registry to drift — the SSOT single-master principle). The gate checks both directions (a planning ADR missing from masterplan, and a masterplan IP citing a missing/superseded ADR) and treats `Superseded-by-NNNN` as satisfied-by-successor (Nygard ADR immutability). This converts "we keep forgetting to update masterplan" (the audit found 8.8% ADR binding) into a mechanical CI failure.

## Key Assumptions to Validate
- [ ] Planning-impacting ADRs are a small curated subset (~tens, not 294) — count `planning_impact` candidates before flipping the gate to blocking.
- [ ] "Planning impact" has a crisp definition (changes sequencing / scope / surface set) so the tag isn't a rubber-stamp — require reviewer sign-off in the ADR.
- [ ] Existing master-plan-completion + planning-closure gates can be extended rather than duplicated.

## Minimum first slice
In: `planning-ssot-coverage` gate (bidirectional ADR↔masterplan + spec↔root-hub + supersession-aware); one-time `planning_impact` tagging pass on planning ADRs; bind the unbound wave-3 ADRs (incl ADR-0357) + ci-farm spec + deferred-surfaces into a structured `planning_authority` masterplan section. Out: auto-generating plan.md/todo.md (fast-follow).

## Not Doing (and Why)
- Bind all 294 ADRs — noise; only planning-impacting ones.
- A new `planning-index.json` authority — re-creates the multi-authority fragmentation we're eliminating.
- Auto-generate projections now — valuable but separable; ship the gate first.
- Block on advisory/superseded ADRs — supersession-aware, so retiring a decision never breaks the gate.
- Retire tasks/plan.md+todo.md — keep as declared subordinate projections.

## Open Questions
- Where does the `planning_impact` tag get its crisp boundary — in the ADR template, or a `decision-principles` rule?
- Should the gate be report-only for one cycle to size the backlog before blocking?
