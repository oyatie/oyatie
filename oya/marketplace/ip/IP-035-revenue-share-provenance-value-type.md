---
doc_class: ImplementationPlan
microservice: marketplace
status: Accepted
date: 2026-05-21
owner_team: axis-marketplace
primary_adr: ADR-0329
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0249, ADR-0263, ADR-0314, ADR-0329, ADR-0330, ADR-0331]
companion_docs: [microservices/marketplace/REMEDIATION-NOTES-2026-05-21.md]
planned_enforcement_ref: oya-governance-marketplace-doc-set
naming_justifications: BNF v4 service_action_resource grammar and 13-layer-enum conformance are declared inline in this document
impl_plan_id: IP-035-revenue-share-provenance-value-type
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
---

# IP-035: Structured RevenueShareProvenance Value-Type

## Intent
Promote the audit_chain_ref from opaque string to a structured `RevenueShareProvenance` value-type carrying all evidence fields required for downstream provability. Closes audit gap §3.4.B.ii item 5.

## Boundary
- Owns: `RevenueShareProvenance` kernel value-type, its serialization, its BLAKE3 chain-position contract.
- Consumes: audit-chain.seal for chain position.
- Does not own: audit-chain's underlying merkle structure.

## Deliverables
1. Kernel value-type `RevenueShareProvenance` (Rust struct + proto3 message):
   ```rust
   pub struct RevenueShareProvenance {
       pub deal_set_id: DealSetId,
       pub listing_id: ListingId,
       pub buyer_tenant_id: TenantId,
       pub seller_tenant_id: TenantId,
       pub gross_amount_minor_units: i128,
       pub currency: CurrencyCode,
       pub oyatie_share_basis_points: u16,           // 0..10000
       pub oyatie_share_amount_minor_units: i128,
       pub blake3_chain_position: ChainPosition,
       pub contract_terms_hash: Blake3Digest,
       pub revenue_share_cohort_id: RevenueShareCohortId,
       pub fx_snapshot_ref: Option<FxSnapshotRef>,
       pub category: ListingCategory,                // plugins|apps|workflows|agents|models|datasets
   }
   ```
2. Proto3 message `marketplace.RevenueShareProvenance` for cross-service consumption.
3. Domain invariant: `gross_amount_minor_units * oyatie_share_basis_points / 10000 == oyatie_share_amount_minor_units` (within rounding rule).
4. Serialization: deterministic BLAKE3-friendly canonical bytes order.
5. All revenue-share emit sites updated to populate this value-type.
6. Backward-compatible facade: legacy `audit_chain_ref` string derived from `provenance.blake3_chain_position.canonical_string()`.

## Acceptance criteria
- Property test on rounding rule (1M random inputs).
- Cross-service serialization round-trip (Rust ↔ proto3 ↔ JSON) byte-identical.
- Audit-chain seal of the structured provenance recovers to a deterministic BLAKE3 root.
- Backward-compat: legacy string consumers see the canonical form.

## Naming justifications
- BNF v4: `marketplace.kernel.revenue-share-provenance`
- Layer enum: kernel + domain + observability
- Crate name: `oya-marketplace-kernel-revenue-share-provenance`
