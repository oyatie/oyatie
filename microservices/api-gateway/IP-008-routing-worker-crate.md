# IP-008: `oya-api-gateway-routing-worker` crate

**Status:** design-ready
**Owner:** axis-network

## A — Scope

Worker process responsible for route-state fanout to Envoy via SDS/xDS. Owns the xDS server.

## B — Acceptance criteria

- xDS v3 conformance (Envoy 1.32+).
- Push-based; sub-second propagation.
- mTLS SDS for cert distribution.

## Wave 15 A-G substance

### A - Problem
Route bundles, xDS updates, and policy fragment changes must be reconciled outside request handlers to avoid stale admission behavior.

### B - Approach
Implement `oya-api-gateway-routing-worker` from `catalog/oya-api-gateway-routing-worker.yaml` as the reconciler for xDS route bundles, Cedar fragment activation windows, route-cache warmup, and audit evidence.

### C - Deliverables
- Worker loop consuming route-bundle change events.
- xDS publisher adapter for staged route bundle updates.
- Soak timer aligned to ADR-0294 before activation.
- Audit events for bundle staged, activated, rejected, and rolled back.
- Runbook hooks for blue/green rollback and admission regression.

### D - Ordered implementation steps
1. Add route-bundle event model and worker state machine.
2. Validate staged bundles through domain/usecase fixtures.
3. Publish staged xDS snapshots only after validation.
4. Hold activation through soak windows.
5. Emit audit-chain evidence for staged, activated, rejected, and rollback paths.
6. Test duplicate bundle, invalid route, soak timeout, and rollback.

### E - Acceptance gates
- `cargo test -p oya-api-gateway-routing-worker` passes.
- Invalid route bundles are rejected before xDS publication.
- Soak and rollback paths are covered.
- SLO burn or admission regression can trigger rollback using runbook evidence.

### F - Evidence
Grounding files: `catalog/oya-api-gateway-routing-worker.yaml`, `ARCHITECTURE.md`, `iac/envoy-config.yaml`, `runbooks/blue-green-rollback.md`, `runbooks/edge-admission-regression.md`, and `policy/route-authorization.cedar`.

### G - Counterpart comparison
Kong hybrid mode, Apigee proxy deployments, and AWS stage deployments all control route rollout. Oyatie maps that to xDS snapshots, Cedar soak, audit records, and SLO-triggered rollback.

## Remediation notes

- GitHub API ingress is the concrete counterpart for worker-driven rollout because route, abuse, and rate-limit changes must propagate without dropping active request traffic or accepting stale webhook deliveries.
- Worker tests must cover staged bundle validation, duplicate bundle delivery, xDS publication failure, soak timeout, rollback, and audit-chain event emission.
- The worker should never publish a route bundle that the domain/usecase fixtures reject; xDS is an output channel, not a validator.
- SLO-triggered rollback remains part of the worker contract because route rollout correctness is operational, not just schema-level.

## File-specific validation matrix

| Check | Local artifact | Expected evidence |
| --- | --- | --- |
| Bundle intake | `catalog/oya-api-gateway-routing-worker.yaml` | Route-bundle event model has staged and active epochs. |
| Domain validation | `IP-002-routing-domain-crate.md` | Invalid route templates reject before xDS publication. |
| Usecase validation | `IP-004-routing-usecase-crate.md` | Route decisions remain valid after bundle activation. |
| xDS output | `iac/envoy-config.yaml` | Published snapshots match Envoy v3 expectations. |
| Soak gate | `docs/decisions/ADR-0294.md` | Cedar fragment and route bundle soak complete before activation. |
| Rollback | `runbooks/blue-green-rollback.md` | Failed publish, SLO burn, and invalid bundle paths roll back. |
| GitHub ingress pressure | GitHub API ingress | Webhook/API requests are not routed through stale bundles. |
| Audit events | `contracts/api-gateway.asyncapi.yaml` | Staged, activated, rejected, and rolled-back events are emitted. |
| Cell evacuation | `runbooks/cell-evac.md` | Depooled cell removes route eligibility. |
| Regression hook | `runbooks/edge-admission-regression.md` | Admission regression runs before activation. |

## Remediation follow-up checklist

- Add a GitHub API ingress route-bundle fixture with active webhook traffic.
- Add duplicate bundle, invalid route, and stale bundle worker tests.
- Add xDS ACK timeout and NACK rollback tests.
- Add Cedar fragment soak fixture tied to ADR-0294.
- Add audit assertions for staged, activated, rejected, and rolled-back events.
- Add cell-depool fixture proving no new routes select the depooled cell.
- Keep SLO-triggered rollback as worker behavior, not just runbook prose.

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/api-gateway/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/IP-008-routing-worker-crate.md`.

## DR posture (per ADR-0343)

- Target source: `microservices/api-gateway/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `900` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`object_storage_versioned`, `audit_chain_merkle_seal`, `valkey`].
- Surface evidence: `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/rate-limit-saturation.md`, `microservices/api-gateway/manifest.json`, `microservices/api-gateway/IP-008-routing-worker-crate.md`.

## Sustainability emission (per ADR-0344)

- Per-call audit row emission MUST include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on the same metering/audit event.
- Carbon-aware scheduling eligibility: eligible only when the workload is not Tier 0/Tier 1 and not one of `eu-ai-act-annex-iii`, `hipaa-em-incident-response`, or `pci-dss-realtime-fraud-detection`; excluded calls emit `defer_rejected`.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Cost source: `microservices/api-gateway/manifest.json#paid_billing_components_emitted` declares `["per_usage"]`.
- Surface evidence: `microservices/api-gateway/manifest.json`, `microservices/api-gateway/IP-008-routing-worker-crate.md`.
