# Architect Review — AMENDMENT-PLAN round 3

**Reviewer lane:** architect (separate from authoring/planner lane).
**Verdict:** **APPROVE-WITH-CONDITIONS.** The wave order is sound; round-2's provenance correction (D-1 → DERIVED) is verified-correct and surgical. But round-3 surfaces **two new findings the prior rounds normalized away**: a load-bearing *knowability paradox* in the consolidation-set freeze (HIGH), and a *mis-classified provenance in the opposite direction* — D11(d) makes L2's sweep-membership RULED, which the plan never states and which slightly reshapes the §A.6 pass mandate (MEDIUM). Plus one genuine tradeoff tension the plan resolves by assertion, and three principle-flags.

**Date:** 2026-06-06. **Round:** 3.

---

## 0. Independent re-verification (founder rule: trust no prior verdict)

I re-ran the round-2 provenance checks against the SSOT myself, not from the plan's self-report:

| Claim | My grep/read result | Verdict |
|---|---|---|
| `grep double-work` over SSOT | **ZERO** | round-2 correct; D-1 is not SSOT-ruled |
| D11 `:52` rules sweep-before-backfill only | confirmed — "(runs before masterplan backfill …)", a batched one-way door over (a)(b)(c)(d), no internal sequence | correct |
| D13 `:43` no-id-reuse + no-dangling invariant | confirmed verbatim | map-before-sweep RULED is grounded |
| census 831/43 at `:107` | confirmed verbatim | correct |
| `grep re-author` → only `:4` boilerplate + `:34` Cedar bullet | confirmed; neither is an ordering | D-1 correctly DERIVED |

**The round-2 relabel is verified surgical and honest. The fabricated quotation is gone. I find no *further* false-RULED tag in the D-1 family.** Round-3's open re-review trigger (I.2: "confirm the fabricated quote is deleted, relabel is surgical, no further false-RULED tags") is **satisfied for the direction round-2 looked.** My new findings are in directions round-2 did *not* look.

---

## 1. STRONGEST STEELMAN ANTITHESIS — "the plan polices RULED-inflation but is blind to RULED-*deflation*, and that blindness sits on the irreversible lane"

Round-2's entire correction machinery (§A.5/§A.6 provenance pass) is built to catch one error class: **a DERIVED edge wearing a RULED tag** (false-positive RULED). It caught D-1 and hardened against recurrence. But the §A.6 mandate as written — *"for every RULED tag, confirm a citable SSOT line exists; downgrade if none"* — is a **one-directional filter.** It can only ever *remove* RULED tags. It structurally cannot detect the opposite error: **a genuinely-RULED constraint the plan labeled DERIVED or left untagged** (false-negative RULED). And I found one on the load-bearing lane.

**The concrete instance — L2's sweep-membership is RULED, and the plan treats the whole L2-vs-L1 relationship as DERIVED.** D11 (`:52`) authorizes "the FULL sweep" as a **batched one-way door**, and its content (d) is *"foundry→cloud-intelligence/governance bulk-rename."* The foundry rename **is a named content of the D11 sweep.** Therefore:

- **"L2 runs as part of the Wave-0 sweep, before masterplan backfill" is RULED** (D11 `:52`, the same line that grounds sweep-before-backfill). The plan only ever cites D11 `:52` for L3 (integrity sweep). It silently amputates L2 from its own ruling and re-derives L2's Wave-0 placement through the *weaker* D-1 double-work argument.
- This matters because the plan's headline framing — *"L2-before-L1 is DERIVED, founder-ratifiable, the founder may overrule and run Option 2"* — is **only half true.** The founder may overrule *rename-before-**reauthor*** (the sequencing of L2 relative to L1's re-authoring — genuinely DERIVED). The founder may **not** coherently overrule *rename-as-part-of-the-sweep-before-backfill* (RULED by D11(d)+`:52`). Option 2 ("refound-first, sweep second") as the plan describes it would run the foundry bulk-rename **after** the re-foundation — but D11 puts the bulk-rename *in the sweep that runs before backfill*, and re-foundation feeds backfill. **Option 2 as literally worded mildly violates D11's batched-sweep-before-backfill door.** The plan presents Option 2 as a clean founder choice; it is actually a choice with a RULED constraint the plan didn't surface, because it lost track of L2's membership in D11.

So the antithesis is: **the plan over-corrected toward "everything is DERIVED, let the founder choose" and in doing so detached a lane (L2) from its actual ruling (D11(d)).** Round-2 fixed RULED-inflation and introduced a subtler RULED-deflation. The founder is now at risk of being told "you may freely pick Option 2" when Option 2 partially collides with D11 — the mirror-image of the round-2 defect (being told "you must pick Option 1" when no ruling forced it). Same root cause both rounds: **the planner reasons about L2's *order* from the double-work driver and never re-reads D11(d) to see L2 is already *inside* a ruled batch.**

**Why this is the strongest form:** it is not a wording nit. It uses the plan's own verified method (grep the SSOT, don't infer from overlap) and the plan's own highest-priority machinery (provenance discipline), and shows that machinery is **directionally incomplete on the one lane (L2) that is irreversible and 831 files wide.** It also self-certifies: the §A.6 pass, run as written, would *pass this plan* while the defect stands — because the pass only audits existing RULED tags, never asks "is anything that should be RULED missing its tag?"

---

## 2. SECOND FINDING (HIGH) — the consolidation-set freeze has a knowability paradox the plan asserts away

§A.0-2 / L1.0-CONS is the keystone of the whole Wave-0 parallelization: it freezes *"which old Accepted ADRs fold into which ADR-0000+ doc (and are therefore archived-by-re-foundation)"* as a door:one-way primary source **at the very start of Wave-0**, so L2.2/L3.3/L1-amend can skip throwaway files. The S1 split (L1-amend ∥ Wave-0) is *entirely* load-bearing on this freeze being knowable up front.

**The paradox:** the consolidation set is the output of *deciding how to re-found*, but the freeze must happen *before* re-foundation authoring (which is Wave-1). You cannot know "0124 folds into the new oya-ci ADR, 0511 is consolidated, 0369/0367/0366 are phased-in" (D3, `:114-118`) **until you have designed the oya-ci re-foundation ADR** — that design *is* what determines the fold set. Same for D5 (which of 0187/0421/0476 archive), D6 (the phantom-0150 re-author), D7 (which isolation ADRs fold). The decision-record gives the *clusters* but not a file-level fold map; that map is produced *by* L1.1 authoring, which is Wave-1.

So L1.0-CONS in Wave-0 must either:
- **(a)** front-load the re-foundation *design decisions* (not the authoring) into Wave-0 — i.e. a real chunk of L1-refound's intellectual work moves to Wave-0 under a different name, which the plan's wave-boundary hides; or
- **(b)** freeze a *provisional* consolidation set that L1.1 may contradict, in which case L1.0-VERIFY's "consolidation COMPLETENESS, S1 partition provably total" (SY3) is proving completeness of a guess, and a Wave-1 re-foundation discovering a new fold retroactively invalidates a Wave-0 L1-amend rename — **the exact wasted-work-on-archived-file defect A.0-2 exists to prevent.**

The plan's L1.0-VERIFY (SY3) *names* this risk ("without it, a Wave-1 re-foundation could discover a missed fold and an L1-amend rename would already have landed") and claims the completeness check closes it. **It does not.** A completeness check proves *every candidate is classified*; it cannot prove *the classification won't change when the re-foundation is actually authored in Wave-1.* The closure criterion ("every Accepted ADR carrying a foundry-sense term that is also a re-foundation candidate is classified") is checkable only if "re-foundation candidate" is already pinned — and pinning it is the Wave-1 work. **The freeze is door:one-way; if it's provisional, a one-way door is being signed over a guess.** This is a genuine ordering circularity, not a verification gap, and it is the first thing every Wave-0 lane keys off.

---

## 3. TRADEOFF TENSION (real, the plan resolves by assertion) — D-LANES maximal-parallelism ⊥ build-first growing-graph cleanliness, AND the S1 "recovery" partly re-creates the very serialization it claims to recover

The plan (§A.4) honestly names one tension: serial waves narrow the D-LANES one-way organizing principle (`:84`). It claims S1 (L1-amend ∥ Wave-0) "recovers most of the lost cross-wave parallelism" so "the narrowing is minimal."

**The tension is deeper than the plan admits, and it compounds with Finding 2.** S1's recovery is *conditional on the consolidation-set freeze being a real, non-provisional partition* (§2). If the freeze is provisional (which §2 argues it must be), then L1-amend cannot safely run in Wave-0 — because any file it amends in place could turn out to be a Wave-1 fold, and the amend is then wasted *and* dirties the immutability diff. **The plan's own parallelism recovery (S1) and its own correctness gate (A.0-2 freeze) are in tension:** S1 needs the freeze to be final to be safe; the ordering (re-foundation is Wave-1) makes the freeze provisional. You can have S1-parallelism *or* a provably-final freeze, not both, under the stated wave order. The plan asserts it has both.

**The deepest version (ties to SY2/Finding-from-round-2):** build-first-cutover-later (`:26`) *requires* the graph to grow through Waves 1–2 (superseded-on-cutover ADRs stay live). D-LANES *wants* maximal parallel authoring. No-dangling-ref (D13 `:43`) wants a stable, complete graph to verify against. These three RULED principles form a genuine trilemma: **you cannot simultaneously (i) author supersessions in parallel across waves, (ii) keep superseded bridges live so the graph grows, and (iii) prove zero-dangling against a frozen snapshot.** Round-2's SY2 (terminal re-resolution on the delivered graph) is the *correct* resolution of (iii)-vs-(ii) — but it implicitly **re-serializes**: the terminal sweep cannot run until the *last* Wave-2 supersession lands, so Wave-3 is hard-blocked on all of Waves 1–2 completing. That is a serialization the §A.4 "minimal narrowing" framing doesn't account for. The narrowing is *not* minimal; it has a mandatory whole-corpus barrier at the end. The plan should *name* this barrier as a second acknowledged serialization, not fold it silently into a "cheap re-run."

---

## 4. PRINCIPLE-VIOLATION FLAGS

- **PV-1 (verify-at-each-step, directional gap) — HIGH.** §A.6's ruling-provenance pass is mandated one-directional (audit RULED → downgrade). It cannot catch RULED-deflation (Finding 1). The founder rule "trust no prior verdict, re-read primary sources" is satisfied *for the tags that exist* but not *for the tags that should exist*. **A verification pass that can only ever remove constraints, never add a missing one, is not a complete verification of an ordering — it is a complete verification of the *non-inflation* of an ordering.** The plan presents §A.6 as proving "the plan's orderings are honest"; it actually proves "the plan's *asserted* orderings are not over-claimed." Those are different claims.
- **PV-2 (door:one-way over an unfinalized artifact) — HIGH.** A.0-2 consolidation-set freeze is door:one-way *and* (per §2) provisional-until-Wave-1. Signing a one-way door over an artifact that downstream authoring may revise violates the spirit of door:one-way (irreversible commitment requires a finalized object). Either the freeze is genuinely final (then the re-foundation design must be in Wave-0, and the wave map is mislabeled) or it isn't (then it shouldn't be door:one-way yet).
- **PV-3 (worth-documenting ⇒ reachable, latent) — LOW/MEDIUM.** The "manual stand-in → ported gate" debt (SY5 standing caveat) is tracked in the masterplan-wiring meta-ADR. Good. But the debt's *liveness* (is the generator on dev yet?) is a runtime fact that the canon can only record as-of-a-date (the D1 supersede records `worktree-only @ 2026-06-06`). A future reader sees a stale date, not a live status. The reachability rule is satisfied for the *fact*; it is not satisfied for the *current truth*. Minor, but the plan claims the canon carries the status — it carries a dated snapshot of the status.

---

## 5. SYNTHESIS — improvements that strengthen the plan without changing the (correct) wave order

1. **[Finding 1] Make §A.6 bidirectional, and re-tag L2.** Add a second mandate to the §A.6 ruling-provenance pass: *"for every D-decision in the SSOT that names a concrete amendment action (D11(a)-(d), D12, D13, D14, D-EVENT, …), confirm the plan's corresponding lane carries the matching RULED tag; surface any ruled action the plan left DERIVED/untagged."* Then re-tag: **L2's membership in the Wave-0 sweep is `[RULED: D11(d)+:52, foundry bulk-rename is a named sweep content, batched one-way before backfill]`.** Keep `rename-before-reauthor` (L2-vs-L1.1 *internal sequencing*) as `[DERIVED: D-1]`. The distinction the founder actually needs: *the rename happens in the pre-backfill sweep (RULED); whether it happens before or interleaved-with the re-authoring of the same files is the DERIVED choice.* This **narrows Option 2**: refound-first is only legal if the foundry rename still lands inside the pre-backfill sweep — i.e. Option 2 cannot mean "rename after re-foundation," only "author the ADR-0000+ shells first, then run the (still-pre-backfill) sweep over them." Surface *that* as the real Option-2, so the founder choice is honest in both directions.

2. **[Finding 2 / PV-2] Split L1.0-CONS into a Wave-0 *design-freeze* and a Wave-1 *confirm-or-amend* checkpoint.** Rename L1.0-CONS to make explicit that what lands in Wave-0 is the **re-foundation cluster-design decision** (the D3/D5/D6/D7 fold *intent*, which the SSOT already rules), not a guaranteed-final file map. Add a **Wave-1 L1.1-CONS-CONFIRM** unit: when each re-foundation ADR is authored, the verifier confirms the actual fold set equals the Wave-0 frozen set; **any delta is a re-review trigger (re-open the door:one-way), not a silent amend** — and L1-amend renames on any newly-discovered fold are reverted before the file is archived. This makes the provisional-vs-final honest, keeps door:one-way meaningful (you sign the *design*; the *file map* is confirmed not assumed), and gives S1 a real safety net instead of an asserted one. The completeness check (SY3) then proves completeness *of the design-level classification* (legitimately checkable in Wave-0), and the Wave-1 confirm proves *file-level stability* (checkable only when authored). Two checks for two genuinely different claims.

3. **[Finding 3 / Tension] Name the terminal barrier as a second acknowledged serialization.** Amend §A.4 to read: *"the wave-serialization narrows D-LANES at two points — the Wave-0 gate (buys map-before-sweep) and the Wave-3 terminal cross-ref barrier (buys delivered-graph no-dangling under build-first growing-graph). Both are RULED-driven (D13 `:43`), both are accepted, and S1 recovers within-Wave-0 parallelism but not the terminal barrier."* This makes the "minimal narrowing" claim accurate rather than optimistic, and pre-empts a future reviewer re-discovering the barrier as a surprise.

4. **[PV-3] Make the aspirational-status queryable, not just dated.** Replace the static `generator = worktree-only @ 2026-06-06` supersede with a `cutover_status:` front-matter field on the masterplan-wiring meta-ADR (mirroring the `cutover_trigger:` pattern the plan already mandates for build-first), so the *current* port-status is a queryable field updated when the generator lands on dev — not a prose date a reader must distrust. Closes the "canon carries a stale snapshot" gap with the plan's own already-accepted mechanism.

---

## 6. What I explicitly AGREE is correct (do not re-litigate)

- **The wave order (Option 1).** Verified-correct on D-2 (RULED, D13 `:43`) + D-1 (DERIVED but sound). My findings reshape *provenance* and *Option-2's legality*, not the recommended sequence.
- **Round-2's D-1 relabel.** Surgical, honest, verified. The fabricated quote is gone.
- **Build-first-cutover-later sequencing** with `cutover_trigger:` required front-matter (closes build-first-cutover-*never*). Correct and well-grounded in `:26`.
- **Aspirational-vs-enforced discipline** (generator/cohesion/drift/safety-hook flagged, manual stand-ins, adversarial cohesion lane, runtime-hook-pending on the safety ADR). The R3 adversarial-falsification cohesion stand-in is the right manual approximation of a by-construction gate; the safety `runtime-hook-pending` flag is the correct treatment of the highest-consequence aspirational gap.
- **Verify-at-each-step adequacy, with the PV-1 caveat.** Per-lane verifier gates are evidence-based, separate-lane, no-self-approval. The one structural gap is directional (RULED-deflation), addressed by synthesis #1.

---

## 7. Disposition

**APPROVE-WITH-CONDITIONS.** Conditions, in priority order:
1. **(HIGH, before §A.6 pass is declared complete)** Make §A.6 bidirectional and re-tag L2's sweep-membership as RULED (D11(d)); re-state Option-2's actual legal shape (synthesis #1). A §A.6 pass that surfaces this missing RULED tag is a re-review trigger, consistent with the plan's own round-2 rule.
2. **(HIGH, before A.0-2 is signed as door:one-way)** Resolve the consolidation-set knowability paradox via the design-freeze / Wave-1-confirm split (synthesis #2 / PV-2).
3. **(MEDIUM)** Name the Wave-3 terminal barrier as a second acknowledged D-LANES serialization (synthesis #3).
4. **(LOW)** `cutover_status:` field for aspirational-tooling liveness (synthesis #4 / PV-3).

None of these change the wave order. Two of them (#1, #2) are the *same defect class round-2 fixed, found in the opposite direction and a second location* — which is itself evidence that the directional/one-artifact blind spot is real and should be closed structurally (bidirectional provenance pass) rather than patched per-instance.
