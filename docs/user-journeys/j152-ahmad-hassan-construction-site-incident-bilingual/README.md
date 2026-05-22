---
doc_class: User-Journey-README
journey_id: j152-ahmad-hassan-construction-site-incident-bilingual
slice: blue-collar-osha-incident-bilingual-arabic-english
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Ahmad Hassan
audience_type: B2B_FIELD_WORKER
microservice_count: 5
pack_overlay_anchor: US-OSHA + US-EEOC-language-access + ISO-45001
related_adrs:
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0243-cedar-as-universal-gate
  - ADR-0263-observability-emission-contract
  - ADR-0317-role-based-projection-unified-ux-shell
  - ADR-0298-emergency-services-bypass-life-safety
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0318-collar-color-workspace-universality
  - ADR-0251-compliance-pack-cell-certification-levels
---

# j152 — Ahmad Hassan: Construction-site incident with bilingual reporting

## At a glance

Ahmad Hassan is a Construction Site Lead employed by **Halcyon Build LLC**, a 47-person residential general contractor working on a 9-story mixed-use project at 4421 Telegraph Ave, Oakland CA 94609. He is bilingual Arabic-English (L1 Arabic Levantine; L2 English C1). Two of his eight-person crew are Arabic-only (Levantine + Egyptian dialects); three are Spanish-only; three are English-native. At 14:37 PDT on a Tuesday in mid-October, his junior fitter **Khalil Mansour** is struck on the shoulder by a 3.6-meter rebar bundle that slipped out of the tower-crane sling on the 6th-floor deck. Khalil is conscious and ambulatory but bleeding from a scalp laceration and complaining of right-shoulder pain. Ahmad has to:

1. Trigger the OSHA-compliant incident workflow from his **handheld-rugged** device (Kyocera DuraForce) before the bleeding stops, while keeping the cardinal facts captured in **both Arabic and English** for the post-incident review
2. Pull the **last 90 seconds of crane-load telemetry** and the **last 4 minutes of deck-camera footage** into the incident record automatically
3. Broadcast an Arabic-Spanish-English **stop-work** directive on the site `messenger` channel to all 8 crew + 3 subcontractors on deck
4. Seal the **audit-chain** evidence trail before the foreman's GC liaison or the OSHA Form 301 worker can touch it, so the chain-of-custody survives the OSHA Area Office review
5. Open a **workplace-integration** sync so that the **HR system of record at Halcyon Build (Paycom)** and the **workers'-compensation carrier (State Fund)** see the incident inside SLA — 8 hours for Cal/OSHA recordable, 24 hours for serious-injury under §342

The journey exercises **five microservices under field-grade bilingual conditions**: `incident-management`, `messenger`, `audit-chain`, `workplace-integration`, `drive` — with secondary touches on `identity`, `tenancy`, `workflow-engine`, `compliance`, `observability`, and `connect`. The Cedar policy invokes ADR-0298 emergency-services bypass to allow Ahmad to attach Khalil's pre-existing medical-history excerpt (limited to relevant allergies) from `drive` without invoking the normal 4-eyes HR-access policy.

## Why this journey matters

Ahmad Hassan is **MASTER-ROSTER §3.2 row 13** — the highest-leverage blue-collar field persona representing US construction's bilingual reality (per BLS 2024, 30.7% of US construction workers speak a language other than English at home). The journey closes two critical-path gaps the §10 matrix flagged:

- Critical-path row 8 (blue-collar field incident reporting under language-access obligations)
- Critical-path row 30 (multi-tenant data flow into a tenant's HR system-of-record and the workers'-comp carrier)

Hyperscaler benchmark: ServiceNow IT Service Management for the incident-record lifecycle; Procore for the construction-domain field surfaces; Twilio Voice + Conversations for the bilingual stop-work broadcast.

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| `story.md` | Time-stamped narrative from 14:37:11 PDT through 18:52 PDT close-of-shift; named crew; bilingual lexicon switching at each beat | Every beat names the actor, the Cedar permit, the ADR-0263 audit-event class, the camera ID, the crane sensor ID, the language pair |
| `ux-flow.md` | Screen-by-screen progression on Ahmad's DuraForce + the Halcyon Build site-trailer dashboard | Every screen names the modal copy in EN + AR; the language-switch glyph; the field-glove-friendly tap-target size (≥56dp) |
| `handshake.md` | Per-microservice contract with named API call, request body shape, response shape, error paths | Each row enumerates the gRPC method or HTTP route, the proto3 message field set, the Cedar permit, the audit-event class |
| `integration-test-plan.md` | Concrete pass/fail criteria with seeded data | Each test names the seed values, expected event chain, failure-injection trigger |
| `schemas/openapi-incident-create.json` | OpenAPI 3.2.0 for `POST /v1/sites/{site_id}/incidents` endpoint | Real request/response shape with OSHA injury-type enum + bilingual `narrative` array |
| `schemas/asyncapi-crane-telemetry.yaml` | AsyncAPI 3.1.0 for `crane.load_pin.sensor_v1` topic | Real schema; partition key strategy; 90-second windowed retention rule |
| `schemas/journey-messages.proto` | proto3 for the 6 core RPC messages | Field tags, enum values, semantic versioning markers |
| `schemas/cedar-policy.cedar` | Cedar fragment with ADR-0298 bypass + ADR-0311 dual-tenant medical-access rule | Named principal, action, resource, when-clause |
| `schemas/workplace-integration-paycom-map.yaml` | Field-mapping spec for Halcyon Build → Paycom HR sync | Source field → target field with transform rule per OSHA 301 line item |

## The five microservices in scope

| µservice | Role in this journey | Critical-path row |
|---|---|---|
| `incident-management` | Owns the incident record `INC-2026-1014-HB-OAK-4421-0007`; drives the OSHA-301 + Cal/OSHA-§342 workflow; carries the bilingual narrative pair | row 8 |
| `messenger` | Hosts site channel `site-hb-oak-4421-deck-6`; carries the stop-work broadcast in EN + AR + ES; logs ACK from each crew device | row 30 |
| `audit-chain` | Seals the incident record, the bypass invocation, every workplace-integration handoff under ADR-0263 emission classes | row 8 |
| `workplace-integration` | Bidirectional sync to Paycom (incident → workers'-comp record) and to State Fund (incident → claim FROI-1) within the §342 8-hour window | row 30 |
| `drive` | Holds Khalil Mansour's pre-existing medical-allergy excerpt (consented under ADR-0311); attaches the camera footage + crane-telemetry CSV as evidence | row 8, row 30 |

## Secondary microservices touched

| µservice | Touch reason |
|---|---|
| `identity` | Resolves Ahmad's site-lead role + Khalil's worker role; passkey step-up for the bypass invocation |
| `tenancy` | Halcyon Build LLC is the tenant; each site is a sub-resource; the carrier (State Fund) is `connect`-bridged |
| `workflow-engine` | Durable orchestration of the 11-step incident workflow; compensation if Paycom is down |
| `compliance` | Activates the US-OSHA pack overlay + the EEOC language-access pack |
| `observability` | Emits the trace `incident-hb-oak-4421-2026-1014` with 11 spans; SLO is incident-create → workplace-integration ack ≤ 8 hours |
| `connect` | Bridges to State Fund (workers'-comp carrier); separately bridges to Cal/OSHA Area Office Oakland |

## Pack overlays

| Pack | Activation reason |
|---|---|
| US-OSHA | The incident is potentially OSHA-recordable (29 CFR 1904); Form 300/301 path activates |
| US-EEOC-language-access | EEOC 2002 language-access guidance for limited-English-proficiency workers requires the stop-work broadcast in all three site languages |
| ISO-45001 | Halcyon Build is ISO 45001-certified; the incident-management workflow surfaces the §10.2 corrective-action loop |

## Regulatory anchors

1. 29 CFR 1904 (OSHA recordkeeping); the incident is at minimum a Form 301 first-aid record; if Khalil is referred to an ER, it escalates to recordable
2. Cal/OSHA T8 §342 (serious-injury 8-hour reporting); the workflow timer is bound to landfall-on-incident + 8h
3. EEOC 2002 language-access guidance (the stop-work broadcast must be in all languages the workforce uses at work)
4. California Labor Code §3550 (workers'-comp posting); State Fund FROI-1 (First Report of Injury) auto-derived from the incident record
5. ADR-0298 emergency-services bypass for Ahmad's attachment of Khalil's allergy excerpt (skips 4-eyes review during the 60-minute acute window)
6. ADR-0244 tenant scoping: every event carries `tenant_id = halcyon_build_llc`
7. ADR-0263 audit-event classes seal every state transition

## Cell + certification matrix

| Cell | Certification | Journey use |
|---|---|---|
| `us-west-2-primary` | US-OSHA-ready + HIPAA-ready (for medical-allergy excerpt) | Primary placement for Halcyon Build tenant; hosts incident record |
| `us-west-2-edge-oakland` | edge-cell | Holds the deck-camera + crane-telemetry stream within 30km of the site for sub-200ms ingestion |
| `global-shared-control-plane` | SOC2 | Hosts the `messenger` broadcast; content-encrypted client-side |

## Cedar permit class (excerpt — full text in `schemas/cedar-policy.cedar`)

```cedar
permit (
    principal == User::"ahmad.hassan@halcyon-build.com",
    action in [
        Action::"incident.create",
        Action::"incident.attach_medical_excerpt",
        Action::"incident.stop_work_broadcast",
        Action::"workplace_integration.paycom.write_incident"
    ],
    resource is Site
) when {
    principal.passkey_step_up_within_seconds(120) &&
    resource.tenant_id == "halcyon_build_llc" &&
    principal.role_on_site(resource.site_id) == "site_lead" &&
    context.incident_class in ["near_miss", "first_aid", "recordable", "serious"] &&
    (
        action != Action::"incident.attach_medical_excerpt" ||
        (
            context.adr_0298_bypass_active == true &&
            context.acute_window_minutes <= 60 &&
            principal.has_consent_token(context.affected_worker_id, "allergy_excerpt") == true
        )
    )
};
```

## Why ADR-0298 bypass is invoked rather than the normal medical-access flow

The normal `drive` rule for any worker's medical record requires a 4-eyes review (HR officer + worker's verbal consent inside 24 hours). The active 60-minute acute window starts when the incident is created with severity ≥ first_aid and Khalil has a flagged allergy (sulfa drugs, codeine) that the responding EMT needs to see immediately on the paramedic's tablet.

ADR-0298 authorises the bypass when:

- A site-lead role-of-record is the invoker
- The worker has a pre-signed standing-consent token for allergy disclosure to first responders (signed under ADR-0311 when Khalil onboarded with Halcyon Build)
- The acute window is ≤60 minutes since `incident.create`
- The excerpt scope is narrow — only the `allergies` and `current_medications` fields, not the full medical record
- Every disclosure is sealed in `audit-chain` under class `EVT-J152-DRIVE-MED-EMRG-DISCLOSE-NNN`

The Cedar policy refuses if Ahmad's passkey step-up is older than 120 seconds at attachment time, if Khalil's standing-consent token has been revoked, or if the acute window has already lapsed (a "stale-bypass" attempt produces a `EVT-J152-CEDAR-DENY-STALE-BYPASS` audit event).

## Acceptance summary

| AC | Result expected |
|---|---|
| AC-J152-001 | Incident `INC-2026-1014-HB-OAK-4421-0007` created within 90 seconds of Ahmad's `Report Incident` tap; bilingual narrative pair (EN + AR) stored as two structured fields not concatenated |
| AC-J152-002 | Stop-work broadcast delivered to all 8 crew + 3 subcontractor devices on `site-hb-oak-4421-deck-6` within 8 seconds; ACK from each device recorded under `EVT-J152-MSG-STOPWORK-ACK-NNN` |
| AC-J152-003 | Crane load-pin telemetry (90s window, 50Hz, 4,500 samples) and deck camera ID `cam-deck-6-northwest` 4-minute clip attached to the incident automatically; chain-of-custody hash sealed in `audit-chain` |
| AC-J152-004 | ADR-0298 bypass invocation for Khalil's allergy excerpt succeeds; the excerpt shows `sulfa_drugs`, `codeine` only; full medical record is NOT exposed; the disclosure event is sealed |
| AC-J152-005 | Paycom HR-system sync writes the incident as workers'-comp pending claim within 6 minutes; State Fund receives FROI-1 derivation within 90 minutes |
| AC-J152-006 | Cal/OSHA §342 8-hour timer is set on incident create; the timer fires a reminder at T+6h; at T+8h, if the report is not submitted via the `connect` bridge to the Area Office, the workflow escalates to Halcyon Build's safety officer's pager |
| AC-J152-007 | The bilingual incident narrative survives export to the OSHA-301 PDF in both EN and AR (Arabic right-to-left rendering preserved); the audit-chain hash includes both narratives |
| AC-J152-008 | An attempt by a non-site-lead crew member to invoke `incident.attach_medical_excerpt` is denied with `EVT-J152-CEDAR-DENY-NOT-SITE-LEAD` audit event; the worker can still create a basic incident but cannot pull medical |

## Cross-references

- Persona dossier: `docs/personas/ahmad-hassan.md`
- MASTER-ROSTER §3.2 row 13
- Matrix §10 j152 recommendation
- Existing related journeys: j109 (construction co hires freelance specialist), j110 (multi-employer roster), j114 (employee secondment)
- Pack roster: `packs/us-osha/`, `packs/us-eeoc/`, `packs/iso-45001/`
- ADR-0298 emergency-services bypass life-safety
- ADR-0263 observability emission contract
- ADR-0311 dual-tenant identity (Ahmad-as-site-lead vs Ahmad-as-contractor)
- ADR-0317 role-projection (Ahmad is site-lead AND bilingual interpreter; the journey activates only the site-lead role)
- ADR-0318 collar-color workspace universality (blue-collar field surfaces are first-class, not an afterthought)

## What this journey deliberately does NOT cover

- The post-incident OSHA Area Office investigation interview (separate journey)
- Khalil's workers'-comp benefits payout (downstream from FROI-1)
- The crane manufacturer's product-liability investigation (out of scope)
- Halcyon Build's safety-meeting follow-up the next morning (steady-state journey)

## Stop condition

This journey is complete when all 8 acceptance criteria pass on the seeded test fixture, the schema files validate against their meta-schemas, every named ADR resolves, every named µservice exists in `/microservices/`, and the persona dossier matches MASTER-ROSTER §3.2 row 13.
