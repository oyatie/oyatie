---
ip_id: IP-020
microservice: tenancy
bounded_context: data-residency-enforcement
layer: adapter
status: planned
related_adrs: [ADR-0244, ADR-0248, ADR-0251, ADR-0243]
---

# IP-020 — data-residency enforcer adapter

## A. Problem

The tenancy PRD makes `jurisdiction_code` immutable and forbids cross-pack movement by default, but callers still need an adapter-level enforcement point for outbound events, DR promotion, DSR fan-out, and cross-region reads. If tenancy emits an event without residency metadata, downstream µservices cannot distinguish legitimate same-pack processing from a cross-border leak.

## B. Approach

Build `oya-tenancy-data-residency-enforcer-adapter` as an adapter crate that wraps outbound event and RPC ports. It injects residency metadata, evaluates `policy/data-residency.cedar`, blocks disallowed routes, and emits a denial audit event instead of relying on every downstream service to re-derive the rule.

## C. Deliverables

| Artifact | Action | Purpose |
|---|---|---|
| `microservices/tenancy/src/crates/oya-tenancy-data-residency-enforcer-adapter/Cargo.toml` | create | Adapter crate. |
| `src/envelope.rs` | create | Adds tenant home jurisdiction and pack labels to outbound envelopes. |
| `src/cedar_eval.rs` | create | Library-first Cedar evaluator wrapper for `data-residency.cedar`. |
| `src/outbound_guard.rs` | create | Guard for event and RPC dispatch. |
| `src/errors.rs` | create | `ResidencyViolationBlocked`, `MissingResidencyMetadata`, `PolicyUnavailable`. |
| `microservices/tenancy/policy/data-residency.cedar` | extend if needed | Entity/action names used by adapter. |
| `microservices/tenancy/catalog/oya-tenancy-data-residency-enforcer-adapter.yaml` | create | Catalog row. |

## D. Implementation

1. Define `ResidencyEnvelope` with `tenant_id`, `home_jurisdiction`, `source_cell`, `target_cell`, `source_pack`, `target_pack`, `action`, and `audit_correlation_id`.
2. Add `guard_outbound_event(envelope, next)` and `guard_rpc_call(envelope, next)` wrappers.
3. Load `policy/data-residency.cedar` through the service's policy port and fail closed for mutation traffic if policy is unavailable.
4. Permit same-pack routes, explicit legal-transfer register routes, and DSR receipt aggregation where policy allows it.
5. Block cross-pack calls by default and emit `oya.tenancy.residency-violation-blocked`.
6. Add tests for same-pack allow, KR-to-US deny, US-HC-to-US deny without BAA/legal basis, DSR allowed route, and missing metadata failure.
7. Add a contract note in `contracts/asyncapi/tenant-events.yaml` requiring residency labels on tenancy-emitted events.

## E. Acceptance

- `cargo nextest run -p oya-tenancy-data-residency-enforcer-adapter --all-features`.
- Cedar decision tests use real `microservices/tenancy/policy/data-residency.cedar`.
- Every denial carries audit evidence and no outbound event/RPC is dispatched.
- `rg "home_jurisdiction|source_pack|target_pack" microservices/tenancy/contracts` finds the event contract labels after the implementation.
- `cargo run -p oya-dev-cli -- gate validate tenancy-residency-conformance --microservice tenancy`.

## F. Evidence

- `microservices/tenancy/PRD.md` data-residency section states cross-pack replication is forbidden by default.
- `microservices/tenancy/policy/data-residency.cedar` is the real policy file.
- `microservices/tenancy/contracts/asyncapi/tenant-events.yaml` is the outbound event surface.
- `microservices/tenancy/IP-019-dr-pairing-controller.md` depends on this guard for same-jurisdiction DR.

## G. Counterparts

| Counterpart | Relevant capability | Gap this IP closes |
|---|---|---|
| Salesforce Hyperforce | Region-aware data residency commitments | Converts residency from prose into outbound event/RPC enforcement. |
| Microsoft Entra | Tenant-region and cross-cloud policy boundaries | Keeps identity/tenant context region labels attached to control events. |
| Stripe | Region- and account-scoped payment data constraints | Prevents payments-facing tenant state from crossing pack boundaries. |

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/tenancy/IP-020-data-residency-enforcer-adapter.md` matched `asyncapi`; contract files `microservices/tenancy/contracts/openapi/tenancy.yaml, microservices/tenancy/contracts/asyncapi/tenant-events.yaml, microservices/tenancy/contracts/proto/tenancy.proto`; type anchor `crates/oya-tenancy-api/src/lib.rs::TenantCreateApiRequest`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/tenancy/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/tenancy/IP-020-data-residency-enforcer-adapter.md` matched `payment`; anchors `microservices/tenancy/runbooks/dr-pair-promotion-drill.md, crates/oya-tenancy-api/src/lib.rs`; type anchor `crates/oya-tenancy-api/src/lib.rs::TenantCreateApiRequest`.
