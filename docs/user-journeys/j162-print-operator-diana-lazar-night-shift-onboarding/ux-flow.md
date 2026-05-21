---
doc_class: User-Journey-UX-Flow
journey_id: j162-print-operator-diana-lazar-night-shift-onboarding
date: 2026-05-20
authority_tier: 2
status: draft
---

# j162 — UX flow: 6-day final-onboarding-phase + first solo night-shift

Five device contexts: Diana's Panasonic Toughpad FZ-G2 on the press (continuing from j157, with night-shift mode overlay); Adriana Stanciu's Toughpad during assessment; Vladimir's tablet during proctoring; Securitas alarm-cooperative interface (small wall-mounted touchscreen at the depot side door + cloud admin UI for the cooperative tenant); Diana's personal iPhone (lone-worker bracelet pairing + family emergency contingency); Mihai's iPhone for escalation; Camelia's Toughpad at handoff.

The unifying UX rule: the **night-shift mode chip** ("🌙 NIGHT") + **lone-worker mode chip** ("👤 SOLO") persist at the top of Diana's screens during the actual night shift. Day-shift screens show "☀️ DAY" + no solo chip. The competency context is always visible.

## Screen 1 — Final competency assessment scoreboard (Tue Jan 26 22:42 EET · Adriana's Toughpad)

```
┌──────────────────────────────────────────────────┐
│ 🏥 adriana-stanciu-consulting-ro                 │
│ Evaluare · Diana Lazăr · noaptea solo            │
├──────────────────────────────────────────────────┤
│                                                  │
│  Candidat: Diana Lazăr (FOGRA-PSO L2)             │
│  Proctor: Vladimir Csikós                         │
│  Observator: Adriana Stanciu (HSE)                │
│                                                  │
│  Începere: 21:18 EET · Durată: 84 min             │
│                                                  │
│  ── Scoruri pe scenariu ──                       │
│   1  cold-startup low-light          92 ✓        │
│   2  ink-density mid-run             87 ✓        │
│   3  paper-jam clear                 94 ✓        │
│   4  spill response                  91 ✓        │
│   5  emergency-stop drill            96 ✓        │
│   6  dead-man check-in              100 ✓        │
│   7  substrate change-over           89 ✓        │
│   8  ΔE2000 trend alert              92 ✓        │
│   9  audit-readiness                 88 ✓        │
│  10  plant-maintenance interact.     90 ✓        │
│  11  pharma-PIL protocol             95 ✓        │
│  12  cross-shift handoff             93 ✓        │
│  13  personal-tenant emergency       89 ✓        │
│  14  integrative scenario            93 ✓        │
│                                                  │
│  Toate scenariile ≥ 85 ✓                         │
│  Patru categorii ≥ 90 ✓                          │
│                                                  │
│  Semnături calitative:                           │
│  ✓ Proctor Vladimir Csikós                       │
│  ✓ HSE Adriana Stanciu                           │
│                                                  │
│  Diacritic check:                                │
│  Diana Lazăr ✓ Csikós ✓ Lazăr ✓ Tăbârcă ✓        │
│                                                  │
│  Cross-journey:                                  │
│  j157 cert chain: FOGRA-PSO L2 + ISO-12647-2 ✓   │
│  prerequisites verified                          │
│                                                  │
│  ┌─────────────────────────────────────────────┐ │
│  │  ✓ DESBLOCHEAZĂ COMPETENȚA                   │ │
│  │  Unlock night-shift solo competency          │ │
│  └─────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

UX notes:

- Adriana's tenant chip is visible: she is consulting from her own consulting-firm tenant.
- The cross-journey continuity line confirms j157's certs satisfy the prerequisite — without forcing Diana to re-prove what she's already proven.
- Diacritic check is on-screen as a small but critical signal — Romanian + Hungarian both preserved.

## Screen 2 — Competency unlocked (Tue Jan 26 22:48 EET · Diana's Toughpad)

```
┌──────────────────────────────────────────────────┐
│ 🏢 tipografia-lazar-petrescu-ro · firma          │
│ ☀️ DAY · solo: nu (zi)                            │
├──────────────────────────────────────────────────┤
│                                                  │
│      🏆 COMPETENȚĂ DESBLOCATĂ                    │
│                                                  │
│  Competență: night-shift-solo-authorization-2027 │
│  Operator:   Diana Lazăr                         │
│  Valabilitate: 2027-01-26 → 2028-01-26            │
│                                                  │
│  Această competență deblochează:                  │
│  ✓ Tură de noapte 22:00-06:30 EET                 │
│  ✓ Operare solo                                  │
│  ✓ Alarm-cooperative Securitas de-arm scope     │
│  ✓ Plată night-shift +25% (RO §126)              │
│                                                  │
│  ── Pași următori ──                             │
│  Mâine 09:00: workplace-integration provisioning │
│  Mâine 11:42: dead-man protocol enrolment        │
│  Joi 14:18: primul work-order de noapte           │
│  Sâmbătă 10:18: walkthrough cu Vladimir          │
│  Luni 22:00: prima tură solo                     │
│                                                  │
│  Mulțumiri:                                      │
│  Vladimir Csikós + Adriana Stanciu               │
│                                                  │
│  Audit dual-seal: ✓                              │
│                                                  │
│  ┌─────────────────────────────────────────┐    │
│  │  📋 VEZI PLANUL                          │    │
│  └─────────────────────────────────────────┘    │
└──────────────────────────────────────────────────┘
```

UX notes:

- Day-mode chip still active because we're at the moment of unlock, not yet on night-shift.
- Forward-looking schedule is concrete with timestamps.
- Acknowledgment of the proctor + HSE observer is built into the unlock screen.

## Screen 3 — Workplace-integration provisioning checklist (Wed Jan 27 09:18 EET · Diana's Toughpad)

```
┌──────────────────────────────────────────────────┐
│ 🏢 tipografia-lazar-petrescu-ro                  │
├──────────────────────────────────────────────────┤
│ Workplace-integration · Tură noapte              │
│                                                  │
│ 📅 Şift schedule                                  │
│ ✓ Mon 2027-02-01 22:00-06:30 EET                  │
│ ✓ next planned: each Mon + alternate weeks       │
│                                                  │
│ 📍 Geofence                                       │
│ ✓ tipografia-depot-skvrnany-perimeter-18m         │
│ ✓ added to night-shift valid zones                │
│                                                  │
│ 🪪 Badge                                          │
│ ✓ RFID badge rfid-diana-lazar-2024-09            │
│ ✓ added role: night_shift_authorized              │
│ ✓ added scopes: after-hours-entry + alarm-dearm  │
│                                                  │
│ 🛡 Securitas alarm-cooperative                    │
│ Cross-tenant call:                               │
│ tipografia-lazar-petrescu-ro                     │
│ → cz-securitas-alarm-cooperative-tenant-ro       │
│ ✓ Diana's biometric added to night-dearmer roster│
│ ✓ Alarm zones in scope: pressroom-night-shift    │
│ ✓ Auto-revoke on competency expiry              │
│                                                  │
│ 💰 Payroll night-shift differential               │
│ ✓ +25% per RO §126                                │
│ ✓ base 47 RON/h → night 58.75 RON/h               │
│ ✓ Carmen (bookkeeper) confirmed                   │
│                                                  │
│ Audit dual-seal: ✓ Tipografia + Securitas         │
│                                                  │
│ Următoarea etapă: dead-man enrolment              │
│ Programat 11:42 EET azi                          │
└──────────────────────────────────────────────────┘
```

UX notes:

- All 5 sub-steps of provisioning visible as a checklist Diana can see resolving in real-time.
- The Securitas cross-tenant call's source + target tenants are explicit.
- Auto-revoke on competency expiry is named — the doctrine is on-screen.

## Screen 4 — Dead-man protocol enrolment (Wed Jan 27 11:42 EET · Toughpad)

```
┌──────────────────────────────────────────────────┐
│ 🏢 tipografia-lazar-petrescu-ro                  │
├──────────────────────────────────────────────────┤
│ Lone-worker dead-man protocol · enrolment        │
│                                                  │
│ Modul biometric (low-light reconfigure):         │
│ Capturi efectuate: 4 / 4 ✓                       │
│ Validare: ✓ (low-light + standard match)         │
│                                                  │
│ PIN fallback: configured (****) ✓                │
│                                                  │
│ Cadență check-in:                                │
│ Interval: 4 ore                                  │
│ Fereastră răspuns: 60 secunde                    │
│ Metodă: tablet tap + face_id                     │
│                                                  │
│ ── Lanț de escaladare ──                         │
│ Order 1: Mihai Lazăr-Petrescu (personal tenant) │
│   Method: voice call + messenger priority       │
│   Consent: ✓ Mihai a acceptat                   │
│                                                  │
│ Order 2: Adriana Stanciu (HSE consultant)        │
│   Method: messenger priority                    │
│                                                  │
│ Order 3: Marius Iancu (Heidelberg tech)          │
│   Method: messenger                             │
│                                                  │
│ ── Rută de urgență familială ──                  │
│ Şcoala Internationale Cluj                       │
│ (linia de urgență pentru fiica Maria)            │
│                                                  │
│ ⚠ Aceasta este o rută cross-tenant.              │
│ Permisă doar pentru urgență familială            │
│ EU-GDPR consent: ✓ explicit                      │
│                                                  │
│ Diacritic check: Lazăr ✓ Petrescu ✓ Iancu ✓      │
│                                                  │
│ ┌─────────────────────────────────────────────┐ │
│ │   ✓ ÎNROLEAZĂ                                │ │
│ └─────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

UX notes:

- Each escalation contact is named + their consent state is visible.
- The cross-tenant ESC route to Maria's school is explicit + EU-GDPR-flagged.
- Diacritic check is a small inline confirmation.

## Screen 5 — Pre-shift evening at home (Sun Feb 1 18:42 EET · Diana's personal iPhone)

```
┌──────────────────────────────────────────────────┐
│ 🏠 diana.lazar-petrescu.personal · personal      │
├──────────────────────────────────────────────────┤
│                                                  │
│  📅 Programul de mâine                            │
│                                                  │
│  Luni 1 Februarie 2027                           │
│  21:00 plecare de acasă (estimate)               │
│  21:50 arrival la depot                          │
│  22:00 clock-in (geofence + biometric)           │
│  22:00-06:30 tură de noapte solo                  │
│                                                  │
│  ── Echipament ──                                │
│  ✓ Toughpad work                                  │
│  ✓ iPhone personal                                │
│  ✓ Brățara lone-worker (de la Vladimir)           │
│  ✓ Termos cu ceai verde                          │
│  ✓ Sandwich (de la Răzvan)                       │
│                                                  │
│  ── Reflectarea ──                               │
│  📝 Notez în jurnal: "prima noapte solo. Calmă.   │
│       Vladimir m-a învățat bine. Mâine voi face   │
│       o cafea românească tradițională la 22:30."  │
│                                                  │
│  Auto-save personal-tenant ✓                     │
│                                                  │
│  📞 Mihai (escalation contact): ✓ activ           │
│  📞 Adriana (HSE consultant): ✓ activ             │
│                                                  │
└──────────────────────────────────────────────────┘
```

UX notes:

- This screen is in Diana's **personal tenant** — note the chip color is different (muted-green, not the firm's muted-blue).
- The journal entry is personal-tenant content; the cross-tenant boundary is preserved.
- Equipment checklist is informal but useful.

## Screen 6 — Alarm de-arm at depot side door (Mon Feb 1 21:54 EET · Securitas wall-mounted touchscreen)

```
┌──────────────────────────────────────────────────┐
│ 🛡 SECURITAS · alarm-cooperative                  │
├──────────────────────────────────────────────────┤
│                                                  │
│  Detected entry attempt: 21:53:42 EET            │
│  Scanned biometric: face                         │
│                                                  │
│  ── Identitate ──                                │
│  Match: Diana Lazăr (Tipografia)                  │
│  Match score: 0.987 (high)                       │
│                                                  │
│  ── Cedar verification ──                        │
│  ✓ shift_scheduled_for_now == true (22:00)        │
│  ✓ competency_unexpired == true                   │
│  ✓ workplace_integration_provisioned == night    │
│  ✓ biometric_low_light_match == true              │
│                                                  │
│  Decizie:                                         │
│  ✓ DE-ARM zonă pressroom                          │
│                                                  │
│  De-armat la: 21:54:18 EET                        │
│                                                  │
│  Audit dual-seal:                                │
│  ✓ tipografia-lazar-petrescu-ro                   │
│  ✓ cz-securitas-alarm-cooperative-tenant-ro      │
│                                                  │
│  ── Light-up profile ──                          │
│  Aplicat: 60% ambient + 100% inspection-station   │
│                                                  │
│  Bun venit, Diana. Tură bună.                    │
│  Bună noapte.                                    │
└──────────────────────────────────────────────────┘
```

UX notes:

- Securitas's UI is its own (they're a cooperative tenant) but the Cedar context is preserved cross-tenant.
- The bilingual welcome is small but human.
- Both tenants' audit checkmarks confirm the dual-seal.

## Screen 7 — Night-shift clock-in (Mon Feb 1 22:00 EET · Diana's Toughpad on press)

```
┌──────────────────────────────────────────────────┐
│ 🏢 tipografia-lazar-petrescu-ro · firma          │
│ 🌙 NIGHT · 👤 SOLO · Diana Lazăr                  │
├──────────────────────────────────────────────────┤
│                                                  │
│  CLOCK-IN · tură de noapte                       │
│                                                  │
│  Geofence: ✓ depot perimeter (18m)                │
│  Biometric: ✓ face_id low-light                   │
│                                                  │
│  Schimbul începe:                                │
│  📅 Luni 1 Februarie 2027 · 22:00:00 EET          │
│                                                  │
│  Lucrare planificată:                            │
│  WO-TIP-2027-02-01-NIGHT-WO-NSAID-batch-2         │
│  38,400 PIL Antibiotice                          │
│                                                  │
│  Off-press operator:                             │
│  Andrei Tăbârcă (a clock-in la 22:00:30)         │
│                                                  │
│  ── Dead-man cadence ──                          │
│  Următorul check-in: 02:00 EET                   │
│  Fereastră răspuns: 60s                          │
│                                                  │
│  ── Echipament check ──                          │
│  ✓ Press cold-startup checklist (next)            │
│  ✓ Substrate disponibil (Munken 70gsm)            │
│  ✓ Securitas zonă de-armat                        │
│  ✓ Brățara lone-worker active                    │
│                                                  │
│  Shift health: ✓ Operațional                     │
│                                                  │
│  ┌─────────────────────────────────────────┐    │
│  │  🟢 START PRESS COLD-STARTUP             │    │
│  └─────────────────────────────────────────┘    │
└──────────────────────────────────────────────────┘
```

UX notes:

- Both chips visible: 🌙 NIGHT + 👤 SOLO.
- Equipment checklist is concrete.
- Next dead-man check-in is named explicitly.

## Screen 8 — Dead-man check-in (Tue Feb 2 02:00:06 EET · Toughpad)

```
┌──────────────────────────────────────────────────┐
│ 🌙 NIGHT · 👤 SOLO · Diana Lazăr                  │
├──────────────────────────────────────────────────┤
│                                                  │
│  ⏰ DEAD-MAN CHECK-IN                             │
│                                                  │
│  Time: 02:00:06 EET                              │
│  Fereastra de răspuns: 54s rămase                │
│                                                  │
│  ┌───────────────────────────────────────────┐   │
│  │                                           │   │
│  │           👤 TAP ⇒ VERIFICĂ                │   │
│  │              I am alive                   │   │
│  │                                           │   │
│  └───────────────────────────────────────────┘   │
│                                                  │
│  Verifică prin: tablet tap + face_id              │
│                                                  │
│  În caz de scapare:                              │
│  ─ → escalation la Mihai (90s)                    │
│  ─ → Securitas alarm-cooperative notify          │
│  ─ → audit dual-seal                              │
│                                                  │
│  Echipament check (auto):                        │
│  ✓ Press running (ΔE 1.2)                        │
│  ✓ Brățara active                                │
│  ✓ Geofence still valid                          │
│  ✓ Sheets count consistent                       │
└──────────────────────────────────────────────────┘
```

UX notes:

- The big tap target is centered and obvious.
- The fallback escalation is named so Diana knows what happens if she misses.
- Equipment health check is automated.

## Screen 9 — Shift handoff (Tue Feb 2 06:30 EET · Toughpad)

```
┌──────────────────────────────────────────────────┐
│ Predare la tură de zi · 06:30 EET                │
├──────────────────────────────────────────────────┤
│                                                  │
│  Predare: Diana Lazăr → Camelia Lazăr             │
│                                                  │
│  ── Bilanț turei ──                              │
│  Lucrare: WO-TIP-2027-02-01-NIGHT-WO-NSAID-bch-2 │
│  Sheets bune: 34,348                              │
│  Sheets voided: 152                               │
│  Lucrare planificată: 38,400 PIL                 │
│  Status: în curs (mai sunt ~ 4,000 pentru zi)    │
│                                                  │
│  ── Evenimente ──                                │
│  ✓ Paper-jam 23:42 (rezolvat în 14 min)           │
│  ✓ ΔE drift 04:18 (corectat la 1.2)               │
│  ✓ Dead-man check-in 02:00 ✓                      │
│  ✓ Dead-man check-in 06:00 ✓                      │
│                                                  │
│  ── Pregătire pentru ziua ──                     │
│  Ink levels: ✓ suficient                         │
│  Substrat: ✓ stoc verificat                       │
│  Inspection station: ✓ lighting 100%             │
│                                                  │
│  ── Note ──                                      │
│  📝 "Camelia, batch Antibiotice — recall din      │
│       februarie 2027 încă activ. Roller #4        │
│       înlocuit a fost. Test ΔE la cold-startup    │
│       0.9. Re-run pe slot Heidelberg #2 nu este   │
│       necesar; toate sheet-urile produse sunt în  │
│       toleranță. Le predau cu încredere."         │
│                                                  │
│  Camelia confirmă (passkey + face_id) ✓          │
│                                                  │
│  Audit dual-seal: ✓                              │
│                                                  │
│  ┌─────────────────────────────────────────┐    │
│  │  📋 CONFIRM PREDARE                      │    │
│  └─────────────────────────────────────────┘    │
└──────────────────────────────────────────────────┘
```

UX notes:

- The handoff carries everything Camelia needs: events + counts + notes + cross-link to j157 (the February 2027 recall).
- Both signatures required for handoff completion.

## Screen 10 — Night-shift premium payslip (Fri Feb 5 14:00 EET · Diana's personal mobile)

```
┌──────────────────────────────────────────────────┐
│ 🏠 diana.lazar-petrescu.personal · personal      │
├──────────────────────────────────────────────────┤
│                                                  │
│  📋 PAYSLIP · Februarie 2027 · S1                 │
│                                                  │
│  Tipografia Lazăr-Petrescu SRL                   │
│  IF 12345678 (firma)                             │
│                                                  │
│  ── Ore lucrate ──                               │
│  Tură noapte solo (Luni 22:00-06:30): 8.5h        │
│  Tură zi normală (alte zile): 32h                │
│                                                  │
│  ── Plată ──                                     │
│  Bază zi (47 RON/h × 32h): 1,504.00 RON           │
│  Bază noapte (47 RON/h × 8.5h): 399.50 RON        │
│  Premium noapte 25% (RO §126): +99.88 RON         │
│                                                  │
│  Brut total: 2,003.38 RON                        │
│                                                  │
│  Deduceri:                                       │
│  CAS (10%): -200.34 RON                          │
│  CASS (5.5%): -110.19 RON                        │
│  Impozit pe venit (10% după deducere): -169.28 RON│
│                                                  │
│  Net: 1,523.57 RON                               │
│                                                  │
│  ── Transfer ──                                  │
│  La BCR cont personal (IBAN ROxx...): 1,523.57    │
│  Data efectivă: Vineri 5 Februarie 2027           │
│                                                  │
│  ── ANAF raport ──                               │
│  ✓ Premium noapte raportat la ANAF (RO-tenant)    │
│                                                  │
└──────────────────────────────────────────────────┘
```

UX notes:

- Payslip is on Diana's **personal** tenant (the firm's payroll system pushes to her personal-tenant on her opt-in).
- Night-shift premium is broken out so Diana can verify the +25% applied correctly.
- ANAF reporting is confirmed inline.

## Locale + accessibility

- Diana's locale: `ro-RO` primary; `hu-HU` (with Vladimir); `en-GB` secondary
- Diacritic policy: UTF-8 NFC throughout for Romanian + Hungarian (Csikós, Tăbârcă) + cross-link Romanian/Hungarian phrase handling
- Glove mode: capacitive touch threshold increased; tap targets ≥48dp; haptic feedback on every confirm
- Color tokens: firm-tenant chip muted-blue (#2A6F97); Securitas-tenant chip slate (#37474F); HSE-tenant chip teal (#26A69A); personal-tenant chip muted-green (#3FA34D)
- Night-shift mode UI: 60% brightness; high-contrast (6:1 minimum); larger fonts; orange accent for caution-required actions
- Inspection-station UI overlay: 100% brightness (independent of main lighting)
- Voice fallback: Romanian + Hungarian voice input available
- Lone-worker bracelet pairs to personal iPhone via Bluetooth; tablet listens via internal radio

## Failure-mode UX

| Failure | UX response |
|---|---|
| Competency missing on solo-action | Hard refusal with competency-class shown; alternate path: schedule re-assessment |
| Dead-man check-in missed | Auto-escalation walk; Mihai's mobile rings; Securitas alerted |
| Personal-tenant escalation contact consent revoked | Pre-shift warning; alternate fallback chain proposed |
| Geofence breach mid-shift | Soft alert + audit; if persistent, escalation |
| Securitas alarm-cooperative API unreachable | Local cached competency state used; manual de-arm fallback via keycode (Cedar-gated emergency path) |
| Biometric low-light fail | PIN fallback offered; two retries before lockout |
| Cross-journey identity mismatch | Hard refusal; manual verification required |
| Night-shift premium calc drift | Carmen's payroll module flags before pay-cycle close; Diana can dispute via `messenger` |

## Stop condition

The UX flow is correct when Diana can complete the 6-day final-onboarding-phase + 8.5h first-solo-night-shift in Romanian primary locale with diacritic + Hungarian + cross-link-to-j157 fidelity preserved across all persisted fields, when the night-shift + lone-worker mode chips are persistent throughout her solo-shift screens, when the dead-man check-in 60s window functions correctly with the cross-tenant escalation path tested, when the Securitas alarm-cooperative cross-tenant de-arm works without ambient access, and when the night-shift premium pay correctly applies per RO Codul Muncii §126.
