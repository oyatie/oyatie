---
doc_class: User-Journey-Story
journey_id: j153-devon-williams-hvac-side-business-tax-end-of-year
date: 2026-05-20
authority_tier: 2
status: draft
---

# j153 — Story: Devon Williams, kitchen table, Hayward CA, 19:42 PST

## Cast

| Role | Name | Device | Persona surface |
|---|---|---|---|
| Sole proprietor | Devon Williams | iPhone 16 Pro + iPad mini 7 (paired) | Devon-as-handyman-side-business |
| Tax preparer | Linda Cookson, EA | Cookson Tax & Accounting desktop | external (connect-bridged) |
| Repeat customer | Marisol Vargas (Zelle customer) | n/a | side-business customer, no oya account |
| Marketplace gig source | oya neighbor-marketplace 3 gigs | platform | platform surface |
| Devon's accountant-assistant agent | "Bookie" (oya-intelligence Schedule-C copilot) | iPad mini surface | intelligence-substrate ML model |
| Customer #11 | Alex Tran (Stripe Connect, recurring filter-change) | n/a | side-business customer |
| Customer #34 | Jacqui Pierre (Venmo Q2 install) | n/a | side-business customer |

## Context

- Date: Monday December 28, 2026, 19:42 PST
- Location: 14882 Catalpa Ave, Hayward CA 94545, Devon's rental kitchen
- Tax year: 2026
- Days until end-of-year: 4 (Dec 28 → Dec 31)
- Tenants in scope:
  - `bayshore_climate_systems` (W-2 employer) — Devon is `employee` here
  - `devon_williams_hvac_llc` (side business) — Devon is `sole_proprietor_admin` here

## Beat-by-beat

### 19:42 PST — Devon opens the iPhone

Devon's been putting this off since Thanksgiving. He opens oya on the iPhone. The role-projection layer (ADR-0317) shows him the **tenant-switcher pill** at the top: currently `Devon-at-work · Bayshore`. He taps it. A bottom-sheet appears:

- Bayshore Climate Systems (work)
- **Devon Williams HVAC LLC (side business)** ← he taps this
- Personal (Devon-as-handyman, B2C)

The switch costs a passkey step-up (any tenant-switch requires it, per ADR-0311). He Face-IDs. Step-up issued. Now the role-projection shows him the **side-business surface**: a finops-focused home dashboard with three tiles: **Income**, **Expenses**, **Year-End**.

Audit event: `EVT-J153-IDENTITY-TENANT-SWITCH-OK-001`.

### 19:43 PST — Year-End dashboard

He taps **Year-End**. The dashboard shows:

| Metric | Status |
|---|---|
| Total side-business gross 2026 (provisional) | $34,436.77 across 64 jobs |
| Receipts captured | 187 of 187 expected (auto-detected from gas-station + Home Depot + Snap-On Tools + AutoZone purchase intents) |
| Mileage logged | 4,217 mi across 73 trips |
| CA-CDTFA sales tax obligation (provisional) | $348.50 |
| 1099-K threshold crossed | ✓ Yes (threshold = $2,500 for tax year 2026) |
| Reconciliation status | **Not started** |
| Days until tax-year end | 4 |

He taps **Start year-end reconciliation**.

Audit event: `EVT-J153-FINOPS-YEAR-END-START-002`.

### 19:44 PST — Step 1: income reconciliation

The finops-portal walks Devon through three sub-flows.

**Sub-flow 1.1 — Stripe deposits**

The portal shows 47 deposits totaling $28,419.27. Each row has: date, customer name, job-tasks-id, amount, fee, net. Devon scrolls through. He approves all 47 in a batch by passkey-tapping the **Approve 47 deposits** button.

Audit: `EVT-J153-PAYMENTS-STRIPE-RECONCILE-003` with the batch hash.

**Sub-flow 1.2 — Venmo import**

Devon connects his Venmo via the Plaid-Exchange-style bridge under `payments`. He authorises with his Venmo password (Devon never enabled provider-credential BYOK for Venmo under ADR-0255 §D-4; the platform-default rail handles auth via OAuth handshake). The bridge fetches 14 transactions totalling $4,217.50 from Q1-Q2 2026 marked with the Venmo "Goods & Services" flag (these are business transactions per IRS rules).

The portal asks: "Are these 14 Venmo transactions side-business income or personal?" Devon taps **Side-business income — confirm**. Each transaction is auto-matched to a job-tasks-id where possible (12 match; 2 do not — the portal flags those for manual job-link).

Audit: `EVT-J153-PAYMENTS-VENMO-IMPORT-004`.

**Sub-flow 1.3 — Zelle (Marisol's repeat business)**

Zelle does not provide a Plaid-Exchange-style import. Devon imports manually:

- Mar 14, 2026 — Marisol Vargas — $600 (job J-2026-0314-MV)
- Jul 22, 2026 — Marisol Vargas — $600 (job J-2026-0722-MV)
- Oct 11, 2026 — Marisol Vargas — $600 (job J-2026-1011-MV)

Total: $1,800 across 3 jobs.

`finops-portal` cross-references Marisol's customer record with the `tasks` job log. Matches all three. No duplicate against Stripe or Venmo. Confidence: 1.0.

Audit: `EVT-J153-PAYMENTS-ZELLE-MANUAL-IMPORT-005`.

### 19:51 PST — Step 2: 1099-K threshold tracking

`finops-portal` computes total side-business gross across all rails:

- Stripe Connect: $28,419.27
- Venmo G&S: $4,217.50
- Zelle: $1,800.00
- **Total: $34,436.77**

1099-K threshold for tax year 2026: $2,500. Stripe will issue Devon a 1099-K because his Stripe gross alone is way above. Venmo's threshold tracking — Venmo legally issues a 1099-K too. Zelle does not (Zelle is bank-to-bank, no third-party-settlement-organization status).

The portal shows: "Expect 1099-K from Stripe and Venmo by Jan 31. Zelle is reported by you on Schedule C directly."

Audit: `EVT-J153-FINOPS-1099K-THRESHOLD-COMPUTED-006`.

### 19:53 PST — Step 3: expense categorization (187 receipts)

This is the big chunk. Devon taps **Categorize 187 receipts**. The Bookie copilot opens — an `intelligence`-substrate model fine-tuned on IRS Schedule-C. Bookie streams categorization suggestions:

| Schedule-C Line | Category | Count | Confidence |
|---|---|---|---|
| Line 8 — Advertising | "Yelp Business Premium", "oya marketplace listing fee" | 11 | 0.97 |
| Line 9 — Car and truck (other than mileage) | "Tolls", "Parking" — small | 8 | 0.91 |
| Line 10 — Commissions and fees | "oya marketplace 8% commission" (the 3 gigs) | 3 | 1.00 |
| Line 13 — Depreciation | "Snap-On Tools — manifold gauge set $1,247" | 1 | 0.88 |
| Line 22 — Supplies | "AutoZone refrigerant", "Home Depot HVAC tape", etc. | 89 | 0.92 |
| Line 23 — Taxes and licenses | "CA-CDTFA registration", "Hayward business license" | 4 | 0.99 |
| Line 27a — Other expenses | "Phone (business %)", "Cloud subscription business %" | 16 | 0.82 |
| Cost of Goods Sold (Part III) | "Installed parts — copper line set $480 (J-2026-0518)" | 55 | 0.90 |

Devon goes through the 25 receipts where confidence is <0.85. Each takes ~6 seconds. He stops at receipt #142 — a $84 Home Depot receipt. Bookie tagged it "Supplies" but Devon recognises this was a personal trip (he bought lumber for a bookshelf for his daughter Aaliyah's room). He recategorizes to **NOT BUSINESS — exclude**. Bookie thanks him and updates its confidence model.

Audit events: `EVT-J153-FINOPS-RECEIPT-CAT-NNN` (187 events, batched into 4 audit-chain seal calls for efficiency).

### 20:14 PST — Step 4: mileage log

Devon taps **Mileage log**. From `tasks`:

- 73 business trips
- Total business miles: 4,217
- Standard mileage deduction at 54.5¢/mi: $2,298.27
- (Alternative — actual expense method — would require Devon's vehicle's depreciation basis, which he doesn't have set up. Bookie suggests sticking with standard.)

Each trip in the log has a job-tasks-id, a start odometer reading (from the truck's MDM-managed odometer telematics), an end odometer reading, and a calculated mileage. Devon approves all 73 trips.

Audit: `EVT-J153-TASKS-MILEAGE-EXPORT-007`.

### 20:21 PST — Step 5: CA-CDTFA sales tax on installed parts

`finops-portal` flags: "California requires sales tax on tangible personal property you install in HVAC repairs. Hayward rate is 10.25% (Alameda County base + district). You collected sales tax on 17 jobs where parts > $0. Total collected: $348.50. You owe this to CDTFA by April 30, 2027 (annual filing)."

Devon taps **File with CDTFA (queue)**. The compliance overlay routes this to the CA-CDTFA pack. Devon's signature confirms he holds a seller's permit (`SR-FNH-12-1244419`). A draft CDTFA filing is created with a `due_at` of April 30, 2027. Audit: `EVT-J153-COMPLIANCE-CDTFA-DRAFT-008`.

### 20:34 PST — Step 6: Schedule-C compute

`finops-portal` computes Devon's draft Schedule-C 1040:

| Schedule-C Line | Description | Amount |
|---|---|---|
| Line 1 | Gross receipts/sales | $34,436.77 |
| Line 2 | Returns and allowances | $0 |
| Line 4 | Cost of goods sold (Part III) | $7,140.55 |
| Line 7 | Gross income (1 − 2 − 4) | $27,296.22 |
| Line 8 | Advertising | $612.18 |
| Line 9 | Car and truck (other than mileage; this is tolls/parking only) | $147.50 |
| Line 10 | Commissions and fees | $1,142.41 |
| Line 13 | Depreciation (Section 179 — Snap-On gauge set) | $1,247.00 |
| Line 22 | Supplies | $4,217.84 |
| Line 23 | Taxes and licenses | $1,089.50 |
| Line 27a | Other expenses (incl. phone/cloud business %) | $912.14 |
| Line 28 | Total expenses | $9,368.57 |
| Line 29 | Tentative profit (7 − 28) | $17,927.65 |
| Line 30 | Home office | $0 (Devon doesn't claim) |
| Line 31 | Net profit (taxable) | $17,927.65 |

(The mileage deduction of $2,298.27 goes on **Form 4562 / Schedule-C line 9 alternate path**, not subtracted from line 9 dollars above. The portal handles the form-level split correctly.)

Devon reviews. He taps **Confirm Schedule-C draft**. Audit: `EVT-J153-FINOPS-SCHED-C-DRAFT-CONFIRMED-009`.

### 20:51 PST — Step 7: workflow-studio nightly automation

Devon taps **Set up nightly reconciliation**. The workflow-studio opens with a pre-templated flow:

```
[trigger: nightly @ 23:30 PST]
  → [payments: fetch new transactions since last_run]
  → [finops-portal: categorize new transactions via Bookie]
    → if confidence < 0.85, queue for Devon-review
    → else auto-categorize
  → [tasks: fetch new mileage trips]
  → [finops-portal: update Schedule-C draft]
  → [notify Devon via push: "Nightly reconcile complete: $N new income, $M new expense"]
```

Devon tweaks: he changes the trigger time to 22:30 PST (he gets home from Bayshore around 19:00 and is usually winding down by 22:30). He saves the flow as `nightly-side-business-reconcile-v1`. The workflow-studio compiles to a workflow-engine DAG and schedules it.

Audit: `EVT-J153-WORKFLOW-STUDIO-NIGHTLY-PUBLISHED-010`.

### 21:14 PST — Step 8: cross-tenant strict-separation check

Devon is curious. He tries to import a Bayshore W-2 payroll detail into his side-business reconciliation (testing the boundary). He taps **Import other income**, picks `bayshore_climate_systems` as the source. Cedar **denies**:

```
deny: action=finops.import_transactions
  source_tenant_id=bayshore_climate_systems
  resource.tenant_id=devon_williams_hvac_llc
  reason=adr_0311_strict_separation
```

A red banner: "**Cross-tenant import refused**. Your W-2 from Bayshore is reported on your personal 1040 directly, not through Schedule C. Your tax preparer handles the combination."

Audit: `EVT-J153-CEDAR-DENY-W2-INTO-SCHED-C-011`.

(Devon nods. He wasn't going to actually file that way; he was testing the boundary because his cousin had a similar oya setup and got confused.)

### 21:42 PST — Step 9: tax-preparer share

Devon taps **Share with my tax preparer**. The connect surface asks:

- Recipient: pre-filled with **Cookson Tax & Accounting — Hayward, CA** (Devon set this up earlier in the year)
- Tax year: 2026
- Documents to share:
  - ✓ Schedule-C draft (JSON + PDF)
  - ✓ 1099-K summary (from Stripe + Venmo, when issued in January)
  - ✓ Mileage total ($2,298.27)
  - ✓ CDTFA filing draft (for Cookson to review)
  - ☐ Customer PII detail (off by default; tax preparer doesn't need names)
  - ☐ Bayshore W-2 (off; ADR-0311; Devon will share that to Cookson via Bayshore's own surface)
- Share-link TTL: until Jan 31, 2027 23:59 PST (default; Devon can extend)
- Watermark on PDFs: "Devon Williams HVAC LLC · Cookson Tax & Accounting · 2026 Tax Year · share-id sl-2026-12-28-cookson-7741"

Devon taps **Send via connect**. Cedar evaluates: `permit` because the preparer-ID matches the saved one and the scope fields don't include `customer_pii`. The share-link is minted; Linda Cookson receives an email with the link.

Audit: `EVT-J153-CONNECT-TAX-PREPARER-SHARE-MINT-012`.

### 22:07 PST — Linda Cookson clicks the link

Linda is still at her office (Cookson keeps late hours in December). She clicks the link. It loads in her browser. She sees:

- Schedule-C 1040 draft (PDF, watermarked)
- 1099-K summary placeholder (Stripe + Venmo will populate in January)
- Mileage total
- CDTFA filing draft

She downloads the PDFs. Each download is sealed: `EVT-J153-CONNECT-PREPARER-DOWNLOAD-NNN`. She replies to Devon via email — outside oya — confirming she'll incorporate this into his 1040 next week.

### 22:11 PST — Devon walks to the kitchen for water

He's relieved. He almost goes to bed but remembers one more thing.

### 22:14 PST — Step 10: community review push

`community` shows him: "You have 4 customer-review-pending nudges for completed Q4 jobs." He sends each customer a non-pushy review request via email (community handles the templating). 12 of his Q4 customers will see this; he expects 7-8 to leave reviews based on historical rate.

Audit: `EVT-J153-COMMUNITY-REVIEW-NUDGE-013`.

### 22:21 PST — Final state

Devon closes the iPhone. The side-business tenant's year-end reconciliation status is now `CONFIRMED-AWAITING-PREPARER-FILING`. The next checkpoint is when Linda Cookson sends back the prepared 1040 in late February for Devon's review.

Across the session: 13 audit-event classes emitted, 187 receipts processed, 64 income transactions reconciled, 0 cross-tenant data leaks, 1 Cedar deny (the W-2 import attempt — correctly refused).

### 01:14 PST (Dec 29) — Bookie's nightly run

Eleven minutes after Dec 28 turns to Dec 29 — wait, this is the next day. The `nightly-side-business-reconcile-v1` is scheduled for 22:30 PST nightly. Tonight is the first run.

It fires at 22:30 PST Dec 29. There are 2 new Stripe deposits since the last run (yesterday), 1 new mileage trip, 0 new receipts. Bookie categorizes the 2 deposits (both are repeat customers, confidence 0.99) and adds the mileage (3.2 mi). Schedule-C draft updated. Push notification to Devon:

> "Nightly reconcile complete: 2 new deposits ($172.50), 1 trip (3.2 mi, +$1.74 mileage deduction). No review needed."

Audit: `EVT-J153-WORKFLOW-NIGHTLY-RECONCILE-RUN-014`.

This is the steady state Devon wanted. No more frantic Decembers.

## What did NOT happen

- No Bayshore W-2 data ever crossed the tenant boundary into the LLC.
- No customer PII was included in the Cookson share.
- The CDTFA filing was queued, not auto-submitted (Devon must consciously file by April 30).
- The 1099-K from Stripe + Venmo did NOT exist yet (those will arrive in late January).
- Devon's SECA (self-employment tax) calculation was NOT performed by oya — Cookson does that.

## Audit-event chain (sealed, sequence)

| # | Event class | Timestamp |
|---|---|---|
| 001 | EVT-J153-IDENTITY-TENANT-SWITCH-OK | 19:42:18 |
| 002 | EVT-J153-FINOPS-YEAR-END-START | 19:43:08 |
| 003 | EVT-J153-PAYMENTS-STRIPE-RECONCILE | 19:48:11 |
| 004 | EVT-J153-PAYMENTS-VENMO-IMPORT | 19:49:42 |
| 005 | EVT-J153-PAYMENTS-ZELLE-MANUAL-IMPORT | 19:51:04 |
| 006 | EVT-J153-FINOPS-1099K-THRESHOLD-COMPUTED | 19:53:18 |
| 007a-007d | EVT-J153-FINOPS-RECEIPT-CAT-{batch_1..4} | 19:54 — 20:13 |
| 008 | EVT-J153-TASKS-MILEAGE-EXPORT | 20:14:51 |
| 009 | EVT-J153-COMPLIANCE-CDTFA-DRAFT | 20:21:33 |
| 010 | EVT-J153-FINOPS-SCHED-C-DRAFT-CONFIRMED | 20:34:18 |
| 011 | EVT-J153-WORKFLOW-STUDIO-NIGHTLY-PUBLISHED | 20:51:42 |
| 012 | EVT-J153-CEDAR-DENY-W2-INTO-SCHED-C | 21:14:18 |
| 013 | EVT-J153-CONNECT-TAX-PREPARER-SHARE-MINT | 21:42:11 |
| 014..014d | EVT-J153-CONNECT-PREPARER-DOWNLOAD-NNN | 22:07 — 22:09 |
| 015 | EVT-J153-COMMUNITY-REVIEW-NUDGE | 22:14:33 |
| 016 | EVT-J153-WORKFLOW-NIGHTLY-RECONCILE-RUN | (next day 22:30) |

All events seal under ADR-0263 emission contract. Every event carries `tenant_id = devon_williams_hvac_llc`, `journey_id = j153`.
