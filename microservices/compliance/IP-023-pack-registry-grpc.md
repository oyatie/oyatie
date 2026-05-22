---
ip_id: IP-023
microservice: compliance
bounded_context: pack-registry
layer: api
status: planned
related_adrs: [ADR-0253, ADR-0258, ADR-0251, ADR-0243]
---

# IP-023 — pack-registry gRPC surface

## A. Problem

Internal callers such as Foundry validators, tenant provisioning, and cell certification need a typed pack-registry API, but `contracts/compliance.proto` currently exposes only `RecordEvidence`. Without gRPC, teams will either parse pack docs or call REST paths intended for auditors. That creates drift between pack lifecycle, pack subscription, and evidence generation.

## B. Approach

Extend `contracts/compliance.proto` with a `PackRegistry` service backed by IP-016/IP-017. gRPC is for trusted internal callers over SPIFFE/mTLS; REST auditor surfaces remain read-only and engagement scoped. The service exposes pack manifest read, publish-state read, tenant subscription request, and pack requirement query.

## C. Deliverables

| Artifact | Change |
|---|---|
| `microservices/compliance/contracts/compliance.proto` | add `PackRegistry` service and messages |
| `microservices/compliance/catalog/api-asyncapi.yaml` and `api-rest.yaml` | cross-reference protocol split where needed |
| `microservices/compliance/policy/action-authorization.cedar` | authorize publish/subscribe actions |
| `microservices/compliance/policy/pack-overlay-authorization.cedar` | authorize tenant pack subscription facts |

## D. Implementation

1. Add proto package comments declaring SemVer compatibility and tenant metadata requirements.
2. Define `GetPackManifest`, `ListPublishedPacks`, `GetPackRequirements`, `SubscribeTenantToPack`, and `GetTenantPackSubscriptions`.
3. Include fields: `tenant_id`, `pack_id`, `version`, `framework_ids`, `artifact_kind_ids`, `data_classes`, `residency_rules`, and `audit_event`.
4. Require SPIFFE mTLS plus Cedar action authorization before domain command calls.
5. Map domain errors to explicit gRPC status: `InvalidArgument`, `FailedPrecondition`, `PermissionDenied`, `Unavailable`.
6. Emit audit events for mutating calls and denied subscription attempts.
7. Add contract tests for proto compatibility, tenant metadata missing, denied subscription, stale version, and published manifest read.

## E. Acceptance

- `contracts/compliance.proto` exposes pack registry without weakening `ComplianceEvidence.RecordEvidence`.
- Mutating gRPC calls require Cedar and produce audit events.
- External auditors cannot use this internal gRPC surface for arbitrary payload reads.
- Existing REST contract remains compatible.

## F. Evidence

- `microservices/compliance/contracts/compliance.proto` is the current gRPC authority.
- `microservices/compliance/manifest.json` lists pack ids and bounded contexts.
- `microservices/compliance/competitor-parity-matrix.md` shows commercial counterparts expose framework configuration APIs, but Oyatie keeps subscription policy in Cedar.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| Vanta / Drata | Provides internal framework/pack APIs while avoiding SaaS coupling. |
| AWS Audit Manager | Mirrors framework-management API discipline for operator-owned packs. |
| ServiceNow GRC | Narrows platform integration parity through typed gRPC instead of manual exports. |

## H. Non-goals and handoff boundaries

- Do not make gRPC the external auditor surface; IP-020 remains the read-only auditor REST API.
- Do not allow unauthenticated local calls; SPIFFE/mTLS and Cedar still apply.
- Do not expose raw pack authoring text; return typed manifest fields and refs.
- Do not let gRPC subscription skip domain conflict checks.
- Do not break existing `ComplianceEvidence.RecordEvidence` compatibility.

## I. Fixture set

- `get_published_pack_manifest.grpc.json` proves read shape.
- `subscribe_missing_tenant_metadata.grpc.json` proves metadata requirement.
- `subscribe_residency_denied.grpc.json` proves Cedar/domain denial.
- `list_published_packs.grpc.json` proves roster behavior.
- `proto_backward_compat_record_evidence.grpc.json` proves existing service compatibility.

## J. Launch blockers

- gRPC calls work without SPIFFE/mTLS metadata.
- Subscription mutation bypasses IP-017 domain checks.
- Proto responses omit audit event ids on mutating calls.
- Existing `RecordEvidence` consumers require a breaking migration.
- External auditor credentials can invoke pack mutation methods.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/compliance/IP-023-pack-registry-grpc.md` matched `asyncapi, .proto`; contract files `microservices/compliance/contracts/openapi.yaml, microservices/compliance/contracts/asyncapi.yaml, microservices/compliance/contracts/compliance.proto`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.
