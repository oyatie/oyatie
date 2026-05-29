# Tutorial — Publish a listing, take a purchase, walk escrow, and trigger payout

Goal: end-to-end commercial cycle: Alice publishes a workflow listing, Bob purchases, escrow releases, Alice gets paid. All on a
loopback cell with mock payment rails.

Pre-reqs:
- Loopback cell: `make dev-cell.up CELL=marketplace-loopback-1 PROFILE=marketplace-dev`
- Two dev tenants:
  ```bash
  make dev-tenant.create T=oyatie.b2c.indie.alice TENANT_CLASS=paid BILLING_COMPONENTS=revenue_share,per_usage
  make dev-tenant.create T=oyatie.b2c.indie.bob TENANT_CLASS=paid BILLING_COMPONENTS=per_seat
  ```
- KYC pre-populated by dev-cell:
  ```bash
  ./bin/oya identity kyc-attach \
    --tenant oyatie.b2c.indie.alice \
    --kyc-artifact-id kya-dev-alice-paid \
    --bank-account-mock test-checking
  ./bin/oya identity kyc-attach \
    --tenant oyatie.b2c.indie.bob \
    --payment-method-mock test-card-visa
  ```

## Step 1 — Alice publishes a listing

```bash
./bin/oya marketplace listing create \
  --tenant oyatie.b2c.indie.alice \
  --category workflow \
  --title "Daily Standup Summarizer" \
  --description "Aggregates yesterday's git activity, calendar events, and outstanding PRs into a 5-bullet summary." \
  --pricing one-time-usd-1900 \
  --region-availability "US,CA,GB,EU,KR,JP,AU" \
  --payload-file ./samples/listings/daily-standup-summarizer.workflow.json \
  --screenshots ./samples/listings/screenshots/*.png \
  --license MIT
```

Expected output:
```
listing_id    : lst-2026-05-20-daily-standup-summarizer
status        : Published
slug          : daily-standup-summarizer
seller_payout : $16.10  (gross $19.00 - platform 10 % - $0.30 = $16.10)
audit_event   : ce-…
```

The listing is now discoverable:
```bash
./bin/oya marketplace search --query "standup" --category workflow
```

## Step 2 — Bob purchases

```bash
./bin/oya marketplace purchase \
  --tenant oyatie.b2c.indie.bob \
  --listing lst-2026-05-20-daily-standup-summarizer \
  --payment-method test-card-visa
```

Expected:
```
purchase_id : pur-2026-05-20-…
amount_usd  : 19.00
escrow_state: Held
escrow_until: 2026-05-27T08:30:00Z   (7 d paid escrow window)
license_id  : lic-2026-05-20-bob-daily-standup-…
audit_event : ce-…
```

Bob now has a license; the workflow can be installed.

## Step 3 — Inspect the ledger

```bash
./bin/oya marketplace ledger show \
  --tenant oyatie.b2c.indie.alice \
  --since 1h
```

Expected (trimmed):
```
ts                          type            amount     direction   to/from
2026-05-20T08:31:14Z         gross-charge    $19.00     credit      bob (held in escrow)
2026-05-20T08:31:14Z         platform-fee    -$2.20     debit       platform
2026-05-20T08:31:14Z         payment-fee     -$0.30     debit       payments-µservice
2026-05-20T08:31:14Z         net-payable     $16.50     pending     alice (escrow released 2026-05-27)
```

## Step 4 — Buyer requests refund within escrow

(Optional happy-path skip: jump to Step 5 if Bob is satisfied.)

```bash
./bin/oya marketplace refund-request \
  --tenant oyatie.b2c.indie.bob \
  --purchase pur-2026-05-20-… \
  --reason "Doesn't match my workflow"
```

Within 48 h, Alice can object. With no objection:
```
escrow_state: Refunded
refund_amount: $19.00 to bob
ledger:
  refund-issued     -$19.00 from alice's net-payable
  refund-platform-fee +$2.20 returned to alice (platform absorbs for paid tenants)
  refund-payment-fee +$0.30 returned to alice
audit_event: ce-…
```

## Step 5 — Escrow window expires; payout queues

Either fast-forward time on the dev cell (`./bin/oya time advance --days 7`) or wait. At escrow expiry:
```
escrow_state: Released
net_payable_to_alice: $16.50 → payout queue
payout_eta: 2026-05-21T08:30:00Z (paid: weekly payouts on Sunday)
```

## Step 6 — Payout runs

```bash
./bin/oya marketplace payout run \
  --tenant oyatie.b2c.indie.alice \
  --cycle weekly-2026-W21
```

Expected:
```
payout_id  : po-2026-W21-alice-…
amount_usd : 16.50
rail       : ACH (test-checking mock)
status     : Initiated
expected_settle: 2026-05-23T17:00:00Z
audit_event: ce-…
```

## Step 7 — Verify audit chain integrity

```bash
./bin/oya audit query --tenant oyatie.b2c.indie.alice --window 8d
```

You'll see the full event sequence: listing-created → purchase → ledger entries → escrow-released → payout-initiated →
payout-settled. Each event's `prev_hash` matches the previous `curr_hash`.

## Step 8 — Cleanup

```bash
./bin/oya marketplace listing delete --tenant oyatie.b2c.indie.alice --listing lst-2026-05-20-…
make dev-tenant.delete T=oyatie.b2c.indie.alice
make dev-tenant.delete T=oyatie.b2c.indie.bob
```

Listing delete triggers the 90-day retention countdown for paid tenants; listing remains queryable for existing licensees during retention.

## What you proved

- Universal settlement across categories — same flow works for any of the 6 listing categories.
- Escrow + auto-release works on a wall-clock timer (or fast-forwarded).
- Refund-within-escrow is non-disruptive and auto-credits platform fees back.
- Payout schedules respect tenant_class and billing_components (paid weekly in this fixture).
- Every commercial event chains in the audit ledger.
