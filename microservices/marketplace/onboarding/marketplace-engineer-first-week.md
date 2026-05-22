# Marketplace Engineer — First Week on `marketplace`

Audience: an engineer with payments / marketplace / Stripe Connect / Shopify experience joining the `oya-marketplace-*` lane.

## Day 1 — required reading

- `docs/decisions/ADR-0249-multi-category-marketplace-doctrine.md` — what the marketplace is.
- `docs/decisions/ADR-0314-universal-deal-settlement.md` — one settlement ledger across categories.
- `microservices/marketplace/REMEDIATION-NOTES-2026-05-21.md#tenant-class-behavior` — tenant_class and billing_components surface.
- `microservices/payments/` — sibling µservice that handles raw card processing (marketplace builds on payments).
- IRS DAC7 + MTR documentation (US sales tax for marketplace facilitators) — you'll be wiring tax engines.
- EU DSA (Digital Services Act) Article 30 — marketplace traceability obligations.

Clone:
```bash
./bin/oya git worktree-add --base dev --branch onboarding/$USER-marketplace-week1 .worktrees/$USER-marketplace-week1
```

## Day 2 — walk the listing lifecycle

```bash
make dev-cell.up CELL=marketplace-loopback-1 PROFILE=marketplace-dev
make dev-tenant.create T=oyatie.b2c.indie.alice TENANT_CLASS=demo_trial
make dev-tenant.create T=oyatie.b2c.indie.bob TENANT_CLASS=demo_trial
```

Alice publishes; Bob buys:
```bash
./bin/oya marketplace listing create \
  --tenant oyatie.b2c.indie.alice \
  --category workflow \
  --title "Daily Standup Summarizer" \
  --pricing one-time-usd-1900 \
  --region-availability "US,CA,GB,EU,KR,JP,AU"

LISTING_ID=$(jq -r .id last-listing.json)

./bin/oya marketplace purchase \
  --tenant oyatie.b2c.indie.bob \
  --listing $LISTING_ID \
  --payment-method test-card-visa
```

Inspect the settlement ledger:
```bash
./bin/oya marketplace ledger show \
  --tenant oyatie.b2c.indie.alice \
  --since 1h
```

You'll see escrow, platform-fee, and net-to-seller rows.

## Day 3 — read the code

Walk:
1. `crates/oya-marketplace-domain/src/listing.rs` — closed `ListingCategory` enum.
2. `crates/oya-marketplace-domain/src/settlement.rs` — the universal ledger model.
3. `crates/oya-marketplace-kernel/src/escrow.rs` — escrow state machine (Pending → Held → Released | Refunded | Disputed).
4. `crates/oya-marketplace-port-payments/src/lib.rs` — outbound to `payments`.
5. `crates/oya-marketplace-port-tax/src/lib.rs` — outbound to tax engine (Avalara / TaxJar / Stripe Tax / direct).
6. `crates/oya-marketplace-app/src/listing_api.rs` — REST + gRPC surface.

## Day 4 — author a dispute resolution policy

Pick a starter task from `microservices/marketplace/backlog/starter-disputes.md`. Implement the policy under
`crates/oya-marketplace-rules-disputes/`:

```rust
use oya_marketplace_rules::prelude::*;

#[derive(DisputeRule)]
#[rule(
    rule_id = "auto-refund-low-value-undelivered",
    applies_to = "category=workflow,plugin",
    max_value_usd = 100.0
)]
pub struct AutoRefundLowValueUndelivered;

impl DisputeRule for AutoRefundLowValueUndelivered {
    fn evaluate(&self, ctx: &DisputeCtx) -> DisputeOutcome {
        if ctx.evidence.delivery_status == DeliveryStatus::Undelivered
            && ctx.transaction.amount_usd <= 100.0
        {
            DisputeOutcome::AutoRefundBuyer
        } else {
            DisputeOutcome::EscalateToHuman
        }
    }
}
```

Hermetic tests against the dispute simulator:
```bash
cargo test -p oya-marketplace-rules-disputes
```

## Day 5 — ship a real listing-category change

```bash
./bin/oya vcs claim \
  --agent marketplace-eng-$USER \
  --intent marketplace-add-dataset-licence-clauses \
  crates/oya-marketplace-domain microservices/marketplace
```

Implement + verify + done + PR. Foundry handles admission.

## Done with week 1

- [ ] You completed a publish + purchase + escrow + release end-to-end.
- [ ] You can recite the 6 listing categories and the universal-settlement principle.
- [ ] You authored, signed, and merged a dispute rule.
- [ ] You read ADR-0249 + ADR-0314.
- [ ] You traced a settlement event through the audit chain.

## Rookie traps

1. **Per-category shadow ledgers.** Forbidden. One ledger across all 6 categories.
2. **Trusting buyer evidence.** Disputes must consider seller evidence too; one-sided evidence is a smell.
3. **Cleartext PII in audit events.** Names + emails are PII; audit events store hashed references.
4. **Forgetting region availability.** Selling into a region without availability declared triggers `lean-a2-region-availability` lane.
5. **Skipping KYC checks.** A tenant without KYC at its tier cannot list; cedar refuses.
