---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j157-diana-lazar-print-operator-batch-defect-and-quality-recall
date: 2026-05-20
authority_tier: 2
status: draft
---

# j157 — Integration test plan

Intern-buildable plan: stand up the seeded two-tenant fixture (`tipografia-lazar-petrescu-ro` + `antibiotice-sa-ro`) plus mocks for the Heidelberg press control bus, GMI ColorProof + Prinect Inpress Control, Cargus courier API, ANMDMR regulator template service, and an external FOGRA reference lab. Walk every test in order; every test names seed values, exact API calls, expected event chain across both tenants (dual-seal mandatory), and pass/fail criteria.

## Test environment

| Component | Source |
|---|---|
| Seed tenant — Tipografia | `tests/fixtures/tenants/tipografia-lazar-petrescu-ro.yaml` |
| Seed tenant — Antibiotice | `tests/fixtures/tenants/antibiotice-sa-ro.yaml` |
| Seed tenant — ANMDMR-inspectorate (hold) | `tests/fixtures/tenants/anmdmr-inspectorate-ro.yaml` |
| Seed personas | `tests/fixtures/personas/{diana-lazar,mihai-lazar-petrescu,liviu-apostol,andrei-tabarca,vladimir-csikos,marius-iancu,cristina-munteanu,andrei-popescu,carmen-ene}.yaml` |
| Seed press | `tests/fixtures/plant-maintenance/heidelberg-cx-102-6-lx-01.yaml` |
| Seed press #2 | `tests/fixtures/plant-maintenance/heidelberg-cx-102-6-lx-02.yaml` |
| Seed batch | `tests/fixtures/quality-management/BCH-2027-02-23-0612-pharma-leaflet-NSAID-RO.yaml` |
| Seed approved leaflet text | `tests/fixtures/compliance/anmdmr-approved-text-ibuprofen-400mg-2027-01-18.json` |
| Seed certifications | `tests/fixtures/learning-management/diana-certs-2026-09.yaml` (FOGRA-PSO-L2 + ISO-12647-2-Trained) |
| Seed MSA | `tests/fixtures/contracts/msa-tipografia-antibiotice-2024-q2.yaml` (§7.4 defect-liability) |
| Seed Cedar bundle | `tests/fixtures/cedar/j157/cedar-bundle-tipografia-antibiotice-recall-v1.cedar` |
| Wire mock — Heidelberg Prinect | `tests/mocks/heidelberg-prinect-inpress-control-v3.toml` |
| Wire mock — GMI ColorProof | `tests/mocks/gmi-colorproof-2024.toml` |
| Wire mock — X-Rite eXact 2 | `tests/mocks/xrite-exact-2.toml` |
| Wire mock — Cargus courier | `tests/mocks/cargus-express-ro-v4.toml` |
| Wire mock — ANMDMR regulator | `tests/mocks/anmdmr-recall-template-2026.toml` |
| Wire mock — FOGRA reference lab | `tests/mocks/fogra-reference-lab-de.toml` |
| Frozen clock | `freeze_clock(2027-02-23T11:42:14+02:00)` then advance per test |
| Frozen pressroom state | `ambient_noise_db = 75; FOGRA D50 lighting active; sheet_count_at_freeze = 23847` |

## Seed data summary

| Datum | Value |
|---|---|
| Diana passkey root | `passkey-toughpad-diana-lazar-2025-09` |
| Batch id | `BCH-2027-02-23-0612-pharma-leaflet-NSAID-RO` |
| Approved text | "Nu administrați copiilor sub 6 ani fără sfatul medicului" (UTF-8 NFC) |
| Defect: rendered text | "Nu administrați copiilor ub 6 ani fără sfatul medicului" |
| ΔE2000 cap | 3.0 (FOGRA-PSO solid) |
| ΔE2000 at breach | 4.7 |
| Registration shift | 1.2 mm Y |
| Affected zone | sheets 22,000 → 23,847 (1,848 sheets) |
| Sample plan | ISO-2859-1 AQL 0.4 (pharma-class), n=315, c=3 |
| MSA §7.4 SLA | customer ack within 4 h |
| HLC tier | default; CAPA filing uses TrueTime-class for cross-tenant fence |

## Test catalog

### T-J157-001 — Telemetry breach → quality alert

**Pre-conditions:** clock `2027-02-23T11:42:14+02:00`. Press running at 16,500 sheets/hr. ΔE history flat at 1.4 ± 0.2 since 06:18 EET.

**Action sequence:**

1. Inject ΔE2000 reading 4.7 at sheet 23,847
2. Quality-mgmt evaluates rule `pharma_PIL + delta_e_2000 > 3.0`
3. Alert lands on Diana's tablet at 11:42:15

**Expected events:**

- `EVT-J157-QUALITY-TELEMETRY-BREACH-000` sealed in `tipografia-lazar-petrescu-ro`
- Tablet receives push within 1 s

**Pass criteria:**

- Push latency ≤ 1 s
- Alert payload includes ΔE trajectory (last 60 s) + press camera frame
- ΔE history is queryable by Diana via "show ΔE history" action

**Fail criteria:** push >2 s; alert missing camera frame; ΔE history empty.

### T-J157-002 — Operator-authority line stop (no manager gate)

**Pre-conditions:** T-J157-001 passed. Diana's FOGRA-PSO-L2 cert active (exp 2027-09-18).

**Action sequence:**

1. Diana taps LINE STOP at 11:42:38
2. Cedar evaluates `quality.production_line_stop`
3. Press receives halt command

**Expected events:**

- `EVT-J157-LINE-STOP-001` sealed in `tipografia-lazar-petrescu-ro`
- Press halt confirmed by 11:42:56
- `EVT-J157-PRESS-HALT-CONFIRMED-002` sealed

**Pass criteria:**

- Cedar decision: `permit`; reason text contains "operator certification IS authority"
- No manager-approval API call attempted (verified by inspecting outbound traffic — must be zero calls to any manager-gate endpoint)
- Press halt complete in ≤ 14 s
- 3 in-transit sheets moved to quarantine pallet (logged in `tasks`)
- Audit shows operator authority basis: `FOGRA-PSO-Operator-Level-2`

**Fail criteria:** any manager-gate call attempted; press halt >18 s; in-transit sheets not segregated; audit fails to record cert basis.

### T-J157-003 — Diacritic preservation invariant

**Pre-conditions:** Persona seed `diana-lazar.yaml` has legal name `Diana Lazăr` (UTF-8 NFC).

**Action sequence:**

1. Read Diana's name from `identity` µservice
2. Use in line-stop payload + audit-chain seal + customer notification + CAPA + handoff
3. After all writes, query each persisted field

**Expected behavior:** All persisted fields hold exact byte sequence `Diana Laz\xc4\x83r` (NFC) — never `Diana Lazar` (ASCII normalized).

**Pass criteria:**

- 0 fields contain "Lazar" without diacritic
- Search "Lazar" returns NO match unless diacritic-insensitive flag explicitly set
- Search "Lazăr" returns Diana's record
- Search "Lazár" (Hungarian acute, not Romanian breve) returns NO match — these are distinct characters
- CAPA PDF rendered with "Lazăr" intact

**Fail criteria:** any field stored as "Lazar"; search returns wrong matches; PDF renders incorrect glyph.

### T-J157-004 — Recall workflow state machine (6 states, valid transitions)

**Pre-conditions:** T-J157-002 passed.

**Action sequence:** Walk all valid transitions per `schemas/recall-state-machine.yaml`:
`stop_called → quarantine → defect_root_cause → customer_notify → recall_execute → closure_post_mortem`

**Expected events:**

- 5 transition audit events, each dual-sealed
- Each transition Cedar-gated

**Pass criteria:**

- All 5 transitions land in order
- Each transition includes evidence required by its state (e.g. `defect_root_cause` requires inspection report linked)
- Skip from `stop_called` to `closure_post_mortem` refused with audit `EVT-J157-WORKFLOW-INVALID-TRANSITION-012f`
- p95 transition latency ≤ 380 ms

**Fail criteria:** any out-of-order transition; skip allowed; missing evidence accepted.

### T-J157-005 — 14 tasks materialize + complete

**Pre-conditions:** T-J157-004 partial.

**Action sequence:**

1. Bulk materialize at 11:44:42
2. Complete tasks 1–14 in canonical order
3. Verify each task has evidence

**Expected events:** 14 task-complete audits, each dual-sealed.

**Pass criteria:**

- Each task carries the required evidence (counts, photos, root-cause inspection, tracking number, etc.)
- Tasks 6 (photos) requires ≥8 photos; the task does not complete with <8
- Task 7 (sample ship) requires tracking number + tamper seal
- Task 10 (root-cause confirm) requires linked inspection + Marius endorsement
- p95 task-complete latency ≤ 480 ms

**Fail criteria:** any task completes without required evidence.

### T-J157-006 — ANMDMR-approved-text deviation detection

**Pre-conditions:** Approved text seed loaded.

**Action sequence:**

1. Capture defect photograph of sheet 23,847
2. OCR pipeline extracts rendered text: "Nu administrați copiilor ub 6 ani fără sfatul medicului"
3. Compare to approved text via Levenshtein + structural diff

**Expected behavior:**

- Diff detects: missing "s" in "sub"
- Severity classification: `ANMDMR_approved_text_deviation` (critical)
- `EVT-J157-DEFECT-CLASSIFIED-004a` sealed with this severity

**Pass criteria:**

- Diff is exact and pinpoints the "s" → "" deletion
- Severity escalation triggers regulator-template preparation
- Diacritic-aware comparison: "administrați" matches "administrați" exactly (NFC), not normalized to "administrati"

**Fail criteria:** diff misses the missing "s"; severity downgraded; diacritic comparison normalizes.

### T-J157-007 — Cross-tenant customer notification within 90 min

**Pre-conditions:** T-J157-002–006 passed.

**Action sequence:**

1. Diana drafts customer notification at 12:31 EET
2. Sends at 12:42:18 EET (60 min after stop)
3. Cristina Munteanu reads + responds at 12:51:14 EET

**Expected events:**

- `EVT-J157-CUSTOMER-NOTIFY-005` dual-sealed
- `EVT-J157-CUSTOMER-CONFIRM-006` dual-sealed
- MLS group epoch advances 0 → 1 (notification) → 2 (response)

**Pass criteria:**

- Notification sent within 90 min of stop (T-J157-002 + 90 min = 13:12:38 latest)
- Response received within 4 h MSA SLA
- Both messages preserve diacritics
- E2EE preserved end-of-line
- CRM activity record created on Tipografia side; Antibiotice CRM-equivalent record created on their side
- ANMDMR-template prepared but `status=prepared_hold` (not sent)

**Fail criteria:** notification >90 min; diacritic loss; CRM record missing; ANMDMR sent prematurely.

### T-J157-008 — Cedar deny: expired cert variant

**Pre-conditions:** Variant fixture: Diana's FOGRA-PSO-L2 cert expired 2 days prior.

**Action sequence:** Diana attempts line stop.

**Expected events:**

- Cedar evaluates `forbid` (FORBID-1)
- `EVT-J157-CEDAR-DENY-CERT-MISSING-012a` sealed
- HTTP 403 with body `{"error":"missing_certification","required":"FOGRA-PSO-Operator-Level-2","status":"expired"}`
- Diana is shown a fallback: "request emergency authorization from manager" — which is an explicit escalation, NOT a silent allow

**Pass criteria:**

- Line stop refused
- Press continues; this is unsafe but Cedar is honest: the operator is uncertified
- The press's OWN safety interlocks (independent of oyatie) remain active — these are not bypassed
- Manager-escalation path is explicit, audited, and Mihai must sign with passkey

**Fail criteria:** any allow path on expired cert; silent fallback; manager-escalation not audited.

### T-J157-009 — Cedar deny: off-shift variant

**Pre-conditions:** Variant fixture: Diana attempts line stop at 22:00 EET (night shift, she is NOT scheduled).

**Action sequence:** Same as T-J157-002 but at the off-shift time.

**Expected events:**

- Cedar evaluates `forbid` (FORBID-2)
- `EVT-J157-CEDAR-DENY-OFF-SHIFT-012b` sealed

**Pass criteria:**

- 403 with reason "off shift"
- Diana sees a clarification: "you are not the operator on this shift; alert Vladimir Csikós"

**Fail criteria:** allow despite off-shift.

### T-J157-010 — Cross-tenant audit dual-seal invariant fuzz

**Pre-conditions:** All prior tests pass.

**Action sequence:** 500 generated cross-tenant operations from the closed set `{customer.notify, customer.confirm, recall.read, capa.read, regulator.preview_only}` × {Tipografia source, Antibiotice target}.

**Expected behavior:** Every permitted op dual-seals; every denied op dual-seals deny.

**Pass criteria:**

- 0 single-seal events
- 0 silent passes
- p99 dual-seal ≤ 280 ms
- Merkle chain validates across all 500 events

**Fail criteria:** any single-seal; any silent pass; merkle break.

### T-J157-011 — CAPA filing fence (TrueTime-class)

**Pre-conditions:** Diana drafted CAPA at 14:32–15:42 EET.

**Action sequence:**

1. Diana endorses §1; Mihai endorses §1
2. Diana endorses §2; Mihai endorses §2
3. Diana endorses §3; Mihai endorses §3
4. Diana clicks "FILE CAPA TO QMS"

**Expected events:**

- `EVT-J157-CAPA-FILED-008` dual-sealed under TrueTime-class fence
- Both endorsements present for each section before file accepted

**Pass criteria:**

- TrueTime uncertainty window ≤ 10 ms
- CAPA cannot file with any section's endorsement missing
- Persisted PDF carries both signatures (Diana + Mihai) with passkey assertions
- Diacritics preserved in PDF + JSON

**Fail criteria:** file accepted with missing endorsement; TrueTime fence violation; PDF signature missing.

### T-J157-012 — Sample shipment with tamper-evident chain

**Pre-conditions:** Customer requested 50-sample inspection.

**Action sequence:**

1. Andrei prepares 50 samples with stamps
2. Tamper seal `ts-2027-02-23-tip-anti-001` applied
3. Cargus pickup at 16:18; tracking `cargus-2027-02-23-CC-7741293`
4. Delivery to Antibiotice Iași QA at 09:30 next day
5. Antibiotice records receipt + validates tamper seal

**Expected events:**

- `EVT-J157-SAMPLES-SHIPPED-007` (Tipografia)
- `EVT-J157-SAMPLES-RECEIVED-007a` (Antibiotice, dual-sealed)
- Tamper seal validation `EVT-J157-TAMPER-SEAL-VALIDATED-007b`

**Pass criteria:**

- Tracking number resolves via Cargus mock
- Tamper seal ID persisted in both tenants
- Receipt within 24 h
- Seal validation pass

**Fail criteria:** tracking unresolvable; seal mismatch; receipt >36 h.

### T-J157-013 — Production re-plan cascade

**Pre-conditions:** T-J157-002–005 passed; Heidelberg #2 available.

**Action sequence:**

1. Production-planning µservice consumes the recall + plant-maintenance work order
2. Schedules re-run on Heidelberg #2 at 22:00 EET
3. Verifies downstream batches do not cascade-delay

**Expected events:**

- `EVT-J157-PRODUCTION-REPLAN-010` sealed
- Downstream batches retain their original slots

**Pass criteria:**

- Replan slot exact: 22:00 EET → 03:30 next day on Heidelberg #2
- 23,653 remaining sheets allocated
- 0 minute cascade delay on downstream
- Original press marked `down_for_dampener_replacement` with WO link

**Fail criteria:** downstream delay >0; replan slot drift >15 min; WO link broken.

### T-J157-014 — Diacritic-aware search vs diacritic-insensitive

**Pre-conditions:** Persona seeds include "Lazăr" (Diana), "Lazar Marian" (a different person, no diacritic, no relation), "Lázár Anna" (Hungarian operator at sister plant).

**Action sequence:**

1. Default search "Lazar" — expect 1 hit (Lazar Marian)
2. Diacritic-aware search "Lazăr" — expect 1 hit (Diana Lazăr)
3. Diacritic-aware search "Lázár" — expect 1 hit (Lázár Anna)
4. Diacritic-insensitive search "lazar" with flag `diacritic_insensitive=true` — expect 3 hits
5. Legal-document search (always diacritic-strict) — never insensitive

**Pass criteria:**

- All four searches return exact expected hits
- Cedar policy enforces that any payments/contracts/regulatory search uses strict mode (not insensitive)

**Fail criteria:** wrong hit counts; strict mode bypassed in legal context.

### T-J157-015 — Shift handoff invariant

**Pre-conditions:** Diana at end of shift.

**Action sequence:**

1. Vladimir arrives 20:00
2. Diana initiates handoff at 20:14
3. Vladimir confirms via passkey + face_id at 20:17:18

**Expected events:**

- `EVT-J157-SHIFT-HANDOFF-011` sealed
- Active recall ownership transferred
- All in-flight tasks visible to Vladimir

**Pass criteria:**

- Handoff completes in ≤ 5 min interaction
- Vladimir's view shows all open tasks + recall state
- Diana's mobile session can still read (audit) but cannot edit after handoff
- Diacritic preservation: "Csikós" + "Vladimír" intact

**Fail criteria:** Diana retains edit rights post-handoff; Vladimir's view missing context; diacritic loss.

## Performance gates

| Operation | p50 | p95 | p99 |
|---|---|---|---|
| Quality alert push | 240 ms | 1.0 s | 1.8 s |
| Line stop Cedar eval | 30 ms | 90 ms | 180 ms |
| Press halt complete | 8 s | 14 s | 18 s |
| Recall state transition | 140 ms | 380 ms | 620 ms |
| Task complete | 180 ms | 480 ms | 820 ms |
| Customer notify dual-seal | 120 ms | 280 ms | 480 ms |
| CAPA file TrueTime fence | 1.4 s | 2.8 s | 4.2 s |
| Diacritic-aware search | 40 ms | 110 ms | 220 ms |

## Cross-tenant invariant tests

| Invariant | Probe | Pass condition |
|---|---|---|
| Antibiotice pre-publish CAPA read | `antibiotice → capa.read(draft)` | 403 + dual-seal |
| Tipografia reads Antibiotice MSA | `tipografia → contract.read(antibiotice MSA)` | permit (shared MSA) |
| Antibiotice triggers ANMDMR unilaterally | `antibiotice → anmdmr.escalate(recall)` | refused unless customer-side authority confirmed |
| Diacritic in legal document | search "Lazar" in legal mode | 0 hits |
| Shift handoff race | parallel claim by 2 incoming operators | only 1 succeeds; other refused |

## Chaos scenarios

1. **Telemetry stream lost mid-batch** — Quality-mgmt switches to operator-visual fallback; large banner shown; line continues at operator discretion until next inspection
2. **Cedar service degraded** — Line stop endpoint preserves availability (safer-default); recall workflow advancement pauses
3. **Cargus courier API unreachable** — Sample-ship task queues; alternate-courier path (Dpd Romania) tried
4. **MLS DS partition Tipografia ↔ Antibiotice for 8 min** — Notifications queue locally; deliver on recovery; epoch correctness preserved
5. **Press fails to halt within 14 s window** — Physical E-stop logged; manager paged; audit `EVT-J157-PRESS-EMERGENCY-ESTOP-CHAOS-5`

## Sign-off checklist

- [ ] All 15 tests pass
- [ ] All invariant probes return expected dual-seal
- [ ] Performance gates met
- [ ] Chaos scenarios complete without data loss
- [ ] All 5 µservices in `/microservices/` resolve: quality-management, tasks, workflow-engine, audit-chain, messenger
- [ ] All 8 ADRs cited resolve
- [ ] Diacritic preservation invariant: 0 normalization events
- [ ] ANMDMR template prepared but NOT sent in baseline test
- [ ] CAPA reconstructible from audit chain 7 years forward
- [ ] DPO sign-off both tenants

## Stop condition

Plan complete when all 15 tests pass, the diacritic invariant holds across all persisted fields, the operator-line-stop-authority Cedar permit functions without manager-gate, the customer notification dual-seals within MSA SLA, the CAPA fence (TrueTime-class) commits atomically, and the ISO-9001 audit chain reconstructs from merkle root.
