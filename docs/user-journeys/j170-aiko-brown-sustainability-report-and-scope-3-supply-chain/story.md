---
doc_class: User-Journey-Story
journey_id: j170-aiko-brown-sustainability-report-and-scope-3-supply-chain
date: 2026-05-20
authority_tier: 2
status: draft
---

# j170 — Story: 07:42 EDT in the Halle Building, the emissions report begins

## §0 — Monday September 14, 2026, 07:42 EDT — MFI Cleveland HQ, Halle Building floor 8

The Halle Building at 1228 Euclid Avenue is a 1910 Beaux-Arts retail palace turned office building, in downtown Cleveland's Playhouse Square district. Marlboro-Forge Industries took the top two floors (8 + 9) as its corporate HQ in 1997 when it moved out of the original 1907 forge-shop building in the Flats district. The 8th floor sustainability + ESG team's office sits in the southwest corner, looking out toward the Terminal Tower + the steel-and-glass Public Square renovation. Today is **Monday September 14, 2026** at 07:42 EDT. Cleveland weather: cool morning, 14 °C, light overcast, the start of fall. Aiko Brown is at her desk wearing a navy mock-neck sweater + grey wool trousers + leather Chelsea boots (Cleveland office is business-casual; sustainability team trends a bit dressed-down). She has a Phoenix Coffee Roasters dark-roast pour-over in a Hydroflask + her **Surface Laptop 7 (15-inch, sage-green, 32 GB RAM, 1 TB SSD)** open on her standing desk, plus a second monitor (a Dell U2722DE) displaying the workflow dashboard.

Her tenant chip at the top of the Surface reads:

> **🏢 marlboro-forge-industries-inc-cleveland-oh-us · publicly-listed-corp · 1 tenant active**

She opens the FY2025 emissions-report archive first (the previous year's report; filed March 28, 2026). She reviews the Ernst & Young assurance partner's "matters for management" letter — 6 immaterial observations from last year's review:

1. Scope-3 Category-1 spend-based-estimate for textiles supplier was a 14% overestimate; switch to activity-data for FY2026
2. Sherbrooke QC plant's gas-meter calibration interval missed the Q3-2025 schedule by 18 days (no material effect; corrected); add to FY2026 control
3. Monterrey Apodaca plant's electricity purchase under PPA with CFE's Solar-A program — confirm market-based emission factor with PPA contract effective date
4. Norsk Hydro Sherbrooke aluminum activity data missing 1 month (May 2025); reach out earlier in FY2026 cycle
5. EPA Supply-Chain GHG Emission Factors database version mismatch — used v1.2 but v1.3 released Aug 2025; upgrade for FY2026
6. ESRS-E1 disclosure E1-6.71 (financed emissions for any portfolio investments) — confirm MFI's pension plan portfolio is in/out of scope per latest interpretive guidance

Aiko nods. She marks each as "carried-forward-to-FY2026" in the workflow. Her direct report **Joshua Park** (28, joined the team in 2024 from a sustainability-consulting role at Anthesis Boston) is in the office across the hall; he comes in at 07:58.

**Joshua 07:58 EDT** (English, Boston-area accent): "Morning Aiko. Saw the FY2026 workflow kickoff calendar invite. You want me to handle the utility-bill structured-extract batch this year? I've been working with Hadrian on the Q3 invoices."

**Aiko 07:59 EDT**: "Yes. Take Scope-2 ingest end-to-end. I'll do Scope-1 + own the Scope-3 supplier outreach. We have **Maya Iyengar** starting next Monday — she'll learn by shadowing the assurance review prep in February."

**Joshua 08:00 EDT**: "Got it. Anita asked about the SBTi re-validation deadline — is that hard March 31 or do we have slack?"

**Aiko 08:01 EDT**: "SBTi gave us a one-quarter buffer to June 30, but Anita wants it bundled with the March filings. We'll target March."

Joshua nods and goes to his desk.

Aiko opens the `compliance` µservice's emissions-report workflow editor. She instantiates the workflow `wkfl-emissions-report-fy2026-mfi`. The workflow materializes **247 atomic tasks** across the 4 phases:

- **Phase 1 — Scope-1+2 ingest** (85 tasks; runs Sep 14 - Oct 31)
- **Phase 2 — Scope-3 outreach** (62 tasks; runs Nov 2 - Dec 18)
- **Phase 3 — Scope-3 ingest + reconciliation** (54 tasks; runs Dec 21 - Feb 26)
- **Phase 4 — Multi-framework composition + assurance + filing** (46 tasks; runs Mar 1 - Mar 31)

`EVT-J170-WORKFLOW-INIT-001` seals at 10:18:18 EDT.

## §1 — Mon Sep 14 – Fri Oct 31 — Scope-1 + Scope-2 ingest

Aiko spends the next 7 weeks coordinating with each of MFI's 7 plant environmental-engineering leads:

- **Cleveland HQ + Riverside facility** (the company's small co-located forge): **Bill Sokolich**, 56, plant environmental engineer since 1998
- **Akron Coventry Works**: **Marcia Walters**, 47, joined 2019 from BridgestoneFirestone Akron
- **Pittsburgh Carrie Furnaces site**: **Daniel O'Hare**, 52, third-generation Pittsburgh steel-and-iron-industry environmental engineer
- **Indianapolis Speedway-Adjacent**: **Krishna Iyer**, 39, joined from Cummins Indianapolis 2021
- **Louisville Standiford**: **Tasha Wilkerson**, 44, has been at MFI Louisville since the plant opened in 2007
- **Sherbrooke East Hubert (Quebec, Canada)**: **Sophie Lapointe**, 41, French-Canadian, bilingual French + English
- **Monterrey Apodaca (Mexico)**: **Lic. Roberto Salgado**, 48, Monterrey native, environmental engineer at the plant since MFI acquired the facility in 2016

For Scope-1, each plant reports monthly fuel consumption (natural gas + propane + diesel for plant equipment + a small amount of coal at Pittsburgh's site) + process emissions (the Cleveland + Akron + Pittsburgh forges have specific process-emission profiles from cold-rolling oils + hot-metal handling). The data flows into MFI's existing **FAS-COMSCO emissions-tracking system** (an SAP-adjacent legacy system MFI has used since 2012) which then exports to the oyatie `compliance` µservice via a structured-extract pipeline that Joshua set up last year.

For Scope-2, each plant's electricity consumption is reported per the utility provider:

- Cleveland + Akron + Pittsburgh + Indianapolis + Louisville: served by various US utilities (FirstEnergy, AEP, Duke Energy); Scope-2 location-based uses eGRID 2024 subregion emission factors; market-based uses MFI's renewable PPA contracts (MFI has 2 PPAs: a 47 MW wind PPA with EDF Renewables in Indiana, and a 31 MW solar PPA with Lightsource bp in Kentucky)
- Sherbrooke: served by Hydro-Québec; market-based ≈ 0 because Hydro-Québec is 99% hydroelectric
- Monterrey: served by CFE (Comisión Federal de Electricidad); under MFI's Solar-A PPA with CFE, market-based emission factor is ~32% lower than location-based

Aiko has a Tuesday 10:00 EDT weekly standup with all 7 plant leads via Webex. The standups run 45 minutes; each plant lead reports the previous week's data quality + any anomalies. By **Fri Oct 31** all 7 plants are reconciled. The cumulative Scope-1+2 numbers:

- **Scope-1**: 184,200 tCO2e (down 4.2% vs FY2025 184k → 176k; the Pittsburgh site's hot-metal-handling upgrade landed)
- **Scope-2 location-based**: 142,800 tCO2e (down 8.7% vs FY2025 156k → 143k; grid greening + Indiana wind PPA expanded)
- **Scope-2 market-based**: 96,400 tCO2e (down 14.2% vs FY2025 112k → 96k; PPA portfolio expansion)

`EVT-J170-SCOPE-1-COMPLETE-002` + `EVT-J170-SCOPE-2-COMPLETE-003` seal Friday Oct 31 17:18 EDT.

## §2 — Mon Nov 2 – Fri Dec 18 — Scope-3 supplier outreach

The Scope-3 phase is the hardest part. MFI has 412 tier-1 suppliers. The GHG-Protocol Scope-3 Standard requires reporting across 15 categories; MFI's material categories are:

- **Category 1: Purchased goods and services** (the biggest by far — raw materials: steel + aluminum + copper + alloying elements + lubricants + cutting oils; plus services: facility cleaning + security + IT + logistics-overhead) — **~68% of Scope-3**
- **Category 4: Upstream transportation and distribution** (rail + truck inbound material; rail + truck + intermodal outbound product) — **~14% of Scope-3**
- **Category 5: Waste generated in operations** (~3%)
- **Category 6: Business travel** (~1%)
- **Category 7: Employee commuting** (~2%)
- **Category 11: Use of sold products** (downstream — MFI's products go into auto + appliance + HVAC; calculating use-phase emissions is complex; ~8%)
- **Category 12: End-of-life treatment** (~4% — recyclability of steel + aluminum is high but the rest is small)

Aiko segments the 412 suppliers into 3 bands:

- **Band A (top 50 by spend)**: 81% of upstream emissions; full activity-data requested
- **Band B (51–200)**: 12% of emissions; partial activity-data + spend-based hybrid
- **Band C (201–412)**: 7% of emissions; spend-based only using EPA Supply-Chain GHG Emission Factors v1.3

Top-50 Band A includes (a sample):

- **Cleveland-Cliffs Inc.** (iron ore + pellets + DRI) — MFI's largest single supplier by spend (USD 412M FY2025)
- **U.S. Steel Corporation** (hot-rolled coil + cold-rolled feedstock) — USD 218M
- **Nucor Corporation** (re-bar + structural shapes) — USD 142M
- **Steel Dynamics Inc.** (flat-rolled steel) — USD 124M
- **Norsk Hydro ASA** (Sherbrooke aluminum primary metal) — USD 96M
- **Aluminerie Alouette Inc.** (Quebec primary aluminum) — USD 78M
- **Rio Tinto Aluminium** (cross-Canadian operations) — USD 64M
- **Quanex Building Products** (specialty aluminum) — USD 48M
- **Olin Corporation** (chlorine + industrial chemicals for surface treatment) — USD 42M
- **Eastman Chemical** (cutting oils + lubricants base stocks) — USD 38M
- **Quaker Houghton** (metalworking fluids) — USD 31M
- **Quaker Chemical** (separately; surface conditioners) — USD 28M
- **BNSF Railway** (inbound + outbound rail) — USD 89M (transport)
- **Norfolk Southern Railway** — USD 64M (transport)
- **Union Pacific Railroad** — USD 47M (transport)
- **C.H. Robinson Worldwide** (intermodal logistics) — USD 32M (transport)
- **ABM Industries** (facility cleaning across MFI plants) — USD 14M (services)
- **Honeywell International** (industrial controls + sensors maintenance contracts) — USD 18M (services)
- **Vinjamuri Industries Ltd.** (Indian alloying-element supplier — chromium + nickel + manganese) — USD 24M
- **Vale S.A.** (Brazilian nickel + iron) — USD 19M
- **Glencore plc** (cobalt + copper alloying elements) — USD 14M

(...and 29 more Band A suppliers)

Aiko + Joshua coordinate the outreach. For each Band A supplier, MFI sends a structured request via the `connect` µservice's cross-tenant data exchange channel. Each request carries:

- The mutual NDA reference (MFI has master-NDA with most Band A suppliers from years of business; for the few without, a fresh NDA is signed via QES through `contract-lifecycle-management`)
- The Scope-3 Category specification (which category MFI is requesting data for)
- The data structure expected (GHG Protocol Scope-3 Standard's activity-data fields)
- The submission deadline (Fri Jan 16 for activity-data; Fri Jan 30 for late stragglers)
- The cross-tenant audit-chain dual-seal protocol acknowledgement

Each Band A supplier's data-submitter principal (typically their own sustainability or supply-chain reporting officer) authenticates with WebAuthn-passkey-derived workload identity, signs the NDA acknowledgement, and gets a per-supplier Cedar permit scoped to the specific data category for this fiscal year.

By **Fri Dec 18** the outreach is complete:

- **Band A (50)**: 47 confirmed structured-submission commitment; 3 hesitant (will provide spend-based + supplemental data only)
- **Band B (150)**: 102 confirmed partial-activity-data commitment; 48 will provide spend-based only
- **Band C (212)**: all 212 will be modeled spend-based with EPA v1.3 emission factors (no outreach required)

`EVT-J170-SCOPE-3-OUTREACH-COMPLETE-004` seals Friday Dec 18 16:42 EST (clock change to Eastern Standard happened Nov 1).

Aiko + Joshua take 2 weeks off over the Christmas + New Year holidays. Aiko visits her parents in Wilmette IL with Emiko and Mateo for the week of Christmas; Joshua flies to Pittsburgh to spend the holidays with his family.

## §3 — Mon Dec 21 – Fri Jan 30 — Scope-3 supplier-data ingest

In early January Aiko returns to work + opens the `connect` µservice's supplier-data ingest panel. The ingest workflow is structured per-supplier:

1. Supplier-side data-submitter logs in to their tenant + opens the cross-tenant submission link
2. Cedar policy `connect.supplier_data_submit` evaluates (per the schema below)
3. Supplier submits structured data per the agreed schema
4. Data is dual-sealed in MFI tenant + supplier tenant under TrueTime fence
5. The `ontology` µservice maps the supplier-entity to an `Oyatie::SupplyChainPartner` node with emissions-attribution tags
6. The `audit-chain` µservice computes a per-supplier Merkle root + adds to the rolling FY2026 report Merkle tree

By **Fri Jan 30** all 47 Band A activity-data submissions are in. The aggregate Scope-3 emissions:

- **Category 1 (Purchased goods)**: 2,134,800 tCO2e (up 2.1% vs FY2025 — supply-chain growth)
- **Category 4 (Upstream transport)**: 412,400 tCO2e (up 3.4% — modal mix shifted slightly to truck from rail for Mexico-bound freight)
- **Category 11 (Use of sold products)**: 281,200 tCO2e (down 1.8% — auto OEMs' fleet-fuel-efficiency improvements pull this metric down)
- **All other categories combined**: 312,000 tCO2e
- **Scope-3 total**: 3,140,400 tCO2e (up 2.4% vs FY2025 3,068k)

The grand totals:

- **Scope 1+2 (market-based) + 3**: 184,200 + 96,400 + 3,140,400 = **3,421,000 tCO2e**
- FY2025 equivalent: **3,484,000 tCO2e**
- **YoY delta: -1.8%** (aggregate; MFI's Scope-1+2 reductions absorbed by Scope-3 growth)

`EVT-J170-SCOPE-3-INGEST-COMPLETE-005` seals Friday Jan 30 18:18 EST.

## §4 — Mon Feb 1 – Fri Feb 26 — reconciliation + assurance

The first 3 weeks of February are reconciliation. Aiko + Joshua walk through every emission-factor source citation, every supplier's data quality flag, every YoY-delta explanation. The `analytics` µservice generates per-category trend visualizations + the SBTi-alignment trajectory chart.

The SBTi trajectory:

- 2020 baseline (MFI's SBTi baseline): 4,128,000 tCO2e
- 2030 interim target (1.5°C-aligned): 2,394,240 tCO2e (42% reduction)
- FY2026 current: 3,421,000 tCO2e (17.1% reduction cumulative; on track for 1.5°C trajectory by 2030)

Mon Feb 23, 09:00 EST — the Ernst & Young Cleveland office assurance team arrives at the Halle Building for the 3-day on-site assurance review. The lead assurance partner is **Sarah Halloran-Park**, 51, E&Y Cleveland senior partner since 2018, has been MFI's sustainability assurance lead since 2020. She brings 3 staff — a Senior Manager (Greta Volkmann), a Manager (Tristan Liu-Schwartz), and a Senior (Priya Devasundaram).

The review is structured as **3-day Merkle-attestation-replay**:

- **Day 1 (Mon Feb 23)**: Scope-1+2 reconciliation. The E&Y team uses the `audit-chain` µservice's replay-mode to walk the Merkle proofs from each plant's monthly meter readings → through the FAS-COMSCO reconciliation → into the FY2026 aggregate. They probe 10% of readings randomly (480 readings sampled out of ~4,800). All 480 trace back cleanly.
- **Day 2 (Tue Feb 24)**: Scope-3 reconciliation. The E&Y team uses `connect` µservice's per-supplier Merkle proof to verify each Band A supplier's submission. They sample 5 of 47 Band A submissions. They request the cross-tenant audit-chain replay for each — the dual-seal in MFI tenant + supplier tenant must match byte-for-byte. All 5 match.
- **Day 3 (Wed Feb 25)**: Multi-framework composition validation. The E&Y team validates that the same underlying Merkle-attested data set composes into 4 framework variants with each variant's specific disclosure requirements honored.

At end of Day 3, Sarah Halloran-Park submits the **assurance opinion**:

- **0 material findings**
- **4 immaterial observations** (carried forward for FY2027 attention):
  1. Norsk Hydro Sherbrooke's smelter electricity emission factor uses 2024 average; consider using monthly granularity for FY2027
  2. Monterrey's PPA contract documentation has a 2 day gap (Aug 19-21, 2026) where market-based vs location-based assignment is ambiguous; reach out to CFE legal for clarification
  3. Category-11 (use of sold products) methodology assumes 10-year average vehicle lifetime; consider using OEM-specific lifetime data
  4. Joshua's spend-based-to-emission-factor mapping for the 312 Band-C suppliers uses EPA v1.3; recommend cross-validation with one or two leading consulting firms' emission-factor libraries

`EVT-J170-ASSURANCE-PASSED-007` seals Wednesday Feb 25 17:42 EST.

Anita Sehgal (CSO) takes Sarah out for dinner Wednesday evening at Le Petit Triangle Café (a French bistro in Ohio City; quiet, good wine list). Aiko + Joshua are not invited (they had their assurance-review-completion drink at the office). Aiko goes home + makes pasta with Mateo + plays with Emiko before bedtime.

## §5 — Mon Mar 1 – Fri Mar 19 — multi-framework composition

The first 3 weeks of March are filing-document drafting. The `compliance` µservice's multi-framework composer takes the single Merkle-attested data set and emits 4 report variants:

1. **CDP 2026 Climate Change Questionnaire response** — a structured JSON submission of ~340 fields covering MFI's climate-risk + emissions + targets + transition plan. Aiko + Joshua fill in narrative answers for ~80 of the 340 (the rest are auto-derived from the data).
2. **SEC 10-K Climate Disclosure** — fits into the existing 10-K Item 1 Business + Item 1A Risk Factors + Item 7 MD&A + new Item 16 Climate-related Information. The composer drafts the Item-16-specific language; legal counsel reviews.
3. **ESRS-E1 Climate Change disclosure for EU-CSRD** — emitted via Marlboro-Forge Holdings GmbH's German-language report. Aiko's German is limited; she uses the `intelligence` µservice's NLLB-200 translation with human-editor review (similar to j169's localization pattern; the human-editor is Veritem... wait, no, the human-editor for MFI's German is **Dr. Heinrich Brandt**, a Frankfurt-based sustainability-reporting consultant who has worked with MFI's German holding subsidiary since 2024).
4. **IFRS-S2 Climate-related Disclosures** — voluntary; MFI publishes on its investor-relations website. Composed in English; aligns with TCFD's 4-pillar structure (Governance + Strategy + Risk Management + Metrics & Targets).

`EVT-J170-MULTI-FRAMEWORK-COMPOSED-008` seals Friday Mar 19 18:42 EDT.

## §6 — Wed Mar 24, 14:42 EDT — final quorum signoff

Wednesday March 24 the final-filing Cedar quorum gate opens. The 4 quorum members:

1. **CSO Dr. Anita Sehgal** — 58, joined MFI as CSO in 2021 from a 14-year tenure at Cargill; PhD in Environmental Science from University of Wisconsin-Madison 1992
2. **CFO Marcus Engdahl** — 56, joined 2019 from a CFO seat at Olympic Steel; CPA + MBA from Northwestern Kellogg
3. **General Counsel Robert Cho** — 49, joined MFI 2022 from a Kirkland & Ellis Cleveland office partnership; Securities + ESG specialty
4. **Audit Committee Chair Dr. Elena Petrov** — 67, MFI Board member since 2018; PhD chemical engineering from Carnegie Mellon; previously CEO of a privately-held specialty-metals company in Pittsburgh

By 14:42:42 EDT all 4 vote PERMIT. `EVT-J170-FILING-PERMIT-009` seals at 14:42:42.118 EDT under TrueTime fence (uncertainty 1.4 ms).

Aiko goes home that evening + makes katsudon with her mother's recipe (Yuriko taught her; the breadcrumbs from Mitsuwa Marketplace in Westmont IL that her mother ships her quarterly). Mateo + Emiko + Aiko eat together. Emiko is at the stage where she insists on holding her own chopsticks (children's-size, plastic, pink); she drops them three times during dinner.

## §7 — Wed Mar 31, 14:00 EDT — filings

Wednesday March 31 14:00 EDT is the filing window. Aiko's team coordinates with:

- **EDGAR Filing Services Inc.** (the third-party filer MFI uses for SEC submissions) — the 10-K filing submits at 14:00:18 EDT; the SEC EDGAR receipt comes back at 14:08:42 EDT
- **CDP Worldwide** (the CDP submission portal) — the structured JSON response submits at 14:08 EDT; CDP confirms receipt at 14:14 EDT
- **Marlboro-Forge Holdings GmbH** — the ESRS-E1 disclosure is routed to the German subsidiary tenant `marlboro-forge-holdings-gmbh-frankfurt-de`, where the German-language report is filed with German Bundesanzeiger at 20:08 CET (= 14:08 EDT)
- **MFI Investor Relations website** — the IFRS-S2 disclosure is published at 14:18 EDT

`EVT-J170-FILINGS-COMPLETE-010` seals at 14:42 EDT.

The audit-chain Merkle root for the full FY2026 emissions report: `sha384-3b1d7a8e2c4f6b9a5d8e1c3f5a7c9b2d4e6f8a1c3e5b7d9f2a4c6e8b1d3f5a7c9e2b4d6f8a1c3e5b7d9f2a4`.

Aiko closes her Surface Laptop at 16:42 EDT. She goes home. Mateo has picked up Emiko from her grandparents' (Mateo's parents live in Lakewood OH; they watch Emiko on Wednesdays). Aiko's mother Yuriko calls her at 19:18 EDT from Wilmette. They speak Japanese.

**Yuriko 19:18 EDT** (Japanese): "明子、レポート無事に出した?" *("Akiko-chan, did the report submit OK?")* — note: Yuriko calls Aiko her Japanese name "Akiko" not the Anglicized "Aiko"; Aiko is the form she uses professionally; Akiko is her family name.

**Aiko 19:19 EDT** (Japanese): "うん、全部出した。SEC、CDP、ヨーロッパも、IFRSも全部。今年は静か。" *("Yes, all submitted. SEC, CDP, Europe, and IFRS too. This year was quiet.")*

**Yuriko 19:19 EDT**: "良かった。来週、お母さん遊びに行くね。" *("Good. Mom will visit you next week.")*

**Aiko 19:20 EDT**: "うん、来てね。Emikoも会いたい。" *("Yes please. Emiko wants to see you too.")*

## §8 — Beats not on the wire (the human texture)

- Aiko's mother Yuriko visited Cleveland in late January (around Aiko's 34th birthday) for a 10-day visit. While she was here, Yuriko cooked Japanese home-style meals every night and froze a 30-day supply for Aiko + Mateo + Emiko's freezer (tonkatsu, korokke, hayashi-rice, oyakodon, kabocha-no-nimono). Aiko got teary one evening when she saw all the labeled freezer-containers stacked in the freezer; Yuriko just said "Akiko, you have a baby, you work too much. お母さんから." Aiko ate the karaage with Mateo on a Thursday in mid-February when the assurance-review prep was particularly draining.
- The Norsk Hydro Sherbrooke sustainability-data-submitter is **Marie-Eve Boucher**, 38, a Quebec-City-trained chemist who has been at Norsk Hydro Sherbrooke since 2014. Aiko and Marie-Eve never met in person before this fiscal year — all their cross-tenant interaction was through the `connect` µservice. During the January data-ingest window Marie-Eve sent Aiko a Slack-bridged note saying that the smelter's monthly-granularity electricity-emission-factor calculation would help Norsk Hydro better attribute the seasonal hydro-vs-thermal mix to MFI's portion of their aluminum production. Aiko added it to the FY2027 carry-forward immediately. They have not met in person but they have a professional-respect relationship now.
- The Monterrey Apodaca plant environmental engineer **Lic. Roberto Salgado** is 48, a Monterrey native who has worked at the plant since MFI acquired it in 2016. He speaks Spanish (native, regio-montano dialect — same northern-Mexico dialect as Diego Vargas from j167) and English (B2). Aiko's Spanish is reading-only; on their bi-weekly Webex calls she speaks English; he speaks English with a strong accent + a few Spanish phrases. They have a comfortable working relationship built on 4 years of shared annual reports. Roberto sent her a WhatsApp message on December 28 wishing her felices fiestas; she replied feliz Navidad.
- The Pittsburgh Carrie Furnaces plant lead **Daniel O'Hare** is a third-generation Pittsburgh steel-and-iron-industry environmental engineer. His grandfather Patrick O'Hare worked at the original Carrie Furnaces site (which is now a National Historic Landmark site adjacent to MFI's modern Pittsburgh plant); his father Sean O'Hare worked at the same site through the 1970s LTV-Steel era. Daniel is 52, has a sentimental attachment to the place. During the November Scope-3 outreach he mentioned to Aiko in passing that one of MFI's Band-A suppliers (Cleveland-Cliffs) bought the historical Carrie Furnaces site in 2024 for preservation. Aiko didn't know this. She made a small note in her Moleskine.
- Joshua Park, Aiko's direct report, is preparing to apply to PhD programs in Sustainable Industrial Systems Engineering for fall-2027 admission. He has Northwestern's IEMS PhD program on his list (his and Aiko's alma mater for the BA; Aiko did her BA there). Aiko has agreed to write him a recommendation letter; she will write it in early summer 2027. He hasn't told the team yet.
- The Ernst & Young Senior Manager Greta Volkmann (who came on the assurance team this year) is German-American (born Cologne, moved to Cleveland with her family at age 9). She speaks German fluently. After Aiko realized that Greta could review the German-language ESRS-E1 disclosure independently of Dr. Heinrich Brandt, she + Anita decided to bring Greta in for the EU-subsidiary path validation. Greta did a side-by-side English-German check of the ESRS-E1 disclosure on March 18; she found 3 small phrasing improvements that Dr. Brandt agreed with. This is a good example of finding hidden depth in the audit team that the substrate's cross-tenant collaboration enabled.
- Aiko's father Robert Brown is American (Anglo-Irish descent). He retired in 2024 from a long career as an environmental-engineer at a Chicago-area municipal water-treatment authority. He understands the substantive content of his daughter's job in a way that her husband Mateo (a software engineer) doesn't. On a phone call in February Robert asked her: "Akiko, is the Cleveland-Cliffs Indiana Harbor pellet supply still using the natural-gas-DRI process or did they finally switch to hydrogen-DRI for any portion?" Aiko had to actually look it up; Cleveland-Cliffs had announced an H2-DRI pilot at one Minnesota site but not Indiana Harbor yet. Her father said "OK, watch the Toledo announcement, that's the one with hydrogen." She wrote it in her Moleskine.

## §9 — Stop condition for this story

This story documents the 6-month lived texture of MFI's FY2026 annual Scope-1+2+3 emissions reporting cycle. The acceptance criteria in `README.md`, the API shapes in `handshake.md`, the test cases in `integration-test-plan.md`, and the schema files together encode the machine semantics. The story exists so that the next human or agent reading the codepath can understand WHY the per-supplier Cedar permit + NDA-bound MLS-encrypted `connect` channel matters for honest cross-tenant emissions-data exchange (suppliers will only submit if they know their data is scoped to MFI's tenant alone), WHY the ontology mapping supplier-entity → `Oyatie::SupplyChainPartner` matters for the emissions-attribution chain (so a single supplier's data can be re-used in MFI's report + the supplier's own report + each of their customers' reports without losing provenance), WHY the multi-framework composer from a single Merkle-attested data set matters (the 4 framework variants must be byte-attestable as derived from the same source), WHY the E&Y assurance review uses Merkle-attestation-replay rather than PDF-binder review (the replay scales to 4,800+ readings + 47+ supplier submissions in 3 days vs the 2-week PDF-binder reviews of 2018-2022), and WHY a sustainability-officer with a Surface Laptop + 2 direct reports + occasional E&Y assurance reviews can run a full multi-framework annual disclosure cycle that historically required external consulting firms + 6-9 months of FTE-equivalent labor.
