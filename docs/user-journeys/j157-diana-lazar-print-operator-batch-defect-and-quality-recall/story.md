---
doc_class: User-Journey-Story
journey_id: j157-diana-lazar-print-operator-batch-defect-and-quality-recall
date: 2026-05-20
authority_tier: 2
status: draft
---

# j157 — Story: 11:42 EET in Cluj-Napoca, the colorbar flickers

## §0 — Mid-shift, Tuesday February 23, 2027, 11:42 EET

The pressroom at Tipografia Lazăr-Petrescu SRL is loud in the way that all sheet-fed offset pressrooms are loud: the Heidelberg Speedmaster CX 102-6+LX runs at 16,500 sheets/hour during normal production, and the rhythmic *thump-thump-thump-thump* of the grippers + the high-pitched whine of the suction feeder + the chemical sharpness of fountain solution + isopropanol fills the room. Diana Lazăr is wearing her FOGRA-blue cotton work shirt with the embroidered "D. Lazăr" tag her father stitched on himself in 2019 when she made operator. Her hair is tied back in a single braid. She wears safety glasses, ear protection (–28 dB foam plugs because the molded ones broke last week and she hasn't picked up new ones), and the lightweight nitrile gloves the FOGRA standard requires for color-touch handling.

She has been running batch `BCH-2027-02-23-0612-pharma-leaflet-NSAID-RO` since 06:18 EET. The batch is a 47,500-unit run of patient information leaflets (PIL) for **Antibiotice SA** (Iași), covering their new ibuprofen 400mg generic launch. The leaflet wording is regulator-locked: ANMDMR (Agenția Națională a Medicamentului) approved the wording on 2027-01-18 and any deviation makes the entire batch unreleasable. The PIL is a single-fold A5 sheet, 4-color front, 1-color (PMS Black) back, printed on **Munken 70gsm bible paper**.

By 11:42 EET she has run 23,847 good sheets. The remaining 23,653 are scheduled to finish by 14:08 EET if the press holds.

The press does not hold.

At **11:42:14 EET** the inline GMI ColorProof spectrophotometer (mounted between the last printing unit and the coater) flags a ΔE2000 spike. Diana sees it first on the **Heidelberg Prinect Cockpit** screen mounted at the operator station: the C/M registration solid bar's ΔE column flickers from a steady 1.4 to 4.7. The FOGRA-PSO tolerance ceiling for 100% solid is ΔE 3.0. She is now above tolerance.

Her oyatie tablet (a Panasonic Toughpad FZ-G2 mounted to the press's operator station via VESA arm) chirps with the same alert one second later — `quality-management` µservice is reading the same telemetry the press is.

She pulls the tablet, glances at the colorbar visualization, taps "show sample" — the live press camera (Heidelberg Inpress Control) snaps a frame of sheet 23,847 and the affected zone. She zooms in.

The bold-red allergy warning box on the front face is registering 1.2 mm low. The Romanian text at the bottom edge reads:

> **Nu administrați copiilor sub 6 ani fără sfatul medicului**

But the bottom 1.2 mm of the type body is being clipped by the box edge. On the affected sheets, the text reads:

> **Nu administrați copiilor ub 6 ani fără sfatul medicului**

The "s" of "sub" is half-truncated. It looks, to a casual eye, like "ub 6 ani" — easily misread by an elderly patient as "above 6 years" instead of "under 6 years". The mistake inverts the warning. A child under 6 receiving an adult-strength NSAID without medical advice is exactly what this leaflet exists to prevent.

She does not call her father. She does not call the dayshift manager. She acts.

## §1 — 11:42:38 EET: line stop

She taps the giant red **OPRIRE LINIE / LINE STOP** button on the oyatie tablet. The button is intentionally enormous — it dominates the lower third of the screen when an out-of-tolerance condition is active.

The Cedar evaluation runs in 47 ms:

- Principal: `diana.lazăr@tipografia-lazar-petrescu-ro`
- Action: `quality.production_line_stop`
- Resource: `press-line-heidelberg-cx-102-6-lx-01`
- Context: `principal.has_certification_unexpired("FOGRA-PSO-Operator-Level-2") == true`, `principal.role_in_tenant == "press_operator_day_shift"`, `shift_active == true`, `product_class == "pharma_PIL"`

Permit. No manager approval required. Her cert is her authority.

The press receives a clean halt command at **11:42:42 EET**. The Heidelberg Speedmaster decelerates over 14 seconds — by 11:42:56 the sheets have stopped advancing. Three sheets are in transit when she presses stop; the press's protocol marks them as **in-transit-quarantine** and they are physically moved by the off-press operator (her colleague Andrei Tăbârcă) to the segregated quarantine pallet within 2 minutes.

The `quality-management` µservice emits `EVT-J157-LINE-STOP-001` sealed in `tipografia-lazar-petrescu-ro`. The press control system confirms halt at 11:42:56; `EVT-J157-PRESS-HALT-CONFIRMED-002` follows.

Diana's tablet shows the post-stop screen:

```
LINE STOPPED — 11:42:42 EET
batch: BCH-2027-02-23-0612-pharma-leaflet-NSAID-RO
good sheets: 23,847
in-transit quarantine: 3
remaining to print: 23,653
operator: Diana Lazăr (FOGRA-PSO L2)
authority: operator-level (no manager approval required)

⇒ initiate recall workflow?
```

She taps yes.

## §2 — 11:44–11:51 EET: recall workflow initialization

The `workflow-engine` materializes the recall workflow `recall-bch-2027-02-23-0612-pharma-leaflet-NSAID-RO-2027-02-23` in state `stop_called` at 11:44:18 EET. The recall-state-machine.yaml drives 6 states with Cedar guards on every transition.

The `tasks` µservice materializes 14 atomic tasks at 11:44:42 EET:

1. ✓ line stop (auto-completed) — 11:42:42
2. ✓ in-transit quarantine (auto-completed) — 11:43:12
3. count clean sheets (1–23,847)
4. segregate suspect sheets (3 in-transit + the last batch zone)
5. retrospective sample inspection (47 sheets across the run, sampled per ISO-2859-1 AQL 1.0)
6. photograph each defect type (color drift + registration shift)
7. ship samples to QA lab (Tipografia internal lab + external Antibiotice QA if requested)
8. draft customer notification (RO + EN)
9. mechanical inspection of suspect roller (dampener-cylinder #4)
10. confirm root cause + classify
11. CAPA plan (correction + corrective + preventive)
12. customer recall execution (logistics: nothing has shipped yet so this is "do not ship")
13. regulator notification (ANMDMR template prepared, hold-and-wait pending customer response)
14. closure + post-mortem

Diana scans the list. She has done this before — three times in her 9 years at the press. This is the second time it involved a pharma product (the first was 2024-11-08, a Sandoz aspirin leaflet where the same dampener-roller wear pattern caused a similar registration shift; she diagnosed that one too).

She starts task #3 — count clean sheets. The off-press operator Andrei joins her at the delivery stack.

## §3 — 11:51–12:23 EET: counting, segregating, sampling

Diana and Andrei count the delivery stack. The Heidelberg's electronic counter says 23,847 good sheets exited the press. They confirm this manually by paper-counting in lifts of 100 — a slow process but ISO-9001 requires manual confirmation for any defect-triggered count of pharma-class product. They reach 23,847 ± 0 by 12:08 EET. The 3 in-transit sheets are quarantined on a separate red-tag pallet labeled with the batch ID + date + Diana's operator handle.

She moves to task #5: retrospective sampling. The ISO-2859-1 AQL 1.0 plan for a 23,847-unit lot calls for sampling 200 units with an acceptance number of 5 (i.e. accept if ≤5 defects in the sample, reject if ≥6). But Diana opts for the tighter pharma-class plan that Tipografia uses for ANMDMR-class product: AQL 0.4, sample size 315, acceptance number 3.

She and Andrei sample 315 sheets across the run — 21 from the first 5,000, 21 from the second 5,000, 21 from each subsequent block, plus 21 from the immediately-pre-defect block (sheets 22,847–23,847). Each sheet is barcode-scanned via her tablet to record the sample-ID + position-in-run + timestamp.

The inspection: she lays the samples on the inspection table under the FOGRA D50 5,000K lighting and walks each one with a 10× loupe + the ΔE2000 spectrophotometer (X-Rite eXact 2). She finds:

- 0 defects in samples from runs 0–10,000 (sheets 1–10,000)
- 0 defects in samples from runs 10,001–18,000
- 2 marginal-but-in-tolerance ΔE 2.7 + 2.9 readings in samples from runs 18,001–22,000 (early warning of the drift but within FOGRA spec)
- 14 out-of-tolerance ΔE ≥3.0 readings in samples from runs 22,001–23,847 (the affected zone)

The defect onset zone is sharp: it starts somewhere between sheet 22,000 and sheet 22,500. She marks the suspect zone as **sheets 22,000–23,847 (1,848 sheets)** with high confidence that the true defect-zone starts at sheet ~22,200 (interpolating from the ΔE trajectory).

She photographs 8 reference defects with her tablet's 12 MP camera + the ringlight attachment, and uploads via `tasks` task #6. Each photo carries EXIF: sample-ID + batch-ID + Diana's operator handle + GPS (the press's stationary GPS).

## §4 — 12:23–12:51 EET: root cause + customer notify draft

She walks to her father's office at the end of the pressroom. Mihai Lazăr-Petrescu is 62, the managing director since Vasile retired in 2018, and was a press operator himself for 27 years before he took the office. He looks up from his laptop. Diana speaks Romanian, her natural register:

**Diana 12:24 EET**: "Tată, am oprit linia. Batch-ul Antibiotice — registru deviat 1.2 mm jos pe ultimele 1,848 de coli. Avertismentul de pe leafletul cu ibuprofen este trunchiat — 'sub 6 ani' arată ca 'ub 6 ani'."

**Mihai 12:24 EET** (in Romanian): "Bine făcut că ai oprit. Cylinder #4 din nou?"

**Diana 12:24 EET**: "Aproape sigur. Verific cu Marius după ce notific Antibiotice."

(Marius Iancu is the press's plant-maintenance technician, an external Heidelberg-certified service partner.)

Mihai nods. He pulls up the oyatie messenger thread `dm-tipografia-antibiotice-bch-pharma-2027-02-23` — the standing thread between the two firms for the active project. The thread participants are: Diana + Mihai + her father's QA controller Liviu Apostol; on Antibiotice's side, Dr. Cristina Munteanu (Director Calitate) + Dr. Andrei Popescu (Persoană Calificată) + their procurement lead Carmen Ene.

Diana drafts the customer notification at 12:31 EET. The `workflow-engine` provides a template (in RO + EN); she fills in the structured fields and adds free-text. The bilingual content is structured side-by-side so neither version is "primary" — both are equal under the contract terms (Antibiotice's QMS requires English; Tipografia's QMS requires Romanian; both are persisted with diacritics intact).

Draft customer notification (substantive excerpt):

```
SUBJECT: Notificare urgentă recall lot BCH-2027-02-23-0612 — leaflet NSAID-RO 400mg ibuprofen Antibiotice
        Urgent recall notification, batch BCH-2027-02-23-0612 — leaflet NSAID-RO 400mg ibuprofen Antibiotice

Date/Ora:    2027-02-23 12:31 EET
Lot:         BCH-2027-02-23-0612-pharma-leaflet-NSAID-RO
Cantitate:   47,500 leaflets planificate; 23,847 tipărite din care 1,848 suspecte (sheets 22,000–23,847)
             47,500 leaflets planned; 23,847 printed of which 1,848 suspect (sheets 22,000–23,847)
Defect:      ΔE2000 registration drift > FOGRA tolerance + 1.2mm low shift on red allergy-warning box,
             clipping legally-required warning text "Nu administrați copiilor sub 6 ani"
Statut:      LINE STOPPED 11:42 EET; entire affected zone quarantined; no shipping; ZERO releases to market
             LINE STOPPED 11:42 EET; full affected zone quarantined; no shipping; ZERO releases to market
Root-cause hypothesis:
             dampener-roller cylinder #4 wear (suspected); plant-maintenance investigation in progress
             dampener-roller cylinder #4 wear (suspected); plant-maintenance investigation underway
Action requested from Antibiotice:
  (1) Confirm receipt within 4 hours per MSA §7.4
  (2) Confirm whether Antibiotice QA wishes external sample inspection (we will ship 50 sheets next-day)
  (3) Confirm whether to re-run the suspect zone (we estimate next available slot 23-feb 22:00 EET on Heidelberg #2)
  (4) Confirm regulator-notification posture (we have NOT triggered ANMDMR notification because
      no product reached market; recommendation is to hold the regulator template pending your assessment)

Signed:      Diana Lazăr (Operator FOGRA-PSO L2) + Mihai Lazăr-Petrescu (Managing Director)
```

She sends at 12:42:18 EET — within the 90-minute SLA per AC-J157-005. `EVT-J157-CUSTOMER-NOTIFY-005` dual-seals in BOTH `tipografia-lazar-petrescu-ro` AND `antibiotice-sa-ro`.

## §5 — 12:51–13:18 EET: customer response

Dr. Cristina Munteanu reads the notification at 12:48 EET (she had been in another QA review meeting). She responds at 12:51:14 EET. Her response is in Romanian:

**Cristina 12:51 EET**: "Mulțumim Diana, mulțumim Mihai. Confirmăm că ați acționat corect. Vă rugăm să ne trimiteți 50 de coli pentru inspecție internă în lab-ul Antibiotice Iași. Convenim re-rularea în slot-ul 23-feb 22:00 EET pe Heidelberg #2. Cu privire la ANMDMR — confirmăm că nu vom escala câtă vreme niciun leaflet nu a părăsit incinta voastră. Vă rog să ne trimiteți raportul CAPA în 48 de ore conform §7.4. Cu stimă, Dr. Cristina Munteanu, Director Calitate Antibiotice."

(English gloss: "Thank you Diana, thank you Mihai. We confirm you acted correctly. Please send us 50 sheets for internal inspection at Antibiotice Iași lab. We agree to re-run in the Feb-23 22:00 EET slot on Heidelberg #2. Re ANMDMR — we confirm we will not escalate as long as no leaflet has left your premises. Please send us the CAPA report within 48 hours per §7.4. Sincerely, Dr. Cristina Munteanu, Director Calitate Antibiotice.")

`EVT-J157-CUSTOMER-CONFIRM-006` dual-seals.

`EVT-J157-REGULATOR-PATH-PREPARED-007` records the ANMDMR template's prepared-but-not-sent state.

Mihai exhales. Diana writes a one-word reply: "Mulțumim."

## §6 — 13:18–15:42 EET: root cause + CAPA

Marius Iancu (Heidelberg field-service technician) arrives at 13:32 EET. He has done this dance before; the press's dampener-roller cylinder #4 was last serviced 2024-11-12 after the Sandoz aspirin event. He pulls cylinder #4 with Andrei's help, mounts it on the inspection rig, measures circumferential runout with a dial indicator:

- TIR (total indicated runout): 0.067 mm
- Heidelberg service spec: ≤ 0.020 mm
- Heidelberg replace spec: ≥ 0.050 mm

The roller is past replace-spec. The wear is asymmetric — Marius traces it to a slight bearing-housing misalignment that has accumulated 2.4 years of runtime since the 2024 service. The 1.2 mm registration shift in the print zone correlates exactly with the 0.067 mm cylinder runout multiplied by the registration-amplification factor for the specific print-unit geometry.

`EVT-J157-ROOT-CAUSE-CONFIRMED-009` seals at 14:18 EET. Plant-maintenance issues work order `WO-TIP-2027-02-23-DAMPENER-ROLLER-04-REPLACE` for cylinder replacement during the 22:00 EET shift turnover.

Diana drafts the CAPA plan in the `notes` µservice at 14:32–15:42 EET — collaborative editing with her father:

**Correction (immediate, hours):**
- Quarantine all 23,847 sheets pending Antibiotice QA's 50-sheet sample inspection
- Replace dampener-roller cylinder #4
- Re-run the suspect zone in the Feb-23 22:00 EET slot

**Corrective action (this incident, days):**
- Update press maintenance schedule to inspect dampener-roller cylinders monthly (was quarterly)
- Add ΔE2000 trend-alert at ΔE ≥ 2.5 (was ΔE ≥ 3.0) — earlier warning
- Train all 6 day-shift + night-shift operators on the FOGRA-PSO operator-line-stop authority drill

**Preventive action (systemic, weeks):**
- Adopt Heidelberg's optional "predictive bearing-alignment monitoring" upgrade (€18,400) — capital request prepared
- Quarterly cross-checking with FOGRA reference samples at independent lab
- Annual cert refresh for all operators (FOGRA-PSO L2 + ISO-12647-2)

`EVT-J157-CAPA-FILED-008` seals at 15:48 EET. The CAPA artifact is structured per ISO-9001 §10.2 and is reconstructible from the audit chain.

## §7 — 15:42–20:17 EET: sample ship, re-run prep, shift handoff

Andrei prepares 50 sample sheets for Antibiotice Iași. He stamps each "QUARANTINE — DEFECT SAMPLE — DO NOT USE" in Romanian + English on the back face, packs them in an acid-free archival sleeve inside a tamper-evident box (`tamper-seal-id: ts-2027-02-23-tip-anti-001` recorded in `tasks`), and seals it. The courier (Cargus express) collects at 16:18 EET. Tracking number `cargus-2027-02-23-CC-7741293` is logged in `tasks` task #7. Estimated delivery to Antibiotice Iași QA: Wed Feb 24 09:30 EET.

Marius replaces cylinder #4 between 16:00 and 19:30 EET. He re-pulls test sheets at 19:42 EET on Heidelberg CX 102; ΔE2000 reading on the test set is 0.8 — well within FOGRA spec.

At 20:00 EET the night-shift operator Vladimir Csikós arrives. Vladimir is 41, Hungarian-Romanian dual identity (his messenger handle uses both spellings: "Vladimir Csikós / Csikós Vladimír"). He has 14 years of experience on the Heidelberg fleet. Diana hands off:

**Diana 20:14 EET** (Romanian + Hungarian, code-switching naturally as they always do): "Vladimir, batch-ul Antibiotice — recall în execuție. Roller #4 înlocuit. Test sheets passed. Re-run scheduled for 22:00 EET on Heidelberg #2 not on #1. Toate notele sunt în oyatie sub recall ID. CAPA file-uit."

**Vladimir 20:14 EET**: "Köszönöm, Diana. Értem. Megnézem a részleteket."

(Hungarian: "Thank you Diana. Understood. I'll review the details.")

`EVT-J157-SHIFT-HANDOFF-011` seals at 20:17:18 EET. Diana clocks out, picks up her daughter Maria from after-school care at 20:42, and is home by 21:18.

## §8 — Beats not on the wire (the human texture)

- At 12:18 EET Andrei brought Diana a glass of cold water from the breakroom cooler. The water was Borsec — Romanian sparkling mineral water. He has been bringing her water during quality events since 2019. Mihai's office radio plays Hungarian folk music quietly on most days — Diana doesn't speak much Hungarian but recognizes the melodies.
- At 14:42 EET Diana texted her daughter Maria via the family chat (NOT through any work tenant; this is her personal device on its own messenger): "ajung mai târziu astăzi pisicuță, recall pe presă, vorbim seara". The boundary between her work and personal communications is clean — no oyatie work-tenant pill is visible on her family chat.
- At 17:14 EET Cristina Munteanu sent a private follow-up to Mihai (a side-channel within the messenger thread, marked Mihai-only by Cristina): "Mihai, vă mulțumesc pentru transparență. Dacă veneau leafletele cu greșeala asta, era recall ANMDMR și pierdeam încrederea publicului. Diana a salvat cazul. Spune-i te rog." Mihai forwarded it (with permission) to Diana at 17:42. She read it walking to her car at 20:18 and smiled.
- The Heidelberg's chief engineer back in Heidelberg, Germany, will see Marius's service report in the global Heidelberg telemetry on Wed Feb 24. He has been watching the dampener-roller wear pattern across the European fleet since 2024. This Romanian event will become the 47th data point in the global pattern.

## §9 — Stop condition for this story

This story documents the lived texture of the 8h35m journey from defect detection to shift handoff. The acceptance criteria in `README.md`, the API shapes in `handshake.md`, the test cases in `integration-test-plan.md`, and the schema files together encode the machine semantics. The story exists so that the next human or agent reading the codepath can understand WHY the Cedar policy grants the operator line-stop authority without manager approval, WHY the diacritic preservation invariant matters, and WHY the bilingual + cross-tenant + regulator-touching pattern is built the way it is.
