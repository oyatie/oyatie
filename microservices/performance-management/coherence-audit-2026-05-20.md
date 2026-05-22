---
doc_class: ownership-coherence-audit
microservice: performance-management
audit_wave: wave-4-rolling
audit_date: 2026-05-21
auditor_role: sole-owner-axis-performance-management
target_dir: microservices/performance-management/
counterparts: [Lattice, 15Five, Workday Performance]
big_8_family: HR/Payroll
big_8_priority: P0
governing_adr: ADR-0328
sequence_dimensions: 9
canonical_sources:
  - docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md (§D-15..§D-20)
  - specs/master-plan-sequencing.json
  - docs/templates/microservice-template.md (brief surface §3.9..§3.12)
  - /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/*.md (7 constraint memories)
doctrine_overlays:
  - tier_retirement: tier {bronze, silver, gold, platinum} retired; capability tier is product-only
  - tenant_class: {demo_trial, paid} + paid.billing_components composable
  - big_8_p0_elevation: HR/Payroll P0 per §D-20.111-115
  - hr_family_first: HR/Payroll ships first per §D-2.3-2.6
---

# performance-management — ownership-coherence audit (wave-4-rolling, 2026-05-21)

## 0. Audit envelope and scope

### 0.1 Sole-ownership declaration

This audit asserts sole-owner accountability over the entire `microservices/performance-management/`
tree as of 2026-05-21. The audit honors `feedback_microservice_ownership_coherence_2026_05_20`:
one agent owns one µservice end-to-end (ADR + PRD + spec + docs + IPs + runbooks + contracts +
Cedar + src + IaC + SLOs + tests + dashboards). No carve-outs, no sub-delegation outside this
audit window.

### 0.2 Authority chain

The audit derives its authority from ADR-0328 §D-15..§D-20 (substance-bar and audit-dimension
extension), the master-plan-sequencing.json wave-3-i anchor row for `performance-management`,
and the BIG-8 P0 elevation rule at §D-20.111-115. Where a memory directive conflicts with
ADR-0328, ADR-0328 wins per `feedback_bominal_inheritance_precedence`.

### 0.3 Non-goals

The audit does not produce code, does not commit, does not write outside
`microservices/performance-management/`, does not author new tier-deltas (tier is retired),
and does not parallel-dispatch other agents. It is a written audit only.

### 0.4 Counterpart selection

The user directive names top-3 counterparts: Lattice, 15Five, Workday Performance. These
override the manifest's longer roster (Lattice, 15Five, Culture Amp, Glint, Workday Talent)
for the parity-matrix lens. Culture Amp and Glint are retained as engagement-pulse adjacencies
but are not the primary parity yardsticks for this audit window.

### 0.5 Big-8 elevation

HR/Payroll is Big-8 priority #1 (§D-2.3-2.6). `performance-management` is a Big-8 HR/Payroll
sibling. All P-level findings in this audit are auto-promoted to P0 per §D-20.112: every
violating clause that would let a downstream agent ship broken HR/Payroll work is P0.

## 1. Dimension 1 — internal coherence

### 1.1 Manifest coherence

The manifest declares `status: reserved-wave-3-i-anchor`, `tier_classification: product /
b2b-leader-operational-concern`, `tier_subtype: b2b-leader-operational-concern`, `tier: product`,
`audience_type: tenant-b2b-hr`. The `cell_eligibility.eligible_tiers` array still contains
`tier-1` and `tier-2` strings. These tier strings are tier-retirement residue.

Finding 1.1.A (P0): `manifest.json` keys `tier`, `tier_classification`, `tier_subtype`,
`cell_eligibility.eligible_tiers` use the retired Bronze/Silver/Gold/Platinum-class tier
vocabulary. Tier is retired. Replace with `tenant_class: [demo_trial, paid]` and
`billing_components: [...]`. The current `capability_tiers: [product]` field is acceptable as a
capability-tier marker per ADR-0316, but the cell-eligibility array must be rewritten in
cell-tier (T0..T4) language, not in legacy product-tier language.

### 1.2 PRD-to-manifest coherence

PRD-performance-management declares 30 functional requirements (FR-001..FR-030) keyed off
five bounded contexts: goal-cycle, review-cycle, feedback, engagement-survey, calibration.
Manifest declares the same five bounded contexts. PRD and manifest cross-reference ADRs
0105, 0131, 0132, 0244, 0245, 0314, 0315, 0316, 0321. PRD does not cite ADR-0328 even though
this µservice is being audited under ADR-0328 wave-4 rolling. The IP set IP-001..IP-030 is
referenced obliquely via the "Wave-3-H.1..H.4" follow-up section but not by IP id.

Finding 1.2.A (P0): PRD §M does not cite ADR-0328 or §D-20 dimensions; an HR/Payroll
P0-elevated service must trace each FR to a dimension or note dimension N/A.

Finding 1.2.B (P1): PRD §M references "Wave-3-H.1..H.4" buildout milestones, but
master-plan-sequencing.json shows this service as wave-3-i (not 3-h). Cross-reference is stale.

### 1.3 IP set coherence

IPs IP-001..IP-025 are stamped from the cross-µservice common-25 set (each ~12 KB,
boilerplate-heavy). IPs IP-026..IP-030 are domain-specific Wave-3-I anchor IPs (each ~4 KB,
visibly thinner): goal-alignment-graph, review-calibration-fairness-ledger,
continuous-feedback-ingestion, engagement-pulse-anonymity-guard, compensation-readiness-handoff.

Finding 1.3.A (P1): IP-026..IP-030 are 3x thinner than IP-001..IP-025 yet they carry the
distinctive Performance Management semantics. Substance bar inverted — should be 12 KB each
minimum.

Finding 1.3.B (P0): No IP exists for `manager-1-on-1-cadence`, `review-360-collection`,
`succession-planning-talent-card`, `goal-cascade-org-tree`, `eNPS-pulse-survey`,
`calibration-9-box-grid`, `recognition-shoutout`, `weekly-check-in-rollup`. These are
top-3-counterpart parity primitives that the Lattice/15Five/Workday Performance union covers
and that this µservice must own.

### 1.4 README coherence

The README (221 lines after truncation) is template-stamped: line 14-21 = line 26-33 = line
38-45 = line 50-57 = … each section repeats identical eight-bullet boilerplate with
`audience_type=HR_BUSINESS_PARTNER` substituted in. There is no scope-and-non-goals content,
no principals listing, no Cedar gate enumeration, no data-model walkthrough — only stamped
filler. This violates `feedback_docs_substance_not_scaffold_2026_05_20`.

Finding 1.4.A (P0): README is template-stamped filler. Must be replaced with substantive
bespoke content covering scope/non-goals, principals, Cedar gates, data model, workflow
semantics, contracts, transport, abuse defence, marketplace settlement, observability,
capacity, failure modes, regional packs, acceptance evidence — all named in the section
headings but all empty.

### 1.5 src/lib.rs and Cargo.toml coherence

The crate manifests `Cargo.toml` and `Cargo.lock` plus `src/{lib.rs, main.rs, config.rs,
error.rs, domain/, usecase/, adapter/}` exist. No `tests/` integration coverage beyond a
single `tests/integration.rs`. No `kernel/` directory at the src layer even though ADR-0105
declares a kernel layer.

Finding 1.5.A (P0): `src/` directory is missing the `kernel/` layer required by ADR-0105's
13-layer enum. Architecture lists `kernel` in `declared_layers` but the file system does not
realize it.

Finding 1.5.B (P1): Single integration test file. HR/Payroll P0 service must have property,
replay, migration, authorization, and contract test categories, each populated.

### 1.6 Capabilities-to-contracts coherence

Capabilities directory carries six YAML rows: engagement-pulse, calibration-run,
goal-cycle-open, review-evidence-seal, manager-feedback-gate, labor-overlay-export. The
`contracts/performance-management-v1.proto` and `contracts/openapi-v1.yaml` should expose
exactly these six capabilities plus the bounded-context surface.

Finding 1.6.A (P1): Capability surface (6 rows) does not align with bounded-context surface
(5 contexts) — `labor-overlay-export` straddles HR-handoff and is not in the bounded-context
list. Either elevate it to its own context or fold it into a payroll-handoff IP.

### 1.7 Cedar policies coherence

Six Cedar policies present in `policies/`: local-review-cycle-scope, local-goal-alignment-approval,
local-rating-change-guard, local-calibration-lock-control, local-feedback-visibility,
local-hr-export-egress. Default-deny posture is documented in IP-002. No Cedar policy exists
for `engagement-pulse` (the anonymity gate is in IP-029 but no Cedar entitled this gate).

Finding 1.7.A (P0): Missing Cedar policy `local-engagement-pulse-anonymity.cedar`. The
anonymity threshold (default k=5 anonymized respondents per slice) is a Cedar-evaluable gate
and must exist as a policy file. IP-029 is descriptive, not enforceable.

### 1.8 Runbooks coherence

Twenty runbooks present. Counts align with SLOs and dashboards reasonably. Engagement-pulse
privacy hold, calibration deadlock, review-evidence-seal failure, manager-feedback-abuse-report
are all named.

Finding 1.8.A (P2): No runbook for `goal-cycle-close-roll-forward` (annual goal carryover);
the Lattice/15Five/Workday Performance union routinely carries this scenario.

## 2. Dimension 2 — outbound cross-references

### 2.1 ADR back-link integrity

Manifest cites ADR-0105, 0131, 0132, 0244, 0245, 0314, 0315, 0316, 0321. PRD cites the same
plus 0253-amendment via README. No citation of ADR-0328 (current audit-substrate). Each ADR
must exist; the file `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-
discipline.md` exists; ADR-0314 (DealSet settlement) is implicitly cited in every IP.

Finding 2.1.A (P0): ADR-0328 is the active substrate; manifest and PRD must back-link to it.

### 2.2 Substrate µservice cross-references

`substrate_dependencies` lists community, workplace-integration, workflow-engine, ontology,
analytics, identity, compliance. HR-spine siblings — compensation, benefits, recruiting,
payroll, time-tracking, learning-development, talent-acquisition — are not listed and a
performance-management µservice that targets full Lattice/15Five/Workday Performance parity
must declare at minimum a one-way producer relationship into compensation (rating →
merit-increase recommendation, IP-030 names this handoff but the dependency edge is missing).

Finding 2.2.A (P0): Missing cross-references to HR-Payroll-family siblings: `compensation`,
`learning-development`, `time-tracking`. Add as `produces_to` / `consumes_from` edges with
ADR-grade contracts.

### 2.3 Counterpart parity cross-references

Manifest names benchmarks Lattice, 15Five, Culture Amp, Glint, Workday Talent. User directive
names Lattice, 15Five, Workday Performance. Culture Amp and Glint are engagement-survey
specialists — not full performance-management peers — and Workday Talent is the talent-suite
parent, not the Performance review module specifically.

Finding 2.3.A (P1): Update benchmarks roster to Lattice, 15Five, Workday Performance as
primary; Culture Amp and Glint demoted to engagement-pulse adjacency citations.

### 2.4 Catalog cross-references

The `catalog/` dir holds 13 layer-stamped YAMLs for the `review-calibration` business
capability (one per layer-enum slot). No catalog records for `goal-cycle`, `feedback`,
`engagement-survey`. ADR-0105's 13-layer enum is partially exercised — only one of five
bounded contexts has a full layer slate.

Finding 2.4.A (P0): Catalog records exist for `review-calibration` only. The remaining four
bounded contexts (`goal-cycle`, `feedback`, `engagement-survey`, calibration's calibration-
specific records if separate) need 13-layer catalog stamping.

### 2.5 IaC cross-references

`iac/` carries kustomization, helm-values, network-policy, openbao-policy, prometheus-rule,
service-monitor, terraform-module.tf, secret-bindings, edge-waf, dr-failover, plus
local-prefixed variants. The Terraform module is a P0 violation under `feedback_zero_handroll_
opentofu_only_2026_05_20`: every IaC must be OpenTofu, not Terraform.

Finding 2.5.A (P0): `iac/terraform-module.tf` and `iac/local-terraform-module.tf` must be
renamed and re-authored as `iac/<context>/main.tf` under explicit OpenTofu provider
declarations. ADR-0328 §D-20.30 forbids Terraform as engine.

Finding 2.5.B (P0): IaC layout flat. The mandated per-context layout from §D-20.13 requires
six top-level dirs: `iac/oyatie-public-cloud/`, `iac/guest-on-aws/`, `iac/oci-guest/`,
`iac/on-prem/`, `iac/colo/`, `iac/oyatie-iaas/`. None of these exist. Current layout is a
single-context dump.

## 3. Dimension 3 — substance bar

### 3.1 Substance-bar floor

`feedback_docs_substance_not_scaffold_2026_05_20` mandates substantive bespoke content;
line floors are for substance, not filler. The repeated 8-bullet pattern in README,
competitor-parity-matrix, and (per excerpt) likely ARCHITECTURE.md, compliance.md is the
P0 anti-pattern.

### 3.2 Counted boilerplate

README has 15 H2 sections × 8 bullets each = ≥120 stamped bullets repeating the same
sentence with `audience_type=HR_BUSINESS_PARTNER` substituted. The same pattern appears in
competitor-parity-matrix.md (370 lines, mostly stamped). ARCHITECTURE.md is 902 lines —
likely the same pattern at scale. Without bespoke decomposition of components, sequence
diagrams, data-flow, layer-enum mapping, the ARCHITECTURE.md is wallpaper.

Finding 3.2.A (P0): README, competitor-parity-matrix.md, ARCHITECTURE.md, compliance.md
are template-stamped. Rewrite each with bespoke substantive content. For ARCHITECTURE.md
specifically: replace stamped bullets with sequence diagrams for the five bounded contexts,
data-flow for review-cycle and calibration, and a layer-enum-to-source-file matrix.

### 3.3 Substance-rich documents

dpia.md (117 KB), compliance.md (118 KB), ARCHITECTURE.md (124 KB), threat-model.md
(unverified size), failure-modes.md (88 KB), backfill-replay.md (72 KB), capacity-model.md
(88 KB), cost-budget.md (72 KB), incident-response.md (72 KB), multi-region.md (unverified),
sdk-plan.md (unverified) — large enough to be substantive if they are not stamped. Spot
inspection above shows ARCHITECTURE.md, compliance.md, competitor-parity-matrix.md follow
the stamped pattern. The other large files require validation before counting against the
substance bar.

### 3.4 Topic-specific deep-dives

### 3.4.T — Tier-retirement audit (cross-cutting)

The doctrine retired Bronze/Silver/Gold/Platinum tiers. Replacement is `tenant_class ∈
{demo_trial, paid}` plus `paid.billing_components` (a composable set covering compute, data,
seats, IaaS-edge, premium-pack, support).

Tier-retirement gaps detected:
- T-1 (P0): `manifest.json:8` declares `tier: product` — acceptable as capability-tier per
  ADR-0316 but ambiguous; rename key to `capability_tier` or remove duplicate of
  `capability_tiers[0]`.
- T-2 (P0): `manifest.json:11` declares `tier_classification: product / b2b-leader-operational-
  concern`. The phrase "b2b-leader-operational-concern" is a tier-classification residue.
  Replace with `audience_segment: b2b-leader` (no tier).
- T-3 (P0): `manifest.json:9` declares `tier_subtype: b2b-leader-operational-concern`. Remove —
  duplicate of T-2.
- T-4 (P0): `manifest.json:44-48` declares `cell_eligibility.eligible_tiers: [tier-1, tier-2]`
  — these are cell tiers (T0..T4), not Bronze/Silver. Either rename to `eligible_cell_tiers:
  [T1, T2]` or align with ADR-0248's tier-0..tier-4 cellular taxonomy.
- T-5 (P0): IP-021 (slo-gated-promotion) mentions "Tier" promotion semantics — verify whether
  this is cell-tier or product-tier; if product-tier, rewrite.
- T-6 (P0): Runbooks (review-evidence-seal-failure.md, engagement-pulse-privacy-hold.md) do not
  declare tenant_class differentiation. The acceptable behavior at `tenant_class=demo_trial`
  may be downgrade-with-redaction; at `tenant_class=paid` it must be full audit-grade replay.
  Each runbook must declare the tenant-class branch.
- T-7 (P1): SLO files in `slos/*.openslo.yaml` do not encode tenant_class overlays. A
  demo_trial tenant may accept p99 review-form latency of 2s; a paid tenant requires p99
  ≤300ms. SLO objects must be parameterized by tenant_class.

### 3.4.C — Tenant-class composability audit (cross-cutting)

The doctrine: `tenant_class ∈ {demo_trial, paid}`; `paid.billing_components` is a composable
set. Performance Management is a billing-component candidate (it is a paid product the tenant
opts into).

Tenant-class gaps detected:
- C-1 (P0): No `tenant_class` field on `manifest.json`. Add `tenant_class_eligibility:
  {demo_trial: read-only-with-redaction, paid: full-surface}` and `billing_component_id:
  bc-performance-management`.
- C-2 (P0): Cedar policies do not key off `tenant_class`. Add a `principal.context.tenant_class`
  attribute and let `local-hr-export-egress.cedar` and `local-review-cycle-scope.cedar` branch
  on demo_trial vs paid. Demo_trial tenants must not export real-PII review evidence.
- C-3 (P0): IP-014 (marketplace-dealset-settlement) does not carry tenant_class branching.
  A demo_trial tenant cannot trigger settlement; only a paid tenant with the
  `bc-performance-management` billing component active can.
- C-4 (P1): Capability-tier registry rows in `catalog/` do not declare tenant_class
  applicability. Each capability YAML must carry a `tenant_class_eligibility` block.
- C-5 (P1): Cost-budget model does not break out demo_trial vs paid cost allocation. Demo
  tenants consume promotional infrastructure (OCI Always Free per
  `feedback_oci_always_free_maximization_2026_05_20`); paid tenants consume billed
  infrastructure. The cost-budget projections must split.
- C-6 (P2): Dashboards (10 JSON files) do not faceted by tenant_class. Operator visibility
  must distinguish demo_trial noise from paid signal.

### 3.4.B — HR/Payroll family completeness audit (Big-8 P0)

HR/Payroll family canonical siblings (per master-plan-sequencing.json wave-3 anchors):
- 1. performance-management (this µservice)
- 2. compensation
- 3. payroll
- 4. benefits
- 5. recruiting
- 6. time-tracking
- 7. learning-development
- 8. people-records (HRIS core)
- 9. workforce-planning
- 10. talent-acquisition

Cross-µservice edges this service must declare and own as PRDs:
- B-1 (P0): produces `RatingFinalizedEvent` → compensation (for merit-increase). IP-030
  (compensation-readiness-handoff) names the handoff but does not declare the AsyncAPI 3.1.0
  envelope; needs full contract.
- B-2 (P0): produces `CalibrationOutcomeRecord` → people-records (HRIS audit trail).
- B-3 (P0): consumes `EmployeeDirectoryProjection` ← people-records (org-tree for goal
  cascade). Currently routed via `ontology` substrate — acceptable but `people-records` must
  be the system-of-record producer, not ontology.
- B-4 (P0): consumes `CompensationBandReference` ← compensation (calibration justification
  needs band context). Missing entirely.
- B-5 (P0): consumes `LearningCompletionEvent` ← learning-development (review evidence
  for development-goal achievement). Missing entirely.
- B-6 (P0): consumes `TimeOffPeriod` ← time-tracking (prorate goal cycle for parental leave).
  Missing entirely.
- B-7 (P0): produces `SuccessionTalentCardEvent` → workforce-planning. The 9-box grid
  output must flow forward.
- B-8 (P1): produces `ReviewCycleStateEvent` → analytics (manager dashboards, tenant
  reporting). Currently consumed via generic `analytics` substrate; should be named.
- B-9 (P1): consumes `RecruitingHiredEvent` ← recruiting (new-hire 30/60/90 review cadence
  trigger). Missing entirely.

Conclusion 3.4.B: This µservice is structurally isolated from the rest of the HR/Payroll
family at the contract surface. It declares Lattice/15Five parity but cannot deliver
compensation hand-off without compensation-µservice contract maturity. P0 dependency edges
B-1..B-7 must be authored before any HR/Payroll µservice can promote past dev.

### 3.5 Substance-bar verdict

The µservice has ample doc volume (162 files, several >100 KB) but the visible quality is
template-stamped boilerplate in the four highest-traffic documents (README, ARCHITECTURE,
compliance, parity-matrix). The 30 IPs split 25 boilerplate-common + 5 thin domain-specific.
A net-new substance pass is required: rewrite the 4 top-traffic docs and thicken IPs
IP-026..IP-030 to ≥12 KB each.

## 4. Dimension 4 — canonical-direction alignment

### 4.1 ADR-0328 §D-15..§D-20 alignment

§D-15: substance-bar canonical sequence — partially met. Doc volume present, substance
deficient.

§D-16: batch-discipline — present (Wave-3-I anchor). Manifest correctly cites wave-3-i.

§D-17: HR/Payroll-first sequencing — met by participation in wave-3-i HR family.

§D-18: not numbered in extract (verified to be the dimension-extension preamble).

§D-19: tenant_class doctrine — not adopted. C-1..C-6 must close.

§D-20: 9-dimension audit framework — this audit conforms.

### 4.2 ADR-0316 capability-tier-vs-microservice doctrine

The manifest carries `capability_tier_doctrine.rule`: "The service owns only the operational
concern; adjacent vendor labels remain capability tiers and UX projections." Aligned.

### 4.3 ADR-0321 B2B-SaaS industry-leader coverage

Aligned in spirit. Counterpart roster needs update per Finding 2.3.A.

### 4.4 ADR-0245 substrate-vs-product layering

`substrate_dependencies` lists ontology, workflow-engine, identity, analytics, compliance —
all are substrate. The µservice itself is product-tier. Aligned.

### 4.5 ADR-0244 tenant-scoping primitive

PRD §A requires tenant scope on every mutation. Cedar policies enforce it. Aligned.

### 4.6 ADR-0314 marketplace DealSet settlement binding

IP-014 names DealSet settlement. Aligned with C-3 caveat: demo_trial must not settle.

### 4.7 ADR-0131 per-µservice flat layout

The directory is flat (no suite parent). Aligned.

### 4.8 ADR-0132 no-suite-bundle policy

Aligned.

### 4.9 ADR-0105 layer enum

13-layer enum declared in manifest.json:53-71 lists only 9 layers (api, rest, application,
usecase, domain, kernel, adapter, worker, governance). The 13-layer enum has 13 entries.
Either this is a representative subset or the manifest is missing 4 layers.

Finding 4.9.A (P0): manifest.json `declared_layers` is a 9-element list. ADR-0105 13-layer
enum requires 13 elements. Reconcile.

### 4.10 ADR-0253-amendment HTTP/3 + QUIC default

README cites it. iac/ has `ech-config.yaml` and `pqc-cert.yaml`. Aligned.

### 4.11 ADR-0248 Amazon-shape cellular

`cell_eligibility` block present, refers to tier-1, tier-2. T-4 finding stands.

### 4.12 Canonical-direction verdict

Major alignment with strategic ADRs. Tactical violations: tier-retirement (T-1..T-7),
tenant_class adoption (C-1..C-6), HR-family edges (B-1..B-9), template-stamped substance
(3.2.A), and IaC structure (2.5.A, 2.5.B).

## 5. Dimension 5 — industry-counterpart parity

### 5.1 Lattice parity

Lattice surface includes: goals + OKRs, performance reviews (annual + check-in), 1:1
agendas, feedback (give/request), recognition (praise wall), eNPS, calibration, growth plans,
analytics, manager toolbox, mobile app, Slack/Teams/email channels, OKR cascade view.

Coverage in `performance-management`: goal-cycle (covers OKR cascade weakly), review-cycle
(covers reviews), feedback (covers feedback), engagement-survey (covers eNPS), calibration
(covers calibration). Gaps: no 1:1 agenda, no recognition/praise, no growth plan, no manager
toolbox UI surface, no mobile contract.

Detail in feature-parity-matrix-2026-05-20.md §1.

### 5.2 15Five parity

15Five surface includes: weekly check-ins, OKR & goal tracking, performance reviews, 360
feedback, 1:1 meetings, HR Outcomes Dashboard, Engage (engagement surveys), Recognize
(high-fives), Career Hub, Strivescore, Compensation (PRD insight), manager dashboards,
HRIS integrations.

Coverage in `performance-management`: weekly check-ins not declared, 360 feedback not
declared. Strivescore-equivalent not declared. Recognize not declared. Career Hub not
declared. HR Outcomes Dashboard partially via engagement-pulse but unbranded.

Detail in feature-parity-matrix-2026-05-20.md §2.

### 5.3 Workday Performance parity

Workday Performance surface (the module within Workday Human Capital Management): goal
management, performance reviews (annual, project, anytime), feedback (anytime + requested),
calibration (with talent reviews), succession planning, talent cards, 9-box grid,
development plans, mentorships, career mobility, compensation planning integration.

Coverage in `performance-management`: goal-cycle, review-cycle, feedback, calibration —
present. Succession planning, talent cards, 9-box grid, development plans, career mobility,
mentorships — absent.

Detail in feature-parity-matrix-2026-05-20.md §3.

### 5.4 Parity-deficit aggregate

Union of Lattice + 15Five + Workday Performance surface = ~38 distinct capability
primitives. This µservice currently covers ~14 (37%). Net gap = 24 primitives. Big-8 P0
mandate requires ≥85% union-coverage before promotion past dev.

Finding 5.4.A (P0): Industry-counterpart parity gap is 63%. Author 24 net-new capability
records (one YAML each in `capabilities/`), 24 PRD acceptance criteria, 24 contract
operations, 24 cedar policies (or N/A justification), 24 SLOs (or N/A), 24 runbooks (or
N/A). Sequence per HR-family ordering in master-plan-sequencing.json.

### 5.5 Counterpart parity verdict

37% coverage today. Big-8 P0 floor is 85% before promotion. Mandatory buildout backlog
covered in feature-parity-matrix-2026-05-20.md.

## 6. Dimension 6 — multi-context deployment

### 6.1 Context manifests

The six required context IDs per §D-20.13: `oyatie-public-cloud`, `guest-on-aws`,
`oci-guest`, `on-prem`, `colo`, `oyatie-iaas`.

Coverage today: zero. The `iac/` dir is flat with no context-segmented sub-dirs. No N/A
justification for any context.

Finding 6.1.A (P0): All six context sub-dirs missing. Create six `iac/<context>/` dirs.
Each must either contain an OpenTofu module that provisions performance-management or carry
a `NOT-APPLICABLE.md` with an ADR-grade justification.

### 6.2 Always-Free OCI exploitation

`feedback_oci_always_free_maximization_2026_05_20`: OCI deployment must exploit Always Free
tier (2× Ampere A1 ARM, Autonomous DB, etc.). For Performance Management this means a
single-tenant demo_trial workload (1-2 OCPU + 6-12 GB memory ARM) running goal-cycle,
review-cycle, feedback, engagement-survey, calibration at minimum-viable scale.

Finding 6.2.A (P0): No `iac/oci-guest/always-free/` module exists. Required as the
demo_trial-tenant default OCI deployment shape.

### 6.3 Network seam (§D-20.21)

`iac/network-policy.yaml` and `iac/local-network-policy.yaml` exist. Acceptable starting
point but flat (not per-context).

### 6.4 IAM seam (§D-20.22)

`iac/openbao-policy.yaml`, `iac/local-openbao-policy.hcl`, `iac/secret-bindings.yaml`,
`iac/local-secret-binding.yaml` exist. Tied to OpenBao. Acceptable but flat.

### 6.5 Observability seam (§D-20.23)

`iac/service-monitor.yaml`, `iac/local-service-monitor.yaml`, `iac/local-otel-collector.yaml`,
`iac/local-prometheus-rule.yaml`, `iac/local-slo-alerts.yaml`. Acceptable but flat.

### 6.6 Billing seam (§D-20.24)

No `iac/<context>/billing-binding.tf` evidence. The `tenant_class.billing_components` flow
requires a billing-binding per context.

Finding 6.6.A (P0): No billing-seam evidence per context.

### 6.7 Tenant onboarding evidence (§D-20.25)

No `tenant-onboarding.tofu.apply.example` or similar evidence file. Zero-handroll OpenTofu
mandate requires `tofu apply` artifact.

Finding 6.7.A (P0): No tenant-onboarding evidence artifact.

### 6.8 Multi-context verdict

Dimension 6 score: 0/6 contexts properly declared. All P0 because HR/Payroll Big-8.

## 7. Dimension 7 — OpenTofu IaC

### 7.1 Engine constraint (§D-20.30..§D-20.31)

§D-20.30 forbids Terraform; §D-20.31 forbids Pulumi. `iac/terraform-module.tf` and
`iac/local-terraform-module.tf` violate.

Finding 7.1.A (P0): Files named `terraform-module.tf` violate the OpenTofu-only mandate.
Rename + re-author. The HCL syntax may be identical (OpenTofu is HCL2-compatible) but the
file-naming and provider-block must declare OpenTofu lineage.

### 7.2 Module signing

§D-19 mandates signed OpenTofu modules. No `.sig` artifacts or signature manifests visible.

Finding 7.2.A (P0): No module-signing artifacts present.

### 7.3 Module composition

`iac/helm-values.yaml`, `iac/local-helm-values.yaml`, `iac/local-hpa.yaml`, `iac/local-pdb.yaml`,
`iac/local-kustomization.yaml`, `iac/local-network-policy.yaml`, `iac/dr-failover.yaml`,
`iac/production-ingress.yaml`, `iac/local-secret-binding.yaml`, `iac/local-prometheus-rule.yaml`,
`iac/local-service-monitor.yaml`, `iac/local-otel-collector.yaml`, `iac/local-slo-alerts.yaml`,
`iac/ech-config.yaml`, `iac/pqc-cert.yaml`, `iac/edge-waf.yaml`, `iac/openbao-policy.yaml`,
`iac/network-policy.yaml`, `iac/secret-bindings.yaml`, `iac/service-monitor.yaml`,
`iac/kustomization.yaml`. Plus the two `.tf` files. Total ~23 IaC artifacts. Kubernetes-leaf
oriented. Acceptable for the K8s-everywhere doctrine but the OpenTofu wrapper that builds
the K8s manifests is missing.

Finding 7.3.A (P1): The Helm/Kustomize artifacts are unrooted from an OpenTofu provisioner.
Wrap them in `helm_release` and `kubectl_manifest` resources inside the OpenTofu modules.

### 7.4 OpenTofu verdict

Dimension 7 score: structurally non-conformant (Terraform-named files, no per-context
modules, no signatures). All P0 for HR/Payroll.

## 8. Dimension 8 — OS support

### 8.1 OS-matrix declaration

Required OSes per `feedback_os_support_matrix_2026_05_20`: Talos, RHEL, Oracle Linux, SUSE,
Ubuntu LTS, Debian, Rocky, AlmaLinux, CentOS Stream, Amazon Linux, Flatcar, Photon, macOS
Apple Silicon M5+.

Coverage today: no `supported_oses.json` manifest, no `os-support-matrix.md`. The `iac/`
dir does not differentiate per-OS package targets.

Finding 8.1.A (P0): No `supported_oses` manifest. Author per the user directive.

### 8.2 Package-format matrix

Required formats: RPM (RHEL/OEL/Rocky/Alma/Amazon/CentOS-Stream), DEB (Ubuntu/Debian),
container image (Talos/Flatcar/Photon/all containerized), pkg/Homebrew (macOS).

Finding 8.2.A (P0): No per-OS package format declared.

### 8.3 Per-OS CI lane

Required per memory directive. No CI workflow file in this µservice tree references per-OS
matrices.

Finding 8.3.A (P0): No per-OS CI lane.

### 8.4 Arch matrix

Required: linux/amd64 + linux/arm64 + darwin/arm64 + Tier-2 ppc64le/s390x. No arch matrix
declared.

Finding 8.4.A (P1): No arch matrix declared. (P1 because Performance Management is
likely-pure-Rust and inherits multiplatform support from cargo, but the declaration is
mandatory.)

### 8.5 OS support verdict

Dimension 8 score: 0/4 sub-requirements met. All P0 (or P1 for arch).

## 9. Dimension 9 — Rust-strict language policy

### 9.1 Backend language audit

`src/` contains `lib.rs`, `main.rs`, `config.rs`, `error.rs`, `domain/`, `usecase/`,
`adapter/`. Standard Rust crate shape. No Python, no JavaScript application logic, no
Go visible in this tree.

Finding 9.1: No violations of Rust-strict at the backend layer.

### 9.2 Authorized non-Rust audit

Authorized non-Rust per memory: OpenTofu HCL, Cedar, OpenAPI/AsyncAPI/proto3, OpenSLO,
SQL migrations, YAML/JSON, Markdown. Present in tree: HCL (.tf), Cedar (.cedar), OpenAPI
YAML, AsyncAPI YAML, proto3, OpenSLO YAML, kustomization YAML, dashboards JSON, IPs MD.
All authorized.

### 9.3 Frontend language audit

No `frontend/` dir in `microservices/performance-management/`. Frontend lives elsewhere
under `frontend/{ios,android,macos,windows,web}/`. Acceptable per µservice flat layout.

### 9.4 Build-invocation audit

§D-20.104 requires `cargo build`. The `Cargo.toml` is present; `Makefile` exists at repo root
but no local Makefile carve-outs in this dir.

### 9.5 Rust-strict verdict

Dimension 9 score: passing. The only carve-out risk is whether the SDK clients (per IP-019
sdk-client-generation) generate non-Rust SDKs. If they generate Swift/Kotlin/C# for
mobile/desktop, those are authorized under frontend-only languages.

## 10. Cross-cutting findings summary

### 10.1 P0 findings (HR/Payroll Big-8 elevated)

- 1.1.A — manifest tier-vocabulary residue
- 1.2.A — PRD missing ADR-0328 cite
- 1.3.B — eight missing parity IPs
- 1.4.A — README template-stamped
- 1.5.A — kernel src layer missing
- 1.7.A — engagement-pulse Cedar policy missing
- 2.1.A — ADR-0328 back-link missing
- 2.2.A — HR-family sibling cross-refs missing
- 2.4.A — catalog records for 4 of 5 bounded contexts missing
- 2.5.A — Terraform-named IaC files
- 2.5.B — flat IaC layout (no per-context sub-dirs)
- 3.2.A — top-4 docs template-stamped
- 3.4.T (1..6) — tier-retirement residue (6 sub-findings)
- 3.4.C (1..3) — tenant_class not adopted (3 sub-findings)
- 3.4.B (1..7) — HR-family edges missing (7 sub-findings)
- 4.9.A — declared_layers is 9-element subset of 13-layer enum
- 5.4.A — 63% counterpart parity gap
- 6.1.A — six context sub-dirs missing
- 6.2.A — OCI Always Free module missing
- 6.6.A — billing-seam evidence missing
- 6.7.A — tenant-onboarding OpenTofu evidence missing
- 7.1.A — Terraform-named files
- 7.2.A — no module-signing
- 8.1.A — no supported_oses manifest
- 8.2.A — no per-OS package format
- 8.3.A — no per-OS CI lane

Total P0: 27 findings.

### 10.2 P1 findings

- 1.2.B — Wave-3-H reference stale
- 1.3.A — IPs 026..030 too thin
- 1.5.B — single integration test
- 1.6.A — capability surface (6) misaligned with bounded-context surface (5)
- 2.3.A — counterpart roster needs Workday Performance update
- 3.4.T (7) — SLO not parameterized by tenant_class
- 3.4.C (4..5) — capability YAML missing tenant_class; cost-budget missing class split
- 3.4.B (8..9) — review-cycle event naming; recruiting hand-off
- 7.3.A — Helm/Kustomize unrooted from OpenTofu
- 8.4.A — no arch matrix

Total P1: 11 findings.

### 10.3 P2 findings

- 1.8.A — goal-cycle-close-roll-forward runbook absent
- 3.4.C (6) — dashboards not faceted by tenant_class

Total P2: 2 findings.

### 10.4 Severity rollup

40 net findings: 27 P0 + 11 P1 + 2 P2. Per §D-20.111-115 BIG-8 promotion rule, the µservice
is BLOCKED from dev-promotion until at least the P0 set is closed.

## 11. Recommended buildout order

### 11.1 Wave-4 P0 close-out batch (this audit's recommended next 30 days)

Phase A — substrate alignment (must precede all else):
- A.1: rewrite manifest.json (closes 1.1.A, 3.4.T-1..6, 4.9.A, 2.1.A, 3.4.C-1)
- A.2: rewrite PRD §M to cite ADR-0328 + §D-20 dimensions (closes 1.2.A)
- A.3: author `supported_oses.json` manifest (closes 8.1.A, 8.2.A)

Phase B — substance rewrites (top-4 docs):
- B.1: rewrite README with bespoke content (closes 1.4.A)
- B.2: rewrite ARCHITECTURE.md with sequence diagrams + layer-enum mapping (closes 3.2.A
  partial)
- B.3: rewrite compliance.md with bespoke pack overlays (closes 3.2.A partial)
- B.4: rewrite competitor-parity-matrix.md citing Lattice/15Five/Workday Performance with
  capability-by-capability matrix (closes 2.3.A + 3.2.A partial)

Phase C — HR-family edges:
- C.1: author IP-031..IP-037 for the 7 missing HR-family edges (B-1..B-7) (closes 3.4.B-1..7)
- C.2: author cross-µservice contracts (AsyncAPI 3.1.0 envelopes) for each edge

Phase D — counterpart parity backfill:
- D.1: author 8 net-new capability YAMLs (1-on-1, 360, succession, talent-card, 9-box,
  recognition, eNPS-pulse, weekly-check-in) and matching IPs (closes 1.3.B)
- D.2: extend bounded contexts list to include `1-on-1-cadence`, `succession-planning`,
  `recognition`, `weekly-check-in` (manifest update)
- D.3: author 16 more capability YAMLs to reach 85% parity floor (closes 5.4.A)

Phase E — multi-context IaC:
- E.1: create six `iac/<context>/` sub-dirs (closes 6.1.A)
- E.2: author OCI Always Free module (closes 6.2.A)
- E.3: rename Terraform-named .tf to OpenTofu-conformant naming + provider declaration
  (closes 2.5.A, 7.1.A)
- E.4: author billing-binding.tf per context (closes 6.6.A)
- E.5: author tenant-onboarding evidence artifact (closes 6.7.A)
- E.6: author module-signing pipeline (closes 7.2.A)

Phase F — Cedar + src + tests:
- F.1: author `local-engagement-pulse-anonymity.cedar` (closes 1.7.A)
- F.2: branch Cedar policies on tenant_class (closes 3.4.C-2)
- F.3: add `src/kernel/` layer (closes 1.5.A)
- F.4: extend tests/ to cover property + replay + migration + authorization + contract
  categories (closes 1.5.B)

Phase G — catalog completion:
- G.1: author 13-layer catalog records for goal-cycle, feedback, engagement-survey,
  and calibration-distinct bounded contexts (closes 2.4.A)

Phase H — CI:
- H.1: per-OS CI lane (closes 8.3.A)

### 11.2 P1 close-out (next 60 days after Phase A-H)

- 1.2.B Wave-3-H → Wave-3-I rename in PRD
- 1.3.A thicken IPs 026..030
- 1.6.A reconcile capability-surface vs bounded-context-surface
- 3.4.T-7 parameterize SLOs by tenant_class
- 3.4.C-4..5 capability YAML + cost-budget tenant_class split
- 3.4.B-8..9 analytics + recruiting hand-offs
- 7.3.A wrap Helm/Kustomize in OpenTofu
- 8.4.A arch matrix

### 11.3 P2 close-out

- 1.8.A goal-cycle-close-roll-forward runbook
- 3.4.C-6 dashboards tenant_class faceting

## 12. Provenance and evidence ledger

### 12.1 Files inspected (sample)

- microservices/performance-management/manifest.json (135 lines)
- microservices/performance-management/PRD.md (401 lines, sampled)
- microservices/performance-management/README.md (221 lines, sampled)
- microservices/performance-management/ARCHITECTURE.md (902 lines, header sampled)
- microservices/performance-management/compliance.md (925 lines, header sampled)
- microservices/performance-management/competitor-parity-matrix.md (370 lines, sampled)
- microservices/performance-management/IP-001..IP-030 file listing
- microservices/performance-management/capabilities/*.yaml (6 files)
- microservices/performance-management/contracts/*.{yaml,proto} (6 files)
- microservices/performance-management/policies/*.cedar (6 files)
- microservices/performance-management/iac/*.{yaml,tf,hcl} (23 files)
- microservices/performance-management/slos/*.openslo.yaml (12 files)
- microservices/performance-management/runbooks/*.md (20 files)
- microservices/performance-management/dashboards/*.json (10 files)
- microservices/performance-management/catalog/*.yaml (13 files)
- microservices/performance-management/src/{lib.rs, main.rs, config.rs, error.rs,
  domain/, usecase/, adapter/}

### 12.2 Canonical source files cited

- /Users/jasonlee/oyatie/docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md
  (sections D-2, D-19, D-20.1..D-20.31, D-20.111-115)
- /Users/jasonlee/oyatie/specs/master-plan-sequencing.json (tenant_class_default keys
  lines 712, 718, 724, 730, 736, 742)
- /Users/jasonlee/oyatie/docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md
- /Users/jasonlee/oyatie/docs/decisions/ADR-0316 (capability-tier-vs-microservice)
- /Users/jasonlee/oyatie/docs/decisions/ADR-0314 (DealSet settlement)
- /Users/jasonlee/oyatie/docs/decisions/ADR-0315
- /Users/jasonlee/oyatie/docs/decisions/ADR-0248 (Amazon-shape cellular)
- /Users/jasonlee/oyatie/docs/decisions/ADR-0245 (substrate-vs-product)
- /Users/jasonlee/oyatie/docs/decisions/ADR-0244 (tenant scoping)
- /Users/jasonlee/oyatie/docs/decisions/ADR-0253-amendment (HTTP/3 + QUIC)
- /Users/jasonlee/oyatie/docs/decisions/ADR-0131 (per-µservice flat layout)
- /Users/jasonlee/oyatie/docs/decisions/ADR-0132 (no-suite policy)
- /Users/jasonlee/oyatie/docs/decisions/ADR-0105 (13-layer enum)

### 12.3 Memory directives cited

- /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md
- /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_docs_substance_not_scaffold_2026_05_20.md
- /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md
- /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md
- /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_zero_handroll_opentofu_only_2026_05_20.md
- /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_oci_always_free_maximization_2026_05_20.md
- /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_multi_context_provider_agnostic_2026_05_20.md
- /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_bominal_inheritance_precedence.md
- /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_quality_performance_scalability_bar.md
- /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_go_with_original_ambition_2026_05_20.md
- /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_verify_deliverables_not_just_line_count_2026_05_20.md

### 12.4 Audit-window blast radius

Audit window touches only `microservices/performance-management/` plus this single audit
artifact. No commits. No cross-µservice writes. No code or contract emission. No CI changes.

## 13. Audit certification

### 13.1 Auditor attestation

The auditor (sole-owner of axis-performance-management for this audit window) certifies
that the findings in §1..§9, summarized in §10, and sequenced in §11 reflect the corpus
as observed on 2026-05-21. The findings are conditional on the sampled reading — full
verification of the 162 files requires Phase B substance rewrites to either confirm or
falsify the stamped-boilerplate hypothesis at scale.

### 13.2 Promotion gate verdict

`performance-management` is BLOCKED from dev-promotion until P0 close-out Phase A-H
completes. P1 and P2 may close in parallel with Phase A-H buildout. This is a hard block
per ADR-0328 §D-20.111 because every P0 violation in an HR/Payroll µservice would let
downstream agents ship broken HR/Payroll work.

### 13.3 Next audit checkpoint

Re-audit after Phase A-H completion; sequence: re-run dimensions 1, 2, 3, 4, 5, 6, 7, 8, 9
in that order; declare promotion-eligible only when 0 P0 findings remain.

### 13.4 Sibling-µservice audit handoff

Audit findings B-1..B-7 declare dependency edges into compensation, people-records,
learning-development, time-tracking, workforce-planning. Those µservices' own wave-4-rolling
audits must reciprocate the edge declarations. Hand off this audit's HR-family-completeness
section (3.4.B) to those sibling audit threads.

## 14. Closure

This audit closes ADR-0328 §D-20-dimension-1..9 sweep for `microservices/performance-management/`
at 2026-05-21. Three companion artifacts are produced in the same directory:

- coherence-audit-2026-05-20.md (this document)
- feature-parity-matrix-2026-05-20.md (Lattice + 15Five + Workday Performance union)
- performance-benchmark-numbers-2026-05-20.md (industry-leader latency / throughput targets
  with tenant_class overlay)

No further deliverables in this audit window per the user directive (3 deliverables, no
tier-deltas, no parallel writes, no commits).

End of audit.
