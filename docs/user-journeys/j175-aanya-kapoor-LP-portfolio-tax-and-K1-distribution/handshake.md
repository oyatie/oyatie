---
doc_class: User-Journey-Handshake
journey_id: j175-aanya-kapoor-LP-portfolio-tax-and-K1-distribution
date: 2026-05-20
authority_tier: 2
status: draft
---

# j175 — Handshake matrix

Every named µservice call for the 5-day K-1 reconciliation cycle (May 20 19:48 PDT → May 24 21:18 PDT). Transport HTTPS over QUIC per ADR-0253. Cross-tenant Cedar-validated per ADR-0244. Hindi (Devanagari) + English + Tamil + Mandarin + Japanese + Indonesian preservation UTF-8 NFC byte-exact.

## Notation

- `[LP]` Aanya's personal tenant principal
- `[GP]` Fund GP tenant principal (a16z / Sequoia / KKR / Insight)
- `[CPA]` Wells Goldman & Associates CPA principal
- `[IRS]` IRS Direct Pay principal
- `[STATE]` State revenue department principal (CA FTB / NY DTF / etc.)
- `→` synchronous request
- `←` synchronous response
- `↪` side-effect on bus

## §1 LP cockpit open (May 20 19:48 PDT)

`[LP:aanya.kapoor] → lp-cockpit` — `GET /v1/lp-cockpit/open`

```json
{
  "principal": "aanya.kapoor@aanya-kapoor-personal-2008",
  "accredited_investor_attestation_id": "ai-aanya-2027-wells-goldman-Δ4801",
  "qualified_purchaser_attestation_id": "qp-aanya-2027-wells-goldman-Δ4802",
  "passkey_assertion_token": "wb-jwt-...",
  "face_attestation_token": "fa-aanya-2027-..."
}
```

Cedar: permit (personal_tenant + accredited + qualified_purchaser + passkey + face). Audit: `EVT-J175-LP-COCKPIT-OPENED-Δ000`.

## §2 K-1 PDF ingestion (4 funds)

`[GP:any] → finops-portal` — `POST /v1/finops/k1/ingest`

```protobuf
message K1IngestRequest {
  string lp_principal = 1;
  string fund_gp_tenant = 2;
  string fund_name = 3;
  string fund_vintage = 4;
  string filename = 5;
  bytes pdf_content = 6;
  string content_sha256 = 7;
  string tax_year = 8;
  string filing_form = 9;            // "Form_1065_Schedule_K-1"
  bool schedule_k_3_attached = 10;
}
```

Cedar: permit (fund_gp_tenant + lp_principal_recipient). Audit: `EVT-J175-K1-INGESTED-Δ001a-{fund}`; composite `EVT-J175-K1-INGESTED-001`.

## §3 LP capital account reconciliation

`[LP:aanya.kapoor] → finops-portal` — `POST /v1/finops/lp-capital-account/reconcile`

```json
{
  "lp_session_id": "lp-reconciliation-aanya-fy2026",
  "fund_capital_accounts": [
    {
      "fund_name": "Andreessen Horowitz Fund VII LP",
      "opening_capital_account_2026": 3184228,
      "contributions_2026": 0,
      "distributions_2026": 282184,
      "ordinary_income_allocated": 42184,
      "capital_gain_allocated_LT": 148228,
      "closing_capital_account_2026": 3840456,
      "gp_capital_account_statement_id": "a16z-capital-account-statement-Δ4810"
    },
    {
      "fund_name": "Sequoia Capital U.S. Growth Fund IX",
      "opening_capital_account_2026": 3184128,
      "contributions_2026": 0,
      "distributions_2026": 182184,
      "ordinary_income_allocated": 32148,
      "capital_gain_allocated_LT": 62184,
      "capital_gain_allocated_ST": 22044,
      "closing_capital_account_2026": 3410184
    },
    {
      "fund_name": "KKR Asian Fund V",
      "opening_capital_account_2026": 1884228,
      "contributions_2026": 400000,
      "distributions_2026": 84128,
      "ordinary_income_allocated": 24184,
      "capital_gain_allocated_LT": 48184,
      "foreign_source_income_allocated": 34148,
      "closing_capital_account_2026": 2420156
    },
    {
      "fund_name": "Insight Venture Partners XII LP",
      "opening_capital_account_2026": 2200128,
      "contributions_2026": 300000,
      "distributions_2026": 48184,
      "ordinary_income_allocated": 48228,
      "capital_gain_allocated_LT": 182184,
      "section_199a_allocated": 24184,
      "closing_capital_account_2026": 2710184
    }
  ],
  "aggregate_lp_capital_eod_2026": 12380980,
  "total_committed": 14200000,
  "uncalled_remaining": 1819020
}
```

Audit: `EVT-J175-CAPITAL-ACCOUNT-RECONCILED-002`.

## §4 Tax-character categorization

`[LP:aanya.kapoor] → finops-portal` — `POST /v1/finops/tax-character/categorize`

```protobuf
message TaxCharacterCategorizeRequest {
  string lp_session_id = 1;
  repeated K1LineItems per_fund = 2;
}

message K1LineItems {
  string fund_name = 1;
  double ordinary_income = 2;
  double ltcg = 3;
  double stcg = 4;
  double qualified_dividends = 5;
  double interest_income = 6;
  double section_199a = 7;
  double foreign_source_income = 8;
}
```

Audit: `EVT-J175-TAX-CHARACTER-003`.

## §5 Section 199A QBI compute

`[LP:aanya.kapoor] → finops-portal` — `POST /v1/finops/section-199a/compute`

```json
{
  "qbi_aggregate": 42332,
  "gross_199a_at_20pct": 8466,
  "w2_income_2026": 892000,
  "taxable_income_2026_projected": 1484228,
  "filing_status": "married_filing_jointly",
  "phaseout_threshold_mfj_2026": 383900,
  "phaseout_completion_mfj_2026": 483900,
  "phaseout_state": "fully_phased_out",
  "effective_199a_deduction_after_phaseout": 0,
  "computed_at": "2027-05-21T21:42:00-07:00"
}
```

Audit: `EVT-J175-SECTION-199A-COMPUTED-004`.

## §6 Section 1411 NIIT compute

`[LP:aanya.kapoor] → finops-portal` — `POST /v1/finops/section-1411-niit/compute`

```json
{
  "net_investment_income": 719030,
  "magi_2026_projection": 1488228,
  "magi_threshold_mfj": 250000,
  "niit_base": 719030,
  "niit_rate": 0.038,
  "niit_owed": 27323
}
```

Audit: `EVT-J175-SECTION-1411-NIIT-005`.

## §7 State-by-state apportionment

`[LP:aanya.kapoor] → compliance` — `POST /v1/compliance/state-apportionment/compute`

```protobuf
message StateApportionmentRequest {
  string lp_session_id = 1;
  string residence_state = 2;
  repeated PerFundStateBreakdown per_fund = 3;
}

message PerFundStateBreakdown {
  string fund_name = 1;
  map<string, double> state_source_percentages = 2;  // {"CA": 0.62, "NY": 0.18, ...}
  double total_k1_income = 3;
  map<string, double> foreign_source_percentages = 4;  // if applicable
}

message StateApportionmentResponse {
  map<string, double> per_state_aanya_income = 1;
  double ca_projected_tax = 2;
  double out_of_state_credit_aggregate = 3;
  double net_ca_tax_after_credit = 4;
}
```

Audit: `EVT-J175-STATE-APPORTIONMENT-006`.

## §8 Foreign tax credit (Form 1116) compute

`[LP:aanya.kapoor] → compliance` — `POST /v1/compliance/foreign-tax-credit/compute`

```json
{
  "lp_session_id": "lp-reconciliation-aanya-fy2026",
  "foreign_taxes_paid": [
    {"jurisdiction": "Canada", "amount": 452, "fund": "a16z"},
    {"jurisdiction": "UK", "amount": 516, "fund": "a16z"},
    {"jurisdiction": "Singapore", "amount": 14182, "fund": "kkr"},
    {"jurisdiction": "India", "amount": 8442, "fund": "kkr"},
    {"jurisdiction": "Indonesia", "amount": 4808, "fund": "kkr"},
    {"jurisdiction": "Hong Kong", "amount": 2824, "fund": "kkr"}
  ],
  "ftc_basket_passive": 4824,
  "ftc_basket_general": 0,
  "ftc_creditable_this_year": 4824,
  "ftc_unused_carryforward_to_2027": 30144
}
```

Audit: `EVT-J175-FOREIGN-TAX-CREDIT-007`.

## §9 AMT compute

`[LP:aanya.kapoor] → compliance` — `POST /v1/compliance/amt/compute`

```json
{
  "amti": 1448228,
  "amt_exemption_mfj_2026": 133300,
  "phaseout_completion": 1218700,
  "exemption_after_phaseout": 0,
  "amti_minus_exemption": 1448228,
  "amt_tentative": 405503,
  "regular_tax": 432184,
  "amt_owed": 0
}
```

Audit: `EVT-J175-AMT-COMPUTED-008`.

## §10 Quarterly estimated tax payments

### 10.1 IRS payment

`[LP:aanya.kapoor] → payments` — `POST /v1/payments/irs-direct-pay/dispatch`

```json
{
  "tax_year": "2026",
  "quarter": "Q2",
  "amount_usd": 48228,
  "payment_method": "ACH",
  "irs_direct_pay_account_id": "aanya-direct-pay-2027",
  "filer_ssn_last4": "[REDACTED]",
  "payment_at": "2027-05-23T11:42:18-07:00"
}
```

Cedar: permit (personal_tenant + sanctions_clean_for_govt). Audit: `EVT-J175-ESTIMATED-TAX-IRS-Δ009a`.

### 10.2 State payments (×4 states + others)

`[LP:aanya.kapoor] → payments` — `POST /v1/payments/state-revenue/dispatch` (×4)

```json
{
  "tax_year": "2026",
  "quarter": "Q2",
  "state": "CA",
  "state_revenue_dept": "CA_FTB",
  "amount_usd": 24648,
  "payment_method": "ACH_via_ftb_web_pay",
  "payment_at": "2027-05-23T11:44:18-07:00"
}
```

Audit: `EVT-J175-ESTIMATED-TAX-STATE-Δ009-{state}`; composite `EVT-J175-ESTIMATED-TAX-PAID-009`.

## §11 GP-LP communication channel (2 clarification dialogues)

### 11.1 KKR Asian Fund V — Indonesia FTC clarification

`[LP:aanya.kapoor] → connect` — `POST /v1/connect/gp-lp-channel/send`

```protobuf
message GPLPMessageRequest {
  string channel_id = 1;             // "gp-lp-channel-aanya-kkr-asian-fund-v"
  string sender_principal = 2;
  PayloadClass payload_class = 3;
  bytes mls_encrypted_payload = 4;
  google.protobuf.Timestamp sent_at = 5;
}

enum PayloadClass {
  PAYLOAD_CLASS_UNSPECIFIED = 0;
  K1_CLARIFICATION_QUESTION = 1;
  K1_CLARIFICATION_ANSWER = 2;
  CAPITAL_CALL_NOTICE = 3;
  DISTRIBUTION_NOTICE = 4;
  PARTNER_ALLOCATION_CLARIFICATION = 5;
}
```

Audit: `EVT-J175-GP-LP-CLARIFICATION-Δ010-kkr-Δ001` + `EVT-J175-GP-LP-CLARIFICATION-Δ010-kkr-Δ002` (answer).

### 11.2 Insight Section 199A clarification

`[LP:aanya.kapoor] → connect` — same RPC with channel_id=`gp-lp-channel-aanya-insight-xii`.

Audit: `EVT-J175-GP-LP-CLARIFICATION-Δ010-insight-Δ001`.

### 11.3 Composite

Audit: `EVT-J175-GP-LP-CLARIFICATIONS-010`.

## §12 WORM archival (16 artifacts)

`[LP:aanya.kapoor] → drive` — `POST /v1/drive/k1-worm/archive`

```protobuf
message K1WORMArchiveRequest {
  string lp_session_id = 1;
  string worm_cell = 2;                    // "us-west-tier-1-worm-irs-retention"
  string seal_class = 3;                   // "irs-aligned-worm-class-1"
  uint32 retention_years_minimum = 4;       // 7
  bool indelible_storage_attestation = 5;
  string time_stamp_authority_id = 6;
  repeated ArtifactSeal artifacts = 7;
}

message ArtifactSeal {
  string artifact_id = 1;
  string filename = 2;
  uint64 size_bytes = 3;
  string sha256 = 4;
  string mime_type = 5;
  ArtifactClass artifact_class = 6;       // K1_PDF | CAPITAL_ACCOUNT_STATEMENT | PARTNER_ALLOCATION_SCHEDULE | FOREIGN_TAX_CREDIT_FOOTNOTE
}
```

Audit: `EVT-J175-WORM-ARCHIVED-011`.

## §13 CPA package + pack manifest + audit-chain attestation

### 13.1 CPA package delivery

`[LP:aanya.kapoor] → drive` — `POST /v1/drive/cpa-package/share`

```json
{
  "package_id": "cpa-package-aanya-kapoor-fy2026-2027-05-24",
  "recipient_tenant": "wells-goldman-cpa",
  "recipient_principal": "patricia.wells-goldman@wells-goldman-cpa",
  "contents_manifest": "[...listed in story §6...]",
  "delivery_method": "drive_shared_with_cpa_tenant_read_only"
}
```

Audit: `EVT-J175-CPA-PACKAGE-SENT-Δ012a`.

### 13.2 Pack manifest

`[LP:aanya.kapoor] → compliance` — `GET /v1/compliance/pack-manifest?session=lp-reconciliation-aanya-fy2026`

```json
{
  "active_packs": [
    "pack-irs-schedule-k-1-1065-v3",
    "pack-irs-section-199a-qbi-2026",
    "pack-irs-section-1411-niit-2026",
    "pack-state-tax-apportionment-multi-2026",
    "pack-irc-section-754-step-up",
    "pack-amt-2026",
    "pack-foreign-tax-credit-form-1116-2026",
    "pack-eu-aifmd-non-eu-fund-marketing",
    "pack-uk-nppr-non-eu-fund",
    "pack-accredited-investor-reg-501-rule-144a"
  ],
  "cross_validation_state": "passed",
  "pack_manifest_signature": "sha256:e8f4...a921"
}
```

Audit: `EVT-J175-PACK-MANIFEST-Δ012b`.

## §14 Summary

| Event class | Count | Cedar permits | Cross-tenant |
|---|---|---|---|
| EVT-J175-LP-COCKPIT-OPENED-Δ000 | 1 | accredited + qualified_purchaser | no |
| EVT-J175-K1-INGESTED-Δ001a-{fund} | 4 | fund_gp + lp_recipient | yes (4 GP tenants) |
| EVT-J175-K1-INGESTED-001 | 1 composite | | |
| EVT-J175-CAPITAL-ACCOUNT-RECONCILED-002 | 1 | finops + lp | no |
| EVT-J175-TAX-CHARACTER-003 | 1 | finops | no |
| EVT-J175-SECTION-199A-COMPUTED-004 | 1 | finops | no |
| EVT-J175-SECTION-1411-NIIT-005 | 1 | finops | no |
| EVT-J175-STATE-APPORTIONMENT-006 | 1 | compliance | no |
| EVT-J175-FOREIGN-TAX-CREDIT-007 | 1 | compliance | yes (4 foreign juris) |
| EVT-J175-AMT-COMPUTED-008 | 1 | compliance | no |
| EVT-J175-ESTIMATED-TAX-IRS-Δ009a | 1 | payments + irs | yes (IRS) |
| EVT-J175-ESTIMATED-TAX-STATE-Δ009-{state} | 4 | payments + state | yes (state) |
| EVT-J175-ESTIMATED-TAX-PAID-009 | 1 composite | | |
| EVT-J175-GP-LP-CLARIFICATION-Δ010-{fund}-Δ{n} | 4 | connect + gp_lp | yes (GP tenants) |
| EVT-J175-GP-LP-CLARIFICATIONS-010 | 1 composite | | |
| EVT-J175-WORM-ARCHIVED-011 | 1 (16 artifacts) | drive + worm | no |
| EVT-J175-CPA-PACKAGE-SENT-Δ012a | 1 | drive + cpa | yes (CPA tenant) |
| EVT-J175-PACK-MANIFEST-Δ012b | 1 | compliance | no |

Total: ~25 substantive audit events across 5-day intensive cycle. Cross-tenant Cedar-validated. WORM 7-year retention. Multi-language preservation UTF-8 NFC byte-exact.
