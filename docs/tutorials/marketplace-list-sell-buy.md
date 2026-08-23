---
doc_class: Tutorial
tutorial_id: TUT-OYATIE-MKT-DEAL-004
persona: "Yejin Choi, independent seller using Oyatie Marketplace"
prerequisite_packs:
  - canonical-base
  - marketplace-seller
  - marketplace-buyer
  - payments-basic
related_oyatie_adrs:
  - ADR-0244
  - ADR-0249
  - ADR-0263
  - ADR-0314
  - ADR-0316
status: Draft
date: 2026-05-20
owner: docs-experience
estimated_completion_time: "85 minutes"
---

Tenant class model: `tenant_class` is `controlled_evaluation` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels. Tutorial execution remains a guided operator walkthrough until the relevant M03-P04/M03-P08 changeset evidence is green.

# List, Sell, Buy, and Settle a Marketplace Deal

## Goal

You will list a physical product, publish it to Marketplace, purchase it as a buyer, accept the order as the seller, record fulfillment, settle the deal through the DealSet lifecycle, and verify the settlement evidence required by ADR-0314.

## Prerequisites

- Seller account: `yejin.choi@example.kr`.
- Buyer account: `aiyana.ross@example.com`.
- Seller tenant: `b2c-yejin-choi`.
- Buyer tenant: `b2c-aiyana-ross`.
- Marketplace listing id: `listing-vintage-jacket-2026-001`.
- DealSet id to create: `dealset-vintage-jacket-2026-001`.
- Product title: `Vintage indigo work jacket`.
- Product SKU: `VINTAGE-JACKET-INDIGO-M`.
- Price: `USD 86.00`.
- Shipping price: `USD 12.00`.
- Tax estimate: `USD 7.84`.
- Settlement rail: `stripe-connect-standard`.
- Seller payout account: `acct_yejin_marketplace_001`.
- Subscribed microservices: `marketplace`, `payments`, `treasury`, `finops-portal`, `workflow-engine`, `ontology`, `policy-engine`, `audit-chain`, `mail`, `drive`.
- Required Cedar permit: `marketplace.listing.create`.
- Required Cedar permit: `marketplace.listing.publish`.
- Required Cedar permit: `marketplace.deal.offer.create`.
- Required Cedar permit: `marketplace.deal.accept`.
- Required Cedar permit: `marketplace.deal.settle`.
- Required Cedar permit: `payments.intent.authorize`.
- Required Cedar permit: `payments.payout.release`.
- Required Cedar permit: `audit.marketplace.read`.
- Active capability tier: `marketplace-seller-core`.
- Active capability tier: `marketplace-buyer-core`.
- Test photo file: `jacket-front-redacted.jpg`.
- Test shipping carrier: `OyaShip Ground`.
- Test tracking number: `OYATIE-GROUND-20260520-001`.

## Step-by-Step

1. Open Marketplace as the seller.
   - Sign in as `yejin.choi@example.kr`.
   - Select tenant `Personal - Yejin Choi`.
   - Open `Marketplace -> Seller Center`.
   - Confirm seller status: `Verified`.
   - Confirm payout account: `acct_yejin_marketplace_001`.
   - Confirm capability tier: `marketplace-seller-core`.
   - Screenshot checkpoint: capture the Seller Center header.
   - If the seller status is not verified, stop and complete identity review first.
   - The seller tenant remains the owner of the listing and seller-side DealSet role.
   - Do not list under a work tenant for this tutorial.

2. Create the listing draft.
   - Click `New listing`.
   - Listing id: `listing-vintage-jacket-2026-001`.
   - Category: `Goods -> Clothing -> Jackets`.
   - Title: `Vintage indigo work jacket`.
   - SKU: `VINTAGE-JACKET-INDIGO-M`.
   - Condition: `Used - excellent`.
   - Quantity: `1`.
   - Price: `USD 86.00`.
   - Shipping: `OyaShip Ground - USD 12.00`.
   - Click `Save draft`.
   - Expected toast: `Listing draft saved`.

3. Add listing media and provenance.
   - Open the `Media` tab.
   - Upload `jacket-front-redacted.jpg`.
   - Alt text: `Front view of vintage indigo work jacket`.
   - Provenance note: `Purchased at estate sale in Seoul, 2024`.
   - Data class: `Public listing media`.
   - Click `Run media checks`.
   - Expected result: `No restricted content detected`.
   - Screenshot checkpoint: capture the media check result.
   - Keep all personal addresses out of images.
   - Return to the listing draft.

4. Configure DealSet terms.
   - Open the `Deal terms` tab.
   - Deal category: `goods`.
   - DealSet id: `dealset-vintage-jacket-2026-001`.
   - Seller role: `seller_tenant`.
   - Buyer role: `consumer_principal`.
   - Commercial terms: `Price USD 86.00, shipping USD 12.00`.
   - Settlement terms: `Authorize at purchase, capture on seller acceptance`.
   - Tax terms: `Marketplace estimated tax USD 7.84`.
   - Dispute window: `P14D after delivery`.
   - Click `Validate DealSet`.
   - Expected result: `DealSet terms valid`.

5. Publish the listing.
   - Open `Review`.
   - Confirm title and SKU.
   - Confirm quantity `1`.
   - Confirm DealSet id.
   - Confirm payout account.
   - Confirm `Cedar policy: marketplace.goods.standard-v1`.
   - Click `Publish listing`.
   - Expected toast: `Vintage indigo work jacket is live`.
   - Expected audit event: `DealSetOffered`.
   - Screenshot checkpoint: capture the live listing page.
   - Copy the listing URL.

6. Open the listing as the buyer.
   - Sign out or open a separate browser profile.
   - Sign in as `aiyana.ross@example.com`.
   - Select tenant `Personal - Aiyana Ross`.
   - Open the copied listing URL.
   - Confirm title: `Vintage indigo work jacket`.
   - Confirm seller: `Yejin Choi`.
   - Confirm total: `USD 105.84`.
   - Confirm buyer protection: `DealSet settlement`.
   - Screenshot checkpoint: capture the product detail panel.
   - Do not use a work tenant payment method.

7. Add the product to cart.
   - Click `Add to cart`.
   - Cart line item should show SKU `VINTAGE-JACKET-INDIGO-M`.
   - Quantity should be locked to `1`.
   - Click `Checkout`.
   - Shipping address label: `Aiyana home test address`.
   - Payment method: `Visa test card ending 4242`.
   - Tax line: `USD 7.84`.
   - Order total: `USD 105.84`.
   - Screenshot checkpoint: capture the checkout summary.
   - Continue to review.

8. Accept the offer as buyer.
   - On `Review order`, expand `Settlement details`.
   - Confirm DealSet id: `dealset-vintage-jacket-2026-001`.
   - Confirm state: `Published`.
   - Confirm buyer role: `consumer_principal`.
   - Confirm settlement rail: `stripe-connect-standard`.
   - Click `Place order`.
   - Expected toast: `Order placed. Waiting for seller acceptance.`
   - Expected DealSet state: `AcceptancePending`.
   - Screenshot checkpoint: capture the order confirmation.
   - Copy the order id shown on the page.

9. Inspect buyer-side audit summary.
   - Open `Marketplace -> Purchases`.
   - Select the new order.
   - Open `Audit summary`.
   - Confirm event: `DealSetAccepted`.
   - Confirm payment state: `authorized`.
   - Confirm capture state: `pending seller acceptance`.
   - Confirm tenant scope: `b2c-aiyana-ross`.
   - Screenshot checkpoint: capture the audit summary.
   - Close the drawer.
   - The buyer does not see seller payout account details.
   - The seller does not see full buyer card details.

10. Accept the order as seller.
    - Return to Yejin's seller browser session.
    - Open `Seller Center -> Orders`.
    - Select order for `VINTAGE-JACKET-INDIGO-M`.
    - Confirm DealSet id: `dealset-vintage-jacket-2026-001`.
    - Confirm payment authorization: `authorized`.
    - Click `Accept order`.
    - Expected toast: `Order accepted and payment capture requested`.
    - Expected DealSet state: `SettlementPending`.
    - Screenshot checkpoint: capture seller order detail.
    - This transition emits the seller-side settlement event.

11. Record fulfillment.
    - In the order detail, click `Fulfill`.
    - Carrier: `OyaShip Ground`.
    - Tracking number: `OYATIE-GROUND-20260520-001`.
    - Ship date: `2026-05-20`.
    - Package weight: `1.1 kg`.
    - Click `Save fulfillment`.
    - Expected toast: `Fulfillment recorded`.
    - Expected event: `DealSetFulfillmentRecorded`.
    - Screenshot checkpoint: capture the fulfillment row.
    - Aiyana should receive a Mail notification.

12. Confirm buyer delivery notification.
    - Aiyana opens `Mail`.
    - Open message subject `Your order has shipped`.
    - Confirm tracking number `OYATIE-GROUND-20260520-001`.
    - Click `View order`.
    - Confirm DealSet state: `Fulfilled`.
    - Confirm dispute window text: `14 days after delivery`.
    - Screenshot checkpoint: capture the buyer order timeline.
    - Do not open a dispute in the happy-path tutorial.
    - Return to the order timeline.
    - Wait for simulated delivery.

13. Confirm delivery as buyer.
    - In Purchases, click `Confirm delivery`.
    - Confirmation checkbox: `I received the jacket`.
    - Feedback: `Item received in described condition.`
    - Click `Confirm`.
    - Expected toast: `Delivery confirmed`.
    - Expected state: `DeliveryConfirmed`.
    - Screenshot checkpoint: capture the delivery confirmation.
    - This starts payout release based on the configured risk policy.
    - The dispute window remains visible.
    - No review is required for settlement.

14. Release seller payout.
    - Yejin opens `Seller Center -> Payouts`.
    - Select payout for DealSet `dealset-vintage-jacket-2026-001`.
    - Confirm gross amount: `USD 105.84`.
    - Confirm marketplace fee: `USD 8.60`.
    - Confirm shipping pass-through: `USD 12.00`.
    - Confirm tax remittance: `USD 7.84`.
    - Confirm seller net: `USD 77.40`.
    - Click `Release payout`.
    - Expected state: `PayoutReleased`.
    - Screenshot checkpoint: capture payout ledger row.

15. Review settlement evidence.
    - Open `Seller Center -> Orders -> Settlement evidence`.
    - Confirm event sequence includes `DealSetOffered`.
    - Confirm event sequence includes `DealSetAccepted`.
    - Confirm event sequence includes `DealSetSettled`.
    - Confirm ontology object ref begins with `ontology://marketplace/goods#DealSet`.
    - Confirm workflow run ref begins with `wf-marketplace-goods-settlement`.
    - Confirm audit chain seal is present.
    - Click `Export evidence`.
    - Save file as `dealset-vintage-jacket-2026-001-evidence.pdf`.
    - Screenshot checkpoint: capture the exported evidence row.

16. Run the settlement verification query.
    - Open `Marketplace -> Seller Center -> Saved checks`.
    - Choose `tutorial.marketplace_deal_settlement_status`.
    - Input `deal_set_id=dealset-vintage-jacket-2026-001`.
    - Input `seller_tenant=b2c-yejin-choi`.
    - Input `buyer_tenant=b2c-aiyana-ross`.
    - Click `Run`.
    - Expected title: `Marketplace DealSet settled`.
    - Expected state: `PASS`.
    - Screenshot checkpoint: capture the query output.
    - Store the evidence PDF in `Drive -> Marketplace Evidence`.
    - The tutorial is complete when the query and evidence export agree.

## Verification

- Named query: `tutorial.marketplace_deal_settlement_status`.
- Query location: `Marketplace -> Seller Center -> Saved checks`.
- Query input `deal_set_id`: `dealset-vintage-jacket-2026-001`.
- Query input `listing_id`: `listing-vintage-jacket-2026-001`.
- Query input `seller_tenant`: `b2c-yejin-choi`.
- Query input `buyer_tenant`: `b2c-aiyana-ross`.
- Expected output field: `listing_state`.
- Expected output value: `sold`.
- Expected output field: `deal_set_state`.
- Expected output value: `settled`.
- Expected output field: `payment_authorized`.
- Expected output value: `true`.
- Expected output field: `payment_captured`.
- Expected output value: `true`.
- Expected output field: `payout_released`.
- Expected output value: `true`.
- Expected output field: `ontology_projection_ref`.
- Expected output value prefix: `ontology://marketplace/goods#DealSet`.
- Expected output field: `workflow_run_ref`.
- Expected output value prefix: `wf-marketplace-goods-settlement`.
- Expected output field: `audit_chain_seal_present`.
- Expected output value: `true`.
- Expected output field: `counterparty_roles_count`.
- Expected output value: `2`.
- Expected output field: `result_label`.
- Expected output value: `Marketplace DealSet settled`.
- Governed verifier record expected line: `PASS tutorial.marketplace_deal_settlement_status`.
- Governed verifier record expected line: `deal_set_state=settled`.
- Governed verifier record expected line: `payout_released=true`.
- Audit event to inspect: `DealSetOffered`.
- Audit event to inspect: `DealSetAccepted`.
- Audit event to inspect: `DealSetFulfillmentRecorded`.
- Audit event to inspect: `DealSetSettled`.
- Audit event to inspect: `PaymentPayoutReleased`.
- Dashboard: `Marketplace -> Settlement Health`.
- Expected tile: `DealSet settlement healthy`.
- Evidence artifact: `dealset-vintage-jacket-2026-001-evidence.pdf`.
- Evidence folder: `Drive -> Marketplace Evidence`.

## Common Pitfalls + Recovery

- Pitfall: the seller lists from a work tenant by mistake.
- Recovery: unpublish the listing and recreate it under `b2c-yejin-choi`.
- Pitfall: the listing has no DealSet id.
- Recovery: reopen `Deal terms`, set `dealset-vintage-jacket-2026-001`, and validate before publishing.
- Pitfall: the buyer checks out with a work payment method.
- Recovery: cancel the order and reorder from `Personal - Aiyana Ross`.
- Pitfall: seller acceptance captures payment before buyer authorization appears.
- Recovery: stop fulfillment and inspect the payment intent; capture requires prior authorization.
- Pitfall: the item quantity is greater than one.
- Recovery: set quantity to `1` for this tutorial because the SKU is a unique vintage item.
- Pitfall: the seller uploads a photo with a home address visible.
- Recovery: replace it with `jacket-front-redacted.jpg` and rerun media checks.
- Pitfall: tax estimate is missing.
- Recovery: refresh checkout and confirm tax line `USD 7.84` before placing the order.
- Pitfall: payout net does not match `USD 77.40`.
- Recovery: expand the fee calculation and verify fee, shipping, and tax rows.
- Pitfall: the buyer opens a dispute during the happy path.
- Recovery: resolve or cancel the dispute before verifying settlement.
- Pitfall: fulfillment is recorded without a tracking number.
- Recovery: edit fulfillment and enter `OYATIE-GROUND-20260520-001`.
- Pitfall: evidence export lacks ontology object ref.
- Recovery: rerun DealSet projection or inspect ontology projection health.
- Pitfall: evidence export lacks audit seal.
- Recovery: wait for audit-chain seal or block payout release for high-risk deals.
- Pitfall: the marketplace fee is charged twice.
- Recovery: open a FinOps correction and settle through a compensating DealSet transition.
- Pitfall: the seller deletes the listing after sale.
- Recovery: archive only; ADR-0314 requires audit-preserving settlement history.
- Pitfall: a duplicate buyer acceptance appears.
- Recovery: verify idempotency key collapse and keep only one accepted DealSet transition.
- Pitfall: PSP outage occurs during capture.
- Recovery: state should become `settlement_deferred`; do not manually mark settled.
- Pitfall: the query reports `counterparty_roles_count=1`.
- Recovery: inspect DealSet roles and repair the missing buyer role before settlement verification.
- Pitfall: the buyer cannot confirm delivery.
- Recovery: ensure shipment state is `Fulfilled` and buyer is viewing the correct personal tenant.

## DealSet Settlement Evidence Checklist

Use this checklist to prove the sale completed as an ADR-0314 DealSet.

- DealSet id should be `dealset-vintage-jacket-2026-001`.
- Listing id should be `listing-vintage-jacket-2026-001`.
- Seller should be `yejin.choi@example.com`.
- Buyer should be `aiyana.ross@example.com`.
- Item title should be `Vintage canvas field jacket`.
- Offer price should be `USD 98.00`.
- Shipping charge should be `USD 12.00`.
- Tax estimate should be `USD 7.84`.
- Marketplace fee should be `USD 20.60`.
- Seller payout should be `USD 77.40`.
- Tracking number should be `OYATIE-GROUND-20260520-001`.
- Deal state should progress `listed -> ordered -> accepted -> fulfilled -> delivered -> settled`.
- Audit event `DealSetCreated` should precede `MarketplaceListingPublished`.
- Audit event `BuyerOrderPlaced` should include the buyer tenant context.
- Audit event `SellerOrderAccepted` should include the seller tenant context.
- Audit event `SettlementCaptured` should include PSP reference.
- Audit event `PayoutReleased` should include payout net.

The settlement packet should contain four artifacts.

1. Listing snapshot.
2. Buyer order receipt.
3. Fulfillment record.
4. Settlement ledger entry.

The listing snapshot proves what was offered.

The buyer receipt proves what was purchased.

The fulfillment record proves delivery handoff.

The settlement ledger entry proves value movement.

Do not mark the tutorial complete with only the listing screenshot.

The user goal is not to create a listing.

The user goal is to sell, accept, fulfill, and settle.

If settlement is deferred, document `settlement_deferred` and stop before claiming success.

If payout is adjusted, include the adjustment id in the evidence export.

If buyer delivery confirmation is missing, do not force payout.

This tutorial is complete when the query returns `dealset_settled=true`.

## Next Tutorials

- [Set up a cross-tenant messenger channel](cross-tenant-channel-setup.md).
- [Project Salesforce CRM data into Oyatie ontology](ontology-projection-from-external-source.md).
- [Use intelligence to summarize a contract](ai-assisted-document-summarization.md).
- [Handle a GDPR erasure request](data-subject-erasure-request-handling.md).

## References

- [ADR-0314 Marketplace as Universal Deal Settlement](../decisions/ADR-0314-marketplace-as-universal-deal-settlement.md).
- [Marketplace deal settlement flow](../architecture/diagrams/marketplace-deal-settlement-flow.md).
- [Marketplace listing and first sale journey](../user-journeys/j23-marketplace-listing-and-first-sale/README.md).
- [Marketplace purchase as buyer journey](../user-journeys/j24-marketplace-purchase-as-buyer/README.md).
- [Marketplace listing takedown runbook](../runbooks/saas/marketplace-listing-takedown.md).
- [Capability Tier Over Product Fragmentation ADR](../decisions/ADR-0316-capability-tier-over-product-fragmentation.md).
- [Cedar Policy Evaluation Flow](../architecture/diagrams/cedar-policy-evaluation-flow.md).
- [Documentation Rigor](../standards/documentation-rigor.md).
- [Doc Style](../standards/doc-style.md).
