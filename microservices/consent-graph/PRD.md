---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-consent-graph
microservice: consent-graph
status: Drafted
authority_tier: 2
owner_team: axis-consent-graph
related_adrs: [ADR-0003, ADR-0028, ADR-0056, ADR-0058, ADR-0078, ADR-0090, ADR-0105, ADR-0110, ADR-0130, ADR-0131, ADR-0132, ADR-0214, ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345]
related_specs: [/specs/compliance-pack-floors.json, /specs/finops-dimensional-model.json]
date: 2026-05-18
doc_status: drafted
---

# consent-graph — Product Requirements Document

- Owner: axis-consent-graph
- Status: Drafted → target Active upon merge of PR #143 to `dev`
- Authority ADR: docs/decisions/ADR-0214-cross-tenant-real-time-visibility.md
- Related ADRs: see `related_adrs` frontmatter; legacy anchors include ADR-0003 (audit-chain), ADR-0028 (cloud µservice arch), ADR-0056 (clean architecture
  BNF), ADR-0058 (ontology), ADR-0078 (Pulsar substrate), ADR-0090 (Cedar policy engine), ADR-0105
  (13-layer enum), ADR-0110 (state machine), ADR-0130 (SLO-gated promotion), ADR-0131 (flat layout),
  ADR-0132 (no-suite policy).
- Date: 2026-05-18
- Schema version: 1.0

---

## 1. Problem statement

Enterprises in shared ecosystems (supply chain, healthcare network, financial mesh, marketplace) cannot
exchange real-time entity-level data with partners without sacrificing one or more of:

1. **Audit defensibility** — who saw what, when, under what authority.
2. **Revocability** — partner access stops immediately when business relationship ends.
3. **Sovereignty** — grantor's data physically stays in grantor's region.
4. **Scope narrowness** — partner sees only the fields the agreement permits.
5. **Real-time** — sub-second freshness for operational decisions.

Existing solutions force you to pick at most two. EDI gives you none. Per-tenant API tokens give you
none. Snowflake Secure Data Share gives you (1) and (3) but not (2), (4), or (5). Open Banking gives
you (1), (2), and (4) but is financial-only and not real-time-entity-shaped. Hyperledger gives you (1)
but burns the others.

We must give partners **all five**.

## 2. Users and personas

| Persona | Role | Primary action | Frequency |
|---------|------|----------------|-----------|
| Partnership-Manager (grantor) | Business owner of partner relationships | Draft + offer agreement | weekly |
| Partnership-Manager (grantee) | Business owner of inbound visibility | Accept agreement | weekly |
| Data-Steward (grantor) | Data-governance owner | Approve scope, set redaction | per-agreement |
| Compliance-Officer (grantor + grantee) | Audit + regulatory | Read audit chain, approve sensitive scopes | per-agreement + monthly review |
| Operations-User (grantee) | Day-job operator | Consume projection in app | continuous |
| Integration-Engineer (grantee) | Software | Subscribe to projection topic via SDK | per-onboarding |
| Security-Operator (both sides) | Incident response | Revoke immediately, investigate forgery | on-incident |
| Privacy-Officer (grantor) | Data subject rights | Enforce DSAR / right-to-erasure across grants | per-DSAR |

## 3. Goals (must)

1. First-class `DataSharingAgreement` entity with full lifecycle state machine.
2. Cedar enforcement at every cross-tenant hop, deny-by-default.
3. Bilateral audit chain entries on every grant, modification, projection-read, revocation.
4. Real-time revocation: p99 propagation ≤1s, p100 ≤3s.
5. Zero-copy projection: grantor's row never physically migrates to grantee's region.
6. Three sharing modes: Projection / Aggregate / AttestedQuery.
7. Scope narrowing: entity-level, field-level, predicate-level, and k-anonymity-level (for Aggregate).
8. Geographic constraints: agreement may forbid cross-border-transfer; enforced at acceptance + projection.
9. Partner-directory: handshake protocol to register peer tenant + exchange trust anchors.
10. SDK: typed Rust + TypeScript + Python clients for both grantor and grantee sides.
11. Self-service revocation by data subject (B2C use case) when agreement is consumer-initiated.
12. Compliance pack overlays: kr, eu, us, us-healthcare, jp, sg, au, in, br, ae, ksa.

## 4. Non-goals (explicit out of scope)

1. Cross-tenant *write* (only reads/projections; writes remain grantor-internal).
2. Multi-grantor joins (each agreement is bilateral; multi-grantor analytics is a Workflow Studio
   concern, not consent-graph's).
3. Token-based access (we are agreement-based, not token-based; SDKs use mTLS + agreement-bound JWT,
   not bearer tokens).
4. Replacing the audit-chain µservice (consent-graph emits *to* audit-chain, doesn't reimplement it).
5. Replacing the ontology µservice (consent-graph extends ontology's projection model with cross-tenant
   ACLs; doesn't reimplement entity storage).
6. Blockchain settlement (Merkle-sealed bilateral chain is sufficient — see ADR-0214 §3.5).
7. Pricing/billing for grants (separate µservice).
8. Discovery/marketplace for finding partners (separate µservice).

## 5. Success metrics

| Metric | Target | SLO file |
|--------|--------|----------|
| Consent-grant E2E latency p95 | ≤2s | slos/consent-grant-latency.openslo.yaml |
| Cross-tenant projection freshness p95 | ≤500ms grantor-commit → grantee-visible | slos/cross-tenant-projection-freshness.openslo.yaml |
| Revocation propagation latency p99 | ≤1s | slos/revocation-propagation-latency.openslo.yaml |
| Cedar evaluation latency p99 | ≤10ms | slos/cedar-evaluation-latency.openslo.yaml |
| Audit-chain coverage completeness | 1.0 (no event un-sealed) | slos/audit-chain-coverage-completeness.openslo.yaml |
| Agreement state divergence rate | 0 (grantor + grantee always agree on state) | slos/agreement-state-divergence-zero.openslo.yaml |
| Sovereignty violation count | 0 (no projection escapes constrained region) | slos/sovereignty-violation-zero.openslo.yaml |
| Bilateral chain link integrity | 1.0 (every grantor entry has paired grantee entry) | slos/bilateral-chain-link-integrity.openslo.yaml |
| Partner handshake completion latency p95 | ≤30s | slos/partner-handshake-latency.openslo.yaml |

## 6. Worked use cases (each one must work day-1)

### 6.1 Manufacturer → Retailer (supply chain)
- Agreement: scope `{entity: FinishedGoodsInventory, fields: [sku, qty_on_hand, ship_eta], predicate: "sku IN retailer.open_po_skus"}`.
- Mode: Projection.
- Geographic: grantor-region only, retailer's WMS subscribes cross-region read.
- Revocation trigger: PO closed.
- Cedar policy excerpt:
  ```cedar
  permit (
    principal in Tenant::"retailer-acme",
    action == Action::"project.subscribe",
    resource in EntityType::"FinishedGoodsInventory"
  )
  when {
    resource.sku in principal.open_po_skus &&
    context.purpose_of_use == "inventory-visibility"
  };
  ```

### 6.2 Retailer → Logistics Provider (supply chain)
- Agreement: scope `{entity: Shipment, fields: [tracking_id, status, eta, exception_code]}` — excludes
  `customer_pii`, `order_value`.
- Mode: Projection.
- Revocation trigger: shipment delivered (auto-revoke) or contract end.

### 6.3 Hospital → Health Insurer (healthcare)
- Agreement: scope `{entity: PatientEligibility, fields: [member_id, coverage_status, copay_tier], predicate: "patient in active_care_set"}`.
- Mode: AttestedQuery (insurer asks, hospital answers with sealed result).
- Purpose-of-use: `eligibility-verification`; break-glass `emergency-treatment` with mandatory
  post-hoc audit-officer review.
- Compliance: US-HIPAA pack overlay mandates min-necessary log + 7y retention.

### 6.4 Vendor → Buyer (procurement)
- Agreement: scope `{entity: PurchaseOrderState, fields: [po_id, line_status, expected_ship_date, invoice_state]}`.
- Mode: Projection.
- Revocation: PO completed + invoice paid.

### 6.5 Brand → Consumer (B2C)
- Agreement: consumer-initiated; brand auto-accepts via standing policy.
- Scope: `{entity: ConsumerOrder, fields: [order_id, status, carrier_tracking, eta]}` — restricted to
  consumer's own orders by `predicate: "order.consumer_id == principal.consumer_id"`.
- Self-revocable by consumer at any time.

### 6.6 Bank → Customer / Fintech (Open Banking parity)
- Agreement: customer grants fintech read.
- Scope: `{entity: Account, fields: [balance, transaction_status]}` and `{entity: Transaction, fields: [amount, merchant, timestamp, status]}`.
- Mode: AttestedQuery for balance, Projection for transactions.
- Compliance: EU PSD2 + KR Open Banking pack overlay.

### 6.7 Marketplace → Seller (marketplace)
- Agreement: marketplace grants seller aggregate cohort stats.
- Mode: Aggregate (k≥5).
- Scope: `{entity: BuyerBehavior, aggregation: [view_count, cart_count, purchase_count], group_by: [region, week]}`.
- Privacy: differential-privacy ε=1.0, k-anonymity k=5 enforced at grantor-side aggregation.

## 7. Functional requirements

### 7.1 Agreement bounded context
- F-AGR-1 CRUD on `DataSharingAgreement` with optimistic-concurrency.
- F-AGR-2 Lifecycle state machine: `Drafted → Offered → Accepted → Active → {Suspended, Revoked, Expired}`.
- F-AGR-3 Bilateral acceptance: agreement is not `Active` until grantee accepts.
- F-AGR-4 Cedar policy auto-compilation at acceptance from scope + terms.
- F-AGR-5 Agreement templates: 5 starter templates (supply-chain, healthcare, banking, marketplace, B2C).
- F-AGR-6 Versioned scope amendments: amending scope creates a new immutable agreement version + supersedes
  the prior.
- F-AGR-7 Expiration warnings emitted 30d/7d/1d before expiry.

### 7.2 Enforcement bounded context
- F-ENF-1 Single gateway API `enforcement::evaluate(grantor, grantee, entity, action, context) → Permit|Deny`.
- F-ENF-2 Deny-by-default; absent agreement = deny.
- F-ENF-3 Latency budget: p99 ≤10ms.
- F-ENF-4 Failure mode: closed (deny) on unreachable consent-graph; emit audit event.
- F-ENF-5 Cedar compiled-policy cache keyed by `agreement_id`; cache invalidated on revocation event.

### 7.3 Revocation bounded context
- F-REV-1 Revocation API: `revoke(agreement_id, reason)` → emits to Pulsar within 100ms.
- F-REV-2 Pulsar topic `oya.consent-graph.revocation.v1` partitioned by `(grantor, grantee)`.
- F-REV-3 Subscriber workers in ontology, analytics, observability close projection subscribers within 1s.
- F-REV-4 Audit-chain entries on both sides confirming propagation.
- F-REV-5 Idempotent: re-revocation is a no-op.

### 7.4 Projection-gateway bounded context
- F-PRJ-1 Mints Pulsar topic per `(grantor, grantee, entity)` triple at agreement-acceptance.
- F-PRJ-2 Topic ACL: tenant-aware; only grantee may subscribe.
- F-PRJ-3 Topic resides in grantor's Pulsar cluster (sovereignty).
- F-PRJ-4 Scope-narrowed: grantor-side projection emits only permitted fields.
- F-PRJ-5 Aggregate mode: emits aggregated tuples with k-anonymity guard at emission time.

### 7.5 Audit-bridge bounded context
- F-AUD-1 Every consent-graph event (grant, accept, amend, revoke, project-subscribe, project-read,
  enforcement-deny) emits to audit-chain on **both** sides.
- F-AUD-2 Bilateral cross-pointer: each grantor entry carries the paired grantee entry's
  `(chain_id, seq)`; symmetric on grantee side.
- F-AUD-3 Lag budget: ≤500ms from event to audit-chain seal-ready.

### 7.6 Partner-directory bounded context
- F-DIR-1 Register peer tenant: handshake exchange of trust anchor (mTLS cert + Pulsar token issuer
  pub-key).
- F-DIR-2 Verify peer's audit-chain Merkle root (proves peer runs audit-chain µservice).
- F-DIR-3 Partner status: `Onboarding → Verified → Active → Suspended → Offboarded`.

## 8. Non-functional requirements

### 8.1 Scale targets (year-1)
- 10M active agreements, 100K new agreements/day, 1M revocations/day.
- 100B projection events/day across all agreements.
- 10K concurrent partner-directory peers.
- p99 enforcement latency ≤10ms at 100K req/s.

### 8.2 Availability
- consent-graph µservice: 99.99% (4 nines), region-redundant.
- Revocation propagation: 100% (no missed revocations is a hard requirement; if propagation fails, all
  cross-tenant reads must fail closed).

### 8.3 Sovereignty
- Grantor row never physically migrates.
- Projection cache may live in grantee region only if `geographic_constraints` permits.
- Cross-border-transfer-forbidden agreements: projection cache must be co-located with grantor or
  agreement is rejected at acceptance.

### 8.4 Compliance
- KR PIPA cross-border-transfer §17, §18.
- EU GDPR Art. 28 (processor), Art. 44–49 (cross-border transfer).
- US HIPAA min-necessary, 45 CFR §164.502(b).
- US-state CCPA / CPRA right-to-know, right-to-delete.
- JP APPI cross-border-transfer §24.
- SG PDPA §26 cross-border-transfer.

### 8.5 Privacy
- DSAR cross-tenant: when grantor receives a DSAR for subject X, consent-graph enumerates every active
  agreement projecting subject X's data, fans out a tombstone signal, and audit-chains the cascade.
- Right-to-erasure: cascade across all grantees within 7 days (regulatory cap).

### 8.6 DR posture (ADR-0343)

- Target: RTO ≤1800s and RPO ≤300s for agreements, revocations, enforcement decisions, partner trust anchors, and bilateral audit pointers, matching manifest `dr.rto_p99_seconds=1800` and `dr.rpo_p99_seconds=300`.
- Compliance-pack floors considered: EU-AI-ACT-2024-HIGH-RISK (1800s/300s, multi-region), HIPAA-2024 (3600s/300s, multi-region), KR-CSAP-v3.1 (3600s/900s, multi-region), SOC2-T2 (14400s/900s), PCI-DSS-L1-v4 (86400s/3600s), ISO27001-2022/SOX-404 (14400s/3600s), and KR-PIPA-2023-amendment (14400s/900s). Effective target is RTO 1800s, RPO 300s, multi-region required.
- Failover runbook: `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, matching manifest `dr.failover_runbook`; revocation recovery uses `microservices/consent-graph/runbooks/revocation-incident.md`.
- Multi-region active-active: yes for agreement state, enforcement cache invalidation, partner-directory trust anchors, and revocation fan-out; projection data remains grantor-region resident.
- WHY: partners and data subjects can revoke or enforce agreements through a regional outage without losing bilateral non-repudiation or violating sovereignty constraints.

### 8.7 Capacity model (ADR-0340)

- Per-tenant baseline: 0.12 vCPU, 192 MiB RAM, 4 GiB active-agreement/index storage, 3 Valkey connections, 3 Postgres connections, and 5 outbound Pulsar/audit-chain/partner-directory slots, matching manifest `capacity_model`.
- Scaling dimension: `per_message`, because manifest doctrine treats revocations, agreement changes, audit emissions, and partner callbacks as consent-event shaped.
- Cell placement class: Tier-1 cross-tenant policy substrate, matching manifest `capacity_model.cell_placement_class`; runtime placement maps to pod runtime Tier 1 because manifest `pod_runtime_tier=1`.
- Autoscaling boundary: minimum 2 enforcement replicas, 1 revocation worker, 1 projection worker, and 1 audit bridge per pack/cell; maximum 12 enforcement replicas and 8 revocation/projection workers per tenant-pair shard before agreement graph partitioning is required.
- WHY: the model supports 10M active agreements, real-time revocation, and high-volume projection reads while keeping each grantor/grantee pair isolated.

### 8.8 Sustainability + cost attribution (ADR-0344)

- Every agreement grant, offer, accept, amend, revoke, projection subscription/read, enforcement decision, partner handshake, and bilateral audit pointer emits `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, and `carbon_intensity_source`.
- Provider-routing affected by carbon: no for revocation, enforcement, HIPAA emergency access, or any realtime projection path; yes for batch reconciliation, monthly partner reports, and non-urgent agreement analytics.
- Per-tenant cost surface: consent admins see agreement/projection cost in the tenant FinOps dashboard filtered by tenant, product, capability, provider, cell, and compliance_pack; bilateral exports preserve both grantor and grantee attribution.
- WHY: cross-tenant visibility is a regulated data-sharing surface, so customer bills and climate disclosures must show who paid for each agreement and which side's cell emitted the carbon.

### 8.9 API versioning posture (ADR-0342)

- Public API version model: agreement, revocation, partner-directory, projection subscription, and tenant-facing audit-export APIs use the YYYY-MM-DD carrier triplet: `Oyatie-Version` header, `/v/<YYYY-MM-DD>/...` URL prefix, and `oyatie_version` proto3 field.
- SDK semver model: grantor/grantee SDKs ship as major.minor.patch, with explicit mappings to supported public date versions.
- Support window: last 3 public API versions for at least 180 days.
- Per-tenant pinning: yes for grantor and grantee API clients; enforcement hot-path callers use mesh compatibility instead.
- Internal-mesh exemption: yes; enforcement and projection-gateway direct gRPC remain exempt under ADR-0145 to preserve low-latency policy checks.

## 9. API surface (high-level)

### 9.1 REST
- `POST /v1/agreements` — draft new agreement (grantor).
- `POST /v1/agreements/{id}/offer` — offer to grantee.
- `POST /v1/agreements/{id}/accept` — grantee acceptance.
- `POST /v1/agreements/{id}/revoke` — either party may revoke.
- `POST /v1/agreements/{id}/amend` — grantor amends scope (creates new version).
- `GET /v1/agreements/{id}` — read agreement (RLS-tenant-scoped).
- `GET /v1/agreements?grantor=...&grantee=...&state=...` — list with filters.
- `POST /v1/enforcement/evaluate` — Cedar evaluation (internal-only).
- `POST /v1/partner-directory/handshake` — peer-tenant handshake.

### 9.2 AsyncAPI / Pulsar
- `oya.consent-graph.agreement-lifecycle.v1` — grant / amend / accept / revoke / expire events.
- `oya.consent-graph.revocation.v1` — high-priority real-time revocation.
- `oya.consent-graph.audit-bridge.v1` — fan-out to audit-chain.
- `oya.consent-graph.projection-mint.v1` — projection topic creation/destruction events.

### 9.3 gRPC (internal)
- `EnforcementService.Evaluate` — hot-path enforcement, p99 ≤10ms.

## 10. Bounded contexts → crates

Per ADR-0131 flat layout + ADR-0105 13-layer enum. 6 bounded contexts × ~8–9 layers each = 51 crates.

(Full list in `manifest.json` § bounded_contexts.)

| BC | Crates | Lead IP |
|----|--------|---------|
| agreement | 9 | IP-001 (kernel), IP-002 (domain), IP-003 (usecase/api/adapter/rest/sdk/app) |
| enforcement | 8 | IP-004 (kernel), IP-005 (domain), IP-006 (usecase + adapter + api) |
| revocation | 8 | IP-007 (kernel + worker), IP-008 (real-time fan-out) |
| projection-gateway | 9 | IP-009 (kernel), IP-010 (mint + ACL), IP-011 (scope narrowing + aggregate) |
| audit-bridge | 7 | IP-012 (bilateral emitter), IP-013 (cross-pointer integrity) |
| partner-directory | 7 | IP-014 (handshake + trust-anchor verification) |
| self-observability | 1 wiring slice | IP-015 (SLO wiring + HG-CONSENT registration) |

## 11. Out-of-µservice integration points

- audit-chain µservice via `audit-chain-emission-sdk` (already shipped).
- ontology µservice via its cross-tenant projection extension (IPs `IP-CT-001..IP-CT-005`).
- observability µservice for self-SLO emission.
- identity µservice for tenant-id + principal-id resolution at enforcement-time.
- api-gateway µservice for north-south enforcement.

## 12. Sequencing

PHASE-01 (this PR) lands the full substrate (15 IPs). Hyperscaler maturity claim gate (ADR-0123) is
NOT claimed at GA in PHASE-01; PHASE-02 (separate PR, post-merge) runs the maturity audit.

## 13. Open questions / future work

- Cross-tenant write-back (out of scope for v1; possibly v2 via Workflow Studio actions with bilateral
  consent on both directions).
- Multi-grantor joins (Workflow Studio concern).
- Marketplace discovery + agreement template store (future µservice).
- Differential-privacy noise budget shared across grantees of same grantor (research item).
