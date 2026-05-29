---
doc_class: User-Journey-Story
journey_id: j154-tomas-pieter-channel-partner-co-marketing-launch
date: 2026-05-20
authority_tier: 2
status: draft
---

# j154 — Story: Tomas Pieter, WeWork Weesperstraat, 14:11 CET

## Cast

| Role | Name | Tenant | Device |
|---|---|---|---|
| Channel Partner Manager | Tomas Pieter | partnerlift_nl | MacBook Pro M4 + iPad Pro |
| PartnerLift CMO | Anneke van der Meer | partnerlift_nl | desktop |
| PartnerLift DPO | Lara de Wit | partnerlift_nl | desktop |
| Glacier ERP Marketing Director | Henrik Faulkner | glacier_erp_de | desktop |
| Glacier ERP CMO | Beate Hoffmann | glacier_erp_de | desktop |
| Glacier ERP DPO | Dr. Stefan Köhler | glacier_erp_de | desktop |
| Tri-party compliance reviewer | Esther Bakker (external counsel) | external (connect-bridged) | desktop |
| Bookie-Marketing copilot | "MarketSmith" (oya-intelligence campaign-optimizer) | shared tenant | model |

## Context

- Date: Tuesday December 30, 2026, 14:11 CET
- Location: PartnerLift's hot-desk pod, WeWork Weesperstraat, Amsterdam
- Campaign launch target: Monday January 12, 2027, 09:00 CET
- Budget: €180,000 (€90K Glacier + €90K PartnerLift) escrowed in `payments`
- Tenants: `partnerlift_nl`, `glacier_erp_de`, and (about to provision) `glacier-partnerlift-q1-2027-mfg-de-nl-be`
- Pack overlays: EU-GDPR, NL-Telecom, EU-DSA, ICC-Marketing

## Beat-by-beat

### 14:11 CET — Tomas opens the campaign workspace

Tomas opens the partner workspace. The tenant-switcher pill shows `PartnerLift B.V.` He stays here. The home shows three pinned campaigns; the new "Glacier Q1-2027 Manufacturing" entry is at top with a status `Awaiting trinity provisioning`. He taps it.

### 14:13 CET — Trinity provisioning wizard

Step 1 of 6 — **Provision shared tenant**. A wizard explains:

> A shared tenant is a temporary, contract-scoped tenant that holds data both partners contribute. It has a defined lifecycle (today → Mar 31 wind-down → Jun 30 archive). Both partners are joint controllers under GDPR Art 26.

Tomas confirms the proposed tenant ID: `glacier-partnerlift-q1-2027-mfg-de-nl-be`. He picks data residency: **eu-amsterdam-secondary** (default; both parties agreed in the contract). The wizard validates with Glacier's side — Henrik must co-sign. A request is sent to Henrik's inbox via the `connector` bridge.

Audit: `EVT-J154-TENANCY-SHARED-PROVISION-REQUEST-001`.

### 14:26 CET — Henrik co-signs

Henrik (Frankfurt) co-signs the provisioning. The shared tenant goes from `pending_co_sign` → `provisioned`. Both partners receive the tenant_id and the joint-controller roles. Cedar policy bundle compiled. Audit: `EVT-J154-TENANCY-SHARED-PROVISIONED-002`.

### 14:30 CET — DPA upload + signature collection

Step 2 — **Upload tri-party GDPR DPA**. Tomas drags the signed PDF into the wizard. The PDF was negotiated by both legal teams in December; it bears three signatures: Anneke (PartnerLift), Beate (Glacier), and Esther (external counsel acting for the shared tenant). The `connector` µservice verifies each signature against the signatory's passkey-bound identity.

- Anneke: verified ✓
- Beate: verified ✓
- Esther: verified ✓ (via her LawFirm external DocuSign-class identity, federated through connect)

The DPA is stored in all three tenants' audit trails — replicated, not shared by reference, because each tenant must hold its own copy under GDPR Art 30 record-keeping obligations.

Audit: `EVT-J154-CONNECT-DPA-VERIFIED-003`.

### 14:42 CET — Sender domain configuration

Step 3 — **Configure sender domains**. The campaign uses three sender domains:

1. `mfg.glacier-erp.de` — Glacier's marketing sub-domain, German prospects
2. `mfg.partnerlift.nl` — PartnerLift's marketing sub-domain, Dutch prospects
3. `joint.glacier-partnerlift.eu` — co-branded sender, used sparingly for cross-border prospects (Belgium)

Tomas pulls up the DKIM/SPF/DMARC dashboard. Each domain shows its current alignment:

| Domain | DKIM | SPF | DMARC | Reputation budget (sends/day) |
|---|---|---|---|---|
| mfg.glacier-erp.de | ✓ aligned | ✓ pass | ✓ p=reject | 15,000 |
| mfg.partnerlift.nl | ✓ aligned | ✓ pass | ✓ p=quarantine | 8,000 |
| joint.glacier-partnerlift.eu | ⚠ pending DNS publish | n/a | n/a | n/a |

The joint domain is pending DNS publish. Tomas copies the required DKIM TXT record and forwards it to PartnerLift's IT (Esmé van Wijk handles DNS). Esmé publishes it within 6 minutes. By 14:51 CET, the joint domain shows ✓ aligned. Audit: `EVT-J154-COMMS-EMAIL-DKIM-VERIFY-004`.

### 14:55 CET — Bilingual content authoring

Step 4 — **Author the email + LinkedIn + Display content**. Tomas opens the marketing-automation surface. MarketSmith (the copilot) suggests:

- Email Sequence A — German prospects — 5 emails (D+0, D+3, D+7, D+14, D+28). Sender: `mfg.glacier-erp.de`. CTA: "Demo planen" (Schedule a demo). All five drafts pre-translated to German by MarketSmith with native-speaker review chips for Henrik to approve.
- Email Sequence B — Dutch prospects — 5 emails. Sender: `mfg.partnerlift.nl`. CTA: "Demo plannen". Tomas reviews each line. He overrides email 3's subject line: MarketSmith proposed "Een betere ERP — gegarandeerd"; Tomas changes to "Glacier ERP — beproefd in Duitse mfg" (more credible for the Dutch market).
- LinkedIn carousel — 4 frames each in NL + DE
- Display — 3 banner variants per locale, GIPC-compliant (no medical imagery, no political imagery, no comparative-claims imagery)

Audit per asset: `EVT-J154-MARKETING-ASSET-AUTHOR-NNN` (29 events total: 10 emails + 8 LinkedIn frames + 6 display banners + 5 landing-page variants).

### 15:38 CET — GDPR landing-page consent gates

Step 5a — **Landing-page GDPR consent**. The campaign drives prospects to 4 landing pages:

- `joint.glacier-partnerlift.eu/mfg/de` — German prospects
- `joint.glacier-partnerlift.eu/mfg/nl` — Dutch prospects
- `joint.glacier-partnerlift.eu/mfg/be-fr` — Walloon prospects
- `joint.glacier-partnerlift.eu/mfg/be-nl` — Flemish prospects

Each landing page has the **ADR-0272 per-purpose cookie consent** banner: 4 purposes (Strictly necessary / Analytics / Advertising / Personalisation). Each is opt-in (no pre-checked boxes; valid GDPR consent). The default state is "all opt-out except strictly necessary". A separate banner asks for email-marketing consent under NL-Telecom §11.7 for the Dutch page (double-opt-in required: a confirmation email is sent after the first form submit).

Audit: `EVT-J154-COMPLIANCE-GDPR-LANDING-PAGE-CONSENT-005`.

### 16:02 CET — Lead-routing rules

Step 5b — **Lead-routing rules**. The CRM rules are set per the contract's attribution rule:

- A lead generated from a `mfg.glacier-erp.de` email → 60% Glacier (source), 40% PartnerLift (partner)
- A lead generated from a `mfg.partnerlift.nl` email → 60% PartnerLift, 40% Glacier
- A lead generated from a `joint.glacier-partnerlift.eu` LP form-fill → 50/50 (split-credit)
- A lead generated from LinkedIn Lead Gen Form (Glacier-funded ad) → 60% Glacier, 40% PartnerLift

The routing flows to **both** HubSpot (PartnerLift's CRM) and Salesforce (Glacier's CRM). The CRM µservice acts as the broker. Each lead carries a `co_marketing_attribution` object with the percentages. Audit: `EVT-J154-CRM-ROUTING-RULES-CONFIGURED-006`.

### 16:31 CET — Cross-tenant CRM read attempt — DENIED

Tomas is curious. He tries — via the CRM surface — to read Glacier's full HubSpot-equivalent (Salesforce) internal lead list (not the shared pool). Cedar evaluates:

```
principal = User::"tomas.pieter@partnerlift.nl"
action = crm.read
resource.tenant_id = "glacier_erp_de"
principal.role_in_tenant("glacier_erp_de") = "joint_controller_partnerlift" (NOT in [marketing_director, sales_director, system_admin])
result: deny
```

A red banner explains: "You can read leads inside the shared tenant. You cannot read Glacier's internal CRM. This boundary is set by ADR-0311 + the tri-party DPA."

Audit: `EVT-J154-CEDAR-DENY-CROSS-PARTNER-CRM-READ-007`.

### 16:45 CET — Community channel setup

Step 5c — **Slack-Connect-class private channel**. Tomas opens `community`. He creates the channel `#mfg-q1-2027-glacier-partnerlift-coord` inside the shared tenant. He invites:

- PartnerLift side (8): Tomas, Anneke (CMO), Mira (paid-media), Joost (content), Roos (events), Bram (ops), Lara (DPO observer), Hendrik (analytics)
- Glacier side (6): Henrik, Beate (CMO), Pia (paid-media), Klaus (content), Stefan (DPO observer), Frieda (analytics)

Total: 14 members. The channel data lives in the shared tenant; data residency = eu-amsterdam-secondary. End-to-end MLS encryption (per the MLS-RFC-9420 doctrine — keystone bundle item, KS#5).

Audit: `EVT-J154-COMMUNITY-CHANNEL-CREATE-008`.

### 17:11 CET — Tomas wraps Day 1

Tomas closes the laptop. The campaign workspace status is `Configured — Awaiting QA`. Henrik will run QA on the German content tomorrow. Tomas and Henrik will dry-run the campaign Friday Jan 9. Launch is Monday Jan 12 09:00 CET.

## Jan 5 — A/B test cell adjustments

(Five days later.) MarketSmith reports: the test-cell dry-run on 200 internal employee inboxes (50/50 split between subject-line A and B for the Dutch sequence) showed:

- A: "Glacier ERP — beproefd in Duitse mfg" — open rate 47%
- B: "ERP voor mfg: vergeleken met de top 3" — open rate 39%

A wins. The Dutch campaign uses A. The German campaign keeps Henrik's preferred subject line.

Audit: `EVT-J154-MARKETING-AB-TEST-RESOLVED-009`.

## Jan 9 — Pre-launch checklist

Friday Jan 9, 16:00 CET. Tomas and Henrik run the pre-launch checklist together over the community channel + a video call:

| Check | Status |
|---|---|
| Sender domains: 3/3 DKIM aligned | ✓ |
| GDPR landing pages: 4/4 consent banner verified | ✓ |
| Per-country email reputation: ≥ "good" for all three sender domains | ✓ |
| DPA on file: 3/3 tenants | ✓ |
| Cedar policy bundle: deployed and validated | ✓ |
| CRM routing rules: verified with 5 dry-run leads (each routed correctly to both CRMs with attribution) | ✓ |
| LinkedIn Ads campaign: paused-ready, set to launch at Jan 12 09:00 CET | ✓ |
| Google Display campaign: paused-ready | ✓ |
| Escrow: €180,000 confirmed; release schedule = post-Q1 attribution settlement | ✓ |
| Bounce/complaint thresholds: alerts wired to community channel | ✓ |
| DSA transparency log: wired to audit-chain | ✓ |

Audit: `EVT-J154-CAMPAIGN-PRELAUNCH-CHECKLIST-COMPLETE-010`.

## Jan 12 09:00 CET — LAUNCH

Tomas taps **Launch campaign**. The marketing-automation engine flips the campaign from `paused` → `live`:

- Email Sequence A (German) — first 8,000 emails fan out from `mfg.glacier-erp.de` at the country-optimised send time (09:00 CET for Germany)
- Email Sequence B (Dutch) — first 5,000 emails fan out from `mfg.partnerlift.nl`
- LinkedIn Ads campaign begins bidding
- Google Display campaign begins bidding

Audit: `EVT-J154-CAMPAIGN-LAUNCH-011`.

## Jan 12 13:00 CET — 4 hours in

Metrics from the analytics projection:

| Metric | Glacier (DE) | PartnerLift (NL) |
|---|---|---|
| Sent | 8,000 | 5,000 |
| Delivered | 7,841 (98.0%) | 4,927 (98.5%) |
| Bounced | 159 (2.0%) | 73 (1.5%) |
| Opens | 3,762 (48%) | 2,217 (45%) |
| Clicks | 412 (5.2%) | 251 (5.1%) |
| Form submits | 89 | 51 |
| Form-submit consent rate (opt-in to marketing) | 78% | 84% |
| Complaint rate | 0.02% | 0.04% |
| Leads created in CRMs | 89 (Glacier-attributed) | 51 (PartnerLift-attributed) |

All thresholds within range. No deliverability incidents. Audit: `EVT-J154-CAMPAIGN-LAUNCH-DAY1-METRICS-012`.

## Jan 13 — A spam-trap hit

At 09:14 CET on Jan 13, the `comms-email` deliverability monitor catches that one of the German send-list addresses is a known spam-trap. The campaign auto-suppresses the address, emits `EVT-J154-COMMS-EMAIL-SPAM-TRAP-CAUGHT-013`, and surfaces the case to Henrik. Henrik investigates: the address was scraped from a stale list. He removes 47 other questionable addresses from that source. Reputation budget intact.

## End-of-Q1 — Mar 31 23:59 CET attribution settlement

Three months later. The €180K escrow settles via `payments`:

- Glacier-sourced leads converted to opportunities: 482 (PartnerLift gets 40% credit on these)
- PartnerLift-sourced leads converted: 311 (Glacier gets 40% credit on these)
- Joint-pool leads converted: 184 (50/50 split)
- Total revenue attributed to the campaign (per the joint analytics projection): €4.2M ARR signed in Q1

Escrow disbursement:

- €60,000 → Glacier (their original €90K minus the 40% credit owed to PartnerLift)
- €30,000 → PartnerLift (their original €90K minus the 60% credit owed back to Glacier on Glacier-sourced wins)
- €90,000 → settlement balancing per the formula (PartnerLift net receives, Glacier net pays back — actual numbers in the seeded fixture)

Audit: `EVT-J154-PAYMENTS-ATTRIBUTION-SETTLEMENT-014`.

## Apr 1 — Shared tenant wind-down

The shared tenant moves from `active` → `winding_down`. New writes are refused. Reads continue for 90 days for audit purposes. Audit: `EVT-J154-TENANCY-SHARED-WIND-DOWN-015`.

## Jul 1 — Shared tenant archive

The shared tenant moves from `winding_down` → `archived`. Data is held in cold storage per GDPR retention obligations (PartnerLift's policy: 7 years for marketing-attribution audit data). Audit: `EVT-J154-TENANCY-SHARED-ARCHIVED-016`.

## What did NOT happen

- PartnerLift never saw Glacier's internal Salesforce lead list (only the shared pool)
- Glacier never saw PartnerLift's internal HubSpot lead list (only the shared pool)
- No cookie tracking happened on EU users who opted out at the landing-page banner
- No double-opt-in skipped for Dutch prospects
- No spam-trap hit caused a deliverability incident large enough to throttle the campaign
- The shared tenant's tax-treatment is per the contract (PartnerLift handles VAT; out of journey scope)

## Audit-event chain sequence (sealed)

| # | Event class | Day |
|---|---|---|
| 001 | EVT-J154-TENANCY-SHARED-PROVISION-REQUEST | Dec 30 |
| 002 | EVT-J154-TENANCY-SHARED-PROVISIONED | Dec 30 |
| 003 | EVT-J154-CONNECT-DPA-VERIFIED | Dec 30 |
| 004 | EVT-J154-COMMS-EMAIL-DKIM-VERIFY | Dec 30 |
| 005 | EVT-J154-COMPLIANCE-GDPR-LANDING-PAGE-CONSENT | Dec 30 |
| 006 | EVT-J154-CRM-ROUTING-RULES-CONFIGURED | Dec 30 |
| 007 | EVT-J154-CEDAR-DENY-CROSS-PARTNER-CRM-READ | Dec 30 |
| 008 | EVT-J154-COMMUNITY-CHANNEL-CREATE | Dec 30 |
| 009 | EVT-J154-MARKETING-AB-TEST-RESOLVED | Jan 5 |
| 010 | EVT-J154-CAMPAIGN-PRELAUNCH-CHECKLIST-COMPLETE | Jan 9 |
| 011 | EVT-J154-CAMPAIGN-LAUNCH | Jan 12 |
| 012 | EVT-J154-CAMPAIGN-LAUNCH-DAY1-METRICS | Jan 12 |
| 013 | EVT-J154-COMMS-EMAIL-SPAM-TRAP-CAUGHT | Jan 13 |
| 014 | EVT-J154-PAYMENTS-ATTRIBUTION-SETTLEMENT | Mar 31 |
| 015 | EVT-J154-TENANCY-SHARED-WIND-DOWN | Apr 1 |
| 016 | EVT-J154-TENANCY-SHARED-ARCHIVED | Jul 1 |

All events seal under ADR-0263. Every event carries the appropriate `tenant_id` (one of the three) plus `journey_id = j154`.
