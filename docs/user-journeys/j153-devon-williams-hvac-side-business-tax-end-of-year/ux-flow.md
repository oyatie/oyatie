---
doc_class: User-Journey-UX-Flow
journey_id: j153-devon-williams-hvac-side-business-tax-end-of-year
date: 2026-05-20
authority_tier: 2
status: draft
---

# j153 — UX flow

## Devices

| Surface | Device | OS | Notes |
|---|---|---|---|
| Devon — primary | iPhone 16 Pro | iOS 19 | The tenant-switch happens here |
| Devon — receipt review | iPad mini 7 | iPadOS 19 (paired session, Universal Clipboard) | Better for the 187-receipt walk-through |
| Linda Cookson — preparer | desktop Mac | macOS 15 (Safari 19) | Receives the connect share-link |
| Tracking-truck — odometer ingest | dash-mounted MDM tablet | Android 14 + Bayshore-MDM | Pre-existing; supplies mileage data via tasks |

## Tenant-switch pill (top of every screen)

Persistent pill, top-center:

- Pill text: "Devon Williams HVAC LLC ▾"
- Color: green (side-business tenant)
- Tap → bottom-sheet with 3 options:
  1. Bayshore Climate Systems (work) — blue
  2. Devon Williams HVAC LLC (side business) — green (active)
  3. Personal (Devon-as-handyman, B2C) — gray
- Switch requires passkey step-up (Face ID); step-up is silent if recent (≤300s) and the pill just flips color

## Screen 1 — Side-business home (after switch)

- Hero card: "Side business at-a-glance"
- KPIs: gross-to-date, net-to-date, jobs YTD, mileage YTD
- Tiles: **Income · Expenses · Jobs · Year-End · Settings**
- Bottom toast: "Year-end deadline in 4 days. Tap Year-End to start."

## Screen 2 — Year-End dashboard

- Top: progress bar — 7 steps (0/7 complete)
- Step list:
  1. Reconcile income (Stripe + Venmo + Zelle)
  2. Track 1099-K threshold
  3. Categorize 187 receipts
  4. Mileage log
  5. CA-CDTFA sales tax
  6. Schedule-C compute
  7. Share with preparer
- Right rail: "Set up nightly automation (recommended)"

## Screen 3 — Stripe Connect deposits batch

- Header: "47 deposits, $28,419.27 gross"
- Sortable table: date · customer · job · gross · fee · net
- Bulk-select checkbox; select-all default
- Primary: "Approve 47 deposits" (passkey biometric on tap)

## Screen 4 — Venmo import wizard

- Step a: "Connect your Venmo account" — OAuth button
- Step b: Plaid-Exchange-style consent screen rendered inside the app
- Step c: "Review 14 transactions" — table; each row "Side-business? [Yes/No/Maybe]"
- Step d: "Match to jobs" — 12/14 auto-matched (green check); 2 manual (amber "Link to job"); tap to pick the job from the tasks job picker
- Primary on each step: "Continue"
- Tertiary on step d: "Skip the 2 unmatched — review later"

## Screen 5 — Zelle manual entry

- Form: date / customer / amount / job picker
- 3 entries pre-filled (Marisol Vargas × 3) since `community` recognised the recurring customer name
- Add row button (rare)

## Screen 6 — Receipt categorization (review queue, 25 receipts)

- Card-per-receipt:
  - Top: image of the receipt (OCR result)
  - Middle: vendor · date · amount
  - Bookie's suggested category (pill, amber if <0.85 confidence)
  - Pick override: dropdown with all Schedule-C lines plus EXCLUDE
- Swipe right = accept; swipe left = reject + manual pick
- Footer: "X of 25 reviewed"
- After last: "All 187 receipts categorized. Continue?" → tap Continue

## Screen 7 — Mileage approval

- Map view (Mapbox-style) with the 73 trips overlaid (color = season)
- Sortable list below: trip date · job · miles · deductible
- Total miles header: 4,217
- Footer pill: "$2,298.27 deductible (54.5¢/mi standard rate, IRS 2026)"
- Primary: "Approve all 73 trips"

## Screen 8 — CA-CDTFA flag

- Banner (amber): "California sales tax on installed parts"
- Body: explains the obligation; Hayward 10.25% rate; $348.50 collected on 17 jobs
- Field: seller's permit (pre-filled SR-FNH-12-1244419)
- Primary: "Queue CDTFA filing (due Apr 30, 2027)"
- Tertiary: "Defer for later"

## Screen 9 — Schedule-C compute (results)

- Top: "Your Schedule-C 2026 draft"
- Line-by-line table (lines 1, 4, 7, 8, 9, 10, 13, 22, 23, 27a, 28, 29, 31)
- Big green stat: "Net profit $17,927.65"
- Side panel: "Need help understanding?" → Bookie chat
- Primary: "Confirm draft" (passkey)
- Secondary: "Make adjustments"

## Screen 10 — workflow-studio nightly automation builder

- Canvas-style flow editor (drag-drop on desktop; tap-and-arrange on mobile)
- Pre-template loaded; nodes: trigger → fetch → categorize → mileage → compute → notify
- Trigger node opens a drawer:
  - Time picker: 22:30 PST (default 23:30, Devon changed it)
  - Frequency: nightly
- Each node shows estimated runtime
- Primary: "Save & publish"

## Screen 11 — Cross-tenant deny banner

When Devon taps "Import from Bayshore" (testing the boundary):

- Red full-width banner
- Title: "Cross-tenant import refused"
- Body: explains ADR-0311 in plain language ("Your W-2 from Bayshore goes on your personal 1040, not your Schedule C. Your tax preparer combines them.")
- Tertiary: "Why this rule exists" → opens an explainer drawer

## Screen 12 — Connect share with preparer

- Recipient card: "Cookson Tax & Accounting — Hayward, CA" (verified ✓)
- Scope checkboxes (mandatory on by default):
  - ✓ Schedule-C 2026 draft
  - ✓ 1099-K summary
  - ✓ Mileage total
  - ✓ CDTFA filing draft
- Scope checkboxes (off by default):
  - ☐ Customer PII details
  - ☐ Bayshore W-2 (locked off; tooltip: "ADR-0311 prevents cross-tenant share")
- TTL picker: default Jan 31 2027 23:59 PST
- Watermark preview
- Primary: "Send via connect" (passkey)

## Screen 13 — Linda's browser view (preparer side)

- Browser tab title: "Devon Williams HVAC LLC · 2026 Tax · Cookson Tax"
- Top: bold yellow strip: "Watermarked share · Linda Cookson · expires Jan 31"
- Four download tiles: Schedule-C PDF, 1099-K summary, mileage CSV, CDTFA draft
- Each download is a one-tap, audit-sealed event

## Screen 14 — Community review nudge composer

- Top: "12 Q4 customers haven't reviewed you yet"
- Body: pre-templated email (Devon can edit, but the template is short and tested)
- Channel: email (Devon doesn't have customer phone numbers for half)
- Throttle: max 1 nudge per customer per 30 days
- Primary: "Send 12 nudges"

## State transitions

| Trigger | From | To | Side-effect |
|---|---|---|---|
| Tenant pill tap → switch | (any) | side-business surface | passkey step-up |
| Year-End start | any | reconciliation in progress | finops-portal arms a year-end session |
| Each step approve | step N | step N+1 | audit-sealed |
| All steps complete | step 6 | step 7 | Schedule-C draft confirmed |
| Workflow-studio publish | n/a | scheduled flow | workflow-engine arms next run |
| Connect share mint | n/a | share-link active | preparer receives email |
| Share link first download | share-link active | share-link partially-consumed | each download audit-sealed |
| TTL passes | share-link active | share-link expired | preparer cannot access |

## Accessibility specifics

- High-contrast mode auto-engaged at dawn/dusk and when the device's ambient sensor reports <500 lux (Devon often works late)
- All Schedule-C line numbers spoken via VoiceOver as "Line eight, Advertising" (not just "8")
- The receipt-OCR confidence is announced via VoiceOver too
- Color blindness palettes: the income vs expense color distinction is reinforced with shape (income = circle, expense = square)
- Tap-target floor 48dp on iPhone (slightly under the gloved floor of j152 because Devon is not in field gloves at the kitchen table)
- Bookie's chat surface supports speech-to-text input

## Copy specifics — Schedule-C line picker

The line picker displays each line with its IRS canonical text + a plain-English rephrase:

| IRS line | IRS canonical text | Plain English |
|---|---|---|
| 8 | "Advertising" | "Things I paid to get customers" |
| 9 | "Car and truck expenses" | "Tolls, parking, anything other than mileage" |
| 10 | "Commissions and fees" | "Fees I paid (marketplace cut, lead-gen)" |
| 13 | "Depreciation and Section 179" | "Big tools and equipment I'm writing off" |
| 22 | "Supplies" | "Tape, fittings, refrigerant, small parts" |
| 23 | "Taxes and licenses" | "Business license, seller's permit, registrations" |
| 27a | "Other expenses" | "Phone, cloud, anything that doesn't fit above" |

## Anti-pattern guardrails

1. Never default to auto-categorize-and-skip-review for receipts below 0.85 confidence. The review queue is mandatory.
2. Never silently combine W-2 and Schedule-C income. Cedar blocks; UI also blocks at the picker level.
3. Never share customer PII with a tax preparer by default. The checkbox is off; toggling it on requires an additional confirmation modal.
4. Never auto-submit a CDTFA filing. The filing is queued; Devon must consciously file.
5. Never let Bookie hallucinate a category. If Bookie's confidence < 0.85, surface the receipt for human review; never auto-apply.
