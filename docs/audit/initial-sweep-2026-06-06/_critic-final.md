# CRITIC FINAL — AMENDMENT-PLAN round-4 consensus check (against round-3 conditions)

> Reviewer: critic lane, SEPARATE from the planner/executor who revised the plan and from the round-3 architect/critic lanes. Founder rule honored: I re-grepped + line-read the SSOT (`synthesis/decision-record-oyatie-canon.md`) MYSELF this round and did NOT trust the plan's self-report, the round-3 architect's self-report, or the round-3 critic's self-report. Every claim below cites a file+line I personally read or a grep I personally ran. No phantom findings. **Date:** 2026-06-06. **Round:** 4 (final consensus).

---

## 0. BOTTOM LINE

The revision **closes both round-3 HIGH blocking findings on the merits** and does NOT over-correct: Finding-1's RULED re-tag of L2 is **genuinely grounded in the SSOT** (I verified D11`:52`(d) verbatim) — it is a correct relabel, not a RULED-inflation in the opposite direction. Finding-2's design-freeze/Wave-1-confirm split removes the door:one-way-over-a-guess defect. FIX-3 (terminal barrier) and FIX-4 (`cutover_status:`) are present and sound. The wave order (Option 1) is unchanged and re-verified correct.

**One residual gap survives, and it is the EXACT gap the round-3 critic's iteration item #1 explicitly pre-warned about**: the bidirectional §A.6 mandate landed in the §A.6 prose (lines 93-107) and the §H.2 evidence row (line 320), but the **§E sign-off gating predicate (line 229) and the §G sign-off entry/exit predicates (lines 249, 261) still describe the §A.6 pass in one-directional language only** — they restate "every `[RULED]` tag resolves to an SSOT line; no `[DERIVED]` edge presented as law" and "a pass that surfaces additional false-RULED tags is a re-review trigger," with **zero mention of direction-2 (RULED-deflation)**. The round-3 critic's item #1 named this precisely: *"the bidirectional mandate must be added to §A.6 itself AND to the §H.2 A.6 row AND to the §G/§E gating predicate … all four currently encode a one-directional pass. A patch that fixes only §A.5's L2 tag but leaves the §A.6 mandate one-directional would let the next ordering constraint slip through the same hole."* Two of the four loci (§E, §G) were not updated.

This is **MAJOR, not a fresh HIGH blocker**: the gating predicates point BY REFERENCE to "the §A.6 ruling-provenance pass," and §A.6 itself is now bidirectional, so an executor running "the §A.6 pass" runs both directions. The defect is that the predicate PROSE that a founder/reviewer reads at sign-off time still advertises a one-directional contract — the same prose-vs-mechanism drift that let the round-2 over-exclusion and round-3 deflation survive. It must be fixed before APPROVE, but it is a two-line edit to two loci, not a structural re-design.

**VERDICT: ITERATE** — one MAJOR residual (the §E/§G predicate prose was not made bidirectional, against the round-3 critic's explicit four-loci instruction) plus two MINOR consistency nits. The two HIGH findings are substantively CLOSED; the wave order is correct; no door:one-way is signed over a guess; no over-correction. This is a near-miss, not a re-litigation.

---

## 1. INDEPENDENT SSOT RE-DERIVATION (trust nothing as-is)

| Claim under test | My grep/read THIS round | Verdict |
|---|---|---|
| `grep double-work` over SSOT | **ZERO** (exit=1, re-confirmed) | D-1 still correctly not-SSOT-ruled |
| D11`:52`(d) names the foundry rename as sweep content | **CONFIRMED VERBATIM**: `:52` = `**Ruling: authorize the FULL sweep** (runs before masterplan backfill, each fix read-only-verified): … (d) foundry→cloud-intelligence/governance bulk-rename (0347 Proposed→Accept; fix 0363's false "eradicated" claim). **Door:** one-way (batch).` | **Finding-1 re-tag GROUNDED — the foundry rename IS a named content (d) of the batched one-way pre-backfill sweep. Re-tagging L2 sweep-membership `[RULED: D11(d)+:52]` is CORRECT, not over-correction.** |
| D13`:43` no-id-reuse + no-dangling invariant | **CONFIRMED VERBATIM** (`:43` "Strict **no-id-reuse + no-dangling-ref invariant**; renumber linux 0001–0026 → **0515+**… **Door:** one-way") | map-before-sweep (D-2) RULED still grounded; terminal barrier (D13`:43`) grounded |
| D3`:114-118` gives fold INTENT (cluster), not a file map | **CONFIRMED**: `:115` "supersedes 0124 · phases 0369 · builds 0367 · sequences 0366 last"; `:118` "**seed ONE clean ratifying ADR (ADR-0000+ series)** … supersede/relate 0511/0124; mark 0369/0367/0366 phased." The file-level fold map is the OUTPUT of authoring that ADR (Wave-1). | **Finding-2 GROUNDED — SSOT pins cluster-level fold INTENT, NOT a frozen file map. The Wave-0 freeze of a file map would indeed be a door over a guess.** |
| D-META`:26` build-first-cutover-later keeps bridges live (graph grows) | **CONFIRMED VERBATIM** (`:26` "**Never retire/archive a bridge before its owned replacement is built and proven** … marked 'superseded-on-cutover (pending build+proof)', not archived immediately") | terminal-barrier trilemma (FIX-3) grounded |
| D-LANES`:84` is a door:one-way organizing principle | **CONFIRMED VERBATIM** (`:84` "treat everything as PARALLEL LANES … Fractal parallel/sequential structure (not a global serial chain) … **Door:** one-way (organizing principle)") | §A.4 narrowing-is-itself-one-way framing grounded |
| census 831/43 at `:107` | **CONFIRMED VERBATIM** (`:107` "total non-ADR = 831 (not 731), Palantir-Foundry carve-out = 43 files (not 105)") | census RULED, intact |

**All seven load-bearing SSOT anchors verified by me directly this round. Both round-3 findings rest on real SSOT lines. The architect and round-3 critic honored the founder rule; so do I.**

---

## 2. FINDING 1 (RULED-deflation of L2-in-the-D11-sweep) — substantively CLOSED, one prose-locus gap

### 2(a) MEMBERSHIP re-tagged RULED citing D11 — ✅ CONFIRMED, GROUNDED, NOT over-corrected
- §A.5 line 84 adds the new row: **"L2 foundry-rename ∈ the pre-backfill sweep (sweep-membership) | `[RULED]` (R3/Finding-1) | D11`:52` content (d) … is a NAMED CONTENT of 'the FULL sweep (runs before masterplan backfill …) Door: one-way (batch)'."** I verified D11`:52`(d) verbatim (§1 above): the tag is correct.
- §B-L2 header line 135 re-tagged: **"L2's membership in the pre-backfill sweep is `[RULED: D11(d)+:52`…`]` (R3/Finding-1 — re-tagged from DERIVED this round)."** Present.
- §C line 183 carries the edge `L2 foundry-rename ∈ pre-backfill sweep ── [RULED: D11(d)+:52, batched one-way before backfill]`. Present.
- Critical-path RULED set (line 202) now lists **"L2-foundry-rename-∈-the-pre-backfill-sweep (D11(d)+`:52` — R3/Finding-1, re-tagged RULED this round)"** in the hard `[RULED]` set. Present.
- **Over-correction check (the opposite failure):** the plan correctly keeps **rename-vs-reauthor *internal sequencing* `[DERIVED: D-1]`** (§A.5 line 91, §B-L2 line 135, §C line 184). The distinction is stated precisely: *the rename happening inside the pre-backfill sweep is RULED; whether it lands before-or-interleaved-with re-authoring is DERIVED.* This is the surgically-correct split — it does NOT inflate the sequencing edge to RULED. **No over-correction. The SSOT genuinely rules membership (D11(d)) and genuinely does NOT rule the internal sequencing (grep double-work=0). The plan now matches the SSOT in both directions.**

### 2(b) §A.6 made BIDIRECTIONAL — ✅ in §A.6 + §H.2, ❌ NOT in §E/§G predicate prose [MAJOR residual]
- §A.6 PROSE: **bidirectional, correct.** Line 93 title "**BIDIRECTIONAL (R3/Finding-1)**"; lines 97-101 Direction-(1) RULED-inflation; lines 103-105 Direction-(2) RULED-deflation ("for **every D-decision in the SSOT that names a concrete amendment ACTION** … confirm the plan's corresponding lane carries the matching `[RULED:<SSOT-line>]` tag. Surface any ruled action the plan left `[DERIVED]`/untagged as a re-review trigger"). Line 107 exit gates BOTH directions. This is exactly the mandate the architect's synthesis #1 and the round-3 critic's item #1 specified. **Correct.**
- §H.2 A.6 evidence row (line 320): **bidirectional, correct.** Carries "Direction (1) RULED-inflation … Direction (2) RULED-deflation [R3/Finding-1] … L2's sweep-membership is tagged `[RULED: D11(d)+:52]` … no SSOT-ruled action left DERIVED/untagged. Any further false-RULED tag (dir 1) OR any RULED-deflated action (dir 2) = re-review trigger." **Correct.**
- **§E gating predicate (line 229): NOT bidirectional.** Reads: *"before ANY §E door:one-way sign-off, the §A.6 ruling-provenance verifier pass MUST be green — every `[RULED]` tag in the plan resolves to a citable SSOT line; no `[DERIVED]` edge is presented as non-negotiable law. … A pass that surfaces additional false-RULED tags is a re-review trigger, not a silent fix."* This is the round-2 ONE-DIRECTIONAL contract verbatim. No mention of RULED-deflation / "every SSOT-ruled action resolves to a matching RULED tag" / direction-2.
- **§G sign-off entry predicate (line 249): NOT bidirectional.** Reads: *"the §A.6 ruling-provenance pass is green (every `[RULED]` tag resolves to an SSOT line; no `[DERIVED]` edge presented as law; the fabricated quote deleted)…"* One-directional.
- **§G exit criterion (line 261): NOT bidirectional.** Reads: *"§A.6 ruling-provenance pass green (every RULED tag SSOT-backed; D-1 correctly DERIVED; fabricated quote deleted)…"* One-directional.

**Verified by grep:** `awk 'NR>=225 && NR<=269' AMENDMENT-PLAN.md | grep -i "deflation|bidirectional|direction.2|matching RULED|forward into the plan"` returns **ZERO matches.** The §E/§G predicate prose was not touched for direction-2.

**Why this is MAJOR and not cosmetic:** the §E/§G predicates are *the contract the founder and verifier read at sign-off time.* The round-3 critic's item #1 said verbatim that leaving these one-directional "would let the next ordering constraint slip through the same hole" and named "the §E/§G gating predicate" as a required locus. The plan's own §A.6 is now bidirectional, so the *mechanism* is correct — but the predicate prose still advertises the old one-directional contract. This is precisely the prose-vs-mechanism drift that produced the round-2 and round-3 defects (the §A.6 mechanism caught inflation but the framing claimed it proved "honest orderings"). A reviewer who reads only the §E/§G entry predicate (the natural sign-off checklist) would believe a one-directional pass suffices. **The fix is two lines** (append "AND every SSOT-ruled amendment action resolves to a matching RULED tag (direction-2); no SSOT ruling is silently deflated to DERIVED" to lines 229, 249, 261). But it MUST be done — the round-3 critic explicitly gated on it ("the structural fix (bidirectional mandate in all four loci) is the gating item").

### 2(c) Option 2's legal shape restated as BOUNDED — ✅ CONFIRMED, consistent everywhere
- §A.3 Option-2 header (line 54): **"[LEGAL SHAPE RESTATED — R3/Finding-1] … the foundry rename MUST still sit INSIDE the pre-backfill sweep (D11(d)+`:52`, RULED). Option 2's ONLY legal shape is therefore 'author the ADR-0000+ shells first, THEN run the (still-pre-backfill) sweep over them.' An Option-2 that runs the foundry rename after re-foundation AND after masterplan backfill is NOT a legal alternative."** Grounded and correct (I verified D11 puts the rename before backfill; re-foundation feeds backfill).
- §A.3 invalidation rationale (line 58): explicitly **"It is NOT a clean free choice and NOT excluded by fiat … it neither forecloses Option 2 with a non-existent ruling (round-2 over-exclusion fixed) nor presents it as fully-legal when D11(d) partially binds it (round-3 over-inclusion fixed)."** This is the precise both-directions-honest framing. Correct.
- §E (line 228) and §F (line 241) both carry the BOUNDED framing consistently. **No residual "clean choice" framing survives** (grep for "clean choice|free choice|fully-legal" returns only the lines that explicitly NEGATE those framings). Correct.

**Finding-1 disposition: substantively CLOSED (2a + 2c fully correct, no over-correction). Residual: 2b — §E/§G predicate prose not made bidirectional (MAJOR, two-line fix, explicitly pre-gated by round-3 critic item #1).**

---

## 3. FINDING 2 (consolidation-set freeze paradox) — ✅ CLOSED, sound, no door over a guess

### 3(a) Split into Wave-0 PROVISIONAL design-freeze + Wave-1 CONFIRM — CONFIRMED
- §A.0-2 (line 23): retitled **"SPLIT into a Wave-0 PROVISIONAL design-freeze + a Wave-1 CONFIRM checkpoint [R3/Finding-2]"**; states the knowability paradox, then **"(a) Wave-0 PROVISIONAL design-freeze — freeze the re-foundation cluster-design decision (the D3/D5/D6/D7 fold INTENT the SSOT genuinely rules) … The founder signs the design-level classification, NOT a guaranteed file map. (b) Wave-1 CONFIRM-OR-AMEND checkpoint (L1.1-CONS-CONFIRM)."** Correct.
- L1.0-CONS (line 117): retitled **"PROVISIONAL DESIGN-FREEZE [R3/Finding-2]"**; **"the founder signs the DESIGN-LEVEL classification, NOT a guaranteed-final file map (R3/Finding-2/PV-2): the file-level fold map is the OUTPUT of Wave-1 authoring (L1.1), so a Wave-0 freeze of it would be a one-way door over a guess."** Correct — this is the architect's PV-2 resolved exactly.
- **L1.1-CONS-CONFIRM is a real new Wave-1 unit** (line 129): "**(seq, Wave-1, after each L1.1 ADR is authored, before L1.3 archival)** … confirms the actual file-level fold set produced by the authored ADR == the Wave-0 PROVISIONAL design-set." Wave membership line 113 confirms Wave-1 holds L1-refound; line 129 places CONFIRM there. §C edge line 178 wires it: `L1.1 re-foundation authored ──> L1.1-CONS-CONFIRM (file-fold == provisional? delta = re-review + revert)`. §D adds a dedicated gate (line 211). **Present and wired in all loci.**

### 3(b) Delta = re-review trigger (not silent amend); door no longer signed over unfinalized output — CONFIRMED
- Line 129: **"Any delta (a fold the provisional set missed …) is a RE-REVIEW TRIGGER that re-opens the A.0-2 door:one-way — NOT a silent amend (§A.1 principle-4: an unverified verdict never authorizes an irreversible act) … (b) revert any L1-amend rename already landed on a newly-discovered fold before the file is archived."** Correct — the door is re-opened on delta, so it is NOT signed over an unfinalized object; the founder signs the *design intent* (legitimately final at Wave-0, grounded in D3`:114-118` which I verified pins cluster intent), and the file map is confirmed-not-assumed.
- **Completeness-vs-provisional tension resolved cleanly:** L1.0-VERIFY (line 120) now reads **"provably total AT THE DESIGN LEVEL: the closure check proves every candidate is classified at Wave-0, but per R3/Finding-2 it cannot prove the classification won't change when the re-foundation is actually authored in Wave-1 (completeness ≠ stability-under-authoring). File-level stability is the separate L1.1-CONS-CONFIRM Wave-1 check."** This is the correct two-claims-two-checks separation the architect/critic demanded. The §G exit line 262 ("proven complete not just frozen") is consistent because "complete" is now explicitly design-level, not file-level-final. **No internal contradiction.**
- L1-amend (line 118): safety-net is **"CONDITIONAL on the Wave-1 L1.1-CONS-CONFIRM, not the Wave-0 provisional freeze … revert-on-delta."** The S1-recovery-vs-provisional-freeze tension the round-3 critic flagged (§4) is now acknowledged and resolved by the recoverable checkpoint. Correct.

**Finding-2 disposition: CLOSED. No door:one-way is signed over an unfinalized Wave-1 output. No over-correction (the design-intent freeze IS genuinely SSOT-ruled per D3`:114-118`).**

---

## 4. FIX 3 (Wave-3 terminal barrier named) — ✅ PRESENT & SOUND
- §A.4 (line 73): **"TWO acknowledged serialization points (R3/Finding-3 …): (1) the Wave-0 gate … and (2) the Wave-3 TERMINAL CROSS-REF BARRIER — the masterplan backfill (Wave-3) HARD-BLOCKS on the LAST Wave-2 supersession resolving on the delivered ADR graph … Both serializations are RULED-driven (D13`:43`) … S1 recovers within-Wave-0 parallelism but DOES NOT recover the terminal barrier."** The "minimal narrowing" claim (line 72/75) is now corrected to "within-Wave-0 minimal (post-S1) but carrying the named Wave-3 terminal barrier." Honest.
- §C edge line 194: **`LAST Wave-2 supersession ═TERMINAL BARRIER═> Wave-3 masterplan backfill (hard-block; 2nd serialization) [R3/Finding-3; D13:43; NAMED]`.** Named in the graph.
- **Grounding check:** the trilemma (build-first graph-grows D-META`:26` ⊥ D-LANES parallel`:84` ⊥ no-dangling-on-frozen-snapshot D13`:43`) is real — I verified all three SSOT lines. The terminal re-resolution (SY2, line 221/308) on the *delivered* graph is the correct resolution, and it does implicitly re-serialize (Wave-3 can't run until the last Wave-2 supersession lands). Naming it as a second serialization is accurate, not invented. **Sound.**

---

## 5. FIX 4 (cutover_status liveness field) — ✅ PRESENT & SOUND
- §E (line 231): **"[R3/Finding-4/PV-3 — aspirational-tooling liveness is QUERYABLE, not a dated snapshot]: each [ASPIRATIONAL] tooling item carries a `cutover_status:` front-matter field on the masterplan-wiring meta-ADR (L5.1) — oya_gen_masterplan / cohesion_gate / drift_gate / safety_runtime_hook, each valued `worktree-only` now and flipped to `live-on-dev` when ported."** Present.
- L5.1 (line 156): carries the field as a first-class deliverable, mirroring the already-mandated `cutover_trigger:` pattern, with the `decision-record D1` supersede recording the field's existence + initial values (supersede-never-edit honored). Present.
- §G standing caveat (line 268) + §H.2 L5 row (line 328) + Failure-3 mitigation (line 299): all reference the queryable field, "each item is not 'done' until its `cutover_status:` reads `live-on-dev`." **Consistent across loci.**
- **Soundness:** it reuses the plan's own already-accepted `cutover_trigger:` mechanism, so it introduces no new machinery; it closes the "canon carries a stale dated snapshot" gap the architect's PV-3 named. **Sound, LOW-value, correctly LOW-severity.**

---

## 6. WAVE ORDER — ✅ UNCHANGED AND STILL CORRECT
Wave membership (line 113): Wave-0 = A.0-1 + A.0-2 + L3 + L2 + L6 + L1-amend ∥ L1.0 MAP-freeze; Wave-1 = L1-refound + L1.2 Proposed batch + L1.1-CONS-CONFIRM; Wave-2 = L4 + L5 + L7; Wave-3 = masterplan backfill (terminal). This matches Option 1 (§A.3 line 48-49). The two new units (L1.1-CONS-CONFIRM, the terminal barrier) are *additions within existing waves*, not reorderings. D-2 (map-before-sweep, D13`:43`), D11 (sweep-before-backfill), census (`:107`) all stay RULED and intact. **The order is over-determined by D-2 (RULED) + L2-sweep-membership (RULED, new this round) + D-1 (DERIVED, sound). Confirmed correct a fourth time.**

---

## 7. OVER-CORRECTION / NEW-CONTRADICTION SWEEP (the watch-items)
- **RULED-inflation introduced by the fix?** NO. The only new RULED tag is L2-sweep-membership = D11(d)+`:52`, which I verified verbatim is a named sweep content under a one-way batch door. The internal-sequencing edge correctly STAYS DERIVED. The relabel-up is as surgical as round-2's relabel-down (round-3 critic §6.3 endorsed exactly this — I independently re-confirm it).
- **New internal contradiction?** NO. The completeness-vs-provisional tension (the obvious risk) is explicitly resolved: line 120 scopes completeness to "design level," line 262 exit says "complete not just frozen" (design-level), line 129 puts file-level stability in the Wave-1 confirm. The two checks prove two genuinely different claims. No contradiction.
- **Any door:one-way still signed over a guess?** NO — A.0-2 is now the PROVISIONAL design-intent (SSOT-ruled per D3`:114-118`), with the Wave-1 confirm + door-re-open-on-delta. The MAP door (L1.0) is still gated on L1.0-VERIFY bijection+completeness (G3). No door over an unverified/unfinalized object remains.
- **Acceptance criteria / pre-mortem / verification adequate?** Pre-mortem: Failure-6 (line 310) added for the freeze-over-provisional-map scenario; Failures 0-5 retained. Deliberate bar (pre-mortem + expanded per-lane verification §H.2) MET. Verification: §D per-lane gates + the new L1.1-CONS-CONFIRM gate (line 211) are present; the ONE hole is that the §E/§G *predicate prose* (not the §A.6 mechanism) still advertises a one-directional pass (§2b).

---

## 8. ENFORCEMENT-CHECKLIST SCORING

| Bar | Status | Note |
|---|---|---|
| Principle-option consistency | **PASS, one MAJOR prose-locus gap** | PV-1 mechanism fixed (§A.6 bidirectional); PV-2 fixed (design-freeze/Wave-1-confirm). Residual: §E/§G predicate PROSE not made bidirectional (§2b). |
| Fair alternatives | **PASS** | Option 2 restated as BOUNDED, consistent in §A.3/§E/§F; neither over-excluded (round-2) nor over-included (round-3). Grounded in verified D11(d). |
| Risk-mitigation clarity | **PASS** | S1-recovery-vs-provisional tension acknowledged + resolved via revert-on-delta; terminal barrier named; Failure-6 added. |
| Testable acceptance criteria | **PASS** | §H.2 greppable; published-falsification-log exit retained; terminal barrier now in §G exit + §C. |
| Concrete verification steps | **PASS, one prose hole** | §D gates + L1.1-CONS-CONFIRM gate real; §A.6 mechanism bidirectional. Hole: §E/§G predicate prose (§2b). |
| Deliberate-mode: pre-mortem + expanded verification | **PASS** | Failures 0-6 concrete + gate-mapped; §H.2 per-lane evidence table bidirectional in the A.6 row. |

Five bars PASS; one PASS-with-a-MAJOR-prose-gap. The gap is localized, named, two-line-fixable, and was explicitly pre-gated by the round-3 critic — so this is ITERATE (near-miss), not APPROVE (the predicate prose a founder reads at sign-off still advertises a one-directional contract) and emphatically not REJECT (both HIGHs substantively closed, wave order correct, no over-correction, no door over a guess).

---

## 9. REQUIRED ITERATION ITEMS (the remaining blockers)

1. **[MAJOR — gating before APPROVE] Propagate the bidirectional §A.6 mandate into the §E and §G predicate PROSE (the two loci round-3 critic item #1 named that were missed).** Append to line 229 (§E gating predicate), line 249 (§G sign-off entry predicate), and line 261 (§G exit criterion) the direction-2 contract, e.g.: *"AND every SSOT D-decision that names a concrete amendment action resolves to a matching `[RULED]` tag in the plan (direction-2); no SSOT ruling is silently deflated to DERIVED. A pass that surfaces a further false-RULED tag (dir-1) OR a RULED-deflated action (dir-2) is a re-review trigger."* Evidence the gap is real: `awk 'NR>=225 && NR<=269' AMENDMENT-PLAN.md | grep -i "deflation|bidirectional|direction.2"` → ZERO matches; §A.6 (93-107) and §H.2 (320) ARE bidirectional but §E/§G predicate prose is not. This is the plan's own round-3 instruction ("bidirectional mandate in all four loci") only 2/4 satisfied.

2. **[MINOR] Failure-6 line 310 cross-reference.** Failure-6 ("Wave-0 consolidation-set freeze signed door:one-way over a PROVISIONAL fold map") is present and correct, but the round-3 critic's item #5 asked it be added as a tracked pre-mortem entry — it IS present (good); confirm its mitigation pointer says "item #2's design-freeze/Wave-1-confirm split" maps to the now-implemented L1.1-CONS-CONFIRM (it does, via "[R3/Finding-2] SPLIT the freeze" line 312). No change needed beyond verifying the cross-link reads cleanly. (Trivial — flagged for completeness only.)

3. **[MINOR] §I.3 round-3 disposition row for Finding-1 (line 374)** lists "§E + §F wave-shape (bounded choice)" and "§H.2 A.6 row (both directions)" as where Finding-1 was applied, but does NOT list the §E/§G *ruling-provenance predicate* among the bidirectional-updated loci — consistent with the actual gap in #1 (they weren't updated). When #1 is fixed, add §E line 229 + §G lines 249/261 to this disposition row so the audit trail is honest.

---

## 10. SYNTHESIS — what is settled, what remains

**Settled (verified TRUE this round):** Finding-1's L2 RULED re-tag is grounded in verbatim D11`:52`(d) and is NOT an over-correction (internal sequencing correctly stays DERIVED); Finding-2's design-freeze/Wave-1-confirm split removes the door-over-a-guess and introduces no completeness-vs-stability contradiction; FIX-3 (terminal barrier) and FIX-4 (`cutover_status:`) are present, grounded, and sound; the wave order (Option 1) is unchanged and correct a fourth time; Option 2 is BOUNDED-honest in all three loci; the pre-mortem (Failures 0-6) and expanded verification clear the deliberate bar; no door:one-way is signed over an unfinalized object.

**Remaining (this iteration, ONE MAJOR + two MINOR):** the bidirectional §A.6 mandate reached the §A.6 prose and the §H.2 evidence row but **NOT** the §E/§G sign-off predicate prose (lines 229, 249, 261) — the exact four-loci requirement the round-3 critic pre-gated on, only 2/4 met. The mechanism is bidirectional; the contract a founder reads at sign-off still says one-directional. Fix is two lines per locus. Until then, a reviewer keying off the §E/§G entry predicate (the natural sign-off checklist) would run/accept a one-directional pass — the precise prose-vs-mechanism drift that produced the last two rounds' defects. Plus two MINOR audit-trail consistency nits.

**This is a near-miss, not a re-litigation.** The substantive engineering is done correctly and the wave order is right. APPROVE is withheld solely because the round-3 critic explicitly made "bidirectional mandate in all four loci" the gating item and 2/4 loci were missed — granting APPROVE would ship a sign-off contract that still advertises the one-directional pass the whole round-3 finding exists to kill. One more tight pass closes it.

---

VERDICT: ITERATE

REMAINING BLOCKERS:
- [MAJOR, gating before APPROVE] The bidirectional §A.6 ruling-provenance mandate (direction-2 / RULED-deflation) is present in §A.6 prose (lines 93-107) and the §H.2 A.6 evidence row (line 320) but is ABSENT from the §E sign-off gating predicate (line 229), the §G sign-off entry predicate (line 249), and the §G exit criterion (line 261) — all three still describe a one-directional pass ("every `[RULED]` tag resolves to an SSOT line … a pass that surfaces further false-RULED tags is a re-review trigger"), with zero mention of "every SSOT-ruled action resolves to a matching RULED tag." This is the EXACT four-loci requirement the round-3 critic's item #1 pre-gated ("the structural fix — bidirectional mandate in all four loci — is the gating item"); only 2/4 loci were updated. Fix: append the direction-2 contract to lines 229, 249, 261.
- [MINOR] §I.3 Finding-1 disposition row (line 374) should list §E/§G predicate loci as bidirectional-updated once the MAJOR fix lands (currently honest-by-omission since they aren't).

(Both round-3 HIGH findings — RULED-deflation re-tag and the consolidation-freeze paradox — are substantively CLOSED and correctly grounded with no over-correction; FIX-3 and FIX-4 are sound; the wave order is unchanged and correct. The sole gating item is the §E/§G predicate-prose propagation.)
