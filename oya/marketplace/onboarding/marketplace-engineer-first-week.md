# Marketplace Engineer — First Week on `marketplace`

Audience: an engineer with payments / marketplace / Stripe / Shopify experience joining the `oya-marketplace-*` lane.

## Day 1 — required reading

- `docs/decisions/ADR-0249-multi-category-marketplace-doctrine.md` — what the marketplace is.
- `docs/decisions/ADR-0314-universal-deal-settlement.md` — one settlement ledger across categories.
- `oya/marketplace/REMEDIATION-NOTES-2026-05-21.md#tenant-class-behavior` — tenant_class and billing_components surface.
- `oya/payments/` — sibling service that handles raw card processing (marketplace builds on payments) when present in the lane manifest.
- IRS DAC7 + MTR documentation (US sales tax for marketplace facilitators) — you'll be wiring tax engines.
- EU DSA (Digital Services Act) Article 30 — marketplace traceability obligations.

Clone:
```bash
git fetch origin dev
git worktree add -b onboarding/$USER-marketplace-week1 .worktrees/$USER-marketplace-week1 origin/dev
```

## Day 2 — walk the listing lifecycle

Bring up the `marketplace-loopback-1` dev cell through the registered Buck2/Prow dev-cell harness with `PROFILE=marketplace-dev`. Seed demo
tenants `oyatie.b2c.indie.alice` and `oyatie.b2c.indie.bob` through the lane-owned Rust fixture, not ad hoc shell targets.

Alice publishes; Bob buys:
Use the marketplace control-plane API or its registered Rust harness to create a workflow listing for tenant
`oyatie.b2c.indie.alice`, buy it as `oyatie.b2c.indie.bob`, and capture the returned listing id in PR evidence. Do not add local CLI wrapper
commands to the repo.

Inspect the settlement ledger:
Use the registered ledger read harness for tenant `oyatie.b2c.indie.alice` with a one-hour window.

You'll see escrow, platform-fee, and net-to-seller rows.

## Day 3 — read the code

Walk:
1. `oya/marketplace/contracts/` — REST/gRPC/listing contracts.
2. `oya/marketplace/capabilities/` — category and settlement capability declarations.
3. `oya/marketplace/policies/` — Cedar policies for category, escrow, mediation, and revenue-share decisions.
4. `oya/marketplace/catalog/` — service component catalog rows.
5. `oya/marketplace/reference-implementations/` — executable pattern references for publish/purchase flows.
6. `oya/marketplace/crates/oya-marketplace-doc-set-scaffold/` — current checked-in Rust shard for doc-set scaffolding.

## Day 4 — author a dispute resolution policy

Pick a starter task from `oya/marketplace/IPs/` or `oya/marketplace/migration-playbooks/`. Implement the policy in the lane-owned Rust shard or
`oya/marketplace/policies/` path named by that task:

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
buck2 test <registered-marketplace-dispute-rules-target>
```

## Day 5 — ship a real listing-category change

Implement the listing-category change in your isolated worktree, run the registered Buck2 targets, and open a PR against `dev`. Merge readiness
comes from reviewer approval plus trusted Prow/Kubernetes-native `oya-ci-required`; GitHub Actions is shadow compatibility only.

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
