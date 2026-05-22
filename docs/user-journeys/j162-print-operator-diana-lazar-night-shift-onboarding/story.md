---
doc_class: User-Journey-Story
journey_id: j162-print-operator-diana-lazar-night-shift-onboarding
date: 2026-05-20
authority_tier: 2
status: draft
---

# j162 — Story: 21:18 EET in the pressroom, Vladimir watches

## §0 — Tuesday January 26, 2027, 21:18 EET — Tipografia Lazăr-Petrescu, Cluj-Napoca

The pressroom is quieter at 21:18 EET than it was during Diana's 11:42 EET defect-detection moment back in February 2027 in j157 — the day-shift ended at 14:30 EET, the press has been on standby since 18:00 EET, and only three people are in the building now: Diana Lazăr (her diacritics still spelled exactly that way per the firm's identity records continuous since 2018), Vladimir Csikós (also continuously diacritic-preserved from j157), and Adriana Stanciu, the firm's external HSE consultant (a 53-year-old former chemical-engineer-turned-occupational-health-specialist who has consulted for Tipografia since 2021).

The Heidelberg Speedmaster CX 102-6+LX has been cleaned + standby for the assessment scenarios Adriana arranged with Vladimir over the past two weeks. The lights are at 60% — the night-shift baseline. The inspection station at the delivery end is at 100% (Adriana insisted on this — night-shift visual inspection cannot be at 60%; that's a CSN-EN-1837 baseline interpretation).

Diana is in the same FOGRA-blue cotton work shirt that appeared in j157. Her FOGRA-PSO L2 cert (issued 2024-09-18, valid through 2027-09-18) is the same cert that already authorizes her day-shift line-stop authority. What she's being assessed for tonight is **night-shift solo authorization** — a separate competency that her father Mihai needs her to hold before he can pull her into the night rotation his firm now needs because the pharma-PIL workload from Antibiotice (her j157 customer) has nearly doubled in 11 months.

She has spent the past 4 weeks in a structured onboarding workflow that `learning-management` µservice has driven. The training modules:

- **Module 1 (8h, Dec 28–30)**: RO-Codul-Muncii Law 53/2003 §§Title III Chapter II night-work obligations + worker rights (taught by Adriana)
- **Module 2 (4h, Jan 5)**: ISO-45001 night-shift specific fatigue management + lone-worker protocol (taught by Adriana + Vladimir together)
- **Module 3 (4h, Jan 12)**: CSN-EN-1837 industrial print-room low-light protocol + Heidelberg-specific operating procedures under reduced illumination (taught by Vladimir + the Heidelberg field-service partner Marius Iancu, same Marius from j157)

And 8 supervised night-shifts logged from Dec 29 through Jan 23 — paired runs where Diana operated under Vladimir's direct supervision, gradually taking on more responsibility (the first 2 shifts she shadowed; shifts 3–5 she ran with Vladimir present-but-passive; shifts 6–8 she ran with Vladimir nearby-but-on-another-press).

Tonight is the final practical assessment. 14 scenarios. Pass = ≥85% per category + Vladimir's qualitative sign-off + Adriana's HSE-compliance sign-off.

Adriana opens the assessment on her own Panasonic Toughpad FZ-G2 (the same model as the press operator station, deliberately — Adriana wanted to assess Diana on the actual tablet UI Diana will use):

```
ASSESSMENT — Night-shift solo authorization
candidate: Diana Lazăr (FOGRA-PSO L2 since 2024-09-18)
proctor: Vladimir Csikós (night-shift senior operator)
observer: Adriana Stanciu (external HSE consultant)
scenarios: 14
pass threshold: ≥85% per scenario + qualitative sign-off
start: 21:18 EET 2027-01-26
```

## §1 — 21:18–22:42 EET: 14 scenarios

The scenarios run as live operational exercises on the standby press:

1. **Cold-startup sequence under low-light** — Diana runs the full cold-startup sequence (alarm-system de-arm via the cooperative Securitas interface; lights configuration; press warmup; ink-density baseline) in the actual night-shift lighting profile. She speaks the steps aloud per Vladimir's preferred coaching style.
2. **Mid-run ink-density correction (low-light)** — Vladimir introduces a simulated 3% ink-density drift on print-unit 4 cyan. Diana detects it within 90 seconds via the on-tablet ΔE2000 alerter (same data feed as j157), corrects via the press's Prinect Inpress Control, and re-verifies. 87% on this scenario.
3. **Paper-jam clearance during low-staffed shift** — Vladimir fakes a paper-jam in the feeder. Diana stops the press cleanly, identifies the jam location with the inspection-station 100% lighting, clears the sheet, re-tensions the feeder, runs a 50-sheet test, validates. 94%.
4. **Chemical spill response with lone-worker** — Vladimir spills a small amount of fountain-solution + isopropanol on the press base mat (cleaning solvent — non-hazardous quantity for assessment purposes). Diana follows the lone-worker spill protocol: isolate, ventilate, log via `tasks` with photo, escalate if exceeds 250ml. 91%.
5. **Emergency-stop drill** — Vladimir signals an emergency. Diana hits the press E-stop physical button + tablet emergency button; logs the event via `quality-management` `quality.emergency_stop`; verifies all sheets-in-transit accounted for. 96%.
6. **Lone-worker dead-man check-in simulation** — The tablet fires a check-in alert at 21:42; Diana taps + face_id within the 60s window. 100%.
7. **Substrate change-over** — Diana swaps from 70gsm bible paper to 100gsm coated stock (the substrate change for an early-morning batch). Adjusts feeder + delivery pile-height + press registration tolerances. 89%.
8. **ΔE2000 trend alert response** — Vladimir simulates a ΔE 2.6 reading at the inspection — within tolerance but trending. Diana initiates a corrective ink-flow adjustment without halting. 92%.
9. **Customer audit-readiness query** — Vladimir asks Diana to produce, from her tablet, the audit-chain proof of cold-startup for tonight (continuous from §1.1) and an extract of the last 30 minutes of press telemetry. Diana finds + exports both in 4 minutes. 88%.
10. **Plant-maintenance interaction** — Vladimir tells Diana that the plant-maintenance van is arriving for a 02:00 EET scheduled work-order on print-unit 6 (a different press). Diana confirms via `messenger` to the maintenance tech (Marius Iancu's apprentice on this shift) and coordinates the work-order pause + resume around the schedule. 90%.
11. **Pharma-PIL specific protocol** — Vladimir presents Diana with a simulated NSAID-class pharma-PIL batch (intentionally connecting to her j157 history). Diana walks through the additional registration-tolerance + legal-text-clipping inspection cadence required for pharma per AC-J157-002 + ANMDMR-aware protocols. 95%.
12. **Cross-shift handoff scenario** — Vladimir simulates a 06:30 EET handoff to day-shift operator (Diana's sister-in-law Camelia Lazăr, who is on the firm's relief roster). Diana drafts the handoff notes in the `notes` µservice, attaches the active-recall context + open-task summary + production-planning delta. 93%.
13. **Personal-tenant family-emergency contingency** — Vladimir asks Diana to demonstrate the personal-tenant escalation protocol: what happens if Diana's daughter Maria is sick and Diana needs to leave mid-shift. Diana shows the personal-tenant emergency-relief request via `messenger` to Mihai (her father, who is on the escalation roster) + the Securitas re-arming protocol. Vladimir notes that the personal-tenant cross-tenant request is correctly handled (Cedar context: Diana's personal tenant calls the firm tenant; firm tenant logs the unscheduled-leave; alarm rearm scope is properly Cedar-narrowed). 89%.
14. **Final integrative scenario** — Vladimir runs a 25-minute live production simulation: cold-startup → 1500-sheet pharma-PIL run → mid-run paper-jam → ΔE drift → dead-man check-in → shift-end cool-down. Diana handles the whole sequence smoothly. 93%.

At 22:42:18 EET Adriana stops the assessment. She reviews her scoring notes on the Toughpad.

**Adriana 22:43 EET** (Romanian): "Diana, ai trecut. Toate scenariile peste 85%. Patru categorii la 90% sau peste. Mihail va fi mulțumit. Vladimir, semnezi?"

**Vladimir 22:43 EET** (Hungarian-Romanian code-switching, characteristic): "Da. Semnez. Diana, gata. Ai noaptea ta acum."

(Hungarian-Romanian gloss: "Yes. I sign. Diana, ready. You have your night now.")

`EVT-J162-COMPETENCY-ASSESSED-001` seals at 22:43:42 EET. Vladimir's sign-off + Adriana's sign-off both recorded.

The Cedar evaluation runs:

- Principal: `diana.lazăr@tipografia-lazar-petrescu-ro`
- Action: `learning_management.competency_unlock_night_shift_solo`
- Resource: `Competency::"night-shift-solo-authorization-2027"`
- Context: `assessment_score_min_85_per_category == true`, `proctor_signoff == true`, `hse_consultant_signoff == true`, `prerequisite_certs_unexpired == true`, `supervised_shifts_logged >= 8`

Permit. The competency unlocks. `EVT-J162-COMPETENCY-UNLOCKED-002` seals at 22:48:14 EET.

## §2 — Wed Jan 27, 09:00 EET — workplace-integration provisioning

Wednesday morning Diana is back at the depot at 09:00 EET for the workplace-integration provisioning session. Her father Mihai is there + the firm's bookkeeper Carmen Petrescu (Mihai's late-wife's cousin, hired 2019 to handle Tipografia's accounts + payroll + tax filings + Securitas alarm-cooperative interface).

The `workplace-integration` µservice provisions the following in a sequence over 47 minutes:

1. **Shift schedule entry** — Mon Feb 1 22:00–06:30 EET added to Diana's shift schedule; Carmen confirms via her own tablet; audit fires
2. **Geofenced clock-in zone** — the Tipografia depot perimeter (Strada Mihail Kogălniceanu specific GPS coordinates) is now valid for Diana's clock-in on the night-shift schedule
3. **Badge updated** — Diana's RFID badge role is updated to include night-shift entry + alarm-cooperative interaction
4. **Securitas alarm-cooperative scope** — Diana's biometric is now in the Securitas cooperative-business roster as a night-shift authorized de-armer (this is cross-tenant: Tipografia ↔ Securitas Bucharest's Romanian division, which is itself a cooperative-tenant with Tipografia paying alarm-monitoring fees)
5. **Payroll night-shift differential** — +25% per RO Codul Muncii §126 enabled in Carmen's payroll module; Diana's standard rate of 47 RON/hour for FOGRA-PSO L2 day-shift work becomes 58.75 RON/hour for night-shift hours
6. **Dead-man protocol pre-arm** — `identity` flags Diana as ready for dead-man enrollment (next step)

`EVT-J162-WORKPLACE-INTEGRATION-PROVISIONED-003` dual-seals in `tipografia-lazar-petrescu-ro` AND `cz-securitas-alarm-cooperative-tenant-ro` at 09:47:18 EET.

## §3 — Wed Jan 27, 11:42 EET — identity + dead-man enrollment

Mid-morning Diana sits at her father's office desk with her own Toughpad (the one she'll use on press). The `identity` µservice walks her through dead-man enrollment:

- **Biometric reconfiguration for low-light** — her face_id template is re-enrolled with low-light captures (the press's night-shift 60% lighting); takes 4 captures + 90 seconds of validation
- **PIN fallback** — she sets a 6-digit PIN for cases where face_id fails (e.g., heavy goggles + the inspection-station glare)
- **Lone-worker dead-man cadence** — she selects 4-hour intervals as her primary check-in cadence; with 60-second response window; with auto-escalation to Mihai's mobile if she misses
- **Personal-tenant escalation contact opt-in** — she explicitly enrolls Mihai's personal-mobile (his personal tenant `mihai.lazar-petrescu.personal`) as the dead-man escalation contact; EU-GDPR consent capture is explicit (Mihai must opt-in too; he does so from his own mobile)
- **Family emergency override** — she enrolls her daughter Maria's afterschool program's emergency phone as a separate cross-tenant escalation route (school's tenant `scoala-internationala-cluj-ro`)

`EVT-J162-DEAD-MAN-ENROLLED-004` seals at 12:18:42 EET.

## §4 — Thu Jan 28 14:18 EET — first night-shift work-order

Thursday afternoon Diana receives her first night-shift work-order via `tasks`:

```
WO-TIP-2027-02-01-NIGHT-WO-NSAID-batch-2
Tipografia Lazăr-Petrescu SRL → Antibiotice SA (cross-link to j157 customer)
batch: BCH-2027-02-01-2200-pharma-leaflet-NSAID-RO-batch-2
quantity: 38,400 PILs
substrate: Munken 70gsm bible paper (same as j157)
front: 4-color + back: PMS Black (same)
deadline: Tue Feb 2 14:00 EET delivery
press: Heidelberg CX 102-6-LX-01 (j157 press)
night shift: Mon Feb 1 22:00–06:30 EET
operator: Diana Lazăr (first solo authorization)
off-press operator: Andrei Tăbârcă (relieves Sergiu, who is on leave)
```

Mihai chose this batch intentionally: it is the **continuation of a batch sequence Diana had previously run** (Antibiotice has been a steady customer since the j157 event in Feb 2027). The pharma-PIL workflow is familiar; the night-shift is new; the chosen overlap reduces compound novelty.

`EVT-J162-FIRST-WO-ISSUED-005` seals.

## §5 — Sat Jan 30, 10:18 EET — walkthrough with Vladimir

Saturday morning Diana spends 4 hours at the press with Vladimir during a slow day-shift moment. They walk through the specific night-shift readiness sequence:

- Alarm de-arm (Securitas interface): Diana practices the biometric + Cedar-context call; the alarm system acknowledges; she rearms; un-arms; rearms (three repetitions to build muscle memory)
- Lighting setup: how she'll switch from day-mode 100% to night-mode 60% + how the inspection-station maintains 100% independently
- Paper-stock verification: where the Munken 70gsm bible paper inventory is stored; the dehumidified room; the inventory log accessible via `tasks` µservice
- Dead-man check-in cadence: Vladimir confirms 4-hour intervals are correct; for a 22:00–06:30 shift that's 02:00 + 06:00; Diana mentally drills the 60-second response window
- Emergency contact: Vladimir confirms Mihai is on-call as primary escalation; Adriana as secondary HSE escalation; Marius (Heidelberg field-service) as tertiary technical escalation

Diana feels prepared. Vladimir hands her his personal lone-worker bracelet — a thin titanium band with a small embedded fall-detect accelerometer that he's worn for 7 years; it's not required by the protocol but it's an additional safeguard. She accepts it.

## §6 — Sun Feb 1, 18:42 EET — pre-shift evening

Sunday evening. Diana arrives home from her half-day relief shift at 18:42 EET. Maria is at the kitchen table doing math homework (4th grade now; she's working on long division). Diana's husband — Răzvan Lazăr-Petrescu, 36, married to Diana since 2016, a teacher at Liceul Teoretic Avram Iancu in Cluj-Napoca — is making mămăligă cu brânză for dinner. The flat smells of cornmeal + butter + sheep's cheese.

**Răzvan 18:48 EET** (Romanian): "Diana, ai stat și astăzi pe presă?"

**Diana 18:48 EET**: "Doar trei ore. Diseară am tura de noapte primă solo. Voi dormi un pic acum."

**Răzvan 18:49 EET**: "Aha. Mâncați cu Maria, eu mă duc să fac corecturile."

She eats with Maria + Răzvan; takes Maria to bed at 20:30 (Maria reads Iliada copilăriei — a Romanian children's adaptation of the Iliad — for 10 minutes, then sleeps); Diana sleeps from 20:48 to 20:54 (a power-nap, not real sleep), then packs her bag: her Toughpad (work-tenant device); her personal iPhone; the lone-worker bracelet Vladimir gave her; a thermos of green tea; a sandwich (Răzvan made it).

At 21:32 she's in her car (the family's Skoda Octavia, 2019, manual transmission, she always drives it because she likes the gearshift) driving south on Strada Bisericii Ortodoxe toward the depot. Cluj-Napoca is quiet on a Sunday night; light snow has been falling since 19:00. The temperature is -3°C. Her headlights pick up snowflakes spiraling into the windshield.

## §7 — Mon Feb 1, 21:51 EET — depot, alarm de-arm

She parks at the depot at 21:51 EET. The depot's external lights are on; the press hall internal lights are off (standby). She walks to the side door (the operator entrance, not the main reception which is locked outside business hours).

She holds her badge to the reader. Her face is scanned by the low-light Securitas camera. The Cedar context is built:

- Principal: `diana.lazăr@tipografia-lazar-petrescu-ro`
- Action: `securitas.alarm_cooperative_dearm`
- Resource: `AlarmZone::"tipografia-pressroom-night-shift"`
- Context: `principal.workplace_integration_provisioned == "night-shift"`, `shift_scheduled_for_now == true`, `biometric_low_light_match == true`, `principal.has_competency_unexpired("night-shift-solo-authorization-2027") == true`

Permit. The Securitas system de-arms the pressroom zone at 21:54:18 EET. `EVT-J162-ALARM-DEARMED-006` dual-seals in `tipografia-lazar-petrescu-ro` AND `cz-securitas-alarm-cooperative-tenant-ro`.

She walks into the press hall. The lights come up to 60% via the building-management overlay (a small detail Mihai added in October 2026 specifically for this night-shift: a separate lighting profile triggered by night-shift de-arm). She inhales — the press hall always smells of ink + isopropanol + a faint cedar from the paper-stock room; tonight it also smells faintly of clove from a stick of incense Vladimir keeps in his locker (he says it helps him stay alert on the night shift).

Andrei Tăbârcă arrives at 21:53. The two of them walk the pressroom once (the standard pre-shift sweep). All is in order.

## §8 — Mon Feb 1, 22:00 EET — clock-in + shift start

At 22:00:00 EET exactly Diana taps her Toughpad's clock-in button. The geofence verifies (she's within the depot's 18m radius); biometric (face_id) confirms; her shift starts. `EVT-J162-SHIFT-CLOCK-IN-006a` seals.

Andrei clocks in at 22:00:30 EET. They're both on shift. Diana speaks first:

**Diana 22:00 EET** (Romanian): "Andrei, ești bine? Tu ai dormit cumva ieri?"

**Andrei 22:01 EET**: "Diana, da, am dormit până la 17:00. Și tu?"

**Diana 22:01 EET**: "Am dormit puțin. Voi face cafea."

She makes Romanian instant coffee (a tradition from her father's pressroom days). They drink it standing. Then they start the press cold-startup.

## §9 — Mon Feb 1, 22:00 EET – Tue Feb 2, 06:30 EET — the night shift

The shift is unremarkable in the way that good shifts are unremarkable:

- **22:00–22:42**: cold-startup; ink baseline; substrate loading; first 50 sheets test print; ΔE 0.9 (excellent; within FOGRA tolerance)
- **22:42–02:00**: ~ 4 hours of continuous run; 17,200 sheets printed by 02:00; one paper-jam at 23:42 EET (caught by Diana within 30s; cleared in 14 minutes; 28 sheets in transit voided as bad copies; documented in `tasks`)
- **02:00 EET**: dead-man check-in fires; Diana taps + face_id within 6 seconds; check-in confirmed; `EVT-J162-DEAD-MAN-CHECKIN-006b` seals
- **02:00–06:00**: another ~ 4 hours of continuous run; 16,800 sheets by 06:00; one ΔE alert at 04:18 EET (drift to 2.7, still in tolerance but climbing; Diana applies an ink-density correction proactively; ΔE returns to 1.2 within 8 minutes)
- **06:00 EET**: dead-man check-in fires again; she taps + face_id within 4 seconds; confirmed
- **06:00–06:30**: cool-down; press cleaning sequence; final-shift count: 34,500 sheets good, 28 voided (paper-jam transit), 100 voided as initial test prints, 24 voided as drift-period sheets (subset of the 04:18 alert window even though they were technically in-tolerance, Diana voided them as conservative); net good sheets 34,348 of the 38,400 planned

Andrei worked the delivery end + the substrate dehumidification room cycle + the secondary inspection station throughout. They spoke about 18 times over the 8h30m: technical exchanges + 3 coffee breaks + once about Andrei's daughter's school project + once about the snow.

## §10 — Tue Feb 2, 06:30–06:42 EET — handoff to Camelia

Day-shift operator Camelia Lazăr arrives at 06:18 EET. Camelia is 37, Diana's husband Răzvan's cousin, married, two children (twin boys age 6), works the day-shift relief role one week per month while her primary job is at a different print firm in Florești. Camelia is FOGRA-PSO L2 certified herself; she has worked Tipografia's presses since 2017.

**Camelia 06:30 EET** (Romanian): "Diana, prima noapte? Cum a fost?"

**Diana 06:31 EET**: "Bună. Un singur jam, o derivă mică. Roller schimbat luna trecută face minuni. Reziduuri sunt în coșul roșu. Notele complete în oyatie sub WO-2027-02-01."

**Camelia 06:32 EET**: "Excelent. Mihai e mândru."

Diana hands off via the structured workflow. `EVT-J162-SHIFT-HANDOFF-008` seals at 06:42:18 EET.

She walks out into the cold air at 06:48 EET. The sun is just starting to come up over Cluj-Napoca's eastern hills. The snow has stopped; the sky is pale gray-blue. She drives home with the radio off, in silence.

`EVT-J162-FIRST-NIGHT-SHIFT-COMPLETE-007` seals at 06:42 EET. Subsequent: `EVT-J162-NIGHT-PREMIUM-PAID-009` queues for the next Friday payroll cycle (Diana's night-shift hours: 8.5h × 58.75 RON/h = 499.38 RON gross; net after CAS + sănătate + impozit per Codul Muncii ≈ 312 RON).

## §11 — Beats not on the wire (the human texture)

- The lone-worker bracelet Vladimir gave Diana on Saturday was his wife's. His wife Erika died in 2019 of a cardiac event; she wore it (Vladimir bought her a matching one when he started wearing his); after she died Vladimir kept hers. He gave it to Diana not as a casual gift but as a careful one — he never explicitly told Diana whose bracelet it was. Diana noticed during the assessment that it was slightly smaller than Vladimir's (men's vs women's sizing) but didn't ask. She wore it for her first night shift. She'll keep wearing it.
- Camelia's relief-shift presence is a quiet family arrangement Mihai made after his wife Elena (Diana's mother) died in 2022. Camelia is Răzvan's cousin; Răzvan and Diana married in 2016; the families have been close since. Camelia's relief work is paid + audited + treated as professional, but it's also part of how the family stays inside the firm's life. Diana didn't ask Mihai to put Camelia on her first-solo-shift's handoff slot — Mihai made that choice himself, knowing that "the first solo shift's handoff is into family" is a quiet kindness.
- At 02:18 EET, between the dead-man check-in and her next ink-density verification, Diana's personal iPhone buzzed once — a single text from Răzvan: "tu ești bine?" ("are you okay?"). She replied: "da, lucrez. te iubesc." ("yes, working. I love you.") He didn't reply. She put the phone down. The texts were on her personal device + personal tenant; her work tablet logged only her dead-man check-in + her press telemetry; the personal exchange was in a separate boundary. This is the ADR-0311 doctrine working in lived practice — Diana's personal life is allowed into the night shift but does not cross-contaminate the work-tenant audit chain.
- The Antibiotice batch Diana ran was the third night-shift pharma-PIL run Tipografia had ever produced, and the first by Diana solo. The previous two were run by Vladimir. Diana's print quality on this run (ΔE max 2.7, final mean 1.1) was slightly tighter than Vladimir's average for the same batch class (his historical mean is 1.3). She does not yet know this; Mihai will mention it offhandedly in two weeks at a family dinner.
- The Skoda Octavia's gearbox makes a slight whine in 5th gear above 80 km/h. Diana has been meaning to take it to her mechanic in Mănăștur for 6 months. She'll take it next Wednesday. The bracelet Vladimir gave her sits on the dashboard during the drive home Tuesday morning; it catches the early light at a curve near Dej Highway and reflects briefly into her eye.

## §12 — Stop condition for this story

This story documents the lived texture of the 6-day final-onboarding-phase journey from Tuesday Jan 26's assessment through Tuesday Feb 2's first-solo-shift completion. The acceptance criteria in `README.md`, the API shapes in `handshake.md`, the test cases in `integration-test-plan.md`, and the schema files together encode the machine semantics. The story exists so that the next human or agent reading the codepath can understand WHY competency is a Cedar context (not a static flag), WHY night-shift authority must be separately gated from day-shift authority even for the same human, WHY the workplace-integration provisioning sequence couples shift schedule + geofence + payroll + alarm-cooperative scope as one transactional unit, WHY the lone-worker dead-man protocol's personal-tenant escalation contact is consented-not-defaulted, and WHY cross-journey persona continuity (Diana's j157 cert informing her j162 prerequisite check) is a first-class concern rather than an HR data-warehouse afterthought.
