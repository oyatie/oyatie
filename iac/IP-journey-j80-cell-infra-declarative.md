---
doc_class: Implementation-Plan
ip_id: IP-journey-j80-cell-infra-declarative
journey_ref: docs/user-journeys/j80-kr-pipa-personal-info-cross-border-transfer/
microservice: cloud-iac
status: draft
date: 2026-05-21
---

# IP-journey-j80: KR PIPA cross-border cell infrastructure

## A. Problem
J80 needs cloud-iac to prove that a KR-PIPA transfer cell is declared, rendered, validated, applied, and rolled back without mixing provider credentials with encryption key custody. The stamped version described generic rows; this IP closes the concrete gap between `docs/user-journeys/j80-kr-pipa-personal-info-cross-border-transfer/README.md` and the real cloud-iac control plane.

## B. Approach
Bind the journey to the existing renderer/validator/applier/registry/rollback split in `iac/manifest.json`. The renderer emits OpenTofu plus Helm/Kustomize material through `iac/contracts/openapi/cloud-iac.yaml`; the validator enforces KR pack residency using `iac/policy/data-residency.md`; the applier records signed apply evidence against `iac/slos/slsa-provenance-completeness.openslo.yaml`; rollback follows `iac/runbooks/rollback-orchestration.md`.

## C. Deliverables
- Extend the j80 request/response examples in `iac/contracts/openapi/cloud-iac.yaml`.
- Add async event examples to `iac/contracts/asyncapi/cloud-iac-events.yaml`.
- Bind renderer catalog evidence from `iac/catalog/oya-cloud-iac-iac-renderer-kernel.yaml`.
- Bind validator catalog evidence from `iac/catalog/oya-cloud-iac-iac-validator-kernel.yaml`.
- Reference KR residency and tenant isolation through `iac/policy/tenant-scope.cedar` and `iac/policy/data-residency.md`.

## D. Implementation
1. Add a j80 pack input shape to the OpenAPI examples with `tenant_id`, `pack_id`, `jurisdiction_code=KR`, `tofu_state_ref`, and `key_reference_mode`.
2. Require the renderer to mark provider credential references separately from encryption key references before plan creation.
3. Run validator checks for KR-PIPA residency, state backend locality, and SLSA provenance before apply.
4. Apply only through the iac-applier bounded context and emit the existing cloud-iac apply event stream.
5. Register the resulting cell state in the iac-registry path and expose the state ref to tenancy, not raw provider metadata.
6. Validate rollback by replaying `iac/runbooks/rollback-orchestration.md` with a stale KR pack input.

## E. Acceptance
- `iac/contracts/openapi/cloud-iac.yaml` has j80 examples for render, validate, apply, rollback.
- `iac/policy/data-residency.md` is cited by the validator acceptance text.
- `iac/slos/iac-apply-latency.openslo.yaml` and `iac/slos/slsa-provenance-completeness.openslo.yaml` are named in the verification bundle.
- No apply succeeds without a tenant-scoped Cedar decision and an audit-chain event.

## F. Evidence
- Journey: `docs/user-journeys/j80-kr-pipa-personal-info-cross-border-transfer/README.md`.
- Service architecture: `iac/ARCHITECTURE.md`.
- Competitive bar: `iac/competitor-parity-matrix.md`.

## G. Counterparts
| Counterpart | Gap closed |
|---|---|
| Terraform Cloud | Adds tenant-pack residency and KR-specific OpenTofu validation before state mutation. |
| ArgoCD / Flux | Keeps GitOps reconciliation but adds cloud-iac apply provenance and Cedar tenant scope. |
| Spacelift / Env0 | Matches policy-gated IaC orchestration while retaining self-hosted OpenTofu and KR residency evidence. |

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `iac/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/proto/cloud-iac.proto`, `iac/IP-journey-j80-cell-infra-declarative.md`.
