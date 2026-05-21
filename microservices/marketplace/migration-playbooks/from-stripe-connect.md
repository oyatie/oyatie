# Migration playbook — Stripe Connect → Oyatie `marketplace`

Audience: a platform running a marketplace on Stripe Connect (Express, Custom, or Standard) — typically a SaaS that takes a cut of
transactions between two parties (e.g. Uber-style, Substack-style, Etsy-style, B2B SaaS marketplace).

> Phase budget: 60 days for a small marketplace (≤ 1k sellers); 120 days for mid-market (≤ 100k sellers); 240 days for large
> (≥ 1M sellers) with regulatory complexity.

## Phase 0 — Inventory (Day 0…14)

1. **Export Stripe accounts:**
   ```bash
   stripe accounts list --limit 100 -o accounts.json
   # paginate via starting_after if you have >100
   ```
2. **Categorize each account:**
   - Account type (Express / Custom / Standard)
   - Country
   - Onboarding state (`details_submitted`, `payouts_enabled`)
   - Volume (last-12-month gross processed)
3. **Export connected charges:**
   ```bash
   stripe charges list --transfer_data[destination] all -o charges.json
   ```
4. **Map your listings:**
   - What kind of listings does the marketplace sell (apps, plugins, workflows, services)?
   - Which Oyatie listing category fits (or do you need a new payload schema)?
5. **Tax engine inventory:**
   - Are you using Stripe Tax? In what jurisdictions?
   - Do you collect 1099-K data?
   - Do you handle DAC7 / MTR reporting?

## Phase 1 — Tenant provisioning + KYC handoff (Day 14…30)

```bash
./bin/oya tenant create \
  --id oyatie.b2b.marketplace.<your-platform> \
  --tenant-class paid \
  --billing-components revenue_share,per_seat,per_usage \
  --region us-east-2 \
  --pack-set "soc2-type-ii-v2017,gdpr-eu-v2018"
```

For each Stripe-connected seller, create an Oyatie tenant and link their KYC:
```bash
./bin/oya tenant create \
  --id oyatie.b2c.marketplace.<your-platform>.seller.alice \
  --parent oyatie.b2b.marketplace.<your-platform> \
  --tenant-class paid \
  --billing-components revenue_share,per_usage
./bin/oya identity kyc-import \
  --tenant oyatie.b2c.marketplace.<your-platform>.seller.alice \
  --source-format stripe-connect \
  --source-account-id acct_…
```

The KYC importer pulls Stripe's `requirements` + `verification` blobs and produces a KYC artifact in `identity` µservice.

## Phase 2 — Listing migration (Day 30…60)

For each existing listing or product in your Stripe marketplace:
```bash
./bin/oya marketplace listing import \
  --tenant oyatie.b2c.marketplace.<your-platform>.seller.alice \
  --source-format stripe-product \
  --source-id prod_… \
  --category workflow   # or whichever fits
```

Listings imported but unpublished by default; publish per-seller after they've signed Oyatie ToS.

## Phase 3 — Dual-run (Day 60…90)

For each purchase your platform handles, fire both Stripe Connect and Oyatie marketplace in parallel:
```python
async def purchase(buyer, listing):
    # Source of truth: Stripe
    stripe_charge = stripe.Charge.create(...)
    # Shadow: Oyatie marketplace
    try:
        oya_purchase = await oya_client.marketplace.purchase({
            "tenant": buyer.tenant_id,
            "listing_id": listing.oya_listing_id,
            "payment_method": shadow_payment_method,
        })
        await migration_diff.record(stripe_charge, oya_purchase)
    except Exception as e:
        await migration_diff.record_error(stripe_charge, e)
    return stripe_charge
```

Compare daily for 30 d; target 99.9 % outcome-parity.

## Phase 4 — Tax + reporting migration (Day 90…120)

Move tax responsibility to Oyatie marketplace:
- If using Stripe Tax: leave it on; configure `marketplace` to delegate tax calculation to `payments`+Stripe Tax adapter.
- If using Avalara/TaxJar: configure the `marketplace` tax port to point at your existing engine.
- DAC7 / MTR reporting: enable `marketplace.reporting.dac7 = true`; Oyatie will collect required seller data + file annual reports.

## Phase 5 — Cutover (Day 120…135)

1. New purchases route to Oyatie marketplace only.
2. Existing escrows + recurring subscriptions continue to flow through Stripe Connect for their natural lifetime.
3. After all Stripe-side escrows release, disable new Stripe Connect charges.

## Phase 6 — Stripe Connect decommission (Day 135+)

- For accounts with no pending balance: deactivate.
- For accounts with pending balance: wait for final payout, then deactivate.
- After 30 d of no charges: archive the Stripe Connect platform account (Stripe retains records but stops new charges).

## Rollback

Within the dual-run + new-purchase cutover (60-day window):
- Disable the marketplace-first routing.
- Re-enable Stripe Connect first.
- Migrate any Oyatie-only escrows back to Stripe via a one-time refund + re-charge.

After full decommission: rollback requires re-onboarding sellers on Stripe Connect, which is a 7-30 day per-seller process.

## What you gain

- One ledger across listing categories (vs Stripe's single-category orientation).
- Lower platform fee on the paid revenue_share path (5.6 % vs effective 3.5 % Stripe Connect but excluding tax + KYC + dispute external costs; total
  ~5.6 % vs ~5 % all-in, so close).
- Cedar-gated dispute stages.
- BLAKE3 audit chain.
- provider-credential BYOK at the listing (ADR-0255 §D-4).
- EU AI Act pack overlay.

## What you give up

- Stripe's mature payment-rail coverage in some niche regions (we ride Stripe under the hood for those, so coverage is the same).
- Instant Payouts (Stripe has them at $0.50/txn; marketplace exposes them through paid tenant policy plus the relevant billing_components).
- Stripe Dashboard UX (marketplace has its own UX via `workflow-studio` + `finops-portal`).
- The "Stripe is everywhere" mental model.
