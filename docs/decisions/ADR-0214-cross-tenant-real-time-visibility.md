---
id: ADR-0214
status: Accepted
---

# ADR-0214: Cross-Tenant Real-Time Visibility (Consent-Graph + Ontology Projection Extension)

- Status: Proposed (target: Accepted upon PR #143 merge to `dev`)
- Date: 2026-05-18
- Authors: oyatie axis-consent-graph (with axis-ontology + axis-audit-chain consults)
- Supersedes: none
- Superseded by: none
- Related ADRs:
  - ADR-0003 Audit Chain And Evidence Emission (bilateral chain entries use this substrate)
  - ADR-0028 Cloud Microservice Architecture (per-µservice flat layout per ADR-0131)
  - ADR-0056 Rust Clean Architecture BNF (12+1 layer enum boundary)
  - ADR-0105 13-Layer Enum (canonical layer set used here)
  - ADR-0110 Changeset State Machine (lifecycle states reused for agreement state machine)
  - ADR-0130 Agentic SLO-Gated Promotion (SLO authoring mandatory before dev→stage)
  - ADR-0131 Per-Microservice Flat Layout (consent-graph µservice ships under `microservices/consent-graph/`)
  - ADR-0132 No-Grouping Policy (consent-graph is single-concern, not a suite)
- Coordination surface: `foundry_pipeline`
- Regulatory packs touched: kr, eu, us, us-healthcare, jp, sg, au, in, br, ae, ksa

## 1. Context

Enterprises in a shared ecosystem (supply chain, healthcare network, financial mesh, marketplace) need
real-time visibility into partner-held data when partners grant access. Today this is solved poorly:

- **EDI 850/856/810**: batch document exchange, no consent primitives, no audit primitives, no revocation,
  no real-time. Industry-standard but failure-by-design for modern visibility.
- **Per-tenant API tokens**: every pair-wise relationship requires a bespoke integration; no central audit;
  revocation is token rotation across all consumers; no field-level scope; tokens leak.
- **Shared databases / data lakes**: violates sovereignty (data physically leaves grantor's cell/region);
  data residency laws (GDPR, KR PIPA cross-border-transfer, US HIPAA min-necessary, ME data-localization)
  cannot be honoured.
- **Third-party data integrators** (CommerceHub, GHX, IQVIA, Plaid as intermediary): vendor sits in the
  trust path; outage means visibility blackout; integrator monetizes the data; integrator's audit chain
  is the source of truth, not ours.
- **Hyperledger / blockchain-backed multi-party visibility (IBM Food Trust, TradeLens)**: settlement is
  measured in seconds-to-minutes, not sub-second; consensus overhead is wasted because we already have
  a single source-of-truth per entity (the grantor's Ontology); over-engineered.
- **Snowflake Secure Data Sharing / Databricks Delta Sharing**: zero-copy share at the storage layer
  (good!), but batch-oriented, no real-time event stream, no fine-grained Cedar enforcement, no audit
  chain integration, ties customer to single warehouse vendor.

The **Open Banking pattern** (PSD2 / UK Open Banking / FAPI / Plaid TPP scope) is the right *consent*
model: customer consents → TPP gets narrow read scope → revocable → audited. The **HIE / TEFCA / Direct
Trust pattern** is the right *audit* model: bilateral chain entries on both grantor and grantee sides
with purpose-of-use binding. The **Snowflake Secure Data Share pattern** is the right *storage* model:
zero-copy, data stays in source region.

**No competitor fuses all three.** This is the EaaS moat.

## 2. Decision

Build a single-concern `consent-graph` µservice (`microservices/consent-graph/`) as the kernel for
all cross-tenant data flows, fused with a cross-tenant projection extension to the existing Ontology
µservice. Authorization at every cross-tenant hop is Cedar-enforced. Storage is zero-copy projection
(grantor's Ontology emits projection events to a Pulsar topic; grantee subscribes in grantee's local
cell; grantor's row never physically migrates). Revocation is real-time fan-out (≤1s end-to-end
propagation). Bilateral chain entries are emitted to audit-chain on both sides of every hop.

### 2.1 First-class entity: `DataSharingAgreement`

```rust
pub struct DataSharingAgreement {
    pub agreement_id: AgreementId,             // ULID, monotonic, audit-citable
    pub grantor: TenantId,                     // data holder
    pub grantee: TenantId,                     // data consumer
    pub scope: EntityScope,                    // {entity_type, field_set, predicate}
    pub terms: SharingTerms,                   // {purpose_of_use, mode, redaction, k_anonymity}
    pub lifecycle_state: AgreementState,       // Drafted | Offered | Accepted | Active | Suspended | Revoked | Expired
    pub bilateral_chain_link: ChainLinkPair,   // (grantor_seq, grantee_seq) into audit-chain
    pub revocable: bool,                       // true by default; legal-hold can pin false (rare)
    pub expiration: Option<Timestamp>,         // RFC3339, ≤2y default
    pub geographic_constraints: SovereigntyCfg,
    pub cedar_policy_id: CedarPolicyId,        // pre-compiled policy bound at acceptance
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub schema_version: u16,
}

pub enum SharingMode {
    Projection,           // real-time row-level projection (default)
    Aggregate,            // aggregate-only (k-anonymity-enforced; no row reads)
    AttestedQuery,        // grantee submits query, grantor returns attested result
}
```

The `lifecycle_state` machine reuses ADR-0110 changeset state machine semantics: monotonic, audit-emitted,
revocable from any active state.

### 2.2 Three sharing modes (decision rationale)

| Mode | Latency | Sovereignty | Use case | Privacy guarantee |
|------|---------|-------------|----------|-------------------|
| Projection | ≤500ms p95 | zero-copy, region-pinned | manufacturer→retailer inventory; hospital→insurance eligibility | field-level redaction |
| Aggregate | ≤2s p95 | zero-copy + k-anonymized | brand→consumer cohort analytics; supplier→buyer SKU velocity | k≥5 differential privacy ε≤1.0 |
| AttestedQuery | ≤5s p95 | grantor computes, returns sealed result | bank→fintech balance check; vendor→buyer order status | cryptographic attestation |

### 2.3 Enforcement: Cedar at every hop

Every cross-tenant read or projection-subscribe call passes through `consent-graph::enforcement::api`,
which:
1. Resolves `(grantor, grantee, entity, action)` → matching active `DataSharingAgreement`.
2. Loads the compiled `cedar_policy_id` for that agreement.
3. Evaluates Cedar with full context (purpose-of-use, requesting user, requesting capability tier,
   region, time, prior-revocation-checks).
4. Permits or denies, emitting an audit-chain event on both grantor and grantee sides.
5. **Failure mode is deny**: if `consent-graph` is unreachable, cross-tenant operations fail closed.
   This is by design — better a 503 than a sovereignty violation.

### 2.4 Revocation: real-time, ≤1s propagation

Revocation publishes to a Pulsar topic `oya.consent-graph.revocation.v1` partitioned by `(grantor,
grantee)`. Every µservice that holds a projection subscription (initially: ontology, analytics, observability)
subscribes with `read_compacted=true`; the subscriber's projection-gateway worker:
1. Receives revocation event.
2. Closes any open Pulsar subscriber for that `(grantor, grantee, entity)` triple within 100ms.
3. Tombstones the local grantee-side projection cache within 1s.
4. Emits audit-chain entry confirming revocation propagation.

SLO `consent-graph-revocation-propagation-latency` targets p99 ≤1s, p100 ≤3s (hard cap).

### 2.5 Sovereignty: zero-copy projection, region-pinned

The grantor's raw row never physically migrates. The projection topic for a `(grantor, grantee, entity)`
triple is created in the **grantor's** Pulsar cluster; the grantee subscribes cross-region read-only via
a tenant-aware Pulsar ACL. The grantee's local cache is a denormalized projection, not a copy of the
authoritative row; the projection contains only the fields permitted by the agreement's scope.

The `geographic_constraints` field on the agreement explicitly enumerates which grantee-side regions
may host the projection cache. KR-grantor + EU-grantee with no cross-border-transfer clause → agreement
rejected at acceptance time.

### 2.6 Bilateral chain entries

Every grant, modification, projection-subscribe, projection-read, revocation, expiration emits an audit-chain
event on **both** the grantor-side chain and the grantee-side chain, with cross-pointer links
(`grantor_chain_link → grantee_chain_link`). This is mandatory for HIE-grade audit defensibility:
either side can independently prove the other side's actions.

## 3. Alternatives Considered

### 3.1 Per-tenant API tokens (rejected)
- No central audit primitive. No revocation primitive. No scope-narrowing. Tokens leak.
- N×M integration cost (every pair-wise relationship is bespoke).
- Industry-standard but failure-by-design.

### 3.2 EDI batch (rejected)
- Not real-time (24h+ latency typical).
- No consent primitives, no revocation primitives, no audit primitives.
- Document-shaped, not entity-shaped.

### 3.3 Shared database / data lake (rejected)
- Sovereignty violation: grantor's row physically migrates to shared store.
- GDPR Art. 44 cross-border-transfer violations.
- KR PIPA, US HIPAA min-necessary violations.

### 3.4 Third-party data integrator (rejected)
- Vendor sits in trust path.
- Vendor outage = visibility blackout.
- Vendor monetizes the data (data-broker problem).
- Vendor's audit chain becomes source-of-truth, not ours.

### 3.5 Hyperledger / blockchain (rejected)
- Settlement seconds-to-minutes, not sub-second.
- Consensus overhead wasted (single source-of-truth per entity in grantor's Ontology).
- ~1000× slower than Merkle-sealed bilateral chain for equivalent audit guarantee.
- Smart-contract attack surface.

### 3.6 Snowflake Secure Data Share / Databricks Delta Sharing (rejected as sole solution)
- Right zero-copy storage model, but:
- Batch-oriented (15min+ refresh typical), not real-time event stream.
- No fine-grained Cedar enforcement (RBAC only).
- No audit chain integration.
- Vendor lock-in.
- **Fused into our design** as the zero-copy projection inspiration; not adopted wholesale.

### 3.7 Open Banking / PSD2 / FAPI (partially adopted)
- Right consent model (TPP scope, revocable, audited).
- Financial-only; not vertical-agnostic; not entity-shaped.
- **Consent semantics adopted**, not the protocol.

### 3.8 HIE / TEFCA / Direct Trust (partially adopted)
- Right audit model (bilateral, purpose-of-use, break-glass-with-audit).
- Healthcare-only protocol stack (DIRECT, XDS.b, FHIR-Bulk-Data).
- **Audit semantics adopted**, not the protocol.

## 4. Concrete Use Cases (must work day-1)

1. **Manufacturer → Retailer**: manufacturer grants retailer real-time view of finished-goods inventory
   + ship-ETA for SKUs in retailer's PO scope. Retailer's WMS subscribes to the projection topic.
   Revocation upon PO closure.
2. **Retailer → Logistics Provider**: retailer grants 3PL real-time shipment-status visibility for active
   shipments. Field-level scope: tracking-id, status, ETA, exception-code. Excluded: customer PII, order
   value.
3. **Hospital → Health Insurer**: hospital grants insurer real-time eligibility-verification access for
   patients in care. Purpose-of-use: `eligibility-verification`. Break-glass for `emergency-treatment`
   with mandatory audit review.
4. **Vendor → Buyer**: vendor grants buyer real-time PO-status, ASN, invoice-state visibility per the
   buyer's open POs. Revocation on PO completion.
5. **Brand → Consumer (B2C)**: brand grants consumer real-time order-tracking. Scope: order-id, status,
   carrier-tracking, ETA. Self-revocable by consumer at any time.
6. **Bank → Customer / Fintech**: bank grants customer (or customer-authorized fintech) real-time
   transaction-status, balance, account-state. FAPI-equivalent scope semantics.
7. **Marketplace → Seller**: marketplace grants seller real-time order-volume, listing-performance,
   buyer-cohort aggregate stats. Aggregate-mode (k≥5) by default.

Each use case lands as a worked example in `microservices/consent-graph/PRD.md` §6 with concrete Cedar
policy text, agreement-template JSON, and projection-scope manifest.

## 5. In-House Roadmap

100% in-house. No third-party data integrator, no third-party consent broker, no SaaS audit vendor in
the trust path. The substrate is:

- **Cedar** (already adopted, ADR-0183): policy engine.
- **Pulsar projection adapter** (consent-graph-specific substrate; broker-choice ADR assignment required before GA): cross-tenant projection topics; tenant-aware ACLs.
- **audit-chain µservice** (already shipped, ADR-0003): bilateral chain entries.
- **ontology µservice** (already shipped, ADR-0058): entity model + projection model (extended here).
- **OpenBao** (already adopted, ADR-0043): per-agreement secret material (HMAC keys for projection
  topic ACL).
- **Postgres + Citus** (already adopted, ADR-0034): agreement registry storage with RLS per tenant.

No external vendor dependency. This IS the moat.

## 6. Consequences

### 6.1 Positive
- Single Cedar enforcement point for every cross-tenant data flow → centralized audit + revocation.
- Zero-copy projection preserves sovereignty by construction.
- Bilateral chain entries make HIE-grade / Open-Banking-grade compliance achievable without per-vertical
  rebuild.
- Vertical-agnostic: same substrate serves supply chain, healthcare, finance, marketplace, B2C.
- Real-time (≤500ms projection p95) competitive with no existing solution.
- Revocable in ≤1s (no industry equivalent — Snowflake share revocation is minutes).

### 6.2 Negative / accepted
- New µservice surface increases ops load (mitigated by audit-chain-class SLO suite + 8 runbooks).
- Cedar policy authoring requires partner training (mitigated by 5 reusable agreement templates).
- Pulsar cross-region ACL is non-trivial (IP-002 of the Ontology projection extension).
- Projection lag means grantee-side eventual-consistency by design — must be documented in partner SDK.

### 6.3 Required follow-on work
- Ontology cross-tenant projection extension IPs: IP-CT-001..IP-CT-005 land in this PR under
  `microservices/ontology/IP-CT-*.md`.
- Workflow Studio cross-tenant trigger node (separate PR, depends on this).
- Capability-tier T3 promotion for `consent.grant` and `consent.project.subscribe` (gated by
  ADR-0123 hyperscaler maturity claim gate).

## 7. Verification

- `cargo build` clean for all 51 crates listed in `microservices/consent-graph/manifest.json`.
- Cedar policy unit-tests cover deny-by-default, revocation-takes-effect, scope-narrowing, purpose-of-use
  binding, geographic-constraint enforcement.
- Integration test: end-to-end grant → project → read → revoke → re-read-denied within 1.5s wall-clock.
- SLO manifests authored: `consent-grant-latency`, `cross-tenant-projection-freshness`,
  `revocation-propagation-latency`, `audit-chain-coverage-completeness`,
  `cedar-evaluation-latency`, `agreement-state-divergence-zero`,
  `sovereignty-violation-zero`, `bilateral-chain-link-integrity`, `partner-handshake-latency`
  (9 SLOs total, per ADR-0130 promotion gate).
- 8 runbooks authored: revocation-incident, partner-onboarding, consent-forgery-detected,
  audit-chain-divergence-recovery, regional-sovereignty-violation, GDPR-DSAR-cross-tenant,
  partner-offboarding, data-residency-enforcement.
- Threat model covers: cross-tenant data leakage, consent forgery, replay attack, revocation latency
  exploit, projection topic ACL bypass, sovereignty bypass, audit-chain divergence.

## 8. Sunset

This ADR has no sunset. Cross-tenant visibility is foundational to EaaS. Any change in substrate (e.g.,
swap Cedar for OPA, swap Pulsar for Kafka) requires a superseding ADR + minimum 6-month sunset window.
