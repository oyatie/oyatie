# Wave Findings Aggregation — running tally

Updated 2026-05-21. Grows as audits complete.

Per-µservice findings are durable at `microservices/<name>/coherence-audit-2026-05-20.md` (orchestrator HTML comment + §4 findings table). This file is the cross-µservice rollup that will become the canonical Wave 14 aggregation.

## Wave 0 — Pre-realignment investigation (2026-05-20 evening)

### Purpose
Diagnose why the Oyatie corpus had drifted from canonical direction during earlier multi-wave authoring. Three causal lanes investigated in parallel.

### Lane 1 (codex tracer): brief-scope cause
- **Status**: codex died without producing deliverable; agent crashed mid-run
- **Inferred from synthesis**: briefs lacked per-µservice anchor citations + clear constraint enumeration → agents authored without doctrine grounding

### Lane 2 (codex tracer): coordination/concurrency/ownership cause — COMPLETE
- **Artifact**: `/Users/jasonlee/oyatie/.omc/specs/deep-dive-trace-realign-oyatie-corpus-lane-2.md` (3,337 lines)
- **Verdict**: STRONG evidence that drift was materially enabled by a surface-wave coordination model
- **Top findings**:
  - **F-01 STRONG**: ADR-0321 contains exact duplicate vendor sections (D-139/D-149 Fly.io at line 19675/22240; D-140/D-150 Cloudflare Workers at 19840/22404; D-141/D-151 Cloudflare R2 at 20356/22571) — proof of parallel-write race
  - **F-02..F-15 STRONG**: surface-first waves catalogued (Doc-suite W1..W10, per-msvc ADR A..F, runbooks W1..W4, ERP/B2B IP waves, journey waves, Rust-source waves, ADR-0321 author-waves D-111..D-155) — all organized by artifact surface, NOT by service owner
  - **High-touch µservices identified** as coherence-risk hotspots (9-11 wave tags + dozens to hundreds of missing internal references)
  - **Claim ratchet failed to prevent collision** — Oya VCS claim ledger had no durable same-file claim barrier preventing overlapping ADR-0321 appenders
- **Critical unknown**: was the claim ratchet bypassed deliberately or did it never enforce same-file collision detection?
- **Recommendation**: per-µservice ownership pattern (1 agent owns 1 µservice end-to-end) + claim ledger enforced before any same-file write

### Lane 3 (codex tracer): verification-failure cause
- **Status**: codex died without producing deliverable
- **Inferred from synthesis**: verification was line-count-based not substance-based → agents reported "completed" with shallow output that passed automated checks

### Synthesis (orchestrator-authored)
- **Artifact**: `/Users/jasonlee/oyatie/.omc/specs/deep-dive-trace-realign-oyatie-corpus-to-canonical.md`
- **Causal chain**: briefs → coordination → verification (all three lanes converge — drift was MULTI-CAUSED, no single root cause)
- **Final spec**: `/Users/jasonlee/oyatie/.omc/specs/deep-dive-realign-oyatie-corpus-to-canonical.md`
- **Implementation plan**: `/Users/jasonlee/oyatie/.omc/plans/realign-oyatie-corpus-plan-2026-05-20.md`

## Wave 1 — Canonical doctrine authoring (2026-05-20 21:35 → 22:45)

### Purpose
Land the canonical-direction doctrine that all Wave 2+ audits enforce. These are AUTHORING deliverables (not findings per se — substrate for everything else).

### Wave 1 Tasks 1.1-1.3 (initial dispatch)

| File | Initial lines | Purpose |
|---|---:|---|
| `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` | 1,888 | Codifies 5-phase sequence + Big 8 sub-sequence + agent-class anchor sets + 9-dim audit + 4-doc audit deliverables + brief format |
| `specs/master-plan-sequencing.json` | 705 | Machine-readable canonical_build_sequence + realignment_wave_sequence + later additions |
| `docs/standards/brief-template.md` | 1,181 | 5-citation header + agent-class anchor templates + decision trees + forbidden patterns |
| **Initial TOTAL** | **3,774** | |

### Wave 1 Task 1.4 amendment (cross-cutting constraints)

Adds 5 cross-cutting constraints to ADR-0328 §D-15..D-20 + master-plan-sequencing.json top-level keys + brief-template.md §3.9..3.12.

| File | After 1.4 | Δ | Added |
|---|---:|---:|---|
| ADR-0328 | 4,313 | +2,425 | §D-15 multi-context / §D-16 OpenTofu IaC / §D-17 OS matrix / §D-18 Rust-strict / §D-19 OCI Always Free / §D-20 audit decision tree (160 numbered clauses) |
| master-plan-sequencing.json | 869 | +164 | deployment_contexts (6 ids) / iac_substrate / supported_oses (13+2+6) / language_policy / oci_always_free top-level keys |
| brief-template.md | 1,891 | +710 | §3.9 multi-context anchor / §3.10 OpenTofu anchor / §3.11 OS anchor / §3.12 language policy anchor + Amendment summary |
| **After 1.4 TOTAL** | **7,073** | **+3,299** | |

### Wave 1 mid-flight amendments (2026-05-20 22:30 → 22:45)

| Amendment | Trigger | Affected files |
|---|---|---|
| Leptos web frontend (SSR + WASM hydration) | User directive | ADR-0328 §D-18.51a-l (12 clauses) + §D-20.91a; master-plan-sequencing.json `frontend_allowlist.web`; brief-template.md FRONTEND_ALLOWLIST; memory `feedback_rust_strict_only_no_python_2026_05_20.md` |
| Selective island-hydration refinement | User directive | ADR-0328 §D-18.51m-r (6 clauses adding hydration boundary granularity + WASM payload budgets + TTI telemetry); §D-20.91a; master-plan-sequencing.json web value renamed |

### Final Wave 1 deliverable sizes (post all amendments)

| File | Final lines | Δ from initial |
|---|---:|---:|
| ADR-0328 | 4,395 | +2,507 (+132%) |
| master-plan-sequencing.json | 870 | +165 (+23%) |
| brief-template.md | 1,891 | +710 (+60%) |
| **TOTAL** | **7,156** | **+3,382 (+90%)** |

### Wave 1 verification
- 5-canonical-anchor cross-check: PASS (6 deployment contexts, OpenTofu-not-Terraform, macOS-M5+-only, Rust-strict+frontend-Swift/Kotlin/WinUI/Leptos, OCI Always Free — all present with substance)
- Substance bar: PASS (each clause numbered + concrete + forbidden patterns enumerated + audit-finding severities defined)
- JSON validation: PASS (`jq empty specs/master-plan-sequencing.json` ✓)
- Markdown structure: PASS (code fences balanced; section line counts well above floors)

### Doctrine evolution within session (durable in memory)
- 5 cross-cutting constraints (memories: multi-context / OpenTofu / OS-matrix / Rust-strict / OCI-Always-Free) — all dated 2026-05-20
- Tier system retirement (no-capability-tiers memory) — 2026-05-20
- Tenant-class model — 2-class binary + composable billing_components (memory) — 2026-05-20, evolved 2026-05-20
- developer-sdk = Stainless-class generator (10 languages incl C/C++ added; Ruby/PHP/Elixir dropped) — 2026-05-20
- Cell µservice retires + network → community + community 4-pillar + social separate + mobile app bundle (memory) — 2026-05-21

## Audit dispatch log

### Wave 2 Batch 2.1 — Phase 0 cloud-infra (8 codex) — 2026-05-20 22:33 → 23:04 — COMPLETE+VERIFIED

| µservice | Coherence | Parity | Benchmarks | Tier-Deltas | P0 | P1 | P2 | P3 | Sum | Counterparts |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| cloud-iam | 638 | 407 | 301 | 399 | 0 | 11 | 11 | 7 | 29 | AWS IAM / Google IAM / Microsoft Entra ID |
| cloud-kms | 609 | 411 | 314 | 370 | 0 | 12 | 9 | 2 | 23 | AWS KMS / Google KMS / HashiCorp Vault |
| cloud-secrets | 731 | 409 | 437 | 353 | 0 | 12 | 9 | 4 | 25 | AWS Secrets Manager / Google Secret Manager / HashiCorp Vault |
| cloud-iac | 748 | 420 | 302 | 356 | 0 | 10 | 12 | 4 | 26 | Terraform Cloud / Pulumi Cloud / Spacelift |
| cloud-network | 668 | 411 | 422 | 355 | 0 | 8 | 8 | 3 | 19 | AWS VPC / Google VPC / Azure VNet |
| cloud-network-dns | 635 | 400 | 326 | 484 | 0 | 19 | 12 | 3 | 34 | AWS Route 53 / Google DNS / Cloudflare DNS |
| cloud-data | 647 | 411 | 346 | 360 | 0 | 22 | 23 | 3 | 48 | AWS RDS+Aurora / Google Spanner / Azure SQL+Cosmos |
| cloud-storage | 617 | 412 | 355 | 397 | 0 | 14 | 15 | 4 | 33 | AWS S3 / Google Cloud Storage / Azure Blob |
| **W2 TOTAL** | **5293** | **3281** | **2803** | **3074** | **0** | **108** | **99** | **30** | **237** | |

### Wave 3 Batch 3.1 — Phase 1 foundations 8/13 (8 codex) — 2026-05-20 23:09 → 23:40 — COMPLETE+VERIFIED

| µservice | Coherence | Parity | Benchmarks | Tier-Deltas | P0 | P1 | P2 | P3 | Sum | Counterparts |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| identity | 690 | 416 | 307 | 366 | **4** | 9 | 7 | 4 | 24 | Auth0 / Okta / Microsoft Entra ID |
| tenancy | 811 | 426 | 335 | 371 | 0 | 9 | 12 | 4 | 25 | AWS Organizations / GCP Resource Manager / Entra Tenants |
| audit-chain | 646 | 402 | 521 | 398 | 0 | 13 | 7 | 2 | 22 | AWS CloudTrail / GCP Audit Logs / MS Purview Audit |
| governance | 786 | 459 | 430 | 361 | 0 | 13 | 10 | 3 | 26 | AWS Control Tower / Azure Policy / GCP Org Policy |
| compliance | 806 | 405 | 306 | 368 | 0 | 16 | 13 | 3 | 32 | Vanta / Drata / OneTrust |
| observability | 830 | 421 | 507 | 362 | 0 | 7 | 15 | 5 | 27 | Datadog / New Relic / Grafana Cloud |
| payments | 795 | 412 | 312 | 357 | 0 | 6 | 9 | 3 | 18 | Stripe / Adyen / Braintree |
| finops-portal | 747 | 430 | 312 | 431 | 0 | 6 | 10 | 3 | 19 | Vantage / Cloudability / CloudHealth |
| **W3B1 TOTAL** | **6111** | **3371** | **3030** | **3014** | **4** | **79** | **83** | **27** | **193** | |

### Wave 3 Batch 3.2 — Phase 1 foundations 5/13 (5 codex; 3-deliverable schema, tier-deltas dropped) — 2026-05-20 23:42 → 00:15 — COMPLETE+VERIFIED

| µservice | Coherence | Parity | Benchmarks | P0 | P1 | P2 | P3 | Sum | Tier-Retire | Counterparts |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| api-gateway | 685 | 408 | 311 | 0 | 6 | 9 | 3 | 18 | 43 | AWS API Gateway / Kong / Apigee |
| application | 718 | 550 | 653 | 0 | 9 | 11 | 3 | 23 | 27 | Heroku / Vercel / Fly.io |
| developer-sdk | 659 | 475 | 369 | 0 | 7 | 8 | 2 | 17 | 28 | Stainless / Speakeasy / Fern |
| network | 688 | 421 | 343 | 0 | 5 | 9 | 4 | 18 | 35 | (mis-assigned: AWS VPC Lattice / GCP Cross-Cloud / Azure VWAN — µservice is LinkedIn-class; re-audit in 15K) |
| cell | 658 | 405 | 366 | 0 | TBD | TBD | TBD | TBD | TBD | (mis-assigned: AWS Cell / GCP Distributed / Fastly — cell µservice retiring per 15L; this audit becomes migration source) |
| **W3B2 TOTAL** | **3408** | **2259** | **2042** | **0** | **27+** | **37+** | **12+** | **76+** | **133+** | |

### Wave 4-rolling — Claude agents Round 1 (3 of 3 COMPLETE) — 2026-05-21 00:11 → ~

| µservice | Coherence | Parity | Benchmarks | P0 | P1 | P2 | P3 | Sum | Tier-Retire | Tenant-Class Gaps | Verdict |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|---|
| cloud-billing | 638 | 438 | 388 | **12** | 13 | 10 | 5 | 40 | 10 | YES — billing_components NOT MODELED (rev_share absent; per_seat absent; per_usage partial); tenant_class enum not on tree | REVISE |
| cloud-billing-tax | 1404 | 1634 | 958 | 0 | 27 | 30 | 14 | 71 | 12 | YES — 14 specific gaps (C-01..C-14); principal.tenant_class binding absent; billing_components context absent; downgrade prohibition absent | REVISE |
| cloud-k8s | 986 | 390 | 593 | 0 | 3 | 10 | 11 | 24 | TBD | YES — finding F-DIR-02 records need to re-express Platinum tier Cloud Hypervisor + Kata pods under deployment-context post-tier-retirement | PASS-w/findings |
| **CLAUDE-R1 TOTAL** | **3028** | **2462** | **1939** | **12** | **43** | **50** | **30** | **135** | **22+** | | |

### Wave 4-rolling — Rolling codex cohort (20 codex) — 2026-05-21 00:10 → IN FLIGHT

Dispatched µservices: intelligence / ontology / workflow-engine / workflow-studio / consent-graph / detection / mail / drive / calendar / meet / recordings / notes / docs / sheets / slides / forms / connect / comms-email / community / shorts

NOTE: community used OLD counterparts (Discourse/Circle/Vanilla) — needs re-audit in Wave 15K with Reddit/Teamblind/Handshake per 2026-05-21 directive.
NOTE: shorts is becoming the video substrate (NOT consumer-feed) per 2026-05-21 mobile-app-bundle directive — needs scope re-audit.

Findings tally: PENDING (watcher b6uz3gins armed)

### Wave 4-rolling — Claude agents Round 2 (3 of 3 COMPLETE) — 2026-05-21 ~00:30 → ~01:00

| µservice | Coherence | Parity | Benchmarks | P0 | P1 | P2 | P3 | Other | Sum | Verdict | Key gaps |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|---|
| marketplace | 625 | 401 | 327 | ~7 (6-cat gap) | ~30 | ~20 | TBD | 5 SUPERIOR | ~62 | APPROVE_WITH_DOCUMENTED_GAPS | 6-category coverage (0/6 per-category manifests for plugins/apps/workflows/agents/models/datasets); revenue_share missing 11 pieces (marketplace→cloud-billing ingestion contract, settlement event, clawback, demo_trial denial path, FX snapshot); 152 capability rows × 3 counterparts × 6 categories × 2 tenant classes. 5 SUPERIOR preserved (single cross-category ledger, BLAKE3 audit, EU AI Act pack) |
| feature-flags | 614 | 436 | 423 | 0 | 4 | 6 | 1 | 1 retraction | 12 | PASS-WITH-FINDINGS | F-COH-007 tenant_class absent from EvaluationContext (universal seam); F-COH-008 zero consumer crate imports; F-COH-009 zero per-context IaC; **F-COH-010 iac/terraform/main.tf uses HashiCorp Terraform engine (forbidden_engines violation)**; tier-matrix retraction |
| crm (Big 8 P0-severity) | 695 | 479 | 516 | **94** | 9 | 1 | 0 | 9 INFO/preserve | **113** | BIG-8 PROMOTION BLOCKED | **Template-stamping pervasive** (README 169 identical rows, ARCHITECTURE §H 90 traces, competitor-parity-matrix 327 stamped rows, PRD §C 30 stamped user stories → all BIG-8 P0 per ADR-0324 anti-stamping); 23 tier call-sites + 81 stamped-loop in PRD; tenant_class adoption gap TOTAL (0/10 surfaces); **Big-8 counterpart inversion** (Salesforce treated as #3 not anchor; HubSpot ABSENT from matrix/README/ARCHITECTURE; Dynamics named with stale "Customer Engagement" suffix); OpenTofu IaC zero-coverage (1 7-line Terraform stub, no resources); OS matrix zero-declaration; UNION-coverage 30-40%, active-gap pool 78% (50/64 capabilities); Missing CRM primitives: CPQ, Sales Cadences, Lead/Opportunity AI scoring, Reports primitive, Mobile CRM, Lead/Contact bounded contexts, OpportunityTeam, Quote-to-Cash, Custom Objects extensibility |

**Claude R2 total**: 1,353 (marketplace) + 1,473 (feature-flags) + 1,690 (crm) = **4,516 lines authored**
**Claude R2 findings total**: ~62 + 12 + 113 = **~187 findings** including **~101 P0s** (94 in crm alone — template-stamping pattern)

### Wave 4-rolling — Recovery dispatch (1 of 1 COMPLETE) — 2026-05-21

| µservice | Coherence | Parity | Benchmarks | P0 | P1 | P2 | P3 | Sum | Verdict | Key gaps |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---|---|
| messenger (recovery) | 627 | 460 | 401 | 3 | 11 | 10 | 6 | 30 | REVISE | **F-MSGR-001 (P0)** zero of 6 canonical deployment-context iac/ directories; **F-MSGR-002 (P0)** directory still named `iac/terraform/` (forbidden engine name per brief-template §3.10); **F-MSGR-003 (P0)** mobile-app-bundle directive (messenger + mail + social + community = one app per 2026-05-21) has ZERO coordination in messenger PRD/manifest/handoffs. **MLS RFC 9420 E2EE adoption: YES** (17 file hits + ADR-MSG-001 substance-bar-grade with 80+ named decisions on ciphersuite/KeyPackage/Welcome/Commit/Cedar gates/OpenBao paths/key rotation; coverage gap: tenant_class × compliance-pack binding for MLS opt-in not codified F-MSGR-007). |

## Aggregate findings so far (Waves 1-3 + Claude R1 + Claude R2 + Recovery)

| Severity | Wave 2 | Wave 3 B1 | Wave 3 B2 | Claude R1 | Claude R2 | Recovery | TOTAL |
|---|---:|---:|---:|---:|---:|---:|---:|
| P0 | 0 | 4 | 0 | 12 | ~101 | 3 | **~120** |
| P1 | 108 | 79 | 27+ | 43 | ~43 | 11 | **~311+** |
| P2 | 99 | 83 | 37+ | 50 | ~27 | 10 | **~306+** |
| P3 | 30 | 27 | 12+ | 30 | ~1 | 6 | **~106+** |
| Other | 0 | 0 | 0 | 0 | 15 | 0 | **~15** |
| **TOTAL** | **237** | **193** | **76+** | **135** | **~187** | **30** | **~858+** |

Audited so far: **48 µservices (out of 77 active)** — 62% coverage.

## P0 hotspots (need Wave 15A remediation priority)

| µservice | P0 count | Findings summary |
|---|---:|---|
| **crm** | **94** | **Template-stamping pattern at scale** — README 169 stamped rows + ARCHITECTURE §H 90 traces + competitor-parity-matrix 327 stamped rows + PRD §C 30 user stories all = BIG-8 P0 per ADR-0324. Plus Big-8 counterpart inversion (Salesforce #3 not anchor, HubSpot absent, Dynamics stale-suffix), OpenTofu IaC zero-coverage, OS matrix zero-declaration, tenant_class adoption TOTAL gap, 50/64 capabilities active-gap. The single most-incoherent µservice in the corpus. |
| **cloud-billing** | **12** | billing_components NOT MODELED (revenue_share/per_seat absent, per_usage partial); 6-context iac/ absent; supported-oses.json absent; OCI Always Free unmapped; Cedar policies absent; PRD/ARCHITECTURE/README/contracts/SLOs all absent (kernel is hyperscaler-grade but spec surface missing); contradictions BL-013..BL-016, BL-025 |
| marketplace | ~7 | 6-category coverage gap (0/6 per-category surfaces for plugins/apps/workflows/agents/models/datasets); tier-matrix.md DELETE candidate; revenue_share missing critical pieces (marketplace→cloud-billing ingestion contract, settlement event, clawback) |
| identity | 4 | OpenTofu context modules missing (T0 substrate); supported-oses.json missing; OCI Bronze ↛ Always Free mapping; multi-context resolver manifest/capability maturity mismatch |
| messenger | 3 | F-MSGR-001 zero deployment-context iac/; F-MSGR-002 dir still named `iac/terraform/` (forbidden engine name); F-MSGR-003 mobile-app-bundle directive zero coordination in PRD/manifest/handoffs |
| **performance-management** | **27** | Big-8 HR/Payroll elevation. 24% counterpart coverage (Big-8 floor 85%); template-stamped boilerplate (4 highest-traffic docs same 8-bullet pattern 15+ times); tier residue + tenant_class missing; Terraform-named IaC files; 0/6 deployment-context sub-dirs; missing HR-family sibling edges. Promotion BLOCKED. |
| data-warehouse | 6 | Template-stamped README + PRD (30-150× sentence-with-placeholder); tenant_class ZERO grep hits; Databricks Lakehouse substrate ABSENT (Delta/Iceberg/Hudi/Unity-Catalog/Photon/Auto-Loader/DLT/CDF); 60-primitive union 11/24/25 (18% pass rate). Verdict CONDITIONALLY-COHERENT, REMEDIATION-REQUIRED |
| **incident-management** | **10** | Big-8 ServiceNow Phase 4A.4 elevation. Bounded-context plurality divergence (5 in docs vs 1 in src/catalog/Cargo); ADR-0316 still cited as live authority; README + competitor-parity + PRD §C/§D template-stamped filler; supported-oses.json absent; iac/ flat + terraform-module.tf filename; OCI Always Free module absent; deployment_contexts not declared in manifest; SLOs tier-segmented; substrate_dependencies missing cmdb+itsm+change-management. Counterpart-selection delta (xMatters vs PagerDuty/Opsgenie/FireHydrant). Phase 4A.4 ship BLOCKED. |
| data-pipeline | ~3 | Template-stamped: competitor-parity-matrix + PRD §B/C/D/H + ARCH §F. Verdict YELLOW; 16 remediation IPs filed inline (IP-031..IP-037 + REMEDIATEs). 47 primitives (Fivetran+Airbyte+dbt Cloud): 38/5/4. Kernel quality GREEN. 2,736 lines (deepest audit yet). |
| healthcare-integration | 2 | Template-stamped competitor-parity-matrix.md ~90% (P0); tier vocabulary not migrated (P0). 215 features × 14 domains: 74% covered. HIPAA pack PARTIAL. FHIR R5+R4 covered. **Performance LEADERSHIP**: FHIR READ p99 38ms (7× Redox); HL7v2 102.8K msgs/sec; DICOM C-STORE 10.25K inst/min; MPI 130ms p99 (9× Redox). Counterpart mismatch (EHR vendors used) corrected to Redox/Mirth/Health Gorilla. 2,215 lines. |
| financial-planning | 9 | **Template-stamping at massive scale**: PRD 250 trace rows + 25 generic user stories; competitor-parity-matrix 8 × 16 sections; ARCHITECTURE §F × 20 anchors; +6 more docs stamped. STRONG IP-level substance: IP-026..IP-030 (~200 lines each) + ADR-FP-001 Proposed. **Vocabulary contradiction** across ARCHITECTURE/ADR-FP-001/PRD. **Industry precedent error**: "Microsoft Power BI planning integrations" (BI tool, not FP&A). **Excel integration MISSING** (Vena's identifying capability). 159 features × 18 groups: 96 covered (60.4%). **BEHIND by 3 orders on formula throughput sustained** (Phase-5 hyperblock engine needed). Phase 4 ERP promotion BLOCKED. 1,590 lines. |
| learning-management | 8 | Big-8 HR/Payroll. Counterpart-set contradiction (manifest 6 vs brief 3). Canvas LMS bounded-context gap (no assignment/grade-book/SCORM/xAPI/LTI). Template-stamped 6 docs. Rust src/ 1-vs-5 bounded contexts. Audit author REFUSED to pad coherence-audit to 600-line floor (503 actual) — chose substance per ADR-0328 §D-4. Architectural decision needed: absorb Canvas semantics OR reserve for future learning-academic µservice? 1,575 lines. |
| **itsm** | **33** | Big-8 ServiceNow Phase 4A.4. 227 total findings. 8/28 ServiceNow ITSM surfaces covered. **Performance leadership**: SLA breach 15s p99 (8× SN); workflow 800/sec sustained (7×); CMDB 380ms 3-hop (3.7×). 1,614 lines. |
| **contract-lifecycle-management** | **100** | 96 legal-complexity + 4 non-legal P0s. SHOWSTOPPER tier-matrix. 25 distinct tier call-sites + 39 local-* files. ZERO of 12 tenant-class surfaces. **20 P0 legal-compliance gaps** (GDPR/eIDAS/ESIGN/UETA/HIPAA/SOX/KR-PIPA/FCPA/EU AI Act/SEC 17a-4). 30-40% UNION (Ironclad/DocuSign/Conga). Template-stamped (PRD §C 25 + §M 217 + ARCH §F 208 + parity 320+). STRONG: IP-026..IP-030 hyperscaler-grade legal-bespoke; ADR-CLM-001 substantive; Rust-strict PASS; 120-cell perf grid. 1,958 lines. |
| **marketing-automation** | **103** | **Big-8 HubSpot elevation**: 96 Big-8 + 7 non-Big-8 P0s. 119 total findings (96 BIG-8 + 7 P0 + 9 P1 + 3 P2 + 4 INFO). Top stamped surfaces: ARCH §F 210 + README 154 + competitor-parity 304 + PRD §C-D 55 + 20 uniform-55-line IPs (IP-006..IP-025). 24 tier call-sites + 25 IP frontmatter scrubs. **ZERO of 11 tenant-class surfaces** (mirrors crm posture). Big-8 family: HubSpot 25-35% / Marketo 25-35% / Mailchimp 20-30% / UNION ~25-30%. 25 MISSING bounded contexts (Email, Landing Page, Form, Workflow visual canvas, Lead Scoring, ABM, Lifecycle Stage, Subscription Type, A/B Test, Send-Time Optimization, Marketing Asset library, Static List, Chatflow, Ad Network, SEO, CMS, etc.). 20 NEEDS-DECISION items (ND-1..ND-20). 10 DIFFERENTIATORS preserved: real-time segment materialization (40× HubSpot Active List), consent suppression ledger HLC-aware, multi-touch attribution reconciler (6× HubSpot), deliverability warmup state machine, cross-channel frequency cap (novel), marketplace audience-license (novel). 2,114 lines. |
| **TOTAL P0** | **~467** | + treasury (4). **Template-stamping at 15/26 = 58% Claude-audit rate** — DOMINANT pattern. **Counterpart-mismatch sub-pattern in 4 µservices** (healthcare-integration used EHR vendors; learning-management 6 vs 3; quality-management 4 internal docs/3 different sets; treasury used ERP-cash modules instead of TMS-leaders) — pre-existing wave-by-surface authoring chose obvious cloud-suite vendors rather than domain-specific best-of-breed leaders. |

## Codex audit findings summary (12 µservices, no elevation)

| µservice | P0 | P1 | P2 | P3 | Tier refs | Counterparts |
|---|---:|---:|---:|---:|---:|---|
| social | 0 | 12 | 9 | 4 | 0 + false-pos | TikTok / Instagram / Snapchat (corrected per 2026-05-21 directive) |
| analytics | 0 | 3 | 15 | 2 | 148 | GA4 / Mixpanel / Amplitude |
| tasks | 0 | 4 | 10 | 2 | 167 | Linear / Jira / Asana |
| translate | 0 | 4 | 13 | 2 | 234 | Google Translate / DeepL / AWS Translate |
| whiteboard | 0 | 9 | 14 | 5 | 28 | Miro / FigJam / Lucidchart |
| design-collaboration | 0 | 6 | 8 | 3 | 163 broad | Figma / Adobe XD / InVision |
| plugin-app-store | 0 | 8 | 12 | 4 | 46 | VS Code Marketplace / Chrome Web Store / Shopify App Store (ADR-PAS-0004 explicitly tier-named — retraction target) |
| workplace-integration | 0 | 8 | 8 | 2 | 29 | Slack App Directory / Microsoft Teams App Store / Zapier Integrations |
| ops-dashboard-control-center | 0 | 4 | 10 | 4 | 30 | Datadog / PagerDuty / AWS CloudWatch + Systems Manager |
| real-estate | 0 | 3 | 11 | 2 | 45 | AppFolio / Yardi / RealPage |
| **plant-maintenance** | **3** | 7 | 14 | 6 | **401** | SAP PM / IBM Maximo / UpKeep |
| global-trade | 0 | 8 | 11 | 2 | **509** | SAP GTS / Thomson Reuters ONESOURCE / Descartes |
| **CODEX TOTAL** | **3** | **76** | **135** | **38** | **~1,900** | (12 µservices) |

**Wave 15J retirement scope REVISED**: ~1,900 tier references just from these 12 codex audits + Claude-audit candidates pushes total above 3,000+. Original 9,300-occurrence count was character-level (Bronze appears in word "Bronze" but also "BronzeMedal" etc.); my 1,680 distinct-call-site estimate WAS A UNDERCOUNT.

**Critical pattern**: most codex µservices have ZERO literal Bronze/Silver/Gold/Platinum mentions but huge "broader tier-model semantics" (pricing-tier vocabulary, environment-stage labels, "tier-1/tier-2/tier-3" capacity descriptors, capability-tier doctrine cross-refs). The retirement must handle ALL forms, not just the literal Bronze/Silver/Gold/Platinum tokens.

## Wave 15 retirement candidates by category

### Wave 15J — Tier system retirement (running candidate count)
- Wave 2: 8 µservices each have capability-tiers/tier-matrix.md
- Wave 3 B1: 8 µservices each have capability-tiers/tier-matrix.md
- Wave 3 B2: 133+ tier-retirement candidates catalogued (api-gateway:43, application:27, developer-sdk:28, network:35, cell: TBD)
- Claude R1: 22+ tier-retirement candidates (cloud-billing:10, cloud-billing-tax:12, cloud-k8s: TBD)
- TOTAL identified so far: **155+ tier references** across 24 µservices (out of ~9,300 total in corpus per earlier scope-audit)

### Wave 15K — network → community merge
- network's Wave 3 B2 audit self-flagged counterpart mismatch — that audit IS the migration source
- community's in-flight codex audit will use OLD counterparts (Discourse/Circle/Vanilla) — needs re-dispatch

### Wave 15L — cell µservice retire
- cell's Wave 3 B2 audit (658+405+366 lines) becomes migration source
- Absorbed by tenancy + cloud-iac + observability + oyatie-shuffle-sharding crate + api-gateway + audit-chain
- µservice count: 79 → 78

## Tenant-class adoption gaps (cross-µservice pattern)

Every audited µservice flags `tenant_class` adoption gaps:
- Wave 2 + W3 B1 + W3 B2 + Claude R1 ALL report "tenant_class_adoption_gaps: yes"
- Common pattern: principal.tenant_class claim not modeled; billing_components context attribute absent; demo_trial cap-breach behavior absent; demo_trial → paid conversion flow absent
- Most detailed: cloud-billing-tax (14 specific gaps C-01..C-14) + cloud-billing (12 P0s tied to billing_components implementation)
- This is the universal substrate gap — needs cross-µservice doctrine in Wave 15J + a per-µservice plumbing IP in each affected µservice

## Counterpart matrix coverage

Most µservices report:
- 30-60 distinct counterpart capabilities per counterpart in §2 of parity matrix
- UNION-coverage matrix in §4 with capability × C1/C2/C3 × Oyatie × gap-classification
- Family summary
- Headline gap analysis (typically 5-15 critical gaps)
- Additive surface (Oyatie capabilities not in counterparts)

Notable counterpart-related findings:
- network µservice: counterparts mis-assigned (LinkedIn-product audited against networking-infra counterparts) — Wave 15K fix
- cell µservice: counterparts mis-assigned (cellular-architecture-pattern audited against edge-cloud products) — Wave 15L migration source
- cloud-billing-tax: 5 vendors in existing benchmark (Stripe+Avalara+TaxJar+Vertex+Sovos) vs 3-vendor prompt assignment — recorded as F-DIM4-01/F-DIM5-01 disagreement per ADR-0328 §D-5.3

## In-flight (not yet aggregated)

- Rolling codex (20 µservices): intelligence / ontology / workflow-engine / workflow-studio / consent-graph / detection / mail / drive / calendar / meet / recordings / notes / docs / sheets / slides / forms / connect / comms-email / community / shorts — ALL completed (44 audited on disk per find query)
- Claude R2 (3 µservices): crm / marketplace / feature-flags — IN FLIGHT
- Claude recovery (1 µservice): messenger — IN FLIGHT (recovered from earlier dispatch gap)

## Audit coverage verification (2026-05-21)

| Metric | Count |
|---|---:|
| Total active µservices (excluding README, erp-summary.json, foundry-retiring) | 77 |
| Audited on disk (coherence-audit-2026-05-20.md exists) | 44 |
| In flight (Claude R2 + recovery) | 4 |
| Remaining queue | 29 |
| **Sum check** | 44+4+29 = 77 ✓ |

## Wave dispatch summary

| Wave | Mechanism | µservices | Status |
|---|---|---:|---|
| Wave 0 | Investigation (3 lanes: 1 codex+ synthesis + Lane 2 codex completed 3,337 lines) | n/a (causal trace) | Complete |
| Wave 1 Tasks 1.1-1.3 | Initial doctrine authoring (3 files: ADR-0328 + spec + brief template) | n/a (doctrine substrate) | Complete |
| Wave 1 Task 1.4 amendment | Cross-cutting constraints (+3,299 lines) | n/a (doctrine substrate) | Complete |
| Wave 1 mid-flight | Leptos + selective-island-hydration (+82 lines ADR-0328) | n/a (doctrine substrate) | Complete |
| **Wave 2 Batch 2.1** | 8 codex (Phase 0 cloud-infra first 8) | 8 | Complete + verified |
| Wave 2 Batch 2.2-2.3 (planned not dispatched) | (3 Phase 0 remainder absorbed into Claude R1) | (0) | Replaced by Claude R1 |
| **Wave 3 Batch 3.1** | 8 codex (Phase 1 foundations first 8) | 8 | Complete + verified |
| **Wave 3 Batch 3.2** | 5 codex (Phase 1 foundations remaining 5; 3-deliverable schema) | 5 | Complete + verified |
| **Wave 4-rolling Claude R1** | 3 Claude agents (Phase 0 remainder) | 3 | Complete |
| **Wave 4-rolling codex** | 20 codex (Wave 4 6 µservices + Phase 3 first cohort 14) | 20 | Complete |
| **Wave 4-rolling Claude R2** | 3 Claude agents (Phase 4 priority: crm/marketplace/feature-flags) | 3 | IN FLIGHT |
| **Wave 4-rolling recovery** | 1 Claude agent (messenger gap fix) | 1 | IN FLIGHT |
| **Audit total** | | 47 dispatched / 44 on disk / 4 in flight | |
| Remaining queue | (Phase 3 rest + Phase 4 distribution + Big 8 + long-tail) | 29 | Queued |
| Wave 14 aggregation | Orchestrator-authored | n/a | Incremental (this file) |
| Wave 15A-L remediation | Sub-waves | n/a | Pending post-Wave-13/14 |

## Wave 14 aggregation deliverable (final)

When all audits complete, this running tally will be polished into the canonical Wave 14 aggregation at `.omc/state/wave-14-aggregation.md` with:
- Per-phase findings rollup
- P0 → P3 prioritized remediation backlog
- Wave 15A-L sub-wave routing
- Cross-µservice systemic patterns
- Tenant-class adoption plumbing IP (across all µservices)
- Counterpart-research database (durable across realignment + future product work)
