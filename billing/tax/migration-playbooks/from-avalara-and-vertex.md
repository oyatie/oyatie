# Migration playbook — Avalara AvaTax + Vertex O Series → Oyatie `cloud-billing-tax`

Audience: a tax/IT team using Avalara AvaTax for calculation + Returns + CertCapture, and Vertex O Series for some specific
verticals or jurisdictions. Goal: migrate to `cloud-billing-tax` without tax-period gap and with full filing continuity.

## Phase 0 — Inventory (Day 0…7)

### From Avalara

1. Export all configured companies + nexus state:
   ```bash
   curl -u "$AVALARA_USER:$AVALARA_PASS" \
     "https://rest.avatax.com/api/v2/companies" > avalara-companies.json
   curl -u "$AVALARA_USER:$AVALARA_PASS" \
     "https://rest.avatax.com/api/v2/companies/$COMPANY_ID/nexus" > avalara-nexus.json
   ```
2. Export tax-code mappings + custom codes:
   ```bash
   curl -u "$AVALARA_USER:$AVALARA_PASS" \
     "https://rest.avatax.com/api/v2/definitions/taxcodes" > avalara-taxcodes.json
   curl -u "$AVALARA_USER:$AVALARA_PASS" \
     "https://rest.avatax.com/api/v2/companies/$COMPANY_ID/items" > avalara-items.json
   ```
3. Export exemption certificates from CertCapture:
   ```bash
   certcapture-cli certificates export --company $COMPANY --format zip > avalara-certs.zip
   ```
4. Export historical returns (last 7 y for SOX-404):
   ```bash
   for year in 2019 2020 2021 2022 2023 2024 2025 2026; do
     curl -u "$AVALARA_USER:$AVALARA_PASS" \
       "https://rest.avatax.com/api/v2/companies/$COMPANY_ID/filings?year=$year" > "avalara-filings-$year.json"
   done
   ```

### From Vertex

1. Export tax-rate overrides + sourcing rules:
   ```bash
   vertex-cli rates export --company $COMPANY_CODE --format xml > vertex-rates.xml
   ```
2. Export taxpayer profiles + sourcing exceptions.
3. Export filing history.

## Phase 1 — Tenant + catalog provisioning (Day 7…14)

```bash
./bin/oya tax tenant register \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --tenant-class paid

./bin/oya tax catalog attach \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --catalog tax-codes-global-paid-v1
```

Translate Avalara tax codes → Oyatie codes:
```bash
./bin/oya tax migrate avalara-codes-to-oya \
  --input avalara-taxcodes.json \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --output code-mapping-acme.yaml
```

The mapping is best-effort; manual review is mandatory for codes outside Oyatie's catalog. Items that don't map register as
"unmapped" and require a `oya tax codes propose` ticket before cut-over.

## Phase 2 — Nexus + seller registration import (Day 14…21)

```bash
./bin/oya tax migrate avalara-nexus-to-seller-jurisdictions \
  --input avalara-nexus.json \
  --tenant oyatie.b2b.midmarket.acme-corp
```

For each jurisdiction in the Avalara nexus state, register the seller jurisdiction in `cloud-billing-tax` with the same
registration ID + effective dates. The migrator validates registration numbers against issuer databases where available.

## Phase 3 — Exemption certificate import (Day 21…35)

```bash
unzip avalara-certs.zip -d ./avalara-certs/
./bin/oya tax migrate certcapture-import \
  --input-dir ./avalara-certs/ \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --concurrency 4
```

Each cert is:
1. Re-OCR'd against the Oyatie OCR pipeline (provenance reset).
2. Cross-checked against the issuer DB (TX Comptroller, CA CDTFA, etc.) where the validity window includes the current date.
3. Re-encrypted under `cloud-kms` with AAD binding `(tenant_id, customer_id, jurisdiction)`.
4. Indexed by `(customer_id, jurisdiction)` for fast lookup at calculation time.

Expected throughput: ~6 certs/sec (OCR-bound); a 50,000-cert import completes in ~2.5 h.

## Phase 4 — Dual-calculation shadow phase (Day 35…56)

For two billing periods, route every transaction through BOTH Avalara/Vertex AND `cloud-billing-tax`. Only the legacy decision
is presented to the customer; the Oyatie decision is shadow-stored.

The SDK ships a dual-calculation wrapper:
```rust
use cloud_billing_tax_sdk::DualCalculation;

let tax = DualCalculation::builder()
    .primary(LegacyAvalaraClient::new(&avalara_account))
    .secondary(cloud_billing_tax_sdk::TaxClient::connect(cfg).await?)
    .strategy(DualCalculationStrategy::UseLegacy_ShadowRecordOya)
    .build()?;
```

Run divergence telemetry:
```bash
./bin/oya tax migrate divergence-report \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --since "24h ago" \
  --tolerance 0.001  # 0.1 % per-calc tolerance
```

Investigate any divergence > 0.1 %. Common sources: tax-code mismatch (often the migrator picks a slightly different Oyatie code),
nexus state drift (Avalara updated nexus, Oyatie didn't yet), rate-card lag (Oyatie 4-14 d behind authority bulletin).

## Phase 5 — Filing parity period (Day 56…112)

For at least one full filing period in each jurisdiction, generate the filing artefact in `cloud-billing-tax` and compare line-by-line
against the Avalara/Vertex filing for the same period.

```bash
./bin/oya tax filing-artefact generate \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --jurisdiction US-TX \
  --period 2026-05 \
  --format tx-comptroller-monthly \
  --shadow

./bin/oya tax migrate filing-compare \
  --avalara-filing avalara-tx-2026-05.xml \
  --filing filing/US-TX/2026-05/return.xml \
  --tolerance 0.01  # $0.01 line-item tolerance
```

Goal: zero non-rounding divergence. Any divergence triggers an investigation before the actual filing.

## Phase 6 — Cut-over (Day 112…140)

1. Flip the SDK from `UseLegacy_ShadowRecordOya` to `UseOya_ShadowRecordLegacy`.
2. Submit the next filing period from `cloud-billing-tax` directly (no shadow).
3. Avalara/Vertex continue to receive transactions in shadow mode for 1 more period (safety net).
4. Cancel Avalara CertCapture renewal (you're using `cloud-billing-tax` cert storage).

## Phase 7 — Decommission (Day 140+)

After 60 d clean run:
- Cancel Avalara AvaTax + Returns subscription (saves $7,800/mo at typical mid-market scale).
- Cancel Vertex subscription if applicable.
- Archive Avalara historical data export (7 y for SOX-404 retention).

## Rollback strategy

Within Phase 6 shadow safety net:
- Flip the SDK back to `UseLegacy_ShadowRecordOya`.
- Submit the next filing from Avalara/Vertex; `cloud-billing-tax` shadow continues.
- Cost is zero.

After Phase 7 decommission:
- Reactivate Avalara subscription (vendor can restore from snapshots within 24-48 h).
- Re-import the current period's transactions to Avalara via `avalara-cli import`.
- Plan 2-4 weeks of dual-running before re-cutting back.

## What you gain

- 50-72 % TCO reduction vs Avalara/Vertex/Sovos at mid-market scale.
- 3-7× lower calculation latency.
- In-process Cedar tax engine at paid (vendors out-of-process).
- AAD-bound exemption certificate encryption (vendors don't).
- BLAKE3 audit chain.
- OECD BEPS Pillar Two integration.
- EU ViDA 2030 ready.
- HTTP/3 QUIC, per-tenant compliance pack overlays.

## What you give up

- Avalara's 22,000-code catalog breadth (Oyatie 9,800; niche industries lighter).
- Avalara CertCapture's 15 years of issuer-DB integration maturity.
- Sovos's 50+ country e-invoice breadth (Oyatie 30-50).
- Stripe Tax's drop-in ergonomics for Stripe-native businesses.
- Avalara's marketplace UX (Oyatie surfaces via `workflow-studio`).
