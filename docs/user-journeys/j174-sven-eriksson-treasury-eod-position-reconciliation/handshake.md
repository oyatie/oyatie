---
doc_class: User-Journey-Handshake
journey_id: j174-sven-eriksson-treasury-eod-position-reconciliation
date: 2026-05-20
authority_tier: 2
status: draft
---

# j174 — Handshake matrix

Every named µservice call for the 8.5-hour EOD cycle (May 20 14:48 → 23:18 CEST). Transport HTTPS over QUIC per ADR-0253. SWIFT MT940/MT942 ingested via SWIFT FINplus + ISO 20022 MX hybrid. Per-cut-off-window Cedar-validated per ADR-0243 + ADR-0244. HLC timestamps per ADR-0252. Swedish + Norwegian + Danish + Finnish + German + English + JP + KR + ZH + PT preservation UTF-8 NFC byte-exact.

## Notation

- `[BHI]` Bohlin-Hjelmqvist Industries parent
- `[ENT]` Subsidiary entity (DE, US, UK, SG, JP, KR, BR)
- `[BANK]` Bank tenant (Nordea / Handelsbanken / JPMorgan / HSBC / Mizuho / SMBC / Bradesco)
- `[MMF]` Money market fund counterparty
- `→` synchronous request
- `←` synchronous response
- `↪` side-effect on bus
- `⟂` denied path

## §1 Cockpit open (May 20 14:48 CEST)

`[BHI:sven.eriksson] → treasury-cockpit` — `GET /v1/treasury/cockpit/open`

```json
{
  "principal": "sven.eriksson@bohlin-hjelmqvist-industries-ab-parent",
  "role_assertion": "treasury_operations_manager",
  "ctp_attestation_id": "ctp-sven-2019-Δ4810",
  "passkey_assertion_token": "wb-jwt-...",
  "yubikey_attestation": "yk-5c-nfc-sven-2025"
}
```

Cedar: permit (treasury_operations_manager + CTP + passkey). Audit: `EVT-J174-COCKPIT-OPENED-Δ000`.

## §2 SWIFT MT942 intraday ingestion (rolling, 15-min cadence)

`[BANK:nordea-bank-ab] → payments` — `POST /v1/payments/swift-mt942/ingest`

```protobuf
message SwiftMT942IngestRequest {
  string mt942_message_id = 1;
  string sender_bic = 2;
  string receiver_bic = 3;
  string account_identifier = 4;
  string statement_type = 5;             // "MT942_intraday"
  string intraday_balance_currency = 6;
  double intraday_balance_amount = 7;
  uint32 booked_movements_since_last_report = 8;
  string intraday_window_start = 9;       // YYYYMMDDHHMM
  string intraday_window_end = 10;
  google.protobuf.Timestamp dispatched_at = 11;
}
```

Audit: `EVT-J174-INTRADAY-MT942-ROLLUP-Δ001a` (every 15-min window; composite per cut).

## §3 SWIFT MT940 EOD ingestion (per cut-off window)

### 3.1 Nordea SEK4820012 (Stockholm cut)

`[BANK:nordea-bank-ab] → payments` — `POST /v1/payments/swift-mt940/ingest`

```protobuf
message SwiftMT940IngestRequest {
  string mt940_message_id = 1;
  string sender_bic = 2;                  // "NDEASESS"
  string receiver_bic = 3;                 // "BOHLSESS"
  string account_identifier = 4;
  string statement_type = 5;              // "MT940_EOD"
  string opening_balance_currency = 6;
  double opening_balance_amount = 7;
  string opening_balance_basis = 8;       // "previous_business_day_closing"
  string closing_balance_currency = 9;
  double closing_balance_amount = 10;
  string closing_balance_basis = 11;       // "current_eod"
  uint32 booked_movements_count = 12;
  double intraday_turnover_amount = 13;
  google.protobuf.Timestamp dispatched_at = 14;
  google.protobuf.Timestamp received_at = 15;
  uint32 ingestion_latency_seconds = 16;
}
```

Cedar: permit (bank_tenant + sender_bic_valid). Audit: `EVT-J174-MT940-INGESTION-Δ001b-acct-{n}`.

### 3.2 Per-cut composite

`payments → audit-chain` — internal RPC `Payments/EmitMT940Composite`

```json
{
  "cut_window": "europe_cut",
  "cut_window_open_t": "2027-05-20T15:00:00+02:00",
  "cut_window_close_t": "2027-05-20T15:18:14+02:00",
  "total_mt940_received": 16,
  "total_mt940_ingested": 16,
  "ingestion_p95_latency_seconds": 18,
  "total_intraday_turnover_eur_equivalent": 148422184.18,
  "total_eod_balance_eur_equivalent": 298184148.42,
  "parse_errors": 0,
  "mt942_vs_mt940_reconciliation_discrepancy_eur": 0.00
}
```

Audit: `EVT-J174-MT940-INGESTION-001-{cut}` (per cut composite) + `EVT-J174-MT940-INGESTION-001` (daily composite).

## §4 Cash position computation (post-cut interim)

`[BHI:sven.eriksson] → treasury` — `POST /v1/treasury/cash-position/compute`

```protobuf
message CashPositionComputeRequest {
  string eod_session_id = 1;                     // "eod-bhi-2027-05-20"
  string cut_window = 2;                          // "europe_cut" | "london_cut" | "us_cut" | "final"
  string fx_rate_source_primary = 3;             // "Refinitiv"
  string fx_rate_source_backup = 4;              // "Bloomberg"
  google.protobuf.Timestamp fx_rate_snapshot_t = 5;
  double fx_rate_spread_tolerance_pct = 6;        // 0.0001
  string reference_currency = 7;                  // "EUR"
  bool include_intraday_mt942 = 8;
}

message CashPositionComputeResponse {
  string compute_id = 1;
  map<string, double> per_entity_position_eur = 2;
  map<string, double> per_currency_position_eur = 3;
  double group_position_interim_eur = 4;
  double projected_eod_group_eur = 5;
  double projected_eod_overnight_usd = 6;
  google.protobuf.Timestamp computed_at = 7;
  string hlc_compute_timestamp = 8;
}
```

Cedar: permit (treasury_operations_manager + cut_window_open). Audit: `EVT-J174-CASH-POSITION-COMPUTED-002`.

## §5 FX hedge book delta-hedging compute

`[BHI:sven.eriksson] → treasury` — `POST /v1/treasury/fx-delta-hedge/compute`

```json
{
  "eod_session_id": "eod-bhi-2027-05-20",
  "hedge_book_snapshot_t": "2027-05-20T17:00:00+02:00",
  "categories_to_evaluate": [
    "sek_eur", "sek_usd", "sek_gbp", "sek_jpy", "sek_other"
  ],
  "rebalance_trigger_threshold_pct": 0.0050,
  "fx_rate_snapshot_t": "2027-05-20T17:00:00+02:00"
}
```

Response:

```json
{
  "fx_delta_id": "fx-delta-bhi-2027-05-20",
  "per_category": {
    "sek_eur": {"forwards": 42, "swaps": 8, "notional_sek": 8400000000, "delta_pct_today": 0.0042},
    "sek_usd": {"forwards": 58, "swaps": 12, "notional_sek": 6200000000, "delta_pct_today": -0.0031},
    "sek_gbp": {"forwards": 18, "swaps": 4, "notional_sek": 2800000000, "delta_pct_today": 0.0018},
    "sek_jpy": {"forwards": 8, "swaps": 2, "notional_sek": 600000000, "delta_pct_today": -0.0008},
    "sek_other": {"forwards": 16, "swaps": 2, "notional_sek": 400000000, "delta_pct_today": 0.0006}
  },
  "max_delta_pct_today": 0.0042,
  "rebalance_triggered": false,
  "hedge_book_pnl_today_sek": -8184228,
  "underlying_gain_today_sek": 12184228,
  "net_today_sek": 4000000,
  "hedge_book_pnl_ytd_sek": 142184228,
  "computed_at": "2027-05-20T17:00:18+02:00"
}
```

Cedar: permit (treasury_operations_manager + hedge_book_read). Audit: `EVT-J174-FX-DELTA-HEDGE-003`.

## §6 Intercompany netting matrix

`[BHI:sven.eriksson] → treasury` — `POST /v1/treasury/intercompany-netting/compute`

```protobuf
message IntercompanyNettingRequest {
  string eod_session_id = 1;
  repeated string entity_ids = 2;                // 8 entities
  string reference_currency = 3;                 // "EUR"
  string netting_period = 4;                     // "2027-05-20"
  string in_house_bank_id = 5;
}

message IntercompanyNettingResponse {
  string netting_id = 1;
  uint32 matrix_dimension = 2;
  repeated PerPairPosition pairs = 3;
  map<string, double> net_per_entity_eur = 4;
  double total_netted_volume_eur = 5;
  string settlement_method = 6;                  // "in_house_bank_book_transfer"
  string settlement_t = 7;
  string hlc_settlement_timestamp = 8;
}

message PerPairPosition {
  string entity_a = 1;
  string entity_b = 2;
  double net_eur = 3;                            // positive = a receives from b
}
```

Cedar: permit (treasury + intercompany_netting). Audit: `EVT-J174-INTERCOMPANY-NETTING-004`.

## §7 Cash sweep with Group Treasurer co-sign

### 7.1 Sweep preparation

`[BHI:sven.eriksson] → treasury` — `POST /v1/treasury/cash-sweep/prepare`

```protobuf
message CashSweepPrepareRequest {
  string eod_session_id = 1;
  double target_overnight_amount_usd = 2;        // 340000000
  repeated MMFAllocation mmf_allocations = 3;
  repeated OvernightDepositAllocation overnight_deposit_allocations = 4;
}

message MMFAllocation {
  string vehicle_name = 1;
  string vehicle_issuer = 2;
  double amount_usd = 3;
  double yield_1d_pct = 4;
}

message OvernightDepositAllocation {
  string bank_principal = 1;
  double amount_usd = 2;
  double yield_1d_pct = 3;
}
```

Audit: `EVT-J174-CASH-SWEEP-PREPARED-Δ005-prep`.

### 7.2 Group Treasurer co-sign

`[BHI:annika.lindqvist-holmberg] → treasury` — `POST /v1/treasury/cash-sweep/co-sign`

```json
{
  "sweep_id": "sweep-bhi-2027-05-20",
  "co_signer_principal": "annika.lindqvist-holmberg@bohlin-hjelmqvist-industries-ab-parent",
  "co_signer_role_assertion": "group_treasurer_act_fia_certified",
  "co_signer_passkey_assertion_token": "wb-jwt-annika-...",
  "co_signer_yubikey_attestation": "yk-5c-nfc-annika-2025",
  "approval_decision": "approve",
  "co_signed_at": "2027-05-20T22:00:18+02:00"
}
```

Cedar: permit (group_treasurer + ACT_FIA + passkey + sweep_amount_under_ceiling). Audit: `EVT-J174-CASH-SWEEP-CO-SIGN-Δ005a`.

### 7.3 Sweep execution

`[BHI:sven.eriksson] → treasury` — `POST /v1/treasury/cash-sweep/execute`

```json
{
  "sweep_id": "sweep-bhi-2027-05-20",
  "executed_at": "2027-05-20T23:14:50+02:00",
  "hlc_sweep_timestamp": "hlc:2027-05-20T21:14:50Z:Δ0070",
  "sanctions_destination_clean": true,
  "actual_overnight_position_usd": 340001184
}
```

Audit: `EVT-J174-CASH-SWEEP-005`.

## §8 Basel-III LCR computation

`[BHI:sven.eriksson] → finops-portal` — `POST /v1/finops/lcr/compute`

```protobuf
message LCRComputeRequest {
  string eod_session_id = 1;
  string reference_currency = 2;                 // "EUR"
  google.protobuf.Timestamp compute_t = 3;
  bool include_30d_outflow_projection = 4;
  bool include_inflow_cap_75pct = 5;
}

message LCRComputeResponse {
  string lcr_id = 1;
  double hqla_level_1_eur = 2;
  double hqla_level_2a_eur = 3;
  double hqla_level_2b_eur = 4;
  double hqla_total_eur = 5;
  double outflow_30d_retail_eur = 6;
  double outflow_30d_wholesale_eur = 7;
  double outflow_30d_secured_eur = 8;
  double outflow_30d_derivative_eur = 9;
  double inflow_30d_eur = 10;
  double net_outflow_30d_eur = 11;
  double lcr_ratio = 12;
  bool ratio_threshold_met = 13;                   // ≥ 1.0
}
```

Audit: `EVT-J174-LCR-COMPUTED-006`.

## §9 Per-account Merkle attestation (47 anchors)

`[BHI:sven.eriksson] → audit-chain` — `POST /v1/audit-chain/eod-account-anchor/emit` (×47)

```protobuf
message EODAccountAnchorRequest {
  string anchor_id = 1;
  string eod_session_id = 2;
  string account_identifier = 3;
  string bank_principal = 4;
  string entity_id = 5;
  string currency = 6;
  double closing_balance_amount = 7;
  bytes merkle_root = 8;
  string external_transparency_log_batch = 9;
  ProofClass proof_class = 10;                    // INCLUSION_PROOF
  bool ifrs_9_attestation = 11;
  bool ias_21_fx_translation_attestation = 12;
  google.protobuf.Timestamp emitted_at = 13;
}
```

Audit: `EVT-J174-MERKLE-EOD-{account_id}-Δ007{n}`; composite `EVT-J174-MERKLE-EOD-007`.

## §10 Cut-off ordering attestation

`audit-chain → external-transparency-log` — internal RPC `AuditChain/EmitCutOffOrdering`

```json
{
  "eod_session_id": "eod-bhi-2027-05-20",
  "cut_off_ordering": [
    {"cut": "asia_cut", "closed_at": "2027-05-20T11:00:00+02:00", "ordering_seq": 1},
    {"cut": "sg_cut", "closed_at": "2027-05-20T11:00:00+02:00", "ordering_seq": 2},
    {"cut": "europe_cut", "closed_at": "2027-05-20T15:18:14+02:00", "ordering_seq": 3},
    {"cut": "london_cut", "closed_at": "2027-05-20T18:18:42+02:00", "ordering_seq": 4},
    {"cut": "us_cut", "closed_at": "2027-05-20T23:14:00+02:00", "ordering_seq": 5},
    {"cut": "stockholm_night_cut", "closed_at": "2027-05-20T23:18:00+02:00", "ordering_seq": 6}
  ],
  "ordering_preserved": true
}
```

Audit: `EVT-J174-CUT-OFF-ORDERING-008`.

## §11 Pack manifest + observability + Cedar deny

### 11.1 Pack manifest

`[BHI:sven.eriksson] → compliance` — `GET /v1/compliance/pack-manifest?eod=eod-bhi-2027-05-20`

```json
{
  "active_packs": [
    "pack-ifrs-9-financial-instruments",
    "pack-ias-21-fx-effects",
    "pack-emir-article-9-trade-reporting",
    "pack-dodd-frank-title-vii-swap-reporting",
    "pack-mifid-ii-transaction-reporting",
    "pack-bis-basel-iii-lcr",
    "pack-iso-20022-mx-2026",
    "pack-ibor-transition-rfr-2027"
  ],
  "cross_validation_state": "passed",
  "pack_manifest_signature": "sha256:c8d4...fa72"
}
```

Audit: `EVT-J174-PACK-MANIFEST-009`.

### 11.2 Observability SLO

`observability → audit-chain` — `Observability/EmitEODLatencyReport`

```json
{
  "eod_session_id": "eod-bhi-2027-05-20",
  "mt940_ingestion_p95_seconds": 18,
  "cash_position_compute_p95_seconds": 11,
  "fx_delta_hedge_compute_p95_seconds": 4,
  "intercompany_netting_compute_p95_seconds": 6,
  "lcr_compute_p95_seconds": 4,
  "per_region_latency_targets_met": true,
  "overall_status": "ALL_PASS"
}
```

Audit: `EVT-J174-OBSERVABILITY-SLO-010`.

### 11.3 Cedar deny coverage

`[BHI:sven.eriksson] → audit-chain` — `GET /v1/audit-chain/cedar-deny-coverage?eod=eod-bhi-2027-05-20`

```json
{
  "denied_sweep_without_treasurer_cosign": 4,
  "denied_bank_statement_metadata_read": 6,
  "total_denied": 10,
  "observability_redaction_pct": 100
}
```

Audit: `EVT-J174-CEDAR-DENY-COVERAGE-011`.

## §12 EOD posted

`[BHI:sven.eriksson] → treasury` — `POST /v1/treasury/eod/post`

```json
{
  "eod_session_id": "eod-bhi-2027-05-20",
  "state_transition_from": "us_cut",
  "state_transition_to": "eod_posted",
  "group_position_final_eur": 312418224.18,
  "overnight_position_actual_usd": 340001184,
  "lcr_ratio": 1.42,
  "merkle_anchors_emitted": 47,
  "posted_at": "2027-05-20T23:18:42+02:00"
}
```

Audit: `EVT-J174-EOD-POSTED-Δ010`.

## §13 Summary

| Event class | Count | Cedar permits |
|---|---|---|
| EVT-J174-COCKPIT-OPENED-Δ000 | 1 | treasury_ops_mgr + CTP |
| EVT-J174-INTRADAY-MT942-ROLLUP-Δ001a | composite | bank + payments |
| EVT-J174-MT940-INGESTION-001 | composite (per cut + daily) | bank + payments |
| EVT-J174-CASH-POSITION-COMPUTED-002 | rolling | treasury + cut_window_open |
| EVT-J174-FX-DELTA-HEDGE-003 | 1 | treasury + hedge_book_read |
| EVT-J174-EXECUTIVE-ESCALATION-Δ003a | 1 | messenger executive |
| EVT-J174-INTERCOMPANY-NETTING-004 | 1 | treasury + intercompany |
| EVT-J174-CASH-SWEEP-PREPARED-Δ005-prep | 1 | treasury |
| EVT-J174-CASH-SWEEP-CO-SIGN-Δ005a | 1 | group_treasurer |
| EVT-J174-CASH-SWEEP-005 | 1 | treasury + co_sign |
| EVT-J174-LCR-COMPUTED-006 | 1 | finops_portal + lcr |
| EVT-J174-MERKLE-EOD-007 | 47 anchors + composite | audit_chain |
| EVT-J174-CUT-OFF-ORDERING-008 | 1 | audit_chain + ext-log |
| EVT-J174-PACK-MANIFEST-009 | 1 | compliance |
| EVT-J174-OBSERVABILITY-SLO-010 | 1 | observability |
| EVT-J174-CEDAR-DENY-COVERAGE-011 | 1 | audit_chain |
| EVT-J174-EOD-POSTED-Δ010 | 1 | treasury + state_transition |

Total: ~6,800 daily MT942/MT940 statement events + ~80 composite audit events. Cut-off ordering preserved. Multi-language preservation UTF-8 NFC byte-exact.
