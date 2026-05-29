---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j154-tomas-pieter-channel-partner-co-marketing-launch
date: 2026-05-20
authority_tier: 2
status: draft
---

# j154 — Integration test plan

This plan is intern-buildable: a new engineer stands up the seeded trinity-tenant test environment and walks every test in order. Each test names seed values, exact API calls, expected event chain across all three tenants, and pass/fail criteria.

## Test environment

| Component | Source |
|---|---|
| Seed tenant A | `tests/fixtures/tenants/partnerlift_nl.yaml` |
| Seed tenant B | `tests/fixtures/tenants/glacier_erp_de.yaml` |
| Shared tenant blueprint | `tests/fixtures/tenants/glacier-partnerlift-q1-2027-mfg-de-nl-be.template.yaml` |
| Seed personas | `tests/fixtures/personas/{tomas-pieter,anneke-vandermeer,lara-dewit,henrik-faulkner,beate-hoffmann,stefan-koehler,esther-bakker}.yaml` |
| Seed DPA PDF | `tests/fixtures/contracts/dpa-glacpl-mfg-q1-2027-signed.pdf` (real PDF/A-3, 3 AdES/QES signatures) |
| Seed sender domains | `tests/fixtures/dns/{mfg-glacier-erp-de,mfg-partnerlift-nl,joint-glacier-partnerlift-eu}.zone` |
| Seed Cedar policy bundle | `tests/fixtures/cedar/j154/cedar-bundle-trinity-glacpl-mfg-q1-v1.cedar` |
| Wire mock — HubSpot | `tests/mocks/hubspot-crm-v3.toml` |
| Wire mock — Salesforce | `tests/mocks/salesforce-rest-v60.toml` |
| Wire mock — LinkedIn Ads | `tests/mocks/linkedin-marketing-api-2026.toml` |
| Wire mock — Google Display | `tests/mocks/google-ads-api-v17.toml` |
| Wire mock — SEPA initiation | `tests/mocks/sepa-credit-transfer-pacs008.toml` |
| Wire mock — eIDAS QES validator | `tests/mocks/eidas-qes-validation-2024.toml` |
| Wire mock — DNS resolver | `tests/mocks/unbound-deterministic.toml` (deterministic responses for DKIM/SPF/DMARC queries) |
| Frozen clock | `freeze_clock(2026-12-30T13:11:00+01:00)` then advance per test |

## Test catalog

### T-J154-001 — Trinity provisioning happy path

**Pre-conditions:** clock at `2026-12-30T13:11:00+01:00`. Tomas and Henrik pre-enrolled. PartnerLift home and Glacier home tenants active. Shared tenant slot reserved but not provisioned.

**Action sequence:**

1. POST `/v1/tenants/shared-co-marketing` from `tomas.pieter@partnerlift.nl` with body in handshake §1.1
2. Advance clock 13 minutes
3. POST `/v1/tenants/{provisioning_id}/co-sign` from `henrik.faulkner@glacier-erp.de` with body in handshake §1.2

**Expected events (in order across 3 tenants):**

- `EVT-J154-TENANCY-SHARED-PROVISION-REQUEST-001` (sealed in partnerlift_nl + glacier_erp_de)
- `EVT-J154-TENANCY-SHARED-PROVISIONED-002` (sealed in partnerlift_nl + glacier_erp_de + glacier-partnerlift-q1-2027-mfg-de-nl-be)

**Pass criteria:**

- Shared tenant exists with `state = provisioned`, `data_residency_primary = eu-amsterdam-secondary`
- Both joint controllers active
- Cedar bundle `cedar-bundle-trinity-glacpl-mfg-q1-v1` deployed and validates
- Audit events sealed in all named tenants; merkle proofs cross-validate
- p95 of `co-sign accept → tenant active` ≤ 680ms

**Fail criteria:**

- Shared tenant created before co-sign
- Cedar bundle missing the trinity-forbid policy for cross-partner internal CRM reads
- Any audit event sealed in fewer than the expected tenants

### T-J154-002 — DPA tri-party verification happy path

**Pre-conditions:** T-J154-001 completed.

**Action sequence:**

1. POST `/v1/attestations/dpa-tri-party` with the seed DPA PDF + 3 signatory metadata blocks

**Expected events:**

- `EVT-J154-CONNECT-DPA-VERIFIED-003` (sealed in all 3 tenants)

**Pass criteria:**

- All 3 signatures verify against their respective AATL/eIDAS roots
- `retention_until = 2034-07-01T00:00:00+02:00` (7 years after archive)
- DPA stored in all 3 tenants' audit trails (replicated, not linked-by-reference) per GDPR Art 30
- `context.dpa_signed` flag flips to `true` in the trinity Cedar context

**Fail criteria:**

- Any signature verification skipped or marked "trusted without check"
- DPA stored by reference only (a single canonical record) — would fail Art 30 record-keeping

### T-J154-003 — DPA fails when one cert is revoked

**Pre-conditions:** T-J154-001 done. Esther's eIDAS cert revoked at the CRL endpoint mocked.

**Action sequence:** same as T-J154-002.

**Expected events:**

- `EVT-J154-CONNECT-DPA-CERT-REVOKED-NNN`

**Pass criteria:**

- returns `409 Conflict`
- DPA not stored in any tenant's audit trail (failure prevents replication)
- `context.dpa_signed` remains `false`

**Fail criteria:**

- DPA stored despite revoked cert
- `context.dpa_signed` flips to `true`

### T-J154-004 — Sender-domain alignment per ADR-0273

**Pre-conditions:** T-J154-002 completed.

**Action sequence:**

1. POST `/v1/sender-domains` for `mfg.glacier-erp.de` (DKIM + SPF + DMARC already aligned)
2. POST `/v1/sender-domains` for `mfg.partnerlift.nl` (same)
3. POST `/v1/sender-domains` for `joint.glacier-partnerlift.eu` (DNS not yet published)
4. Advance DNS mock to publish the joint-domain TXT records
5. POST `/v1/sender-domains/{id}/verify` for the joint domain

**Expected events:**

- `EVT-J154-COMMS-EMAIL-DKIM-VERIFY-004` (one per domain, 3 total)

**Pass criteria:**

- All 3 domains end in `dkim_aligned=true, spf_aligned=true, dmarc_published=true`
- Joint domain starts with `daily_send_budget_active=5000` (linear warm-up)
- Reputation scores ≥ floor of 30

**Fail criteria:**

- Joint domain marked aligned before DNS publication
- Any domain with `daily_send_budget_active=0` after verification

### T-J154-005 — Sender-domain DKIM missing

**Pre-conditions:** T-J154-002 done. DNS mock does NOT publish DKIM for the joint domain.

**Action sequence:**

1. POST `/v1/sender-domains` for `joint.glacier-partnerlift.eu`
2. POST `/v1/sender-domains/{id}/verify` immediately

**Expected events:**

- `EVT-J154-COMMS-EMAIL-DKIM-MISSING-NNN`

**Pass criteria:**

- Verification returns `424 Failed Dependency` with body naming the missing record
- Joint domain marked `dkim_aligned=false`, `daily_send_budget_active=0`
- Campaign launch later refuses to arm with this domain in scope (see T-J154-009)

### T-J154-006 — Cedar trinity-forbid: PartnerLift principal reads Glacier internal CRM

**Pre-conditions:** Trinity active. Tomas has `joint_controller_partnerlift` role on shared tenant; no role on Glacier internal CRM.

**Action sequence:**

1. POST `/v1/cedar/decide` with principal `tomas.pieter@partnerlift.nl`, action `crm.read`, resource `Tenant::"glacier_erp_de"`

**Expected events:**

- `EVT-J154-CEDAR-DENY-CROSS-PARTNER-CRM-READ-007`

**Pass criteria:**

- Cedar returns `deny`
- Deny reason names `principal.role_in_tenant("glacier_erp_de") notIn ["marketing_director","sales_director","system_admin"]`
- Human-readable explanation cites ADR-0311 + the tri-party DPA
- Recovery path link points to the shared lead pool

**Fail criteria:**

- Cedar returns `permit`
- Deny message leaks Glacier internal CRM data structure or row count

### T-J154-007 — Lead routing happy path (60/40 attribution)

**Pre-conditions:** T-J154-004 done. CRM routing rules active. HubSpot + Salesforce wire mocks return success.

**Action sequence:**

1. Simulate 10 form submits on `joint.glacier-partnerlift.eu/mfg/nl` with full GDPR + NL-Telecom consent
2. Simulate 10 LinkedIn lead-gen-form submits attributed to Glacier-funded ad

**Expected events (per lead):**

- `EVT-J154-CRM-LEAD-ROUTED-NNN`

**Pass criteria:**

- Each lead upserts into BOTH HubSpot and Salesforce with the same `co_marketing_attribution` object
- Joint LP lead: `attribution_source_share=0.50, attribution_partner_share=0.50`
- LinkedIn lead: `attribution_source_share=0.60` to Glacier, `attribution_partner_share=0.40` to PartnerLift
- Deduplication by email-hash: same email twice → second route is idempotent (returns existing lead_id_shared)
- p95 of `lead.created → both CRMs routed` ≤ 420ms

**Fail criteria:**

- Any lead routed to only one CRM
- Attribution percentages off by ≥1 percentage point
- Duplicate leads with different lead_id_shared (dedup broken)

### T-J154-008 — Lead routing denied when consent missing

**Pre-conditions:** T-J154-004 done.

**Action sequence:**

1. Simulate a Dutch form submit where `marketing_email_optin_method != "double_optin_confirmed"`
2. Attempt route

**Expected events:**

- `EVT-J154-CRM-DENY-LEAD-NO-CONSENT-NNN`

**Pass criteria:**

- CRM rejects with `412 Precondition Failed`
- Lead held in a quarantine queue for 24 hours awaiting double-opt-in confirmation
- After 24 hours with no confirmation, lead purged (NL Telecom §11.7 + GDPR Art 5(1)(c) minimisation)
- Neither HubSpot nor Salesforce receives the lead

### T-J154-009 — Campaign launch refused without sender-domain alignment

**Pre-conditions:** T-J154-005 simulated (one sender domain misaligned).

**Action sequence:**

1. POST `/v1/campaigns/{id}/launch`

**Expected events:**

- `EVT-J154-CAMPAIGN-LAUNCH-DELIVERABILITY-FLOOR-NNN`

**Pass criteria:**

- Launch returns `409 Conflict`
- Response names the failing domain
- Audit event names the deliverability gate
- Campaign state remains `armed_for_scheduled_launch` but `launch_blocked=true`

### T-J154-010 — Campaign launch happy path

**Pre-conditions:** T-J154-001..T-J154-008 done. Both CMOs in approval chain.

**Action sequence:**

1. Advance clock to `2027-01-12T08:59:00+01:00`
2. POST `/v1/campaigns/camp-glacpl-mfg-q1-2027/launch` with full body from handshake §4.2
3. Advance clock to `2027-01-12T09:00:00+01:00`

**Expected events:**

- `EVT-J154-CAMPAIGN-LAUNCH-011` at exactly T0
- 13,000 individual email-dispatch events within the first hour (8,000 DE + 5,000 NL)
- LinkedIn + Display campaigns transition to `active`

**Pass criteria:**

- First 1,000 emails dispatched within 22s (p95)
- DKIM signatures present on every outbound email
- Bounce rate over first 4 hours: < 3%
- Complaint rate over first 4 hours: < 0.1%
- p95 deliverability ≥ 97% per sender domain
- All events sealed in the shared tenant; mirror events sealed in PartnerLift + Glacier when those tenants are referenced

### T-J154-011 — Campaign launch refused without DPA

**Pre-conditions:** T-J154-001 + T-J154-003 done (DPA failed). T-J154-004..T-J154-008 done.

**Action sequence:** same as T-J154-010.

**Expected events:**

- `EVT-J154-CAMPAIGN-LAUNCH-MISSING-DPA-NNN`

**Pass criteria:**

- Launch returns `409 Conflict`
- Response body names the missing DPA attestation
- Campaign remains `armed_for_scheduled_launch` but blocked

### T-J154-012 — DSA transparency log writes

**Pre-conditions:** T-J154-010 done. LinkedIn + Display mocks emit impression-class events.

**Action sequence:**

1. LinkedIn mock emits 5,000 impression events over 1 hour
2. Google Display mock emits 12,000 impression events over 1 hour

**Expected events:**

- `EVT-J154-COMPLIANCE-DSA-IMPRESSION-LOG-BATCH-NNN` (batched per 100 impressions)

**Pass criteria:**

- 170 batch events sealed in total
- Each event includes `advertiser_legal_name` listing BOTH PartnerLift B.V. AND Glacier ERP GmbH (joint controllers)
- Each event includes a human-readable `targeting_criteria_plain` per DSA Art 26
- All sealed in the shared tenant + replicated to both partner tenants

### T-J154-013 — Spam-trap detection auto-suppress

**Pre-conditions:** T-J154-010 done. Spam-trap address `procurement.team@spamhaus-trap-001-de` is in the German send list.

**Action sequence:**

1. Advance clock to `2027-01-13T09:14:00+01:00`
2. Email-dispatch hits the address; spam-trap detection model flags

**Expected events:**

- `EVT-J154-COMMS-EMAIL-SPAM-TRAP-CAUGHT-013`

**Pass criteria:**

- Address auto-suppressed (no further sends)
- Henrik notified via the community channel
- Sender-domain reputation budget unchanged (no penalty applied for single trap caught)
- A "review source list" follow-up task created in Henrik's queue

### T-J154-014 — Q1 attribution settlement happy path

**Pre-conditions:** T-J154-010 done. Workflow timer armed. Q1 simulated converted-lead set loaded:
- 482 Glacier-sourced converted
- 311 PartnerLift-sourced converted
- 184 joint-pool converted
- Total Q1 ARR signed: `€4,232,118`

**Action sequence:**

1. Advance clock to `2027-03-31T23:59:59+01:00`
2. Workflow-engine fires the `q1_attribution_settlement` timer
3. Payments computes + initiates SEPA transfers

**Expected events:**

- `EVT-J154-PAYMENTS-ATTRIBUTION-SETTLEMENT-014`
- Two SEPA-credit-transfer-init events (one per recipient tenant)

**Pass criteria:**

- Disbursement matches formula deterministically:
  - Glacier receives € 60,418.00
  - PartnerLift receives € 119,582.00
- Total disbursed = € 180,000.00 (escrow balance == 0)
- SEPA pacs.008 messages valid against the schema, with `BICFI` and `IBAN` of both parties populated
- Audit event sealed in all 3 tenants
- p95 of `timer fire → SEPA initiation` ≤ 14m

**Fail criteria:**

- Disbursement does not sum to € 180,000.00
- Either party receives 0 (would indicate divide-by-zero or null in attribution math)
- Audit event missing from any tenant

### T-J154-015 — Cedar denies premature escrow release

**Pre-conditions:** T-J154-010 done. Attempt manual release on Mar 30.

**Action sequence:**

1. Advance clock to `2027-03-30T16:00:00+01:00`
2. POST `/v1/escrows/{id}/release` from `tomas.pieter@partnerlift.nl` (premature)

**Expected events:**

- `EVT-J154-PAYMENTS-DENY-PREMATURE-RELEASE-NNN`

**Pass criteria:**

- Cedar returns `deny` (timer not yet fired)
- Response names the next-eligible-release timestamp
- Escrow balance unchanged

### T-J154-016 — Apr 1 wind-down refuses writes

**Pre-conditions:** T-J154-014 done.

**Action sequence:**

1. Advance clock to `2027-04-01T00:00:01+02:00`
2. Attempt POST `/v1/leads` on the shared tenant

**Expected events:**

- `EVT-J154-TENANCY-SHARED-WIND-DOWN-015`
- `EVT-J154-TENANCY-DENY-WRITE-DURING-WINDDOWN-NNN`

**Pass criteria:**

- Tenant state = `winding_down`
- Write returns `423 Locked`
- Reads still succeed (90-day audit window)
- Cedar context `tenant_state == winding_down` evaluates correctly

### T-J154-017 — Jul 1 archive

**Pre-conditions:** T-J154-016 done.

**Action sequence:**

1. Advance clock to `2027-07-01T00:00:01+02:00`

**Expected events:**

- `EVT-J154-TENANCY-SHARED-ARCHIVED-016`

**Pass criteria:**

- Tenant state = `archived`
- Reads return from cold storage (latency p95 ≤ 4s)
- Data retained for 7 years per PartnerLift policy
- All 3 tenants' audit trails still hold their replicated DPA + audit events

### T-J154-018 — Cross-partner asset preview denied

**Pre-conditions:** Trinity active. Tomas attempts to preview Glacier-internal-only marketing asset (not joint).

**Action sequence:**

1. GET `/v1/marketing-assets/{glacier-internal-asset-id}/preview` from `tomas.pieter@partnerlift.nl`

**Expected events:**

- `EVT-J154-CEDAR-DENY-CROSS-PARTNER-ASSET-PREVIEW-NNN`

**Pass criteria:**

- Cedar denies with reason `asset.owner_tenant != principal.active_tenant && asset.shared_visibility != joint_controller_visible`
- Response includes link to the shared marketing-asset gallery (the recovery path)

### T-J154-019 — Joint analytics denied to non-controller

**Pre-conditions:** A PartnerLift employee NOT in the campaign coord channel (e.g., `random.intern@partnerlift.nl`) attempts to view joint analytics.

**Action sequence:**

1. GET `/v1/analytics/camp-glacpl-mfg-q1-2027` from the intern principal

**Expected events:**

- `EVT-J154-CEDAR-DENY-ANALYTICS-NON-CONTROLLER-NNN`

**Pass criteria:**

- Cedar denies because `principal.role_in_tenant("glacier-partnerlift-q1-2027-mfg-de-nl-be")` is unset
- No row counts or campaign metadata leaked in the denial response
- Intern sees a tenant-context picker, not the analytics surface

### T-J154-020 — End-to-end trinity audit verifiability

**Pre-conditions:** All prior tests completed.

**Action sequence:**

1. Compute merkle root of partnerlift_nl audit chain on `2027-04-02T00:00:00+02:00`
2. Compute merkle root of glacier_erp_de audit chain on same day
3. Compute merkle root of shared tenant audit chain on same day
4. Run cross-tenant verifier on each replicated event

**Pass criteria:**

- For every event with `trinity_replication_tenants` listing multiple tenants, the verifier finds matching sealed entries in each listed tenant
- Daily merkle roots are independently verifiable from the public-key-pinned read endpoint
- No replicated event has tenant-divergent payloads (per-tenant `payload` blobs are byte-identical for the same event class)

**Fail criteria:**

- Any replicated event missing from one of its expected tenants
- Tenant-divergent payloads (would indicate per-tenant tampering)

## Coverage map

| Acceptance criterion (README) | Test(s) |
|---|---|
| AC-J154-001 shared tenant provisioned | T-J154-001 |
| AC-J154-002 tri-party DPA stored | T-J154-002, T-J154-003 |
| AC-J154-003 DKIM/SPF/DMARC verified | T-J154-004, T-J154-005 |
| AC-J154-004 bilingual sequences live | T-J154-010 |
| AC-J154-005 Display + DSA log writes | T-J154-012 |
| AC-J154-006 CRM lead routing | T-J154-007, T-J154-008 |
| AC-J154-007 partner-only community channel | (covered indirectly by T-J154-019 access controls) |
| AC-J154-008 launch button + first-1000 SLO | T-J154-010, T-J154-011 |
| AC-J154-009 Cedar denies cross-CRM read | T-J154-006 |
| AC-J154-010 Q1 settlement | T-J154-014, T-J154-015 |

## Anti-flake controls

1. All wire mocks return deterministic responses keyed on the test seed. No real internet egress.
2. Clock is frozen; advance is explicit per test step. Spam-trap timing in T-J154-013 uses an injected clock-advance.
3. DNS resolver mock returns deterministic DKIM/SPF/DMARC TXT records keyed on the seed zones.
4. eIDAS QES validator mock keys on the seed PDF's SHA-256 + signer cert chain root; never makes a network call.
5. SEPA pacs.008 messages are validated against the canonical `pacs.008.001.08` XSD shipped in `tests/fixtures/iso20022/`. Failures show the XSD error line.
6. Audit-chain merkle roots are deterministic given the seed event set; the test asserts byte-exact root hashes per day.

## Performance gates (CI-enforced)

| SLO | Target |
|---|---|
| trinity-provision co-sign → tenant active (p95) | ≤ 680ms |
| DPA upload → 3 signatures verified (p95) | ≤ 1.2s |
| campaign.launch → first 1000 emails (p95) | ≤ 22s |
| lead.created → both CRMs routed (p95) | ≤ 420ms |
| Cedar deny on cross-CRM read (p95) | ≤ 180ms |
| Q1 settlement timer fire → SEPA initiated (p95) | ≤ 14m |

Failures push the build to red; the build won't merge into `dev` until SLOs hold.
