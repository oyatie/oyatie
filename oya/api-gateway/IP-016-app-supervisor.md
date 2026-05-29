# IP-016: `oya-api-gateway-app-supervisor` crate

**Status:** design-ready
**Owner:** axis-network

## A — Scope

The top-level Rust app crate that wires kernel + adapters + workers + REST + gRPC; produces the deploy binary.

## B — Acceptance criteria

- Single binary < 80MB.
- Cold-start < 1s.
- All workers supervised via tokio::select + restart-with-backoff.
- Graceful shutdown on SIGTERM (drain connections in ≤30s).

## Wave 15 A-G substance

### A - Problem
The gateway crate set needs one deployable supervisor that wires request admission, REST/gRPC management, workers, observability, secrets, and graceful shutdown without hiding failures behind a monolithic process.

### B - Approach
Implement `oya-api-gateway-app-supervisor` from `catalog/oya-api-gateway-app.yaml` as the top-level Rust binary. It composes domain/usecase crates, adapters, REST/gRPC servers, Envoy-facing workers, canary worker, TLS rotation worker, abuse Wasm build artifact registration, and lifecycle telemetry.

### C - Deliverables
- Configuration loader for cell, tenant pack roster, listener, Envoy/SDS endpoints, OpenBao, Valkey, audit-chain, identity, and observability.
- Supervision tree for REST, gRPC, routing worker, rate-limit adapter health, TLS rotation worker, canary shifter, audit emitter, and readiness probes.
- Startup dependency checks with explicit degraded/failed modes.
- Graceful shutdown that stops accepting new requests, drains listeners, flushes audit buffers, and stops workers within 30 seconds.
- Backoff and restart policy per worker with circuit-breaker visibility.
- Build gate for binary size, cold-start budget, and feature-flag matrix.

### D - Ordered implementation steps
1. Define app config schema and fixture config files for local, CI, and cell deploy.
2. Wire domain/usecase/adapters through explicit constructors.
3. Start REST/gRPC listeners and workers under a supervision tree.
4. Add health/readiness endpoints that reflect dependency and worker state.
5. Add graceful shutdown and drain tests.
6. Add startup failure tests for missing OpenBao, Valkey unavailable, audit unavailable, and invalid route bundle.
7. Add binary-size and cold-start checks to CI once the crate exists.

### E - Acceptance gates
- `cargo test -p oya-api-gateway-app --features fixtures` passes.
- Supervisor fails startup on invalid mandatory config and enters degraded mode only for documented optional dependencies.
- SIGTERM drain fixture completes within 30 seconds and flushes audit events.
- Worker restart tests prove bounded backoff and no restart storm.
- Binary size remains under 80 MB and cold-start remains under 1 second in release fixture.

### F - Evidence
Grounding files: `catalog/oya-api-gateway-app.yaml`, `manifest.json`, `iac/k8s-deployment.yaml`, `iac/network-policy.yaml`, `iac/envoy-config.yaml`, `operational-boundaries.md`, `incident-response.md`, and `dashboards/edge-overview.json`.

### G - Counterpart comparison
ServiceNow API ingress is the concrete counterpart because enterprise ingress must supervise multiple integration surfaces while surfacing degraded dependency state to operators. Oyatie mirrors that operational shape with a single Rust binary, explicit worker supervision, and cell-aware readiness gates.

## Remediation notes

- Rewrote the app-supervisor stub into a deployable-binary plan with config, supervision tree, startup/degraded modes, shutdown, and build-budget gates.
- Keep this IP focused on composition and lifecycle; domain decisions, Valkey behavior, TLS rotation, and canary policy stay in their dedicated IPs.
- Future remediation should align the crate/package name with the catalog file `oya-api-gateway-app.yaml` and manifest title if implementation chooses `oya-api-gateway-app` rather than `app-supervisor`.

## File-specific validation matrix

| Check | Local artifact | Expected evidence |
| --- | --- | --- |
| Catalog | `catalog/oya-api-gateway-app.yaml` | Binary ownership and service identity are declared. |
| Deployment | `iac/k8s-deployment.yaml` | Probes, resources, and process shape match supervisor behavior. |
| Network policy | `iac/network-policy.yaml` | Egress matches Valkey, OpenBao, audit, identity, Envoy/SDS, and observability needs. |
| Envoy | `iac/envoy-config.yaml` | Supervisor starts workers compatible with xDS/SDS sidecars. |
| ServiceNow pressure | ServiceNow API ingress | Enterprise integration traffic sees clear degraded/ready states. |
| Shutdown | `operational-boundaries.md` | Drain, audit flush, and worker stop are bounded. |
| Incident response | `incident-response.md` | Degraded dependencies map to SEV and action paths. |
| Dashboard | `dashboards/edge-overview.json` | Worker and listener health is observable. |
| Worker tree | `IP-008-routing-worker-crate.md` | Routing worker is supervised with bounded restart policy. |
| TLS worker | `IP-014-tls-cert-rotation-worker.md` | Rotation failures degrade or alert without hidden crash loops. |
| Canary worker | `IP-015-canary-cohort-shifter.md` | SLO-driven rollback worker is visible in health. |
| Build budget | `performance-benchmark-numbers-2026-05-20.md` | Cold-start and binary-size claims have a verification path. |

## Remediation follow-up checklist

- Add ServiceNow-style steady integration traffic fixture for readiness changes.
- Add missing mandatory config and optional dependency degraded-mode fixtures.
- Add SIGTERM drain fixture that proves audit flush and listener drain complete.
- Add worker restart storm fixture with bounded backoff.
- Add binary-size and cold-start checks when the crate exists.

## DR posture (per ADR-0343)

- Target source: `microservices/api-gateway/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `900` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`object_storage_versioned`, `audit_chain_merkle_seal`, `valkey`].
- Surface evidence: `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/rate-limit-saturation.md`, `microservices/api-gateway/manifest.json`, `microservices/api-gateway/IP-016-app-supervisor.md`.
