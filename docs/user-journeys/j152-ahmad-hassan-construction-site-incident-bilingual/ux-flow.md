---
doc_class: User-Journey-UX-Flow
journey_id: j152-ahmad-hassan-construction-site-incident-bilingual
date: 2026-05-20
authority_tier: 2
status: draft
---

# j152 — UX flow: Ahmad's DuraForce + Priya's desktop

## Device + render targets

| Surface | Device | OS | Form factor | Constraints |
|---|---|---|---|---|
| Ahmad — site lead | Kyocera DuraForce Pro 3 | Android 14 + Halcyon MDM | 5.5" 1920×1080 sunlight-readable, glove-mode capacitive | One-handed bias; outdoor luminance up to 80,000 lux; wet-finger tolerance |
| Priya — HSE officer | MacBook Air M3 | macOS 15 | desktop browser (Chrome 145) | Standard responsive |
| Roberto — crane cab | Samsung Galaxy Tab Active 5 | Android 14 + Liebherr-MDM | 8.0" 1920×1200 vibration-mounted | High-vibration tolerance; gloved touch |
| Khalil + crew | Mixed DuraForce E7 | Android 14 + Halcyon MDM | 5.0" 1280×720 | Smaller tap targets allowed (54dp min); haptic confirmations mandatory |

## Tap-target accessibility floor

All site-lead surfaces enforce **≥56dp** tap targets and **≥18sp** text (per Material 3 + WCAG 2.2 AA + the gloved-hand override). Color contrast ≥7:1 (WCAG AAA) because the device is sun-exposed.

## Language switching glyph

A small **AR|EN|ES** glyph at the top-right of every multilingual surface; one tap rotates the active language. The glyph carries the active language's flag-of-language (not flag-of-country — Arabic shows the Arabic League gold, not Saudi Arabia's). Per Halcyon Build's EEOC overlay, the language list is admin-set (en-US, ar-EG, es-MX) and the same triad shows everywhere.

## Screen-by-screen progression

### Screen 1 — Home (locked)

- Background: hi-vis amber Halcyon Build wordmark
- Top bar: time `14:36`, battery 78%, FirstNet signal 5/5
- Center: site name `HB-OAK-4421 · Deck 6 · J-7`
- Bottom: 4 primary tiles — "Crew", "Crane", "Inspection", "Reports"
- **Hardware**: lower-right red SOS button (physical, recessed)

### Screen 2 — SOS pressed → Incident launch (active 14:37:18)

Triggered by physical SOS button press.

- Full-screen red gradient
- Large title (EN): "Incident — Report now"
- Title (AR, right-aligned): "بلاغ حادث — الآن"
- Subtitle (EN): "Touch fingerprint sensor to continue"
- Subtitle (AR): "اضغط على بصمة الإصبع للمتابعة"
- Center: animated fingerprint ring
- Bottom: "Cancel" link in low-contrast gray

**Failure copy** (biometric mismatch):

- (EN) "Try again. If you can't use the fingerprint, tap **PIN BACKUP**."
- (AR) "حاول مجدداً. إذا لم تستطع استخدام البصمة، اضغط **PIN احتياطي**."

### Screen 3 — Incident form, top

- Top bar: small "INCIDENT" pill, red
- Section 1 "WHERE" — pre-filled site/deck/grid; tap to edit (rare)
- Section 2 "WHO" — empty roster picker; "Tap to select affected worker(s)"
- Section 3 "WHAT" — three giant tiles:
  - Yellow "NEAR-MISS"
  - Orange "FIRST-AID"
  - Red "ESCALATE TO 911"
- Section 4 "WHEN" — pre-filled timestamp, editable

### Screen 4 — Roster picker (modal)

- Modal title (EN): "Select affected workers"
- Modal title (AR): "اختر العمال المتضررين"
- List: 8 crew tiles with photo (taken at onboarding), full name, role
- Tile order: by physical proximity (the workforce ontology projection includes deck-grid presence)
- Multi-select via checkbox
- Primary button: "Add (1)" or "Add (N)"

### Screen 5 — ESCALATE-TO-911 confirmation (red full-screen modal)

- Title (EN): "Call 911 — This is a real emergency?"
- Title (AR): "اتصال 911 — هل هذه حالة طوارئ حقيقية؟"
- Buttons:
  - Primary (filled red): "**CALL 911 NOW**" (EN) / "**اتصل 911 الآن**" (AR)
  - Secondary (outlined): "**Already called**" / "**اتصلت بالفعل**"
  - Tertiary (text link, gray): "Wait — change classification"
- Subtext: "This will not record audio. The PSAP recording is the legal record."

### Screen 6 — 911 active (split-screen)

While the device is on the 911 call, the incident form remains visible on the bottom 60% of the screen (the device runs the form behind the dialer). On the top 40%:

- Phone call UI showing "911 — Oakland PSAP"
- Live captioning ON (FirstNet device feature) — Ahmad can read what the dispatcher said in EN even if he's distracted
- Mute button DISABLED for the duration of the call

### Screen 7 — Stop-Work broadcast composer

After Ahmad taps **STOP-WORK BROADCAST** from the incident form:

- Title (EN): "Stop-work broadcast"
- Subtitle: "Three languages will fan out. Decks 4-7."
- Language tags: [✓] EN [✓] AR [✓] ES — each a chip; uncheck to omit (rare)
- Body field — pre-filled with the site-default stop-work text. Editable. Three text boxes stacked, one per language; only EN is auto-filled and the AR/ES are auto-translated by the on-device model (DistilLama-9B-int4 + Arabic + Spanish LoRA adapters).
- Auto-translation badge under AR + ES: a small "auto-translated — review" tag in amber
- Primary button: "**SEND TO 4 DECKS · 19 PEOPLE**" (live count)
- Secondary: "Change recipients"

**Failure copy** (no LTE/5G):

- (EN) "No cell signal. Switching to deck Wi-Fi mesh. This may add 4 seconds."
- (AR) "لا توجد إشارة. التحويل إلى شبكة الطابق. قد يضيف 4 ثوانٍ."

### Screen 8 — Voice-note capture

After 911 call ends (Ahmad still on the deck with Khalil), the form returns to focus.

- Section "Voice note" with **two tabs**: 🅰 AR — 🅱 EN
- Big circular RECORD button (red center, gray ring shows recording level)
- AR is the default tab (the device locale)
- Tap-and-hold to record (Ahmad can use his thumb while his other hand holds gauze to Khalil's scalp)
- Live transcription appears below the button; user can stop, replay, re-record
- After AR recording: a "Switch to EN" pill nudges Ahmad to also record an English version
- Both tabs must have content before the form can be submitted (site-lead requirement; non-site-lead path allows single-language)

### Screen 9 — Pull-medical (acute window) — red modal

Triggered when Ahmad taps "Pull worker medical (acute window)" from the incident form.

- Background: deep red with a yellow caution chevron
- Title (EN): "Emergency medical disclosure"
- Title (AR): "كشف طبي طارئ"
- Body (EN):
  > This invokes the **ADR-0298 emergency bypass**.
  > Only the **allergies + current medications** of the affected worker will be disclosed to this incident.
  > Window: **60 minutes from now**.
  > This action is **audit-sealed and reviewable** by your safety officer.
- Body (AR): translated equivalent, right-to-left rendering
- Affected worker name + photo: Khalil Mansour
- Primary button: "**Continue — disclose allergies & meds**"
- Secondary: "Cancel"
- Tertiary (gray link): "Why this is needed"

Tap "Continue" → device step-up freshness check; if ≤120s, proceed without re-auth; if >120s, biometric re-prompt.

### Screen 10 — Disclosed excerpt (in the incident form)

After bypass:

- New section "Medical (acute window — 59:42 remaining)"
- Two lines:
  - "Allergies: sulfa, codeine"
  - "Current medications: none reported"
- Footer pill: "Auto-sealed in audit-chain · expires at 15:37:11 PDT"
- No expand/show-more action. The full record is NOT accessible from this screen.

### Screen 11 — Stop-work ACK monitor

Bottom-of-screen sticky widget:

- Header: "Stop-Work ACK · 19 / 19"
- Live count climbs from 0/19 to 19/19 as devices ACK
- Per-deck pills: "Deck 4 · 3/3 · Deck 5 · 4/4 · Deck 6 · 7/7 · Deck 7 · 5/5"
- Color: green when all acked; amber if any timeout; red if any rejected

### Screen 12 — Roberto's crane-cab tablet

Roberto Santos's cab tablet receives:

- The stop-work broadcast (ES locale; he sees the Spanish text)
- A persistent banner: "STOP WORK — Deck 6 · J-7 · 14:37 · Ahmad"
- The crane controls switch to "Locked – Park Mode"
- Roberto must ACK the stop-work AND confirm "crane parked" via a separate 2-tap sequence

### Screen 13 — Priya's desktop (HSE officer view)

URL: `https://hse.halcyon-build.com/incidents/INC-2026-1014-HB-OAK-4421-0007`

Layout (left-to-right):

- **Left rail** — incident metadata, status pill, §342 timer countdown, severity
- **Center pane**:
  - Bilingual narrative pair, side-by-side, EN on left, AR on right (right-to-left)
  - Crane telemetry chart (load-pin spike highlighted, 90s window, x-axis: PDT timestamps)
  - Camera clip player (4 minutes, NW + SE angle, click to switch)
  - Medical excerpt (read-only, with disclosure-event link)
- **Right rail** — workplace-integration status (Paycom ✓, State Fund ✓), Cal/OSHA timer + courtesy filing button, follow-up task list

### Screen 14 — Cal/OSHA §342 reviewer modal (Priya)

When Priya clicks "Review §342 reportability":

- Title: "§342 reportability ruling"
- Decision tree (radio):
  - "Reportable under §342" → triggers the formal report path
  - "Not reportable — but courtesy filing recommended" (default if hospital admission < 24h)
  - "Not reportable — no filing"
- Required: written rationale (≥200 chars, English; AR optional)
- Required: signature pad with passkey step-up
- Primary button: "File ruling"
- Audit pill at bottom: "Your decision will be sealed under ADR-0263 emission class EVT-J152-COMPLIANCE-CALOSHA-RULING"

### Screen 15 — Ahmad's role-projection cross-check (ADR-0317)

When Ahmad later opens the same incident from his DuraForce:

- View is `site_lead_incident_v1`
- He can see: narrative (both languages), camera clips, his own actions log, the §342 timer status, the stop-work broadcast log
- He CANNOT see: Khalil's full medical record, Khalil's pay/SSN, the Paycom employee-injury-report internal fields, the State Fund FROI-1 PII, Priya's written rationale (he can see "rationale filed" but not the text)
- A small lock pill at the top: "You are viewing this incident as **Site Lead**. To see the HR view, switch role from the HSE dashboard."

## Critical state transitions

| Trigger | From state | To state | Side-effect |
|---|---|---|---|
| Ahmad presses SOS | LOCKED | INCIDENT-LAUNCH | Step-up demanded |
| Step-up succeeds | INCIDENT-LAUNCH | INCIDENT-COMPOSE | Form opens |
| ESCALATE-TO-911 confirmed | INCIDENT-COMPOSE | EMS-DISPATCHED | 911 dial + incident.create |
| Stop-Work fanout sent | EMS-DISPATCHED | STOP-WORK-ACTIVE | messenger broadcast |
| AR voice + EN voice both attached | STOP-WORK-ACTIVE | NARRATIVE-COMPLETE | transcription pipelines fire |
| Medical bypass confirmed | NARRATIVE-COMPLETE | MED-BYPASS-ACTIVE (60min) | drive projection |
| AMR arrives (PSAP CAD event) | MED-BYPASS-ACTIVE | EMS-ON-SCENE | connect link consumed |
| AMR transport | EMS-ON-SCENE | UNDER-TRANSPORT | transport_log written |
| §342 timer set | UNDER-TRANSPORT | §342-COUNTDOWN | workflow-engine arms timer |
| Hospital admission | §342-COUNTDOWN | UNDER-OBSERVATION | outcome_update |
| Priya's §342 ruling | UNDER-OBSERVATION | RULING-FILED | compliance filing |
| Bypass window expires | MED-BYPASS-ACTIVE | BYPASS-EXPIRED | medical excerpt unreadable |
| Close-of-shift | RULING-FILED | OPEN-MEDICAL-UNDER-OBSERVATION | overnight monitoring |

## Accessibility specifics

- **High-contrast outdoor mode**: automatically engaged when ambient luminance >20,000 lux. Backgrounds darken, text becomes white-on-black with 8:1 contrast.
- **Glove mode**: detected via the DuraForce's capacitance hint; tap-target floor raised from 56dp to 72dp; haptic confirmation strengthened from 30ms to 60ms.
- **Wet-finger mode**: if the screen detects water film, switch to physical-button-priority — the SOS hardware button is honored even if the screen is unreliable.
- **One-handed reach**: critical actions (SOS, Stop-Work, Pull-Medical) are positioned within the right-thumb reach zone (bottom-right quadrant) on the 5.5" screen.
- **RTL Arabic rendering**: all Arabic text is rendered right-to-left; mixed AR-EN sentences use the Unicode bidirectional algorithm; punctuation flips correctly.

## Copy review

Every user-facing string is reviewed by **two** native speakers (one Levantine + one Egyptian for Arabic; one Mexican + one Castilian for Spanish; one US-EN copywriter) before shipping. The review log is held in `drive` under `journey-j152/copy-review-log.yaml`.

## Anti-pattern guardrails

1. Never auto-translate the **stop-work** body silently. Always show the auto-translation badge in amber and require a site-lead review the first time a new site-default stop-work text is created.
2. Never default to English-only on a multilingual site. The site language list is admin-set and the UI shows all of them.
3. Never bury the medical-bypass rationale. The "Why this is needed" link must surface ADR-0298, the audit-event class, and the 60-minute window in plain language in both languages.
4. Never auto-cancel the §342 timer. Only an authorised HSE officer's written ruling cancels it.
5. Never expose Khalil's full medical record to the site-lead view. The role-projection (ADR-0317) enforces this even if the data could be fetched server-side.
