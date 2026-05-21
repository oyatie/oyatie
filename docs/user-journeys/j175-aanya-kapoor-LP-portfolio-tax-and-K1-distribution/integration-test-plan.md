---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j175-aanya-kapoor-LP-portfolio-tax-and-K1-distribution
date: 2026-05-20
authority_tier: 2
status: draft
---

# j175 — Integration test plan

Intern-buildable plan: stand up Aanya's personal tenant + 4 fund GP tenants + IRS tenant + 8 state revenue tenants + Wells Goldman CPA tenant; mock 4 K-1 PDFs with deterministic line items; mock LP capital account reconciliation; mock tax-character categorization; mock state apportionment; mock foreign tax credit; mock AMT + 1411 NIIT + 199A; mock quarterly estimated tax payments; mock WORM archival; mock GP-LP communication channel; seed 10 pack overlays; seed Cedar bundle.

## Test environment

| Component | Source |
|---|---|
| Seed LP personal tenant | `tests/fixtures/tenants/aanya-kapoor-personal-2008.yaml` |
| Seed GP tenants | 4 YAML files (a16z + Sequoia + KKR + Insight) |
| Seed CPA tenant | `tests/fixtures/tenants/wells-goldman-cpa.yaml` |
| Seed IRS tenant | `tests/fixtures/tenants/irs-direct-pay.yaml` |
| Seed state revenue tenants | 8 YAML files (CA + NY + MA + TX + WA + CO + TN + FL) |
| Seed personas | `tests/fixtures/personas/{aanya-kapoor,vikram-kapoor,patricia-wells-goldman,kerry-park-holt,anil-subramaniam-reid,sarah-chen-marlowe-a16z,david-park-sequoia,bjorn-mendelsohn-insight}.yaml` |
| Seed K-1 PDFs | 4 PDF fixtures with deterministic line items |
| Seed capital account statements | 4 YAML files |
| Seed partner allocation schedules | 4 YAML files |
| Seed foreign tax credit footnotes | 4 PDF fixtures |
| Seed packs | 10 pack overlays |
| Seed Cedar bundle | `tests/fixtures/cedar/j175/cedar-bundle-lp-k1.cedar` |
| Wire mock — K-1 parser | deterministic PDF → Schedule K-1 line items |
| Wire mock — Section 199A | QBI + W-2 phaseout |
| Wire mock — Section 1411 NIIT | NIIT base + 3.8% rate |
| Wire mock — state apportionment | 8-state matrix + foreign-source |
| Wire mock — Form 1116 FTC | per-jurisdiction basket allocation |
| Wire mock — AMT | AMTI + exemption phaseout |
| Wire mock — IRS Direct Pay | ACH ack |
| Wire mock — state revenue portals | ACH ack |
| Wire mock — WORM cell | indelible storage + time-stamp |
| Frozen clock | `freeze_clock(2027-05-20T19:48:00-07:00)` step to 21:18 PDT Sunday |

## Seed data summary

| Datum | Value |
|---|---|
| LP session | `lp-reconciliation-aanya-fy2026` |
| Funds | 4 |
| Total LP capital | $12,380,980 |
| Total committed | $14,200,000 |
| K-1 total income | $715,206 |
| Foreign-source income | $38,170 |
| Section 199A QBI | $42,332 |
| Section 199A effective deduction | $0 (W-2 phaseout) |
| Section 1411 NIIT | $27,323 |
| 8 US states | CA + NY + MA + TX + WA + CO + TN + FL |
| Foreign jurisdictions | 4 (SG + IN + ID + HK) for FTC |
| Form 1116 FTC creditable | $4,824 |
| FTC carryforward | $30,144 |
| AMT owed | $0 |
| Q2 IRS payment | $48,228 |
| Q2 state aggregate | $27,298 |
| WORM artifacts | 16 |
| Pack overlays | 10 |

## Test catalog

### T-J175-001 — LP cockpit open with accredited + qualified purchaser attestation

**Action:** Aanya opens cockpit.

**Expected:** `EVT-J175-LP-COCKPIT-OPENED-Δ000`.

**Pass criteria:** Cedar permit granted; AI + QP attestations validated.

**Fail criteria:** Missing attestation; Cedar deny.

### T-J175-002 — K-1 PDF ingestion from 4 fund GPs

**Action:** 4 GPs send K-1 PDFs.

**Expected:** `EVT-J175-K1-INGESTED-001` composite.

**Pass criteria:** 4 K-1 PDFs parsed; Schedule K-1 line items extracted; KKR's Schedule K-3 (international) attached.

**Fail criteria:** Parse error; line items wrong.

### T-J175-003 — LP capital account reconciliation

**Action:** Reconcile 4 funds' capital accounts.

**Expected:** `EVT-J175-CAPITAL-ACCOUNT-RECONCILED-002`.

**Pass criteria:** Per-fund closing balance matches GP statement; aggregate $12.38M.

**Fail criteria:** Reconciliation discrepancy.

### T-J175-004 — Tax-character categorization

**Action:** Categorize K-1 line items.

**Expected:** `EVT-J175-TAX-CHARACTER-003`.

**Pass criteria:** Aggregate $715,206 matches K-1 line items; 7 categories computed (ordinary + LTCG + STCG + qualified div + interest + 199A + foreign).

**Fail criteria:** Aggregate mismatch.

### T-J175-005 — Section 199A QBI compute

**Action:** Compute 199A with W-2 phaseout.

**Expected:** `EVT-J175-SECTION-199A-COMPUTED-004`.

**Pass criteria:** Phaseout state correct; effective deduction $0 due to W-2 limitation.

**Fail criteria:** Deduction granted beyond phaseout.

### T-J175-006 — Section 1411 NIIT compute

**Action:** Compute 1411 NIIT.

**Expected:** `EVT-J175-SECTION-1411-NIIT-005`.

**Pass criteria:** NIIT owed $27,323; rate 3.8%; base correct.

**Fail criteria:** NIIT rate wrong; base wrong.

### T-J175-007 — State-by-state apportionment

**Action:** Compute 8-state apportionment using Schedule K-3.

**Expected:** `EVT-J175-STATE-APPORTIONMENT-006`.

**Pass criteria:** Per-state matrix sums to total income; CA-source matches residence rule; out-of-state credit computed.

**Fail criteria:** Matrix sum mismatch.

### T-J175-008 — Foreign tax credit Form 1116 compute

**Action:** Compute FTC across 6 jurisdictions.

**Expected:** `EVT-J175-FOREIGN-TAX-CREDIT-007`.

**Pass criteria:** Passive basket $4,824; carryforward $30,144; per-jurisdiction breakdown correct.

**Fail criteria:** Basket wrong; carryforward wrong.

### T-J175-009 — AMT compute

**Action:** Compute AMT projection.

**Expected:** `EVT-J175-AMT-COMPUTED-008`.

**Pass criteria:** AMT owed $0 (regular tax exceeds tentative AMT); exemption phaseout correct.

**Fail criteria:** AMT computation wrong.

### T-J175-010 — Q2 quarterly estimated tax payments

**Action:** Aanya pays Q2 IRS + 4 state revenue depts.

**Expected:** `EVT-J175-ESTIMATED-TAX-IRS-Δ009a` + 4 state events + composite.

**Pass criteria:** IRS $48,228 + CA $24,648 + NY $1,242 + MA $1,084 + CO $324 all ACKed.

**Fail criteria:** Any payment fails; ACK missing.

### T-J175-011 — GP-LP communication (2 substantive clarifications)

**Action:** Aanya queries KKR (Indonesia FTC) + Insight (Section 199A).

**Expected:** `EVT-J175-GP-LP-CLARIFICATIONS-010` composite.

**Pass criteria:** MLS E2EE channel; GP responses received within reasonable time; workpapers attached.

**Fail criteria:** Channel not E2EE; workpaper not received.

### T-J175-012 — WORM archival (16 artifacts)

**Action:** Archive 4 K-1 PDFs + 4 capital account statements + 4 partner allocation schedules + 4 FTC footnotes.

**Expected:** `EVT-J175-WORM-ARCHIVED-011`.

**Pass criteria:** 16 artifacts; 7-year retention; indelible storage; time-stamp authority; case Merkle root.

**Fail criteria:** Wrong retention; mutability post-seal.

### T-J175-013 — CPA package delivery (drive shared)

**Action:** Aanya shares CPA package with Patricia.

**Expected:** `EVT-J175-CPA-PACKAGE-SENT-Δ012a`.

**Pass criteria:** CPA tenant receives read-only access; contents manifest complete.

**Fail criteria:** CPA tenant not added; manifest incomplete.

### T-J175-014 — Pack manifest assertion (10 packs)

**Action:** Compliance asserts 10 active packs.

**Expected:** `EVT-J175-PACK-MANIFEST-Δ012b`.

**Pass criteria:** 10 packs cross-validated; signature recorded.

**Fail criteria:** Pack count != 10.

### T-J175-015 — Cross-fund tax-character invariant

**Action:** Verify aggregate per-character matches sum of per-fund line items.

**Expected:** invariant pass.

**Pass criteria:** ordinary + LTCG + STCG + qual div + interest + 199A + foreign sums match.

**Fail criteria:** Any sum mismatch.

### T-J175-016 — Per-state apportionment invariant

**Action:** Verify per-state matrix sums correctly.

**Expected:** invariant pass.

**Pass criteria:** sum across states + foreign = aggregate K-1 income.

**Fail criteria:** Mismatch.

### T-J175-017 — IRS records retention 7y invariant

**Action:** Verify WORM artifacts cannot be mutated post-seal.

**Expected:** any update/delete attempt denied.

**Pass criteria:** Cedar deny on update; deny on delete.

**Fail criteria:** Mutation succeeds.

### T-J175-018 — GP-LP channel enumeration deny

**Action:** Non-permitted principals (3 attempts) try to enumerate GP-LP channel.

**Expected:** 3 Cedar denies.

**Pass criteria:** All denied + audit-logged.

**Fail criteria:** Any non-permitted enumeration.

### T-J175-019 — Multi-language preservation (UTF-8 NFC byte-exact)

**Action:** Hindi + English + Tamil + Mandarin + Japanese + Indonesian round-trip.

**Expected:** all texts byte-exact.

**Pass criteria:** sha256 byte-exact.

**Fail criteria:** Any mutation.

### T-J175-020 — Safe-harbor check for estimated tax

**Action:** Verify Aanya's Q1+Q2+Q3+Q4 estimated tax meets safe harbor (90% current-year or 110% prior-year).

**Expected:** safe harbor met; no underpayment penalty.

**Pass criteria:** safe harbor calculation correct.

**Fail criteria:** Underpayment penalty risk missed.

## Cross-test invariants

1. **Per-fund reconciliation invariant**: closing balance matches GP capital account statement.
2. **Tax-character aggregate invariant**: aggregate matches per-K-1 line items.
3. **State apportionment invariant**: per-state matrix sums to total income.
4. **FTC carryforward invariant**: creditable + carryforward = total foreign tax paid.
5. **199A phaseout invariant**: W-2/UBIA limitation respected for high-income MFJ.
6. **NIIT base invariant**: min(NII, MAGI - threshold) computed correctly.
7. **AMT invariant**: AMT owed = max(0, tentative AMT - regular tax).
8. **WORM seal invariant**: artifacts immutable post-seal.
9. **GP-LP channel invariant**: MLS E2EE; only permitted principals.
10. **Multi-language preservation invariant**: byte-exact UTF-8 NFC.

## CI integration

- Lane: `lean-a7-lp-k1-multi-fund-tax`
- Owner: `oya-governance-tax-lp`
- Gate: BLOCKER day 1 (LP tax reconciliation accuracy class)
- Cadence: every PR touching finops-portal + compliance + payments + drive + connect
- Coverage: 20 tests pass with 0 failures.

## Exit criteria

20/20 tests pass; cross-test invariants hold; CI lane green; PR carries `lean-a7-lp-k1-multi-fund-tax` label; sign-off from CPA (Patricia) + tax counsel.
