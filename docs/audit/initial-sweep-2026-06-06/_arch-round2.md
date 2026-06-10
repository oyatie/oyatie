# ARCHITECT REVIEW — AMENDMENT-PLAN round 2

> Reviewer: architect lane (separate from the authoring lane and from the round-1 architect/critic lanes). Verdict basis: re-read the full round-2 AMENDMENT-PLAN, re-read the SSOT (`synthesis/decision-record-oyatie-canon.md`) line-by-line, re-read `_arch-round1.md` + `_critic-round1.md`, and independently re-verified every load-bearing ordering claim against the SSOT this round. Founder rule honored: no phantom findings — every claim cites a file+line I read this round, and I re-derived the dependency edges from the SSOT rather than trusting the plan's self-report of what the SSOT rules.
> **Overall: APPROVE-WITH-CONDITIONS (narrowed) — but with ONE new HIGH finding the round-1 lanes missed.** The round-2 revision correctly absorbs G1–G5 / R1–R6 / S1–S6; the four headline facts are settled and the gates are well-built. But the plan's central ordering justification rests on a **RULED dependency that is not actually in the SSOT** (the "D-1 rename-before-reauthor" edge is reviewer-derived, not founder-ruled), and the plan now states it is "RULED" in three places. That is a provenance defect at the exact load-bearing joint, and it changes what must be signed. Details below.

---

## 0. WHAT ROUND-2 GOT RIGHT (verified, not assumed)

I re-verified, against primary sources, that the round-2 additions are real and correctly wired — not cosmetic:

- **G1 census reconciliation is genuinely needed and now gated.** Confirmed against the register: `docs-sweep/00-REST-OF-DOCS-REGISTER.md:108` ("731 files (ADR-excluded)"), `:113` ("Palantir Foundry (105 journey files …)"), `:146` (table row "foundry | 731"); SSOT `decision-record:107` ("total non-ADR = 831 (not 731), Palantir-Foundry carve-out = 43 files (not 105)"). The contradiction is real, in a cited primary source, and §A.0-1 + the §D A.0 gate now block Wave-0 on its correction. **Correctly resolved.**
- **G2 consolidation-set freeze** is lifted to its own door:one-way primary source at the very start of Wave-0 with the edge `consolidation-set-freeze → {L2.2, L3.3, L1-amend}` (§A.0-2, L1.0-CONS, §C). This was the sharpest real order-defect in round-1 and it is now fixed at the right altitude.
- **G3 MAP self-consistency verifier** (L1.0-VERIFY, bijection/surjection/no-reuse ahead of founder sign-off), **G4 DROP inbound-reference-safety gate** (L1.2-DROP, incl. Train), **G5 D-LANES-narrowing named + S1 L1-amend∥Wave-0 split** — all present, all wired into §C edges, §D gates, §F sign-off, §G exit. The S1 split is the right move: it recovers most of the parallelism the wave-serialization costs.
- **R1 safety `runtime-hook-pending`, R2 `cutover_trigger:`, R3 adversarial cohesion stand-in, R4 published falsification-log exit evidence, R5 D-EVENT precedence (0005 amended not co-equal), R6 L2.0b template-first census** — all folded. The §I disposition table is an accurate map of round-1→round-2; I spot-checked five rows against their cited sections and they land where claimed.

The round-2 plan is materially stronger than round-1. The conditions below are narrower than round-1's — but one is new and HIGH.

---

## 1. STRONGEST STEELMAN ANTITHESIS (round-2) — "the plan's load-bearing ordering edge is reviewer-canon masquerading as founder-canon, and the plan now over-asserts it as RULED"

The plan's entire wave architecture hangs on one edge: **L2 foundry-rename must precede L1-refound (rename-before-reauthor), to avoid double-work on immutable docs (driver D-1).** The plan does not merely use this edge — it elevates it to founder-ruled law and cites the SSOT as authority:

- Line 53: *"The decision record explicitly rules (D11 §c→§d, and the dependency note 'L2 foundry-rename must precede ADR-0000+ re-author to avoid double-work') that the mechanical base must be clean before re-foundation."*
- Line 143 (§C): `L2 foundry-rename ─> L1-refound ADR-0000+ re-author … [driver D-1, RULED]`.
- Line 159: lists "rename-before-reauthor (D1)" among the "Hard ordering constraints (RULED, non-negotiable)."

**I went to the SSOT to verify that ruling. It is not there.** Re-reading `decision-record-oyatie-canon.md` end-to-end this round:

- **D11 (`:51-52`)** authorizes "the FULL sweep (runs before masterplan backfill, each fix read-only-verified)" with sub-items (a) KCMVP/KISA, (b) self-referential renames, (c) dangling edges, (d) foundry bulk-rename. The **only** ordering D11 rules is *sweep-before-masterplan-backfill*. The (a)(b)(c)(d) are a **list of sweep contents, not a sequence** — "(c)→(d)" is not an ordering arrow, it is two bullets in one batched door:one-way ruling. The plan's "D11 §c→§d" citation reads a sequence into a list.
- **D13 (`:42-43`)** rules the renumber + the "clean ADR-0000+ re-founding series (consolidates-provenance, archive old frozen)." It says **nothing** about whether the foundry rename runs before or after the re-foundation. No ordering edge to D11/L2 exists in D13.
- A direct grep for the claimed "dependency note" — `re-author`, `reauthor`, `before.*rename`, `rename.*first`, `clean base`, `double-work` — over the SSOT returns **zero** matches outside the boilerplate "supersede/re-author into the clean series" status line (`:4`) and the Cedar re-author bullet (`:34`). **There is no "L2 foundry-rename must precede ADR-0000+ re-author" dependency note in the SSOT.** It is a round-0 *planner inference* that the plan has, across two revisions, hardened into a quoted founder ruling.

Why this is the strongest argument the plan is risky — and why neither round-1 lane caught it: **both round-1 reviewers explicitly endorsed this edge as "RULED" and "genuinely right" without checking its provenance in the SSOT.** `_arch-round1.md:61` ("Confirmed correct … The decision-record backs this") and `_critic-round1.md:31,58,91` ("the L2-before-L1 edge is genuinely right … verified: D-1 rename-before-reauthor"). The critic's §4 enforcement table (`:58`) even lists "D-1 rename-before-reauthor" as a *verified ruling-violation* that invalidates Option 2. **Three lanes have now certified as founder-ruled an edge the founder never ruled.** That is precisely the failure mode the founder rule exists to prevent: a verdict propagating because each lane trusted the prior lane's "RULED" tag instead of re-reading the primary source. The plan's own §A.1 principle 4 ("an unverified verdict never authorizes a delete/amend") is being violated *by the plan's own justification*: the wave architecture — which authorizes a specific irreversible execution order — rests on an unverified-against-SSOT verdict.

**The sharp consequence:** the rename-before-reauthor edge is, on the engineering merits, *a good idea* (the double-work argument is sound — see §4). But "good idea the architect derived" and "RULED door:one-way founder constraint" are different things with different sign-off requirements. By labeling it RULED, the plan removes it from the set of decisions the founder is asked to actually decide, and smuggles a reviewer's sequencing preference into the non-negotiable layer. If the founder would actually prefer Option-2 shape (refound-first, accept the double-work to see ADR-0000+ canon sooner — a legitimate preference the plan dismisses at line 51 as merely "good if the founder wants to validate the template early"), the plan has foreclosed that choice by mislabeling its own preference as the founder's law. **The plan is not wrong about the order; it is wrong about who decided the order, at the one joint where that distinction is load-bearing.**

Corollary (the same defect, smaller): line 53's "Option 2 … contradicts a ruled dependency" invalidation rationale is therefore **circular** — Option 2 is rejected for violating a "ruling" that is actually the plan's own derived edge. Option 2's real disqualifier is the *engineering* double-work argument (D-1 as a driver), which is sufficient on its own. The plan does not need the false RULED label; it weakens itself by claiming one.

---

## 2. TRADEOFF TENSIONS

### TENSION A (primary, NEW) — "RULED" provenance vs the plan's own honest aspirational-labeling discipline

The plan is scrupulous about one provenance distinction — [ASPIRATIONAL] vs [ENFORCED] for tooling — and built an entire R1/FLAG-3 apparatus (supersede D1 to record `generator=worktree-only@2026-06-06`) so that "what is real vs what is aspirational" lives in the canon, not just the plan. **It applies no such discipline to the provenance of its own ordering constraints.** There are now two classes of "RULED" in the plan: (i) genuinely founder-ruled door:one-way edges (sweep-before-backfill D11, renumber D13, the census-of-record D-correction), and (ii) reviewer-derived engineering edges relabeled RULED (rename-before-reauthor "D-1"). The tension: the plan's credibility rests on its provenance honesty (it is the plan's best quality), and it spends that credibility on tooling-status while quietly debasing it on ordering-status. A founder who trusts the plan's [ASPIRATIONAL] tags has no signal that "RULED" carries a *weaker* guarantee than [ENFORCED] does. **Resolution direction:** the plan should distinguish **`[RULED]`** (cite SSOT line) from **`[DERIVED]`** (engineering driver, not founder-ruled, founder may overrule) on every ordering constraint, exactly as it distinguishes ASPIRATIONAL from ENFORCED on every tool.

### TENSION B (real, carried from round-1, sharpened) — build-first-cutover-later makes the cross-ref graph a growing target the frozen MAP does not stabilize

`cutover_trigger:` (R2) makes the pending-cutover set *queryable*, which is good, but it does not make the corpus *smaller*. Every Wave-1/Wave-2 supersession adds a live `ADR-0000+` doc while the old frozen ADR stays live (build-first: don't archive the bridge). The L1.0 MAP freeze (D-2) stabilizes *renumber* edges, but **supersession edges are authored after the freeze**, so the "zero dangling edges" invariant the L3 verifier must hold (§D L3 gate) is being checked against a graph that keeps growing through Waves 1–2. The plan's L3.3 runs in Wave-0 against the frozen id-space; nothing re-runs the whole-corpus cross-ref resolver *after* Wave-2 supersessions land. The G4 DROP gate closes the *inbound-orphan-on-drop* case, but not the *inbound-dangling-on-new-supersession* case (a freshly authored `supersedes:[X]` whose X was itself renumbered/consolidated between MAP-freeze and authoring). **This is a genuine tension, not a defect** — you cannot both honor build-first (corpus grows) and have a single Wave-0 integrity sweep prove the final graph clean. **Resolution direction:** add a terminal (Wave-3) whole-corpus cross-ref re-resolution gate — the L3.3 resolver re-run *after* the last supersession lands, not only in Wave-0 — so the "zero dangling" claim is true of the *delivered* graph, not just the Wave-0 snapshot. (This is cheap: it is re-running an existing gate, not a new mechanism.)

### TENSION C (carried, adequately mitigated) — verifier-as-cohesion-judge vs verifier-independence

Round-2's R3 (adversarial second lane, falsification mandate) + R4 (published falsification log) is the correct mitigation and I endorse it as sufficient *for the cohesion gate*. The residual tension is narrower: the adversarial lane is still a *manual stand-in* for an [ASPIRATIONAL] gate, and a manual falsification search over a large corpus has no completeness guarantee the way a by-construction enum-keyed gate would. The plan is honest about this (it labels the stand-in and requires the log). I flag it not as a defect but as the irreducible residue of running a not-yet-ported gate manually — see §6.

---

## 3. PRINCIPLE-VIOLATION FLAGS

- **[FLAG-A — HIGH, provenance — NEW] The plan asserts "rename-before-reauthor (D1/D-1)" as RULED in three places (lines 53, 143, 159); the SSOT rules no such ordering.** D11 rules only sweep-before-backfill; D13 rules the re-founding with no ordering vs the rename; the quoted "dependency note" does not exist in the SSOT (verified by grep this round). This violates the founder rule (verify-at-each-step against primary sources; an unverified verdict never authorizes an amend) at the load-bearing joint, and it has propagated through three lanes uncaught. **Fix:** relabel the edge `[DERIVED — engineering driver D-1, NOT founder-ruled; founder may overrule]`; delete the "decision record explicitly rules … dependency note" sentence at line 53 (it cites a non-existent ruling); keep the *engineering* double-work argument (which is valid and sufficient); and **add the rename-vs-reauthor order to the §F founder sign-off set as a genuine decision** (Option-1 wave-shape vs Option-2 refound-first), since it is in fact an open founder choice, not a closed ruling. This does not change the recommended order — Option 1 is still the right engineering call — it changes its *status* from "non-negotiable RULED" to "founder-ratifiable DERIVED," which is what it actually is.

- **[FLAG-B — MEDIUM, completeness] The terminal cross-ref graph is never re-verified after Wave-2 supersessions (Tension B).** The "zero dangling supersedes/amends" invariant (§D L3 gate, §G exit) is proven against the Wave-0 frozen snapshot, but supersession edges authored in Waves 1–2 are outside that snapshot. **Fix:** add a Wave-3 whole-corpus cross-ref re-resolution gate (re-run the L3.3 resolver post-last-supersession) to §D Wave-3 gate + §G exit. Cheap; closes the only integrity hole the MAP-freeze leaves open.

- **[FLAG-C — LOW, self-consistency of the consolidation set vs L1-amend partition] The S1 partition depends on the A.0-2 freeze being not just frozen but COMPLETE.** L1-amend's safety claim ("touches only files L1-refound does not archive") is sound *only if* the consolidation set is exhaustive at freeze time. If a later Wave-1 re-foundation discovers another old ADR that should fold in (a consolidation the freeze missed), then an L1-amend rename already landed on a file that L1-refound now archives — the exact wasted-work + dirtied-immutability-diff the freeze exists to prevent, re-introduced. The plan treats the freeze as complete-by-fiat. **Fix:** the L1.0-VERIFY gate (G3) already proves consolidation *surjection* (every archived id → one target); extend it to prove consolidation *completeness against a stated closure criterion* (e.g., "every Accepted ADR carrying a foundry-sense term that is also a re-foundation candidate is classified consolidate-or-amend, none unclassified") so the partition L1-amend relies on is provably total, not assumed total. Low severity because the foundry-term set is small and enumerable; flagged because the S1 split's correctness is silently load-bearing on it.

- **[FLAG-D — LOW, naming] "831 census-of-record" is itself still partly an estimate (A.0-3 concedes 274+135=409≠831, remainder = carve-outs + FP + sampled journeys/personas residue).** The plan honestly carries this (A.0-3, L2.0b, R6) but then the §D L2 gate says "per-file count reconciles to the 831 census." If 831 itself has a sampled component until L2.0b completes, "reconciles to 831" is reconciling to a number with a soft tail. **Fix (wording):** the L2 exit should read "reconciles to the *post-L2.0b* census," making explicit that 831 is the census-of-record *after* L2.0b hardens the journeys/personas tail, not before. The plan's logic already intends this (L2.0b precedes L2.2 claiming coverage); the gate wording should match.

---

## 4. LANE-DEPENDENCY-EDGE REVIEW (the asked question: is L2-before-L1 sound, or does it fight L1?)

**Verdict: the ORDER is right; the plan's STATED BASIS for it is wrong (see FLAG-A). On the engineering merits, L2-before-L1-refound does not fight L1 — it serves it.**

- **The edge is engineering-correct.** If L1-refound re-authored first, every re-authored ADR-0000+ doc would hand-resolve `foundry` inline, then the bulk rename would re-touch the freshly-authored series → double-work + merge churn on supersede-only docs. Renaming the live Accepted corpus first means re-foundation consolidates already-clean text. The disposition table shows dozens of Accepted ADRs carrying "re-home foundry→intelligence/governance" amendments, so the overlap is real. **The driver D-1 is a valid engineering argument.** It is simply not a founder ruling (FLAG-A).

- **The round-1 carve is now correctly closed.** Round-1's sharpest finding — that L2 might rename a file L1-refound is about to archive — is resolved by G2/A.0-2: the consolidation-set freeze precedes L2.2, and L2.2 "skips any file in the A.0-2 consolidation set" (line 100), renaming the new ADR-0000+ text once instead. The edge is now correctly `consolidation-set-freeze → L2.2 (skip-archived) → L1-refound`. **This is the right shape and I have no further objection to it** beyond FLAG-C (prove the freeze is complete, not just frozen).

- **S1 L1-amend∥Wave-0 is sound** and does not fight L2: L1-amend touches only non-consolidation-set Accepted ADRs, which is the same mechanical class as L2's in-place de-foundry — they partition the corpus, they do not collide. Good.

- **Does the order fight D-LANES?** Yes, and the plan now says so (§A.4, named + signed). The wave-serialization narrows D-LANES; S1 minimizes the narrowing; §F signs it. **This is now handled correctly** — round-2's headline improvement. My only addition: the §A.4 narrowing is signed as a tradeoff *to buy D-1 and D-2*; since D-1 is actually DERIVED not RULED (FLAG-A), the §A.4 sign-off text should read "to buy D-2 (RULED: map-before-sweep) and the DERIVED D-1 (rename-before-reauthor, founder-ratifiable)" — so the founder knows the wave-serialization is bought partly for a constraint they are *also* being asked to ratify in the same signature, not a pre-existing law.

**Net on ordering:** order correct, carve closed, D-LANES handled. The defect is provenance (FLAG-A), not sequence.

---

## 5. BUILD-FIRST-CUTOVER-LATER SEQUENCING — now adequate, one residual (Tension B)

R2's `cutover_trigger:` required front-matter (pointing at ADR-0510/0250, verified named in SSOT `D-META:26`) closes the round-1 ownerless-pending gap: the pending-cutover set is now queryable with a named gate per entry, and the verifier confirms no `superseded-on-cutover` ADR lacks the field (§D, §G exit). **Correctly resolved.** The residual is Tension B / FLAG-B: making the pending set *queryable* is not the same as keeping the cross-ref graph *clean as it grows*. Add the Wave-3 cross-ref re-resolution (FLAG-B) and build-first is fully covered. The "don't archive the bridge before its owned replacement is proven" rule is correctly applied throughout (Jenkins/Argo stay operative; `superseded-on-cutover` not archived now) and matches SSOT D-META:26 / D3:118.

---

## 6. ASPIRATIONAL-TOOLING RISK (generator not on dev) — handled well; one honesty upgrade and one residual

Round-2 handles this correctly and at the right altitude:
- Every gate-dependent step carries the [ASPIRATIONAL] tag; the durable work (clean ADR front-matter) is correctly decoupled and lands on `dev` regardless of generator availability (D1's "backfill ≡ clean front-matter").
- **R1/FLAG-3 is the right structural fix:** supersede `decision-record D1` to record `generator=worktree-only@2026-06-06` so the aspirational status survives in the canon after this plan is archived. This is the model the plan should also apply to its ordering provenance (Tension A) — it already knows how to put "what is actually true vs asserted" into the canon; it just doesn't do it for its own RULED labels.
- **The safety sub-case (R1/§6) is correctly escalated:** the safety-gate ADR ships as design+invariant with its D16 runtime-Cedar hook marked `runtime-hook-pending`, NOT asserted-live. This is the highest-consequence aspirational-vs-enforced gap and the plan now refuses to overstate it. **I endorse this as the single most important safety call in the plan, and it is correct.**

**Residual (not a defect, a standing risk to track):** the manual cohesion stand-in (R3 adversarial lane) and the manual masterplan stand-in are both substitutes for gates that have *by-construction* guarantees the manual versions cannot fully replicate. The plan is honest (labels, falsification logs), but the founder should understand that **"verifier-green on a manual stand-in" is strictly weaker evidence than "gate-green on dev,"** and the gap persists until the generator+gates are ported. The R1 follow-up (port-to-dev as a first-class deliverable) is the right closure; I would add a **standing exit caveat** to §G: the boulder may stop with the amendment complete, but the "manual stand-in → real gate" debt remains open and tracked in the masterplan-wiring meta-ADR until ported. Don't let "amendment done" read as "gates live."

---

## 7. VERIFY-AT-EACH-STEP ADEQUACY — strong; the one hole is the plan's own provenance (FLAG-A) and the terminal graph (FLAG-B)

The separate-verifier-lane discipline is well-built and round-2 hardened it correctly: G3 MAP self-consistency verifier (closes the "who verifies the bijection" hole), G4 DROP inbound-safety, R3 adversarial cohesion, R4 published falsification log (closes the "unfalsifiable exit" hole). The §D per-lane gates are concrete and greppable. Remaining holes:

1. **The verifier verifies execution against the plan's stated rulings, but nobody verified the plan's stated rulings against the SSOT (FLAG-A).** The whole verification apparatus checks "did we do what the plan says the SSOT rules" — but the plan misquotes the SSOT on the load-bearing ordering edge. A verifier re-reading the SSOT (per the founder rule) to check the L2-before-L1 edge would find no such ruling and should raise it. **The verification design is sound; it was just never pointed at the plan's own RULED claims.** Fix: a one-time "ruling-provenance" verifier pass — for each "[RULED]" tag in the plan, confirm a citable SSOT line exists; downgrade any that don't to [DERIVED]. (This finds exactly FLAG-A and confirms the rest.)
2. **Terminal cross-ref graph unverified post-Wave-2 (FLAG-B).** Add the Wave-3 re-resolution.
3. **Consolidation-set completeness assumed, not proven (FLAG-C).** Extend L1.0-VERIFY.

---

## 8. SYNTHESIS — improvements that strengthen the plan without breaking any ruling

- **SY1 (closes FLAG-A, the new HIGH).** Introduce a provenance tag on every ordering constraint, mirroring the plan's own ASPIRATIONAL/ENFORCED discipline: **`[RULED:<SSOT-line>]`** vs **`[DERIVED:<driver>]`**. Relabel "rename-before-reauthor" as `[DERIVED: D-1 engineering double-work]`, delete the false "decision record explicitly rules … dependency note" sentence (line 53), and **move the Option-1-vs-Option-2 wave-shape into the §F founder sign-off set as a real (not pre-decided) choice.** Keep Option 1 as the recommended call with its engineering rationale intact. Net effect: the order doesn't change; its honesty does. Run a one-time ruling-provenance verifier pass (§7 hole 1) to catch any other DERIVED-labeled-as-RULED edges.

- **SY2 (closes FLAG-B + Tension B).** Add a **Wave-3 terminal whole-corpus cross-ref re-resolution gate** — re-run the L3.3 resolver after the last Wave-2 supersession lands, asserting zero dangling supersedes/amends on the *delivered* graph (not just the Wave-0 snapshot). Wire into §D Wave-3 gate + §G exit. Cheap (re-runs an existing gate).

- **SY3 (closes FLAG-C).** Extend the L1.0-VERIFY gate (G3) to prove consolidation **completeness against a stated closure criterion**, not just surjection — so the S1 L1-amend partition is provably total (no Wave-1 re-foundation can discover a missed fold that retroactively wastes an L1-amend rename).

- **SY4 (closes FLAG-D, wording).** Change the §D/§H.2 L2 exit from "reconciles to the 831 census" to "reconciles to the *post-L2.0b* census," so the soft journeys/personas tail is acknowledged as hardened-by-L2.0b, not assumed-hard at 831.

- **SY5 (closes §6 residual).** Add a standing exit caveat to §G: amendment-complete ≠ gates-live; the "manual stand-in → ported gate" debt (generator, cohesion, drift, safety runtime hook) remains open and tracked in the masterplan-wiring meta-ADR until ported to `dev`. Prevents "boulder stopped" from being misread as "tooling real."

---

## 9. BOTTOM LINE

Round-2 is a strong revision: it absorbed all of round-1's G1–G5 / R1–R6 / S1–S6, the four headline facts are settled and correctly wired, the consolidation-set carve is closed (G2), the MAP and DROP gates are real (G3/G4), the D-LANES narrowing is named and signed (G5/§A.4), and the safety `runtime-hook-pending` call is exactly right.

The new finding is provenance, not sequence: **the plan's load-bearing "rename-before-reauthor" edge is asserted as RULED in three places, but the SSOT rules no such ordering** (D11 = sweep-before-backfill only; D13 = re-founding with no ordering vs the rename; the quoted "dependency note" does not exist — verified by grep this round). All three prior lanes certified this edge as founder-ruled without checking its SSOT provenance — the exact verdict-propagation the founder rule forbids. The edge is *engineering-correct* (the double-work argument is valid and sufficient on its own), so **the order does not change** — but its **status** must change from "non-negotiable RULED" to "founder-ratifiable DERIVED," and the Option-1 wave-shape must enter the §F sign-off set as a genuine founder choice rather than a closed ruling (SY1).

Two smaller real gaps: the terminal cross-ref graph is never re-verified after Wave-2 supersessions (FLAG-B/SY2, cheap fix), and the consolidation-set is assumed complete rather than proven complete (FLAG-C/SY3).

**Verdict: APPROVE-WITH-CONDITIONS.** Conditions: apply SY1 (HIGH — relabel the false RULED, move the wave-shape into founder sign-off, run the provenance pass) **before founder sign-off**, since it changes *what the founder is signing*; apply SY2–SY3 before Wave-3 / before the consolidation freeze is treated as load-bearing; fold SY4–SY5 as wording/tracking. None of these change the recommended wave order — Option 1 remains right — they correct who-decided-it and close two verification holes. With SY1 applied, the founder is signing an honest plan instead of one that has quietly promoted a reviewer's good engineering instinct into a founder ruling.
