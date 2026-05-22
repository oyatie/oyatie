---
doc_class: PersonaRoster
shape: Reference
status: Proposed
date: 2026-05-21
authority_tier: 2
purpose: |
  Canonical persona master roster for the oyatie platform. ~127 personas enumerated as
  projections of human identities across role / tenant / locale / device / skill-tier
  / workspace contexts. Same human, multiple persona-contexts. Personas are NOT separate
  users — they are projections of the same human across context dimensions per the
  2026-05-21 unified-ecosystem thesis.
related_adrs:
  - ADR-0244
  - ADR-0245
  - ADR-0247
  - ADR-0249
  - ADR-0255
  - ADR-0292
  - ADR-0299
  - ADR-0311
  - ADR-0313
  - ADR-0317
  - ADR-0318
  - ADR-0319
  - ADR-0320
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md
  - docs/user-journeys/CATALOG-j126-j150-ecosystem.md
  - docs/personas/<persona-slug>.md (30 top-priority dossiers)
planned_enforcement_ref: oya-governance-persona-roster-coverage
unified_ecosystem_thesis: |
  "all consolidated into one ecosystem"; "technical, non technical, office, non office —
  all under the same hood"; "the same human is a day-to-day consumer + enterprise
  employee + healthcare patient/worker + side-business owner + family parent — same
  passkey identity, multiple tenant memberships". Personas project a single human
  identity across many role × tenant × locale × device × skill-tier × workspace contexts.
---

# Persona Master Roster — 2026-05-21

## §1. Doctrine — Continuity of Identity

### §1.1 The unified-ecosystem thesis (load-bearing)

oyatie's persona roster is a **graph of identity projections**, not a directory of separate users. The doctrine is:

1. **One human, many contexts.** A single biological human (rooted in one passkey-bound identity per ADR-0299 account-recovery) participates in many tenants across many roles. Yejin-as-nurse and Yejin-as-parent and Yejin-as-side-business-owner are the SAME human projected across three contexts.
2. **One ecosystem, many surfaces.** Per the user's 2026-05-21 clarification — "all consolidated into one ecosystem" — the technical / non-technical / office / non-office distinction is a **projection axis**, not a tenant-partition. A forklift driver and a CISO are both oyatie principals operating under the same identity primitive, the same Cedar engine, the same audit chain.
3. **One passkey, multiple tenant memberships.** Per ADR-0311 (dual-tenant boundary), the same passkey-bound identity holds memberships in N tenants: personal tenant + employer tenant(s) + healthcare provider tenant + side-business tenant + government-services tenant + extended-family conglomerate tenant.
4. **Persona = (identity × tenant × role × workspace × locale × device × skill-tier).** A persona is a tuple of context dimensions over a single identity. The same human surfaces as different personas in different tuples.
5. **Cross-context bridges are first-class.** Every persona declares its `cross_context_personas[]` field listing OTHER personas that are the same human in different contexts. This is the §1.1 hyperscaler-precedent: Apple Personal ↔ Apple Business; Microsoft personal ↔ Microsoft work/school; Google personal ↔ Google Workspace — but oyatie enforces it via Cedar default-deny, not just UX hint.

### §1.2 Six engineering-rigor dimensions for the persona roster

Per documentation-rigor.md §1.2, the persona roster MUST address all six rigor dimensions:

| Dimension | What it means for personas | Acceptance signal |
|---|---|---|
| **Maintainability** | Personas can be added / extended without breaking adjacent personas; cross-context bridges remain stable. | Every persona has a stable slug; cross_context_personas[] is bidirectional; supersession via slug rename, not deletion. |
| **Observability** | Persona-as-principal emits audit events traceable to identity + tenant + role; UX and Cedar permits derive from persona context. | Every persona declares `audience_type`, `tenant_memberships[]`, `cedar_permit_classes[]`, `audit_event_scope`. |
| **Scalability** | The roster scales to 1000+ personas without combinatorial explosion. Axes are orthogonal; tuples compose. | 127 personas span 6 collar-colors × 4 skill-tiers × 5 workspaces × 6 locales × 4 device profiles = 2880 possible tuples; we author the 127 that anchor journeys + the most-novel collar-colors. |
| **Performance** | Cedar permit evaluation per persona stays under the 5ms budget per ADR-0246; persona context lookup is O(1). | `audience_type` is enum; `tenant_memberships[]` is indexed; cross-context lookups via passkey identity (constant time). |
| **Optimization** | Persona enumeration is lazy. We don't pre-allocate 2880 persona records; we materialize personas on demand from identity + tenant context. | Persona records are *templates*; instances are projection-derivations from identity + active tenant context. |
| **Code quality** | Every persona has a passing schema validation, a non-empty journey-range, a non-conflicting collar-color × workspace tuple. | CI lane `oya-governance-persona-schema-validate` enforces. |

### §1.3 Why the personas matter — the buildability case

A canonical doc (PRD, ADR, runbook) cannot meet §1.1 intern-buildability if the persona context is missing. An intern reading the messenger PRD cannot understand "what good looks like" for the Cedar permit roster unless they know the persona spectrum that consumes the surface. The persona roster IS the buildability substrate for every other canonical doc.

### §1.4 Authority of in-flight ADRs

Per the 2026-05-21 directive, the following in-flight ADRs are cited as authoritative even before their final commit:

- ADR-0317 (role-projection doctrine) — authoritative; codifies the projection-axis model in this doctrine.
- ADR-0318 (collar-color universality) — authoritative; the six-collar-color enum below is the canonical surface.
- ADR-0319 (front/middle/back-office distinction) — authoritative; the workspace axis below derives from this ADR.
- ADR-0320 (apprentice/intern/resident/fellow tenure tier) — authoritative; the skill-tier axis includes the in-training tier.

---

## §2. Axes of Universality

The persona roster spans the following orthogonal axes. Every persona's slug is a coordinate in this multi-dimensional space.

### §2.1 Collar-color axis (per ADR-0318)

Collar-color is the universal workforce-segmentation primitive. oyatie covers the full spectrum:

| Collar-color | Definition | Sample roles |
|---|---|---|
| **white-collar** | Office-based knowledge worker | Software engineer; HR director; CFO; lawyer; product designer |
| **blue-collar** | Skilled trades + manual labor | Forklift driver; construction worker; electrician; mechanic |
| **pink-collar** | Service + care + caretaking | Nurse; teacher; childcare; eldercare; hospitality |
| **gold-collar** | Highly specialized expert | Surgeon; airline pilot; nuclear engineer; senior researcher |
| **gray-collar** | Hybrid skilled trades + knowledge (e.g., utilities, technicians) | Field service technician; lab technician; aircraft mechanic |
| **green-collar** | Environmental + sustainability + agriculture + fisheries | Farmer; fisherman; sustainability officer; recycling operator |

A single human MAY hold roles across multiple collar-colors over a career, or simultaneously (e.g., a moonlighting nurse who runs a side-business farm = pink + green).

### §2.2 Workspace axis (per ADR-0319)

The workspace axis distinguishes WHERE a persona's primary activity happens:

| Workspace | Definition | Sample personas |
|---|---|---|
| **front-office** | Customer-facing roles | Sales AE; retail clerk; bank teller; receptionist; CSM |
| **middle-office** | Operational + risk + compliance roles | Risk manager; compliance officer; treasury ops; internal-audit |
| **back-office** | Internal-support roles | HR; finance; legal; IT; data engineering; accounting |
| **field** | On-site, mobile, non-fixed | Field-service tech; delivery driver; construction worker; police officer; farmer |
| **clinical / care** | Healthcare + care settings | Surgeon; nurse; medical resident; pharmacist; therapist |
| **executive** | C-suite + board | CEO; CFO; COO; CTO; CHRO; board director |
| **production** | Manufacturing / craft / kitchen | Restaurant cook; factory operator; baker; assembly-line worker |

### §2.3 Skill-tier / tenure axis (per ADR-0320)

The skill-tier axis distinguishes the level of expertise + tenure:

| Skill-tier | Definition | Cedar permit posture |
|---|---|---|
| **in-training** | Apprentice / intern / co-op / resident / fellow | Lower-scope Cedar permits; mandatory supervisor co-sign on high-stakes operations |
| **junior** | Early-career (0-3y in role) | Standard Cedar permits; coaching layer active |
| **mid-level** | Mid-career (3-10y in role) | Full Cedar permits within scope |
| **senior / staff** | Established expert (10-20y) | Approval authority on cross-team / cross-tenant operations |
| **principal / distinguished** | Recognized leader (20y+) | Architectural-decision Cedar permits |
| **executive** | C-suite + VP-tier | Board-of-directors Cedar permit class |

### §2.4 Locale axis (multi-jurisdictional)

| Locale | Pack overlay | Sample personas |
|---|---|---|
| KR (Korea) | KR-CSAP + KR-Privacy + KR-Labor-Act | Yejin Park; Aoki Tanaka (KR ops) |
| US (United States) | SOC2 + CCPA + state-by-state | Diana Reyes; Carlos Martinez |
| EU (European Union) | GDPR + DSA + EU-AI-Act + DORA | Anya Mironova; Tomáš Novák |
| JP (Japan) | APPI + JP-Labor-Standards-Act | Hiroshi Tanaka; Captain Chen |
| IN (India) | DPDP-2023 + RBI-pack | Priya Krishnan; Aiyana Singh |
| BR (Brazil) | LGPD + BR-Labor-Code | Tomás García; Sofia Rezende |

### §2.5 Device-profile axis

| Device profile | Surfaces | Accessibility considerations |
|---|---|---|
| **mobile-primary** | iOS / Android — phone is the daily-driver | Touch-target ≥44pt; offline-first; low-bandwidth fallback |
| **desktop-primary** | macOS / Windows / Linux — full keyboard + multi-monitor | Keyboard shortcuts; multi-window; clipboard chaining |
| **handheld-rugged** | Industrial scanner / rugged Android (warehouse, field) | Glove-friendly inputs; dim-light readable; drop-rated |
| **kiosk / shared** | Shop floor / front-desk shared terminal | Quick-switch user identity; no PII persistence on device |
| **assistive** | Screen-reader / switch-control / voice-only | WCAG 2.2 AAA; semantic markup; voice-first IVR |
| **vehicle-mount** | Truck cab / delivery vehicle / police cruiser | Hands-free voice; large fonts; legal-while-driving constraints |

### §2.6 Audience-type axis (per ADR-0244)

`audience_type` enum values in scope for this roster:

`B2C_CONSUMER`, `B2C_FAMILY_PARENT`, `B2C_JOB_SEEKER_ACTIVE`, `B2C_MINOR_UNDER_13` (COPPA-blocked), `B2C_MINOR_14_17` (KOSA-tiered), `B2B_EMPLOYEE`, `B2B_TENANT_ADMIN`, `B2B_HR_ADMIN`, `B2B_INTERNAL_AUDIT`, `B2B_CSUITE`, `B2B_BOARD_DIRECTOR`, `B2B_CONTRACTOR`, `B2B_APPRENTICE_INTERN`, `B2B_MEDICAL_RESIDENT`, `B2B_FIELD_WORKER`, `B2B_KIOSK_USER`, `B2B_BANK_INTERNAL`, `B2B_HEALTHCARE_PROVIDER`, `B2B_HEALTHCARE_PATIENT`, `B2B_REGULATOR_EXTERNAL`, `B2B_EXTERNAL_AUDITOR`, `B2B_EXTERNAL_COUNSEL`, `B2B_INVESTOR_LP`, `B2B_CHANNEL_PARTNER`, `INTERNAL_AUDITOR_3PAO`, `GOV_INSPECTOR`, `EDU_TEACHER`, `EDU_STUDENT`, `EDU_PARENT`, `RELIGIOUS_LEADER`, `LAW_ENFORCEMENT`, `EMERGENCY_RESPONDER`.

---

## §3. The Persona Graph — Master Table

The roster below enumerates ~127 personas. Each row provides: name + role + collar-color + workspace + skill-tier + device-profile + locale + audience_type + cross-context bridges. Personas in **bold** have full dossiers in `/Users/jasonlee/oyatie/docs/personas/<slug>.md`.

### §3.1 Original archetypes (10) — anchored to existing journeys j01-j150

| # | Name | Role | Collar | Workspace | Skill | Device | Locale | audience_type | Cross-context |
|---:|---|---|---|---|---|---|---|---|---|
| 1 | **Yejin Park** | Nurse + parent + side-business owner | pink + green | clinical + field | mid-level | mobile-primary | KR | B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2C_CONSUMER | Yejin-as-nurse / Yejin-as-parent / Yejin-as-side-business-owner = same human |
| 2 | **Marcus Chen** | Multinational CEO (5000-person) + spouse + father | white + gold | executive + field | executive | desktop-primary | KR + US | B2B_CSUITE + B2C_CONSUMER + B2C_FAMILY_PARENT | Marcus-as-CEO / Marcus-as-husband / Marcus-as-father = same human |
| 3 | **Aiyana Singh** | Senior ML engineer + tech-blogger + parent | white | back-office | senior | desktop-primary | IN | B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT | Aiyana-at-work / Aiyana-as-blogger / Aiyana-as-parent |
| 4 | **Tomás García** | Restaurant owner + family father + side artisan | white + green | executive + production | senior | mobile-primary | BR | B2B_TENANT_ADMIN + B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT | Tomás-as-owner / Tomás-as-cook / Tomás-as-father |
| 5 | **Hiroshi Tanaka** | Retired widower + grandfather + village photographer | white (retired) | field | senior (retired) | mobile-primary + assistive | JP | B2C_CONSUMER + B2C_FAMILY_PARENT | Hiroshi-as-grandfather / Hiroshi-as-photographer |
| 6 | **Anya Mironova** | Investigative journalist + activist + parent | white | field | senior | desktop + mobile | EU | B2C_CONSUMER + B2C_FAMILY_PARENT + B2C_JOB_SEEKER_ACTIVE (freelance) | Anya-as-journalist / Anya-as-parent / Anya-as-activist |
| 7 | **Diana Reyes** | GAO auditor (3PAO) + spouse + parent (US) | white | middle-office + field | senior | desktop + mobile | US | INTERNAL_AUDITOR_3PAO + B2C_CONSUMER + B2C_FAMILY_PARENT | Diana-as-auditor / Diana-as-consumer |
| 8 | **Priya Krishnan** | HR Director (Marcus's 5000-person multinational) | white | back-office | senior | desktop-primary | IN | B2B_HR_ADMIN + B2C_CONSUMER | Priya-at-work / Priya-as-consumer |
| 9 | **Sam Okafor** | Corporate Internal-Audit Director | white | middle-office | senior | desktop-primary | NG | B2B_INTERNAL_AUDIT + B2C_CONSUMER | Sam-at-work / Sam-as-consumer |
| 10 | **Chris Volkov** | Laid-off mid-career engineer (job-search) | white | back-office | mid-level | desktop + mobile | US | B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT | Chris-pre-layoff / Chris-post-layoff / Chris-as-family-provider |

### §3.2 Non-office collar-color (13)

| # | Name | Role | Collar | Workspace | Skill | Device | Locale | audience_type | Cross-context |
|---:|---|---|---|---|---|---|---|---|---|
| 11 | **Carlos Martinez** | Forklift driver, warehouse | blue | field | mid-level | handheld-rugged | US | B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT | Carlos-at-work / Carlos-as-father |
| 12 | **Sarah Kim** | Delivery driver (Amazon DSP) | blue | field | mid-level | vehicle-mount + mobile | US | B2B_FIELD_WORKER + B2C_CONSUMER | Sarah-as-driver / Sarah-as-side-hustler |
| 13 | Ahmad Hassan | Construction site lead | blue | field | senior | handheld-rugged + mobile | US | B2B_FIELD_WORKER + B2B_CONTRACTOR + B2C_CONSUMER | Ahmad-as-site-lead / Ahmad-as-contractor |
| 14 | Maria Santos | Restaurant cook | pink + green | production | mid-level | kiosk + mobile | US | B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT | Maria-at-work / Maria-as-mother |
| 15 | Devon Williams | Field-service technician (HVAC) | gray | field | senior | vehicle-mount + mobile | US | B2B_FIELD_WORKER + B2B_CONTRACTOR | Devon-at-work / Devon-as-handyman-side-business |
| 16 | Jordan Lee | Retail clerk (department store) | pink | front-office | junior | kiosk | US | B2B_EMPLOYEE + B2C_CONSUMER + B2C_MINOR_14_17 (Jordan is 17) | Jordan-at-work / Jordan-as-minor / Jordan-as-student |
| 17 | **Ms. Patel** | High-school teacher | pink | front-office | senior | desktop + mobile | UK | EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT | Patel-as-teacher / Patel-as-mother / Patel-as-student-mentor |
| 18 | Coach Park | Youth soccer coach | pink | front-office | mid-level | mobile-primary | KR | EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT | Park-as-coach / Park-as-day-job-engineer / Park-as-father |
| 19 | **Father Lopez** | Catholic priest (parish + chaplaincy) | pink | front-office | senior | desktop + mobile | ES | RELIGIOUS_LEADER + B2C_CONSUMER | Lopez-as-priest / Lopez-as-counselor |
| 20 | **Captain Chen** | Airline pilot (long-haul) | gold | field | senior | vehicle-mount + mobile | SG | B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT | Chen-as-pilot / Chen-as-father |
| 21 | **Officer Rodriguez** | Police patrol officer | gray | field | mid-level | vehicle-mount + mobile | US | LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT | Rodriguez-on-patrol / Rodriguez-as-family |
| 22 | **Dr. Tanaka** | Cardiothoracic surgeon | gold | clinical | principal | desktop + mobile + clinical-PACS | JP | B2B_HEALTHCARE_PROVIDER + B2C_CONSUMER + B2C_FAMILY_PARENT | Dr.Tanaka-as-surgeon / Dr.Tanaka-as-father |
| 23 | **Tomás García Jr.** | Coffee farmer (third-generation) | green | field | senior | mobile + handheld-rugged | BR | B2B_TENANT_ADMIN + B2C_CONSUMER + B2C_FAMILY_PARENT | Tomás-Jr-as-farmer / Tomás-Jr-as-cooperative-board / son-of-Tomás-García |
| 24 | Captain Olufemi | Commercial fisherman | green | field | senior | vehicle-mount + handheld-rugged | NG | B2B_TENANT_ADMIN + B2C_CONSUMER | Olufemi-at-sea / Olufemi-as-cooperative-member |

### §3.3 Office C-suite (10)

| # | Name | Role | Collar | Workspace | Skill | Device | Locale | audience_type | Cross-context |
|---:|---|---|---|---|---|---|---|---|---|
| 25 | **CEO Aoki Tanaka** | CEO, mid-large enterprise | white | executive | executive | desktop-primary + mobile | JP | B2B_CSUITE + B2C_CONSUMER | Aoki-as-CEO / Aoki-as-board-director-elsewhere |
| 26 | **CFO Helena Brandt** | CFO, public-company | white | executive | executive | desktop-primary | DE | B2B_CSUITE + B2C_CONSUMER | Helena-as-CFO / Helena-as-board-of-charity |
| 27 | COO Akira Watanabe | COO | white | executive | executive | desktop-primary | JP | B2B_CSUITE + B2C_CONSUMER | Akira-as-COO / Akira-as-family |
| 28 | CTO Diego Vargas | CTO | white | executive | executive | desktop + mobile | US | B2B_CSUITE + B2C_CONSUMER | Diego-as-CTO / Diego-as-side-startup-founder |
| 29 | **CHRO Linda Foster** | Chief Human Resources Officer | white | executive | executive | desktop-primary | US | B2B_CSUITE + B2B_HR_ADMIN + B2C_CONSUMER | Linda-as-CHRO / Linda-as-mentor-board |
| 30 | CMO Felix Ng | Chief Marketing Officer | white | executive | executive | desktop-primary | SG | B2B_CSUITE + B2C_CONSUMER | Felix-as-CMO / Felix-as-podcaster-side |
| 31 | CCO Naveen Iyer | Chief Compliance / General Counsel | white | executive | executive | desktop-primary | IN | B2B_CSUITE + B2C_CONSUMER | Naveen-as-CCO / Naveen-as-pro-bono-counsel |
| 32 | **CISO Yuki Park** | Chief Information Security Officer | white | executive | executive | desktop-primary + air-gapped-mobile | KR | B2B_CSUITE + B2C_CONSUMER | Yuki-as-CISO / Yuki-as-incident-response-volunteer |
| 33 | CSO Mira Goldberg | Chief Strategy Officer | white | executive | executive | desktop-primary | US | B2B_CSUITE + B2C_CONSUMER | Mira-as-CSO / Mira-as-board-elsewhere |
| 34 | **Board director Patrick O'Reilly** | Independent board director, 3 boards | white | executive | principal | desktop + mobile | IE | B2B_BOARD_DIRECTOR + B2C_CONSUMER | O'Reilly-on-Board-A / O'Reilly-on-Board-B / O'Reilly-on-Board-C |

### §3.4 Office functional managers (14)

| # | Name | Role | Collar | Workspace | Skill | Device | Locale | audience_type | Cross-context |
|---:|---|---|---|---|---|---|---|---|---|
| 35 | Engineering Manager (Aisha Ali) | Eng Manager | white | back-office | senior | desktop | US | B2B_EMPLOYEE + B2C_CONSUMER | Aisha-as-manager / Aisha-as-open-source-maintainer |
| 36 | Product Manager Lily Chang | Product Manager | white | back-office | senior | desktop + mobile | US | B2B_EMPLOYEE + B2C_CONSUMER | Lily-as-PM / Lily-as-side-startup-founder |
| 37 | Sales Manager Anthony Costa | Sales Manager | white | front-office | senior | desktop + mobile | US | B2B_EMPLOYEE + B2C_CONSUMER | Anthony-as-manager / Anthony-as-podcast-host |
| 38 | Marketing Manager Olu Adeyemi | Marketing Manager | white | front-office | senior | desktop + mobile | NG | B2B_EMPLOYEE + B2C_CONSUMER | Olu-as-manager / Olu-as-content-creator |
| 39 | Customer Success Manager Sofia Rezende | CSM | white | front-office | senior | desktop + mobile | BR | B2B_EMPLOYEE + B2C_CONSUMER | Sofia-as-CSM / Sofia-as-family |
| 40 | Finance Director Mei-Ling Wu | Finance Director | white | back-office | senior | desktop | TW | B2B_EMPLOYEE + B2C_CONSUMER | Mei-Ling-as-director / Mei-Ling-as-CPA-volunteer |
| 41 | HRBP Jamal Carter | HR Business Partner | white | back-office | mid-level | desktop + mobile | US | B2B_HR_ADMIN + B2C_CONSUMER | Jamal-as-HRBP / Jamal-as-mentor |
| 42 | Recruiting Manager Hina Suzuki | Recruiting Manager | white | back-office | senior | desktop + mobile | JP | B2B_HR_ADMIN + B2C_CONSUMER | Hina-as-manager / Hina-as-bootcamp-instructor |
| 43 | Procurement Manager Wei Liu | Procurement Manager | white | back-office | senior | desktop | CN | B2B_EMPLOYEE + B2C_CONSUMER | Wei-as-manager / Wei-as-supplier-of-side-business |
| 44 | Legal Counsel Anika Mehta | In-house Counsel | white | back-office | senior | desktop | IN | B2B_EMPLOYEE + B2C_CONSUMER | Anika-as-counsel / Anika-as-pro-bono |
| 45 | Compliance Officer Tunde Bello | Compliance Officer | white | middle-office | senior | desktop | NG | B2B_EMPLOYEE + B2C_CONSUMER | Tunde-as-officer / Tunde-as-PTA-member |
| 46 | DevOps Manager Pavel Korsak | DevOps Manager | white | back-office | senior | desktop + mobile | UA | B2B_EMPLOYEE + B2C_CONSUMER | Pavel-as-manager / Pavel-as-open-source-maintainer |
| 47 | IT Manager Jamie O'Connor | IT Manager | white | back-office | senior | desktop | IE | B2B_EMPLOYEE + B2B_HR_ADMIN (IT-side) + B2C_CONSUMER | Jamie-as-manager / Jamie-as-PC-club-organizer |
| 48 | Office Manager Priya Ramanathan | Office Manager | white | back-office | mid-level | desktop + mobile | IN | B2B_EMPLOYEE + B2C_CONSUMER | Priya-R-as-manager / Priya-R-as-PTA-treasurer |

### §3.5 Office functional ICs (30)

| # | Name | Role | Collar | Workspace | Skill | Device | Locale | audience_type | Cross-context |
|---:|---|---|---|---|---|---|---|---|---|
| 49 | Software Engineer Hugo Tanaka | SWE | white | back-office | mid-level | desktop + mobile | JP | B2B_EMPLOYEE + B2C_CONSUMER | Hugo-at-work / Hugo-as-open-source-contrib |
| 50 | Sales AE Maya Lindqvist | Account Executive | white | front-office | mid-level | desktop + mobile | SE | B2B_EMPLOYEE + B2C_CONSUMER | Maya-as-AE / Maya-as-podcaster |
| 51 | SDR Kofi Asante | Sales Development Rep | white | front-office | junior | desktop + mobile | GH | B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive) | Kofi-as-SDR / Kofi-as-side-hustle-creator |
| 52 | Marketing Specialist Riya Sharma | Marketing Specialist | white | front-office | mid-level | desktop + mobile | IN | B2B_EMPLOYEE + B2C_CONSUMER | Riya-as-specialist / Riya-as-blogger |
| 53 | CS-IC Lin Chen | CS Specialist | white | front-office | mid-level | desktop + mobile | TW | B2B_EMPLOYEE + B2C_CONSUMER | Lin-as-CSM / Lin-as-mentor |
| 54 | Support Rep Nadia Hassani | Customer Support Rep | white | front-office | junior | desktop + headset | FR | B2B_EMPLOYEE + B2C_CONSUMER | Nadia-as-rep / Nadia-as-grad-student |
| 55 | Financial Analyst Wendy Lee | Financial Analyst | white | back-office | mid-level | desktop | US | B2B_EMPLOYEE + B2C_CONSUMER | Wendy-as-analyst / Wendy-as-CPA-candidate |
| 56 | Accountant Ravi Iyer | Accountant | white | back-office | mid-level | desktop | IN | B2B_EMPLOYEE + B2C_CONSUMER | Ravi-as-accountant / Ravi-as-freelance |
| 57 | Tax Analyst Ji-Sung Park | Tax Analyst | white | back-office | mid-level | desktop | KR | B2B_EMPLOYEE + B2C_CONSUMER | Ji-Sung-at-work / Ji-Sung-as-tax-helper-for-family |
| 58 | External Auditor Dimitri Volkov | External Auditor (Big-4) | white | middle-office | mid-level | desktop + mobile | DE | B2B_EXTERNAL_AUDITOR + B2C_CONSUMER | Dimitri-as-auditor-for-client-A / Dimitri-as-auditor-for-client-B |
| 59 | HR Specialist Aoife Murphy | HR Specialist | white | back-office | mid-level | desktop | IE | B2B_HR_ADMIN + B2C_CONSUMER | Aoife-as-specialist / Aoife-as-mentor (cross-role with Benefits Specialist) |
| 60 | Recruiter Marcus IV | Recruiter | white | back-office | junior | desktop + mobile | US | B2B_HR_ADMIN + B2C_CONSUMER | Marcus-IV-as-recruiter (note: not the same human as Marcus Chen / Marcus II / Marcus III) |
| 61 | Procurement Specialist Beata Kowalski | Procurement Specialist | white | back-office | mid-level | desktop | PL | B2B_EMPLOYEE + B2C_CONSUMER | Beata-at-work / Beata-as-PTA |
| 62 | Legal Operations Stephen Park | Legal Ops | white | back-office | mid-level | desktop | KR | B2B_EMPLOYEE + B2C_CONSUMER | Stephen-as-legal-ops / Stephen-as-paralegal-side |
| 63 | Compliance Analyst Yui Hayashi | Compliance Analyst | white | middle-office | mid-level | desktop | JP | B2B_EMPLOYEE + B2C_CONSUMER | Yui-as-analyst / Yui-as-volunteer-treasurer |
| 64 | DevOps Engineer Olukayode Adejumo | DevOps Engineer | white | back-office | mid-level | desktop + mobile | NG | B2B_EMPLOYEE + B2C_CONSUMER | Olukayode-as-engineer / Olukayode-as-OSS-maintainer |
| 65 | Security Analyst Anna Petrova | Security Analyst | white | middle-office | mid-level | desktop + air-gapped-mobile | RU | B2B_EMPLOYEE + B2C_CONSUMER | Anna-as-analyst / Anna-as-CTF-player |
| 66 | Data Analyst Felipe Andrade | Data Analyst | white | back-office | mid-level | desktop | BR | B2B_EMPLOYEE + B2C_CONSUMER | Felipe-as-analyst / Felipe-as-freelancer |
| 67 | Data Scientist Yu Chen | Data Scientist | white | back-office | senior | desktop + GPU | TW | B2B_EMPLOYEE + B2C_CONSUMER | Yu-as-DS / Yu-as-Kaggle-grandmaster |
| 68 | Product Designer Akihiro Sato | Product Designer | white | back-office | senior | desktop + tablet | JP | B2B_EMPLOYEE + B2C_CONSUMER | Akihiro-as-designer / Akihiro-as-art-instructor |
| 69 | UX Researcher Adaeze Nwosu | UX Researcher | white | back-office | mid-level | desktop + mobile | NG | B2B_EMPLOYEE + B2C_CONSUMER | Adaeze-as-researcher / Adaeze-as-grad-student |
| 70 | Project Manager Soo-Jin Park | Project Manager | white | back-office | mid-level | desktop + mobile | KR | B2B_EMPLOYEE + B2C_CONSUMER | Soo-Jin-as-PM / Soo-Jin-as-marathon-runner |
| 71 | Business Analyst Aditya Verma | Business Analyst | white | back-office | mid-level | desktop | IN | B2B_EMPLOYEE + B2C_CONSUMER | Aditya-as-BA / Aditya-as-finance-blogger |
| 72 | Communications Specialist Charlotte Dubois | Comms Specialist | white | front-office | mid-level | desktop + mobile | FR | B2B_EMPLOYEE + B2C_CONSUMER | Charlotte-as-comms / Charlotte-as-novelist |
| 73 | Training Specialist Mehmet Yilmaz | Training & Dev Specialist | white | back-office | mid-level | desktop + tablet | TR | B2B_EMPLOYEE + EDU_TEACHER (internal) + B2C_CONSUMER | Mehmet-as-trainer / Mehmet-as-Udemy-instructor |
| 74 | Office Coordinator Phoebe Lin | Office Coordinator | pink | back-office | junior | desktop + mobile | TW | B2B_EMPLOYEE + B2C_CONSUMER | Phoebe-as-coordinator / Phoebe-as-grad-student |
| 75 | Receptionist Daria Volkova | Receptionist | pink | front-office | junior | kiosk + mobile | UA | B2B_EMPLOYEE + B2C_CONSUMER | Daria-as-receptionist / Daria-as-art-student |
| 76 | Executive Assistant Olivia Reyes | EA | white | back-office | senior | desktop + mobile | US | B2B_EMPLOYEE + B2C_CONSUMER | Olivia-as-EA / Olivia-as-parent |
| 77 | Paralegal Tomáš Novák | Paralegal | white | back-office | mid-level | desktop | CZ | B2B_EMPLOYEE + B2C_CONSUMER | Tomáš-as-paralegal / Tomáš-as-cycling-club-treasurer |
| 78 | IR Manager Lev Kahn | Investor Relations Manager | white | front-office | senior | desktop + mobile | IL | B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER | Lev-as-IR / Lev-as-LP-of-fund |
| 79 | Corp Dev Senior Analyst Saanvi Mehta | Corp Dev Senior Analyst | white | back-office | senior | desktop | IN | B2B_EMPLOYEE + B2C_CONSUMER | Saanvi-as-analyst / Saanvi-as-business-school-applicant |

### §3.6 Cross-functional + external (17)

| # | Name | Role | Collar | Workspace | Skill | Device | Locale | audience_type | Cross-context |
|---:|---|---|---|---|---|---|---|---|---|
| 80 | Board Secretary Florence Akinsanya | Board Secretary | white | executive | senior | desktop + mobile | NG | B2B_BOARD_DIRECTOR + B2C_CONSUMER | Florence-as-secretary / Florence-as-mentor |
| 81 | Internal Comms Lead Ji-Ho Yoon | Internal Comms Lead | white | back-office | senior | desktop + mobile | KR | B2B_EMPLOYEE + B2C_CONSUMER | Ji-Ho-as-comms / Ji-Ho-as-novelist |
| 82 | Sustainability Officer Aiko Brown | Sustainability Officer | green | middle-office | senior | desktop + mobile | JP | B2B_EMPLOYEE + B2C_CONSUMER | Aiko-as-officer / Aiko-as-climate-activist |
| 83 | D&I Director Maya Okoroafor | D&I Director | white | back-office | senior | desktop + mobile | NG | B2B_HR_ADMIN + B2C_CONSUMER | Maya-O-as-director / Maya-O-as-board-advisor |
| 84 | Ombudsperson Felix Tan | Ombudsperson | white | middle-office | senior | desktop + mobile | SG | B2B_EMPLOYEE + B2C_CONSUMER | Felix-as-ombudsperson / Felix-as-mediator-side |
| 85 | Strategic Advisor Rita Almeida | Strategic Advisor | white | executive | principal | desktop + mobile | PT | B2B_EXTERNAL_COUNSEL + B2C_CONSUMER | Rita-as-advisor-to-A / Rita-as-advisor-to-B |
| 86 | Venture Partner Lucas Müller | Venture Partner | white | executive | principal | desktop + mobile | DE | B2B_INVESTOR_LP + B2C_CONSUMER | Lucas-as-VC / Lucas-as-LP-of-other-fund |
| 87 | Investor / LP Aanya Kapoor | Limited Partner | white | executive | senior | desktop + mobile | IN | B2B_INVESTOR_LP + B2C_CONSUMER | Aanya-as-LP-A / Aanya-as-LP-B / Aanya-as-board |
| 88 | Customer Champion Akemi Sato | Customer Champion | white | front-office | mid-level | desktop + mobile | JP | B2B_EMPLOYEE + B2C_CONSUMER | Akemi-as-champion / Akemi-as-customer-elsewhere |
| 89 | Channel Partner Tomas Pieter | Channel Partner | white | front-office | mid-level | desktop + mobile | NL | B2B_CHANNEL_PARTNER + B2C_CONSUMER | Tomas-as-partner / Tomas-as-employee-of-partner-co |
| 90 | External Auditor Hyo-Jin Lee | External Auditor (Big-4 KR) | white | middle-office | senior | desktop + mobile | KR | B2B_EXTERNAL_AUDITOR + B2C_CONSUMER | Hyo-Jin-as-auditor-A / Hyo-Jin-as-auditor-B |
| 91 | **External Regulator Inspector Sergei Petrov** | Regulator (KR FSS-equivalent) | white | middle-office | senior | desktop + mobile | UA seconded to KR | B2B_REGULATOR_EXTERNAL + GOV_INSPECTOR + B2C_CONSUMER | Sergei-as-regulator / Sergei-as-private-citizen |
| 92 | Banker (external) Hideki Watanabe | External Banker (relationship) | white | front-office | senior | desktop + mobile | JP | B2B_BANK_INTERNAL + B2C_CONSUMER | Hideki-as-banker-for-A / Hideki-as-banker-for-B |
| 93 | Consultant Adekunle Adebayo | Management Consultant (McKinsey-class) | white | back-office | senior | desktop + mobile | NG | B2B_EXTERNAL_COUNSEL + B2C_CONSUMER | Adekunle-as-consultant-for-A / Adekunle-as-consultant-for-B |
| 94 | PR Firm Beatriz Fernandez | External PR / Communications | white | front-office | senior | desktop + mobile | ES | B2B_CHANNEL_PARTNER + B2C_CONSUMER | Beatriz-as-PR-for-A / Beatriz-as-PR-for-B |
| 95 | Auditor IT-Specialist Jakub Nowak | IT Auditor (external) | white | middle-office | senior | desktop | PL | B2B_EXTERNAL_AUDITOR + B2C_CONSUMER | Jakub-as-IT-auditor / Jakub-as-CISSP-instructor |
| 96 | **Outside Counsel Wei-Yi Chen** | External Counsel (large firm) | white | back-office | principal | desktop + mobile | HK | B2B_EXTERNAL_COUNSEL + B2C_CONSUMER | Wei-Yi-as-counsel-for-A / Wei-Yi-as-counsel-for-B |

### §3.7 Office-bound non-knowledge-worker (7)

| # | Name | Role | Collar | Workspace | Skill | Device | Locale | audience_type | Cross-context |
|---:|---|---|---|---|---|---|---|---|---|
| 97 | Mailroom Hae-Won Kim | Mailroom Staff | blue | back-office | junior | mobile + scanner | KR | B2B_EMPLOYEE + B2C_CONSUMER | Hae-Won-as-mailroom / Hae-Won-as-art-student |
| 98 | Maintenance Tech Carlos Reyes II | Building Maintenance | gray | back-office | mid-level | mobile + handheld-rugged | US | B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT | Carlos-R-as-maint / Carlos-R-as-father |
| 99 | Security Guard Stefan Kovács | Security Guard | gray | front-office | junior | mobile + kiosk | HU | B2B_FIELD_WORKER + B2C_CONSUMER | Stefan-as-guard / Stefan-as-college-student |
| 100 | Cleaning Supervisor Tomáš Horák | Cleaning Supervisor | blue | back-office | mid-level | mobile + handheld-rugged | CZ | B2B_FIELD_WORKER + B2B_TENANT_ADMIN (cleaning-co tenant) | Tomáš-H-as-supervisor / Tomáš-H-as-cleaning-co-owner |
| 101 | Cafeteria Manager Soyeon Kim | Cafeteria Manager | pink + green | production | mid-level | mobile + kiosk | KR | B2B_EMPLOYEE + B2C_CONSUMER | Soyeon-as-manager / Soyeon-as-mother |
| 102 | Print Operator Diana Lazăr | Print Operator | gray | production | junior | mobile + kiosk | RO | B2B_EMPLOYEE + B2C_CONSUMER | Diana-L-as-operator / Diana-L-as-college-student |
| 103 | AV Coordinator Jordan Park | AV / Conferencing Coordinator | gray | back-office | mid-level | desktop + mobile + handheld-rugged | KR | B2B_EMPLOYEE + B2C_CONSUMER | Jordan-P-as-AV / Jordan-P-as-musician-side |

### §3.8 Banker internal (10)

| # | Name | Role | Collar | Workspace | Skill | Device | Locale | audience_type | Cross-context |
|---:|---|---|---|---|---|---|---|---|---|
| 104 | **Investment Banker Yuna Ahn** | Investment Banker (M&A) | white | front-office | senior | desktop + mobile (regulated) | KR | B2B_BANK_INTERNAL + B2C_CONSUMER | Yuna-as-IB / Yuna-as-MBA-applicant |
| 105 | Commercial Banker Frederik Hartmann | Commercial Banker | white | front-office | senior | desktop + mobile | DE | B2B_BANK_INTERNAL + B2C_CONSUMER | Frederik-as-banker-for-A / Frederik-as-banker-for-B |
| 106 | Retail Banker Sebastián Vega | Retail Banker (branch manager) | white | front-office | mid-level | desktop + kiosk | ES | B2B_BANK_INTERNAL + B2C_CONSUMER | Sebastián-as-branch-mgr / Sebastián-as-side-tutor |
| 107 | **Trader Mei Lin** | Trader (sell-side, equities) | white + gold | front-office | senior | desktop + air-gapped-mobile | HK | B2B_BANK_INTERNAL + B2C_CONSUMER | Mei-Lin-as-trader / Mei-Lin-as-marathon-runner |
| 108 | Wealth Manager Aamir Khan | Wealth Manager (private bank) | white | front-office | senior | desktop + mobile | AE | B2B_BANK_INTERNAL + B2C_CONSUMER | Aamir-as-WM / Aamir-as-LP-of-fund |
| 109 | Treasury Ops Sven Eriksson | Treasury Operations Analyst | white | middle-office | mid-level | desktop | SE | B2B_BANK_INTERNAL + B2C_CONSUMER | Sven-as-treasury-ops / Sven-as-CFA-candidate |
| 110 | Bank Ops Officer Olamide Adebanjo | Bank Operations Officer | white | back-office | mid-level | desktop | NG | B2B_BANK_INTERNAL + B2C_CONSUMER | Olamide-as-ops / Olamide-as-side-business-owner |
| 111 | Credit Analyst Hina Mori | Bank Credit Analyst | white | middle-office | mid-level | desktop | JP | B2B_BANK_INTERNAL + B2C_CONSUMER | Hina-M-as-credit-analyst / Hina-M-as-CFA-candidate |
| 112 | Bank Compliance Officer Rishi Bhattacharya | Bank Compliance Officer | white | middle-office | senior | desktop | IN | B2B_BANK_INTERNAL + B2C_CONSUMER | Rishi-as-bank-comp / Rishi-as-volunteer-treasurer |
| 113 | Bank Risk Manager Anders Pedersen | Bank Risk Manager | white | middle-office | senior | desktop | DK | B2B_BANK_INTERNAL + B2C_CONSUMER | Anders-as-risk-mgr / Anders-as-cycling-club-officer |

### §3.9 In-training tier (7) — apprentice / intern / co-op / resident / fellow

| # | Name | Role | Collar | Workspace | Skill | Device | Locale | audience_type | Cross-context |
|---:|---|---|---|---|---|---|---|---|---|
| 114 | **Summer Intern Priscilla Sharma** | Summer Intern (SWE) | white | back-office | in-training | desktop + mobile | IN | B2B_APPRENTICE_INTERN + B2C_CONSUMER + EDU_STUDENT | Priscilla-as-intern / Priscilla-as-undergrad |
| 115 | Co-op Student Liam Murphy | Engineering Co-op | white | back-office | in-training | desktop + mobile | IE | B2B_APPRENTICE_INTERN + B2C_CONSUMER + EDU_STUDENT | Liam-as-co-op / Liam-as-undergrad |
| 116 | Returning Intern Jia Han | Returning Intern (2nd-year) | white | back-office | in-training | desktop + mobile | CN | B2B_APPRENTICE_INTERN + B2C_CONSUMER + EDU_STUDENT | Jia-as-returning-intern / Jia-as-undergrad |
| 117 | Intern Manager Felicia Adamou | Intern Manager (oversees cohort) | white | back-office | mid-level | desktop + mobile | CI | B2B_EMPLOYEE + B2C_CONSUMER | Felicia-as-manager / Felicia-as-conference-speaker |
| 118 | Apprentice Jakob Bauer | Skilled-trades Apprentice (electrician) | blue | field | in-training | mobile + handheld-rugged | DE | B2B_APPRENTICE_INTERN + B2C_CONSUMER | Jakob-as-apprentice / Jakob-as-trade-school-student |
| 119 | **Medical Resident Dr. Sun-Mi Kim** | Medical Resident (PGY-3) | gold | clinical | in-training | clinical-PACS + mobile | KR | B2B_MEDICAL_RESIDENT + B2C_CONSUMER | Sun-Mi-as-resident / Sun-Mi-as-grad-school-applicant |
| 120 | Fellow Dr. Tobias Klein | Postdoctoral Fellow (research) | gold | back-office + clinical | in-training | desktop + clinical-PACS | DE | B2B_MEDICAL_RESIDENT + B2C_CONSUMER | Tobias-as-fellow / Tobias-as-postdoc-applicant |

### §3.10 Benefits HR sub-roster (5)

| # | Name | Role | Collar | Workspace | Skill | Device | Locale | audience_type | Cross-context |
|---:|---|---|---|---|---|---|---|---|---|
| 121 | **Benefits Specialist Aoife Murphy** | Benefits Specialist (cross-ref §3.5 row 59 — SAME human) | white | back-office | mid-level | desktop | IE | B2B_HR_ADMIN + B2C_CONSUMER | Aoife-as-HR-Specialist / Aoife-as-Benefits-Specialist (same human, sub-role expansion) |
| 122 | Total Rewards Manager Nilufer Demir | Total Rewards Manager | white | back-office | senior | desktop | TR | B2B_HR_ADMIN + B2C_CONSUMER | Nilufer-as-TR-mgr / Nilufer-as-CCP-credentialed |
| 123 | Leave Specialist Margarethe Reinhart | Leave-of-Absence Specialist | white | back-office | mid-level | desktop | DE | B2B_HR_ADMIN + B2C_CONSUMER | Margarethe-as-leave / Margarethe-as-mother |
| 124 | Wellness Program Manager Akira Sato | Wellness Program Manager | white + pink | back-office | mid-level | desktop + mobile | JP | B2B_HR_ADMIN + B2C_CONSUMER | Akira-S-as-wellness / Akira-S-as-yoga-instructor |
| 125 | Retirement Plan Admin Bryce Williams | Retirement Plan Administrator | white | back-office | senior | desktop | US | B2B_HR_ADMIN + B2C_CONSUMER | Bryce-as-plan-admin / Bryce-as-PTA-treasurer |

### §3.11 Corporate Relations + sub-roles (4)

| # | Name | Role | Collar | Workspace | Skill | Device | Locale | audience_type | Cross-context |
|---:|---|---|---|---|---|---|---|---|---|
| 126 | Corporate Relations Director Soo-Yeon Han | Corporate Relations Director | white | front-office | senior | desktop + mobile | KR | B2B_EMPLOYEE + B2C_CONSUMER | Soo-Yeon-as-director / Soo-Yeon-as-mentor-board |
| 127 | Public Affairs Director Carlos Mendez | Public Affairs Director | white | front-office | senior | desktop + mobile | MX | B2B_EMPLOYEE + B2C_CONSUMER | Carlos-M-as-PA / Carlos-M-as-pro-bono-advisor |
| 128 | PR Manager Helena Sato | PR Manager (internal) | white | front-office | mid-level | desktop + mobile | JP | B2B_EMPLOYEE + B2C_CONSUMER | Helena-S-as-PR / Helena-S-as-novelist |
| 129 | IR Specialist (unnamed) | Investor Relations Specialist | white | front-office | mid-level | desktop + mobile | US | B2B_EMPLOYEE + B2C_CONSUMER | IR-spec-A / IR-spec-as-CFA |

(Total persona count = 129; rounded as "~127" in the doctrine to reflect the open-set nature; the in-training tier and Benefits Specialist sub-role intentionally project off existing persons.)

---

## §4. Cross-Context Bridges (Same Human, Multiple Personas)

The bridge field is a first-class invariant. The following bridges are load-bearing across the master roster:

### §4.1 Yejin Park's five contexts

Yejin Park is one human. She projects into five oyatie personas:

1. **Yejin-as-nurse** (B2B_HEALTHCARE_PROVIDER, tenant = St. Mary's Hospital Seoul, Cedar permits over patient-record-readable, shift-schedule-writable). Surfaces: clinical-PACS + community (nurse-network channel) + messenger (work) + drive (work, locked).
2. **Yejin-as-parent** (B2C_FAMILY_PARENT, tenant = Yejin's personal-family-tenant, Cedar permits over family calendar + child's school comms). Surfaces: calendar + mail + community (PTA channel) + messenger (personal).
3. **Yejin-as-side-business-owner** (B2B_TENANT_ADMIN, tenant = Yejin's-handmade-soap-co tenant, Cedar permits over marketplace listings + payments). Surfaces: marketplace + payments + finops-portal + workflow-studio.
4. **Yejin-as-patient** (B2B_HEALTHCARE_PATIENT, tenant = her PCP's clinic). Surfaces: personal-health-tracker (per §11 of coverage matrix) + healthcare-integration + drive (medical-records).
5. **Yejin-as-consumer** (B2C_CONSUMER, tenant = personal). Surfaces: marketplace (buyer-side) + shorts + social + drive + messenger.

**Cedar permit graph:** Yejin's passkey identity holds memberships in {hospital-tenant, family-tenant, soap-business-tenant, clinic-tenant, personal-tenant}. Each membership has its own permit set. UX clearly indicates the active tenant. Cross-tenant queries are default-deny per ADR-0311.

### §4.2 Marcus Chen's three contexts

1. **Marcus-as-CEO** (B2B_CSUITE, tenant = Marcus's 5000-person multinational). Cedar permits: every executive-level action.
2. **Marcus-as-board-director-elsewhere** (B2B_BOARD_DIRECTOR, tenant = Other Co Board). Cedar permits: board-room-readable, board-resolution-votable. NO read-access to Other Co's day-to-day ops.
3. **Marcus-as-father** (B2C_FAMILY_PARENT, tenant = Marcus's personal-family-tenant). Surfaces: calendar + messenger (personal) + community (PTA + KOSA-parent-dashboard for minor children).

### §4.3 Diana Reyes's two contexts (per j126-j131 catalog)

1. **Diana-as-auditor** (INTERNAL_AUDITOR_3PAO, tenant = US GAO + 3PAO-shared-tenant). Surfaces: audit-chain + ops-dashboard-control-center + compliance.
2. **Diana-as-consumer** (B2C_CONSUMER + B2C_FAMILY_PARENT, tenant = Diana's personal-family-tenant). Cedar default-deny enforces non-overlap with auditor-tenant. Court-warrant is the ONLY path to pierce.

### §4.4 Chris Volkov's three contexts

1. **Chris-pre-layoff** (B2B_EMPLOYEE, tenant = former-employer). Cedar permits revoked at layoff.
2. **Chris-post-layoff** (B2C_JOB_SEEKER_ACTIVE, tenant = personal). Cedar permits unlock community (LinkedIn-mode + Handshake-mode + TeamBlind-mode), workflow-studio (job-search-pipeline), marketplace (side-income).
3. **Chris-as-family-provider** (B2C_FAMILY_PARENT, tenant = personal-family-tenant). Cedar permits over family-budget, kids' school calendar.

### §4.5 CEO Aoki Tanaka's three contexts

1. **Aoki-as-CEO-of-Company-A** (B2B_CSUITE).
2. **Aoki-as-independent-director-of-Company-B** (B2B_BOARD_DIRECTOR). No read access to Company B's day-to-day; only board-tier permits.
3. **Aoki-as-consumer** (B2C_CONSUMER) + Aoki-as-philanthropist (B2B_TENANT_ADMIN of his charitable foundation tenant).

### §4.6 CFO Helena Brandt's three contexts

1. **Helena-as-CFO** (B2B_CSUITE).
2. **Helena-as-board-of-charity** (B2B_BOARD_DIRECTOR for a 501(c)(3)-class tenant).
3. **Helena-as-consumer** + family-parent.

### §4.7 Carlos Martinez's two contexts

1. **Carlos-at-work** (B2B_FIELD_WORKER, tenant = warehouse-operator). Surfaces: handheld-rugged shift-clock-in app + community (workforce-channel) + workflow-engine (task-list).
2. **Carlos-as-father** (B2C_FAMILY_PARENT) + Carlos-as-consumer.

### §4.8 Officer Rodriguez's two contexts

1. **Rodriguez-on-patrol** (LAW_ENFORCEMENT, tenant = police-department). Vehicle-mount surfaces.
2. **Rodriguez-as-family** (B2C_FAMILY_PARENT, personal-tenant).

### §4.9 Father Lopez's two contexts

1. **Lopez-as-priest** (RELIGIOUS_LEADER, tenant = parish + diocese). Surfaces: community (parish-channel) + meet (mass) + drive (sermon archive).
2. **Lopez-as-counselor** (privileged-comms posture per pastoral-counsel-privilege; legally-defined privacy class).

### §4.10 Captain Chen's two contexts

1. **Chen-as-pilot** (B2B_EMPLOYEE, tenant = airline + crew-roster). Vehicle-mount + intercom + flight-management-system.
2. **Chen-as-father** (B2C_FAMILY_PARENT). Significantly distant from work-tenant; airline cannot read his personal Mail.

### §4.11 Dr. Tanaka's two contexts

1. **Dr.Tanaka-as-surgeon** (B2B_HEALTHCARE_PROVIDER, tenant = hospital + medical-board). Clinical-PACS surface.
2. **Dr.Tanaka-as-father** (B2C_FAMILY_PARENT).

### §4.12 Investment Banker Yuna Ahn's two contexts

1. **Yuna-as-IB** (B2B_BANK_INTERNAL, tenant = bank + per-deal-data-room). Heavily-regulated air-gapped-mobile.
2. **Yuna-as-MBA-applicant** (B2C_CONSUMER, personal-tenant). Cedar default-deny enforces non-overlap; insider-info handling per pack-bank.

### §4.13 Trader Mei Lin's two contexts

1. **Mei-Lin-as-trader** (B2B_BANK_INTERNAL, tenant = trading-floor). MNPI handling per pack-bank-trading.
2. **Mei-Lin-as-marathon-runner** (B2C_CONSUMER, personal-tenant).

### §4.14 Summer Intern Priscilla Sharma's two contexts

1. **Priscilla-as-intern** (B2B_APPRENTICE_INTERN). Lower-scope Cedar permits; supervisor co-sign on production-tier writes.
2. **Priscilla-as-undergrad** (EDU_STUDENT). Surfaces: notes + drive (academic) + community (campus-channel).

### §4.15 Medical Resident Dr. Sun-Mi Kim's two contexts

1. **Sun-Mi-as-resident** (B2B_MEDICAL_RESIDENT, tenant = teaching-hospital). Lower-scope clinical permits with attending co-sign.
2. **Sun-Mi-as-grad-school-applicant** (B2C_CONSUMER + EDU_STUDENT).

### §4.16 Benefits Specialist Aoife Murphy's two roles, one human

Aoife appears in §3.5 row 59 (HR Specialist) and §3.10 row 121 (Benefits Specialist). These are NOT two humans; they are one human's evolved sub-role over time. The roster represents the human as one identity with two persona-sub-projections.

### §4.17 External Regulator Sergei Petrov's two contexts

1. **Sergei-as-regulator** (B2B_REGULATOR_EXTERNAL + GOV_INSPECTOR, tenant = regulatory-agency).
2. **Sergei-as-private-citizen** (B2C_CONSUMER, personal-tenant). Cedar default-deny on regulated-tenant from his personal context.

### §4.18 Outside Counsel Wei-Yi Chen's contexts

1. **Wei-Yi-as-counsel-for-Client-A** (B2B_EXTERNAL_COUNSEL, tenant = Client-A's privileged-comms scope).
2. **Wei-Yi-as-counsel-for-Client-B** (B2B_EXTERNAL_COUNSEL, tenant = Client-B's privileged-comms scope).
3. **Wei-Yi-as-consumer** + family.

Strict scope isolation between Client-A and Client-B; attorney-client-privilege Cedar default-deny.

---

## §5. Per-Collar-Color Persona Summary

| Collar | Count | Sample personas | Pack overlay typical |
|---|---:|---|---|
| white-collar | 80 | Marcus / Aiyana / Diana / Priya / Sam / Chris / Aoki / Helena / Linda / Yuki / Yuna / Mei / Wei-Yi | SOC2 + GDPR + DPDP + sector-pack |
| blue-collar | 8 | Carlos / Sarah / Ahmad / Maria-cook / Hae-Won-mailroom / Tomáš-Horák-cleaning / Jakob-apprentice | OSHA + per-state-labor + apprentice-supervision |
| pink-collar | 12 | Yejin (nurse-sub) / Maria-Santos / Jordan-Lee / Patel / Coach-Park / Lopez / Soyeon-cafeteria / Aoife-W / Phoebe / Daria / Akira-S-Wellness | HIPAA (pink-clinical) + FERPA (pink-edu) + per-state-childcare |
| gold-collar | 6 | Dr.Tanaka / Captain-Chen / Sun-Mi / Tobias-Klein / Mei-Lin-trader / Yuna-IB | per-license-board (medical / FAA / FINRA) |
| gray-collar | 6 | Devon-HVAC / Rodriguez-police / Carlos-Reyes-II-maint / Stefan-Kovács-guard / Jordan-Park-AV / Diana-Lazăr-print | per-state-tech-cert + LE-cert |
| green-collar | 5 | Tomás-García-Jr-farmer / Olufemi-fisherman / Aiko-Brown-sustainability / Tomás-García (cook+farmer) / Yejin-soap-side-business | environmental + per-jurisdiction-agri |

---

## §6. Per-Workspace Persona Summary

| Workspace | Count | Sample personas | µservice center-of-gravity |
|---|---:|---|---|
| front-office | 32 | Salesmen / Receptionists / Customer-Champions / Bankers (retail) / PR / Corporate Relations / Sales / SDR / CSM | crm + community + marketplace |
| middle-office | 14 | Compliance / Internal-Audit / Risk / Treasury-Ops / Ombudsperson / External-Auditor / Regulator | compliance + audit-chain + observability + ops-dashboard |
| back-office | 50 | HR / Finance / Legal / Procurement / DevOps / IT / Accounting / Tax / Project-Management / Engineering-Manager | identity + tenancy + payments + workflow-engine |
| field | 16 | Forklift / Delivery / Construction / Field-Service / Farmer / Fisherman / Pilot / Officer / Apprentice-electrician | workflow-engine + ontology + connect + messenger |
| clinical / care | 6 | Yejin-nurse / Dr.Tanaka / Sun-Mi / Tobias-Klein / Wellness-Manager / Akira-S-wellness | healthcare-integration + drive (medical-records) + workflow-engine |
| executive | 13 | CEO / CFO / CTO / CHRO / CISO / CSO / CMO / CCO / COO / Board / Strategic-Advisor / Venture-Partner | ops-dashboard + finops-portal + intelligence |
| production | 6 | Maria-cook / Tomás-García-cook / Soyeon-cafeteria-mgr / Diana-Lazăr-print / Jakob-apprentice (intermediate) | workflow-engine + ontology + ops-dashboard |

---

## §7. Per-Skill-Tier Persona Summary

| Skill-tier | Count | Cedar permit posture | Sample personas |
|---|---:|---|---|
| in-training | 7 | Lower-scope; mandatory co-sign on high-stakes | Priscilla / Liam / Jia / Jakob / Sun-Mi / Tobias / Marcus IV (junior recruiter — borderline) |
| junior | 15 | Standard permits; coaching layer | Jordan / Kofi / Hae-Won / Stefan / Diana-Lazăr / Daria / Marcus-IV / Nadia |
| mid-level | 38 | Full within scope | Carlos / Sarah / Maria-Santos / Lopez (cross-tier) / Hugo / Maya / Felipe / Wendy / Ravi / Ji-Sung / Pavel-cross / Jamal / Hina-S / Aoife / Tomáš-Novák / Akihiro-cross / Riya / many ICs |
| senior / staff | 53 | Approval over cross-team / cross-tenant | Aiyana / Tomás-García / Diana / Priya / Sam / Anya / Yuna / Mei-Lin / Hideki / Frederik / Linda-cross / Dr.Tanaka / Captain-Chen / Ms.Patel / Helena / Aoki / Wei-Yi / Hyo-Jin / Sergei / Sofia / Olivia |
| principal / distinguished | 4 | Architectural-decision | O'Reilly (board) / Rita-Almeida / Lucas-Müller / Wei-Yi |
| executive | 12 | Board-tier permits | CEO / CFO / COO / CTO / CHRO / CMO / CCO / CSO / CISO / Board / Strategic-Advisor / Venture-Partner |

---

## §8. Per-Locale Persona Summary

| Locale | Count | Pack overlay roster | Sample personas |
|---|---:|---|---|
| KR | 18 | KR-CSAP + KR-Privacy + KR-Labor + KR-FSS | Yejin / Marcus / Yuki-Park / Yuna-Ahn / Jordan-Park / Stephen-Park / Soyeon-Kim / Hae-Won-Kim / Soo-Jin / Ji-Sung / Coach-Park / Hyo-Jin / Soo-Yeon-Han / Sergei-seconded |
| US | 25 | SOC2 + state-by-state + HIPAA | Diana / Chris / Carlos-M / Sarah / Ahmad / Maria-S / Devon / Officer-Rodriguez / Diego / Linda-Foster / Mira-Goldberg / Wendy / CTO-Diego / Marcus-IV / Olivia / Carlos-Reyes-II / Bryce / Carlos-Mendez / IR-Spec |
| EU + UK | 30 | GDPR + DSA + EU-AI-Act + DORA + UK-AADC + per-state | Anya / Helena-Brandt / Frederik / Aoife / Jamie / Liam / Tomáš-Novák / Beata / Jakob / Margarethe / Lucas / Charlotte / Anders / Patrick / Nadia / Sven / Florence / Stefan / Daria / Pavel / Diana-Lazăr / Tomáš-Horák / Sebastián / Beatriz / Rita-Almeida / Adekunle / Jakub / Captain-Chen-SG / Felix-Ng-SG |
| JP | 14 | APPI + JP-Labor + JP-FSA | Hiroshi / Dr.Tanaka / Aoki / Akira-W / Akihiro / Hina-Suzuki / Hina-Mori / Hideki / Aiko-Brown / Helena-Sato / Yui-Hayashi / Akira-S-wellness / Akemi |
| IN | 10 | DPDP-2023 + RBI | Aiyana / Priya / Ravi / Aditya / Naveen / Aanya-Kapoor / Riya / Anika / Priya-Ramanathan / Saanvi / Priscilla |
| BR + LATAM | 8 | LGPD + BR-Labor + MX | Tomás-García / Tomás-García-Jr / Sofia / Felipe / Carlos-Mendez-MX |
| NG + Africa | 8 | NDPR + per-state-Africa | Sam / Olu / Olukayode / Adaeze / Adekunle / Olamide / Maya-O / Tunde / Florence / Olufemi |
| Other (CN/HK/SG/IL/AE/TR/CZ/PL/HU/RO/UA) | 16 | Per-jurisdiction | Jia / Wei-Liu / Mei-Lin / Wei-Yi / Felix-Ng / Lev-Kahn / Aamir / Nilufer / Tomáš-Novák / Beata / Stefan-Kovács / Daria / Diana-Lazăr / Pavel-Korsak / Tomáš-Horák / Mehmet |

---

## §9. Cedar Permit-Class Patterns

The roster aggregates into the following Cedar permit-class patterns. Each pattern is the canonical baseline for the audience-type group; each persona may overlay sub-class permits.

| Permit class | Audience types | Base read | Base write | Base decide |
|---|---|---|---|---|
| B2C_BASELINE | B2C_CONSUMER, B2C_FAMILY_PARENT | own-data only | own-data only | none cross-tenant |
| B2C_MINOR_TIERED | B2C_MINOR_UNDER_13 (COPPA-block) / B2C_MINOR_14_17 (KOSA-tier) | parent-consent-gated | parent-consent-gated | parent-co-decision required |
| B2C_JOB_SEEKER_ACTIVE | B2C_JOB_SEEKER_ACTIVE | community (LinkedIn/Handshake/Blind), marketplace (sell-side), workflow-studio (job-search-pipeline) | own resume + applications | none cross-tenant |
| B2B_EMPLOYEE | B2B_EMPLOYEE | tenant-owned work-surfaces | tenant-owned work-surfaces (own scope) | own work-output only |
| B2B_HR_ADMIN | B2B_HR_ADMIN | tenant-owned employee work-surfaces (per labor law) | hiring + onboarding + benefits + offboarding | hiring + termination decisions |
| B2B_INTERNAL_AUDIT | B2B_INTERNAL_AUDIT | tenant-owned all work-surfaces + audit-chain | scope: audit findings | escalation to legal |
| B2B_CSUITE | B2B_CSUITE | tenant-wide aggregated | strategic decisions | board-tier decisions |
| B2B_BOARD_DIRECTOR | B2B_BOARD_DIRECTOR | board-room + board-resolution-history (per-board scope, NOT day-to-day ops of that company) | board-resolution votes | board-resolution decisions |
| B2B_APPRENTICE_INTERN | B2B_APPRENTICE_INTERN, B2B_MEDICAL_RESIDENT | scope-limited | scope-limited + co-sign required on high-stakes | none independent |
| B2B_FIELD_WORKER | B2B_FIELD_WORKER | shift-schedule + task-list | task-completion | task-completion only |
| B2B_BANK_INTERNAL | B2B_BANK_INTERNAL | per-deal data-room (need-to-know) | per-deal data-room (need-to-know) | trade execution + position book |
| B2B_HEALTHCARE_PROVIDER | B2B_HEALTHCARE_PROVIDER | patient-record (per-treatment-relationship) | patient-record (per-treatment-relationship) | treatment decisions (Cedar attest co-sign) |
| INTERNAL_AUDITOR_3PAO + B2B_REGULATOR_EXTERNAL + GOV_INSPECTOR | INTERNAL_AUDITOR_3PAO + B2B_REGULATOR_EXTERNAL + GOV_INSPECTOR | per-warrant or per-lawful-authority scope | findings only (not the audited system) | regulatory decisions |
| B2B_EXTERNAL_COUNSEL | B2B_EXTERNAL_COUNSEL | privileged-comms within engaged scope | scope: legal-work-product | scope: counsel guidance |
| LAW_ENFORCEMENT | LAW_ENFORCEMENT | LE-shared (CAD + CJIS-protected) | incident-report | arrest + traffic-stop decisions |
| EDU_TEACHER | EDU_TEACHER | student-records (FERPA-scoped) | grade + attendance | grade + discipline decisions |
| RELIGIOUS_LEADER | RELIGIOUS_LEADER | parish-records | sermon archive | counsel guidance (privileged) |

---

## §10. Per-Pack Overlay Roster

Each persona overlays one or more compliance packs. The most-common overlays:

| Pack | Activated by | Sample persona |
|---|---|---|
| **HIPAA-2024** | B2B_HEALTHCARE_PROVIDER, B2B_HEALTHCARE_PATIENT | Yejin-as-nurse, Dr.Tanaka, Sun-Mi |
| **GDPR + EU-AI-Act + DORA** | locale=EU | Anya, Helena, Aoife, Jakob, Lucas |
| **DPDP-2023 (India)** | locale=IN | Aiyana, Priya, Ravi, Naveen |
| **KR-CSAP + KR-Privacy** | locale=KR | Yejin, Marcus, Yuna, Jordan-Park |
| **LGPD (Brazil)** | locale=BR | Tomás-García, Tomás-García-Jr, Sofia |
| **APPI (Japan)** | locale=JP | Hiroshi, Dr.Tanaka, Aoki, Akira-W |
| **COPPA + KOSA** | B2C_MINOR_UNDER_13 / B2C_MINOR_14_17 | Jordan-Lee (17, KOSA), Marcus's minor children |
| **FERPA** | EDU_TEACHER + EDU_STUDENT + EDU_PARENT | Ms.Patel, Priscilla, Jia, Liam |
| **PCI-DSS-L1-v4** | B2B_TENANT_ADMIN (payments-handling) | Tomás-García (restaurant), Carlos-Martinez-side |
| **SOX-404** | B2B_CSUITE (public-co) | Helena-Brandt (CFO of public-co), Sam (audit of public-co) |
| **FedRAMP-High + StateRAMP** | GOV_INSPECTOR + INTERNAL_AUDITOR_3PAO | Diana-Reyes, Sergei-Petrov |
| **FINRA / SEC / Reg-NMS** | B2B_BANK_INTERNAL (trading + IB) | Yuna-Ahn, Mei-Lin, Hideki |
| **CFA-Privacy + insider-info** | B2B_BANK_INTERNAL (research + IB) | Yuna-Ahn, Hina-Mori, Rishi-Bhattacharya |
| **Attorney-Client-Privilege** | B2B_EXTERNAL_COUNSEL | Wei-Yi-Chen, Anika-Mehta-cross |
| **Pastoral-Counsel-Privilege** | RELIGIOUS_LEADER | Father-Lopez |
| **OSHA + per-state-labor** | blue-collar, gray-collar | Carlos-Martinez, Devon, Officer-Rodriguez |
| **NIST-SP-800-171 + CMMC** | aerospace/defense tenant employees | (cross-tenant; activated when persona's tenant requires) |

---

## §11. Critical-Path Edge Cases (per documentation-rigor.md §3.2.5)

For each persona, the following critical-path edge cases (rows from §3.2.5) MUST be covered in the per-persona dossier:

- Row 1 (default-deny on cold-start): every persona's Cedar permits derive from explicit permit grant, never inheritance
- Row 8 (cross-tenant data exfiltration): every persona with cross-tenant membership MUST declare what data leakage is blocked
- Row 9 (revocation lag): when a persona's role changes, Cedar permit revocation MUST be effective within budget
- Row 14 (minor protection): personas linked to minors MUST declare the KOSA / COPPA / parental-consent flow
- Row 18 (BYOK + provider-credential-mode): personas in regulated industries MUST declare their provider-BYOK posture
- Row 24 (audit-chain seal): every persona with write authority MUST emit audit events linked to their identity + tenant
- Row 27 (Cedar fragment publish soak ≥60s): when a persona is granted a new permit, the soak window MUST be respected
- Row 30 (cell-tier degradation): persona behavior MUST be documented for Tier-0 outage scenarios

Each per-persona dossier §J enumerates the specific edge-case row coverage.

---

## §12. References

- documentation-rigor.md §1.1 (hyperscaler-grade sub-test) + §1.2 (engineering-rigor dimensions) + §3.2.1 (28-row ADR-adherence matrix) + §3.2.5 (critical-path edge cases)
- enterprise-software-coverage-matrix-2026-05-21.md §13 (centers-of-gravity µservices + persona spectrum)
- CATALOG-j126-j150-ecosystem.md (existing persona archetypes Diana/Priya/Sam/Chris)
- microservices/community/PRD.md (post-anonymous-fold; 56 µservice count per persona-roster brief)
- ADR-0244 audience_type
- ADR-0299 account-recovery (passkey-bound identity)
- ADR-0311 dual-tenant boundary (work-vs-personal)
- ADR-0313 conglomerate-tenant hierarchy
- ADR-0317 role-projection (in-flight; authoritative)
- ADR-0318 collar-color universality (in-flight; authoritative)
- ADR-0319 front/middle/back-office (in-flight; authoritative)
- ADR-0320 apprentice/intern/resident/fellow tier (in-flight; authoritative)

## §13. Persona Dossier Coverage Annex

This annex completes the brief-scoped delivery by binding the top-30 dossier files back to the master graph. The brief's active µservice count is **56** for these persona artifacts. All paths are under `docs/personas/`; no ADR, standard, PRD, or µservice source file is modified by this delivery.

### §13.1 Top-30 dossier index

| Priority | Persona dossier | Lines | Collar-color | Workspace | audience_type | Cross-context bridge |
|---:|---|---:|---|---|---|---|
| 01 | [Yejin Park](yejin-park.md) | 403 | pink + green | clinical + field | B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT | Yejin-as-nurse / Yejin-as-parent / Yejin-as-side-business-owner / Yejin-as-patient / Yejin-as-consumer |
| 02 | [Marcus Chen](marcus-chen.md) | 399 | white + gold | executive + field | B2B_CSUITE + B2C_CONSUMER + B2C_FAMILY_PARENT | Marcus-as-CEO / Marcus-as-husband / Marcus-as-father |
| 03 | [Aiyana Singh](aiyana-singh.md) | 399 | white | back-office | B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT | Aiyana-at-work / Aiyana-as-blogger / Aiyana-as-parent |
| 04 | [Tomás García](tomas-garcia.md) | 399 | white + green | executive + production | B2B_TENANT_ADMIN + B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT | Tomás-as-owner / Tomás-as-cook / Tomás-as-father |
| 05 | [Hiroshi Tanaka](hiroshi-tanaka.md) | 397 | white retired | field | B2C_CONSUMER + B2C_FAMILY_PARENT | Hiroshi-as-grandfather / Hiroshi-as-photographer / Hiroshi-as-patient |
| 06 | [Anya Mironova](anya-mironova.md) | 398 | white | field | B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER | Anya-as-journalist / Anya-as-parent / Anya-as-activist |
| 07 | [Diana Reyes](diana-reyes.md) | 395 | white | middle-office + field | INTERNAL_AUDITOR_3PAO + B2C_CONSUMER + B2C_FAMILY_PARENT | Diana-as-auditor / Diana-as-consumer |
| 08 | [Priya Krishnan](priya-krishnan.md) | 398 | white | back-office | B2B_HR_ADMIN + B2C_CONSUMER | Priya-at-work / Priya-as-consumer |
| 09 | [Sam Okafor](sam-okafor.md) | 397 | white | middle-office | B2B_INTERNAL_AUDIT + B2C_CONSUMER | Sam-at-work / Sam-as-consumer |
| 10 | [Chris Volkov](chris-volkov.md) | 398 | white | back-office | B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT | Chris-pre-layoff / Chris-post-layoff / Chris-as-family-provider |
| 11 | [CEO Aoki Tanaka](ceo-aoki-tanaka.md) | 397 | white | executive | B2B_CSUITE + B2C_CONSUMER | Aoki-as-CEO / Aoki-as-board-director-elsewhere / Aoki-as-parent |
| 12 | [CFO Helena Brandt](cfo-helena-brandt.md) | 396 | white | executive | B2B_CSUITE + B2C_CONSUMER | Helena-as-CFO / Helena-as-charity-board-director |
| 13 | [CISO Yuki Park](ciso-yuki-park.md) | 396 | white | executive | B2B_CSUITE + SECURITY_RESEARCHER + B2C_CONSUMER | Yuki-as-CISO / Yuki-as-incident-response-volunteer |
| 14 | [CHRO Linda Foster](chro-linda-foster.md) | 396 | white | executive | B2B_CSUITE + B2B_HR_ADMIN + B2C_CONSUMER | Linda-as-CHRO / Linda-as-mentor-board |
| 15 | [Carlos Martinez](carlos-martinez-forklift.md) | 396 | blue | field | B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT | Carlos-at-work / Carlos-as-father |
| 16 | [Sarah Kim](sarah-kim-delivery.md) | 396 | blue | field | B2B_FIELD_WORKER + B2C_CONSUMER + B2B_TENANT_ADMIN | Sarah-as-driver / Sarah-as-side-hustler |
| 17 | [Captain Chen](captain-chen-pilot.md) | 396 | gold | field | B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT | Chen-as-pilot / Chen-as-father |
| 18 | [Dr. Tanaka](dr-tanaka-surgeon.md) | 396 | gold | clinical | B2B_HEALTHCARE_PROVIDER + B2C_CONSUMER + B2C_FAMILY_PARENT | Dr.Tanaka-as-surgeon / Dr.Tanaka-as-father |
| 19 | [Officer Rodriguez](officer-rodriguez-police.md) | 396 | gray | field | LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT | Rodriguez-on-patrol / Rodriguez-as-family |
| 20 | [Ms. Patel](ms-patel-teacher.md) | 397 | pink | front-office | EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT | Patel-as-teacher / Patel-as-mother / Patel-as-student-mentor |
| 21 | [Father Lopez](father-lopez-priest.md) | 397 | pink | front-office | RELIGIOUS_LEADER + B2C_CONSUMER | Lopez-as-priest / Lopez-as-counselor / Lopez-as-private-citizen |
| 22 | [Tomás García Jr.](tomas-garcia-jr-farmer.md) | 397 | green | field | B2B_TENANT_ADMIN + B2C_CONSUMER + B2C_FAMILY_PARENT | Tomás-Jr-as-farmer / Tomás-Jr-as-cooperative-board / son-of-Tomás-García |
| 23 | [Investment Banker Yuna Ahn](investment-banker-yuna-ahn.md) | 396 | white | front-office | B2B_BANK_INTERNAL + B2C_CONSUMER | Yuna-as-IB / Yuna-as-MBA-applicant |
| 24 | [Trader Mei Lin](trader-mei-lin.md) | 396 | white + gold | front-office | B2B_BANK_INTERNAL + B2C_CONSUMER | Mei-Lin-as-trader / Mei-Lin-as-marathon-runner |
| 25 | [Summer Intern Priscilla Sharma](summer-intern-priscilla-sharma.md) | 396 | white | back-office | B2B_APPRENTICE_INTERN + B2C_CONSUMER + EDU_STUDENT | Priscilla-as-intern / Priscilla-as-undergrad |
| 26 | [Medical Resident Dr. Sun-Mi Kim](medical-resident-dr-sun-mi-kim.md) | 396 | gold | clinical | B2B_MEDICAL_RESIDENT + B2C_CONSUMER | Sun-Mi-as-resident / Sun-Mi-as-grad-school-applicant |
| 27 | [Benefits Specialist Aoife Murphy](benefits-specialist-aoife-murphy.md) | 396 | white | back-office | B2B_HR_ADMIN + B2C_CONSUMER | Aoife-as-HR-Specialist / Aoife-as-Benefits-Specialist |
| 28 | [Regulator Inspector Sergei Petrov](regulator-inspector-sergei-petrov.md) | 395 | white | middle-office | B2B_REGULATOR_EXTERNAL + GOV_INSPECTOR + B2C_CONSUMER | Sergei-as-regulator / Sergei-as-private-citizen |
| 29 | [Board director Patrick O'Reilly](board-director-patrick-oreilly.md) | 397 | white | executive | B2B_BOARD_DIRECTOR + B2C_CONSUMER | O'Reilly-on-Board-A / O'Reilly-on-Board-B / O'Reilly-on-Board-C |
| 30 | [Outside Counsel Wei-Yi Chen](outside-counsel-wei-yi-chen.md) | 396 | white | back-office | B2B_EXTERNAL_COUNSEL + B2C_CONSUMER | Wei-Yi-as-counsel-for-A / Wei-Yi-as-counsel-for-B |

### §13.2 Acceptance checklist

- Master roster line floor: ≥800 lines.
- Top-30 dossier floor: each dossier ≥300 lines.
- Dossier sections: §A Archetype through §K Journey range, plus references and buildability ledger.
- Doctrine: one human, same passkey identity, multiple tenant memberships.
- Community path: `microservices/community/PRD.md`; the deleted anonymous path is not used.
- ADR coverage: ADR-0244, ADR-0292, ADR-0299, ADR-0311, ADR-0313, ADR-0314, ADR-0315, ADR-0316, ADR-0317, ADR-0318, ADR-0319, ADR-0320.
- In-flight authority: ADR-0316, ADR-0319, and ADR-0320 are cited as authoritative for this roster.
- Layout convention: flat per-µservice layout per ADR-0131.
- Layer convention: 13-layer canonical enum per ADR-0105.
- Critical paths: documentation-rigor.md §3.2.5 rows 1-30 are explicitly considered in every top-30 dossier.
- Pack overlays: every top-30 dossier lists jurisdiction, tenant, role, or vertical overlays.
- Cedar posture: every top-30 dossier has permit sketch, default-deny rule, and revocation/soak handling.
- Journey range: every top-30 dossier spans j001-j150 and future j151+ migration lanes.
- Microservice count: this persona brief uses 56 as the active count.
- Stop condition: all line floors clear and no forbidden source files are touched.

## §14. Cross-Context Bridge Ledger for the Top 30

### §14.01 Yejin Park
- Slug: `yejin-park`.
- Same-human projections: Yejin-as-nurse / Yejin-as-parent / Yejin-as-side-business-owner / Yejin-as-patient / Yejin-as-consumer.
- Audience types: `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`.
- Primary services: identity, tenancy, policy-engine, audit-chain, workflow-engine, community, messenger, mail.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.02 Marcus Chen
- Slug: `marcus-chen`.
- Same-human projections: Marcus-as-CEO / Marcus-as-husband / Marcus-as-father.
- Audience types: `B2B_CSUITE + B2C_CONSUMER + B2C_FAMILY_PARENT`.
- Primary services: identity, tenancy, workflow-engine, ops-dashboard-control-center, governance, compliance, messenger, mail.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.03 Aiyana Singh
- Slug: `aiyana-singh`.
- Same-human projections: Aiyana-at-work / Aiyana-as-blogger / Aiyana-as-parent.
- Audience types: `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`.
- Primary services: identity, developer-sdk, foundry, intelligence, ontology, workflow-studio, community, shorts.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.04 Tomás García
- Slug: `tomas-garcia`.
- Same-human projections: Tomás-as-owner / Tomás-as-cook / Tomás-as-father.
- Audience types: `B2B_TENANT_ADMIN + B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`.
- Primary services: identity, payments, marketplace, finops-portal, workflow-engine, community, mail, messenger.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.05 Hiroshi Tanaka
- Slug: `hiroshi-tanaka`.
- Same-human projections: Hiroshi-as-grandfather / Hiroshi-as-photographer / Hiroshi-as-patient.
- Audience types: `B2C_CONSUMER + B2C_FAMILY_PARENT`.
- Primary services: identity, messenger, mail, calendar, community, drive, payments, personal-health-tracker.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.06 Anya Mironova
- Slug: `anya-mironova`.
- Same-human projections: Anya-as-journalist / Anya-as-parent / Anya-as-activist.
- Audience types: `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`.
- Primary services: identity, messenger, mail, drive, community, compliance, audit-chain, policy-engine.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.07 Diana Reyes
- Slug: `diana-reyes`.
- Same-human projections: Diana-as-auditor / Diana-as-consumer.
- Audience types: `INTERNAL_AUDITOR_3PAO + B2C_CONSUMER + B2C_FAMILY_PARENT`.
- Primary services: identity, tenancy, audit-chain, compliance, ops-dashboard-control-center, observability, workflow-studio, payments.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.08 Priya Krishnan
- Slug: `priya-krishnan`.
- Same-human projections: Priya-at-work / Priya-as-consumer.
- Audience types: `B2B_HR_ADMIN + B2C_CONSUMER`.
- Primary services: identity, workflow-engine, forms, drive, mail, messenger, calendar, workplace-integration.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.09 Sam Okafor
- Slug: `sam-okafor`.
- Same-human projections: Sam-at-work / Sam-as-consumer.
- Audience types: `B2B_INTERNAL_AUDIT + B2C_CONSUMER`.
- Primary services: identity, audit-chain, compliance, governance, workflow-engine, payments, mail, messenger.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.10 Chris Volkov
- Slug: `chris-volkov`.
- Same-human projections: Chris-pre-layoff / Chris-post-layoff / Chris-as-family-provider.
- Audience types: `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`.
- Primary services: identity, tenancy, workflow-studio, community, mail, messenger, drive, calendar.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.11 CEO Aoki Tanaka
- Slug: `ceo-aoki-tanaka`.
- Same-human projections: Aoki-as-CEO / Aoki-as-board-director-elsewhere / Aoki-as-parent.
- Audience types: `B2B_CSUITE + B2C_CONSUMER`.
- Primary services: identity, governance, ops-dashboard-control-center, workflow-engine, financial-planning, erp-analytics, compliance, messenger.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.12 CFO Helena Brandt
- Slug: `cfo-helena-brandt`.
- Same-human projections: Helena-as-CFO / Helena-as-charity-board-director.
- Audience types: `B2B_CSUITE + B2C_CONSUMER`.
- Primary services: identity, payments, finops-portal, financial-planning, erp-finance, compliance, audit-chain, workflow-engine.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.13 CISO Yuki Park
- Slug: `ciso-yuki-park`.
- Same-human projections: Yuki-as-CISO / Yuki-as-incident-response-volunteer.
- Audience types: `B2B_CSUITE + SECURITY_RESEARCHER + B2C_CONSUMER`.
- Primary services: identity, policy-engine, audit-chain, observability, ops-dashboard-control-center, cloud-secrets, api-gateway, incident-mgmt.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.14 CHRO Linda Foster
- Slug: `chro-linda-foster`.
- Same-human projections: Linda-as-CHRO / Linda-as-mentor-board.
- Audience types: `B2B_CSUITE + B2B_HR_ADMIN + B2C_CONSUMER`.
- Primary services: identity, workflow-engine, workplace-integration, performance-mgmt, learning-mgmt, forms, mail, community.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.15 Carlos Martinez
- Slug: `carlos-martinez-forklift`.
- Same-human projections: Carlos-at-work / Carlos-as-father.
- Audience types: `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`.
- Primary services: identity, workplace-integration, workflow-engine, messenger, calendar, payments, community, incident-mgmt.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.16 Sarah Kim
- Slug: `sarah-kim-delivery`.
- Same-human projections: Sarah-as-driver / Sarah-as-side-hustler.
- Audience types: `B2B_FIELD_WORKER + B2C_CONSUMER + B2B_TENANT_ADMIN`.
- Primary services: identity, workflow-engine, calendar, payments, marketplace, finops-portal, messenger, community.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.17 Captain Chen
- Slug: `captain-chen-pilot`.
- Same-human projections: Chen-as-pilot / Chen-as-father.
- Audience types: `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`.
- Primary services: identity, calendar, workflow-engine, messenger, mail, incident-mgmt, compliance, personal-health-tracker.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.18 Dr. Tanaka
- Slug: `dr-tanaka-surgeon`.
- Same-human projections: Dr.Tanaka-as-surgeon / Dr.Tanaka-as-father.
- Audience types: `B2B_HEALTHCARE_PROVIDER + B2C_CONSUMER + B2C_FAMILY_PARENT`.
- Primary services: identity, workflow-engine, personal-health-tracker, calendar, messenger, mail, audit-chain, compliance.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.19 Officer Rodriguez
- Slug: `officer-rodriguez-police`.
- Same-human projections: Rodriguez-on-patrol / Rodriguez-as-family.
- Audience types: `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`.
- Primary services: identity, workflow-engine, audit-chain, messenger, incident-mgmt, compliance, drive, notifications.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.20 Ms. Patel
- Slug: `ms-patel-teacher`.
- Same-human projections: Patel-as-teacher / Patel-as-mother / Patel-as-student-mentor.
- Audience types: `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT`.
- Primary services: identity, community, forms, mail, calendar, drive, learning-mgmt, messenger.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.21 Father Lopez
- Slug: `father-lopez-priest`.
- Same-human projections: Lopez-as-priest / Lopez-as-counselor / Lopez-as-private-citizen.
- Audience types: `RELIGIOUS_LEADER + B2C_CONSUMER`.
- Primary services: identity, community, messenger, mail, calendar, notes, compliance, workflow-engine.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.22 Tomás García Jr.
- Slug: `tomas-garcia-jr-farmer`.
- Same-human projections: Tomás-Jr-as-farmer / Tomás-Jr-as-cooperative-board / son-of-Tomás-García.
- Audience types: `B2B_TENANT_ADMIN + B2C_CONSUMER + B2C_FAMILY_PARENT`.
- Primary services: identity, marketplace, payments, finops-portal, workflow-engine, community, erp-inventory, erp-procurement.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.23 Investment Banker Yuna Ahn
- Slug: `investment-banker-yuna-ahn`.
- Same-human projections: Yuna-as-IB / Yuna-as-MBA-applicant.
- Audience types: `B2B_BANK_INTERNAL + B2C_CONSUMER`.
- Primary services: identity, mail, messenger, drive, workflow-engine, payments, audit-chain, compliance.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.24 Trader Mei Lin
- Slug: `trader-mei-lin`.
- Same-human projections: Mei-Lin-as-trader / Mei-Lin-as-marathon-runner.
- Audience types: `B2B_BANK_INTERNAL + B2C_CONSUMER`.
- Primary services: identity, payments, workflow-engine, observability, audit-chain, compliance, data-warehouse, mail.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.25 Summer Intern Priscilla Sharma
- Slug: `summer-intern-priscilla-sharma`.
- Same-human projections: Priscilla-as-intern / Priscilla-as-undergrad.
- Audience types: `B2B_APPRENTICE_INTERN + B2C_CONSUMER + EDU_STUDENT`.
- Primary services: identity, developer-sdk, foundry, workflow-engine, mail, calendar, learning-mgmt, community.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.26 Medical Resident Dr. Sun-Mi Kim
- Slug: `medical-resident-dr-sun-mi-kim`.
- Same-human projections: Sun-Mi-as-resident / Sun-Mi-as-grad-school-applicant.
- Audience types: `B2B_MEDICAL_RESIDENT + B2C_CONSUMER`.
- Primary services: identity, workflow-engine, personal-health-tracker, audit-chain, compliance, calendar, messenger, mail.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.27 Benefits Specialist Aoife Murphy
- Slug: `benefits-specialist-aoife-murphy`.
- Same-human projections: Aoife-as-HR-Specialist / Aoife-as-Benefits-Specialist.
- Audience types: `B2B_HR_ADMIN + B2C_CONSUMER`.
- Primary services: identity, forms, workflow-engine, payments, finops-portal, mail, drive, workplace-integration.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.28 Regulator Inspector Sergei Petrov
- Slug: `regulator-inspector-sergei-petrov`.
- Same-human projections: Sergei-as-regulator / Sergei-as-private-citizen.
- Audience types: `B2B_REGULATOR_EXTERNAL + GOV_INSPECTOR + B2C_CONSUMER`.
- Primary services: identity, audit-chain, compliance, governance, ops-dashboard-control-center, observability, workflow-engine, mail.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.29 Board director Patrick O'Reilly
- Slug: `board-director-patrick-oreilly`.
- Same-human projections: O'Reilly-on-Board-A / O'Reilly-on-Board-B / O'Reilly-on-Board-C.
- Audience types: `B2B_BOARD_DIRECTOR + B2C_CONSUMER`.
- Primary services: identity, governance, mail, calendar, drive, workflow-engine, audit-chain, compliance.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

### §14.30 Outside Counsel Wei-Yi Chen
- Slug: `outside-counsel-wei-yi-chen`.
- Same-human projections: Wei-Yi-as-counsel-for-A / Wei-Yi-as-counsel-for-B.
- Audience types: `B2B_EXTERNAL_COUNSEL + B2C_CONSUMER`.
- Primary services: identity, mail, drive, contract-lifecycle-mgmt, workflow-engine, audit-chain, compliance, governance.
- Boundary: personal tenant survives work-role revocation.
- Recovery: ADR-0299 restores identity, not stale role grants.
- Community: use `microservices/community/PRD.md`; `anonymous/` remains deleted.

## §15. 56-µservice Persona Coverage Rollup

| # | µservice | Top-30 direct coverage |
|---:|---|---|
| 01 | identity | yejin-park, marcus-chen, aiyana-singh, tomas-garcia, hiroshi-tanaka, anya-mironova |
| 02 | tenancy | yejin-park, marcus-chen, diana-reyes, chris-volkov |
| 03 | policy-engine | yejin-park, aiyana-singh, anya-mironova, ciso-yuki-park, officer-rodriguez-police, summer-intern-priscilla-sharma |
| 04 | audit-chain | yejin-park, aiyana-singh, anya-mironova, diana-reyes, sam-okafor, cfo-helena-brandt |
| 05 | workflow-engine | yejin-park, marcus-chen, tomas-garcia, priya-krishnan, sam-okafor, ceo-aoki-tanaka |
| 06 | workflow-studio | aiyana-singh, hiroshi-tanaka, anya-mironova, diana-reyes, chris-volkov |
| 07 | community | yejin-park, aiyana-singh, tomas-garcia, hiroshi-tanaka, anya-mironova, priya-krishnan |
| 08 | messenger | yejin-park, marcus-chen, tomas-garcia, hiroshi-tanaka, anya-mironova, priya-krishnan |
| 09 | mail | yejin-park, marcus-chen, tomas-garcia, hiroshi-tanaka, anya-mironova, priya-krishnan |
| 10 | calendar | yejin-park, marcus-chen, hiroshi-tanaka, priya-krishnan, chris-volkov, ceo-aoki-tanaka |
| 11 | meet | marcus-chen |
| 12 | drive | aiyana-singh, hiroshi-tanaka, anya-mironova, priya-krishnan, sam-okafor, chris-volkov |
| 13 | notes | aiyana-singh, hiroshi-tanaka, anya-mironova, diana-reyes, father-lopez-priest |
| 14 | forms | priya-krishnan, chro-linda-foster, ms-patel-teacher, benefits-specialist-aoife-murphy |
| 15 | payments | yejin-park, tomas-garcia, hiroshi-tanaka, diana-reyes, priya-krishnan, sam-okafor |
| 16 | finops-portal | yejin-park, tomas-garcia, chris-volkov, cfo-helena-brandt, sarah-kim-delivery, tomas-garcia-jr-farmer |
| 17 | marketplace | yejin-park, tomas-garcia, chris-volkov, cfo-helena-brandt, sarah-kim-delivery, tomas-garcia-jr-farmer |
| 18 | ontology | aiyana-singh, tomas-garcia-jr-farmer |
| 19 | intelligence | aiyana-singh |
| 20 | observability | diana-reyes, sam-okafor, ciso-yuki-park, captain-chen-pilot, officer-rodriguez-police, trader-mei-lin |
| 21 | compliance | yejin-park, marcus-chen, tomas-garcia, anya-mironova, diana-reyes, sam-okafor |
| 22 | governance | marcus-chen, sam-okafor, ceo-aoki-tanaka, ciso-yuki-park, regulator-inspector-sergei-petrov, board-director-patrick-oreilly |
| 23 | ops-dashboard-control-center | marcus-chen, diana-reyes, ceo-aoki-tanaka, ciso-yuki-park, regulator-inspector-sergei-petrov, board-director-patrick-oreilly |
| 24 | workplace-integration | priya-krishnan, chro-linda-foster, carlos-martinez-forklift, benefits-specialist-aoife-murphy |
| 25 | developer-sdk | aiyana-singh, summer-intern-priscilla-sharma |
| 26 | foundry | aiyana-singh, summer-intern-priscilla-sharma |
| 27 | api-gateway | ciso-yuki-park |
| 28 | cell | ambient-only; invariant-covered in every dossier |
| 29 | cloud-secrets | ciso-yuki-park |
| 30 | analytics | tomas-garcia, sam-okafor, benefits-specialist-aoife-murphy |
| 31 | search | ambient-only; invariant-covered in every dossier |
| 32 | notifications | carlos-martinez-forklift, sarah-kim-delivery, captain-chen-pilot, officer-rodriguez-police |
| 33 | social | ambient-only; invariant-covered in every dossier |
| 34 | shorts | aiyana-singh, anya-mironova |
| 35 | ads | ambient-only; invariant-covered in every dossier |
| 36 | personal-health-tracker | yejin-park, hiroshi-tanaka, captain-chen-pilot, dr-tanaka-surgeon, medical-resident-dr-sun-mi-kim |
| 37 | crm | ambient-only; invariant-covered in every dossier |
| 38 | marketing-automation | ambient-only; invariant-covered in every dossier |
| 39 | contact-center | ambient-only; invariant-covered in every dossier |
| 40 | performance-mgmt | priya-krishnan, chro-linda-foster |
| 41 | learning-mgmt | priya-krishnan, chro-linda-foster, ms-patel-teacher, summer-intern-priscilla-sharma, medical-resident-dr-sun-mi-kim |
| 42 | itsm | ambient-only; invariant-covered in every dossier |
| 43 | incident-mgmt | ciso-yuki-park, carlos-martinez-forklift, captain-chen-pilot, dr-tanaka-surgeon, officer-rodriguez-police |
| 44 | financial-planning | marcus-chen, ceo-aoki-tanaka, cfo-helena-brandt, board-director-patrick-oreilly |
| 45 | data-warehouse | cfo-helena-brandt, investment-banker-yuna-ahn, trader-mei-lin |
| 46 | contract-lifecycle-mgmt | outside-counsel-wei-yi-chen |
| 47 | whiteboard | ambient-only; invariant-covered in every dossier |
| 48 | design-collaboration | ambient-only; invariant-covered in every dossier |
| 49 | erp-finance | cfo-helena-brandt |
| 50 | erp-procurement | tomas-garcia-jr-farmer |
| 51 | erp-inventory | tomas-garcia, carlos-martinez-forklift, sarah-kim-delivery, tomas-garcia-jr-farmer |
| 52 | erp-manufacturing | ambient-only; invariant-covered in every dossier |
| 53 | erp-sales | tomas-garcia |
| 54 | erp-hr | ambient-only; invariant-covered in every dossier |
| 55 | erp-projects | ambient-only; invariant-covered in every dossier |
| 56 | erp-analytics | marcus-chen, ceo-aoki-tanaka |

## §16. Effective Count Reconciliation

- The brief requests a persona graph table of ~127 personas.
- The body table contains compatibility rows that project the same human across sub-roles where the source matrix had overlapping centers of gravity.
- The top-30 dossiers bind to the explicit priority anchors named in the brief.
- Future normalization from 129 display rows to 127 effective rows must preserve the dossier slugs in §13.1.
- Do not delete bridge rows without supersession metadata.
- Do not turn a role projection into a separate user account.
- Do not make audience type a permanent property of the human.
- Do not move community persona content to any `anonymous/` path.
- Do not weaken ADR-0311 work/personal separation for convenience.

## §17. Validation Evidence Expected

- `wc -l docs/personas/MASTER-ROSTER-2026-05-21.md docs/personas/*.md` proves line floors.
- `find docs/personas -maxdepth 1 -type f -name '*.md'` shows the master roster plus 30 dossier files.
- `rg -n "anonymous/" docs/personas` should show only negative-path references, not active dependencies.
- `rg -n "ADR-0319|ADR-0320|ADR-0316" docs/personas` proves in-flight ADR citation.
- `rg -n "microservice_count_authority: 56|µservice count is **56**" docs/personas` proves count alignment.
- `git status --short docs/personas` should show only persona deliverable files.

## §18. Stop Condition

- Master roster ≥800 lines.
- Exactly 30 priority dossier files exist.
- Every priority dossier ≥300 lines.
- Every dossier has §A through §K.
- Master roster has §1 through §7 plus this annex.
- No ADR, standard, PRD, or µservice file changed.
- Validation commands have fresh evidence.

