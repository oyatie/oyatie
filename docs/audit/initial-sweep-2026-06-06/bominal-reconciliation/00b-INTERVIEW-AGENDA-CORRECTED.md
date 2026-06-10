---
doc_class: BominalReconciliation
title: FOUNDER INTERVIEW AGENDA (CORRECTED) — bominal restoration with verified severities
status: ready-to-ask
date: 2026-06-06
supersedes: 00-INTERVIEW-AGENDA.md
correction_basis: >
  Independent verification pass against ADR-LEGACY-REGRESSION-MAPPING.md (127-row
  migration-coverage map), fresh grep across ALL of source/docs/ (not just
  docs/decisions/), user-journeys/, PRDs, regional-packs, and team charters.
  See _VERIFICATION-CORRECTION.md for the full per-item evidence table.
key_corrections:
  - "Q2.1 CDSS: NOT absent — ADR-0332 C.6 has full service charter. Owed: HITL/MFDS safety-gate depth."
  - "Q2.2 Mfg-AI/AMR: NOT absent — ADR-0027 EXPANDED + j157 journey + vertical-industrial CHARTER. Owed: capability spec + OT invariant."
  - "Q2.3 Bid-pricing: NOT absent — ADR-0115 PARTIAL + j160 journey. Owed: KR 낙찰하한율 statutory floor."
  - "Q5.1 KR employment: NOT absent — KR domain EXPANDED (payroll PRD + KR pack). Owed: typed 8-class enum as data artifact."
  - "Q5.2 KR statutes: NOT absent — KR pack and ADR-0064 have statute citations. Owed: home-market-FIRST posture declaration."
  - "Q6.1 M3 customer target: NOT absent — present in payroll PRD + ADR-0060 + kr.md. Owed: build-spine + block-edges."
  - "8.10 UserProfile/OrgProfile: FULL in regression map — confirm-only, no recovery."
scope_discipline: >
  Same scope as 00-INTERVIEW-AGENDA.md — skips the 16 canon-ruled D-decisions and
  7 .Trash-recovered items. Reframes product-spec questions from "recover lost spec"
  to their TRUE action per verified status.
how_to_read: >
  [door:one-way] = expensive/irreversible to revisit; rule carefully.
  Corrected severity labels: HIGH-DEPTH-OWED (genuine depth gap on tracked surface),
  HIGH-GENUINELY-ABSENT (truly nowhere), TRACKED-PARTIAL (promote, not recover),
  CONFIRM-ONLY (already present — just confirm).
  Ordered by leverage. Themes 1–3 = highest.
---

# 00b — FOUNDER INTERVIEW AGENDA (CORRECTED)

The migration kept the architecture spine and most far-future verticals. A fresh
verification across ALL of source/docs — including user-journeys/, PRDs,
regional-packs, team charters, and the 127-row ADR-LEGACY-REGRESSION-MAPPING.md —
shows the original agenda **over-stated 10 items as "absent/0 hits"** when they are
already TRACKED-PARTIAL or have partial instantiation in non-ADR surfaces. The
genuine losses are narrower and more specific than the original framing.

**What changes:** product-spec questions are re-framed from "recover lost spec" to
their true action: promote an existing PARTIAL ADR to authored depth, or author the
specific missing OT-regulatory or statutory detail. The **door:one-way items, the
OT/regulatory depth gaps, the infra-sovereignty schedule, the typed KR enum, D8,
and the safety-gate unification** are preserved at full weight — these are real.

---

## THEME 1 — Build SEQUENCE & infra-sovereignty ratchet SCHEDULE  [HIGHEST LEVERAGE]

Status: **correctly identified in original agenda — no correction needed.**

**Q1.1 — Restore the ordered substrate list + M0 evidence-gate shape for the
infra-sovereignty ratchet?**

Verification confirms: the ratchet *principle* is in canon D-META and ADR-0014
(EXPANDED with air-gap-first IaC posture). The *ordered which-substrate-when
schedule with M0 = contract + incumbent-benchmark + evidence per replacement* is
genuinely absent as an artifact across all of source/docs.

- **(A)** Restore the full **dated** quarter-by-quarter schedule (which substrate
  owned in which quarter).
- **(B)** Restore the **ordered list + M0 evidence-gate shape**, drop calendar dates
  (dates are fiction this early; the order and gate are load-bearing).
- **(C)** Keep trigger-gated only (build-when-proven) — canon D-META already covers
  the endpoint.
- **Recommendation: (B).** The order + gate fill the masterplan's missing dated
  roadmap (canon D1 backfill) and reconcile the still-owed D8 capacity budget.
  **[door:one-way]** — build order anchors the whole roadmap.

**Q1.2 — Adopt the uniform 4-step vertical milestone shape as the standard template?**

Verification confirms: ROADMAP.md has per-wave gate criteria; the per-vertical
uniform `research(benchmark matrix) → M0-gate → doc(operating-contract ADR) → EPIC`
template is not codified as a reusable standard.

- **(A)** Adopt as the **standard template** for every vertical.
- **(B)** Free-form per vertical.
- **Recommendation: (A).** Cheap, high-discipline, the gating discipline for "do we
  build this vertical?" — the founder's verify-at-each-step rule applied to scope.

**Q1.3 — Name the D8 crown-jewel substrate built first?**

Verified genuinely absent as a named decision in any artifact.

- **(A)** oya-ci first (CI/CD substrate; D3 already ratified its shape).
- **(B)** Data-tier owned engine (highest ADR investment).
- **(C)** oya-identity (founder-locked in D5).
- **(D)** Defer — set the *rule* (one crown jewel at a time) without naming it.
- **Recommendation: (A) oya-ci first.** It is the only substrate whose output is
  *the ability to build the others*.  **[door:one-way]**

---

## THEME 2 — Vertical-scope: DEPTH owed on tracked surfaces  [HIGH LEVERAGE]

> **Correction to original framing:** the original agenda said these specs "exist
> NOWHERE (0 grep hits)" and called for full spec recovery. Verification shows they
> are TRACKED-PARTIAL or EXPANDED in the regression map with structural presence in
> source/docs. The true action is **authoring the specific missing depth on existing
> tracked surfaces**, not recovering from zero.

**Q2.1 — CDSS: author the HITL/MFDS safety-gate depth into ADR-0332 C.6?**

*Corrected framing:* ADR-0332 C.6 `microservices/clinical-decision-support/` exists
with full service charter, gRPC bindings, EU MDR regulatory refs, and competitor
benchmarks (UpToDate/Lexicomp/Micromedex). GLOSSARY.md §7 cites CDSS. The regression
map has ADR-0033 PARTIAL + ADR-0016 PARTIAL (clinical released-view contract not yet
authored — already in the council-attention list).

**What is genuinely absent** (depth-owed, not the whole spec): the bominal safety-gate
doctrine that makes the vertical legally shippable:
- HITL non-negotiable (clinician-override on every BPA; no autonomous diagnosis)
- MFDS "Informing"-tier classification (not "Replacing" — the SaMD regulatory posture)
- Prospective-validation ship-gate (clinical study before production rollout)
- Processor-not-controller boundary (data-processing vs clinical decision authority)
- Self-hosted open-weight medical LLMs (Meditron/Med42, never cloud, PHI-residency)
- Wong-2021 Epic-Sepsis as the negative example (what "autonomous" looks like wrong)

Options:
- **(A)** Author the full safety-gate doctrine as a standalone safety ADR, then fold
  into ADR-0332 C.6.
- **(B)** Fold the safety-gate doctrine directly into ADR-0332 C.6 (no new ADR;
  amend the existing service charter).
- **(C)** Defer — clinical vertical is later; accept ADR-0033 PARTIAL status.
- **Recommendation: (A).** The safety gate is a cross-vertical invariant (Theme 7
  unification); authoring it standalone makes it reusable. ADR-0016 PARTIAL should
  be promoted in the same pass (released-view-as-projection + per-surface
  write-authority invariant).  **[door:one-way]** — HITL/no-autonomous-Dx is a
  safety commitment that is very expensive to walk back.

**Q2.2 — Manufacturing-AI + AMR: author the OT-safety specs and adopt the OT
no-actuation boundary as a cross-vertical invariant?**

*Corrected framing:* Manufacturing-AI is EXPANDED in the regression map (ADR-0033
vertical-industrial + ADR-0035 workflow + ADR-0026 in-house model substrate). ADR-0027
robotics-vision-speech is EXPANDED (robotics substrate centralized). j157 user journey
(defect detection recall, ISO-9001, EU-GPSR) instantiates defect detection as a
real scenario. vertical-industrial/CHARTER.md explicitly names: MES, OEE, ISA-95,
OPC UA, SCADA historian, OT/IT boundary safety controls. ADR-0027 has `Sc1Informational
// observe-only; no actuation` as a SafetyClass enum value.

**What is genuinely absent** (depth-owed):
- The 6-capability manufacturing-AI spec (defect/maintenance/fault/accountability/
  workflow/financial-optimization) with ISA-95/OPC-UA/OEE/NCR/CAPA data model,
  edge inference posture, and KR competitors (SUALAB, MAKINAROCKS)
- AMR-specific spec: 3D-SLAM mapping, ROS2, Open-RMF, VDA-5050 (SLAM=0, ROS2=0,
  VDA-5050=0 confirmed across all source/docs)
- The OT no-closed-loop-actuation boundary as a **named cross-vertical invariant**
  (it exists as a SafetyClass enum value in ADR-0027 but is not declared as a
  cross-vertical rule)

Options:
- **(A)** Author both specs as vertical ADRs + promote Sc1Informational to an
  explicit named cross-vertical OT invariant.
- **(B)** Author Manufacturing-AI now (near-term KR-industrial wedge); defer AMR
  (physical-safety, further out). Promote the OT invariant now (it governs both
  + powergrid).
- **(C)** Author only the OT invariant; defer both capability specs.
- **Recommendation: (B) + the OT invariant.** ISA-95 reusable for near-term wedge;
  AMR is later. OT invariant belongs in Theme 7's safety-gate unification pass.
  **[door:one-way]** for the OT no-actuation boundary.

**Q2.3 — Bid-Pricing Engine: promote ADR-0115 PARTIAL by adding the KR statutory
floor (낙찰하한율) and confirming the payroll-moat coupling?**

*Corrected framing:* ADR-0115 bid-pricing-engine is PARTIAL in the regression map
("vertical-specific module (industrial/construction); flag for council
vertical-industrial"). j160 user journey (cross-tenant bid lifecycle, structured
ČSN-EN-13549 cost model, 8-state bid+onboard lifecycle, Cedar-gated transitions)
instantiates a real bid scenario for the Czech market. The regression map notes it
"may stay PARTIAL until that vertical is prioritized."

**What is genuinely absent** (depth-owed):
- KR 국가계약법 낙찰하한율 statutory bid floor (the moat-defining detail: 0 hits
  across all source/docs)
- Explicit live-payroll → bid-pricing moat coupling (payroll labor-rate → bid
  engine as the unreproducible competitive differentiator vs Procore/Deltek)

**For the tenant-configurable optimization platform** (ADR-0145 EXPANDED in regression
map): the concept is captured but the RSM/Bayesian-opt/DOE/Pareto/bandit template spec
is not authored anywhere. This is GENUINELY-ABSENT as a spec.

Options for bid-pricing:
- **(A)** Promote ADR-0115 PARTIAL to authored depth: add the 낙찰하한율 floor + the
  payroll-moat coupling as the KR-industrial go-to-market wedge.
- **(B)** Promote ADR-0115 with only the payroll-moat coupling; defer 낙찰하한율
  until the vertical is sequenced per Theme 1's M0 gate.
- **(C)** Keep ADR-0115 PARTIAL — accept deferred status.

Options for optimization platform:
- **(X)** Author the tenant-configurable optimization spec now (RSM/Bayesian-opt/DOE/
  Pareto/bandit templates).
- **(Y)** Capture as a named-deferred surface (one-paragraph intent); full spec at M0.
- **Recommendation: (A) for bid-pricing (KR-industrial wedge is near-term); (Y) for
  optimization platform (distinctive but later self-serve play).**

---

## THEME 3 — Regulatory DEPTH for highest-risk verticals (powergrid / orbital / capital-markets)

> **Correction to original framing:** the original stated these verticals are "partly
> captured as ADRs, depth still owed." Verification confirms this is exactly correct —
> the verticals are named; the OT operating-contract depth is the verified gap. No
> over-statement here.

**Q3.1 — Powergrid / civil-infra: author the NERC-CIP / IEC-62443 / SCADA OT
operating-contract?**

Verification: `NERC-CIP`=2 hits (one Cedar policy expression, one retention-schedule
rule — not an operating-contract ADR). `IEC-62443`=0. `SCADA historian` named in
vertical-industrial CHARTER.md but no OT network-isolation architecture ADR.

- **(A)** Author the full OT operating-contract ADR (NERC-CIP + IEC-62443 zone/conduit
  model + SCADA + safety-critical posture) now.
- **(B)** Author a thin "OT-depth owed" placeholder that names the owed depth and
  binds it to the Theme 1 M0 evidence gate — depth authored when the vertical is
  sequenced.
- **(C)** Leave as the current named ADR with no OT depth.
- **Recommendation: (B).** Per Theme 1's uniform milestone shape, the OT
  operating-contract is the M0 doc-gate deliverable. Placeholder prevents silent loss.

**Q3.2 — Orbital/space-domain + capital-markets: recover or fold?**

Orbital: `orbital`=0, `SGP4`=0 across all source/docs — genuinely absent specifics.
Capital-markets: `전자금융업` is in ADR-0064 KR pack scope; `PortOne`=0; license
ladder not authored.

- **(A)** Recover both as ADRs now.
- **(B)** Fold both as named-but-deferred: capture orbital's feed-spec (AIS/ADS-B/
  TLE/SGP4) and the capital-markets **license ladder boundary** (software-over-
  regulated-partners → own the license later) as one-paragraph intents in the
  vertical-coverage map; full spec at M0.
- **Recommendation: (B).** Geospatial substrate landed; what's lost is detail. The
  "software over regulated partners → own the license later" discipline is the
  single load-bearing recovery; 전자금융업 is already in the KR pack scope.

---

## THEME 4 — Module → service LINEAGE & naming continuity

> **Correction to original framing:** Q4.1a (healthcare released-view invariant) is
> already in the council-attention list as ADR-0016 PARTIAL — it is a tracked gap,
> not a new lineage fault. The correct action is promoting ADR-0016 PARTIAL.

**Q4.1 — Confirm the inferred rename lines + carry the lost invariants.**

- **(a) Healthcare released-view invariant:** ADR-0016 PARTIAL in regression map —
  "clinical released-view contract not yet authored." `released-view`/`released_view`
  = 0 hits in ADR-0332.
  → **PROMOTE ADR-0016 PARTIAL:** fold the `patient = released-view projection, NOT
  a 2nd system-of-record + per-surface write-authority` invariant into ADR-0332 C.6.
  This is the same pass as Q2.1 CDSS depth authoring.  **[door:one-way]** (security-
  relevant record boundary).
- **(b) `medical`→`emr` line:** inferred from SAP-code + README, documented in
  ADR-0332 structure.
  → **CONFIRM** — clean; no recovery needed.
- **(c) Connect per-context encryption matrix:** per-tenant DEK exists in ADR-0043/
  0140/0299; `Personal-E2EE-user-DEK` vs `Professional-tenant-DEK` per-channel
  differentiation = 0 hits.
  → **DEPTH-OWED:** fold the per-context channel DEK split into the comms-product
  doc; record native-rebuild ambition as deferred.

**Founder call:** approve the PROMOTE(a) / CONFIRM(b) / FOLD(c) batch as-is, or
pull any sub-item to a full recovery ADR?

---

## THEME 5 — KR-first STATUTORY DNA

> **Correction to original framing:** the KR employment domain is EXPANDED in the
> regression map; the payroll PRD and KR regional pack contain 4대보험/주52시간/
> 통상임금 depth. The two specific missing artifacts are (1) the typed 8-class
> enum and (2) the explicit home-market-FIRST posture declaration.

**Q5.1 — Author the typed KR 8-class employment classification enum?**

Verification: `정규직`=0, `계약직`=0, `파견`=0, `EmploymentClassification`=0 as a
typed enum across all source/docs. Surrounding KR compliance domain is EXPANDED
(ADR-0033 KR-employment EXPANDED; payroll PRD has 4대보험 EDI in detail; KR pack
has 주52시간/통상임금/퇴직금). The enum is the specific data-model artifact missing.

- **(A)** **Author the typed enum as an ADR** — `EmploymentClassification` enum
  (정규직/계약직/단시간/파견/도급/프리랜서/인턴/임원) with branching rules per class
  (4대보험 / 52h / severance / withholding) that make the recovered .Trash payroll
  packs actually buildable.
- **(B)** Fold the enum into the existing KR regional-pack doc as a data-model detail.
- **Recommendation: (A).** The typed enum is the spine the payroll packs hang off;
  it drives conditional payroll logic across all 8 classes. Not optional context.

**Q5.2 — Declare KR-home-market-FIRST as an explicit sequencing posture?**

Verification: statute citations exist in KR PACK.md (근로기준법/4대보험/주52시간) and
ADR-0064 (전자금융업/PIPA/산업안전보호법 as KR-pack-scoped). The home-market-FIRST
posture as a named sequencing rule is 0-hit. The *statutes themselves do not need
re-threading* — they are present; only the posture declaration is missing.

- **(A)** Declare **KR-home-market-FIRST** as an explicit sequencing posture in the
  KR regional pack (one sentence: "KR is the first-priority market; all vertical
  sequencing prioritizes KR statutory compliance before other regions"). Per-vertical
  statute citations are already present and do not need to be re-threaded.
- **(B)** Keep statutes in the compliance pack only; no posture declaration.
- **Recommendation: (A).** The posture declaration couples to Theme 1's M0 gate
  (the statute IS the operating contract for KR verticals) and makes the sequencing
  order concrete. One sentence; high context-recovery.  **[door:one-way]** (it
  orders the whole roadmap).

---

## THEME 6 — Portfolio compounding SPINE & first-customer launch target

> **Correction to original framing:** the M3 first-customer target is NOT absent.
> prds/payroll.md, ADR-0060 (bominal-inheritance-precedence.md:90), and
> localization-packs/kr.md all reference "M03 KR group payroll launch = oyatie M03
> first-paying-tenant target." The regression map's DROPPED-WITH-REASON for ADR-0050
> retired the *wave vocabulary*, not the commercial milestone.
> What is genuinely absent: the ordered portfolio build-spine + block-edges.

**Q6.1 — Restore the portfolio build-spine + block-edges; confirm the M3 first-
customer target is live?**

*Corrected framing:*
- **M3 first-customer target:** PRESENT in payroll PRD + ADR-0060 + kr.md. Action
  is to **confirm it is the canonical first-slice target** and fold it with .Trash
  #2 (First Proof Slice) into one crisp artifact. Not recovery.
- **Ordered build-spine** (`platform→payments→corporate→messaging→…`) + **block-edges**
  (trust ⊥ healthcare-billing@L4, ⊥ payments@L4): **genuinely absent** (0-hit).

- **(A)** Restore **both** the ordered spine+block-edges AND confirm+formalize the
  M3 first-customer target.
- **(B)** Restore only the **block-edges** as recovered invariants; let Theme 1's
  substrate sequence subsume the spine; confirm the M3 target as-is.
- **(C)** Defer both.
- **Recommendation: (B).** The build-spine overlaps Theme 1's sequence (don't restore
  twice). The block-edges (what CANNOT be built before what) are the distinct recovery.
  Confirm M3 target as already present; formalize in one "first buildable+sellable
  slice" artifact with .Trash #2.

---

## THEME 7 — Safety-gate doctrines as CROSS-VERTICAL invariant set  [door:one-way]

> Status: **correctly identified — no correction needed.** Individual gates exist
> (ADR-0027 Sc1Informational no-actuation class; autonomy-ceiling ADR-0022 EXPANDED;
> defense lethal-use guard in 37 ADRs). The unified governance-owned invariant set
> artifact does not exist.

**Q7.1 — Author one governance-owned cross-vertical safety-gate invariant set?**

The four gates and their enforcement hooks:
- **HITL for clinical** (no autonomous diagnosis; every BPA requires clinician override)
- **OT no-closed-loop-actuation** (for manufacturing/AMR/powergrid — promote from
  ADR-0027 Sc1Informational SafetyClass to a named cross-vertical rule)
- **Biometric default-off + 5-stage escalating-risk pipeline** (for CCTV/vision —
  stages 1-2 edge, 3-5 central; facial/identity only with jurisdiction-gated consent)
- **No-autonomous-lethal-use** (for defense — already in 37 defense ADRs; unify here)

All four hook into D16's runtime Cedar gate (owned by `governance`).

- **(A)** **Yes** — author one safety-gate invariant ADR under `governance`, referenced
  by each vertical spec (Q2.1 CDSS, Q2.2 mfg/AMR, Q3.1 powergrid, Theme 8 CCTV).
- **(B)** **No** — keep each gate inside its own vertical spec.
- **Recommendation: (A).** Gates share one enforcement mechanism (D16 Cedar gate);
  unifying makes safety posture auditable in one place.  **[door:one-way]** — these
  are safety/liability commitments.

---

## THEME 8 — BATCH SWEEP: MED/LOW residue (corrected)

> **Correction:** 8.10 (UserProfile/OrgProfile) is FULL in the regression map
> (ADR-0138/0139 FULL via OG model + DUB). Removed from recovery list; added to
> confirm-only column.

| # | Item | Corrected status | Recommendation | Founder: accept / pull-up / drop? |
|---|---|---|---|---|
| 8.1 | **CCTV 5-stage escalating-risk vision pipeline + biometric-default-off** (ADR-0141) | TRACKED-PARTIAL (ADR-0027 EXPANDED; biometric=15 ADR hits; 5-stage pipeline not authored as product op-contract) | FOLD into the vision-product operating contract + promote to Theme 7 safety gate. | |
| 8.2 | **Email/messenger MINING product + person-pillar HARD exclusion zone** (ADR-0136; 통신비밀보호법) | TRACKED-PARTIAL (DUB EXPANDED covers exclusion semantics; mining product + explicit 통신비밀보호법 exclusion-zone not authored as product intent) | FOLD into comms-intelligence + data-ownership-pillar doc; restate exclusion-zone as security invariant. | |
| 8.3 | **AI-surfaces catalog (~17 intelligence domains, OR/MIP/graph-before-ML)** (ADR-0144) | TRACKED-PARTIAL (ADR-0026 PARTIAL in regression map — already in council-attention list: "confirm enumeration no longer needed") | FOLD into intelligence/data-AI-governance doc as capability catalog. Council sign-off already flagged. | |
| 8.4 | **`insurance` / `banking` owning-service disposition** (18/14 ADR hits, no dir, no retirement ADR) | TRACKED-PARTIAL (present as ERP/fintech capabilities; disposition not recorded) | CONFIRM in vertical-map: capability-only by intent, or owe a service dir? Record disposition. | |
| 8.5 | **`ats` (applicant tracking)** (3 ADR hits, no owner) | TRACKED-PARTIAL (referenced; no enumerated owner) | FOLD: name ATS as explicit `hr` capability-tier or give it an owner. | |
| 8.6 | **`security` as a sellable customer-facing product** (vs substrate-only) (ADR-0072) | GENUINELY-ABSENT as a product decision | CONFIRM whether the *sold* security product survives or is substrate-only; record. | |
| 8.7 | **Consumer hospitality/lifestyle modules** (dining/cellar/pos/retail/fashion/career — PARKED pre-L1) | DROPPED-WITH-REASON (intentionally PARKED under D9 "scope-OUT stays VISIBLE") | CONFIRM the parked list survives into the masterplan roadmap; no ADR needed. | |
| 8.8 | **Bominal Law / Finance / Train** named-but-deferred product tracks | GENUINELY-ABSENT as named deferred tracks | FOLD into roadmap as named-deferred tracks; low intent-loss. | |
| 8.9 | **PortOne→Toss PG-rail + 전자금융업 license ladder** (marketplace payments) | TRACKED-PARTIAL (ADR-0135 marketplace PARTIAL; 전자금융업 in KR pack scope; PortOne=0; license ladder not authored) | FOLD into marketplace/payments ADR: restore PG-rail specifics + license-ladder boundary (couples to Q3.2 "software over regulated partners"). | |
| 8.10 | **User/Org profiling architecture** (UserProfile + OrgProfile) | TRACKED-FULL (ADR-0138/0139 FULL in regression map — OG model + DUB fully capture both) | CONFIRM-ONLY. No recovery. The agenda's LOW rating was correct; the item should not appear as a recovery candidate. | |
| 8.11 | **Kantara consent-receipt standard re-cite; ontology 5-differentiator restatement; data-ownership-pillar worker-rights-override strengthen** | TRACKED-FULL (persona-tier=12, ownership-pillar present; Kantara=0 but a re-cite only) | CONFIRM-only trio — these invariants survived. Optionally re-cite Kantara and restate 5 differentiators. NOT losses. | |

---

## Coverage check (all register rows routed — corrected)

- **11 HIGH rows** → Themes 1 (Q1.1/Q1.2 ratchet-sequence; Q1.3 D8 crown-jewel),
  2 (Q2.1 CDSS safety-gate depth; Q2.2 mfg-AI spec + OT invariant; Q2.3
  bid-pricing KR depth), 3 (Q3.1 powergrid OT-contract), 5 (Q5.1 typed enum;
  Q5.2 posture declaration), 7 (Q7.1 safety-gate unification). **All 11 surfaced.**
- **11 MED rows** → Themes 2/3 (insurance/banking confirm → 8.4; orbital Q3.2;
  capital-markets Q3.2; portfolio-spine + block-edges Q6.1), 4 (healthcare invariant
  → ADR-0016 PARTIAL promotion; Connect encryption Q4.1c; six-arm wedge-map),
  6 (M3 target confirm), 8 (CCTV 8.1; comms-mining 8.2; PG-rails 8.9).
- **13 LOW rows** → Theme 4 (rename confirms), 8 (ats, security-product, parked-
  modules, Law/Finance/Train, AI-catalog, Kantara/ontology/pillar confirms, 8.10
  UserProfile CONFIRM-ONLY).
- **SKIPPED (canon-ruled):** same as original agenda.
- **NOT duplicated (.Trash-recovered):** same as original agenda.

---

## Door:one-way items (unchanged from original — correctly identified)

Q1.1 (build-order anchor) · Q1.3 (crown-jewel-first) · Q2.1 CDSS HITL/no-autonomous-Dx
safety commitment · Q2.2 OT no-actuation boundary · Q4.1a healthcare record-boundary
invariant · Q5.2 KR-home-market-first posture · Q7.1 centralized safety-gate set.

---

## Summary of re-framings (for the chair reading this agenda before the interview)

| Original framing | Corrected framing | Evidence |
|---|---|---|
| "CDSS exists NOWHERE (0 grep hits)" | ADR-0332 C.6 has full service charter; owed: HITL/MFDS safety-gate depth | ADR-0332 C.6 line 844; GLOSSARY.md line 328 |
| "Manufacturing-AI absent (defect detection=0)" | j157 journey + vertical-industrial CHARTER + ADR-0027; owed: 6-capability spec + OT invariant promotion | j157 README; CHARTER.md; ADR-0027 line 114 |
| "Bid-pricing exists NOWHERE (낙찰하한율=0)" | ADR-0115 PARTIAL + j160 journey; owed: KR 낙찰하한율 floor | Regression map row ADR-0115; j160 README |
| "Healthcare released-view invariant is a lineage fault" | Already in council-attention list as ADR-0016 PARTIAL | Regression map §3.1 row ADR-0016 |
| "KR employment domain absent (정규직=0)" | Domain EXPANDED; payroll PRD + KR pack have 4대보험/52시간; owed: typed 8-class enum as artifact | payroll.md; PACK.md |
| "KR statute grounding is most churned context" | KR pack and ADR-0064 contain statute citations; owed: home-market-FIRST posture declaration only | PACK.md; ADR-0064 line 156 |
| "M3 first-customer target absent/weaker" | Present in payroll PRD + ADR-0060 + kr.md; confirm-only | payroll.md; ADR-0060 line 90; kr.md line 177 |
| "UserProfile/OrgProfile absent (LOW)" | FULL in regression map (ADR-0138/0139 via OG model + DUB) | Regression map rows ADR-0138/0139 |
| "전자금융업 capital-markets path churned" | 전자금융업 scoped to KR pack in ADR-0064; PortOne/license ladder = depth-owed | ADR-0064 line 156 |
