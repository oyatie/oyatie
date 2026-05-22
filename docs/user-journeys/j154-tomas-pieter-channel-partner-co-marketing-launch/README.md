---
doc_class: User-Journey-README
journey_id: j154-tomas-pieter-channel-partner-co-marketing-launch
slice: channel-partner-co-marketing-cross-tenant-trinity
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Channel Partner Tomas Pieter
audience_type: B2B_CHANNEL_PARTNER
microservice_count: 5
pack_overlay_anchor: EU-GDPR + NL-Telecommunicatiewet + EU-Digital-Services-Act + ICC-Code-Marketing
related_adrs:
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0243-cedar-as-universal-gate
  - ADR-0263-observability-emission-contract
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0249-multi-category-marketplace-doctrine
  - ADR-0314-marketplace-as-universal-deal-settlement
  - ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability
  - ADR-0272-cookie-consent-per-purpose-analytics-opt-in
---

# j154 — Tomas Pieter: Channel-partner co-marketing campaign launch

## At a glance

Tomas Pieter is a Channel Partner at **PartnerLift B.V.**, an Amsterdam-based reseller representing 14 SaaS vendors across the Benelux + DACH region. PartnerLift has signed a co-marketing agreement with **Glacier ERP GmbH** (Frankfurt) for Q1-2027 to jointly run a campaign targeting **mid-market manufacturing prospects in the Netherlands, Germany, and Belgium**. The campaign budget is €180,000 split 50/50 (€90K Glacier + €90K PartnerLift). Both companies' CMOs signed the contract on December 15. Launch date: January 12.

Today is December 30, 14:11 CET. Tomas is in PartnerLift's WeWork on Weesperstraat. He has 13 days to:

1. Stand up the co-marketing flow across **three tenants in trinity**: PartnerLift (Tomas's home tenant), Glacier ERP (the vendor partner — needs read-write into PartnerLift's leads), and the **shared campaign tenant** `glacier-partnerlift-q1-2027-mfg-de-nl-be` (a temporary tenant per ADR-0244 that holds the joint asset library, the joint lead pool, and the joint attribution model)
2. Configure `marketing-automation` for the bilingual NL + DE email + LinkedIn + Display campaign — including DKIM/SPF/DMARC alignment for both sender domains (ADR-0273)
3. Stand up `crm` synchronisation: PartnerLift uses HubSpot; Glacier uses Salesforce. The shared lead pool flows to both, attributed correctly per the contract's 60/40 rule (60% to whoever sourced; 40% to the partner)
4. Configure `comms-email` deliverability — three sender domains, two languages (NL + DE), per-country reputation budget. EU-GDPR + the Dutch Telecommunicatiewet require **per-purpose** opt-in (ADR-0272)
5. Use `community` to seed a private **partner-only** Slack-like channel for the two marketing teams to coordinate during the campaign
6. Use `connect` to broker the contract-level attestations (CMO-signed co-marketing contract, the EU-GDPR Data Processing Addendum between the three tenants, the Cedar policy bundle that scopes who-sees-what)

Microservices: `marketing-automation`, `crm`, `comms-email`, `community`, `connect`. Secondary touches: `identity` (three-tenant binding), `tenancy` (the shared tenant lifecycle), `audit-chain`, `compliance` (GDPR + Telecom-NL + DSA), `payments` (the 50/50 budget escrow + the attribution settlement at end of Q1), `analytics` (campaign metrics dashboard).

The Cedar policy is a **trinity policy** — three tenants, with the shared tenant being the only place where data from both sides commingles, and even there only the contracted scope.

## Why this journey matters

Tomas Pieter is **MASTER-ROSTER §3.2 row 89** — the canonical channel-partner persona, a category that comprises 11% of B2B GTM motion globally. The journey closes:

- Critical-path row 8 (B2B channel-partner dual-tenant identity)
- Critical-path row 24 (cross-tenant data flow under contract; co-marketing being the most common cross-tenant marketing topology)

Hyperscaler benchmark: HubSpot Marketing Hub + Marketing Contacts; Salesforce Marketing Cloud; PartnerStack for the channel-partner attribution; ZoomInfo for the prospect graph; Mailgun + AWS SES for the per-country email reputation; Slack Connect for the cross-org channel.

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| `story.md` | Beat-by-beat from Dec 30 14:11 CET through launch day Jan 12 09:00 CET | Specific assets, specific German vs Dutch copy lines, specific lead-routing rules |
| `ux-flow.md` | Three personas' screen progressions — Tomas, Glacier's marketing director Henrik, the EU compliance officer Esther | Sender-domain verification screen, the GDPR DPA review modal, the campaign-launch checklist screen |
| `handshake.md` | Per-microservice API + per-tenant scoping | Each row includes the source tenant + the target tenant + the Cedar trinity decision |
| `integration-test-plan.md` | Trinity-tenant tests; per-jurisdiction tests; deliverability tests | Each test names the source tenant + the expected attribution + the expected event class |
| `schemas/openapi-shared-tenant-provision.json` | OpenAPI for `POST /v1/tenants/shared-co-marketing` | The trinity tenant lifecycle |
| `schemas/openapi-campaign-launch.json` | OpenAPI for the campaign launch endpoint | Three-tenant payload |
| `schemas/journey-messages.proto` | proto3 for the 8 RPCs | Field tags, enum values |
| `schemas/cedar-policy.cedar` | Trinity Cedar policy | Three tenants, per-action role check, GDPR scope enforcement |
| `schemas/attribution-rule.yaml` | The 60/40 attribution rule formalised | Per-rule per-source per-destination |

## The five microservices in scope

| µservice | Role | Critical-path row |
|---|---|---|
| `marketing-automation` | Hosts the email + LinkedIn + Display campaigns; runs A/B; respects per-country GDPR consent; ADR-0273 DKIM/SPF/DMARC alignment | row 8 |
| `crm` | Bidirectional sync to HubSpot (PartnerLift) and Salesforce (Glacier); routes shared-pool leads with attribution | row 24 |
| `comms-email` | Per-country deliverability; sender domain reputation tracking; bounce/complaint/feedback-loop handling | row 8 |
| `community` | Private partner-only channel for the two marketing teams | row 24 |
| `connect` | Brokers the contract attestation + DPA + Cedar bundle handoff between the three tenants | row 24 |

## Secondary microservices touched

| µservice | Touch reason |
|---|---|
| `identity` | Tomas authenticates against PartnerLift (home); Henrik authenticates against Glacier; both need cross-tenant grants on the shared tenant |
| `tenancy` | Hosts the lifecycle of the shared tenant (provision → active → wind-down at end of Q1 → archived) |
| `audit-chain` | Seals every cross-tenant grant, every lead movement, every attribution event |
| `compliance` | Activates EU-GDPR, NL-Telecom, EU-DSA, ICC-Code-Marketing packs |
| `payments` | Holds the €180K escrow split 50/50; releases per attribution settlement at end of Q1 |
| `analytics` | Builds the campaign dashboard; per-tenant projection (each side sees their own attribution, not the other's gross-revenue numbers) |

## Pack overlays

| Pack | Activation reason |
|---|---|
| EU-GDPR | Data subjects are EU residents; the campaign processes personal data under GDPR Art 6(1)(f) (legitimate interests for B2B prospecting) + Art 6(1)(a) (consent for direct-to-individual email) |
| NL-Telecommunicatiewet | The Dutch Telecom Act §11.7 requires double-opt-in for direct B2B email solicitation to natural persons; soft opt-in exception for existing customers |
| EU-DSA | The Digital Services Act mandates VLOP-style transparency on Display advertising even at PartnerLift's scale (under the §27/§28 small-business safe harbor, but still requires logging) |
| ICC-Code-Marketing | International Chamber of Commerce code on direct marketing — voluntary but contract requires |

## Regulatory anchors

1. EU-GDPR Art 6, 7, 14, 28 (lawful basis, consent, transparency, processor obligations)
2. EU-GDPR DPA between PartnerLift, Glacier, and the shared tenant (a tri-party DPA per the European Data Protection Board guidance on joint controllers)
3. NL-Telecom §11.7 (direct-marketing email opt-in)
4. EU-DSA Art 24 (transparency reporting for online advertising)
5. ADR-0273 per-tenant DKIM/SPF/DMARC (each sender domain has its own reputation; aligned per RFC 5321/RFC 8617)
6. ADR-0272 cookie-consent per purpose (Display advertising tracking)
7. ADR-0311 dual-tenant identity (Tomas-as-partner vs Tomas-as-employee-of-partner-co)
8. ADR-0244 tenant scoping (three tenants, three sets of audit events)

## Cell + certification matrix

| Cell | Certification | Journey use |
|---|---|---|
| `eu-frankfurt-primary` | EU-GDPR-ready + ISO 27001 + EU-Cloud-Code-of-Conduct | Primary placement for Glacier; data-residency in EU/EEA |
| `eu-amsterdam-secondary` | EU-GDPR-ready | Primary for PartnerLift + the shared tenant |
| `eu-paris-readonly-replica` | EU-GDPR-ready | Read replica for Belgian prospects (Wallonia handled via NL replica; Flanders via NL replica too) |

## Cedar trinity policy (excerpt — full text in `schemas/cedar-policy.cedar`)

```cedar
// PartnerLift home — Tomas full power
permit (
    principal == User::"tomas.pieter@partnerlift.nl",
    action,
    resource is Tenant
) when {
    resource.tenant_id == "partnerlift_nl" &&
    principal.role_in_tenant("partnerlift_nl") == "channel_partner_manager"
};

// Glacier home — Henrik full power
permit (
    principal == User::"henrik.faulkner@glacier-erp.de",
    action,
    resource is Tenant
) when {
    resource.tenant_id == "glacier_erp_de" &&
    principal.role_in_tenant("glacier_erp_de") == "marketing_director"
};

// Shared tenant — both can act, scope-limited
permit (
    principal,
    action in [
        Action::"campaign.author",
        Action::"campaign.review",
        Action::"crm.lead_route",
        Action::"comms.send",
        Action::"community.post"
    ],
    resource is Tenant
) when {
    resource.tenant_id == "glacier-partnerlift-q1-2027-mfg-de-nl-be" &&
    (
        principal.role_in_tenant("glacier-partnerlift-q1-2027-mfg-de-nl-be") == "joint_controller_partnerlift" ||
        principal.role_in_tenant("glacier-partnerlift-q1-2027-mfg-de-nl-be") == "joint_controller_glacier"
    ) &&
    context.gdpr_lawful_basis_set == true &&
    context.dpa_signed == true
};

// Forbid PartnerLift role from reading Glacier internal CRM
forbid (
    principal,
    action == Action::"crm.read",
    resource is Tenant
) when {
    resource.tenant_id == "glacier_erp_de" &&
    principal.role_in_tenant("glacier_erp_de") notIn ["marketing_director", "sales_director", "system_admin"]
};
```

## Acceptance summary

| AC | Result expected |
|---|---|
| AC-J154-001 | Shared tenant `glacier-partnerlift-q1-2027-mfg-de-nl-be` provisioned; PartnerLift + Glacier both granted joint-controller role; data-residency = `eu-amsterdam-secondary` |
| AC-J154-002 | Tri-party GDPR DPA stored in connect under all three tenant audit trails; signature verified for all three CMO + DPO signatories |
| AC-J154-003 | DKIM/SPF/DMARC verified for `mfg.glacier-erp.de`, `mfg.partnerlift.nl`, and `joint.glacier-partnerlift.eu` — per-domain reputation budgets configured |
| AC-J154-004 | Bilingual NL + DE email sequences live; per-country sending-time-optimization configured; per-country GDPR consent gate present on landing pages |
| AC-J154-005 | Display campaign live on LinkedIn Ads + Google Display; DSA transparency log writes every impression-class event to audit-chain |
| AC-J154-006 | crm lead routing: HubSpot receives PartnerLift-sourced leads; Salesforce receives Glacier-sourced leads; shared-pool leads land in both with attribution metadata (60% source / 40% partner) |
| AC-J154-007 | The Slack-Connect-class community channel created; 8 PartnerLift + 6 Glacier marketers invited; data-residency confirmed to eu-amsterdam-secondary |
| AC-J154-008 | Campaign launch button works (Jan 12 09:00 CET); first 1,000 emails fan out; bounce rate < 3%; complaint rate < 0.1%; per-country deliverability ≥ 97% within 4h |
| AC-J154-009 | Cedar denies a PartnerLift role reading Glacier's HubSpot-equivalent internal CRM (only the shared lead pool is cross-visible) |
| AC-J154-010 | At end of Q1 (Mar 31 23:59 CET), the €180K escrow settles per attribution; each tenant sees only their own attribution numbers in analytics |

## Cross-references

- Persona dossier: `docs/personas/tomas-pieter.md`
- MASTER-ROSTER §3.2 row 89
- Matrix §10 j154 recommendation
- Related: j101 (multi-tier supply-chain formation), j105 (cross-tenant arbitration), j112 (tenant-to-tenant RFQ + bid)
- Pack roster: `packs/eu-gdpr/`, `packs/nl-telecom/`, `packs/eu-dsa/`, `packs/icc-marketing/`
- ADR-0273 per-tenant DKIM/SPF/DMARC
- ADR-0314 marketplace as universal deal settlement (the attribution-based €180K release is a marketplace-substrate use)
- ADR-0249 multi-category marketplace doctrine

## Stop condition

This journey is complete when all 10 acceptance criteria pass on the seeded test fixture, the schema files validate, every named ADR resolves, every named µservice exists in `/microservices/`, and the persona dossier matches MASTER-ROSTER §3.2 row 89.
