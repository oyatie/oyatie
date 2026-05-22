---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-performance-management
microservice: performance-management
status: wave-4-rolling-remediation
date: 2026-05-21
owner_team: axis-performance-management + council-product
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0245
  - ADR-0248
  - ADR-0314
  - ADR-0315
  - ADR-0316
  - ADR-0321
  - ADR-0328
  - ADR-0329
  - ADR-0330
  - ADR-0331
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
companion_docs:
  - microservices/performance-management/README.md
  - microservices/performance-management/ARCHITECTURE.md
  - microservices/performance-management/compliance.md
  - microservices/performance-management/manifest.json
  - microservices/performance-management/coherence-audit-2026-05-20.md
big_8_family: HR/Payroll
big_8_priority: P0
audit_dimensions_traced:
  - D-15 substance-bar
  - D-16 batch-discipline
  - D-17 HR/Payroll-first
  - D-19 tenant_class
  - D-20 9-dimension audit
---

# PRD-performance-management: Performance Management

## A. Problem

Companies must run a tight loop between goals, feedback, reviews, and pay decisions. Today
the loop fragments across vendor silos — Lattice for OKRs + reviews + 1:1s, 15Five for
check-ins + engagement, Workday Performance for calibration + succession — and each
integration leaks data, leaks consent, and leaks audit trail. Buyers want one operational
plane that talks to every HR sibling (compensation, people-records, learning-management,
time-tracking, workforce-planning, recruiting) without forcing them to pick a suite vendor.

`performance-management` exists because **the operational concern of running performance
itself** cannot safely live inside `compensation` (different lifecycle), `people-records`
(different access pattern), or `learning-management` (different identity). It is the
HR/Payroll Big-8 P0 µservice per ADR-0328 §D-20.111-115 — every promotion past `dev` requires
zero P0 findings open against this PRD.

## B. Target users

Per `feedback_quality_performance_scalability_bar` we target hyperscaler-leader experience
for these primary personas. Each has a specific operational pain that drives a use case.

- **Maya Hernandez**, manager of an 8-person engineering team — needs goal cascade, 1:1 prep
  packet, and quarterly review form. Pain: reading a goal cycle, drafting a review, and
  preparing a 1:1 are three separate tools today.
- **Marcus Chen**, ops owner at a 600-person B2B SaaS company — needs review-cycle close
  end-to-end without vendor lock-in, with audit evidence for board / investors.
- **Diana Alvarez**, HRBP at an agency — runs quarterly engagement pulses across 12 tenant
  clients. Needs anonymity floor, segmentation, and tenant-class redaction.
- **Nadia Singh**, enterprise CHRO admin at a 50k-employee tenant — needs calibration with
  nine-box, succession plans, executive views, and EU works-council consultation flow.
- **Omar Watkins**, SRE accountable for review-cycle close incident response — needs
  rollback + replay + breakglass with security-plus-service-owner approval chain.
- **Hana Mori**, internal auditor for SOC 2 + ISO 27001 — needs full audit trail of every
  rating change, calibration outcome, and engagement release.
- **Yejin Park**, side-business owner running a 4-person team — needs minimum viable
  performance (goals + weekly check-in + recognition) on the demo_trial tier without
  paying enterprise fees.
- **Sarah Kim**, executive at a 5k-employee tenant — needs talent map, succession board,
  and aggregated calibration view across her org.

## C. Functional requirements

Mapped to ADR-0328 §D-20 audit dimensions: each FR cites which dimension(s) it exercises.

### C.1 Goal-cycle context

- **FR-001** (D-20.1, D-20.5) — Tenant-scoped goal authoring with title, description,
  metric (key result), target, due date, owner, parent goal (for cascade), priority.
- **FR-002** (D-20.5) — Org-tree cascade: an employee can align a goal up to manager and
  down to direct reports; alignment edges form a DAG (no cycles).
- **FR-003** (D-20.5) — Quarterly check-in: progress update, blockers, confidence rating.
- **FR-004** (D-20.5) — Goal-cycle close with carry-forward to next cycle.
- **FR-005** (D-20.2) — OKR cascade view: manager sees aggregated progress for their report
  tree; cell-aware roll-up per ADR-0248.

### C.2 Review-cycle context

- **FR-006** (D-20.1, D-20.5) — Configurable review-cycle templates (annual, semi-annual,
  project-anytime, probationary, 30/60/90 new-hire).
- **FR-007** (D-20.5) — Review form authoring: free-text + structured rating + competency
  scoring (1-5 scale by default; tenant-configurable).
- **FR-008** (D-20.5) — 360 feedback collection: peer + report + manager + skip-level.
- **FR-009** (D-20.5) — Manager rating + narrative summary with override flag.
- **FR-010** (D-20.5) — Review evidence sealing per IP-027 (immutable hash chain).
- **FR-011** (D-20.5) — Outbound `RatingFinalizedEvent` to compensation per IP-030.

### C.3 Feedback context

- **FR-012** (D-20.5) — Anytime feedback give/request (asks like "what should I keep doing").
- **FR-013** (D-20.5) — 360 feedback request loop (manager initiates; peers respond).
- **FR-014** (D-20.5) — Feedback visibility rules per Cedar gate; manager note class is
  private to manager + skip-level + HRBP only.

### C.4 Engagement-survey context

- **FR-015** (D-20.1, D-20.5) — Pulse survey (eNPS-style; weekly/monthly/quarterly cadence).
- **FR-016** (D-20.5) — Full engagement survey (configurable question bank).
- **FR-017** (D-20.5) — Anonymity floor enforcement (k≥8 default; pack override allowed).
- **FR-018** (D-20.5) — Sentiment keyword extraction over free-text responses (per
  audit directive 8; engagement-pulse keyword extraction).

### C.5 Calibration context

- **FR-019** (D-20.5) — Calibration session: cohort load, distribution check, force-
  distribution overlay, nine-box grid.
- **FR-020** (D-20.5) — Calibration lock per IP-027 fairness ledger.
- **FR-021** (D-20.5) — Talent calibration with executive review; readiness rating
  (ready-now, ready-1-year, ready-2-year, not-ready).

### C.6 One-on-one and weekly check-in contexts

- **FR-022** (D-20.5) — One-on-one agenda authoring + action items + history.
- **FR-023** (D-20.5) — Weekly check-in: priorities, blockers, mood; manager rollup.
- **FR-024** (D-20.5) — Auto-prep packet for 1:1 (pulls recent feedback, goal progress,
  recognition received).

### C.7 Succession + talent context

- **FR-025** (D-20.5) — Talent card per employee (current role, next role candidates,
  readiness, development plan ref).
- **FR-026** (D-20.5) — Succession plan per role (incumbent + successors + bench).
- **FR-027** (D-20.5) — Outbound `SuccessionTalentCardEvent` to workforce-planning.

### C.8 Recognition context

- **FR-028** (D-20.5) — Public recognition post (kudos to a colleague; visible on wall).
- **FR-029** (D-20.5) — Reaction (peer endorses recognition).

### C.9 Manager tooling

- **FR-030** (D-20.5) — Manager dashboard: report-tree goals, recent feedback, upcoming
  1:1s, calibration cohort.
- **FR-031** (D-20.5) — Performance summary generator (LLM-assisted draft, manager edit).

### C.10 Analytics + reporting

- **FR-032** (D-20.5) — HRBP analytics dashboard: engagement trend, rating distribution,
  feedback volume, calibration fairness.
- **FR-033** (D-20.5) — Sentiment trend (per team, per quarter).
- **FR-034** (D-20.5) — Export to CSV/Excel with pack-redaction overlay.

### C.11 Mobile

- **FR-035** (D-20.1, D-20.5) — Swift iOS + Kotlin Android client SDK per
  `feedback_rust_strict_only_no_python_2026_05_20` frontend allowance. SDKs ship
  through `IP-019-sdk-client-generation.md`. Mobile surface covers goal check-in, feedback
  give, recognition, engagement pulse response, 1:1 agenda read.

### C.12 Compliance + audit

- **FR-036** (D-20.5) — Full audit trail of every review form draft, rating change,
  calibration outcome, engagement release.
- **FR-037** (D-20.5) — Pack-driven egress redaction per `compliance.md`.

### C.13 Sibling cross-handoff

- **FR-038** (D-20.2) — Consume `EmployeeDirectoryProjection` ← `people-records`.
- **FR-039** (D-20.2) — Consume `CompensationBandReference` ← `compensation`.
- **FR-040** (D-20.2) — Consume `LearningCompletionEvent` ← `learning-management`.
- **FR-041** (D-20.2) — Consume `TimeOffPeriod` ← `time-tracking`.
- **FR-042** (D-20.2) — Consume `RecruitingHiredEvent` ← `recruiting`.
- **FR-043** (D-20.2) — Produce `CalibrationOutcomeRecord` → `people-records`.
- **FR-044** (D-20.2) — Produce `RatingFinalizedEvent` → `compensation`.
- **FR-045** (D-20.2) — Produce `SuccessionTalentCardEvent` → `workforce-planning`.

## D. Non-functional requirements

- **NFR-001** (D-20.4 canonical-direction) — Cedar default-deny gates all writes; ADR-0243.
- **NFR-002** (D-20.5 counterpart parity) — ≥85% union coverage against Lattice + 15Five +
  Workday Performance per `feature-parity-matrix-2026-05-20.md` Big-8 P0 floor.
- **NFR-003** (D-20.6 multi-context) — Six deployment contexts present at
  `iac/<context>/` each: oyatie-public-cloud, guest-on-aws, oci-guest (with always-free
  sub-module), on-prem, colo, oyatie-iaas.
- **NFR-004** (D-20.7 OpenTofu) — Zero Terraform-named files; `iac/<context>/main.tf` lineage
  declares OpenTofu provider with signed modules.
- **NFR-005** (D-20.8 OS matrix) — 13-OS support per `supported_oses.json`; per-OS CI lane;
  arch matrix linux/amd64 + linux/arm64 + darwin/arm64 + Tier-2 ppc64le/s390x.
- **NFR-006** (D-20.9 Rust-strict) — Backend in Rust only; authorized non-Rust limited to
  HCL (OpenTofu), Cedar, OpenAPI/AsyncAPI/proto3, OpenSLO, SQL migrations, YAML/JSON,
  Markdown, and frontend Swift/Kotlin/WinUI/.NET.
- **NFR-007** (latency, paid tenant SLO) — Review form open p99 ≤ 300ms; goal cascade
  apply p99 ≤ 800ms; engagement pulse release p99 ≤ 1.5s; calibration outcome publish
  p99 ≤ 2s.
- **NFR-008** (availability) — ≥99.95% paid; ≥99.5% demo_trial.
- **NFR-009** (throughput) — Peak: 50k-employee tenant during annual close: 8M review-form
  opens / 14 days; 200k calibration writes / 14 days; 1.2M feedback entries.
- **NFR-010** (durability) — Audit chain append-only; review evidence seal immutable.
- **NFR-011** (HTTP/3 + QUIC default) — Per ADR-0253-amendment; ECH + PQC at the QUIC
  handshake.
- **NFR-012** (tenant scope) — Every row carries `tenant_id`; cross-tenant joins impossible
  by composite primary keys.
- **NFR-013** (tenant-class branching) — All Cedar gates, SLO targets, marketplace
  settlement, and cost-budget split on `tenant_class ∈ {demo_trial, paid}`.
- **NFR-014** (cellular shape per ADR-0248) — Cell tiers T1, T2; shuffle sharding across
  4 sub-cells for review-close peak; cell-local cache TTL ≤5min.

### DR posture per ADR-0343

- Target: RTO 3600 seconds and RPO 300 seconds for review-cycle, calibration, engagement-pulse, and manager-note commitments.
- Compliance floors: HIPAA-2024 requires 3600/300 with multi-region, KR-PIPA defaults to 14400/900 and tightens to 3600/300 with multi-region for resident-registration-number data, SOC2-T2 requires 14400/900, and ISO27001-2022 requires 14400/3600. The effective target is the strictest active floor: 3600/300.
- Failover runbook reference: `microservices/performance-management/iac/dr-failover.yaml`, with service recovery drill evidence in `runbooks/cycle-close-backfill.md`, `runbooks/review-evidence-seal-failure.md`, and `runbooks/calibration-deadlock.md`.
- Multi-region active-active posture: enabled for tenant home-cell command routing and evidence sealing; cross-cell replication remains metadata-only unless a compliance pack explicitly permits payload movement.
- Why: annual review close and calibration sessions are tenant-visible HR commitments, so failover must preserve in-progress rating evidence without widening works-council or HIPAA audit exposure.

### Capacity model per ADR-0340

- Per-tenant baseline: 0.07 vCPU, 192 MiB RAM, 3 GiB retained review/evidence metadata storage, 5 Postgres connections, 3 Valkey connections, and 14 outbound HTTP sockets.
- Scaling dimension: `per_user`, with review-close burst multipliers on form-open, feedback-write, and calibration-lock traffic.
- Cell placement class: Tier-3 in `manifest.json#capacity_model`, with T1/T2 cell eligibility handled by admission and pack placement controls rather than the baseline capacity class.
- Autoscaling boundaries: minimum 1 warm replica per tenant home cell, maximum 8 replicas per paid tenant during review-close, and async workers capped at 6 per tenant to prevent calibration replay storms.
- Why: the expected load profile is a few steady manager actions most weeks, then a two-week annual-close spike with millions of reads and hundreds of thousands of rating/calibration writes.

### Sustainability and cost attribution per ADR-0344

- Every audit-chain row emitted by goal, review, feedback, calibration, engagement, and export paths carries `cost_usd_minor_units`, `co2_grams`, and `watt_hours` beside tenant, cell, capability, provider, and compliance-pack dimensions.
- Carbon-aware provider routing: yes for async analytics, sentiment trend, export, and backfill jobs; no for live review submission, calibration lock, or breakglass paths where latency and labor-law evidence windows dominate.
- Tenant cost surface: FinOps Portal exposes per-tenant performance-management cost by capability and review-cycle window; local cost-budget evidence remains in `IP-017-cost-budget-enforcer.md`.
- Why: CSRD, SB-253, and SEC climate-disclosure customers need auditable cost and emissions attribution without hiding HR evidence-plane overhead in platform averages.

### API versioning posture per ADR-0342

- Public API model: YYYY-MM-DD carrier triplet across `Oyatie-Version`, `/v/<YYYY-MM-DD>/performance-management/...`, and proto3 `oyatie_version`.
- SDK model: generated Swift, Kotlin, and service SDKs use semantic `major.minor.patch` versions.
- Support window: the last 3 public API versions remain supported for at least 180 days.
- Per-tenant pinning: yes, because HRIS, compensation, and mobile-client migration windows are tenant-specific.
- Internal mesh exemption: yes, preserving ADR-0145 direct gRPC between sibling HR services while public contracts keep the hybrid version carrier.

## E. User stories

### E.1 Goal cycle

- **US-001** — As Maya Hernandez, manager, I want to open a goal for my team that aligns to
  my manager's goal so that the cascade reflects org priorities.
  - **Acceptance**: POST `/v1/performance-management/goals` with `parent_goal_id`; OpenAPI
    3.2.0 schema validates; AsyncAPI `goal.created.v1` event emitted; Cedar
    `goal-alignment-approval` policy permits; audit event sealed.
- **US-002** — As Marcus, ops owner, I want quarterly check-in reminders so that goals don't
  go stale.
  - **Acceptance**: Workflow `wf.perf.goal-cycle.cycle-life` fires reminder 30 days into
    quarter; reminder visible in mobile + email.
- **US-003** — As Sarah, exec, I want to see goal-cascade rollup for my org.
  - **Acceptance**: GET `/v1/performance-management/goal-cascade?org_id=...` returns DAG.

### E.2 Review cycle

- **US-004** — As Maya, manager, I want to write a review form for each direct report.
  - **Acceptance**: review form template fetched; form draft saved; form submitted; rating
    finalized; outbound `RatingFinalizedEvent` to compensation.
- **US-005** — As Diana, HRBP, I want 360 feedback to be collected from peers + reports.
  - **Acceptance**: 360 collection request created; peers receive prompts; responses
    aggregated; manager view shows aggregated 360 deck.
- **US-006** — As Omar, SRE, I want to seal evidence so it's immutable.
  - **Acceptance**: review seal step writes audit hash chain; rating immutable post-seal
    except via `rating.change.breakglass`.

### E.3 Feedback

- **US-007** — As Maya, manager, I want anytime feedback for my reports.
  - **Acceptance**: POST `/v1/performance-management/feedback` saves; feedback visibility
    follows Cedar gate; recipient sees feedback within 30s.
- **US-008** — As Yejin (demo_trial), I want my team to give each other peer feedback.
  - **Acceptance**: feedback writes allowed; egress redacted to synthetic ids when exported.

### E.4 Engagement

- **US-009** — As Diana, HRBP, I want a monthly engagement pulse for my client tenants.
  - **Acceptance**: pulse cadence scheduled; sentiment keyword extraction runs; anonymity
    floor enforced at k≥8; aggregate release published.
- **US-010** — As Sarah, exec, I want eNPS for my org.
  - **Acceptance**: eNPS computed; trend visible in dashboard.

### E.5 Calibration

- **US-011** — As Diana, HRBP, I want to lock a calibration session.
  - **Acceptance**: session lock granted; ledger record created; deadlock recovery via
    `runbooks/calibration-deadlock.md`.
- **US-012** — As Sarah, exec, I want force-distribution + nine-box overlay.
  - **Acceptance**: distribution check runs; nine-box grid emitted; visible in calibration UI.

### E.6 One-on-one and weekly check-in

- **US-013** — As Maya, manager, I want a 1:1 agenda for each direct report.
  - **Acceptance**: agenda persists across meetings; action items tracked.
- **US-014** — As Maya, manager, I want auto-prep packet for 1:1.
  - **Acceptance**: GET `/v1/performance-management/one-on-ones/{id}/prep-packet` returns
    recent feedback, goal progress, recognition.
- **US-015** — As Marcus, ops owner, I want weekly check-ins from my org.
  - **Acceptance**: weekly check-in workflow fires Friday; manager rollup visible Monday.

### E.7 Succession and talent

- **US-016** — As Diana, HRBP, I want talent cards.
  - **Acceptance**: talent card authored; readiness rating set; outbound
    `SuccessionTalentCardEvent` to workforce-planning.
- **US-017** — As Sarah, exec, I want succession board for VP roles.
  - **Acceptance**: succession plan per role authored; bench listed.

### E.8 Recognition

- **US-018** — As Maya, manager, I want to recognize team members publicly.
  - **Acceptance**: recognition post visible on wall; peers can react.

### E.9 Analytics and reporting

- **US-019** — As Diana, HRBP, I want analytics dashboards.
  - **Acceptance**: dashboard tiles render with tenant_class facet; export via pack-redaction.
- **US-020** — As Hana, auditor, I want sentiment trend and full audit trail.
  - **Acceptance**: GET audit events; CSV export with pack-redaction.

### E.10 Mobile

- **US-021** — As Marcus, ops owner, I want my team to use the mobile app for daily flow.
  - **Acceptance**: Swift iOS + Kotlin Android clients support FR-035 surface; SDKs
    distributed via `IP-019`.

## F. Non-goals

- Not a payroll engine (delegate to `payroll`).
- Not a benefits administrator (delegate to `benefits`).
- Not a recruiting ATS (delegate to `recruiting`).
- Not a learning content provider (delegate to `learning-management`).
- Not a time-and-attendance tracker (delegate to `time-tracking`).
- Not the HRIS system of record (delegate to `people-records`).

## G. Risks

- **R-1** — HR/Payroll Big-8 P0 elevation means any P0 audit finding blocks promotion.
  Mitigation: this PRD ships alongside complete `coherence-audit-2026-05-20.md` and
  remediation log `REMEDIATION-NOTES-2026-05-21.md`.
- **R-2** — Cross-µservice handoff edges (B-1..B-9) require sibling readiness; the
  sibling `compensation` and `people-records` audits must reciprocate.
- **R-3** — Engagement-pulse anonymity policy violation = legal + reputational. Mitigation:
  Cedar `local-engagement-pulse-anonymity.cedar` enforces k≥8 floor; release is denied
  default-deny if cohort below floor.
- **R-4** — Workday Performance customers expect calibration with talent reviews; the
  succession + nine-box surface (US-011, US-012, US-016, US-017) is critical migration
  must-have. Mitigation: full IP coverage in IP-031..IP-037.
- **R-5** — EU works-council consultation flow can block calibration. Mitigation: pack
  `eu-worker-council` activates pre-cycle notification workflow.

## H. Open questions

- **OQ-1** — Does the engagement-pulse sentiment keyword extraction live in this µservice
  or in `analytics`? Default: extraction runs locally to maintain anonymity guarantee;
  aggregate-only signal published to `analytics` substrate. Closed by ADR-0331.
- **OQ-2** — Should manager-feedback-gate enforce a cool-down period? Default: 24h cool-down
  on negative-class feedback to allow manager edit; opt-out per tenant config.
- **OQ-3** — Calibration nine-box: tenant-configurable axes (performance × potential vs
  performance × impact)? Default: performance × potential; configurable axis names per
  pack overlay.

## I. Acceptance criteria

The PRD is acceptance-complete when:

1. All FR-001..FR-045 implemented + tested + traced to acceptance evidence.
2. Counterpart coverage ≥85% per `feature-parity-matrix-2026-05-20.md`.
3. All NFR-001..NFR-014 measured + green at SLO targets.
4. Zero P0 findings against `coherence-audit-2026-05-20.md` (audit re-run).
5. Six deployment contexts under `iac/<context>/` populated.
6. Cedar policy `local-engagement-pulse-anonymity.cedar` present + Cedar test green.
7. Seven HR-family cross-handoff contracts under `contracts/hr-handoff-*.asyncapi.yaml`.
8. `supported_oses.json` covering thirteen OS targets.
9. Per-OS + per-arch CI lane green.

## J. Migration plan

For tenants migrating from Lattice / 15Five / Workday Performance, see
`competitor-parity-matrix.md` §Migration and IP-027 (review-calibration-fairness-ledger).
Migration is staged:

- **Phase 1**: org tree from HRIS or counterpart export.
- **Phase 2**: historical goal cycles imported (read-only).
- **Phase 3**: in-flight review cycles imported (state-preserved).
- **Phase 4**: engagement-pulse history imported as aggregate-only.
- **Phase 5**: production cut-over; counterpart deactivated.

## K. Dependencies

- Substrate: `community`, `workplace-integration`, `workflow-engine`, `ontology`,
  `analytics`, `identity`, `compliance`, `policy-engine`, `audit-chain`.
- HR-Payroll siblings: `people-records`, `compensation`, `learning-management`,
  `time-tracking`, `workforce-planning`, `recruiting`.

## L. Out-of-scope reductions

None. Wave-4 remediation expands the surface (12 contexts vs original 5) to close the
counterpart-parity gap. No scope cut accepted per `feedback_go_with_original_ambition_2026_05_20`.

## M. Traceability and audit-dimension trace

This PRD traces to ADR-0328 §D-20 dimensions:

| Dimension | Coverage |
|---|---|
| D-20.1 internal coherence | NFR-001, NFR-012, manifest + README + ARCHITECTURE alignment |
| D-20.2 outbound cross-refs | FR-038..FR-045, all HR-family edges declared |
| D-20.3 substance bar | this PRD is bespoke; no template-stamping |
| D-20.4 canonical direction | NFR-001 (Cedar), NFR-006 (Rust-strict), NFR-011 (HTTP/3) |
| D-20.5 counterpart parity | FR-001..FR-035 cover Lattice + 15Five + Workday Performance union |
| D-20.6 multi-context | NFR-003 |
| D-20.7 OpenTofu | NFR-004 |
| D-20.8 OS matrix | NFR-005 |
| D-20.9 Rust-strict | NFR-006 |

## N. Companion artifacts

- `manifest.json` — machine-readable spec.
- `README.md` — operational summary.
- `ARCHITECTURE.md` — system design.
- `compliance.md` — per-pack control mapping.
- `competitor-parity-matrix.md` — capability-by-capability counterpart matrix.
- `feature-parity-matrix-2026-05-20.md` — coverage breakdown.
- `coherence-audit-2026-05-20.md` — audit source.
- `REMEDIATION-NOTES-2026-05-21.md` — remediation log.
- `IP-001..IP-037` — implementation plans.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-foundry-fitness-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `performance-management` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `performance-management` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 5 module pin(s) across 4 context(s).
- Scaling input: `per_user` with cell placement `Tier-3` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
