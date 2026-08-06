---
id: ADR-0311
status: Rejected
date: 2026-05-20
owners:
  - council-architecture
  - council-product
  - council-privacy
  - council-security
  - council-legal
  - ops-compliance
  - ops-sre-reliability
  - axis-tenancy
  - axis-identity
  - axis-policy-engine
  - axis-audit-chain
  - axis-workplace-integration
supersedes: []
amends:
  - ADR-0244-tenant-as-universal-scoping-primitive.md (extends audience_type enum with 4 new values)
superseded_by: []
related:
  - ADR-0028-audit-chain-merkle-sealed.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0188-passkey-webauthn-as-canonical-auth.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0263-observability-emission-contract.md
  - ADR-0276-backup-portability-gdpr-art-20.md
  - ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md
  - ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md
  - ADR-0299-account-recovery-resilience.md
  - ADR-0300-whistleblower-press-freedom-anonymity.md
  - ADR-0304-cross-jurisdiction-data-conflict-resolution.md
  - ADR-0312-court-warrant-scoped-piercing.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/tenant-model.json
  - /specs/microservices/tenancy.json
  - /specs/microservices/identity.json
  - /specs/microservices/policy-engine.json
  - /specs/microservices/audit-chain.json
  - /specs/microservices/workplace-integration.json
  - /specs/microservice-manifest-schema.json
  - /specs/dual-tenant-identity-schema.json
related_memory:
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_cedar_as_universal_gate
  - feedback_tenant_as_universal_scoping_primitive
  - feedback_substrate_vs_product_layering
  - feedback_naming_justification
  - feedback_quality_performance_scalability_bar
  - feedback_clean_architecture_requirements
  - feedback_canonical_base_localization
  - feedback_no_silent_regression
  - feedback_autonomous_implementation_artifacts
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: dual-tenant-identity-personal-vs-work-boundary
purpose: >
  Codify the dual-tenant identity doctrine surfaced by the Wave-3-E
  ecosystem journey catalog (j126-j150). A single human MUST be able to
  hold two distinct tenant memberships — one personal, one employer-owned
  — bridged by the same passkey identity (per ADR-0299), with Cedar
  permits scoped per-tenant such that the employer's tenant CANNOT read
  the employee's personal-tenant surfaces even on suspicion. Work
  Messenger + Work Email + Work Drive + Work Calendar + Work Workflow
  Engine activity is tenant-owned (employer); Personal Messenger + Mail
  + Drive + Calendar + Workflow Studio + Payments + Marketplace is
  personal-tenant-owned (employee). Same human bridges both via shared
  passkey. UI MUST clearly indicate active tenant context. Cedar
  default-deny holds at the personal-tenant boundary. Subpoena piercing
  is per ADR-0312 (scope-bounded by judicial review).
enforcement_status: advisory-until-2026-09-15-blocker-thereafter
enforced_by:
  - cloud-ci/Rust gate packet dual-tenant-boundary-enforced
  - cloud-ci/Rust gate packet personal-tenant-cedar-deny
  - cloud-ci/Rust gate packet work-tenant-audit-scope-coherent
  - cloud-ci/Rust gate packet ui-tenant-context-indicator-present
  - cloud-ci/Rust gate packet per-jurisdiction-labor-law-overlay
  - cloud-ci/Rust gate packet onboarding-consent-captured
  - cloud-ci/Rust gate packet offboarding-portable-export-honored
  - cloud-ci/Rust gate packet audience-type-enum-coherence
naming_justifications:
  - name: oya-shared-dual-tenant-boundary
    layer: layer_5_shared_substrate
    bnf_segments: oya.shared.dual-tenant-boundary
    justification: >
      Per ADR-0105 13-layer canonical enum row 5 (shared-substrate), the
      crate that exposes the work-vs-personal boundary resolver, the
      per-row tenant-ownership classifier, the Cedar default-deny gate
      for personal-tenant surfaces, the per-jurisdiction labor-law
      overlay adapter, the onboarding-consent token issuer, and the
      offboarding portable-export sequencer belongs at the shared
      layer. Naming `oya-shared-dual-tenant-boundary` keeps the
      single-concern flat layout per ADR-0131 and avoids any "suite"
      packaging per ADR-0132. Drop-in companion to
      `oya-shared-whistleblower-channel` (ADR-0300),
      `oya-shared-account-recovery` (ADR-0299),
      `oya-shared-warrant-handler` (ADR-0312).
  - name: oya-governance-dual-tenant-boundary-enforced
    layer: N/A (foundry-fitness aggregate CI lane)
    bnf_segments: oya.foundry-fitness.dual-tenant-boundary-enforced
    justification: >
      Aggregate CI fitness lane per ADR-0212 buildability doctrine;
      rolls up per-µservice declaration of the row-level
      tenant-ownership shape (work-tenant vs personal-tenant), the
      Cedar fragment that forbids cross-tenant read, the UI
      tenant-context indicator, and the per-jurisdiction labor-law
      overlay. Naming follows the canonical
      `oya-governance-<concern>` shape per documentation-rigor.md
      §3.2.3 and sibling lanes from ADR-0297/0299/0300.
  - name: oya-governance-personal-tenant-cedar-deny
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.personal-tenant-cedar-deny
    justification: >
      Per-µservice child lane verifying the default-deny Cedar
      fragment is present on every µservice that hosts personal-
      tenant data; verifies no permit grants an employer-tenant
      principal read access to a personal-tenant resource without
      either (a) the principal also being the personal-tenant
      owner, or (b) a court-warrant-scoped grant per ADR-0312.
  - name: oya-governance-work-tenant-audit-scope-coherent
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.work-tenant-audit-scope-coherent
    justification: >
      Per-µservice child lane verifying every work-tenant surface's
      Cedar audit-permit declares its `tenant_id` scope explicitly
      (never wildcards); verifies internal-audit principals (Sam's
      `B2B_INTERNAL_AUDIT` audience-type) carry the right scope-
      bounded permits and CANNOT enumerate personal-tenant
      principals via side channel.
  - name: oya-governance-ui-tenant-context-indicator
    layer: N/A (foundry-fitness CI lane)
    bnf_segments: oya.foundry-fitness.ui-tenant-context-indicator
    justification: >
      Per-µservice child lane verifying every user-facing surface
      renders an unambiguous current-tenant-context indicator (per
      documentation-rigor.md §1.1 UX-floor); satisfies the
      Apple/Microsoft/Google precedent that the user MUST know
      which tenant a click is operating against.
  - name: INTERNAL_AUDITOR_3PAO
    layer: N/A (Tenant.audience_type enum value per ADR-0244)
    bnf_segments: audience_type.INTERNAL_AUDITOR_3PAO
    justification: >
      Tenant.audience_type enum extension per ADR-0244 §D-3 +
      §D-11. Identifies a principal who acts as both an internal
      auditor of a tenant AND a FedRAMP/3PAO/SOC2 third-party
      assessor; the principal carries dual scope (e.g., Inspector
      Diana Reyes at GAO is `INTERNAL_AUDITOR_3PAO` under
      `oyatie.gov.gao` AND `B2C_CONSUMER` under her personal
      tenant). The enum value enables Cedar permits to scope read
      access to audit-relevant surfaces only.
  - name: B2B_HR_ADMIN
    layer: N/A (Tenant.audience_type enum value per ADR-0244)
    bnf_segments: audience_type.B2B_HR_ADMIN
    justification: >
      Tenant.audience_type enum extension per ADR-0244 §D-3 +
      §D-11; sub-tier of `B2B_TENANT_ADMIN`. Identifies an HR
      principal (Priya Krishnan archetype) with strong tenant-
      owned access to work-surfaces (work Messenger, work Mail,
      payroll, performance reviews, benefits) but NO access to
      personal-tenant surfaces of any tenant member. Pattern
      precedent: Workday + BambooHR HR-admin scope.
  - name: B2B_INTERNAL_AUDIT
    layer: N/A (Tenant.audience_type enum value per ADR-0244)
    bnf_segments: audience_type.B2B_INTERNAL_AUDIT
    justification: >
      Tenant.audience_type enum extension per ADR-0244 §D-3 +
      §D-11; sub-tier of `B2B_TENANT_ADMIN`. Identifies a
      corporate-internal-audit principal (Sam Okafor archetype)
      with Cedar read access to work-tenant audit surfaces only;
      hard boundary at the personal-tenant edge. Pattern precedent:
      PwC + Deloitte internal-audit consulting scope.
  - name: B2C_JOB_SEEKER_ACTIVE
    layer: N/A (Tenant.audience_type enum value per ADR-0244)
    bnf_segments: audience_type.B2C_JOB_SEEKER_ACTIVE
    justification: >
      Tenant.audience_type enum extension per ADR-0244 §D-3 +
      §D-11; sub-tier of `B2C_CONSUMER`. Identifies a personal-
      tenant principal (Chris Volkov archetype) actively job-
      searching; Cedar permit unlocks Community (LinkedIn-mode +
      Handshake-mode + TeamBlind-mode) job-board features +
      Workflow Studio job-search-pipeline templates + Marketplace
      gig-income features. Set by the user; opt-in surface.
  - name: TenantBoundaryWorkPersonalRead
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: TenantBoundary.WorkPersonalRead
    justification: >
      Audit-event-class emitted whenever a work-tenant principal
      attempts to read a personal-tenant resource; registered in
      ADR-0263 central registry. The default outcome is DENY (per
      ADR-0243 Cedar default-deny); the audit event records every
      attempt for compliance evidence.
  - name: TenantBoundaryOnboardingConsent
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: TenantBoundary.OnboardingConsent
    justification: >
      Audit-event-class emitted at hire-time when the employee
      consents to work-surface audit under the active per-
      jurisdiction labor-law overlay. Consent is signed by the
      employee's passkey + sealed in the audit chain per ADR-0028.
  - name: TenantBoundaryOffboardingExport
    layer: N/A (audit-event-class taxonomy per ADR-0263)
    bnf_segments: TenantBoundary.OffboardingExport
    justification: >
      Audit-event-class emitted at offboarding when the personal-
      tenant portable export per ADR-0276 (GDPR Art. 20) is issued
      and the work-tenant access is revoked.
  - name: policy/tenant-boundary-work-vs-personal.cedar
    layer: N/A (canonical Cedar fragment filename)
    bnf_segments: policy.tenant-boundary-work-vs-personal
    justification: >
      Canonical filename for the per-µservice tenant-boundary
      Cedar fragment under the µservice's `policy/` directory per
      ADR-0246 + ADR-0243 fragment-lifecycle conventions; single-
      concern naming keeps the policy directory's contract-by-name
      invariant.
  - name: X-Oya-Tenant-Context
    layer: N/A (HTTP response header naming)
    bnf_segments: X-Oya.Tenant-Context
    justification: >
      Custom HTTP response header carrying the active tenant-
      context slug (e.g., `tenant-acme-corp` or `b2c-7f3a9c2e`)
      for the rendered surface; consumed by the UI layer to render
      the unambiguous tenant-context indicator per the
      Apple/Microsoft/Google precedent. Namespace prefix `X-Oya-`
      reserves the platform's header surface.
---

# ADR-0311: Dual-Tenant Identity — Personal-vs-Work Boundary

## Status

Proposed — 2026-05-20.

Bundled with the keystone-bundle 2026-05-20 foundational doctrine
synthesis as the **dual-tenant-identity-personal-vs-work-boundary**
ADR, surfaced by the Wave-3-E ecosystem journey catalog (j126-j150).
The catalog introduced four new persona archetypes (Inspector Diana
Reyes, Priya Krishnan, Sam Okafor, Chris Volkov) and 25 journeys whose
load-bearing constraint is that a single human MUST be able to act
under two distinct tenants without the employer's Cedar reach ever
crossing into the personal tenant.

Enforcement is `advisory-until-2026-09-15-blocker-thereafter`. The
doctrine accepts in text immediately; the CI lanes that enforce it
promote to BLOCKER on 2026-09-15 to give the 45-µservice rollout
sequenced by per-surface ownership-declaration (per §F migration) time
to land. Until 2026-09-15, validators emit findings without failing
CI; post-2026-09-15, the lanes block merge.

## Date

2026-05-20.

## §A. Context

### §A.1. The personal/work data boundary problem

Hyperscaler-grade consumer-and-enterprise platforms must serve a single
human across two strictly-separated data domains:

1. **Work surfaces** — communications, files, calendars, workflow
   executions, payments, and other artifacts produced in the course of
   employment. These surfaces belong to the employer (the work tenant).
   Per per-jurisdiction labor law (US ECPA 1986 §2511(2)(d) consent
   exception + employer-monitoring case law including *Smyth v. Pillsbury*
   1996 + KR PIPA Art. 26 [outsourcing of personal data] + EU CCPI
   Recital 47 + JP-APPI Art. 23-2), the employer has a lawful basis to
   audit work surfaces subject to onboarding-consent capture and proper
   scope-bounded Cedar permits.
2. **Personal surfaces** — communications, files, calendars, workflow
   executions, payments, marketplace activity, community posts, and
   other artifacts the human produces outside employment. These
   surfaces belong to the human (the personal tenant). The employer
   has NO lawful basis to audit personal surfaces; piercing requires
   a court warrant per ADR-0312 (this ADR's companion).

When the catalog's Wave-3-E surfaced four new personas (Diana, Priya,
Sam, Chris) and 25 journeys (j126-j150), the load-bearing constraint
became sharp: **the same passkey identity (per ADR-0299) MUST be able
to authenticate the same human into two distinct tenants, with Cedar
permits strictly scoped per-tenant, and a hard default-deny boundary
between them.** Any code path that lets an employer's principal read a
personal-tenant row — even on suspicion — is a `feedback_no_silent_regression`
violation and would shatter consumer trust at launch.

### §A.2. Hyperscaler precedents — the pattern is unambiguous

Every named consumer-and-enterprise hyperscaler operates the dual-tenant
pattern at the substrate layer + per-tenant Cedar-equivalent resolution
at the policy layer. The pattern is:

- **Apple — Personal Apple ID vs Apple Business Manager / Apple School
  Manager.** Since 2018 (per Apple's WWDC 2018 keynote and the
  Enterprise Developer Documentation), Apple operates two distinct
  identity domains: a personal Apple ID (iCloud, App Store, Messages,
  Photos, Mail) and an Apple Business Manager / Apple School Manager
  organizational identity (managed Apple IDs, Business Essentials,
  managed-device payload). The same human can hold both; the device
  enforces strict UI indicators (work-profile vs personal-profile
  context), and managed-Apple-ID administrators can NEVER read
  personal Apple ID surfaces. Apple's "User Enrollment" (introduced
  iOS 13, hardened iOS 17) explicitly separates managed and personal
  data via cryptographic volume separation. (Source: Apple Platform
  Deployment Guide 2024 "Separate work and personal data on iPhone
  and iPad"; Apple WWDC 2023 session WWDC23-10046.)
- **Microsoft — Personal Microsoft Account vs Microsoft Work or
  School Account.** Microsoft has run dual-identity domains since
  2014 (Microsoft account [MSA] vs Azure AD work/school). The same
  human can hold both via "guest" linkage in Azure AD B2B
  Collaboration; per Microsoft 365 Enterprise documentation, work-or-
  school tenant administrators CANNOT see personal OneDrive,
  personal Outlook.com, or personal Xbox surfaces — strict logical
  separation enforced by Azure AD conditional-access + Intune
  app-protection policies. Windows 11 explicitly renders work and
  personal contexts as separate tiles in Start. (Source: Microsoft
  Build 2024 "Identity and access keynote"; Azure AD docs 2024-2025
  "Personal MSA vs work/school account"; Microsoft 365 Compliance
  Manager docs 2024 "Separation of work and personal data".)
- **Google — Personal Google Account vs Google Workspace Account.**
  Google operates dual-identity since Google Apps (2006), formalized
  into Google Workspace 2020. Same human can hold both; Workspace
  admins have console access to work Drive / Gmail / Calendar but
  CANNOT read personal Drive / Gmail.com / personal Calendar. Per
  Google's Account Help "Switch between work or school account and
  personal account" doc + Google Workspace Security Whitepaper 2024,
  the boundary is enforced at the identity-provider layer with
  per-account session tokens and per-account audit logs. Android's
  "Work Profile" (since Android 5.0 Lollipop) renders the work and
  personal app drawers separately with distinct badges. (Source:
  Google Workspace Security Whitepaper 2024; Android Enterprise
  Documentation 2024 "Work profile separation".)
- **Slack — Personal Slack Account vs Slack Enterprise Workspace.**
  Slack permits a single email to hold both personal-workspace
  membership and enterprise-workspace membership; Enterprise Grid
  administrators see only the enterprise-workspace surfaces, not the
  personal-workspace ones. (Source: Slack Help Center 2024 "Sign in
  to multiple workspaces"; Slack Enterprise Grid documentation 2024.)
- **Stripe — Personal Stripe Account vs Connected Account
  Owner.** A Stripe individual user may own a personal Stripe account
  AND act as the owner of a Connected Account under a platform
  (e.g., Lyft driver with personal Stripe wallet); the platform's
  capability-list cannot bleed into personal-account data. (Source:
  Stripe Documentation 2024-2025 "Account Owner Boundary".)

The pattern is **dual identity, single human, per-tenant Cedar-
equivalent permits, hard boundary at the personal/work edge,
explicit UI indicator**. This ADR adopts the pattern verbatim with
two refinements native to oyatie's substrate:

1. The boundary is enforced by Cedar (per ADR-0243), not by application
   convention. A permit that lacks the `tenant_id` scope-binding is a
   Cedar-fragment validation error (per ADR-0294 fragment soak).
2. The same passkey (per ADR-0299) is the bridge — not two separate
   passkeys. The human authenticates once; the post-auth tenant
   selector chooses which tenant context the session inhabits. This
   matches the Apple "User Enrollment" device-context shape and the
   Google Workspace "switch account" shape.

### §A.3. Why this is its own ADR (not a subsection of ADR-0244)

ADR-0244 establishes tenant-as-universal-scoping-primitive. It already
supports multiple tenants per human via the `audience_type` enum and
the `primary_tenants[]` array. **What ADR-0244 does NOT codify** is:

- The per-µservice declaration that THIS row / event / column belongs
  to work-tenant vs personal-tenant ownership.
- The Cedar-fragment grammar for the cross-tenant default-deny
  boundary at the personal-tenant edge.
- The per-jurisdiction labor-law overlay deciding what employer audit
  scope is lawful where.
- The UI invariant that the user MUST see which tenant they are
  operating in.
- The onboarding/offboarding handshake — consent at hire, portable
  export at separation.
- The four new `audience_type` enum values surfaced by j126-j150
  (`INTERNAL_AUDITOR_3PAO`, `B2B_HR_ADMIN`, `B2B_INTERNAL_AUDIT`,
  `B2C_JOB_SEEKER_ACTIVE`).
- The layoff-cascade semantics — when employer revokes work-tenant
  access, personal-tenant survives untouched.

These constitute a doctrinal cluster sharp enough to warrant its own
ADR (per documentation-rigor.md §2 row "ADR" — ≥1500 lines, ≥2
hyperscaler precedents, failure-mode tree, capacity math, observability
hooks, rollback path, multi-region awareness, sovereign-cell awareness,
versioning + deprecation).

### §A.4. The catalog journey constraints (j126-j150) that crystallized this ADR

The Wave-3-E ecosystem catalog (`docs/user-journeys/CATALOG-j126-j150-ecosystem.md`)
introduced four new personas and 25 journeys; the journeys that
load-bear this ADR are:

- **j126** — Inspector Diana Reyes conducts a FedRAMP 3PAO audit. Her
  work-tenant principal (`oyatie.gov.gao`, `audience_type =
  INTERNAL_AUDITOR_3PAO`) carries scoped Cedar permits to read
  audit-relevant surfaces of the audited tenant. Her personal-tenant
  principal (under `b2c-<hash>`, `audience_type = B2C_CONSUMER`) is
  CRYPTOGRAPHICALLY UNREACHABLE from her work principal even though
  both bind to her single passkey.
- **j127** — Marcus's engineer resigns. Work-tenant access is revoked;
  personal-tenant surfaces (Notes, Mail, Drive, Workflow Studio
  templates, Marketplace artifacts) survive untouched. The same
  passkey continues to authenticate the personal tenant.
- **j128** — Diana uses Workflow Studio under her personal tenant for
  tax preparation. Per ADR-0243 default-deny, no GAO-tenant principal
  (including herself acting as GAO) can read her personal Workflow
  Studio executions. The boundary is enforced even when the human is
  the same.
- **j129** — Court warrant pierces personal-tenant with judicial
  oversight. The warrant scope MUST be bounded by per-ADR-0312
  judicial-review semantics; the warrant grants a *scope-bounded
  Cedar permit* with explicit expiry, action set, and resource set.
  ADR-0312 covers the warrant-handler primitive; this ADR's §D-8
  defines the cross-tenant Cedar permit grammar that the warrant
  handler emits.
- **j130** — Diana receives a bribery attempt via her personal
  Messenger. Her personal-tenant audit-chain captures the inbound
  message under her personal tenant's seal. The ombudsman per
  ADR-0300 can review her audit trail under reporter-privilege
  scope; the GAO cannot.
- **j131** — Cross-jurisdiction audit (EU vs KR discrepancy). The
  audit-tenant's `jurisdiction_code` interacts with each audited
  tenant's `jurisdiction_code`; higher-restriction wins per ADR-0304.
- **j132-j136** — Priya as HR. Her `B2B_HR_ADMIN` audience-type
  enables tenant-wide read of employee work-surfaces; the personal-
  tenant boundary still holds.
- **j137-j141** — Sam as internal audit. His `B2B_INTERNAL_AUDIT`
  audience-type grants tenant-wide read of audit-relevant work
  surfaces; the personal-tenant boundary still holds. j141 is the
  hard-boundary test case where Sam tries to read an employee's
  personal Messenger on suspicion of fraud — the request is DENIED
  by Cedar (subpoena-only path per ADR-0312).
- **j142-j147** — Chris (laid off). Work-tenant access is revoked;
  personal-tenant survives. He uses the `B2C_JOB_SEEKER_ACTIVE`
  audience-type sub-tier to unlock job-board surfaces.
- **j148-j150** — Creative ecosystem stories that touch ≥3 µservices
  each + ≥1 cross-tenant counterparty; they exercise the boundary
  under supply-chain, gig-economy, and creator-economy scenarios.

A failure to codify the dual-tenant boundary as a substrate primitive
would cascade into every one of these 25 journeys as ad-hoc per-µservice
implementations — duplicating logic, drifting in language, and creating
the silent-regression surface that `feedback_no_silent_regression`
forbids.

### §A.5. Compliance landscape — labor law by jurisdiction

The per-jurisdiction labor-law overlay (§D-6) interacts with the
following statutes:

- **United States.** Federal: Electronic Communications Privacy Act
  1986 (ECPA) §2511(2)(d) — consent exception permits employer
  interception of work communications with employee consent;
  Stored Communications Act (SCA) §2701-2712 — protects stored
  communications. State law varies: California ECPA + CCPA §1798.140
  recognizes "employment-related" carve-outs but with strong
  protections for personal accounts; New York CPLR §52 — written
  notice required for employer electronic monitoring; Connecticut
  General Statutes §31-48d — written notice required.
- **South Korea.** Personal Information Protection Act (PIPA) Art. 26
  — outsourcing of personal-information processing (employer-as-
  outsourcer pattern); Act on Promotion of Information and
  Communications Network Utilization and Information Protection
  (Network Act) Art. 28 — workplace privacy duties; Labor Standards
  Act Art. 17 — employment-contract disclosure; Industrial Safety
  and Health Act Art. 41 — privacy of mental-health surveys.
- **European Union.** GDPR Art. 6(1)(b) "necessary for the performance
  of a contract" + Art. 6(1)(f) "legitimate interests" overlap; GDPR
  Recital 155 + Art. 88 — Member States may enact specific
  employment-context rules; Working Time Directive 2003/88/EC; EU
  Whistleblower Directive 2019/1937 — workplace anonymity for whistle-
  blowers (cross-ref ADR-0300). The EU Data Protection Working Party
  Opinion 2/2017 on data processing at work + the EDPB Guidelines
  3/2019 on processing personal data through video devices clarify
  proportionality for workplace monitoring. The German Federal Data
  Protection Act §26 — strictest in EU; works-council co-determination
  is mandatory.
- **Japan.** Act on the Protection of Personal Information (APPI)
  Art. 23-2 — joint-controller arrangements between employer and
  service provider; Labor Standards Act Art. 89 — workplace rules
  must be disclosed.
- **United Kingdom.** Data Protection Act 2018 + UK GDPR Art. 6 +
  ICO Employment Practices Code 2024.
- **Australia.** Privacy Act 1988 — employee-records exemption is
  narrowing; Fair Work Act 2009 + Surveillance Devices Act (per-
  state, e.g., NSW Workplace Surveillance Act 2005).
- **Canada.** PIPEDA + Quebec Bill 64; provincial privacy law (BC
  PIPA, Alberta PIPA, Quebec Privacy Act).
- **Singapore.** Personal Data Protection Act (PDPA) 2012 + 2020
  amendments; Employment Act Cap. 91.
- **India.** Digital Personal Data Protection Act 2023; Information
  Technology Act 2000 §72A.
- **Brazil.** Lei Geral de Proteção de Dados (LGPD) 2018 Art. 7 +
  Consolidação das Leis do Trabalho (CLT) Art. 444.
- **China.** Personal Information Protection Law (PIPL) 2021 Art. 13
  + Labor Contract Law 2008 — sovereign-cloud pack overlay applies.

Each jurisdiction's overlay determines: (a) what the lawful scope of
employer monitoring is, (b) what consent/notice is mandatory at hire,
(c) what categories of work data are exempt from monitoring (e.g.,
union activity, mental-health surveys, whistleblowing tip-lines), and
(d) what offboarding portability obligations apply.

The pack overlay system (per ADR-0251) is the natural carrier for
these per-jurisdiction rules.

### §A.6. Failure modes — what goes wrong without this ADR

Without an explicit dual-tenant-boundary ADR codifying the work-vs-
personal data segregation, the following failure modes emerge:

1. **Per-µservice tenant-ownership drift.** Each µservice author
   decides ad-hoc whether a Messenger thread, a Mail thread, a Drive
   file, a Workflow execution, a Payment, or a Calendar event belongs
   to work-tenant or personal-tenant — and the decisions drift.
   (Failure mode F-DTB-001.)
2. **Cedar permit over-scope.** An HR admin's permit accidentally
   grants read access to personal-tenant rows because the permit
   omits the `tenant_id` scope-binding. (Failure mode F-DTB-002.)
3. **UI tenant-context confusion.** The user sends a personal message
   from inside the work context (or vice versa) because the UI doesn't
   render the active-tenant indicator. (Failure mode F-DTB-003.)
4. **Layoff data destruction.** Employer's offboarding script
   inadvertently deletes personal-tenant data (Notes, Marketplace,
   Workflow Studio) because the tenant-ownership boundary is implicit.
   (Failure mode F-DTB-004.)
5. **Subpoena over-pierce.** A court warrant for the work tenant
   accidentally reaches personal-tenant rows because no scope-bounded
   Cedar grant exists; ADR-0312 closes this with the
   warrant-handler primitive. (Failure mode F-DTB-005.)
6. **Cross-jurisdiction labor-law violation.** Employer in jurisdiction
   X (lenient) audits work-surface of employee resident in
   jurisdiction Y (strict); the stricter rule must win per ADR-0304.
   (Failure mode F-DTB-006.)
7. **Onboarding consent missing.** Employee was never asked to
   consent to work-surface audit at hire-time, violating ECPA §2511(2)(d)
   or KR PIPA Art. 26 or EU Art. 88 implementations. (Failure mode
   F-DTB-007.)
8. **Offboarding portability denied.** Departing employee cannot
   export their personal-tenant data because the portability path is
   not wired (GDPR Art. 20 violation). (Failure mode F-DTB-008.)
9. **Identity-mixing in audit chain.** Audit-chain emissions from
   work-tenant get co-mingled with personal-tenant emissions because
   the audit-stream selection logic is not tenant-scoped. (Failure
   mode F-DTB-009.)
10. **Internal-audit principal-identity confusion.** Sam's
    `B2B_INTERNAL_AUDIT` principal accidentally enumerates personal-
    tenant principals via a side channel (e.g., the workplace
    integration µservice's user-list endpoint). (Failure mode
    F-DTB-010.)

All ten failure modes are addressed by the §D mechanics below.

## §B. Decision

The platform adopts the **Dual-Tenant Identity — Personal-vs-Work
Boundary** doctrine. The following decisions are locked:

### §B.1. The boundary is real and enforced by Cedar

- A single human (one passkey identity per ADR-0299) MAY hold
  membership in N tenants (N ≥ 1), of which AT MOST ONE is the
  human's personal tenant (`audience_type = B2C_CONSUMER` or a
  `B2C_*` sub-tier) and ≥0 are work / employer / partner / agency
  tenants.
- A principal acting under tenant T holds Cedar permits scoped to
  `principal.tenant_id == T`. The default-deny baseline (ADR-0243)
  forbids any action where `principal.tenant_id != resource.owner_tenant`
  unless an explicit `CrossTenantGrant` (per ADR-0244 §D-4) authorizes
  the action.
- No `CrossTenantGrant` may grant a work-tenant principal read access
  to a personal-tenant resource on suspicion alone. The only legitimate
  pathways from work-tenant to personal-tenant are: (a) the human is
  also the personal-tenant owner (self-access), or (b) a
  court-warrant-scoped grant per ADR-0312.

### §B.2. Per-surface ownership is declared by the µservice

Every µservice that stores user-authored data MUST declare, per row /
event / column, whether the artifact is work-tenant-owned or personal-
tenant-owned. The declaration lives in:

- The Postgres DDL — every row-bearing table has a `tenant_ownership_class`
  CHECK constraint of {`WORK_TENANT`, `PERSONAL_TENANT`,
  `PLATFORM_OWNED`, `CROSS_TENANT_VIA_GRANT`}.
- The Cedar entity-type — every entity in `policy-engine/schemas/`
  carries a `tenant_ownership_class` attribute.
- The OpenAPI / AsyncAPI contract — every event payload carries a
  `tenant_ownership_class` field tagged in the schema.

Per-µservice ownership tables (§D-3) declare the canonical mapping for
the 45 µservices.

### §B.3. The same passkey bridges both

- One passkey (per ADR-0299 §D-1 WebAuthn passkey-as-canonical-auth)
  authenticates the human into the identity substrate.
- Post-auth, the identity µservice issues a session token bound to a
  chosen `tenant_id` from the human's `principal_id → tenant_memberships[]`
  set. The session token's `tenant_id` claim is the active tenant.
- Switching tenants requires a tenant-selector handshake (similar to
  Microsoft "switch account" or Apple "User Enrollment device-context
  switch"); the new session token's `tenant_id` claim is the new
  active tenant.
- Per-tenant audit events bind to the active tenant's audit stream;
  cross-tenant audit-event-class emissions (e.g.,
  `TenantBoundaryWorkPersonalRead`) emit under BOTH tenants' audit
  streams for forensics traceability.

### §B.4. UI MUST clearly indicate the active tenant context

- Every user-facing surface (web, mobile, terminal, CLI, IDE plugin)
  MUST render an unambiguous current-tenant-context indicator (per
  documentation-rigor.md §1.1 UX-floor).
- The indicator MUST be visible in the surface chrome (e.g., header
  badge in web; top-bar indicator in mobile; CLI prompt segment;
  IDE status-bar entry).
- The indicator MUST distinguish work-tenant from personal-tenant
  visually (per Apple "work-profile badge" + Google "managed-profile
  tint" precedents); accessible label MUST be screen-reader-readable.
- The indicator's content (the tenant slug or display name) is
  carried in the `X-Oya-Tenant-Context` HTTP response header by every
  µservice that serves user-facing content; the UI shell consumes
  this header.

### §B.5. Cedar default-deny holds at the personal-tenant edge

- The baseline Cedar fragment `policy/tenant-boundary-work-vs-personal.cedar`
  (§D-4) forbids any cross-tenant action where the source tenant has
  `audience_type IN (B2B_TENANT_ADMIN, B2B_HR_ADMIN,
  B2B_INTERNAL_AUDIT, PARTNER_AGENCY, RESELLER)` and the target tenant
  has `audience_type IN (B2C_CONSUMER, B2C_JOB_SEEKER_ACTIVE)`.
- The only valid cross-tenant grant from work-to-personal is:
  (a) self-access (same `principal_id`), (b) a court-warrant Cedar
  grant per ADR-0312 §D-1.

### §B.6. Per-jurisdiction labor-law overlay determines lawful audit scope

- Each tenant declares its `jurisdiction_code` (per ADR-0244 §D-3).
- Each compliance pack (per ADR-0251) carries a `labor_law_overlay`
  sub-field that specifies: (a) consent-text required at hire,
  (b) categories of work data that are exempt from monitoring,
  (c) notice/posting requirements, (d) portability obligations.
- Cross-jurisdiction conflicts (e.g., employer in US, employee
  resident in EU) resolve per ADR-0304 (higher-restriction wins).

### §B.7. Onboarding consent capture and offboarding portable export

- At hire-time, the employer's onboarding workflow MUST capture the
  employee's signed consent (passkey signature) to work-surface
  audit under the active per-jurisdiction labor-law overlay. The
  consent is sealed in the audit chain (per ADR-0028) and is the
  legal basis for subsequent employer audit.
- At offboarding, the employer's offboarding workflow MUST issue a
  portable-export bundle (per ADR-0276 GDPR Art. 20) for the
  employee's personal-tenant data, AND revoke the employee's
  work-tenant principal access. The personal-tenant data survives;
  the work-tenant access does not.

### §B.8. The four new `audience_type` enum values

Per ADR-0244 §D-11, the `audience_type` enum gains four new values:

- `INTERNAL_AUDITOR_3PAO` — government auditor + FedRAMP / 3PAO /
  SOC2 third-party assessor. Sub-tier of `B2B_TENANT_ADMIN`.
- `B2B_HR_ADMIN` — HR principal with tenant-wide read of work-
  surfaces but no personal-tenant reach. Sub-tier of
  `B2B_TENANT_ADMIN`.
- `B2B_INTERNAL_AUDIT` — corporate internal-audit principal with
  scoped audit read of work-surfaces. Sub-tier of `B2B_TENANT_ADMIN`.
- `B2C_JOB_SEEKER_ACTIVE` — personal-tenant principal actively job-
  searching. Sub-tier of `B2C_CONSUMER`.

### §B.9. Layoff cascade — personal-tenant survives

- When employer revokes work-tenant access (offboarding workflow),
  the personal-tenant survives untouched. The same passkey continues
  to authenticate; the personal-tenant principal is unchanged; all
  personal-tenant resources (Notes, Mail, Drive, Workflow Studio,
  Payments, Marketplace, Community) are unaffected.
- The work-tenant data may be retained per the employer's compliance
  pack (e.g., 7-year SOX retention); personal-tenant data is governed
  by the employee's personal jurisdiction's retention rules.

## §C. Consequences

The decision triggers consequences across the six engineering-rigor
dimensions of documentation-rigor.md §1.2.

### §C.1. Maintainability

- New crate `oya-shared-dual-tenant-boundary` becomes a substrate
  dependency of every user-data-bearing µservice.
- Postgres DDL across all 45 µservices grows a `tenant_ownership_class`
  CHECK constraint on user-data tables.
- Cedar fragments across all 45 µservices declare the
  `policy/tenant-boundary-work-vs-personal.cedar` baseline.
- The four new `audience_type` enum values land via amendment to
  ADR-0244; migration `0003_dual_tenant_audience_types.sql` adds
  them. Per ADR-0258 SemVer policy, the enum addition is a non-
  breaking change (additive).
- Maintenance cost is bounded by the shared substrate; per-µservice
  cost is only the DDL update + Cedar fragment + ownership-class
  declaration.

### §C.2. Observability

- New audit-event-classes: `TenantBoundaryWorkPersonalRead`,
  `TenantBoundaryOnboardingConsent`, `TenantBoundaryOffboardingExport`,
  `TenantBoundaryPersonalSurvived` (emitted at layoff to record
  that personal-tenant survived the work-tenant revocation).
- Per-µservice metrics: `oya_dual_tenant_boundary_denials_total{from_tenant,
  to_tenant, action_class}` (Prometheus counter), `oya_dual_tenant_consent_capture_latency_seconds`
  (histogram), `oya_dual_tenant_offboard_export_bytes_total{tenant_id}`
  (counter).
- Trace span shape: every cross-tenant action carries a span attribute
  `oya.tenant_boundary.from`, `oya.tenant_boundary.to`,
  `oya.tenant_boundary.permit_id`, `oya.tenant_boundary.outcome` ∈
  {`ALLOW`, `DENY`, `ALLOW_VIA_GRANT`, `ALLOW_VIA_WARRANT`}.
- Dashboards: `dashboards/dual-tenant-boundary-grafana.json` rolls
  up per-µservice boundary-denial rate, consent-capture latency, and
  offboard-export volume.

### §C.3. Scalability

- The boundary check is O(1) per-request (Cedar evaluation with the
  ADR-0246 library-first dispatch hits the in-process evaluator
  cache; tenant-ownership classification is a single column read).
- Capacity math (Little's Law): at peak 100k req/s across all
  internet-facing µservices, the boundary check adds ≤200 µs P95
  latency (Cedar evaluation P95 = 80 µs per ADR-0246 §benchmarks +
  Postgres tenant-ownership-class column read P95 = 120 µs per
  ADR-0009 cell-architecture benchmarks). 100k req/s × 200 µs = 20
  CPU-cores worth of evaluator load distributed across the per-cell
  Cedar library cohort.
- Sub-scope cardinality: per ADR-0244 §D-2 hard limit 100,000
  sub-scopes per tenant. The personal-vs-work boundary uses ≤2
  sub-scopes per tenant (the root + one specialized sub-scope, e.g.,
  `oyatie.foundry.ci-agent`), so the boundary does not strain the
  cardinality budget.
- Horizontal scale-out: the boundary substrate is stateless; it
  scales with the µservice consumer's pod count.

### §C.4. Performance

- P50 boundary-check overhead: ≤50 µs.
- P95: ≤200 µs.
- P99: ≤500 µs (Cedar evaluation tail + Postgres column read tail).
- P99.9: ≤2 ms (cache-miss path to network Cedar evaluator).
- Cold-start budget: the Cedar fragment is preloaded at µservice
  startup (per ADR-0294 fragment-soak); no cold-start surprise.
- Tail-latency mitigation: the library-first dispatch (ADR-0246
  amendment) hedges the network fallback against the in-process
  cache with a 2-of-3 fan-out for high-priority boundary checks
  (configurable per `policy_evaluation_mode`).

### §C.5. Optimization

- Per-call cost model: ≤1 Cedar evaluation + ≤1 Postgres column read
  + ≤1 audit-event emission = ~0.3 CPU-ms + ~10 KB RAM + 1 audit log
  line per ~$0.000001 marginal cost. At 100k req/s peak: ~$3.6/hour
  evaluator cost across the global cell mesh — negligible relative
  to the per-request value.
- Caching strategy: Cedar evaluator caches per `(principal_id,
  resource_owner_tenant, action_class)` tuple with TTL ≤60 s; cache
  invalidation per ADR-0294 fragment publish.
- Lazy vs eager: lazy is correct — the boundary check is on the
  request path, not pre-computed.
- Cold-vs-warm latency split: warm path 50-200 µs; cold path (cache
  miss + network Cedar) 2-5 ms.

### §C.6. Code quality

- The shared substrate crate `oya-shared-dual-tenant-boundary` MUST
  pass:
  - `cargo test` line coverage ≥85%, branch coverage ≥75%.
  - `cargo clippy -- -D warnings` (no warnings).
  - `cargo fmt --check`.
  - `cargo deny check` (no advisories).
  - `proptest` property tests on the boundary classifier (invariants:
    same-tenant → ALLOW; cross-tenant work-to-personal → DENY unless
    grant; cross-tenant personal-to-work → DENY unless grant;
    grant-bounded → ALLOW within scope only).
  - Mutation testing via `cargo-mutants` with ≥80% kill rate.
  - Loom property test for concurrent permit-revocation under
    request-in-flight conditions.
- Required test classes per consumer µservice: unit, property, fuzz
  (libFuzzer on the Cedar permit grammar), integration (against the
  in-cell Cedar evaluator), end-to-end (against staging cell mesh).
- ABI policy: the substrate crate's public surface is `#[non_exhaustive]`
  on enums; SemVer 1.x.y with deprecation cadence per ADR-0258.

### §C.7. Cross-µservice impact

- All 45 µservices declare ownership class on user-data tables.
- All user-facing surfaces (web shells, mobile shells, IDE plugins)
  render the tenant-context indicator.
- The workplace-integration µservice (per §A) becomes a load-bearing
  consumer of the substrate; its IP plan adds the consent-capture
  workflow and the offboarding portable-export workflow.

## §D. Detailed mechanics

### §D-1. Cedar entity types for the dual binding

`microservices/policy-engine/schemas/dual-tenant-binding.cedarschema`
extends the tenant-scoping Cedar schema from ADR-0244 §D-4:

```cedar
// microservices/policy-engine/schemas/dual-tenant-binding.cedarschema
// Per ADR-0311 §D-1. Cedar v4.2 grammar.
// Extends the Tenancy namespace from ADR-0244 §D-4.

namespace Tenancy {

    // PrincipalInTenant — a binding of a passkey-identity (Principal)
    // to a specific Tenant. The same principal_id (per ADR-0299
    // passkey identity) can have N such bindings; one per tenant
    // the human is a member of.
    entity PrincipalInTenant in [Tenant] = {
        "principal_id":                     String,    // stable passkey-bound identity (ADR-0299)
        "tenant_id":                        String,
        "membership_kind":                  String,    // owner | admin | member | guest | partner | reseller_agent
        "is_personal_tenant_owner":         Bool,      // TRUE iff this binding is the human's personal tenant
        "active":                           Bool,
        "joined_at":                        Long,
        "revoked_at":                       Long,
        "labels":                           Set<String>
    };

    // PassportSession — a session token bound to one tenant at a time.
    // Switching tenants requires a fresh PassportSession.
    entity PassportSession in [PrincipalInTenant] = {
        "session_id":                       String,
        "principal_id":                     String,
        "active_tenant_id":                 String,
        "active_audience_type":             String,
        "issued_at":                        Long,
        "expires_at":                       Long,
        "device_attestation":               String,
        "mfa_strength":                     String
    };

    action "SwitchActiveTenant"             appliesTo {
        principal: [PrincipalInTenant],
        resource:  [PrincipalInTenant]
    };

    action "ReadAcrossTenants"              appliesTo {
        principal: [PrincipalInTenant],
        resource:  [Resource]
    };

    action "WriteAcrossTenants"             appliesTo {
        principal: [PrincipalInTenant],
        resource:  [Resource]
    };
}
```

Cedar invariants:

- `PrincipalInTenant.is_personal_tenant_owner == TRUE` requires the
  tenant's `audience_type` to be one of `B2C_CONSUMER` or
  `B2C_JOB_SEEKER_ACTIVE`. CI lane
  `oya-governance-personal-tenant-cedar-deny` validates this.
- A human MUST have AT MOST ONE `PrincipalInTenant` binding with
  `is_personal_tenant_owner == TRUE` across all their bindings. The
  `microservices/identity/` enforces this at write-time.
- `PassportSession.active_tenant_id` MUST equal one of the human's
  active `PrincipalInTenant.tenant_id` values.

### §D-2. The 4 NEW `audience_type` enum values

Migration `microservices/tenancy/migrations/0003_dual_tenant_audience_types.sql`:

```sql
-- microservices/tenancy/migrations/0003_dual_tenant_audience_types.sql
-- Per ADR-0311 §D-2. Lands after ADR-0244's 0002_canonical_tenant_schema.sql.

-- 1. Extend the audience_type enum.
--    Postgres requires ALTER TYPE for new enum values; each is non-breaking.
ALTER TYPE audience_type ADD VALUE IF NOT EXISTS 'INTERNAL_AUDITOR_3PAO';
ALTER TYPE audience_type ADD VALUE IF NOT EXISTS 'B2B_HR_ADMIN';
ALTER TYPE audience_type ADD VALUE IF NOT EXISTS 'B2B_INTERNAL_AUDIT';
ALTER TYPE audience_type ADD VALUE IF NOT EXISTS 'B2C_JOB_SEEKER_ACTIVE';

-- 2. Add the ownership-class column to tenants for the
--    work-tenant-vs-personal-tenant classifier shortcut.
ALTER TABLE tenants ADD COLUMN IF NOT EXISTS is_personal_tenant BOOLEAN
    GENERATED ALWAYS AS (
        audience_type IN ('B2C_CONSUMER', 'B2C_JOB_SEEKER_ACTIVE')
    ) STORED;

CREATE INDEX IF NOT EXISTS idx_tenants_is_personal_tenant
    ON tenants (is_personal_tenant) WHERE is_personal_tenant;

-- 3. The principal_in_tenant bridge table.
CREATE TABLE IF NOT EXISTS principal_in_tenant (
    principal_id          TEXT        NOT NULL,
                                      -- Stable passkey-bound identity (per ADR-0299).

    tenant_id             TEXT        NOT NULL REFERENCES tenants(tenant_id)
                                                ON DELETE CASCADE,

    membership_kind       TEXT        NOT NULL
                                      CHECK (membership_kind IN
                                          ('owner', 'admin', 'member', 'guest',
                                           'partner', 'reseller_agent',
                                           'hr_admin', 'internal_audit', 'auditor_3pao')),

    is_personal_tenant_owner BOOLEAN  NOT NULL DEFAULT FALSE,

    active                BOOLEAN     NOT NULL DEFAULT TRUE,
    joined_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at            TIMESTAMPTZ,

    labels                TEXT[]      NOT NULL DEFAULT ARRAY[]::TEXT[],

    -- Audit trail
    created_by            TEXT        NOT NULL,
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    revision              BIGINT      NOT NULL DEFAULT 1,

    PRIMARY KEY (principal_id, tenant_id),

    -- Personal-tenant ownership invariant: at most one personal-tenant
    -- binding per principal_id.
    CONSTRAINT one_personal_tenant_per_principal
        EXCLUDE (principal_id WITH =) WHERE (is_personal_tenant_owner)
);

CREATE INDEX IF NOT EXISTS idx_principal_in_tenant_principal
    ON principal_in_tenant (principal_id);
CREATE INDEX IF NOT EXISTS idx_principal_in_tenant_tenant
    ON principal_in_tenant (tenant_id);
CREATE INDEX IF NOT EXISTS idx_principal_in_tenant_personal
    ON principal_in_tenant (principal_id)
    WHERE is_personal_tenant_owner;

-- 4. Citus shard.
SELECT create_distributed_table('principal_in_tenant', 'tenant_id',
                                colocate_with => 'tenants');

-- 5. The passport_session bridge.
CREATE TABLE IF NOT EXISTS passport_session (
    session_id            TEXT        PRIMARY KEY,
    principal_id          TEXT        NOT NULL,
    active_tenant_id      TEXT        NOT NULL REFERENCES tenants(tenant_id),
    active_audience_type  audience_type NOT NULL,
    issued_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at            TIMESTAMPTZ NOT NULL,
    device_attestation    TEXT,
    mfa_strength          TEXT        NOT NULL
                                      CHECK (mfa_strength IN
                                          ('none', 'totp', 'webauthn',
                                           'hardware-key', 'biometric-bound')),
    revoked_at            TIMESTAMPTZ,
    revision              BIGINT      NOT NULL DEFAULT 1
);

CREATE INDEX IF NOT EXISTS idx_passport_session_principal
    ON passport_session (principal_id);
CREATE INDEX IF NOT EXISTS idx_passport_session_tenant
    ON passport_session (active_tenant_id);

SELECT create_distributed_table('passport_session', 'active_tenant_id',
                                colocate_with => 'tenants');
```

Semantic notes:

- `principal_in_tenant.is_personal_tenant_owner` carries the
  invariant "exactly one personal tenant per human". The EXCLUDE
  constraint enforces it at the DB level.
- `principal_in_tenant.membership_kind` distinguishes the role kinds
  surfaced by j126-j150 (hr_admin, internal_audit, auditor_3pao) so
  Cedar fragments can short-circuit common cases without re-evaluating
  the full audience-type enum.
- `passport_session.active_tenant_id` is the runtime tenant context.
  Switching tenants creates a NEW session; sessions are not mutated
  in place (immutability simplifies audit).

### §D-3. Per-surface ownership declaration (which µservices have work-tenant vs personal-tenant scope at the row level)

For each of the 45 µservices, the per-µservice ARCHITECTURE.md
declares its rows' `tenant_ownership_class`. The canonical mapping
table (also published as `specs/dual-tenant-ownership-map.json`):

| µservice | Primary ownership class | Notes |
|---|---|---|
| `messenger` | Both — per-thread declared | Work-thread = WORK_TENANT; personal-thread = PERSONAL_TENANT. The thread's owning tenant is set at creation; immutable. |
| `mail` | Both — per-mailbox declared | Work mailbox = WORK_TENANT; personal mailbox = PERSONAL_TENANT. The mailbox's owning tenant is set at provision; immutable. |
| `drive` | Both — per-folder declared | Drive folders inherit ownership from the owning tenant; cross-tenant share requires explicit `CrossTenantGrant`. |
| `calendar` | Both — per-calendar declared | Work calendar = WORK_TENANT; personal calendar = PERSONAL_TENANT. Free-busy may cross via grant. |
| `workflow-engine` | Both — per-execution declared | Workflow executions carry the owning tenant of the workflow definition. |
| `workflow-studio` | Both — per-workspace declared | Per-tenant studios; cross-tenant template share via marketplace. |
| `notes` | PERSONAL_TENANT only | Notes µservice is personal-by-design; work-context notes go to drive or workflow-studio. |
| `payments` | Both — per-account declared | Work payments (employer-issued cards, payroll) = WORK_TENANT; personal payments (Stripe consumer wallet) = PERSONAL_TENANT. |
| `marketplace` | PERSONAL_TENANT primary | Marketplace listings are personal-tenant-owned even when posted by an employee acting as a small-business owner. |
| `community` | Both — per-channel declared | Work-org channels = WORK_TENANT; consumer communities (TeamBlind-mode, Handshake-mode, LinkedIn-mode) = PERSONAL_TENANT. |
| `meet` | Both — per-meeting declared | Work meeting = WORK_TENANT (host is work-principal); personal meeting = PERSONAL_TENANT. |
| `comms-email` | Mirror of mail; per-mailbox declared | |
| `connector` | Both — per-integration declared | Work integrations = WORK_TENANT; personal integrations = PERSONAL_TENANT. |
| `identity` | PLATFORM_OWNED with per-binding scope | Principal-in-tenant bindings carry the tenant scope; identity µservice owns the substrate but each binding is tenant-scoped. |
| `tenancy` | PLATFORM_OWNED | The tenant directory itself is platform-owned. |
| `policy-engine` | PLATFORM_OWNED | Cedar fragment evaluator. |
| `audit-chain` | Per-stream tenant-scoped | Each audit stream belongs to one tenant. Cross-tenant emissions stream to BOTH. |
| `observability` | PLATFORM_OWNED with per-stream scope | Metrics, traces, logs carry tenant_id label; per-tenant rollup. |
| `compliance` | Both — per-pack declared | Compliance posture per tenant. |
| `governance` | PLATFORM_OWNED | Governance lane substrate. |
| `intelligence` | Both — per-call tenant-scoped | Intelligence calls carry tenant scope per ADR-0255 amendment. |
| `ontology` | Per-graph tenant-scoped | Each ontology graph belongs to one tenant; cross-tenant projection requires grant. |
| `shorts` | PERSONAL_TENANT primary | Shorts is consumer-by-design. |
| `social` | Both — per-channel declared | |
| `feature-flags` | PLATFORM_OWNED with per-tenant override | |
| `api-gateway` | PLATFORM_OWNED with per-request scope | The gateway sees both work and personal traffic; per-request the tenant scope is the active session's tenant. |
| `cloud-iac` | PLATFORM_OWNED | |
| `cloud-secrets` | Per-binding tenant-scoped | OpenBao secret paths under `secret/<tenant_id>/...`. |
| `finops-portal` | Both — per-account declared | Work spend = WORK_TENANT; personal spend = PERSONAL_TENANT. |
| `ops-dashboard-control-center` | PLATFORM_OWNED | |
| `workplace-integration` | WORK_TENANT primary | This µservice mediates the work-tenant audit surfaces. j135's harassment-complaint flow lives here. |
| `foundry` | PLATFORM_OWNED with per-tenant scope | Foundry meta-trust per ADR-0293. |
| `cell` | PLATFORM_OWNED | |
| `forms` | Both — per-form declared | |
| `data-lake` | Per-dataset tenant-scoped | |
| `search` | Per-index tenant-scoped | |
| `vector-search` | Per-collection tenant-scoped | |
| `ml-feature-store` | Per-feature-group tenant-scoped | |
| `webhook-delivery` | Both — per-endpoint declared | |
| `scheduled-jobs` | Both — per-schedule declared | |
| `email-deliverability` | Per-tenant DKIM scoped (ADR-0273) | |
| `device-management` | Both — per-device declared | Work-managed device = WORK_TENANT (employer policy applies); personal device = PERSONAL_TENANT (employer policy does NOT apply). |
| `incident-response` | PLATFORM_OWNED with per-tenant scope | |
| `consent-management` | Per-grant tenant-scoped | Per ADR-0272. |
| `legal-hold` | Per-hold tenant-scoped | Legal hold MUST cite the holding tenant; cross-tenant hold requires court order. |

A µservice that fails to declare ownership class for every user-data
table is REVISE per the §C.6 code-quality bar; CI lane
`oya-governance-dual-tenant-boundary-enforced` flags the gap.

### §D-4. Tenant-boundary Cedar fragments

Per-µservice Cedar fragment `policy/tenant-boundary-work-vs-personal.cedar`
in every consumer µservice; the canonical content is shared via the
substrate crate and reviewed per ADR-0294 fragment soak:

```cedar
// policy/tenant-boundary-work-vs-personal.cedar
// Per ADR-0311 §D-4. Cedar v4.2. Loaded by every internet-facing
// µservice that stores user-authored data.

// Baseline: deny cross-tenant work-to-personal read.
// This is the load-bearing default-deny rule.
forbid (
    principal in Tenancy::PrincipalInTenant,
    action in [
        Tenancy::Action::"ReadAcrossTenants",
        Tenancy::Action::"WriteAcrossTenants",
        Tenancy::Action::"ReadInScope",
        Tenancy::Action::"WriteInScope",
        Tenancy::Action::"DeleteInScope"
    ],
    resource is Tenancy::Resource
)
when {
    // Source is work-tenant (employer / HR / internal-audit / agency).
    principal.tenant_id != resource.owner_tenant
    && Tenancy::Tenant::(principal.tenant_id).audience_type in [
        "B2B_TENANT_ADMIN",
        "B2B_HR_ADMIN",
        "B2B_INTERNAL_AUDIT",
        "PARTNER_AGENCY",
        "RESELLER",
        "INTERNAL_AUDITOR_3PAO"
    ]
    // Target is personal tenant.
    && Tenancy::Tenant::(resource.owner_tenant).is_personal_tenant
};

// Self-access carve-out: same human acting on own personal tenant.
// PrincipalInTenant.principal_id matches the personal-tenant binding.
permit (
    principal in Tenancy::PrincipalInTenant,
    action,
    resource is Tenancy::Resource
)
when {
    Tenancy::PrincipalInTenant::(principal).principal_id ==
        Tenancy::Tenant::(resource.owner_tenant).personal_tenant_owner_principal_id
    && Tenancy::Tenant::(resource.owner_tenant).is_personal_tenant
};

// Court-warrant carve-out: explicit grant from warrant-handler per
// ADR-0312 §D-1. The grant is materialised as a CrossTenantGrant
// with grant_kind = "court_warrant_scoped" and bounded scope.
permit (
    principal in Tenancy::PrincipalInTenant,
    action,
    resource is Tenancy::Resource
)
when {
    Tenancy::CrossTenantGrant::?some_grant.from_tenant ==
        principal.tenant_id
    && Tenancy::CrossTenantGrant::?some_grant.to_tenant ==
        resource.owner_tenant
    && Tenancy::CrossTenantGrant::?some_grant.grant_kind ==
        "court_warrant_scoped"
    && Tenancy::CrossTenantGrant::?some_grant.revoked == false
    && Tenancy::CrossTenantGrant::?some_grant.expires_at > now()
    && action in Tenancy::CrossTenantGrant::?some_grant.actions_permitted
    && resource.resource_id in Tenancy::CrossTenantGrant::?some_grant.resources_permitted
};
```

The fragment is signed per ADR-0294, soaked ≥60 s before promotion to
the live evaluator, and emitted as `TenantBoundaryFragmentPromoted`
audit event class.

### §D-5. UI invariant — current-tenant-context indicator

Per documentation-rigor.md §1.1 UX-floor, every user-facing surface
MUST render an unambiguous current-tenant-context indicator:

**Web shell**:

```html
<!-- Web shell header (rendered by every µservice's user-facing surface) -->
<div class="oya-tenant-context"
     data-tenant-id="${tenant_id}"
     data-audience-type="${audience_type}"
     data-is-personal-tenant="${is_personal_tenant}"
     aria-label="Active tenant: ${tenant_display_name}, ${audience_type_label}">
    <span class="oya-tenant-badge ${is_personal_tenant ? 'personal' : 'work'}">
        ${is_personal_tenant ? 'PERSONAL' : 'WORK'}
    </span>
    <span class="oya-tenant-name">${tenant_display_name}</span>
    <button class="oya-tenant-switch" aria-label="Switch active tenant">
        ↕
    </button>
</div>
```

**Mobile shell** (iOS / Android):

- Top-bar badge with WORK or PERSONAL label.
- Accessible label via `accessibilityLabel`.
- Tap → tenant-switcher modal (matches Microsoft "Switch account"
  pattern).

**CLI** (`oya` command, terminal):

- Prompt segment: `[work:tenant-acme-corp]$` or
  `[personal:b2c-7f3a9c2e]$`.
- `OYA_TENANT_CONTEXT` environment variable carries the active
  tenant slug.

**IDE plugin** (VS Code, JetBrains):

- Status-bar entry: `oya tenant: PERSONAL — Diana Reyes (b2c-…)` or
  `oya tenant: WORK — GAO (oyatie.gov.gao)`.

The indicator's content is carried in the `X-Oya-Tenant-Context` HTTP
response header. CI lane `oya-governance-ui-tenant-context-indicator`
validates that every internet-facing µservice emits the header and
that every shell consumes it.

### §D-6. Per-jurisdiction labor-law overlay

The per-jurisdiction labor-law overlay lives in the compliance-pack
spec extension `specs/labor-law-overlay-schema.json` (new spec):

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://specs.oyatie.local/labor-law-overlay-schema.json",
  "title": "Labor-law overlay (per ADR-0311 §D-6)",
  "_meta": {
    "purpose": "Per-jurisdiction labor-law rules attached to compliance packs",
    "binding_adr": "ADR-0311",
    "industry_citations": [
      "US ECPA 1986 §2511(2)(d)",
      "KR PIPA Art. 26 + Network Act Art. 28",
      "EU GDPR Art. 88 + Member State implementations",
      "JP APPI Art. 23-2",
      "AU Privacy Act 1988 (employee-records exemption narrowing)",
      "DE BDSG §26"
    ]
  },
  "type": "object",
  "required": ["jurisdiction_code", "consent_text_template",
               "monitoring_exempt_categories", "notice_requirements",
               "portability_obligations"],
  "properties": {
    "jurisdiction_code": {
      "type": "string",
      "pattern": "^[A-Z]{2}(-[A-Z]{1,3})?$",
      "examples": ["US", "US-CA", "US-NY", "EU", "DE", "KR", "JP",
                   "GB", "AU", "AU-NSW", "CA", "CA-QC", "SG", "IN",
                   "BR", "CN"]
    },
    "consent_text_template": {
      "type": "string",
      "description": "Mustache-template for the consent text shown at hire-time. Must include placeholders {tenant_display_name}, {jurisdiction_full_name}, {monitoring_scope_summary}, {portability_summary}."
    },
    "consent_required_at_hire": {
      "type": "boolean",
      "description": "TRUE if jurisdiction requires explicit signed consent at hire-time (US ECPA, KR PIPA, DE BDSG §26)."
    },
    "monitoring_exempt_categories": {
      "type": "array",
      "items": {"type": "string"},
      "description": "Categories of employee data NOT subject to employer monitoring (e.g., 'union_activity', 'whistleblowing_tip_line', 'mental_health_survey', 'medical_correspondence', 'works_council_communication').",
      "examples": [
        ["union_activity", "whistleblowing_tip_line"],
        ["mental_health_survey", "works_council_communication"]
      ]
    },
    "notice_requirements": {
      "type": "object",
      "description": "Notice + posting + works-council co-determination requirements.",
      "properties": {
        "written_notice_required": {"type": "boolean"},
        "posting_required": {"type": "boolean"},
        "works_council_codetermination_required": {"type": "boolean"},
        "regulator_notification_required": {"type": "boolean"}
      }
    },
    "portability_obligations": {
      "type": "object",
      "description": "Offboarding portability per GDPR Art. 20 + local equivalents.",
      "properties": {
        "personal_tenant_export_required": {"type": "boolean"},
        "export_format": {
          "type": "string",
          "enum": ["JSON", "CSV", "RFC-5322-email-archive",
                   "Matrix-export", "GDPR-Art-20-bundle"]
        },
        "export_sla_days": {"type": "integer", "minimum": 1}
      }
    },
    "monitoring_scope_lawful": {
      "type": "object",
      "description": "What categories of work-tenant monitoring are lawful in this jurisdiction.",
      "properties": {
        "messaging_audit_lawful": {"type": "boolean"},
        "email_audit_lawful": {"type": "boolean"},
        "drive_audit_lawful": {"type": "boolean"},
        "calendar_audit_lawful": {"type": "boolean"},
        "screen_capture_lawful": {"type": "boolean"},
        "keystroke_lawful": {"type": "boolean"},
        "biometric_lawful": {"type": "boolean"}
      }
    }
  }
}
```

Pack examples:

- **US-pack** — ECPA + state-level. Consent at hire required.
  Monitoring categories: messaging (lawful with consent), email
  (lawful with consent), drive (lawful with consent), calendar
  (lawful), screen capture (lawful with disclosure), keystroke
  (lawful with disclosure), biometric (state-by-state).
- **KR-pack** — PIPA Art. 26 + Network Act Art. 28. Consent at hire
  required. Mental-health-survey exempt. Whistleblowing tip-line
  exempt per Anti-Corruption Act.
- **EU-pack-baseline** — GDPR Art. 88 baseline. Member-state overlays
  layer on top (e.g., DE-pack adds BDSG §26 with works-council
  co-determination).
- **JP-pack** — APPI Art. 23-2. Consent required.
- **DE-pack** — BDSG §26. Works-council co-determination required;
  strictest in EU.

### §D-7. Layoff cascade — when employer revokes work-tenant access, personal-tenant survives untouched

j142-j147 (Chris) is the canonical worked example. The cascade
sequence:

1. **Day 0, T=00:00.** HR (Priya) executes the offboarding workflow
   in `workflow-engine`. The workflow's first step is to revoke
   Chris's `principal_in_tenant` binding for `tenant-marcus-multinational`
   (the work tenant): `UPDATE principal_in_tenant SET active = FALSE,
   revoked_at = now() WHERE principal_id = '<chris-principal>' AND
   tenant_id = 'tenant-marcus-multinational'`.
2. **T=00:01.** The identity µservice invalidates Chris's active
   `passport_session` rows where `active_tenant_id =
   tenant-marcus-multinational`.
3. **T=00:05.** The mail µservice transitions Chris's work-mailbox
   to read-only (per the work-tenant's retention rules).
4. **T=00:10.** The drive µservice transitions Chris's work-drive
   folders to ownership-transfer-pending (per tenant's retention
   rules). Personal-tenant files remain untouched.
5. **T=00:15.** The messenger µservice archives Chris's work-messenger
   threads.
6. **T=00:30.** The portable-export workflow per §D-10 issues Chris
   a personal-tenant data bundle (GDPR Art. 20).
7. **T=01:00.** Chris's same passkey continues to authenticate his
   personal-tenant principal (`b2c-<hash>`). His personal Messenger,
   Mail, Drive, Calendar, Notes, Workflow Studio, Payments, Marketplace,
   Community are unchanged.
8. **T=07-90 days.** Per the work-tenant's retention pack, the
   work-tenant data is retained per regulatory mandate (e.g., 7-year
   SOX, 5-year HIPAA, 1-year COPPA).

The cascade emits the audit event `TenantBoundaryPersonalSurvived`
under Chris's personal-tenant audit stream — recording that despite
work-tenant revocation, the personal-tenant binding survives.

j143's `laid-off-imports-work-portfolio-into-personal-tenant` exercise
shows the OPPOSITE direction: Chris CAN explicitly export work-Drive
artifacts (subject to DLP per Domain 7 of documentation-rigor.md
§3.2.4 + employer's NDA enforcement) into his personal Drive. The
export is one-way; the cross-tenant grant for this export is
`grant_kind = "offboarding_portable_export"` per §D-8.

### §D-8. Cross-tenant Cedar permit grammar

When the employer grants temporary read of work-Messenger to internal-
audit (Sam), the permit is tenant_id-scoped to the work-tenant only.
The CrossTenantGrant row shape:

```cedar
// Cross-tenant grant; ADR-0244 §D-4 entity-type with semantic refinement
// per ADR-0311 §D-8.

// Example grant: Sam (internal-audit) reads work-Messenger of employees
// for SOX-controls test (j137).
CrossTenantGrant::"grant-sox-q1-2026-001" {
    grant_id:            "grant-sox-q1-2026-001",
    from_tenant:         "tenant-marcus-multinational",   // grantor: the work tenant
    to_tenant:           "tenant-marcus-multinational",   // SAME tenant; internal scoping
    from_sub_scope:      "tenant-marcus-multinational",   // root
    to_sub_scope:        "tenant-marcus-multinational.audit", // scoped sub-scope
    grant_kind:          "internal_audit_scoped",         // NEW grant_kind value
    actions_permitted:   {
        "ReadInScope"  // read; not write/delete
    },
    resources_permitted: {
        "messenger:work-thread:*",   // work-Messenger threads
        "mail:work-mailbox:*",       // work-Mail mailboxes
        "drive:work-folder:*",       // work-Drive folders
        "workflow-engine:work-execution:*", // work-Workflow executions
        "payments:work-account:*"    // work-Payments approval chains
    },
    issued_at:           1716163200, // 2026-05-20T00:00Z
    expires_at:          1721347200, // 2026-07-18T00:00Z (60-day SOX window)
    revoked:             false,
    approver_principal:  "principal-cfo-marcus", // CFO approval signature
    evidence_uri:        "openbao://audit-chain/grant-sox-q1-2026-001.signed"
};
```

The `grant_kind` enum gains four NEW values per ADR-0311:

- `internal_audit_scoped` — Sam's pattern; work-tenant internal-audit
  read.
- `hr_admin_scoped` — Priya's pattern; HR-admin tenant-wide read.
- `court_warrant_scoped` — j129 pattern; cross-tenant work-to-personal
  read under judicial review (ADR-0312).
- `offboarding_portable_export` — Chris's j143 pattern; one-way
  work-to-personal export.

Cedar fragment for Sam's grant evaluation:

```cedar
// policy/internal-audit-scoped-grant.cedar
permit (
    principal in Tenancy::PrincipalInTenant,
    action in [Tenancy::Action::"ReadInScope"],
    resource is Tenancy::Resource
)
when {
    Tenancy::PrincipalInTenant::(principal).membership_kind == "internal_audit"
    && Tenancy::CrossTenantGrant::?grant.grant_kind == "internal_audit_scoped"
    && Tenancy::CrossTenantGrant::?grant.from_tenant ==
        Tenancy::PrincipalInTenant::(principal).tenant_id
    && Tenancy::CrossTenantGrant::?grant.from_tenant ==
        resource.owner_tenant
    && Tenancy::CrossTenantGrant::?grant.revoked == false
    && Tenancy::CrossTenantGrant::?grant.expires_at > now()
    && resource.resource_id in Tenancy::CrossTenantGrant::?grant.resources_permitted
    // Hard floor: cannot reach personal-tenant resources.
    && !Tenancy::Tenant::(resource.owner_tenant).is_personal_tenant
};
```

The hard floor `!is_personal_tenant` is the load-bearing check that
guarantees Sam's `B2B_INTERNAL_AUDIT` permit can NEVER reach an
employee's personal Messenger even if a misauthored grant attempts to.

### §D-9. Onboarding consent — per-jurisdiction labor-law-compliant employee consent

At hire-time, the employer's onboarding workflow MUST capture the
employee's signed consent to work-surface audit. The consent flow:

1. **Step 1** — HR-admin (Priya) issues an onboarding offer in
   `workplace-integration`. The offer carries the employer's tenant
   jurisdiction + active compliance pack.
2. **Step 2** — The candidate accepts the offer; their personal
   passkey (per ADR-0299) authenticates the acceptance. The candidate
   has a personal-tenant principal already (`b2c-<hash>`); they do
   NOT have a work-tenant principal yet.
3. **Step 3** — `workplace-integration` calls
   `oya-shared-dual-tenant-boundary::render_consent_text(
   {jurisdiction_code, tenant_display_name, pack_overlay,
   monitoring_scope_summary, portability_summary})`. The function
   returns the consent text using the per-jurisdiction overlay's
   `consent_text_template`.
4. **Step 4** — The candidate reads the consent text + clicks
   "I consent". The consent submission is signed by their personal
   passkey (per ADR-0299).
5. **Step 5** — `workplace-integration` creates the `principal_in_tenant`
   binding for the candidate in the employer's tenant: `INSERT INTO
   principal_in_tenant (principal_id, tenant_id, membership_kind,
   is_personal_tenant_owner, active, joined_at, created_by) VALUES
   ('<candidate-principal>', '<employer-tenant>', 'member', FALSE,
   TRUE, now(), '<priya-principal>')`.
6. **Step 6** — Emit audit event `TenantBoundaryOnboardingConsent`:

```json
{
  "event_class": "TenantBoundaryOnboardingConsent",
  "tenant_id": "tenant-marcus-multinational",
  "principal_id": "<candidate-principal>",
  "consent_text_hash": "sha256:<hash-of-text>",
  "consent_signature": "<webauthn-signature>",
  "pack_overlay_id": "us-labor-law-baseline-2026-q1",
  "jurisdiction_code": "US-CA",
  "consent_timestamp": "2026-05-20T14:32:11Z",
  "consent_evidence_uri": "openbao://audit-chain/consent-<candidate-principal>-<timestamp>.signed",
  "created_by": "<priya-principal>"
}
```

The consent evidence URI is retained under the work-tenant's audit
chain. If the employee later disputes the audit, the consent text +
signature + timestamp is the load-bearing legal evidence (matching
the SCA's affirmative-consent shape).

Per-pack consent-text examples are in the new spec file
`specs/labor-law-consent-templates.json`.

### §D-10. Offboarding handshake — work-tenant access revoked; portable data export

At offboarding, the employer's offboarding workflow MUST issue a
portable export of the employee's personal-tenant data. The handshake:

1. **Step 1** — HR initiates offboarding in `workflow-engine`. The
   workflow carries the offboarding reason (resignation, layoff,
   termination-for-cause) which determines retention rules.
2. **Step 2** — `workflow-engine` calls
   `oya-shared-dual-tenant-boundary::initiate_offboarding_export(
   {principal_id, employer_tenant, reason, jurisdiction_code})`.
3. **Step 3** — The substrate:
   - Identifies the employee's personal-tenant principal binding.
   - Calls each user-data µservice's `/v1/portable-export?
     principal_id=<id>&format=GDPR-Art-20-bundle` endpoint.
   - Aggregates the responses into a tamper-evident bundle (Merkle-
     sealed per ADR-0028).
4. **Step 4** — Bundle delivered via:
   - Encrypted download link sent to employee's personal Mail.
   - Optional direct sync to employee's personal-tenant Drive.
   - Optional postal-shipped physical media (per jurisdiction).
5. **Step 5** — Revoke work-tenant binding (per §D-7 step 1).
6. **Step 6** — Emit audit event `TenantBoundaryOffboardingExport`:

```json
{
  "event_class": "TenantBoundaryOffboardingExport",
  "tenant_id": "tenant-marcus-multinational",
  "principal_id": "<employee-principal>",
  "reason": "layoff",
  "export_format": "GDPR-Art-20-bundle",
  "export_size_bytes": 1234567,
  "export_uri": "openbao://offboarding-exports/<bundle-id>.signed",
  "export_sla_days_remaining": 30,
  "delivered_at": "2026-05-20T15:00:00Z",
  "created_by": "<priya-principal>"
}
```

The export's contents are personal-tenant-only by Cedar invariant.
Work-tenant data is NOT in the export; the employer retains it per
the work-tenant retention pack.

## §E. Implementation footprint

### §E.1. New crate `oya-shared-dual-tenant-boundary`

Layer: shared-substrate (layer 5, per ADR-0105). Single-concern flat
layout per ADR-0131. No suite packaging per ADR-0132.

```
crates/oya-shared-dual-tenant-boundary/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── boundary_classifier.rs        # tenant_ownership_class resolver
│   ├── cedar_evaluator.rs            # library-first Cedar dispatch (per ADR-0246)
│   ├── consent.rs                    # onboarding consent capture
│   ├── offboarding.rs                # offboarding portable export sequencer
│   ├── labor_law_overlay.rs          # per-jurisdiction overlay resolver
│   ├── ui_indicator.rs               # X-Oya-Tenant-Context header emitter
│   ├── audit_emitter.rs              # TenantBoundary* event emitter
│   └── types.rs                      # TenantOwnershipClass enum + types
├── tests/
│   ├── boundary_classifier_proptest.rs
│   ├── cedar_evaluator_integration.rs
│   ├── consent_capture_e2e.rs
│   ├── offboarding_export_e2e.rs
│   ├── ui_indicator_test.rs
│   ├── audit_emitter_test.rs
│   └── multi_jurisdiction_fuzz.rs
├── benches/
│   ├── boundary_check_bench.rs
│   └── cedar_eval_bench.rs
└── fuzz/
    └── boundary_input_fuzz.rs
```

Public surface (extract):

```rust
// crates/oya-shared-dual-tenant-boundary/src/lib.rs

#[non_exhaustive]
pub enum TenantOwnershipClass {
    WorkTenant,
    PersonalTenant,
    PlatformOwned,
    CrossTenantViaGrant,
}

#[non_exhaustive]
pub enum BoundaryOutcome {
    Allow,
    Deny { reason: DenyReason },
    AllowViaGrant { grant_id: String },
    AllowViaWarrant { warrant_id: String },
}

pub struct BoundaryRequest<'a> {
    pub principal_id: &'a str,
    pub principal_tenant_id: &'a str,
    pub resource_owner_tenant: &'a str,
    pub resource_ownership_class: TenantOwnershipClass,
    pub action_class: &'a str,
}

pub trait DualTenantBoundary {
    fn check(&self, req: &BoundaryRequest<'_>) -> BoundaryOutcome;
    fn capture_consent(
        &self,
        candidate: &str,
        employer_tenant: &str,
        jurisdiction: &str,
        consent_signature: &[u8],
    ) -> Result<ConsentReceipt, BoundaryError>;
    fn initiate_offboarding_export(
        &self,
        principal_id: &str,
        employer_tenant: &str,
        reason: OffboardingReason,
    ) -> Result<ExportBundle, BoundaryError>;
    fn current_tenant_context_header(&self, session: &PassportSession) -> String;
}
```

### §E.2. Per-µservice consumer of the boundary primitive

Every user-data-bearing µservice gains:

- `Cargo.toml` dependency on `oya-shared-dual-tenant-boundary`.
- `ARCHITECTURE.md` section `§dual-tenant-boundary` declaring its
  per-row ownership class per §D-3.
- `policy/tenant-boundary-work-vs-personal.cedar` Cedar fragment.
- Migration adding `tenant_ownership_class` column to user-data
  tables.
- Manifest update declaring `dual_tenant_boundary_consumer: true`.

### §E.3. New CI lanes

- `oya-governance-dual-tenant-boundary-enforced` — aggregate lane;
  rolls up the child lanes.
- `oya-governance-personal-tenant-cedar-deny` — per-µservice; checks
  the default-deny Cedar fragment is present.
- `oya-governance-work-tenant-audit-scope-coherent` — per-µservice;
  checks audit permits are scope-bounded.
- `oya-governance-ui-tenant-context-indicator` — per-µservice; checks
  the X-Oya-Tenant-Context header is emitted.
- `oya-governance-per-jurisdiction-labor-law-overlay` — per-pack;
  checks labor-law overlay schema is satisfied.

### §E.4. New specs + new amendments

- `specs/dual-tenant-identity-schema.json` — the dual-tenant binding
  + the four new audience-type values.
- `specs/dual-tenant-ownership-map.json` — the canonical per-µservice
  ownership map (§D-3).
- `specs/labor-law-overlay-schema.json` — the per-jurisdiction overlay
  schema (§D-6).
- `specs/labor-law-consent-templates.json` — the per-jurisdiction
  consent text templates.
- ADR-0244-amendment to add the four new `audience_type` enum values
  (separate amendment ADR per ADR-0244 amendment-naming convention).

### §E.5. Microservice-extension flags (not scaffolded in this ADR)

The following microservice extensions are flagged for follow-up IPs:

- `microservices/workplace-integration/IP-NN-dual-tenant-onboarding-consent.md`
- `microservices/workplace-integration/IP-NN-dual-tenant-offboarding-export.md`
- `microservices/identity/IP-NN-passport-session-tenant-switcher.md`
- `microservices/policy-engine/IP-NN-tenant-boundary-cedar-fragment-soak.md`
- `microservices/compliance/IP-NN-labor-law-overlay-packs.md`

These are NOT scaffolded in this ADR per the constraints; they appear
in the Wave-3-F authoring queue.

## §F. Migration

### §F.1. Sequencing

Apply to all 45 µservices in the following ordered waves:

**Wave 1 (substrate; t=0 to t+14d):**
- `tenancy` — apply 0003_dual_tenant_audience_types.sql.
- `identity` — add the passport_session table + tenant-switcher API.
- `policy-engine` — load the `dual-tenant-binding.cedarschema` +
  load `policy/tenant-boundary-work-vs-personal.cedar` baseline.
- `audit-chain` — register `TenantBoundary*` event classes per
  ADR-0263.

**Wave 2 (highest-risk consumer µservices; t+14d to t+30d):**
- `messenger`, `mail`, `drive`, `calendar`, `workflow-engine`,
  `workflow-studio` — these are the surfaces j126-j150 most
  explicitly exercise.

**Wave 3 (consumer-bearing µservices; t+30d to t+60d):**
- `notes`, `payments`, `marketplace`, `community`, `meet`,
  `comms-email`, `connector`, `shorts`, `social`, `forms`, `finops-portal`.

**Wave 4 (platform-owned + scoped µservices; t+60d to t+90d):**
- `compliance`, `governance`, `intelligence`, `ontology`,
  `feature-flags`, `api-gateway`, `cloud-iac`, `cloud-secrets`,
  `ops-dashboard-control-center`, `foundry`, `cell`, `data-lake`,
  `search`, `vector-search`, `ml-feature-store`, `webhook-delivery`,
  `scheduled-jobs`, `email-deliverability`, `device-management`,
  `incident-response`, `consent-management`, `legal-hold`,
  `observability`, `workplace-integration`.

### §F.2. Per-wave rollback plan

Each wave's rollback procedure:

1. **Revert µservice deployment** to the prior tag (per ADR-0254).
2. **Disable the `dual-tenant-boundary` feature flag** in
   `feature-flags` µservice for the affected µservice — the
   substrate falls back to the pre-ADR-0311 behavior (default-allow
   per ADR-0243 baseline with the legacy audience-type semantics).
3. **Roll back the Postgres migration** by reverting the
   `tenant_ownership_class` column addition (per-table; non-
   destructive — column drop without data destruction).
4. **Cedar fragment rollback** per ADR-0294 emergency rollback
   procedure: invalidate the fragment in the live evaluator within
   ≤60 s.
5. **Audit event class rollback** — `TenantBoundary*` event classes
   are not deleted; they remain in the registry but stop emitting.

### §F.3. Multi-region awareness

- The migration sequences PER REGION; each cell tier (per ADR-0248)
  runs its own migration cohort.
- Cross-region replication (per ADR-0049) carries the new columns
  forward via Citus shard replication; the `principal_in_tenant`
  table is colocated with `tenants`.
- DR pair behavior: passive DR cells receive the migration via the
  ADR-0241 active-passive replication stream; DR failover preserves
  the dual-tenant binding.
- Sovereign cloud overlays (per ADR-0240): each sovereign-cell pack
  inherits the migration; the per-jurisdiction labor-law overlay is
  pack-specific.

### §F.4. Sunset of legacy behavior

The legacy single-tenant-per-principal behavior (pre-ADR-0311) is
sunset on 2026-09-15. Until then, both behaviors coexist:

- New tenant memberships use the new `principal_in_tenant` table.
- Legacy memberships (pre-ADR-0311) are migrated lazily as principals
  authenticate; the identity µservice rewrites legacy tokens to new-
  format on first login.
- The legacy CI lane that validated single-tenant-per-principal is
  retired on 2026-09-15.

### §F.5. Versioning

- `oya-shared-dual-tenant-boundary` ships at 1.0.0 with `#[non_exhaustive]`
  on public enums; SemVer policy per ADR-0258.
- The `audience_type` enum additions are non-breaking (additive); the
  ADR-0244-amendment captures the change with a separate ADR ID.
- The `principal_in_tenant` table schema bumps per `tenants.schema_version`.

## §G. References

### §G.1. Regulatory + statutory anchors

- **United States.** ECPA 1986 §2511(2)(d); SCA 1986 §2701-2712;
  California Penal Code §631 + CCPA §1798.140 employment exemption;
  NY CPLR §52 written-notice requirement; CT Gen Stat §31-48d
  written-notice requirement; SOX 404 internal-controls.
- **South Korea.** PIPA Art. 26 + Network Act Art. 28; Labor
  Standards Act Art. 17; Anti-Corruption and Bribery Prohibition Act
  (whistleblower carve-out); Industrial Safety and Health Act Art. 41.
- **European Union.** GDPR Art. 6 + Art. 88 + Recital 155; EU
  Whistleblower Directive 2019/1937; Working Time Directive 2003/88/EC;
  EU Data Protection Working Party Opinion 2/2017; EDPB Guidelines
  3/2019.
- **Germany.** BDSG §26 (employment-context processing); Works
  Constitution Act §87 (works-council co-determination).
- **Japan.** APPI Art. 23-2; Labor Standards Act Art. 89.
- **United Kingdom.** Data Protection Act 2018; UK GDPR Art. 6;
  ICO Employment Practices Code 2024.
- **Australia.** Privacy Act 1988 (employee-records exemption);
  Fair Work Act 2009; NSW Workplace Surveillance Act 2005.
- **Canada.** PIPEDA; Quebec Bill 64; BC PIPA; Alberta PIPA.
- **Singapore.** PDPA 2012 + 2020 amendments.
- **India.** Digital Personal Data Protection Act 2023; IT Act 2000
  §72A.
- **Brazil.** LGPD 2018 Art. 7; CLT Art. 444.
- **China.** PIPL 2021 Art. 13; Labor Contract Law 2008.

### §G.2. Hyperscaler precedents

- Apple Platform Deployment Guide 2024 "Separate work and personal
  data on iPhone and iPad".
- Apple WWDC 2018 keynote; WWDC 2023 session WWDC23-10046.
- Microsoft Build 2024 "Identity and access keynote".
- Azure AD documentation 2024-2025 "Personal MSA vs work/school
  account".
- Microsoft 365 Compliance Manager 2024 "Separation of work and
  personal data".
- Google Workspace Security Whitepaper 2024.
- Android Enterprise Documentation 2024 "Work profile separation".
- Google Account Help "Switch between work or school account and
  personal account" 2024.
- Slack Help Center 2024 "Sign in to multiple workspaces".
- Slack Enterprise Grid documentation 2024.
- Stripe Documentation 2024-2025 "Account Owner Boundary".

### §G.3. Internal references

- documentation-rigor.md §1.1 (intern-buildability + hyperscaler-grade
  sub-test), §1.2 (engineering-rigor dimensions), §2 (ADR rigor row),
  §3.2.1 (ADR-adherence matrix), §3.2.5 (critical-path matrix).
- docs/user-journeys/CATALOG-j126-j150-ecosystem.md (the surfacing
  catalog).
- ADR-0244 §D-3 (audience_type enum), §D-4 (Cedar entity-types),
  §D-7 (lifecycle), §D-11 (audience_type semantics).
- ADR-0243 (Cedar default-deny gate).
- ADR-0246 + amendment (library-first Cedar dispatch).
- ADR-0247 (self-modification doctrine — internal-tenant principals).
- ADR-0263 (audit-event emission contract).
- ADR-0272 (consent-management).
- ADR-0276 (backup portability GDPR Art. 20).
- ADR-0292 (minor-user doctrine — COPPA-13 personal-tenant carve-out).
- ADR-0294 (Cedar fragment soak).
- ADR-0299 (account-recovery — passkey identity).
- ADR-0300 (whistleblower-press-freedom — cross-link j130).
- ADR-0304 (cross-jurisdiction conflict resolution — cross-link j131).
- ADR-0312 (court-warrant scoped piercing — companion ADR).

### §G.4. Catalog cross-link

This ADR was surfaced by `docs/user-journeys/CATALOG-j126-j150-ecosystem.md`.
The 25 journeys j126-j150 collectively exercise every §B decision +
every §D mechanic.

## §H. Change log

- 2026-05-20: Initial draft (this document). Surfaced by Wave-3-E
  ecosystem catalog (j126-j150). Keystone-bundle 2026-05-20.
  Companion to ADR-0312.

— End of ADR-0311 —
