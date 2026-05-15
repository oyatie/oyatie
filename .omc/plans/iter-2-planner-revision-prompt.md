---
purpose: Auto-backfilled purpose for iter-2-planner-revision-prompt.md
---

# Iteration-2 Planner revision prompt — ralplan-oyatie-sst-consolidation

Pre-drafted 2026-05-12 while Critic runs. Used when Critic returns ITERATE and the Planner must revise. Skip if Critic returns APPROVE.

---

## Skill invocation

```
Agent(
  description: "Ralplan Planner iteration 2 — incorporate Architect + Critic feedback + user directive",
  subagent_type: "oh-my-claudecode:planner",
  prompt: <body below>,
  run_in_background: true
)
```

## Prompt body

```
You are the **Planner** in iteration 2 of the ralplan `--consensus --direct --deliberate-auto` loop. Revise the plan at `/Users/jasonlee/oyatie/.omc/plans/ralplan-oyatie-sst-consolidation.md` IN PLACE based on Architect + Critic feedback + a mid-loop user directive. Do NOT execute, do NOT mutate source code, do NOT call git/gh.

## INPUTS

1. `/Users/jasonlee/oyatie/.omc/plans/ralplan-oyatie-sst-consolidation.md` — your iter-1 plan
2. `/Users/jasonlee/oyatie/.omc/plans/architect-review-iter-1.md` — Architect's review (verdict ITERATE; 4 violations; 8 revision requests)
3. `/Users/jasonlee/oyatie/.omc/plans/critic-review-iter-1.md` — Critic's evaluation (verdict {{CRITIC_VERDICT}}; load-bearing triage; new P3.5 requirement)
4. `/Users/jasonlee/oyatie/.omc/plans/open-questions-resolutions-2026-05-12.md` — mechanical resolutions: ADR slots 0052/0053/0054; Rust helper named `oya-tooling-agent-read`; scaffold-claim fallback is icm-coordination-lock (Cargo.toml verified-not-indexed)
5. `/Users/jasonlee/oyatie/.omc/scratch/pre-cutover-drafts-2026-05-12.md` — Draft 2 (scaffold-claim) and Draft 3 (demo symbols) are lift-sources
6. `/Users/jasonlee/oyatie/.omc/scratch/foundry-salvage-from-ultragoal-2026-05-12.md` — foundry content extracted from both ultragoal dirs (per user mid-loop directive)
7. `/Users/jasonlee/oyatie/.omc/scratch/deep-dive-oyatie-sst-consolidation.md` — the spec (unchanged)

## REQUIRED REVISIONS (must land in iter-2)

### Architect violations 1-4 (all four are gating)
- **V1 P2 chicken-and-egg**: P2 must adopt the scaffold-claim pattern. Move scaffold-claim to canonical-for-new-crate-phases. Cite Draft 2 in P2 by path.
- **V2 P3 chicken-and-egg**: P3 scaffolds `oya-foundry-fitness-portfolio-citation-kernel` — apply same scaffold-claim pattern. Cite Draft 2.
- **V3 P10 chicken-and-egg**: P10 scaffolds `oya-foundry-fitness-authoritative-tracked-kernel` — apply same scaffold-claim pattern. Cite Draft 2.
- **V4 ADR false during P1-P2**: amend ADR draft per Architect revision request #2 — add "cutover bootstrap window" clause naming P1-P2 as the bootstrap interval; banned-primitives lane activates at P5 merge.

### Architect revision requests (all 8)
1. **Add P0.5** (or fold into P1): land ADR-0054 grit-scaffold-claim-pattern BEFORE P2. Lift from Draft 2.
2. **ADR §Decision bootstrap-window clause** — see V4 above.
3. **Replace P7 gate** with: (a) banned-primitives lane green on main HEAD post-P6-merge, (b) new `oya-foundry-fitness-archive-orphan` lane (scaffold at P6) green, (c) inventory ledger per-row `archived_at` non-null for every ARCHIVE row.
4. **Add ADR §Consequences §Neutral**: cutover is one-time human-orchestrator carve-out; not retroactively flowed through `grit done`.
5. **Define "human orchestrator"** in ADR §Glossary: named in `oyatie/docs/RACI-OWNERSHIP.md`; each carve-out invocation `icm store -t cutover-orchestrator-actions -c '<action>' -i critical` BEFORE execution.
6. **P8 pin demo symbols** to Draft 3: `oya-cloud-billing-app/src/lib.rs::CloudBillingEventIngestAppStatus` + `::CloudBillingMeterUnitRecord`. Reuse Draft 3 script as runbook seed.
7. **Add to P5 and P10 Linus-data-shape lines**: P5 = "scattered git/gh references in agent skills" → "single grit/icm/helper invocation pattern"; P10 = "authoritative state spread across tracked-and-ignored paths" → "authoritative ≡ tracked."
8. **ADR single source of truth**: delete inline §3 ADR Block; reference `/Users/jasonlee/oyatie/.omc/scratch/adr-draft-grit-icm-sanctioned-primitives.md` as canonical lift-source.

### New P3.5 phase (per user mid-loop directive)
**Phase ID**: P3.5 — Salvage ultragoal foundry content into canonical SPEC.
**Inputs/preconditions**: P1 (inventory ADR committed; foundry salvage draft exists at `.omc/scratch/foundry-salvage-from-ultragoal-2026-05-12.md`).
**Symbols to claim**: doc-only — coordinate via `icm-lock-P3.5` topic. Target file is `oyatie/docs/SPEC.md` §Foundry (existing section to expand) OR new `oyatie/docs/products/foundry/SPEC.md` (creation under scaffold-claim if Cargo.toml-equivalent-for-docs is not lockable).
**Agents and parallelism**: 1 agent; serial gate before P6 (must land before archive/delete destroys ultragoal sources).
**Outputs / acceptance evidence**: foundry SPEC content (Phase 00 ProviderAccount/AuthSession/UsageWindow/SecretReference contract surface, account-auth design, visibility surface, gates/validators/evidence template shape, transport parity manifest) committed to oyatie/docs/; inventory ledger row for `bominal/agents/ultragoal/2026-05-12-foundry-ultragoal-mega-plan.md` and `oyatie/.omx/ultragoal/**` updated from ARCHIVE/DELETE to ARCHIVE+SALVAGED (new classification, or annotated note); the cutover ADR cites the salvage as a precondition for P6.
**Maps to spec criterion**: A2 (inventory completeness covers salvage), A3 (archive precedes delete with audit evidence — salvage prerequisite).
**Linus data-shape line**: "Ephemeral planning-corpus state in two directories" → "canonical product-SPEC authority in single tracked doc tree."

### Critic open-questions handling
- **Q3 (carve-out scope)**: leave flagged for user; do NOT block iter-2 on it.
- **Q6 (retention policy)**: leave flagged for user; do NOT block iter-2 on it.

## OUTPUT

Rewrite `/Users/jasonlee/oyatie/.omc/plans/ralplan-oyatie-sst-consolidation.md` IN PLACE. Mark iteration as "Iteration 2 — incorporates Architect + Critic + user mid-loop directive". Status remains `pending approval`.

Return a 200-word delta summary (NOT the full plan — just what changed iter-1 → iter-2) so the parent orchestrator can dispatch Architect for iter-2 review.

Hard rules: do NOT edit source code; do NOT call git/gh; do NOT mutate state outside .omc/plans/; do NOT skip any of the 11 above-listed revisions; if any revision conflicts with another, FLAG it in the delta summary.
```

---

## Notes for orchestrator

- The `{{CRITIC_VERDICT}}` token is filled in at dispatch time from Critic's returned verdict (ITERATE or REJECT).
- If Critic returns APPROVE outright, skip iter-2 and proceed to user execution-approval AskUserQuestion.
- If Critic returns REJECT (unrecoverable), surface to user with the Critic's reason and pause the loop.
- iter-2 dispatched in background; while it runs, the orchestrator may do further read-only prep (e.g., dispatch the iter-2 Architect prompt skeleton, audit the foundry-salvage output for completeness).
- Loop max: 5 iterations per ralplan skill rule. Currently at iter 1 → 2; budget = 3 more before manual escalation.
