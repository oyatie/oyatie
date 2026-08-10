---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-cloud-billing
microservice: cloud-billing
status: Accepted
sales_segment: shared-substrate
tenant_class_scope: both
billing_components_scope: [revenue_share, per_seat, per_usage]
deployment_contexts:
  - oyatie-public-cloud
  - guest-on-aws
  - guest-on-oci
  - on-prem
  - colo
  - oyatie-as-cloud-provider
milestone_first_ship: M01-foundation
related_adrs:
  - ADR-0330
  - ADR-0329
  - ADR-0331
  - ADR-0328
  - ADR-0244
  - ADR-0243
  - ADR-0251
  - ADR-0255
  - ADR-0249
  - ADR-0131
  - ADR-0132
  - ADR-0145
  - ADR-0263
  - ADR-0130
  - ADR-0253
  - ADR-0252
  - ADR-0248
  - ADR-0218
  - ADR-0215
  - ADR-0216
  - ADR-0064
  - ADR-0039
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
related_specs:
  - /specs/tenant-model.json
  - /specs/billing/billing-component-schema.json
  - /specs/master-plan-sequencing.json
  - /specs/microservices/manifest-schema.json
local_adrs:
  - decisions/ADR-MS-001-billing-components-composability.md
  - decisions/ADR-MS-002-revenue-share-settlement-pipeline.md
date: 2026-05-21
owner_team: axis-cloud-billing + council-finance
doc_status: published
top_3_counterparts:
  - Stripe Billing
  - AWS Billing & Cost Management
  - Recurly
---

# PRD-cloud-billing: Composable Billing Substrate for the Two-Class Tenant Model

## 1. Purpose and Mission

The `cloud-billing` microservice is Oyatie's source-of-truth for **commercial state** — tenant class membership, billing-component composition, metering ledgers, multi-currency invoicing, rate-card lifecycle, reservation purchasing, credit memo issuance, FX lock provenance, revenue-share settlement, per-seat counting, per-usage aggregation, FOCUS 1.1 export, and ERP reconciliation. It is the keystone µservice for the binary `tenant_class ∈ {demo_trial, paid}` enum and for the composable `billing_components ⊆ {revenue_share, per_seat, per_usage}` set defined in ADR-0330. cloud-billing publishes the canonical tenant-class read API consumed by cloud-iam at principal-issuance time; it owns the demo_trial → paid conversion transaction; it owns cap-breach detection and grace-window enforcement; it owns the monthly settlement engine that drives payments and the audit-chain.

The mission of cloud-billing is to make commercial state engineering-grade. Every dollar that enters or leaves Oyatie's ledger does so through a deterministic, idempotent, audit-chain-anchored pipeline whose source data is principle-class invariant: tenant_id is required on every event, region matches the originating resource, idempotency_key is required on every emission, currency is a closed ISO 4217 + Oyatie internal credit code, totals are reconstructable from line items, period boundaries are validated, and tax invoice format is derived from the regional pack rather than carried out-of-band. The kernel crate `oya-cloud-billing-domain` already enforces these invariants at the type system layer (1,030 lines of strict Rust as of 2026-05-21). This PRD documents the kernel's commitments and extends them to address the billing_components composability that ADR-0330 made canonical.

cloud-billing is **shared substrate** in the ADR-0245 sense, not a hero product. It is consumed by `finops-portal` (tenant-facing FinOps surface, a Phase-1 product that presents cloud-billing's data to end users), by `payments` (charge attempts, mandate handling, settlement payout execution), by `cloud-billing-tax` (per-jurisdiction tax computation overlaying cloud-billing's tax-naive subtotals), by `audit-chain` (every billing event seals into the immutable ledger), by `cloud-iam` (reads tenant_class for principal claim emission), by `tenancy` (reads tenant_class for tenant lifecycle UX), by `cloud-storage` (FOCUS export delivery target), by `cloud-kms` (signs invoices + settlement statements), by `observability` (per-tenant cost metrics emission), and by every other Phase-0/Phase-1/Phase-2 µservice that contributes per_usage meter events. It is upstream of every product surface that touches money.

cloud-billing is a Phase-0 substrate per ADR-0328 §D-1 canonical build sequence. It must promote to staging before any Phase-1 product that touches money may promote past dev. Its kernel-strong / spec-light asymmetry as observed in the 2026-05-21 coherence audit is remediated by this PRD plus the companion contract surfaces, SLOs, Cedar policies, IaC modules, and OS manifest authored under the same sprint.

## 2. Primary Outcomes

### 2.1 Tenant outcomes

- **Outcome 1 — Trial fairness.** A demo_trial tenant gets the full product surface, full quality bar, full performance budget, and full architectural posture. The only differences vs paid are usage caps (enforced as soft alerts at 80% and Cedar-deny writes after 100%+grace), absence of contractual SLO commitment, and gates on compliance pack activation / BYOK / marketplace listing. There is no feature lockout; there is no degraded model; there is no second-class UX surface. This rule is CI-enforced via `oya-governance-paid-quality-bar-parity`.
- **Outcome 2 — Paid clarity.** A paid tenant gets a single monthly statement that aggregates per_seat line items, per_usage line items, revenue_share settlement statements (or payout-direction settlement statements when oyatie owes the tenant), and clawback nettings — all denominated in the tenant's contracted settlement currency with explicit FX provenance. The statement reconstructs from line items; the line items reconstruct from per-event sources; the per-event sources are sealed in the audit-chain. SOX-404, ASC 606, IFRS 15, K-FSI, and FedRAMP retention regimes are satisfied by the same provenance chain without per-regime re-authoring.
- **Outcome 3 — Multi-currency honesty.** Multi-currency activity is recorded at the transaction-time FX rate from a documented FX feed (ECB-reference-rates-daily for fiat; per-vendor mid-rate for stable-coin) and settled in the tenant's contracted settlement currency with an explicit settlement-FX-adjustment line item. There is no hidden FX margin; there is no surprise re-FX at settlement; there is no ambiguity about which day's rate applied. The kernel's `Money::checked_add` invariant forbids cross-currency arithmetic without provenance.
- **Outcome 4 — Composability without surprise.** A paid tenant whose contract carries `{per_seat, per_usage}` may add `revenue_share` at any point by contract amendment. The amendment is a single atomic transaction (cloud-billing applies the change, emits the audit-chain event, refreshes principal tokens within 30 seconds). No re-onboarding, no data migration, no functional surface change.
- **Outcome 5 — Conversion that doesn't lose data.** When a demo_trial tenant converts to paid, the conversion is atomic. Usage history is retained but not retroactively billed (the contract default is "trial usage is free"; the contract may opt into retroactive billing); audit-chain entries persist; tenant cap state clears; principal cap_breached claim flips to false; tokens refresh within 30 seconds.

### 2.2 Operator outcomes

- **Outcome 6 — Invoice timing.** Monthly close emits invoices within the contracted SLO window. The unified industry-leader bar is "p99 ≤ 4 hours from period close to invoice issuance" matching Stripe Billing's posted invoicing SLO. Tenant-class does not stratify this number; deployment-context overlays may add a small constant for on-prem and colo where ERP-export round-trip applies.
- **Outcome 7 — Reservation flight-deck.** Reservation recommender ingests the prior 60 days of usage, computes break-even per workload kind (vCPU, memory, function invocation, K8s pod-minute), produces recommendations with explicit savings projections, and offers a Cedar-gated `ConvertReservation` action that the tenant finance lead authorizes. Auto-purchase is opt-in per tenant; default is recommend-only.
- **Outcome 8 — Chargeback attribution.** Every dollar of vendor pass-through cost (when applicable for hosted tenants) and every dollar of Oyatie-provisioned substrate cost is attributable to a cost-center via attribution rules. Mismatches between vendor tags and tenant cost-centers surface in finops-portal with explicit reconciliation actions.
- **Outcome 9 — Sovereign deployability.** Sovereign deployments (KR K-FSI, MAS-TRM, CSAP, EU-AI-Act, FedRAMP High) operate cloud-billing without changes to the kernel. Sovereign behavior is encoded in regional packs (`oya-pack-*-tax`) that map to TaxInvoiceFormat variants. Air-gapped operation uses a local metering bus with one-way replication to the sovereign control plane.

### 2.3 Substrate outcomes

- **Outcome 10 — Phase-0 readiness.** cloud-billing is buildable from cold by an intern engineer using the PRD + ARCHITECTURE + contract surfaces + SLOs + Cedar policies + IaC modules + OS manifest authored under this sprint. Substance bar is enforced.
- **Outcome 11 — Industry-leader parity.** Capability surface covers the UNION of Stripe Billing + AWS Billing & Cost Management + Recurly per the feature-parity-matrix-2026-05-20.md. Subscription primitive, dunning policy, ASC 606 revenue recognition, demo_trial cap-breach alerts (AWS Free Tier Alerts equivalent), Billing Conductor-class linked-account chargeback, and FOCUS 1.1 export are all in scope.
- **Outcome 12 — Cedar-gated state mutations.** Every state-mutation surface (issue invoice, convert reservation, settle rev-share, mutate billing_components, change tenant_class) is gated by a named Cedar action and a named Cedar policy. There is no inline `if` guard; there is no per-microservice authorization shortcut.

## 3. Personas

### 3.1 Primary personas

- **P1 — Tenant-side CFO / Finance Lead.** Cares about: closing the books on time, accurate AR aging, ASC 606 revenue recognition, multi-jurisdiction tax filings, settlement audit trail, FX exposure. Reads finops-portal (cloud-billing's downstream consumer). Authorizes contract amendments (billing_components mutations, per-seat ceiling increases, rev-share contract terms).
- **P2 — Tenant-side AR / Billing Operator.** Cares about: invoice contents, payment-method management, dispute handling, chargeback investigation, credit memo issuance. Reads invoices via finops-portal; reads settlement statements; files disputes.
- **P3 — Tenant-side RevOps Analyst.** Cares about: usage projections, soft-cap configuration, hard-cap configuration, per-meter pricing trends, revenue recognition forecasts. Reads finops-portal cost forecasts; tunes soft-caps.
- **P4 — Tenant-side FinOps Practitioner.** Cares about: cost-center attribution accuracy, reservation purchase recommendations, anomaly detection, tag hygiene, FOCUS export consumption. Cross-references cloud-billing FOCUS export with internal cost-allocation models.
- **P5 — Oyatie-side Site Reliability Engineer (SRE).** Cares about: metering bus uptime, period-close timing, FX feed liveness, audit-chain sealing latency, ERP-export integrity. Owns the cloud-billing runbooks (invoice-generation-timeout, per-tenant-cost-attribution-mismatch, reservation-recommendation-engine-stall) and on-call rotation.
- **P6 — Oyatie-side Compliance Officer.** Cares about: SOX-404 segregation-of-duties evidence, K-FSI audit packet, MAS-TRM logbook, PCI DSS scope minimization, GDPR data-flow accuracy, EU-AI-Act billing-decisions-impact log. Reads cloud-billing's compliance-pack overlays.
- **P7 — Oyatie-side Engineer (any microservice that emits usage events).** Cares about: meter shape declaration, idempotency-key generation, event-schema versioning, audit-chain seal semantics, deployment-context portability. Reads cloud-billing's contracts (AsyncAPI for event subscription; OpenAPI for invoice query; proto3 for inter-µservice gRPC).
- **P8 — Oyatie-side Marketplace Seller (revenue_share tenant).** Cares about: settlement timing, payout method configuration, clawback handling, FX exposure on multi-currency sales, audit-chain provenance for tax-purposes. Reads settlement statements via finops-portal.

### 3.2 Secondary personas

- **P9 — Oyatie-side Customer Success.** Cares about: tenant trial expiry, cap-breach warning, conversion CTA effectiveness, upgrade funnel telemetry. Reads cloud-billing's tenant-state surface via crm.
- **P10 — Oyatie-side Sales.** Cares about: contract-term presentation in invoices, custom pricing approval flow, commitment forecasts, deal-size projections. Reads cloud-billing's commitment ledger.
- **P11 — External Auditor (SOC2 Type II, ISO 27001, K-FSI, MAS-TRM, FedRAMP).** Cares about: replayable audit trail, sealed evidence emissions, segregation-of-duties controls, retention policy enforcement, deletion provenance. Reads cloud-billing's audit-chain entries via the audit-chain microservice.
- **P12 — Tenant-side ERP Administrator.** Cares about: invoice ingestion to SAP / NetSuite / Oracle EBS, settlement reconciliation, journal entry mapping, intercompany handling. Consumes cloud-billing's ERP-export adapter output.

## 4. Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority | tenant_class |
|---|---|---|---|---|---|---|
| FR-01 | Phase-0 µservice (any) | to emit a CloudBillingEvent with tenant_id + resource_id + region + units + rate_card_ref + idempotency_key | usage is metered into the canonical ledger | metering-bus | Must | both |
| FR-02 | cloud-iam | to read tenant_class + billing_components atomically from cloud-billing | principal claims carry up-to-date commercial state | tenant-class-api | Must | both |
| FR-03 | finops-portal | to query per-tenant usage rollups per (hour, day, week, month) | tenants see cost in near-real-time | aggregation-worker | Must | both |
| FR-04 | tenant-admin | to mutate billing_components on a paid tenant via contract amendment | composability rule (ADR-0330 §B.2.5) honored | contract-amendment | Must | paid |
| FR-05 | invoice-worker | to generate a monthly invoice at period close, summing line items per billing_component | tenant receives a single statement | invoice-worker | Must | paid |
| FR-06 | settlement-engine | to compute monthly revenue-share settlement, apply contract commission, net clawbacks, emit settlement-statement event to payments | marketplace settlement closes on time | settlement-engine | Must | paid (rev_share) |
| FR-07 | seat-counter | to read active seat count from cloud-iam at monthly close, compute per_seat invoice line, fail-closed on ceiling breach | per-seat enforcement honored | seat-counter | Must | paid (per_seat) |
| FR-08 | meter-aggregator | to aggregate per_usage meter events with idempotency dedup (7-day window), compute invoice line items per (meter_unit, pricing_dimension) | per-usage billing honored | meter-aggregator | Must | paid (per_usage) |
| FR-09 | cap-breach-monitor | to poll demo_trial tenant usage every 5 minutes, emit cap-breach event at 100% threshold, trigger grace state | trial fairness + conversion CTA timing | cap-breach-monitor | Must | demo_trial |
| FR-10 | grace-window | to enforce 7-day grace after cap-breach with Cedar-deny on writes, retain read access, support early conversion | trial UX clean exit | grace-window | Must | demo_trial |
| FR-11 | conversion-engine | to atomically convert demo_trial → paid: validate contract, settle pre-conversion usage if billable, write audit-chain event, refresh tokens | conversion is single transaction | conversion-engine | Must | demo_trial → paid |
| FR-12 | reservation-recommender | to analyze prior 60 days of usage, produce recommendations with explicit savings projections, Cedar-gate auto-purchase | reservations buy savings without surprise | reservation-recommender | Must | paid |
| FR-13 | fx-lock-service | to capture FX rate at transaction time from ECB-daily (fiat) or vendor mid-rate (stable-coin), record provenance hash, refuse cross-currency arithmetic without lock | FX honesty | fx-lock-service | Must | paid (multi-currency) |
| FR-14 | tax-handoff | to produce a tax-naive subtotal per line item, send to cloud-billing-tax for per-jurisdiction overlay, accept tax-line response | tax computation delegated correctly | tax-handoff | Must | both |
| FR-15 | invoice-format-derivation | to derive TaxInvoiceFormat from regional_pack (per the kernel's closed enum mapping) | format is not carried out-of-band | invoice-format | Must | both |
| FR-16 | credit-memo-issuer | to issue Cedar-gated credit memos with explicit reason + audit-chain seal | error correction is auditable | credit-memo-issuer | Must | paid |
| FR-17 | focus-export-adapter | to export FOCUS 1.1 columnar data per tenant per month to cloud-storage | tenant FinOps tooling interop | focus-export-adapter | Must | paid |
| FR-18 | erp-export-adapter | to export invoice + journal entries to SAP / NetSuite / Oracle EBS via tenant-configured connector | tenant ERP ingestion | erp-export-adapter | Should | paid |
| FR-19 | dunning-policy | to drive failed-payment retry per contract terms (default: 3 retries at +1d/+3d/+7d), then enter delinquent state, then suspension | AR aging honored | dunning-policy | Must | paid |
| FR-20 | subscription-primitive | to model a Subscription resource bound to a paid tenant + billing_components set + lifecycle state machine (created, plan_changed, paused, resumed, canceled) | Stripe + Recurly parity | subscription | Must | paid |
| FR-21 | proration-engine | to compute mid-period upgrade/downgrade proration per billing_component, emit proration line items | upgrade/downgrade fairness | proration-engine | Must | paid |
| FR-22 | usage-projection | to compute next-month projected invoice based on rolling 30-day usage, surface via finops-portal | tenant cost foresight | usage-projection | Should | paid |
| FR-23 | audit-chain-emission | to seal every billing event into audit-chain per ADR-0263 emission contract | provenance preserved | audit-chain-emission | Must | both |
| FR-24 | beps-pillar-two-export | to export OECD BEPS Pillar Two GloBE data for tenants with >€750M revenue threshold | multinational tax compliance | beps-export | May | paid (eligible) |
| FR-25 | clawback-handler | to record revenue_event reversals tied to original idempotency_key, net in next settlement | clawback fairness | clawback-handler | Must | paid (rev_share) |
| FR-26 | byok-credential-binder | to honor tenant BYOK provider credentials for downstream payment / KMS / LLM provider, refuse demo_trial BYOK | BYOK gated by tenant_class | byok-binder | Must | paid |
| FR-27 | sovereign-replicator | to replicate metering bus from air-gapped sovereign deployment to oyatie control plane one-way | sovereign deployments operate | sovereign-replicator | Must | paid (sovereign) |
| FR-28 | tenant-class-mutated-publisher | to emit tenant-class-mutated event on gRPC substrate within 30s of change | downstream caches refresh | tenant-class-publisher | Must | both |

## 5. Non-Functional Requirements

### 5.1 Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Metering event ingest (single event, end-to-end ledger + meter record) | 8 ms | 35 ms | 90 ms | matches Stripe Sigma ingest profile |
| Sustained metering ingest throughput (per cell) | 5,000,000 events/sec | — | — | with 18,000,000 events/sec burst capacity |
| Per-tenant idempotency dedup lookup (7-day window) | 1 ms | 5 ms | 12 ms | BTreeMap O(log n) + TTL sweep |
| Invoice generation (single tenant, single billing_component) | 50 ms | 200 ms | 500 ms | bounded by line-item count |
| Invoice generation (single tenant, all three billing_components) | 120 ms | 450 ms | 900 ms | sum of components plus FX adjustment |
| Monthly close (per cell, full tenant population) | 30 min | 90 min | 4 hr | matches Stripe Billing's posted invoicing SLO |
| Period-close to invoice issuance (single tenant) | 30 min | 4 hr | 12 hr | unified bar; deployment-context overlay adds +1h for on-prem/colo |
| Settlement engine monthly close (revenue_share, per cell) | 60 min | 3 hr | 6 hr | bounded by FX lock provenance fetch |
| FX lock fetch (single transaction) | 5 ms | 30 ms | 80 ms | cached ECB-daily; vendor mid-rate latency varies |
| tenant_class read API (cloud-iam principal-issue path) | 2 ms | 10 ms | 25 ms | hot read; Postgres-backed; replicated cache |
| Cedar evaluation (tenant_class + billing_components attribute matching) | 1 ms | 4 ms | 10 ms | per ADR-0243 gate latency budget |
| FOCUS 1.1 export (per tenant per month) | 5 sec | 30 sec | 90 sec | bounded by row count; parquet codec |
| ERP export (per tenant per month, SAP IDoc) | 10 sec | 60 sec | 5 min | bounded by connector round-trip |
| Reservation recommender (per tenant, 60-day analysis) | 200 ms | 1 sec | 3 sec | bounded by query cost |
| Audit-chain seal (per event) | 5 ms | 30 ms | 80 ms | per ADR-0263 |
| Demo_trial cap-breach poll cadence | 5 min | — | — | global polling cadence |
| Conversion (demo_trial → paid) transaction | 100 ms | 500 ms | 2 sec | atomic transaction + token refresh |

### 5.2 Scalability

cloud-billing scales horizontally per cell (ADR-0248 cellular architecture). Each cell handles up to 50,000 tenants and 5M events/sec sustained. Cross-cell traffic is forbidden for demo_trial tenants per the cross-cloud-forbidden-pattern in ADR-0328 §D-15.44; paid tenants may operate sub-tenancies across cells with explicit Cedar grant.

Sharding key for the metering ledger is (tenant_id, region). Sharding key for the invoice ledger is (tenant_id, period_start). Sharding key for the settlement ledger is (tenant_id, settlement_period_start, billing_component).

The kernel's `events_by_idempotency` BTreeMap is process-local and bounded by a 7-day TTL sweep (introduced under remediation per the coherence audit). Cross-process idempotency is enforced by the Postgres-backed ledger via a unique constraint on (tenant_id, idempotency_key).

### 5.3 Security

- Every billing event emission is signed by the emitting µservice's per-environment Ed25519 key per ADR-0263 audit-emission contract.
- Every invoice is signed by cloud-kms before issuance; the invoice carries the signature in a non-spoofable header.
- Tax registration IDs are FINANCIAL data class per the kernel's `TaxRegistrationId` type; access requires Cedar-permitted principal-attribute match.
- Payment method references are INTERNAL_ONLY data class per the kernel's `PaymentMethodRef` type; raw payment-instrument data never enters cloud-billing.
- BYOK provider credentials are stored in OpenBao per the secrets-management directive; cloud-billing reads via SecretReference only.
- Settlement statement PDFs are content-addressed (SHA256) and stored in cloud-storage with object-lock retention per the contract's settlement retention class (default SOX 7 years).
- Cedar default-deny gates every mutation surface (per ADR-0243).
- Air-gapped sovereign deployments use the one-way replicator with public-key-pinned destination to prevent cross-context exfiltration.

### 5.4 Audit + Compliance

cloud-billing emits the following audit-chain events per ADR-0263:

- `cloud_billing.usage.recorded` — per-event ingest seal.
- `cloud_billing.invoice.issued` — per-invoice issuance seal.
- `cloud_billing.invoice.voided` — invoice void / credit memo issuance seal.
- `cloud_billing.credit_memo.issued` — per-credit-memo seal.
- `cloud_billing.reservation.purchased` — per-reservation purchase seal.
- `cloud_billing.reservation.converted` — per-reservation conversion seal.
- `cloud_billing.settlement.computed` — monthly settlement statement seal.
- `cloud_billing.settlement.payout_initiated` — payout direction seal.
- `cloud_billing.settlement.invoice_issued` — collect direction seal.
- `cloud_billing.settlement.clawback_netted` — clawback netting seal.
- `cloud_billing.tenant_class.transitioned` — demo_trial → paid conversion seal.
- `cloud_billing.billing_components.mutated` — composability mutation seal.
- `cloud_billing.cap_breach.warning_emitted` — 80% threshold seal.
- `cloud_billing.cap_breach.emitted` — 100% threshold seal.
- `cloud_billing.grace_window.expired` — grace expiry seal.
- `cloud_billing.fx_lock.captured` — FX lock provenance seal.
- `cloud_billing.focus_export.completed` — FOCUS 1.1 export seal.
- `cloud_billing.erp_export.completed` — ERP export seal.
- `cloud_billing.subscription.lifecycle_event` — Subscription lifecycle seal (created, plan_changed, paused, resumed, canceled).

Event class naming convention is lowercase dotted snake-case per ADR-0263. The retired uppercase EVT_* convention from the runbooks is replaced.

Compliance pack overlays (activated when `tenant_class == paid` per ADR-0330 §B.3.6):

- **SOC 2 Type II** — control evidence: segregation-of-duties (invoice issuance vs payment authorization separation), change-management traceability (every cloud-billing kernel change requires ADR + reviewer-agent verdict).
- **SOC 1** — control evidence: per-tenant invoice accuracy (kernel reconstructs total from line items; CI-enforced).
- **ISO 27001** — control evidence: access logging (Cedar evaluation log per gate hit).
- **GDPR** — Art 6(1)(b) contract + Art 6(1)(c) legal obligation for invoice retention; DPIA at `dpia.md`; data subject right-to-erasure handled via cloud-billing's deletion-provenance flow.
- **PCI DSS v4.0** — scope-minimized: cloud-billing never stores PAN, CVV, or expiry; only `PaymentMethodRef` (an opaque token issued by payments).
- **EU AI Act** — Annex III §5 (employment + credit + access decisions): cloud-billing's billing decisions are not Annex III; cap-breach + conversion CTA are not credit decisions in the legal sense.
- **CSAP-KR** — Korean cloud security accreditation: regional pack `oya-pack-electronic-tax` activates; sovereign replicator at sovereign-pack overlay.
- **K-FSI** — Korean financial supervisory: regional pack `oya-pack-qualified-tax`; SOX-equivalent retention; sovereign deployment context.
- **MAS-TRM** — Singapore monetary authority technology risk management: deployment context guest-on-aws (Singapore region) or on-prem; per-tenant cell anchoring.
- **SOX-404** — segregation of duties + retention of 7 years: enforced by cloud-billing's audit-chain seal + cloud-storage object-lock.
- **FedRAMP High** — 3-year retention minimum; deployment context oyatie-as-cloud-provider (sovereign US tenants).

### 5.5 Availability + SLO

| SLO | Target | Window | Notes |
|---|---|---|---|
| Metering bus availability | 99.99% | 30d rolling | per cell |
| Invoice generation availability | 99.95% | 30d rolling | per cell |
| tenant_class read API availability | 99.99% | 30d rolling | hot read; cloud-iam dependency |
| Settlement engine availability | 99.9% | monthly | per cell |
| FX lock service availability | 99.95% | 30d rolling | ECB-daily dependency |
| Monthly close completion | 99.9% | quarterly | per cell |
| RTO | ≤ 15 min | per incident | invoice generation worker re-launch |
| RPO | ≤ 60 sec | per incident | metering bus exactly-once with idempotency dedup |
| Cell isolation | hard | per ADR-0248 | no cross-cell write paths |

### 5.6 Data residency

Per-tenant data residency is enforced by the tenant's deployment_context choice (ADR-0218). Sovereign packs override with stricter posture:
- `oya-pack-electronic-tax` (KR CSAP / K-FSI) — KR region; air-gapped option.
- `oya-pack-qualified-tax` (KR K-FSI) — KR region; SOX-equivalent retention.
- `oya-pack-country-tax` (EU member state) — EU region; GDPR transfer rules.
- `oya-pack-gst-tax` (IN GST) — IN region.
- `oya-pack-vat-tax` (UAE 5% VAT) — UAE region.

FX feeds (ECB-daily) are fetched from the primary deployment context's network egress; air-gapped deployments use a pre-mirrored daily snapshot via the sovereign replicator.

### 5.7 DR posture (ADR-0343)

- Declared target: RTO <= 900 seconds and RPO <= 60 seconds, matching the existing Availability + SLO table. `manifest.json` is not present for this µservice, so D-2 manifest backfill must copy these values rather than invent a looser DR block.
- Applicable floors: HIPAA-2024 (3600/300, multi-region), PCI-DSS-L1-v4 (86400/3600), SOC2-T2 (14400/900), ISO27001-2022 (14400/3600), SOX-404 (14400/3600), KR-CSAP-v3.1 (3600/900, multi-region), and KR-PIPA-2023-amendment (14400/900). The effective strictness is the smallest numeric target across PRD and packs: RTO 900 seconds, RPO 60 seconds; multi-region is required when HIPAA, KR-CSAP, sovereign paid, or contractually active-active tenant cells are enabled.
- Failover runbook reference: pending `runbooks/billing-cell-failover.md`; the current runbook set covers invoice timeout, cost-attribution mismatch, and reservation recommender stalls but not full ledger failover.
- multi_region_active_active posture: yes for paid Tier-0/Tier-1 sovereign or multi-cell billing ledgers; demo_trial writes remain home-cell pinned and fail closed when their home cell is unavailable.
- WHY: tenants must still see invoice state, cap state, tenant_class, settlement posture, and usage-ledger replayability during a cell outage; the tighter RPO prevents chargeable usage from existing outside the audit-chain evidence window.

### 5.8 Capacity model (ADR-0340)

- Per-tenant baseline: D-2 has not produced a `capacity_model` block, so CPU/RAM/storage/connection baselines are intentionally not fabricated here. The current PRD capacity envelope remains per cell: 50,000 tenants, 5,000,000 sustained metering events/sec, and 18,000,000 burst events/sec.
- Scaling dimension: `per_request` for invoice/API reads and `per_usage_event` for metering, settlement, FOCUS export, and cap-breach processing.
- Cell placement class: Tier-1 for ledger, invoice, settlement, and tenant_class mutation paths because commercial state gates paid access and revenue evidence; demo_trial cap-poll and read-only projections may run in lower-cost trial cells only when they preserve the same quality bar.
- Autoscaling boundaries: until manifest backfill declares min/max per tenant, the PRD-boundary is three active workers per paid cell for metering/invoice/seal paths, five active workers for Tier-0 sovereign cells, and throughput-based horizontal scale to the event-rate targets above.
- WHY: the service load profile is bursty at period close and steady for metering ingest; capacity is sized around commercial correctness under spikes rather than average dashboard traffic.

### 5.9 Sustainability + cost attribution (ADR-0344)

- Every audit-chain row emitted by cloud-billing also carries `cost_usd_minor_units`, `co2_grams`, and `watt_hours`, aligned to the finops axes tenant, product, capability, provider, cell, and compliance_pack.
- Provider routing is carbon-aware for non-urgent FOCUS export, reservation recommendation, ERP export, and settlement reporting jobs. It is not carbon-routed when invoice issuance, conversion, PCI-scoped payment coordination, or legal period-close deadlines would miss their SLO.
- Tenant transparency surface: finops-portal cost explorer, monthly invoice line items, FOCUS 1.1 export, and settlement statements.
- WHY: CSRD, SB-253, SEC climate-disclosure, SOX-404, and customer FinOps expectations require the same billing event to explain price, emissions, energy, and jurisdictional attribution.

### 5.10 API versioning posture (ADR-0342)

- Public API version model: cloud-billing's invoice REST API, tenant_class read API, ERP/FOCUS export APIs, and tenant-facing SDKs use the YYYY-MM-DD carrier triplet: `Oyatie-Version` header, `/v/<YYYY-MM-DD>/...` URL prefix, and proto3 version field. Existing `/v1` endpoints remain migration aliases only.
- SDK semver model: SDK packages use major.minor.patch, with major bumps reserved for incompatible date-version carrier behavior or generated type breaks.
- Support window: last N=3 public API versions are supported for at least 180 days, with deprecation calendars sealed into audit-chain before removal.
- Per-tenant pinning: supported for paid tenants and sovereign packs; demo_trial tenants follow the default current-minus-one window unless conversion requires a temporary pin.
- Internal-mesh exemption: yes; direct gRPC inside the mesh keeps ADR-0145 semantics while still carrying the date-version field for boundary logging and replay.

## 6. tenant_class semantics

### 6.1 tenant_class enum

cloud-billing is the canonical source-of-truth for `tenant_class` per ADR-0330 §B.10.1. The enum is closed:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TenantClass {
    DemoTrial,
    Paid,
}
```

The enum is exhaustive at the kernel source (no `#[non_exhaustive]`). Downstream consumer crates use `#[non_exhaustive]` re-exports to prevent unknown-variant panics when the upstream surface evolves.

### 6.2 tenant_class state transitions

The legal transitions are:
- `DemoTrial → Paid` (one-way; via conversion-engine atomic transaction)
- `Paid → Paid` (no-op; with billing_components mutation as a separate atomic transaction)

The forbidden transitions are:
- `Paid → DemoTrial` (downgrade refused; ADR-0330 §B.1.3 rationale)
- `DemoTrial → DemoTrial` (no-op; class-flip refused without contract)

Every transition emits a `cloud_billing.tenant_class.transitioned` event into the audit-chain with old class, new class, contract identifier (when applicable), and initial billing_components.

### 6.3 demo_trial caps

demo_trial tenants are subject to two cap families:

**Time cap.** Default 30 days from tenant creation. Configurable per global policy (60 days for higher-touch trials) or per-µservice override. The time cap counts wall-clock days, not active-use days. At T-7d, T-3d, and T-0, cloud-billing emits a `cap_breach_warning_emitted` event consumed by notifications.

**Usage caps.** Per-µservice limits on resource units:
- `oya-agentic-agent`: max 5 active agents
- `oya-workflow-engine`: max 100 workflow executions per day
- `cloud-iam`: max 10 seats
- `oya-messaging-mls`: max 3 MLS groups
- `cloud-data-store`: max 5 GB stored
- `cloud-compute-functions`: max 10,000 invocations per month
- `cloud-compute-k8s`: max 100 pod-minutes per day
- `cloud-compute-vm`: max 1 vCPU + 2 GB RAM
- `cloud-storage`: max 5 GB storage
- `oya-search-index`: max 1,000 vector queries per day
- `oya-intelligence-inference`: max 100,000 input tokens + 25,000 output tokens per day (per default model class)

Each cap is configurable per tenant via the tenant-class trial policy. Hard caps fail-closed via Cedar deny on write paths; read paths remain open during grace.

### 6.4 demo_trial gates

demo_trial tenants are denied (via Cedar) the following actions:

- `activate_compliance_pack` (ADR-0251 + ADR-0330 §B.3.6)
- `configure_byok_provider` (ADR-0255 §D-4 + ADR-0330 §B.3.7)
- `publish_marketplace_listing` (ADR-0249 + ADR-0330 §B.3.8)
- `purchase_marketplace_paid_listing` (limited to free listings during trial)
- `change_tenant_class to demo_trial` (when current state is paid; ADR-0330 §B.1.3)

### 6.5 paid tenant semantics

paid tenants choose a `deployment_context` from the canonical 6:
- `oyatie-public-cloud`
- `guest-on-aws`
- `guest-on-oci`
- `on-prem`
- `colo`
- `oyatie-as-cloud-provider`

paid tenants may operate sub-tenancies. The parent tenant aggregates sub-tenancy billing into a single invoice unless the contract specifies separate invoicing.

paid tenants receive contractual SLO posture matching the unified industry-leader bar.

## 7. billing_components composability

### 7.1 The composability primitive

For paid tenants, `billing_components` is a subset of `{revenue_share, per_seat, per_usage}`. The 8 valid combinations are documented in ADR-0330 §B.2.4. cloud-billing's contract record carries the chosen subset; mutations are contract-amendment-gated.

The kernel introduces (under this PRD's authority) the following types:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum BillingComponent {
    RevenueShare,
    PerSeat,
    PerUsage,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct BillingComponentSet(BTreeSet<BillingComponent>);

impl BillingComponentSet {
    pub fn contains_revenue_share(&self) -> bool { self.0.contains(&BillingComponent::RevenueShare) }
    pub fn contains_per_seat(&self) -> bool { self.0.contains(&BillingComponent::PerSeat) }
    pub fn contains_per_usage(&self) -> bool { self.0.contains(&BillingComponent::PerUsage) }
}
```

The `BillingAccount` type gains `tenant_class: Classified<TenantClass>` and `billing_components: Classified<BillingComponentSet>` fields. The fields are `INTERNAL_ONLY` data class.

### 7.2 revenue_share component

cloud-billing emits `cloud_billing.usage.recorded` events with `kind = CloudBillingEventKind::RevenueShare` for revenue-bearing transactions. The new kind variant is added under this PRD's authority:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CloudBillingEventKind {
    ResourceCreated,
    ResourceTerminated,
    Usage,
    Reservation,
    Commitment,
    Credit,
    RevenueShare,     // new
    RevenueShareReversal, // new for clawback
    SeatCount,        // new for per_seat snapshot
    Subscription,     // new for Subscription lifecycle
}
```

Revenue-share contract terms:
- Commission rate: per category (plugins / apps / workflows / agents / models / datasets). Defaults are documented in the per-marketplace-category ADRs (pending Wave 15K authoring).
- Settlement cadence: monthly close (default last calendar day of the month, UTC).
- Settlement currency: per contract; FX from transaction-time rate.
- Clawback handling: revenue_event_reversal entries tied to original idempotency_key; netted in next settlement.
- Direction: `oyatie_collects` (tenant owes oyatie) or `oyatie_pays` (oyatie owes tenant).
- Negative revenue_share (affiliate / referral): direction = oyatie_pays; settlement pipeline is symmetric.

The settlement engine at monthly close:
1. Gathers all `RevenueShare` events for (tenant, settlement_window).
2. Applies contract commission per category.
3. Computes oyatie's share + tenant's share + withheld taxes (via cloud-billing-tax) + FX adjustments + clawback nettings.
4. Generates the settlement statement (PDF via document-generation; JSON via API).
5. Emits `settlement_statement_event` to payments with direction + amount + destination payment method.
6. payments executes the money movement; emits `payout_completed_event` or `invoice_issued_event` back.
7. cloud-billing closes the settlement window; emits `settlement_closed_event` to audit-chain.

### 7.3 per_seat component

cloud-billing reads active seat counts from cloud-iam at monthly close. A seat is one named human user or one named non-human principal (service account, headless bot, scheduled job) per contract.

Seat counting rules:
- Deactivated users drop from the seat count after a configurable grace window (default 7 days).
- Over-seat principals fail-closed via Cedar deny after the contract's grace window.
- Multi-tenant users (one human associated with multiple tenants) consume one seat per tenant; no cross-tenant pooling.

Seat pricing:
- Set per contract; varies by negotiated terms.
- Monthly cadence default; annual prepay produces a single invoice at anniversary + monthly true-up for mid-cycle adds.
- True-up rule: seats added mid-cycle are prorated to the remaining days in the cycle.

Seat add/remove operations are authority-tier 2 (tenant-admin authorization required) per ADR-0330 §B.6.7.

### 7.4 per_usage component

cloud-billing aggregates meter events from every contributing µservice. Each µservice declares its meter shape in its own PRD:

| Microservice | Meter unit | Cadence | Pricing dimension |
|---|---|---|---|
| oya-intelligence-inference | `llm_input_tokens` | continuous | model, region |
| oya-intelligence-inference | `llm_output_tokens` | continuous | model, region |
| oya-intelligence-inference | `gpu_seconds` | continuous | model class, region |
| oya-workflow-engine | `workflow_executions` | continuous | template_class, region |
| cloud-data-store | `gb_stored` | hourly snapshot | region, storage_class |
| cloud-data-store | `gb_egress` | continuous | region, dest_class |
| cloud-api-gateway | `api_calls` | continuous | region, tier (req/sec band) |
| cloud-compute-vm | `vcpu_hour` | continuous | instance class, region |
| cloud-compute-vm | `memory_gb_hour` | continuous | instance class, region |
| cloud-compute-k8s | `pod_minute` | continuous | pod class, region |
| cloud-compute-functions | `invocation_count` | continuous | region |
| cloud-compute-functions | `gb_seconds` | continuous | region |
| oya-search-index | `vector_search_queries` | continuous | index class, region |
| cloud-storage | `gb_stored` | hourly snapshot | region, storage_class |
| cloud-storage | `requests` | continuous | region, operation_class |

Aggregation cadence: continuous metering with hourly / daily / weekly visibility in finops-portal. Monthly close groups by (meter_unit, pricing_dimension) into invoice line items.

Soft-cap configuration: tenant may opt-in via cloud-billing API; at threshold, alert is emitted; tenant may set hard-cap for Cedar-deny on excess.

Meter idempotency: required on every event; cloud-billing dedups on (tenant_id, meter_unit, idempotency_key) within a 7-day window.

Correction handling: `correction_for` field references original event's idempotency_key; correction applied in next monthly close.

### 7.5 Composability examples

The 8 valid combinations (per ADR-0330 §B.2.4 + §B.8):

- `{}` paid no-component-yet — transient; non-blocking advisory after 7 days.
- `{revenue_share}` — pure marketplace seller / B2C operator. Pure monthly settlement statement.
- `{per_seat}` — pure B2B enterprise named-user. Pure monthly per-seat invoice.
- `{per_usage}` — pure metered consumption. Pure monthly per-meter invoice.
- `{revenue_share, per_seat}` — reseller with internal team. Monthly per-seat invoice + monthly settlement statement.
- `{revenue_share, per_usage}` — marketplace seller with metered ops. Monthly per-meter invoice + monthly settlement statement.
- `{per_seat, per_usage}` — enterprise with consumption workload. Combined monthly invoice (per-seat + per-meter lines).
- `{revenue_share, per_seat, per_usage}` — complex enterprise reseller. Combined monthly invoice + monthly settlement statement.

## 8. Cross-microservice handoffs

### 8.1 cloud-iam (Phase-0 service #11)

cloud-iam reads cloud-billing's tenant-class read API at every principal issuance (login, token refresh, service-account token mint). cloud-iam embeds `principal.tenant_class`, `principal.tenant_id` (per ADR-0244), `principal.billing_components` (Cedar set type), and `principal.cap_breached` (boolean; relevant for demo_trial) as principal claims.

Token TTL: 1 hour default; tenant-class-mutated events propagate via gRPC substrate within 30 sec, triggering token refresh.

Direction: gRPC unary call from cloud-iam to cloud-billing's `GetTenantClass(tenant_id) → (tenant_class, billing_components, cap_breached)` API. Endpoint: `cloud-billing.internal.oyatie.dev:50051` (gRPC-over-HTTP/3 per ADR-0253).

### 8.2 cloud-billing-tax (Phase-0 service #13)

cloud-billing produces tax-naive subtotal per line item; cloud-billing-tax overlays per-jurisdiction tax-line response.

Direction: gRPC unary call from cloud-billing to cloud-billing-tax's `ComputeTax(line_items, tax_invoice_format, regional_pack, tenant_id, billing_components) → tax_lines` API.

Per-component tax treatment:
- per_seat invoices: jurisdictional SaaS tax rules (ship-from / ship-to / tenant-billing-address).
- per_usage invoices: jurisdictional consumption tax (EU digital services VAT; US-state SaaS tax variability).
- revenue_share settlements: jurisdictional payee-side withholding (US W-9/W-8 backup withholding; EU VAT reverse-charge; APAC per-country withholding).

### 8.3 payments (Phase-1 service #07)

cloud-billing emits `settlement_statement_event` to payments with direction + amount + destination payment method. payments executes money movement; emits `payout_completed_event` or `invoice_issued_event` back.

Direction: gRPC bidirectional stream cloud-billing ↔ payments. Settlement statements include audit-chain hash for provenance.

Stripe / Adyen / Toss / NICE-Payments / KakaoPay / 페이코 / KCP are payments providers; cloud-billing never references them directly. All payment-method shape is opaque (`PaymentMethodRef`).

### 8.4 audit-chain (Phase-0 service #14)

Every cloud-billing event seals into audit-chain per ADR-0263. Audit-chain event class taxonomy is registered with audit-chain's central registry; cloud-billing publishes its taxonomy to `audit-chain/registry/cloud-billing-event-classes.yaml`.

### 8.5 cloud-storage (Phase-0 service #06)

FOCUS 1.1 exports + settlement statement PDFs + invoice PDFs are stored in cloud-storage with object-lock retention per the contract's retention class (default SOX 7 years; FedRAMP 3 years; K-FSI 5 years; tenant-contract may be longer).

### 8.6 cloud-kms (Phase-0 service #10)

Invoices and settlement statements are signed by cloud-kms before issuance. Signature is non-spoofable header on PDF + JSON.

### 8.7 observability (Phase-0 service #15)

cloud-billing emits per-tenant cost metrics (cost-by-tenant, cost-by-meter, cost-by-pricing-dimension, monthly-close-latency, fx-lock-fetch-latency, settlement-window-latency) via OpenTelemetry.

### 8.8 cloud-compute-{vm,k8s,functions} (Phase-0 services #01-#03)

Each cloud-compute-* µservice emits per_usage meter events to cloud-billing per the meter shape declared in §7.4. cloud-billing aggregates per (meter_unit, pricing_dimension).

The retired `cloud-compute` singular reference is replaced with the canonical three µservices throughout.

### 8.9 finops-portal (Phase-1 product surface)

finops-portal is the tenant-facing FinOps surface that presents cloud-billing's data. Per ADR-0245 substrate-vs-product layering, cloud-billing is substrate; finops-portal is product. finops-portal owns the tenant UX; cloud-billing owns the data + invariants.

### 8.10 notifications (Phase-1 service)

cloud-billing emits notifications for:
- T-7d, T-3d, T-0 trial expiry warnings
- 80% cap-breach warnings
- 100% cap-breach + grace start
- Grace window expiry without conversion
- Conversion success
- Monthly invoice ready
- Settlement statement ready
- Payout completed
- Dunning retry initiated
- Delinquent state entered

### 8.11 tenancy (Phase-0 service #16)

tenancy reads tenant_class for tenant lifecycle UX (display "trial — 12 days remaining", "paid", etc.). tenancy does NOT mutate tenant_class; mutation is cloud-billing's authority.

### 8.12 governance (Phase-0 substrate)

governance enforces the `oya-governance-cloud-billing-source-of-truth`, `oya-governance-tenant-class-enum-closed`, `oya-governance-billing-components-subset-closed`, and `oya-governance-paid-quality-bar-parity` lanes per ADR-0330 §enforced_by.

### 8.13 marketplace (Phase-1 product surface)

marketplace listings are gated by `billing_components.contains_revenue_share()` when the tenant is paid. demo_trial tenants may consume free listings only.

## 9. Constraints + Invariants

### 9.1 Kernel invariants (already enforced by `oya-cloud-billing-domain`)

- Every `BillingAccount` is created with `data_class == Financial` (kernel rejects others).
- Every `CloudBillingEvent` is created with `data_class == Public` (kernel rejects others).
- `tenant_id` matches the resource's tenant per `ResourceId::tenant_id()`.
- `region` matches the resource's region per `ResourceId::region()`.
- `metering_tag` follows `oya:metering:<tenant_id>:<kind>` shape.
- `idempotency_key` deduplication is keyed on the field; duplicates return the original event.
- `Money::checked_add` rejects cross-currency addition.
- `BillingPeriod` rejects start ≥ end.
- `Invoice` rejects `due_at ≤ issued_at` and `issued_at < period.end`.
- `TaxInvoiceFormat` is derived from `regional_pack` (closed enum mapping).
- `TaxRegistrationId` shape is per-format-validated.
- Invoice totals reconstruct from line item subtotals.
- Duplicate invoice IDs are rejected.

### 9.2 Cross-cell invariant (per ADR-0248)

cloud-billing is cell-aware. The metering bus is per-cell; cross-cell traffic is forbidden for demo_trial tenants. Paid tenants may operate cross-cell sub-tenancies with explicit Cedar grant.

### 9.3 Single-concern invariant (per ADR-0132)

cloud-billing is single-concern. Tax computation lives in cloud-billing-tax (a separate µservice per ADR-0131 flat layout). Payment-method handling lives in payments. cloud-billing does not embed downstream provider logic.

### 9.4 Substance-bar invariant (per ADR-0328 §D-4.11)

The kernel + this PRD + ARCHITECTURE + contracts + SLOs + Cedar + IaC + OS manifest together enable a cold intern to build the µservice. No required canonical surface is absent.

### 9.5 Quality-bar parity invariant (per ADR-0330 §B.9)

demo_trial and paid tenants receive uniform performance SLO targets, uniform scalability targets, uniform security posture, uniform observability coverage, uniform accessibility compliance, uniform localization coverage. The only acceptable difference is SLO commitment posture (paid carries contractual penalty; demo_trial is best-effort).

CI-enforced via `oya-governance-paid-quality-bar-parity`.

### 9.6 Cedar-gate-only invariant (per ADR-0243)

Every state-mutation surface is gated by a named Cedar action + a named Cedar policy. There is no inline `if tenant_class == "demo_trial"` guard.

### 9.7 Tenant-scoping invariant (per ADR-0244)

Every audited row carries tenant_id; every cloud-billing row now also carries tenant_class snapshot and billing_components snapshot at the time of the audited operation.

### 9.8 No-stratification invariant

No microservice may stratify product behavior by tenant_class. Cap-hit messages are acceptable ("you have reached your trial limit; upgrade to continue"); feature lockout is not. Differential accuracy / latency / telemetry is not. Differential SLO commitment is acceptable.

## 10. Out of Scope (intentional)

- **Per-tier capability gating.** Retired with ADR-0316 superseded by ADR-0330.
- **Payment method storage.** Owned by payments. cloud-billing stores opaque `PaymentMethodRef` only.
- **Tax line computation.** Owned by cloud-billing-tax. cloud-billing produces tax-naive subtotal only.
- **Fraud detection.** Owned by payments + dedicated fraud-detection µservice.
- **Cost forecasting (multi-month).** finops-portal presents projections; cloud-billing provides the underlying rolling-30-day data.
- **Cost-allocation policy authoring.** finops-portal owns the policy UX; cloud-billing executes the attribution rules.
- **Customer success outreach.** crm owns the renewal outreach; cloud-billing emits the underlying lifecycle events.
- **Inline payment authorization decisions.** payments owns the authorization decision; cloud-billing receives the result.

## 11. Tradeoffs

### 11.1 Composable subset vs single contract template

The composable `billing_components` set provides 8 valid combinations vs a single contract template's 1 shape. Tradeoff: implementation complexity (8 settlement paths × N µservices contributing meters) vs commercial honesty (real customers buy composed shapes; forcing them into a single template misrepresents the commercial reality).

Decision: composability per ADR-0330. Implementation complexity is bounded by per-component orthogonality (B.2.7) — no component blocks another, requires another, or depends on another at the config layer.

### 11.2 Monthly close vs continuous billing

Continuous billing (real-time per-transaction invoicing) would reduce DSO (days-sales-outstanding) and remove the monthly-close batch from the SLO budget. Tradeoff: ledger consistency complexity (multi-day FX windows, multi-day clawback handling, multi-day tax-rate changes mid-month) vs operational visibility (continuous in finops-portal already).

Decision: monthly close for invoice generation; continuous for usage visibility. This matches Stripe Billing's posted cadence and aligns with ASC 606 revenue recognition periods.

### 11.3 Strict ISO 4217 currency code vs Oyatie internal credit code (OYC)

The kernel's `CurrencyCode::new` requires three ASCII uppercase letters. ISO 4217 + the Oyatie internal credit code `OYC` both pass this check. OYC is the internal credit currency used for demo_trial credits and for inter-cell settlement during cross-cell sub-tenancy operations.

Tradeoff: strict ISO 4217 only would reject OYC (we'd need a separate type for internal credits, doubling the surface). Accepting non-ISO 3-letter uppercase codes risks accidental drift (someone mints "XYZ" for a non-canonical purpose).

Decision: accept any 3-letter ASCII uppercase; document the Oyatie-internal codes in a CurrencyCode appendix; CI lane `oya-governance-currency-code-allowlist` enforces the closed set in production tenant invoices.

### 11.4 Demo_trial reverse-DNS tenant naming vs `ten_*` prefix

The kernel requires `tenant_id` to start with `ten_`. Per Q-11 in the coherence audit, the canonical decision per the user directive is dotted reverse-DNS (`oyatie.b2b.smb.acme-software`) for the tenant business name, with a deterministic SHA-256 hash producing the kernel-canonical `ten_*` form for short references.

Decision: the tenancy µservice mints `ten_<hash12>` from the dotted reverse-DNS name at tenant-creation time; the dotted reverse-DNS is stored as `tenant.display_name`; the `ten_*` form is the kernel-canonical identifier. demo_trial tenants use `demo_*` prefix for the canonical identifier; the dotted reverse-DNS is `oyatie.demo.<segment>.<prospect-name>`.

### 11.5 Composable orthogonal billing vs single revenue model

A single revenue model (e.g., "per_seat only" or "per_usage only") would simplify pricing collateral and finops modeling. Tradeoff: misalignment with how real Oyatie customers buy (per Stripe / Palantir / Snowflake honest model per ADR-0330 §A.3).

Decision: composability. Marketing collateral handles tier-segmented presentation; engineering layer is composable.

## 12. Open Questions (deferred to Wave 14 aggregation)

The 15 open questions from the coherence audit §5 are documented at `coherence-audit-2026-05-20.md`. The PRD-affecting questions are:

- Q-3 (billing_components mid-contract mutation proration semantics) — recommend: pro-rate to remaining days in cycle, settle removed component before mutation takes effect, no cross-component pro-ration.
- Q-4 (revenue_share clawback policy) — recommend: clawback fully recovers oyatie's commission cut; netted in next settlement; if net negative, invoice the tenant.
- Q-5 (subscription primitive) — confirmed: Subscription is a first-class primitive bound to paid tenant + billing_components.
- Q-8 (cloud-compute meter taxonomy) — confirmed in §7.4 meter shape table.
- Q-12 (BEPS Pillar Two scope) — confirmed: scope is per-tenant per-jurisdiction subtotal export; group-level rollup is out-of-scope (tenant aggregates downstream).

## 13. Acceptance Criteria

The PRD is accepted when:

1. CI lane `ci-tenant-class-adoption-check` is green for cloud-billing.
2. Kernel implements `TenantClass`, `BillingComponentSet`, the 4 new `CloudBillingEventKind` variants, and the conversion / cap-breach / grace-window / settlement / seat-counter / meter-aggregator modules per the implementation plans (IP-001..IP-015).
3. Contracts (OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3) cover every FR row in §4.
4. SLOs (10+ OpenSLO files) cover every NFR row in §5.1.
5. Cedar policies (6+ files) cover every gate in §6.4.
6. IaC modules (6 deployment-context dirs + 1 OCI Always Free dir) cover every deployment context in ADR-0218.
7. supported-oses.json declares all 13 Tier-1 OSes + Tier-2 + out-of-scope.
8. Audit-chain registry includes every event class in §5.4.
9. competitor-parity-matrix.md covers ≥100 capabilities from the Stripe Billing + AWS B&CM + Recurly UNION.
10. The 12 P0 findings in the coherence audit §4 are addressed.

## 14. Migration Plan

### 14.1 Wave 15B (this sprint — spec authoring)

- PRD.md (this file)
- ARCHITECTURE.md
- README.md
- contracts/openapi.yaml + asyncapi.yaml + proto/cloud-billing.proto
- slos/*.openslo.yaml (≥10)
- policies/*.cedar (≥6)
- iac/<context>/ (6 contexts + OCI Always Free)
- supported-oses.json
- decisions/ADR-MS-001 + ADR-MS-002
- implementation-plans/IP-001..IP-015
- competitor-parity-matrix.md
- REMEDIATION-NOTES-2026-05-21.md

### 14.2 Wave 15B (this sprint — kernel extension)

- TenantClass enum + BillingComponentSet (kernel)
- New CloudBillingEventKind variants
- Conversion engine
- Cap-breach detection
- Grace-window state machine
- Settlement engine
- Seat counter
- Meter aggregator
- Subscription primitive
- FX lock service module
- Audit-chain emission with new event class names

### 14.3 Wave 15A (P0 contradictions cleanup)

- Reconcile idempotency_key vs event_id naming + 7-day TTL on dedup BTreeMap.
- Align tenant_id shape (reverse-DNS + hash).
- Align resource_id shape (kernel-canonical with SDK helper).
- Document CurrencyCode appendix (OYC + ISO 4217).
- Pick lowercase-dotted snake-case event class naming.

### 14.4 Wave 15J (tenant_class migration)

- Delete `tenant_class adoption record`.
- Rewrite benchmarks per deployment context.
- Rewrite runbook SLO authority per unified bar + deployment-context overlay.
- Rewrite onboarding + tutorials + migration-playbooks to use tenant_class + billing_components vocabulary.

### 14.5 Wave 15I (foundry retirement)

- Rewrite cross-µservice lists in runbooks to absorb foundry references into intelligence + workflow-engine + workflow-studio + ontology + governance + tenancy per ADR-0328 §D-12.

### 14.6 Wave 15H (cross-ref cleanup)

- Add `payments` to runbook cross-µservice lists.
- Rewrite `cloud-compute` singular → canonical `cloud-compute-{vm,k8s,functions}`.
- Rewrite "wraps AWS CUR" phrasing.
- Cite ADR-0263 in audit-chain integration section.

## 15. Cross-references

- ADR-0330 — tenant class + composable billing components (keystone authority)
- ADR-0329 — tier system retirement
- ADR-0331 — per-microservice tenant-class adoption template
- ADR-0328 §D-1, §D-4, §D-15..§D-20 — Phase-0 placement + 5-dimension audit + multi-context + IaC + OS + Rust-strict + OCI Always Free
- ADR-0244 — tenant-as-universal-scoping-primitive
- ADR-0243 — Cedar-as-universal-gate
- ADR-0251 — compliance pack primitive
- ADR-0255 §D-4 — BYOK credentials gated by tenant_class
- ADR-0249 — multi-category marketplace
- ADR-0131 — per-microservice flat layout
- ADR-0132 — no-grouping policy
- ADR-0145 — direct-gRPC inter-µservice
- ADR-0263 — audit-emission contract
- ADR-0130 — agentic SLO-gated promotion
- ADR-0253 — HTTP/3 + QUIC default
- ADR-0252 — HLC default for causality (TrueTime opt-in for fin-grade)
- ADR-0248 — Amazon cellular architecture
- ADR-0218 — per-tenant deployment context
- ADR-0215 — multi-context engine
- ADR-0216 — open integration
- ADR-0064 — canonical-base neutrality (KR pack overlays)
- ADR-0039 — sigstore + cosign signing for IaC modules

## 16. Authoring substance bar evidence

This PRD is authored under the substance-bar requirement of ADR-0322 (line floor 800; bespoke clauses, not templated text). Every section is bespoke; the meter shape table in §7.4 is per-µservice; the cross-µservice handoff table in §8 is per-µservice; the FR table in §4 is per-FR; the NFR table in §5 is per-metric; the persona table in §3 is per-persona.

The PRD pre-supposes the kernel implementation in `crates/oya-cloud-billing-domain/src/lib.rs` (1,030 lines as of 2026-05-21) and extends the kernel to address billing_components composability per ADR-0330 §D-1. Kernel preservation is mandatory; the kernel's invariants are the substance truth that the PRD describes.

## 17. Authoring history

| Date | Revision | Author | Notes |
|---|---|---|---|
| 2026-05-21 | 1.0.0 | axis-cloud-billing | Initial authoring under Wave 15B spec-authoring sprint per coherence-audit-2026-05-20.md remediation. Addresses 12 P0 findings. |

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

<!--
COMPLETION REPORT — Wave 15B cloud-billing spec-authoring sprint
================================================================

This PRD addresses the kernel-ahead-of-spec inversion identified in
coherence-audit-2026-05-20.md. The 1,030-line Rust kernel in
crates/oya-cloud-billing-domain is the SUBSTANCE TRUTH; this PRD
documents what the kernel does and extends to address billing_components
composability per ADR-0330.

12 P0 audit findings addressed by this PRD + sibling artifacts:
- CB-F-001 PRD missing → THIS FILE
- CB-F-002 ARCHITECTURE missing → ARCHITECTURE.md
- CB-F-003 foundry references → §8 + remediation notes
- CB-F-004 tenant_class migration → §1 + §3 + §5 + remediation notes
- CB-F-005 tenant_class authority → §6 + ADR-MS-001
- CB-F-006 billing_components model → §7 + ADR-MS-001
- CB-F-007 OpenTofu IaC missing → iac/*/
- CB-F-008 multi-context manifest → supported-deployment-contexts inline §6.5
- CB-F-009 OS manifest missing → supported-oses.json
- CB-F-010 contracts missing → contracts/
- CB-F-011 SLOs missing → slos/
- CB-F-012 Cedar policies missing → policies/

Deliverables shipped under this sprint:
1. PRD.md (this file, ~885 lines)
2. ARCHITECTURE.md
3. README.md
4. contracts/openapi.yaml + asyncapi.yaml + proto/cloud-billing.proto
5. slos/*.openslo.yaml (10 files)
6. policies/*.cedar (6 files)
7. iac/<context>/ (7 dirs incl. OCI Always Free)
8. supported-oses.json
9. decisions/ADR-MS-001-billing-components-composability.md
10. decisions/ADR-MS-002-revenue-share-settlement-pipeline.md
11. implementation-plans/IP-001..IP-015
12. competitor-parity-matrix.md (>100 capabilities)
13. REMEDIATION-NOTES-2026-05-21.md

NO COMMITS produced per execution rules. NO scripting/stamping.
KERNEL PRESERVED — crates/oya-cloud-billing-domain untouched.
SPEC AUTHORING ONLY — billing/ scope only.
-->

## ADR-0339 adoption
- Lifecycle: PROPOSED for `cloud-billing` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `cloud-billing` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 9 module pin(s) across 5 context(s).
- Scaling input: `per_message` with cell placement `Tier-3` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
