---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j156-carlos-reyes-ii-maintenance-emergency-after-hours
date: 2026-05-20
authority_tier: 2
status: draft
---

# j156 — Integration test plan

Intern-buildable plan: a new engineer stands up the seeded two-tenant fixture (`cascade-fm-services-llc-us` + `meridianstack-hosting-co-us`) plus the wire mocks for EPA E-GGRT, Trane vendor line, badge readers, Fluke meter Bluetooth, FLIR thermal camera, and the regulator anchors, then walks every test in order. Every test names seed values, exact API calls, expected event chain across the two tenants (dual-seal invariant), and pass/fail criteria.

## Test environment

| Component | Source |
|---|---|
| Seed tenant — Cascade | `tests/fixtures/tenants/cascade-fm-services-llc-us.yaml` |
| Seed tenant — MeridianStack | `tests/fixtures/tenants/meridianstack-hosting-co-us.yaml` |
| Seed tenant — Trane vendor | `tests/fixtures/tenants/trane-technologies-emergency-vendor-na.yaml` |
| Seed personas | `tests/fixtures/personas/{carlos-reyes-ii,tomas-alvarado,priya-subramanian,marcus-whitfield,yesenia-reyes,trane-emergency-dispatch-anita}.yaml` |
| Seed certifications | `tests/fixtures/learning-management/carlos-certs-2026-09.yaml` (EPA-608-Universal exp 2027-04-18; NFPA-70E-CAT-2 exp 2027-01-12; OSHA-30 exp 2028-03-04) |
| Seed facility | `tests/fixtures/plant-maintenance/dc-phx-3-facility-map.yaml` (rooms, panels, pumps, chillers) |
| Seed equipment | `tests/fixtures/plant-maintenance/7b-pump-04-trane-rtaf-200.yaml` (model, vintage, prior service history) |
| Seed cylinder | `tests/fixtures/refrigerant-cylinders/R454B-CYL-DC-PHX-3-2026-Q3-007.yaml` |
| Seed contract | `tests/fixtures/contracts/contract-meridianstack-cascade-fm-2024-09-01.yaml` |
| Seed escalation tree | `tests/fixtures/incident-management/esc-tree-dc-phx-3-after-hours-hvac-2026.yaml` |
| Seed Cedar bundle | `tests/fixtures/cedar/j156/cedar-bundle-cascade-meridianstack-cross-tenant-v1.cedar` |
| Wire mock — EPA E-GGRT | `tests/mocks/epa-egrt-2026.toml` |
| Wire mock — Trane vendor line | `tests/mocks/trane-emergency-na-2026.toml` |
| Wire mock — Fluke T6-1000 BT | `tests/mocks/fluke-t6-1000-bluetooth.toml` |
| Wire mock — FLIR ONE Pro | `tests/mocks/flir-one-pro-usbc.toml` |
| Wire mock — badge readers | `tests/mocks/hid-iclass-se-r40.toml` |
| Wire mock — Cascade payroll | `tests/mocks/cascade-payroll-adp-workforce-now.toml` |
| Wire mock — XCover7 Pro TEE | `tests/mocks/samsung-xcover7-tee.toml` |
| Frozen clock | `freeze_clock(2026-10-17T02:47:02-07:00)` then advance per test |
| Frozen weather | `phoenix-az = 97°F at 02:47, dewpoint 62°F, wind 3 mph SE` |

## Seed data summary

| Datum | Value |
|---|---|
| Carlos passkey root | `passkey-samsung-xcover7-carlos-cascade-2025-09` |
| Cascade employee id | `cascade-emp-carlos-reyes-ii-2018-04-19` |
| Vendor contract | `contract-meridianstack-cascade-fm-2024-09-01` (24/7 on-call NFPA-70E + EPA-608 scope) |
| Incident id | `incident-dc-phx-3-2026-10-17-0247-7b-chl-overtemp` |
| Permit id | `permit-dc-phx-3-2026-10-17-0251-7b` |
| Cross-grant id | `cross-grant-cascade-meridianstack-2026-10-17-carlos-0247` |
| Cylinder of origin | `R454B-CYL-DC-PHX-3-2026-Q3-007` (charged by Marcus Whitfield 2026-07-22, 51.4 lb in-loop) |
| Release estimate | 1.4 lb R-454B |
| Emergency-call rate | $1,247 base + $87.50/hr after 2-hr min |
| HLC tier | default; LOTO transitions use HLC; cross-tenant audit uses HLC + sequence; HIPAA roll-up uses TrueTime-class |

## Test catalog

### T-J156-001 — P1 page acknowledgment within 90 s

**Pre-conditions:** clock `2026-10-17T02:47:02-07:00`. Carlos's phone enrolled. Cross-grant pre-issued in provisional state.

**Action sequence:**

1. `incident-management` emits the P1 page on the escalation tree
2. Carlos's phone displays at 02:47:14 MST (12 s push)
3. Carlos acks at 02:48:11 MST via latent-print + passkey

**Expected events:**

- `EVT-J156-INCIDENT-DETECTED-000` sealed in `meridianstack-hosting-co-us`
- `EVT-J156-INCIDENT-ACK-001` dual-sealed in BOTH tenants
- Cross-grant state transitions `provisional` → `awaiting_co_sign`

**Pass criteria:**

- Push latency ≤ 30 s
- Ack-to-200 ≤ 380 ms
- Audit appears in BOTH tenants within 1 s of ack
- Auto-shed countdown shows 9 min 51 sec at ack moment
- Cedar permit on ack: `permit`

**Fail criteria:** push >60 s; ack appears in only one tenant; countdown drift >1 s.

### T-J156-002 — Cross-tenant grant issuance + binding

**Pre-conditions:** T-J156-001 passed. Vendor contract active.

**Action sequence:**

1. `identity` issues `cross-grant-cascade-meridianstack-2026-10-17-carlos-0247` at 02:48:18
2. Cascade tenant binds grant to Carlos's session

**Expected events:**

- `EVT-J156-IDENTITY-CROSS-GRANT-002` dual-sealed
- Grant `valid_from=02:47:00`, `valid_until=09:00:00`
- Grant scope includes exactly: `dc-phx-3-aisle-7b` + 9 named actions

**Pass criteria:**

- Grant active in both tenants within 200 ms
- Cedar evaluation at 02:48:42 of `incident.acknowledge` against MeridianStack tenant: `permit`
- Cedar evaluation of `incident.acknowledge` against any other facility (e.g. `dc-phx-4`): `forbid`
- Active-tenant pill flips to `Cascade · MeridianStack (scoped)` within 380 ms

**Fail criteria:** grant scope leaks beyond `aisle-7b`; pill remains italic (provisional) after binding; Cedar permits a non-7B target.

### T-J156-003 — Permit creation + Tomás co-sign + Priya co-sign

**Pre-conditions:** T-J156-002 passed.

**Action sequence:**

1. Carlos creates permit at 02:51:00
2. Tomás co-signs at 02:51:42 (Cascade-side)
3. Priya co-signs at 02:53:08 (MeridianStack-side)

**Expected events:**

- `EVT-J156-WORKFLOW-PERMIT-CREATED-003` (Cascade)
- `EVT-J156-WORKFLOW-PERMIT-COSIGN-003a` (Tomás, dual-sealed)
- `EVT-J156-WORKFLOW-PERMIT-COSIGN-003b` (Priya, dual-sealed)
- Permit status `awaiting_co_sign` → `co_signed_active` at 02:53:08

**Pass criteria:**

- All three audit events sealed within 1 s of each action
- Permit ID exact: `permit-dc-phx-3-2026-10-17-0251-7b`
- Each co-sign includes passkey assertion + attestation text
- Cert verification calls `learning-management` and pulls 3 active certs

**Fail criteria:** any cosign missing dual-seal; permit activates without both co-signs; expired cert accepted.

### T-J156-004 — LOTO state machine (5 transitions, 4 photos, 1 voltage test)

**Pre-conditions:** T-J156-003 passed. Carlos on-site.

**Action sequence:** Walk each transition per `schemas/loto-state-machine.yaml`. Photos uploaded. Fluke T6 readings 0/0/0 V.

**Expected events:**

- 4 dual-sealed LOTO transition events (003c, 003d, 003e, 004)
- Photo evidence linked per transition
- Fluke BT receipt recorded

**Pass criteria:**

- State machine refuses any invalid transition (e.g. `lockout_pending → energized_normal` direct); refuse + audit `EVT-J156-LOTO-INVALID-TRANSITION-009c`
- Final state: `locked_isolated_verified`
- Joint-observe toggle (Priya) confirmed before voltage-test transition
- All 4 photos passed EXIF validation (GPS + timestamp + task-id)
- p95 transition latency ≤ 320 ms

**Fail criteria:** any invalid transition accepted; photo missing GPS; Priya joint-observe not recorded; voltage reading >0.5 V on any phase.

### T-J156-005 — 11 tasks materialize + complete

**Pre-conditions:** T-J156-003 passed.

**Action sequence:**

1. Bulk materialize at 02:51:14
2. Per-task complete with photo + GPS

**Expected events:**

- `EVT-J156-TASKS-MATERIALIZED-005` × 1
- `EVT-J156-TASKS-COMPLETED-005-{1..11}` × 11

**Pass criteria:**

- All 11 tasks reach `completed`
- Each task carries ≥1 photo with valid EXIF
- Each task seals dual-tenant audit
- Task #6 (pump-rebuild) records part `KIT-RTAF-200-SHAFT-SEAL-V3` consumed
- Task #7 (refrigerant-recovery) records `delta_weight = 1.4 lb`
- Task #10 (CMMS) closes work order `WO-DC-PHX-3-2026-10-17-7B-PUMP-04-SHAFT-SEAL`
- p95 task-complete latency ≤ 480 ms

**Fail criteria:** any task without photo evidence; CMMS work order remains open; part consumption not recorded.

### T-J156-006 — EPA-608 disclosure for 1.4 lb release

**Pre-conditions:** T-J156-005 task #7 in progress.

**Action sequence:**

1. Open `wkfl-epa608-release-disclosure-dc-phx-3-2026-10-17` workflow
2. Populate form per §6 schema
3. Submit to EPA E-GGRT mock
4. Receive receipt `egrt-receipt-2026-10-17-dc-phx-3-001`

**Expected events:**

- `EVT-J156-EPA608-DISCLOSURE-006` triple-sealed (Cascade + MeridianStack + compliance regulator-anchor)
- E-GGRT receipt persisted with merkle anchor

**Pass criteria:**

- Disclosure delivered to E-GGRT mock within 60 s
- Receipt validated against deterministic mock signature
- 3 required photos attached (cylinder label + leak site + recovery unit)
- Cylinder provenance complete: charged by Marcus Whitfield 2026-07-22 to 51.4 lb; recovered to 50.0 lb; release 1.4 lb
- Submission cannot succeed without all three attestations + cause field

**Fail criteria:** submit accepted with missing field; receipt missing; cylinder provenance broken; release estimate >2 lb without supervisor review escalation.

### T-J156-007 — Cross-tenant audit dual-seal invariant fuzz

**Pre-conditions:** All prior tests pass. Fuzz harness configured.

**Action sequence:** 500 generated cross-tenant operations from the closed set `{incident.acknowledge, incident.resolve, tasks.execute, workflow.permit_sign, workflow.loto_lock, workflow.loto_release, plant.cmms_close_workorder, compliance.epa608_disclose_release}` × {Cascade source, MeridianStack target}.

**Expected behavior:** Every permitted op MUST seal in BOTH tenants. Every denied op MUST also seal in BOTH (deny dual-seal under ADR-0263).

**Pass criteria:**

- 0 single-seal events
- 0 silent passes
- p99 dual-seal latency ≤ 240 ms
- Merkle linkage validates across all 500 events

**Fail criteria:** any single-seal; any silent pass; merkle chain breaks.

### T-J156-008 — Cross-tenant grant expiration at 09:00:00 MST exactly

**Pre-conditions:** T-J156-005 task #11 complete (closeout signed at 06:57:21). Carlos's grant still nominally active.

**Action sequence:**

1. Advance frozen clock to `2026-10-17T08:59:59.999-07:00`
2. Submit one final action (`plant.cmms_read_workorder`) — must permit
3. Advance to `2026-10-17T09:00:00.000-07:00`
4. Submit `plant.cmms_read_workorder` again — must deny

**Expected events:**

- `EVT-J156-CROSS-GRANT-EXPIRED-008` dual-sealed at 09:00:00.000
- `EVT-J156-CEDAR-DENY-EXPIRED-GRANT-009` dual-sealed on the post-expiration attempt

**Pass criteria:**

- Pre-expiration action returns 200; post-expiration returns 403
- Auto-revocation is scheduler-driven (no Cedar evaluation drift)
- Active-tenant pill drops `· MeridianStack (scoped)` suffix within 1 s
- A 30-day re-grant is requirable but defaults to denied; reissuance requires fresh incident or fresh manager approval

**Fail criteria:** action permitted post-09:00; pill remains showing scoped after expiration; auto-revoke doesn't fire.

### T-J156-009 — Overtime payroll computation

**Pre-conditions:** Carlos signed closeout at 06:57:21 MST. Cascade payroll mock active.

**Action sequence:**

1. `workplace-integration` computes hours: 6 h 18 min on-site (02:48:11 → 09:05:18 minus the lunch/recovery breaks tracked by the tasks pipeline)
2. Emergency-call premium: $1,247 base + $87.50/hr after 2 hrs = $1,247 + 4.30 × $87.50 = $1,247 + $376.25 = $1,623.25
3. Post to Cascade payroll within 1 hr of closeout

**Expected events:**

- `EVT-J156-PAYROLL-OVERTIME-POSTED-009` sealed in `cascade-fm-services-llc-us`
- NO event in MeridianStack (payroll is a Cascade-internal concern; vendor invoicing is separate)
- SMS to Yesenia per Carlos's notification settings

**Pass criteria:**

- Computed amount $1,623.25 ± $0.01
- Posted within 60 min of closeout
- No payroll leak to MeridianStack tenant
- Yesenia's SMS arrives within 10 min of post (no message content visible to MeridianStack)

**Fail criteria:** wrong amount; payroll posted to wrong tenant; SMS delayed >30 min.

### T-J156-010 — HIPAA daily-roll-up audit

**Pre-conditions:** Incident closed.

**Action sequence:**

1. At `2026-10-18T00:00:00-07:00`, the HIPAA roll-up scheduler fires
2. `audit-chain` walks all 87 events of the day
3. Computes merkle root + per-event proof
4. Delivers daily-roll-up to MeridianStack's covered-entity customer (the healthcare-integration tenant)

**Expected events:**

- `EVT-J156-HIPAA-ROLLUP-MERKLE-011` sealed in MeridianStack
- Roll-up artifact `rollup-dc-phx-3-2026-10-17-hipaa-fac-control-audit.json` delivered

**Pass criteria:**

- Exact 87 events covered
- Merkle root deterministic and reproducible
- Roll-up includes facility-control event class breakdown
- Covered-entity customer's SLA: receives by 09:00 next day
- 0 PII content in the roll-up (only event class + count + merkle proof)

**Fail criteria:** event count mismatch; PII leak; SLA missed.

### T-J156-011 — Offline resilience (LTE loss in mechanical room)

**Pre-conditions:** Carlos at MECH-RM-07B. Simulate LTE blackout from 03:19:00 to 03:34:00 MST.

**Action sequence:**

1. Inject LTE loss; phone falls back to local-queue mode
2. Carlos completes task #3, #4, #5 offline (15 events queued)
3. LTE returns at 03:34:00
4. Flush queue

**Expected events:**

- During blackout: 0 cross-tenant events (correct — events queued)
- At flush: all 15 events sealed in BOTH tenants with original timestamps preserved
- `EVT-J156-OFFLINE-SYNC-FLUSH-013` sealed

**Pass criteria:**

- All 15 events carry original `local_event_time` (NOT the flush time)
- HLC sequence preserves causal order
- No data loss
- Auto-shed countdown still PAUSED throughout (the on-site marker survives offline)

**Fail criteria:** any event lost; timestamps coerced to flush time; HLC out of order.

### T-J156-012 — Cedar deny: missing cert variant

**Pre-conditions:** Variant fixture where Carlos's NFPA-70E-Cat-2 cert is expired (synthetic data).

**Action sequence:** Permit creation flow attempted as in T-J156-003.

**Expected events:**

- `EVT-J156-CEDAR-DENY-CERT-MISSING-009b` dual-sealed
- HTTP 403 with body `{"error":"missing_certification","required":"NFPA-70E-CAT-2","status":"expired"}`
- Permit creation refused

**Pass criteria:**

- Deny is explicit; permit cannot be force-created
- Manager-override path (Tomás) requires a second co-signer; manager override CANNOT be a single click
- Audit appears in both tenants
- Carlos sees an actionable message: "renew cert before next on-call rotation"

**Fail criteria:** permit creation succeeds despite missing cert; single-click override; manager-override not audited.

### T-J156-013 — Trane vendor escalation (B2B partner-tenant channel)

**Pre-conditions:** Release >1 lb confirmed (from T-J156-006).

**Action sequence:**

1. `messenger` opens sub-channel with `trane-technologies-emergency-vendor-na` principal
2. Carlos sends loop diagnostic + release estimate
3. Trane's emergency dispatch (Anita) responds with factory-bulletin reference and parts availability

**Expected events:**

- `EVT-J156-MESSENGER-VENDOR-ESCALATION-007a` triple-sealed (Cascade + MeridianStack + Trane)
- MLS sub-group created with epoch 0

**Pass criteria:**

- Triple-seal validates across all three tenants' audit chains
- Trane's response delivered E2EE end-of-line
- No data leak from MeridianStack tenant to Trane beyond the redacted disclosure form (R-454B + 1.4 lb + part number + cylinder ID — not site PII)
- Cedar policy on Trane's read scope is read-only on the disclosure-bundle resource

**Fail criteria:** Trane sees site PII outside disclosure scope; triple-seal incomplete; MLS group fails to bootstrap.

### T-J156-014 — Post-mortem auto-population

**Pre-conditions:** Incident closed.

**Action sequence:**

1. Priya joins ops review at 10:14 MST
2. `incident-management` auto-populates post-mortem template from audit-chain
3. Three actions drafted; ops review records sign-off

**Expected events:**

- `EVT-J156-POSTMORTEM-CLOSED-013` sealed
- Three follow-up tasks created in MeridianStack tenant's `tasks` µservice

**Pass criteria:**

- Auto-population includes 100% of audit events with merkle proofs
- 3 follow-up actions accurate (Trane factory inspection / pressure-decay alarm / runbook update)
- Sign-off requires NOC controller + facility ops manager
- Post-mortem PDF generated, signed, and archived in plant-maintenance

**Fail criteria:** auto-population misses events; follow-up actions wrong or missing; PDF unsigned.

### T-J156-015 — Failure injection: Cedar service degraded

**Pre-conditions:** Active permit + ongoing tasks.

**Action sequence:**

1. Inject Cedar service degradation (50% timeouts)
2. Carlos attempts task #6 completion

**Expected behavior:**

- Circuit-breaker opens: fail-closed (deny all)
- UI surfaces red "policy service degraded — retry"
- Audit `EVT-J156-CEDAR-DEGRADED-CIRCUIT-OPEN-CHAOS-15` sealed
- Carlos's queue state preserved; retry succeeds when Cedar recovers

**Pass criteria:**

- No silent permit; no soft-allow path
- p99 recovery time ≤ 90 s after Cedar restored
- 0 data corruption
- Audit explicitly shows the chaos event

**Fail criteria:** any silent permit during degradation; data corruption; recovery >5 min.

## Performance gates

| Operation | p50 | p95 | p99 |
|---|---|---|---|
| P1 page push | 8 s | 22 s | 30 s |
| Ack-to-200 | 180 ms | 380 ms | 620 ms |
| Cross-grant issuance | 90 ms | 200 ms | 380 ms |
| Permit co-sign | 220 ms | 480 ms | 820 ms |
| LOTO transition | 140 ms | 320 ms | 580 ms |
| Task complete | 200 ms | 480 ms | 820 ms |
| EPA E-GGRT submit | 28 s | 53 s | 90 s |
| Cedar evaluation | 60 ms | 180 ms | 280 ms |
| Dual-seal write | 100 ms | 240 ms | 420 ms |

## Cross-tenant invariant tests (run after every other test)

| Invariant | Probe | Pass condition |
|---|---|---|
| Cascade tech outside `aisle-7b` | `carlos → tasks.execute(aisle-8a)` | 403 + dual-seal |
| Cross-grant outside time window | `carlos → action @ 09:00:01` | 403 + dual-seal |
| Carlos without active cert | `carlos(expired cert) → permit create` | 403 + dual-seal |
| MeridianStack non-controller tries permit | `noc-junior → permit cosign` | 403 + dual-seal |
| LOTO skip transition | `direct lockout → energized` | refused + audit |
| EPA submit incomplete | missing required field | refused + audit |

## Chaos scenarios

1. **Trane vendor unreachable for 12 min** — escalation queues, then proceeds to alternate vendor (Carrier emergency line); permit-to-work continues
2. **Cross-grant scheduler 17-second clock skew** — TrueTime fence catches drift; grant expires at most 1 ms late
3. **MLS DS partition between Cascade and MeridianStack** — messenger queues; Carlos and Priya fall back to voice (logged)
4. **Audit-chain merkle service degraded** — fail-closed: writes pause; UI explicit "audit paused — work paused"
5. **Badge reader offline at staff entrance** — manual override by Priya from NOC with full audit + manager attestation

## Sign-off checklist

- [ ] All 15 tests pass
- [ ] All invariant probes return dual-seal
- [ ] Performance gates met
- [ ] Chaos scenarios complete without data loss
- [ ] All 5 µservices in `/microservices/` resolve: incident-management, tasks, messenger, audit-chain, workflow-engine
- [ ] All 8 ADRs cited resolve: ADR-0244, ADR-0243, ADR-0263, ADR-0248, ADR-0252, ADR-0250, ADR-0254, ADR-0247
- [ ] EPA E-GGRT receipt validates against deterministic mock signature
- [ ] HIPAA daily-roll-up delivers to covered entity by 09:00 next day
- [ ] DPO sign-off: MeridianStack-side DPO + Cascade-side DPO

## Stop condition

Plan complete when all 15 tests pass on the seeded environment, the dual-seal invariant holds under the 500-variant fuzz, the EPA-608 disclosure delivers within statutory window, the cross-tenant grant expires deterministically at 09:00:00.000 MST, and the HIPAA daily-roll-up produces a merkle proof reproducible from the audit-chain.
