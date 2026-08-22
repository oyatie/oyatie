# Tutorial — Meter usage, attribute to cost centers, issue invoice, export FOCUS 1.1

Goal: emit synthetic usage from two µservices, attribute it to two cost centers, generate a multi-currency monthly invoice,
and produce a FOCUS 1.1 Parquet export. End-to-end on a loopback `cloud-billing` cell.

Pre-reqs:
- Loopback billing cell: `make dev-cell.up CELL=billing-loopback-1 PROFILE=cloud-billing-dev`
- Tenant: `make dev-tenant.create T=oyatie.b2b.smb.acme-software TENANT_CLASS=paid CURRENCY=EUR`
- Rate card: `make dev-rate-card.attach T=oyatie.b2b.smb.acme-software CARD=rate-card-smb-paid-v1`
- `jq`, `python3` (for FOCUS validation), `parquet-tools` on PATH.

## Step 1 — configure cost centers

```bash
./bin/oya billing cost-center create \
  --tenant oyatie.b2b.smb.acme-software \
  --code engineering --display-name "Engineering" --owner-email cto@acme-software.io

./bin/oya billing cost-center create \
  --tenant oyatie.b2b.smb.acme-software \
  --code product --display-name "Product" --owner-email vpp@acme-software.io
```

## Step 2 — configure attribution rules

```bash
./bin/oya billing attribution-rule create \
  --tenant oyatie.b2b.smb.acme-software \
  --rule-name "k8s-ns-product" \
  --priority 100 \
  --match '{"resource_kind":"cloud_compute_k8s.pod_minute","dimensions.namespace":"ns:acme-product-*"}' \
  --target "cost_center=product"

./bin/oya billing attribution-rule create \
  --tenant oyatie.b2b.smb.acme-software \
  --rule-name "default-engineering" \
  --priority 9999 \
  --match '{"resource_kind":"cloud_compute_k8s.pod_minute"}' \
  --target "cost_center=engineering"
```

## Step 3 — emit usage events

Simulate a day's worth of pod-minute usage from two namespaces:
```bash
# Engineering namespace, 5000 pod-minutes
for i in $(seq 1 5000); do
  ./bin/oya billing meter emit \
    --tenant oyatie.b2b.smb.acme-software \
    --event-id "$(uuidgen)" \
    --resource-kind cloud_compute_k8s.pod_minute \
    --resource-id "ns:acme-engineering/pod:webapp-7d4f-${i}" \
    --quantity 1 --unit minute \
    --timestamp "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --dimensions '{"region":"eu-west-1","pack":"none","namespace":"ns:acme-engineering"}'
done &

# Product namespace, 3000 pod-minutes
for i in $(seq 1 3000); do
  ./bin/oya billing meter emit \
    --tenant oyatie.b2b.smb.acme-software \
    --event-id "$(uuidgen)" \
    --resource-kind cloud_compute_k8s.pod_minute \
    --resource-id "ns:acme-product-discovery/pod:svc-${i}" \
    --quantity 1 --unit minute \
    --timestamp "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --dimensions '{"region":"eu-west-1","pack":"none","namespace":"ns:acme-product-discovery"}'
done &
wait
```

Dev profile sends in ≤ 60 s with batching.

## Step 4 — verify attribution

```bash
./bin/oya billing usage query \
  --tenant oyatie.b2b.smb.acme-software \
  --since "1h ago" \
  --group-by cost_center
```

Expected:
```
cost_center=engineering  resource_kind=cloud_compute_k8s.pod_minute  quantity=5000.0
cost_center=product      resource_kind=cloud_compute_k8s.pod_minute  quantity=3000.0
```

## Step 5 — trigger a synthetic month-end close

Dev profile fast-forwards time:
```bash
./bin/oya billing close --tenant oyatie.b2b.smb.acme-software --period 2026-05 --fast-forward
```

Expected:
```
period_close      : 2026-05-01 → 2026-05-31
events_aggregated : 8000
aggregation_passes: 3 (per-minute → per-hour → per-day → per-period)
invoice_generated : inv-2026-05-acme-software
audit_chain_event : ce-2026-06-01T08:00:01Z-…
```

## Step 6 — inspect the invoice

```bash
./bin/oya billing invoice show \
  --tenant oyatie.b2b.smb.acme-software \
  --period 2026-05 \
  --format json | jq .
```

Expected (relevant portion):
```json
{
  "invoice_id": "inv-2026-05-acme-software",
  "currency": "EUR",
  "fx_rate_used": {"USD_to_EUR": 0.9214, "source": "ECB-reference-rates-daily", "as_of": "2026-06-01T07:00:00Z"},
  "subtotal_usd": 113.60,
  "subtotal": 104.67,
  "line_items": [
    {
      "cost_center": "engineering",
      "resource_kind": "cloud_compute_k8s.pod_minute",
      "quantity": 5000.0,
      "unit_rate_usd": 0.0142,
      "amount_usd": 71.00,
      "amount": 65.42
    },
    {
      "cost_center": "product",
      "resource_kind": "cloud_compute_k8s.pod_minute",
      "quantity": 3000.0,
      "unit_rate_usd": 0.0142,
      "amount_usd": 42.60,
      "amount": 39.25
    }
  ]
}
```

## Step 7 — export to FOCUS 1.1 Parquet

```bash
./bin/oya billing focus-export \
  --tenant oyatie.b2b.smb.acme-software \
  --period 2026-05 \
  --format parquet \
  --output focus-2026-05-acme.parquet
```

Validate the schema:
```bash
./bin/oya billing focus-validate --file focus-2026-05-acme.parquet
```

Expected:
```
focus_version     : 1.1
rows              : 8000
required_columns  : ChargeCategory, ChargeClass, BilledCost, EffectiveCost, ListCost, ConsumedQuantity, ServiceName, ... (all present)
schema_errors     : 0
extension_columns : tenant_id, cost_center, pack_id
```

Inspect via `parquet-tools`:
```bash
parquet-tools head -n 2 focus-2026-05-acme.parquet
```

Output (truncated):
```
ChargeCategory=Usage, ChargeClass=Standard, ServiceName=cloud_compute_k8s, ConsumedQuantity=1.0, ConsumedUnit=minute, BilledCost=0.0131, BillingCurrency=EUR, tenant_id=oyatie.b2b.smb.acme-software, cost_center=engineering
```

## Step 8 — deliver to tenant warehouse (optional)

If the tenant has configured `cloud-storage` integration:
```bash
./bin/oya billing focus-deliver \
  --tenant oyatie.b2b.smb.acme-software \
  --period 2026-05 \
  --target s3://acme-software-finops-warehouse/focus/2026-05/
```

This writes the Parquet to the tenant's bucket under their KMS-encryption, using the per-tenant Cedar-gated principal.

## What you just demonstrated

- Idempotent metering with UUID v7 event IDs.
- Two-rule attribution engine with priority ordering.
- ECB-rate-locked multi-currency invoicing (no silent revaluation).
- Audit-chain-anchored period close.
- FOCUS 1.1 Parquet export with extension columns for Oyatie metadata.
- Tenant-side delivery via cross-µservice integration with `cloud-storage`.
