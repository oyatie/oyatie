---
doc_class: User-Journey-Story
journey_id: j155-stefan-kovacs-college-night-shift-and-finals-week
date: 2026-05-20
authority_tier: 2
status: draft
---

# j155 — Story: Stefan Kovács, OSZK foyer guard desk, 21:48 CET Sunday

## Cast

| Role | Name | Tenant | Device |
|---|---|---|---|
| Night-shift security guard / student | Stefan Kovács | oszk-security-services_hu + bme-student-bodv75_hu + personal-stefan-kovacs-hu | Pixel 8a (personal phone) + Lenovo IdeaPad 5 14" (study laptop) + OSZK kiosk Dell Wyse 5070 |
| OSZK shift supervisor | Csilla Bartók | oszk-security-services_hu | OSZK desktop |
| OSZK HR officer | József Almásy | oszk-security-services_hu | OSZK desktop |
| OSZK rare-books archivist (on call) | Anna Lukács | oszk-security-services_hu | mobile |
| OSZK colleague (sick today) | Réka Hahn | oszk-security-services_hu | mobile (at home) |
| BME OS course lead | Dr. Gábor Halász | bme-student-bodv75_hu (instructor side) | desktop |
| BME OS TA | Mihály Pap (PhD candidate) | bme-student-bodv75_hu (TA side) | desktop |
| BME study-group classmate | Bálint Szabó | bme-student-bodv75_hu | Pixel 7 |
| Stefan's roommate (DV survivor — separate j-line) | Anikó Szász | personal-anikó-szász-hu | n/a in this story |
| Sleep-research PI | Dr. Eszter Boros (BME-Egészségtudomány) | bme-research-cohort-001-hu | desktop |

## Context

- Date: Sunday December 14, 2026, 21:48 CET — through Friday December 19, 13:00 CET
- Location: Országos Széchényi Könyvtár (OSZK), Buda Castle district VIII, Budapest H-1014 Szent György tér 4–6; back-office guard desk in the cellar level
- Critical exam: Operating Systems (course code VIK-AUT-VIIIAB1015), Tuesday Dec 16 08:00 CET, BME Building Q (Magyar tudósok körútja 2)
- Tuition installment due: Friday Dec 19 23:59 CET, HUF 187,500 (≈ €475)
- Tenants: `personal-stefan-kovacs-hu`, `oszk-security-services_hu`, `bme-student-bodv75_hu`
- Pack overlays: EU-GDPR, HU-Labour, EU-WTD, Bologna-academic-records, HU-Education-Act-CXC

## Beat-by-beat

### 21:48 CET, Sunday — Stefan arrives at the OSZK staff entrance

Stefan rings the staff buzzer at the south service door. The OSZK kiosk (Dell Wyse 5070 running the OSZK-branded oyatie work-tenant front-end) wakes up. Stefan taps his Pixel 8a to the NFC reader. The kiosk:

```
+--------------------------------------------------+
|  OSZK Biztonsági szolgálat                        |
|  Kovács István — éjszakai őr                      |
|  Műszak: 22:00 - 06:00                            |
|  Hét hónapnyi átlag: 22 óra/hét (cap: 48)         |
|                                                   |
|  [ Műszak megerősítése ]                          |
+--------------------------------------------------+
```

Active-tenant indicator at the top: **`OSZK Biztonsági szolgálat`** in OSZK navy. Stefan's Pixel briefly shows a tenant-switch-confirmation toast on its lock screen: *"Belépés az OSZK munkahelyi környezetbe — jóváhagyva"* (Entering OSZK work context — approved).

Csilla (shift supervisor) sits at the security control room, sees Stefan's clock-in event appear: `EVT-J155-CALENDAR-SHIFT-CONFIRM-001`.

### 21:53 CET — Stefan stows his bag in the staff locker

The lockers are in the basement cellar. The locker has a small electronic lock keyed to his OSZK employee badge. He puts in:

- Lenovo IdeaPad 5 14" laptop (his study laptop — for break-time only)
- Pixel 8a charging brick + cable
- Two textbooks: Tanenbaum *Modern Operating Systems* (Hungarian translation, 5th ed.) + a Discrete Math II practice exam packet printed off the BME course Drive
- His insulated thermos full of coffee from the Mongolian Bárka night cafe on Rákóczi út

He keeps on his person:

- His OSZK uniform shirt + duty belt (no firearm — Hungarian library guards carry only a maglite + radio)
- His Pixel 8a (active tenant: OSZK)
- The OSZK Motorola TLK-100 PTT radio

### 22:00 CET — Shift officially begins

Csilla's monitor shows the foyer cameras switch to "night mode" (1lux IR). Stefan walks the four-corner check:

1. Main reading-room foyer — locked, alarm armed → ✓
2. Rare-books vault outer door — sealed, motion sensors green → ✓
3. Manuscript-collection corridor — alarm green → ✓
4. Staff entrance — locked, badge log clean → ✓

Audit: `EVT-J155-WORKPLACE-PERIMETER-CHECK-001` × 4 (one per corner).

He returns to the guard desk. The Wyse 5070 shows a 30-second tickertape of next-shift handoff notes from Réka, who finished her Friday shift on Dec 12: *"szelet papírba csomagolt karácsonyi süti hagytam a hűtőben, vidd, ha akarod"* — a sweetly mundane note about leftover Christmas cookies in the fridge.

### 22:14 CET — Réka's shift-swap offer

Stefan's Pixel chimes — OSZK messenger notification. Réka:

> Szia Stefán, kérlek-kérlek vedd át a keddi műszakomat? 22-06. Influenza, lázam 38.7. Nagyon-nagyon hálás lennék 🙏

(Hi Stefan, please please cover my Tuesday shift? 22-06. Flu, fever 38.7. I'd be incredibly grateful 🙏)

Stefan thinks. Tuesday is his OS final at 08:00. A 22:00→06:00 shift Monday night means he'd be coming off shift exactly at 06:00, then needing to be exam-ready in 2 hours after a sleepless night. He can't.

He drafts a reply in the OSZK messenger client (active tenant: `oszk-security-services_hu`):

> Réka, nagyon sajnálom, kedden nem tudok. Csütörtök reggel ki tudok jönni helyetted, ha az segít. Jobbulást!

(Réka, I'm so sorry, I can't on Tuesday. I can come in Thursday morning if that helps. Get well!)

**Critical UX moment**: Stefan does NOT mention his OS final. The OSZK messenger compose surface is gated by the work tenant; the BME exam-schedule data would only be accessible if he switched tenant — which he doesn't, because that information is private to him and shouldn't leak into the OSZK audit chain. Cedar would deny the cross-tenant probe anyway; but more importantly Stefan has been trained to keep the contexts separate and the UI helps him: the compose field shows a small lock pill *"OSZK environment — your BME tenant is not connected here"*.

He sends. Audit: `EVT-J155-MESSENGER-SWAP-DECLINED-002`.

Réka replies a few minutes later: *"köszi, megértem, csütörtök az jó lenne! 🙏"* (thanks, understood, Thursday would be great!). Stefan accepts the Thursday swap via a separate calendar interaction.

### 22:18 CET — Stefan switches to study mode (BME tenant)

OSZK policy permits up to 2 hours of personal-device study time during a 22:00–06:00 shift if all alarms are green AND no incident is active. Stefan checks: green and green.

He pulls the IdeaPad from the locker, opens it on the guard desk (the camera feed is still visible on the Wyse 5070 to his left), and powers on. The IdeaPad's lockscreen prompts for passkey via Stefan's USB-C YubiKey 5C NFC.

After unlock, the active-tenant pill at the top of the screen shows the current state: `Personal — Stefan Kovács`. Stefan clicks the pill. A modal:

```
+--------------------------------------------------+
|  Aktív környezet váltása                          |
|                                                   |
|  Jelenlegi: Személyes (személyes felhasználó)    |
|                                                   |
|  Választható környezetek:                         |
|   ○ BME hallgató — bme-student-bodv75_hu         |
|   ○ OSZK munkavállaló — oszk-security-services_hu|
|                                                   |
|  Tartsd nyomva 2 mp-ig: [ BME hallgató ]         |
+--------------------------------------------------+
```

He long-presses the BME button for 2 seconds (anti-accidental-switch pattern). The pill flips to **`BME hallgató — Kovács István · Második évfolyam`**. The desktop background shifts from a personal photo of his late grandmother's garden to the BME purple-and-yellow wordmark.

Audit: `EVT-J155-IDENTITY-TENANT-SWITCH-003` (sealed in personal tenant + BME tenant; NOT in OSZK tenant — the OSZK tenant has no business knowing that Stefan switched contexts on his break).

### 22:21 CET — Stefan opens BME LMS — OS course

URL: `https://lms.bme.hu/courses/VIK-AUT-VIIIAB1015/oszi-2026`

The BME learning-management surface loads. Active-tenant pill: BME. Stefan clicks "Past exams archive". A list of 14 past finals (Spring 2020 → Spring 2026). He opens the **Spring 2026 final** + **Fall 2025 final** in two tabs.

The LMS records the access events as `EVT-J155-LMS-NOTES-READ-004` × 2 (one per file open). Sealed in BME tenant. NOT in OSZK tenant.

### 22:38 CET — `#os-finals-2026` study channel — Bálint asks a question

Stefan opens the BME community client (still on IdeaPad, BME tenant). The `#os-finals-2026` MLS-encrypted channel shows 47 members + 3 unread.

Bálint Szabó (a classmate from the practical-lab cohort) just posted:

> Sziasztok, valaki emlékszik, hogy Halász tanár úr a memóriafedett laphibák kezelését hogyan kérdezte tavalyelőtt? A Tanenbaum 5e 3.6.2 vagy a Silberschatz 10e 9.7?

(Hi all, anyone remember how Prof Halász asked about page-fault handling under copy-on-write the year before last? Tanenbaum 5e §3.6.2 or Silberschatz 10e §9.7?)

Stefan checks his Fall-2024 archive note: it was Tanenbaum's framing, with the optimization angle on private/shared mappings. He posts:

> Tanenbaum 5e 3.6.2, de Halász mindig hozzáfűzi a `madvise(MADV_DONTNEED)` kérdést — érdemes átnézni a glibc oldalt is.

(Tanenbaum 5e §3.6.2, but Halász always adds the `madvise(MADV_DONTNEED)` question — worth reviewing the glibc page too.)

Audit (BME tenant only): `EVT-J155-COMMUNITY-POST-STUDENT-007`. Bálint hearts the post. Two other classmates respond. Stefan reads for 10 minutes.

### 22:50 CET — Csilla's intercom

Csilla (the supervisor) buzzes the guard-desk intercom:

> *"Stefán, az ablakon kintről egy mókus megint próbálkozik a manuscript folyosó ablakával. Megnézted? Nem riasztó, csak ellenőrzés."*

(Stefan, that squirrel is trying the manuscript-corridor window again from outside. Did you check? Not an alarm, just verifying.)

Stefan acknowledges. He **does not switch tenants** — the supervisor's call comes through the OSZK PTT radio, which is bound to the work-tenant principal regardless of which tenant his laptop is in. He walks the manuscript corridor: empty. The squirrel has given up. He returns.

This beat demonstrates the **device-vs-tenant distinction**: his IdeaPad is in BME tenant for study; the OSZK radio is in OSZK tenant for work duties. Both are concurrent without bleed. Audit: `EVT-J155-WORKPLACE-PERIMETER-CHECK-INTERIM-009`.

### 23:14 CET — Stefan reads OS chapter 7 (deadlocks)

Back at the desk, IdeaPad still in BME tenant. Stefan reads Tanenbaum §6.5 (deadlock recovery). The LMS records two more `EVT-J155-LMS-NOTES-READ-004`. He highlights three sentences; the highlights save to his BME drive, sealed under `EVT-J155-DRIVE-ANNOTATE-NNN`.

### 23:48 CET — A small foyer sensor alert (false positive)

The Wyse 5070 (still on the desk, in OSZK tenant) beeps. Foyer pressure-mat alert: someone walked across it. Stefan's eyes snap to the OSZK monitor. The camera shows: nothing. Then a small movement — a mouse. Library mouse, well-fed, indifferent to security systems.

Stefan logs the false-positive: he taps "False positive — mouse" on the OSZK incident-management quick-tile. This DOES emit an event because foyer-sensor data is sealed regardless:

```json
{
  "event_class": "EVT-J155-INCIDENT-FALSE-POSITIVE-MOUSE-LOG-010",
  "tenant_id": "oszk-security-services_hu",
  "occurred_at": "2026-12-14T23:48:21+01:00",
  "subject_principal": "stefan.kovacs.work@oszk-security-services_hu",
  "payload": {
    "sensor_id": "pressure-mat-foyer-northwest-001",
    "category": "fauna_false_positive",
    "subcategory": "mouse",
    "stefan_handling": "logged_and_continued"
  },
  "emitting_microservice": "incident-management"
}
```

Stefan smiles, scratches a tally on a sticky note (this is the third mouse this month), and goes back to deadlocks.

### Monday 02:30 CET — A real concentration block

Stefan studies straight from 23:50 to 02:30. He works through:

- Tanenbaum §6.4 (deadlock avoidance / Banker's algorithm — 3 worked problems)
- Tanenbaum §6.5 (deadlock detection — 2 problems)
- Silberschatz §8.4 cross-reference (deadlock states formalism — 1 problem)

He posts another question to `#os-finals-2026` at 01:14 about whether Halász counts the Banker's algorithm as a "deadlock avoidance" or "deadlock prevention" technique (the textbooks disagree slightly). Mihály Pap (the TA) replies 8 minutes later: "Avoidance per Halász's framing — that's been the rubric for 3 years running."

Audit: `EVT-J155-COMMUNITY-POST-STUDENT-007`.

### Monday 02:30 CET — Brief perimeter walk

Stefan walks the four corners. All green. Back to desk. He resists the urge to switch back to OSZK to check Anikó's roommate-tenant messenger (he hasn't heard from Anikó since Saturday — but he keeps boundaries). Anikó's tenant is a separate B2C tenant under j04-class shelter-mode protections; even if he wanted to "ping her", the proper path is via his personal tenant, not from a work or student device.

### Monday 04:15 CET — Stefan dozes (allowed)

OSZK policy permits seated rest at the desk between 03:00–05:00 if all alarms are armed and all four corners checked in the previous hour. Stefan sets a vibration alarm on his Pixel for 04:35 (the next required walk). He closes his eyes.

The Pixel's biometric watch sensor (an opt-in BME student-wellness feature he enrolled in during semester 1) captures:

- Heart rate: 64 bpm (slow → drowsy)
- Body temperature: 36.6°C
- Sleep stage (predicted): light → REM transition
- Bedtime hour: irregular (3rd irregular night in 7-day window)

This data flows directly to the BME observability pipeline (research cohort `cohort-2026-sleep-grade-fall`) — never to OSZK. The pipeline immediately anonymizes:

- Stefan's principal ID is hashed against the cohort's per-study salt
- The data point is added to the cohort distribution
- Cohort PI Dr. Eszter Boros can later query "average light-sleep duration during finals week N=247 students" but never "Stefan's sleep last night"

Audit (BME research tenant, replicated to personal tenant under Stefan's consent): `EVT-J155-OBSERVABILITY-SLEEP-GRADE-EMIT-008`.

### Monday 04:35 CET — Vibration alarm — perimeter walk

Stefan wakes, walks the corners (all green), returns. He studies until 06:00. Audit: `EVT-J155-WORKPLACE-PERIMETER-CHECK-019` (cumulative 19th of shift).

### Monday 06:00 CET — Shift ends, Stefan goes home

At the kiosk, he taps his Pixel to NFC clock-out:

```
+--------------------------------------------------+
|  Műszak vége                                      |
|  Kovács István — éjszakai őr                      |
|  Műszak hossza: 8 óra 0 perc                      |
|  Heti óraszám új átlag: 22.4 óra                  |
|  Pihenőidő minimum (EU-WTD): következő 11 óra     |
|                                                   |
|  [ Kijelentkezés ]                                |
+--------------------------------------------------+
```

He taps **Kijelentkezés** (Clock out). The Wyse 5070 calculates: next-eligible-shift-start = 17:00 CET (06:00 + 11 hr WTD minimum). Audit: `EVT-J155-WORKPLACE-CLOCK-OUT-020`.

He takes Metro line 2 from Batthyány tér → Deák Ferenc tér → Metro line 3 → Újpest-központ, home by 06:42, asleep by 07:15.

### Monday 12:00 CET — Stefan wakes, eats, studies more (Pixel on personal tenant, IdeaPad on BME tenant)

He studies until 18:00. He goes to bed at 22:00 to be rested for Tuesday's exam.

### Tuesday Dec 16 06:30 CET — Exam morning

Stefan wakes, dresses, eats a slice of cold pizza, takes Metro line 3 → BME Q building. At 07:55 he sits in lecture hall Q-002. Active-tenant pill on his Pixel (which he must surrender at the exam-room door per BME proctoring rules — but the device stays passively in airplane mode on the proctor's desk) is BME.

Exam at 08:00 CET. 90 minutes. 6 questions. Stefan answers 5 cleanly. Question 4 (Banker's algorithm worked problem) he gets stuck on briefly but recovers using the rubric he posted about Sunday night. He hands in his bluebook at 09:24.

Audit emitted by BME exam-system (not in scope of this journey's primary µservices, but worth noting): `EVT-BME-EXAM-SUBMIT-VIK-AUT-VIIIAB1015-S2026-stefan-kovacs`.

### Tuesday afternoon — Stefan checks the OSZK schedule

After lunch, Stefan opens his Pixel in OSZK tenant. He confirms his next shift (Thursday morning swap for Réka — 06:00 → 14:00). Audit: `EVT-J155-CALENDAR-SHIFT-CONFIRM-021`.

He also notices a notification from José Almásy (OSZK HR): the December payroll will run tonight, and his payroll-deduction tuition-bridge transfer to BME is set to process.

### Tuesday 21:00 CET — Payroll bridge processes the tuition payment

OSZK's monthly payroll runs at 21:00 CET on the 16th of each month. Stefan's net pay this month is HUF 312,400 (after taxes + social-security contribution). The bridge to BME processes:

1. ADP-Streamline-HU (workplace-integration) computes Stefan's net: HUF 312,400
2. The bridge consults Stefan's personal-tenant standing instruction: *"Auto-pay BME tuition installment from OSZK payroll on payday; cap HUF 200,000"*
3. The bridge ALSO consults the BME student-tenant active billing record: *"Installment 3 of 4 due Dec 19; HUF 187,500; reference number TR-2026-W-bodv75-3-of-4"*
4. The bridge runs Cedar 3-way:
   - OSZK side: `payments.payroll_deduct` against employee record — permit (because Stefan signed the auto-deduction form in October)
   - Personal side: `payments.standing_instruction_execute` — permit (instruction matches; amount is within cap)
   - BME side: `payments.tuition_credit_received` — permit (active enrolment, valid invoice)
5. SEPA transfer initiated: OSZK B&MD bank account → BME Magyar Államkincstár ledger; HUF 187,500
6. Stefan's personal tenant receives a confirmation: net pay HUF 124,900 (= 312,400 − 187,500) lands in his personal MKB Bank account
7. BME's billing tenant marks installment 3 paid; balance: HUF 187,500 remaining (last installment due Spring 2027)

Audit events (cross-tenant trace ID `tr-payroll-bridge-2026-12-16-stefan`):

- `EVT-J155-PAYMENTS-PAYROLL-NET-COMPUTED-022` (oszk-security-services_hu)
- `EVT-J155-PAYMENTS-STANDING-INSTRUCTION-MATCH-023` (personal-stefan-kovacs-hu)
- `EVT-J155-PAYMENTS-TUITION-PAYROLL-BRIDGE-006` (sealed in all 3 tenants under the same trace_id)
- `EVT-J155-PAYMENTS-TUITION-CREDIT-024` (bme-student-bodv75_hu)
- `EVT-J155-PAYMENTS-NET-LANDED-PERSONAL-025` (personal-stefan-kovacs-hu)

Stefan's Pixel emits a quiet pip + a notification in personal-tenant style: *"Net fizetés HUF 124 900 megérkezett az MKB-számládra. Tandíj részlet 3/4 (HUF 187 500) automatikusan teljesítve."*

He smiles. He sleeps.

### Wednesday Dec 17 — Recovery day + Discrete Math study

Stefan does not work Wednesday (his usual rest day). He studies Discrete Math II in his apartment.

### Thursday Dec 18 06:00–14:00 CET — Réka's swapped shift

Stefan covers Réka's morning shift (the swap they negotiated). Day shift = no study window per OSZK policy (day shifts are higher tempo). All four corners clean throughout. He clocks out at 14:00. Audit: `EVT-J155-WORKPLACE-CLOCK-OUT-NNN`.

### Thursday evening — Discrete Math II study with the `#dm-finals-2026` channel

Same BME community pattern as Sunday night, different channel (`#dm-finals-2026`).

### Friday Dec 19 08:00 CET — Discrete Math II final at BME

Stefan sits the exam. 120 minutes. He answers all 8 questions. Hands in at 09:42.

### Friday Dec 19 12:30 CET — Stefan checks his BME tuition status

Active tenant: BME. URL: `https://billing.bme.hu/student/bodv75/account`. The page shows:

```
Tandíj 2026/27 őszi félév
  Részlet 1/4 — HUF 187 500 — TELJESÍTVE 2026-10-16
  Részlet 2/4 — HUF 187 500 — TELJESÍTVE 2026-11-16
  Részlet 3/4 — HUF 187 500 — TELJESÍTVE 2026-12-16
  Részlet 4/4 — HUF 187 500 — esedékes 2027-02-15

Aktív státusz: Hallgató — második évfolyam — beíratva tavaszi 2027 félévre
```

Audit: `EVT-J155-LMS-BILLING-VIEW-NNN`.

### Friday Dec 19 13:00 CET — Stefan logs off

He closes the IdeaPad. The BME tenant deactivates. He walks to a kávézó on Móricz Zsigmond körtér for a celebratory turkey sandwich and an espresso.

## Finals-week mode (Mon Dec 14 → Fri Dec 19)

A meta-feature: when Stefan tags a calendar event as `category=academic_final`, the BME tenant emits a hint to the personal tenant (via consent-graph) suggesting **finals-week mode**: non-emergency notifications across both tenants are paused; only OSZK alarms + family-emergency channels break through. Stefan can opt in/out per finals window. He opted in Dec 1 for this period.

Audit (personal tenant): `EVT-J155-NOTIFICATIONS-FINALS-MODE-ACTIVE-NNN`.

## What did NOT happen

- OSZK admin (Csilla, József) never saw Stefan's BME study activity, exam schedule, or tuition status
- BME LMS never saw Stefan's OSZK shift schedule, paycheck amount, or work activity (except through the payroll-bridge handshake, which only exposes the tuition-credit event, not Stefan's salary)
- Sleep-grade telemetry never reached OSZK; it stayed in the BME research cohort and was anonymized
- Stefan never accidentally posted to `#os-finals-2026` while in OSZK tenant (would have failed Cedar; the UX prevents the attempt by showing a "wrong tenant" badge)
- Réka never learned why Stefan declined the Tuesday swap (the OS final was private information)
- The OSZK paycheck never landed in Stefan's account in full; the auto-deduction is the contract he signed and the bridge enforces it deterministically

## Audit-event chain sequence (sealed)

| # | Event class | Tenant |
|---|---|---|
| 001 | EVT-J155-CALENDAR-SHIFT-CONFIRM | oszk-security-services_hu |
| 002 | EVT-J155-MESSENGER-SWAP-DECLINED | oszk-security-services_hu |
| 003 | EVT-J155-IDENTITY-TENANT-SWITCH | personal + bme |
| 004 | EVT-J155-LMS-NOTES-READ | bme-student-bodv75_hu |
| 005 | EVT-J155-CEDAR-DENY-CROSS-TENANT-LMS-PROBE (when OSZK admin tries) | oszk-security-services_hu (with cross-tenant denial seal in BME) |
| 006 | EVT-J155-PAYMENTS-TUITION-PAYROLL-BRIDGE | all 3 tenants |
| 007 | EVT-J155-COMMUNITY-POST-STUDENT | bme-student-bodv75_hu |
| 008 | EVT-J155-OBSERVABILITY-SLEEP-GRADE-EMIT | bme-research-cohort + personal |
| 009 | EVT-J155-WORKPLACE-PERIMETER-CHECK-INTERIM | oszk-security-services_hu |
| 010 | EVT-J155-INCIDENT-FALSE-POSITIVE-MOUSE-LOG | oszk-security-services_hu |
| 020 | EVT-J155-WORKPLACE-CLOCK-OUT | oszk-security-services_hu |
| 021 | EVT-J155-CALENDAR-SHIFT-CONFIRM (Thursday swap) | oszk-security-services_hu |
| 022 | EVT-J155-PAYMENTS-PAYROLL-NET-COMPUTED | oszk-security-services_hu |
| 023 | EVT-J155-PAYMENTS-STANDING-INSTRUCTION-MATCH | personal-stefan-kovacs-hu |
| 024 | EVT-J155-PAYMENTS-TUITION-CREDIT | bme-student-bodv75_hu |
| 025 | EVT-J155-PAYMENTS-NET-LANDED-PERSONAL | personal-stefan-kovacs-hu |

All events seal under ADR-0263. The cross-tenant trace_id for the payroll bridge (`tr-payroll-bridge-2026-12-16-stefan`) links events 022–025 into a verifiable cross-tenant chain without leaking any tenant's internal data to another tenant's audit reader.
