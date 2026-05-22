---
doc_class: JourneyCatalogSlice
shape: Reference
status: Proposed
date: 2026-05-21
authority_tier: 2
slice_name: ecosystem-economy
journey_range: j126-j150
journey_count: 25
related_adrs:
  - ADR-0242
  - ADR-0244
  - ADR-0247
  - ADR-0249
  - ADR-0292
  - ADR-0297
  - ADR-0299
  - ADR-0300
  - ADR-0301
  - ADR-0304
  - ADR-0305
  - ADR-0307
  - ADR-0308
  - ADR-0309
  - ADR-0310
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/user-journeys/README.md
new_personas_introduced:
  - Inspector Diana Reyes — government auditor with dual-tenant identity (work + personal)
  - Priya Krishnan — corporate HR director (5000-person multinational)
  - Sam Okafor — corporate internal-audit director (Cedar-gated cross-employee read)
  - Chris Volkov — laid-off mid-career engineer (former-employer revoked; personal-tenant intact)
keystone_doctrine:
  - Work Messenger + Work Email = tenant-owned (employer can lawfully audit per labor law)
  - Personal Messenger + Personal Email = personal-tenant-owned (employer CANNOT access)
  - Same human has both — same passkey identity, two tenant memberships, Cedar scopes per tenant
ecosystem_emphasis: supply-chain-economy-ecosystem (3+ µservices per journey, ≥1 cross-tenant counterparty)
---

# Journey Catalog Slice — Ecosystem Economy (j126-j150)

This catalog slice introduces 4 new personas and 25 journeys focused on:
- **Government auditor dual-tenant identity** (j126-j131)
- **Corporate HR workflows** (j132-j136)
- **Corporate internal audit with employee-personal-tenant boundary** (j137-j141)
- **Laid-off worker re-entering the economy** (j142-j147)
- **Creative ecosystem-economy stories** (j148-j150)

Per user clarification 2026-05-21: "three or more microservices involved. think of it as ecosystem. supply chain. economy." Every journey in this slice touches ≥3 µservices and involves ≥1 cross-tenant relationship.

## New persona archetypes

### Inspector Diana Reyes (Washington DC, 47)
- **Work tenant:** US GAO (Government Accountability Office) — also acts as FedRAMP 3PAO; can audit any tenant her agency has lawful authority over
- **Personal tenant:** her own family-account (Stripe consumer + Mail + Messenger + Workflow Studio for taxes + Notes + family Calendar + Drive)
- **Critical boundary:** her personal tenant is NOT subpoenable by her agency without a separate court warrant; oyatie's reserved-namespace + Cedar default-deny enforce
- **Audience type values:** Diana-as-auditor = `INTERNAL_AUDITOR_3PAO`; Diana-as-consumer = `B2C_CONSUMER`
- **Hyperscaler precedent:** the SEC enforcement attorney pattern — same employee, two contexts, strict data-isolation by case + by jurisdiction

### Priya Krishnan (Bangalore, 39)
- **Role:** HR Director at Marcus's 5000-person multinational
- **Tenant scope:** corporate tenant; she owns the HR sub-scope `<tenant>.hr` with Cedar permits for hiring + offboarding + benefits + performance
- **Boundaries:** can read tenant-owned employee work-Messenger + work-Mail (per labor law); CANNOT read employee personal-tenant surfaces
- **Hyperscaler precedent:** Workday + BambooHR pattern — HR has strong tenant-owned access; tenant-owned-surfaces-only
- **Audience type:** `B2B_HR_ADMIN` (a new sub-tier of `B2B_TENANT_ADMIN`)

### Sam Okafor (Lagos, 35)
- **Role:** Corporate internal-audit director at Marcus's multinational
- **Tenant scope:** corporate tenant; `<tenant>.audit` sub-scope; Cedar permit reads:
  - work-Messenger archives
  - work-Mail archives
  - work-Drive contents
  - work-Workflow Engine execution logs
  - work-Payments approval chains
  - audit-chain seals
- **What Sam CANNOT access:** employee personal-tenant surfaces (Messenger/Mail/Drive); Cedar default-deny blocks even with suspicion; subpoena-only path
- **Hyperscaler precedent:** PwC / Deloitte internal-audit consulting model; Cedar permit with strong tenant-owned scope, hard boundary at personal tenant
- **Audience type:** `B2B_INTERNAL_AUDIT` (sub-tier of `B2B_TENANT_ADMIN`)

### Chris Volkov (Detroit, 33)
- **Pre-layoff state:** principal under former-employer's tenant (manufacturing-tech company)
- **Post-layoff transition:** former-employer revokes work-tenant access (work Messenger archived, work Mail archived, work Drive transferred); his PERSONAL tenant identity (same passkey) is intact
- **Job search:** uses Community (LinkedIn-mode + Handshake-mode + TeamBlind-mode) + Workflow Studio + Mail + Calendar + Notes + Payments + finops-portal
- **Hyperscaler precedent:** LinkedIn + Indeed + Hired user pattern; oyatie integrates these surfaces into a coherent job-search-pipeline
- **Audience type:** `B2C_CONSUMER` + a new sub-tier `B2C_JOB_SEEKER_ACTIVE` (Cedar permit unlocks job-board features)

## The dual-tenant identity boundary doctrine (load-bearing)

**Work surfaces = tenant-owned.** When an employee accepts employment, they consent (via per-jurisdiction labor-law-compliant onboarding) that their work Messenger + work Mail + work Calendar + work Drive + work Workflow Engine activity are:
- STORED IN the employer's tenant
- AUDITABLE by the employer per Cedar permit (subject to jurisdiction labor law)
- NOT mixed with their personal data
- Subject to retention per the employer's compliance pack

**Personal surfaces = personal-tenant-owned.** The employee's personal Messenger + Mail + Drive + Calendar + Notes + Workflow Studio + Payments + Marketplace are:
- STORED IN the employee's personal tenant
- NOT readable by the employer's Cedar permits (default-deny holds even on suspicion)
- Pierced only by a court warrant with appropriate scope (per ADR-0300 + ADR-0304)
- Subject to retention per the employee's personal jurisdiction (e.g., EU GDPR if employee is EU-resident)

**The same human bridges both** via:
- Same passkey identity (per ADR-0299)
- Two distinct `tenant_id` memberships
- Cedar permits scoped per-tenant
- UI clearly indicates which tenant context the user is operating in (per documentation-rigor.md UX-floor)

**Hyperscaler precedent:** Apple Personal vs Apple Business; Microsoft personal account vs Microsoft work/school account; Google personal vs Google Workspace. oyatie's distinction is enforced by Cedar (not just UX hint).

This doctrine should become its own ADR. **Proposed: ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md** — covers the persona/audience-type extension + Cedar default-deny + per-surface ownership invariant + cross-tenant Cedar permit grammar + per-jurisdiction labor-law overlay.

## The 25 journeys (j126-j150)

### Government auditor (j126-j131)

| ID | Slug | Personas | µservices touched | New ADRs surfaced |
|---|---|---|---|---|
| j126 | government-auditor-3pao-conducts-fedramp-audit | Diana | identity + tenancy + audit-chain + compliance + ops-dashboard + observability (6) | ADR-0311 dual-tenant boundary |
| j127 | dual-tenant-identity-employee-resigns-keeps-personal | Marcus's engineer | identity + tenancy + messenger + mail + drive + workflow-engine (6) | ADR-0311 |
| j128 | auditor-personal-side-uses-workflow-studio-for-taxes | Diana | workflow-studio + workflow-engine + connect + payments + notes + identity (6) | ADR-0311 |
| j129 | court-warrant-pierces-personal-tenant-with-judicial-oversight | Diana | identity + audit-chain + compliance + governance + workflow-engine + community (6) | ADR-0312 court-warrant-scoped-piercing |
| j130 | auditor-receives-bribery-attempt-via-personal-messenger | Diana | messenger + community + audit-chain + compliance + identity (5) | ADR-0300 cross-link |
| j131 | cross-jurisdiction-audit-eu-vs-kr-discrepancy | Diana | audit-chain + compliance + workflow-engine + tenancy + observability (5) | ADR-0304 cross-link |

### Corporate HR (Priya) (j132-j136)

| ID | Slug | µservices | Notable |
|---|---|---|---|
| j132 | hr-mass-hiring-event-100-roles | community + workflow-engine + intelligence + mail + meet + calendar + workplace-integration + identity + tenancy + compliance (10) | EU-AI-Act fairness audit on AI-screening |
| j133 | hr-conducts-layoff-with-dignity-and-compliance | workflow-engine + mail + messenger + payments + finops-portal + identity + tenancy + community + drive + compliance (10) | Triggers Chris's j142-j147 |
| j134 | hr-cross-tenant-recruitment-via-staffing-agency | community + workflow-engine + identity + tenancy + payments + workplace-integration (6) | 3rd-party staffing agency as facilitator-tenant |
| j135 | hr-handles-harassment-complaint-with-dual-tenant-boundary | community + messenger + identity + tenancy + audit-chain + compliance + workflow-engine (7) | Work surfaces auditable; personal surfaces protected |
| j136 | hr-administers-benefits-open-enrollment | workflow-engine + forms + drive + connect + payments + mail + identity + tenancy (8) | Multi-vendor benefits administration |

### Corporate internal audit (Sam) (j137-j141)

| ID | Slug | µservices | Notable |
|---|---|---|---|
| j137 | corporate-internal-audit-sox-controls-test | messenger + mail + workflow-engine + payments + audit-chain + ops-dashboard + identity + compliance (8) | Quarterly SOX 404 |
| j138 | corporate-audit-fraud-investigation-via-pattern-detection | detection + payments + workflow-engine + mail + audit-chain + community (6) | DRMP signal triggers investigation |
| j139 | internal-audit-policy-violation-cedar-permit-misuse | policy-engine + identity + audit-chain + ops-dashboard + workflow-engine (5) | Cedar over-scope detection |
| j140 | internal-audit-data-loss-prevention-egress-trip | drive + identity + workflow-engine + audit-chain + observability + workplace-integration (6) | DLP per §3.2.4 Domain 7 |
| j141 | internal-audit-respects-employee-personal-tenant-boundary | messenger + identity + audit-chain + compliance + governance (5) | The hard-boundary test case |

### Laid-off worker (Chris) (j142-j147)

| ID | Slug | µservices | Notable |
|---|---|---|---|
| j142 | layoff-day-zero-from-employees-side | identity + tenancy + workflow-engine + mail + meet + payments + messenger + drive (8) | Mirror of j133 from employee POV |
| j143 | laid-off-imports-work-portfolio-into-personal-tenant | drive + identity + audit-chain + workflow-engine + compliance + ops-dashboard (6) | Cross-tenant data export with DLP scrub |
| j144 | laid-off-builds-job-search-pipeline-in-workflow-studio | workflow-studio + workflow-engine + connect + intelligence + notes + calendar + mail (7) | AI-assisted personal job pipeline |
| j145 | laid-off-applies-via-community-handshake-linkedin-mode | community + identity + workflow-engine + tenancy + mail + meet + payments (7) | Cross-tenant onboarding flow |
| j146 | laid-off-uses-marketplace-as-temporary-income | marketplace + payments + finops-portal + identity + mail (5) | Side income while searching |
| j147 | laid-off-cohort-mutual-aid-community-channel | community + identity + messenger + workflow-engine (4) | Verified-former-employer cohort |

### Creative ecosystem (j148-j150)

| ID | Slug | µservices | Notable |
|---|---|---|---|
| j148 | supply-chain-circular-economy-electronics-recycling | marketplace + payments + workflow-engine + ontology + audit-chain + connect + community (7) | Full material-provenance chain |
| j149 | gig-economy-multi-platform-worker | payments + finops-portal + identity + tenancy + connect + community + workflow-engine (7) | Personal tenant aggregating multi-employer income |
| j150 | creator-economy-shorts-creator-monetization-stack | shorts + payments + marketplace + community + ontology + intelligence + finops-portal + identity (8) | KOSA-minor with parental dashboard |

## Per-journey artifacts (Wave-3-F authoring target)

Each of the 25 journeys produces 7 artifacts:
- story.md (≥800 lines) — concrete persona narrative
- ux-flow.md (≥400 lines) — screen-by-screen
- handshake.md (≥600 lines) — cross-µservice sequence + Cedar permits + audit events
- schemas/*.json — shared objects across µservices
- Per-µservice IP slices at `microservices/<svc>/IP-journey-j<NN>-<role>.md` (≥400 lines each)
- integration-test-plan.md (≥400 lines)
- README.md (≥300 lines)

Total: 25 × 7 = 175 artifacts; ~45,000+ lines.

## Wave-3-F dispatch plan

When Anthropic API recovers (529 overload affected j01-j125 — they need retry):
1. Retry partial j01-j125 (some have story.md + ux-flow.md + handshake.md but no per-µservice IPs)
2. Author j126-j150 fresh
3. Generate `ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md` (new ADR surfaced by this slice)
4. Generate `ADR-0312-court-warrant-scoped-piercing.md` (new ADR for j129's judicial-oversight path)

## Cross-references

- documentation-rigor.md §1.2 (engineering-rigor dimensions) + §3.2.1 (52-row ADR-adherence matrix; rows 28-48 most-relevant for this slice) + §3.2.5 (30-row critical-path matrix; rows 8/9/24/27 most-relevant)
- ADR-0244 §audience_type (extend with `INTERNAL_AUDITOR_3PAO`, `B2B_HR_ADMIN`, `B2B_INTERNAL_AUDIT`, `B2C_JOB_SEEKER_ACTIVE` — to be filed as ADR-0244 amendment)
- ADR-0297 §abuse-defence (j138 + j139 + j140 are detection-substrate consumers)
- ADR-0299 §account-recovery (Chris's passkey-survives-layoff path)
- ADR-0307 §detection-substrate (j138 fraud-pattern + j140 DLP-egress)
- The 4 Slice-8 reference PRDs (payments + identity + workflow-engine + ontology) are integration anchors
