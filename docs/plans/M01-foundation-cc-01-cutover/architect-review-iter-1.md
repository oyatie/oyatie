---
doc_status: published
---

# Architect Review — ralplan-oyatie-sst-consolidation (Iteration 1)

<!--
status: Accepted
date: 2026-05-12
related_adrs: ADR-0052, ADR-0053, ADR-0054, ADR-0055
-->

Captured 2026-05-12. Reviewer: oh-my-claudecode:architect. Verdict: **ITERATE**.

---

## Summary verdict
ITERATE-WITH-REVISIONS. Option A's strict-phased archive-first sequencing is the right call, the verification matrix lines up 1:1 with A1-A10, and the principle-citation discipline is visible at every phase. But three load-bearing holes will surface mid-cutover and stall agents:

1. Plan never references the **scaffold-claim pattern** from `pre-cutover-drafts §Draft 2`, so P2's "claim `tools/oya-agent-read/src/main.rs::main`" is impossible-at-claim-time (the file does not exist yet, hence is not in the grit symbol index, hence the claim will FK-fail exactly like the orchestrator already reproduced).
2. The "two consecutive CI green runs" gate at P7 is unfalsifiable as written — no definition of *which* CI runs count.
3. P3 lets the bidirectional-citation lane crate (`oya-foundry-fitness-portfolio-citation`) be scaffolded under the same chicken-and-egg as P2 without acknowledging the pattern.

Fix those three and the plan is approvable.

## Steelman antithesis: Option D — invert P2 and P4
Rewrite agent-facing memory FIRST, ship `oya-agent-read` SECOND. Argument: the cutover's *own* execution agents are bound by the rule the moment any agent runs P2, but P2 *is* the moment the rule needs to be relaxed (P2 builds the helper that the rule references). Inverting makes the rule self-consistent: the rule applies to *steady-state* agent operations; the cutover itself is a privileged window. Naming that window in P4 first makes the rule honest. Doing P2 first hides the contradiction.

**Architect lands on**: Option A wins on rollback-boundary cleanliness, but only if the plan **explicitly tags the P1-P3 window as a "cutover bootstrap window" in the ADR** so the meta-contradiction is acknowledged rather than ignored.

## Real tradeoff tensions
1. **Helper-first (P2 before P4) vs. rule-first (P4 before P2).** Plan picks P2-first; defensible but requires acknowledging the bootstrap-window meta-contradiction in the ADR.
2. **Archive + delete same-PR (Option C) vs. archive-then-delete two-PR (Option A).** Two-PR buys safety only if the second PR catches a problem. P7's gate as written does not actually catch deletion mistakes — it catches banned-token-leaks, which is orthogonal. Option A's safety story is partially fictive vs. Option C.
3. **`grit done` semantics vs. PR-preserving merge.** The cutover itself does not flow through `claim → work → done` — it uses human-orchestrator `git mv` / `git rm`. First demonstration of the new pipeline is itself outside the pipeline.

## Synthesis
- **Tension 1**: Keep P2 before P4 but add the bootstrap-window clause to the ADR. Revision request #2.
- **Tension 2**: Keep two-PR split but redefine P7 gate concretely (banned-primitives lane green + new archive-orphan lane + per-row `archived_at` timestamps non-null). Revision request #3.
- **Tension 3**: Name the cutover as a one-time manual bootstrap in the ADR §Consequences §Neutral. Revision request #4.

## Principle violations (deliberate-mode)

- **VIOLATION 1** — P2 symbol-claim list (`tools/oya-agent-read/src/main.rs::main`, ...) is impossible at claim-time (file does not exist; not indexed by `grit init`; FK error reproduces deterministically on non-indexed symbols). Violates §Constraints items 1 and 5.
- **VIOLATION 2** — P3 has the same chicken-and-egg as P2 and never acknowledges it (`crates/oya-foundry-fitness-portfolio-citation/src/lib.rs::check`, `::main`).
- **VIOLATION 3** — P10 has the same problem a third time (`crates/oya-foundry-fitness-authoritative-tracked/src/lib.rs::check`, `::main`).
- **VIOLATION 4** — ADR §Decision is literally false during P1-P2. The sanctioned set during that window is `{grit, icm}` plus implicit bootstrap-window carve-out for read-side `git`/`gh`. Plan does not name this.

## Load-bearing-question answers

| Q | Answer | Detail |
|---|---|---|
| A. Chicken-and-egg handled? | **NO** | Plan does not cite Draft 2; P2/P3/P10 all violate. Load-bearing failure. Fix: lift Draft 2 to ADR and reference from P2/P3/P10. |
| B. Carve-outs at P6/P7/P9? | **PARTIAL** | Tight scope (only `git mv`/`git rm`/`gh issue create`); flagged inline. But plan does not name *who* is allowed to be the orchestrator or whether orchestrator auth is itself an icm event. Fix: revision request #5. |
| C. P4 precedes P6/P7? | **YES** | Correctly sequenced. Banned-primitives lane enforceable at P4 land time. |
| D. "Two green CI runs" defined? | **NO** | Unfalsifiable. Fix: revision request #3. |
| E. LEDG-008/017/021/024 preserved? | **YES** | Clean. ADR §Why Chosen line 175 explicitly preserves. |
| F. Parallel-claim demo demonstrates A7? | **NEARLY** | Plan doesn't cite Draft 3. Per-symbol locking within a single file should be explicit. Fix: revision request #6. |
| G. `oya-agent-read` surface bounded? | **YES** | Exactly `log`/`diff`/`pr-view`/`pr-comments`. No write verbs. Clean. |
| H. Linus-audit complete? | **PARTIAL** | P5 and P10 lack data-shape justifications. Fix: revision request #7. |

## Specific revision requests

1. **Add P0.5 (or fold into P1)**: land `registry/placeholder-debt/adr-follow-ups.yaml#grit-scaffold-claim-pattern (superseded by ADR-0116)` (lift from `pre-cutover-drafts §Draft 2`) before P2. Without this, P2's claim-list is impossible. Satisfies A2; unblocks A4.
2. **Amend ADR §Decision**: add "cutover bootstrap window" clause naming P1-P2 as the bootstrap interval during which the sanctioned set is `{grit, icm}` + audited carve-out; banned-primitives lane activates at P5 merge. Without this, ADR is literally false during cutover. Satisfies A5/A6 with clear activation timing.
3. **Replace P7's gate** with: "P7 merges only after (a) banned-primitives lane green on main HEAD post-P6-merge, (b) `oya-foundry-fitness-archive-orphan` lane confirms no living code/docs/configs reference archived paths (lane scaffolded at P6), (c) inventory ledger's per-row `archived_at` timestamp non-null for every ARCHIVE-class row." Without this, gate is unfalsifiable. Satisfies A3; tightens A2.
4. **Add ADR §Consequences §Neutral**: "The cutover itself runs under a one-time human-orchestrator carve-out for `git mv`/`git rm`/`gh issue create` (P6/P7/P9). Post-cutover, lifecycle is `grit claim → work → grit done`; cutover commits are not retroactively flowed through `grit done`." Documentation hygiene. Supports A10.
5. **Define "human orchestrator"** in ADR §Decision Drivers or §Glossary: named individuals authorized in `oyatie/docs/RACI-OWNERSHIP.md`; each carve-out invocation recorded via `icm store -t cutover-orchestrator-actions -c '<action>' -i critical` before execution. Without this, "orchestrator" is rhetorical cover. Satisfies §Constraints item 1 by making carve-out scope literal and audited.
6. **P8: pin demo symbols** to Draft 3's concrete proposals: `crates/oya-cloud-billing-app/src/lib.rs::CloudBillingEventIngestAppStatus` and `::CloudBillingMeterUnitRecord`. Add one sentence to runbook outline: "Demonstrates per-symbol locking within a single file, satisfying A7's non-overlapping-symbols criterion." Reuse Draft 3 as runbook seed. Without this, demo is deferred and risks compile-fail mid-cutover. Satisfies A7 with concrete artifact.
7. **Add to P5 and P10 acceptance evidence**: name the data-shape reshape — P5 reshapes "scattered git/gh references in agent skills" → "single grit/icm/helper invocation pattern"; P10 reshapes "authoritative state spread across tracked-and-ignored paths" → "authoritative ≡ tracked." Without this, A10's Linus-audit is incomplete for two phases. Satisfies A10.
8. **Mark `adr-draft-grit-icm-sanctioned-primitives.md` as the canonical lift-source.** Plan currently re-derives ADR text inline at §3 (lines 153-189), risking drift from the pre-draft. Either delete the inline §3 or delete the pre-draft, but not both. Cleanup. Supports A2 (single ADR source).

## Architect verdict
**ITERATE**

---

## Note to Critic
The orchestrator separately resolved three mechanical open-questions items in `open-questions-resolutions-2026-05-12.md` (next ADR slots are 0052/0053/0054, helper crate language is Rust as `oya-tooling-agent-read`, scaffold-claim fallback is icm-coordination-lock because Cargo.toml is verified-not-indexed by grit). The Architect's revision requests are consistent with those resolutions.

**Cross-cutting orchestrator note**: the user issued a mid-loop directive — "Make sure to read ultragoal from both dir for consolidated foundry spec." This expands the cutover scope: the foundry-relevant content in `bominal/agents/ultragoal/*` and `oyatie/.omx/ultragoal/*` must be salvaged into the canonical foundry SPEC (or a new `docs/products/foundry/SPEC.md`) BEFORE the archive/delete phases (P6/P7) destroy the source material. The Critic should require a new phase **P3.5 — Salvage ultragoal foundry content into canonical SPEC** as a precondition for P6.
