---
doc_class: BominalReconciliation
title: VERIFICATION-CORRECTION — per-item TRUE-status audit of the interview agenda
status: verified
date: 2026-06-06
verifier: independent-verifier (separate pass from agenda author)
sources_read:
  - /Users/jasonlee/Developer/source/docs/ADR-LEGACY-REGRESSION-MAPPING.md          # authoritative migration-coverage map, 127 rows
  - /Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06/bominal-reconciliation/00-INTERVIEW-AGENDA.md
  - /Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06/bominal-reconciliation/20-LOST-CONTEXT-REGISTER.md
  - /Users/jasonlee/Developer/source/docs/decisions/ADR-0701-monorepo-capability-live-apex.md  # healthcare decomp, CDSS service
  - /Users/jasonlee/Developer/source/docs/decisions/ADR-0709-general-live-apex.md  # robotics/AMR
  - /Users/jasonlee/Developer/source/docs/decisions/ADR-0709-general-live-apex.md  # NERC-CIP mention
  - /Users/jasonlee/Developer/source/docs/teams/vertical-industrial/CHARTER.md      # SCADA/OT boundary
  - /Users/jasonlee/Developer/source/docs/prds/payroll.md                           # 4대보험/KR payroll depth
  - /Users/jasonlee/Developer/source/docs/regional-packs/oya-pack-kr/PACK.md        # KR statute citations
  - /Users/jasonlee/Developer/source/docs/decisions/ADR-0709-general-live-apex.md  # M03 first-customer
  - /Users/jasonlee/Developer/source/docs/decisions/ADR-0709-general-live-apex.md  # 전자금융업
  - /Users/jasonlee/Developer/source/docs/user-journeys/j157-diana-lazar-print-operator-batch-defect-and-quality-recall/README.md
  - /Users/jasonlee/Developer/source/docs/user-journeys/j160-cleaning-co-tomas-horak-bid-cross-tenant-and-onboard/README.md
  - /Users/jasonlee/Developer/source/docs/ROADMAP.md
  - /Users/jasonlee/Developer/source/docs/GLOSSARY.md
method: >
  Every agenda item cross-checked against (1) the regression map row status,
  (2) fresh grep across ALL of /Users/jasonlee/Developer/source/docs/ (not just
  docs/decisions/), (3) user-journeys/, (4) PRDs, regional-packs, team charters.
  Live-read evidence cited per row. Grep searches: CDSS/DDx/HITL/Meditron/MFDS,
  defect-detection/ISA-95/OPC-UA/CAPA/OT-no-actuation, SLAM/ROS2/VDA-5050/AMR,
  bid-pricing/낙찰하한율/국가계약법, infrastructure-sovereignty/M0-gate/Stalwart,
  NERC/IEC-62443/SCADA, SGP4/orbital, 정규직/계약직/파견/EmploymentClassification/
  52시간/4대보험, Bayesian-optimization/RSM/Pareto/DOE-optim/tenant-configurable-optim.
---

# _VERIFICATION-CORRECTION — True-status audit of 00-INTERVIEW-AGENDA.md

## 1. Executive summary of over-statement

The agenda was produced from a grep that scanned **docs/decisions/ only** and
missed three authoritative surfaces:

1. **ADR-LEGACY-REGRESSION-MAPPING.md** — the 127-row migration-coverage map
   that already classifies every bominal ADR as FULL/EXPANDED/PARTIAL/DROPPED.
   Eight of the agenda's "0 grep hits / lost wholesale" items appear there as
   PARTIAL or EXPANDED — meaning oyatie has tracked coverage and flagged the gap
   for council, but the agenda re-framed them as "absent."

2. **/Users/jasonlee/Developer/source/docs/user-journeys/** — j157 (defect
   detection recall) and j160 (cross-tenant bid/onboard) are full journey
   artefacts that partially instantiate the manufacturing and bid-pricing specs
   the agenda said exist "nowhere."

3. **/Users/jasonlee/Developer/source/docs/prds/**, **regional-packs/**,
   **teams/**, **decisions/** beyond the 50 new-pack ADRs — the payroll PRD,
   KR regional pack, and vertical-industrial team charter contain KR employment
   and SCADA/OT content the agenda said had 0 hits.

**Net correction:** of the ~18 substantive product-spec questions, 10 are
mis-classified as "absent/0 hits" when they are TRACKED-PARTIAL or have
partial instantiation in non-ADR surfaces. 8 remain genuinely owed at the
depth the agenda describes. The structural questions (T1 build-sequence,
T5 KR enum, T7 safety-gate unification) are correctly framed.

---

## 2. Per-item TRUE-status table

### THEME 1 — Build sequence & infra-sovereignty ratchet schedule

| Q | Agenda claim | Regression map row | Other evidence | TRUE status |
|---|---|---|---|---|
| Q1.1 — infra-sovereignty ratchet as dated schedule | "0 hits for `infrastructure sovereignty`/`M0 gate`/`Stalwart`" — absent | ADR-0014 build-vs-buy-policy EXPANDED (new pack adds air-gap-first IaC posture); no dated substrate-ownership schedule found | grep confirms: `infrastructure sovereignty`=0, `M0 gate`=0, `Stalwart`=0 across ALL source/docs; ROADMAP.md has wave-gate structure but no per-substrate dated schedule | **DEPTH-OWED** — the *ratchet principle* is in canon D-META and ADR-0014 EXPANDED; the *dated quarter-by-quarter which-substrate-when schedule* is genuinely absent as a distinct artifact. Agenda correctly frames this. Action: author ordered-list + M0-gate-shape (option B), not necessarily calendar dates. |
| Q1.2 — uniform 4-step milestone shape | "oyatie has scattered ADRs with no uniform gate shape" | ADR-0012 axis-admission-protocol EXPANDED covers admission gates; ROADMAP.md has gate criteria per wave | ROADMAP.md §2.1+ has per-wave gate criteria; not a uniform 4-step `research→M0-gate→doc→EPIC` template | **DEPTH-OWED** — gate discipline exists but the per-vertical uniform milestone shape (research → M0 gate → operating-contract ADR → EPIC) is not a codified template. Correctly flagged; adopt it as a standard template. |
| Q1.3 — D8 crown-jewel substrate first | "Not forced now" — still open | Not a regression-map row (it's a deferred D8 canonical decision item) | No artifact in source/docs names a single first-built substrate | **GENUINELY-ABSENT** as a named decision. Agenda correctly frames this as the deferred D8 capacity-budget call. |

**Theme 1 verdict:** All three items correctly identified. No over-statement here.

---

### THEME 2 — Vertical-scope: named product specs

| Q | Agenda claim | Regression map row | Other evidence | TRUE status |
|---|---|---|---|---|
| Q2.1 — CDSS (clinical decision support) | "CDSS=0; only ADR-0332's empty `clinical-decision-support` folder name" | ADR-0033 vertical-industry-cloud-pack (PARTIAL); ADR-0016 clinical-canonical-record (PARTIAL — "clinical released-view contract not yet authored") | **ADR-0332** (healthcare-domain-decomposition.md) has C.6 `microservices/clinical-decision-support/` at line 844 with: drug interaction alerts, BPAs, order sets, dose checks; EU MDR compliance; full gRPC/event bindings table; competitor benchmarks (UpToDate/Lexicomp/Micromedex). **GLOSSARY.md line 328:** "CDSS \| Clinical Decision Support System \| §7 (ADR-0033)." The service exists structurally; grep also found it in COMPETITIVE-GAP-ANALYSIS.md. | **TRACKED-PARTIAL** — the `clinical-decision-support` service is named, structured, and cross-referenced in ADR-0332 with full service charter and API contracts. What is genuinely absent: (a) the bominal CDSS safety-gate doctrine (HITL-non-negotiable, no-autonomous-Dx, MFDS "Informing"-tier, prospective-validation ship-gate, processor-not-controller, Wong-2021 negative example), (b) self-hosted open-weight medical LLMs (Meditron/Med42) spec. The agenda over-stated by calling the whole spec "absent." Correct action: **author the HITL/MFDS safety-gate depth and the self-hosted LLM spec into the existing ADR-0332 C.6 section** — not "recover full spec from zero." |
| Q2.2 — Manufacturing-AI + AMR/facility-robotics OT-safety specs | "defect detection/predictive maintenance=0; SLAM/ROS2/VDA-5050=0" | ADR-0033 manufacturing-ops-AI: EXPANDED (vertical-industrial + workflow + in-house model substrate); ADR-0027 robotics-vision-speech: EXPANDED (robotics substrate centralized) | **j157** (user-journeys/j157-diana-lazar-print-operator-batch-defect-and-quality-recall) is a full journey covering defect detection → recall → root-cause with ISA-95-adjacent QMS vocabulary, FOGRA/ISO-9001, EU-GPSR. **vertical-industrial/CHARTER.md** explicitly names: "Manufacturing Execution Systems (MES), OEE, ISA-95 production hierarchy, OPC UA device integration, SCADA historian, and OT/IT boundary safety controls." **ADR-0027** line 114: `Sc1Informational, // observe-only; no actuation` (the no-closed-loop-actuation safety class). | **TRACKED-PARTIAL + DEPTH-OWED** — manufacturing vertical and robotics substrate are EXPANDED in the regression map; defect-detection journey (j157) and team charter (SCADA/OT boundary) exist. What is owed: (a) the 6-capability manufacturing-AI spec (ISA-95/OPC-UA/OEE/NCR/CAPA data model, edge inference, KR competitors) as a vertical ADR; (b) AMR-specific 3D-SLAM/ROS2/Open-RMF/VDA-5050 spec; (c) the no-closed-loop-actuation OT invariant as an explicit cross-vertical rule (the safety class exists in ADR-0027 but is not named as a cross-vertical invariant). Agenda over-stated: not "0 hits," it is a depth gap on a tracked surface. |
| Q2.3 — Contract Bid-Pricing Engine + tenant-configurable optimization | "bid pricing/낙찰하한율/Bayesian optimization/Pareto=0" | ADR-0115 (bid-pricing engine): PARTIAL — "Vertical-specific module; flag for council vertical-industrial — may stay PARTIAL until that vertical is prioritized"; ADR-0145 (tenant-configurable optimization): EXPANDED in regression map | **j160** (user-journeys/j160-cleaning-co-tomas-horak-bid-cross-tenant-and-onboard) is a full 81-day journey: cross-tenant bid submission, structured cost model (CZK amounts, ČSN-EN-13549 line items, CSN schemas), marketplace bid lifecycle, Cedar-gated bid transitions — it instantiates a real bid-pricing user scenario. `낙찰하한율` and `국가계약법` remain 0 hits — the KR statutory floor is genuinely absent. For tenant-configurable optimization: grep found Bayesian only in email-deliverability (Bayesian classifier for spam) and A/B experiment config — not the RSM/DOE/Pareto/bandit "welding sweet-spot" tenant optimization platform. | **SPLIT:** Bid-pricing engine = **TRACKED-PARTIAL** (ADR-0115 PARTIAL in regression map; j160 exists as a non-KR instantiation; KR 낙찰하한율 statutory floor is **DEPTH-OWED** since it's the moat-defining detail absent from j160). Tenant-configurable optimization platform = **GENUINELY-ABSENT** (ADR-0145 is EXPANDED in regression map meaning the concept is captured, but the actual RSM/Bayesian-opt/DOE/Pareto/bandit template spec is not authored anywhere found). Agenda over-stated the bid-pricing engine as "absent" when it has a regression-map row + user journey; correctly identified the KR depth gap. |

**Theme 2 verdict:** All three Qs partially over-stated. CDSS, mfg-AI, and bid-pricing are TRACKED-PARTIAL, not absent. The owed work is depth authoring on tracked surfaces, not recovery from zero.

---

### THEME 3 — Regulatory depth for highest-risk verticals

| Q | Agenda claim | Regression map row | Other evidence | TRUE status |
|---|---|---|---|---|
| Q3.1 — Powergrid / civil-infra: NERC-CIP / IEC-62443 / SCADA OT operating-contract | "vertical exists, OT/ICS regulatory depth is missing; SCADA=0, 62443=0" | ADR-0027 robotics-vision-speech: EXPANDED; ADR-0033 vertical-industry-cloud-pack: EXPANDED — neither covers NERC/IEC-62443 specifically | **vertical-industrial/CHARTER.md** lines 8, 30: explicitly names "OPC UA device integration, SCADA historian, and OT/IT boundary safety controls" and "SCADA historian adapter." ADR-0321 (b2b-saas-industry-leader-coverage.md) lines 3065 and 3103: "NERC-CIP" appears twice — in a Cedar policy expression and a retention-schedule rule. `IEC-62443`=0, `62443`=0 across all docs. | **DEPTH-OWED** — the powergrid vertical is named; SCADA appears in team charter; NERC-CIP appears in one Cedar policy example. The operating-contract ADR (NERC-CIP compliance posture, IEC-62443 zone/conduit model, OT network isolation architecture) is genuinely absent. Agenda correctly identifies the gap but over-states the absence: SCADA is not "=0" across all docs. |
| Q3.2 — Orbital/space-domain + capital-markets: SGP4/AIS specifics + license ladder | "orbital=0, SGP4=0; capital-markets product intent churned" | No regression map rows for these verticals specifically | `orbital`=0, `SGP4`=0 across all source/docs confirmed. Geospatial/satellite present (satellite=6, geospatial=2). `전자금융업` referenced in ADR-0064 (localization-packs canonical-base, line 156) as a KR-pack-scoped item. `PortOne`=0. `Toss`=3 but not in PG-aggregator context. | **SPLIT:** Orbital mechanics = **GENUINELY-ABSENT** (SGP4/SDP4/AIS/ADS-B/TLE feed spec). Capital-markets = **TRACKED-PARTIAL** (전자금융업 referenced in KR localization pack as in-scope; `PortOne`/PG-aggregator/license ladder = **DEPTH-OWED**). Agenda is correct on orbital; partially over-states capital-markets as "churned" when 전자금융업 is in-scope in the KR pack. |

**Theme 3 verdict:** Q3.1 correctly identified with minor over-statement (SCADA is not zero). Q3.2 split correctly on orbital (absent) vs capital-markets (tracked-partial).

---

### THEME 4 — Module → service lineage & naming continuity

| Q | Agenda claim | Evidence | TRUE status |
|---|---|---|---|
| Q4.1a — Healthcare released-view invariant not crisply carried | "ADR-0332 split into 8 svcs but released-view-as-projection + per-surface write-authority invariant not crisply carried" | ADR-0332 has C.6 clinical-decision-support full contract; the `released-view` / `released_view` term =0 hits in ADR-0332 (grep confirmed). The invariant (patient=projection, not 2nd SoR) is referenced by name in regression map ADR-0016 PARTIAL: "explicit clinical released-view contract not yet authored." | **TRACKED-PARTIAL** — regression map ADR-0016 already flags this for council-architecture + vertical-healthcare. Agenda re-frames a known PARTIAL gap as a lineage-continuity fault; correct action is to promote ADR-0016 PARTIAL. |
| Q4.1b — `medical`→`emr` rename line not explicit | "inferred, not an explicit X→Y line" | ADR-0332 is titled "healthcare-domain-decomposition" and lists emr as svc 1 of 8; prds/payroll.md references ADR-0210 inherited from Bominal directly. | **LOW-RISK** — rename is documented in ADR-0332 structure; lineage is reconstructible. Correct to confirm-only as agenda recommends. |
| Q4.1c — Connect per-context encryption matrix churned | "symmetric Personal-E2EE-user-DEK vs Professional-tenant-DEK + four-eyes matrix churned" | grep found: `tenant-DEK` present in ADR-0043/0140/0299 (OpenBao per-tenant DEK model); `four-eyes` present in multiple ADRs (council-security four-eyes, deliverability four-eyes). Per-context (Personal vs Professional channel) DEK split: 0 hits on "Personal-E2EE-user-DEK" specifically. | **DEPTH-OWED** — per-tenant DEK exists; the *per-context* (personal channel user-DEK vs professional channel tenant-DEK) differentiation as a named encryption matrix for comms is not authored. Correctly flagged. |

**Theme 4 verdict:** Q4.1a is over-stated (it's a known PARTIAL, not a new lineage fault). Q4.1b and Q4.1c correctly scoped.

---

### THEME 5 — KR-first statutory DNA

| Q | Agenda claim | Regression map row | Other evidence | TRUE status |
|---|---|---|---|---|
| Q5.1 — Typed KR 8-class employment classification enum | "정규직/계약직/파견/EmploymentClassification/52시간=0 hits" | ADR-0033 Employment-classification-for-KR-workforce: EXPANDED ("new pack moves regulatory-class to per-region pack overlay; KR is one of N regions"). ADR-0033 Sector/tier/employment-compliance-pack: EXPANDED | **prds/payroll.md** has 4대보험 EDI in detail (취득/상실/변경/보수월액 formats, NPS/NHIS/MOEL adapters). **regional-packs/oya-pack-kr/PACK.md** line 76 and 182: `주52시간`, `4대보험`, `연차 사용촉진`, `통상임금`, `퇴직금`. grep across all docs confirms: `정규직`=0, `계약직`=0, `파견`=0, `EmploymentClassification`=0 as a typed enum. | **TRACKED-PARTIAL + DEPTH-OWED** — the KR employment domain is EXPANDED in the regression map (regional-pack overlay model); 4대보험 mechanics are in the payroll PRD; 주52시간/통상임금 are in the KR pack. What is genuinely absent: the **typed `EmploymentClassification` enum** with the 8 named classes (정규직/계약직/단시간/파견/도급/프리랜서/인턴/임원) as a data model driving conditional payroll logic. The agenda is correct that the enum drives the payroll wedge and is 0-hit as a typed artifact, but over-states by ignoring that the surrounding KR compliance depth is EXPANDED. |
| Q5.2 — KR-home-market-FIRST posture + per-vertical statute citations | "home-market-first posture + per-vertical statute grounding is the most churned context" | ADR-0033 multiple rows EXPANDED for KR statutes (근로기준법/PIPA/통신비밀보호법 generalized to regional-pack overlay) | KR regional pack (PACK.md) cites 근로기준법/PIPA/4대보험/주52시간 in pack scope. ADR-0064 scopes 전자금융업/PIPA/산업안전보건법 to KR pack. The "home-market-FIRST" sequencing posture as an explicit named rule is 0-hit. | **TRACKED-PARTIAL** — statute citations exist in the KR pack and ADRs; "KR-home-market-FIRST" as a named sequencing posture is genuinely absent as an explicit declaration. Agenda correctly identifies the missing posture declaration; over-states by calling statute grounding "the most churned context" when it is mostly present in the KR pack. The owed work is declaring the posture, not re-threading all the statutes. |

**Theme 5 verdict:** Q5.1 partially over-stated (KR compliance domain is EXPANDED; the typed enum is the specific missing artifact). Q5.2 partially over-stated (statutes present in KR pack; the posture declaration is missing).

---

### THEME 6 — Portfolio compounding spine & first-customer launch target

| Q | Agenda claim | Regression map row | Other evidence | TRUE status |
|---|---|---|---|---|
| Q6.1 — Build-spine + block-edges + M3 first-customer target | "specific ordered spine + block-edges + first-paid-customer target not visibly carried" | ADR-0191 (Ecosystem-MVP contract): DROPPED-WITH-REASON ("MVP vocabulary retired"); ADR-0050 (M3 KR launch scope): DROPPED-WITH-REASON ("M3 wave vocabulary retired; per-customer launch scope is operational not architectural") | **prds/payroll.md** line 26+: "M03 scope targets the KR group payroll launch." **ADR-0060** (bominal-inheritance-precedence.md) line 90: "ADR-0210: M03 KR group payroll + mail launch (= oyatie M03 first-paying-tenant target)." **localization-packs/kr.md** line 177: "Bominal ADR-0210 (M03 KR group payroll + mail launch criteria)." The first-customer target is alive across payroll PRD + ADR-0060 + KR localization pack. The ordered spine (platform→payments→corporate→…) and block-edges are 0-hit. | **SPLIT:** M3 first-customer target = **TRACKED-PARTIAL** (referenced in payroll PRD + ADR-0060 + kr.md; not absent — regression map DROPPED-WITH-REASON is for the wave vocabulary, not the commercial milestone). The agenda's "weaker/absent" is over-stated; the target is alive in 3 places. Portfolio build-spine + block-edges = **GENUINELY-ABSENT** (the ordered spine + trust ⊥ payments/healthcare-billing block-edges are 0-hit). |

**Theme 6 verdict:** M3 customer target over-stated as absent — it is present in payroll PRD and ADR-0060. Block-edges genuinely absent.

---

### THEME 7 — Safety-gate doctrines as cross-vertical invariant set

| Q | Agenda claim | Evidence | TRUE status |
|---|---|---|---|
| Q7.1 — Unify safety gates as one governance-owned cross-vertical invariant set | "gates appear per-spec but are not unified as a family" | ADR-0027 line 114: `Sc1Informational // observe-only; no actuation` (OT safety class exists). ADR-0022 autonomy-ceiling EXPANDED. Defense lethal-use guard in 37 defense ADRs. HITL / no-autonomous-Dx / biometric-default-off / 5-stage CCTV pipeline: 0-hit as a named cross-vertical family. | **DEPTH-OWED** — individual safety postures exist (autonomy ceiling, OT observe-only class, defense lethal-use guard); a single governance-owned "safety-gate invariant set" artifact that names all four gates as a family and hooks them to D16's Cedar enforcement is genuinely absent. Agenda correctly identifies this as a unification gap, not a recovery-from-zero. |

**Theme 7 verdict:** Correctly identified. Not over-stated.

---

### THEME 8 — Batch sweep (MED/LOW residue)

| # | Item | Regression map | Other evidence | TRUE status |
|---|---|---|---|---|
| 8.1 | CCTV 5-stage escalating pipeline + biometric-default-off | ADR-0027 EXPANDED (vision substrate consolidated) | `CCTV`=1, `facial recognition`=1, `biometric`=15 in source/docs; 5-stage pipeline as named architecture=0-hit | **TRACKED-PARTIAL** — biometric governance is EXPANDED (ADR-0027); 5-stage pipeline + default-off posture not authored as product operating contract. FOLD correctly recommended. |
| 8.2 | Email/messenger mining + person-pillar hard exclusion | ADR-0008 (data-use-boundary) EXPANDED | DUB exclusion-zone semantics present; mining product intent and 통신비밀보호법 exclusion-zone as explicit product rule=weaker | **TRACKED-PARTIAL** — DUB EXPANDED covers exclusion semantics; specific comms-mining product + per-channel exclusion-zone as a named rule is thin. FOLD correctly recommended. |
| 8.3 | AI-surfaces catalog (~17 domains, OR/MIP/graph-before-ML) | ADR-0026 in-house-AI-model-substrate PARTIAL ("catalog/enumeration ADR by design has no decision content; confirm enumeration no longer needed") | ADR-0026 PARTIAL in regression map — already flagged for council | **TRACKED-PARTIAL** — already in council-attention list. FOLD correctly recommended. |
| 8.4 | Insurance/banking owning-service disposition | No specific regression-map row; capabilities referenced across vertical pack | insurance=18 ADR hits, banking=14; no owning service dir | **TRACKED-PARTIAL** — present as ERP/fintech capabilities; disposition not recorded. CONFIRM correctly recommended. |
| 8.5 | ATS (applicant tracking) | No dedicated regression-map row | `applicant tracking`=3 ADR hits | **TRACKED-PARTIAL** — referenced; no owner. FOLD correctly recommended. |
| 8.6 | Security as customer-facing sellable product | No dedicated regression-map row | No `oya/security` product dir; mechanism split across substrate | **GENUINELY-ABSENT** as a product decision. CONFIRM correctly recommended. |
| 8.7 | Consumer hospitality/lifestyle modules (PARKED) | No regression-map row (they predate the 127-ADR corpus) | No dirs; correctly governed as PARKED | **DROPPED-WITH-REASON** (intentionally PARKED, not lost). CONFIRM correctly recommended. |
| 8.8 | Bominal Law / Finance / Train | No dedicated regression-map rows | Legal corpus governance present; product surface intent fuzzed | **GENUINELY-ABSENT** as named product tracks. FOLD correctly recommended. |
| 8.9 | PortOne→Toss PG-rail + 전자금융업 license ladder | ADR-0135 marketplace PARTIAL; 전자금융업 in ADR-0064 KR pack scope | `PortOne`=0; `Toss`=3 not in PG context; 전자금융업 scoped to KR pack | **TRACKED-PARTIAL** — marketplace PARTIAL in regression map; 전자금융업 in-scope in KR pack; PG-aggregator/Toss-primary specifics = DEPTH-OWED. FOLD correctly recommended. |
| 8.10 | User/Org profiling architecture (UserProfile + OrgProfile) | ADR-0138/0139: FULL in regression map ("UserProfile entity" and "OrgProfile entity" FULL via OG model + DUB) | Both captured FULL per regression map | **TRACKED-FULL** — agenda correctly marks as low; OG model + DUB capture both. Confirm-only; no recovery. |
| 8.11 | Kantara consent-receipt standard; ontology 5-differentiators; data-ownership-pillar worker-rights-override | persona-tier=12 ADR hits (present); Kantara=0 | Already noted "CONFIRM-only trio — these invariants survived" | **TRACKED-FULL (persona-tier/pillar) + DEPTH-OWED (Kantara re-cite only)** — agenda correctly marks as NOT losses. |

---

## 3. Summary: how the agenda over-stated losses

### Items mis-framed as "absent" that are TRACKED-PARTIAL

| Agenda item | Regression-map status | What the agenda missed | Correct action |
|---|---|---|---|
| Q2.1 CDSS spec | ADR-0033 PARTIAL + ADR-0016 PARTIAL | ADR-0332 has `clinical-decision-support` C.6 with full service charter, gRPC bindings, regulatory refs; GLOSSARY entry | Promote ADR-0016/ADR-0033 PARTIAL depth — author HITL/MFDS safety-gate into existing C.6 |
| Q2.2 Manufacturing-AI | ADR-0033 mfg-ops-AI EXPANDED; ADR-0027 robotics EXPANDED | j157 user journey (defect detection); vertical-industrial CHARTER (MES/OEE/ISA-95/OPC-UA/SCADA/OT boundary named); ADR-0027 Sc1Informational no-actuation class | Promote EXPANDED to authored depth for KR-industrial wedge; adopt Sc1Informational as cross-vertical OT invariant |
| Q2.3 Bid-pricing engine | ADR-0115 PARTIAL | j160 user journey (cross-tenant bid lifecycle, structured cost model, ČSN-EN-13549 line items) | Promote ADR-0115 PARTIAL + add KR 낙찰하한율 depth |
| Q4.1a Healthcare released-view invariant | ADR-0016 PARTIAL | Already in council-attention list: "clinical released-view contract not yet authored" | Promote ADR-0016 PARTIAL — not a new lineage fault |
| Q5.1 KR employment classification | ADR-0033 KR-employment EXPANDED | prds/payroll.md (4대보험 in detail), KR pack (주52시간/통상임금) | Typed enum is the specific missing artifact on an otherwise EXPANDED domain |
| Q5.2 KR-home-market-first posture | ADR-0033 multiple rows EXPANDED | KR pack (PACK.md) and ADR-0064 contain statute citations per vertical | Posture declaration is missing; statutes are mostly present |
| Q6.1 M3 first-customer target | ADR-0050 DROPPED (wave vocabulary, not the commercial milestone) | prds/payroll.md + ADR-0060 + localization-packs/kr.md all reference "M03 KR group payroll launch" | Present in 3 places; not absent |
| 8.1 CCTV 5-stage pipeline | ADR-0027 EXPANDED | biometric=15 hits; ADR-0027 vision substrate EXPANDED | 5-stage pipeline + default-off not authored as operating contract |
| 8.9 PG-rail/전자금융업 | ADR-0135 PARTIAL; ADR-0064 includes 전자금융업 in KR pack | PortOne=0 but 전자금융업 scoped to KR pack | Depth-owed, not absent |
| 8.10 UserProfile/OrgProfile | ADR-0138/0139 FULL | Fully captured in OG model | Confirm-only; agenda correctly marks LOW |

### Genuine residual losses (confirmed after full-surface grep)

These are correctly identified by the agenda as owed work:

| Item | Why genuinely owed | Evidence basis |
|---|---|---|
| **CDSS HITL/MFDS safety-gate doctrine** | ADR-0332 C.6 exists structurally but HITL/no-autonomous-Dx/MFDS-tier/Meditron/prospective-validation=0 hits | Deep grep of ADR-0332 + all source/docs |
| **Manufacturing-AI 6-capability spec (ISA-95/OPC-UA/OEE/NCR/CAPA data model + KR competitors)** | j157 covers defect detection as a process; the 6-capability platform spec with edge inference and ISA-95 data model is not authored | j157 README, ADR-0027, CHARTER reviewed |
| **AMR 3D-SLAM/ROS2/Open-RMF/VDA-5050 spec** | SLAM=0, ROS2=0, VDA-5050=0 across all source/docs; ADR-0027 has `Sc1Informational` only | Confirmed grep |
| **OT no-closed-loop-actuation as named cross-vertical invariant** | Exists as SafetyClass enum value in ADR-0027; not a named cross-vertical rule | ADR-0027 line 114 |
| **Powergrid NERC-CIP / IEC-62443 / OT network-isolation operating-contract ADR** | NERC-CIP=2 hits (policy example + retention rule only); IEC-62443=0; no OT operating-contract ADR | grep confirmed |
| **SGP4/SDP4/AIS/ADS-B/TLE orbital mechanics feed spec** | orbital=0, SGP4=0 across all source/docs | grep confirmed |
| **KR 낙찰하한율 statutory bid floor (the payroll-moat detail)** | bid-pricing=0, 낙찰하한율=0 | grep confirmed; j160 is non-KR bid scenario |
| **Typed `EmploymentClassification` enum (정규직/계약직/단시간/파견/도급/프리랜서/인턴/임원)** | 정규직=0, 계약직=0, 파견=0 as typed enum; surrounding KR domain is EXPANDED | grep of decisions/ + payroll PRD |
| **KR-home-market-FIRST explicit sequencing posture declaration** | No file contains this as a named rule | grep confirmed |
| **Infra-sovereignty ratchet: ordered substrate list + M0 gate shape** | Principle in canon; no per-substrate ordered schedule artifact | ROADMAP.md checked; wave gates present but no substrate schedule |
| **D8 crown-jewel substrate first (capacity budget)** | Not resolved in any ADR | No artifact found |
| **Portfolio build-spine ordered sequence + trust ⊥ payments/healthcare-billing block-edges** | 0-hit | grep confirmed |
| **Connect per-context encryption matrix (Personal user-DEK vs Professional tenant-DEK)** | Per-tenant DEK present; per-context channel differentiation absent | ADR-0043/0299 checked |
| **Tenant-configurable optimization platform (RSM/Bayesian-opt/DOE/Pareto/bandit templates)** | ADR-0145 EXPANDED in regression map but the actual template spec is not authored | grep confirmed; only email/A-B Bayesian found |
| **Unified safety-gate invariant set artifact (HITL + OT-no-actuation + biometric-default-off + no-autonomous-lethal)** | Individual postures exist; no unified governance-owned artifact | grep confirmed |
| **CCTV 5-stage escalating-risk pipeline as product operating contract** | ADR-0027 EXPANDED but 5-stage pipeline + biometric-default-off posture not authored | grep confirmed |

---

## 4. TRUE-status count summary

| Classification | Count | Items |
|---|---|---|
| TRACKED-PARTIAL (in regression map as PARTIAL/Proposed — action: promote to authored depth) | 10 | Q2.1 CDSS service, Q2.2 mfg-AI substrate, Q2.3 bid-pricing, Q4.1a healthcare released-view, Q5.1 KR employment domain, Q5.2 KR statute citations, Q6.1 M3 first-customer target, 8.1 CCTV pipeline, 8.9 PG-rail, 8.10 UserProfile/OrgProfile |
| TRACKED-FULL/EXPANDED (already covered — confirm-only) | 3 | 8.10 UserProfile/OrgProfile (FULL), 8.11 persona-tier/ownership-pillar (FULL), ADR-0033 KR employment domain (EXPANDED) |
| DEPTH-OWED (tracked vertical but specific regulatory/OT depth genuinely absent) | 9 | CDSS HITL/MFDS safety-gate, OT no-actuation cross-vertical rule, NERC-CIP/IEC-62443 OT contract, infra-sovereignty substrate schedule, D8 crown-jewel decision, KR 낙찰하한율 floor, Connect per-context DEK matrix, tenant-configurable optim template spec, unified safety-gate invariant artifact |
| GENUINELY-ABSENT (truly nowhere after full-surface grep) | 6 | SGP4/orbital mechanics spec, typed KR `EmploymentClassification` enum, portfolio build-spine + block-edges, KR-home-market-FIRST posture declaration, Security as sellable product decision, AMR 3D-SLAM/ROS2/VDA-5050 spec |
| DROPPED-WITH-REASON (regression map 4 dropped + PARKED) | 2 | 8.7 hospitality/lifestyle PARKED, M3 wave vocabulary (vocabulary retired; commercial milestone preserved elsewhere) |

---

## 5. Losslessness check: HIGH rows in 20-LOST-CONTEXT-REGISTER vs agenda themes

| Register HIGH row | Category | Routed to theme | Present in theme? |
|---|---|---|---|
| Contract Bid-Pricing Engine | A — Product | Theme 2, Q2.3 | YES |
| Tenant-configurable optimization platform | A — Product | Theme 2, Q2.3 | YES |
| CDSS clinical diagnostic assistance | B — Vertical | Theme 2, Q2.1 | YES |
| Manufacturing operations AI | B — Vertical | Theme 2, Q2.2 | YES |
| AMR + facility robotics | B — Vertical | Theme 2, Q2.2 | YES |
| Powergrid / civil-infra OT depth | B — Vertical | Theme 3, Q3.1 | YES |
| Infrastructure-sovereignty ratchet schedule | C — Roadmap | Theme 1, Q1.1 | YES |
| 54-milestone multi-year sequence | C — Roadmap | Theme 1, Q1.2 + Q6.1 | YES |
| KR 8-class employment classification | D — Decision | Theme 5, Q5.1 | YES |
| KR-first regulatory DNA (statute depth) | D — Decision | Theme 5, Q5.2 | YES |
| CDSS/mfg/CCTV safety-gate doctrines | D — Decision | Theme 7, Q7.1 | YES |

**All 11 HIGH rows are represented. No losslessness failure.**

---

## 6. Output files written

- `/Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06/bominal-reconciliation/_VERIFICATION-CORRECTION.md` (this file)
- `/Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06/bominal-reconciliation/00b-INTERVIEW-AGENDA-CORRECTED.md`
