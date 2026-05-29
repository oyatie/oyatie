# IP-014: `oya-api-gateway-tls-cert-rotation-worker` crate

**Status:** design-ready
**Owner:** axis-network + ops-security
**Authority:** ADR-0253, ADR-0295, ADR-0296.

## A — Scope

Worker that polls OpenBao for fresh TLS / ECH / PQC cert bundles and pushes to Envoy via SDS.

## B — Acceptance criteria

- Sub-60s rotation propagation.
- Zero connection drop.
- Audit event per rotation.
- Graceful degradation if OpenBao unavailable (continue with current cert; alert).

## Wave 15 A-G substance

### A - Problem
The gateway terminates TLS 1.3, ECH, and PQC negotiation, so certificate and key-material rotation must be automated without interrupting active north-south traffic or bypassing audit requirements.

### B - Approach
Implement `oya-api-gateway-tls-cert-rotation-worker` from `catalog/oya-api-gateway-tls-cert-rotation-worker.yaml` as the OpenBao-to-Envoy SDS reconciler. It watches certificate bundles, validates trust-chain and policy constraints, stages SDS updates, observes propagation, and emits audit-chain rotation records.

### C - Deliverables
- OpenBao bundle reader for TLS, ECH config, PQC certificate material, intermediate chain, and bundle epoch.
- SDS publisher with staged update, active epoch tracking, and rollback to last known good bundle.
- Policy validator for SAN, SPIFFE trust domain, tenant/cell eligibility, expiry window, and key algorithm.
- Rotation state machine for pending, staged, active, failed, and rolled back.
- Alert and audit events for rotation success, stale bundle, validation failure, OpenBao unavailable, and SDS propagation failure.
- Graceful degradation path that keeps the current valid bundle while page/alert signals fire.

### D - Ordered implementation steps
1. Define bundle DTOs for TLS, ECH, PQC, chain, epoch, and provenance fields.
2. Add OpenBao read port and fixture bundle source.
3. Validate bundle policy before SDS publication.
4. Stage SDS update and wait for Envoy ACK/health evidence.
5. Activate new epoch only after propagation succeeds.
6. Emit audit records and metrics for every transition.
7. Add rollback and stale-bundle tests.

### E - Acceptance gates
- `cargo test -p oya-api-gateway-tls-cert-rotation-worker --features fixtures` passes.
- Rotation propagation fixture completes under 60 seconds.
- Failed OpenBao reads do not drop existing valid connections.
- Invalid SAN, expired chain, wrong SPIFFE trust domain, and PQC mismatch are rejected before SDS publication.
- Audit events align with `oya.api_gateway.tls.cert.rotated`, `oya.api_gateway.ech.config.rotated`, and `oya.api_gateway.pqc.handshake.completed`.

### F - Evidence
Grounding files: `catalog/oya-api-gateway-tls-cert-rotation-worker.yaml`, `iac/cert-manager.yaml`, `iac/ech-config.yaml`, `iac/pqc-cert.yaml`, `iac/spire-trust-bundle.yaml`, `iac/openbao-policy.hcl`, `slos/tls-handshake-success.openslo.yaml`, and `runbooks/audit-key-rotation.md`.

### G - Counterpart comparison
Salesforce API ingress is the concrete counterpart because enterprise API edges must rotate certificate chains, preserve client trust, and avoid tenant-impacting outages during rotation. Oyatie adds OpenBao provenance, SDS staging, ECH/PQC coverage, and audit-chain records.

## Remediation notes

- Rewrote the TLS worker stub into a reconciler plan with bundle validation, SDS staging, rollback, and audit gates.
- This IP covers runtime certificate material flow only; route admission and service auth decisions remain in routing/auth IPs.
- Future remediation should add a dedicated `runbooks/tls-cert-rotation.md` if the architecture reference remains separate from `runbooks/audit-key-rotation.md`.

## File-specific validation matrix

| Check | Local artifact | Expected evidence |
| --- | --- | --- |
| OpenBao policy | `iac/openbao-policy.hcl` | Worker can read only expected certificate bundle paths. |
| SDS publish | `iac/envoy-config.yaml` | Envoy receives staged secret updates. |
| ECH | `iac/ech-config.yaml` | ECH config epoch rotates with audit evidence. |
| PQC | `iac/pqc-cert.yaml` | PQC material is validated before publication. |
| SPIFFE | `iac/spire-trust-bundle.yaml` | Trust domain and SVID constraints are checked. |
| Salesforce pressure | Salesforce API ingress | Enterprise API clients retain trust during certificate rotation. |
| TLS SLO | `slos/tls-handshake-success.openslo.yaml` | Rotation does not degrade handshake success. |
| Audit | `runbooks/audit-key-rotation.md` | Rotation record is sealed and operator-visible. |
| Failure mode | `failure-modes.md` | OpenBao unavailable, invalid bundle, and SDS reject are distinct. |
| Graceful degradation | `incident-response.md` | Current valid cert remains active while alerts fire. |
| Network policy | `iac/network-policy.yaml` | Worker egress is limited to OpenBao/SDS paths. |
| App integration | `IP-016-app-supervisor.md` | Supervisor restarts or degrades worker state predictably. |

## Remediation follow-up checklist

- Add Salesforce-style long-lived client fixture across certificate rotation.
- Add invalid SAN, expired chain, wrong trust-domain, and PQC mismatch fixtures.
- Add OpenBao unavailable fixture proving current valid bundle stays active.
- Add SDS NACK rollback fixture.
- Add audit assertions for TLS, ECH, and PQC rotation events.

## DR posture (per ADR-0343)

- Target source: `microservices/api-gateway/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `900` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`object_storage_versioned`, `audit_chain_merkle_seal`, `valkey`].
- Surface evidence: `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/rate-limit-saturation.md`, `microservices/api-gateway/manifest.json`, `microservices/api-gateway/IP-014-tls-cert-rotation-worker.md`.
