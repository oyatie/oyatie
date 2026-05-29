# IP-005: `oya-api-gateway-routing-adapter` crate

**Status:** design-ready
**Owner:** axis-network

## A — Scope

Adapter implementations of kernel ports — wires use-case to real-world I/O.

- `EnvoyRouteLookupAdapter` — backed by Envoy xDS subscription.
- `CellResidencyAdapter` — backed by tenancy µservice gRPC.
- `AuditEmitAdapter` — backed by audit-chain µservice + sidecar signer.

## B — Acceptance criteria

- Mockable for tests.
- Real I/O behind feature flags.
- Per-adapter integration test (against test fixtures).

## C — Dependencies

- `oya-api-gateway-routing-kernel`, `oya-api-gateway-routing-domain`, `tonic`, `tokio`.

## Wave 15 A-G substance

### A - Problem
The usecase needs real route lookup, residency, and audit adapters without leaking Envoy xDS, tenancy, or audit-chain APIs into domain/usecase layers.

### B - Approach
Implement `oya-api-gateway-routing-adapter` from `catalog/oya-api-gateway-routing-adapter.yaml` as the I/O layer for Envoy xDS route subscriptions, caller-side policy-eval context, tenancy-backed cell data, and audit-chain emission.

### C - Deliverables
- Envoy xDS-backed `RouteLookupPort` adapter consuming `/specs/envoy-xds-v3.proto`.
- Cell residency adapter reading signed principal context without hot-path calls to a standalone cell service.
- Audit adapter emitting `oya.api_gateway.request.admitted` and `oya.api_gateway.request.denied`.
- Fixture adapters for route bundles, cell-health states, and audit assertions.
- Feature flags for live Envoy, tenancy, and audit-chain integration.

### D - Ordered implementation steps
1. Build fixture adapters first.
2. Add Envoy xDS route bundle decoding.
3. Add cell context adapter using `tenant.cell`, `cell_epoch`, `pack`, and `region`.
4. Add audit payload builder with Merkle-seal handoff fields.
5. Gate live network integrations behind explicit features.
6. Run adapter integration tests against route and policy fixtures.

### E - Acceptance gates
- `cargo test -p oya-api-gateway-routing-adapter --features fixtures` passes.
- Live I/O features are off by default.
- Route fixtures cover Envoy route, no route, duplicate route, and stale bundle.
- Audit payload tests include tenant, principal, cell, route, request, and deny reason.

### F - Evidence
Grounding files: `catalog/oya-api-gateway-routing-adapter.yaml`, `ARCHITECTURE.md`, `iac/envoy-config.yaml`, `contracts/api_gateway.proto`, `policy/route-authorization.cedar`, `policy/sov-cloud-overlay.cedar`, and `runbooks/edge-admission-regression.md`.

### G - Counterpart comparison
Kong and Apigee adapters/plugins bridge abstract policy flows to runtime APIs; AWS does the same through integrations and authorizers. Oyatie's adapter keeps Envoy xDS, tenancy cell state, and audit-chain emission replaceable for sovereign cells and local tests.

## Remediation notes

- GitHub webhook/API ingress is the concrete counterpart for adapter work because live ingress depends on external delivery headers, retry behavior, route bundle freshness, and audit-ready denial records.
- The adapter must convert runtime sources into kernel values: Envoy xDS route snapshots, principal/cell context, and audit-chain payloads. It should not invent domain rules or silently reclassify routes.
- Fixture adapters need webhook cases for duplicate delivery, missing signature context, stale route bundle, and route-denied audit emission.
- Live adapters stay feature-gated until route-bundle and audit fixtures prove the same behavior without network dependencies.

## File-specific validation matrix

| Check | Local artifact | Expected evidence |
| --- | --- | --- |
| xDS route source | `iac/envoy-config.yaml` | Envoy route snapshots convert into domain route values. |
| Cell context | `ARCHITECTURE.md` | `tenant.cell`, `cell_epoch`, `pack`, and `region` are available before admit. |
| Audit adapter | `contracts/api-gateway.asyncapi.yaml` | Admitted and denied events carry route, tenant, cell, and request identifiers. |
| Feature gates | `catalog/oya-api-gateway-routing-adapter.yaml` | Live Envoy, tenancy, and audit clients are explicit opt-ins. |
| Fixture mode | `runbooks/edge-admission-regression.md` | Regression tests run without live network dependencies. |
| Stale bundle | `runbooks/blue-green-rollback.md` | Stale or rejected route bundles trigger rollback evidence. |
| Webhook retry | GitHub webhook/API ingress | Duplicate delivery does not create duplicate audit or route decisions. |
| Signature context | GitHub webhook/API ingress | Missing signature context denies before upstream selection. |
| Sovereign behavior | `policy/sov-cloud-overlay.cedar` | Adapter passes cell facts without rewriting policy outcomes. |
| Error mapping | `failure-modes.md` | Route unavailable and audit unavailable states are distinct. |

## Remediation follow-up checklist

- Confirm fixture adapter names match the eventual Rust module names.
- Add one stale xDS bundle fixture tied to a GitHub webhook replay route.
- Add one audit-chain unavailable fixture that preserves request denial behavior.
- Add one sovereign-cell fixture that proves adapter facts are passed through unchanged.
- Keep live network clients behind explicit features until fixture parity is green.

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/api-gateway/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/IP-005-routing-adapter-crate.md`.

## Sustainability emission (per ADR-0344)

- Per-call audit row emission MUST include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on the same metering/audit event.
- Carbon-aware scheduling eligibility: eligible only when the workload is not Tier 0/Tier 1 and not one of `eu-ai-act-annex-iii`, `hipaa-em-incident-response`, or `pci-dss-realtime-fraud-detection`; excluded calls emit `defer_rejected`.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Cost source: `microservices/api-gateway/manifest.json#paid_billing_components_emitted` declares `["per_usage"]`.
- Surface evidence: `microservices/api-gateway/manifest.json`, `microservices/api-gateway/IP-005-routing-adapter-crate.md`.

## Pod runtime tier (per ADR-0338)

- `pod_runtime_tier: 0`.
- Justification: tenant-customer code is present in this IP's execution path; Tier 0 requires Kata plus Cloud Hypervisor isolation.
- Surface evidence: `microservices/api-gateway/runbooks/edge-admission-regression.md`, `microservices/api-gateway/manifest.json`, `microservices/api-gateway/IP-005-routing-adapter-crate.md`; matched trigger term(s): `plugin`.
- Admission expectation: spawned workloads for this path use `kata-cloud-hypervisor`; first-party helpers may only run outside Tier 0 when split into a separate non-tenant-customer IP.
