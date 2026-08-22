---
ip_id: IP-018
microservice: tenancy
bounded_context: kyb-kyc
layer: domain
status: planned
related_adrs: [ADR-0244, ADR-0250, ADR-0292, ADR-0263]
---

# IP-018 — KYB-KYC verifier domain

## A. Problem

`tenancy` can model tenant lifecycle states, but business-tier activation still lacks a domain model for "this tenant is allowed to become active." The PRD makes tenant activation a load-bearing control because every downstream µservice trusts `TenantContext`; payments additionally depends on verified tenant and sub-merchant status before money movement. Without a KYB/KYC domain, activation can race ahead of sanctions, UBO, minor-protection, or business-registration evidence.

## B. Approach

Create `tenancy-kyb-kyc-verifier-domain` as a pure domain crate that models verification cases, document requirements, screening results, decision state, expiry, and escalation. Provider calls remain outside this IP; this domain receives provider-independent facts and decides whether the tenant lifecycle may advance from `Created` to `Activated`.

## C. Deliverables

| Artifact | Action | Purpose |
|---|---|---|
| `microservices/tenancy/src/crates/tenancy-kyb-kyc-verifier-domain/Cargo.toml` | create | Domain crate. |
| `src/case.rs` | create | `VerificationCase`, `CaseStatus`, `Decision`. |
| `src/document.rs` | create | `DocumentRequirement`, `DocumentSubmission`, `DocumentStatus`. |
| `src/screening.rs` | create | Sanctions, PEP, adverse-media, minor-protection screening results. |
| `src/rules.rs` | create | Country, tenant_class, and audience-specific requirement resolver. |
| `src/events.rs` | create | Domain events for completed, declined, escalated, expired. |
| `microservices/tenancy/catalog/tenancy-kyb-kyc-verifier-domain.yaml` | update/create | Catalog row already present in the service inventory. |
| `microservices/tenancy/capabilities/kyb-kyc-complete.yaml` | align | Capability declares this domain as decision owner. |

## D. Implementation

1. Define `VerificationCase` keyed by `tenant_id`, `case_id`, `country`, `tenant_class`, `business_type`, and `requested_capability`.
2. Implement `required_documents(country, tenant_class, requested_capability)` with B2B KYB documents, B2C KYC age checks, UBO evidence, and KR-PASS where `jurisdiction_code = KR`.
3. Add `record_document_submission` and `record_screening_result` as pure mutators that append events and never call a provider.
4. Implement `decide()` so `Approve` requires all required documents verified, sanctions clear, PEP/adverse-media below escalation threshold, and minor-protection rules satisfied.
5. Implement decline and escalation reasons: `SanctionsHit`, `MissingUbo`, `MinorRefused`, `DocumentRejected`, `ProviderTimeout`, `ManualReviewRequired`.
6. Add expiry rules so stale KYB evidence cannot activate a tenant after a configured window.
7. Add tests for KR business signup, EU UBO requirement, US healthcare BAA prerequisite, COPPA under-13 refusal, sanctions escalation, and provider timeout keeping the tenant pending.

## E. Acceptance

- `cargo nextest run -p tenancy-kyb-kyc-verifier-domain --all-features`.
- No network, database, HTTP, or provider SDK dependencies appear in the crate.
- Tests show `TenantStatus::Activated` eligibility only after `Decision::Approve`.
- Domain events map to `oya.tenancy.kyb-kyc-completed`, `oya.tenancy.kyb-kyc-declined`, and `oya.tenancy.kyb-kyc-escalated` for `IP-024`.
- `microservices/tenancy/capabilities/kyb-kyc-complete.yaml` names this crate as the domain decision source.

## F. Evidence

- `microservices/tenancy/PRD.md` requires sub-5-minute self-serve activation and identifies tenant lifecycle as the authority every µservice trusts.
- `microservices/tenancy/manifest.json` lists `kyb-kyc-complete.yaml` and the `tenancy-kyb-kyc-verifier-domain` catalog row.
- `microservices/tenancy/runbooks/kyb-kyc-pipeline-stalled.md` already exists as the operational path for stuck cases.
- `microservices/payments/competitor-parity-matrix.md` shows Stripe Identity/pressure on verified account onboarding; tenancy must supply the tenant-side proof before payments handles sub-merchants.

## G. Counterparts

| Counterpart | Relevant capability | Gap this IP closes |
|---|---|---|
| Stripe Identity / Stripe | Account identity and business verification before money movement | Gives Oyatie a tenant-side approval decision before payments creates PSP sub-merchant state. |
| WorkOS | Enterprise organization onboarding evidence | Adds B2B tenant verification before organization activation. |
| Auth0 Organizations | Organization lifecycle metadata | Adds compliance-grade KYB/KYC state beyond identity-provider org metadata. |

## DR posture (per ADR-0343)
- Manifest target source: `microservices/tenancy/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/tenancy/IP-018-kyb-kyc-verifier-domain.md` matched `payment`; anchors `microservices/tenancy/runbooks/dr-pair-promotion-drill.md, crates/tenancy-api/src/lib.rs`; type anchor `crates/tenancy-api/src/lib.rs::TenantCreateApiRequest`.
