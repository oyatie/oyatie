---
doc_class: User-Journey-UX-Flow
journey_id: j155-stefan-kovacs-college-night-shift-and-finals-week
date: 2026-05-20
authority_tier: 2
status: draft
---

# j155 — UX flow: Pixel 8a (OSZK) + IdeaPad (BME) + Dell Wyse kiosk

## Device + render targets

| Surface | Device | OS | Form factor | Constraints |
|---|---|---|---|---|
| Stefan — work pocket | Google Pixel 8a (personal-owned, work-MDM lite) | Android 14 + OSZK security profile | 6.1" 1080×2400 | Night-shift dimmed mode; bright sunlight rare (cellar). |
| Stefan — study laptop | Lenovo IdeaPad 5 14" | Linux Mint 22 Xfce | 14" 1920×1080 | Personal-owned; tenant-switching front-end is the OSZK-BME workspace shell. |
| OSZK shift kiosk | Dell Wyse 5070 thin client | Wyse ThinOS 9.5 | 24" 1080p touchscreen on staff-entrance pillar | Locked-down kiosk; only OSZK tenant; NFC reader + iris scanner; never personal/BME |
| OSZK PTT radio | Motorola TLK-100 | proprietary | 5.7" PoC handheld | Voice + presence only; OSZK tenant principal binding |
| Csilla — supervisor | iMac M3 24" | macOS 15 | desktop | OSZK security control room |
| Réka — at home (sick) | iPhone 15 (personal) + OSZK tenant via app | iOS 18 | mobile | Bedridden flu |
| Bálint — classmate | Google Pixel 7 | Android 14 | mobile | BME student tenant active |

## Locale + RTL

Default locale: `hu-HU` (Hungarian). Stefan can flip to `en-US` for class materials (some BME OS course materials are English-only; the LMS does NOT auto-translate them — translation is consented per source, per ADR-0292 cognitive-load protection).

The active-tenant pill colors:

- `personal-stefan-kovacs-hu` — **forest green** with a stylised oak-leaf glyph
- `oszk-security-services_hu` — **OSZK navy** with the National Library wordmark in cream
- `bme-student-bodv75_hu` — **BME purple** with the BME crest in gold
- `bme-research-cohort-2026-sleep-grade-fall` — never shown to Stefan (research cohort tenants are invisible to subjects; they only see consent prompts)

Switching requires 2-second deliberate hold. Switching is BLOCKED while clocked in to OSZK from any device that is logged in to OSZK — except study devices like the IdeaPad that have a separate session entirely. (The Pixel's OSZK session must clock out before Pixel can switch tenants; the IdeaPad starts in personal/BME and never gains OSZK on Stefan's plan.)

## Screen-by-screen progression

### Screen 1 — OSZK kiosk: Stefan's NFC tap-to-clock-in (21:48 CET Sunday)

The Dell Wyse 5070 standing at the staff entrance shows the OSZK shift-start UI in `hu-HU`:

```
+--------------------------------------------------+
|  OSZK Biztonsági szolgálat                       |
|                                                  |
|  ╔══════════════════════════════════════════╗   |
|  ║                                          ║   |
|  ║       érintse a kártyáját ide            ║   |
|  ║       vagy az okostelefonját              ║   |
|  ║                                          ║   |
|  ║      [   NFC olvasó zóna   ]              ║   |
|  ║                                          ║   |
|  ╚══════════════════════════════════════════╝   |
|                                                  |
|  Idő: 2026.12.14 21:48 CET                      |
|  Helyszín: H-1014 Bp. Szent György tér 4-6     |
+--------------------------------------------------+
```

Stefan taps his Pixel to the NFC zone. The pixel briefly shows on its own lock screen:

```
[OSZK Biztonsági szolgálat] szeretne belépést megerősíteni
[ Megerősít ]  [ Mégse ]
```

He taps **Megerősít**. The kiosk transitions:

```
+--------------------------------------------------+
|  Kovács István — éjszakai őr                     |
|  Mai műszak: 22:00 → 06:00 (8 óra)              |
|                                                  |
|  Heti óraszám 7 napos átlag: 22.0 / 48          |
|  Pihenőidő utolsó műszak óta: 47 óra ✓          |
|                                                  |
|  [ Műszak megerősítése ]                         |
|  [ Hibajelzés ]                                  |
+--------------------------------------------------+
```

Stefan presses **Műszak megerősítése**. Toast: *"Megerősítve. Jó műszakot!"* (Confirmed. Have a good shift!)

### Screen 2 — Pixel notification: Réka's swap offer (22:14 CET)

The Pixel pulses gently. Notification (active tenant indicator on the notification: small OSZK navy dot):

```
🔔 OSZK Messenger — Hahn Réka
"Szia Stefán, kérlek-kérlek vedd át a keddi műszakomat? 22-06.
Influenza, lázam 38.7. Nagyon-nagyon hálás lennék 🙏"

[Megnyitás]   [Csendesít 1 órára]
```

Stefan taps **Megnyitás**. Messenger opens to the thread with Réka. The top bar of the messenger has a slim OSZK navy ribbon: *"OSZK környezet — BME tenant nincs csatlakoztatva"* (OSZK environment — BME tenant is not connected).

This banner is non-removable while the user is in the OSZK messenger. It explicitly tells Stefan: any decline message he writes can only reference OSZK-tenant information.

### Screen 3 — Pixel messenger: Stefan composes the decline

The compose surface, with active-tenant pill always visible at top:

```
+----------------------------------------------------+
| 🛡 OSZK Biztonsági szolgálat                         |
| Hahn Réka                                          |
|----------------------------------------------------|
| Réka: "Szia Stefán, kérlek-kérlek..."             |
|                                                    |
| [Compose box]                                      |
|                                                    |
|  Réka, nagyon sajnálom, kedden nem tudok.         |
|  Csütörtök reggel ki tudok jönni helyetted,       |
|  ha az segít. Jobbulást!                          |
|                                                    |
| 🔒 OSZK tenant — BME naptár nem hozzáférhető      |
|                                                    |
|   [Cancel]            [Küldés]                    |
+----------------------------------------------------+
```

Note the lock pill at the bottom: it tells Stefan in plain Hungarian that his BME calendar is not accessible from here. This is the **dual-role protection guardrail** as visible UI — preventing him from typing "I have an OS final" by reminding him that OSZK doesn't know about BME.

Stefan taps **Küldés**. Toast: *"Elküldve."* The thread updates.

### Screen 4 — IdeaPad opens (22:18 CET): tenant-switcher modal

Stefan opens the Lenovo IdeaPad on the guard desk. Logged-in active tenant: `Personal — Stefan Kovács`. Top bar shows a forest-green pill with an oak-leaf glyph.

Stefan clicks the pill. Modal:

```
+----------------------------------------------------+
| Aktív környezet váltása                            |
|                                                    |
| Jelenlegi: 🌳 Személyes (Stefan Kovács)           |
|                                                    |
| Választható környezetek (ehhez az eszközhöz       |
| jóváhagyva):                                       |
|                                                    |
|   ○ 🎓 BME hallgató — bme-student-bodv75_hu       |
|     [tartsd nyomva a gombot 2 mp-ig]              |
|                                                    |
|   ✗ 🛡 OSZK munkavállaló — nem engedélyezett      |
|     ebből az eszközből                            |
|     (Ez egy személyes eszköz; csak a Pixelről     |
|      és a kioszkról elérhető OSZK)                |
|                                                    |
| Eszköz: Lenovo IdeaPad 5 14"                       |
| Másik eszközön aktív: Pixel 8a (OSZK; clocked-in) |
+----------------------------------------------------+
```

The OSZK option is explicitly disabled for this device with a plain-language reason. Stefan presses-and-holds **BME hallgató** for 2 seconds.

After release:

```
Toast: 🎓 BME hallgató környezet aktiválva.
       A háttérszín most BME lila-arany.
```

The desktop background shifts. The wallpaper transitions from a personal photo to the BME Műegyetem rakpart vista at dusk.

### Screen 5 — BME LMS in the IdeaPad workspace shell (22:21 CET)

URL: `https://lms.bme.hu/courses/VIK-AUT-VIIIAB1015/oszi-2026`

Top bar: BME purple pill `🎓 BME hallgató · Kovács István · 2. évfolyam`.

Page:

```
+----------------------------------------------------+
| Operációs rendszerek (VIK-AUT-VIIIAB1015)          |
| Oktató: dr. Halász Gábor                           |
| Félév: 2026/27 ősz · ZH eredmény 1: 78%            |
|----------------------------------------------------|
| Mai cél: Záróvizsga felkészülés — 34 óra van hátra |
|                                                    |
|  📂 Előadás-jegyzetek (12 db)                      |
|  📂 Múlt vizsgák archívuma (14 db)         <-- Stefan clicks
|  📂 Gyakorló feladatok (28 db)                     |
|  📂 Tananyag-PDF Tanenbaum 5e (eng+hun)            |
|                                                    |
|  💬 #os-finals-2026 közösségi csatorna (47 fő, 3 új) |
+----------------------------------------------------+
```

After clicking "Múlt vizsgák archívuma", a list of past exams. Stefan opens Spring 2026 + Fall 2025 in two tabs.

### Screen 6 — BME community channel `#os-finals-2026`

Layout:

```
+--------------------------------+----------------------+
| Channels                       | #os-finals-2026      |
|--------------------------------|----------------------|
| 🔔 #os-finals-2026     (3 új) | Bálint Szabó 22:35   |
| #dm-finals-2026         (12) | Sziasztok, valaki     |
| #sigals-finals-2026     (0)  | emlékszik, hogy       |
| #ca2-finals-2026        (1)  | Halász tanár úr a     |
|                                | memóriafedett...     |
|                                |                      |
|                                | Stefan (compose)     |
|                                | _____________________|
|                                | [Send] 🔒 BME tenant |
+--------------------------------+----------------------+
```

The compose box is locked to the BME tenant — confirmed by the lock pill below the send button. If Stefan tried to attach an OSZK-tenant file (e.g. an OSZK schedule screenshot), the attach dialog would refuse with: *"Csak BME tenant fájljai csatolhatók."* (Only BME tenant files may be attached.)

Stefan types his Tanenbaum-Halász answer and hits Enter. The message appears in the thread within 90ms. MLS commit epoch advances to 42.

### Screen 7 — Dell Wyse 5070: Csilla's intercom alert overlay (22:50)

While the IdeaPad is on BME tenant, the Wyse 5070 (still OSZK tenant) flashes a small toast on the foyer-camera quadrant:

```
🔊 Csilla — szakasz vezető
"Stefán, az ablakon kintről egy mókus..."

[Hangkapcsolat fenn]   [Megválaszolva]
```

Stefan picks up the OSZK PTT radio (separate device, OSZK tenant principal). He replies vocally. The radio acknowledges. The Wyse 5070 dismisses the toast.

**Critical UX**: this OSZK incident did NOT switch the IdeaPad. Devices and tenants are decoupled. Stefan walked the perimeter, returned, sat down, and continued studying on the BME-tenant IdeaPad.

### Screen 8 — IdeaPad finals-week mode banner

A small banner at the top of every BME workspace screen during the Dec 14–19 window (Stefan opted into finals-week mode Dec 1):

```
🎯 Záróvizsga-üzemmód aktív (Dec 14–19)
   Nem sürgős értesítések szüneteltetve mindkét környezetben.
   Csak vészjelzések (OSZK riasztások, családi vészjelzés) jutnak át.
   [Beállítások]    [Kikapcsolás]
```

The banner is one line tall, in muted BME purple, dismissable for 24h but auto-reappears at next BME-tenant login if the period is still active.

### Screen 9 — Pixel 8a at 04:35 CET: vibration alarm + perimeter walk

Stefan's Pixel buzzes (vibration mode — no sound; he's still on shift, on rest). Lockscreen:

```
🚶 Idő a kerületi ellenőrzéshez (4 sarok)
04:35 → várható befejezés 04:45
[ Megnyitás ] [ Csendesít 5 perc ]
```

He taps Megnyitás. The Pixel opens the OSZK perimeter-walk checklist (tenant: OSZK).

```
+----------------------------------------------------+
| 🛡 OSZK Kerületi ellenőrzés #19                    |
|                                                    |
| Pipálj minden ellenőrzött pontot:                  |
|  ☐ Fő olvasóterem előcsarnok                       |
|  ☐ Régi-könyv páncélszoba külső ajtó              |
|  ☐ Kézirat-folyosó                                 |
|  ☐ Személyzeti bejárat                             |
|                                                    |
| [ Helyszín-pecsétek (NFC) ]                        |
+----------------------------------------------------+
```

Each corner has an NFC sticker; Stefan must tap the Pixel to each one. The four taps complete in 11 minutes.

### Screen 10 — Tuesday payroll-bridge notification (21:00 CET)

While Stefan is in his apartment in Újpest (post-exam, evening), his Pixel chimes once (personal-tenant style — gentle bell, not the OSZK alarm tone). Notification (personal-tenant forest-green dot):

```
💰 Fizetési értesítés (Személyes tenant)
Nettó fizetés: HUF 124 900 megérkezett az MKB-számládra.
Tandíj részlet (3/4) HUF 187 500 automatikusan teljesítve a BME-felé.
Részletek: [ Megnyitás ]
```

He taps Megnyitás. Personal-tenant payments overview:

```
+----------------------------------------------------+
| 🌳 Személyes — Stefan Kovács                       |
|                                                    |
| Fizetés december 2026                              |
|                                                    |
|  Bruttó (OSZK):              HUF 488 000           |
|  SZJA:                      − HUF  73 200          |
|  Egészségbiztosítás + nyugdíj − HUF  92 400        |
|  ────────────────────────────────────              |
|  Nettó:                       HUF 312 400          |
|                                                    |
|  Automatikus tandíj-levonás (BME):                 |
|    Részlet 3 / 4              − HUF 187 500        |
|  ────────────────────────────────────              |
|  Te kézhez kapod:             HUF 124 900          |
|                                                    |
| Forrás: OSZK Biztonsági szolg. → MKB Bank          |
| Cél: MKB-számla HU48 1077 1717 1234 5678 0000 0001 |
| Tranzakció ID: tr-payroll-bridge-2026-12-16-stefan |
|                                                    |
| BME oldal: [ Tandíj státusz megnézése ]            |
+----------------------------------------------------+
```

The "Tandíj státusz megnézése" link does NOT cross tenants silently. It opens a tenant-switcher prompt: *"Át kell váltanod BME környezetre. Tartsd nyomva 2 mp-ig:"* before showing the BME billing page.

### Screen 11 — Friday post-exam: BME billing view

After Stefan returns from the Discrete Math II exam, he switches tenant to BME on his Pixel and opens the billing surface.

```
+----------------------------------------------------+
| 🎓 BME hallgató — Kovács István · 2. évfolyam     |
|                                                    |
| Tandíj 2026/27 őszi félév                          |
|                                                    |
|  Részlet 1/4   HUF 187 500   TELJESÍTVE   2026-10-16 |
|  Részlet 2/4   HUF 187 500   TELJESÍTVE   2026-11-16 |
|  Részlet 3/4   HUF 187 500   TELJESÍTVE   2026-12-16 ✓ |
|  Részlet 4/4   HUF 187 500   esedékes     2027-02-15 |
|                                                    |
|  Aktív státusz: Hallgató — második évfolyam       |
|  Tavaszi 2027 félévre beíratva ✓                  |
+----------------------------------------------------+
```

A small footer note: *"A részletek automatikus levonással teljesülnek az OSZK munkáltatói béred terhére (megállapodás 2026-10-04). Bármikor felfüggesztheted a [ Beállítások / Standing Instructions ] menüben."*

(Installments are paid automatically via deduction from your OSZK salary per the 2026-10-04 agreement. You can suspend at any time in Settings / Standing Instructions.)

### Screen 12 — OSZK Csilla's denial — attempted cross-tenant probe (admin curiosity)

A back-office UX worth documenting because it's a Cedar deny moment captured from the admin's perspective. Csilla (out of misplaced curiosity, NOT routine, NOT lawful) tries on her iMac to look up "what does Stefan do on his breaks?" via a vendor analytics tool that asks for cross-tenant insight.

She opens the OSZK analytics surface and types:

```
Lekérdezés: stefan.kovacs — minden tenant — utolsó 7 nap aktivitás
```

The system replies:

```
+----------------------------------------------------+
| ⛔ Hozzáférés megtagadva                          |
|                                                    |
| Az OSZK rendszergazdai szerepköre nem tartalmaz   |
| jogot Kovács István más környezeteibe való        |
| bepillantásra. Ez a határ az ADR-0311 (kettős     |
| tenant azonosság) és a HU-Munka Tv. 11/A. §       |
| (munkavállaló privát szférája) alapján van        |
| beállítva.                                         |
|                                                    |
| Ha munkajogi vagy lett alapod van, kérdezd meg a  |
| jogtanácsost; szabályos eljárásban (bírósági       |
| végzés, GDPR Art 6(1)(c) jogi kötelezettség) más   |
| út is lehet.                                       |
|                                                    |
| Audit-esemény: EVT-J155-CEDAR-DENY-CROSS-TENANT-   |
|                LMS-PROBE-005                       |
|                                                    |
| [Vissza]   [Miért van ez a határ?]                |
+----------------------------------------------------+
```

The "Miért van ez a határ?" link opens a 200-word plain-Hungarian explainer rooted in ADR-0311 + Hungarian labour law. Csilla closes the window. Audit event seals in OSZK tenant. Stefan's BME/personal tenants are unaware Csilla attempted this — they only see that someone tried (the deny event seals an opaque attempt-counter on BME but no payload that could reveal Csilla's identity).

## Critical state transitions

| Trigger | From state | To state | Side-effect |
|---|---|---|---|
| NFC tap kiosk | NOT-SHIFT | SHIFT-CONFIRMED | calendar.confirm_shift; OSZK audit |
| Réka's swap message arrives | SHIFT-CONFIRMED | SHIFT-WITH-SWAP-OFFER | OSZK messenger push |
| Stefan declines | SHIFT-WITH-SWAP-OFFER | SHIFT-NO-SWAP | OSZK audit |
| Tenant-switch on IdeaPad to BME | BME-INACTIVE | BME-ACTIVE-STUDY | dual-tenant session table updated |
| LMS read | BME-ACTIVE-STUDY | BME-ACTIVE-STUDY (idempotent) | BME audit; OSZK has no view |
| Csilla's intercom + perimeter walk | BME-ACTIVE-STUDY (parallel) | OSZK-PERIMETER-WALK (parallel) | OSZK audit |
| Vibration alarm 04:35 | BME-ACTIVE-STUDY | OSZK-PERIMETER-WALK | OSZK audit × 4 NFC |
| Shift clock-out | SHIFT-CONFIRMED | OFF-SHIFT | OSZK audit; WTD rest timer arms |
| Payroll bridge fire (Tuesday 21:00) | OFF-SHIFT | PAYROLL-PROCESSED | trinity SEPA bridge; audit × 4 |
| Discrete Math final ends | BME-ACTIVE | BME-IDLE | none |
| Csilla cross-tenant probe attempt | OSZK-ANALYTICS-IDLE | OSZK-ANALYTICS-CEDAR-DENIED | OSZK audit (deny) |

## Accessibility specifics

- **Night-shift dim mode**: the Pixel + IdeaPad detect time-of-day + ambient lux and shift to a dimmed, low-blue-light theme between 21:00 and 06:00. Maximum brightness floor: 15%.
- **Bilingual fallback**: any English text in BME OS course materials is preserved; auto-translation is OFF by default. Stefan can request inline translation on a per-paragraph basis, consenting per source.
- **Cognitive-load protection during finals week**: notification volume is gated (only emergency tier passes). The "finals-week mode" banner is dismissable but auto-reappears at next session login during the active period.
- **Wakeup alarm robustness**: the OSZK perimeter-walk reminder uses vibration only (no sound) so Stefan can rest at his desk without waking colleagues over the radio.
- **Single-handed operation**: shift-confirmation + decline-swap + perimeter checklist are all reachable within the right-thumb zone on a 6.1" phone. Even-faster ALT path: hardware-double-press the power button to confirm a perimeter walk.

## Anti-pattern guardrails

1. Never let a personal device gain an OSZK clock-in without an explicit NFC tap + biometric confirm — prevents accidental clock-in.
2. Never display BME tenant data on the OSZK kiosk under any circumstance — even an admin lookup is locked to OSZK tenant.
3. Never auto-translate sensitive surfaces (exam materials, work directives, payroll docs) — translation is per-paragraph, per consent.
4. Never bury the tenant-switch lock-out reason. The disabled OSZK option on the IdeaPad explicitly says "this is a personal device; OSZK is reachable from the Pixel and the kiosk".
5. Never let an OSZK admin's curiosity-probe seal silently. The Cedar deny path emits an audit-event + a transparent denial UI with ADR-0311 citation + Hungarian labour law citation.
6. Never let the payroll bridge release without the 3-way Cedar permit. If any side denies (e.g. BME has marked Stefan as withdrawn), the bridge halts and surfaces a remediation task to HR.
7. Never let Stefan accidentally accept a swap that breaches the EU WTD weekly cap. Calendar swap-accept refuses if cumulative hours would exceed 48.
