---
doc_class: Implementation-Plan
ip_id: IP-journey-j98-au-privacy-apra-cps234
journey_ref: docs/user-journeys/j98-au-privacy-apra-cps-234-tenant/
microservice: cloud-iac
status: draft
date: 2026-05-21
---

# IP-journey-j98: AU Privacy / APRA CPS 234 infrastructure tenant

## A. Problem
J98 needs APRA CPS 234 infrastructure controls: security posture, incident recovery, provider isolation, and evidence retention. cloud-iac supplies the plan/apply/rollback substrate and must avoid claiming it owns business-risk assessment.

## B. Approach
Use cloud-iac validator and applier contracts with incident-response evidence from `microservices/cloud-iac/incident-response.md`, isolation rules from `microservices/cloud-iac/policy/iac-isolation.md`, and rollback operations from `microservices/cloud-iac/runbooks/rollback-orchestration.md`.

## C. Deliverables
- APRA profile fields in `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml`.
- Incident and rollback events in `microservices/cloud-iac/contracts/asyncapi/cloud-iac-events.yaml`.
- Evidence links to incident response, isolation, and rollback docs.
- Dashboard references to `microservices/cloud-iac/dashboards/apply-success-rate.json`.

## D. Implementation
1. Add `apra_cps234_profile`, `incident_response_ref`, and `provider_isolation_ref` to j98 examples.
2. Validate isolation policy before rendering deployable outputs.
3. Require signed apply evidence for each APRA-profile mutation.
4. Record incident recovery readiness in the registry evidence bundle.
5. Test rollback for compromised provider credential references.
6. Keep business-risk scoring outside cloud-iac ownership.

## E. Acceptance
- APRA examples cite incident response and isolation paths.
- Rollback protects evidence and never deletes audit-chain records.
- Apply success dashboard is part of the readiness check.
- Contract examples avoid provider secrets.

## F. Evidence
- Journey: `docs/user-journeys/j98-au-privacy-apra-cps-234-tenant/README.md`.
- Incident response: `microservices/cloud-iac/incident-response.md`.
- Dashboard: `microservices/cloud-iac/dashboards/apply-success-rate.json`.

## G. Counterparts
| Counterpart | Gap closed |
|---|---|
| Terraform Cloud | Adds APRA-profile readiness and incident recovery evidence. |
| Spacelift / Env0 | Matches policy-gated IaC while binding rollback to audit-chain. |
| OpenTofu | Adds regulated operational evidence around OSS state mutation. |

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/cloud-iac/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml`, `microservices/cloud-iac/contracts/asyncapi/cloud-iac-events.yaml`, `microservices/cloud-iac/contracts/proto/cloud-iac.proto`, `microservices/cloud-iac/IP-journey-j98-au-privacy-apra-cps234.md`.
