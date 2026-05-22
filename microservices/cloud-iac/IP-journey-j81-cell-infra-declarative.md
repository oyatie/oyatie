---
doc_class: Implementation-Plan
ip_id: IP-journey-j81-cell-infra-declarative
journey_ref: docs/user-journeys/j81-kr-csap-sovereign-cell-audit-pull/
microservice: cloud-iac
status: draft
date: 2026-05-21
---

# IP-journey-j81: KR CSAP sovereign audit pull infrastructure

## A. Problem
J81 asks a Korean sovereign auditor to pull proof that a cell is CSAP-ready. cloud-iac owns the infrastructure evidence: declared OpenTofu state, signed module provenance, namespace/bootstrap history, and rollback posture. The old stamped IP did not say which evidence surfaces the auditor can inspect.

## B. Approach
Use the iac-registry as the audit index and the iac-applier event stream as the immutable proof source. The OpenAPI contract in `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml` exposes read-only registry evidence; `microservices/cloud-iac/policy/auditor-scope.cedar` limits the auditor principal; `microservices/cloud-iac/dashboards/registry-health.json` and `microservices/cloud-iac/dashboards/drift-coverage.json` supply operational context.

## C. Deliverables
- Add j81 audit-pull examples to `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml`.
- Add an `iac.registry.audit_pulled` style event example to `microservices/cloud-iac/contracts/asyncapi/cloud-iac-events.yaml`.
- Bind auditor authorization to `microservices/cloud-iac/policy/auditor-scope.cedar`.
- Reference restore and registry recovery in `microservices/cloud-iac/runbooks/registry-restore.md`.

## D. Implementation
1. Define the read-only registry query shape with `tenant_id`, `cell_id`, `pack_id`, `audit_window`, and `evidence_hash`.
2. Ensure auditor requests never expose OpenBao leases, provider credentials, or raw state backend secrets.
3. Attach SLSA provenance evidence from `microservices/cloud-iac/slos/slsa-provenance-completeness.openslo.yaml`.
4. Require drift status from `microservices/cloud-iac/dashboards/drift-coverage.json` before the audit bundle is considered complete.
5. Exercise the registry restore path so an auditor pull can be replayed after loss of a projection.
6. Record negative tests for wrong tenant, expired audit window, and non-auditor principal.

## E. Acceptance
- Auditor read examples exist in OpenAPI and do not include mutating verbs.
- Cedar auditor-scope text explicitly gates j81 audit reads.
- Registry-health and drift dashboards are linked in the IP evidence bundle.
- The rollback/restore path names `microservices/cloud-iac/runbooks/registry-restore.md`.

## F. Evidence
- Journey: `docs/user-journeys/j81-kr-csap-sovereign-cell-audit-pull/README.md`.
- Service anchor: `microservices/cloud-iac/ARCHITECTURE.md`.
- Runbook: `microservices/cloud-iac/runbooks/registry-restore.md`.

## G. Counterparts
| Counterpart | Gap closed |
|---|---|
| Terraform Cloud | Adds tenant-scoped auditor read surfaces instead of org-wide workspace history. |
| ArgoCD | Adds CSAP evidence packaging beyond application sync state. |
| Spacelift | Matches audit trail expectations while keeping Cedar-scoped read authorization. |

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/cloud-iac/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml`, `microservices/cloud-iac/contracts/asyncapi/cloud-iac-events.yaml`, `microservices/cloud-iac/contracts/proto/cloud-iac.proto`, `microservices/cloud-iac/IP-journey-j81-cell-infra-declarative.md`.
