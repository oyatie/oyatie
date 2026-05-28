---
doc_class: User-Journey-README
journey_id: j153-devon-williams-hvac-side-business-tax-end-of-year
slice: gray-collar-gig-economy-dual-tenant-tax-year-end
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Devon Williams
audience_type: B2B_CONTRACTOR
microservice_count: 5
pack_overlay_anchor: US-IRS-Schedule-C + US-Form-1099-K + US-State-CA-CDTFA + ISO-20022
related_adrs:
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0243-cedar-as-universal-gate
  - ADR-0263-observability-emission-contract
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0317-role-based-projection-unified-ux-shell
  - ADR-0255-byok-everywhere-credentials
  - ADR-0249-multi-category-marketplace-doctrine
  - ADR-0314-marketplace-as-universal-deal-settlement
---

# j153 — Devon Williams: HVAC side-business and tax year-end

## At a glance

Devon Williams is an HVAC Field-Service Technician with two simultaneous tenant identities:

1. **Devon-at-work** — W-2 employee of **Bayshore Climate Systems Inc.** (his primary employer; commercial-HVAC service in the SF Bay Area)
2. **Devon-as-handyman-side-business** — single-member LLC `Devon Williams HVAC LLC` (sole-proprietor side business, registered in California, EIN 88-2904471; takes residential HVAC repair gigs evenings + weekends)

It is **December 28**, the Monday after Christmas, 19:42 PST. Devon is at the kitchen table in his rental in Hayward, CA. He has 4 days until end-of-tax-year. He has to:

1. Reconcile his **2026 side-business income** across the three places it lived: `payments` (Stripe-Connect-style direct deposits), Venmo (legacy, 14 jobs Q1-Q2), Zelle (3 jobs from one repeat customer)
2. Categorize and tag **187 receipts** from the year — gas, tools, parts, vehicle maintenance, supplies — into Schedule-C-conformant categories via `finops-portal`
3. File his **mileage log** from `tasks` (the side-business job log) into the IRS-recognized format (54.5¢/mi for 2026)
4. Use `workflow-studio` to set up an **end-of-year automation** so that next year's reconciliation runs nightly instead of one frantic December evening
5. Use `connector` to push the consolidated Schedule-C data to his tax preparer (Cookson Tax & Accounting, Hayward) over a secure, audit-sealed channel — without exposing the underlying Bayshore W-2 (which is Devon-at-work's tenant data, not Devon-as-side-business's)

Five microservices: `payments`, `finops-portal`, `tasks`, `connector`, `workflow-studio`. Secondary touches on `identity` (tenant-switching), `tenancy` (dual-tenant boundary per ADR-0311), `audit-chain`, `compliance` (CA-CDTFA sales tax for installed parts), `marketplace` (Devon found 3 of his side gigs through the oya neighbor-marketplace surface), `community` (the customer reviews from his repeat customers shape his SEO discoverability).

The Cedar policy enforces **dual-tenant strict separation** (ADR-0311): Bayshore's W-2 payroll data never leaks into `Devon Williams HVAC LLC`, and the LLC's customer list never leaks into Bayshore.

## Why this journey matters

Devon Williams is **MASTER-ROSTER §3.2 row 15** — the canonical gray-collar gig-economy persona. The journey closes:

- Critical-path row 8 (gray-collar field worker with dual-tenant identity)
- Critical-path row 9 (income reconciliation across multiple payment rails for self-employment tax)

Hyperscaler benchmark: Stripe for the Connect-style payments + Stripe Tax for 1099-K threshold tracking; QuickBooks Self-Employed for the Schedule-C categorization; MileIQ for the mileage log; Plaid Exchange for the bank reconciliation; Zapier for the workflow-studio automation.

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| `story.md` | Narrative beat-by-beat from 19:42 PST through 01:14 next-day; specific transactions; specific tax categories; specific dollar amounts | Every beat names the customer, the date, the dollar, the Cedar tenant boundary check, the audit event class |
| `ux-flow.md` | Screen progression on Devon's iPhone 16 + iPad mini reconciliation surface | Every screen names the tenant-switcher pill, the Schedule-C category picker copy, the receipt-OCR confirmation modal |
| `handshake.md` | Per-microservice contract with named API call, request body shape, response shape, error paths | Each row enumerates the gRPC method or HTTP route, the proto3 message field set, the Cedar permit |
| `integration-test-plan.md` | Concrete pass/fail criteria | Each test names the seed transactions, the expected event chain, the failure-injection trigger |
| `schemas/openapi-finops-reconcile.json` | OpenAPI 3.2.0 for `POST /v1/tenants/{tenant_id}/finops/year-end-reconcile` | Schedule-C category enum + 1099-K threshold field |
| `schemas/schedule-c-category-map.yaml` | Source-transaction-type → Schedule-C line mapping | Per-line IRS Schedule-C 1040 reference |
| `schemas/journey-messages.proto` | proto3 for the 7 core RPC messages | Field tags, enum values |
| `schemas/cedar-policy.cedar` | Cedar policy with ADR-0311 dual-tenant strict-separation rule | Forbid cross-tenant data leak |
| `schemas/asyncapi-payments-ledger.yaml` | AsyncAPI 3.1.0 for the `payments.ledger.transactions_v1` topic | Per-tenant partitioning |

## The five microservices in scope

| µservice | Role in this journey | Critical-path row |
|---|---|---|
| `payments` | Holds Devon's side-business payment-rail receipts (Stripe deposits, plus the imported Venmo and Zelle reconciliations); produces the canonical 2026 income statement | row 9 |
| `finops-portal` | Categorizes 187 receipts; computes the Schedule-C 1040 lines; tracks the 1099-K $2,500 threshold for 2026 (per ARPA changes); flags the CA-CDTFA sales tax obligation on installed parts | row 9 |
| `tasks` | Hosts the side-business job log including the mileage log; produces the IRS-recognized 54.5¢/mi summary | row 8 |
| `connector` | Pushes the consolidated Schedule-C export to Cookson Tax & Accounting; manages the tax-preparer share-link with a per-document watermark | row 9 |
| `workflow-studio` | Enables Devon to author the nightly reconciliation automation; visual flow editor for the gray-collar persona | row 8, row 9 |

## Secondary microservices touched

| µservice | Touch reason |
|---|---|
| `identity` | Resolves the dual-tenant switch (Devon-at-work vs Devon-as-handyman) with strict ADR-0311 isolation |
| `tenancy` | Holds the two tenant principals: `bayshore_climate_systems` (Devon is an employee resource) and `devon_williams_hvac_llc` (Devon is the sole-proprietor admin) |
| `audit-chain` | Seals every cross-tenant boundary check; seals the tax-export disclosure |
| `compliance` | Activates the US-IRS-Schedule-C pack overlay + the US-Form-1099-K pack + the CA-CDTFA pack |
| `marketplace` | Records the 3 gigs Devon found via the neighbor-marketplace; their fees are auto-categorized as Schedule-C line 10 (commissions and fees) |
| `community` | Holds Devon's customer reviews; the 12 5-star reviews feed his discoverability score |

## Pack overlays

| Pack | Activation reason |
|---|---|
| US-IRS-Schedule-C | Sole-proprietor income tax reporting; Schedule-C 1040 line-item enumeration |
| US-Form-1099-K | 2026 threshold ($2,500 per ARPA 2021 as amended by 2025 reconciliation act); Stripe issues 1099-K when threshold crossed; Venmo + Zelle threshold-tracking imports |
| US-State-CA-CDTFA | California sales tax on tangible personal property installed in HVAC repairs; District tax for Alameda County (Hayward) |
| ISO-20022 | Cross-rail payment reconciliation; canonical message format for the bank-import path |

## Regulatory anchors

1. IRS Schedule-C 1040 (Profit or Loss from Business — Sole Proprietorship)
2. IRS Form 1099-K threshold $2,500 for tax year 2026 (per the ARPA 2021 amendment + 2025 reconciliation act adjustment); Stripe issues 1099-K to Devon if his side-business gross crossed the threshold
3. IRS Pub. 463 standard mileage rate 54.5¢/mi for 2026
4. CA-CDTFA District Tax Lookup for Hayward (Alameda County base 7.25% + Alameda district 2.75% = 10.25% for 2026)
5. ADR-0311 dual-tenant identity strict separation (no W-2 ↔ LLC cross-leak)
6. ADR-0244 tenant scoping invariant
7. ADR-0263 audit-event classes

## Cell + certification matrix

| Cell | Certification | Journey use |
|---|---|---|
| `us-west-2-primary` | SOC2 + PCI-DSS Level-1 | Primary placement for both Bayshore and Devon Williams HVAC LLC tenants |
| `us-west-2-finops-cell` | SOC2 + IRS-1075 (tax-data handling) | Hosts the Schedule-C compute + the 1099-K threshold ledger |
| `global-shared-control-plane` | SOC2 | Hosts the `connector` share-link to Cookson Tax |

## Cedar permit class (excerpt — full text in `schemas/cedar-policy.cedar`)

```cedar
// Devon's side-business tenant gives him full admin
permit (
    principal == User::"devon.williams@oyamail.network",
    action,
    resource is Tenant
) when {
    resource.tenant_id == "devon_williams_hvac_llc" &&
    principal.role_in_tenant("devon_williams_hvac_llc") == "sole_proprietor_admin" &&
    principal.passkey_step_up_within_seconds(300)
};

// Strict deny on cross-tenant data flow into the side-business
forbid (
    principal,
    action == Action::"finops.import_transactions",
    resource is Tenant
) when {
    context.source_tenant_id == "bayshore_climate_systems" &&
    resource.tenant_id == "devon_williams_hvac_llc"
};

// Tax-preparer share — requires named preparer, scoped to year, scoped to fields
permit (
    principal == User::"devon.williams@oyamail.network",
    action == Action::"connect.tax_preparer_share",
    resource is Tenant
) when {
    resource.tenant_id == "devon_williams_hvac_llc" &&
    context.tax_year == 2026 &&
    context.preparer_id == "cookson-tax-accounting-hayward-ca-prep-id-7741" &&
    context.share_scope_fields containsAll ["schedule_c_lines", "1099_k_summary", "mileage_total"] &&
    context.share_scope_fields notContains "customer_pii"
};
```

## Why ADR-0311 strict separation is non-negotiable

Devon's W-2 income from Bayshore is reported on his **personal 1040** (not Schedule C). If Bayshore's payroll data were imported into the Devon Williams HVAC LLC tenant's reconciliation, his Schedule-C would double-count his W-2 wages — a felony-grade tax filing error. ADR-0311 forbids this at the Cedar layer and at the data-layer of `finops-portal`. Every cross-tenant import attempt emits `EVT-J153-CEDAR-DENY-W2-INTO-SCHED-C`.

The acceptable cross-tenant signal is **at the personal-1040 level**, where Devon (as the human) combines both — but that combination happens in Cookson Tax's system, not in oya. Oya emits Devon's side-business Schedule-C and Devon's W-2 1040 wages separately and Cookson combines them.

## Acceptance summary

| AC | Result expected |
|---|---|
| AC-J153-001 | All 2026 Stripe deposits ($28,419.27 across 47 jobs) reconcile to side-business tenant `devon_williams_hvac_llc`; zero cross-tenant leak |
| AC-J153-002 | Venmo import ($4,217.50 across 14 Q1-Q2 jobs) deduplicates against any duplicate Stripe entry; 1099-K threshold tracked across rails |
| AC-J153-003 | Zelle import ($1,800 across 3 jobs from one repeat customer) categorized correctly; no duplicate; commingled-account warning surfaced if customer's Zelle name doesn't match invoice |
| AC-J153-004 | All 187 receipts categorized to Schedule-C lines 1, 8, 9, 10, 13, 22, 23, 27a per the IRS schedule; auto-categorization confidence ≥0.85 on 162/187, manual on 25/187 |
| AC-J153-005 | Mileage log from `tasks` (4,217 business miles across 73 trips, all log-correlated with a job ID) produces $2,298.27 deductible (4,217 × 54.5¢) |
| AC-J153-006 | CA-CDTFA sales tax on installed parts ($3,400 of installed-parts revenue at 10.25% Hayward rate = $348.50 collected; reconciled and remitted via the CDTFA filing path) |
| AC-J153-007 | workflow-studio automation `nightly-side-business-reconcile-v1` runs at 23:30 PST nightly thereafter; emits a success event by 23:42 PST 90% of the time |
| AC-J153-008 | Cookson Tax & Accounting receives a connect share-link containing exactly the Schedule-C JSON + 1099-K summary + mileage total; receives nothing else; watermark on every PDF page; share-link expires Jan 31 at 23:59 PST |
| AC-J153-009 | The same operations attempted by Devon-at-work (a different role projection) are denied; Cedar emits `EVT-J153-CEDAR-DENY-WRONG-ROLE` |
| AC-J153-010 | The audit-chain bundle of the year-end reconciliation is exportable in a single JSON-LD file signed by Devon's passkey |

## Cross-references

- Persona dossier: `docs/personas/devon-williams.md`
- MASTER-ROSTER §3.2 row 15
- Matrix §10 j153 recommendation
- Related: j159 (Saanvi Mehta MBA application spans personal/work — same dual-tenant pattern)
- Pack roster: `packs/us-irs/`, `packs/us-form-1099-k/`, `packs/us-ca-cdtfa/`
- ADR-0311 dual-tenant identity strict separation
- ADR-0244 tenant scoping
- ADR-0249 multi-category marketplace (the 3 gigs Devon found via marketplace)
- ADR-0255 §D-4 provider-credential BYOK (Devon optionally uses his own Stripe API key under platform_default mode)

## What this journey deliberately does NOT cover

- Devon-at-work's W-2 reconciliation (handled by Bayshore's HR; separate journey)
- Devon's personal 1040 filing (Cookson does that downstream)
- Self-employment tax / SECA calculation (Cookson does that)
- Quarterly estimated tax payments (separate journey)
- Devon's IRA / 401k contributions (separate journey)
- The CA Franchise Tax Board LLC $800 minimum franchise tax (paid earlier in the year)

## Stop condition

This journey is complete when all 10 acceptance criteria pass on the seeded test fixture, the schema files validate against their meta-schemas, every named ADR resolves, every named µservice exists, and the persona dossier matches MASTER-ROSTER §3.2 row 15.
