---
doc_class: User-Journey-Handshake
journey_id: j154-tomas-pieter-channel-partner-co-marketing-launch
date: 2026-05-20
authority_tier: 2
status: draft
---

# j154 — Handshake matrix

This document enumerates every named µservice call exchanged across the **trinity tenant topology** (`partnerlift_nl` + `glacier_erp_de` + `glacier-partnerlift-q1-2027-mfg-de-nl-be`) during the Dec-30 → Jan-12 → Mar-31 → Jul-1 lifecycle of the joint co-marketing campaign. Order matches the timeline in `story.md`. Every row carries source + target tenant, exact API surface, wire shape, Cedar trinity-permit class, and ADR-0263 audit-event class.

## Notation

- `[P]` source tenant `partnerlift_nl`
- `[G]` source tenant `glacier_erp_de`
- `[S]` source tenant `glacier-partnerlift-q1-2027-mfg-de-nl-be`
- `→` synchronous request from caller to callee
- `←` synchronous response
- `↪` side-effect event published to the message bus
- `⟂` denied trinity path (must be tested in `integration-test-plan.md`)

All transport is HTTPS over QUIC (HTTP/3, ADR-0253). All inter-service gRPC also rides HTTP/3.

## §1 Tenancy — provisioning the shared tenant

### 1.1 Shared-tenant provisioning request

`[P] → tenancy` — `POST /v1/tenants/shared-co-marketing` (OpenAPI in `schemas/openapi-shared-tenant-provision.json`)

Request:

```json
{
  "requested_tenant_id": "glacier-partnerlift-q1-2027-mfg-de-nl-be",
  "tenant_class": "shared.co_marketing.joint_controller_pair",
  "joint_controllers": [
    {"tenant_id": "partnerlift_nl", "role": "joint_controller_partnerlift", "sponsor_principal": "tomas.pieter@partnerlift.nl"},
    {"tenant_id": "glacier_erp_de", "role": "joint_controller_glacier",     "sponsor_principal": "henrik.faulkner@glacier-erp.de"}
  ],
  "data_residency_primary": "eu-amsterdam-secondary",
  "data_residency_replicas": ["eu-frankfurt-primary", "eu-paris-readonly-replica"],
  "lifecycle_plan": {
    "active_until": "2027-03-31T23:59:59+01:00",
    "wind_down_at":  "2027-04-01T00:00:00+02:00",
    "archive_at":    "2027-07-01T00:00:00+02:00",
    "retention_after_archive_years": 7
  },
  "co_sign_required": true,
  "pack_overlays_requested": ["eu-gdpr", "nl-telecom", "eu-dsa", "icc-marketing"]
}
```

Response (`202 Accepted` — co-sign required):

```json
{
  "provisioning_id": "tprv-2026-1230-141318-glacpl-mfg-q1",
  "status": "pending_co_sign",
  "co_sign_pending_from": ["henrik.faulkner@glacier-erp.de"],
  "co_sign_request_url": "https://connect.oya.network/co-sign/tprv-2026-1230-141318",
  "expires_at": "2026-12-31T13:13:18+01:00"
}
```

Cedar trinity-permit class: `tenancy.shared_co_marketing_provision`. Evaluated against:

```
principal == User::"tomas.pieter@partnerlift.nl"
action    == Action::"tenancy.provision_shared_co_marketing"
resource  == Tenant::"partnerlift_nl"     // home tenant sponsorship
context.target_shared_tenant_id == "glacier-partnerlift-q1-2027-mfg-de-nl-be"
context.principal_role_in_home("partnerlift_nl") == "channel_partner_manager"
context.contract_attestation_id != ""
```

Audit: `EVT-J154-TENANCY-SHARED-PROVISION-REQUEST-001`. Replicated to all three tenants per ADR-0263 trinity rule.

### 1.2 Co-sign + provisioning complete

`[G] → tenancy` — `POST /v1/tenants/{provisioning_id}/co-sign`

```json
{
  "provisioning_id": "tprv-2026-1230-141318-glacpl-mfg-q1",
  "co_signer_principal": "henrik.faulkner@glacier-erp.de",
  "co_signer_attestation": {
    "passkey_id": "pk-henrik-yubikey5c-9a31",
    "ed25519_signature": "<b64>",
    "signed_payload_sha256": "<hex>"
  },
  "accept_terms": true
}
```

Response (`201 Created`):

```json
{
  "tenant_id": "glacier-partnerlift-q1-2027-mfg-de-nl-be",
  "tenant_state": "provisioned",
  "data_residency_primary": "eu-amsterdam-secondary",
  "cedar_bundle_id": "cedar-bundle-trinity-glacpl-mfg-q1-v1",
  "joint_controllers_active": ["partnerlift_nl", "glacier_erp_de"],
  "kms_root_key_id": "kms-eu-ams-trinity-glacpl-mfg-q1-001",
  "audit_streams_primed": ["partnerlift_nl", "glacier_erp_de", "glacier-partnerlift-q1-2027-mfg-de-nl-be"]
}
```

Audit: `EVT-J154-TENANCY-SHARED-PROVISIONED-002` (sealed in all 3 tenants).

## §2 Connect — tri-party DPA attestation

### 2.1 DPA upload + signature verification

`[S] → connect` — `POST /v1/attestations/dpa-tri-party`

Multipart:

```
provisioning_id: tprv-2026-1230-141318-glacpl-mfg-q1
dpa_pdf: <bytes; PDF/A-3 signed copy; SHA-256 in metadata>
signatories[0]: {"principal":"anneke.vandermeer@partnerlift.nl","role":"controller_partnerlift_cmo","signing_method":"adobe_aes_eu"}
signatories[1]: {"principal":"beate.hoffmann@glacier-erp.de","role":"controller_glacier_cmo","signing_method":"adobe_aes_eu"}
signatories[2]: {"principal":"esther.bakker@vandersluis-law.nl","role":"shared_controller_counsel","signing_method":"docusign_qes_eidas"}
dpa_class: gdpr_art_26_joint_controllers
```

Response (`201 Created`):

```json
{
  "attestation_id": "att-dpa-glacpl-mfg-q1-2026-1230",
  "verification_results": [
    {"signatory":"anneke.vandermeer@partnerlift.nl","verified":true,"signing_cert_chain_root":"AdobeAATL-EU-2024"},
    {"signatory":"beate.hoffmann@glacier-erp.de","verified":true,"signing_cert_chain_root":"AdobeAATL-EU-2024"},
    {"signatory":"esther.bakker@vandersluis-law.nl","verified":true,"signing_cert_chain_root":"eIDAS-QES-NL-KPN-2024"}
  ],
  "stored_in_audit_trails": ["partnerlift_nl","glacier_erp_de","glacier-partnerlift-q1-2027-mfg-de-nl-be"],
  "retention_until": "2034-07-01T00:00:00+02:00"
}
```

Audit: `EVT-J154-CONNECT-DPA-VERIFIED-003`.

Failure modes:

- `⟂` signing cert revoked at time-of-sign → `409 Conflict` + `EVT-J154-CONNECT-DPA-CERT-REVOKED-NNN`
- `⟂` only 2 of 3 signatures → `400 Bad Request` + `EVT-J154-CONNECT-DPA-INCOMPLETE-SIGNATORIES-NNN`

## §3 Comms-email — DKIM/SPF/DMARC alignment (ADR-0273)

### 3.1 Sender-domain registration

For each of three domains, `[S] → comms-email` — `POST /v1/sender-domains`:

```json
{
  "domain": "mfg.glacier-erp.de",
  "owner_tenant": "glacier_erp_de",
  "use_in_shared_tenants": ["glacier-partnerlift-q1-2027-mfg-de-nl-be"],
  "reputation_class": "marketing_b2b_mid_market",
  "daily_send_budget_planned": 15000,
  "warmup_curve_days": 0,
  "dkim_selector": "oya-mfg-2027a",
  "spf_include": "_spf.oya-mail-eu.network",
  "dmarc_policy_target": "p=reject; rua=mailto:dmarc@glacier-erp.de"
}
```

Response (`201 Created`):

```json
{
  "sender_domain_id": "snd-mfgglacde-2026-1230",
  "dns_records_required": [
    {"type":"TXT","name":"oya-mfg-2027a._domainkey.mfg.glacier-erp.de","value":"v=DKIM1; k=rsa; p=MIIBIjANBgkqhkiG..."},
    {"type":"TXT","name":"mfg.glacier-erp.de","value":"v=spf1 include:_spf.oya-mail-eu.network -all"},
    {"type":"TXT","name":"_dmarc.mfg.glacier-erp.de","value":"v=DMARC1; p=reject; rua=mailto:dmarc@glacier-erp.de; fo=1"}
  ],
  "next_step": "publish_dns_then_verify"
}
```

After Esmé van Wijk publishes the DNS records, `[S] → comms-email` — `POST /v1/sender-domains/{id}/verify`:

```json
{"sender_domain_id": "snd-jntglacpl-2026-1230"}
```

Response when joint domain is aligned (`200 OK`):

```json
{
  "sender_domain_id": "snd-jntglacpl-2026-1230",
  "dkim_aligned": true,
  "spf_aligned": true,
  "dmarc_published": true,
  "reputation_score_start": 72.3,
  "daily_send_budget_active": 5000,
  "warmup_curve": "linear_2x_per_3_days_capped_at_planned"
}
```

Audit: `EVT-J154-COMMS-EMAIL-DKIM-VERIFY-004` (one per domain, three total).

Failure modes:

- `⟂` DKIM record not found → `EVT-J154-COMMS-EMAIL-DKIM-MISSING-NNN`
- `⟂` SPF record contains `+all` (open relay) → `EVT-J154-COMMS-EMAIL-SPF-INSECURE-NNN`
- `⟂` Reputation below floor 30 → domain marked `quarantine_only`

## §4 Marketing-automation — campaign authoring + launch

### 4.1 Author email sequence

`[S] → marketing-automation` — `POST /v1/campaigns/{campaign_id}/sequences`

Path: `campaign_id = camp-glacpl-mfg-q1-2027`

Body (email sequence B, Dutch, 5 emails):

```json
{
  "sequence_id": "seq-glacpl-mfg-q1-nl",
  "audience_segment_id": "seg-mfg-de-nl-be-mid-market-100to1000-fte",
  "audience_country_codes": ["NL"],
  "sender_domain": "mfg.partnerlift.nl",
  "language": "nl-NL",
  "cadence": [
    {"step":1,"send_at_relative_to_enroll":"P0D","subject":"Glacier ERP — beproefd in Duitse mfg","sti_send_window":"09:00-11:00 Europe/Amsterdam","preheader":"Mid-market mfg ERP. Live demo binnen 30 min."},
    {"step":2,"send_at_relative_to_enroll":"P3D","subject":"Hoe Lemmensgroep €2.3M reststock liquideerde","sti_send_window":"10:00-12:00"},
    {"step":3,"send_at_relative_to_enroll":"P7D","subject":"3-stappen-roadmap voor uw ERP-migratie","sti_send_window":"09:00-11:00"},
    {"step":4,"send_at_relative_to_enroll":"P14D","subject":"Webinar — Glacier vs SAP S/4 vs Microsoft Dynamics","sti_send_window":"14:00-16:00"},
    {"step":5,"send_at_relative_to_enroll":"P28D","subject":"Laatste herinnering — pilot Q2-2027","sti_send_window":"09:00-11:00"}
  ],
  "consent_basis": ["gdpr_art_6_1_f_legitimate_interest_b2b","nl_telecom_11_7_double_opt_in_for_natural_persons"],
  "unsubscribe_link_required": true,
  "list_unsubscribe_header": "<mailto:unsubscribe-mfgpl@partnerlift.nl>, <https://joint.glacier-partnerlift.eu/unsubscribe?seq=glacpl-mfg-q1-nl&p={persona_token}>",
  "attribution_rule_id": "attr-rule-glacpl-60-40"
}
```

Response (`201 Created`):

```json
{
  "sequence_id": "seq-glacpl-mfg-q1-nl",
  "status": "draft",
  "audit_event": "EVT-J154-MARKETING-ASSET-AUTHOR-002-NL"
}
```

Cedar permit: `campaign.author` against `Tenant::"glacier-partnerlift-q1-2027-mfg-de-nl-be"`. Audit: `EVT-J154-MARKETING-ASSET-AUTHOR-NNN` (29 events covering 10 emails + 8 LinkedIn frames + 6 display banners + 5 landing-page variants).

### 4.2 Campaign launch

`[S] → marketing-automation` — `POST /v1/campaigns/{campaign_id}/launch` (OpenAPI in `schemas/openapi-campaign-launch.json`)

Body:

```json
{
  "campaign_id": "camp-glacpl-mfg-q1-2027",
  "scheduled_launch_at": "2027-01-12T09:00:00+01:00",
  "sequences_in_scope": ["seq-glacpl-mfg-q1-de","seq-glacpl-mfg-q1-nl"],
  "linkedin_ads_in_scope": ["li-camp-glacpl-mfg-q1-de","li-camp-glacpl-mfg-q1-nl-flemish","li-camp-glacpl-mfg-q1-walloon-fr"],
  "google_display_in_scope": ["gdn-camp-glacpl-mfg-q1-de","gdn-camp-glacpl-mfg-q1-nl-be"],
  "launching_principal": "tomas.pieter@partnerlift.nl",
  "approval_chain_present": ["anneke.vandermeer@partnerlift.nl","beate.hoffmann@glacier-erp.de"],
  "deliverability_pre_check_passed": true,
  "dpa_attestation_id": "att-dpa-glacpl-mfg-q1-2026-1230",
  "cedar_bundle_id": "cedar-bundle-trinity-glacpl-mfg-q1-v1"
}
```

Response (`202 Accepted`):

```json
{
  "launch_id": "lnch-glacpl-mfg-q1-2027-1112-0900",
  "state": "armed_for_scheduled_launch",
  "estimated_first_hour_email_volume": 13000,
  "linkedin_first_hour_impression_target": 48000,
  "google_display_first_hour_impression_target": 92000
}
```

Audit: `EVT-J154-CAMPAIGN-LAUNCH-011` at launch wall-clock T0.

Failure modes:

- `⟂` DPA attestation missing → `409 Conflict` + `EVT-J154-CAMPAIGN-LAUNCH-MISSING-DPA-NNN`
- `⟂` Sender-domain reputation below floor → `409 Conflict` + `EVT-J154-CAMPAIGN-LAUNCH-DELIVERABILITY-FLOOR-NNN`
- `⟂` Approval chain missing CMO of either side → `403 Forbidden` + `EVT-J154-CAMPAIGN-LAUNCH-APPROVAL-INCOMPLETE-NNN`

## §5 CRM — bidirectional sync + lead routing

### 5.1 Configure lead-routing rules

`[S] → crm` — `POST /v1/routing-rules`

Body:

```json
{
  "tenant_id": "glacier-partnerlift-q1-2027-mfg-de-nl-be",
  "attribution_rule_id": "attr-rule-glacpl-60-40",
  "rule_set": [
    {"source":"mfg.glacier-erp.de.email_form_submit","route_to":["hubspot://partnerlift_nl","salesforce://glacier_erp_de"],"attribution_source_share":0.60,"attribution_partner_share":0.40},
    {"source":"mfg.partnerlift.nl.email_form_submit","route_to":["hubspot://partnerlift_nl","salesforce://glacier_erp_de"],"attribution_source_share":0.60,"attribution_partner_share":0.40},
    {"source":"joint.glacier-partnerlift.eu.lp_form_fill","route_to":["hubspot://partnerlift_nl","salesforce://glacier_erp_de"],"attribution_source_share":0.50,"attribution_partner_share":0.50},
    {"source":"linkedin.lead_gen_form.glacier_funded","route_to":["hubspot://partnerlift_nl","salesforce://glacier_erp_de"],"attribution_source_share":0.60,"attribution_partner_share":0.40}
  ],
  "deduplication_key_priority": ["email","company_domain_norm","linkedin_urn"],
  "consent_propagation_required": true,
  "audit_per_route": true
}
```

Response (`201 Created`):

```json
{
  "rule_set_id": "rule-set-glacpl-mfg-q1-2027",
  "active": true,
  "hubspot_bridge_status": "connected_oauth_partnerlift_nl",
  "salesforce_bridge_status": "connected_oauth_glacier_erp_de"
}
```

Audit: `EVT-J154-CRM-ROUTING-RULES-CONFIGURED-006`.

### 5.2 Lead created → routed

On every form submit or LinkedIn lead-gen-form, `crm` fans out:

```
[S] crm internal event "lead.created"
   ↪ [P] hubspot upsert via POST https://api.hubapi.com/crm/v3/objects/contacts (oauth-bridged)
   ↪ [G] salesforce upsert via POST https://glacier.my.salesforce.com/services/data/v60.0/sobjects/Lead (oauth-bridged)
   ↪ [S] audit EVT-J154-CRM-LEAD-ROUTED-NNN
```

Per-lead payload (after dedup):

```json
{
  "lead_id_shared": "lead-glacpl-mfg-q1-2027-000847",
  "email_hash": "<sha256 of normalized email>",
  "email_pii_encrypted_blob": "<gcm bytes>",
  "company_domain": "stalengieterij-utrecht.nl",
  "company_size_band": "100_500_fte",
  "country": "NL",
  "source_url": "https://joint.glacier-partnerlift.eu/mfg/nl?utm_source=mfg.partnerlift.nl&utm_campaign=q1-2027-seq-b-step-1",
  "form_id": "lp-joint-mfg-nl",
  "consent": {
    "marketing_email_optin": true,
    "marketing_email_optin_method": "double_optin_confirmed",
    "marketing_email_optin_at": "2027-01-12T11:42:03+01:00",
    "tracking_cookies_optin": false,
    "personalization_optin": true
  },
  "co_marketing_attribution": {
    "rule_id": "attr-rule-glacpl-60-40",
    "source_partner": "partnerlift_nl",
    "partner_partner": "glacier_erp_de",
    "source_share": 0.60,
    "partner_share": 0.40
  }
}
```

### 5.3 Denied — cross-partner internal CRM read

`[P-principal-in-G-context] → crm` — `GET /v1/internal-leads?owner_tenant=glacier_erp_de`

Cedar evaluates:

```
principal == User::"tomas.pieter@partnerlift.nl"
action    == Action::"crm.read"
resource  == Tenant::"glacier_erp_de"
principal.role_in_tenant("glacier_erp_de") == "joint_controller_partnerlift"  // NOT in [marketing_director, sales_director, system_admin]
decision: deny
```

Response (`403 Forbidden`):

```json
{
  "error":"cedar_deny",
  "policy_id":"cedar-trinity-glacpl-forbid-cross-partner-internal-crm",
  "human_explanation":"You can read leads inside the shared tenant. You cannot read Glacier's internal CRM. This boundary is set by ADR-0311 and the tri-party DPA.",
  "recovery_path":"open_shared_lead_pool"
}
```

Audit: `EVT-J154-CEDAR-DENY-CROSS-PARTNER-CRM-READ-007`.

## §6 Community — partner-only channel

### 6.1 Create channel

`[S] → community` — `POST /v1/channels`

Body:

```json
{
  "tenant_id": "glacier-partnerlift-q1-2027-mfg-de-nl-be",
  "channel_name": "mfg-q1-2027-glacier-partnerlift-coord",
  "channel_class": "private_partner_only",
  "data_residency": "eu-amsterdam-secondary",
  "e2ee": "mls_rfc_9420",
  "default_retention_days": 365,
  "members": [
    {"principal":"tomas.pieter@partnerlift.nl","role":"channel_admin"},
    {"principal":"anneke.vandermeer@partnerlift.nl","role":"member"},
    {"principal":"mira.devries@partnerlift.nl","role":"member"},
    {"principal":"joost.lievens@partnerlift.nl","role":"member"},
    {"principal":"roos.vanveen@partnerlift.nl","role":"member"},
    {"principal":"bram.dehaan@partnerlift.nl","role":"member"},
    {"principal":"lara.dewit@partnerlift.nl","role":"observer_dpo"},
    {"principal":"hendrik.bos@partnerlift.nl","role":"member"},
    {"principal":"henrik.faulkner@glacier-erp.de","role":"channel_admin"},
    {"principal":"beate.hoffmann@glacier-erp.de","role":"member"},
    {"principal":"pia.weber@glacier-erp.de","role":"member"},
    {"principal":"klaus.lehmann@glacier-erp.de","role":"member"},
    {"principal":"stefan.koehler@glacier-erp.de","role":"observer_dpo"},
    {"principal":"frieda.bauer@glacier-erp.de","role":"member"}
  ]
}
```

Response (`201 Created`):

```json
{
  "channel_id": "ch-mfgq1-glacpl-coord-2026-1230",
  "mls_group_id": "<opaque>",
  "members_active": 14,
  "audit_event": "EVT-J154-COMMUNITY-CHANNEL-CREATE-008"
}
```

## §7 Workflow-engine — campaign timers + escrow release

### 7.1 Q1 attribution-settlement timer

`[S] → workflow-engine` — `POST /v1/workflows/{wf_id}/timers`

Path: `wf_id = wf-camp-glacpl-mfg-q1-2027`

Body:

```json
{
  "timer_name": "q1_attribution_settlement",
  "fires_at": "2027-03-31T23:59:59+01:00",
  "reminder_at_minus_seconds": [604800, 86400],
  "on_fire_action": "trigger_payments_escrow_release",
  "on_reminder_action": "notify_both_dpo_for_data_review",
  "guard_evaluator": "payments.attribution_finalized == true"
}
```

Response: `{"timer_id":"tmr-q1-attribution-settlement-camp-glacpl"}`. Audit: `EVT-J154-WORKFLOW-TIMER-SET-Q1-SETTLEMENT-NNN`.

## §8 Payments — escrow + Q1 attribution settlement

### 8.1 Escrow setup

`[S] → payments` — `POST /v1/escrows`

Body:

```json
{
  "tenant_id": "glacier-partnerlift-q1-2027-mfg-de-nl-be",
  "purpose": "co_marketing_q1_2027_budget",
  "currency": "EUR",
  "amount_minor_units": 18000000,
  "contributions": [
    {"contributor_tenant":"partnerlift_nl","amount_minor_units":9000000,"sepa_credit_at":"2026-12-22"},
    {"contributor_tenant":"glacier_erp_de","amount_minor_units":9000000,"sepa_credit_at":"2026-12-22"}
  ],
  "release_rule_ref":"attr-rule-glacpl-60-40",
  "released_at_target":"2027-03-31T23:59:59+01:00",
  "settlement_partition_method":"attribution_weighted_per_lead_per_revenue"
}
```

Response (`201 Created`):

```json
{
  "escrow_id":"esc-glacpl-mfg-q1-2027",
  "state":"funded",
  "kms_partition_key":"kms-eu-ams-escrow-glacpl-mfg-q1-001",
  "release_function":"f_settle_v3_co_marketing_60_40_with_split_credit_pool"
}
```

### 8.2 Q1 settlement

At T = 2027-03-31T23:59:59+01:00, workflow-engine fires the timer, payments computes:

```
revenue_attributed_to_campaign_q1 = €4,232,118 ARR signed
joint_pool_split_credit = 50/50 on 184 converted leads
glacier_sourced_leads_converted = 482 → PartnerLift receives 40% credit on those
partnerlift_sourced_leads_converted = 311 → Glacier receives 40% credit on those
```

Resulting disbursement:

```json
{
  "escrow_id": "esc-glacpl-mfg-q1-2027",
  "settlement_id": "stmt-glacpl-mfg-q1-2027-03-31",
  "transfers": [
    {"to_tenant":"glacier_erp_de","amount_minor_units":6041800,"reason":"original_contribution_less_owed_to_partner"},
    {"to_tenant":"partnerlift_nl","amount_minor_units":11958200,"reason":"original_contribution_plus_credit_owed_by_partner"}
  ],
  "audit_event":"EVT-J154-PAYMENTS-ATTRIBUTION-SETTLEMENT-014"
}
```

## §9 Compliance — DSA transparency + DPA renewal

### 9.1 DSA transparency log writes

For each LinkedIn + Google Display impression-class event, `compliance` writes a transparency record:

```proto
message DsaImpressionRecord {
  string campaign_id = 1;
  string ad_id = 2;
  string platform = 3;             // "linkedin" | "google_display"
  string targeting_criteria_hash = 4;
  string targeting_criteria_plain = 5;   // human-readable per DSA Art 26
  string advertiser_legal_name = 6;
  string advertiser_paid_for_by = 7;     // both partner names per joint
  google.protobuf.Timestamp impression_at = 8;
  string country_eu = 9;
  string tenant_id = 10;
}
```

Audit per batch: `EVT-J154-COMPLIANCE-DSA-IMPRESSION-LOG-BATCH-NNN`.

## §10 Audit-chain — sealing contract

Every event seals via `audit-chain`:

```proto
message AuditSealRequest {
  string event_class = 1;          // EVT-J154-...
  string tenant_id = 2;            // partnerlift_nl | glacier_erp_de | glacier-partnerlift-q1-2027-mfg-de-nl-be
  string journey_id = 3;           // j154
  string trace_id = 4;
  string subject_principal = 5;
  string resource_ref = 6;
  google.protobuf.Timestamp occurred_at = 7;
  google.protobuf.Struct payload = 8;
  string emitting_microservice = 9;
  string trinity_replication_tenants = 10;  // CSV when event must seal in >1 tenant
}
```

Trinity-replication rule: any event whose `payload` references data from another tenant in the trinity also seals in that tenant. The merkle proofs are anchored per-tenant; a cross-tenant verifier walks all three.

## §11 Denied paths (must be exercised by integration tests)

| Denied trinity action | Reason | Audit-event class |
|---|---|---|
| PartnerLift principal reads Glacier internal CRM | Cedar trinity forbid: role not in [marketing_director, sales_director, system_admin] | `EVT-J154-CEDAR-DENY-CROSS-PARTNER-CRM-READ` |
| Either principal writes to the shared tenant before DPA verified | Cedar trinity: `context.dpa_signed == false` | `EVT-J154-CEDAR-DENY-NO-DPA` |
| Campaign launch without sender-domain alignment | Pre-flight floor fails | `EVT-J154-CAMPAIGN-LAUNCH-DELIVERABILITY-FLOOR` |
| Lead routed without consent | NL-Telecom §11.7 + GDPR Art 6 | `EVT-J154-CRM-DENY-LEAD-NO-CONSENT` |
| Cross-tenant marketing-asset preview (PartnerLift principal previews Glacier-internal asset) | Trinity scoping | `EVT-J154-CEDAR-DENY-CROSS-PARTNER-ASSET-PREVIEW` |
| DPA upload with revoked signing cert | Connect verification fails | `EVT-J154-CONNECT-DPA-CERT-REVOKED` |
| Escrow release before T-fire | Workflow guard fails | `EVT-J154-PAYMENTS-DENY-PREMATURE-RELEASE` |
| Shared tenant write after Apr 1 wind-down | Lifecycle state machine | `EVT-J154-TENANCY-DENY-WRITE-DURING-WINDDOWN` |
| Public reads of joint analytics by non-controller | Trinity Cedar | `EVT-J154-CEDAR-DENY-ANALYTICS-NON-CONTROLLER` |

## §12 Cross-µservice timing budget

| Edge | p50 | p95 | p99 |
|---|---|---|---|
| trinity-provision request → co-sign accepted | 30s (human) | 12m (human) | 4h (human SLA) |
| co-sign → tenant active | 240ms | 680ms | 1.4s |
| DPA upload → 3 signatures verified | 480ms | 1.2s | 3.1s |
| sender-domain DNS publish → DKIM aligned | 60s (DNS prop) | 300s | 900s |
| campaign.author event → audit sealed | 90ms | 240ms | 480ms |
| campaign.launch → first 1000 emails dispatched | 8s | 22s | 41s |
| lead.created → routed to both HubSpot + Salesforce | 140ms | 420ms | 1.1s |
| denied cross-CRM read → 403 returned | 60ms | 180ms | 320ms |
| Q1 settlement fire → SEPA initiations | 6m | 14m | 28m |

SLO: launch-time campaign.launch → first delivered email observed (p95) ≤ 90s. Sustained deliverability per sender domain ≥ 97% throughout the campaign window.
