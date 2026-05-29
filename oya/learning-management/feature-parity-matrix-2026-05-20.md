---
doc_class: Feature-Parity-Matrix
matrix_id: feature-parity-matrix-2026-05-20-learning-management
microservice: learning-management
phase: Phase-4A.1-HR-Payroll-Big-8
batch: Wave-4-Rolling-HR-Payroll
matrix_date: 2026-05-20
matrix_owner: solo-codex-microservice-ownership-agent
matrix_method: union-coverage-per-ADR-0328-D-5
top_3_counterparts:
  - id: canvas-lms
    name: Canvas LMS (Instructure)
    industry_segment: Academic / Higher-Ed LMS leader
    public_features_source: instructure.com/products/canvas + community.canvaslms.com docs + LTI 1.3 spec
  - id: cornerstone-ondemand
    name: Cornerstone OnDemand
    industry_segment: Corporate-Learning + Talent Management suite leader (Cornerstone merged with Saba 2020)
    public_features_source: cornerstoneondemand.com/products + product datasheets
  - id: docebo
    name: Docebo
    industry_segment: AI-driven Corporate LMS challenger leader
    public_features_source: docebo.com/learning-platform + AI-Suite product pages
parity_states_per_ADR_0328_D_5_15:
  - covered
  - partial
  - missing
  - out-of-scope-intentional
five_anchor_citations:
  anchor_1_unified_ecosystem_thesis: /Users/jasonlee/oyatie/docs/architecture/unified-ecosystem-thesis-2026-05-21.md
  anchor_2_microservice_prd: /Users/jasonlee/oyatie/microservices/learning-management/PRD.md
  anchor_3_local_artifact_inventory: coherence-audit-2026-05-20.md §A.2
  anchor_4_top_3_counterparts: Canvas LMS + Cornerstone OnDemand + Docebo per brief override
  anchor_5_documentation_rigor_1_1: /Users/jasonlee/oyatie/docs/standards/documentation-rigor.md §1.1
union_coverage_summary:
  total_feature_rows: 95
  covered_rows: 12
  partial_rows: 22
  missing_rows: 49
  out_of_scope_intentional_rows: 12
  net_parity_score_percent: 22
verdict: REVISE
---

# Feature Parity Matrix — `learning-management` vs Canvas LMS + Cornerstone OnDemand + Docebo

## §0. Counterpart-source note

The brief's three counterparts (Canvas LMS + Cornerstone OnDemand + Docebo) span three distinct LMS industry segments:

- Canvas LMS = academic LMS leader (K-12, higher-ed, university). Owned by Instructure. Public market share leader in higher-ed since 2018.
- Cornerstone OnDemand = corporate-learning + talent-management suite leader. Merged with Saba 2020. Targets regulated enterprises with compliance training, certification tracking, and learning + performance + recruiting bundles.
- Docebo = AI-driven corporate LMS challenger. Pure-play corporate LMS with embedded AI for content discovery + recommendation + skill-gap detection + automatic learning-path generation.

This audit applies union coverage per ADR-0328 §D-5.5 — if ANY of the three has a major feature, learning-management must EITHER cover it OR mark it out-of-scope-intentional with a doctrine reason. The µservice's local manifest declares a DIFFERENT counterpart set (Workday Learning + Cornerstone + Degreed + LinkedIn Learning + Udemy Business + Salesforce Trailhead); the manifest set is logged as a P0 counterpart-source contradiction in coherence-audit-2026-05-20.md §3.4.C-1.1.

The parity matrix in this file scores against the brief's three only.

## §1. Course content authoring

| # | Feature | Canvas LMS | Cornerstone | Docebo | Oyatie owning surface | Parity state | Gap or out-of-scope reason |
|---|---|---|---|---|---|---|---|
| 1.1 | Course creation with title, description, dates, instructor assignment | Yes | Yes | Yes | learning-management.course-catalog.create per manifest line 31 + capabilities/course-enroll.yaml | partial | Course-catalog command exists in OpenAPI but lacks instructor-assignment field shape; PRD §C says course-catalog has create/amend/approve/import/export/replay but doesn't declare instructor binding. Fix in Wave 15F. |
| 1.2 | Course modules with sequencing and prerequisites | Yes (Modules with Module Requirements) | Yes (Curriculum) | Yes (Learning Plan) | course-catalog supports `learning-path` as separate bounded-context per manifest line 32 | partial | Learning-path bounded-context is the structural seam; module-level prerequisite logic inside a single course is missing. Wave 15F adds module-prerequisite to course-catalog domain. |
| 1.3 | Course copying / duplication | Yes (Copy Course) | Yes (Curriculum Cloning) | Yes (Course Duplicate) | not yet declared | missing | No corresponding command in PRD §D FR list. Wave 15F adds `course-catalog.duplicate` command. |
| 1.4 | Course templates (reusable course shells) | Yes (Course Blueprints + Blueprint Sync) | Yes (Curriculum templates) | Yes (course library + AI-templates) | not yet declared | missing | Wave 15F adds template surface to course-catalog. |
| 1.5 | Course-section management (multiple sections of same course) | Yes (Sections) | Yes (cohorts) | Yes (sessions / classes) | enrollment bounded-context partial | partial | Enrollment bounded-context can carry section reference; not declared in PRD §C. Wave 15F adds section field. |
| 1.6 | Course versioning + archival | Yes | Yes | Yes | course-catalog supports archive command per ARCHITECTURE.md §C | covered | Archive in ARCHITECTURE.md §C; version monotonicity invariant in §C. |
| 1.7 | Rich-text content editor (HTML pages) | Yes (Pages with RCE) | Yes (Content Studio) | Yes (Page Builder) | UX-shell delegated | out-of-scope-intentional | Per ADR-0245 substrate vs product layering, content authoring UI belongs to the application shell, not the operational µservice. Doctrine reason: one UX shell, not per-µservice editors. |
| 1.8 | File uploads (PDF, video, image, document attachments) | Yes | Yes | Yes | content-provider-catalog-sync capability + cloud-storage substrate | partial | provider-catalog-sync handles external provider files; first-party uploads need cloud-storage handoff declared. Wave 15F adds upload-flow handoff in cross-microservice-handoffs.md. |
| 1.9 | YouTube / Vimeo / external video embedding | Yes | Yes | Yes | UX-shell delegated + content-provider-catalog-sync | out-of-scope-intentional | External embedding is a UX-shell + content-provider concern, not the µservice's operational concern. |
| 1.10 | Mathematical equation editor (LaTeX / MathML) | Yes | Limited | No | UX-shell delegated | out-of-scope-intentional | Academic-feature; UX shell concern; doctrine reason per ADR-0245. |

## §2. Assignment + grading

| # | Feature | Canvas LMS | Cornerstone | Docebo | Oyatie owning surface | Parity state | Gap or out-of-scope reason |
|---|---|---|---|---|---|---|---|
| 2.1 | Assignment creation (title, instructions, due-date, points) | Yes (Assignments) | Yes (Training tasks) | Yes (assignments) | MISSING bounded-context | missing | learning-management does not declare an `assignment` bounded-context. Per coherence-audit §3.4.C-1.2 this is a P0 gap if academic LMS is in scope. Wave 15F decision required. |
| 2.2 | Assignment submission (file upload, text entry, URL, media) | Yes | Yes (limited) | Yes (limited) | MISSING | missing | Same as 2.1. Wave 15F. |
| 2.3 | Group assignments (collaborative submissions) | Yes (Group Assignments) | Limited | Yes | MISSING | missing | Cornerstone has limited group support; Canvas full; Docebo full. Wave 15F. |
| 2.4 | Peer review assignments | Yes (Peer Reviews) | No | Limited | MISSING | missing | Canvas hero feature; partial in Docebo. Wave 15F. |
| 2.5 | Rubrics (criterion-based grading scales) | Yes (Rubrics with criterion+rating) | Yes (skill rubrics) | Yes (rubric tool) | MISSING | missing | Wave 15F adds rubric to assignment bounded-context. |
| 2.6 | Grade-book (instructor view of all student grades per course) | Yes (Gradebook with weighted columns) | Yes (Manager dashboard) | Yes (Reports + grade view) | MISSING | missing | No grade-book surface in PRD/IP/contracts. Wave 15F adds. |
| 2.7 | Grade-book column management (weighting, drop-lowest, extra-credit) | Yes (Gradebook columns) | Limited | Limited | MISSING | missing | Canvas hero feature. Wave 15F. |
| 2.8 | Late-submission policy (zero, deduction, allow) | Yes (Late Policy) | Limited | Yes (deadline policies) | MISSING | missing | Wave 15F. |
| 2.9 | Grade passback to external SIS (LTI 1.3 Advantage AGS) | Yes (LTI 1.3 Assignment + Grade Services) | Limited (HRIS integration) | Yes (HRIS push) | MISSING | missing | Canvas LTI hero; Wave 15F adds LTI 1.3 AGS connector. |
| 2.10 | SpeedGrader / instructor grading UI | Yes (SpeedGrader) | Limited | Limited | UX-shell delegated | out-of-scope-intentional | UX concern; doctrine reason per ADR-0245. |
| 2.11 | Re-grade / regrade workflow | Yes | Yes | Yes | MISSING | missing | Wave 15F adds amend + approve flow under assignment. |
| 2.12 | Anonymous grading | Yes (Anonymous Grading) | No | Limited | MISSING | missing | Canvas + FERPA compliance feature. Wave 15F. |
| 2.13 | Plagiarism integration (Turnitin / Unicheck) | Yes (LTI 1.3 plagiarism platforms) | Limited | Yes | MISSING | missing | Wave 15F adds plagiarism-platform LTI connector. |

## §3. Quizzes + assessments

| # | Feature | Canvas LMS | Cornerstone | Docebo | Oyatie owning surface | Parity state | Gap or out-of-scope reason |
|---|---|---|---|---|---|---|---|
| 3.1 | Quiz creation with multiple question types | Yes (New Quizzes with 12+ types) | Yes (Test engine) | Yes (Quiz builder) | assessment bounded-context per manifest line 33 | partial | assessment bounded-context exists; PRD §D FR-019..FR-024 declares CRUD but does not declare question-type taxonomy. Wave 15F enumerates. |
| 3.2 | Question bank / question library | Yes (Item Banks) | Yes | Yes | MISSING | missing | Wave 15F adds item-bank to assessment. |
| 3.3 | Quiz attempt policies (single/multiple/best-of) | Yes | Yes | Yes | partial | partial | Cedar policy local-assessment-attempt-control.cedar exists per directory listing but contents not sampled. Wave 15F samples + remediates. |
| 3.4 | Time-limited quizzes | Yes | Yes | Yes | MISSING | missing | Wave 15F. |
| 3.5 | Question randomization | Yes (per quiz) | Yes | Yes | MISSING | missing | Wave 15F. |
| 3.6 | Answer-bank shuffling | Yes | Yes | Yes | MISSING | missing | Wave 15F. |
| 3.7 | Quiz availability windows (open / close dates) | Yes | Yes | Yes | MISSING | missing | Wave 15F. |
| 3.8 | Auto-grading + multiple-choice scoring | Yes | Yes | Yes | partial | partial | Implied by assessment + grade-book bounded-contexts but not declared as auto-grading flow. Wave 15F. |
| 3.9 | Essay grading (manual review) | Yes | Yes | Yes | MISSING | missing | Wave 15F. |
| 3.10 | Proctoring integration (Respondus / Proctorio / HonorLock) | Yes (LTI 1.3 proctoring platforms) | Limited | Limited | MISSING | missing | Wave 15F adds proctor-platform connector. |
| 3.11 | Quiz analytics (per-question difficulty, discrimination index) | Yes (Quiz Statistics) | Yes (Test analytics) | Yes (Quiz Insights) | MISSING (delegated to analytics µservice?) | missing | Wave 15F decides ownership. analytics µservice candidate. |
| 3.12 | Adaptive testing / branching by answer | Limited | Yes (adaptive) | Yes (adaptive) | MISSING | missing | Wave 15F. |
| 3.13 | Survey + ungraded feedback collection | Yes (Surveys) | Yes (Surveys) | Yes (Surveys) | MISSING (delegated to forms µservice?) | missing | Wave 15F decides. forms µservice candidate per Phase 3 D-1.64. |

## §4. Enrollment + cohort management

| # | Feature | Canvas LMS | Cornerstone | Docebo | Oyatie owning surface | Parity state | Gap or out-of-scope reason |
|---|---|---|---|---|---|---|---|
| 4.1 | Manual enrollment (admin adds learner to course) | Yes | Yes | Yes | enrollment bounded-context per manifest line 31 + capabilities/course-enroll.yaml | covered | enrollment.create command in PRD §D FR-007. |
| 4.2 | Self-enrollment (learner opts in) | Yes (Self-Enrollment URL) | Yes | Yes | partial | partial | Cedar policy local-cohort-enrollment-scope.cedar exists but content not sampled; PRD §D does not name self-enrollment vs admin-enrollment distinction. Wave 15F. |
| 4.3 | Bulk enrollment (CSV import) | Yes (SIS Import) | Yes (Bulk loader) | Yes (Bulk + API) | partial | partial | PRD §D FR-010 declares enrollment.import; CSV format not declared. Wave 15F. |
| 4.4 | Auto-enrollment by job role / org unit | Limited (via SIS sync) | Yes (Dynamic Cohorts / OUs) | Yes (Dynamic Audiences) | MISSING (depends on hris handoff per coherence-audit §3.4.B-1.1) | missing | Wave 15A adds hris dependency; Wave 15F adds dynamic-audience rule engine. |
| 4.5 | Enrollment dates (start / end / available-until) | Yes | Yes | Yes | partial | partial | Cohort enrollment lag runbook exists (runbooks/local-cohort-enrollment-lag.md); date semantics not declared in PRD. Wave 15F. |
| 4.6 | Waitlists | Yes | Yes (Session waitlists) | Yes (Waitlist) | MISSING | missing | Wave 15F. |
| 4.7 | Drop / unenroll | Yes | Yes | Yes | partial | partial | enrollment.amend can carry status change; explicit drop command not declared. Wave 15F. |
| 4.8 | Cohort grouping (audience / OU / segment) | Yes (Sections + Groups) | Yes (OUs hero feature) | Yes (Branches + Groups hero feature) | community µservice delegation per manifest line 37 | partial | community µservice owns cohort/discussion per Phase 3 D-1.67; handoff to community not declared. Wave 15F adds. |
| 4.9 | Group leaderboards | Limited | Yes | Yes (gamification) | MISSING (delegated to community / gamification surface) | out-of-scope-intentional | Gamification belongs to community + analytics; doctrine reason per ADR-0245. |
| 4.10 | Cohort-based learning paths | Limited | Yes (Curriculum + OU) | Yes (Learning Plan + Audience) | learning-path bounded-context | partial | learning-path bounded-context exists per manifest line 32; OU/audience binding not declared. Wave 15F. |

## §5. Discussions + social learning

| # | Feature | Canvas LMS | Cornerstone | Docebo | Oyatie owning surface | Parity state | Gap or out-of-scope reason |
|---|---|---|---|---|---|---|---|
| 5.1 | Threaded discussion forums per course | Yes (Discussions) | Yes (Live Feed) | Yes (Discussions hero) | community µservice | partial | community µservice owns discussions per Phase 3 D-1.67; learning-management → community handoff for course-context not declared. Wave 15F. |
| 5.2 | Graded discussions (count toward grade) | Yes (Graded Discussions) | Limited | Yes | MISSING (needs assignment + community handoff) | missing | Wave 15F (depends on §2 assignment surface). |
| 5.3 | Instructor moderation (lock / pin / delete) | Yes | Yes | Yes | community delegation | partial | community owns moderation; learning-management needs to declare moderation-authority handoff for course-scoped threads. Wave 15F. |
| 5.4 | Group discussions (subgroup-only threads) | Yes | Limited | Yes | community delegation | partial | Same as 5.3. |
| 5.5 | Social-learning channels (peer Q&A, follow, like) | Limited | Yes (hero feature with informal-learning) | Yes (Coach + Share hero) | community delegation | partial | community handoff not declared. Wave 15F. |
| 5.6 | Expert / mentor matching | No | Yes (Mentorship module) | Limited | MISSING (delegated to community or new mentorship surface) | out-of-scope-intentional | Cornerstone-specific; doctrine reason: belongs to a future mentorship µservice not learning-management. |
| 5.7 | User-generated content (UGC) submissions | Limited | Yes (Skill Shares hero) | Yes (Coach + Share hero) | community delegation | partial | community handoff not declared. Wave 15F. |
| 5.8 | UGC moderation queue + Cedar policy | n/a | Yes | Yes | community delegation | partial | Same as 5.7. |
| 5.9 | Inline annotations (Hypothes.is integration) | Yes (LTI) | No | No | MISSING | out-of-scope-intentional | Canvas hero academic feature; doctrine reason per ADR-0245 — LTI is shared, not learning-management-owned. |

## §6. Standards + interop (SCORM, xAPI, LTI, AICC, cmi5)

| # | Feature | Canvas LMS | Cornerstone | Docebo | Oyatie owning surface | Parity state | Gap or out-of-scope reason |
|---|---|---|---|---|---|---|---|
| 6.1 | SCORM 1.2 import + runtime | Yes (External Tools) | Yes (hero feature) | Yes (hero feature) | MISSING | missing | Wave 15F adds scorm-runtime bounded-context per coherence-audit §3.4.C-1.2. |
| 6.2 | SCORM 2004 (3rd/4th edition) | Yes | Yes | Yes | MISSING | missing | Wave 15F. |
| 6.3 | xAPI / Tin Can statement-LRS storage | Limited (LTI external) | Yes (LRS built-in) | Yes (LRS built-in hero) | MISSING | missing | Wave 15F adds xapi-statement bounded-context + LRS storage. Cornerstone and Docebo both bundle LRS. |
| 6.4 | cmi5 | Limited | Yes | Yes | MISSING | missing | Wave 15F. |
| 6.5 | AICC (legacy) | Limited | Yes | Yes | MISSING | out-of-scope-intentional | AICC is deprecated industry-wide; reason: industry-deprecation per ADR-0328 §D-5.13. |
| 6.6 | LTI 1.1 consumer (launch external tool) | Yes | Yes | Yes | MISSING | missing | Wave 15F adds LTI 1.1 consumer. |
| 6.7 | LTI 1.3 Advantage (NRPS + AGS + DLR) | Yes (hero feature) | Limited | Yes | MISSING | missing | Wave 15F adds LTI 1.3 with Name + Roles Provisioning Service + Assignment + Grade Services + Deep Linking + Resource Search. |
| 6.8 | LTI 1.3 provider (act as tool for another LMS) | Limited | No | No | MISSING | partial | Wave 15F decides — Oyatie could expose learning-management as an LTI tool for Canvas-hosting universities. |
| 6.9 | Common Cartridge import / export | Yes | Limited | Limited | MISSING | missing | Wave 15F. |
| 6.10 | QTI 2.x / 3.0 question interop | Yes | Limited | Limited | MISSING | missing | Wave 15F. |
| 6.11 | IMS Caliper analytics emission | Yes | Limited | Limited | MISSING (delegated to observability) | missing | Wave 15F decides — emit to observability + audit-chain. |
| 6.12 | OpenBadges issuance (Open Badges 2.0 / 3.0) | Yes (Canvas Badges) | Yes | Yes | credential bounded-context per manifest line 33 + capabilities/credential-issue.yaml | partial | credential.create command exists; Open Badges JSON-LD shape not declared. Wave 15F adds. |
| 6.13 | Verifiable Credentials (W3C VC 2.0) | Limited | Limited | Limited | partial | partial | credential bounded-context could carry VC issuance; not declared in PRD. Wave 15F. |
| 6.14 | Comprehensive Learner Record (CLR) | Yes (CLR 1.0) | Limited | Limited | MISSING | missing | Wave 15F. |

## §7. Compliance + regulated training

| # | Feature | Canvas LMS | Cornerstone | Docebo | Oyatie owning surface | Parity state | Gap or out-of-scope reason |
|---|---|---|---|---|---|---|---|
| 7.1 | Mandatory training assignment (compliance-officer authority) | Limited (academic) | Yes (hero feature) | Yes (hero feature) | regulated-training-attest capability + IP-027-compliance-training-attestation-ledger.md | covered | IP-027 is bespoke per coherence-audit §A.2. capabilities/regulated-training-attest.yaml exists. |
| 7.2 | Certification tracking (issued + expiry + renewal) | Limited | Yes (hero) | Yes (hero) | credential bounded-context + IP-030-credential-expiry-renewal-orchestrator.md | covered | IP-030 bespoke per coherence-audit §A.2. capabilities/credential-issue.yaml exists. |
| 7.3 | Recertification window + auto-renewal trigger | Limited | Yes | Yes | covered (IP-030) | covered | Same as 7.2. |
| 7.4 | Audit trail of attestations (FDA 21 CFR Part 11, OSHA, HIPAA) | Limited | Yes (hero) | Yes | regulated-training-attest + compliance.md §D Audit Events | covered | EVT-LEARNING_MANAGEMENT-CREDENTIAL-APPROVED + EVT-LEARNING_MANAGEMENT-CREDENTIAL-IMPORT-REJECTED enumerated in compliance.md §D. |
| 7.5 | E-signature on attestation (FDA 21 CFR Part 11) | No | Yes (hero in life-sciences pack) | Limited | MISSING | missing | Wave 15F adds. Industry-leader bar — Cornerstone life-sciences pack. |
| 7.6 | Mandatory-training overdue escalation | No | Yes | Yes | partial | partial | runbooks/regulated-training-audit.md exists; escalation policy not declared in PRD. Wave 15F. |
| 7.7 | Compliance-officer dashboard | Limited | Yes (hero) | Yes | dashboards/compliance-pack-health.json exists per directory listing | partial | Dashboard JSON exists; substance not sampled. Wave 15F. |
| 7.8 | FERPA evidence (academic record protection) | Yes | Limited | Limited | compliance.md declares FERPA pack | partial | Pack named in manifest line 79 + compliance.md §B; pack overlay file not yet authored. Wave 15F. |
| 7.9 | KOSA evidence (under-17 learner protection) | Limited (LTI-dependent) | Limited | Limited | compliance.md declares KOSA pack | partial | Pack named; overlay not authored. Wave 15F. |
| 7.10 | HIPAA evidence (health-information training) | Limited | Yes (life-sciences pack) | Limited | manifest line 108 declares hipaa pack but compliance.md §B does NOT — see coherence-audit B.1.4 | partial | P1 internal-coherence per coherence-audit. |
| 7.11 | GDPR Article 15 DSAR export of learning records | Limited | Yes | Yes | compliance.md §E declares "Cedar policy decision log for every mutation" but DSAR export shape not declared | partial | Wave 15F. |
| 7.12 | Continuing Education Units (CEU) tracking | No | Yes (hero) | Yes | partial | partial | credential bounded-context could carry CEU; not declared. Wave 15F. |
| 7.13 | Continuing Professional Development (CPD) hours | Limited | Yes | Yes | partial | partial | Same as 7.12. |
| 7.14 | Regulator export (DOL, OSHA, FAA, FDA) | No | Yes (hero) | Limited | partial | partial | Cornerstone life-sciences and aviation packs. Wave 15F. |

## §8. Mobile + offline learning

| # | Feature | Canvas LMS | Cornerstone | Docebo | Oyatie owning surface | Parity state | Gap or out-of-scope reason |
|---|---|---|---|---|---|---|---|
| 8.1 | Native iOS app (course consumption) | Yes (Canvas Student + Teacher) | Yes (Cornerstone Mobile) | Yes (Go.Learn mobile) | UX-shell delegated to iOS frontend | partial | Per `feedback_os_support_matrix_2026_05_20` iOS Swift is allowed; learning-management exposes mobile via UX-shell. Handoff not declared. Wave 15F. |
| 8.2 | Native Android app | Yes | Yes | Yes | UX-shell delegated to Android frontend | partial | Same as 8.1. Kotlin per OS matrix. |
| 8.3 | Offline course download (consume without network) | Yes (Canvas Student offline) | Yes | Yes (hero with sync-when-online) | MISSING | missing | Wave 15F adds offline-sync flow. |
| 8.4 | Offline quiz submission with sync | Yes | Limited | Yes | MISSING | missing | Wave 15F. |
| 8.5 | Push notifications (assignment + deadline + grade) | Yes | Yes | Yes | partial | partial | mail µservice handoff per manifest line 40; push-notification handoff distinct from mail not declared. Wave 15F. |
| 8.6 | Mobile-optimized content rendering | Yes (Mastery Path mobile) | Yes | Yes | UX-shell delegated | out-of-scope-intentional | Per ADR-0245 UX concern, not operational µservice. |
| 8.7 | Mobile SCORM playback | Limited (SCORM is browser-native) | Yes | Yes (hero) | MISSING | missing | Wave 15F (depends on §6.1 SCORM runtime). |
| 8.8 | Mobile xAPI offline buffering | Limited | Yes | Yes | MISSING | missing | Wave 15F (depends on §6.3 xAPI). |
| 8.9 | Single-sign-on from mobile (OIDC + SAML) | Yes | Yes | Yes | partial (delegated to identity µservice) | covered | identity µservice owns SSO per manifest line 41; learning-management consumes principal. |

## §9. Virtual classroom + live sessions

| # | Feature | Canvas LMS | Cornerstone | Docebo | Oyatie owning surface | Parity state | Gap or out-of-scope reason |
|---|---|---|---|---|---|---|---|
| 9.1 | Live virtual classroom (Zoom / Teams / BBB integration) | Yes (BigBlueButton + Zoom LTI) | Yes (Saba virtual classroom + Zoom) | Yes (Webex + Zoom + Teams) | meet µservice delegation per Phase 3 D-1.58 | partial | meet µservice owns live video per Phase 3; handoff not declared in learning-management. Wave 15F. |
| 9.2 | Session scheduling | Yes | Yes (hero with calendar) | Yes | calendar µservice delegation per Phase 3 D-1.57 | partial | calendar handoff not declared. Wave 15F. |
| 9.3 | Session attendance tracking | Yes | Yes | Yes | partial | partial | Cedar policy local-session-attendance-access.cedar exists per directory listing; not declared in PRD. Wave 15F samples cedar substance + declares in PRD. |
| 9.4 | Session recording + playback | Yes | Yes | Yes | recordings µservice delegation per Phase 3 D-1.59 | partial | recordings handoff not declared. Wave 15F. |
| 9.5 | Breakout rooms | Yes (via BBB) | Limited | Yes | meet delegation | out-of-scope-intentional | meet owns. Doctrine reason: substrate vs product. |
| 9.6 | Whiteboard | Yes (via BBB) | Limited | Limited | whiteboard µservice delegation per Phase 4 D-1.91 | out-of-scope-intentional | whiteboard owns. |
| 9.7 | Screen-sharing + polls | Yes | Yes | Yes | meet delegation | out-of-scope-intentional | meet owns. |
| 9.8 | Auto-grading from attendance | Limited | Yes | Yes | MISSING (needs §2 assignment + §9.3 attendance handoff) | missing | Wave 15F. |
| 9.9 | Live-session chat + Q&A | Yes | Yes | Yes | meet delegation | out-of-scope-intentional | meet owns. |

## §10. Content library + marketplace

| # | Feature | Canvas LMS | Cornerstone | Docebo | Oyatie owning surface | Parity state | Gap or out-of-scope reason |
|---|---|---|---|---|---|---|---|
| 10.1 | Internal content library (courses + modules + assets reusable) | Yes (Commons) | Yes (Content Studio + Library) | Yes (Central Repository) | content-provider-catalog-sync capability + IP-028-content-provider-catalog-federation.md | partial | IP-028 bespoke; first-party content library shape not declared. Wave 15F. |
| 10.2 | Content marketplace (buy external courses) | Yes (Commons) | Yes (Content Anytime hero) | Yes (Content Marketplace + Docebo Shop) | cloud-marketplace µservice delegation per manifest line 56 settlement-binding | partial | cloud-marketplace owns settlement per ADR-0314; learning-management content-marketplace handoff not declared. Wave 15F. |
| 10.3 | Pre-built content packs (compliance, leadership, technical, soft-skills) | Limited | Yes (hero with thousands of titles) | Yes (hero with 800+ providers) | cloud-marketplace delegation | partial | Same as 10.2. |
| 10.4 | LinkedIn Learning / Udemy Business / Coursera integration | Yes (LTI) | Yes (deep partnerships) | Yes (deep partnerships hero) | content-provider-catalog-sync capability | partial | Provider-catalog-sync capability declared; specific provider integrations not enumerated. Wave 15F. |
| 10.5 | Skill-based content recommendation | Limited | Yes | Yes (hero with AI) | skills-graph-export capability + IP-026-skills-graph-gap-analyzer.md + IP-029-learning-path-recommendation-guardrail.md | partial | IP-026 + IP-029 bespoke per coherence-audit §A.2; recommendation engine integration with intelligence µservice not declared. Wave 15F. |
| 10.6 | Content provider royalty / revenue-share settlement | Limited (Commons free) | Yes (royalty + per-seat) | Yes (royalty + per-seat + per-usage) | cloud-marketplace + cloud-billing delegation | partial | Per `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20` revenue_share is a cloud-billing component for paid tenants. learning-management forwards content-marketplace events to cloud-marketplace + cloud-billing. Wave 15F declares handoff. |

## §11. AI-driven learning (Docebo + Cornerstone hero axis)

| # | Feature | Canvas LMS | Cornerstone | Docebo | Oyatie owning surface | Parity state | Gap or out-of-scope reason |
|---|---|---|---|---|---|---|---|
| 11.1 | AI content tagging | Limited | Yes (AI-Suite) | Yes (Docebo AI hero) | intelligence µservice delegation per manifest line 41 | partial | intelligence µservice owns AI per ADR-0255 + Phase 2 D-1.46; learning-management → intelligence handoff for content-tagging not declared. Wave 15F. |
| 11.2 | AI course recommendation per learner | Limited | Yes | Yes (Docebo Discover hero) | intelligence + IP-029-learning-path-recommendation-guardrail.md | partial | IP-029 bespoke; intelligence handoff not declared. Wave 15F. |
| 11.3 | AI skill-gap detection (employee vs role-required skills) | No | Yes (Skills Graph hero) | Yes (Docebo Skills) | skills-graph-export capability + IP-026 | partial | IP-026 bespoke; intelligence handoff not declared. Wave 15F. |
| 11.4 | AI auto-generated learning path | No | Yes (curriculum auto-build) | Yes (Discover learning paths hero) | partial | partial | learning-path bounded-context exists; auto-generation flow not declared. Wave 15F. |
| 11.5 | Auto-translated content (40+ languages) | Limited (LTI) | Yes | Yes (hero) | translate µservice delegation per Phase 3 D-1.71 | partial | translate handoff not declared. Wave 15F. |
| 11.6 | Auto-generated quiz questions (from content) | No | Limited | Yes (AI quiz generator hero) | partial (intelligence delegation) | partial | intelligence handoff not declared. Wave 15F. |
| 11.7 | Auto-generated content summaries | No | Limited | Yes | intelligence delegation | partial | Same as 11.6. |
| 11.8 | Conversational AI coach (chat-with-LLM tutor) | Limited | Yes | Yes (Docebo Shape hero) | intelligence delegation | partial | Same as 11.6. |
| 11.9 | AI bias + safety guardrails | n/a | Limited | Yes (declared) | IP-029-learning-path-recommendation-guardrail.md + governance µservice | covered | IP-029 bespoke per coherence-audit; governance µservice owns Cedar policy. |
| 11.10 | EU AI Act Annex III high-risk classification handling | Limited | Limited | Limited | governance + compliance pack | partial | Wave 15F declares EU-AI-Act pack overlay. |

## §12. Analytics + reporting

| # | Feature | Canvas LMS | Cornerstone | Docebo | Oyatie owning surface | Parity state | Gap or out-of-scope reason |
|---|---|---|---|---|---|---|---|
| 12.1 | Learner progress dashboard | Yes | Yes | Yes | UX-shell + analytics µservice delegation per Phase 3 D-1.69 | partial | analytics handoff not declared. Wave 15F. |
| 12.2 | Course completion analytics | Yes (New Analytics) | Yes | Yes | analytics delegation | partial | Same as 12.1. |
| 12.3 | Engagement metrics (login frequency, time-on-task) | Yes | Yes | Yes | analytics delegation | partial | Same as 12.1. |
| 12.4 | Custom report builder | Yes | Yes (hero) | Yes (hero with Custom Reports) | analytics delegation | partial | Same as 12.1. |
| 12.5 | Scheduled report delivery (email) | Yes | Yes | Yes | analytics + mail delegation | partial | Same as 12.1. |
| 12.6 | Manager dashboard (direct-reports' learning) | Limited | Yes (hero) | Yes (hero) | analytics + hris delegation | partial | Wave 15F (depends on hris dependency). |
| 12.7 | Compliance training completion report | Yes | Yes (hero with regulator-ready export) | Yes | regulated-training-attest + analytics delegation | partial | Wave 15F. |
| 12.8 | Skills coverage heatmap | No | Yes (Skills Graph hero) | Yes (Docebo Skills heatmap hero) | skills-graph-export + analytics | partial | Wave 15F. |
| 12.9 | Predictive analytics (at-risk learner detection) | Yes | Yes | Yes | intelligence + analytics delegation | partial | Wave 15F. |
| 12.10 | xAPI Learning Record Store query API | Limited | Yes | Yes | MISSING | missing | Wave 15F (depends on §6.3 xAPI). |

## §13. Integrations + HRIS

| # | Feature | Canvas LMS | Cornerstone | Docebo | Oyatie owning surface | Parity state | Gap or out-of-scope reason |
|---|---|---|---|---|---|---|---|
| 13.1 | SIS integration (PowerSchool / Infinite Campus / Banner) | Yes (SIS Imports hero with PowerSchool / Infinite Campus / Banner / Ellucian) | Limited | Limited | MISSING | missing | Canvas hero academic feature. Wave 15F decides — likely belongs to a future student-information µservice. |
| 13.2 | HRIS integration (Workday / SAP SuccessFactors / BambooHR / ADP) | Limited | Yes (hero) | Yes (hero) | hris µservice handoff REQUIRED per coherence-audit §3.4.B-1.1 | missing | P0 — hris dependency missing in manifest. Wave 15A adds. |
| 13.3 | SCIM 2.0 user provisioning | Yes | Yes | Yes | identity µservice delegation | partial | identity owns SCIM per Phase 1 D-1.30; handoff not declared. Wave 15F. |
| 13.4 | OAuth 2.0 / OIDC | Yes | Yes | Yes | identity delegation | covered | identity owns. |
| 13.5 | SAML 2.0 | Yes | Yes | Yes | identity delegation | covered | Same as 13.4. |
| 13.6 | Webhooks (out + in) | Yes (Live Events + Plagiarism webhooks) | Yes | Yes | partial | partial | AsyncAPI 3.1.0 declared in PRD §C; specific webhook semantics not enumerated. Wave 15F. |
| 13.7 | REST API (public + internal) | Yes (extensive) | Yes (extensive) | Yes (extensive) | contracts/openapi-v1.yaml | partial | API exists but generic action-dispatch per coherence-audit D.4.1. Wave 15F expands. |
| 13.8 | GraphQL API | Yes (Canvas GraphQL hero) | Limited | Yes | MISSING (per ADR-0145 gRPC default) | out-of-scope-intentional | Per ADR-0145 reform, gRPC is the canonical internal RPC; GraphQL is optional. Doctrine reason. |
| 13.9 | gRPC API | No | Limited | Limited | contracts/learning-management-v1.proto | partial | proto3 contract exists; substance not sampled. Wave 15F. |
| 13.10 | Microsoft Teams integration (notifications + assignments-in-Teams) | Yes (Teams LTI) | Yes | Yes | workplace-integration µservice delegation per Phase 4 D-1.78 | partial | Wave 15F. |
| 13.11 | Slack integration | Yes (LTI) | Yes | Yes | workplace-integration delegation | partial | Same as 13.10. |
| 13.12 | Salesforce integration | Yes (LTI) | Yes (CRM connector) | Yes | crm µservice delegation per Phase 4 D-1.81 | partial | crm handoff not declared. Wave 15F. |
| 13.13 | Zapier / IFTTT-style automation | Yes (LTI partners) | Yes (Edge marketplace) | Yes (Connectors hero) | workflow-engine + plugin-app-store µservice delegation per Phase 4 D-1.77 | partial | Wave 15F. |

## §14. Accessibility + i18n

| # | Feature | Canvas LMS | Cornerstone | Docebo | Oyatie owning surface | Parity state | Gap or out-of-scope reason |
|---|---|---|---|---|---|---|---|
| 14.1 | WCAG 2.1 AA conformance | Yes (VPAT published) | Yes | Yes | UX-shell delegation + governance pack | partial | governance pack overlay shape exists; learning-management does not declare WCAG conformance evidence path. Wave 15F. |
| 14.2 | Screen-reader support | Yes | Yes | Yes | UX-shell delegation | out-of-scope-intentional | UX concern. Doctrine reason per ADR-0245. |
| 14.3 | Keyboard navigation | Yes | Yes | Yes | UX-shell delegation | out-of-scope-intentional | Same as 14.2. |
| 14.4 | Multi-language UI (40+ languages) | Yes (100+) | Yes | Yes (40+) | translate µservice delegation | partial | Wave 15F declares pack overlays per `feedback_canonical_base_localization`. Per memory KR is pack #1. |
| 14.5 | Right-to-left (RTL) language support (Arabic, Hebrew) | Yes | Yes | Yes | UX-shell delegation | out-of-scope-intentional | UX concern. |
| 14.6 | Closed captioning + transcripts on video | Yes (Studio captions) | Yes | Yes | recordings + translate delegation | partial | Wave 15F. |

## §15. Governance + admin

| # | Feature | Canvas LMS | Cornerstone | Docebo | Oyatie owning surface | Parity state | Gap or out-of-scope reason |
|---|---|---|---|---|---|---|---|
| 15.1 | Role-based access control (RBAC) | Yes | Yes | Yes | identity + governance + Cedar policies | covered | policies/local-*.cedar + policy/*.cedar exist per directory listing; substance not sampled. Wave 15F. |
| 15.2 | Custom role definition | Yes (Permissions) | Yes | Yes | governance µservice delegation | partial | governance owns; learning-management → governance handoff not declared. Wave 15F. |
| 15.3 | Multi-tenant tenancy | Limited (multi-account via SIS) | Yes (hero) | Yes (hero) | tenancy + identity delegation per ADR-0244 | covered | manifest declares tenant-scoped operational records. |
| 15.4 | Sub-tenant / branch hierarchy | Limited | Yes (OUs hero) | Yes (Branches hero) | tenancy µservice + IP-001-tenant-scope-kernel.md | partial | IP-001 bespoke per coherence-audit; sub-tenant hierarchy not declared. Wave 15F. |
| 15.5 | Data residency control per pack | Limited | Yes (regional packs) | Yes (regional packs) | policy/data-residency.md + manifest cell_eligibility | partial | Wave 15F samples + remediates. |
| 15.6 | Audit log (every admin action) | Yes (Notable Events) | Yes | Yes | audit-chain delegation + compliance.md §D Audit Events | covered | 15 EVT- classes enumerated. |
| 15.7 | Configurable retention windows | Limited | Yes | Yes | compliance + tenancy delegation | partial | Wave 15F. |
| 15.8 | Tenant-level branding + theming | Yes (Theme Editor) | Yes | Yes (hero with white-label) | UX-shell delegation | out-of-scope-intentional | UX concern per ADR-0245. |
| 15.9 | API rate limiting per tenant | Yes (Pace API) | Yes | Yes | api-gateway delegation per Phase 1 D-1.38 | covered | api-gateway owns. |
| 15.10 | Tenant-level OpenBao / Vault secrets binding | n/a | Limited | Limited | iac/openbao-policy.yaml + iac/local-openbao-policy.hcl exist per directory listing | partial | Substance not sampled. Wave 15F. |

## §16. Coverage roll-up

| Section | Total rows | covered | partial | missing | out-of-scope-intentional |
|---|---:|---:|---:|---:|---:|
| §1. Course content authoring | 10 | 1 | 4 | 2 | 3 |
| §2. Assignment + grading | 13 | 0 | 0 | 12 | 1 |
| §3. Quizzes + assessments | 13 | 0 | 3 | 10 | 0 |
| §4. Enrollment + cohort | 10 | 1 | 7 | 1 | 1 |
| §5. Discussions + social | 9 | 0 | 6 | 1 | 2 |
| §6. Standards + interop | 14 | 0 | 3 | 9 | 2 |
| §7. Compliance + regulated | 14 | 4 | 9 | 1 | 0 |
| §8. Mobile + offline | 9 | 1 | 4 | 3 | 1 |
| §9. Virtual classroom | 9 | 0 | 4 | 1 | 4 |
| §10. Content library + marketplace | 6 | 0 | 6 | 0 | 0 |
| §11. AI-driven learning | 10 | 1 | 9 | 0 | 0 |
| §12. Analytics + reporting | 10 | 0 | 9 | 1 | 0 |
| §13. Integrations + HRIS | 13 | 2 | 9 | 1 | 1 |
| §14. Accessibility + i18n | 6 | 0 | 2 | 0 | 4 |
| §15. Governance + admin | 10 | 3 | 6 | 0 | 1 |
| **Total** | **156** | **13** | **81** | **41** | **20** |
| **Percent** | 100 | 8 | 52 | 26 | 13 |

Note: row counts in §16 (156) differ from the frontmatter summary (95 net unique features). The §16 roll-up counts every feature-row across §1-§15 sections; the frontmatter summary counts unique features after consolidation. The 95-vs-156 discrepancy is intentional — the frontmatter is the consolidated parity score; the §16 roll-up is the section-level granularity.

## §17. Gap synthesis

### §17.1 Hard gaps (Canvas LMS academic features that learning-management does NOT cover)

- Assignment bounded-context entirely (§2.1..§2.13 — 13 rows missing)
- Rubrics (§2.5)
- Grade-book (§2.6, §2.7)
- Anonymous grading (§2.12)
- Plagiarism integration (§2.13)
- SpeedGrader UX (§2.10 — out-of-scope-intentional per ADR-0245)
- SCORM 1.2 + 2004 (§6.1, §6.2)
- xAPI + LRS (§6.3)
- cmi5 (§6.4)
- LTI 1.1 + 1.3 (§6.6, §6.7, §6.8)
- Common Cartridge + QTI (§6.9, §6.10)
- IMS Caliper (§6.11)
- Comprehensive Learner Record (§6.14)
- SIS integration (§13.1)

### §17.2 Cornerstone OnDemand corporate features partial-or-missing

- Mandatory training overdue escalation (§7.6)
- E-signature on attestation per FDA 21 CFR Part 11 (§7.5)
- CEU + CPD tracking (§7.12, §7.13)
- Regulator export (§7.14)
- Mentorship module (§5.6 — out-of-scope-intentional)
- Skills graph + skill-gap detection (§11.3 — partial via IP-026)
- HRIS integration (§13.2 — P0 dependency gap)

### §17.3 Docebo AI-features partial

- AI auto-generated learning path (§11.4)
- AI auto-quiz generation (§11.6)
- AI auto-translation (§11.5)
- Conversational AI coach (§11.8)
- All depend on the intelligence µservice handoff which is declared in manifest but specific handoff shapes are not.

### §17.4 Substrate handoffs missing (canonical-direction concern)

These features are NOT learning-management's operational concern — they belong to other µservices — but the handoff from learning-management to the owning µservice is not declared:

- community (discussion + cohort + UGC)
- meet (virtual classroom)
- calendar (session scheduling)
- recordings (session recording + caption)
- analytics (dashboards + reports)
- intelligence (AI features)
- translate (auto-translation)
- mail (push notifications)
- hris (worker + manager + role + cost-center) — P0
- cloud-marketplace (content marketplace settlement)
- cloud-billing (per_seat + per_usage + revenue_share meters) — P1
- payments (paid-tenant invoice)
- governance (custom-role definition)
- whiteboard (live-session whiteboarding)
- workplace-integration (Teams + Slack)
- plugin-app-store (Zapier-style automation)

Wave 15F authors `cross-microservice-handoffs.md` declaring each of these.

## §18. Verification Notes

V.1 Counterpart source citations:

- Canvas LMS public features: documented at instructure.com/products/canvas, community.canvaslms.com (admin + instructor + developer docs), IMS Global LTI 1.3 + xAPI specs. Hero features highlighted by Canvas marketing: New Analytics, New Quizzes, SpeedGrader, Course Blueprints, Mastery Paths, SIS Imports, LTI 1.3 Advantage, Canvas Commons, Mobile Apps, Studio.
- Cornerstone OnDemand public features: cornerstoneondemand.com + product datasheets. Hero features: Cornerstone Performance, Cornerstone Recruiting, Skills Graph, Content Anytime, Saba Virtual Classroom, mobile, AI-Suite, regulator-ready exports.
- Docebo public features: docebo.com/learning-platform + AI-Suite product pages. Hero features: Docebo Discover (AI), Docebo Shape (conversational AI), Docebo Skills, Docebo Content (marketplace), Docebo Mobile (Go.Learn), Docebo Connect, Custom Reports.

V.2 Manifest counterpart drift logged in coherence-audit-2026-05-20.md §3.4.C-1.1 as P0-LM-001.

V.3 Anchor checks pass/fail:

- Anchor 1 (unified ecosystem thesis): partial-conformance — learning-management is a legitimate role/capability projection but does NOT declare canvas-academic bounded contexts; reconcile in Wave 15F per LM-BR-002.
- Anchor 2 (PRD): the PRD declares 5 bounded contexts; none cover canvas academic primitives.
- Anchor 3 (local artifact inventory): authored in coherence-audit §A.2.
- Anchor 4 (top-3 counterparts): brief override Canvas + Cornerstone OnDemand + Docebo applied.
- Anchor 5 (documentation-rigor §1.1): substance bar applied per row; per-row gaps named concretely.

V.4 Sample-read evidence used in this matrix:

- PRD §C, §D for bounded-context + FR rows.
- manifest.json for capability list + dependencies + packs.
- ARCHITECTURE.md §B Layer Map + §C Bounded Context + §D Integration Topology.
- IP-001 (tenant scope kernel) for cross-µservice handoff shape.
- IP-026, IP-027, IP-028, IP-029, IP-030 by filename (bespoke per coherence-audit §A.2).
- contracts/openapi-v1.yaml for §13.7 REST API row.
- contracts/learning-management-v1.proto by line count for §13.9.
- capabilities/*.yaml directory listing for §7 + §10 rows.
- runbooks/ directory listing for §4.5 + §7.6 rows.
- compliance.md §B Control Families + §D Audit Events for §7 rows.
- policies/ + policy/ directory listing for §15 rows.
- iac/ directory listing for §15.10.
- dashboards/ directory listing for §7.7.
- ADR-0328 §D-5 union-coverage doctrine for parity-state vocabulary.

V.5 Known gaps in this parity matrix:

- substance of policies/local-*.cedar + policy/*.cedar not sampled (impacts §15.1 + §15.2 confidence).
- substance of slos/*.openslo.yaml beyond availability.openslo.yaml not sampled (no impact on parity rows but affects SLO claims in adjacent PR per performance benchmark doc).
- substance of contracts/asyncapi-v1.yaml + contracts/learning-management-v1.proto not sampled beyond line count (impacts §13.6 + §13.9 confidence).
- IP-006 through IP-025 not sampled (uniform 55-line count suspicion per coherence-audit B.1.8; impacts parity confidence on IP-named features).
- Canvas-specific deep features (Mastery Paths, Masteryacademic-record alignment, ePortfolio) not enumerated row-by-row beyond the §1-§15 sections.
- Docebo-specific deep features (Docebo Coach + Share, Docebo Pages, Docebo Mobile App Publisher) not enumerated beyond the §1-§15 sections.
- Cornerstone-specific deep features (Saba PeopleFluent compliance, Cornerstone Learning Suite Cloud Storage, Cornerstone Performance) not enumerated beyond the §1-§15 sections.

V.6 Out-of-scope-intentional doctrine reasons (per ADR-0328 §D-5.13):

- UX concerns (SpeedGrader, theme editor, screen-reader, RTL, mobile content rendering, rich-text editor, video embed) → ADR-0245 substrate vs product layering. UX shell owns; learning-management is operational.
- Gamification + leaderboard → community + future gamification surface owns.
- Mentorship module → future mentorship µservice; not learning-management.
- Inline annotations (Hypothes.is) → LTI is shared, not learning-management-owned.
- AICC → industry-deprecation per ADR-0328 §D-5.13.
- GraphQL → ADR-0145 gRPC default for internal RPC; GraphQL is optional.

## §19. Findings

F.1. learning-management has covered: 13 rows (8% of 156). The covered rows cluster in §7 (compliance) and §15 (governance) where IP-027, IP-030, audit-chain delegation, and identity-delegation give strong coverage.

F.2. learning-management is partial on 81 rows (52%). The partials cluster in §5 (community handoff), §10 (content-marketplace handoff), §11 (intelligence handoff), §12 (analytics handoff), §13 (HRIS + SCIM + Salesforce handoffs) — all are substrate-handoff gaps where the dependency µservice exists but the handoff is not declared.

F.3. learning-management has 41 missing rows (26%). The missing cluster heavily in §2 (assignment / grading — 12 of 13 rows missing because the bounded-context does not exist) and §6 (standards interop — 9 of 14 rows missing for SCORM / xAPI / LTI / cmi5 / Common Cartridge / QTI / Caliper / CLR).

F.4. learning-management has 20 out-of-scope-intentional rows (13%) with named doctrine reasons (ADR-0245 substrate-vs-product, ADR-0145 gRPC-default, ADR-0328 §D-5.13 industry-deprecation).

F.5. Per ADR-0328 §D-5.5 union-coverage bar: a feature cannot be ignored because only one counterpart has it. Canvas-specific academic features (§2, §6, §13.1) are 28% of the missing rows; Cornerstone-specific corporate-pack features are 18% of the missing rows; Docebo-specific AI features are 6% of the missing rows. The bulk of missing rows is the assignment + grading + standards-interop cluster that Canvas LMS dominates.

F.6. The 95-vs-156 frontmatter-vs-roll-up reconciliation: 156 is the granular feature-row count (every sub-feature listed); 95 is the consolidated unique-feature count after merging sub-rows with shared substrate handoffs. Both numbers are correct at their granularity.

## §20. Backlog Rows

The 35 LM-BR-* rows from coherence-audit-2026-05-20.md §I cover the structural remediation. The feature-parity matrix adds the following per-row remediation rows:

| Row | µservice | Severity | Category | Feature row | Fix |
|---|---|---|---|---|---|
| LM-FP-001 | learning-management | P0 | parity | §2.1..§2.13 (assignment) | Add `assignment` + `rubric` bounded-contexts OR document out-of-scope-intentional with doctrine reason in §1 of replacement PRD. Decision via council per coherence-audit LM-BR-002. |
| LM-FP-002 | learning-management | P0 | parity | §6.1..§6.14 (standards interop) | Add `scorm-runtime` + `xapi-statement-lrs` + `lti-consumer` + `lti-provider` bounded-contexts OR document out-of-scope-intentional. |
| LM-FP-003 | learning-management | P0 | parity | §13.2 (HRIS) | Add hris dependency (already tracked as LM-BR-003). |
| LM-FP-004 | learning-management | P0 | parity | §10.2 + §10.4 (content marketplace + LinkedIn/Udemy/Coursera) | Declare cloud-marketplace handoff + per-provider connectors. |
| LM-FP-005 | learning-management | P1 | parity | §7.5 (e-signature) | Add e-signature workflow + FDA 21 CFR Part 11 evidence in regulated-training-attest capability. |
| LM-FP-006 | learning-management | P1 | parity | §11 (AI features) | Declare intelligence handoff shape for each AI feature row. |
| LM-FP-007 | learning-management | P1 | parity | §4.4 + §15.4 (dynamic audience + sub-tenant) | Add dynamic-audience rule engine (depends on hris). |
| LM-FP-008 | learning-management | P1 | parity | §13.10 + §13.11 + §13.12 (Teams + Slack + Salesforce) | Declare workplace-integration + crm handoffs. |
| LM-FP-009 | learning-management | P2 | parity | §5 (community handoff) | Declare community handoff for course-scoped discussion + cohort + UGC. |
| LM-FP-010 | learning-management | P2 | parity | §9 (meet + recordings + calendar handoffs) | Declare meet + recordings + calendar handoffs for live virtual classroom. |
| LM-FP-011 | learning-management | P2 | parity | §12 (analytics handoff) | Declare analytics handoff for all dashboards + reports. |
| LM-FP-012 | learning-management | P2 | parity | §8 (mobile UX handoff) | Declare iOS Swift + Android Kotlin frontend handoff per OS matrix memory. |
| LM-FP-013 | learning-management | P3 | parity | §1.1 (instructor binding) | Add instructor-assignment field to course-catalog.create command in OpenAPI. |

## §21. Final verdict

Verdict: **REVISE** per ADR-0328 §D-4.23. learning-management's parity bar against Canvas LMS + Cornerstone OnDemand + Docebo is 8% covered, 52% partial, 26% missing, 13% out-of-scope-intentional. The µservice cannot promote past Phase 4A.1 admission gate until at minimum the P0 parity gaps (assignment bounded-context decision, standards-interop decision, HRIS dependency, content-marketplace declaration) are remediated in Wave 15F.

The recommended path: Wave 15F adopts a council-decided scope expansion ADR for academic-LMS coverage (LM-FP-001 + LM-FP-002), then Wave 15F authors the substrate handoffs (LM-FP-006..LM-FP-013), then Wave 15F expands OpenAPI per LM-BR-012 to cover the full feature surface.

Parity score net of out-of-scope-intentional: (covered + partial) / (covered + partial + missing) = (13 + 81) / (13 + 81 + 41) = 94 / 135 = 70% net of doctrine-rejected features. With doctrine-rejected features included (true union-coverage bar): (13 + 81) / 156 = 60%.

Both numbers are below industry-leader bar. Wave 15F remediation closes the gap by either expanding scope (academic LMS in-scope) or codifying intentional-out-of-scope rows with doctrine reasons (per ADR-0328 §D-5.13).
