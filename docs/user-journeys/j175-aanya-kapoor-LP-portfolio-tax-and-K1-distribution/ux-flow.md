---
doc_class: User-Journey-UX-Flow
journey_id: j175-aanya-kapoor-LP-portfolio-tax-and-K1-distribution
date: 2026-05-20
authority_tier: 2
status: draft
---

# j175 — UX flow: LP cockpit, per-fund capital account drilldown, tax-character categorization, state apportionment, GP-LP channel, K-1 PDF archive

Six primary surfaces:

- Aanya's LP cockpit (4-fund overview)
- Per-fund capital account drilldown (with reconciliation status)
- Tax-character categorization grid (ordinary + LTCG + STCG + qualified div + interest + 199A + foreign)
- State-by-state apportionment matrix (8 US states)
- GP-LP communication channel (per fund)
- Quarterly estimated tax payment + K-1 WORM archive

All screens preserve Hindi (Devanagari) + English + Tamil + Mandarin + Japanese + Indonesian byte-exact UTF-8 NFC. Per-fund attestation indicator + Section 199A + Section 1411 NIIT indicators always visible.

## Screen 1 — LP cockpit (May 20 19:48 PDT)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  LP COCKPIT · FY2026 K-1 Reconciliation · Aanya Kapoor                  │
├──────────────────────────────────────────────────────────────────────────┤
│  active tenant: aanya-kapoor-personal-2008 · personal · accredited       │
│  fy_tax_year: 2026 (calendar)                                            │
│  filing_deadline: 2027-06-15 (federal w/extension)                       │
│  cpa_filing_target: 2027-06-01 (Patricia's deadline)                     │
│  days_until_cpa_filing: 12                                               │
│                                                                          │
│  ┌─ 4 FUNDS · K-1 STATUS ──────────────────────────────────────────────┐ │
│  │  ✓ Andreessen Horowitz Fund VII LP                                   │ │
│  │    LP cap acct: $3.84M  · K-1 arrived 2027-04-14                     │ │
│  │  ✓ Sequoia Capital U.S. Growth Fund IX                               │ │
│  │    LP cap acct: $3.41M  · K-1 arrived 2027-04-22                     │ │
│  │  ✓ KKR Asian Fund V                                                  │ │
│  │    LP cap acct: $2.42M  · K-1 arrived 2027-05-08 (slightly late)    │ │
│  │  ✓ Insight Venture Partners XII LP                                   │ │
│  │    LP cap acct: $2.71M  · K-1 arrived 2027-05-12 (delayed; Q3 exit) │ │
│  │  ─                                                                   │ │
│  │  total committed: $14.2M  · total LP cap eod: $12.38M               │ │
│  │  total uncalled commitment: $1.82M                                  │ │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ QUARTERLY ESTIMATED TAX TRACKER ──────────────────────────────────┐  │
│  │  Q1 (Apr 15) ✓ paid                                                  │  │
│  │  Q2 (Jun 15) ○ to pay this weekend                                   │  │
│  │  Q3 (Sep 15) ○ pending (this cycle computes)                         │  │
│  │  Q4 (Jan 15 2028) ○ pending                                          │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  Cedar permit: personal_tenant + accredited_investor + qualified_purch  │
│  Audit class: EVT-J175-LP-COCKPIT-OPENED-Δ000                            │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 2 — Per-fund capital account drilldown (a16z example)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  CAPITAL ACCOUNT DRILLDOWN · Andreessen Horowitz Fund VII LP            │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─ FY2026 CAPITAL ACCOUNT ROLL ──────────────────────────────────────┐  │
│  │  opening_capital_account_2026:        $3,184,228                    │  │
│  │  + contributions_during_2026:          $0 (no capital call)         │  │
│  │  - distributions_during_2026:        -$282,184                      │  │
│  │  + ordinary_income_allocated:         $42,184                       │  │
│  │  + capital_gain_allocated_LT:         $148,228                      │  │
│  │  ─                                                                   │  │
│  │  closing_capital_account_2026:        $3,840,456                    │  │
│  │  ✓ reconciles with GP capital_account_statement_Δ4810               │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ K-1 LINE ITEMS (Schedule K-1 1065) ──────────────────────────────┐   │
│  │  Box 1 (Ordinary income):          $42,184                          │   │
│  │  Box 4a (Cap gain - LT):           $148,228                         │   │
│  │  Box 4b (Cap gain - ST):           $0                               │   │
│  │  Box 5 (Interest):                 $4,212                           │   │
│  │  Box 6a (Ord div):                 $0                               │   │
│  │  Box 6b (Qual div):                $8,184                           │   │
│  │  Box 16-A (Sec 199A):              $18,148                          │   │
│  │  Box 16-O (Foreign tax paid):      $968 (Canada $452 + UK $516)     │   │
│  │  Box 20 (Other):                   $0                               │   │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  [open k1 pdf]  [download K-3 international]  [send GP question]        │
│                                                                          │
│  Cedar permit: lp_capital_account_reconcile × accredited                 │
│  Audit class: EVT-J175-CAPITAL-ACCOUNT-RECONCILED-002                    │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 3 — Tax-character categorization grid (post-Thursday compute)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  TAX-CHARACTER CATEGORIZATION GRID · FY2026                             │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─ CHARACTER ────────────── a16z ── seq ── kkr ── ins ── total ──┐    │
│  │  ordinary_income           42,184  32,148  24,184  48,228  146,744 │    │
│  │  LTCG                      148,228 62,184  48,184  182,184 441,712 │    │
│  │  STCG (taxed as ord)            0  22,044       0       0   22,044 │    │
│  │  qualified_dividends         8,184   6,242      0       0   14,426 │    │
│  │  interest_income             4,212   2,418      0   3,148    9,778 │    │
│  │  Section 199A (QBI)         18,148      0      0  24,184   42,332 │    │
│  │  foreign-source (FTC)        4,022      0  34,148      0   38,170 │    │
│  │  ─                                                                  │    │
│  │  total                                                  $715,206  │    │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ SECTION 199A QBI ─────────────────────────────────────────────────┐  │
│  │  gross_199a_qbi: $42,332                                            │  │
│  │  20% gross deduction: $8,466                                        │  │
│  │  W-2/UBIA limitation phaseout: fully phased out (MFJ > $483,900)    │  │
│  │  effective deduction: $0                                            │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ SECTION 1411 NIIT (3.8%) ─────────────────────────────────────────┐  │
│  │  net_investment_income: $719,030                                    │  │
│  │  niit_base: $719,030                                                │  │
│  │  niit_owed: $27,323                                                 │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  Cedar permit: finops_portal.tax_character_categorize                    │
│  Audit class: EVT-J175-TAX-CHARACTER-003 + EVT-J175-SECTION-199A-004 +   │
│               EVT-J175-SECTION-1411-NIIT-005                             │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 4 — State-by-state apportionment matrix

```
┌──────────────────────────────────────────────────────────────────────────┐
│  STATE-BY-STATE APPORTIONMENT MATRIX                                    │
├──────────────────────────────────────────────────────────────────────────┤
│  residence: California (CA Form 540 resident)                            │
│                                                                          │
│  ┌─ PER-FUND × PER-STATE MATRIX ─────────────────────────────────────┐   │
│  │  state ── a16z ── seq ── kkr ── ins ── total ──                     │   │
│  │  CA      151,992  97,528   5,113  72,168  326,801                   │   │
│  │  NY       44,127  10,003   8,521  97,943  160,594                   │   │
│  │  MA       19,612   5,001   1,704  30,929   57,246                   │   │
│  │  TX        9,806   7,502   1,704  20,620   39,632                   │   │
│  │  WA       14,709   5,001       0       0   19,710                   │   │
│  │  CO        4,903       0       0  15,465   20,368                   │   │
│  │  TN            0       0       0  10,310   10,310                   │   │
│  │  FL            0       0   1,705  10,310   12,015                   │   │
│  │  foreign       0       0  88,074       0   88,074                   │   │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ CA TAX PROJECTION ────────────────────────────────────────────────┐  │
│  │  CA-resident pays tax on worldwide income (CA R&TC §17041)          │  │
│  │  K-1 total income: $715,206                                         │  │
│  │  CA tax at projected effective rate 12.3%: $87,970                  │  │
│  │  out-of-state credit (CA R&TC §18001-18006): $9,148                 │  │
│  │  net CA tax: $78,822                                                │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  Cedar permit: compliance.state_apportionment_compute                    │
│  Audit class: EVT-J175-STATE-APPORTIONMENT-006                           │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 5 — GP-LP communication channel (KKR Asian Fund V)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  GP-LP CHANNEL · KKR Asian Fund V · investor_relations                  │
├──────────────────────────────────────────────────────────────────────────┤
│  channel_class: gp_lp_quarterly_investor_relations                       │
│  permitted: aanya.kapoor + kkr-lp-relations + kerry-park-holt           │
│  mls_e2ee: active                                                        │
│                                                                          │
│  22:14 PDT  aanya: "Hi KKR LP team — quick clarification on my FY2026    │
│             K-1 Box 16-O Indonesia line: $4,808 foreign tax paid for    │
│             Indonesia stands out as higher than the country's share     │
│             of fund revenue would suggest..."                            │
│                                                                          │
│  09:42 PDT  kerry: "Hi Aanya — thanks for catching this. The Indonesia  │
│             $4,808 is correct + reflects a Indonesia-side withholding   │
│             tax accrual on a 2026 dividend distribution from one of      │
│             the portfolio companies (PT Bukit Asam, a coal company we    │
│             exited mid-2026; 15% under US-Indonesia DTAA). Form 1116   │
│             should accept this as Indonesia-source. Happy to send the   │
│             supporting workpaper. — Kerry"                              │
│                                                                          │
│  09:54 PDT  aanya: "Thanks Kerry — perfect, that explains it. Please    │
│             send the workpaper for my CPA's audit-trail."               │
│                                                                          │
│  10:08 PDT  [workpaper attached: kkr-asian-pt-bukit-asam-ftc-           │
│             workpaper-2026.pdf]                                         │
│                                                                          │
│  Cedar permit: connect.gp_lp_channel_send                                │
│  Audit class: EVT-J175-GP-LP-CLARIFICATION-Δ010-kkr-Δ001                 │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 6 — Quarterly estimated tax + WORM archival

```
┌──────────────────────────────────────────────────────────────────────────┐
│  QUARTERLY ESTIMATED TAX + WORM ARCHIVAL · FY2026                       │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─ Q2 PAYMENTS (May 23 11:42 PDT) ──────────────────────────────────┐  │
│  │  IRS:       $48,228   Treasury Direct Pay  ✓ ACK                   │  │
│  │  CA FTB:    $24,648   CA FTB Web Pay        ✓ ACK                   │  │
│  │  NY DTF:     $1,242   NY DTF                 ✓ ACK                   │  │
│  │  MA DOR:     $1,084   MA MassTaxConnect     ✓ ACK                   │  │
│  │  CO DOR:       $324   Revenue Online         ✓ ACK                   │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ Q3 + Q4 SCHEDULE ─────────────────────────────────────────────────┐  │
│  │  Q3 (Sep 15): IRS $84,184 + CA $32,148 + 7 others $2,184           │  │
│  │  Q4 (Jan 15 2028): IRS $84,232 + CA $30,140 + 7 others $1,668       │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ WORM ARCHIVAL (16 artifacts) ─────────────────────────────────────┐  │
│  │  cell: us-west-tier-1-worm-irs-retention                            │  │
│  │  retention: 7 years (IRS records retention rule)                    │  │
│  │  seal_class: irs-aligned-worm-class-1                               │  │
│  │  indelible_storage: ✓                                               │  │
│  │  time_stamp_authority: rfc-3161-tsa-2027                            │  │
│  │  artifacts:                                                         │  │
│  │     · 4 K-1 PDFs                                                    │  │
│  │     · 4 capital account statements                                  │  │
│  │     · 4 partner-allocation schedules                                │  │
│  │     · 4 foreign-tax-credit footnotes + workpapers                   │  │
│  │  per-artifact merkle leaf: 16                                       │  │
│  │  case merkle root: sha256:d5e8…9f12                                 │  │
│  │  external transparency log: external-tl-batch-2027-05-23           │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  Cedar permit: payments.estimated_tax_pay + drive.k1_pdf_worm_archive    │
│  Audit class: EVT-J175-ESTIMATED-TAX-PAID-009 +                          │
│               EVT-J175-WORM-ARCHIVED-011                                 │
└──────────────────────────────────────────────────────────────────────────┘
```

## Cross-screen rules

1. **Accredited investor attestation indicator**: visible on every LP-cockpit surface.
2. **Per-fund attestation indicator**: each fund displayed with K-1 arrival + reconciliation status.
3. **Tax-character grid**: aggregate matches per-K-1 line items.
4. **Per-state apportionment**: 8 US states + foreign-source row.
5. **GP-LP channel**: MLS E2EE; only Aanya + GP IR principals.
6. **Estimated tax tracker**: Q1-Q4 tracker visible on cockpit.
7. **WORM cell**: 7-year IRS-aligned retention.
8. **Cedar permit + audit class binding**: every screen has both.
9. **Language preservation**: byte-exact UTF-8 NFC across all languages.
10. **Section 199A + 1411 NIIT indicators**: visible on tax-character grid.

## Accessibility + i18n

- Screen reader: every estimated tax payment ack announced.
- Color: per-state matrix uses WCAG AA 4.5:1.
- Language picker: Hindi (Devanagari) + English + Tamil + Mandarin + Japanese + Indonesian + Hebrew (Aanya's own preference is English; CPA in English).
- Mobile: cockpit + GP-LP channel mobile-accessible; tax computations desktop-only.
