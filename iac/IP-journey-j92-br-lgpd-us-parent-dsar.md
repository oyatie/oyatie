---
doc_class: Implementation-Plan
ip_id: IP-journey-j92-br-lgpd-us-parent-dsar
journey_ref: docs/user-journeys/j92-br-lgpd-dsar-with-us-parent/
microservice: cloud-iac
status: draft
date: 2026-05-21
---

# IP-journey-j92: BR LGPD DSAR infrastructure boundary

## A. Problem
J92 needs infrastructure evidence that a Brazilian LGPD tenant can serve a DSAR while a US parent exists. cloud-iac owns residency-aware state, not DSAR content; it must prove the cell, storage, and audit substrate do not collapse parent and subsidiary boundaries.

## B. Approach
Bind DSAR infra checks to `microservices/cloud-iac/policy/data-residency.md`, `microservices/cloud-iac/contracts/proto/cloud-iac.proto`, and registry health. The plan records cell and state references for BR pack overlays while leaving subject export logic to the DSAR-owning service.

## C. Deliverables
- BR pack/state examples in `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml`.
- Registry read examples in `microservices/cloud-iac/contracts/asyncapi/cloud-iac-events.yaml`.
- Residency acceptance text linked to `microservices/cloud-iac/policy/data-residency.md`.
- Recovery evidence linked to `microservices/cloud-iac/runbooks/registry-restore.md`.

## D. Implementation
1. Add a j92 example with `subject_region=BR`, `parent_tenant_region=US`, and `residency_pack_ref`.
2. Validate that infrastructure state refs for BR and US parent cells remain separate.
3. Emit registry evidence for DSAR infrastructure readiness without exposing DSAR payloads.
4. Use `microservices/cloud-iac/dashboards/registry-health.json` as the readiness dashboard.
5. Test restore of registry projection for BR evidence after projection loss.
6. Deny apply if the pack overlay routes BR subject storage to a US-only state backend.

## E. Acceptance
- The IP distinguishes infrastructure residency proof from DSAR data export.
- BR/US parent state references are separately modeled in examples.
- Registry restore and data-residency policy are both named.
- No raw DSAR payload fields appear in cloud-iac contracts.

## F. Evidence
- Journey: `docs/user-journeys/j92-br-lgpd-dsar-with-us-parent/README.md`.
- Policy: `microservices/cloud-iac/policy/data-residency.md`.
- Dashboard: `microservices/cloud-iac/dashboards/registry-health.json`.

## G. Counterparts
| Counterpart | Gap closed |
|---|---|
| Terraform Cloud | Adds subject-region residency proof beyond workspace region selection. |
| Env0 | Matches tenant environment orchestration while separating parent/subsidiary evidence. |
| OpenTofu | Adds service-level DSAR infrastructure evidence around raw state operations. |

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/cloud-iac/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml`, `microservices/cloud-iac/contracts/asyncapi/cloud-iac-events.yaml`, `microservices/cloud-iac/contracts/proto/cloud-iac.proto`, `microservices/cloud-iac/IP-journey-j92-br-lgpd-us-parent-dsar.md`.
