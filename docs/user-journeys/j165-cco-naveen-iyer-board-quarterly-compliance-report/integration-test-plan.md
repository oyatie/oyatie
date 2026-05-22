---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j165-cco-naveen-iyer-board-quarterly-compliance-report
date: 2026-05-20
authority_tier: 2
status: draft
---

# j165 — Integration test plan

Intern-buildable plan: stand up the seeded `tessellate-health-ai-inc` tenant fixture with 8 active compliance packs (480-540 evidence artifacts across 3 regions); mock the LLM (Sonnet-Compliance-Tuned-v3); mock the audit-chain Merkle compute deterministically; mock SEC 8-K trigger evaluator; mock external transparency log; seed 5 audit-committee members + GC + board members. Walk every test in order.

## Test environment

| Component | Source |
|---|---|
| Seed tenant | `tests/fixtures/tenants/tessellate-health-ai-inc.yaml` |
| Seed personas | `tests/fixtures/personas/{naveen-iyer,hampton-reese,jasmine-wells-okafor,tunde-akinwale,lisa-cheng-halsey,margaret-donovan-walsh,marcus-lin,patricia-hwong,vinod-thomas-meyer,aisha-kone-stevens}.yaml` |
| Seed packs | 8 packs with seeded evidence ledgers + findings + remediation states |
| Seed cells | 3 evidence cells + 1 WORM cell + 1 external transparency log mock |
| Seed Cedar bundle | `tests/fixtures/cedar/j165/cedar-bundle-board-compliance-report-v1.cedar` |
| Wire mock — LLM | `tests/mocks/sonnet-compliance-tuned-v3.toml` (deterministic with seeded prompt) |
| Wire mock — Merkle | `tests/mocks/audit-chain-merkle-deterministic.toml` |
| Wire mock — SEC trigger | `tests/mocks/sec-8k-trigger-evaluator.toml` |
| Wire mock — external log | `tests/mocks/external-transparency-log-batch-2027-04.toml` |
| Frozen clock | `freeze_clock(2027-04-08T06:18:14-04:00)` |

## Seed data summary

| Datum | Value |
|---|---|
| Tenant ID | `tessellate-health-ai-inc` |
| Report ID | `q1-2027-quarterly` |
| Quarter | `fy2027-q1` |
| Active packs | 8 |
| Total evidence artifacts | 496 (142+87+64+71+42+38+28+24) |
| Total findings | 7 |
| Total open risks | 10 |
| Board scheduled | `2027-04-14T14:00:00-04:00` |
| Pre-read due | `2027-04-13T17:00:00-04:00` |
| AC quorum threshold | 3 of 5 |
| Retention | 7 years (SEC-pre-IPO-adapted) |
| Super-Merkle root (expected) | `0xf3a8c2e7b6d9f4a1c8e3b5d7f0a2c4e6b8d1f5a3c7e9b2d4f6a8c0e2b5d7f1a4` |

## Test catalog

### T-J165-001 — Workflow initiation

**Action:** Naveen initiates the workflow at 06:24:18 EDT Thursday.

**Expected events:** `EVT-J165-WORKFLOW-INITIATED-000` sealed.

**Pass criteria:** Cedar permit succeeds (cco + passkey + tenant); state transitions to `draft`; deadline computed correctly.

**Fail criteria:** Cedar deny; wrong state; missing event.

### T-J165-002 — Cross-pack evidence pull (8 packs in parallel)

**Action:** Fanout query to 8 pack evidence ledgers across 3 regions.

**Expected events:** `EVT-J165-PACK-EVIDENCE-PULL-001` dual-sealed per region.

**Pass criteria:**

- 8 packs returned with correct evidence counts (142+87+64+71+42+38+28+24 = 496)
- Findings counts match: 3+1+2+0+0+1+0+0 = 7
- Open risks match: 2+1+1+4+0+0+0+2 = 10
- Total pull duration ≤ 5min (measured 3m36s)
- Cross-region pulls (EU + KR) complete with their evidence staying local — only summary statistics cross

**Fail criteria:** wrong count; missing pack; evidence data crossed regions; > 5min total.

### T-J165-003 — Per-pack Merkle root deterministic compute (8 packs)

**Action:** Compute Merkle root per pack from each evidence ledger.

**Expected events:** `EVT-J165-PER-PACK-MERKLE-002` × 8.

**Pass criteria:**

- All 8 roots computed within 90s (measured 64s)
- Each root deterministic across two independent runs (byte-equal)
- Roots match expected values from seed fixture
- SOC 2 root: `0x7a2f4b8c1e9d5f3a6b2c8e0f4d7a9b1c`
- HIPAA root: `0x3e8b2f9a6c4d1e7f5a8b3c0d6e9f2a4b`
- (etc — 8 total)

**Fail criteria:** root non-deterministic; wrong root; missing root.

### T-J165-004 — LLM-assisted executive summary draft

**Action:** Invoke intelligence µservice for exec summary draft.

**Expected events:** `EVT-J165-LLM-DRAFT-ASSIST-004` sealed.

**Pass criteria:**

- Model identity declared: `sonnet-compliance-tuned-v3@oyatie-2027-03`
- EU AI Act Article 50 declaration included in provenance metadata
- Draft within 4 pages
- Token counts present (input 14820 + output 1840)
- LLM provenance preserved end-of-line
- Naveen edit distance 38% (validates human-in-the-loop)

**Fail criteria:** missing provenance; missing Article 50 declaration; draft exceeds 4 pages.

### T-J165-005 — SEC Form 8-K trigger evaluation

**Action:** Run 8-K trigger evaluator against Q1 evidence.

**Expected events:** `EVT-J165-SEC-8K-EVAL-005` sealed.

**Pass criteria:**

- 7 items evaluated
- 0 triggers fired (pre-IPO)
- Status `pre_ipo_not_yet_obligated`
- Note recorded about post-S-1 effectiveness
- Form NT also evaluated: 0 triggers

**Fail criteria:** wrong trigger count; missing items; wrong status.

### T-J165-006 — Super-Merkle of Merkles compute

**Action:** Compute super-Merkle root from the 8 per-pack roots.

**Expected events:** `EVT-J165-SUPER-MERKLE-003` sealed.

**Pass criteria:**

- Ordering: pack_id ascending (deterministic)
- Output: `0xf3a8c2e7b6d9f4a1c8e3b5d7f0a2c4e6b8d1f5a3c7e9b2d4f6a8c0e2b5d7f1a4`
- Compute duration ≤ 30s (measured 18s)
- Reproducible across two independent runs

**Fail criteria:** wrong root; non-deterministic; wrong ordering.

### T-J165-007 — Workflow transition Draft → Counsel Review

**Action:** Naveen transitions at 17:42 EDT Friday.

**Expected events:** `EVT-J165-TRANSITION-DRAFT-TO-COUNSEL-006` sealed.

**Pass criteria:**

- Cedar permit succeeds (cco_signoff_present + passkey + super_merkle_root_present + twelve_sections_complete)
- State changes to `counsel_review`
- Hampton Reese notified

**Fail criteria:** Cedar deny; missing guard satisfaction; state stays in draft.

### T-J165-008 — Counsel review with 3 redlines

**Action:** Hampton reviews + produces 3 redlines.

**Expected events:** `EVT-J165-COUNSEL-REVIEW-007` sealed.

**Pass criteria:**

- 3 redlines recorded (sections 4, 7, 10)
- Review duration 248 min (≤ 6 business hours)
- Passkey asserted
- Cedar permit succeeds (general_counsel + cco_signoff_present)

**Fail criteria:** missing redlines; missing passkey; wrong role.

### T-J165-009 — Workflow transition Counsel → Audit Committee

**Action:** Hampton transitions at 16:32 EDT Saturday.

**Expected events:** `EVT-J165-TRANSITION-COUNSEL-TO-AC-008` sealed.

**Pass criteria:**

- Cedar permit succeeds (cco_signoff_present + counsel_review_present + redlines_resolved)

**Fail criteria:** Cedar deny.

### T-J165-010 — Audit committee quorum sign-off

**Action:** Jasmine + Tunde + Lisa sign off.

**Expected events:** `EVT-J165-AUDIT-COMMITTEE-SIGNOFF-009` sealed.

**Pass criteria:**

- 3 sign-offs recorded
- Quorum threshold 3 reached
- Each sign-off has passkey + role validation
- Marcus Lin + Patricia Hwong deferred to pre-read (not blocking)

**Fail criteria:** quorum < 3; missing passkey; wrong role.

### T-J165-011 — Workflow transition Audit Committee → Board

**Action:** Jasmine transitions at 17:42 EDT Sunday.

**Expected events:** `EVT-J165-TRANSITION-AC-TO-BOARD-010` sealed.

**Pass criteria:**

- Cedar permit succeeds (counsel_review_present + audit_committee_quorum_reached + quorum_count >= 3)

**Fail criteria:** Cedar deny; quorum missing.

### T-J165-012 — Drive WORM archive with 7-year retention

**Action:** Archive final 47-page PDF.

**Expected events:** `EVT-J165-REPORT-ARCHIVED-011` sealed.

**Pass criteria:**

- WORM lock engaged
- Retention until 2034-04-11 (7-year)
- SHA-256 of PDF stored
- 5-step approval chain preserved (CCO + counsel + AC chair + 2 AC independents)
- Super-Merkle root metadata preserved

**Fail criteria:** WORM missing; retention < 7 years; approval chain incomplete.

### T-J165-013 — External transparency log anchor

**Action:** Anchor super-Merkle root to external transparency log batch.

**Expected events:** `EVT-J165-EXTERNAL-ANCHOR-013` sealed.

**Pass criteria:**

- Batch ID `external-transparency-log-batch-2027-04-11T1742`
- Anchor latency ≤ 60s (measured 24s)
- Independent observer can verify

**Fail criteria:** missing anchor; latency > 60s.

### T-J165-014 — Regional evidence preservation

**Action:** Verify per-region evidence stayed local.

**Expected events:** `EVT-J165-REGIONAL-EVIDENCE-PRESERVED-012` sealed.

**Pass criteria:**

- us-east 281 artifacts local
- eu-frankfurt 135 artifacts local
- kr-seoul 80 artifacts local
- only_hashes_crossed_regions = true
- data_residency_invariant_held = true

**Fail criteria:** evidence material crossed regions; invariant violated.

### T-J165-015 — Pre-read distribution

**Action:** Naveen distributes to 8 board members at 11:18 EDT Monday.

**Expected events:** `EVT-J165-PRE-READ-DISTRIBUTED-014` sealed.

**Pass criteria:**

- All 8 board members notified
- Pre-read window 2 days (Monday 11:18 → Wednesday 13:00)

**Fail criteria:** missing recipient; wrong window.

### T-J165-016 — Forbid: non-CCO initiates workflow

**Action:** Hampton attempts to initiate workflow.

**Expected events:** Cedar deny + `EVT-J165-CEDAR-DENY-NON-CCO-INIT-Δ001` sealed.

**Pass criteria:** 403; workflow not started.

**Fail criteria:** workflow started by non-CCO.

### T-J165-017 — Forbid: AC sign-off without counsel review

**Action:** Jasmine attempts to sign off before Hampton's counsel review.

**Expected events:** Cedar deny + `EVT-J165-CEDAR-DENY-AC-WITHOUT-COUNSEL-Δ002` sealed.

**Pass criteria:** 403; sign-off rejected.

**Fail criteria:** sign-off recorded out-of-order.

### T-J165-018 — Forbid: board transition without AC quorum

**Action:** Jasmine attempts transition with only 1 AC sign-off (her own).

**Expected events:** Cedar deny + `EVT-J165-CEDAR-DENY-BOARD-NO-QUORUM-Δ003` sealed.

**Pass criteria:** 403; transition rejected; state stays in `audit_committee_sign_off`.

**Fail criteria:** transition succeeds without quorum.

### T-J165-019 — Forbid: cross-region evidence material transfer

**Action:** Attempt to copy raw EU evidence artifact to us-east cell.

**Expected events:** Cedar deny + `EVT-J165-CEDAR-DENY-EVIDENCE-CROSS-REGION-Δ004` sealed.

**Pass criteria:** 403; only hashes crossable.

**Fail criteria:** material crosses; data residency violated.

### T-J165-020 — Merkle root determinism across runs

**Action:** Re-run the same fixture twice; compare super-Merkle root.

**Pass criteria:**

- Two runs produce byte-equal super-Merkle root
- Pack ordering consistent
- All 8 per-pack roots byte-equal across runs

**Fail criteria:** any byte difference.

### T-J165-021 — End-to-end happy path replay

**Action:** Run full 3-day journey on the seeded fixture.

**Pass criteria:** all 14 README acceptance criteria pass; workflow reaches `board_presentation`; super-Merkle external-anchored; regional evidence preserved.

**Fail criteria:** any AC fails.

### T-J165-022 — Character preservation across all artifacts

**Action:** Verify Devanagari (नवीन अय्यर) + Tamil + Hangul + German diacritic + Korean hospital names byte-exact across drive + audit + report PDF + LLM provenance.

**Pass criteria:** every name byte-exact; no normalization.

**Fail criteria:** any normalization.

## Failure scenarios

| Scenario | Expected response |
|---|---|
| Cross-region pull times out on one pack | Surface diagnostic; continue with remaining 7; require Naveen explicit acknowledgment to proceed |
| LLM unavailable | Fall back to template; Naveen drafts manually; provenance metadata records "no_llm_used" |
| Per-pack Merkle compute fails | Block super-Merkle; surface failed pack; require recompute |
| Counsel declines to sign | Block transition; Naveen + Hampton discuss; redraft |
| AC quorum cannot be reached | Block board presentation; Naveen escalates to chair; reschedule |
| External transparency log unavailable | Archive locally; flag pending external anchor; retry batch |
| Drive WORM not engaged | Block archival; surface explicit error; recover |

## Notes for the test author

- The Merkle root determinism test (T-J165-020) is THE highest-priority correctness assertion — non-determinism breaks the external transparency anchor contract.
- The cross-region data residency invariant (T-J165-014 + T-J165-019) is the GDPR + CSAP compliance signature — fuzz this aggressively.
- The Cedar transition guards form a transitive chain: draft → counsel requires cco_signoff; counsel → ac requires counsel_review; ac → board requires quorum. Each guard must reject when its precondition is absent.
- The LLM provenance metadata (T-J165-004) is the EU AI Act Article 50 compliance test for the internal-tooling surface — assert model + license + prompt template + tokens.
- The audit committee quorum-of-3-of-5 (T-J165-010) is the workflow-engine differentiation test — the substrate, not the UI, enforces quorum.
