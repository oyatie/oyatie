---
doc_class: User-Journey-Story
journey_id: j174-sven-eriksson-treasury-eod-position-reconciliation
date: 2026-05-20
authority_tier: 2
status: draft
---

# j174 — Story: Sven Eriksson opens EOD at 14:48 CEST Thursday May 20

## §0 — Thursday May 20, 2027, 14:48 CEST — BHI HQ, 7th floor Treasury Ops, Birger Jarlsgatan 41, Stockholm

Spring in Stockholm. 14°C and overcast, light rain forecast for the evening. Sven arrives at his Treasury Ops desk at 07:48 CEST as usual; today he is in the office because his manager Annika is presenting the Q1 funding strategy to the BHI board at 16:00. He's been working on intraday positions throughout the day. Now 14:48 CEST: the European cut-off marker is 12 minutes away.

His desk: a Vitra HAL chair, two 27" monitors + a third 32" on a vertical pivot (his cash-position dashboard), a Logitech MX Master 3S, and a green-and-white *Sverige Treasury* mug Annika gave him in 2021. He drinks his fourth coffee of the day (Löfbergs Lila brewed at the 6th-floor pantry). He authenticates: passkey + YubiKey + CTP attestation.

The active-tenant pill: `bohlin-hjelmqvist-industries-ab-parent · treasury · treasury_operations_manager · CTP_certified`.

His treasury operations cockpit:

```
[TREASURY OPS COCKPIT] BHI EOD 2027-05-20
─
state:                          intraday → europe_cut window opening at 15:00 CEST
asia_cut:                       closed at 11:00 CEST (4 hours ago)
europe_cut:                     T-12 minutes (opens at 15:00 CEST)
americas_cut:                   T+8h12 (23:00 CEST)
stockholm_night_cut:            T+8h42 (23:30 CEST)

accounts_in_scope:              47 across 12 currencies
banks:                          8 (Nordea + Handelsbanken + JPMorgan + HSBC + Mizuho + SMBC + Bradesco + 1 small correspondent)
entities:                       8 (BHI parent + 7 subsidiaries)
intraday_mt942_received_today:  3,142 (15-min cadence across 47 accounts × 11 windows since 06:00 CEST)
mt940_received_today_asia:      14 (Tokyo + SG cuts)
mt940_due_eu_cut:               16 (EU + UK)
mt940_due_us_cut:                8 (US)
mt940_due_stockholm_night:        9 (CEE + SEK overnight)

current_intraday_group_position_eur:  €312,418,224.18
target_eod_overnight_position_usd:    $340,000,000
fx_hedge_book_open_positions:         142 forwards + 28 cross-currency swaps
fx_delta_today_sek_eur:                +0.42%
fx_delta_today_sek_usd:                -0.31%
rebalance_trigger_threshold:           ±0.50%  (no trigger today)
```

`EVT-J174-COCKPIT-OPENED-Δ000` sealed at 14:48:18 CEST.

## §1 — May 20 14:50–15:00 CEST: pre-EU-cut intraday MT942 finalisation

Throughout the day, the BHI treasury system pulls **MT942** (intraday balance reports) from each bank every 15 minutes for active accounts. By 14:50, the intraday position for each EU + UK account is freshly updated:

```
[INTRADAY MT942 ROLLUP] 14:50 CEST EU + UK
─
nordea-bhi-parent-sek-sek4820012:     SEK 142,184,228.42  (last MT942 14:48:14)
nordea-bhi-parent-eur-eur4820013:    €82,148,492.18    (last MT942 14:47:28)
nordea-bhi-de-eur-eur4820014:         €48,184,228.84    (last MT942 14:46:44)
nordea-bhi-de-eur-eur4820015:         €18,242,082.92    (last MT942 14:46:48)
handelsbanken-bhi-parent-sek-shb1:    SEK 92,148,228.18  (last MT942 14:48:08)
handelsbanken-bhi-parent-eur-shb2:    €38,142,028.42    (last MT942 14:47:14)
jpmorgan-bhi-uk-gbp-jpm-1:            £42,184,028.18    (last MT942 14:48:18)
jpmorgan-bhi-uk-gbp-jpm-2:            £12,148,228.92    (last MT942 14:47:32)
nordea-bhi-no-nok-nordea-no-1:       NOK 84,148,228.42  (last MT942 14:47:08)
nordea-bhi-dk-dkk-nordea-dk-1:        DKK 62,148,228.42  (last MT942 14:46:14)
[...16 EU + UK accounts total...]
```

`EVT-J174-INTRADAY-MT942-ROLLUP-Δ001a` sealed at 14:50 CEST.

## §2 — May 20 15:00–15:18 CEST: EU cut — MT940 EOD statement ingestion

At 15:00:00 CEST, the EU cut-off opens. Nordea begins dispatching MT940 EOD statements for SEK + EUR + DKK + NOK accounts. The MT940 messages arrive at the BHI MT940 ingestion endpoint via SWIFT FINplus. The ingestion µservice parses each message:

```
[MT940 INGESTION] 15:00:42 CEST — Nordea
─
mt940_message_id:                NDEASESS-MT940-2027-05-20-Δ4820012
sender_bic:                      NDEASESS (Nordea Bank AB Stockholm)
receiver_bic:                     BOHLSESS (BHI parent SE)
account_identifier:               SEK4820012
statement_type:                   MT940 EOD
opening_balance:                  SEK 138,000,000.00 (book-closing-2027-05-19)
closing_balance:                  SEK 142,184,228.42 (eod-2027-05-20)
booked_movements:                123 (intraday)
intraday_turnover_sek:           SEK 248,184,228.00 (gross movement total)
ingestion_latency_seconds:        18 (sub-2-minute SLA met)
audit_event_id:                   EVT-J174-MT940-INGESTION-Δ001b-acct-001
```

Through 15:00–15:18 CEST, all **16 EU + UK MT940 messages** arrive + ingest:

```
[MT940 EU + UK CUT ROLLUP] 15:18:14 CEST
─
total_eu_uk_mt940_received:    16
total_eu_uk_mt940_ingested:    16 (100% within 2-min SLA)
total_intraday_turnover_eu_uk_eur_equivalent:  €148,422,184.18
total_eod_balance_eu_uk_eur_equivalent:       €298,184,148.42
parse_errors:                   0
discrepancy_vs_intraday_mt942:  €0.00 (perfect reconciliation)
```

`EVT-J174-MT940-INGESTION-001` (composite event for EU cut) sealed at 15:18 CEST.

## §3 — May 20 15:18–16:00 CEST: cash position computation + entity rollup

Sven runs the cash-position computation:

```
[CASH POSITION COMPUTATION] 15:18:42 CEST — post-EU-cut interim view
─
fx_rate_source:                Refinitiv (primary) + Bloomberg (backup; 0.0001% spread tolerance)
fx_rate_snapshot_t:            2027-05-20T15:18:00.018+02:00
fx_rates_in_effect:
  SEK_EUR:    0.0884       (1 SEK = 0.0884 EUR; mid)
  SEK_USD:    0.0944       (mid)
  EUR_USD:    1.0680       (mid)
  GBP_EUR:    1.1748       (mid)
  GBP_USD:    1.2545       (mid)
  NOK_EUR:    0.0848       (mid)
  DKK_EUR:    0.1342       (mid)
  CHF_EUR:    1.0418       (mid)
  JPY_EUR:    0.00608      (mid)
  CNY_EUR:    0.1284       (mid)
  KRW_EUR:    0.000698     (mid)
  BRL_EUR:    0.1748       (mid)

per_entity_position_eur:
  bhi_parent_se:        €98,148,228.18
  bhi_manufacturing_de: €82,184,228.42
  bhi_usa_inc:           €0.00 (Asia + US cut pending)
  bhi_uk_ltd:           €58,184,148.84
  bhi_asia_pte_sg:      €18,148,228.42 (Asia cut done)
  bhi_japan_kk:         €22,148,228.18 (Asia cut done)
  bhi_korea:            €4,184,228.42  (Asia cut done)
  bhi_brasil_ltda:      €8,148,228.42  (intraday only; LATAM cut later)

group_position_eur_interim:    €291,145,718.88
projected_eod_group_eur:       €312,418,224.18 (post-US cut)
projected_eod_overnight_usd:   $333,742,184.18 (close to $340M target)
```

`EVT-J174-CASH-POSITION-COMPUTED-002` (interim; post-EU cut) sealed at 16:00 CEST.

## §4 — May 20 16:00–17:00 CEST: FX hedge book delta-hedging compute

Sven turns to the FX hedge book delta-hedging compute. The treasury µservice loads the 142 open FX forwards + 28 cross-currency swaps:

```
[FX HEDGE BOOK SUMMARY] 16:00:18 CEST
─
total_forwards_open:            142
total_swaps_open:                28
forwards_total_notional_sek:    SEK 12,148,228,142
swaps_total_notional_sek:       SEK 6,242,184,028
total_hedge_book_notional_sek:  SEK 18,390,412,170 (≈ SEK 18.4B)

hedge_categories:
  hedge_against_sek_eur_exposure:   42 forwards + 8 swaps  notional SEK 8.4B
  hedge_against_sek_usd_exposure:   58 forwards + 12 swaps notional SEK 6.2B
  hedge_against_sek_gbp_exposure:   18 forwards + 4 swaps  notional SEK 2.8B
  hedge_against_sek_jpy_exposure:    8 forwards + 2 swaps  notional SEK 0.6B
  hedge_against_sek_other_exposures: 16 forwards + 2 swaps notional SEK 0.4B

underlying_today:
  sek_eur_movement_today:    +0.42%  (mid 0.0880 → 0.0884)
  sek_usd_movement_today:    -0.31%  (mid 0.0947 → 0.0944)
  sek_gbp_movement_today:    +0.18%  (mid)
  sek_jpy_movement_today:    -0.08%  (mid)

delta_vs_rebalance_trigger_0.5pct:  none triggered today (max delta 0.42%)
hedge_book_pnl_today_sek:           -SEK 8,184,228 (small loss from sek-eur favourable move)
hedge_book_pnl_ytd_sek:              SEK 142,184,228 (positive from hedging strategy)
```

`EVT-J174-FX-DELTA-HEDGE-003` sealed at 17:00 CEST.

Annika (the Group Treasurer) pings Sven via the executive escalation channel:

> "Sven — the SEK-EUR move today is favourable for the underlying but we're losing on the hedge. Anything material to flag for the board this afternoon?"

Sven replies in Swedish:

> "Annika — totalt nettoeffekt på dagen är liten (≈ -SEK 8M på hedgen vs gain på underliggande). Ingen rebalance triggad (0.42% under 0.50% tröskel). Hedgen YTD är fortfarande +SEK 142M. Nothing material to flag — vi ligger fortfarande på strategins design."

*("Annika — total net effect today is small (≈ -SEK 8M on the hedge vs gain on the underlying). No rebalance triggered (0.42% under 0.50% threshold). The hedge YTD is still +SEK 142M. Nothing material to flag — we're still in line with the strategy design.")*

`EVT-J174-EXECUTIVE-ESCALATION-Δ003a` sealed at 17:08 CEST.

## §5 — May 20 17:00–18:00 CEST: intercompany netting matrix

The treasury µservice computes the **8-entity intercompany netting matrix**:

```
[INTERCOMPANY NETTING MATRIX] 17:00:48 CEST
─
matrix_dimension:                8 entities (parent + 7 subsidiaries)
reference_currency:               EUR
netting_period:                   2027-05-20 daily
in_house_bank:                    BHI Internal Treasury (parent SE)

per-pair-position-eur (excerpt):
  bhi_parent_se ↔ bhi_manufacturing_de:    parent receives net €4,148,228 from DE
  bhi_parent_se ↔ bhi_uk_ltd:              parent pays net €2,184,228 to UK
  bhi_parent_se ↔ bhi_usa_inc:             parent receives net €8,184,228 from USA
  bhi_manufacturing_de ↔ bhi_uk_ltd:        DE pays net €1,148,228 to UK
  bhi_asia_pte_sg ↔ bhi_japan_kk:           SG receives net €2,148,228 from JP
  bhi_korea ↔ bhi_japan_kk:                  KR pays net €842,184 to JP
  bhi_brasil_ltda ↔ bhi_parent_se:           BR pays net €1,184,228 to parent
  [...total 28 pairs...]

net_per_entity_eur:
  bhi_parent_se:           +€8,148,228 (net receiver)
  bhi_manufacturing_de:    -€3,000,000 (net payer)
  bhi_usa_inc:              -€8,184,228 (net payer)
  bhi_uk_ltd:                +€1,036,000 (net receiver)
  bhi_asia_pte_sg:           +€1,306,000 (net receiver)
  bhi_japan_kk:              -€1,306,000 (net payer)
  bhi_korea:                 -€842,184 (net payer)
  bhi_brasil_ltda:           -€1,184,228 (net payer)
total_netted_volume_eur:    €18,648,228 (≈ €18.6M)
total_settlement_volume_post_netting_eur: €18,648,228 (settles via in-house bank)

settlement_method:                 in-house-bank-book-transfer (no external SWIFT)
settlement_time:                   2027-05-20T17:42:00+02:00
hlc_settlement_timestamp:          hlc:2027-05-20T15:42:00Z:Δ0042
```

`EVT-J174-INTERCOMPANY-NETTING-004` sealed at 17:42 CEST.

## §6 — May 20 18:00–22:00 CEST: London + NY MT940 ingestion + cash sweep prep

London EOD cut at 18:00 CEST (= 17:00 BST). JPMorgan UK MT940 statements arrive:

```
[MT940 INGESTION] 18:00:42 CEST — JPMorgan UK
─
jpm-bhi-uk-gbp-jpm-1 closing:      £42,184,028.18
jpm-bhi-uk-gbp-jpm-2 closing:      £12,148,228.92
all 8 UK + London-pertinent MT940s ingested by 18:18 CEST
```

`EVT-J174-MT940-INGESTION-001-london` sealed at 18:18 CEST.

NY EOD cut opens at 23:00 CEST (17:00 EST). Before that, Sven prepares the **cash sweep**:

```
[CASH SWEEP PREPARATION] 21:42 CEST
─
projected_eod_overnight_position_usd: $340,184,228 (post-US cut estimate)
target_overnight_position_usd:        $340,000,000

sweep_allocation:
  money_market_funds (3 vehicles):
    BlackRock TempCash Plus (US Treasury MMF):       $98,000,000 (RBC, 1-day yield 4.82%)
    Fidelity Government Cash Reserves (Govt MMF):    $84,000,000 (Fidelity, 1-day yield 4.78%)
    JPM US Treasury Plus MMF (US Treasury MMF):      $66,000,000 (JPMorgan, 1-day yield 4.85%)
    subtotal money_market_funds:                     $248,000,000

  overnight_deposits (4 banks):
    JPMorgan Chase ($26M overnight; rate 4.62%):     $26,000,000
    HSBC overnight ($22M; rate 4.58%):                $22,000,000
    Citi overnight ($24M; rate 4.65%):                $24,000,000
    BNY Mellon overnight ($20M; rate 4.55%):          $20,000,000
    subtotal overnight_deposits:                     $92,000,000

  total_sweep_allocation_usd:                       $340,000,000
  sanctions_screening_destination_counterparties:    clean
  group_treasurer_co_sign_required:                  pending (Annika)
```

Sven pings Annika for the co-sign:

> "Annika — kassasvepet är planerat. $340M; allokering: $248M MMF (BR + Fidelity + JPM Govt) + $92M overnight deposits (JPM + HSBC + Citi + BNY). Räntor 4.55–4.85%. Behöver din co-sign inom 30 minuter."

Annika replies + co-signs at 22:00 CEST:

> "Godkänt. Sven, kör."

`EVT-J174-CASH-SWEEP-CO-SIGN-Δ005a` sealed at 22:00 CEST.

## §7 — May 20 22:30–23:18 CEST: US cut + final EOD posting + Merkle attestation

US EOD cut opens at 23:00 CEST. JPMorgan US MT940 + HSBC US MT940 arrive:

```
[MT940 INGESTION] 23:00:18 CEST — US cut
─
jpm-bhi-usa-usd-jpm-us-1 closing:    $148,184,228.42
jpm-bhi-usa-usd-jpm-us-2 closing:    $82,148,228.18
hsbc-bhi-usa-usd-hsbc-us-1 closing:  $48,148,228.84
[...8 US MT940s total...]
all 8 US MT940s ingested by 23:14 CEST
```

`EVT-J174-MT940-INGESTION-001-us` sealed at 23:14 CEST.

Final cash position computation:

```
[FINAL EOD CASH POSITION] 23:14:48 CEST
─
group_position_eur_final:           €312,418,224.18
group_position_usd_equivalent:      $333,742,184 (pre-sweep; we'll allocate $340M target)

actual_overnight_position_target:    $340,000,000 (after FX consolidation from EUR + SEK + GBP holdings)
```

Cash sweep executes:

```
[CASH SWEEP EXECUTION] 23:14:50 CEST
─
sweep_legs:                           7 (3 MMF + 4 deposits)
all_legs_dispatched:                 true
hlc_sweep_timestamp:                   hlc:2027-05-20T21:14:50Z:Δ0070
audit_event_id:                       EVT-J174-CASH-SWEEP-005
sanctions_destination_clean:          true
overnight_position_us_actual:        $340,001,184 (slight FX favourable)
```

`EVT-J174-CASH-SWEEP-005` sealed at 23:14:50 CEST.

Basel-III LCR computation:

```
[BASEL III LCR COMPUTATION] 23:16:00 CEST
─
hqla_total_eur:                      €298,148,228 (level-1 + level-2A high-quality liquid assets)
total_net_cash_outflow_30d_eur:      €210,148,228 (projected over 30 days)
lcr_ratio:                            1.42 (target ≥ 1.0; comfortable)
lcr_components:
  level_1_hqla_eur:                 €248,184,228 (cash + central-bank reserves + Level-1 govt bonds)
  level_2a_hqla_eur:                 €49,964,000 (Level-2A corporate bonds; capped at 40%)
  level_2b_hqla_eur:                  €0 (none currently)
  outflow_30d_retail_eur:           €18,142,028
  outflow_30d_wholesale_eur:        €148,142,028
  outflow_30d_secured_eur:           €12,184,228
  outflow_30d_derivative_eur:         €4,148,228
  inflow_30d_eur:                    -€56,148,228 (capped at 75% of outflow)
  ratio_threshold_met:               true
```

`EVT-J174-LCR-COMPUTED-006` sealed at 23:16 CEST.

Per-account Merkle attestation:

```
[PER-ACCOUNT MERKLE ATTESTATION] 23:17:00 CEST
─
total_accounts:                       47
per_account_anchors_emitted:          47
merkle_root_per_account:              (recorded per account in audit-chain)
external_transparency_log_batch:      external-transparency-log-batch-2027-05-20-stockholm
batch_emitted_at:                      2027-05-20T23:17:00+02:00
proof_class:                          inclusion_proof
ifrs_9_attestation:                    enabled
ias_21_fx_translation_attestation:    enabled
```

`EVT-J174-MERKLE-EOD-007` sealed at 23:17 CEST.

Per-region cut-off ordering check:

```
[CUT-OFF ORDERING ATTESTATION] 23:17:30 CEST
─
asia_cut_completed:    2027-05-20T11:00:00+02:00 (CEST)
sg_cut_completed:      2027-05-20T11:00:00+02:00 (same window)
europe_cut_completed:   2027-05-20T15:18:14+02:00
london_cut_completed:   2027-05-20T18:18:42+02:00
us_cut_completed:       2027-05-20T23:14:00+02:00
stockholm_night_cut:    2027-05-20T23:18:00+02:00 (post-Merkle)
ordering_preserved:    true
```

`EVT-J174-CUT-OFF-ORDERING-008` sealed at 23:17:30 CEST.

Pack manifest assertion:

```
[PACK MANIFEST ASSERTION] 23:18:00 CEST
─
active_packs:                          8
cross_validation:                     passed
pack_manifest_signature:              sha256:c8d4…fa72
```

`EVT-J174-PACK-MANIFEST-009` sealed at 23:18 CEST.

Observability SLO summary:

```
[OBSERVABILITY SLO REPORT] EOD 2027-05-20
─
mt940_ingestion_p95_seconds:          18 (target < 120s; PASS)
cash_position_compute_p95_seconds:    11 (target < 18s; PASS)
fx_delta_hedge_compute_p95_seconds:    4 (target < 10s; PASS)
intercompany_netting_compute_p95_seconds: 6 (target < 12s; PASS)
lcr_compute_p95_seconds:               4 (target < 8s; PASS)
per_region_latency_targets:           all met
overall_status:                       ALL_PASS
```

`EVT-J174-OBSERVABILITY-SLO-010` sealed at 23:18 CEST.

Cedar deny coverage report:

```
[CEDAR DENY COVERAGE] 23:18:14 CEST
─
denied_sweep_without_treasurer_cosign:  4 attempts (junior analyst pre-Annika-cosign)
denied_bank_statement_metadata_read:    6 attempts (non-treasury principals)
total_denied:                            10
observability_redaction_pct:             100
```

`EVT-J174-CEDAR-DENY-COVERAGE-011` sealed at 23:18:14 CEST.

Sven sends a final EOD wrap to Annika:

> "Annika — EOD klar. Group position eod €312,418,224. Overnight $340.0M kassasvep allokerat. LCR 1.42. Merkle anchors emitted (47). No exceptions. Hejdå."

Annika replies: "Tack! Sov gott."

`EVT-J174-EOD-POSTED-Δ010` sealed at 23:18:42 CEST.

## §8 — Stop condition

All 12 AC pass on the seeded fixture; EOD posting is sealed across 47 accounts with per-account Merkle attestation; the $340.001M overnight position is invested across 3 money-market funds + 4 overnight deposits; the LCR is 1.42 (≥ 1.0 threshold); the intercompany netting matrix is cleared via the BHI in-house bank; the per-region cut-off ordering Tokyo → SG → London → NY → Stockholm is preserved; all 8 pack manifests are cross-validated. Swedish + Norwegian + Danish + Finnish + German + English + Japanese + Korean + Chinese + Portuguese + diacritics UTF-8 NFC byte-exact.
