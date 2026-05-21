---
doc_class: User-Journey-Story
journey_id: j175-aanya-kapoor-LP-portfolio-tax-and-K1-distribution
date: 2026-05-20
authority_tier: 2
status: draft
---

# j175 — Story: Aanya Kapoor sits down with her 4 K-1s on Wednesday May 20 at 19:48 PDT

## §0 — Wednesday May 20, 2027, 19:48 PDT — Aanya's home, Noe Valley, San Francisco

Spring evening in San Francisco. 14°C and the famous SF fog rolling in over Twin Peaks. Aanya is at her dining-room table — a 1920s reclaimed-oak refectory table she and Vikram bought from a Berkeley estate sale in 2022. The kids are asleep (8-year-old Anika and 5-year-old Arjun; story time was 19:00, lights out 19:30). Vikram is in Tel Aviv. She has a glass of Chablis (Domaine Robert Marrec, '24 — she and Vikram order a case every year from K&L Wine).

She authenticates her work laptop (a 16" M4 MacBook Pro): passkey + face attestation + her **accredited investor + qualified purchaser** attestation token (issued by Wells Goldman & Associates her CPA firm 6 months ago after their annual KYC update).

The active-tenant pill: `aanya-kapoor-personal-2008 · personal · accredited_investor + qualified_purchaser`.

Her LP cockpit opens:

```
[LP COCKPIT] FY2026 K-1 Reconciliation · Aanya Kapoor
─
state:                          k1_arrived → ingestion_pending
fy_tax_year:                    2026 (calendar)
filing_deadline:                2027-06-15 (federal; with CPA extension)
cpa_filing_target:              2027-06-01 (CPA's working deadline)
days_until_cpa_filing:          12

funds_in_scope:                 4
  · Andreessen Horowitz Fund VII LP        $4.0M committed; LP capital account $3.84M
  · Sequoia Capital U.S. Growth Fund IX   $3.5M committed; LP capital account $3.41M
  · KKR Asian Fund V                       $3.5M committed; LP capital account $2.42M
  · Insight Venture Partners XII LP        $3.2M committed; LP capital account $2.71M
  ─
  total committed:                         $14.2M
  total LP capital current:                $12.38M
  total uncalled commitment:                $1.82M

k1_arrivals:
  · a16z Fund VII K-1:                       arrived 2027-04-14 (timely)
  · Sequoia U.S. Growth IX K-1:              arrived 2027-04-22 (timely)
  · KKR Asian Fund V K-1:                    arrived 2027-05-08 (slightly late; KKR's APAC fund typically files later due to portfolio complexity)
  · Insight Venture Partners XII K-1:        arrived 2027-05-12 (delayed; Insight had a major portfolio exit in Q3 2026 that required additional Section 199A work)

quarterly_estimated_tax_paid_ytd:
  · Q1 (April 15):  IRS $34,184 + CA $11,248 + 7 other states $3,148
  · Q2 (June 15):    [upcoming May 20 evening]
  · Q3 (Sept 15):    [pending — this cycle generates the Q3 estimate]
  · Q4 (Jan 15 2028): [pending — this cycle generates Q4 estimate]

cpa:                            patricia.wells-goldman@wells-goldman-cpa
```

`EVT-J175-LP-COCKPIT-OPENED-Δ000` sealed at 19:48:18 PDT.

She has a tea kettle next to her. She refills her Chablis. She has 12 days. Patricia is firm: drafts must be ready for review by June 1.

## §1 — May 20 19:48–22:18 PDT: K-1 PDF ingestion + capital account reconciliation

Aanya begins by ingesting all 4 K-1 PDFs into her LP cockpit. The finops-portal µservice has built-in K-1 parsers that read IRS Form 1065 Schedule K-1 directly:

```
[K-1 INGESTION] 19:48:42 PDT
─
k1_1_a16z_fund_vii:
  filename:                  a16z-fund-vii-k1-2026-aanya-kapoor-Δ4810.pdf
  size_bytes:                 1,142,028
  sha256:                     a1b3...ef21
  parse_state:                parsing → parsed
  parse_latency_seconds:      0.84
  schedule_k_1_line_items:   42 (lines 1–24 plus supplementary)
  audit_event_id:             EVT-J175-K1-INGESTED-Δ001a-a16z

k1_2_sequoia_us_growth_ix:
  filename:                  sequoia-us-growth-ix-k1-2026-aanya-kapoor-Δ4821.pdf
  size_bytes:                 982,184
  parse_state:                parsed
  parse_latency_seconds:      0.72
  audit_event_id:             EVT-J175-K1-INGESTED-Δ001b-sequoia

k1_3_kkr_asian_fund_v:
  filename:                  kkr-asian-fund-v-k1-2026-aanya-kapoor-Δ4842.pdf
  size_bytes:                 1,484,228
  parse_state:                parsed
  parse_latency_seconds:      1.18
  schedule_k_3_international_info_attached:  yes (foreign-source income disclosure)
  audit_event_id:             EVT-J175-K1-INGESTED-Δ001c-kkr

k1_4_insight_venture_xii:
  filename:                  insight-venture-xii-k1-2026-aanya-kapoor-Δ4855.pdf
  size_bytes:                 1,242,028
  parse_state:                parsed
  parse_latency_seconds:      0.92
  audit_event_id:             EVT-J175-K1-INGESTED-Δ001d-insight
```

`EVT-J175-K1-INGESTED-001` (composite) sealed at 19:50 PDT.

Then per-fund LP capital account reconciliation:

```
[CAPITAL ACCOUNT RECONCILIATION] 20:14 PDT
─
a16z_fund_vii:
  opening_capital_account_2026:        $3,184,228 (from FY2025 K-1)
  contributions_during_2026:            $0 (no capital call in 2026)
  cumulative_capital_contributed:       $3,200,000 (matches a16z's records)
  distributions_during_2026:            $282,184 (cash distributions)
  ordinary_income_allocated:            $42,184
  capital_gain_allocated:               $148,228
  closing_capital_account_2026:         $3,840,456
  ✓ reconciles_with_gp_records (a16z capital_account_statement_Δ4810)

sequoia_us_growth_ix:
  opening_capital_account_2026:        $3,184,128
  contributions_2026:                   $0
  cumulative_capital_contributed:        $2,800,000
  distributions_2026:                    $182,184
  ordinary_income_allocated:             $32,148
  capital_gain_allocated_LTCG:           $62,184
  capital_gain_allocated_STCG:           $22,044
  closing_capital_account_2026:          $3,410,184
  ✓ reconciles_with_gp_records (sequoia capital_account_statement_Δ4821)

kkr_asian_fund_v:
  opening_capital_account_2026:         $1,884,228
  contributions_2026:                    $400,000 (Q2 capital call)
  cumulative_capital_contributed:        $2,400,000
  distributions_2026:                    $84,128
  ordinary_income_allocated:             $24,184
  capital_gain_allocated_LTCG:           $48,184
  foreign_source_income_allocated:       $34,148
  closing_capital_account_2026:          $2,420,156
  ✓ reconciles_with_gp_records (kkr capital_account_statement_Δ4842)

insight_venture_xii:
  opening_capital_account_2026:         $2,200,128
  contributions_2026:                    $300,000 (Q1 capital call)
  cumulative_capital_contributed:        $2,600,000
  distributions_2026:                    $48,184
  ordinary_income_allocated:             $48,228
  capital_gain_allocated_LTCG:           $182,184
  section_199a_allocated:                $24,184
  closing_capital_account_2026:          $2,710,184
  ✓ reconciles_with_gp_records (insight capital_account_statement_Δ4855)

aggregate_lp_capital_account_eod_2026:   $12,380,980
total_committed_capital:                  $14,200,000
total_uncalled_remaining:                  $1,819,020
```

`EVT-J175-CAPITAL-ACCOUNT-RECONCILED-002` sealed at 20:14 PDT.

## §2 — May 20 22:18 PDT: send a clarification to KKR + Insight; pause for the evening

While reviewing KKR's K-1, Aanya finds Box 16-O (Foreign Tax Paid - country-by-country breakdown) shows:
- Singapore: $14,182 foreign tax paid
- India: $8,442 foreign tax paid
- Indonesia: $4,808 foreign tax paid (← question: this is much higher than expected for Aanya's fund share)
- Hong Kong: $2,824 foreign tax paid

The Indonesia number is unusual. She wants clarification before computing her Form 1116. She opens the GP-LP communication channel for KKR Asian Fund V:

```
[GP-LP CHANNEL] kkr-asian-fund-v · gp-lp-channel-aanya-kapoor
─
channel_class:                  gp_lp_quarterly_investor_relations
permitted_principals:           [
                                  aanya.kapoor@aanya-kapoor-personal-2008,
                                  kkr-asian-fund-v-investor-relations@kkr-asian-fund-v,
                                  lp-relations-kerry-park-holt@kkr-asian-fund-v
                                ]
mls_e2ee:                       active
audit_class:                    investor_relations
```

Aanya's message (English; she sends a brief polite query):

> "Hi KKR LP team — quick clarification on my FY2026 K-1 Box 16-O Indonesia line: $4,808 foreign tax paid for Indonesia stands out as higher than the country's share of fund revenue would suggest. Can you confirm this is correct and explain the Indonesia-side tax accrual? Asking for Form 1116 preparation. Thanks — Aanya"

`EVT-J175-GP-LP-CLARIFICATION-Δ010-kkr-Δ001` sealed at 22:14 PDT.

Similarly to Insight Venture Partners XII on the Section 199A:

> "Hi Insight LP team — my K-1 Box 20-Z Section 199A pass-through shows $24,184 from your Q3 portfolio company exit. Can you confirm which portfolio company drove the QBI pass-through + which trade-or-business UBI test applies for my Section 199A worksheet? My W-2 income is above the phaseout threshold so the deduction will phase out for me but I still need to compute the gross before phaseout. Thanks — Aanya"

`EVT-J175-GP-LP-CLARIFICATION-Δ010-insight-Δ001` sealed at 22:16 PDT.

Aanya closes her laptop at 22:18 PDT. She'll resume Thursday evening.

## §3 — Thursday May 21 19:42–22:42 PDT: tax-character categorization + Section 199A + NIIT

Thursday evening. Aanya reads emails first. KKR LP relations (Kerry Park-Holt) replied at 09:42 PDT today:

> "Hi Aanya — thanks for catching this. The Indonesia $4,808 is correct + reflects a Indonesia-side withholding tax accrual on a 2026 dividend distribution from one of the portfolio companies (PT Bukit Asam, a coal company we exited mid-2026; the Indonesia-side dividend withholding is 15% under the US-Indonesia DTAA). The accrual flows through to your K-1 because we allocate FTC based on each LP's share of the portfolio company. Form 1116 should accept this as Indonesia-source. Happy to send the supporting workpaper. — Kerry"

Aanya replies: "Thanks Kerry — perfect, that explains it. Please send the workpaper for my CPA's audit-trail."

The workpaper arrives 14 minutes later. Aanya archives it to the K-1 drive room.

Insight LP relations (Anil Subramaniam-Reid) also replied:

> "Hi Aanya — the Section 199A pass-through came from the Insight Venture Partners XII portfolio company FleetSmart Inc., which Insight exited in Q3 2026 (acquired by Bell Industries for $1.4B). FleetSmart was a Trade or Business operating as a fleet-management SaaS; its QBI flow-through to LPs was computed under IRC § 199A(b)(2) using the SaaS-as-active-trade-or-business framework. The supporting workpaper is in your portfolio drive room. — Anil"

Aanya replies: "Thanks Anil — clear. I'll proceed."

`EVT-J175-GP-LP-CLARIFICATIONS-010` (composite) sealed at 19:48 PDT Thursday.

Now Aanya runs the **tax-character categorization**:

```
[TAX CHARACTER CATEGORIZATION] FY2026 · 21:14 PDT Thursday
─
ordinary_income_aggregate:                  $146,744
  · a16z:                                    $42,184
  · sequoia:                                  $32,148
  · kkr:                                     $24,184
  · insight:                                  $48,228

ltcg_aggregate:                              $441,712
  · a16z:                                    $148,228 (LT)
  · sequoia:                                 $62,184 (LT)
  · kkr:                                     $48,184 (LT)
  · insight:                                 $182,184 (LT)
  · sequoia_short_term_separate:             $22,044 (ST) → treated as ordinary

stcg_aggregate (taxed_as_ordinary):          $22,044
  · sequoia:                                  $22,044

qualified_dividends_aggregate:                $14,426
  · a16z:                                     $8,184
  · sequoia:                                   $6,242

interest_income_aggregate:                    $9,778
  · a16z:                                      $4,212
  · sequoia:                                    $2,418
  · insight:                                    $3,148

section_199a_aggregate:                       $42,332
  · a16z:                                      $18,148
  · insight:                                   $24,184

foreign_source_income_aggregate:              $38,170
  · a16z:                                      $4,022 (Canada + UK)
  · kkr:                                       $34,148 (SG + IN + ID + HK)

grand_total_k1_income:                        $715,206

note: aggregate matches per-K-1 line items reconciled with capital account.
```

`EVT-J175-TAX-CHARACTER-003` sealed at 21:14 PDT.

Section 199A QBI deduction:

```
[SECTION 199A QBI] 21:42 PDT
─
qbi_aggregate:                                $42,332
gross_199a_deduction_at_20pct:                $8,466
aanya_w2_income_2026:                         ~$892,000 (McKinsey SVP salary + RSUs)
aanya_taxable_income_2026_projected:           ~$1,484,228 (W-2 + K-1 + Vikram K-1 + other)
filing_status:                                 married_filing_jointly
2026_phaseout_thresholds_mfj:                  $383,900 - $483,900
phaseout_state:                                fully_phased_out (income > $483,900)
effective_199a_deduction_after_phaseout:      $0 (W-2/UBIA limitation kicks in fully)
note: 199A computed for completeness + documented; deduction = $0 due to W-2 wages limitation
```

`EVT-J175-SECTION-199A-COMPUTED-004` sealed at 21:42 PDT.

Section 1411 NIIT:

```
[SECTION 1411 NIIT] 22:14 PDT
─
net_investment_income_aggregate:              $715,206 (K-1 investment income)
  + dividend_income_McKinsey_RSU_dividends:    $2,184
  + interest_savings_+_bond:                    $4,128
  - investment_expenses:                       -$2,488
  net_investment_income:                       $719,030

magi_threshold_mfj:                           $250,000
magi_projection_2026:                          ~$1,488,228 (filing jointly)
niit_base:                                     min(net_investment_income, magi - threshold)
                                                = min($719,030, $1,238,228)
                                                = $719,030
niit_rate:                                     3.8%
niit_owed:                                     $27,323
```

`EVT-J175-SECTION-1411-NIIT-005` sealed at 22:14 PDT.

Aanya closes for the evening at 22:42 PDT.

## §4 — Friday May 22 19:14–21:48 PDT: state-by-state apportionment + foreign tax credit

Friday evening. Aanya tackles state-by-state apportionment.

```
[STATE-BY-STATE APPORTIONMENT] FY2026 · 19:42 PDT Friday
─
aanya_residence_state:                        California (Noe Valley, SF; CA Form 540)
primary_filing_state:                          California (residence)

source-rule analysis (per fund per K-1 Schedule K-3):
  a16z_fund_vii (SF-based; investments diverse):
    Sched_K-3_state_breakdown:
      CA-source: 62% (predominant; many CA portfolio companies)
      NY-source: 18%
      MA-source: 8%
      TX-source: 4%
      WA-source: 6%
      CO-source: 2%
    ca_source_share_of_a16z_k1: 62% × $245,148 = $151,992

  sequoia_us_growth_ix (Menlo Park):
    Sched_K-3_state_breakdown:
      CA-source: 78% (very CA-heavy; Sequoia's typical pattern)
      NY-source: 8%
      MA-source: 4%
      TX-source: 6%
      WA-source: 4%
    ca_source_share_of_sequoia_k1: 78% × $125,036 = $97,528

  kkr_asian_fund_v (NY-based; mostly APAC portfolio):
    Sched_K-3_state_breakdown_us:
      US-source: 16% (small; mostly Asia)
        US-state-breakdown:
          NY-source: 50%
          CA-source: 30%
          TX-source: 10%
          FL-source: 10%
      Asia-source: 84% (foreign)
    ca_source_share_of_kkr_k1: 30% × 16% × $106,516 = $5,113

  insight_venture_xii (NY-based; diverse US portfolio):
    Sched_K-3_state_breakdown:
      NY-source: 38%
      CA-source: 28%
      MA-source: 12%
      TX-source: 8%
      CO-source: 6%
      TN-source: 4%
      FL-source: 4%
    ca_source_share_of_insight_k1: 28% × $257,744 = $72,168

aanya_total_ca_source_income_from_lp:          $326,801
aanya_total_ny_source_income_from_lp:          $66,478
aanya_total_ma_source_income_from_lp:          $51,818
aanya_total_tx_source_income_from_lp:          $32,118
aanya_total_wa_source_income_from_lp:          $19,712
aanya_total_co_source_income_from_lp:          $19,612
aanya_total_tn_source_income_from_lp:          $10,310
aanya_total_fl_source_income_from_lp:          $14,478

ca_filing_status:                              resident; pay tax on worldwide income
out_of_state_credit:                           California allows credit for other-state tax paid (CA R&TC § 18001-18006)
projected_ca_state_tax_on_k1:                  $326,801 × 12.3% effective rate ≈ $40,196
projected_other_state_tax_aggregate:           $9,148 (sum of small out-of-state tax payments)
net_ca_tax_after_other_state_credit:          $40,196 - $9,148 = $31,048
```

`EVT-J175-STATE-APPORTIONMENT-006` sealed at 20:42 PDT Friday.

Foreign tax credit (Form 1116):

```
[FORM 1116 FOREIGN TAX CREDIT] 21:14 PDT Friday
─
foreign_taxes_paid_aggregate:                 $34,968
  · a16z (CA + UK):                            $968
  · kkr (SG + IN + ID + HK):                   $30,256
  · other_foreign:                              $3,744

ftc_basket_passive:                           $4,824 (creditable per Form 1116 passive basket)
ftc_basket_general:                           $0 (no general basket income)
ftc_overall_limitation:                       (foreign_source_taxable_income / total_taxable_income) × us_tax
ftc_carryover_from_prior_year:                $0
ftc_creditable_this_year:                     $4,824
ftc_unused_carryforward_to_2027:               $30,144 (most foreign tax unable to be credited this year)
```

`EVT-J175-FOREIGN-TAX-CREDIT-007` sealed at 21:42 PDT.

AMT projection:

```
[AMT PROJECTION] 21:48 PDT
─
amti (alternative minimum taxable income):    $1,448,228 (post-W-2 + K-1 + AMT prefs)
amt_exemption_mfj_2026:                       $133,300
amt_phaseout_completion_2026_mfj:             starting $1,218,700
exemption_after_phaseout:                      $0 (above phaseout completion)
amti_minus_exemption:                          $1,448,228
amt_tentative_28pct:                          $405,503 (above $221,200 threshold; mix of rates)
regular_tax_before_amt:                       $432,184 (federal regular)
amt_owed:                                      $0 (regular tax exceeds tentative AMT)
amt_zone:                                      no
```

`EVT-J175-AMT-COMPUTED-008` sealed at 21:48 PDT.

## §5 — Saturday May 23 10:18 PDT: Q3 + Q4 estimated tax payments + WORM archival

Saturday morning. Aanya at the same table; coffee instead of wine. Q3 + Q4 estimated tax computations:

```
[QUARTERLY ESTIMATED TAX PAYMENTS] 10:48 PDT Saturday
─
total_estimated_2026_federal_tax_liability:    $284,228 (full year projection)
total_estimated_2026_ca_state_tax_liability:    $98,184
total_estimated_2026_other_states_aggregate:   $9,148

q1_2026_paid:                                  $34,184 + CA $11,248 + 7 other states $3,148
q2_2026_to_pay_today:                          $48,228 + CA $24,648 + 7 other states $2,148
q3_2026_to_pay_sept_15:                         $84,184 + CA $32,148 + 7 other states $2,184
q4_2026_to_pay_jan_15_2028:                    $84,232 + CA $30,140 + 7 other states $1,668

safe_harbor_check:
  prior_year_110pct_total_tax_safe_harbor:    not applicable (Aanya's prior-year AGI > $150K)
  current_year_90pct_safe_harbor:              applicable
  total_q1+q2+q3+q4_paid_target:                $250,828 + CA + states
  underpayment_penalty_risk:                    none (current paydown matches projection)

payments_to_make_this_weekend:
  q2_irs:                                      $48,228 (due June 15 but Aanya paying early)
  q2_ca:                                       $24,648
  q2_ny:                                       $1,242
  q2_ma:                                       $1,084
  q2_tx:                                        $0 (no income tax)
  q2_wa:                                        $0 (no income tax)
  q2_co:                                        $324
  q2_tn:                                        $0 (no income tax)
  q2_fl:                                        $0 (no income tax)
```

Sanctions screening + payments:

```
[SANCTIONS SCREENING] 11:14 PDT
─
recipients: IRS + CA FTB + NY DTF + MA DOR + CO DOR (all government entities; clean by definition)
total_hits: 0
status: clean
```

```
[ESTIMATED TAX PAYMENT EXECUTION] 11:42 PDT Saturday
─
payment_method:    ACH (Direct Pay for IRS; state-specific portals for state)
payments:
  · IRS:        $48,228 → Treasury Direct Pay
  · CA FTB:     $24,648 → CA FTB Web Pay
  · NY DTF:     $1,242 → NY DTF online services
  · MA DOR:     $1,084 → MA DOR MassTaxConnect
  · CO DOR:     $324  → CO Revenue Online

all_payments_dispatched:    true
all_payments_acked:         true
hlc_payment_timestamp:      hlc:2027-05-23T18:42:00Z:Δ0080
audit_event_id:              EVT-J175-ESTIMATED-TAX-PAID-009
```

`EVT-J175-ESTIMATED-TAX-PAID-009` sealed at 11:42 PDT Saturday.

Then WORM archival of all 16 artifacts:

```
[WORM ARCHIVAL] 14:18 PDT Saturday
─
worm_cell:                            us-west-tier-1-worm-irs-retention
retention_class:                       irs_records_retention_7y
seal_class:                            irs-aligned-worm-class-1
indelible_storage_attestation:        true
time_stamp_authority:                  rfc-3161-tsa-2027

artifacts_sealed (16):
  · 4 K-1 PDFs (a16z + sequoia + kkr + insight)
  · 4 capital account statements (one per fund GP)
  · 4 partner-allocation schedules (one per fund GP)
  · 4 foreign-tax-credit footnotes + workpapers (KKR Indonesia workpaper + Insight FleetSmart workpaper + Sequoia + a16z FTC documentation)

per_artifact_merkle_leaf:              16 leaves
case_merkle_root:                       sha256:d5e8...9f12
external_transparency_log_batch:       external-transparency-log-batch-2027-05-23
proof_class:                           inclusion_proof
ifrs_irs_attestation:                  enabled
```

`EVT-J175-WORM-ARCHIVED-011` sealed at 14:18 PDT Saturday.

## §6 — Sunday May 24 19:18–21:18 PDT: CPA submission + pack manifest finalization

Sunday evening. Aanya packages everything for her CPA Patricia Wells-Goldman.

```
[CPA PACKAGE] 20:42 PDT Sunday
─
package_id:                            cpa-package-aanya-kapoor-fy2026-2027-05-24
recipient:                              patricia.wells-goldman@wells-goldman-cpa
contents:
  · 4 K-1 PDFs + reconciliation reports
  · capital account aggregate summary ($12.38M)
  · tax-character categorization summary ($715K)
  · Section 199A + 1411 NIIT projections
  · 8-state apportionment matrix
  · Form 1116 FTC computation
  · AMT projection
  · Q1 + Q2 estimated tax payments confirmation
  · Q3 + Q4 estimated tax schedule
  · 2 GP-LP clarification dialogues + supporting workpapers
  · WORM archival receipt + Merkle root attestation
delivery_method:                       drive_shared_with_cpa_tenant (read-only access)
delivery_at:                            2027-05-24T20:42:18-07:00
cpa_acknowledgment_expected:           2027-05-26 (within 2 business days)
```

`EVT-J175-CPA-PACKAGE-SENT-Δ012a` sealed at 20:42 PDT Sunday.

Pack manifest assertion:

```
[PACK MANIFEST] 21:00 PDT
─
active_packs:                          10
  · pack-irs-schedule-k-1-1065-v3
  · pack-irs-section-199a-qbi-2026
  · pack-irs-section-1411-niit-2026
  · pack-state-tax-apportionment-multi-2026
  · pack-irc-section-754-step-up
  · pack-amt-2026
  · pack-foreign-tax-credit-form-1116-2026
  · pack-eu-aifmd-non-eu-fund-marketing
  · pack-uk-nppr-non-eu-fund
  · pack-accredited-investor-reg-501-rule-144a
cross_validation_state:                passed
pack_manifest_signature:               sha256:e8f4...a921
```

`EVT-J175-PACK-MANIFEST-Δ012b` sealed at 21:00 PDT.

Aanya closes the LP cockpit at 21:18 PDT Sunday. She pours one more Chablis. Vikram is back Thursday. The CPA has the package. K-1 reconciliation done.

## §7 — Stop condition

All 12 AC pass on the seeded fixture; the 4 K-1s are reconciled with capital account statements; tax-character is categorized ($715K); state apportionment is computed for 8 US states; foreign tax credit is computed for 6 jurisdictions; Section 199A + 1411 NIIT + AMT are computed; Q3 + Q4 quarterly estimated tax payments are made to IRS + state revenue departments; the 4 GP-LP clarification dialogues are resolved (2 substantive + 2 informational); the 16 artifacts are WORM-archived with 7-year retention; the CPA package is delivered. Hindi + English + Tamil + Mandarin + Japanese + Indonesian + diacritics UTF-8 NFC byte-exact.
