---
doc_class: User-Journey-UX-Flow
journey_id: j174-sven-eriksson-treasury-eod-position-reconciliation
date: 2026-05-20
authority_tier: 2
status: draft
---

# j174 — UX flow: treasury cockpit, cash-position dashboard, FX delta-hedge panel, intercompany netting matrix, cash sweep, LCR, EOD Merkle attestation

Six primary surfaces:

- Sven's treasury operations cockpit (3-monitor desktop layout)
- Cash position dashboard (per-account + per-entity + per-currency + group EUR view)
- FX hedge book delta-hedging panel
- Intercompany netting matrix (8-entity)
- Cash sweep to overnight investment + co-sign flow
- Basel-III LCR + EOD Merkle attestation

All screens preserve Swedish + Norwegian + Danish + Finnish + German + English + Japanese + Korean + Chinese + Portuguese byte-exact UTF-8 NFC. Per-cut-off-window indicator is always visible.

## Screen 1 — Treasury Operations cockpit (May 20 14:48 CEST)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  TREASURY OPS COCKPIT · BHI · Sven Eriksson (CTP, Treasury Ops Mgr)     │
├──────────────────────────────────────────────────────────────────────────┤
│  active tenant: bohlin-hjelmqvist-industries-ab-parent · treasury        │
│                                                                          │
│  ┌─ EOD STATE BAR (cut-off windows) ─────────────────────────────────┐  │
│  │  asia_cut       ✓ closed   2027-05-20T11:00 CEST                    │  │
│  │  europe_cut     ○ opens    2027-05-20T15:00 CEST  (T-12min)         │  │
│  │  london_cut     ○ opens    2027-05-20T18:00 CEST                     │  │
│  │  americas_cut   ○ opens    2027-05-20T23:00 CEST                    │  │
│  │  stockholm_night_cut ○     2027-05-20T23:30 CEST                    │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ ACCOUNTS + BANKS ─────────────────────────────────────────────────┐  │
│  │  47 accounts · 12 currencies · 8 banks                             │  │
│  │  Nordea · Handelsbanken · JPMorgan · HSBC · Mizuho · SMBC ·         │  │
│  │  Bradesco · DnB (correspondent for NOK)                             │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ INTRADAY MT942 ROLLUP (last 15 min) ──────────────────────────────┐ │
│  │  current_intraday_group_position_eur: €312,418,224                  │ │
│  │  intraday_turnover_today_sek: SEK 1.18B (≈ 1.2B target)             │ │
│  │  projected_eod_overnight_usd: $333,742,184 (close to $340M target)  │ │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ FX HEDGE BOOK ─────────────────────────────────────────────────────┐ │
│  │  forwards open: 142  · swaps open: 28                               │ │
│  │  notional total: SEK 18.4B                                          │ │
│  │  delta today vs underlying:                                         │ │
│  │     sek-eur +0.42%  · sek-usd -0.31%                                │ │
│  │  rebalance trigger: ±0.5% (no trigger today)                        │ │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  Cedar permit: treasury_operations_manager × CTP × passkey               │
│  Audit class: EVT-J174-COCKPIT-OPENED-Δ000                               │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 2 — Cash position dashboard (post-EU cut, 15:18 CEST)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  CASH POSITION DASHBOARD · EOD 2027-05-20 · interim (post-EU cut)       │
├──────────────────────────────────────────────────────────────────────────┤
│  fx_rate_source: Refinitiv (primary) + Bloomberg (backup)                │
│  fx_rate_snapshot_t: 15:18:00 CEST                                       │
│                                                                          │
│  ┌─ PER ENTITY (EUR-reference) ───────────────────────────────────────┐  │
│  │  bhi_parent_se          €98,148,228                                 │  │
│  │  bhi_manufacturing_de   €82,184,228                                 │  │
│  │  bhi_uk_ltd              €58,184,148                                 │  │
│  │  bhi_asia_pte_sg         €18,148,228 (Asia cut done)                │  │
│  │  bhi_japan_kk            €22,148,228 (Asia cut done)                │  │
│  │  bhi_korea               €4,184,228  (Asia cut done)                │  │
│  │  bhi_brasil_ltda         €8,148,228 (intraday only)                 │  │
│  │  bhi_usa_inc             €0          (US cut pending)               │  │
│  │  ─                                                                  │  │
│  │  group_position_interim  €291,145,718                                │  │
│  │  projected_eod_group     €312,418,224                                │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ PER CURRENCY ─────────────────────────────────────────────────────┐  │
│  │  SEK   (47% group, 4 accounts)                                      │  │
│  │  EUR   (32% group, 14 accounts)                                     │  │
│  │  USD   (9% group, 8 accounts)                                       │  │
│  │  GBP   (6% group, 4 accounts)                                       │  │
│  │  NOK + DKK + CHF + JPY + CNY + KRW + BRL + AUD (combined 6%)        │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  Cedar permit: treasury.cash_position_compute                            │
│  Audit class: EVT-J174-CASH-POSITION-COMPUTED-002                        │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 3 — FX hedge book delta-hedging panel (17:00 CEST)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  FX HEDGE BOOK DELTA-HEDGING · post-EU-cut                              │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─ HEDGE CATEGORIES ─────────────────────────────────────────────────┐  │
│  │  sek_eur:    42 fwd + 8 swap   notional SEK 8.4B   delta +0.42%   │  │
│  │  sek_usd:    58 fwd + 12 swap  notional SEK 6.2B   delta -0.31%   │  │
│  │  sek_gbp:    18 fwd + 4 swap   notional SEK 2.8B   delta +0.18%   │  │
│  │  sek_jpy:     8 fwd + 2 swap   notional SEK 0.6B   delta -0.08%   │  │
│  │  sek_other:  16 fwd + 2 swap   notional SEK 0.4B   delta misc      │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ REBALANCE TRIGGER ────────────────────────────────────────────────┐  │
│  │  threshold: ±0.50%   max delta today: 0.42%                         │  │
│  │  rebalance_triggered: false                                         │  │
│  │  next eval: tomorrow 09:00 CEST                                     │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ PNL ──────────────────────────────────────────────────────────────┐  │
│  │  hedge_book_pnl_today_sek:    -SEK 8,184,228 (small hedge loss)    │  │
│  │  underlying_gain_today_sek:    +SEK 12,184,228 (favourable move)   │  │
│  │  net_today_sek:                +SEK 4,000,000                       │  │
│  │  hedge_book_pnl_ytd_sek:       +SEK 142,184,228                     │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  Cedar permit: treasury.fx_delta_hedge_compute                           │
│  Audit class: EVT-J174-FX-DELTA-HEDGE-003                                │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 4 — Intercompany netting matrix (17:42 CEST)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  INTERCOMPANY NETTING MATRIX · 8 entities · EUR reference               │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─ NET PER ENTITY ───────────────────────────────────────────────────┐  │
│  │  bhi_parent_se          +€8,148,228 (net receiver)                  │  │
│  │  bhi_uk_ltd             +€1,036,000 (net receiver)                  │  │
│  │  bhi_asia_pte_sg        +€1,306,000 (net receiver)                  │  │
│  │  bhi_manufacturing_de   -€3,000,000 (net payer)                     │  │
│  │  bhi_usa_inc            -€8,184,228 (net payer)                     │  │
│  │  bhi_japan_kk           -€1,306,000 (net payer)                     │  │
│  │  bhi_korea              -€842,184  (net payer)                      │  │
│  │  bhi_brasil_ltda        -€1,184,228 (net payer)                     │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ SETTLEMENT ──────────────────────────────────────────────────────┐   │
│  │  total_netted_volume_eur:    €18,648,228                           │   │
│  │  settlement_method:           in-house-bank-book-transfer           │   │
│  │  external_swift_required:    no                                     │   │
│  │  hlc_settlement_t:            hlc:2027-05-20T15:42:00Z              │   │
│  │  ✓ settled                                                          │   │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  Cedar permit: treasury.intercompany_netting_compute                     │
│  Audit class: EVT-J174-INTERCOMPANY-NETTING-004                          │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 5 — Cash sweep + Group Treasurer co-sign (21:42–22:00 CEST)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  CASH SWEEP · $340M overnight investment · co-sign required             │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─ MMF (3 vehicles) ─────────────────────────────────────────────────┐  │
│  │  BlackRock TempCash Plus       $98,000,000   rate 4.82%             │  │
│  │  Fidelity Government Reserves  $84,000,000   rate 4.78%             │  │
│  │  JPM US Treasury Plus MMF      $66,000,000   rate 4.85%             │  │
│  │  subtotal MMF:                 $248,000,000                          │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ Overnight deposits (4 banks) ─────────────────────────────────────┐  │
│  │  JPMorgan overnight    $26,000,000   rate 4.62%                     │  │
│  │  HSBC overnight        $22,000,000   rate 4.58%                     │  │
│  │  Citi overnight        $24,000,000   rate 4.65%                     │  │
│  │  BNY Mellon overnight  $20,000,000   rate 4.55%                     │  │
│  │  subtotal deposits:    $92,000,000                                  │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  total: $340,000,000   weighted avg yield: 4.75%                         │
│                                                                          │
│  ┌─ CO-SIGN PANEL ────────────────────────────────────────────────────┐  │
│  │  ✓ sanctions screening on destination counterparties: clean         │  │
│  │  ○ group_treasurer_co_sign:  pending (Annika)                       │  │
│  │  [send for co-sign]                                                 │  │
│  │                                                                    │  │
│  │  Annika co-signed at 22:00 CEST ✓                                   │  │
│  │  [execute sweep now]                                                │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  Cedar permit: treasury.cash_sweep_initiate × treasurer_co_sign         │
│  Audit class: EVT-J174-CASH-SWEEP-005                                    │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 6 — Basel-III LCR + EOD Merkle attestation (23:14–23:18 CEST)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  BASEL-III LCR + EOD MERKLE ATTESTATION                                 │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─ LCR COMPONENTS ───────────────────────────────────────────────────┐  │
│  │  HQLA (level 1):       €248,184,228                                 │  │
│  │  HQLA (level 2A):       €49,964,000                                  │  │
│  │  HQLA (level 2B):        €0                                          │  │
│  │  HQLA total:           €298,148,228                                 │  │
│  │  ─                                                                  │  │
│  │  outflow_30d_retail:    €18,142,028                                  │  │
│  │  outflow_30d_wholesale: €148,142,028                                 │  │
│  │  outflow_30d_secured:    €12,184,228                                 │  │
│  │  outflow_30d_derivative: €4,148,228                                  │  │
│  │  inflow_30d (cap 75%): -€56,148,228                                 │  │
│  │  net_outflow_30d:       €210,148,228                                │  │
│  │                                                                    │  │
│  │  LCR ratio:             1.42  (target ≥ 1.0)  PASS                  │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ PER-ACCOUNT MERKLE ATTESTATION ──────────────────────────────────┐   │
│  │  accounts: 47   anchors emitted: 47                                  │   │
│  │  external_transparency_log_batch: external-tl-batch-2027-05-20      │   │
│  │  proof_class: inclusion_proof                                       │   │
│  │  ifrs_9_attestation: ✓                                              │   │
│  │  ias_21_fx_translation_attestation: ✓                               │   │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ CUT-OFF ORDERING ATTESTATION ─────────────────────────────────────┐  │
│  │  asia ✓ → sg ✓ → europe ✓ → london ✓ → us ✓ → stockholm_night ✓    │  │
│  │  ordering_preserved: true                                           │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  Cedar permit: finops_portal.lcr_compute × audit_chain.eod_account_anchor│
│  Audit class: EVT-J174-LCR-COMPUTED-006 + EVT-J174-MERKLE-EOD-007        │
└──────────────────────────────────────────────────────────────────────────┘
```

## Cross-screen rules

1. **Cut-off window indicator**: visible on every screen during EOD.
2. **MT940 freshness indicator**: every cash position screen shows last MT942/MT940 ingestion timestamp per account.
3. **FX rate source**: every FX-related screen shows source + backup + spread tolerance.
4. **Per-share-currency separation**: every screen shows entity + currency separately; no aggregation that obscures FX position.
5. **Co-sign required**: cash sweep above $X requires Group Treasurer co-sign.
6. **LCR threshold**: LCR < 1.0 triggers immediate escalation.
7. **Per-account Merkle**: every account has its own anchor; no composite-only views.
8. **Language preservation**: byte-exact UTF-8 NFC across all languages.
9. **Cedar permit binding**: every screen has a specific Cedar permit + audit-event class.
10. **Pack manifest**: 8 packs visible on cockpit + EOD attestation screen.

## Accessibility + i18n

- Screen reader: cut-off window state + LCR PASS/FAIL clearly announced.
- Color: cut-off indicators + LCR pass/fail use WCAG AA 4.5:1 contrast.
- Language picker: Swedish + Norwegian + Danish + Finnish + German + English + JP + KR + ZH + PT.
- Mobile: read-only LCR + group position available on mobile for after-hours emergency.
