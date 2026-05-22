---
doc_class: Implementation-Plan
ip_id: IP-journey-j91-us-msb-mtl-overlay
journey_ref: docs/user-journeys/j91-us-state-money-transmitter-licensing/
microservice: cloud-iac
status: draft
date: 2026-05-21
---

# IP-journey-j91: US MSB / MTL infrastructure overlay

## A. Problem
J91 does not make cloud-iac a money-transmission service. It requires cloud-iac to supply licensed-state infrastructure controls: region/state tagged module inputs, immutable apply evidence, and rollback paths for regulated fintech tenants.

## B. Approach
Use `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml` for state-scoped plan/apply examples, `microservices/cloud-iac/policy/tenant-scope.cedar` for tenant isolation, and `microservices/cloud-iac/cost-budget.md` for state-specific capacity/cost evidence. Apply and rollback stay in the existing iac-applier and iac-rollback bounded contexts.

## C. Deliverables
- State overlay fields in OpenAPI examples.
- Audit event examples in `microservices/cloud-iac/contracts/asyncapi/cloud-iac-events.yaml`.
- SLO references to `microservices/cloud-iac/slos/iac-apply-latency.openslo.yaml`.
- Rollback references to `microservices/cloud-iac/runbooks/rollback-orchestration.md`.

## D. Implementation
1. Add `licensed_state_codes`, `tenant_id`, `pack_id`, and `state_backend_ref` to j91 examples.
2. Validate state overlays before apply so an unlicensed state cannot receive infrastructure.
3. Attach audit-chain evidence to render, validate, apply, rollback, and drift transitions.
4. Register the applied state in iac-registry with tenant and state metadata.
5. Exercise rollback for a state overlay removed after license suspension.
6. Document that payment thresholds remain outside cloud-iac ownership.

## E. Acceptance
- j91 examples show state-scoped infrastructure gating, not payment logic.
- Tenant Cedar scope is mandatory for mutation.
- Rollback evidence uses append-only compensation.
- The IP cites real cloud-iac contracts, policy, SLO, and runbook files.

## F. Evidence
- Journey: `docs/user-journeys/j91-us-state-money-transmitter-licensing/README.md`.
- Service PRD: `microservices/cloud-iac/PRD.md`.
- Parity matrix: `microservices/cloud-iac/competitor-parity-matrix.md`.

## G. Counterparts
| Counterpart | Gap closed |
|---|---|
| Terraform Cloud | Adds regulated-state metadata and Cedar-denied applies. |
| Spacelift | Matches policy runs while binding state evidence to tenant packs. |
| GitHub Actions IaC | Replaces ad hoc workflows with registry-backed apply/rollback evidence. |

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/cloud-iac/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml`, `microservices/cloud-iac/contracts/asyncapi/cloud-iac-events.yaml`, `microservices/cloud-iac/contracts/proto/cloud-iac.proto`, `microservices/cloud-iac/IP-journey-j91-us-msb-mtl-overlay.md`.

## DR posture (per ADR-0343)

- Target source: `microservices/cloud-iac/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `PCI-DSS-L1-v4` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `86400` seconds; RPO p99 <= `3600` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`object_storage_versioned`, `seaweedfs_replicated`, `postgres_wal_g`].
- Surface evidence: `microservices/cloud-iac/runbooks/restore-drill-quarterly.md`, `microservices/cloud-iac/runbooks/seaweedfs-volume-failover.md`, `microservices/cloud-iac/manifest.json`, `microservices/cloud-iac/IP-journey-j91-us-msb-mtl-overlay.md`.

## Sustainability emission (per ADR-0344)

- Per-call audit row emission MUST include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on the same metering/audit event.
- Carbon-aware scheduling eligibility: eligible only when the workload is not Tier 0/Tier 1 and not one of `eu-ai-act-annex-iii`, `hipaa-em-incident-response`, or `pci-dss-realtime-fraud-detection`; excluded calls emit `defer_rejected`.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Cost source: `microservices/cloud-iac/manifest.json#paid_billing_components_emitted` declares `["per_usage"]`.
- Surface evidence: `microservices/cloud-iac/manifest.json`, `microservices/cloud-iac/IP-journey-j91-us-msb-mtl-overlay.md`.
