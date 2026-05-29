---
doc_class: User-Journey-README
journey_id: j151-captain-olufemi-typhoon-evacuation-and-co-op-cash-flow
slice: green-collar-emergency-and-cooperative-payments
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Captain Olufemi
audience_type: B2B_TENANT_ADMIN
microservice_count: 5
pack_overlay_anchor: NG-NDPR + ECOWAS-Maritime-Safety + ISO-31000
related_adrs:
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0243-cedar-as-universal-gate
  - ADR-0263-observability-emission-contract
  - ADR-0298-emergency-services-bypass-life-safety
  - ADR-0317-role-projection-doctrine
  - ADR-0299-account-recovery-passkey-bound-identity
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0248-amazon-shape-cellular-architecture
---

# j151 — Captain Olufemi: Typhoon evacuation and co-op cash flow

## At a glance

Captain Olufemi runs the Bonny-Lekki Cooperative — fourteen owner-operator fishing vessels homeported in Lekki, Lagos State, Nigeria. At 04:18 WAT on a Thursday in October, NIMET (Nigerian Meteorological Agency) escalates Cyclone Aisha from category-2 to category-3 with projected landfall in 26 hours. Olufemi has to:

1. Recall all fourteen captains while they are at sea over 96–280 nautical miles offshore on patchy VHF + Inmarsat-C
2. Stand up a co-op-wide evacuation roll-call on `messenger` that handles the 14 captains plus 87 crew plus ~310 dependent family members on a single broadcast channel
3. Release emergency cash advances from the co-op's pooled escrow — ₦4.2M total — to crew families that need it before the storm makes landing impossible
4. Capture an `audit-chain` trail that survives the regulator's post-storm review (NIMASA, NIMET, the cooperative regulator) and an insurance adjuster's reconstruction
5. Notify `connector`-bridged buyers (three cold-storage warehouses, two restaurant chains in Lagos, one export broker in Abidjan) that the next 9-day catch window is cancelled

The journey exercises **five microservices in concert under emergency conditions** — `payments`, `finops-portal`, `messenger`, `audit-chain`, `connector` — with the secondary touch on `identity`, `tenancy`, `workflow-engine`, `compliance`, and `observability`. The Cedar policy invokes **ADR-0298 emergency-services bypass** to lift the normal 24-hour escrow disbursement hold without disabling the audit ledger.

## Why this journey matters

Captain Olufemi is the highest-leverage green-collar persona in MASTER-ROSTER §3.2 row 24. His tenant (Bonny-Lekki Cooperative) is a B2B_TENANT_ADMIN — a small but real cooperative — operating in a regulated maritime jurisdiction with intermittent connectivity. The journey closes three gaps the §10 matrix called out:

- Critical-path row 1 (life-safety bypass) for the green-collar field cluster
- Critical-path row 30 (multi-tenant cooperative settlement)
- The supply-chain-cluster gap on buyer-side notification when the supply side fails for safety reasons

Hyperscaler benchmark: Stripe for the cooperative-payment flow; Twilio Conversations for the broadcast roll-call channel; AWS IoT Core for the vessel telemetry that confirms each captain's GPS position before the recall is declared complete.

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| `story.md` | Narrative with timestamped beats; named captains and crew; specific cash amounts; specific Cedar decisions | Every beat names the actor, the exact Cedar permit class, the exact ADR-0263 audit event, the GPS coordinates or vessel name |
| `ux-flow.md` | Screen-by-screen progression on Olufemi's vehicle-mount handheld + co-op desktop dashboard | Every screen names the modal, the copy on the primary button, the failure copy, the accessibility affordance for low-light conditions on a moving boat |
| `handshake.md` | Per-microservice contract with named API call, request body shape, response shape, error paths | Each row enumerates the exact gRPC method or HTTP route, the proto3 message field set, the Cedar permit, and the audit-event class |
| `integration-test-plan.md` | Concrete pass/fail criteria with seeded data | Each test names the seed values, the expected event chain, the failure-injection trigger |
| `schemas/openapi-emergency-recall.json` | OpenAPI 3.2.0 for the `POST /v1/co-op/{coop_id}/emergency-recalls` endpoint | Real request/response shape with NIMET hazard code enum |
| `schemas/asyncapi-vessel-telemetry.yaml` | AsyncAPI 3.1.0 for `vessel.telemetry.gps_v1` topic | Real Avro-equivalent schema; partition key strategy; retention |
| `schemas/journey-messages.proto` | proto3 for the five core RPC messages | Field tags, enum values, semantic versioning markers |
| `schemas/cedar-policy.cedar` | Cedar policy fragment with ADR-0298 bypass condition | Named principal, action, resource, when-clause |

## The five microservices in scope

| µservice | Role in this journey | Critical-path row |
|---|---|---|
| `payments` | Releases the ₦4.2M crew-family emergency advance from the co-op pooled escrow; tracks each disbursement against the per-crew daily cap (₦35,000) under ADR-0298 bypass | row 30 |
| `finops-portal` | Surfaces the cooperative's escrow balance to Olufemi in real time; runs the post-storm reconciliation against the actual catch loss | row 30 |
| `messenger` | Hosts the `co-op-evacuation-2026-10-23` broadcast channel; ferries ACK / NACK / NEED-FUEL / NEED-MEDICAL flags from each captain back to the co-op dashboard | row 1 |
| `audit-chain` | Seals every roll-call entry, every Cedar bypass invocation, every payment disbursement under ADR-0263 emission classes; provides the regulator-ready evidence bundle | row 1, row 30 |
| `connector` | Brokers the cancellation notice to the six external buyers (three cold-storage, two restaurants, one Abidjan broker) without exposing the co-op's internal financial state | row 30 |

## Secondary microservices touched

| µservice | Touch reason |
|---|---|
| `identity` | Resolves each captain's passkey principal at sea; uses pre-staged WebAuthn-backed-by-SIM-eUICC credential for offshore re-auth |
| `tenancy` | Maintains the cooperative as a single tenant; each vessel is a sub-resource not a sub-tenant |
| `workflow-engine` | Durable orchestration of the 9-step recall workflow; compensation on partial failure |
| `compliance` | Activates the NG-NDPR pack overlay for crew personal data; activates the maritime-safety pack for evacuation evidence |
| `observability` | Emits the `co-op-evacuation-2026-10-23` trace span; surfaces the 14 captain ACK latencies on a single dashboard |

## Pack overlays

| Pack | Activation reason |
|---|---|
| NG-NDPR | Crew personal data (passkey, family contact, family-cash-recipient bank account) is processed under the Nigeria Data Protection Regulation 2019 |
| ECOWAS-Maritime-Safety | Evacuation evidence retention (7 years) and incident-reporting cadence (24h initial / 7d final) |
| ISO-31000 | Risk-management evidence; the Cedar bypass invocation is itself a risk-decision event |

## Regulatory anchors

1. NIMET storm escalation protocol; cooperative recall decision linked to NIMET hazard code (in this case `NIMET-2026-AISHA-CAT3`)
2. NIMASA (Nigerian Maritime Administration and Safety Agency) vessel-recall protocol Form NM-7B; the cooperative files a single Form NM-7B for all fourteen vessels via the `connector` bridge
3. NG-NDPR Section 2.1 lawful-basis (vital interests of crew + family) for the personal-data processing during the recall
4. ECOWAS Convention on Search and Rescue 1979 article 3 obligations for cooperative-level recall coordination
5. ADR-0298 emergency-services bypass — the Cedar policy that lifts the normal 24-hour escrow hold on the ₦4.2M disbursement
6. ADR-0244 tenant scoping — every audit-event carries `tenant_id = bonny_lekki_cooperative`
7. ADR-0263 audit-event classes — every state transition emits a sealed event

## Cell + certification matrix

| Cell | Certification | Journey use |
|---|---|---|
| `ng-lagos-primary` | NG-NDPR-ready | Primary placement for the cooperative tenant; hosts the live escrow ledger |
| `eu-frankfurt-readonly-replica` | GDPR-ready | Cross-border read replica for the Abidjan broker (Côte d'Ivoire is an ECOWAS member and uses the EU-aligned data-residency lane) |
| `global-shared-control-plane` | SOC2 | Hosts the `messenger` broadcast channel; messages are content-encrypted client-side, server holds metadata only |

## Cedar permit class (excerpt — full text in `schemas/cedar-policy.cedar`)

```cedar
permit (
    principal == User::"olufemi.adekunle@bonny-lekki-coop",
    action in [
        Action::"emergency.recall.initiate",
        Action::"emergency.escrow.disburse",
        Action::"emergency.buyer.notify"
    ],
    resource is Cooperative
) when {
    principal.passkey_step_up_within_seconds(60) &&
    resource.tenant_id == "bonny_lekki_cooperative" &&
    context.emergency_declared == true &&
    context.nimet_hazard_code matches "NIMET-[0-9]{4}-[A-Z]+-CAT[1-5]" &&
    context.adr_0298_bypass_active == true &&
    principal.role_in_cooperative == "captain_of_record"
};
```

## Why ADR-0298 bypass is invoked rather than the normal escrow flow

The cooperative escrow has a **24-hour standard hold** before any disbursement (this hold is itself a Cedar policy class — `payment.escrow.standard_hold_24h`). The storm-landfall window is 26 hours. Without the bypass, the ₦35,000-per-family advances would not arrive before the storm makes mobile-money agent visits impossible (the agents close shop at landfall-minus-12h).

ADR-0298 explicitly authorises the bypass when:

- A named NIMET / NEMA / NIMASA emergency declaration is in force
- The bypass invoker (Olufemi) is a designated cooperative officer
- The disbursement targets a pre-registered crew-family bank account (no new payees)
- The per-recipient cap stays under the cooperative's pre-approved emergency limit (₦35,000 per crew member)
- Every disbursement is sealed in `audit-chain` under the `EVT-J151-PAYMENTS-EMRG-DISBURSE-NNN` class

The Cedar policy refuses the bypass if the storm declaration is later than 24 hours old without re-confirmation, or if Olufemi's passkey step-up is older than 60 seconds at the disbursement instant.

## Acceptance summary

| AC | Result expected |
|---|---|
| AC-J151-001 | All 14 captains ACK their position via VHF or Inmarsat-C within 90 minutes of the recall; `messenger` shows 14/14 green |
| AC-J151-002 | All 87 crew members are flagged personally-safe within 120 minutes; the 5 reported NEED-MEDICAL routed to NEMA via the `connector` bridge |
| AC-J151-003 | All 87 crew-family advances (₦35,000 × 87 = ₦3.045M) disbursed within 180 minutes; the bypass invocation is sealed in `audit-chain` with the NIMET hazard code in the event payload |
| AC-J151-004 | The reserve ₦1.155M (for fuel + spoiled-catch compensation) is held pending post-storm review; the `finops-portal` shows the cooperative escrow remaining balance correctly |
| AC-J151-005 | All 6 external buyers receive a cancellation notice via `connector` within 60 minutes of the recall declaration; the notices show only the cancellation reason ("maritime-safety-declared-emergency") not the cooperative's internal financial state |
| AC-J151-006 | The post-storm regulator bundle (NIMASA Form NM-7B + NIMET acknowledgement + NDPR processing log) is exportable from `audit-chain` in a single click, signed by the cooperative's passkey-bound identity |
| AC-J151-007 | No Cedar permit denies a legitimate operation; no Cedar permit allows a non-emergency operation during the bypass window |
| AC-J151-008 | The 60-second passkey step-up is enforced for every disbursement; an attempt by a non-captain-of-record member to invoke the bypass is refused with an audit-event sealed |

## Cross-references

- Persona dossier: `docs/personas/captain-olufemi.md`
- MASTER-ROSTER §3.2 row 24
- Matrix §10 j151 recommendation
- Existing related journeys: j11 (offline-first sync), j107 (supply-chain disruption), j124 (multi-tenant coordinated launch)
- Pack roster: `packs/ng-ndpr/` (when present) plus the maritime-safety overlay
- ADR-0298 emergency-services bypass life-safety
- ADR-0263 observability emission contract
- ADR-0244 tenant scoping
- ADR-0317 role-projection (Olufemi is captain-of-record AND cooperative-officer AND family-provider; the journey activates only the first two)

## What this journey deliberately does NOT cover

- The post-storm insurance-claim journey (that is j151+α follow-up; in scope for a future journey)
- Vessel sale / new-vessel acquisition (different lifecycle)
- The fishery quota / catch reporting under NIMASA — that is the steady-state journey, not the emergency journey
- Cross-border vessel handling for Cameroonian / São Toméan vessels visiting Lekki (different tenant)

## Stop condition

This journey is complete when all 8 acceptance criteria pass on the seeded test fixture, the schema files validate against their meta-schemas, every named ADR resolves, every named µservice exists in `/microservices/`, and the persona dossier matches MASTER-ROSTER §3.2 row 24.
