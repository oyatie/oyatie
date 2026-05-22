---
doc_class: User-Journey-Story
journey_id: j160-cleaning-co-tomas-horak-bid-cross-tenant-and-onboard
date: 2026-05-20
authority_tier: 2
status: draft
---

# j160 — Story: 14:42 CET in Plzeň-Skvrňany, a bid lands in the depot

## §0 — Tuesday October 14, 2026, 14:42 CET — Skvrňanská třída depot, Plzeň-Skvrňany

The depot of **Úklid Horák s.r.o.** sits on a quiet stretch of Skvrňanská třída, between a tire-fitting garage and a long-defunct printing works that's been converted to artist studios. It is a single-story warehouse with a 240 m² floor: 180 m² for cleaning-supply pallets + 4 industrial-grade machines (a Kärcher BD 50/50 scrubber, a Tennant T7AMR autoscrubber rented from a Brno equipment partner, two Vileda UltraSpeed Pro mop systems on rolling carts), and 60 m² partitioned at the back as Tomáš's office. The office has a single desk, a tatty leather chair, two visitor chairs, a Czech-flag pennant from his time as a junior soldier in 2004, a 1:24 scale model Tatra 815 in the window, and a Lenovo Tab P12 Pro on a slim aluminum stand. Outside the day is grey, 11 °C, light drizzle, the kind of Czech autumn that comes in October before the first frost.

Tomáš Horák, 41, navy-blue work polo, jeans, work boots, hair cropped short and going grey at the temples, looks up from his coffee — Lavazza Crema e Gusto, his fourth of the day — at the new alert on the tablet. He has been expecting this; PolyCraft Bohemia's procurement manager Ing. Martina Procházková mentioned in their casual conversation at the regional ICCSN industry day in September that the firm's Plzeň cleaning contract would go out to bid in October. The current incumbent is **ČistoCentrum Praha s.r.o.**, a 95-person Prague firm that won the contract in 2020 and has held it through one renewal. Their service has been adequate but not enthusiastic — Procházková hinted that PolyCraft's plant manager is unhappy with the production-floor solvent-residue protocol.

The tablet's lock screen requires Tomáš's passkey + face_id (PIN fallback for gloved hands). Tomáš taps. The tenant chip at the top reads:

> **🏢 uklid-horak-sro-plzen-cz · firma · 1 tenant active**

He opens the `marketplace` µservice. The bid request from PolyCraft is at the top of his Inbox:

```
Otevřená výzva k podání nabídky · open call for bid
PolyCraft Bohemia a.s. · IČ 47714232 · DIČ CZ47714232
Plzeňský závod · facilities cleaning service contract
24 měsíců · 2027-01-04 → 2028-12-31
plocha celkem 12 400 m² (výrobní 9 200 + sklad 1 800 + administ. 980 + společné 420)
maximální cena bez DPH: CZK 4,2M ročně (CZK 8,4M za 24 měsíců)
specifikace: ČSN-EN-13549 quality grade 4 minimum, ISO-9001 certifikát požadován
termín pro podání nabídek: pátek 17. 10. 2026 17:00 CET
```

The text is fully in Czech, diacritics correct — including his own firm name "Úklid Horák" elsewhere in the system spelled with the proper "Ú" + "á". He nods. The bid is structured: 14 sections, structured-form fields for cost line items, file attachments for cert + insurance + crew composition.

He reads the spec for 20 minutes. Then he calls Procházková on his iPhone 13 mini.

**Tomáš 15:08 CET** (Czech): "Dobrý den paní inženýrko Procházková, tady Tomáš Horák z Úklidu Horák. Viděl jsem vaši výzvu — díky za informaci v září. Mohli bychom udělat prohlídku závodu před tím, než podáme nabídku? Klidně i tento týden, jak vám to vyhovuje."

**Procházková 15:08 CET**: "Dobrý den pane Horáku. Ano, určitě — domluvme to. Zítra ráno v 8:00 mám čas. Vyhovovalo by vám to? Ukázal bych vám výrobu, sklad, kantýnu, šatny — vše co je v zadání."

**Tomáš 15:09 CET**: "Skvělé, zítra v 8:00 jsem u brány. Děkuju moc."

He hangs up. The site walk is set.

He opens the `workflow-engine` µservice and instantiates the built-in bid-preparation workflow `wkfl-marketplace-bid-prep-v3-cleaning`. The workflow materializes 22 atomic tasks via `tasks` µservice — from "site-walk recon" (task 1) through "final bid review by independent reviewer" (task 22). He marks task 1 as "scheduled tomorrow 08:00".

Then he writes a quick voice-note via the `notes` µservice (Czech voice input, NFC preservation):

> "Příští bod: ČistoCentrum drží tuhle smlouvu od roku 2020. Pochopil jsem od Martiny, že provozář PolyCraftu není spokojený s tím, jak řeší zbytkové rozpouštědlo na výrobní hale. Pokud máme vyhrát, musíme nabídnout konkrétně lepší protokol pro solvent residue — možná Tennant T7AMR plus speciální detergent + neutralizace. Ne ČistoCentrum-light. Lepší."

The note auto-saves at 15:42 CET. `EVT-J160-BID-REQUEST-READ-001` seals in `uklid-horak-sro-plzen-cz` at 15:43 CET.

## §1 — Wednesday October 15, 2026, 08:00–14:18 CET — PolyCraft Bohemia Plzeň plant

The PolyCraft plant occupies a 14-hectare site on the southern edge of Plzeň, near the U Borských polí business park. Tomáš drives his white VW Caddy company van (registration 4P3 9421) through the security gate at 07:54 CET. The guard checks his name against the visitor list, hands him a paper visitor badge, and directs him to administrative reception.

Procházková meets him in the lobby at 08:02 CET. She is 38, dark hair in a low bun, navy suit, sensible flat shoes, the kind of corporate-procurement professional who knows the difference between a cleaning company's marketing pitch and its actual operating capacity. Her oyatie tenant tag reads `polycraft-bohemia-as-plzen-cz`; her email is `m.prochazkova@polycraft-bohemia-as-plzen-cz`.

**Procházková 08:03 CET**: "Pane Horáku, vítejte. Pojďme rovnou — máme šest hodin, projdeme celý závod. Postupujeme od administrativy přes kantýnu, šatny, sklad, výrobu — výroba je nejdelší část."

**Tomáš 08:03 CET**: "Děkuji. Mohu fotit a nahrávat poznámky?"

**Procházková 08:04 CET**: "Ano, fotky ano. Žádné konkrétní výrobky ale prosím — recepturní citlivost. Já vás upozorním kde to bude problém."

They walk. Tomáš opens his Lenovo Tab P12 Pro and uses the `tasks` µservice's site-walk evidence-capture flow. He photographs:

- **Admin offices** (980 m², 4 floors): standard open-plan, ~140 desks total, kitchenettes on each floor, two glass-walled meeting rooms per floor, executive offices on the 4th floor. Cleaning frequency need: 5 days/week, 1 deep clean/month, including window-cleaning quarterly. *photos: 8.*
- **Canteen** (180 m² of 420 m² common area): seating for 80, full hot-line, grease trap visible behind a service door — Procházková notes the grease trap is currently emptied by the food-service vendor monthly but maintenance of the floor + walls + ceiling around it is the cleaning contractor's responsibility. *photos: 6, including a close-up of the floor at the grease-trap drain showing chronic discolouration.*
- **Lockers + showers** (180 m²): 240 lockers, 16 shower stalls. Procházková confirms what she hinted at: the chronic odor problem is concentrated in the ventilation grilles' wall section — current contractor cleans the lockers but not the wall section behind them. *photos: 4, including ventilation grille close-ups.*
- **Restrooms** (60 m² × 8 sets across the site = 480 m² total, included in common areas + zones): standard fixtures, regular cleaning needed. *photos: 6 across two representative sets.*
- **Warehouse** (1,800 m²): pallet-rack racking 3 levels high, two RTC dock doors, forklift traffic Mon-Fri. Cleaning need: sweep + scrub-machine pass nightly; spill response on demand. *photos: 4.*
- **Production floor** (9,200 m², the bulk of the contract): this is what the bid is really about. Procházková walks Tomáš through 7 production zones. The polymer-additives process produces chemical residues — primarily solvent-based plasticizers (DEHP-substitutes in PolyCraft's current product mix), with episodic spills of small quantities. Floor coating is industrial epoxy; the current cleaner is using a general-purpose alkaline degreaser which is not optimal for solvent residue (alkaline degreasers don't penetrate or neutralize plasticizer residue effectively). Procházková says the current contractor's protocol leaves a thin film that the plant manager Ing. Vavřinec Daneš has complained about for 18 months. *photos: 11, including close-ups of the floor in 4 different zones with visible residue patterns.* Voice-note from Tomáš (Czech): "Tady je ten problém. Použít neutralizér + low-residue surfactant + Tennant T7AMR s mikrofiber padem. Ne ten alkalický degreaser."

The walk ends at 13:48 CET. They have lunch at the canteen (Tomáš has a goulash + bread + a small beer; Procházková a salad + Mattoni sparkling water).

At 14:18 CET they exchange business cards (paper, traditional — both Procházková and Tomáš are old-school enough to do this) and part. Tomáš drives back to the Skvrňany depot.

`EVT-J160-SITE-WALK-002` seals at 14:42 CET with 47 photographs + 12 voice-notes attached.

## §2 — Wednesday October 15, 16:00–19:42 CET — bid drafting in the depot

Back at the depot, Tomáš opens the `marketplace` µservice's bid-draft form for the PolyCraft RFP. The structured-form has 14 sections; he fills them in over 3.5 hours.

The most important section is the **structured cost breakdown by zone × frequency × ČSN-EN-13549 quality grade**. He uses the schema `cleaning-bid-line-items-csn-en-13549.json` to structure his cost lines:

| Zone | m² | Frequency | ČSN-EN-13549 grade | Monthly labor cost (CZK) | Monthly supplies (CZK) | Monthly equipment-amortization (CZK) | Monthly total (CZK) |
|---|---|---|---|---|---|---|---|
| Production floor zone 1 (extrusion) | 1,400 | nightly + monthly deep | grade 4 | 38,200 | 6,400 | 4,800 | 49,400 |
| Production floor zone 2 (mixing) | 1,800 | nightly + weekly deep | grade 4 | 47,600 | 9,800 | 4,800 | 62,200 |
| Production floor zone 3 (compounding) | 2,200 | nightly + biweekly deep | grade 4 | 51,400 | 10,200 | 4,800 | 66,400 |
| Production floor zones 4–7 | 3,800 | nightly + monthly | grade 4 | 67,800 | 12,400 | 4,800 | 85,000 |
| Warehouse | 1,800 | nightly | grade 3 | 14,200 | 3,600 | 1,200 | 19,000 |
| Admin offices | 980 | 5 days/wk | grade 4 | 18,400 | 4,200 | — | 22,600 |
| Canteen | 180 | 5 days/wk + monthly deep including grease-zone | grade 4 (food-handling-adjacent) | 7,800 | 2,400 | — | 10,200 |
| Lockers + showers | 180 | nightly + monthly ventilation-grille deep | grade 4 | 5,400 | 1,800 | — | 7,200 |
| Restrooms ×8 sets | 480 | 5 days/wk × 8 | grade 4 | 8,400 | 2,400 | — | 10,800 |
| **Total monthly** | 12,820 m² (mismatched; 12,400 spec) | mixed | grade 4 majority | **259,200** | **53,200** | **20,400** | **332,800** |

(The 12,820 m² total in his sum is 420 m² over the spec 12,400 — he reconciles this by treating the 420 m² "common areas" as the lockers + showers + restrooms total which double-counts; the final reconciled per-month cost is **CZK 331,000 round** including a small reserve for spill-response and a +3.1% YoY escalation clause for year 2 to track Czech CPI.)

The 24-month total: CZK 331,000 × 24 = **CZK 7,944,000** (gross of VAT 21%). He submits this as **CZK 7,940,000** (round) → CZK 9,607,400 with VAT.

He attaches:

- ISO-9001 cert (PDF, scanned, dated 2025-04-18, valid through 2028-04-18, issued by TÜV SÜD Czech)
- ISSA-CIMS-2024 cert (PDF, dated 2024-11-12, valid through 2027-11-12)
- Czech VAT registration (auto-pulled from ARES via the ARES integration; IČ 27488123, DIČ CZ27488123)
- Public-liability insurance certificate (Generali Pojišťovna, CZK 50M coverage)
- Crew composition plan: 5 dedicated FTE (1 foreman + 4 cleaners), with named existing employees + 3 planned new hires
- ČSN-EN-13549 quality measurement protocol (PDF, his firm's own template)
- Reference letters: 2 existing clients (Plzeňský Prazdroj brewery's R&D lab cleaning contract since 2022; Škoda Auto Plzeň component-warehouse cleaning since 2021)

He drafts a 2-page narrative cover letter in Czech describing his firm's approach, with specific mention of the solvent-residue protocol improvement using Tennant T7AMR + neutralizer + low-residue surfactant.

He submits at 16:42 CET Wednesday Oct 15 via `marketplace.bid_submit`. Cedar evaluates in 84 ms:

- Principal: `tomas.horak@uklid-horak-sro-plzen-cz`
- Action: `marketplace.bid_submit`
- Resource: `BidRequest::"bid-polycraft-plzen-cleaning-2027-01-04"`
- Context: `principal.has_certification_unexpired("ISSA-CIMS-2024") == true`, `principal.has_certification_unexpired("CSN-EN-ISO-9001-2015") == true`, `principal.business_register_verified_via_ARES == true`, `principal.cz_vat_id_active == true`, `bid_window_open == true`, `unicode_normalization == "NFC"`

Permit. `EVT-J160-BID-SUBMITTED-003` dual-seals in `uklid-horak-sro-plzen-cz` AND `polycraft-bohemia-as-plzen-cz` at 16:42:38 CET.

Tomáš stretches. He drinks his fifth coffee of the day. He drives home.

## §3 — Friday Oct 17 17:00 CET — bid window closes

By Friday 17:00 the bid window closes. 5 bidders submitted:

- ČistoCentrum Praha s.r.o. (incumbent) — Tomáš sees their rank slot as "Bidder A"
- Úklid Horák s.r.o. — "Bidder B" (his own)
- AB Facility s.r.o. (Prague) — "Bidder C"
- Šeba Service s.r.o. (Brno) — "Bidder D"
- Zenova SK Bohemia s.r.o. (Plzeň regional, smaller) — "Bidder E"

The marketplace UI shows him only anonymized rank pricing (so he can see the spread but not specific bidders): the 5 bid totals range from CZK 7.4M (low) to CZK 9.2M (high). His CZK 7.94M is in the lower-middle band. Procházková enters the review window for 10 days.

## §4 — Monday Oct 27 14:00 CET — award

Tomáš is mid-conversation with his foreman Pavel Novák in the depot when his iPhone chimes. The chime is the oyatie `messenger` urgent-priority sound, distinct from email. He pulls the phone.

```
🏆 Award notification
PolyCraft Bohemia a.s. → Úklid Horák s.r.o.
Contract: facilities cleaning Plzeň plant 2027-01-04 → 2028-12-31
Bid amount: CZK 7,940,000 (excl. VAT)
Award decision: accepted
Procházková (Ing.) Martina · 14:00:12 CET
```

Pavel sees Tomáš's face change. Tomáš says, in Czech, "Pavle, vyhráli jsme PolyCraft." Pavel — who has been at the firm since 2018 — sets down his coffee mug and shakes Tomáš's hand. They stand in the depot for a moment, not saying anything more.

`EVT-J160-AWARD-RECEIVED-005` dual-seals at 14:00:18 CET in both tenants. The `workflow-engine` advances the bid-and-onboard state machine from `bid_evaluated` → `award_received`.

Within the next hour Tomáš and Procházková exchange brief congratulatory messages via `messenger`. They agree to start contract negotiation Tuesday Oct 28 09:00 CET.

## §5 — Tuesday Oct 28 – Friday Nov 7 — contract negotiation

Over 11 days they exchange 23 messages + 4 drafts of the contract. Key points negotiated:

1. **Weekend cover SLA**: PolyCraft's production runs 5 days/week (Mon–Fri 06:00–22:00, two shifts), so weekends are administrative. Original spec called for Saturday-morning emergency response within 2 hours. Tomáš negotiates this to "Saturday-morning emergency response within 4 hours; same-day Sunday response within 8 hours unless declared emergency."
2. **Holiday cover**: PolyCraft observes Czech state holidays (Nový rok, Velký pátek, Velikonoční pondělí, Svátek práce, Den vítězství, Den slovanských věrozvěstů Cyrila a Metoděje, Den upálení mistra Jana Husa, Den české státnosti, Den vzniku samostatného československého státu, Den boje za svobodu a demokracii, Štědrý den + 25.+26. prosince — 13 days). Tomáš negotiates a holiday-pay surcharge of +75% (Czech labor code §115 minimum; he gets +75%).
3. **Solvent-residue protocol**: written into Appendix B with specific equipment (Tennant T7AMR) + specific neutralizer chemistry (a glycol-ether-based product from Diversey EU). Tomáš commits to a 12-week ramp-up program with monthly ČSN-EN-13549 quality measurements; if grade 4 is not achieved by month 6, PolyCraft can claw back 15% of monthly fees pending improvement.
4. **GDPR + CZ-110/2019 data-handling**: Tomáš's crew will have access to PolyCraft's facility-access system (biometric badges) which contains PolyCraft employees' personal data. Appendix C names the data processor obligations, the cross-tenant data flow, the audit-chain retention (7 years per CZ-Civil-Code), and the deletion protocol at contract end.
5. **Liability cap**: CZK 12M per incident (cap roughly = 18 months' contract value); insurance bumped to CZK 100M aggregate (Tomáš's Generali Pojišťovna agreed to upgrade).
6. **Renewal clause**: After 24 months, automatic 12-month renewal unless either party gives 90-day notice.

Contract signed Friday Nov 7 11:18 CET via dual-tenant qualified electronic signature (eIDAS-compliant QES through I.CA — Czech-state-recognized CA — for Tomáš, and SecuSign QES for Procházková). The signing is mediated by `contract-lifecycle-management` µservice with a TrueTime fence (uncertainty ≤ 10 ms) per ADR-0252.

`EVT-J160-CONTRACT-SIGNED-006` dual-seals at 11:18:42 CET under TrueTime fence.

Datová schránka (Czech state electronic mailbox) receives a notification of the contract via the legally-mandated archival path. Tomáš's IČ 27488123 is on file with Datová schránka; the notification carries the contract hash + the dual-tenant attestation; Czech state record-keeping is now in possession of the metadata.

## §6 — Monday Nov 10 09:00 CET — crew hiring opens

Of Tomáš's 14 existing employees:

- 5 are assigned to the Plzeňský Prazdroj brewery contract (5 days/week, 22:00–04:00 nightshift)
- 3 are assigned to the Škoda Auto Plzeň warehouse contract (5 days/week, 18:00–02:00)
- 4 are assigned to a portfolio of small office contracts (5 days/week, 17:00–22:00)
- 2 are reserves (1 maternity-cover, 1 sick-pool)

For PolyCraft he needs 5 dedicated FTE. He can rotate 2 reserves; he needs to hire 3.

The 3 new positions:

1. **Foreman / team-lead** — full-time, CZK 38,000/month gross, 5y experience, ISSA-CIMS Level 2 cert preferred, Czech-native preferred, German B1 helpful
2. **Cleaner** — full-time, CZK 28,500/month gross, willingness to work shifts including occasional weekends, will be ČSN-262-2006 trained on company time
3. **Cleaner** — same as above

He posts the 3 roles via `marketplace.labor-pool` (oyatie's labor-pool subflow that publishes the openings to the regional cleaning-industry candidate pool). The labor-pool integrates with Czech state employment systems (Úřad práce). Applications come in over 2 weeks.

Selected hires (by Wednesday Nov 26):

- **Pavel Novák** — Tomáš's existing foreman at Plzeňský Prazdroj — promoted to PolyCraft team-lead role (internal rotation #1; he wanted the daytime hours for family reasons). 42 years old, 8 years at the firm.
- **Lenka Šimková** — internal rotation #2 from the office-contracts portfolio. 36 years old, 5 years at the firm. Diacritic-strict (Šimková).
- **Hoàng Văn Long** — new hire #1. 34 years old, Vietnamese, lives in Plzeň for 11 years, married to a Czech woman, has Czech permanent residence, B2 Czech, will need ČSN-262-2006 training. The system handles the Vietnamese diacritics + tone marks (Hoàng Văn Long has 4 distinct diacritic+tone glyphs that must be preserved across all persisted fields).
- **Mária Kováčová** — new hire #2. 39 years old, Slovak, EU citizen, lives in Plzeň since 2018, A2 German (helpful for occasional cross-border training with Bavarian sister-firm). Slovak diacritic-strict.
- **Іван Шевченко** (Ivan Shevchenko) — new hire #3. 31 years old, Ukrainian, came as refugee in 2022, has temporary protection status, B1 Czech, will need additional GDPR + CZ-110/2019 training because his work-permit status interacts with facility-access records. Cyrillic + Latin transliteration both stored; UI shows whichever the user prefers.

`EVT-J160-CREW-SELECTED-008-prep` seals Wed Nov 26 11:42 CET.

## §7 — Wed Nov 27 – Fri Dec 19 — onboarding

The `workflow-engine` materializes 23 atomic onboarding tasks × 3 new hires = 69 tasks (the 2 internal rotations skip the hiring-paperwork tasks but do all the PolyCraft-specific training).

Key beats:

- **Wed Nov 27**: Employment contracts signed via Czech-state-recognized QES. OSSZ (Okresní správa sociálního zabezpečení) electronic registration for the 3 new hires. ZP (Zdravotní pojišťovna; each new hire selects: Hoàng Văn Long → VZP, Mária Kováčová → OZP, Іван Шевченко → ZPMV) electronic registration.
- **Mon Dec 1 – Fri Dec 5**: ČSN-262-2006 occupational-safety training (Czech labor code §103 requires this BEFORE work begins). Delivered via `learning-management` µservice with Tomáš's firm-specific module + a third-party general-safety module from BOZP-info.cz. Each new hire completes 8 hours of training; passes assessment. Tomáš himself participates as trainer for the firm-specific portion.
- **Mon Dec 8 – Wed Dec 10**: GDPR + CZ-110/2019-Sb data-handling training. 6 hours. PolyCraft-context-aware (covers what crew may see in facility-access records and what they may not).
- **Thu Dec 11 – Fri Dec 12**: Tennant T7AMR equipment-specific training. Tennant Czech sends a trainer to the Skvrňany depot. Pavel + Lenka + Mária get certified operators; the others get trained-spotter level.
- **Mon Dec 15 – Wed Dec 17**: PolyCraft-specific induction. Procházková schedules a 3-day session at the PolyCraft plant. Day 1: facility walk-through + zone-specific protocols. Day 2: emergency procedures (chemical spill response, fire evacuation, lockout-tagout for the production machinery). Day 3: biometric badge enrollment (each crew member's biometric template is enrolled to PolyCraft's access system via cross-tenant identity handshake).
- **Thu Dec 18 – Fri Dec 19**: First supervised shift (Tomáš + Pavel + 1 PolyCraft maintenance lead on-site). Each crew member walks their assigned zone with the supervisor.

`EVT-J160-CREW-ONBOARDED-008` seals Friday Dec 19 18:42 CET. All 5 crew complete training. All 5 biometric badges enrolled and active in PolyCraft's access system. `EVT-J160-BIOMETRIC-BADGES-ENROLLED-009` dual-seals.

## §8 — Sat Jan 3, 2027, 23:42 CET — final readiness

Christmas + New Year break passes. Tomáš sleeps in until 09:00 on Jan 1 (the first time in years). On Saturday Jan 3 at 23:42 CET he runs the final readiness check via the `workflow-engine`. All 5 crew confirmed. All equipment confirmed loaded at the depot. All supplies inventoried. Pavel will pick up the van at 04:42 Monday morning and drive Lenka + Hoàng + Mária + Іван to the PolyCraft gate by 05:42.

Tomáš goes to bed at 00:18 Sunday Jan 4. He sets his alarm for 04:18.

## §9 — Mon Jan 4, 2027, 06:00 CET — contract live, first shift

At 05:42 CET Pavel's van pulls up to the PolyCraft Plzeň plant security gate. The guard checks each crew member's biometric badge. All 5 scan green. The crew enters at 05:48. They change in the cleaner-locker area (PolyCraft provides), pick up Tomáš's pre-positioned Tennant T7AMR + Vileda UltraSpeed Pro carts + Diversey neutralizer cartridges from the cleaner-storage room (which PolyCraft provisioned per the contract appendix), and walk to their assigned zones.

At 06:00:18 CET Pavel scans his badge at the production-floor zone-1 entry. The first cleaning cycle begins. Tomáš is on site too (he drives down at 05:30 to be there for the first shift); he walks the production floor with Pavel for the first hour, then steps back. Procházková arrives at 07:18 (her normal start time) and walks the canteen + admin areas; she meets Tomáš at 08:42 in the canteen for coffee.

**Procházková 08:43 CET**: "Tomáši, jste tady — to je hezké. První dojem dobrý."

**Tomáš 08:43 CET**: "Děkuju Martino. Doufám že tak zůstane."

The first shift ends at 14:00 CET. Production-floor zone-1 visual check at end-of-shift: clean. Zone-2: clean. Zone-3 (compounding): the residue-protocol shows visible improvement on the Tennant T7AMR's mikrofiber pad uptake (Pavel saves the pad as evidence). Pavel emails the daily-shift report to Tomáš at 14:18.

`EVT-J160-CONTRACT-LIVE-007` and `EVT-J160-FIRST-SHIFT-COMPLETE-010` dual-seal at 14:18:42 CET. The 24-month contract is in execution.

## §10 — Beats not on the wire (the human texture)

- On Wednesday Nov 26 evening, after Tomáš sent the offer letters to Hoàng, Mária, and Іван, his wife Petra (a teacher at ZŠ Plzeň-Skvrňany, 38 years old, doesn't usually ask about work) asked at dinner: "Tomáši, vyhrál jsi PolyCraft, ano?" He nodded. She said: "Tak nemusíš se bát o příští dva roky." (So you don't have to worry about the next two years.) Their daughter Anna (age 11) was eating buchty with poppy-seed filling and didn't look up.
- Tomáš's father — Jaromír Horák, 71, retired auto-mechanic in Klatovy — called him on Saturday Dec 21 after he saw a casual mention of "PolyCraft" in a Plzeň-region newspaper. The old man asked, in his thick Klatovy-region Czech: "Synu, není to ten chemický závod?" (Son, isn't that the chemical plant?) Tomáš said yes, it's the chemical plant. His father said: "Tak ať tvoji lidé dávají pozor." (So your people should be careful.) Tomáš said yes, they'll be careful. His father said okay and hung up.
- Lenka Šimková — the internal rotation from the office-contracts portfolio — is 36, divorced, one son aged 9. She accepted the PolyCraft rotation specifically because the 06:00–14:00 shift means she can be home when her son is home from school. She told Tomáš this during the onboarding. Tomáš noted it. He arranged her zone assignments to be ones with predictable end-times.
- Hoàng Văn Long — the Vietnamese new hire — has lived in Plzeň for 11 years. His Czech is fluent enough that the ČSN-262 training was no problem. But the diacritic preservation of his name matters to him: an earlier employer (a different cleaning firm, 2018–2022) had stored his name as "Hoang Van Long" without the tone marks; he had to spell it out at every state-system interaction. With oyatie's NFC preservation, his name renders correctly on his employment contract, his OSSZ registration, his ZP card, his ID badge, his payslip — all without him having to ask. He told Pavel about this on day 3 of onboarding; Pavel told Tomáš; Tomáš mentioned it in his community post that evening.
- The Czech cleaning-industry peer community (`cz-cleaning-industry-owner-operators-community`) — Tomáš has been a member since 2022 — has 184 members across the Czech Republic. He posted 4 questions during this journey: (1) "Anyone have a glycol-ether neutralizer source besides Diversey?" — 6 replies, 1 useful (a Brno firm pointed him to Czech distributor Tatrachema); (2) "What's your standard biometric-badge enrollment protocol for client access systems? Is the Cedar-scoped pattern working for you?" — 4 replies; (3) "Hiring a Ukrainian refugee with temporary protection — any GDPR special-category-of-data nuances I should worry about?" — 11 replies (including a former data-protection-officer who used to be at Datový úřad); (4) "Anybody else have the Tennant T7AMR's mikrofiber pad uptake problem on epoxy floors?" — 3 replies. The community is private to the cleaning-industry-owner-operators tenant; PolyCraft has zero visibility.

## §11 — Stop condition for this story

This story documents the lived texture of the 81-day journey from bid prep through first-shift live. The acceptance criteria in `README.md`, the API shapes in `handshake.md`, the test cases in `integration-test-plan.md`, and the schema files together encode the machine semantics. The story exists so that the next human or agent reading the codepath can understand WHY the cross-tenant marketplace bid carries structured cost line items with ČSN-EN-13549 quality grades, WHY the contract-signing TrueTime fence matters for QES dual-tenant signature attestation, WHY Czech-state-system integration (ARES + OSSZ + ZP + Datová schránka) is a first-class concern not a bolt-on, WHY the biometric-badge cross-tenant enrollment is Cedar-scoped rather than ambient, and WHY the diacritic preservation across Czech + Vietnamese + Slovak + Ukrainian names is a non-negotiable invariant for any system that wants to serve the actual European blue-collar small-business owner-operator.
