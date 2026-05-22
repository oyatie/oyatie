---
doc_class: User-Journey-README
journey_id: j174-sven-eriksson-treasury-eod-position-reconciliation
slice: treasury-eod-47-bank-accounts-12-currencies-mt940-mt942-fx-hedge-delta-intercompany-netting-cash-sweep-overnight-investment-sek-1-2b-daily-turnover-usd-340M-overnight
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Sven Eriksson (white/middle-office; treasury operations — Stockholm HQ multinational manufacturer)
audience_type: B2B_TREASURY + EOD_RECONCILIATION + MULTI_CURRENCY + INTERCOMPANY_NETTING
microservice_count: 5
pack_overlay_anchor: IFRS-9 + IAS-21 + EMIR-Article-9 + Dodd-Frank-Title-VII + MiFID-II + ISO-20022-MX + BIS-Basel-III-LCR + IBOR-Transition-RFR
related_adrs:
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0251-compliance-pack-primitive
  - ADR-0252-hlc-default-truetime-tier
  - ADR-0253-http3-quic-default-protocol
  - ADR-0263-observability-emission-contract
  - ADR-0254-kubernetes-everywhere-pods-cloud-hypervisor
  - ADR-0248-amazon-shape-cellular-architecture
---

# j174 — Treasury Ops Sven Eriksson runs EOD position reconciliation across 47 bank accounts + 12 currencies for Bohlin-Hjelmqvist Industries

## At a glance

Sven Eriksson (Sven Bertil Eriksson) is a **38-year-old Treasury Operations Manager** at **Bohlin-Hjelmqvist Industries AB** (publ; OMX:BHIA; Stockholm-headquartered multinational industrial-equipment manufacturer; ~21,400 employees across 14 countries; FY2026 revenue SEK 38.4B ≈ $3.6B; market cap SEK 142B ≈ $13.4B). Sven is Swedish (born Uppsala 1989, Lund University Civilekonom 2013, MSc Finance LSE 2015, CTP-certified Certified Treasury Professional 2019), joined Bohlin-Hjelmqvist in 2016-04 from a Treasury Analyst role at SEB. He reports to the Group Treasurer (Mrs. Annika Lindqvist-Holmberg, 50, ID-1976, ACT-FIA certified, joined BHI 2018 from a Treasurer role at Sandvik AB).

It is **Thursday May 20, 2027, 14:48 CEST (Stockholm)**. EOD (end-of-day) global cash position reconciliation begins at **15:00 CEST** (the European cut-off marker; Asia closed earlier, Americas still open). Sven is at the BHI Treasury Operations desk on the 7th floor of the BHI HQ on Birger Jarlsgatan 41, Stockholm. The treasury operations team is 4 people (Sven + 1 senior analyst + 2 junior analysts).

The scope of today's EOD:

- **47 bank accounts** across **12 currencies** (SEK, EUR, USD, GBP, NOK, DKK, CHF, JPY, CNY, KRW, BRL, AUD)
- **8 named banks**: **Nordea** (SEK + DKK + NOK + EUR primary), **Svenska Handelsbanken** (SEK + EUR secondary), **JPMorgan Chase** (USD + GBP + AUD), **HSBC** (HKD + CNY + USD secondary), **Mizuho Bank** (JPY + KRW), **SMBC** (JPY secondary), **Banco Bradesco** (BRL), and a few smaller correspondents
- **8 legal entities**: Bohlin-Hjelmqvist Industries AB (parent SE), BHI Manufacturing GmbH (DE), BHI USA Inc. (US), BHI UK Ltd. (GB), BHI Asia Pte Ltd. (SG), BHI Japan KK (JP), BHI Korea Ltd. (KR), BHI Brasil Ltda. (BR)
- **SEK 1.2B daily turnover** (across 47 accounts; in/out movement)
- **USD 340M overnight position** (target end-of-day net cash sweep into overnight money market funds + bank deposits)
- **FX hedge book**: 142 outstanding FX forwards + 28 cross-currency swaps; total notional SEK 18.4B; primary purpose hedging SEK-vs-EUR + SEK-vs-USD exposure
- **Delta hedging window**: today's underlying FX movement vs hedge book; rebalance trigger ±0.5% delta
- **Intercompany netting matrix**: 8 entities; net positions computed in EUR (group reference currency); cleared via the BHI in-house bank
- **EOD cut-offs**: Tokyo 18:00 JST (11:00 CEST) → SG 17:00 SGT (11:00 CEST) → London 17:00 BST (18:00 CEST) → NY 17:00 EST (23:00 CEST); Stockholm runs a 15:00 CEST EU-side cut + a 17:30 CEST Stockholm-day cut + a 23:30 CEST night cut

Microservices: `payments` (SWIFT MT940/MT942 statement ingestion + payment status), `treasury` (cash position computation + FX position vs hedge book delta + intercompany netting + cash sweep), `finops-portal` (executive view of group cash position + overnight position + projected liquidity coverage ratio), `audit-chain` (per-account Merkle attestation for EOD posting integrity), `observability` (per-region latency targets + per-MT940 ingestion SLA).

The journey covers Sven's **8.5 hours** (May 20 14:48 CEST → 23:18 CEST; spanning 3 cut-off windows) of:

1. **payments** µservice — SWIFT MT940 EOD statement ingestion from 8 banks (47 accounts; intraday MT942 messages cycled throughout the day at 15-minute intervals; final EOD MT940 at each bank's cut-off)
2. **treasury** µservice — cash position computation per account → per entity → per currency → group EUR-reference; FX position vs hedge book delta-hedging compute; intercompany netting matrix; cash sweep to overnight investment ($340M target)
3. **finops-portal** µservice — executive view + LCR computation per Basel-III + overnight position dashboard + projected next-business-day funding need
4. **audit-chain** µservice — per-account Merkle attestation for the EOD-posted balance; immutable EOD record per IFRS-9 + IAS-21 audit requirement
5. **observability** µservice — per-MT940 ingestion latency SLA (target < 2 minutes from bank-side dispatch); per-region latency observation; SLO board

Microservices: `payments`, `treasury`, `finops-portal`, `audit-chain`, `observability`. Secondary: `identity` (Sven's passkey + YubiKey + CTP attestation), `tenancy` (BHI parent + 7 subsidiary tenants + 8 bank tenants), `messenger` (treasury operations team channel + Group Treasurer escalation channel), `notes` (Sven's working EOD playbook), `compliance` (8 pack overlays), `cell` (Stockholm tier-1 + per-region edge cells for bank-statement ingestion latency), `intelligence` (intraday FX-movement prediction + LCR projection).

## Why this journey matters

Sven Eriksson is **MASTER-ROSTER §5.8 row 342** — the canonical Treasury Operations Manager persona at a mid-cap-to-large industrial multinational. This persona covers ~12,400 CTP-certified Treasury Operations Manager-class roles globally (BLS 2024 code 13-2031 "Treasury Operations Manager"). EOD reconciliation is the highest-frequency, highest-volume treasury workflow; mis-posting destroys downstream audit + LCR + intercompany netting integrity.

The journey closes:

- **Critical-path row 224** (SWIFT MT940/MT942 ingestion across 8 banks + 47 accounts with deterministic per-statement parsing + audit anchoring)
- **Critical-path row 225** (Multi-currency cash position computation with FX exchange to group reference + per-entity + per-currency views)
- **Critical-path row 226** (FX hedge book delta-hedging vs underlying — rebalance trigger ±0.5%)
- **Critical-path row 227** (8-entity intercompany netting matrix computed + cleared via in-house bank)
- **Critical-path row 228** (Cash sweep to overnight money-market + deposits — $340M overnight position target)
- **Critical-path row 229** (Basel-III LCR computation + per-region cut-off ordering — Tokyo → SG → London → NY → Stockholm 3-cut)
- **Critical-path row 230** (Per-account Merkle attestation for EOD posting integrity supporting IFRS-9 + IAS-21 audit)

Hyperscaler benchmark: traditional treasury management systems (Kyriba + FIS + ION) handle the workflow but not natively as cross-µservice Cedar-permitted with per-account Merkle attestation. EOD reconciliation at this scale (47 accounts × 12 currencies × 12 ingestions/day = ~6,800 daily statement events) with sub-2-minute MT940 ingestion SLA is a hyperscaler benchmark that oyatie's [[amazon-cellular]] architecture targets.

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| `story.md` | Beat-by-beat May 20 14:48 → 23:18 CEST across 3 EOD cut-off windows | Stockholm late-spring climate + specific bank reference numbers + specific FX rates + specific MT940 message identifiers + specific entity-level position breakdown + LCR projection + overnight investment vehicle selection |
| `ux-flow.md` | Sven's treasury operations cockpit + cash position dashboard + FX delta-hedge panel + intercompany netting matrix + cash sweep + LCR + EOD attestation | Per-screen Cedar permit + per-cut-off-window indicator + MT940 ingestion freshness + per-account audit anchor |
| `handshake.md` | Per-µservice API; SWIFT MT940/MT942 ingestion + cash position compute + FX delta compute + netting + sweep + LCR + Merkle anchor | Each row names cut-off window + Cedar permit + audit class + MT940 message id |
| `integration-test-plan.md` | MT940 ingestion determinism + FX rate sourcing + netting algorithm + sweep allocation + LCR computation + Merkle anchor | Per-test seed + per-cut-off invariant + per-account Merkle invariant |
| `schemas/cedar-policy.cedar` | Treasury EOD Cedar policy | Treasury operator + group treasurer + bank ingestion + executive view permits; cut-off-window permits |
| `schemas/journey-messages.proto` | proto3 for all RPCs | Swedish + Norwegian + Danish + Finnish + German + English + Japanese + Korean + Chinese + Portuguese preservation |
| `schemas/openapi-treasury-eod.json` | OpenAPI for treasury EOD endpoints | Cash position + FX + netting + sweep + LCR + EOD attest |
| `schemas/openapi-swift-statement-ingestion.json` | OpenAPI for SWIFT MT940/MT942 ingestion | Per-bank + per-account + per-statement |
| `schemas/eod-state-machine.yaml` | 6-state EOD lifecycle | intraday → asia_cut → europe_cut → americas_cut → eod_posted → archive |

## The five primary microservices in scope

| µservice | Role | Critical-path row |
|---|---|---|
| `payments` | SWIFT MT940/MT942 statement ingestion from 8 banks | row 224 |
| `treasury` | Cash position + FX delta + intercompany netting + cash sweep | row 225, 226, 227, 228 |
| `finops-portal` | Executive view + Basel-III LCR + overnight position dashboard | row 229 |
| `audit-chain` | Per-account Merkle attestation for EOD posting integrity | row 230 |
| `observability` | Per-MT940 ingestion latency SLA + per-region SLO board | n/a (cross-cutting) |

## Secondary microservices touched

| µservice | Touch reason |
|---|---|
| `identity` | Sven's passkey + YubiKey + CTP attestation; Group Treasurer Annika's escalation principal; bank principal authentication |
| `tenancy` | `bohlin-hjelmqvist-industries-ab-parent` + 7 subsidiary tenants + 8 bank tenants |
| `messenger` | Treasury ops team channel + Group Treasurer escalation channel |
| `notes` | Sven's working EOD playbook |
| `compliance` | 8 pack overlays activated daily |
| `cell` | Stockholm tier-1 + per-region edge cells |
| `intelligence` | Intraday FX-movement prediction + LCR projection ML |

## Pack overlays (8 active)

| Pack | Activation reason | Pack ID |
|---|---|---|
| IFRS-9 | Financial instruments accounting | `pack-ifrs-9-financial-instruments` |
| IAS-21 | Effects of changes in foreign exchange rates | `pack-ias-21-fx-effects` |
| EMIR-Article-9 | EMIR Article 9 trade reporting for derivatives (FX swaps + forwards) | `pack-emir-article-9-trade-reporting` |
| Dodd-Frank-Title-VII | US Dodd-Frank Title VII for cross-border swap reporting | `pack-dodd-frank-title-vii-swap-reporting` |
| MiFID-II | MiFID II transaction reporting for in-house trading | `pack-mifid-ii-transaction-reporting` |
| BIS-Basel-III-LCR | Basel III Liquidity Coverage Ratio computation | `pack-bis-basel-iii-lcr` |
| ISO-20022-MX | ISO 20022 MX message format (transitioning from MT) | `pack-iso-20022-mx-2026` |
| IBOR-Transition-RFR | IBOR cessation + RFR transition (SOFR + ESTR + SARON + TONA) | `pack-ibor-transition-rfr-2027` |

## Regulatory anchors

1. **IFRS 9** — Financial Instruments: classification + measurement + impairment + hedge accounting
2. **IAS 21** — Effects of Changes in Foreign Exchange Rates
3. **IFRS 7** — Financial Instruments: Disclosures
4. **EMIR Article 9** — EU Regulation 648/2012 trade reporting for derivative transactions
5. **Dodd-Frank Title VII** — 7 U.S.C. § 6r + 15 U.S.C. § 78m(d) for swap reporting
6. **MiFID II** — Directive 2014/65/EU transaction reporting (Article 26)
7. **BIS Basel III** — Liquidity Coverage Ratio Standard (BCBS 238)
8. **CRD V / CRR II** — EU implementation of Basel III
9. **ISO 20022 MX** — payment + reporting message standard (replacing MT formats; coexistence period 2023-2025; full MX by 2026 for cross-border)
10. **ADR-0243 + ADR-0244 + ADR-0245 + ADR-0248 + ADR-0251 + ADR-0252 + ADR-0253 + ADR-0254 + ADR-0263**

## Cell + region matrix

| Cell | Role | Journey use |
|---|---|---|
| `eu-stockholm-tier-1-treasury-bhi` | BHI treasury primary cell | Sven's cockpit |
| `eu-frankfurt-tier-2-bank-statement-ingestion` | EU bank statement ingestion edge | Nordea + Handelsbanken statements |
| `us-east-tier-2-bank-statement-ingestion` | US bank statement ingestion edge | JPMorgan statements |
| `apac-tokyo-tier-2-bank-statement-ingestion` | Tokyo bank statement ingestion edge | Mizuho + SMBC statements |
| `apac-singapore-tier-2-bank-statement-ingestion` | SG bank statement ingestion edge | HSBC Asia statements |
| `latam-sao-paulo-tier-2-bank-statement-ingestion` | LATAM bank statement ingestion edge | Bradesco statements |
| `eu-stockholm-tier-1-worm-treasury-attest` | BHI WORM for EOD attestation | Per-account Merkle anchor + EOD posting |
| `external-transparency-log-batch-2027-05-20` | External transparency log | Daily EOD batch |

## Cedar permits (excerpt — full text in `schemas/cedar-policy.cedar`)

```cedar
permit (
    principal == User::"sven.eriksson@bohlin-hjelmqvist-industries-ab-parent",
    action in [
        Action::"treasury.cash_position_compute",
        Action::"treasury.fx_delta_hedge_compute",
        Action::"treasury.intercompany_netting_compute",
        Action::"treasury.cash_sweep_initiate",
        Action::"payments.swift_mt940_ingest_request",
        Action::"payments.swift_mt942_intraday_ingest_request",
        Action::"finops_portal.lcr_compute",
        Action::"finops_portal.overnight_position_dashboard_read",
        Action::"audit_chain.eod_account_anchor_emit",
        Action::"observability.eod_latency_slo_read"
    ],
    resource is EODSession
) when {
    principal.role_in_tenant("bohlin-hjelmqvist-industries-ab-parent") == "treasury_operations_manager" &&
    principal.ctp_attestation_id != "" &&
    context.passkey_assertion_present == true &&
    context.cut_off_window_open == true
};

permit (
    principal,
    action == Action::"treasury.cash_sweep_initiate",
    resource is CashSweep
) when {
    resource.target_overnight_amount_usd <= 500000000 &&  // $500M ceiling
    context.group_treasurer_co_sign_present == true &&
    context.sanctions_screening_on_destination_counterparty_clean == true
};
```

## Acceptance summary

| AC | Result expected |
|---|---|
| AC-J174-001 | SWIFT MT940 + MT942 ingestion across 47 accounts × 8 banks; daily volume ~6,800 statement events; per-statement ingestion < 2 minutes from bank-side dispatch; audit `EVT-J174-MT940-INGESTION-001` |
| AC-J174-002 | Cash position computed per account → per entity → per currency → group EUR reference; FX-rate sourcing from Refinitiv (primary) + Bloomberg (backup); audit `EVT-J174-CASH-POSITION-COMPUTED-002` |
| AC-J174-003 | FX hedge book delta-hedging compute vs 142 forwards + 28 cross-currency swaps; rebalance trigger ±0.5%; today's underlying movement +0.42% vs SEK-EUR + -0.31% vs SEK-USD (no rebalance triggered); audit `EVT-J174-FX-DELTA-HEDGE-003` |
| AC-J174-004 | 8-entity intercompany netting matrix computed; net positions in EUR; cleared via BHI in-house bank; audit `EVT-J174-INTERCOMPANY-NETTING-004` |
| AC-J174-005 | Cash sweep to overnight investment: $340M target ($248M to money-market funds across 3 vehicles + $92M to overnight deposits across 4 banks); audit `EVT-J174-CASH-SWEEP-005` |
| AC-J174-006 | Basel-III LCR computation: high-quality liquid assets (HQLA) / total net cash outflow over 30 days; ratio 1.42 (target ≥ 1.0); audit `EVT-J174-LCR-COMPUTED-006` |
| AC-J174-007 | Per-account Merkle attestation for EOD posting: 47 anchors emitted with external transparency log batch; audit `EVT-J174-MERKLE-EOD-007` |
| AC-J174-008 | Per-region cut-off ordering: Tokyo 11:00 CEST → SG 11:00 CEST → London 18:00 CEST → NY 23:00 CEST → Stockholm 23:30 CEST (night cut); audit `EVT-J174-CUT-OFF-ORDERING-008` |
| AC-J174-009 | Pack manifest assertion: 8 packs active + cross-validated; audit `EVT-J174-PACK-MANIFEST-009` |
| AC-J174-010 | Observability SLOs: MT940 ingestion P95 < 2min; cash position compute P95 < 18s; LCR compute P95 < 8s; per-region latency targets all met; audit `EVT-J174-OBSERVABILITY-SLO-010` |
| AC-J174-011 | Cedar deny coverage: 4 attempts to initiate sweep without Group Treasurer co-sign denied; 6 attempts to enumerate bank-statement metadata from non-treasury principals denied; audit `EVT-J174-CEDAR-DENY-COVERAGE-011` |
| AC-J174-012 | Swedish + Norwegian + Danish + Finnish + German + English + Japanese + Korean + Chinese + Portuguese + diacritic preservation byte-exact |

## Cross-references

- Persona dossier: `docs/personas/treasury-ops-sven-eriksson.md`
- MASTER-ROSTER §5.8 row 342
- Matrix §10 j174 recommendation
- Related: j106 (multi-currency cross-border payment), j120 (tenant treasury multi-currency FX hedge), j122 (vendor payment batch with tax withholding), j173 (multi-jurisdictional trust restructure $42M consolidation)
- Pack roster: `packs/ifrs-9-financial-instruments/`, `packs/ias-21-fx-effects/`, `packs/emir-article-9-trade-reporting/`, `packs/dodd-frank-title-vii-swap-reporting/`, `packs/mifid-ii-transaction-reporting/`, `packs/bis-basel-iii-lcr/`, `packs/iso-20022-mx-2026/`, `packs/ibor-transition-rfr-2027/`
- ADRs as listed above

## Stop condition

Journey complete when all 12 AC pass on the seeded fixture, EOD posting is sealed across 47 accounts with per-account Merkle attestation, the $340M overnight position is invested across 3 money-market funds + 4 overnight deposits, the LCR is computed and ≥ 1.0, the intercompany netting matrix is cleared via the in-house bank, the per-region cut-off ordering is preserved, and all 8 pack manifests are cross-validated.
