---
doc_class: feature-parity-matrix
microservice: performance-management
audit_wave: wave-4-rolling
audit_date: 2026-05-21
counterparts: [Lattice, 15Five, Workday Performance]
coverage_lens: union
big_8_family: HR/Payroll
big_8_priority: P0
parity_floor_percent: 85
current_coverage_percent: 37
governing_adr: ADR-0328
related_adrs: [ADR-0316, ADR-0321, ADR-0244, ADR-0245, ADR-0314]
companion_docs:
  - microservices/performance-management/coherence-audit-2026-05-20.md
  - microservices/performance-management/performance-benchmark-numbers-2026-05-20.md
---

# performance-management — feature-parity matrix (Lattice + 15Five + Workday Performance union, 2026-05-21)

## 0. Matrix envelope

### 0.1 Purpose

Build the union-coverage table for `performance-management` against the three top counterparts
Lattice, 15Five, Workday Performance. The matrix is the empirical basis for the 63% parity
gap recorded in coherence-audit-2026-05-20.md §5.4 and feeds the Phase D buildout plan
recorded in §11.1-D of the same audit.

### 0.2 Method

Each row in the matrix is one industry-counterpart capability primitive observed across the
three vendors' published feature surfaces. For each row the matrix records:

- column V (Vendor coverage): which of the three vendors expose this primitive
- column P (Present in performance-management): does the µservice already declare it (capability
  YAML, IP, contract operation, or PRD acceptance criterion)
- column G (Gap severity): P0/P1/P2 per ADR-0328 §D-20.111-119 with HR/Payroll auto-elevation
- column B (Bounded context owner): which of {goal-cycle, review-cycle, feedback, engagement-
  survey, calibration, new-context} owns the primitive
- column T (Tenant-class applicability): demo_trial / paid / both / paid-with-component-X
- column C (Cell tier minimum): T0/T1/T2/T3/T4 per ADR-0248

### 0.3 Scope

This matrix is product-feature-level. It does not enumerate substrate or operational
capabilities (those are in the coherence audit's dimension-1 internal-coherence section).

### 0.4 Counterpart surface reference

Lattice feature surface inventoried from Lattice public docs as of 2026-Q2.
15Five feature surface inventoried from 15Five public docs as of 2026-Q2.
Workday Performance feature surface inventoried from Workday HCM Performance module
documentation as of 2026-Q2.

Where one vendor's feature is a marketing-named-but-functionally-equivalent of another's, the
row is consolidated with the more generic primitive name.

## 1. Goals and OKRs surface

### 1.1 Matrix — goal management

| # | Capability primitive | V:Lattice | V:15Five | V:Wd Perf | P | G | B | T | C |
|---|---|---|---|---|---|---|---|---|---|
| 1.1 | Goal create (individual) | yes | yes | yes | partial (`goal-cycle.create`) | P0 | goal-cycle | both | T1 |
| 1.2 | Goal create (team) | yes | yes | yes | partial | P0 | goal-cycle | both | T1 |
| 1.3 | Goal create (department) | yes | yes | yes | absent | P0 | goal-cycle | paid | T1 |
| 1.4 | Goal create (company) | yes | yes | yes | absent | P0 | goal-cycle | paid | T1 |
| 1.5 | OKR objective-key-result split | yes | yes | yes | absent | P0 | goal-cycle | paid | T1 |
| 1.6 | Goal cascade (org tree) | yes | yes | yes | partial (IP-026 names alignment graph) | P0 | goal-cycle | paid | T1 |
| 1.7 | Goal alignment graph view | yes | partial | yes | partial (IP-026) | P0 | goal-cycle | paid | T1 |
| 1.8 | Goal progress check-in (manual %) | yes | yes | yes | partial | P0 | goal-cycle | both | T1 |
| 1.9 | Goal progress auto-update (integration) | yes | partial | partial | absent | P1 | goal-cycle | paid | T2 |
| 1.10 | Goal close (success / partial / missed) | yes | yes | yes | partial | P0 | goal-cycle | both | T1 |
| 1.11 | Goal carryover (annual close) | yes | yes | yes | absent | P0 | goal-cycle | both | T1 |
| 1.12 | Goal commenting + activity feed | yes | yes | yes | absent | P1 | goal-cycle | both | T2 |
| 1.13 | Goal weighting | yes | yes | yes | absent | P1 | goal-cycle | paid | T2 |
| 1.14 | Goal templates / library | yes | yes | yes | absent | P1 | goal-cycle | both | T1 |
| 1.15 | Goal milestones / sub-goals | yes | yes | yes | absent | P1 | goal-cycle | paid | T2 |
| 1.16 | Goal categories / tags | yes | yes | yes | absent | P2 | goal-cycle | both | T1 |
| 1.17 | Stretch goals | yes | yes | partial | absent | P2 | goal-cycle | paid | T2 |

### 1.2 Goals-surface findings

Row 1.1-1.10 are the canonical Lattice/15Five/Workday Performance shape. P0 across the
HR/Payroll Big-8 elevation. The µservice has `goal-cycle.create/amend/approve/import/export/replay`
listed in PRD FR-001..FR-006 but no decomposition into individual/team/department/company
tiering. PRD must extend FR-001 into FR-001a..FR-001d for the four scope tiers.

Row 1.5 (OKR split) is the structural foundation of the Lattice/15Five marketing pitch.
Workday Performance treats the same primitive as "anytime goals" but supports OKR via the
extended-content pack. The µservice must adopt OKR-shape as a first-class data primitive,
not via free-text or a generic "goal" envelope.

Row 1.6 + 1.7 (cascade + alignment graph) is partially served by IP-026 but the IP is thin
(4 KB). The Lattice cascade view is a high-bar competitive surface; the µservice's IP-026
must thicken to the same operational density as the rest of the IP set (≥12 KB) with full
contract, query model, projection target.

Row 1.11 (carryover) is missing. A goal-cycle that does not carry forward across calendar
boundaries is a feature regression compared to all three vendors. Add `goal-cycle.roll-forward`
to PRD FR set.

## 2. Performance reviews surface

### 2.1 Matrix — review cycles

| # | Capability primitive | V:Lattice | V:15Five | V:Wd Perf | P | G | B | T | C |
|---|---|---|---|---|---|---|---|---|---|
| 2.1 | Annual review cycle | yes | yes | yes | partial (`review-cycle.create`) | P0 | review-cycle | paid | T1 |
| 2.2 | Semi-annual review cycle | yes | yes | yes | partial | P0 | review-cycle | paid | T1 |
| 2.3 | Quarterly review cycle | yes | yes | partial | partial | P0 | review-cycle | paid | T1 |
| 2.4 | Project-based review | partial | yes | yes | absent | P0 | review-cycle | paid | T2 |
| 2.5 | Anytime/ad-hoc review | partial | yes | yes | absent | P0 | review-cycle | paid | T2 |
| 2.6 | Self-review section | yes | yes | yes | absent | P0 | review-cycle | paid | T1 |
| 2.7 | Manager review section | yes | yes | yes | absent | P0 | review-cycle | paid | T1 |
| 2.8 | Skip-level review section | yes | yes | yes | absent | P0 | review-cycle | paid | T2 |
| 2.9 | Peer review section | yes | yes | yes | absent | P0 | review-cycle | paid | T1 |
| 2.10 | Upward review section | yes | yes | yes | absent | P0 | review-cycle | paid | T2 |
| 2.11 | 360-degree review (all directions) | yes | yes | yes | absent | P0 | review-cycle | paid | T2 |
| 2.12 | Customizable review forms | yes | yes | yes | absent | P0 | review-cycle | paid | T1 |
| 2.13 | Question library | yes | yes | yes | absent | P1 | review-cycle | paid | T1 |
| 2.14 | Competency framework | yes | yes | yes | absent | P0 | review-cycle | paid | T2 |
| 2.15 | Competency rating scale | yes | yes | yes | absent | P0 | review-cycle | paid | T1 |
| 2.16 | Numeric rating scale | yes | yes | yes | absent | P0 | review-cycle | paid | T1 |
| 2.17 | Narrative review section | yes | yes | yes | absent | P0 | review-cycle | paid | T1 |
| 2.18 | Review evidence seal (immutable) | partial | partial | yes | present (capability) | n/a | review-cycle | paid | T2 |
| 2.19 | Review draft auto-save | yes | yes | yes | absent | P2 | review-cycle | paid | T1 |
| 2.20 | Review reminder cadence | yes | yes | yes | absent | P1 | review-cycle | paid | T1 |
| 2.21 | Review approval workflow | yes | yes | yes | partial (`review-cycle.approve`) | P1 | review-cycle | paid | T2 |
| 2.22 | Review writing assist (AI) | partial | yes | yes | absent | P2 | review-cycle | paid | T2 |
| 2.23 | Review summary / one-pager | yes | yes | yes | absent | P1 | review-cycle | paid | T2 |
| 2.24 | Review delivery to employee | yes | yes | yes | absent | P0 | review-cycle | paid | T1 |
| 2.25 | Review acknowledgment by employee | yes | yes | yes | absent | P0 | review-cycle | paid | T1 |

### 2.2 Reviews-surface findings

Of 25 review-cycle primitives, the µservice declares 4 (review-cycle.create/amend/approve/
+evidence-seal). Coverage 16%.

Row 2.6-2.11 (review direction sections) are the structural backbone of the review form.
PRD must explode `review-cycle.create` into per-section sub-operations or model sections as
a `review_section` aggregate root.

Row 2.11 (full 360) is the single most-cited Lattice/15Five/Workday Performance differentiator.
A µservice claiming HR/Payroll Big-8 parity without 360 is incomplete.

Row 2.14-2.16 (competency framework + rating scale) is the rating-system backbone. Without
it, calibration (§5) has no input. Author `competency` aggregate root in the ontology.

Row 2.18 (evidence seal) is the µservice's distinctive offering and the only row above
"partial" in the counterparts. This is the place to lead. Workday's audit trail is the
closest counterpart; the µservice's evidence-seal-with-Merkle-root design (per IP-027) is
a differentiator if executed at substance.

## 3. Feedback surface

### 3.1 Matrix — feedback

| # | Capability primitive | V:Lattice | V:15Five | V:Wd Perf | P | G | B | T | C |
|---|---|---|---|---|---|---|---|---|---|
| 3.1 | Praise/recognition wall | yes | yes | partial | absent | P0 | recognition (new) | both | T1 |
| 3.2 | Public praise | yes | yes | partial | absent | P0 | recognition (new) | both | T1 |
| 3.3 | Private praise | yes | yes | yes | absent | P1 | recognition (new) | both | T1 |
| 3.4 | Praise tied to value/competency | yes | yes | yes | absent | P0 | recognition (new) | both | T1 |
| 3.5 | Praise reactions (emojis / hearts) | yes | yes | partial | absent | P2 | recognition (new) | both | T1 |
| 3.6 | Feedback give (peer→peer) | yes | yes | yes | partial (`feedback.create`) | P0 | feedback | both | T1 |
| 3.7 | Feedback give (manager→employee) | yes | yes | yes | partial | P0 | feedback | both | T1 |
| 3.8 | Feedback give (employee→manager) | yes | yes | yes | partial | P0 | feedback | both | T1 |
| 3.9 | Feedback request (pull) | yes | yes | yes | absent | P0 | feedback | both | T1 |
| 3.10 | Feedback templates | yes | yes | yes | absent | P1 | feedback | both | T1 |
| 3.11 | Feedback link to goal | yes | yes | yes | absent | P0 | feedback | both | T1 |
| 3.12 | Feedback link to review | yes | yes | yes | absent | P0 | feedback | both | T1 |
| 3.13 | Anonymous feedback | partial | yes | yes | absent | P1 | feedback | paid | T2 |
| 3.14 | Continuous feedback ingestion | yes | yes | yes | partial (IP-028) | P1 | feedback | both | T1 |
| 3.15 | Feedback abuse / harassment flag | yes | yes | yes | partial (runbook) | P0 | feedback | both | T1 |
| 3.16 | Feedback visibility scope | yes | yes | yes | partial (Cedar policy) | P1 | feedback | both | T1 |

### 3.2 Feedback-surface findings

Recognition/praise (rows 3.1-3.5) is a Lattice/15Five-leader category. Workday Performance
treats it as a lighter capability. The µservice has no recognition surface at all. Add
`recognition` as a 6th bounded context (or fold under `feedback` but with distinct capability
records). The lightweight version is shoutouts; the heavyweight version adds value-tag,
visibility scopes, leaderboards (gamified).

Row 3.9 (pull feedback) is absent. Lattice and 15Five both ship "request feedback from N
people" as a first-class flow. PRD must add `feedback.request` operation.

Row 3.11-3.12 (link to goal / link to review) require the feedback aggregate to carry
foreign-key references to goal_id and review_id. The ontology projection (IP-003) must
encode these edges.

Row 3.15 (abuse flag) has a runbook (manager-feedback-abuse-report.md) but no Cedar policy
guarding the visibility-during-investigation path. Author `local-feedback-flagged-quarantine.cedar`.

## 4. 1:1s and check-ins surface

### 4.1 Matrix — 1-on-1s and check-ins

| # | Capability primitive | V:Lattice | V:15Five | V:Wd Perf | P | G | B | T | C |
|---|---|---|---|---|---|---|---|---|---|
| 4.1 | 1-on-1 meeting agenda | yes | yes | partial | absent | P0 | 1-on-1 (new) | both | T1 |
| 4.2 | Shared 1-on-1 doc (manager+employee) | yes | yes | partial | absent | P0 | 1-on-1 (new) | both | T1 |
| 4.3 | 1-on-1 talking points | yes | yes | partial | absent | P0 | 1-on-1 (new) | both | T1 |
| 4.4 | 1-on-1 action items (carry forward) | yes | yes | partial | absent | P0 | 1-on-1 (new) | both | T1 |
| 4.5 | 1-on-1 templates | yes | yes | partial | absent | P1 | 1-on-1 (new) | both | T1 |
| 4.6 | 1-on-1 question suggestions | yes | yes | absent | absent | P2 | 1-on-1 (new) | both | T1 |
| 4.7 | Weekly check-in (15Five-class) | partial | yes | absent | absent | P0 | weekly-check-in (new) | both | T1 |
| 4.8 | Check-in priorities / wins | partial | yes | absent | absent | P0 | weekly-check-in (new) | both | T1 |
| 4.9 | Check-in mood / pulse | partial | yes | absent | absent | P1 | weekly-check-in (new) | both | T1 |
| 4.10 | Check-in roll-up to manager dashboard | partial | yes | absent | absent | P1 | weekly-check-in (new) | both | T2 |
| 4.11 | Check-in roll-up to skip-level | partial | yes | absent | absent | P2 | weekly-check-in (new) | both | T2 |
| 4.12 | Strivescore (15Five mood index) | absent | yes | absent | absent | P2 | weekly-check-in (new) | paid | T2 |

### 4.2 1-on-1 / check-in surface findings

The µservice has zero coverage of 1-on-1 and weekly check-in surfaces. 15Five's flagship
"weekly check-in" defined the category; Lattice replicated it. Workday Performance is the
weak counterpart here. Add two new bounded contexts: `1-on-1` and `weekly-check-in`. Author
capability YAMLs, IPs, Cedar policies, SLOs, runbooks for each.

This is a single high-leverage Phase D batch: ~12 net-new capability records to close
rows 4.1-4.12.

## 5. Calibration surface

### 5.1 Matrix — calibration

| # | Capability primitive | V:Lattice | V:15Five | V:Wd Perf | P | G | B | T | C |
|---|---|---|---|---|---|---|---|---|---|
| 5.1 | Calibration session (manager group) | yes | yes | yes | partial (`calibration.create`) | P0 | calibration | paid | T2 |
| 5.2 | Calibration distribution view | yes | yes | yes | partial | P0 | calibration | paid | T2 |
| 5.3 | Calibration forced distribution | partial | partial | yes | absent | P1 | calibration | paid | T2 |
| 5.4 | Calibration rating drag-drop | yes | yes | yes | absent | P0 | calibration | paid | T2 |
| 5.5 | Calibration audit log | yes | yes | yes | partial (IP-027) | P0 | calibration | paid | T2 |
| 5.6 | Calibration lock (post-decision) | yes | yes | yes | partial (Cedar policy) | P1 | calibration | paid | T2 |
| 5.7 | Calibration fairness analytics | yes | yes | yes | partial (IP-027) | P0 | calibration | paid | T2 |
| 5.8 | Calibration diversity slice | yes | yes | yes | absent | P0 | calibration | paid | T2 |
| 5.9 | Calibration manager-vs-final delta | yes | yes | yes | absent | P1 | calibration | paid | T2 |
| 5.10 | Calibration outcome publish | yes | yes | yes | partial | P1 | calibration | paid | T2 |
| 5.11 | 9-box grid (performance × potential) | partial | partial | yes | absent | P0 | succession (new) | paid | T2 |
| 5.12 | Calibration → compensation handoff | yes | yes | yes | partial (IP-030) | P0 | calibration | paid | T2 |
| 5.13 | Talent rating (consistent / high / hipo) | partial | yes | yes | absent | P0 | succession (new) | paid | T2 |

### 5.2 Calibration-surface findings

Calibration is one of the better-covered surfaces in the current µservice. IP-027 names the
fairness ledger; IP-030 names the compensation handoff. Coverage 5/13 = 38%.

Row 5.4 (drag-drop) is the canonical Workday Performance UX — must surface as a structured
API command (`calibration.move-employee`) even if the UX is rendered by a separate frontend.

Row 5.7+5.8 (fairness + diversity slice) is the EU-worker-council + US-labor pack-overlay
relevance. The µservice already has compliance pack overlays; the calibration fairness
analytics must consume the pack-overlay configuration to choose what diversity dimensions
to surface.

Row 5.11 (9-box) is succession-planning surface, not calibration proper. Move under a new
`succession` bounded context.

## 6. Engagement and surveys surface

### 6.1 Matrix — engagement

| # | Capability primitive | V:Lattice | V:15Five | V:Wd Perf | P | G | B | T | C |
|---|---|---|---|---|---|---|---|---|---|
| 6.1 | Engagement survey (annual full) | yes | yes | yes | partial (`engagement-survey.create`) | P0 | engagement-survey | paid | T1 |
| 6.2 | Pulse survey (quarterly / weekly) | yes | yes | partial | partial (capability) | P0 | engagement-survey | paid | T1 |
| 6.3 | eNPS (employee NPS) | yes | yes | yes | absent | P0 | engagement-survey | paid | T1 |
| 6.4 | Anonymity guard (k-anon threshold) | yes | yes | yes | partial (IP-029) | P0 | engagement-survey | paid | T2 |
| 6.5 | Question library / templates | yes | yes | yes | absent | P1 | engagement-survey | paid | T1 |
| 6.6 | Likert scale (1-5, 1-7) | yes | yes | yes | absent | P0 | engagement-survey | paid | T1 |
| 6.7 | Open-ended response | yes | yes | yes | absent | P0 | engagement-survey | paid | T1 |
| 6.8 | NLP sentiment on open-ended | partial | yes | yes | absent | P2 | engagement-survey | paid | T2 |
| 6.9 | Heatmap (department × question) | yes | yes | yes | absent | P0 | engagement-survey | paid | T2 |
| 6.10 | Driver analysis | yes | yes | yes | absent | P1 | engagement-survey | paid | T2 |
| 6.11 | Benchmark (internal trend) | yes | yes | yes | absent | P1 | engagement-survey | paid | T1 |
| 6.12 | Benchmark (external industry) | partial | yes | yes | absent | P2 | engagement-survey | paid | T2 |
| 6.13 | Action planning from results | yes | yes | yes | absent | P1 | engagement-survey | paid | T2 |
| 6.14 | Survey reminder cadence | yes | yes | yes | absent | P2 | engagement-survey | paid | T1 |
| 6.15 | Survey embargo / blackout | yes | yes | yes | absent | P1 | engagement-survey | paid | T2 |

### 6.2 Engagement-surface findings

Coverage 2/15 = 13% (excluding the half-credit on engagement-survey.create). The µservice
declares `engagement-pulse` capability but no full survey lifecycle.

Row 6.3 (eNPS) is the universally-cited engagement KPI. Mandatory.

Row 6.4 (anonymity guard) is well-served by IP-029 in concept but lacks Cedar enforcement
(coherence audit finding 1.7.A).

Row 6.9 (heatmap) is a Glint-pioneered visualization the entire engagement-survey category
now expects. Author the projection contract.

## 7. Succession and talent management surface

### 7.1 Matrix — succession

| # | Capability primitive | V:Lattice | V:15Five | V:Wd Perf | P | G | B | T | C |
|---|---|---|---|---|---|---|---|---|---|
| 7.1 | Talent card (per-employee profile) | partial | partial | yes | absent | P0 | succession (new) | paid | T2 |
| 7.2 | Talent rating (current vs potential) | partial | partial | yes | absent | P0 | succession (new) | paid | T2 |
| 7.3 | Successor identification | absent | absent | yes | absent | P0 | succession (new) | paid | T2 |
| 7.4 | Successor readiness rating | absent | absent | yes | absent | P0 | succession (new) | paid | T2 |
| 7.5 | Talent pool grouping | absent | absent | yes | absent | P0 | succession (new) | paid | T2 |
| 7.6 | Mentorship matching | absent | absent | yes | absent | P1 | succession (new) | paid | T2 |
| 7.7 | Career mobility / internal moves | partial | partial | yes | absent | P1 | succession (new) | paid | T2 |
| 7.8 | Development plan | yes | yes | yes | absent | P0 | succession (new) | both | T1 |
| 7.9 | Development plan goal linkage | yes | yes | yes | absent | P0 | succession (new) | both | T1 |
| 7.10 | Learning content recommendation | partial | partial | yes | absent | P1 | succession (new) | paid | T2 |

### 7.2 Succession-surface findings

Workday Performance's strongest differentiator vs Lattice + 15Five is the succession +
talent-management depth. The µservice has zero coverage. Author `succession` bounded context
and the 10 capability records.

This is the strategic high-bar surface — the area where Workday Performance leads. If
performance-management aims at the enterprise B2B-leader segment, succession is mandatory.

## 8. Manager toolbox and analytics surface

### 8.1 Matrix — manager tools

| # | Capability primitive | V:Lattice | V:15Five | V:Wd Perf | P | G | B | T | C |
|---|---|---|---|---|---|---|---|---|---|
| 8.1 | Manager dashboard | yes | yes | yes | absent | P0 | manager-tools (new) | paid | T2 |
| 8.2 | Team goal roll-up | yes | yes | yes | absent | P0 | manager-tools (new) | paid | T2 |
| 8.3 | Team review-cycle status | yes | yes | yes | absent | P0 | manager-tools (new) | paid | T2 |
| 8.4 | Team feedback volume | yes | yes | yes | absent | P0 | manager-tools (new) | paid | T2 |
| 8.5 | Team engagement score | yes | yes | yes | absent | P0 | manager-tools (new) | paid | T2 |
| 8.6 | Manager-direct-report relationship view | yes | yes | yes | absent | P0 | manager-tools (new) | paid | T2 |
| 8.7 | Manager calibration history | yes | yes | yes | absent | P1 | manager-tools (new) | paid | T2 |
| 8.8 | Org-tree navigation | yes | yes | yes | absent | P0 | manager-tools (new) | paid | T2 |
| 8.9 | Skip-level visibility | yes | yes | yes | absent | P1 | manager-tools (new) | paid | T2 |
| 8.10 | Workforce analytics (turnover, tenure) | partial | yes | yes | absent | P1 | manager-tools (new) | paid | T2 |
| 8.11 | Manager nudge / prompt | yes | yes | partial | absent | P2 | manager-tools (new) | paid | T2 |
| 8.12 | Manager certification / training | absent | absent | yes | absent | P2 | manager-tools (new) | paid | T2 |

### 8.2 Manager-toolbox findings

Zero coverage. Manager toolbox is the UX projection layer of all the data the µservice
already (mostly) holds. The contract surface must expose query-aggregate operations:
`manager.team-status`, `manager.team-goal-roll-up`, etc.

## 9. Mobile and integration surface

### 9.1 Matrix — mobile + integration

| # | Capability primitive | V:Lattice | V:15Five | V:Wd Perf | P | G | B | T | C |
|---|---|---|---|---|---|---|---|---|---|
| 9.1 | iOS native app | yes | yes | yes | absent (frontend separate) | P1 | n/a (frontend) | both | T1 |
| 9.2 | Android native app | yes | yes | yes | absent (frontend separate) | P1 | n/a (frontend) | both | T1 |
| 9.3 | Mobile-optimized web | yes | yes | yes | absent (frontend separate) | P2 | n/a (frontend) | both | T1 |
| 9.4 | Push notification cadence | yes | yes | yes | absent | P1 | n/a (notification substrate) | both | T1 |
| 9.5 | Slack integration | yes | yes | partial | absent | P1 | workplace-integration | paid | T2 |
| 9.6 | Microsoft Teams integration | yes | yes | yes | absent | P1 | workplace-integration | paid | T2 |
| 9.7 | Email digest | yes | yes | yes | absent | P1 | workplace-integration | both | T1 |
| 9.8 | Calendar integration (1-on-1s) | yes | yes | partial | absent | P1 | workplace-integration | both | T1 |
| 9.9 | HRIS sync (Workday, BambooHR, etc.) | yes | yes | native | absent | P1 | workplace-integration | paid | T2 |
| 9.10 | ATS sync (Greenhouse, Lever) | partial | partial | partial | absent | P2 | workplace-integration | paid | T2 |
| 9.11 | SSO (SAML, OIDC) | yes | yes | yes | inherited (identity µservice) | n/a | identity | paid | T1 |
| 9.12 | SCIM provisioning | yes | yes | yes | inherited | n/a | identity | paid | T1 |
| 9.13 | API access (REST) | yes | yes | yes | present (openapi-v1.yaml) | n/a | rest | paid | T1 |
| 9.14 | API access (GraphQL) | partial | partial | partial | absent | P2 | rest | paid | T2 |
| 9.15 | Webhook | yes | yes | yes | partial (asyncapi) | P1 | rest | paid | T1 |

### 9.2 Mobile/integration findings

Mobile clients live outside `microservices/performance-management/` per per-µservice flat
layout (frontend/ios, frontend/android). The µservice's responsibility is the mobile-friendly
contract — pagination, low-bandwidth-encodings, offline-capable. Author a `mobile-protocol-
binding.md` or fold into IP-019 sdk-client-generation.

Workplace integrations (Slack/Teams/email/calendar) are owned by `workplace-integration`
µservice but performance-management must declare the event-envelopes that flow.

HRIS sync (row 9.9) is bidirectional with people-records µservice. Authored as part of
HR-family edge B-3 in the coherence audit.

## 10. Compliance, security, and privacy surface

### 10.1 Matrix — compliance

| # | Capability primitive | V:Lattice | V:15Five | V:Wd Perf | P | G | B | T | C |
|---|---|---|---|---|---|---|---|---|---|
| 10.1 | SOC 2 Type II | yes | yes | yes | present (pack) | n/a | governance | paid | T1 |
| 10.2 | ISO 27001 | yes | partial | yes | present | n/a | governance | paid | T1 |
| 10.3 | GDPR | yes | yes | yes | present | n/a | governance | paid | T1 |
| 10.4 | KR-PIPA | partial | partial | yes | present | n/a | governance | paid | T2 |
| 10.5 | EU worker council | partial | partial | yes | present | n/a | governance | paid | T2 |
| 10.6 | US labor law | partial | partial | yes | present | n/a | governance | paid | T1 |
| 10.7 | HIPAA | partial | partial | yes | present (pack listed) | n/a | governance | paid | T2 |
| 10.8 | Data residency (EU / US / KR) | yes | yes | yes | present (IP-015) | n/a | governance | paid | T2 |
| 10.9 | DSAR (right of access) | yes | yes | yes | partial | P1 | governance | paid | T1 |
| 10.10 | Right to erasure | yes | yes | yes | partial | P1 | governance | paid | T1 |
| 10.11 | DPIA artifact | yes | yes | yes | present (dpia.md) | n/a | governance | paid | T1 |
| 10.12 | Threat model artifact | yes | yes | yes | present (threat-model.md) | n/a | governance | paid | T1 |
| 10.13 | Audit log (immutable) | yes | yes | yes | present (IP-011) | n/a | governance | paid | T1 |
| 10.14 | Audit export | yes | yes | yes | present (compliance.md) | n/a | governance | paid | T1 |

### 10.2 Compliance-surface findings

Compliance is the µservice's strongest declared surface. Pack overlays well documented.
Two P1 gaps: DSAR + erasure must thread through the goal-cycle, review-cycle, feedback,
engagement-survey, calibration aggregates with explicit per-aggregate erasure workflows
(some content is immutable evidence — needs nuanced erasure-with-redaction).

## 11. Aggregate coverage summary

### 11.1 By bounded context

| Bounded context | Total primitives | Present | Coverage |
|---|---|---|---|
| goal-cycle | 17 | 6 (35%) | 6/17 |
| review-cycle | 25 | 4 (16%) | 4/25 |
| feedback | 16 | 3 (19%) | 3/16 |
| recognition (new) | 5 | 0 | 0/5 |
| 1-on-1 (new) | 6 | 0 | 0/6 |
| weekly-check-in (new) | 6 | 0 | 0/6 |
| calibration | 13 | 5 (38%) | 5/13 |
| succession (new) | 10 | 0 | 0/10 |
| engagement-survey | 15 | 2 (13%) | 2/15 |
| manager-tools (new) | 12 | 0 | 0/12 |
| mobile/integration | 15 | 5 (33%, partial-credit) | 5/15 |
| compliance | 14 | 12 (86%) | 12/14 |
| **Total** | **154** | **37 (24%)** | **37/154** |

### 11.2 By coverage tier

- Present (full or partial): 37/154 = 24%
- Absent: 117/154 = 76%
- Gap-to-85%-floor (BIG-8 P0 mandate): 94 net-new capabilities required

### 11.3 Sub-summary discrepancy with coherence audit

The coherence audit §5.4 cited 37% coverage against a 38-primitive surface (high-level
union). This finer-grained matrix counts 154 distinct primitives at 24%. Both summaries
indicate the same direction: the µservice has a deep gap and is below the BIG-8 P0
promotion floor.

## 12. Recommended buildout sequence

### 12.1 Phase D.1 — high-leverage missing primitives (next 30 days)

Author capability YAMLs, PRD acceptance criteria, IPs, contracts, Cedar policies, SLOs, and
runbooks for:

- 1-on-1 bounded context (rows 4.1-4.6): 6 capabilities
- weekly-check-in bounded context (rows 4.7-4.12): 6 capabilities
- recognition bounded context (rows 3.1-3.5): 5 capabilities
- review-section decomposition (rows 2.6-2.11): 6 capabilities
- eNPS + survey anonymity Cedar (rows 6.3-6.4): 2 capabilities + 1 Cedar policy
- goal-cycle.roll-forward (row 1.11): 1 capability + 1 runbook

Phase D.1 batch size: 26 capability records.

### 12.2 Phase D.2 — calibration + succession depth (days 30-60)

- 9-box / talent-card / talent-rating (rows 5.11, 5.13, 7.1, 7.2): 4 capabilities
- successor identification + readiness + pool (rows 7.3-7.5): 3 capabilities
- competency framework (rows 2.14-2.16): 3 capabilities

Phase D.2 batch size: 10 capability records.

### 12.3 Phase D.3 — manager toolbox + integration (days 60-90)

- Manager dashboard + roll-ups (rows 8.1-8.6, 8.8): 7 capabilities
- HRIS sync + workplace integration envelopes (rows 9.5-9.9): 5 capabilities
- Mobile-protocol-binding (rows 9.1-9.4): 1 capability + binding doc

Phase D.3 batch size: 13 capability records.

### 12.4 Phase D.4 — engagement survey depth (days 90-120)

- Likert + open-ended + heatmap (rows 6.6-6.9): 4 capabilities
- Driver + benchmark + action planning (rows 6.10-6.13, 6.15): 5 capabilities
- Survey question library (row 6.5): 1 capability

Phase D.4 batch size: 10 capability records.

### 12.5 Phase D.5 — review-cycle depth (days 120-150)

- Anytime + project + cycle variants (rows 2.2-2.5): 4 capabilities
- Customizable forms + question library (rows 2.12-2.13): 2 capabilities
- Acknowledgment + delivery + reminder + approval (rows 2.20-2.21, 2.23-2.25): 5 capabilities

Phase D.5 batch size: 11 capability records.

### 12.6 Phase D.6 — long-tail (days 150-180)

- Remaining rows from §1.1, §2.1, §3.1, §5.1, §10.1: ~14 capabilities

Phase D.6 batch size: 14 capability records.

### 12.7 Phase D total

26 + 10 + 13 + 10 + 11 + 14 = 84 new capability records over 6 months. Aligned with
`feedback_go_with_original_ambition_2026_05_20`. End state: 121/154 = 78% coverage, just
below the 85% BIG-8 floor. Final 13 capabilities (selected based on Phase E review) close
the gap.

## 13. Tenant-class overlay across the matrix

### 13.1 Demo_trial tenant scope

A demo_trial tenant of `performance-management` must be able to exercise:
- Goal create individual (1.1) + Goal close (1.10) — to demonstrate the core flow
- Feedback give peer→peer (3.6) — to demonstrate the feedback flow
- Review cycle annual (2.1) with one section (2.6, 2.7) — to demonstrate the review flow
- Engagement pulse (6.2) at limited cadence — to demonstrate the engagement flow
- Manager dashboard (8.1) read-only — to demonstrate the visibility flow

Demo_trial scope must NOT expose: calibration (5.x), succession (7.x), HRIS sync (9.9),
data residency overrides (10.8), DSAR self-service (10.9). These are paid-only.

### 13.2 Paid tenant scope

Full surface. Optional billing components decompose the surface into a la carte purchases:
- bc-performance-management-core: goals + reviews + feedback + 1-on-1 + check-in
- bc-performance-management-calibration: calibration + competency framework + fairness
- bc-performance-management-succession: succession + talent cards + 9-box
- bc-performance-management-engagement: engagement surveys + eNPS + heatmap + analytics
- bc-performance-management-manager-toolbox: manager dashboards + roll-ups

Manifest must declare these billing-components.

### 13.3 Cell-tier overlay

T1-eligible primitives are mostly individual-employee facing (goals, feedback, 1-on-1,
review section, eNPS). T2-eligible primitives require richer compute (cascade graph,
calibration, heatmap, succession). T0 is platform-substrate only (out of scope). T3 and T4
add geographic-residency overlays and sovereign-cloud pack engagement.

## 14. Closure

This matrix supersedes the existing `competitor-parity-matrix.md` (which is template-
stamped per coherence-audit §3.2.A). The existing file should be rewritten with this
matrix's substance plus updated formatting for the rest of the doc-set (header
references, mandatory sections per microservice-template).

Total industry-counterpart primitives in scope: 154.
Current coverage: 24%.
BIG-8 P0 promotion floor: 85%.
Buildout: 84+ net-new capabilities across 6 months.

End of feature-parity matrix.
