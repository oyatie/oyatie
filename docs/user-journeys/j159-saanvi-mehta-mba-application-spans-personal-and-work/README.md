---
doc_class: User-Journey-README
journey_id: j159-saanvi-mehta-mba-application-spans-personal-and-work
slice: dual-tenant-personal-vs-work-mba-application-with-cedar-boundary
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Mid-Career MBA Applicant Saanvi Mehta
audience_type: B2C_PERSONAL_TENANT + B2B_KNOWLEDGE_WORKER_WORK_TENANT
microservice_count: 6
pack_overlay_anchor: ADR-0311-dual-tenant-identity-personal-vs-work-boundary + EDU-AACSB-pack + GMAC-GMAT-pack + EU-GDPR + US-FERPA + UK-DPA-2018 + IN-DPDP-2023
related_adrs:
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0243-cedar-as-universal-gate
  - ADR-0263-observability-emission-contract
  - ADR-0252-hlc-default-truetime-tier
  - ADR-0255-byok-everywhere-credentials
  - ADR-0248-amazon-shape-cellular-architecture
  - ADR-0250-build-ahead-of-certification
  - ADR-0247-self-modification-doctrine
  - ADR-0251-compliance-pack-primitive
---

# j159 — Saanvi Mehta: MBA application straddling personal and work tenants

## At a glance

Saanvi Mehta is a **34-year-old senior product manager** at **Stripe India Pvt Ltd** (Bangalore, Karnataka) — five-and-a-half years into a career that began at HUL in Mumbai before she lateral-moved to Stripe in 2022 for the Asia-Pacific platform team. She is Indian, born in Pune, educated at IIT Bombay (B.Tech Electrical Engineering, 2014) and IIM Calcutta (PGDM Finance + Strategy, 2019), fluent in Hindi + English + Marathi + functional Gujarati, lives with her husband Arjun (a hospital-administrator at Manipal Hospitals Whitefield) and 4-year-old daughter Anaya in a 3BHK rental in Indiranagar. Her work-tenant is **`stripe-india-pvt-ltd`**; her personal-tenant is **`saanvi.mehta.personal`** (created when she onboarded oyatie on her own device in late 2026 to migrate away from a fragmented Gmail + Google Drive + Notion + DocuSign personal stack).

It is **Sunday December 6, 2026, 21:47 IST**. The Round 2 MBA application window closes on **January 5, 2027 23:59 ET** (which is January 6, 2027 10:29 IST) for Stanford GSB, Wharton, Harvard Business School, Chicago Booth, and INSEAD Singapore-Fontainebleau. She is applying to all five. Her GMAT (Focus Edition) score from her October 18, 2026 sitting at the Pearson VUE test center in Whitefield is **745 (Q90 V87 DI88, total 99th percentile)** — high enough to be competitive everywhere. Her undergraduate IIT Bombay CGPA is 8.9/10; her IIM Calcutta is 3.62/4.33 (top quintile). She has prepared her essays for three months. Tonight is the night she has to submit Wharton — its Round 2 portal closes earliest at 11:59 PM ET January 5 (no extensions for "we lost track of timezone").

What makes this journey hard is not the writing. The writing is done. What makes it hard is that **simultaneously**, **Stripe's annual performance review cycle** is in motion — her manager **Priya Krishnamurthy** (Director of Product, Stripe APAC) is collecting peer feedback this week, the calibration meeting is December 14, and HR (a US-based principal `hr-systems@stripe-corporate-us`) is auditing Saanvi's work-tenant document access for the standard "are people doing what they should be" anti-leak compliance sweep.

The friction is this: her **recommendation letters** are being written by **two recommenders who are work-tenant principals** —

1. **Priya Krishnamurthy** (Saanvi's current manager, Stripe APAC) — primary recommender. Priya needs to write the recommendation as Saanvi's manager (using work-context context) but **cannot** have the recommendation visible to anyone else at Stripe, including HR. The recommendation is a **personal-tenant artifact** that Priya authors **from her work-tenant identity** through a Cedar cross-tenant capability.
2. **Rajesh Subramanian** (Saanvi's former skip-manager at HUL, now VP at Marico — `marico-india-pvt-ltd` tenant) — supplementary recommender. Cross-tenant from a third party.

Saanvi must be able to:

- Write essays in her **personal-tenant** (`saanvi.mehta.personal`) using `drive` + `mail` for the school correspondence — without ANY of this leaking to her Stripe work-tenant
- Receive **recommendation letters** from Priya and Rajesh into her personal-tenant — even though Priya is a work-tenant principal
- Pay the application fees ($275 × 5 = $1,375 USD) via her personal-tenant `payments` µservice (her HDFC personal credit card, NOT her Stripe Amex corporate card)
- Take GMAT prep refreshers via `learning-management` (Manhattan Prep Plus subscription, personal-tenant)
- Engage with the **Wharton 2027 Round 2 admit candidate community** (`community` µservice) — private peer-mentorship space hosted on a third tenant `wharton-r2-2027-prospective-applicants-community` that she joined September 2026
- And critically, **none of the above must be visible** to Stripe's HR audit sweep, to Priya outside the recommendation-letter scope, or to any work-tenant principal incidentally

This journey covers the next **9 days of Saanvi's overlapping personal + work life** from Sunday Dec 6 21:47 IST through Tuesday Dec 15 19:18 IST (the calibration day at Stripe + the last upload to Booth + INSEAD), with the following beats:

1. **Sunday Dec 6 21:47 IST** — Saanvi finalizes her Wharton "Why an MBA, Why Wharton, Why Now" essay (650 words) in personal-tenant `drive` after Anaya is asleep; she shares the draft with her husband Arjun (`arjun.mehta.personal` tenant) for spousal review via cross-tenant invitation
2. **Monday Dec 7 09:30 IST** — Priya Krishnamurthy receives Saanvi's recommender invitation from Wharton's recommender portal forwarded via the personal-tenant `mail` µservice; Priya must accept from her **work-tenant identity** (`priya.krishnamurthy@stripe-india-pvt-ltd`) because Saanvi is her direct report, but the recommendation lands in `saanvi.mehta.personal`'s `drive` — Cedar permit `recommendation.cross_tenant_author_to_personal` evaluates
3. **Wednesday Dec 9 14:18 IST** — Stripe's HR audit principal `hr-systems@stripe-corporate-us` runs a doc-access compliance sweep across all Stripe India work-tenant employees; the sweep tries to walk Saanvi's work-tenant Drive — Cedar **forbids** any read of her personal-tenant drive; the work-tenant sweep returns zero personal-tenant artifacts
4. **Friday Dec 11 22:14 IST** — Saanvi submits Wharton Round 2 application; payment via `payments` (HDFC personal card $275) settles; the Wharton tenant (`wharton-mba-admissions-us`) is briefly invited as a third tenant for the cross-tenant ack
5. **Saturday Dec 12 19:30 IST** — Saanvi takes a Manhattan Prep GMAT refresher session via `learning-management` (90-minute Quant DI module) — score-up insurance in case any school wants a retake
6. **Sunday Dec 13 11:08 IST** — Saanvi posts a question to the `wharton-r2-2027-prospective-applicants-community` about whether to disclose a 2023 personal-leave gap in the optional essay; 6 peer-applicants reply; the entire interaction is private to the community tenant (third tenant, isolated from work + personal)
7. **Monday Dec 14 09:00–18:00 IST** — Stripe's calibration meeting day; Priya advocates for Saanvi's "Exceeds" rating in the work-tenant calibration tool; the rating goes through. Saanvi never logs into her personal-tenant during work hours; the boundary is **clean**
8. **Tuesday Dec 15 19:18 IST** — Saanvi submits Booth + INSEAD applications (Stanford on Dec 18, Harvard on Dec 22 follow); the journey closes when all five Wharton/Stanford/Harvard/Booth/INSEAD Round 2 applications have submitted-confirmed status in her personal-tenant, with zero work-tenant data crossover

Primary microservices: `identity`, `mail`, `drive`, `payments`, `community`, `learning-management`. Secondary: `tenancy` (dual-tenant boundary), `messenger` (recommender + community DMs), `tasks` (Saanvi's application checklist), `workflow-engine` (recommender invitation lifecycle), `notes` (essay drafts), `calendar` (Round 2 deadlines), `compliance` (GDPR + DPDP + FERPA pack activation), `audit-chain` (cross-tenant transition seals), `observability`, `analytics`.

This is a **white/back-office, knowledge-worker, dual-tenant-defining** journey. It demonstrates that oyatie's `identity → tenancy → drive + mail + payments` substrate, gated by ADR-0311's personal-vs-work boundary doctrine, allows a 34-year-old IIM Calcutta-trained product manager at a US-headquartered fintech to do **the most career-defining personal thing of her decade** (applying to top-5 global MBA programs) without leaking to her employer, AND simultaneously allows her work-tenant identity to be the legitimate signing principal for actions whose home is her personal tenant (her manager's recommendation letter), via a Cedar-gated cross-tenant capability rather than ambient access.

## Why this journey matters

Saanvi Mehta is **MASTER-ROSTER §4.2 row 103** — the canonical "knowledge-worker dual-tenant identity" persona. She is the test bench for ADR-0311's most operationally hostile case: when an action originates **on the work-tenant principal** (Priya as Saanvi's manager writing a recommendation IS a work-context action — Priya can only speak to Saanvi's professional performance because she is Saanvi's manager AT WORK) but the action's **home, retention class, audit class, and visibility scope** are all on the **personal tenant**.

The persona covers an estimated **180 million globally** mid-career knowledge workers who simultaneously hold personal projects (graduate school applications, side businesses, freelance work, advocacy work, healthcare proxy responsibility for elderly parents, etc.) and full-time employer relationships. The category is acutely under-served by enterprise SaaS because most products force a single-tenant model where personal artifacts leak into the employer's audit surface (Microsoft 365 + Google Workspace are the obvious offenders — both default to mixing personal browser sessions with corporate ones), AND most consumer-side products force the inverse where work principals cannot legitimately participate in personal actions (Gmail can technically receive an MBA recommendation from a `@stripe.com` address but the recommendation is then forever in the employer's mail system).

The journey closes:

- **Critical-path row 29** (Dual-tenant Cedar boundary with cross-tenant capability for recommendations)
- **Critical-path row 30** (Personal-tenant `payments` provider-credential BYOK with HDFC personal card, never work card; ADR-0255 §D-4)
- **Critical-path row 31** (Third-tenant community participation isolated from both personal-broadcast and work-tenant)
- **Critical-path row 32** (Work-tenant HR audit MUST NOT walk into personal-tenant drive — Cedar forbid is mandatory)

Hyperscaler benchmark: Google Workspace + Microsoft 365 + Slack + Notion Workspaces + Dropbox Personal+Business. The unique part of oyatie is that **Cedar policy makes personal-vs-work boundary a first-class enforceable invariant**, AND **cross-tenant capability is granted action-by-action** (Priya can author a recommendation in Saanvi's personal-tenant — but only that action, not browse Saanvi's other personal-tenant artifacts — and not via "share this folder" delegation that decays into permanent access).

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| `story.md` | Beat-by-beat 9-day journey from Wharton essay finalization through 5th-school submit at INSEAD | Hindi/English code-switching dialogue, specific schools (Wharton/Stanford/HBS/Booth/INSEAD), specific essay prompts + word counts, GMAT 745 specifics, HDFC card payments, Stripe-corporate-US HR sweep, Priya Krishnamurthy + Rajesh Subramanian named recommenders, Whitefield/Indiranagar geography, daughter Anaya's bedtime as boundary anchor |
| `ux-flow.md` | Personal-tenant tablet (Saanvi's iPad Pro M4), work laptop (her Stripe MacBook Pro 16"), recommender's web view (Priya's Stripe MacBook), HR audit dashboard, Wharton portal, community web | Visible-tenant chip indicator at top of every screen; tenant-switcher modal; payment screen with HDFC card and NO corporate card option; calibration screen in work-tenant has zero personal-tenant artifacts visible |
| `handshake.md` | Per-µservice API across `saanvi.mehta.personal` + `stripe-india-pvt-ltd` + `marico-india-pvt-ltd` + `wharton-mba-admissions-us` + `wharton-r2-2027-prospective-applicants-community` + `arjun.mehta.personal` + `stripe-corporate-us` tenants | Each row names source + target tenant, Cedar permit, cross-tenant audit dual-seal class, payment routing through provider-credential BYOK credentials (ADR-0255 §D-4) |
| `integration-test-plan.md` | Dual-tenant boundary tests + recommender cross-tenant tests + payment isolation tests + community-tenant isolation tests + HR audit refusal tests + GMAT prep visibility tests | Each test names seed values + expected event chain + ADR-0311 invariant probe pass/fail thresholds |
| `schemas/openapi-dual-tenant-mba.json` | OpenAPI for recommender invitations + personal-tenant payments + cross-tenant ack endpoints | Recommender invite lifecycle + Cedar-gated cross-tenant capability + personal-tenant payment flow |
| `schemas/cedar-policy.cedar` | Dual-tenant + recommender + HR-audit-refusal Cedar policy | Personal-vs-work boundary explicit forbid + recommender capability permit + HR-audit forbid into personal-tenant + provider-credential BYOK card-routing context (ADR-0255 §D-4) |
| `schemas/journey-messages.proto` | proto3 for all RPCs | UTF-8 NFC Hindi-Devanagari names; dual-tenant principal type; recommender capability proto; personal-tenant payment proto with explicit `not_corporate_card == true` guard |
| `schemas/mba-application-state-machine.yaml` | 7-state MBA application lifecycle | Per-school state machine: `essay_drafting → recommender_invited → recommender_letter_received → fee_paid → submitted → ack_received → decision_received` (decision_received not in scope for this journey) |
| `schemas/recommender-invitation-form.json` | Cross-tenant recommender invitation schema | Required fields: school name, deadline, prompt, recommender's home tenant, signature method, retention overlay (15 years per AACSB), cross-tenant capability scope (write-once, no browse) |

## The six microservices in scope

| µservice | Role | Critical-path row |
|---|---|---|
| `identity` | Saanvi's dual-tenant principal `saanvi.mehta.personal` + `saanvi.mehta@stripe-india-pvt-ltd`; ADR-0311 boundary enforcement; passkey-per-tenant; tenant chip at session start | row 29 |
| `mail` | Personal-tenant inbox receives Wharton recommender-invite forwards + school correspondence; work-tenant mail receives Stripe HR calibration emails; never crosses | row 29 |
| `drive` | Personal-tenant holds essays + transcripts + GMAT score reports + recommender letters; work-tenant holds Stripe APAC roadmap docs; cross-tenant capability grants Priya WRITE-ONLY access to a specific recommender-letter slot | row 29 |
| `payments` | Personal-tenant accepts HDFC personal card for $275 × 5 = $1,375 application fees; provider-credential BYOK routing context (ADR-0255 §D-4); corporate card explicitly refused for personal-tenant payments | row 30 |
| `community` | Third-tenant `wharton-r2-2027-prospective-applicants-community` membership; private peer forum; isolated from both personal-broadcast and work-tenant | row 31 |
| `learning-management` | Personal-tenant Manhattan Prep Plus subscription; GMAT prep modules + retake-prep insurance; pulls GMAT 745 score history | row 30 |

## Secondary microservices touched

| µservice | Touch reason |
|---|---|
| `tenancy` | Three primary tenants (`saanvi.mehta.personal` + `stripe-india-pvt-ltd` + `wharton-r2-2027-prospective-applicants-community`); two spousal/recommender tenants invited (`arjun.mehta.personal` + `marico-india-pvt-ltd`); one application target tenant (`wharton-mba-admissions-us`) briefly engaged; one HR audit principal (`stripe-corporate-us`) |
| `compliance` | Activates EU-GDPR (Wharton has EU students), US-FERPA (Stanford + HBS + Booth tenant data), IN-DPDP-2023 (Saanvi's Indian residency), UK-DPA-2018 (INSEAD has UK affiliations), AACSB-EDU pack |
| `messenger` | MLS-encrypted DMs with Priya (limited to recommender topic) + spouse Arjun (essay review) + community peers |
| `tasks` | Saanvi's application checklist (5 schools × 9 sub-tasks = 45 tasks) |
| `workflow-engine` | Recommender-invitation lifecycle (invite sent → accepted → letter drafted → letter submitted → school confirms receipt) |
| `notes` | Essay drafts; "why MBA, why this school, why now" outlines |
| `calendar` | Round 2 deadlines (5 schools, 5 due dates Dec 5–Jan 5); GMAT retake slot held Feb 14 just in case |
| `audit-chain` | Every cross-tenant transition dual-seals (personal + work tenant + community tenant + admissions tenant) |
| `crm` | Recommender CRM-light: relationship records of Priya + Rajesh + 2 backup recommenders held in personal-tenant |
| `analytics` | Saanvi's personal-tenant dashboard: 5 schools' completion %; deadline countdown; GMAT prep hours logged |

## Pack overlays

| Pack | Activation reason |
|---|---|
| ADR-0311 personal-vs-work pack | Saanvi is dual-tenant; the entire journey is a stress test |
| AACSB-EDU pack | Applications are to AACSB-accredited business schools; 15-year retention for application records |
| GMAC-GMAT pack | GMAT score retrieval + score-send-to-school subflow |
| EU-GDPR | Wharton has EU applicants; INSEAD has EU campus (Fontainebleau); data residency considerations |
| IN-DPDP-2023 | Saanvi's primary residency is India; DPDP applies to her personal data |
| US-FERPA | US schools' educational records pack; recommender letters once submitted are FERPA records |
| UK-DPA-2018 | INSEAD has UK affiliations and may transfer to UK |
| Stripe-internal-compliance-pack | Saanvi's work-tenant runs SOX + PCI-DSS + ISO-27001 packs; these MUST NOT walk into personal-tenant |

## Regulatory anchors

1. ADR-0311 dual-tenant identity personal-vs-work boundary doctrine
2. ADR-0244 tenant scoping primitive
3. ADR-0263 audit dual-seal on cross-tenant transitions
4. ADR-0255 §D-4 provider-credential BYOK credentials for payments
5. ADR-0251 compliance pack primitive
6. EU-GDPR Articles 6, 9, 17 (right to erasure on personal-tenant withdrawal)
7. IN-DPDP-2023 Section 6 (lawful processing) + Section 12 (data fiduciary obligations)
8. US-FERPA 34 CFR 99.31 (recommendation letter confidentiality)
9. AACSB Standard 9 (degree-applicant records retention)
10. PCI-DSS v4 (payments via HDFC personal card; corporate card refusal context)

## Cell + certification matrix

| Cell | Certification | Journey use |
|---|---|---|
| `ap-mumbai-primary` | IN-DPDP + ISO 27001 + PCI-DSS | Primary cell for Saanvi's personal-tenant + Stripe-India work-tenant (data residency) |
| `us-east-virginia-secondary` | US-FERPA + SOC 2 Type II + ISO 27001 | Wharton + Stanford + HBS + Booth tenant residency |
| `eu-paris-tertiary` | EU-GDPR + ISO 27001 | INSEAD Fontainebleau campus residency |
| `sg-singapore-secondary` | SG-PDPA + ISO 27001 | INSEAD Singapore campus residency |

## Cedar personal-vs-work boundary policy (excerpt — full text in `schemas/cedar-policy.cedar`)

```cedar
// ADR-0311 personal-vs-work boundary — Cedar invariant
// The fundamental rule: a work-tenant principal NEVER has ambient access to personal-tenant data
// UNLESS the personal-tenant principal has explicitly granted a write-only capability for a
// specific named action on a specific named resource.

forbid (
    principal,
    action,
    resource is PersonalTenantResource
) when {
    principal.tenant_class == "work_tenant" &&
    !context.cross_tenant_capability_grant_present &&
    !context.principal_is_same_human_dual_identity
}
advice {
    "forbid_rule_id": "forbid-work-tenant-ambient-access-to-personal-tenant",
    "doctrine_anchor": "ADR-0311",
    "audit_class": "EVT-J159-CEDAR-DENY-WORK-TENANT-INTO-PERSONAL-014a"
};

// PERMIT — Recommender cross-tenant capability (write-only, one-shot)
permit (
    principal is User,
    action == Action::"drive.write_recommendation_letter_to_slot",
    resource is RecommendationLetterSlot
) when {
    context.capability_grant.granted_by_personal_tenant_principal == true &&
    context.capability_grant.scope == "write_once_no_browse" &&
    resource.target_tenant != principal.tenant
};
```

## Acceptance summary

| AC | Result expected |
|---|---|
| AC-J159-001 | Saanvi finalizes Wharton essay in personal-tenant drive; word count 647 of 650 max; audit `EVT-J159-ESSAY-FINALIZED-001` sealed in `saanvi.mehta.personal` |
| AC-J159-002 | Priya accepts recommender invitation via work-tenant identity; Cedar permits write-once capability into Saanvi's personal-tenant; audit `EVT-J159-RECOMMENDER-ACCEPT-002` dual-sealed in `saanvi.mehta.personal` AND `stripe-india-pvt-ltd` |
| AC-J159-003 | Rajesh accepts secondary recommender invitation via Marico tenant; audit `EVT-J159-RECOMMENDER-MARICO-ACCEPT-003` dual-sealed in `saanvi.mehta.personal` AND `marico-india-pvt-ltd` |
| AC-J159-004 | Stripe-corporate-US HR sweep walks `stripe-india-pvt-ltd` work-tenant for Saanvi; ZERO personal-tenant artifacts returned; audit `EVT-J159-HR-SWEEP-NO-PERSONAL-LEAK-004` confirms refusal |
| AC-J159-005 | Wharton application fee $275 charged to HDFC personal card via personal-tenant `payments`; Stripe corporate Amex explicitly refused with reason "corporate_card_not_eligible_for_personal_tenant_payment"; audit `EVT-J159-WHARTON-FEE-PAID-005` |
| AC-J159-006 | Wharton Round 2 application submitted at 22:14 IST Friday Dec 11; cross-tenant ack `EVT-J159-WHARTON-ACK-006` dual-sealed |
| AC-J159-007 | GMAT 745 score-send to all 5 schools via `learning-management`; audit `EVT-J159-GMAT-SCORE-SEND-007` |
| AC-J159-008 | Community participation in `wharton-r2-2027-prospective-applicants-community` posts isolated from work-tenant; audit `EVT-J159-COMMUNITY-PARTICIPATION-008` |
| AC-J159-009 | Stripe calibration meeting Dec 14 produces "Exceeds" rating for Saanvi; her personal-tenant has zero activity log entries during 09:00–18:00 IST that day (clean boundary); audit `EVT-J159-CALIBRATION-DAY-CLEAN-BOUNDARY-009` |
| AC-J159-010 | All 5 schools (Wharton/Stanford/HBS/Booth/INSEAD) reach `submitted` state by Dec 22 23:59 IST; final audit `EVT-J159-ALL-SCHOOLS-SUBMITTED-010` |
| AC-J159-011 | Diacritic + Devanagari fidelity: "Saanvi", "Priya Krishnamurthy", "Rajesh Subramanian", "Anaya", "Arjun" preserve UTF-8 NFC across all persisted fields; no transliteration to ASCII without explicit user request |
| AC-J159-012 | Spousal review: Arjun's `arjun.mehta.personal` tenant gains read-only access to Saanvi's essay drafts via spousal-tenant capability; Arjun's access scope is bounded; audit `EVT-J159-SPOUSAL-REVIEW-012` |
| AC-J159-013 | Withdrawal probe: Saanvi can withdraw a recommender invitation; the cross-tenant capability revokes within 90 seconds; subsequent write attempts by the recommender fail with 403 |

## Cross-references

- Persona dossier: `docs/personas/knowledge-worker-saanvi-mehta.md`
- MASTER-ROSTER §4.2 row 103
- Matrix §11 j159 recommendation
- Related: j155 (gray-collar dual-role), j157 (gray-collar mid-shift quality), j100 (pack rollout), j109 (cross-tenant freelance specialist), j115 (SaaS-vendor-API-multi-tenant)
- Pack roster: `packs/adr-0311-personal-vs-work/`, `packs/aacsb-edu/`, `packs/gmac-gmat/`, `packs/eu-gdpr/`, `packs/in-dpdp-2023/`, `packs/us-ferpa/`
- ADR-0311 dual-tenant identity boundary (the keystone for this journey)
- ADR-0244 tenant scoping
- ADR-0263 audit dual-seal
- ADR-0255 §D-4 provider-credential BYOK credentials

## Stop condition

This journey is complete when all 13 acceptance criteria pass on the seeded multi-tenant fixture, all 5 MBA Round 2 applications reach `submitted` state with their respective cross-tenant acks dual-sealed, the personal-vs-work boundary holds against all 9 stressors (HR sweep, calibration day, recommender cross-tenant write, payment routing, community participation, GMAT score-send, spousal review, recommender withdrawal, decision-window pre-emption), and the diacritic + Devanagari fidelity invariant holds across every persisted field.
