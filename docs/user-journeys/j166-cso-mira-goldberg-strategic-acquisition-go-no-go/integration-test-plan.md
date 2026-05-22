---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j166-cso-mira-goldberg-strategic-acquisition-go-no-go
date: 2026-05-20
authority_tier: 2
status: draft
---

# j166 — Integration test plan

Intern-buildable plan: stand up the seeded `skylark-logistics-solutions-inc` + `mendelsohn-routing-technologies-inc-de` two-tenant fixture; mock the cross-tenant NDA channel; mock 28 MRT diligence documents (anonymized + named); mock the financial-planning M&A model; mock the intelligence µservice ML pipelines (Monte-Carlo + cohort churn + integration cost); mock 5 jurisdictional merger filing evaluators; seed 9 board members + GC + CFO + 2 committee chairs.

## Test environment

| Component | Source |
|---|---|
| Seed acquirer tenant | `tests/fixtures/tenants/skylark-logistics-solutions-inc.yaml` |
| Seed target tenant | `tests/fixtures/tenants/mendelsohn-routing-technologies-inc-de.yaml` |
| Seed personas | `tests/fixtures/personas/{mira-goldberg,bjorn-mendelsohn,adrian-cheng-whitford,reginald-otis,daphne-harrowgate,hannah-beauregard,margarita-velasco-heim,kenji-park-holloway,christine-adebayo-lin,anil-subramaniam,david-hofmann-reyes,joon-ho-park,patricia-wells-okonkwo}.yaml` |
| Seed NDA | `tests/fixtures/nda/skylark-mrt-2027-03-08.yaml` |
| Seed channel | `tests/fixtures/connect/cross-tenant-channel-skylark-mrt-2027-q2.yaml` |
| Seed diligence docs | 28 simulated MRT documents (CSV + PDF + JSON) |
| Seed packs | 6 Skylark packs + 5 MRT packs |
| Seed Cedar bundle | `tests/fixtures/cedar/j166/cedar-bundle-acquisition-v1.cedar` |
| Wire mock — ML | deterministic seed Monte-Carlo + cohort + integration models |
| Wire mock — merger filing evaluators | per-jurisdiction threshold evaluators |
| Frozen clock | `freeze_clock(2027-05-15T07:42:14-04:00)` |

## Seed data summary

| Datum | Value |
|---|---|
| Acquirer tenant | `skylark-logistics-solutions-inc` |
| Target tenant | `mendelsohn-routing-technologies-inc-de` |
| Deal ID | `mrt-acquisition-2027-q2` |
| Working price | $186M |
| Price range | $172M–$202M |
| Earnout | $30M (Bjorn vest 24mo) |
| Board target date | `2027-05-25T09:00:00-04:00` |
| Active packs (acquirer) | 6 |
| Active packs (target) | 5 |
| Required filings | HSR US + German BWB |
| Voluntary filings | UK CMA |
| Diligence docs over journey | 28 |
| Board size | 9 |
| Board majority threshold | 5 of 9 |

## Test catalog

### T-J166-001 — Workspace open with Cedar passkey

**Action:** Mira opens the M&A workspace.

**Expected events:** `EVT-J166-WORKSPACE-OPEN-000` sealed.

**Pass criteria:** Cedar permit (cso + passkey + nda_active); state set to `due_diligence`.

**Fail criteria:** Cedar deny; missing passkey.

### T-J166-002 — Cross-tenant NDA channel document arrival with envelope validation

**Action:** MRT sends `mrt-q1-2027-cohort-churn-anonymized.csv` via cross-tenant channel.

**Expected events:**

- `EVT-J166-DOC-ARRIVED-mrt-cohort-churn-Δ001` sealed
- `EVT-J166-NDA-PAYLOAD-VALIDATED-Δ001a` sealed
- Cedar permit (nda_active + payload_class in whitelist + sender authorized)

**Pass criteria:**

- Payload class `diligence_response_anonymized`
- PII scan 0 hits
- Financial value scan 0 hits
- e2ee envelope intact
- NDA scope authorized
- Sender on MRT's authorized signer list
- Archive flag set to cross_tenant_evidence with return-or-destroy deadline

**Fail criteria:** PII detected; envelope broken; sender unauthorized; missing flag.

### T-J166-003 — 28-document diligence aggregate

**Action:** Run through 28-document arrival sequence over 9 days.

**Expected events:** `EVT-J166-DILIGENCE-DOCS-EXCHANGED-002` rolling summary sealed.

**Pass criteria:** all 28 documents validated; per-doc Cedar evaluation succeeds; rolling sum matches.

**Fail criteria:** any document fails; envelope broken; tally mismatch.

### T-J166-004 — Financial model 3-price-point compute

**Action:** Invoke financial-planning M&A model at $172M / $186M / $202M.

**Expected events:** `EVT-J166-M-A-MODEL-COMPUTED-003` sealed.

**Pass criteria:**

- 3 scenarios returned
- At $186M: rev multiple 4.4x, NTM accretive 23 mo, Y3 IRR 18%, Y5 IRR 22%
- Computation deterministic across runs
- 18% IRR correctly flagged as below 20% threshold

**Fail criteria:** wrong scenario count; non-deterministic; missing threshold flag.

### T-J166-005 — ML Monte-Carlo + cohort churn + integration cost

**Action:** Invoke 3 ML models on MRT inputs.

**Expected events:** `EVT-J166-ML-SCENARIOS-004` sealed.

**Pass criteria:**

- Monte-Carlo 10K iterations × 3 macro scenarios
- Probability-weighted IRR @ Y5: 23%
- Cohort churn 5-year forecast computed
- Integration cost point $14.2M with 95% CI [$9.4M, $19.0M]
- All 3 models declare provenance + EU AI Act Article 50 declaration present
- Reproducibility: re-run with same seed produces byte-equal result

**Fail criteria:** wrong IRR; non-deterministic; missing provenance.

### T-J166-006 — Pack-manifest cross-check + blocker detection

**Action:** Run pack cross-check between Skylark and MRT manifests.

**Expected events:** `EVT-J166-PACK-CROSS-CHECK-005` sealed.

**Pass criteria:**

- Overlap analysis: 3 common, 3 acquirer-only, 2 target-only
- Blocker identified: SOC 2 Type 1→2 remediation ($480K)
- Strategic positive identified: TISAX/VDA opens automotive
- Compatibility score 84%
- German BDSG correctly identified as subsumed by GDPR

**Fail criteria:** missed blocker; wrong overlap; wrong score.

### T-J166-007 — Merger filing requirements compute

**Action:** Run per-jurisdiction filing threshold evaluators.

**Expected events:** `EVT-J166-MERGER-FILINGS-006` sealed.

**Pass criteria:**

- HSR (US): required + $280K fee + 30-day waiting
- EU MR: not required (parties below combined threshold)
- German BWB: required + 1-month window (national alternative to EU)
- UK CMA: voluntary recommended (MRT UK turnover below £70M)
- Israeli IMC: not required (below thresholds)
- Total clearance estimate 30–45 days

**Fail criteria:** missed required filing; wrong threshold; wrong window.

### T-J166-008 — Counsel review with 4 redlines + deal-term clarification

**Action:** Daphne submits counsel review.

**Expected events:** `EVT-J166-COUNSEL-REVIEW-007` sealed.

**Pass criteria:**

- 4 redlines documented (R1, R2, R3, R4)
- Deal-term clarification documented (Bjorn relocation)
- Counsel passkey asserted
- Review duration recorded (524 min)

**Fail criteria:** missing redlines; missing passkey; wrong count.

### T-J166-009 — CFO sign-off on financial model

**Action:** Reginald submits CFO sign-off.

**Expected events:** `EVT-J166-CFO-SIGNOFF-008` sealed.

**Pass criteria:**

- CFO passkey asserted
- Financial model review complete
- ML scenarios acknowledged
- Concerns documented (3)

**Fail criteria:** missing passkey; missing concerns.

### T-J166-010 — Committee endorsement (strategy + audit)

**Action:** Submit endorsements from strategy committee (4/5) + audit committee (3/5).

**Expected events:** `EVT-J166-COMMITTEE-ENDORSEMENT-009` sealed.

**Pass criteria:**

- Strategy: 4 endorse + 1 reservation; endorsed=true
- Audit: 3 endorse; endorsed=true
- Reservations documented (Kenji Park-Holloway integration cost)

**Fail criteria:** below thresholds; missing reservation.

### T-J166-011 — Board go/no-go vote

**Action:** Board votes May 25 09:00 EDT.

**Expected events:** `EVT-J166-BOARD-VOTE-010` sealed.

**Pass criteria:**

- 9 votes recorded: 7 yes + 1 no + 1 abstain
- Each vote passkey-asserted
- Result: GO (majority threshold 5 reached)
- Cedar permit succeeds (committee + counsel + CFO + passkey)
- Joon-Ho Park's hangul name `박준호` byte-exact

**Fail criteria:** missing passkey; wrong tally; Cedar deny on guard.

### T-J166-012 — Decision record + super-Merkle anchor

**Action:** Record decision; compute Merkle; anchor externally.

**Expected events:** `EVT-J166-DECISION-RECORDED-011` sealed.

**Pass criteria:**

- 10 bundle components hashed (executive_summary through integration_playbook)
- Super-Merkle root deterministic
- Drive WORM 7-year retention engaged
- External transparency log anchor present
- NDA-bound diligence documents NOT in bundle (return-or-destroy preserved)

**Fail criteria:** non-deterministic Merkle; WORM missing; NDA-bound docs accidentally bundled.

### T-J166-013 — Forbid: diligence response with PII

**Action:** Mock document with PII (email + name fields).

**Expected events:** Cedar deny + `EVT-J166-CEDAR-DENY-NDA-PAYLOAD-PII-Δ010` sealed.

**Pass criteria:** payload rejected; not archived.

**Fail criteria:** PII payload accepted.

### T-J166-014 — Forbid: non-NDA-bound cross-tenant flow

**Action:** Attempt cross-tenant flow on a channel without active NDA.

**Expected events:** Cedar deny + `EVT-J166-CEDAR-DENY-NON-NDA-FLOW-Δ011` sealed.

**Pass criteria:** flow blocked.

**Fail criteria:** flow proceeds without NDA.

### T-J166-015 — Forbid: board vote without CFO sign-off

**Action:** Attempt board vote with CFO sign-off missing.

**Expected events:** Cedar deny + `EVT-J166-CEDAR-DENY-VOTE-NO-CFO-Δ012` sealed.

**Pass criteria:** vote blocked.

**Fail criteria:** vote proceeds without CFO.

### T-J166-016 — Forbid: non-board member attempts go/no-go vote

**Action:** Mira (CSO) attempts to cast a vote.

**Expected events:** Cedar deny + `EVT-J166-CEDAR-DENY-NON-BOARD-VOTE-Δ013` sealed.

**Pass criteria:** vote rejected.

**Fail criteria:** non-board vote accepted.

### T-J166-017 — Forbid: NDA channel after NDA expiry

**Action:** Simulate NDA expiry; attempt new document send.

**Expected events:** Cedar deny + `EVT-J166-CEDAR-DENY-NDA-EXPIRED-Δ014` sealed.

**Pass criteria:** send blocked.

**Fail criteria:** send succeeds after NDA expiry.

### T-J166-018 — ML reproducibility

**Action:** Run ML scenario battery with the same seed twice.

**Pass criteria:**

- Byte-equal Monte-Carlo results
- Byte-equal cohort churn forecast
- Byte-equal integration cost point estimate

**Fail criteria:** any byte difference under same seed.

### T-J166-019 — Cross-tenant data residency invariant

**Action:** Verify MRT data stays in EU; only document subsets crossable via channel; only hashes in Merkle bundle.

**Pass criteria:**

- MRT primary cell remains `eu-frankfurt-tier-2-tenant-mrt`
- Channel documents Cedar-validated per payload class
- Super-Merkle bundle contains only metadata + hashes; raw MRT data not in bundle

**Fail criteria:** MRT raw data in bundle; channel bypass; residency violated.

### T-J166-020 — Return-or-destroy obligation tracker

**Action:** Verify post-close return-or-destroy deadline correctly computed.

**Pass criteria:**

- Deadline 2027-09-30 if deal terminates
- Deadline overridden if deal closes successfully
- Tracker fires reminder at T-30 days

**Fail criteria:** deadline missing; tracker silent.

### T-J166-021 — End-to-end happy path replay

**Action:** Run full 9-day journey on seeded fixture.

**Pass criteria:** all 12 README AC pass; board votes GO; Merkle externally anchored; super-Merkle root matches expected.

**Fail criteria:** any AC fails.

### T-J166-022 — Character preservation across artifacts

**Action:** Verify Hebrew (מירה גולדברג) + German (Mendelsohn / München) + Hangul (박준호) + diacritic byte-exact preservation.

**Pass criteria:** every name byte-exact.

**Fail criteria:** any normalization.

## Failure scenarios

| Scenario | Expected response |
|---|---|
| Diligence document arrives with PII | Cedar deny; surface to MRT for re-submission |
| Channel envelope fails e2ee check | Surface; require re-send; never store |
| ML scenario reproducibility fails | Block subsequent compute; investigate model determinism |
| Pack cross-check identifies a P1 blocker | Block board package; require remediation plan or deal restructure |
| Counsel declines to sign | Block transition; redraft |
| CFO declines to sign | Block board vote |
| Committee falls below endorsement threshold | Block board vote |
| Board vote falls below majority | Decision = NO-GO; record + archive |
| External transparency log unavailable | Archive locally; flag pending; retry |
| NDA expires mid-journey | Pause channel; require renewal |

## Notes for the test author

- The cross-tenant NDA channel is the highest-risk test surface; fuzz the payload classes + PII scans + sender authorization aggressively.
- ML reproducibility (T-J166-018) is THE correctness assertion — non-determinism breaks the auditability contract for the board.
- The Cedar guard chain on the board vote (committee + counsel + CFO + passkey) is the most consequential gate — test each missing precondition.
- The data residency invariant (T-J166-019) is GDPR + cross-tenant compliance — fuzz the bundle composition.
- The 9 board votes with 9 individual passkey assertions is a distinct stress test — verify each member's passkey + role independently.
