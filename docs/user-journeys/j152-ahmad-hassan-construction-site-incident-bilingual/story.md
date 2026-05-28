---
doc_class: User-Journey-Story
journey_id: j152-ahmad-hassan-construction-site-incident-bilingual
date: 2026-05-20
authority_tier: 2
status: draft
---

# j152 — Story: Ahmad Hassan, 6th-floor deck, 14:37 PDT

## Cast

| Role | Name | Device | Language |
|---|---|---|---|
| Site Lead | Ahmad Hassan | Kyocera DuraForce Pro 3 (Android 14, rugged Bluetooth glove-pen) | AR (L1 Levantine) + EN (C1) |
| Affected worker | Khalil Mansour | Kyocera DuraForce E7 | AR (L1 Egyptian) |
| Crane operator | Roberto Santos | Cab-tethered cabin tablet | ES (L1 Mexican) + EN (B1) |
| Foreman | Bryan O'Connor | Mobile iOS 19 | EN (L1) |
| Halcyon HSE officer | Priya Mehrotra | Desktop + mobile | EN (L1) |
| First responder (paramedic) | Marcus Tate (AMR Oakland Unit 3-Charlie) | AMR ePCR tablet | EN |
| Halcyon HR (Paycom) | Sandra Velez | Desktop browser | EN + ES |
| State Fund claim adjuster | Yuki Tanaka | Carrier-side desktop | EN |
| Cal/OSHA Area Office Oakland | duty officer rotation | shared inbox | EN |

## Site context (frozen snapshot at 14:36:00 PDT)

- Site: Halcyon Build LLC project `HB-OAK-4421` — 9-story residential + ground-floor retail, 4421 Telegraph Ave, Oakland CA 94609
- Phase: structural steel + concrete deck pour day-cycle, deck 6 active
- Crew on deck 6: 8 (Ahmad + 7 fitters)
- Crew on adjacent decks: 11 (decks 4, 5, 7) — within stop-work broadcast reach
- Tower crane: Liebherr 280EC-H 12 Litronic; load-pin sensor `crane-LB-280-S01-loadpin` streaming at 50Hz to `crane.load_pin.sensor_v1`
- Deck cameras (live): `cam-deck-6-northwest`, `cam-deck-6-southeast`, `cam-deck-7-overlook` (4K H.265, 30fps, 4-minute ring buffer)
- Weather: clear, 68°F, wind 8 mph WSW (within crane operating envelope)
- Tenant: `halcyon_build_llc` (B2B_TENANT)
- Cell placement: `us-west-2-primary` + edge cell `us-west-2-edge-oakland`

## Beat-by-beat timeline

### 14:36:11 PDT — Crane lift in progress

Crane lifts a 3.6m × 800kg rebar bundle from the ground stockpile, tracking to deck 6 northwest grid line J-7. Load-pin telemetry stream `crane.load_pin.sensor_v1` shows nominal sequence: 7,840 N → 7,860 N. Span trace `lift-trace-2026-1014-14h36-LB-280-S01-lift-9217` open.

### 14:37:08 PDT — Sling slip

Load-pin telemetry shows a 47ms transient — a 1,200N step-down spike — when the bundle's outer band shears against the inner sling wrap. Bundle slips ~14cm laterally as it crosses the deck-6 perimeter. Roberto Santos in the crane cab sees the wobble on his cabin tablet (the load-pin graph spikes red).

### 14:37:11 PDT — Strike

The leading 60cm of the bundle clips Khalil Mansour on the right shoulder as he is repositioning a tie-off anchor 2.4m inside the deck-6 perimeter at grid line J-7. Khalil falls onto his right side, hits his scalp on the lip of the formwork. Bundle settles to the deck without crushing him. He is conscious. Bleeding from scalp. Right shoulder dislocation visually obvious.

### 14:37:14 PDT — Ahmad's first words

Ahmad is 6 meters away, near the crane signaler station. He shouts in Arabic first (instinctive — his crewmate is Arabic):

> "خليل! خليل، هل تسمعني؟ ابقَ مكانك!"
> ("Khalil! Khalil, do you hear me? Stay where you are!")

Then in English, loud, for everyone else on the deck:

> "Stop work. Stop the lift. Get the kit."

### 14:37:18 PDT — Ahmad's DuraForce — Home screen

Ahmad pulls his DuraForce out of his hi-vis chest pocket. The lock screen is face-up. He hits the **physical lower-right "SOS" hardware button** that Halcyon Build provisioned at device enrollment. The SOS button is bound to the `incident-management` panic intent. (Bound at MDM enrollment; ADR-0317 role-projection puts the SOS surface on every site-lead device.)

### 14:37:19 PDT — Step-up auth

The DuraForce launches the **Incident** screen and immediately demands passkey step-up. Ahmad's WebAuthn passkey is bound to his fingerprint + the device's hardware secure element. He taps the fingerprint sensor on the back of the device with his right index. Step-up succeeds at `14:37:20.812 PDT`. The 120-second step-up freshness window opens.

Cedar permit evaluated: `incident.create` against `Site::"HB-OAK-4421"`. Principal: `User::"ahmad.hassan@halcyon-build.com"`. Result: **permit**. Audit event: `EVT-J152-IDENTITY-STEPUP-OK-001` emitted to `audit-chain` (sealed at 14:37:20.871).

### 14:37:22 PDT — Incident-create form, top of screen

The form opens with these field clusters:

1. **Where** — pre-filled `Site: HB-OAK-4421 · Deck 6 · Grid J-7` via the device's site-geofence + the live workforce-presence projection from the workforce ontology
2. **Who** — Ahmad taps "Affected worker" and his crew roster appears with photos; he taps Khalil Mansour's tile
3. **What** — three big-thumb tiles: NEAR-MISS · FIRST-AID · ESCALATE-TO-911
4. **When** — pre-filled with `14:37:11 PDT` from the load-pin spike correlation (the system already correlated the strike-instant to the load-pin transient)

### 14:37:24 PDT — Ahmad picks ESCALATE-TO-911

Tap on `ESCALATE-TO-911` opens a confirmation modal with two giant buttons:

- "CALL 911 NOW" (red, primary)
- "ALREADY CALLED" (gray, secondary)

He taps "CALL 911 NOW". The DuraForce dials 911 via the device's cellular radio (T-Mobile FirstNet primary, Verizon secondary). The dial-out is logged but its audio is NOT recorded by Halcyon Build (legal — California requires two-party consent; the 911 PSAP recording is the legal record). Audit event: `EVT-J152-IDENTITY-911-DIAL-002` sealed with the call's start timestamp, the FirstNet IMSI, and the PSAP cell-ID, but no audio.

### 14:37:31 PDT — 911 call connects to Oakland PSAP

Ahmad on speakerphone, in English (he is C1 capable; the PSAP dispatcher is English-only):

> Ahmad: "Construction site, 4421 Telegraph Ave, Oakland. Sixth floor deck. Worker hit on the shoulder by a rebar bundle. Conscious. Bleeding from the head. Right shoulder dislocation. I am the site lead."
>
> Dispatcher: "AMR is rolling. Stay with him. Is the area secure?"
>
> Ahmad: "Stopping work now. Crane is parked."

The 911 audio is on the PSAP system; the Halcyon Build incident record just notes `911_dispatch_acknowledged_at = 14:37:42 PDT, dispatch_unit = AMR-Oakland-Unit-3-Charlie`.

### 14:37:44 PDT — Ahmad triggers stop-work broadcast

Ahmad swipes back to the incident form and taps the secondary action **STOP-WORK BROADCAST**. A modal asks the language pair. He taps `EN + AR + ES` (Halcyon Build pre-configured the three site languages; the system never auto-detects — language is an admin-defined site attribute, per the EEOC overlay).

The broadcast goes to `messenger` channel `site-hb-oak-4421-deck-6` plus the adjacent decks 4, 5, 7. The message body:

- EN: "STOP WORK. Deck 6 incident. Stay clear of J-7. Site Lead Ahmad will direct."
- AR (right-to-left): "أوقفوا العمل. حادث في الطابق السادس. ابتعدوا عن المربع J-7. سيوجهكم القائد أحمد."
- ES: "ALTO TOTAL. Incidente piso 6. No se acerquen a J-7. El líder Ahmad dirigirá."

`messenger` API: `POST /v1/channels/site-hb-oak-4421-deck-6/broadcast` with `multi_lang=true`, `urgency=stop_work`. 19 devices receive within 8 seconds (median 2.1s, p99 7.4s). 19 ACKs received within 14 seconds. Audit events `EVT-J152-MSG-STOPWORK-FANOUT-003` and 19× `EVT-J152-MSG-STOPWORK-ACK-NNN` sealed.

### 14:37:55 PDT — Ahmad walks to Khalil

Ahmad is 6 meters from Khalil. He kneels next to him, in Arabic Levantine register softening to Khalil's Egyptian:

> "خليل، أنا معك. سيارة الإسعاف في الطريق. لا تتحرك."
> ("Khalil, I'm with you. Ambulance is on the way. Don't move.")

Khalil, eyes open, in Egyptian Arabic:

> "صدري وكتفي... ما أقدر أحرّك يدي اليمنى."
> ("My chest and shoulder... I can't move my right arm.")

Ahmad uses his left hand to hold the device and his right to apply gauze from the deck-6 first-aid kit to Khalil's scalp. With his thumb (one-handed), he dictates into the **DuraForce voice-note** field on the incident:

- Voice-note locale: device locale is Arabic (Ahmad's primary). The transcription pipeline routes to the Arabic ASR endpoint. The voice-note appears in the `narrative_ar` field. Audit event: `EVT-J152-INCIDENT-NARRATIVE-VOICE-AR-005`.

He then taps the language toggle (a small EN/AR glyph in the top-right of the voice-note widget) and dictates a second voice-note in English summarising the same facts. This appears in `narrative_en`. (Per the README spec, the narrative pair is stored as two structured fields, not concatenated.) Audit event: `EVT-J152-INCIDENT-NARRATIVE-VOICE-EN-006`.

### 14:38:02 PDT — Auto-attachment: crane telemetry + camera footage

The `incident-management` service, on `incident.create`, fires an automatic side-effect (orchestrated by `workflow-engine` step 3 of 11):

- Pulls 90 seconds of `crane.load_pin.sensor_v1` data centred on `14:37:11 PDT` (45s pre, 45s post). 4,500 samples. CSV format. SHA-256 hash sealed in `audit-chain`.
- Pulls 4 minutes of `cam-deck-6-northwest` footage centred on `14:37:11 PDT` (2 min pre, 2 min post). H.265, 30fps. SHA-256 hash sealed.
- Pulls the corresponding clip from `cam-deck-6-southeast` for cross-angle. Same hashing.

Each attachment is stored in `drive` under the incident record's evidence folder `INC-2026-1014-HB-OAK-4421-0007/evidence/`. Drive enforces the **chain-of-custody** policy: once attached to an open incident, the file is read-only and any access emits `EVT-J152-DRIVE-EVIDENCE-ACCESS-NNN`. Even Halcyon Build's CEO cannot delete or modify.

### 14:38:18 PDT — ADR-0298 medical bypass

Ahmad asks Khalil (Arabic): "هل عندك حساسية لأي دواء؟" ("Are you allergic to any medication?"). Khalil, slurring: "السلفا... والكوديين..." ("Sulfa... and codeine..."). Ahmad needs to make sure the responding paramedic sees this within 60 seconds.

Ahmad taps the **Pull Worker Medical (Acute Window)** action on the incident form. A red confirmation modal:

- EN: "This invokes the ADR-0298 emergency bypass. Only allergies + current medications will be disclosed, only to the incident record, only for the next 60 minutes. This action is recorded. Continue?"
- AR: "هذه الخطوة تستخدم استثناء حالة الطوارئ ADR-0298. سيتم الكشف فقط عن الحساسية والأدوية الحالية، فقط لسجل الحادث، فقط للستين دقيقة القادمة. هذا الإجراء مسجَّل. متابعة؟"

Ahmad taps **Continue**. Cedar evaluates:

```
principal = User::"ahmad.hassan@halcyon-build.com"
action = Action::"incident.attach_medical_excerpt"
resource = Site::"HB-OAK-4421"
context = {
  affected_worker_id: "khalil.mansour@halcyon-build.com",
  adr_0298_bypass_active: true,
  acute_window_minutes: 1,
  step_up_seconds_ago: 58,
  consent_token_present: true,
  consent_token_scope: "allergy_excerpt"
}
```

Result: **permit**. The `drive` service projects exactly two fields from Khalil's medical record: `allergies = ["sulfa", "codeine"]` and `current_medications = []`. These are attached to the incident record as a narrow structured object, NOT the full medical PDF. Audit event: `EVT-J152-DRIVE-MED-EMRG-DISCLOSE-007` sealed with the projection's bytes-hash, the bypass justification, and the 60-minute window expiry timestamp.

### 14:42:14 PDT — AMR Unit 3-Charlie arrives

Marcus Tate from AMR pulls his ePCR tablet. He has a one-time-use **first-responder share link** that the dispatcher's CAD generated when the 911 call was logged. He taps the link. The link surfaces the narrow allergy excerpt (read-only, watermarked with Marcus's PSAP-issued identity) for 60 minutes. He sees `sulfa, codeine`. He documents in his ePCR.

Audit event: `EVT-J152-CONNECT-EMS-EXCERPT-VIEW-008` sealed with the first-responder identity, the watermark hash, the access time.

### 14:48:31 PDT — Khalil transported

Marcus packages Khalil onto a backboard with C-collar and shoulder splint. They transport him to Highland Hospital Trauma Center (Oakland). Marcus updates the incident record from his ePCR (the bridge accepts an HL7 FHIR `Encounter` resource on the AMR side, transformed to the incident's `transport_log` field). Audit: `EVT-J152-INCIDENT-TRANSPORT-LOG-009`.

### 14:50:00 PDT — Cal/OSHA §342 8-hour timer set

The `workflow-engine` step 6 fires. The incident is provisionally classified as `severe_injury` (right-shoulder dislocation + scalp laceration; meets §342 "loss of consciousness, serious injury, or substantial part of the body" criterion conditionally on hospital admission). The 8-hour reporting timer is set to expire `22:37:11 PDT`. A T-6h reminder fires at `20:37:11 PDT` to Priya Mehrotra (Halcyon HSE) and an escalation at T-8h fires to the safety officer's pager.

Audit: `EVT-J152-WORKFLOW-TIMER-SET-010`.

### 14:54:08 PDT — Workplace-integration sync to Paycom

`workplace-integration` step 7 runs. It maps the incident fields to Paycom's `Employee Injury Report` object:

| Incident field | Paycom field | Transform |
|---|---|---|
| `affected_worker_id` | `EmployeeID` | lookup via federation |
| `narrative_en` | `IncidentDescription` | direct |
| `narrative_ar` | `IncidentDescriptionAlt` | direct |
| `incident_class` | `InjurySeverity` | enum map |
| `occurred_at` | `IncidentDateTime` | direct |
| `site_id` | `WorkLocationCode` | lookup |
| `transport_log.destination` | `TreatmentFacility` | direct |
| `oshe_recordable_provisional` | `RecordableFlag` | direct |

Paycom returns HTTP 201 with `paycom_injury_report_id = PCM-EIR-49217`. Audit: `EVT-J152-WORKPLACE-PAYCOM-WRITE-011`.

### 14:55:42 PDT — Workplace-integration sync to State Fund

Same step derives the **FROI-1** (First Report of Injury, California DWC Form 5020) automatically from the incident + Paycom employee record. The FROI-1 fields are populated:

- Employer: Halcyon Build LLC, DIR# 1234567
- Worker: Khalil Mansour (DOB redacted-but-present)
- Injury date/time: 14:37:11 PDT 2026-10-14
- Injury location: 4421 Telegraph Ave, Oakland CA 94609
- Body part: right shoulder + head
- Cause: struck by falling object (rebar bundle)
- Initial treatment: AMR transport, Highland Hospital

The FROI-1 is submitted to State Fund via the `connector` bridge using their FROI EDI 148 endpoint. State Fund returns acknowledgement code `SF-FROI-ACK-2026-10-14-49217`. Audit: `EVT-J152-WORKPLACE-STATEFUND-FROI-012`.

### 15:14 PDT — Priya Mehrotra reviews

Priya, Halcyon's HSE officer, opens the incident dashboard on her desktop. She sees the incident record, the bilingual narrative pair, the auto-attached telemetry + camera evidence, the AMR transport log, the Paycom + State Fund acknowledgements, the live §342 timer (currently T-7h14m). She does NOT need to do anything urgent. She adds a follow-up task: "Inspect the crane sling — the outer-band shear pattern needs root-cause analysis." Audit: `EVT-J152-INCIDENT-FOLLOWUP-TASK-013`.

### 16:02 PDT — Ahmad's role-projection cross-check (ADR-0317)

Ahmad opens the same incident on his DuraForce. The role-projection layer (ADR-0317) shows him only the site-lead surface — not the HR-officer surface, not the carrier-claims surface. He can edit narrative, add witnesses, attach further photos. He cannot see Khalil's salary, his SSN, or his full medical history. The view is named `site_lead_incident_v1`.

Audit (on view, not edit): `EVT-J152-INCIDENT-VIEW-SITELEAD-014`.

### 17:11 PDT — Highland Hospital intake confirmation

Highland Hospital trauma intake confirms via the AMR ePCR bridge: Khalil admitted, no surgical intervention, shoulder reduction performed (closed), scalp suture (4 stitches), discharged with sling + 48h observation. This updates `transport_log.outcome = "admitted_then_discharged_same_day"`. Audit: `EVT-J152-INCIDENT-OUTCOME-UPDATE-015`.

The `severe_injury` provisional flag is downgraded — but the §342 8-hour reporting **still applies** because the worker required hospitalization (admitted, not just treated). Per Cal/OSHA T8 §342 the trigger is "in-patient hospitalization > 24 hours" OR "amputation" OR "loss of an eye". Admission-then-same-day-discharge is the edge case. The workflow flags it as **REQUIRES SAFETY-OFFICER REVIEW** and refuses to auto-cancel the timer. Priya gets a notification.

### 18:21 PDT — Priya rules the §342 path

Priya reviews Highland's discharge summary (attached via the AMR ePCR bridge with Khalil's consent). She determines hospitalization was <24h (admitted 14:50, discharged 17:08), no amputation, no eye loss. She marks `§342_reportable = false` with a written justification in EN. The system **still files a courtesy report** to Cal/OSHA via the `connector` bridge because Halcyon Build's HSE policy is to over-report. The Cal/OSHA inbox receives the incident summary at `18:24:11 PDT`.

Audit: `EVT-J152-COMPLIANCE-CALOSHA-COURTESY-FILED-016`.

### 18:52 PDT — Close-of-shift

Ahmad closes shift. The incident is at status `OPEN-MEDICAL-UNDER-OBSERVATION`. The 60-minute medical-bypass window is long expired (it expired at 15:37:14 PDT). The chain-of-custody is sealed. The OSHA-301 form is auto-generated as a draft for Priya's review next morning. Paycom shows Khalil as `Active – Out on Workers Comp`. State Fund FROI-1 is acknowledged. Cedar denied 0 legitimate actions. Cedar denied 1 illegitimate action: at 17:42 PDT a deck-7 fitter named Manuel Reyes attempted to attach a private photo to the incident from his personal Drive (not the Halcyon-tenant Drive) — Cedar refused; audit event `EVT-J152-CEDAR-DENY-CROSS-TENANT-DRIVE-017` sealed.

## Bilingual narrative pair (verbatim, as stored)

### narrative_en (Ahmad's voice-note, English transcription)

> "Two thirty-seven PM, Tuesday October fourteen. Crane lifting a rebar bundle to deck six grid J-7. The outer sling band sheared. Bundle slipped about fourteen centimeters laterally. Leading edge struck Khalil Mansour on the right shoulder. He fell on his right side, hit his head on the formwork lip. Conscious. Bleeding from the scalp. Right shoulder probable dislocation. I called nine-one-one. AMR is on the way. Stop-work is in effect on decks four through seven. Crane parked. Khalil reports allergies to sulfa and codeine."

### narrative_ar (Ahmad's voice-note, Arabic transcription)

> "الساعة الثانية وسبع وثلاثون دقيقة بعد الظهر، يوم الثلاثاء الرابع عشر من أكتوبر. الرافعة كانت ترفع حزمة حديد تسليح إلى الطابق السادس عند الخط J-7. الشريط الخارجي للسلسلة انقطع. الحزمة انزلقت حوالي أربعة عشر سنتيمتراً جانبياً. الحافة الأمامية ضربت خليل منصور على الكتف الأيمن. سقط على جانبه الأيمن، اصطدم رأسه بحافة القالب. واعٍ. ينزف من فروة الرأس. كتف أيمن - خلع محتمل. اتصلت بالطوارئ. سيارة الإسعاف في الطريق. توقف العمل ساري في الطوابق من الرابع إلى السابع. الرافعة متوقفة. خليل أبلغ عن حساسية للسلفا والكوديين."

## Audit-event chain (sealed, sequence)

| # | Event class | Timestamp | Sealed by |
|---|---|---|---|
| 001 | EVT-J152-IDENTITY-STEPUP-OK | 14:37:20.871 | identity |
| 002 | EVT-J152-IDENTITY-911-DIAL | 14:37:31.214 | identity |
| 003 | EVT-J152-MSG-STOPWORK-FANOUT | 14:37:45.402 | messenger |
| 003a-003s | EVT-J152-MSG-STOPWORK-ACK-NNN (×19) | 14:37:47 – 14:37:59 | messenger |
| 004 | EVT-J152-INCIDENT-CREATE | 14:37:59.811 | incident-management |
| 005 | EVT-J152-INCIDENT-NARRATIVE-VOICE-AR | 14:38:02.117 | incident-management |
| 006 | EVT-J152-INCIDENT-NARRATIVE-VOICE-EN | 14:38:14.220 | incident-management |
| 007 | EVT-J152-DRIVE-MED-EMRG-DISCLOSE | 14:38:21.005 | drive |
| 008 | EVT-J152-CONNECT-EMS-EXCERPT-VIEW | 14:42:14.881 | connector |
| 009 | EVT-J152-INCIDENT-TRANSPORT-LOG | 14:48:31.901 | incident-management |
| 010 | EVT-J152-WORKFLOW-TIMER-SET | 14:50:00.000 | workflow-engine |
| 011 | EVT-J152-WORKPLACE-PAYCOM-WRITE | 14:54:08.412 | workplace-integration |
| 012 | EVT-J152-WORKPLACE-STATEFUND-FROI | 14:55:42.117 | workplace-integration |
| 013 | EVT-J152-INCIDENT-FOLLOWUP-TASK | 15:14:01.504 | incident-management |
| 014 | EVT-J152-INCIDENT-VIEW-SITELEAD | 16:02:11.012 | incident-management |
| 015 | EVT-J152-INCIDENT-OUTCOME-UPDATE | 17:11:48.301 | incident-management |
| 016 | EVT-J152-COMPLIANCE-CALOSHA-COURTESY-FILED | 18:24:11.701 | compliance |
| 017 | EVT-J152-CEDAR-DENY-CROSS-TENANT-DRIVE | 17:42:39.117 | identity |

All 17 events seal under ADR-0263 emission contract. Every event carries `tenant_id = halcyon_build_llc`, `journey_id = j152`, `trace_id = incident-hb-oak-4421-2026-1014`.

## What did NOT happen

- The full medical record was never disclosed. Only the allergy excerpt.
- The 911 audio was never captured by Halcyon Build (only the metadata).
- Cross-tenant drive access was correctly refused.
- The §342 timer did not fire because Priya reviewed before T-8h (the safety-officer escalation at 22:37 PDT was averted).
- The crane operator Roberto Santos's voice/audio in the cab was NOT recorded by the deck cameras (he was inside the cab; cab audio is its own legal regime per California PC §632).

## What is next (out of scope for this journey)

- The OSHA Area Office Oakland investigation interview the following Monday.
- The crane-sling root-cause analysis (Halcyon Build will engage a third-party inspection — that is a separate workflow under `quality-management`).
- Khalil's workers'-comp benefit determination, which is State Fund's downstream process.
- The crew safety-meeting the next morning where Ahmad presents in EN + AR + ES.
