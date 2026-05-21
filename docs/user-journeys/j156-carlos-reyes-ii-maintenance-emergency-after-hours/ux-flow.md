---
doc_class: User-Journey-UX-Flow
journey_id: j156-carlos-reyes-ii-maintenance-emergency-after-hours
date: 2026-05-20
authority_tier: 2
status: draft
---

# j156 — UX flow: 2:47 AM page → 09:05 MST signed-off

Three device contexts, two persona lenses (Carlos field-mobile + Priya NOC-control + Tomás manager-mobile), one consistent active-tenant pill behavior, and an emergency-dark theme that respects retinal scotopic-vision physics during night-shift work.

Carlos's primary device is the **Samsung XCover7 Pro** — IP68/IP69K rugged, MIL-STD-810H, drop-tested to 1.8m, glove-touch screen, dedicated red SOS button on the right edge, and (critical for this journey) a latent-print fingerprint reader on the power button so an arc-flash-gloved technician can authenticate by removing one glove only briefly. The phone's lock-screen is set to the emergency-dark theme (#0A0A0A background, #F5A623 critical glyph, #4A90E2 secondary action) — designed to preserve dark adaptation when Carlos wakes up at 02:47 MST.

## Screen 1 — Lockscreen page (02:47 MST · Pixel XCover7 Pro)

```
┌─────────────────────────────────────────┐
│ 🌙 02:47 · Sat Oct 17                   │
│                                         │
│                                         │
│   🚨  P1 · DC-PHX-3                    │
│                                         │
│   aisle 7B · chiller-loop overtemp     │
│                                         │
│   ΔT  14.2°F  ╱  cap  6.0°F            │
│   4 racks @ 88°F intake                 │
│                                         │
│   auto-shed in                          │
│       ⏱  11:47                         │
│                                         │
│   ┌───────────┐    ┌───────────┐       │
│   │   ✓  YES  │    │   ✕  NO   │       │
│   └───────────┘    └───────────┘       │
│                                         │
│   Cascade · MeridianStack (scoped)      │
└─────────────────────────────────────────┘
```

UX notes:

- The countdown is a live digit; rolls every 100 ms. The eye locks on this instantly.
- The Yes button is on the right (dominant-hand thumb reach). The No button is left and slightly smaller — discouraging accidental decline.
- The active-tenant pill at the bottom shows the cross-tenant scope BEFORE the grant is fully active — this is honest: the grant is provisional until ack.
- Pulling down on the alert reveals telemetry sparklines (last 30 min ΔT curve, last 6 hr loop pressure).
- A long-press on the alert (≥600 ms) opens "delegate to backup on-call" — for the rare case Carlos cannot respond. The delegate cascades through the escalation tree.
- Voice-activated yes: saying "ack confirm Carlos" while holding the SOS button works hands-free.

## Screen 2 — Permit drawer (02:51 MST · Pixel XCover7 Pro)

```
┌─────────────────────────────────────────┐
│ ◀ Incident · ack confirmed              │
├─────────────────────────────────────────┤
│  PERMIT-TO-WORK · draft                 │
│  permit-dc-phx-3-2026-10-17-0251-7b     │
│                                         │
│  📍 aisle 7B · 7B-CHL-02 · 7B-PUMP-04   │
│  ⚡ 480V/3φ · NFPA-70E Cat-2           │
│  ❄️ R-454B in loop · EPA-608 required   │
│  🔒 LOTO required                       │
│                                         │
│  ─── CO-SIGNERS ───                     │
│                                         │
│  Cascade manager                        │
│  ☐ Tomás Alvarado (awaiting…)           │
│                                         │
│  Host NOC controller                    │
│  ☐ Priya Subramanian (awaiting…)        │
│                                         │
│  ─── YOUR CERTS ───                     │
│  ✓ EPA-608 Universal · exp 2027-04-18   │
│  ✓ NFPA-70E Cat-2 · exp 2027-01-12      │
│  ✓ OSHA-30 General Industry             │
│                                         │
│  valid 02:51 → 09:00 MST                │
│                                         │
│  Cascade · MeridianStack (scoped)       │
└─────────────────────────────────────────┘
```

UX notes:

- Co-signer status is **live** — when Tomás signs at 02:51:42, the ☐ flips to ✓ in 320 ms.
- Cert verification badges are clickable; tapping pulls the actual `learning-management` cert PDF.
- The phone vibrates twice on each co-sign — once on Tomás, once on Priya — so Carlos knows without looking.
- "Cascade · MeridianStack (scoped)" pill is now bold (grant fully active) vs italic (provisional).

## Screen 3 — Driving CarPlay (02:54–03:11 MST · F-150)

```
┌─────────────────────────────────────────┐
│ DC-PHX-3 · 19.2 mi · 16 min             │
│ Loop 101 S → I-10 W → 35th Ave          │
│                                         │
│  🚨 active incident                     │
│      ack at 02:48                       │
│      auto-shed in 08:21                 │
│                                         │
│  Cascade · MeridianStack (scoped)       │
└─────────────────────────────────────────┘
```

UX notes:

- CarPlay shows the incident header always pinned. The countdown ticks visibly.
- Voice-only interactions: "hey oya, ETA to Priya" → "9 minutes, sent". This is the messenger update.
- No screen-typing — illegal anyway, but the system also refuses to render input fields while `speed > 5 mph`.

## Screen 4 — Mechanical-room work view (03:19–06:48 MST · Pixel XCover7 Pro · glove mode)

```
┌─────────────────────────────────────────┐
│ ◀ Permit · co-signed active             │
│  permit-dc-phx-3-2026-10-17-0251-7b     │
├─────────────────────────────────────────┤
│  TASKS  ● 2 done · 9 pending            │
│                                         │
│  ✓ 1 drive-to-site (auto)               │
│  ✓ 2 badge-in (auto)                    │
│  ▶ 3 ladder-setup                       │
│  ☐ 4 lockout-tagout                     │
│  ☐ 5 condensate-line-inspection         │
│  ☐ 6 pump-rebuild                       │
│  ☐ 7 refrigerant-recovery               │
│  ☐ 8 post-leak-test                     │
│  ☐ 9 re-energize                        │
│  ☐ 10 log-in-CMMS                       │
│  ☐ 11 sign-permit-closeout              │
│                                         │
│  ┌─────────────────────────────────┐    │
│  │  📷  capture photo evidence    │    │
│  └─────────────────────────────────┘    │
│                                         │
│  auto-shed countdown PAUSED             │
│  (technician on-site)                   │
└─────────────────────────────────────────┘
```

UX notes:

- Glove-mode increases touch-target sizes by 1.4× and accepts capacitive thresholds for nitrile + leather gloves.
- The active task is highlighted with a thick left border and a small ▶ glyph.
- Photo capture launches the camera with auto-EXIF including GPS + timestamp + task-id + permit-id; the photo never lives on the device's general camera roll — only the `tasks` evidence vault.
- Auto-shed countdown PAUSES when a technician is on-site (incident-management's policy under ADR-0263).

## Screen 5 — LOTO state machine drawer (03:21–03:23 MST)

```
┌─────────────────────────────────────────┐
│ ◀ Task 4 · lockout-tagout · active     │
├─────────────────────────────────────────┤
│  PNL-7B-04 · 480V/3φ/60Hz               │
│                                         │
│  STATE: ◯─◯─●─◯─◯                       │
│         │ │ │ │ └─ locked_isolated_v…   │
│         │ │ │ └── tested_voltage_a…     │
│         │ │ └──── personal_lock_appl…   │
│         │ └────── disconnect_open       │
│         └──────── lockout_pending       │
│                                         │
│  current: tested_voltage_absent         │
│                                         │
│  Fluke T6-1000                          │
│  Phase A:  0.0 V                        │
│  Phase B:  0.0 V                        │
│  Phase C:  0.0 V                        │
│                                         │
│  📷  voltage-tested.heic ✓              │
│                                         │
│  ┌─────────────────────────────────┐    │
│  │ ADVANCE → locked_isolated_v…    │    │
│  └─────────────────────────────────┘    │
│                                         │
│  Carlos R. (Cascade) + Priya S. (NOC)   │
│  joint observer  ✓ ✓                    │
└─────────────────────────────────────────┘
```

UX notes:

- The state-machine dots are unidirectional. A direct skip to `energized_normal` is impossible from the UI; the underlying `workflow-engine` also refuses (and seals a deny audit).
- Voltage readings are transcribed via Fluke's Bluetooth handshake; if the meter is offline, Carlos types them manually with a forced photo of the meter face.
- "Joint observer" line shows that Priya is watching live; the LOTO ceremony is observed by NOC per ASHRAE-15.

## Screen 6 — EPA-608 disclosure form (03:35–03:43 MST)

```
┌─────────────────────────────────────────┐
│ ◀ Disclosure · 40 CFR Part 82 Subpart F │
├─────────────────────────────────────────┤
│  ⚠ REFRIGERANT RELEASE ≥ 1 LB           │
│                                         │
│  refrigerant     R-454B                 │
│  cylinder        R454B-CYL-DC-PHX-3-    │
│                  2026-Q3-007            │
│  release est     1.4 lb                 │
│  cause           shaft seal failure     │
│  first observed  03:34:42 MST           │
│  location        DC-PHX-3 aisle 7B      │
│                  7B-PUMP-04             │
│                                         │
│  📷 cylinder-label ✓                    │
│  📷 leak-site ✓                         │
│  📷 recovery-unit ✓                     │
│                                         │
│  signed by                              │
│  ✓ Carlos Reyes II · EPA-608 Universal  │
│  ✓ Cascade FM tenant attestation        │
│  ✓ MeridianStack site attestation       │
│                                         │
│  ┌─────────────────────────────────┐    │
│  │  SUBMIT TO EPA E-GGRT           │    │
│  └─────────────────────────────────┘    │
│                                         │
│  After submit: do NOT close incident    │
│  until E-GGRT receipt received          │
└─────────────────────────────────────────┘
```

UX notes:

- The "≥1 LB" badge is the only red glyph on the screen. Releases under 1 lb show the same form but in amber and require only Cascade attestation.
- Photo fields cannot be skipped. The submit button stays disabled until all three required photos + the three attestations + the cause field are filled.
- The "do NOT close incident until E-GGRT receipt received" warning prevents a common bug pattern in legacy CMMS where the incident closes before the regulator commits, leaving an orphaned audit-event.

## Screen 7 — Priya at NOC (Chandler control room)

Priya's setup: three 27" monitors (Dell U2722DE) on a sit/stand desk, Plantronics headset, the MeridianStack ops dashboard on the left, the chiller-loop telemetry on center, and the messenger thread + permit + post-mortem template on right.

```
┌──── monitor 1 · ops dashboard ──────────┐
│ DC-PHX-3 status: 🟡 INCIDENT             │
│ active incident: P1 HVAC aisle 7B       │
│ on-site tech: Carlos R. (Cascade)       │
│ permit: co-signed active                │
│ auto-shed: PAUSED                       │
│                                         │
│ telemetry continuity: 100% ✓            │
│ PII data plane: nominal ✓               │
│ HIPAA fac-control audit: live           │
└─────────────────────────────────────────┘

┌──── monitor 2 · telemetry ──────────────┐
│ chl-loop-7b · ΔT graph (live)           │
│                                         │
│ 14.2 ●╲                                 │
│       ╲                                 │
│        ╲___                             │
│            ╲                            │
│ 8.0          ╲___                       │
│                  ●─── 5.8 (06:47)       │
│                                         │
│ ─────────────────────────────           │
│ 02:47 … 03:00 … 04:00 … 05:00 … 06:47   │
│                                         │
│ 7B-PUMP-04 status: re-energized · OK    │
└─────────────────────────────────────────┘
```

UX notes:

- The "auto-shed PAUSED" indicator is amber, not green — Priya should still pay attention.
- The telemetry graph is the truth source; everything else is secondary.
- Priya's permit-signature flow includes a live "joint-observe LOTO" toggle which she activates when Carlos starts task #4.

## Screen 8 — Closeout drawer (06:48–06:57 MST)

```
┌─────────────────────────────────────────┐
│ ◀ Permit · pending closeout signatures  │
├─────────────────────────────────────────┤
│  permit-dc-phx-3-2026-10-17-0251-7b     │
│                                         │
│  lifecycle:                             │
│  ✓ created             02:51:14         │
│  ✓ co-signed           02:53:08         │
│  ✓ LOTO locked         03:23:14         │
│  ✓ work complete       06:47:11         │
│  ▶ closeout signed     (you)            │
│  ☐ closeout co-sign    (Tomás)          │
│  ☐ closeout co-sign    (Priya)          │
│                                         │
│  87 audit events ✓                      │
│  EPA-608 disclosure filed ✓             │
│  egrt-receipt-2026-10-17-…01 ✓          │
│  WO closed in CMMS ✓                    │
│                                         │
│  ┌─────────────────────────────────┐    │
│  │  SIGN CLOSEOUT                  │    │
│  └─────────────────────────────────┘    │
└─────────────────────────────────────────┘
```

## Screen 9 — Cross-grant expiration banner (09:00:00 MST)

```
┌─────────────────────────────────────────┐
│ ⏱ Cross-tenant grant expired 09:00 MST  │
│                                         │
│ Cascade FM · MeridianStack (expired)    │
│                                         │
│ You can no longer act inside            │
│ MeridianStack tenant. The incident      │
│ is closed and the work order is logged. │
│                                         │
│ ┌─────────────────────────────────┐     │
│ │  OK — return to Cascade view    │     │
│ └─────────────────────────────────┘     │
└─────────────────────────────────────────┘
```

UX notes:

- The expiration is visible — no silent revocation; Carlos sees the tenant scope ending.
- The active-tenant pill across all screens drops the `· MeridianStack (scoped)` suffix in unison.

## Locale + accessibility

- Carlos's locale: `en-US-AZ` primary; `es-MX` secondary (his messenger thread with Tomás is in Spanish; the permit and EPA forms are in English per regulator requirement)
- Font: SF Pro 17pt body / 22pt header on phone; system default
- Color contrast: meets WCAG AA in emergency-dark theme; large-tap targets ≥48dp; voice-control fallback at every screen
- Screen-reader compatibility: TalkBack tested; permit drawer is fully labeled
- The phone vibrates with TWO distinct patterns: long-three-pulse for P1 page, short-two-pulse for co-sign events; muscle memory is honored

## Failure-mode UX

| Failure | UX response |
|---|---|
| Loss of LTE in mechanical room | Switch to "offline-queue" indicator (yellow chip top-right); tasks queue locally; flush on reconnect; user is told "23 events queued" |
| Camera fails | Manual-entry fallback with photo retry hint; permit advance disabled until photo captured |
| Permit co-signer unreachable for >5 min | UI surfaces "escalate to alternate co-signer"; the escalation tree provides 2 alternates per role |
| Cedar evaluation timeout | UI shows red "policy service degraded — retry"; never silently degrades to allow |
| EPA E-GGRT submission fails | UI shows red banner with explicit "retry submit" and "save draft locally"; never silently drops |

## Stop condition

The UX flow is correct when Carlos can complete the 6h18m journey wearing arc-flash PPE, in a loud mechanical room, with intermittent LTE, on glove-mode capacitive input, using voice-fallback when both hands are wrenching, and never face an ambiguous screen, dead-end button, or silently-failed audit-seal.
