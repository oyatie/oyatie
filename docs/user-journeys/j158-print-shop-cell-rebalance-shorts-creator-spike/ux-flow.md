---
doc_class: User-Journey-UX-Flow
journey_id: j158-print-shop-cell-rebalance-shorts-creator-spike
date: 2026-05-20
authority_tier: 2
status: draft
---

# j158 — UX flow: dual-tenant dashboard, disclosure bridge, cell rebalance

Three device + context surfaces:

- Hae-Won's Galaxy S25 Ultra + desk-mounted oyatie tablet (dual-tenant aware; explicit active-tenant pill)
- Lee Min-Jun's dual-monitor desktop (employer-tenant ops console; never sees personal-tenant data)
- The cell-rebalance ops console (shared employer-tenant view across desktops + tablets)

All screens render Hangul + Hanja + diacritics natively (UTF-8 NFC). The active-tenant pill is always visible. The dual-tenant boundary is communicated visually as a hard line — never a soft tint.

## Screen 1 — Personal-tenant autoscale notification (14:18 KST · Galaxy S25 Ultra)

```
┌─────────────────────────────────────────────────┐
│  🌐 personal-haewon-kim-kr · @haewon_paperlife  │
├─────────────────────────────────────────────────┤
│                                                 │
│   📈 AUTOSCALE ENGAGED                          │
│                                                 │
│   short:                                        │
│   "8시간 동안 종이 접는 소리만"                  │
│   "8 hours of folding paper sounds"             │
│                                                 │
│   stats:                                        │
│   ┌─────────────────────────────────────────┐   │
│   │  21.7M views · trending #2 KR           │   │
│   │  watch-through: 94%                     │   │
│   │  cell: kr-seoul-shorts-creator-tier-4   │   │
│   │  scale factor: 8.4×                     │   │
│   └─────────────────────────────────────────┘   │
│                                                 │
│   replicas: 6 → 28 · ready 14:18:42             │
│   estimated burst: 24-96 hours                  │
│                                                 │
│   ⓘ This stays inside your personal tenant.     │
│     Nothing crosses to your employer.           │
│                                                 │
│   ┌─────────────────────────────────────────┐   │
│   │  • optional: send disclosure signal to  │   │
│   │    your employer (info-only)            │   │
│   └─────────────────────────────────────────┘   │
│                                                 │
│   active tenant: personal-haewon-kim-kr         │
└─────────────────────────────────────────────────┘
```

UX notes:

- The active-tenant pill at the bottom is bold (her personal context).
- The "stays inside your personal tenant" line is explicit reassurance — it tells Hae-Won that nothing is leaking.
- The "send disclosure signal" action is suggested but never automated. Hae-Won has full agency.
- Stats are coarse — she sees 21.7M and 94%, the same numbers her audience would see. No internal-only metrics are exposed.

## Screen 2 — Creator-employer disclosure signal modal (14:24 KST)

```
┌─────────────────────────────────────────────────┐
│  ◀ DISCLOSURE SIGNAL — personal → employer      │
├─────────────────────────────────────────────────┤
│                                                 │
│  This is a ONE-WAY INFO-ONLY signal.            │
│                                                 │
│  To:    Sungkyul-Sangsa Print Co. (employer)    │
│         routed to: Lee Min-Jun (사장님) +       │
│                    your work-tenant inbox       │
│                                                 │
│  Disclosure record:                             │
│   ✓ disclosure-haewon-kim-sungkyul-sangsa-     │
│     2024-08-12 · active                         │
│                                                 │
│  The signal carries:                            │
│   • coarse-grained "spike happening" assertion  │
│   • your suggested heads-up timeline            │
│   • optional offer to help                      │
│                                                 │
│  The signal CANNOT carry:                       │
│   ✗ audience PII                                │
│   ✗ revenue figures                             │
│   ✗ audience demographics                       │
│   ✗ DM threads                                  │
│   ✗ payload exceeding 1024 bytes                │
│                                                 │
│  message (Korean + English allowed):            │
│  ┌─────────────────────────────────────────┐   │
│  │ 사장님 안녕하세요. 저의 개인 채널         │   │
│  │ (@haewon_paperlife) 에 올린 종이접기   │   │
│  │ ASMR 영상이 갑자기 바이럴 됐어요 —      │   │
│  │ 지금까지 21.7M views, 한국 #2          │   │
│  │ trending. ...                          │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
│  bytes used: 612 / 1024                         │
│  Hangul preservation check: ✓                   │
│                                                 │
│  ┌─────────────────┐    ┌─────────────────┐    │
│  │  ✕ CANCEL       │    │  📨 SEND SIGNAL │    │
│  └─────────────────┘    └─────────────────┘    │
└─────────────────────────────────────────────────┘
```

UX notes:

- "ONE-WAY INFO-ONLY" line is the dominant header — communicates the boundary explicitly.
- The "cannot carry" list is shown alongside "carries" — explicit refusal pattern, not just a positive whitelist.
- Bytes-used counter is live; helps Hae-Won know she's safely under the cap.
- Hangul preservation check is a live validation — visible reassurance that the system preserved her Korean characters byte-exact.
- Cancel is left, send is right — matches the cultural cue that cancel is "go back" (typical Korean mobile UX).

## Screen 3 — Active-tenant switch (14:34 KST)

```
┌─────────────────────────────────────────────────┐
│  ▼ SWITCH ACTIVE TENANT                         │
├─────────────────────────────────────────────────┤
│                                                 │
│  current: personal-haewon-kim-kr                │
│                                                 │
│  available:                                     │
│  ◯ personal-haewon-kim-kr                       │
│  ● sungkyul-sangsa-print-co-kr                  │
│    role: logistics_coordinator                  │
│                                                 │
│  hold confirm 2 sec to switch ▱▱▱▱▱             │
│                                                 │
└─────────────────────────────────────────────────┘
```

UX notes:

- Explicit 2-second hold on switch — prevents accidental tenant flip.
- The current role in the destination tenant is shown so Hae-Won knows what role she'll act with.
- After switch, the pill changes immediately + a haptic confirms.

## Screen 4 — Employer-tenant cell-rebalance dashboard (14:36 KST · oyatie tablet)

```
┌─────────────────────────────────────────────────┐
│  Sungkyul-Sangsa · 마포-1 · logistics_coord     │
├─────────────────────────────────────────────────┤
│  [TASK · HIGH PRIORITY]                         │
│  cell-rebalance proposed                        │
│  auto-generated from disclosure-signal-…1424    │
│                                                 │
│  state-machine progress:                        │
│  ● capacity_signal_detected (14:34:18)          │
│  ◯ rebalance_proposed                           │
│  ◯ cross_cell_grant_negotiated                  │
│  ◯ traffic_shift                                │
│  ◯ post_rebalance_validation                    │
│                                                 │
│  cells in scope:                                │
│  ┌─────────────────────────────────────────┐   │
│  │ kr-seoul-employer-print-shop-mid-volume │   │
│  │   ● primary · 87% cap utilization        │   │
│  │ kr-seoul-employer-print-shop-burst-1    │   │
│  │   ◯ warm-spare · ETA 22 min to ready    │   │
│  │ kr-seoul-employer-print-shop-burst-2    │   │
│  │   ◯ warm-spare · ETA 22 min to ready    │   │
│  │ kr-seoul-employer-print-shop-secondary  │   │
│  │   ● secondary · 32% util · DR-ready     │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
│  expected order-intake scale: 3.7×              │
│  expected burst window: 96 hours                │
│                                                 │
│  authority required:                            │
│   ✓ logistics_coordinator (you)                 │
│   ☐ owner co-sign (Lee Min-Jun)                 │
│                                                 │
│  ┌─────────────────────────────────────────┐   │
│  │  PROPOSE REBALANCE                      │   │
│  └─────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
```

UX notes:

- State-machine pill is sticky at top.
- Cell utilization is live; primary at 87% is amber-coded.
- "authority required" is explicit; the owner co-sign requirement is visible upfront.

## Screen 5 — Owner co-sign view (14:42 KST · Lee Min-Jun's desktop)

```
┌──── monitor 2 · cell-rebalance proposal ────────┐
│ Sungkyul-Sangsa · owner view                    │
│                                                 │
│ rebalance: rebalance-2027-03-17-1434-…          │
│ proposed by: Hae-Won Kim (logistics)            │
│ context: external_causal_signal_consumer_…       │
│                                                 │
│ ⓘ This rebalance was triggered by a disclosure  │
│   signal from your employee's personal tenant.  │
│   You see ONLY the signal she chose to send.    │
│                                                 │
│ proposal summary:                               │
│  • warm 2 burst cells (~22 min each)            │
│  • allocate 2 reserved capacity units            │
│  • daily reassessment at 04:00 KST              │
│  • expected scale 3.7× · window 96 hr            │
│                                                 │
│ cost impact (estimated):                        │
│  • burst-cell-hours: 192 cell-hr × ₩1,840/hr     │
│  • additional staff overtime: ₩3.2M                │
│  • paper inventory order: ₩4.8M                  │
│                                                 │
│ KR-LSA staff-hours check:                       │
│  • haewon: green · 38.5 hr                      │
│  • park: green-monitor · 47.2 hr                │
│  • lee.minjun: yellow · 51.8 hr (you)           │
│   ⓘ recommendation: redistribute your hours     │
│                                                 │
│ ┌─────────────────┐    ┌─────────────────┐     │
│ │  ✕ DECLINE      │    │  ✓ CO-SIGN      │     │
│ └─────────────────┘    └─────────────────┘     │
└─────────────────────────────────────────────────┘
```

UX notes:

- The explicit "ⓘ This rebalance was triggered by a disclosure signal..." line documents the cross-tenant boundary etiquette for the owner.
- Cost impact in KRW with concrete numbers — supports decision.
- KR-LSA staff-hours check with the owner's own row flagged yellow — protective design that turns the owner's attention to his own overcommitment.
- Decline is intentionally a real option; this is not a rubber-stamp gate.

## Screen 6 — Traffic shift live view (15:02–16:32 KST · ops console)

```
┌─────────────────────────────────────────────────┐
│  TRAFFIC SHIFT · increment 5 of 10              │
├─────────────────────────────────────────────────┤
│                                                 │
│  ramp progress:  [██████░░░░] 50% (50 min)      │
│                                                 │
│  current distribution:                          │
│  ┌─────────────────────────────────────────┐   │
│  │  primary       60% │█████████████░░░░│  │   │
│  │  burst-1       22% │████░░░░░░░░░░░░░│  │   │
│  │  burst-2       18% │███░░░░░░░░░░░░░░│  │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
│  target distribution:                           │
│  ┌─────────────────────────────────────────┐   │
│  │  primary       40% │██████████░░░░░░░│  │   │
│  │  burst-1       32% │████████░░░░░░░░░│  │   │
│  │  burst-2       28% │███████░░░░░░░░░░│  │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
│  live metrics (p95 latency):                    │
│   primary  142 ms │ burst-1 168 ms │ burst-2 173│
│   error rate: 0.04% (threshold 1.5%)            │
│   queued orphans: 0                             │
│                                                 │
│  rollback threshold: latency p95 ≤ 280 ms       │
│   ✓ all cells within bounds                     │
│                                                 │
│  ┌──────────────┐  ┌──────────────────────────┐│
│  │  ⏸ PAUSE     │  │  🔁 ROLLBACK (if needed) ││
│  └──────────────┘  └──────────────────────────┘│
└─────────────────────────────────────────────────┘
```

UX notes:

- The visual side-by-side current vs target distribution is immediately legible.
- Pause and rollback are always visible — emergency-stop options.
- The rollback threshold is named explicitly so operators know the line.

## Screen 7 — Order-intake queue post-shift (16:42 KST)

```
┌─────────────────────────────────────────────────┐
│  ORDER INTAKE · 18 new today (Mon-Wed: 42)      │
├─────────────────────────────────────────────────┤
│                                                 │
│  ▼ filter: today                                │
│                                                 │
│  ┌─────────────────────────────────────────┐   │
│  │ 비즈페이퍼 (Bizpaper Co.)                │   │
│  │ 2,400 business cards · SLA 18:18         │   │
│  │ routed to: Hae-Won (logistics)           │   │
│  │ source: organic web inquiry              │   │
│  ├─────────────────────────────────────────┤   │
│  │ Café Hoso (카페 호소)                    │   │
│  │ 1,800 menu cards · SLA tomorrow 12:00    │   │
│  │ routed to: Park Jae-Won (binding)        │   │
│  │ source: video found-your-shop            │   │
│  ├─────────────────────────────────────────┤   │
│  │ ... (16 more) ...                        │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
│  SLA timer summary: 14 within 4-hour window     │
│                       4 next-day               │
└─────────────────────────────────────────────────┘
```

UX notes:

- The "source: video found-your-shop" tag is visible at order level — operations can see the causal link organically.
- SLA timer summary is the at-a-glance health check.
- Hangul + Hanja in customer names render correctly.

## Screen 8 — Boundary invariant test (17:14 KST · ops console)

```
┌─────────────────────────────────────────────────┐
│  BOUNDARY INVARIANT · scheduled probe           │
├─────────────────────────────────────────────────┤
│                                                 │
│  Probe 1: employer → personal                   │
│  ────────────────────────────                   │
│  GET /v1/tenants/personal-haewon-kim-kr/        │
│       shorts/metrics                            │
│                                                 │
│  result: 403 FORBIDDEN ✓                        │
│  reason: forbid-employer-to-personal            │
│  audit: dual-sealed ✓                           │
│                                                 │
│                                                 │
│  Probe 2: personal → employer (no permit)       │
│  ──────────────────────────────────────         │
│  GET /v1/tenants/sungkyul-sangsa-print-co-kr/   │
│       tasks                                     │
│                                                 │
│  result: 403 FORBIDDEN ✓                        │
│  reason: forbid-personal-to-employer-no-permit  │
│  audit: dual-sealed ✓                           │
│                                                 │
│                                                 │
│  Probe 3: Hangul preservation                   │
│  ─────────────────────────────                  │
│  field write: 김해원 (5 bytes UTF-8)            │
│  field read back: 김해원 ✓ (byte-exact)         │
│  field write: 성결상사 인쇄소 (14 bytes)        │
│  field read back: 성결상사 인쇄소 ✓             │
│                                                 │
│                                                 │
│  ALL INVARIANTS HOLD                            │
└─────────────────────────────────────────────────┘
```

UX notes:

- Probes are visible and ungated — operators can run them on demand.
- Each probe is followed by its actual response.
- "ALL INVARIANTS HOLD" is the binary outcome that builds confidence.

## Screen 9 — End-of-day summary (18:42 KST)

```
┌─────────────────────────────────────────────────┐
│  REBALANCE · post-validation complete           │
├─────────────────────────────────────────────────┤
│                                                 │
│  duration: 4h 24m                               │
│  states: 5/5 ✓                                  │
│  audits: 134 events · merkle coherent ✓         │
│                                                 │
│  cell health:                                   │
│   primary  142 ms p95 · 41% util · stable        │
│   burst-1  168 ms p95 · 33% util · stable        │
│   burst-2  173 ms p95 · 29% util · stable        │
│                                                 │
│  order-intake: 42 new orders Mon-Wed (3.7×)      │
│  staff KR-LSA: all green except owner yellow    │
│                                                 │
│  burst window: continues 96 hr                   │
│  daily reassessment: 04:00 KST                   │
│                                                 │
│  signed off by: Hae-Won Kim (logistics_coord)    │
│  observed by:   Lee Min-Jun (owner)              │
│                                                 │
│  ┌─────────────────────────────────────────┐   │
│  │  📋 CLOSE TODAY · HANDOFF TO TOMORROW   │   │
│  └─────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
```

## Locale + accessibility

- Hae-Won's locale: `ko-KR` primary; `en-US` secondary
- Hangul preservation: UTF-8 NFC throughout; never normalized; render with appropriate Korean font (Noto Sans KR)
- Hanja preservation: any traditional/Chinese characters in company names persist (e.g. 사장님 with proper rendering)
- Font: Noto Sans KR for Korean text + system default for English; supports family-name-first naming conventions natively
- Color contrast: WCAG AA in light mode + dark mode; cell utilization bars use color + texture for accessibility
- Touch targets: ≥44dp standard; tablet uses ≥48dp
- The dual-tenant pill always uses distinct visual treatment — italic for tentative/personal, bold for active/work

## Failure-mode UX

| Failure | UX response |
|---|---|
| Disclosure signal exceeds 1024 bytes | Send button greyed; live byte counter at limit shows red |
| Disclosure signal flagged for PII | Send blocked; user sees explicit reason + diff |
| Burst cell warm-start fails | Workflow halts; rollback proposed; alternate cell suggested |
| Latency exceeds rollback threshold | Auto-pause; banner; operator must resume or roll back |
| Employer accidentally tries to query personal | 403 with explicit boundary message; not a 500 |
| KR-LSA evaluator returns red | Burst staffing redistribution required before workflow advances |
| Hangul normalization detected | Hard error; write rejected; audit dual-sealed |

## Stop condition

The UX flow is correct when Hae-Won can complete the 4h24m journey on her phone + tablet in a busy mailroom, switch tenants explicitly without bleed, send a disclosure signal that respects the boundary, drive the cell-rebalance as a logistics coordinator, and see the dual-tenant boundary invariants validated visibly on the same console — all with Hangul + Hanja preserved at byte-level fidelity and the active-tenant pill never lying about which context is in scope.
