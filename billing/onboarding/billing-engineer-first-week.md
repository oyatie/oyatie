# Billing Engineer — First Week on `cloud-billing`

Audience: a finance/FinOps engineer with AWS Cost & Usage Reports + Stripe + ERP integration experience joining the
`cloud-billing-*` lane. Goal: by Friday EOD you can emit a usage event, generate an invoice, walk a chargeback,
and produce a FOCUS 1.1 export.

## Day 1 — read before touching

- `docs/decisions/ADR-0702-identity-authz-live-apex.md` — every usage row carries `tenant_id`.
- `docs/decisions/ADR-0701-monorepo-capability-live-apex.md` — `cloud-billing` is substrate; `finops-portal` is product.
- The FinOps Foundation FOCUS 1.1 spec (vendored at `vendor/focus-1.1/SCHEMA.md`).
- `microservices/cloud-billing/tenant_class adoption record` — the two tenant classes.
- Re-read on day 2 if anything is unclear: ECB reference rate methodology + OECD BEPS Pillar Two GloBE rules.

Clone:
```bash
./bin/oya git worktree-add --base dev --branch onboarding/$USER-billing-week1 .worktrees/$USER-billing-week1
cd .worktrees/$USER-billing-week1
```

## Day 2 — bring up a loopback billing cell

```bash
make dev-cell.up CELL=billing-loopback-1 PROFILE=cloud-billing-dev
make dev-tenant.create T=oyatie.b2b.smb.acme-software TENANT_CLASS=paid
make dev-rate-card.attach T=oyatie.b2b.smb.acme-software CARD=rate-card-smb-paid-v1
```

Emit a usage event (synthetic — pretends `cloud-compute-k8s` is reporting):
```bash
./bin/oya billing meter emit \
  --tenant oyatie.b2b.smb.acme-software \
  --event-id "$(uuidgen)" \
  --resource-kind cloud_compute_k8s.pod_minute \
  --resource-id "ns:acme/pod:webapp-7d4f-abcd" \
  --quantity 60 \
  --unit minute \
  --timestamp "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --dimensions '{"region":"us-east-2","pack":"none","cost_center":"engineering"}'
```

Verify ingestion:
```bash
./bin/oya billing usage query \
  --tenant oyatie.b2b.smb.acme-software \
  --since "1h ago" \
  --resource-kind cloud_compute_k8s.pod_minute
```

You should see your event aggregated into the minute bucket.

## Day 3 — invoicing

Trigger a synthetic month-end close (dev profile fast-forwards time):
```bash
./bin/oya billing close --tenant oyatie.b2b.smb.acme-software --period 2026-05
```

Inspect the invoice:
```bash
./bin/oya billing invoice show --tenant oyatie.b2b.smb.acme-software --period 2026-05
```

Expected (truncated):
```
invoice_id          : inv-2026-05-acme-software
period              : 2026-05-01 → 2026-05-31
currency            : USD
subtotal            : $1,247.84
fx_rate_used        : (n/a, USD-native)
line_items          :
  cloud_compute_k8s.pod_minute  86,400 min × $0.0142 = $1,226.88
  cloud_kms.dek_issuance        14,800 ops × $0.0010 = $14.80
  cloud_storage.gb_month         3.5 GB-mo × $0.023 = $0.08
  (etc.)
chargeback_split    :
  engineering   $1,124.06
  product       $98.32
  ops           $25.46
issued_at           : 2026-06-01T08:00:00Z
audit_chain_event   : ce-2026-06-01T08:00:01Z-…
```

## Day 4 — chargeback configuration

The paid tenant_class allows ≤ 5 cost centers. Configure them:
```bash
./bin/oya billing cost-center create \
  --tenant oyatie.b2b.smb.acme-software \
  --code engineering \
  --display-name "Engineering" \
  --owner-email cto@acme-software.io
./bin/oya billing cost-center create \
  --tenant oyatie.b2b.smb.acme-software \
  --code product \
  --display-name "Product" \
  --owner-email vpp@acme-software.io
```

Attribution rules — every usage event must carry a `cost_center` dimension. Configure default:
```bash
./bin/oya billing attribution-rule create \
  --tenant oyatie.b2b.smb.acme-software \
  --rule-name "default-by-namespace" \
  --priority 100 \
  --match '{"dimensions.namespace": "ns:acme/product-*"}' \
  --target "cost_center=product"
```

Test the rule:
```bash
./bin/oya billing attribution-rule simulate \
  --tenant oyatie.b2b.smb.acme-software \
  --event-id … \
  --resource-id "ns:acme/product-discovery-svc-xyz"
```

## Day 5 — FOCUS 1.1 export + reservation purchase

Export FOCUS:
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

Expected: `focus_version: 1.1`, `rows: 12_847`, `schema_errors: 0`.

Purchase a 1-year reservation (Paid allows 30 % discount):
```bash
./bin/oya billing reservation purchase \
  --tenant oyatie.b2b.smb.acme-software \
  --resource-kind cloud_compute_k8s.pod_minute \
  --commitment 1y \
  --quantity-per-month 100000 \
  --start 2026-06-01
```

Expected output:
```
reservation_id    : res-2026-05-20-acme-001
commitment        : 1y
discount          : 30 %
upfront_charge    : $1,022.40 (= 100000 × $0.0142 × 12 × 0.7 / 12)
auto_renewal      : false
audit_chain_event : ce-2026-05-20T15:01:42Z-…
```

## What "done with week 1" means

- [ ] You can recite the two tenant classes and currency support per tenant_class.
- [ ] You emitted, queried, and saw a usage event hit a monthly invoice.
- [ ] You configured cost centers + an attribution rule + simulated it.
- [ ] You produced a FOCUS 1.1 export and validated the schema.
- [ ] You purchased a reservation and understand the upfront-charge math.
- [ ] You read ADR-0244 + ADR-0245 + the FOCUS 1.1 spec.

## Rookie traps

1. **Emitting events without `event_id`.** Idempotency requires UUID v7; missing IDs are rejected at the metering bus.
2. **Skipping `tenant_id` in dimensions.** The `lean-a3-tenant-trace` lane catches it; usage without tenant attribution is bug-class.
3. **Manually writing to the ledger.** Every credit/refund must flow through `cloud_billing::Action::IssueCreditMemo`; direct
   `UPDATE` on the ledger is a P1 incident.
4. **Cross-currency revaluation.** Never re-FX an invoice after issuance; the FX rate at issuance is locked. Use credit memos
   for FX corrections.
5. **Reservation overage.** Reservations are commitments — under-use is non-refundable. Configure thresholds before purchase.
6. **Skipping FOCUS validation.** A non-FOCUS-conformant export breaks downstream FinOps tooling; always validate before delivery.
