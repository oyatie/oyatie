---
doc_class: User-Journey-UX-Flow
journey_id: j154-tomas-pieter-channel-partner-co-marketing-launch
date: 2026-05-20
authority_tier: 2
status: draft
---

# j154 — UX flow: Tomas's MacBook + Henrik's desktop + Esther's review

## Device + render targets

| Surface | Device | OS | Form factor | Constraints |
|---|---|---|---|---|
| Tomas — channel partner manager | MacBook Pro M4 14" + Magic Mouse + iPad Pro 11" sidecar | macOS 15.2 | desktop browser Chrome 145 | NL keyboard, Dutch IME, Anglo-EU date format |
| Henrik — Glacier marketing director | Dell XPS 15 9540 + dual 27" monitors | Windows 11 23H2 | desktop browser Edge 145 | DE keyboard, ISO 8601 + DIN-5008 conventions |
| Anneke — PartnerLift CMO | MacBook Air M3 | macOS 15 | desktop browser Safari 18 | low-spec network from her Amstelveen home office |
| Beate — Glacier CMO | iPad Pro 13" + Magic Keyboard | iPadOS 18 | tablet-as-desktop | uses Pencil for contract signing |
| Esther — external counsel | ThinkPad X1 Carbon Gen 12 | Ubuntu 24.04 + Firefox 134 | desktop browser | privacy-hardened browser; uKey4 hardware key |
| MarketSmith — campaign copilot | server | n/a | model surface | response budget 1.4s p95 |

## Locale + RTL

All Dutch text is rendered in `nl-NL` (default sentence case, no Title Case headlines). German text is `de-DE` with formal `Sie` form in B2B copy. English fallback is `en-EU` (date `DD/MM/YYYY`, time `09:00` 24h, euro symbol after amount with NBSP per EU style: `€ 180.000,00`).

The active-tenant pill is always present in the top-left of every screen. Color codes:

- `partnerlift_nl` — **navy** with PartnerLift wordmark
- `glacier_erp_de` — **slate** with Glacier glacier-blue accent
- `glacier-partnerlift-q1-2027-mfg-de-nl-be` — **co-branded gradient navy→slate** with a small "shared" lock glyph

Switching tenants requires a click + 2-second deliberate hold (anti-accidental-disclosure pattern per ADR-0311).

## Screen-by-screen progression — Tomas's MacBook

### Screen 1 — Tomas opens the channel-partner workspace (14:11 CET)

URL: `https://workspace.oya.network/t/partnerlift_nl/channel-partner/dashboard`

Layout:
- **Top bar** — active-tenant pill `PartnerLift B.V.`; right side: locale picker `NL`, notification bell (3 unread), profile menu
- **Left rail** — 6 icons: Dashboard / Campaigns / Partners / Leads / Community / Settings
- **Center**:
  - Card "Active campaigns (3)" — 3 tiles; the "Glacier Q1-2027 Manufacturing" tile is top, status pill amber: `Awaiting trinity provisioning`
  - Card "Pipeline" — €4.7M weighted, 22 deals
  - Card "Partners" — 14 active vendor relationships
- **Right rail** — Today's tasks, partner messages, alerts

Tomas clicks the Glacier Q1-2027 tile.

### Screen 2 — Trinity provisioning wizard, step 1/6 (14:13 CET)

Modal title: **Provision the shared co-marketing tenant**

Subtitle: *A shared tenant holds data both partners contribute. It has a defined lifecycle (today → 31 March wind-down → 1 July archive). Both partners are joint controllers under GDPR Art 26.*

Form:
- Proposed tenant ID: `glacier-partnerlift-q1-2027-mfg-de-nl-be` (editable; auto-generated from contract metadata)
- Data residency: dropdown — `eu-amsterdam-secondary` (selected) | `eu-frankfurt-primary` | `eu-paris-readonly-replica`
- Lifecycle preview (read-only):
  - Active: today → 31-03-2027 23:59 CET
  - Wind-down: 01-04-2027 → 30-06-2027 (read-only, write-refused)
  - Archive: from 01-07-2027 (cold storage, 7-year retention per PartnerLift policy)
- Pack overlays (pre-selected, gray): `eu-gdpr` `nl-telecom` `eu-dsa` `icc-marketing`

Primary button: **Send to Henrik for co-sign**
Secondary: **Save as draft**

Below the form, an info card: *Glacier ERP's marketing director Henrik Faulkner will receive a connect-bridged request to co-sign. Until they co-sign, the shared tenant does not exist.*

### Screen 3 — Pending co-sign view (14:13 — 14:26 CET)

After Tomas sends:

- Toast (top-right): *Provisioning request sent. Henrik has 24 hours to co-sign.*
- The tile in Screen 1 now shows status: `Awaiting co-sign · 23h 56m remaining`
- A live activity stream below the tile:
  - 14:13:18 — *You sent the provisioning request to Henrik Faulkner*
  - 14:26:04 — *Henrik Faulkner co-signed. Cedar bundle compiling…*
  - 14:26:11 — *Shared tenant active. Joint-controller roles granted.*

### Screen 4 — DPA upload (step 2/6, 14:30 CET)

Modal title: **Upload the tri-party GDPR DPA**

Subtitle: *Three signatures required: PartnerLift CMO + Glacier CMO + the external counsel acting for the shared tenant.*

Drop-zone (dashed border): *Drag the signed PDF here, or click to browse*

After Tomas drops `dpa-glacpl-mfg-q1-2027-signed.pdf`:

Status panel auto-populates:

| Signatory | Role | Verification |
|---|---|---|
| Anneke van der Meer | PartnerLift CMO | ✓ Adobe AATL-EU 2024 |
| Beate Hoffmann | Glacier CMO | ✓ Adobe AATL-EU 2024 |
| Esther Bakker | External counsel (shared) | ✓ eIDAS QES (NL KPN 2024) |

Below: *Storing in all three tenants' audit trails per GDPR Art 30…*

Primary button: **Continue to sender-domain configuration**

### Screen 5 — Sender-domain configuration (step 3/6, 14:42 CET)

Title: **Configure sender domains**

A table with three rows:

| Domain | DKIM | SPF | DMARC | Reputation budget |
|---|---|---|---|---|
| `mfg.glacier-erp.de` | ✓ aligned | ✓ pass | ✓ p=reject | 15,000/day |
| `mfg.partnerlift.nl` | ✓ aligned | ✓ pass | ✓ p=quarantine | 8,000/day |
| `joint.glacier-partnerlift.eu` | ⚠ pending DNS publish | n/a | n/a | n/a |

The joint domain row expands on click. The expand panel shows:

- DNS records required (3 TXT records — DKIM/SPF/DMARC)
- A "Copy all records" button
- A "Send to PartnerLift IT" button (auto-emails Esmé van Wijk)

After Esmé publishes (6 minutes later), the row auto-refreshes:

| `joint.glacier-partnerlift.eu` | ✓ aligned | ✓ pass | ✓ p=quarantine | 5,000/day (linear warm-up) |

Primary button: **Continue to content authoring**

### Screen 6 — Content authoring (step 4/6, 14:55 CET)

URL: `https://workspace.oya.network/t/glacier-partnerlift-q1-2027-mfg-de-nl-be/marketing/campaign/camp-glacpl-mfg-q1-2027`

Active-tenant pill switches to co-branded gradient. Tomas confirms the switch (2-second hold).

Layout:

- **Left rail** — campaign tree: Sequences (2) / LinkedIn (3) / Display (2) / Landing pages (4)
- **Center** — selected sequence editor: "Email sequence B — Dutch — 5 emails"
- **Right rail** — MarketSmith copilot panel (collapsible)

Email editor for step 1:

- Subject line: editable, with auto-Dutch suggestion from MarketSmith
- Original suggestion (in a gray box at top): *"Een betere ERP — gegarandeerd"* with a "tone score" badge: `78/100 corporate-confidence-NL`
- Tomas overrides to: *"Glacier ERP — beproefd in Duitse mfg"*
- Subject preview (mock inbox): `09:23 · mfg@partnerlift.nl · Glacier ERP — beproefd in Duitse mfg · Mid-market mfg ERP. Live demo binnen…`

Body editor:

- WYSIWYG with `nl-NL` spell-check
- Side panel "MarketSmith review" highlights:
  - 1 amber: *Sentence "ROI binnen 14 maanden gegarandeerd" — guarantee claim may need legal review under NL Misleidende Reclame*
  - 1 green: *Reading level: Flesch-NL 52 (matches B2B procurement audience)*

Footer compliance bar (sticky, gray):
- *GDPR Art 6 basis*: `legitimate_interests_b2b` | `consent_for_natural_persons`
- *Unsubscribe link*: ✓ present
- *NL Telecom §11.7*: ✓ double-opt-in confirmed
- *DSA*: ✓ logged

Primary button (top-right): **Save draft + request Henrik's review**

### Screen 7 — German content panel (Henrik's perspective)

Same screen as Screen 6 but Henrik's view of sequence A (German). The right-rail MarketSmith renders in `de-DE` with formal `Sie` form.

Henrik's edits:
- Step 1 subject: *"Vergleichen Sie 3 Mid-Market-ERPs — 7-Tage-Demo"*
- Step 4 body: he replaces a vague phrase with *"Pilotbetrieb in 60 Tagen, vertraglich garantiert"*

After Henrik approves all 5 German emails, Tomas's screen 6 shows the sequence A entries marked **✓ co-approved by Glacier**.

### Screen 8 — Landing-page GDPR consent banner (step 5a, 15:38 CET)

Tomas reviews `joint.glacier-partnerlift.eu/mfg/nl` in a preview panel.

Below the visible page mock, a per-purpose consent banner overlay:

```
+--------------------------------------------------------------+
| Cookies & data — uw toestemming                              |
|                                                              |
|  We gebruiken cookies en verwerken data voor:               |
|                                                              |
|  ☑ Strikt noodzakelijk (kan niet worden uitgeschakeld)      |
|  ☐ Analytics (sessieduur, paginastromen)                    |
|  ☐ Advertising (LinkedIn, Google Display)                   |
|  ☐ Personalisatie (op uw bedrijfsgrootte aangepaste demo)   |
|                                                              |
|  [Alle weigeren]  [Aangepast opslaan]  [Alle accepteren]    |
|                                                              |
|  Verantwoordelijken: PartnerLift B.V. + Glacier ERP GmbH    |
|  (gezamenlijke controllers, Art 26 AVG).                    |
|  Lees ons privacybeleid: privacy@joint.glacier-partnerlift  |
+--------------------------------------------------------------+
```

Below the cookie banner, a separate banner appears for email-marketing consent (NL Telecom §11.7 double-opt-in):

```
+--------------------------------------------------------------+
|  Wilt u onze ERP-nieuwsbrief en demo-uitnodigingen?         |
|                                                              |
|  E-mailadres: [_______________________]                     |
|  [✓] Ja, stuur me een bevestigingsmail om mijn keuze        |
|      vast te leggen (dubbele opt-in, AVG-conform)           |
|                                                              |
|  [Inschrijven]                                              |
+--------------------------------------------------------------+
```

Tomas verifies the same structure for `/mfg/de`, `/mfg/be-fr`, `/mfg/be-nl`. The Walloon page uses `fr-BE` copy; the Flemish page uses `nl-BE` (different from `nl-NL`).

### Screen 9 — Lead-routing rules editor (step 5b, 16:02 CET)

Title: **Lead-routing rules — attribution 60/40**

A rule-table editor. Each row shows source → routes → attribution.

| Source | Routes to | Source % | Partner % |
|---|---|---|---|
| Glacier email form-fill | HubSpot + Salesforce | 60% Glacier | 40% PartnerLift |
| PartnerLift email form-fill | HubSpot + Salesforce | 60% PartnerLift | 40% Glacier |
| Joint LP form-fill | HubSpot + Salesforce | 50% PartnerLift | 50% Glacier |
| LinkedIn lead-gen (Glacier-funded) | HubSpot + Salesforce | 60% Glacier | 40% PartnerLift |

Each row has a "Dry-run with sample lead" button. Tomas runs 5 dry-runs:

```
Dry-run 1 — sample.acme@stalengieterij-utrecht.nl
  Source: mfg.partnerlift.nl.email_form_submit
  Route 1: HubSpot https://api.hubapi.com/.../contacts upsert: 201
  Route 2: Salesforce .../sobjects/Lead upsert: 201
  Attribution: PartnerLift 60%, Glacier 40%
  Both target CRMs received the lead with the same co_marketing_attribution object: ✓
```

All 5 dry-runs pass. Primary button: **Activate routing rules**

### Screen 10 — Cross-tenant denial (16:31 CET)

Tomas (curious) opens the CRM left-rail and clicks "Glacier internal CRM" (an option that should not exist for his role but is visible because of the trinity context).

Red full-screen modal:

```
+----------------------------------------------------------+
|  Access denied                                           |
|                                                          |
|  You can read leads inside the shared tenant.            |
|  You cannot read Glacier's internal CRM.                 |
|                                                          |
|  This boundary is set by ADR-0311 (dual-tenant identity) |
|  and the tri-party DPA between PartnerLift, Glacier,     |
|  and the shared tenant.                                  |
|                                                          |
|  Audit-event: EVT-J154-CEDAR-DENY-CROSS-PARTNER-CRM-READ |
|                                                          |
|  [Open shared lead pool]   [Why this boundary exists]   |
+----------------------------------------------------------+
```

The "Why this boundary exists" link opens a plain-language explainer:

*The shared tenant is the only place where data from both sides commingles, and even there only the contracted scope. Reading Glacier's internal CRM would mean reading their private business data — which the DPA, GDPR Art 26, and the partnership contract all forbid. If you need data on a specific lead, ask Henrik to share it in the shared tenant.*

### Screen 11 — Community channel creation (16:45 CET)

URL: `https://workspace.oya.network/t/glacier-partnerlift-q1-2027-mfg-de-nl-be/community/new-channel`

Channel form:
- Name: `mfg-q1-2027-glacier-partnerlift-coord`
- Visibility: **Private — partner-only**
- E2EE: **MLS RFC 9420** (mandatory, not toggleable)
- Data residency: `eu-amsterdam-secondary` (locked to shared tenant)
- Retention: 365 days default

Member picker shows two columns:
- PartnerLift roster (filterable) — Tomas selects 8 names
- Glacier roster (filterable) — Tomas selects 6 names

A small badge next to two members: **DPO observer** (Lara, Stefan) — they are added in read-only mode so they can audit but not post.

Primary button: **Create channel**

After creation, the channel opens. The first system message reads:

*Welcome to mfg-q1-2027-glacier-partnerlift-coord. This is a partner-only channel inside the shared tenant. End-to-end encryption is on. Lara de Wit and Dr. Stefan Köhler are DPO observers. All messages here seal in audit-chain across all three tenants. Be excellent.*

### Screen 12 — Pre-launch checklist (Jan 9 16:00 CET)

URL: `https://workspace.oya.network/t/glacier-partnerlift-q1-2027-mfg-de-nl-be/campaign/camp-glacpl-mfg-q1-2027/prelaunch`

Two-column layout. Left column is the campaign tree from Screen 6. Right column is the checklist:

| Check | Status | Owner |
|---|---|---|
| Sender domains: 3/3 DKIM aligned | ✓ | Tomas |
| GDPR landing pages: 4/4 consent banner verified | ✓ | Tomas |
| Per-country email reputation: ≥ "good" | ✓ | Comms-email auto |
| DPA on file: 3/3 tenants | ✓ | auto |
| Cedar policy bundle: deployed + validated | ✓ | Auto |
| CRM routing rules: 5/5 dry-runs passed | ✓ | Tomas |
| LinkedIn Ads: paused-ready | ✓ | Henrik |
| Google Display: paused-ready | ✓ | Henrik |
| Escrow: €180,000 confirmed | ✓ | Payments auto |
| Bounce/complaint thresholds: alerts wired | ✓ | Auto |
| DSA transparency log: wired | ✓ | Auto |

Below the table, a green pill: **All checks passed — ready for Jan 12 09:00 CET launch**

Primary button (disabled until T-30min before launch): **Arm campaign for scheduled launch**

### Screen 13 — Launch (Jan 12 09:00 CET)

T-3min, Tomas opens the campaign console on his iPad Pro (he's on the train to Utrecht for a partner visit).

Layout (iPad portrait):

- Top: Active-tenant pill (co-branded), countdown `T-02:47`
- Center: A big circular "LAUNCH" button, red gradient
- Below: a 3-line summary —
  - *Email Sequence A (DE): 8,000 emails ready*
  - *Email Sequence B (NL): 5,000 emails ready*
  - *LinkedIn + Display: armed*

At T-00, the button activates. Tomas taps **LAUNCH**. A confirmation modal:

```
+----------------------------------------------------------+
|  Launch campaign Glacier Q1-2027?                        |
|                                                          |
|  This will:                                              |
|   - Dispatch 8,000 emails from mfg.glacier-erp.de        |
|   - Dispatch 5,000 emails from mfg.partnerlift.nl        |
|   - Activate LinkedIn campaigns (3)                      |
|   - Activate Google Display campaigns (2)                |
|                                                          |
|  Approved by:                                            |
|   ✓ Anneke van der Meer (PartnerLift CMO)               |
|   ✓ Beate Hoffmann (Glacier CMO)                        |
|                                                          |
|  [Cancel]   [Launch now]                                |
+----------------------------------------------------------+
```

Tomas taps **Launch now**. Toast: *Campaign launched. Audit event sealed in all 3 tenants.*

### Screen 14 — 4-hour-in metrics dashboard (Jan 12 13:00 CET)

URL: `https://workspace.oya.network/t/glacier-partnerlift-q1-2027-mfg-de-nl-be/analytics/camp-glacpl-mfg-q1-2027`

Two columns side-by-side:

**Glacier (DE) panel** (Henrik's branded color):
- Sent: 8,000 | Delivered: 7,841 (98.0%)
- Opens: 3,762 (48%) | Clicks: 412 (5.2%)
- Form submits: 89 | Consent rate: 78%
- Complaint rate: 0.02% (well below 0.10% threshold)

**PartnerLift (NL) panel** (Tomas's branded color):
- Sent: 5,000 | Delivered: 4,927 (98.5%)
- Opens: 2,217 (45%) | Clicks: 251 (5.1%)
- Form submits: 51 | Consent rate: 84%
- Complaint rate: 0.04%

Below: a "Joint analytics" tab (read-only by either side; each side sees their own attribution, not the other's gross revenue):

- Leads created: 140 total → HubSpot 140, Salesforce 140 (dual-routed)
- Attribution snapshot: Glacier-sourced 89 + PartnerLift-sourced 51

### Screen 15 — Spam-trap hit (Jan 13 09:14 CET)

A red banner appears in the campaign console:

```
⚠ Spam-trap detected
Email address `procurement.team@spamhaus-trap-001-de` is a known spam-trap.
Address auto-suppressed from sequence A.
Henrik, please review your source list and remove any related addresses.
```

A "Review source list" button opens a filter view showing 48 addresses from the same source pool. Henrik clicks each, removes 47, and confirms. Toast: *48 addresses suppressed; reputation budget intact.*

### Screen 16 — Q1 settlement view (Mar 31 23:55 CET → Apr 1 00:01 CET)

URL: `https://workspace.oya.network/t/glacier-partnerlift-q1-2027-mfg-de-nl-be/escrow/esc-glacpl-mfg-q1-2027`

A timer counts down to settlement. At T+0, the panel refreshes:

```
Q1 settlement complete.
- Joint pool: 184 converted leads (50/50 split)
- Glacier-sourced converted: 482 (PartnerLift earns 40% credit)
- PartnerLift-sourced converted: 311 (Glacier earns 40% credit)
- Total Q1 revenue attributed: €4,232,118 ARR

Disbursement (SEPA initiations):
- Glacier ERP GmbH receives € 60,418.00
- PartnerLift B.V. receives € 119,582.00

Audit-event: EVT-J154-PAYMENTS-ATTRIBUTION-SETTLEMENT-014
Sealed in all three tenants.
```

Tomas closes the laptop. The campaign workspace status flips to `Q1 settled — wind-down armed for Apr 1 00:00 CET`.

## Critical state transitions

| Trigger | From state | To state | Side-effect |
|---|---|---|---|
| Tomas sends provisioning request | UNPROVISIONED | PENDING-CO-SIGN | connect notify Henrik |
| Henrik co-signs | PENDING-CO-SIGN | PROVISIONED | Cedar bundle compiled |
| DPA verified | PROVISIONED | DPA-ON-FILE | seal in 3 tenants |
| Sender domains aligned | DPA-ON-FILE | DELIVERABILITY-READY | reputation budgets active |
| Content approved (both sides) | DELIVERABILITY-READY | CONTENT-APPROVED | sequences in `ready` |
| Lead-routing dry-runs pass | CONTENT-APPROVED | ROUTING-READY | rules `active` |
| Community channel created | ROUTING-READY | COORD-CHANNEL-ACTIVE | MLS group provisioned |
| Pre-launch checklist passes | COORD-CHANNEL-ACTIVE | LAUNCH-ARMED | launch button enabled |
| Tomas taps launch | LAUNCH-ARMED | CAMPAIGN-LIVE | first emails dispatched |
| Q1 timer fires | CAMPAIGN-LIVE | SETTLING | payments computes |
| Settlement complete | SETTLING | SETTLED | SEPA initiated |
| Apr 1 00:00 CET | SETTLED | WINDING-DOWN | writes refused |
| Jul 1 00:00 CET | WINDING-DOWN | ARCHIVED | cold storage |

## Accessibility specifics

- **Right-to-left support**: not used in this journey (no Arabic / Hebrew prospects), but the layout engine supports RTL switching if a future Q-pack adds MENA targeting.
- **Color-blind palette**: the active-tenant pills use shape + label, not color alone. The gradient pill (shared) has a distinct lock glyph.
- **Keyboard-only navigation**: every action reachable via Tab/Shift+Tab + Enter. Launch button has a 2-second hold equivalent: Tab to focus → press-and-hold Enter for 2 seconds.
- **Low-bandwidth mode**: the campaign metrics dashboard collapses graphics into compact tables (Anneke's home connection is variable).
- **Translation badges**: any auto-translated content displays a "translated — review" amber badge until a native speaker confirms.

## Copy review

Every customer-facing string (subject lines, CTA buttons, landing-page copy, consent banners, denial copy) is reviewed by **two native speakers per locale**:

- Dutch: Tomas (NL native) + Roos van Veen (Belgian NL fluent)
- German: Henrik (DE native) + Klaus Lehmann (Austrian DE fluent — controls for Swiss/Austrian variants)
- French (Walloon): contracted reviewer Jean-Luc Wéry (Liège)
- Flemish (BE-NL): Roos as above + a Brussels contractor

The review log is stored in `drive` under `journey-j154/copy-review-log.yaml`.

## Anti-pattern guardrails

1. Never auto-launch a campaign without both CMOs in the approval chain. Cedar enforces this; UI hides the "Launch" button until both signatures land.
2. Never display Glacier's internal CRM rows to a PartnerLift principal, even with a stale Cedar cache. The trinity forbid evaluates server-side every request.
3. Never silently extend the shared-tenant lifecycle past Jul 1 archive without explicit contract amendment. Wind-down → archive is hard-coded into the lifecycle state machine.
4. Never propagate consent across tenants without explicit per-purpose mapping. The consent-graph µservice rejects copy operations that lack a mapping.
5. Never bury the "Why this boundary exists" link on denial screens. Plain-language explanation + audit-event reference + recovery action are required by the denial-UX contract.
6. Never default to English on consent banners shown in NL/BE/DE. Locale is set per landing-page URL; auto-detect is only a fallback.
7. Never expose the joint analytics dashboard to a non-controller. The Cedar trinity gate evaluates on every analytics page load.
