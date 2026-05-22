---
doc_class: User-Journey-UX-Flow
journey_id: j166-cso-mira-goldberg-strategic-acquisition-go-no-go
date: 2026-05-20
authority_tier: 2
status: draft
---

# j166 — UX flow: M&A console, NDA channel, ML scenarios, pack cross-check, board vote

Five primary surfaces:

- Mira's M&A console (focused workspace; right monitor)
- NDA-bound cross-tenant channel inbox (with sender-tenant boundary indicators)
- Financial-planning M&A model canvas (3-price-point scenarios side-by-side)
- ML scenario explorer (Monte-Carlo + cohort churn + integration cost)
- Pack-manifest cross-check matrix (acquirer × target)
- Board go/no-go vote screen with passkey-per-member roll call

All screens preserve Hebrew + German + Hangul + diacritics UTF-8 NFC byte-exact.

## Screen 1 — M&A workspace (May 15 07:42 EDT)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  M&A WORKSPACE · Skylark Logistics · CSO Mira Goldberg                   │
├──────────────────────────────────────────────────────────────────────────┤
│  active deal: MRT Acquisition Q2-2027                                    │
│  state: due_diligence (week 9 of 9)                                      │
│  board decision target: 2027-05-25T09:00 EDT (7 business days)           │
│                                                                          │
│  ┌─ DEAL TERMS ────────────────────────────────────────────────────────┐ │
│  │  target: Mendelsohn Routing Technologies (MRT)                      │ │
│  │  working price: $186M (range $172M–$202M)                           │ │
│  │  structure: 60% cash + 40% Skylark stock                            │ │
│  │  earnout: $30M (Bjorn Mendelsohn 24-month vest)                     │ │
│  │  closing target: Q3-2027                                            │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─ PIPELINE ──────────────────────────────────────────────────────────┐ │
│  │   ✓ Cross-tenant NDA channel open                                   │ │
│  │   ○ Diligence document inbox (4 new today)                          │ │
│  │   ○ Financial model 3-price-point compute                           │ │
│  │   ○ ML scenario modeling                                            │ │
│  │   ○ Pack manifest cross-check                                       │ │
│  │   ○ Merger filing requirements compute                              │ │
│  │   ○ Counsel review                                                  │ │
│  │   ○ CFO sign-off                                                    │ │
│  │   ○ Strategy + Audit committee endorsement                          │ │
│  │   ○ Board go/no-go vote                                             │ │
│  │   ○ Decision record + Merkle anchor                                 │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 2 — Cross-tenant NDA channel inbox (May 15 07:42 EDT)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  CROSS-TENANT CHANNEL · skylark-mrt-2027-q2                              │
│  ⚠ NDA-BOUND · payload classes restricted · return-or-destroy active     │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  NDA: nda-skylark-mrt-2027-03-08 · ACTIVE (expires 2027-12-31)           │
│  return-or-destroy deadline if deal terminates: 2027-09-30                │
│                                                                          │
│  ┌─ NEW DOCUMENTS (4) ─────────────────────────────────────────────────┐ │
│  │                                                                     │ │
│  │  📄 mrt-q1-2027-cohort-churn-anonymized.csv         1.2 MB · 02:14  │ │
│  │     class: diligence_response_anonymized                            │ │
│  │     ✓ PII scan: 0 hits   ✓ size: 1.2/50 MB                          │ │
│  │     ✓ e2ee envelope intact   ✓ sender authorized                    │ │
│  │     from: bjorn.mendelsohn@MRT (Berlin)                             │ │
│  │     [open] [archive to drive]                                       │ │
│  │                                                                     │ │
│  │  📄 mrt-2026-customer-concentration-named.pdf       4.8 MB · 02:48  │ │
│  │     class: diligence_response_named                                 │ │
│  │     ✓ PII scan: 0 hits   ✓ NDA scope authorized                     │ │
│  │     [open] [archive]                                                │ │
│  │                                                                     │ │
│  │  📄 mrt-integration-architecture-overview-v2.pdf    8.4 MB · 04:22  │ │
│  │     class: diligence_response_anonymized                            │ │
│  │     [open] [archive]                                                │ │
│  │                                                                     │ │
│  │  📄 mrt-data-residency-attestation-eu.pdf            612 KB · 06:18 │ │
│  │     class: regulatory_filing_input                                  │ │
│  │     [open] [archive]                                                │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  cumulative documents (channel lifetime): 27 from MRT + 14 from Skylark │
│  next return-or-destroy review: 2027-06-15                              │
└──────────────────────────────────────────────────────────────────────────┘
```

UX notes:

- NDA-bound warning is foregrounded in the channel header — Mira always sees the boundary.
- Each document shows payload class + PII scan + e2ee envelope status + sender tenant authorization.
- Cross-tenant boundary visually distinct (different color band on the channel header).
- Return-or-destroy deadline visible — operational reminder.

## Screen 3 — Financial model M&A canvas (May 15 14:42 EDT)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  M&A FINANCIAL MODEL · MRT · 3-price-point scenario                       │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─ TARGET INPUTS ─────────────────────────────────────────────────────┐ │
│  │  ARR (TTM)              $42.0M                                      │ │
│  │  ARR growth YoY            31%                                      │ │
│  │  gross margin              78%                                      │ │
│  │  customer count            340                                      │ │
│  │  avg ARR per customer   $123,500                                    │ │
│  │  CAC (LTM)              $48,000                                     │ │
│  │  LTV/CAC                   4.2x                                     │ │
│  │  NDR                       117%                                     │ │
│  │  customer concentration   top-10 = 31% ARR                          │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─ SCENARIOS ─────────────────────────────────────────────────────────┐ │
│  │                                                                     │ │
│  │  metric          $172M       $186M       $202M                      │ │
│  │  ────                                                               │ │
│  │  rev multiple      4.1x        4.4x        4.8x                     │ │
│  │  NTM accretive  18 mo       23 mo       34 mo                       │ │
│  │  year-3 IRR        22%         18%         12%   ⚠ < 20% threshold  │ │
│  │  year-5 IRR        27%         22%         16%                      │ │
│  │  Y3 cash impact  -$48M      -$62M       -$78M                       │ │
│  │  Y5 cash impact +$104M      +$84M       +$54M                       │ │
│  │  dilution         4.2%        4.6%        5.0%                      │ │
│  │                                                                     │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ⚠ At $186M working price the year-3 IRR (18%) is below the board's     │
│    20% minimum yield criterion. Either negotiate down toward             │
│    $176-180M, or strategic case must justify sub-threshold IRR.          │
│                                                                          │
│  [save] [share with CFO] [advance to ML scenarios]                      │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 4 — ML scenario explorer (May 18 06:54 EDT)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  ML SCENARIO EXPLORER · MRT Acquisition · Monte-Carlo + Cohort + Integ.  │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  MODEL PROVENANCE (EU AI Act Article 50 declaration)                     │
│   - monte-carlo-mid-market-saas-v7@oyatie-2027-02 · generative AI        │
│   - cohort-churn-forecast-saas-v5@oyatie-2027-04 · generative AI         │
│   - integration-cost-forecast-cross-stack-v3@oyatie-2027-01              │
│                                                                          │
│  ┌─ MONTE-CARLO 10K × 3 MACRO SCENARIOS @ $186M ──────────────────────┐  │
│  │                                                                    │  │
│  │   Recession (P10):                                                  │  │
│  │     ARR @ Y5:  $58M    IRR Y5: 8%   neg CF Y3+: 18%                │  │
│  │                                                                    │  │
│  │   Neutral (P50):                                                    │  │
│  │     ARR @ Y5:  $98M    IRR Y5: 24%  neg CF Y3+: 3%                 │  │
│  │                                                                    │  │
│  │   Tailwind (P90):                                                   │  │
│  │     ARR @ Y5: $148M    IRR Y5: 38%  neg CF Y3+: 0%                 │  │
│  │                                                                    │  │
│  │   ─────                                                             │  │
│  │   Probability-weighted:                                             │  │
│  │     ARR @ Y5:  $94M    IRR Y5: 23%  ← ABOVE 20% THRESHOLD ✓       │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ COHORT CHURN FORECAST · 5-YEAR HORIZON ──────────────────────────┐   │
│  │  Year   Gross churn   Net dollar retention                         │   │
│  │   1        6.2%         88%                                        │   │
│  │   2        6.8%         87%                                        │   │
│  │   3        7.1%         86%                                        │   │
│  │   4        7.4%         86%                                        │   │
│  │   5        7.6%         85%                                        │   │
│  │  (NDR mean-reverts from MRT's current 117% toward 85%)             │   │
│  └────────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  ┌─ INTEGRATION COST FORECAST ───────────────────────────────────────┐   │
│  │  point estimate: $14.2M                                            │   │
│  │  95% CI: $9.4M – $19.0M                                            │   │
│  │  primary drivers:                                                   │   │
│  │   • Postgres 14→16 migration + Skylark shard alignment              │   │
│  │   • Identity unification (Auth0 → oyatie identity)                  │   │
│  │   • Route-optimization engine integration (4–6 senior eng × 9 mo)   │   │
│  └────────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  [reproduce with new seed] [export to model] [archive to deal package]  │
└──────────────────────────────────────────────────────────────────────────┘
```

UX notes:

- EU AI Act Article 50 declaration is foregrounded with all 3 model identities + types.
- Probability-weighted IRR is highlighted (23% vs 20% threshold).
- Reproduce-with-new-seed button — explicit determinism affordance.

## Screen 5 — Pack-manifest cross-check matrix (May 18 16:42 EDT)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  PACK MANIFEST CROSS-CHECK · skylark + mrt                                │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──── matrix ─────────────────────────────────────────────────────────┐ │
│  │  Pack class            Skylark        MRT               Status      │ │
│  │  ────                                                               │ │
│  │  SOC 2 Type 2          ✓ (PwC)        — (Type 1 only)  ⚠ Type1→2   │ │
│  │  GDPR controller       ✓              ✓                 ✓ aligned   │ │
│  │  ISO 27001              ✓              ✓                 ✓ aligned   │ │
│  │  CCPA                   ✓              —                 acquirer only│ │
│  │  HIPAA BA              ✓ (8 BAAs)     —                 acquirer only│ │
│  │  PCI DSS SAQ-C          ✓              —                 acquirer only│ │
│  │  German BDSG            —              ✓                 subsumed by GDPR│ │
│  │  TISAX/VDA              —              ✓                 ✓ additive (automotive)│ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌──── blockers ──────────────────────────────────────────────────────┐ │
│  │  ⚠ SOC 2 Type 1 vs Type 2                                          │ │
│  │     MRT post-close must upgrade to Type 2 within 18 months          │ │
│  │     Remediation cost: $480K                                         │ │
│  │     Owner (post-close): MRT engineering + Skylark CCO joint plan    │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌──── strategic positives ──────────────────────────────────────────┐  │
│  │  ✓ TISAX/VDA — opens automotive vertical                           │  │
│  │  ✓ ISO 27001 alignment — no friction                               │  │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  Compatibility score: 84% (HIGH)                                         │
│                                                                          │
│  [save to deal package]                                                  │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 6 — Merger filings dashboard (May 18 17:32 EDT)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  MERGER FILING REQUIREMENTS · MRT acquisition                             │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─ HSR (US) ─────────────────────────────────────────────────────────┐ │
│  │  threshold:   $111.4M (2027)                                        │ │
│  │  deal_size:  $186M                                                  │ │
│  │  ✓ REQUIRED                                                          │ │
│  │  filing_fee: $280,000 (size class $161.5M–$268.5M)                   │ │
│  │  waiting:    30 days                                                │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─ EU MERGER CONTROL (REG 139/2004) ──────────────────────────────────┐ │
│  │  threshold: global > €5B AND community > €250M (each party)         │ │
│  │  parties below threshold                                            │ │
│  │  ✗ EU-level NOT required                                            │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─ GERMAN BWB (NATIONAL ALTERNATIVE TO EU-MR) ───────────────────────┐ │
│  │  threshold: €50M domestic AND €17.5M-€500K secondary                │ │
│  │  parties cross threshold                                            │ │
│  │  ✓ REQUIRED                                                          │ │
│  │  filing_window: 1 month                                             │ │
│  │  recommended counsel: Hengeler Mueller (Frankfurt)                  │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─ UK CMA ───────────────────────────────────────────────────────────┐ │
│  │  threshold: UK turnover > £70M OR 25% share of supply               │ │
│  │  MRT UK turnover: £3.4M (below threshold)                           │ │
│  │  ✗ Mandatory NOT required                                            │ │
│  │  ⓘ Voluntary notification recommended                                │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─ ISRAELI IMC ──────────────────────────────────────────────────────┐ │
│  │  threshold: NIS 360M combined + each > NIS 20M                      │ │
│  │  parties below threshold                                            │ │
│  │  ✗ NOT required                                                      │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  Total filings required: HSR US + German BWB                             │
│  Voluntary: UK CMA                                                       │
│  Estimated clearance window: 30–45 days post-signing                     │
│                                                                          │
│  [save to deal package]                                                  │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 7 — Board go/no-go vote screen (May 25 09:00 EDT)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  BOARD GO/NO-GO VOTE · MRT Acquisition · 2027-05-25 09:00 EDT             │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Resolution:                                                             │
│   Resolved, that the Board approves the strategic acquisition of         │
│   Mendelsohn Routing Technologies, Inc. for total consideration up to    │
│   $186 million (60% cash, 40% Skylark common stock) plus contingent      │
│   earnout of $30 million subject to the team-retention provisions        │
│   recommended by Counsel review R1, with closing subject to HSR + German │
│   BWB clearance.                                                          │
│                                                                          │
│  Pre-conditions satisfied (Cedar guard):                                 │
│   ✓ audit_committee_endorsement_present (3 of 5)                         │
│   ✓ strategy_committee_endorsement_present (4 of 5)                      │
│   ✓ counsel_review_present (Daphne Harrowgate, 4 redlines resolved)      │
│   ✓ financial_model_signoff_cfo_present (Reginald Otis)                  │
│   ✓ ML scenarios reproducible + provenance preserved                     │
│   ✓ pack_manifest_cross_check_no_blocker (84% compatibility)             │
│   ✓ merger_filings_path_clear (HSR + BWB)                                │
│                                                                          │
│  Vote roll call:                                                         │
│                                                                          │
│   Adrian Cheng-Whitford (CEO, chair)               passkey ✓  YES        │
│   Hannah Beauregard (audit chair)                  passkey ✓  YES        │
│   Margarita Velasco-Heim (strategy chair)          passkey ✓  YES        │
│   Kenji Park-Holloway (independent)                passkey ✓  ABSTAIN    │
│   Christine Adebayo-Lin (independent)              passkey ✓  YES        │
│   Anil Subramaniam (independent)                   passkey ✓  YES        │
│   David Hofmann-Reyes (independent)                passkey ✓  YES        │
│   Joon-Ho Park (independent · 박준호)              passkey ✓  YES        │
│   Patricia Wells-Okonkwo (NED)                     passkey ✓  NO         │
│                                                                          │
│   YES: 7   NO: 1   ABSTAIN: 1                                            │
│   Threshold: 5 of 9 (simple majority)                                    │
│   Result:    GO ✓                                                        │
│                                                                          │
│  [record decision + anchor to audit chain]                               │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 8 — Decision record + Merkle anchor confirmation (May 25 11:18 EDT)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  DECISION RECORD · MRT Acquisition · GO                                   │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ✓ Board voted GO (7 yes / 1 no / 1 abstain) at 10:54:18 EDT             │
│  ✓ Decision recorded to drive WORM `skylark/board/2027/q2/mrt-acq/`      │
│  ✓ 7-year retention timer engaged (until 2034-05-25)                     │
│  ✓ Super-Merkle root computed:                                           │
│    0xc2f8a4b7e1d6f3a9c4e7b2d5f8a1c6e9b3d7f0a4c8e2b5d9f3a7c0e4b8d2f6a1    │
│  ✓ Anchored to audit-chain-spine-skylark-m-a-2027-q2                     │
│  ✓ Anchored to external-transparency-log-batch-2027-05-25T1118           │
│                                                                          │
│  Bundle components archived (10):                                        │
│   1. executive_summary                                                   │
│   2. financial_model                                                     │
│   3. ml_scenarios                                                        │
│   4. pack_cross_check                                                    │
│   5. merger_filings                                                      │
│   6. counsel_review                                                      │
│   7. cfo_signoff                                                         │
│   8. committee_endorsement                                               │
│   9. board_vote_roll_call                                                │
│  10. integration_playbook                                                │
│                                                                          │
│  NDA-bound diligence documents NOT in the bundle (held under MRT's      │
│  rights; return-or-destroy obligation runs through 2027-09-30 if deal   │
│  terminates pre-close).                                                  │
│                                                                          │
│  Next actions:                                                           │
│   • Counsel retains Cooley LLP + Hengeler Mueller (Frankfurt)            │
│   • Signing target: 2027-05-30                                          │
│   • HSR filing within 2 days of signing                                 │
│   • German BWB filing within 5 days of signing                          │
│   • Target close: ~2027-07-14                                            │
│                                                                          │
│  [done]                                                                  │
└──────────────────────────────────────────────────────────────────────────┘
```
