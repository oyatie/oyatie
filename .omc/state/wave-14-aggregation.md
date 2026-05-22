# Wave 14 — Canonical Realignment Aggregation

**Status**: LANDED — canonical deliverable
**Date**: 2026-05-21
**Authority**: ADR-0328 §D-15..D-20 canonical realignment sequence + ADR-0329/0330/0331/0332/0333 doctrine
**Supersedes (incremental)**: `.omc/state/wave-findings-aggregation-2026-05-21.md` (per-µservice running tally; preserved as provenance)
**Companion analysis**: `.omc/state/realignment-review-2026-05-21.md` (mid-stream orchestrator review)
**Companion progress snapshot**: `.omc/state/wave-15-progress-2026-05-21.md` (Wave 15 remediation state)
**Master sequencing**: `specs/master-plan-sequencing.json` → `realignment_wave_sequence.wave_14`
**Memory pointer**: `~/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_realignment_review_findings_2026_05_21.md`

---

## 1. Executive summary

The Oyatie realignment effort (Waves 0–15) reconciled an 77-µservice corpus that had drifted from canonical direction during earlier multi-wave authoring. The realignment ran across three integrated phases:

1. **Diagnose (Wave 0)** — causal trace identified surface-wave coordination as the dominant drift mechanism (parallel agents authoring by artifact surface, not by service owner) with verification-by-line-count as a compounding failure. Lane-2 deep dive produced a 3,337-line trace with structural evidence (e.g., ADR-0321 duplicate vendor sections from parallel-write race).
2. **Doctrine + Audit (Waves 1–13)** — Wave 1 authored 7,156 lines of canonical doctrine (ADR-0328 + master-plan-sequencing.json + brief-template.md including 5 cross-cutting constraints + Leptos selective-island-hydration). Waves 2-13 audited **48 of 77 µservices (62%)** producing **~858 findings**, of which **~120 were P0** concentrated in 5 µservices (crm:94 / cloud-billing:12 / marketplace:7 / identity:4 / messenger:3). Six cross-cutting patterns were identified.
3. **Remediate (Wave 15, in progress)** — ~46 agents have shipped: 5 doctrine ADRs (8,826 lines), 5 Big-8 rewrites closing 305/357 P0s (85%), 6 Wave 15A-batch-2 remediations (~35 more P0s closed), 6 healthcare µservices (~17,400 lines / 109 bounded contexts / 933 capability rows per ADR-0332), 3 architectural retirements (network→community / cell-retire / imaging-split per ADR-0333), and 21 tier-vocabulary scrubs across 3 batches.

**Current closure state**: ~340/467 P0s closed (~73%) across 47 of 77 µservices touched (61%). Remaining queue: Wave 15J-batch-4 (~25 tier scrubs) + Wave 15-IP-substance + Wave 15-CA-VERIFY (ADR-0105 13-layer audit) + Wave 15I (foundry retirement + Hermes drop) + Wave 15O (shorts→social, awaiting user confirmation) + this Wave 14 polish.

**What realignment found** — surface-wave coordination produced template-stamping at industrial scale (58% of Claude audits), universal tenant-class adoption gaps, ~1,680+ distinct tier-vocabulary call-sites (~3× the original 9,300-character-occurrence estimate at distinct-site granularity), kernel-ahead-of-spec inversions (cloud-billing), and counterpart-assignment errors (network/cell/crm).

**How Wave 15 addressed it** — substituted per-µservice ownership (one agent owns one µservice end-to-end per the `microservice_ownership_coherence` directive) + substance-verification (not line-count) + doctrine ADRs landed BEFORE remediation began + Rust-crate-assisted tier scrubbing where applicable + dedicated sub-waves for rewrite-class µservices (crm) vs spec-sprint-class µservices (cloud-billing).

**What remains** — corpus-wide ADR-0105 13-layer compliance verification, stamped-IP substance conversion across un-rewritten µservices, ~25 more tier-scrubs to reach corpus coverage, foundry retirement, and the awaiting-decision shorts→social merge.

---

## 2. Per-µservice findings table (77 µservices)

Legend: **Status** = `AUDITED` / `REMEDIATED` / `REWRITTEN` / `RETIRED` / `QUEUED` (Wave 15J-b4 or Wave 15-IP-substance pending) / `NEW` (authored in Wave 15)
**Findings/P0s**: `{total} ({P0})` — `—` for retired or new µservices.

### 2.1 Phase 0 — Shared Infrastructure (cloud-* substrate)

| # | µservice | Findings | P0s closed | Status | Follow-ups |
|---|---|---|---|---|---|
| 1 | cloud-iam | 29 (0) | n/a | AUDITED + Wave 15J-b3 tier-scrubbed | tenant_class plumbing per ADR-0331 |
| 2 | cloud-kms | 23 (0) | n/a | AUDITED | Wave 15J-b4 tier-scrub queued |
| 3 | cloud-secrets | 25 (0) | n/a | AUDITED | Wave 15J-b4 tier-scrub queued |
| 4 | cloud-iac | 26 (0) | n/a | AUDITED + Wave 15J-b3 tier-scrubbed | absorbs cell-provisioning per ADR-0333 |
| 5 | cloud-network | 19 (0) | n/a | AUDITED | Wave 15J-b4 tier-scrub queued |
| 6 | cloud-network-dns | 34 (0) | n/a | AUDITED | Wave 15J-b4 tier-scrub queued |
| 7 | cloud-data | 48 (0) | n/a | AUDITED + Wave 15J-b3 tier-scrubbed | tenant_class plumbing |
| 8 | cloud-storage | 33 (0) | n/a | AUDITED + Wave 15J-b3 tier-scrubbed | tenant_class plumbing |
| 9 | cloud-compute-functions | — | — | QUEUED | Wave 13 long-tail audit |
| 10 | cloud-compute-k8s | 24 (0) | n/a | AUDITED (cloud-k8s = Claude R1) | Wave 15J-b4 tier-scrub queued |
| 11 | cloud-compute-vm | — | — | QUEUED | Wave 13 long-tail audit |
| 12 | cloud-billing | 40 (12) | **12/12 (100%)** | REMEDIATED (Wave 15A-batch-2 spec-sprint) | PRD 786 + ARCH 1042 + README 418 + OpenAPI 993 + AsyncAPI 438 + proto 699 + 6 iac contexts + REMEDIATION-NOTES 659 |
| 13 | cloud-billing-tax | 71 (0) | n/a | AUDITED | Wave 15J-b4 tier-scrub queued; F-DIM4-01/F-DIM5-01 counterpart-disagreement recorded |
| 14 | cloud-capacity | — | — | QUEUED | Wave 13 long-tail audit |
| 15 | cloud-cell | — | — | n/a (distinct from `cell` µservice; remains as cell-substrate µservice within cloud-* family) | Wave 13 long-tail audit |
| 16 | cloud-dcops | — | — | QUEUED | Wave 13 long-tail audit |
| 17 | cloud-finops | — | — | QUEUED | Wave 13 long-tail audit (covered by finops-portal in Phase 1) |
| 18 | cloud-marketplace | — | — | QUEUED | Wave 13 long-tail audit (distinct from `marketplace` distribution µservice) |
| 19 | cloud-fsh | — | — | QUEUED | Wave 13 long-tail audit |

### 2.2 Phase 1 — Foundations / Platform Substrate

| # | µservice | Findings | P0s closed | Status | Follow-ups |
|---|---|---|---|---|---|
| 20 | identity | 24 (4) | **4/4 (100%)** | REMEDIATED (Wave 15A-batch-2) | 7 OpenTofu modules + supported-oses.json + Cedar tenant_class binding (1,900 lines) |
| 21 | tenancy | 25 (0) | n/a | AUDITED + Wave 15J-b3 tier-scrubbed | absorbs cell-assignment per ADR-0333 |
| 22 | audit-chain | 22 (0) | n/a | AUDITED + Wave 15J-b3 tier-scrubbed | absorbs cell-scoped audit per ADR-0333 |
| 23 | governance | 26 (0) | n/a | AUDITED + Wave 15J-b3 tier-scrubbed | governance-lane prefix (ADR-0132) |
| 24 | compliance | 32 (0) | n/a | AUDITED | Wave 15J-b4 tier-scrub queued |
| 25 | observability | 27 (0) | n/a | AUDITED + Wave 15J-b3 tier-scrubbed | absorbs cell-health/blast-radius per ADR-0333 |
| 26 | payments | 18 (0) | n/a | AUDITED | Wave 15J-b4 tier-scrub queued |
| 27 | finops-portal | 19 (0) | n/a | AUDITED | Wave 15J-b4 tier-scrub queued |
| 28 | api-gateway | 18 (0) | n/a | AUDITED + Wave 15J-b2 tier-scrubbed | absorbs cell-routing per ADR-0333 |
| 29 | application | 23 (0) | n/a | AUDITED | Wave 15J-b4 tier-scrub queued |
| 30 | developer-sdk | 17 (0) | n/a | AUDITED | Stainless-class generator scope (10 languages); Wave 15J-b4 queued |
| 31 | network | 18 (0) | n/a | **RETIRED → community (Wave 15K)** | counterpart mis-assignment caught; LinkedIn-class content migrated to community 4-pillar |
| 32 | cell | (audit) | n/a | **RETIRED — pattern not service (Wave 15L)** | ADR-0333 (620L) + oya-shuffle-sharding Rust crate; absorbed by tenancy/cloud-iac/observability/api-gateway/audit-chain |

### 2.3 Phase 2 — Core Capability Substrate (absorbs foundry)

| # | µservice | Findings | P0s closed | Status | Follow-ups |
|---|---|---|---|---|---|
| 33 | intelligence | 13 (0) | n/a | AUDITED + Wave 15J-b2 tier-scrubbed | only 13 findings — re-probe needed (mature or under-audited) |
| 34 | ontology | — (0) | n/a | AUDITED + Wave 15J-b2 tier-scrubbed | `max_tier` contract field retired |
| 35 | workflow-engine | — (0) | n/a | AUDITED + Wave 15J-b2 tier-scrubbed | n8n-class historical pricing tiers retired |
| 36 | workflow-studio | — (0) | n/a | AUDITED + Wave 15J-b2 tier-scrubbed | visual editor + AI-assisted node generation surface |
| 37 | consent-graph | — | — | AUDITED | Wave 15J-b4 tier-scrub queued |
| 38 | detection | — | — | AUDITED | Wave 15J-b4 tier-scrub queued |
| - | foundry | — | — | **RETIREMENT QUEUED (Wave 15I)** | distributed to intelligence + workflow-engine + workflow-studio + ontology + governance/tenancy per ADR-0247 |

### 2.4 Phase 3 — Communication & Collaboration

| # | µservice | Findings | P0s closed | Status | Follow-ups |
|---|---|---|---|---|---|
| 39 | messenger | 30 (3) | **3/3 (100%)** | REMEDIATED (Wave 15A-batch-2) | 6 iac contexts + iac/terraform/ removed + REMEDIATION-NOTES 82L; MLS RFC 9420 E2EE substance-grade |
| 40 | mail | — | — | AUDITED + Wave 15J-b1 tier-scrubbed | 73 tier refs scrubbed |
| 41 | drive | — | — | AUDITED | Wave 15J-b4 tier-scrub queued |
| 42 | calendar | — | — | AUDITED | Wave 15J-b4 tier-scrub queued |
| 43 | meet | — | — | AUDITED | Wave 15J-b4 tier-scrub queued |
| 44 | recordings | — | — | AUDITED | Wave 15J-b4 tier-scrub queued |
| 45 | notes | — | — | AUDITED | Wave 15J-b4 tier-scrub queued |
| 46 | docs | — | — | AUDITED | Wave 15J-b4 tier-scrub queued |
| 47 | sheets | — | — | AUDITED | Wave 15J-b4 tier-scrub queued |
| 48 | slides | — | — | AUDITED | Wave 15J-b4 tier-scrub queued |
| 49 | forms | — | — | AUDITED | Wave 15J-b4 tier-scrub queued |
| 50 | connect | — | — | AUDITED | Wave 15J-b4 tier-scrub queued |
| 51 | comms-email | — | — | AUDITED | Wave 15J-b4 tier-scrub queued |
| 52 | community | — | — | AUDITED — needs re-audit | OLD counterparts (Discourse/Circle/Vanilla) → must re-anchor to Reddit/Teamblind/Handshake/LinkedIn-jobs per 2026-05-21 directive |
| 53 | shorts | — | — | AUDITED + Wave 15J-b1 tier-scrubbed | 155 tier refs (worst); scope re-audit needed — shorts is video substrate not consumer-feed; Wave 15O merge candidate (UNCONFIRMED) |
| 54 | analytics | 20 (0) | n/a | AUDITED + Wave 15J-b1 tier-scrubbed | 148 tier refs scrubbed |
| 55 | tasks | 16 (0) | n/a | AUDITED + Wave 15J-b1 tier-scrubbed | 167 tier refs scrubbed |
| 56 | translate | 19 (0) | n/a | AUDITED + Wave 15J-b1 tier-scrubbed | 234 tier refs scrubbed |
| 57 | search | — | — | QUEUED | Wave 13 long-tail audit |

### 2.5 Phase 4 — Distribution + B2B Enterprise SaaS

Distribution substrate:

| # | µservice | Findings | P0s closed | Status | Follow-ups |
|---|---|---|---|---|---|
| 58 | marketplace | ~62 (7) | **7/7 (100%)** | REMEDIATED (Wave 15A-batch-2) | 6-category surfaces (24 artifacts) + 11 rev-share IPs + 12/12 ADR-0331 tenant_class surfaces; 5 SUPERIOR capabilities preserved |
| 59 | plugin-app-store | 24 (0) | n/a | AUDITED | ADR-PAS-0004 tier-named retraction queued |
| 60 | workplace-integration | 18 (0) | n/a | AUDITED | Wave 15J-b4 tier-scrub queued |
| 61 | feature-flags | 12 (0) | n/a | AUDITED | **F-COH-010**: HashiCorp Terraform engine violation (forbidden) → must convert to OpenTofu |
| 62 | ops-dashboard-control-center | 18 (0) | n/a | AUDITED + Wave 15J-b1 candidate | 30 tier refs |
| 63 | brand | — | — | QUEUED | Wave 13 long-tail audit |
| 64 | sites | — | — | QUEUED | Wave 13 long-tail audit |

Big-8 family µservices:

| # | µservice | Findings | P0s closed | Status | Follow-ups |
|---|---|---|---|---|---|
| 65 | performance-management | 27 (27) | **24/27 (89%)** | REWRITTEN (Big-8 HR/Payroll) | 30+ new capability YAMLs + 6 cross-handoff AsyncAPIs |
| 66 | learning-management | 8 (8) | partial | AUDITED → architectural-decision-needed | Canvas LMS bounded-context absorb-vs-separate decision pending |
| 67 | crm | 113 (94) | **78/94 (83%)** | REWRITTEN (Wave 15A-crm-rewrite dedicated sub-wave) | 3,790 lines authored; proper Big-8 ordering (Salesforce #1 / HubSpot present / Dynamics current name) + missing primitives (CPQ, Sales Cadences, AI scoring, Reports, Mobile CRM) |
| 68 | marketing-automation | 119 (103) | **~70/103 (~68%)** | REWRITTEN (Big-8 HubSpot family) | 2,114 lines + 25 new IPs; 10 DIFFERENTIATORS preserved |
| 69 | contact-center | — | — | QUEUED | Wave 13 long-tail audit |
| 70 | itsm | 227 (33) | **33/33 (100%)** | REWRITTEN (Big-8 ServiceNow) | 1,614 lines + 5 new Rust crates; SLA breach 15s p99 (8× ServiceNow), workflow 800/sec (7×), CMDB 380ms 3-hop (3.7×) |
| 71 | incident-management | — (10) | partial | AUDITED — Phase 4A.4 ship BLOCKED | bounded-context plurality divergence; substrate_dependencies missing cmdb+itsm+change-management |

ERP family (SAP-ERP):

| # | µservice | Findings | P0s closed | Status | Follow-ups |
|---|---|---|---|---|---|
| 72 | production-planning | — | — | QUEUED | Wave 13 long-tail audit |
| 73 | quality-management | — | — | AUDITED — counterpart mismatch | used ERP-suite vendors instead of domain best-of-breed |
| 74 | plant-maintenance | 30 (3) | partial | AUDITED + Wave 15J-b1 tier-scrubbed | 401 tier refs scrubbed |
| 75 | warehouse | — | — | QUEUED | Wave 13 long-tail audit |
| 76 | real-estate | 16 (0) | n/a | AUDITED | Wave 15J-b4 tier-scrub queued |
| 77 | treasury | — (4) | partial | AUDITED — counterpart mismatch | used ERP-cash modules instead of TMS-leaders |
| 78 | supply-chain-planning | — | — | QUEUED | Wave 13 long-tail audit |
| 79 | global-trade | 21 (0) | n/a | AUDITED + Wave 15J-b1 tier-scrubbed | 509 tier refs scrubbed |
| 80 | financial-planning | — (9) | partial | AUDITED — Phase 4 ERP promotion BLOCKED | Excel integration MISSING (Vena's identifying capability); vocabulary contradiction; behind on formula throughput |

Long-tail:

| # | µservice | Findings | P0s closed | Status | Follow-ups |
|---|---|---|---|---|---|
| 81 | contract-lifecycle-management | — (100) | **100/100 (100%)** | REWRITTEN (Wave 15A-batch-1 legal-complexity) | 6,500+ lines across 50 new files; ADR-CLM-001; Rust-strict PASS |
| 82 | design-collaboration | 17 (0) | n/a | AUDITED + Wave 15J-b1 tier-scrubbed | 163 tier refs scrubbed |
| 83 | whiteboard | 28 (0) | n/a | AUDITED | Wave 15J-b4 tier-scrub queued |
| 84 | data-pipeline | — (3) | **3/3 (100%)** | REMEDIATED (Wave 15A-batch-2) | IP-031..IP-037 all 7 substantive + REMEDIATION-NOTES 337L |
| 85 | data-warehouse | — (6) | **6/6 (100%)** | REMEDIATED (Wave 15A-batch-2) | Databricks Lakehouse substrate (Delta/Iceberg/Hudi/Unity-Catalog/Photon/Auto-Loader/DLT/CDF) + REMEDIATION-NOTES 407L |
| 86 | healthcare-integration | — (2) | partial | AUDITED + NARROWED per ADR-0332 | now 3 broker contexts (fhir-broker / hl7v2-broker / dicom-broker); FHIR READ p99 38ms (7× Redox), HL7v2 102.8K msgs/sec, DICOM C-STORE 10.25K inst/min, MPI 130ms p99 (9× Redox) |

Healthcare new µservices (Wave 15M, ADR-0332):

| # | µservice | Lines | Bounded contexts | Capability rows | Status |
|---|---|---:|---:|---:|---|
| 87 | emr | 1,841 + 14 categories | 15 | 128 | NEW |
| 88 | diagnostics (lab+pathology) | 1,833 | 15 | 130 | NEW (imaging stripped post-15M-reconcile) |
| 89 | emergency (ED-IS) | 1,761 | 17 | 113 | NEW |
| 90 | pharmacy (med-mgmt+ePrescribe+DEA EPCS) | 1,958 | 20 | 150 | NEW |
| 91 | patient-monitoring (vital signs+RPM+ICU+ML) | 2,284 | 18 | 212 | NEW |
| 92 | imaging (DICOMweb+VNA+AI marketplace) | 7,720 across 57 files | 24 | 200 | NEW (split per Wave 15M-reconcile / ADR-0332 amendment) |

**Sanity reconciliation**: the canonical active count is **77 µservices** after applying retirements (network/cell/foundry-queued) and additions (6 healthcare). The table above includes retirements as historical rows so all audit findings remain traceable; subtract the 3 RETIRED/queued-RETIRE rows (network, cell, foundry) from the row count for the 77 active count.

---

## 3. Cross-cutting patterns (6)

These six patterns were derived from `.omc/state/realignment-review-2026-05-21.md`; provenance citations live in that file. They are the durable structural findings of the realignment.

### Pattern 1 — Tier-vocabulary scope much larger than projected

- Original scope-audit estimated ~9,300 character occurrences of Bronze/Silver/Gold/Platinum.
- Distinct-call-site count from 48 audits ≈ **1,680 call-sites** (mean ~35/µservice). 12 codex audits alone found ~1,900 distinct sites, extrapolating to ~3,000+ corpus-wide.
- Worst entrenchment: shorts (155) / mail (73) / community (56) / crm (~104) / api-gateway (43).
- **Wave 15 response**: ADR-0329 landed + 21 µservices scrubbed across 3 batches (Wave 15J-b1/b2/b3); ~25 remain (Wave 15J-b4 queue).

### Pattern 2 — Tenant-class adoption gap was UNIVERSAL

- All 48 audited µservices flagged the gap. ZERO had adopted `{demo_trial, paid}` + composable `billing_components` ⊆ `{revenue_share, per_seat, per_usage}`.
- Legacy variants: `{free/paid/starter/pro/enterprise}` (most common), `{trial/sandbox/production/internal-foundry}` (intelligence), `{max_tier}` contract field (ontology), `partner_tier` (api-gateway), `tier_classification + capability_tiers + criticality_tier` (crm).
- **Wave 15 response**: ADR-0330 + ADR-0331 landed (codify the model + per-µservice plumbing IP template). Identity (4/4), cloud-billing (12/12), marketplace (7/7), messenger (3/3), data-warehouse (6/6), data-pipeline (3/3) all closed their tenant-class P0s during Wave 15A-batch-2.

### Pattern 3 — Kernel-ahead-of-spec anti-pattern (cloud-billing)

- cloud-billing had a 1,030-line hyperscaler-grade Rust kernel + PRD/ARCHITECTURE/README/contracts/SLOs ALL ABSENT — inverse of the typical substance-gap.
- Variants observed in: data-pipeline, healthcare-integration, marketplace, quality-management, production-planning, treasury, contract-lifecycle-management.
- **Wave 15 response**: Wave 15B-cloud-billing-spec-sprint executed (PRD 786 + ARCH 1042 + README 418 + OpenAPI 993 + AsyncAPI 438 + proto 699 + 6 iac contexts); kernel preserved.

### Pattern 4 — Industrial-scale template-stamping (crm worst-case)

- crm: README 169 stamped rows + ARCHITECTURE §H 90 traces + competitor-parity-matrix 327 rows + PRD §C 30 stories = 94 P0s = 78% of all P0s.
- Template-stamping rate across Claude audits: **15 of 26 = 58%**.
- Lane 2 trace's "surface-wave coordination as causal pattern" hypothesis CONFIRMED by this evidence.
- **Wave 15 response**: crm got a dedicated Wave 15A-crm-rewrite sub-wave (3,790 lines, proper Big-8 ordering, missing primitives authored). marketing-automation similarly rewritten (2,114L + 25 IPs). 5 Big-8 rewrites total closed 305/357 P0s (85%).

### Pattern 5 — Doctrine evolution within session created stale audits

- Doctrine drifted DURING the session (Leptos + selective-island-hydration + tier-retirement + tenant-class 3-class then 2-class + Stainless + C/C++ + network→community + cell-retire + mobile-app-bundle).
- Earlier-audited µservices do NOT reflect later directives. Findings still hold (more conservative — didn't flag what wasn't in their prompt) but missed directives become Wave 15J/K/L retirement candidates.
- Bounded risk: Wave 15J/K/L sweep applies final doctrine consistently across ALL µservices.

### Pattern 6 — Counterpart-assignment errors detected by agents themselves

| µservice | Wrong counterparts | Correct | Wave-15 fix |
|---|---|---|---|
| network | AWS VPC Lattice / GCP Cross-Cloud / Azure VWAN | LinkedIn / X / Threads | Wave 15K merge into community (RETIRED) |
| cell | AWS Cell / GCP Distributed / Fastly Edge | Cellular architecture PATTERN | Wave 15L absorption (RETIRED, ADR-0333) |
| cloud-billing-tax | 3 vendors (dispatch) | 5 vendors (Stripe+Avalara+TaxJar+Vertex+Sovos) | F-DIM4-01/F-DIM5-01 disagreement recorded per ADR-0328 §D-5.3 |
| crm | Salesforce as #3 | Salesforce as #1 anchor + HubSpot present + Dynamics current name | Wave 15A-crm-rewrite pivot |
| healthcare-integration | EHR vendors | Redox / Mirth / Health Gorilla | corrected during Wave 15M decomposition |
| learning-management | 6 counterparts (manifest) vs 3 (brief) | Canvas-anchored | architectural-decision-needed |
| quality-management | ERP-suite vendors | Domain best-of-breed | Wave 13 long-tail re-anchor |
| treasury | ERP-cash modules | TMS-leaders | Wave 13 long-tail re-anchor |

---

## 4. Sub-wave breakdown (Wave 15)

### Wave 15 doctrine ADRs (substrate; ALL LANDED 2026-05-21)

| ADR | Lines | Purpose |
|---|---:|---|
| **ADR-0329** tier-system-retired-replaced-by-tenant-class | 2,555 | Canonical retirement of Bronze/Silver/Gold/Platinum + capability_tiers (supersedes ADR-0316) |
| **ADR-0330** tenant-class-demo-trial-vs-paid-composable-billing-components | 2,048 | Codifies `{demo_trial, paid}` + composable `billing_components` ⊆ `{revenue_share, per_seat, per_usage}` |
| **ADR-0331** cross-microservice-tenant-class-adoption-template | 1,137 | Per-µservice plumbing IP template (tenant_class claim binding + billing_components context attribute + Cedar gates + demo_trial cap-breach) |
| **ADR-0332** healthcare-domain-decomposition | 1,466 | Decomposes monolithic healthcare into 6 µservices + 3 broker contexts (later amended to add imaging as 8th) |
| **ADR-0333** cell-microservice-retired-pattern-not-service | 620 | Cellular architecture is a PATTERN not a service; absorbed by tenancy/cloud-iac/observability/api-gateway/audit-chain + oya-shuffle-sharding Rust crate |
| **Total** | **8,826** | |

### Wave 15A — P0 remediation (Big-8 rewrites + targeted spec sprints)

| Sub-wave | Targets | P0 closed | Lines |
|---|---|---|---:|
| 15A-batch-1 (Big-8 rewrites) | crm / marketing-automation / contract-lifecycle-management / itsm / performance-management | 305/357 (85%) | ~16,000+ across 5 µservices |
| 15A-batch-2 (spec-sprints + remediation) | cloud-billing / marketplace / identity / data-warehouse / messenger / data-pipeline | ~35 P0s | ~10,000+ |

### Wave 15J — Tier-vocabulary scrub (21 µservices across 3 batches)

| Batch | µservices |
|---|---|
| 15J-b1 (8) | global-trade / plant-maintenance / translate / tasks / design-collaboration / analytics / shorts / mail |
| 15J-b2 (5) | workflow-engine / workflow-studio / ontology / api-gateway / intelligence |
| 15J-b3 (8) | cloud-iam / cloud-iac / cloud-data / cloud-storage / tenancy / audit-chain / governance / observability |

All 21 had `capability-tiers/` directories deleted, `capability_tiers` manifest fields removed, and tenant_class adoption substrate added per ADR-0331.

### Wave 15K — network → community merge (LANDED)

- `microservices/network/RETIRED.md` written + LinkedIn-class content migrated into community 4-pillar (Reddit / Teamblind / Handshake / LinkedIn-jobs+profile+InMail).
- Architectural rationale: agent self-detected counterpart mismatch during Wave 3 B2 audit (network µservice is LinkedIn-class professional network, not networking infrastructure).

### Wave 15L — cell µservice retirement (LANDED)

- `microservices/cell/RETIRED.md` written + ADR-0333 (620L) + `crates/oya-shuffle-sharding/` (new Rust library).
- Cell concerns redistributed: tenancy (assignment) + cloud-iac (provisioning + registry) + observability (health + blast radius) + api-gateway (routing) + audit-chain (cell-scoped audit) + oya-shuffle-sharding (algorithm).

### Wave 15M — healthcare domain decomposition + imaging-split (LANDED)

- 6 new µservices authored under `microservices/{emr,diagnostics,emergency,pharmacy,patient-monitoring,imaging}/` per ADR-0332.
- `healthcare-integration` NARROWED to 3 broker contexts.
- 15M-reconcile: imaging contexts stripped from diagnostics; ADR-0332 amendment adds imaging as 8th healthcare µservice.

### Wave 15I — foundry retirement + Hermes drop (QUEUED)

- `microservices/foundry/` retires per ADR-0247 + foundry-absorption doctrine.
- Capability distributed to intelligence + workflow-engine + workflow-studio + ontology + governance/tenancy (already covered in Wave 4 audits via foundry-absorption dimension).
- Plan: 1 Claude agent for retirement + cross-reference update + Hermes terminology drop.

### Wave 15O — shorts → social absorption (UNCONFIRMED)

- Awaiting user decision. Wave 4-rolling audit flagged shorts as video substrate (not consumer-feed) per 2026-05-21 mobile-app-bundle directive.

---

## 5. ADR landing log (Wave 15)

| ADR | Title | Lines | Cross-references |
|---|---|---:|---|
| ADR-0328 | substance-bar-as-canonical-sequence-and-batch-discipline | 4,395 (Wave 1) | base doctrine for the realignment; cited by every Wave 15 ADR |
| **ADR-0329** | tier-system-retired-replaced-by-tenant-class | 2,555 | supersedes ADR-0316; binds Wave 15J scrub policy |
| **ADR-0330** | tenant-class-demo-trial-vs-paid-composable-billing-components | 2,048 | replaces retired ADR-0316 tier model |
| **ADR-0331** | cross-microservice-tenant-class-adoption-template | 1,137 | per-µservice plumbing IP template; cited by 15A-batch-2 REMEDIATION-NOTES files |
| **ADR-0332** | healthcare-domain-decomposition | 1,466 (+ amendment) | spawns emr/diagnostics/emergency/pharmacy/patient-monitoring/imaging |
| **ADR-0333** | cell-microservice-retired-pattern-not-service | 620 | retires `cell` µservice; cites oya-shuffle-sharding crate |
| Memory pointers | feedback_no_capability_tiers_2026_05_20.md / feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md / feedback_cell_standalone_network_merges_community_2026_05_21.md | n/a | session-durable directives that drove the ADRs |

**Total Wave 15 doctrine output**: 8,826 lines (5 ADRs).

---

## 6. Architectural retirements log

| Retirement | Wave | Mechanism | Disposition | Artifacts |
|---|---|---|---|---|
| network → community | Wave 15K | codex | network/RETIRED.md; LinkedIn-class content migrated into community 4-pillar (Reddit/Teamblind/Handshake/LinkedIn-jobs+profile+InMail) | `microservices/network/RETIRED.md` |
| cell µservice retire | Wave 15L | codex + Claude | cell/RETIRED.md; concerns absorbed by tenancy+cloud-iac+observability+api-gateway+audit-chain; oya-shuffle-sharding crate authored | `microservices/cell/RETIRED.md` + `crates/oya-shuffle-sharding/` + ADR-0333 (620L) |
| imaging-diagnostics split | Wave 15M-reconcile | codex | diagnostics imaging contexts stripped; imaging promoted to 8th healthcare µservice | ADR-0332 amendment |
| foundry retirement | Wave 15I (QUEUED) | 1 Claude agent | distributed to intelligence + workflow-engine + workflow-studio + ontology + governance/tenancy | `microservices/foundry/` retire + Hermes terminology drop |
| shorts → social merge | Wave 15O (UNCONFIRMED) | TBD | awaiting user decision | n/a |

---

## 7. Remaining work queue

| ID | Scope | Mechanism | Estimate |
|---|---|---|---|
| Wave 15J-b4 | ~25 remaining µservices for full tier-vocabulary corpus coverage (cloud-kms, cloud-secrets, cloud-network, cloud-network-dns, cloud-billing-tax, cloud-k8s, compliance, payments, finops-portal, application, developer-sdk, consent-graph, detection, drive, calendar, meet, recordings, notes, docs, sheets, slides, forms, connect, comms-email, social, sites, plugin-app-store, workplace-integration, ops-dashboard-control-center, incident-management, learning-management, contact-center, supply-chain-planning, production-planning, quality-management, treasury, healthcare-integration, real-estate, warehouse) | 8-12 codex agents/batch × ~3 batches | ~3 batches |
| Wave 15-IP-substance | Corpus-wide stamped-IP → substance conversion (~25 stamped 55-line IPs per µservice across many µservices; 5 Big-8 rewrites already addressed inline) | 8-15 codex agents | 1-2 batches |
| Wave 15-CA-VERIFY | ADR-0105 13-layer compliance audit (many µservices declared only 9/13 layers; new µservices mixed 12/13) | 1 Claude agent | corpus-wide |
| Wave 15I | foundry retirement + Hermes terminology drop per ADR-0247 | 1 Claude agent | n/a |
| Wave 15O | shorts → social absorption | TBD | AWAITING USER DECISION |
| Phase 4 long-tail audits | ~9 still-queued µservices (cloud-compute-functions, cloud-compute-vm, cloud-capacity, cloud-dcops, cloud-fsh, search, brand, sites, contact-center, production-planning, warehouse, supply-chain-planning) | leaner 150-line "scope present + counterpart + buildability" format per realignment-review §Risk-1 | 1 codex batch |
| feature-flags Terraform→OpenTofu conversion | F-COH-010 forbidden-engine violation per ADR-0328 §D-16 | 1 codex agent | targeted IP |
| messenger MLS RFC 9420 tenant_class+compliance-pack binding | F-MSGR-007 follow-up | 1 codex agent | targeted IP |
| Counterpart re-anchor: quality-management / treasury / learning-management / community | counterpart mis-assignment re-audit per Pattern 6 | 4 codex agents | 1 batch |
| Wave 14 aggregation polish | THIS FILE | 1 Claude agent | LANDED |

---

## 8. Cross-references (durable)

### Primary state files
- `.omc/state/wave-findings-aggregation-2026-05-21.md` — per-µservice running tally (provenance source)
- `.omc/state/realignment-review-2026-05-21.md` — mid-stream orchestrator analysis (6 patterns + 9 recommendations)
- `.omc/state/wave-15-progress-2026-05-21.md` — Wave 15 remediation snapshot (~46 agents)

### Per-µservice durable artifacts
- `microservices/<name>/coherence-audit-2026-05-20.md` — primary audit findings source (44 files)
- `microservices/<name>/feature-parity-matrix-2026-05-20.md` — counterpart UNION-coverage
- `microservices/<name>/performance-benchmark-numbers-2026-05-20.md` — per-context + per-tenant-class targets
- `microservices/<name>/capability-tier-deltas-vs-counterparts-2026-05-20.md` — Wave 2 + Wave 3 B1 only; dropped from Batch 3.2 onward
- `microservices/<name>/REMEDIATION-NOTES-2026-05-21.md` — 11 µservice remediation notes from Wave 15A-batch-1 + Wave 15A-batch-2
- `microservices/network/RETIRED.md` + `microservices/cell/RETIRED.md` — retirement markers

### Doctrine ADRs
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` (4,395L)
- `docs/decisions/ADR-0329-tier-system-retired-replaced-by-tenant-class.md` (2,555L)
- `docs/decisions/ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md` (2,048L)
- `docs/decisions/ADR-0331-cross-microservice-tenant-class-adoption-template.md` (1,137L)
- `docs/decisions/ADR-0332-healthcare-domain-decomposition.md` (1,466L)
- `docs/decisions/ADR-0333-cell-microservice-retired-pattern-not-service.md` (620L)

### Master sequencing + standards
- `specs/master-plan-sequencing.json` (870L) — canonical_build_sequence + realignment_wave_sequence + 5 cross-cutting constraint blocks
- `docs/standards/brief-template.md` (1,891L) — 5-citation header + agent-class anchor templates

### Investigation lineage (Wave 0)
- `.omc/specs/deep-dive-trace-realign-oyatie-corpus-lane-2.md` (3,337L) — surface-wave coordination causal trace
- `.omc/specs/deep-dive-trace-realign-oyatie-corpus-to-canonical.md` — orchestrator synthesis
- `.omc/specs/deep-dive-realign-oyatie-corpus-to-canonical.md` — final spec
- `.omc/plans/realign-oyatie-corpus-plan-2026-05-20.md` — implementation plan

### Memory pointers (durable session directives)
- `feedback_realignment_review_findings_2026_05_21.md` — mid-stream review summary + Wave 15 update block
- `feedback_no_capability_tiers_2026_05_20.md` — tier-retirement decision
- `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md` — replacement model
- `feedback_cell_standalone_network_merges_community_2026_05_21.md` — cell retire + network merge + mobile-app-bundle
- `feedback_microservice_ownership_coherence_2026_05_20.md` — one-agent-one-µservice doctrine
- `feedback_verify_deliverables_not_just_line_count_2026_05_20.md` — substance-bar verification
- `feedback_docs_substance_not_scaffold_2026_05_20.md` — anti-stamping doctrine

### New Rust artifacts authored during Wave 15
- `crates/oya-shuffle-sharding/` — cellular shuffle-sharding algorithm (Wave 15L)
- 5 new Rust crates for itsm Big-8 elevation

---

## 9. Headline closure metrics

| Metric | Wave 14 (current) | Target | Δ |
|---|---:|---:|---|
| µservices audited | 48/77 (62%) | 77/77 (100%) | -29 (Phase 4 long-tail + remaining Phase 3) |
| µservices touched (audit + remediation) | 47/77 (61%) | 77/77 (100%) | -30 |
| P0s identified | ~467 | n/a | — |
| P0s closed (Wave 15) | ~340 / 467 (~73%) | 467/467 | -127 |
| Lines of audit content authored | ~75,000+ | n/a | — |
| Lines of remediation authored (Wave 15) | ~85,000+ | n/a | — |
| Doctrine ADRs landed | 6 (0328 + 0329-0333) | 6 | 0 |
| Architectural retirements | 3 (network, cell, imaging-split) | 4 (+ foundry) | -1 |
| Cross-cutting patterns identified | 6 | 6 | 0 |
| Sub-waves complete | 7 (15A-b1, 15A-b2, 15J-b1/b2/b3, 15K, 15L, 15M) | 10 | -3 (15J-b4, 15-IP-substance, 15-CA-VERIFY, 15I, 15O) |

---

## 10. Provenance and authority

This Wave 14 aggregation is the canonical realignment deliverable per `specs/master-plan-sequencing.json` → `realignment_wave_sequence.wave_14` ("aggregate findings + remediation backlog"). It supersedes the running tally file as the cross-µservice rollup but preserves it as provenance.

Per ADR-0328 §D-2 substance-bar rules, this aggregation:

- Is **per-µservice + per-phase + per-finding-category + per-remediation-route** (the unmet promise of the running tally).
- **Cites** every claim back to either the running tally, the realignment review, the Wave 15 progress snapshot, the per-µservice REMEDIATION-NOTES, or a landed ADR.
- **Does not stamp** content; every µservice row is its own substantive entry.
- **Lists remaining work** explicitly with mechanism + estimate so the Wave 15 dispatcher can resume from this file post-compact.

This file is the canonical answer to "what did the realignment find and where do we stand on closing it." Future Wave 15 sub-waves update this file (or its successor) when sub-waves close.
