---
doc_class: IP
ip_id: IP-019-sdk-client-generation
microservice: itsm
status: rewritten-wave-15-ip-substance
date: 2026-05-21
owner_team: axis-itsm + sdk-platform
counterparts: [ServiceNow ITSM, Jira Service Management, Freshservice]
source_artifacts:
  - microservices/itsm/contracts/openapi-v1.yaml
  - microservices/itsm/contracts/itsm-v1.proto
  - microservices/itsm/contracts/asyncapi-v1.yaml
  - microservices/itsm/src/lib.rs
---

# IP-019 ITSM SDK Client Generation

## A. Problem
Integrators expect ServiceNow/Jira/Freshservice-style client SDKs, but generated clients must not hide tenant id, Cedar purpose, data class, or DealSet settlement. The stamped IP did not name contracts or target clients.

This IP defines SDK generation from the three concrete ITSM contract files.

## B. Approach
Generate SDKs from canonical contracts, not hand-written samples:

| Contract | SDK responsibility |
|---|---|
| `contracts/openapi-v1.yaml` | public REST client for tenant apps |
| `contracts/itsm-v1.proto` | internal gRPC client for sibling µservices |
| `contracts/asyncapi-v1.yaml` | event consumer/publisher types |

Every generated method must expose required context fields rather than filling them from globals.

## C. Deliverables
- SDK generation target definitions for Rust first, other languages only after contract stability.
- Contract lint that rejects missing `tenant_id`, `principal_id`, `purpose`, or `data_class`.
- Generated client docs showing `OpenIncident`, `RecomputeSla`, and `ApproveChange` examples.
- Tests that generated request builders cannot omit tenant id.
- Compatibility policy for versioned ITSM contracts.

## D. Implementation
1. Validate OpenAPI, proto, and AsyncAPI contract files before generation.
2. Generate Rust client types into the established SDK output path when that path exists; do not invent a new repo layout here.
3. Ensure request builders require tenant id, principal id, purpose, and data class at construction.
4. For gRPC, use named RPCs from IP-007 once landed.
5. For events, generate typed event structs matching IP-006 event families.
6. Add snapshot tests for generated type names and required fields.
7. Add docs warning that ServiceNow sys_id/Jira issue key/Freshservice id are aliases only.
8. Version SDK artifacts with contract semver and changelog entries.

## E. Acceptance
- Contract validation runs before SDK generation.
- SDK examples compile once generated.
- Required tenant and policy context cannot be silently defaulted.
- Source-system ids are represented as aliases, not authorization fields.

## F. Evidence
- `src/lib.rs` publishes constants for OpenAPI, gRPC, and AsyncAPI contract paths.
- `contracts/openapi-v1.yaml`, `contracts/itsm-v1.proto`, and `contracts/asyncapi-v1.yaml` exist.
- ADR-0253 governs transport; ADR-0244 governs tenant context.

## G. Counterparts
| Counterpart | Gap closed by this IP |
|---|---|
| ServiceNow Table/API SDKs | SDKs preserve Oyatie tenant/policy context |
| Jira Service Management REST clients | Generated clients expose purpose/data class explicitly |
| Freshservice API clients | Source-system ids remain migration aliases |

## H. Cold-start buildability notes
- Run contract validation before client generation.
- Start with Rust SDK output only.
- Require tenant id in request-builder constructors.
- Keep generated names aligned with proto RPC names from IP-007.
- Generate event types only after IP-006 event names are stable.
- Add snapshot tests for required fields.
- Do not invent SDK output directories before repo convention is confirmed.
- Keep source-system ids as aliases in example code.
- Version clients by contract semver.
- Record unsupported languages as follow-up.
- Preserve manual examples until generated examples compile.
- Keep public REST and internal gRPC clients separate.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/itsm/contracts/asyncapi-v1.yaml`, `microservices/itsm/contracts/itsm-v1.proto`, `microservices/itsm/contracts/local-asyncapi-v1.yaml`, `microservices/itsm/contracts/local-openapi-v1.yaml`, `microservices/itsm/contracts/local-operations-v1.proto`, `microservices/itsm/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`openapi`, `.proto`, `asyncapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.
