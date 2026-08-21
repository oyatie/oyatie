---
ip_id: IP-021
microservice: tenancy
bounded_context: lifecycle-locks
layer: kernel
status: in-progress
related_adrs: [ADR-0244, ADR-0276, ADR-0263]
---

# IP-021 — lifecycle-locks kernel

> **Delivery note (2026-08-20).** Implemented in tenancy/core/lifecycle-locks as `tenancy-lifecycle-locks`, collapsed into that ONE crate
> as a module tree rather than this plan's multi-crate fan-out: the capability is capped at 12 crates
> and `Cargo.lock` is a hub path owned by `integ/build`, so neither a new crate nor a new dependency
> was available to this lane. Landed: the reason-by-action precedence matrix, expiry, lease and holder-bound release authorization. Deferred and named as a gap in the crate's `lib.rs` header:
> durable lock persistence; the store here is in-memory behind the port. The crate names in the tables below are this plan's original
> proposal, not what shipped.


## A. Problem

Tenant lifecycle plans define create, suspend, resume, delete, and DSR transitions, but they do not model holds that intentionally block an otherwise valid transition. This is a material gap for legal hold, active incident containment, payment dispute retention, regulator investigation, and jurisdiction-change freezes. Without a kernel lock model, each usecase can invent its own "do not delete yet" flag and silently diverge from DSR and audit-chain expectations.

## B. Approach

Create `oya-tenancy-lifecycle-locks-kernel` as pure logic for lock creation, precedence, release authorization, expiry, and decision explanation. Usecases consume it before `RequestTenantDeletion`, jurisdiction migration, payment method removal, KYB/KYC re-verification, and DR promotion.

## C. Deliverables

| Artifact | Action | Purpose |
|---|---|---|
| `microservices/tenancy/src/crates/oya-tenancy-lifecycle-locks-kernel/Cargo.toml` | create | Pure kernel crate. |
| `src/lock.rs` | create | `LifecycleLock`, `LockKind`, `LockScope`, `LockState`. |
| `src/precedence.rs` | create | Delete-lock, legal-hold, incident-hold, soft-lock precedence. |
| `src/release.rs` | create | Multi-party release decision model. |
| `src/errors.rs` | create | `LifecycleLockError`. |
| `tenancy/catalog/oya-tenancy-lifecycle-locks-kernel.yaml` | update/create | Catalog evidence. |

## D. Implementation

1. Define `LockKind`: `LegalHold`, `DsrGrace`, `IncidentContainment`, `RegulatorInvestigation`, `PaymentDisputeRetention`, `JurisdictionFreeze`, `ManualSoftLock`.
2. Define `BlockedTransition`: `DeleteTenant`, `ChangeJurisdiction`, `RemovePaymentCredential`, `PromoteDrPair`, `DeactivateKyb`.
3. Implement `effective_decision(locks, transition)` returning allow/deny plus the highest-precedence lock and audit explanation.
4. Implement release rules: legal hold requires DPO plus counsel, incident containment requires ops-security, payment dispute retention requires finance/compliance, soft lock can be released by tenant admin.
5. Preserve locks across DR pair promotion by making the kernel state independent of cell location.
6. Add property tests for precedence ordering and lock survival through lifecycle state transitions.
7. Add examples for `IP-009-dsr-cascade-runner.md` showing DSR request accepted but deletion delayed with statutory-retention basis.

## E. Acceptance

- `cargo nextest run -p oya-tenancy-lifecycle-locks-kernel --all-features`.
- Tests cover delete-lock over suspend-lock, legal-hold release quorum, expired soft-lock, and DR promotion lock survival.
- Kernel has no I/O dependencies and no policy-engine dependency.
- Domain events are specified for `oya.tenancy.lifecycle-lock-applied`, `oya.tenancy.lifecycle-lock-release-requested`, and `oya.tenancy.lifecycle-lock-released`.

## F. Evidence

- `tenancy/PRD.md` DSR cascade and compliance sections require proof-of-erasure while preserving lawful residual data bases.
- `tenancy/contracts/openapi/tenancy.yaml` `DsrRequest` schema already models `residual_data_basis`.
- `tenancy/runbooks/tenant-isolation-breach-response.md` and `runbooks/cross-tenant-data-leak-containment.md` need lock semantics during incidents.

## G. Counterparts

| Counterpart | Relevant capability | Gap this IP closes |
|---|---|---|
| AWS S3 Object Lock | Governance/compliance retention lock | Brings explicit irreversible-operation locks to tenant lifecycle transitions. |
| Cloudflare zone lock | Human-gated protection for critical account changes | Prevents accidental tenant deletion or jurisdiction changes during incidents. |
| Stripe | Dispute and chargeback retention obligations | Provides a tenant-level hold model for payment-related retention before deletion. |

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `tenancy/IP-021-lifecycle-locks-kernel.md` matched `openapi`; contract files `tenancy/contracts/openapi/tenancy.yaml, tenancy/contracts/asyncapi/tenant-events.yaml, tenancy/contracts/proto/tenancy.proto`; type anchor `crates/oya-tenancy-api/src/lib.rs::TenantCreateApiRequest`.

## DR posture (per ADR-0343)
- Manifest target source: `tenancy/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `tenancy/IP-021-lifecycle-locks-kernel.md` matched `payment`; anchors `tenancy/runbooks/dr-pair-promotion-drill.md, crates/oya-tenancy-api/src/lib.rs`; type anchor `crates/oya-tenancy-api/src/lib.rs::TenantCreateApiRequest`.
