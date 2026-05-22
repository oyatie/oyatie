---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j173-aamir-khan-wealth-manager-multi-jurisdictional-trust-restructure
date: 2026-05-20
authority_tier: 2
status: draft
---

# j173 — Integration test plan

Intern-buildable plan: stand up the 4-firm + 4-bank + 7-family-principal fixture; mock 6 CLM documents with deterministic redline injection; mock HMRC CGT clearance; mock CRS + FATCA reissuance; mock SWIFT MT103 dispatch + correspondent bank routing; mock 6-list sanctions screening + 5-jurisdiction AML; mock 4-jurisdiction WORM cells; seed 8 pack overlays; seed Cedar bundle.

## Test environment

| Component | Source |
|---|---|
| Seed primary tenant | `tests/fixtures/tenants/halberd-mercer-private-wealth-difc.yaml` |
| Seed counsel tenants | `tests/fixtures/tenants/{mishcon-de-reya-london,maples-group-grand-cayman,allen-gledhill-singapore,bin-suwaidan-co-difc}.yaml` |
| Seed bank tenants | `tests/fixtures/tenants/{coutts-uk,dbs-singapore,butterfield-cayman,mashreq-bank-uae,hsbc-correspondent,jpmorgan-correspondent,bny-mellon-correspondent}.yaml` |
| Seed HMRC tenant | `tests/fixtures/tenants/hmrc-customer-compliance.yaml` |
| Seed family principal tenants | 7 personal-tenant YAML files |
| Seed personas | `tests/fixtures/personas/{aamir-khan,william-pemberton-brodsky,eleanor-goldsworthy-reid,conrad-hartman-whyte,kerry-anne-osullivan,mei-ling-tan-whitford,joon-ho-park-lim,saira-al-maktoum-hartington,khalid-al-maktoum-hartington,aisha-al-maktoum-hartington,yusuf-al-maktoum-hartington,jonathan-hartington-pemberton,nathaniel-tan-lim,daniel-carmichael-holt}.yaml` |
| Seed CLM documents | 6 doc fixtures with deterministic redline state |
| Seed packs | 8 pack overlays |
| Seed Cedar bundle | `tests/fixtures/cedar/j173/cedar-bundle-amht-2027.cedar` |
| Wire mock — HMRC | deterministic clearance grant 6 business days |
| Wire mock — CRS + FATCA | per-entity classification + GIIN issuance harness |
| Wire mock — SWIFT MT103 | 3-leg dispatch + arrival timing harness |
| Wire mock — sanctions screening | 6-list scan with deterministic clean results |
| Wire mock — AML | 5-jurisdiction screening |
| Wire mock — jurisdiction-aware WORM | 4 cells with retention enforcement |
| Frozen clock | `freeze_clock(2027-05-10T06:42:14+04:00)` step to 2027-05-21T18:18:00+04:00 |

## Seed data summary

| Datum | Value |
|---|---|
| Engagement ID | `client-family-amht-restructure-2027` |
| AUM | $340.4M |
| Documents | 6 |
| Counsel firms | 3 + Halberd-Mercer + Bin Suwaidan |
| Family signing principals | 4 (Saira + Khalid + Aisha + Yusuf) |
| Family non-signing beneficiaries | 3 (Jonathan + minor grandchildren + Nathaniel) |
| Consolidation amount | $42,000,000 |
| Consolidation legs | 3 (UK + SG + KY → DIFC) |
| Sanctions lists | 6 |
| AML jurisdictions | 5 |
| WORM cells | 4 (UK 12y + UAE 8y + SG 7y + KY 6y) |
| Pack overlays | 8 |
| DTAA paths | 3 (UK-UAE + UK-SG + UK-KY) |
| Merkle anchors | 6 |

## Test catalog

### T-J173-001 — Cockpit + pack manifest activation

**Action:** Aamir opens cockpit + activates 8 packs.

**Expected:** `EVT-J173-COCKPIT-OPENED-Δ000` + `EVT-J173-PACK-MANIFEST-008` (preliminary).

**Pass criteria:** DFSA + STEP TEP attestation validated; 8 packs cross-validated.

**Fail criteria:** Missing attestation; pack cross-validation fail.

### T-J173-002 — CLM workflow open with 6 documents

**Action:** Aamir opens CLM workflow assigning 6 docs to 3 firms.

**Expected:** `EVT-J173-CLM-WORKFLOW-OPENED-001`.

**Pass criteria:** Each doc has owner firm + cross-review firms; bar attestation per counsel validated.

**Fail criteria:** Unauthorized counsel principal accepts redline.

### T-J173-003 — Counsel cross-review with cross-firm redlines

**Action:** 24 redlines circulated across 3 firms; 4-round cross-review.

**Expected:** `EVT-J173-COUNSEL-CROSS-REVIEW-002` composite.

**Pass criteria:** All redlines tagged by perspective jurisdiction; STEP-privileged class preserved.

**Fail criteria:** Redline from non-bar-attested principal accepted.

### T-J173-004 — STEP-privileged channel enumeration deny

**Action:** Non-tetrad principals (3 attempts) try to enumerate STEP-privileged channel.

**Expected:** 3 Cedar denies.

**Pass criteria:** All 3 denied; channel not enumerable.

**Fail criteria:** Any non-tetrad principal sees channel metadata.

### T-J173-005 — Tax-scenario ML compute

**Action:** Aamir invokes intelligence µservice tax-scenario.

**Expected:** `EVT-J173-TAX-SCENARIO-COMPUTED-Δ001b`.

**Pass criteria:** UK CGT after holdover £2.4M computed; 3 DTAA paths evaluated; ML provenance recorded.

**Fail criteria:** Tax scenario math wrong; ML provenance missing.

### T-J173-006 — HMRC CGT clearance application + grant

**Action:** Mishcon files CGT clearance; HMRC grants in 6 business days.

**Expected:** `EVT-J173-HMRC-CGT-CLEARANCE-APPLIED-Δ006a` + `EVT-J173-CGT-CLEARANCE-006`.

**Pass criteria:** Expedited request granted; conditions recorded; £2.4M provisional CGT scheduled.

**Fail criteria:** Late grant; conditions missing.

### T-J173-007 — CRS reissuance for 2 new entities

**Action:** Compliance reissues CRS for 2 new entities.

**Expected:** `EVT-J173-CRS-REISSUED-Δ004a`.

**Pass criteria:** 2 entities classified as Investment-Entity-managed-by-FI; reportable jurisdictions correct.

**Fail criteria:** Misclassification; GIIN not pending.

### T-J173-008 — FATCA Form W-8BEN-E reissuance

**Action:** Compliance reissues FATCA form for 2 new entities.

**Expected:** `EVT-J173-FATCA-FORM-REISSUED-Δ004b`.

**Pass criteria:** Annex II classification correct; no US-person beneficial owner test passes.

**Fail criteria:** US-person test fail; Annex II classification wrong.

### T-J173-009 — Family principal sign 4 of 4 with KYC + MiFID II suitability

**Action:** 4 family principals sign 6 docs each over 4 days; KYC + MiFID II suitability for UK principals.

**Expected:** 4 `EVT-J173-FAMILY-SIGN-003-{principal}` + composite `EVT-J173-FAMILY-SIGN-003`.

**Pass criteria:** Each signature has passkey + KYC attestation + MiFID II suitability (UK) + HLC timestamp.

**Fail criteria:** Sign without KYC; MiFID II suitability missing for UK principal.

### T-J173-010 — Sanctions screening across 6 lists clean

**Action:** Payments screens 12 principals across 6 lists.

**Expected:** `EVT-J173-SANCTIONS-SCREEN-CLEAN-Δ005a`.

**Pass criteria:** 0 hits + 0 fuzzy matches + no manual review required.

**Fail criteria:** False-positive not handled; missed real-positive.

### T-J173-011 — AML screening across 5 jurisdictions pass

**Action:** Compliance screens AML across 5 frameworks.

**Expected:** `EVT-J173-AML-SCREEN-CLEAN-Δ005b`.

**Pass criteria:** Source of funds verified; beneficial ownership clear; PEP risk-assessed.

**Fail criteria:** PEP not flagged; source of funds verification incomplete.

### T-J173-012 — SWIFT MT103 dispatch 3 legs

**Action:** Payments dispatches 3 SWIFT MT103 messages.

**Expected:** 3 `EVT-J173-MT103-LEG{n}` events + composite `EVT-J173-CONSOLIDATION-TRANSFER-005`.

**Pass criteria:** Each MT103 has sender_bic + receiver_bic + Field-20 + Field-32A + cover MT202.

**Fail criteria:** Missing fields; bad BIC.

### T-J173-013 — Consolidation reconciliation with FX-favourable variance

**Action:** Arrivals reconcile to $42,001,184 (FX favourable +$1,184).

**Expected:** `EVT-J173-CONSOLIDATION-RECONCILED-Δ005f`.

**Pass criteria:** Discrepancy explained; reconciliation state ok.

**Fail criteria:** Unexplained discrepancy; reconciliation fails.

### T-J173-014 — Merkle anchor per document with tax-authority compulsion path

**Action:** 6 documents anchored with applicable tax authority compulsion paths.

**Expected:** 6 `EVT-J173-MERKLE-DOC-{n}-Δ009{a..f}` events + composite `EVT-J173-MERKLE-PER-DOCUMENT-009`.

**Pass criteria:** Each anchor has merkle_root + applicable compulsion paths + proof_class = inclusion_proof_only_without_payload.

**Fail criteria:** Payload disclosure in proof; compulsion path missing.

### T-J173-015 — Jurisdiction-aware WORM with per-jurisdiction retention

**Action:** 6 documents WORM-sealed in 4 jurisdiction cells.

**Expected:** `EVT-J173-WORM-JURISDICTION-AWARE-010`.

**Pass criteria:** UK 12y + UAE 8y + SG 7y + KY 6y; indelible storage; seal class per jurisdiction.

**Fail criteria:** Wrong retention; wrong cell.

### T-J173-016 — DTAA optimization attestation

**Action:** Compliance attests 3 DTAA paths.

**Expected:** `EVT-J173-DTAA-OPTIMIZATION-007`.

**Pass criteria:** 3 treaties documented + evidence anchors referenced.

**Fail criteria:** Treaty article wrong; evidence anchor missing.

### T-J173-017 — Settlement complete state transition

**Action:** Aamir transitions to settlement_complete.

**Expected:** `EVT-J173-SETTLEMENT-COMPLETE-Δ010`.

**Pass criteria:** All preconditions met (6 signed + Merkle + WORM + sanctions + AML + DTAA).

**Fail criteria:** Transition without preconditions.

### T-J173-018 — Cedar deny coverage (18 denied)

**Action:** Aggregate deny report.

**Expected:** `EVT-J173-CEDAR-DENY-COVERAGE-011`.

**Pass criteria:** 12 + 4 + 2 = 18 denials; redacted observability.

**Fail criteria:** Counter off; metadata leak.

### T-J173-019 — Tax-authority compulsion synthetic test

**Action:** Synthetic HMRC compulsion order for doc-1; system returns inclusion proof only.

**Expected:** `EVT-J173-TAX-AUTHORITY-COMPULSION-PROOF-SYNTHETIC`.

**Pass criteria:** Proof only; payload not disclosed.

**Fail criteria:** Payload disclosed.

### T-J173-020 — Multi-language preservation (UTF-8 NFC byte-exact)

**Action:** All Arabic + Urdu + Cantonese + Cambridge-English + Cayman-English + Singapore-English texts round-trip.

**Expected:** All audit events include lang fields.

**Pass criteria:** sha256 byte-exact equality across all texts.

**Fail criteria:** Any text mutates.

## Cross-test invariants

1. **STEP-privileged channel invariant**: only the 4-tetrad members can enumerate.
2. **Cross-jurisdiction Cedar validation**: every cross-firm call validates bar attestation.
3. **Sanctions+AML pre-payment invariant**: every payment leg requires clean sanctions + AML.
4. **KYC pre-signing invariant**: every family signature requires KYC attestation.
5. **MiFID II suitability for UK invariant**: every UK-domicile family signature includes suitability acknowledgment.
6. **WORM jurisdiction invariant**: each document sealed in its applicable jurisdiction WORM cell with correct retention.
7. **Merkle proof invariant**: every proof emission is inclusion-only; payload requires court order.
8. **Pack manifest cross-validation invariant**: 8 packs cross-validated before any tax-position document executes.
9. **HLC timestamp invariant**: every signature has HLC tag.
10. **Language preservation invariant**: byte-exact UTF-8 NFC across all CJK + Arabic + Latin.

## CI integration

- Lane: `lean-a7-multi-jurisdictional-wealth-trust-restructure`
- Owner: `oya-governance-wealth-management`
- Gate: BLOCKER day 1 (cross-jurisdiction tax + sanctions class)
- Cadence: every PR touching clm + payments + compliance + audit-chain + drive + intelligence
- Coverage: 20 tests pass with 0 failures.

## Failure handling

| Failure | Surface to | Action |
|---|---|---|
| Cross-firm Cedar deny gap | Aamir + counsel + STEP discipline | Cedar bundle review |
| Sanctions screening false-negative | Payments + AML team | Sanctions list refresh + regression test |
| WORM cell jurisdiction misplacement | Drive + GC | ADR-0244 incident |
| HMRC clearance reject | Aamir + Mishcon + tax counsel | Clearance application review |
| DTAA misapplication | Aamir + compliance + tax counsel | Treaty article re-check |

## Exit criteria

20/20 tests pass; cross-test invariants hold; CI lane green; PR carries the `lean-a7-multi-jurisdictional-wealth-trust-restructure` label; sign-off from STEP TEP + 2 of 3 counsel partners + GC + DFSA compliance officer.
