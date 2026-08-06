---
ip_id: IP-025
microservice: tenancy
bounded_context: dr-pairing
layer: adapter
status: planned
related_adrs: [ADR-0263, ADR-0252, ADR-0244]
---

# IP-025 — DR-pairing AsyncAPI emitter

## A. Problem

`IP-019` can decide to promote or restore a tenant DR pair, but that decision is operationally useless unless observability, ops-dashboard, audit-chain, and downstream tenant-context caches receive the event with ordering and idempotency guarantees. A generic event emitter is insufficient because DR promotion is one of the few tenancy actions where stale or duplicate events can split tenant traffic.

## B. Approach

Create `oya-tenancy-dr-pairing-async-emitter` as an adapter around `contracts/asyncapi/tenant-events.yaml`. It emits signed, idempotent promotion/restoration events with pair version, TrueTime/HLC ordering fields, home/DR cell ids, residency labels, and audit-chain correlation.

## C. Deliverables

| Artifact | Action | Purpose |
|---|---|---|
| `microservices/tenancy/src/crates/oya-tenancy-dr-pairing-async-emitter/Cargo.toml` | create | Adapter crate. |
| `src/emitter.rs` | create | Emits promotion/restoration envelopes. |
| `src/signing.rs` | create | HMAC/Ed25519 envelope signing abstraction. |
| `src/idempotency.rs` | create | Idempotency-key derivation and duplicate suppression. |
| `microservices/tenancy/contracts/asyncapi/tenant-events.yaml` | update | Add DR pairing channels and payload schemas. |
| `microservices/tenancy/capabilities/dr-pair-promote.yaml` | align | Capability event evidence. |

## D. Implementation

1. Add channels `oya.tenancy.dr-pairing-promoted.v1` and `oya.tenancy.dr-pairing-restored.v1`.
2. Define payload fields: `tenant_id`, `pair_version`, `home_cell_id`, `dr_cell_id`, `active_cell_id`, `jurisdiction_code`, `pack`, `promoted_at`, `ordering_token`, `idempotency_key`, and `audit_correlation_id`.
3. Derive idempotency key from `(tenant_id, pair_version, event_kind)` so replay during retries is safe.
4. Sign the envelope before dispatch and verify tests reject tampered payloads.
5. Require `IP-020` residency envelope fields before emission.
6. Emit to observability, ops-dashboard, audit-chain, and tenant-context cache invalidation topics.
7. Add tests for duplicate retry, out-of-order pair version, signature failure, and missing residency metadata.

## E. Acceptance

- `cargo nextest run -p oya-tenancy-dr-pairing-async-emitter --all-features`.
- AsyncAPI validates with both DR channels.
- Duplicate event with same idempotency key is a no-op.
- Event with older pair version is rejected and emits diagnostic evidence.
- `runbooks/dr-pair-promotion-drill.md` names the channels as verification points.

## F. Evidence

- `microservices/tenancy/IP-019-dr-pairing-controller.md` owns the promotion decision.
- `microservices/tenancy/contracts/asyncapi/tenant-events.yaml` is the existing event surface.
- `microservices/tenancy/dashboards/dr-pairing-state.json` is the dashboard consumer.
- `microservices/tenancy/policy/data-residency.cedar` governs pack/jurisdiction metadata.

## G. Counterparts

| Counterpart | Relevant capability | Gap this IP closes |
|---|---|---|
| AWS EventBridge | Signed/typed operational events | Publishes DR state transitions as contract-backed events. |
| Databricks Unity Catalog | Audit-forwarded governance events | Keeps DR promotion visible to governance/audit consumers. |
| Stripe | Webhook idempotency and signature discipline | Applies signed idempotent webhook-style delivery to tenancy DR events. |

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/tenancy/IP-025-dr-pairing-async-emit.md` matched `asyncapi`; contract files `microservices/tenancy/contracts/openapi/tenancy.yaml, microservices/tenancy/contracts/asyncapi/tenant-events.yaml, microservices/tenancy/contracts/proto/tenancy.proto`; type anchor `crates/oya-tenancy-api/src/lib.rs::TenantCreateApiRequest`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/tenancy/IP-025-dr-pairing-async-emit.md` matched `emission`; anchors `microservices/tenancy/manifest.json, crates/oya-tenancy-api/src/lib.rs`; type anchor `crates/oya-tenancy-api/src/lib.rs::TenantCreateApiRequest`.
