# Tutorial — Calculate multi-jurisdiction tax + file a quarterly return

Goal: walk a multi-line cross-border B2B + B2C transaction set, calculate tax, generate an EU OSS VAT return for the period,
and file via the loopback simulator. End-to-end on a loopback `cloud-billing-tax` cell.

Pre-reqs:
- Loopback tax cell: `make dev-cell.up CELL=tax-loopback-1 PROFILE=cloud-billing-tax-dev`
- Tenant: `make dev-tenant.create T=oyatie.b2b.smb.acme-software TENANT_CLASS=paid`
- Tax catalog: `make dev-tax-codes.attach T=oyatie.b2b.smb.acme-software CATALOG=oya-tax-codes-multiregion-paid-v1`
- Seller registered in: US-CA (home), DE (EU OSS Union scheme registered as IE-OSS for SaaS B2C across EU).

## Step 1 — register seller jurisdictions

```bash
./bin/oya tax seller-jurisdiction register \
  --tenant oyatie.b2b.smb.acme-software \
  --jurisdiction US-CA \
  --registration-id "12-345678" \
  --registration-effective 2024-01-01

./bin/oya tax seller-jurisdiction register \
  --tenant oyatie.b2b.smb.acme-software \
  --jurisdiction EU-OSS-Union \
  --registration-id "IE9123456A" \
  --registration-effective 2024-04-01 \
  --home-member-state IE
```

## Step 2 — calculate tax for a batch of transactions

`transactions.json`:
```json
[
  {
    "calculation_id": "cal-tut-001",
    "seller_location": {"jurisdiction": "US-CA", "postal_code": "94107", "city": "San Francisco"},
    "buyer_location": {"jurisdiction": "US-TX", "postal_code": "78701", "city": "Austin"},
    "buyer_type": "business",
    "transaction_currency": "USD",
    "lines": [
      {"tax_code": "SW054001", "description": "Acme SaaS Pro plan", "amount": 1200.00, "quantity": 1}
    ]
  },
  {
    "calculation_id": "cal-tut-002",
    "seller_location": {"jurisdiction": "US-CA", "postal_code": "94107"},
    "buyer_location": {"jurisdiction": "DE", "postal_code": "10115", "city": "Berlin"},
    "buyer_type": "consumer",
    "transaction_currency": "EUR",
    "evidence": ["ip:5.1.2.3 (DE)", "billing:DE", "payment:DE"],
    "lines": [
      {"tax_code": "SW054001", "description": "Acme SaaS Pro plan", "amount": 79.00, "quantity": 1}
    ]
  },
  {
    "calculation_id": "cal-tut-003",
    "seller_location": {"jurisdiction": "US-CA"},
    "buyer_location": {"jurisdiction": "FR", "postal_code": "75001"},
    "buyer_type": "business",
    "buyer_vat_number": "FR12345678901",
    "transaction_currency": "EUR",
    "lines": [
      {"tax_code": "SW054001", "description": "Acme SaaS Pro plan", "amount": 999.00, "quantity": 1}
    ]
  },
  {
    "calculation_id": "cal-tut-004",
    "seller_location": {"jurisdiction": "US-CA"},
    "buyer_location": {"jurisdiction": "KR", "postal_code": "06236", "city": "Seoul"},
    "buyer_type": "business",
    "buyer_brn": "123-45-67890",
    "transaction_currency": "KRW",
    "lines": [
      {"tax_code": "SW054001", "description": "Acme SaaS Pro plan", "amount": 1500000, "quantity": 1}
    ]
  }
]
```

Calculate:
```bash
./bin/oya tax calculate-batch \
  --tenant oyatie.b2b.smb.acme-software \
  --input transactions.json \
  --output results.json
```

Inspect:
```bash
jq '.[] | {calculation_id, total_tax, effective_rate, tax_lines: [.tax_lines[] | {jurisdiction, rate, amount}]}' results.json
```

Expected (truncated):
```json
{
  "calculation_id": "cal-tut-001",
  "total_tax": 99.00,
  "effective_rate": 0.0825,
  "tax_lines": [
    {"jurisdiction": "US-TX state", "rate": 0.0625, "amount": 75.00},
    {"jurisdiction": "US-TX Austin city", "rate": 0.01, "amount": 12.00},
    {"jurisdiction": "US-TX Travis County RTA", "rate": 0.01, "amount": 12.00}
  ]
}
{
  "calculation_id": "cal-tut-002",
  "total_tax": 15.01,
  "effective_rate": 0.19,
  "tax_lines": [
    {"jurisdiction": "DE (EU OSS scheme; B2C cross-border digital)", "rate": 0.19, "amount": 15.01}
  ]
}
{
  "calculation_id": "cal-tut-003",
  "total_tax": 0,
  "effective_rate": 0,
  "tax_lines": [
    {"jurisdiction": "FR (EU B2B reverse charge; buyer VAT FR12345678901 verified via VIES)", "rate": 0, "amount": 0}
  ],
  "buyer_obligation": "reverse-charge: buyer self-accounts at FR rate 20 %"
}
{
  "calculation_id": "cal-tut-004",
  "total_tax": 150000,
  "effective_rate": 0.10,
  "tax_lines": [
    {"jurisdiction": "KR VAT (B2B digital services, reverse-charge not applicable; non-resident registered seller pays)", "rate": 0.10, "amount": 150000}
  ]
}
```

## Step 3 — verify EU OSS aggregation

Confirm the DE B2C sale was classified under the EU OSS Union scheme:
```bash
./bin/oya tax oss-aggregate \
  --tenant oyatie.b2b.smb.acme-software \
  --period 2026-Q2 \
  --scheme eu-oss-union
```

Expected:
```
scheme              : EU OSS Union Scheme
home_member_state   : IE
period              : 2026-Q2
applicable_lines    : 1 (cal-tut-002)
country_breakdown   :
  DE  €79.00 net  €15.01 VAT  rate 19 %
total_vat           : €15.01
filing_due_date     : 2026-07-31
```

## Step 4 — generate the EU OSS VAT XML

```bash
./bin/oya tax filing-artefact generate \
  --tenant oyatie.b2b.smb.acme-software \
  --jurisdiction EU-OSS-Union \
  --period 2026-Q2 \
  --format eu-vat-moss-xml \
  --output filing/EU-OSS/2026-Q2/moss.xml
```

Output:
```
form               : EU OSS Union Scheme VAT Return (Council Directive 2006/112/EC Annex II)
period             : 2026-Q2 (2026-04-01 → 2026-06-30)
home_ms            : IE (Revenue Online Service submission target)
country_lines      : 1
total_vat_eur      : €15.01
pre_file_reconciliation_ok: true
xml_validation     : passes XSD https://ec.europa.eu/taxation_customs/vies/services/checkVatService
xml_file_size      : 4.2 KB
output_path        : filing/EU-OSS/2026-Q2/moss.xml
```

## Step 5 — file via loopback simulator

The dev profile bundles a loopback "Revenue Online Service" simulator:
```bash
./bin/oya tax filing-submit \
  --tenant oyatie.b2b.smb.acme-software \
  --jurisdiction EU-OSS-Union \
  --period 2026-Q2 \
  --xml-file filing/EU-OSS/2026-Q2/moss.xml \
  --target loopback
```

Expected:
```
submission_id     : sub-2026-05-20-...
gateway           : loopback-revenue-online-service
submission_status : Accepted
acknowledgement   : ROS-ACK-2026-Q2-OSS-...
audit_chain_event : ce-2026-05-20T15:42:18Z-…
```

## Step 6 — file the US-TX state return for the same period

```bash
./bin/oya tax filing-artefact generate \
  --tenant oyatie.b2b.smb.acme-software \
  --jurisdiction US-TX \
  --period 2026-05 \
  --format tx-comptroller-monthly \
  --output filing/US-TX/2026-05/return.xml

./bin/oya tax filing-submit \
  --tenant oyatie.b2b.smb.acme-software \
  --jurisdiction US-TX \
  --period 2026-05 \
  --xml-file filing/US-TX/2026-05/return.xml \
  --target loopback
```

## Step 7 — view the audit-chain trail

```bash
./bin/oya audit-chain query \
  --tenant oyatie.b2b.smb.acme-software \
  --kind 'cloud_billing_tax.*' \
  --since "2h ago"
```

You should see (at minimum):
- `cloud_billing_tax.calculation.completed` (×4)
- `cloud_billing_tax.oss_aggregate.computed`
- `cloud_billing_tax.filing_artefact.generated` (×2)
- `cloud_billing_tax.filing.submitted` (×2)
- `cloud_billing_tax.filing.acknowledged` (×2)

## What you just demonstrated

- Multi-jurisdiction tax calculation including US (state + local) and EU OSS Union scheme.
- B2B reverse-charge handling (VIES validation of the FR VAT number).
- Korea VAT for foreign non-resident registered sellers.
- EU OSS quarterly VAT MOSS XML generation conformant with Council Directive 2006/112/EC.
- Pre-file reconciliation against `cloud-billing` raw ledger.
- Filing submission with gateway acknowledgement.
- BLAKE3 audit-chain anchoring across the full calculate → file → acknowledge lifecycle.
