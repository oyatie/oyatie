---
doc_class: User-Journey-README
journey_id: j160-cleaning-co-tomas-horak-bid-cross-tenant-and-onboard
slice: cross-tenant-bid-marketplace-award-and-crew-onboard-cascade
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Czech Cleaning Co. Owner-Operator Tomáš Horák
audience_type: B2B_BLUE_COLLAR_SMALL_BIZ_OWNER + B2B_FACILITIES_PROCUREMENT
microservice_count: 5
pack_overlay_anchor: CZ-OZ-262-2006-zakonik-prace + CSN-EN-ISO-9001 + CZ-110-2019-Sb-data-protection + EU-GDPR + ISSA-CIMS-cleaning + CSN-EN-13549-cleaning-quality + CZ-VAT-235-2004-Sb
related_adrs:
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0243-cedar-as-universal-gate
  - ADR-0263-observability-emission-contract
  - ADR-0249-multi-category-marketplace-doctrine
  - ADR-0252-hlc-default-truetime-tier
  - ADR-0248-amazon-shape-cellular-architecture
  - ADR-0250-build-ahead-of-certification
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0245-substrate-vs-product-layering
---

# j160 — Tomáš Horák: cleaning-co bid → award → crew onboarding cascade

## At a glance

Tomáš Horák is a **41-year-old owner-operator** of **Úklid Horák s.r.o.** ("Horák Cleaning Ltd."), a 14-person commercial cleaning company based in **Plzeň, Czech Republic**, founded by him in 2017 after he left a project-manager role at a larger Prague facilities-management firm to start his own business. He is Czech (born in Klatovy, raised in Plzeň), speaks Czech (native), German (B2; learned for cross-border contracts with Bavarian firms), and English (B1; functional for European procurement). His company tenant is `uklid-horak-sro-plzen-cz`. He runs operations out of a 240 m² rented depot on Skvrňanská třída in Plzeň's Skvrňany district (warehousing for cleaning supplies + equipment + a small office). His 14 employees — 8 women (mostly Czech, plus one Slovak and two Vietnamese) and 6 men (Czech, plus one Ukrainian) — cover daytime office cleaning, end-of-tenancy deep cleans, and a growing portfolio of small industrial-facility contracts. Revenue 2025: CZK 18.4M (≈ €730k). Net margin: 11.3%.

It is **Tuesday October 14, 2026, 14:42 CET**. The Plzeň regional office of **PolyCraft Bohemia a.s.** (a mid-size Czech polymer-additives manufacturer, 380 employees, three industrial sites: Plzeň + Mladá Boleslav + Ostrava) has posted an open bid via the oyatie `marketplace` µservice for **facilities cleaning services for their Plzeň plant** — a 12,400 m² facility comprising 9,200 m² of production-floor area, 1,800 m² of warehouse, 980 m² of admin offices, and 420 m² of common areas (lockers, canteen, restrooms). The bid covers a **24-month service contract** starting January 4, 2027 with monthly invoicing, ISO-9001 quality audit clauses, ČSN-EN-13549 cleaning-quality measurement obligations, GDPR + CZ-110/2019 Sb. data-handling on facility-access records, and a CZK 4.2M annual ceiling (≈ €166k/year, CZK 8.4M over 24 months). Three competitor cleaning firms have already submitted bids (visible to Tomáš as anonymized bid-rank slots).

The bid window closes Friday October 17, 2026 17:00 CET — three calendar days from now. PolyCraft Bohemia's tenant on oyatie is `polycraft-bohemia-as-plzen-cz`; their procurement principal is **Ing. Martina Procházková** (a 38-year-old procurement manager, MBA from VŠE Praha, has been at PolyCraft 6 years, runs all three sites' facility procurement). The award decision is calendar-anchored to **Monday October 27, 2026 14:00 CET**; if Tomáš wins, contract start is **Monday January 4, 2027 06:00 CET**, which means his crew must be **fully onboarded** (employment paperwork, ČSN-262-2006 occupational-safety training, GDPR data-handling training for facility-access records, facility-specific PolyCraft induction, biometric-badge enrollment to PolyCraft's access system) by **Sunday January 3, 2027 23:59 CET**.

This journey covers the **81 days from bid prep through full crew onboarding** with the following spine of beats:

1. **Tuesday Oct 14 14:42–18:18 CET** — Tomáš reads the bid request on his oyatie tablet (Lenovo Tab P12 Pro in the depot office); he uses `marketplace` to read the structured bid spec; he runs `workflow-engine`'s built-in bid-prep workflow to assemble cost estimates, schedule estimates, certifications attached
2. **Wednesday Oct 15 08:00–14:18 CET** — Tomáš walks the PolyCraft site (Procházková arranged the site walk); he photographs key zones; he records voice-notes about specific cleaning challenges (the production-floor solvent residue, the lockers' chronic odor problem, the canteen's grease trap)
3. **Wednesday Oct 15 16:42 CET** — Tomáš submits his bid via `marketplace` cross-tenant API; bid amount **CZK 7.94M** for 24 months (about CZK 331k/month including supplies); the bid carries his company's ISO-9001 cert, ČSN-EN-13549 cleaning-quality assessment plan, and his proposed crew composition (5 dedicated FTE for the Plzeň contract)
4. **Friday Oct 17 17:00 CET** — bid window closes; 5 bidders in total; Procházková enters review window
5. **Monday Oct 27 14:00 CET** — PolyCraft awards the contract to Úklid Horák; cross-tenant award notification via `messenger`; `workflow-engine` advances the bid lifecycle from `bid_submitted` to `award_received` to `contract_in_negotiation`
6. **Tuesday Oct 28–Friday Nov 7 CET** — Tomáš + Procházková negotiate contract terms (specific liability clauses, weekend-cover SLA, holiday-cover protocol, escalation chain); contract signed Friday Nov 7 11:18 CET via dual-tenant DSA + qualified electronic signature; `EVT-J160-CONTRACT-SIGNED-006` dual-sealed
7. **Monday Nov 10 09:00 CET** — Tomáš opens the crew-hiring workflow; existing 14 employees can't all rotate to PolyCraft (PolyCraft needs 5 dedicated FTE); he hires 3 new staff (a foreman/team-lead + 2 cleaners) via the `marketplace` labor-pool subflow + 2 internal rotations
8. **Wed Nov 12–Tue Dec 2 CET** — `tenancy` µservice provisions sub-tenant identity for the 3 new hires within `uklid-horak-sro-plzen-cz`; payroll setup + Czech OSSZ + ZP (health insurance) registration; `workflow-engine` drives 23-step employee onboarding
9. **Wed Dec 3–Mon Dec 22 CET** — ČSN-262-2006 safety training + GDPR training + facility-specific PolyCraft induction + biometric-badge enrollment; `community` µservice connects Tomáš to the Czech cleaning-industry peer community for cross-firm advice; specific Czech regulatory anchors handled (Hygienická stanice Plzeňského kraje permits, OOPP equipment specifications per ČSN-EN-407 chemical-glove ratings)
10. **Sat Jan 3 2027 23:42 CET** — final readiness check by `workflow-engine`; all 5 dedicated crew (the 3 new hires + 2 rotated) have completed onboarding; biometric badges active on PolyCraft's access system; Tomáš transmits the readiness attestation to Procházková
11. **Mon Jan 4 2027 05:42 CET** — first morning of service; Tomáš himself drives to PolyCraft to introduce the crew and walk the first shift; contract goes live; `EVT-J160-CONTRACT-LIVE-007` seals

Primary microservices: `marketplace`, `workflow-engine`, `payments`, `tenancy`, `community`. Secondary: `messenger` (cross-tenant negotiations + ongoing comms), `identity` (employee + crew passkey enrollment + biometric-badge cross-tenant), `tasks` (23-step onboarding x 5 crew = 115 tasks materialized), `notes` (Tomáš's site-walk + crew briefings + safety protocols), `crm` (PolyCraft relationship record), `contract-lifecycle-management` (24-month MSA + Czech-specific clauses + ICCSN-262 + ICCSN-13549), `compliance` (CZ-OZ-262-2006 + CSN-EN-ISO-9001 + CZ-110/2019-Sb + EU-GDPR + ISSA-CIMS + CSN-EN-13549 + CZ-VAT-235/2004-Sb), `audit-chain`, `observability`, `analytics`.

This is a **blue-collar small-business, cross-tenant, multi-month** journey. It demonstrates that oyatie's `marketplace → workflow-engine → tenancy → payments` substrate, gated by Czech regulatory packs, supports a 14-person cleaning company to win + execute a 24-month industrial cleaning contract against larger competitors WITHOUT becoming a SaaS-stack-management job for the owner. Tomáš is a competent operator but he is not an IT person; the system has to be **legible to him in Czech**, has to preserve the diacritics in his name and his employees' names (Tomáš Horák, Lenka Šimková, Hoàng Văn Long, Іван Шевченко, Mária Kováčová) at byte-level fidelity, has to integrate with the Czech state systems (OSSZ for social insurance + ZP for health insurance + Finanční úřad for VAT + Datová schránka for the electronic-mailbox), and has to enable cross-tenant cooperation with PolyCraft's more sophisticated procurement systems without forcing Tomáš to be the IT-translator between them.

## Why this journey matters

Tomáš Horák is **MASTER-ROSTER §5.1 row 104** — the canonical blue-collar small-business owner persona. He is the test bench for oyatie's claim that the same substrate that powers Stripe-scale companies also powers a 14-person cleaning firm in Plzeň, AND that the substrate respects national regulatory texture (the Czech labor code's specific provisions, ČSN technical standards, Czech VAT registration) without forcing the local firm to translate into US-first concepts.

The persona covers an estimated **45 million European blue-collar small-business owner-operators** in fragmented service industries (cleaning, security, landscaping, building maintenance, logistics, food service) where bidding for institutional contracts is the path from sole-proprietor scale to mid-market scale, but where the bidding + onboarding + compliance cost stack often eats the margin gain. The category is acutely under-served by SaaS — there are bid-management tools (BidExpress, ProConnect), there are cleaning-specific operations tools (Janitorial Manager, Swept, Tennant Insights), there are payroll tools (Czech firms use Pohoda + Money S3 widely) — but no integrated substrate that lets a 14-person Czech cleaning company present coherently to a 380-person manufacturer's procurement system, especially across the diacritic + locale + regulatory + biometric-badge boundaries.

The journey closes:

- **Critical-path row 33** (Cross-tenant marketplace bid with structured cost + cert + crew composition)
- **Critical-path row 34** (Bid-award → contract-negotiate → contract-sign cross-tenant cascade)
- **Critical-path row 35** (Crew hiring + onboarding workflow with Czech regulatory packs)
- **Critical-path row 36** (Biometric-badge cross-tenant enrollment to client's access system; identity-handshake without ambient access)
- **Critical-path row 37** (Diacritic-strict mode for Czech + Vietnamese + Slovak + Ukrainian names)

Hyperscaler benchmark: SAP Ariba + Coupa + Workday Strategic Sourcing + Procore + ServiceTitan + Jobber. The unique part of oyatie is that **Cedar policy gates each cross-tenant transition** (bid-submit, award-accept, biometric-badge enroll, payroll-disclosure scope, GDPR data-handling scope) AND that **Czech-specific regulatory packs activate automatically based on tenant residency** (no opt-in toggle — the moment Tomáš's tenant is provisioned with `country_residency_cz`, the OSSZ + ZP + Datová schránka + ČSN-262 packs activate).

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| `story.md` | Beat-by-beat 81-day journey from bid prep through first-shift live | Czech-language dialogue + named places (Skvrňany depot, PolyCraft Plzeň plant, Hygienická stanice Plzeňského kraje), diacritic-strict names (Tomáš Horák, Martina Procházková, Lenka Šimková, Hoàng Văn Long), specific equipment (Kärcher BD 50/50, Tennant T7AMR autoscrubber, Vileda UltraSpeed Pro mop systems), Czech regulatory texture (OSSZ + ZP + Finanční úřad + Datová schránka + Hygienická stanice + ČSN technical standards), ISSA-CIMS cleaning-industry vocabulary, fee numbers in CZK |
| `ux-flow.md` | Tomáš's Lenovo Tab P12 Pro in the depot, his iPhone 13 mini on-site, Procházková's Dell Latitude at PolyCraft, the crew's foreman's Samsung A35 tablet on first day, Datová schránka mobile screens | Czech-language primary UI; diacritic input modal; CZK currency throughout; tax-fields auto-fill from ARES (Czech business register) lookup; Datová schránka cross-link surface |
| `handshake.md` | Per-µservice API across `uklid-horak-sro-plzen-cz` + `polycraft-bohemia-as-plzen-cz` + `cz-ossz-state-tenant` + `cz-zdravotni-pojistovna-vzp` + `cz-financni-urad-tenant` + `cz-datova-schranka` + `crew-foreman-pavel-novak-employee-subtenant` | Each row names source + target tenant, Cedar permit, cross-tenant audit dual-seal class, Czech state systems' interaction shapes |
| `integration-test-plan.md` | Bid lifecycle tests + contract-sign tests + crew hiring tests + onboarding workflow tests + biometric-badge cross-tenant tests + diacritic fidelity tests + Czech state systems integration tests | Each test names seed values + expected event chain + ČSN-262 + GDPR invariant probe pass/fail thresholds |
| `schemas/openapi-marketplace-bid.json` | OpenAPI for bid submit + bid-award + contract-sign + crew-onboard endpoints | Bid structured cost line items + cert attachments + cross-tenant award flow + Czech-specific compliance fields |
| `schemas/cedar-policy.cedar` | Bid + contract + onboarding + biometric-badge + Czech-regulatory Cedar policy | Per-transition Cedar guards + diacritic strict mode + cross-tenant biometric-badge isolation + payroll disclosure scoping |
| `schemas/journey-messages.proto` | proto3 for all RPCs | UTF-8 NFC Czech/Vietnamese/Slovak/Ukrainian names; bid line-item proto; biometric-badge enrollment proto; Czech state systems integration messages |
| `schemas/bid-and-onboard-state-machine.yaml` | 8-state bid-and-onboard lifecycle | `bid_prep → bid_submitted → bid_evaluated → award_received → contract_signed → crew_hiring → crew_onboarding → contract_live`; Cedar guards per transition |
| `schemas/cleaning-bid-line-items-csn-en-13549.json` | Cleaning-specific bid line-item schema (zoned m² + frequency + quality grade per ČSN-EN-13549) | Production-floor m² × frequency × quality-grade structured; supplies + labor + equipment breakouts; CZ-VAT-235/2004 line-level VAT handling |

## The five microservices in scope

| µservice | Role | Critical-path row |
|---|---|---|
| `marketplace` | Lists PolyCraft's bid request; Tomáš's bid submit + competitor bid rank visibility; cross-tenant bid lifecycle | row 33 |
| `workflow-engine` | Drives 8-state bid + onboard lifecycle; per-stage Cedar gates; 115 onboarding tasks materialized | rows 33 + 34 + 35 |
| `payments` | Handles bid-deposit (CZK 12,000 refundable per PolyCraft's bid policy); contract signing fee invoicing; ongoing monthly invoicing prep | row 34 |
| `tenancy` | Provisions employee sub-tenant identities for new hires; Czech OSSZ + ZP integration; cross-tenant biometric-badge enrollment scope | rows 35 + 36 |
| `community` | Czech cleaning-industry peer community participation; advice-seeking + best-practice sharing; isolated from client tenant | row 35 |

## Secondary microservices touched

| µservice | Touch reason |
|---|---|
| `messenger` | MLS-encrypted cross-tenant thread Tomáš ↔ Procházková; internal crew foreman thread; bid-clarification questions during evaluation window |
| `identity` | Tomáš's passkey + diacritic-preserving name; crew passkey enrollment; biometric-badge cross-tenant identity proof |
| `tasks` | 23-step onboarding × 5 crew = 115 tasks; site-walk evidence; safety training completion proofs; supplies ordering tasks |
| `notes` | Site-walk notes + voice-memos (Czech transcripts); crew briefings; CAPA-equivalent safety incident notes |
| `crm` | PolyCraft relationship record; SLA-clock state; renewal probability scoring |
| `contract-lifecycle-management` | 24-month MSA + appendices for Czech-specific clauses + ICCSN-262 acceptance + ICCSN-13549 quality protocol; qualified electronic signature |
| `compliance` | Activates CZ-OZ-262-2006 (Czech labor code), CSN-EN-ISO-9001, CZ-110/2019-Sb (CZ data protection adapting GDPR), EU-GDPR, ISSA-CIMS (cleaning-industry certification), CSN-EN-13549 (cleaning quality), CZ-VAT-235/2004-Sb (VAT law) |
| `audit-chain` | Every bid + contract + onboarding event dual-sealed (Úklid Horák + PolyCraft) |
| `observability` | Captures the bid lifecycle telemetry; onboarding workflow health; biometric-badge enrollment latency |
| `analytics` | Bid win-rate dashboard; onboarding velocity; per-employee training-completion rate |
| `learning-management` | ČSN-262-2006 safety training modules; GDPR data-handling training; PolyCraft-specific induction modules; equipment-specific training (Tennant T7AMR operator cert) |

## Pack overlays

| Pack | Activation reason |
|---|---|
| CZ-OZ-262/2006-zákoník-práce | Czech labor code; all employee-related events activate this pack |
| CZ-110/2019-Sb | Czech data protection law (adapts GDPR for Czech context); GDPR-compatible but with Czech-specific provisions |
| EU-GDPR | Pan-European; PolyCraft's facility-access records contain personal data |
| ISSA-CIMS | International Sanitary Supply Association Cleaning Industry Management Standard; Tomáš's firm is ISSA-CIMS certified |
| CSN-EN-13549 | European cleaning-quality measurement standard; required by PolyCraft's bid spec |
| CSN-EN-ISO-9001 | QMS; required by PolyCraft's spec |
| CSN-EN-407 | Heat + chemical protection gloves rating; OOPP equipment specifications for cleaners handling solvent residues |
| CZ-VAT-235/2004-Sb | Czech VAT law; invoicing format + tax-point timing |
| CZ-ARES | Czech business-register integration; auto-fill VAT ID + business address |
| CZ-Datová-schránka | Czech electronic mailbox for state communications; legally equivalent to registered mail |
| CZ-OSSZ-okresní-správa-sociálního-zabezpečení | Czech social insurance; employee registration |
| CZ-zdravotní-pojišťovna | Czech health insurance; employee registration (typically VZP, OZP, ČPZP, ZPMV, OZP, VoZP, ZP-MA depending on employee choice) |

## Regulatory anchors

1. ADR-0249 multi-category marketplace doctrine (cleaning is one of the canonical service marketplace categories)
2. ADR-0244 tenant scoping primitive
3. ADR-0263 audit dual-seal on cross-tenant transitions
4. ADR-0252 HLC + TrueTime for contract-signing fence
5. Czech zákoník práce (zákon č. 262/2006 Sb.) §38 (employment relationship establishment) + §103 (occupational safety + health) + §234 (training obligations)
6. Czech zákon o ochraně osobních údajů (110/2019 Sb.) §16 (data subject rights) + §27 (cross-border transfer with adequacy)
7. EU-GDPR Articles 6, 7, 28 (data processor obligations for facility-access records)
8. ČSN-EN-13549:2001 (Cleaning services — basic requirements + recommendations for quality measuring systems)
9. ČSN-EN-ISO-9001:2015 §10.2 (QMS nonconformity + corrective action)
10. ISSA-CIMS-2024 (cleaning-industry management standard, latest revision)

## Cell + certification matrix

| Cell | Certification | Journey use |
|---|---|---|
| `eu-prague-primary` | EU-GDPR + ISO 27001 + ISO 9001 + CZ-110/2019-Sb | Primary cell for both Úklid Horák + PolyCraft Bohemia tenants (CZ data residency) |
| `eu-frankfurt-secondary` | EU-GDPR + ISO 27001 | DR replica |
| `eu-vienna-tertiary` | EU-GDPR + ISO 27001 | Read replica for analytics |

## Cedar cross-tenant bid policy (excerpt — full text in `schemas/cedar-policy.cedar`)

```cedar
// Cross-tenant bid submit — Cedar gates on ARES-verified business identity + cert checks
permit (
    principal == User::"tomas.horak@uklid-horak-sro-plzen-cz",
    action == Action::"marketplace.bid_submit",
    resource is BidRequest
) when {
    resource.target_tenant != principal.tenant &&
    principal.has_certification_unexpired("ISSA-CIMS-2024") &&
    principal.has_certification_unexpired("CSN-EN-ISO-9001-2015") &&
    principal.business_register_verified_via_ARES == true &&
    principal.cz_vat_id_active == true &&
    context.bid_window_open == true &&
    context.unicode_normalization == "NFC"
};

// PERMIT — Biometric-badge enrollment to client's access system (scoped, not ambient)
permit (
    principal,
    action == Action::"identity.biometric_badge_enroll_to_client_access_system",
    resource is ClientAccessSystem
) when {
    resource.client_tenant == "polycraft-bohemia-as-plzen-cz" &&
    principal.crew_assigned_to_contract == "contract-uklid-horak-polycraft-2027-01-04" &&
    principal.has_completed_training("csn-262-2006-safety") &&
    principal.has_completed_training("gdpr-data-handling") &&
    principal.has_completed_training("polycraft-induction")
};
```

## Acceptance summary

| AC | Result expected |
|---|---|
| AC-J160-001 | Tomáš reads PolyCraft bid request on Tuesday Oct 14 14:42 CET; bid spec rendered in Czech with full diacritics; audit `EVT-J160-BID-REQUEST-READ-001` sealed |
| AC-J160-002 | Site walk Wed Oct 15 produces 47 photographs + 12 voice-notes; all stored in `uklid-horak-sro-plzen-cz` tenant; audit `EVT-J160-SITE-WALK-002` |
| AC-J160-003 | Bid submitted Wed Oct 15 16:42 CET; bid total CZK 7.94M for 24 months; structured cost breakdown by zone × frequency × ČSN-EN-13549 quality grade; audit `EVT-J160-BID-SUBMITTED-003` dual-sealed |
| AC-J160-004 | Bid evaluated; Procházková's review record dual-sealed; audit `EVT-J160-BID-EVALUATED-004` |
| AC-J160-005 | Award Monday Oct 27 14:00 CET; cross-tenant award notification via `messenger`; audit `EVT-J160-AWARD-RECEIVED-005` dual-sealed |
| AC-J160-006 | Contract signed Friday Nov 7 11:18 CET via dual-tenant qualified electronic signature; audit `EVT-J160-CONTRACT-SIGNED-006` dual-sealed under TrueTime fence |
| AC-J160-007 | 3 new hires onboarded by Friday Dec 19 (3-week onboarding); 2 internal rotations confirmed; all 5 crew complete ČSN-262 safety + GDPR + PolyCraft induction training; audit `EVT-J160-CREW-ONBOARDED-008` |
| AC-J160-008 | Biometric badges enrolled to PolyCraft's access system for all 5 crew by Sat Jan 3 2027 18:42 CET; audit `EVT-J160-BIOMETRIC-BADGES-ENROLLED-009` dual-sealed |
| AC-J160-009 | Diacritic fidelity: "Tomáš Horák", "Martina Procházková", "Lenka Šimková", "Hoàng Văn Long", "Іван Шевченко", "Mária Kováčová", "Pavel Novák" preserve UTF-8 NFC across all persisted fields |
| AC-J160-010 | Czech state-system integrations: ARES lookup auto-fills tenant; OSSZ employee registration for 3 new hires; ZP (health insurance) enrollment; Datová schránka receives contract notification |
| AC-J160-011 | Contract goes live Mon Jan 4 2027 06:00 CET; first shift completes 06:00–14:00; audit `EVT-J160-CONTRACT-LIVE-007` and `EVT-J160-FIRST-SHIFT-COMPLETE-010` dual-sealed |
| AC-J160-012 | Cross-tenant audit dual-seal invariant: every cross-tenant transition (bid submit, award accept, contract sign, biometric badge enroll, first-shift complete) dual-seals in both Úklid Horák + PolyCraft tenants |
| AC-J160-013 | Community participation: Tomáš posts 4 questions to the Czech cleaning-industry peer community during the journey; isolated from PolyCraft client tenant |

## Cross-references

- Persona dossier: `docs/personas/blue-collar-owner-operator-tomas-horak.md`
- MASTER-ROSTER §5.1 row 104
- Matrix §12 j160 recommendation
- Related: j109 (construction-co cross-tenant freelance specialist), j112 (tenant-to-tenant RFQ + bid), j115 (SaaS-vendor-API-multi-tenant), j151 (typhoon-evac small-biz)
- Pack roster: `packs/cz-oz-262-2006/`, `packs/cz-110-2019-sb/`, `packs/eu-gdpr/`, `packs/issa-cims/`, `packs/csn-en-13549/`, `packs/csn-en-iso-9001/`, `packs/cz-vat-235-2004-sb/`, `packs/cz-ares/`, `packs/cz-datova-schranka/`
- ADR-0249 marketplace doctrine
- ADR-0244 tenant scoping
- ADR-0263 audit dual-seal
- ADR-0311 dual-tenant boundary (Tomáš's personal vs business tenants; out of primary scope but invariant holds)

## Stop condition

This journey is complete when all 13 acceptance criteria pass on the seeded two-tenant fixture (plus the Czech state-system mocks), the bid lifecycle reaches `contract_live`, the crew of 5 has full biometric-badge access to PolyCraft's facility, all Czech regulatory pack activations attest correctly, the diacritic preservation invariant holds across Czech + Vietnamese + Slovak + Ukrainian names, and the first shift completes without incident on Mon Jan 4 2027.
