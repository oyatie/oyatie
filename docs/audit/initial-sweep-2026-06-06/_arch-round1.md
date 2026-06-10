# ARCHITECT REVIEW — AMENDMENT-PLAN round 1

> Reviewer: architect lane (separate from authoring lane). Verdict basis: read the full AMENDMENT-PLAN + the decision-record SSOT, and cross-checked load-bearing claims against the primary registers (`docs-sweep/00-REST-OF-DOCS-REGISTER.md`, `synthesis/03-PROPOSED-RESOLUTION-LEDGER.md`, `synthesis/01-ADR-DISPOSITION-TABLE.md`). No phantom findings: every claim below cites a file+line I actually read.
> **Overall: APPROVE-WITH-CONDITIONS.** The wave-ordering is sound and the three hard ordering constraints (rename→reauthor, map→sweep, sweep→backfill) are correctly derived. But the plan has one un-resolved data contradiction in its own cited source, one structural tension with a founder ruling (D-LANES vs the serial critical path), and an under-hedged dependency on a generator that does not exist on `dev`. None are fatal; all are fixable pre-Wave-0.

---

## 1. STRONGEST STEELMAN ANTITHESIS — "the plan is sequencing-correct but base-data-wrong, and that is the more dangerous failure"

The plan spends its entire §A energy defending the *ordering* (which wave runs first). That is the wrong thing to be most confident about, because **the ordering is already over-determined by three RULED dependencies** (D1 rename-before-reauthor, D-2 map-before-sweep, D11 sweep-before-backfill) — there was never a real degree of freedom there; Option 2 and Option 3 are strawmen that violate explicit rulings (the plan even says so at lines 40, 45). So §A is a long proof of something nobody contested.

Meanwhile the thing that actually determines whether L2 (the largest lane, 831 files, irreversible mass rename) succeeds — **the file census and carve-out set** — is internally contradictory *in the very register the plan cites as primary source*:

- `docs-sweep/00-REST-OF-DOCS-REGISTER.md:108` and `:146`: **"731 files (ADR-excluded)"**, Palantir carve-out = **"105 journey files"** (`:113`).
- `decision-record-oyatie-canon.md:107` (the SSOT): **"total non-ADR = 831 (not 731), Palantir-Foundry carve-out = 43 files (not 105)."**
- The plan adopts the corrected 831/43 (lines 6, 71, 72) — correctly — **but the register body it points L2's verifier gate at still says 731/105.**

This is the steelman: **the L2 verifier gate (§D line 134, §H.2 line 204) is told to assert "43 Palantir hits byte-unchanged" against a primary source that says there are 105.** A verifier that re-reads primary sources each iteration (as the founder rule demands, line 197) will read the *register*, find 105, and either (a) raise a false dangling-finding, or (b) — worse — "reconcile" to 105 and let ~62 mis-counted files through. The plan's own anti-phantom-findings discipline turns this latent contradiction into a guaranteed verifier stall or a silent miscount on the single most irreversible lane. **The biggest risk is not in the wave order the plan obsesses over; it is in the un-reconciled census the plan inherited and did not flag.**

Corollary blast-radius: the routing-rule split itself is approximate ("~274 intelligence / ~135 governance", 274+135 = 409, far short of 831 — the rest are carve-outs + FP + journeys/personas residue which the register itself flags as a *sampled estimate, not a census*, `00-REST-OF-DOCS-REGISTER.md:107` "journeys/personas residue = sampled estimate"). L2.2 "per-file rename" presumes a per-file census that does not yet exist for the journeys/personas bulk. The plan promotes an estimate to an execution target.

**Why this is the best argument the plan is wrong/risky:** it attacks the plan on its *own* terms (verify-at-each-step, no-phantom-findings, primary-source-grounded) and shows the plan ported a known-imprecise number into an irreversible mass-edit lane without a reconciliation step. A mis-routed `oya-foundry-*` → `oya-intelligence-*` rename on a governance-sense file is itself a *new* canon contradiction (the exact CC-1 class the amendment exists to kill), produced by the amendment.

---

## 2. TRADEOFF TENSIONS (at least one real, load-bearing)

### TENSION A (primary) — D-LANES "everything is parallel" vs the plan's 7-node serial critical path
The founder RULED (decision-record D-LANES, line 83): *"treat everything as PARALLEL LANES … not a global serial chain."* The plan's §C critical path (line 125) is:
`L1.0 → (L2+L3+L6) → L1.1 → L1.2 → L1.3 → L5 → L7 → Wave-3`
— a **strictly serial 7-stage chain**, with each stage gating the next. The plan acknowledges the tension and answers it (line 47: "Option 1 is a wave-ordering over parallel lanes, not a global serial chain"), but the answer is partly rhetorical: *within* a wave there is real parallelism, but the *waves themselves* are serial, and the waves are where the wall-clock lives. Wave-1 (L1 re-foundation, ~the whole ADR-0000+ series + 132 Proposed) cannot start until Wave-0 fully closes, and Wave-2/3 cannot start until Wave-1 closes.

This is a genuine tradeoff, not a defect: the serial waves *buy* the double-work avoidance (D-1) and stable-id-space (D-2) that the rulings demand. You cannot have both maximal parallelism (D-LANES) and single-pass-over-clean-text (D-1). **The plan silently resolves this in favor of D-1 over D-LANES without naming that it is overriding a founder ruling to do so.** That should be an explicit, signed tradeoff in §A, not an implicit one. (See SYNTHESIS S1 for the partial reconciliation that recovers most of the lost parallelism.)

### TENSION B — Build-first-cutover-later vs one-doc-per-PR throughput on a frozen-immutable corpus
Build-first-cutover-later (line 18) + supersede-never-edit (line 17) + one-doc-per-PR (line 20) interact badly under linear history (line 20). Every superseded ADR becomes a *new* ADR-0000+ doc (its own PR) carrying `superseded-on-cutover (pending build+proof)`, while the old frozen ADR stays live. So the corpus **grows** during the amendment (old + new coexist by design until cutover, which is deferred indefinitely per build-first). The cross-ref graph the L3 sweep must keep clean is therefore a *moving, growing* target even after the renumber MAP freeze, because every Wave-1/Wave-2 supersession adds edges. The plan's "freeze the MAP first" (D-2) stabilizes *renumber* edges but not *supersession* edges authored later. Tension: the more faithfully you honor build-first (don't retire bridges), the larger the live-but-superseded surface the verifier must re-validate every iteration.

### TENSION C — Verifier-as-gate-substitute vs verifier-independence
§H.1 Failure-3 mitigation (line 195) makes the verifier lane *stand in for* the absent cohesion/drift gate ("verifier reads the sibling read-set and asserts no contradicting atom"). But §A.1 principle 4 (line 19) and §D (line 131) require the verifier to be a *read-only checker that is not the authoring lane*. Asking the verifier to perform the cohesion *judgment* the gate would automate makes it an authoring-adjacent decision-maker, not a checker — and there is no second lane to verify the verifier's manual cohesion call. The plan has the verifier both *produce* the cohesion verdict and *be* the only check on it. That is a self-approval the founder rule forbids (line 19 "no self-approval").

---

## 3. PRINCIPLE-VIOLATION FLAGS

- **[FLAG-1 — MEDIUM, source-data] Cited primary source contradicts the SSOT correction, unreconciled.** `00-REST-OF-DOCS-REGISTER.md` body (731/105) vs decision-record `:107` (831/43). The plan uses 831/43 but never instructs anyone to *fix the register* so the verifier reads a consistent number. Violates "verify-at-each-step against primary sources" because the primary source is self-inconsistent. **Pre-Wave-0 fix:** patch the register §2/§footprint counts to 831/43 with a correction note (one-doc-per-PR), OR make `decision-record:107` the explicit single census-of-record and re-point the L2 gate (§D/§H.2) at it. Do this *before* L2.0 routing-rule freeze.

- **[FLAG-2 — MEDIUM, ruling-override unstated] D-LANES override is implicit.** See Tension A. The plan overrides "everything parallel" with a serial wave chain for sound D-1/D-2 reasons but does not surface this as a founder-signed tradeoff. Since D-LANES is itself door:one-way (decision-record line 84), *overriding its shape* arguably needs the same sign-off the plan demands for everything else. **Fix:** add a line to §A.2 / §F naming "wave-serialization is a deliberate, founder-acknowledged narrowing of D-LANES, justified by D-1+D-2; within-wave parallelism preserved."

- **[FLAG-3 — MEDIUM, aspirational-tooling drift at the SSOT layer] The SSOT asserts the generator as real; only the plan hedges it.** `decision-record D1:10` states the masterplan is generated by `oya gen masterplan` + drift gate "per the already-Accepted ADR-0364/0365" with **no** worktree/aspirational caveat. The plan correctly flags it [ASPIRATIONAL] (lines 8, 26, 117, 122, 148, 195), but **the SSOT it derives from does not** — so a future reader of the decision-record (the authority) will believe the gate exists. Worth-documenting⇒worth-reading⇒reachable cuts both ways: the *aspirational status* is worth-documenting and is currently only reachable from the plan, not the canon. **Fix:** the masterplan-wiring meta-ADR (L5.1/L1.1) must carry the [ASPIRATIONAL→port-to-dev] follow-up as a first-class ADR deliverable, and the decision-record D1 should be amended (supersede, not edit) to record that the generator is worktree-only as of 2026-06-06.

- **[FLAG-4 — LOW, self-approval risk] Verifier stands in for the cohesion gate AND is the only check on its own cohesion verdict** (Tension C). **Fix:** when the verifier performs the manual cohesion stand-in, route the *cohesion verdict itself* to a second (opus) verifier or to the founder door:one-way — never single-lane self-cleared.

- **[FLAG-5 — LOW, scope-of-estimate] "Per-file" rename built on a sampled estimate** for the journeys/personas bulk (register `:107`, `:22` "325 raw mostly mechanical/FP"). The plan's L2.2 "per-file rename per file family" implies a census the register says it does not have for that lane. **Fix:** add an L2.0b "complete the journeys/personas census" step (template-first, since the register flags `intern-month-one.md` 138-hit fabricated-precision filler at `:164` — regenerate-from-template kills most of it O(1)) before claiming per-file coverage.

---

## 4. LANE-DEPENDENCY-EDGE REVIEW (the question asked: is L2-before-L1 right, or does it fight L1?)

**Verdict: L2-before-L1 (rename precedes re-foundation) is CORRECT and does NOT fight L1 — with one carve.**

- The edge `L2 foundry-rename → L1.1 ADR-0000+ re-author` (line 111, driver D-1, RULED) is sound: if you re-author into the clean series *first*, every re-authored ADR hand-resolves `foundry` inline, then the bulk rename re-touches the same freshly-authored files → double-work + merge churn on immutable docs (which, being supersede-only, are expensive to re-touch). The decision-record backs this (the disposition table shows dozens of Accepted ADRs with "re-home foundry→intelligence/governance" amendments, e.g. ledger `:45 :46 :47 :55`). Renaming the *live Accepted corpus* in place first means the re-foundation consolidates already-clean text. **Confirmed correct.**

- **The carve (real friction):** L2 renames the *live* corpus, but L1.1 *re-authors a subset of that corpus into new ADR-0000+ docs and archives the originals* (D13, line 17). For any ADR that is BOTH foundry-renamed (L2) AND re-founded (L1.1), the L2 work on the soon-to-be-archived original is **discarded** — you renamed a file you are about to archive. The plan's own §A.3 Option-1 *cons* (line 33) gestures at this ("first wave touches many already-Accepted ADRs in place … not being re-founded, only de-foundried") but the disambiguation "which Accepted ADRs are re-founded vs merely amended-in-place" is **not frozen anywhere** — and it must be, because it decides whether an L2 rename on a given file is useful or wasted. **This is a missing primary source:** the ADR-0000+ *consolidation map* (which old ids fold into which 0000+ ADR) is referenced as part of L1.0 (line 57) but L2 runs in Wave-0 *before* L1.1 proves that map. So L2 cannot know which files are throwaway. **Fix:** the L1.0 MAP freeze must include the consolidation set *before* L2.2 executes, so L2 can skip renaming files destined for archival-by-re-foundation (rename the *new* ADR-0000+ text instead, once). This tightens the edge to: `L1.0 (MAP incl. consolidation set) → L2.2 per-file rename (skipping to-be-archived originals) → L1.1 re-author`. The plan currently lets L2.2 and the consolidation-set freeze float in the same wave without ordering them.

- `L6 vocab → L1.1` (line 113) and `L3 sweep → backfill` (line 114): both clean, correctly Wave-0/pre-backfill, no objection.

- `L1.0 MAP → L3.3 dangling-ref` (line 109, driver D-2): correct and well-defended (Failure-2, line 191). The branch-locality edges (0421/0457/0429/0443/0428) deferred to merge-time (line 67) is the right call — don't fake them now.

**Net on ordering:** the wave order is right. The one real edge defect is intra-Wave-0 ordering (consolidation-set-freeze must precede per-file rename), not the inter-wave order the plan litigated.

---

## 5. BUILD-FIRST-CUTOVER-LATER SEQUENCING — adequate, with one gap

The rule is correctly applied: superseded-by-unbuilt → `superseded-on-cutover (pending build+proof)`, not archived (lines 18, 149; decision-record D-META:26, D3:118). Jenkins/Argo stay operative. **Gap:** the plan never defines *who fires the cutover gate* or *where the proof lives* for the amendment-created supersessions. "pending build+proof" is a status with no owner and no evidence-pointer in the plan. Over a multi-quarter horizon this status accretes (Tension B) into a large "pending" set with no triggering authority recorded. **Fix:** every `superseded-on-cutover` ADR must cite its cutover-trigger ADR (decision-record names ADR-0510 cutover-trigger, ADR-0250 build-ahead, D-META:26) as a required front-matter field, so the pending set is queryable and each entry has a named gate. Otherwise build-first-cutover-later becomes build-first-cutover-never, and the corpus carries permanent dual-canon (old bridge ADR + new owned ADR both live).

---

## 6. ASPIRATIONAL-TOOLING RISK (generator not on dev) — the plan's strongest section, but hedged in the wrong place

The plan handles this well operationally (Failure-3, line 193-195): flag every gate-dependent step [ASPIRATIONAL], run manually with verifier stand-in, file a port-to-dev follow-up, never hand-author the masterplan as authority, label the manual artifact "manually generated pending `oya gen masterplan` on dev." The reframe "backfill ≡ clean ADR front-matter, which lands on dev regardless of generator" (line 195) is the right decoupling — the *durable* work (front-matter completeness) is gate-independent. **This is correct and I endorse it.**

**The residual risk is two layers up (FLAG-3):** the *authority document* (decision-record D1) does not carry the aspirational flag, so the canon asserts a capability that does not exist on `dev`. The plan compensates at execution time, but compensation that lives only in a `pending-approval` build plan is itself unreachable from the canon. If the plan is approved and then archived, the "generator is worktree-only" knowledge evaporates. **The aspirational status must be promoted into an ADR** (the masterplan-wiring meta-ADR, L5.1) so it survives the plan's own lifecycle. Second residual: D15 cohesion-by-construction (line 122) and the safety-gate runtime hook (line 148) are *both* aspirational, but the safety-gate (D-SAFETY, line 87) is a *liability/safety commitment* — a manual verifier stand-in for a safety gate is weaker than the plan implies. **Recommend:** the safety-gate ADR (L5.1) ships as a *design+invariant* doc now, but its "hooked into D16's runtime Cedar gate" claim must be marked `runtime-hook-pending` (not asserted-live), because asserting a live safety enforcement that runs only in a worktree is the highest-consequence aspirational-vs-enforced gap in the whole plan.

---

## 7. VERIFY-AT-EACH-STEP ADEQUACY — strong design, three holes

The separate-verifier-lane discipline (§D, §H.2) is well-constructed: per-lane evidence tables, opus for one-way/security, re-read-primary-sources-each-iteration (line 197). Holes:

1. **The verifier inherits the 731/105 vs 831/43 contradiction (FLAG-1).** A verifier re-reading primary sources will read the contradiction and cannot self-resolve which is canon. Verify-at-each-step *requires a consistent primary source*; here it has two. **Must fix before any L2 verification runs.**
2. **No verifier-of-the-verifier for the manual cohesion stand-in (FLAG-4 / Tension C).** The one place the verifier produces a *judgment* rather than a *check* is exactly where the plan provides no second check.
3. **"Zero dangling edges" is checked against the frozen MAP (§D L3 gate, line 135) but the MAP itself has no verifier gate listed.** L1.0 is door:one-way founder-signed (line 57), which is a *governance* gate, not a *correctness* gate. Who verifies the MAP is internally consistent (no two old-ids → same new-id, no new-id reused, consolidation set covers every archived id) *before* L3.3 and L1.3 build on it? A bad MAP poisons both downstream lanes silently. **Fix:** add an explicit "L1.0 MAP self-consistency verifier gate" (bijection check on renumber, surjection check on consolidation) ahead of founder sign-off.

---

## 8. SYNTHESIS — improvements that make the plan stronger without breaking its rulings

- **S1 (recovers most of the D-LANES parallelism lost to Tension A):** Wave-0 and Wave-1 need not be fully serial. Split L1 into **L1-amend** (in-place de-foundry/de-stale of Accepted ADRs that are NOT being re-founded) and **L1-refound** (the ADR-0000+ consolidation of those that ARE). L1-amend can run *concurrently with* L2/L3/L6 in Wave-0 (it is the same class of mechanical fix). Only L1-refound must wait for the clean base. This honors D-LANES (more parallel) without violating D-1 (re-foundation still over clean text), because the files L1-amend touches are by definition the ones L1-refound does *not* archive. Requires S2.

- **S2 (fixes the §4 carve + FLAG missing-source):** Move the **ADR-0000+ consolidation set** out of L1.0's general MAP and freeze it as its own door:one-way primary source *at the very start of Wave-0*, so L2/L3/L1-amend all know which files are throwaway-by-re-foundation and skip them. This converts the floating intra-Wave-0 ordering into an explicit edge: `consolidation-set-freeze → {L2.2, L3.3, L1-amend}`.

- **S3 (closes FLAG-1):** Make `decision-record:107` the single census-of-record (831/43) and land a one-doc-per-PR correction to `00-REST-OF-DOCS-REGISTER.md` §2/footprint *as the first Wave-0 unit*, so every downstream verifier reads one number. Add an L2.0b journeys/personas census-completion (template-first) step (FLAG-5).

- **S4 (closes Tension B + §5 gap):** Make `cutover_trigger:` a required front-matter field on every `superseded-on-cutover` ADR, pointing at ADR-0510/0250. Adds a queryable "pending-cutover" set with named gates → build-first-cutover-later can't silently become cutover-never.

- **S5 (closes FLAG-3 + §6 safety residual):** Promote the aspirational-status into the masterplan-wiring meta-ADR (L5.1) as a first-class deliverable, amend decision-record D1 (supersede) to record generator=worktree-only@2026-06-06, and mark the safety-gate ADR's runtime hook `runtime-hook-pending` rather than asserted-live.

- **S6 (closes §7 hole 3):** Add an L1.0-MAP self-consistency verifier gate (renumber-bijection + consolidation-surjection + no-id-reuse) ahead of the founder sign-off on the MAP.

---

## 9. BOTTOM LINE

The plan's *sequencing logic is correct and well-defended* — the wave order satisfies all three RULED ordering constraints, and the L2-before-L1 edge the review was asked to stress is right (rename-before-reauthor genuinely avoids double-work on immutable docs). The plan is **not wrong about order; it is under-defended about data and shape**:
- it inherited a self-contradictory file census (731/105 vs 831/43) into its most irreversible lane and did not flag the reconciliation (FLAG-1, the strongest antithesis);
- it serialized the waves in a way that silently narrows the founder's D-LANES ruling without naming the tradeoff (Tension A / FLAG-2), recoverable via S1/S2;
- the aspirational generator is hedged in the plan but asserted-as-real in the canon it derives from (FLAG-3), and the safety-gate runtime hook is the highest-consequence instance (§6);
- the intra-Wave-0 ordering (consolidation-set-freeze before per-file rename) is unspecified, making some L2 work potentially throwaway (§4 carve).

Apply S1–S6 (especially S2 and S3 before Wave-0 starts) and the plan is execution-ready. None of these block founder sign-off on the *sequencing decision* (§A); they are pre-Wave-0 conditions on the *base data and primary-source consistency* the lanes will run against.
