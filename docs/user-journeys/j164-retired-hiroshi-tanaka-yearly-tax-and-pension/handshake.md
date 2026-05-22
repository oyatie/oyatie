---
doc_class: User-Journey-Handshake
journey_id: j164-retired-hiroshi-tanaka-yearly-tax-and-pension
date: 2026-05-20
authority_tier: 2
status: draft
---

# j164 — Handshake matrix

Every named µservice call for Hiroshi Tanaka's annual tax + pension reconciliation on 2027-02-27 between 09:14 and 14:36 JST. Order matches `story.md`. Every row names principal + tenant + Cedar permit + ADR-0263 audit class. Transport: HTTPS over QUIC (HTTP/3) per ADR-0253. Japanese full-width + Kanji + Hiragana + Katakana preserved UTF-8 NFC byte-exact. Accessibility context (TalkBack / voice / high-contrast) preserved in every call.

## Notation

- `[T]` Tablet (Xiaomi Pad 6 Pro)
- `[NFC]` My-Number Card NFC tap
- `[V]` Voice command
- `→` synchronous request
- `←` synchronous response
- `↪` side-effect on bus
- `⟂` denied path

## §1 Workflow open (09:14:42 JST)

### 1.1 Open workflow-studio

`[T] → workflow-studio` — `POST /v1/workflow-studio/workflows/open`

```json
{
  "tenant_id": "personal-hiroshi-tanaka-jp",
  "principal": "hiroshi.tanaka@personal-hiroshi-tanaka-jp",
  "workflow_id": "annual-tax-prep-fy2026",
  "workflow_template": "kakutei-shinkoku-b-individual-pensioner-jp",
  "fiscal_year": "fy2026",
  "fiscal_year_japanese_era": "reiwa-8",
  "accessibility_context": {
    "talkback_active": true,
    "talkback_voice": "female-voice-2",
    "talkback_speech_rate": 0.85,
    "high_contrast_theme_active": true,
    "high_contrast_palette": {"background": "#0A0A0A", "foreground": "#F0F0F0", "accent": "#FFD700"},
    "large_text_active": true,
    "large_text_minimum_pt": 18,
    "voice_navigation_active": true,
    "haptic_feedback_active": true
  },
  "device_context": {
    "device_class": "tablet",
    "device_model": "xiaomi-pad-6-pro",
    "screen_inches": 12.4,
    "screen_brightness": 70,
    "color_temperature_kelvin": 5400
  },
  "opened_at": "2027-02-27T09:14:42+09:00"
}
```

Cedar: permit (owner + personal_tenant + accessibility_context_valid). Audit: `EVT-J164-WORKFLOW-OPEN-001`.

### 1.2 Voice command "進む" (susumu — proceed)

`[V] → workflow-studio` — `POST /v1/workflow-studio/voice-command`

```json
{
  "workflow_id": "annual-tax-prep-fy2026",
  "voice_transcript_utf8_nfc": "進む",
  "voice_transcript_romanized": "susumu",
  "voice_confidence": 0.94,
  "voice_intent_resolved": "next_step",
  "command_received_at": "2027-02-27T09:42:08+09:00"
}
```

Audit: `EVT-J164-VOICE-CMD-NEXT-Δ001a`.

## §2 Receipt collection (09:42–10:48 JST)

### 2.1 Camera-based receipt OCR (17 invocations)

`[T] → intelligence` — `POST /v1/intelligence/ocr/receipt`

Example (receipt #1):

```json
{
  "tenant_id": "personal-hiroshi-tanaka-jp",
  "image_base64": "<jpeg base64>",
  "image_size_bytes": 421008,
  "ocr_class": "japanese_medical_receipt",
  "expected_fields": ["payee_name", "date", "amount_jpy", "category"],
  "captured_at": "2027-02-27T09:48:18+09:00"
}
```

Response:

```json
{
  "ocr_result": {
    "payee_name": "倉敷中央病院 眼科",
    "payee_name_romanized": "Kurashiki Chuo Byoin Ganka",
    "date": "2026-01-18",
    "date_japanese_era": "reiwa-8-01-18",
    "amount_jpy": 4200,
    "category_inferred": "medical_outpatient",
    "ocr_confidence_per_field": {
      "payee_name": 0.97,
      "date": 0.99,
      "amount_jpy": 0.99
    }
  }
}
```

Audit: `EVT-J164-RECEIPT-OCR-{n}-002a` per row.

### 2.2 Drive write receipt to WORM room

`[T] → drive` — `POST /v1/drive/rooms/{room}/files`

```json
{
  "drive_room": "personal-hiroshi-tanaka-jp/tax/fy2026/receipts",
  "filename": "001-2026-01-18-kurashiki-chuo-byoin-ganka-4200.json",
  "content_type": "application/oyatie.receipt+json",
  "size_bytes": 1208,
  "sha256": "<sha256>",
  "worm": true,
  "worm_until": "2034-02-27T00:00:00+09:00",
  "retention_authority": "JP-Income-Tax-Act-148"
}
```

Audit: `EVT-J164-DRIVE-RECEIPT-WRITE-{n}-002b`.

### 2.3 Receipt collection close gate

`[T] → workflow-studio` — `POST /v1/workflow-studio/step/close`

```json
{
  "workflow_id": "annual-tax-prep-fy2026",
  "step_id": "step_1_receipt_collection",
  "receipts_collected_count": 17,
  "receipts_total_medical_jpy": 126400,
  "receipts_total_property_tax_jpy": 139200,
  "receipts_total_honorarium_jpy": 20000,
  "closed_at": "2027-02-27T10:48:14+09:00"
}
```

Audit: `EVT-J164-RECEIPTS-COLLECTED-002`.

## §3 My-Number Card NFC + pension reconciliation (10:54–11:24 JST)

### 3.1 My-Number Card NFC tap (3 attempts before success)

`[NFC] → identity` — `POST /v1/identity/my-number-card/authenticate`

Attempt 1 (10:54:18):

```json
{
  "tenant_id": "personal-hiroshi-tanaka-jp",
  "subject_principal": "hiroshi.tanaka@personal-hiroshi-tanaka-jp",
  "purpose_scope": "pension_reconciliation",
  "purpose_declared_in_audit": true,
  "nfc_signal_dbm": -54,
  "nfc_read_duration_ms": 320,
  "nfc_read_complete": false,
  "failure_reason": "card_moved_during_read",
  "user_message_class": "calm_retry_no_blame"
}
```

Response: `{"state": "retry_requested", "timeout_seconds_remaining": 28}`. Audit: `EVT-J164-MY-NUMBER-NFC-RETRY-Δ006a`.

Attempt 2 (10:55:42): similar; failure_reason `card_angle_off`. Audit: `EVT-J164-MY-NUMBER-NFC-RETRY-Δ006b`.

Attempt 3 (10:56:42): success.

```json
{
  "tenant_id": "personal-hiroshi-tanaka-jp",
  "subject_principal": "hiroshi.tanaka@personal-hiroshi-tanaka-jp",
  "purpose_scope": "pension_reconciliation",
  "purpose_declared_in_audit": true,
  "nfc_read_complete": true,
  "card_certificate_serial": "JPKI-****-7842",
  "card_validity_ok": true,
  "authenticated_at": "2027-02-27T10:56:42+09:00"
}
```

Cedar: permit (my-number per-purpose `pension_reconciliation` + access count today ≤ 4). Audit: `EVT-J164-MY-NUMBER-NFC-007`.

### 3.2 Pension reconciliation query

`[T] → payments` — `POST /v1/payments/pension/reconcile`

```json
{
  "tenant_id": "personal-hiroshi-tanaka-jp",
  "principal": "hiroshi.tanaka@personal-hiroshi-tanaka-jp",
  "fiscal_year": "fy2026",
  "pension_authority": "jp-jps-nippon-nenkin-kiko",
  "my_number_token_short_lived": "<scoped-token>",
  "scope": "pension_record_read_one_fiscal_year",
  "scope_expires_at": "2027-02-27T11:30:00+09:00"
}
```

Response:

```json
{
  "subject_pension_record_id_anonymized": "****-****-1234",
  "monthly_deposits": [
    {"month": "2026-01", "amount_jpy": 182000, "deposit_date": "2026-01-15"},
    {"month": "2026-02", "amount_jpy": 182000, "deposit_date": "2026-02-15"},
    {"month": "2026-03", "amount_jpy": 182000, "deposit_date": "2026-03-13"},
    {"month": "2026-04", "amount_jpy": 182000, "deposit_date": "2026-04-15"},
    {"month": "2026-05", "amount_jpy": 182000, "deposit_date": "2026-05-15"},
    {"month": "2026-06", "amount_jpy": 182000, "deposit_date": "2026-06-15"},
    {"month": "2026-07", "amount_jpy": 182000, "deposit_date": "2026-07-15"},
    {"month": "2026-08", "amount_jpy": 182000, "deposit_date": "2026-08-14"},
    {"month": "2026-09", "amount_jpy": 182000, "deposit_date": "2026-09-15"},
    {"month": "2026-10", "amount_jpy": 182000, "deposit_date": "2026-10-15"},
    {"month": "2026-11", "amount_jpy": 182000, "deposit_date": "2026-11-13"},
    {"month": "2026-12", "amount_jpy": 182000, "deposit_date": "2026-12-15"}
  ],
  "annual_total_jpy": 2184000,
  "withholding_tax_jpy": 56750
}
```

Cross-check against Chugoku Bank (中国銀行) deposit feed: 12/12 match.

`[T] → payments` — `POST /v1/payments/bank/cross-check`

Audit: `EVT-J164-PENSION-RECONCILED-003`.

## §4 Tax payment reconciliation (11:24–11:54 JST)

`[T] → payments` — `POST /v1/payments/tax/estimated-quarterly/read`

```json
{
  "tenant_id": "personal-hiroshi-tanaka-jp",
  "fiscal_year": "fy2026",
  "tax_class": "estimated_quarterly_individual"
}
```

Response:

```json
{
  "payments": [
    {"period": "fy2026-q1", "amount_jpy": 18400, "paid_at": "2026-07-31"},
    {"period": "fy2026-q2", "amount_jpy": 18400, "paid_at": "2026-10-31"},
    {"period": "fy2026-q3", "amount_jpy": 18400, "paid_at": "2027-01-31"}
  ],
  "total_jpy": 55200,
  "note": "fy2026-q4 settled via kakutei-shinkoku"
}
```

`[T] → payments` — additional calls for 国民健康保険 (¥84,000) + 介護保険 (¥48,000).

Audit: `EVT-J164-TAX-PAYMENTS-RECONCILED-004`.

## §5 Lunch break + workflow pause/resume

### 5.1 Pause

`[V] → workflow-studio` — `POST /v1/workflow-studio/voice-command`

```json
{
  "voice_transcript_utf8_nfc": "休憩",
  "voice_intent_resolved": "pause_workflow",
  "command_received_at": "2027-02-27T11:54:42+09:00"
}
```

Audit: `EVT-J164-WORKFLOW-PAUSE-Δ005a`.

### 5.2 Resume

`[V] → workflow-studio` — voice command "続ける" at 12:24:18. Audit: `EVT-J164-WORKFLOW-RESUME-Δ005b`.

## §6 Year-over-year comparison + form drafting (12:24–13:18 JST)

### 6.1 YoY compare

`[T] → workflow-studio` — `POST /v1/workflow-studio/year-over-year/compare`

```json
{
  "tenant_id": "personal-hiroshi-tanaka-jp",
  "fiscal_year_current": "fy2026",
  "fiscal_year_prior": "fy2025",
  "compare_sections": ["income", "deductions", "withholding", "estimated_tax"]
}
```

Response: side-by-side comparison panel JSON (see ux-flow Screen 4).

Audit: `EVT-J164-YOY-COMPARE-005`.

### 6.2 Form drafting

`[T] → compliance` — `POST /v1/compliance/tax-form/draft`

```json
{
  "tenant_id": "personal-hiroshi-tanaka-jp",
  "principal": "hiroshi.tanaka@personal-hiroshi-tanaka-jp",
  "form_class": "kakutei_shinkoku_b",
  "fiscal_year": "fy2026",
  "income": {
    "public_pension_jpy": 2184000,
    "miscellaneous_income_jpy": 20000,
    "interest_income_jpy": 3200
  },
  "deductions": {
    "social_insurance_premium_jpy": 132000,
    "medical_expense_jpy_eligible": 26400,
    "basic_deduction_jpy": 480000
  },
  "withholding": {
    "pension_withholding_jpy": 56750,
    "estimated_tax_paid_jpy": 55200
  },
  "compute_at": "2027-02-27T13:18:42+09:00"
}
```

Response:

```json
{
  "form_draft_id": "kakutei-shinkoku-b-hiroshi-fy2026-draft-001",
  "income_total_jpy": 2207200,
  "deductions_total_jpy": 638400,
  "taxable_income_jpy": 1568800,
  "income_tax_jpy": 37400,
  "reconstruction_special_tax_jpy": 785,
  "total_tax_jpy": 38185,
  "withholding_total_jpy": 111950,
  "refund_jpy": 73765
}
```

Audit: `EVT-J164-FORM-DRAFTED-006`.

## §7 Review (13:18–13:42 JST)

`[T] → workflow-studio` — `POST /v1/workflow-studio/step/review`

```json
{
  "workflow_id": "annual-tax-prep-fy2026",
  "form_draft_id": "kakutei-shinkoku-b-hiroshi-fy2026-draft-001",
  "section_confirmations": [
    {"section": "income", "confirmed": true, "voice_confirmation": "ご確認しました"},
    {"section": "deductions", "confirmed": true},
    {"section": "withholding", "confirmed": true},
    {"section": "tax_calculation", "confirmed": true},
    {"section": "refund_estimate", "confirmed": true}
  ],
  "in_context_help_invocations": [
    {"section": "miscellaneous_income", "help_id": "honorarium-vs-business-income", "voice_query": "原稿料は雑所得か事業所得か"}
  ],
  "review_complete_at": "2027-02-27T13:42:08+09:00"
}
```

Audit: `EVT-J164-FORM-REVIEW-006a`.

## §8 e-Tax submission (13:42–14:14 JST)

### 8.1 Second My-Number tap (etax_submission purpose)

`[NFC] → identity` — `POST /v1/identity/my-number-card/authenticate`

```json
{
  "purpose_scope": "etax_submission",
  "purpose_declared_in_audit": true,
  "nfc_read_complete": true,
  "authenticated_at": "2027-02-27T13:48:18+09:00"
}
```

Audit: `EVT-J164-MY-NUMBER-NFC-008` (purpose count today: 3 of max 4).

### 8.2 Face-ID + passkey assertion

`[T] → identity` — `POST /v1/identity/passkey/assert`

```json
{
  "principal": "hiroshi.tanaka@personal-hiroshi-tanaka-jp",
  "passkey_class": "face_id_with_pin_fallback",
  "face_id_match_score": 0.96,
  "assertion_duration_ms": 1408,
  "asserted_at": "2027-02-27T13:50:08+09:00"
}
```

Audit: `EVT-J164-PASSKEY-ASSERT-008a`.

### 8.3 e-Tax submission

`[T] → compliance` — `POST /v1/compliance/tax-form/submit-etax`

```json
{
  "tenant_id": "personal-hiroshi-tanaka-jp",
  "form_draft_id": "kakutei-shinkoku-b-hiroshi-fy2026-draft-001",
  "etax_endpoint": "https://e-tax.nta.go.jp/api/v2/individual-submission",
  "channel_cell": "jp-tokyo-etax-linkage-readonly",
  "signature_alg": "JPKI-RSA-2048-with-passkey-counter-sign",
  "attachments": [
    {"role": "form_body", "size_bytes": 3142008},
    {"role": "receipts_bundle", "count": 17}
  ],
  "submitted_at": "2027-02-27T14:14:42+09:00"
}
```

Response:

```json
{
  "etax_acknowledgment": {
    "receipt_number": "20270227-1414-008-T-7842965",
    "received_at": "2027-02-27T14:14:42+09:00",
    "filer_name": "田中 浩",
    "filer_my_number_short": "****-****-1234",
    "form_class": "kakutei_shinkoku_b",
    "fiscal_year": "reiwa-8",
    "refund_jpy": 73765,
    "refund_expected_at": "2027-04-08"
  }
}
```

Cedar: permit (etax_submission + my-number-card-nfc + passkey + form_review_complete + year_over_year_reviewed). Audit: `EVT-J164-ETAX-SUBMITTED-009`.

### 8.4 Receipt archival to drive WORM

`[T] → drive` — `POST /v1/drive/rooms/{room}/files`

```json
{
  "drive_room": "personal-hiroshi-tanaka-jp/tax/fy2026/submission",
  "filename": "etax-receipt-20270227-1414-008-T-7842965.pdf",
  "size_bytes": 218408,
  "worm": true,
  "worm_until": "2034-02-27T00:00:00+09:00"
}
```

Audit: `EVT-J164-ETAX-RECEIPT-ARCHIVED-009a`.

## §9 Diary entry (notes)

`[V] → notes` — `POST /v1/notes/voice-dictate`

```json
{
  "tenant_id": "personal-hiroshi-tanaka-jp",
  "principal": "hiroshi.tanaka@personal-hiroshi-tanaka-jp",
  "notebook_id": "late-life-record-keeping",
  "voice_transcript_utf8_nfc": "令和9年2月27日。確定申告を提出した。還付は7万3千7百65円。タマは元気。みさきから明後日電話するそうだ。サチコへ — 今年もちゃんと終わらせました。あなたが見ていてくれているといいな。",
  "voice_confidence": 0.96,
  "succession_ready_tag": true,
  "series_continuation": "annual-letter-to-sachiko",
  "dictated_at": "2027-02-27T14:36:18+09:00"
}
```

Audit: `EVT-J164-NOTES-DIARY-008`.

## §10 My-Number access log (PIPA Article 19 compliance)

At end of journey, the compliance µservice writes the per-purpose My-Number access summary:

`[T] → compliance` — `POST /v1/compliance/my-number/access-log-summary`

```json
{
  "tenant_id": "personal-hiroshi-tanaka-jp",
  "subject_principal": "hiroshi.tanaka@personal-hiroshi-tanaka-jp",
  "date": "2027-02-27",
  "accesses": [
    {"purpose_scope": "pension_reconciliation", "accessed_at": "2027-02-27T10:56:42+09:00", "audit_event_id": "EVT-J164-MY-NUMBER-NFC-007"},
    {"purpose_scope": "etax_submission", "accessed_at": "2027-02-27T13:48:18+09:00", "audit_event_id": "EVT-J164-MY-NUMBER-NFC-008"}
  ],
  "access_count": 2,
  "daily_ceiling": 4,
  "bleed_detected": false
}
```

Audit: `EVT-J164-MY-NUMBER-DAILY-SUMMARY-011`.

## §11 Denied paths

### 11.1 ⟂ Workflow Misaki attempts to read Hiroshi's tax form without succession activation

`[external:misaki] → drive` — `GET /v1/drive/rooms/personal-hiroshi-tanaka-jp/tax/fy2026/`

Cedar: forbid (succession not yet activated; cross-tenant access blocked). Audit: `EVT-J164-CEDAR-DENY-DAUGHTER-Δ010`.

### 11.2 ⟂ My-Number access without declared purpose

`[T] → identity` — `POST /v1/identity/my-number-card/authenticate` with `purpose_scope: ""` (empty).

Cedar: forbid (purpose_scope must be in declared whitelist). Audit: `EVT-J164-CEDAR-DENY-MY-NUMBER-NO-PURPOSE-Δ011`.

### 11.3 ⟂ e-Tax submission without form_review_complete

Cedar: forbid (pre-condition missing). Audit: `EVT-J164-CEDAR-DENY-ETAX-NOT-REVIEWED-Δ012`.

## §12 SLA + accessibility summary

| Stage | Substance | Observed |
|---|---|---|
| Workflow open | TalkBack reads ≤ 600ms after tap | 412ms |
| Voice command recognition | ≥ 92% confidence | 94% average |
| OCR per receipt | ≤ 4s | 2.1s avg |
| My-Number NFC retry tolerance | 30s timeout per attempt | retries handled gracefully |
| Pension reconciliation | ≤ 8s | 4.8s |
| Form drafting compute | ≤ 12s | 6.4s |
| e-Tax submission round-trip | ≤ 60s | 24s |
| TalkBack coverage | 100% of interactive elements | 100% |
| High-contrast theme | active throughout | active |
| Large-text body ≥ 18pt | active | 18pt confirmed |
| Haptic feedback per action | confirm + denial patterns | all confirmed |
