---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j174-sven-eriksson-treasury-eod-position-reconciliation
date: 2026-05-20
authority_tier: 2
status: draft
---

# j174 — Integration test plan

Intern-buildable plan: stand up BHI parent + 7 subsidiary tenants + 8 bank tenants; mock 47 bank accounts × 12 currencies; mock SWIFT MT940/MT942 ingestion with deterministic dispatch; mock FX rate source (Refinitiv + Bloomberg); mock 142 FX forwards + 28 cross-currency swaps; mock 8-entity intercompany matrix; mock cash sweep destinations (3 MMF + 4 banks); mock LCR computation; mock 47 Merkle anchors; seed 8 pack overlays; seed Cedar bundle.

## Test environment

| Component | Source |
|---|---|
| Seed parent tenant | `tests/fixtures/tenants/bohlin-hjelmqvist-industries-ab-parent.yaml` |
| Seed subsidiary tenants | 7 YAML files (DE/US/UK/SG/JP/KR/BR) |
| Seed bank tenants | 8 YAML files (Nordea + Handelsbanken + JPMorgan + HSBC + Mizuho + SMBC + Bradesco + DnB) |
| Seed personas | `tests/fixtures/personas/{sven-eriksson,annika-lindqvist-holmberg,...}.yaml` |
| Seed accounts | 47 account YAML files |
| Seed FX hedge book | 142 forwards + 28 swaps fixtures |
| Seed packs | 8 pack overlays |
| Seed Cedar bundle | `tests/fixtures/cedar/j174/cedar-bundle-treasury-eod.cedar` |
| Wire mock — SWIFT MT940/MT942 | deterministic per-bank dispatch + parse harness |
| Wire mock — Refinitiv + Bloomberg | deterministic FX rate fixture |
| Wire mock — MMF vehicles | deterministic yield |
| Wire mock — overnight deposits | deterministic yield |
| Wire mock — LCR | per-component compute |
| Frozen clock | `freeze_clock(2027-05-20T14:48:00+02:00)` step to 23:18:42 CEST |

## Seed data summary

| Datum | Value |
|---|---|
| EOD session | `eod-bhi-2027-05-20` |
| Accounts | 47 |
| Currencies | 12 |
| Banks | 8 |
| Entities | 8 |
| FX forwards | 142 |
| FX swaps | 28 |
| Hedge book notional | SEK 18.4B |
| Daily turnover | SEK 1.2B target |
| Overnight position | $340M target |
| LCR threshold | 1.0 |
| Pack overlays | 8 |
| MT942 intraday windows | 11 per day (15-min) |
| MT940 expected at EU cut | 16 |
| MT940 expected at London cut | 8 |
| MT940 expected at US cut | 8 |
| MT940 expected at Stockholm night | 9 |

## Test catalog

### T-J174-001 — Cockpit + cut-off bar render

**Action:** Sven opens cockpit.

**Expected:** `EVT-J174-COCKPIT-OPENED-Δ000`.

**Pass criteria:** Cut-off bar shows 6 windows (asia + sg + europe + london + us + stockholm_night); CTP + passkey + yubikey validated.

**Fail criteria:** Cedar deny without attestation.

### T-J174-002 — MT942 intraday ingestion (11 windows × 47 accounts)

**Action:** 11 MT942 windows ingest across 47 accounts = ~517 events per day.

**Expected:** `EVT-J174-INTRADAY-MT942-ROLLUP-Δ001a` rolling.

**Pass criteria:** Each MT942 parsed; per-account latest position updated; sub-2-minute SLA met.

**Fail criteria:** MT942 parse error; SLA breach.

### T-J174-003 — MT940 ingestion across 4 cut windows

**Action:** 16 EU + 8 London + 8 US + 9 Stockholm night MT940s ingested.

**Expected:** `EVT-J174-MT940-INGESTION-001` per cut + daily composite.

**Pass criteria:** Per-statement ingestion P95 < 2 minutes; per-account opening + closing balances correct; MT942-vs-MT940 reconciliation 0.

**Fail criteria:** SLA breach; reconciliation discrepancy.

### T-J174-004 — FX rate sourcing (Refinitiv primary + Bloomberg backup)

**Action:** Treasury µservice pulls FX rates with 0.0001% spread tolerance.

**Expected:** rates loaded from Refinitiv; backup-vs-primary spread check.

**Pass criteria:** spread within tolerance; FX snapshot timestamp recorded.

**Fail criteria:** spread > tolerance; rate source missing.

### T-J174-005 — Cash position computation (post-cut interim + final)

**Action:** Per-entity + per-currency + group EUR computation runs.

**Expected:** `EVT-J174-CASH-POSITION-COMPUTED-002`.

**Pass criteria:** Arithmetic exact; HLC timestamp present; per-entity rollup matches per-account sum.

**Fail criteria:** Arithmetic error; HLC missing.

### T-J174-006 — FX delta-hedging compute (no rebalance today)

**Action:** Delta compute across 5 categories.

**Expected:** `EVT-J174-FX-DELTA-HEDGE-003`.

**Pass criteria:** Max delta 0.42% < 0.50% threshold; rebalance_triggered=false; PnL computed.

**Fail criteria:** Delta computation wrong; trigger logic broken.

### T-J174-007 — FX delta-hedging compute with rebalance trigger (synthetic)

**Action:** Synthetic input pushes one category delta to 0.55%.

**Expected:** `EVT-J174-FX-DELTA-REBALANCE-TRIGGER-SYNTHETIC`.

**Pass criteria:** Rebalance triggered; treasurer escalation event; hedge book delta recomputed.

**Fail criteria:** Trigger missed.

### T-J174-008 — Intercompany netting matrix (8 entities, 28 pairs)

**Action:** Netting compute.

**Expected:** `EVT-J174-INTERCOMPANY-NETTING-004`.

**Pass criteria:** Matrix dim = 8; net-per-entity sums to 0 (closed system); settlement via in-house bank no external SWIFT.

**Fail criteria:** Matrix incomplete; sum != 0.

### T-J174-009 — Cash sweep with co-sign

**Action:** Sven prepares sweep + Annika co-signs + executes.

**Expected:** `EVT-J174-CASH-SWEEP-PREPARED-Δ005-prep` + `EVT-J174-CASH-SWEEP-CO-SIGN-Δ005a` + `EVT-J174-CASH-SWEEP-005`.

**Pass criteria:** Co-sign required for amount; sanctions screen clean; sweep executes only after co-sign; allocation matches plan.

**Fail criteria:** Sweep executes without co-sign; sanctions check skipped.

### T-J174-010 — Cash sweep without co-sign deny

**Action:** Junior analyst attempts sweep without Annika's co-sign.

**Expected:** Cedar deny.

**Pass criteria:** Deny logged; sweep not executed.

**Fail criteria:** Sweep executes.

### T-J174-011 — Basel-III LCR computation

**Action:** finops-portal computes LCR.

**Expected:** `EVT-J174-LCR-COMPUTED-006`; ratio 1.42.

**Pass criteria:** HQLA components correct; outflow projection correct; ratio ≥ 1.0.

**Fail criteria:** Components misclassified; ratio < 1.0 missed escalation.

### T-J174-012 — Per-account Merkle attestation (47 anchors)

**Action:** audit-chain emits 47 per-account anchors.

**Expected:** `EVT-J174-MERKLE-EOD-007`.

**Pass criteria:** 47 anchors; per-anchor includes account_identifier + closing_balance + merkle_root + external_transparency_log_batch; ifrs_9 + ias_21 attestations present.

**Fail criteria:** Anchor count wrong; attestation missing.

### T-J174-013 — Cut-off ordering preserved (asia → sg → europe → london → us → stockholm)

**Action:** audit-chain emits cut-off ordering attestation.

**Expected:** `EVT-J174-CUT-OFF-ORDERING-008`; ordering_preserved=true.

**Pass criteria:** All 6 cuts in monotonically increasing UTC; no out-of-order cut.

**Fail criteria:** Out-of-order cut.

### T-J174-014 — Pack manifest assertion

**Action:** Compliance asserts 8 packs.

**Expected:** `EVT-J174-PACK-MANIFEST-009`.

**Pass criteria:** 8 packs cross-validated; signature recorded.

**Fail criteria:** Pack count != 8.

### T-J174-015 — Observability SLO report

**Action:** Observability emits SLO report.

**Expected:** `EVT-J174-OBSERVABILITY-SLO-010`; ALL_PASS.

**Pass criteria:** MT940 P95 < 2min; cash position P95 < 18s; LCR P95 < 8s.

**Fail criteria:** Any SLO breach.

### T-J174-016 — Cedar deny coverage (10 denials)

**Action:** Adversarial sweep-without-cosign + bank-statement-enumerate attempts.

**Expected:** `EVT-J174-CEDAR-DENY-COVERAGE-011`; 4 + 6 = 10 denials.

**Pass criteria:** All 10 denied + logged + redacted.

**Fail criteria:** Any deny succeeds.

### T-J174-017 — MT942-vs-MT940 reconciliation invariant

**Action:** Verify MT942 intraday positions reconcile with MT940 EOD.

**Expected:** discrepancy_eur = 0.

**Pass criteria:** 0 discrepancy across all 47 accounts.

**Fail criteria:** Any discrepancy.

### T-J174-018 — IFRS-9 + IAS-21 FX translation attestation invariant

**Action:** Audit-chain attests IFRS-9 + IAS-21 FX translation per anchor.

**Expected:** Both attestations present on every anchor.

**Pass criteria:** 47/47 anchors with both attestations.

**Fail criteria:** Any missing.

### T-J174-019 — EOD posted state transition

**Action:** Sven transitions to eod_posted state.

**Expected:** `EVT-J174-EOD-POSTED-Δ010`.

**Pass criteria:** All preconditions met (MT940 + cash position + FX + netting + sweep + LCR + Merkle).

**Fail criteria:** Transition without preconditions.

### T-J174-020 — Multi-language preservation (UTF-8 NFC byte-exact)

**Action:** Swedish + Norwegian + Danish + Finnish + German + English + JP + KR + ZH + PT texts round-trip.

**Expected:** All audit events include lang fields.

**Pass criteria:** byte-exact across all languages.

**Fail criteria:** Any text mutates.

## Cross-test invariants

1. **Cut-off ordering invariant**: cuts always in monotonically increasing UTC.
2. **MT942-vs-MT940 reconciliation invariant**: 0 discrepancy.
3. **Per-account Merkle invariant**: 47 anchors emitted; no composite-only.
4. **Co-sign invariant**: cash sweep above ceiling requires Group Treasurer co-sign.
5. **Sanctions screening invariant**: every sweep counterparty screened.
6. **LCR threshold invariant**: LCR < 1.0 triggers escalation event.
7. **Rebalance trigger invariant**: max delta ≥ 0.5% triggers rebalance.
8. **Pack manifest cross-validation invariant**: 8 packs cross-validated before EOD posted.
9. **HLC timestamp invariant**: every settlement + sweep has HLC tag.
10. **Multi-language preservation invariant**: byte-exact UTF-8 NFC.

## CI integration

- Lane: `lean-a7-treasury-eod-multi-currency`
- Owner: `oya-governance-treasury`
- Gate: BLOCKER day 1 (treasury reconciliation accuracy class)
- Cadence: every PR touching payments + treasury + finops-portal + audit-chain + observability
- Coverage: 20 tests pass with 0 failures.

## Exit criteria

20/20 tests pass; cross-test invariants hold; CI lane green; PR carries `lean-a7-treasury-eod-multi-currency` label; sign-off from CTP-certified + Group Treasurer + audit lead.
