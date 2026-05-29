---
ip_id: IP-015
microservice: cloud-billing
title: Counterpart parity — Stripe Billing + Recurly + Zuora Billing + Chargebee
wave: Wave-15B-cloud-billing-spec-sprint
date: 2026-05-21
owner: axis-cloud-billing
status: drafted
priority: P0
binding_adrs: [ADR-0330, ADR-0145, ADR-0244, ADR-0263]
counterpart_parity: [Stripe Billing, Recurly, Zuora Billing, Chargebee, AWS Billing & Cost Management]
capabilities_touched: all
billing_components: [revenue_share, per_seat, per_usage]
tenant_class_scope: both
---

# IP-015 — Counterpart parity: Stripe / Recurly / Zuora / Chargebee

## §A Objective

Document cloud-billing's feature parity against the four named counterparts (Stripe Billing, Recurly, Zuora Billing, Chargebee) plus AWS Billing & Cost Management. This is the canonical capability-coverage ground-truth referenced by the Wave-4 audit and the 2026-05-20 feature-parity-matrix.

The reference deck:

- Stripe Billing — industry leader for SaaS subscription billing.
- Recurly — strong subscription primitives, ASC 606 revenue recognition built-in.
- Zuora Billing — enterprise-grade quote-to-cash for complex commercial models.
- Chargebee — flexible plan composition + metered components.
- AWS Billing & Cost Management — hyperscaler cost attribution + reservation flight-deck.

cloud-billing covers the UNION of these counterparts (per PRD §2.3 Outcome 11). Where cloud-billing exceeds counterparts, the delta is intentional (e.g. composable billing_components, cellular residency, Cedar-gated mutations).

## §B Scope

In scope:

- Feature-by-feature parity table covering ~50 distinct capabilities.
- Per-capability counterpart citation.
- Per-capability oyatie surface (which IP / file / Cedar gate).
- Delta classification: ✓ parity / ▲ exceeds / ▼ deferred (with reason).

Out of scope:

- Per-feature implementation detail (covered in sibling IPs).
- Per-feature SLO target (covered in `slos/` per-µservice).
- Pricing comparison (commercial; not engineering scope).

## §C Architecture

### §C.1 Capability coverage matrix

| Capability | Stripe Billing | Recurly | Zuora Billing | Chargebee | AWS Billing | Oyatie cloud-billing | Anchor |
|---|---|---|---|---|---|---|---|
| Subscription primitive | ✓ | ✓ | ✓ | ✓ | — | ✓ (SubscriptionApi RPCs; proto3 lines 451–505) | IP-008 |
| Per-seat charge model | ✓ | ✓ | ✓ | ✓ | — | ✓ (per_seat billing component) | IP-004 |
| Per-usage / metered charge | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (per_usage billing component + MeteringApi) | IP-004 / IP-008 |
| Revenue share / platform fee | ✓ (Connect) | — | ✓ | — | — | ✓ (revenue_share billing component) | IP-004 / IP-011 |
| Tiered / volume pricing | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (rate-card-manager opaque ref) | IP-002 |
| Multi-currency invoicing | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (CurrencyCode + Money checked_add) | IP-001 |
| FX rate locking | ✓ | — | ✓ | — | ✓ | ✓ (FxLockApi proto3 lines 553–570) | IP-008 |
| Tax computation | ✓ Stripe Tax | ✓ (Avalara/TaxJar integration) | ✓ Zuora Tax | ✓ Chargebee Tax | — | ✓ (TaxInvoiceFormat + cloud-billing-tax µservice) | IP-003 |
| E-invoicing (KR, BR, IT, etc.) | partial | partial | ✓ | partial | — | ✓ (10 regional packs) | IP-003 |
| Invoice immutability | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (DuplicateInvoice on re-issue) | IP-001 |
| Idempotency keys | ✓ | ✓ | partial | ✓ | — | ✓ (Idempotency-Key + idempotency_key field) | IP-001 / IP-006 |
| Credit memo | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (IssueCreditMemo RPC) | IP-006 / IP-008 |
| Void / cancel invoice | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (VoidInvoice RPC) | IP-006 / IP-008 |
| Dunning policy | ✓ | ✓ | ✓ | ✓ | — | ✓ (DunningApi RPCs) | IP-008 |
| Free trial | ✓ | ✓ | ✓ | ✓ | ✓ Free Tier | ✓ (demo_trial tenant_class) | IP-005 |
| Cap-breach / quota | partial | — | — | — | ✓ Free Tier alerts | ✓ (cap.cloud.billing.deny_demo_trial_writes_after_cap_breach) | IP-005 |
| Subscription pause/resume | ✓ | ✓ | ✓ | ✓ | — | ✓ (Subscription.state: Paused) | IP-008 |
| Plan change / proration | ✓ | ✓ | ✓ | ✓ | — | ✓ (Subscription.proration_behavior) | IP-008 |
| Reservation / commitment | — | — | ✓ | — | ✓ (RIs, Savings Plans) | ✓ (ReservationApi + Commitment event kind) | IP-008 |
| Reservation recommender | — | — | partial | — | ✓ Cost Explorer | ✓ (cloud-billing-reservation-recommender) | IP-008 / PRD §2.2 |
| ASC 606 revenue recognition | partial | ✓ | ✓ | ✓ | — | ✓ (FOCUS / ERP export with recognition timing) | IP-011 / IP-015 |
| IFRS 15 revenue recognition | partial | ✓ | ✓ | ✓ | — | ✓ (same export pipeline) | IP-011 |
| SOX-404 evidence | partial | ✓ | ✓ | partial | ✓ | ✓ (audit-chain seal per mutation) | IP-010 |
| FOCUS 1.1 export | — | — | — | — | ✓ FOCUS | ✓ (TriggerFocusExport RPC) | IP-008 |
| ERP connector (SAP/NetSuite/Oracle) | partial | ✓ | ✓ | ✓ | partial | ✓ (TriggerErpExport RPC) | IP-008 |
| Cost-center attribution | — | — | partial | — | ✓ Billing Conductor | ✓ (cost-center hierarchy per tenant sub-scope) | IP-011 |
| Chargeback / showback | — | — | partial | — | ✓ | ✓ (finops-portal reads cloud-billing attribution) | IP-011 |
| Audit log | ✓ Sigma | ✓ | ✓ | ✓ | ✓ CloudTrail | ✓ (audit-chain with Ed25519 + Merkle) | IP-010 |
| Webhooks / events | ✓ | ✓ | ✓ Kafka | ✓ | ✓ EventBridge | ✓ (AsyncAPI CloudEvents + Protobuf) | IP-007 |
| API: REST | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (OpenAPI 3.2.0) | IP-006 |
| API: gRPC | — | — | — | — | — | ✓ (proto3, 11 services) | IP-008 |
| API: GraphQL | — | — | — | ✓ | — | ▼ (intentionally deferred per Rust-strict + complexity-budget) | — |
| API: SOAP | — | partial | ✓ | — | — | ▼ (not authored — modern proto3 substrates) | — |
| Multi-cell / multi-region | — | — | ✓ (DR pairs) | — | ✓ | ▲ (per-cell deployment with shuffle-sharding) | IP-014 |
| Data residency (EU/CN/KR) | partial | ✓ | ✓ | ✓ | ✓ | ▲ (cell-aware residency by deployment context) | IP-014 |
| Sovereign deployment (on-prem) | — | — | partial | — | — Outposts | ✓ (deployment_context enum + sovereign-invoice gate) | IP-003 / IP-008 |
| BEPS Pillar Two export | — | — | — | — | — | ▲ (cap.cloud.billing.settlement.beps_export) | IP-011 |
| Compliance: SOC2 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (compliance pack `soc2-type-ii`) | IP-009 |
| Compliance: PCI-DSS | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (compliance pack `pci-dss-v4`) | IP-009 |
| Compliance: HIPAA | partial | partial | partial | partial | ✓ | ✓ (compliance pack `hipaa-2024`) | IP-009 |
| Compliance: GDPR | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (compliance pack `gdpr` + DSR cascade) | IP-009 / IP-013 |
| Compliance: K-FSI | — | — | — | — | — | ▲ (compliance pack `k-fsi`) | IP-009 |
| Compliance: CSAP-KR | — | — | — | — | — | ▲ (compliance pack `csap-kr`) | IP-009 |
| Compliance: MAS-TRM | — | — | — | — | — | ▲ (compliance pack `mas-trm`) | IP-009 |
| Compliance: FedRAMP High | — | — | — | — | ✓ | ▲ (compliance pack `fedramp-high`) | IP-009 |
| Compliance: EU AI Act | — | — | — | — | — | ▲ (compliance pack `eu-ai-act`) | IP-009 |
| BYOK | — | — | — | — | ✓ (KMS) | ✓ (BYOK opt-in for paid; demo_trial denied) | IP-005 |
| Cedar-gated mutations | — | — | — | — | — | ▲ (every state-mutation behind named Cedar gate) | IP-009 |
| Two-person rule (void / payout) | partial | partial | partial | partial | partial | ✓ (context.has_reviewer_approval) | IP-009 |
| Atomic conversion transaction | — | — | partial | — | — | ▲ (conversion-gates.cedar atomicity) | IP-005 |
| Composable billing_components | partial (Connect-only) | — | partial | partial | — | ▲ (3-axis subset composition) | IP-004 |
| HTTP/3 + QUIC transport | — | — | — | — | — | ▲ (per ADR-0253) | IP-006 / IP-008 |
| Audit-chain crypto anchor | — | — | — | — | — | ▲ (Ed25519 + Merkle root) | IP-010 |
| Rust-strict implementation | — | — | — | — | — | ▲ (per Rust-strict-only directive) | IP-001 / IP-002 |

Legend: ✓ parity / ▲ exceeds counterpart / ▼ deferred (intentional).

### §C.2 Counterparts that exceed cloud-billing (today)

Honest assessment of where cloud-billing trails counterparts at the substantive feature level (these are gaps to close in future waves):

| Gap | Counterpart | Cloud-billing remediation IP | Priority |
|---|---|---|---|
| Hosted "Customer Portal" UI for self-serve invoice download / payment-method update | Stripe / Recurly / Chargebee | finops-portal owns; IP-015-extension to align UX feature list | P1 |
| Native A/B testing of pricing experiments | Chargebee | Not in scope; rate-card-manager µservice ticket | P3 |
| Built-in revenue forecasting / churn prediction | Recurly | finops-portal + ml-platform integration; future IP | P2 |
| Stripe Atlas-class company formation tools | Stripe Atlas | Out of scope; partnership integration | — |
| Per-invoice email customization templating | All four | finops-portal owns rendering; cloud-billing emits PDF handle | P2 |
| Account-balance dashboards | All four | finops-portal owns; cloud-billing exposes via gRPC | ✓ planned |

### §C.3 Counterparts oyatie intentionally diverges from

| Divergence | Counterpart pattern | Oyatie pattern | Reason |
|---|---|---|---|
| Customer-rooted vs tenant-rooted | Stripe: `Customer` is the root; subscriptions hang off | tenant_id is the root; BillingAccount + Subscription hang off tenant | ADR-0244 tenant scoping is universal; matches multi-product Oyatie shape |
| Decimal-string money | Stripe `amount_decimal: "10.50"` (string) | `Money { currency, minor_units: u64 }` | Avoid float-decimal drift; u64 minor_units is canonical |
| Pricing model variety as enum | Zuora `Charge.Model` enum | Pricing curve opaque inside rate_card_ref | Keep pricing model out of contract surface; rate-card-manager owns variety |
| GraphQL API | Chargebee provides GraphQL | REST + gRPC only | Per Rust-strict + complexity-budget directives |
| SOAP API | Zuora legacy SOAP | proto3 only | Modern substrate |
| Per-API-key scopes | Stripe permissions | Per-action Cedar gates | Finer-grained authz |
| Customer.tax_ids[] | Stripe customer-level tax IDs | BillingAccount.tax_registration_id (per-account) | Multi-jurisdiction tenants use multiple accounts; cleaner audit |

### §C.4 Capability coverage rollup

- Total tracked capabilities: ~50.
- Parity (✓): ~32.
- Exceeds (▲): ~14.
- Deferred (▼): ~2 (GraphQL, SOAP — intentional).
- Gaps to close (substantive): ~6 (customer portal UI, A/B pricing, churn prediction, etc.).

cloud-billing's substantive capability coverage is **at the industry-leader bar** per PRD §2.3 Outcome 11. Gaps are documented and prioritized.

## §D Lifecycle

This IP is a parity ledger, not a state machine. The lifecycle is the every-quarter review:

1. New Stripe / Recurly / Zuora / Chargebee feature lands.
2. axis-cloud-billing team evaluates: does it fit cloud-billing's shape?
3. If yes: add to IP-NNN for the corresponding capability + cross-link here.
4. If no (intentional divergence): document in §C.3.

## §E Cedar Policy Bindings

This IP itself does not author Cedar fragments; it references existing gates in IP-009.

## §F Evidence

### §F.1 Source files

- All sibling IPs in this sprint (IP-001 through IP-014).
- `microservices/cloud-billing/feature-parity-matrix-2026-05-20.md` (canonical parity ledger; this IP is the structured summary).
- `microservices/cloud-billing/PRD.md` (§2 Outcomes ties to counterpart parity).

### §F.2 ADR anchors

- ADR-0330 §B.11 composable billing_components (canonical replacement for tier model).
- ADR-0145 direct gRPC (transport divergence from REST-only counterparts).
- ADR-0244 tenant scoping (root divergence from customer-rooted counterparts).
- ADR-0263 audit-chain (cryptographic anchor exceeds counterparts).

## §G Counterpart parity (this IP IS the counterpart-parity ledger)

See §C above.

## §H Open questions

- Whether to add a per-quarter automated parity-diff job that scrapes Stripe/Recurly changelog and surfaces new features. Current decision: manual quarterly review; automation cost > benefit at current cadence.
- Whether to expose the parity matrix as a tenant-facing capability advertisement. Current decision: yes — `microservices/cloud-billing/feature-parity-matrix-2026-05-20.md` is published; finops-portal links to it for B2B procurement teams evaluating cloud-billing against incumbent tools.
