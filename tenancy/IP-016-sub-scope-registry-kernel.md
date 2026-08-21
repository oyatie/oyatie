---
ip_id: IP-016
microservice: tenancy
bounded_context: sub-scope-registry
layer: kernel
status: in-progress
related_adrs: [ADR-0244, ADR-0083, ADR-0105, ADR-0131]
---

# IP-016 — sub-scope-registry kernel

> **Delivery note (2026-08-20).** Implemented in tenancy/core/sub-scope-registry as `tenancy-sub-scope-registry`, collapsed into that ONE crate
> as a module tree rather than this plan's multi-crate fan-out: the capability is capped at 12 crates
> and `Cargo.lock` is a hub path owned by `integ/build`, so neither a new crate nor a new dependency
> was available to this lane. Landed: the scope hierarchy with every error variant reachable and every read path tenant-scoped. Deferred and named as a gap in the crate's `lib.rs` header:
> the Postgres adapter, which is IP-023. The crate names in the tables below are this plan's original
> proposal, not what shipped.


## A. Problem

`tenancy` currently owns the tenant lifecycle and the tenant-to-cell assignment, but it does not have a typed kernel for scopes below the tenant. That gap forces downstream products to improvise workspace, project, department, engagement, and helper-scope identifiers outside the canonical `TenantContext` model described in `tenancy/PRD.md`. The missing kernel becomes dangerous once `IP-journey-j116-tenant-install-boundary.md`, `IP-journey-j118-projection-scope-registry.md`, and `IP-journey-j123-shared-workspace-scope.md` need stable parent-child scope semantics.

This IP creates the zero-I/O substrate for sub-scopes. It is not a UI, not an adapter, and not a new tenant lifecycle service. It supplies the pure types and invariants that later REST, Postgres, Cedar, and projection slices can reuse without re-creating hierarchy rules.

## B. Approach

Author `oya-tenancy-sub-scope-registry-kernel` as a pure Rust crate under the tenancy flat-layout target. The kernel defines `SubScopeId`, `SubScopeKind`, `SubScope`, `SubScopeParent`, `SubScopePath`, `HierarchyEdge`, and `SubScopeRegistryPort`. It enforces cycle refusal, maximum depth, tenant boundary preservation, immutable root scope, and namespace normalization before persistence exists.

The hierarchy model is an adjacency edge plus materialized path projection: the kernel validates the edge set and produces a canonical path representation for the future Postgres closure-table adapter in `IP-023-sub-scope-registry-adapter-postgres.md`.

## C. Deliverables

| Artifact | Action | Purpose |
|---|---|---|
| `microservices/tenancy/src/crates/oya-tenancy-sub-scope-registry-kernel/Cargo.toml` | create | Register pure kernel dependencies only. |
| `microservices/tenancy/src/crates/oya-tenancy-sub-scope-registry-kernel/src/lib.rs` | create | Export public types and deny unsafe code. |
| `microservices/tenancy/src/crates/oya-tenancy-sub-scope-registry-kernel/src/entities.rs` | create | `SubScope`, `SubScopeId`, `SubScopeKind`, `SubScopePath`. |
| `microservices/tenancy/src/crates/oya-tenancy-sub-scope-registry-kernel/src/hierarchy.rs` | create | Cycle, depth, parent-kind, and tenant-boundary validators. |
| `microservices/tenancy/src/crates/oya-tenancy-sub-scope-registry-kernel/src/ports.rs` | create | `SubScopeRegistryPort` and `SubScopeHierarchyReadPort`. |
| `microservices/tenancy/src/crates/oya-tenancy-sub-scope-registry-kernel/src/errors.rs` | create | `SubScopeKernelError` with no `anyhow`. |
| `tenancy/catalog/oya-tenancy-sub-scope-registry-kernel.yaml` | update/create | Catalog row already listed in `manifest.json` and must match crate path. |

## D. Implementation

1. Add the crate with only serialization, UUID/ULID, and internal shared tenant-type dependencies already allowed by existing tenancy IPs.
2. Define `SubScopeKind` as a closed enum: `Workspace`, `Project`, `OrgUnit`, `Engagement`, `HelperScope`, `CareTeam`, `ProviderPanel`, `AuditRoom`.
3. Define `SubScopeId` as a tenant-local opaque id and require every `SubScope` to carry `tenant_id` from the tenancy kernel, never a raw string.
4. Implement `validate_new_edge(parent, child, ancestors)` so a child cannot cross tenant id, cannot exceed depth 6, cannot make a cycle, and cannot attach an `OrgUnit` under an `Engagement`.
5. Implement `canonical_path(root, edges)` returning a stable `SubScopePath` for the future closure-table adapter.
6. Add property tests for acyclic trees, random insertion order, duplicate edge rejection, and tenant-boundary refusal.
7. Add doc tests showing how `IP-023` will persist `sub_scopes` and `sub_scope_hierarchy_closure` without duplicating kernel validation.

## E. Acceptance

- `cargo check -p oya-tenancy-sub-scope-registry-kernel --all-features`.
- `cargo nextest run -p oya-tenancy-sub-scope-registry-kernel --all-features`.
- Property tests prove no cycles, max depth 6, stable path ordering, and same-tenant-only parentage.
- `cargo run -p oya-dev-cli -- gate validate layer-correctness --crate oya-tenancy-sub-scope-registry-kernel`.
- `cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice tenancy`.

## F. Evidence

- `tenancy/PRD.md` requires `TenantContext` as the only valid tenant representation and calls tenancy the single authority for every tenant decision.
- `tenancy/manifest.json` already lists `tenancy/catalog/oya-tenancy-sub-scope-registry-kernel.yaml`.
- `tenancy/IP-023-sub-scope-registry-adapter-postgres.md` depends on this kernel for `sub_scopes` and closure-table persistence.
- `tenancy/policy/tenant-scope.cedar` and `tenancy/policy/action-authorization.cedar` are the future policy consumers for sub-scope permissions.

## G. Counterparts

| Counterpart | Relevant capability | Gap this IP closes |
|---|---|---|
| Stripe | Connected accounts under a platform account | Provides a tenant-local child-scope model before payment/account scopes reuse tenancy ids. |
| Slack Enterprise Grid | Workspace hierarchy inside an enterprise org | Adds first-class workspace/project/org-unit hierarchy instead of ad hoc downstream strings. |
| AWS Organizations | OU tree with parent-child constraints | Brings cycle/depth validation into the kernel before the Postgres adapter persists closure rows. |

## DR posture (per ADR-0343)
- Manifest target source: `tenancy/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `tenancy/IP-016-sub-scope-registry-kernel.md` matched `payment`; anchors `tenancy/runbooks/dr-pair-promotion-drill.md, crates/oya-tenancy-api/src/lib.rs`; type anchor `crates/oya-tenancy-api/src/lib.rs::TenantCreateApiRequest`.
