---
doc_class: Implementation-Plan
ip_id: IP-journey-j96-ksa-uae-mena-onboarding
journey_ref: docs/user-journeys/j96-ksa-uae-mena-tenant-onboarding/
microservice: cloud-iac
status: draft
date: 2026-05-21
---

# IP-journey-j96: KSA/UAE MENA tenant onboarding infrastructure

## A. Problem
J96 needs cloud-iac to provision MENA tenant infrastructure with explicit pack and region evidence. The infrastructure plan must keep KSA and UAE residency decisions visible and reversible.

## B. Approach
Use OpenTofu-rendered module inputs through cloud-iac, validate residency in `iac/policy/data-residency.md`, apply through iac-applier, and record readiness in iac-registry. Operational evidence comes from `iac/runbooks/drift-remediation.md` and `iac/dashboards/drift-coverage.json`.

## C. Deliverables
- MENA region/pack examples in `iac/contracts/openapi/cloud-iac.yaml`.
- Drift and apply events in `iac/contracts/asyncapi/cloud-iac-events.yaml`.
- Residency and tenant-scope policy references.
- Drift-remediation runbook acceptance.

## D. Implementation
1. Add `jurisdiction_code`, `mena_pack_ref`, `cell_id`, and `state_backend_ref` to j96 examples.
2. Validate KSA and UAE pack routing before apply.
3. Record registry state with tenant, pack, and cell fields.
4. Require drift coverage before the onboarding handoff is marked complete.
5. Run rollback for a tenant moved to the wrong MENA pack.
6. Ensure no provider credential or OpenBao material appears in public contract examples.

## E. Acceptance
- MENA pack examples are present in OpenAPI and async events.
- Data-residency and tenant Cedar controls are cited.
- Drift remediation is an explicit verification step.
- Registry evidence is sufficient for tenancy handoff.

## F. Evidence
- Journey: `docs/user-journeys/j96-ksa-uae-mena-tenant-onboarding/README.md`.
- Policy: `iac/policy/data-residency.md`.
- Runbook: `iac/runbooks/drift-remediation.md`.

## G. Counterparts
| Counterpart | Gap closed |
|---|---|
| Terraform Cloud | Adds KSA/UAE residency pack routing and tenant-scoped registry evidence. |
| Env0 | Matches environment orchestration while adding sovereignty gates. |
| OpenTofu | Adds Oyatie policy and audit evidence around module execution. |

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `iac/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/proto/cloud-iac.proto`, `iac/IP-journey-j96-ksa-uae-mena-onboarding.md`.
