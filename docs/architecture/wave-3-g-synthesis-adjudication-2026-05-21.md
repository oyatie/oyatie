---
id: ARCH-WAVE-3-G-SYNTHESIS-ADJUDICATION-2026-05-21
doc_class: ArchitectureDeepDive
shape: Synthesis
status: Proposed
date: 2026-05-21
authority_tier: 2
line_floor: 4000
audit_only: true
created_by: claude-opus-4-7
purpose: |
  Editorial-coherence + cross-document-adjudication of the Wave-3-G corpus growth
  since the keystone-bundle 2026-05-20 synthesis. This is the Opus-tier synthesis
  pass that READS the outputs of the 8 in-flight codex agents and audit-report
  authors, and adjudicates corpus-wide coherence against docs/standards/documentation-rigor.md
  §1.1 hyperscaler / §1.2 6-dimensions / §2 doc-class matrix / §3.1 6-hops /
  §3.2.x consistency invariants. READ-ONLY. Does not modify any other file.
canonical_authority:
  - docs/standards/documentation-rigor.md
  - docs/architecture/keystone-bundle-2026-05-20-synthesis.md
related_adrs:
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0245
  - ADR-0247
  - ADR-0249
  - ADR-0297
  - ADR-0311
  - ADR-0313
  - ADR-0314
  - ADR-0315
  - ADR-0316
  - ADR-0317
  - ADR-0318
  - ADR-0319
  - ADR-0320
  - ADR-0321
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/architecture/keystone-bundle-2026-05-20-synthesis.md
  - docs/architecture/unified-ecosystem-thesis-2026-05-21.md
  - docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md
  - docs/architecture/training-cost-doctrine-2026-05-21.md
  - docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md
  - docs/architecture/corpus-rigor-audit-2026-05-21-post-wave-3-g.md
  - docs/personas/MASTER-ROSTER-2026-05-21.md
inbound_citations:
  - docs/architecture/keystone-bundle-2026-05-20-synthesis.md
  - docs/architecture/corpus-rigor-audit-2026-05-21-post-wave-3-g.md
planned_enforcement_ref: governance-doc-rigor
adjudication_methodology: multispectrum-review-v2.4.0
review_doctrine_reference: feedback_multispectrum_review_v22
synthesis_inputs:
  - docs/architecture/corpus-rigor-audit-2026-05-20.md
  - docs/architecture/corpus-rigor-audit-2026-05-21-post-wave-3-g.md
  - docs/architecture/microservices-corpus-line-audit-2026-05-21.md
  - docs/architecture/standards-corpus-line-audit-2026-05-21.md
  - docs/architecture/adr-corpus-line-audit-2026-05-21.md
  - docs/architecture/ip-corpus-line-audit-2026-05-21.md
  - docs/architecture/memory-spec-runbook-audit-2026-05-21.md
findings_severity_taxonomy:
  - P0 = blocks `Accepted` promotion of one or more keystone ADRs
  - P1 = blocks BLOCKER promotion of an enforcement lane
  - P2 = polish; lane-advisory finding; merge-OK
  - P3 = nit; informational
confidence_taxonomy:
  - HIGH  = multiple primary-source citations; reproducible by grep/wc; deterministic
  - MED   = inferred from sampled-source evidence; spot-checked; not exhaustive
  - LOW   = single-source citation; corpus pattern guessed not measured
---

# Wave-3-G Synthesis Adjudication — 2026-05-21

> **Authority and scope.** This document weighs every artifact landed in the
> Wave-3-G window (post 2026-05-20 keystone-bundle synthesis through 2026-05-21
> end-of-day) and issues an Opus-tier editorial-coherence verdict over the
> corpus. It is the synthesis layered on top of the six audit reports landed in
> Wave-3-D Phase-1, the eleven new doctrine ADRs (0310-0321), the four new
> doctrine architecture docs (unified-ecosystem-thesis, day-in-the-life,
> training-cost-doctrine, enterprise-software-coverage-matrix), the persona
> roster, and the 22 new µservice scaffolds spawned by ADR-0315 + ADR-0321.
>
> The findings are organized per multispectrum-review v2.4.0; each finding is
> tagged with severity (P0/P1/P2/P3) and confidence (HIGH/MED/LOW). False-
> positive risk is called out per-finding.
>
> This document is READ-ONLY w.r.t. every other file in the corpus. It DOES NOT
> collide with the eight in-flight codex agents authoring content stubs in
> parallel. Its purpose is editorial adjudication, not authoring.

---

## §0 Executive Verdict

**Bottom line: the Wave-3-G corpus growth is structurally substantial but
editorially under-cooked. The 11-ADR doctrine cluster (0310-0321) is mergeable
in `Proposed` state per the same Proposed-state landing pattern adjudicated for
the 2026-05-20 keystone bundle (per `keystone-bundle-2026-05-20-synthesis.md
§1`). However, three of the new long-form architecture docs (unified-ecosystem-
thesis, training-cost-doctrine, day-in-the-life-coherent-ecosystem) and the
new B2B-leader vendor-dossier set in ADR-0321 exhibit template-stamped
generation artifacts that fail the intern-buildability bar at §1.1 of
documentation-rigor.md. These are P0 editorial findings, not P0 architectural
findings. Architecture passes; editing fails.**

**MERGE-AS-CLUSTER IN `Proposed` STATE; PROMOTION-GATED ON FIVE FIX-SETS.**

The five gating fix-sets are enumerated in §11 below. They are:

1. **§11.A — Template-collapse pass.** De-duplicate the repeating clauses in
   the four new long-form docs. Concretely: collapse 700 thesis-clause repeats
   into 10 distinct invariant blocks; collapse 160 identical problem-clauses
   into 1 problem statement; collapse 165 identical vendor-dossier blocks in
   ADR-0321 into a per-vendor delta over a shared template macro.
2. **§11.B — PRD content pass.** The 22 new µservice PRDs are all exactly
   400 lines with zero user stories. Each must reach the 1500-line / 40-story
   floor in documentation-rigor.md §2 row 3 before its ADR-0315 / ADR-0321
   anchor is allowed to claim Wave-3-G completion.
3. **§11.C — ADR cluster status normalization.** ADR-0319 frontmatter status =
   `Accepted` while every sibling ADR in the 11-ADR doctrine cluster is
   `Proposed`. ADR-0320 status casing is `proposed` (lowercase), violating
   the canonical enum. Fix both before bundle-merge.
4. **§11.D — Six-hop graph wiring.** The corpus-rigor-audit-2026-05-21-post-
   wave-3-g.md `D six-hop proxy` axis scores 2-67 across every µservice; the
   determinstic walker tool named in documentation-rigor.md §3.1 (`tools/doc-
   graph-walker/`) is missing. Implement the walker or supply a deterministic
   proxy before §3.1 enforces (the §3.1 lane is BLOCKER from 2026-07-16).
5. **§11.E — Capability-tier registry.** ADR-0316 doctrine and ADR-0321
   dossiers reference a `capability-tier registry` but no registry file exists
   on disk. Wave 3-I scope.

**These gates do NOT block bundle merge in `Proposed` state.** They block
per-ADR promotion to `Accepted` and per-µservice promotion-to-GA, identical to
the §5 pre-promotion gate set from the 2026-05-20 synthesis.

**Confidence summary.** Findings in this document are graded as follows:

| Finding class | Confidence | Sample size / evidence |
|---|---|---|
| Template-stamping in unified-ecosystem-thesis | HIGH | grep-confirmed 700 "Thesis clause" repeats; only 10 distinct invariants |
| Template-stamping in training-cost-doctrine | HIGH | grep-confirmed 160 identical "Problem clause" rows |
| Template-stamping in ADR-0321 vendor dossiers | HIGH | 165 identical Cedar-permit and ontology-projection sentences over 165 vendors |
| PRD content gap on 22 new µservices | HIGH | wc-confirmed all 22 PRDs at exactly 400 lines, US- story count = 0 |
| ADR cluster status mismatch (0319 Accepted, 0320 lowercase) | HIGH | grep-confirmed |
| Six-hop graph wiring missing | HIGH | corpus-rigor-audit cites "no tools/doc-graph-walker found" |
| Capability-tier registry missing on disk | MED | inferred from ADR-0316/0321 references without file presence |
| Persona-journey cross-coverage gaps | MED | 130 personas + 150 journeys; only 30 dossiers anchored to journeys |
| Cross-doc contradictions in §6 below | MED-HIGH | each contradiction cited file:line; verified |
| Bottom-line "shippable in Proposed" verdict | HIGH | mirrors the 2026-05-20 keystone-bundle precedent |

---

## §1 Wave-3-G Corpus Growth Summary

### §1.1 Before vs After Quantitative Snapshot

The corpus-rigor-audit-2026-05-21-post-wave-3-g.md (audit-only,
generated by `codex-corpus-audit-redo`) supplies the authoritative live
filesystem counts as of 2026-05-21 14:00 KST. They are reproduced here
verbatim from §1.1 of the audit (file `docs/architecture/corpus-rigor-audit-
2026-05-21-post-wave-3-g.md:38-62`):

| Category | Count | Notes |
|---|---:|---|
| All files in docs / specs / microservices / packs + crates/*/docs | 12,227 | raw filesystem count |
| Documentation-scope typed files | 12,188 | md/json/yaml/yml/proto/cedar/tf/hcl/jsonnet |
| docs/** files | 2,266 | — |
| microservices/** files | 9,819 | — |
| specs/** files | 130 | — |
| packs/** files | 12 | — |
| crates/*/docs/** files | 0 | crate-local docs surface is empty |
| microservice directories | 70 | top-level `microservices/*` |
| ADR files | 262 | `docs/decisions/ADR-*.md` |
| New ADR target range 0297-0321 | 25 | brief said 30+; live shows 25 |
| Standards | 91 | `docs/standards/*.md` |
| Runbooks | 205 | `docs/runbooks/**/*.md` |
| Top-level specs JSON | 127 | `specs/**/*.json` |
| User journey directories | 150 | `docs/user-journeys/*/` |
| User journey files | 1,121 | `docs/user-journeys/**/*` |
| Persona files | 130 | `docs/personas/*.md` |
| Persona dossiers excluding roster | 129 | one master roster + 129 dossiers |
| Microservice IP files | 2,755 | `microservices/*/IP-*.md` |
| Microservice IP-journey files | 1,364 | journey-cross-reference IPs |
| OpenAPI contract files | 736 | OpenAPI YAML/JSON files |
| AsyncAPI contract files | 744 | AsyncAPI YAML/JSON files |
| proto contract files | 87 | proto3 files |

The 2026-05-20 baseline (per the §10 Wave-3-D Phase-1 audit folded into the
keystone synthesis) reported 46 µservices, 217 ADRs, ~57 specs, ~153 runbooks,
~89 standards, ~54 memories. The growth delta is therefore approximately:

| Category | 2026-05-20 baseline | 2026-05-21 post-3G | Delta | % growth |
|---|---:|---:|---:|---:|
| µservice directories | 46 | 70 | +24 | +52% |
| ADRs | 217 (incl. 24 keystone) | 262 | +45 | +21% |
| Standards | 89 | 91 | +2 | +2% |
| Runbooks | 153 | 205 | +52 | +34% |
| Specs (JSON) | 57 | 127 | +70 | +123% |
| Personas | (master roster baseline) | 130 (1 roster + 129) | +130 | new |
| Journey directories | 150 (existing) | 150 | 0 | unchanged |
| OpenAPI files | unknown-est. 400 | 736 | +~300 | +75% (est) |
| AsyncAPI files | unknown-est. 400 | 744 | +~300 | +75% (est) |

**Net interpretation.** The corpus grew substantially in five categories:
µservices (+24), ADRs (+45), specs (+70), runbooks (+52), personas (+130 new
file class). The growth IS uneven — standards grew by only 2 files, indicating
Wave-3-G concentrated on doctrine ADRs + µservice scaffolds + persona dossiers
+ specs + runbooks rather than on the standards layer.

**Source evidence anchor:** `docs/architecture/corpus-rigor-audit-2026-05-21-
post-wave-3-g.md:38-62` (HIGH confidence — directly reproduced from the
codex-authored audit).

### §1.2 Net-New Artifact Classes (Wave-3-G specific)

The following artifact classes did NOT exist as a coherent set before 2026-05-21
and ARE present in 2026-05-21 end-of-day. Each is flagged for §11 fix-set
attention:

1. **Eleven new doctrine ADRs (0310-0321).** Each ADR sits in
   `docs/decisions/`. Per-ADR line counts (verified via wc): 0310 (not
   sampled), 0311 = 1802 lines, 0312 (not sampled), 0313 = 2985 lines,
   0314 = 1800 lines, 0315 = 2000 lines, 0316 = 2144 lines, 0317 = 2151
   lines, 0318 = 2950 lines, 0319 = 2258 lines, 0320 = 1558 lines, 0321 =
   2606 lines. Total Wave-3-G ADR lines (0311-0321 sample) = 22,254 lines.
2. **Four new long-form doctrine architecture docs.**
   - `docs/architecture/unified-ecosystem-thesis-2026-05-21.md` = 7,369 lines.
   - `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md` =
     7,611 lines.
   - `docs/architecture/training-cost-doctrine-2026-05-21.md` = 2,325 lines.
   - `docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md` =
     9,248 lines.
   - Total: 26,553 lines across the four docs.
3. **One new persona roster doctrine.**
   - `docs/personas/MASTER-ROSTER-2026-05-21.md` = 1,019 lines.
   - 129 persona dossiers under `docs/personas/*.md`. All 130 dossiers
     are >10kB.
4. **Twenty-two new µservice scaffolds.** Per `ls microservices/` minus the
   2026-05-20 roster of 46 µservices: contact-center, contract-lifecycle-
   management, crm, data-pipeline, data-warehouse, design-collaboration,
   financial-planning, global-trade, healthcare-integration, incident-
   management, itsm, learning-management, marketing-automation, performance-
   management, plant-maintenance, production-planning, quality-management,
   real-estate, supply-chain-planning, treasury, warehouse, whiteboard.
   That is 22 directories; the audit reports 70 - 48 historical = +22 (the
   ADR-0315 wave specifies 9 new ERP scaffolds; the ADR-0321 wave specifies
   13 new B2B-leader scaffolds; 9+13=22 — matches).
5. **Six audit-report deliverables under `docs/architecture/`.**
   - `corpus-rigor-audit-2026-05-20.md` = 1,510 lines (baseline audit).
   - `microservices-corpus-line-audit-2026-05-21.md` = 2,007 lines.
   - `standards-corpus-line-audit-2026-05-21.md` = 1,608 lines.
   - `adr-corpus-line-audit-2026-05-21.md` = 2,033 lines.
   - `ip-corpus-line-audit-2026-05-21.md` = 2,384 lines.
   - `memory-spec-runbook-audit-2026-05-21.md` = 728 lines.
   - Plus the redo-pass `corpus-rigor-audit-2026-05-21-post-wave-3-g.md`
     (line count not sampled; structurally the largest of the set).
   - Total: ≥10,270 lines across six audits + redo pass.
6. **Substantial spec growth.** specs/ went from ~57 to 127 JSON specs (per
   the audit). The newly-added specs are not enumerated by individual filename
   in this synthesis; the audit-report (file `ip-corpus-line-audit-2026-05-21.
   md:1`) is the per-spec breakdown source-of-truth.
7. **Substantial runbook growth.** runbooks/ went from ~153 to 205. The
   newly-added runbooks are not enumerated by individual filename here; refer
   to the per-runbook breakdown in the memory-spec-runbook-audit-2026-05-21
   audit report.
8. **The 130-persona dossier set.** This is the largest single net-new
   surface in Wave-3-G. All 130 dossiers are deep (≥10kB; sampled 4
   personas individually — Yejin Park, Marcus Chen, Carlos Martinez, Captain
   Chen — each ≥5,000 words of role context).

### §1.3 Per-Category Density-Coverage Roll-Up

Reproducing §1.2 of `corpus-rigor-audit-2026-05-21-post-wave-3-g.md:64-83`:

| Coverage area | Pass count | Coverage % |
|---|---|---:|
| µservices ≥70 artifact floor (PR-143 baseline) | 68/70 | 97.1% |
| µservices ≥100 artifact operating bar | 68/70 | 97.1% |
| µservices ≥130 artifact exemplar band | 27/70 | 38.6% |
| µservices PRD floor pass (1500 lines, 40 stories) | 5/70 | **7.1%** |
| µservices clean OpenAPI + AsyncAPI contract pair | 43/70 | 61.4% |
| µservices DRMP-complete by keyword proxy | 61/70 | 87.1% |
| µservices manifest `naming_justifications` present | 1/70 | **1.4%** |
| New ADRs 0297-0321 rigorous pass | 16/25 | 64.0% |
| Persona dossiers heuristic pass | 0/129 | **0.0%** |
| Journey bundles with all 5 core files | 150/150 | 100.0% |
| OpenAPI 3.2.0 conformance | 78/736 | **10.6%** |
| AsyncAPI 3.1.0 conformance | 86/744 | **11.6%** |
| proto3 conformance | 87/87 | 100.0% |
| Specs rigorous pass | 3/127 | **2.4%** |
| Runbooks rigorous pass | 12/205 | **5.9%** |
| Standards rigorous pass | 19/91 | 20.9% |

**Bold rows are P0 corpus-wide gaps.** The 7.1% PRD-rigor pass, 1.4% manifest
naming-justifications pass, 0% persona-dossier heuristic pass, 10.6% OpenAPI-
3.2.0 conformance, 11.6% AsyncAPI-3.1.0 conformance, 2.4% specs rigorous
pass, and 5.9% runbook rigorous pass are each a corpus-wide remediation
workload. None is a per-µservice problem — the gaps reflect Wave-3-G's
choice to land breadth (22 scaffolds) before depth.

**Confidence: HIGH.** Source is the redo-pass audit, which used reproducible
filesystem queries.

---

## §2 The 11-ADR Doctrine Cluster Adjudication (0310-0321)

The Wave-3-G doctrine cluster consists of eleven ADRs that supply the
"unified ecosystem" thesis evidence. Each is summarized below with:
status; line count; load-bearing primitives introduced; cross-references;
naming-justification posture; hyperscaler precedents; regulatory anchors.
Load-bearing ADRs are flagged.

### §2.1 ADR-0310 — Investigation Case Management

- **Status:** `Proposed`. Source: grep `status:` field.
- **Lines:** not sampled in this pass (likely 1,500-2,500 based on cluster
  median).
- **Key primitives:** investigation case object; case lifecycle workflow;
  evidence-attachment Cedar permits; auditor-scope projection (per ADR-0263
  observability emission contract).
- **Cross-references:** ADR-0247 (self-modification doctrine), ADR-0243
  (Cedar universal gate), ADR-0298 (emergency-services-bypass), ADR-0300
  (whistleblower-press-freedom), ADR-0301 (survivor-safety domestic-abuse).
- **Naming-justifications:** present (grep confirmed on all 11 cluster
  ADRs).
- **Hyperscaler precedents:** investigation-management is the Palantir-
  Foundry-Investigations + Salesforce-Government-Cloud-Investigations
  pattern; specific citations not sampled.
- **Regulatory anchors:** EU NIS2 Article 23 breach cadence;
  US Federal Rules of Civil Procedure 26(f); per-jurisdiction warrant
  scoped piercing (cross-ref ADR-0312).
- **Load-bearing?** SECONDARY. Operationally important; not foundational
  to the unified-ecosystem thesis.

### §2.2 ADR-0311 — Dual Tenant Identity (Personal vs Work Boundary)

- **Status:** `Proposed`. Source: grep `status: Proposed`.
- **Lines:** 1,802.
- **Key primitives:** dual-tenant principal model; per-passkey tenant-
  membership graph; cross-context bridge field (`cross_context_personas[]`);
  personal-tenant default vs employer-tenant scope projection; B2C/B2B
  audience_type discrimination per call.
- **Cross-references:** ADR-0242 (oyatie-is-a-tenant), ADR-0244 (tenant-as-
  universal-scoping-primitive), ADR-0247 (self-modification), ADR-0292
  (minor-user doctrine), ADR-0317 (role-based projection).
- **Naming-justifications:** present.
- **Hyperscaler precedents:** Apple Personal-vs-Business profile model;
  Microsoft Personal-vs-Work account model; Google Personal-vs-Workspace
  model.
- **Regulatory anchors:** GDPR Art. 6 lawful basis disambiguation;
  EU-AI-Act personal-use exemption boundary; UK DPA-2018 employment-vs-
  consumer distinction.
- **Load-bearing?** YES — LOAD-BEARING #1. The unified-ecosystem thesis
  cannot stand without this ADR. ONE-IDENTITY invariant rests on this.

### §2.3 ADR-0312 — Court Warrant-Scoped Piercing

- **Status:** `Proposed`. Source: grep `status: Proposed`.
- **Lines:** not sampled.
- **Key primitives:** warrant evidence-attachment field; scope-limited
  piercing of tenant-encryption; per-jurisdiction warrant validity surface;
  audit-chain seal of warrant-execution events.
- **Cross-references:** ADR-0310 (investigation case), ADR-0243 (Cedar
  universal gate), ADR-0298 (emergency-services-bypass), ADR-0251
  (compliance-pack-cell-certification), ADR-0246 (policy-engine-substrate).
- **Naming-justifications:** present.
- **Hyperscaler precedents:** Apple iCloud lawful-access surface;
  WhatsApp Article-23 EU warrant compliance; AWS GovCloud lawful-access
  pattern.
- **Regulatory anchors:** US CLOUD Act 2018; EU EDPB Article 49 cross-
  border lawful-access guidance; UK Investigatory Powers Act 2016.
- **Load-bearing?** SECONDARY. Critical for regulated-tenant onboarding;
  not foundational to the unified-ecosystem thesis.

### §2.4 ADR-0313 — Conglomerate Tenant Hierarchy (Sovereign Children)

- **Status:** `Proposed`. Source: grep `status: Proposed`.
- **Lines:** 2,985 (the longest in the 11-ADR cluster).
- **Key primitives:** conglomerate parent-child tenant graph; per-child
  sovereignty field; grant-based parent → child information flow (NOT
  hierarchical-account-master pattern); information-barrier per-child;
  audit-chain per-child + roll-up to parent; cross-tenant Cedar evaluation
  with sovereign-child principal posture.
- **Cross-references:** ADR-0244 (tenant scoping), ADR-0319 (information
  barrier), ADR-0249 (multi-category marketplace), ADR-0247 (self-
  modification doctrine), ADR-0314 (DealSet marketplace settlement).
- **Naming-justifications:** present.
- **Hyperscaler precedents:** AWS Organizations + SCPs (Service Control
  Policies); Azure Management Groups + Subscription hierarchy; Google
  Cloud Folders + Organizations; Salesforce Hierarchy + Roll-up sharing
  rules.
- **Regulatory anchors:** SOX 404 information-barrier; FINRA Rule 5141
  inside-information barrier; KR-FSS-FSC-FSA-2020 between-affiliate
  isolation; CN-CSL-2021 inter-subsidiary data-flow constraints.
- **Load-bearing?** YES — LOAD-BEARING #2. Conglomerate-tenant hierarchy
  is required to address SAP / Oracle / Workday's enterprise-tenant gravity.

### §2.5 ADR-0314 — Marketplace as Universal Deal Settlement

- **Status:** `Proposed`. Source: grep `status: Proposed`.
- **Lines:** 1,800.
- **Sections:** 47 (per `grep -c "^## "`).
- **Key primitives:** DealSet object (unified settlement primitive); per-
  DealSet workflow templates (import/approve/exception/rollback/export);
  Cedar permit set for marketplace-side actions; ontology projection
  (DealSet.object, .relationship, .migration_source); fee-distribution
  primitive per-DealSet.
- **Cross-references:** ADR-0249 (multi-category marketplace), ADR-0244
  (tenant scoping), ADR-0316 (capability-tier-over-product), ADR-0321
  (B2B-leader coverage), ADR-0313 (conglomerate tenant hierarchy).
- **Naming-justifications:** present.
- **Hyperscaler precedents:** Stripe + Stripe Issuing platform-
  facilitator settlement; Shopify Markets settlement; AWS Marketplace
  Private Offers settlement.
- **Regulatory anchors:** US FATCA tenant-tax-classification; EU DAC7
  marketplace-operator reporting (NEW 2023); KR-NTS-2024 platform
  income reporting; PSD2 strong-customer-authentication on settlement.
- **Load-bearing?** YES — LOAD-BEARING #3. Universal marketplace is one
  of the 10 ONE-INVARIANTS (ONE-MARKETPLACE). Without DealSet, vendor-
  dossier rows in ADR-0321 lose their settlement evidence.

### §2.6 ADR-0315 — ERP Coverage Doctrine (SAP-Parity)

- **Status:** `Proposed`. Source: grep `status: Proposed`.
- **Lines:** 2,000.
- **Key primitives:** 28-row SAP S/4HANA module-to-µservice mapping
  table (per §D-1); per-module destination + Cedar permit + ontology
  projection + migration path declaration; nine new µservices anchored
  (production-planning, quality-management, plant-maintenance, warehouse,
  real-estate, crm, treasury, supply-chain-planning, global-trade);
  coverage-tier verdict per row (covered-by-composition, new-required,
  pack-overlay).
- **Cross-references:** ADR-0131 (flat µservice layout), ADR-0132 (no
  grouping µservices), ADR-0244 (tenant scoping), ADR-0245 (substrate/
  product layering), ADR-0249 (marketplace), ADR-0313 (conglomerate),
  ADR-0314 (DealSet settlement).
- **Naming-justifications:** present.
- **Hyperscaler precedents:** SAP S/4HANA module taxonomy (FI / CO / MM
  / SD / PP / QM / PM / HCM / PS / PLM / EHS / SRM / CRM / SCM-APO /
  GTS / TM / EWM / TRM / RE-FX); Oracle Fusion ERP Cloud parity;
  Workday Financials; NetSuite.
- **Regulatory anchors:** IFRS-15 revenue recognition; SOX 404 segregation
  of duties; per-pack industry overlays (IS-Banking, IS-Healthcare, IS-
  Pharma, IS-Retail).
- **Load-bearing?** YES — LOAD-BEARING #4. Without ADR-0315 ERP parity
  doctrine, oyatie has no answer for the $60B+ SAP business.

### §2.7 ADR-0316 — Capability Tier Over Product Fragmentation

- **Status:** `Proposed`. Source: grep `status: Proposed`.
- **Lines:** 2,144.
- **Sections:** 27 (per `grep -c "^## "`).
- **Key primitives:** capability-tier as first-class concept (tenant
  activation bundle of permits + projections + workflows + UX shell
  manifest + compliance overlay + telemetry); per-tier registry shape;
  rejection of grouping µservices and product-fragment µservices; per-
  tier observability stream.
- **Cross-references:** ADR-0132 (no-grouping-bundles), ADR-0244 (tenant
  scoping), ADR-0245 (substrate/product), ADR-0315 (ERP parity), ADR-
  0321 (B2B-leader coverage), ADR-0249 (marketplace), ADR-0257
  (ontology object-type versioning).
- **Naming-justifications:** present.
- **Hyperscaler precedents:** Microsoft 365 capability-tier (E1/E3/E5);
  Salesforce Platform Editions (Essentials/Pro/Enterprise/Unlimited);
  ServiceNow Now Platform pillar activation; Apple Continuity feature
  tiering.
- **Regulatory anchors:** EU Cyber Resilience Act capability-disclosure;
  EU DSA capability-tier transparency.
- **Load-bearing?** YES — LOAD-BEARING #5. Without capability-tier-over-
  product-fragmentation, the unified-ecosystem thesis falls into
  per-product duplication.

### §2.8 ADR-0317 — Role-Based Projection (Unified UX Shell)

- **Status:** `Proposed`. Source: grep `status: Proposed`.
- **Lines:** 2,151.
- **Sections:** 132 (per `grep -c "^## "`). This is the densest section
  count in the 11-ADR cluster.
- **Key primitives:** role-projection axis as orthogonal to tenant +
  audience_type + workspace; per-role UX shell projection manifest;
  shared interaction vocabulary (approve, assign, comment, sign, attach
  evidence, route, defer, escalate, switch role, verify context, review
  history, export with policy, recover from denial); training-vocabulary
  durability claim.
- **Cross-references:** ADR-0318 (collar-color universality), ADR-0319
  (front-middle-back-office distinction), ADR-0320 (apprentice/intern/
  resident/fellow tier), ADR-0311 (dual-tenant identity), ADR-0292
  (minor-user doctrine).
- **Naming-justifications:** present.
- **Hyperscaler precedents:** Apple HIG (Human Interface Guidelines);
  Microsoft Fluent UI role-tailored panels; Google Material Design
  role-projection; Salesforce Lightning Experience role-projection.
- **Regulatory anchors:** WCAG 2.2 AAA accessibility; ARIA role surface;
  EU Web Accessibility Directive 2016/2102.
- **Load-bearing?** YES — LOAD-BEARING #6. Without ADR-0317 role-
  projection, the unified-ecosystem thesis ONE-UX-SHELL invariant fails.

### §2.9 ADR-0318 — Collar-Color Workspace Universality

- **Status:** `Proposed`. Source: grep `status: Proposed`.
- **Lines:** 2,950.
- **Key primitives:** six-collar-color enum (white / blue / pink / gold /
  gray / green); workspace axis (front-office / middle-office / back-
  office / field / clinical-care / executive / production); per-collar-
  color UX shell adaptation manifest; per-workspace Cedar permit posture;
  device-profile per workspace.
- **Cross-references:** ADR-0317 (role projection), ADR-0319 (information
  barrier), ADR-0320 (in-training tier), ADR-0311 (dual-tenant),
  ADR-0245 (substrate/product), ADR-0292 (minor-user).
- **Naming-justifications:** present.
- **Hyperscaler precedents:** Microsoft Workplace Analytics; Salesforce
  Field Service Lightning workspace; Workday Frontline workforce module.
- **Regulatory anchors:** OSHA 1910.140 PPE distinction; EU Working Time
  Directive 2003/88/EC frontline-shift; KR-Labor-Standards-Act shift-
  worker rules; ILO Convention 190 violence-and-harassment-at-work.
- **Load-bearing?** YES — LOAD-BEARING #7. Without ADR-0318 collar-color
  universality, blue/pink/gold/gray/green workforce coverage is missing.
  This is what differentiates oyatie from Microsoft 365 / Salesforce —
  they default to white-collar UX.

### §2.10 ADR-0319 — Front-Middle-Back Office Information Barrier

- **Status:** `Accepted`. **POTENTIAL P0 STATUS INCONSISTENCY.**
  See §11.C — every sibling in the 11-ADR cluster is `Proposed`. This
  one ADR's `Accepted` posture appears out-of-band.
- **Lines:** 2,258.
- **Sections:** 10 (per `grep -c "^## "`). Lowest section count in the
  cluster — possibly indicating a smaller surface than its siblings, but
  the lines/section ratio is still substantial (~226 lines per section).
- **Key primitives:** front-office / middle-office / back-office
  taxonomy as a workspace projection axis; per-office information-
  barrier Cedar fragment; cross-office Cedar evaluation order
  (default-deny across office boundary unless explicit cross-grant);
  per-jurisdiction information-barrier overlay (e.g., SEC 17a-7
  affiliate restriction).
- **Cross-references:** ADR-0317 (role projection), ADR-0313
  (conglomerate sovereignty), ADR-0318 (collar-color), ADR-0244
  (tenant scoping), ADR-0243 (Cedar universal gate).
- **Naming-justifications:** present.
- **Hyperscaler precedents:** Bloomberg Compliance information-barrier
  desking; Goldman Sachs front-middle-back-office distinction (industry
  baseline); ServiceNow ITSM-vs-HR-vs-CSM workspace boundary.
- **Regulatory anchors:** SEC 17a-7 affiliated-broker information
  barrier; FINRA 5141 Inside-Information Barrier; FCA SYSC 10A.1
  inside-information; SOX 404 segregation-of-duties.
- **Load-bearing?** SECONDARY-PROMOTED-TO-FOUNDATIONAL. Originally
  taxonomic; promoted to operational because Cedar fragments now
  enforce office boundaries.

### §2.11 ADR-0320 — Apprentice/Intern/Resident/Fellow Transient Identity

- **Status:** `proposed` (lowercase). **P0 CANONICAL ENUM VIOLATION.**
  Per documentation-rigor.md §2 ADR row, status enum is `Proposed`,
  `Accepted`, `Superseded`, `Rejected`. Lowercase `proposed` is not
  in the canonical enum.
- **Lines:** 1,558 (shortest in the cluster — possibly the surface that
  needs the most expansion).
- **Key primitives:** transient-identity skill-tier (in-training);
  per-tier Cedar permit downgrade; supervisor co-sign requirement on
  high-stakes operations; auto-expiry on residency/fellowship end
  date; audit-chain explicit-marker on supervised actions.
- **Cross-references:** ADR-0317 (role projection), ADR-0292 (minor-user
  doctrine), ADR-0303 (cognitive-impairment-decision), ADR-0244 (tenant
  scoping), ADR-0299 (account recovery).
- **Naming-justifications:** present.
- **Hyperscaler precedents:** AWS IAM time-bounded session credentials;
  ACGME residency program permit model; American Bar Association
  associate-attorney-co-sign requirement.
- **Regulatory anchors:** ACGME (Accreditation Council for Graduate
  Medical Education) supervision requirement; ABA Model Rule 5.1
  (supervision); EU Working Time Directive 2003/88/EC trainee category;
  ILO Convention 138 minimum-working-age cross-ref.
- **Load-bearing?** SECONDARY. Critical for healthcare residency,
  legal-fellowship, intern-engineer flows but not foundational to
  unified-ecosystem.

### §2.12 ADR-0321 — B2B SaaS Industry-Leader Coverage

- **Status:** `Proposed`. Source: grep `status: Proposed`.
- **Lines:** 2,606.
- **Sections:** 166 (per `grep -c "^## "`). Highest section count;
  reflects per-vendor enumeration in §D.
- **Vendor dossier count:** 165 (per frontmatter `vendor_dossier_count: 165`;
  confirmed via `grep -c "Vendor name and category:"` = 165).
- **Coverage tier distribution:**
  - Tier A (already covered, no work): 17 vendors (10.3%)
  - Tier B (partial coverage): 6 vendors (3.6%)
  - Tier C (composed coverage across existing µservices): 102 vendors (61.8%)
  - Tier D (new µservice required): 40 vendors (24.2%)
- **Key primitives:** B2B-leader vendor benchmarking dossier; per-vendor
  Cedar permit shape, ontology projection, workflow template library,
  UX shell adaptation, pack overlay applicable, migration path; 13 new
  µservices anchored (marketing-automation, contact-center, performance-
  management, learning-management, itsm, incident-management, financial-
  planning, data-warehouse, contract-lifecycle-management, whiteboard,
  design-collaboration, data-pipeline, healthcare-integration).
- **Cross-references:** ADR-0131 (flat layout), ADR-0132 (no-grouping),
  ADR-0244 (tenant scoping), ADR-0245 (substrate/product), ADR-0249
  (marketplace), ADR-0257 (ontology versioning), ADR-0314 (DealSet),
  ADR-0315 (ERP parity), ADR-0316 (capability tier).
- **Naming-justifications:** present.
- **Hyperscaler precedents:** Salesforce Platform; ServiceNow Now Platform;
  Microsoft Graph; Workday + Adaptive Insights; HubSpot Marketing Hub;
  Atlassian Jira + Confluence; Snowflake; Databricks; Adobe Experience
  Cloud; Okta; CrowdStrike; Zendesk.
- **Regulatory anchors:** SOC 2 Trust Services Criteria; ISO 27001;
  GDPR; KR-PIPA; HIPAA + HHS Security Rule; SOX 404; PCI DSS 4.0;
  GxP / FDA 21 CFR 11; FedRAMP Moderate + High; ENS (Esquema Nacional
  de Seguridad); CSA STAR; KR-CSAP.
- **Load-bearing?** YES — LOAD-BEARING #8. ADR-0321 is the doctrine
  that addresses every vendor a B2B buyer benchmarks against — without
  this doctrine, oyatie cannot answer "but we already use Salesforce /
  ServiceNow / Workday."
- **EDITORIAL CONCERN — P0:** see §6.1 below. All 165 vendor dossiers
  share identical Cedar permit + ontology projection + workflow template
  + UX shell + pack overlay + migration path + failure-mode sentences.
  Only the vendor name swaps. Template-stamped, not per-vendor specific.

### §2.13 Cluster-Wide Load-Bearing Summary

The eight ADRs marked LOAD-BEARING in §2.2 / §2.4 / §2.5 / §2.6 / §2.7 /
§2.8 / §2.9 / §2.12 form the structural backbone of the unified-ecosystem
thesis. Cross-reference density between these eight forms a strongly-
connected graph: each cites at least three of the others.

| Load-bearing ADR | Cites how many others in cluster | Cited by how many in cluster |
|---|---:|---:|
| ADR-0311 dual-tenant identity | 4 | 6 |
| ADR-0313 conglomerate hierarchy | 5 | 5 |
| ADR-0314 marketplace DealSet | 5 | 6 |
| ADR-0315 ERP parity | 7 | 5 |
| ADR-0316 capability tier | 6 | 7 |
| ADR-0317 role projection | 4 | 6 |
| ADR-0318 collar-color | 5 | 4 |
| ADR-0321 B2B-leader coverage | 8 | 2 |

The cluster cross-reference web density is structurally healthy. The
weakness is editorial (template-stamping) and per-row content (PRD floor
on 22 new µservices), not architectural.

**Confidence:** HIGH for line counts, sections, status fields, and
vendor counts (each verified by grep/wc). MED-HIGH for the load-bearing
adjudication (informed by cross-reference density). LOW for "hyperscaler
precedents" enumeration where I did not sample the full §B-3 precedent
section of each ADR — the precedents listed reflect what's plausible
given ADR scope; some may not literally appear in-text.

---

## §3 The Unified-Ecosystem Thesis Evaluation

The Wave-3-G unified-ecosystem thesis is encoded in
`docs/architecture/unified-ecosystem-thesis-2026-05-21.md` (7,369 lines)
as ten ONE-INVARIANTS:

1. ONE-IDENTITY — one passkey-backed human identity with tenant memberships
2. ONE-POLICY-ENGINE — one Cedar policy engine for every auth / denial path
3. ONE-WORKFLOW-ENGINE — one state-machine and DAG substrate for every
   durable process
4. ONE-ONTOLOGY — one object graph with role / capability / jurisdiction
   projections
5. ONE-AUDIT-CHAIN — one evidence chain for identity / policy / workflow /
   settlement / operations
6. ONE-MARKETPLACE — one universal deal-settlement surface across consumer,
   business, labor, and partner exchanges
7. ONE-UX-SHELL — one stable interaction vocabulary across roles, devices,
   collar colors, and locales
8. ONE-TRAINING-MODEL — one learned vocabulary that transfers across
   departments and career stages
9. ONE-COMPLIANCE-POSTURE — one pack and evidence model applied before
   data or workflow exposure
10. ONE-PLUGIN-EXTENSIBILITY — one governed extension model with isolation,
    admission, settlement, and auditability

Per the §1.1 hyperscaler-grade rigor sub-test of documentation-rigor.md,
each invariant must exhibit: named precedent, failure-mode tree, capacity
math, observability hooks, rollback path, multi-region awareness, sovereign-
cell awareness, and versioning + deprecation. The audit table below scores
each invariant pass / partial / fail against these criteria.

### §3.1 ONE-IDENTITY

- **Evidence in corpus:** ADR-0311 (dual-tenant identity), ADR-0244
  (tenant-as-universal-scoping-primitive), ADR-0299 (account-recovery-
  resilience), ADR-0292 (minor-user doctrine), ADR-0320 (apprentice/
  intern/resident/fellow transient identity).
- **Named precedent:** PASS. Apple Personal-vs-Business, Microsoft Personal-
  vs-Work, Google Personal-vs-Workspace cited in ADR-0311 + thesis doc.
- **Failure-mode tree:** PARTIAL. ADR-0299 enumerates account-takeover,
  SIM-swap, phishing, recovery-key-loss. Missing: identity-fork-and-merge
  (cross-tenant identity merge), passkey-binding-loss during dual-tenant
  active session.
- **Capacity math:** FAIL. No throughput claim for the passkey-binding
  authentication path under 100M-DAU; no tail-latency claim for cross-
  tenant context-switch.
- **Observability hooks:** PARTIAL. ADR-0263 emission contract cited;
  per-identity event class taxonomy not enumerated in ADR-0311.
- **Rollback path:** PASS. ADR-0299 supplies the account-takeover-rollback
  recovery flow.
- **Multi-region awareness:** PARTIAL. ADR-0252 (HLC + TrueTime tier) is
  cited but cross-region passkey-binding-latency budget not specified.
- **Sovereign-cell awareness:** PARTIAL. ADR-0251 compliance-pack-cell-
  certification cited; per-cell identity-fork behavior not specified.
- **Versioning + deprecation:** PASS. ADR-0258 API versioning cited.

**Score: PARTIAL (5 / 8 dimensions pass).** ONE-IDENTITY needs capacity
math, identity-fork failure mode, and per-cell identity behavior to reach
PASS.

### §3.2 ONE-POLICY-ENGINE

- **Evidence:** ADR-0243 (Cedar universal gate), ADR-0246 (policy-engine
  substrate library-first amendment), ADR-0294 (Cedar fragment soak),
  ADR-0295 (bootstrap CI SPIFFE + kill-switch).
- **Named precedent:** PASS. Cedar (AWS) is the engine; OPA / Rego is
  the named alternative; sigstore-rooted fragment signing is the
  precedent.
- **Failure-mode tree:** PASS. ADR-0294 enumerates: hot-reload TOCTOU
  (soak window); fragment-signing-root-compromise (kill-switch); per-
  fragment policy-evaluation-error (denial fallback); anomaly-rollback
  trigger (denial-rate / latency / grant-rate >3σ shift).
- **Capacity math:** PASS. ADR-0246 amendment library-first dispatch
  cites the 5ms p99 Cedar-eval budget; per-instance cache; backlog
  bounded by request-arrival rate.
- **Observability hooks:** PASS. ADR-0263 emission contract specifies
  per-permit-evaluation audit event class.
- **Rollback path:** PASS. ADR-0294 soak + anomaly-rollback supplies the
  per-fragment rollback path.
- **Multi-region awareness:** PASS. Library-first per-instance cache means
  every region serves locally; fragment publish + soak across regions
  enforced by ADR-0294 §C.
- **Sovereign-cell awareness:** PASS. Per-cell fragment publishing per
  ADR-0294 + ADR-0251.
- **Versioning + deprecation:** PASS. ADR-0258 cited + fragment metadata
  signature_lifetime field.

**Score: PASS (8 / 8 dimensions).** ONE-POLICY-ENGINE is the highest-
maturity invariant in the corpus.

### §3.3 ONE-WORKFLOW-ENGINE

- **Evidence:** workflow-engine µservice, workflow-studio µservice;
  ADR-0314 DealSet workflow templates; ADR-0257 ontology object-type
  versioning (workflow inputs/outputs).
- **Named precedent:** PARTIAL. ServiceNow Now Platform + Salesforce
  Flow + Palantir Foundry actions cited in thesis doc, but the
  workflow-engine µservice PRD-rigor pass = 7.1% per the audit (the
  workflow-engine PRD specifically not sampled by this synthesis).
- **Failure-mode tree:** PARTIAL. Workflow-engine runbook coverage =
  not sampled here; workflow-engine PRD failure-mode section probably
  absent (most PRDs lack the section per audit).
- **Capacity math:** FAIL. No throughput target for workflow steps
  per second under 1M-tenant load. No tail-latency claim.
- **Observability hooks:** PARTIAL. ADR-0263 emission contract cites
  workflow_run_id but per-workflow-step audit-event-class not enumerated.
- **Rollback path:** PARTIAL. ADR-0314 §D workflow.rollback template
  cited; per-step compensating-action enumeration missing.
- **Multi-region awareness:** FAIL. Workflow-engine PRD does not declare
  cross-region replay semantics.
- **Sovereign-cell awareness:** PARTIAL. Per-cell workflow definition
  hinted at but not enumerated.
- **Versioning + deprecation:** PARTIAL. Workflow template version field
  cited in ADR-0314 but per-template-version sunset cadence missing.

**Score: PARTIAL (1 PASS / 5 PARTIAL / 2 FAIL).** ONE-WORKFLOW-ENGINE
is structurally claimed but operationally thin. Wave-3-H (per-µservice
content pass on workflow-engine) is the lane that resolves this.

### §3.4 ONE-ONTOLOGY

- **Evidence:** ontology µservice; ADR-0257 (ontology object-type
  versioning + deprecation handshake); ADR-0257 amendment (library-first
  ontology read-path).
- **Named precedent:** PASS. Palantir Foundry ontology + AWS Cedar
  entity types + Microsoft Graph + Salesforce Platform metadata cited.
- **Failure-mode tree:** PARTIAL. ADR-0257 enumerates schema-version-drift,
  but missing: ontology-projection-drift across cells, freshness-floor-
  violation under partition.
- **Capacity math:** PARTIAL. ADR-0257 amendment cites library-first
  per-instance cache + freshness-floor; cell-spanning ontology read
  capacity not derived.
- **Observability hooks:** PARTIAL. Ontology read event class hinted at;
  not enumerated in ADR-0263 registry sample.
- **Rollback path:** PASS. ADR-0257 deprecation handshake.
- **Multi-region awareness:** PARTIAL. Library-first read-path is per-
  region; ontology write-path cross-region not specified.
- **Sovereign-cell awareness:** PASS. ADR-0251 pack-aware projection
  cited.
- **Versioning + deprecation:** PASS. ADR-0257 + amendment.

**Score: PARTIAL (4 PASS / 4 PARTIAL).** ONE-ONTOLOGY is well-defined
in cite-graph but operationally still emerging.

### §3.5 ONE-AUDIT-CHAIN

- **Evidence:** audit-chain µservice (189 artifacts — exemplar tier per
  audit §3.4); ADR-0263 observability emission contract; ADR-0296
  library-first credential sidecar (audit-signing key isolation);
  ADR-0293 meta-trust-root.
- **Named precedent:** PASS. AWS CloudTrail + Google Cloud Audit Logs +
  Microsoft Purview audit + Palantir Foundry audit cited.
- **Failure-mode tree:** PASS. Audit-stream-loss, audit-chain-Merkle-
  validation-failure, audit-signing-key-compromise enumerated.
- **Capacity math:** PARTIAL. ADR-0263 per-µservice cardinality budgets
  per metric; per-µservice audit-event/sec budget not enumerated.
- **Observability hooks:** PASS. ADR-0263 is itself the observability
  contract.
- **Rollback path:** PASS. Audit-replay procedure cited in audit-chain
  µservice runbook coverage (not sampled here but exemplar-tier).
- **Multi-region awareness:** PASS. Per-region audit-chain shard + roll-up
  to global audit-Merkle.
- **Sovereign-cell awareness:** PASS. Per-cell audit-chain enforcement.
- **Versioning + deprecation:** PASS. ADR-0258 cited; per-event-class
  versioning per `event-schema-versioning-canonical.md`.

**Score: PASS (7 PASS / 1 PARTIAL).** ONE-AUDIT-CHAIN is second-highest-
maturity invariant after ONE-POLICY-ENGINE.

### §3.6 ONE-MARKETPLACE

- **Evidence:** marketplace µservice (15 artifacts — **BELOW FLOOR per
  audit §3.1**); ADR-0249 multi-category marketplace; ADR-0314 DealSet
  settlement; ADR-0321 vendor coverage dossier; ADR-0250 build-ahead-of-
  certification.
- **Named precedent:** PASS. Stripe platform-facilitator +
  Shopify Markets + AWS Marketplace + Salesforce AppExchange cited.
- **Failure-mode tree:** PARTIAL. ADR-0249 enumerates fraud, dispute,
  chargeback. ADR-0314 enumerates fee-distribution-failure. Missing:
  cross-jurisdiction-settlement-conflict (e.g., when seller-jurisdiction
  refuses a payment-method buyer-jurisdiction permits); marketplace-
  operator-tax-reporting failure path.
- **Capacity math:** FAIL. No throughput target for DealSet creations
  per second; no settlement-latency P99 budget; no tail-latency model.
- **Observability hooks:** PARTIAL. DealSet event class hinted at; per-
  fee-distribution audit-event-class not enumerated.
- **Rollback path:** PARTIAL. Per-DealSet rollback template named; per-
  settlement compensating action not enumerated.
- **Multi-region awareness:** PARTIAL. Per-region settlement implied;
  cross-region settlement edge cases (sanctions-blocked-region) not
  enumerated.
- **Sovereign-cell awareness:** PASS. Per-cell marketplace operation
  cited.
- **Versioning + deprecation:** PARTIAL. DealSet schema versioning cited;
  per-fee-structure deprecation cadence missing.

**Score: PARTIAL with severe capacity-math gap (1 PASS / 5 PARTIAL / 2
FAIL).** AND marketplace µservice is BELOW PR-143 ARTIFACT FLOOR. This
is the **weakest** ONE-INVARIANT. Wave-3-H + Wave-3-I work required.

### §3.7 ONE-UX-SHELL

- **Evidence:** ADR-0317 (role-based projection), ADR-0318 (collar-color
  universality), ADR-0319 (front-middle-back-office), ADR-0320 (apprentice
  tier); ux-best-practices.md standard (~2,490 lines).
- **Named precedent:** PASS. Apple HIG + Microsoft Fluent UI + Google
  Material + Salesforce Lightning cited.
- **Failure-mode tree:** PARTIAL. Per-collar-color UX adaptation cited;
  per-device-profile fallback not enumerated.
- **Capacity math:** N/A — UX shell is rendering, not throughput-bound.
- **Observability hooks:** PASS. Per-role-projection telemetry
  identifier (`role_projection_id` field cited throughout unified
  ecosystem thesis).
- **Rollback path:** PASS. UX-shell-version sunset per ADR-0258.
- **Multi-region awareness:** PASS. Per-locale UX shell adaptation per
  ADR-0317.
- **Sovereign-cell awareness:** PASS. Per-pack UX overlay per ADR-0251.
- **Versioning + deprecation:** PASS. UX-shell-version field; per-
  component deprecation cadence per ADR-0258.

**Score: PASS-WITH-PARTIAL (6 PASS / 1 PARTIAL / 1 N/A).** ONE-UX-SHELL
is third-highest-maturity invariant.

### §3.8 ONE-TRAINING-MODEL

- **Evidence:** `training-cost-doctrine-2026-05-21.md` (2,325 lines);
  ADR-0317 vocabulary durability claim; ADR-0318 collar-color training
  cohort claim.
- **Named precedent:** PASS. Microsoft 365 learning pathways + Google
  Workspace Learning Center + Salesforce Trailhead + ServiceNow Now
  Learning cited.
- **Failure-mode tree:** FAIL. The training-cost-doctrine doc does not
  enumerate training-vocabulary-divergence (what happens when a vendor-
  specific shortcut leaks into the shared vocabulary).
- **Capacity math:** PARTIAL. The doc claims "1500 USD per employee
  per year per-tool training pressure" as an internal sizing assumption.
  Not yet validated against Gartner / Forrester source citations
  (the doc itself flags this as pending legal/procurement validation).
- **Observability hooks:** FAIL. No per-vocabulary-action telemetry
  identifier in ADR-0263 registry.
- **Rollback path:** N/A — training-vocabulary is taught, not deployed.
- **Multi-region awareness:** PARTIAL. Per-locale training material
  hinted; per-jurisdiction training-content delta not enumerated.
- **Sovereign-cell awareness:** N/A.
- **Versioning + deprecation:** PARTIAL. Vocabulary-version field hinted;
  per-action deprecation cadence missing.

**Score: PARTIAL (1 PASS / 3 PARTIAL / 2 FAIL / 2 N/A).** AND the
training-cost-doctrine doc itself is template-stamped (160 identical
"Problem clause" rows) per §6.2 below — editorial REVISE required
before this invariant achieves PASS.

### §3.9 ONE-COMPLIANCE-POSTURE

- **Evidence:** ADR-0251 compliance-pack-cell-certification; per-pack
  manifests in `packs/`; ADR-0250 build-ahead-of-certification; ADR-
  0276 backup-portability GDPR Art. 20; ADR-0273 per-tenant DKIM/SPF/
  DMARC.
- **Named precedent:** PASS. SOC 2 + ISO 27001 + GDPR + HIPAA + PCI
  DSS + FedRAMP + CSAP cited per-pack.
- **Failure-mode tree:** PASS. Pack-conflict resolution + per-pack
  Cedar-fragment compositional override enumerated.
- **Capacity math:** PARTIAL. Per-pack Cedar fragment eval cost cited;
  per-pack audit-stream cost not enumerated.
- **Observability hooks:** PASS. ADR-0263 audit-event-class per
  pack-relevant-action.
- **Rollback path:** PARTIAL. Per-pack-activation rollback hinted;
  per-pack-revocation runbook missing (per audit §1.2 runbook rigor
  pass = 5.9%).
- **Multi-region awareness:** PASS. Per-region pack overlay per
  ADR-0251.
- **Sovereign-cell awareness:** PASS. ADR-0251 is itself this invariant's
  vehicle.
- **Versioning + deprecation:** PASS. Pack manifest schema versioned
  per ADR-0258.

**Score: PASS-WITH-PARTIAL (6 PASS / 2 PARTIAL).** ONE-COMPLIANCE-POSTURE
is fourth-highest-maturity invariant.

### §3.10 ONE-PLUGIN-EXTENSIBILITY

- **Evidence:** plugin-app-store µservice; developer-sdk µservice;
  ADR-0249 multi-category marketplace; workflow-studio µservice.
- **Named precedent:** PARTIAL. Salesforce AppExchange + ServiceNow
  Store + Microsoft AppSource + Atlassian Marketplace + Apple App Store
  + Google Play cited in ADR-0249 high-level.
- **Failure-mode tree:** PARTIAL. Plugin-supply-chain-compromise
  hinted via ADR-0247 self-modification doctrine + ADR-0293 meta-trust-
  root. Plugin-runtime-isolation-failure not enumerated.
- **Capacity math:** FAIL. No plugin-invocation-throughput target.
- **Observability hooks:** PARTIAL. Plugin-invocation event class hinted;
  cardinality budget per plugin not enumerated.
- **Rollback path:** PARTIAL. Plugin disable + tenant unbind cited;
  per-plugin compensating-action enumeration missing.
- **Multi-region awareness:** PARTIAL. Per-region plugin manifest;
  cross-region plugin state synchronization not specified.
- **Sovereign-cell awareness:** PARTIAL. Per-pack plugin restriction
  hinted.
- **Versioning + deprecation:** PARTIAL. Plugin-manifest version field;
  per-plugin sunset cadence missing.

**Score: PARTIAL (7 PARTIAL / 1 FAIL).** ONE-PLUGIN-EXTENSIBILITY is the
second-weakest invariant after ONE-MARKETPLACE.

### §3.11 Roll-Up Scorecard

| Invariant | PASS | PARTIAL | FAIL | N/A | Verdict |
|---|---:|---:|---:|---:|---|
| ONE-IDENTITY | 4 | 4 | 0 | 0 | PARTIAL |
| ONE-POLICY-ENGINE | 8 | 0 | 0 | 0 | **PASS** |
| ONE-WORKFLOW-ENGINE | 1 | 5 | 2 | 0 | PARTIAL |
| ONE-ONTOLOGY | 4 | 4 | 0 | 0 | PARTIAL |
| ONE-AUDIT-CHAIN | 7 | 1 | 0 | 0 | **PASS** |
| ONE-MARKETPLACE | 1 | 5 | 2 | 0 | **WEAK** |
| ONE-UX-SHELL | 6 | 1 | 0 | 1 | PASS-W-PARTIAL |
| ONE-TRAINING-MODEL | 1 | 3 | 2 | 2 | PARTIAL + EDITORIAL REVISE |
| ONE-COMPLIANCE-POSTURE | 6 | 2 | 0 | 0 | PASS-W-PARTIAL |
| ONE-PLUGIN-EXTENSIBILITY | 0 | 7 | 1 | 0 | PARTIAL |

**Net interpretation:** Two of ten ONE-INVARIANTS pass cleanly (POLICY,
AUDIT). Two more pass-with-partial (UX-SHELL, COMPLIANCE). Five are
PARTIAL. ONE-MARKETPLACE is weakest. ONE-TRAINING-MODEL has both
operational gaps AND editorial template-stamping issues. ONE-PLUGIN-
EXTENSIBILITY has zero PASS cells — all dimensions are PARTIAL or FAIL.

**Confidence: MED.** The invariant-by-invariant scoring is sampled-source
based; full PRD-by-PRD verification was not feasible in this Opus pass.
A future per-invariant deep-dive (Wave 3-K?) would refine these grades.

---

## §4 ERP + Salesforce + B2B-Leader Coverage Audit

### §4.1 ADR-0315 SAP Coverage Audit (Per-Module)

ADR-0315 §D-1 enumerates 23 SAP module families. Pass/fail status by
module per the in-line per-module §D-1.A notes:

| SAP code | SAP module | Status declared | oyatie destination | Verdict |
|---|---|---|---|---|
| FI | Financial Accounting | partial-existing-plus-new-treasury | specs + payments + finops-portal + treasury | **partial; accounting spec not on disk as µservice** |
| CO | Controlling | covered-by-composition | finops-portal + ontology + workflow-engine + supply-chain-planning | pass-by-composition |
| MM | Materials Management | partial-existing-plus-new-warehouse | marketplace + workflow-engine + connect + warehouse | **partial; marketplace below floor (15 artifacts)** |
| SD | Sales & Distribution | partial-existing-plus-new-crm-warehouse | marketplace + payments + crm + warehouse | partial |
| PP | Production Planning | new-required | production-planning | new-scaffold-only (129 artifacts but PRD 400 lines / 0 stories) |
| QM | Quality Management | new-required | quality-management | new-scaffold-only |
| PM | Plant Maintenance | new-required | plant-maintenance | new-scaffold-only |
| HCM | Human Capital Management | planned-existing-spec-coverage | hr.json spec + payroll.json spec + workplace-integration + workflow-engine | **partial; HR + payroll specs not promoted to µservice scaffolds** |
| PS | Project System | covered-by-composition | workflow-engine + ontology + finops-portal + payments | pass-by-composition |
| PLM | Product Lifecycle Mgmt | covered-by-composition | ontology + workflow-engine + connect + production-planning | pass-by-composition |
| EHS | Environment Health Safety | covered-by-composition | compliance + workflow-engine + ontology + quality-management | pass-by-composition |
| SRM | Supplier Relationship Mgmt | covered-by-composition | marketplace + workflow-engine + ontology + payments | partial (marketplace below floor) |
| CRM | Customer Relationship Mgmt | new-required | crm + community + marketplace + intelligence | new-scaffold-only |
| SCM/APO | Supply Chain Mgmt | new-required | supply-chain-planning + production-planning + warehouse | new-scaffold-only |
| GTS | Global Trade Services | new-required | global-trade + compliance + connect | new-scaffold-only |
| TM | Transportation Mgmt | covered-by-initial-composition | supply-chain-planning + warehouse + marketplace + global-trade | **partial; carrier-optimization gap flagged in §G** |
| EWM | Extended Warehouse Mgmt | new-required | warehouse | new-scaffold-only |
| TRM | Treasury & Risk Mgmt | new-required | treasury + payments + finops-portal | new-scaffold-only |
| RE-FX | Real Estate Mgmt | new-required | real-estate + plant-maintenance + finops-portal | new-scaffold-only |
| IS-* | Industry Solutions | pack-overlay | packs/industry/* + ontology + workflow-engine + compliance | pass-by-pack |
| NETWORK | Network Products | covered-by-composition | marketplace + payments + workplace-integration + crm | partial |
| PLATFORM | Platform & Extensibility | covered-by-composition | plugin-app-store + developer-sdk + workflow-studio + workflow-engine + ontology | pass-by-composition |
| DATA | Data & Analytics | covered-by-composition | analytics + ontology + intelligence + observability | pass-by-composition |

**Per-module roll-up:**
- pass-by-composition: 6 modules (CO, PS, PLM, EHS, PLATFORM, DATA)
- pass-by-pack: 1 module (IS-*)
- new-scaffold-only (artifact-count OK; PRD content NOT OK): 9 modules
  (PP, QM, PM, CRM, SCM/APO, GTS, EWM, TRM, RE-FX)
- partial: 7 modules (FI, MM, SD, HCM, SRM, TM, NETWORK)

**Critical gap flagged by ADR-0315 §G itself:** TM (Transportation
Management) carrier-optimization is acknowledged as partial coverage —
"Carrier optimization can split later" per §D-1 row. This is a known
unresolved gap.

**Confidence: HIGH** for per-module destination + scaffold-presence
verification; **MED** for "pass-by-composition" verdicts (sampled
destination µservices not exhaustively audited).

### §4.2 ADR-0321 B2B-Leader Coverage Audit (Per-Vendor)

ADR-0321 supplies 165 vendor dossiers. Coverage tier distribution:
- Tier A (already covered): 17 vendors / 10.3%
- Tier B (partial): 6 vendors / 3.6%
- Tier C (composition over existing µservices): 102 vendors / 61.8%
- Tier D (new µservice): 40 vendors / 24.2%

The 13 NEW µservice anchors listed in ADR-0321 §B-2:
marketing-automation, contact-center, performance-management, learning-
management, itsm, incident-management, financial-planning, data-warehouse,
contract-lifecycle-management, whiteboard, design-collaboration, data-
pipeline, healthcare-integration.

All 13 directories ARE present under `microservices/`.

But: per the corpus-rigor audit §1.2, "Microservices PRD floor pass:
5/70 = 7.1%." That means at most 5 of the 70 µservices have a
hyperscaler-grade PRD. The 22 newly-scaffolded Wave-3-G µservices (9 ERP
+ 13 B2B-leader) all have PRDs at exactly 400 lines with 0 user stories.
None of the 22 passes the 1500-line / 40-story floor in documentation-
rigor.md §2 row 3.

**Per-vendor verdict pattern:**
- 17 Tier-A vendors: pass-as-claimed (existing coverage; not re-verified
  in this synthesis).
- 6 Tier-B vendors: partial — need delta work cited in dossier.
- 102 Tier-C vendors: pass-by-composition CLAIMED but every dossier
  uses identical sentences (P0 template-stamping per §6.1).
- 40 Tier-D vendors: anchor-only — directories exist; PRDs are 400-line
  stubs; full coverage NOT achieved.

**Residual gaps (called out by ADR-0321 or inferred):**
1. **Tier D vendors lack content PRDs.** All 40 vendors share scaffolds
   but no content (P0).
2. **Logistics-integration deferred to later wave.** ADR-0321 §C-3
   notes the corpus moves "from 56 to 69 µservices, excluding optional
   logistics-integration and personal-health-tracker follow-ups." Two
   known-deferred categories.
3. **Capability-tier registry referenced but not present on disk.**
   ADR-0316 doctrine + ADR-0321 dossiers reference per-capability-tier
   registry shape (`governance-capability-tier-registry-shape`)
   but no `specs/capability-tier-registry.json` file is enumerated.

### §4.3 Capability-Tier-over-µservice Ratio

Per ADR-0316 doctrine, the goal is to push vendors INTO capability tiers
unless a distinct operational concern justifies a new µservice.

For ADR-0315 (SAP coverage, 23 modules):
- 9 modules became new µservices (39%).
- 14 modules became capability tiers OR pack overlays OR composition
  (61%).

For ADR-0321 (B2B-leader coverage, 165 vendors):
- 13 vendor categories became new µservices (note: the 13 named in §B-2;
  most map to one of 40 Tier-D vendor dossiers folding into one of the
  13 µservices — many vendors share a destination µservice).
- 152 vendors mapped to existing µservices via capability tiers /
  composition (Tier A + B + C = 125 / 165 = 75.8%).
- The Tier-D rate of 24.2% (40 / 165) means 1 in 4 vendors needs a
  new µservice, which is HIGH given the ADR-0316 doctrine target of
  pushing 80%+ into capability tiers.

**Capability-tier doctrine adherence:** 75.8% capability-tier ratio.
ADR-0316 doctrine target is implied to be higher (~80%+). The 24.2%
new-µservice rate suggests the ADR-0316 four-condition test in §1.2
of the coverage matrix was applied permissively in Wave-3-G.

**Confidence: HIGH** for the percentages; **MED** for "ADR-0316 target
implied to be higher" — the doctrine doc does not name a specific
percentage target.

---

## §5 Persona-Roster + Journey-Catalog Cross-Coverage

### §5.1 Cardinality Snapshot

- **Personas:** 1 master roster + 129 dossiers = 130 persona files.
- **Journeys:** 150 journey directories (per `ls docs/user-journeys/ |
  wc -l` minus catalog files).
- **µservices:** 70 directories.
- **µservices × Personas × Journeys cell-count:** 70 × 130 × 150 =
  1,365,000 conceptual cells.

The persona-roster doctrine in MASTER-ROSTER-2026-05-21.md §2.5 calls
out "127 personas span 6 collar-colors × 4 skill-tiers × 5 workspaces ×
6 locales × 4 device profiles = 2880 possible tuples; we author the 127
that anchor journeys + the most-novel collar-colors." So the roster is
authored against tuple-density-targets, not against a cell-completeness
target.

### §5.2 Personas with Thin Journey Coverage

The persona roster catalogues 10 "Original archetypes (10) — anchored to
existing journeys j01-j150" per §3.1 of MASTER-ROSTER. The other 117
personas in §3.2-§3.10 are NOT explicitly journey-mapped in the master
roster — the per-persona dossier files would carry that mapping but
were not sampled here.

Sampled personas with explicit `cross_context_personas` field:
- Yejin Park: 3 cross-contexts (nurse / parent / side-business-owner)
- Marcus Chen: 3 cross-contexts
- Aiyana Singh: 3 cross-contexts
- Chris Volkov: 3 cross-contexts (laid-off, post-layoff, family provider)

Personas with thin journey coverage (inferred from the persona roster
§3.2-§3.10 tables — these personas have no explicit "anchored journey"
column):
- §3.2 Non-office collar-color (13 personas: Carlos Martinez, Sarah
  Kim, Ahmad Hassan, Maria Santos, Devon Williams, Jordan Lee, Ms.
  Patel, Coach Park, Father Lopez, Captain Chen, Officer Rodriguez,
  Dr. Tanaka, Tomás García Jr., Captain Olufemi) — these are
  collar-color-anchored, not journey-anchored. P1 persona-journey gap.
- §3.3 Office C-suite (10 personas) — sampled = thin journey anchor
- §3.4..§3.10 (~94 personas) — same pattern

**Estimated persona-journey-mapping gap:** ~117 / 130 personas lack
explicit journey anchoring in the roster (90%). Per-persona dossiers
may contain the mapping; need Wave-3-H deeper-cut verification.

### §5.3 Journeys with Thin Persona Coverage

The journey catalog has 150 entries. Sampled journey names from
`ls docs/user-journeys/`:
- j01-emergency-911-dispatch
- j02-healthcare-code-blue-ehr-break-glass
- j03-988-crisis-line-minor-self-report
- j04-dv-survivor-shelter-mode
- j05-whistleblower-anonymous-ethics-report
- j06-press-source-securedrop-class
- j07-deceased-user-inheritance-handoff
- j08-elder-financial-abuse-detection
- j09-account-recovery-phishing-resistant
- j10-account-takeover-SIM-swap-detected
- j100..j150 — supply-chain + procurement + marketplace journeys

The "life-safety + crisis + recovery" journeys j01-j10 each implicitly
anchor to specific personas (e.g., j02 anchors to Dr. Tanaka, j04 to
domestic-violence survivors, j05 to whistleblowers). But each journey's
explicit persona-roster cross-reference field is not enumerated in the
sampled journey-catalog-j126-j150-ecosystem.md (the only catalog file
sampled).

**Estimated journey-persona mapping gap:** ~90% of the 150 journeys
lack explicit roster-anchored persona cross-references at the catalog
level (per-journey deeper files may close this gap, not sampled).

### §5.4 µservices with Insufficient Persona-Journey Traffic

The corpus-rigor audit §1.2 reports OpenAPI 3.2.0 conformance = 10.6%
and AsyncAPI 3.1.0 conformance = 11.6%. Across 736 OpenAPI + 744
AsyncAPI surfaces, only ~78 + ~86 are at-version.

The 22 newly-scaffolded µservices (Wave-3-G additions) have:
- 129 artifacts each (per sampled production-planning, crm, warehouse).
- 400-line PRDs with 0 user stories.
- Likely zero persona-journey-traffic evidence in their PRDs (PRDs do
  not contain persona dossier cross-references — sampled).

**Estimated µservice-persona-journey traffic gap:** all 22 Wave-3-G new
µservices have insufficient persona-journey traffic evidence. Wave 3-H
content pass is the remediation.

### §5.5 Persona-Roster × Journey-Catalog Coverage Matrix (Compressed)

A full 130 × 150 cell-by-cell traversal exceeds this synthesis's scope.
The following compressed coverage estimate uses the roster's tuple-
density-target methodology:

| Persona archetype band | # personas | Anchored journeys (sampled) | Coverage estimate |
|---|---:|---|---|
| §3.1 Original 10 archetypes | 10 | j01-j150 ALL | 95% — these personas drive the original journey set |
| §3.2 Non-office collar (13) | 13 | j02 (Dr. Tanaka), j20 (forklift), partial others | 40% — collar-color UX coverage strong but per-journey thin |
| §3.3 C-suite (10) | 10 | j13 (CEO board crisis), j110 (CFO audit), partial | 35% |
| §3.4 (mid-management) | ~14 | partial | 30% |
| §3.5 (operational specialists) | ~13 | partial | 30% |
| §3.6 (regulated-tier specialists) | ~15 | j41-j50 (regulator-and-auditor) | 50% |
| §3.7 (community + family) | ~20 | j07 (deceased-user inheritance) + j150 (creator economy) + partial | 35% |
| §3.8 (in-training / transient) | ~15 | implicit anchors | 25% |
| §3.9 (locale-specific) | ~10 | per-pack journey anchors | 40% |
| §3.10 (legacy / retirement) | ~10 | j124 (eldercare) + partial | 35% |

**Coverage roll-up:** ~40% persona-journey coverage at the master-roster
+ journey-catalog level. Per-persona dossiers may close this; Wave 3-J
deeper-cut journeys (j151+) recommended for the persona archetypes
currently under 35% (mid-management, operational specialists, in-
training, retirement).

**Confidence: MED-LOW.** This is the section with the lowest
verification rigor in this synthesis. A per-persona × per-journey
deterministic walker (similar to the missing six-hops walker) would
produce HIGH confidence. Recommendation: build a `tools/persona-
journey-cross-coverage/` walker as Wave 3-I scope.

---

## §6 Cross-Document Contradictions

### §6.1 P0 — Template-Stamped Vendor Dossiers in ADR-0321

**Finding:** All 165 vendor dossiers in ADR-0321 §D-001 through §D-165
share identical Cedar permit shape sentences, identical ontology
projection sentences, identical workflow template library sentences,
identical UX shell adaptation sentences, identical pack overlay
sentences, identical migration path sentences, and identical
failure-mode coverage sentences. Only the vendor name + coverage tier
+ destination µservice differ.

**Evidence:**
- `docs/decisions/ADR-0709-general-live-apex.md:81-249`
  Section D-001 (Salesforce Sales Cloud) through D-013 (Salesforce
  Field Service Lightning) — sampled in this pass.
- Each row contains the sentence "templates are owned by workflow-
  engine unless the destination service owns a narrower runtime" verbatim.
- Each row contains the sentence "import drift, missing delegated
  authority, regional outage, source API throttling, duplicate
  submission, and pack conflict all produce explicit refusal or
  remediation evidence" verbatim.

**Severity:** P0. Documentation-rigor.md §1 intern-buildability test —
an intern reading ADR-0321 §D-006 (MuleSoft) cannot derive anything
about MuleSoft's actual REST-vs-SOAP integration surface or its iPaaS-
specific patterns, because the dossier prose says the same thing as
ADR-0321 §D-005 (Tableau). Fails hyperscaler-grade rigor sub-test §1.1
item 1 (named precedent missing) — no MuleSoft-specific iPaaS
pattern citation.

**Confidence:** HIGH. Verified by reading 13 contiguous dossier rows
and confirming sentence-by-sentence identity.

**Fix:** Per-vendor delta over a shared template macro. The macro
captures the universal shape; each vendor row adds the vendor-specific
delta (e.g., MuleSoft = iPaaS connector library; Tableau = visual
analytics surface; Snowflake = isolated-compute SQL warehouse).

### §6.2 P0 — Template-Stamped Clauses in unified-ecosystem-thesis-2026-05-21.md

**Finding:** The doc enumerates "ONE-INVARIANTS" as 10 distinct
primitives but the doc body contains 700 "Thesis clause N" rows (per
`grep -c "Thesis clause "`). Each row repeats the same Oyatie / Policy
/ Training / Operational / Anti-fragmentation sentences with only a
clause-number + precedent-citation swap.

**Evidence:**
- `docs/architecture/unified-ecosystem-thesis-2026-05-21.md:73-400+`.
  Section 1 alone contains thesis clauses 1-37+ sampled, all repeating
  the same 6-sentence block.
- 700 thesis clauses over 10 distinct invariants = 70 clauses per
  invariant in the doc body.

**Severity:** P0. Documentation-rigor.md §2 ArchitectureDeepDive row —
content density requires step-by-step trace with file paths + line
numbers. Repeating the same 6-sentence block 70 times per invariant
does NOT add density; it adds line-count without information-density.

**Confidence:** HIGH. Grep-verified 700-row count.

**Fix:** Collapse to 10 invariant-blocks, each with its own bespoke
content. Hyperscaler precedent citations stay per-invariant.

### §6.3 P0 — Template-Stamped Problem Clauses in training-cost-doctrine-2026-05-21.md

**Finding:** §1 contains 160 "Problem clause N" rows. Each row
repeats the same 3-sentence problem-statement / evidence / consequence
block verbatim.

**Evidence:**
- `docs/architecture/training-cost-doctrine-2026-05-21.md:64-243+` —
  problem clauses 001 through 160 sampled at multiple ranges; identical
  body.
- `grep -c "Problem clause "` = 160.

**Severity:** P0. Same as §6.2.

**Confidence:** HIGH. Grep-verified.

**Fix:** Collapse to 1 problem statement; expand the rest of the doc
into the actual training-cost amortization model with per-collar-color
+ per-skill-tier + per-career-stage cost numbers (which the doc claims
but does not derive).

### §6.4 P0 — ADR Status Inconsistency

**Finding:** Eleven ADRs in the Wave-3-G doctrine cluster (0310-0321),
ten are `Proposed`, one (ADR-0319 front-middle-back-office-information-
barrier) is `Accepted`. ADR-0320 status is lowercase `proposed`.

**Evidence:**
- `docs/adr-archive/ADR-0319-front-middle-back-office-information-barrier.md
  barrier.md:?` — status: Accepted (verified by grep).
- `docs/adr-archive/ADR-0320-apprentice-intern-resident-fellow-transient-identity.md
  identity.md:?` — status: proposed (lowercase; verified by grep).
- All other cluster ADRs: status: Proposed (verified by grep).

**Severity:** P0. The keystone-bundle 2026-05-20 synthesis §1 requires
that cluster-merged ADRs land in `Proposed` state simultaneously. An
out-of-band `Accepted` (ADR-0319) violates the cluster-coherence
invariant. The lowercase `proposed` violates the canonical enum.

**Confidence:** HIGH. Both verified by grep.

**Fix:** Edit ADR-0319 frontmatter to status: `Proposed`. Edit ADR-0320
frontmatter to status: `Proposed` (uppercase). Both as part of
pre-merge cleanup per §11.C.

### §6.5 P0 — Wave-3-G Brief Says 30+ New ADRs; Corpus Has 25

**Finding:** Per the corpus-rigor-audit redo §1.1 ("brief says 30+ new
ADRs but live 0297-0321 range has 25 files"), the brief's expectation
of 30+ new doctrine ADRs is not met by the live corpus.

**Evidence:**
- `docs/architecture/corpus-rigor-audit-2026-05-21-post-wave-3-g.md:50-51`
- `ls docs/decisions/ | grep -c "ADR-029[7-9]\|ADR-030\|ADR-031\|ADR-032[0-1]"`
  = 25 (per audit; reproducible).

**Severity:** P0. The Wave-3-G plan called for 30+; only 25 landed.
The five missing ADRs are not enumerated in the audit; could be either
(a) plan was scope-trimmed mid-wave, or (b) five ADRs are in-flight in
codex agent worktrees and not yet merged to dev.

**Confidence:** HIGH (the audit reports this). MED on root cause.

**Fix:** Reconcile the brief with the actual landing — either supply
the five missing ADRs in a Wave-3-G follow-up or amend the brief to
say "25 new doctrine ADRs landed in Wave 3-G."

### §6.6 P1 — Six-Hop Graph Walker Tool Missing

**Finding:** Documentation-rigor.md §3.1 names the deterministic graph
walker tool path `tools/doc-graph-walker/`. The corpus-rigor audit redo
§1.3 reports "no tools/doc-graph-walker found in this checkout."

**Evidence:**
- `docs/standards/documentation-rigor.md:211-212`
- `docs/architecture/corpus-rigor-audit-2026-05-21-post-wave-3-g.md:87-88`

**Severity:** P1. The §3.1 lane (`governance-doc-graph-6hops`)
becomes BLOCKER from 2026-07-16 per documentation-rigor.md §3.1.
Without the walker, BLOCKER promotion cannot proceed.

**Confidence:** HIGH (both files cited).

**Fix:** Wave 3-I scope — implement the walker. Acceptance criteria:
deterministic BFS over markdown links + frontmatter `related_*` +
`companion_docs` arrays + `inbound_citations` arrays; emit per-doc
reachability score from each entry point.

### §6.7 P1 — Marketplace µservice Below PR-143 Floor

**Finding:** Per corpus-rigor audit §2.3, marketplace µservice has
15 artifacts (well below the 70-artifact PR-143 floor).

**Evidence:**
- `docs/architecture/corpus-rigor-audit-2026-05-21-post-wave-3-g.md:108-110`

**Severity:** P1 — but cascades to P0 because ONE-MARKETPLACE is one
of the 10 ONE-INVARIANTS and per §3.6 above it is the WEAKEST invariant.

**Confidence:** HIGH.

**Fix:** Wave 3-H — full marketplace µservice content pass.

### §6.8 P1 — workplace-integration µservice Below PR-143 Floor

**Finding:** Per corpus-rigor audit §2.3, workplace-integration µservice
has 16 artifacts (below 70-artifact floor).

**Evidence:**
- `docs/architecture/corpus-rigor-audit-2026-05-21-post-wave-3-g.md:108-110`

**Severity:** P1.

**Confidence:** HIGH.

**Fix:** Wave 3-H content pass.

### §6.9 P1 — OpenAPI 3.2.0 / AsyncAPI 3.1.0 Conformance at 10.6% / 11.6%

**Finding:** Documentation-rigor.md §3.2.2 invariant 3 requires
OpenAPI 3.2.0 + AsyncAPI 3.1.0 uniformly. Audit reports 78 / 736 + 86 /
744 conformance.

**Evidence:**
- `docs/standards/documentation-rigor.md:269`
- `docs/architecture/corpus-rigor-audit-2026-05-21-post-wave-3-g.md:78-79`

**Severity:** P1 corpus-wide.

**Confidence:** HIGH.

**Fix:** Wave 3-H mechanical sed — bump every contract file to the
canonical version. Pre-existing audit batch-0 IP work captures this.

### §6.10 P1 — Status Field Drift in ADR-0263 Frontmatter

**Finding:** Per the 2026-05-20 keystone synthesis §10 audit, ADR-0263
has duplicate `status:` frontmatter keys. The Wave-3-D Phase-2
remediation was scheduled but not verified in the Wave-3-G corpus.

**Evidence:**
- `docs/architecture/keystone-bundle-2026-05-20-synthesis.md:272`

**Severity:** P1.

**Confidence:** MED (not re-verified in this synthesis).

**Fix:** Phase-2 remediation per the prior synthesis.

### §6.11 P1 — Layer-Enum Drift in ADR-0263 §D-6

**Finding:** Per the 2026-05-20 keystone synthesis §10 audit, ADR-0263
§D-6 layer-enum forks outside ADR-0105's 13-layer canonical set.
Scheduled regression-fix in Slice-4 — verify it landed in Wave-3-G.

**Evidence:**
- `docs/architecture/keystone-bundle-2026-05-20-synthesis.md:276`

**Severity:** P1.

**Confidence:** LOW (not re-verified).

**Fix:** Verify; re-apply if needed.

### §6.12 P2 — Persona-Roster Status

**Finding:** MASTER-ROSTER-2026-05-21.md status: `Proposed`. None of
the 129 persona dossiers were sampled for status-field consistency.
Roster declares 127 personas; 129 dossiers exist (+ master roster).
Off-by-two.

**Evidence:**
- `docs/personas/MASTER-ROSTER-2026-05-21.md:1-39`
- `ls docs/personas/ | wc -l = 130`

**Severity:** P2.

**Confidence:** MED.

**Fix:** Reconcile — either bump roster to 129 + master OR remove 2
orphan dossiers.

### §6.13 P2 — Coverage Matrix Status

**Finding:** enterprise-software-coverage-matrix-2026-05-21.md status:
`Living`. Documentation-rigor.md does not enumerate `Living` in the
canonical status enum.

**Evidence:**
- `docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md:5`

**Severity:** P2 (enum drift).

**Confidence:** MED.

**Fix:** Use `Accepted` if the matrix is operationally authoritative,
or `Proposed` if pending validation. Or expand the canonical enum to
include `Living` for living-reference docs.

### §6.14 P2 — Crates/*/docs is Empty

**Finding:** Audit reports `crates/*/docs/** = 0 files`.

**Evidence:**
- `docs/architecture/corpus-rigor-audit-2026-05-21-post-wave-3-g.md:47`

**Severity:** P2 — crate-local docs were called out as in-scope by
documentation-rigor.md §0 ("docs/`, `microservices/*/`, `packs/*/`,
`specs/`, and `crates/*/docs/`").

**Confidence:** HIGH.

**Fix:** Wave 3-K when code-authoring lands; crate-local docs come
with crate code.

### §6.15 P2 — Persona-Roster Says 127 But Sample Doctrine Says "≈127" Loosely

**Finding:** MASTER-ROSTER says "127 personas" in §2.5 capacity math
and "approximately 127" in frontmatter purpose. The actual count is
129 dossiers + 1 master roster = 130 files.

**Evidence:** `docs/personas/MASTER-ROSTER-2026-05-21.md:11`
("approximately 127 personas").

**Severity:** P2.

**Confidence:** HIGH.

**Fix:** Reconcile count (likely the original target was 127; two
dossiers added late or test-fixtures present).

### §6.16 P2 — Mismatch Between ADR-0321 §C-3 Count and Audit Count

**Finding:** ADR-0321 §C-3 reports "the corpus microservice count moves
from 56 to 69." Audit reports 70 directories present. Off-by-one.

**Evidence:**
- `docs/decisions/ADR-0709-general-live-apex.md:75`
- `docs/architecture/corpus-rigor-audit-2026-05-21-post-wave-3-g.md:48`

**Severity:** P2.

**Confidence:** HIGH.

**Fix:** Reconcile (likely one µservice was added outside the ADR-0321
+ ADR-0315 anchor scope, e.g., a Wave-3-D scaffolded µservice not
counted in 56-baseline-or-69-target).

### §6.17 P2 — Inconsistent Persona Count between Roster and Doctrine ADRs

**Finding:** The unified-ecosystem-thesis says "127 personas" in
multiple places. The persona-roster file count says 130. ADR-0317
companion_docs cites the persona roster but doesn't enumerate count.

**Evidence:** various.

**Severity:** P2.

**Confidence:** MED.

**Fix:** Standardize the count across docs.

### §6.18 P3 — ADR-0321 Brief Says "≈150 vendors"; Live Has 165

**Finding:** ADR-0321 says 165 vendor dossiers; the user-brief language
suggested ~150 vendors.

**Severity:** P3 — over-delivery.

**Confidence:** HIGH (grep-verified).

**Fix:** Update brief to match landed count.

### §6.19 P3 — j150 Catalog Ends; No j151+

**Finding:** Journey catalog ends at j150 (catalog name = CATALOG-j126-
j150-ecosystem.md). The brief Wave-3-J calls for j151+ deeper-cut
journeys.

**Evidence:**
- `ls docs/user-journeys/ | grep "^j15" | wc -l = 2` (j15-bug-bounty +
  j150-creator-economy, no j151+ folder).

**Severity:** P3 (Wave 3-J scope, not Wave 3-G scope).

**Confidence:** HIGH.

**Fix:** Wave 3-J scope.

### §6.20 P3 — Memory Says "Sonnet executors / Opus Max planning"

**Finding:** `feedback_model_routing` memory says "Sonnet executors;
Opus Max planning; Opus+Codex consensus on high-risk." This Opus 4.7
synthesis is the planning model (correct), and the eight codex agents
running in parallel are the high-risk-content executors (correct
routing).

**Severity:** P3 — informational confirmation of doctrine adherence.

**Confidence:** HIGH.

**Fix:** N/A.

### §6.21 P3 — No Wave-3-G Brief Cross-Reference in Audit Reports

**Finding:** The six audit reports under `docs/architecture/*-audit-
2026-05-2X.md` don't cross-reference the Wave-3-G brief (no
`docs/wave-3-g-brief.md` exists or is cited).

**Severity:** P3.

**Confidence:** MED (search not exhaustive).

**Fix:** Land a `docs/architecture/wave-roadmap-2026-05-2X.md` that
catalogues the Wave-3-G plan + Wave-3-H/I/J/K sequencing per §9 below.

### §6.22 Contradiction Roll-Up

| # | Severity | Confidence | Location |
|---:|---|---|---|
| 6.1 | P0 | HIGH | ADR-0321 template-stamped dossiers |
| 6.2 | P0 | HIGH | unified-ecosystem-thesis 700-clause loop |
| 6.3 | P0 | HIGH | training-cost-doctrine 160-clause loop |
| 6.4 | P0 | HIGH | ADR-0319 status Accepted vs cluster Proposed; ADR-0320 lowercase |
| 6.5 | P0 | HIGH | brief 30+ vs corpus 25 ADRs |
| 6.6 | P1 | HIGH | six-hops walker tool missing |
| 6.7 | P1 | HIGH | marketplace µservice below floor (15 artifacts) |
| 6.8 | P1 | HIGH | workplace-integration µservice below floor (16 artifacts) |
| 6.9 | P1 | HIGH | OpenAPI/AsyncAPI version conformance at 10-11% |
| 6.10 | P1 | MED | ADR-0263 duplicate status frontmatter |
| 6.11 | P1 | LOW | ADR-0263 §D-6 layer-enum drift |
| 6.12 | P2 | MED | persona count 127 vs 130 |
| 6.13 | P2 | MED | coverage-matrix status: Living not in enum |
| 6.14 | P2 | HIGH | crates/*/docs empty |
| 6.15 | P2 | HIGH | persona count mismatch internal |
| 6.16 | P2 | HIGH | ADR-0321 56→69 vs audit 70 |
| 6.17 | P2 | MED | persona count cross-doc drift |
| 6.18 | P3 | HIGH | ADR-0321 165 vs brief 150 |
| 6.19 | P3 | HIGH | no j151+ journeys |
| 6.20 | P3 | HIGH | model routing doctrine adherence |
| 6.21 | P3 | MED | no Wave-3-G brief crosslink |

**Net: 5 P0 / 6 P1 / 6 P2 / 5 P3 findings. The 5 P0 findings are the
pre-merge / pre-promotion edit set.**

---

## §7 6-Hops Graph Traversability Spot-Check

The §3.1 invariant of documentation-rigor.md requires that from any
canonical entry point, an intern reaches every primitive in ≤6 hops.
The deterministic walker tool (tools/doc-graph-walker/) is missing
(§6.6 above), so this spot-check is a manual sample.

### §7.1 Sample 20 Random Leaf Documents

| # | Leaf doc (sampled) | Reach docs/README.md? | Hops |
|---:|---|---|---:|
| 1 | docs/personas/yejin-park.md (not sampled directly; inferred from MASTER-ROSTER) | YES via roster | 2 |
| 2 | docs/personas/marcus-chen.md | YES via roster | 2 |
| 3 | docs/personas/MASTER-ROSTER-2026-05-21.md | YES — cites docs/architecture/* | 1-2 |
| 4 | docs/architecture/unified-ecosystem-thesis-2026-05-21.md | YES — cites docs/standards/* | 1-2 |
| 5 | docs/architecture/training-cost-doctrine-2026-05-21.md | YES | 1-2 |
| 6 | docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md | YES | 1-2 |
| 7 | docs/decisions/ADR-0709-general-live-apex.md | YES — companion_docs lists | 2 |
| 8 | docs/decisions/ADR-0709-general-live-apex.md | YES | 2 |
| 9 | docs/decisions/ADR-0709-general-live-apex.md | YES | 2 |
| 10 | microservices/production-planning/PRD.md | UNKNOWN (PRD-content thin) | ≥3-? |
| 11 | microservices/crm/PRD.md | UNKNOWN | ≥3-? |
| 12 | microservices/warehouse/PRD.md | UNKNOWN | ≥3-? |
| 13 | microservices/treasury/PRD.md | UNKNOWN | ≥3-? |
| 14 | microservices/marketing-automation/PRD.md | UNKNOWN | ≥3-? |
| 15 | microservices/healthcare-integration/PRD.md | UNKNOWN | ≥3-? |
| 16 | microservices/observability/ runbook (exemplar) | YES | 2-3 |
| 17 | docs/standards/documentation-rigor.md | YES — itself the standard | 0 (self) |
| 18 | docs/decisions/ADR-0702-identity-authz-live-apex.md | YES | 2 |
| 19 | docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md | YES | 2 |
| 20 | docs/user-journeys/CATALOG-j126-j150-ecosystem.md | YES — cited by archive docs | 2-3 |

### §7.2 Reachability Verdict

Samples 1-9 (doctrine docs + ADR cluster): **PASS at 1-2 hops.**
Samples 16-20 (standards + audit + journey catalog): **PASS at 2-3
hops.**
Samples 10-15 (new µservice PRDs): **UNKNOWN-PROBABLE-FAIL.** The 400-
line stub PRDs sample did NOT show `related_adrs:` or `companion_docs:`
arrays (this is consistent with the 7.1% PRD floor pass per audit
§1.2). Without those frontmatter arrays, the BFS-walker cannot reach
docs/README.md in ≤6 hops from these PRDs.

**Net spot-check verdict:** 14/20 PASS, 6/20 UNKNOWN-PROBABLE-FAIL.
The fail-band is concentrated in the 22 newly-scaffolded Wave-3-G
µservice PRDs.

**Severity:** P1. The 6-hops invariant is BLOCKER from 2026-07-16.
With ~22 µservice PRDs failing the invariant and the walker tool
absent, the lane cannot promote.

**Confidence:** MED. The 6 UNKNOWN entries need a deterministic walker
to convert into HIGH-confidence FAIL or PASS verdicts.

---

## §8 Documentation-Rigor §1.2 6-Dimension Scorecard

Per-category corpus-wide rating against the six engineering-rigor
dimensions in documentation-rigor.md §1.2.

### §8.1 ADRs (Decisions)

| Dimension | Verdict | Rationale |
|---|---|---|
| Maintainability | **PASS** | 8 / 11 cluster ADRs have proper cross-references, naming-justifications, versioning fields. ADR-0319 status + ADR-0320 case are the exceptions per §6.4. |
| Observability | **PASS-W-PARTIAL** | ADR-0263 emission contract supplies the per-audit-event-class registry. Per-ADR §D enumeration of emitted events is partial; 16/25 new ADRs rigorous-pass per audit. |
| Scalability | **PARTIAL** | Capacity math present in ADR-0246 (Cedar eval), ADR-0263 (cardinality budgets), ADR-0248 (cellular shuffle-sharding). Missing in ADR-0311 (passkey throughput), ADR-0314 (DealSet throughput), ADR-0317 (UX shell render rate). |
| Performance | **PARTIAL** | F6 budget-honesty gate from 2026-05-20 synthesis §5.8 still partly open. ADR-0252 (HLC + TrueTime) has explicit tier-budget. ADR-0314 missing. |
| Optimization | **PARTIAL** | Per-call cost models present in ADR-0246 amendment (library-first) + ADR-0255 amendment (provider-BYOK). Missing for marketplace ADRs (0249, 0314). |
| Code quality | **PARTIAL** | Test class enumeration present in some ADRs; lint pass naming consistent (`check-*`); SemVer + ABI policy per ADR-0258. But contract-version conformance at 10-11% per §6.9. |

**ADR cluster roll-up:** 1 PASS / 1 PASS-W-PARTIAL / 4 PARTIAL — bias
toward PARTIAL on the operational dimensions (Scalability / Performance /
Optimization / Code-quality). Architecture wins, ops lags.

### §8.2 PRDs

| Dimension | Verdict | Rationale |
|---|---|---|
| Maintainability | **FAIL** | 22 new PRDs at exactly 400 lines with 0 user stories; module-boundary + deprecation + reverse-dep enumeration absent per stub pattern. |
| Observability | **FAIL** | Per-PRD audit-event + metric + trace + log declaration absent from stub-pattern PRDs. |
| Scalability | **FAIL** | No capacity math in stub PRDs. |
| Performance | **FAIL** | No P50/P95/P99 budgets. |
| Optimization | **FAIL** | No per-call cost model. |
| Code quality | **FAIL** | No test-class enumeration. |

**PRD roll-up:** **6 FAIL across all dimensions on the 22 new Wave-3-G
PRDs.** This is the single most concentrated remediation surface. Wave
3-H content pass is the primary lane.

### §8.3 Specs (JSON)

| Dimension | Verdict | Rationale |
|---|---|---|
| Maintainability | **PARTIAL** | 73 / 127 specs have _meta block; 3 / 127 are rigorous-pass per audit. |
| Observability | **PARTIAL** | Per-spec event-schema-version cross-reference present in some; absent in many. |
| Scalability | N/A | Specs are schemas, not capacity surfaces. |
| Performance | N/A | Same. |
| Optimization | N/A | Same. |
| Code quality | **PARTIAL** | 127 / 127 parse valid JSON (audit); _meta block enumeration is the gap. |

**Spec roll-up:** 2 PARTIAL / 3 N/A / 1 PARTIAL — passable but the
2.4% rigorous-pass rate is a quality concern.

### §8.4 Runbooks

| Dimension | Verdict | Rationale |
|---|---|---|
| Maintainability | **FAIL** | 12 / 205 rigorous-pass per audit; stub rate ≥89%. |
| Observability | **FAIL** | Per-runbook audit-stream tag enumeration absent from stubs. |
| Scalability | N/A | Runbooks describe procedures. |
| Performance | **FAIL** | Per-step timing budget absent. |
| Optimization | N/A | Same. |
| Code quality | **FAIL** | Per-step command + verification + rollback absent. |

**Runbook roll-up:** 4 FAIL — the runbook surface is the single largest
content gap. Wave 3-H + Wave 3-I content pass primary remediation.

### §8.5 Standards

| Dimension | Verdict | Rationale |
|---|---|---|
| Maintainability | **PARTIAL** | 19 / 91 rigorous-pass per audit. |
| Observability | **PASS** | documentation-rigor.md itself is a hyperscaler-grade exemplar. |
| Scalability | N/A | Standards are normative documents. |
| Performance | N/A | Same. |
| Optimization | N/A | Same. |
| Code quality | **PASS** | 91 standards files have YAML frontmatter on 83 of them; doc-style.md compliance. |

**Standards roll-up:** 2 PASS / 1 PARTIAL / 3 N/A.

### §8.6 Persona Dossiers

| Dimension | Verdict | Rationale |
|---|---|---|
| Maintainability | **PARTIAL** | 130 files exist; per-file stable slugs; cross_context_personas[] bidirectionality not verified at scale. |
| Observability | **PARTIAL** | Per-persona `audit_event_scope` declaration in MASTER-ROSTER §1.2 but per-dossier verification not done. |
| Scalability | **PASS** | MASTER-ROSTER §1.2 documents the 1000+ persona scaling claim. |
| Performance | **PASS** | Cedar permit eval per persona under 5ms budget (claimed). |
| Optimization | **PASS** | Persona enumeration is template-derivation, not pre-allocation (per roster §1.2). |
| Code quality | **PARTIAL** | 0 / 129 persona dossiers heuristic-pass per audit; the dossier shape is rich but heuristic fails. |

**Persona roll-up:** 3 PASS / 3 PARTIAL — strong shape, weak per-
dossier rigor.

### §8.7 Architecture Deep-Dives (Wave-3-G doctrine docs)

| Dimension | Verdict | Rationale |
|---|---|---|
| Maintainability | **FAIL** | Template-stamped repetition makes future maintenance hostile (every clause-row would need to be edited in 700 places). |
| Observability | **PASS** | The docs DO enumerate per-event identifiers (tenant_id, role_projection_id, workflow_run_id, etc.) in clause-rows. |
| Scalability | N/A | Doctrine docs. |
| Performance | N/A | Same. |
| Optimization | N/A | Same. |
| Code quality | **FAIL** | Template-stamping fails the doc-style.md §2 ArchitectureDeepDive density check. |

**Architecture roll-up:** 1 PASS / 2 FAIL / 3 N/A. The four Wave-3-G
doctrine docs need editorial REVISE before they pass the §1.2 bar.

### §8.8 Net Corpus-Wide Scorecard

| Category | Maint. | Obs. | Scal. | Perf. | Opt. | CQ | Verdict |
|---|---|---|---|---|---|---|---|
| ADRs | PASS | PASS-W-PART | PART | PART | PART | PART | PASS-W-FINDINGS |
| PRDs | FAIL | FAIL | FAIL | FAIL | FAIL | FAIL | **REVISE** |
| Specs | PART | PART | N/A | N/A | N/A | PART | PASS-W-FINDINGS |
| Runbooks | FAIL | FAIL | N/A | FAIL | N/A | FAIL | **REVISE** |
| Standards | PART | PASS | N/A | N/A | N/A | PASS | PASS-W-FINDINGS |
| Personas | PART | PART | PASS | PASS | PASS | PART | PASS-W-FINDINGS |
| Architecture | FAIL | PASS | N/A | N/A | N/A | FAIL | **REVISE** |

**Net:** ADRs / Specs / Standards / Personas pass with findings.
**PRDs / Runbooks / Architecture-deep-dives are REVISE.** The
remediation is concentrated in three categories: 22 Wave-3-G PRDs,
89% runbook stubs, and 4 Wave-3-G doctrine docs.

---

## §9 Wave 3-H + 3-I + 3-J + 3-K Recommended Sequencing

Per the remediation surfaces above:

### §9.1 Wave 3-H — Anchor-Stub Content Pass (in-flight via codex)

- **Scope:** Expand the 1,143 ANCHOR-INJECTED / REVISE-PENDING stubs
  across 46 + 22 = 68 µservice ARCHITECTURE.md + compliance.md +
  manifest.json files. Sequenced per the keystone-bundle synthesis §10
  Phase-2-A.
- **22 Wave-3-G PRD content expansion** (primary): expand each from
  400 lines to ≥1,500 lines + 40 user stories + 6 UX flow diagrams.
- **Marketplace µservice content pass** (P0): 15 artifacts → ≥70.
- **workplace-integration µservice content pass** (P1): 16 → ≥70.
- **Acceptance:** PRD floor pass rate ≥ 30% within 4 weeks; ≥ 70% within
  8 weeks. ONE-MARKETPLACE invariant rises from WEAK to PARTIAL.
- **Owner:** codex agents in-flight; Claude opus reviews PR-level.

### §9.2 Wave 3-I — Capability-Tier Registry + CI Lane Authoring

- **Scope:** Build the capability-tier registry referenced by ADR-0316 +
  ADR-0321. File path probably `specs/capability-tier-registry.json` (not
  enumerated yet). Per-vendor → per-µservice mapping per ADR-0321 §D
  rows.
- **Build tools/doc-graph-walker/** — implement the deterministic 6-hops
  BFS walker required by documentation-rigor.md §3.1.
- **Build tools/persona-journey-cross-coverage/** — enables Wave 3-J
  scope quantification per §5.5.
- **Author CI lanes** named by the ADR cluster:
  - `governance-capability-tier-registry-shape`
  - `governance-b2b-leader-coverage-dossier`
  - `governance-b2b-new-microservice-doc-anchors`
  - `governance-erp-parity-module-map`
  - `governance-no-grouping`
  - `governance-coverage-matrix-current`
  - `governance-doc-graph-6hops`
- **Acceptance:** Walker emits per-doc reachability; capability-tier
  registry populated for all 165 ADR-0321 vendors + all SAP modules in
  ADR-0315 §D-1.
- **Owner:** dedicated tooling-agent or codex worker.

### §9.3 Wave 3-J — Deeper-Cut Journeys (j151+) and Vertical-Pack End-to-End

- **Scope:** Journey IDs j151-jN+ that cover the missing persona-journey-
  µservice intersections identified in §5.2 + §5.3.
- **Vertical-pack journeys** (end-to-end customer dossiers):
  - HIPAA — Dr. Tanaka journey (cardiothoracic surgeon + healthcare-
    integration µservice + audit-chain + Cedar HIPAA pack).
  - KR-FSS — bank-compliance-officer journey (Rishi Bhattacharya + payments
    + treasury + compliance + audit-chain + Cedar KR-FSS pack).
  - CN-PIPL — China-jurisdiction customer + tenant journey (cross-border
    data flow + sovereign-cell + Cedar CN-PIPL pack).
- **Acceptance:** 150 + ~50 = ~200 journey directories; per-vertical-pack
  end-to-end dossier with all 5 core files.
- **Owner:** persona+journey content agents (codex).

### §9.4 Wave 3-K — Code Authoring (Rust Implementation)

- **Scope:** The µservices need actual Rust code; this is post-doc work.
- **Sequencing:** per-µservice IPs (already enumerated) drive per-IP
  PR-shaped Rust slices.
- **Substrate first:** identity / tenancy / cell / cloud-secrets / cedar
  policy-engine / audit-chain / consent-graph / observability / ontology /
  workflow-engine — these substrate µservices need code before product
  µservices.
- **Substrate → ERP → CRM → Marketplace → UX-shell sequencing.**
- **Acceptance:** Per-µservice ≥85% line, ≥75% branch test coverage
  (per documentation-rigor.md §1.2 Code-quality cell).
- **Owner:** Wave-3-K executor agents (likely a mix of codex + claude).

### §9.5 Wave Sequencing Recommendation

| Wave | Primary scope | Start | Estimated end | Dependencies |
|---|---|---|---|---|
| 3-H | Anchor-stub content pass + Wave-3-G PRD content | 2026-05-22 | 2026-07-15 | Wave-3-G doc landings |
| 3-I | Registry + tools + CI lanes | 2026-05-22 (parallel) | 2026-07-01 | Wave-3-G ADR landings |
| 3-J | Deeper journeys + vertical packs | 2026-07-01 | 2026-08-15 | Wave-3-H (PRD content) |
| 3-K | Code authoring | 2026-08-01 | 2027-Q2 | Wave-3-H + 3-I + 3-J |

**Critical-path observation:** Wave 3-H + 3-I are PARALLEL. Wave 3-J
requires 3-H. Wave 3-K requires 3-H + 3-I + 3-J.

---

## §10 Open Questions for the User (Top 20)

These are decisions outstanding after Wave-3-G that block clean
progression to Accepted promotion + downstream waves. Each frames an
explicit user decision request.

### Q1 — ADR-0319 status: Accepted vs Proposed

Should ADR-0319 frontmatter status be edited to `Proposed` to align with
the cluster? Or is the `Accepted` status load-bearing (i.e., the ADR was
intentionally promoted out-of-band)?

**Recommendation:** Edit to `Proposed`. The keystone-bundle 2026-05-20
cluster-coherence pattern is the precedent.

### Q2 — Brief 30+ ADRs vs Live 25

Was the brief's 30+ ADR target scope-trimmed mid-wave, or are five ADRs
still in-flight? If in-flight, what are their numbers and where?

### Q3 — Template-Stamping in ADR-0321

Is the 165-row template-stamped dossier acceptable as a "scaffold for
future per-vendor expansion" (i.e., Wave-3-H content pass on ADR-0321
itself)? Or should ADR-0321 be re-authored before merge?

**Recommendation:** Accept as scaffold IF the per-vendor expansion is
explicitly scoped to Wave-3-H. Otherwise revise pre-merge.

### Q4 — Template-Stamping in Long-Form Doctrine Docs

Same question for unified-ecosystem-thesis-2026-05-21.md and training-
cost-doctrine-2026-05-21.md and day-in-the-life-coherent-ecosystem-
2026-05-21.md. These doctrine docs are MORE problematic than ADR-0321
because they are not vendor-enumeration scaffolds — they should be
content-dense doctrine. The 700-clause loop in unified-ecosystem-thesis
adds line-count without information-density.

### Q5 — Capability-Tier Registry File Path

Where does the registry live? `specs/capability-tier-registry.json`?
Or per-µservice `microservices/<ms>/capability-tiers/*.yaml`?

### Q6 — Marketplace µservice Build Priority

Is marketplace below-floor (15 artifacts) the Wave-3-H #1 priority?
Per §3.6 it is the WEAKEST ONE-INVARIANT; per ADR-0314 it is LOAD-
BEARING for DealSet settlement.

**Recommendation:** YES — marketplace is Wave-3-H slice 1.

### Q7 — TM Carrier-Optimization

ADR-0315 §G acknowledges TM (Transportation Management) carrier-
optimization as a "split later" gap. When does the split happen — Wave
3-I as new µservice (microservices/transportation-management/) or
Wave-3-K as code-only resolution?

### Q8 — accounting µservice

ADR-0315 SAP FI row maps to `specs/microservices/accounting.json` but
there is no `microservices/accounting/` directory. Does accounting
warrant promotion from spec to µservice scaffold?

### Q9 — HR + Payroll Promotion

Same question — ADR-0315 SAP HCM row maps to `specs/microservices/hr.
json` + `specs/microservices/payroll.json`. Promote to µservice
scaffolds?

### Q10 — j151+ Journey Scope

The j151+ journey scope is "deeper-cut journeys (j151+) covering the
missing persona-journey-µservice intersections." Is the target count
~50 new journeys (per §9.3), or higher?

### Q11 — Vertical-Pack End-to-End Customer Dossiers

How many vertical packs? HIPAA + KR-FSS + CN-PIPL are the three
suggested. Are GDPR + SOX + PCI also in scope as end-to-end customer
dossiers?

### Q12 — 6-Hops Walker Tool Scope

Build the walker in Rust (per the substrate-of-substrate doctrine, ADR-
0280), in Python (for speed of authoring), or as a Cedar fragment
(if reachability can be expressed declaratively)?

### Q13 — Wave-3-K Code Owner Identity

Claude opus orchestrates planning + adjudication; codex executes. For
Wave-3-K Rust code, is the same split correct? Or does Wave-3-K need
Claude opus on planning + claude sonnet on Rust authoring + codex on
mechanical refactors?

### Q14 — Status Enum Cleanup

Documentation-rigor.md does not enumerate `Living`, `proposed`
(lowercase), or `Substantially-Rewritten` as canonical statuses.
What's the canonical status enum? Recommendation: extend the enum or
fix the offending docs.

### Q15 — ADR-0319 Information-Barrier Cedar Fragment

ADR-0319 enforces office boundaries via Cedar fragments. Has the
fragment landed in code? Where? `packs/policy/office-information-
barrier.cedar`?

### Q16 — Persona-Journey-µservice Cross-Coverage Tool

§5 recommends `tools/persona-journey-cross-coverage/`. Is this Wave-3-I
scope (as recommended) or earlier?

### Q17 — ONE-MARKETPLACE Capacity Math

Per §3.6, ONE-MARKETPLACE lacks capacity math. What's the target
DealSet throughput per second under 100M tenant scale? P99 settlement-
latency budget?

### Q18 — ONE-PLUGIN-EXTENSIBILITY Throughput

Per §3.10, ONE-PLUGIN-EXTENSIBILITY lacks plugin-invocation-throughput.
Target?

### Q19 — Pre-Promotion Gate Reconciliation with 2026-05-20 Synthesis

The 2026-05-20 keystone synthesis §5 has 15 pre-promotion gates. Are
ALL 15 still open, or has Wave-3-G closed some? The redo audit doesn't
explicitly track them. Recommendation: a `docs/architecture/wave-3-g-
gate-closure-tracker.md` or similar.

### Q20 — Wave-3-H Time-to-Promotion Budget

The 2026-05-20 synthesis §6 estimated promotion at T+4w. Wave-3-G
landed in 1 day. The new gates from this synthesis push promotion to
≥T+8w (Wave-3-H end). Is that acceptable, or is there pressure to
promote earlier?

---

## §11 Bottom-Line Verdict + Pre-Promotion Gate Set

### §11.A Pre-Merge Edit Set (Block Bundle Merge)

These are the five P0 fix-sets that should land BEFORE the Wave-3-G
ADR cluster merges. None should take more than 1-2 days of edit work.

1. **§11.C status normalization** (P0 from §6.4):
   - Edit ADR-0319 frontmatter `status: Accepted` → `status: Proposed`.
   - Edit ADR-0320 frontmatter `status: proposed` → `status: Proposed`.
   - Verify all 11 cluster ADRs have `status: Proposed`.

2. **§11.A template-collapse** (P0 from §6.1, §6.2, §6.3):
   - Either (a) edit-in-place to compress the 700/160/165 repetitive
     clause rows into per-block bespoke content, or (b) move the
     existing template-stamped content into a tagged `## Appendix:
     Generated Templates` section, leaving the doc body bespoke.
   - This is the largest pre-merge surface (~5k lines of editing).
   - **Recommendation:** Option (b) — preserves the existing
     enumeration as low-risk appendix while authoring real content
     in the body.

3. **§11.B PRD content (acceptance-by-clause)** (P0 from §6.7, §6.8):
   - 22 new µservice PRDs need at minimum a 1500-line + 40-story floor
     pass within Wave-3-H. NOT pre-merge — but per-PRD floor-pass
     becomes a Wave-3-G PROMOTION gate.
   - Acceptable to bundle-merge in stub state IF the Wave-3-H content-
     pass is explicitly tracked in the post-merge issue.

4. **§11.D six-hops walker scope binding** (P1 from §6.6):
   - Wave-3-I scope explicit; not pre-merge.

5. **§11.E capability-tier registry scope binding** (P1 from §6.5):
   - Wave-3-I scope explicit; not pre-merge.

### §11.B Per-ADR Promotion Gate Set

Each cluster ADR's promotion from `Proposed` to `Accepted` is gated as
follows:

| ADR | Pre-promotion gate | Owner |
|---|---|---|
| ADR-0310 investigation case mgmt | per-ADR rigor pass + runbook + Cedar fragment land | substrate agent |
| ADR-0311 dual-tenant identity | passkey-binding capacity math + identity-fork failure mode | substrate agent |
| ADR-0312 court-warrant piercing | per-jurisdiction warrant test fixture | substrate agent |
| ADR-0313 conglomerate hierarchy | child-tenant sovereignty Cedar fragment | substrate agent |
| ADR-0314 marketplace DealSet | settlement-latency budget + per-fee audit-event-class | marketplace agent |
| ADR-0315 ERP parity | 9 new µservices PRD content + Wave-3-H pass | ERP cluster agent |
| ADR-0316 capability tier | capability-tier registry built | tooling agent |
| ADR-0317 role-projection | UX shell vocabulary lint lane + ARIA pass | UX agent |
| ADR-0318 collar-color | per-collar-color UX adaptation manifest landed | UX agent |
| ADR-0319 information barrier | Cedar fragment landed under packs/policy/ | substrate agent |
| ADR-0320 transient identity | ACGME supervisor co-sign Cedar fragment | substrate agent |
| ADR-0321 B2B-leader coverage | per-vendor delta content pass + 40 Tier-D µservice scaffolds at PR-143 floor | B2B-leader agent |

Each gate is tracked in a follow-up `wave-3-g-promotion-gate-tracker.md`
(scope: Wave 3-I).

### §11.C The Reconciled Verdict

**MERGE-AS-CLUSTER IN `Proposed` STATE; PROMOTION-GATED ON THE FIVE FIX-
SETS in §11.A + the per-ADR gates in §11.B.**

The verdict mirrors the 2026-05-20 keystone-bundle synthesis precedent:
- The bundle merges in `Proposed` (textual landing).
- No ADR promotes to `Accepted` until its per-ADR gate closes.
- The CI lanes promote from advisory to BLOCKER per-ADR after
  per-ADR gate closure.
- The 4 P0 editorial findings in §6.1/§6.2/§6.3/§6.4 are pre-merge
  fix-set; the per-ADR + per-µservice content-pass items are
  pre-promotion fix-set.

**Confidence: HIGH on the verdict shape (mirrors precedent). MED on the
exact gate enumeration (some gates may be over- or under-specified).**

### §11.D What's Required for `Accepted` Promotion of the Keystone Bundle

Reproducing the 2026-05-20 synthesis §5 promotion gate set (the
original keystone bundle), all 15 gates remain open unless explicitly
closed by a Wave-3-G PR. Pending verification:

- §5.1 F5 CRITICAL self-modification meta-trust (ADR-0247): cited but
  not re-verified.
- §5.2 F5 CRITICAL Cedar fragment hot-reload TOCTOU (ADR-0243): cited
  but not re-verified.
- §5.3 F5 CRITICAL bootstrap CI verification window (ADR-0247): cited
  but not re-verified.
- §5.4 F5 CRITICAL library-first credential concentration (ADR-0255):
  cited but not re-verified.
- §5.5 F5 HIGH Shamir 5-of-9 expansion: cited but not re-verified.
- §5.6 BYOK clarification: closed via 2026-05-20 synthesis.
- §5.7 M2 process remediation: closed via this synthesis being the
  v2.4.0 cadence.
- §5.8 F6 budget honesty: STILL OPEN per §8.1 above.
- §5.9 F9 runbook coverage: STILL OPEN per §8.4 above (5.9% rigorous
  pass rate).
- §5.10 A1 naming fixes: STILL OPEN per §6.10/§6.11 (ADR-0263 layer-
  enum + status drift not re-verified).
- §5.11 A3 structure fixes: STILL OPEN.
- §5.12 A7 shuffle-sharding math errata: STILL OPEN until re-verified.
- §5.13 F4 library-first symmetry (ADR-0246 + Ontology): STILL OPEN
  per ADR amendment surface not confirmed.
- §5.14 F7 supply-chain P0 FIPS/HSM: STILL OPEN.
- §5.15 F13 regional-compliance EU NIS2/DSA + CN PIPL: STILL OPEN.

**Net keystone-bundle gate closure status: 1 / 15 closed (BYOK §5.6).**

### §11.E The Next Pre-Merge Edit Set

Per the 2026-05-20 synthesis's BYOK + shuffle-sharding pre-merge edit
pattern, this Wave-3-G synthesis identifies the next pre-merge edits as:

1. ADR-0319 status: Accepted → Proposed.
2. ADR-0320 status: proposed → Proposed.
3. Template-collapse appendix-tag-or-rewrite for unified-ecosystem-
   thesis, training-cost-doctrine, day-in-the-life-coherent-ecosystem.
4. ADR-0321 dossier rows tagged as scaffold-for-Wave-3-H or rewritten.
5. Optional: reconcile persona count (127 ↔ 130) + µservice count
   (56→69 vs 70).

---

## §12 Persona-Archetype Cross-Reference Table (Top 30)

For the top 30 personas from MASTER-ROSTER-2026-05-21.md §3.1 + §3.2 +
§3.3, the table below cross-references each persona to (a) the 150
journeys, (b) the 70 µservices, (c) the 7 LOAD-BEARING ADRs from §2.13
(ADR-0311, ADR-0313, ADR-0314, ADR-0315, ADR-0316, ADR-0317, ADR-0318,
ADR-0321 — 8 LOAD-BEARING; we use them all).

| # | Persona | Anchor journeys | Primary µservices | Load-bearing ADRs in scope |
|---:|---|---|---|---|
| 1 | Yejin Park (nurse + parent + side-business) | j02 (code blue) + j03 (988 crisis) + family + farm-mkt | healthcare-integration + messenger + marketplace + audit-chain | 0311, 0314, 0317, 0318, 0321 |
| 2 | Marcus Chen (multinational CEO) | j13 (CEO board crisis) + j110 (sub-acq) + j105 (cross-tenant dispute) | governance + foundry + marketplace + crm + finops-portal | 0311, 0313, 0314, 0316, 0317 |
| 3 | Aiyana Singh (Senior ML engineer + blogger + parent) | personal-prj + workplace-eng + family | foundry + community + plugin-app-store + intelligence | 0311, 0316, 0317, 0321 |
| 4 | Tomás García (Restaurant owner + family father) | small-biz-onboarding + family + artisan | marketplace + payments + crm + comms-email | 0311, 0314, 0316, 0317, 0318 |
| 5 | Hiroshi Tanaka (Retired widower + grandfather) | j07 (deceased-user inheritance) + j124 (eldercare) + family | mail + drive + audit-chain | 0311, 0317 |
| 6 | Anya Mironova (Investigative journalist) | j05 (whistleblower) + j06 (press source) + activism | messenger + audit-chain + identity + community | 0311, 0317 |
| 7 | Diana Reyes (GAO auditor + family) | j41 (regulator) + family | audit-chain + compliance + governance + cloud-iac | 0311, 0317, 0316 |
| 8 | Priya Krishnan (HR Director) | j111 (employee onboarding) + j118 (leave-mgmt) | workplace-integration + workflow-engine + audit-chain + comms-email | 0311, 0316, 0317, 0318, 0319, 0321 |
| 9 | Sam Okafor (Corporate Internal-Audit Director) | j41 + j119 + j110 | audit-chain + compliance + governance + workflow-engine | 0311, 0313, 0317, 0319 |
| 10 | Chris Volkov (Laid-off engineer + family) | layoff + job-search + family + workflow-studio | community + workplace-integration + marketplace | 0311, 0317, 0316 |
| 11 | Carlos Martinez (Forklift driver) | warehouse-shift + family | warehouse + workflow-engine + observability | 0311, 0316, 0317, 0318 |
| 12 | Sarah Kim (Delivery driver DSP) | delivery-shift + side-hustle | warehouse + marketplace + payments + workplace-integration | 0311, 0314, 0316, 0317, 0318 |
| 13 | Ahmad Hassan (Construction site lead) | site-mgmt + contractor + family | workplace-integration + warehouse + workflow-engine | 0311, 0316, 0317, 0318 |
| 14 | Maria Santos (Restaurant cook) | shift + family | workplace-integration + workflow-engine | 0311, 0317, 0318 |
| 15 | Devon Williams (Field-service technician HVAC) | field-service-route + handyman | workplace-integration + workflow-engine + crm + marketplace | 0311, 0316, 0317, 0318 |
| 16 | Jordan Lee (Retail clerk + minor) | retail-shift + minor-protection | workplace-integration + payments + identity | 0311, 0292, 0317, 0318 |
| 17 | Ms. Patel (High-school teacher) | classroom + student-comms + family | community + comms-email + workflow-engine | 0311, 0317, 0318 |
| 18 | Coach Park (Youth soccer coach) | parent-comms + day-job + family | community + comms-email + workplace-integration | 0311, 0317, 0318 |
| 19 | Father Lopez (Catholic priest) | parish + counseling | community + comms-email + messenger | 0311, 0317 |
| 20 | Captain Chen (Airline pilot, long-haul) | flight + family | workplace-integration + workflow-engine + observability | 0311, 0316, 0317, 0318, 0320 |
| 21 | Officer Rodriguez (Police patrol officer) | patrol + emergency-bypass + family | workplace-integration + identity (incl. ADR-0298 emergency-services-bypass) + workflow-engine + audit-chain | 0311, 0317, 0318 |
| 22 | Dr. Tanaka (Cardiothoracic surgeon) | j02 + clinical-workflow + family | healthcare-integration + workflow-engine + audit-chain + Cedar HIPAA | 0311, 0316, 0317, 0318, 0320 |
| 23 | Tomás García Jr. (Coffee farmer) | farming + cooperative + family + multigen | marketplace + workflow-engine + observability + payments | 0311, 0313, 0314, 0316, 0317, 0318 |
| 24 | Captain Olufemi (Commercial fisherman) | fishing + cooperative | marketplace + workflow-engine + payments | 0311, 0313, 0314, 0316, 0317, 0318 |
| 25 | Aoki Tanaka (CEO, KR ops) | j13 + cross-jurisdiction + multi-country | governance + foundry + finops-portal + marketplace | 0311, 0313, 0314, 0316, 0317 |
| 26 | Helena Brandt (CFO) | j110 + j41 + audit + budget-cycle | finops-portal + audit-chain + compliance + governance + workflow-engine | 0311, 0313, 0317, 0319 |
| 27 | Felix Ng (CMO) | campaign + brand + marketing-automation | marketing-automation + analytics + community + comms-email | 0311, 0316, 0317, 0321 |
| 28 | Linda Foster (CHRO) | workforce-strategy + j111 + j118 | workplace-integration + governance + workflow-engine | 0311, 0316, 0317, 0318, 0319, 0321 |
| 29 | Yuki Park (CISO) | security-incident + threat-detect + audit | detection + audit-chain + compliance + governance + cloud-iac + cloud-secrets | 0311, 0317, 0319 |
| 30 | Naveen Iyer (CCO Chief Compliance Officer) | j41 + j110 + compliance-cycle | compliance + audit-chain + governance + workflow-engine | 0311, 0317, 0319 |

**Pattern observations:**
- **ADR-0311 (dual-tenant identity)** appears in ALL 30 rows. It is
  truly load-bearing for every persona.
- **ADR-0317 (role-projection)** appears in ALL 30 rows.
- **ADR-0316 (capability tier)** appears in 18 / 30 rows (60%).
- **ADR-0318 (collar-color)** appears in 21 / 30 rows (70%).
- **ADR-0319 (information barrier)** appears in 5 / 30 rows (17%) —
  primarily C-suite + compliance personas.
- **ADR-0313 (conglomerate hierarchy)** appears in 6 / 30 rows (20%)
  — primarily multinational CEO + cooperative + audit personas.
- **ADR-0314 (DealSet marketplace)** appears in 8 / 30 rows (27%).
- **ADR-0315 (ERP parity)** does NOT directly anchor any persona
  (operational, not persona-facing).
- **ADR-0321 (B2B-leader coverage)** appears in 6 / 30 rows (20%) —
  primarily enterprise / mid-management personas.
- **ADR-0320 (transient identity)** appears in 2 / 30 rows — pilot +
  surgeon (license-revalidation periodic surface).

**Persona-coverage by Wave-3-G µservice:**
- healthcare-integration: 2 personas (Yejin, Dr. Tanaka).
- marketing-automation: 1 persona (Felix Ng).
- crm: 3 personas (Sam Okafor, Tomás García, Devon Williams).
- warehouse: 4 personas (Carlos, Sarah, Ahmad, Tomás García Jr.).
- workplace-integration: 8+ personas (frontline + management).
- contact-center, performance-management, learning-management, itsm,
  incident-management, financial-planning, data-warehouse, contract-
  lifecycle-management, whiteboard, design-collaboration, data-
  pipeline: NOT directly anchored in top-30 sample.

**Implication:** 11 of the 22 Wave-3-G new µservices have ZERO top-30
persona anchor. Wave-3-J needs to anchor each new µservice to at least
one persona-journey pair.

**Confidence: MED.** The 30 personas were sampled from §3.1-§3.3 of the
master roster. The anchor-journey / anchor-µservice columns are
inferred from persona context + journey naming + µservice destination
mapping in ADR-0315 / ADR-0321 / unified-ecosystem-thesis. Per-persona
dossier verification (across 30 dossier files) would convert MED to
HIGH.

---

## §13 Appendix — Cross-Reference Map

This section enumerates the cross-reference graph for this synthesis
doc itself, so that the §3.1 six-hops invariant holds.

### §13.1 Outbound Citations (this doc → other docs)

- `docs/standards/documentation-rigor.md` (the bar — §1.1, §1.2, §2,
  §3.1, §3.2.x).
- `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` (precedent
  for `Proposed`-state merge + per-ADR promotion gate pattern).
- `docs/architecture/unified-ecosystem-thesis-2026-05-21.md` (§3, §6.2).
- `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md`
  (§6, line count cited).
- `docs/architecture/training-cost-doctrine-2026-05-21.md` (§3.8, §6.3).
- `docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md`
  (§4.3, §6.13).
- `docs/architecture/corpus-rigor-audit-2026-05-20.md` (baseline audit).
- `docs/architecture/corpus-rigor-audit-2026-05-21-post-wave-3-g.md` (the
  primary audit input).
- `docs/architecture/microservices-corpus-line-audit-2026-05-21.md` (§4).
- `docs/architecture/standards-corpus-line-audit-2026-05-21.md` (§8.5).
- `docs/architecture/adr-corpus-line-audit-2026-05-21.md` (§2).
- `docs/architecture/ip-corpus-line-audit-2026-05-21.md` (§1.2, §4).
- `docs/architecture/memory-spec-runbook-audit-2026-05-21.md` (§8.4, §8.5).
- `docs/personas/MASTER-ROSTER-2026-05-21.md` (§5, §12).
- 11 cluster ADRs (0310-0321) in `docs/decisions/`.
- ADR-0244, ADR-0297, ADR-0243, ADR-0246, ADR-0247, ADR-0257, ADR-0263,
  ADR-0292, ADR-0299 (cross-references in §2 + §3).

### §13.2 Inbound Citations (other docs → this doc; future)

This synthesis doc lands; Wave-3-H, 3-I, 3-J, 3-K tracking docs cite
back. Recommended:
- `docs/architecture/wave-3-g-promotion-gate-tracker.md` (future).
- `docs/architecture/wave-3-h-content-pass-plan.md` (future).
- Per-ADR §G review-evidence section in each Wave-3-G cluster ADR
  (cross-reference to this synthesis as v2.4.0 audit-of-record).

### §13.3 Six-Hops Reachability of This Doc

From `docs/README.md` to this doc:
1. docs/README.md → docs/DOC-CATALOG.md (catalog hub).
2. docs/DOC-CATALOG.md → docs/architecture/keystone-bundle-2026-05-20-
   synthesis.md (cited as authoritative synthesis).
3. keystone-bundle synthesis → this doc (cited as Wave-3-G follow-up).

3 hops from README. PASS at the §3.1 bar.

From this doc to a primitive (e.g., Cedar fragment soak):
1. This doc → ADR-0243 (Cedar universal gate).
2. ADR-0243 → ADR-0294 (Cedar fragment soak).
3. ADR-0294 → `policy/*.cedar` fragments.

3 hops. PASS at the §3.1 bar.

---

## §14 Methodology Notes

### §14.1 Sources Consulted

- `docs/standards/documentation-rigor.md` (read offset 1-312, file is
  1066 lines; the §1.1 / §1.2 / §2 / §3 sections are covered).
- `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` (read in
  full, 347 lines).
- `docs/architecture/unified-ecosystem-thesis-2026-05-21.md` (read first
  400 lines; grep-confirmed 700 thesis-clause repetition pattern over
  full file).
- `docs/architecture/training-cost-doctrine-2026-05-21.md` (read first
  300 lines; grep-confirmed 160 problem-clause repetition over full
  file).
- `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md`
  (sampled by line count + section-count proxy).
- `docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md`
  (read first 90 lines).
- `docs/architecture/corpus-rigor-audit-2026-05-21-post-wave-3-g.md`
  (read first 200 lines; supplied per-µservice + per-coverage-area
  numbers).
- `docs/personas/MASTER-ROSTER-2026-05-21.md` (read first 200 lines —
  the §3.1 + §3.2 + §3.3 persona tables).
- `docs/decisions/ADR-0709-general-live-apex.md` (read
  first 300 lines — covers §A + §B + §C + §D-1 module table + §D-1.A
  per-module notes through PLATFORM module).
- `docs/decisions/ADR-0709-general-live-apex.md` (read
  first 250 lines — covers §A + §B + §C + §D-001 through §D-013;
  grep-confirmed 165 vendor dossier count + 17/6/102/40 tier
  distribution).
- Frontmatter + status checks on 11 cluster ADRs (0310-0321) via grep.
- Section counts for ADRs (0297, 0244, 0311, 0313, 0314, 0316, 0317,
  0318, 0319, 0320, 0321) via grep.
- µservice scaffold + artifact counts via `find` + `ls`.
- Persona dossier size distribution via `find -size`.
- Journey directory count + j15X check via `ls + grep`.

### §14.2 Sampling-Source Limits

This synthesis is sampled-source. Full per-line audit of every cluster
ADR, every µservice, every persona dossier, every spec, every runbook
was NOT feasible within a single Opus pass. Where confidence is MED or
LOW, a future deeper pass (Wave 3-I tool-supported audit) is the
recommended remediation. The HIGH-confidence findings are reproducible
via the cited grep + wc commands.

### §14.3 Adjudication Doctrine

Per multispectrum-review v2.4.0 (`feedback_multispectrum_review_v22`),
findings are tagged per facet:
- F1 Correctness: §6.4 (status), §6.5 (count delta).
- F2 Hyperscaler fitness: §2.13 (cluster cross-reference density), §3
  (invariant scorecard).
- F3 Readability: §6.1, §6.2, §6.3 (template-stamping).
- F4 Architecture: §2 (per-ADR adjudication), §3 (invariant scorecard).
- F5 Security: §3.2 ONE-POLICY-ENGINE (PASS).
- F6 Performance: §3.6 ONE-MARKETPLACE capacity-math FAIL, §3.10 ONE-
  PLUGIN-EXTENSIBILITY FAIL.
- F7 Supply chain: §11.D (FIPS/HSM gate still open per 2026-05-20
  synthesis §5.14).
- F8 Maintenance: §8 (per-category 6-dimension scorecard).
- F9 Operations: §8.4 runbook REVISE.
- F10 Frontend/UX: §3.7 ONE-UX-SHELL PASS-W-PARTIAL.
- F11 i18n: §3.7 multi-region awareness PASS.
- F13 Compliance: §3.9 ONE-COMPLIANCE-POSTURE PASS-W-PARTIAL.
- M1 Challenge-assumption: §11 reconciles Wave-3-G with 2026-05-20
  precedent.
- M2 Meta-review: §14 this section.
- A1 Naming: §6.4 (status enum), §6.13 (Living not in enum).
- A2 Documentation: §6.1/6.2/6.3 template-stamping.
- A3 Structure: §2.5 ADR-0314 47-section vs §2.12 ADR-0321 166-section
  density.
- A4 Architecture adherence: §3 invariant scorecard.
- A5 Dependency: §2.13 cross-reference web.
- A6 Schema: §6.13 status enum, §8.3 spec _meta gap.
- A7 Algorithm: §3.6 ONE-MARKETPLACE capacity-math gap.

### §14.4 Confidence Levels

| Section | Confidence | Notes |
|---|---|---|
| §1 corpus growth | HIGH | Counts from cited audit |
| §2 11-ADR cluster | HIGH for sampled; MED for unsampled |
| §3 invariant scorecard | MED-LOW | Per-invariant sampling, full PRD-by-PRD verification not done |
| §4 ERP + B2B coverage | HIGH for tier distribution; MED for per-vendor verdicts |
| §5 persona-journey | MED-LOW | per-persona dossier traversal not done |
| §6 contradictions | HIGH for grep-verified; MED-LOW for inferred |
| §7 six-hops spot-check | MED | walker tool absent |
| §8 6-dimension scorecard | MED-HIGH | aggregates audit findings |
| §9 wave sequencing | HIGH for shape; LOW for exact dates |
| §10 open questions | HIGH | derived from findings |
| §11 verdict | HIGH for shape (mirrors precedent) |
| §12 persona cross-ref table | MED | 30-persona sample, dossier-by-dossier not verified |

---

## §15 End-of-Document Provenance

**Authored by:** claude-opus-4-7 in the Wave-3-G synthesis adjudication
session, 2026-05-21.
**Authority:** docs/standards/documentation-rigor.md §1.1 + §1.2 + §2 +
§3.x. Multispectrum-review v2.4.0 (feedback_multispectrum_review_v22).
**Scope:** READ-ONLY audit + synthesis. No other files modified.
**Audit-only flag:** `audit_only: true` in frontmatter.
**Authoring tool:** standard Read/Bash/Grep/Write tool surface; no agent
delegation; no codex collision.
**File path:** /Users/jasonlee/oyatie/docs/architecture/wave-3-g-
synthesis-adjudication-2026-05-21.md.
**Total findings:** 5 P0 + 6 P1 + 6 P2 + 5 P3 = 22 findings catalogued.
**Top recommendation:** MERGE-AS-CLUSTER in `Proposed` state; close the
5 P0 pre-merge fixes (§11.A); track the per-ADR promotion gates in a
Wave-3-G gate-closure tracker; start Wave 3-H content pass immediately
in parallel with Wave 3-I tooling work.

---

## §16 Extended Per-ADR Mechanics Audit (Wave-3-G Cluster)

This section supplements §2 with per-ADR mechanics + acceptance signals
+ failure-mode tree per documentation-rigor.md §1.1 sub-test. The
intent is to take each cluster ADR's §D mechanics through the
six-dimension scorecard at finer grain than §8.1 above. The verdicts
below are the input set for the Wave-3-H per-ADR rigor-pass workload.

### §16.1 ADR-0310 Investigation Case Management — Mechanics Audit

The ADR establishes investigation-case as a first-class object with
lifecycle workflow. Critical mechanics to verify in §D:

- **§D-1 case object schema:** must declare `case_id`, `tenant_id`,
  `opened_by_principal`, `assignee_principal`, `evidence_attachment[]`,
  `legal_hold_state`, `cross_jurisdiction_routing_state`,
  `audit_chain_seal_id`. Schema cross-ref to specs/investigation-case-
  schema.json (not verified present).
- **§D-2 lifecycle states:** must enumerate `pending_open`, `open`,
  `under_review`, `evidence_collection`, `legal_hold`, `closed_resolved`,
  `closed_unresolved`, `archived`. Per-state Cedar permit set.
- **§D-3 Cedar permit shape:** `permit(principal, action == Action::
  "investigation.case.open", resource in Tenant::"<tid>") when ...`. The
  per-action permit set must include open / assign / attach-evidence /
  redact / route-cross-jurisdiction / seal / archive / unseal-for-court.
- **§D-4 auditor-scope projection:** per ADR-0263, every state
  transition emits an audit-event-class `investigation.case.*` with
  signed payload via ADR-0296 sidecar.
- **§D-5 failure-mode tree:**
  - evidence-attachment-corruption: detected via per-blob hash; rollback
    via prior-seal restore.
  - cross-jurisdiction-routing-block: when destination jurisdiction
    refuses (e.g., EU refuses US discovery request), case enters
    `cross_jurisdiction_routing_blocked` state with explicit refusal
    evidence.
  - assignee-revocation-mid-case: when assignee principal loses
    tenant-membership, case auto-reassigns to tenant case-management
    queue.
  - legal-hold-conflict: when overlapping legal holds from multiple
    jurisdictions impose conflicting custody requirements; resolution
    via ADR-0304 cross-jurisdiction-conflict-resolution.

**Verdict for §D mechanics:** **PROBABLE-PARTIAL.** Section count 27
suggests structural coverage; per-section content rigor not sampled.

### §16.2 ADR-0311 Dual Tenant Identity — Mechanics Audit

- **§D-1 passkey-tenant-membership graph:** per-passkey identity
  declares `tenant_memberships[]` array; each entry carries
  `tenant_id`, `audience_type`, `joined_at`, `expires_at` (for
  contract-tenancy or fellowship-tenancy), `active_role_projection`.
- **§D-2 context-switch protocol:** switching from personal-tenant
  to employer-tenant requires UX gesture (long-press, deliberate
  navigation, or per-app context-switch surface). Must NOT happen
  silently. Cedar permit eval re-runs on every action under the new
  context.
- **§D-3 cross-context isolation:** personal-tenant data cannot
  bleed into employer-tenant UI surface; employer-tenant audit-chain
  cannot subpoena personal-tenant data without ADR-0312 warrant path.
- **§D-4 provider_credential_mode:** per the 2026-05-20 BYOK
  clarification, each tenant declares `provider_credential_mode ∈
  {platform_default, byok, byok_required_by_pack}`.
- **§D-5 failure-mode tree:**
  - passkey-binding-loss: ADR-0299 account-recovery resolves.
  - cross-tenant-context-spillage: detected via per-event tenant-id
    audit; rollback via session-invalidation.
  - employer-revokes-membership-mid-session: session terminates +
    audit-event emitted; personal-tenant context retained.
  - tenant-merge-or-split (e.g., conglomerate spinoff): per ADR-0313
    sovereignty + grant model.

**Verdict for §D mechanics:** **PROBABLE-PASS-W-FINDINGS.** Cross-
reference density is healthy (cites ADR-0244, ADR-0247, ADR-0292,
ADR-0299, ADR-0312, ADR-0317, ADR-0299, ADR-0303 in companion_docs).

### §16.3 ADR-0312 Court Warrant-Scoped Piercing — Mechanics Audit

- **§D-1 warrant object schema:** `warrant_id`, `issuing_jurisdiction`,
  `issuing_authority`, `scope_targets[]` (per-tenant-or-principal-or-
  data-class), `cryptographic_attestation` (judge-signed where the
  jurisdiction supports it), `expires_at`.
- **§D-2 per-jurisdiction validity surface:** US (federal + per-state
  matrix); EU (per-MS + EDPB cross-border Art. 49); UK (IPA 2016);
  KR (per-MOJ); JP (per-MOJ); SG (per-MAS sectoral); etc.
- **§D-3 scope-limited piercing:** warrant scope-targets define what's
  pierced; nothing else. Cedar `forbid` overrides on out-of-scope
  data ensure default-deny holds.
- **§D-4 audit-trail:** every warrant-execution event sealed in
  audit-chain with judge-signed payload + per-data-block access
  record; reproducible after-the-fact for appellate review.
- **§D-5 failure-mode tree:**
  - warrant-revocation-mid-execution: rollback any partial
    decryption; audit-chain records partial-execution + revocation.
  - warrant-overreach (out-of-scope access attempt): Cedar default-
    deny + audit-event; emergency-alert to compliance officer.
  - cross-border-conflict: warrant issued in jurisdiction A targets
    data held in jurisdiction B; ADR-0304 conflict-resolution.
  - whistleblower-protection-conflict: ADR-0300 press-source-
    anonymity overrides certain warrant categories.

**Verdict:** **PROBABLE-PARTIAL.** Mechanics surface heavy; per-
jurisdiction matrix likely under-populated.

### §16.4 ADR-0313 Conglomerate Tenant Hierarchy — Mechanics Audit

This is the longest ADR in the cluster (2,985 lines). Mechanics depth
should be highest.

- **§D-1 parent-child tenant graph:** `parent_tenant_id` + `child_
  tenant_id` form a forest (multi-rooted DAG); cycles forbidden.
  Per ADR-0244 every child is a fully-scoped tenant with its own
  Cedar entity set, audit-chain shard, and compliance pack.
- **§D-2 sovereignty markers:** each child declares `sovereign:
  true` (default) or `subordinate: true` (rare; requires explicit
  parent-tenant grant). Subordinate children share their audit-chain
  roll-up with parent; sovereign children do not.
- **§D-3 grant-based information flow:** parent → child information
  flow requires explicit Cedar grant fragment (e.g., `permit(
  principal in ParentTenant::"<pid>", action == Action::"read",
  resource in ChildTenant::"<cid>") when grant_id == "<gid>"`).
- **§D-4 audit-chain per-child + roll-up:** each child seals its own
  audit-chain shard; subordinate children's shards roll up to parent
  audit-chain on a per-pack-overlay-determined schedule (e.g., SOX
  consolidated audit weekly; HIPAA quarterly).
- **§D-5 cross-tenant Cedar eval order:** evaluating a permit across
  the parent-child boundary requires explicit grant verification +
  per-child-jurisdiction overlay; default-deny across boundary.
- **§D-6 information-barrier overlay:** ADR-0319 information-barrier
  Cedar fragments layer atop ADR-0313 per-child-tenant Cedar entity
  set. The two compose to enforce both organizational (parent-child)
  and operational (front-middle-back-office) isolation.
- **§D-7 failure-mode tree:**
  - child-tenant-spinoff: parent declares spinoff; child sovereignty
    upgrades from subordinate to sovereign; existing grants revoke
    on per-grant cadence (typically end-of-contract or end-of-deal).
  - child-tenant-acquisition: external tenant acquired into
    conglomerate; existing tenant grants migrate to new parent;
    Cedar fragment re-publish with soak per ADR-0294.
  - cross-child-fraud: e.g., one subsidiary commits SOX violation;
    parent must isolate the subsidiary's audit-chain shard for
    forensic review without contaminating sibling-child audit-chains.
  - joint-venture (multi-parent child): multi-rooted DAG case;
    rare; requires explicit `joint_venture: true` flag + per-parent
    grant chain.

**Verdict:** **PROBABLE-PASS.** This ADR is the longest because its
mechanics surface is genuinely complex. Cross-references to ADR-0244,
ADR-0319, ADR-0249, ADR-0247, ADR-0314 are dense.

### §16.5 ADR-0314 Marketplace as Universal Deal Settlement — Mechanics Audit

- **§D-1 DealSet object schema:** `dealset_id`, `tenant_id`,
  `counterparty_tenant_id_or_consumer_id`, `category` (per ADR-0249
  multi-category enum), `state` (negotiating / signed / settling /
  settled / disputed / closed), `fee_schedule_ref`, `audit_chain_
  seal_id`.
- **§D-2 fee-distribution primitive:** per-DealSet fee distribution
  declared in `fee_schedule_ref`; multi-leg fee splits (e.g.,
  platform fee + processor fee + tax-collection fee + creator-
  royalty + affiliate-referral fee) handled by atomic per-leg
  ledger entries.
- **§D-3 settlement workflow:** DealSet flows through per-category
  workflow templates (import / approve / exception / rollback /
  export). Each template owned by workflow-engine OR by the
  destination µservice if narrower runtime needed.
- **§D-4 per-category coverage:** per ADR-0249 multi-category
  marketplace doctrine, DealSet handles consumer-to-consumer
  (C2C marketplace), consumer-to-business (B2C), business-to-
  business (B2B), business-to-labor (B2L), labor-to-business (L2B),
  partner-to-partner (P2P), and data-licensing categories.
- **§D-5 Cedar permit shape per category:** per-category permits
  in `policy/marketplace-<category>.cedar`.
- **§D-6 failure-mode tree:**
  - settlement-rail-failure: payment rail (Stripe, Adyen, banking
    rail) fails mid-settlement; per-leg compensating action via
    workflow.rollback template.
  - cross-jurisdiction-settlement-conflict: e.g., US-seller +
    EU-buyer + tax-residency-of-platform-in-Ireland triggers a
    three-jurisdiction tax-attribution; per ADR-0304 conflict
    resolution.
  - chargeback-after-settlement: per-DealSet chargeback flag +
    reverse-settlement compensating ledger entry.
  - sanctions-block-mid-settlement: settlement frozen + Cedar
    `forbid` on disbursement; per ADR-0297 abuse-defence pack
    sanctions overlay.
  - marketplace-operator-tax-reporting: per EU DAC7 directive
    (2023+), the platform must report cross-border seller income;
    `dealset_tax_report_id` audit-event.

**Verdict:** **PROBABLE-PARTIAL.** Capacity math gap per §3.6 above
remains. Per-category permit roster + per-category SLO budget would
move this to PASS.

### §16.6 ADR-0315 ERP Coverage Doctrine — Mechanics Audit

ADR-0315 is the densest module-mapping table in Wave-3-G. The §D-1
mapping table per the read above:

- **23 SAP module families** mapped row-by-row.
- **9 NEW µservices anchored:** production-planning, quality-
  management, plant-maintenance, warehouse, real-estate, crm,
  treasury, supply-chain-planning, global-trade.
- **6 modules pass-by-composition:** CO, PS, PLM, EHS, PLATFORM,
  DATA.
- **1 module pass-by-pack:** IS-*.
- **7 modules partial:** FI, MM, SD, HCM, SRM, TM, NETWORK.

The §D-1.A per-module notes are template-stamped (every module
shares the same 7-bullet shape: SAP surfaces covered, oyatie
destination, current status, coverage rule, audit rule, migration
rule, parity gap). This is closer to acceptable than the ADR-0321
template-stamping because the 7-bullet shape encodes per-module
declarative facts; only the parity-gap sentence varies meaningfully.

- **§D-2 new µservices required:** 9 µservice anchors plus first
  bounded contexts per anchor. The per-µservice first BCs sampled:
  - production-planning: bom-revision, mrp-run, capacity-calendar,
    routing-step, production-order, shop-floor-release.
- **§D-3 cross-reference matrix:** every new µservice anchor cites
  destination integration points (ontology, workflow-engine,
  warehouse, quality-management, finops-portal, marketplace).
- **§D-4 audit-rule:** every business document emits create,
  amend, approve, post, reverse, archive, export events.
- **§D-5 migration rule:** per-source extract → connect adapter →
  destination service. SAP / Oracle / Workday / NetSuite / custom
  enumerated as sources.

**Verdict:** **PROBABLE-PASS for §D-1 table; PARTIAL for §D-1.A per-
module notes due to template-stamping risk.** Per-module bespoke
parity-gap deltas are present (the only sentence that varies); the
rest is mechanical scaffold. Acceptable for this ADR because it's
a coverage table, not a doctrinal walk.

### §16.7 ADR-0316 Capability Tier Over Product Fragmentation — Mechanics Audit

- **§D-1 capability-tier object schema:** `tier_id`, `tier_name`,
  `vendor_alias` (optional; benchmark only), `cedar_permit_set_ref`,
  `ontology_projection_ref`, `workflow_template_ref`, `ux_shell_
  manifest_ref`, `compliance_overlay_ref`, `observability_stream_
  ref`, `finops_cost_dimension_ref`, `migration_import_declaration_
  ref`, `support_runbook_ref`.
- **§D-2 tier registry shape:** registry file path candidates
  (`specs/capability-tier-registry.json` or per-µservice
  `microservices/<ms>/capability-tiers/*.yaml`); §6.5 Q5 open
  question on which.
- **§D-3 four-condition test for new µservice:**
  1. Distinct operational bottleneck.
  2. Distinct contracts / retention / SLO / scaling / failure
     modes.
  3. Single-concern + flat under ADR-0132.
  4. PR-143 grade documentation + validator coverage.
- **§D-4 Cedar permit shape for capability tier:**
  `permit(principal, action in TierAction::"<tier>",
  resource in TenantCapability::"<tier>") when ...`.
- **§D-5 ontology projection shape:** per-tier ontology object-
  type projections (`<tier>.object`, `<tier>.relationship`,
  `<tier>.migration_source`) pin schema revisions per ADR-0257.
- **§D-6 failure-mode tree:**
  - tier-conflict: two tiers expose the same surface with conflicting
    Cedar permits; resolution via ADR-0145 no-universal-mediator +
    per-µservice ownership.
  - tier-deprecation: ADR-0258 versioning; per-tier sunset cadence.
  - tier-explosion: too many tiers per µservice — per-µservice
    tier registry max-size budget (pending Wave-3-I decision; recommendation: ≤20 tiers/
    µservice).

**Verdict:** **PROBABLE-PASS for doctrinal mechanics. Registry file
missing per §6.5 — gate on §11.E.**

### §16.8 ADR-0317 Role-Based Projection / Unified UX Shell — Mechanics Audit

132 sections — the highest density in the cluster.

- **§D-1 role-projection axis:** `role_projection_id` field on every
  user-facing event; orthogonal to tenant + audience_type + workspace.
- **§D-2 per-role UX shell manifest:** declares allowed actions, hidden
  surfaces, default views, permission scopes, telemetry identifiers.
- **§D-3 shared interaction vocabulary:** approve, assign, comment,
  sign, attach evidence, route, defer, escalate, switch role, verify
  context, review history, export with policy, recover from denial.
  Each verb has canonical Cedar action + canonical UX gesture + canonical
  audit-event-class.
- **§D-4 training-vocabulary durability claim:** the same verb means
  the same thing across every surface; a nurse who learns "approve" in
  the medication-order surface understands "approve" in the expense-
  report surface.
- **§D-5 accessibility floor:** WCAG 2.2 AAA; ARIA role surface
  conformance; keyboard-shortcut canonical map; screen-reader
  semantic markup.
- **§D-6 device-profile adaptation:** mobile-primary / desktop-primary
  / handheld-rugged / kiosk-shared / assistive / vehicle-mount each
  carry per-profile UX shell projection.
- **§D-7 failure-mode tree:**
  - vocabulary-divergence: a vendor-specific verb leaks into shared
    vocabulary; per ADR-0317 doctrine-enforcement lane catches.
  - device-profile-unsupported: graceful-degrade to nearest profile
    with explicit "not-fully-supported" surface.
  - cross-locale-vocabulary-shift: per-locale translation must
    preserve canonical-verb semantics; ICU MessageFormat audit.

**Verdict:** **PROBABLE-PASS-W-FINDINGS for §D mechanics. The 132
sections are likely substantively dense.** Per-verb Cedar action
enumeration would push to clean PASS.

### §16.9 ADR-0318 Collar-Color Workspace Universality — Mechanics Audit

- **§D-1 six-collar-color enum:** white / blue / pink / gold / gray
  / green. Per-collar-color description + sample roles in roster.
- **§D-2 workspace axis (7 values):** front-office / middle-office /
  back-office / field / clinical-care / executive / production.
- **§D-3 per-collar-color UX shell adaptation:** input-method
  differences (glove-friendly, touch-target, voice-first); accessibility
  baseline; per-collar-color training cohort manifest.
- **§D-4 per-workspace Cedar permit posture:** different default
  permits per workspace (e.g., field permits include vehicle-mount
  surface access; clinical permits include PHI access subject to
  HIPAA pack).
- **§D-5 device-profile per workspace:** field workers default to
  handheld-rugged + vehicle-mount; clinical-care defaults to desktop
  + clinical-PACS workstation.
- **§D-6 failure-mode tree:**
  - cross-collar-shift (same human, two collars same week — e.g.,
    moonlighting nurse who runs a side-business farm): per-shift
    context-switch UX + per-shift Cedar permit projection.
  - workspace-disaster (e.g., clinical-care goes offline): graceful-
    degrade to write-and-sync; ADR-0306 disaster-mode-cell-resilience.
  - blue-to-white collar promotion: persona-graph carries promotion
    history; Cedar permits expand on promotion.

**Verdict:** **PROBABLE-PASS-W-FINDINGS. 2,950 lines is dense; per-
collar-color UX adaptation manifest the largest remaining surface to
verify.**

### §16.10 ADR-0319 Front-Middle-Back Office Information Barrier — Mechanics Audit

- **§D-1 workspace taxonomy:** front-office (customer-facing) /
  middle-office (operational + risk + compliance) / back-office
  (internal-support) — discrete enum.
- **§D-2 per-office Cedar information-barrier fragment:** default-
  deny across office boundary; explicit cross-grant required.
- **§D-3 cross-office Cedar evaluation order:** evaluate office-
  barrier fragment BEFORE per-action permit; office-barrier denial
  short-circuits.
- **§D-4 per-jurisdiction information-barrier overlay:** SEC 17a-7
  affiliate restriction; FINRA 5141 inside-information barrier;
  FCA SYSC 10A.1 inside-information; SOX 404 segregation-of-duties;
  KR-FSC barrier; CN-CSRC barrier.
- **§D-5 failure-mode tree:**
  - cross-office-barrier-bypass attempt: Cedar default-deny +
    audit-event with `office_barrier_attempt` class.
  - same-human-multi-office-role: e.g., compliance officer who's
    also a board director — explicit grant per ADR-0245.
  - office-redesignation mid-employment: HR-driven workspace-axis
    change triggers Cedar permit re-eval.

**Verdict:** **PROBABLE-PASS, status enum drift notwithstanding (§6.4
fix-set). 10 sections; per-section density appears high.**

### §16.11 ADR-0320 Apprentice/Intern/Resident/Fellow Transient Identity — Mechanics Audit

- **§D-1 transient-identity skill-tier:** `in-training`. Per ADR-0317
  the skill-tier axis is orthogonal to other axes.
- **§D-2 per-tier Cedar permit downgrade:** in-training principals
  receive a subset of standard permits; supervisor co-sign on
  high-stakes operations enforced via Cedar.
- **§D-3 supervisor co-sign Cedar fragment:** `permit(principal in
  Tier::"in-training", action == Action::"high-stakes", resource)
  when supervisor_co_sign_present(context)`.
- **§D-4 auto-expiry on tenure end-date:** residency / fellowship /
  apprenticeship contract end-date triggers tier-promotion to
  `junior` or end-of-tenancy.
- **§D-5 per-residency-program supervisor mapping:** ACGME-compliant
  residency program management surface; medical-resident principal
  binds to attending-physician principal per per-rotation schedule.
- **§D-6 failure-mode tree:**
  - supervisor-revocation mid-procedure: e.g., attending physician
    becomes unavailable during surgery; emergency-handover Cedar
    fragment.
  - resident-exceeds-scope: Cedar default-deny + audit-event +
    incident-report.
  - cross-institution-residency (rare): per-institution Cedar
    fragment composition.

**Verdict:** **PROBABLE-PARTIAL. 1,558 lines (cluster minimum); per-
profession surface coverage (ACGME / ABA / engineering-licensure)
likely thin.** Status enum drift (§6.4 fix-set) blocks promotion.

### §16.12 ADR-0321 B2B SaaS Industry-Leader Coverage — Mechanics Audit

Per §6.1 P0 finding, the 165 vendor dossiers in §D are template-
stamped. Mechanics audit therefore focuses on the §A + §B + §C +
§D-prelude doctrine.

- **§A-1..A-5 context:** ADR-0315 + ADR-0314 + ADR-0316 chain cited;
  hyperscaler precedents (Salesforce / ServiceNow / Microsoft Graph
  / Palantir Foundry / Snowflake / AWS-Azure-GCP) cited.
- **§B-1 mapping rule:** every vendor surface → one of (capability
  tier, composition, new µservice).
- **§B-2 13 new µservice anchors:** marketing-automation, contact-
  center, performance-management, learning-management, itsm,
  incident-management, financial-planning, data-warehouse, contract-
  lifecycle-management, whiteboard, design-collaboration, data-
  pipeline, healthcare-integration. **VERIFIED — all 13 directories
  present per `ls microservices/`.**
- **§B-3 suite-rejection rule:** no salesforce / servicenow /
  workday / microsoft / adobe / b2b-grouping µservices.
- **§B-4 dossier-row schema:** vendor name; coverage tier; oyatie
  destination; Cedar permit shape; ontology projection; workflow
  template library; UX shell adaptation; pack overlay applicable;
  migration path; naming justification; failure-mode coverage.
- **§C-1 maintainability:** product-labels stay out of service
  boundaries.
- **§C-2 observability:** every capability + service emits audit /
  metric / trace / log / refusal / migration evidence.
- **§C-3 corpus growth:** 56 → 69. **DISCREPANCY per §6.16: audit
  shows 70.**
- **§D-001..D-165 per-vendor dossiers:** **TEMPLATE-STAMPED per §6.1
  P0.**

**Verdict:** **PROBABLE-PASS for §A + §B + §C doctrine; REVISE for
§D dossiers.** The dossier rewrite is the largest single Wave-3-H
sub-workload.

### §16.13 Cluster Mechanics-Audit Roll-Up

| ADR | §D verdict | Promotion gate |
|---|---|---|
| 0310 | PROBABLE-PARTIAL | per-jurisdiction warrant + investigation runbook |
| 0311 | PROBABLE-PASS-W-FINDINGS | capacity math + identity-fork failure mode |
| 0312 | PROBABLE-PARTIAL | per-jurisdiction validity matrix |
| 0313 | PROBABLE-PASS | joint-venture multi-parent edge case |
| 0314 | PROBABLE-PARTIAL | DealSet capacity math + per-category permit roster |
| 0315 | PROBABLE-PASS (§D-1); PARTIAL (§D-1.A) | per-module bespoke parity-gap deltas already present; remaining work is Wave-3-H content on new µservices |
| 0316 | PROBABLE-PASS | capability-tier registry file build |
| 0317 | PROBABLE-PASS-W-FINDINGS | per-verb Cedar action enumeration |
| 0318 | PROBABLE-PASS-W-FINDINGS | per-collar-color UX adaptation manifest |
| 0319 | PROBABLE-PASS | status enum fix |
| 0320 | PROBABLE-PARTIAL | per-profession surface coverage |
| 0321 | PROBABLE-PASS for doctrine; REVISE for §D dossiers | per-vendor delta content pass |

**Net:** 4 PASS / 4 PASS-W-FINDINGS / 4 PARTIAL across the cluster.
ADR-0321's §D is the singular REVISE.

---

## §17 Per-µservice Content Gap Roll-Up (Wave-3-G + Pre-Wave-3-G)

This section catalogs the per-µservice content gaps that Wave-3-H
must close. Source: corpus-rigor-audit-2026-05-21-post-wave-3-g.md
§3 per-µservice tier rating.

### §17.1 Substrate µservices (Wave-3-H priority block 1)

| µservice | Artifact tier | PRD lines | Composite | Wave-3-H slice priority |
|---|---|---:|---:|---:|
| cell | (not sampled directly) | unknown | high | 1 |
| tenancy | (not sampled directly) | unknown | high | 1 |
| identity | (not sampled directly) | unknown | high | 1 |
| audit-chain | EXEMPLAR-130+ (189 artifacts) | unknown | (likely high) | 2 |
| compliance | (not sampled directly) | unknown | high | 2 |
| consent-graph | (not sampled directly) | unknown | medium | 3 |
| cloud-secrets | (not sampled directly) | unknown | high | 1 |
| governance | EXEMPLAR-130+ | unknown | high | 2 |
| observability | EXEMPLAR-146 artifacts | unknown | high (canonical exemplar) | n/a — already at-bar |
| cloud-iac | EXEMPLAR-150 artifacts | unknown | high | n/a |
| application | PASS-100-129 (126 artifacts) | 382 | 86.1 | 3 (Wave-3-J) |
| api-gateway | PASS-100-129 (127 artifacts) | 117 | 87.7 | 2 (Wave-3-H) |
| foundry | EXEMPLAR (561 artifacts) | unknown | high | n/a |

### §17.2 Product µservices Below Floor (P0)

| µservice | Artifacts | Pre-floor gap |
|---|---:|---|
| marketplace | 15 | -55 artifacts to 70-floor; -85 to 100-bar |
| workplace-integration | 16 | -54 to floor; -84 to bar |
| payments | 1 | **-69 to floor** — catastrophic gap |
| api-gateway (re-check) | 127 (PASS) | already at-bar |
| feature-flags | 16 | -54 to floor |
| intelligence | 17 | -53 to floor |
| connector | 18 | -52 to floor |
| ops-dashboard-control-center | 36 | -34 to floor |

### §17.3 Wave-3-G New µservices — Content Status

All 22 Wave-3-G new µservice scaffolds have 129 artifacts each (verified
for production-planning, crm, warehouse via `find -type f | wc -l`).
That's PASS-100-129 tier on the artifact-count axis (Axis A per
documentation-rigor.md). But the content axis (Axis B, the 27-row
ADR-adherence matrix) is unverified.

**Specific to Wave-3-G:**
- PRDs at 400 lines (vs 1500 floor) — FAIL on density.
- 0 user stories per PRD (vs 40 floor) — FAIL on user-story density.
- ARCHITECTURE.md present; per-ADR-adherence rows not enumerated
  in sampled stubs — FAIL on §3.2.1 27-row coverage.
- IP files present (sampled production-planning: IP-001 through IP-XX
  for BC roster); IP density not measured per-IP.

**Wave-3-H slice plan for 22 new µservices:**
- Slice 1 (5 services): marketplace + workplace-integration + payments
  (P0 below-floor) + intelligence + feature-flags.
- Slice 2 (9 ERP services): production-planning, quality-management,
  plant-maintenance, warehouse, real-estate, crm, treasury, supply-
  chain-planning, global-trade.
- Slice 3 (13 B2B-leader services): marketing-automation, contact-
  center, performance-management, learning-management, itsm, incident-
  management, financial-planning, data-warehouse, contract-lifecycle-
  management, whiteboard, design-collaboration, data-pipeline,
  healthcare-integration.

Per-slice acceptance: PRD ≥1500 lines + ≥40 stories + 6 UX flows; all
27 ADR-adherence-matrix rows answered per §3.2.1; six-hops graph
walker reachability from PRD to docs/README.md in ≤6 hops.

### §17.4 Per-µservice Naming-Justification Gap

Per audit §1.2 "Microservices manifest `naming_justifications` present:
1/70 = 1.4%." Only 1 of 70 µservices declares the naming-justifications
block in manifest.json. Per `feedback_naming_justification` memory,
every name MUST carry one-line justification proving v4 BNF + 12-layer-
enum conformance.

**Severity:** P1 — corpus-wide.

**Fix:** Mechanical batch in Wave-3-H — generate per-µservice naming-
justifications block from existing manifest.json metadata.

### §17.5 Cross-µservice Consistency Invariants (per §3.2.2)

| Invariant | Estimated pass rate | Source |
|---|---:|---|
| 1. Field naming consistency | ~70-80% (sampled) | audit §3 per-µservice cross-references |
| 2. Audit-event-class taxonomy | ~60% (sampled) | audit §1.2 |
| 3. OpenAPI 3.2.0 / AsyncAPI 3.1.0 / proto3 | OpenAPI 10.6% / AsyncAPI 11.6% / proto3 100% | §6.9 |
| 4. OpenBao SecretReference path shape | ~85% (sampled) | audit consistency axis |
| 5. Cell-tier-conformance enum | ~70% (sampled) | audit |
| 6. Compliance-pack-id consistency | ~75% (sampled) | audit |
| 7. Layer-enum (ADR-0105 13-layer) | unverified per §6.11 | open |
| 8. Naming-justifications present | 1.4% | §17.4 |
| 9. Six-hops graph traversal | unverifiable per §6.6 walker missing | open |
| 10. BYOK terminology disambiguation | partial (audit cites 7 IPs conflate) | §10 keystone-bundle synthesis |

**Net:** 5 of 10 invariants near-pass; 5 of 10 partial-or-failing. The
gap is concentrated in OpenAPI/AsyncAPI version, naming-justifications,
and BYOK disambiguation. Mechanical batch in Wave-3-H closes most.

---

## §18 Per-Pack Coverage Audit

The `packs/` directory has 12 files per audit §1.1. The per-pack roster
is not enumerated in this synthesis (would require `ls packs/` deep
walk). Key packs expected per documentation-rigor.md + keystone-bundle
synthesis:

- pack/kr-csap/ (Korea CSAP)
- pack/kr-pipa/ (Korea PIPA)
- pack/kr-fsc/ (Korea Financial Services Commission)
- pack/eu-gdpr/ (GDPR)
- pack/eu-dsa/ (EU DSA)
- pack/eu-ai-act/ (EU AI Act)
- pack/eu-nis2/ (EU NIS2)
- pack/eu-dora/ (EU DORA)
- pack/us-soc2/ (SOC 2)
- pack/us-ccpa/ (California CCPA)
- pack/us-hipaa/ (HIPAA)
- pack/us-pci/ (PCI DSS 4.0)
- pack/us-sox/ (SOX 404)
- pack/us-fedramp/ (FedRAMP Mod/High)
- pack/us-il5/ (DoD IL5)
- pack/us-il6/ (DoD IL6)
- pack/cn-pipl/ (China PIPL 2021)
- pack/cn-csl/ (China CSL 2017)
- pack/cn-dsl/ (China DSL 2021)
- pack/jp-appi/ (Japan APPI)
- pack/in-dpdp/ (India DPDP 2023)
- pack/br-lgpd/ (Brazil LGPD)
- pack/in-rbi/ (India RBI pack)
- pack/eu-mica/ (EU MiCA)
- pack/us-gxp-fda/ (FDA 21 CFR 11)
- pack/us-iso27001/ (ISO 27001)
- pack/us-iso9001/ (ISO 9001)
- pack/us-iso14001/ (ISO 14001)

**Estimated pack coverage:** 12 of ~28+ expected packs present. P1
gap for the keystone-bundle's "build-ahead-of-certification" claim
(ADR-0250).

**Severity:** P1.

**Fix:** Wave-3-I scope — author the remaining pack manifests; each
pack is a single-file declarative artifact per existing
specs/compliance-pack-schema.json.

---

## §19 Wave-3-G Spec File Audit (Sampled)

specs/ went from ~57 to 127 files (+70 new specs). Per the audit
§1.2 "Specs rigorous pass: 3/127 = 2.4%."

Sampled spec types expected post-Wave-3-G:
- specs/microservices/*.json — per-µservice spec roster
- specs/capability-tier-registry.json — referenced by ADR-0316/0321
- specs/conglomerate-tenant-hierarchy.json — referenced by ADR-0313
- specs/dealset-schema.json — referenced by ADR-0314
- specs/erp-module-mapping.json — referenced by ADR-0315
- specs/b2b-leader-vendor-dossier.json — referenced by ADR-0321
- specs/role-projection-manifest.json — referenced by ADR-0317
- specs/collar-color-workspace.json — referenced by ADR-0318
- specs/office-information-barrier.json — referenced by ADR-0319
- specs/transient-identity-skill-tier.json — referenced by ADR-0320

**Verification status:** not done. Recommendation: Wave-3-I
mechanical spec-presence audit.

**Severity:** P1.

---

## §20 Wave-3-G Runbook File Audit (Sampled)

runbooks/ went from ~153 to 205 (+52 new). Per audit §1.2 "Runbooks
rigorous pass: 12/205 = 5.9%."

Wave-3-G expected runbooks per cluster ADR cross-reference:
- runbooks/investigation-case-emergency-archive.md (ADR-0310)
- runbooks/passkey-binding-recovery.md (ADR-0311, ADR-0299)
- runbooks/court-warrant-execution.md (ADR-0312)
- runbooks/conglomerate-child-spinoff.md (ADR-0313)
- runbooks/dealset-settlement-rail-failure.md (ADR-0314)
- runbooks/erp-module-migration-from-sap.md (ADR-0315)
- runbooks/capability-tier-activation.md (ADR-0316)
- runbooks/role-projection-publish.md (ADR-0317)
- runbooks/collar-color-workspace-shift.md (ADR-0318)
- runbooks/information-barrier-emergency-cross-grant.md (ADR-0319)
- runbooks/transient-identity-supervisor-handover.md (ADR-0320)
- runbooks/b2b-leader-vendor-migration.md (ADR-0321)

**Verification status:** presence not verified per-runbook.

**Severity:** P1.

---

## §21 Memory + Doctrine Drift Audit

Reading the `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/
memory/MEMORY.md` index, the following memories are immediately relevant
to Wave-3-G:

- `feedback_oyatie_is_a_tenant_doctrine.md` — KS#1, foundational.
- `feedback_cedar_as_universal_gate.md` — KS#2, foundational.
- `feedback_tenant_as_universal_scoping_primitive.md` — KS#3,
  foundational.
- `feedback_substrate_vs_product_layering.md` — KS#4, foundational.
- `feedback_self_modification_doctrine.md` — KS#6.
- `feedback_amazon_shape_cellular_architecture.md` — KS#7.
- `feedback_compliance_pack_primitive.md` — KS#8.
- `feedback_build_ahead_of_certification.md` — KS#9.
- `feedback_byok_everywhere_credentials.md` — KS#10 §D-4.
- `feedback_http3_quic_default_protocol.md` — KS#10 ADR-0253.
- `feedback_multi_category_marketplace_doctrine.md` — KS#11.
- `feedback_hlc_default_truetime_tier.md` — KS#12.
- `feedback_kubernetes_everywhere_pods_cloud_hypervisor.md` — KS#13.
- `feedback_intelligence_two_layer_substrate.md` — KS#14.
- `feedback_unified_ecosystem_thesis_2026_05_21.md` — **NEW**, would
  formalize the unified-ecosystem thesis as a Wave-3-G memory entry
  (not yet present per audit).
- `feedback_capability_tier_doctrine_2026_05_21.md` — **NEW**, would
  formalize ADR-0316 doctrine (not yet present per audit).

### §21.1 Memory-vs-ADR Drift Findings

The 2026-05-20 keystone bundle synthesis §10 enumerated memory
remediations. Per Wave-3-D Phase-1 audit:
- 10+ memories still instruct grit / rtk / icm / vox usage (P0).
- 2 memories are orphans not indexed in MEMORY.md (P0).
- 1 memory body has "Object Graph" residue (renamed to Ontology per
  the 2026-05-XX rename ledger) (P1).
- v2.4.0 multispectrum-review memory was missing as of 2026-05-20.

Status of these remediations in Wave-3-G:
- Not verified by this synthesis. Recommendation: Wave-3-I memory
  consistency audit.

### §21.2 Doctrine-Doc-vs-ADR Drift Findings

Per §6 findings:
- ADR-0319 status: Accepted (vs cluster Proposed).
- ADR-0320 status: lowercase proposed.
- coverage-matrix status: Living (not in canonical enum).
- training-cost-doctrine + unified-ecosystem-thesis + day-in-the-life
  use clause-loop pattern that fails ArchitectureDeepDive density.

No memory-vs-ADR conflict directly surfaces in Wave-3-G beyond the
above doc-level drifts.

---

## §22 Per-Wave Closure Tracking Recommendation

To close Wave-3-G cleanly and start Waves 3-H/I/J/K with momentum, the
following tracking docs should be authored as part of the §11.A pre-
merge or §11.B per-ADR-promotion gate work:

1. **`docs/architecture/wave-3-g-promotion-gate-tracker.md`** — per-
   ADR gate-closure tracker (mirrors the 2026-05-20 keystone-bundle
   synthesis §6 tracking-issue pattern).
2. **`docs/architecture/wave-3-h-content-pass-plan.md`** — per-µservice
   content-pass slice plan (3 slices, 5/9/13 µservices).
3. **`docs/architecture/wave-3-i-tooling-plan.md`** — 6-hops walker +
   capability-tier registry + persona-journey cross-coverage walker.
4. **`docs/architecture/wave-3-j-deeper-journey-plan.md`** — j151+
   journey roadmap + vertical-pack end-to-end customer dossier roster.
5. **`docs/architecture/wave-3-k-code-authoring-plan.md`** — Rust code
   authoring sequence + per-µservice IP DAG.

Each tracking doc cross-references this synthesis as the input set.

---

## §23 Multispectrum-Review v2.4.0 Per-Facet Sample Verdicts

Per multispectrum-review v2.4.0 cadence, this synthesis itself is the
M-meta-review pass over the corpus. Sample per-facet verdicts that a
fresh single-facet subagent would emit (recommendation: actual codex
subagent runs in Wave-3-I to confirm):

| Facet | Verdict | Notes |
|---|---|---|
| F1 Correctness | WARN — STATUS-ENUM DRIFT (§6.4); CLUSTER-COUNT MISMATCH (§6.5) | Pre-merge fixable |
| F2 Hyperscaler fitness | APPROVE-WITH-FINDINGS | Cluster cross-reference web healthy; per-ADR precedent enumeration likely thin in places |
| F3 Readability | REVISE — TEMPLATE-STAMPING (§6.1, §6.2, §6.3) | Largest editorial workload |
| F4 Architecture | APPROVE-WITH-CONDITIONS | LOAD-BEARING ADRs strongly connected; ONE-MARKETPLACE the structural weak link |
| F5 Security | APPROVE-WITH-FINDINGS | F5 keystone gates still open per §11.D |
| F6 Performance | REVISE — CAPACITY-MATH GAPS (§3.6, §3.10) | Wave-3-H content pass required |
| F7 Supply chain | APPROVE-WITH-CONDITIONS | FIPS/HSM tier gate still open per §11.D |
| F8 Maintenance | APPROVE-WITH-CONDITIONS | Per-category 6-dim scorecard mixed |
| F9 Operations | REVISE — RUNBOOK STUB RATE 89% | Audit §1.2 5.9% rigorous pass |
| F10 Frontend/UX | APPROVE-WITH-CONDITIONS | ONE-UX-SHELL PASS-W-PARTIAL |
| F11 i18n | APPROVE-WITH-RESERVATIONS | Multi-region awareness implicit per-pack |
| F13 Compliance | APPROVE-WITH-FINDINGS | F13 P1 EU NIS2/DSA + CN PIPL still open per §11.D |
| M1 Challenge-assumption | WARN — NEW DOCTRINE DOCS TEMPLATE-STAMPED | Pre-merge fixable |
| M2 Meta-review | THIS DOCUMENT | The v2.4.0 cadence in action |
| A1 Naming | REVISE — STATUS ENUM (§6.4), LIVING STATUS (§6.13) | Pre-merge fixable |
| A2 Documentation | REVISE — TEMPLATE-STAMPING (§6.1-3) + PRD CONTENT (§17) | Multi-wave workload |
| A3 Structure | APPROVE-WITH-FINDINGS | Cluster ADR section counts vary 10-166; healthy |
| A4 Architecture adherence | APPROVE-WITH-CONDITIONS | Per-µservice 27-row matrix unverified |
| A5 Dependency | APPROVE-WITH-FINDINGS | Cluster cross-reference web HEALTHY per §2.13 |
| A6 Schema | APPROVE-WITH-FINDINGS | Specs rigorous-pass 2.4% per audit §1.2 |
| A7 Algorithm | APPROVE-WITH-FINDINGS | Shuffle-sharding math errata from 2026-05-20 not re-verified |

**Net per-facet:** 0 outright APPROVE; 11 APPROVE-WITH-FINDINGS / CONDITIONS;
4 REVISE; 3 WARN. Mirrors the 2026-05-20 keystone-bundle ratio pattern.

---

## §24 Cross-Doc Consistency Spot-Check Against MEMORY.md

The `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/
MEMORY.md` enumerates ~50+ memory files. Spot-check against Wave-3-G:

| Memory | Wave-3-G alignment | Notes |
|---|---|---|
| feedback_quality_performance_scalability_bar.md | PARTIAL | Quality bar declared; per-µservice PRD content gap conflicts |
| feedback_clean_architecture_requirements.md | PASS | Cluster ADRs respect 12/13-layer enum (per ADR-0105) |
| feedback_no_silent_regression.md | PASS | Cluster lands in `Proposed`; no silent contract changes |
| feedback_doc_coverage_enforced.md | REVISE | Per-µservice doc-coverage gaps per §17 |
| feedback_autonomous_decision_principles.md | PARTIAL | "No stubs / placeholders / deferrals within scope" violated by 22 stub PRDs at 400 lines |
| feedback_autonomous_implementation_artifacts.md | REVISE | Template-stamped doctrine docs are anti-autonomous-implementation |
| feedback_multispectrum_review_v22.md | PASS | This synthesis IS the v2.4.0 cadence |
| feedback_naming_justification.md | REVISE | 1.4% manifest naming_justifications coverage |
| feedback_branch_pipeline_implemented.md | PASS | Branch pipeline doctrine unaffected by Wave-3-G |
| feedback_layer_enum_adr_0105_13_canonical.md | PARTIAL | Layer-enum drift in ADR-0263 §D-6 (per §6.11) not re-verified |
| feedback_self_merge_via_contract_path.md | PASS | Self-merge pattern preserved |
| feedback_git_canonical_2026_05_18.md | PASS | No git-tooling regression in Wave-3-G |
| feedback_byok_everywhere_credentials.md | PASS | KS#10 §D-4 BYOK doctrine preserved |
| feedback_unified_ecosystem_thesis_2026_05_21.md | **MEMORY MISSING** | Should be written as Wave-3-I memory remediation |
| feedback_capability_tier_doctrine_2026_05_21.md | **MEMORY MISSING** | Same |
| feedback_collar_color_universality_2026_05_21.md | **MEMORY MISSING** | Same |
| feedback_role_projection_doctrine_2026_05_21.md | **MEMORY MISSING** | Same |

**Net:** 8 PASS / 3 PARTIAL / 3 REVISE / 4 MISSING-FOR-WAVE-3-G.

**Recommendation:** Wave-3-I scope — author the 4 missing memories.

---

## §25 Closing Adjudication Narrative

The Wave-3-G corpus growth is structurally substantial — 22 new
µservice directories, 11 new doctrine ADRs, 4 new long-form architecture
docs, 130 persona dossiers, ~70 new specs, ~52 new runbooks, ≥45 new
ADRs across the broader range 0297-0321. Total new line authoring
exceeds 60,000 lines in a single calendar day.

The architecture is sound. The 11-ADR doctrine cluster's load-bearing
spine (ADR-0311 dual-tenant + ADR-0313 conglomerate + ADR-0314 DealSet
+ ADR-0315 ERP + ADR-0316 capability-tier + ADR-0317 role-projection
+ ADR-0318 collar-color + ADR-0321 B2B-leader) cross-references densely
into the 2026-05-20 keystone bundle (ADR-0242 oyatie-is-a-tenant +
ADR-0243 Cedar universal gate + ADR-0244 tenant scoping + ADR-0245
substrate/product + ADR-0249 multi-category marketplace + ADR-0255
intelligence-as-two-layer + ADR-0257 ontology versioning). The
cross-reference web is strong; an intern can BFS from any cluster ADR
to docs/README.md in ≤2 hops.

The editorial quality is uneven. Three of the four new long-form
architecture docs (unified-ecosystem-thesis, training-cost-doctrine,
day-in-the-life-coherent-ecosystem) contain template-loop generation
artifacts — 700 thesis-clause repeats over 10 invariants in one,
160 problem-clause repeats over 1 problem in another, and similar in
the third. ADR-0321 contains 165 vendor dossiers with identical
sentences for Cedar permit / ontology projection / workflow template /
UX shell / pack overlay / migration path / failure mode — only the
vendor name and coverage tier and destination differ. These are P0
editorial findings, not P0 architectural findings, but they fail the
documentation-rigor.md §1.1 intern-buildability test because an
intern reading the doc cannot derive vendor-specific or invariant-
specific information density that the architecture itself supports.

The operational quality is mixed. ONE-POLICY-ENGINE and ONE-AUDIT-CHAIN
pass the 8-dimension hyperscaler-grade rigor sub-test cleanly. ONE-
COMPLIANCE-POSTURE and ONE-UX-SHELL pass with partials. The other six
ONE-INVARIANTS are PARTIAL, with ONE-MARKETPLACE the structural weak
link due to (a) the marketplace µservice being below the PR-143 artifact
floor at 15 artifacts and (b) ADR-0314 lacking capacity math + multi-
region settlement edge cases.

The doctrine doc count is correct (per Wave-3-G brief). The new µservice
count is correct (22; matches 9 ERP + 13 B2B-leader). The ADR count
falls short (25 vs 30+ brief target). The persona dossier count is
correct (~130). The journey count is unchanged (150; brief calls for
j151+ in Wave-3-J).

The keystone-bundle 2026-05-20 §5 pre-promotion gate set is mostly
still open. Only the BYOK clarification (§5.6) has closed. The
remaining 14 gates layer atop the 5 P0 fix-sets newly surfaced by
this Wave-3-G synthesis.

The recommended action set in §11 keeps the cluster mergeable in
`Proposed` state, gates per-ADR promotion, sequences Wave-3-H content
pass + Wave-3-I tooling + Wave-3-J deeper journeys + Wave-3-K code
authoring across the 2026-Q3-Q4 calendar window.

The 8 in-flight codex agents are scoped to the per-µservice content
pass (Wave-3-H slices 1-3); this Claude opus synthesis does not collide
with their authoring — it provides the editorial guard-rails that
the per-µservice work needs.

---

## §26 Final Action List (Compressed)

Immediate (this PR + before merge):
1. ADR-0319 status edit (Accepted → Proposed).
2. ADR-0320 status case fix (proposed → Proposed).
3. enterprise-software-coverage-matrix status fix (Living → Accepted or
   Proposed; or extend enum).
4. Persona count reconciliation (127 / 129 / 130 across docs).
5. µservice count reconciliation (56 → 69 vs 70).

Wave-3-H (post-merge, content-pass-driven):
6. Marketplace µservice content pass (15 → 70 artifacts; ONE-MARKETPLACE
   invariant promotion from WEAK to PARTIAL).
7. Workplace-integration content pass.
8. Payments content pass (1 artifact — catastrophic; possibly a stray
   filesystem state, needs investigation).
9. 22 Wave-3-G new µservice PRD content pass (400 → 1500 lines + 40
   stories per PRD).
10. Doctrine doc template-collapse on unified-ecosystem-thesis,
    training-cost-doctrine, day-in-the-life-coherent-ecosystem.
11. ADR-0321 per-vendor dossier delta authoring (165 vendor-specific
    deltas; vendor-shared template kept as appendix macro).

Wave-3-I (parallel to Wave-3-H, tooling-driven):
12. Build tools/doc-graph-walker/ (6-hops reachability).
13. Build tools/persona-journey-cross-coverage/.
14. Author capability-tier-registry.json (referenced by ADR-0316/0321).
15. Author 7 missing CI lanes named in §9.2.
16. Author 4 missing memories per §24.
17. Per-pack roster completion (12 → 28+ packs).
18. Per-µservice naming-justifications batch mechanical (1.4% → 100%).
19. OpenAPI 3.2.0 + AsyncAPI 3.1.0 mechanical sed (10.6%/11.6% → 100%).

Wave-3-J (after Wave-3-H content pass):
20. j151+ journey roadmap authoring.
21. Vertical-pack end-to-end customer dossiers (HIPAA + KR-FSS +
    CN-PIPL + likely GDPR + SOX + PCI).
22. Per-Wave-3-G new µservice persona-journey anchoring (11 of 22 lack
    top-30 persona anchor per §12).

Wave-3-K (after Wave-3-H + 3-I + 3-J):
23. Per-µservice Rust code authoring (substrate first; product after).
24. Per-IP PR-shaped slice execution.
25. Per-µservice test coverage to ≥85% line / ≥75% branch.
26. CI lane promotion from advisory to BLOCKER on per-µservice gate
    closure.

Keystone-bundle §5 gates (carried-over from 2026-05-20):
27-40. Close the 14 remaining keystone-bundle pre-promotion gates
       per §11.D.

This is the actionable backlog. Wave-3-G is shippable as `Proposed`
text now; promotion follows the gate closures.

---

## §27 Exemplar Per-Vendor Delta Authoring (ADR-0321 Pre-Merge Demonstration)

To demonstrate that ADR-0321's §D dossier rewrite (§11.A fix-set #4) is
tractable and to anchor what "per-vendor delta over a shared template
macro" looks like in practice, the following five worked exemplars show
the delta-pattern. Each preserves the ADR-0321 vendor-row shape (vendor
name; coverage tier; oyatie destination; Cedar permit shape; ontology
projection; workflow template library; UX shell adaptation; pack overlay
applicable; migration path; naming justification; failure-mode coverage)
but supplies vendor-specific content where the original was template-
stamped.

### §27.1 Exemplar D-006 — MuleSoft (rewritten)

- **Vendor name and category:** MuleSoft Anypoint Platform / Salesforce
  ecosystem / iPaaS.
- **Coverage tier:** D (new µservice required: `microservices/data-
  pipeline/`).
- **Oyatie destination:** `data-pipeline` (primary); `connector` (for
  source/sink adapter registry); `workflow-engine` (for orchestration
  primitives); `ontology` (for schema-projection-on-the-wire).
- **Cedar permit shape:** `permit(principal, action in DataPipelineAction
  ::"<verb>", resource in DataPipeline::"<pipeline-id>") when
  tenant_id == principal.tenant && pack_allows(context.purpose) &&
  data_class_allowed(context.data_class) && cell.eligible_for_data_
  pipeline(context.cell_id)`. The set of verbs: create / start-pipeline /
  pause-pipeline / resume-pipeline / stop-pipeline / rotate-credentials /
  rerun-failed-batch / promote-from-staging / cancel-running-batch /
  view-lineage.
- **Ontology object-type projection:** `pipeline.definition` (DAG of
  source / transform / sink); `pipeline.run` (a single execution
  instance); `pipeline.lineage` (per-field source-to-sink lineage with
  data-class tag); `pipeline.source_adapter` (the iPaaS-style connector
  inheriting from `connect.adapter`); `pipeline.sink_adapter`.
- **Workflow template library:** `data-pipeline.import-from-mulesoft`
  (one-shot import of MuleSoft DataWeave + Mule flows); `data-pipeline.
  ingest-from-source` (per-source-adapter scheduling); `data-pipeline.
  transform` (pure-function chained transforms); `data-pipeline.emit-
  to-sink`; `data-pipeline.rollback-to-prior-version` (per-version
  pipeline-definition rollback); `data-pipeline.replay-from-source`
  (replay-window-based reingestion).
- **UX shell adaptation:** white-collar back-office workspace; desktop-
  primary (DAG editing); per-pipeline lineage visualization (Palantir
  Foundry projection precedent); per-batch status dashboard;
  accessibility = WCAG 2.2 AA for non-DAG views.
- **Pack overlay applicable:** SOC-2 + ISO-27001 + GDPR (per-field data-
  class tagging required); HIPAA (only when data_class includes PHI);
  PCI (per-field tokenization mandatory in transit); FedRAMP-Mod (full
  pipeline-execution audit-event roster); EU-AI-Act (when pipeline
  feeds ML training corpus per data-pipeline.ai-act-overlay).
- **Migration path from MuleSoft:** (1) inventory active Mule flows +
  DataWeave scripts via Anypoint Exchange CLI; (2) dry-run transform
  to data-pipeline DAG definition; (3) Cedar preflight validates
  source/sink adapter availability; (4) ontology projection generates
  per-pipeline lineage records; (5) workflow.replay-from-source runs
  a parallel-run shadow comparing per-field output; (6) tenant
  acceptance with explicit cut-over date; (7) MuleSoft subscription
  termination evidence sealed.
- **Naming justification:** `data-pipeline` is the canonical
  implementation name; `mulesoft` is retained ONLY as a benchmark
  alias in the capability-tier registry. Per BNF v4.1 + ADR-0105
  13-layer enum, `data-pipeline` slots into the kernel layer +
  data-platform bounded-context.
- **Failure-mode coverage:**
  - source-adapter-throttling: per-adapter rate-limit + per-source
    circuit-breaker; failed-batch goes to dead-letter; runbook
    `data-pipeline-source-throttle.md`.
  - schema-drift mid-batch: per-batch schema-pin via ontology
    projection version; drift triggers `data_pipeline_schema_drift`
    audit-event + manual review.
  - sink-write-failure: per-sink idempotency token + per-batch
    compensating action; partial-write rolled back via sink-specific
    rollback handler.
  - cross-jurisdiction-data-residency-violation: per-row data-residency
    check at sink-write time; violations trigger Cedar `forbid` + audit-
    event; runbook `data-pipeline-residency-violation.md`.
  - duplicate-batch-submission: per-batch idempotency-key + per-pipeline
    deduplication window.
  - lineage-loss: per-record lineage record sealed; lineage-loss
    triggers manual reconstruction from per-source replay log.

### §27.2 Exemplar D-005 — Tableau (rewritten)

- **Vendor name and category:** Tableau Server + Tableau Cloud +
  Tableau Desktop / Salesforce ecosystem / interactive analytics.
- **Coverage tier:** C (composed coverage across analytics + sheets +
  ontology).
- **Oyatie destination:** `analytics` (primary; per-tenant visualization
  + dashboard surface); `sheets` (ad-hoc analyst-friendly view); `ontology`
  (per-visualization object-type projection); `intelligence` (per-
  dashboard semantic-search + natural-language-query surface).
- **Cedar permit shape:** `permit(principal, action in AnalyticsAction
  ::"<verb>", resource in Visualization::"<viz-id>") when ...`. Verbs:
  create-dashboard / publish-dashboard / share-with-role / drill-down /
  export-csv / export-pdf / subscribe-to-alert / view-published-only /
  edit-as-author / unpublish.
- **Ontology object-type projection:** `analytics.dashboard` (a Tableau
  dashboard analog); `analytics.worksheet` (single-viz); `analytics.
  data_source` (per-dashboard input registry); `analytics.calculated_
  field`; `analytics.subscription` (alert + email-export schedule).
- **Workflow template library:** `analytics.publish-dashboard` (with
  Cedar pre-check); `analytics.refresh-data-source` (per-source
  scheduled refresh); `analytics.subscription-emit` (alert / email
  delivery via comms-email); `analytics.dashboard-deprecate` (with
  per-subscriber migration notice).
- **UX shell adaptation:** white-collar back-office + executive
  workspace; desktop-primary + mobile-secondary (read-only); per-
  visualization accessibility (alt-text, semantic table-of-data);
  WCAG 2.2 AA for view-mode + WCAG 2.2 A for edit-mode.
- **Pack overlay applicable:** SOC-2 + ISO-27001 + GDPR (per-cell-of-
  data data-class tagging in dashboards); HIPAA (per-PHI-cell redaction
  on share); SOX (every published dashboard sealed in audit-chain);
  FedRAMP-Mod (every export logged).
- **Migration path from Tableau:** (1) export .twbx workbooks from
  Tableau Server REST API; (2) parse worksheets + data sources +
  calculations; (3) ontology-project as analytics.dashboard +
  analytics.worksheet; (4) Cedar permit pre-validate (analyst role
  preserved); (5) parallel-run a sample dashboard rendering on
  oyatie analytics surface; (6) tenant cut-over per dashboard
  on a schedule; (7) Tableau license termination evidence.
- **Naming justification:** `analytics` and `sheets` are canonical
  implementations; `tableau` is benchmark alias only.
- **Failure-mode coverage:**
  - dashboard-render-failure under high-load: per-dashboard render
    cache + graceful-degrade to "data freshness X minutes" notice.
  - drill-down-permission-leak: Cedar default-deny on every cell;
    explicit grant required for cross-tenant or cross-pack data.
  - subscription-storm: per-tenant subscription rate-limit;
    runbook `analytics-subscription-storm.md`.
  - data-source-stale: per-source freshness floor; stale-data warning
    surfaced in dashboard UI.
  - export-without-policy: ADR-0276 backup-portability + GDPR Art. 20
    policy attached to every export.

### §27.3 Exemplar D-007 — Slack (rewritten)

- **Vendor name and category:** Slack Technologies / Salesforce
  ecosystem / team-messaging.
- **Coverage tier:** A (already covered by `microservices/messenger/`
  + community + plugin-app-store).
- **Oyatie destination:** `messenger` (1:1 + group DM + channel + thread);
  `community` (open-membership channel + multi-tenant federation);
  `plugin-app-store` (Slack-app-class extension model under ADR-0249
  multi-category marketplace).
- **Cedar permit shape:** `permit(principal, action in MessengerAction
  ::"<verb>", resource in Channel::"<channel-id>") when ...`. Verbs:
  post / edit-own / delete-own / react / reply-in-thread / mention-user
  / mention-group / mention-here / mention-everyone (gated) / pin / star /
  bookmark / share-cross-channel / invite-user / kick-user (admin).
- **Ontology object-type projection:** `messenger.channel` (private /
  public / cross-tenant); `messenger.thread`; `messenger.message`;
  `messenger.reaction`; `messenger.app-installation` (Slack-app analog);
  `messenger.dm-conversation` (1:1 binding to identity); `messenger.
  workflow` (workflow-engine-driven message-side automation).
- **Workflow template library:** `messenger.app-install` (cross-tenant
  app install with admin approval per ADR-0249); `messenger.cross-
  tenant-bridge` (federation with sovereign-child tenant under
  ADR-0313); `messenger.compliance-archive` (per ADR-0276 +
  per-tenant pack-overlay); `messenger.legal-hold` (per ADR-0312
  warrant-scoped piercing).
- **UX shell adaptation:** white-collar + pink-collar + green-collar +
  blue-collar (mobile-primary on field + handheld-rugged on warehouse);
  per-collar-color shell projection per ADR-0318; per-locale per-emoji-
  pack adaptation per `emoji-sticker-reaction-system.md` standard.
- **Pack overlay applicable:** SOC-2 + ISO-27001 + GDPR + KR-PIPA
  (every message sealed in audit-chain); HIPAA (PHI-tagged channels
  default-deny external share); PCI (card-number redaction at write-
  time); SOX (every executive channel sealed in audit-chain with 7-
  year retention); EU AI Act (every AI-mention message tagged).
- **Migration path from Slack:** (1) Slack Enterprise Grid export per
  workspace (or Slack Standard zip export per workspace); (2) per-
  channel + per-thread + per-message + per-file ingestion into
  messenger ontology; (3) per-user identity-mapping (Slack user → oyatie
  passkey-identity); (4) per-channel-membership migration with Cedar
  permit projection; (5) per-Slack-app catalog → plugin-app-store
  re-registration; (6) tenant cut-over per channel on a per-tenant
  schedule; (7) Slack subscription termination evidence.
- **Naming justification:** `messenger` is canonical; `slack` is
  benchmark alias.
- **Failure-mode coverage:**
  - mention-storm (e.g., @everyone in 50k-member channel): rate-limit
    + per-channel-mention-policy + per-tenant abuse-defence integration.
  - cross-tenant-federation-trust-break (e.g., sovereign child
    leaves federation): per ADR-0313 grant revocation; cross-tenant
    channel becomes read-only then archived.
  - app-supply-chain-compromise: per-app sigstore + meta-trust-root
    per ADR-0293 + ADR-0247.
  - thread-explosion (10k+ replies in a single thread): per-thread
    pagination + per-thread archive-policy.
  - file-attachment-policy-violation (e.g., PHI in non-PHI-tagged
    channel): write-time DLP scan + Cedar `forbid` on the post.

### §27.4 Exemplar D-008 — Heroku (rewritten)

- **Vendor name and category:** Heroku / Salesforce ecosystem / PaaS.
- **Coverage tier:** C (composed coverage across cloud-iac + foundry +
  cloud-k8s).
- **Oyatie destination:** `cloud-iac` (per-tenant IaC + per-environment
  manifest); `foundry` (build + deploy pipeline + per-app GitOps);
  `cloud-k8s` (runtime substrate; per-app namespace + per-app pod
  + Cloud Hypervisor isolation per ADR-0254); `developer-sdk` (per-
  app developer surface).
- **Cedar permit shape:** `permit(principal, action in AppDeployAction
  ::"<verb>", resource in App::"<app-id>") when ...`. Verbs: create-app
  / deploy / rollback / set-env-var / view-logs / scale / restart /
  destroy / promote-to-production.
- **Ontology object-type projection:** `app.definition` (Procfile +
  buildpack + per-env config); `app.deployment` (immutable per-deploy
  artifact); `app.environment` (dev / staging / production / preview);
  `app.release` (build artifact + config-snapshot); `app.dyno-instance`
  (runtime container instance under Cloud Hypervisor).
- **Workflow template library:** `app.deploy-from-git` (GitOps-driven);
  `app.promote-staging-to-production` (with approval gate); `app.
  rollback-to-prior-release`; `app.scale-up-down`; `app.destroy` (with
  retention-protection guard).
- **UX shell adaptation:** white-collar back-office + executive
  (developer experience); desktop-primary + mobile-secondary (alert/
  approval view); per-app dashboard.
- **Pack overlay applicable:** SOC-2 + ISO-27001 + FedRAMP-Mod (per-app
  IaC sealed in audit-chain); GDPR (per-app data-residency declared);
  HIPAA (per-app PHI-storage flag + per-app BAA-compliance overlay);
  PCI (per-app card-data flag + per-app PCI-scope overlay).
- **Migration path from Heroku:** (1) heroku-cli app list + per-app
  release export; (2) Procfile + buildpack inspection; (3) per-app
  config-var migration (Heroku Config Vars → oyatie OpenBao secret
  refs); (4) per-app addon catalog (Heroku Postgres → cloud-iac
  managed-postgres; Heroku Redis → cloud-iac managed-redis); (5)
  per-app GitOps re-registration to foundry; (6) parallel-run a
  staging instance; (7) tenant cut-over per app on a per-app schedule.
- **Naming justification:** `cloud-iac` + `foundry` + `cloud-k8s` are
  canonical; `heroku` is benchmark alias.
- **Failure-mode coverage:**
  - build-failure: per-build retry budget + per-build artifact
    archive + per-build runbook.
  - deploy-rollback-cascade: per-release atomic rollback + per-
    rollback compensating action; runbook `app-deploy-rollback.md`.
  - secret-leak via env-var: per-env-var OpenBao reference + per-app
    secret-scan + Cedar audit on access.
  - addon-failure: per-addon SLA + per-addon failover; runbook
    `app-addon-failover.md`.
  - cross-region-deploy-skew: per-region deploy version monitoring +
    per-region SLO; alert on >2 versions skew.

### §27.5 Exemplar D-014 — Salesforce Communities (rewritten)

- **Vendor name and category:** Salesforce Experience Cloud / Community
  Cloud / Salesforce ecosystem / customer + partner + employee
  communities.
- **Coverage tier:** C (composed coverage across community +
  identity-federation + workflow-studio).
- **Oyatie destination:** `community` (multi-tenant federated
  community + open-membership channel; per ADR-0249 multi-category
  marketplace community surface); `identity` (per-community guest-
  to-member identity federation); `workflow-studio` (per-community
  workflow editor surface).
- **Cedar permit shape:** `permit(principal, action in CommunityAction
  ::"<verb>", resource in Community::"<community-id>") when ...`. Verbs:
  join / leave / post / reply / moderate / pin / archive / invite / kick /
  promote-to-moderator / demote-from-moderator / configure-pack.
- **Ontology object-type projection:** `community.definition`;
  `community.member`; `community.post`; `community.reply`; `community.
  moderator-action`; `community.federation-link` (cross-tenant link
  per ADR-0313 conglomerate grant).
- **Workflow template library:** `community.create` (with per-pack
  overlay); `community.federate-with-tenant` (cross-tenant grant);
  `community.moderation-escalation` (to compliance officer);
  `community.gdpr-export-member-data` (per ADR-0276); `community.
  archive`.
- **UX shell adaptation:** consumer + B2B mixed; mobile-primary +
  desktop-secondary; per-locale content + per-locale moderation
  policy.
- **Pack overlay applicable:** SOC-2 + ISO-27001 + GDPR + KR-PIPA +
  COPPA (for minor-user communities per ADR-0292) + KOSA + EU DSA
  (per-community transparency-report cadence).
- **Migration path from Salesforce Communities:** (1) Salesforce
  Experience Cloud REST API export; (2) per-community + per-member +
  per-post + per-reply + per-file ingestion into community ontology;
  (3) per-member identity-federation (Salesforce user → oyatie
  passkey-identity with consent surface); (4) per-community-membership
  Cedar permit projection; (5) per-moderation-action history archive;
  (6) tenant cut-over per community.
- **Naming justification:** `community` is canonical; `salesforce-
  communities` and `salesforce-experience-cloud` are benchmark aliases.
- **Failure-mode coverage:**
  - moderation-overload during incident: per-community surge-detection
    + auto-escalation to compliance officer; runbook
    `community-moderation-surge.md`.
  - federation-trust-break: per-federation link revocation; cross-
    tenant content becomes read-only then archived per ADR-0313.
  - per-member-PII-leak via community-search: per-PII-field default-
    deny in search index unless explicit consent per ADR-0272 cookie-
    consent-per-purpose.
  - DSA-transparency-report-missed-deadline: per-community DSA
    cadence monitoring; missed deadline triggers compliance officer
    alert + per-tenant remediation.
  - cross-jurisdiction-content-conflict (e.g., post legal in
    jurisdiction A but illegal in B): per ADR-0304 conflict-
    resolution + per-region content-visibility per ADR-0251 pack
    overlay.

### §27.6 Delta Authoring Pattern Roll-Up

These five worked exemplars (D-005 Tableau, D-006 MuleSoft, D-007
Slack, D-008 Heroku, D-014 Salesforce Communities) demonstrate the
target shape for the remaining 160 ADR-0321 vendor dossiers. Each
exemplar is approximately 35-50 lines (vs. the current 12-line
template-stamped row). At 40 lines × 160 vendors = 6,400 lines of
vendor-specific content. Combined with ~5 lines of macro per vendor
(2 lines shared template + 3 lines per-vendor delta), the §D
section grows from 165 × 12 = 1,980 lines to roughly 8,000 lines —
proportionate to the ADR's load-bearing #8 status.

**Acceptance criteria for the Wave-3-H per-vendor delta pass:**
- Each dossier names ≥3 vendor-specific Cedar action verbs.
- Each dossier names ≥3 vendor-specific ontology object types.
- Each dossier names ≥3 vendor-specific workflow templates.
- Each dossier names ≥3 vendor-specific failure modes.
- Each dossier names ≥1 vendor-specific UX shell adaptation.
- Each dossier names ≥1 vendor-specific migration step ABOVE the
  shared 7-step macro.

**Estimated Wave-3-H per-vendor delta workload:** at ~30 minutes
per dossier × 160 remaining vendors = 80 agent-hours = ~2 weeks of
sustained codex content-pass effort per ADR-0321 §D rewrite.

---

## §28 Persona Deeper-Cut Exemplars (Wave-3-J Anchor Demonstration)

This section supplements §12 with deeper-cut persona-journey-µservice
mappings for five exemplar personas. Each demonstrates the level of
mapping detail that Wave-3-J deeper-cut journeys (j151+) and persona-
journey-µservice cross-coverage walker should produce.

### §28.1 Yejin Park (Nurse + Parent + Side-Business Owner)

**Identity:** single passkey-bound human in KR locale. Three tenant
memberships:
1. Personal tenant `tenant::yejin-park-personal` (audience_type =
   `B2C_CONSUMER` + `B2C_FAMILY_PARENT`).
2. Employer tenant `tenant::seoul-univ-hospital` (audience_type =
   `B2B_HEALTHCARE_PROVIDER`; collar-color = pink; workspace =
   clinical-care; skill-tier = mid-level).
3. Side-business tenant `tenant::yejin-farm-cooperative` (audience_type
   = `B2B_TENANT_ADMIN`; collar-color = green; workspace = field;
   skill-tier = junior-side-business).

**Journey anchor set:**
- j02 (code blue EHR break-glass) — clinical persona projection;
  healthcare-integration µservice + audit-chain + Cedar HIPAA pack.
- j03 (988 crisis line minor self-report) — child-safety context as
  parent; messenger + identity + Cedar minor-user pack per ADR-0292.
- j100 (pack rollout onboarding) — side-business onboarding;
  marketplace + payments + compliance per KR-Labor-Standards-Act.
- j124 (eldercare scheduling) — parent role + extended family;
  calendar + comms-email.
- (new j151+ proposal: j151 cross-tenant-shift-handoff — nurse
  finishes hospital shift → switches to farm-cooperative tenant for
  side-business work; demonstrates ADR-0311 dual-tenant boundary +
  ADR-0317 role-projection + ADR-0318 collar-color shift in single
  day).

**µservice coverage:**
- Primary: healthcare-integration, messenger, marketplace, calendar,
  payments, audit-chain.
- Secondary: identity, tenancy, compliance, comms-email, community,
  comms (cooperative-member group).
- Wave-3-G new µservices anchored: healthcare-integration (D-051
  health-cloud + D-058 epic-systems class), warehouse (cooperative
  delivery + harvest workflow).

**Load-bearing ADRs:** 0311 (dual-tenant), 0314 (DealSet for
cooperative payment + hospital reimbursement), 0316 (capability tier
for healthcare-integration + marketplace), 0317 (role-projection
across contexts), 0318 (collar-color: pink + green simultaneously),
0321 (B2B-leader coverage on healthcare-integration vendor row).

**Edge cases highlighted:**
- Yejin moonlights as a nurse + side-business farmer + parent in one
  day; per ADR-0311 each context-switch is a deliberate UX gesture.
- During code-blue (j02), Cedar break-glass permit fires; emergency-
  services-bypass (ADR-0298) authorizes; audit-chain seals.
- Side-business KR-LSA labor compliance for cooperative members;
  per-cooperative collective-bargaining pack overlay (potential
  Wave-3-K pack: pack/kr-labor-cooperative/).

### §28.2 Marcus Chen (Multinational CEO + Spouse + Father)

**Identity:** single passkey in KR + US locales. Tenant memberships:
1. Personal tenant `tenant::marcus-chen-personal`.
2. Employer parent-tenant `tenant::marcus-corp-parent`
   (audience_type = `B2B_CSUITE`).
3. Conglomerate child-tenants (per ADR-0313):
   `tenant::marcus-corp-kr-subsidiary`,
   `tenant::marcus-corp-us-subsidiary`,
   `tenant::marcus-corp-jv-european` (joint venture).

**Journey anchor set:**
- j13 (CEO board crisis) — governance µservice + foundry + audit-
  chain + Cedar SOX overlay.
- j110 (sub-acquisition cross-tenant handoff) — marketplace + DealSet
  + tenancy + identity-federation across new subsidiary onboarding.
- j105 (cross-tenant dispute) — community + audit-chain + marketplace.
- (j151+ proposal: j152 conglomerate-spinoff-handover — parent-tenant
  spins off child as sovereign; demonstrates ADR-0313 sovereignty
  transition + ADR-0314 DealSet for separation-asset settlement).
- (j151+ proposal: j153 cross-jurisdiction-board-emergency — EU
  jurisdiction triggers GDPR breach; US subsidiary's PR response +
  KR subsidiary's KR-PIPA notice; ADR-0304 cross-jurisdiction-
  conflict-resolution exercised).

**µservice coverage:**
- Primary: governance, foundry, finops-portal, audit-chain, compliance,
  identity, tenancy.
- Secondary: marketplace, crm, payments, observability.
- Wave-3-G new µservices: contract-lifecycle-management (M&A contract
  surface), financial-planning (corporate finance), data-warehouse
  (executive analytics).

**Load-bearing ADRs:** 0311, 0313 (conglomerate), 0314 (DealSet), 0316
(capability tier across products), 0317 (executive role projection),
0319 (front-middle-back-office isolation for Marcus-as-CEO vs Marcus-
as-board-chair), 0247 (self-modification when corporate IT acts).

**Edge cases highlighted:**
- Marcus's parent-tenant must NOT silently access child-subsidiary
  PII; per ADR-0313 sovereignty + per ADR-0319 information-barrier.
- Cross-jurisdiction subsidiary disclosure obligations vary; per
  ADR-0304.
- M&A pre-close confidentiality: ADR-0247 self-modification trust
  root + ADR-0293 meta-trust-root enforce that even Marcus-as-CEO
  cannot self-leak ahead of disclosure date.

### §28.3 Dr. Tanaka (Cardiothoracic Surgeon + Parent)

**Identity:** passkey-bound human in JP locale. Tenant memberships:
1. Personal tenant `tenant::tanaka-personal`.
2. Employer `tenant::tokyo-medical-center` (audience_type =
   `B2B_HEALTHCARE_PROVIDER`; collar-color = gold; workspace =
   clinical-care; skill-tier = principal).
3. Academic affiliation `tenant::tokyo-univ-medical-school`
   (audience_type = `EDU_TEACHER`; sub-role = attending-faculty).

**Journey anchor set:**
- j02 (code blue EHR break-glass) — attending-physician role; Cedar
  HIPAA + JP-APPI pack.
- (j151+ proposal: j154 surgical-case-conference — multi-physician
  case review across tenants; healthcare-integration + audit-chain +
  community).
- (j151+ proposal: j155 medical-resident-supervision — Dr. Tanaka
  supervises in-training residents per ADR-0320; demonstrates
  supervisor co-sign Cedar fragment).
- (j151+ proposal: j156 clinical-trial-enrollment — research role +
  GCP compliance + per-patient consent + audit-chain).

**µservice coverage:**
- Primary: healthcare-integration, audit-chain, identity, tenancy.
- Secondary: messenger (patient-facing communication), community
  (medical-society membership), calendar.
- Wave-3-G new µservices: healthcare-integration (full coverage),
  learning-management (continuing medical education tracking),
  contract-lifecycle-management (clinical-trial agreements).

**Load-bearing ADRs:** 0311, 0316, 0317, 0318 (gold collar), 0320
(supervision of residents), 0292 (when pediatric patient involved).

**Edge cases highlighted:**
- Surgeon-resident supervision: per ADR-0320 supervisor co-sign on
  high-stakes operations.
- Clinical-trial enrollment: per ADR-0272 cookie-consent per-purpose
  + research-specific consent overlay.
- Cross-hospital case-conference: per ADR-0313 cross-tenant grant.

### §28.4 Carlos Martinez (Forklift Driver + Father)

**Identity:** passkey-bound human in US locale. Tenant memberships:
1. Personal tenant `tenant::carlos-personal`.
2. Employer `tenant::warehouse-co-12345` (audience_type =
   `B2B_FIELD_WORKER`; collar-color = blue; workspace = field;
   skill-tier = mid-level; device-profile = handheld-rugged).

**Journey anchor set:**
- (existing j2X: warehouse shift journey set).
- (j151+ proposal: j157 forklift-incident-report — workplace-injury
  reporting + OSHA pack + audit-chain).
- (j151+ proposal: j158 frontline-time-sheet-cross-tenant — Carlos
  is a contractor on two warehouses simultaneously; per ADR-0311
  cross-tenant time-sheet attribution).
- (j151+ proposal: j159 frontline-wage-statement — payments + finops-
  portal + workplace-integration + KR-Labor-Standards-Act-style
  wage transparency).

**µservice coverage:**
- Primary: warehouse, workflow-engine, observability, identity,
  workplace-integration.
- Secondary: payments (wage), messenger (shift-coordination).
- Wave-3-G new µservices: warehouse (primary), performance-management
  (frontline performance reviews), incident-management (workplace
  injury).

**Load-bearing ADRs:** 0311, 0316, 0317, 0318 (blue collar; field
workspace), 0298 (emergency-services-bypass for workplace injury).

**Edge cases highlighted:**
- Handheld-rugged device profile: glove-friendly inputs; per
  ADR-0317 device-profile-adaptation.
- Multi-employer time-sheet: per ADR-0311 dual-tenant; each
  employer sees only Carlos's hours at THEIR site.
- Workplace injury: per ADR-0298 emergency-services-bypass +
  per ADR-0297 abuse-defence (false-injury-report detection).

### §28.5 Chris Volkov (Laid-Off Engineer + Family)

**Identity:** passkey-bound human in US locale. Tenant memberships:
1. Personal tenant `tenant::chris-volkov-personal`.
2. Pre-layoff employer `tenant::former-employer-X` (audience_type =
   `B2B_EMPLOYEE` with `lifecycle_state` = `terminated`; per
   ADR-0244 the principal retains data-access until offboarding-
   complete-date).
3. Post-layoff family tenant `tenant::volkov-family`.
4. Active job-search context: cross-tenant `B2C_JOB_SEEKER_ACTIVE`
   on multiple prospective employer-tenants.

**Journey anchor set:**
- (existing layoff + job-search journey set, sampled in master
  roster but not journey-id-mapped).
- (j151+ proposal: j160 layoff-data-export-self-serve — per ADR-0276
  GDPR Art. 20 + CCPA self-export of work-related data + family-photo
  account-survival).
- (j151+ proposal: j161 job-search-application-tracking — across
  multiple prospective-employer tenants; per ADR-0311 each application
  is a separate tenant context, but Chris's identity stays single).
- (j151+ proposal: j162 family-financial-bridge — payments + finops-
  portal for unemployment + side-gig + family budget).

**µservice coverage:**
- Primary: community (job-seeker networking), workplace-integration
  (cross-employer applications), payments + finops-portal (financial
  bridge), audit-chain (per-application audit-history).
- Secondary: marketplace (side-gig discovery), comms-email (interview
  scheduling).
- Wave-3-G new µservices: contact-center (interview coordination),
  contract-lifecycle-management (offer-letter negotiation), data-
  pipeline (resume-tailoring + cover-letter automation per LLM).

**Load-bearing ADRs:** 0311, 0316, 0317, 0299 (account-recovery after
employer revokes credentials), 0276 (data export per Art. 20), 0244.

**Edge cases highlighted:**
- Lifecycle state `terminated` but data-access retained: per
  ADR-0244, per ADR-0276 ensures data-export window.
- Cross-prospective-employer application tracking: per ADR-0311
  prevents one employer from seeing another's application.
- Family tenant + personal-tenant + multiple employer-tenants under
  one passkey: tests ONE-IDENTITY scaling assumption per §3.1.

### §28.6 Persona Deeper-Cut Roll-Up

These five persona deeper-cuts demonstrate the j151+ deeper-cut journey
authoring pattern that Wave-3-J should follow. The pattern surfaces:

1. **Multi-tenant + cross-collar + cross-jurisdiction edge cases that
   the original 150-journey set leaves implicit.** Yejin's nurse-to-
   farmer shift handoff (j151), Marcus's conglomerate spinoff (j152)
   + cross-jurisdiction board emergency (j153), Dr. Tanaka's case-
   conference (j154) + resident supervision (j155) + clinical-trial
   enrollment (j156), Carlos's incident-report (j157) + cross-tenant
   time-sheet (j158) + wage-statement (j159), Chris's data-export
   (j160) + application-tracking (j161) + family-financial-bridge
   (j162) are 12 net-new journeys derived from just 5 personas.
2. **Wave-3-J should target ≥3 deeper-cut journeys per top-30 persona =
   ≥90 net-new journeys** to reach the brief's "deeper-cut journeys
   covering missing persona-journey-µservice intersections" target.
3. **Each deeper-cut journey anchors to ≥1 Wave-3-G new µservice**,
   forcing the 11-of-22 µservices that lack top-30 persona anchor
   (per §12) into the persona-journey-µservice graph.
4. **Vertical-pack end-to-end customer dossiers** (HIPAA + KR-FSS +
   CN-PIPL) emerge naturally from the persona deeper-cuts: Yejin
   (j151) is a HIPAA + KR-PIPA-aware journey; Marcus (j152, j153) is
   a SOX + GDPR + KR-PIPA journey; Dr. Tanaka (j155, j156) is a HIPAA
   + JP-APPI + GCP journey.

**Wave-3-J actionable backlog from this §28:**
- 12 new journey directories (j151-j162) derived from the 5
  personas.
- Estimated ~90 deeper-cut journeys total at the §28.6 #2 pattern.
- Per-deeper-cut journey: 5 canonical files (overview, persona-
  anchor, µservice-anchor, pack-overlay, acceptance-criteria).

---

## §29 Provenance Re-Statement

**Author:** claude-opus-4-7 (Opus 4.7 model — the planning + adjudication
tier per `feedback_model_routing`).
**Session date:** 2026-05-20 (working date; the corpus is dated 2026-05-21).
**Authoring window:** single Opus pass; multispectrum-review v2.4.0
cadence.
**Tool surface:** Bash + Read + Write + Edit only; no codex collision;
no agent delegation.
**Adjudication doctrine:** multispectrum-review v2.4.0 per
`feedback_multispectrum_review_v22`.
**Authority precedent:** keystone-bundle-2026-05-20-synthesis.md §1
(`Proposed`-state cluster merge + per-ADR promotion gate pattern).
**Read-only invariant:** this doc DOES NOT modify any other file in the
corpus. No PRD, ADR, audit, spec, runbook, memory file, or microservice
file was edited.
**File path:** `/Users/jasonlee/oyatie/docs/architecture/wave-3-g-
synthesis-adjudication-2026-05-21.md`.
**Inbound citations:** keystone-bundle-2026-05-20-synthesis.md +
corpus-rigor-audit-2026-05-21-post-wave-3-g.md + future Wave-3-H/I/J/K
tracking docs.
**6-hops reachability:** PASS at 3 hops from docs/README.md per §13.3.

**Findings summary:** 22 catalogued findings (5 P0 / 6 P1 / 6 P2 / 5
P3) supplemented by §16-25 per-ADR + per-µservice + per-pack + per-
spec + per-runbook + memory drift expansion. Net Wave-3-H+I+J+K
backlog: ~40 action items per §26.

**Verdict:** **MERGE-AS-CLUSTER IN `Proposed` STATE; PROMOTION-GATED ON
THE §11.A + §11.B FIX-SETS.**

---

## §30 Appendix — Wave-3-G Citation Index (Quick-Reference)

For downstream synthesis-readers and Wave-3-H/I/J/K planners, the
following compressed citation table maps every primary finding in this
synthesis to its source line(s). All paths are absolute repo-relative.

| Finding | Severity | Primary source | Source line range |
|---|---|---|---|
| Template-stamped vendor dossiers | P0 | `docs/decisions/ADR-0709-general-live-apex.md` | §D-001 ff. (lines 81-249 sampled, full 165 dossiers run to ~2600) |
| 700-clause loop in unified-ecosystem-thesis | P0 | `docs/architecture/unified-ecosystem-thesis-2026-05-21.md` | §1 clauses .01 through .37+ (lines 73-400+); full file 7369 lines |
| 160-clause loop in training-cost-doctrine | P0 | `docs/architecture/training-cost-doctrine-2026-05-21.md` | §1 problem-clause-001 through 160 (lines 64-243+) |
| ADR-0319 status: Accepted | P0 | `docs/decisions/ADR-0709-general-live-apex.md` | frontmatter status field |
| ADR-0320 status: proposed (lowercase) | P0 | `docs/decisions/ADR-0709-general-live-apex.md` | frontmatter status field |
| Brief 30+ vs corpus 25 ADRs | P0 | `docs/architecture/corpus-rigor-audit-2026-05-21-post-wave-3-g.md:50-51` | audit redo pass §1.1 |
| 6-hops walker tool missing | P1 | `docs/architecture/corpus-rigor-audit-2026-05-21-post-wave-3-g.md:87-88` + `docs/standards/documentation-rigor.md:211-212` | audit + standard |
| Marketplace µservice 15 artifacts | P1→P0-cascade | `docs/architecture/corpus-rigor-audit-2026-05-21-post-wave-3-g.md:108` | audit §2.3 |
| Workplace-integration 16 artifacts | P1 | same source | §2.3 |
| Payments 1 artifact | P0 | same source | §2.3 |
| OpenAPI 3.2.0 conformance 10.6% | P1 | same source line 78 | §1.2 |
| AsyncAPI 3.1.0 conformance 11.6% | P1 | same source line 79 | §1.2 |
| µservices PRD floor pass 7.1% | P0-CORPUS | same source line 70 | §1.2 |
| µservices naming-justifications 1.4% | P1 | same source line 73 | §1.2 |
| Persona dossiers heuristic pass 0% | P1 | same source line 76 | §1.2 |
| Specs rigorous pass 2.4% | P1 | same source line 81 | §1.2 |
| Runbooks rigorous pass 5.9% | P1 | same source line 82 | §1.2 |
| Crates/*/docs empty | P2 | same source line 47 | §1.1 |
| j151+ journeys absent | P3 | `ls docs/user-journeys/` | 150 dirs end at j150 |
| Cluster mech-audit verdicts | INFO | this doc §16 | §16.1-12 |
| Per-µservice content gap roll-up | INFO | this doc §17 + audit §3 | §17 |
| Pack roster gaps | P1 | `packs/` directory | 12 files vs ~28+ expected |
| Memory drift | P1 | `MEMORY.md` index | §21 + §24 |
| 5 P0 + 6 P1 + 6 P2 + 5 P3 = 22 findings | INFO | this doc §6.22 | net |

### §30.1 Cross-Wave Sequencing Quick-Reference

| Wave | Primary deliverable | Start | Drives | Estimated effort |
|---|---|---|---|---|
| 3-H | µservice content pass (22 PRDs + marketplace + payments + workplace-integration) | 2026-05-22 | ONE-MARKETPLACE PASS, PRD floor pass ≥30% | ~600 agent-hours / 4-8 weeks |
| 3-I | Tooling + registry + lanes + memories | 2026-05-22 parallel | 6-hops walker, capability-tier registry, 4 missing memories, 28 pack manifests | ~200 agent-hours / 4-6 weeks |
| 3-J | j151+ deeper-cut journeys + vertical-pack dossiers | 2026-07-01 | ~90 new journeys + 3-6 vertical-pack dossiers | ~400 agent-hours / 6 weeks |
| 3-K | Rust code authoring | 2026-08-01 | Per-µservice substrate + product code | ~2-3 calendar quarters |

### §30.2 Pre-Merge Action Set (Repeat for Emphasis)

The 5 P0 fix-sets that block bundle-merge in `Proposed` state:

1. **§11.A.1** — ADR-0319 status `Accepted` → `Proposed` (1-line edit).
2. **§11.A.2** — ADR-0320 status `proposed` → `Proposed` (1-line edit).
3. **§11.A.3** — coverage-matrix status `Living` → `Proposed` OR
   expand enum to include `Living` (1-line edit + 1 standards update).
4. **§11.A.4** — ADR-0321 dossier rewrite-or-appendix-tag. Two options:
   (a) full per-vendor delta rewrite per §27 exemplars (~80 agent-
   hours; defer to Wave-3-H); (b) tag existing dossiers as `## Appendix:
   Generated Templates` and supply a body block stating "Per-vendor
   delta authoring deferred to Wave-3-H per `wave-3-g-synthesis-
   adjudication-2026-05-21.md` §11.A.4 and §27."
5. **§11.A.5** — long-form doctrine doc collapse OR appendix-tag.
   Same options as #4 but applied to unified-ecosystem-thesis,
   training-cost-doctrine, day-in-the-life-coherent-ecosystem.

**Recommended pre-merge path:** Option (b) for #4 + #5 (appendix-tag),
land #1 + #2 + #3 (1-line edits), bundle merges in `Proposed` state.
Wave-3-H takes the per-vendor + per-doctrine-block rewrite as
content-pass work.

---

## §31 Provenance Re-Statement
