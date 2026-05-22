---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j158-print-shop-cell-rebalance-shorts-creator-spike
date: 2026-05-20
authority_tier: 2
status: draft
---

# j158 — Integration test plan

Intern-buildable plan: stand up the seeded two-tenant fixture (`personal-haewon-kim-kr` + `sungkyul-sangsa-print-co-kr`) plus the shorts-platform creator cell, the cell-rebalance ops console, the KR-LSA evaluator, and the mocks for cell warm-start + traffic-shift. Walk every test in order. Every test names seed values, exact API calls, expected event chain across both tenants (dual-seal mandatory where cross-tenant), and pass/fail criteria.

## Test environment

| Component | Source |
|---|---|
| Seed tenant — personal | `tests/fixtures/tenants/personal-haewon-kim-kr.yaml` |
| Seed tenant — employer | `tests/fixtures/tenants/sungkyul-sangsa-print-co-kr.yaml` |
| Seed personas | `tests/fixtures/personas/{haewon-kim,lee-minjun,park-jaewon,kim-junho,lee-haein}.yaml` |
| Seed disclosure record | `tests/fixtures/identity/disclosure-haewon-kim-sungkyul-sangsa-2024-08-12.yaml` |
| Seed cells | `tests/fixtures/cell/seoul-employer-cells-set.yaml` |
| Seed creator content | `tests/fixtures/shorts/short-haewon-paperlife-2027-03-14-8h-paper-folding-asmr.yaml` |
| Seed inquiry queue | `tests/fixtures/crm/inquiries-sungkyul-sangsa-2027-03-16-to-17.yaml` (42 inquiries) |
| Seed Cedar bundle | `tests/fixtures/cedar/j158/cedar-bundle-dual-tenant-disclosure-v1.cedar` |
| Wire mock — shorts autoscale | `tests/mocks/shorts-autoscale-2027.toml` |
| Wire mock — cell warm-start | `tests/mocks/cell-warm-start-kr-seoul.toml` |
| Wire mock — moorim paper vendor | `tests/mocks/moorim-paper-supplier.toml` |
| Wire mock — KR-LSA evaluator | `tests/mocks/kr-lsa-weekly-hours.toml` |
| Frozen clock | `freeze_clock(2027-03-17T14:18:22+09:00)` |
| Frozen creator metrics | `views=21.7M, watch-through=94%, trending_rank=2` |

## Seed data summary

| Datum | Value |
|---|---|
| Hae-Won passkey root | `passkey-galaxy-s25-haewon-personal-2025-09` |
| Personal tenant ID | `personal-haewon-kim-kr` |
| Employer tenant ID | `sungkyul-sangsa-print-co-kr` |
| Disclosure record ID | `disclosure-haewon-kim-sungkyul-sangsa-2024-08-12` |
| Disclosure expiry | `2027-08-12T00:00:00+09:00` |
| Personal cell | `kr-seoul-shorts-creator-tier-4` |
| Employer cells | primary + burst-1 + burst-2 + secondary + busan-readonly |
| Creator content ID | `short-haewon-paperlife-2027-03-14-8h-paper-folding-asmr` |
| Views at test start | 21,700,000 |
| Scale factor | 8.4× |
| Expected order-intake spike | 3.7× |
| HLC tier | default; rebalance uses HLC; KR-LSA evaluator uses TrueTime-class |

## Test catalog

### T-J158-001 — Personal-tenant autoscale activates internally

**Pre-conditions:** clock `2027-03-17T14:18:22+09:00`. Personal-tenant cell at baseline 6 replicas.

**Action sequence:**

1. Shorts µservice emits capacity signal at 14:18:22
2. Cell autoscale engages
3. Replicas ramp 6 → 28 by 14:18:42

**Expected events:**

- `EVT-J158-AUTOSCALE-SIGNAL-000` sealed in `personal-haewon-kim-kr` only
- `EVT-J158-AUTOSCALE-PERSONAL-001` sealed in `personal-haewon-kim-kr` only

**Pass criteria:**

- Both audits visible only in personal tenant; employer tenant query returns empty
- Replicas reach 28 within 20 s
- No leak to employer tenant: confirmed by direct query

**Fail criteria:** any event in employer tenant; ramp >60 s; partial scale-up.

### T-J158-002 — Disclosure signal sends with active disclosure record

**Pre-conditions:** disclosure-haewon-kim-sungkyul-sangsa-2024-08-12 active.

**Action sequence:**

1. Hae-Won composes signal at 14:24
2. Payload includes Korean message, 612 bytes
3. Send

**Expected events:**

- `EVT-J158-DISCLOSURE-SIGNAL-002` dual-sealed in BOTH tenants
- MLS group for signal ID `signal-haewon-2027-03-17-1424` created
- E2EE preserved end-of-line

**Pass criteria:**

- Cedar decision: `permit` with reason `creator-employer-disclosure-active`
- Payload size validated: 612 ≤ 1024
- Payload class validated: `creator_spike_info_only`
- No audience PII; no revenue figures; structural-assertions all true
- Both Lee Min-Jun and Hae-Won's employer-tenant principal receive copy
- Hangul preservation: "사장님" + "해원" preserved byte-exact

**Fail criteria:** any field rejected; Cedar deny; PII leaked; size overflow; Hangul normalized.

### T-J158-003 — Disclosure signal rejected without active disclosure record

**Pre-conditions:** Variant: disclosure record expired or revoked.

**Action sequence:** Same as T-J158-002.

**Expected events:**

- Cedar deny
- `EVT-J158-DISCLOSURE-INACTIVE-DENY-002a` dual-sealed
- HTTP 403 with body `{"error":"disclosure_record_not_active"}`

**Pass criteria:**

- Signal NOT delivered
- Audit dual-sealed
- Hae-Won sees a clarification: "your disclosure record has expired; please re-sign with your employer to enable this signal"

**Fail criteria:** signal delivered despite inactive record.

### T-J158-004 — Disclosure signal rejected with PII payload

**Pre-conditions:** Variant: payload contains text like "user @johndoe asked about..."

**Action sequence:** Submit signal with PII-flagged content.

**Expected events:**

- DLP scan flags
- `EVT-J158-DISCLOSURE-PII-LEAK-DENY-008b` dual-sealed
- HTTP 403

**Pass criteria:**

- Signal blocked
- User sees diff highlighting the flagged content
- No partial delivery

**Fail criteria:** signal accepted; partial leak.

### T-J158-005 — Cell rebalance workflow lifecycle (5 states)

**Pre-conditions:** T-J158-002 passed.

**Action sequence:** Walk all 5 transitions per `schemas/cell-rebalance-state-machine.yaml`.

**Expected events:** 4 dual-sealed transition events + 1 closing event.

**Pass criteria:**

- All transitions in order
- Each transition Cedar-gated; required context present
- Skip from `capacity_signal_detected` to `traffic_shift` denied with audit `EVT-J158-WORKFLOW-INVALID-TRANSITION-008d`
- p95 transition latency ≤ 380 ms

**Fail criteria:** any skip allowed; missing context accepted.

### T-J158-006 — Owner co-sign for rebalance

**Pre-conditions:** Workflow in `rebalance_proposed`.

**Action sequence:**

1. Lee Min-Jun reads proposal at 14:42:00
2. Signs with passkey + face_id at 14:42:08
3. Workflow advances to `cross_cell_grant_negotiated`

**Expected events:**

- `EVT-J158-REBALANCE-OWNER-COSIGN-003b` sealed in employer tenant

**Pass criteria:**

- Two-factor confirm (passkey + face_id) required
- Attestation text persisted
- Decline path is functional (manager can decline; workflow stops; alternate plan suggested)

**Fail criteria:** single-factor accepted; decline path missing.

### T-J158-007 — Burst cell warm-start

**Pre-conditions:** T-J158-006 passed.

**Action sequence:**

1. `cell` µservice initiates warm-start of `burst-1` + `burst-2`
2. Both cells reach `ready` within 25 min

**Expected events:**

- `EVT-J158-CELLS-WARMED-003c` sealed in employer tenant
- Both cells visible in ops console as `ready`

**Pass criteria:**

- Warm-start completes ≤ 25 min per cell
- Both cells pass health-check + capacity-validate
- No leak to personal tenant

**Fail criteria:** warm-start fails; cells not healthy.

### T-J158-008 — Traffic shift gradual ramp

**Pre-conditions:** T-J158-007 passed.

**Action sequence:**

1. Initiate traffic shift at 15:02:18
2. 10 increments over 90 min
3. Each increment validates latency + error rate before advancing

**Expected events:**

- 10 audits `EVT-J158-TRAFFIC-INCREMENT-004-{n}` (n=1..10)
- 1 closing audit `EVT-J158-REBALANCE-TRAFFIC-SHIFT-004`

**Pass criteria:**

- Final distribution exact: 40/32/28
- p95 latency on every cell ≤ 280 ms throughout (rollback threshold)
- Error rate ≤ 1.5% throughout
- Pause + rollback paths functional (test by pausing at increment 6 and resuming)

**Fail criteria:** latency breach without pause; ramp out of order; rollback path broken.

### T-J158-009 — Cedar deny: employer → personal probe

**Pre-conditions:** All tenants active.

**Action sequence:**

1. From employer-tenant ops console, attempt `GET /v1/tenants/personal-haewon-kim-kr/shorts/metrics`

**Expected events:**

- Cedar evaluates `forbid` (FORBID-1)
- `EVT-J158-CEDAR-DENY-EMPLOYER-TO-PERSONAL-008` dual-sealed
- HTTP 403

**Pass criteria:**

- Deny is explicit; no silent allow
- Audit visible in BOTH tenants
- ops console shows boundary message, not generic 500

**Fail criteria:** any allow; single-seal; generic error.

### T-J158-010 — Cedar deny: personal → employer probe without permit

**Pre-conditions:** All tenants active.

**Action sequence:** From personal tenant attempt `GET /v1/tenants/sungkyul-sangsa-print-co-kr/tasks`.

**Expected events:**

- Cedar deny
- `EVT-J158-CEDAR-DENY-PERSONAL-TO-EMPLOYER-NO-PERMIT-008a` dual-sealed
- HTTP 403

**Pass criteria:**

- Deny explicit
- Audit dual-sealed
- User sees boundary message

**Fail criteria:** any allow; single-seal.

### T-J158-011 — Hangul + Hanja preservation invariant

**Pre-conditions:** Persona seed includes Korean name fields.

**Action sequence:**

1. Write `김해원` to identity field
2. Write `성결상사 인쇄소` to tenant display name
3. Write `사장님` in messenger text
4. Read each back from both tenants

**Expected behavior:** Byte-exact match.

**Pass criteria:**

- All Hangul preserved UTF-8 NFC
- Hanja (if used) preserved
- Search "김해원" matches; search "kimhaewon" (romanized) does NOT match unless flag explicit
- PDF render uses Noto Sans KR; visual inspection passes

**Fail criteria:** any normalization; romanization fallback; visual corruption.

### T-J158-012 — KR-LSA evaluator weekly-hours cap

**Pre-conditions:** Staff projections seeded.

**Action sequence:** Evaluate weekly hours for each staff member after rebalance staffing changes.

**Expected behavior:**

- Hae-Won: 38.5 hr → green
- Park Jae-Won: 47.2 hr → green-monitor (within cap but flagged)
- Lee Min-Jun: 51.8 hr → yellow (within cap 52 but recommend redistribution)
- Variant: any staff projected >52 → red + deny additional shift assignment

**Pass criteria:**

- All evaluations match expected
- Red result blocks shift assignment with audit `EVT-J158-KR-LSA-OVERCAP-DENY-008e`
- Yellow result surfaces recommendation but does not block

**Fail criteria:** red result allowed; cap math wrong.

### T-J158-013 — Order-intake task materialization + routing

**Pre-conditions:** 42 inquiries seeded.

**Action sequence:**

1. Task materialization at 16:42 KST
2. Routing per `tasks-order-intake-burst-v1` template

**Expected events:**

- 42 tasks materialized
- 14 routed to Hae-Won (logistics)
- 22 routed to Park Jae-Won (binding)
- 6 escalated to Lee Min-Jun (high-value or contract-needing)

**Pass criteria:**

- All inquiries → tasks; no orphans
- Routing matches template rules
- SLA timers running
- Hangul customer names preserved

**Fail criteria:** orphans; mis-routing; SLA timer not started.

### T-J158-014 — Dual-tenant audit-chain merkle coherence

**Pre-conditions:** All tests run; 134 audit events generated total.

**Action sequence:**

1. Walk both tenant's audit chains
2. Validate merkle linkage
3. Cross-validate dual-seal events appear in BOTH tenants with consistent payload hash

**Expected behavior:**

- Each dual-seal event has byte-identical payload-hash in both tenants
- Merkle chains validate independently in each tenant
- Cross-tenant trace_id ties the two streams together

**Pass criteria:**

- 100% dual-seal events match
- Both merkle chains validate
- trace_id resolution: any cross-tenant event resolvable from either side

**Fail criteria:** hash mismatch; chain break; trace_id unresolvable.

### T-J158-015 — Post-rebalance validation

**Pre-conditions:** T-J158-005 through T-J158-013 passed.

**Action sequence:**

1. Run validation at 18:42:00
2. Check all metrics + invariants

**Expected events:** `EVT-J158-POST-REBALANCE-VALIDATION-009` sealed in employer tenant.

**Pass criteria:**

- All cell latencies p95 ≤ 200 ms
- No orphaned messages
- Boundary invariants hold (employer→personal + personal→employer-no-permit deny)
- KR-LSA evaluator green/yellow only (no red)
- Audit chain coherent
- Daily reassessment scheduled for 04:00 KST next day

**Fail criteria:** any metric breach; invariant fail; audit incoherent.

## Performance gates

| Operation | p50 | p95 | p99 |
|---|---|---|---|
| Personal autoscale engage | 8 s | 20 s | 30 s |
| Disclosure signal Cedar eval | 40 ms | 110 ms | 220 ms |
| Disclosure signal delivery | 180 ms | 480 ms | 820 ms |
| Cell warm-start | 18 min | 22 min | 28 min |
| Traffic shift increment | 90 s | 180 s | 320 s |
| KR-LSA evaluation | 60 ms | 180 ms | 280 ms |
| Boundary invariant probe | 30 ms | 90 ms | 180 ms |
| Hangul write+read roundtrip | 12 ms | 40 ms | 80 ms |

## Cross-tenant invariant tests (run after every other test)

| Invariant | Probe | Pass condition |
|---|---|---|
| employer → personal | any read action | 403 + dual-seal |
| personal → employer no permit | any read action | 403 + dual-seal |
| disclosure signal w/ PII | DLP scan flag | refused + audit |
| disclosure signal > 1024 bytes | size validate | refused + audit |
| rebalance state-machine skip | direct → traffic_shift | refused + audit |
| Hangul normalize | write field then compare | bytes match strictly |
| KR-LSA red | shift assignment | refused + audit |
| Reverse autoscale signal employer → personal | employer attempts to push to personal cell | refused; cellular boundary holds |

## Chaos scenarios

1. **Personal-tenant autoscale fails (replica budget exhausted)** — degrades to best-effort; signal to employer still sends; user sees explicit message about degraded creator-side performance
2. **Burst-1 warm-start fails** — workflow proposes alternate cell (Busan); user approves; burst window shortens by 24 hr
3. **Disclosure-record signer unreachable** — falls back to "view-only" disclosure mode; signal cannot send; user prompted to schedule renewal
4. **MLS DS partition personal ↔ employer** — disclosure signal queues locally; delivers on recovery; epoch correctness preserved
5. **KR-LSA evaluator service degraded** — workflow auto-pauses; user explicit "compliance check unavailable; manual override requires extra signature"

## Sign-off checklist

- [ ] All 15 tests pass
- [ ] All invariant probes return expected dual-seal
- [ ] Performance gates met
- [ ] Chaos scenarios complete without data loss
- [ ] All 4 µservices in `/microservices/` resolve: tasks, workflow-engine, shorts, messenger
- [ ] All 8 ADRs cited resolve
- [ ] Hangul + Hanja preservation invariant: 0 normalization events
- [ ] Disclosure permit holds dual-seal across all signals
- [ ] Reverse direction (employer → personal) denies in all variants
- [ ] DPO sign-off both tenants

## Stop condition

Plan complete when all 15 tests pass, the dual-tenant boundary holds inverse (employer→personal denied symmetrically with personal→employer-no-permit), the disclosure-signal mechanism preserves info-only one-way semantics, Hangul + Hanja preservation invariant holds across writes/reads, the cell-rebalance workflow reaches `post_rebalance_validation`, KR-LSA stays within statutory caps, and the dual audit chains link via cross-tenant trace_id with byte-identical payload hashes.
