---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j162-print-operator-diana-lazar-night-shift-onboarding
date: 2026-05-20
authority_tier: 2
status: draft
---

# j162 — Integration test plan

Intern-buildable plan: stand up the multi-tenant seeded fixture (Tipografia + Antibiotice cross-link to j157 + Securitas cooperative + Adriana Stanciu consulting + Diana personal + Mihai personal + Maria's school + RO-ANAF state) plus mocks for low-light biometric capture, Securitas alarm-cooperative API, lone-worker bracelet pairing, RO-ANAF payroll-reporting, and cross-journey persona-continuity reads from j157 sealed events. Walk every test in order; every test names seed values, exact API calls, expected event chain (dual-seal where cross-tenant), and pass/fail criteria.

## Test environment

| Component | Source |
|---|---|
| Seed tenant — Tipografia | `tests/fixtures/tenants/tipografia-lazar-petrescu-ro.yaml` (shared with j157) |
| Seed tenant — Antibiotice (j157 customer) | `tests/fixtures/tenants/antibiotice-sa-ro.yaml` (shared with j157) |
| Seed tenant — Securitas alarm-cooperative | `tests/fixtures/tenants/cz-securitas-alarm-cooperative-tenant-ro.yaml` |
| Seed tenant — Adriana Stanciu HSE | `tests/fixtures/tenants/adriana-stanciu-consulting-ro.yaml` |
| Seed tenant — Diana personal | `tests/fixtures/tenants/diana-lazar-petrescu-personal.yaml` |
| Seed tenant — Mihai personal | `tests/fixtures/tenants/mihai-lazar-petrescu-personal.yaml` |
| Seed tenant — Maria's school | `tests/fixtures/tenants/scoala-internationala-cluj-ro.yaml` |
| Seed tenant — RO-ANAF | `tests/fixtures/tenants/ro-anaf-tenant.yaml` |
| Seed personas | `tests/fixtures/personas/{diana-lazar,mihai-lazar-petrescu,vladimir-csikos,adriana-stanciu,andrei-tabarca,camelia-lazar,razvan-lazar-petrescu,marius-iancu,carmen-petrescu,maria-lazar}.yaml` |
| Seed j157 sealed events | `tests/fixtures/cross-journey/j157-sealed-events-for-j162-prereq.yaml` (FOGRA-PSO L2 + ISO-12647-2 cert chain + line-stop authority chain) |
| Seed competency catalog | `tests/fixtures/learning-management/competency-catalog-night-shift-solo-2027.yaml` |
| Seed press (j157 press) | `tests/fixtures/plant-maintenance/heidelberg-cx-102-6-lx-01.yaml` |
| Seed Cedar bundle | `tests/fixtures/cedar/j162/cedar-bundle-night-shift-solo-onboarding-v1.cedar` |
| Wire mock — Securitas alarm-cooperative | `tests/mocks/cz-securitas-alarm-cooperative-ro-v2.toml` |
| Wire mock — RO-ANAF payroll reporting | `tests/mocks/ro-anaf-payroll-night-premium-reporting.toml` |
| Wire mock — Lone-worker bracelet | `tests/mocks/lone-worker-bracelet-titanium-v3.toml` |
| Wire mock — Low-light biometric | `tests/mocks/face-id-low-light-capture-pipeline.toml` |
| Frozen clock | `freeze_clock(2027-01-26T21:18:00+02:00)` then advance per test |
| Locale | `ro-RO` primary; `hu-HU` (Vladimir code-switch); `en-GB` |

## Seed data summary

| Datum | Value |
|---|---|
| Diana's principal | `diana.lazăr@tipografia-lazar-petrescu-ro` (shared with j157) |
| Diana's day-shift cert chain (from j157) | FOGRA-PSO-Operator-Level-2 (2024-09-18 → 2027-09-18) + ISO-12647-2-Trained |
| j162 competency target | night-shift-solo-authorization-2027 (1-year validity) |
| Assessment scenarios | 14 |
| Pass threshold | ≥85% per category |
| Supervised shifts logged | 8 (Dec 29 2026 – Jan 23 2027) |
| First solo shift | Mon 2027-02-01 22:00 – Tue 2027-02-02 06:30 EET |
| First WO | WO-TIP-2027-02-01-NIGHT-WO-NSAID-batch-2 (continuation of j157 customer) |
| Dead-man interval | 4 hours, 60s response window |
| Night-shift premium | +25% per RO Codul Muncii §126 |
| Base rate | 47 RON/h → 58.75 RON/h night |

## Test catalog

### T-J162-001 — Cross-journey persona continuity prerequisite check

**Pre-conditions:** Clock `2027-01-26T21:18:00+02:00`. j157 sealed events seeded.

**Action sequence:**

1. Read Diana's cert chain via `learning-management.competency-link.read`
2. Verify FOGRA-PSO L2 + ISO-12647-2-Trained both unexpired
3. Confirm prerequisite chain links to j157's audit events

**Expected events:**

- Prerequisite read returns valid
- Cross-journey link audited as `EVT-J162-CROSS-JOURNEY-PREREQ-VERIFIED-001a`

**Pass criteria:**

- No duplicate cert capture
- j157 events readable as prerequisites
- Cedar context includes prerequisite-verified flag
- UTF-8 NFC preserved (Diana Lazăr)

**Fail criteria:** prereq read fails; identity mismatch; diacritic loss.

### T-J162-002 — Competency assessment scoring

**Pre-conditions:** T-J162-001 passed. Assessment session created.

**Action sequence:**

1. 14 scenarios scored
2. Proctor (Vladimir) sign-off
3. HSE observer (Adriana) sign-off
4. Assessment completed

**Expected events:**

- 14 × `EVT-J162-ASSESSMENT-SCENARIO-001-scenario-N` sealed
- `EVT-J162-COMPETENCY-ASSESSED-001` sealed at 22:43 EET

**Pass criteria:**

- All scenarios ≥85%
- Both qualitative sign-offs present
- p95 scenario record latency ≤ 220 ms
- HSE consultant's tenant audit ref captured

**Fail criteria:** any scenario <85% accepted; missing sign-off; latency >480 ms.

### T-J162-003 — Competency unlock + Cedar permit fires

**Pre-conditions:** T-J162-002 passed.

**Action sequence:**

1. POST `/v1/lms/competencies/unlock`
2. Cedar evaluates `learning_management.competency_unlock_night_shift_solo`
3. Competency profile updates

**Expected events:**

- Cedar permit fires
- `EVT-J162-COMPETENCY-UNLOCKED-002` sealed

**Pass criteria:**

- Cedar context shows `assessment_score_min_85_per_category == true`, `proctor_signoff == true`, `hse_consultant_signoff == true`
- Competency valid_from + valid_through correctly set (2027-01-26 → 2028-01-26)
- Cross-journey link to j157 captured

**Fail criteria:** Cedar permit fails; competency profile drift; cross-journey link missing.

### T-J162-004 — Workplace-integration provisioning (5 sub-steps)

**Pre-conditions:** T-J162-003 passed.

**Action sequence:**

1. Shift schedule entry created
2. Geofence enabled
3. Badge role updated
4. Securitas cross-tenant scope added
5. Payroll night-shift differential enabled

**Expected events:**

- `EVT-J162-WORKPLACE-INTEGRATION-PROVISIONED-003` dual-sealed in `tipografia-lazar-petrescu-ro` AND `cz-securitas-alarm-cooperative-tenant-ro`
- Auto-revoke on competency expiry configured

**Pass criteria:**

- All 5 sub-steps complete
- Carmen (bookkeeper) confirms differential
- p95 provisioning sequence ≤ 4.2 s

**Fail criteria:** any sub-step missing; cross-tenant Securitas call fails.

### T-J162-005 — Lone-worker dead-man enrollment

**Pre-conditions:** T-J162-004 passed.

**Action sequence:**

1. Biometric reconfigured for low-light (4 captures)
2. PIN fallback set
3. 4-hour interval + 60s response window configured
4. Escalation chain set (Mihai personal-tenant + Adriana + Marius)
5. Personal-tenant consent from Mihai captured

**Expected events:**

- `EVT-J162-DEAD-MAN-ENROLLED-004` dual-sealed in `tipografia-lazar-petrescu-ro` AND `mihai.lazar-petrescu.personal`

**Pass criteria:**

- Biometric low-light validation passes
- PIN fallback works
- Mihai's GDPR consent recorded as explicit-consent basis
- Escalation chain ordered correctly

**Fail criteria:** biometric validation fails; missing consent; escalation chain drift.

### T-J162-006 — First night-shift work-order issuance

**Pre-conditions:** All prior passed.

**Action sequence:**

1. WO created Thu Jan 28 14:18 EET
2. Customer tenant (Antibiotice) cross-linked from j157
3. Diana assigned as operator

**Expected events:**

- `EVT-J162-FIRST-WO-ISSUED-005` sealed

**Pass criteria:**

- WO links to j157's Antibiotice customer relationship
- Lower-risk batch chosen (continuation pattern)
- Diana visible as principal owner

**Fail criteria:** customer link broken; risk-class mismatch.

### T-J162-007 — Alarm de-arm via biometric + Cedar context

**Pre-conditions:** Pre-shift; Diana arrives Mon Feb 1 21:54 EET.

**Action sequence:**

1. Biometric face_id scanned
2. Cedar context built
3. Securitas cooperative de-arm called

**Expected events:**

- `EVT-J162-ALARM-DEARMED-006` dual-sealed in `tipografia-lazar-petrescu-ro` AND `cz-securitas-alarm-cooperative-tenant-ro`
- Pressroom light profile activates (60% ambient + 100% inspection)

**Pass criteria:**

- Biometric match score ≥ 0.95
- Cedar context all positive
- De-arm latency ≤ 1.4 s
- Light profile applies correctly

**Fail criteria:** biometric fail; Cedar deny; de-arm latency >2 s.

### T-J162-008 — Shift clock-in geofence + biometric

**Pre-conditions:** T-J162-007 passed.

**Action sequence:**

1. Diana taps clock-in at 22:00:00 EET
2. Geofence verifies
3. Biometric face_id verifies
4. Shift starts

**Expected events:**

- `EVT-J162-SHIFT-CLOCK-IN-006a` sealed

**Pass criteria:**

- Geofence match precise (within 18m radius)
- Biometric match score ≥ 0.95
- Clock-in latency ≤ 1.8 s
- Shift state transitions correctly

**Fail criteria:** geofence false-positive; biometric fail.

### T-J162-009 — Dead-man check-in within 60s window

**Pre-conditions:** T-J162-008 passed. Shift active.

**Action sequence:**

1. Dead-man check-in fires at 02:00 EET
2. Diana taps within 60s window
3. Face_id verifies

**Expected events:**

- `EVT-J162-DEAD-MAN-CHECKIN-006b` sealed at 02:00:06 EET
- Repeated at 06:00 EET as `EVT-J162-DEAD-MAN-CHECKIN-006c`

**Pass criteria:**

- Check-in within 60s window
- Biometric match ≥ 0.95
- No escalation triggered

**Fail criteria:** check-in beyond 60s; biometric fail; escalation fires unnecessarily.

### T-J162-010 — Dead-man miss → escalation chain walk (FORBID-3 path)

**Pre-conditions:** Variant fixture: clock advanced past 60s without check-in.

**Action sequence:**

1. Dead-man fires at 02:00 EET
2. No check-in within 60s
3. Auto-escalation triggers

**Expected events:**

- Cedar `forbid` FORBID-3
- `EVT-J162-CEDAR-DENY-DEAD-MAN-MISS-014c` sealed
- Escalation to Mihai's personal-mobile within 90s
- Securitas alarm-cooperative notified

**Pass criteria:**

- Escalation chain walked in order
- Mihai's mobile receives priority call + messenger push within 90s
- Securitas alarmed
- Audit dual-sealed across tenants

**Fail criteria:** escalation lag >90s; Mihai not reached; Securitas not alerted.

### T-J162-011 — Cedar deny on missing competency (FORBID-1)

**Pre-conditions:** Variant fixture: Diana's `night-shift-solo-authorization-2027` competency revoked.

**Action sequence:** Diana attempts solo night-shift operation.

**Expected events:**

- Cedar `forbid` FORBID-1
- `EVT-J162-CEDAR-DENY-COMPETENCY-MISSING-014a` sealed
- HTTP 403 with explicit competency-missing message in Romanian

**Pass criteria:**

- Operation refused
- Fallback: re-assessment can be scheduled
- Audit dual-sealed

**Fail criteria:** operation allowed; silent fail.

### T-J162-012 — Geofence breach (FORBID-7)

**Pre-conditions:** Diana attempts clock-in from outside the geofenced perimeter.

**Action sequence:** Clock-in attempt from a location 80m from depot.

**Expected events:**

- Cedar `forbid` FORBID-7
- `EVT-J162-CEDAR-DENY-GEOFENCE-014g` sealed

**Pass criteria:**

- Clock-in refused
- Audit dual-sealed
- User shown geofence-help message

**Fail criteria:** clock-in allowed; silent fail.

### T-J162-013 — Personal-tenant consent revocation propagation

**Pre-conditions:** All prior passed. Mihai revokes his dead-man escalation consent.

**Action sequence:**

1. Mihai (in his personal tenant) revokes consent via `tenancy.personal-tenant-cross-tenant-consent-revoke`
2. Diana's next pre-shift readiness check fires

**Expected events:**

- Consent revocation dual-sealed
- Diana's pre-shift readiness check shows missing escalation contact warning
- Alternate fallback chain proposed

**Pass criteria:**

- Revocation propagates within 90s
- Diana cannot start shift until alternate fallback configured

**Fail criteria:** revocation lag; shift starts without fallback.

### T-J162-014 — Press operations during shift (paper-jam + ΔE drift)

**Pre-conditions:** Shift active 22:00-06:30.

**Action sequence:**

1. Paper-jam injected at 23:42 EET
2. Diana clears within 14 min
3. ΔE drift injected at 04:18 EET
4. Diana corrects proactively

**Expected events:**

- Paper-jam audit at 23:42
- ΔE correction audit at 04:18 (ΔE 2.7 → 1.2)

**Pass criteria:**

- Paper-jam resolved within 14 min
- ΔE returns to <1.5 within 8 min
- Operator-solo flag captured for each event

**Fail criteria:** events not captured; resolution time mismatch.

### T-J162-015 — Shift handoff to Camelia

**Pre-conditions:** All prior passed.

**Action sequence:**

1. Camelia arrives 06:18 EET
2. Handoff initiated 06:30 EET
3. Handoff completes 06:42 EET

**Expected events:**

- `EVT-J162-SHIFT-HANDOFF-008` sealed
- Both Diana + Camelia passkey + face_id captured

**Pass criteria:**

- Handoff notes include cross-link to j157 + current shift events
- Camelia receives full context (active jobs, ink levels, substrate, etc.)
- Diana cannot edit shift records post-handoff

**Fail criteria:** handoff missing data; Diana retains edit rights.

### T-J162-016 — Night-shift premium payroll application

**Pre-conditions:** Shift completed. Next payroll cycle Fri Feb 5.

**Action sequence:**

1. Carmen runs payroll for the week
2. Diana's hours calculated: 32h day + 8.5h night
3. Night-shift premium +25% applied

**Expected events:**

- `EVT-J162-NIGHT-PREMIUM-PAID-009` sealed in `tipografia-lazar-petrescu-ro` AND `ro-anaf-tenant`
- Payroll line items broken out

**Pass criteria:**

- Day rate (47 × 32) + night rate (58.75 × 8.5) + premium calc correct
- Total gross: 2,003.38 RON exact
- ANAF reporting filed

**Fail criteria:** premium not applied; ANAF report missing; calc drift.

### T-J162-017 — Diacritic + Romanian + Hungarian fidelity

**Pre-conditions:** All personas + tenants loaded.

**Action sequence:**

1. Read all names from identity + tenancy
2. Write to all fields (workflow + payroll + audit + handoff)
3. Query persisted forms

**Pass criteria:**

- "Diana Lazăr", "Mihai Lazăr-Petrescu", "Camelia Lazăr", "Andrei Tăbârcă", "Răzvan Lazăr-Petrescu", "Vladimir Csikós" all preserve UTF-8 NFC
- Hungarian phrases ("Köszönöm", "Értem") preserve NFC
- No Romanization in legal fields
- ANAF report preserves Romanian diacritics

**Fail criteria:** any normalization; transliteration in legal field.

### T-J162-018 — State machine valid transitions

**Pre-conditions:** All prior reached.

**Action sequence:** Walk all 6 competency-unlock states + 6 shift states.

**Pass criteria:**

- All transitions land in order with required preconditions
- Skip transitions refused

**Fail criteria:** any skip allowed; state drift.

### T-J162-019 — Cross-tenant audit dual-seal fuzz

**Pre-conditions:** All prior passed.

**Action sequence:** 950 generated cross-tenant operations.

**Expected behavior:** every permitted op dual-seals; every denied op dual-seals deny.

**Pass criteria:**

- 0 single-seal events
- 0 silent passes
- p99 dual-seal ≤ 480 ms
- Merkle chain validates

**Fail criteria:** any single-seal; silent pass; merkle break.

### T-J162-020 — Lone-worker bracelet pairing

**Pre-conditions:** Diana has bracelet (Vladimir's wife's, Saturday gift).

**Action sequence:**

1. Bracelet pairs to Diana's personal iPhone via Bluetooth
2. Pairing exposed to work tenant via Cedar-narrowed scope
3. Fall-detect events bridge to work tenant during shift

**Pass criteria:**

- Pairing succeeds
- Cedar scopes work tenant access to fall-detect events only (not health-data)
- Fall-detect heartbeat visible during shift

**Fail criteria:** pairing fails; broad data leak; missing heartbeat.

## Performance gates

| Operation | p50 | p95 | p99 |
|---|---|---|---|
| Competency assessment session create | 280 ms | 680 ms | 1.4 s |
| Per-scenario score record | 80 ms | 220 ms | 380 ms |
| Competency unlock + Cedar eval | 95 ms | 280 ms | 480 ms |
| Workplace-integration provisioning sequence | 1.8 s | 4.2 s | 7.8 s |
| Securitas cross-tenant scope update | 480 ms | 1.1 s | 2.2 s |
| Biometric low-light enrollment | 6 s | 12 s | 18 s |
| Dead-man enroll | 480 ms | 1.1 s | 2.2 s |
| Alarm de-arm with biometric | 280 ms | 680 ms | 1.4 s |
| Shift clock-in geofence + biometric | 380 ms | 920 ms | 1.8 s |
| Dead-man check-in | 140 ms | 320 ms | 540 ms |
| Dead-man escalation walk | 1.8 s | 4.2 s | 7.8 s |
| Shift handoff | 240 ms | 580 ms | 1.0 s |
| Payroll night-shift differential calc | 320 ms | 780 ms | 1.6 s |
| Cross-tenant audit dual-seal | 120 ms | 280 ms | 480 ms |

## Cross-tenant invariant tests

| Invariant | Probe | Pass condition |
|---|---|---|
| Cert-missing operation attempt | `night-shift action without competency` | 403 + dual-seal |
| Cross-journey identity mismatch | `Diana's j157 cert ≠ Diana's j162 identity` | 403 + identity-mismatch audit |
| Personal-tenant consent revocation | Mihai revokes mid-shift | shift continues; fallback chain notified |
| Geofence breach mid-shift | Diana physically leaves perimeter | soft alert; persistent → escalation |
| Securitas reads Tipografia payroll | `securitas → payroll.read` | 403 + dual-seal |
| Romanization in legal field | ANAF name = "Diana Lazar" | 422 + diff |
| ΔE alert during shift | press telemetry > FOGRA tolerance | operator corrective action; audit dual-sealed |
| Cross-cell drift Diana's data | `write to us-east cell` | 403 |

## Chaos scenarios

1. **Securitas alarm-cooperative API down at depot entry** — Local cached competency state used; manual de-arm fallback via keycode; Cedar-gated emergency path
2. **Lone-worker bracelet Bluetooth disconnect** — Tablet biometric still functions; bracelet re-pair attempted; missing-bracelet alerted but shift continues
3. **Mihai's personal-tenant unreachable for escalation** — Fallback to Adriana within 30s; tertiary fallback Marius
4. **RO-ANAF reporting endpoint degraded** — Payroll calc completes; ANAF report queues with retry; legal-effect timer paused
5. **Cedar service degraded** — Halt + safety-critical actions remain available (fail-safe); workflow advancement paused
6. **Power outage during night shift** — Press auto-halts; emergency lights activate; lone-worker bracelet's accelerometer-fall-detect remains active on battery
7. **Heidelberg telemetry stream interrupted** — Operator visual fallback per j157 chaos doctrine; audit flagged

## Sign-off checklist

- [ ] All 20 tests pass
- [ ] All 8 cross-tenant invariant probes return expected dual-seal
- [ ] Performance gates met
- [ ] Chaos scenarios complete without data loss
- [ ] All 4 µservices in `/microservices/` resolve: learning-management, workplace-integration, identity, tasks
- [ ] All 9 ADRs cited resolve
- [ ] Diacritic + Romanian + Hungarian + cross-journey-continuity invariant: 0 normalization events
- [ ] Cross-journey link to j157 sealed events functions
- [ ] Lone-worker dead-man 60s window holds across 100 fuzz probes
- [ ] Securitas cooperative auto-revoke on competency expiry confirmed
- [ ] DPO sign-off Tipografia + Securitas + Mihai personal + Maria school
- [ ] Night-shift premium pay correctly applied + ANAF reported

## Stop condition

Plan complete when all 20 tests pass, the cross-journey persona continuity from j157 to j162 holds (FOGRA-PSO L2 prerequisite read without duplicate capture), the competency-gated authority Cedar predicates function correctly, the lone-worker dead-man protocol's 60-second window holds with personal-tenant escalation contact consented and active, the cross-tenant Securitas alarm-cooperative scope auto-revokes on competency expiry, and Diana's first solo night-shift completes at 06:42 EET Tue Feb 2 2027 with the night-shift premium correctly applied and reported to RO-ANAF.
