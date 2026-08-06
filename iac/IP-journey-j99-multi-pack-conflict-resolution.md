---
doc_class: Implementation-Plan
ip_id: IP-journey-j99-multi-pack-conflict-resolution
journey_ref: docs/user-journeys/j99-cross-jurisdiction-multi-pack-conflict-resolution/
microservice: cloud-iac
status: draft
date: 2026-05-21
---

# IP-journey-j99: Multi-pack infrastructure conflict resolution

## A. Problem
J99 requires cloud-iac to resolve infrastructure conflicts when multiple jurisdiction packs apply to the same tenant. The service must choose the stricter state/backend/cell constraints before rendering or applying infrastructure.

## B. Approach
Move conflict handling into validator and registry evidence. `iac/policy/data-residency.md` names residency constraints, `iac/policy/tenant-scope.cedar` gates mutation, and `iac/runbooks/drift-remediation.md` handles already-applied infrastructure after a stricter pack wins.

## C. Deliverables
- Multi-pack conflict examples in `iac/contracts/openapi/cloud-iac.yaml`.
- Conflict-denied and conflict-resolved events in `iac/contracts/asyncapi/cloud-iac-events.yaml`.
- Registry and drift references.
- Rollback orchestration references for stricter-pack migration.

## D. Implementation
1. Add `active_pack_refs`, `winning_pack_ref`, and `conflict_resolution_ref` to j99 examples.
2. Validate that renderer output uses the stricter pack before plan creation.
3. Deny apply when two packs imply incompatible state backends.
4. Record the chosen pack and evidence hash in iac-registry.
5. Run drift remediation for infrastructure previously applied under a weaker pack.
6. Roll back or re-apply through cloud-iac only; never patch state manually.

## E. Acceptance
- A conflicting pack pair fails before apply unless `winning_pack_ref` is present.
- Registry records the conflict decision and state backend selection.
- Drift remediation and rollback are both cited.
- No cross-service generic invariant rows remain.

## F. Evidence
- Journey: `docs/user-journeys/j99-cross-jurisdiction-multi-pack-conflict-resolution/README.md`.
- Policy: `iac/policy/data-residency.md`.
- Runbook: `iac/runbooks/drift-remediation.md`.

## G. Counterparts
| Counterpart | Gap closed |
|---|---|
| Terraform Cloud | Adds pack-conflict resolution before state backend mutation. |
| Spacelift | Matches policy enforcement while binding stricter-pack choice to registry evidence. |
| ArgoCD / Flux | Keeps reconciliation downstream of resolved infrastructure constraints. |

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `iac/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/proto/cloud-iac.proto`, `iac/IP-journey-j99-multi-pack-conflict-resolution.md`.
