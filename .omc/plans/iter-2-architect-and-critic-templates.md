# Iter-2 Architect & Critic dispatch templates

Pre-staged 2026-05-12 while iter-2 Planner runs. Used to dispatch the next two loop stages when Planner iter-2 returns.

---

## Iter-2 Architect dispatch

Inputs (DIFFERENT from iter-1):
- `/Users/jasonlee/oyatie/.omc/plans/ralplan-oyatie-sst-consolidation.md` — iter-2 plan (revised in place)
- `/Users/jasonlee/oyatie/.omc/plans/architect-review-iter-1.md` — iter-1 Architect review (for continuity check)
- `/Users/jasonlee/oyatie/.omc/plans/critic-review-iter-1.md` — iter-1 Critic review
- `/Users/jasonlee/oyatie/.omc/plans/iter-2-delta-summary.md` — iter-2 Planner's 250-word delta summary (will be created by Planner)
- All iter-1 sources (spec, drafts, inventory, foundry-salvage, ADR-draft, orchestrator-existence-findings, open-questions-resolutions) — unchanged context

Prompt skeleton:

```
You are the **Architect** in iteration 2 of the ralplan consensus loop. Iter-1 Architect verdict was ITERATE with 4 violations + 8 revision requests. Iter-1 Critic verdict was ITERATE with concurrence + 4 additional findings. Iter-2 Planner has revised the plan in place to address 12 MUST-LAND revisions.

Your job: verify each of the 12 revisions actually landed and is sound. Specifically:

- V1/V2/V3 (chicken-and-egg at P2/P3/P10): does the plan now cite Draft 2 + ADR-0054 for each new-crate phase? Are claim-list symbols replaced with icm-coordination-lock topics where the file doesn't exist yet?
- V4 (ADR bootstrap-window clause): did the Planner edit `.omc/scratch/adr-draft-grit-icm-sanctioned-primitives.md` to add the clause? Or did they only edit the plan?
- Arch rev #1-#8: each one tracked. Use the iter-1 review's revision list as a checklist.
- Critic finding #1-#4: each one tracked.
- P3.5 (cross-cite refinement, not salvage-from-destruction): does the new phase have the correct deadline (BEFORE P1, not BEFORE P6)?
- Phantom-path correction: did the inventory-draft phantom path get corrected, or only the plan?

Verdict: APPROVE | ITERATE | REJECT.

If APPROVE: the plan is ready for user execution-approval. Return verdict + 200-word summary.
If ITERATE: list specific revisions for iter-3.
If REJECT: name the unrecoverable issue.

Same output format as iter-1. Same hard rules.
```

---

## Iter-2 Critic dispatch (after Architect)

Inputs:
- iter-2 plan (post-Architect iter-2 review)
- iter-2 Architect review (saved to `.omc/plans/architect-review-iter-2.md`)
- iter-1 reviews (for continuity)
- All static sources

Prompt skeleton:

```
You are the **Critic** in iteration 2 of the ralplan consensus loop. Iter-1 Critic verdict was ITERATE with 4 new findings. Iter-2 Architect has just reviewed the iter-2 plan. Your job: final verdict.

Specifically verify:
- All 12 iter-1 MUST-LAND revisions are present in iter-2.
- No new violations introduced by iter-2 changes.
- ADR source-of-truth divergence resolved (Critic #1 from iter-1).
- Helper crate name unified across all docs (Critic #2 from iter-1).
- A6 hook/skill enumeration present (Critic #3 from iter-1).
- A9 explicitly Critic-attested (Critic #4 from iter-1).
- P3.5 phantom-path correction (the `oyatie/.omx/ultragoal/` line in inventory).
- New `oya-foundry-fitness-archive-orphan` lane defined (Arch rev #3).
- Demo symbols pinned to `CloudBillingEventIngestAppStatus` + `CloudBillingMeterUnitRecord` (Arch rev #6).

Verdict: APPROVE | ITERATE | REJECT.

If APPROVE, the plan moves to user execution-approval question (skeleton at `.omc/plans/user-execution-approval-question-skeleton.md`). If ITERATE, max 3 more iterations remaining (ralplan rule: 5 max).

Same output format as iter-1. Same hard rules.
```

---

## Flow control after iter-2

| Iter-2 Critic verdict | Next action |
|---|---|
| APPROVE | Dispatch user-execution-approval AskUserQuestion (skeleton ready). On user "Approve — execute via /autopilot", invoke `Skill("oh-my-claudecode:autopilot")` with `.omc/plans/ralplan-oyatie-sst-consolidation.md` as input. |
| ITERATE | Iter-3 loop. Save Critic-iter-2 review; dispatch Planner-iter-3. Max 3 more iters. |
| REJECT | Surface to user with REJECT reason and full review chain. Pause; await user decision (manual revision, scope reduction, or abandon). |

---

## Bookkeeping

After each iter completes, the orchestrator updates:
- TaskList: mark agent task complete, advance loop state.
- ICM: store the consensus state (`icm store -t context-oyatie -c "ralplan iter-N: <verdict>" -i high`).
- Optional: update /goal with current state. (User-invoked; cannot self-invoke.)
