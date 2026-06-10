# ADR Audit Artifact — source-36

- **side:** SOURCE (`~/Developer/source`, `jason931225/oyatie`)
- **chunk:** source-36 (slice 246–252 of `ls -1 docs/decisions/ADR-*.md | sort`)
- **range:** ADR-0309 → ADR-0315
- **ADRs reviewed:** 7 (0309, 0310, 0311, 0312, 0313, 0314, 0315)
- **auditor posture:** READ-ONLY; masterplan = GENERATED from the immutable ADR log (ADR-0364/0365 doctrine). Keystone map (`_map/canonical-posture-and-supersession-map.md`) used as the supersession/retired-vocab baseline.
- **theme of chunk:** This is the **Wave-3-E / Wave-3-G enterprise + civil-rights cluster** — a tightly cross-referencing bundle (the 2026-05-20 "foundational-doctrine" keystone bundle + the ERP-parity scaffold). All 7 are `status: Proposed`, all dated 2026-05-20, all author-clean (Cedar-gated, tenant-scoped, ADR-0244/0243 anchored). None is superseded; none appears in the keystone supersession graph. The dominant audit question is therefore **RATIFY vs DROP for the Proposed bundle**, plus three genuine founder-scope questions (civil-rights ML breadth, dual-tenant labor-law surface, full SAP-parity ambition).

---

### ADR-0309 — Detection Fairness + Civil-Rights Compliance Baseline

- **decision_atom:** Every production ML/detection model passes a substrate-level five-invariant fairness gate (no-proxy-discrimination, ±2pp per-class TPR/FPR equity, 4/5ths disparate-impact, explainability floor, per-jurisdiction model variants incl. EU-AI-Act-Art.5 refusal) before serving adverse-action traffic.
- **domain:** authz-policy (primary) / intelligence-ai (cross-cutting) — also touches compliance-residency.
- **current_status:** Proposed (enforcement `advisory-until-2026-09-15-blocker-thereafter`).
- **disposition:** KEEP (ratify). Sound, well-anchored, non-conflicting; the only edit-worthy detail is naming drift (see truth_flag/AMEND note).
- **proposed_resolution:** **RATIFY.** It is the binding ADR for documentation-rigor §3.2.6 invariant set; dropping it ships an EU-AI-Act/ECOA/Fair-Housing violation by construction. Ratify, but require the naming-residue amendment below as a condition.
- **governing:** n/a (not superseded).
- **truth_flag:** PARTIAL — substance TRUE, but carries **retired-vocab leakage**: naming_justifications still use `oya-governance-detection-fairness-*` with `layer: N/A (foundry-fitness CI lane)` and `bnf_segments: oya.foundry-fitness.*`. Per keystone §2 + ADR-0347, the live CI-lane prefix is `oya-governance-*` and `foundry-fitness` BNF is retired brand residue. Crate names are fine; the `oya.foundry-fitness.*` BNF segments are stale.
- **in_masterplan:** YES (planning binding — substrate gate with BLOCKER promotion date and SLO floors; clearly carries planning_impact).
- **tensions:** None hard. Soft tension with LINUX ADR-0021 (owned compile-to-Rust policy language) only insofar as fairness gates would eventually compile into that engine; not a conflict (both are policy-as-substrate). `Redis`-free here (good).
- **hyperscaler_challenge:** ALIGNED. Google (Vertex Responsible-AI), Microsoft (Fairlearn-gated), Stripe Radar all run exactly this quarterly-fairness/per-class-equity/explainability shape. A hyperscaler WOULD make this decision; the ±2pp hard threshold + 4/5ths gate is industry-conventional. Argues for KEEP, not archive.
- **ai_slop / refinement / consensus_needed:** Not slop — dense but every clause is regulator-anchored. Refinement: collapse the `foundry-fitness` BNF to `governance`. **Founder question:** is ±2pp the universal substrate floor, or should it be per-pack configurable from day 0 (HUD housing arguably demands tighter)? Low-stakes; ratify with a parameterization note.

---

### ADR-0310 — Investigation Case-Management Substrate

- **decision_atom:** All detection signals, appeals, and regulator inquiries route through one substrate case-management workflow (triage→investigate→escalate/dismiss/adjudicate→feedback→retain) with Merkle-sealed chain-of-custody (ADR-0028), Cedar-gated PII access (ADR-0243), and per-pack regulator-surface emission.
- **domain:** security-supplychain (trust-&-safety/investigation) (primary) / governance-process (cross-cutting) — also observability (10 new audit-event classes) and compliance-residency.
- **current_status:** Proposed (enforcement `advisory-until-2026-10-15-blocker-thereafter`).
- **disposition:** AMEND (then ratify). Decision is sound and KEEP-grade, but it carries two concrete drifts that must be fixed before it feeds the masterplan (see truth_flag).
- **proposed_resolution:** **RATIFY** the doctrine; AMEND the stale substrate refs. Without it, "appeals route nowhere" — it is the sink for ADR-0309's appeal-links and ADR-0308's appeal mechanism. Drop is not viable.
- **governing:** n/a.
- **truth_flag:** PARTIAL — TRUE doctrine, but **two retired-substrate references**: §C.3/§C.5 scalability+optimization text says "Kafka topic `audit.investigation.case.opened`" and "**Redis** Cluster shard increase / Redis-cached investigator-pool". Per keystone §2: Kafka→**Pulsar 4.x + Oxia** (ADR-0377-kafka-to-pulsar, sup. ADR-0005) and Redis→**Valkey** (ADR-0336). Both are STALE substrate names. Also `oya-governance-investigation-*` lanes carry the retired `oya.foundry-fitness.*` BNF (same as 0309).
- **in_masterplan:** YES (substrate primitive, BLOCKER SLOs, regulator-emission cadence — strong planning binding).
- **tensions:** None ADR-level. Internal consistency tension: it cites both Postgres read-replica fan-out AND Citus/Kafka/Redis in the same breath — the LINUX own-DB posture (ADR-0001 "eliminate PostgreSQL") would object, but that is the known fault-line #1, not a 0310-specific defect.
- **hyperscaler_challenge:** ALIGNED. Splunk SOAR, Palo Alto XSOAR, Google Chronicle/Siemplify, AWS Security Hub all ship exactly this centralized case-management + analyst-label→ML-retrain loop. A hyperscaler WOULD build this. The Kafka/Redis naming is the only thing a hyperscaler-2026 reviewer would flag → argues for AMEND, not archive.
- **ai_slop / refinement / consensus_needed:** Not slop. Refinement: swap Kafka→Pulsar, Redis→Valkey, `foundry-fitness`→`governance` BNF. No founder question; ratify post-amend.

---

### ADR-0311 — Dual-Tenant Identity (Personal-vs-Work Boundary)

- **decision_atom:** One passkey identity may hold N tenant memberships (≤1 personal); a hard Cedar default-deny boundary forbids any employer/work-tenant principal from reading personal-tenant surfaces except via self-access or a court-warrant grant, with per-µservice row-level ownership declaration, mandatory UI tenant-context indicator, and onboarding-consent/offboarding-portable-export handshake.
- **domain:** tenancy (primary) / identity-authn (cross-cutting) — amends ADR-0244 (adds 4 `audience_type` enum values).
- **current_status:** Proposed (`advisory-until-2026-09-15-blocker-thereafter`); `amends: ADR-0244`.
- **disposition:** AMEND (then ratify). KEEP-grade doctrine; one residue + one scope note.
- **proposed_resolution:** **RATIFY.** Load-bearing for 25 Wave-3-E journeys and for ADR-0312/0313 (which build on its boundary). The `amends ADR-0244` enum extension is additive/non-breaking and correctly declared. Drop would re-scatter the boundary into 45 ad-hoc per-µservice implementations.
- **governing:** n/a.
- **truth_flag:** PARTIAL — TRUE, but the §D-3 ownership-map table still lists a `foundry` µservice row ("`foundry` | PLATFORM_OWNED with per-tenant scope | Foundry meta-trust per ADR-0293") and a `cell` µservice row. Per keystone §2 the **foundry brand is RETIRED** (→ intelligence/governance per ADR-0335/0347) and **cell-as-µservice is retired** (→ pattern only, ADR-0333). The `intelligence` row is correctly present alongside, so this is residue, not a live contradiction. Also `&& principal.tenant_access_permits` / `oyatie.foundry.ci-agent` sub-scope example in §C.3 is retired-vocab.
- **in_masterplan:** YES (amends a keystone tenancy ADR; 45-µservice rollout; BLOCKER lanes).
- **tensions:** Mild with ADR-0329 (tenant-class vs tier): 0311 correctly uses `audience_type` (B2C/B2B sub-tiers), NOT the retired tenant tier-system — no conflict, but an auditor should confirm `B2C_*`/`B2B_*` sub-tiers don't reintroduce `tier`-keyed billing. Companion-coupled to ADR-0312 (warrant) and ADR-0313 (conglomerate) — all three must ratify together.
- **hyperscaler_challenge:** ALIGNED — explicitly models Apple (Personal Apple ID vs Business Manager / User Enrollment), Microsoft (MSA vs Entra work/school), Google (personal vs Workspace), Slack Grid, Stripe Connect. This is verbatim the hyperscaler dual-identity pattern; a hyperscaler WOULD make this decision. Argues KEEP.
- **ai_slop / refinement / consensus_needed:** Not slop. Refinement: purge `foundry`/`cell`-as-service rows from the §D-3 map (rename to intelligence/governance + cell-as-pattern). **Founder question:** the per-jurisdiction labor-law overlay enumerates ~11 jurisdictions (US/KR/EU/JP/UK/AU/CA/SG/IN/BR/CN) as day-0 surface — is that the intended launch breadth, or should it be a pack-overlay ratchet (start US+KR+EU, add on demand)? Scope-sizing question, not a correctness one.

---

### ADR-0312 — Court-Warrant Scoped Piercing

- **decision_atom:** The ONLY path through the ADR-0311 personal-tenant boundary is a time-limited, scope-bounded Cedar `grant_kind="court_warrant_scoped"` CrossTenantGrant, gated by judicial-authority + statutory-anchor validation, mandatory SEV2+ ombudsman attestation, Merkle-sealed chain-of-custody, cross-jurisdiction higher-restriction-wins (ADR-0304), warrant-canary transparency reporting, and bulk/over-scope/reporter-privilege refusal.
- **domain:** authz-policy (primary) / governance-process (cross-cutting) — also compliance-residency, identity-authn.
- **current_status:** Proposed (`advisory-until-2026-09-15-blocker-thereafter`).
- **disposition:** KEEP (ratify). Clean, no retired-vocab residue spotted; the rare ADR in this chunk with no substrate drift.
- **proposed_resolution:** **RATIFY.** Without it, ADR-0311's boundary is either too rigid (contempt of court) or too leaky (no scope review). It is the necessary companion; drop is incoherent.
- **governing:** n/a.
- **truth_flag:** TRUE. Well-formed (state machine, refusal enum, per-jurisdiction scope table, transparency-report JSON-Schema). Note: warrant volume math is honestly bounded (≤100/yr at GA vs Google's ~163k H2-2024) — realistic, not slop. Minor: audit-event class list in §C.2 names `WarrantOmbudsmanAttested`/`WarrantGrantUsed`/`WarrantGrantExpired` that aren't all mirrored in the front-matter naming_justifications — a small completeness gap, not a truth defect.
- **in_masterplan:** YES (the sole pierce mechanism; BLOCKER lanes; transparency-report spec).
- **tensions:** Forge/identity fault-line indirectly: assumes a `governance` µservice with `warrant-intake/` subdir (correct — governance is the live name, not foundry). Cross-references ADR-0300 (whistleblower) + ADR-0304 (jurisdiction) — both live. No contradiction.
- **hyperscaler_challenge:** ALIGNED — built directly on Microsoft-Ireland/CLOUD-Act, Twitter-v-Harris, Apple/Google/Microsoft/Cloudflare transparency-report + warrant-canary precedent. This is THE industry-standard warrant-handling shape. A hyperscaler WOULD (and legally must) make this decision. Strongly KEEP.
- **ai_slop / refinement / consensus_needed:** Not slop — arguably the highest-quality ADR in the chunk. No founder question; ratify as-is. Optional refinement: reconcile the §C.2 audit-event list with front-matter.

---

### ADR-0313 — Conglomerate-Tenant Hierarchy (Sovereign-Child + Policy-Mediated Controlling-Entity Grant)

- **decision_atom:** Parent/subsidiary (holding-company, JV, chaebol/keiretsu, marketplace-platform) relationships are modeled as fully-sovereign ADR-0244 child tenants plus Cedar permits against a `conglomerate_grants` source-of-truth table, so restructuring (spinoff/divestiture/IPO/bankruptcy/JV) is a 1-step Cedar revoke+grant — never a data migration — bounded by six invariants (no transitive auto-include, residency-preserved, personal-tenant boundary held, corporate-governance attestation, cross-child information barrier, audit dual-seal).
- **domain:** tenancy (primary) / authz-policy (cross-cutting) — amends ADR-0244 (adds `controls_tenants`/`controlled_by_tenants` denorm columns + `conglomerate_grants` table).
- **current_status:** Proposed (`advisory-until-conglomerate-substrate-lands`; BLOCKER post-2026-07-15 per keystone bundle).
- **disposition:** AMEND (then ratify). Architecturally KEEP-grade; carries retired-substrate residue.
- **proposed_resolution:** **RATIFY** the doctrine; AMEND substrate names. It is explicitly required before any holding-co/multi-subsidiary/platform-of-platforms customer onboards at production scale — high planning value. Drop not viable.
- **governing:** n/a.
- **truth_flag:** PARTIAL — TRUE doctrine, but **stale references**: (a) §C.3 scalability cites "Valkey, 1s TTL — per **ADR-0243** §D-7" in one place (good) but also "per-cell **Valkey** provisioned tier per **ADR-0046**" where ADR-0046 is the **Superseded** vector-store ADR (→ ADR-0192 Milvus) per keystone §1.1 — wrong cross-ref. (b) §A.5 lists oyatie sub-tenants as `oyatie.foundry`, `oyatie.ops`, `oyatie.engineering`, `oyatie.intelligence` — `oyatie.foundry` is retired-brand residue (should be `oyatie.intelligence`/`oyatie.governance` per ADR-0335). (c) related-ADR front-matter mislabels several refs (e.g. `ADR-0028-cloud-microservice-architecture` vs the canonical `ADR-0028-audit-chain-merkle-sealed`, `ADR-0299-cross-pack-data-residency` vs `ADR-0299-account-recovery`) — internal cross-ref drift to flag.
- **in_masterplan:** YES (amends ADR-0244; new SoT table + Cedar entity-type + 6 CI lanes; strong binding).
- **tensions:** Inherits fault-line #1 (own-DB vs Postgres/Citus): the migrations are deeply Postgres+Citus-specific (RLS, `create_distributed_table`), which LINUX ADR-0001 would reject. Surface, don't resolve. Companion-coupled to 0311/0312. Forward-references ADR-0319 (front/middle/back-office information barrier) which is out of this chunk.
- **hyperscaler_challenge:** ALIGNED — adopts AWS Organizations, Microsoft MTO, Google Cloud Org→Folder→Project, Stripe Connect facilitator, Salesforce multi-org, Okta hub-and-spoke verbatim. The sovereign-leaf + policy-mediated-parent pattern is exactly what all nine cited hyperscalers converged on. A hyperscaler WOULD make this decision. KEEP.
- **ai_slop / refinement / consensus_needed:** Not slop — strong, with real regulatory grounding (chaebol/keiretsu/Konzern, Glass-Steagall/MiFID ring-fences). Refinement: fix the ADR-0046 mis-cite, the `oyatie.foundry` sub-tenant name, and the related-ADR front-matter title drift. No founder question beyond the shared own-DB fault-line.

---

### ADR-0314 — Marketplace as Universal Deal-Settlement Substrate

- **decision_atom:** Every tenant-to-tenant and tenant-to-consumer commercial exchange (goods, services, subscriptions, capability grants, workforce, M&A/JV, data licenses, receivables) is expressed as one tenant-scoped, Cedar-gated `DealSet` envelope settled via payments/treasury/finops — one settlement primitive, not one deal-table per ERP module.
- **domain:** marketplace-commerce (primary) / api-contracts (cross-cutting; OpenAPI 3.2.0/AsyncAPI 3.1.0/proto3 + ontology projection).
- **current_status:** Proposed (extends ADR-0249 multi-category-marketplace).
- **disposition:** AMEND (then ratify). The CORE decision (DealSet as universal settlement primitive) is KEEP-grade and clean. The document body, however, is **padded with ~20 near-identical `### §D-X Deal primitive:` blocks** (retail-order, purchase-requisition, goods-receipt, … each a copy-paste template with the same six rules). That is the one genuine AI-slop signal in the chunk.
- **proposed_resolution:** **RATIFY** the §B decision; AMEND to collapse the repetitive §D-X template blocks into one parameterized table (they add no decision content). The settlement primitive is required for ADR-0315 ERP parity; drop not viable.
- **governing:** n/a.
- **truth_flag:** PARTIAL — decision TRUE; body PARTIAL/borderline-GARBAGE in the §D-X section (≈20 stamped-out identical blocks differing only by the deal-type noun; "SAP Ariba or Coupa covers procurement/spend shape; Stripe or Salesforce Commerce Cloud covers platform-facilitated settlement shape as applicable" repeated verbatim per block). Front-matter is thin (single `owner` line, no naming_justifications, no keystone_bundle) vs the rich 0309–0313 front-matter — a quality step-down.
- **in_masterplan:** YES (settlement primitive feeding ERP parity; enforced_by `oya-governance-marketplace-deal-settlement-coverage`).
- **tensions:** With ADR-0132 (no-grouping): 0314 is careful to say it is NOT an ERP-suite µservice and ownership stays distributed (marketplace/payments/treasury/finops/ontology/workflow-engine/connector/global-trade) — consistent. With ADR-0329 tenant-class: 0314 keeps deals tenant-scoped (good). Mild concern: the breadth of DealSet (M&A + receivables-factoring + JV-capital-call as "marketplace" settlement) is an enormous scope claim resting on a thin ADR.
- **hyperscaler_challenge:** QUESTIONABLE (on scope, not pattern). The DealSet-as-universal-envelope idea echoes SAP Ariba / Stripe Connect / Coupa / Salesforce Commerce — but no single hyperscaler unifies retail + procurement + M&A + receivables + JV under ONE settlement object; they ship separate products. A hyperscaler would likely make a *narrower* version of this decision (commerce/payments settlement) and NOT fold M&A/JV/receivables into "marketplace." Argues for AMEND (tighten scope claim + de-slop the body), not archive.
- **ai_slop / refinement / consensus_needed:** **Yes, partial slop** — the §D-X block farm. Refinement: collapse to a single deal-category table (the §D-2 table already does this well; the §D-X blocks are redundant). **Founder question (contested):** Is "marketplace = universal settlement substrate for M&A, JV capital calls, and receivables factoring" a real day-0 commitment, or is that over-claiming a retail/procurement marketplace into corporate-finance territory? This is the sharpest "is-this-truly-needed-in-the-masterplan" question in the chunk.

---

### ADR-0315 — ERP Coverage Doctrine (SAP-Parity Goal)

- **decision_atom:** oyatie targets full SAP S/4HANA module parity *through composition* across flat single-concern microservices (never a `microservices/erp/` monolith), mapping each SAP module to existing services or a minimal set of nine new Wave-3-G services (production-planning, quality-management, plant-maintenance, warehouse, real-estate, crm, treasury, supply-chain-planning, global-trade).
- **domain:** marketplace-commerce (ERP/enterprise) (primary) / governance-process (the module-map governance doctrine) — also product-ux.
- **current_status:** Proposed ("authorizes the nine Wave-3-G microservice scaffolds").
- **disposition:** AMEND (then ratify) — OR escalate to founder. The composable-vs-monolith principle is KEEP-grade and consistent with ADR-0131/0132. But it **authorizes nine new microservice scaffolds** as a side-effect of an `Proposed` doctrine ADR — that is a large, irreversible-ish footprint commitment riding on a not-yet-ratified ADR.
- **proposed_resolution:** **RATIFY the doctrine, but gate the nine-scaffold authorization.** The "ERP via composition, no `erp/` monolith" rule is correct and should be ratified. However, ratifying ADR-0315 as written silently green-lights 9 new live µservices — recommend RATIFY-WITH-CONDITION (doctrine accepted; the 9 scaffolds require a separate explicit founder go/no-go, since each is a multi-quarter build). NOT a DROP — the doctrine is sound and the masterplan needs an ERP-coverage position.
- **governing:** n/a.
- **truth_flag:** PARTIAL — doctrine TRUE; one **vocab flag**: §A.2/§B use "**capability tier**" / "Each SAP module maps to a **capability tier** across microservices." Per keystone §2, `capability-tier` is a RETIRED tenant-tier synonym (ADR-0316→ADR-0329). Here it is used loosely to mean "capability map," NOT tenant billing tiers, so it is *probably* benign — but it is exactly the retired token and should be reworded to avoid reviving `tier` vocabulary. Front-matter is thin (single `owner` line) like 0314.
- **in_masterplan:** YES (declares 9 new services + a module-parity map; `oya-governance-erp-parity-module-map` + `oya-governance-no-grouping` lanes; very strong planning_impact).
- **tensions:** With ADR-0132 (no-grouping): 0315 is *explicitly* compliant ("no `microservices/erp/`, no ERP platform folder") — consistent. With the founder GOAL ("if it isn't needed for the masterplan, it isn't needed"): full SAP S/4HANA parity (FI/CO/MM/SD/PP/QM/PM/HCM/PS/PLM/EHS/SRM/CRM/SCM/GTS/TM/EWM/TRM/RE-FX/IS-*) is a **vast** scope — the single biggest breadth claim in the entire chunk, and arguably in tension with a "minimal/secure viable Linux-parity kernel FIRST" sequencing (the cloud/k8s side is supposed to be the *later* optimization per the project memory's GOAL ORDER).
- **hyperscaler_challenge:** QUESTIONABLE→MISALIGNED on timing/scope. The *composable-ERP* architecture is defensible (it is how Workday/NetSuite/Dynamics decompose internally), and "no ABAP clone, use SDK+Workflow+Cedar+ontology" is smart. BUT no hyperscaler attempts full SAP-S/4HANA-parity as a *doctrine commitment* before its core platform exists — they buy/acquire (Microsoft↔Dynamics, Oracle↔NetSuite) and sequence over a decade. A hyperscaler would NOT commit to 21-module SAP parity at this stage. Argues for AMEND (keep the doctrine, defer/sequence the parity ambition + the 9 scaffolds).
- **ai_slop / refinement / consensus_needed:** Partial slop — the §D-1.A "Detailed module notes" repeat the same five boilerplate rules (Coverage/Audit/Migration/Parity-gap) for all ~21 modules, and the §D-2 "naming justification" blocks repeat an identical template per service (same pattern as 0314's §D-X farm). Refinement: collapse the per-module boilerplate; reword "capability tier". **Founder question (contested, highest-stakes in chunk):** Is full SAP S/4HANA module parity an actual masterplan commitment now, and should this `Proposed` ADR be allowed to authorize 9 new live microservices — or is ERP a post-core-platform "opt" per the stated GOAL ORDER (Linux-parity kernel first)? This needs an explicit founder ruling before RATIFY.

---

## Chunk notes

**Bundle character.** Source-36 is two coherent, deliberately-coupled clusters authored same-day (2026-05-20), all `Proposed`:
- **Civil-rights / trust-&-safety / boundary cluster (0309–0313):** detection-fairness → investigation-case-mgmt → dual-tenant-boundary → court-warrant-pierce → conglomerate-hierarchy. These are mutually-reinforcing companions (0310 is the sink for 0309's appeals; 0312 is the only pierce of 0311's boundary; 0313 builds on 0311/0312). They should **ratify or drop as a set**, not piecemeal — dropping any one leaves a dangling reference in the others.
- **Enterprise/ERP cluster (0314–0315):** DealSet settlement primitive → SAP-parity coverage doctrine. 0315 depends on 0314's DealSet.

**No supersession events.** None of the 7 appears in the keystone supersession graph (§1) or retired-vocab table as a *governing* or *retired* ADR. All carry `supersedes: [] / superseded_by: []`. So there are **no ARCHIVE/SUPERSEDE/MERGE dispositions** in this chunk — only KEEP/AMEND and the Proposed→RATIFY/DROP question. Recommendation: **RATIFY all 7** (with the per-ADR amendments below); **DROP none** — every one carries genuine planning_impact and none is redundant or contradicted.

**Retired-vocabulary leakage is the dominant defect** (consistent with keystone §2 "lint signal"). Across the chunk:
- `oya.foundry-fitness.*` BNF segments in naming_justifications (0309, 0310, 0311) — should be `oya.governance.*` per ADR-0347.
- `foundry` as a live µservice/sub-tenant (0311 §D-3 ownership map; 0313 §A.5 `oyatie.foundry`) — retired brand per ADR-0335 (→ intelligence/governance).
- `cell` as a µservice (0311 §D-3) — retired per ADR-0333 (cell = pattern only).
- **Kafka** topic + **Redis** Cluster/cache (0310 §C.3/§C.5) — should be **Pulsar+Oxia** (ADR-0377) and **Valkey** (ADR-0336).
- "**capability tier**" (0315 §A.2/§B) — retired `tier` token (ADR-0316→0329); reword to "capability map."
- Stale cross-ref: 0313 cites **ADR-0046** (a Superseded vector-store ADR) for a Valkey tier, and mislabels several related-ADR titles in front-matter.

None of these change the *decision*; all are AMEND-grade naming/ref fixes that should be applied before these ADRs feed the generated masterplan (since the masterplan inherits ADR front-matter verbatim under the ADR-0364/0365 generated-from-ADRs model).

**AI-slop signals** (the only two in the chunk): ADR-0314 §D-X (~20 copy-paste deal-primitive blocks) and ADR-0315 §D-1.A + §D-2 (per-module / per-service boilerplate template farms). These are the lower-front-matter-quality pair (single-`owner` line, no keystone_bundle, no naming_justifications) vs the dense, well-structured 0309–0313. De-slop by collapsing repeated blocks into the tables that already exist (§D-2 for 0314, §D-1 for 0315).

**Two genuine founder consensus questions** (beyond naming):
1. **ADR-0314 scope:** does "marketplace = universal settlement substrate" legitimately include M&A, JV capital calls, and receivables factoring, or is that over-claiming a commerce marketplace into corporate-finance? (questionable vs hyperscaler practice — no one unifies these under one object).
2. **ADR-0315 timing/scope (highest-stakes):** is full SAP S/4HANA module parity a real masterplan commitment now — and may a `Proposed` ADR authorize 9 new live microservices — given the stated GOAL ORDER ("minimal viable Linux-parity kernel FIRST, cloud/ERP as later opt")? This squarely tests the founder's "if it isn't needed for the masterplan, it isn't needed" doctrine and should get an explicit ruling before ratification.

**Cross-side (LINUX) note:** ADRs 0313/0310's deep Postgres+Citus dependence sits on the wrong side of fault-line #1 (LINUX ADR-0001 "eliminate PostgreSQL" own-DB posture). Not a source-internal defect — flagged only so the keystone fault-line carries through on merge.
