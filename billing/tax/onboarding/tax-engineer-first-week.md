# Tax Engineer — First Week on `cloud-billing-tax`

Audience: a tax/indirect-tax engineer with Avalara AvaTax + Vertex + Stripe Tax integration experience joining the
`cloud-billing-tax-*` lane. Goal: by Friday EOD you can calculate tax on a multi-jurisdiction transaction, upload an exemption
certificate, track nexus, and generate a filing artefact.

## Day 1 — read before touching

- `docs/decisions/ADR-0702-identity-authz-live-apex.md` — every calculation carries `tenant_id`.
- `docs/decisions/ADR-0701-monorepo-capability-live-apex.md` — `cloud-billing-tax` is substrate beneath `cloud-billing`.
- OECD VAT/GST International Guidelines (vendored at `vendor/oecd-vat-gst-guidelines/SUMMARY.md`).
- EU VAT in the Digital Age (ViDA) Directive — particularly the e-invoicing pillar effective 2030.
- US Wayfair v. South Dakota (2018) ruling — establishes the economic-nexus framework.
- `microservices/cloud-billing-tax/tenant_class adoption record` — the two tenant classes + jurisdiction coverage.

Clone:
```bash
./bin/oya git worktree-add --base dev --branch onboarding/$USER-tax-week1 .worktrees/$USER-tax-week1
cd .worktrees/$USER-tax-week1
```

## Day 2 — bring up a loopback tax cell

```bash
make dev-cell.up CELL=tax-loopback-1 PROFILE=cloud-billing-tax-dev
make dev-tenant.create T=oyatie.b2b.smb.acme-software TENANT_CLASS=paid
make dev-tax-codes.attach T=oyatie.b2b.smb.acme-software CATALOG=tax-codes-multiregion-paid-v1
```

Calculate tax on a single line item — a SaaS subscription sold from a CA seller to a TX customer:
```bash
./bin/oya tax calculate \
  --tenant oyatie.b2b.smb.acme-software \
  --calculation-id "$(uuidgen)" \
  --seller-location '{"jurisdiction":"US-CA","postal_code":"94107","city":"San Francisco"}' \
  --buyer-location '{"jurisdiction":"US-TX","postal_code":"78701","city":"Austin"}' \
  --line '{"tax_code":"SW054001","amount":1200.00,"quantity":1,"description":"Acme SaaS Pro plan, 1 user, monthly"}'
```

Expected output:
```
calculation_id      : cal-2026-05-20-...
ship_from_jurisdiction: US-CA
ship_to_jurisdiction  : US-TX
nexus_status          : nexus-present (acme has TX nexus per tenant config)
tax_lines:
  - jurisdiction: US-TX (state)              rate: 6.25 %   amount: $75.00
  - jurisdiction: US-TX (Travis County)      rate: 0.00 %   amount: $0.00
  - jurisdiction: US-TX (City of Austin)     rate: 1.00 %   amount: $12.00
  - jurisdiction: US-TX (Capital Metro RTA)  rate: 1.00 %   amount: $12.00
total_tax           : $99.00
effective_rate      : 8.25 %
rate_card_version   : tax-codes-multiregion-paid-v1@2026-05-01
audit_chain_event   : ce-2026-05-20T09:21:11Z-…
```

## Day 3 — exemption certificates

Upload an exemption certificate (resale certificate from a wholesaler buyer):
```bash
./bin/oya tax exemption-cert upload \
  --tenant oyatie.b2b.smb.acme-software \
  --customer-id "cust-7741" \
  --jurisdiction "US-CA" \
  --certificate-type "resale" \
  --reseller-permit-number "SR Y AAB 12-345678" \
  --issuer "California Department of Tax and Fee Administration" \
  --valid-from "2026-01-01" \
  --valid-through "2027-12-31" \
  --document /tmp/resale-cert-acme.pdf
```

Behind the scenes:
1. The PDF is OCR'd; key fields are extracted + cross-checked against the CDTFA Reseller Permit Lookup API.
2. The doc is encrypted under `cloud-kms` with AAD `(tenant_id=oyatie.b2b.smb.acme-software, customer_id=cust-7741, jurisdiction=US-CA)`.
3. A `cloud_billing_tax.exemption_cert.uploaded` audit event is anchored.

Recalculate tax for a sale to this customer:
```bash
./bin/oya tax calculate \
  --tenant oyatie.b2b.smb.acme-software \
  --calculation-id "$(uuidgen)" \
  --customer-id "cust-7741" \
  --seller-location '{"jurisdiction":"US-CA"}' \
  --buyer-location '{"jurisdiction":"US-CA"}' \
  --line '{"tax_code":"P0000000","amount":10000.00,"quantity":50,"description":"wholesale t-shirts"}'
```

Expected: `total_tax: $0.00, exemption_applied: certs:cust-7741/US-CA/resale`.

## Day 4 — nexus tracking

The paid tenant_class auto-tracks economic nexus thresholds per jurisdiction. Add a few months of synthetic sales:
```bash
./bin/oya tax simulate-sales \
  --tenant oyatie.b2b.smb.acme-software \
  --to-jurisdiction US-CO \
  --amount-usd 95000 \
  --transactions 180 \
  --over-months 3
```

Check nexus state:
```bash
./bin/oya tax nexus show --tenant oyatie.b2b.smb.acme-software --jurisdiction US-CO
```

Expected:
```
jurisdiction      : US-CO
threshold         : $100,000 OR 200 transactions (CO economic nexus)
ytd_sales         : $95,000 (95 % of threshold)
ytd_transactions  : 180 (90 % of threshold)
projected_breach  : 2026-08-15 (ML-extrapolation; available at Paid; Paid shows latest historical only)
status            : approaching-threshold (90 % alert tier)
grace_window      : 30 d after breach
recommended_action: prepare to register CO Department of Revenue
```

## Day 5 — generate a filing artefact

Generate a US-CA monthly sales tax return XML for May 2026:
```bash
./bin/oya tax filing-artefact generate \
  --tenant oyatie.b2b.smb.acme-software \
  --jurisdiction US-CA \
  --period 2026-05 \
  --format ca-cdtfa-401-form \
  --output filing/US-CA/2026-05/return.xml
```

Output (truncated):
```
form_name        : CDTFA-401-A (State, Local, and District Sales and Use Tax Return)
period           : 2026-05
gross_sales      : $128,400.00
deductions       : $14,200.00 (resale exemptions $12,000; nontaxable $2,200)
net_taxable      : $114,200.00
state_tax_due    : $8,565.00 (7.25 %)
district_tax_due : $1,142.00 (varies by district)
total_due        : $9,707.00
pre_file_reconciliation_ok: true
output_path      : filing/US-CA/2026-05/return.xml
```

Generate the EU VAT MOSS return (if the tenant has EU customers):
```bash
./bin/oya tax filing-artefact generate \
  --tenant oyatie.b2b.smb.acme-software \
  --jurisdiction EU-OSS-Union \
  --period 2026-Q2 \
  --format eu-vat-moss-xml \
  --output filing/EU-OSS/2026-Q2/moss.xml
```

## What "done with week 1" means

- [ ] You can recite the two tenant classes and which jurisdiction categories each unlocks.
- [ ] You calculated multi-jurisdiction tax for a US interstate sale.
- [ ] You uploaded an exemption certificate + saw it suppress tax on a subsequent sale.
- [ ] You walked an automatic economic-nexus tracking flow.
- [ ] You generated a CA CDTFA-401 filing artefact and an EU VAT MOSS XML.
- [ ] You read ADR-0244 + ADR-0245 + OECD VAT/GST guidelines + Wayfair.

## Rookie traps

1. **Using the wrong tax code.** SW054001 (SaaS) ≠ DC080100 (downloadable software); the rate differs by jurisdiction.
   Always validate the code with `oya tax codes lookup --tax-code <code>`.
2. **Ignoring the nexus grace window.** Economic nexus typically requires registration within 30-60 d of threshold breach;
   skipping leads to back-tax + penalties. The CLI shows `recommended_action` — act on it.
3. **Hard-coding rates in app code.** Rates change quarterly; never cache rates client-side. Use `oya tax calculate` per
   transaction.
4. **Skipping exemption certificate validation.** Self-asserted exemption with no certificate fails on audit. Upload + OCR-validate.
5. **Filing without pre-reconciliation.** The generator runs a pre-file reconciliation against `cloud-billing` raw ledger. Never
   `--skip-reconciliation` in production.
6. **Mixing currencies in tax calculation.** Calculate in the line currency; never FX before tax. FX happens in `cloud-billing`.
