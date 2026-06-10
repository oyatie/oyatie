---
doc_class: BominalReconciliation
title: FOUNDER INTERVIEW AGENDA — what to restore from bominal (own-history recovery)
status: ready-to-ask
date: 2026-06-06
premise: >
  oyatie WAS bominal (renamed + migrated; the migration churned/lost context). This is recovery of
  oyatie's OWN history, NOT adoption of a foreign repo and NOT blind recovery. Each item below is a
  FOUNDER CALL: recover-as-ADR vs intentionally-dropped vs fold-into-doc (per canon D-LINEAGE).
inputs:
  - bominal-reconciliation/20-LOST-CONTEXT-REGISTER.md   (35 verified losses: 11 HIGH / 11 MED / 13 LOW)
  - synthesis/decision-record-oyatie-canon.md            (16 ALREADY-RULED D-decisions — NOT re-asked)
  - legacy-recovery/00-RECOVERY-REGISTER.md              (7 .Trash recoveries — NOT duplicated)
scope_discipline: >
  SKIPPED because already ruled in the decision record: maximal-vertical-scope-as-endpoint (D9),
  own-everything ratchet / build-first-cutover-later (D-META/D4), forge=GitHub-now-bespoke-later (D2),
  identity=oya-identity+Zitadel-bridge (D5), Cedar-as-permanent-contract (D6), framekernel-host (D7),
  data-tier-own-all (D4), masterplan-generated-from-ADRs (D1), cloud-as-dogfood (D-LAYER), the flat-
  catalog/no-"Arms" doctrine (D9/E), the ontology rename (E). These are CLOSED — the questions below
  are the OPEN residue the canon did NOT settle.
  SKIPPED because already recovered in .Trash register: KR HR/payroll 8-pack bundle, First Proof
  Slice (SLICE-001), 4-lane closure model, EaaS self-tenant invariant, 9-key-class trust taxonomy,
  billing-deferral rationale, 8-stage delivery chain. Q4/Q7 below CONNECT to these but ask a NEW thing.
how_to_read: >
  Each theme = 1-3 crisp founder questions, each with concrete options + a recommendation. Ordered by
  LEVERAGE (theme 1 = highest). [door:one-way] = expensive/irreversible to revisit; rule carefully.
  This is a TIGHT agenda — highest-value decisions first, not exhaustive. The MED/LOW residue is
  batched at the end (theme 8) for one fast yes/no sweep.
---

# 00 — FOUNDER INTERVIEW AGENDA (bominal → oyatie restoration)

The migration kept the architecture spine and even most far-future verticals (defense/ITAR/public-
sector landed as ADRs). What it genuinely **lost** clusters in three places: (1) named regulator-
grounded **product specs** that exist nowhere (0 grep hits), (2) the **dated, gated build SEQUENCE**
that turned breadth into a buildable plan, (3) **KR-first statutory DNA** depth. The 16 canon rulings
fixed the *endpoints* (own everything, maximal scope, the substrates). They did NOT fix **what to
build, in what order, and how deep into regulation** — that is this interview.

Eight themes, ordered by leverage. Themes 1-5 are the high-leverage calls (mostly the 11 HIGH rows).
Theme 6-7 are structural. Theme 8 batches the MED/LOW residue.

---

## THEME 1 — Build SEQUENCE & the infra-sovereignty ratchet SCHEDULE  [HIGHEST LEVERAGE]

> *Why first:* The canon ruled "maximal scope, **sequenced not cut**" (D9) and "own everything via
> the ratchet" (D-META) but explicitly **deferred the day-0 capacity budget (D8) and never restored
> the dated schedule.** Every other restoration question ("do we build CDSS?") is unanswerable until
> we fix HOW we sequence. This theme converts the open D8/D9 endpoint into an operational order.

**Q1.1 — Do we restore the ADR-0150 infra-sovereignty RATCHET as a dated schedule, or keep it
purely trigger-gated?**  *(bominal had: 2027-Q2 IaC/OpenTofu → Q3 API-Gateway+KMS/Vault → Q4 Mail/
Cache/Event-Streaming → 2028-Q1 DB+Storage → Q2+ shadow candidates, each gated at M0 = contract +
incumbent benchmark + evidence. oyatie: 0 hits for `infrastructure sovereignty`/`M0 gate`/`Stalwart`.)*
- **(A)** Restore the full **dated** quarter-by-quarter schedule as a new ADR + masterplan input (which substrate owned when).
- **(B)** Restore only the **ordered list + M0 evidence-gate shape**, drop the calendar dates (dates are fiction this early).
- **(C)** Keep it purely **trigger-gated** (build-when-proven, no schedule) — the canon D-META rule already covers "cutover when proven."
- **Recommendation: (B).** The *order* and the *M0 gate per substrate* are the load-bearing recovery; calendar dates on a pre-revenue greenfield are theater. Restoring order+gate directly fills the masterplan's missing dated roadmap (canon D1 backfill) and reconciles with the still-owed D8 capacity budget — they answer the same question: build order.  **[door:one-way]** — the build order anchors the whole roadmap; cheap to set now, expensive to re-sequence after teams commit.

**Q1.2 — What is the SHAPE of every vertical milestone?**  *(bominal: a uniform
`research(benchmark matrix) → PHASE M0 gate → doc(operating-contract ADR) → EPIC` shape on all 54
milestones. oyatie has scattered ADRs with no uniform gate shape.)*
- **(A)** Adopt the uniform 4-step milestone shape as the **standard template** for every vertical (gates breadth honestly).
- **(B)** Free-form per vertical.
- **Recommendation: (A).** A single uniform "is this vertical proven enough to build?" gate is exactly the founder's standing *verify-at-each-step / no-phantom-findings* rule applied to roadmap scope. Cheap, high-discipline, reusable.

**Q1.3 — Which ONE substrate is the crown-jewel built FIRST (the deferred D8 capacity budget)?**
*(Canon D-META flagged this as "not forced now." It is now the gating unknown for Theme 1.)*
- **(A)** The **CI/CD substrate (oya-ci)** — it builds everything else (already ratified shape in D3).
- **(B)** The **data-tier owned engine** (highest ADR investment, 514 ADRs of detail).
- **(C)** The **identity endpoint (oya-identity)** (founder-locked already in D5).
- **(D)** Defer again — set the *rule* (one crown jewel at a time) without naming it.
- **Recommendation: (A) oya-ci first.** It is the only substrate whose output is *the ability to build the others*; D3 already ratified its shape, so it is the lowest-marginal-risk first cutover and unblocks the dogfood loop. Name it now so "own everything" stops meaning "build everything at once."  **[door:one-way]**

---

## THEME 2 — Vertical-scope RESTORATION: the named product SPECS lost wholesale

> *Why second:* Canon D9 ruled the verticals are *in scope as ambition*, and a live grep proved most
> far-future verticals landed as ADRs. But **four named, regulator-grounded product specs exist
> NOWHERE (0 grep hits).** These are unreproducible from the flat tree — the founder spent real design
> effort on them. This theme is "which of these specs do we re-author as ADRs vs intentionally drop."
> This is the heart of "NOT blind recovery": each is a discrete recover/drop call.

**Q2.1 — CDSS (clinical decision support / diagnostics): recover the spec or leave as a named-but-
empty folder?**  *(bominal ADR-0137: DDx engine + multimodal imaging/CXR + EKG + red-flag escalation;
self-hosted open-weight medical LLMs (Meditron/Med42), never cloud, PHI-residency; MFDS "Informing"-
tier; **HITL non-negotiable**; processor-not-controller; Wong-2021 Epic-Sepsis as the negative
example. oyatie: `CDSS`=0; only ADR-0332's empty `clinical-decision-support` folder name.)*
- **(A)** **Recover the full spec as a clean ADR** — including the load-bearing **safety gate** (HITL, prospective-validation ship-gate, no-autonomous-Dx).
- **(B)** Recover **only the safety-gate doctrine** now; defer the DDx/imaging engine spec.
- **(C)** Intentionally drop — keep healthcare to EMR/records, no diagnostics.
- **Recommendation: (A).** This is the deepest product+regulatory spec bominal ever produced and the safety gate is **not separable** from the spec — it is the part that makes the vertical legally shippable. Recover whole.  **[door:one-way]** — the no-autonomous-diagnosis / HITL posture is a safety commitment that is very expensive to walk back once code exists.

**Q2.2 — Manufacturing-AI + AMR/facility-robotics: recover the OT-safety specs?**  *(bominal ADR-0143/
0142: 6 mfg capabilities (defect-detection, predictive-maintenance, root-cause, …) on ISA-95/OPC-UA/
OEE/NCR/CAPA, edge inference, **OT no-closed-loop-actuation**; AMR = 3D-SLAM/ROS2/Open-RMF/VDA-5050 +
robotics safety sandbox. oyatie: `defect detection`/`predictive maintenance`/`SLAM`/`ROS2`/`VDA 5050` = 0.)*
- **(A)** Recover **both** specs as ADRs, anchored on the **OT no-actuation safety boundary**.
- **(B)** Recover **Manufacturing-AI now**, defer **AMR/robotics** (physical-safety, further out).
- **(C)** Recover only the **safety-boundary doctrine** (no closed-loop actuation) as a cross-vertical OT invariant; defer the capability specs.
- **Recommendation: (B) + the (C) invariant.** Manufacturing-AI feeds the near-term KR-industrial wedge and the ISA-95 model is reusable; AMR is genuinely later. But adopt the **no-closed-loop-actuation OT invariant NOW** as a standing safety rule (it governs both, plus powergrid — Theme 3).  **[door:one-way]** for the OT-actuation boundary.

**Q2.3 — Contract Bid-Pricing Engine + tenant-configurable optimization platform: recover?**
*(bominal ADR-0115/0145: bid→contract→budget engine with **live labor-rate feed from corp-payroll**
(the moat vs Procore/Deltek) + KR 국가계약법 **낙찰하한율** statutory floor; and a Workflow-Studio
extension where tenants bring data+objectives and the platform supplies RSM/Bayesian-opt/DOE/Pareto/
bandit templates (Foundry-parity self-serve). oyatie: `bid pricing`/`낙찰하한율`/`Bayesian optimization`/
`Pareto` = 0.)*
- **(A)** Recover **both** as ADRs (named GTM wedge + distinctive platform-AI surface).
- **(B)** Recover **bid-pricing only** (concrete KR-corporate wedge, ties to the payroll moat); defer the optimization platform.
- **(C)** Defer both — capability-tier later.
- **Recommendation: (B).** The bid-pricing engine's moat is **live-payroll → bid-pricing**, which is unreproducible and ties directly to the already-recovered KR payroll wedge (.Trash #1) — high synergy, near-term. The optimization platform ("customers encode their company into us") is distinctive but a later self-serve play; capture it as a *named deferred* surface, don't author the full spec yet.

---

## THEME 3 — Regulatory DEPTH for the highest-risk verticals (powergrid / orbital / capital-markets)

> *Why third:* Canon D9 explicitly named **defense + power-grid as the net-new capture** and a grep
> showed the verticals are NAMED in ADRs — but the **OT/safety regulatory DEPTH is the verified gap**
> (`SCADA`/`62443`/`powergrid`/`SGP4` = 0). The vertical exists; the operating contract that makes it
> real does not. This is "how deep into regulation do we commit now vs later."

**Q3.1 — Powergrid / civil-infra: author the NERC-CIP / IEC-62443 / SCADA OT operating-contract now?**
*(This IS the canon-D9 "net-new power-grid" item — partly captured as ADRs, depth still owed.)*
- **(A)** Author the **full OT operating-contract ADR** (NERC-CIP + IEC-62443 + SCADA + safety-critical posture) now.
- **(B)** Author a **thin "OT-depth owed" placeholder** that gates the vertical behind the M0 evidence gate (Theme 1.2) — depth authored when the vertical is sequenced.
- **(C)** Leave as the current named ADR with no OT depth.
- **Recommendation: (B).** Per Theme 1's uniform milestone shape, the regulatory operating-contract is the **M0 doc-gate deliverable** — authoring full NERC-CIP depth before the vertical is sequenced is premature. But a placeholder that *names the owed depth + binds it to the gate* prevents silent loss.

**Q3.2 — Orbital/space-domain + capital-markets: recover the lost specifics or fold?**  *(bominal:
SGP4/SDP4 orbit-prediction + AIS/ADS-B/TLE feeds (geospatial is present; `orbital`/SGP4 = 0); and the
capital-markets **regulated-partner → own-license ladder** + 전자금융업 path (substrate present;
product-arm intent churned).)*
- **(A)** Recover both as ADRs now.
- **(B)** **Fold** both as named-but-deferred: capture orbital's feed-spec (AIS/ADS-B/TLE/SGP4) and the capital-markets **license ladder boundary** (software-over-regulated-partners) as one-paragraph intents in the vertical-coverage map; full spec at M0.
- **Recommendation: (B).** Geospatial and billing/finops substrate already landed; what's lost is *detail*, not the vertical. The single load-bearing recovery is the **"software over regulated partners → own the license later" boundary discipline** — capture that sentence, defer the rest.

---

## THEME 4 — Module → service LINEAGE & naming continuity (verify the rename map)

> *Why fourth:* The migration's decompositions (healthcare→8 svcs, Connect→mail/messenger, mfg→SAP-
> shaped) are mostly **clean and documented** — but a few rename lines are **inferred, not explicit**,
> and a few invariants did not survive the split. This is cheap-to-fix bookkeeping that prevents a
> future session from mis-reading lineage. Lower leverage than 1-3 but a one-way data-integrity gate.

**Q4.1 — Confirm the inferred rename lines + carry the lost invariants.**  *(Three sub-items, one
decision each — answer as a batch.)*
- **(a) Healthcare released-view invariant:** ADR-0332 split healthcare into 8 svcs but the **"patient = released-view projection, NOT a 2nd system-of-record" + per-surface write-authority** invariant didn't carry crisply. → **Recommendation: FOLD** the invariant back into the ADR-0332 decomposition doc. *(Connects to Q2.1 CDSS but is a separate record-boundary rule.)*
- **(b) `medical`→`emr` line:** inferred from SAP-code + README, not an explicit "X→Y" line. → **Recommendation: CONFIRM** the mapping line explicitly.
- **(c) Connect per-context encryption matrix:** the Connect→mail/messenger rename is clean, but the **symmetric Personal-E2EE-user-DEK vs Professional-tenant-DEK + four-eyes** matrix + native-rebuild-of-10-platforms ambition is churned. → **Recommendation: FOLD** the encryption matrix into the comms-product doc (ADR-0311 is thinner); record native-rebuild as deferred ambition.
- **Founder call:** approve the FOLD/CONFIRM batch as-is, or pull any sub-item up to a full recovery ADR?  **[door:one-way]** for (a) the record-boundary invariant (security-relevant).

---

## THEME 5 — KR-first STATUTORY DNA (the depth that motivated every vertical)

> *Why fifth:* The .Trash register already recovered the KR HR/payroll 8-pack BUNDLE (#1). This theme
> is the **layer underneath** it that is still lost: the **typed 8-class employment enum** and the
> **per-vertical statute grounding**. The canon did not rule on statute depth. This is the single most
> churned *context* (per the register headline), but it FEEDS the already-recovered packs rather than
> standing alone — hence theme 5, not theme 1.

**Q5.1 — Restore the typed KR 8-class employment classification enum?**  *(bominal ADR-0126/0127:
`EmploymentClassification` enum (정규직/계약직/단시간/파견/도급/프리랜서/인턴/임원) driving payroll/
4대보험/leave/52시간/severance/withholding. oyatie: enum + 정규직/계약직/파견/52시간 = 0 hits.)*
- **(A)** **Recover as an ADR** — the typed enum is the spine the .Trash payroll packs (#1) hang off.
- **(B)** Fold the enum into the existing KR regional-pack doc as a data-model detail.
- **Recommendation: (A).** This typed enum *drives the entire payroll wedge* (4대보험/52h/severance all branch on it) and reinforces the already-recovered packs — recovering it makes .Trash #1 actually buildable. Not optional context; it's the model.

**Q5.2 — Adopt "KR-home-market-FIRST" as the explicit posture, and re-thread per-vertical statute
citations?**  *(bominal grounded every vertical in 근로기준법/PIPA/통신비밀보호법/의료법/국가계약법/
KCMVP/52시간제. oyatie has compliance packs + canon D11 restores KCMVP/KISA, but the home-market-first
posture + per-vertical statute grounding is the most churned context.)*
- **(A)** Declare **KR-home-market-FIRST** as an explicit sequencing posture + re-thread statute citations per vertical (one pass).
- **(B)** Keep statutes in the compliance pack only; no per-vertical re-threading.
- **Recommendation: (A).** The statute grounding is the **DNA that motivated the whole product set** — without it the verticals read as generic SaaS. Re-threading per vertical also makes Theme 1's M0 gate concrete (the statute IS the operating contract). One pass, high context-recovery.  **[door:one-way]** for the home-market-first posture (it orders the whole roadmap — couples to Theme 1).

---

## THEME 6 — Portfolio compounding SPINE & first-customer launch target

> *Why sixth:* Canon ruled the portfolio plane exists and D-LAYER ruled the dogfood loop, but the
> **specific ordered build-spine + block-edges** and the **concrete first-paid-customer target** are
> strategy detail not visibly carried. Connects to .Trash #2 (First Proof Slice) without duplicating
> it: #2 defines the *seam*; this defines the *sellable-customer milestone*.

**Q6.1 — Restore the portfolio build-spine + block-edges, and the M3 first-customer target?**
*(bominal: spine `platform→payments→corporate→messaging→documents→notify→healthcare-billing→
intelligence` with block-edges (trust ⊥ healthcare-billing@L4, ⊥ payments@L4) + per-arm investment
levels; M3 target = **≥1 paid KR group (~3000 wage employees) closes real payroll before M3 done**,
payroll-first public claim.)*
- **(A)** Restore **both** the ordered spine+block-edges AND the M3 paid-customer target.
- **(B)** Restore only the **M3 first-customer target** (fold with .Trash #2 First Proof Slice into one "first buildable+sellable slice" artifact); let Theme 1's sequence subsume the spine.
- **(C)** Defer both to the masterplan/capacity-budget work.
- **Recommendation: (B).** The build-spine overlaps Theme 1's sequence (don't restore twice). The **M3 paid-customer target** is the missing concrete "what do we sell FIRST" that .Trash #2's seam definition lacks — folding them gives one crisp first-slice artifact. Keep the block-edges (trust ⊥ payments/healthcare-billing) as a recovered *invariant*.

---

## THEME 7 — Safety-gate doctrines as a CROSS-VERTICAL invariant set

> *Why seventh:* The individual safety gates appear inside Theme 2/3 specs, but bominal treated them
> as a **family** (HITL · OT-no-actuation · biometric-default-off · no-autonomous-lethal-use). The
> canon's D16 autonomy-ceiling (runtime Cedar gate, owned by `governance`) is the natural HOME for
> them. This theme asks whether to unify them as one governance-owned invariant set rather than
> scattering them per spec.

**Q7.1 — Unify the safety gates as one `governance`-owned cross-vertical invariant set?**
- **(A)** **Yes** — author one safety-gate invariant ADR under `governance` (HITL for clinical, no-closed-loop-actuation for OT/mfg/AMR, biometric-default-off + 5-stage escalation for CCTV/vision, no-autonomous-lethal for defense), referenced by each vertical spec. Hooks into D16's runtime Cedar gate.
- **(B)** **No** — keep each gate inside its own vertical spec (Theme 2/3).
- **Recommendation: (A).** The gates are the *load-bearing, legally-protective* part of each spec and they share one enforcement mechanism (D16's runtime Cedar gate, owned by governance). Unifying them makes the safety posture auditable in one place and prevents a future vertical from shipping without a gate.  **[door:one-way]** — these are safety/liability commitments; centralizing them is the responsible default.

---

## THEME 8 — BATCH SWEEP: the MED/LOW residue (one fast pass)

> *Why last:* These are low-intent-loss FOLD/CONFIRM items. Ask as a single yes-to-recommendations
> sweep; pull anything up only if the founder flags it. None are door:one-way.

| # | Item (bominal ref) | Recommendation | Founder: accept / pull-up / drop? |
|---|---|---|---|
| 8.1 | **CCTV 5-stage escalating-risk vision pipeline + biometric-default-off** (ADR-0141) | FOLD into the vision-product operating contract (biometric governance already present; restate the 5-stage + default-off). *Gate goes to Theme 7.* | |
| 8.2 | **Email/messenger MINING product + person-pillar HARD exclusion zone** (ADR-0136; 통신비밀보호법) | FOLD into comms-intelligence + data-ownership-pillar doc; the exclusion-zone is a security invariant worth restating. | |
| 8.3 | **AI-surfaces catalog (~17 intelligence domains, OR/MIP/graph-before-ML)** (ADR-0144) | FOLD into the intelligence/data-AI-governance doc as a capability catalog. | |
| 8.4 | **`insurance` / `banking` owning-service disposition** (18/14 ADR hits, no dir, no retirement ADR) | CONFIRM in vertical-map: capability-only by intent, or owe a service dir? Record the disposition (don't silently leave dir-less). | |
| 8.5 | **`ats` (applicant tracking)** (3 ADR hits, no owner) | FOLD: name ATS as an explicit `hr` capability-tier or give it an owner. | |
| 8.6 | **`security` as a sellable customer-facing product** (vs substrate-only) (ADR-0072) | CONFIRM whether the *sold* security product survives or is substrate-only; record. | |
| 8.7 | **Consumer hospitality/lifestyle modules** (dining/cellar/pos/retail/fashion/career — PARKED pre-L1) | CONFIRM the PARKED list survives into the masterplan roadmap (worth-documenting⇒reachable); no ADR. | |
| 8.8 | **Bominal Law / Finance / Train** named-but-deferred product tracks | FOLD into roadmap as named-deferred tracks; low intent-loss. | |
| 8.9 | **PortOne→Toss PG-rail + 전자금융업 license ladder** (marketplace payments) | FOLD into marketplace/payments ADR: restore PG-rail specifics + license-ladder boundary. *(Couples to Q3.2's "software over regulated partners.")* | |
| 8.10 | **User/Org profiling architecture** (UserProfile + OrgProfile + health_score) (ADR-0138/0139) | Capture as a marketplace/fraud-feeding capability IF still in scope; else drop. | |
| 8.11 | **Kantara consent-receipt standard re-cite; ontology 5-differentiator restatement; data-ownership-pillar worker-rights-override strengthen** | CONFIRM-only trio — these invariants survived (verified present); optionally strengthen. NOT losses. | |

---

## Coverage check (every register row routed)

- **11 HIGH rows** → Themes 1 (ratchet-schedule, 54-milestone-sequence), 2 (CDSS, Mfg-AI, AMR, bid-
  pricing, optimization-platform), 3 (powergrid OT-depth), 5 (8-class enum, statute DNA), 7 (safety
  gates). **All 11 surfaced as primary questions.**
- **11 MED rows** → Themes 2/3 (insurance·banking confirm folded to 8.4; orbital, capital-markets,
  portfolio-spine, M3-target), 4 (healthcare invariant, Connect encryption, six-arm wedge-map→Theme 2),
  6, 8 (CCTV, comms-mining, PG-rails).
- **13 LOW rows** → Theme 4 (rename confirms), 8 (ats, security-product, parked-modules, Law/Finance/
  Train, profiling, AI-catalog, Kantara/ontology/pillar confirms).
- **SKIPPED (canon-ruled):** maximal-scope-endpoint, ownership ratchet, build-first-cutover, the
  decompositions-are-improvements (Section G "do NOT restore"), flat-catalog/ontology renames.
- **NOT duplicated (.Trash-recovered):** KR payroll packs (Theme 5 feeds, doesn't re-author the
  bundle), First Proof Slice (Theme 6 folds-with, doesn't re-define the seam), 4-lane closure, EaaS,
  9-key-classes, billing-deferral, 8-stage chain.

## Door:one-way items to rule most carefully
Q1.1 (build-order anchor) · Q1.3 (crown-jewel-first) · Q2.1 (CDSS HITL/no-autonomous-Dx safety
commitment) · Q2.2 (OT no-actuation boundary) · Q4.1a (healthcare record-boundary invariant) · Q5.2
(KR-home-market-first posture) · Q7.1 (centralized safety-gate set). These set commitments that are
expensive or liability-laden to reverse — rule with the founder's verify-at-each-step rigor.
