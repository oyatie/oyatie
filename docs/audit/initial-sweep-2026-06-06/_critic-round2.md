# CRITIC REVIEW — AMENDMENT-PLAN round 2 (against Architect round-2)

> Reviewer: critic lane, separate from the authoring lane and from the round-2 architect lane. Verdict basis: re-read the full round-2 AMENDMENT-PLAN, re-read the round-2 Architect review (`_arch-round2.md`), re-read the SSOT (`synthesis/decision-record-oyatie-canon.md`) end-to-end, re-read both round-1 reviews (`_arch-round1.md`, `_critic-round1.md`), and **independently re-derived the architect's load-bearing HIGH finding (FLAG-A) by grepping the SSOT myself** rather than trusting the architect's self-report. Founder rule honored: every claim below cites a file+line I read or a grep I ran this round; no phantom findings; I do not certify any "RULED" tag I did not personally find in the SSOT.

**Bottom line up front: the architect's new HIGH finding (FLAG-A) is TRUE, and I verified it independently. The plan asserts a founder ruling that does not exist in the SSOT, at the exact joint its entire wave architecture hangs on. This is a real principle-option-consistency + fair-alternatives defect, not a wording nit, and it changes what the founder is asked to sign. It is patchable (SY1), localized, and does not change the recommended order — which lands at ITERATE, not REJECT.**

---

## 1. INDEPENDENT VERIFICATION OF THE ARCHITECT'S HIGH FINDING (FLAG-A)

The architect claims (§1, §3 FLAG-A, §9) that the plan's load-bearing edge — "L2 foundry-rename must precede L1-refound (rename-before-reauthor), driver D-1" — is asserted as **RULED** in three places but is **not ruled in the SSOT**. Because three prior lanes already certified this edge as RULED, I refused to trust a fourth verdict on it. I grepped the SSOT myself this round.

**Grep results against `synthesis/decision-record-oyatie-canon.md` (verbatim, this round):**

- `double-work` → **ZERO matches.** The plan's line 53 puts the string *"the dependency note 'L2 foundry-rename must precede ADR-0000+ re-author to avoid double-work'"* in quotation marks as if quoting the SSOT. **That note does not exist in the SSOT.** This is a fabricated quotation at the load-bearing joint.
- `re-?author` → 2 matches, **neither an ordering ruling**: line 4 (boilerplate "supersede/re-author into the clean `ADR-0000+` series; never in-place") and line 34 (D6: "re-author the phantom Cedar-engine anchor 0150"). Neither says rename-before-reauthor.
- `before.*rename | rename.*before | rename.*first | rename.*preced` → the **only** ordering hit is D11 line 52: *"runs before masterplan backfill"* — i.e. **sweep-before-backfill**, a different edge entirely.
- `clean base` → **ZERO matches.**

**Reading the two ADRs the plan cites as authority:**

- **D11 (line 52):** authorizes "the FULL sweep (runs before masterplan backfill, each fix read-only-verified): (a) KCMVP/KISA … (b) self-referential renames … (c) dangling edges … (d) foundry bulk-rename." The **(a)(b)(c)(d) are a list of sweep contents under ONE batched door:one-way ruling.** The only sequence D11 rules is *sweep-before-masterplan-backfill*. The plan's line-53 citation **"D11 §c→§d"** reads an ordering arrow into a flat list. **Confirmed: the architect is right — "§c→§d" is not a sequence in the SSOT.**
- **D13 (line 43):** rules the renumber + "the clean ADR-0000+ re-founding series (consolidates-provenance, archive old frozen)." It says **nothing** about whether the foundry rename runs before or after re-foundation. **No ordering edge to L2 exists in D13.** Confirmed.

**Verdict on FLAG-A: TRUE, independently re-derived.** The plan asserts the edge as RULED in three places I confirmed:
- **Line 53:** "The decision record explicitly rules (D11 §c→§d, and the dependency note '…') that the mechanical base must be clean before re-foundation." — cites a non-existent ruling + a fabricated quote.
- **Line 143 (§C):** `L2 foundry-rename ─> L1-refound … [driver D-1, RULED]`.
- **Line 159:** lists "rename-before-reauthor (D1)" among "Hard ordering constraints (RULED, non-negotiable)."

None of the three is in the SSOT. The SSOT rules sweep-before-backfill (D11) and the re-founding itself (D13), with **no** rename-vs-reauthor ordering.

## 2. INDEPENDENT VERIFICATION OF THE ARCHITECT'S "THREE LANES CERTIFIED IT" CLAIM

The architect's sharpest meta-point (§1, §9) is that this false RULED tag *propagated through three lanes uncaught* — the exact verdict-propagation the founder rule forbids. I verified this against the round-1 files:

- **`_critic-round1.md:58`** — enforcement table lists *"verified: D-1 rename-before-reauthor"* as a genuine ruling-violation that fairly excludes Option 2.
- **`_critic-round1.md:91`** — "the L2-before-L1 edge is genuinely right (rename-before-reauthor avoids double-work on immutable docs)."
- **`_arch-round1.md:10`** — "the ordering is already over-determined by three RULED dependencies (D1 rename-before-reauthor, D-2 …, D11 …)."
- **`_arch-round1.md:61`** — "The decision-record backs this" — but the evidence cited is the **disposition table showing foundry amendments exist** (`ledger :45 :46 :47 :55`), which proves *foundry overlap exists*, NOT that an *ordering was ruled*. That is the precise inferential slip: "overlap is real" was laundered into "ordering is RULED."

**Verdict: TRUE.** Both round-1 lanes certified the edge as founder-RULED while citing only evidence that the *overlap* is real. The architect-round2 is the first lane to grep the SSOT for the ruling string and find it absent. This is exactly the founder failure mode (a verdict propagating because each lane trusted the prior "RULED" tag) and it is fair for the architect to escalate it to HIGH.

## 3. IS THE FINDING LOAD-BEARING ENOUGH TO GATE? (my enforcement mandate)

My mandate is to REJECT if principle-option consistency, fair alternatives, or risk-mitigation clarity are missing/weak. FLAG-A hits two of these:

- **Fair-alternatives FAIL at line 53.** The plan's invalidation rationale for Option 2 is **circular**: "Option 2 … contradicts a ruled dependency" — but the "ruled dependency" is the plan's own derived edge. Option 2 (refound-first) is dismissed at line 51 as merely "good if the founder wants to validate the template early," then killed at line 53 by a ruling that does not exist. The *engineering* double-work argument (D-1 as a driver) is real and sufficient to prefer Option 1 — but the plan does not rely on it; it relies on a false RULED label to foreclose a legitimate founder choice. That is an unfair alternative: Option 2 is excluded by fiat, not by merits.
- **Principle-option-consistency FAIL.** §A.1 principle 4 ("an unverified verdict never authorizes a delete/amend") is violated *by the plan's own justification*: the wave architecture authorizes a specific irreversible execution order, and it rests on a verdict (rename-before-reauthor = RULED) that is unverified-against-SSOT and in fact false. The plan applies scrupulous provenance discipline to *tooling* ([ASPIRATIONAL] vs [ENFORCED], even superseding D1 to record `generator=worktree-only`) but applies **none** to the provenance of its own ordering constraints. The architect's Tension A names this precisely and I concur.

This is not pervasive (the rest of the plan's RULED tags — D11 sweep-before-backfill, D13 renumber, D14 drops, the 831/43 census-of-record — I spot-confirmed against SSOT lines 52, 43, 46, 107 and they are genuine). It is **localized to one joint** and **patchable by SY1**. The order does not change. Therefore: **ITERATE, not REJECT.** A REJECT would be disproportionate to a single mislabeled-but-correctable edge in an otherwise strong plan; an APPROVE would let an irreversible execution order ship on a fabricated ruling and foreclose a real founder choice. ITERATE is the calibrated verdict.

## 4. VERIFICATION OF THE ARCHITECT'S SECONDARY FINDINGS

I checked each so the founder is not asked to act on an unverified architect verdict either.

- **FLAG-B / Tension B / SY2 (terminal cross-ref graph never re-verified after Wave-2) — VERIFIED REAL, MEDIUM.** SSOT D-META:26 confirms build-first-cutover-later keeps superseded-on-cutover ADRs live (not archived), so the corpus *grows* through Waves 1–2. The plan's L3.3 (§B L3, line 107) and the L3 gate (§D, line 172) run only in Wave-0 against the frozen snapshot; the G4 DROP gate (line 170) closes inbound-orphan-**on-drop** but not inbound-dangling-**on-new-supersession** (a freshly authored `supersedes:[X]` where X was renumbered/consolidated between MAP-freeze and authoring). No Wave-3 re-resolution exists in §D or §G. The "zero dangling" exit (§G line 214) is therefore proven of the Wave-0 snapshot, not the delivered graph. **The gap is real and the fix is cheap (re-run an existing resolver). Accept SY2.**
- **FLAG-C / SY3 (consolidation-set assumed complete, not proven) — VERIFIED REAL, LOW.** The S1 split's safety claim (L1-amend "touches only files L1-refound does not archive," line 80) holds only if the A.0-2 freeze is *exhaustive*. L1.0-VERIFY (line 82) proves surjection (every archived id → one target) but not *completeness* (every re-foundation-candidate classified). If Wave-1 discovers a missed fold, an L1-amend rename already landed on a now-archived file — the exact waste the freeze exists to prevent. Architect's fix (extend L1.0-VERIFY with a stated closure criterion) is sound. **Accept SY3.** Genuinely LOW (foundry-term set is small/enumerable), correctly rated.
- **FLAG-D / SY4 (831 is partly a sampled estimate; "reconciles to 831" overstates) — VERIFIED REAL, LOW/wording.** The SSOT itself (line 107) flags "journeys/personas residue = sampled estimate," and the plan's A.0-3 concedes 274+135=409≠831. The §D L2 gate (line 171) says "reconciles to the 831 census" while L2.0b is what hardens the tail. "Reconciles to the *post-L2.0b* census" is the honest wording. **Accept SY4.**
- **SY5 (standing exit caveat: amendment-complete ≠ gates-live) — VERIFIED SOUND, tracking.** The plan already labels stand-ins and supersedes D1 to record generator-status; a §G caveat that the "manual stand-in → ported gate" debt stays open is a consistent, cheap honesty upgrade. **Accept SY5.**
- **`cutover_trigger:` / D-META:26 citation — VERIFIED TRUE.** ADR-0510 cutover-trigger + ADR-0250 build-ahead are named at SSOT line 26 exactly as the architect (§5) and the plan (R2) state. R2 is correctly grounded.

**No phantom findings in the architect review.** Every architect claim I checked resolved to a real SSOT line or a real plan line. The architect honored the founder rule.

## 5. WHERE I PUSH BACK ON / SHARPEN THE ARCHITECT

- **The architect's own remedy must not over-correct into removing a real constraint.** SY1 says "move the Option-1-vs-Option-2 wave-shape into the §F founder sign-off set as a real choice." Correct — but the plan must **keep D-2 (map-before-sweep) as genuinely RULED**, because *that* edge IS grounded: D13 (line 43) rules "no-dangling-ref invariant + renumber," and the integrity sweep against a moving id-space is incoherent — the map-before-sweep ordering is forced by the no-dangling invariant, not derived. So §A.4's sign-off text must read, per the architect's own §4 closing: "wave-serialization buys **D-2 (RULED: no-dangling-ref ⇒ map-before-sweep)** and the **DERIVED D-1 (rename-before-reauthor, founder-ratifiable)**." Do not let the FLAG-A correction collateralize D-2 into "also just derived." D-2 stays RULED; only D-1 drops to DERIVED. (The architect implies this at §4 close but the plan must state it explicitly so the relabel is surgical.)
- **The provenance pass (§7 hole 1 / SY1) is the single most important iteration item** and I add a sharpening: it must check **every** `[RULED]`/`RULED`/"the decision record rules" assertion in the plan against a citable SSOT line, not just the one edge the architect caught. The architect found D-1; the discipline that found it (grep the SSOT for the ruling string) must be run corpus-wide on the plan's own tags, because the same inferential slip ("overlap is real ⇒ ordering is ruled") could recur elsewhere. I spot-checked the headline ones (D11/D13/D14/D-EVENT/census) and they hold, but a one-time exhaustive pass is owed before the founder signs.
- **On REJECT-vs-ITERATE calibration:** I considered REJECT because a *fabricated quotation* (line 53's quoted non-existent "dependency note") is a serious integrity failure in a document headed for door:one-way founder sign-off — it is the kind of thing that, uncaught, becomes canon. But (a) the underlying order is engineering-correct, (b) the fix is a relabel + one deletion + moving one item into the sign-off set + a provenance pass, and (c) the plan's verification architecture is otherwise strong and honest. A fabricated-quote defect that is localized, identified, and cheaply correctable is ITERATE territory. **It would become REJECT if a re-review found the provenance pass surfaced additional false-RULED tags, or if the line-53 fabricated quote were defended rather than deleted.**

## 6. ENFORCEMENT-CHECKLIST SCORING (my mandate)

| Bar | Status | Note |
|---|---|---|
| Principle-option consistency | **WEAK (one joint)** | §A.1 principle-4 violated by the plan's own RULED-D-1 justification; patch = SY1. Rest consistent. |
| Fair alternatives | **WEAK (one joint)** | Option-2 invalidation (line 53) is circular — killed by a non-existent ruling, not on merits. The engineering case is sufficient; the false label is not. Patch = SY1 (move to founder choice). |
| Risk-mitigation clarity | **STRONG** | Pre-mortem Failures 0–3 + 2b/2c are concrete and map to gates; mitigations cite the specific gate that closes each. |
| Testable acceptance criteria | **STRONG (4 small wording/coverage holes)** | §H.2 is greppable; FLAG-B adds a missing terminal test, FLAG-D a wording fix. The R4 published-falsification-log exit (line 217) correctly de-fangs the round-1 unfalsifiable-exit hole. |
| Concrete verification steps | **STRONG** | §D per-lane gates + separate-verifier-lane discipline are real; G3 MAP-self-consistency + G4 DROP-inbound close the round-1 holes. The one missing step is the **ruling-provenance pass** (verify the plan's OWN tags) — owed. |
| Deliberate-mode: real pre-mortem + expanded verification | **PRESENT & STRONG** | §H.1 Failures 0–3/2b/2c + §H.2 per-lane evidence table clear the deliberate bar cleanly. |

Two bars land WEAK (principle-option consistency, fair alternatives), both at the *same* FLAG-A joint, both closed by SY1. Per my mandate ("REJECT if missing/weak"), "weak" here is localized and patchable, not pervasive or absent — the calibrated outcome is ITERATE.

## 7. REQUIRED ITERATION ITEMS (gating before founder sign-off)

1. **[HIGH — SY1, gating, before sign-off]** Relabel "rename-before-reauthor" from `[RULED]` to **`[DERIVED: D-1 engineering double-work, founder-ratifiable]`** at lines 143 and 159. **Delete the fabricated-quotation sentence at line 53** ("The decision record explicitly rules (D11 §c→§d, and the dependency note '…')") — it quotes a ruling that does not exist in the SSOT. Keep the engineering double-work argument (it is valid and sufficient). **Move the Option-1-vs-Option-2 wave-shape into the §F founder sign-off set as a genuine choice.** Keep Option 1 as the recommended call with its engineering rationale intact. This changes *what the founder signs*, so it must land before sign-off.
2. **[HIGH — provenance pass, gating]** Run a one-time ruling-provenance verifier pass: for **every** `RULED`/"the decision record rules" tag in the plan, confirm a citable SSOT line exists; downgrade any that don't to `[DERIVED]`. Keep D-2 (map-before-sweep) and D11 (sweep-before-backfill) as genuinely RULED — both grounded (D13 no-dangling invariant; D11:52). Only D-1 drops.
3. **[MEDIUM — SY2, before Wave-3]** Add a Wave-3 terminal whole-corpus cross-ref re-resolution gate (re-run the L3.3 resolver after the last Wave-2 supersession), wired into §D Wave-3 + §G exit, so "zero dangling" is true of the *delivered* graph.
4. **[LOW — SY3, before the freeze is treated as load-bearing]** Extend L1.0-VERIFY (G3) to prove consolidation *completeness against a stated closure criterion*, not just surjection.
5. **[LOW/wording — SY4]** §D/§H.2 L2 exit: "reconciles to the *post-L2.0b* census," not "to the 831 census."
6. **[tracking — SY5]** §G standing caveat: amendment-complete ≠ gates-live; the manual-stand-in→ported-gate debt stays open and tracked in the masterplan-wiring meta-ADR.

## 8. SYNTHESIS — what is settled and what remains

Settled (verified TRUE this round, no further iteration owed): the round-2 G1–G5 / R1–R6 / S1–S6 absorptions are real and correctly wired (census 831/43 at SSOT:107; consolidation-set freeze; G3/G4 gates; D-LANES narrowing named + S1 split; safety `runtime-hook-pending`; `cutover_trigger:` at SSOT:26; R4 published-falsification-log exit). The wave *order* is engineering-correct and the L2-before-L1-refound edge serves L1 rather than fighting it (the consolidation-set carve closes the round-1 worry). The architect review is evidence-grounded with **no phantom findings**.

Remaining (this round's iteration): the plan's load-bearing ordering edge is **DERIVED masquerading as RULED**, with a **fabricated SSOT quotation** at line 53, foreclosing a genuine founder choice via a circular invalidation. This is a principle-option-consistency + fair-alternatives defect at the one joint where who-decided-it is load-bearing. It is localized, identified, and cheaply correctable (SY1 + a provenance pass), plus two small verification gaps (SY2 terminal re-resolution, SY3 completeness) and two wording/tracking upgrades (SY4/SY5). The order does not change; its *honesty* does. With SY1 + the provenance pass applied, the founder signs an honest plan.

**The boulder does not stop here:** the plan must absorb the round-2 conditions (SY1 + provenance pass gating before sign-off; SY2–SY3 before their load-bearing points; SY4–SY5 as wording/tracking) and return for a round-3 confirmation that the fabricated quote is deleted, the relabel is surgical (D-1→DERIVED, D-2 stays RULED), and the provenance pass surfaced no further false-RULED tags.

---

VERDICT: ITERATE
