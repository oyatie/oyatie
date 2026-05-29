---
doc_class: Implementation-Plan
ip_id: IP-journey-j87-cell-infra-declarative
journey_ref: docs/user-journeys/j87-fedramp-high-il5-air-gap-deployment/
microservice: cloud-iac
status: draft
date: 2026-05-21
---

# IP-journey-j87: FedRAMP High / IL5 air-gap deployment infrastructure

## A. Problem
J87 cannot depend on SaaS IaC runners or live public registries. cloud-iac must package an air-gap-capable OpenTofu/GitOps bundle and prove that apply, rollback, and drift checks work from local artifacts.

## B. Approach
Use the existing self-hosted renderer and applier crates from `microservices/cloud-iac/manifest.json`. The bundle includes contract examples from `microservices/cloud-iac/contracts/proto/cloud-iac.proto`, signed module provenance from `microservices/cloud-iac/slos/slsa-provenance-completeness.openslo.yaml`, and operational recovery through `microservices/cloud-iac/runbooks/stuck-apply-recovery.md`.

## C. Deliverables
- Air-gap bundle fields in `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml`.
- Offline apply event examples in `microservices/cloud-iac/contracts/asyncapi/cloud-iac-events.yaml`.
- Evidence links to `microservices/cloud-iac/runbooks/stuck-apply-recovery.md` and `microservices/cloud-iac/runbooks/restore-drill-quarterly.md`.
- Capability references to `microservices/cloud-iac/capabilities/iac-apply.yaml` and `microservices/cloud-iac/capabilities/iac-rollback.yaml`.

## D. Implementation
1. Define `artifact_bundle_ref`, `signature_bundle_ref`, and `offline_registry_ref` in j87 examples.
2. Require renderer output to be reproducible from local source inputs.
3. Validate SLSA/cosign provenance before the applier opens an apply session.
4. Route rollback through the rollback bounded context, not manual shell commands.
5. Document quarterly restore drills as acceptance evidence for IL5 readiness.
6. Deny any plan that references Terraform Cloud, Pulumi Cloud, Spacelift, or other SaaS-only execution.

## E. Acceptance
- j87 examples show no SaaS runner dependency.
- Stuck apply and restore drill runbooks are cited.
- Air-gap bundle validation emits a signed audit event before mutation.
- `microservices/cloud-iac/competitor-parity-matrix.md` claim boundaries remain respected.

## F. Evidence
- Journey: `docs/user-journeys/j87-fedramp-high-il5-air-gap-deployment/README.md`.
- Runbook: `microservices/cloud-iac/runbooks/stuck-apply-recovery.md`.
- Parity matrix: `microservices/cloud-iac/competitor-parity-matrix.md`.

## G. Counterparts
| Counterpart | Gap closed |
|---|---|
| Terraform Cloud | Avoids SaaS execution for IL5 air-gap contexts. |
| Spacelift / Env0 | Keeps policy orchestration but makes it portable to disconnected cells. |
| ArgoCD / Flux | Adds provenance-gated OpenTofu apply beside GitOps sync. |

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/cloud-iac/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml`, `microservices/cloud-iac/contracts/asyncapi/cloud-iac-events.yaml`, `microservices/cloud-iac/contracts/proto/cloud-iac.proto`, `microservices/cloud-iac/IP-journey-j87-cell-infra-declarative.md`.
