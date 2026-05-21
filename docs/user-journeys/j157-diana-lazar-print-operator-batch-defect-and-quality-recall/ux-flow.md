---
doc_class: User-Journey-UX-Flow
journey_id: j157-diana-lazar-print-operator-batch-defect-and-quality-recall
date: 2026-05-20
authority_tier: 2
status: draft
---

# j157 — UX flow: 11:42 EET defect → 20:17 EET handoff

Three device contexts: Diana's Panasonic Toughpad FZ-G2 (press operator station, VESA-mounted), Mihai's desktop in the manager office (Dell Precision 3680), and Cristina Munteanu's laptop at Antibiotice Iași QA office (Lenovo ThinkPad P14s). All three render diacritics natively (UTF-8 NFC) and support RO + EN + HU locale switching.

The press environment is industrial: 75 dB ambient noise, FOGRA D50 5,000K controlled lighting, occasional ink/solvent on hands. The UX must work in glove-mode, must not require keyboard typing for the critical line-stop action, and must render text legible at arm's length from the operator station.

## Screen 1 — Quality alert + press camera frame (11:42:18 EET · Toughpad FZ-G2)

```
┌─────────────────────────────────────────────────┐
│ ⚠ ALERTĂ CALITATE — ABATERE FOGRA-PSO          │
│   QUALITY ALERT — FOGRA-PSO DEVIATION          │
│                                                 │
│   Lot:  BCH-2027-02-23-0612-pharma-leaflet-…   │
│   Sheet: 23,847  ·  ΔE 1.4 → 4.7  (cap 3.0)    │
│   Registration shift: +1.2 mm Y                 │
│                                                 │
│   ┌───────────────────────────────────────┐    │
│   │  📷  press camera @ sheet 23,847       │   │
│   │                                       │    │
│   │  [shows the red allergy-warning      │    │
│   │   box with the "sub" → "ub" clip]    │    │
│   │                                       │    │
│   │  ⚠ "Nu administrați copiilor          │    │
│   │     ub 6 ani fără sfatul medicului"   │    │
│   │   ← LEGAL TEXT CLIPPED                │    │
│   └───────────────────────────────────────┘    │
│                                                 │
│   ┌─────────────────────────────────────────┐  │
│   │   🛑  OPREȘTE LINIA / LINE STOP        │  │
│   └─────────────────────────────────────────┘  │
│                                                 │
│   secondary: "show ΔE history" · "adjust ink" │
│                                                 │
│   Operator: Diana Lazăr · FOGRA-PSO L2          │
└─────────────────────────────────────────────────┘
```

UX notes:

- The big red **OPREȘTE LINIA / LINE STOP** button occupies the lower third — designed for thumb-press while wearing nitrile gloves.
- Bilingual label is mandatory for legal-warning text + line-stop button per ISO-12647-2 procedural-controls guidance.
- The press camera preview shows the actual defect, not an abstraction. Diana saw what was wrong before tapping; she did not trust the colorbar alone.
- The "show ΔE history" secondary action surfaces the trajectory (1.4 → 1.5 → 1.6 → 2.0 → 2.7 → 4.7) — confirms drift is rising, not noise.
- The "adjust ink" secondary is intentionally subordinated; the cultural cue is that for legal-text clipping, the right action is STOP, not ADJUST.
- A long-press (>2 s) on LINE STOP fires a "hold to confirm" — protects against accidental tap; a short tap shows a confirm dialog instead.

## Screen 2 — Line-stop confirm (11:42:34 EET)

```
┌─────────────────────────────────────────────────┐
│  Opri linia? / Stop the line?                   │
│                                                 │
│  Batch:    BCH-2027-02-23-0612-pharma-…         │
│  Reason:   ΔE breach + 1.2mm registration       │
│            shift clipping legal warning text    │
│                                                 │
│  După oprire / After stop:                      │
│    • Press halts ≤ 14 s                         │
│    • Sheets in transit → quarantine             │
│    • Recall workflow initiates automatically    │
│    • Customer (Antibiotice) will be notified   │
│      within 90 minutes per MSA §7.4            │
│                                                 │
│  Authority basis:                               │
│  ✓ Your FOGRA-PSO L2 cert is the authority      │
│  ✗ NO manager approval required                 │
│                                                 │
│  ┌─────────────────┐    ┌─────────────────┐    │
│  │  ✕ ANULEAZĂ     │    │  ✓ DA, OPREȘTE  │    │
│  │  ✕ Cancel       │    │  ✓ Yes, stop    │    │
│  └─────────────────┘    └─────────────────┘    │
└─────────────────────────────────────────────────┘
```

UX notes:

- The "NO manager approval required" line is **always visible** — explicit affirmation of operator authority. This single line eliminates the cultural hesitation that caused the 2024-11-08 Sandoz delay.
- Cancel is left, confirm is right — matches Diana's dominant-hand thumb.
- The "Authority basis" pulls live from `learning-management` to show the actual cert ID + expiry. If her cert were expired, this screen would show an entirely different flow.

## Screen 3 — Recall workflow dashboard (11:44–14:42 EET)

```
┌─────────────────────────────────────────────────┐
│ RECALL · BCH-2027-02-23-0612 · stop_called      │
├─────────────────────────────────────────────────┤
│                                                 │
│   state-machine:                                │
│   ● stop_called   (you are here, 11:42:42)      │
│   ◯ quarantine                                  │
│   ◯ defect_root_cause                           │
│   ◯ customer_notify                             │
│   ◯ recall_execute                              │
│   ◯ closure_post_mortem                         │
│                                                 │
│   ──── TASKS · 2 / 14 done ────                 │
│   ✓ 1  line-stop                                │
│   ✓ 2  in-transit-quarantine                    │
│   ▶ 3  count clean sheets (23,847)             │
│   ☐ 4  segregate suspect sheets                 │
│   ☐ 5  retrospective sample (AQL 0.4, n=315)    │
│   ☐ 6  photograph defects                       │
│   ☐ 7  ship samples to customer QA              │
│   ☐ 8  customer notification                    │
│   ☐ 9  mechanical inspection                    │
│   ☐ 10 root cause confirm                       │
│   ☐ 11 CAPA correction/corrective/preventive    │
│   ☐ 12 customer recall execute (do not ship)    │
│   ☐ 13 regulator notification (hold)            │
│   ☐ 14 closure + post-mortem                    │
│                                                 │
│   ──── EVIDENCE ────                            │
│   📷 press-frame-23847.jpg ✓                    │
│   📷 colorbar-history.json ✓                    │
│   ☐  defect photographs (8 needed)              │
│                                                 │
│   Operator: Diana Lazăr · FOGRA-PSO L2          │
└─────────────────────────────────────────────────┘
```

UX notes:

- The state-machine pill is sticky at top — Diana always knows where the recall is.
- "you are here" is shown at the current state with the exact transition timestamp.
- Tasks are checkboxes for quick visual; tapping any task opens it for evidence capture.
- "8 needed" photo placeholder is explicit — the screen won't show task #6 as done until 8 photos are uploaded.

## Screen 4 — Defect photograph capture (12:18 EET)

```
┌─────────────────────────────────────────────────┐
│ TASK 6 · photograph defects                     │
├─────────────────────────────────────────────────┤
│                                                 │
│   📷 capture                                    │
│   ┌───────────────────────────┐                 │
│   │  [live camera view]       │                 │
│   │                           │                 │
│   │  ringlight: ON            │                 │
│   │  focus: macro 10×         │                 │
│   │  white-balance: D50       │                 │
│   └───────────────────────────┘                 │
│                                                 │
│   Defect type for this photo:                   │
│   ◯ Color drift (solid C/M)                     │
│   ◯ Registration shift                          │
│   ● Legal-text clipping ← selected              │
│   ◯ Other                                       │
│                                                 │
│   Sample-ID:  [sample-23847-block-23000-A]      │
│   Position:   sheet 23,847                      │
│   Operator:   Diana Lazăr                       │
│                                                 │
│   captured: 5 / 8                               │
│                                                 │
│   ┌─────────────────────────────────────────┐  │
│   │   📷 CAPTURE                            │  │
│   └─────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

UX notes:

- Defect-type radio button is required before capture — forces classification at capture time, not later.
- Sample-ID auto-populates from the last scanned barcode.
- Photo lands in `tasks` evidence vault with EXIF including all metadata fields; never enters the device's general camera roll.

## Screen 5 — Customer notification draft (12:31–12:42 EET)

```
┌─────────────────────────────────────────────────┐
│ CUSTOMER NOTIFICATION · draft                   │
├─────────────────────────────────────────────────┤
│ thread: recall-tipografia-antibiotice-…         │
│ recipients: Cristina M., Andrei P., Carmen E.   │
│                                                 │
│ ┌─ ROMÂNĂ ─────────────┬─ ENGLISH ─────────────┐│
│ │                      │                       ││
│ │ Lot: BCH-2027-02-…   │ Batch: BCH-2027-02-…  ││
│ │ Cantitate: 47,500    │ Quantity: 47,500      ││
│ │ Defect: ΔE+1.2mm…    │ Defect: ΔE+1.2mm…     ││
│ │ Status: LINE STOPPED │ Status: LINE STOPPED  ││
│ │ Root-cause:          │ Root-cause:           ││
│ │ dampener-roller…     │ dampener-roller…      ││
│ │                      │                       ││
│ │ Solicitări:          │ Action requested:     ││
│ │  (1) Confirmare 4h   │  (1) Confirm 4h       ││
│ │  (2) Inspecție?      │  (2) Inspection?      ││
│ │  (3) Re-run slot     │  (3) Re-run slot      ││
│ │  (4) ANMDMR posture  │  (4) ANMDMR posture   ││
│ │                      │                       ││
│ └──────────────────────┴───────────────────────┘│
│                                                 │
│ Diacritic check: Lazăr ✓ administrați ✓ ăâîșț ✓ │
│ Both sides equal authority per MSA §7.4          │
│                                                 │
│ ┌─────────────────────────────────────────────┐ │
│ │  📨 SEND TO CUSTOMER                        │ │
│ └─────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
```

UX notes:

- The split-pane RO/EN layout makes it visually clear neither version is "primary"; both are equal under contract.
- "Diacritic check" line proves the system stored the characters correctly. Diana sees "Lazăr" not "Lazar" — a critical legal-fidelity signal.
- The customer-side participants are named explicitly so the operator knows who reads the message.
- A 4-hour SLA timer starts when SEND is tapped; the timer is visible on subsequent screens.

## Screen 6 — Mihai's desktop view (12:45 EET · Dell Precision 3680)

Mihai's desktop has 3 monitors. Left: press production-planning calendar. Center: recall workflow dashboard mirroring Diana's view. Right: the messenger thread with Antibiotice.

```
┌──── monitor 2 · recall dashboard (manager) ─────┐
│ RECALL · BCH-2027-02-23-0612 · customer_notify  │
│                                                 │
│ state-machine progress: 4 / 6                   │
│ tasks done: 7 / 14                              │
│ customer SLA: 03:48:14 remaining                │
│ customer status: notification sent 12:42:18 ✓   │
│                                                 │
│ ──── ACTIONS AVAILABLE TO YOU ────              │
│ • Endorse CAPA when filed                       │
│ • Approve sample-ship destination               │
│ • Escalate to ANMDMR (currently HOLD)           │
│ • Read-only: Diana's operator notes             │
│                                                 │
│ ──── EXPLICIT NOTE FROM SYSTEM ────              │
│ ⓘ Diana initiated the line stop under her       │
│   FOGRA-PSO L2 cert authority. NO manager      │
│   approval was required or sought. This is     │
│   the correct policy under ADR-0263 audit      │
│   doctrine.                                    │
└─────────────────────────────────────────────────┘
```

UX notes:

- Mihai's view makes the operator authority explicit. The "NO manager approval was required" line preserves the doctrine across the manager UI — preventing future cultural drift where managers expect to be consulted.
- "Read-only: Diana's operator notes" — Mihai can read, NOT edit. Diana owns the record of her action.

## Screen 7 — Cristina at Antibiotice (12:48 EET · Lenovo ThinkPad P14s)

```
┌─ Antibiotice · Sistem Calitate · Inbox ─────────┐
│                                                 │
│  ⚠ Recall notification — Tipografia Lazăr-…     │
│    Diana Lazăr · 12:42:18                       │
│                                                 │
│  [opens thread]                                 │
│                                                 │
│  Lot:  BCH-2027-02-23-0612                      │
│  Statut: LINE STOPPED 11:42 EET                 │
│  Defect critic: legal warning text clipped      │
│  Zero products to market ✓                      │
│                                                 │
│  Diana's action sequence (audit-verified):      │
│  • 11:42:14 ΔE breach detected by GMI           │
│  • 11:42:38 LINE STOP initiated (operator auth) │
│  • 11:42:56 Press halted                        │
│  • 12:08:00 Manual count confirmed 23,847       │
│  • 12:18:00 Defect photographs captured         │
│  • 12:31:00 Customer notification drafted       │
│  • 12:42:18 Customer notification sent          │
│                                                 │
│  Audit merkle proof: ✓ verified                 │
│                                                 │
│  ┌─────────────────────────────────────────┐    │
│  │   📨 REPLY                              │    │
│  └─────────────────────────────────────────┘    │
└─────────────────────────────────────────────────┘
```

UX notes:

- Cristina sees Diana's full action sequence with timestamps + the audit merkle proof. The merkle proof is a single visual checkmark — it means the chain is intact and tamper-evident.
- "Zero products to market ✓" is a key reassurance — Cristina can verify this assertion via the audit chain.
- The reply button is the only action; her response will be templated by Antibiotice's QMS but personalized.

## Screen 8 — CAPA collaborative draft (14:32–15:42 EET · split between Diana's tablet + Mihai's desktop)

```
┌─────────────────────────────────────────────────┐
│ CAPA · bch-2027-02-23-0612 · co-edit            │
├─────────────────────────────────────────────────┤
│ co-authors: Diana Lazăr · Mihai Lazăr-Petrescu   │
│ template: ISO-9001 §10.2 nonconformity + CA    │
│                                                 │
│ ── §1 CORRECTION (immediate, hours) ────────    │
│ • Quarantine 23,847 sheets pending QA samp.     │
│ • Replace dampener-roller cylinder #4           │
│ • Re-run suspect zone 22:00 EET on Heid. #2    │
│                                                 │
│ ── §2 CORRECTIVE ACTION (this incident) ────    │
│ • Monthly dampener-cylinder inspection (was Q)  │
│ • ΔE2000 trend alert at 2.5 (was 3.0)           │
│ • Train all operators on line-stop authority    │
│   drill (FOGRA-PSO L2 cohort, 6 people)         │
│                                                 │
│ ── §3 PREVENTIVE ACTION (systemic, weeks) ──    │
│ • Heidelberg predictive bearing-monitoring      │
│   upgrade (capex €18,400)                       │
│ • Quarterly FOGRA reference cross-check         │
│   at independent lab                            │
│ • Annual cert refresh: FOGRA-PSO L2 +           │
│   ISO-12647-2 trained                           │
│                                                 │
│ Diana drafted §1 ✓                              │
│ Mihai endorsed §1 ✓                             │
│ Diana drafted §2 ✓                              │
│ Mihai endorsed §2 — pending                     │
│ Diana drafted §3 ✓                              │
│ Mihai endorsed §3 — pending                     │
│                                                 │
│ ┌─────────────────────────────────────────────┐ │
│ │  📋 FILE CAPA TO QMS                        │ │
│ └─────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
```

UX notes:

- Co-edit with sectional sign-off (Diana drafts; Mihai endorses each section). No CAPA is filed until both authors sign all three sections.
- The diacritics in "Lazăr-Petrescu" persist correctly throughout, including in the file metadata.
- File-to-QMS action triggers immutable audit + merkle anchor.

## Screen 9 — Shift handoff (20:14 EET)

```
┌─────────────────────────────────────────────────┐
│ Shift handoff · day → night                     │
├─────────────────────────────────────────────────┤
│ outgoing: Diana Lazăr                           │
│ incoming: Vladimir Csikós / Csikós Vladimír     │
│                                                 │
│ ── ACTIVE RECALL ────────────────────────────   │
│ recall-bch-2027-02-23-0612 (in progress)        │
│   state: customer_notify (awaiting QA sample)   │
│   next milestone: re-run 22:00 EET Heid. #2     │
│                                                 │
│ ── SCHEDULED RUNS ──────────────────────────    │
│ 22:00 EET · BCH-…0612 re-run (suspect zone)     │
│        ↳ press: Heidelberg #2                   │
│        ↳ remaining: 23,653 sheets               │
│                                                 │
│ ── CAPA FILED ──────────────────────────────    │
│ ✓ capa-bch-2027-02-23-0612                      │
│                                                 │
│ ── HANDOFF NOTES ───────────────────────────    │
│ "Roller #4 replaced 19:30. Test ΔE 0.8.         │
│  Re-run scheduled. Customer QA sample           │
│  shipped via Cargus 16:18, ETA 09:30 next      │
│  day. ANMDMR template held. No regulator       │
│  notification needed (no products to market)." │
│                                                 │
│ Vladimir confirmed (passkey + face_id) ✓        │
│                                                 │
│ Locale at handoff: RO ↔ HU code-switching       │
└─────────────────────────────────────────────────┘
```

UX notes:

- Vladimir's name renders in both Romanian-first form (Vladimir Csikós) and Hungarian-first form (Csikós Vladimír) — the diacritic + ordering preference is per-user.
- Handoff requires Vladimir's passkey + face_id to confirm — preventing accidental shift attribution.
- The locale chip indicates that the two operators code-switch RO↔HU — system supports this without forcing one canonical locale.

## Locale + accessibility

- Diana's locale: `ro-RO` primary; `en-GB` secondary; `hu-HU` tertiary (light)
- Diacritic policy: UTF-8 NFC throughout; never normalized to ASCII; search supports diacritic-aware mode and diacritic-insensitive mode with explicit flag
- Font: System default Latin-Extended-A supporting all RO + HU diacritics + ligatures (Liberation Sans on Linux, San Francisco on macOS, Segoe UI on Windows, Roboto on Android tablet)
- Glove mode: capacitive touch threshold increased; tap targets ≥48dp; haptic feedback on every confirm
- Color contrast: emergency-red (#D72638) on cream-white (#F8F4E3) for STOP buttons; verified WCAG AAA contrast
- Press environment: 75 dB ambient — no audio-only signals; all critical alerts have visual + haptic redundancy
- Voice fallback: not used in pressroom (ambient noise prohibitive); voice IS used in Mihai's office and Cristina's office

## Failure-mode UX

| Failure | UX response |
|---|---|
| Telemetry lost mid-batch | Quality-mgmt switches to operator-visual fallback; large amber banner: "auto-detect off; rely on visual" |
| Press fails to halt within 14 s | Escalate to physical E-stop button + manager page + audit |
| Customer messenger unreachable | Notification queues locally; alternate-channel SMS + email tried; audit per attempt |
| Cedar service degraded | Fail-closed; line stop still works (it's the safer default) but recall workflow advances paused |
| Photo upload fails | Tasks marked "pending evidence"; retry queue; permits final-state advance only after all uploaded |
| Diacritic loss detected in any persisted field | Hard error; field write rejected; audit `EVT-J157-DIACRITIC-NORMALIZE-DENY-012e` |

## Stop condition

The UX flow is correct when Diana can complete the 8h35m journey in a 75 dB pressroom, in glove-mode, with all diacritic-bearing fields persisted at byte-level fidelity, with the operator-line-stop-authority doctrine visible at every screen where a manager might otherwise be expected to gate, and with the bilingual + cross-tenant + regulator-touching customer-facing flow working without ambiguity.
