# IP-010: `oya-api-gateway-rate-limit-adapter-valkey` crate

**Status:** design-ready
**Owner:** axis-network

## A — Scope

Valkey Cluster-backed adapter for rate-limit bucket persistence.

## B — Acceptance criteria

- Per-cell Valkey Cluster.
- Shuffle-shard via key salting.
- Sub-1ms p99 GET/INCR.
- Pipeline batching.

## Wave 15 A-G substance

### A - Problem
The pure rate-limit domain can calculate bucket outcomes, but the gateway needs a per-cell persistence adapter that keeps hot-path quota checks deterministic under burst, retry, and failover pressure.

### B - Approach
Implement `oya-api-gateway-rate-limit-adapter-valkey` from `catalog/oya-api-gateway-rate-limit-adapter-valkey.yaml` as the Valkey-backed implementation of rate-limit persistence ports. The adapter owns key shape, salting, pipeline execution, Lua/script registration, fallback behavior, and metrics; it does not own quota math.

### C - Deliverables
- Per-cell Valkey Cluster client with explicit tenant/cell/route bucket key encoding.
- Shuffle-sharded key salting so one abusive tenant cannot hot-spot a single shard.
- Atomic consume/refill script wrapper returning remaining, reset, retry-after, and decision metadata.
- Idempotency-aware retry path that can recognize duplicate write attempts without double-consuming a fresh quota token.
- Circuit-open fallback mode that fails closed for high-risk writes and degrades to local emergency buckets for low-risk reads.
- Metrics for lookup latency, script error, shard saturation, fallback decision, and bucket-cardinality drift.

### D - Ordered implementation steps
1. Define adapter port DTOs using `oya-api-gateway-rate-limit-domain` values.
2. Encode key format as `cell:{cell_id}:tenant:{tenant_id}:route:{route_id}:class:{limit_class}:salt:{n}`.
3. Add script registration and SHA pinning with startup validation.
4. Implement pipelined consume for route, tenant, fingerprint, and source-IP buckets.
5. Add idempotency-key lookup for write routes before consuming write tokens.
6. Add fallback policy for Valkey timeout, shard unavailable, and script mismatch.
7. Add integration fixtures for saturation, failover, resharding, and duplicate retries.

### E - Acceptance gates
- `cargo test -p oya-api-gateway-rate-limit-adapter-valkey --features fixtures` passes.
- Adapter tests prove atomic consume/refill, duplicate idempotency handling, per-cell key isolation, and shard-salt distribution.
- Timeout tests show high-risk writes deny and low-risk reads use bounded local emergency buckets.
- Metrics align with `dashboards/rate-limit-hits.json` and `contracts/metric-naming-convention.md`.
- No quota arithmetic is duplicated outside the domain crate.

### F - Evidence
Grounding files: `catalog/oya-api-gateway-rate-limit-adapter-valkey.yaml`, `policy/rate-limit.cedar`, `runbooks/rate-limit-saturation.md`, `dashboards/rate-limit-hits.json`, `capacity-model.md`, `performance-benchmark-numbers-2026-05-20.md`, and `contracts/api-gateway.openapi.yaml`.

### G - Counterpart comparison
Stripe API idempotency/rate-limit pressure is the concrete counterpart. Stripe-style APIs must tolerate repeated client retries, preserve idempotency semantics, and still enforce tenant/route buckets; this adapter mirrors that pressure while adding per-cell Valkey isolation and Cedar-derived route classes.

## Remediation notes

- Rewrote the thin Valkey stub into a service-specific adapter plan with key format, idempotency, fallback, metrics, and fixture gates.
- Keep Valkey naming precise in implementation follow-up; the IP now uses Valkey vocabulary while preserving the RESP protocol surface.
- Future remediation should add a manifest entry for IP-017 and IP-018 if those IPs remain promoted alongside IP-010..016.

## File-specific validation matrix

| Check | Local artifact | Expected evidence |
| --- | --- | --- |
| Key isolation | `manifest.json` | Cell and tenant identifiers are part of every persisted bucket key. |
| Valkey script | `policy/rate-limit.cedar` | Script inputs correspond to Cedar-derived route and tenant classes. |
| Idempotency | `contracts/api-gateway.openapi.yaml` | Idempotency key can be bound to write-route quota decisions. |
| Stripe-style retries | Stripe API idempotency/rate-limit pressure | Duplicate write retry does not double-consume fresh quota. |
| Metrics | `dashboards/rate-limit-hits.json` | Hit, exceed, fallback, shard, and latency series are observable. |
| Saturation runbook | `runbooks/rate-limit-saturation.md` | Valkey timeout and shard saturation map to operator actions. |
| Capacity | `capacity-model.md` | Lookup throughput supports the documented per-cell target. |
| Fallback | `failure-modes.md` | High-risk write deny and emergency read bucket modes are distinct. |
| Domain separation | `IP-009-rate-limit-domain-crate.md` | Adapter persists decisions without duplicating bucket arithmetic. |
| Audit | `contracts/api-gateway.asyncapi.yaml` | Exceeded and fallback decisions produce audit evidence. |
| Resharding | `multi-region.md` | Per-cell key design survives cell failover and shard movement. |
| Feature flags | `catalog/oya-api-gateway-rate-limit-adapter-valkey.yaml` | Live cluster behavior is opt-in for tests. |

## Remediation follow-up checklist

- Add a Stripe-style idempotent write retry fixture.
- Add per-cell key collision and salt-distribution fixtures.
- Add Valkey timeout, script mismatch, and shard-saturation fixtures.
- Add fallback tests for high-risk write deny and low-risk read emergency bucket.
- Add dashboard label checks for hit, exceed, fallback, and shard saturation.

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/api-gateway/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/IP-010-rate-limit-adapter-valkey.md`.

## DR posture (per ADR-0343)

- Target source: `microservices/api-gateway/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `900` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`object_storage_versioned`, `audit_chain_merkle_seal`, `valkey`].
- Surface evidence: `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/rate-limit-saturation.md`, `microservices/api-gateway/manifest.json`, `microservices/api-gateway/IP-010-rate-limit-adapter-valkey.md`.
