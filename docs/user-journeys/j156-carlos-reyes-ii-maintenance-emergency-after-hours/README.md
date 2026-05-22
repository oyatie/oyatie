---
doc_class: User-Journey-README
journey_id: j156-carlos-reyes-ii-maintenance-emergency-after-hours
slice: hvac-emergency-after-hours-incident-mobile-workflow-permit-to-work
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Facility Maintenance Technician Carlos Reyes II
audience_type: B2B_FIELD_WORKER + B2B_FACILITY_OPERATIONS
microservice_count: 5
pack_overlay_anchor: OSHA-29-CFR-1910 + NFPA-70E + ASHRAE-15-refrigerant-safety + ISO-41001-FM + US-DOL-overtime
related_adrs:
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0243-cedar-as-universal-gate
  - ADR-0263-observability-emission-contract
  - ADR-0248-amazon-shape-cellular-architecture
  - ADR-0252-hlc-default-truetime-tier
  - ADR-0250-build-ahead-of-certification
  - ADR-0254-kubernetes-everywhere-pods-cloud-hypervisor
  - ADR-0247-self-modification-doctrine
---

# j156 — Carlos Reyes II: 2:47 AM HVAC emergency at DC-PHX-3

## At a glance

Carlos Reyes II is a **certified facility maintenance technician** (EPA 608 Universal + NATE Senior + OSHA 30) employed by **Cascade Facility Services LLC** (a B2B FM contractor) on a 24/7 on-call rotation covering data-center HVAC + power for **MeridianStack Hosting** (a regional colo operator with 6 data centers in the US Southwest). He is 41 years old, second-generation Mexican-American (his father, Carlos Reyes Sr., taught him refrigeration in Yuma in 1998), bilingual EN/ES, lives in Glendale AZ with his wife Yesenia and two children, and has been Cascade's "go-to" for after-hours emergencies at MeridianStack's flagship facility, **DC-PHX-3** (Phoenix, AZ; 86,400 sq ft; 18 MW IT load).

It is **Saturday October 17, 2026, 02:47 MST** (Phoenix does not observe DST). The Sonoran-summer-overrun has Phoenix at 97°F at 02:47 AM. Carlos is asleep in his bed when his oyatie-issued **Samsung XCover7 Pro** vibrates with a P1 page from `incident-management`:

> 🚨 **P1-HVAC** DC-PHX-3 aisle 7B chiller-loop overtemp · ΔT inlet/outlet 14.2°F (cap 6.0°F) · 4 racks at 88°F intake · auto-shed in **11 min 47 sec** if uncorrected · respond Y/N

This journey covers the next 6 hours and 18 minutes of Carlos's life:

1. **Page acknowledgment** through the `incident-management` µservice within the **Cascade tenant** (`cascade-fm-services-llc-us`) — Cedar permit `incident.acknowledge` against the MeridianStack cross-tenant principal
2. **Cross-tenant grant**: MeridianStack's after-hours dispatch (`meridianstack-hosting-co-us`) auto-issues Carlos a **scoped maintenance principal** for DC-PHX-3 valid 02:47 → 09:00 MST under ADR-0311 dual-tenant identity boundary
3. **Mobile permit-to-work** via `workflow-engine` — Carlos receives an electronic permit covering EPA-608 refrigerant handling + NFPA-70E arc-flash boundary for the 480V chilled-water pump 7B-PUMP-04; the permit is **co-signed digitally** by Cascade's on-call manager Tomás Alvarado at 02:51 MST and by MeridianStack's NOC controller Priya Subramanian at 02:53 MST
4. **Tasks µservice** materializes 11 atomic tasks (drive-to-site, badge-in, ladder-setup, lockout-tagout, condensate-line-inspection, pump-rebuild, refrigerant-recovery, post-leak-test, re-energize, log-in-CMMS, sign-permit-closeout); each task records start/end + photo evidence + Cedar permit context
5. **Audit-chain** captures every action with **per-action merkle anchoring**: at every chiller-loop probe, every valve actuation, every pressure reading, the audit-event seal is a P0 governance event because this facility serves PII workloads for a healthcare-integration µservice customer (HIPAA-covered)
6. **Messenger** carries the encrypted ops thread between Carlos, Tomás (Cascade manager), Priya (MeridianStack NOC), and (escalated at 03:34 MST when Carlos discovers the chiller-loop leak is **>1 lb of R-454B refrigerant**) the EPA Section 608 Class IV vendor — Trane Technologies' factory-emergency line — for a per-pound disclosure under 40 CFR Part 82 Subpart F

Microservices in scope: `incident-management`, `tasks`, `messenger`, `audit-chain`, `workflow-engine`. Secondary: `identity`, `tenancy`, `compliance` (OSHA + NFPA + ASHRAE + EPA-608 packs), `observability`, `learning-management` (auto-pulls Carlos's most recent EPA refresher certificate), `workplace-integration` (pulls Carlos's overtime eligibility from Cascade's payroll), `plant-maintenance` (the CMMS the post-mortem records to), `network` (cross-tenant routing).

This is a **gray-collar, after-hours, cross-tenant, regulator-touching emergency** journey. It demonstrates that oyatie's `incident-management → tasks → workflow-engine` triad can drive a 6-hour operations response with regulator-grade audit trails AND mobile-first interaction AND cross-tenant Cedar permits, all without Carlos ever touching a desktop.

## Why this journey matters

Carlos Reyes II is **MASTER-ROSTER §3.4 row 100** — the canonical gray-collar field-service technician persona. The persona covers an estimated 8.4 million US workers in facility maintenance, HVAC, refrigeration, electrical, plumbing, and elevator trades. They are routinely under-served by enterprise software because most CMMS / FSM products assume desktop-first interaction; mobile is a bolt-on, the photo-evidence pipeline is brittle, and cross-tenant permits (contractor → customer site) require manual paper or PDF.

The journey closes:

- **Critical-path row 18** (After-hours mission-critical operations with mobile-first device)
- **Critical-path row 19** (Cross-tenant regulator-grade audit for facility incidents)
- **Critical-path row 20** (Permit-to-work as a Cedar-enforced cross-tenant grant)
- **Critical-path row 23** (Refrigerant-recovery EPA disclosure with per-pound provenance)

Hyperscaler benchmark: ServiceMax + Salesforce Field Service + Microsoft Dynamics Field Service + IBM Maximo + IFS Field Service Management + Trane Connect + Johnson Controls OpenBlue. The unique part of oyatie is that **the incident-management → tasks → workflow-engine → audit-chain pipeline is single-tenant by default but lights up cross-tenant grants surgically** under Cedar (ADR-0311), so Carlos's Cascade-tenant principal can act inside MeridianStack's tenant for exactly the 6h18m he's needed and not one minute longer.

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| `story.md` | Beat-by-beat 02:47 MST page → 09:05 MST sign-off | Specific minute-by-minute, named rooms (DC-PHX-3 aisle 7B rack-row 124–138), specific equipment (7B-PUMP-04 Trane RTAF-200), specific refrigerant (R-454B), specific dollar amounts ($1,247 emergency-call rate), specific certificate IDs |
| `ux-flow.md` | Samsung XCover7 Pro screens, the headlamp-friendly dark-emergency theme, voice-driven hand-free entry while wearing arc-flash PPE | Active-tenant pill behavior; permit signature drawer; LOTO interlock screens |
| `handshake.md` | Per-µservice API + per-tenant scoping | Each row names source tenant + target tenant + Cedar permit + cross-tenant grant lifecycle |
| `integration-test-plan.md` | Permit-to-work tests + LOTO tests + refrigerant-disclosure tests + cross-tenant audit dual-seal tests + offline-resilience tests | Each test names seed values + expected event chain + pass/fail thresholds + chaos scenarios |
| `schemas/openapi-incident-and-permit.json` | OpenAPI for incident → permit endpoints | Permit + LOTO + co-sign endpoints |
| `schemas/cedar-policy.cedar` | Cross-tenant maintenance permit policy | Cascade-tech-acting-in-MeridianStack scope + time-window + LOTO-required gates |
| `schemas/journey-messages.proto` | proto3 for the RPCs | 12 messages, including the EPA-608 disclosure |
| `schemas/loto-state-machine.yaml` | Lockout-tagout state machine | 9 states + 14 transitions + per-state Cedar guard |
| `schemas/refrigerant-disclosure-form-40cfr82-f.json` | EPA Section 608 disclosure schema | Per-pound + cylinder-tracking + Class IV vendor binding |

## The five microservices in scope

| µservice | Role | Critical-path row |
|---|---|---|
| `incident-management` | Owns the P1 page, the SLA timer (auto-shed in 11:47), the escalation tree, the post-mortem | row 18, row 19 |
| `tasks` | Materializes the 11 atomic tasks; each task carries photo evidence + Cedar context | row 18 |
| `messenger` | MLS-encrypted ops thread: Carlos ↔ Tomás (manager) ↔ Priya (NOC); escalation channel for Trane vendor | row 18 |
| `audit-chain` | Per-action merkle anchor; HIPAA-grade retention; dual-seal on cross-tenant transitions | row 19 |
| `workflow-engine` | Mobile permit-to-work workflow; LOTO state machine; co-sign collection; EPA disclosure form | row 20, row 23 |

## Secondary microservices touched

| µservice | Touch reason |
|---|---|
| `identity` | Carlos's passkey root + Cascade employee mapping + scoped MeridianStack principal grant |
| `tenancy` | Two tenants in scope (Cascade + MeridianStack); both audit streams cross-trace |
| `compliance` | Activates OSHA-29-CFR-1910, NFPA-70E-arc-flash, ASHRAE-15-refrigerant, EPA-608, US-DOL-overtime |
| `observability` | Captures the chiller-loop telemetry stream Priya reads alongside Carlos |
| `learning-management` | Pulls Carlos's most-recent EPA-608 + NFPA-70E refresher certs to populate the permit |
| `workplace-integration` | Pulls Carlos's Cascade payroll overtime eligibility + emergency-call rate ($1,247 base + $87.50/hr after first 2 hrs) |
| `plant-maintenance` | CMMS where the closed-out work order persists; consumed by MeridianStack's monthly facility-ops report |
| `network` | Cross-tenant routing (Cascade ⇄ MeridianStack) over private peering link `peer-cascade-meridianstack-az-1` |
| `consent-graph` | Carlos's photo-evidence consent (any face-capture during the response triggers the consent gate) |

## Pack overlays

| Pack | Activation reason |
|---|---|
| OSHA-29-CFR-1910 | General industry standards; LOTO §1910.147; arc-flash boundary §1910.333; PPE §1910.132 |
| NFPA-70E | Arc-flash + electrical-safety in the workplace; PPE category for 480V class boundaries |
| ASHRAE-15 | Refrigerant safety; mechanical-room ventilation; leak detection |
| EPA-608-40-CFR-82-F | Refrigerant recovery, leak repair, disclosure; per-pound tracking for releases ≥1 lb |
| ISO-41001-FM | Facility-management management-system requirements; audit basis for the post-mortem |
| US-DOL-overtime | Carlos's hours and overtime computation; emergency-call premium |
| HIPAA-FAC | Facility-controls for HIPAA-covered tenants (MeridianStack hosts a covered entity) |

## Regulatory anchors

1. 29 CFR 1910.147 — Lockout/Tagout (one of the OSHA "fatal-four" controls)
2. 29 CFR 1910.333 — Selection and use of work practices for electrical safety
3. NFPA 70E-2024 — Standard for electrical safety in the workplace; arc-flash boundary calculation
4. ASHRAE Standard 15-2022 — Safety Standard for Refrigeration Systems
5. 40 CFR Part 82 Subpart F — Recycling and Emissions Reduction; per-lb release disclosure ≥1 lb
6. 45 CFR Part 164 — HIPAA Security Rule (technical + physical safeguards include facility controls)
7. ADR-0311 dual-tenant identity boundary (Cascade tech acts inside MeridianStack tenant for the 6h18m window)
8. ADR-0244 tenant scoping (two tenants, two audit streams, one cross-tenant trace_id)
9. ADR-0248 cellular architecture (DC-PHX-3 itself is a Tier-1 cell; the incident is cell-scoped)
10. ADR-0250 build-ahead-of-certification (HIPAA + EPA-608 + OSHA gates active day-one)

## Cell + certification matrix

| Cell | Certification | Journey use |
|---|---|---|
| `us-phoenix-edge-tier1` | SOC2-Type-II + HIPAA + ISO 27001 | Primary cell for DC-PHX-3 and MeridianStack tenant |
| `us-phoenix-control-plane` | SOC2-Type-II + HIPAA | Cross-tenant routing, Cedar evaluation |
| `us-las-vegas-secondary` | SOC2-Type-II + HIPAA | DR replica for MeridianStack |
| `us-portland-cascade-home` | SOC2-Type-II | Cascade FM Services tenant home |

## Cedar cross-tenant permit (excerpt — full text in `schemas/cedar-policy.cedar`)

```cedar
// Cascade tech acting inside MeridianStack tenant for the 6h18m window
permit (
    principal == User::"carlos.reyes-ii@cascade-fm-services-llc-us",
    action in [
        Action::"incident.acknowledge",
        Action::"incident.resolve",
        Action::"tasks.execute",
        Action::"workflow.permit_sign",
        Action::"workflow.loto_lock",
        Action::"workflow.loto_release",
        Action::"plant.cmms_close_workorder"
    ],
    resource is Tenant
) when {
    resource.tenant_id == "meridianstack-hosting-co-us" &&
    principal.cross_tenant_grant_active(
        host_tenant = "meridianstack-hosting-co-us",
        scope = "dc-phx-3-aisle-7b",
        valid_from = "2026-10-17T02:47:00-07:00",
        valid_until = "2026-10-17T09:00:00-07:00"
    ) &&
    principal.has_certification("EPA-608-Universal") &&
    principal.has_certification("NFPA-70E-CAT-2") &&
    principal.has_certification("OSHA-30-General-Industry") &&
    context.permit_id like "permit-dc-phx-3-2026-10-17-*" &&
    context.permit_status in ["co_signed_active", "post_repair_verification"]
};
```

## Acceptance summary

| AC | Result expected |
|---|---|
| AC-J156-001 | Carlos acknowledges P1 page within 90s; audit `EVT-J156-INCIDENT-ACK-001` sealed in BOTH tenants |
| AC-J156-002 | MeridianStack issues scoped cross-tenant principal grant valid 02:47 → 09:00 MST; audit `EVT-J156-IDENTITY-CROSS-GRANT-002` |
| AC-J156-003 | Permit-to-work co-signed by Tomás (Cascade manager) and Priya (MeridianStack NOC) within 6 min of page; audit `EVT-J156-WORKFLOW-PERMIT-COSIGN-003` |
| AC-J156-004 | LOTO state machine reaches `locked_isolated_verified` before any 480V work begins; audit `EVT-J156-LOTO-LOCKED-004` |
| AC-J156-005 | 11 tasks each carry photo + GPS + Cedar context; audit per-task |
| AC-J156-006 | R-454B leak >1 lb triggers EPA disclosure workflow; audit `EVT-J156-EPA608-DISCLOSURE-006` |
| AC-J156-007 | Cross-tenant audit dual-seal invariant holds — every Carlos action seals in BOTH Cascade and MeridianStack |
| AC-J156-008 | At 09:00 MST exactly, Carlos's cross-tenant grant auto-expires; subsequent action attempts deny |
| AC-J156-009 | Carlos's overtime hours (6h18m emergency-call) post to Cascade payroll within 1 hr of closeout |
| AC-J156-010 | HIPAA facility-control audit produces a daily-roll-up with merkle proof for MeridianStack's covered-entity customer |

## Cross-references

- Persona dossier: `docs/personas/facility-maintenance-tech-carlos-reyes-ii.md`
- MASTER-ROSTER §3.4 row 100
- Matrix §10 j156 recommendation
- Related: j155 (gray-collar dual-role), j112 (B2B contractor relationship), j124 (supply-chain disruption emergency)
- Pack roster: `packs/osha-1910/`, `packs/nfpa-70e/`, `packs/ashrae-15/`, `packs/epa-608/`, `packs/iso-41001/`
- ADR-0311 dual-tenant identity boundary
- ADR-0244 tenant scoping
- ADR-0263 audit dual-seal on cross-tenant transitions

## Stop condition

This journey is complete when all 10 acceptance criteria pass on the seeded two-tenant fixture, the 11 tasks materialize with photo evidence, the LOTO state machine reaches `closed_post_verification`, the EPA-608 disclosure delivers within the statutory 30-day window, the cross-tenant grant auto-expires at 09:00 MST, and the HIPAA daily-roll-up audit produces merkle proofs for the MeridianStack covered-entity report.
