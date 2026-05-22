---
doc_class: User-Journey-README
journey_id: j162-print-operator-diana-lazar-night-shift-onboarding
slice: first-night-shift-solo-authorization-onboarding-and-workplace-integration
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Print Operator Diana Lazăr (cross-link to j157)
audience_type: B2B_PRODUCTION_WORKER + B2B_HR_ONBOARDING
microservice_count: 4
pack_overlay_anchor: FOGRA-PSO + ISO-12647-2-print-color + RO-Codul-Muncii-Law-53-2003 + RO-OUG-21-2021-consumer + EU-GDPR + ISO-45001-OHS
related_adrs:
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0243-cedar-as-universal-gate
  - ADR-0263-observability-emission-contract
  - ADR-0252-hlc-default-truetime-tier
  - ADR-0248-amazon-shape-cellular-architecture
  - ADR-0250-build-ahead-of-certification
  - ADR-0247-self-modification-doctrine
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
---

# j162 — Diana Lazăr: first solo night-shift, learning-management gate + workplace-integration

## At a glance

This journey shares its persona with **j157** but follows a different beat of Diana's professional development. Where j157 captured a 9-hour mid-shift quality-recall as a fully-authorized day-shift operator, j162 follows the **onboarding-to-night-shift-solo-authorization** beat: Diana Lazăr — same person, same firm (Tipografia Lazăr-Petrescu SRL, Cluj-Napoca), same Heidelberg Speedmaster CX 102-6+LX press — is being qualified to take over a **first solo night-shift** (22:00–06:30 EET), a step her father Mihai Lazăr-Petrescu (managing director) decided in late 2026 to allow because the firm's growing pharma-PIL workload requires a 2-shift cadence and the existing night-shift operator Vladimir Csikós is reducing his hours by 30% for family reasons.

It is **Tuesday January 26, 2027, 21:18 EET** — six days before the planned first solo run. Diana has spent the prior 4 weeks (since Dec 28, 2026) in a structured onboarding workflow that the `learning-management` µservice has driven through the `tasks` and `workflow-engine` substrate. The competency requirements:

1. **FOGRA-PSO Operator Level 2** — already certified since 2024-09-18, valid through 2027-09-18 (this is the day-shift authority she already holds)
2. **Night-shift solo authorization** — a separate competency requiring 16 hours of additional training + 8 supervised night shifts logged + a final practical assessment with Vladimir as proctor
3. **ISO-45001 night-shift specific OHS module** — Romanian labor-code Law 53/2003 § Title III Chapter II requires explicit night-shift training including fatigue management, ergonomics under low-light conditions, lone-worker protocol, emergency self-rescue
4. **CSN-EN-1837 industrial print-room low-light protocol** — Heidelberg-specific operating procedures for the press in reduced ambient illumination (night-shift pressrooms keep 60% of the day-shift task lighting, except inspection stations which match day)
5. **Workplace-integration provision** — `workplace-integration` µservice provisions her access scope: shift schedule, geofenced clock-in zone, badge unlock priority, on-call escalation chain, payroll night-shift differential, the after-hours alarm-system de-arm scope (Tipografia uses a Securitas alarm with cooperative business-only access; Diana's badge needs to be added to the night-shift roster)

This journey covers the **6-day final phase of her onboarding** (Mon Jan 26 – Sun Feb 1, 2027) from the final competency assessment through her first solo night-shift completing at 06:42 EET Mon Feb 2:

1. **Tue Jan 26 21:18 EET** — Diana completes the final `learning-management` competency module: 90-minute practical assessment on the Heidelberg with Vladimir present as proctor + the firm's external HSE consultant Adriana Stanciu observing. Assessment covers 14 scenarios: cold-startup sequence, mid-run ink-density correction under low-light, paper-jam clearance during low-staffed shift, chemical spill response with lone-worker, emergency-stop drill, etc. Pass criteria: ≥85% per category + Vladimir's qualitative sign-off + Adriana's HSE-compliance sign-off
2. **Tue Jan 26 22:48 EET** — `learning-management` records her assessment scores: 14/14 categories ≥85%; Vladimir's sign-off recorded; Adriana's sign-off recorded; Cedar permit `learning_management.competency_unlock_night_shift_solo` triggers; her competency profile updates
3. **Wed Jan 27 09:00 EET** — `workplace-integration` µservice provisions her night-shift access: shift schedule entry for Mon Feb 1 22:00–06:30 EET; geofenced clock-in zone added; badge updated with night-shift role; Securitas alarm-cooperative scope updated; payroll night-shift differential (+25% per RO labor code §126) enabled
4. **Wed Jan 27 11:42 EET** — `identity` provisions her enhanced passkey for the on-press tablet — biometric (face_id) plus PIN-fallback (no glove-removal required for low-light recognition); she also enrolls a personal escalation contact (her father Mihai's mobile) for lone-worker dead-man protocol
5. **Thu Jan 28 14:18 EET** — Diana receives her first night-shift work-order via `tasks`: `WO-TIP-2027-02-01-NIGHT-WO-NSAID-batch-2` — a continuation of a pharma-PIL batch she's run before (lower risk for a first-solo shift, intentionally chosen by Mihai for graduated exposure)
6. **Sat Jan 30 10:18 EET** — Diana does a 4-hour walk-through with Vladimir at the press during a slow day-shift moment; covers the specific night-shift readiness sequence (alarm de-arm, lighting setup, paper-stock verification for the planned run)
7. **Sun Feb 1 18:42 EET** — Diana arrives home from a half-day Sunday shift; rests until 20:18; her daughter Maria (now age 9) goes to bed; she packs her bag for the night
8. **Mon Feb 1 21:42 EET** — Diana drives to the depot; arrives 21:51; alarm-system de-arm via Securitas (Cedar permit + biometric); enters press hall; lights configuration check; presses-state cold-startup
9. **Mon Feb 1 22:00 EET** — Diana clocks in via `workplace-integration` (geofence + biometric); shift officially starts; off-press operator Andrei Tăbârcă (j157 colleague, also on night-shift rotation) clocks in 30 seconds later
10. **Mon Feb 1 22:18 EET** — Press startup sequence; ink-density baseline reading; substrate loading; first sheets at 22:42 EET
11. **Mon Feb 1 22:42 EET – Tue Feb 2 06:30 EET** — Production run; Diana completes the 8h30m shift with 1 paper-jam (cleared in 14 minutes), 1 ink-density alert (resolved without manager call), 1 dead-man check-in at 02:00 EET (she taps the tablet's "alive" button + face_id within the 60s window; Mihai gets no escalation)
12. **Tue Feb 2 06:30–06:42 EET** — Shift handoff to day-shift operator (her sister-in-law Camelia Lazăr who covers a once-a-week relief); audit chain closure; `EVT-J162-FIRST-NIGHT-SHIFT-COMPLETE-007` seals at 06:42 EET

Primary microservices: `learning-management`, `workplace-integration`, `identity`, `tasks`. Secondary: `tenancy` (firm tenant + RO-state systems + Securitas cooperative tenant), `messenger` (lone-worker escalation), `workflow-engine` (onboarding workflow + shift workflow), `notes` (Diana's onboarding journal + post-shift reflection), `compliance` (RO labor code + ISO-45001 + FOGRA-PSO + ISO-12647-2 + EU-GDPR), `audit-chain`, `crm` (Tipografia's client relationship records — relevant for the pharma client whose batch Diana runs), `observability`, `analytics`.

This is a **gray-collar production-operator onboarding** journey — specifically the **competency-unlock + workplace-integration + first-solo-execution** triad. It demonstrates that oyatie's `learning-management → workplace-integration → identity → tasks` substrate supports a structured graduated-authority pattern where a worker's authority is not a binary flag but a multi-dimensional Cedar context (day-shift FOGRA-PSO L2 authority + night-shift solo authorization + low-light operating protocol + lone-worker dead-man enabled, each a separate Cedar predicate).

## Why this journey matters

Diana Lazăr is the same canonical gray-collar persona as j157 (MASTER-ROSTER §3.4 row 101) but this journey captures her **onboarding-to-next-level** beat. The cross-link to j157 is critical: j157 demonstrates Diana **using** her existing FOGRA-PSO L2 authority to halt a line on her day shift; j162 demonstrates Diana **acquiring** the separate night-shift solo authority through structured competency gating. This persona-replay across journeys is intentional — real workers' lives are not single-incident but a sequence of authority steps over years.

The persona covers an estimated **8.4 million EU industrial production workers** in regulated industries who are mid-career and progressing from day-shift to shift-rotation authority (typically with night-shift premium pay being a key step in wage growth). The category is under-served by HR software (Workday, BambooHR, Personio) that treats competency as a static field rather than a Cedar-evaluable context, and by HSE software (Enablon, EHS Insight, ProcessMAP) that focuses on compliance reporting rather than worker-level competency unlock.

The journey closes:

- **Critical-path row 43** (Competency-gated authority unlock via `learning-management`)
- **Critical-path row 44** (Workplace-integration provisioning across shift schedule + geofence + payroll differential + alarm-cooperative scope)
- **Critical-path row 45** (Lone-worker dead-man protocol with personal-tenant escalation)
- **Critical-path row 46** (Cross-journey persona continuity — Diana's records from j157 inform her j162 onboarding context)
- **Critical-path row 47** (Diacritic + cross-tenant identity preservation including cooperative-business tenant boundaries)

Hyperscaler benchmark: Workday + SAP SuccessFactors + Personio + Enablon + Convergence + Cority. The unique part of oyatie is that **competency authority is a Cedar context, not a flag** — Diana's permit to operate solo at night is computed from `principal.has_competency_unexpired("night-shift-solo-authorization") && context.shift_active_night && principal.workplace_integration_provisioned("night-shift") && principal.dead_man_protocol_enrolled` rather than from a static "is_night_shift_qualified: true" boolean.

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| `story.md` | Beat-by-beat 6-day journey from final assessment through first solo night-shift completion | Cross-link to j157 explicit (same persona, different beat); diacritic-strict Romanian + Hungarian dialogue; specific equipment names (Heidelberg Speedmaster CX 102-6+LX with LE UV); specific FOGRA-PSO + ISO-45001 + ISO-12647-2 + CSN-EN-1837 references; Securitas alarm-cooperative cross-tenant context; lone-worker dead-man timing; Adriana Stanciu HSE consultant + Camelia Lazăr handoff |
| `ux-flow.md` | Diana's Toughpad FZ-G2 on the press (continuing from j157 but with night-shift mode UI overlay); Vladimir's tablet during assessment; Mihai's mobile for emergency escalation; Securitas alarm-cooperative interface; payroll night-shift differential display | Romanian-primary; low-light-mode UI (60% brightness, larger contrast); dead-man check-in big button; lone-worker mode chip in tenant indicator; shift-clock visualization |
| `handshake.md` | Per-µservice API across `tipografia-lazar-petrescu-ro` + `cz-securitas-alarm-cooperative-tenant-ro` (Romanian cooperative tenant) + `ro-ans-labor-state-tenant` (Casa Națională de Asigurări de Sănătate + ANAF) + Adriana Stanciu's external HSE consultant `adriana-stanciu-consulting-ro` + Diana's personal tenant `diana.lazar-petrescu.personal` (lone-worker escalation contact) | Each row names source + target tenant, Cedar permit, cross-tenant audit dual-seal class, diacritic-strict invariant |
| `integration-test-plan.md` | Competency assessment scoring tests + Cedar gated authority unlock tests + workplace-integration provisioning tests + lone-worker dead-man tests + cross-tenant alarm-cooperative tests + first-solo-shift execution tests + cross-journey persona-continuity tests with j157 | Each test names seed values + expected event chain + RO labor code + ISO-45001 invariant probe pass/fail thresholds |
| `schemas/openapi-onboarding-and-night-shift.json` | OpenAPI for competency unlock + workplace-integration provisioning + lone-worker enrollment + shift clock-in endpoints | Competency unlock lifecycle + Cedar context predicates + lone-worker dead-man API + shift-differential payroll |
| `schemas/cedar-policy.cedar` | Competency-gated solo-authority + lone-worker dead-man + cross-tenant alarm-cooperative Cedar policy | Per-competency-class permits + dead-man-window FORBID + alarm-cooperative-scope permits + diacritic-strict mode |
| `schemas/journey-messages.proto` | proto3 for all RPCs | UTF-8 NFC Romanian + Hungarian preserved; competency-assessment proto; lone-worker dead-man proto; shift-clock + payroll-differential proto |
| `schemas/competency-unlock-state-machine.yaml` | 6-state competency-unlock lifecycle | `prereq_pending → training_in_progress → supervised_shifts_logged → final_assessment_scheduled → final_assessment_passed → night_shift_solo_authorized`; Cedar guards per transition |
| `schemas/lone-worker-dead-man-protocol-form.json` | Lone-worker dead-man + emergency-escalation form (ISO-45001 + RO labor code) | Required fields: check-in interval; escalation chain; personal-tenant escalation contact opt-in; equipment-state heartbeat; alarm-cooperative pre-arming |

## The four microservices in scope

| µservice | Role | Critical-path row |
|---|---|---|
| `learning-management` | Drives the 16-hour night-shift training + 8 supervised shifts + final assessment + competency unlock | row 43 |
| `workplace-integration` | Provisions shift schedule + geofenced clock-in + badge role + Securitas alarm-cooperative scope + payroll night-shift differential | row 44 |
| `identity` | Manages Diana's passkey + biometric + lone-worker dead-man protocol enrollment + cross-tenant escalation contact | row 45 |
| `tasks` | Issues her first night-shift work-order + tracks shift task completion + post-shift handoff | rows 44 + 46 |

## Secondary microservices touched

| µservice | Touch reason |
|---|---|
| `tenancy` | Five tenants in scope (Tipografia + Securitas cooperative + RO labor state + Adriana's consulting + Diana's personal escalation tenant) |
| `messenger` | MLS-encrypted lone-worker escalation channel; Vladimir + Diana onboarding thread; family-emergency-only personal channel |
| `workflow-engine` | Onboarding workflow (6-state) + shift workflow (clock-in → shift-active → shift-complete → handoff) |
| `notes` | Diana's onboarding journal (mixed RO/EN with Hungarian phrases from Vladimir); Adriana's HSE observation notes; post-shift reflection |
| `compliance` | Activates RO-Codul-Muncii Law 53/2003 (labor code) + ISO-45001 OHS + FOGRA-PSO + ISO-12647-2 + CSN-EN-1837 (low-light protocol) + EU-GDPR + RO-OUG-21-2021 |
| `audit-chain` | Every competency unlock + workplace-integration update + dead-man check-in + shift transition dual-sealed |
| `crm` | Tipografia's client relationship for the pharma client whose batch Diana runs |
| `observability` | Captures the press telemetry + Diana's dead-man check-in cadence + shift health |
| `analytics` | Per-operator competency progression dashboard; shift-attendance + premium-payout |
| `payments` | Night-shift differential payroll path (paid via Tipografia's standard payroll cycle to Diana's personal HDFC equivalent — Romanian BCR personal account) |

## Pack overlays

| Pack | Activation reason |
|---|---|
| FOGRA-PSO | Diana's day-shift authority (continues from j157) |
| ISO-12647-2 | Print color tolerance reference |
| CSN-EN-1837 | Industrial print-room low-light protocol |
| ISO-45001-2018 | OHS specific to lone-worker + night-shift |
| RO-Codul-Muncii-Law-53-2003 | Romanian labor code §§Title III Chapter II night-shift training + §126 night-shift premium pay |
| RO-OUG-21-2021 | Romanian consumer protection (cross-link to j157 because she runs a pharma-PIL on her first solo shift) |
| EU-GDPR | Personal escalation contact opt-in handling |
| HU-Hungarian-locale-pack | Vladimir code-switches RO/HU per j157; tablet supports both |
| RO-ANAF-tenant | Romanian state tax/labor authority for night-shift differential reporting |

## Regulatory anchors

1. RO-Codul-Muncii Law 53/2003 §§Title III Chapter II (night work) + §126 (night-shift premium)
2. ISO 45001:2018 §6.1.2 (OH&S risk assessment) + §8.1.2 (eliminating + reducing risks)
3. ISO 12647-2:2013 (offset color tolerances)
4. FOGRA PSO 2024 (operator authority schedules)
5. ČSN EN 1837 / RO equivalent (industrial print-room low-light protocol)
6. ADR-0244 tenant scoping
7. ADR-0263 audit dual-seal on cross-tenant transitions
8. ADR-0252 HLC + TrueTime for shift-clock fence
9. ADR-0311 dual-tenant boundary (Diana's professional vs personal escalation tenant)
10. EU-GDPR Articles 6, 7 (personal escalation contact consent)

## Cell + certification matrix

| Cell | Certification | Journey use |
|---|---|---|
| `eu-bucharest-primary` | EU-GDPR + ISO 27001 + ISO 9001 + ISO 45001 + RO-110/2019 equiv | Primary cell for Tipografia tenant (RO data residency, cross-link to j157) |
| `eu-frankfurt-secondary` | EU-GDPR + ISO 27001 | DR replica |
| `eu-amsterdam-readonly-replica` | EU-GDPR | Cross-region read replica for analytics |

## Cedar competency-gated authority policy (excerpt — full text in `schemas/cedar-policy.cedar`)

```cedar
// Competency-gated authority — Cedar gates on competency unlock + active shift class + dead-man protocol
permit (
    principal == User::"diana.lazăr@tipografia-lazar-petrescu-ro",
    action in [
        Action::"press.operate_solo_night_shift",
        Action::"press.cold_startup_solo",
        Action::"press.ink_density_correct_solo",
        Action::"press.emergency_response_solo"
    ],
    resource is PressLine
) when {
    resource.tenant_id == "tipografia-lazar-petrescu-ro" &&
    principal.has_certification_unexpired("FOGRA-PSO-Operator-Level-2") &&
    principal.has_competency_unexpired("night-shift-solo-authorization-2027") &&
    principal.workplace_integration_provisioned == "night-shift" &&
    principal.dead_man_protocol_enrolled == true &&
    context.shift_active_night == true &&
    context.lone_worker_low_light_mode == true
};
```

## Acceptance summary

| AC | Result expected |
|---|---|
| AC-J162-001 | Diana completes final competency assessment Tue Jan 26 22:48 EET; scores 14/14 categories ≥85%; Vladimir + Adriana sign-off; audit `EVT-J162-COMPETENCY-ASSESSED-001` sealed |
| AC-J162-002 | Competency unlock fires Cedar permit `learning_management.competency_unlock_night_shift_solo`; audit `EVT-J162-COMPETENCY-UNLOCKED-002` |
| AC-J162-003 | Workplace-integration provisions night-shift Wed Jan 27 09:00 EET: shift schedule + geofence + badge role + payroll differential + Securitas alarm-cooperative scope; audit `EVT-J162-WORKPLACE-INTEGRATION-PROVISIONED-003` dual-sealed |
| AC-J162-004 | Identity provisions lone-worker dead-man protocol Wed Jan 27 11:42 EET: biometric reconfigured for low-light + PIN fallback; personal escalation contact enrolled with EU-GDPR consent; audit `EVT-J162-DEAD-MAN-ENROLLED-004` |
| AC-J162-005 | First night-shift work-order issued Thu Jan 28; pharma-PIL batch (lower-risk choice for first solo); audit `EVT-J162-FIRST-WO-ISSUED-005` |
| AC-J162-006 | Diana arrives at depot Mon Feb 1 21:51 EET; Securitas alarm de-arm via biometric + Cedar context; audit `EVT-J162-ALARM-DEARMED-006` dual-sealed in `tipografia-lazar-petrescu-ro` AND `cz-securitas-alarm-cooperative-tenant-ro` |
| AC-J162-007 | Diana clocks in Mon Feb 1 22:00 EET via geofence + biometric; shift officially starts; audit `EVT-J162-SHIFT-CLOCK-IN-006a` |
| AC-J162-008 | Dead-man check-in 02:00 EET completes within 60-second window via face_id + tablet tap; audit `EVT-J162-DEAD-MAN-CHECKIN-006b` |
| AC-J162-009 | Shift completes 06:30 EET with 1 paper-jam resolved + 1 ink-density alert resolved + 1 dead-man check-in; audit `EVT-J162-FIRST-NIGHT-SHIFT-COMPLETE-007` sealed at 06:42 EET |
| AC-J162-010 | Shift handoff to Camelia Lazăr at 06:42 EET; audit `EVT-J162-SHIFT-HANDOFF-008` |
| AC-J162-011 | Night-shift premium payroll +25% applied per RO §126; audit `EVT-J162-NIGHT-PREMIUM-PAID-009` |
| AC-J162-012 | Diacritic fidelity: "Diana Lazăr", "Camelia Lazăr", "Adriana Stanciu", "Mihai Lazăr-Petrescu", "Vladimir Csikós", "Andrei Tăbârcă" preserve UTF-8 NFC across all persisted fields; no Romanization in legal/regulator fields |
| AC-J162-013 | Cross-journey persona continuity: Diana's j157-sealed events (FOGRA-PSO L2 cert + day-shift authority) inform j162 onboarding context (cert verified during night-shift competency assessment); no duplicate cert capture |
| AC-J162-014 | Lone-worker dead-man personal escalation: simulated dead-man miss → escalation to Mihai's mobile within 90s; Cedar context validates `personal_tenant_escalation_consent_active` |

## Cross-references

- Persona dossier: `docs/personas/print-operator-diana-lazar.md` (same persona as j157)
- MASTER-ROSTER §3.4 row 101 (cross-journey continuation)
- Matrix §14 j162 recommendation
- Related: j157 (mid-shift quality recall - Diana's day-shift authority demo), j158 (cell rebalance), j155 (gray-collar dual-role), j156 (gray-collar after-hours emergency)
- Pack roster: `packs/fogra-pso/`, `packs/iso-12647-2/`, `packs/iso-45001/`, `packs/ro-codul-muncii-53-2003/`, `packs/csn-en-1837/`, `packs/eu-gdpr/`, `packs/hu-locale-pack/`
- ADR-0244 tenant scoping
- ADR-0263 audit dual-seal
- ADR-0252 HLC + TrueTime fence
- ADR-0311 dual-tenant boundary (Diana's personal escalation contact)

## Stop condition

This journey is complete when all 14 acceptance criteria pass on the seeded multi-tenant fixture, the competency unlock chain (assessment → unlock → workplace-integration → dead-man enrollment → first work-order → shift clock-in → shift complete → handoff) executes without intervention, cross-journey persona continuity with j157 is preserved, the diacritic + Romanian + Hungarian + Cyrillic fidelity invariant holds across every persisted field, the lone-worker dead-man protocol's 60-second window functions correctly, and Diana's first solo night-shift completes at 06:42 EET Tue Feb 2 2027 with the night-shift premium payroll correctly applied.
