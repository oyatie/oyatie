---
ip_id: IP-023
microservice: tenancy
bounded_context: sub-scope-registry
layer: adapter
status: planned
related_adrs: [ADR-0244, ADR-0105, ADR-0131]
---

# IP-023 — sub-scope-registry Postgres adapter

## A. Problem

`IP-016` defines sub-scope hierarchy invariants, but those invariants need a durable storage adapter that preserves tenant isolation under Citus and Postgres RLS. A simple adjacency table is not enough because authorization and projection paths need fast descendant checks for shared workspace, care-team, borrower-bank-counterparty, and engagement scopes.

## B. Approach

Create `oya-tenancy-sub-scope-registry-adapter-postgres` implementing the kernel ports against Citus-distributed Postgres. Store `sub_scopes` and `sub_scope_hierarchy_closure`, distribute by `tenant_id`, enforce RLS, and maintain closure rows transactionally whenever a parent-child edge changes.

## C. Deliverables

| Artifact | Action | Purpose |
|---|---|---|
| `microservices/tenancy/src/crates/oya-tenancy-sub-scope-registry-adapter-postgres/Cargo.toml` | create | Adapter crate. |
| `src/repository.rs` | create | Implements `SubScopeRegistryPort`. |
| `src/closure_table.rs` | create | Transactional closure-table maintenance. |
| `src/migrations/001_sub_scopes.sql` | create | Tables, indexes, Citus distribution, RLS. |
| `src/migrations/002_sub_scope_closure.sql` | create | Closure table and constraints. |
| `microservices/tenancy/catalog/oya-tenancy-sub-scope-registry-adapter-postgres.yaml` | create | Catalog row. |

## D. Implementation

1. Create `sub_scopes(tenant_id, sub_scope_id, kind, slug, parent_id, created_at, archived_at)` with composite primary key `(tenant_id, sub_scope_id)`.
2. Create `sub_scope_hierarchy_closure(tenant_id, ancestor_id, descendant_id, depth)` with indexes for ancestor and descendant lookups.
3. Enable and force RLS on both tables with `tenant_id = current_setting('app.current_tenant_id')`.
4. Call `checkout_tenant_scoped` from the existing tenancy adapter pattern before any query.
5. On insert or move, call the IP-016 kernel validator first, then update closure rows in one transaction.
6. Reject cross-tenant parent ids before SQL execution and assert the database constraint also rejects them.
7. Add integration tests with a real Postgres/Citus-compatible test container for closure rows, RLS isolation, archive behavior, and duplicate slug per parent.

## E. Acceptance

- `cargo nextest run -p oya-tenancy-sub-scope-registry-adapter-postgres --all-features`.
- `cargo run -p oya-dev-cli -- gate validate tenant-context-setlocal-present`.
- `cargo run -p oya-dev-cli -- gate validate rls-force-on-tenant-tables`.
- Tests verify tenant A cannot read tenant B closure rows even with matching sub-scope ids.
- Closure-table reads support ancestor and descendant lookup without recursive runtime SQL.

## F. Evidence

- `tenancy/IP-016-sub-scope-registry-kernel.md` owns the pure hierarchy rules consumed here.
- `tenancy/policy/rls-isolation.md` describes the RLS pattern.
- `tenancy/IP-005-tenant-lifecycle-adapter-postgres.md` gives the existing `SET LOCAL app.current_tenant_id` adapter pattern.
- `tenancy/competitor-parity-matrix.md` positions Citus as the multi-tenant Postgres substrate.

## G. Counterparts

| Counterpart | Relevant capability | Gap this IP closes |
|---|---|---|
| Citus | Tenant-distributed Postgres | Persists sub-scope hierarchy with the same tenant shard key as lifecycle data. |
| Slack Enterprise Grid | Workspace membership hierarchy | Makes workspace/project lookups efficient and tenant isolated. |
| Stripe | Connected account hierarchy | Gives payments and marketplace flows durable child-scope lookup under a tenant. |

## DR posture (per ADR-0343)
- Manifest target source: `tenancy/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `tenancy/IP-023-sub-scope-registry-adapter-postgres.md` matched `payment`; anchors `tenancy/runbooks/dr-pair-promotion-drill.md, crates/oya-tenancy-api/src/lib.rs`; type anchor `crates/oya-tenancy-api/src/lib.rs::TenantCreateApiRequest`.
