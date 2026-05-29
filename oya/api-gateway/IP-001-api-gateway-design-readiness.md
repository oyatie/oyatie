---
doc_kind: implementation-plan
id: IP-001
title: API gateway design readiness bundle
status: Accepted
owner_team: axis-network
related_adrs: [ADR-0157, ADR-0182, ADR-0183]
---

# IP-001: API Gateway Design Readiness Bundle

## Intent

Close the design/spec surface for the dedicated north-south edge tier without claiming runtime readiness. The implementation path remains separate from this evidence bundle.

## Scope

- Bind API gateway contracts across OpenAPI, AsyncAPI, and proto3.
- Define the edge admission capability, tenant policy, OpenSLO target, runbook, threat model, failure modes, FinOps model, and operational boundaries.
- Keep coarse edge authorization at the gateway while fine-grained authorization stays with workload services.

## Acceptance

- `manifest.json` references ADR authority, contracts, capability, SLO, IP, residency packs, and audit-chain events.
- `contracts/` contains OpenAPI, AsyncAPI, and proto3 surfaces for edge admission and denial events.
- `policy/tenant-scope.cedar` denies cross-tenant and cross-cell admission at the edge.
- `runbooks/edge-admission-regression.md`, `threat-model.md`, `failure-modes.md`, `cost-budget.md`, and `operational-boundaries.md` explain operator-facing boundaries without asserting production evidence.

## Wave 15 A-G substance

### A - Problem
The design bundle must prove api-gateway has enough local authority to start crate work without pretending runtime readiness exists.

### B - Approach
Bind the readiness claim to `PRD.md`, `ARCHITECTURE.md`, `manifest.json`, `contracts/api-gateway.openapi.yaml`, `contracts/api-gateway.asyncapi.yaml`, `contracts/api_gateway.proto`, `policy/tenant-scope.cedar`, `policy/route-authorization.cedar`, `slos/api-gateway.openslo.yaml`, and `runbooks/edge-admission-regression.md`.

### C - Deliverables
- Contract roster for edge admission, denial events, and proto management hooks.
- Manifest linkage for ADR-0157, ADR-0182, ADR-0183, ADR-0243, ADR-0248, ADR-0253, and ADR-0263.
- Policy roster for tenant scope, public read, CI scope, route authorization, rate limit, TLS, abuse defence, and sovereign overlay.
- SLO/runbook set for edge availability, added latency, TLS handshake, HTTP/3 negotiation, PQC negotiation, and admission regression.
- Non-runtime boundary: this IP does not deploy Envoy, Valkey, OpenBao, or Cloudflare config.

### D - Ordered implementation steps
1. Parse `manifest.json` and confirm listed contract and SLO files exist.
2. Verify OpenAPI, AsyncAPI, and proto surfaces name admission or denial events.
3. Confirm Cedar fragments include tenant/cell guards before upstream route selection.
4. Check `PRD.md` and `ARCHITECTURE.md` agree on the north-south-only boundary.
5. Link failure, threat, cost, and operational-boundary docs to runbook evidence.
6. Mark design-ready only after the docs avoid production-runtime claims.

### E - Acceptance gates
- `python -m json.tool microservices/api-gateway/manifest.json` succeeds.
- `rg "tenant_id|cell_id|route_id" microservices/api-gateway/contracts microservices/api-gateway/policy` returns local contract/policy evidence.
- `rg "edge-availability|tls-handshake|h3-negotiation|pqc-negotiation" microservices/api-gateway/slos` finds SLO coverage.
- `rg "business logic|north-south|fine-grained" microservices/api-gateway/PRD.md microservices/api-gateway/ARCHITECTURE.md` confirms boundary language.

### F - Evidence
Source artifacts: `PRD.md`, `ARCHITECTURE.md`, `manifest.json`, `competitor-parity-matrix.md`, `feature-parity-matrix-2026-05-20.md`, `policy/*.cedar`, `contracts/*`, `slos/*`, `runbooks/edge-admission-regression.md`, `threat-model.md`, and `failure-modes.md`.

### G - Counterpart comparison
AWS API Gateway, Kong Gateway, and Apigee all expose readiness evidence through contracts, policy, logs, and monitoring. This IP matches that shape locally while adding Cedar caller-side admission, SPIFFE upstream identity, ECH/PQC design, and Merkle-sealed audit events; it does not yet claim AWS usage-plan or Apigee API-product parity.

## Remediation notes

- Wave 15 gap closed by making the readiness bundle name the exact local files that prove design readiness instead of treating api-gateway as a generic edge service.
- GitHub webhook/API ingress is the concrete counterpart for a high-volume public ingress that must publish request contracts, denial semantics, signature expectations, and replay boundaries before accepting production traffic.
- The gateway bundle must therefore keep webhook-style evidence explicit: request identity, tenant/cell routing, idempotency key handling, denial event shape, retry semantics, and audit-chain sealing.
- This IP remains a foundation IP only; deployment evidence belongs to later crate/worker IPs and cannot be inferred from this bundle.
- Future remediation should update `manifest.json` when new IP files are promoted so readiness evidence and machine-readable IP inventory stay aligned.

## File-specific validation matrix

| Check | Local artifact | Expected evidence |
| --- | --- | --- |
| Contract readiness | `contracts/api-gateway.openapi.yaml` | Admission request and denial response fields are present. |
| Event readiness | `contracts/api-gateway.asyncapi.yaml` | Admitted, denied, WAF, rate-limit, TLS, canary, and honeypot events are named. |
| Proto readiness | `contracts/api_gateway.proto` | Management/control-plane fields match REST identifiers. |
| Policy readiness | `policy/tenant-scope.cedar` | Cross-tenant and cross-cell admission denial is explicit. |
| Route readiness | `policy/route-authorization.cedar` | Route action identity is separated from workload authorization. |
| SLO readiness | `slos/*.openslo.yaml` | Availability, latency, TLS, H3, and PQC targets are declared. |
| Operational readiness | `runbooks/edge-admission-regression.md` | Regression path can be followed without production claims. |
| Counterpart readiness | GitHub webhook/API ingress | Signature, retry, idempotency, and denial semantics are inspectable. |

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/api-gateway/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/IP-001-api-gateway-design-readiness.md`.

## DR posture (per ADR-0343)

- Target source: `microservices/api-gateway/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `900` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`object_storage_versioned`, `audit_chain_merkle_seal`, `valkey`].
- Surface evidence: `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/rate-limit-saturation.md`, `microservices/api-gateway/manifest.json`, `microservices/api-gateway/IP-001-api-gateway-design-readiness.md`.

## Sustainability emission (per ADR-0344)

- Per-call audit row emission MUST include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on the same metering/audit event.
- Carbon-aware scheduling eligibility: eligible only when the workload is not Tier 0/Tier 1 and not one of `eu-ai-act-annex-iii`, `hipaa-em-incident-response`, or `pci-dss-realtime-fraud-detection`; excluded calls emit `defer_rejected`.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Cost source: `microservices/api-gateway/manifest.json#paid_billing_components_emitted` declares `["per_usage"]`.
- Surface evidence: `microservices/api-gateway/manifest.json`, `microservices/api-gateway/IP-001-api-gateway-design-readiness.md`.
