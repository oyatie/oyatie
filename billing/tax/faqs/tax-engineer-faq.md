# `cloud-billing-tax` µservice — Tax Engineer FAQ

20 real questions raised against `cloud-billing-tax` (the µservice that owns Oyatie's tax-calculation substrate).

---

**Q1. Does `cloud-billing-tax` replace Avalara AvaTax / Vertex O Series / TaxJar / Stripe Tax?**

For Oyatie-tenant workloads — yes. The catalog covers all major jurisdictions, the calculation engine is in-process at paid,
and the filing artefacts match (or exceed) what AvaTax Returns / Vertex Filing produces. For tenants whose external billing
systems already call Avalara, we ship a passthrough adapter (`cloud-billing-tax-adapter-avalara-*`) but the canonical path is
`cloud-billing-tax` direct.

---

**Q2. How does the catalog compare to Avalara's?**

Avalara ships ~22,000 tax codes globally; the `oya-tax-codes-sovereign-paid-v1` catalog ships ~9,800. The Oyatie catalog is
narrower-by-design (modern SaaS-and-services centric); we have parity on SaaS, professional services, digital goods, telecom,
and physical-goods e-commerce. Avalara wins on niche categories (alcohol fuel taxes by state, cannabis tax by county). We file
new codes via `oya tax codes propose` with governance review.

---

**Q3. What's the calculation engine architecture?**

Two-tier:
1. **In-process Cedar tax engine** (paid) — runs the rate lookup + nexus check + exemption check via Cedar policies, returns
   in ≤ 14 ms p95. Handles 95 % of paths.
2. **Out-of-process tax kernel** — for complex multi-line, multi-jurisdiction, multi-currency calculations (e.g. a US sale
   shipping internationally with duty implications), invoked over HTTP/3 to the kernel cell, ≤ 28 ms p95.

The split is transparent — clients call `tax.calculate(...)` and the SDK routes.

---

**Q4. How are tax rates kept current?**

Rate cards are versioned weekly. The source pipeline:
1. **State/country bulletins**: ingested from primary sources (CA CDTFA bulletins, IRS Pub 510, EU VAT Directive amendments, UK
   HMRC notices, BR NCM tables, etc.) via scrapers + manual editors.
2. **Validation pass**: ML-assisted change-detection vs prior week + human review for rate moves.
3. **Cedar lint**: every rate change is reviewed against the existing Cedar policies for impact.
4. **Stage rollout**: shadow run on a synthetic transaction corpus for divergence vs Avalara/Vertex (sanity check).
5. **Publish + audit**: rate-card-version-hash anchored on `audit-chain`.

Median lag from authority bulletin to rate-card publish: 4 d. SLA: ≤ 14 d at paid, ≤ 21 d at Paid.

---

**Q5. How does economic nexus tracking work?**

Each tenant carries a per-jurisdiction nexus profile. Every taxable transaction increments the YTD-sales + YTD-transactions
counters for the destination jurisdiction. When either crosses 80 %, an "approaching-threshold" alert fires; at 100 %, the
"breached" event fires and a 30-day grace timer starts. After 30 d, calculations for that jurisdiction default to "tax-due"
even if the tenant hasn't registered (Cedar permit `cloud_billing_tax::Action::ApplyPostNexusTax` enforces).

---

**Q6. What if a tenant disputes a tax outcome?**

`cloud_billing_tax::Action::DisputeTaxAssessment` allows the tenant to flag a calculation as disputed. The disputed line is
quarantined (still recorded but not included in filing artefacts) until reviewer-agent + governance resolve. Resolution leads
to either (a) confirm calculation (tax stands), (b) credit memo issued via `cloud-billing`, or (c) catalog correction (rate
card or tax code fix).

---

**Q7. Can the same line item be subject to multiple taxes?**

Yes — most jurisdictions are layered. Example: a Texas sale incurs:
- TX state sales tax (6.25 %)
- TX county tax (0.5 % typical)
- TX city tax (1-2 %)
- Transit authority tax (1 % in some cities)
Combined effective rate up to 8.25 %. Each layer is a separate tax line in the response.

---

**Q8. How are EU VAT One-Stop Shop (OSS) sales handled?**

For B2C cross-border digital sales within the EU, the seller charges the buyer's country's VAT rate. `cloud-billing-tax`
detects the buyer's jurisdiction (country-of-residence per EU 282/2011 Art. 24-bis evidence rules — IP, billing address, payment
country, etc.) and applies the correct rate. The quarterly OSS filing artefact aggregates all EU sales into a single XML
submitted to the seller's home country tax authority (which then forwards proportionally).

---

**Q9. How does India GST handle inter-state vs intra-state?**

- **Intra-state** (same state): CGST (Central) + SGST (State), typically 9 % + 9 % = 18 %.
- **Inter-state**: IGST (Integrated), typically 18 %.
- **Union Territory**: CGST + UTGST.

The calculator picks based on `seller_state` vs `buyer_state`. Special economic zones (SEZ) are zero-rated. Composition-scheme
sellers have separate brackets. All this is encoded in `oya-tax-codes-global-paid-v1`'s IN-specific section.

---

**Q10. How are exemption certificates validated?**

Multi-stage:
1. **OCR extraction** of the certificate image/PDF (Amazon Textract or in-house equivalent).
2. **Cross-check against issuer database** where available (TX Comptroller, CA CDTFA, etc.). For states without a public lookup
   API, the certificate is accepted "as filed" with audit-trail.
3. **AAD-bound encryption** under `cloud-kms` with `(tenant_id, customer_id, jurisdiction)` as AAD.
4. **Expiry tracking**: certificates with `valid_through` < now refuse to apply.
5. **Renewal reminders**: 60 d before `valid_through`, the tenant gets a `comms-email` notification.

---

**Q11. What's the e-invoicing story?**

E-invoicing is mandatory in 30+ countries (IT SDI, BR NF-e, MX CFDI, IN GST, EG e-Receipt, SA ZATCA, PH BIR EIS, TR e-Fatura,
KR e-Tax, etc.). At Paid/Paid, `cloud-billing-tax`:
1. Generates the country-specific e-invoice format (XML in most cases).
2. Submits to the country's clearance system (SDI for IT, NFe.fazenda for BR, NTS for KR, etc.).
3. Receives the clearance receipt + invoice authorization number (IRN in IN, CFDI UUID in MX, etc.).
4. Anchors the receipt on `audit-chain`.

EU ViDA 2030 mandate is fully supported — paid tenants migrate seamlessly to the 2030 cross-border B2B e-invoicing model.

---

**Q12. How does cache invalidation work on rate-card updates?**

When a rate-card version publishes, the cache invalidator emits a `cloud_billing_tax.rate_card.published` event. Cells that
hold cache entries for affected (jurisdiction, tax_code) tuples evict on the next request matching the tuple. The shadow-window
(24 h) ensures in-flight calculations from the prior rate card remain reproducible for reconciliation.

---

**Q13. What if a calculation fails (e.g. rate-card row missing)?**

Three outcomes by tenant_class:
- **DemoTrial**: returns `TaxError::RateMissing` and refuses the calculation; the calling app must surface this to the user.
- **Paid**: applies a "safe-conservative" fallback (highest historical rate for the jurisdiction in trailing 13 months) and
  flags the line for review.
- **Paid/Paid**: reviewer-agent ticket auto-opens; calculation pauses until resolved; SLA 2 h for resolution at Paid,
  20 min at Paid.

---

**Q14. How is sourcing (origin-based vs destination-based) determined?**

By jurisdiction policy. US: most states are destination-based (TX, CA, NY, FL, etc.); a few are origin-based (AZ, IL, MS, MO,
OH, PA, TN, TX intra-state, UT, VA, WI — varies by transaction type). EU: destination-based for B2C, origin-based for B2B
(reverse charge). The engine encodes this per-jurisdiction-per-transaction-type.

---

**Q15. How does the engine handle SaaS-specific edge cases?**

- **CA SaaS taxability**: not taxable as of 2026-05 (tracked).
- **TX SaaS taxability**: taxable (Texas considers it data processing, 6.25 % + 2 % local).
- **NY SaaS**: taxable since 2008.
- **CO Software-as-a-Service**: not taxable but custom software is.
- **EU SaaS**: taxable, OSS applies for B2C.
- **JP SaaS (consumption tax)**: taxable, reverse charge for B2B foreign sellers.
- **AU GST on imported services**: 10 %, registration required if AUD 75k+ to AU customers.

All encoded in the tax-code catalog with examples.

---

**Q16. Can `cloud-billing-tax` calculate withholding tax?**

Yes — withholding tax (e.g. royalty WHT, services WHT, treaty rates) is supported as a separate `withholding` tax kind.
The engine applies the relevant treaty rate (OECD model, US-IRC §1441-§1443, etc.) and emits a separate filing artefact
(US 1042-S, IN TDS Form 26Q, etc.).

---

**Q17. How does this work for digital products with variable rates by content?**

E-books (UK 0 %, IE 9 %, FR 5.5 %, DE 7 %, USA varies by state), streaming (different rates than downloads in some jurisdictions),
in-app purchases (PSD2 in EU separates payment service from content). The catalog has per-content-category codes — e.g.
`SW054001` SaaS-general, `SW055002` SaaS-with-physical-component, `EB060001` e-book, `ST070001` streaming-subscription-monthly.

---

**Q18. How are refunds handled tax-wise?**

A refund is a negative calculation with the same `calculation_id_parent` pointing to the original calculation. The engine
applies the same rate that was effective at the original calculation time (rate locked by version). The refund is included
in the filing artefact for the period of the refund (not the original calculation).

---

**Q19. Where does Foundry hook in?**

Foundry pipelines that handle billing-tax dataset updates (rate card publishes, tax code additions) run as
`oyatie.foundry.<pipeline-id>` principals with narrow Cedar permits: `ProposeRateCardVersion`, `LintRateCard`, `PublishRateCard`.
No `EmergencyTaxReversal` or `IssueSovereignEInvoice`.

---

**Q20. How is the µservice tested for jurisdiction correctness?**

Each rate-card publish triggers a 50,000-transaction synthetic corpus run that:
1. Compares the new rate-card results against the prior version for divergence.
2. Compares results against Avalara/Vertex/TaxJar sandbox responses (vendor parity test).
3. Spot-checks against published tax tables from primary sources.

Divergence > 0.5 % from any sanity baseline blocks the publish. Test corpus + ground truth lives at
`crates/oya-cloud-billing-tax-test-corpus-v1/`.
