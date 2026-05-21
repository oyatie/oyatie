---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j155-stefan-kovacs-college-night-shift-and-finals-week
date: 2026-05-20
authority_tier: 2
status: draft
---

# j155 — Integration test plan

This plan is intern-buildable: a new engineer stands up the seeded three-tenant fixture (`personal-stefan-kovacs-hu` + `oszk-security-services_hu` + `bme-student-bodv75_hu`) plus the `bme-research-cohort-2026-sleep-grade-fall` cohort tenant, then walks every test in order. Every test names seed values, exact API calls, expected event chain across the named tenants, the Cedar permit (or deny), and pass/fail criteria.

The dual-role-identity (ADR-0311) + role-projection (ADR-0317) doctrines turn every cross-tenant probe into a deny case that MUST seal an audit event in BOTH source and target tenants — never one without the other. The integration plan therefore over-weights deny tests.

## Test environment

| Component | Source |
|---|---|
| Seed tenant — personal | `tests/fixtures/tenants/personal-stefan-kovacs-hu.yaml` |
| Seed tenant — work | `tests/fixtures/tenants/oszk-security-services_hu.yaml` |
| Seed tenant — student | `tests/fixtures/tenants/bme-student-bodv75_hu.yaml` |
| Seed tenant — cohort | `tests/fixtures/tenants/bme-research-cohort-2026-sleep-grade-fall.yaml` |
| Seed personas | `tests/fixtures/personas/{stefan-kovacs,reka-hahn,dr-erika-balogh-bme-os-prof,oszk-night-shift-manager-anna-toth,bme-dpo-laszlo-virag,oszk-dpo-marta-szabo}.yaml` |
| Seed shifts | `tests/fixtures/calendar/oszk-shifts-2026-12-14-to-2026-12-19.yaml` |
| Seed exam timetable | `tests/fixtures/calendar/bme-vik-finals-2026w50.yaml` |
| Seed LMS course pack | `tests/fixtures/lms/bme-vik-aut-viiiab1015-operating-systems-2026fall.tar.gz` |
| Seed community channel | `tests/fixtures/community/os-finals-2026-mls-bootstrap.json` |
| Seed Cedar bundle | `tests/fixtures/cedar/j155/cedar-bundle-stefan-dual-role-v1.cedar` |
| Seed payroll arrangement | `tests/fixtures/workplace-integration/oszk-adp-streamline-hu-bme-deduction-stefan.yaml` |
| Wire mock — ADP Streamline HU | `tests/mocks/adp-streamline-hu-v2025.toml` |
| Wire mock — Hungarian SEPA / GIRO | `tests/mocks/giro-sepa-instant-hu.toml` |
| Wire mock — BME Neptun (Hungarian SIS) | `tests/mocks/neptun-bme-sis-v6.toml` |
| Wire mock — NFC kiosk | `tests/mocks/dell-wyse-5070-nfc.toml` |
| Wire mock — pixel 8a TEE attestation | `tests/mocks/pixel-8a-strongbox.toml` |
| Wire mock — MLS delivery service | `tests/mocks/messenger-mls-ds-v1.toml` |
| Frozen clock | `freeze_clock(2026-12-14T21:48:14+01:00)` then advance per test |
| Frozen weather | `OSZK-foyer outside temp = 1°C, light rain, wind 12 km/h ENE` (for the indoor humidity sensor's correlated reading) |

## Seed data summary

| Datum | Value |
|---|---|
| Stefan passkey root | `passkey-pixel-8a-stefan-personal-93a7c` |
| OSZK employee id | `emp-stefan-kovacs-oszk-2025-09-14-night-guard` |
| BME student id (Neptun) | `BODV75` (real Neptun-style 6-char code) |
| BME course code OS | `VIK-AUT-VIIIAB1015` (Operating Systems) |
| BME course code Discrete Math II | `VIK-AUT-VIIMA9302` |
| HUF tuition installment | `187500` (Q4 2026 payable Dec 20) |
| Réka Hahn employee id | `emp-reka-hahn-oszk-2024-03-02-night-guard` |
| OSZK kiosk id | `dell-wyse-5070-oszk-staff-entrance-001` |
| HLC tier | default (HLC); payroll-bridge call ONLY uses TrueTime-class clock |
| Cohort id | `cohort-bme-vik-sleep-grade-2026-fall-n-127` |

## Test catalog

### T-J155-001 — Shift confirmation (happy path)

**Pre-conditions:** clock at `2026-12-14T21:48:14+01:00`. Stefan's last clock-out was `2026-12-12T06:00:00+01:00` (>11 hr rest ✓). Weekly running average 22.0 hr.

**Action sequence:**

1. Stefan taps Pixel 8a (NFC) on `dell-wyse-5070-oszk-staff-entrance-001`
2. POST `/v1/tenants/oszk-security-services_hu/shifts/shift-stefan-2026-12-14-22-night/confirm` with handshake §1.1 body

**Expected events (order):**

- `EVT-J155-CALENDAR-SHIFT-CONFIRM-001` sealed in `oszk-security-services_hu`
- `EVT-J155-AUDIT-SEAL-OBSERVABILITY-EMIT-001a` emitted to `observability` partitioned to OSZK only

**Pass criteria:**

- HTTP 200, `shift_confirmation_id = shift-conf-2026-12-14-stefan-001`
- `weekly_hours_running_avg = 22.0`, `wtd_weekly_cap_remaining = 26.0`
- Audit visible ONLY in `oszk-security-services_hu`; query against `personal-stefan-kovacs-hu` AND `bme-student-bodv75_hu` returns empty
- p95 latency tap-to-200 ≤ 480 ms
- Cedar evaluated decision: `permit` with reason `night_shift_guard + acting_tenant_match`

**Fail criteria:** event sealed in any non-OSZK tenant; weekly avg not 22.0; latency >900 ms; HTTP non-200.

### T-J155-002 — Swap offer arrives via messenger (OSZK-only)

**Pre-conditions:** Réka Hahn principal active in OSZK tenant; Stefan logged in at the kiosk.

**Action sequence:**

1. Réka POSTs `/v1/tenants/oszk-security-services_hu/messenger/dm` with body `{"to":"emp-stefan-kovacs-oszk-2025-09-14-night-guard","subject":"swap_request","shift_id":"shift-reka-2026-12-15-22-night","reason":"flu_temp_38.4"}`
2. MLS group `dm-stefan-reka-oszk` materializes
3. Stefan receives push at 21:53:11

**Expected events:**

- `EVT-J155-MESSENGER-SWAP-OFFER-RECEIVED-002a` sealed in `oszk-security-services_hu`
- `EVT-J155-MESSENGER-SWAP-OFFER-DELIVERED-002b` sealed in `oszk-security-services_hu`

**Pass criteria:**

- MLS group epoch advances 0 → 1
- Push notification metadata contains NO message content (E2EE preserved end-of-line)
- Audit shows source = Réka, target = Stefan, both within OSZK tenant
- No audit in `personal-stefan-kovacs-hu` or `bme-student-bodv75_hu`

**Fail criteria:** push payload leaks message body; MLS epoch fails to advance; audit appears in personal or student tenant.

### T-J155-003 — Polite decline does not leak exam context

**Pre-conditions:** T-J155-002 passed.

**Action sequence:**

1. Stefan composes decline: `"Bocs, kedden nem tudom. Pénteken igen ha kell."` ("Sorry, can't on Tuesday. Friday yes if needed.")
2. POST `/v1/tenants/oszk-security-services_hu/messenger/reply` with body containing only that text
3. Server-side DLP scan runs

**Expected events:**

- `EVT-J155-MESSENGER-SWAP-DECLINED-003` sealed in `oszk-security-services_hu`
- `EVT-J155-DLP-EXAM-LEAK-NEGATIVE-003a` sealed in `oszk-security-services_hu` (DLP saw no BME exam vocabulary)

**Pass criteria:**

- DLP keyword set `{vizsga, OS-finals, VIK-AUT-VIIIAB1015, Neptun, BME, BODV75}` returns 0 matches
- Message delivered in <300 ms
- `messenger.reply` Cedar permit `permit` with reason `night_shift_guard + active_tenant_match`

**Fail criteria:** any keyword leak; message body uploaded to personal or student tenant.

### T-J155-004 — Cedar denies cross-tenant probe (OSZK admin tries to read BME LMS)

**Pre-conditions:** OSZK night-shift manager Anna Tóth attempts to query Stefan's BME study activity through a misconfigured admin tool.

**Action sequence:**

1. Anna's session token has acting-tenant `oszk-security-services_hu`
2. She POSTs `/v1/tenants/bme-student-bodv75_hu/lms/users/stefan.kovacs/activity-summary`

**Expected events:**

- Cedar evaluates `deny` (forbid rule §FORBID-1 in `cedar-policy.cedar`)
- `EVT-J155-CEDAR-DENY-CROSS-TENANT-LMS-PROBE-005` sealed in BOTH `oszk-security-services_hu` (where the principal was acting) AND `bme-student-bodv75_hu` (where the resource lives)
- Latency-class audit also emitted to `observability` with reason code `OSZK_TO_BME_DENY`

**Pass criteria:**

- HTTP 403 with body `{"error":"cedar_forbid","decision_id":"<uuid>","forbid_rule":"forbid-osz-to-bme"}`
- Audit appears in BOTH tenants (this is the dual-seal invariant per ADR-0263)
- BME's DPO László Virág receives a daily-digest entry by 06:00 the next day

**Fail criteria:** any non-403 response; audit appears in only ONE tenant (single-seal is a bug); DPO digest missing.

### T-J155-005 — Tenant switch from work to student

**Pre-conditions:** Stefan completes T-J155-001, is now in OSZK context.

**Action sequence:**

1. Stefan opens the active-tenant pill on his Pixel 8a
2. Selects "BME — student-bodv75"
3. Holds confirm 2.00 seconds (the explicit-confirmation interval)
4. Pixel 8a issues `POST /v1/identity/tenant-switch` with body `{"from":"oszk-security-services_hu","to":"bme-student-bodv75_hu","passkey_assertion":"<webauthn b64>"}`

**Expected events:**

- WebAuthn assertion validates (`user_present=true, user_verified=true`)
- `EVT-J155-IDENTITY-TENANT-SWITCH-003` sealed in `personal-stefan-kovacs-hu` (the identity bearer) AND a redacted copy in `bme-student-bodv75_hu`
- OSZK does NOT receive an audit (privacy-preserving: the work tenant should not know where Stefan went next)

**Pass criteria:**

- Active-tenant pill on every screen surface flips to "BME — student" within 380 ms
- Background OSZK alarm subscription remains active in the system tray with a "duty-active" marker
- Cedar `lms.read_notes` now permits; `calendar.confirm_shift` now denies
- p95 switch time ≤ 600 ms

**Fail criteria:** OSZK receives a destination-disclosing audit; passkey skip; switch fails to flip Cedar decision; latency >1200 ms.

### T-J155-006 — Read OS lecture notes from BME LMS

**Pre-conditions:** T-J155-005 passed.

**Action sequence:**

1. Stefan navigates to course `VIK-AUT-VIIIAB1015`
2. Opens "Lecture 12 — Scheduling: Multilevel Feedback Queues"
3. GET `/v1/tenants/bme-student-bodv75_hu/lms/courses/VIK-AUT-VIIIAB1015/modules/lecture-12/notes`

**Expected events:**

- `EVT-J155-LMS-NOTES-READ-004` sealed in `bme-student-bodv75_hu`
- NO event in `oszk-security-services_hu` (this is the boundary; verify via direct DB query)

**Pass criteria:**

- HTTP 200, body contains 47-page PDF + 12-min lecture video manifest
- Read receipt visible in BME LMS analytics under "active_student"
- OSZK queries against the same path return 403 + audit per T-J155-004
- p95 GET latency ≤ 220 ms

**Fail criteria:** any event in OSZK tenant; lecture content cached on the OSZK kiosk; HTTP non-200.

### T-J155-007 — Tuition payment via payroll-bridge (TrueTime-class)

**Pre-conditions:** Stefan's December paycheck of HUF 312,000 net is queued in ADP-Streamline-HU. Bridge configuration: HUF 187,500 to BME, residual HUF 124,500 to Stefan's personal account.

**Action sequence:**

1. Stefan confirms the tuition installment in the BME LMS finance pane at `2026-12-14T23:14:00+01:00`
2. POST `/v1/tenants/bme-student-bodv75_hu/payments/tuition/installment-q4-2026-confirm` with `{"amount":"187500","currency":"HUF","funding_source":"payroll_bridge","installment_id":"inst-bme-bodv75-2026q4"}`
3. workplace-integration µservice initiates the three-way handshake (OSZK ↔ personal ↔ BME)
4. ADP-Streamline-HU computes net split via mock at clock `2026-12-15T01:00:00+01:00`
5. SEPA Instant CT routed via GIRO at clock `2026-12-15T01:00:14+01:00`

**Expected events (strict order — TrueTime fences enforce):**

- `EVT-J155-PAYMENTS-TUITION-INTENT-006a` in `bme-student-bodv75_hu` (HLC ts)
- `EVT-J155-WORKPLACE-BRIDGE-PROPOSE-006b` in `oszk-security-services_hu` (HLC ts)
- `EVT-J155-PERSONAL-BRIDGE-CONSENT-006c` in `personal-stefan-kovacs-hu` (HLC ts)
- `EVT-J155-WORKPLACE-BRIDGE-COMMIT-006d` (TrueTime-class ts — within ±7 ms TT-interval) sealed in ALL THREE tenants atomically
- `EVT-J155-PAYMENTS-TUITION-PAID-006e` in `bme-student-bodv75_hu`

**Pass criteria:**

- The three-tenant commit's TrueTime intervals do NOT overlap with any other transaction's interval (the "uncertainty fence" passes)
- BME's tuition ledger shows HUF 187,500 paid against `inst-bme-bodv75-2026q4`
- Personal tenant shows HUF 124,500 credited
- OSZK shows the gross HUF 312,000 paid out with a deduction line item
- p99 end-to-end ≤ 4.2 seconds (the SEPA Instant SLA)
- Stefan can verify the payment trail in his personal tenant's "money flow" view; each row carries a cross-tenant `trace_id`

**Fail criteria:** TrueTime interval overlap (would indicate a clock-fencing bug); partial commit (event sealed in 2 of 3 tenants); BME ledger and personal tenant disagree on amount; SEPA call timeout >5s.

### T-J155-008 — Personal-tenant consent gate denies if Stefan hasn't pre-authorized

**Pre-conditions:** Stefan's personal-tenant consent for the OSZK→BME payroll bridge is REVOKED (test variant).

**Action sequence:**

1. Same as T-J155-007 steps 1–3
2. Personal-tenant Cedar evaluates `workplace_bridge.consent_required`

**Expected events:**

- `EVT-J155-PERSONAL-CEDAR-DENY-BRIDGE-008` sealed in `personal-stefan-kovacs-hu`
- Workplace-integration rolls back; OSZK paycheck proceeds without bridge deduction
- BME LMS shows installment unpaid; Stefan receives a non-coercive nudge: "Re-authorize the OSZK→BME bridge to use payroll deduction, or pay directly in app"

**Pass criteria:**

- HTTP 402 from BME `tuition/installment-confirm` with body `{"error":"personal_consent_revoked","resolve":"reauthorize_or_pay_direct"}`
- OSZK paycheck of HUF 312,000 lands intact in Stefan's personal account
- BME installment status: `unpaid`, due-date countdown begins
- No partial transfer reaches BME; no negative balance in any tenant

**Fail criteria:** any HUF moves to BME; OSZK's paycheck split incorrectly; Stefan is locked out of either tenant.

### T-J155-009 — Sleep-grade telemetry: per-event anonymization gate

**Pre-conditions:** Stefan opted in to the BME sleep-grade research cohort in semester 1. The pipeline is configured to allow ONLY pseudonymous emit to `cohort-bme-vik-sleep-grade-2026-fall-n-127`.

**Action sequence:**

1. Stefan's Pixel 8a uploads a sleep window `{"start":"2026-12-15T07:14:00+01:00","end":"2026-12-15T11:42:00+01:00","wake_count":2,"deep_sleep_minutes":58,"rem_minutes":71}`
2. The observability pipeline applies the `sleep-grade-correlation-pipeline.yaml` transform
3. The transform strips identifier columns and replaces `stefan.kovacs@personal-id.oya` with the cohort pseudonym `cohort-pseudo-bodv75-fall26`
4. Egress to BME cohort tenant ONLY

**Expected events:**

- `EVT-J155-OBSERVABILITY-SLEEP-GRADE-EMIT-008` sealed in `personal-stefan-kovacs-hu` (the source of consent) AND `bme-research-cohort-2026-sleep-grade-fall` (the egress destination)
- NO event in `oszk-security-services_hu` or `bme-student-bodv75_hu`

**Pass criteria:**

- Cohort row carries pseudonym only; no `stefan.kovacs`, no `BODV75`, no `passkey-pixel-8a-stefan-personal-93a7c`
- Egress sink receipt id `cohort-egress-rcpt-stefan-2026-12-15-08` recorded in personal-tenant ledger
- OSZK probes against the cohort tenant return 403 + dual-seal deny audit
- Stefan can review what was sent in his personal-tenant privacy dashboard within 24 hours (Art 15 GDPR ready)

**Fail criteria:** identifier leak to cohort tenant; egress to non-cohort tenant; OSZK can see the cohort tenant; pipeline drops the sleep window silently.

### T-J155-010 — `#os-finals-2026` MLS post + OSZK opacity

**Pre-conditions:** T-J155-005 passed; Stefan is in BME context; community channel `#os-finals-2026` has 47 members; MLS epoch 412.

**Action sequence:**

1. Stefan posts: `"VIIIAB1015 zh10: page-fault hierarchy magyarázat valakinek? Aki tudja, részletesen kérem"` ("Lecture 10 page-fault hierarchy explanation anyone? Whoever knows, in detail please")
2. POST `/v1/tenants/bme-student-bodv75_hu/community/channels/os-finals-2026/post`

**Expected events:**

- MLS epoch advances 412 → 413
- `EVT-J155-COMMUNITY-POST-STUDENT-007` sealed in `bme-student-bodv75_hu`
- Indexing pipeline applies the BME-only retention class (180-day soft delete)

**Pass criteria:**

- Server stores ciphertext only; plaintext indexing happens in client-side scope
- Replies from peers arrive in ≤90 seconds (p50 for active channels)
- OSZK admin probe against any channel resource returns 403 + dual-seal
- No leakage to personal-tenant analytics

**Fail criteria:** any plaintext storage; OSZK probe succeeds; epoch fails to advance; reply latency >5 min.

### T-J155-011 — Finals-week mode auto-pauses cross-tenant notifications

**Pre-conditions:** clock `2026-12-15T07:42:00+01:00`. Stefan's BME exam timetable shows OS final Dec 16 08:00. The personal-tenant finals-week mode toggles ON automatically at Dec 14 18:00.

**Action sequence:**

1. A non-emergency community @-mention arrives in BME context
2. A non-emergency messenger DM arrives in personal context
3. An OSZK shift reminder arrives for Wednesday 22:00

**Expected behavior:**

- BME and personal notifications are queued silently; no audible alert; the notification tray shows "🌙 Finals mode — 3 queued"
- OSZK shift reminder DOES break through (emergency/duty class)

**Pass criteria:**

- 0 audible alerts from BME or personal channels during Mon 12:00 → Fri 13:00 window
- 100% of OSZK duty-class alerts deliver within 30 s
- Stefan can review the queue at any time
- Audit `EVT-J155-NOTIFICATION-FINALS-MODE-QUEUE-011` per queued item

**Fail criteria:** any BME/personal audible during the window; any OSZK duty-class missed; queue corruption.

### T-J155-012 — Working-Time-Directive evaluator blocks Friday extension

**Pre-conditions:** Stefan accepted Réka's swap (T-J155-001 + T-J155-002 + Thursday day shift); the OSZK manager proposes adding a Friday 22:00 night shift. Stefan's running total would hit 50 hr/week.

**Action sequence:**

1. Manager Anna Tóth POSTs `/v1/tenants/oszk-security-services_hu/shifts/propose` with `{"target_employee":"emp-stefan-kovacs-oszk-2025-09-14-night-guard","date":"2026-12-19","start":"22:00","duration_hr":8}`
2. WTD evaluator runs pre-flight

**Expected events:**

- WTD evaluator yields `weekly_total_projected = 50.0`, `cap = 48.0`, decision `BLOCK`
- `EVT-J155-CALENDAR-WTD-BLOCK-012` sealed in `oszk-security-services_hu`
- Anna Tóth's screen shows a non-bypassable block dialog

**Pass criteria:**

- HTTP 409 with body `{"error":"wtd_weekly_cap_breach","cap":48.0,"projected":50.0,"adr":"ADR-0244"}`
- No shift assignment created
- Stefan is unaware of the attempt (manager-side workflow)

**Fail criteria:** shift assigned despite breach; Stefan receives a coercive nudge; cap calculation off by >0.5 hr.

### T-J155-013 — Low-bandwidth offline-first sync at OSZK basement

**Pre-conditions:** Stefan's Pixel 8a temporarily loses uplink at 02:14 in the OSZK basement (rare-books storage check). Clock-out is scheduled 06:00.

**Action sequence:**

1. Stefan completes 3 basement-check tasks at 02:18, 02:31, 02:47 (each task generates 1 event in the OSZK tenant)
2. Connectivity returns at 03:02
3. Pixel 8a flushes the offline buffer

**Expected events:**

- All 3 tasks materialize with their `local_event_time` preserved
- HLC reconciliation orders them within the OSZK tenant stream
- `EVT-J155-OFFLINE-SYNC-FLUSH-013` sealed in `oszk-security-services_hu`

**Pass criteria:**

- All 3 task records carry the original 02:18 / 02:31 / 02:47 timestamps (NOT 03:02)
- HLC sequencing maintains causal order
- Clock-out at 06:00 reflects full 8-hour shift; no time-fraud signal raised
- No data loss; offline buffer drains in ≤2.5 s

**Fail criteria:** any task lost; timestamps coerced to flush time; clock-out shows <8 hr.

### T-J155-014 — DPO Art-15 export across all three tenants

**Pre-conditions:** Stefan files a GDPR Art 15 access request from his personal tenant.

**Action sequence:**

1. POST `/v1/tenants/personal-stefan-kovacs-hu/dpo/art15-export-request`
2. Fan-out to personal + work + student tenants in parallel
3. Each tenant produces a per-tenant export, sealed locally
4. Aggregator merges into a single ZIP with per-tenant subfolders + a manifest

**Expected events:**

- 3 per-tenant exports: `personal-export-2026-12-20-stefan.zip`, `oszk-export-2026-12-20-stefan.zip`, `bme-export-2026-12-20-stefan.zip`
- `EVT-J155-DPO-ART15-EXPORT-014` sealed in EACH tenant
- Aggregator's manifest cites the merkle-root of every audit event referenced

**Pass criteria:**

- Export delivered in ≤30 days (GDPR statutory)
- Stefan-as-employee data does NOT appear in the BME export
- Stefan-as-student data does NOT appear in the OSZK export
- Sleep-grade pseudonymous cohort row is mapped back to Stefan in the personal-tenant export only (he can see his own row even though no one else can)
- ZIP signed by `personal-stefan-kovacs-hu` with timestamp from a qualified eIDAS QTSA

**Fail criteria:** any cross-tenant leak; export delayed >30 days; signature invalid; pseudonym row appears in BME or OSZK export.

### T-J155-015 — TenantBoundary fuzz: 500 cross-tenant probe variants

**Pre-conditions:** fuzz harness configured with the dual-role policy. Test runs nightly in CI.

**Action sequence:** 500 generated requests crossing `{principal_acting_tenant} × {target_resource_tenant} × {action}`, drawing from the closed-set vocab of 23 actions × 4 tenants.

**Expected behavior:** Every request that crosses tenant lines (the matrix of 12 cross combinations × 23 actions = 276 deny cases) MUST produce a `cedar_forbid` decision AND a dual-seal audit.

**Pass criteria:**

- 276/276 deny cases produce HTTP 403 + dual-seal
- 0 silent passes (a silent pass is a P0 bug)
- p99 Cedar evaluation ≤ 180 ms
- Fuzz harness reports the dual-seal invariant green

**Fail criteria:** any single-seal; any silent pass; latency >300 ms; fuzz aborts.

## Test execution order

```
T-J155-001 → T-J155-002 → T-J155-003 → T-J155-005 → T-J155-006 → T-J155-004 (must run AFTER 006 to test the deny path against a non-empty resource) → T-J155-007 (variant pass) → T-J155-008 (variant deny) → T-J155-009 → T-J155-010 → T-J155-011 → T-J155-012 → T-J155-013 → T-J155-014 → T-J155-015 (CI nightly)
```

## Cross-tenant invariant tests (run after every other test)

| Invariant | Probe | Pass condition |
|---|---|---|
| OSZK admin cannot read BME LMS | `oszk-admin → lms.read_notes(BME)` | 403 + dual-seal |
| BME prof cannot read OSZK shift | `bme-prof → calendar.read_shift(OSZK)` | 403 + dual-seal |
| OSZK admin cannot read cohort | `oszk-admin → analytics.read(cohort)` | 403 + dual-seal |
| BME admin cannot read personal payments | `bme-admin → payments.read(personal)` | 403 + dual-seal |
| Personal-tenant export omits exam data | Art-15 export → grep "VIK-AUT-VIIIAB1015" | 0 hits (in OSZK export); ≥1 hit (in BME export) |
| Sleep cohort row pseudonymous | grep `stefan` in cohort emission | 0 hits |
| Finals-mode silences non-emergency | inject ping at 14:30 Tue | queued, no audible |

## Performance gates

| Operation | p50 | p95 | p99 |
|---|---|---|---|
| Shift confirm (NFC) | 220 ms | 480 ms | 720 ms |
| Tenant switch | 250 ms | 600 ms | 900 ms |
| LMS notes read | 90 ms | 220 ms | 400 ms |
| Community post | 110 ms | 280 ms | 520 ms |
| Cedar evaluation (any) | 60 ms | 180 ms | 280 ms |
| Payroll-bridge commit (TT) | 1.8 s | 3.4 s | 4.2 s |
| Sleep-grade pipeline emit | 140 ms | 320 ms | 580 ms |

## Failure-injection scenarios (chaos)

1. **OSZK kiosk loses network mid-NFC-tap** → fall back to local-stored shift, sync on reconnect, no double-confirm
2. **MLS DS unreachable for 8 min during Réka's swap offer** → queue locally, deliver on recovery, epoch correctness preserved
3. **TrueTime interval widens to 18 ms during payroll commit** → bridge waits for the fence to close, p99 stretches to 6.0 s, no partial commit
4. **Cedar service degraded** → fail-closed (deny all); audit `EVT-J155-CEDAR-DEGRADED-CIRCUIT-OPEN-CHAOS-3`
5. **One of the three tenants is region-isolated by simulated DR drill** → bridge waits; user sees "bridge paused, retry in 47s"; no data corruption

## Sign-off checklist

- [ ] All 15 tests pass in seeded environment
- [ ] All invariant probes return dual-seal
- [ ] Performance gates met on cell `eu-frankfurt-primary` and `eu-amsterdam-secondary`
- [ ] Chaos scenarios complete without data loss
- [ ] Audit events match the registry in ADR-0263
- [ ] Persona dossier MASTER-ROSTER §3.4 row 99 referenced
- [ ] All 5 µservices in `/microservices/` resolve: calendar, learning-management, payments, community, observability
- [ ] All 5 ADRs cited resolve: ADR-0311, ADR-0317, ADR-0244, ADR-0263, ADR-0252
- [ ] DPO sign-off from `oszk-dpo-marta-szabo` AND `bme-dpo-laszlo-virag`

## Stop condition

This plan is complete when all 15 tests pass in the seeded three-tenant + cohort environment, all dual-seal invariants hold under the 500-variant fuzz, and the GDPR Art-15 export delivers per-tenant separation with the pseudonym-mapping row visible ONLY in Stefan's personal-tenant export.
