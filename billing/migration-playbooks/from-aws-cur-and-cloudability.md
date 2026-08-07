# Migration playbook — AWS CUR + Apptio Cloudability → Oyatie `cloud-billing`

Audience: a FinOps team using AWS Cost & Usage Reports as the source of truth and Apptio Cloudability as the chargeback +
showback layer. Goal: migrate to `cloud-billing` with zero invoice-issuance gap and complete historical preservation.

## Phase 0 — Inventory (Day 0…7)

### From AWS CUR

1. Confirm CUR is enabled in standard or 2.0 format:
   ```bash
   aws ce describe-cost-allocation-tag-rules
   aws billingconductor list-billing-groups
   ```
2. Download trailing 13 months of CUR (Parquet preferred):
   ```bash
   aws s3 sync s3://acme-cur-bucket/CUR/year=2025/ ./cur-historical/2025/
   aws s3 sync s3://acme-cur-bucket/CUR/year=2026/ ./cur-historical/2026/
   ```
3. Pull existing Cost Categories + Cost Allocation Tags + Billing Conductor groups.

### From Apptio Cloudability

1. Export chargeback rules + cost-center definitions:
   ```bash
   cloudability-cli export --resource cost-centers --format json > cloudability-cost-centers.json
   cloudability-cli export --resource chargeback-rules --format json > cloudability-chargeback-rules.json
   ```
2. Export anomaly-detection thresholds + savings recommendations history.
3. Export user/group access map (will translate to Cedar policies on `cloud-billing`).

## Phase 1 — Tenant + rate-card provisioning (Day 7…14)

```bash
./bin/oya billing tenant register \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --tenant-class paid \
  --billing-currency USD \
  --invoice-cadence monthly
```

Create a rate card mirroring your AWS effective rates (use the historical CUR to back-solve):
```bash
./bin/oya billing rate-card create \
  --rate-card-id oya-rate-card-acme-corp-v1 \
  --effective-from 2026-01-01 \
  --currency USD \
  --items-file acme-effective-rates.yaml
```

`acme-effective-rates.yaml` derives per-resource-kind rates from CUR `lineItem/UnblendedRate` aggregations. For passthrough items
(EC2 hours, S3 GB-mo, etc.), the rate is the effective-blended-rate-after-RIs from CUR.

## Phase 2 — Historical backfill (Day 14…35)

Ingest 13 months of historical CUR into `cloud-billing`:
```bash
./bin/oya billing ingest aws-cur \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --cur-path ./cur-historical/ \
  --year-range 2025-2026 \
  --concurrency 8 \
  --resume-cursor /var/lib/oya/migrate/acme-cur-backfill.cursor
```

This populates the raw ledger + rolled-up aggregates with historical data. A typical mid-market 13-month backfill is 800 M
line items, completing in ~36 h at 6,200 lines/sec/worker × 8 workers. The operation is resumable.

Verify totals match AWS:
```bash
./bin/oya billing reconcile aws-cur \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --period 2025-09 \
  --tolerance 0.05  # 5 cents per $1k tolerance
```

Expected: `reconciliation_diff: 0.001 %, status: OK`.

## Phase 3 — Cost-center + attribution rule translation (Day 35…56)

Translate Cloudability cost-centers + chargeback rules:
```bash
./bin/oya billing migrate cloudability-to-cost-centers \
  --input cloudability-cost-centers.json \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --output cost-centers-acme.yaml

./bin/oya billing migrate cloudability-to-attribution-rules \
  --input cloudability-chargeback-rules.json \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --output attribution-rules-acme.yaml
```

Lint + push:
```bash
./bin/oya billing attribution-rule lint --file attribution-rules-acme.yaml
./bin/oya billing attribution-rule push --tenant oyatie.b2b.midmarket.acme-corp --file attribution-rules-acme.yaml
```

Re-attribute historical usage:
```bash
./bin/oya billing reattribute-historical \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --period-range 2025-04..2026-04
```

This rewrites the historical aggregate rows with the new cost-center attribution. The raw ledger is immutable.

## Phase 4 — Dual-invoicing phase (Day 56…84)

For two billing periods, generate invoices on BOTH platforms and compare. Don't bill the customer twice — `cloud-billing` invoices
are in "shadow" mode (issued but flagged `shadow=true`).

```bash
./bin/oya billing invoice generate \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --period 2026-05 \
  --mode shadow
```

Compare side-by-side:
```bash
./bin/oya billing compare cloudability \
  --tenant oyatie.b2b.midmarket.acme-corp \
  --period 2026-05 \
  --cloudability-export cloudability-2026-05-invoice.csv
```

Expected: `subtotal_diff: ≤ 0.1 %`, `cost-center-attribution-match: ≥ 99 %`.

Investigate any divergence > 0.1 %. Common sources: tag drift, RI / Savings Plan amortization mismatch, currency rounding.

## Phase 5 — Cut-over (Day 84…112)

1. Switch `cloud-billing` invoice mode from `shadow` to `live`:
   ```bash
   ./bin/oya billing invoice-mode set --tenant oyatie.b2b.midmarket.acme-corp --mode live
   ```
2. Disable Cloudability invoice generation; keep ingestion for 30 d as a safety net.
3. Update your AR/billing operations to consume `cloud-billing` invoices (PDF + JSON via API).
4. Switch chargeback exports to your ERP (NetSuite/SAP/etc.) to come from `cloud-billing` via the ERP adapter.

## Phase 6 — Decommission overlaps (Day 112+)

After 30 d clean run on `cloud-billing`:
- Cancel Cloudability subscription (saves ~$8,000/mo).
- Keep AWS CUR enabled (it's the upstream data source; `cloud-billing` continues to ingest it).
- Archive Cloudability data export (keep 7 y for SOX-404).

## Rollback strategy

Within Phase 4 dual-invoicing:
- Just stop generating `cloud-billing` invoices; Cloudability is still authoritative.
- Cost is zero.

After Phase 5 cut-over:
- Re-enable Cloudability invoice generation.
- Flag a window where invoices generated by `cloud-billing` are credit-memo'd and re-issued by Cloudability.
- Pause `cloud-billing` for the affected tenant + investigate divergence.

After 30 d clean run: rolling back is a 4-6 week project (reconstitute Cloudability historical data + reverse the AR ops change).

## What you gain

- 49-65 % TCO reduction vs Cloudability + AWS CUR combo.
- Second-fresh metering (vs Cloudability ≥ 5 min lag, AWS CUR 24 h).
- FOCUS 1.1 native (vs Cloudability post-ingest mapping).
- OECD BEPS Pillar Two export.
- Multi-axis chargeback beyond Cloudability's dimension model.
- Cedar-gated credit memos + BLAKE3 audit chain.
- Multi-currency with ECB rate locks.
- HTTP/3 QUIC + per-tenant compliance pack overlays.

## What you give up

- Cloudability's mature savings-recommendation engine (deferred to `finops-portal` in Oyatie; less mature at v1).
- Cloudability's vendor-billing ingestion breadth (Snowflake, Databricks, Datadog, etc.) — Oyatie ships AWS+GCP+Azure+Stripe at v1.
- Public Cloudability UX (Oyatie's surface is `finops-portal`, currently less polished).
- Marketplace integrations (Slack, ServiceNow, Jira) — partial in Oyatie via `comms-*` µservices.
