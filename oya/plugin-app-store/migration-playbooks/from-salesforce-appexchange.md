---
doc_class: MigrationPlaybook
microservice: plugin-app-store
source_vendor: Salesforce AppExchange
related_adrs: [ADR-0316, ADR-0249, ADR-0251]
date: 2026-05-20
doc_status: published
---

# Migration Playbook — Salesforce AppExchange → oyatie plugin-app-store

Audience: an ISV (Independent Software Vendor) currently selling on Salesforce AppExchange who wants to publish equivalent listings on oyatie's marketplace, either in addition to or instead of AppExchange.

Outcome: AppExchange listings reproduced as oyatie listings with comparable feature parity, SBOM emission, payment processing, multi-currency support.

## Phase 0 — discovery (week 1)

1. Inventory AppExchange listings:
   - Listing IDs + names.
   - Pricing model (free / one-time / subscription).
   - Customer base + geographic distribution.
   - Salesforce-version compatibility matrix.
   - External dependencies + integrations.
2. Identify reusable assets:
   - Marketing copy (rewrite for oyatie host product, not Salesforce).
   - Screenshots (may need refresh; oyatie UI differs from Salesforce).
   - Security questionnaires (re-use for oyatie review).
   - Customer testimonials.
3. Identify host-product translation requirements:
   - Salesforce listing extends Salesforce; oyatie listing extends oyatie's host µservice (e.g. `crm`, `docs`, `cloud-billing-tax-app`).
   - Significant code changes likely required: Apex (Salesforce-proprietary) → Rust / TypeScript / Python; Visualforce / LWC → oyatie's UI framework or external-app pattern.

Deliverable: `migration-plan.md` with code-change estimate per listing.

## Phase 1 — code port (weeks 2-8)

Reality: AppExchange listings are typically deeply embedded in Salesforce's runtime (Apex, Lightning Component Framework, Visualforce). Most are not portable as-is to oyatie. You essentially rebuild the product.

Two common patterns:
1. **External app + integration**: keep your Salesforce listing for Salesforce customers; build an oyatie-native equivalent + integrate with oyatie's relevant µservice (most likely `crm` since AppExchange is CRM-centric). Each "listing" is a separate product line.
2. **Re-architecture**: rebuild your product as oyatie-native from the start, treating Salesforce as deprecated. Higher upfront cost but lower long-term maintenance.

For pattern 1, your oyatie listing is essentially a new product. For pattern 2, you maintain two codebases for a transition period.

## Phase 2 — oyatie listing authoring (weeks 9-12)

Follow `tutorials/publish-paid-plugin-with-sbom-and-stripe.md` for the publishing workflow:
1. Initialise listing project.
2. Author `oma.json` manifest.
3. Build container artifact + emit SBOM.
4. Run security + license scans.
5. Author marketing assets.
6. Submit for review.

Map your AppExchange pricing model to oyatie:
- AppExchange "subscription" → oyatie subscription pricing.
- AppExchange "per-user/per-month" → oyatie subscription with tenant-user count metric.
- AppExchange "free" → oyatie free offering.
- AppExchange "freemium" → oyatie billing_components options (free + paid).

## Phase 3 — customer migration (weeks 13-16)

For each existing AppExchange customer:
1. Email outreach: "We've launched [Product] on oyatie. If you're an oyatie user (or planning to be), here's how to install: [link]."
2. Offer migration support: tenant-data migration, configuration transfer, training.
3. Discount or free trial extension for early adopters.

Most customers won't migrate immediately; expect 6-12 month tail before significant revenue shifts to oyatie.

## Phase 4 — dual-marketplace operation (weeks 17+)

Long-term, you likely maintain both:
- AppExchange listing for Salesforce customers (continues earning).
- oyatie listing for oyatie customers (growing).

Consolidate gradually if/when oyatie revenue exceeds AppExchange.

## Common pitfalls

| Pitfall | Mitigation |
|---|---|
| Apex business logic that's Salesforce-platform-bound (custom objects, formulas, triggers) | Reimplement in Rust/TypeScript; this is the largest cost of migration |
| Lightning UI components | Rebuild using oyatie's UI patterns; or build as a standalone external app |
| Customer data in Salesforce custom objects | Provide migration tools / API for customer self-migration; expect resistance from customers happy with Salesforce |
| Salesforce-specific permissions (Profiles, Permission Sets) | Map to Cedar permits in oyatie; design before publish |
| Salesforce Native AppExchange (Lightning Vault) — must "package" everything | oyatie uses container images + manifest; simpler but different lifecycle |
| AppExchange security review (ISV pre-built) | Submit equivalent security questionnaire to oyatie; many similarities (OWASP, encryption, data handling) |
| Revenue share differs (Salesforce 15-25% vs oyatie default 30%) | Negotiate oyatie's revenue share down (paid billing_components tier allows up to 85/15 for partners) |
