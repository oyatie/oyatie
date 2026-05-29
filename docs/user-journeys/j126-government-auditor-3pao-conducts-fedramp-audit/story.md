---
doc_class: User-Journey-Story
journey_id: j126-government-auditor-3pao-conducts-fedramp-audit
journey_slug: j126-government-auditor-3pao-conducts-fedramp-audit
status: draft
date: 2026-05-20
authority_tier: 3
audience: [council-product, council-architecture, council-security, council-legal, council-compliance, axis-identity, axis-audit-chain, axis-compliance]
related_adrs:
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0312-court-warrant-scoped-piercing
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0243-cedar-as-universal-gate
  - ADR-0242-oyatie-is-a-tenant-doctrine
  - ADR-0245-substrate-vs-product-layering
  - ADR-0247-self-modification-doctrine
  - ADR-0248-amazon-shape-cellular-architecture
  - ADR-0251-compliance-pack-cell-certification-levels
  - ADR-0263-observability-emission-contract
  - ADR-0028-audit-chain-merkle-sealed
  - ADR-0188-passkey-webauthn-as-canonical-auth
  - ADR-0299-account-recovery-resilience
related_specs:
  - /specs/microservices/identity.json
  - /specs/microservices/tenancy.json
  - /specs/microservices/audit-chain.json
  - /specs/microservices/compliance.json
  - /specs/microservices/ops-dashboard-control-center.json
  - /specs/microservices/observability.json
related_packs:
  - packs/global-fedramp-mod
  - packs/global-fedramp-high
  - packs/us-nist-sp-800-53-rev5
  - packs/us-omb-circular-a-130
  - packs/us-fisma-2014
critical_path_rows:
  - documentation-rigor.md §3.2.5 row 18 (Audit / regulator / law-enforcement access)
  - documentation-rigor.md §3.2.5 row 23 (Cross-jurisdiction conflict — partial; cross-link j131)
anchor_archetype: diana-reyes-47-washington-dc
locale: en-US
regulatory_anchors:
  - FedRAMP Moderate and High baselines (rev. 2024-10)
  - NIST SP 800-53 Rev 5 (especially AU-2, AU-12, AC-3, IA-2)
  - OMB Circular A-130 (Managing Information as a Strategic Resource)
  - FISMA 2014 (44 USC §3554)
  - 5 USC §552 (FOIA) — non-applicable read scope
purpose: >
  Narrate ONE concrete human's experience exercising the dual-tenant identity
  boundary: Inspector Diana Reyes, a US GAO auditor doubling as a registered
  FedRAMP 3PAO, conducts a FedRAMP Moderate annual ConMon audit of Marcus
  Chen's federal-contractor tenant. The bypass-class invariant is that
  Diana's PERSONAL tenant — her family Messenger DMs, her tax workflow,
  her vintage-record-collecting Marketplace — is structurally invisible
  to her audit role and to her agency. Two tenants, one passkey, Cedar
  default-deny holding the line. If any seam in this story leaks her
  family DMs to her agency, ADR-0311 is broken and the platform's claim
  of dual-tenant identity is false. Every line of code we ship must
  preserve this boundary.
---

# j126 — Diana Reyes runs a FedRAMP audit while her family group chat stays sealed

> **Purpose.** This is not a hypothetical. This is the story of Inspector
> Diana Reyes, 47, a Senior Auditor at the US Government Accountability
> Office (GAO) and a registered FedRAMP 3PAO under accreditation
> 3PAO-2023-0147, at 09:14 EST on Monday 2026-05-26, when she opens her
> agency-issued ThinkPad to begin the annual ConMon audit pull for Chen
> Aerospace Manufacturing, a 5,000-employee federal contractor whose
> Marcus Chen is CEO. Every µservice in the oyatie fabric that touches
> identity, tenancy, audit, compliance, or observability will be
> exercised in the next forty-three minutes. The story is concrete
> because the contract is concrete: dual-tenant identity is the
> *highest-frequency* critical-path scenario for a B2B-Tenant + Auditor
> ecosystem, and ADR-0311 makes the boundary architecturally mandatory.
> If any seam in this story leaks Diana's personal family-chat to her
> agency tenant — or, equally bad, denies her legitimate cross-tenant
> audit access to Marcus's tenant — ADR-0311 is broken.

## 1. Diana's continuity of identity — one human, two tenants, Cedar default-deny in between

Diana Reyes is **not two users**. She is one human across two tenants
that oyatie distinguishes by **tenant_id membership** (ADR-0244) and
**Cedar permit graph** (ADR-0243), not by fragmenting her identity. Her
passkey, enrolled to her FIDO2 hardware-key in 2025-08-12 (a YubiKey 5C
NFC issued by GAO IT), authenticates her to BOTH tenants per
ADR-0188 + ADR-0299.

| Context | Tenant | Principal | Cell tier | Pack overlay | Audience type |
|---|---|---|---|---|---|
| **Work — US GAO auditor + FedRAMP 3PAO** | `gao.audit.fedramp-3pao` (reserved-namespace; bound to GAO federal-employee SSO via PIV/CAC) | `diana.reyes@gao.gov` (work passkey + PIV smart-card 2nd factor) | Tier-3 (FedRAMP Moderate cell — `us-gov-east-1`) | `pack-us-fedramp-mod + pack-us-nist-sp-800-53-rev5 + pack-us-omb-a-130 + pack-us-fisma-2014` | `INTERNAL_AUDITOR_3PAO` |
| **Personal — consumer + family + vintage records hobby** | `diana-reyes-personal-92381` (her personal tenant, opened 2024-03 at oyatie consumer signup; family-mode enabled 2025-01 when she added her wife Jennifer) | `diana@diana-reyes.me` (personal passkey on same YubiKey) | Tier-2 (US-east-1 consumer general-purpose cell) | `pack-us-ccpa-2023 + pack-us-coppa-1998 (children sub-pack, son-aged-9) + pack-us-state-va-cdpa-2023` | `B2C_CONSUMER` |

Two distinct `tenant_id` values. Same human. Same passkey identity
(per ADR-0299 §account-recovery + ADR-0188 §passkey-cross-tenant). The
oyatie identity µservice's `multi_context_principal_resolver` (per
identity/IP-017) detects the same `webauthn_credential_id` enrolled to
both `tenant_id`s and presents the user with a **context picker** at
session-init.

The context-picker is a UI artifact, but the Cedar permit graph is the
*enforcement*. If Diana selects "Work — GAO" at the picker, her
session-scope is set to `tenant_id = gao.audit.fedramp-3pao` and EVERY
downstream µservice's Cedar policy evaluates her actions against
`gao.audit.fedramp-3pao` permits. There is no permit named
`gao-auditor.read_personal_tenant_messenger` — and Cedar's default-deny
baseline (per ADR-0243 §B) holds. Her family group chat is not
"hidden" from her work session by UI; it is **inaccessible** at the
storage layer because the work session's principal does not present any
Cedar permit that resolves to her personal tenant's resources.

This is the **dual-tenant identity boundary doctrine** from the
CATALOG-j126-j150-ecosystem.md catalog, made concrete:

- Same human bridges both tenants via shared passkey identity.
- Each tenant has its own Cedar permit graph.
- No cross-tenant permit exists by default; the platform has zero
  "auditor can read auditor's own personal data via their own auditor
  role" path.
- If Diana wants to read her own family chat, she switches the context
  picker to "Personal — Diana", and her session re-roots its
  `tenant_id` claim. The work-session permits no longer apply.

The hyperscaler precedent is the SEC enforcement-attorney pattern: at
the SEC, the case-management system (Tyler Technologies' Federal Case
Management System, or the equivalent at FINRA) deliberately separates
case-context from employee-personal-context so an attorney cannot mix
discovery materials with personal records even if a court subpoena
seeks both. oyatie's distinction is enforced by Cedar, not by case-by-
case workflow.

## 2. The forty-five minutes before — 08:30 EST, Monday, 2026-05-26

Diana wakes at 06:00 EST. She makes coffee, takes her son Marcus to
the school bus (her son is named Marcus too — a recurring source of
inside-the-family confusion she finds funny). At 07:45 she sits at the
kitchen counter and opens her personal iPhone 16 Pro to check her
**personal** oyatie surfaces:

- **Messenger** — "Reyes Family" group chat (her wife Jennifer, her
  sister Beatriz in San Antonio, her parents in Florida): 14 unread
  messages. Her mother is sending the usual Sunday-night photo dump
  from her Florida garden, plus a question about Easter travel dates.
- **Mail** — three personal: a Stripe receipt from the vintage-jazz
  record shop she bought a 1958 Mingus pressing from on Friday; a
  Patagonia order shipping confirmation; a Hill Country Camping
  reservation reminder for the Memorial Day weekend.
- **Calendar** — a personal event at 18:00 EST: "wife's gallery
  opening — Eastern Market".
- **Notes** — a draft from yesterday: "ConMon prep — pull NIST 800-53
  AU-12 evidence schema; brief the team at 09:00".
- **Workflow Studio** — a personal automation that runs every Sunday
  night: "Reconcile Stripe consumer → personal accountant
  Drive folder" (she runs a tiny side practice of bookkeeping for two
  freelance friends; she keeps it strictly outside her GAO role per
  her ethics agreement).

Diana scrolls her family chat for two minutes, sends her mother a
heart emoji and a sentence about Easter, then closes Messenger. Her
personal-tenant session is now backgrounded but still active on her
device. Cedar permits scoped to `tenant_id = diana-reyes-personal-92381`
are still on the device's session-store with a TTL of 8 hours; she has
not signed out.

She switches devices. Her **work** ThinkPad is in her home office. She
boots it. The PIV smart-card prompt appears at the BIOS-level Tier-3
attestation gate (her ThinkPad is a FIPS-140-3 Level-2 certified device
managed by GAO IT's Intune-equivalent fleet management). She inserts
her PIV card, types the GAO PIN, and the device unlocks.

She opens her oyatie work-tenant browser session. The context picker
fires immediately:

```
┌────────────────────────────────────────────────┐
│  Welcome back, Diana                            │
│  Two oyatie tenants detected on this credential.│
│  Which would you like to work in?               │
│                                                 │
│  ◉ Work — US GAO (FedRAMP 3PAO)                 │
│      tenant: gao.audit.fedramp-3pao             │
│      cell: us-gov-east-1 (FedRAMP Mod)          │
│      pack: pack-us-fedramp-mod + 3 more         │
│                                                 │
│  ○ Personal — Diana                             │
│      tenant: diana-reyes-personal-92381         │
│      cell: us-east-1 (consumer)                 │
│      pack: pack-us-ccpa + 2 more                │
│                                                 │
│  [Continue]   [Cancel]                          │
└────────────────────────────────────────────────┘
```

She selects "Work — US GAO" and clicks Continue. The session is
established as:

```
session.tenant_id = "gao.audit.fedramp-3pao"
session.principal_id = "diana.reyes@gao.gov"
session.audience_type = "INTERNAL_AUDITOR_3PAO"
session.cell_id = "us-gov-east-1"
session.packs_active = ["pack-us-fedramp-mod", "pack-us-nist-sp-800-53-rev5",
                       "pack-us-omb-a-130", "pack-us-fisma-2014"]
session.authentication_method = "passkey + piv-cac"
session.expires_at = 2026-05-26T17:30:00-04:00  (FedRAMP-Mod 8h work-hours cap)
```

The personal-tenant session on her phone is **untouched**. Cedar permits
for `diana-reyes-personal-92381` continue to exist on her phone's session
store. Her work-tenant session on the ThinkPad has zero overlap with
those permits.

## 3. T+00:00 — 09:14 EST — The audit pull begins

Diana navigates to the GAO's ops-dashboard-control-center
(`gao.audit.fedramp-3pao` view) and opens the "FedRAMP ConMon Annual
Audit" workflow. The dashboard shows her assigned 3PAO docket:

```
DOCKET: 3PAO-2026-MAY-CHEN-AERO-001
CSP under audit: Chen Aerospace Manufacturing
CSP tenant_id: chen-aerospace.federal-contractor.us
FedRAMP baseline: Moderate
Audit class: Annual ConMon (Continuous Monitoring)
Period under review: 2025-05-01 → 2026-04-30
Lead 3PAO: Diana Reyes (diana.reyes@gao.gov)
3PAO accreditation: 3PAO-2023-0147
Authorizing official: Patricia Wallace, OMB
```

She clicks "Begin evidence pull". The next forty-three minutes unfold
across the oyatie substrate.

### 3.1 Cross-tenant Cedar permit request — what makes this legal

The KEY architectural fact: Diana's `gao.audit.fedramp-3pao` tenant
does NOT have default access to `chen-aerospace.federal-contractor.us`
tenant data. Marcus Chen's tenant is a separate entity in the oyatie
identity graph with its own Cedar policy graph.

What enables Diana's audit pull is a **cross-tenant Cedar permit**
issued by the FedRAMP authorizing official (OMB) and accepted by Marcus
Chen's tenant when Chen Aerospace entered FedRAMP authorization (a
contractual obligation, codified in the tenant's CSP agreement
referenced by `pack-us-fedramp-mod`).

The Cedar fragment that authorizes Diana to read Marcus's audit
evidence is:

```cedar
// In Marcus's tenant: chen-aerospace.federal-contractor.us
// Fragment: cross-tenant-fedramp-3pao-audit-evidence.cedar
permit (
  principal in Tenant::"gao.audit.fedramp-3pao",
  action in [
    Action::"audit_chain.read_sealed_evidence",
    Action::"compliance.read_control_evidence",
    Action::"observability.read_metric_export",
    Action::"identity.read_principal_roster",
    Action::"tenancy.read_compliance_pack_roster",
    Action::"ops_dashboard.read_control_status"
  ],
  resource in Tenant::"chen-aerospace.federal-contractor.us"
) when {
  principal.audience_type == "INTERNAL_AUDITOR_3PAO" &&
  principal.fedramp_3pao_accreditation_active == true &&
  resource.compliance_packs.contains("pack-us-fedramp-mod") &&
  context.audit_docket_id matches "3PAO-2026-MAY-CHEN-AERO-*" &&
  context.audit_period_start >= datetime("2025-05-01T00:00:00Z") &&
  context.audit_period_end <= datetime("2026-04-30T23:59:59Z")
};
```

Several invariants are encoded:

- The permit is **principal-scoped to a tenant**, not to a user. Any
  active 3PAO auditor inside `gao.audit.fedramp-3pao` could in
  principle exercise it — but each must present `audience_type =
  INTERNAL_AUDITOR_3PAO` (a Cedar attribute that is set at session-init
  only when the user's GAO principal is also accredited as a 3PAO in
  GAO's identity µservice).
- The permit is **resource-scoped to a tenant**, not to specific rows.
  Diana cannot use this permit to pull, say, Marcus's vendor invoices
  (those are `payments` µservice resources, NOT `audit_chain` resources,
  and the action enum does not include them). She gets the audit
  evidence Marcus's tenant must by FedRAMP regulation produce — not the
  underlying business data unless an explicit, broader subpoena issues.
- The permit is **time-bounded** to the audit period. After 2026-04-30,
  Diana cannot use this permit to read any new evidence; she would need
  a fresh docket.
- The permit is **soak-windowed** per ADR-0294 (Cedar fragment soak):
  any change to it must respect the ≥60s soak before being usable.

Critically, this permit does **NOT** grant Diana any access to her own
personal tenant. The `principal in Tenant::"gao.audit.fedramp-3pao"`
clause ensures only her work-principal can exercise it. Her personal
principal is in a different tenant and would fail the principal-binding.

### 3.2 The actual evidence pull — what flows

Diana's dashboard click triggers the following sequence (compressed —
full sequence in `handshake.md`):

1. The ops-dashboard-control-center µservice constructs an audit-pull
   request and submits it to `api-gateway` as a workflow trigger.
2. `api-gateway` invokes `policy-engine` library-mode (per ADR-0246
   amendment) to evaluate the cross-tenant Cedar permit. Permit
   evaluates `Allow` (Diana is INTERNAL_AUDITOR_3PAO, accreditation
   active, audit period within bounds).
3. `workflow-engine` orchestrates the multi-µservice evidence pull
   (audit-chain + compliance + observability + identity + tenancy).
4. Each downstream µservice's library-mode policy-engine again
   independently evaluates the permit (defense-in-depth per ADR-0246
   amendment §re-evaluation).
5. Each downstream µservice emits the requested evidence into a
   tamper-evident audit-evidence bundle, sealed by `audit-chain` per
   ADR-0028 Merkle-sealing.
6. `observability` emits `CrossTenantAuditEvidencePulled` audit events
   (registered in ADR-0263 §D-N) — both to Diana's GAO tenant's audit
   log AND to Marcus's Chen-Aerospace tenant's audit log. **Both
   parties have provenance.**
7. The evidence bundle is delivered to Diana's ops-dashboard as a
   sealed envelope; she can browse the contents, but each browse
   emission emits a fresh `CrossTenantAuditEvidenceAccessed` event.
8. Marcus's tenant-admin Cedar permit graph has a **mandatory
   notification** wired: every cross-tenant audit-evidence pull emits
   a tenant-admin notification to `marcus.chen@chen-aerospace.us`
   within 15 minutes (per FedRAMP transparency obligations and ADR-0311
   §B-7 cross-tenant transparency invariant).

## 4. T+00:23 — 09:37 EST — Diana opens the first evidence pack

The audit-chain µservice has delivered a sealed bundle to Diana's
dashboard. She opens it. The contents include:

- **AU-2 (Auditable Events) — evidence:** 47 audit-event-class
  emissions sampled from Marcus's tenant over the audit period, with
  Merkle-chain proofs. She can verify any one against the chain root.
- **AU-12 (Audit Generation) — evidence:** per-µservice emission
  configuration manifests (which audit events each µservice in
  Marcus's tenant emits, with cardinality budgets). 41 µservices, each
  attesting full ADR-0263 compliance.
- **AC-3 (Access Enforcement) — evidence:** the Cedar permit graph
  (export-only-of-permit-shapes, not of who-used-them) for Marcus's
  tenant. Diana sees that no permit grants cross-tenant access except
  to her own GAO tenant for the audit.
- **IA-2 (Identification and Authentication) — evidence:** WebAuthn
  enrollment counts, hardware-key attestation summaries, PIV/CAC
  uptake within Marcus's federal-contractor employees.
- **CM-3 (Configuration Change Control) — evidence:** the per-µservice
  CHANGELOG.md attestation hashes + Foundry pipeline merge-queue
  receipts.

She begins her review. Around 09:42 EST her **personal** phone
buzzes. The lock-screen shows: "Reyes Family • Mom: 'OK we'll come
Easter Sat-Mon, can Jenn pick us up at DCA?'"

She glances at her phone, picks it up, taps the notification. Her
**personal-tenant** session is still active. The Messenger app opens
to the family chat. She types: "yes she'll be there. flight number?"
and sends. Then she puts the phone down face-up next to her ThinkPad
and returns to the audit.

The KEY architectural moment: nothing about her interaction with the
family chat appears anywhere in the GAO tenant. The Messenger µservice's
emission to audit-chain is **scoped to her personal tenant**, lands in
her **personal-tenant audit log**, and is not visible from her
work-session ops-dashboard. There is no permit by which her work
session could read it. The Cedar permit graph **does not contain such
a permit**.

This is the load-bearing invariant of ADR-0311. The boundary holds.

## 5. T+00:31 — 09:45 EST — Diana finds an anomaly

In the AU-2 evidence, Diana notices that Marcus's tenant's `audit_chain`
emission cardinality for `PaymentRiskScoreEmitted` is **higher** than
expected for a federal-contractor tenant whose Stripe surface
is mostly inbound vendor invoices. She opens the per-µservice drill-
down.

The cardinality is `847,231` emissions over the audit period. Marcus's
tenant has 5,000 employees. That's ~170 emissions per employee per
year — but `PaymentRiskScoreEmitted` is a **fraud-related** event
emitted on consumer-facing transactions, not on B2B vendor invoicing.
For a federal-contractor tenant, this should be near-zero unless they
have a consumer-facing surface.

Diana files an audit finding in the dashboard:

```
FINDING: 3PAO-2026-MAY-CHEN-AERO-001-F012
Control: AU-2 (Auditable Events)
Severity: APPROVE_WITH_FINDINGS
Description: PaymentRiskScoreEmitted cardinality (847,231) over
audit period inconsistent with declared B2B-only Stripe Connect
surface. Request CSP explanation of consumer-facing payment surface
or correction of event-class emission scope.
Required CSP response: 30 days per FedRAMP ConMon SOP.
```

She submits the finding. The `workflow-engine` routes it back to
Marcus's tenant — specifically to Marcus's CISO's queue, with a
notification to Marcus's tenant-admin. Marcus's tenant has 30 days to
respond.

## 6. T+00:40 — 09:54 EST — Diana wraps the morning session

Diana saves her draft findings, exports the evidence bundle locally to
her ThinkPad's encrypted drive (FedRAMP Mod-eligible cell encryption-key BYOK key, ADR-0251 §D-10),
and closes the dashboard. She has fifteen minutes before her
10:00 EST team standup.

She picks up her phone. She wants to confirm her flight pickup with
Jennifer. She opens Messenger — her personal-tenant session is still
active. She sends: "Mom wants you to grab her at DCA Easter Sat 3pm
flight tbd". Jennifer responds within seconds: "ok lmk the flight no
when she has it 💜".

Diana smiles. She also has a brief thought: it would be VERY useful if
she could pull her own personal-tenant Messenger archive sometime to
show her therapist some patterns her mother's messages exhibit (an
inside-the-family thing she's working through). She makes a mental
note to do this from her personal-tenant Workflow Studio over the
weekend.

She does NOT — could not, even if she wanted to — pull that archive
from her work session. The architecture forbids it. This is the right
architecture.

## 7. T+00:48 — 10:02 EST — Diana joins the team standup

She joins her GAO ConMon team standup in oyatie Meet. She shares her
finding-F012, asks her teammate Aliyah for a second pair of eyes, and
the team agrees Diana should also pull additional evidence on Marcus's
consumer-facing surface to disambiguate.

Diana adds a follow-up audit task in the workflow-engine. The task is
visible to her GAO team. It is invisible to her personal tenant.

The team standup ends at 10:30 EST. Diana goes for a coffee break.

## 8. The architectural diff — what would have to be true for this to break

For Diana's family chat to leak to GAO, ONE of the following would
have had to be true:

1. **Identity µservice would have had to merge tenant memberships.**
   This is forbidden by ADR-0244 §B-3: tenant memberships are
   independent rows; a user can be a principal in many tenants, but
   no surface aggregates rows across tenants automatically.
2. **Cedar would have had to default-permit cross-tenant.** This is
   forbidden by ADR-0243 §B (default-deny baseline). Every permit is
   explicit.
3. **A cross-tenant permit named something like
   `agency.read_employees_personal_data` would have to exist.** It
   does not — and §3.2.6.D prevention invariant 1 mandates no
   single-point-of-failure permit ever grants cross-tenant access
   without policy authority.
4. **Marcus Chen's tenant Cedar fragment would have had to mistakenly
   permit Diana to read consumer-tenant resources.** Cross-tenant
   permits are scoped by both principal-tenant AND resource-tenant; a
   permit in Marcus's fragment can grant access to Marcus's resources,
   not Diana's personal tenant's resources.
5. **The session context-picker would have had to fail silently.**
   The picker is a defense-in-depth UX surface, but Cedar enforcement
   is the architectural floor — even with a picker bug, default-deny
   holds.

Five independent invariants. Any one of them holding alone is
sufficient. Defense-in-depth means we have all five.

## 9. The architectural diff — what would have to be true for the AUDIT to break

For Diana's legitimate audit pull to fail, ONE of the following would
have had to be true:

1. **The cross-tenant Cedar permit was not issued.** Mitigated by the
   FedRAMP onboarding workflow which mandates this fragment as part
   of CSP authorization.
2. **Diana's INTERNAL_AUDITOR_3PAO attribute was missing.** Mitigated
   by identity µservice's accreditation tracking; only accredited
   3PAOs are tagged.
3. **The audit-evidence emission was not seal-able.** Mitigated by
   ADR-0028 audit-chain Merkle-seal substrate.
4. **Marcus's tenant did not emit the audit events in the first place.**
   Mitigated by ADR-0263 ObservabilityEmissionContract — non-emission
   is itself a control failure flaggable in audit.
5. **The Foundry pipeline merge-queue receipts were unavailable.**
   Mitigated by ADR-0111 projected merge state + ADR-0113 VCS
   orchestrator end-to-end.

Five independent invariants. The audit holds because they all hold.

## 10. The audit-event trail — what was emitted

Across the forty-three minutes, the following audit events were
emitted to BOTH tenants' audit logs (per ADR-0311 §B-9 dual-tenant
transparency for cross-tenant operations):

### To Diana's GAO tenant audit log (sealed by audit-chain)

| T+ms | Audit class | Resource | Details |
|---:|---|---|---|
| 0 | `AuditDocketOpened` | docket://3PAO-2026-MAY-CHEN-AERO-001 | Diana opened the docket |
| 12000 | `CrossTenantPermitEvaluatedAllow` | tenant://chen-aerospace.federal-contractor.us | Permit evaluation succeeded |
| 13000 | `CrossTenantAuditEvidencePulled` | bundle://3PAO-2026-MAY-CHEN-AERO-001/AU-2 | First evidence pack pulled |
| 41000 | `AuditFindingFiled` | finding://F012 | Diana filed AU-2 cardinality finding |
| 41500 | `CrossTenantNotificationDispatched` | tenant://chen-aerospace.federal-contractor.us | Marcus's tenant notified |

### To Marcus's Chen-Aerospace tenant audit log (sealed by audit-chain)

| T+ms | Audit class | Resource | Details |
|---:|---|---|---|
| 12000 | `CrossTenantPermitExercised` | principal://diana.reyes@gao.gov | Diana's tenant exercised the permit |
| 13000 | `CrossTenantAuditEvidenceExported` | bundle://3PAO-2026-MAY-CHEN-AERO-001/AU-2 | Evidence was exported under permit |
| 13500 | `TenantAdminNotificationDispatched` | admin://marcus.chen@chen-aerospace.us | Marcus was notified |
| 41500 | `AuditFindingReceived` | finding://F012 | Finding routed to CISO queue |

### To Diana's PERSONAL tenant audit log (sealed by audit-chain)

| T+ms | Audit class | Resource | Details |
|---:|---|---|---|
| 28000 | `MessengerMessageRead` | thread://reyes-family | Diana read mom's message |
| 28100 | `MessengerMessageSent` | thread://reyes-family | Diana replied about Easter |
| 46000 | `MessengerMessageSent` | thread://reyes-family | Diana confirmed flight pickup |

**Crucially**: Diana's personal-tenant audit log is sealed in the
audit-chain substrate of her personal tenant (not GAO's audit-chain
substrate). The two chains have no cross-references. The only common
denominator is the audit-chain µservice itself — which serves both
tenants, but seals each tenant's chain independently per ADR-0028
§D-tenant-isolation.

## 11. The cell-tier story — why two cells matter

Diana's work tenant lives in cell `us-gov-east-1` (FedRAMP Mod-eligible,
Tier-3, hosted in a GovCloud-equivalent region with FIPS-140-3 KMS).
Marcus's tenant lives in cell `us-east-1-fedramp` (FedRAMP Mod-eligible,
Tier-3, regular us-east-1 region with FedRAMP-Mod sub-cell isolation).
Diana's personal tenant lives in cell `us-east-1` (Tier-2, consumer
general-purpose).

The cross-tenant evidence pull crosses the cell boundary from
`us-gov-east-1` ↔ `us-east-1-fedramp`. This is a Tier-3 ↔ Tier-3
crossing, which is permitted by ADR-0248 §D-3 cell-shuffle-sharding
when both endpoints are FedRAMP Mod-eligible and the egress contract
is mTLS-attested per ADR-0254.

The personal tenant in `us-east-1` is in a SEPARATE cell from her
work tenant. No mTLS path exists from `us-gov-east-1` to consumer
`us-east-1` for tenant data egress. The cell boundary is itself a
defense-in-depth layer per §3.2.6.D L1 (Network).

If Diana had logged into her personal tenant from her work ThinkPad
(possible — passkey works on the device), she would have been
prompted by Cedar: "Cross-cell access from Tier-3 cell to Tier-2 cell;
proceed?". She did not; she used her phone for personal.

## 12. The pack-overlay story — three regulatory regimes interplay

Diana's work tenant has:
- `pack-us-fedramp-mod` — defines control families AU, AC, IA, CM that
  the audit pulls evidence against.
- `pack-us-nist-sp-800-53-rev5` — supplies the control definitions
  themselves.
- `pack-us-omb-a-130` — supplies the strategic-asset framing for the
  audit.
- `pack-us-fisma-2014` — supplies the legal authority for the audit.

Marcus's tenant has:
- `pack-us-fedramp-mod` — same as Diana's, ensures auditability.
- `pack-pci-dss-v4` — applies to Marcus's Stripe payments
  surface.
- `pack-us-itar-2024` — applies because Chen Aerospace is a defense
  contractor.

Diana's personal tenant has:
- `pack-us-ccpa-2023` — California consumer-privacy (Diana's wife is
  a California resident and the family pack inherits broadest-applicable).
- `pack-us-coppa-1998` — her son is 9, the family pack activates child
  protection.
- `pack-us-state-va-cdpa-2023` — Diana is a Virginia resident.

**No pack overlaps grant cross-tenant access.** Each tenant's pack
set is independent. The Cedar permit graph respects pack boundaries:
a permit defined in `pack-us-fedramp-mod` Cedar fragment-set is loaded
into BOTH Diana's GAO tenant AND Marcus's tenant — but it scopes to
each tenant's own resources, not across.

## 13. The wider economy implication

Diana is one of ~280 active FedRAMP 3PAOs in the United States. Each
has the same dual-tenant identity profile. Each is a critical link in
the federal-cloud-security supply chain. If oyatie were to make
ANY architectural decision that compromised this boundary, the entire
3PAO industry would have to migrate off the platform. The federal
auditor's trust in the architecture is a load-bearing economic
relationship, not a feature.

This story — j126 — is the **highest-stakes** non-emergency dual-tenant
journey in the catalog. Six journeys (j126-j131) elaborate the same
architecture under different stress scenarios:

- j127 (resignation) tests **tenant departure** — what happens when
  work-tenant access is revoked but personal-tenant must continue.
- j128 (personal tax workflow) tests **personal-tenant productive use**
  — what happens when Diana exercises Workflow Studio on her personal
  side, the agency cannot see.
- j129 (court warrant) tests **judicial-oversight piercing** — what
  happens when a court issues a warrant scoped to her personal tenant
  unrelated to her work.
- j130 (bribery attempt) tests **personal-to-work bridging via
  Community whistleblower** — Diana receives a personal-tenant DM
  offering a bribe; she reports via Community; cross-tenant evidence
  chain.
- j131 (cross-jurisdiction) tests **EU-vs-KR data-residency
  reconciliation** — Diana audits a multinational with subsidiaries
  in both regions.

Each preserves ADR-0311. Each preserves ADR-0312 where a piercing is
involved. Each is operationally distinct.

## 14. The fairness corollary — why this also benefits Marcus

The same architecture that protects Diana's family chat from her
agency ALSO protects Marcus from regulator overreach. If a GAO
auditor later wanted to pull, say, Marcus's CFO's personal Messenger
to investigate an unrelated suspicion, they could not — because
Marcus's CFO has a separate personal tenant outside the
`chen-aerospace.federal-contractor.us` boundary. Cedar default-deny
holds for the CFO too.

Symmetry. Both parties get the same guarantees. The platform does not
asymmetrically advantage the auditor; it disciplines BOTH the auditor
(no overreach) AND the auditee (no concealment).

This is the Stripe-Connect-platform pattern: the platform-facilitator
makes BOTH sides better off by enforcing rules neither could enforce
alone. Diana's agency couldn't enforce no-overreach against itself;
Marcus couldn't enforce no-concealment against himself. The platform
enforces both.

## 15. The forty-three minutes — Diana's day moves on

By 10:35 EST Diana is at her kitchen counter making a second coffee.
Her ThinkPad is locked. Her personal phone is on the counter showing
the family chat thread. Her wife pings: "you free for lunch at the
Eastern Market today before the gallery thing tonight?". Diana replies:
"yes 12:30 at South Hall".

She has a follow-up audit task to come back to at 11:00 EST. The
platform held the boundary. Marcus Chen's audit is in progress.
Diana's family chat is sealed in a tenant only she can see.

This is the bar. ADR-0311 ships. The audit-chain holds. The Cedar
default-deny holds. The cell boundary holds. The pack overlays
compose without leakage. The same human, two tenants, one passkey,
zero leakage. Hyperscaler-grade dual-tenant identity.

If we ship anything that fails this, we ship something that the SEC,
the GAO, every federal agency with a 3PAO arrangement, every
multinational corporation with an HR-internal-audit boundary, every
journalist with a press-source surface, every domestic-violence
survivor with a shelter-mode profile, and every minor with a
parental-oversight setup will refuse to trust. The architecture has
to be right. j126 is the highest-frequency proving ground. We do not
ship until j126 works exactly as described above.

## 16. Open hooks — what j126 leaves to siblings

- The **Workflow Studio** surface (Diana's tax workflow) is exercised
  in detail in j128. j126 only references it.
- The **Marketplace** surface (Diana's vintage records) is referenced
  but not exercised; j130 brings it in via the bribery-attempt chain.
- The **EU jurisdiction overlay** is not exercised here (Diana audits
  a US-only contractor); j131 brings the EU+KR multi-jurisdiction
  story.
- The **judicial-piercing path** is not exercised here (no warrant);
  j129 brings it.
- The **resignation path** (work-tenant revocation, personal-tenant
  intact) is not exercised here (Diana is not changing employers);
  j127 brings it.

j126 is the **foundation** story. The remaining five build on it.

## 17. Hyperscaler precedent — who else has shipped this shape

Three named precedents make this architecturally tractable:

1. **Apple Personal Apple ID vs. Apple Business / Apple School
   Manager Managed Apple ID.** Same human, two identity contexts,
   strict data-isolation by tenant. iCloud personal photos NOT
   visible in Managed Apple ID. Apple's documentation explicitly
   forbids cross-tenant reads even for Apple-internal support
   without user-initiated consent.
2. **Microsoft Personal Account vs. Microsoft Work/School Account
   (Entra ID).** Same human, two distinct Azure AD tenants for
   identity. Personal-account OneDrive NOT readable from
   work-account. Cross-tenant collaboration requires explicit
   guest-invite (Entra B2B) — analogous to oyatie's
   cross-tenant Cedar permit.
3. **Google Personal Account vs. Google Workspace.** Same human,
   two distinct domains. Workspace admin CANNOT read personal
   Gmail of the same human; Vault audit is scoped to Workspace
   domain only.

oyatie's distinction over these: **enforcement at the policy layer
(Cedar)** rather than at the UI/feature layer. Apple, Microsoft,
Google enforce by feature-disabling cross-tenant surfaces. oyatie
enforces by default-denying every cross-tenant action and requiring
explicit, scoped, attested, soak-windowed permits to lift the default.

This is more robust against feature drift. New µservices are added
to the platform every release; the default-deny baseline means new
µservices automatically inherit the boundary without per-µservice
configuration. Apple/Microsoft/Google must explicitly extend their
boundary to each new feature; we get it for free.

## 18. What this story would look like with audience-type wired wrong

If `audience_type = INTERNAL_AUDITOR_3PAO` were not enforced and
Diana presented as `audience_type = B2C_CONSUMER`, the cross-tenant
permit in Marcus's tenant would not match (the permit requires the
3PAO attribute). Diana would receive a Cedar Deny response from
`policy-engine` and the dashboard would show:

```
ACCESS DENIED
Permit evaluation: DENY
Reason: principal.audience_type = "B2C_CONSUMER" does not match required
attribute audience_type = "INTERNAL_AUDITOR_3PAO"
Appeal: see policy/cross-tenant-fedramp-3pao-audit-evidence.cedar
or contact your agency's 3PAO accreditation officer.
```

This is the intended behavior. Cedar protects against attribute
spoofing. The audience-type attribute is set ONLY at session-init
when identity verifies the user's 3PAO accreditation in the
accreditation registry. It cannot be self-asserted.

Symmetrically, if a non-auditor in Diana's agency (say, an
administrative employee in GAO's HR department) tried to exercise
the cross-tenant permit, they would fail the
`audience_type == "INTERNAL_AUDITOR_3PAO"` check. Even within the
same tenant, the permit is scoped to a sub-class of principal.

## 19. The story's invariants — what j126 promises

j126's narrative MUST hold the following invariants at runtime,
verified by integration tests in `integration-test-plan.md`:

1. Diana's family-chat message at T+28000ms emits zero audit events
   in her GAO tenant's audit log.
2. Diana's audit-pull at T+13000ms emits the matching cross-tenant
   pair: `CrossTenantAuditEvidencePulled` in GAO + `CrossTenantAuditEvidenceExported` in Chen-Aerospace.
3. Marcus's tenant-admin Marcus Chen receives a notification at
   T+13500ms (within the 15-minute floor).
4. The Cedar permit evaluation at T+12000ms takes ≤50ms (p99) per
   ADR-0246 amendment §D-policy-evaluation-latency.
5. The audit-evidence bundle's Merkle-seal is verifiable end-to-end
   from Diana's dashboard through audit-chain seal root.
6. The cross-tenant emission to BOTH tenants is atomic per
   ADR-0028 §D-cross-tenant-atomicity (either both emit or neither).
7. The personal-tenant Messenger emission lands in personal-tenant
   audit-chain only (zero cross-references to GAO tenant).
8. Diana's `audience_type` attribute set at session-init reflects
   her current accreditation status (live lookup, ≤200ms p99).
9. No GAO tenant principal can read any resource in Diana's
   personal tenant. Integration test verifies by attempting and
   observing Deny.
10. No personal-tenant principal can read any resource in Marcus's
    Chen-Aerospace tenant. Integration test verifies by attempting
    and observing Deny.

Ten invariants. Tests verify ten. Architecture ships when all ten
hold.

## 20. The deliberate omission

This story does NOT describe what happens if Diana's accreditation
LAPSES mid-audit. That edge case is critical-path row 19 (tenant
break-glass / dead-account recovery) territory and belongs to a
sibling journey (j137 corporate internal-audit SOX controls test
exercises the corollary). The ADR-0311 fragment specifies the
behavior — accreditation lapse mid-audit revokes the permit on the
next evaluation, and the in-flight bundle is sealed-and-handed-off to
a fresh 3PAO via the workflow-engine's reassignment surface.

## 21. The bottom line

Diana Reyes audited Marcus Chen. Diana Reyes texted her mom.
Both happened in forty-five minutes on the same person using the
same passkey on two devices.

Neither leaked to the other.

That is the bar oyatie ships. ADR-0311 is the codification. j126 is
the proof.

## Completion expansion — j126 story rigor pass

Scope: FedRAMP 3PAO audit with Diana work/personal tenant separation.
Persona: Diana Reyes.
Services: identity + tenancy + audit-chain + compliance + ops-dashboard-control-center + observability.
Applicable ADRs: ADR-0244, ADR-0299, ADR-0311, ADR-0312, ADR-0313, ADR-0314, ADR-0315, ADR-0316, ADR-0317, ADR-0318, ADR-0319, ADR-0320.
The expansion below is intentionally explicit so an intern can build and verify the journey without oral context.

Narrative beat 001: Diana Reyes advances FedRAMP 3PAO audit with Diana work/personal tenant separation; the active tenant label remains visible before any tenancy action is accepted.
Boundary assertion 002: Cedar evaluates the principal tenant, resource tenant, audience type, and purpose binding; personal-tenant content remains unavailable unless ADR-0312 judicial scope exists.
Evidence note 003: compliance emits an ADR-0263 audit event with tenant_id, actor, case_id, pack_set, and HLC timestamp so the journey can be reconstructed without private content.
Human impact 004: Diana Reyes sees plain-language status, a reasoned refusal when needed, and a next lawful step that does not leak cross-tenant facts.
Narrative beat 005: Diana Reyes advances FedRAMP 3PAO audit with Diana work/personal tenant separation; the active tenant label remains visible before any observability action is accepted.
Boundary assertion 006: Cedar evaluates the principal tenant, resource tenant, audience type, and purpose binding; personal-tenant content remains unavailable unless ADR-0312 judicial scope exists.
Evidence note 007: tenancy emits an ADR-0263 audit event with tenant_id, actor, case_id, pack_set, and HLC timestamp so the journey can be reconstructed without private content.
Human impact 008: Diana Reyes sees plain-language status, a reasoned refusal when needed, and a next lawful step that does not leak cross-tenant facts.
Narrative beat 009: Diana Reyes advances FedRAMP 3PAO audit with Diana work/personal tenant separation; the active tenant label remains visible before any compliance action is accepted.
Boundary assertion 010: Cedar evaluates the principal tenant, resource tenant, audience type, and purpose binding; personal-tenant content remains unavailable unless ADR-0312 judicial scope exists.
Evidence note 011: observability emits an ADR-0263 audit event with tenant_id, actor, case_id, pack_set, and HLC timestamp so the journey can be reconstructed without private content.
Human impact 012: Diana Reyes sees plain-language status, a reasoned refusal when needed, and a next lawful step that does not leak cross-tenant facts.
Narrative beat 013: Diana Reyes advances FedRAMP 3PAO audit with Diana work/personal tenant separation; the active tenant label remains visible before any tenancy action is accepted.
Boundary assertion 014: Cedar evaluates the principal tenant, resource tenant, audience type, and purpose binding; personal-tenant content remains unavailable unless ADR-0312 judicial scope exists.
Evidence note 015: compliance emits an ADR-0263 audit event with tenant_id, actor, case_id, pack_set, and HLC timestamp so the journey can be reconstructed without private content.
Human impact 016: Diana Reyes sees plain-language status, a reasoned refusal when needed, and a next lawful step that does not leak cross-tenant facts.
Checkpoint 01: preserve OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar default-deny, audit-chain seal, and 56-µservice flat-layout assumptions.
Narrative beat 017: Diana Reyes advances FedRAMP 3PAO audit with Diana work/personal tenant separation; the active tenant label remains visible before any observability action is accepted.
Boundary assertion 018: Cedar evaluates the principal tenant, resource tenant, audience type, and purpose binding; personal-tenant content remains unavailable unless ADR-0312 judicial scope exists.
Evidence note 019: tenancy emits an ADR-0263 audit event with tenant_id, actor, case_id, pack_set, and HLC timestamp so the journey can be reconstructed without private content.
Human impact 020: Diana Reyes sees plain-language status, a reasoned refusal when needed, and a next lawful step that does not leak cross-tenant facts.
Narrative beat 021: Diana Reyes advances FedRAMP 3PAO audit with Diana work/personal tenant separation; the active tenant label remains visible before any compliance action is accepted.
Boundary assertion 022: Cedar evaluates the principal tenant, resource tenant, audience type, and purpose binding; personal-tenant content remains unavailable unless ADR-0312 judicial scope exists.
Evidence note 023: observability emits an ADR-0263 audit event with tenant_id, actor, case_id, pack_set, and HLC timestamp so the journey can be reconstructed without private content.
Human impact 024: Diana Reyes sees plain-language status, a reasoned refusal when needed, and a next lawful step that does not leak cross-tenant facts.
Narrative beat 025: Diana Reyes advances FedRAMP 3PAO audit with Diana work/personal tenant separation; the active tenant label remains visible before any tenancy action is accepted.
Boundary assertion 026: Cedar evaluates the principal tenant, resource tenant, audience type, and purpose binding; personal-tenant content remains unavailable unless ADR-0312 judicial scope exists.
Evidence note 027: compliance emits an ADR-0263 audit event with tenant_id, actor, case_id, pack_set, and HLC timestamp so the journey can be reconstructed without private content.
