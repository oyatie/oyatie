# ADR Audit Artifact — source-37

- **side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **chunk:** source-37
- **range:** slice 253–259 of `ls -1 docs/decisions/ADR-*.md | sort`
- **ADRs reviewed:** ADR-0316, ADR-0317, ADR-0318, ADR-0319, ADR-0320, ADR-0321, ADR-0322 (7)
- **auditor posture:** masterplan = GENERATED from the live ADR log (ADRs immutable SSOT); a decision not represented as a LIVE ADR is "not needed". Retired vocab per keystone map enforced (foundry→intelligence/governance, tier-system→tenant-class, cell-as-service→pattern, Redis→Valkey, Kafka→Pulsar, M0–M3/MVP).

> **Theme of this chunk:** This is the Wave-3-G "product-surface projection" cluster (0316–0321) plus a documentation-governance keystone (0322). The unifying TRUE idea across 0316/0317/0318/0319/0320/0321 is **projection over shared substrate, not product/role/workforce forks** — one tenant model, one Cedar gate, one ontology, one workflow engine, one UX-shell vocabulary, with named surfaces realized as activation/projection bundles. 0322 is the meta-rule that forces all of these documents (and their successors) to carry bespoke substance.

---

### ADR-0316 — Capability-Tier Over Product Fragmentation Doctrine
- **decision_atom:** Adjacent enterprise product categories (CRM, HR, ITSM, marketing, CLM, LMS, FP&A, etc.) are tenant-granted **capability tiers** — versioned projection bundles of Cedar permit sets + ontology projections + workflow templates + UX-shell + compliance overlays + cost metadata over shared flat services — and never become product-fragment microservices unless a candidate proves a distinct operational concern (the D-10 test).
- **domain:** product-ux / tenancy (cross-cutting: marketplace-commerce).
- **current_status:** Proposed (front-matter `status: Proposed`, `superseded_by: [ADR-0329]`, with a `supersession_note` saying ADR-0329 retires the doctrine but status stays Proposed until the Wave-15J migration lands).
- **disposition:** ARCHIVE (superseded).
- **proposed_resolution:** DROP — do not RATIFY as-is. ADR-0329 (Accepted) explicitly retires the "capability-tier" **vocabulary** (tier-system → tenant-class). The *anti-fragmentation projection mechanism* (permit-set + ontology-projection + workflow-template + UX-shell activation bundle) is the TRUE survivor and should be re-expressed under the tenant-class/projection vocabulary in the masterplan; the word "tier" must go. Why: keystone §2 retired-vocab table + §1.1 confirms 0316 archive.
- **governing:** ADR-0329 (tier-system retired → tenant-class; supersedes ADR-0316).
- **truth_flag:** PARTIAL — the projection-over-substrate mechanism is TRUE; the "capability-tier" naming is STALE/retired.
- **in_masterplan:** PARTIAL — mechanism belongs in masterplan (as tenant-class projection bundle); the "tier" label does not.
- **tensions:** (1) Direct vocab collision with ADR-0329 (tenant-class) — already adjudicated, 0316 loses the name. (2) Heavy downstream coupling: ADR-0321's entire Bronze/Silver/Gold/Platinum dossier mapping and ADR-0317/0318/0320's "capability tier" language all inherit 0316's retired term → they all carry retired-vocab leakage that AMEND must scrub. (3) The keystone autonomy-tier T1–T4 (policy axis) is a DIFFERENT live concept — do not conflate during the rename.
- **hyperscaler_challenge:** ALIGNED on mechanism, QUESTIONABLE on label. Salesforce (metadata-driven platform), ServiceNow (Now Platform app composition), Palantir Foundry (ontology projection) all compose product surfaces from shared primitives rather than per-product data planes — exactly 0316's thesis. A hyperscaler would make this decision. Implication: KEEP the mechanism, AMEND/ARCHIVE the "tier" surface naming under 0329.
- **ai_slop:** Low-moderate. The doctrine is substantive (D-1..D-10 are bespoke), but the C.1–C.6 "Risk/Mitigation" blocks are verbatim-duplicated across all six consequence dimensions (template-stamp smell — ironically the exact pattern ADR-0322 later bans). refinement: collapse duplicated risk paragraphs; rename out of "tier". consensus_needed: **"Confirm: the capability-tier *projection mechanism* survives ADR-0329's retirement under the tenant-class vocabulary — yes/no?"**

---

### ADR-0317 — Role-Based Projection + Unified UX Shell Doctrine
- **decision_atom:** One authenticated human gets multiple **role projections** — typed bindings of (Cedar permit set + ontology projection + workflow template library + UX shell + device/locale/a11y profile) per active role inside one tenant/sub-scope — that change authorization/visibility/affordances but never fork the underlying passkey identity, tenant primitive, Cedar gate, ontology, workflow engine, or design-token vocabulary; role switch is an audited first-class transition with a 500 ms p95 budget and a mandatory role-context indicator.
- **domain:** product-ux (cross-cutting: identity-authn).
- **current_status:** Proposed (`superseded_by: []`).
- **disposition:** AMEND.
- **proposed_resolution:** RATIFY (with amendments) — this is a sound, live, non-superseded doctrine with no governing retirement. Amend: (1) ADR-0316 is cited as "file absent in current checkout at authoring time" in `related`/§A.4 — that citation is now STALE (0316 exists, and is itself being archived under 0329); fix the dangling reference. (2) Scrub any "capability tier" inheritance to tenant-class vocabulary. Why ratify: the same-primitives invariant table is the genuine TRUE decision and underpins 0318.
- **governing:** n/a (not superseded).
- **truth_flag:** TRUE (core), with a STALE forward-reference to 0316.
- **in_masterplan:** YES — role projection + unified-shell + role-context-indicator + 500 ms switch SLO are masterplan-grade UX/identity invariants.
- **tensions:** (1) Self-admitted broken/absent ADR-0316 citation (authoring-time artifact) — now factually wrong, AMEND. (2) Overlap with ADR-0318 (collar-color/workspace universality) and ADR-0244/0311 — these compose rather than conflict, but masterplan must state 0317 owns *role* projection while 0318 owns *device/workspace* projection to avoid a turf seam.
- **hyperscaler_challenge:** ALIGNED. Apple Managed-vs-personal Apple Account, Microsoft work/school-vs-personal account separation, Google Workspace admin-managed accounts, Salesforce Lightning role home pages, ServiceNow portal-vs-workspace are all real precedents for "same human, different governed context, shared core identity." Google/MS/Apple all ship this. Implication: KEEP.
- **ai_slop:** Moderate-high structural repetition. The §D-1..§D-8 "Failure modes and controls" and "Role application examples" blocks are copy-paste identical across all eight primitives (the ROLE_CLINICIAN_NURSE…ROLE_INTERNAL_AUDITOR sextet repeats verbatim per section). refinement: factor the shared failure-mode/role-example boilerplate into one normative table referenced by each primitive. consensus_needed: none (decision is clear); refinement is editorial.

---

### ADR-0318 — Collar-Color and Workspace Universality Doctrine
- **decision_atom:** Oyatie is ONE universal platform across collar-color, workspace, device class, tenure, locale, and disability context: every user-facing microservice MUST declare device-profile and workspace-profile adapters + collar-color shells + accommodations as projections that can hide/reorder/summarize/stage information but MUST NOT change authorization, audit semantics, workflow identity, data class, tenant scope, or legal obligation — no per-workforce product forks.
- **domain:** product-ux (cross-cutting: compliance-residency via locale/age overlays).
- **current_status:** Proposed (`superseded_by: []`).
- **disposition:** AMEND.
- **proposed_resolution:** RATIFY (with amendments) — live, non-superseded, sound. Amend: (1) Like 0319, it explicitly notes "the specified ADR-0317 file was absent from the live repository … this ADR MUST be patched to replace the in-flight citation with the concrete path" — 0317 now exists, so execute that self-prescribed patch. (2) `length_cap: 2600` in front-matter sits oddly against ADR-0322's ≥800-line Tier-1/≥500 Tier-2 density floor and the file's 2,951 lines — reconcile. Why ratify: the four-axis (skill/workspace/tenure/locale) invariant + "projection, not fork" rule is a TRUE masterplan principle and the frontline/blue-collar universality is a genuine differentiator.
- **governing:** n/a.
- **truth_flag:** TRUE (core), STALE forward-reference to 0317.
- **in_masterplan:** YES — universality invariants (audit-event parity across projections, capability-equivalent accommodations, offline-first via ADR-0306) are masterplan-grade.
- **tensions:** (1) Self-flagged absent-0317 citation, now wrong → AMEND. (2) Device/workspace projection scope overlaps ADR-0317's role projection and ADR-0316's UX-shell — define ownership seam in masterplan (0318 = device+workspace+collar axes; 0317 = role axis; 0316 = product-tier shell composition). (3) Depends on ADR-0306 (disaster-mode/offline CRDT) and ADR-0292 (minor doctrine) — both must stay live for 0318's offline + age-overlay clauses to hold.
- **hyperscaler_challenge:** ALIGNED. Walmart Me@Walmart/Store Assist, UPS ORION/UPSNav, Kaiser mobile, RealWear smartglasses, Apple Required Device Capabilities, Google Play feature filters are all real frontline-adaptation precedents. The "one platform, many device/workspace projections" pattern is exactly how Walmart/UPS/Microsoft operate. Implication: KEEP.
- **ai_slop:** High structural repetition — the C-1..C-6 consequence blocks repeat the same 10 bullets verbatim per dimension, and D-1.xx / D-2.xx device/workspace entries are heavily templated (same capability/accessibility/privacy/audit/anti-pattern/test/migration lines per profile). This is the densest template-stamp smell in the chunk. refinement: dedupe consequence boilerplate; keep per-profile bespoke deltas only. consensus_needed: none.

---

### ADR-0319 — Front Office / Middle Office / Back Office Information-Barrier Doctrine
- **decision_atom:** Inside each tenant, model FRONT/MIDDLE/BACK office scopes and IB/Trading/Research/AssetMgmt/WealthMgmt information-barrier boundaries as first-class Cedar entities with default-deny cross-boundary access, time-boxed purpose-bound dual-sealed clearances, taint-labeling of derived artifacts, and per-pack regulatory overlays — refining (never replacing) tenant scope under ADR-0244 and evaluated through the ADR-0243 Cedar gate.
- **domain:** authz-policy (cross-cutting: compliance-residency).
- **current_status:** Proposed (`supersedes: []`; no `superseded_by` field present → live).
- **disposition:** AMEND.
- **proposed_resolution:** RATIFY (with amendments) — live, non-superseded, regulator-anchored, sound. Amend: (1) §Decision-Summary says "ADR-0316 is not present in this checkout at authoring time" yet front-matter `related`/`depends_on` both list ADR-0316 — internal inconsistency; 0316 exists, so remove the absent-file disclaimer. Note the new wrinkle: 0316 is being archived under 0329, so re-point the dependency to the surviving tenant-class projection concept. (2) The ADR self-corrects a brief error (FINRA 4514 → 4512(a)(1)(F)) — that correction is TRUE and should be preserved. Why ratify: the Chinese-Wall-as-Cedar-primitive is a real, defensible, high-value regulated-finance capability with exact statutory anchors (FINRA 5280/5290/3110/4530, EU MAR Art 9/14/16/17/18, MiFID II 16(3)/16(8)/23, KR FSCMA 174–178, UK SYSC 10.2, SG SFA 218–220, AU 912A/1043A).
- **governing:** n/a.
- **truth_flag:** TRUE (regulatory anchors verified-by-name in body), with a STALE absent-0316 disclaimer.
- **in_masterplan:** YES — office-scope/office-boundary Cedar entities + default-deny + dual-seal audit are masterplan-grade authz primitives for the regulated-finance packs.
- **tensions:** (1) Internal: absent-0316 prose vs present-0316 front-matter → AMEND. (2) Depends on ADR-0313 (conglomerate sovereign-child) and ADR-0263 (observability emission) — both must remain live. (3) Cedar-everywhere posture aligns with canonical §3 (Cedar universal gate ADR-0243/0246) — no conflict; this is an extension, not a competitor.
- **hyperscaler_challenge:** ALIGNED (mechanism), with a build-vs-buy note. AWS Verified Permissions/Cedar, Google Cloud IAM Conditions, Azure PIM (time-bound activation), AWS CloudTrail (immutable decision evidence) are cited as precedents and are apt. Hyperscalers ship the *primitives*; information-barrier *doctrine* is the bespoke regulated-finance layer Oyatie adds on top — a defensible "own the compliance overlay, reuse the policy engine" stance. Implication: KEEP.
- **ai_slop:** Moderate. The D-0 P01..P10 primitive-precedent matrix repeats the identical four regulatory + four hyperscaler precedent bullets for every primitive — template-stamp smell. The Cedar fragments and per-pack overlays (IS-Banking/IS-Investment-Mgmt/IS-Insurance/General) are genuinely bespoke. refinement: factor the repeated precedent boilerplate. consensus_needed: none; this is a clean ratify-after-cleanup.

---

### ADR-0320 — Apprentice, Intern, Resident, and Fellow Transient Identity Doctrine
- **decision_atom:** Model short-term cross-tenant program participation (`program_type` closed enum APPRENTICE/INTERN/RESIDENT/FELLOW/COOP/EXTERN) as a transient `program_tenant_membership` linking person/personal/source/host/program tenants, with time-bound Cedar capability tiers, auto-revocation at program end, jurisdiction-exact labor/training overlays, and personal-tenant portfolio survival — implemented through a shared `oya-shared-program-transient-identity` crate so consumers never re-derive program rules.
- **domain:** identity-authn / tenancy (cross-cutting: compliance-residency for labor law).
- **current_status:** Proposed (`amends: [ADR-0244, ADR-0311, ADR-0313]`; no `superseded_by` → live).
- **disposition:** AMEND.
- **proposed_resolution:** RATIFY (with amendments) — live, non-superseded, sound, and unusually concrete (closed enum + DDL + Cedar + statute-exact overlays for US FLSA/NLRA/ACGME, EU 2019/1152 + Quality Framework, KR LSA/Min-Wage/Vocational). Amend: (1) "capability tier" language (observer/contributor/supervised_operator/…) collides with the retired ADR-0316 "capability-tier" term — these are a DIFFERENT axis (program capability bundle, not product tier); rename or namespace to avoid masterplan confusion with both 0316-tiers and tenant-class. (2) front-matter `verification_expectations: line_count_at_least_1500` is a self-imposed density rule that ADR-0322 generalizes/supersedes-in-spirit — align to 0322's tier schedule. Why ratify: transient cross-tenant identity with portfolio survival is a real, differentiated identity capability with no competitor ADR.
- **governing:** n/a.
- **truth_flag:** TRUE — regulatory articles are named exactly (29 U.S.C. §§203/206/207, 29 C.F.R. §785.27, EU Dir 2019/1152 Arts 1/3/4/5/8/17, KR LSA Arts 17/50/53/54/55/60, ACGME 6.20/6.21.b/6.25.a/6.28). PARTIAL only on the "capability tier" naming overlap.
- **in_masterplan:** YES — program-transient-identity + auto-revoke + portfolio-survival + shared-crate ownership are masterplan-grade identity/tenancy primitives.
- **tensions:** (1) "capability tier" term overload vs 0316/0329 — naming AMEND. (2) `amends` ADR-0244/0311/0313 — confirm those remain live anchors (they are canonical per keystone §3 tenancy row). (3) D-1..D-10 primitive bodies are near-identical templated bullets (same 12-line A.x/D.x scaffold per primitive) — substance-bar smell vs ADR-0322.
- **hyperscaler_challenge:** ALIGNED. LinkedIn early-talent, Handshake three-party (student/school/employer), Workday/SuccessFactors early-career, Epic clinical-training, plus cloud-IAM temporary-credentials/time-bound-session precedents are apt. The "revoke access without deleting the person profile" pattern is standard enterprise-IAM doctrine. Implication: KEEP.
- **ai_slop:** Moderate-high. The 10 D-N primitives (D-1..D-10) each repeat the same 12 invariant bullets verbatim with only the heading changing — this is precisely the "lambda-wrap / template-stamp" failure ADR-0322 §Context names; the bespoke content (DDL, Cedar sketch, D-5.13 overlays, E.x consumer footprint) is real and good. refinement: collapse the repeated primitive scaffold; keep bespoke DDL/overlays/footprint. consensus_needed: none.

---

### ADR-0321 — B2B SaaS Industry-Leader Coverage Doctrine
- **decision_atom:** Cover every benchmarked B2B SaaS leader (Salesforce, ServiceNow, Workday, Atlassian, Microsoft, Adobe, HubSpot, Zendesk, Snowflake, Databricks, …; 165-vendor dossier) by mapping each vendor surface to one of {existing-microservice capability tier, composition across existing services, new flat microservice} — authorizing exactly 13 new microservice anchors (marketing-automation, contact-center, performance-management, learning-management, itsm, incident-management, financial-planning, data-warehouse, contract-lifecycle-management, whiteboard, design-collaboration, data-pipeline, healthcare-integration) and explicitly forbidding suite/product-fragment microservices (no salesforce/servicenow/workday/microsoft service).
- **domain:** product-ux / api-contracts (cross-cutting: marketplace-commerce, governance-process).
- **current_status:** Proposed (`status: Proposed`; no `superseded_by` field → live).
- **disposition:** AMEND.
- **proposed_resolution:** RATIFY (with amendments) — live, non-superseded, and the most operationally concrete ADR in the chunk (per-vendor data models, real API endpoints, Cedar verbs, ontology projections, migration playbooks, vendor-specific failure modes). Amend: (1) It is the direct operational child of ADR-0316; since 0316 is archived under 0329, re-anchor the "capability tier / Bronze-Silver-Gold-Platinum" mapping vocabulary to the surviving tenant-class projection model. (2) `vendor_dossier_count: 165` / `new_microservice_count: 13` and "corpus microservice count moves from 56 to 69" are point-in-time figures that will drift — treat as snapshot, not invariant; masterplan should record the *13 authorized anchors* and the *capability-tier-first decision rule*, not the running count. (3) Verify the 13 anchors against later forge/orchestration ADRs for any renames. Why ratify: the "cover the leader, refuse the suite boundary" rule is a TRUE, masterplan-grade product-strategy decision.
- **governing:** n/a (inherits, not superseded by, 0316→0329 — must re-anchor vocabulary).
- **truth_flag:** TRUE on decision (13 anchors + capability-tier-first rule), STALE on the "capability-tier/tier-name" vocabulary and on the volatile counts.
- **in_masterplan:** YES (PARTIAL) — the 13 authorized new-microservice anchors + the vendor-coverage decision rule belong in masterplan; the 165-row dossier is reference detail, not masterplan authority, and the tier names need the 0329 rename.
- **tensions:** (1) Inherits ADR-0316's retired "tier" vocabulary throughout the Bronze/Silver/Gold/Platinum mappings → AMEND with 0329. (2) The 13 new anchors must reconcile with ADR-0315 (SAP/ERP nine anchors, deliberately not reopened) and any later microservice-catalog ADRs — confirm no double-allocation. (3) "Foundry pipeline" / `axis-foundry`-style references in this Wave-3-G cluster are retired-vocab (foundry→intelligence/governance) — scrub. (4) Marketplace-as-deal-settlement dependency on ADR-0314 must remain live.
- **hyperscaler_challenge:** ALIGNED. Salesforce metadata projection, ServiceNow Now-module workflow projection, Microsoft Graph cross-product object access, Palantir Foundry ontology actions, Snowflake isolated compute are the exact precedents — and "absorb the category, refuse the vendor's suite boundary" is how a platform-grade hyperscaler would attack the B2B SaaS map. QUESTIONABLE only on breadth/sequencing: 13 new services + 165 migration dossiers is enormous scope; a hyperscaler would stage these behind demand gates rather than author all anchors at once (the ADR itself concedes "PR-143-shaped anchor documentation only" with buildout sequenced later). Implication: KEEP decision, AMEND to make the staging/sequencing explicit and de-emphasize the volatile counts.
- **ai_slop:** Low on substance (dossiers are genuinely bespoke per vendor — real schemas, endpoints, governor limits), but the sheer 2.7 MB / 165-row scale is itself the kind of artifact ADR-0322's wave-correlation + density checks exist to police; verify dossiers are not internally template-stamped row-to-row. refinement: split the 165-row dossier into a referenced spec/architecture artifact; keep the ADR to the decision rule + 13 anchors. consensus_needed: **"Are all 13 new-microservice anchors still authorized as of the current forge/orchestration canon, and should the masterplan carry the decision-rule only (with the dossier demoted to a referenced spec)?"**

---

### ADR-0322 — Substance Bar as Doctrine and CI Enforcement
- **decision_atom:** Promote the "substance bar" from review heuristic to **BLOCKER-class doctrine**: every in-scope documentation artifact (ADRs, journey docs, IP slices, microservice READMEs, PRDs, specs) must clear S-1..S-8 (bespoke-ratio ≥0.65, per-tier density floor, ≥10 bespoke D-N mechanics, non-duplicate Cedar hooks, audit-class declaration, no template-stamping, ≥12 named cross-refs, F4-substance reviewer-agent binding) enforced by the `oya-governance-substance-bar` crate family before merge-queue eligibility.
- **domain:** governance-process / docs-ssot-masterplan (cross-cutting: ci-cd-build).
- **current_status:** Proposed (`enforcement_status: blocker-day-one`; phased rollout to Accepted at T+60 per ADR-0327 gates; no `superseded_by` → live).
- **disposition:** KEEP (AMEND only for naming/vocab).
- **proposed_resolution:** RATIFY — live, non-superseded, self-consistent, authority_tier 1, and it is the *enforcement doctrine that makes the masterplan-from-ADRs model trustworthy* (it directly bans the template-stamping/lambda-wrap/table-of-contents failure modes that plague 0316–0321 in this very chunk). It even names ADR-0319/0320/0321 initial drafts as Incident I-3 (under-200-line drafts reauthored). Amend only: governance crates are `oya-governance-*` (correct, post-rename); ensure no residual `oya-foundry-*` / `axis-foundry` leakage in the reviewer-agent dispatch (§D-11 says "Reviewer agents are dispatched via the foundry pipeline" — that is retired vocab → AMEND to intelligence/oya-ci).
- **governing:** n/a.
- **truth_flag:** TRUE — internally coherent, mechanism is concrete (Rabin-Karp shingles, Jaccard bespoke-ratio, corpus snapshotting, ed25519 facet signatures).
- **in_masterplan:** YES — this is a masterplan-grade SSOT-integrity gate; under the "masterplan generated from ADRs" reading it is load-bearing (it guarantees ADRs carry real, non-duplicated decision content).
- **tensions:** (1) §D-11 "dispatched via the foundry pipeline" + `axis-foundry` owner = retired-vocab leakage (foundry→intelligence/governance) → AMEND. (2) Cites ADR-0327 (promotion gates), ADR-0323 (wave sequencing), ADR-0324 (anti-script) as live dependencies — confirm those exist and are live (out of this chunk). (3) Mild self-reference paradox: 0322 mandates ≥0.65 bespoke ratio while several peers it governs (0317/0318/0320) are themselves heavily templated — 0322 is the corrective, not a conflict.
- **hyperscaler_challenge:** ALIGNED. Google's readability/review culture, Kubernetes KEP rigor, and large-monorepo doc-gating (CI-enforced doc quality) are the precedent. A hyperscaler running a generated-from-source-of-truth doc system would absolutely gate on substance/anti-duplication. Implication: KEEP.
- **ai_slop:** Very low — this ADR practices what it preaches (bespoke D-1..D-14, worked example, failure-mode catalog F-1..F-8). consensus_needed: none. One watch-item: it references the "foundry pipeline" for reviewer dispatch — the only retired-vocab slip in an otherwise clean doc.

---

## Chunk notes

**Overall verdict.** This is a coherent, high-value cluster — the Wave-3-G "projection doctrine" family. The single TRUE meta-decision is **"named product/role/workforce/program surfaces are PROJECTIONS over one shared substrate (one tenant, one Cedar gate, one ontology, one workflow engine, one UX-shell vocabulary), never forks."** That principle is masterplan-grade and recurs in 0316, 0317, 0318, 0319 (office-barrier projection), 0320 (program-identity projection), and 0321 (vendor-coverage projection). ADR-0322 is the enforcement keystone that keeps the whole ADR-as-SSOT model honest.

**One ARCHIVE, five AMEND-then-RATIFY, one clean KEEP.**
- ARCHIVE: **0316** (superseded by 0329; mechanism survives, "tier" name dies).
- AMEND→RATIFY: **0317, 0318, 0319, 0320, 0321** (all live, sound, non-superseded; each needs cleanup).
- KEEP: **0322** (ratify; only a single retired-vocab slip to fix).

**Cross-cutting AMEND actions the masterplan generator must apply to this whole chunk:**
1. **Retire "capability tier" naming** everywhere in 0316/0317/0318/0320/0321 → re-express under ADR-0329 tenant-class + composable projection vocabulary. The *mechanism* (permit-set + ontology-projection + workflow-template + UX-shell activation bundle) is TRUE and survives; only the word "tier" is dead. Do NOT conflate with the live autonomy-tier T1–T4 policy axis.
2. **Fix the "authoring-time absent file" disclaimers.** 0317 says 0316 absent; 0318 says 0317 absent; 0319 says 0316 absent. All three referents now exist on disk — these self-aware placeholders are now factually STALE and must be patched (0317/0318/0319 each explicitly instruct their own future patch).
3. **Scrub retired `foundry` vocabulary:** `axis-foundry` owners (0316, 0322) and "foundry pipeline" reviewer dispatch (0322 §D-11) → intelligence/governance / oya-ci per ADR-0335/0347/0363.
4. **Demote volatile counts to snapshots:** 0321's 165-dossier / 13-anchor / "56→69 services" and 0320's `line_count_at_least_1500` are point-in-time; masterplan should carry the *decision rules and authorized anchor set*, not the running totals.
5. **Substance-bar irony:** 0316/0317/0318/0320 (and possibly the 0321 dossier rows) exhibit exactly the template-stamping / repeated-block pattern that 0322 bans (0322 even names 0319/0320/0321 initial drafts as remediation incident I-3). Under the generated-from-ADRs masterplan model, these should be de-slopped before the masterplan absorbs them — the *decisions* are TRUE, the *prose padding* is not.

**Founder questions surfaced (contested / needs sign-off):**
- (0316/0329) Confirm the capability-tier *projection mechanism* survives the tier-system retirement under tenant-class vocabulary. (Strongly implied yes by 0321/0317/0318/0320 all depending on it, but the word must change.)
- (0321) Are all 13 new-microservice anchors still authorized as of current forge/orchestration canon, and should the masterplan carry only the decision-rule (capability-tier-first + 13 anchors) with the 165-row vendor dossier demoted to a referenced spec/architecture artifact?
- (chunk-wide) Ownership seam between the three projection axes — 0316 (product/tenant-class shell composition), 0317 (role projection), 0318 (device/workspace/collar projection) — should be stated explicitly in the masterplan so the three doctrines compose rather than contend.

**No number collisions, no duplicate IDs, no Proposed-without-disposition in this slice.** All seven are `status: Proposed`; only 0316 carries a `superseded_by` edge (→0329), which the keystone §1.1 confirms. The other six are live with empty `superseded_by`. No GARBAGE, no WRONG in this chunk — worst truth_flag is PARTIAL (0316 vocab; 0321 vocab+counts).
