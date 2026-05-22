---
doc_class: MigrationPlaybook
microservice: payments
vendor: Stripe (direct integration)
date: 2026-05-20
doc_status: published
---

# Migration playbook — Stripe (direct) → oyatie payments

Audience: a merchant or oyatie tenant currently integrated directly to Stripe (using Stripe-hosted Checkout, Stripe Payment Element, or raw Stripe API + Connect) who wants to move to oyatie's `payments` µservice. The primary drivers: multi-PSP failover, per-tenant fee schedules (incl. FX margin recapture), ledger consistency with the rest of oyatie, audit-chain non-repudiation, and sovereignty-pack support.

## Why this migration matters

Direct Stripe integration is excellent for the 80 % case but has structural limits:

- Single-PSP exposure: Stripe outages = your outage (6× P1-class outages in 2024-2025 per Stripe's own status page).
- FX margin: Stripe captures ~ 1 % FX margin on cross-currency transactions; you can't recapture it.
- No KR / CN / sovereign-pack support: Stripe doesn't operate KCP / Inicis / Alipay / WeChat Pay; tenants in those markets need a parallel PSP integration.
- Ledger drift: Stripe's Balance Transaction API is the source of truth for Stripe; reconciling to your internal ledger is a daily ETL job.

oyatie payments addresses all four with multi-PSP routing + double-entry ledger + sovereign-pack PSP allowlists.

## Step 1 — Inventory Stripe surface in use (≤ 2-3 days)

```bash
# From a Stripe admin user:
stripe customers list --limit 100  # paginate; total customer count
stripe charges list --limit 100 --created.gte=1735689600  # 2025-01-01 onward
stripe subscriptions list --limit 100
stripe connect-accounts list  # if Connect is used
stripe products list  # subscription products
```

Document:

- Customer count, charge count (last 12 mo), subscription count.
- Stripe products in use: Payment Intents, Connect, Billing, Issuing, Treasury, Tax, Radar, etc.
- Webhooks subscribed (Stripe → your endpoint).
- Stripe Connect topology if applicable (standard, express, custom).
- FX exposure: count of cross-currency charges + currencies in use.

Typical mid-market merchant: 500k-2M customers, 4-12M charges/year, 50-500k active subscriptions, 1-2 Connect topologies, 8-14 webhook event types subscribed.

## Step 2 — Plan the cutover model (≤ 1 week)

Three patterns:

1. **Dual-write parallel run** (recommended for high-revenue tenants): every new charge is created against BOTH Stripe (existing direct) AND oyatie (new). Both succeed; the customer is charged once because oyatie routes to Stripe under the hood at first (effectively the same Stripe charge with extra oyatie metadata). Cut over after 4-8 weeks of clean parallel run.
2. **Forward-only cutover** (recommended for lower-revenue or test tenants): cutover-day, redirect new charges to oyatie. Stripe direct integration stays live for in-flight subscriptions + chargebacks for the trailing 12-18 months until they all close out.
3. **Big-bang migration** (only for greenfield or low-volume): rare; only do if Stripe direct integration is < 1 month old and no recurring subscriptions exist.

Pattern (1) gives the cleanest end-state but requires dual-emission discipline. Pattern (2) is most common.

## Step 3 — Migrate customer + payment-method records (≤ 1-2 weeks)

Stripe → oyatie customer record migration:

```sh
# Export Stripe customers
stripe customers list --limit 100 --auto-paginate > stripe-customers.jsonl

# Convert + import
oya payments migrate import-customers \
    --tenant acme-corp \
    --source stripe \
    --input stripe-customers.jsonl \
    --field-mapping mappings/stripe-customer-to-oyatie.yaml
```

Field mapping (`mappings/stripe-customer-to-oyatie.yaml`):

```yaml
customer:
  id:    "{{stripe.id}}"           # Keep Stripe's cus_* ID as the canonical ID
  email: "{{stripe.email}}"
  name:  "{{stripe.name}}"
  phone: "{{stripe.phone}}"
  metadata:
    stripe_customer_id: "{{stripe.id}}"
    migrated_at: "{{now}}"
    migration_batch_id: "{{batch_id}}"
```

Payment methods are PSP-specific tokens; you cannot transfer a Stripe token to Adyen. The migration creates a `oya_payments_customer` record that references the Stripe token; on first charge after migration, oyatie offers the customer to re-enter their card to network-tokenize across all active PSPs (one-click; takes 5-10 s; transparent to UX).

Until re-tokenization, charges against this customer route ONLY to Stripe (the only PSP that holds a valid token). This is the "migration grace period."

## Step 4 — Migrate Subscriptions (≤ 2-4 weeks)

```sh
stripe subscriptions list --limit 100 --auto-paginate > stripe-subscriptions.jsonl

oya payments migrate import-subscriptions \
    --tenant acme-corp \
    --source stripe \
    --input stripe-subscriptions.jsonl \
    --strategy continue-stripe-billing-until-renewal
```

Strategy options:

- `continue-stripe-billing-until-renewal`: keep Stripe as the billing engine for existing subs; new subs land in oyatie. Existing subs migrate at their next renewal (typically monthly or annual).
- `migrate-at-next-renewal`: same as above but make the migration explicit; the renewal will trigger an oyatie charge instead of a Stripe charge.
- `migrate-immediately-with-proration`: rare; only for tenants who want to break the existing Stripe billing cycle. Pro-rates the unused portion as a credit.

Most tenants use option 1. The grace period drags on for 12-18 months as annual subs renew.

## Step 5 — Migrate Connect topology (if applicable; ≤ 1 month)

Stripe Connect has three account types:

- **Standard**: connected accounts have their own Stripe dashboards. Hardest to migrate.
- **Express**: oyatie's `payments` µservice models this natively as `connected_merchant` records.
- **Custom**: oyatie models this natively as `custom_managed_merchant` records.

For Express + Custom: migrate via:

```sh
stripe connect-accounts list --limit 100 --auto-paginate > stripe-connected.jsonl

oya payments migrate import-connected-accounts \
    --tenant acme-corp \
    --source stripe \
    --input stripe-connected.jsonl \
    --keep-stripe-as-psp-for-existing-accounts
```

For Standard: each connected merchant needs their own migration; oyatie can't unilaterally migrate Stripe Standard accounts because they're independent Stripe accounts. We hand the merchant a self-serve migration tool that uses Stripe's OAuth disconnect flow + oyatie's re-connect flow.

## Step 6 — Webhook receivers (≤ 1 week)

Stripe webhooks → oyatie events:

| Stripe webhook | oyatie audit-chain event class |
|---|---|
| `charge.succeeded` | `payments.charge.succeeded` |
| `charge.refunded` | `payments.refund.executed` |
| `charge.dispute.created` | `payments.chargeback.opened` |
| `customer.subscription.created` | `payments.subscription.created` |
| `customer.subscription.deleted` | `payments.subscription.canceled` |
| `invoice.payment_succeeded` | `payments.invoice.settled` |
| `payout.created` | `payments.payout.initiated` |
| `payout.paid` | `payments.payout.settled` |

The oyatie webhook adapter accepts Stripe webhook events 1:1 and emits oyatie events. The merchant's existing webhook handlers can continue to receive Stripe-shaped webhooks during the migration grace period; we double-emit (Stripe → merchant + oyatie → audit-chain).

## Step 7 — Reconcile + cut over (≤ 4-12 weeks for dual-write)

```sh
oya payments reconcile \
    --tenant acme-corp \
    --source-a stripe-direct \
    --source-b oyatie \
    --window-day 2026-05-20 \
    --report ./reconciliation.json
```

Acceptance criteria for cutover:

- Drift < 0.01 % per day on charge count + amount + fee.
- Refund + chargeback flows verified end-to-end on oyatie.
- All subscriptions either migrated or in `continue-stripe-billing-until-renewal` mode with a documented end-date.
- Connect topology migrated (Express + Custom) or self-serve handed off (Standard).

After ≥ 4 consecutive weeks of clean reconciliation, flip the default-PSP from `stripe-direct` to `oyatie`:

```sh
oya governance set-config \
    --tenant acme-corp \
    --key default_payment_provider \
    --value oyatie

oya audit emit \
    --tenant acme-corp \
    --event-class governance.payment_substrate.cut_over \
    --payload '{"from":"stripe-direct","to":"oyatie","cutover_at":"2026-05-20T14:00:00Z"}'
```

## Step 8 — Sunset Stripe direct integration (≤ 12-18 months post-cutover)

Stripe direct integration stays live until:

- All recurring subscriptions in `continue-stripe-billing-until-renewal` have renewed at least once into oyatie.
- All chargeback windows have closed (typically 90-180 d post-last-charge).
- All Connect Standard merchants have completed self-serve migration.

At that point:

```sh
oya audit emit \
    --tenant acme-corp \
    --event-class governance.payment_substrate.decommissioned \
    --payload '{"vendor":"stripe-direct","decommission_at":"2027-08-20T14:00:00Z"}'
```

The Stripe API keys remain rotatable in `oya secrets rotate` for ≥ 90 d post-decommission in case of post-hoc reconciliation needs.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Customer balks at re-entering card | High | Network Tokens (Visa Token Service, Mastercard MDES) preserve the token across PSPs without customer re-entry; opt-in at tokenize time |
| Stripe webhook handlers diverge from oyatie events | Medium | Run the webhook adapter in shadow first; reconcile event-by-event for ≥ 14 d |
| FX margin drops during migration (Stripe captures vs oyatie captures) | Low (revenue increases) | Configure the oyatie fee schedule to recapture the margin; document for the tenant |
| Subscriptions miss a renewal due to migration timing | High | The `continue-stripe-billing-until-renewal` strategy explicitly avoids this |
| Stripe Standard connected accounts fail self-serve migration | Medium | Provide white-glove migration support for top 10 % accounts |
| Reconciliation drift > 0.01 % blocks cutover | High | Block cutover; investigate; common cause: webhook delivery delay |
| PCI-DSS scope expands during migration | High | Pre-validate the Cedar + Cilium scope before going live; QSA can scope-amend mid-year |
| Customer service receives questions during dual-write | Low | Train CS on the dual-write model + provide canned responses |
| Cross-border tenants need a non-Stripe PSP for KR / CN markets | Medium | Add the relevant pack PSPs (KCP, Toss, Alipay) BEFORE cutover |
