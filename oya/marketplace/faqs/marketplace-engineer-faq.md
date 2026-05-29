# `marketplace` µservice — Marketplace Engineer FAQ

20 real questions raised against the µservice that owns Oyatie's universal deal-settlement surface.

---

**Q1. Why one µservice across plugins/apps/workflows/agents/models/datasets?**

ADR-0249 + ADR-0314: the **settlement primitive is identical** across categories. Escrow, payouts, taxes, KYC, disputes — these
don't change based on what you're selling. Separating them per-category would duplicate code, fragment audit trails, and force
cross-category reconciliation. One µservice with category-specific `payload_schema` is the cleanest design.

---

**Q2. Does `marketplace` replace `payments`?**

No. `payments` handles raw card processing, payment-method storage, refunds at the rail level. `marketplace` is built **on top** of
`payments`: when a buyer hits "Buy", marketplace calls `payments::charge`, gets a payment intent ID, and tracks it through the
escrow lifecycle.

---

**Q3. How does escrow work?**

Funds enter escrow at charge-time. The state machine:
- `Pending` — payment intent in flight.
- `Held` — payment succeeded; funds in platform escrow account.
- `Released` — escrow window expired or buyer released; payout queued to seller.
- `Refunded` — buyer requested refund within escrow window; funds returned.
- `Disputed` — buyer or seller raised a dispute; funds frozen until resolution.

Each transition is Cedar-gated.

---

**Q4. Who is the seller-of-record for tax purposes?**

Tenant-class dependent. `demo_trial` tenants cannot run live taxable settlement. `paid` tenants use marketplace facilitator handling
for digital VAT/sales tax in supported jurisdictions; we calculate, collect, remit. Listing owner still gets a 1099-K or equivalent.

---

**Q5. What tax engine do we use?**

Multiple, per region:
- Stripe Tax for US sales tax + EU VAT (paid default).
- Avalara for high-volume paid tenants with custom needs.
- TaxJar for SMB.
- Direct integration with KR National Tax Service for sovereign-KR paid tenants.

The choice is per-tenant Cedar-policy override; the abstraction is `crates/oya-marketplace-port-tax/`.

---

**Q6. How is KYC done?**

`identity` µservice handles the KYC artifact storage; `marketplace` references it by ID. The tenant-class shape is:
- `demo_trial`: no live payout; mock KYC only.
- `paid`: Persona or Onfido document scan + selfie + bank, with UBO tracing above risk and volume thresholds.
- Sovereign paid tenants: regulator-cleared KYC partners where the active pack requires them.

---

**Q7. What if a regulator-cleared KYC partner is required for a sovereign tenant?**

Sovereign tenants pick their own KYC partner from a regulator-approved list. The partner integration is per-tenant; we ship adapters
for the most common partners (Trulioo, Onfido, Persona, Sumsub, Jumio, KR FSC partners).

---

**Q8. How does dispute resolution work?**

Three layers:
1. **Auto-rules** — declarative `DisputeRule` impls handle clear-cut cases (e.g. auto-refund for low-value undelivered).
2. **Tenant-resolved** — the platform invites both parties to evidence; deadline-driven; tenant resolves.
3. **Platform-escalated** — disputes above value thresholds or with regulator implications go to the platform dispute team.

Each layer is Cedar-gated. Sovereign tenants can disable layer 3 by policy.

---

**Q9. How is platform fee calculated?**

Per `ListingPricingModel`:
- `OneTime`: platform fee on charge.
- `Subscription`: platform fee on each recurring charge.
- `UsageBased`: platform fee on metered settlement at billing-period close.
- `Enterprise`: platform fee per contract terms (negotiated).

The fee is recorded as a separate ledger entry so auditors see gross transaction + platform fee + net-to-seller distinctly.

---

**Q10. How are payouts scheduled?**

Per tenant_class and billing_components. `demo_trial` has no live payout; paid defaults to weekly payouts, can move to daily or
realtime through policy-gated billing components. Payout runs via `payments` payout rails (ACH, SEPA, faster payments, etc). Failed
payouts queue for retry with exponential backoff.

---

**Q11. Can a seller delist?**

Yes, but listings have a retention period:
- `demo_trial`: 30 d after delisting for mock listings.
- `paid`: 90 d default.
- `paid` with regulated packs: policy-defined retention, often 7 y for financial-related listings.

Hard delete requires regulatory clearance.

---

**Q12. What's the relation to `plugin-app-store`?**

`marketplace` is the settlement engine; `plugin-app-store` is the discovery + install surface for the `plugin` listing category.
Same separation for `workflow-studio` (workflows), `ontology` (datasets), `intelligence` (models), `connector` (apps), `foundry`
(agents). Discovery surfaces own UX; settlement is centralized.

---

**Q13. How do we handle category-specific payload schemas?**

Each category has a `payload_schema` enforced at listing creation:
- Plugin: WASM binary + manifest + screenshots + permissions declaration.
- Workflow: JSON workflow definition + version + required µservice dependencies.
- Agent: agent persona definition + tools list + model requirements.
- Model: model weights manifest + license + benchmark scorecards.
- Dataset: dataset card + sample + license + governance.
- App: app manifest + screenshots + permissions.

The schemas live in `crates/oya-marketplace-domain/src/payload_schemas/`.

---

**Q14. Can the same listing have multiple pricing tiers?**

Yes via `PricingModel::Tiered { tiers: Vec<PricingTier> }`. The buyer picks at checkout. Subscription tiers are first-class.

---

**Q15. How does refund work?**

Buyer-initiated within escrow window: auto-refund unless seller objects within 48 h. Outside escrow window: requires dispute. The
refund itself flows through `payments::refund`; marketplace records the ledger entries.

---

**Q16. How is the audit chain different from `payments`'s audit?**

`payments` audits card events (charge, refund). `marketplace` audits commercial events (listing, purchase, escrow, dispute, payout).
The two chains are linked: every marketplace audit event referencing a payments event includes the payments `intent_id` for cross-chain
trace.

---

**Q17. What's the throughput ceiling?**

Listing creation: 50 listings/s tenant-wide for `demo_trial`, scaling to 100k+ for paid tenants with per_usage capacity policy.
Transactions: 10/s for `demo_trial`, 5,000/s for paid default, and policy-governed higher ceilings for paid tenants. Ledger writes
are partitioned by `(tenant_id, year_month)` for write parallelism.

---

**Q18. How do enterprise contracts work?**

`PricingModel::Enterprise` is a contract reference + custom billing terms. The contract lives in `crm` µservice; marketplace
references it. Invoicing flows through `cloud-billing-tax` µservice. Settlement happens off-platform with platform-fee reconciliation
on close.

---

**Q19. Are sovereign listings isolated?**

Yes. A sovereign tenant's listings live in their sovereign cells only; non-sovereign tenants cannot see them; sovereign-region
data residency is enforced. Cross-listing search filters by tenant + sovereign attestation.

---

**Q20. What's the provider-credential BYOK story for marketplace (ADR-0255 §D-4)?**

Per `feedback_byok_everywhere_credentials.md` + ADR-0255 §D-4: a listing that wraps an external provider (e.g. an OpenAI model
listing wrapping the OpenAI API) can declare `byok_required: true`. Buyers must then supply their own OpenAI credentials at purchase
time; the listing seller never sees buyer credentials.
