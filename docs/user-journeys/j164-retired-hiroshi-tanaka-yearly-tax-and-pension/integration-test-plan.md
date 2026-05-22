---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j164-retired-hiroshi-tanaka-yearly-tax-and-pension
date: 2026-05-20
authority_tier: 2
status: draft
---

# j164 — Integration test plan

Intern-buildable plan: stand up the seeded `personal-hiroshi-tanaka-jp` fixture with all 17 FY2026 receipts pre-photographed; mock JPS pension feed; mock Chugoku Bank deposit feed; mock the 国税庁 e-Tax endpoint; seed Hiroshi's My-Number Card with NFC retry simulation; activate JIS X 8341-3 accessibility substrate (TalkBack + voice + high-contrast + large-text). Walk every test in order.

## Test environment

| Component | Source |
|---|---|
| Seed tenant | `tests/fixtures/tenants/personal-hiroshi-tanaka-jp.yaml` |
| Seed persona | `tests/fixtures/personas/hiroshi-tanaka.yaml` |
| Seed receipts | `tests/fixtures/drive/fy2026-receipts-17.yaml` (17 receipts with photo + ground-truth OCR) |
| Seed pension feed | `tests/fixtures/payments/jps-fy2026-pension-feed-hiroshi.yaml` (12 monthly deposits) |
| Seed bank feed | `tests/fixtures/payments/chugoku-bank-fy2026-hiroshi.yaml` |
| Seed estimated tax payments | `tests/fixtures/payments/estimated-quarterly-fy2026-hiroshi.yaml` |
| Seed My-Number Card | `tests/fixtures/identity/my-number-card-hiroshi-jpki.yaml` |
| Seed Cedar bundle | `tests/fixtures/cedar/j164/cedar-bundle-retiree-tax-prep-v1.cedar` |
| Wire mock — e-Tax | `tests/mocks/nta-etax-individual-submission.toml` |
| Wire mock — NFC retry | `tests/mocks/nfc-retry-with-tremor-simulation.toml` |
| Wire mock — TalkBack assertion | `tests/mocks/talkback-assertion-substrate.toml` |
| Frozen clock | `freeze_clock(2027-02-27T09:14:08+09:00)` |

## Seed data summary

| Datum | Value |
|---|---|
| Hiroshi's tenant ID | `personal-hiroshi-tanaka-jp` |
| Hiroshi's primary cell | `jp-osaka-tier-2-personal-retiree` |
| My-Number daily access ceiling | 4 |
| Tax form class | `kakutei_shinkoku_b` |
| Fiscal year | `fy2026` (= 令和8年度) |
| Filing deadline | `2027-03-15` |
| Days remaining | 16 |
| Pension annual total | ¥2,184,000 |
| Medical expense total | ¥126,400 |
| Medical deduction eligible | ¥26,400 (excess over ¥100,000) |
| Expected refund | ¥73,765 |

## Test catalog

### T-J164-001 — Workflow opens with TalkBack within 600ms

**Pre-conditions:** clock `2027-02-27T09:14:08+09:00`. Tablet at home screen. Accessibility substrate active.

**Action:**

1. Hiroshi taps the workflow icon
2. TalkBack receives the announcement event
3. Haptic motor fires confirm pulse + open pulse

**Expected events:**

- `EVT-J164-WORKFLOW-OPEN-001` sealed
- TalkBack announcement event matches "ワークフロー・スタジオを開いています。お待ちください。"

**Pass criteria:**

- Time from tap to TalkBack first phoneme ≤ 600ms (measured 412ms)
- Haptic confirm pulse fires
- Active-tenant pill shows `personal-hiroshi-tanaka-jp`
- High-contrast theme + 18pt+ body text + TalkBack voice "female-voice-2" at 0.85x rate all asserted

**Fail criteria:** TalkBack delayed > 600ms; missing haptic; wrong theme.

### T-J164-002 — Voice command "進む" advances workflow

**Action:** Voice command "進む" (susumu — proceed) emitted.

**Expected events:**

- `EVT-J164-VOICE-CMD-NEXT-Δ001a` sealed
- Voice confidence 0.94
- Workflow advances to step 1 (receipt collection)

**Pass criteria:**

- Voice intent resolved to `next_step`
- Confidence ≥ 0.92 (project SLA)
- Step transition succeeds within 200ms

**Fail criteria:** confidence < 0.92; wrong intent resolved; no transition.

### T-J164-003 — 17 receipts OCR'd with per-receipt verification

**Action:** Run through all 17 receipt photos via camera + OCR.

**Expected events:**

- 17 × `EVT-J164-RECEIPT-OCR-{n}-002a`
- 17 × `EVT-J164-DRIVE-RECEIPT-WRITE-{n}-002b`
- `EVT-J164-RECEIPTS-COLLECTED-002` close gate

**Pass criteria:**

- All 17 receipts OCR'd with payee + date + amount confidence ≥ 0.95
- Tama-bump simulation on receipt #5 triggers calm retry message (not error)
- Medical total = ¥126,400, property tax = ¥139,200, honorarium = ¥20,000
- Each receipt archived to drive WORM with 7-year retention
- Kanji preservation byte-exact (`倉敷中央病院` + `はやし整形外科` + `やまもと歯科`)

**Fail criteria:** any OCR confidence < 0.95; failure language used; missing WORM lock.

### T-J164-004 — My-Number NFC patient retry (3 attempts to success)

**Action:** Simulate hand-tremor NFC misalignment on attempts 1 + 2; success on attempt 3.

**Expected events:**

- `EVT-J164-MY-NUMBER-NFC-RETRY-Δ006a` (attempt 1 failure: card moved during read)
- `EVT-J164-MY-NUMBER-NFC-RETRY-Δ006b` (attempt 2 failure: card angle off)
- `EVT-J164-MY-NUMBER-NFC-007` (attempt 3 success at 10:56:42 JST)

**Pass criteria:**

- 30-second timeout per attempt (no premature failure)
- Retry message is "もう一度どうぞ" not "エラー" / "失敗"
- Purpose scope declared: `pension_reconciliation`
- Per-purpose access count incremented to 1
- JPKI card certificate validated

**Fail criteria:** timeout < 30s; failure language used; purpose missing; access count not incremented.

### T-J164-005 — Pension reconciliation against bank feed

**Action:** Query JPS feed + cross-check Chugoku Bank deposits.

**Expected events:**

- `EVT-J164-PENSION-RECONCILED-003` sealed

**Pass criteria:**

- 12 monthly deposits each ¥182,000
- Annual total ¥2,184,000
- Withholding ¥56,750
- 12/12 bank cross-check match
- All 12 deposit dates byte-exact (note 2026-03-13, 08-14, 11-13 are non-15th due to weekend roll)

**Fail criteria:** missing deposit; cross-check mismatch; wrong total.

### T-J164-006 — Tax payment reconciliation

**Action:** Query estimated quarterly tax + national health insurance + long-term care insurance.

**Expected events:**

- `EVT-J164-TAX-PAYMENTS-RECONCILED-004` sealed

**Pass criteria:**

- 3 estimated quarterly payments × ¥18,400 = ¥55,200
- 国民健康保険 ¥84,000 (12 monthly × ¥7,000)
- 介護保険 ¥48,000 (12 monthly × ¥4,000)
- All bank-debited dates verified

**Fail criteria:** any payment missing; total mismatch.

### T-J164-007 — Workflow pause + resume (lunch break)

**Action:** Voice command "休憩" at 11:54:42; voice command "続ける" at 12:24:18.

**Expected events:**

- `EVT-J164-WORKFLOW-PAUSE-Δ005a`
- `EVT-J164-WORKFLOW-RESUME-Δ005b`

**Pass criteria:**

- State durably persisted to `personal-hiroshi-tanaka-jp`
- Resume restores exact step + receipt count
- 30-minute pause does not time out
- TalkBack acknowledges pause + resume

**Fail criteria:** state loss; premature timeout; wrong step on resume.

### T-J164-008 — Year-over-year compare with TalkBack reading delta

**Action:** Generate FY2026 vs FY2025 comparison panel.

**Expected events:**

- `EVT-J164-YOY-COMPARE-005` sealed

**Pass criteria:**

- Income side: pension ±¥0, honorarium +¥20,000 (first appearance), interest ±¥0
- Deduction side: medical +¥28,000 (with breakdown showing ¥26,400 newly eligible)
- Withholding side: pension ±¥0, estimated tax -¥18,400
- TalkBack reads "プラス" for + and "マイナス" for −
- Voice command "詳しく" zooms into a specific row
- The honorarium first-appearance highlighted with gold underline

**Fail criteria:** wrong delta; missing breakdown; wrong TalkBack phrasing.

### T-J164-009 — Form drafting math correctness

**Action:** Compliance µservice drafts kakutei-shinkoku-b form.

**Expected events:**

- `EVT-J164-FORM-DRAFTED-006` sealed

**Pass criteria:**

- Income total ¥2,207,200
- Deductions total ¥638,400
- Taxable income ¥1,568,800
- Income tax ¥37,400
- Reconstruction special tax ¥785
- Total tax ¥38,185
- Withholding total ¥111,950
- Refund ¥73,765
- Public pension deduction correctly applied (this is where the YoY preview's ¥17,800 understated; actual is ¥73,765)

**Fail criteria:** any math error; public pension deduction omitted.

### T-J164-010 — In-context help: honorarium vs business income

**Action:** Voice query "原稿料は雑所得か事業所得か".

**Pass criteria:**

- Help surfaces with 4 reasoning bullets
- Resolution: 雑所得 (correct for Hiroshi's profile)
- Voice command "良し、進む" advances

**Fail criteria:** wrong classification; missing reasoning.

### T-J164-011 — Second My-Number tap (etax purpose) succeeds first try

**Action:** Hiroshi taps for `etax_submission` purpose at 13:48:18 JST (warmer hands, kerosene stove worked).

**Expected events:**

- `EVT-J164-MY-NUMBER-NFC-008` sealed
- Purpose count today: 2 (under 4 ceiling)

**Pass criteria:**

- First attempt succeeds
- Purpose scope `etax_submission` declared
- Per-purpose access counter incremented correctly

**Fail criteria:** wrong purpose; count not incremented; access > 4.

### T-J164-012 — Face-ID passkey assertion succeeds

**Action:** Hiroshi performs face authentication.

**Pass criteria:**

- Face match score ≥ 0.95 (measured 0.96)
- Assertion duration ≤ 2000ms (measured 1408ms)
- Audit `EVT-J164-PASSKEY-ASSERT-008a` sealed

**Fail criteria:** match < 0.95; duration > 2000ms.

### T-J164-013 — e-Tax submission round-trip

**Action:** Submit form to 国税庁 e-Tax via QUIC channel.

**Expected events:**

- `EVT-J164-ETAX-SUBMITTED-009` sealed
- NTA acknowledgment receipt arrives

**Pass criteria:**

- Submission size 3.2 MB + 17 receipts
- Round-trip ≤ 60s (measured 24s)
- Receipt number issued: `20270227-1414-008-T-7842965`
- Refund amount echoed back: ¥73,765
- Refund expected date echoed: 2027-04-08
- Signature algorithm `JPKI-RSA-2048-with-passkey-counter-sign` validated

**Fail criteria:** round-trip > 60s; missing receipt number; refund mismatch.

### T-J164-014 — e-Tax receipt archived to drive WORM

**Action:** Archive NTA receipt PDF.

**Expected events:**

- `EVT-J164-ETAX-RECEIPT-ARCHIVED-009a` sealed

**Pass criteria:**

- WORM lock engaged
- Retention timer set to 2034-02-27 (7-year)
- PDF size 218408 bytes
- File path correct

**Fail criteria:** WORM missing; retention < 7 years.

### T-J164-015 — Diary entry via voice dictation

**Action:** Hiroshi dictates the diary entry to Sachiko.

**Expected events:**

- `EVT-J164-NOTES-DIARY-008` sealed

**Pass criteria:**

- Voice transcription confidence ≥ 0.95 (measured 0.96)
- Difficult name "サチコ" preserved from Hiroshi's contact graph
- Series-continuation tag `annual-letter-to-sachiko` (4th year)
- Succession-ready tag present
- Kanji + Hiragana + Katakana byte-exact preservation
- Notebook ID `late-life-record-keeping`

**Fail criteria:** confidence < 0.95; Sachiko name failure; missing tags.

### T-J164-016 — My-Number daily access log + PIPA compliance

**Action:** End-of-journey summary of My-Number accesses.

**Expected events:**

- `EVT-J164-MY-NUMBER-DAILY-SUMMARY-011` sealed

**Pass criteria:**

- 2 accesses total today (pension_reconciliation + etax_submission)
- Each access has declared purpose scope
- `bleed_detected == false`
- Both purposes match the whitelist
- Under daily ceiling 4

**Fail criteria:** > 4 accesses; missing purpose declaration; bleed detected.

### T-J164-017 — Forbid: external party reads Hiroshi's tax form

**Action:** Misaki attempts to read `personal-hiroshi-tanaka-jp/tax/fy2026/` without succession activation.

**Expected events:**

- Cedar deny
- `EVT-J164-CEDAR-DENY-DAUGHTER-Δ010` sealed

**Pass criteria:** 403; no data leaked.

**Fail criteria:** any data leak.

### T-J164-018 — Forbid: My-Number access without declared purpose

**Action:** Attempt NFC with empty `purpose_scope`.

**Expected events:**

- Cedar deny
- `EVT-J164-CEDAR-DENY-MY-NUMBER-NO-PURPOSE-Δ011` sealed

**Pass criteria:** Cedar deny; no My-Number read.

**Fail criteria:** any read without purpose.

### T-J164-019 — Forbid: e-Tax without form review complete

**Action:** Attempt e-Tax submission with `form_review_complete=false`.

**Expected events:**

- Cedar deny
- `EVT-J164-CEDAR-DENY-ETAX-NOT-REVIEWED-Δ012` sealed

**Pass criteria:** Cedar deny; no submission.

**Fail criteria:** submission proceeds without review.

### T-J164-020 — Accessibility invariants throughout

**Action:** Run accessibility audit across all screens during the journey.

**Pass criteria:**

- TalkBack coverage 100% of interactive elements
- Voice command recognition ≥ 92% on canonical command set
- High-contrast theme active throughout
- Body text ≥ 18pt; icon labels ≥ 24pt bold
- Haptic feedback on every action (tap-confirm + action-complete + retry-needed)
- No red-only error indication
- 30-second timeouts on all NFC/network operations
- No "error" / "failure" language; "もう一度どうぞ" + "お待ちください" used

**Fail criteria:** any invariant violated.

### T-J164-021 — Japanese character preservation

**Action:** Verify byte-exact preservation across drive + notes + audit + e-Tax submission.

**Pass criteria:**

- `田中浩` (Hiroshi's name) byte-exact in all artifacts
- `倉敷中央病院` byte-exact in OCR + drive + form
- `サチコ` byte-exact in diary entry
- `令和8年度` byte-exact in form
- Full-width vs half-width preservation maintained

**Fail criteria:** any normalization; any byte difference.

### T-J164-022 — End-to-end happy path replay

**Action:** Run the full 09:14 JST → 14:36 JST journey on the seeded fixture.

**Pass criteria:**

- All 12 README acceptance criteria pass
- All audit events emitted in canonical order
- Form computation deterministic across two runs
- e-Tax acknowledgment receipt reproducible (mocked NTA)
- Total wall-clock 5h22m (within 5h30m ceiling)

**Fail criteria:** any AC fails; non-deterministic computation.

## Failure scenarios

| Scenario | Expected response |
|---|---|
| NFC card never reads (5+ failures) | Surface calm guidance + offer alternative authentication (passkey-only mode) |
| Pension feed unavailable | Block step 2; surface "JPSへのアクセスが一時的にできません" + retry button |
| OCR confidence < 0.85 on a receipt | Ask user to confirm parsed values out loud; only proceed if voice confirms |
| Hiroshi voice command not recognized | TalkBack reads "もう一度どうぞ"; voice transcript saved for diagnostic |
| e-Tax endpoint times out | Save draft locally + retry; never lose form state |
| My-Number access ceiling reached (4 used) | Block; surface "今日のマイナンバー使用上限に達しました。明日再試行してください" |
| Tablet runs out of battery mid-workflow | State persisted to cell; resume from exact step on next session |

## Notes for the test author

- The "patient retry" pattern (no blame language; 30s timeout; haptic gentle) is the highest-priority test surface for this persona — TDD this aggressively.
- The math-correctness test is independent of the accessibility test; both must pass.
- The My-Number per-purpose access log is THE PIPA compliance test — failure here is a regulatory blocker.
- The "Sachiko" name preservation in voice dictation is the late-life record-keeping signature test.
- Year-over-year comparison correctness (especially the "first-time honorarium row" highlighting) is the workflow-studio differentiation test.
