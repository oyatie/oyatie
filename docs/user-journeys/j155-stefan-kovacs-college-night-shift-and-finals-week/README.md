---
doc_class: User-Journey-README
journey_id: j155-stefan-kovacs-college-night-shift-and-finals-week
slice: dual-role-student-and-night-shift-worker-finals-week
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Security Guard Stefan Kovács
audience_type: B2B_FIELD_WORKER + B2C_CONSUMER + EDU_STUDENT
microservice_count: 5
pack_overlay_anchor: EU-GDPR + HU-Labour-Code + EU-Working-Time-Directive + Bologna-Process-academic-records
related_adrs:
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0243-cedar-as-universal-gate
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0317-role-projection-doctrine
  - ADR-0263-observability-emission-contract
  - ADR-0292-minor-and-vulnerable-user-doctrine
  - ADR-0252-hlc-default-truetime-tier
  - ADR-0254-kubernetes-everywhere-pods-cloud-hypervisor
---

# j155 — Stefan Kovács: College finals week and the night shift

## At a glance

Stefan Kovács is a **second-year Computer Engineering student** at the Budapest University of Technology and Economics (Budapesti Műszaki és Gazdaságtudományi Egyetem, BME — affectionately "Műegyetem") and a **part-time overnight security guard** at the Hungarian National Library (Országos Széchényi Könyvtár, OSZK) in Buda Castle district VIII. He is 20 years old, a Hungarian citizen, lives with two roommates in a panel-block apartment in Újpest, and is two semesters away from graduation if he can pass the four hardest courses on the BME catalogue: **Computer Architecture II, Operating Systems, Discrete Mathematics II, and Signals and Systems**.

Today is **Sunday December 14, 2026, 21:48 CET**. Stefan has just clocked in for his weekend overnight shift (22:00 → 06:00) at the OSZK rare-books reading-room foyer. His Operating Systems final is **Tuesday December 16, 08:00 CET** — 34 hours away. His Discrete Mathematics II final is **Friday December 19, 08:00 CET**. He must:

1. **Confirm shift attendance** through the `calendar` µservice in his **OSZK work tenant** (`oszk-security-services_hu`)
2. **Receive a shift-swap offer** from his coworker Réka Hahn, who is sick and wants Stefan to take her Tuesday-night shift — except Tuesday is his OS final + Wednesday is his recovery day; he must **decline politely** through messenger inside the OSZK tenant without leaking personal context
3. **Pull up his Operating Systems study notes** via the `learning-management` µservice in his **BME student tenant** (`bme-student-bodv75_hu`) — these are tied to his student-account identity, not his employee identity
4. **Watch the OSZK foyer entrance** through the building access-control feed, which is part of his work-tenant duties; if any incident occurs, he must report it via the OSZK incident-management channel WITHOUT revealing personal study material on the same device
5. **Pay his BME tuition installment** of HUF 187,500 via `payments` from his personal tenant (`personal-stefan-kovacs-hu`), which is funded by his OSZK paycheck through a **payroll-deduction arrangement** that the HR system (workplace-integration → ADP-Streamline-HU) negotiates between the two tenants. Stefan never sees the money; it flows OSZK → BME via the payroll bridge
6. **Engage with the BME student-community** µservice during the night shift's downtime — specifically the `#os-finals-2026` study channel where 47 classmates are pooling past-exam questions — but only when no shift duty is active; OSZK's policy permits up to 2 hours of personal-device use during a 22:00–06:00 shift if all access-control alarms are green
7. **Have his sleep-grade correlation telemetry** captured by `observability` under a **student-wellness research program** he opted into in semester 1 — the data flows into BME's anonymous-cohort analytics, NEVER into OSZK's view

Microservices: `calendar`, `learning-management`, `payments`, `community`, `observability`. Secondary touches: `identity` (dual tenant binding), `tenancy` (three tenants: personal + work + student), `messenger` (Réka's swap offer), `workplace-integration` (the payroll bridge between OSZK and BME), `incident-management` (if an alarm fires), `audit-chain`, `compliance` (EU-GDPR + HU-Labour + EU-WTD + Bologna academic-records).

This is a **dual-role identity** journey: Stefan-as-student and Stefan-as-employee are the same passkey-bound human but live in distinct tenants with surgical Cedar boundaries (ADR-0311). The journey demonstrates that the **same device** (his Pixel 8a phone + his Lenovo IdeaPad work laptop) can present both contexts without bleed, with the active-tenant indicator preventing accidental disclosure (e.g. the OSZK admin must NEVER see that Stefan is studying for an OS final on his break; BME's professor must NEVER see Stefan's employment record).

## Why this journey matters

Stefan Kovács is **MASTER-ROSTER §3.4 row 99** — the canonical gray-collar working-student persona. This persona represents a category that comprises 14% of the EU undergraduate workforce per Eurostat 2025: students who are not financially independent and not financially supported, who hold sub-25-hour-per-week night-shift jobs because day shifts conflict with classes. The category is acutely under-served by enterprise software because most products assume either a "student" or "employee" identity, not both.

The journey closes:

- **Critical-path row 14** (Low-bandwidth + commute-zone offline-first — Stefan often loses connectivity in the OSZK basement security room)
- **Critical-path row 17** (Regulator-deadline outage — the EU Working Time Directive caps weekly hours; if OSZK's HR system is down, Stefan's hours-tracking must continue via the workflow-engine fallback)
- **Critical-path row 22** (Disaster-zone surge — semi-rare but real; the 2024 Danube flood forced OSZK to extend overnight staffing; Stefan picked up extra shifts and oyatie had to keep the dual-tenant boundary clean even under operational stress)

Hyperscaler benchmark: Workday Student + Workday Workforce; Oracle Cloud HCM + Oracle Student Cloud; ADP Workforce Now + Canvas LMS; Microsoft Education + Microsoft Viva; Time-clock platforms like Deputy + Kronos for night-shift workers; the unique part of oyatie is that **all of these are the same human projection**, not federated identities.

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| `story.md` | Beat-by-beat Sunday 21:48 CET (clock-in) through Friday 13:00 CET (final exam ends) | Specific shift events, specific exam questions, specific HUF amounts, specific BME course codes (VIK-AUT-VIIIAB1015, VIK-AUT-VIIMA9302, VIK-AUT-VIIIAB1015) |
| `ux-flow.md` | Three-screen contexts — Pixel 8a (work alerts) + IdeaPad (study) + the OSZK kiosk + tenant-switcher mechanics | Active-tenant pill behavior, study-mode banner, low-light night-shift theme, the HU/EN locale switch |
| `handshake.md` | Per-µservice API + per-tenant scoping | Each row includes source tenant + target tenant + Cedar dual-role decision + the payroll-bridge handoff |
| `integration-test-plan.md` | Dual-role boundary tests + shift-attendance tests + tuition-payroll-bridge tests + sleep-telemetry partition tests | Each test names the seed values + the expected boundary behavior + Cedar deny on cross-tenant probes |
| `schemas/openapi-shift-attendance.json` | OpenAPI for OSZK's shift confirm/swap endpoint | Per-shift Cedar permit context |
| `schemas/openapi-tuition-payroll-bridge.json` | OpenAPI for the OSZK → BME payroll-deduction bridge | Three-way handshake including stefan's personal-tenant consent |
| `schemas/journey-messages.proto` | proto3 for the 9 RPCs | Field tags, enum values, dual-role principal context |
| `schemas/cedar-policy.cedar` | Dual-role Cedar policy | Stefan-as-student vs Stefan-as-employee permits + DPO observer |
| `schemas/sleep-grade-correlation-pipeline.yaml` | Observability pipeline for the student-wellness research | Per-event anonymization rules + cohort-only egress |

## The five microservices in scope

| µservice | Role | Critical-path row |
|---|---|---|
| `calendar` | Holds Stefan's shift schedule (work tenant) + his exam schedule (student tenant); the two are NEVER joined cross-tenant | row 14 |
| `learning-management` | BME student tenant; serves Stefan's OS lecture notes, past-exam library, and the `#os-finals-2026` study channel index | row 24 |
| `payments` | Personal tenant; receives the OSZK paycheck via SEPA and dispatches the BME tuition payment via the same bridge | row 15 (financial inclusion) |
| `community` | BME student tenant; hosts the `#os-finals-2026` channel where 47 classmates pool study questions; MLS-encrypted | row 24 |
| `observability` | Captures sleep-grade correlation telemetry per Stefan's opt-in; egresses ONLY to BME's anonymous-cohort analytics; cannot be queried by OSZK | row 21 (pseudonymity) |

## Secondary microservices touched

| µservice | Touch reason |
|---|---|
| `identity` | Stefan has one passkey root; three tenant memberships (personal + work + student); active-tenant switcher confirms every transition |
| `tenancy` | Three tenants: `personal-stefan-kovacs-hu`, `oszk-security-services_hu`, `bme-student-bodv75_hu`. Three lifecycles, three audit streams |
| `messenger` | Réka Hahn's shift-swap offer arrives via OSZK-tenant messenger; Stefan's decline stays inside the OSZK tenant |
| `workplace-integration` | The OSZK→BME payroll-deduction bridge (HU's ADP-Streamline-HU API) |
| `incident-management` | Reserved for the (improbable) overnight alarm at the OSZK foyer |
| `audit-chain` | Seals all consequential events; per-tenant retention; cross-tenant trace_id |
| `compliance` | Activates EU-GDPR, HU-Labour-Code, EU-Working-Time-Directive, Bologna-academic-records packs |
| `analytics` | BME's anonymous cohort dashboard reads sleep-grade correlation; OSZK has NO view |

## Pack overlays

| Pack | Activation reason |
|---|---|
| EU-GDPR | Stefan is an EU resident; both work and student tenants process personal data under Art 6 |
| HU-Labour-Code | Munka Törvénykönyve §99–§102 caps weekly hours; §110 governs night-shift premium pay; §103 governs minimum rest |
| EU-Working-Time-Directive | 2003/88/EC limits 48 hr/week + minimum 11 hr daily rest + maximum 8 hr in any 24 hr night-work window for workers under 18; Stefan is 20 so the night-work cap doesn't bite, but the rest minimums do |
| Bologna-academic-records | The EHEA framework for portable transcripts; Stefan's BME academic record is portable to any EHEA institution if he transfers |
| HU-Education-Act-CXC-2011 | Hungarian higher-education statute governing tuition, enrolment, examinations |

## Regulatory anchors

1. EU-GDPR Art 6 (lawful basis for OSZK and BME), Art 9 (no special-category data flow OSZK↔BME), Art 26 (joint-controller mapping if any sleep-research cohort is shared with a third-party EHEA partner, which it isn't here), Art 88 (employment context special protections)
2. HU-Munka-Törvénykönyve §99–§110 (working time, rest, night-shift premium)
3. EU-Working-Time-Directive 2003/88/EC (rest minimums, weekly cap)
4. HU-Nemzeti-Köznevelési-Törvény-CXC-2011 (higher-education statute — tuition + examinations)
5. ADR-0311 dual-tenant identity boundary (work data never reaches student tenant; student data never reaches work tenant)
6. ADR-0317 role-projection doctrine (the same human can be student OR employee at any moment; never both simultaneously in the same context)
7. ADR-0292 minor / vulnerable-user doctrine (Stefan is 20 so not in the minor cohort, but the doctrine's "non-financially-independent young adult" lens applies — slow-down nudges on irreversible payments)
8. ADR-0244 tenant scoping (three tenants, three audit streams, three retention policies)
9. ADR-0252 HLC + TrueTime tier (HLC for routine; payroll bridge uses TrueTime-class for the cross-tenant transfer ordering invariant)

## Cell + certification matrix

| Cell | Certification | Journey use |
|---|---|---|
| `eu-frankfurt-primary` | EU-GDPR-ready + ISO 27001 + EU-Cloud-Code-of-Conduct | Primary for OSZK; HR data residency |
| `eu-amsterdam-secondary` | EU-GDPR-ready | Primary for personal-stefan-kovacs-hu and bme-student-bodv75_hu |
| `eu-paris-readonly-replica` | EU-GDPR-ready | Read replica for resilience |

## Cedar dual-role policy (excerpt — full text in `schemas/cedar-policy.cedar`)

```cedar
// Stefan as OSZK employee — work-tenant scope
permit (
    principal == User::"stefan.kovacs@personal-id.oya",
    action in [
        Action::"calendar.confirm_shift",
        Action::"messenger.read_work",
        Action::"messenger.send_work",
        Action::"incident.report",
        Action::"workplace.clock_in",
        Action::"workplace.clock_out"
    ],
    resource is Tenant
) when {
    resource.tenant_id == "oszk-security-services_hu" &&
    principal.role_in_tenant("oszk-security-services_hu") == "night_shift_guard" &&
    context.active_tenant == "oszk-security-services_hu"
};

// Stefan as BME student — student-tenant scope
permit (
    principal == User::"stefan.kovacs@personal-id.oya",
    action in [
        Action::"lms.read_notes",
        Action::"lms.read_past_exams",
        Action::"community.post_student",
        Action::"community.read_student",
        Action::"payments.tuition_pay"
    ],
    resource is Tenant
) when {
    resource.tenant_id == "bme-student-bodv75_hu" &&
    principal.role_in_tenant("bme-student-bodv75_hu") == "active_student" &&
    context.active_tenant == "bme-student-bodv75_hu"
};

// CRITICAL FORBID — OSZK cannot read BME tenant
forbid (
    principal,
    action,
    resource is Tenant
) when {
    resource.tenant_id == "bme-student-bodv75_hu" &&
    principal.acting_tenant == "oszk-security-services_hu"
};

// CRITICAL FORBID — BME cannot read OSZK tenant
forbid (
    principal,
    action,
    resource is Tenant
) when {
    resource.tenant_id == "oszk-security-services_hu" &&
    principal.acting_tenant == "bme-student-bodv75_hu"
};
```

## Acceptance summary

| AC | Result expected |
|---|---|
| AC-J155-001 | Stefan confirms his 22:00 shift via the work-tenant calendar; audit `EVT-J155-CALENDAR-SHIFT-CONFIRM-001` seals in `oszk-security-services_hu` ONLY |
| AC-J155-002 | Stefan receives Réka's swap offer via OSZK messenger; declines without exposing exam schedule; audit `EVT-J155-MESSENGER-SWAP-DECLINED-002` |
| AC-J155-003 | Stefan switches tenant from OSZK to BME with explicit 2-second hold; audit `EVT-J155-IDENTITY-TENANT-SWITCH-003` |
| AC-J155-004 | Stefan reads OS lecture notes from BME LMS; OSZK can never see this read; audit `EVT-J155-LMS-NOTES-READ-004` (BME tenant only) |
| AC-J155-005 | Cedar denies OSZK admin attempt to query Stefan's BME study activity; `EVT-J155-CEDAR-DENY-CROSS-TENANT-LMS-PROBE-005` |
| AC-J155-006 | Tuition payment HUF 187,500 flows OSZK paycheck → bridge → BME via workplace-integration; the bridge is Cedar-gated through three tenants; audit `EVT-J155-PAYMENTS-TUITION-PAYROLL-BRIDGE-006` |
| AC-J155-007 | Stefan posts to `#os-finals-2026` MLS-encrypted community channel; only BME peers see it; OSZK has no visibility; audit `EVT-J155-COMMUNITY-POST-STUDENT-007` |
| AC-J155-008 | Sleep-grade correlation telemetry captured in observability; egresses to BME anonymous-cohort analytics ONLY; never queryable by OSZK; audit `EVT-J155-OBSERVABILITY-SLEEP-GRADE-EMIT-008` |
| AC-J155-009 | EU-WTD weekly-cap evaluator green; Stefan's 22-week running average is 22 hr/week (well under 48 hr cap) |
| AC-J155-010 | Finals-week mode (Mon-Fri Dec 14–19) auto-pauses non-emergency notifications across both tenants; only OSZK alarms break through |

## Cross-references

- Persona dossier: `docs/personas/security-guard-stefan-kovacs.md`
- MASTER-ROSTER §3.4 row 99
- Matrix §10 j155 recommendation
- Related: j127 (dual-tenant identity employee resigns and keeps personal), j150 (gig economy minor safety; complementary "young earner" lens), j143 (portfolio import after layoff)
- Pack roster: `packs/eu-gdpr/`, `packs/hu-labour/`, `packs/eu-wtd/`, `packs/bologna/`
- ADR-0311 dual-tenant identity boundary
- ADR-0317 role-projection doctrine
- ADR-0252 HLC + TrueTime tier (payroll bridge uses TrueTime-class)

## Stop condition

This journey is complete when all 10 acceptance criteria pass on the seeded test fixture, the schema files validate, every named ADR resolves, every named µservice exists in `/microservices/`, and the persona dossier matches MASTER-ROSTER §3.4 row 99.
