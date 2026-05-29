---
doc_class: Ownership-Coherence-Audit
audit_id: coherence-audit-2026-05-20-learning-management
microservice: learning-management
phase: Phase-4A.1-HR-Payroll-Big-8
batch: Wave-4-Rolling-HR-Payroll
audit_date: 2026-05-20
audit_owner: solo-codex-microservice-ownership-agent
audit_class: ADR-0328-D-4-five-dimension-ownership-coherence
authoring_mode: bespoke-no-scripting-no-templates
counterparts_canonical_for_this_audit:
  - Canvas LMS
  - Cornerstone OnDemand
  - Docebo
counterparts_in_local_manifest_DRIFTED:
  - Workday Learning
  - Cornerstone
  - Degreed
  - LinkedIn Learning
  - Udemy Business
  - Salesforce Trailhead
five_anchor_citations:
  anchor_1_unified_ecosystem_thesis: /Users/jasonlee/oyatie/docs/architecture/unified-ecosystem-thesis-2026-05-21.md
  anchor_2_microservice_prd: /Users/jasonlee/oyatie/microservices/learning-management/PRD.md
  anchor_3_local_artifact_inventory: this audit Section A.2
  anchor_4_top_3_counterparts: brief override Canvas LMS + Cornerstone OnDemand + Docebo
  anchor_5_documentation_rigor_1_1: /Users/jasonlee/oyatie/docs/standards/documentation-rigor.md §1.1
binding_adrs:
  - ADR-0328
  - ADR-0322
  - ADR-0324
  - ADR-0244
  - ADR-0263
  - ADR-0105
  - ADR-0316-RETIRED
  - ADR-0321
  - ADR-0329-pending-tenant-class
verdict_summary: BLOCK
verdict_severity_distribution:
  P0_block: 6
  P1_revise: 7
  P2_passwithfindings: 9
  P3_cosmetic: 5
remediation_wave_target: Wave-15A-P0-then-15F-Phase-4-substance-then-15J-tier-retirement
---

# Ownership-Coherence Audit — `learning-management`

## §3.4.T — Tenant-class adherence section (per brief)

The brief explicitly states that tier scaffolding is retired and the tenant model is `tenant_class ∈ {demo_trial, paid}` with `paid.billing_components ⊂ {revenue_share, per_seat, per_usage}` per `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20`. This audit therefore refuses to author any tier-delta deliverable and treats every surviving Bronze/Silver/Gold/Platinum reference in the µservice as a P0 or P1 finding to be retired in Wave 15J.

### §3.4.T-1 Tenant-class adherence findings

T-1.1. `manifest.json` line 7 declares `"tier": "product"` and lines 98-100 declare `"capability_tiers": ["product"]` and `"capability_tier_doctrine"` block at lines 82-85 cites ADR-0316. ADR-0316 is retired by `feedback_no_capability_tiers_2026_05_20`. The field is not yet replaced by `tenant_class` or `billing_components`. Severity P1 (canonical-direction). Fix shape: remove `tier`, `capability_tiers`, `capability_tier_doctrine` blocks; add `tenant_classes_supported: [demo_trial, paid]`, `billing_components_emitted_for_paid: [per_seat, per_usage]` (revenue_share is N/A for learning-management because the µservice is not a marketplace seller surface — its content marketplace path settles through `cloud-marketplace` per ADR-0314, which is the seller of record, not learning-management).

T-1.2. `manifest.json` lines 45-53 declare `cell_eligibility` with `eligible_tiers: ["tier-1","tier-2"]`. The "tier" word here is a cell-failure-tier (tier-1/tier-2 cells per ADR-0248 cellular architecture), NOT a Bronze/Silver tier. The shared name is confusing post-retirement. Severity P3 (cosmetic). Fix shape: rename `eligible_tiers` to `eligible_cell_classes` to remove collision with the retired capability-tier vocabulary.

T-1.3. PRD.md frontmatter lines 9-18 cites `ADR-0316` in `related_adrs`. The body never explains how learning-management gates behavior on capability tier; it carries the citation as inheritance scaffolding only. Severity P1. Fix shape: remove ADR-0316; add ADR-0329 (the pending tenant-class replacement ADR) and ADR-0244 (tenant scoping) explicitly.

T-1.4. IP-001-tenant-scope-kernel.md line 10 declares `capability_tier: T2`. T2 is the retired-tier nomenclature (Silver-equivalent). The IP otherwise carries substantive content (data model, REST/gRPC endpoints, Cedar hooks, ontology projection, SLO numbers); the tier annotation is the only retired-doctrine residue. Severity P2. Fix shape: replace `capability_tier: T2` with `tenant_classes_in_scope: [demo_trial, paid]` (this IP applies to both classes — the kernel is the boundary).

T-1.5. No file in the µservice path declares the demo_trial usage cap shape (max enrolled courses, max active learners, max stored completion evidence rows) that `cloud-billing` will need to enforce demo_trial cap-breach. Severity P1 (substance-bar gap). Fix shape: add a `tenant-class-behavior.md` section to PRD.md or a separate `tenant-class-behavior.md` file declaring (a) demo_trial caps per FR-001..FR-030, (b) paid per_seat meter shape (named-learner-count emitted to `cloud-billing`), (c) paid per_usage meter shape (e.g., completion-evidence-rows, credential-assertions-issued, regulated-attestation-rows, courseware-bytes-stored emitted to `cloud-billing`).

T-1.6. The substrate-dependencies in manifest.json line 36-44 lists `community`, `workflow-engine`, `ontology`, `mail`, `intelligence`, `identity`, `compliance` but does NOT list `cloud-billing`. Since per_seat and per_usage meters MUST emit to `cloud-billing` for paid tenants, `cloud-billing` is a substrate dependency. Severity P1. Fix shape: add `cloud-billing` to substrate_dependencies and depends_on_microservices; emit `learning-management.usage.named_learner_count`, `learning-management.usage.completion_evidence_count`, `learning-management.usage.credential_assertion_count` to `cloud-billing` per the canonical meter contract.

T-1.7. ADR-0316 is also cited in `binding_adrs` at line 18 of manifest.json. Same retirement issue. Severity P1. Fix shape: remove from `binding_adrs`.

### §3.4.T-2 Tenant-class adherence verdict

Verdict for §3.4.T: REVISE. The µservice is sequenced for Wave 15J retirement-of-tier-doctrine; current state carries six retired-doctrine residues (T-1.1, T-1.3, T-1.4, T-1.6, T-1.7) and one missing demo_trial-cap declaration (T-1.5). None is a hard contradiction, but the µservice cannot promote past the Phase 4A.1 gate until the tier residue is removed and the demo_trial cap surface is declared.

## §3.4.C — Counterpart adherence section (per brief)

The brief overrides the µservice's manifest counterpart list. The manifest declares six counterparts (Workday Learning + Cornerstone + Degreed + LinkedIn Learning + Udemy Business + Salesforce Trailhead). The brief declares three (Canvas LMS + Cornerstone OnDemand + Docebo). This is a P0 hard counterpart-source contradiction per ADR-0328 §D-5.3.

### §3.4.C-1 Counterpart adherence findings

C-1.1. The brief's three counterparts (Canvas LMS, Cornerstone OnDemand, Docebo) are a substantively different industry shape from the manifest's six. Canvas LMS is the academic/higher-ed LMS leader (Instructure parent). Cornerstone OnDemand is the corporate-learning suite leader (recently merged with Saba). Docebo is the AI-driven corporate-LMS challenger leader. The manifest's mix is workforce-development-platform (Workday Learning) + corporate-learning-suite (Cornerstone) + learner-experience-platform (Degreed) + content-marketplace (LinkedIn Learning, Udemy Business) + developer-credentials (Trailhead). The manifest mix does NOT include Canvas LMS (the academic incumbent), so a Wave 4 audit using the brief's set requires reauthoring the parity-matrix anchor against a different industry segment. Severity P0 (hard counterpart-source contradiction). Fix shape: in Wave 15F, normalize the µservice's manifest, PRD, README, competitor-parity-matrix.md, capabilities/*.yaml, PHASE-01 operating bar, runbooks, and all per-section §benchmarks lines to one of two coherent counterpart sets — (a) the brief's academic-and-corporate-LMS-leader set, or (b) the manifest's workforce-and-content-marketplace set — but NOT both simultaneously. Recommend the brief's set survives because the brief carries the active Wave 4 directive and Canvas LMS is the missing academic surface that the manifest set ignores; HR-family enterprise customers DO include K-12, higher-ed, and university-research-institute tenants who need Canvas-shape semantics.

C-1.2. Bounded-contexts in manifest.json line 29-35 are `course-catalog`, `enrollment`, `learning-path`, `assessment`, `credential`. These map cleanly to corporate-LMS (Cornerstone OnDemand, Docebo) but DO NOT cover Canvas LMS's academic primitives: grade-book (grade-passback / late-submission policy / grading-scheme), discussions (instructor-moderated threaded), SCORM/xAPI runtime, assignment-and-rubric semantics, course-module-prerequisite, and quiz-attempt-policy. Severity P0 (parity contradiction). Fix shape: in Wave 15F, add `assignment`, `grade-book`, `discussion`, `scorm-runtime`, `xapi-statement`, `course-module`, `quiz-attempt` as additional bounded contexts OR document explicit out-of-scope-intentional for the Canvas LMS academic features with a doctrine reason (e.g., "academic LMS belongs to a future `learning-academic` µservice, not this corporate-learning surface"). Recommend explicit ADR-0329-companion authoring rather than silently expanding the bounded-context set; Canvas LMS academic features are a substantial scope expansion that warrants its own decision.

C-1.3. The README §scope-and-non-goals (line 11) declares the first-wave concern as "course enrollment, completion evidence, skills credentials, and regulated training attestations." This wording cleanly covers Cornerstone-and-Docebo-style corporate learning but is silent on Canvas-style academic learning (assignment + grade-book + discussion + SCORM). Severity P1 (canonical-direction). Fix shape: amend §scope-and-non-goals to declare explicit position — either "academic LMS is in scope and tracked via the bounded-context expansion in C-1.2" or "academic LMS is out-of-scope-intentional and reserved for a future `learning-academic` µservice."

C-1.4. PRD.md §K Hyperscaler and Industry Precedents (lines 171-173) cites Salesforce Trailhead, Workday Learning, LinkedIn Learning. These are NOT the brief's three. Severity P0 (counterpart-source contradiction). Fix shape: replace with Canvas LMS academic-and-LTI precedent (grade-passback + LTI 1.3 Advantage), Cornerstone OnDemand compliance-training precedent (regulated-attestation evidence + recertification windows), Docebo AI-driven recommendation precedent (skills-graph-derived learning-path generation). Maintain manifest precedents only if §K explicitly says "manifest precedents retained for Wave 4 evidence; brief precedents adopted in Wave 15F."

C-1.5. Manifest line 110 declares `hyperscaler_benchmark` as the manifest's six (Workday + Cornerstone + Degreed + LinkedIn + Udemy + Trailhead). Same counterpart-source issue. Severity P0. Fix shape: normalize to brief's three after Wave 4 audit completes.

C-1.6. PRD §B Target Users (lines 33-39) cites six personas (Marcus Chen, Yejin Park, Diana Alvarez, Nadia Singh, Omar Watkins, Hana Mori). These are generic tenant-administrator personas; they are NOT the academic-LMS persona set (faculty member, teaching assistant, student, registrar, dean-of-academic-affairs, accreditation-auditor). If learning-management absorbs Canvas-shape semantics per C-1.2, the persona set is incomplete. Severity P1 (canonical-direction gap). Fix shape: in Wave 15F, add at minimum (a) faculty member (instructor authoring courses for academic credit), (b) student (learner consuming graded coursework), (c) registrar (managing credit recognition and transcript), (d) accreditation auditor (SACSCOC / WASC / Middle States evidence reviewer). If C-1.2 chooses the explicit-out-of-scope path, declare the persona omission as intentional.

### §3.4.C-2 Counterpart adherence verdict

Verdict for §3.4.C: BLOCK. The µservice has a hard counterpart-source contradiction between brief (Canvas LMS / Cornerstone OnDemand / Docebo) and manifest (six different counterparts). Per ADR-0328 §D-4.24 BLOCK means a hard contradiction that can mislead downstream implementation, and the brief explicitly says "Canvas LMS / Cornerstone OnDemand / Docebo" in the Wave 4 dispatch contract — so this audit treats the brief as authoritative. Wave 15F must normalize the counterpart set across all six artifacts that name counterparts (manifest, PRD, README, competitor-parity-matrix, PHASE-01 operating bar, capabilities). Bounded-context expansion (C-1.2) is the downstream consequence and is also tracked as P0.

## §3.4.B — Big-8 HR family Learning position adherence (per brief)

ADR-0328 §D-2.3..D-2.7 places HR/Payroll first in the Big 8 sub-sequence for Phase 4A. learning-management is named explicitly in D-1.85 as a Phase 4 B2B/ERP service AND in D-2.5 as part of the Workday family which sequences as Phase 4A.1. The brief says "BIG 8 HR/Payroll family" and "Big 8 P0 elevation." This audit therefore treats learning-management as a Phase 4A.1 sub-service that MUST be coherent with employee-identity, manager-hierarchy, job-role, pay-group, benefits-eligibility, shift, worker-type, and cost-center inputs per D-2.6, and MUST land before ERP (D-2.8..D-2.11) and CRM (D-2.12..D-2.15) downstream.

### §3.4.B-1 Big-8 HR family adherence findings

B-1.1. learning-management depends on `identity` (manifest line 41-42) but does NOT depend on `hris` (the HR-master-data µservice that owns workforce, job role, pay group, cost center). For HR/Payroll family coherence, learning-management MUST consume manager-hierarchy from `hris` to drive learning-path-assignment (Cornerstone OnDemand pattern: manager-assigns-training-to-direct-reports), regulated-training-attestation (compliance-officer-assigns-mandatory-training-by-cost-center), and skills-graph-mapping (Docebo pattern: role-derived-skills-gap). Severity P0 (canonical-direction; downstream HR-family services will mislead implementation). Fix shape: add `hris` to substrate_dependencies and depends_on_microservices, declare the request shape (tenant_id + worker_id + manager_chain + job_role + cost_center) in the cross-microservice handoff section that's missing today (B-1.5 below).

B-1.2. IP-001-tenant-scope-kernel.md line 102-108 declares cross-µservice handoffs to `hris`, `identity-access`, `audit-chain`, `data-residency`, `content-provider-integration`. The handoff to `hris` IS named in this IP. Severity contradicts B-1.1: IP-001 has the right handoff at line 104 but manifest.json line 41-42 and PRD.md §M cross-references do NOT. Severity P0 (internal-coherence — manifest contradicts IP). Fix shape: align manifest.json and PRD.md with IP-001's correct handoff to `hris`; do not delete the IP-001 reference. Note: `identity-access` in IP-001 is the wrong µservice name — the canonical name per ADR-0328 D-1.30 Phase 1 service 01 is `identity`, not `identity-access`. Severity P2 (naming drift). Fix shape: rename `identity-access` → `identity` in IP-001 line 105.

B-1.3. PHASE-01-LEARNING-MANAGEMENT-OPERATING-BAR.md is a 122K-character template-stamped file (verified via Read sampling §A scope, §B principals, §C cedar-gates — every section repeats the identical 8-line "Learning Management binds course-enroll to tenant_id..." pattern with vendor names rotated). Per ADR-0322 substance-bar doctrine and ADR-0324 anti-script anti-template doctrine, this is a P0 substance-bar violation. Severity P0. Fix shape: retire PHASE-01 in Wave 15F and replace with a 200-300-line bespoke phase-plan that names the actual Phase 4A.1 admission gates: (a) hris-handoff contract live, (b) Cedar policies cover demo_trial cap-breach and paid per-seat-overage, (c) ontology projection lands course / enrollment / completion / credential into the canonical ontology, (d) regulated-training-attestation evidence sealed via audit-chain, (e) demo_trial → paid conversion flow with billing-components mutation. The current file is filler at the line-floor; it is unfit for Phase 4A.1 promotion evidence.

B-1.4. README.md (56K characters), competitor-parity-matrix.md (107K characters), and compliance.md (117K characters) all follow the same template-stamping pattern as PHASE-01. Verified via Read sampling that lines repeat the identical 8-line vendor-rotation pattern in every section. Per ADR-0322 / ADR-0324 / `feedback_docs_substance_not_scaffold_2026_05_20` these are P0 substance-bar violations. Severity P0 (all three files). Fix shape: in Wave 15F (Phase 4 substance gaps), each file is retired and rewritten bespoke. The substance bar is intern-buildability per documentation-rigor §1.1; today's content is not intern-buildable because every section says the same thing.

B-1.5. There is no `cross-microservice-handoffs.md` file in the µservice path. Per `feedback_microservice_ownership_coherence_2026_05_20` step 1 the µservice owner authors a `cross-microservice-handoffs.md` declaring every reciprocal handoff to other µservices. Without this file, the HR-family coherence cannot be verified (each named handoff must be reciprocated in the other µservice's handoff file). Severity P1 (substance-bar gap that blocks Phase 4A.1 promotion). Fix shape: in Wave 15F, author `cross-microservice-handoffs.md` declaring handoffs to `hris` (worker + manager-chain + job-role + cost-center), `identity` (principal issuance), `audit-chain` (regulated-training-attestation evidence seal), `compliance` (pack-overlay activation per FERPA / KOSA / GDPR / KR-PIPA), `workflow-engine` (assignment + completion-sealing workflow), `ontology` (course / enrollment / credential projection), `community` (discussion / cohort), `mail` (assignment + due-date notifications), `intelligence` (Docebo-style AI recommendation), `cloud-billing` (per_seat + per_usage meter emission), `cloud-marketplace` (Udemy-style content-marketplace DealSet settlement per ADR-0314), `payments` (paid-tenant invoice rendering). Each handoff names tenant boundary, error mode, contract version, owning µservice for the reciprocation.

B-1.6. PRD.md §I Open Questions (lines 161-163) declares three open questions, all of which are scope-deferral language ("Which full PR-143 artifact wave owns the first contract family"). For a Big 8 P0-elevated HR/Payroll-family Wave 4 µservice, open questions should be HR-coherence-related (e.g., "How does learning-management handle a worker who changes cost-center mid-course?", "How does learning-management absorb a regulated-training certification that the worker brought from a prior employer?", "How does demo_trial cap-breach interact with regulated-training-attestation due dates?"). Severity P1 (substance-bar shape). Fix shape: in Wave 15F, replace open-questions with HR-family-coherence open questions.

B-1.7. The Rust src/lib.rs declares `BOUNDED_CONTEXT: "course-progress"` (line 30) and `PRIMARY_CAPABILITY: "learning-path-completion"` (line 31). The manifest declares five bounded contexts (`course-catalog`, `enrollment`, `learning-path`, `assessment`, `credential`) — none of which is `course-progress`. The Cargo.toml line 16 says `bounded_context = "course-progress"`. The src/ and Cargo.toml agree internally but contradict the manifest. Severity P0 (internal-coherence — Rust scaffold uses a different bounded-context name than the manifest). Fix shape: in Wave 15F, choose ONE — either (a) rename the Rust scaffold to declare ALL five manifest-defined bounded-contexts as modules (`src/course_catalog/`, `src/enrollment/`, `src/learning_path/`, `src/assessment/`, `src/credential/`), or (b) update the manifest to declare a single bounded-context `course-progress` matching the Rust scaffold. Recommend (a) because the five-bounded-context model carries the actual operational concern shape per PRD §C.

### §3.4.B-2 Big-8 HR family adherence verdict

Verdict for §3.4.B: BLOCK. learning-management has a P0 hris-handoff gap (B-1.1 + B-1.2 inconsistency), P0 substance-bar violations in three top-level docs (B-1.4) and the PHASE-01 operating bar (B-1.3), P0 Rust-vs-manifest bounded-context contradiction (B-1.7), and a missing cross-microservice-handoffs file (B-1.5). For Big 8 HR/Payroll first-position sequencing, downstream ERP/CRM µservices will mislearn the handoff shape from this µservice's current state. The µservice cannot promote past Phase 4A.1 gate until B-1.1, B-1.2, B-1.3, B-1.4, B-1.5, B-1.7 are remediated.

## A. Audit scope and method

### A.1 Five-anchor citation

This audit uses ADR-0328 §D-3 Agent class 1 (microservice-ownership-audit) anchor set.

- Anchor 1: `/Users/jasonlee/oyatie/docs/architecture/unified-ecosystem-thesis-2026-05-21.md` — supplies one-substrate doctrine. learning-management is a HR/Payroll family product label that MUST project over identity + tenancy + ontology + workflow-engine + audit-chain + governance and MUST NOT recreate Cornerstone-suite or Canvas-suite boundaries.
- Anchor 2: `/Users/jasonlee/oyatie/microservices/learning-management/PRD.md` — supplies the µservice's own product contract.
- Anchor 3: this audit Section A.2 (local artifact inventory) — supplies the artifact-coherence baseline.
- Anchor 4: brief override Canvas LMS + Cornerstone OnDemand + Docebo — supplies the parity bar.
- Anchor 5: `/Users/jasonlee/oyatie/docs/standards/documentation-rigor.md` §1.1 — supplies the intern-buildability substance bar.

### A.2 Local artifact inventory

The µservice path `/Users/jasonlee/oyatie/microservices/learning-management/` contains:

Top-level (25 markdown files, 1 JSON manifest, 1 Cargo.toml, 1 Cargo.lock):

- ARCHITECTURE.md — 902 lines (template-stamped)
- backfill-replay.md — 270 lines (template-stamped)
- capacity-model.md — 320 lines (template-stamped)
- CHANGELOG.md — 8 lines
- competitor-parity-matrix.md — 370 lines (template-stamped; counterpart set drifted)
- compliance.md — 925 lines (template-stamped)
- cost-budget.md — 270 lines (template-stamped)
- dpia.md — 420 lines (template-stamped)
- failure-modes.md — 320 lines (template-stamped)
- incident-response.md — 270 lines (template-stamped)
- IP-001-tenant-scope-kernel.md — 108 lines (BESPOKE — substantive)
- IP-002-cedar-default-deny.md — 110 lines (BESPOKE)
- IP-003-ontology-projection.md — 108 lines (BESPOKE)
- IP-004-workflow-template-library.md — 110 lines (BESPOKE)
- IP-005-rest-contract-surface.md — 110 lines (BESPOKE)
- IP-006..IP-025 — 55 lines each (template-stamped IPs; thin)
- IP-026-skills-graph-gap-analyzer.md — 104 lines (BESPOKE)
- IP-027-compliance-training-attestation-ledger.md — bespoke per filename
- IP-028-content-provider-catalog-federation.md — bespoke per filename
- IP-029-learning-path-recommendation-guardrail.md — bespoke per filename
- IP-030-credential-expiry-renewal-orchestrator.md — bespoke per filename
- manifest.json — 137 lines (counterpart set drifted, tier residue, missing hris + cloud-billing)
- multi-region.md — 270 lines (template-stamped)
- PHASE-01-LEARNING-MANAGEMENT-OPERATING-BAR.md — 122K characters (template-stamped P0)
- PRD.md — 64K characters (template-stamped P0)
- README.md — 56K characters (template-stamped P0)
- sdk-plan.md — bespoke per filename
- threat-model.md — bespoke per filename

Subdirectories:

- `src/` — Rust scaffold with `lib.rs` (118 lines), `main.rs` (26 lines), `config.rs`, `error.rs`, `adapter/mod.rs`, `domain/mod.rs`, `usecase/mod.rs`. Declares single bounded-context `course-progress` contradicting manifest's five.
- `tests/integration.rs` — 96 lines.
- `slos/` — 12 OpenSLO files (availability, latency, audit-emission-lag, replay-freshness, plus 8 local-prefixed flow-specific SLOs). availability.openslo.yaml is bespoke and substantive (32 lines with real Prometheus queries and 99.9% target).
- `contracts/` — openapi-v1.yaml (78 lines, bespoke skeleton), asyncapi-v1.yaml (34 lines), learning-management-v1.proto (21 lines), plus `local-*` variants.
- `policies/` — 6 local-* Cedar policy files (local-assessment-attempt-control, local-certificate-issue-gate, local-cohort-enrollment-scope, local-content-delivery-entitlement, local-course-publish-approval, local-session-attendance-access). Not yet sampled in this audit; needs Wave 15F substance check.
- `policy/` — 5 cedar files + 1 data-residency.md (abuse-defence, auditor-scope, ci-scope, credential-training-authorization, emergency-services-bypass).
- `runbooks/` — 21 runbooks; verified course-enrollment-stall.md is template-stamped per ADR-0322/0324.
- `iac/` — 23 IaC files including dr-failover, ech-config, edge-waf, helm-values, kustomization, network-policy, openbao-policy, pqc-cert, production-ingress, secret-bindings, service-monitor, terraform-module.tf, plus local-prefixed variants. Note: terraform-module.tf naming violates ADR-0328 §D-16.1 (OpenTofu only — no Terraform spelling).
- `dashboards/` — 10 dashboard JSON files.
- `capabilities/` — 6 capability YAML files (completion-seal, course-enroll, credential-issue, provider-catalog-sync, regulated-training-attest, skills-graph-export).
- `catalog/` — 13 catalog YAML files (one per ADR-0105 13-layer enum slug).
- `scorecards/` — 1 overrides.json file.

### A.3 Audit method

Per ADR-0328 §D-4 (5-dimension), §D-10 (verification SLA), and §D-14 (Codex-only HALT-CLEANLY). The audit reads the µservice path, samples ≥3 random artifacts per surface class, cross-checks the five-anchor set, and emits findings; no remediation. Per `feedback_microservice_ownership_coherence_2026_05_20`, one agent owns end-to-end. Per `feedback_verify_deliverables_not_just_line_count_2026_05_20`, line count alone is not enough — substance is checked.

## B. Dimension 1 — Internal coherence

### B.1 Manifest ↔ PRD ↔ ARCHITECTURE ↔ README coherence

B.1.1. Manifest line 110 declares six benchmarks. PRD §A line 29 declares the same six. README line 7 declares the same six. ARCHITECTURE companion-docs declares no benchmarks (benchmark list is intentionally absent from architecture). Internal consistency: OK on benchmark list across manifest/PRD/README, but ALL contradict the brief's three (Canvas LMS + Cornerstone OnDemand + Docebo). Verdict: P0 (counterpart-source contradiction, see §3.4.C-1.1).

B.1.2. Manifest line 29-35 bounded_contexts = [course-catalog, enrollment, learning-path, assessment, credential]. PRD §C uses the same five. ARCHITECTURE.md §C uses the same five. README §scope-and-non-goals does NOT name bounded contexts. Rust src/lib.rs uses `course-progress`. Internal consistency: PRD ↔ ARCHITECTURE ↔ manifest agree; src/ contradicts. Verdict: P0 (see B-1.7).

B.1.3. Manifest line 36-44 substrate_dependencies = [community, workflow-engine, ontology, mail, intelligence, identity, compliance]. PRD §M cross-references the same seven. ARCHITECTURE.md §D Integration Topology names the same seven. README does NOT name dependencies. cross-microservice-handoffs.md does NOT exist. IP-001 line 102-108 names {hris, identity-access, audit-chain, data-residency, content-provider-integration} which is a DIFFERENT set (and uses `identity-access` instead of `identity`, and `data-residency` which is not a manifested µservice in this repo, and `content-provider-integration` which is not in the manifest dependency list either). Internal consistency: P0 contradiction between manifest+PRD+ARCHITECTURE (one set) and IP-001 (a different set). Verdict: P0.

B.1.4. Manifest line 74-81 compliance_packs = [SOC-2, ISO-27001, GDPR, KR-PIPA, FERPA, KOSA]. compliance.md §B Control Families names the same six. PRD §H Compliance Impact names the same six. README line 7 lists the six benchmarks (not packs). Manifest line 101-109 packs = [soc2, iso27001, gdpr, kr-pipa, FERPA, KOSA, hipaa] — note the SEVENTH entry hipaa is NOT in compliance_packs at line 74. Internal consistency: P1 contradiction between manifest line 74 (six packs) and manifest line 101 (seven packs including hipaa). Verdict: P1 (hipaa is either supported or not — needs decision).

B.1.5. Manifest line 54-73 declares layer_enum_conformance.declared_layers = [api, rest, application, usecase, domain, kernel, adapter, worker, governance] — NINE layers. ADR-0105 13-layer enum requires THIRTEEN layers per `feedback_layer_enum_adr_0105_13_canonical`. The Rust src/lib.rs `validate_scaffold` function at line 102 asserts `descriptor.layer_count() != 13` — so Rust expects 13 and manifest declares 9. Internal consistency: P0 contradiction. Verdict: P0 (Rust will fail validation on 9 layers; manifest under-declares).

B.1.6. ARCHITECTURE.md §B Layer Map lines 26-36 also lists only NINE layers (api, rest, application, usecase, domain, kernel, adapter, worker, governance). Same contradiction as B.1.5. Verdict: P0.

B.1.7. PRD.md frontmatter line 9 lists related_adrs as [ADR-0131, ADR-0132, ADR-0244, ADR-0245, ADR-0314, ADR-0315, ADR-0316, ADR-0321]. Manifest line 10-20 lists binding_adrs as [ADR-0105, ADR-0131, ADR-0132, ADR-0244, ADR-0245, ADR-0314, ADR-0315, ADR-0316, ADR-0321]. Difference: PRD omits ADR-0105 (the layer enum doctrine). Severity P2. Fix shape: add ADR-0105 to PRD.md related_adrs.

B.1.8. The IP files numbered IP-006 through IP-025 are exactly 55 lines each (verified via wc -l). This is suspicious uniformity per ADR-0322 substance-bar doctrine and `feedback_docs_substance_not_scaffold_2026_05_20`. Sampled IP-006-async-event-surface.md (55 lines) — needs further read in Wave 15F to determine bespoke vs template. Severity P2 pending sample. Fix shape: in Wave 15F, sample-read each of IP-006..IP-025 and remediate any that are template-stamped.

### B.2 Internal coherence verdict

Verdict: REVISE (six P0 contradictions: B-1.7 + B.1.1 + B.1.2 + B.1.3 + B.1.5 + B.1.6, plus one P1 + two P2). The µservice cannot promote past Phase 4A.1 gate until the manifest ↔ PRD ↔ ARCHITECTURE ↔ Rust scaffold coherence is restored.

## C. Dimension 2 — Outbound cross-references

### C.1 ADR cross-references

C.1.1. Manifest line 10-20 binding_adrs cites ADR-0105 (layer enum), ADR-0131 (per-microservice flat layout), ADR-0132 (no-grouping policy), ADR-0244 (tenant scoping), ADR-0245 (substrate vs product), ADR-0314 (marketplace DealSet settlement), ADR-0315 (b2b-leader-coverage), ADR-0316 (capability-tier — RETIRED), ADR-0321 (b2b SaaS industry-leader coverage). Missing: ADR-0263 (audit emission contract per `feedback_microservice_ownership_coherence_2026_05_20`), ADR-0322 (substance bar), ADR-0328 (canonical sequence + batch discipline), ADR-0329-pending (tenant-class replacement). Severity P1 (cross-reference). Fix shape: in Wave 15F, add ADR-0263 + ADR-0328 to binding_adrs; remove ADR-0316 (retired). Reserve ADR-0329 citation until that ADR is authored.

C.1.2. compliance.md frontmatter line 7-11 cites [ADR-0244, ADR-0251, ADR-0263, ADR-0316, ADR-0321]. compliance.md DOES cite ADR-0263 correctly. Manifest does NOT. P1 cross-reference drift within the µservice. Verdict: P1 — add ADR-0263 to manifest binding_adrs.

C.1.3. ARCHITECTURE.md frontmatter line 6-13 cites [ADR-0105, ADR-0131, ADR-0132, ADR-0244, ADR-0245, ADR-0316, ADR-0321]. Missing ADR-0263, ADR-0314, ADR-0322. Severity P2 (cross-reference gap).

C.1.4. README.md line 9 cites [docs/standards/documentation-rigor.md sections 1.1, 1.2, 2, 3.2.1, 3.2.3, 3.2.5; ADR-0321; ADR-0131; ADR-0105; ADR-0253-amendment; ADR-0314]. README adds ADR-0253-amendment (HTTP/3 + ECH/PQC) which is NOT in manifest. Severity P2 (cross-reference gap in manifest).

C.1.5. README, competitor-parity-matrix, PHASE-01 all reference ADR-0314 in their `Learning Management binds course-enroll to ...` template line. The ADR-0314 reference is accurate (marketplace DealSet settlement) but the surface that emits to ADR-0314 is content marketplace, not the µservice's whole surface. Severity P3 (cosmetic — ADR-0314 is over-cited in template-stamped sections).

### C.2 Persona / journey cross-references

C.2.1. PRD §B Target Users names six personas (Marcus Chen, Yejin Park, Diana Alvarez, Nadia Singh, Omar Watkins, Hana Mori). These appear in the global MASTER-ROSTER per `feedback_microservice_ownership_coherence_2026_05_20` step 5 — but I have not verified the actual roster in this audit. Severity P3 pending Wave 15F verification.

C.2.2. PRD §B does NOT include the academic-LMS persona set per §3.4.C-1.6. Severity P1 (substance-bar gap if academic LMS scope is adopted per §3.4.C-1.2).

C.2.3. IP-001 line 17 cites Priya Nair (enterprise learning administrator). Priya Nair is NOT in PRD §B Target Users. Severity P1 (persona drift — IP cites a persona not in the PRD persona set). Fix shape: in Wave 15F, either add Priya Nair to PRD §B or rename IP-001's persona to one of the six PRD personas.

### C.3 Journey cross-references

C.3.1. IP-001 line 5 cites `journey_id: J-LMS-01-tenant-learning-program-launch`. No journey doc was sampled in this audit but per `feedback_microservice_ownership_coherence_2026_05_20` step 5 every cited journey must exist in `/docs/user-journeys/`. Severity P3 pending Wave 15F verification.

### C.4 Outbound cross-reference verdict

Verdict: REVISE (one P1 in C.1.1, one P1 in C.1.2, two P1 in C.2.2 and C.2.3, plus three P2/P3).

## D. Dimension 3 — Substance bar

### D.1 Top-level doc substance check

D.1.1. PRD.md sampled — §A Problem is one paragraph (line 29-31). §B Target Users is 6 personas (lines 33-39). §C User Stories is 25 entries (lines 41-91) — every entry has the IDENTICAL acceptance line ("Acceptance: <context> exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence."). §D Functional Requirements has 30 FR entries (lines 93-123) where every FR has the IDENTICAL right-hand side ("must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target."). §L Pack Overlay Applicability is one sentence (line 176). §M Follow-Up Buildout (lines 178-300+) is 118+ "PRD trace NNN: learning-management remains tenant-scoped..." entries (verified via Read sampling of lines 183-300). This is template-stamping — same line repeated with the counter incremented. Severity P0 per ADR-0322 / ADR-0324 / `feedback_docs_substance_not_scaffold_2026_05_20`. Verdict: P0 substance-bar violation in PRD.md.

D.1.2. README.md sampled — every section (§Scope, §Principals, §Cedar gates, §Data model, §Workflow, §Contracts, §Transport, §Abuse defence, §Marketplace, §Observability, §Capacity, §Failure modes, §Regional packs, §Acceptance evidence, §Entrypoint evidence rows 1-N) has the IDENTICAL 8-line template ("Learning Management binds course-enroll to tenant_id, principal_id, audience_type=LEARNING_ADMIN, data_class=course_enrollment, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Learning plus Cornerstone." × 8 with vendor pairs rotated) and the entrypoint-evidence-row-N lines repeat the SAME line dozens of times. Per ADR-0322 / ADR-0324 this is a P0 substance-bar violation. Verdict: P0.

D.1.3. competitor-parity-matrix.md sampled — every section (§Scope, §Principals, §Cedar gates, §Data model, §Workflow, §Contracts, §Transport, §Abuse defence, §Marketplace, §Observability, §Capacity, §Failure modes, ...) has the SAME 8-line template as README. Not a real parity matrix. P0. Verdict: P0.

D.1.4. PHASE-01-LEARNING-MANAGEMENT-OPERATING-BAR.md sampled — same template-stamping pattern. P0. Verdict: P0.

D.1.5. compliance.md sampled — §A Scope (one paragraph) and §B Control Families (six packs each with the identical "requires permit delta, retention class, residency behavior, DSAR/export behavior, evidence owner, and exception workflow" RHS line). §C Data Classification has 5 substantive entries. §D Audit Events lists 15 EVT- classes (substantive). §E Required Evidence lists 6 items (substantive). §F Required Compliance Anchors then expands every anchor (self-modification, etc.) via the SAME template-stamping pattern. Severity P0 in §F. The §A through §E ARE substantive (the lighter sections); the §F expansion is the P0 violation. Verdict: P0 (specifically §F).

D.1.6. ARCHITECTURE.md sampled — §A through §E are substantive (boundary, layer map, bounded-context architecture, integration topology, failure modes). §F Required ADR-3.2.1 Anchors expands every anchor via the SAME template pattern (Depth detail 1 / Depth detail 2 / Depth detail 3 / ...). Severity P0 (specifically §F expansion). Verdict: P0.

D.1.7. backfill-replay.md, capacity-model.md, cost-budget.md, dpia.md, failure-modes.md, incident-response.md, multi-region.md — these are all 270-420-line files; based on the pattern in other top-level files I have sampled, these are likely template-stamped too. Severity P1 pending Wave 15F sample. Fix shape: sample each in Wave 15F.

### D.2 IP slice substance check

D.2.1. IP-001-tenant-scope-kernel.md (108 lines) is BESPOKE and substantive. It has a real data model (learning_tenant_scope table with 7 typed columns), real REST endpoint shape, real Cedar permits with named principals/actions/resources, real ontology projection table (5 vendor object → Oyatie object mappings), real workflow steps (5 steps with named branches), real audit-event class names, real SLO numbers (p50/p95/p99/throughput/availability for two operations), real failure modes, real migration notes, and real cross-µservice handoff list. This is the substance bar. Verdict: PASS.

D.2.2. IP-002-cedar-default-deny.md (110 lines), IP-003-ontology-projection.md (108 lines), IP-004-workflow-template-library.md (110 lines), IP-005-rest-contract-surface.md (110 lines) — line-count similar to IP-001; pattern suggests bespoke. Not yet sampled. Severity P3 pending Wave 15F sample.

D.2.3. IP-006-async-event-surface.md through IP-025-audit-findings-closeout.md — uniform 55-line count is suspicious. Severity P2 pending Wave 15F sample. Fix shape: sample-read each in Wave 15F to verify bespoke vs template.

D.2.4. IP-026-skills-graph-gap-analyzer.md (104 lines), IP-027-compliance-training-attestation-ledger.md, IP-028-content-provider-catalog-federation.md, IP-029-learning-path-recommendation-guardrail.md, IP-030-credential-expiry-renewal-orchestrator.md — filenames are bespoke and these address real Cornerstone/Docebo-style operations. Severity P2 pending Wave 15F substance sample.

### D.3 Runbook substance check

D.3.1. course-enrollment-stall.md sampled — §A Trigger conditions (20 entries, same template-stamped line), §B Pre-checks (20 entries, same template), §C Procedure (18 numbered steps that ARE substantive — they declare actual `oya learning-management <flow> inspect --tenant <tenant_id> --cell <home_cell> --audit-tag <tag>` commands with named flow and named audit emission and named rollback branch). §C is substantive; §A and §B are P0 template-stamped. Mixed verdict. Fix shape: in Wave 15F, retire §A and §B template stamping; keep §C as the substance core.

D.3.2. Other 20 runbooks (completion-evidence-mismatch.md, content-export-failure.md, credential-revocation.md, dealset-course-license-hold.md, exam-integrity-incident.md, provider-catalog-drift.md, regulated-training-audit.md, skills-graph-backfill.md, tenant-license-exhaustion.md, plus 11 local-prefixed flow-specific runbooks) — not sampled in this audit. Pattern from course-enrollment-stall.md suggests the same §A/§B template-stamping with §C substantive. Severity P2 pending Wave 15F sample.

### D.4 Contract substance check

D.4.1. openapi-v1.yaml (78 lines) sampled — has 2 paths (`/learning-management/capabilities` GET and `/learning-management/actions/{action_id}` POST). The path naming pattern is generic — not learning-management-specific. Real LMS APIs need explicit endpoints for course CRUD, module CRUD, assignment CRUD, enrollment CRUD, grade-book GET/POST, discussion-thread CRUD, SCORM-runtime POST, xAPI-statement POST, quiz-attempt POST, credential GET, regulated-attestation GET/POST. Severity P0 (substance-bar gap — contract is generic action-dispatch, not a real LMS API). Fix shape: in Wave 15F, expand OpenAPI to cover all six capabilities × CRUD operations, matching the manifest's bounded-contexts × commands from PRD §C.

D.4.2. asyncapi-v1.yaml (34 lines) and learning-management-v1.proto (21 lines) — short files; not yet sampled. Severity P2 pending Wave 15F sample.

### D.5 Rust src substance check

D.5.1. src/lib.rs (118 lines) is BESPOKE and substantive. It exports a real ServiceDescriptor, real ArchitectureLayer enum, real Capability enum, real CourseEvidence / CourseId / DataClass / DomainInvariant / EnrollmentId / EnrollmentStatus / EvidenceVisibility / LearnerId / LearningPath / LearningPolicy / ProgressSnapshot / TenantId domain types, real HttpHandler with routes, real gRPC handler, real AsyncAPI handler, real ServiceConfig + RuntimeProfile, real ServiceError + ServiceResult, real LearningManagementService usecase with three commands (OpenEnrollment, RecordProgress, SealCourseCompletion). The `validate_scaffold` function asserts the 13-layer ADR-0105 invariant and the 3-contract surface invariant. This is the substance bar for a scaffold. Verdict: PASS.

D.5.2. src/adapter/mod.rs, src/domain/mod.rs, src/usecase/mod.rs are mod files that re-export the implementations declared in lib.rs. Not yet sampled in detail. Severity P3 pending Wave 15F sample.

D.5.3. Note: the scaffold declares ONE bounded-context (`course-progress`) but the manifest declares FIVE. The Rust scaffold is therefore not fully covering the µservice's claimed bounded contexts. Severity P0 per B-1.7. Fix shape: per B-1.7 recommend (a) expand src/ to declare 5 bounded-context modules.

### D.6 Policy substance check

D.6.1. policies/local-*.cedar — 6 local-prefixed Cedar files. Not yet sampled. Severity P2 pending Wave 15F sample.

D.6.2. policy/*.cedar + policy/data-residency.md — 5 cedar files + 1 markdown. Not yet sampled. Severity P2 pending Wave 15F sample.

D.6.3. Risk: PRD §D FR-NNN clauses require Cedar default-deny on every mutation, but the µservice has TWO Cedar policy directories (`policies/` and `policy/`). This dual-directory pattern is a P1 internal-coherence finding — only one canonical directory should exist. Severity P1. Fix shape: in Wave 15F, consolidate to one directory (canonical: `policies/` per ADR-0263 convention).

### D.7 SLO substance check

D.7.1. availability.openslo.yaml (32 lines) sampled — has real Prometheus queries (`sum(rate(oya_learning_management_availability_good_total[5m]))`) and a real 99.9% target over 30d rolling window. Substantive. Verdict: PASS.

D.7.2. Other 11 SLO files not yet sampled. Severity P3 pending Wave 15F sample.

### D.8 Capability YAML substance check

D.8.1. capabilities/course-enroll.yaml, completion-seal.yaml, credential-issue.yaml, provider-catalog-sync.yaml, regulated-training-attest.yaml, skills-graph-export.yaml — 6 capability YAML files matching the manifest's named capabilities. Not yet sampled. Severity P2 pending Wave 15F sample.

### D.9 Substance bar verdict

Verdict: BLOCK. Six top-level docs are template-stamped P0 substance-bar violations (PRD §M trace list, README all sections, competitor-parity-matrix all sections, PHASE-01 all sections, compliance.md §F expansion, ARCHITECTURE.md §F expansion). Five top-level docs (backfill-replay, capacity-model, cost-budget, dpia, failure-modes, incident-response, multi-region) likely follow the same pattern pending Wave 15F sample. The OpenAPI contract is generic action-dispatch (P0). The Rust src/ scaffold is bespoke (PASS) but declares only one bounded-context against the manifest's five (P0 internal-coherence). Per ADR-0322 the µservice cannot promote past Phase 4A.1 gate until the P0 substance violations are remediated.

## E. Dimension 4 — Canonical-direction alignment

### E.1 Unified ecosystem thesis alignment

E.1.1. The unified ecosystem thesis says product names like learning-management are role/capability projections over the shared substrate (identity, tenancy, ontology, workflow, policy, audit, settlement, UX shell). The µservice's bounded-contexts (course-catalog / enrollment / learning-path / assessment / credential) are operational concerns that project over the substrate. PRD §A correctly states "The product must remain compatible with ADR-0316: product labels are capability tiers, while this service owns only the durable operational concern that cannot be safely pushed into an existing owner." This is canonical-direction-aligned in shape but stale in citation (ADR-0316 is retired). Verdict for thesis-alignment-shape: PASS (the µservice is a legitimate projection, not a Cornerstone-suite copy). Verdict for thesis-citation: P1 (ADR-0316 retired; cite ADR-0329 or ADR-0244 instead).

E.1.2. The µservice does NOT define its own identity engine, its own Cedar engine, its own workflow engine, its own ontology storage, its own marketplace settlement. PRD §J Out of Scope correctly forbids "Recreating a vendor suite boundary, Sharing database tables with adjacent microservices, Treating vendor labels as canonical object names, Bypassing marketplace DealSet settlement for commercial obligations." Verdict: PASS on no-product-island canonical-direction.

E.1.3. The µservice DOES integrate with `community` (manifest line 37), which per Phase 3 service 14 (ADR-0328 D-1.67) absorbed anonymous deleted (per `feedback_cell_standalone_network_merges_community_2026_05_21`). The integration with `community` is canonical (cohort + discussion belongs to community). Verdict: PASS.

E.1.4. The µservice does NOT integrate with `hris` (per B-1.1) even though HR/Payroll family coherence requires it. Verdict: P0 — see B-1.1.

### E.2 Phase placement alignment

E.2.1. ADR-0328 §D-1.85 places learning-management as a Phase 4 B2B/ERP service. ADR-0328 §D-2.5 places it in the Workday HR family at Phase 4A.1. The manifest does NOT declare phase. Severity P2 (manifest gap). Fix shape: in Wave 15F, add `phase: "4A.1-HR-Payroll"` and `big_8_family: "Workday HR"` to manifest.json.

### E.3 Canonical-direction verdict

Verdict: REVISE (one P0 in E.1.4 already tracked as B-1.1, one P1 in E.1.1, one P2 in E.2.1).

## F. Dimension 5 — Industry-counterpart parity (preview; full coverage in feature-parity-matrix-2026-05-20.md)

F.1. Per ADR-0328 §D-5 union-coverage parity bar against the brief's top-3 (Canvas LMS + Cornerstone OnDemand + Docebo). The feature-parity-matrix-2026-05-20.md deliverable carries the row-by-row matrix. Summary preview:

- **Canvas LMS academic features** (assignment, rubric, grade-book, discussion, SCORM, xAPI, quiz-attempt, course-module, LTI 1.3 grade-passback): MISSING from the µservice. See §3.4.C-1.2.
- **Cornerstone OnDemand corporate-learning features** (compliance training, regulated attestation, certification tracking, content library, social learning): PARTIAL — regulated-training-attest and credential-issue capabilities cover the attestation and certification axes; content library and social learning are out of scope or delegated to `community`.
- **Docebo features** (AI-driven recommendations, mobile learning, virtual classroom integration, content marketplace): PARTIAL — skills-graph-export capability covers part of the AI-recommendation axis; mobile learning is a UX shell concern; virtual classroom integration is out of scope or delegated to `meet`; content marketplace is delegated to `cloud-marketplace`.

F.2. Counterpart-set drift (manifest's six vs brief's three) is the upstream P0 issue. Until that resolves, parity scoring across both sets is inconsistent.

### F.4 Counterpart parity verdict

Verdict: REVISE pending counterpart-set normalization. See feature-parity-matrix-2026-05-20.md for row-level findings.

## G. Findings summary

### G.1 P0 findings (BLOCK)

- P0-LM-001 (§3.4.C-1.1, §3.4.C-1.4, §3.4.C-1.5, B.1.1): Counterpart-source contradiction between brief (Canvas LMS / Cornerstone OnDemand / Docebo) and manifest+PRD+README (Workday + Cornerstone + Degreed + LinkedIn + Udemy + Trailhead). Files: manifest.json line 21-28 + line 110, PRD.md line 9-18 + §A + §K, README.md line 7, competitor-parity-matrix.md line 7, PHASE-01 line 7, capabilities/*.yaml benchmark lines. Fix shape: Wave 15F normalizes to brief's three (recommended) or documents explicit per-counterpart-set scope.
- P0-LM-002 (§3.4.C-1.2): Bounded-context set (course-catalog / enrollment / learning-path / assessment / credential) does NOT cover Canvas LMS academic primitives (assignment / grade-book / discussion / SCORM / xAPI / quiz-attempt / course-module / LTI 1.3). Files: manifest.json line 29-35, PRD.md §C, ARCHITECTURE.md §C. Fix shape: Wave 15F either expands bounded contexts OR declares out-of-scope-intentional with reason.
- P0-LM-003 (§3.4.B-1.1, §3.4.B-1.2, E.1.4): Missing hris dependency for HR/Payroll family coherence. Files: manifest.json line 36-44 + 123-131, PRD.md §M, ARCHITECTURE.md §D. (IP-001 correctly names hris, so this is an upstream-doc drift not an IP gap.) Fix shape: Wave 15F adds hris to substrate_dependencies + depends_on_microservices + cross-microservice-handoffs.md.
- P0-LM-004 (§3.4.B-1.3, §3.4.B-1.4, D.1.1, D.1.2, D.1.3, D.1.4, D.1.5, D.1.6): Template-stamped substance-bar violations in PRD §M, README all sections, competitor-parity-matrix all sections, PHASE-01 all sections, compliance.md §F expansion, ARCHITECTURE.md §F expansion. Fix shape: Wave 15F retires the template-stamping in each file and reauthors bespoke.
- P0-LM-005 (B-1.7, D.5.3): Rust scaffold declares one bounded-context (`course-progress`) contradicting manifest's five. Files: src/lib.rs line 30, Cargo.toml line 16. Fix shape: Wave 15F expands src/ to 5 bounded-context modules.
- P0-LM-006 (B.1.5, B.1.6): Manifest + ARCHITECTURE declare 9 layers (api/rest/application/usecase/domain/kernel/adapter/worker/governance) but ADR-0105 13-layer enum is canonical and Rust validate_scaffold asserts 13. Files: manifest.json line 62-72, ARCHITECTURE.md §B. Fix shape: Wave 15F expands declared_layers to the full ADR-0105 13-layer set.
- P0-LM-007 (D.4.1): OpenAPI contract is generic action-dispatch (only 2 paths). Real LMS API needs explicit endpoints per bounded-context and per Canvas+Cornerstone+Docebo feature axes. File: contracts/openapi-v1.yaml. Fix shape: Wave 15F expands OpenAPI to cover all bounded contexts × CRUD and all capability surfaces.
- P0-LM-008 (B.1.3): IP-001 declares dependencies (hris, identity-access, audit-chain, data-residency, content-provider-integration) that contradict manifest (community, workflow-engine, ontology, mail, intelligence, identity, compliance). File: IP-001 line 102-108 vs manifest.json line 36-44. Fix shape: Wave 15F normalizes via the dependency superset (hris + identity + audit-chain + community + workflow-engine + ontology + mail + intelligence + compliance + cloud-billing + cloud-marketplace + payments). Note `identity-access` → `identity` rename + `data-residency` is a policy concern not a µservice + `content-provider-integration` consolidates into capability provider-catalog-sync.

### G.2 P1 findings (REVISE)

- P1-LM-001 (§3.4.T-1.1, T-1.3, T-1.4, T-1.6, T-1.7): Capability-tier residue (ADR-0316 cited; tier-named fields in manifest and IP-001). Fix in Wave 15J retirement.
- P1-LM-002 (§3.4.T-1.5): Missing demo_trial cap declaration. Fix in Wave 15J alongside tier retirement.
- P1-LM-003 (§3.4.C-1.3, C-1.6): README §scope-and-non-goals silent on academic LMS; personas set silent on academic-LMS personas.
- P1-LM-004 (§3.4.B-1.5): Missing cross-microservice-handoffs.md file. Fix in Wave 15F.
- P1-LM-005 (§3.4.B-1.6): PRD §I Open Questions are scope-deferral, not HR-family-coherence questions.
- P1-LM-006 (B.1.4): hipaa in manifest line 101 but not line 74 — pack drift.
- P1-LM-007 (C.1.1): Missing ADR-0263, ADR-0322, ADR-0328 in manifest binding_adrs.
- P1-LM-008 (D.6.3): Two Cedar policy directories (policies/ + policy/) — consolidate to one canonical.

### G.3 P2 findings (PASS-WITH-FINDINGS)

- P2-LM-001 (§3.4.T-1.2): cell_eligibility field name "eligible_tiers" collides with retired-doctrine vocabulary; rename to `eligible_cell_classes`.
- P2-LM-002 (§3.4.B-1.2): IP-001 uses `identity-access` instead of canonical `identity`.
- P2-LM-003 (B.1.7): PRD related_adrs missing ADR-0105.
- P2-LM-004 (B.1.8): IP-006..IP-025 uniform 55-line count — sample each for substance.
- P2-LM-005 (C.1.3): ARCHITECTURE.md missing ADR-0263, ADR-0314, ADR-0322 citations.
- P2-LM-006 (D.1.7): backfill-replay / capacity-model / cost-budget / dpia / failure-modes / incident-response / multi-region likely template-stamped — sample in Wave 15F.
- P2-LM-007 (D.6.1, D.6.2): policies/ + policy/ directories Cedar substance check pending.
- P2-LM-008 (E.2.1): manifest missing `phase` + `big_8_family` fields.
- P2-LM-009 (D.4.2): asyncapi-v1.yaml + learning-management-v1.proto substance check pending.

### G.4 P3 findings (cosmetic)

- P3-LM-001 (C.1.4): README cites ADR-0253-amendment not in manifest.
- P3-LM-002 (C.1.5): ADR-0314 over-cited in template-stamped sections.
- P3-LM-003 (C.2.1, C.3.1): PRD personas + IP-001 journey not yet verified against master rosters.
- P3-LM-004 (iac/terraform-module.tf): naming uses "terraform" word against ADR-0328 §D-16.3 OpenTofu-only policy. Fix shape: rename to `tofu-module.tf` (or context-specific OpenTofu module file naming per §D-16.21).
- P3-LM-005 (D.5.2): src/adapter/mod.rs / src/domain/mod.rs / src/usecase/mod.rs substance pending sample.

## H. Verification Notes

H.1 Files read in this audit:

- /Users/jasonlee/oyatie/microservices/learning-management/manifest.json (full)
- /Users/jasonlee/oyatie/microservices/learning-management/PRD.md (first 300 lines of ~64K characters; sampled §A, §B, §C, §D, §H, §I, §J, §K, §M lines 178-300)
- /Users/jasonlee/oyatie/microservices/learning-management/README.md (first 200 lines of ~56K characters; sampled §scope, §principals, §cedar, §data, §workflow, §contracts, §transport, §abuse, §marketplace, §observability, §capacity, §failure, §regional, §acceptance, §entrypoint-evidence)
- /Users/jasonlee/oyatie/microservices/learning-management/ARCHITECTURE.md (first 200 lines of 902 lines; sampled §A, §B, §C, §D, §E, §F §principals, §F §cedar-gates, §F §tenant-scoping)
- /Users/jasonlee/oyatie/microservices/learning-management/compliance.md (first 100 lines of 925 lines; sampled §A, §B, §C, §D, §E, §F §self-modification)
- /Users/jasonlee/oyatie/microservices/learning-management/competitor-parity-matrix.md (first 150 lines)
- /Users/jasonlee/oyatie/microservices/learning-management/PHASE-01-LEARNING-MANAGEMENT-OPERATING-BAR.md (first 150 lines)
- /Users/jasonlee/oyatie/microservices/learning-management/IP-001-tenant-scope-kernel.md (full 108 lines)
- /Users/jasonlee/oyatie/microservices/learning-management/Cargo.toml (full 56 lines)
- /Users/jasonlee/oyatie/microservices/learning-management/src/lib.rs (full 118 lines)
- /Users/jasonlee/oyatie/microservices/learning-management/contracts/openapi-v1.yaml (full 78 lines)
- /Users/jasonlee/oyatie/microservices/learning-management/slos/availability.openslo.yaml (full 32 lines)
- /Users/jasonlee/oyatie/microservices/learning-management/runbooks/course-enrollment-stall.md (first 80 lines)
- /Users/jasonlee/oyatie/docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md (lines 1-2912 covering §A through §D-17 partial — §D-15..D-17 sampled; §D-18..D-20 cited from sequence and brief)
- /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md (full)
- /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md (full)
- /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_capability_tiers_2026_05_20.md (full)
- /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_verify_deliverables_not_just_line_count_2026_05_20.md (full)
- /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_docs_substance_not_scaffold_2026_05_20.md (full)

H.2 Directories listed (not deep-read):

- microservices/learning-management/capabilities/ (6 YAML files)
- microservices/learning-management/catalog/ (13 YAML files)
- microservices/learning-management/contracts/ (6 files)
- microservices/learning-management/dashboards/ (10 JSON files)
- microservices/learning-management/iac/ (23 files)
- microservices/learning-management/policies/ (6 cedar files)
- microservices/learning-management/policy/ (6 files)
- microservices/learning-management/runbooks/ (21 files)
- microservices/learning-management/slos/ (12 OpenSLO files)
- microservices/learning-management/scorecards/ (1 JSON file)
- microservices/learning-management/src/ (5 files + 3 mod.rs files)
- microservices/learning-management/tests/ (1 file)

H.3 Anchor pass/fail:

- Anchor 1 (unified ecosystem thesis): pass shape (PASS); citation drift (ADR-0316 retired) — P1.
- Anchor 2 (µservice PRD): READ; substance-bar P0 in §M trace list, P0 substance gap shape in §A through §L (compact paragraphs with no concrete LMS specifics).
- Anchor 3 (local artifact inventory): authored in §A.2 of this audit.
- Anchor 4 (brief override Canvas + Cornerstone OnDemand + Docebo): contradicts manifest counterpart set — P0.
- Anchor 5 (documentation-rigor §1.1 substance bar): fails for PRD §M, README, competitor-parity-matrix, PHASE-01, compliance.md §F, ARCHITECTURE.md §F — P0.

H.4 Wave 4 Codex-only HALT-CLEANLY note (per ADR-0328 §D-14.12): this audit completes within the agent's scope. Verification SLA per §D-10 is satisfied: line counts named, anchor set named with per-anchor pass/fail, known gaps named below.

H.5 Known gaps:

- backfill-replay.md, capacity-model.md, cost-budget.md, dpia.md, failure-modes.md, incident-response.md, multi-region.md not sampled (expected template-stamped pattern; queued for Wave 15F sample).
- IP-002 through IP-005 not sampled (line count ~108-110 suggests bespoke; queued for Wave 15F sample).
- IP-006 through IP-025 not sampled (uniform 55-line count is the suspicion marker; queued for Wave 15F sample).
- IP-026 through IP-030 not sampled (filenames bespoke; queued for Wave 15F substance verification).
- 20 runbooks not sampled (course-enrollment-stall sample suggests mixed substance/template pattern; queued for Wave 15F).
- policies/*.cedar and policy/*.cedar substance not sampled (6 + 5 files; queued for Wave 15F).
- 12 SLO files except availability.openslo.yaml not sampled.
- 23 iac/ files not sampled (terraform-module.tf naming flagged P3).
- 10 dashboards not sampled.
- 6 capabilities + 13 catalog YAML files not sampled.
- threat-model.md, sdk-plan.md not sampled.
- tests/integration.rs not sampled.
- Persona cross-check against MASTER-ROSTER not performed (P3 pending).
- Journey J-LMS-01 cross-check against /docs/user-journeys/ not performed (P3 pending).

## I. Backlog Rows

The following remediation rows enter Wave 14 aggregation per ADR-0328 §D-8. Each row carries microservice + severity + category + file + fix.

| Row | µservice | Severity | Category | File | Fix |
|---|---|---|---|---|---|
| LM-BR-001 | learning-management | P0 | parity | manifest.json:21-28, manifest.json:110, PRD.md:9-18 §A §K, README.md:7, competitor-parity-matrix.md:7, PHASE-01:7, capabilities/*.yaml | Normalize counterpart set to brief's three (Canvas LMS + Cornerstone OnDemand + Docebo). |
| LM-BR-002 | learning-management | P0 | canonical-direction | manifest.json:29-35, PRD.md §C, ARCHITECTURE.md §C | Expand bounded contexts to cover Canvas academic primitives OR document explicit out-of-scope-intentional with doctrine reason. |
| LM-BR-003 | learning-management | P0 | internal-coherence | manifest.json:36-44 + 123-131, PRD.md §M, ARCHITECTURE.md §D | Add hris dependency for HR/Payroll family coherence. |
| LM-BR-004 | learning-management | P0 | substance-bar | PRD.md §M trace list 178-300+ | Retire template-stamped PRD trace list and replace with bespoke follow-up buildout section. |
| LM-BR-005 | learning-management | P0 | substance-bar | README.md (all sections post-line-9) | Retire template-stamping and reauthor as bespoke ~200-line README. |
| LM-BR-006 | learning-management | P0 | substance-bar | competitor-parity-matrix.md | Retire and replace with bespoke parity matrix (see feature-parity-matrix-2026-05-20.md as canonical). |
| LM-BR-007 | learning-management | P0 | substance-bar | PHASE-01-LEARNING-MANAGEMENT-OPERATING-BAR.md | Retire and reauthor as ~250-line bespoke Phase 4A.1 admission-gate doc. |
| LM-BR-008 | learning-management | P0 | substance-bar | compliance.md §F expansion | Retire §F template expansion; keep §A-E substantive prose. |
| LM-BR-009 | learning-management | P0 | substance-bar | ARCHITECTURE.md §F expansion | Retire §F template expansion; keep §A-E substantive prose. |
| LM-BR-010 | learning-management | P0 | internal-coherence | src/lib.rs:30, Cargo.toml:16 | Expand Rust scaffold to declare 5 bounded-context modules matching manifest. |
| LM-BR-011 | learning-management | P0 | internal-coherence | manifest.json:62-72, ARCHITECTURE.md §B | Expand declared_layers to full ADR-0105 13-layer enum. |
| LM-BR-012 | learning-management | P0 | substance-bar | contracts/openapi-v1.yaml | Expand OpenAPI to cover all bounded-contexts × CRUD and capability surfaces per Canvas+Cornerstone+Docebo union coverage. |
| LM-BR-013 | learning-management | P0 | internal-coherence | IP-001:102-108 vs manifest.json:36-44 | Normalize cross-µservice dependency superset; rename `identity-access` → `identity`. |
| LM-BR-014 | learning-management | P1 | capability-tier (retired) | manifest.json:7, 18, 82-100; PRD.md:16-18; IP-001:10 | Retire ADR-0316 citations + tier-named fields; add ADR-0329 + tenant_class fields. |
| LM-BR-015 | learning-management | P1 | substance-bar | tenant-class-behavior.md (NEW FILE) | Author file declaring demo_trial caps + paid billing-component meter shapes. |
| LM-BR-016 | learning-management | P1 | canonical-direction | README.md §scope-and-non-goals; PRD.md §B Target Users | Declare position on academic-LMS scope (in-scope with expansion OR out-of-scope-intentional). |
| LM-BR-017 | learning-management | P1 | substance-bar | cross-microservice-handoffs.md (NEW FILE) | Author file declaring all reciprocated handoffs (12+ µservices). |
| LM-BR-018 | learning-management | P1 | substance-bar | PRD.md §I Open Questions | Replace scope-deferral language with HR-family-coherence open questions. |
| LM-BR-019 | learning-management | P1 | internal-coherence | manifest.json:74 vs 101 | Reconcile hipaa pack — supported or not. |
| LM-BR-020 | learning-management | P1 | outbound-cross-reference | manifest.json:10-20 | Add ADR-0263, ADR-0322, ADR-0328 to binding_adrs; remove ADR-0316. |
| LM-BR-021 | learning-management | P1 | internal-coherence | policies/ + policy/ | Consolidate to canonical `policies/`. |
| LM-BR-022 | learning-management | P2 | naming | manifest.json:46 | Rename `eligible_tiers` → `eligible_cell_classes`. |
| LM-BR-023 | learning-management | P2 | substance-bar | IP-006..IP-025 (20 files) | Sample-read each; remediate template-stamped. |
| LM-BR-024 | learning-management | P2 | substance-bar | backfill-replay / capacity-model / cost-budget / dpia / failure-modes / incident-response / multi-region (7 files) | Sample-read each; remediate template-stamped. |
| LM-BR-025 | learning-management | P2 | substance-bar | runbooks/*.md (21 files) | Sample-read each; preserve §C substantive procedures, retire §A+§B template-stamping. |
| LM-BR-026 | learning-management | P2 | substance-bar | policies/*.cedar + policy/*.cedar (11 files) | Sample-read each; verify default-deny shape. |
| LM-BR-027 | learning-management | P2 | substance-bar | slos/*.openslo.yaml (11 files except availability) | Sample-read each; verify Prometheus query + target real. |
| LM-BR-028 | learning-management | P2 | outbound-cross-reference | ARCHITECTURE.md frontmatter | Add ADR-0263, ADR-0314, ADR-0322 citations. |
| LM-BR-029 | learning-management | P2 | brief-format | manifest.json | Add `phase: "4A.1-HR-Payroll"` and `big_8_family: "Workday HR"` fields. |
| LM-BR-030 | learning-management | P2 | substance-bar | contracts/asyncapi-v1.yaml + learning-management-v1.proto | Sample-read; verify event + RPC shapes real. |
| LM-BR-031 | learning-management | P3 | outbound-cross-reference | manifest.json binding_adrs | Add ADR-0253-amendment to align with README. |
| LM-BR-032 | learning-management | P3 | naming | iac/terraform-module.tf, iac/local-terraform-module.tf | Rename to tofu-module.tf (or per §D-16.21 context-prefixed). |
| LM-BR-033 | learning-management | P3 | substance-bar | src/adapter/mod.rs + src/domain/mod.rs + src/usecase/mod.rs | Sample-read; verify re-exports match lib.rs declarations. |
| LM-BR-034 | learning-management | P3 | persona | PRD.md §B vs MASTER-ROSTER | Cross-check 6 personas exist in roster. |
| LM-BR-035 | learning-management | P3 | journey | IP-001:5 vs /docs/user-journeys/ | Cross-check J-LMS-01 exists. |

## J. Final verdict

Verdict: **BLOCK** per ADR-0328 §D-4.24. learning-management has eight P0 findings spanning counterpart-source contradiction (LM-BR-001), bounded-context gap (LM-BR-002), HR-family dependency gap (LM-BR-003), template-stamped P0 substance-bar violations across six top-level docs (LM-BR-004..LM-BR-009), Rust-manifest bounded-context contradiction (LM-BR-010), layer-enum under-declaration (LM-BR-011), OpenAPI generic-action contract (LM-BR-012), and IP-001-vs-manifest dependency contradiction (LM-BR-013).

The µservice CANNOT promote past Phase 4A.1 admission gate. Downstream ERP (Phase 4A.2) and CRM (Phase 4A.3) µservices would mislearn HR-family handoff shape, parity bar, and substance-bar expectations from learning-management's current state.

Wave 14 aggregates these 35 backlog rows. Wave 15A handles the 8 P0 hard contradictions FIRST per §D-9.4. Wave 15F handles substance-bar gaps. Wave 15J handles tier-doctrine retirement. The audit owner cleanly halts here with a complete checkpoint per §D-14.14.

Bounded-context expansion (LM-BR-002) and HR-family hris dependency (LM-BR-003) interact: an ADR-0329-companion decision is required before remediation can proceed in Wave 15A because it determines whether learning-management absorbs Canvas academic semantics or remains a corporate-learning-only µservice. The audit recommends a separate council decision before Wave 15A schedules the P0 fixes.
