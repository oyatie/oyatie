---
doc_class: BominalReconciliation
title: LOST-CONTEXT REGISTER — what the bominal→oyatie migration churned or lost
status: synthesized
date: 2026-06-06
premise: >
  oyatie WAS bominal (renamed + migrated; context churned). This register diffs bominal (past)
  against oyatie (present) to find what the migration LOST or fuzzed. This is recovery of
  oyatie's OWN history, not adoption of a foreign repo.
inputs:
  - bominal-reconciliation/10-bominal-product-roadmap.md   (16 modules + 54-milestone roadmap + ADR-0150 sovereignty schedule)
  - bominal-reconciliation/11-bominal-decisions.md         (132 bominal ADRs, grouped present/weaker/absent)
  - bominal-reconciliation/12-bominal-strategy.md          (moat/sequencing/Proof-Ladder/kill/Moves program)
  - bominal-reconciliation/13-oyatie-present.md            (live oya/+cloud/ surface + rename map)
  - legacy-recovery/00-RECOVERY-REGISTER.md                (7 .Trash recoveries — referenced, NOT duplicated)
  - synthesis/decision-record-oyatie-canon.md             (founder-ruled canon — NOT re-surfaced)
method: >
  Every status flag was SPOT-VERIFIED against the live oyatie ADR corpus
  (/Users/jasonlee/Developer/source/docs/decisions, ADR-0001..0514) and the live oya/ service
  tree by grep, NOT taken from the upstream digests' estimates. Where the live read contradicts
  a digest estimate, the live read wins and the correction is noted. Severity reflects genuine
  intent-loss risk after that verification.
legend:
  oyatie_status: absent | weaker | renamed-unclear | present(+/stronger)
  severity: HIGH | MED | LOW
relation_to_canon: >
  Does NOT re-surface the 16 founder-ruled D-decisions (own-everything ratchet, forge=GitHub-now,
  data-tier-own-all, identity=oya-identity+Zitadel, Cedar-contract, framekernel-host, D9
  maximal-vertical-scope incl. defense+powergrid, D-LAYER dogfood, D-LINEAGE bominal-fact). Does
  NOT duplicate the 7 .Trash recoveries (KR HR/payroll packs, First Proof Slice, 4-lane closure
  model, EaaS framing, trust 9-key-classes, deferral rationale, 8-stage chain). Where this register
  touches one of those, it is to LOCATE the bominal-side authoritative source, not re-rule it.
---

# 20 — LOST-CONTEXT REGISTER (bominal → oyatie)

## 0. Headline + honest framing

The migration kept the **architecture spine** (Object Graph→`ontology`, hexagonal, FORCE-RLS
tenancy, Merkle audit-chain, Cedar, data-tier matrix, multi-runtime, cloud-native infra) — those
are PRESENT and usually **stronger** in oyatie (own-everything ratchet, 514 ADRs vs 132). The
migration also did **not** drop the far-future verticals wholesale: a live grep found
**defense (37 ADRs), ITAR (10), public-sector (12), datacenter (8), satellite (6), drone (4),
agriculture (3), utilities (3), NERC (2), geospatial (2)** — so the canon's D9 "net-new
defense/powergrid" capture has largely **already landed** as ADRs. Honest correction to the
upstream digests, which estimated these as ABSENT/WEAKER from the canon doc without a live read:
**most of the far-future TIER is present; what is lost is narrower and more specific.**

What the churn genuinely LOST concentrates in three places:
1. **Named, regulator-grounded PRODUCT SPECS** that exist nowhere in oyatie (verified 0 hits):
   CDSS clinical-diagnostics, Manufacturing-AI, AMR/robotics (SLAM/ROS2/VDA-5050),
   Contract-Bid-Pricing-Engine (낙찰하한율 floor), the tenant-configurable optimization platform.
2. **The multi-year ROADMAP SEQUENCING + the ADR-0150 infra-sovereignty RATCHET SCHEDULE**
   (which substrate owned in which quarter) — verticals exist as ADRs but the **ordered, gated,
   dated build plan that sequences them is gone** (no staged-schedule artifact; `M0 gate`,
   `infrastructure sovereignty`, `Stalwart` = 0 hits).
3. **KR-first regulatory DNA depth** — the 8-class employment enum (정규직/계약직/파견 = 0 hits),
   the 낙찰하한율 statutory bid floor, PortOne/전자금융업 payment-rail specifics: the home-market
   statute grounding that MOTIVATED every vertical is the single most churned context.

A few digest estimates were **over-pessimistic** and are corrected to PRESENT here: persona-tier
(12 ADRs), Ecosystem-as-a-Service (13), Bench host-surface (63), consent-receipts (7),
insurance/banking as ADR capabilities (18/14). These are NOT losses — flagged to prevent
over-recovery.

---

## A. PRODUCT / MODULE scope lost or fuzzed

| Item | What it was (bominal ref) | oyatie status | Severity | Restore recommendation |
|---|---|---|---|---|
| **Contract Bid-Pricing Engine** | Platform-wide bid→contract→budget engine; live labor-rate feed from corp-pay (the moat vs Procore/Deltek); KR 국가계약법 **낙찰하한율** statutory floor enforcement (ADR-0115) | **absent** (verified: `bid pricing`/`labor rate`/`낙찰하한율` = 0 ADR hits) | **HIGH** | RECOVER-as-ADR. Named go-to-market wedge whose moat (live-payroll → bid pricing) is unreproducible from the flat tree; feeds Task #18 corporate vertical. |
| **Tenant-configurable optimization platform** | Workflow-Studio extension: tenants bring data+parameter-space+objectives; platform supplies RSM/Bayesian-opt/DOE/Pareto/bandit templates ("welding sweet-spot" generalization; Foundry-parity self-serve) (ADR-0145) | **absent** (verified: `response surface`/`Bayesian optimization`/`Pareto` = 0) | **HIGH** | RECOVER-as-ADR. The "customers encode their company into us" doctrine made self-serve — a distinctive platform-AI surface with no oyatie trace. |
| **Healthcare 5-surface record-boundary split** | Medical = canonical write authority; patient = released-view projection (NOT a 2nd SoR); Pharmacy/Emergency/Records as scoped surfaces; "one medical domain, multiple surfaces" (ADR-0016/0011) | **weaker** (ADR-0332 decomposed healthcare into 8 svcs `emr`/`diagnostics`/`imaging`/… but the **released-view-as-projection + per-surface write-authority invariant** is not crisply carried) | **MED** | FOLD into the ADR-0332 healthcare-decomposition doc: restate released-view-is-not-a-record + scoped-write-authority as an invariant. |
| **`insurance` / `banking` owning service** | bominal FinTech-arm verticals, named in ADR-0058 catalog | **renamed-unclear** (no `oya/insurance` or `oya/banking` dir, BUT 18 / 14 ADR hits — present as ERP/fintech *capabilities*, no owning service, no retirement ADR) | **MED** | Confirm in Task #18 whether these are intentionally capability-only or owe a service dir; record the disposition (don't silently leave dir-less). |
| **`ats` (applicant tracking)** | Recruiting surface inside Corporate (ADR-0058) | **renamed-unclear** (no dir; `applicant tracking` = 3 ADR hits — thin, likely folded into `hr`/`performance-management` with no enumerated owner) | **LOW** | FOLD: name ATS as an explicit `hr` capability-tier or give it an owner; thin but real recruiting gap. |
| **`security` as a customer-facing product** | Bominal Security Service: customer-facing log-ingest/posture/evidence + Workflow-backed response playbooks (ADR-0072 hexagonal skeleton) | **renamed-unclear** (no product `oya/security` dir; mechanism split across `detection`/`compliance`/`governance` as substrate, not a sold product) | **LOW** | Confirm whether the *sellable* security product survives or is substrate-only; record. |
| **Consumer hospitality/lifestyle modules** (`dining`, `cellar`, `pos`, `retail`, `fashion`, `career`) | Hospitality+Lifestyle arm; explicitly PARKED pre-L1 (`#1450`), not killed (ADR-0203) | **absent** (no dirs) but **already governed as PARKED** | **LOW** | Already correctly parked under D9 "scope-OUT stays VISIBLE." Ensure the parked list survives into the masterplan roadmap so it isn't silently lost; no ADR needed. |
| **Bominal Law / Bominal Finance / Bominal Train** (adjacent product tracks) | Law on `oya-kernel-legal` (cite-not-advise); Finance reference-rewrite; Train brownfield hobby track (ADR-0190/0220) | **weaker** (legal-corpus governance present; "Law/Finance/Train as named product surfaces" intent fuzzed) | **LOW** | FOLD into roadmap as named-but-deferred product tracks; low intent-loss. |

**Corrected (NOT losses — were over-flagged in digests):** EaaS framing (13 ADRs), Bench host
surface (63), persona-tier (12), product-control plane (present, stronger). Do not recover.

---

## B. VERTICAL scope lost (far-future tier especially)

> **Major correction vs digests:** the far-future verticals are mostly PRESENT in oyatie ADRs.
> The loss is the **OT/safety regulatory depth** for the highest-risk ones + a few named specs.

| Item | What it was (bominal ref) | oyatie status | Severity | Restore recommendation |
|---|---|---|---|---|
| **CDSS clinical diagnostic assistance** | DDx engine + multimodal imaging (CXR) + EKG + red-flag escalation; self-hosted open-weight medical LLMs (Meditron/Med42, never cloud, PHI-residency); MFDS "Informing"-tier; HITL non-negotiable; Bominal=processor-not-controller; Wong-2021 Epic-Sepsis negative example (ADR-0137) | **absent** (verified: `CDSS` = 0; `clinical decision` = 1 = only ADR-0332's *named-folder-pending* `clinical-decision-support`, no spec; no `oya/clinical-decision-support` dir) | **HIGH** | RECOVER-as-ADR. Deepest product+regulatory spec in bominal; the safety-gate doctrine (HITL, prospective-validation ship gate, no-autonomous-Dx) is load-bearing and unreproducible. |
| **Manufacturing operations AI** | 6 capabilities (defect detection, predictive maintenance, fault/root-cause, accountability, workflow + tax/financial optimization); ISA-95/OPC-UA/OEE/NCR/CAPA data model; edge inference; OT no-closed-loop-actuation; KR competitors SUALAB/MAKINAROCKS (ADR-0143/0217) | **absent** (verified: `defect detection`/`predictive maintenance` = 0; `visual inspection` = 1 incidental) | **HIGH** | RECOVER-as-ADR. The OT-safety (no-actuation) boundary + ISA-95 model is a hard safety posture; feeds Task #18 manufacturing vertical. |
| **AMR + facility robotics** | 3D SLAM mapping, hazard detection, autonomous servicing, pathfinding, fleet mgmt; ROS2 + edge perception; Open-RMF/VDA-5050; robotics safety sandbox (ADR-0142) | **absent** (verified: `SLAM`/`ROS2`/`ROS 2`/`VDA 5050` = 0; `AMR` = 5 but unrelated — abbreviation collisions) | **HIGH** | RECOVER-as-ADR. Physical-safety-critical vertical with zero oyatie trace; the safety-sandbox doctrine matters. |
| **powergrid / civil-infra OT depth (NERC-CIP, IEC-62443, SCADA)** | 2030-Q1 Civil-Infra+Utilities milestone (#47); utility-network operating contract; OT safety-critical (roadmap §b far-future) | **weaker** (verified: `utilities` 3, `NERC` 2, `power-grid` 1 — vertical NAMED in ADRs, but `SCADA` = 0, `62443` = 0, `powergrid` = 0 — the **OT/ICS regulatory depth is missing**) | **HIGH** | Feed Task #18: the vertical exists but owes a NERC-CIP/IEC-62443/SCADA OT operating-contract ADR. This IS the canon-D9 "net-new power-grid" item — partly captured, depth still owed. |
| **Realtime tracking + orbital intelligence** | Ships/flights/satellites; AIS/ADS-B/TLE-OMM; **SGP4/SDP4 orbit prediction**; defense-COP events; restricted-defense-use guard (2028-Q2, #1119/#1128 Infrastructure Moat) | **weaker→absent** (verified: `satellite` 6, `geospatial` 2 present, BUT `orbital` = 0, SGP4/SDP4 = 0 — the **orbit-prediction/space-domain specifics are gone**) | **MED** | FOLD/RECOVER: geospatial is captured; the orbital-mechanics + AIS/ADS-B/TLE feed spec is the lost detail. |
| **Capital-markets fintech** | 2028-Q1 instrument master, market/reference data, portfolio/NAV/P&L, ETF holdings, buy/sell-side research; Aladdin/Bloomberg benchmarks (roadmap §b) | **weaker** (billing/finops substrate stronger; capital-markets-as-product-arm + the PG→bank-license ladder + 전자금융업 path not carried as product intent) | **MED** | Feed Task #18 fintech vertical: capture the regulated-partner→own-license ladder + product-boundary discipline (software-over-regulated-partners). |
| **CCTV vision pipeline** | 5 escalating-risk stages (motion→object→personnel→facial→identity-match); biometric OFF by default, jurisdiction-gated (BIPA/EU-AI-Act/PIPA Art.23); edge stages 1-2 / central 3-5; encrypted identity gallery (ADR-0141) | **weaker** (verified: `CCTV` 1, `facial recognition` 1, `biometric` 15 — biometric governance PRESENT and broad, but the **5-stage escalating-risk pipeline + default-off posture as a named product** is thinner) | **MED** | FOLD: oyatie has biometric governance; restate the 5-stage pipeline + biometric-default-off as the vision-product operating contract. |
| **Email/messenger MINING product + person-pillar exclusion** | F1 org-pillar BUILD (BEC defense, revenue-intel, workforce analytics); F2 person-pillar **hard EXCLUSION ZONE** (do-not-build, 통신비밀보호법 1-10yr) (ADR-0136) | **weaker** (org/person split is canon-adjacent; the *mining product + hard person-pillar exclusion* is not stated as product intent) | **MED** | FOLD into comms-intelligence + the data-ownership-pillar doc; the exclusion-zone is a security-relevant invariant. |
| **User/Org profiling architecture** | UserProfile + behavioral-event pipeline; OrganizationProfile + firmographic enrichment + health_score (feeds marketplace/fraud) (ADR-0138/0139) | **absent** (no profiling-program ADR trace) | **LOW** | Capture as a marketplace/fraud-feeding capability if still in scope; lower intent-loss. |

**Corrected (NOT losses):** defense (37), ITAR (10), public-sector (12), agriculture (3),
data-center/datacenter (3/8) — the canon-D9 far-future tier has largely landed as ADRs. The loss
is **depth** (OT/orbital/safety specs), not the verticals.

---

## C. ROADMAP / SEQUENCING lost

> **This is the highest-leverage structural loss after the named specs.** oyatie has the
> verticals as a flat ADR set; it lost the **ordered, dated, evidence-gated build plan** that
> turned breadth into a buildable sequence.

| Item | What it was (bominal ref) | oyatie status | Severity | Restore recommendation |
|---|---|---|---|---|
| **Infrastructure-Sovereignty RATCHET schedule** | ADR-0150 staged own-the-substrate plan: which substrate owned in which quarter — **2027-Q2** IaC/OpenTofu → **2027-Q3** API-Gateway + KMS/Vault → **2027-Q4** Mail(Stalwart-class) + Cache + Event-Streaming → **2028-Q1** DB + Storage → **2028-Q2+** shadow candidates; each gated at **M0 = contract + incumbent benchmark + evidence** before replacement | **absent** (verified: `infrastructure sovereignty` 0, `M0 gate` 0, `Stalwart` 0, `shadow candidate` 1; `own the substrate` 2 generic. The *ratchet PRINCIPLE* survives as canon D-META, but the **dated which-substrate-when schedule does not exist** as an artifact) | **HIGH** | RECOVER-as-ADR (+ masterplan). D-META rules own-the-endpoint; this is the operational SCHEDULE that sequences it. Reconcile against the day-0 capacity-budget (audit #8, still owed) — they answer the same question: build order. |
| **54-milestone multi-year vertical roadmap (2026→2030)** | The ordered milestone graph: KR-corporate+industrial-spine (2026) → health-AI/marketplace/vision/robotics (2027) → conglomerate+capital-markets+geospatial (2028) → far-future tier (2029-2030); each far-future milestone uniform-shaped `research(benchmark matrix) → PHASE M0 → doc(adr) operating contract → EPIC` | **absent as an ordered plan** (verticals exist as scattered ADRs; the **dated sequence + the uniform M0-gate milestone shape** is gone — only 7 ADR files carry any quarter-dated milestone) | **HIGH** | Feed Task #18 + masterplan: the vertical-coverage map should carry the SEQUENCE + the per-vertical M0 evidence-gate shape, not just the list. This directly answers D9's "sequenced, not cut." |
| **Portfolio compounding spine** | `platform→payments→corporate→messaging→documents→notify→healthcare-billing→intelligence` build order, with block-edges (trust ⊥ healthcare-billing@L4, ⊥ payments@L4) and per-arm investment levels (growth/maintain/seed) | **weaker** (portfolio governance + Proof-Ladder present per recovery register; the *specific ordered spine + block-edges + investment-levels* is the strategy detail not visibly carried) | **MED** | FOLD into the portfolio/masterplan doc; ties to the still-owed D8 capacity budget. |
| **M3 first-customer launch target** | M3 scope = KR group payroll + corporate Mail production; **≥1 paid KR group customer (~3000 wage employees) closes real payroll before M3 done**; payroll-first public claim (ADR-0210) | **weaker/absent** (concrete first-customer go-to-market target; maps loosely to the .Trash "First Proof Slice" gap, recovery #2) | **MED** | FOLD with legacy-recovery #2 (First Proof Slice). Together they are the missing "what is the FIRST buildable+sellable slice" artifact. |
| **Enterprise-Cloud-Readiness M0–M3 gate program** | 2027-Q2→2028-Q1 claim-boundary → business-cloud package → enterprise-beta → one-stop-claim; "no broad public claim before M2 evidence" (ADR-0187) | **weaker** (claim-evidence discipline is canon-adjacent; the staged claim-gate program not carried) | **LOW** | FOLD into the trust/claims doc; low intent-loss. |

---

## D. DECISION / RATIONALE lost (intent encoded in bominal ADRs, not carried)

| Item | What it was (bominal ref) | oyatie status | Severity | Restore recommendation |
|---|---|---|---|---|
| **KR 8-class employment classification** | Typed `EmploymentClassification` enum (정규직/계약직/단시간/파견/도급/프리랜서/인턴/임원) driving payroll/4대보험/leave/52h/severance/withholding-stream (ADR-0126/0127) | **absent** (verified: 정규직/계약직/파견/EmploymentClassification/52시간 = 0; `4대보험` = 1, `employment classification` = 5 generic) | **HIGH** | RECOVER-as-ADR. Concrete lost product context; matches legacy-recovery #1 (KR HR/payroll packs). The typed enum drives the entire payroll wedge. |
| **KR-first regulatory DNA (statute depth)** | Every vertical cites KR statute: 근로기준법, PIPA, 통신비밀보호법, MFDS/의료법, 국가계약법, 4대보험, KCMVP (ARIA/LEA not bare AES), 52시간제 (ADR-0104/0126/0127/0136/0137/0140/0190) | **weaker** (compliance/regional packs exist + canon D11 restores KCMVP/KISA from corruption; the **home-market-FIRST posture + per-vertical statute grounding** is the single most churned context) | **HIGH** | Feed Task #18 + the KR regional-pack doc: re-thread the statute citations per vertical. This is the DNA that motivated the whole product set. |
| **KR marketplace payment rails** | PortOne (IMP) PG aggregator → Toss primary; double-entry settlement ledger; KYB via NTS; 전자상거래법/통신판매업자 registration; 전자금융업 path (ADR-0135/0203/0120/0124) | **weaker** (verified: `Toss` 3, `전자상거래` 3, `통신판매` 1, `double-entry` 1 — partial; `PortOne` = 0, the PG-aggregator→Toss-primary + 전자금융업 license ladder not carried) | **MED** | FOLD into the marketplace/payments ADR: restore the PG-rail specifics + the license-ladder boundary (software-over-regulated-partners). |
| **Data-ownership pillars + cross-pillar join prohibition** | Org vs Person pillars; immutable `ownership_pillar` on every event/property/feature/consent/audit; cross-pillar join PROHIBITED at policy layer; worker-rights override (PIPA 22-2/37) (ADR-0132) | **present** (verified: `ownership_pillar` 2, `cross-pillar` 3, `ownership pillar` 1 — the invariant IS carried; thinner than bominal's full treatment) | **LOW** | CONFIRM-only: the invariant survives. Optionally strengthen the worker-rights-override clause. Not a recovery. |
| **Persona tier model T1–T4 (data-collection axis)** | T1 marketable / T2 collectable-non-marketable / T3 anon-aggregate / T4 non-collection; Kantara consent receipts; T4 enforced at ingest (ADR-0131) | **present** (verified: `persona tier` 12, `consent receipt` 7 — CARRIED; `Kantara` = 0, the specific consent-receipt standard is the only lost detail) | **LOW** | CONFIRM-only. Digest D1 worried oyatie's `tier` namespacing (canon D12) orphaned this — live read shows persona-tier survived. Optionally re-cite Kantara. NOT a loss. |
| **CDSS/manufacturing/CCTV safety-gate doctrines** | HITL-non-negotiable (CDSS), OT-no-closed-loop-actuation (mfg/AMR), biometric-default-off (CCTV), prohibited-autonomous-lethal-use (defense) | **mixed** (defense lethal-use guard present in the 37 defense ADRs; the **CDSS/mfg/AMR safety gates are absent with their specs** — see A/B) | **HIGH** | Bundled with the A/B spec recoveries — the safety gate is the load-bearing part of each spec, not separable. |
| **AI surfaces catalog (~17 intelligence domains)** | Method-tagged enumeration; "some are pure algorithmic" — OR/MIP/graph before ML where it fits (ADR-0144) | **weaker** (breadth-of-AI-surface enumeration intent not stated) | **LOW** | FOLD into the intelligence/data-AI-governance doc as a capability catalog. Low intent-loss. |

---

## E. NAMING-CONTINUITY broken (rename map unclear / ambiguous)

| Item | What it was (bominal ref) | oyatie status | Severity | Restore recommendation |
|---|---|---|---|---|
| **Six business-arm taxonomy** | Healthcare / Corporate SaaS / FinTech / Platform-Ops / Hospitality+Lifestyle / Communications-Connect, each with Tier-0 foothold/Tier-1 wedge/Tier-2+ deepening per sub-vertical (ADR-0203/0185) | **absent as a taxonomy** (verified: `business arm`/`six arm` = 0; ADR-0058/0060 explicitly RETIRED "Arms" to sales/GTM labels + flat catalog) | **MED** | This is a DELIBERATE rename (flat-catalog doctrine, canon-adjacent), NOT an accidental loss — but the **per-arm wedge map (which sub-vertical is the Tier-1 entry) is lost** and IS Task #18's deliverable. Recover the wedge-map content, not the "Arm" grouping. |
| **`connect` super-app → 8 standalone svcs** | Connect = native dual-context comms shell (Mail/Messenger/Community); rebuild-Slack/Discord/Signal/KakaoTalk natively; per-context encryption (Personal E2EE user-DEK vs Professional tenant-DEK + four-eyes) (ADR-0208/0215) | **renamed (clear) + weaker (ambition)** (ADR-0237/0238 dissolved Connect into `mail`/`messenger`/`calendar`/… — rename HIGH-confidence; but the **native-rebuild-of-10-platforms ambition + symmetric per-context encryption matrix** is the churned intent) | **MED** | Rename is clean (don't re-thread). FOLD the per-context encryption matrix + native-rebuild ambition into the comms-product doc; ADR-0311 dual-context exists but is thinner. |
| **Object Graph → `ontology` (anti-Palantir naming)** | "Object Graph = Palantir-Ontology-but-better; NEVER call it ontology"; 5 differentiators (engine-RLS, Merkle audit, rule-packs-as-primitives, portable semantics, multi-renderer) (ADR-0106/0192) | **renamed (clear) — irony noted** (oyatie's canonical substrate is literally named `ontology`, the exact name bominal forbade; ADR-0060 glossary states the rename. Substrate PRESENT+stronger; the 5-differentiator framing weaker) | **LOW** | CONFIRM-only. Rename is intentional + documented. Optionally restate the 5 differentiators in the ontology doc. The "never call it ontology" rule is moot/superseded. |
| **`manufacturing`/`logistics` → SAP-shaped flat services** | Coarse bominal modules → `production-planning`+`quality-management`+`plant-maintenance` (mfg); `warehouse`+`supply-chain-planning` (logistics) (ADR-0315 §D-1) | **renamed (clear, HIGH-confidence)** — ADR-0315 §D-1 has an explicit destination column | **LOW** | CONFIRM-only. Clean documented decomposition. The lost part is the **Manufacturing-AI layer on top** (category B), not the ERP decomposition. |
| **`medical` → `emr` (+ 7 clinical svcs)** | ADR-0332 healthcare decomposition | **renamed (MED-confidence)** — inferred from SAP-code + README, not a per-service "X→Y" line | **LOW** | CONFIRM the `medical`→`emr` mapping line; mostly fine. |
| **"Bench" host-surface naming** | Bench = unified host surface (renames "shell"); owns login + module activation per tenant module-graph (ADR-0130/0121/0123) | **present** (verified: `Bench` 63 ADR hits — the concept + naming SURVIVED, contra digest C2's "WEAKER/ABSENT" estimate) | **LOW** | CONFIRM-only. NOT a loss; corrected from digest. |

---

## F. Counts per category (post-verification)

| Category | HIGH | MED | LOW | Total rows |
|---|---|---|---|---|
| A — Product/module scope | 2 | 2 | 4 | 8 |
| B — Vertical scope | 4 | 4 | 1 | 9 |
| C — Roadmap/sequencing | 2 | 2 | 1 | 5 |
| D — Decision/rationale | 3 | 1 | 3 | 7 |
| E — Naming-continuity | 0 | 2 | 4 | 6 |
| **Total** | **11** | **11** | **13** | **35** |

---

## G. Where oyatie is genuinely STRONGER (migration improved it — do NOT "restore")

- **Own-everything ratchet** (canon D-META/D4): bominal ADR-0150's Rust-first-sovereignty is the
  SEED; oyatie generalized it to own-the-endpoint/vendor-the-bridge across the whole stack.
- **Data tier**: oyatie owns the entire tier via vendored→owned ratchet (514 ADRs of substrate
  detail) vs bominal's transition matrix.
- **Audit-chain / RLS tenancy / Cedar**: PRESENT and stronger (Merkle-Ed25519 explicit; FORCE-RLS;
  Cedar-as-permanent-contract with owned PARC engine).
- **Healthcare/ERP/Connect decompositions**: bominal's coarse modules → clean single-concern
  services (ADR-0332/0315/0321/0237/0238) — a genuine architecture improvement.
- **Cloud-as-dogfood-substrate** (canon D-LAYER): the `cloud/` 26-svc IaaS/PaaS tier products run
  on as tenant workloads — past anything bominal had.
- **Builder-OS / Portfolio plane / Proof-Ladder / Maturity-at-Full-Scale program**: present and
  operationalized (live GitHub backlog), stronger than bominal's docs.
- Far-future verticals (defense/public-sector/datacenter/agriculture) already captured as ADRs —
  the canon-D9 capture mostly landed.

---

## H. Feed-forward (where each loss goes — no new workflow needed)

- **Task #18 (vertical-coverage map):** the named-spec recoveries (CDSS, Manufacturing-AI, AMR,
  bid-pricing, optimization platform), the powergrid OT-depth, capital-markets ladder, and the
  six-arm wedge-map content. Plus the SEQUENCE + per-vertical M0-gate milestone shape (category C).
- **Task #19 (founder interview on bominal restoration):** the HIGH-severity rows are the
  interview agenda — recover-as-ADR vs intentionally-dropped is a founder call, NOT blind recovery
  (per canon D-LINEAGE). Lead the agenda with the 11 HIGH rows.
- **Legacy-recovery register (#1 KR packs, #2 First Proof Slice, #3 4-lane closure):** category D's
  KR employment enum + statute DNA reinforce #1; category C's M3 launch target reinforces #2.
- **Masterplan backfill (canon D1):** category C's infra-sovereignty schedule + 54-milestone
  sequence are the dated roadmap the generated masterplan currently lacks.

---

## I. Caveats

- All status flags spot-verified by grep against the live ADR corpus + service tree on 2026-06-06;
  this corrects the upstream digests' explicit "estimates keyed off the canon doc, not a fresh
  live read." Where corrected, noted inline.
- Grep is presence-detection: a 0-hit is strong evidence of absence for a distinctive term
  (낙찰하한율, SGP4, SLAM, CDSS); a nonzero hit confirms presence but not depth/quality — the
  "weaker" rows still owe a content read before any recovery ADR is authored.
- bominal ADR IDs (0010–0233) collide with oyatie's ID space; under canon D13 every recovered
  decision re-enters as a clean ADR-0000+ record, never by bominal's old number.
- Did NOT re-surface the 16 founder-ruled canon decisions nor duplicate the 7 .Trash recoveries;
  touch-points are cited to locate the bominal-side authoritative source only.
